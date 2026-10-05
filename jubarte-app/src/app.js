// Jubarte frontend: two modes over one window. Redline takes two slots,
// Convert takes one. All real work happens in Rust; this file is drag-and-drop
// plumbing plus rendering the outcome.

const { invoke, convertFileSrc } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// pdf: the redline exported as a PDF (Export PDF), shown in its place until
// "Back to the redline". side: the original shown beside the redline.
const state = {
  original: null,
  modified: null,
  busy: false,
  result: null,
  pdf: null,
  exporting: false,
  side: localStorage.getItem("view") === "side",
};
const conv = { doc: null, busy: false, result: null };
let mode = localStorage.getItem("mode") === "convert" ? "convert" : "redline";

// The author field auto-fills from the *modified* document's author until the
// user types their own; the filename field auto-proposes until the user edits.
let authorTouched = false;
let fallbackAuthor = "Jubarte";
// Each mode proposes its own output name and remembers the one the user typed.
const names = { redline: { value: "", touched: false }, convert: { value: "", touched: false } };

const $ = (id) => document.getElementById(id);
const zones = { original: $("zone-original"), modified: $("zone-modified"), convert: $("zone-convert") };
const runBtn = $("run");
const authorInput = $("author");
const authorHint = $("author-hint");
const filenameInput = $("filename");

/* ---------- formatting ---------- */

const fmtSize = (b) => {
  if (b < 1024) return `${b} B`;
  if (b < 1024 * 1024) return `${(b / 1024).toFixed(0)} KB`;
  return `${(b / 1024 / 1024).toFixed(1)} MB`;
};
const fmtDate = (ms) =>
  ms ? new Date(ms).toLocaleDateString(undefined, { day: "numeric", month: "short", year: "numeric" }) : "";

const stem = (name) => name.replace(/\.docx$/i, "");
const proposedName = () => {
  if (mode === "convert") return conv.doc ? `${stem(conv.doc.name)}.pdf` : "";
  return state.original && state.modified ? `${stem(state.original.name)}_v_${stem(state.modified.name)}.docx` : "";
};
const busy = () => state.busy || conv.busy || state.exporting;
/** The PDF the window shows: the converted document, or the redline's. */
const shownPdf = () => (mode === "convert" ? conv.result : state.pdf);
/** What Open, Show in Finder and Save a copy take: what the window shows. */
const current = () => (mode === "convert" ? conv.result : (state.pdf ?? state.result));
const isPdf = (r) => /\.pdf$/i.test(r.output_name);

/* ---------- toasts ---------- */

function toast(msg, kind = "info", ttl = 4200) {
  const el = document.createElement("div");
  el.className = `toast ${kind}`;
  el.textContent = msg;
  $("toasts").appendChild(el);
  setTimeout(() => el.remove(), ttl);
}
// The menu bar (menu.js) speaks through the same toasts.
window.jubarteToast = toast;

/* ---------- slots ---------- */

function renderSlot(slot) {
  const zone = zones[slot];
  const info = slot === "convert" ? conv.doc : state[slot];
  const name = zone.querySelector(".filename");
  const meta = zone.querySelector(".filemeta");
  if (info) {
    zone.classList.add("filled");
    name.textContent = info.name;
    meta.textContent = `DOCX · ${fmtSize(info.size)} · ${fmtDate(info.modified_ms)}`;
  } else {
    zone.classList.remove("filled");
    name.innerHTML = "Drop a .docx<br/><em>or click to browse</em>";
    meta.textContent = "";
  }
}

/** The documents the current mode has. */
const docsIn = () => (mode === "convert" ? [conv.doc] : [state.original, state.modified]).filter(Boolean);
const haveAll = () => docsIn().length === (mode === "convert" ? 1 : 2);
/** The run button can act: every document is in and nothing is running. */
const ready = () => runBtn.getAttribute("aria-disabled") !== "true";

/**
 * A control that cannot act stays pressable and says why (panel.js nudges);
 * `why` null means it can act.
 */
function setBlocked(el, why) {
  el.setAttribute("aria-disabled", String(!!why));
  if (why) el.dataset.why = why;
  else delete el.dataset.why;
}

