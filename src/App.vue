<script setup lang="ts">
import { onMounted, useTemplateRef } from "vue";
import AnalysisSidePanel from "./components/AnalysisSidePanel.vue";
import AppHeader from "./components/AppHeader.vue";
import AppToolbar from "./components/AppToolbar.vue";
import EmptyWorkspace from "./components/EmptyWorkspace.vue";
import GlossaryModal from "./components/GlossaryModal.vue";
import ImagePreviewBlock from "./components/ImagePreviewBlock.vue";
import PdfExportSurface from "./components/PdfExportSurface.vue";
import { APP_DISPLAY_NAME } from "./constants/appMeta";
import { useAppToast } from "./composables/useAppToast";
import { useColorAuxMode } from "./composables/useColorAuxMode";
import { useGlossaryModal } from "./composables/useGlossaryModal";
import { useImageAnalysisSession } from "./composables/useImageAnalysisSession";
import { usePaletteDangerConfirm } from "./composables/usePaletteDangerConfirm";
import { usePickerPaletteApp } from "./composables/usePickerPaletteApp";
import { installAppMenu } from "./setupAppMenu";

const appDisplayName = APP_DISPLAY_NAME;

const { toast, showToast, copyText } = useAppToast();
const { paletteDangerConfirm } = usePaletteDangerConfirm(appDisplayName);
const { colorAuxMode } = useColorAuxMode();
const {
  glossaryOpen,
  glossaryFocusEntryId,
  openGlossary,
  onGlossaryClose,
} = useGlossaryModal();

const pdfHostRef = useTemplateRef<HTMLElement>("pdfHost");

const {
  loading,
  error,
  analysis,
  picked,
  pdfExportMount,
  previewSrc,
  previewImageAlt,
  exportJsonText,
  openImage,
  closeImage,
  onPreviewSample,
  copyJson,
  saveJson,
  savePdf,
  importAnalysisJson,
} = useImageAnalysisSession({ showToast, pdfHostRef });

const {
  paletteState,
  paletteLabelDraft,
  activePaletteSet,
  pickerPalette,
  canDeletePaletteSet,
  activePaletteTitleLabel,
  setActivePaletteId,
  updateActiveSetName,
  addPaletteSet,
  duplicateActivePaletteSet,
  deleteActivePaletteSet,
  addPickedToPalette,
  setPaletteEntryLabel,
  removePaletteEntry,
  clearPickerPalette,
  copyPickerPaletteHexLines,
  copyPickerPaletteLabeledLines,
  copyPickerPaletteJson,
  savePickerPaletteJson,
  importPickerPaletteReplace,
  importPickerPaletteMerge,
} = usePickerPaletteApp({
  showToast,
  paletteDangerConfirm,
  copyText,
  picked,
});

onMounted(() => {
  void installAppMenu(
    {
      openImage,
      closeImage,
      copyJson,
      saveJson,
      savePdf,
      importPickerPaletteReplace,
      importPickerPaletteMerge,
      importAnalysisJson,
      openGlossary,
    },
    {
      onAsyncHandlerError: (label, err) => {
        showToast(`「${label}」でエラー: ${err}`);
      },
    },
  );
});
</script>

