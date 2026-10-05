import { readFileSync } from "node:fs";
import { beforeAll, describe, expect, it } from "vitest";

// The Mac app's redline, seen two ways (the redline alone, or the original
// beside it) and exported as a PDF. The app has no test runner of its own, so
// its markup and pure functions are pinned here.
const read = (path: string) => readFileSync(new URL(`../../../${path}`, import.meta.url), "utf8");
const html = read("src/index.html");
const app = read("src/app.js");
const VIEWS = new URL("../../../src/views.js", import.meta.url).href;

type Views = typeof import("../../../src/views.js");
let mod: Views;
beforeAll(async () => {
  mod = (await import(/* @vite-ignore */ VIEWS)) as Views;
});

const tag = (id: string) => html.match(new RegExp(`<[a-z]+[^>]*\\bid="${id}"[^>]*>`))?.[0] ?? "";
/** The body of a function in app.js, from its name to its closing brace. */
const body = (name: string) => {
  const from = app.slice(app.indexOf(name));
  return from.slice(0, from.indexOf("\n}\n"));
};
const para = (...runs: [string, string][]) => ({
  runs: runs.map(([kind, text]) => ({ kind, text })),
});

describe("the original beside the redline", () => {
  it("is the redline with its insertions taken out and its deletions kept, unmarked", () => {
    const original = mod.originalParagraphs([
      para(["same", "Term of "], ["del", "twelve"], ["ins", "twenty-four"], ["same", " months."]),
      para(["moveins", "Interest at 1%."]),
      para(["movedel", "Interest at 1%."], ["same", " Paid monthly."]),
      para(),
    ]);
    expect(original).toEqual([
      para(["same", "Term of "], ["same", "twelve"], ["same", " months."]),
      para(["same", "Interest at 1%."], ["same", " Paid monthly."]),
      para(),
    ]);
  });

  it("leaves out a paragraph the modified document added", () => {
    expect(mod.originalParagraphs([para(["ins", "A new clause."])])).toEqual([]);
  });

  it("is chosen with a two-way switch in the result's head", () => {
    expect(tag("view-single")).toContain('aria-pressed="true"');
    expect(tag("view-side")).toContain('aria-pressed="false"');
    expect(tag("paper-original")).toContain('class="paper"');
  });
});

describe("Export PDF", () => {
  it("lays out the redline itself, with the marks Settings chose", () => {
    expect(tag("export-pdf")).toContain("<button");
    const fn = body("async function exportPdf()");
    expect(fn).toContain('invoke("convert_document", {');
    expect(fn).toContain("input: state.result.output_path");
    expect(fn).toMatch(/const marks = currentMarks\(\);[\s\S]*\.\.\.marks,/);
    // Making the PDF is a preview: taking it is what spends a free use.
    expect(fn).not.toContain("noteUse");
    expect(fn).not.toContain("take(");
  });

  it("names the PDF after the redline", () => {
    expect(mod.pdfNameFor("msa-v3_v_msa-v4.docx")).toBe("msa-v3_v_msa-v4.pdf");
    expect(mod.pdfNameFor("Redline (2).DOCX")).toBe("Redline (2).pdf");
  });

  it("leads back to the redline", () => {
    expect(tag("pdf-back")).toContain("<button");
    expect(html).toContain("Back to the redline");
  });

  it("is in the File menu, and a PDF opens and saves as a PDF wherever it was made", () => {
    expect(read("src-tauri/src/menu.rs")).toContain('Action("export-pdf", "Export Redline as PDF"');
    // The button switches to the redline itself, once there is one to export.
    expect(read("src/menu.js")).toContain('"export-pdf": () => press("export-pdf")');
    expect(app).toContain("const isPdf = (r) => /\\.pdf$/i.test(r.output_name);");
  });
});

describe("an export, among the other work", () => {
  it("is work a Finder hand-off waits for, so the wait cannot spin", () => {
    expect(app).toContain("active = exportPdf()");
    // Waiting yields to the event loop even when the job it holds is done.
    expect(body("async function takePending()")).toContain("setTimeout");
  });

  it("is stale when its redline was, and is kept for the way back", () => {
    const fn = body("async function exportPdf()");
    expect(fn).toContain("if (source.stale) r.stale = true;");
    // Back and Export PDF again shows the same PDF: taking it spends nothing twice.
    expect(fn).toContain("source.pdf");
  });

  it("hands focus on to the way back when it lands", () => {
    expect(body("async function exportPdf()")).toContain("landPdf(r)");
    expect(body("function landPdf(")).toContain('$("pdf-back").focus()');
  });

  it("says why it cannot start while other work runs", () => {
    expect(body("function updateCta()")).toContain('setBlocked($("export-pdf")');
  });
});

describe("a preview made at once", () => {
  it("never remakes a result that is still current", () => {
    expect(body("async function previewAtOnce()")).toContain(".stale");
  });
});

describe("a redline", () => {
  it("whose documents changed while it ran is drawn stale", () => {
    expect(body("async function redline()")).toMatch(
      /state\.original !== original \|\| state\.modified !== modified/,
    );
  });
});