function updateCta() {
  const running = mode === "convert" ? conv.busy : state.busy;
  // panel.js, a module, words the reasons; until it loads, the bare rule.
  const blocker = haveAll()
    ? null
    : (window.jubartePanel?.runBlocker(mode, { original: state.original, modified: state.modified, doc: conv.doc }) ?? {
        text: "Add the documents first.",
        slot: "",
      });
  setBlocked(runBtn, busy() ? "Working on it — a moment." : (blocker?.text ?? null));
  runBtn.dataset.slot = blocker?.slot ?? "";
  runBtn.setAttribute("aria-busy", String(running));
  runBtn.classList.toggle("busy", running);
  runBtn.querySelector(".cta-label").textContent =
    mode === "convert" ? (running ? "Laying out pages…" : "Convert to PDF") : running ? "Creating redline…" : "Create redline";
  const hint = blocker && !running ? (window.jubartePanel?.RUN_HINT[blocker.slot] ?? "") : "";
  $("run-hint").textContent = hint ? `— ${hint}` : "";
  const done = !!current();
  const why = window.jubartePanel?.outputBlocker(mode, { docs: docsIn().length, result: done }) ?? "Nothing to take yet.";
  for (const id of ["open-word", "open-pdf", "reveal", "save-copy"]) setBlocked($(id), done ? null : why);
  // Work under way: the export and the documents wait for it, and say so.
  const working = busy() ? "Working on it — a moment." : null;
  setBlocked($("export-pdf"), state.exporting ? null : working);
  setBlocked($("rail-add"), working);
  paintEmpty();
}

/** The empty preview's two lines: what it waits for, and what to do. */
function paintEmpty() {
  const n = docsIn().length;
  const running = mode === "convert" ? conv.busy : state.busy;
  const empty = $("preview-empty");
  empty.classList.toggle("busy", running);
  empty.setAttribute("aria-disabled", String(running || busy()));
  const missing = !state.original ? "original" : "modified";
  if (busy() && !running) empty.dataset.why = "Working on it — a moment.";
  else delete empty.dataset.why;
  const [title, hint] = running
    ? [mode === "convert" ? "Laying out pages…" : "Comparing on this Mac…", "Nothing is uploaded."]
    : busy()
      ? ["Working on it — a moment.", "The other mode is still at work."]
      : mode === "convert"
      ? n
        ? ["Ready to convert.", "Press Convert to PDF."]
        : ["Drop a Word document for a PDF.", "Or click here to choose it."]
      : n === 2
        ? ["Ready to compare.", "Press Create redline."]
        : n === 1
          ? ["One more .docx for a redline.", `Or click here to choose the ${missing}.`]
          : ["Drop two Word documents to compare.", "Or click here to choose them."];
  $("empty-title").textContent = title;
  $("empty-hint").textContent = hint;
}

function updateFilename() {
  if (!names[mode].touched) filenameInput.value = proposedName();
  filenameInput.placeholder = mode === "convert" ? "document.pdf" : "redline.docx";
}

// A result whose inputs changed is drawn faded until it is made again. A
// redline's PDF goes stale with its redline (new documents) and with the PDFs
// (new marks).
function staleResult(which) {
  const results = which === "convert" ? [conv.result, state.pdf] : [state.result, state.pdf];
  for (const r of results) if (r) r.stale = true;
  paintPanes();
}

function renderAll() {
  renderSlot("original");
  renderSlot("modified");
  renderSlot("convert");
  updateCta();
  updateFilename();
}

/* ---------- modes ---------- */

function setMode(next, focus = false) {
  if (next === mode && document.body.dataset.mode === mode) return;
  names[mode].value = filenameInput.value;
  mode = next;
  localStorage.setItem("mode", mode);
  document.body.dataset.mode = mode;
  filenameInput.value = names[mode].value;
  for (const tab of document.querySelectorAll("[data-mode-tab]")) {
    const on = tab.dataset.modeTab === mode;
    tab.setAttribute("aria-selected", String(on));
    tab.tabIndex = on ? 0 : -1;
    if (on && focus) tab.focus();
  }
  for (const el of document.querySelectorAll("[data-mode]")) {
    if (el !== document.body) el.hidden = el.dataset.mode !== mode;
  }
  paintPanes();
  renderAll();
}

