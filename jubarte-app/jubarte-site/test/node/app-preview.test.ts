import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

// The Mac app (../src) draws the same redline preview as the site's Demo and
// App pages. It has no test runner of its own, so its marks are pinned here.
const read = (file: string) =>
  readFileSync(new URL(`../../../src/${file}`, import.meta.url), "utf8");
const css = read("styles.css");
const rule = (selector: string) => {
  const at = css.indexOf(`${selector} {`);
  return at < 0 ? "" : css.slice(at, css.indexOf("}", at));
};
// Settings → Custom overrides the marks through these custom properties; the
// stylesheet's :root holds the conventional ones.
const root = rule(":root");
const declared = (name: string) => root.match(new RegExp(`\\s${name}: ([^;]+);`))?.[1];
/** A rule's text-decoration, with its custom property resolved from :root. */
const decoration = (selector: string) => {
  const value = rule(selector).match(/text-decoration: ([^;]+);/)?.[1] ?? "";
  return value.replace(/var\((--[\w-]+)\)/, (_, name) => declared(name) ?? "");
};

describe("the Mac app's redline preview", () => {
  it("has no legend: the chips are the legend", () => {
    expect(read("index.html")).not.toContain('class="legend"');
    expect(css).not.toMatch(/\.legend/);
    expect(decoration(".chip.ins")).toBe("underline");
    expect(decoration(".chip.del")).toBe("line-through");
    expect(decoration(".chip.mov")).toBe("underline double");
  });

  it("marks insertions and deletions once", () => {
    expect(rule(".paper ins")).toContain("color: var(--ins)");
    expect(decoration(".paper ins")).toBe("underline");
    expect(rule(".paper del")).toContain("color: var(--del)");
    expect(decoration(".paper del")).toBe("line-through");
  });

  it("marks a move twice, in green: double-struck where it left, double-underlined where it landed", () => {
    expect(rule(".paper .movedel")).toContain("color: var(--movfrom)");
    expect(rule(".paper .movedel")).toContain("background: var(--movfrom-bg)");
    expect(declared("--movfrom")).toBe("var(--mov)");
    expect(declared("--movfrom-bg")).toBe("var(--mov-bg)");
    expect(decoration(".paper .movedel")).toBe("line-through double");
    expect(rule(".paper .movedel")).not.toContain("opacity");
    expect(rule(".paper .moveins")).toContain("color: var(--movto)");
    expect(declared("--movto")).toBe("var(--mov)");
    expect(decoration(".paper .moveins")).toBe("underline double");
  });

  it("labels no move with where it came from, and counts formatting as formatted", () => {
    const js = read("app.js");
    expect(js).not.toMatch(/moved from|move-note|movedFrom/);
    expect(css).not.toContain("move-note");
    expect(js).toContain('chip("chip-fmt", r.format_changes, "Formatted")');
  });
});
