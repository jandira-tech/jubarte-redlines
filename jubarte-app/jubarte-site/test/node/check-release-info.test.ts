import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import {
  compareRelease,
  compareSamples,
  type ReleaseInfo,
  releaseInfoDir,
  type SampleFacts,
} from "../../scripts/check-bench.ts";

// The engine's release_info/ as jubarte_release_info's write stage emits it
// (schema jubarte-redlines/release_info/results_{redline,conversion}/1), six
// documents shrunk to what the proof reads. The facts below match it.
const FIX = fileURLToPath(new URL("../fixtures/release_info", import.meta.url));
const TWO = fileURLToPath(new URL("../fixtures/two_stamps/release_info", import.meta.url));
const read = (name: string) => JSON.parse(readFileSync(`${FIX}/${name}`, "utf8")) as ReleaseInfo;
const REDLINE = read("results_redline_0.9.9_10-03-26_16-51.json");
const CONVERSION = read("results_conversion_0.9.9_10-03-26_16-51.json");

const facts = (): SampleFacts => ({
  GENERATED: "2026-10-03",
  TABLES: [
    {
      id: "conversion-sample",
      title: "DOCX → PDF vs Word's own PDF — 6-document sample",
      meta: "6 docs · 2/2/1/1 by state · scorer pixel-v1 · 2026-10-03",
      desc: "A state-balanced 6-document sample.",
      rows: [
        {
          rank: "1",
          tool: "jubarte †",
          pin: "jubarte 0.9.9",
          median: 90.5,
          mean: 85.0,
          docs: 6,
          failed: 1,
          note: "[88.0, 93.0]",
          ours: true,
        },
        {
          rank: "2",
          tool: "soffice",
          pin: "LibreOffice 26.8.0.3",
          median: 70.25,
          mean: 65.5,
          docs: 6,
          failed: 2,
          note: "[66.0, 74.5]",
        },
      ],
    },
    {
      id: "redlines-sample",
      title: "Redlines vs Word's compare — 6-pair sample",
      meta: "6 pairs · one Word compare each · pixel scorer · opened in Word · 2026-10-03",
      desc: "A sample of 6 document pairs, one Word compare each.",
      rows: [
        {
          rank: "1",
          tool: "jubarte-0.9.9 †",
          pin: "jubarte 0.9.9",
          median: 88.25,
          mean: 81.5,
          docs: 6,
          failed: 1,
          note: "= 100: 2 · ≥ 90: 4",
          ours: true,
        },
        {
          rank: "2",
          tool: "docxodus",
          pin: "Docxodus 12.6.5 (C#)",
          median: 80.5,
          mean: 70.25,
          docs: 6,
          failed: 2,
          note: "= 100: 1 · ≥ 90: 3",
        },
      ],
    },
  ],
  STATES: [
    {
      name: "clean",
      n: "2",
      rows: [
        { tool: "jubarte †", median: 96.0, ours: true },
        { tool: "soffice", median: 90.0 },
      ],
    },
    {
      name: "tracking, no comments",
      n: "4",
      rows: [
        { tool: "jubarte †", median: 85.0, ours: true },
        { tool: "soffice", median: 53.75 },
      ],
    },
  ],
  HEADLINES: [
    {
      label: "DOCX → PDF · median, 6-doc sample",
      value: "90.50",
      vs: "LibreOffice 26.8.0.3 70.25",
      sub: "6 documents, state-balanced.",
    },
    {
      label: "Redline vs Word compare · median, 6-pair sample",
      value: "88.25",
      vs: "Docxodus 80.5",
      sub: "6 pairs, one Word compare each; paired 95% CI of the difference [0.5, 4.25].",
    },
  ],
  HOME_GROUPS: [
    {
      title: "DOCX → PDF vs Word's own export",
      meta: "6-doc sample · median · 2026-10-03",
      rows: [
        { name: "jubarte 0.9.9 †", v: 90.5, ours: true },
        { name: "LibreOffice 26.8.0.3", v: 70.25 },
      ],
    },
    {
      title: "Redlines vs Word's compare, opened in Word",
      meta: "6-pair sample · median · 2026-10-03",
      rows: [
        { name: "jubarte 0.9.9 †", v: 88.25, ours: true },
        { name: "Docxodus 12.6.5 (C#)", v: 80.5 },
      ],
    },
  ],
});

