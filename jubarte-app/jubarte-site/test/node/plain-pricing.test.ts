import { describe, expect, it } from "vitest";
import { APP_STORE } from "../../site/data/release.ts";
import { render } from "../../site/layout.ts";
import { download } from "../../site/pages/download.ts";
import { homePage } from "../../site/pages/home.ts";

describe("plain pricing, and a path for Windows", () => {
  it("puts the price beside Home's Download for Mac", () => {
    const html = render(homePage({ jubarte: "j", soffice: "s" }));
    expect(html).toContain(`<span>Mac app ${APP_STORE.price}</span>`);
    expect(html).toContain("<span>Browser demo free</span>");
  });

  it("frames the random strip honestly: a loss can show", () => {
    const html = render(homePage({ jubarte: "j", soffice: "s" }));
    expect(html).toContain(
      "The pick is random, not chosen: where jubarte does worse than LibreOffice",
    );
  });

  it("sends Windows and locked-down firm machines to the browser demo", () => {
    const html = render(download);
    expect(html).toMatch(/On Windows, or on a firm laptop[^<]*<a href="\/demo">/);
  });
});
