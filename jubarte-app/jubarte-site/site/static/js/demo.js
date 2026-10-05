// The Demo page: redline two documents and render a document to PDF or PNG,
// all in this tab with jubarte-wasm. Nothing is uploaded.

import {
  $,
  DOCX_TYPE,
  date,
  docxOnly,
  download,
  dropTarget,
  fileName,
  pickFiles,
  seconds,
  size,
  swallowStrayDrops,
  toaster,
  typing,
} from "./common.js";
import {
  countRevisions,
  documentAuthor,
  drawPreview,
  redlinePreview,
  revisionChips,
} from "./docx-preview.js";
import { compare, engine, toPdf } from "./engine.js";
import { instantPreview } from "./instant.js";
import { tabKey } from "./tabs.js";
import { writeZip } from "./zip.js";

const toast = toaster($("toast"));
const previewNow = instantPreview(
  /** @type {HTMLInputElement} */ ($("instant")),
  /** @type {HTMLInputElement} */ ($("c-instant")),
);
swallowStrayDrops();

/** @typedef {{ name: string, size: number, modified: number, bytes: Uint8Array, author: string, complete: boolean }} Doc */

/**
 * A .docx is a ZIP, and a ZIP ends with an end-of-central-directory record in
 * its last 64 KB. A truncated download or attachment lacks it.
 * @param {Uint8Array} b
 */
function zipComplete(b) {
  if (b[0] !== 0x50 || b[1] !== 0x4b) return false;
  for (let i = b.length - 22; i >= Math.max(0, b.length - 65_557); i--) {
    if (b[i] === 0x50 && b[i + 1] === 0x4b && b[i + 2] === 5 && b[i + 3] === 6) return true;
  }
  return false;
}

/** @param {File} f @returns {Promise<Doc>} */
async function load(f) {
  const bytes = new Uint8Array(await f.arrayBuffer());
  return {
    name: f.name,
    size: f.size,
    modified: f.lastModified,
    bytes,
    author: await documentAuthor(bytes),
    complete: zipComplete(bytes),
  };
}

const stem = (/** @type {string} */ n) => n.replace(/\.docx$/i, "");

/* ---------- tabs ---------- */

const tabs = { redline: $("tab-redline"), convert: $("tab-convert") };
const tabOrder = /** @type {const} */ (["redline", "convert"]);
const panels = { redline: $("panel-redline"), convert: $("panel-convert") };

function showTab(/** @type {"redline" | "convert"} */ which, focus = false) {
  for (const k of /** @type {const} */ (["redline", "convert"])) {
    tabs[k].setAttribute("aria-selected", String(k === which));
    tabs[k].tabIndex = k === which ? 0 : -1;
    panels[k].hidden = k !== which;
  }
  if (focus) tabs[which].focus();
  history.replaceState(null, "", which === "convert" ? "#convert" : location.pathname);
}
tabs.redline.addEventListener("click", () => showTab("redline"));
tabs.convert.addEventListener("click", () => showTab("convert"));
tabOrder.forEach((name, i) => {
  tabs[name].addEventListener("keydown", (e) => {
    const to = tabKey(e.key, i, tabOrder.length);
    if (to === null) return;
    e.preventDefault();
    showTab(tabOrder[to], true);
  });
});
if (location.hash === "#convert") showTab("convert");

/* ---------- redline ---------- */

/** @type {{ orig: Doc | null, mod: Doc | null, busy: boolean, result: { docx: Uint8Array, name: string } | null }} */
const red = { orig: null, mod: null, busy: false, result: null };
let authorTouched = false;
const author = /** @type {HTMLInputElement} */ ($("author"));
const run = /** @type {HTMLButtonElement} */ ($("run"));
const status = $("status");

/** Show a message in the status line; an error stays until the inputs change. */
function say(/** @type {string} */ text, error = false, raw = "") {
  status.replaceChildren(text);
  status.dataset.kind = error ? "error" : "";
  if (!raw) return;
  const more = document.createElement("details");
  const pre = document.createElement("pre");
  pre.textContent = raw;
  const sum = document.createElement("summary");
  sum.textContent = "Details";
  more.append(sum, pre);
  status.append(more);
}

/** New input answers an old error: clear it and the slot marks. */
function clearError() {
  if (status.dataset.kind === "error") status.replaceChildren();
  status.dataset.kind = "";
  for (const id of ["orig", "mod"]) $(id).classList.remove("bad");
}

function paintSlot(/** @type {"orig" | "mod"} */ id) {
  const doc = red[id];
  const el = $(id);
  el.classList.toggle("filled", !!doc);
  $(`${id}-name`).replaceChildren(...(doc ? fileName(doc.name) : ["Drop a .docx"]));
  $(`${id}-meta`).textContent = doc
    ? `DOCX · ${size(doc.size)} · ${date(doc.modified)}`
    : "or click to browse";
}

