import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { fact, fill, fold, sections } from "../../site/data/facts.ts";
import { render } from "../../site/layout.ts";
import { privacy, terms } from "../../site/pages/legal.ts";

const read = (path: string) => readFileSync(new URL(path, import.meta.url), "utf8");
const line = (id: string, key: string, value: unknown) =>
  JSON.stringify({ id, ts: "", key, value, source: "test" });

describe("data/facts.jsonl", () => {
  it("gives each key its latest record, whatever the line order, and null retires it", () => {
    const log = [
      line("01a0fe27-0000-7000-8000-000000000002", "engine.version", "0.10.2"),
      line("01a0fe27-0000-7000-8000-000000000001", "engine.version", "0.10.1"),
      line("01a0fe27-0000-7000-8000-000000000001", "site.gone", 1),
      line("01a0fe27-0000-7000-8000-000000000003", "site.gone", null),
    ].join("\n");
    expect([...fold(log)]).toEqual([["engine.version", "0.10.2"]]);
  });

  it("fills a section's placeholders, escaped, and fails on a fact it lacks", () => {
    expect(fill("© {{company.name}}")).toBe(`© ${fact("company.name")}`);
    expect(() => fill("{{no.such.fact}}")).toThrow("data/facts.jsonl has no no.such.fact");
  });

  it("files each Terms and Privacy section as its own item, opening first", () => {
    for (const doc of ["privacy", "terms"]) {
      const list = sections(`legal.${doc}`);
      expect(list.length, doc).toBeGreaterThan(5);
      expect(list[0].heading, doc).toBeNull();
      expect(
        list.slice(1).every((s) => s.heading),
        doc,
      ).toBe(true);
      for (const s of list) expect(() => fill(s.html)).not.toThrow();
    }
    expect(sections("legal.terms").map((s) => s.heading)).toContain("Limitation of liability");
    for (const page of [privacy, terms]) expect(render(page)).not.toContain("{{");
  });

  it("holds one page limit for the Worker, wrangler and the legal pages", () => {
    const limit = fact<number>("site.limit_per_minute");
    // Read as text: the node tsconfig rejects the Worker's extensionless imports.
    const worker = /LIMIT_PER_MINUTE: number = (\d+);/.exec(read("../../src/site.ts"))?.[1];
    expect(Number(worker)).toBe(limit);
    expect(read("../../wrangler.jsonc")).toContain(`"simple": { "limit": ${limit}, "period": 60 }`);
    expect(render(terms)).toContain(`more than ${limit} page requests a minute`);
  });

  it("leaves no changeable value written into the data modules", () => {
    for (const file of ["release.ts", "bench.ts"]) {
      const src = read(`../../site/data/${file}`);
      expect(src, file).not.toMatch(/"\d+\.\d+(\.\d+)?"/);
      expect(src, file).not.toMatch(/\$\d/);
    }
  });
});
