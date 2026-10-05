// Checks site/data/bench.ts against the bench's release evidence:
//
//   node scripts/check-bench.ts [--results ../../../neurotic_docx_bench/RESULTS.md]
//                               [--release-info ../../../jubarte-redlines/release_info]
//
// Two proofs, side by side:
//
// - the tables that stay full-corpus (docxide-metrics, accept-reject,
//   below-3-pages, no-redline) are proved against neurotic_docx_bench's
//   RESULTS.md, as before: every row's median, mean, documents, failures,
//   interval or counts, and the version numbers in each pin;
//
// - the two sample tables (conversion-sample, redlines-sample) are proved
//   against the engine checkout's release_info/results_conversion_<v>_<stamp>
//   .json and results_redline_<v>_<stamp>.json for the version engine.version
//   names — the files jubarte_release_info's write stage emits. Every row's
//   figures and pins, the per-state medians, the home-page bars, the headline
//   figures and GENERATED are checked against them. A missing or two-stamp
//   release_info is a difference too, so the site cannot drift from the
//   published evidence.
//
// It exits 1 and lists each difference.

import { readdirSync, readFileSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
  GENERATED,
  HEADLINES,
  type Headline,
  HOME_GROUPS,
  type HomeGroup,
  type Row,
  STATES,
  type StateCell,
  TABLES,
  type Table,
} from "../site/data/bench.ts";
import { ENGINE_VERSION } from "../site/data/release.ts";

export type MdRow = Record<string, string>;
export type Section = { heading: string; tables: MdRow[][] };

/** Every `### ` section of RESULTS.md with its pipe tables, rows keyed by header. */
export function parseResults(md: string): Section[] {
  const sections: Section[] = [];
  let current: Section | null = null;
  let header: string[] | null = null;
  let rows: MdRow[] | null = null;
  const cells = (line: string) =>
    line
      .trim()
      .replace(/^\||\|$/g, "")
      .split("|")
      .map((c) => c.trim());
  for (const line of md.split("\n")) {
    if (line.startsWith("### ")) {
      current = { heading: line.slice(4).trim(), tables: [] };
      sections.push(current);
      header = null;
      continue;
    }
    if (!current) continue;
    if (!line.startsWith("|")) {
      header = null;
      continue;
    }
    const c = cells(line);
    if (!header) {
      header = c;
      rows = [];
      current.tables.push(rows);
    } else if (rows && !c.every((x) => /^-+$/.test(x))) {
      // The |---| line under the header is not a row.
      rows.push(Object.fromEntries(header.map((h, i) => [h, c[i] ?? ""])));
    }
  }
  return sections;
}

// ------------------------------------------------------------------ the RESULTS.md proof

// bench.ts table id → the RESULTS.md heading it copies. The two sample tables
// (conversion-sample, redlines-sample) are not here: check-bench proves them
// against the engine's release_info/ JSONs instead (below).
const SECTION: Record<string, string> = {
  "docxide-metrics": "docxide_metrics:",
  "accept-reject": "redlines accepted or rejected",
  "below-3-pages": "below 3 pages: corpus/word",
  "no-redline": "docx_to_pdf_no_redline_docs:",
};

const STATE_COLUMN: Record<string, string> = {
  clean: "clean",
  "tracking, no comments": "tracking_without_comments",
  "comments + tracking": "with_comments_tracking",
  "comments, clean": "with_comments_clean",
  // website_data's bench.states names the states with the raw keys the results
  // JSONs use (0.11.2's samples), so those map onto themselves.
  tracking_without_comments: "tracking_without_comments",
  with_comments_tracking: "with_comments_tracking",
  with_comments_clean: "with_comments_clean",
};

const tool = (name: string) => name.replace(/\s*†$/, "").trim();
const num = (s: string) => Number.parseFloat(s.replace(/,/g, ""));
const versions = (s: string): string[] => s.match(/\d+(?:\.\d+)+/g) ?? [];
const first = (s: string) => num(s.split("(")[0]);
const near = (a: number, b: number) => Math.abs(a - b) < 0.005;

