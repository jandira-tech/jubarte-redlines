import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { WHALE_BODY, WHALE_FIN } from "../../site/layout.ts";

// One icon for the Mac app and jubarte.pro's favicon: ../assets/icon.svg
// (`pnpm icons` renders the app's set; build.ts writes the favicons).
const icon = readFileSync(new URL("../../../assets/icon.svg", import.meta.url), "utf8");

describe("the app icon", () => {
  it("is the site's whale mark, so the Dock and the nav show one whale", () => {
    expect(icon).toContain(`d="${WHALE_BODY}"`);
    expect(icon).toContain(`d="${WHALE_FIN}"`);
  });

  it("is drawn in the Night colours, on macOS's plate", () => {
    for (const colour of ["#1A2735", "#0B121A", "#7FC4F0", "#EEF3F7"])
      expect(icon).toContain(colour);
    expect(icon).toContain('<rect x="100" y="100" width="824" height="824" rx="185"/>');
  });

  it("is the favicon the build writes, cropped to its plate so it fills a tab's square", () => {
    const built = readFileSync(new URL("../../public/favicon.svg", import.meta.url), "utf8");
    // macOS draws an icon's plate inside a margin; a browser tab does not, so
    // the favicon drops it: the 824 px plate is the whole viewBox.
    expect(built).toContain('viewBox="100 100 824 824"');
    expect(built.replace('viewBox="100 100 824 824"', 'viewBox="0 0 1024 1024"')).toBe(icon);
  });
});
