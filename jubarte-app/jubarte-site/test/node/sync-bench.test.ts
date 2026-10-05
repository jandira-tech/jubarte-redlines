import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import {
  benchFacts,
  changes,
  findWebsiteData,
  parseWebsiteData,
  proofFacts,
} from "../../scripts/sync-bench.ts";
import type { Table } from "../../site/data/bench.ts";

// The engine's release_info/website_data_<version>_<stamp>.jsonl as
// jubarte_release_info's write stage emits it, shrunk to the six-item samples
// of the results fixtures beside it (check-release-info.test.ts).
const FIX = fileURLToPath(new URL("../fixtures/release_info", import.meta.url));
const NAME = "website_data_0.9.9_10-03-26_16-51.jsonl";
const RECORDS = parseWebsiteData(readFileSync(join(FIX, NAME), "utf8"));

const table = (id: string, title = id): Table => ({ id, title, meta: "", desc: "", rows: [] });
const ids = (facts: Record<string, unknown>) => (facts["bench.tables"] as Table[]).map((t) => t.id);

describe("sync-bench", () => {
  it("takes the bench records that are not placeholders, and nothing else", () => {
    expect(Object.keys(benchFacts("0.9.9", RECORDS, []))).toEqual([
      "bench.generated",
      "bench.tables",
      "bench.headlines",
      "bench.states",
      "bench.home_groups",
      "bench.method",
    ]);
  });

  it("keeps the tables the release does not name, after its own", () => {
    const now = [
      table("conversion-sample", "the previous release's sample"),
      table("docxide-metrics"),
      table("accept-reject"),
    ];
    const facts = benchFacts("0.9.9", RECORDS, now);
    expect(ids(facts)).toEqual([
      "conversion-sample",
      "redlines-sample",
      "docxide-metrics",
      "accept-reject",
    ]);
    expect((facts["bench.tables"] as Table[])[0].title).toContain("6-document sample");
  });

  it("changes nothing the second time", () => {
    const first = benchFacts("0.9.9", RECORDS, [table("docxide-metrics")]);
    const again = benchFacts("0.9.9", RECORDS, first["bench.tables"] as Table[]);
    expect(again).toEqual(first);
    expect(changes(new Map(Object.entries(first)), again)).toEqual([]);
  });

  it("names the keys a merge would move, whatever the order of an object's fields", () => {
    const now = new Map<string, unknown>([
      ["bench.generated", "2026-10-03"],
      ["bench.method", [{ d: "words", t: "Oracle" }]],
    ]);
    expect(
      changes(now, {
        "bench.generated": "2026-10-04",
        "bench.method": [{ t: "Oracle", d: "words" }],
        "bench.states": [],
      }),
    ).toEqual(["bench.generated", "bench.states"]);
  });

  it("refuses the file of another release", () => {
    expect(() => benchFacts("0.9.8", RECORDS, [])).toThrow(/names engine 0\.9\.9, not 0\.9\.8/);
  });

  it("refuses a file that moves no bench figure", () => {
    const none = RECORDS.filter((r) => !r.key.startsWith("bench.") || r.pending);
    expect(() => benchFacts("0.9.9", none, [])).toThrow(/no bench\./);
  });

  it("refuses a line that is not a record, by its number", () => {
    expect(() => parseWebsiteData('{"key":"bench.generated","value":"x"}\n[1]\n')).toThrow(
      /line 2/,
    );
    expect(() => parseWebsiteData("not json\n")).toThrow(/line 1/);
  });

  it("finds a version's one file, and says when there is none or two", () => {
    expect(findWebsiteData(FIX, "0.9.9")).toEqual([NAME]);
    expect(findWebsiteData(FIX, "0.9.8")).toEqual([]);
    const two = mkdtempSync(join(tmpdir(), "sync-bench-"));
    try {
      writeFileSync(join(two, NAME), "");
      writeFileSync(join(two, "website_data_0.9.9_10-04-26_09-00.jsonl"), "");
      // a stray name is not a second stamp
      writeFileSync(join(two, "website_data_0.9.9_16-51.jsonl"), "");
      expect(findWebsiteData(two, "0.9.9")).toHaveLength(2);
    } finally {
      rmSync(two, { recursive: true });
    }
  });

  it("hands the proof the figures as they will read once merged", () => {
    const now = new Map<string, unknown>([
      ["bench.generated", "2026-10-01"],
      ["bench.states", []],
    ]);
    const next = proofFacts(now, benchFacts("0.9.9", RECORDS, []));
    expect(next.GENERATED).toBe("2026-10-03");
    expect(next.TABLES.map((t) => t.id)).toEqual(["conversion-sample", "redlines-sample"]);
    expect(next.STATES).toHaveLength(2);
  });
});
