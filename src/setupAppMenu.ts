import { Menu, MenuItem, PredefinedMenuItem, Submenu } from "@tauri-apps/api/menu";
import packageJson from "../package.json";
import { APP_DISPLAY_NAME } from "./constants/appMeta";

const APP_NAME = APP_DISPLAY_NAME;

export type AppMenuHandlers = {
  openImage: () => void | Promise<void>;
  closeImage: () => void | Promise<void>;
  copyJson: () => void | Promise<void>;
  saveJson: () => void | Promise<void>;
  savePdf: () => void | Promise<void>;
  importPickerPaletteReplace: () => void | Promise<void>;
  importPickerPaletteMerge: () => void | Promise<void>;
  importAnalysisJson: () => void | Promise<void>;
  openGlossary: () => void | Promise<void>;
};

export type InstallAppMenuOptions = {
  /**
   * メニューから起動したハンドラが未捕捉の reject を返したとき（内部 try/catch で拾えなかった例外など）
   */
  onAsyncHandlerError?: (menuLabel: string, error: unknown) => void;
};

function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function runMenuHandler(
  menuLabel: string,
  fn: () => void | Promise<void>,
  onAsyncHandlerError?: InstallAppMenuOptions["onAsyncHandlerError"],
): void {
  void Promise.resolve(fn()).catch((err) => {
    console.error(`[AppMenu] ${menuLabel}`, err);
    onAsyncHandlerError?.(menuLabel, err);
  });
}

/** macOS ではメニュー最上位は Submenu のみ。アプリ／ファイル／ヘルプの並び。 */
export async function installAppMenu(
  h: AppMenuHandlers,
  options?: InstallAppMenuOptions,
): Promise<void> {
  if (!isTauri()) return;

  const onErr = options?.onAsyncHandlerError;

  const version = packageJson.version ?? "0.1.0";

  const appMenu = await Submenu.new({
    text: APP_NAME,
    items: [
      await PredefinedMenuItem.new({
        item: {
          About: {
            name: APP_NAME,
            version,
            shortVersion: version,
            copyright: `© 2026 ${APP_DISPLAY_NAME}`,
          },
        },
      }),
      await PredefinedMenuItem.new({ item: "Separator" }),
      await PredefinedMenuItem.new({ item: "Services" }),
      await PredefinedMenuItem.new({ item: "Separator" }),
      await PredefinedMenuItem.new({ item: "Hide" }),
      await PredefinedMenuItem.new({ item: "HideOthers" }),
      await PredefinedMenuItem.new({ item: "ShowAll" }),
      await PredefinedMenuItem.new({ item: "Separator" }),
      await PredefinedMenuItem.new({ item: "Quit" }),
    ],
  });

  const fileMenu = await Submenu.new({
    text: "ファイル",
    items: [
      await MenuItem.new({
        id: "file-open",
        text: "開く…",
        accelerator: "CmdOrCtrl+O",
        action: () => {
          runMenuHandler("開く…", () => h.openImage(), onErr);
        },
      }),
      await PredefinedMenuItem.new({ item: "Separator" }),
      await MenuItem.new({
        id: "file-close",
        text: "閉じる",
        accelerator: "CmdOrCtrl+W",
        action: () => {
          runMenuHandler("閉じる", () => h.closeImage(), onErr);
        },
      }),
      await PredefinedMenuItem.new({ item: "Separator" }),
      await MenuItem.new({
        id: "file-copy-json",
        text: "JSON をコピー",
        accelerator: "CmdOrCtrl+Shift+C",
        action: () => {
          runMenuHandler("JSON をコピー", () => h.copyJson(), onErr);
        },
      }),
      await MenuItem.new({
        id: "file-save-json",
        text: "JSON を保存…",
        accelerator: "CmdOrCtrl+Shift+S",
        action: () => {
          runMenuHandler("JSON を保存…", () => h.saveJson(), onErr);
        },
      }),
      await MenuItem.new({
        id: "file-save-pdf",
        text: "PDF を保存…",
        action: () => {
          runMenuHandler("PDF を保存…", () => h.savePdf(), onErr);
        },
      }),
      await PredefinedMenuItem.new({ item: "Separator" }),
      await Submenu.new({
        text: "読み込み",
        items: [
          await MenuItem.new({
            id: "import-palette-replace",
            text: "パレット JSON（置換）…",
            action: () => {
              runMenuHandler(
                "パレット JSON（置換）…",
                () => h.importPickerPaletteReplace(),
                onErr,
              );
            },
          }),
          await MenuItem.new({
            id: "import-palette-merge",
            text: "パレット JSON（現在と結合）…",
            action: () => {
              runMenuHandler(
                "パレット JSON（現在と結合）…",
                () => h.importPickerPaletteMerge(),
                onErr,
              );
            },
          }),
          await MenuItem.new({
            id: "import-analysis-json",
            text: "分析 JSON…",
            action: () => {
              runMenuHandler("分析 JSON…", () => h.importAnalysisJson(), onErr);
            },
          }),
        ],
      }),
    ],
  });

  const helpMenu = await Submenu.new({
    text: "ヘルプ",
    items: [
      await MenuItem.new({
        id: "help-glossary",
        text: "用語集…",
        action: () => {
          runMenuHandler("用語集…", () => h.openGlossary(), onErr);
        },
      }),
    ],
  });

  const menu = await Menu.new({
    items: [appMenu, fileMenu, helpMenu],
  });

  await menu.setAsAppMenu();
}
