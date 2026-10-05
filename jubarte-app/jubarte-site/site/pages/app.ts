import { APP_NEXT, APP_RELEASE, APP_STORE, byPhase } from "../data/release.ts";
import { CHECK_ICON, EMPTY_ICON, type Page, SWAP_ICON, whale } from "../layout.ts";
import { WASM_SIZE } from "../wasm-size.ts";
import { INSTANT_LIMIT, instantToggle, sizeLabel, slot } from "./demo.ts";

const NOTES = [
  {
    k: "Intake",
    t: "Two files at once",
    d: `Drop two .docx files, or choose Open With → Jubarte on both: the older becomes the original, both slots fill, and the redline runs. One file fills the first empty slot. Jubarte ${APP_RELEASE.version}${byPhase(", in Apple’s review now,", "")} adds Finder’s Quick Actions: select two documents for Redline with Jubarte, or any number for Convert to PDF with Jubarte. It also previews documents under ${sizeLabel(INSTANT_LIMIT)} the moment they are in (Settings turns that off); a preview is free, and a free use counts when you open, show in Finder or save the result.`,
  },
  {
    k: "Attribution",
    t: "Author from the modified doc",
    d: "The author name stored in the modified document (its creator, or whoever saved it last). Type over it and the auto-fill stops; a “from modified doc” hint shows where the value came from. With no author in the file, the app uses your macOS user name.",
  },
  {
    k: "Output",
    t: "Proposed name, editable",
    d: "&lt;original&gt;_v_&lt;modified&gt;.docx, deduped with (n). Results land in the app’s sandbox container; “Save a copy…” grants write access wherever you choose.",
  },
  {
    k: "Preview",
    t: "Marked-up paper, not a diff",
    d: "Source Serif 4 on paper; insertions underlined in blue, deletions struck in red, and a move in green, double-struck where it left and double-underlined where it landed. Truncated for long documents; the full redline is in the file.",
  },
  {
    k: byPhase(`${APP_RELEASE.version} · in review`, "Convert"),
    t: "Convert to PDF",
    d: 'A second tab takes one .docx and writes a PDF with the same layout engine as the CLI’s <span class="code-inline">jubarte convert</span>. Tracked changes print in red, blue and green, or as Word prints them. The PDF shows in the window and opens in Preview.',
  },
  {
    k: "Price",
    t: "What it costs",
    d: `${byPhase(`Today the app is ${APP_STORE.price} once on the Mac App Store. Jubarte ${APP_RELEASE.version}, in Apple’s review now, is free for ${APP_NEXT.freeUses} uses,`, `On the Mac App Store it is ${APP_STORE.price}, with ${APP_NEXT.freeUses} free uses,`)} a redline or a PDF each, then the Pro Version at ${APP_NEXT.yearly}. Either way nothing leaves the Mac, and this page stays free. Want to help? The Pro Version is how.`,
  },
];