/**
 * Show the active mode's result (a redline, its PDF, or a converted PDF), or
 * the empty state when it has none.
 */
function paintPanes() {
  const pdf = shownPdf();
  $("result").hidden = mode !== "redline" || !state.result || !!state.pdf;
  $("result").classList.toggle("stale", !!state.result?.stale);
  $("pdf-result").hidden = !pdf;
  $("pdf-tools").hidden = mode !== "redline";
  if (pdf) paintPdf(pdf);
  // A redline's PDF opens as a PDF, not in Word.
  $("open-word").hidden = $("note-word").hidden = mode !== "redline" || !!state.pdf;
  $("open-pdf").hidden = $("note-pdf").hidden = !(mode === "convert" || state.pdf);
  $("preview-empty").hidden = !!current();
}

for (const tab of document.querySelectorAll("[data-mode-tab]")) {
  tab.addEventListener("click", () => setMode(tab.dataset.modeTab));
  tab.addEventListener("keydown", (e) => {
    if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
      e.preventDefault();
      setMode(mode === "redline" ? "convert" : "redline", true);
    }
  });
}

/* ---------- author default (from the modified document) ---------- */

async function refreshAuthorDefault() {
  if (authorTouched) return;
  // Settings → "Revisions by" a fixed name wins over the document's author.
  const fixed = window.jubarteSettings?.fixedAuthor() ?? "";
  let fromDoc = "";
  if (!fixed && state.modified) {
    fromDoc = (await invoke("document_author", { path: state.modified.path }).catch(() => "")).trim();
  }
  authorInput.value = fixed || fromDoc || fallbackAuthor;
  authorHint.hidden = !fromDoc;
}
// settings.js calls this when the "Revisions by" setting changes.
window.jubarteRefreshAuthor = refreshAuthorDefault;

authorInput.addEventListener("input", () => {
  authorTouched = true;
  authorHint.hidden = true;
});
filenameInput.addEventListener("input", () => {
  names[mode].touched = true;
});
// Settings changed the tracked-change marks: a PDF on show has the old ones.
window.addEventListener("jb-marks", () => staleResult("convert"));

/* ---------- file intake ---------- */

async function assign(paths, targetSlot = null, autorun = false) {
  const infos = await invoke("stat_files", { paths });
  if (!infos.length) {
    toast("Only .docx files are supported.", "warn");
    return;
  }
  if (mode === "convert") {
    // The previous PDF is another document's: drop it, so Open, Show in
    // Finder and Save a copy cannot act on it.
    conv.doc = infos[0];
    conv.result = null;
    if (infos.length > 1) toast(`Converting takes one document — ${infos[0].name} is in.`);
    paintPanes();
    renderAll();
    if (!autorun) await previewAtOnce();
    return;
  }
  if (infos.length >= 2) {
    const [a, b] = infos.slice(0, 2).sort((x, y) => x.modified_ms - y.modified_ms);
    state.original = a;
    state.modified = b;
    toast("Older file placed as original — swap if that’s wrong.");
  } else {
    const slot = targetSlot ?? (!state.original ? "original" : "modified");
    state[slot] = infos[0];
  }
  renderAll();
  staleResult("redline");
  // Resolve the modified doc's author before a possible auto-run, so the
  // attribution the user sees in the field is the one the redline actually
  // uses (the two race otherwise: run() would fire on the stale fallback).
  await refreshAuthorDefault();
  // Finder "Open with… → Jubarte" on two files: redline right away.
  if (autorun && infos.length >= 2 && !busy()) await run();
  else await previewAtOnce();
}

/**
 * Small documents preview the moment they are chosen (Settings › Preview).
 * A preview spends nothing, but one never opens the paywall on its own: with
 * no free use left and no subscription, the user presses the button.
 */
