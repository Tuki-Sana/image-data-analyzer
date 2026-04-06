<script setup lang="ts">
import type { PickerPaletteEntry } from "../types/analysis";
import type { PickerPaletteSet, PickerPalettesState } from "../utils/pickerPaletteStorage";
import PaletteHomeCard from "./PaletteHomeCard.vue";

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
  openGlossary: [entryId: string];
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
      <PaletteHomeCard
        :palette-state="paletteState"
        :active-palette-set="activePaletteSet"
        :can-delete-palette-set="canDeletePaletteSet"
        :picker-palette="pickerPalette"
        :active-palette-title-label="activePaletteTitleLabel"
        @set-active-palette-id="emit('setActivePaletteId', $event)"
        @update-active-set-name="emit('updateActiveSetName', $event)"
        @new-palette-set="emit('newPaletteSet')"
        @duplicate-palette-set="emit('duplicatePaletteSet')"
        @delete-palette-set="emit('deletePaletteSet')"
        @import-picker-palette-replace="emit('importPickerPaletteReplace')"
        @import-picker-palette-merge="emit('importPickerPaletteMerge')"
        @copy-picker-palette-hex-lines="emit('copyPickerPaletteHexLines')"
        @copy-picker-palette-labeled-lines="emit('copyPickerPaletteLabeledLines')"
        @copy-picker-palette-json="emit('copyPickerPaletteJson')"
        @save-picker-palette-json="emit('savePickerPaletteJson')"
        @open-glossary="emit('openGlossary', $event)"
      />
    </div>
  </div>
</template>
