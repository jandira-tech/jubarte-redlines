import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { compare, parseResults } from "../../scripts/check-bench.ts";

// neurotic_docx_bench's RESULTS.md as the four full-corpus tables of bench.ts
// were last copied from it (2026-10-01 08:32 UTC). The two sample tables are
// proved against the engine's release_info/ instead (check-release-info.test.ts).
const MD = readFileSync(new URL("../fixtures/RESULTS.md", import.meta.url), "utf8");

describe("check-bench", () => {
  it("finds every table of RESULTS.md, rows keyed by header", () => {
    const corpus = parseResults(MD).find((s) => s.heading.startsWith("corpus/word:all:"));
    expect(corpus?.tables[0][0]).toMatchObject({ Tool: "jubarte †", "ITT Median": "80.90" });
    expect(corpus?.tables[1][0].Tool).toBe("jubarte †");
  });

  it("passes bench.ts as copied", () => {
    expect(compare(MD)).toEqual([]);
  });

  it("no longer checks the tables that moved to the release_info proof", () => {
    // corpus-word-all and redlines are not in SECTION, and neither GENERATED
    // nor the per-state block is proved against RESULTS.md any more: the
    // sample tables, GENERATED and bench.states are the release JSONs' business.
    const md = MD.replace(
      "| 77.03 | 80.90 | [80.58, 81.19] |",
      "| 77.03 | 81.20 | [80.88, 81.49] |",
    )
      .replace("| 310 (244) |", "| 311 (244) |")
      .replace("82.05 / 87.54", "82.05 / 88.00")
      .replace("Generated 2026-10-01", "Generated 2026-10-09");
    expect(compare(md)).toEqual([]);
  });

  it("names a changed median, interval, count and failed figure", () => {
    const md = MD.replace(
      "| 61.42 | 65.91 | [65.12, 66.71] |",
      "| 61.42 | 66.25 | [65.12, 66.71] |",
    )
      .replace("| 0 | 97.62 |", "| 0 | 97.70 |")
      .replace("| 166 | 299 |", "| 167 | 299 |")
      .replace("| 2611 | 15 |", "| 2611 | 16 |");
    const diffs = compare(md);
    expect(diffs).toContain(
      "docxide-metrics / jubarte † / jubarte 0.10.1: median 65.91, RESULTS.md 66.25",
    );
    expect(diffs.some((d) => d.includes('note "[65.12, 66.71] · text 97.62"'))).toBe(true);
    expect(
      diffs.some((d) => d.startsWith("accept-reject / docxodus /") && d.includes("RESULTS.md 167")),
    ).toBe(true);
    expect(
      diffs.some(
        (d) => d.startsWith("below-3-pages / soffice /") && d.includes("failed 15, RESULTS.md 16"),
      ),
    ).toBe(true);
  });

  it("names a new engine pin and a row RESULTS.md no longer has", () => {
    const md = MD.replace(
      "| jubarte † | jubarte 0.9.3 | 2026-09-28 |",
      "| jubarte † | jubarte 0.9.4 | 2026-09-28 |",
    ).replace(/^\| 6 \| rdocx .*2026-09-05.*\n/m, "");
    const diffs = compare(md);
    expect(diffs).toContain(
      'below-3-pages / jubarte † / jubarte 0.9.3: pin says 0.9.3; RESULTS.md says "jubarte 0.9.4"',
    );
    expect(diffs).toContain("no-redline / rdocx / rdocx 0.7.0: no such row in RESULTS.md");
  });

  it("names the rows of a section whose table is gone, and checks the rest", () => {
    // The docxide_metrics heading stays; its table lines go.
    const start = MD.indexOf("### docxide_metrics:");
    const end = MD.indexOf("\n### ", start + 1);
    const section = MD.slice(start, end).replace(/^\|.*\n/gm, "");
    const md =
      MD.slice(0, start) + section + MD.slice(end).replace("| 2611 | 15 |", "| 2611 | 16 |");
    const diffs = compare(md);
    expect(
      diffs.some(
        (d) => d.startsWith("docxide-metrics / ") && d.endsWith("no such row in RESULTS.md"),
      ),
    ).toBe(true);
    // The sections after it are still compared.
    expect(
      diffs.some(
        (d) => d.startsWith("below-3-pages / soffice /") && d.includes("failed 15, RESULTS.md 16"),
      ),
    ).toBe(true);
  });
});
