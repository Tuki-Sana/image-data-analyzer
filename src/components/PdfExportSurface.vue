<script setup lang="ts">
import { APP_DISPLAY_NAME } from "../constants/appMeta";
import type { Analysis } from "../types/analysis";
import type { ColorAuxMode } from "../utils/colorFormat";
import { colorAuxModeLabel, formatAuxColor } from "../utils/colorFormat";
import { sharedFileName } from "../utils/sharedDocument";

const props = defineProps<{
  analysis: Analysis;
  previewDataUrl: string;
  auxMode: ColorAuxMode;
}>();

function aux(r: number, g: number, b: number): string {
  return formatAuxColor(props.auxMode, r, g, b);
}

/** 8bit sRGB → #RRGGBB（PDF の支配色列用。`PaletteMatch` に支配側 hex が無いため） */
function rgbToHex(r: number, g: number, b: number): string {
  const h = (n: number) => n.toString(16).toUpperCase().padStart(2, "0");
  return `#${h(r)}${h(g)}${h(b)}`;
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
    <p class="pdf-mono pdf-path">{{ sharedFileName(analysis.path) }}</p>
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

    <template v-if="analysis.openColorMatches.length">
      <h2 class="pdf-h2">Open Color 近似（ΔE2000）</h2>
      <p class="pdf-muted">
        オープンソース配色セット「Open Color」の名前付き色との距離（CIEDE2000）。公式の正解色名ではありません。
      </p>
      <table class="pdf-table pdf-table--match">
        <thead>
          <tr>
            <th>%</th>
            <th>支配色</th>
            <th>近似スウォッチ</th>
            <th>ΔE2000</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(m, i) in analysis.openColorMatches" :key="'oc-' + i">
            <td>{{ m.pct.toFixed(1) }}</td>
            <td>
              <span
                class="pdf-swatch-inline"
                :style="{ backgroundColor: `rgb(${m.domR},${m.domG},${m.domB})` }"
              />
              <span class="pdf-mono">{{ rgbToHex(m.domR, m.domG, m.domB) }}</span>
            </td>
            <td>
              <span
                class="pdf-swatch-inline"
                :style="{ backgroundColor: `rgb(${m.swR},${m.swG},${m.swB})` }"
              />
              <span class="pdf-mono">{{ m.swatchName }} · {{ m.swHex }}</span>
            </td>
            <td class="pdf-mono">{{ m.deltaE.toFixed(1) }}</td>
          </tr>
        </tbody>
      </table>
    </template>

    <template v-if="analysis.tailwindMatches.length">
      <h2 class="pdf-h2">Tailwind 近似（500/600/700 サブセット、ΔE2000）</h2>
      <p class="pdf-muted">
        Tailwind CSS 標準色のうち明るさ 500 / 600 / 700 のみを比較（フルパレットではありません）。
      </p>
      <table class="pdf-table pdf-table--match">
        <thead>
          <tr>
            <th>%</th>
            <th>支配色</th>
            <th>近似スウォッチ</th>
            <th>ΔE2000</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(m, i) in analysis.tailwindMatches" :key="'tw-' + i">
            <td>{{ m.pct.toFixed(1) }}</td>
            <td>
              <span
                class="pdf-swatch-inline"
                :style="{ backgroundColor: `rgb(${m.domR},${m.domG},${m.domB})` }"
              />
              <span class="pdf-mono">{{ rgbToHex(m.domR, m.domG, m.domB) }}</span>
            </td>
            <td>
              <span
                class="pdf-swatch-inline"
                :style="{ backgroundColor: `rgb(${m.swR},${m.swG},${m.swB})` }"
              />
              <span class="pdf-mono">{{ m.swatchName }} · {{ m.swHex }}</span>
            </td>
            <td class="pdf-mono">{{ m.deltaE.toFixed(1) }}</td>
          </tr>
        </tbody>
      </table>
    </template>

    <template v-if="analysis.wcagDominantPair">
      <h2 class="pdf-h2">WCAG コントラスト（主要色 1位 vs 2位）</h2>
      <p class="pdf-mono">
        コントラスト比 {{ analysis.wcagDominantPair.contrastRatio.toFixed(2) }} :1
      </p>
    </template>

    <template v-if="analysis.theory">
      <h2 class="pdf-h2">色彩理論メモ（PCCS 風・非公式）</h2>
      <p class="pdf-theory-disclaimer">{{ analysis.theory.disclaimerJa }}</p>
      <p v-if="analysis.theory.dominantHueSummaryJa" class="pdf-mono pdf-theory-summary">
        {{ analysis.theory.dominantHueSummaryJa }}
      </p>
      <p class="pdf-muted pdf-theory-label">概論との対応（目安）</p>
      <ul class="pdf-outline-list">
        <li v-for="(line, i) in analysis.theory.outlineMappingJa" :key="'om-' + i">
          {{ line }}
        </li>
      </ul>
      <p class="pdf-muted pdf-theory-label">支配色ごとの L*・C*・色相帯・トーン</p>
      <ul class="pdf-theory-dominant">
        <li v-for="(t, i) in analysis.theory.dominantDetails" :key="'td-' + i" class="pdf-theory-row">
          <span class="pdf-swatch-inline pdf-swatch-inline--lg" :style="{ backgroundColor: t.hex }" />
          <div class="pdf-theory-row-text">
            <span class="pdf-mono">{{ t.hex }}</span>
            <span class="pdf-theory-meta">
              L* {{ t.lStar.toFixed(1) }} · C* {{ t.cStar.toFixed(1) }} · h°
              {{ t.hDeg.toFixed(0) }}
            </span>
            <span>{{ t.hueRegionJa }} · {{ t.pccsStyleToneJa }}</span>
          </div>
        </li>
      </ul>
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
      プレビューはアプリ内生成の JPEG です。Open Color / Tailwind 近似は CIEDE2000
      に基づく参考値です。色彩理論メモは公式 PCCS や教科書的定義の再現ではありません。
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

.pdf-table--match td {
  vertical-align: middle;
  font-size: 10px;
}

.pdf-swatch-inline {
  display: inline-block;
  width: 11px;
  height: 11px;
  margin-right: 5px;
  border: 1px solid #999;
  border-radius: 2px;
  vertical-align: middle;
}

.pdf-swatch-inline--lg {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
}

.pdf-theory-disclaimer {
  margin: 0 0 8px;
  padding: 6px 8px;
  font-size: 9.5px;
  line-height: 1.45;
  color: #444;
  background: #fafafa;
  border: 1px solid #e8e8ec;
  border-radius: 4px;
}

.pdf-theory-summary {
  margin: 0 0 8px;
  font-size: 10px;
  line-height: 1.45;
}

.pdf-theory-label {
  margin: 8px 0 4px;
  font-weight: 600;
}

.pdf-outline-list {
  margin: 0 0 6px;
  padding-left: 1.1rem;
  font-size: 9.5px;
  line-height: 1.45;
  color: #333;
}

.pdf-outline-list li {
  margin-bottom: 3px;
}

.pdf-theory-dominant {
  margin: 4px 0 0;
  padding: 0;
  list-style: none;
}

.pdf-theory-row {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  margin-bottom: 8px;
  padding-bottom: 6px;
  border-bottom: 1px solid #eee;
}

.pdf-theory-row:last-child {
  margin-bottom: 0;
  padding-bottom: 0;
  border-bottom: none;
}

.pdf-theory-row-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
  font-size: 9.5px;
  line-height: 1.4;
}

.pdf-theory-meta {
  font-family: ui-monospace, "Cascadia Code", Menlo, monospace;
  font-size: 9px;
  color: #444;
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
