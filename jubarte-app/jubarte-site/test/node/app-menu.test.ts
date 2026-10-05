import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { isOutside } from "../../../src/about.js";
import { ACTIONS } from "../../../src/menu.js";

const app = (file: string) => readFileSync(new URL(`../../../${file}`, import.meta.url), "utf8");
const html = app("src/index.html");
// The menu itself, without its tests (they name ids too).
const menuRs = app("src-tauri/src/menu.rs").split("#[cfg(test)]")[0];
const ids = (re: RegExp) => [...menuRs.matchAll(re)].map((m) => m[1]);

describe("the Mac app's menu bar", () => {
  it("has the window act on every item menu.rs sends it, and on nothing else", () => {
    const sent = ids(/Action\(\s*"([a-z-]+)"/g);
    expect(sent.length).toBeGreaterThan(15);
    expect(Object.keys(ACTIONS).sort()).toEqual([...sent].sort());
    // Links open outside the app from Rust; the window never sees them.
    for (const link of ids(/Link\("([a-z-]+)"/g)) expect(ACTIONS).not.toHaveProperty(link);
  });

  it("presses only controls the window has", () => {
    const js = app("src/menu.js");
    const pressed = [...js.matchAll(/press\("([a-z-]+)"\)/g)].map((m) => m[1]);
    pressed.push("mode-redline", "mode-convert");
    for (const id of new Set(pressed)) expect(html, id).toContain(`id="${id}"`);
  });

  it("loads the menu and About scripts as modules", () => {
    expect(html).toContain('<script type="module" src="about.js"></script>');
    expect(html).toContain('<script type="module" src="menu.js"></script>');
  });
});

describe("About Jubarte, the Terms and the Privacy Policy", () => {
  it("are windows of the app, filled by about.js", () => {
    const js = app("src/about.js");
    for (const id of [...js.matchAll(/\$\("([a-z-]+)"\)/g)].map((m) => m[1])) {
      expect(html, id).toContain(`id="${id}"`);
    }
    expect(html).toMatch(/<dialog class="settings about" id="about"/);
    expect(html).toMatch(/<dialog class="settings legal" id="legal"/);
    expect(html).toContain('data-legal="terms"');
    expect(html).toContain('data-legal="privacy"');
  });

  it("open About on its Done button, so Return closes it and opens no link", () => {
    // showModal focuses the first control it finds unless one asks for it;
    // in About that is the Engine link, drawn with a focus ring.
    const about = html.slice(
      html.indexOf('id="about"'),
      html.indexOf("</dialog>", html.indexOf('id="about"')),
    );
    expect(about).toContain('<button type="submit" class="btn cta" autofocus>Done</button>');
    expect(about.match(/autofocus/g)).toHaveLength(1);
  });

  it("send web and mail links out of the app, and keep the rest in", () => {
    expect(isOutside("https://jubarte.pro/terms")).toBe(true);
    expect(isOutside("mailto:support@jubarte.pro")).toBe(true);
    expect(isOutside("#")).toBe(false);
    expect(isOutside("javascript:alert(1)")).toBe(false);
    expect(isOutside(null)).toBe(false);
  });

  it("name the subscription Jubarte PRO and Apple's menu as it is now called", () => {
    expect(html).toContain('<h2 class="paywall-title" id="pw-title">Jubarte PRO</h2>');
    for (const file of ["src/index.html", "src/paywall.js"]) {
      // macOS 15 renamed Apple ID to Apple Account in System Settings.
      expect(app(file), file).not.toContain("Apple ID →");
    }
  });
});