async function previewAtOnce() {
  const docs = mode === "convert" ? [conv.doc] : [state.original, state.modified];
  if (busy() || !ready() || docs.some((d) => !d)) return;
  // A result that is still current is not made again: that would only add a
  // second preview to pay for.
  const made = mode === "convert" ? conv.result : state.result;
  if (made && !made.stale) return;
  if (!window.jubarteSettings?.previewAtOnce(docs.map((d) => d.size))) return;
  const access = window.jubarte;
  if (!access?.entitled && !(access?.quota?.remaining > 0)) return;
  await run();
}
// The panel's Instant preview switch, turned on with the documents in.
window.jubartePreviewNow = previewAtOnce;

async function browse(slot) {
  const picked = await window.__TAURI__.dialog.open({
    multiple: slot !== "convert",
    filters: [{ name: "Word documents", extensions: ["docx"] }],
  });
  if (!picked) return;
  const paths = Array.isArray(picked) ? picked : [picked];
  assign(paths, paths.length === 1 ? slot : null);
}

for (const [slot, zone] of Object.entries(zones)) {
  zone.addEventListener("click", () => !busy() && browse(slot));
  zone.addEventListener("keydown", (e) => {
    if ((e.key === "Enter" || e.key === " ") && !busy()) browse(slot);
  });
}

// The empty preview chooses what the mode still needs: the convert slot, the
// first empty redline slot, or both at once.
$("preview-empty").addEventListener("click", () => {
  if (mode === "convert") return browse("convert");
  if (!state.original && !state.modified) return browse(null);
  browse(state.original ? "modified" : "original");
});
// panel.js redraws once it has loaded the reasons.
window.jubarteRender = renderAll;

/* ---------- native drag & drop ---------- */

function zoneAt(position) {
  const dpr = window.devicePixelRatio || 1;
  const el = document.elementFromPoint(position.x / dpr, position.y / dpr);
  return el ? el.closest(".dropzone") : null;
}

function highlight(zone) {
  for (const z of Object.values(zones)) z.classList.toggle("hover", z === zone);
}

listen("tauri://drag-enter", (e) => highlight(zoneAt(e.payload.position)));
listen("tauri://drag-over", (e) => highlight(zoneAt(e.payload.position)));
listen("tauri://drag-leave", () => highlight(null));
listen("tauri://drag-drop", (e) => {
  const zone = zoneAt(e.payload.position);
  highlight(null);
  if (busy()) return;
  const slot = zone?.dataset.slot;
  assign(e.payload.paths, slot === "convert" ? null : slot);
});

/* ---------- swap ---------- */

$("swap").addEventListener("click", () => {
  if (busy()) return;
  [state.original, state.modified] = [state.modified, state.original];
  $("swap").classList.toggle("spun");
  renderAll();
  staleResult("redline");
  refreshAuthorDefault().then(previewAtOnce);
});

/* ---------- run ---------- */

// A run the Rust free-quota gate refused opens the paywall instead of a toast.
function failed(err) {
  const msg = String(err);
  if (msg.includes("FREE_LIMIT_REACHED")) window.jubarte?.gate?.();
  else toast(msg, "error", 7000);
}

// The run in flight, if any: a Finder hand-off waits for it (takePending).
let active = Promise.resolve();

async function run() {
  if (!ready()) return;
  // The app is the paid product: both modes need an active subscription or a
  // free use left, though the run itself spends none (take() does). The
  // paywall overlay (paywall.js) also covers the UI, this guards the
  // keyboard-Enter path. Fail closed if paywall.js has not initialized yet
  // (script order / race).
  if (!window.jubarte || !window.jubarte.requireAccess()) return;
  const job = mode === "convert" ? convert() : redline();
  active = job;
  return job;
}

async function redline() {
  state.busy = true;
  updateCta();
  const { original, modified } = state;
  try {
    const r = await invoke("create_redline", {
      original: original.path,
      modified: modified.path,
      author: authorInput.value,
      fingerprint: window.jubarteSettings?.fingerprint() || null,
      filename: filenameInput.value.trim() || null,
    });
    r.originalName = original.name;
    // The documents changed while it ran: it is the old pair's redline.
    if (state.original !== original || state.modified !== modified) r.stale = true;
    state.result = r;
    // A new redline: the PDF of the last one is gone with it.
    state.pdf = null;
    showResult(r);
  } catch (err) {
    failed(err);
  } finally {
    state.busy = false;
    updateCta();
  }
}

