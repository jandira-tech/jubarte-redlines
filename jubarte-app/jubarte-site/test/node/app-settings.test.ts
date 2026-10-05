import { readFileSync } from "node:fs";
import { beforeAll, describe, expect, it } from "vitest";

// The Mac app's Settings (../src/settings.js). The app has no test runner of
// its own, so its pure functions and its markup are pinned here.
const SETTINGS = new URL("../../../src/settings.js", import.meta.url).href;
const read = (path: string) => readFileSync(new URL(`../../../${path}`, import.meta.url), "utf8");
const html = read("src/index.html");
const css = read("src/styles.css");
const settingsJs = read("src/settings.js");

type Settings = typeof import("../../../src/settings.js");
let mod: Settings;
beforeAll(async () => {
  mod = (await import(/* @vite-ignore */ SETTINGS)) as Settings;
});

describe("readSettings", () => {
  it("defaults to Conventional marks, the document's author and the system appearance", () => {
    for (const raw of [null, undefined, "", "not json", "[]", "42", "null"]) {
      const s = mod.readSettings(raw);
      expect(s.marks).toBe("conventional");
      expect(s.palette).toEqual(mod.CONVENTIONAL);
      expect(s.author).toEqual({ mode: "document", name: "" });
      expect(s.appearance).toBe("system");
    }
  });

  it("keeps what is valid and replaces what is not", () => {
    const s = mod.readSettings(
      JSON.stringify({
        marks: "custom",
        palette: {
          inserted: { color: "#1a2b3c", line: "double-underline" },
          deleted: { color: "red", line: "wavy" },
          movedTo: { color: "#00FF00" },
        },
        author: { mode: "fixed", name: "  Counsel  " },
        appearance: "sepia",
      }),
    );
    expect(s.marks).toBe("custom");
    expect(s.palette.inserted).toEqual({ color: "#1A2B3C", line: "double-underline" });
    expect(s.palette.deleted).toEqual(mod.CONVENTIONAL.deleted);
    expect(s.palette.movedFrom).toEqual(mod.CONVENTIONAL.movedFrom);
    expect(s.palette.movedTo).toEqual({ color: "#00FF00", line: "double-underline" });
    expect(s.author).toEqual({ mode: "fixed", name: "Counsel" });
    expect(s.appearance).toBe("system");
    expect(mod.readSettings('{"marks":"sepia"}').marks).toBe("conventional");
  });

  it("falls back to the document's author when the fixed name is blank, and caps it", () => {
    expect(mod.readSettings('{"author":{"mode":"fixed","name":"   "}}').author.mode).toBe(
      "document",
    );
    const long = mod.readSettings(
      JSON.stringify({ author: { mode: "fixed", name: "x".repeat(400) } }),
    );
    expect(long.author.name).toHaveLength(mod.MAX_TEXT);
  });
});

describe("what a conversion is called with", () => {
  it("passes a preset by name and no palette", () => {
    expect(mod.convertArgs(mod.readSettings(null))).toEqual({
      revisions: "conventional",
      revisionPalette: null,
    });
    expect(mod.convertArgs(mod.readSettings('{"marks":"word"}'))).toEqual({
      revisions: "word",
      revisionPalette: null,
    });
  });

  it("passes custom marks as the engine's palette spec", () => {
    const s = mod.readSettings('{"marks":"custom"}');
    expect(mod.convertArgs(s)).toEqual({
      revisions: "custom",
      // The spec the Rust test convert_paints_settings_custom_marks converts.
      revisionPalette:
        "inserted=#0000FF:underline,deleted=#FF0000:strike,moved-from=#008000:double-strike,moved-to=#008000:double-underline",
    });
  });
});

describe("markVars", () => {
  it("leaves the presets to the stylesheet", () => {
    expect(mod.markVars(mod.readSettings(null))).toEqual({});
    expect(mod.markVars(mod.readSettings('{"marks":"word"}'))).toEqual({});
  });

  it("draws every custom mark in its colour and line", () => {
    const s = mod.readSettings(
      JSON.stringify({
        marks: "custom",
        palette: { inserted: { color: "#123456", line: "plain" } },
      }),
    );
    const vars = mod.markVars(s);
    expect(Object.keys(vars)).toHaveLength(12);
    expect(vars["--ins"]).toBe("light-dark(#123456, color-mix(in oklch, #123456 45%, white))");
    expect(vars["--ins-bg"]).toBe("color-mix(in oklch, #123456 14%, transparent)");
    expect(vars["--ins-mark"]).toBe("none");
    expect(vars["--del-mark"]).toBe("line-through");
    expect(vars["--movfrom-mark"]).toBe("line-through double");
    expect(vars["--movto-mark"]).toBe("underline double");
  });

  it("names every line as Word's Track Changes options do", () => {
    expect(Object.keys(mod.LINE_LABEL)).toEqual(mod.LINES);
    expect(mod.LINE_LABEL.strike).toBe("Strikethrough");
  });
});

