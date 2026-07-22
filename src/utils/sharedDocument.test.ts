import { describe, expect, it } from "vitest";
import type { Analysis } from "../types/analysis";
import {
  buildSharedAnalysisExport,
  sharedFileName,
} from "./sharedDocument";

const analysisFixture: Analysis = {
  schemaVersion: 4,
  path: "/Users/example/Pictures/sample.png",
  width: 100,
  height: 80,
  fileSizeBytes: 1234,
  fileSizeDisplay: "1.2 KB",
  modifiedDisplay: "2026-07-22 12:00",
  exif: [{ label: "機種", value: "Example Camera" }],
  previewJpegBase64: "private-preview-data",
  previewWidth: 100,
  previewHeight: 80,
  previewBgDark: false,
  dominants: [{ r: 1, g: 2, b: 3, pct: 100, hex: "#010203" }],
  openColorMatches: [],
  tailwindMatches: [],
  wcagDominantPair: null,
  theory: {
    disclaimerJa: "",
    outlineMappingJa: [],
    dominantDetails: [],
    dominantHueSummaryJa: null,
  },
  harmonyScores: [
    { id: "legacy", labelJa: "旧実験値", score: 0.75 },
  ],
  gist: { lines: [], gistJa: "" },
};

describe("sharedFileName", () => {
  it("removes a Unix absolute path", () => {
    expect(sharedFileName("/Users/example/Pictures/sample.png")).toBe(
      "sample.png",
    );
  });

  it("removes a Windows absolute path", () => {
    expect(sharedFileName("C:\\Users\\example\\Pictures\\sample.png")).toBe(
      "sample.png",
    );
  });
});

describe("buildSharedAnalysisExport", () => {
  it("keeps schema and metadata but removes absolute path and preview bytes", () => {
    const out = buildSharedAnalysisExport(analysisFixture);
    expect(out.schemaVersion).toBe(4);
    expect(out.path).toBe("sample.png");
    expect(out.exif).toEqual(analysisFixture.exif);
    expect(out.modifiedDisplay).toBe(analysisFixture.modifiedDisplay);
    expect(out).not.toHaveProperty("previewJpegBase64");
    expect(out).not.toHaveProperty("harmonyScores");
    expect(JSON.stringify(out)).not.toContain("/Users/example/");
    expect(JSON.stringify(out)).not.toContain("private-preview-data");
    expect(JSON.stringify(out)).not.toContain("旧実験値");
  });
});
