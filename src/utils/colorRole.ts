import type { Dominant } from "../types/analysis";

export type ColorRole = "base" | "assort" | "accent";

export interface ColorRoleThresholds {
  /** ベースカラーの累積上限（%）。デフォルト 70 */
  baseCutoff: number;
  /** アクセントカラーの開始累積（%）。デフォルト 95 */
  accentCutoff: number;
}

export const DEFAULT_COLOR_ROLE_THRESHOLDS: ColorRoleThresholds = {
  baseCutoff: 70,
  accentCutoff: 95,
};

export interface DominantWithRole extends Dominant {
  role: ColorRole;
  /** この色を加えた時点の累積 pct */
  cumulativePct: number;
}

/**
 * 支配色リスト（pct 降順を想定）をベース/アソート/アクセントに分類する。
 *
 * - base  : 累積 pct が baseCutoff 以下に収まる色
 * - assort: baseCutoff 超〜accentCutoff 以下
 * - accent: accentCutoff を超えた色
 */
export function classifyColorRoles(
  dominants: Dominant[],
  thresholds: ColorRoleThresholds = DEFAULT_COLOR_ROLE_THRESHOLDS,
): DominantWithRole[] {
  let cumulative = 0;
  return dominants.map((d) => {
    cumulative += d.pct;
    const role: ColorRole =
      cumulative <= thresholds.baseCutoff
        ? "base"
        : cumulative <= thresholds.accentCutoff
          ? "assort"
          : "accent";
    return { ...d, role, cumulativePct: Math.min(cumulative, 100) };
  });
}

export const COLOR_ROLE_LABEL: Record<ColorRole, string> = {
  base: "ベース",
  assort: "アソート",
  accent: "アクセント",
};
