import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { render } from "../../site/layout.ts";
import { pro } from "../../site/pages/app.ts";
import { useCases, useCasesEmbed } from "../../site/pages/cases.ts";
import { demo } from "../../site/pages/demo.ts";

const html = render(demo);
const desktop = /<nav class="nav-links"[^>]*>(.*?)<\/nav>/s.exec(html)?.[1] ?? "";
const menu = /<nav class="nav-menu-list"[^>]*>(.*?)<\/nav>/s.exec(html)?.[1] ?? "";
const links = (nav: string) =>
  [...nav.matchAll(/<a href="([^"]+)"[^>]*>(?:<span>)?([^<]+)/g)].map((m) => [m[2], m[1]]);

describe("the nav", () => {
  it("lists Home first, then the pages alphabetically, PRO last", () => {
    const expected = [
      ["Home", "/"],
      ["Benchmark", "/benchmark"],
      ["Contact", "/contact"],
      ["Download", "/download"],
      ["Live", "/live"],
      ["Try it", "/demo"],
      ["Use cases", "/use-cases"],
      ["PRO", "/pro"],
    ];
    expect(links(desktop)).toEqual(expected);
    expect(links(menu)).toEqual(expected);
    // The middle stays alphabetical as pages come and go.
    const middle = expected.slice(1, -1).map(([label]) => label);
    expect(middle).toEqual([...middle].sort((a, b) => a.localeCompare(b)));
  });

  it("sends Try it where the home page's main button goes", () => {
    const home = readFileSync(new URL("../../site/pages/home.ts", import.meta.url), "utf8");
    expect(home).toContain(
      '<a class="btn btn-primary btn-lg" href="/demo">Try it in your browser →</a>',
    );
    expect(html).toContain('<a href="/demo" aria-current="page">Try it</a>');
  });

  it("inverts PRO, in the bar and in the menu, and boxes nothing else", () => {
    expect(desktop).toContain('<a href="/pro" class="nav-pro">PRO</a>');
    expect(menu).toContain('<a href="/pro" class="nav-pro"><span>PRO</span></a>');
    expect(html).not.toContain("nav-dl");
  });

  it("calls the brand Jubarte, not a file name", () => {
    expect(html).toContain('<span class="brand-word">JUBARTE</span>');
    expect(html).not.toMatch(/DOCX<\/span><\/a>/);
  });

  it("serves PRO and Use cases at their own paths", () => {
    expect([pro.path, pro.file, pro.nav]).toEqual(["/pro", "pro.html", "pro"]);
    expect([useCases.path, useCases.file, useCases.nav]).toEqual([
      "/use-cases",
      "use-cases.html",
      "usecases",
    ]);
    expect([useCasesEmbed.path, useCasesEmbed.file]).toEqual([
      "/use-cases/embed",
      "use-cases/embed.html",
    ]);
    expect(render(useCases)).toContain("<title>Use cases");
  });
});

describe("PRO", () => {
  const css = readFileSync(new URL("../../site/static/css/site.css", import.meta.url), "utf8");
  it("is bold in the bar and in the menu", () => {
    for (const selector of [".nav-links a.nav-pro", ".nav-menu-list a.nav-pro"]) {
      const at = css.indexOf(`\n${selector} {`);
      expect(css.slice(at, css.indexOf("}", at)), selector).toContain("font-weight: 800");
    }
  });
});

describe("the mobile menu", () => {
  const css = readFileSync(new URL("../../site/static/css/site.css", import.meta.url), "utf8");
  const rule = (selector: string) => {
    const at = css.indexOf(`\n${selector} {`);
    return at < 0 ? "" : css.slice(at, css.indexOf("}", at));
  };

  it("hangs flush from the bar's rule, not beside it", () => {
    // It was 10px under the Menu button: its top border ran 5px above the
    // header's bottom rule, two lines side by side with a step between.
    expect(rule(".site-nav .wrap")).toContain("position: relative");
    expect(rule(".nav-menu-list")).toContain("top: 100%");
    expect(rule(".nav-menu-list")).toContain("border-top: 0");
    expect(rule(".nav-menu-list")).toContain("box-shadow:");
    expect(rule(".nav-menu-list")).not.toContain("calc(100% + 10px)");
  });
});
