import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { computed, nextTick, ref, type Ref } from "vue";
import type { Analysis, PixelSample } from "../types/analysis";
import { parseAnalysisExportJson } from "../utils/analysisImport";
import { buildPdfFromElement } from "../utils/pdfExport";
import { readJsonTextFileDialog } from "../utils/readJsonTextFileDialog";

function buildExportObject(a: Analysis) {
  const { previewJpegBase64: _omit, ...rest } = a;
  return {
    ...rest,
    exportedAt: new Date().toISOString(),
    previewJpegBase64Omitted: true,
    note: "プレビュー画像の base64 はファイルサイズのため省略（分析数値のみの資産向け）",
  };
}

export function useImageAnalysisSession(options: {
  showToast: (msg: string) => void;
  /** PDF キャプチャ用ホスト要素（`useTemplateRef` 等で App 側から渡す） */
  pdfHostRef: Ref<HTMLElement | null>;
}) {
  const { showToast, pdfHostRef } = options;

  const loading = ref(false);
  const error = ref("");
  const analysis = ref<Analysis | null>(null);
  const picked = ref<PixelSample | null>(null);

  const pdfExportMount = ref(false);

  const previewSrc = computed(() => {
    const a = analysis.value;
    if (!a?.previewJpegBase64) return "";
    return `data:image/jpeg;base64,${a.previewJpegBase64}`;
  });

  const previewImageAlt = computed(() => {
    const a = analysis.value;
    if (!a?.path) return "プレビュー";
    const base = a.path.split(/[/\\]/).pop();
    return base ? `プレビュー: ${base}` : "プレビュー";
  });

  const exportJsonText = computed(() => {
    const a = analysis.value;
    if (!a) return "";
    return JSON.stringify(buildExportObject(a), null, 2);
  });

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

  async function importAnalysisJson() {
    try {
      const text = await readJsonTextFileDialog();
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

  return {
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
  };
}
