//! ポジティブ/ネガティブシェイプ分析
//!
//! ## アルゴリズム（背景除去型）
//!
//! 1. Canny エッジ検出
//! 2. エッジを膨張（`DILATION_RADIUS` px）して輪郭の隙間を塞ぐ
//! 3. 膨張エッジを「バリア」として、4隅をシードに連結成分ラベリング
//!    → 4隅と繋がる領域 = ネガティブ（背景・余白）
//!    → どの隅とも繋がらない領域 = ポジティブ（形のある物体）
//!
//! 輝度ではなくシェイプの「閉じ具合」で判定するため、
//! 白い雲・暗い物体・透明背景いずれにも対応できる。

use std::collections::HashSet;
use std::io::Cursor;
use std::path::Path;

use base64::Engine;
use image::{GenericImageView, GrayImage, ImageFormat, Luma, RgbImage};
use imageproc::edges::canny;
use imageproc::distance_transform::Norm;
use imageproc::morphology::dilate;
use imageproc::region_labelling::{connected_components, Connectivity};
use serde::Serialize;

/// シェイプ分析の処理サイズ上限（長辺）。
const SHAPE_MAX_SIDE: u32 = 700;

/// Canny エッジ検出の閾値（低/高）。
const CANNY_LOW: f32 = 12.0;
const CANNY_HIGH: f32 = 35.0;

/// エッジ膨張の半径（px）。大きいほど輪郭の隙間を塞ぎやすいが、細かいシェイプが潰れる。
/// 700px 画像では 10px 程度が雲・イラスト・写真に対してバランスが良い。
const DILATION_RADIUS: u8 = 10;

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
/// 白 = ポジティブ、黒 = ネガティブ（アート練習の慣習に合わせる）。
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

    // 処理サイズにリサイズ
    let proc = if img.dimensions().0.max(img.dimensions().1) > SHAPE_MAX_SIDE {
        img.thumbnail(SHAPE_MAX_SIDE, SHAPE_MAX_SIDE)
    } else {
        img.clone()
    };
    let (pw, ph) = proc.dimensions();
    let total = (pw * ph) as f32;

    // グレースケール → Canny エッジ検出
    let gray = proc.to_luma8();
    let edges: GrayImage = canny(&gray, CANNY_LOW, CANNY_HIGH);

    // エッジ密度（統計値として保持）
    let edge_count = edges.pixels().filter(|p| p[0] > 0).count() as f32;
    let edge_density = edge_count / total;

    // エッジを膨張して輪郭の隙間を塞ぐ（Chebyshev 距離 = 正方形カーネル）
    let dilated: GrayImage = dilate(&edges, Norm::LInf, DILATION_RADIUS);

    // 膨張エッジをバリアとして、非エッジ領域を連結成分ラベリング
    // background = Luma([0]) → エッジ部分（255）はラベル 0 として扱われ区切りになる
    let open_space = GrayImage::from_fn(pw, ph, |x, y| {
        // 膨張エッジ = 0（バリア）、それ以外 = 255（開放空間）
        Luma([if dilated.get_pixel(x, y)[0] > 0 { 0u8 } else { 255u8 }])
    });
    let labeled = connected_components(&open_space, Connectivity::Four, Luma([0u8]));

    // 4隅のラベルを収集 → これらが背景（ネガティブ）のラベル
    let corner_labels: HashSet<u32> = [
        labeled.get_pixel(0, 0)[0],
        labeled.get_pixel(pw - 1, 0)[0],
        labeled.get_pixel(0, ph - 1)[0],
        labeled.get_pixel(pw - 1, ph - 1)[0],
    ]
    .iter()
    .copied()
    .filter(|&l| l != 0) // ラベル 0 はバリア上のピクセル（除外）
    .collect();

    // ポジティブマップ：ラベルが0でなく、かつ背景ラベルでもない領域
    let positive_map: Vec<bool> = (0..ph)
        .flat_map(|y| (0..pw).map(move |x| (x, y)))
        .map(|(x, y)| {
            let label = labeled.get_pixel(x, y)[0];
            label != 0 && !corner_labels.contains(&label)
        })
        .collect();

    let positive_count = positive_map.iter().filter(|&&v| v).count() as f32;
    let positive_area_pct = 100.0 * positive_count / total;
    let negative_area_pct = 100.0 - positive_area_pct;

    // ポジティブ領域の連結成分数を再カウント（スタークビュー画像から）
    let stark_img = build_stark(&positive_map, pw, ph);
    let labeled_pos = connected_components(&stark_img, Connectivity::Eight, Luma([0u8]));
    let region_count = labeled_pos.pixels().map(|p| p[0]).max().unwrap_or(0);

    // 複雑度ラベル（エッジ密度ベース）
    let complexity_ja = if edge_density < 0.04 {
        "シンプル"
    } else if edge_density < 0.12 {
        "中程度"
    } else {
        "複雑"
    }
    .to_string();

    let stark_base64 = encode_gray_png(&stark_img)?;

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

    fn save_test_png(img: &RgbaImage) -> std::path::PathBuf {
        let path = std::env::temp_dir().join("shape_test.png");
        img.save(&path).expect("save test png");
        path
    }

    #[test]
    fn returns_valid_dto_for_simple_image() {
        let mut img = RgbaImage::new(80, 80);
        for y in 0..80u32 {
            for x in 0..80u32 {
                let px = if x >= 20 && x < 60 && y >= 20 && y < 60 {
                    Rgba([220, 220, 220, 255])
                } else {
                    Rgba([30, 30, 30, 255])
                };
                img.put_pixel(x, y, px);
            }
        }
        let path = save_test_png(&img);
        let dto = analyze_shape_path(path.to_str().unwrap()).expect("analyze");
        assert!(dto.positive_area_pct >= 0.0 && dto.positive_area_pct <= 100.0);
        assert!((dto.positive_area_pct + dto.negative_area_pct - 100.0).abs() < 0.1);
        assert!(dto.edge_density >= 0.0 && dto.edge_density <= 1.0);
        assert!(!dto.stark_base64.is_empty());
        assert!(!dto.overlay_base64.is_empty());
    }

    #[test]
    fn solid_color_image_is_all_negative() {
        // 単色画像はエッジなし → 全てネガティブ（四隅から繋がる）
        let img = RgbaImage::from_pixel(60, 60, Rgba([100u8, 149, 237, 255]));
        let path = save_test_png(&img);
        let dto = analyze_shape_path(path.to_str().unwrap()).expect("analyze");
        assert!(
            dto.positive_area_pct < 5.0,
            "単色画像のポジ率は小さいはず: {}",
            dto.positive_area_pct
        );
    }
}
