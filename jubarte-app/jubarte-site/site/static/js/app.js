// The App page: the Mac app's window, driven by the real engine in this tab.
// The walkthrough and the Loaded / Result jumps use a real sample pair
// (static/demo/), so every redline shown here was just computed. Convert to
// PDF is the next release's second mode; its samples convert that redline.

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

const toast = toaster($("a-toast"));
const previewNow = instantPreview(/** @type {HTMLInputElement} */ ($("a-instant")));
swallowStrayDrops();

const FALLBACK_AUTHOR = "Jubarte";
const SAMPLES = [
  { url: "/static/demo/msa-v3.docx", name: "MSA — Acme, v3.docx", ageMs: 9 * 86_400_000 },
  { url: "/static/demo/msa-v4.docx", name: "MSA — Acme, v4 (K. Nguyen).docx", ageMs: 3_600_000 },
];

/** @typedef {{ name: string, size: number, modified: number, bytes: Uint8Array, author: string }} Doc */

/** @type {{ orig: Doc | null, mod: Doc | null, busy: boolean, result: { docx: Uint8Array } | null, stale: boolean }} */
const s = { orig: null, mod: null, busy: false, result: null, stale: false };
/** @type {{ doc: Doc | null, busy: boolean, result: { pdf: Uint8Array, pdfDoc: any } | null, stale: boolean }} */
const conv = { doc: null, busy: false, result: null, stale: false };
/** @type {"redline" | "convert"} */
let mode = "redline";
let authorTouched = false;
// Each mode proposes its own output name and remembers one the user typed.
const names = { redline: { value: "", touched: false }, convert: { value: "", touched: false } };

const authorEl = /** @type {HTMLInputElement} */ ($("a-author"));
const filenameEl = /** @type {HTMLInputElement} */ ($("a-filename"));
const runBtn = /** @type {HTMLButtonElement} */ ($("a-run"));
const revisionsEl = /** @type {HTMLSelectElement} */ ($("a-revisions"));
const stem = (/** @type {string} */ n) => n.replace(/\.docx$/i, "");
const busy = () => s.busy || conv.busy;

/** @param {File} f @returns {Promise<Doc>} */
async function load(f) {
  const bytes = new Uint8Array(await f.arrayBuffer());
  return {
    name: f.name,
    size: f.size,
    modified: f.lastModified,
    bytes,
    author: await documentAuthor(bytes),
  };
}

/** The sample pair, fetched once. @type {Promise<Doc[]> | undefined} */
let samples;
function samplePair() {
  samples ??= Promise.all(
    SAMPLES.map(async (x) => {
      const res = await fetch(x.url);
      if (!res.ok) throw new Error(`${x.url}: ${res.status}`);
      const bytes = new Uint8Array(await res.arrayBuffer());
      return {
        name: x.name,
        size: bytes.length,
        modified: Date.now() - x.ageMs,
        bytes,
        author: await documentAuthor(bytes),
      };
    }),
  );
  return samples;
}

function paintSlot(/** @type {"orig" | "mod" | "conv"} */ id) {
  const doc = id === "conv" ? conv.doc : s[id];
  const el = $(`a-${id}`);
  el.classList.toggle("filled", !!doc);
  $(`a-${id}-name`).replaceChildren(...(doc ? fileName(doc.name) : ["Drop a .docx"]));
  $(`a-${id}-meta`).textContent = doc
    ? `DOCX · ${size(doc.size)} · ${date(doc.modified)}`
    : "or click to browse";
}

