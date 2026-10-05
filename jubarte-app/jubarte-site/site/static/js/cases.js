// The case viewer: every published benchmark case, Word's pages beside each
// engine's, from /data/cases-<bench>.json and the WebP page strips under
// /fixtures. File links point at the same files on Hugging Face, pinned to
// the dataset revision the site was built from.

import {
  applyFilters,
  DEFAULT_FILTERS,
  fixtureUrl,
  pageNav,
  parseHash,
  stateLabel,
} from "./case-filters.js";
import { $, typing } from "./common.js";

const embed = $("viewer").dataset.embed === "true";
const BENCHES = /** @type {const} */ (["convert", "redline"]);
const DEFAULT_SHOWN = {
  convert: { jubarte: true, soffice: true, docxide: false },
  redline: { jubarte: true, docxodus: true, superdoc: false },
};
const OVERLAYS = ["side", "under", "difference", "multiply"];

/** @typedef {import("./case-filters.js").Case} Case */
/** @typedef {{ repo: string, revision: string, hub: string, resolve: string, engines: Record<string, Record<string, string>>, bench: string, cases: Case[] }} Data */

/** @type {Record<string, Promise<Data>>} */
const cache = {};
function data(/** @type {string} */ bench) {
  cache[bench] ??= fetch(`/data/cases-${bench}.json`, { cache: "no-cache" }).then((r) => {
    if (!r.ok) throw new Error(`cases-${bench}.json: ${r.status}`);
    return r.json();
  });
  return cache[bench];
}

const st = {
  bench: /** @type {"convert" | "redline"} */ ("redline"),
  /** @type {Data | null} */ data: null,
  /** @type {Case[]} */ list: [],
  /** @type {Case | null} */ cur: null,
  /** @type {string[]} */ history: [],
  hi: -1,
  // On a phone the filters would fill the first screen: they start closed.
  side: !embed && !matchMedia("(max-width: 760px)").matches,
  scores: !embed,
  overlay: "side",
  /** Zoom, 0–100 on the slider (50 = 100%). */
  zoom: 50,
  /** The engine layer's opacity per overlay mode, in percent. */
  opacity: /** @type {Record<string, number>} */ ({ under: 50, difference: 100, multiply: 100 }),
  page: 1,
  more: false,
  filters: { ...DEFAULT_FILTERS },
  shown: structuredClone(DEFAULT_SHOWN),
};

/** Bumped per stage paint; a pending swap only lands if it is still the latest. */
let swaps = 0;
/** The bench/case the stage last drew. */
let shownKey = "";

/**
 * The strip of a case's pages, or just its first page when that alone is shown:
 * every strip, in both benches, has page one on its own (scripts/site_fixtures.py).
 */
const fixture = (/** @type {string} */ engine, first = false) =>
  fixtureUrl(
    st.bench,
    st.cur?.id ?? "",
    `${engine}${first ? "-p1" : ""}.webp`,
    st.data?.revision ?? "",
  );