function paint() {
  paintSlot("orig");
  paintSlot("mod");
  $("clear-slots").hidden = !(red.orig || red.mod);
  const ready = !!(red.orig && red.mod);
  run.disabled = red.busy || !ready;
  $("r-empty-hint").innerHTML = ready
    ? "Press <b>Create redline</b>, or ⏎."
    : "Drop two .docx files, then <b>Create redline</b>.";
  $("run-label").textContent = red.busy ? "Redlining…" : "Create redline →";
  $("run-spin").hidden = !red.busy;
  if (!authorTouched) {
    const fromDoc = red.mod?.author ?? "";
    author.value = fromDoc || "Jubarte";
    $("author-hint").hidden = !fromDoc;
  }
}

function markStale() {
  if (!red.result) return;
  $("r-result").classList.add("stale");
  say("Inputs changed since this redline — press Create redline again.");
}

async function assign(/** @type {File[]} */ files, /** @type {"orig" | "mod" | null} */ slot) {
  const docx = docxOnly(files);
  if (!docx.length) {
    if (files.length) toast("Only .docx files are supported.", "warn");
    return;
  }
  engine("slim"); // start the download while the files are read
  const docs = await Promise.all(docx.slice(0, 2).map(load));
  if (docs.length === 2) {
    docs.sort((a, b) => a.modified - b.modified);
    [red.orig, red.mod] = docs;
    toast("Older file placed as original — swap if that’s wrong.");
  } else {
    const target = slot ?? (!red.orig ? "orig" : "mod");
    red[target] = docs[0];
  }
  clearError();
  markStale();
  paint();
  if (docs.length === 2) run.focus();
  await redlineNow();
}

/** Both documents in and each small: the redline, without the button. */
async function redlineNow() {
  if (red.orig && red.mod && previewNow([red.orig.size, red.mod.size])) await createRedline();
}

// The sample pair the App page redlines too (static/demo/), written for this site.
const SAMPLES = [
  { url: "/static/demo/msa-v3.docx", name: "MSA — Acme, v3.docx", ageMs: 9 * 86_400_000 },
  { url: "/static/demo/msa-v4.docx", name: "MSA — Acme, v4 (K. Nguyen).docx", ageMs: 3_600_000 },
];
$("sample").addEventListener("click", async () => {
  const btn = /** @type {HTMLButtonElement} */ ($("sample"));
  btn.disabled = true;
  try {
    const now = Date.now();
    const files = await Promise.all(
      SAMPLES.map(async (d) => {
        const res = await fetch(d.url);
        if (!res.ok) throw new Error(`${d.url}: HTTP ${res.status}`);
        return new File([await res.blob()], d.name, {
          type: DOCX_TYPE,
          lastModified: now - d.ageMs,
        });
      }),
    );
    await assign(files, null);
  } catch (err) {
    say(
      "The sample didn’t load. Reload the page and try again.",
      true,
      String(err instanceof Error ? err.message : err),
    );
  } finally {
    btn.disabled = false;
  }
});

for (const id of /** @type {const} */ (["orig", "mod"])) {
  const el = $(id);
  el.addEventListener("click", async () => assign(await pickFiles(true), id));
  dropTarget(el, (files) => assign(files, id));
}
dropTarget(panels.redline, (files) => assign(files, null));

$("swap").addEventListener("click", () => {
  [red.orig, red.mod] = [red.mod, red.orig];
  $("swap").classList.toggle("flipped");
  clearError();
  markStale();
  paint();
  redlineNow();
});

// Empty the slots: the button clears both, Delete on a focused slot clears it.
$("clear-slots").addEventListener("click", () => {
  red.orig = red.mod = null;
  clearError();
  markStale();
  paint();
  $("orig").focus();
});
for (const id of /** @type {const} */ (["orig", "mod"])) {
  $(id).addEventListener("keydown", (e) => {
    if (e.key !== "Delete" && e.key !== "Backspace") return;
    red[id] = null;
    clearError();
    markStale();
    paint();
  });
}

author.addEventListener("input", () => {
  authorTouched = true;
  $("author-hint").hidden = true;
});

