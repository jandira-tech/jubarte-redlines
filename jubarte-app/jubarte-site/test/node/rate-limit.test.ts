import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { render } from "../../site/layout.ts";
import { privacy, terms } from "../../site/pages/legal.ts";

// The binding, the 429 page and the legal pages must state one number. 60 a minute
// lets an ordinary visitor click through the site; assets sit outside the limiter.
const read = (path: string) => readFileSync(new URL(`../../${path}`, import.meta.url), "utf8");

describe("page rate limit", () => {
  // Read as text: importing the Worker's source would put src/ under the node
  // tsconfig, which rejects its extensionless imports.
  it("is 60 pages a minute", () => {
    expect(/LIMIT_PER_MINUTE: number = (\d+);/.exec(read("src/site.ts"))?.[1]).toBe("60");
  });

  it("matches the ratelimit binding in wrangler.jsonc", () => {
    expect(read("wrangler.jsonc")).toMatch(/"simple":\s*\{\s*"limit":\s*60,\s*"period":\s*60\s*\}/);
  });

  it("is the number the privacy and terms text states", () => {
    // The text is data/facts.jsonl's; read it as the pages print it.
    const legal = render(privacy) + render(terms);
    expect(legal).not.toMatch(/more than 10 (in a minute|page requests)/);
    expect(legal.match(/more than 60/g)?.length).toBe(2);
  });

  it("is the number the comment above the binding states", () => {
    const comment = /\/\/ At most (\d+) requests a minute per client IP/.exec(
      read("wrangler.jsonc"),
    );
    expect(comment?.[1]).toBe("60");
  });

  it("dates the legal pages to the revision that raised it, and says what changed", () => {
    for (const page of [privacy, terms]) {
      expect(render(page)).toContain("Last updated: October 2, 2026");
    }
    expect(render(privacy)).toMatch(/on October 2, 2026 we raised the page limit\s+from 10 to 60/);
  });
});
