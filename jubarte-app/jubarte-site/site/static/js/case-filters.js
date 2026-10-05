// The case viewer's filters, kept free of the DOM so the unit tests can run
// them in Node against the real case files.

/**
 * @typedef {{ score: number, failed: boolean, page_scores?: number[], jaccard: number | null, text_boundary: number | null, pages_mismatch?: boolean | null }} EngineScore
 * @typedef {{ pages: number, h: number, offsets: [number, number][] }} Render
 * @typedef {{ id: string, stem: string, state: string, engines: Record<string, EngineScore>, renders: Record<string, Render>, files: string[] }} Case
 * @typedef {{ group: string, bucket: string, maxScore: number, differ: boolean }} Filters
 */

/** @type {Filters} */
export const DEFAULT_FILTERS = { group: "all", bucket: "any", maxScore: 100, differ: false };

const STATE_LABELS = {
  clean: "clean",
  tracking_without_comments: "tracking · no comments",
  with_comments_tracking: "comments + tracking",
  with_comments_clean: "comments · clean",
};

/** @param {string} state */
export function stateLabel(state) {
  return STATE_LABELS[/** @type {keyof typeof STATE_LABELS} */ (state)] ?? state.replace(/_/g, " ");
}

/** The "Pages in Word" bucket a page count falls in. @param {number} pages */
export function bucketOf(pages) {
  if (pages <= 1) return "1";
  if (pages <= 3) return "2-3";
  if (pages <= 10) return "4-10";
  return "11+";
}

/**
 * The page to show, clamped to the pages drawn, and which way it can turn.
 * @param {number} page @param {number} drawn
 */
export function pageNav(page, drawn) {
  const last = Math.max(1, drawn);
  const p = Math.min(Math.max(1, page), last);
  return { page: p, prev: p > 1, next: p < last };
}

/**
 * Whether a shown engine's output has a different page count from Word's.
 * @param {Case} c
 * @param {Record<string, boolean>} shown
 */
export function pagesDiffer(c, shown) {
  const word = c.renders.word?.pages;
  if (word == null) return false;
  return Object.entries(shown).some(([k, on]) => {
    if (!on || c.engines[k]?.failed) return false;
    const r = c.renders[k];
    return !!r && r.pages !== word;
  });
}

/**
 * The cases the filters keep, in their published order.
 * @param {Case[]} cases
 * @param {Filters} f
 * @param {Record<string, boolean>} shown
 */
export function applyFilters(cases, f, shown) {
  return cases.filter((c) => {
    if (f.group !== "all" && c.state !== f.group) return false;
    if (f.bucket !== "any" && bucketOf(c.renders.word?.pages ?? 0) !== f.bucket) return false;
    const jub = c.engines.jubarte;
    if (f.maxScore < 100 && (!jub || (jub.failed ? 0 : jub.score) > f.maxScore)) return false;
    if (f.differ && !pagesDiffer(c, shown)) return false;
    return true;
  });
}

/**
 * `#convert/<id>` or `#redline/<id>`; null for anything else.
 * @param {string} hash
 * @returns {{ bench: "convert" | "redline", id: string } | null}
 */
export function parseHash(hash) {
  const m = /^#(convert|redline)\/([\w.-]+)$/.exec(hash);
  return m ? { bench: /** @type {"convert" | "redline"} */ (m[1]), id: m[2] } : null;
}

/**
 * A case's file under /fixtures, pinned to the dataset revision the data names.
 * The paths stay the same from release to release; the query keeps a cached page
 * strip from outliving the scores it belongs to.
 * @param {string} bench
 * @param {string} id
 * @param {string} file
 * @param {string} revision
 */
export function fixtureUrl(bench, id, file, revision) {
  const path = `/fixtures/${bench}/${id}/${file}`;
  return revision ? `${path}?v=${revision.slice(0, 12)}` : path;
}
