//! CIELAB からの PCCS 風トーン・色相区分・色彩理論概論との対応メモ（公式 PCCS ではない）。

use crate::color_theory::lab_from_srgb;
use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DominantTheoryDto {
    pub hex: String,
    pub l_star: f64,
    pub c_star: f64,
    pub h_deg: f64,
    pub hue_region_ja: String,
    pub pccs_style_tone_ja: String,
    pub pccs_style_tone_id: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TheoryBlock {
    /// 利用上の注意（固定文）
    pub disclaimer_ja: String,
    /// 概論の見出しレベルで「何に相当するか」
    pub outline_mapping_ja: Vec<String>,
    /// 支配色ごとの L*C*h° と PCCS 風トーン
    pub dominant_details: Vec<DominantTheoryDto>,
    /// 支配色の彩度が十分あるときの加重平均色相から一言
    pub dominant_hue_summary_ja: Option<String>,
}

/// a*, b* から色相角（度）。+a が 0° 付近（赤〜マゼンタ寄り）、+b が 90°（黄寄り）。
pub fn hue_deg_from_ab(a: f64, b: f64) -> f64 {
    let rad = b.atan2(a);
    let mut deg = rad.to_degrees();
    if deg < 0.0 {
        deg += 360.0;
    }
    deg
}

pub fn chroma_star(a: f64, b: f64) -> f64 {
    (a * a + b * b).sqrt()
}

/// 10 分割の日本語色相帯（マンセル色相環の考え方に近い粗い区分）。
pub fn hue_region_ja(h_deg: f64) -> &'static str {
    let h = h_deg.rem_euclid(360.0);
    match h {
        x if x >= 342.0 || x < 18.0 => "赤紫系",
        x if x < 54.0 => "赤系",
        x if x < 90.0 => "赤黄系",
        x if x < 126.0 => "黄系",
        x if x < 162.0 => "黄緑系",
        x if x < 198.0 => "緑系",
        x if x < 234.0 => "青緑系",
        x if x < 270.0 => "青系",
        x if x < 306.0 => "青紫系",
        _ => "紫系",
    }
}

/// PCCS 風トーン（L*・C* の便宜的な閾値。商標・公式定義の再現ではない）。
pub fn pccs_style_tone(l: f64, c: f64) -> (&'static str, &'static str) {
    const C_GRAY: f64 = 8.0;
    const C_LOW: f64 = 18.0;
    const C_MID: f64 = 28.0;
    const C_HIGH: f64 = 38.0;

    if c < C_GRAY {
        return ("achromatic", "無彩色寄り（彩度が極めて低い）");
    }
    if l >= 76.0 && c < C_LOW {
        return ("pale", "ペール寄り（明るく薄い）");
    }
    if l >= 68.0 && c >= C_LOW && c < C_MID && l >= 55.0 {
        return ("light_grayish", "ライトグレイッシュ寄り");
    }
    if l >= 72.0 && c >= C_MID && c < C_HIGH {
        return ("light", "ライト寄り（明るめの色調）");
    }
    if l >= 62.0 && c >= C_HIGH {
        return ("bright", "ブライト寄り（明るく鮮やか）");
    }
    if (40.0..=72.0).contains(&l) && c >= C_HIGH {
        return ("vivid", "ビビッド寄り（中明度で鮮やか）");
    }
    if (38.0..62.0).contains(&l) && c >= C_MID {
        return ("strong", "ストロング寄り（やや濃い鮮やかさ）");
    }
    if l < 48.0 && c >= C_HIGH {
        return ("deep", "ディープ寄り（暗く濃い）");
    }
    if l < 50.0 && c >= C_LOW && c < C_HIGH {
        return ("dull", "ダール寄り（くすんだ）");
    }
    if l < 45.0 && c < C_MID {
        return ("dark", "ダーク寄り（暗め）");
    }
    if (52.0..68.0).contains(&l) && c < C_MID {
        return ("soft", "ソフト寄り（やや灰み）");
    }
    ("general", "中間的なトーン")
}

pub fn weighted_mean_hue_deg(weighted_hues: &[(f64, f64)]) -> Option<f64> {
    if weighted_hues.is_empty() {
        return None;
    }
    let mut sx = 0.0;
    let mut sy = 0.0;
    let mut wsum = 0.0;
    for &(h, w) in weighted_hues {
        let r = h.to_radians();
        sx += w * r.cos();
        sy += w * r.sin();
        wsum += w;
    }
    if wsum < 1e-9 {
        return None;
    }
    Some(sy.atan2(sx).to_degrees().rem_euclid(360.0))
}

fn rgb_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

pub fn build_theory_block(palette: &[(u8, u8, u8, f32)]) -> TheoryBlock {
    let disclaimer_ja = "本アプリの「PCCS 風トーン」「色相帯」は CIELAB（L*a*b*）から算出した近似ラベルであり、日本色研の PCCS 公式定義や色彩検定の採点基準とは一致しません。".to_string();

    let outline_mapping_ja = vec![
        "マンセル表色系の考え方: 色は色相・明度・彩度で記述できる → 本アプリでは L*（明度に近い）と a*b* から求めた彩度 C*・色相角 h° で対応づけています。".to_string(),
        "PCCS のトーン: 明度と彩度をまとめた概念 → 本アプリでは L* と C* の便宜的な閾値で、非公式の「トーン風」日本語ラベルを付与しています。".to_string(),
        "色立体・固有明度: 色相ごとに最も鮮やかな色の明度が異なる → トーン名はあくまでラベルであり、立体上の厳密位置は示しません。".to_string(),
        "ΔE2000・パレット近似: 測色・色差の話であり、トーン理論とは別軸の指標です。".to_string(),
    ];

    let mut dominant_details = Vec::new();
    let mut weighted_for_mean: Vec<(f64, f64)> = Vec::new();

    for &(r, g, b, pct) in palette {
        let lab = lab_from_srgb(r, g, b);
        let c_star = chroma_star(lab.a, lab.b);
        let h_deg = hue_deg_from_ab(lab.a, lab.b);
        let (tid, tja) = pccs_style_tone(lab.l, c_star);
        dominant_details.push(DominantTheoryDto {
            hex: rgb_hex(r, g, b),
            l_star: (lab.l * 10.0).round() / 10.0,
            c_star: (c_star * 10.0).round() / 10.0,
            h_deg: (h_deg * 10.0).round() / 10.0,
            hue_region_ja: hue_region_ja(h_deg).to_string(),
            pccs_style_tone_ja: tja.to_string(),
            pccs_style_tone_id: tid.to_string(),
        });
        if c_star >= 12.0 {
            weighted_for_mean.push((h_deg, pct as f64));
        }
    }

    let dominant_hue_summary_ja = weighted_mean_hue_deg(&weighted_for_mean).map(|h| {
        format!(
            "彩度をある程度含む支配色の加重平均色相は約 {:.0}°（{}）付近です。",
            h,
            hue_region_ja(h)
        )
    });

    TheoryBlock {
        disclaimer_ja,
        outline_mapping_ja,
        dominant_details,
        dominant_hue_summary_ja,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn white_is_achromatic() {
        let lab = lab_from_srgb(255, 255, 255);
        let c = chroma_star(lab.a, lab.b);
        let (id, _) = pccs_style_tone(lab.l, c);
        assert_eq!(id, "achromatic");
    }
}
