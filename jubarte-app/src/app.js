// Jubarte frontend: two slots, one button. All real work happens in Rust;
// this file is drag-and-drop plumbing plus rendering the outcome.

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const state = { original: null, modified: null, busy: false, result: null };

// The author field auto-fills from the *modified* document's author until the
// user types their own; the filename field auto-proposes until the user edits.
let authorTouched = false;
let filenameTouched = false;
let fallbackAuthor = "Jubarte";

const $ = (id) => document.getElementById(id);
const zones = { original: $("zone-original"), modified: $("zone-modified") };
const runBtn = $("run");
const authorInput = $("author");
const authorHint = $("author-hint");
const filenameInput = $("filename");
const resultBtns = ["open-word", "reveal", "save-copy"].map($);

/* ---------- formatting ---------- */

const fmtSize = (b) => {
  if (b < 1024) return `${b} B`;
  if (b < 1024 * 1024) return `${(b / 1024).toFixed(0)} KB`;
  return `${(b / 1024 / 1024).toFixed(1)} MB`;
};
const fmtDate = (ms) =>
  ms ? new Date(ms).toLocaleDateString(undefined, { day: "numeric", month: "short", year: "numeric" }) : "";

const stem = (name) => name.replace(/\.docx$/i, "");
const proposedName = () =>
  state.original && state.modified ? `${stem(state.original.name)}_v_${stem(state.modified.name)}.docx` : "";

/* ---------- toasts ---------- */

function toast(msg, kind = "info", ttl = 4200) {
  const el = document.createElement("div");
  el.className = `toast ${kind}`;
  el.textContent = msg;
  $("toasts").appendChild(el);
  setTimeout(() => el.remove(), ttl);
}

/* ---------- slots ---------- */

function renderSlot(slot) {
  const zone = zones[slot];
  const info = state[slot];
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

function updateCta() {
  runBtn.disabled = state.busy || !(state.original && state.modified);
  runBtn.classList.toggle("busy", state.busy);
  runBtn.querySelector(".cta-label").textContent = state.busy ? "Redlining…" : "Create redline";
}

function updateFilename() {
  if (!filenameTouched) filenameInput.value = proposedName();
}

function staleResult() {
  if (state.result) $("result").classList.add("stale");
}

function renderAll() {
  renderSlot("original");
  renderSlot("modified");
  updateCta();
  updateFilename();
}

/* ---------- author default (from the modified document) ---------- */

async function refreshAuthorDefault() {
  if (authorTouched) return;
  let fromDoc = "";
  if (state.modified) {
    fromDoc = (await invoke("document_author", { path: state.modified.path }).catch(() => "")).trim();
  }
  authorInput.value = fromDoc || fallbackAuthor;
  authorHint.hidden = !fromDoc;
}

authorInput.addEventListener("input", () => {
  authorTouched = true;
  authorHint.hidden = true;
});
filenameInput.addEventListener("input", () => {
  filenameTouched = true;
});

/* ---------- file intake ---------- */

async function assign(paths, targetSlot = null, autorun = false) {
  const infos = await invoke("stat_files", { paths });
  if (!infos.length) {
    toast("Only .docx files are supported.", "warn");
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
  staleResult();
  // Resolve the modified doc's author before a possible auto-run, so the
  // attribution the user sees in the field is the one the redline actually
  // uses (the two race otherwise: run() would fire on the stale fallback).
  await refreshAuthorDefault();
  // Finder "Open with… → Jubarte" on two files: redline right away.
  if (autorun && infos.length >= 2 && !state.busy) run();
}

async function browse(slot) {
  const picked = await window.__TAURI__.dialog.open({
    multiple: true,
    filters: [{ name: "Word documents", extensions: ["docx"] }],
  });
  if (!picked) return;
  const paths = Array.isArray(picked) ? picked : [picked];
  assign(paths, paths.length === 1 ? slot : null);
}

for (const [slot, zone] of Object.entries(zones)) {
  zone.addEventListener("click", () => !state.busy && browse(slot));
  zone.addEventListener("keydown", (e) => {
    if (e.key === "Enter" || e.key === " ") browse(slot);
  });
}

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
  if (state.busy) return;
  assign(e.payload.paths, zone ? zone.dataset.slot : null);
});

