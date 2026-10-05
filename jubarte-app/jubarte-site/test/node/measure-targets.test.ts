import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { render } from "../../site/layout.ts";
import { homePage } from "../../site/pages/home.ts";

const css = readFileSync(new URL("../../site/static/css/site.css", import.meta.url), "utf8");
const coarse = /@media \(pointer: coarse\) \{([\s\S]*?)\n\}/.exec(css)?.[1] ?? "";

describe("a readable measure and touch-sized targets", () => {
  it("holds Home's strip caption to the notes' 75ch measure, not the full width", () => {
    const html = render(homePage({ jubarte: "j", soffice: "s" }));
    expect(html).toMatch(/<p class="[^"]*\bstrip-note\b[^"]*">Page one of a document/);
    expect(css).toMatch(/\.strip-note \{[^}]*max-width: 75ch;/);
  });

  it("gives the swap button a 44px square under a coarse pointer", () => {
    // min-width/min-height, not width/height: the base .swap-btn rule (34px) comes
    // later in the file and would win a same-specificity tie.
    expect(coarse).toMatch(/\.swap-btn \{[^}]*min-width: 44px;[^}]*min-height: 44px;/);
  });

  it("gives the Cases page-count buckets a 44px height under a coarse pointer", () => {
    expect(coarse).toMatch(/\.buckets button/);
  });
});
