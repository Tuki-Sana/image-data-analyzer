import { describe, expect, it } from "vitest";
import { parseAnalysisExportJson } from "./analysisImport";

describe("parseAnalysisExportJson", () => {
  it("parses minimal export-like object", () => {
    const text = JSON.stringify({
      schemaVersion: 4,
      path: "/tmp/x.png",
      width: 100,
      height: 100,
      exif: [],
      previewJpegBase64Omitted: true,
      previewWidth: 100,
      previewHeight: 100,
      previewBgDark: false,
      dominants: [
        { r: 255, g: 0, b: 0, pct: 50, hex: "#FF0000" },
      ],
      openColorMatches: [],
      tailwindMatches: [],
      wcagDominantPair: null,
      theory: {
        disclaimerJa: "d",
        outlineMappingJa: [],
        dominantDetails: [],
        dominantHueSummaryJa: null,
      },
      harmonyScores: [
        { id: "legacy", labelJa: "旧実験値", score: 0.5 },
      ],
      gist: { lines: [], gistJa: "" },
    });
    const res = parseAnalysisExportJson(text);
    expect(res.ok).toBe(true);
    if (res.ok) {
      expect(res.analysis.dominants).toHaveLength(1);
      expect(res.analysis.previewJpegBase64).toBe("");
      expect(res.analysis.schemaVersion).toBe(4);
      expect(res.analysis.path).toBe("/tmp/x.png");
      expect(res.analysis.harmonyScores).toEqual([
        { id: "legacy", labelJa: "旧実験値", score: 0.5 },
      ]);
    }
  });

  it("keeps schema 3 imports compatible when harmony data is absent", () => {
    const text = JSON.stringify({
      schemaVersion: 3,
      path: "/Users/example/Pictures/legacy.png",
      width: 64,
      height: 32,
      dominants: [
        { r: 12, g: 34, b: 56, pct: 100, hex: "#0C2238" },
      ],
    });

    const res = parseAnalysisExportJson(text);
    expect(res.ok).toBe(true);
    if (res.ok) {
      expect(res.analysis.schemaVersion).toBe(3);
      expect(res.analysis.path).toBe("/Users/example/Pictures/legacy.png");
      expect(res.analysis.harmonyScores).toEqual([]);
    }
  });

  it("rejects empty object", () => {
    const res = parseAnalysisExportJson("{}");
    expect(res.ok).toBe(false);
  });

  it("rejects invalid json", () => {
    const res = parseAnalysisExportJson("not-json");
    expect(res.ok).toBe(false);
  });

  it("rejects when dominants is empty even if width/height are valid", () => {
    const text = JSON.stringify({ width: 100, height: 100, dominants: [] });
    const res = parseAnalysisExportJson(text);
    expect(res.ok).toBe(false);
    if (!res.ok) expect(res.error).toMatch(/支配色/);
  });

  it("rejects when width is 0 even if dominants exist", () => {
    const text = JSON.stringify({
      width: 0,
      height: 100,
      dominants: [{ r: 255, g: 0, b: 0, pct: 100, hex: "#FF0000" }],
    });
    const res = parseAnalysisExportJson(text);
    expect(res.ok).toBe(false);
    if (!res.ok) expect(res.error).toMatch(/幅・高さ/);
  });
});
