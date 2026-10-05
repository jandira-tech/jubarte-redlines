import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { command } from "../../site/command.ts";
import { render } from "../../site/layout.ts";
import { download } from "../../site/pages/download.ts";
import { homePage } from "../../site/pages/home.ts";
import { live } from "../../site/pages/live.ts";

const css = readFileSync(new URL("../../site/static/css/site.css", import.meta.url), "utf8");

describe("polish", () => {
  it("keeps each word of a command whole, and copies as the plain command", () => {
    const html = command("cargo install jubarte-redlines");
    expect(html).toBe(
      '<span class="tok">cargo</span> <span class="tok">install</span> <span class="tok">jubarte-redlines</span>',
    );
    expect(html.replace(/<[^>]+>/g, "")).toBe("cargo install jubarte-redlines");
    expect(command("a <b>\n  c")).toContain('<span class="tok">&lt;b&gt;</span>\n  <span');
    expect(render(download)).toContain('<span class="tok">jubarte-redlines</span>');
    // The command blocks no longer cut words at any character.
    for (const rule of [".code-block", ".cli-echo"]) {
      const bodies = [...css.matchAll(new RegExp(`\\${rule} \\{[^}]*\\}`, "g"))].map((m) => m[0]);
      expect(
        bodies.some((b) => b.includes("overflow-wrap: anywhere")),
        rule,
      ).toBe(true);
      expect(
        bodies.some((b) => b.includes("break-all")),
        rule,
      ).toBe(false);
    }
  });

  it("numbers Live's steps before their text, and pulses nothing on a page that is not live", () => {
    const html = render(live);
    expect(html).toMatch(/<li><span class="phase-n">01<\/span><span><strong>/);
    expect(html).not.toContain("pulse-dot");
    expect(render(homePage({ jubarte: "j", soffice: "s" }))).not.toContain("pulse-dot");
  });

  it("draws no decorative squares or soft card shadows (DESIGN.md)", () => {
    expect(render(homePage({ jubarte: "j", soffice: "s" }))).toContain(
      '<span class="cap-tag">Compare</span>',
    );
    const frame = /\.browser-frame \{[^}]*\}/.exec(css)?.[0] ?? "";
    expect(frame).not.toContain("box-shadow");
  });
});
