export interface ExifLine {
  label: string;
  value: string;
}

export interface Dominant {
  r: number;
  g: number;
  b: number;
  pct: number;
  hex: string;
}

export interface PaletteMatch {
  domR: number;
  domG: number;
  domB: number;
  pct: number;
  swatchName: string;
  swR: number;
  swG: number;
  swB: number;
  swHex: string;
  deltaE: number;
}

export interface WcagDominantPair {
  r1: number;
  g1: number;
  b1: number;
  r2: number;
  g2: number;
  b2: number;
  contrastRatio: number;
}

/** 支配色ごとの L*C*h° と PCCS 風トーン（非公式近似） */
export interface DominantTheory {
  hex: string;
  lStar: number;
  cStar: number;
  hDeg: number;
  hueRegionJa: string;
  pccsStyleToneJa: string;
  pccsStyleToneId: string;
}

export interface TheoryBlock {
  disclaimerJa: string;
  outlineMappingJa: string[];
  dominantDetails: DominantTheory[];
  dominantHueSummaryJa: string | null;
}

export interface HarmonyScore {
  id: string;
  labelJa: string;
  score: number;
}

export type GistLineRole = "mono" | "label" | "body" | "foot";

export interface GistLine {
  text: string;
  role: GistLineRole;
}

export interface AnalysisGist {
  lines: GistLine[];
  /** lines の text を改行で連結（エクスポート・外部ツール向け） */
  gistJa: string;
}

export interface Analysis {
  schemaVersion: number;
  path: string;
  width: number;
  height: number;
  fileSizeBytes: number | null;
  fileSizeDisplay: string | null;
  modifiedDisplay: string | null;
  exif: ExifLine[];
  previewJpegBase64: string;
  previewWidth: number;
  previewHeight: number;
  previewBgDark: boolean;
  dominants: Dominant[];
  openColorMatches: PaletteMatch[];
  tailwindMatches: PaletteMatch[];
  wcagDominantPair: WcagDominantPair | null;
  theory: TheoryBlock;
  harmonyScores: HarmonyScore[];
  gist: AnalysisGist;
}

export interface PixelSample {
  r: number;
  g: number;
  b: number;
  hex: string;
}

/** スポイトで蓄積するパレット（LocalStorage 永続） */
export interface PickerPaletteEntry {
  id: string;
  r: number;
  g: number;
  b: number;
  hex: string;
  addedAt: string;
  /** キャラ色など任意のメモ（例: 肌・髪） */
  label?: string;
}
