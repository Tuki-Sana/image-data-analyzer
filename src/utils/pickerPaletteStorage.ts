import type { PickerPaletteEntry } from "../types/analysis";

const LS_KEY = "imageMetadataAnalyzer.pickerPalette";
export const PICKER_PALETTE_MAX = 48;
/** ラベル文字数の上限（LocalStorage・UI 共通） */
export const PICKER_LABEL_MAX = 48;
/** カラーセット名の上限 */
export const PICKER_SET_NAME_MAX = 64;

export const PICKER_PALETTES_SCHEMA_VERSION = 1;

export interface PickerPaletteSet {
  id: string;
  /** 空なら UI では「無題」 */
  name: string;
  entries: PickerPaletteEntry[];
  updatedAt: string;
}

export interface PickerPalettesState {
  schemaVersion: number;
  activePaletteId: string;
  palettes: PickerPaletteSet[];
}

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

function isPaletteSetRaw(x: unknown): x is Record<string, unknown> {
  return x !== null && typeof x === "object";
}

export function createDefaultPickerPalettesState(): PickerPalettesState {
  const id = crypto.randomUUID();
  const now = new Date().toISOString();
  return {
    schemaVersion: PICKER_PALETTES_SCHEMA_VERSION,
    activePaletteId: id,
    palettes: [{ id, name: "", entries: [], updatedAt: now }],
  };
}

/** 旧形式（エントリ配列のみ）からの移行（テストからも利用可） */
export function pickerPalettesStateFromLegacyEntries(
  legacy: unknown[],
): PickerPalettesState {
  const entries = legacy.filter(isEntry).slice(0, PICKER_PALETTE_MAX);
  const id = crypto.randomUUID();
  const now = new Date().toISOString();
  return {
    schemaVersion: PICKER_PALETTES_SCHEMA_VERSION,
    activePaletteId: id,
    palettes: [{ id, name: "", entries, updatedAt: now }],
  };
}

function normalizePaletteSet(raw: unknown): PickerPaletteSet | null {
  if (!isPaletteSetRaw(raw)) return null;
  const p = raw as Record<string, unknown>;
  if (typeof p.id !== "string" || p.id.length === 0) return null;
  const name =
    typeof p.name === "string"
      ? p.name.slice(0, PICKER_SET_NAME_MAX)
      : "";
  const entries = Array.isArray(p.entries)
    ? p.entries.filter(isEntry).slice(0, PICKER_PALETTE_MAX)
    : [];
  const updatedAt =
    typeof p.updatedAt === "string" && p.updatedAt.length > 0
      ? p.updatedAt
      : new Date().toISOString();
  return { id: p.id, name, entries, updatedAt };
}

/**
 * localStorage に保存した JSON を解釈（旧配列形式と v1 オブジェクトの両方）。
 * テストでは localStorage なしで直接呼ぶ。
 */
export function parsePickerPalettesPersistedJson(
  parsed: unknown,
): PickerPalettesState {
  if (Array.isArray(parsed)) {
    return pickerPalettesStateFromLegacyEntries(parsed);
  }
  if (parsed === null || typeof parsed !== "object") {
    return createDefaultPickerPalettesState();
  }
  const root = parsed as Record<string, unknown>;
  if (!Array.isArray(root.palettes)) {
    return createDefaultPickerPalettesState();
  }
  const palettes: PickerPaletteSet[] = [];
  for (const item of root.palettes) {
    const s = normalizePaletteSet(item);
    if (s) palettes.push(s);
  }
  if (palettes.length === 0) {
    return createDefaultPickerPalettesState();
  }
  let activePaletteId =
    typeof root.activePaletteId === "string" ? root.activePaletteId : "";
  if (!palettes.some((p) => p.id === activePaletteId)) {
    activePaletteId = palettes[0]!.id;
  }
  const schemaVersion =
    typeof root.schemaVersion === "number"
      ? root.schemaVersion
      : PICKER_PALETTES_SCHEMA_VERSION;
  return { schemaVersion, activePaletteId, palettes };
}

export function getActivePaletteIndex(root: PickerPalettesState): number {
  const i = root.palettes.findIndex((p) => p.id === root.activePaletteId);
  return i >= 0 ? i : 0;
}

export function loadPickerPalettesState(): PickerPalettesState {
  try {
    const raw = localStorage.getItem(LS_KEY);
    if (!raw) return createDefaultPickerPalettesState();
    const parsed: unknown = JSON.parse(raw);
    return parsePickerPalettesPersistedJson(parsed);
  } catch {
    return createDefaultPickerPalettesState();
  }
}

export function savePickerPalettesState(state: PickerPalettesState): void {
  try {
    const toSave: PickerPalettesState = {
      ...state,
      palettes: state.palettes.map((p) => ({
        ...p,
        entries: p.entries.slice(0, PICKER_PALETTE_MAX),
      })),
    };
    localStorage.setItem(LS_KEY, JSON.stringify(toSave));
  } catch {
    /* ignore quota / private mode */
  }
}

export function paletteSetDisplayName(p: PickerPaletteSet): string {
  const t = p.name.trim();
  return t.length > 0 ? t : "無題";
}

/** `パレット 1` のようにアプリ既定の連番名だけマッチ（前後空白は無視） */
const AUTO_PALETTE_NAME_RE = /^パレット \d+$/;

export function isAutoNumberedPaletteName(name: string): boolean {
  return AUTO_PALETTE_NAME_RE.test(name.trim());
}

/**
 * 名前が `パレット N` 形式のセットだけ、配列順に 1 から振り直す。
 * 任意名のセットは変更しない。
 */
export function renumberAutoPaletteSetNames(
  palettes: PickerPaletteSet[],
): PickerPaletteSet[] {
  const now = new Date().toISOString();
  let seq = 0;
  return palettes.map((p) => {
    if (!isAutoNumberedPaletteName(p.name)) return p;
    seq += 1;
    const name = `パレット ${seq}`.slice(0, PICKER_SET_NAME_MAX);
    return { ...p, name, updatedAt: now };
  });
}
