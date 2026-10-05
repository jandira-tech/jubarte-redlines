import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import {
  APP_IN_REVIEW,
  APP_NEXT,
  APP_RELEASE,
  APP_STORE,
  ARCHIVES,
  COUNSEL_EMAIL,
  SUPPORT_EMAIL,
} from "../../site/data/release.ts";
import { type Page, render } from "../../site/layout.ts";
import { pro } from "../../site/pages/app.ts";
import { benchmarkPage } from "../../site/pages/benchmark.ts";
import { useCases, useCasesEmbed } from "../../site/pages/cases.ts";
import { contact } from "../../site/pages/contact.ts";
import { demo } from "../../site/pages/demo.ts";
import { download, pendingRows } from "../../site/pages/download.ts";
import { homePage } from "../../site/pages/home.ts";
import { notFound, privacy, terms } from "../../site/pages/legal.ts";
import { live } from "../../site/pages/live.ts";
import { WASM_SIZE } from "../../site/wasm-size.ts";
import { THEME_KEY, THEME_SCRIPT, THEME_SCRIPT_SHA256 } from "../../src/theme.ts";

const PAGES: Page[] = [
  homePage({ jubarte: "jubarte 0.10.1", soffice: "LibreOffice 26.8.0.3" }),
  live,
  demo,
  benchmarkPage({ bench: "convert", id: "clean-010cd893de" }),
  useCases,
  useCasesEmbed,
  pro,
  contact,
  download,
  privacy,
  terms,
  notFound,
];

// Files the build writes besides the pages.
const FILES = new Set([
  "/favicon.svg",
  "/favicon-32.png",
  "/apple-touch-icon.png",
  "/og.png",
  "/static/css/site.css",
  "/static/fonts/manrope-latin-wght-normal.woff2",
  "/static/fonts/jetbrains-mono-latin-wght-normal.woff2",
]);

const internalLinks = (html: string) =>
  [...html.matchAll(/(?:href|src)="(\/[^"]*)"/g)].map((m) => m[1].replace(/[?#].*$/, ""));