describe("check-bench release_info", () => {
  it("accepts the engine checkout or its release_info folder as --release-info", () => {
    expect(releaseInfoDir("/engines/jubarte-redlines")).toBe(
      "/engines/jubarte-redlines/release_info",
    );
    expect(releaseInfoDir("/engines/jubarte-redlines/release_info")).toBe(
      "/engines/jubarte-redlines/release_info",
    );
  });

  it("passes facts that match the two evidence files", () => {
    expect(compareSamples(facts(), REDLINE, CONVERSION)).toEqual([]);
    expect(compareRelease(FIX, "0.9.9", facts())).toEqual([]);
    // the engine root resolves to its release_info folder
    expect(compareRelease(`${FIX}/..`, "0.9.9", facts())).toEqual([]);
  });

  it("names a drifted median, count, pin, interval and per-state median", () => {
    const f = facts();
    const row = f.TABLES[1].rows[0];
    row.median = 88.26;
    row.note = "= 100: 3 · ≥ 90: 5";
    row.pin = "jubarte 0.9.8";
    f.STATES[0].rows[0].median = 96.5;
    f.HEADLINES[1].sub = "…paired 95% CI of the difference [0.6, 4.3].";
    const diffs = compareSamples(f, REDLINE, CONVERSION);
    expect(diffs).toContain(
      "redlines-sample / jubarte-0.9.9 † / jubarte 0.9.8: median 88.26, release_info 88.25",
    );
    expect(diffs).toContain(
      'redlines-sample / jubarte-0.9.9 † / jubarte 0.9.8: pin "jubarte 0.9.8", release_info "jubarte 0.9.9"',
    );
    expect(
      diffs.some((d) => d.includes('note "= 100: 3 · ≥ 90: 5", release_info = 100: 2 · ≥ 90: 4')),
    ).toBe(true);
    expect(diffs).toContain("STATES / clean / jubarte †: 96.5, release_info 96");
    expect(diffs.some((d) => d.startsWith("headlines / Redline") && d.includes("[0.6, 4.3]"))).toBe(
      true,
    );
  });

  it("holds a headline's comparator to its name and its median at two decimals", () => {
    // 0.11.2 shipped "Docxodus 78.8939" beside "84.28": the evidence's raw
    // median, which a substring test accepted, as it would a longer number
    // that only starts with the median.
    const f = facts();
    f.HEADLINES[1].vs = "Docxodus 80.5123";
    f.HEADLINES[0].vs = "docxide 70.25";
    const diffs = compareSamples(f, REDLINE, CONVERSION);
    expect(diffs.some((d) => d.startsWith("headlines / Redline") && d.includes("80.5123"))).toBe(
      true,
    );
    expect(diffs.some((d) => d.startsWith("headlines / DOCX") && d.includes("docxide"))).toBe(true);
  });

  it("names a sample table or bar the facts no longer carry", () => {
    const f = facts();
    f.TABLES = f.TABLES.filter((t) => t.id !== "conversion-sample");
    f.HOME_GROUPS[0].rows[0].v = 91.0;
    const diffs = compareSamples(f, REDLINE, CONVERSION);
    expect(diffs).toContain("conversion-sample: bench.ts has no such table");
    expect(
      diffs.some((d) => d.startsWith("home_groups / DOCX") && d.includes("91, release_info 90.5")),
    ).toBe(true);
  });

  it("names a missing evidence file a version has none of", () => {
    const diffs = compareRelease(FIX, "0.9.8", facts());
    expect(diffs).toContain(
      `release_info has no results_redline_0.9.8_*.json in ${releaseInfoDir(FIX)}`,
    );
    expect(diffs).toContain(
      `release_info has no results_conversion_0.9.8_*.json in ${releaseInfoDir(FIX)}`,
    );
    // nothing further is claimed about evidence that is not there
    expect(diffs).toHaveLength(2);
  });

  it("names a version held under two stamps, and still checks the other kind", () => {
    const diffs = compareRelease(TWO, "0.9.9", facts());
    expect(diffs).toContain(
      "release_info holds 2 stamps of results_redline_0.9.9: results_redline_0.9.9_10-03-26_16-51.json, results_redline_0.9.9_10-04-26_09-00.json",
    );
    expect(diffs).toHaveLength(1);
  });

  it("names a GENERATED that is not the date the samples were drawn", () => {
    const f = { ...facts(), GENERATED: "2026-10-04" };
    const diffs = compareSamples(f, REDLINE, CONVERSION);
    expect(diffs).toContain("GENERATED is 2026-10-04; the redline sample was drawn 2026-10-03");
    expect(diffs).toContain("GENERATED is 2026-10-04; the conversion sample was drawn 2026-10-03");
  });

  it("checks nothing while a file is missing, and everything once it is there", () => {
    // The conversion evidence absent: only the redline half runs.
    const f = facts();
    f.TABLES[1].rows[0].median = 88.26;
    const diffs = compareSamples(f, REDLINE, null);
    expect(diffs).toEqual([
      "redlines-sample / jubarte-0.9.9 † / jubarte 0.9.9: median 88.26, release_info 88.25",
    ]);
  });
});
