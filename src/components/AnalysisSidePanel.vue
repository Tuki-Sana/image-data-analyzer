<script setup lang="ts">
import { harmonyScoreLegendLines } from "../constants/harmonyScoreLegend";
import type { Analysis, PickerPaletteEntry, PixelSample } from "../types/analysis";
import type { ColorAuxMode } from "../utils/colorFormat";
import { formatAuxColor } from "../utils/colorFormat";
import {
  PICKER_LABEL_MAX,
  PICKER_PALETTE_MAX,
  type PickerPaletteSet,
  type PickerPalettesState,
} from "../utils/pickerPaletteStorage";
import PickerPaletteSetBar from "./PickerPaletteSetBar.vue";

const props = defineProps<{
  analysis: Analysis;
  picked: PixelSample | null;
  exportJsonText: string;
  paletteState: PickerPalettesState;
  activePaletteSet: PickerPaletteSet;
  canDeletePaletteSet: boolean;
  pickerPalette: PickerPaletteEntry[];
}>();

const colorAuxMode = defineModel<ColorAuxMode>("colorAuxMode", { required: true });
const paletteLabelDraft = defineModel<string>("paletteLabelDraft", {
  required: true,
});

const emit = defineEmits<{
  copyText: [text: string, label: string];
  addPickedToPalette: [];
  openGlossary: [entryId: string];
  setActivePaletteId: [id: string];
  updateActiveSetName: [name: string];
  newPaletteSet: [];
  duplicatePaletteSet: [];
  deletePaletteSet: [];
  setPaletteEntryLabel: [id: string, value: string];
  removePaletteEntry: [id: string];
  importPickerPaletteReplace: [];
  importPickerPaletteMerge: [];
  copyPickerPaletteHexLines: [];
  copyPickerPaletteLabeledLines: [];
  copyPickerPaletteJson: [];
  savePickerPaletteJson: [];
  clearPickerPalette: [];
}>();

function gistRowClass(role: string): string {
  const parts = ["gist-line"];
  if (role === "mono") {
    parts.push("mono", "small");
  } else if (role === "label") {
    parts.push("muted", "small", "gist-label");
  } else if (role === "foot") {
    parts.push("muted", "small", "gist-foot");
  } else {
    parts.push("small");
  }
  return parts.join(" ");
}
</script>