describe("pages", () => {
  const paths = new Set(PAGES.map((p) => p.path));

  it.each(PAGES.map((p) => [p.path, p] as const))("%s renders a complete document", (_, page) => {
    const html = render(page);
    expect(html.startsWith("<!doctype html>")).toBe(true);
    expect(html).toContain(`<link rel="canonical" href="https://jubarte.pro${page.path}">`);
    // The CSP allows one inline script, the theme restore, by its hash; every
    // other script is an external module.
    expect(html.split(`<script>${THEME_SCRIPT}</script>`).length - 1).toBe(1);
    const others = html.replace(`<script>${THEME_SCRIPT}</script>`, "");
    for (const tag of others.match(/<script\b[^>]*>/g) ?? []) {
      expect(tag).toMatch(/type="module" src="\/static\/js\/[\w-]+\.js"/);
    }
    expect(html).not.toMatch(/\son[a-z]+="/);
    // Element ids are unique.
    const ids = [...html.matchAll(/\sid="([^"]+)"/g)].map((m) => m[1]);
    expect(ids.filter((id, i) => ids.indexOf(id) !== i)).toEqual([]);
  });

  it("links only to pages and files that exist", () => {
    for (const page of PAGES) {
      for (const link of internalLinks(render(page))) {
        const known =
          paths.has(link) ||
          FILES.has(link) ||
          link.startsWith("/static/js/") ||
          link.startsWith("/use-cases/embed");
        expect(known, `${page.path} links to ${link}`).toBe(true);
      }
    }
  });

  it("keeps Live a coming-soon page", () => {
    const html = render(live);
    expect(html).toContain("Coming soon");
    expect(html).not.toContain('<script type="module" src="/static/js/live');
  });

  it("names the Live link 'Live, coming soon', not 'Livesoon'", () => {
    const html = render(live);
    const names = [...html.matchAll(/<a href="\/live"[^>]*>(.*?)<\/a>/g)].map((m) =>
      m[1].replace(/<[^>]+>/g, ""),
    );
    expect(names.length).toBeGreaterThan(0);
    for (const name of names) expect(name).toBe("Live, coming soon");
  });

  it("labels the Convert options in words, with the command line as a secondary echo", () => {
    const html = render(demo);
    expect(html).toContain("<span>Smaller file</span></label>");
    expect(html).toContain('for="dpi">Resolution (dpi)</label>');
    expect(html).toContain("The same job on the command line");
  });

  it("heads the benchmark columns with the figures they hold: failures count as 0", () => {
    const html = render(benchmarkPage({ bench: "convert", id: "clean-010cd893de" })).replaceAll(
      ' role="columnheader"',
      "",
    );
    expect(html).toContain(
      '<span>Median · 95% CI</span><span class="r">Median (fail = 0)</span><span class="r">Mean (fail = 0)</span>',
    );
    expect(html).not.toContain('<span class="r">Median</span>');
  });

  it("states the App Store price and the subscription truthfully", () => {
    const html = render(download);
    expect(html).toContain("$99.99 on the store today");
    expect(html).toContain("$99.99 a year");
  });

  it("marks only the CLI targets the release has no archive for as not built", () => {
    const windows = { target: "Windows x86_64", file: "jubarte-9.9.9-windows-x86_64.zip", size: 1 };
    expect(pendingRows([...ARCHIVES, windows])).not.toContain("Windows x86_64");
    const pending = pendingRows(ARCHIVES.filter((a) => a.target !== "Linux aarch64"));
    expect(pending).toContain("Linux aarch64");
    const html = render(download);
    for (const a of ARCHIVES) expect(pendingRows(ARCHIVES)).not.toContain(a.target);
    expect(html.split("not built yet").length - 1).toBe(5 - ARCHIVES.length);
  });

  it("quotes the wasm sizes measured from the shipped package", () => {
    const html = render(demo);
    expect(WASM_SIZE.slim.raw).toMatch(/^\d+\.\d MB$/);
    expect(html).toContain(
      `The ${WASM_SIZE.slim.raw} compare build (about ${WASM_SIZE.slim.wire} over`,
    );
    expect(html).toContain(`data-wire="${WASM_SIZE.full.wire}"`);
  });

  it("draws the App page's own head, jumps, whale and notes, not the Demo's", () => {
    const html = render(pro);
    expect(html).toContain('class="page-head head-app"');
    expect(html).toContain('class="seg mono-seg jump-seg"');
    expect(html).toContain('class="whale-float-sm" style="width:130px;height:auto"');
    expect(html).toContain('class="cells app-notes"');
    expect(html).toContain('class="tags chips" id="a-chips"');
    // The App design has no page watermark; its window's whale sits under the result pane.
    expect(html).not.toContain('class="watermark');
  });

  it("shows the App window's Convert to PDF mode as the submitted release's, beside Redline", () => {
    const html = render(pro);
    expect(html).toContain('data-mode-tab="redline" aria-pressed="true"');
    expect(html).toContain('data-mode-tab="convert" aria-pressed="false"');
    // One slot and the tracked-change choice, hidden until the tab is picked.
    expect(html).toMatch(
      /<div class="slots single" data-mode="convert" hidden>\s*<button[^>]*id="a-conv"/,
    );
    expect(html).toContain('<select id="a-revisions"><option value="conventional">');
    expect(html).toContain('<option value="word">As Word prints</option>');
    expect(html).toContain('class="page-grid app-pages" id="a-pages"');
    // While Apple reviews the release that adds Convert, the page names it.
    if (APP_IN_REVIEW) {
      expect(html).toContain(
        `The Convert tab is the PDF mode of ${APP_RELEASE.version}, in Apple’s review now.`,
      );
      expect(html).toContain(
        `<p class="kicker">${APP_RELEASE.version} · in review</p><p class="note-title">Convert to PDF</p>`,
      );
    } else {
      expect(html).toContain("The Convert tab is its PDF mode.");
    }
    expect(html).toContain(`data-wire="${WASM_SIZE.full.wire}"`);
  });

  it("lets the Cases shortcuts be switched off, and keeps hover text readable", () => {
    const html = render(useCases);
    // WCAG 2.1.4: a control turns the single-key shortcuts off.
    expect(html).toContain('id="toggle-keys" aria-pressed="true"');
    const css = readFileSync(new URL("../../site/static/css/site.css", import.meta.url), "utf8");
    // The dark chip keeps a dark hover: the light hover fill left --on-deep text on near-white.
    expect(css).toMatch(/\.chip-btn:hover:not\(:disabled\):not\(\.dark\) \{/);
    expect(css).toMatch(
      /\.chip-btn\.dark:hover:not\(:disabled\) \{\s*background: var\(--primary\);/,
    );
    // A selected filter's count takes the selected text colour, not --muted on --deep.
    expect(css).toMatch(
      /\.side-btn\[aria-pressed="true"\]:not\(\.engine\) \.mono \{\s*color: inherit;/,
    );
  });

  it("gives every page a way past the nav, and Cases a heading and a short live region", () => {
    for (const page of PAGES.filter((p) => !p.bare)) {
      const html = render(page);
      // 2.4.1: a skip link whose target is a <main> that exists on the page.
      const target = /<a class="skip-link" href="#([^"]+)">/.exec(html)?.[1];
      expect(target, `${page.path} has a skip link`).toBeTruthy();
      expect(html, `${page.path} skip target`).toMatch(
        new RegExp(`<main\\b[^>]*\\bid="${target}"`),
      );
    }
    const html = render(useCases);
    expect(html).toContain('<h1 class="sr-only">');
    // The whole stage is no longer a live region; one short status line is.
    expect(html).not.toContain('class="stage" aria-live');
    expect(html).toContain('id="case-live" role="status"');
    const css = readFileSync(new URL("../../site/static/css/site.css", import.meta.url), "utf8");
    expect(css).toMatch(/::placeholder \{\s*color: var\(--disabled\);/);
    expect(css).toMatch(/scroll-padding-top: \d+px;/);
    // Fields keep a 2px ring on focus, not a one-pixel border change alone.
    expect(css).toMatch(/\.field:focus-within \{[^}]*outline: 2px solid/);
  });

  it("keeps the Demo's errors honest: named, persistent for Convert, clearable", () => {
    const html = render(demo);
    expect(html).toContain('id="clear-slots"');
    expect(html).toContain('id="c-status"');
    // Toasts stay in the accessibility tree: faded, not hidden.
    expect(html).toContain('class="toast off" id="toast" role="status"></div>');
    const js = readFileSync(new URL("../../site/static/js/demo.js", import.meta.url), "utf8");
    expect(js).toContain("function zipComplete");
    expect(js).toContain("function clearError");
  });

  it("opens Home with a marked-up clause, and keeps the developer material below the scoreboard", () => {
    const html = render(PAGES[0]);
    const hero = html.slice(html.indexOf('<section class="hero">'), html.indexOf('id="feed"'));
    // The redline a lawyer receives: deletion before insertion, as Word orders them.
    expect(hero).toContain('class="hero-redline"');
    expect(hero).toMatch(/<del>New York<\/del><ins>Delaware<\/ins>/);
    // No CLI command or installer in the lawyer's first screens.
    const devAt = html.indexOf('id="developers"');
    expect(devAt).toBeGreaterThan(html.indexOf("Not a claim. A scoreboard."));
    expect(html.indexOf('<span class="tok">accept</span>')).toBeGreaterThan(devAt);
    expect(html.indexOf('role="tablist" aria-label="Install jubarte"')).toBeGreaterThan(devAt);
    // The strip's rotation can be stopped (2.2.2).
    expect(html).toContain('id="feed-pause" aria-pressed="false"');
  });

  it("shows page one of any case, Convert or Compare, from its own image, not the whole strip", () => {
    const js = readFileSync(new URL("../../site/static/js/cases.js", import.meta.url), "utf8");
    expect(js).toMatch(/\$\{first \? "-p1" : ""\}\.webp/);
    expect(js).not.toContain("FIRST_PAGE");
    expect(js).toContain("fixture(engine, page === 1)");
    // The fixture script cuts page one from every strip, not only Home's three engines.
    const py = readFileSync(new URL("../../scripts/site_fixtures.py", import.meta.url), "utf8");
    expect(py).not.toContain("HOME_ENGINES");
    expect(py).toContain("str(target), True))");
  });

  it("leads Cases with the pages on a phone, the case's name and links after them", () => {
    const html = render(useCases);
    // The toolbar groups flatten into one row on a phone, views before navigation.
    expect(html).toContain('class="row gap-8 tb-nav"');
    expect(html).toContain('class="row gap-8 tb-views"');
    expect(html).not.toContain("toolbar-in m-col");
    // A phone has no keyboard: each key hint is a span the phone layout hides.
    expect(html).toContain('filters<span class="key-hint"> · s</span>');
    expect(html).toContain('more pages<span class="key-hint"> · m</span>');
    const css = readFileSync(new URL("../../site/static/css/site.css", import.meta.url), "utf8");
    const phone = css.slice(css.indexOf("/* Cases on a phone"));
    expect(phone).toMatch(/\.stage-head \{\s*display: contents;/);
    expect(phone).toMatch(/\.stage-id \{\s*order: 1;/);
    expect(phone).toMatch(/\.key-hint \{\s*display: none;/);
    // Zoom and the page turner share one row under the overlay switch.
    expect(html).toContain('class="row gap-10 zoom-row"');
    expect(phone).toMatch(/\.zoom-row \{[^}]*flex-wrap: nowrap;/);
    expect(phone).toMatch(/\.stage-ctl > \.seg \{\s*flex: 1 0 100%;/);
  });

  it("lets the chips be the legend: each count wears its revision's mark", () => {
    const css = readFileSync(new URL("../../site/static/css/site.css", import.meta.url), "utf8");
    expect(css).toMatch(/\.tag\.ins \{[^}]*text-decoration: underline;/);
    expect(css).toMatch(/\.tag\.del \{[^}]*text-decoration: line-through;/);
    expect(css).toMatch(/\.tag\.mov \{[^}]*text-decoration: underline double;/);
    expect(css).not.toMatch(/(^|\n)\.legend \{/);
    for (const page of [demo, pro]) expect(render(page)).not.toContain('class="legend"');
    expect(render(pro)).toContain(
      "insertions underlined in blue, deletions struck in red, and a move in green, double-struck where it left and double-underlined where it landed",
    );
  });

  it("marks insertions and deletions once and moves twice, so a move never reads as an insertion", () => {
    const css = readFileSync(new URL("../../site/static/css/site.css", import.meta.url), "utf8");
    expect(css).toMatch(/\nins,\n\.ins \{[^}]*text-decoration: underline;/);
    expect(css).toMatch(/\ndel,\n\.del \{[^}]*text-decoration: line-through;/);
    expect(css).toMatch(/\.paper \.moveins \{[^}]*text-decoration: underline double;/);
    expect(css).toMatch(/\.paper \.movedel \{\s*text-decoration: line-through double;\s*\}/);
    expect(css).not.toContain("move-note");
  });

  it("keeps the contact form off the app window's inline .field row", () => {
    // .field strips its input's border and pads checkboxes into wide boxes.
    const html = render(contact);
    expect(html).not.toMatch(/class="field"/);
    expect(html.match(/class="form-field"/g)?.length).toBe(6);
  });

  it("states today's one-time purchase before the subscription terms that follow it", () => {
    const html = render(terms);
    const once = html.indexOf(`Jubarte ${APP_STORE.version}`);
    expect(once).toBeGreaterThan(-1);
    expect(html).toContain(`one-time purchase of ${APP_STORE.price}`);
    expect(html).toContain(`${APP_NEXT.freeUses} free uses, each a redline or a PDF conversion`);
    expect(once).toBeLessThan(html.indexOf("Payment is charged"));
  });

  it("counts a PDF against the app's free uses wherever the offer is stated", () => {
    // The app meters redlines and conversions from one pool (src/paywall.js).
    for (const page of [pro, download, terms]) {
      const html = render(page);
      expect(html).not.toMatch(/free redlines|free for \d+ redlines/);
      expect(html).toMatch(new RegExp(`${APP_NEXT.freeUses} (free )?uses`));
    }
  });

  it("asks for help on both app pages, naming the subscription the Pro Version", () => {
    for (const page of [pro, download]) {
      const html = render(page);
      expect(html).toContain(`then the Pro Version at ${APP_NEXT.yearly}`);
      expect(html).toContain("Want to help? The Pro Version is how.");
      expect(html).not.toContain("Jubarte Pro at");
    }
  });

  it("names the home page's fixture engines from the fixtures, not from the engine release", () => {
    // The fixtures are re-rendered on their own schedule: after an engine
    // release they still show the version the benchmark ran.
    const html = render(homePage({ jubarte: "jubarte 0.9.9", soffice: "LibreOffice 1.2.3" }));
    expect(html).toContain("<dt>jubarte 0.9.9</dt>");
    expect(html).toContain("<dt>LibreOffice 1.2.3</dt>");
  });

  it("pairs every install tab with its panel and opens only the first", () => {
    // home.js switches tabs through aria-controls; a tab without its panel, or
    // two open panels, would leave the installer showing the wrong command.
    const html = render(homePage({ jubarte: "jubarte 0.10.1", soffice: "LibreOffice 26.8.0.3" }));
    const tabs = [
      ...html.matchAll(
        /<button type="button" role="tab" id="inst-tab-(\w+)" aria-controls="inst-(\w+)" aria-selected="(true|false)"( tabindex="-1")?>/g,
      ),
    ];
    const panels = [
      ...html.matchAll(
        /<div class="installer-panel" role="tabpanel" id="inst-(\w+)" aria-labelledby="inst-tab-(\w+)" tabindex="0"( hidden)?>/g,
      ),
    ];
    expect(tabs.map((t) => t[1])).toEqual(["cli", "rust", "python", "node"]);
    expect(panels.map((p) => p[1])).toEqual(tabs.map((t) => t[2]));
    for (const [, tab, ctl] of tabs) expect(ctl).toBe(tab);
    for (const [, panel, label] of panels) expect(label).toBe(panel);
    expect(tabs.map((t) => t[3] === "true")).toEqual([true, false, false, false]);
    // Roving tabindex: only the selected tab is in the Tab order.
    expect(tabs.map((t) => t[4] === undefined)).toEqual([true, false, false, false]);
    expect(panels.map((p) => p[3] === undefined)).toEqual([true, false, false, false]);
  });

  it("says what the verification service does with Apple's subscription notifications", () => {
    // jubarte.pro/notifications is verify-worker's route (verify-worker/wrangler.jsonc):
    // each notification updates the stored subscription record.
    const html = render(privacy);
    expect(html).not.toMatch(/do not currently process App Store Server Notifications/);
    expect(html).toContain("App Store Server Notifications");
    expect(html).toMatch(
      /renews, lapses, enters a billing grace period, is cancelled or is\s+refunded/,
    );
    expect(html).toContain("only to update that same record");
    expect(html).toContain(`Jubarte ${APP_STORE.version}`);
  });

  it("addresses the drafted email from release.ts, not from a second copy in the script", () => {
    const html = render(contact);
    expect(html).toContain(`data-support="${SUPPORT_EMAIL}"`);
    expect(html).toContain(`data-counsel="${COUNSEL_EMAIL}"`);
    const script = readFileSync(
      new URL("../../site/static/js/contact.js", import.meta.url),
      "utf8",
    );
    expect(script).not.toMatch(/[\w.-]+@[\w-]+\.[a-z]+/);
  });

  it("restores the stored mode before the stylesheet paints, under the CSP's hash", () => {
    const html = render(homePage({ jubarte: "jubarte 0.10.1", soffice: "LibreOffice 26.8.0.3" }));
    expect(html.indexOf(`<script>${THEME_SCRIPT}</script>`)).toBeLessThan(
      html.indexOf('<link rel="stylesheet"'),
    );
    expect(createHash("sha256").update(THEME_SCRIPT, "utf8").digest("base64")).toBe(
      THEME_SCRIPT_SHA256,
    );
    const toggle = readFileSync(new URL("../../site/static/js/theme.js", import.meta.url), "utf8");
    expect(toggle).toContain(`const KEY = "${THEME_KEY}";`);
  });

  it("puts the light/dark switch at the bottom of every page that has a footer", () => {
    for (const page of PAGES) {
      const html = render(page);
      const footer = html.slice(html.indexOf('<footer class="site-footer">'));
      if (page.bare) {
        expect(html).not.toContain("theme-toggle");
        continue;
      }
      expect(footer, page.path).toContain('<button type="button" class="theme-toggle">');
      expect(footer).toContain('<span class="to-dark">Dark mode</span>');
      expect(footer).toContain('<span class="to-light">Light mode</span>');
      expect(html).toContain('<script type="module" src="/static/js/theme.js"></script>');
    }
  });

  it("names every color in the stylesheet in OKLCH, with a light and a dark value", () => {
    const css = readFileSync(new URL("../../site/static/css/site.css", import.meta.url), "utf8");
    expect(css).not.toMatch(/#[0-9a-f]{3,8}\b|\brgba?\(|\bhsla?\(/i);
    const root = css.slice(css.indexOf(":root {"), css.indexOf("\n}\n", css.indexOf(":root {")));
    // Every palette token is a light-dark() pair (the max-contrast muted and the
    // brand-color cta reuse one); only the page and the radii are mode-free.
    for (const [, name, value] of root.matchAll(/^\s+--([\w-]+):\s*([^;]+);/gm)) {
      if (/^(sans|mono|serif|max|radius|radius-page|page|muted|cta)$/.test(name)) continue;
      expect(value, `--${name}`).toMatch(/^light-dark\(oklch\(.+\), oklch\(.+\)\)$/);
    }
  });
});
