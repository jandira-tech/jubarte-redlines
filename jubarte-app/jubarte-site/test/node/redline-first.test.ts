import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { render } from "../../site/layout.ts";
import { benchmarkPage } from "../../site/pages/benchmark.ts";
import { useCases } from "../../site/pages/cases.ts";
import { homePage } from "../../site/pages/home.ts";

const precedes = (html: string, a: string, b: string) => {
  expect(html.indexOf(a), a).toBeGreaterThan(-1);
  expect(html.indexOf(b), b).toBeGreaterThan(-1);
  return html.indexOf(a) < html.indexOf(b);
};

// Lawyers first (PRODUCT.md; critique A, P1): the redline leads wherever the
// site shows both benches.
describe("redlines first", () => {
  it("Cases opens on Compare (redline) when no case is named", () => {
    const html = render(useCases);
    expect(html).toContain('id="bench-redline" data-bench="redline" aria-pressed="true"');
    expect(html).toContain('id="bench-convert" data-bench="convert" aria-pressed="false"');
    expect(precedes(html, 'id="bench-redline"', 'id="bench-convert"')).toBe(true);
    const js = readFileSync(new URL("../../site/static/js/cases.js", import.meta.url), "utf8");
    expect(js).toContain('setBench(initial?.bench ?? "redline"');
  });

  it("Benchmark puts the redline headlines and tables before the converter's", () => {
    const html = render(benchmarkPage({ bench: "convert", id: "x" }));
    const redline = html.indexOf("Redline vs Word compare · median");
    expect(redline).toBeGreaterThan(-1);
    // Both sample headlines are on the page (release_info/results_conversion +
    // results_redline); the redline one leads.
    const convert = html.indexOf("DOCX → PDF · median");
    expect(convert).toBeGreaterThan(-1);
    expect(redline).toBeLessThan(convert);
    expect(precedes(html, 'id="redlines-sample"', 'id="docxide-metrics"')).toBe(true);
    expect(precedes(html, 'id="accept-reject"', 'id="docxide-metrics"')).toBe(true);
  });

  it("Home's scoreboard leads with the redline bars", () => {
    const html = render(homePage({ jubarte: "jubarte", soffice: "LibreOffice" }));
    expect(
      precedes(
        html,
        "Redlines vs Word's compare, opened in Word",
        "DOCX → PDF vs Word's own export",
      ),
    ).toBe(true);
  });
});
