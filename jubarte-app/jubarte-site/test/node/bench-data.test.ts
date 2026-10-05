import { describe, expect, it } from "vitest";
import {
  HEADLINES,
  HOME_GROUPS,
  METHOD,
  STATES,
  stateLabel,
  TABLES,
} from "../../site/data/bench.ts";

// check-bench.ts proves the sample tables against the engine's release_info/
// JSONs and the full-corpus tables against RESULTS.md; the home page bars and
// the headline figures restate those tables, so they are proved against them here.
const table = (id: string) => {
  const t = TABLES.find((x) => x.id === id);
  if (!t) throw new Error(`no table ${id}`);
  return t;
};
const row = (id: string, tool: string) => {
  const r = table(id).rows.find((x) => x.tool.replace(" †", "") === tool);
  if (!r) throw new Error(`no ${tool} in ${id}`);
  return r;
};

describe("bench data", () => {
  it("draws the redline home bars from the sample table, versions included", () => {
    const group = HOME_GROUPS.find((g) => !g.title.startsWith("DOCX"));
    if (!group) throw new Error("no redline group");
    expect(group.meta).toContain("600-pair sample");
    for (const bar of group.rows) {
      const r = bar.ours
        ? row("redlines-sample", "jubarte-0.11.2")
        : row("redlines-sample", "docxodus");
      expect(bar.v).toBe(r.median);
      // "jubarte 0.11.2 †" names the pin's version; "Docxodus 12.6.5 (C#)" is the pin.
      const version = bar.name.match(/\d+(?:\.\d+)+/)?.[0];
      if (version && !bar.name.startsWith("Docxodus")) expect(r.pin).toContain(version);
    }
  });

  it("draws the conversion home bars from the conversion sample table", () => {
    const group = HOME_GROUPS.find((g) => g.title.startsWith("DOCX"));
    if (!group) throw new Error("no conversion group");
    expect(group.meta).toContain("600-doc sample");
    for (const bar of group.rows) {
      const r = bar.ours
        ? row("conversion-sample", "jubarte")
        : row("conversion-sample", "soffice");
      expect(bar.v).toBe(r.median);
      const version = bar.name.match(/\d+(?:\.\d+)+/)?.[0];
      if (version) expect(r.pin).toContain(version);
    }
  });

  it("states both sample headlines as their tables have them", () => {
    expect(HEADLINES).toHaveLength(2);
    const conv = HEADLINES.find((h) => h.label.startsWith("DOCX"));
    const red = HEADLINES.find((h) => !h.label.startsWith("DOCX"));
    if (!conv || !red) throw new Error("not both headlines");
    const jub = row("conversion-sample", "jubarte");
    const soffice = row("conversion-sample", "soffice");
    expect(conv.label).toBe("DOCX → PDF · median, 600-doc sample");
    expect(conv.value).toBe(jub.median.toFixed(2));
    expect(conv.vs).toBe(`${soffice.pin} ${soffice.median.toFixed(2)}`);
    expect(conv.sub).toContain("600 documents, state-balanced");
    expect(conv.sub).toContain("0 jubarte failures");
    const rJub = row("redlines-sample", "jubarte-0.11.2");
    const doc = row("redlines-sample", "docxodus");
    expect(red.label).toBe("Redline vs Word compare · median, 600-pair sample");
    expect(red.value).toBe(rJub.median.toFixed(2));
    expect(red.vs).toBe(`Docxodus ${doc.median.toFixed(2)}`);
    // The paired 95% CI of jubarte − Docxodus, as the headline sub carries it.
    expect(red.sub).toMatch(/paired 95% CI of the difference \[\d+\.\d+, \d+\.\d+\]\./);
    expect(red.sub).toContain("600 document pairs, one Word compare each");
  });

  it("names the draw of every sample table it shows", () => {
    const red = table("redlines-sample");
    expect(red.title).toContain("600-pair sample");
    // The meta names the draw: size, one Word compare per pair, date scored.
    expect(red.meta).toContain("600 pairs · one Word compare each");
    expect(red.meta).toContain("2026-10-03");
    // The desc names where every sampled file's path and sha256 is listed.
    expect(red.desc).toContain("release_info");
    expect(red.desc).toContain("sample_redline_0.11.2");
    const conv = table("conversion-sample");
    expect(conv.title).toContain("600-document sample");
    expect(conv.meta).toContain("600 docs");
    expect(conv.meta).toContain("150/150/117/183 by state");
    expect(conv.desc).toContain("release_info");
    expect(conv.desc).toContain("sample_conversion_0.11.2");
  });

  it("adds the release-samples point to the method note, keeping the others", () => {
    expect(METHOD.map((m) => m.t)).toEqual([
      "Oracle: real Word",
      "Intent-to-treat",
      "Paired bootstrap",
      "Release samples",
      "Author-affiliated, same rules",
    ]);
    expect(METHOD.find((m) => m.t === "Release samples")?.d).toContain("the release's two samples");
  });
});

describe("corpus state names", () => {
  it("reads every state of the facts as words, not as the bench's key", () => {
    expect(STATES.map((s) => stateLabel(s.name))).toEqual([
      "clean",
      "tracking, no comments",
      "comments, clean",
      "comments + tracking",
    ]);
  });

  it("leaves a name it does not know as the facts have it", () => {
    expect(stateLabel("a new state")).toBe("a new state");
  });
});