function paint() {
  const converting = mode === "convert";
  for (const el of document.querySelectorAll("#app-window [data-mode]")) {
    /** @type {HTMLElement} */ (el).hidden = /** @type {HTMLElement} */ (el).dataset.mode !== mode;
  }
  for (const b of document.querySelectorAll("[data-mode-tab]")) {
    b.setAttribute("aria-pressed", String(/** @type {HTMLElement} */ (b).dataset.modeTab === mode));
  }
  paintSlot("orig");
  paintSlot("mod");
  paintSlot("conv");
  const ready = converting ? !!conv.doc : !!(s.orig && s.mod);
  const running = converting ? conv.busy : s.busy;
  const done = converting ? !!conv.result : !!s.result;
  if (!authorTouched) {
    const fromDoc = s.mod?.author ?? "";
    authorEl.value = fromDoc || FALLBACK_AUTHOR;
    $("a-author-hint").hidden = !fromDoc;
  }
  if (!names[mode].touched) filenameEl.value = proposedName();
  filenameEl.placeholder = converting ? "document.pdf" : "redline.docx";
  runBtn.disabled = busy() || !ready;
  $("a-run-label").textContent = converting
    ? running
      ? "Converting…"
      : "Convert to PDF"
    : running
      ? "Redlining…"
      : "Create redline";
  $("a-run-spin").hidden = !running;
  $("a-open").textContent = converting ? "Open PDF" : "Open in Word";
  for (const id of ["a-open", "a-reveal", "a-save"]) {
    /** @type {HTMLButtonElement} */ ($(id)).disabled = !done;
  }
  $("a-empty").hidden = done;
  $("a-result").hidden = converting || !s.result;
  $("a-result").classList.toggle("stale", s.stale);
  $("a-stale").hidden = !s.stale;
  $("a-pdf").hidden = !converting || !conv.result;
  $("a-pdf").classList.toggle("stale", conv.stale);
  $("a-pdf-stale").hidden = !conv.stale;
  $("a-empty-title").textContent = converting
    ? "Your PDF appears here."
    : "Your redline preview appears here.";
  $("a-empty-hint").textContent = converting
    ? ready
      ? "Press Convert to PDF, or ⏎."
      : "Drop a .docx, then Convert to PDF."
    : ready
      ? "Press Create redline, or ⏎."
      : "Drop two .docx files, then Create redline.";
  $("a-outname").textContent = outputName("redline");
  $("a-pdf-name").textContent = outputName("convert");
  const which = done ? "result" : ready ? "loaded" : "empty";
  for (const b of document.querySelectorAll("[data-jump]")) {
    b.setAttribute("aria-pressed", String(/** @type {HTMLElement} */ (b).dataset.jump === which));
  }
}

function proposedName() {
  if (mode === "convert") return conv.doc ? `${stem(conv.doc.name)}.pdf` : "";
  return s.orig && s.mod ? `${stem(s.orig.name)}_v_${stem(s.mod.name)}.docx` : "";
}

/** The name the output gets: what the field says, with its mode's extension. */
function outputName(/** @type {"redline" | "convert"} */ m = mode) {
  const typed = (m === mode ? filenameEl.value : names[m].value).trim();
  const ext = m === "convert" ? "pdf" : "docx";
  const base = typed.replace(/\.(docx|pdf)$/i, "") || (m === "convert" ? "document" : "redline");
  return `${base}.${ext}`;
}

function setMode(/** @type {"redline" | "convert"} */ next) {
  if (next === mode) return;
  names[mode].value = filenameEl.value;
  mode = next;
  filenameEl.value = names[mode].value;
  if (mode === "convert") engine("full");
  paint();
}
for (const b of document.querySelectorAll("[data-mode-tab]")) {
  b.addEventListener("click", () => {
    stopWalkthrough();
    setMode(/** @type {"redline" | "convert"} */ (/** @type {HTMLElement} */ (b).dataset.modeTab));
  });
}

