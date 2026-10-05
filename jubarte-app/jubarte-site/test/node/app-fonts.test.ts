import { existsSync, readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const src = new URL("../../../src/", import.meta.url);
const read = (file: string) => readFileSync(new URL(file, src), "utf8");

describe("the Mac app's fonts", () => {
  it("are bundled, so launching the app fetches nothing", () => {
    // The download page promises the app goes online only to confirm a
    // purchase; Google Fonts would be a request on every launch.
    const html = read("index.html");
    expect(html).not.toMatch(/fonts\.(googleapis|gstatic)\.com/);
    expect(html).not.toMatch(/<(link|script)[^>]+(href|src)="https?:/);
    expect(html).toContain('<link rel="stylesheet" href="fonts.css" />');
  });

  it("load every face the styles name, from files beside the app", () => {
    const css = read("fonts.css");
    const files = [...css.matchAll(/url\("([^"]+)"\)/g)].map((m) => m[1]);
    expect(files).toHaveLength(6);
    for (const file of files) {
      expect(file, file).toMatch(/^fonts\/[a-z0-9-]+\.woff2$/);
      expect(existsSync(new URL(file, src)), file).toBe(true);
    }
    const families = new Set([...css.matchAll(/font-family: "([^"]+)"/g)].map((m) => m[1]));
    expect([...families]).toEqual(["Manrope", "JetBrains Mono", "Source Serif 4"]);
    for (const family of families) expect(read("styles.css")).toContain(`"${family}"`);
  });

  it("ship with their licenses", () => {
    for (const name of ["manrope", "jetbrains-mono", "source-serif-4"]) {
      expect(read(`fonts/LICENSE-${name}.txt`)).toMatch(/SIL Open Font License/i);
    }
  });
});