/** The marks a PDF is painted with: Settings' choice, or the menu's alone. */
function currentMarks() {
  return window.jubarteSettings?.convertArgs() ?? { revisions: "conventional", revisionPalette: null };
}

async function convert() {
  conv.busy = true;
  updateCta();
  const doc = conv.doc;
  const marks = currentMarks();
  try {
    const r = await invoke("convert_document", {
      input: doc.path,
      ...marks,
      filename: filenameInput.value.trim() || null,
    });
    // Another document arrived while this one converted: its PDF is not shown.
    if (conv.doc !== doc) return;
    conv.result = r;
    paintPanes();
    // The tracked-change choice moved mid-run: the PDF shows the old one.
    if (JSON.stringify(currentMarks()) !== JSON.stringify(marks)) staleResult("convert");
  } catch (err) {
    failed(err);
  } finally {
    conv.busy = false;
    updateCta();
  }
}
runBtn.addEventListener("click", run);

/**
 * Export PDF: the redline itself laid out as a PDF, its tracked changes drawn
 * with the marks Settings chose. Like any run, it makes a free preview; taking
 * the PDF spends a use.
 */
async function exportPdf() {
  if (!state.result) return toast("Create the redline first.");
  if (busy()) return;
  setMode("redline");
  const source = state.result;
  const marks = currentMarks();
  const key = JSON.stringify(marks);
  // Back to the redline and Export PDF again: the same PDF, so taking it
  // again spends nothing.
  if (source.pdf && source.pdfKey === key) return landPdf(source.pdf);
  if (!window.jubarte?.requireAccess()) return;
  state.exporting = true;
  paintExport();
  updateCta();
  try {
    const r = await invoke("convert_document", {
      input: state.result.output_path,
      ...marks,
      // Named after the redline as made, not a taken copy's "(2)".
      filename: window.jubarteViews?.pdfNameFor(baseName(source.preview_path ?? source.output_path)) ?? null,
    });
    // A new redline arrived meanwhile: this PDF is of the old one.
    if (state.result !== source) return;
    if (source.stale) r.stale = true;
    Object.assign(source, { pdf: r, pdfKey: key });
    landPdf(r);
    if (JSON.stringify(currentMarks()) !== key) staleResult("convert");
  } catch (err) {
    failed(err);
  } finally {
    state.exporting = false;
    paintExport();
    updateCta();
  }
}

/** Shows the redline's PDF, and hands focus on to the way back. */
function landPdf(r) {
  const focused = document.activeElement === $("export-pdf");
  state.pdf = r;
  paintPanes();
  updateCta();
  if (focused) $("pdf-back").focus();
}

const baseName = (path) => path.split(/[\\/]/).pop();

function paintExport() {
  const btn = $("export-pdf");
  btn.classList.toggle("busy", state.exporting);
  btn.setAttribute("aria-busy", String(state.exporting));
  btn.querySelector(".tool-label").textContent = state.exporting ? "Exporting…" : "Export PDF";
}

// A Finder hand-off waits for an export as for any run (takePending).
$("export-pdf").addEventListener("click", () => {
  active = exportPdf();
});
$("pdf-back").addEventListener("click", () => {
  state.pdf = null;
  paintPanes();
  updateCta();
  $("export-pdf").focus();
});

/* ---------- the redline, alone or beside the original ---------- */

function setSide(on) {
  state.side = on;
  localStorage.setItem("view", on ? "side" : "single");
  $("view-single").setAttribute("aria-pressed", String(!on));
  $("view-side").setAttribute("aria-pressed", String(on));
  $("paper-pair").classList.toggle("two", on);
  $("original-col").hidden = !on;
  $("redline-cap").hidden = !on;
  if (on && state.result) paintOriginal(state.result);
}
$("view-single").addEventListener("click", () => setSide(false));
$("view-side").addEventListener("click", () => setSide(true));
setSide(state.side);

