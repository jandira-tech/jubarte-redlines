import { fact } from "../data/facts.ts";
import {
  CHECK_ICON,
  DOC_ICON,
  DOC_ICON_MOD,
  EMPTY_ICON,
  type Page,
  SWAP_ICON,
  watermark,
} from "../layout.ts";
import { WASM_SIZE } from "../wasm-size.ts";

/** Documents under this many bytes preview the moment they are in. */
export const INSTANT_LIMIT = fact<number>("app.instant_preview_max_bytes");

/** A size as Finder shows it: 1,000,000 bytes is "1 MB". */
export const sizeLabel = (bytes: number) =>
  bytes >= 1_000_000 ? `${+(bytes / 1_000_000).toFixed(1)} MB` : `${Math.round(bytes / 1000)} KB`;

/** The switch for previewing small documents at once (static/js/instant.js). */
export const instantToggle = (id: string) =>
  `<label class="check instant"><input type="checkbox" id="${id}" data-limit="${INSTANT_LIMIT}" checked><span>Preview as soon as the documents are in, when each is under ${sizeLabel(INSTANT_LIMIT)}</span></label>`;

/** A drop slot. `kind` picks the icon; `dashed` is the single-file source slot. */
export function slot(
  id: string,
  label: string,
  kind: "orig" | "mod",
  hint: string,
  dashed = false,
) {
  return `<button type="button" class="slot${dashed ? " dashed" : ""}" id="${id}" data-slot="${id}" aria-describedby="${id}-meta">
<span class="slot-label">${label}</span>
${kind === "mod" ? DOC_ICON_MOD : DOC_ICON}
<span><span class="slot-name" id="${id}-name">Drop a .docx</span><span class="slot-meta" id="${id}-meta">${hint}</span></span>
</button>`;
}