async function assign(/** @type {File[]} */ files, /** @type {"orig" | "mod" | null} */ slot) {
  const docx = docxOnly(files);
  if (!docx.length) {
    if (files.length) toast("Only .docx files are supported.");
    return;
  }
  if (mode === "convert") {
    engine("full");
    const doc = await load(docx[0]);
    // The previous PDF is another document's: drop it, so Open PDF and Save a
    // copy cannot hand it out under the new document's name.
    conv.result?.pdfDoc.loadingTask.destroy();
    Object.assign(conv, { doc, result: null, stale: false });
    $("a-pages").textContent = "";
    if (docx.length > 1) toast(`Converting takes one document — ${docx[0].name} is in.`);
    paint();
    if (previewNow([doc.size])) await convert();
    return;
  }
  engine("slim");
  const docs = await Promise.all(docx.slice(0, 2).map(load));
  if (docs.length === 2) {
    docs.sort((a, b) => a.modified - b.modified);
    [s.orig, s.mod] = docs;
    toast("Older file placed as original — swap if that’s wrong.");
  } else {
    s[slot ?? (!s.orig ? "orig" : "mod")] = docs[0];
  }
  s.stale = !!s.result;
  paint();
  // Two files at once run straight away, as Open With does in the app; a
  // pair of small documents does too, as the app's Settings › Preview has it.
  if (docs.length === 2 || (s.orig && s.mod && previewNow([s.orig.size, s.mod.size]))) await run();
}

async function run() {
  if (mode === "convert") return convert();
  if (!s.orig || !s.mod || busy()) return;
  s.busy = true;
  paint();
  try {
    const who = authorEl.value.trim() || FALLBACK_AUTHOR;
    const out = await compare(s.orig.bytes, s.mod.bytes, who);
    s.result = { docx: out.docx };
    s.stale = false;
    const c = countRevisions(out.revisions);
    $("a-chips").innerHTML = revisionChips(c);
    $("a-outmeta").textContent =
      `Ready in ${seconds(out.ms)} · ${who} — “Save a copy” to choose where it goes`;
    const preview = await redlinePreview(out.docx);
    drawPreview($("a-paper"), preview);
    $("a-trunc").hidden = !preview.truncated;
  } catch (err) {
    toast(`The engine could not compare these files: ${err instanceof Error ? err.message : err}`);
  } finally {
    s.busy = false;
    paint();
  }
}

async function convert() {
  if (!conv.doc || busy()) return;
  conv.busy = true;
  paint();
  const loading = setTimeout(
    () => toast(`Loading the PDF engine — about ${runBtn.dataset.wire} over the wire, once.`),
    600,
  );
  const doc = conv.doc;
  const revisions = revisionsEl.value;
  try {
    const out = await toPdf(doc.bytes, { compress: true, revisions });
    clearTimeout(loading);
    const { openPdf, thumbnails } = await import("./pdfview.js");
    const pdfDoc = await openPdf(out.pdf);
    // Another document arrived while this one rendered: its result is stale.
    if (conv.doc !== doc) return void pdfDoc.loadingTask.destroy();
    // pdf.js 6 closes a document, and the worker it started, through its loading task.
    conv.result?.pdfDoc.loadingTask.destroy();
    conv.result = { pdf: out.pdf, pdfDoc };
    // The tracked-change choice moved mid-run: the PDF shows the old one.
    conv.stale = revisionsEl.value !== revisions;
    $("a-pdf-chips").innerHTML = [
      ["state", `${out.pages} ${out.pages === 1 ? "page" : "pages"}`],
      ["", size(out.pdf.length)],
    ]
      .map(([cls, t]) => `<span class="tag ${cls}">${t}</span>`)
      .join("");
    $("a-pdf-meta").textContent =
      `Ready in ${seconds(out.ms)} — “Save a copy” to choose where it goes`;
    thumbnails(pdfDoc, $("a-pages"));
  } catch (err) {
    toast(`The engine could not convert this file: ${err instanceof Error ? err.message : err}`);
  } finally {
    clearTimeout(loading);
    conv.busy = false;
    paint();
  }
}

for (const id of /** @type {const} */ (["orig", "mod"])) {
  const el = $(`a-${id}`);
  el.addEventListener("click", async () => assign(await pickFiles(true), id));
  dropTarget(el, (files) => assign(files, id));
}
$("a-conv").addEventListener("click", async () => assign(await pickFiles(false), null));
dropTarget($("a-conv"), (files) => assign(files, null));
revisionsEl.addEventListener("change", () => {
  conv.stale = !!conv.result;
  paint();
});
dropTarget($("app-window"), (files) => assign(files, null));

