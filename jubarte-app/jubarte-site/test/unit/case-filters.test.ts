import { describe, expect, it } from "vitest";
import {
  applyFilters,
  bucketOf,
  type Case,
  DEFAULT_FILTERS,
  fixtureUrl,
  pageNav,
  pagesDiffer,
  parseHash,
  stateLabel,
} from "../../site/static/js/case-filters.js";

const render = (pages: number) => ({ pages, h: pages * 906, offsets: [] as [number, number][] });
const kase = (
  id: string,
  state: string,
  wordPages: number,
  jub: number,
  jubPages = wordPages,
  failed = false,
): Case => ({
  id,
  stem: id,
  state,
  engines: {
    jubarte: { score: jub, failed, jaccard: null, text_boundary: null },
    soffice: { score: 50, failed: false, jaccard: null, text_boundary: null },
  },
  renders: { word: render(wordPages), jubarte: render(jubPages), soffice: render(wordPages) },
  files: [],
});

const CASES = [
  kase("a", "clean", 1, 99),
  kase("b", "clean", 3, 60, 4),
  kase("c", "tracking_without_comments", 12, 85),
  kase("d", "with_comments_clean", 5, 0, 5, true),
];
const shown = { jubarte: true, soffice: true };
const ids = (cs: Case[]) => cs.map((c) => c.id);

describe("case filters", () => {
  it("keeps everything by default", () => {
    expect(ids(applyFilters(CASES, DEFAULT_FILTERS, shown))).toEqual(["a", "b", "c", "d"]);
  });

  it("filters by group and by Word's page count", () => {
    expect(ids(applyFilters(CASES, { ...DEFAULT_FILTERS, group: "clean" }, shown))).toEqual([
      "a",
      "b",
    ]);
    expect(ids(applyFilters(CASES, { ...DEFAULT_FILTERS, bucket: "11+" }, shown))).toEqual(["c"]);
    expect(ids(applyFilters(CASES, { ...DEFAULT_FILTERS, bucket: "2-3" }, shown))).toEqual(["b"]);
  });

  it("caps jubarte's score, counting a failure as 0", () => {
    expect(ids(applyFilters(CASES, { ...DEFAULT_FILTERS, maxScore: 60 }, shown))).toEqual([
      "b",
      "d",
    ]);
  });

  it("finds page-count mismatches only among the engines shown", () => {
    expect(ids(applyFilters(CASES, { ...DEFAULT_FILTERS, differ: true }, shown))).toEqual(["b"]);
    expect(pagesDiffer(CASES[1], { jubarte: false, soffice: true })).toBe(false);
  });

  it("buckets page counts and labels states", () => {
    expect([1, 2, 3, 4, 10, 11].map(bucketOf)).toEqual(["1", "2-3", "2-3", "4-10", "4-10", "11+"]);
    expect(stateLabel("tracking_without_comments")).toBe("tracking · no comments");
    expect(stateLabel("new_state")).toBe("new state");
  });

  it("parses permalinks", () => {
    expect(parseHash("#redline/r-ecad91f16e")).toEqual({ bench: "redline", id: "r-ecad91f16e" });
    expect(parseHash("#convert/clean-010cd893de")).toEqual({
      bench: "convert",
      id: "clean-010cd893de",
    });
    expect(parseHash("#other/x")).toBeNull();
    expect(parseHash("")).toBeNull();
  });
});

describe("pageNav", () => {
  it("offers a page turn only toward a page that was drawn", () => {
    expect(pageNav(1, 3)).toEqual({ page: 1, prev: false, next: true });
    expect(pageNav(2, 3)).toEqual({ page: 2, prev: true, next: true });
    expect(pageNav(3, 3)).toEqual({ page: 3, prev: true, next: false });
  });

  it("keeps the page inside the drawn ones, and a one-page case turns nowhere", () => {
    expect(pageNav(9, 3)).toEqual({ page: 3, prev: true, next: false });
    expect(pageNav(0, 3)).toEqual({ page: 1, prev: false, next: true });
    expect(pageNav(1, 1)).toEqual({ page: 1, prev: false, next: false });
    expect(pageNav(1, 0)).toEqual({ page: 1, prev: false, next: false });
  });
});

describe("fixtureUrl", () => {
  it("pins a page strip to the dataset revision, so a release never pairs old pages with new scores", () => {
    expect(fixtureUrl("convert", "clean-490dfc032d", "jubarte.webp", "f0ea4a014a6aaaab7dcd")).toBe(
      "/fixtures/convert/clean-490dfc032d/jubarte.webp?v=f0ea4a014a6a",
    );
  });

  it("keeps the bare path when the data names no revision", () => {
    expect(fixtureUrl("redline", "r-00baea9614", "word.webp", "")).toBe(
      "/fixtures/redline/r-00baea9614/word.webp",
    );
  });
});
