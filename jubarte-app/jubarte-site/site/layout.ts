// The frame every page shares: head, nav, footer, the whale. Pages are plain
// HTML strings; anything that did not come from this repository goes through
// esc().

import { THEME_COLOR, THEME_SCRIPT } from "../src/theme.ts";

export type NavKey =
  | "benchmark"
  | "contact"
  | "download"
  | "home"
  | "live"
  | "demo"
  | "usecases"
  | "pro";

export type Page = {
  /** File written under public/, e.g. "demo.html" (served at /demo). */
  file: string;
  /** Canonical path, e.g. "/demo". */
  path: string;
  title: string;
  description: string;
  nav: NavKey | null;
  body: string;
  /** Module scripts under /static/js, e.g. "demo.js". */
  scripts?: string[];
  /** Footer links; the default set when omitted. */
  footer?: [string, string][];
  /** Footer credit line. */
  credit?: string;
  /** Extra <head> lines (preloads). */
  head?: string;
  /** Keep out of the sitemap and search results. */
  noindex?: boolean;
  /** No nav or footer: a page meant to be framed (the case viewer embed). */
  bare?: boolean;
};

export const ORIGIN: string = "https://jubarte.pro";

const ENTITIES: Record<string, string> = {
  "&": "&amp;",
  "<": "&lt;",
  ">": "&gt;",
  '"': "&quot;",
  "'": "&#39;",
};

