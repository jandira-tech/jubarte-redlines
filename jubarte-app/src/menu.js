// The menu bar's choices (src-tauri/src/menu.rs sends each item's id as a
// `menu` event). Each does what the window's own control does, by pressing
// that control, so a menu choice obeys the same rules: a disabled button stays
// disabled, a busy run is not interrupted, and the free-use gate still asks.

const $ = (id) => document.getElementById(id);
const press = (id) => $(id)?.click();
const tab = (mode) => press(mode === "convert" ? "mode-convert" : "mode-redline");

/** What each menu item does, by id. */
export const ACTIONS = {
  about: () => window.jubarteAbout?.open(),
  settings: () => press("open-settings"),
  pro: () => {
    if (window.jubarte?.entitled) window.jubarteToast?.("Jubarte PRO is active on this Mac.");
    else window.jubarte?.openPaywall?.();
  },
  restore: () => window.jubarte?.restorePurchase?.(),
  "choose-original": () => (tab("redline"), press("zone-original")),
  "choose-modified": () => (tab("redline"), press("zone-modified")),
  "choose-convert": () => (tab("convert"), press("zone-convert")),
  "make-redline": () => (tab("redline"), press("run")),
  "make-pdf": () => (tab("convert"), press("run")),
  "export-pdf": () => press("export-pdf"),
  swap: () => (tab("redline"), press("swap")),
  "open-result": () => press($("open-pdf").hidden ? "open-word" : "open-pdf"),
  reveal: () => press("reveal"),
  "save-copy": () => press("save-copy"),
  "show-redline": () => tab("redline"),
  "show-convert": () => tab("convert"),
  "toggle-panel": () => window.jubartePanel?.toggle(),
  "appearance-system": () => window.jubarteSettings?.setAppearance("system"),
  "appearance-light": () => window.jubarteSettings?.setAppearance("light"),
  "appearance-dark": () => window.jubarteSettings?.setAppearance("dark"),
  terms: () => window.jubarteAbout?.legal("terms"),
  privacy: () => window.jubarteAbout?.legal("privacy"),
};

// Choices that open a window over the others, or change only how the app
// looks, leave open windows be; anything else is work in the main window, so
// an open sheet steps aside for it.
const KEEP_OPEN = new Set([
  "about",
  "settings",
  "pro",
  "restore",
  "terms",
  "privacy",
  "appearance-system",
  "appearance-light",
  "appearance-dark",
]);

if (typeof window !== "undefined" && window.__TAURI__?.event) {
  window.__TAURI__.event.listen("menu", (e) => {
    const id = String(e.payload);
    const act = ACTIONS[id];
    if (!act) return;
    if (!KEEP_OPEN.has(id)) for (const d of document.querySelectorAll("dialog[open]")) d.close();
    act();
  });
}
