import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { render } from "../../site/layout.ts";
import { demo } from "../../site/pages/demo.ts";
import { WASM_SIZE } from "../../site/wasm-size.ts";

describe("the Demo's sample pair", () => {
  it("lets a first-timer try the Demo without their own files, and check the privacy claim", () => {
    const html = render(demo);
    expect(html).toContain('id="sample"');
    const js = readFileSync(new URL("../../site/static/js/demo.js", import.meta.url), "utf8");
    // The same bundled pair the App page uses; the build writes both files.
    for (const url of ["/static/demo/msa-v3.docx", "/static/demo/msa-v4.docx"]) {
      expect(js).toContain(`url: "${url}"`);
      expect(readFileSync(new URL(`../../public${url}`, import.meta.url)).length).toBeGreaterThan(
        0,
      );
    }
    // A check a lawyer can run, with the engineering detail behind a disclosure.
    expect(html).toContain("turn off Wi‑Fi");
    const at = html.indexOf('<details class="how-it-runs');
    expect(html.slice(at, html.indexOf("</details>", at))).toContain(WASM_SIZE.slim.raw);
    expect(html.indexOf(WASM_SIZE.slim.raw)).toBeGreaterThan(at);
  });
});
