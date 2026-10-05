// Every number the site prints about the benchmark, read from data/facts.jsonl
// (bench.*), written with scripts/facts.py. The two tables with ids
// `conversion-sample` and `redlines-sample` are the release's 600-item
// samples (600 conversion fixtures; 600 document pairs with one Word compare
// each): their values come from the engine's
// release_info/results_conversion_<version>_<stamp>.json and
// release_info/results_redline_<version>_<stamp>.json for the version
// `engine.version` names, and scripts/check-bench.ts proves them against
// those files. Every other table keeps its RESULTS.md provenance, which
// check-bench.ts proves the same way. Do not round or restate figures by hand.

import { fact } from "./facts.ts";

export const GENERATED: string = fact("bench.generated");
export const BENCH_VERSION: string = fact("bench.version");
export const BENCH_REPO: string = fact("bench.repo");
export const BENCH_VIEWER: string = fact("bench.viewer");
export const RESULTS_URL: string = `${BENCH_REPO}/blob/main/RESULTS.md`;

export type Row = {
  rank: string;
  tool: string;
  pin: string;
  /** ITT median, the ranking statistic. */
  median: number;
  /** ITT mean. */
  mean: number;
  /** Documents (or pairs) scored. */
  docs: number;
  failed: number;
  /** Right of the bar: the 95% CI, or the table's own extra figures. */
  note: string;
  ours?: boolean;
};

export type Table = { id: string; title: string; meta: string; desc: string; rows: Row[] };

export const TABLES: Table[] = fact("bench.tables");

export type Headline = { label: string; value: string; vs: string; sub: string };

export const HEADLINES: Headline[] = fact("bench.headlines");

export type StateRow = { tool: string; median: number; ours?: boolean };
export type StateCell = { name: string; n: string; rows: StateRow[] };

/** The conversion sample by corpus state, ITT median. */
export const STATES: StateCell[] = fact("bench.states");

/** The bench names a state by its corpus folder; the page says it in words. */
const STATE_LABELS: Record<string, string> = {
  tracking_without_comments: "tracking, no comments",
  with_comments_clean: "comments, clean",
  with_comments_tracking: "comments + tracking",
};
export const stateLabel = (name: string): string => STATE_LABELS[name] ?? name;

/** A bar on the home page: a tool, its median score, and whether it is jubarte. */
export type HomeBar = { name: string; v: number; ours?: boolean };
/** `note` explains a bar the number alone would mislead on. */
export type HomeGroup = { title: string; meta: string; rows: HomeBar[]; note?: string };

/** The two bar groups on the home page. */
export const HOME_GROUPS: HomeGroup[] = fact("bench.home_groups");

/** A point of the benchmark method: title and description. */
export type MethodPoint = { t: string; d: string };
export const METHOD: MethodPoint[] = fact("bench.method");