const body = `<div class="page-bg">
${watermark()}
<main class="wrap">
<div class="page-head head-demo">
<div>
<p class="eyebrow">/ DEMO · RUNS IN THIS TAB · NOTHING UPLOADED</p>
<h1 class="h1"><span class="hl">Redlines Word opens as its own</span>, or a PDF.</h1>
<p class="lead">The engine is the same Rust core as the desktop app, compiled to WebAssembly and loaded into this page. Your files are read by your browser and never sent anywhere.</p>
</div>
<span class="pill">Free · unlimited · no account</span>
</div>

<div class="phone-note m-show"><span class="kicker blue">On a phone</span>Pick files from Files, Drive or Mail; the compare still runs on this device and the redline downloads here. Dropping two documents side by side is easier on a desktop.</div>

<div class="tabs" role="tablist" aria-label="Demo mode">
<button type="button" role="tab" id="tab-redline" aria-controls="panel-redline" aria-selected="true">Redline two documents</button>
<button type="button" role="tab" id="tab-convert" aria-controls="panel-convert" aria-selected="false" tabindex="-1">Convert to PDF / PNG</button>
</div>

<section class="demo-grid m-stack" id="panel-redline" role="tabpanel" aria-labelledby="tab-redline">
<div class="demo-controls">
<div class="slots">
${slot("orig", "Original", "orig", "or click to browse")}
<button type="button" class="swap-btn" id="swap" title="Swap original ↔ modified" aria-label="Swap original and modified">${SWAP_ICON}</button>
${slot("mod", "Modified", "mod", "or click to browse")}
</div>
<button type="button" class="btn-text small-mono" id="clear-slots" hidden>Clear both files</button>
<div class="sample-row"><button type="button" class="btn btn-outline" id="sample">Try it with a sample contract</button><p class="small">No files to hand? Two versions of a made-up services agreement come with this page.</p></div>
<label class="field"><span>Revisions by</span><input type="text" id="author" value="Jubarte" spellcheck="false" autocomplete="off"><span class="hint" id="author-hint" hidden>from modified doc</span></label>
<p class="small">The name Word shows beside each change. It starts as the modified document’s author.</p>
${instantToggle("instant")}
<button type="button" class="btn btn-primary btn-lg" id="run" disabled><span id="run-label">Create redline →</span><span class="spinner" id="run-spin" hidden></span></button>
<p class="small status" id="status" aria-live="polite"></p>
<ol class="steps">
<li><span>01</span>Both files are read into this tab’s memory.</li>
<li><span>02</span>The comparison runs in this tab: well under a second for a short pair, seconds for a long one.</li>
<li><span>03</span>The download is armed only once the compare succeeds.</li>
</ol>
</div>
<div class="demo-preview">
<div class="empty" id="r-empty">${EMPTY_ICON}<p>Your redline preview appears here.</p><span class="mono" id="r-empty-hint">Drop two .docx files, then <b>Create redline</b>.</span></div>
<div id="r-result" hidden>
<div class="result-head">
<div class="result-file">${CHECK_ICON}<div><strong id="r-name"></strong><span class="mono" id="r-meta"></span></div></div>
<div class="tags chips" id="r-chips"></div>
</div>
<div class="paper" id="r-paper" tabindex="0" aria-label="Redline preview"></div>
<div class="row mt-18">
<button type="button" class="btn btn-primary" id="r-download">Download redline ↓</button>
<button type="button" class="btn btn-outline" id="r-to-convert">Render this redline to PDF →</button>
</div>
<p class="small mt-12">Open the file in Word and use Review → Accept or Reject on each change, as with any tracked changes.</p>
</div>
</div>
</section>

<section class="demo-grid m-stack" id="panel-convert" role="tabpanel" aria-labelledby="tab-convert" hidden>
<div class="demo-controls">
${slot("conv", "Source", "orig", "or click to browse · tracked changes welcome", true)}
<div class="seg seg-2" role="group" aria-label="Output format">
<button type="button" id="fmt-pdf" aria-pressed="true">PDF</button>
<button type="button" id="fmt-png" aria-pressed="false">PNG pages</button>
</div>
<div>
<label class="label" for="palette">Revision colours</label>
<select class="select" id="palette">
<option value="word">Word’s own colours</option>
<option value="conventional">Red struck deletions, blue double-underlined insertions</option>
<option value="custom">Custom: deleted #AA0000 struck, inserted #0055FF double-underlined</option>
</select>
</div>
<div class="two-col">
<div><label class="label" for="dpi">Resolution (dpi)</label><input class="input mono" type="number" id="dpi" value="144" min="24" max="600" disabled></div>
<label class="check"><input type="checkbox" id="compress" checked><span>Smaller file</span></label>
</div>
${instantToggle("c-instant")}
<button type="button" class="btn btn-primary btn-lg" id="convert" data-wire="${WASM_SIZE.full.wire}" disabled><span id="convert-label">Convert to PDF →</span><span class="spinner" id="convert-spin" hidden></span></button>
<p class="small status" id="c-status" aria-live="polite"></p>
<p class="label">The same job on the command line</p>
<pre class="cli-echo" id="cli">$ jubarte convert FILE.docx --pdf --revisions word --compress</pre>
<p class="small" id="png-note" hidden>PNG pages here are drawn in your browser by pdf.js from jubarte’s PDF. The CLI’s <span class="code-inline">--png</span> rasterises natively.</p>
</div>
<div class="demo-preview">
<div class="empty" id="c-empty">${EMPTY_ICON}<p>Rendered pages appear here.</p><span class="mono">No Word or LibreOffice process is launched: the layout engine is in the wasm.</span></div>
<div id="c-result" hidden>
<div class="result-head">
<div class="result-file">${CHECK_ICON}<div><strong id="c-name"></strong><span class="mono" id="c-meta"></span></div></div>
<button type="button" class="btn btn-primary" id="c-download">Download PDF ↓</button>
</div>
<div class="page-grid" id="c-pages"></div>
</div>
</div>
</section>

<div class="cells cols-auto-240 mt-56">
<div class="cell-pad"><p class="note-title">Nothing is uploaded</p><p class="small mt-12">You can check this yourself. Run the sample once, which loads the engine into this tab. Then turn off Wi‑Fi and drop your own files: the redline still comes out.</p><details class="how-it-runs mt-12"><summary>How it runs</summary><p class="small mt-12">The comparison and the layout engine run as WebAssembly in a Web Worker in this tab. The ${WASM_SIZE.slim.raw} compare build (about ${WASM_SIZE.slim.wire} over the wire) starts downloading the moment you drop the first file or open the sample; the ${WASM_SIZE.full.raw} PDF build (about ${WASM_SIZE.full.wire}) only once Convert has a file.</p></details></div>
<div class="cell-pad"><p class="note-title">A real Word redline</p><p class="small mt-12">You get a .docx with genuine tracked revisions Word can accept, reject and filter by author: insertions, deletions, moves and formatting changes alike.</p></div>
<div class="cell-pad"><p class="note-title">Free, without a counter</p><p class="small mt-12">No account, no card, no limit on redlines or conversions in this tab. If you compare documents all day and want it in your own pipeline, <a href="/contact">talk to us</a>.</p></div>
</div>
</main>
</div>
<div class="toast off" id="toast" role="status"></div>`;

export const demo: Page = {
  file: "demo.html",
  path: "/demo",
  title: "Demo — redline two Word documents in your browser · Jubarte",
  description:
    "Drop two .docx files and get a Word redline with real tracked changes, or render a .docx to PDF. Runs as WebAssembly in your tab; nothing is uploaded.",
  nav: "demo",
  body,
  scripts: ["demo.js"],
};
