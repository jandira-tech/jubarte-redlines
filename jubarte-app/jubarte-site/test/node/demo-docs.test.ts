import { createRequire } from "node:module";
import { describe, expect, it } from "vitest";
import { DEMO_MODIFIED, DEMO_ORIGINAL, demoDocs } from "../../site/demo-docs.ts";
import {
  countRevisions,
  documentAuthor,
  redlinePreview,
} from "../../site/static/js/docx-preview.js";
import { restampRevisions, wordDate } from "../../site/static/js/revision-date.js";
import { entries, readEntry, readText } from "../../site/static/js/zip.js";

// The engine the site vendors, in its Node build: the App page's walkthrough
// must produce exactly this redline in the browser.
const engine = createRequire(import.meta.url)("jubarte-wasm/slim") as {
  compareDocuments(a: Uint8Array, b: Uint8Array, author: string): Uint8Array;
  getRevisions(docx: Uint8Array): string;
};

describe("the App page's sample pair", () => {
  const docs = demoDocs();
  const original = docs[DEMO_ORIGINAL];
  const modified = docs[DEMO_MODIFIED];

  it("is two complete Word packages, byte-for-byte reproducible", () => {
    for (const doc of [original, modified]) {
      expect([...entries(doc).keys()]).toEqual([
        "[Content_Types].xml",
        "_rels/.rels",
        "word/document.xml",
        "word/_rels/document.xml.rels",
        "word/styles.xml",
        "docProps/core.xml",
        "docProps/app.xml",
      ]);
    }
    expect(demoDocs()[DEMO_MODIFIED]).toEqual(modified);
  });

  it("names K. Nguyen as the modified document's author", async () => {
    expect(await documentAuthor(modified)).toBe("K. Nguyen");
    expect(await documentAuthor(original)).toBe("Acme Legal");
  });

  it("redlines into insertions, deletions and one moved paragraph", async () => {
    const redline = engine.compareDocuments(original, modified, "K. Nguyen");
    expect(countRevisions(engine.getRevisions(redline))).toEqual({
      inserted: 7,
      deleted: 4,
      moved: 2,
      format: 0,
    });
    const preview = await redlinePreview(redline);
    const kinds = preview.paragraphs.flatMap((p) => p.runs.map((r) => r.kind));
    expect(kinds).toContain("movedel");
    expect(kinds).toContain("moveins");
    const authors = new Set(
      preview.paragraphs.flatMap((p) => p.runs.map((r) => r.author)).filter(Boolean),
    );
    expect([...authors]).toEqual(["K. Nguyen"]);
    // The move shows where it left (struck) and where it landed, with no label.
    const at = (kind: string) =>
      preview.paragraphs.findIndex((p) => p.runs.some((r) => r.kind === kind));
    expect(at("movedel")).toBeGreaterThan(-1);
    expect(at("moveins")).toBeGreaterThan(-1);
    expect(at("movedel")).not.toBe(at("moveins"));
    expect(preview.paragraphs.some((p) => "movedFrom" in p)).toBe(false);
  });

  it("dates the browser's redline now, not at the engine's pinned 1970", async () => {
    const pinned = engine.compareDocuments(original, modified, "K. Nguyen");
    const before = (await readText(pinned, "word/document.xml")) ?? "";
    const marks = before.split('w:date="1970-01-01T00:00:00Z"').length - 1;
    expect(marks).toBeGreaterThan(0);

    const date = wordDate(new Date(Date.UTC(2026, 9, 1, 8, 45, 21, 999)));
    expect(date).toBe("2026-10-01T08:45:21Z");
    const redline = await restampRevisions(pinned, date);

    const after = (await readText(redline, "word/document.xml")) ?? "";
    expect(after).not.toContain("1970-01-01");
    expect(after.split(`w:date="${date}"`).length - 1).toBe(marks);
    expect(after.replaceAll(date, "1970-01-01T00:00:00Z")).toBe(before);
    // Same parts, same order; untouched parts are the same bytes.
    expect([...entries(redline).keys()]).toEqual([...entries(pinned).keys()]);
    for (const name of entries(pinned).keys()) {
      if (name === "word/document.xml") continue;
      expect(await readEntry(redline, name), name).toEqual(await readEntry(pinned, name));
    }
    // The engine reads the rewritten package back to the same revisions.
    expect(countRevisions(engine.getRevisions(redline))).toEqual(
      countRevisions(engine.getRevisions(pinned)),
    );
  });

  it("leaves a package without pinned dates byte-for-byte alone", async () => {
    expect(await restampRevisions(original)).toBe(original);
  });
});
