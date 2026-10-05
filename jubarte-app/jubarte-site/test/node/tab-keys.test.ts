import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { render } from "../../site/layout.ts";
import { demo } from "../../site/pages/demo.ts";
import { homePage } from "../../site/pages/home.ts";

const js = (name: string) =>
  readFileSync(new URL(`../../site/static/js/${name}`, import.meta.url), "utf8");

describe("tab sets follow the roving-tabindex pattern from the first paint", () => {
  it("keeps Demo's unselected Convert tab out of the Tab order before any script runs", () => {
    expect(render(demo)).toMatch(
      /<button[^>]*id="tab-convert"[^>]*aria-selected="false" tabindex="-1">/,
    );
  });

  it("lets Tab reach an installer panel, which holds no focusable element of its own", () => {
    const html = render(homePage({ jubarte: "j", soffice: "s" }));
    const panels = [...html.matchAll(/<div class="installer-panel"[^>]*>/g)].map((m) => m[0]);
    expect(panels.length).toBeGreaterThan(1);
    for (const p of panels) expect(p).toContain('tabindex="0"');
  });

  // What each key does is tested on tabKey itself (test/unit/tabs.test.ts);
  // here, that both tab sets hand their keys to it rather than to their own copy.
  for (const [file, select] of [
    ["home.js", "showInstall(to, true)"],
    ["demo.js", "showTab(tabOrder[to], true)"],
  ] as const) {
    it(`${file} selects the tab tabKey names`, () => {
      const src = js(file);
      expect(src).toContain('import { tabKey } from "./tabs.js";');
      expect(src).toContain(select);
      expect(src).not.toMatch(/e\.key === "(Home|End|ArrowRight|ArrowLeft)"/);
    });
  }
});
