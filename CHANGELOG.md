# Changelog

このプロジェクトの重要な変更はこのファイルに記載します。形式は [Keep a Changelog](https://keepachangelog.com/ja/1.1.0/) に近づけています。

## [0.2.0] - 2026-04-06

### Added

- スポイトパレットの**複数カラーセット**（切替・セット名・新規／複製／セット削除）
- **パレット JSON**・**分析 JSON**のファイル読み込み（メニューおよび画面）
- Rust `read_text_file` コマンド（フロントから JSON テキスト取得）
- 破壊的操作の確認に **Tauri `plugin-dialog` の `confirm`**（`window.confirm` はブラウザ時のみ）
- メニューから起動した非同期ハンドラの**未捕捉 reject**をログ・トースト
- カラーセット削除後、`パレット N` 形式の名前の**連番振り直し**
- Vitest による `analysisImport` / `pickerPaletteImport` / `pickerPaletteStorage` のユニットテスト

### Changed

- スポイトパレットの LocalStorage を **v1 スキーマ**（`palettes[]` 等）に変更。旧エントリ配列は自動移行

## [0.1.0] - 初期リリース

- Tauri + Vue による画像解析・配色 UI、パレット・用語集・PDF 書き出しなど
