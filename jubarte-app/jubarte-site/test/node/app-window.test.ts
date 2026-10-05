import { readFileSync } from "node:fs";
import { beforeAll, describe, expect, it } from "vitest";

// The Mac app's window (../src): a panel that folds into a strip of icons, a
// preview beside it, and controls that say why they cannot act yet. The app
// has no test runner of its own, so its markup and pure functions are pinned
// here, as app-settings.test.ts does for Settings.
const read = (path: string) => readFileSync(new URL(`../../../${path}`, import.meta.url), "utf8");
const html = read("src/index.html");
const css = read("src/styles.css");
const app = read("src/app.js");
const PANEL = new URL("../../../src/panel.js", import.meta.url).href;

type Panel = typeof import("../../../src/panel.js");
let mod: Panel;
beforeAll(async () => {
  mod = (await import(/* @vite-ignore */ PANEL)) as Panel;
});

/** The element with this id, as written in index.html. */
const tag = (id: string) => html.match(new RegExp(`<[a-z]+[^>]*\\bid="${id}"[^>]*>`))?.[0] ?? "";
const strip = () => /<nav class="rail"[^>]*>([\s\S]*?)<\/nav>/.exec(html)?.[1] ?? "";

describe("the app bar", () => {
  it("names the app and nothing else", () => {
    expect(html).toContain('<span class="appbar-title">JUBARTE</span>');
    expect(html).not.toContain("REDLINE &amp; PDF ENGINE");
  });
});

describe("the panel", () => {
  it("folds into a strip and opens again", () => {
    expect(tag("side")).toContain('class="side"');
    expect(tag("collapse")).toContain('aria-label="Hide panel"');
    expect(tag("rail-expand")).toContain('aria-label="Show panel"');
    expect(html).toContain('<script type="module" src="panel.js"></script>');
  });

  it("keeps its floating whale while open", () => {
    expect(html).toMatch(/<section class="panel"[\s\S]*<svg class="whale"/);
    expect(css).toMatch(/\.whale \{[^}]*animation: float/);
  });

  it("is remembered folded or open, and opens unless it was folded", () => {
    expect(mod.readCollapsed(null)).toBe(false);
    expect(mod.readCollapsed("nonsense")).toBe(false);
    expect(mod.readCollapsed("collapsed")).toBe(true);
    expect(mod.readCollapsed("open")).toBe(false);
  });
});

describe("the strip", () => {
  it("wears the app icon, the very file the Dock shows", () => {
    expect(strip()).toMatch(/<img class="rail-icon" src="icon\.svg"/);
    expect(read("src/icon.svg")).toBe(read("assets/icon.svg"));
  });

  it("offers the panel, the documents, the two modes and the three outputs, each named", () => {
    const labels = [...strip().matchAll(/<button[^>]*aria-label="([^"]+)"/g)].map((m) => m[1]);
    expect(labels).toEqual([
      "Show panel",
      "Add documents",
      "Redline",
      "Convert to PDF",
      "Open",
      "Show in Finder",
      "Save a copy",
      "Free uses",
    ]);
  });

  it("presses the panel's own controls, so both obey the same rules", () => {
    for (const id of [...strip().matchAll(/data-press="([a-z-]+)"/g)].map((m) => m[1]))
      expect(html, id).toContain(`id="${id}"`);
  });
});

describe("Convert's formats", () => {
  it("offer PDF, and PNG pages as coming soon", () => {
    expect(tag("fmt-pdf")).toContain('aria-pressed="true"');
    expect(tag("fmt-png")).toContain('aria-disabled="true"');
    expect(tag("fmt-png")).toContain('data-tip="Soon"');
  });

  it("leave the tracked-change marks to Settings", () => {
    expect(html).not.toContain('id="revisions"');
  });
});

describe("a control that cannot act yet", () => {
  it("stays pressable, so it can say why", () => {
    for (const id of ["run", "open-word", "open-pdf", "reveal", "save-copy"]) {
      expect(tag(id), id).not.toMatch(/\sdisabled[\s>]/);
      expect(tag(id), id).toContain('aria-disabled="true"');
    }
  });

  it("nudges a redline with one document toward the missing one", () => {
    const one = { original: { name: "a.docx" }, modified: null, doc: null };
    expect(mod.runBlocker("redline", one)).toEqual({
      text: "Add the modified document to make a redline.",
      slot: "modified",
    });
    expect(
      mod.runBlocker("redline", { ...one, original: null, modified: { name: "b.docx" } }),
    ).toEqual({
      text: "Add the original document to make a redline.",
      slot: "original",
    });
    expect(mod.runBlocker("redline", { original: null, modified: null, doc: null })?.text).toBe(
      "Add the two documents to compare.",
    );
    expect(mod.runBlocker("redline", { ...one, modified: { name: "b.docx" } })).toBeNull();
  });

  it("never switches modes for the user", () => {
    const one = { original: null, modified: null, doc: { name: "a.docx" } };
    expect(mod.runBlocker("convert", one)).toBeNull();
    expect(mod.runBlocker("convert", { ...one, doc: null })).toEqual({
      text: "Add a document to convert.",
      slot: "convert",
    });
    expect(app).not.toMatch(/setMode\("convert"\)[^;]*length === 1/);
  });

  it("says what a result is waiting for", () => {
    expect(mod.outputBlocker("redline", { docs: 1, result: false })).toBe("Needs two documents.");
    expect(mod.outputBlocker("redline", { docs: 2, result: false })).toBe(
      "Create the redline first.",
    );
    expect(mod.outputBlocker("convert", { docs: 0, result: false })).toBe("Add a document first.");
    expect(mod.outputBlocker("convert", { docs: 1, result: false })).toBe("Convert it first.");
    expect(mod.outputBlocker("convert", { docs: 1, result: true })).toBeNull();
  });
});

describe("a nudge", () => {
  it("is read out from a live region that is always there", () => {
    expect(tag("nudge-say")).toContain('role="status"');
    expect(tag("nudge")).toContain('aria-hidden="true"');
  });

  it("points at the strip when the panel is folded", () => {
    expect(read("src/panel.js")).toContain("getClientRects().length");
  });

  it("is never pressed into a run by Return on a link or in the paywall", () => {
    expect(app).toContain('closest("button, a, .dropzone, dialog, [role=dialog]")');
  });
});

describe("a PDF", () => {
  it("opens in the Mac's own app for PDFs, shows in Finder and saves, as a redline does", () => {
    expect(tag("open-pdf")).toContain('data-mode="convert"');
    expect(app).toContain('$("open-pdf").addEventListener("click", openResult);');
    expect(app).toMatch(/\[\{ name: "PDF document", extensions: \["pdf"\] \}\]/);
  });
});

describe("motion", () => {
  it("stops the floating and drifting whales for anyone who asked for less", () => {
    const block = css.slice(css.indexOf("@media (prefers-reduced-motion: reduce) {"));
    expect(block.length).toBeGreaterThan(0);
    for (const sel of [".whale", ".watermark", ".side"]) expect(block, sel).toContain(sel);
  });
});
