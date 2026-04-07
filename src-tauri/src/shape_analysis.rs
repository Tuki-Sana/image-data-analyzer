//! ポジティブ/ネガティブシェイプ分析
//!
//! Canny エッジ検出 → ガウシアンブラーで局所エッジ密度マップを生成し、
//! 密度の高い領域（モノが「ある」場所）をポジティブシェイプとして判定する。
//! 輝度ではなく「内容の密集度」を基準にするため、背景色に依存しない。

use std::io::Cursor;
use std::path::Path;

use base64::Engine;
use image::{GenericImageView, GrayImage, ImageFormat, Luma, RgbImage};
use imageproc::edges::canny;
use imageproc::filter::gaussian_blur_f32;
use imageproc::region_labelling::{connected_components, Connectivity};
use serde::Serialize;

/// シェイプ分析の処理サイズ上限（長辺）。分析精度とパフォーマンスのバランス。
const SHAPE_MAX_SIDE: u32 = 700;

/// Canny エッジ検出の閾値（低/高）。
const CANNY_LOW: f32 = 12.0;
const CANNY_HIGH: f32 = 35.0;

/// 密度マップのガウシアン σ（ピクセル）。大きいほど広い範囲を「密集あり」と判定する。
const DENSITY_SIGMA: f32 = 14.0;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ShapeAnalysisDto {
    /// ポジティブシェイプ推定面積比（%）
    pub positive_area_pct: f32,
    /// ネガティブシェイプ推定面積比（%）
    pub negative_area_pct: f32,
    /// 全ピクセルに占めるエッジピクセルの割合（0〜1）
    pub edge_density: f32,
    /// ポジティブ連結領域の数
    pub region_count: u32,
    /// 形状複雑度の目安（シンプル / 中程度 / 複雑）
    pub complexity_ja: String,
    /// 白黒スタークビュー PNG（base64）
    pub stark_base64: String,
    /// カラーオーバーレイ PNG（base64）。ポジ=暖色、ネガ=寒色
    pub overlay_base64: String,
    /// 処理に使ったリサイズ後の幅
    pub proc_width: u32,
    /// 処理に使ったリサイズ後の高さ
    pub proc_height: u32,
}

fn encode_gray_png(img: &GrayImage) -> Result<String, String> {
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&buf))
}

fn encode_rgb_png(img: &RgbImage) -> Result<String, String> {
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&buf))
}

/// ポジ/ネガマップからスタークビュー（白黒 PNG）を生成する。
fn build_stark(positive_map: &[bool], w: u32, h: u32) -> GrayImage {
    GrayImage::from_fn(w, h, |x, y| {
        if positive_map[(y * w + x) as usize] {
            Luma([255u8])
        } else {
            Luma([0u8])
        }
    })
}

/// ポジ/ネガマップと元画像からカラーオーバーレイ PNG を生成する。
/// - ポジティブ: 暖色（#FF6B35 = オレンジ）
/// - ネガティブ: 寒色（#4A90D9 = ブルー）
fn build_overlay(rgb: &RgbImage, positive_map: &[bool], w: u32, h: u32) -> RgbImage {
    const POS_R: f32 = 255.0;
    const POS_G: f32 = 107.0;
    const POS_B: f32 = 53.0;
    const POS_A: f32 = 0.50;

    const NEG_R: f32 = 74.0;
    const NEG_G: f32 = 144.0;
    const NEG_B: f32 = 217.0;
    const NEG_A: f32 = 0.38;

    RgbImage::from_fn(w, h, |x, y| {
        let orig = rgb.get_pixel(x, y);
        let (r, g, b) = (orig[0] as f32, orig[1] as f32, orig[2] as f32);
        let (cr, cg, cb, ca) = if positive_map[(y * w + x) as usize] {
            (POS_R, POS_G, POS_B, POS_A)
        } else {
            (NEG_R, NEG_G, NEG_B, NEG_A)
        };
        image::Rgb([
            (r * (1.0 - ca) + cr * ca).round().clamp(0.0, 255.0) as u8,
            (g * (1.0 - ca) + cg * ca).round().clamp(0.0, 255.0) as u8,
            (b * (1.0 - ca) + cb * ca).round().clamp(0.0, 255.0) as u8,
        ])
    })
}