function findRow(table: MdRow[], row: Row, id: string): MdRow | undefined {
  if (id === "accept-reject") {
    const action = row.pin.endsWith("accept all") ? "accept_all" : "reject_all";
    return table.find((r) => tool(r.Tool) === tool(row.tool) && r.Action === action);
  }
  return table.find((r) => tool(r.Tool) === tool(row.tool));
}

const REDLINE_TABLES = new Set(["accept-reject"]);

/** The figures of a row's note: its interval (and text median), or its = 100 / ≥ 90 counts. */
function noteFigures(id: string, note: string): number[] {
  if (REDLINE_TABLES.has(id)) {
    return [/= 100: ([\d,]+)/, /≥ 90: ([\d,]+)/]
      .map((re) => re.exec(note)?.[1])
      .filter((x): x is string => x !== undefined)
      .map(num);
  }
  return (note.match(/-?\d+(?:\.\d+)?/g) ?? []).map(num);
}

/** What `row` should say, from its RESULTS.md row. */
function expected(
  id: string,
  r: MdRow,
  note: string,
): Omit<Row, "rank" | "tool" | "pin" | "note"> & { notes: number[] } {
  if (REDLINE_TABLES.has(id)) {
    const [scored, total] = r.Scored.split(" ")[0].split("/").map(num);
    // A note may give only the = 100 count (the calibration rows do).
    const notes = [first(r["= 100"])];
    if (note.includes("≥ 90")) notes.push(first(r[">= 90"]));
    return {
      median: first(r["ITT Median"]),
      mean: first(r["ITT Mean"]),
      docs: scored,
      failed: total - scored,
      notes,
    };
  }
  const ci = (r["95% CI"].match(/-?\d+(?:\.\d+)?/g) ?? []).map(num);
  const tb = r["text_boundary median"];
  return {
    median: num(r["ITT Median"]),
    mean: num(r["ITT Mean"]),
    docs: num(r.Docs),
    failed: num(r.Failed),
    notes: tb === undefined ? ci : [...ci, num(tb)],
  };
}

/** Each difference between bench.ts and RESULTS.md, as a readable line. */
export function compare(md: string): string[] {
  const sections = parseResults(md);
  const out: string[] = [];
  for (const t of TABLES) {
    const prefix = SECTION[t.id];
    // The sample tables are the release_info proof's business, not RESULTS.md's.
    if (!prefix) continue;
    const section = sections.find((s) => s.heading.startsWith(prefix));
    if (!section) {
      out.push(`${t.id}: RESULTS.md has no "### ${prefix ?? "?"}" section`);
      continue;
    }
    const table = section.tables[0] ?? [];
    for (const row of t.rows) {
      const where = `${t.id} / ${row.tool} / ${row.pin}`;
      const r = findRow(table, row, t.id);
      if (!r) {
        out.push(`${where}: no such row in RESULTS.md`);
        continue;
      }
      const pin = r.Pin ?? r.Version ?? "";
      const missing = versions(row.pin).filter((v) => !versions(pin).includes(v));
      if (missing.length)
        out.push(`${where}: pin says ${missing.join(", ")}; RESULTS.md says "${pin}"`);
      const want = expected(t.id, r, row.note);
      for (const k of ["median", "mean", "docs", "failed"] as const) {
        if (!near(row[k], want[k])) out.push(`${where}: ${k} ${row[k]}, RESULTS.md ${want[k]}`);
      }
      const notes = noteFigures(t.id, row.note);
      if (notes.length !== want.notes.length || notes.some((n, i) => !near(n, want.notes[i]))) {
        out.push(`${where}: note "${row.note}", RESULTS.md ${want.notes.join(", ")}`);
      }
    }
  }
  return out;
}

// ------------------------------------------------------------------ the release_info JSON proof

