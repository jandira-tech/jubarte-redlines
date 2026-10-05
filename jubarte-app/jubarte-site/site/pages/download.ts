import { command } from "../command.ts";
import { fact } from "../data/facts.ts";
import {
  APP_NEXT,
  APP_RELEASE,
  APP_STORE,
  ARCHIVES,
  type Archive,
  byPhase,
  CHANGELOG_URL,
  ENGINE_RELEASED,
  ENGINE_VERSION,
  RELEASE_DL,
  RELEASE_URL,
  RELEASES,
  SUPPORT_EMAIL,
  size,
  WHEELS,
} from "../data/release.ts";
import { esc, type Page } from "../layout.ts";

const CHANNELS = [
  {
    tag: "CLI",
    req: "any OS with cargo",
    title: "cargo install",
    cmd: "cargo install jubarte-redlines\njubarte --version",
    note: 'Installs the <span class="code-inline">jubarte</span> binary: compare, changes, accept, reject, convert, inspect, text, edit, capabilities, debug, self-update.',
    link: "crates.io/crates/jubarte-redlines",
    href: "https://crates.io/crates/jubarte-redlines",
  },
  {
    tag: "Rust crate",
    req: "MSRV 1.88 · edition 2024",
    title: "Library",
    cmd: "cargo add jubarte-redlines \\\n  --no-default-features",
    note: 'Import path is <span class="code-inline">jubarte::</span>. Default features add the CLI, mimalloc and self-update; library consumers opt in deliberately. <span class="code-inline">unsafe_code = "deny"</span>.',
    link: "docs.rs/jubarte-redlines",
    href: "https://docs.rs/jubarte-redlines",
  },
  {
    tag: "Python",
    req: "CPython ≥ 3.10",
    title: "pip install",
    cmd: "pip install jubarte-redlines",
    note: "PyO3 / maturin abi3 wheels for manylinux x86_64 + aarch64 and macOS x86_64 + arm64. Low-level byte API plus the immutable Document API.",
    link: "pypi.org/project/jubarte-redlines",
    href: "https://pypi.org/project/jubarte-redlines/",
  },
  {
    tag: "Node · browser",
    req: "Node ≥ 18 · WebAssembly",
    title: "npm install",
    cmd: "npm install jubarte-wasm",
    note: 'Exports for Node and the browser, full and slim (no PDF renderer). <span class="code-inline">jubarte-wasm/web-slim</span> compares on the Demo page; <span class="code-inline">jubarte-wasm/web</span> renders its PDFs.',
    link: "npmjs.com/package/jubarte-wasm",
    href: "https://www.npmjs.com/package/jubarte-wasm",
  },
];

const fileRow = (target: string, file: string, bytes: number, fonts: string) =>
  `<div class="t-row dl-cols" role="row">
<span class="strong" role="rowheader">${esc(target)}</span>
<span role="cell"><a class="mono" href="${RELEASE_DL}/${file}">${esc(file)}</a></span>
<span class="mono r" role="cell">${size(bytes)}</span>
<span class="mono muted" role="cell">${fonts}</span>
<span role="cell"><span class="tag state">published</span></span>
</div>`;

// release.yml's CLI targets. A target whose build failed has no archive in
// the release (v0.10.1 has no Windows one); it shows as not built yet.
const CLI_TARGETS: string[] = fact("download.cli_targets");

/** A "not built yet" row for each CLI target `archives` has no file for. */
export const pendingRows = (archives: Archive[]) =>
  CLI_TARGETS.filter((t) => !archives.some((a) => a.target === t))
    .map(
      (t) =>
        `<div class="t-row dl-cols pending" role="row"><span class="strong" role="rowheader">${esc(t)}</span><span class="mono muted" role="cell">not built yet</span><span class="mono r muted" role="cell">—</span><span class="mono muted" role="cell">—</span><span role="cell"><span class="tag">not yet</span></span></div>`,
    )
    .join("\n");