pub fn analyze_shape_path(path_str: &str) -> Result<ShapeAnalysisDto, String> {
    let path = Path::new(path_str);
    let img = image::open(path).map_err(|e| e.to_string())?;
    let (orig_w, orig_h) = img.dimensions();

    // 処理サイズにリサイズ（大きい画像でも一定時間で処理できるよう）
    let proc = if orig_w.max(orig_h) > SHAPE_MAX_SIDE {
        img.thumbnail(SHAPE_MAX_SIDE, SHAPE_MAX_SIDE)
    } else {
        img.clone()
    };
    let (pw, ph) = proc.dimensions();
    let total = (pw * ph) as f32;

    // グレースケール変換 → Canny エッジ検出
    let gray = proc.to_luma8();
    let edges: GrayImage = canny(&gray, CANNY_LOW, CANNY_HIGH);

    // エッジ密度（全体の何%がエッジか）
    let edge_count = edges.pixels().filter(|p| p[0] > 0).count() as f32;
    let edge_density = edge_count / total;

    // ガウシアンブラーで「局所エッジ密度マップ」を生成
    let density: GrayImage = gaussian_blur_f32(&edges, DENSITY_SIGMA);

    // 密度マップの平均値を基準に閾値を決定
    // 平均の 40% 以上の密度領域をポジティブと判定
    // 最低閾値 6 を設けて全黒画像でのフォールバックにする
    let density_vals: Vec<f32> = density.pixels().map(|p| p[0] as f32).collect();
    let mean_density: f32 = density_vals.iter().sum::<f32>() / density_vals.len() as f32;
    let threshold = (mean_density * 0.40).max(6.0_f32);

    // ポジティブマップを構築
    let positive_map: Vec<bool> = density_vals.iter().map(|&v| v > threshold).collect();
    let positive_count = positive_map.iter().filter(|&&v| v).count() as f32;
    let positive_area_pct = 100.0 * positive_count / total;
    let negative_area_pct = 100.0 - positive_area_pct;

    // ポジティブ領域の連結成分数（= シェイプのまとまりの数）
    let binary = GrayImage::from_fn(pw, ph, |x, y| {
        if positive_map[(y * pw + x) as usize] {
            Luma([255u8])
        } else {
            Luma([0u8])
        }
    });
    let labeled = connected_components(&binary, Connectivity::Eight, Luma([0u8]));
    let region_count = labeled.pixels().map(|p| p[0]).max().unwrap_or(0);

    // 複雑度ラベル（エッジ密度ベース）
    let complexity_ja = if edge_density < 0.04 {
        "シンプル"
    } else if edge_density < 0.12 {
        "中程度"
    } else {
        "複雑"
    }
    .to_string();

    // スタークビュー（白黒 PNG）
    let stark_img = build_stark(&positive_map, pw, ph);
    let stark_base64 = encode_gray_png(&stark_img)?;

    // オーバーレイ（元画像 + 色マスク）
    let rgb = proc.to_rgb8();
    let overlay_img = build_overlay(&rgb, &positive_map, pw, ph);
    let overlay_base64 = encode_rgb_png(&overlay_img)?;

    Ok(ShapeAnalysisDto {
        positive_area_pct,
        negative_area_pct,
        edge_density,
        region_count,
        complexity_ja,
        stark_base64,
        overlay_base64,
        proc_width: pw,
        proc_height: ph,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    fn dummy_png_path() -> std::path::PathBuf {
        // テスト用に一時ファイルを作成
        let mut img = RgbaImage::new(80, 80);
        for y in 0..80 {
            for x in 0..80 {
                // 中央に白い四角、周囲は黒
                let px = if x >= 20 && x < 60 && y >= 20 && y < 60 {
                    Rgba([220, 220, 220, 255])
                } else {
                    Rgba([30, 30, 30, 255])
                };
                img.put_pixel(x, y, px);
            }
        }
        let path = std::env::temp_dir().join("shape_test.png");
        img.save(&path).expect("save test png");
        path
    }

    #[test]
    fn returns_valid_dto_for_simple_image() {
        let path = dummy_png_path();
        let dto = analyze_shape_path(path.to_str().unwrap()).expect("analyze");
        assert!(dto.positive_area_pct >= 0.0 && dto.positive_area_pct <= 100.0);
        assert!((dto.positive_area_pct + dto.negative_area_pct - 100.0).abs() < 0.1);
        assert!(dto.edge_density >= 0.0 && dto.edge_density <= 1.0);
        assert!(!dto.stark_base64.is_empty());
        assert!(!dto.overlay_base64.is_empty());
    }
}
