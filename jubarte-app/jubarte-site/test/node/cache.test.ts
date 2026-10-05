import { existsSync, readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

// `pnpm test` builds first, so public/ is the site about to deploy.
const pub = (path: string) =>
  readFileSync(new URL(`../../public/${path}`, import.meta.url), "utf8");

describe("cached files after a deploy", () => {
  it("revalidates the case data, so a release's scores reach a returning visitor at once", () => {
    const rule = pub("_headers").split("\n/data/*\n")[1]?.split("\n")[0];
    expect(rule?.trim()).toBe("Cache-Control: public, max-age=0, must-revalidate");
  });

  it("names the dataset revision the home strip's page images are pinned to", () => {
    const home = JSON.parse(pub("data/home.json"));
    const lock = readFileSync(new URL("../../fixtures.lock", import.meta.url), "utf8");
    expect(lock.trim().endsWith(`@${home.revision}`)).toBe(true);
    expect(home.cases.length).toBeGreaterThan(0);
  });
});

describe("data fetches", () => {
  // A browser that cached the data under the old max-age=3600 still calls it fresh;
  // only a revalidating fetch replaces it on the visit after a deploy.
  for (const page of ["home", "cases"]) {
    it(`${page}.js revalidates every /data fetch`, () => {
      const js = readFileSync(new URL(`../../site/static/js/${page}.js`, import.meta.url), "utf8");
      const fetches = [...js.matchAll(/fetch\(([^)]*\/data\/[^)]*)\)/g)].map((m) => m[1]);
      expect(fetches.length).toBeGreaterThan(0);
      for (const call of fetches) expect(call).toContain('{ cache: "no-cache" }');
    });
  }
});

describe("the stylesheet", () => {
  const rule = (path: string) => pub("_headers").split(`\n${path}\n`)[1]?.split("\n")[0]?.trim();
  const exists = (path: string) => existsSync(new URL(`../../public${path}`, import.meta.url));

  it("is cached for a year under a name that carries its content hash", () => {
    expect(rule("/static/css/*")).toBe("Cache-Control: public, max-age=31536000, immutable");
    for (const page of ["index.html", "demo.html", "use-cases.html", "404.html"]) {
      const href = /<link rel="stylesheet" href="([^"]+)">/.exec(pub(page))?.[1];
      expect(href, page).toMatch(/^\/static\/css\/site\.[0-9a-f]{10}\.css$/);
      expect(exists(href ?? ""), `${page} links ${href}`).toBe(true);
    }
    expect(exists("/static/css/site.css")).toBe(false);
  });

  it("reaches the 429 page by a plain name that revalidates", () => {
    const worker = readFileSync(new URL("../../src/site.ts", import.meta.url), "utf8");
    const href = /<link rel="stylesheet" href="([^"]+)">/.exec(worker)?.[1];
    expect(href).toBe("/static/site.css");
    expect(exists("/static/site.css")).toBe(true);
    // No rule covers it, so the asset server's default (max-age=0, must-revalidate) applies.
    expect(pub("_headers")).not.toMatch(/^\/static\/\*$/m);
  });
});
