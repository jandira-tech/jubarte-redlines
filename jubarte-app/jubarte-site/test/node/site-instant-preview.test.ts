import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { fact } from "../../site/data/facts.ts";
import { render } from "../../site/layout.ts";
import { pro } from "../../site/pages/app.ts";
import { demo } from "../../site/pages/demo.ts";
import { KEY, smallEnough } from "../../site/static/js/instant.js";

// The Demo and the App page preview small documents the moment they are in,
// as the Mac app does; a checkbox turns it off for both pages.
const LIMIT = fact<number>("app.instant_preview_max_bytes");
const js = (name: string) =>
  readFileSync(new URL(`../../site/static/js/${name}`, import.meta.url), "utf8");
const box = (id: string) =>
  `<input type="checkbox" id="${id}" data-limit="${LIMIT}" checked><span>Preview as soon as the documents are in, when each is under 1 MB</span>`;

describe("instant preview on jubarte.pro", () => {
  it("offers the switch beside each run button, with the app's size", () => {
    const demoHtml = render(demo);
    expect(demoHtml).toContain(box("instant"));
    expect(demoHtml).toContain(box("c-instant"));
    expect(render(pro)).toContain(box("a-instant"));
  });

  it("previews only when every document is under that size", () => {
    expect(smallEnough([LIMIT - 1, 10], LIMIT)).toBe(true);
    expect(smallEnough([LIMIT - 1, LIMIT], LIMIT)).toBe(false);
    expect(smallEnough([], LIMIT)).toBe(false);
    expect(smallEnough([10], 0)).toBe(false);
    expect(KEY).toBe("jb-instant");
  });

  it("runs the Demo's redline and conversion when they are small", () => {
    const demoJs = js("demo.js");
    expect(demoJs).toContain(
      "if (red.orig && red.mod && previewNow([red.orig.size, red.mod.size])) await createRedline();",
    );
    // After a pick or a drop, and after a swap.
    expect(demoJs.match(/redlineNow\(\);/g)).toHaveLength(2);
    expect(demoJs).toContain("if (previewNow([conv.doc.size])) await renderConvert();");
  });

  it("runs the App page's redline and conversion when they are small", () => {
    const appJs = js("app.js");
    expect(appJs).toContain("if (previewNow([doc.size])) await convert();");
    expect(appJs.match(/previewNow\(\[s\.orig\.size, s\.mod\.size\]\)/g)).toHaveLength(2);
  });
});