export function esc(value: string | number): string {
  return String(value).replace(/[&<>"']/g, (c) => ENTITIES[c]);
}

let versions: Record<string, string> = {};
let vendor = { jubarte: "/vendor/jubarte", pdfjs: "/vendor/pdfjs" };

/** Called by the build once static files are hashed. */
export function setAssetVersions(map: Record<string, string>): void {
  versions = map;
}

/** Called by the build with the versioned folders the wasm and pdf.js live in. */
export function setVendorBases(bases: { jubarte: string; pdfjs: string }): void {
  vendor = bases;
}

/** A static URL with its content hash in the name, so it can be cached for a year. */
export function asset(path: string): string {
  const v = versions[path];
  return v ? path.replace(/(\.\w+)$/, `.${v}$1`) : path;
}

export const WHALE_BODY: string =
  "M 76 192 C 82 172 112 156 150 146 C 196 132 258 112 310 110 C 356 108 402 118 448 148 C 458 152 464 150 470 144 C 478 128 494 106 520 92 C 512 112 508 126 502 134 C 514 140 526 148 540 160 C 520 164 498 160 480 162 C 470 164 460 166 452 166 C 420 190 382 228 328 252 C 270 278 194 286 148 268 C 114 254 88 230 81 208 C 76 198 74 194 76 192 Z";
export const WHALE_FIN: string =
  "M 208 228 C 246 240 286 268 322 298 C 334 308 338 320 330 326 C 296 326 254 306 226 280 C 206 260 200 240 208 228 Z";

// The nav's mark takes the palette (site.css .brand-mark): the primary whale,
// a lighter fin, an eye of the page's own background.
export const BRAND_MARK: string = `<svg class="brand-mark" viewBox="0 0 560 360" width="34" height="22" aria-hidden="true"><g transform="rotate(-5 280 200)"><path class="mk-body" d="${WHALE_BODY}"/><path class="mk-fin" d="${WHALE_FIN}"/><circle class="mk-eye" cx="122" cy="196" r="6"/></g></svg>`;

/** The faint drifting whale behind a page. */
export function watermark(extra = ""): string {
  return `<svg class="watermark ${extra}" viewBox="0 0 560 360" aria-hidden="true"><path d="${WHALE_BODY}" fill="currentColor"/><path d="${WHALE_FIN}" fill="currentColor"/></svg>`;
}

/** The illustrated whale (hero, app window, Live). `id` keeps gradient ids unique. */
export function whale(
  id: string,
  opts: { light?: boolean; extraLines?: boolean; cls?: string; style?: string } = {},
): string {
  const top = opts.light ? "#6691B1" : "#3D7EAE";
  const mid = opts.light ? "#25628F" : "#1E5580";
  const low = opts.light ? "#194361" : "#123B5C";
  const more = opts.extraLines
    ? `<path d="M 88 250 C 142 282 210 298 272 288" stroke="#6FA3BD" stroke-width="3" stroke-linecap="round" opacity="0.75"/><path d="M 92 262 C 138 288 190 302 240 296" stroke="#6FA3BD" stroke-width="2.5" stroke-linecap="round" opacity="0.6"/>`
    : "";
  const tail = opts.extraLines
    ? `<path d="M 80 198 C 100 210 124 217 148 218" stroke="#0F3350" stroke-width="2.5" stroke-linecap="round" opacity="0.45" fill="none"/>`
    : "";
  const wake = opts.extraLines
    ? `<path d="M 208 228 C 246 240 286 268 322 298" stroke="#F2F9FC" stroke-width="2" stroke-linecap="round" opacity="0.6"/><path d="M 110 316 C 200 340 320 340 420 306" stroke="#7FB6CE" stroke-width="4" stroke-linecap="round" opacity="0.35"/><path d="M 160 336 C 240 354 330 352 400 330" stroke="#7FB6CE" stroke-width="3" stroke-linecap="round" opacity="0.22"/>`
    : "";
  return `<svg viewBox="0 0 560 360" fill="none" aria-hidden="true"${opts.cls ? ` class="${opts.cls}"` : ""} style="${opts.style ?? ""}">
<defs>
<linearGradient id="${id}-body" x1="280" y1="90" x2="280" y2="300" gradientUnits="userSpaceOnUse"><stop offset="0" stop-color="${top}"/><stop offset="0.55" stop-color="${mid}"/><stop offset="1" stop-color="${low}"/></linearGradient>
<linearGradient id="${id}-fin" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#CBE2EE"/><stop offset="1" stop-color="#8FB8CE"/></linearGradient>
<clipPath id="${id}-clip"><path d="${WHALE_BODY}"/></clipPath>
</defs>
<g transform="rotate(-5 280 200)">
<path class="whale-body" d="${WHALE_BODY}" fill="url(#${id}-body)"/>
<path d="M 330 112 C 338 98 348 92 358 90 C 352 102 350 110 348 116 Z" fill="${mid}"/>
<g clip-path="url(#${id}-clip)">
<path d="M 56 206 C 170 230 330 240 470 162 L 560 360 L 20 360 Z" fill="#E9F3F7"/>
<path d="M 96 214 C 165 252 250 268 330 252" stroke="#6FA3BD" stroke-width="4.5" stroke-linecap="round" opacity="0.92"/>
<path d="M 91 226 C 156 262 240 280 316 264" stroke="#6FA3BD" stroke-width="4" stroke-linecap="round" opacity="0.85"/>
<path d="M 88 238 C 148 272 226 290 296 276" stroke="#6FA3BD" stroke-width="3.5" stroke-linecap="round" opacity="0.75"/>${more}
</g>${tail}
<circle cx="122" cy="196" r="5" fill="#0B1E2D"/><circle cx="124" cy="194" r="1.6" fill="#DFF0F7"/>
<path d="${WHALE_FIN}" fill="url(#${id}-fin)"/>${wake}
</g></svg>`;
}

export const DOC_ICON: string = `<svg class="slot-icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M6 2h9l5 5v15a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1V3a1 1 0 0 1 1-1z" fill="currentColor" opacity=".16"/><path d="M15 2v5h5" fill="none" stroke="currentColor" stroke-width="1.4"/><path d="M8.5 13h7M8.5 16.5h7M8.5 9.5H12" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/></svg>`;
export const DOC_ICON_MOD: string = `<svg class="slot-icon" viewBox="0 0 24 24" aria-hidden="true"><path d="M6 2h9l5 5v15a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1V3a1 1 0 0 1 1-1z" fill="currentColor" opacity=".16"/><path d="M15 2v5h5" fill="none" stroke="currentColor" stroke-width="1.4"/><path d="M8.5 13h7M8.5 16.5h4.5M8.5 9.5H12" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/><path d="m14.5 17.5 5-5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" opacity=".6"/></svg>`;
export const SWAP_ICON: string = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M8 3 4 7l4 4"/><path d="M4 7h16"/><path d="m16 21 4-4-4-4"/><path d="M20 17H4"/></svg>`;
export const EMPTY_ICON: string = `<svg viewBox="0 0 24 24" style="width:44px;height:44px;color:var(--primary)" aria-hidden="true"><rect x="4" y="3" width="16" height="18" fill="currentColor" opacity=".1"/><path d="M8 8h8M8 12h8M8 16h5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/></svg>`;
export const CHECK_ICON: string = `<svg class="check-ico" viewBox="0 0 24 24" aria-hidden="true"><rect x="2" y="2" width="20" height="20" fill="currentColor" opacity=".12"/><path d="m7 12.5 3 3 6-6.5" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>`;

// Home opens the row, then the pages alphabetically, so a page is where a
// reader expects it; PRO, the paid app, closes the row in inverted colours.
// "Try it" goes where the home page's main button goes: the Demo.
const NAV: [NavKey, string, string][] = [
  ["home", "Home", "/"],
  ["benchmark", "Benchmark", "/benchmark"],
  ["contact", "Contact", "/contact"],
  ["download", "Download", "/download"],
  ["live", "Live", "/live"],
  ["demo", "Try it", "/demo"],
  ["usecases", "Use cases", "/use-cases"],
  ["pro", "PRO", "/pro"],
];

function nav(active: NavKey | null): string {
  const cur = (k: NavKey) => (k === active ? ' aria-current="page"' : "");
  // The hidden words make the link read "Live, coming soon", not "Livesoon".
  const soon = (k: NavKey) =>
    k === "live"
      ? '<span class="sr-only">, coming </span><span class="badge-soon">soon</span>'
      : "";
  const pro = (k: NavKey) => (k === "pro" ? ' class="nav-pro"' : "");
  const links = NAV.map(
    ([k, label, href]) => `<a href="${href}"${cur(k)}${pro(k)}>${label}${soon(k)}</a>`,
  ).join("");
  const menu = NAV.map(
    ([k, label, href]) => `<a href="${href}"${cur(k)}${pro(k)}><span>${label}${soon(k)}</span></a>`,
  ).join("");
  return `<header class="site-nav"><div class="wrap">
<a class="brand" href="/" aria-label="Jubarte home">${BRAND_MARK}<span class="brand-word">JUBARTE</span></a>
<nav class="nav-links" aria-label="Main">${links}</nav>
<details class="nav-menu"><summary>Menu</summary><nav class="nav-menu-list" aria-label="Main">${menu}</nav></details>
</div></header>`;
}

export const DEFAULT_FOOTER: [string, string][] = [
  ["Contact", "/contact"],
  ["Download", "/download"],
  ["GitHub", "https://github.com/jandira-tech/jubarte-redlines"],
  ["Privacy", "/privacy"],
  ["Terms", "/terms"],
];

function footer(page: Page): string {
  const links = (page.footer ?? DEFAULT_FOOTER)
    .map(([label, href]) => `<a href="${href}">${label}</a>`)
    .join("");
  const credit = page.credit ?? "© MMXXVI Jandira Technologies · New York · São Paulo";
  // theme.js wires the switch; site.css shows the label for the mode it turns on.
  const toggle = `<button type="button" class="theme-toggle"><span class="theme-dot" aria-hidden="true"></span><span class="to-dark">Dark mode</span><span class="to-light">Light mode</span></button>`;
  return `<footer class="site-footer"><div class="wrap"><span>${credit}</span><nav aria-label="Footer">${links}</nav>${toggle}</div></footer>`;
}

export function render(page: Page): string {
  const url = `${ORIGIN}${page.path}`;
  // Scripts keep plain URLs (revalidated, see _headers): modules import each
  // other by path, and a versioned entry would load a second copy of them.
  // The first <main> is the skip link's target; a page that names it keeps its id.
  const main = /<main\b[^>]*>/.exec(page.body);
  const mainId = main ? (/\bid="([^"]+)"/.exec(main[0])?.[1] ?? "main") : null;
  const body =
    main && !/\bid="/.test(main[0])
      ? page.body.replace("<main", '<main id="main" tabindex="-1"')
      : page.body;
  const skip = mainId ? `<a class="skip-link" href="#${mainId}">Skip to content</a>` : "";
  const scripts = ["common.js", "theme.js", ...(page.scripts ?? [])]
    .map((s) => `<script type="module" src="/static/js/${s}"></script>`)
    .join("\n");
  return `<!doctype html>
<html lang="en" data-engine="${vendor.jubarte}" data-pdfjs="${vendor.pdfjs}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<script>${THEME_SCRIPT}</script>
<title>${esc(page.title)}</title>
<meta name="description" content="${esc(page.description)}">
<link rel="canonical" href="${url}">
${page.noindex ? '<meta name="robots" content="noindex">\n' : ""}<meta name="theme-color" content="${THEME_COLOR.light}" media="(prefers-color-scheme: light)">
<meta name="theme-color" content="${THEME_COLOR.dark}" media="(prefers-color-scheme: dark)">
<meta property="og:type" content="website">
<meta property="og:site_name" content="Jubarte">
<meta property="og:title" content="${esc(page.title)}">
<meta property="og:description" content="${esc(page.description)}">
<meta property="og:url" content="${url}">
<meta property="og:image" content="${ORIGIN}/og.png">
<meta name="twitter:card" content="summary_large_image">
<link rel="icon" href="/favicon.svg" type="image/svg+xml">
<link rel="icon" href="/favicon-32.png" sizes="32x32" type="image/png">
<link rel="apple-touch-icon" href="/apple-touch-icon.png">
<link rel="preload" href="/static/fonts/manrope-latin-wght-normal.woff2" as="font" type="font/woff2" crossorigin>
<link rel="preload" href="/static/fonts/jetbrains-mono-latin-wght-normal.woff2" as="font" type="font/woff2" crossorigin>
<link rel="stylesheet" href="${asset("/static/css/site.css")}">
${page.head ?? ""}${scripts}
</head>
<body${page.bare ? ' class="bare"' : ""}>
${page.bare ? "" : skip}
${page.bare ? "" : nav(page.nav)}
${body}
${page.bare ? "" : footer(page)}
</body>
</html>
`;
}
