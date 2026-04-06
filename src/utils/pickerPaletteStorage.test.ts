import { describe, expect, it } from "vitest";
import type { PickerPaletteSet } from "./pickerPaletteStorage";
import {
  PICKER_PALETTE_MAX,
  createDefaultPickerPalettesState,
  isAutoNumberedPaletteName,
  parsePickerPalettesPersistedJson,
  pickerPalettesStateFromLegacyEntries,
  renumberAutoPaletteSetNames,
} from "./pickerPaletteStorage";

describe("parsePickerPalettesPersistedJson", () => {
  it("migrates legacy entry array", () => {
    const state = parsePickerPalettesPersistedJson([
      {
        id: "a",
        r: 1,
        g: 2,
        b: 3,
        hex: "#010203",
        addedAt: "t",
      },
    ]);
    expect(state.palettes).toHaveLength(1);
    expect(state.activePaletteId).toBe(state.palettes[0]!.id);
    expect(state.palettes[0]!.entries).toHaveLength(1);
    expect(state.palettes[0]!.entries[0]!.hex).toBe("#010203");
  });

  it("caps legacy entries at max", () => {
    const many = Array.from({ length: PICKER_PALETTE_MAX + 10 }, (_, i) => ({
      id: `x${i}`,
      r: 0,
      g: 0,
      b: 0,
      hex: "#000000",
      addedAt: "t",
    }));
    const state = parsePickerPalettesPersistedJson(many);
    expect(state.palettes[0]!.entries).toHaveLength(PICKER_PALETTE_MAX);
  });

  it("reads v1 object shape", () => {
    const state = parsePickerPalettesPersistedJson({
      schemaVersion: 1,
      activePaletteId: "p2",
      palettes: [
        {
          id: "p1",
          name: "A",
          entries: [],
          updatedAt: "u1",
        },
        {
          id: "p2",
          name: "B",
          entries: [
            {
              id: "e1",
              r: 255,
              g: 0,
              b: 0,
              hex: "#FF0000",
              addedAt: "t",
            },
          ],
          updatedAt: "u2",
        },
      ],
    });
    expect(state.activePaletteId).toBe("p2");
    expect(state.palettes).toHaveLength(2);
    expect(state.palettes[1]!.entries).toHaveLength(1);
  });

  it("fixes invalid activePaletteId", () => {
    const state = parsePickerPalettesPersistedJson({
      schemaVersion: 1,
      activePaletteId: "missing",
      palettes: [
        { id: "only", name: "", entries: [], updatedAt: "u" },
      ],
    });
    expect(state.activePaletteId).toBe("only");
  });

  it("returns default for garbage", () => {
    const a = parsePickerPalettesPersistedJson(null);
    const b = parsePickerPalettesPersistedJson({});
    expect(a.palettes).toHaveLength(1);
    expect(b.palettes).toHaveLength(1);
  });
});

describe("createDefaultPickerPalettesState", () => {
  it("has one empty palette", () => {
    const s = createDefaultPickerPalettesState();
    expect(s.palettes).toHaveLength(1);
    expect(s.palettes[0]!.entries).toHaveLength(0);
    expect(s.activePaletteId).toBe(s.palettes[0]!.id);
  });
});

describe("pickerPalettesStateFromLegacyEntries", () => {
  it("wraps entries", () => {
    const s = pickerPalettesStateFromLegacyEntries([]);
    expect(s.palettes[0]!.entries).toHaveLength(0);
  });
});

describe("renumberAutoPaletteSetNames", () => {
  const mk = (id: string, name: string): PickerPaletteSet => ({
    id,
    name,
    entries: [],
    updatedAt: "u",
  });

  it("renumbers only パレット N pattern in order", () => {
    const out = renumberAutoPaletteSetNames([
      mk("a", "パレット 1"),
      mk("b", "肌パレット"),
      mk("c", "パレット 99"),
    ]);
    expect(out[0]!.name).toBe("パレット 1");
    expect(out[1]!.name).toBe("肌パレット");
    expect(out[2]!.name).toBe("パレット 2");
  });

  it("trims when matching", () => {
    expect(isAutoNumberedPaletteName("  パレット 3  ")).toBe(true);
    const out = renumberAutoPaletteSetNames([mk("x", "  パレット 10  ")]);
    expect(out[0]!.name).toBe("パレット 1");
  });

  it("does not match similar names", () => {
    expect(isAutoNumberedPaletteName("パレット1")).toBe(false);
    expect(isAutoNumberedPaletteName("パレット")).toBe(false);
    const out = renumberAutoPaletteSetNames([mk("x", "パレット1")]);
    expect(out[0]!.name).toBe("パレット1");
  });
});
