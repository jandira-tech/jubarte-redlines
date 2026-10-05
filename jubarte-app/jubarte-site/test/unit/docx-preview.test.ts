import { describe, expect, it } from "vitest";
import {
  authorFromCore,
  countRevisions,
  PREVIEW_MAX_PARAGRAPHS,
  parsePreview,
  revisionChips,
  unescapeXml,
} from "../../site/static/js/docx-preview.js";

const W = 'xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"';

describe("parsePreview", () => {
  it("classifies runs by their revision wrapper and keeps each author", () => {
    const xml = `<w:document ${W}><w:body>
<w:p><w:r><w:t xml:space="preserve">Pay in </w:t></w:r><w:del w:author="Ana"><w:r><w:delText>45</w:delText></w:r></w:del><w:ins w:author="Ana"><w:r><w:t>30</w:t></w:r></w:ins><w:r><w:t xml:space="preserve"> days.</w:t></w:r></w:p>
<w:p><w:moveFrom w:author="Bo"><w:r><w:t>Moved</w:t></w:r></w:moveFrom></w:p>
<w:p><w:moveTo w:author="Bo"><w:r><w:t>Moved</w:t></w:r></w:moveTo></w:p>
</w:body></w:document>`;
    const { paragraphs, truncated } = parsePreview(xml);
    expect(truncated).toBe(false);
    expect(paragraphs[0].runs).toEqual([
      { kind: "same", text: "Pay in ", author: null },
      { kind: "del", text: "45", author: "Ana" },
      { kind: "ins", text: "30", author: "Ana" },
      { kind: "same", text: " days.", author: null },
    ]);
    expect(paragraphs[1].runs).toEqual([{ kind: "movedel", text: "Moved", author: "Bo" }]);
    expect(paragraphs[2].runs).toEqual([{ kind: "moveins", text: "Moved", author: "Bo" }]);
  });

  it("resolves entities, tabs and breaks, and ignores text outside runs", () => {
    const xml = `<w:document ${W}><w:body><w:p><w:pPr><w:rStyle w:val="x"/></w:pPr><w:r><w:t>A &amp; B&#x2019;s</w:t><w:tab/><w:t>C</w:t><w:br/><w:t>D</w:t></w:r></w:p></w:body></w:document>`;
    expect(parsePreview(xml).paragraphs[0].runs).toEqual([
      { kind: "same", text: "A & B’s\tC\nD", author: null },
    ]);
  });

  it("keeps an empty paragraph as a blank line", () => {
    const xml = `<w:document ${W}><w:body><w:p/><w:p><w:r><w:t>x</w:t></w:r></w:p></w:body></w:document>`;
    expect(parsePreview(xml).paragraphs.map((p) => p.runs.length)).toEqual([0, 1]);
  });

  it("stops at the app's paragraph limit and says so", () => {
    const xml = `<w:document ${W}><w:body>${"<w:p><w:r><w:t>x</w:t></w:r></w:p>".repeat(PREVIEW_MAX_PARAGRAPHS + 5)}</w:body></w:document>`;
    const preview = parsePreview(xml);
    expect(preview.truncated).toBe(true);
    expect(preview.paragraphs).toHaveLength(PREVIEW_MAX_PARAGRAPHS);
  });
});

describe("authorFromCore", () => {
  const core = (inner: string) =>
    `<cp:coreProperties xmlns:cp="c" xmlns:dc="http://purl.org/dc/elements/1.1/">${inner}</cp:coreProperties>`;

  it("prefers dc:creator", () => {
    expect(
      authorFromCore(
        core("<dc:creator> K. Nguyen </dc:creator><cp:lastModifiedBy>Ana</cp:lastModifiedBy>"),
      ),
    ).toBe("K. Nguyen");
  });

  it("falls back to cp:lastModifiedBy when the creator is empty", () => {
    expect(
      authorFromCore(
        core("<dc:creator></dc:creator><cp:lastModifiedBy>Ana &amp; Bo</cp:lastModifiedBy>"),
      ),
    ).toBe("Ana & Bo");
  });

  it("returns an empty string when neither is there", () => {
    expect(authorFromCore(core("<dc:title>x</dc:title>"))).toBe("");
  });
});

describe("helpers", () => {
  it("unescapes XML references", () => {
    expect(unescapeXml("&lt;a&gt; &quot;b&quot; &apos;c&apos; &#65;&#x42;")).toBe(`<a> "b" 'c' AB`);
  });

  it("counts revisions the way the app's chips do", () => {
    const json = JSON.stringify([
      { type: "Inserted" },
      { type: "Inserted" },
      { type: "Deleted" },
      { type: "Moved" },
      { type: "FormatChanged" },
    ]);
    expect(countRevisions(json)).toEqual({ inserted: 2, deleted: 1, moved: 1, format: 1 });
  });
});

describe("moves", () => {
  const para = (inner: string) => `<w:p>${inner}</w:p>`;
  const text = (t: string) => `<w:r><w:t>${t}</w:t></w:r>`;
  const from = (name: string, t: string) =>
    para(
      `<w:moveFromRangeStart w:id="1" w:name="${name}"/><w:moveFrom w:id="2">${text(t)}</w:moveFrom><w:moveFromRangeEnd w:id="1"/>`,
    );
  const to = (name: string, t: string) =>
    para(
      `<w:moveToRangeStart w:id="3" w:name="${name}"/><w:moveTo w:id="4">${text(t)}</w:moveTo><w:moveToRangeEnd w:id="3"/>`,
    );
  const doc = (...ps: string[]) => `<w:document ${W}><w:body>${ps.join("")}</w:body></w:document>`;

  it("shows a move where it left and where it landed, with no label", () => {
    const { paragraphs } = parsePreview(
      doc(
        para(text("2. Fees and Payment.")),
        from("move1", "Late payments accrue interest."),
        para(text("3. Confidentiality.")),
        to("move1", "Late payments accrue interest."),
      ),
    );
    expect(paragraphs[1].runs).toEqual([
      { kind: "movedel", text: "Late payments accrue interest.", author: null },
    ]);
    expect(paragraphs[3].runs).toEqual([
      { kind: "moveins", text: "Late payments accrue interest.", author: null },
    ]);
    for (const p of paragraphs) expect(p).not.toHaveProperty("movedFrom");
  });
});

describe("revisionChips", () => {
  it("counts without signs: the chip's mark says what kind it is", () => {
    const html = revisionChips({ inserted: 3, deleted: 2, moved: 1, format: 0 });
    expect(html).toBe(
      '<span class="tag ins">3 inserted</span><span class="tag del">2 deleted</span><span class="tag mov">1 moved</span>',
    );
    expect(html).not.toMatch(/[+−-]\d/);
  });

  it("leaves a zero count plain: no revision, no revision mark", () => {
    expect(revisionChips({ inserted: 0, deleted: 1, moved: 0, format: 0 })).toBe(
      '<span class="tag">0 inserted</span><span class="tag del">1 deleted</span><span class="tag">0 moved</span>',
    );
  });

  it("shows formatting changes only when the redline has some", () => {
    expect(revisionChips({ inserted: 1, deleted: 0, moved: 0, format: 2 })).toMatch(
      /<span class="tag">2 formatted<\/span>$/,
    );
  });
});
