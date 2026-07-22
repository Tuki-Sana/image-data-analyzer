# Teinte

[日本語](README.md)

**A local-first desktop aid for inspecting image colors, organizing colors used
in illustration work, and recording the results. It is not an image editor.**

Teinte analyzes images locally using perceptual color models instead of relying
only on raw RGB distance. It estimates dominant colors with k-means clustering
in CIE L\*a\*b\* space, compares colors with ΔE2000, calculates the WCAG contrast
ratio for the two leading dominant colors, and maps results to named Open Color
and Tailwind palettes.

Picked colors can be named, grouped into multiple sets, and exported as JSON.
Analysis reports can also be exported as rasterized PDFs. Images remain on the
local device and are not uploaded for analysis.

**[Download v0.4.3 →](https://github.com/tsukasa-art/teinte/releases/tag/v0.4.3)**<br>
Release artifacts are available for macOS (Apple Silicon and Intel) and Windows.

## Highlights

- Dominant-color estimates using Lab k-means with deterministic farthest-first initialization
- ΔE2000 nearest-color matching against Open Color and Tailwind palettes
- WCAG contrast ratio for the two leading estimated dominant colors
- Unofficial PCCS-style labels derived from Lab values, not official PCCS classifications
- Eyedropper palettes with up to 48 named colors per set and multiple saved sets
- JSON import and export for analysis results and palettes
- PDF export containing the preview, file name, colors, contrast, and EXIF data
- Local desktop processing without image uploads or analytics

## Technology

| Area | Technology |
|---|---|
| UI | Vue 3, TypeScript, and Vite |
| Desktop shell | Tauri 2 |
| Image and color analysis | Rust |
| Verification | Vitest, `cargo test`, and GitHub Actions |

The Rust implementation is separated into image analysis, palette matching,
color-theory, and legacy hue-layout modules. The Vue frontend separates
application state and user interactions into composables and focused UI components.

- [Architecture](docs/architecture.md)
- [Image-analysis implementation notes](docs/image-analysis.md)
- [Changelog](CHANGELOG.md)

## Build and test

```bash
pnpm install
pnpm run test
pnpm run build
cd src-tauri && cargo test
```

Run the desktop application in development mode with:

```bash
pnpm tauri dev
```

## Release boundary

The published macOS builds are not signed or notarized with an Apple Developer
certificate, so Gatekeeper may require an explicit right-click **Open** action
or removal of the quarantine attribute. See the Japanese README and the release
page for the current installation notes.

## Privacy and known limits

- Supported image inputs are PNG, JPEG, GIF, WebP, and BMP. TIFF / TIF and ICO are currently unsupported.
- Shared JSON and PDF output uses only the source file name, not its absolute path.
- Exports may still contain EXIF, file modification time, dimensions, and analysis results. Review them before sharing.
- WCAG output is only the contrast ratio between the two leading dominant colors; it is not an AA / AAA pass/fail test.
- Color calculations assume sRGB and do not apply ICC color management.
- Color-role labels are percentage-based conveniences. The PCCS-style labels are unofficial approximations.
- Schema 3 / 4 analysis JSON imports still accept legacy `harmonyScores`. Newly copied or saved shared JSON omits that field; the legacy value cannot evaluate missing ideal hues and is not presented as harmony-pattern fitness.
