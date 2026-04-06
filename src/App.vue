<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import GlossaryModal from "./components/GlossaryModal.vue";
import PdfExportSurface from "./components/PdfExportSurface.vue";
import type { Analysis, PickerPaletteEntry, PixelSample } from "./types/analysis";
import type { ColorAuxMode } from "./utils/colorFormat";
import { formatAuxColor } from "./utils/colorFormat";
import { harmonyScoreLegendLines } from "./constants/harmonyScoreLegend";
import { buildPdfFromElement } from "./utils/pdfExport";
import { APP_DISPLAY_NAME } from "./constants/appMeta";
import { installAppMenu } from "./setupAppMenu";
import {
  PICKER_LABEL_MAX,
  PICKER_PALETTE_MAX,
  loadPickerPalette,
  savePickerPalette,
} from "./utils/pickerPaletteStorage";

const appDisplayName = APP_DISPLAY_NAME;

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

const pickerPalette = ref<PickerPaletteEntry[]>(loadPickerPalette());
/** 次に「パレットに追加」するときに付ける名前（任意） */
const paletteLabelDraft = ref("");

watch(
  pickerPalette,
  (v) => {
    savePickerPalette(v);
  },
  { deep: true },
);

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

async function onPreviewClick(e: MouseEvent) {
  const a = analysis.value;
  if (!a) return;
  const el = e.currentTarget as HTMLImageElement;
  const rect = el.getBoundingClientRect();
  const nx = ((e.clientX - rect.left) / rect.width) * el.naturalWidth;
  const ny = ((e.clientY - rect.top) / rect.height) * el.naturalHeight;
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
  return {
    exportedAt: new Date().toISOString(),
    kind: "pickerPalette" as const,
    entries: pickerPalette.value.map((e) => {
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
    }),
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

function removePaletteEntry(id: string) {
  pickerPalette.value = pickerPalette.value.filter((e) => e.id !== id);
}

function clearPickerPalette() {
  if (pickerPalette.value.length === 0) return;
  pickerPalette.value = [];
  showToast("パレットを空にしました");
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

function openGlossary(focusId?: string | null) {
  glossaryFocusEntryId.value = focusId ?? null;
  glossaryOpen.value = true;
}

function onGlossaryClose() {
  glossaryOpen.value = false;
  glossaryFocusEntryId.value = null;
}

onMounted(() => {
  void installAppMenu({
    openImage,
    closeImage,
    copyJson,
    saveJson,
    savePdf,
    openGlossary,
  });
});
</script>

<template>
  <div class="app">
    <header class="header">
      <div class="header-inner">
        <div class="header-brand">
          <span class="header-mark" aria-hidden="true" />
          <div class="header-titles">
            <h1 class="title">{{ appDisplayName }}</h1>
            <p class="subtitle">
              <span class="subtitle-lead"
                >解像度・EXIF・主要色・パレット近似・色彩メモ・調和の目安</span
              >
            </p>
          </div>
        </div>
      </div>
      <div class="header-accent" aria-hidden="true" />
    </header>

    <div class="toolbar">
      <button
        type="button"
        class="btn btn-primary btn-open"
        :disabled="loading"
        @click="openImage"
      >
        {{ loading ? "読み込み中…" : "開く…" }}
      </button>
      <p class="toolbar-hint">
        閉じる・書き出し・用語集は、<strong>メニュー</strong>の「ファイル」「ヘルプ」から
      </p>
    </div>

    <p v-if="error" class="error">{{ error }}</p>

    <div v-if="analysis" class="workspace">
      <div class="main">
        <section
          class="preview-wrap"
          :class="analysis.previewBgDark ? 'canvas-dark' : 'canvas-light'"
        >
          <div class="preview-stage">
            <img
              :src="previewSrc"
              class="preview-img"
              alt="プレビュー"
              @click="onPreviewClick"
            />
          </div>
          <p class="hint">
            画像をクリックでその位置の色を取得（原画像座標でサンプル）
          </p>
        </section>

        <aside class="panel">
        <section class="block">
          <h2 class="h">ファイル</h2>
          <p class="mono path">{{ analysis.path }}</p>
          <p class="muted">
            {{ analysis.width }} × {{ analysis.height }} px
            <template v-if="analysis.fileSizeDisplay">
              · {{ analysis.fileSizeDisplay }}</template
            >
            <template v-if="analysis.modifiedDisplay">
              · 更新 {{ analysis.modifiedDisplay }}</template
            >
          </p>
        </section>

        <section
          v-if="picked || analysis.dominants.length"
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
                @click="colorAuxMode = 'rgb'"
              >
                RGB
              </button>
              <button
                type="button"
                class="toggle-btn"
                :class="{ 'toggle-btn--on': colorAuxMode === 'hsl' }"
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
              :style="{
                backgroundColor: `rgb(${picked.r},${picked.g},${picked.b})`,
              }"
            />
            <div class="dominant-text">
              <button
                type="button"
                class="linkish"
                @click="copyText(picked.hex, 'HEX')"
              >
                {{ picked.hex }}
              </button>
              <p class="mono small aux-line">
                {{ formatAuxColor(colorAuxMode, picked.r, picked.g, picked.b) }}
              </p>
              <button
                type="button"
                class="linkish linkish-tiny"
                @click="
                  copyText(
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
                @click="addPickedToPalette"
              >
                パレットに追加
              </button>
            </div>
          </div>
        </section>

        <section v-if="analysis" class="block block-picker-palette">
          <h2 class="h">スポイトパレット</h2>
          <p class="palette-lead">
            キャラ色など、名前を付けて整理できます（最大
            {{ PICKER_PALETTE_MAX }} 色）
          </p>
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
                @click="copyText(e.hex, 'HEX')"
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
                  setPaletteEntryLabel(
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
                @click="removePaletteEntry(e.id)"
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
          <p class="palette-actions-heading">書き出し</p>
          <div class="palette-actions palette-actions-btns">
            <button
              type="button"
              class="palette-tool-btn"
              :disabled="pickerPalette.length === 0"
              @click="copyPickerPaletteHexLines"
            >
              HEX（1 行 1 色）
            </button>
            <button
              type="button"
              class="palette-tool-btn"
              :disabled="pickerPalette.length === 0"
              @click="copyPickerPaletteLabeledLines"
            >
              名前付きテキスト
            </button>
            <button
              type="button"
              class="palette-tool-btn"
              :disabled="pickerPalette.length === 0"
              @click="copyPickerPaletteJson"
            >
              JSON をコピー
            </button>
            <button
              type="button"
              class="palette-tool-btn"
              :disabled="pickerPalette.length === 0"
              @click="savePickerPaletteJson"
            >
              JSON を保存…
            </button>
            <button
              type="button"
              class="palette-tool-btn palette-tool-btn--danger"
              :disabled="pickerPalette.length === 0"
              @click="clearPickerPalette"
            >
              すべて削除
            </button>
          </div>
        </section>

        <section v-if="analysis.dominants.length" class="block">
          <h2 class="h">主要色（推定）</h2>
          <ul class="list-dominant">
            <li v-for="(d, i) in analysis.dominants" :key="i" class="row-dominant">
              <span
                class="swatch sm"
                :style="{ backgroundColor: `rgb(${d.r},${d.g},${d.b})` }"
              />
              <div class="dominant-text">
                <button
                  type="button"
                  class="linkish"
                  @click="copyText(d.hex, 'HEX')"
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
                    copyText(
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

        <section v-if="analysis.gist.lines.length" class="block gist-block">
          <h2 class="h">ひと目サマリ</h2>
          <p
            v-for="(row, i) in analysis.gist.lines"
            :key="i"
            :class="gistRowClass(row.role)"
          >
            {{ row.text }}
          </p>
          <button
            type="button"
            class="linkish gist-copy"
            @click="copyText(analysis.gist.gistJa, '要約テキスト')"
          >
            gist_ja をコピー（改行付き全文）
          </button>
        </section>

        <section v-if="analysis.openColorMatches.length" class="block block-approx">
          <h2 class="h muted-h">Open Color 近似（ΔE76）</h2>
          <p class="block-lead">
            オープンソースの配色セット「Open Color」の名前付き色に、支配色がどれだけ近いかを示します（ΔE が小さいほど近い）。公式の正解色名ではありません。
            <button
              type="button"
              class="glossary-jump"
              @click="openGlossary('delta-e')"
            >
              用語集
            </button>
          </p>
          <ul class="list-match">
            <li v-for="(m, i) in analysis.openColorMatches" :key="i">
              <div class="row-mini">
                <span
                  class="swatch xs"
                  :style="{
                    backgroundColor: `rgb(${m.domR},${m.domG},${m.domB})`,
                  }"
                />
                <span class="arrow">→</span>
                <span
                  class="swatch xs"
                  :style="{
                    backgroundColor: `rgb(${m.swR},${m.swG},${m.swB})`,
                  }"
                />
              </div>
              <button
                type="button"
                class="linkish"
                @click="copyText(m.swHex, '近似色 HEX')"
              >
                {{ m.pct.toFixed(1) }}% {{ m.swatchName }} · ΔE
                {{ m.deltaE.toFixed(1) }}
              </button>
            </li>
          </ul>
        </section>

        <section v-if="analysis.tailwindMatches.length" class="block block-approx">
          <h2 class="h muted-h">Tailwind 近似（500/600/700 サブセット、ΔE76）</h2>
          <p class="block-lead">
            Web 向け CSS フレームワーク「Tailwind CSS」の標準色のうち、明るさ 500 / 600 / 700 だけを抜き出して比較しています（フルパレットではありません）。
            <button
              type="button"
              class="glossary-jump"
              @click="openGlossary('delta-e')"
            >
              用語集
            </button>
          </p>
          <ul class="list-match">
            <li v-for="(m, i) in analysis.tailwindMatches" :key="i">
              <div class="row-mini">
                <span
                  class="swatch xs"
                  :style="{
                    backgroundColor: `rgb(${m.domR},${m.domG},${m.domB})`,
                  }"
                />
                <span class="arrow">→</span>
                <span
                  class="swatch xs"
                  :style="{
                    backgroundColor: `rgb(${m.swR},${m.swG},${m.swB})`,
                  }"
                />
              </div>
              <button
                type="button"
                class="linkish"
                @click="copyText(m.swHex, '近似色 HEX')"
              >
                {{ m.pct.toFixed(1) }}% {{ m.swatchName }} · ΔE
                {{ m.deltaE.toFixed(1) }}
              </button>
            </li>
          </ul>
        </section>

        <section v-if="analysis.wcagDominantPair" class="block">
          <h2 class="h muted-h">WCAG コントラスト（主要色 1位 vs 2位）</h2>
          <p class="mono">
            比 {{ analysis.wcagDominantPair.contrastRatio.toFixed(2) }} :1
          </p>
        </section>

        <details v-if="analysis.theory" class="block fold">
          <summary class="fold-summary">色彩理論メモ（PCCS 風・非公式）</summary>
          <p class="muted small fold-disclaimer">
            {{ analysis.theory.disclaimerJa }}
          </p>
          <p v-if="analysis.theory.dominantHueSummaryJa" class="mono small">
            {{ analysis.theory.dominantHueSummaryJa }}
          </p>
          <p class="muted small fold-label">概論との対応（目安）</p>
          <ul class="outline-list">
            <li
              v-for="(line, i) in analysis.theory.outlineMappingJa"
              :key="i"
            >
              {{ line }}
            </li>
          </ul>
          <p class="muted small fold-label">支配色ごとの L*・C*・色相帯・トーン</p>
          <ul class="theory-dominant">
            <li
              v-for="(t, i) in analysis.theory.dominantDetails"
              :key="i"
              class="theory-row"
            >
              <span
                class="swatch xs"
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
          v-if="analysis.harmonyScores.length"
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
          <ul class="harmony-list">
            <li
              v-for="h in analysis.harmonyScores"
              :key="h.id"
              class="harmony-row"
            >
              <span class="harmony-label">{{ h.labelJa }}</span>
              <span class="harmony-bar-wrap" aria-hidden="true">
                <span
                  class="harmony-bar"
                  :style="{ width: `${Math.round(h.score * 100)}%` }"
                />
              </span>
              <span class="mono harmony-val">{{
                (h.score * 100).toFixed(0)
              }}%</span>
            </li>
          </ul>
        </details>

        <section class="block">
          <h2 class="h">EXIF</h2>
          <p v-if="!analysis.exif.length" class="muted">
            このファイルからは EXIF を読み取れませんでした。
          </p>
          <dl v-else class="exif">
            <template v-for="(row, i) in analysis.exif" :key="i">
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
      </div>
    </div>

    <div v-else-if="loading" class="workspace workspace--empty">
      <div class="empty">
        <p>読み込み中…</p>
      </div>
    </div>

    <div v-else-if="!error" class="workspace workspace--empty">
      <div class="empty">
        <p>「開く…」から画像を選択してください。</p>
        <div v-if="pickerPalette.length" class="empty-palette-card">
          <p class="empty-palette-title">
            保存中のスポイトパレット（{{ pickerPalette.length }} 色）
          </p>
          <p class="muted small empty-palette-note">
            LocalStorage に保存されています。画像なしでもコピー・書き出しできます。
          </p>
          <div class="empty-palette-swatches">
            <span
              v-for="e in pickerPalette.slice(0, 16)"
              :key="e.id"
              class="swatch sm empty-palette-swatch"
              :title="e.hex"
              :style="{ backgroundColor: `rgb(${e.r},${e.g},${e.b})` }"
            />
            <span v-if="pickerPalette.length > 16" class="empty-palette-more"
              >+{{ pickerPalette.length - 16 }}</span
            >
          </div>
          <div class="palette-actions palette-actions-btns">
            <button
              type="button"
              class="palette-tool-btn"
              @click="copyPickerPaletteHexLines"
            >
              HEX（1 行 1 色）
            </button>
            <button
              type="button"
              class="palette-tool-btn"
              @click="copyPickerPaletteLabeledLines"
            >
              名前付きテキスト
            </button>
            <button
              type="button"
              class="palette-tool-btn"
              @click="copyPickerPaletteJson"
            >
              JSON をコピー
            </button>
            <button
              type="button"
              class="palette-tool-btn"
              @click="savePickerPaletteJson"
            >
              JSON を保存…
            </button>
          </div>
        </div>
      </div>
    </div>

    <div v-if="toast" class="toast">{{ toast }}</div>

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

.header {
  position: relative;
  background: linear-gradient(180deg, #35353a 0%, var(--primary) 100%);
  color: #fff;
  padding: 0.85rem 1.25rem 0;
  border-bottom: none;
  flex-shrink: 0;
}

.header-inner {
  max-width: 1200px;
  margin: 0 auto;
  padding-bottom: 0.75rem;
}

.header-brand {
  display: flex;
  align-items: flex-start;
  gap: 0.85rem;
}

.header-mark {
  width: 2.5rem;
  height: 2.5rem;
  margin-top: 0.15rem;
  flex-shrink: 0;
  border-radius: 12px;
  background: conic-gradient(
    from 200deg,
    #f87171,
    #fbbf24,
    #4ade80,
    #38bdf8,
    #818cf8,
    #e879f9,
    #f87171
  );
  box-shadow:
    0 2px 6px rgba(0, 0, 0, 0.35),
    inset 0 1px 0 rgba(255, 255, 255, 0.35);
}

.header-titles {
  flex: 1;
  min-width: 0;
}

.header-accent {
  height: 3px;
  width: 100%;
  max-width: 1200px;
  margin: 0 auto;
  border-radius: 2px 2px 0 0;
  background: linear-gradient(
    90deg,
    #6366f1,
    #8b5cf6,
    #ec4899,
    #f97316,
    #eab308,
    #22c55e,
    #0ea5e9
  );
  opacity: 0.92;
}

.title {
  margin: 0;
  font-size: 1.35rem;
  font-weight: 800;
  letter-spacing: -0.02em;
  line-height: 1.2;
}

.subtitle {
  margin: 0.35rem 0 0;
}

.subtitle-lead {
  font-size: 0.9375rem;
  line-height: 1.45;
  font-weight: 500;
  opacity: 0.96;
}

.toolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.65rem 1rem;
  padding: 0.65rem 1.25rem;
  background: #f3f3f5;
  border-bottom: 1px solid #d8d8de;
  width: 100%;
  box-sizing: border-box;
  flex-shrink: 0;
}

.toolbar-hint {
  margin: 0;
  flex: 1 1 12rem;
  font-size: 0.8125rem;
  line-height: 1.45;
  color: #5c5c66;
  font-weight: 500;
}

.toolbar-hint strong {
  font-weight: 600;
  color: #3a3a42;
}

/* 右パネル末尾：ツールバー直下だとプレビューと分析の流れを切るためサイドバー内に配置 */
.json-export-fold {
  border: 1px solid var(--stroke);
  border-radius: 10px;
  background: var(--surface);
  overflow: hidden;
}

.json-export-summary {
  padding: 0.55rem 0.9rem;
  cursor: pointer;
  font-size: 1rem;
  color: var(--text);
  font-weight: 600;
  list-style: none;
  user-select: none;
}

.json-export-summary::-webkit-details-marker {
  display: none;
}

.json-export-summary::before {
  content: "▸ ";
  display: inline-block;
  transition: transform 0.1s ease;
}

.json-export-fold[open] > .json-export-summary::before {
  transform: rotate(90deg);
}

.json-export-pre {
  margin: 0;
  padding: 0.65rem 0.85rem;
  max-height: min(360px, 42vh);
  overflow: auto;
  font-family: ui-monospace, monospace;
  font-size: 0.875rem;
  line-height: 1.5;
  color: var(--text);
  background: var(--card);
  border-top: 1px solid var(--stroke);
  white-space: pre;
  word-break: normal;
}

.btn {
  border-radius: 8px;
  padding: 0.5rem 1.1rem;
  font-size: 1rem;
  font-weight: 600;
  cursor: pointer;
  border: 1px solid transparent;
  transition:
    background 0.14s ease,
    border-color 0.14s ease,
    box-shadow 0.14s ease,
    transform 0.1s ease,
    filter 0.14s ease;
}

.btn:focus-visible {
  outline: 2px solid var(--link);
  outline-offset: 2px;
}

.btn:focus:not(:focus-visible) {
  outline: none;
}

.btn:disabled {
  opacity: 0.48;
  cursor: not-allowed;
  box-shadow: none;
  transform: none;
}

/* ツールバー「開く」: システム寄りのアクセント（明るい帯の上で主役になる塗り） */
.btn-primary {
  color: #fff;
  background: linear-gradient(180deg, #0b84ff 0%, #0071e3 100%);
  border-color: rgba(0, 0, 0, 0.12);
  box-shadow:
    0 1px 0 rgba(255, 255, 255, 0.2) inset,
    0 1px 2px rgba(0, 50, 120, 0.22);
}

.btn-primary:hover:not(:disabled) {
  filter: brightness(1.06);
  box-shadow:
    0 1px 0 rgba(255, 255, 255, 0.22) inset,
    0 2px 8px rgba(0, 80, 180, 0.28);
}

.btn-primary:active:not(:disabled) {
  filter: brightness(0.96);
  box-shadow: 0 1px 1px rgba(0, 40, 100, 0.2) inset;
}

.btn-open {
  border-radius: 999px;
  padding: 0.42rem 1.35rem;
  font-size: 0.9375rem;
  font-weight: 600;
  letter-spacing: 0.01em;
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

.workspace {
  flex: 1 1 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.workspace--empty {
  align-items: center;
  justify-content: center;
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

  .panel {
    max-height: 45vh;
  }
}

.preview-wrap {
  border-radius: 12px;
  padding: 12px;
  border: 1px solid var(--stroke);
  min-height: 0;
  min-width: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.preview-stage {
  flex: 1 1 0;
  min-height: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.canvas-light {
  background: repeating-conic-gradient(#e8ecf2 0% 25%, #d8dee8 0% 50%) 50% /
    20px 20px;
}

.canvas-dark {
  background: repeating-conic-gradient(#34383e 0% 25%, #40444c 0% 50%) 50% /
    20px 20px;
}

.preview-img {
  max-width: 100%;
  max-height: 100%;
  width: auto;
  height: auto;
  object-fit: contain;
  display: block;
  border-radius: 8px;
  cursor: crosshair;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.12);
}

.hint {
  margin: 0.5rem 0 0;
  font-size: 1rem;
  line-height: 1.45;
  flex-shrink: 0;
}

/* チェッカー背景が暗いときはグレー文字が沈むため、コントラストを確保 */
.preview-wrap.canvas-light .hint {
  color: #3d3d45;
}

.preview-wrap.canvas-dark .hint {
  color: rgba(255, 255, 255, 0.92);
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.45);
}

.panel {
  background: var(--card);
  border: 1px solid var(--stroke);
  border-radius: 12px;
  padding: 1rem;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  align-self: stretch;
  font-size: 1rem;
  line-height: 1.55;
}

.block {
  margin-bottom: 1.25rem;
}

.block:last-child {
  margin-bottom: 0;
}

.block-tight {
  margin-bottom: 0.85rem;
}

.color-mode-hint {
  margin: 0 0 0.35rem;
}

.color-mode-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.toggle {
  display: inline-flex;
  border: 1px solid var(--stroke);
  border-radius: 6px;
  overflow: hidden;
}

.toggle-btn {
  padding: 0.28rem 0.75rem;
  font-size: 0.78rem;
  font-weight: 500;
  border: none;
  background: var(--card);
  cursor: pointer;
  color: var(--text);
}

.toggle-btn--on {
  background: var(--primary);
  color: #fff;
}

.toggle-btn:hover:not(.toggle-btn--on) {
  background: var(--surface);
}

.row-dominant {
  display: flex;
  align-items: flex-start;
  gap: 0.65rem;
  margin-bottom: 0.55rem;
}

.dominant-text {
  min-width: 0;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
}

.aux-line {
  margin: 0.15rem 0 0;
}

.linkish-tiny {
  font-size: 0.9375rem;
  margin-top: 0.15rem;
}

.pdf-export-host {
  position: fixed;
  left: -14000px;
  top: 0;
  z-index: -1;
  pointer-events: none;
}

.gist-block {
  background: linear-gradient(180deg, #f0f4fc 0%, #fafbff 100%);
  border: 1px solid #c8d4ec;
  border-radius: 10px;
  padding: 0.85rem 1rem;
}

.gist-label {
  margin: 0.5rem 0 0.15rem;
  font-weight: 600;
}

.gist-label:first-of-type {
  margin-top: 0.25rem;
}

.gist-line {
  margin: 0.2rem 0 0;
  line-height: 1.45;
}

.gist-foot {
  margin: 0.35rem 0 0;
  line-height: 1.4;
}

.gist-copy {
  display: inline-block;
  margin-top: 0.65rem;
  font-size: 1rem;
}

/* gist 本文は .small より優先して 16px 相当を確保 */
.gist-block .gist-line {
  font-size: 1rem;
  line-height: 1.6;
}

.h {
  margin: 0 0 0.5rem;
  font-size: 1.125rem;
  font-weight: 700;
}

.muted-h {
  color: var(--muted);
  font-weight: 600;
  font-size: 1.0625rem;
}

.path {
  word-break: break-all;
  font-size: 0.9375rem;
  margin: 0 0 0.35rem;
}

.muted {
  color: var(--muted);
  font-size: 1rem;
  margin: 0;
}

.small {
  font-size: 1rem;
  margin: 0.2rem 0 0;
}

.mono {
  font-family: ui-monospace, monospace;
  font-size: 1rem;
}

.row-color {
  display: flex;
  align-items: center;
  gap: 0.65rem;
  margin-bottom: 0.45rem;
}

.swatch {
  width: 36px;
  height: 36px;
  border-radius: 6px;
  border: 1px solid var(--stroke);
  flex-shrink: 0;
}

.swatch.sm {
  width: 28px;
  height: 28px;
  border-radius: 4px;
}

.swatch.xs {
  width: 22px;
  height: 22px;
  border-radius: 4px;
}

.linkish {
  background: none;
  border: none;
  padding: 0;
  font: inherit;
  text-align: left;
  color: var(--link);
  cursor: pointer;
  text-decoration: none;
}

.linkish:hover {
  text-decoration: underline;
}

.linkish:disabled {
  opacity: 0.45;
  cursor: not-allowed;
  text-decoration: none;
}

.linkish-danger {
  color: var(--danger);
}

.block-picker-palette {
  /* 親 .panel がカード枠のため、ここは中身の区切りのみ */
  margin-top: 0.25rem;
}

.palette-lead {
  margin: 0 0 0.65rem;
  font-size: 0.9375rem;
  color: var(--muted);
  line-height: 1.45;
}

.palette-chip-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 0.65rem 0.75rem;
  margin: 0 0 0.75rem;
  padding: 0.65rem 0.5rem;
  border-radius: 10px;
  background: var(--surface);
  border: 1px solid var(--stroke);
}

.palette-chip-cell {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 0.2rem;
  width: 5.75rem;
}

.palette-chip {
  align-self: center;
  width: 44px;
  height: 44px;
  border-radius: 12px;
  border: 1px solid rgba(0, 0, 0, 0.14);
  cursor: pointer;
  padding: 0;
  box-shadow:
    0 1px 2px rgba(0, 0, 0, 0.06),
    inset 0 1px 0 rgba(255, 255, 255, 0.25);
  transition: transform 0.1s ease, box-shadow 0.1s ease;
}

.palette-chip:hover {
  transform: scale(1.04);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.palette-chip-hex {
  font-size: 0.625rem;
  color: var(--muted);
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-align: center;
}

.palette-label-input {
  width: 100%;
  box-sizing: border-box;
  padding: 0.2rem 0.25rem;
  font-size: 0.6875rem;
  line-height: 1.2;
  border: 1px solid var(--stroke);
  border-radius: 6px;
  background: #fff;
  color: var(--text);
  text-align: center;
}

.palette-label-input::placeholder {
  color: #aaa;
}

.palette-label-input:focus {
  outline: 2px solid var(--link);
  outline-offset: 0;
  border-color: transparent;
}

.palette-chip-remove {
  position: absolute;
  top: -6px;
  right: -4px;
  width: 1.35rem;
  height: 1.35rem;
  padding: 0;
  border: none;
  border-radius: 999px;
  background: #fff;
  color: #666;
  font-size: 1rem;
  line-height: 1;
  cursor: pointer;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.18);
}

.palette-chip-remove:hover {
  background: #fff0f0;
  color: var(--danger);
}

.palette-empty-hint {
  margin: 0 0 0.6rem;
  font-size: 0.9375rem;
  color: var(--muted);
  line-height: 1.45;
}

.palette-details {
  margin: 0 0 0.65rem;
  border-radius: 8px;
  border: 1px solid var(--stroke);
  background: var(--surface);
  overflow: hidden;
}

.palette-details-summary {
  padding: 0.4rem 0.65rem;
  cursor: pointer;
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--text);
  list-style: none;
  user-select: none;
}

.palette-details-summary::-webkit-details-marker {
  display: none;
}

.palette-details-summary::before {
  content: "▸ ";
  display: inline-block;
  transition: transform 0.1s ease;
}

.palette-details[open] > .palette-details-summary::before {
  transform: rotate(90deg);
}

.palette-details-body {
  margin: 0;
  padding: 0 0.65rem 0.55rem;
  border-top: 1px solid var(--stroke);
  padding-top: 0.45rem;
}

.palette-actions-heading {
  margin: 0 0 0.35rem;
  font-size: 0.8125rem;
  font-weight: 700;
  color: #5a5a62;
  letter-spacing: 0.02em;
  text-transform: uppercase;
}

.palette-actions-btns {
  border-top: none;
  margin-top: 0;
  padding-top: 0.65rem;
}

.palette-tool-btn {
  font-size: 0.875rem;
  font-weight: 600;
  padding: 0.38rem 0.75rem;
  border-radius: 8px;
  border: 1px solid #c5c5d0;
  background: #fff;
  color: var(--text);
  cursor: pointer;
  transition:
    background 0.12s ease,
    border-color 0.12s ease;
}

.palette-tool-btn:hover:not(:disabled) {
  background: #f4f4f8;
  border-color: #a8a8b8;
}

.palette-tool-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.palette-tool-btn--danger {
  border-color: #e8c4c4;
  color: #b91c1c;
}

.palette-tool-btn--danger:hover:not(:disabled) {
  background: #fff5f5;
  border-color: #e08080;
}

.block-approx {
  padding-top: 0.25rem;
}

.block-lead {
  margin: 0 0 0.65rem;
  font-size: 0.9375rem;
  line-height: 1.55;
  color: var(--muted);
}

.glossary-jump {
  display: inline;
  margin-left: 0.35rem;
  padding: 0.15rem 0.5rem;
  font-size: 0.8125rem;
  font-weight: 600;
  border-radius: 6px;
  border: 1px solid #c8d4ec;
  background: linear-gradient(180deg, #f5f8ff 0%, #eef3fc 100%);
  color: var(--link);
  cursor: pointer;
  vertical-align: baseline;
}

.glossary-jump:hover {
  border-color: var(--link);
  background: #e8efff;
}

.btn-palette-add {
  margin-top: 0.5rem;
  align-self: flex-start;
  border-radius: 8px;
  padding: 0.35rem 0.75rem;
  font-size: 0.9375rem;
  font-weight: 600;
  cursor: pointer;
  border: 1px solid #c4c2d8;
  background: #fff;
  color: var(--text);
  transition:
    background 0.12s ease,
    border-color 0.12s ease;
}

.btn-palette-add:hover {
  background: #f4f3fa;
  border-color: #a8a4c0;
}

.palette-draft-wrap {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.25rem;
  margin-top: 0.45rem;
  width: 100%;
  max-width: 16rem;
}

.palette-draft-label {
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--muted);
}

.palette-draft-input {
  width: 100%;
  box-sizing: border-box;
  padding: 0.35rem 0.5rem;
  font-size: 0.9375rem;
  border: 1px solid var(--stroke);
  border-radius: 8px;
  background: #fff;
  color: var(--text);
}

.palette-draft-input::placeholder {
  color: #9a9aa8;
}

.palette-draft-input:focus {
  outline: 2px solid var(--link);
  outline-offset: 1px;
  border-color: transparent;
}

.btn-palette-remove {
  flex-shrink: 0;
  font-size: 0.8125rem;
  padding: 0.2rem 0.45rem;
  border-radius: 6px;
  border: 1px solid var(--stroke);
  background: var(--card);
  color: var(--muted);
  cursor: pointer;
}

.btn-palette-remove:hover {
  color: var(--danger);
  border-color: #e0b4b4;
  background: #fff8f8;
}

.palette-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem 0.85rem;
  margin-top: 0.55rem;
  padding-top: 0.5rem;
  border-top: 1px solid var(--stroke);
}

/* パレット書き出し：2 列で揃え、危険操作だけ下段フル幅 */
.palette-actions.palette-actions-btns {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.5rem 0.65rem;
  align-items: stretch;
}

.palette-actions.palette-actions-btns .palette-tool-btn {
  width: 100%;
  min-width: 0;
  box-sizing: border-box;
  justify-self: stretch;
}

.palette-actions.palette-actions-btns .palette-tool-btn--danger {
  grid-column: 1 / -1;
}

.empty-palette-card {
  margin-top: 1.5rem;
  max-width: 28rem;
  padding: 1rem 1.15rem;
  border-radius: 12px;
  border: 1px solid var(--stroke);
  background: var(--card);
  text-align: left;
}

.empty-palette-title {
  margin: 0 0 0.25rem;
  font-weight: 700;
  font-size: 1.0625rem;
}

.empty-palette-note {
  margin: 0 0 0.65rem;
}

.empty-palette-swatches {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.35rem;
  margin-bottom: 0.75rem;
}

.empty-palette-swatch {
  cursor: default;
}

.empty-palette-more {
  font-size: 0.875rem;
  color: var(--muted);
  font-weight: 600;
}

.list-dominant,
.list-match {
  list-style: none;
  margin: 0;
  padding: 0;
}

.list-match li {
  margin-bottom: 0.65rem;
}

.row-mini {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  margin-bottom: 0.2rem;
}

.arrow {
  color: var(--muted);
  font-size: 1rem;
}

.exif {
  margin: 0;
  font-size: 1rem;
}

.exif dt {
  font-weight: 700;
  margin-top: 0.35rem;
}

.exif dd {
  margin: 0.1rem 0 0 0;
}

.fold {
  border: 1px solid var(--stroke);
  border-radius: 10px;
  padding: 0.5rem 0.75rem;
  background: var(--surface);
}

.fold-summary {
  cursor: pointer;
  font-weight: 600;
  font-size: 1.0625rem;
  color: var(--text);
  list-style: none;
}

.fold-summary::-webkit-details-marker {
  display: none;
}

.fold-summary::before {
  content: "▸ ";
  display: inline-block;
  transition: transform 0.12s ease;
}

details[open] > .fold-summary::before {
  transform: rotate(90deg);
}

.fold-disclaimer {
  margin: 0.5rem 0 0.35rem;
  font-size: 1rem;
  line-height: 1.6;
}

.fold-label {
  margin: 0.65rem 0 0.25rem;
  font-weight: 600;
  font-size: 1rem;
}

.outline-list {
  margin: 0;
  padding-left: 1.1rem;
  font-size: 1rem;
  line-height: 1.55;
  color: var(--text);
}

.theory-dominant {
  list-style: none;
  margin: 0;
  padding: 0;
}

.theory-row {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.theory-row-text {
  display: flex;
  flex-direction: column;
  gap: 0.1rem;
  min-width: 0;
}

.harmony-legend {
  margin: 0.4rem 0 0.5rem;
  padding-left: 1.1rem;
  font-size: 1rem;
  line-height: 1.6;
  color: var(--text);
}

.harmony-legend li {
  margin-bottom: 0.25rem;
}

.harmony-list {
  list-style: none;
  margin: 0.35rem 0 0;
  padding: 0;
}

.harmony-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 72px 2.5rem;
  align-items: center;
  gap: 0.4rem;
  font-size: 1rem;
  margin-bottom: 0.35rem;
}

.harmony-label {
  min-width: 0;
}

.harmony-bar-wrap {
  height: 8px;
  background: var(--stroke);
  border-radius: 4px;
  overflow: hidden;
}

.harmony-bar {
  display: block;
  height: 100%;
  background: var(--primary);
  border-radius: 3px;
  min-width: 2px;
}

.harmony-val {
  text-align: right;
  font-size: 1rem;
}

.empty {
  padding: 2rem;
  text-align: center;
  color: var(--muted);
  font-size: 1rem;
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
