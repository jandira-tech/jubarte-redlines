/**
 * Jubarte site — front end.
 *
 * Order of operations for a redline, and why:
 *   1. the compare runs locally, in a worker;
 *   2. only then do we POST /api/redline to charge the allowance;
 *   3. the download link is armed only if that POST returns 200.
 *
 * Charging *after* a successful compare means our own failures never cost a
 * visitor one of their five, and `used` counts redlines that actually
 * happened — the number worth looking at in the funnel. Withholding the file
 * until the POST lands is what keeps the gate meaningful.
 */

const $ = (id) => document.getElementById(id);

const el = {
  meter: $("meter"),
  pips: $("pips"),
  meterCount: $("meter-count"),
  run: $("run"),
  swap: $("swap"),
  author: $("author"),
  status: $("status"),
  result: $("result"),
  outname: $("outname"),
  outmeta: $("outmeta"),
  chips: $("chips"),
  download: $("download"),
  picker: $("picker"),
  contact: $("contact"),
  contactLabel: $("contact-label"),
  contactHead: $("contact-head"),
  contactBody: $("contact-body"),
};

const slots = {
  original: { file: null, bytes: null, zone: $("zone-original") },
  modified: { file: null, bytes: null, zone: $("zone-modified") },
};

let quota = { used: 0, limit: 5, remaining: 5, paywalled: false };
let objectUrl = null;
let busy = false;

/* ───────────────────────────── worker ───────────────────────────── */

let worker = null;
let seq = 0;
const pending = new Map();

function getWorker() {
  if (worker) return worker;
  worker = new Worker("/redline-worker.js", { type: "module" });
  worker.onmessage = (e) => {
    const { id, ok, ...rest } = e.data;
    const entry = pending.get(id);
    if (!entry) return;
    pending.delete(id);
    ok ? entry.resolve(rest) : entry.reject(new Error(rest.error));
  };
  worker.onerror = (e) => {
    // A worker-level failure never resolves the in-flight promise on its own —
    // reject everything outstanding or the button stays stuck on "Comparing…".
    const err = new Error(e.message || "the redline engine failed to load");
    for (const { reject } of pending.values()) reject(err);
    pending.clear();
    worker = null;
  };
  return worker;
}

function ask(message, transfer = []) {
  const id = ++seq;
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject });
    getWorker().postMessage({ id, ...message }, transfer);
  });
}

/** Fetch and instantiate the ~2 MB wasm while the visitor is still picking
 *  files, so pressing the button doesn't wait on the download. */
let warmed = false;
function warm() {
  if (warmed) return;
  warmed = true;
  ask({ kind: "warm" }).catch(() => {
    // Surfacing this now would be noise; the real attempt reports properly.
    warmed = false;
  });
}

/* ───────────────────────────── quota UI ───────────────────────────── */

function renderQuota() {
  el.meter.hidden = false;
  el.pips.replaceChildren(
    ...Array.from({ length: quota.limit }, (_, i) => {
      const pip = document.createElement("i");
      if (i < quota.used) pip.className = "spent";
      return pip;
    }),
  );
  el.meterCount.textContent = quota.paywalled
    ? "none left"
    : `${quota.remaining} of ${quota.limit} left`;
}

/** Rewrite the contact section as the paywall and send the visitor to it. */
function showPaywall({ scroll = true } = {}) {
  el.contact.classList.add("is-wall");
  el.contactLabel.textContent = "THAT'S THE FIVE";
  el.contactHead.innerHTML =
    'You\'ve used your five free redlines. <span class="mark">Let\'s talk about the rest.</span>';
  el.contactBody.textContent =
    "If you needed five, you probably need five hundred — and that is usually a workflow " +
    "conversation rather than a subscription. Tell me what you are comparing and how often, " +
    "and I will tell you honestly whether Jubarte is the right answer.";
  if (scroll) el.contact.scrollIntoView({ behavior: "smooth", block: "start" });
  updateRunState();
}

async function loadQuota() {
  try {
    const res = await fetch("/api/quota", { credentials: "same-origin" });
    if (!res.ok) throw new Error(`quota ${res.status}`);
    quota = await res.json();
    renderQuota();
    if (quota.paywalled) showPaywall({ scroll: false });
  } catch {
    // The counter is a courtesy; a visitor with two documents in hand should
    // still get to press the button and find out from the server.
    el.meter.hidden = true;
  }
}

/* ───────────────────────────── files ───────────────────────────── */

const DOCX_MIME =
  "application/vnd.openxmlformats-officedocument.wordprocessingml.document";

