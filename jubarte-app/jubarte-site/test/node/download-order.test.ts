import { describe, expect, it } from "vitest";
import { APP_STORE } from "../../site/data/release.ts";
import { render } from "../../site/layout.ts";
import { download } from "../../site/pages/download.ts";

const html = render(download);
const at = (needle: string) => {
  const i = html.indexOf(needle);
  expect(i, needle).toBeGreaterThan(-1);
  return i;
};

describe("the download page", () => {
  it("speaks to the lawyer first: the redline, privacy, no Word needed", () => {
    expect(html).toContain('<h1 class="h1">Redlines on your Mac, as Word’s own.</h1>');
    expect(html).not.toContain("Pick your surface");
    const lead = /<p class="lead">(.*?)<\/p>/s.exec(html)?.[1] ?? "";
    expect(lead).toContain("tracked changes");
    expect(lead).toContain("never leave your Mac");
    expect(lead).toContain("Microsoft Word is not needed");
  });

  it("offers the Mac app through the App Store only: no DMG, no notify-me for one", () => {
    // Owner's decision after 0.11.2: the Developer ID build is not distributed.
    expect(html).not.toMatch(/DMG|Direct download|Developer ID build/);
    expect(html).toContain(`href="${APP_STORE.url}"`);
  });

  it("puts the Mac app's technical details right after the Mac app, before the engine", () => {
    const mac = at("<h2>Mac app</h2>");
    const tech = at('<h2 id="mac-tech">Technical details</h2>');
    const dev = at('<h2 id="for-developers">For developers and IT</h2>');
    expect(mac).toBeLessThan(tech);
    expect(tech).toBeLessThan(dev);
    expect(dev).toBeLessThan(at("Engine: jubarte-redlines"));
  });

  it("lists the facts IT asks about, from the release data", () => {
    const spec = /<dl class="spec"[^>]*>(.*?)<\/dl>/s.exec(html)?.[1] ?? "";
    const terms = [...spec.matchAll(/<dt>([^<]+)<\/dt>/g)].map((m) => m[1]);
    expect(terms).toEqual([
      "Requires",
      "Installs from",
      "Opens",
      "Writes",
      "Your documents",
      "Network",
      "Account",
      "Version",
    ]);
    expect(spec).toContain(`${APP_STORE.minOs} or later`);
    expect(spec).toContain(`Jubarte ${APP_STORE.version}`);
  });
});
