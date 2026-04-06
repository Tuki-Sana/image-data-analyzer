<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { confirm as tauriConfirm, open, save } from "@tauri-apps/plugin-dialog";
import AnalysisSidePanel from "./components/AnalysisSidePanel.vue";
import AppHeader from "./components/AppHeader.vue";
import AppToolbar from "./components/AppToolbar.vue";
import EmptyWorkspace from "./components/EmptyWorkspace.vue";
import GlossaryModal from "./components/GlossaryModal.vue";
import ImagePreviewBlock from "./components/ImagePreviewBlock.vue";
import PdfExportSurface from "./components/PdfExportSurface.vue";
import type { Analysis, PickerPaletteEntry, PixelSample } from "./types/analysis";
import type { ColorAuxMode } from "./utils/colorFormat";
import { buildPdfFromElement } from "./utils/pdfExport";
import { APP_DISPLAY_NAME } from "./constants/appMeta";
import { installAppMenu } from "./setupAppMenu";
import { logAppError } from "./utils/appLog";
import { parseAnalysisExportJson } from "./utils/analysisImport";
import {
  mergePickerPalettes,
  parsePickerPaletteExport,
} from "./utils/pickerPaletteImport";
import {
  PICKER_LABEL_MAX,
  PICKER_PALETTE_MAX,
  PICKER_SET_NAME_MAX,
  getActivePaletteIndex,
  loadPickerPalettesState,
  renumberAutoPaletteSetNames,
  savePickerPalettesState,
} from "./utils/pickerPaletteStorage";

const appDisplayName = APP_DISPLAY_NAME;

const previewImageAlt = computed(() => {
  const a = analysis.value;
  if (!a?.path) return "プレビュー";
  const base = a.path.split(/[/\\]/).pop();
  return base ? `プレビュー: ${base}` : "プレビュー";
});