$("a-swap").addEventListener("click", () => {
  [s.orig, s.mod] = [s.mod, s.orig];
  $("a-swap").classList.toggle("flipped");
  s.stale = !!s.result;
  paint();
  if (s.orig && s.mod && previewNow([s.orig.size, s.mod.size])) run();
});
authorEl.addEventListener("input", () => {
  authorTouched = true;
  $("a-author-hint").hidden = true;
});
filenameEl.addEventListener("input", () => {
  names[mode].touched = true;
  $(mode === "convert" ? "a-pdf-name" : "a-outname").textContent = outputName();
});
runBtn.addEventListener("click", run);
document.addEventListener("keydown", (e) => {
  if (e.key !== "Enter" || typing(e) || runBtn.disabled) return;
  const t = /** @type {HTMLElement} */ (e.target);
  if (t.tagName !== "BUTTON" && t.tagName !== "A") run();
});

// The app's three file actions. A browser cannot open Word, Preview or
// Finder, so each one downloads the file and says what the app does instead.
function saveOutput() {
  if (mode === "convert") {
    if (!conv.result) return false;
    download(conv.result.pdf, outputName(), "application/pdf");
  } else {
    if (!s.result) return false;
    download(s.result.docx, outputName(), DOCX_TYPE);
  }
  return true;
}
$("a-open").addEventListener("click", () => {
  if (!saveOutput()) return;
  const app = mode === "convert" ? "your PDF viewer" : "Microsoft Word";
  toast(`Downloaded ${outputName()}. In the app, this opens it in ${app}.`);
});
$("a-reveal").addEventListener("click", () => {
  if (mode === "convert" ? !conv.result : !s.result) return;
  const what = mode === "convert" ? "PDF" : "redline";
  toast(`In the app, this reveals the ${what} in Finder, in the app’s container.`);
});
$("a-save").addEventListener("click", () => {
  if (saveOutput()) toast("Copy saved to your downloads.");
});

/* ---------- jumps and the walkthrough ---------- */

function reset() {
  Object.assign(s, { orig: null, mod: null, result: null, stale: false });
  authorTouched = false;
  names.redline.touched = false;
  $("a-swap").classList.remove("flipped");
}

function resetConvert() {
  conv.result?.pdfDoc.loadingTask.destroy();
  Object.assign(conv, { doc: null, result: null, stale: false });
  names.convert.touched = false;
  $("a-pages").textContent = "";
}

async function loadSamples() {
  const [a, b] = await samplePair();
  reset();
  s.orig = a;
  s.mod = b;
}

/** The sample pair's redline, the document Convert's samples turn into a PDF. @type {Promise<Doc> | undefined} */
let sampleRedline;
function redlineSample() {
  sampleRedline ??= samplePair().then(async ([a, b]) => {
    const out = await compare(a.bytes, b.bytes, b.author || FALLBACK_AUTHOR);
    return {
      name: `${stem(a.name)}_v_${stem(b.name)}.docx`,
      size: out.docx.length,
      modified: Date.now(),
      bytes: out.docx,
      author: b.author,
    };
  });
  return sampleRedline;
}

async function loadConvertSample() {
  const doc = await redlineSample();
  resetConvert();
  conv.doc = doc;
}

/** @type {Record<string, () => Promise<void>>} */
const JUMPS = {
  async empty() {
    if (mode === "convert") resetConvert();
    else reset();
    paint();
  },
  async loaded() {
    if (mode === "convert") await loadConvertSample();
    else await loadSamples();
    paint();
  },
  async result() {
    if (mode === "convert") await loadConvertSample();
    else await loadSamples();
    paint();
    await run();
  },
};
for (const b of document.querySelectorAll("[data-jump]")) {
  b.addEventListener("click", () => {
    stopWalkthrough();
    JUMPS[/** @type {HTMLElement} */ (b).dataset.jump ?? "empty"]().catch((e) => toast(String(e)));
  });
}

let walking = 0;
const wait = (/** @type {number} */ ms) => new Promise((r) => setTimeout(r, ms));
const caption = $("sim-caption");

