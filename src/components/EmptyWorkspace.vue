<script setup lang="ts">
import type { PickerPaletteEntry } from "../types/analysis";
import {
  PICKER_PALETTE_MAX,
  type PickerPaletteSet,
  type PickerPalettesState,
} from "../utils/pickerPaletteStorage";
import PickerPaletteSetBar from "./PickerPaletteSetBar.vue";

defineProps<{
  loading: boolean;
  /** エラー表示中でないときだけアイドル UI（パレット案内など）を出す */
  showIdleContent: boolean;
  paletteState: PickerPalettesState;
  activePaletteSet: PickerPaletteSet;
  canDeletePaletteSet: boolean;
  pickerPalette: PickerPaletteEntry[];
  activePaletteTitleLabel: string;
}>();

const emit = defineEmits<{
  setActivePaletteId: [id: string];
  updateActiveSetName: [name: string];
  newPaletteSet: [];
  duplicatePaletteSet: [];
  deletePaletteSet: [];
  importPickerPaletteReplace: [];
  importPickerPaletteMerge: [];
  copyPickerPaletteHexLines: [];
  copyPickerPaletteLabeledLines: [];
  copyPickerPaletteJson: [];
  savePickerPaletteJson: [];
}>();
</script>

<template>
  <div v-if="loading" class="workspace workspace--empty">
    <div class="empty">
      <p>読み込み中…</p>
    </div>
  </div>

  <div v-else-if="showIdleContent" class="workspace workspace--empty">
    <div class="empty">
      <p>「開く…」から画像を選択してください。</p>
      <div
        v-if="!pickerPalette.length"
        class="empty-palette-card empty-palette-card--import-only"
      >
        <p class="empty-palette-title">スポイトパレット</p>
        <p class="muted small empty-palette-note">
          JSON から読み込むと、画像を開かずに色だけ登録できます（最大
          {{ PICKER_PALETTE_MAX }} 色）。ファイルメニューの「読み込み」からも同じ操作ができます。
        </p>
        <PickerPaletteSetBar
          class="empty-palette-set-bar"
          :palettes="paletteState.palettes"
          :active-id="paletteState.activePaletteId"
          :set-name="activePaletteSet.name"
          :can-delete-set="canDeletePaletteSet"
          @update:active-id="emit('setActivePaletteId', $event)"
          @update:set-name="emit('updateActiveSetName', $event)"
          @new-set="emit('newPaletteSet')"
          @duplicate="emit('duplicatePaletteSet')"
          @delete-set="emit('deletePaletteSet')"
        />
        <p class="palette-actions-heading empty-palette-actions-label">
          読み込み
        </p>
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
      </div>
      <div v-else class="empty-palette-card">
        <p class="empty-palette-title">
          「{{ activePaletteTitleLabel }}」（{{ pickerPalette.length }} 色）
        </p>
        <p class="muted small empty-palette-note">
          LocalStorage に保存されています。画像なしでもコピー・書き出しできます。
        </p>
        <PickerPaletteSetBar
          class="empty-palette-set-bar"
          :palettes="paletteState.palettes"
          :active-id="paletteState.activePaletteId"
          :set-name="activePaletteSet.name"
          :can-delete-set="canDeletePaletteSet"
          @update:active-id="emit('setActivePaletteId', $event)"
          @update:set-name="emit('updateActiveSetName', $event)"
          @new-set="emit('newPaletteSet')"
          @duplicate="emit('duplicatePaletteSet')"
          @delete-set="emit('deletePaletteSet')"
        />
        <div class="empty-palette-swatches">
          <span
            v-for="e in pickerPalette.slice(0, 16)"
            :key="e.id"
            class="swatch sm empty-palette-swatch"
            aria-hidden="true"
            :title="e.hex"
            :style="{ backgroundColor: `rgb(${e.r},${e.g},${e.b})` }"
          />
          <span v-if="pickerPalette.length > 16" class="empty-palette-more"
            >+{{ pickerPalette.length - 16 }}</span
          >
        </div>
        <p class="palette-actions-heading empty-palette-actions-label">
          読み込み
        </p>
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
        <p class="palette-actions-heading empty-palette-actions-label">
          書き出し
        </p>
        <div class="palette-actions palette-actions-btns">
          <button
            type="button"
            class="palette-tool-btn"
            @click="emit('copyPickerPaletteHexLines')"
          >
            HEX（1 行 1 色）
          </button>
          <button
            type="button"
            class="palette-tool-btn"
            @click="emit('copyPickerPaletteLabeledLines')"
          >
            名前付きテキスト
          </button>
          <button
            type="button"
            class="palette-tool-btn"
            @click="emit('copyPickerPaletteJson')"
          >
            JSON をコピー
          </button>
          <button
            type="button"
            class="palette-tool-btn"
            @click="emit('savePickerPaletteJson')"
          >
            JSON を保存…
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.empty-palette-set-bar :deep(.palette-set-bar) {
  margin-bottom: 0.65rem;
  padding-bottom: 0.65rem;
}
</style>