/* ---------- swap ---------- */

$("swap").addEventListener("click", () => {
  if (state.busy) return;
  [state.original, state.modified] = [state.modified, state.original];
  $("swap").classList.toggle("spun");
  renderAll();
  staleResult();
  refreshAuthorDefault();
});

/* ---------- run ---------- */

async function run() {
  if (runBtn.disabled) return;
  // Gate the redline engine behind an active subscription; the paywall overlay
  // (paywall.js) also covers the UI, this guards the keyboard-Enter path.
  // Fail closed if paywall.js has not initialized yet (script order / race).
  if (!window.jubarte || !window.jubarte.requireAccess()) return;
  state.busy = true;
  updateCta();
  try {
    const r = await invoke("create_redline", {
      original: state.original.path,
      modified: state.modified.path,
      author: authorInput.value,
      filename: filenameInput.value.trim() || null,
    });
    state.result = r;
    showResult(r);
    window.jubarte?.noteUse?.();
  } catch (err) {
    const msg = String(err);
    // Rust-side free-quota gate: open the paywall instead of an error toast.
    if (msg.includes("FREE_LIMIT_REACHED")) {
      window.jubarte?.gate?.();
    } else {
      toast(msg, "error", 7000);
    }
  } finally {
    state.busy = false;
    updateCta();
  }
}
runBtn.addEventListener("click", run);
document.addEventListener("keydown", (e) => {
  if (e.key === "Enter" && !e.target.closest(".dropzone") && !e.target.matches("input") && !runBtn.disabled) run();
});

/* ---------- result ---------- */

const KIND_TAG = { ins: "ins", del: "del", moveins: "span", movedel: "span" };

function showResult(r) {
  $("preview-empty").hidden = true;
  const sec = $("result");
  sec.hidden = false;
  sec.classList.remove("stale");
  for (const b of resultBtns) b.disabled = false;

  $("outname").textContent = r.output_name;
  const secs = (r.elapsed_ms / 1000).toFixed(r.elapsed_ms < 9500 ? 1 : 0);
  const who = authorInput.value.trim();
  $("outmeta").textContent = `Ready in ${secs}s${who ? ` · ${who}` : ""} — “Save a copy” to choose where it goes`;

  const chip = (id, n, label) => {
    const el = $(id);
    el.hidden = n === 0;
    el.textContent = `${n} ${label}`;
  };
  chip("chip-ins", r.insertions, "Inserted");
  chip("chip-del", r.deletions, "Deleted");
  chip("chip-mov", r.moves, "Moved");
  chip("chip-fmt", r.format_changes, "Format");

  const paper = $("paper");
  paper.textContent = "";
  const frag = document.createDocumentFragment();
  for (const para of r.paragraphs) {
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
  $("truncnote").hidden = !r.truncated;
  sec.scrollIntoView({ behavior: "smooth", block: "nearest" });
}

$("open-word").addEventListener("click", () => {
  if (state.result) invoke("open_path", { path: state.result.output_path }).catch((e) => toast(String(e), "error"));
});
$("reveal").addEventListener("click", () => {
  if (state.result) invoke("reveal_path", { path: state.result.output_path }).catch((e) => toast(String(e), "error"));
});
$("save-copy").addEventListener("click", async () => {
  if (!state.result) return;
  const dest = await window.__TAURI__.dialog.save({
    defaultPath: state.result.output_name,
    filters: [{ name: "Word document", extensions: ["docx"] }],
  });
  if (!dest) return;
  try {
    await invoke("save_copy", { src: state.result.output_path, dest });
    toast("Copy saved.");
  } catch (e) {
    toast(String(e), "error");
  }
});

/* ---------- "Open with… Jubarte" ---------- */

listen("files-opened", (e) => assign(e.payload, null, true));

(async () => {
  fallbackAuthor = await invoke("default_author").catch(() => "Jubarte");
  authorInput.value = fallbackAuthor;
  const pending = await invoke("take_pending_files").catch(() => []);
  if (pending.length) assign(pending, null, true);
})();