async function createRedline() {
  if (!red.orig || !red.mod || red.busy) return;
  const broken = /** @type {const} */ (["orig", "mod"]).filter((id) => !red[id]?.complete);
  if (broken.length) {
    for (const id of broken) $(id).classList.add("bad");
    const names = broken.map((id) => `“${red[id]?.name}”`).join(" and ");
    say(
      `${broken.length === 1 ? `The file ${names} isn’t a complete Word document` : `The files ${names} aren’t complete Word documents`}: ${broken.length === 1 ? "it" : "they"} may have been cut short in the download or the attachment. Re-save from Word and drop again.`,
      true,
    );
    return;
  }
  red.busy = true;
  say("Comparing in this tab…");
  paint();
  try {
    const who = author.value.trim() || "Jubarte";
    const out = await compare(red.orig.bytes, red.mod.bytes, who);
    const name = `${stem(red.orig.name)}_v_${stem(red.mod.name)}.docx`;
    red.result = { docx: out.docx, name };
    const counts = countRevisions(out.revisions);
    $("r-name").replaceChildren(...fileName(name));
    $("r-meta").textContent = `Ready in ${seconds(out.ms)} · ${who} · ${size(out.docx.length)}`;
    $("r-chips").innerHTML = revisionChips(counts);
    const preview = await redlinePreview(out.docx);
    drawPreview($("r-paper"), preview);
    if (preview.truncated) {
      $("r-paper").insertAdjacentHTML(
        "beforeend",
        '<p class="small">Preview truncated — the full redline is in the download.</p>',
      );
    }
    $("r-empty").hidden = true;
    $("r-result").hidden = false;
    $("r-result").classList.remove("stale");
    status.replaceChildren();
    $("r-download").focus();
  } catch (err) {
    say(
      "The engine could not compare these two files. Check that each opens in Word, then try again.",
      true,
      String(err instanceof Error ? err.message : err),
    );
    toast("Compare failed — is each file a Word document?", "error");
  } finally {
    red.busy = false;
    paint();
  }
}
run.addEventListener("click", createRedline);
document.addEventListener("keydown", (e) => {
  if (e.key === "Enter" && !panels.redline.hidden && !typing(e) && !run.disabled) {
    const t = /** @type {HTMLElement} */ (e.target);
    if (t.tagName !== "BUTTON" && t.tagName !== "A") createRedline();
  }
});
$("r-download").addEventListener("click", () => {
  if (red.result) download(red.result.docx, red.result.name, DOCX_TYPE);
});
$("r-to-convert").addEventListener("click", () => {
  if (!red.result) return;
  conv.doc = {
    name: red.result.name,
    size: red.result.docx.length,
    modified: Date.now(),
    bytes: red.result.docx,
    author: "",
    complete: true,
  };
  clearConvert();
  paintConvert();
  showTab("convert", true);
  engine("full");
});

/* ---------- convert ---------- */

/** @type {{ doc: Doc | null, fmt: "pdf" | "png", busy: boolean, pdf: Uint8Array | null, pdfDoc: any }} */
const conv = { doc: null, fmt: "pdf", busy: false, pdf: null, pdfDoc: null };
const palette = /** @type {HTMLSelectElement} */ ($("palette"));
const dpi = /** @type {HTMLInputElement} */ ($("dpi"));
const compress = /** @type {HTMLInputElement} */ ($("compress"));
const convertBtn = /** @type {HTMLButtonElement} */ ($("convert"));

const CUSTOM = "deleted=#AA0000:strike,inserted=#0055FF:double-underline";

function cli() {
  const parts = ["$ jubarte convert", conv.doc ? shellQuote(conv.doc.name) : "FILE.docx"];
  if (conv.fmt === "pdf") parts.push("--pdf");
  else parts.push("--png", "--dpi", String(dpiValue()));
  parts.push("--revisions", palette.value);
  if (palette.value === "custom") parts.push("--revision-palette", `"${CUSTOM}"`);
  if (conv.fmt === "pdf" && compress.checked) parts.push("--compress");
  return parts.join(" ");
}

const shellQuote = (/** @type {string} */ s) =>
  /^[\w./-]+$/.test(s) ? s : `'${s.replace(/'/g, "'\\''")}'`;
const dpiValue = () => Math.min(600, Math.max(24, Math.round(Number(dpi.value) || 144)));

function paintConvert() {
  const el = $("conv");
  el.classList.toggle("filled", !!conv.doc);
  $("conv-name").replaceChildren(...(conv.doc ? fileName(conv.doc.name) : ["Drop a .docx"]));
  $("conv-meta").textContent = conv.doc
    ? `DOCX · ${size(conv.doc.size)} · ${date(conv.doc.modified)}`
    : "or click to browse · tracked changes welcome";
  $("fmt-pdf").setAttribute("aria-pressed", String(conv.fmt === "pdf"));
  $("fmt-png").setAttribute("aria-pressed", String(conv.fmt === "png"));
  dpi.disabled = conv.fmt === "pdf";
  compress.disabled = conv.fmt === "png";
  $("png-note").hidden = conv.fmt === "pdf";
  convertBtn.disabled = conv.busy || !conv.doc;
  $("convert-label").textContent = conv.busy
    ? "Rendering…"
    : conv.fmt === "pdf"
      ? "Convert to PDF →"
      : "Render PNG pages →";
  $("convert-spin").hidden = !conv.busy;
  $("c-download").textContent = conv.fmt === "pdf" ? "Download PDF ↓" : "Download PNG pages ↓";
  $("cli").textContent = cli();
}

