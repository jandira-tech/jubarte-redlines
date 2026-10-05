import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

describe("colour rules (DESIGN.md)", () => {
  it("keeps the Word colours for revisions and the highlighter for the claim (DESIGN.md)", () => {
    const css = readFileSync(
      new URL("../../site/static/css/site.css", import.meta.url),
      "utf8",
    ).replace(/\/\*[\s\S]*?\*\//g, "");
    const users = (token: RegExp) =>
      [...css.matchAll(/([^{}]+)\{([^{}]*)\}/g)]
        .filter((m) => token.test(m[2]))
        .map((m) => m[1].trim().replace(/\s+/g, " "));
    // Blue, red and green mean inserted, deleted and moved: legends, chips, previews.
    expect(users(/var\(--(ins|del|mov)(-bg)?\)/)).toEqual([
      ".tag.ins",
      ".tag.del",
      ".tag.mov",
      "ins, .ins",
      "del, .del",
      ".mov",
      ".paper .moveins, .paper .movedel",
    ]);
    // The marker: the claim, the selection, a slot a file is over; its tint: featured
    // cells, pills and the "ours" rows. Never a pressed toggle or a notice.
    expect(users(/var\(--hl(-soft)?\)/)).toEqual([
      "::selection",
      ".hl",
      ".featured",
      ".badge-soon",
      ".pill",
      ".slot.over",
      ".t-row.ours",
    ]);
    // A failed engine is not a deletion.
    const cases = readFileSync(new URL("../../site/static/js/cases.js", import.meta.url), "utf8");
    expect(cases).not.toMatch(/class="tag del">failed/);
    // A zero count is not a revision, so its chip takes no Word colour: both
    // pages draw their chips with revisionChips (test/unit/docx-preview.test.ts).
    for (const page of ["demo", "app"]) {
      const js = readFileSync(new URL(`../../site/static/js/${page}.js`, import.meta.url), "utf8");
      expect(js).toMatch(/-chips"\)\.innerHTML = revisionChips\(/);
    }
  });
});
