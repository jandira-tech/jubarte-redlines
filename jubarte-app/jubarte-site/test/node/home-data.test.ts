import { readFileSync } from "node:fs";
import { gzipSync } from "node:zlib";
import { describe, expect, it } from "vitest";

// `pnpm test` builds first, so public/ is the site about to deploy.
const raw = readFileSync(new URL("../../public/data/home.json", import.meta.url));

describe("home.json", () => {
  it("carries only the fields home.js draws", () => {
    const { cases } = JSON.parse(raw.toString("utf8"));
    for (const c of cases) {
      expect(Object.keys(c).sort()).toEqual(["h", "id", "jubarte", "pages", "soffice", "state"]);
    }
    const js = readFileSync(new URL("../../site/static/js/home.js", import.meta.url), "utf8");
    expect(js).not.toMatch(/\.stem\b/);
  });

  it("stays small over the wire (audit: 46 KB gzipped for one case at a time)", () => {
    expect(gzipSync(raw).length).toBeLessThan(24_000);
  });
});
