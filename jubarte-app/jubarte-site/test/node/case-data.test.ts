import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

// `pnpm test` builds first; the build refuses case data without `resolve`.
const data = (bench: string) =>
  JSON.parse(
    readFileSync(new URL(`../../public/data/cases-${bench}.json`, import.meta.url), "utf8"),
  );

describe("case data", () => {
  it.each(["convert", "redline"])("%s: one URL base, file names per case", (bench) => {
    const d = data(bench);
    expect(d.resolve).toBe(`https://huggingface.co/datasets/${d.repo}/resolve/${d.revision}/site`);
    expect(d.hub).toBe(`https://huggingface.co/datasets/${d.repo}/tree/${d.revision}/site`);
    for (const c of d.cases) {
      expect(Array.isArray(c.files)).toBe(true);
      expect(c).not.toHaveProperty("folder");
      for (const f of c.files) expect(f).toMatch(/^[\w.-]+$/);
    }
  });

  it("the viewer builds file and folder links from those bases", () => {
    const js = readFileSync(new URL("../../site/static/js/cases.js", import.meta.url), "utf8");
    expect(js).toMatch(/\$\{d\.resolve\}\/\$\{st\.bench\}\/\$\{c\.id\}\/\$\{n\}/);
    expect(js).toMatch(/\$\{st\.data\?\.hub\}\/\$\{st\.bench\}\/\$\{c\.id\}`/);
    expect(js).not.toMatch(/c\.folder|c\.files\[/);
  });
});