const mailLink = `mailto:?subject=${encodeURIComponent("Jubarte for Mac")}&body=${encodeURIComponent(
  `Jubarte on the Mac App Store: ${APP_STORE.url}\nEvery download: https://jubarte.pro/download\nCLI: cargo install jubarte-redlines`,
)}`;

const body = `<main class="wrap">
<div class="head-download m-stack">
<div>
<p class="eyebrow">/ DOWNLOAD · JUBARTE FOR MAC ${APP_STORE.version}${byPhase(` · ${APP_RELEASE.version} IN APPLE’S REVIEW`, "")}</p>
<h1 class="h1">Redlines on your Mac, as Word’s own.</h1>
<p class="lead">Drop the draft and the counterparty’s turn of it; get a redline with real tracked changes that Word accepts, rejects and filters by author. Your documents never leave your Mac, and Microsoft Word is not needed to make the redline.</p>
<p class="small mt-18">On Windows, or on a firm laptop where you cannot install anything? <a href="/demo">The browser demo</a> runs the same engine in the tab: nothing to install, nothing uploaded.</p>
</div>
<img class="app-icon float" src="/apple-touch-icon.png" alt="Jubarte app icon" width="168" height="168">
</div>

<div class="phone-note m-show">
<span class="kicker blue">On a phone</span>
<p>The Mac app and the CLI install on a computer. Send yourself the link and pick it up there.</p>
<div class="row mt-12 m-col" id="send-link">
<input class="input" type="email" id="send-email" placeholder="you@firm.com" autocomplete="email" aria-label="Your email">
<a class="btn btn-primary" id="send-btn" href="${mailLink}">Email me the download link →</a>
</div>
</div>

<section class="mt-56">
<div class="section-head"><h2>Mac app</h2><span>${APP_STORE.minOs} or later · sandboxed · Mac App Store</span></div>
<div class="cells no-top cols-auto-300">
<div class="cell-pad featured">
<p class="kicker blue">Mac App Store</p>
<p class="surface-title">Jubarte ${APP_STORE.version}</p>
<p class="small mt-12">Drop two Word documents, get a redline next to the original. Documents never leave the Mac. ${byPhase(`${APP_STORE.price} on the store today; Jubarte ${APP_RELEASE.version}, in Apple’s review now, is free for ${APP_NEXT.freeUses} uses`, `${APP_STORE.price} on the Mac App Store; ${APP_NEXT.freeUses} free uses`)}, a redline or a PDF each, then the Pro Version at ${APP_NEXT.yearly}. Want to help? The Pro Version is how.</p>
<div class="row mt-24 m-col">
<a class="btn btn-primary" href="${APP_STORE.url}">Get it on the Mac App Store ↗</a>
<a class="btn btn-text" href="/pro">See it working →</a>
</div>
</div>
</div>
</section>

<section class="mt-56" aria-labelledby="mac-tech">
<div class="section-head"><h2 id="mac-tech">Technical details</h2><span>for you, or for your IT team</span></div>
<dl class="spec">
<div><dt>Requires</dt><dd>${APP_STORE.minOs} or later</dd></div>
<div><dt>Installs from</dt><dd>The Mac App Store, sandboxed by Apple: it reads and writes only the files you choose</dd></div>
<div><dt>Opens</dt><dd>Word documents (.docx), dropped on the window or sent with Open With → Jubarte</dd></div>
<div><dt>Writes</dt><dd>A redline .docx with native tracked changes${byPhase(`; from ${APP_RELEASE.version}, PDFs too`, ", or a PDF")}</dd></div>
<div><dt>Your documents</dt><dd>Compared on this Mac and never sent anywhere. Results stay in the app’s own folder until you save a copy where you want it</dd></div>
<div><dt>Network</dt><dd>Only to confirm a purchase with Apple and our verification service; never for documents</dd></div>
<div><dt>Account</dt><dd>None. Your Apple ID handles the purchase</dd></div>
<div><dt>Version</dt><dd>Jubarte ${APP_STORE.version} on the store today${byPhase(`, ${APP_RELEASE.version} in Apple’s review`, "")}; built on the engine below</dd></div>
</dl>
</section>

