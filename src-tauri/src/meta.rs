//! ファイル情報・EXIF・主要色

use image::RgbaImage;
use std::collections::HashMap;
use std::fs;
use std::io::BufReader;
use std::path::Path;

use exif::{In, Reader, Tag};

use crate::color_theory::lab_from_srgb;

/// 支配色推定で目標とするサンプル数（概算）。`step = sqrt(pixels / N)` で間引き、大画像でも点が薄くなりすぎないようにする。
const DOMINANT_TARGET_SAMPLES: u64 = 100_000;

/// Lab 空間でのビン幅。知覚に近いマージの目安（フェーズ1: k-means 前の量子化）。
const LAB_BIN_L: f64 = 5.0;
const LAB_BIN_AB: f64 = 7.0;

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
struct LabBin(i32, i32, i32);

#[derive(Default)]
struct DominantBinAccum {
    sum_r: u64,
    sum_g: u64,
    sum_b: u64,
    count: u64,
}

fn lab_bin_index(l: f64, a: f64, b: f64) -> LabBin {
    LabBin(
        (l / LAB_BIN_L).floor() as i32,
        (a / LAB_BIN_AB).floor() as i32,
        (b / LAB_BIN_AB).floor() as i32,
    )
}

#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct FileSnapshot {
    pub size_bytes: u64,
    pub modified_display: String,
}

pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn load_file_snapshot(path: &Path) -> Option<FileSnapshot> {
    let m = fs::metadata(path).ok()?;
    let modified_display = m
        .modified()
        .ok()
        .and_then(|t| {
            let utc: chrono::DateTime<chrono::Utc> = t.into();
            Some(
                utc.with_timezone(&chrono::Local)
                    .format("%Y-%m-%d %H:%M")
                    .to_string(),
            )
        })
        .unwrap_or_else(|| "—".to_string());
    Some(FileSnapshot {
        size_bytes: m.len(),
        modified_display,
    })
}

pub fn read_exif_lines(path: &Path) -> Vec<(String, String)> {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    let mut buf = BufReader::new(file);
    let exif = match Reader::new().read_from_container(&mut buf) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let tags: &[(Tag, &str)] = &[
        (Tag::DateTimeOriginal, "撮影日時"),
        (Tag::DateTime, "日時"),
        (Tag::Make, "メーカー"),
        (Tag::Model, "機種"),
        (Tag::LensModel, "レンズ"),
        (Tag::Orientation, "向き"),
        (Tag::PhotographicSensitivity, "ISO"),
        (Tag::FNumber, "F値"),
        (Tag::ExposureTime, "露出時間"),
        (Tag::FocalLength, "焦点距離"),
        (Tag::ImageWidth, "EXIF幅"),
        (Tag::ImageLength, "EXIF高さ"),
    ];

    let mut out = Vec::new();
    for &(tag, label) in tags {
        if let Some(field) = exif.get_field(tag, In::PRIMARY) {
            let v = field.display_value().to_string();
            if !v.is_empty() {
                out.push((label.to_string(), v));
            }
        }
    }
    out
}

/// 支配色（主要色）の推定。フェーズ1: 目標サンプル数に合わせた間引き + **Lab 空間でのビン分割**後、各ビン内の **RGB 平均**を代表色とする（k-means ではない）。
pub fn dominant_colors(rgba: &RgbaImage, max_colors: usize) -> Vec<(u8, u8, u8, f32)> {
    let (w, h) = rgba.dimensions();
    if w == 0 || h == 0 || max_colors == 0 {
        return Vec::new();
    }
    let pixels = (w as u64) * (h as u64);
    let step = ((pixels as f64 / DOMINANT_TARGET_SAMPLES as f64).sqrt().ceil() as u64).max(1) as usize;

    let mut map: HashMap<LabBin, DominantBinAccum> = HashMap::new();
    let mut sampled = 0u64;
    for y in (0..h).step_by(step) {
        for x in (0..w).step_by(step) {
            let p = rgba.get_pixel(x, y);
            if p[3] < 16 {
                continue;
            }
            let r = p[0];
            let g = p[1];
            let b = p[2];
            let lab = lab_from_srgb(r, g, b);
            let key = lab_bin_index(lab.l, lab.a, lab.b);
            let acc = map.entry(key).or_default();
            acc.sum_r += u64::from(r);
            acc.sum_g += u64::from(g);
            acc.sum_b += u64::from(b);
            acc.count += 1;
            sampled += 1;
        }
    }
    if sampled == 0 {
        return Vec::new();
    }
    let mut v: Vec<(u8, u8, u8, f32)> = map
        .into_values()
        .map(|acc| {
            let n = acc.count.max(1);
            let cr = (acc.sum_r / n).min(255) as u8;
            let cg = (acc.sum_g / n).min(255) as u8;
            let cb = (acc.sum_b / n).min(255) as u8;
            let pct = 100.0 * acc.count as f32 / sampled as f32;
            (cr, cg, cb, pct)
        })
        .collect();
    v.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
    v.truncate(max_colors);
    v
}

#[cfg(test)]
mod dominant_tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    #[test]
    fn solid_red_one_dominant() {
        let img = RgbaImage::from_pixel(32, 32, Rgba([255, 0, 0, 255]));
        let d = dominant_colors(&img, 8);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].0, 255);
        assert_eq!(d[0].1, 0);
        assert_eq!(d[0].2, 0);
        assert!((d[0].3 - 100.0).abs() < 0.1);
    }

    #[test]
    fn two_equal_halves_two_dominants() {
        let mut img = RgbaImage::new(40, 40);
        for y in 0..40 {
            for x in 0..40 {
                let px = if x < 20 {
                    Rgba([0, 128, 255, 255])
                } else {
                    Rgba([255, 128, 0, 255])
                };
                img.put_pixel(x, y, px);
            }
        }
        let d = dominant_colors(&img, 8);
        assert!(d.len() >= 2);
        let sum_pct: f32 = d.iter().take(2).map(|t| t.3).sum();
        assert!(sum_pct > 95.0, "expected ~100% in top two, got {}", sum_pct);
    }
}
