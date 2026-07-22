//! 画像パスから分析結果 DTO を構築

use std::io::Cursor;
use std::path::Path;

use image::{GenericImageView, ImageFormat, RgbaImage};
use serde::Serialize;

use crate::color_theory::{lab_from_srgb, wcag_contrast_rgb};
use crate::harmony;
use crate::meta;
use crate::palette_match;
use crate::theory::{self, TheoryBlock};

const PREVIEW_MAX_SIDE: u32 = 1200;

/// 分析結果 JSON（`Analysis`）のスキーマ版。キー追加や、**同じフィールドでも数値の意味が変わる大きなアルゴリズム変更**のときに上げる。
///
/// - **4**（現在）: CIEDE2000 パレット照合、支配色（farthest-first・Lab 重心代表・約 15 万サンプル目標・反復 32 等）、調和スコア（重み付き類似・10° アンカー・テンプレ別 σ）を反映。キー構造は 3 と同じ。
/// - **3**: 従来の分析 JSON 形式（詳細は CHANGELOG・Git 履歴参照）。
pub const ANALYSIS_SCHEMA_VERSION: u32 = 4;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExifLineDto {
    pub label: String,
    pub value: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DominantDto {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub pct: f32,
    pub hex: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PaletteMatchDto {
    pub dom_r: u8,
    pub dom_g: u8,
    pub dom_b: u8,
    pub pct: f32,
    pub swatch_name: String,
    pub sw_r: u8,
    pub sw_g: u8,
    pub sw_b: u8,
    pub sw_hex: String,
    pub delta_e: f64,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WcagDominantPairDto {
    pub r1: u8,
    pub g1: u8,
    pub b1: u8,
    pub r2: u8,
    pub g2: u8,
    pub b2: u8,
    pub contrast_ratio: f64,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GistLineDto {
    pub text: String,
    /// UI 用: `mono` | `label` | `body` | `foot`
    pub role: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisGistDto {
    pub lines: Vec<GistLineDto>,
    /// コピー・JSON 連携用（`lines` の `text` を改行で連結）
    pub gist_ja: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Analysis {
    pub schema_version: u32,
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub file_size_bytes: Option<u64>,
    pub file_size_display: Option<String>,
    pub modified_display: Option<String>,
    pub exif: Vec<ExifLineDto>,
    pub preview_jpeg_base64: String,
    pub preview_width: u32,
    pub preview_height: u32,
    pub preview_bg_dark: bool,
    pub dominants: Vec<DominantDto>,
    pub open_color_matches: Vec<PaletteMatchDto>,
    pub tailwind_matches: Vec<PaletteMatchDto>,
    pub wcag_dominant_pair: Option<WcagDominantPairDto>,
    /// PCCS 風トーン・色相・概論対応メモ（非公式近似）
    pub theory: TheoryBlock,
    /// 互換性のため保持する実験的な旧色相配置値。調和型への適合判定には使わない。
    pub harmony_scores: Vec<harmony::HarmonyScoreDto>,
    /// ひと目サマリ（支配色 1〜3 位のパレット近さ等。実験的な色相配置値は含めない）
    pub gist: AnalysisGistDto,
}

fn rgb_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

fn average_luminance(rgba: &RgbaImage) -> f32 {
    let (w, h) = rgba.dimensions();
    if w == 0 || h == 0 {
        return 0.5;
    }
    let step_x = (w / 48).max(1);
    let step_y = (h / 48).max(1);
    let mut sum = 0f64;
    let mut n = 0u64;
    for y in (0..h).step_by(step_y as usize) {
        for x in (0..w).step_by(step_x as usize) {
            let p = rgba.get_pixel(x, y);
            let r = p[0] as f32 / 255.0;
            let g = p[1] as f32 / 255.0;
            let b = p[2] as f32 / 255.0;
            let l = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            sum += l as f64;
            n += 1;
        }
    }
    (sum / n.max(1) as f64) as f32
}

fn jpeg_preview_base64(img: &image::DynamicImage) -> Result<(String, u32, u32), String> {
    let (w, h) = img.dimensions();
    let thumb = if w.max(h) <= PREVIEW_MAX_SIDE {
        img.clone()
    } else {
        img.thumbnail(PREVIEW_MAX_SIDE, PREVIEW_MAX_SIDE)
    };
    let rgb = thumb.to_rgb8();
    let (tw, th) = rgb.dimensions();
    let mut buf = Vec::new();
    let mut c = Cursor::new(&mut buf);
    rgb.write_to(&mut c, ImageFormat::Jpeg)
        .map_err(|e| e.to_string())?;
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &buf);
    Ok((b64, tw, th))
}

pub fn analyze_path(path_str: &str) -> Result<Analysis, String> {
    let path = Path::new(path_str);
    let img = image::open(path).map_err(|e| e.to_string())?;
    let (width, height) = img.dimensions();
    let rgba = img.to_rgba8();

    let snap = meta::load_file_snapshot(path);
    let exif_lines = meta::read_exif_lines(path);
    let exif: Vec<ExifLineDto> = exif_lines
        .into_iter()
        .map(|(label, value)| ExifLineDto { label, value })
        .collect();

    let preview_bg_dark = average_luminance(&rgba) >= 0.52;
    let palette = meta::dominant_colors(&rgba, 8);
    let open_raw = palette_match::match_dominants(&palette, palette_match::open_color_palette());
    let tail_raw =
        palette_match::match_dominants(&palette, palette_match::tailwind_subset_palette());

    let dominants: Vec<DominantDto> = palette
        .iter()
        .map(|&(r, g, b, pct)| DominantDto {
            r,
            g,
            b,
            pct,
            hex: rgb_hex(r, g, b),
        })
        .collect();

    let open_color_matches: Vec<PaletteMatchDto> = open_raw
        .into_iter()
        .map(|m| PaletteMatchDto {
            dom_r: m.dom_r,
            dom_g: m.dom_g,
            dom_b: m.dom_b,
            pct: m.pct,
            swatch_name: m.swatch_name,
            sw_r: m.sw_r,
            sw_g: m.sw_g,
            sw_b: m.sw_b,
            sw_hex: rgb_hex(m.sw_r, m.sw_g, m.sw_b),
            delta_e: m.delta_e,
        })
        .collect();

    let tailwind_matches: Vec<PaletteMatchDto> = tail_raw
        .into_iter()
        .map(|m| PaletteMatchDto {
            dom_r: m.dom_r,
            dom_g: m.dom_g,
            dom_b: m.dom_b,
            pct: m.pct,
            swatch_name: m.swatch_name,
            sw_r: m.sw_r,
            sw_g: m.sw_g,
            sw_b: m.sw_b,
            sw_hex: rgb_hex(m.sw_r, m.sw_g, m.sw_b),
            delta_e: m.delta_e,
        })
        .collect();

    let wcag_dominant_pair = if palette.len() >= 2 {
        let (r1, g1, b1, _) = palette[0];
        let (r2, g2, b2, _) = palette[1];
        Some(WcagDominantPairDto {
            r1,
            g1,
            b1,
            r2,
            g2,
            b2,
            contrast_ratio: wcag_contrast_rgb(r1, g1, b1, r2, g2, b2),
        })
    } else {
        None
    };

    let (preview_jpeg_base64, preview_width, preview_height) = jpeg_preview_base64(&img)?;

    let theory = theory::build_theory_block(&palette);

    let weighted_hues: Vec<(f64, f64)> = palette
        .iter()
        .filter_map(|&(r, g, b, pct)| {
            let lab = lab_from_srgb(r, g, b);
            let c = theory::chroma_star(lab.a, lab.b);
            if c < 12.0 {
                return None;
            }
            Some((theory::hue_deg_from_ab(lab.a, lab.b), pct as f64))
        })
        .collect();
    let harmony_scores = harmony::harmony_scores(&weighted_hues);
    let gist = build_analysis_gist(&theory, &open_color_matches, &tailwind_matches);

    Ok(Analysis {
        schema_version: ANALYSIS_SCHEMA_VERSION,
        path: path_str.to_string(),
        width,
        height,
        file_size_bytes: snap.as_ref().map(|s| s.size_bytes),
        file_size_display: snap.as_ref().map(|s| meta::format_file_size(s.size_bytes)),
        modified_display: snap.map(|s| s.modified_display),
        exif,
        preview_jpeg_base64,
        preview_width,
        preview_height,
        preview_bg_dark,
        dominants,
        open_color_matches,
        tailwind_matches,
        wcag_dominant_pair,
        theory,
        harmony_scores,
        gist,
    })
}

fn build_analysis_gist(
    theory: &TheoryBlock,
    open: &[PaletteMatchDto],
    tail: &[PaletteMatchDto],
) -> AnalysisGistDto {
    let mut lines: Vec<GistLineDto> = Vec::new();

    if let Some(ref s) = theory.dominant_hue_summary_ja {
        if !s.is_empty() {
            lines.push(GistLineDto {
                text: s.clone(),
                role: "mono".to_string(),
            });
        }
    }

    let n = open.len().min(tail.len()).min(3);
    if n > 0 {
        lines.push(GistLineDto {
            text: "既知パレットとの近さ（色差 ΔE2000・支配色 1〜3 位）".to_string(),
            role: "label".to_string(),
        });
        for i in 0..n {
            let rank = i + 1;
            let oc = &open[i];
            lines.push(GistLineDto {
                text: format!(
                    "Open Color（支配色 {} 位）: 「{}」が最も近い · ΔE2000 {:.1}",
                    rank, oc.swatch_name, oc.delta_e
                ),
                role: "body".to_string(),
            });
            let tw = &tail[i];
            lines.push(GistLineDto {
                text: format!(
                    "Tailwind（支配色 {} 位）: 「{}」が最も近い · ΔE2000 {:.1}",
                    rank, tw.swatch_name, tw.delta_e
                ),
                role: "body".to_string(),
            });
        }
        lines.push(GistLineDto {
            text: "理論的な「正しいパレット」とは別軸です。全支配色の一覧は下の各セクションを参照してください。"
                .to_string(),
            role: "foot".to_string(),
        });
    }

    let gist_ja = lines
        .iter()
        .map(|l| l.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    AnalysisGistDto { lines, gist_ja }
}

pub fn sample_pixel(path_str: &str, x: u32, y: u32) -> Result<Option<(u8, u8, u8)>, String> {
    let path = Path::new(path_str);
    let img = image::open(path).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    if x >= w || y >= h {
        return Ok(None);
    }
    let p = rgba.get_pixel(x, y);
    if p[3] < 16 {
        return Ok(None);
    }
    Ok(Some((p[0], p[1], p[2])))
}