const fmt = (/** @type {number | null | undefined} */ v) => (v == null ? "—" : v.toFixed(2));
const el = (/** @type {string} */ html) => {
  const t = document.createElement("template");
  t.innerHTML = html.trim();
  return /** @type {HTMLElement} */ (t.content.firstElementChild);
};
const escAttr = (/** @type {string} */ s) =>
  s.replace(
    /[&<>"']/g,
    (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c] ?? c,
  );

/** One page of one engine's strip, cropped with CSS: the strip is 640 px wide. */
function pageImage(/** @type {string} */ engine, /** @type {number} */ page, cls = "") {
  const r = st.cur?.renders[engine];
  const off = r?.offsets[page - 1];
  if (!r || !off) return null;
  const [y, h] = off;
  const box = el(`<div class="pg ${cls}" style="aspect-ratio:640/${h}"></div>`);
  const img = document.createElement("img");
  // Page one is about a twelfth of the strip: do not fetch the strip to show it.
  img.src = fixture(engine, page === 1);
  // The cell's caption names the engine; the underlay of an overlay is a duplicate.
  img.alt = cls === "under" ? "" : `Page ${page}`;
  img.decoding = "async";
  img.style.top = `${(-y / h) * 100}%`;
  box.append(img);
  return box;
}

function tagFor(/** @type {{ score: number, failed: boolean }} */ e) {
  if (e.failed) return "FAILED · scores 0";
  if (e.score >= 100) return "PERFECT";
  if (e.score >= 90) return "≥ 90";
  if (e.score < 50) return "< 50";
  return "SCORED";
}

function paintToolbar() {
  for (const b of BENCHES) $(`bench-${b}`).setAttribute("aria-pressed", String(st.bench === b));
  $("toggle-side").setAttribute("aria-pressed", String(st.side));
  $("toggle-scores").setAttribute("aria-pressed", String(st.scores));
  $("side").hidden = !st.side;
  $("cases-main").classList.toggle("with-side", st.side);
  $("scores").hidden = !st.scores;
  $("pos").textContent = st.history.length
    ? `${st.hi + 1} / ${st.history.length} visited · ${st.list.length.toLocaleString("en-US")} match`
    : "";
}

function paintSide() {
  const d = st.data;
  if (!d) return;
  const counts = new Map();
  for (const c of d.cases) counts.set(c.state, (counts.get(c.state) ?? 0) + 1);
  const groups = [
    ["all", "all groups", d.cases.length],
    ...[...counts].map(([k, n]) => [k, stateLabel(k), n]),
  ];
  $("groups").replaceChildren(
    ...groups.map(([k, label, n]) => {
      const b = el(
        `<button type="button" class="side-btn" aria-pressed="${st.filters.group === k}"><span>${label}</span><span class="mono">${Number(n).toLocaleString("en-US")}</span></button>`,
      );
      b.addEventListener("click", () => setFilter({ group: String(k) }));
      return b;
    }),
  );
  for (const b of document.querySelectorAll("[data-bucket]")) {
    b.setAttribute(
      "aria-pressed",
      String(/** @type {HTMLElement} */ (b).dataset.bucket === st.filters.bucket),
    );
  }
  /** @type {HTMLInputElement} */ ($("max-score")).value = String(st.filters.maxScore);
  $("max-score-v").textContent = String(st.filters.maxScore);
  /** @type {HTMLInputElement} */ ($("differ")).checked = st.filters.differ;
  const names = d.engines[st.bench];
  const shown = st.shown[st.bench];
  $("engines").replaceChildren(
    ...Object.keys(shown).map((k) => {
      const b = el(
        `<button type="button" class="side-btn engine" aria-pressed="${shown[k]}"><span class="sq"></span><span>${names[k]}</span></button>`,
      );
      b.addEventListener("click", () => {
        shown[k] = !shown[k];
        paintSide();
        paintStage();
      });
      return b;
    }),
  );
}

function paintStage() {
  const c = st.cur;
  const d = st.data;
  $("no-match").hidden = !!c || !d;
  if (!c || !d) {
    swaps++; // drop a swap still waiting on its images
    shownKey = "";
    $("pages").replaceChildren();
    $("score-rows").replaceChildren();
    return;
  }
  const names = d.engines[st.bench];
  const word = c.renders.word;
  const wordPages = word?.pages ?? 0;
  const drawn = word?.offsets.length ?? 0;
  const nav = pageNav(st.page, drawn);
  st.page = nav.page;
  /** @type {HTMLButtonElement} */ ($("page-prev")).disabled = !nav.prev;
  /** @type {HTMLButtonElement} */ ($("page-next")).disabled = !nav.next;
  const label = st.bench === "redline" ? "redlines vs Word compare" : "docx_to_pdf vs Word export";
  $("case-kicker").textContent =
    `${label} · case ${String(st.list.indexOf(c) + 1).padStart(3, "0")} of ${st.list.length.toLocaleString("en-US")}`;
  $("case-live").textContent =
    `Case ${st.list.indexOf(c) + 1} of ${st.list.length}, page ${st.page} of ${wordPages}`;
  $("case-path").textContent = st.bench === "redline" ? c.stem : `${c.stem}.docx`;
  $("case-state").textContent =
    st.bench === "redline" ? `${stateLabel(c.state)} · pair` : stateLabel(c.state);
  $("case-pages").textContent = `${wordPages} pp in Word`;
  const link = `#${st.bench}/${c.id}`;
  /** @type {HTMLAnchorElement} */ ($("case-link")).href = link;
  /** @type {HTMLAnchorElement} */ ($("case-folder")).href = `${st.data?.hub}/${st.bench}/${c.id}`;
  const full = document.getElementById("open-full");
  if (full) /** @type {HTMLAnchorElement} */ (full).href = `/use-cases${link}`;

  const overlayOn = st.overlay !== "side";
  for (const b of document.querySelectorAll("[data-ov]")) {
    b.setAttribute(
      "aria-pressed",
      String(/** @type {HTMLElement} */ (b).dataset.ov === st.overlay),
    );
  }
  paintSlider();
  $("page-label").textContent =
    `p. ${st.page} of ${wordPages}${drawn < wordPages ? ` · ${drawn} drawn` : ""}`;

  const cells = [];
  const oracle = el(
    `<figure class="case-cell"><figcaption class="cell-head"><span>${st.bench === "redline" ? "Word compare · oracle" : "Word · oracle"}</span><span class="muted">${wordPages} pp</span></figcaption></figure>`,
  );
  oracle.append(
    pageImage("word", st.page) ?? el('<div class="pg missing"><span>no page</span></div>'),
  );
  oracle.append(el('<p class="cell-score"><strong>100</strong><span>REFERENCE</span></p>'));
  cells.push(oracle);

  for (const [k, on] of Object.entries(st.shown[st.bench])) {
    if (!on) continue;
    const e = c.engines[k];
    if (!e) continue;
    const r = c.renders[k];
    const pagesDiffer = !e.failed && r && r.pages !== wordPages;
    const cell = el(
      `<figure class="case-cell${k === "jubarte" ? " ours" : ""}${e.score >= 100 ? " perfect" : ""}"><figcaption class="cell-head"><span title="${escAttr(names[k])}">${names[k]}</span><span class="${pagesDiffer ? "warn" : "muted"}">${e.failed || !r ? "—" : `${r.pages} pp`}</span></figcaption></figure>`,
    );
    const stack = el('<div class="stack"></div>');
    const mine = e.failed ? null : pageImage(k, st.page, "over");
    if (overlayOn && mine) {
      const under = pageImage("word", st.page, "under");
      if (under) stack.append(under);
      mine.style.mixBlendMode = st.overlay === "under" ? "normal" : st.overlay;
      mine.style.opacity = String(st.opacity[st.overlay] / 100);
    }
    stack.append(
      mine ??
        el(
          `<div class="pg missing"><span>${e.failed ? "no output — scores 0" : `no page ${st.page}`}</span></div>`,
        ),
    );
    cell.append(stack);
    const pageScore = e.page_scores?.[st.page - 1];
    cell.append(
      el(
        `<p class="cell-score"><strong>${e.failed ? "0" : fmt(e.score)}</strong><span>${tagFor(e)}</span>${pageScore != null ? `<span class="muted">p. ${st.page}: ${fmt(pageScore)}</span>` : ""}</p>`,
      ),
    );
    cells.push(cell);
  }
  // Same case (mode, page or engine change): keep the old sheets until the
  // new ones decode. A new case swaps at once; its old pages would mislead.
  const key = `${st.bench}/${c.id}`;
  if (key === shownKey) swapWhenDecoded($("pages"), cells);
  else {
    swaps++;
    $("pages").replaceChildren(...cells);
  }
  shownKey = key;
  paintMore();
  paintScores();
}

/**
 * The slider is zoom side by side and the engine layer's opacity in the
 * overlay modes. Applied in place, so dragging it redraws nothing.
 */
function paintSlider() {
  const overlayOn = st.overlay !== "side";
  const value = overlayOn ? st.opacity[st.overlay] : st.zoom;
  const slider = /** @type {HTMLInputElement} */ ($("slider"));
  slider.value = String(value);
  slider.setAttribute(
    "aria-valuetext",
    overlayOn ? `${value}% opacity` : `${Math.round(60 + st.zoom * 0.8)}% zoom`,
  );
  $("slider-label").textContent = overlayOn ? "opacity" : "zoom";
  $("slider-v").textContent = overlayOn ? `${value}%` : `${Math.round(60 + st.zoom * 0.8)}%`;
  $("pages").style.setProperty("--zoom", String((60 + st.zoom * 0.8) / 100));
  if (!overlayOn) return;
  for (const over of $("pages").querySelectorAll(".stack > .pg.over")) {
    /** @type {HTMLElement} */ (over).style.opacity = String(value / 100);
  }
}

/**
 * Replace a container's children once the new pages have decoded (or after a
 * short wait), so switching modes or cases never flashes blank sheets.
 */
function swapWhenDecoded(/** @type {HTMLElement} */ box, /** @type {HTMLElement[]} */ nodes) {
  const me = ++swaps;
  const imgs = nodes.flatMap((n) => [...n.querySelectorAll("img")]);
  const ready = Promise.all(imgs.map((i) => i.decode().catch(() => undefined)));
  const wait = new Promise((r) => setTimeout(r, 600));
  Promise.race([ready, wait]).then(() => {
    if (me === swaps) box.replaceChildren(...nodes);
  });
}

function paintMore() {
  $("more").hidden = !st.more;
  /** @type {Text} */ ($("toggle-more").firstChild).data = st.more ? "hide pages" : "more pages";
  $("toggle-more").setAttribute("aria-expanded", String(st.more));
  if (!st.more || !st.cur) return;
  const c = st.cur;
  const drawn = c.renders.word?.offsets.length ?? 0;
  const jub = c.engines.jubarte;
  const thumbs = [];
  for (let p = 1; p <= drawn; p++) {
    const score = jub?.page_scores?.[p - 1];
    const b = el(
      `<button type="button" class="more-thumb${p === st.page ? " current" : ""}${score != null && score < 90 ? " low" : ""}" aria-label="Page ${p}"><span class="pair"></span><span class="mono">p. ${p}${score != null ? ` · ${score.toFixed(0)}` : ""}</span></button>`,
    );
    const pair = /** @type {HTMLElement} */ (b.querySelector(".pair"));
    pair.append(pageImage("word", p) ?? el('<div class="pg missing"></div>'));
    pair.append(pageImage("jubarte", p) ?? el('<div class="pg missing"></div>'));
    b.addEventListener("click", () => {
      st.page = p;
      paintStage();
    });
    thumbs.push(b);
  }
  $("more-strip").replaceChildren(...thumbs);
}

function paintScores() {
  const c = st.cur;
  const d = st.data;
  if (!c || !d) return;
  // The data is first-party, and the CSP blocks inline handlers; escaping is
  // still what keeps a stray quote in a file name from breaking the row.
  const names = d.engines[st.bench];
  const file = (/** @type {string} */ n, label = n.split(".").pop() ?? n) =>
    c.files.includes(n)
      ? `<a href="${escAttr(`${d.resolve}/${st.bench}/${c.id}/${n}`)}" target="_blank" rel="noopener">${escAttr(label)}</a>`
      : "";
  const sources =
    st.bench === "redline"
      ? `${file("original.docx", "original.docx")} · ${file("modified.docx", "modified.docx")}`
      : file("source.docx", "source.docx");
  const rows = [
    `<div class="t-row score-cols oracle" role="row"><span class="strong" role="rowheader">${escAttr(names.word)} <span class="mono muted">· ${sources}</span></span><span class="mono r" role="cell">${c.renders.word?.pages ?? "—"} pp</span><span class="mono r" role="cell">100.00</span><span class="mono r" role="cell">100.00</span><span class="mono r" role="cell">100.00</span><span class="mono r files" role="cell">${file("word.pdf")} ${file("word.docx")}</span></div>`,
  ];
  for (const k of Object.keys(st.shown[st.bench])) {
    const e = c.engines[k];
    if (!e) continue;
    const r = c.renders[k];
    const differ = r && c.renders.word && r.pages !== c.renders.word.pages;
    rows.push(
      `<div class="t-row score-cols${k === "jubarte" ? " ours" : ""}" role="row"><span class="strong" role="rowheader">${escAttr(names[k])}${e.failed ? ' <span class="tag state">failed</span>' : ""}</span><span class="mono r${differ ? " warn" : ""}" role="cell">${r ? `${r.pages} pp` : "—"}</span><span class="mono r strong" role="cell">${e.failed ? "0.00" : fmt(e.score)}</span><span class="mono r" role="cell">${fmt(e.jaccard)}</span><span class="mono r" role="cell">${fmt(e.text_boundary)}</span><span class="mono r files" role="cell">${file(`${k}.pdf`)} ${file(`${k}.docx`)}</span></div>`,
    );
  }
  $("score-rows").innerHTML = rows.join("");
  $("cases-foot").dataset.revision = d.revision;
}

function paint() {
  paintToolbar();
  paintSide();
  paintStage();
}

/* ---------- navigation ---------- */

function show(/** @type {Case | null} */ c, push = true) {
  st.cur = c;
  st.page = 1;
  if (c && push) {
    st.history = [...st.history.slice(0, st.hi + 1), c.id].slice(-200);
    st.hi = st.history.length - 1;
  }
  if (c) history.replaceState(null, "", `#${st.bench}/${c.id}`);
  paint();
}

function random() {
  if (!st.list.length) return show(null, false);
  const others = st.list.length > 1 ? st.list.filter((c) => c !== st.cur) : st.list;
  show(others[Math.floor(Math.random() * others.length)]);
}

function byId(/** @type {string} */ id) {
  return st.data?.cases.find((c) => c.id === id) ?? null;
}

function back() {
  if (st.hi > 0) {
    st.hi--;
    show(byId(st.history[st.hi]), false);
  }
}
function forward() {
  if (st.hi < st.history.length - 1) {
    st.hi++;
    show(byId(st.history[st.hi]), false);
  } else random();
}

function refilter() {
  if (!st.data) return;
  st.list = applyFilters(st.data.cases, st.filters, st.shown[st.bench]);
  if (!st.cur || !st.list.includes(st.cur)) random();
  else paint();
}

function setFilter(/** @type {Partial<typeof DEFAULT_FILTERS>} */ patch) {
  Object.assign(st.filters, patch);
  refilter();
}

async function setBench(
  /** @type {"convert" | "redline"} */ bench,
  /** @type {string | null} */ id = null,
) {
  st.bench = bench;
  st.history = [];
  st.hi = -1;
  st.filters = { ...DEFAULT_FILTERS };
  $("case-kicker").textContent = "loading cases…";
  try {
    st.data = await data(bench);
  } catch (err) {
    $("case-kicker").textContent =
      `Could not load the cases: ${err instanceof Error ? err.message : err}`;
    return;
  }
  st.list = applyFilters(st.data.cases, st.filters, st.shown[bench]);
  const wanted = id ? byId(id) : null;
  if (wanted) show(wanted);
  else random();
}

/* ---------- wiring ---------- */

for (const b of BENCHES)
  $(`bench-${b}`).addEventListener("click", () => st.bench !== b && setBench(b));
$("prev").addEventListener("click", back);
$("next").addEventListener("click", forward);
$("random").addEventListener("click", random);
$("toggle-side").addEventListener("click", () => {
  st.side = !st.side;
  paintToolbar();
});
$("toggle-scores").addEventListener("click", () => {
  st.scores = !st.scores;
  paintToolbar();
});
const clear = () => setFilter({ ...DEFAULT_FILTERS });
$("clear").addEventListener("click", clear);
$("clear-2").addEventListener("click", clear);
for (const b of document.querySelectorAll("[data-bucket]")) {
  b.addEventListener("click", () =>
    setFilter({ bucket: /** @type {HTMLElement} */ (b).dataset.bucket ?? "any" }),
  );
}
$("max-score").addEventListener("input", (e) => {
  st.filters.maxScore = Number(/** @type {HTMLInputElement} */ (e.target).value);
  $("max-score-v").textContent = String(st.filters.maxScore);
});
$("max-score").addEventListener("change", () => refilter());
$("differ").addEventListener("change", (e) =>
  setFilter({ differ: /** @type {HTMLInputElement} */ (e.target).checked }),
);
for (const b of document.querySelectorAll("[data-ov]")) {
  b.addEventListener("click", () => {
    st.overlay = /** @type {HTMLElement} */ (b).dataset.ov ?? "side";
    paintStage();
  });
}
$("slider").addEventListener("input", (e) => {
  const v = Number(/** @type {HTMLInputElement} */ (e.target).value);
  if (st.overlay === "side") st.zoom = v;
  else st.opacity[st.overlay] = v;
  paintSlider();
});
const turn = (/** @type {number} */ by) => {
  st.page += by;
  paintStage();
};
$("page-prev").addEventListener("click", () => turn(-1));
$("page-next").addEventListener("click", () => turn(1));
$("toggle-more").addEventListener("click", () => {
  st.more = !st.more;
  paintMore();
});

// WCAG 2.1.4: the single-key shortcuts can be turned off (and stay off).
const KEYS = "jb-keys";
let keysOn = true;
try {
  keysOn = localStorage.getItem(KEYS) !== "off";
} catch {
  /* storage blocked: shortcuts stay on for this visit */
}
function paintKeys() {
  $("toggle-keys").setAttribute("aria-pressed", String(keysOn));
  document.body.classList.toggle("keys-off", !keysOn);
}
$("toggle-keys").addEventListener("click", () => {
  keysOn = !keysOn;
  try {
    localStorage.setItem(KEYS, keysOn ? "on" : "off");
  } catch {
    /* storage blocked */
  }
  paintKeys();
});
paintKeys();

document.addEventListener("keydown", (e) => {
  if (!keysOn || typing(e) || e.metaKey || e.ctrlKey || e.altKey) return;
  const k = e.key;
  if (k === "r") random();
  else if (k === "s") {
    st.side = !st.side;
    paintToolbar();
  } else if (k === "t") {
    st.scores = !st.scores;
    paintToolbar();
  } else if (k === "o") {
    st.overlay = OVERLAYS[(OVERLAYS.indexOf(st.overlay) + 1) % OVERLAYS.length];
    paintStage();
  } else if (k === "m") {
    st.more = !st.more;
    paintMore();
  } else if (k === "[") turn(-1);
  else if (k === "]") turn(1);
  // ← → only: ↑ ↓ stay the page's own scrolling.
  else if (k === "ArrowLeft") {
    e.preventDefault();
    back();
  } else if (k === "ArrowRight") {
    e.preventDefault();
    forward();
  }
});

window.addEventListener("hashchange", () => {
  const h = parseHash(location.hash);
  if (!h) return;
  if (h.bench !== st.bench) setBench(h.bench, h.id);
  else if (h.id !== st.cur?.id) {
    // A permalink to a case this set no longer holds lands on a random one,
    // as it does on first load, not on the "no case matches" stage.
    const c = byId(h.id);
    if (c) show(c);
    else random();
  }
});

const initial = parseHash(location.hash);
// Lawyers first (PRODUCT.md): a bare /use-cases opens on the redlines.
setBench(initial?.bench ?? "redline", initial?.id ?? null);