/** One results_*.json of the engine's release_info/, as jubarte_release_info writes it. */
export type ReleaseInfo = {
  release?: string;
  stamp?: string;
  sample?: { n?: number; drawn?: { date?: string; seed?: number } };
  tools?: Record<string, ToolBlock>;
  comparison?: { ci95?: number[] };
};
export type ToolBlock = {
  version?: string;
  n?: number;
  failures?: number;
  mean?: number;
  median?: number;
  exact_100?: number;
  at_least_90?: number;
  median_ci95?: number[];
  by_state?: Record<string, { n?: number; median?: number }>;
};

/** The facts the sample proof checks; the CLI passes bench.ts's live values. */
export type SampleFacts = {
  GENERATED: string;
  TABLES: Table[];
  STATES: StateCell[];
  HEADLINES: Headline[];
  HOME_GROUPS: HomeGroup[];
};

export const sampleFacts = (): SampleFacts => ({
  GENERATED,
  TABLES,
  STATES,
  HEADLINES,
  HOME_GROUPS,
});

/** results_redline / results_conversion, the two kinds of evidence JSON. */
export type Kind = "redline" | "conversion";

const COMPARATOR: Record<Kind, string> = { redline: "docxodus", conversion: "soffice" };
const SAMPLE_TABLE: Record<Kind, string> = {
  redline: "redlines-sample",
  conversion: "conversion-sample",
};

/** The release_info folder of `dir`, be it the engine checkout or the folder itself. */
export function releaseInfoDir(dir: string): string {
  return basename(resolve(dir)) === "release_info"
    ? resolve(dir)
    : join(resolve(dir), "release_info");
}

/** The `results_<kind>_<version>_<stamp>.json` files of `version` in `dir`, sorted. */
export function findResults(dir: string, kind: Kind, version: string): string[] {
  const esc = version.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const re = new RegExp(`^results_${kind}_${esc}_\\d{2}-\\d{2}-\\d{2}_\\d{2}-\\d{2}\\.json$`);
  try {
    return readdirSync(dir)
      .filter((name) => re.test(name))
      .sort();
  } catch {
    return [];
  }
}

/** Which tool key of the JSON a table row or bar belongs to, or null when unknown. */
function toolKey(id: string, name: string, ours?: boolean): string | null {
  if (ours) return "jubarte";
  if (id === "conversion-sample" || id === "conversion") {
    return /soffice|libreoffice/i.test(name) ? "soffice" : null;
  }
  return /docxodus/i.test(name) ? "docxodus" : null;
}

const where = (id: string, row: Row) => `${id} / ${row.tool} / ${row.pin}`;

/** Each difference between bench.ts's sample tables and one results JSON.
 * `doc` null (no file, or two stamps) leaves that capability to the caller's
 * own difference line. */
