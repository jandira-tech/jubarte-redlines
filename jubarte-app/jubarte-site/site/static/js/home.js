// The home page's benchmark strip: a random published case, page one as
// Word, jubarte and LibreOffice rendered it, with the real scores. It moves
// on every few seconds while the tab is visible (not under reduced motion).

import { fixtureUrl, stateLabel } from "./case-filters.js";
import { $ } from "./common.js";
import { tabKey } from "./tabs.js";

/**
 * @typedef {{ score: number, failed: boolean, pages: number | null }} Side
 * @typedef {{ id: string, state: string, pages: number, jubarte: Side, soffice: Side, h: Record<string, number> }} Pick
 */

const PERIOD = 9000;
const still = matchMedia("(prefers-reduced-motion: reduce)").matches;
const seen = { n: 0, ahead: 0 };
/** The dataset revision home.json was built from; it pins the page images. */
let revision = "";

/** @param {Side} s @param {number} pages */
const pp = (s, pages) =>
  s.failed ? "· failed" : `· ${s.pages} pp${s.pages !== pages ? " ≠ Word" : ""}`;

function sheet(/** @type {string} */ engine, /** @type {Pick} */ c) {
  const box = /** @type {HTMLElement} */ (
    document.querySelector(`.sheet[data-engine="${engine}"]`)
  );
  const h = c.h[engine];
  if (!h || (engine !== "word" && c[/** @type {"jubarte" | "soffice"} */ (engine)].failed)) {
    box.innerHTML = '<div class="pending"><span>no output · scores 0</span></div>';
    return Promise.resolve();
  }
  return new Promise((resolve) => {
    const img = new Image();
    img.alt = `Page 1 as ${engine === "word" ? "Word" : engine === "soffice" ? "LibreOffice" : "jubarte"} rendered it`;
    img.decoding = "async";
    // Page one on its own (scripts/site_fixtures.py FIRST), not the whole strip.
    img.src = fixtureUrl("convert", c.id, `${engine}-p1.webp`, revision);
    const pg = document.createElement("div");
    pg.className = "pg";
    pg.append(img);
    const done = () => {
      box.style.aspectRatio = `640/${h}`;
      box.replaceChildren(pg);
      resolve(undefined);
    };
    // decode(), not onload: a large WebP fires load seconds before it can paint,
    // and a blank white page beside Word's reads as a failed render.
    img.decode().then(done, done);
  });
}

async function showCase(/** @type {Pick} */ c) {
  $("fx-path").textContent = `Case: ${c.id.replace(/-[0-9a-f]{10}$/, "").replace(/-/g, " ")}`;
  $("fx-state").textContent = stateLabel(c.state);
  $("fx-pages").textContent = `${c.pages} pp in Word`;
  $("fx-jub").textContent = c.jubarte.failed ? "0" : c.jubarte.score.toFixed(2);
  $("fx-jub-pp").textContent = pp(c.jubarte, c.pages);
  $("fx-sof").textContent = c.soffice.failed ? "0" : c.soffice.score.toFixed(2);
  $("fx-sof-pp").textContent = pp(c.soffice, c.pages);
  /** @type {HTMLAnchorElement} */ ($("fx-case")).href = `/use-cases#convert/${c.id}`;
  await Promise.all(["word", "jubarte", "soffice"].map((e) => sheet(e, c)));
  seen.n++;
  const jub = c.jubarte.failed ? 0 : c.jubarte.score;
  const sof = c.soffice.failed ? 0 : c.soffice.score;
  if (jub > sof) seen.ahead++;
  $("fx-session").textContent =
    `${seen.n} case${seen.n === 1 ? "" : "s"} this visit · jubarte ahead in ${seen.ahead} of ${seen.n}`;
}

async function main() {
  const res = await fetch("/data/home.json", { cache: "no-cache" });
  if (!res.ok) throw new Error(String(res.status));
  /** @type {{ revision: string, cases: Pick[] }} */
  const home = await res.json();
  revision = home.revision;
  const { cases } = home;
  let order = cases.map((_, i) => i).sort(() => Math.random() - 0.5);
  let i = 0;
  // The refresh mark turns while the next case loads, then finishes its turn
  // and rests upright.
  const refresh = $("feed-refresh");
  const next = async () => {
    if (i >= order.length) {
      order = order.sort(() => Math.random() - 0.5);
      i = 0;
    }
    refresh.classList.add("turning");
    try {
      await showCase(cases[order[i++]]);
    } finally {
      refresh.addEventListener("animationiteration", () => refresh.classList.remove("turning"), {
        once: true,
      });
    }
  };
  await next();
  const hold = /** @type {HTMLButtonElement} */ ($("feed-pause"));
  if (still) {
    hold.hidden = true;
    refresh.hidden = true;
    return;
  }
  // 2.2.2: the rotation can be stopped, besides pausing under the pointer or focus.
  let held = false;
  hold.addEventListener("click", () => {
    held = !held;
    hold.setAttribute("aria-pressed", String(held));
    hold.textContent = held ? "play" : "pause";
    refresh.classList.toggle("held", held);
  });
  const feed = $("feed");
  let paused = false;
  feed.addEventListener("pointerenter", () => {
    paused = true;
  });
  feed.addEventListener("pointerleave", () => {
    paused = false;
  });
  feed.addEventListener("focusin", () => {
    paused = true;
  });
  feed.addEventListener("focusout", () => {
    paused = false;
  });
  const tick = async () => {
    if (!document.hidden && !paused && !held) await next();
    setTimeout(tick, PERIOD);
  };
  setTimeout(tick, PERIOD);
}

main().catch(() => {
  $("feed-refresh").hidden = true;
  $("fx-path").textContent = "The benchmark strip could not load.";
  $("fx-session").textContent = "—";
});

/* ---------- install tabs ---------- */

const installTabs = /** @type {HTMLButtonElement[]} */ ([
  ...document.querySelectorAll(".installer-tabs [role=tab]"),
]);

/**
 * Select install tab `i` and show its panel; the others leave the Tab order
 * (roving tabindex), so arrow keys, not Tab, move between them.
 * @param {number} i
 * @param {boolean} [focus] move focus too, as the arrow keys do
 */
function showInstall(i, focus = false) {
  installTabs.forEach((tab, k) => {
    tab.setAttribute("aria-selected", String(k === i));
    tab.tabIndex = k === i ? 0 : -1;
    $(/** @type {string} */ (tab.getAttribute("aria-controls"))).hidden = k !== i;
  });
  if (focus) installTabs[i].focus();
}
// Click selects; the arrows wrap around the tabs, Home and End jump to the ends.
installTabs.forEach((tab, i) => {
  tab.addEventListener("click", () => showInstall(i));
  tab.addEventListener("keydown", (e) => {
    const to = tabKey(e.key, i, installTabs.length);
    if (to === null) return;
    e.preventDefault();
    showInstall(to, true);
  });
});