describe("the Settings window", () => {
  it("opens from the title bar and with ⌘,", () => {
    expect(html).toMatch(/<button[^>]+id="open-settings"[^>]+aria-label="Settings"/);
    expect(html).toContain("Settings (⌘,)");
    expect(html).toContain('<script type="module" src="settings.js">');
  });

  it("orders the groups by how often they are used: marks, author, appearance", () => {
    const at = (id: string) => html.indexOf(`id="${id}"`);
    expect(at("set-marks")).toBeGreaterThan(0);
    expect(at("set-marks")).toBeLessThan(at("set-revisions"));
    expect(at("set-revisions")).toBeLessThan(at("set-appearance"));
  });

  it("offers every mark, and every kind of change a colour and a line", () => {
    for (const value of ["conventional", "word", "custom"]) {
      expect(html).toContain(`<input type="radio" name="marks" value="${value}"`);
      // Settings is the one place the marks are chosen.
      expect(html).not.toMatch(new RegExp(`<option value="${value}"`));
    }
    for (const kind of mod.KINDS) {
      expect(html).toContain(`<input type="color" data-kind="${kind}"`);
      // Each line is a choice drawn as it marks text, not a menu of names.
      expect(html).toContain(`<div class="lines" role="radiogroup" data-lines="${kind}"`);
      expect(html).not.toContain(`<select data-kind="${kind}"`);
    }
  });

  it("names each line as Word does, on the sample it draws", () => {
    expect(Object.keys(mod.LINE_LABEL)).toEqual(mod.LINES);
    expect(settingsJs).toMatch(/textContent = "Aa"/);
  });

  it("heads each group as the window's labels are set: small mono capitals", () => {
    const rule = css.match(/\.settings-group h3 \{([^}]*)\}/)?.[1] ?? "";
    expect(rule).toContain("var(--font-mono)");
    expect(rule).toContain("text-transform: uppercase");
  });

  it("says what each appearance does to the pages", () => {
    expect(mod.appearanceNote("system")).toBe(
      "Follows macOS. Document pages stay white, as Word prints them.",
    );
    expect(mod.appearanceNote("dark")).toBe(
      "A dark window. Document pages stay white, as Word prints them.",
    );
    expect(mod.appearanceNote("light")).toBe("A light window, whatever macOS uses.");
    expect(html).toContain('id="appearance-note"');
  });

  it("tells the fingerprint's limits before they cut it", () => {
    const note = html.match(/id="fingerprint-note">([^<]*)</)?.[1] ?? "";
    expect(note).toContain(`One line, at most ${mod.MAX_TEXT} characters.`);
  });

  it("describes Conventional by its marks, not by another product", () => {
    const conventional = html.match(/<strong>Conventional<\/strong>([^<]*)</)?.[1] ?? "";
    expect(conventional).toContain("underlined in blue");
    expect(html).not.toMatch(/litera/i);
  });

  it("caps the fixed name at a document property's length", () => {
    expect(html).toContain(`id="author-name" maxlength="${mod.MAX_TEXT}"`);
  });

  it("offers agents a fingerprint field under Revisions by, explained in Word's words", () => {
    const at = (needle: string) => html.indexOf(needle);
    expect(at('id="fingerprint"')).toBeGreaterThan(at('id="set-revisions"'));
    expect(at('id="fingerprint"')).toBeLessThan(at('id="set-appearance"'));
    expect(html).toContain(`id="fingerprint" maxlength="${mod.MAX_TEXT}"`);
    expect(html).toContain('aria-describedby="fingerprint-note"');
    expect(html).toContain("custom document property AgentFingerprint");
    expect(html).toContain("File › Properties › Custom");
  });
});

describe("the agent fingerprint", () => {
  it("is stored as one clean line and read back", () => {
    expect(mod.readSettings(null).fingerprint).toBe("");
    expect(mod.readSettings('{"fingerprint":42}').fingerprint).toBe("");
    expect(
      mod.readSettings(JSON.stringify({ fingerprint: "  agent=claude\nrun=7  " })).fingerprint,
    ).toBe("agent=claude run=7");
  });

  it("cleans text as fingerprint.rs clean does", () => {
    // The same cases as the Rust test the_value_is_one_escaped_line_of_at_most_255_characters.
    expect(mod.cleanText("  a\tb\nc\u0000d\uFFFF  ")).toBe("a b cd");
    expect(Array.from(mod.cleanText("é".repeat(300)))).toHaveLength(mod.MAX_TEXT);
    // Characters, not UTF-16 units: an emoji is never cut in half.
    expect(mod.cleanText("😀".repeat(300))).toBe("😀".repeat(mod.MAX_TEXT));
  });
});
