import { describe, expect, it } from "vitest";
import { APP_IN_REVIEW, APP_RELEASE, appInReview } from "../../site/data/release.ts";
import { render } from "../../site/layout.ts";
import { pro } from "../../site/pages/app.ts";
import { download } from "../../site/pages/download.ts";

const store = { url: "u", version: "0.7.0", price: "$99.99", minOs: "macOS 12" };

describe("the app's release phase (app.release.* against app_store.version)", () => {
  it("is in review until the store sells the submitted version", () => {
    expect(appInReview({ version: "0.11.2", submitted: "2026-10-03" }, store)).toBe(true);
    expect(
      appInReview({ version: "0.11.2", submitted: "2026-10-03" }, { ...store, version: "0.11.2" }),
    ).toBe(false);
  });

  it("names the version in review on the pages that sell the app, and never 'the next release'", () => {
    for (const page of [download, pro]) {
      const html = render(page);
      expect(html).not.toMatch(/next release/i);
      if (APP_IN_REVIEW) {
        expect(html).toContain(`${APP_RELEASE.version} IN APPLE’S REVIEW`);
      } else {
        expect(html).not.toMatch(/in Apple’s review/i);
      }
    }
  });

  it("shows the submitted version in the app window's title bar", () => {
    expect(render(pro)).toContain(`<span class="mono">v${APP_RELEASE.version}</span>`);
  });
});
