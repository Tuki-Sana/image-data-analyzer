# Changelog

このプロジェクトの重要な変更はこのファイルに記載します。形式は [Keep a Changelog](https://keepachangelog.com/ja/1.1.0/) に近づけています。

## [Unreleased]

### Changed

- **フロント構成**: `App.vue` をヘッダー・ツールバー・分析パネル・プレビュー・空ワークスペースなどのコンポーネントに分割。画面ロジックを `src/composables/` に集約し、分析パネル周りのスタイルを `src/styles/analysisWorkspace.css` で共有（`main.ts` から読み込み）
- **ヘッダー**: タイトルとマークの縦揃え（flex 中央）およびオプティカル調整（`translateY`）
- **PDF 書き出し**: ホスト要素の ref を Vue 3.5 の `useTemplateRef` で `useImageAnalysisSession` に渡す形に整理

### Documentation

- [README](README.md): フロントの置き場（`composables` / `styles` 等）を追記。[architecture.md](docs/architecture.md) のディレクトリツリーと Mermaid を現状に同期

## [0.2.0] - 2026-04-06

### Added

- スポイトパレットの**複数カラーセット**（切替・セット名・新規／複製／セット削除）
- **パレット JSON**・**分析 JSON**のファイル読み込み（メニューおよび画面）
- Rust `read_text_file` コマンド（フロントから JSON テキスト取得）
- 破壊的操作の確認に **Tauri `plugin-dialog` の `confirm`**（`window.confirm` はブラウザ時のみ）
- メニューから起動した非同期ハンドラの**未捕捉 reject**をログ・トースト
- カラーセット削除後、`パレット N` 形式の名前の**連番振り直し**
- Vitest による `analysisImport` / `pickerPaletteImport` / `pickerPaletteStorage` のユニットテスト
- [`docs/image-analysis.md`](docs/image-analysis.md): 画像解析・配色ロジックの実装メモ（[`docs/architecture.md`](docs/architecture.md)・[README](README.md) からリンク）

### Changed

- **分析 JSON `schemaVersion`**: **4** に更新（[`analyze.rs`](src-tauri/src/analyze.rs) の **`ANALYSIS_SCHEMA_VERSION`**）。キー構造は 3 と同じ。CIEDE2000・支配色・調和など**算出意味**が変わった版の目印（旧 JSON のインポートは可能だが再解析で数値が変わることがある）
- スポイトパレットの LocalStorage を **v1 スキーマ**（`palettes[]` 等）に変更。旧エントリ配列は自動移行
- **支配色推定**（`meta::dominant_colors`）: 目標サンプル数に基づく間引き + **Lab 空間の k-means**。代表色は各クラスタの **Lab 重心を sRGB に逆変換**（同一 RGB は割合をマージ）。**初期重心は farthest-first（決定的 k-means++ 貪欲）**。従来の RGB 量子化 `q=28` から変更
- **Open Color / Tailwind 近似**: 最近傍の距離を **CIEDE2000（ΔE00）** に変更（`delta_e_2000`）。CIE76（`delta_e_76`）は比較用に残す
- **色相調和・類似色**（`harmony::score_analogous`）: **重み付き円周合成**（支配色の `pct` を反映）。補色対のノイズ支配色に引っ張られにくい
- **支配色 k-means**: 最大反復 **`KMEANS_MAX_ITER` を 32** に拡大
- **支配色間引き**: **`DOMINANT_TARGET_SAMPLES` を約 15 万点**に拡大（大画像のサンプル密度向上）
- **調和テンプレート**: アンカー探索を **15°×24 → 10°×36** に変更（向きの取り違えをやや抑制）
- **ドキュメント**: [README](README.md) にドキュメント案内と現行の支配色・調和の要点を反映。[architecture.md](docs/architecture.md) に Rust モジュール対応表と CHANGELOG へのリンク。[image-analysis.md](docs/image-analysis.md) の関連ドキュメントを拡充。用語集の**色相調和スコア**を実装（類似色の重み付き合成・10° アンカー）に同期
- **PDF レポート**: **Open Color / Tailwind 近似**を **ΔE2000** 付きの表で出力。フッター注記を更新。**色相調和スコア**の凡例に計算概要の 1 行を追加（`harmonyScoreLegend.ts`、UI・PDF 共通）
- **PDF レポート**: **色彩理論メモ（TheoryBlock）**を追加（注意書き・加重平均色相・概論対応・支配色ごとの L\*C\*h° とトーン）。**セクション順**をメイン画面に合わせ（主要色 → ひと目サマリ → Open/Tailwind → WCAG → 色彩理論 → 調和 → EXIF）。WCAG 見出しを画面と同じ「コントラスト」表記に統一
- **色相調和**: テンプレート型ごとに **ガウス σ（度）**を分離（補色 20°・トライアド 22°・分割補色 24°・テトラード 24.5°）。[`docs/image-analysis.md`](docs/image-analysis.md) §8・§12（PDF）を更新。凡例・用語集の説明を追随
- **UI / a11y**: ヘッダーの長いサブタイトルを削除。`role="banner"` / `nav` / `main`、エラー `role="alert"`、トースト `aria-live`、プレビュー画像の **キーボード（Enter/Space で中央サンプル）**・動的 `alt`・`aria-describedby`、補助色トグルの **`aria-pressed`**、調和リストの **`aria-label`**、装飾スウォッチの `aria-hidden`、`focus-visible` の強化（`PickerPaletteSetBar` のフォーカスリングを `focus-visible` に）

## [0.1.0] - 初期リリース

- Tauri + Vue による画像解析・配色 UI、パレット・用語集・PDF 書き出しなど
