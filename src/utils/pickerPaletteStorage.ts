import type { PickerPaletteEntry } from "../types/analysis";

const LS_KEY = "imageMetadataAnalyzer.pickerPalette";
export const PICKER_PALETTE_MAX = 48;
/** ラベル文字数の上限（LocalStorage・UI 共通） */
export const PICKER_LABEL_MAX = 48;

function isEntry(x: unknown): x is PickerPaletteEntry {
  if (x === null || typeof x !== "object") return false;
  const o = x as Record<string, unknown>;
  if (
    typeof o.id !== "string" ||
    typeof o.r !== "number" ||
    typeof o.g !== "number" ||
    typeof o.b !== "number" ||
    typeof o.hex !== "string" ||
    typeof o.addedAt !== "string"
  ) {
    return false;
  }
  if ("label" in o && o.label != null && typeof o.label !== "string") {
    return false;
  }
  return true;
}

export function loadPickerPalette(): PickerPaletteEntry[] {
  try {
    const raw = localStorage.getItem(LS_KEY);
    if (!raw) return [];
    const parsed: unknown = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(isEntry).slice(0, PICKER_PALETTE_MAX);
  } catch {
    return [];
  }
}

export function savePickerPalette(entries: PickerPaletteEntry[]): void {
  try {
    const trimmed = entries.slice(0, PICKER_PALETTE_MAX);
    localStorage.setItem(LS_KEY, JSON.stringify(trimmed));
  } catch {
    /* ignore quota / private mode */
  }
}
