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
  openGlossary: () => void | Promise<void>;
};

function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/** macOS ではメニュー最上位は Submenu のみ。アプリ／ファイル／ヘルプの並び。 */
export async function installAppMenu(h: AppMenuHandlers): Promise<void> {
  if (!isTauri()) return;

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
          void h.openImage();
        },
      }),
      await PredefinedMenuItem.new({ item: "Separator" }),
      await MenuItem.new({
        id: "file-close",
        text: "閉じる",
        accelerator: "CmdOrCtrl+W",
        action: () => {
          void h.closeImage();
        },
      }),
      await PredefinedMenuItem.new({ item: "Separator" }),
      await MenuItem.new({
        id: "file-copy-json",
        text: "JSON をコピー",
        accelerator: "CmdOrCtrl+Shift+C",
        action: () => {
          void h.copyJson();
        },
      }),
      await MenuItem.new({
        id: "file-save-json",
        text: "JSON を保存…",
        accelerator: "CmdOrCtrl+Shift+S",
        action: () => {
          void h.saveJson();
        },
      }),
      await MenuItem.new({
        id: "file-save-pdf",
        text: "PDF を保存…",
        action: () => {
          void h.savePdf();
        },
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
          void h.openGlossary();
        },
      }),
    ],
  });

  const menu = await Menu.new({
    items: [appMenu, fileMenu, helpMenu],
  });

  await menu.setAsAppMenu();
}
