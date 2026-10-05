import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { render } from "../../site/layout.ts";
import { homePage } from "../../site/pages/home.ts";

const html = render(homePage({ jubarte: "jubarte 0.10.1", soffice: "LibreOffice 26.8.0.3" }));
const js = readFileSync(new URL("../../site/static/js/home.js", import.meta.url), "utf8");
const css = readFileSync(new URL("../../site/static/css/site.css", import.meta.url), "utf8");
const block = (selector: string) => {
  const at = css.indexOf(`\n${selector} {`);
  return at < 0 ? "" : css.slice(at, css.indexOf("}", at));
};

describe("the home page's benchmark feed", () => {
  it("shows a refresh mark beside its pause button, hidden from screen readers", () => {
    const head =
      /<div class="section-head"><h2>From the benchmark.*?<\/div>/s.exec(html)?.[0] ?? "";
    expect(head).toMatch(/<span class="feed-refresh" id="feed-refresh" aria-hidden="true"><svg/);
    expect(head.indexOf('id="feed-refresh"')).toBeLessThan(head.indexOf('id="feed-pause"'));
  });

  it("turns the mark while the next case loads and settles it upright", () => {
    expect(js).toContain('refresh.classList.add("turning")');
    // It stops at the end of a turn, never mid-spin.
    expect(js).toMatch(/addEventListener\(\s*"animationiteration"/);
    expect(js).toContain('refresh.classList.remove("turning")');
    expect(block(".feed-refresh.turning svg")).toContain("animation: jb-turn");
    expect(css).toMatch(/@keyframes jb-turn \{\s*to \{\s*transform: rotate\(360deg\);/);
  });

  it("dims the mark while the feed is held, and drops it under reduced motion", () => {
    expect(js).toContain('refresh.classList.toggle("held", held)');
    expect(block(".feed-refresh.held")).toContain("opacity:");
    // Under reduced motion the feed does not move on, so nothing refreshes.
    expect(js).toMatch(/if \(still\) \{\s*hold\.hidden = true;\s*refresh\.hidden = true;/);
  });
});

describe("the home page's whale", () => {
  it("swims faintly behind the hero, under its words", () => {
    const hero = /<section class="hero">.*?<\/section>/s.exec(html)?.[0] ?? "";
    expect(hero).toContain('<svg class="watermark hero-mark"');
    expect(block(".hero")).toContain("position: relative");
    expect(block(".hero-grid")).toContain("z-index: 1");
    const mark = block(".watermark.hero-mark");
    expect(mark).toContain("opacity: 0.035");
    expect(mark).toContain("left:");
  });
});
