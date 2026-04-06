//! ファイル情報・EXIF・主要色

use image::RgbaImage;
use std::collections::HashMap;
use std::fs;
use std::io::BufReader;
use std::path::Path;

use exif::{In, Reader, Tag};

use crate::color_theory::{lab_from_srgb, srgb_u8_from_lab, Lab};

/// 支配色推定で目標とするサンプル数（概算）。`step = sqrt(pixels / N)` で間引き、大画像でも点が薄くなりすぎないようにする。
const DOMINANT_TARGET_SAMPLES: u64 = 100_000;

/// Lab 空間 k-means の最大反復回数。
const KMEANS_MAX_ITER: usize = 24;

#[inline]
fn lab_dist2(l1: f64, a1: f64, b1: f64, l2: f64, a2: f64, b2: f64) -> f64 {
    let dl = l1 - l2;
    let da = a1 - a2;
    let db = b1 - b2;
    dl * dl + da * da + db * db
}

/// 透明でないピクセルを間引き、(L\*, a\*, b\*, R, G, B) の列とサンプル総数を返す。
fn collect_dominant_samples(
    rgba: &RgbaImage,
    step: usize,
) -> (Vec<(f64, f64, f64, u8, u8, u8)>, u64) {
    let (w, h) = rgba.dimensions();
    let mut points = Vec::new();
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
            points.push((lab.l, lab.a, lab.b, r, g, b));
        }
    }
    let n = points.len() as u64;
    (points, n)
}

/// 等間隔にサンプルを選び初期重心にする（決定的）。
fn kmeans_initial_centroids(
    points: &[(f64, f64, f64, u8, u8, u8)],
    k: usize,
) -> Vec<(f64, f64, f64)> {
    let n = points.len();
    let k = k.min(n).max(1);
    if k == n {
        return points.iter().map(|p| (p.0, p.1, p.2)).collect();
    }
    (0..k)
        .map(|j| {
            let i = j * (n - 1) / (k - 1).max(1);
            (points[i].0, points[i].1, points[i].2)
        })
        .collect()
}

/// Lab 距離で k-means。収束後の **Lab 重心**とクラスタ点数を返す（代表色は `color_theory::srgb_u8_from_lab` で sRGB に落とす）。
fn kmeans_lab_centroids_with_counts(
    points: &[(f64, f64, f64, u8, u8, u8)],
    k: usize,
) -> Vec<(f64, f64, f64, u64)> {
    let n = points.len();
    let k = k.min(n).max(1);
    let mut centroids = kmeans_initial_centroids(points, k);
    let mut assignments = vec![0usize; n];

    for _ in 0..KMEANS_MAX_ITER {
        let mut changed = false;
        for i in 0..n {
            let mut best_j = 0usize;
            let mut best_d = f64::INFINITY;
            for j in 0..k {
                let c = centroids[j];
                let d = lab_dist2(points[i].0, points[i].1, points[i].2, c.0, c.1, c.2);
                if d < best_d {
                    best_d = d;
                    best_j = j;
                }
            }
            if assignments[i] != best_j {
                changed = true;
            }
            assignments[i] = best_j;
        }

        let mut sum_l = vec![0.0_f64; k];
        let mut sum_a = vec![0.0_f64; k];
        let mut sum_b = vec![0.0_f64; k];
        let mut counts = vec![0usize; k];
        for i in 0..n {
            let j = assignments[i];
            sum_l[j] += points[i].0;
            sum_a[j] += points[i].1;
            sum_b[j] += points[i].2;
            counts[j] += 1;
        }

        for j in 0..k {
            if counts[j] == 0 {
                let idx = (j.wrapping_mul(7919).wrapping_add(n / 2)) % n;
                centroids[j] = (points[idx].0, points[idx].1, points[idx].2);
            } else {
                centroids[j] = (
                    sum_l[j] / counts[j] as f64,
                    sum_a[j] / counts[j] as f64,
                    sum_b[j] / counts[j] as f64,
                );
            }
        }

        if !changed {
            break;
        }
    }

    let mut counts = vec![0u64; k];
    for i in 0..n {
        counts[assignments[i]] += 1;
    }

    (0..k)
        .map(|j| {
            let (l, a, b) = centroids[j];
            (l, a, b, counts[j])
        })
        .collect()
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

/// 支配色（主要色）の推定。目標サンプル数に合わせた間引きのうえで、**Lab 空間の k-means**（k = `max_colors`、最大反復 `KMEANS_MAX_ITER`）でクラスタし、各クラスタの **Lab 重心を sRGB に逆変換**した色を代表色とする。
pub fn dominant_colors(rgba: &RgbaImage, max_colors: usize) -> Vec<(u8, u8, u8, f32)> {
    let (w, h) = rgba.dimensions();
    if w == 0 || h == 0 || max_colors == 0 {
        return Vec::new();
    }
    let pixels = (w as u64) * (h as u64);
    let step = ((pixels as f64 / DOMINANT_TARGET_SAMPLES as f64).sqrt().ceil() as u64).max(1) as usize;

    let (points, sampled) = collect_dominant_samples(rgba, step);
    if sampled == 0 {
        return Vec::new();
    }

    let k = max_colors.min(points.len()).max(1);
    let clusters = kmeans_lab_centroids_with_counts(&points, k);

    let mut merged_pct: HashMap<(u8, u8, u8), f32> = HashMap::new();
    for (l, a, b, cnt) in clusters.into_iter().filter(|(_, _, _, c)| *c > 0) {
        let lab = Lab { l, a, b };
        let (cr, cg, cb) = srgb_u8_from_lab(&lab);
        let pct = 100.0 * cnt as f32 / sampled as f32;
        *merged_pct.entry((cr, cg, cb)).or_insert(0.0) += pct;
    }
    let mut v: Vec<(u8, u8, u8, f32)> = merged_pct
        .into_iter()
        .map(|((r, g, b), pct)| (r, g, b, pct))
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
