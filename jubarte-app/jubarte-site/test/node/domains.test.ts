import { readdirSync, readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { fact } from "../../site/data/facts.ts";

const wrangler = readFileSync(new URL("../../wrangler.jsonc", import.meta.url), "utf8");
const publicDir = new URL("../../public/", import.meta.url);

const customDomains = (config: string) =>
  [...config.matchAll(/"pattern": "([^"]+)", "custom_domain": true/g)].map((m) => m[1]);

describe("the hosts the site answers on", () => {
  it("attaches every one as a custom domain, so a deploy never detaches it", () => {
    expect(customDomains(wrangler)).toEqual(["jubarte.pro", "www.jubarte.pro"]);
  });

  it("are claimed by no other Worker in this repository", () => {
    // Two Workers listing one custom domain take it from each other on every
    // deploy: whichever deployed last answers, the other's pages point at it.
    const ours = new Set(customDomains(wrangler));
    for (const other of ["redlines-site", "verify-worker"]) {
      const theirs = readFileSync(
        new URL(`../../../${other}/wrangler.jsonc`, import.meta.url),
        "utf8",
      );
      expect(
        customDomains(theirs).filter((d) => ours.has(d)),
        other,
      ).toEqual([]);
    }
  });

  it("names only jubarte.pro on its pages", () => {
    const pages = readdirSync(publicDir).filter((f) => f.endsWith(".html"));
    expect(pages.length).toBeGreaterThan(5);
    for (const page of pages) {
      const html = readFileSync(new URL(page, publicDir), "utf8");
      expect(html, page).not.toContain("redlines.free");
      expect(html, page).not.toContain("www.jubarte.pro");
    }
  });
});

describe("the old redline tool's hosts", () => {
  it("redirect to the site's own address", () => {
    const redirect = readFileSync(
      new URL("../../../redlines-site/src/index.ts", import.meta.url),
      "utf8",
    );
    expect(redirect).toContain(`const SITE = "${fact<string>("site.url")}";`);
    // The pages they send people to exist.
    for (const page of ["demo", "privacy", "terms"]) {
      expect(readdirSync(publicDir), page).toContain(`${page}.html`);
    }
  });
});