<template>
  <div class="app">
    <AppHeader :app-display-name="appDisplayName" />
    <AppToolbar :loading="loading" @open="openImage" />

    <p v-if="error" class="error" role="alert" aria-live="assertive">{{ error }}</p>

    <main id="main-content" class="app-main">
    <div v-if="analysis" class="workspace">
      <div class="main">
        <ImagePreviewBlock
          :preview-src="previewSrc"
          :preview-image-alt="previewImageAlt"
          :preview-bg-dark="analysis.previewBgDark"
          @sample="onPreviewSample"
        />

        <AnalysisSidePanel
          :analysis="analysis"
          :picked="picked"
          v-model:color-aux-mode="colorAuxMode"
          v-model:palette-label-draft="paletteLabelDraft"
          :export-json-text="exportJsonText"
          :palette-state="paletteState"
          :active-palette-set="activePaletteSet"
          :can-delete-palette-set="canDeletePaletteSet"
          :picker-palette="pickerPalette"
          @copy-text="copyText"
          @add-picked-to-palette="addPickedToPalette"
          @open-glossary="openGlossary"
          @set-active-palette-id="setActivePaletteId"
          @update-active-set-name="updateActiveSetName"
          @new-palette-set="addPaletteSet"
          @duplicate-palette-set="duplicateActivePaletteSet"
          @delete-palette-set="deleteActivePaletteSet"
          @set-palette-entry-label="setPaletteEntryLabel"
          @remove-palette-entry="removePaletteEntry"
          @import-picker-palette-replace="importPickerPaletteReplace"
          @import-picker-palette-merge="importPickerPaletteMerge"
          @copy-picker-palette-hex-lines="copyPickerPaletteHexLines"
          @copy-picker-palette-labeled-lines="copyPickerPaletteLabeledLines"
          @copy-picker-palette-json="copyPickerPaletteJson"
          @save-picker-palette-json="savePickerPaletteJson"
          @clear-picker-palette="clearPickerPalette"
        />
      </div>
    </div>

    <EmptyWorkspace
      v-else
      :loading="loading"
      :show-idle-content="!error"
      :palette-state="paletteState"
      :active-palette-set="activePaletteSet"
      :can-delete-palette-set="canDeletePaletteSet"
      :picker-palette="pickerPalette"
      :active-palette-title-label="activePaletteTitleLabel"
      @set-active-palette-id="setActivePaletteId"
      @update-active-set-name="updateActiveSetName"
      @new-palette-set="addPaletteSet"
      @duplicate-palette-set="duplicateActivePaletteSet"
      @delete-palette-set="deleteActivePaletteSet"
      @import-picker-palette-replace="importPickerPaletteReplace"
      @import-picker-palette-merge="importPickerPaletteMerge"
      @copy-picker-palette-hex-lines="copyPickerPaletteHexLines"
      @copy-picker-palette-labeled-lines="copyPickerPaletteLabeledLines"
      @copy-picker-palette-json="copyPickerPaletteJson"
      @save-picker-palette-json="savePickerPaletteJson"
      @open-glossary="openGlossary"
    />

    </main>

    <div
      v-if="toast"
      class="toast"
      role="status"
      aria-live="polite"
      aria-atomic="true"
    >
      {{ toast }}
    </div>

    <GlossaryModal
      :open="glossaryOpen"
      :focus-entry-id="glossaryFocusEntryId"
      @close="onGlossaryClose"
    />

    <div
      v-if="pdfExportMount && analysis"
      ref="pdfHost"
      class="pdf-export-host"
      aria-hidden="true"
    >
      <PdfExportSurface
        :analysis="analysis"
        :preview-data-url="previewSrc"
        :aux-mode="colorAuxMode"
      />
    </div>
  </div>
</template>

<style scoped>
.app {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg);
  color: var(--text);
  overflow: hidden;
}

.app-main {
  flex: 1 1 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.error {
  color: var(--danger);
  padding: 0.5rem 1.25rem;
  margin: 0;
  flex-shrink: 0;
  background: #fff5f5;
  border-bottom: 1px solid #f0d0d0;
  font-size: 1rem;
  line-height: 1.5;
}

.main {
  display: grid;
  grid-template-columns: 1fr min(380px, 38vw);
  gap: 1rem;
  padding: 1rem 1.25rem;
  max-width: 1200px;
  margin: 0 auto;
  width: 100%;
  box-sizing: border-box;
  flex: 1 1 0;
  min-height: 0;
  overflow: hidden;
  align-items: stretch;
}

@media (max-width: 800px) {
  .main {
    grid-template-columns: 1fr;
    grid-template-rows: minmax(0, 1fr) auto;
    overflow-y: auto;
  }
}

.pdf-export-host {
  position: fixed;
  left: -14000px;
  top: 0;
  z-index: -1;
  pointer-events: none;
}

.toast {
  position: fixed;
  bottom: 1.25rem;
  left: 50%;
  transform: translateX(-50%);
  background: var(--primary);
  color: #fff;
  padding: 0.5rem 1rem;
  border-radius: 8px;
  font-size: 1rem;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);
  z-index: 100;
}
</style>

<style>
:root {
  --primary: #2e2e32;
  --bg: #f8f8fa;
  --surface: #ececf0;
  --card: #ffffff;
  --text: #18181c;
  --muted: #62626a;
  --stroke: #babac2;
  --link: #2563eb;
  --danger: #c44040;
  font-family:
    system-ui,
    -apple-system,
    "Segoe UI",
    "Hiragino Sans",
    "Hiragino Kaku Gothic ProN",
    Meiryo,
    sans-serif;
  font-size: 16px;
  line-height: 1.5;
}

html,
body {
  margin: 0;
  height: 100%;
  overflow: hidden;
}

#app {
  height: 100%;
}
</style>