export function compareSamples(
  facts: SampleFacts,
  redlineDoc: ReleaseInfo | null,
  conversionDoc: ReleaseInfo | null,
): string[] {
  const out: string[] = [];
  const docs: Record<Kind, ReleaseInfo | null> = { redline: redlineDoc, conversion: conversionDoc };

  for (const kind of ["redline", "conversion"] as const) {
    const doc = docs[kind];
    const id = SAMPLE_TABLE[kind];
    const table = facts.TABLES.find((t) => t.id === id);
    if (!doc) continue;
    if (!table) {
      out.push(`${id}: bench.ts has no such table`);
      continue;
    }
    for (const row of table.rows) {
      const key = toolKey(id, row.tool, row.ours);
      const t = key ? doc.tools?.[key] : undefined;
      if (!t) {
        out.push(`${where(id, row)}: release_info has no ${key ?? "such"} tool`);
        continue;
      }
      if (row.pin !== t.version)
        out.push(`${where(id, row)}: pin "${row.pin}", release_info "${t.version ?? ""}"`);
      for (const [k, want] of [
        ["median", t.median],
        ["mean", t.mean],
        ["docs", t.n],
        ["failed", t.failures],
      ] as const) {
        if (want === undefined || !near(row[k], want))
          out.push(`${where(id, row)}: ${k} ${row[k]}, release_info ${want}`);
      }
      if (kind === "redline") {
        const got = [/= 100: ([\d,]+)/, /≥ 90: ([\d,]+)/].map((re) => re.exec(row.note)?.[1]);
        const want = [t.exact_100, t.at_least_90];
        if (got.some((g, i) => g === undefined || want[i] === undefined || num(g) !== want[i])) {
          out.push(
            `${where(id, row)}: note "${row.note}", release_info = 100: ${t.exact_100} · ≥ 90: ${t.at_least_90}`,
          );
        }
      } else {
        const ci = (row.note.match(/\[(-?\d+(?:\.\d+)?), (-?\d+(?:\.\d+)?)\]/) ?? [])
          .slice(1)
          .map(num);
        const want = t.median_ci95;
        if (!want || ci.length !== 2 || ci.some((v, i) => !near(v, want[i]))) {
          out.push(`${where(id, row)}: note "${row.note}", release_info [${want?.join(", ")}]`);
        }
      }
    }
    const drawn = doc.sample?.drawn?.date;
    if (drawn !== undefined && drawn !== facts.GENERATED)
      out.push(`GENERATED is ${facts.GENERATED}; the ${kind} sample was drawn ${drawn}`);
  }

  // bench.states: the conversion sample's per-state medians and counts.
  if (conversionDoc) {
    const jub = conversionDoc.tools?.jubarte;
    const sof = conversionDoc.tools?.soffice;
    for (const cell of facts.STATES) {
      const col = STATE_COLUMN[cell.name];
      const j = col ? jub?.by_state?.[col] : undefined;
      const s = col ? sof?.by_state?.[col] : undefined;
      if (!j || !s) {
        out.push(`STATES / ${cell.name}: release_info has no ${col ?? "?"} state`);
        continue;
      }
      if (num(cell.n) !== j.n) out.push(`STATES / ${cell.name}: n ${cell.n}, release_info ${j.n}`);
      for (const row of cell.rows) {
        const key = toolKey("conversion-sample", row.tool, row.ours);
        const t = key === "jubarte" ? j : key === "soffice" ? s : undefined;
        const median = t?.median ?? Number.NaN;
        if (!near(row.median, median))
          out.push(`STATES / ${cell.name} / ${row.tool}: ${row.median}, release_info ${t?.median}`);
      }
    }
    for (const [state, block] of Object.entries(conversionDoc.tools?.jubarte?.by_state ?? {})) {
      if (![...Object.keys(STATE_COLUMN)].some((name) => STATE_COLUMN[name] === state))
        out.push(`STATES: release_info has a ${state} state bench.ts does not show (n ${block.n})`);
    }
  }

  // The home-page bars and the headline figures restate the same aggregates.
  for (const [group, doc, id] of [
    [facts.HOME_GROUPS.find((g) => g.title.startsWith("DOCX")), conversionDoc, "conversion"],
    [facts.HOME_GROUPS.find((g) => !g.title.startsWith("DOCX")), redlineDoc, "redline"],
  ] as const) {
    if (!doc) continue;
    if (!group) {
      out.push(`home_groups: bench.ts has no ${id} group`);
      continue;
    }
    for (const bar of group.rows) {
      const key = toolKey(id, bar.name, bar.ours);
      const t = key ? doc.tools?.[key] : undefined;
      if (!t || t.median === undefined || !near(bar.v, t.median)) {
        out.push(`home_groups / ${group.title} / ${bar.name}: ${bar.v}, release_info ${t?.median}`);
        continue;
      }
      const pin = t.version ?? "";
      const missing = versions(bar.name).filter((v) => !versions(pin).includes(v));
      if (missing.length)
        out.push(`home_groups / ${bar.name}: says ${missing.join(", ")}; release_info "${pin}"`);
    }
  }
  for (const h of facts.HEADLINES) {
    const kind: Kind = h.label.startsWith("DOCX") ? "conversion" : "redline";
    const doc = docs[kind];
    if (!doc) continue;
    const jub = doc.tools?.jubarte;
    const cmp = doc.tools?.[COMPARATOR[kind]];
    if (!jub || jub.median === undefined || !cmp || cmp.median === undefined) continue;
    if (h.value !== jub.median.toFixed(2))
      out.push(`headlines / ${h.label}: value ${h.value}, release_info ${jub.median.toFixed(2)}`);
    // The comparator's name, then its median as the page prints ours: two
    // decimals. A substring test let the evidence's raw 78.8939 through.
    const vsNum = /(-?\d+(?:\.\d+)?)\s*$/.exec(h.vs)?.[1];
    const named = h.vs.startsWith((cmp.version ?? "").split(" ")[0]);
    if (!named || vsNum === undefined || num(vsNum) !== num(cmp.median.toFixed(2)))
      out.push(
        `headlines / ${h.label}: vs "${h.vs}", release_info ${cmp.version ?? ""} ${cmp.median}`,
      );
    if (kind === "redline") {
      const ci = (h.sub.match(/\[(-?\d+(?:\.\d+)?), (-?\d+(?:\.\d+)?)\]/) ?? []).slice(1).map(num);
      const want = doc.comparison?.ci95;
      if (!want || ci.length !== 2 || ci.some((v, i) => !near(v, want[i])))
        out.push(`headlines / ${h.label}: paired CI ${h.sub}, release_info [${want?.join(", ")}]`);
    }
  }
  return out;
}

