// The footer's light/dark switch. With nothing stored the page follows the
// browser; a click pins the other mode in localStorage, which the inline head
// script (src/theme.ts) restores before the next page paints.

const KEY = "jb-theme";
const root = document.documentElement;
const prefersDark = matchMedia("(prefers-color-scheme: dark)");
/** @type {HTMLMetaElement[]} */
const metas = [...document.querySelectorAll('meta[name="theme-color"]')].map(
  (m) => /** @type {HTMLMetaElement} */ (m),
);
// Each meta's own color, keyed by the mode its media query names.
const colors = Object.fromEntries(
  metas.map((m) => [m.media.includes("dark") ? "dark" : "light", m.content]),
);

const current = () => root.dataset.theme ?? (prefersDark.matches ? "dark" : "light");

/** A pinned mode overrides the media queries, so both metas carry its color. */
function paintChrome() {
  const pinned = root.dataset.theme;
  for (const m of metas) {
    m.content = pinned ? colors[pinned] : colors[m.media.includes("dark") ? "dark" : "light"];
  }
}

for (const button of document.querySelectorAll(".theme-toggle")) {
  button.addEventListener("click", () => {
    const next = current() === "dark" ? "light" : "dark";
    root.dataset.theme = next;
    try {
      localStorage.setItem(KEY, next);
    } catch {
      // Storage refused (private mode): the choice lasts for this page.
    }
    paintChrome();
  });
}
paintChrome();
