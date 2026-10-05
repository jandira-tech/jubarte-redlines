import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const css = readFileSync(new URL("../../site/static/css/site.css", import.meta.url), "utf8");
const sizes = [...css.matchAll(/font-size:\s*([^;]+);/g)].map((m) => m[1].trim());

describe("type follows the reader's font size", () => {
  it("leaves the root size to the browser, so a reader's larger default applies", () => {
    const root = /(?:^|\n)html,\s*body\s*\{([^}]*)\}/.exec(css)?.[1] ?? "";
    expect(root).toContain("font-family");
    expect(root).not.toMatch(/font-size/);
  });

  it("sets every size in rem or em, clamp bounds included", () => {
    expect(sizes.length).toBeGreaterThan(100);
    for (const size of sizes) expect(size, size).not.toMatch(/\dpx/);
  });

  it("keeps the default rendering: 11px labels are 0.6875rem at a 16px default", () => {
    expect(sizes.filter((s) => s === "0.6875rem").length).toBeGreaterThan(30);
  });
});