const body = `<div class="page-bg">
<main class="wrap">
<div class="page-head head-app">
<div>
<p class="eyebrow">/ THE MAC APP · ${APP_STORE.version} ON THE MAC APP STORE${byPhase(` · ${APP_RELEASE.version} IN APPLE’S REVIEW`, "")} · RUNS ON YOUR MAC</p>
<h1 class="h1">Two slots. One button.</h1>
<p class="lead">This window is the app’s, running the real engine in your browser. Click a slot to browse or drop files onto it; “Revisions by” fills from the modified document’s author and the file name proposes <span class="code-inline">&lt;a&gt;_v_&lt;b&gt;.docx</span>. The Convert tab is ${byPhase(`the PDF mode of ${APP_RELEASE.version}, in Apple’s review now`, "its PDF mode")}. Nothing is uploaded.</p>
</div>
<div class="sim-ctl">
<button type="button" class="btn btn-cta" id="sim"><span class="sim-dot"></span><span id="sim-label">Show me how it works</span></button>
<span class="label">or jump to</span>
<div class="seg mono-seg jump-seg" role="group" aria-label="Jump to a state">
<button type="button" data-jump="empty" aria-pressed="true">Empty</button>
<button type="button" data-jump="loaded" aria-pressed="false">Loaded</button>
<button type="button" data-jump="result" aria-pressed="false">Result</button>
</div>
</div>
</div>

<div class="app-window m-tight" id="app-window">
<div class="app-bar"><span class="lights"><span></span><span></span><span></span></span><span class="app-title">JUBARTE · REDLINE &amp; PDF ENGINE</span><span class="mono">v${APP_RELEASE.version}</span></div>
<div class="app-body m-stack">
<div class="app-left">
<div class="app-hero">
${whale("ap", { cls: "whale-float-sm", style: "width:130px;height:auto" })}
<h2 class="app-word">jubarte</h2>
<p data-mode="redline">Drop two Word documents. Get a <span class="hl">redline Word opens as its own</span>.</p>
<p data-mode="convert" hidden>Drop one Word document. Get a <span class="hl">PDF</span>, made on this Mac.</p>
</div>
<div class="app-sec">
<div class="seg mono-seg app-modes" role="group" aria-label="What to make">
<button type="button" data-mode-tab="redline" aria-pressed="true">Redline</button>
<button type="button" data-mode-tab="convert" aria-pressed="false">Convert to PDF</button>
</div>
<div class="slots" data-mode="redline">
${slot("a-orig", "Original", "orig", "or click to browse")}
<button type="button" class="swap-btn" id="a-swap" title="Swap original ↔ modified" aria-label="Swap original and modified">${SWAP_ICON}</button>
${slot("a-mod", "Modified", "mod", "or click to browse")}
</div>
<div class="slots single" data-mode="convert" hidden>
${slot("a-conv", "Document", "orig", "or click to browse")}
</div>
</div>
<div class="app-sec">
<label class="field" data-mode="redline"><span>Revisions by</span><input type="text" id="a-author" value="Jubarte" spellcheck="false" autocomplete="off" placeholder="Author"><span class="hint" id="a-author-hint" hidden>from modified doc</span></label>
<label class="field" data-mode="convert" hidden><span>Tracked changes</span><select id="a-revisions"><option value="conventional">Red, blue, green</option><option value="word">As Word prints</option></select></label>
<div class="app-actions">
<button type="button" class="btn btn-cta" id="a-run" data-wire="${WASM_SIZE.full.wire}" disabled><span id="a-run-label">Create redline</span><span class="spinner" id="a-run-spin" hidden></span></button>
<button type="button" class="btn btn-primary" id="a-open" disabled>Open in Word</button>
<button type="button" class="btn btn-outline" id="a-reveal" disabled>Show in Finder</button>
<button type="button" class="btn btn-ghost" id="a-save" disabled>Save a copy…</button>
</div>
${instantToggle("a-instant")}
<label class="field field-file"><span>File name</span><input type="text" id="a-filename" spellcheck="false" autocomplete="off" placeholder="redline.docx"></label>
</div>
</div>
<div class="app-right">
<div class="empty" id="a-empty">${EMPTY_ICON}<p id="a-empty-title">Your redline preview appears here.</p><span class="mono" id="a-empty-hint">Drop two .docx files, then Create redline.</span></div>
<div id="a-pdf" hidden>
<div class="result-head">
<div class="result-file">${CHECK_ICON}<div><strong id="a-pdf-name"></strong><span class="mono" id="a-pdf-meta"></span></div></div>
<div class="tags chips" id="a-pdf-chips"></div>
</div>
<div class="page-grid app-pages" id="a-pages"></div>
<p class="stale-note" id="a-pdf-stale" hidden>Inputs changed since this PDF. Press Convert to PDF again.</p>
</div>
<div id="a-result" hidden>
<div class="result-head">
<div class="result-file">${CHECK_ICON}<div><strong id="a-outname"></strong><span class="mono" id="a-outmeta"></span></div></div>
<div class="tags chips" id="a-chips"></div>
</div>
<div class="paper" id="a-paper" tabindex="0" aria-label="Redline preview"></div>
<p class="small" id="a-trunc" hidden>Preview truncated; the full redline is in the saved document.</p>
<p class="stale-note" id="a-stale" hidden>Inputs changed since this redline. Press Create redline again.</p>
</div>
</div>
</div>
<div class="sim-caption" id="sim-caption" hidden><span class="mono" id="sim-step"></span><span id="sim-text"></span></div>
<div class="toast inside off" id="a-toast" role="status"></div>
</div>

<div class="cells app-notes">
${NOTES.map((n) => `<div><p class="kicker">${n.k}</p><p class="note-title">${n.t}</p><p class="small">${n.d}</p></div>`).join("\n")}
</div>
<div class="row mt-36 m-col">
<a class="btn btn-primary btn-lg" href="${APP_STORE.url}">Get it on the Mac App Store ↗</a>
<a class="btn btn-outline btn-lg" href="/download">Every download</a>
<a class="btn btn-text btn-lg" href="/demo">Convert to PDF in the demo →</a>
</div>
</main>
</div>`;

export const pro: Page = {
  file: "pro.html",
  path: "/pro",
  title: "Jubarte PRO, the Mac app — two slots, one button · Jubarte",
  description:
    "Jubarte for Mac: drop two Word documents and get a redline with real tracked changes, compared on your Mac. Try the app’s window here, with the real engine running in your browser.",
  nav: "pro",
  body,
  scripts: ["app.js"],
};
