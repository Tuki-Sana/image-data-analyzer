# アーキテクチャ概要

Image Data Analyzer の**処理の流れ**と**主要ディレクトリ**だけをまとめたメモです。細部はコードとコメントを参照してください。

## Tauri 未経験者向け：このアプリの層

1. **WebView** … ウィンドウの中で **Vue（HTML/CSS/JS）が動く領域**です。見た目・ボタン・状態のほとんどはここです。
2. **invoke（インボーク）** … フロントから **Rust の関数を名前で呼び出す**仕組みです（いわゆる RPC に近い）。**画像解析・ピクセル取得・ファイルの読み書き（パスが決まったあと）**など、重い処理やネイティブ向け API はここを通ります。
3. **plugin-dialog** … **OS のファイル選択・保存・確認ダイアログ**を出すための別経路です。`invoke` とは入口が分かれていますが、どちらもフロント（Vue）から呼びます。
4. **Rust** … WebView の外側で動く **ネイティブコード**。`invoke` の先で実際の計算やディスクアクセスが行われます。

**サーバーはありません。** ブラウザにアップロードするのではなく、**自分の PC 上**で完結します。

## 全体像（コンポーネントと依存）

矢印は **「呼び出す／データが流れる」** の向きです。`palette_match` などは **`lib.rs` から直接 invoke されるのではなく**、`analyze.rs` の解析処理の途中で読み込まれます。

```mermaid
flowchart TB
  subgraph tauri["Tauri アプリ（1 ウィンドウ）"]
    subgraph web["WebView 内（Vue / TypeScript）"]
      App["App.vue\n画面・状態の中心"]
      LS["LocalStorage\nスポイトパレットのみ\nRust は介さない"]
      Parse["analysisImport /\npickerPaletteImport\nJSON → 画面用の型"]
    end

    Inv["invoke\nフロント → Rust"]
    Dlg["plugin-dialog\nファイル選択・保存・確認"]

    subgraph rust["Rust（ネイティブ）"]
      Cmd["lib.rs\ninvoke の受け口・コマンド登録"]
      An["analyze.rs\n画像・支配色・EXIF・gist 等\n解析の中心"]
      Mod["palette_match / theory /\nharmony / color_theory / meta\n※ いずれも analyze から利用"]
    end
  end

  Disk["ディスク上のファイル\n（画像パス・読み書き）"]

  App --> Inv
  App --> Dlg
  App --> LS
  App --> Parse
  Inv --> Cmd
  Cmd --> An
  An --> Mod
  An --> Disk
  Dlg --> OS["OS 標準ダイアログ"]
```

## 典型フロー：画像を開いて解析する（時系列）

上のブロック図の補足として、**よくある 1 操作**だけを順番に示します。

```mermaid
sequenceDiagram
  participant U as ユーザー
  participant App as App.vue（WebView 内）
  participant Dlg as plugin-dialog
  participant OS as OS
  participant Inv as invoke
  participant Rust as "Rust（lib.rs → analyze.rs）"
  participant FS as ファイル

  U->>App: 画像を開く操作
  App->>Dlg: ファイル選択を依頼
  Dlg->>OS: ダイアログ表示
  OS-->>Dlg: 選択されたパス
  Dlg-->>App: パスを返す
  App->>Inv: analyze_image 等（パスを渡す）
  Inv->>Rust: Rust 側ハンドラ実行
  Rust->>FS: 画像を読み込み
  FS-->>Rust: データ
  Rust->>Rust: 支配色・EXIF・配色計算など
  Rust-->>Inv: 解析結果（JSON 相当）
  Inv-->>App: 結果を JS に返す
  App->>App: 画面に表示
```

## 読み方の補足

| 経路 | 役割 |
|------|------|
| **invoke** | 画像解析（`analyze_image`）、ピクセル取得（`sample_pixel`）、テキスト／バイナリの読み書き（`read_text_file` 等）を Rust に依頼する。 |
| **plugin-dialog** | 画像・JSON・PDF のファイル選択・保存、破壊的操作の確認。見た目は OS ネイティブに近い。 |
| **LocalStorage** | スポイトパレット（`schemaVersion: 1` の複数カラーセット形式）。Rust は介さない。 |
| **analysisImport / pickerPaletteImport** | ファイルから読んだ JSON 文字列を、画面用の型にパース（Vitest で検証）。 |

PDF 書き出しは **html2canvas + jsPDF** で DOM を画像化し、生成バイト列を `save_binary_file`（invoke 経由）で保存する流れです。

## 簡易ディレクトリツリー

ビルド成果物（`dist/`, `target/`）や `node_modules/` は省略しています。

```
image-metadata-tool-tauri/
├── .github/workflows/     # CI（テスト・Windows ビルド）
├── docs/
│   ├── architecture.md      # 本ファイル
│   └── image-analysis.md    # 解析・配色アルゴリズム概要
├── public/                # 静的アセット（Vite）
├── src/
│   ├── App.vue            # メイン UI・パレット・解析パネル
│   ├── main.ts
│   ├── setupAppMenu.ts    # ネイティブメニュー（Tauri 時）
│   ├── components/        # GlossaryModal, PdfExportSurface, PickerPaletteSetBar …
│   ├── constants/         # 用語集テキスト、表示名、凡例など
│   ├── types/             # Analysis 等の共有型
│   └── utils/             # 色フォーマット、パレット保存、JSON インポート、PDF、appLog …
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs         # Tauri エントリ・invoke ハンドラ
│   │   ├── main.rs
│   │   ├── analyze.rs     # 画像読込・サンプリング・支配色・EXIF・プレビュー生成
│   │   ├── palette_match.rs
│   │   ├── theory.rs      # PCCS 風トーン等
│   │   ├── color_theory.rs
│   │   ├── harmony.rs
│   │   └── meta.rs
│   ├── assets/            # Open Color / Tailwind 参照 JSON（Rust が読む）
│   ├── capabilities/
│   ├── icons/
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
├── vite.config.ts
├── vitest.config.ts
├── CHANGELOG.md
└── README.md
```

## 関連ドキュメント

- 利用者向けの機能説明・開発コマンド: リポジトリ直下の [README.md](../README.md)
- 画像解析・支配色・色差・調和スコアなどの**アルゴリズム概要**: [image-analysis.md](./image-analysis.md)
