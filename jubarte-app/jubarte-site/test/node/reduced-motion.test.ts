import { readdirSync, readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const css = readFileSync(new URL("../../site/static/css/site.css", import.meta.url), "utf8");
const QUERY = "@media (prefers-reduced-motion: reduce) {";

/** The body of the reduced-motion block, braces balanced. */
function reducedBlock(): string {
  const start = css.indexOf(QUERY);
  expect(start).toBeGreaterThan(-1);
  let depth = 0;
  for (let i = start + QUERY.length - 1; i < css.length; i++) {
    if (css[i] === "{") depth++;
    if (css[i] === "}" && --depth === 0) return css.slice(start + QUERY.length, i);
  }
  throw new Error("unbalanced reduced-motion block");
}

/** Every rule outside the reduced-motion block, as [selectors, declarations]. */
function rules(): [string[], string][] {
  const outside = css.replace(QUERY + reducedBlock(), "").replace(/\/\*[\s\S]*?\*\//g, "");
  return [...outside.matchAll(/([^{}]+)\{([^{}]*)\}/g)].map((m) => [
    m[1].split(",").map((s) => s.trim()),
    m[2],
  ]);
}

const covered = (selector: string) =>
  reducedBlock()
    .replace(/\/\*[\s\S]*?\*\//g, "")
    .split(/[{}]/)
    .filter((_, i) => i % 2 === 0)
    .some((head) => head.split(",").some((s) => s.trim() === selector));

describe("reduced motion, component by component", () => {
  it("no longer flattens every animation and transition in one rule", () => {
    expect(reducedBlock()).not.toMatch(/\*\s*,|0\.001ms/);
  });

  it("stops each animated component by name", () => {
    const animated = rules()
      .filter(([, body]) => /(^|;|\s)animation:/.test(body))
      .flatMap(([sel]) => sel);
    expect(animated.length).toBeGreaterThan(5);
    for (const sel of animated) expect(covered(sel), sel).toBe(true);
  });

  it("drops every transform transition, so nothing slides or spins", () => {
    const moving = rules()
      .filter(([, body]) => /transition:[^;]*transform/.test(body))
      .flatMap(([sel]) => sel);
    expect(moving.length).toBeGreaterThan(0);
    for (const sel of moving) expect(covered(sel), sel).toBe(true);
  });

  it("names only selectors the stylesheet still has", () => {
    const outside = new Set(rules().flatMap(([sel]) => sel));
    const heads = reducedBlock()
      .replace(/\/\*[\s\S]*?\*\//g, "")
      .split(/[{}]/)
      .filter((_, i) => i % 2 === 0)
      .flatMap((head) => head.split(",").map((s) => s.trim()))
      .filter(Boolean);
    expect(heads.length).toBeGreaterThan(5);
    // A ::before of a ruled element, or a descendant of one, needs no rule of its own.
    const base = (sel: string) =>
      sel
        .replace(/::before$/, "")
        .split(" ")
        .pop() ?? sel;
    for (const sel of heads) {
      const known =
        outside.has(sel) || outside.has(base(sel)) || outside.has(sel.replace(/::before$/, ""));
      expect(known, sel).toBe(true);
    }
  });

  it("turns a spinner into a static ellipsis rather than a frozen ring", () => {
    expect(reducedBlock()).toMatch(/\.spinner::before\s*\{[^}]*content:\s*"…"/);
  });

  it("hides a button's spinner, whose busy label already ends in an ellipsis", () => {
    expect(reducedBlock()).toMatch(/\.btn \.spinner\s*\{\s*display:\s*none;/);
    for (const [file, labels] of [
      ["demo.js", ["Redlining…", "Rendering…"]],
      ["app.js", ["Redlining…", "Converting…"]],
    ] as const) {
      const js = readFileSync(new URL(`../../site/static/js/${file}`, import.meta.url), "utf8");
      for (const label of labels) expect(js, file).toContain(`"${label}"`);
    }
  });

  it("leaves no animation in a style attribute, where the stylesheet cannot stop it", () => {
    const pub = new URL("../../public/", import.meta.url);
    const pages = readdirSync(pub, { recursive: true, encoding: "utf8" }).filter((f) =>
      f.endsWith(".html"),
    );
    expect(pages.length).toBeGreaterThan(5);
    for (const page of pages) {
      const html = readFileSync(new URL(page, pub), "utf8");
      expect(html, page).not.toMatch(/style="[^"]*animation/);
    }
  });
});
