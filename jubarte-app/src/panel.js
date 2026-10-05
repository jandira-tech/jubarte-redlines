// The window's left side: the panel, and the strip of icons it folds into.
// Controls that cannot act yet stay pressable (aria-disabled) and say why in
// a nudge; the strip presses the panel's own controls, so both obey the same
// rules. The pure functions are exported for the tests
// (jubarte-site/test/node/app-window.test.ts).

export const STORAGE_KEY = "jb-panel";

/**
 * Whether the panel was left folded. Open unless it was folded.
 * @param {string | null | undefined} raw
 */
export const readCollapsed = (raw) => raw === "collapsed";

/**
 * What the run button waits for, or null when it can run. The mode is the
 * user's: one document never switches a redline to a conversion.
 * @param {"redline" | "convert"} mode
 * @param {{ original: unknown, modified: unknown, doc: unknown }} docs
 * @returns {{ text: string, slot: "original" | "modified" | "convert" | "both" } | null}
 */
export function runBlocker(mode, { original, modified, doc }) {
  if (mode === "convert") return doc ? null : { text: "Add a document to convert.", slot: "convert" };
  if (original && modified) return null;
  if (original) return { text: "Add the modified document to make a redline.", slot: "modified" };
  if (modified) return { text: "Add the original document to make a redline.", slot: "original" };
  return { text: "Add the two documents to compare.", slot: "both" };
}

/**
 * What Open, Show in Finder and Save a copy wait for, or null once there is
 * a result to take.
 * @param {"redline" | "convert"} mode
 * @param {{ docs: number, result: boolean }} state
 */
export function outputBlocker(mode, { docs, result }) {
  if (result) return null;
  if (mode === "convert") return docs ? "Convert it first." : "Add a document first.";
  return docs < 2 ? "Needs two documents." : "Create the redline first.";
}

/** The CTA's short hint for a blocker's slot. */
export const RUN_HINT = {
  both: "add both documents",
  original: "add the original",
  modified: "add the modified",
  convert: "add a document",
};

/* ---------- the window ---------- */