function paintOriginal(r) {
  const paragraphs = window.jubarteViews?.originalParagraphs(r.paragraphs) ?? [];
  paintPaper($("paper-original"), paragraphs);
  $("original-cap").textContent = `Original · ${r.originalName ?? ""}`;
  $("redline-cap").textContent = `Redlined · ${r.output_name}`;
}

document.addEventListener("keydown", (e) => {
  // Return presses the button, so a run that cannot start says why.
  if (e.key === "Enter" && !e.target.closest("button, a, .dropzone, dialog, [role=dialog]") && !e.target.matches("input, select")) runBtn.click();
});

/* ---------- result ---------- */

const KIND_TAG = { ins: "ins", del: "del", moveins: "span", movedel: "span" };

const secsOf = (ms) => (ms / 1000).toFixed(ms < 9500 ? 1 : 0);

/** Fills the PDF pane with `r`: the converted document's, or the redline's. */
function paintPdf(r) {
  const sec = $("pdf-result");
  sec.classList.toggle("stale", !!r.stale);
  $("pdf-name").textContent = r.output_name;
  paintMeta(r);
  $("chip-pages").textContent = `${r.pages} ${r.pages === 1 ? "page" : "pages"}`;
  $("chip-size").textContent = fmtSize(r.bytes);
  // The preview file, which stays put when the PDF is taken; set once, so a
  // repaint does not reload the viewer.
  const src = convertFileSrc(r.preview_path ?? r.output_path);
  if ($("pdfview").getAttribute("src") !== src) $("pdfview").src = src;
}

function showResult(r) {
  const sec = $("result");

  $("outname").textContent = r.output_name;
  r.who = authorInput.value.trim();
  paintMeta(r);

  const chip = (id, n, label) => {
    const el = $(id);
    el.hidden = n === 0;
    el.textContent = `${n} ${label}`;
  };
  chip("chip-ins", r.insertions, "Inserted");
  chip("chip-del", r.deletions, "Deleted");
  chip("chip-mov", r.moves, "Moved");
  chip("chip-fmt", r.format_changes, "Formatted");

  paintPaper($("paper"), r.paragraphs);
  if (state.side) paintOriginal(r);
  $("truncnote").hidden = !r.truncated;
  paintPanes();
  if (!sec.hidden) sec.scrollIntoView({ behavior: "smooth", block: "nearest" });
}

/** Draws preview paragraphs onto a sheet of paper. */
function paintPaper(paper, paragraphs) {
  paper.textContent = "";
  const frag = document.createDocumentFragment();
  for (const para of paragraphs) {
    const p = document.createElement("p");
    if (!para.runs.length) p.className = "blank";
    for (const run of para.runs) {
      if (run.kind === "same") {
        p.appendChild(document.createTextNode(run.text));
        continue;
      }
      const el = document.createElement(KIND_TAG[run.kind] ?? "span");
      if (run.kind === "moveins" || run.kind === "movedel") el.className = run.kind;
      el.textContent = run.text;
      if (run.author) el.title = run.author;
      p.appendChild(el);
    }
    frag.appendChild(p);
  }
  paper.appendChild(frag);
}

/** The result's line: how long it took, and what taking it costs. */
function paintMeta(r) {
  const ready = `Ready in ${secsOf(r.elapsed_ms)}s`;
  const hint =
    r.taken || window.jubarte?.entitled
      ? "“Save a copy” to choose where it goes"
      : "A free preview: opening, showing in Finder or saving it uses a free use";
  if (r === shownPdf()) $("pdf-meta").textContent = `${ready} · ${hint}`;
  if (r === state.result) $("outmeta").textContent = `${ready}${r.who ? ` · ${r.who}` : ""} · ${hint}`;
}

/**
 * Takes the shown result: the first Open, Show in Finder or Save a copy spends
 * a free use (Rust's take_result runs the gate) and copies the preview where
 * Finder can see it. Later ones reuse that copy. Null when the gate refused.
 */
