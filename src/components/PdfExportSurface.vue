<script setup lang="ts">
import { APP_DISPLAY_NAME } from "../constants/appMeta";
import { harmonyScoreLegendLines } from "../constants/harmonyScoreLegend";
import type { Analysis } from "../types/analysis";
import type { ColorAuxMode } from "../utils/colorFormat";
import { colorAuxModeLabel, formatAuxColor } from "../utils/colorFormat";

const props = defineProps<{
  analysis: Analysis;
  previewDataUrl: string;
  auxMode: ColorAuxMode;
}>();

function aux(r: number, g: number, b: number): string {
  return formatAuxColor(props.auxMode, r, g, b);
}

const appName = APP_DISPLAY_NAME;
</script>

<template>
  <div class="pdf-root">
    <h1 class="pdf-title">{{ appName }} レポート</h1>
    <p class="pdf-meta">
      出力日時: {{ new Date().toISOString() }} · 補助色表記:
      {{ colorAuxModeLabel(auxMode) }}
    </p>

    <img
      v-if="previewDataUrl"
      class="pdf-preview"
      :src="previewDataUrl"
      alt="プレビュー"
    />

    <h2 class="pdf-h2">ファイル</h2>
    <p class="pdf-mono pdf-path">{{ analysis.path }}</p>
    <p>
      {{ analysis.width }} × {{ analysis.height }} px
      <template v-if="analysis.fileSizeDisplay">
        · {{ analysis.fileSizeDisplay }}</template
      >
      <template v-if="analysis.modifiedDisplay">
        · 更新 {{ analysis.modifiedDisplay }}</template
      >
    </p>

    <template v-if="analysis.dominants.length">
      <h2 class="pdf-h2">主要色（推定）</h2>
      <table class="pdf-table">
        <thead>
          <tr>
            <th>%</th>
            <th>HEX</th>
            <th>{{ colorAuxModeLabel(auxMode) }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(d, i) in analysis.dominants" :key="i">
            <td>{{ d.pct.toFixed(1) }}</td>
            <td class="pdf-mono">{{ d.hex }}</td>
            <td class="pdf-mono">{{ aux(d.r, d.g, d.b) }}</td>
          </tr>
        </tbody>
      </table>
    </template>

    <template v-if="analysis.gist.lines.length">
      <h2 class="pdf-h2">ひと目サマリ</h2>
      <div class="pdf-gist">
        <p v-for="(row, i) in analysis.gist.lines" :key="i" class="pdf-gist-line">
          {{ row.text }}
        </p>
      </div>
    </template>

    <template v-if="analysis.harmonyScores.length">
      <h2 class="pdf-h2">色相調和スコア（参考・％表示・内部は 0〜1）</h2>
      <p class="pdf-muted">
        彩度が十分な支配色の加重に基づく参考値です。公式の調和理論の再現ではありません。
      </p>
      <div class="pdf-harmony-legend">
        <p v-for="(line, hi) in harmonyScoreLegendLines" :key="hi" class="pdf-legend-line">
          {{ line }}
        </p>
      </div>
      <ul class="pdf-list">
        <li v-for="h in analysis.harmonyScores" :key="h.id">
          {{ h.labelJa }} … {{ (h.score * 100).toFixed(0) }}%
        </li>
      </ul>
    </template>

    <template v-if="analysis.wcagDominantPair">
      <h2 class="pdf-h2">WCAG（主要色 1 位 vs 2 位）</h2>
      <p class="pdf-mono">
        コントラスト比 {{ analysis.wcagDominantPair.contrastRatio.toFixed(2) }} :1
      </p>
    </template>

    <h2 class="pdf-h2">EXIF</h2>
    <template v-if="!analysis.exif.length">
      <p>このファイルからは EXIF を読み取れませんでした。</p>
    </template>
    <dl v-else class="pdf-exif">
      <template v-for="(row, i) in analysis.exif" :key="i">
        <dt>{{ row.label }}</dt>
        <dd>{{ row.value }}</dd>
      </template>
    </dl>

    <p class="pdf-foot">
      プレビューはアプリ内生成の JPEG です。色彩・調和の説明は参考値であり、公式定義の再現ではありません。
    </p>
  </div>
</template>

<style scoped>
.pdf-root {
  width: 780px;
  box-sizing: border-box;
  padding: 20px 28px 36px;
  background: #fff;
  color: #141418;
  font-family:
    system-ui,
    -apple-system,
    "Segoe UI",
    "Hiragino Sans",
    "Hiragino Kaku Gothic ProN",
    Meiryo,
    sans-serif;
  font-size: 11px;
  line-height: 1.5;
}

.pdf-title {
  margin: 0 0 6px;
  font-size: 18px;
  font-weight: 700;
}

.pdf-meta {
  margin: 0 0 14px;
  color: #444;
  font-size: 10px;
}

.pdf-muted {
  margin: 2px 0 6px;
  font-size: 10px;
  line-height: 1.45;
  color: #555;
}

.pdf-harmony-legend {
  margin: 0 0 8px;
  padding: 6px 8px;
  background: #f8f9fc;
  border: 1px solid #e0e4ee;
  border-radius: 4px;
}

.pdf-legend-line {
  margin: 0 0 4px;
  font-size: 9.5px;
  line-height: 1.45;
  color: #3a3a42;
}

.pdf-legend-line:last-child {
  margin-bottom: 0;
}

/* プレビューは 1 ページに収めつつ、識別しやすい大きさ */
.pdf-preview {
  display: block;
  width: auto;
  max-width: 82%;
  max-height: 260px;
  height: auto;
  margin: 0 auto 14px;
  border: 1px solid #ccc;
  border-radius: 4px;
  object-fit: contain;
}

.pdf-h2 {
  margin: 14px 0 6px;
  font-size: 12px;
  font-weight: 700;
  border-bottom: 1px solid #ddd;
  padding-bottom: 2px;
}

.pdf-h2:first-of-type {
  margin-top: 0;
}

.pdf-path {
  word-break: break-all;
  font-size: 9px;
  margin: 0 0 4px;
}

.pdf-mono {
  font-family: ui-monospace, "Cascadia Code", Menlo, monospace;
}

.pdf-table {
  width: 100%;
  border-collapse: collapse;
  margin: 4px 0 0;
}

.pdf-table th,
.pdf-table td {
  border: 1px solid #ccc;
  padding: 4px 6px;
  text-align: left;
}

.pdf-table th {
  background: #f4f4f6;
  font-weight: 600;
}

.pdf-gist {
  margin: 4px 0 0;
  padding: 8px 10px;
  background: #f6f8fc;
  border: 1px solid #c8d4ec;
  border-radius: 4px;
}

.pdf-gist-line {
  margin: 0 0 4px;
  white-space: pre-wrap;
}

.pdf-gist-line:last-child {
  margin-bottom: 0;
}

.pdf-list {
  margin: 4px 0 0;
  padding-left: 1.2rem;
}

.pdf-exif {
  margin: 4px 0 0;
}

.pdf-exif dt {
  font-weight: 700;
  margin-top: 4px;
}

.pdf-exif dd {
  margin: 2px 0 0 0;
  padding: 0;
}

.pdf-foot {
  margin: 20px 0 0;
  font-size: 9px;
  color: #555;
}
</style>
