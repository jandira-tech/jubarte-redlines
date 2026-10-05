// The light/dark choice. With nothing stored the page follows the browser
// (site.css: color-scheme: light dark); the footer switch (theme.js) stores
// "light" or "dark" under THEME_KEY. This script restores it before the first
// paint, so a pinned mode never flashes the other one. It is the only inline
// script: the CSP allows it by its hash, and test/node/pages.test.ts proves
// the hash matches these exact bytes.

export const THEME_KEY: string = "jb-theme";

export const THEME_SCRIPT: string = `try{var t=localStorage.getItem("${THEME_KEY}");if(t==="light"||t==="dark")document.documentElement.dataset.theme=t}catch(e){}`;

export const THEME_SCRIPT_SHA256: string = "NOaFKWONTj8/FBLUAhGxKgyHDFaIpLbbD34RwBxrRUg=";

/** The browser chrome's color for each mode: the page background, Graphite and Night. */
export const THEME_COLOR = { light: "#F5F5F3", dark: "#0E1720" } as const;
