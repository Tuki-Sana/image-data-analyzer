//! 支配色の色相から、代表的な調和パターンへの「当てはまり度」スコア（0〜1 目安、独自定義）。

use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HarmonyScoreDto {
    pub id: String,
    pub label_ja: String,
    pub score: f64,
}

/// テンプレート調和の向きを合わせるアンカー数（`ANCHOR_STEP_DEG` 刻みで 1 周）。
const ANCHOR_COUNT: usize = 36;
const ANCHOR_STEP_DEG: f64 = 10.0;

/// ガウス重みの σ（度）。型ごとに最寄り理想角の取りやすさが違うため別値にする。
const SIGMA_COMPLEMENTARY_DEG: f64 = 20.0;
const SIGMA_SPLIT_COMPLEMENTARY_DEG: f64 = 24.0;
const SIGMA_TRIADIC_DEG: f64 = 22.0;
const SIGMA_TETRAD_DEG: f64 = 24.5;

fn hue_circ_dist_deg(a: f64, b: f64) -> f64 {
    let mut d = (a - b).abs() % 360.0;
    if d > 180.0 {
        d = 360.0 - d;
    }
    d
}

/// (hue_deg, weight), weight は正
fn score_template(weighted: &[(f64, f64)], ideal_rels: &[f64], sigma_deg: f64) -> f64 {
    if weighted.is_empty() || ideal_rels.is_empty() {
        return 0.0;
    }
    let w_sum: f64 = weighted.iter().map(|(_, w)| w).sum();
    if w_sum < 1e-9 {
        return 0.0;
    }

    let mut best_global = 0.0f64;
    for k in 0..ANCHOR_COUNT {
        let anchor = k as f64 * ANCHOR_STEP_DEG;
        let ideals: Vec<f64> = ideal_rels
            .iter()
            .map(|r| (anchor + r).rem_euclid(360.0))
            .collect();

        let mut acc = 0.0f64;
        for &(h, w) in weighted {
            let mut best = f64::INFINITY;
            for &t in &ideals {
                best = best.min(hue_circ_dist_deg(h, t));
            }
            acc += w * (-(best * best) / (2.0 * sigma_deg * sigma_deg)).exp();
        }
        best_global = best_global.max(acc / w_sum);
    }
    best_global
}

/// 類似色: 重み付き円周ベクトルの合成が強いほど高スコア（支配色の面積比を反映）。
/// 合成長さ / 重み和は 2 色等重み・角 θ のとき cos(θ/2) に一致し、旧実装の弧長閾値に近い段階を維持。
fn score_analogous(weighted: &[(f64, f64)]) -> f64 {
    if weighted.len() < 2 {
        return 0.0;
    }
    let w_sum: f64 = weighted.iter().map(|(_, w)| w).sum();
    if w_sum < 1e-9 {
        return 0.0;
    }
    let mut sx = 0.0;
    let mut sy = 0.0;
    for &(h, w) in weighted {
        let r = h.to_radians();
        sx += w * r.cos();
        sy += w * r.sin();
    }
    let conc = (sx * sx + sy * sy).sqrt() / w_sum;
    if conc >= 0.970 {
        0.95
    } else if conc >= 0.923 {
        0.75
    } else if conc >= 0.866 {
        0.50
    } else if conc >= 0.58 {
        0.20
    } else {
        0.12
    }
}

pub fn harmony_scores(weighted_hues: &[(f64, f64)]) -> Vec<HarmonyScoreDto> {
    let mut out = vec![
        HarmonyScoreDto {
            id: "analogous".to_string(),
            label_ja: "類似色（色相が近い）".to_string(),
            score: score_analogous(weighted_hues),
        },
        HarmonyScoreDto {
            id: "complementary".to_string(),
            label_ja: "補色（約 180° 対比）".to_string(),
            score: score_template(weighted_hues, &[0.0, 180.0], SIGMA_COMPLEMENTARY_DEG),
        },
        HarmonyScoreDto {
            id: "split_complementary".to_string(),
            label_ja: "分割補色（0° / 150° / 210° 付近）".to_string(),
            score: score_template(
                weighted_hues,
                &[0.0, 150.0, 210.0],
                SIGMA_SPLIT_COMPLEMENTARY_DEG,
            ),
        },
        HarmonyScoreDto {
            id: "triadic".to_string(),
            label_ja: "トライアド（約 120° 間隔）".to_string(),
            score: score_template(weighted_hues, &[0.0, 120.0, 240.0], SIGMA_TRIADIC_DEG),
        },
        HarmonyScoreDto {
            id: "tetrad".to_string(),
            label_ja: "テトラード（矩形・60°/180°/240° 系）".to_string(),
            score: score_template(weighted_hues, &[0.0, 60.0, 180.0, 240.0], SIGMA_TETRAD_DEG),
        },
    ];

    out.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    for s in &mut out {
        s.score = (s.score * 1000.0).round() / 1000.0;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analogous_two_hues_respects_weight() {
        // 主役 0° とノイズ 180°：等重みなら低い、主役に重みを寄せると高い
        let balanced = vec![(0.0, 50.0), (180.0, 50.0)];
        let skewed = vec![(0.0, 95.0), (180.0, 5.0)];
        assert!(
            score_analogous(&balanced) < score_analogous(&skewed),
            "expected skewed analogous > balanced"
        );
    }

    #[test]
    fn analogous_same_direction_high() {
        let w = vec![(10.0, 30.0), (25.0, 70.0)];
        assert!(score_analogous(&w) >= 0.9);
    }

    #[test]
    fn complementary_pair_low_analogous() {
        let w = vec![(0.0, 50.0), (180.0, 50.0)];
        assert!(score_analogous(&w) < 0.3);
    }

    #[test]
    fn template_triadic_exact_ideal_angles_scores_one() {
        let w = vec![(0.0, 34.0), (120.0, 33.0), (240.0, 33.0)];
        let s = score_template(&w, &[0.0, 120.0, 240.0], SIGMA_TRIADIC_DEG);
        assert!((s - 1.0).abs() < 1e-9, "got {}", s);
    }

    #[test]
    fn template_complementary_exact_scores_one() {
        let w = vec![(0.0, 50.0), (180.0, 50.0)];
        let s = score_template(&w, &[0.0, 180.0], SIGMA_COMPLEMENTARY_DEG);
        assert!((s - 1.0).abs() < 1e-9, "got {}", s);
    }
}