/** Drop the last rendering: it belongs to the previous document. */
function clearConvert() {
  // pdf.js 6 closes a document, and the worker it started, through its loading task.
  conv.pdfDoc?.loadingTask.destroy();
  conv.pdf = null;
  conv.pdfDoc = null;
  $("c-pages").textContent = "";
  $("c-result").hidden = true;
  $("c-empty").hidden = false;
}

async function setConvertFile(/** @type {File[]} */ files) {
  const docx = docxOnly(files);
  if (!docx.length) {
    if (files.length) toast("Only .docx files are supported.", "warn");
    return;
  }
  engine("full");
  conv.doc = await load(docx[0]);
  $("c-status").replaceChildren();
  clearConvert();
  paintConvert();
  convertBtn.focus();
  if (previewNow([conv.doc.size])) await renderConvert();
}
$("conv").addEventListener("click", async () => setConvertFile(await pickFiles(false)));
dropTarget($("conv"), setConvertFile);
dropTarget(panels.convert, setConvertFile);

$("fmt-pdf").addEventListener("click", () => {
  conv.fmt = "pdf";
  paintConvert();
});
$("fmt-png").addEventListener("click", () => {
  conv.fmt = "png";
  paintConvert();
});
for (const el of [palette, dpi, compress]) el.addEventListener("input", paintConvert);

convertBtn.addEventListener("click", () => renderConvert());
async function renderConvert() {
  if (!conv.doc || conv.busy) return;
  conv.busy = true;
  paintConvert();
  const loading = setTimeout(
    () => toast(`Loading the PDF engine — about ${convertBtn.dataset.wire} over the wire, once.`),
    600,
  );
  const doc = conv.doc;
  try {
    const out = await toPdf(doc.bytes, {
      compress: conv.fmt === "pdf" && compress.checked,
      revisions: palette.value,
      palette: palette.value === "custom" ? CUSTOM : undefined,
    });
    clearTimeout(loading);
    const { openPdf, thumbnails } = await import("./pdfview.js");
    const pdfDoc = await openPdf(out.pdf);
    // Another document arrived while this one rendered: its result is stale.
    if (conv.doc !== doc) return void pdfDoc.loadingTask.destroy();
    conv.pdf = out.pdf;
    conv.pdfDoc = pdfDoc;
    $("c-name").textContent = `${stem(doc.name)}.pdf`;
    $("c-meta").textContent =
      `${out.pages} page${out.pages === 1 ? "" : "s"} · ${size(out.pdf.length)} · ${seconds(out.ms)} · ${palette.value === "word" ? "Word’s colours" : palette.value} revision colours`;
    thumbnails(conv.pdfDoc, $("c-pages"));
    $("c-empty").hidden = true;
    $("c-result").hidden = false;
  } catch (err) {
    const raw = String(err instanceof Error ? err.message : err);
    const cStatus = $("c-status");
    cStatus.replaceChildren(
      "The PDF could not be drawn from this document. Check that it opens in Word, then try again.",
    );
    const more = document.createElement("details");
    more.innerHTML = "<summary>Details</summary><pre></pre>";
    /** @type {HTMLElement} */ (more.querySelector("pre")).textContent = raw;
    cStatus.append(more);
    toast("Rendering failed.", "error");
  } finally {
    clearTimeout(loading);
    conv.busy = false;
    paintConvert();
  }
}

$("c-download").addEventListener("click", async () => {
  if (!conv.doc || !conv.pdf) return;
  const base = stem(conv.doc.name);
  if (conv.fmt === "pdf") {
    download(conv.pdf, `${base}.pdf`, "application/pdf");
    return;
  }
  const { pagePng } = await import("./pdfview.js");
  const res = dpiValue();
  const pages = conv.pdfDoc.numPages;
  toast(`Drawing ${pages} page${pages === 1 ? "" : "s"} at ${res} dpi…`);
  /** @type {[string, Uint8Array][]} */
  const files = [];
  for (let n = 1; n <= pages; n++)
    files.push([
      `${base}-page-${String(n).padStart(2, "0")}.png`,
      await pagePng(conv.pdfDoc, n, res),
    ]);
  if (files.length === 1) download(files[0][1], files[0][0], "image/png");
  else download(writeZip(files), `${base}-png.zip`, "application/zip");
});

paint();
paintConvert();