<section class="mt-72" aria-labelledby="for-developers">
<div class="section-head"><h2 id="for-developers">For developers and IT</h2><span>the engine inside the app</span></div>
<p class="small mt-18 dev-lead">The Mac app runs the open jubarte-redlines engine. The same Rust core ships as a CLI, a crate, a Python wheel and a wasm package, for pipelines and for machines that are not Macs. Every engine release is tagged on GitHub with a SHA256SUMS file.</p>
</section>

<section class="mt-36">
<div class="section-head"><h2>Engine: jubarte-redlines ${ENGINE_VERSION}</h2><span>AGPL-3.0-only · released ${ENGINE_RELEASED}</span></div>
<div class="cells no-top cols-auto-240">
${CHANNELS.map(
  (c) => `<div class="cell-pad channel">
<div class="row-between"><span class="kicker blue">${c.tag}</span><span class="mono muted small-mono">${c.req}</span></div>
<p class="surface-title">${c.title}</p>
<pre class="code-block">${command(c.cmd)}</pre>
<p class="small mt-12">${c.note}</p>
<a class="small-mono mt-12" href="${c.href}">${c.link} ↗</a>
</div>`,
).join("\n")}
</div>
</section>

<section class="mt-56">
<div class="section-head"><h2 id="files-title">Prebuilt files for v${ENGINE_VERSION}</h2><a href="${RELEASE_DL}/SHA256SUMS.txt">SHA256SUMS.txt ↓</a></div>
<div class="scroll-x"><div class="dl-table" role="table" aria-labelledby="files-title">
<div class="t-row head dl-cols" role="row"><span role="columnheader">Target</span><span role="columnheader">File</span><span class="r" role="columnheader">Size</span><span role="columnheader">Fonts</span><span role="columnheader">Status</span></div>
${ARCHIVES.map((a) => fileRow(a.target, a.file, a.size, "bundled")).join("\n")}
${WHEELS.map((w) => fileRow(w.target, w.file, w.size, "—")).join("\n")}
${pendingRows(ARCHIVES)}
</div></div>
<p class="note">CLI archives ship the renderer’s supplemental fonts (Roboto Condensed and Selawik, with their notices); <span class="code-inline">scripts/install.sh --fonts-only</span> installs just those. Point <span class="code-inline">JUBARTE_FONT_DIR</span> at your own corporate fonts for closer PDF parity. Once installed, <span class="code-inline">jubarte self-update --check</span> keeps the CLI current. <a href="${RELEASE_URL}">Release page ↗</a></p>
</section>

<section class="mt-56 two-col-release m-stack">
<div>
<h2 class="section-title">Recent releases</h2>
<p class="small mt-12">Pre-1.0: features bump the minor, fixes the patch. Pin an exact version in production and read the changelog before moving: the layout engine is still landing large Word-parity passes.</p>
<a class="btn btn-text mt-18" href="${CHANGELOG_URL}">Full changelog ↗</a>
</div>
<div class="releases">
${RELEASES.map((r) => `<div class="release"><span class="mono strong">${r.v}</span><span class="mono muted">${r.d}</span><span class="small">${esc(r.t)}</span></div>`).join("\n")}
</div>
</section>
</main>`;

export const download: Page = {
  file: "download.html",
  path: "/download",
  title: "Download Jubarte for Mac — plus the CLI, Rust, Python and wasm · Jubarte",
  description: `Jubarte ${APP_STORE.version} on the Mac App Store and jubarte-redlines ${ENGINE_VERSION} as a CLI, Rust crate, Python wheel and npm wasm package. Prebuilt archives with fonts and SHA256SUMS.`,
  nav: "download",
  body,
  scripts: ["download.js"],
  footer: [
    ["Contact", "/contact"],
    ["Privacy", "/privacy"],
    ["Terms", "/terms"],
    [SUPPORT_EMAIL, `mailto:${SUPPORT_EMAIL}`],
  ],
};
