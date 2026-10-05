import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { render } from "../../site/layout.ts";
import { benchmarkPage } from "../../site/pages/benchmark.ts";
import { useCases } from "../../site/pages/cases.ts";
import { download } from "../../site/pages/download.ts";

/** Each role="table" in the page: its label, and the cell count of every row. */
function tables(html: string) {
  return [...html.matchAll(/role="table" aria-labelledby="([^"]+)"/g)].map((m) => {
    const start = m.index ?? 0;
    const end = html.indexOf("</section>", start);
    const rows = html
      .slice(start, end)
      .split('role="row"')
      .slice(1)
      .map((r) => (r.match(/role="(cell|rowheader|columnheader)"/g) ?? []).length);
    return { label: m[1], rows };
  });
}

// The tables are div grids for layout; screen readers need the table roles to
// read them as rows and columns (audit P3, critique A: "a flat stream").
describe("table roles", () => {
  it.each([
    ["benchmark", render(benchmarkPage({ bench: "convert", id: "x" })), 7],
    ["download", render(download), 5],
  ] as const)("%s: every row has one cell per column, under a labelled table", (_, html, cols) => {
    const found = tables(html);
    expect(found.length).toBeGreaterThan(0);
    for (const t of found) {
      expect(html).toContain(`id="${t.label}"`);
      expect(t.rows.length).toBeGreaterThan(1);
      expect(new Set(t.rows)).toEqual(new Set([cols]));
    }
  });

  it("cases: the header and the rows cases.js writes have six cells each", () => {
    const html = render(useCases);
    const [t] = tables(html);
    expect(t.label).toBe("scores-title");
    expect(t.rows).toEqual([6]);
    expect(html).toContain('<div id="score-rows" role="rowgroup">');
    const js = readFileSync(new URL("../../site/static/js/cases.js", import.meta.url), "utf8");
    const written = [...js.matchAll(/`<div class="t-row score-cols[^\n]*/g)].map(
      (m) => (m[0].match(/role="(cell|rowheader)"/g) ?? []).length,
    );
    expect(written).toEqual([6, 6]);
  });
});