function isTauriWindow(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/** WebView の window.confirm が出ない環境向けに Tauri ダイアログを使う */
async function paletteDangerConfirm(message: string): Promise<boolean> {
  if (isTauriWindow()) {
    try {
      return await tauriConfirm(message, {
        title: appDisplayName,
        kind: "warning",
        okLabel: "OK",
        cancelLabel: "キャンセル",
      });
    } catch (e) {
      logAppError("paletteDangerConfirm (Tauri confirm)", e);
      return false;
    }
  }
  return window.confirm(message);
}

const COLOR_AUX_LS = "imageMetadataAnalyzer.colorAuxMode";

const loading = ref(false);
const error = ref("");
const analysis = ref<Analysis | null>(null);
const picked = ref<PixelSample | null>(null);
const toast = ref("");

function readColorAuxMode(): ColorAuxMode {
  try {
    const v = localStorage.getItem(COLOR_AUX_LS);
    if (v === "rgb" || v === "hsl") return v;
  } catch {
    /* ignore */
  }
  return "rgb";
}

const colorAuxMode = ref<ColorAuxMode>(readColorAuxMode());

watch(colorAuxMode, (m) => {
  try {
    localStorage.setItem(COLOR_AUX_LS, m);
  } catch {
    /* ignore */
  }
});

const pdfExportMount = ref(false);
const pdfHostRef = ref<HTMLElement | null>(null);
const glossaryOpen = ref(false);
const glossaryFocusEntryId = ref<string | null>(null);

const paletteState = ref(loadPickerPalettesState());

watch(
  paletteState,
  (v) => {
    savePickerPalettesState(v);
  },
  { deep: true },
);

const activePaletteSet = computed(() => {
  const r = paletteState.value;
  const i = getActivePaletteIndex(r);
  return r.palettes[i]!;
});

const pickerPalette = computed({
  get(): PickerPaletteEntry[] {
    return activePaletteSet.value.entries;
  },
  set(entries: PickerPaletteEntry[]) {
    const r = paletteState.value;
    const i = getActivePaletteIndex(r);
    const nextPals = r.palettes.map((p, j) =>
      j === i
        ? {
            ...p,
            entries: entries.slice(0, PICKER_PALETTE_MAX),
            updatedAt: new Date().toISOString(),
          }
        : p,
    );
    paletteState.value = { ...r, palettes: nextPals };
  },
});

const canDeletePaletteSet = computed(
  () => paletteState.value.palettes.length > 1,
);

/** 次に「パレットに追加」するときに付ける名前（任意） */
const paletteLabelDraft = ref("");

function setActivePaletteId(id: string) {
  if (!paletteState.value.palettes.some((p) => p.id === id)) return;
  paletteState.value = { ...paletteState.value, activePaletteId: id };
}

function updateActiveSetName(name: string) {
  const t = name.slice(0, PICKER_SET_NAME_MAX);
  const r = paletteState.value;
  const i = getActivePaletteIndex(r);
  const nextPals = r.palettes.map((p, j) =>
    j === i
      ? { ...p, name: t, updatedAt: new Date().toISOString() }
      : p,
  );
  paletteState.value = { ...r, palettes: nextPals };
}

function addPaletteSet() {
  const r = paletteState.value;
  const id = crypto.randomUUID();
  const now = new Date().toISOString();
  const n = r.palettes.length + 1;
  paletteState.value = {
    ...r,
    palettes: [
      ...r.palettes,
      { id, name: `パレット ${n}`, entries: [], updatedAt: now },
    ],
    activePaletteId: id,
  };
}

function duplicateActivePaletteSet() {
  const r = paletteState.value;
  const i = getActivePaletteIndex(r);
  const cur = r.palettes[i]!;
  const id = crypto.randomUUID();
  const now = new Date().toISOString();
  const entries = cur.entries.map((e) => ({
    ...e,
    id: crypto.randomUUID(),
    addedAt: new Date().toISOString(),
  }));
  const base = cur.name.trim();
  const copyLabel = base ? `${base} のコピー` : "無題のコピー";
  const name = copyLabel.slice(0, PICKER_SET_NAME_MAX);
  paletteState.value = {
    ...r,
    palettes: [...r.palettes, { id, name, entries, updatedAt: now }],
    activePaletteId: id,
  };
}

async function deleteActivePaletteSet() {
  const r = paletteState.value;
  if (r.palettes.length <= 1) {
    showToast("カラーセットは最低 1 つ必要です");
    return;
  }
  const label = activePaletteTitleLabel.value;
  const n = pickerPalette.value.length;
  const msg =
    `「${label}」というカラーセットを削除しますか？\n` +
    `中の色（${n} 色）もまとめて消えます。元に戻せません。`;
  if (!(await paletteDangerConfirm(msg))) return;
  const id = r.activePaletteId;
  const next = r.palettes.filter((p) => p.id !== id);
  const newActive = next[0]!.id;
  const palettes = renumberAutoPaletteSetNames(next);
  paletteState.value = {
    ...r,
    palettes,
    activePaletteId: newActive,
  };
  showToast("カラーセットを削除しました");
}

const activePaletteTitleLabel = computed(() => {
  const t = activePaletteSet.value.name.trim();
  return t.length > 0 ? t : "無題";
});

function buildExportObject(a: Analysis) {
  const { previewJpegBase64: _omit, ...rest } = a;
  return {
    ...rest,
    exportedAt: new Date().toISOString(),
    previewJpegBase64Omitted: true,
    note: "プレビュー画像の base64 はファイルサイズのため省略（分析数値のみの資産向け）",
  };
}

const previewSrc = computed(() => {
  const a = analysis.value;
  if (!a?.previewJpegBase64) return "";
  return `data:image/jpeg;base64,${a.previewJpegBase64}`;
});

/** コピー・保存ボタンと同じオブジェクト（プレビュー base64 省略済み） */
const exportJsonText = computed(() => {
  const a = analysis.value;
  if (!a) return "";
  return JSON.stringify(buildExportObject(a), null, 2);
});

function showToast(msg: string) {
  toast.value = msg;
  window.setTimeout(() => {
    toast.value = "";
  }, 2200);
}

async function openImage() {
  error.value = "";
  picked.value = null;
  const path = await open({
    multiple: false,
    filters: [
      {
        name: "画像",
        extensions: [
          "png",
          "jpg",
          "jpeg",
          "gif",
          "bmp",
          "webp",
          "ico",
          "tiff",
          "tif",
        ],
      },
    ],
  });
  if (path === null || Array.isArray(path)) return;
  loading.value = true;
  try {
    analysis.value = await invoke<Analysis>("analyze_image", { path });
  } catch (e) {
    analysis.value = null;
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

function closeImage() {
  analysis.value = null;
  picked.value = null;
  error.value = "";
}

async function samplePreviewAtClientXY(
  clientX: number,
  clientY: number,
  el: HTMLImageElement,
) {
  const a = analysis.value;
  if (!a || !el) return;
  const rect = el.getBoundingClientRect();
  const nx = ((clientX - rect.left) / rect.width) * el.naturalWidth;
  const ny = ((clientY - rect.top) / rect.height) * el.naturalHeight;
  const ox = Math.min(
    a.width - 1,
    Math.max(0, Math.floor((nx / a.previewWidth) * a.width)),
  );
  const oy = Math.min(
    a.height - 1,
    Math.max(0, Math.floor((ny / a.previewHeight) * a.height)),
  );
  try {
    picked.value = await invoke<PixelSample | null>("sample_pixel", {
      path: a.path,
      x: ox,
      y: oy,
    });
  } catch {
    picked.value = null;
  }
}

function onPreviewSample(payload: {
  clientX: number;
  clientY: number;
  img: HTMLImageElement;
}) {
  void samplePreviewAtClientXY(
    payload.clientX,
    payload.clientY,
    payload.img,
  );
}

async function copyJson() {
  const a = analysis.value;
  if (!a) return;
  const text = JSON.stringify(buildExportObject(a), null, 2);
  try {
    await navigator.clipboard.writeText(text);
    showToast("JSON をクリップボードにコピーしました");
  } catch {
    showToast("コピーに失敗しました");
  }
}

async function saveJson() {
  const a = analysis.value;
  if (!a) return;
  const outPath = await save({
    filters: [{ name: "JSON", extensions: ["json"] }],
    defaultPath: "color-analysis.json",
  });
  if (outPath === null) return;
  const text = JSON.stringify(buildExportObject(a), null, 2);
  try {
    await invoke("save_text_file", { path: outPath, contents: text });
    showToast("JSON を保存しました");
  } catch (e) {
    showToast(`保存に失敗: ${e}`);
  }
}

async function savePdf() {
  const a = analysis.value;
  if (!a) return;
  const outPath = await save({
    filters: [{ name: "PDF", extensions: ["pdf"] }],
    defaultPath: "color-analysis.pdf",
  });
  if (outPath === null) return;

  pdfExportMount.value = true;
  await nextTick();

  await new Promise<void>((resolve) => {
    const host = pdfHostRef.value;
    if (!host) {
      resolve();
      return;
    }
    const img = host.querySelector("img");
    if (!img?.src) {
      window.setTimeout(() => resolve(), 250);
      return;
    }
    if (img.complete) {
      window.setTimeout(() => resolve(), 80);
      return;
    }
    img.onload = () => resolve();
    img.onerror = () => resolve();
  });

  const host = pdfHostRef.value;
  if (!host) {
    pdfExportMount.value = false;
    showToast("PDF の準備に失敗しました");
    return;
  }

  try {
    const bytes = await buildPdfFromElement(host);
    await invoke("save_binary_file", {
      path: outPath,
      contents: Array.from(bytes),
    });
    showToast("PDF を保存しました");
  } catch (e) {
    showToast(`PDF の保存に失敗: ${e}`);
  } finally {
    pdfExportMount.value = false;
  }
}

async function copyText(text: string, label: string) {
  try {
    await navigator.clipboard.writeText(text);
    showToast(`${label} をコピーしました`);
  } catch {
    showToast("コピーに失敗しました");
  }
}

const pickerPaletteHexLines = computed(() =>
  pickerPalette.value.map((e) => e.hex).join("\n"),
);

/** 1 行あたり「名前 #HEX」または HEX のみ */
const pickerPaletteLabeledLines = computed(() =>
  pickerPalette.value
    .map((e) => {
      const name = e.label?.trim();
      return name ? `${name} ${e.hex}` : e.hex;
    })
    .join("\n"),
);

function buildPickerPaletteExportObject() {
  const setNameTrimmed = activePaletteSet.value.name.trim();
  const entries = pickerPalette.value.map((e) => {
    const base = {
      id: e.id,
      r: e.r,
      g: e.g,
      b: e.b,
      hex: e.hex,
      addedAt: e.addedAt,
    };
    const t = e.label?.trim();
    return t ? { ...base, label: t } : base;
  });
  return {
    exportedAt: new Date().toISOString(),
    kind: "pickerPalette" as const,
    ...(setNameTrimmed ? { name: setNameTrimmed } : {}),
    entries,
  };
}

function addPickedToPalette() {
  const p = picked.value;
  if (!p) return;
  if (pickerPalette.value.length >= PICKER_PALETTE_MAX) {
    showToast(`パレットは最大 ${PICKER_PALETTE_MAX} 色までです`);
    return;
  }
  const draft = paletteLabelDraft.value.trim().slice(0, PICKER_LABEL_MAX);
  const entry: PickerPaletteEntry = {
    id: crypto.randomUUID(),
    r: p.r,
    g: p.g,
    b: p.b,
    hex: p.hex,
    addedAt: new Date().toISOString(),
    ...(draft ? { label: draft } : {}),
  };
  pickerPalette.value = [entry, ...pickerPalette.value].slice(
    0,
    PICKER_PALETTE_MAX,
  );
  paletteLabelDraft.value = "";
  showToast("パレットに追加しました");
}

function setPaletteEntryLabel(id: string, raw: string) {
  const label = raw.trim().slice(0, PICKER_LABEL_MAX);
  pickerPalette.value = pickerPalette.value.map((e) => {
    if (e.id !== id) return e;
    if (!label) {
      return {
        id: e.id,
        r: e.r,
        g: e.g,
        b: e.b,
        hex: e.hex,
        addedAt: e.addedAt,
      };
    }
    return { ...e, label };
  });
}

async function removePaletteEntry(id: string) {
  const e = pickerPalette.value.find((x) => x.id === id);
  if (!e) return;
  const labelPart = e.label?.trim()
    ? `「${e.label.trim()}」`
    : "（名前なし）";
  const msg =
    `この色をパレットから削除しますか？\n${labelPart} ${e.hex}\n元に戻せません。`;
  if (!(await paletteDangerConfirm(msg))) return;
  pickerPalette.value = pickerPalette.value.filter((x) => x.id !== id);
}

async function clearPickerPalette() {
  if (pickerPalette.value.length === 0) return;
  const n = pickerPalette.value.length;
  const label = activePaletteTitleLabel.value;
  const msg =
    `「${label}」の色を ${n} 色すべて削除しますか？\n` +
    `カラーセット自体は残り、空のセットになります。元に戻せません。`;
  if (!(await paletteDangerConfirm(msg))) return;
  pickerPalette.value = [];
  showToast("このセットの色をすべて削除しました");
}

async function copyPickerPaletteHexLines() {
  if (pickerPalette.value.length === 0) return;
  await copyText(pickerPaletteHexLines.value, "HEX 一覧");
}

async function copyPickerPaletteLabeledLines() {
  if (pickerPalette.value.length === 0) return;
  await copyText(pickerPaletteLabeledLines.value, "名前付き一覧");
}

async function copyPickerPaletteJson() {
  if (pickerPalette.value.length === 0) return;
  const text = JSON.stringify(buildPickerPaletteExportObject(), null, 2);
  await copyText(text, "パレット JSON");
}

async function savePickerPaletteJson() {
  if (pickerPalette.value.length === 0) return;
  const outPath = await save({
    filters: [{ name: "JSON", extensions: ["json"] }],
    defaultPath: "spot-palette.json",
  });
  if (outPath === null) return;
  const text = JSON.stringify(buildPickerPaletteExportObject(), null, 2);
  try {
    await invoke("save_text_file", { path: outPath, contents: text });
    showToast("パレットを保存しました");
  } catch (e) {
    showToast(`保存に失敗: ${e}`);
  }
}

async function pickJsonFileContents(): Promise<string | null> {
  const filePath = await open({
    multiple: false,
    filters: [{ name: "JSON", extensions: ["json"] }],
  });
  if (filePath === null || Array.isArray(filePath)) return null;
  return invoke<string>("read_text_file", { path: filePath });
}

async function importPickerPaletteReplace() {
  try {
    const text = await pickJsonFileContents();
    if (text === null) return;
    let data: unknown;
    try {
      data = JSON.parse(text) as unknown;
    } catch {
      showToast("JSON の形式が正しくありません");
      return;
    }
    const res = parsePickerPaletteExport(data, () => crypto.randomUUID());
    if (!res.ok) {
      showToast(res.error);
      return;
    }
    pickerPalette.value = res.entries;
    if (res.setName !== undefined) {
      updateActiveSetName(res.setName);
    }
    showToast(`パレットを読み込みました（${res.entries.length} 色）`);
  } catch (e) {
    showToast(`読み込みに失敗: ${e}`);
  }
}

async function importPickerPaletteMerge() {
  try {
    const text = await pickJsonFileContents();
    if (text === null) return;
    let data: unknown;
    try {
      data = JSON.parse(text) as unknown;
    } catch {
      showToast("JSON の形式が正しくありません");
      return;
    }
    const res = parsePickerPaletteExport(data, () => crypto.randomUUID());
    if (!res.ok) {
      showToast(res.error);
      return;
    }
    pickerPalette.value = mergePickerPalettes(
      pickerPalette.value,
      res.entries,
    );
    showToast(`パレットを結合しました（計 ${pickerPalette.value.length} 色）`);
  } catch (e) {
    showToast(`読み込みに失敗: ${e}`);
  }
}

async function importAnalysisJson() {
  try {
    const text = await pickJsonFileContents();
    if (text === null) return;
    const res = parseAnalysisExportJson(text);
    if (!res.ok) {
      showToast(res.error);
      return;
    }
    analysis.value = res.analysis;
    picked.value = null;
    error.value = "";
    showToast("分析 JSON を読み込みました");
  } catch (e) {
    showToast(`読み込みに失敗: ${e}`);
  }
}

function openGlossary(focusId?: string | null) {
  glossaryFocusEntryId.value = focusId ?? null;
  glossaryOpen.value = true;
}

function onGlossaryClose() {
  glossaryOpen.value = false;
  glossaryFocusEntryId.value = null;
}

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
      ref="pdfHostRef"
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
