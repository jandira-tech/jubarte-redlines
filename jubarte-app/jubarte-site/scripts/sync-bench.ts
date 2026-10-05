// Moves a release's benchmark figures into data/facts.jsonl:
//
//   node scripts/sync-bench.ts 0.11.2 [--release-info ../../../jubarte-redlines] [--check]
//
// The engine's release_info/website_data_<version>_<stamp>.jsonl lists every
// website fact the release moves, one record a line, in the shape of
// data/facts.jsonl. Its bench.* records go in as they are. The records marked
// "pending" are placeholders, and engine.* and release.* are sync-release.ts's
// (they come from the GitHub release), so the log must already name the
// version. bench.tables carries the release's sample tables only: the tables
// of the log it does not name (the full-corpus ones, each on its own run)
// stay, after them.
//
// Nothing unproved is written: the figures are first checked against the
// results JSONs beside the file (check-bench.ts's release_info proof).
// scripts/facts.py writes the records, and only the values that changed, so a
// second run adds nothing. --check writes nothing and exits 1 when a value
// would change.

import { execFileSync } from "node:child_process";
import { readdirSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import type { Headline, HomeGroup, StateCell, Table } from "../site/data/bench.ts";
import { fold } from "../site/data/facts.ts";
import { compareRelease, releaseInfoDir, type SampleFacts } from "./check-bench.ts";

/** One line of website_data: a facts record, or a placeholder the site step fills. */
export type WebsiteRecord = { key: string; value: unknown; pending?: boolean };

/** The records of a website_data file; a line that is not one fails by its number. */
export function parseWebsiteData(text: string): WebsiteRecord[] {
  const out: WebsiteRecord[] = [];
  for (const [i, raw] of text.split("\n").entries()) {
    if (!raw.trim()) continue;
    let rec: unknown;
    try {
      rec = JSON.parse(raw);
    } catch {
      throw new Error(`website_data line ${i + 1} is not JSON`);
    }
    const key = (rec as WebsiteRecord | null)?.key;
    if (typeof key !== "string" || !key || Array.isArray(rec) || !("value" in (rec as object))) {
      throw new Error(`website_data line ${i + 1} is not a { key, value } record`);
    }
    out.push(rec as WebsiteRecord);
  }
  return out;
}

/** The `website_data_<version>_<stamp>.jsonl` files of `version` in `dir`, sorted. */
export function findWebsiteData(dir: string, version: string): string[] {
  const esc = version.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const re = new RegExp(`^website_data_${esc}_\\d{2}-\\d{2}-\\d{2}_\\d{2}-\\d{2}\\.jsonl$`);
  try {
    return readdirSync(dir)
      .filter((name) => re.test(name))
      .sort();
  } catch {
    return [];
  }
}

/** The facts `version`'s website_data moves, keyed as data/facts.jsonl keys them.
 * `tables` is the log's bench.tables as it stands. */
export function benchFacts(
  version: string,
  records: WebsiteRecord[],
  tables: Table[],
): Record<string, unknown> {
  const named = records.find((r) => r.key === "engine.version")?.value;
  if (named !== version) {
    throw new Error(`website_data names engine ${named ?? "nothing"}, not ${version}`);
  }
  const facts: Record<string, unknown> = {};
  for (const r of records) {
    if (r.key.startsWith("bench.") && !r.pending) facts[r.key] = r.value;
  }
  if (!Object.keys(facts).length) throw new Error("website_data moves no bench.* fact");
  const own = facts["bench.tables"] as Table[] | undefined;
  if (own) {
    const ids = new Set(own.map((t) => t.id));
    facts["bench.tables"] = [...own, ...tables.filter((t) => !ids.has(t.id))];
  }
  return facts;
}

/** JSON with every object's keys in order, so two equal values read the same. */
function canonical(value: unknown): string {
  return JSON.stringify(value, (_, v: unknown) =>
    v && typeof v === "object" && !Array.isArray(v)
      ? Object.fromEntries(Object.entries(v).sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0)))
      : v,
  );
}

/** The keys of `values` whose value is not the one `now` holds. */
export function changes(now: Map<string, unknown>, values: Record<string, unknown>): string[] {
  return Object.keys(values).filter(
    (k) => !now.has(k) || canonical(now.get(k)) !== canonical(values[k]),
  );
}

/** What the release_info proof reads, as the log will hold it once `values` is merged. */
export function proofFacts(
  now: Map<string, unknown>,
  values: Record<string, unknown>,
): SampleFacts {
  const get = <T>(key: string, none: T): T => (values[key] ?? now.get(key) ?? none) as T;
  return {
    GENERATED: get<string>("bench.generated", ""),
    TABLES: get<Table[]>("bench.tables", []),
    STATES: get<StateCell[]>("bench.states", []),
    HEADLINES: get<Headline[]>("bench.headlines", []),
    HOME_GROUPS: get<HomeGroup[]>("bench.home_groups", []),
  };
}

function main(argv: string[]): void {
  const version = argv[0];
  if (!/^\d+\.\d+\.\d+$/.test(version ?? "")) {
    throw new Error("usage: node scripts/sync-bench.ts <x.y.z> [--release-info DIR] [--check]");
  }
  const here = dirname(fileURLToPath(import.meta.url));
  const flag = argv.indexOf("--release-info");
  const dir = releaseInfoDir(
    flag > 0 ? argv[flag + 1] : join(here, "../../../../jubarte-redlines/release_info"),
  );
  const now = fold(readFileSync(join(here, "../../data/facts.jsonl"), "utf8"));
  if (now.get("engine.version") !== version) {
    throw new Error(
      `data/facts.jsonl names engine ${now.get("engine.version")}, not ${version}: run scripts/release.sh engine ${version} first`,
    );
  }
  const names = findWebsiteData(dir, version);
  if (names.length !== 1) {
    throw new Error(
      names.length
        ? `release_info holds ${names.length} stamps of website_data_${version}: ${names.join(", ")}`
        : `release_info has no website_data_${version}_*.jsonl in ${dir}`,
    );
  }
  const records = parseWebsiteData(readFileSync(join(dir, names[0]), "utf8"));
  const facts = benchFacts(version, records, (now.get("bench.tables") as Table[]) ?? []);
  const diffs = compareRelease(dir, version, proofFacts(now, facts));
  if (diffs.length) {
    throw new Error(
      `${names[0]} differs from the results beside it in ${diffs.length} place(s); nothing written:\n${diffs.join("\n")}`,
    );
  }
  const moved = changes(now, facts);
  if (argv.includes("--check")) {
    if (moved.length) {
      throw new Error(
        `data/facts.jsonl is not on ${names[0]} (${moved.join(", ")}): run scripts/release.sh figures ${version}`,
      );
    }
    console.log(`data/facts.jsonl holds the figures of ${names[0]}`);
    return;
  }
  const out = execFileSync(
    "uv",
    [
      "run",
      "--python",
      "3.14",
      join(here, "../../scripts/facts.py"),
      "merge",
      "-",
      "--source",
      `release_info/${names[0]} — neurotic_docx_bench release evidence of jubarte ${version}`,
    ],
    { encoding: "utf8", input: JSON.stringify(facts) },
  );
  process.stdout.write(out);
  console.log(`data/facts.jsonl ← ${names[0]}`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    main(process.argv.slice(2));
  } catch (e) {
    console.error(e instanceof Error ? e.message : e);
    process.exit(1);
  }
}