function stopWalkthrough() {
  walking++;
  caption.hidden = true;
  $("sim-label").textContent = "Show me how it works";
  $("sim").classList.remove("on");
  $("a-orig").classList.remove("over");
  $("a-mod").classList.remove("over");
  $("a-conv").classList.remove("over");
}

async function walkthrough() {
  const me = ++walking;
  $("sim-label").textContent = "Stop the walkthrough";
  $("sim").classList.add("on");
  const step = async (
    /** @type {string} */ n,
    /** @type {string} */ text,
    /** @type {() => unknown} */ act,
    /** @type {number} */ ms,
  ) => {
    if (me !== walking) throw new Error("stopped");
    caption.hidden = false;
    $("sim-step").textContent = `Step ${n}`;
    $("sim-text").textContent = text;
    await act();
    paint();
    await wait(ms);
  };
  try {
    const [a, b] = await samplePair();
    engine("slim");
    engine("full");
    await step(
      "1 / 8",
      "Open Jubarte. You get two slots: Original on the left, Modified on the right.",
      () => {
        reset();
        resetConvert();
        setMode("redline");
      },
      2200,
    );
    await step(
      "2 / 8",
      "Drop the original into the left slot, or click it to browse.",
      () => $("a-orig").classList.add("over"),
      700,
    );
    await step(
      "2 / 8",
      "Drop the original into the left slot, or click it to browse.",
      () => {
        $("a-orig").classList.remove("over");
        s.orig = a;
      },
      1800,
    );
    await step(
      "3 / 8",
      "Drop the revised version on the right. “Revisions by” fills in from that file’s author.",
      () => $("a-mod").classList.add("over"),
      700,
    );
    await step(
      "3 / 8",
      "Drop the revised version on the right. “Revisions by” fills in from that file’s author.",
      () => {
        $("a-mod").classList.remove("over");
        s.mod = b;
      },
      2400,
    );
    // The engine runs under step 4's caption (the button spins), so step 5
    // only says "read the redline" once there is one to read.
    await step(
      "4 / 8",
      "Check the proposed file name, then press Create redline.",
      async () => {
        await wait(1200);
        if (me === walking) await run();
      },
      0,
    );
    await step(
      "5 / 8",
      "Read the redline on the right. Open it in Word, reveal it in Finder, or save a copy anywhere.",
      () => {},
      3600,
    );
    await step(
      "6 / 8",
      "Got the order backwards? Swap the slots and run again — the old result dims until you do.",
      () => {
        [s.orig, s.mod] = [s.mod, s.orig];
        s.stale = true;
      },
      2600,
    );
    await step(
      "6 / 8",
      "Swap back and run again: the redline is current once more.",
      async () => {
        [s.orig, s.mod] = [a, b];
        await run();
      },
      2000,
    );
    const redline = await redlineSample();
    await step(
      "7 / 8",
      "Need a PDF? Switch to Convert to PDF and drop one document — here, the redline you just made.",
      () => {
        setMode("convert");
        $("a-conv").classList.add("over");
      },
      900,
    );
    await step(
      "7 / 8",
      "Need a PDF? Switch to Convert to PDF and drop one document — here, the redline you just made.",
      () => {
        $("a-conv").classList.remove("over");
        conv.doc = redline;
      },
      1800,
    );
    await step(
      "8 / 8",
      "Pick how tracked changes print, then press Convert to PDF.",
      async () => {
        await wait(1000);
        if (me === walking) await run();
      },
      0,
    );
    await step(
      "8 / 8",
      "That’s the PDF, page by page. Nothing was uploaded: both engines ran in this tab.",
      () => {},
      3200,
    );
    stopWalkthrough();
    toast("Walkthrough finished — the window is yours.");
  } catch (e) {
    if (me === walking) {
      stopWalkthrough();
      toast(String(e instanceof Error ? e.message : e));
    }
  }
}
$("sim").addEventListener("click", () => {
  if ($("sim").classList.contains("on")) stopWalkthrough();
  else walkthrough();
});

authorEl.value = FALLBACK_AUTHOR;
paint();