function humanSize(n) {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(0)} KB`;
  return `${(n / 1024 / 1024).toFixed(1)} MB`;
}

async function accept(slotName, file) {
  if (!file) return;
  if (!file.name.toLowerCase().endsWith(".docx")) {
    setStatus(`${file.name} is not a .docx — Jubarte compares Word documents.`, true);
    return;
  }

  const slot = slots[slotName];
  // Reading a large file takes long enough for the user to drop a replacement;
  // only the newest accept() for this slot may commit its bytes.
  const seq = (slot.seq = (slot.seq || 0) + 1);
  const bytes = new Uint8Array(await file.arrayBuffer());
  if (slot.seq !== seq) return;
  slot.file = file;
  slot.bytes = bytes;
  slot.zone.classList.add("loaded");
  slot.zone.querySelector(".filename").textContent = file.name;
  slot.zone.querySelector(".filemeta").textContent = humanSize(file.size);

  setStatus("");
  warm();
  updateRunState();
}

function updateRunState() {
  const ready = Boolean(slots.original.bytes && slots.modified.bytes);
  el.run.disabled = busy || !ready || quota.paywalled;
  el.run.querySelector(".label").textContent = quota.paywalled
    ? "No free redlines left"
    : busy
      ? "Comparing…"
      : "Create redline";
}

function setStatus(text, isError = false) {
  el.status.textContent = text;
  el.status.classList.toggle("err", isError);
}

/* wire the dropzones */
for (const [name, slot] of Object.entries(slots)) {
  const zone = slot.zone;

  zone.addEventListener("click", () => {
    el.picker.onchange = () => {
      accept(name, el.picker.files?.[0]);
      el.picker.value = "";
    };
    el.picker.click();
  });

  zone.addEventListener("keydown", (e) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      zone.click();
    }
  });

  zone.addEventListener("dragover", (e) => {
    e.preventDefault();
    zone.classList.add("over");
  });
  zone.addEventListener("dragleave", () => zone.classList.remove("over"));
  zone.addEventListener("drop", (e) => {
    e.preventDefault();
    zone.classList.remove("over");
    accept(name, e.dataTransfer?.files?.[0]);
  });
}

// The whole page swallows stray drops — dropping a contract *next to* the box
// and having the browser navigate away to render it is a genuinely bad moment.
for (const evt of ["dragover", "drop"]) {
  window.addEventListener(evt, (e) => e.preventDefault());
}

/** The "or click to browse" hint, rebuilt when a slot is emptied. */
function emphasised() {
  const em = document.createElement("em");
  em.textContent = "or click to browse";
  return em;
}

el.swap.addEventListener("click", () => {
  const a = slots.original;
  const b = slots.modified;
  [a.file, b.file] = [b.file, a.file];
  [a.bytes, b.bytes] = [b.bytes, a.bytes];
  for (const s of [a, b]) {
    s.zone.classList.toggle("loaded", Boolean(s.bytes));
    const name = s.zone.querySelector(".filename");
    if (s.file) {
      // textContent, never innerHTML — a file called `<img onerror=…>.docx` is
      // attacker-supplied markup, and the visitor picked it precisely because
      // they were not inspecting it.
      name.textContent = s.file.name;
    } else {
      name.replaceChildren("Drop a .docx", document.createElement("br"), emphasised());
    }
    s.zone.querySelector(".filemeta").textContent = s.file ? humanSize(s.file.size) : "";
  }
});

/* ───────────────────────────── run ───────────────────────────── */

function renderChips(revisions) {
  const counts = { Inserted: 0, Deleted: 0, Moved: 0, FormatChanged: 0 };
  for (const r of revisions) {
    if (r?.type in counts) counts[r.type]++;
  }
  const labels = [
    ["Inserted", "insertions", "ins"],
    ["Deleted", "deletions", "del"],
    ["Moved", "moves", ""],
    ["FormatChanged", "format changes", ""],
  ];

  el.chips.replaceChildren(
    ...labels
      .filter(([key]) => counts[key] > 0)
      .map(([key, noun, cls]) => {
        const chip = document.createElement("span");
        chip.className = `chip ${cls}`.trim();
        chip.textContent = `${counts[key]} ${noun}`;
        return chip;
      }),
  );

  if (!el.chips.children.length) {
    const chip = document.createElement("span");
    chip.className = "chip";
    chip.textContent = "No differences found";
    el.chips.append(chip);
  }
}

function outputName() {
  const base = slots.modified.file?.name.replace(/\.docx$/i, "") || "document";
  return `${base}-redline.docx`;
}

el.run.addEventListener("click", async () => {
  if (busy || el.run.disabled) return;

  busy = true;
  el.run.classList.add("is-busy");
  updateRunState();
  el.result.hidden = true;
  setStatus("Comparing locally — your documents are not being uploaded…");

  try {
    // Copy the bytes: the buffers are transferred into the worker, and keeping
    // the originals intact lets the visitor re-run without re-picking files.
    const original = slots.original.bytes.slice().buffer;
    const modified = slots.modified.bytes.slice().buffer;

    const { result, revisions } = await ask(
      { kind: "compare", original, modified, author: el.author.value.trim() },
      [original, modified],
    );

    // Compare succeeded — now charge it. The file stays in memory until this
    // resolves, so an exhausted visitor gets the wall rather than the download.
    setStatus("Finishing up…");
    const res = await fetch("/api/redline", {
      method: "POST",
      credentials: "same-origin",
    });
    const body = await res.json().catch(() => ({}));

    if (res.status === 402) {
      quota = { ...quota, ...body };
      renderQuota();
      showPaywall();
      setStatus("");
      return;
    }
    if (!res.ok) throw new Error("could not reach the quota service — please try again");

    quota = { ...quota, ...body };
    renderQuota();

    if (objectUrl) URL.revokeObjectURL(objectUrl);
    objectUrl = URL.createObjectURL(new Blob([result], { type: DOCX_MIME }));

    const name = outputName();
    el.download.href = objectUrl;
    el.download.download = name;
    el.outname.textContent = name;
    el.outmeta.textContent = `${humanSize(result.byteLength)} · opens in Microsoft Word`;
    renderChips(revisions ?? []);
    el.result.hidden = false;
    setStatus("");

    if (quota.paywalled) showPaywall({ scroll: false });
  } catch (err) {
    setStatus(err.message || "Something went wrong creating that redline.", true);
  } finally {
    busy = false;
    el.run.classList.remove("is-busy");
    updateRunState();
  }
});

loadQuota();