/** The whole release_info proof of `dir` (the engine checkout or its release_info/):
 * missing and two-stamp evidence are differences, then the sample proof. */
export function compareRelease(dir: string, version: string, facts: SampleFacts): string[] {
  const folder = releaseInfoDir(dir);
  const out: string[] = [];
  const docs: Partial<Record<Kind, ReleaseInfo | null>> = {};
  for (const kind of ["redline", "conversion"] as const) {
    const names = findResults(folder, kind, version);
    if (names.length === 0) {
      out.push(`release_info has no results_${kind}_${version}_*.json in ${folder}`);
      docs[kind] = null;
    } else if (names.length > 1) {
      out.push(
        `release_info holds ${names.length} stamps of results_${kind}_${version}: ${names.join(", ")}`,
      );
      docs[kind] = null;
    } else {
      docs[kind] = JSON.parse(readFileSync(join(folder, names[0]), "utf8")) as ReleaseInfo;
    }
  }
  const stamps = new Set(
    (["redline", "conversion"] as const)
      .map((k) => docs[k]?.stamp)
      .filter((s): s is string => s !== undefined),
  );
  if (stamps.size > 1) out.push(`release_info files carry two stamps: ${[...stamps].join(", ")}`);
  return [...out, ...compareSamples(facts, docs.redline ?? null, docs.conversion ?? null)];
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const argv = process.argv.slice(2);
  const flag = argv.indexOf("--results");
  const info = argv.indexOf("--release-info");
  const here = dirname(fileURLToPath(import.meta.url));
  const path =
    flag >= 0 ? argv[flag + 1] : join(here, "../../../../neurotic_docx_bench/RESULTS.md");
  const releaseInfo =
    info >= 0 ? argv[info + 1] : join(here, "../../../../jubarte-redlines/release_info");
  const diffs = [
    ...compare(readFileSync(path, "utf8")),
    ...compareRelease(releaseInfo, ENGINE_VERSION, sampleFacts()),
  ];
  for (const d of diffs) console.error(d);
  if (diffs.length) {
    console.error(
      `site/data/bench.ts differs from ${path} / ${releaseInfoDir(releaseInfo)} in ${diffs.length} place(s)`,
    );
    process.exit(1);
  }
  console.log(
    `site/data/bench.ts matches ${path} and ${releaseInfoDir(releaseInfo)} (${ENGINE_VERSION})`,
  );
}