<template>
  <aside class="panel" aria-label="分析結果">
    <section class="block">
      <h2 class="h">ファイル</h2>
      <p class="mono path">{{ props.analysis.path }}</p>
      <p class="muted">
        {{ props.analysis.width }} × {{ props.analysis.height }} px
        <template v-if="props.analysis.fileSizeDisplay">
          · {{ props.analysis.fileSizeDisplay }}</template
        >
        <template v-if="props.analysis.modifiedDisplay">
          · 更新 {{ props.analysis.modifiedDisplay }}</template
        >
      </p>
    </section>

    <section
      v-if="picked || props.analysis.dominants.length"
      class="block block-tight"
    >
      <p class="muted small color-mode-hint">
        補助色表記（HEX に加えて表示・PDF にも反映）
      </p>
      <div class="color-mode-row">
        <div class="toggle" role="group" aria-label="補助色表記">
          <button
            type="button"
            class="toggle-btn"
            :class="{ 'toggle-btn--on': colorAuxMode === 'rgb' }"
            :aria-pressed="colorAuxMode === 'rgb'"
            @click="colorAuxMode = 'rgb'"
          >
            RGB
          </button>
          <button
            type="button"
            class="toggle-btn"
            :class="{ 'toggle-btn--on': colorAuxMode === 'hsl' }"
            :aria-pressed="colorAuxMode === 'hsl'"
            @click="colorAuxMode = 'hsl'"
          >
            HSL
          </button>
        </div>
      </div>
    </section>

    <section v-if="picked" class="block">
      <h2 class="h">クリックした色</h2>
      <div class="row-color">
        <span
          class="swatch"
          aria-hidden="true"
          :style="{
            backgroundColor: `rgb(${picked.r},${picked.g},${picked.b})`,
          }"
        />
        <div class="dominant-text">
          <button
            type="button"
            class="linkish"
            @click="emit('copyText', picked.hex, 'HEX')"
          >
            {{ picked.hex }}
          </button>
          <p class="mono small aux-line">
            {{
              formatAuxColor(colorAuxMode, picked.r, picked.g, picked.b)
            }}
          </p>
          <button
            type="button"
            class="linkish linkish-tiny"
            @click="
              emit(
                'copyText',
                formatAuxColor(colorAuxMode, picked.r, picked.g, picked.b),
                '補助表記',
              )
            "
          >
            補助表記をコピー
          </button>
          <label class="palette-draft-wrap">
            <span class="palette-draft-label">パレット用の名前（任意）</span>
            <input
              v-model="paletteLabelDraft"
              type="text"
              class="palette-draft-input"
              :maxlength="PICKER_LABEL_MAX"
              placeholder="例: 肌・髪の影・瞳"
              autocomplete="off"
            />
          </label>
          <button
            type="button"
            class="btn-palette-add"
            @click="emit('addPickedToPalette')"
          >
            パレットに追加
          </button>
        </div>
      </div>
    </section>

    <section v-if="props.analysis" class="block block-picker-palette">
      <h2 class="h">スポイトパレット</h2>
      <p class="palette-lead">
        キャラ色など、名前を付けて整理できます（最大
        {{ PICKER_PALETTE_MAX }} 色・カラーセットは複数可）
      </p>
      <PickerPaletteSetBar
        :palettes="props.paletteState.palettes"
        :active-id="props.paletteState.activePaletteId"
        :set-name="props.activePaletteSet.name"
        :can-delete-set="props.canDeletePaletteSet"
        @update:active-id="emit('setActivePaletteId', $event)"
        @update:set-name="emit('updateActiveSetName', $event)"
        @new-set="emit('newPaletteSet')"
        @duplicate="emit('duplicatePaletteSet')"
        @delete-set="emit('deletePaletteSet')"
      />
      <div
        v-if="pickerPalette.length"
        class="palette-chip-grid"
        role="list"
      >
        <div
          v-for="e in pickerPalette"
          :key="e.id"
          class="palette-chip-cell"
          role="listitem"
        >
          <button
            type="button"
            class="palette-chip"
            :title="`${e.label ? `${e.label} · ` : ''}${e.hex} · クリックでコピー`"
            :style="{
              backgroundColor: `rgb(${e.r},${e.g},${e.b})`,
            }"
            @click="emit('copyText', e.hex, 'HEX')"
          />
          <span class="palette-chip-hex mono">{{ e.hex }}</span>
          <input
            class="palette-label-input"
            type="text"
            :value="e.label ?? ''"
            :maxlength="PICKER_LABEL_MAX"
            placeholder="名前"
            aria-label="色の名前"
            @input="
              emit(
                'setPaletteEntryLabel',
                e.id,
                ($event.target as HTMLInputElement).value,
              )
            "
          />
          <button
            type="button"
            class="palette-chip-remove"
            title="削除"
            aria-label="削除"
            @click="emit('removePaletteEntry', e.id)"
          >
            ×
          </button>
        </div>
      </div>
      <p v-else class="palette-empty-hint">
        「クリックした色」の<strong>パレットに追加</strong>から色を登録します。
      </p>
      <details class="palette-details">
        <summary class="palette-details-summary">保存の仕組み</summary>
        <p class="muted small palette-details-body">
          この端末の LocalStorage に保存され、画像を閉じても残ります。ブラウザやアプリのデータ消去で失われることがあります。大切な場合は JSON で書き出してください。
        </p>
      </details>
      <p class="palette-actions-heading">読み込み</p>
      <div class="palette-actions palette-actions-btns">
        <button
          type="button"
          class="palette-tool-btn palette-tool-btn--stacked"
          @click="emit('importPickerPaletteReplace')"
        >
          <span class="palette-tool-btn__line1">パレット JSON</span>
          <span class="palette-tool-btn__line2">（置換）</span>
        </button>
        <button
          type="button"
          class="palette-tool-btn palette-tool-btn--stacked"
          @click="emit('importPickerPaletteMerge')"
        >
          <span class="palette-tool-btn__line1">パレット JSON</span>
          <span class="palette-tool-btn__line2">（現在と結合）</span>
        </button>
      </div>
      <p class="palette-actions-heading">書き出し</p>
      <div class="palette-actions palette-actions-btns">
        <button
          type="button"
          class="palette-tool-btn"
          :disabled="pickerPalette.length === 0"
          @click="emit('copyPickerPaletteHexLines')"
        >
          HEX（1 行 1 色）
        </button>
        <button
          type="button"
          class="palette-tool-btn"
          :disabled="pickerPalette.length === 0"
          @click="emit('copyPickerPaletteLabeledLines')"
        >
          名前付きテキスト
        </button>
        <button
          type="button"
          class="palette-tool-btn"
          :disabled="pickerPalette.length === 0"
          @click="emit('copyPickerPaletteJson')"
        >
          JSON をコピー
        </button>
        <button
          type="button"
          class="palette-tool-btn"
          :disabled="pickerPalette.length === 0"
          @click="emit('savePickerPaletteJson')"
        >
          JSON を保存…
        </button>
        <button
          type="button"
          class="palette-tool-btn palette-tool-btn--stacked palette-tool-btn--danger"
          :disabled="pickerPalette.length === 0"
          title="カラーセットは残り、登録した色だけを空にします"
          @click="emit('clearPickerPalette')"
        >
          <span class="palette-tool-btn__line1">色をすべて削除</span>
          <span class="palette-tool-btn__line2">（セットは残る）</span>
        </button>
      </div>
    </section>

    <section v-if="props.analysis.dominants.length" class="block">
      <h2 class="h">主要色（推定）</h2>
      <ul class="list-dominant">
        <li
          v-for="(d, i) in props.analysis.dominants"
          :key="i"
          class="row-dominant"
        >
          <span
            class="swatch sm"
            aria-hidden="true"
            :style="{ backgroundColor: `rgb(${d.r},${d.g},${d.b})` }"
          />
          <div class="dominant-text">
            <button
              type="button"
              class="linkish"
              @click="emit('copyText', d.hex, 'HEX')"
            >
              {{ d.pct.toFixed(1) }}% {{ d.hex }}
            </button>
            <p class="mono small aux-line">
              {{ formatAuxColor(colorAuxMode, d.r, d.g, d.b) }}
            </p>
            <button
              type="button"
              class="linkish linkish-tiny"
              @click="
                emit(
                  'copyText',
                  formatAuxColor(colorAuxMode, d.r, d.g, d.b),
                  '補助表記',
                )
              "
            >
              補助表記をコピー
            </button>
          </div>
        </li>
      </ul>
    </section>

    <section v-if="props.analysis.gist.lines.length" class="block gist-block">
      <h2 class="h">ひと目サマリ</h2>
      <p
        v-for="(row, i) in props.analysis.gist.lines"
        :key="i"
        :class="gistRowClass(row.role)"
      >
        {{ row.text }}
      </p>
      <button
        type="button"
        class="linkish gist-copy"
        @click="emit('copyText', props.analysis.gist.gistJa, '要約テキスト')"
      >
        gist_ja をコピー（改行付き全文）
      </button>
    </section>

    <section v-if="props.analysis.openColorMatches.length" class="block block-approx">
      <h2 class="h muted-h">Open Color 近似（ΔE2000）</h2>
      <p class="block-lead">
        オープンソースの配色セット「Open Color」の名前付き色に、支配色がどれだけ近いかを示します（ΔE2000 が小さいほど近い）。公式の正解色名ではありません。
        <button
          type="button"
          class="glossary-jump"
          @click="emit('openGlossary', 'delta-e')"
        >
          用語集
        </button>
      </p>
      <ul class="list-match">
        <li v-for="(m, i) in props.analysis.openColorMatches" :key="i">
          <div class="row-mini">
            <span
              class="swatch xs"
              aria-hidden="true"
              :style="{
                backgroundColor: `rgb(${m.domR},${m.domG},${m.domB})`,
              }"
            />
            <span class="arrow" aria-hidden="true">→</span>
            <span
              class="swatch xs"
              aria-hidden="true"
              :style="{
                backgroundColor: `rgb(${m.swR},${m.swG},${m.swB})`,
              }"
            />
          </div>
          <button
            type="button"
            class="linkish"
            @click="emit('copyText', m.swHex, '近似色 HEX')"
          >
            {{ m.pct.toFixed(1) }}% {{ m.swatchName }} · ΔE2000
            {{ m.deltaE.toFixed(1) }}
          </button>
        </li>
      </ul>
    </section>

    <section v-if="props.analysis.tailwindMatches.length" class="block block-approx">
      <h2 class="h muted-h">Tailwind 近似（500/600/700 サブセット、ΔE2000）</h2>
      <p class="block-lead">
        Web 向け CSS フレームワーク「Tailwind CSS」の標準色のうち、明るさ 500 / 600 / 700 だけを抜き出して比較しています（フルパレットではありません）。
        <button
          type="button"
          class="glossary-jump"
          @click="emit('openGlossary', 'delta-e')"
        >
          用語集
        </button>
      </p>
      <ul class="list-match">
        <li v-for="(m, i) in props.analysis.tailwindMatches" :key="i">
          <div class="row-mini">
            <span
              class="swatch xs"
              aria-hidden="true"
              :style="{
                backgroundColor: `rgb(${m.domR},${m.domG},${m.domB})`,
              }"
            />
            <span class="arrow" aria-hidden="true">→</span>
            <span
              class="swatch xs"
              aria-hidden="true"
              :style="{
                backgroundColor: `rgb(${m.swR},${m.swG},${m.swB})`,
              }"
            />
          </div>
          <button
            type="button"
            class="linkish"
            @click="emit('copyText', m.swHex, '近似色 HEX')"
          >
            {{ m.pct.toFixed(1) }}% {{ m.swatchName }} · ΔE2000
            {{ m.deltaE.toFixed(1) }}
          </button>
        </li>
      </ul>
    </section>

    <section v-if="props.analysis.wcagDominantPair" class="block">
      <h2 class="h muted-h">WCAG コントラスト（主要色 1位 vs 2位）</h2>
      <p class="mono">
        比 {{ props.analysis.wcagDominantPair.contrastRatio.toFixed(2) }} :1
      </p>
    </section>

    <details v-if="props.analysis.theory" class="block fold">
      <summary class="fold-summary">色彩理論メモ（PCCS 風・非公式）</summary>
      <p class="muted small fold-disclaimer">
        {{ props.analysis.theory.disclaimerJa }}
      </p>
      <p v-if="props.analysis.theory.dominantHueSummaryJa" class="mono small">
        {{ props.analysis.theory.dominantHueSummaryJa }}
      </p>
      <p class="muted small fold-label">概論との対応（目安）</p>
      <ul class="outline-list">
        <li
          v-for="(line, i) in props.analysis.theory.outlineMappingJa"
          :key="i"
        >
          {{ line }}
        </li>
      </ul>
      <p class="muted small fold-label">支配色ごとの L*・C*・色相帯・トーン</p>
      <ul class="theory-dominant">
        <li
          v-for="(t, i) in props.analysis.theory.dominantDetails"
          :key="i"
          class="theory-row"
        >
          <span
            class="swatch xs"
            aria-hidden="true"
            :style="{ backgroundColor: t.hex }"
          />
          <div class="theory-row-text">
            <span class="mono small">{{ t.hex }}</span>
            <span class="muted small">
              L* {{ t.lStar.toFixed(1) }} · C*
              {{ t.cStar.toFixed(1) }} · h° {{ t.hDeg.toFixed(0) }}
            </span>
            <span class="small">{{ t.hueRegionJa }} · {{ t.pccsStyleToneJa }}</span>
          </div>
        </li>
      </ul>
    </details>

    <details
      v-if="props.analysis.harmonyScores.length"
      class="block fold"
    >
      <summary class="fold-summary">
        色相調和スコア（参考・％表示・内部は 0〜1）
      </summary>
      <p class="muted small fold-disclaimer">
        彩度が十分な支配色の加重に基づく参考値です。公式の調和理論の再現ではありません。詳細はツールバーの「用語集」を参照してください。
      </p>
      <ul class="harmony-legend">
        <li v-for="(line, hi) in harmonyScoreLegendLines" :key="hi">
          {{ line }}
        </li>
      </ul>
      <ul class="harmony-list" aria-label="色相調和スコアの一覧">
        <li
          v-for="h in props.analysis.harmonyScores"
          :key="h.id"
          class="harmony-row"
          :aria-label="`${h.labelJa}、スコア ${(h.score * 100).toFixed(0)} パーセント`"
        >
          <span class="harmony-label" aria-hidden="true">{{ h.labelJa }}</span>
          <span class="harmony-bar-wrap" aria-hidden="true">
            <span
              class="harmony-bar"
              :style="{ width: `${Math.round(h.score * 100)}%` }"
            />
          </span>
          <span class="mono harmony-val" aria-hidden="true">{{
            (h.score * 100).toFixed(0)
          }}%</span>
        </li>
      </ul>
    </details>

    <section class="block">
      <h2 class="h">EXIF</h2>
      <p v-if="!props.analysis.exif.length" class="muted">
        このファイルからは EXIF を読み取れませんでした。
      </p>
      <dl v-else class="exif">
        <template v-for="(row, i) in props.analysis.exif" :key="i">
          <dt>{{ row.label }}</dt>
          <dd>{{ row.value }}</dd>
        </template>
      </dl>
    </section>

    <details class="block json-export-fold">
      <summary class="json-export-summary">
        エクスポート用 JSON（コピー・保存と同じ・プレビュー base64 は省略）
      </summary>
      <pre class="json-export-pre">{{ exportJsonText }}</pre>
    </details>
  </aside>
</template>