function init() {
  const $ = (id) => document.getElementById(id);
  const side = $("side");
  const tip = $("tip");
  const nudgeEl = $("nudge");
  const reduce = window.matchMedia?.("(prefers-reduced-motion: reduce)");
  /** The control the tooltip names, if one is shown. */
  let tipFor = null;
  function hideTip() {
    tip.hidden = true;
    tipFor = null;
  }

  /* folding */

  const setCollapsed = (collapsed, { focus = false } = {}) => {
    side.dataset.collapsed = String(collapsed);
    // The hidden half leaves the tab order and the accessibility tree.
    $("panel").inert = collapsed;
    side.querySelector(".rail").inert = !collapsed;
    localStorage.setItem(STORAGE_KEY, collapsed ? "collapsed" : "open");
    hideTip();
    if (focus) (collapsed ? $("rail-expand") : $("collapse")).focus();
  };
  const toggle = () => setCollapsed(side.dataset.collapsed !== "true", { focus: true });
  setCollapsed(readCollapsed(localStorage.getItem(STORAGE_KEY)));
  $("collapse").addEventListener("click", () => setCollapsed(true, { focus: true }));
  $("rail-expand").addEventListener("click", () => setCollapsed(false, { focus: true }));
  // ⌃⌘S (View › Show or Hide Panel, menu.rs) calls toggle, as a sidebar's
  // shortcut does in every Mac app.

  /* the strip presses the panel's controls */

  const press = (id) => $(id)?.click();
  const mode = () => (document.body.dataset.mode === "convert" ? "convert" : "redline");
  for (const btn of side.querySelectorAll(".rail [data-press]")) {
    btn.addEventListener("click", () => press(btn.dataset.press));
  }
  // Open is the one the panel shows: Word for a redline, the PDF app for a PDF.
  const opener = () => ($("open-pdf").hidden ? "open-word" : "open-pdf");
  $("rail-open").addEventListener("click", () => press(opener()));
  // Add documents: the first empty slot of the mode, else the last one.
  $("rail-add").addEventListener("click", () => {
    if (mode() === "convert") return press("zone-convert");
    press($("zone-original").classList.contains("filled") ? "zone-modified" : "zone-original");
  });

  // Each strip button mirrors the control it presses: its state and its why.
  const mirror = () => {
    for (const btn of side.querySelectorAll(".rail [data-press^='mode-']")) {
      btn.setAttribute("aria-pressed", $(btn.dataset.press).getAttribute("aria-selected") ?? "false");
    }
    const pairs = [
      [$("rail-open"), $(opener())],
      ...[...side.querySelectorAll(".rail [data-press='reveal'], .rail [data-press='save-copy']")].map((b) => [
        b,
        $(b.dataset.press),
      ]),
    ];
    for (const [btn, target] of pairs) {
      btn.setAttribute("aria-disabled", target.getAttribute("aria-disabled") ?? "false");
      if (target.dataset.why) btn.dataset.why = target.dataset.why;
      else delete btn.dataset.why;
    }
    const free = $("free-left");
    const short = $("free-short");
    short.hidden = free.hidden;
    short.textContent = free.dataset.short ?? "";
    short.dataset.tip = free.textContent;
  };
  const watched = ["mode-redline", "mode-convert", "open-word", "open-pdf", "reveal", "save-copy", "free-left"];
  const observer = new MutationObserver(mirror);
  for (const id of watched) {
    observer.observe($(id), {
      attributes: true,
      attributeFilter: ["aria-selected", "aria-disabled", "data-why", "hidden", "data-short"],
      childList: true,
      characterData: true,
      subtree: true,
    });
  }
  new MutationObserver(mirror).observe(document.body, { attributes: true, attributeFilter: ["data-mode"] });
  mirror();

  /* tooltips: the strip's names, and a disabled control's reason */

  /** Places `el` beside `anchor`: right of the strip, else above the control. */
  const place = (el, anchor) => {
    const r = anchor.getBoundingClientRect();
    el.hidden = false;
    const w = el.offsetWidth;
    const h = el.offsetHeight;
    if (anchor.closest(".rail")) {
      el.style.left = `${r.right + 10}px`;
      el.style.top = `${r.top + r.height / 2 - h / 2}px`;
      el.dataset.side = "right";
    } else {
      const left = Math.min(Math.max(8, r.left + r.width / 2 - w / 2), window.innerWidth - w - 8);
      const above = r.top - h - 8;
      el.style.left = `${left}px`;
      el.style.top = `${above > 8 ? above : r.bottom + 8}px`;
      el.dataset.side = above > 8 ? "above" : "below";
    }
  };
  const label = (el) => {
    const name = el.dataset.tip ?? el.getAttribute("aria-label") ?? "";
    const why = el.getAttribute("aria-disabled") === "true" && el.dataset.why;
    return why && el.closest(".rail") ? `${name} — ${why}` : name;
  };
  const showTip = (el) => {
    if (!nudgeEl.hidden) return;
    const text = label(el);
    if (!text) return;
    tipFor = el;
    tip.textContent = text;
    place(tip, el);
  };
  const tipped = (target) => target.closest?.(".rail button, [data-tip]");
  document.addEventListener("pointerover", (e) => {
    const el = tipped(e.target);
    if (el && el !== tipFor) showTip(el);
    else if (!el && tipFor) hideTip();
  });
  document.addEventListener("focusin", (e) => {
    const el = tipped(e.target);
    if (el && e.target.matches(":focus-visible")) showTip(el);
  });
  document.addEventListener("focusout", hideTip);
  window.addEventListener("blur", hideTip);

  /* nudges: a press on a control that cannot act yet says why */

  let nudgeTimer = 0;
  const say = $("nudge-say");
  // A control out of sight (the panel folded, the other mode's pane) is
  // pointed at through what stands for it: its strip button, else the strip,
  // else the run button.
  const seen = (el) => (el?.getClientRects().length ? el : null);
  const standIn = (anchor) =>
    seen(anchor) ??
    seen(side.querySelector(`.rail [data-press="${anchor.id}"]`)) ??
    (anchor.id.startsWith("open-") ? seen($("rail-open")) : null) ??
    seen($("rail-expand")) ??
    $("run");
  const nudge = (anchor, text) => {
    hideTip();
    clearTimeout(nudgeTimer);
    nudgeEl.textContent = text;
    place(nudgeEl, standIn(anchor));
    // Emptied first, so the same reason twice is read out twice.
    say.textContent = "";
    setTimeout(() => {
      say.textContent = text;
    }, 50);
    nudgeEl.classList.remove("in");
    void nudgeEl.offsetWidth;
    nudgeEl.classList.add("in");
    nudgeTimer = setTimeout(() => {
      nudgeEl.hidden = true;
    }, 3200);
  };
  // The zones a run is waiting for show themselves.
  const want = (slot) => {
    const ids = { both: ["zone-original", "zone-modified"], original: ["zone-original"], modified: ["zone-modified"], convert: ["zone-convert"] }[slot] ?? [];
    for (const id of ids) {
      const zone = $(id);
      zone.classList.remove("wanted");
      void zone.offsetWidth;
      zone.classList.add("wanted");
      setTimeout(() => zone.classList.remove("wanted"), reduce?.matches ? 2400 : 1600);
    }
  };
  // Capture phase: the control's own handler never sees a press it cannot act on.
  document.addEventListener(
    "click",
    (e) => {
      const el = e.target.closest?.('[aria-disabled="true"]');
      if (!el) return;
      e.preventDefault();
      e.stopImmediatePropagation();
      const why = el.dataset.why;
      if (why) nudge(el, why);
      const slot = (el.dataset.press ? $(el.dataset.press) : el)?.dataset.slot;
      if (slot) want(slot);
    },
    true,
  );
  window.jubartePanel = { toggle, runBlocker, outputBlocker, RUN_HINT };
  // app.js drew the window before this module loaded: draw it again with the
  // reasons in place.
  window.jubarteRender?.();
}

if (typeof document !== "undefined" && document.getElementById("side")) init();
