export type ColorAuxMode = "rgb" | "hsl";

/** sRGB 0–255 から CSS 慣例の HSL（H: 度, S/L: 0–100%） */
export function rgbToHsl(
  r: number,
  g: number,
  b: number,
): { h: number; s: number; l: number } {
  const rn = r / 255;
  const gn = g / 255;
  const bn = b / 255;
  const max = Math.max(rn, gn, bn);
  const min = Math.min(rn, gn, bn);
  const l = (max + min) / 2;
  let h = 0;
  let s = 0;

  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    switch (max) {
      case rn:
        h = ((gn - bn) / d + (gn < bn ? 6 : 0)) / 6;
        break;
      case gn:
        h = ((bn - rn) / d + 2) / 6;
        break;
      default:
        h = ((rn - gn) / d + 4) / 6;
        break;
    }
  }

  return { h: h * 360, s: s * 100, l: l * 100 };
}

export function formatAuxColor(mode: ColorAuxMode, r: number, g: number, b: number): string {
  if (mode === "rgb") {
    return `rgb(${r}, ${g}, ${b})`;
  }
  const { h, s, l } = rgbToHsl(r, g, b);
  return `hsl(${Math.round(h)}, ${s.toFixed(1)}%, ${l.toFixed(1)}%)`;
}

export function colorAuxModeLabel(mode: ColorAuxMode): string {
  return mode === "rgb" ? "RGB" : "HSL";
}