async function take(r) {
  if (r.taken) return r;
  if (!window.jubarte?.requireAccess()) return null;
  try {
    // The preview stays where the window shows it from; r names the copy.
    const preview = r.output_path;
    Object.assign(r, await invoke("take_result", { path: r.output_path }), { taken: true, preview_path: preview });
    window.jubarte?.noteUse?.();
    if (r === state.result) {
      $("outname").textContent = r.output_name;
      $("redline-cap").textContent = `Redlined · ${r.output_name}`;
    }
    if (r === shownPdf()) $("pdf-name").textContent = r.output_name;
    paintMeta(r);
    return r;
  } catch (err) {
    failed(err);
    return null;
  }
}

const openResult = async () => {
  const r = current() && (await take(current()));
  if (r) invoke("open_path", { path: r.output_path }).catch((e) => toast(String(e), "error"));
};
$("open-word").addEventListener("click", openResult);
$("open-pdf").addEventListener("click", openResult);
$("reveal").addEventListener("click", async () => {
  const r = current() && (await take(current()));
  if (r) invoke("reveal_path", { path: r.output_path }).catch((e) => toast(String(e), "error"));
});
$("save-copy").addEventListener("click", async () => {
  const shown = current();
  if (!shown) return;
  // Choose the place first: cancelling here spends nothing.
  const dest = await window.__TAURI__.dialog.save({
    defaultPath: shown.output_name,
    filters: isPdf(shown)
      ? [{ name: "PDF document", extensions: ["pdf"] }]
      : [{ name: "Word document", extensions: ["docx"] }],
  });
  if (!dest) return;
  const r = await take(shown);
  if (!r) return;
  try {
    await invoke("save_copy", { src: r.output_path, dest });
    toast("Copy saved.");
  } catch (e) {
    toast(String(e), "error");
  }
});

/* ---------- Finder: "Open With → Jubarte", "Redline with Jubarte" ---------- */

// Rust stashes the files and then emits; whoever drains first gets them, so a
// launch that races this listener cannot assign (and redline) the pair twice.
async function takePending() {
  // A run the user started finishes first: the hand-off would take its
  // document and its result would be dropped after spending a free use.
  // The pause yields to the event loop: awaiting a job already settled alone
  // would spin in microtasks and starve the reply that ends the work.
  while (busy()) await Promise.all([active, new Promise((r) => setTimeout(r, 50))]);
  const { paths = [], intent = null } = await invoke("take_pending_files").catch(() => ({}));
  if (!paths.length) return;
  if (intent === "convert") return convertEach(paths);
  // "Redline with Jubarte", or two files from Open With, mean a redline.
  if (intent === "redline" || paths.length >= 2) setMode("redline");
  return assign(paths, null, true);
}

// One hand-off at a time: a request that arrives mid-batch waits for the
// batch. Run at once, it took the document from under the batch, found the
// button busy, and nothing after the first PDF converted.
let intake = Promise.resolve();
function takePendingInTurn() {
  intake = intake.then(takePending).catch((err) => toast(String(err), "error", 7000));
  return intake;
}

// "Convert to PDF with Jubarte": every selected document, one after another,
// each under its own name. A run the gate refuses, or one that fails, stops
// the rest.
async function convertEach(paths) {
  setMode("convert");
  names.convert.touched = false;
  let done = 0;
  for (const path of paths) {
    // As a hand-off, so the document is not also previewed on its own.
    await assign([path], null, true);
    // A file gone since Finder handed it over leaves the previous document
    // in; converting that again would spend a free use on a duplicate.
    if (conv.doc?.path !== path) break;
    await run();
    // assign drops the previous PDF, so a result here is this document's.
    if (!conv.result || conv.doc?.path !== path) break;
    // Finder asked for these PDFs: each is taken, and spends a use, as it is
    // made. One the gate refuses stops the rest.
    if (!(await take(conv.result))) break;
    done++;
  }
  if (done > 1) toast(`Converted ${done} documents — Show in Finder opens their folder.`);
}

setMode(mode);

(async () => {
  // Tauri does not replay an event emitted before its listener exists, so the
  // listener is in place before the startup drain: files Finder hands over in
  // between are then either drained here or announced to the listener.
  await listen("files-opened", takePendingInTurn).catch(() => {});
  fallbackAuthor = await invoke("default_author").catch(() => "Jubarte");
  await refreshAuthorDefault();
  await takePendingInTurn();
})();
