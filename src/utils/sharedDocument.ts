import type { Analysis } from "../types/analysis";

/** 共有用の文書へ出す名前。Unix / Windows の絶対パスをファイル名へ縮退する。 */
export function sharedFileName(path: string): string {
  const normalized = path.replace(/\\/g, "/");
  const name = normalized.split("/").filter(Boolean).pop();
  return name || "image";
}

/**
 * 共有用 JSON の境界。
 * 内部解析では元パスを保持するが、書き出しにはファイル名だけを残す。
 */
export function buildSharedAnalysisExport(a: Analysis) {
  const {
    previewJpegBase64: _previewJpegBase64,
    path: sourcePath,
    harmonyScores: _harmonyScores,
    ...rest
  } = a;
  return {
    ...rest,
    path: sharedFileName(sourcePath),
    exportedAt: new Date().toISOString(),
    previewJpegBase64Omitted: true,
    note: "プレビュー画像の base64 はファイルサイズのため省略（分析数値のみの資産向け）",
  };
}

export function buildSharedAnalysisJson(a: Analysis): string {
  return JSON.stringify(buildSharedAnalysisExport(a), null, 2);
}
