import { readFileSync } from "node:fs";
import { beforeAll, describe, expect, it } from "vitest";
import { fact } from "../../site/data/facts.ts";

// The Mac app previews small documents the moment they are chosen, and a
// preview spends no free use: taking the result (Open, Show in Finder, Save a
// copy) does. Settings › Preview turns the first off.
const read = (path: string) => readFileSync(new URL(`../../../${path}`, import.meta.url), "utf8");
const SETTINGS = new URL("../../../src/settings.js", import.meta.url).href;
const html = read("src/index.html");
const app = read("src/app.js");
const main = read("src-tauri/src/main.rs");
const LIMIT = fact<number>("app.instant_preview_max_bytes");

type Settings = typeof import("../../../src/settings.js");
let mod: Settings;
beforeAll(async () => {
  mod = (await import(/* @vite-ignore */ SETTINGS)) as Settings;
});

describe("the size a document previews at once under", () => {
  it("is a megabyte as Finder counts it, from the facts log", () => {
    expect(LIMIT).toBe(1_000_000);
    expect(mod.sizeLabel(LIMIT)).toBe("1 MB");
    // The label Settings shows before the app reads the fact.
    expect(html).toContain(`<span id="instant-limit">${mod.sizeLabel(LIMIT)}</span>`);
  });

  it("takes every document under it, and nothing when there is none", () => {
    expect(mod.smallEnough([LIMIT - 1], LIMIT)).toBe(true);
    expect(mod.smallEnough([10, LIMIT - 1], LIMIT)).toBe(true);
    expect(mod.smallEnough([10, LIMIT], LIMIT)).toBe(false);
    expect(mod.smallEnough([LIMIT + 1], LIMIT)).toBe(false);
    expect(mod.smallEnough([], LIMIT)).toBe(false);
    // The app could not read the fact: nothing previews on its own.
    expect(mod.smallEnough([10], 0)).toBe(false);
  });

  it("is labelled as Finder labels sizes", () => {
    expect(mod.sizeLabel(1_500_000)).toBe("1.5 MB");
    expect(mod.sizeLabel(640_000)).toBe("640 KB");
  });
});

describe("Settings › Preview", () => {
  it("is on unless the user turned it off", () => {
    for (const raw of [null, "", "not json", "{}", '{"instantPreview":"no"}'])
      expect(mod.readSettings(raw).instantPreview).toBe(true);
    expect(mod.readSettings('{"instantPreview":false}').instantPreview).toBe(false);
  });

  it("is a checkbox in the Settings window that says a preview is free", () => {
    const section = /<section[^>]*aria-labelledby="set-preview">(.*?)<\/section>/s.exec(html)?.[1];
    expect(section).toContain('<input type="checkbox" id="instant-preview" />');
    expect(section).toContain("A preview is free");
  });
});

describe("a free use", () => {
  it("is spent when a result is taken, never when it is made", () => {
    // Making a redline or a PDF writes a preview; no gate, no use.
    const runs = main.slice(
      main.indexOf("async fn create_redline("),
      main.indexOf("fn in_background"),
    );
    expect(runs).not.toMatch(/quota::|metered|is_entitled/);
    expect(runs.match(/output_dir\(&app, PREVIEWS\)/g)).toHaveLength(2);
    // take_result runs the gate.
    const taking = main.slice(main.indexOf("async fn take_result("));
    expect(taking).toContain("quota::try_reserve_free_use");
    expect(taking).toContain("quota::FREE_LIMIT_ERR");
    // The window's runs do not count; its three ways to take a result do.
    for (const fn of ["async function redline()", "async function convert()"]) {
      const body = app.slice(app.indexOf(fn), app.indexOf("\n}\n", app.indexOf(fn)));
      expect(body, fn).not.toContain("noteUse");
    }
    for (const id of ["reveal", "save-copy"]) {
      const handler = app.slice(app.indexOf(`$("${id}").addEventListener`));
      expect(handler.slice(0, handler.indexOf("\n});")), id).toContain("await take(");
    }
    expect(app).toMatch(
      /const openResult = async \(\) => \{\n\s+const r = current\(\) && \(await take\(current\(\)\)\);/,
    );
  });

  it("is never offered by a preview made on its own", () => {
    const auto = app.slice(app.indexOf("async function previewAtOnce()"));
    const body = auto.slice(0, auto.indexOf("\n}\n"));
    expect(body).toContain("window.jubarteSettings?.previewAtOnce(");
    // No subscription and no free use left: the user presses the button,
    // which is what opens the paywall.
    expect(body).toMatch(
      /if \(!access\?\.entitled && !\(access\?\.quota\?\.remaining > 0\)\) return;/,
    );
  });

  it("is what the Terms count", () => {
    const terms = fact<{ html: string }>("legal.terms.purchase-and-subscription").html;
    expect(terms).toContain("a redline or a PDF conversion that you open, show in Finder or\nsave");
    expect(terms).toContain("a preview in the app's window is free");
  });

  it("keeps the window's PDFs readable and nothing else", () => {
    const conf = JSON.parse(read("src-tauri/tauri.conf.json"));
    expect(conf.app.security.assetProtocol.scope).toEqual(["$APPCACHE/previews/conversions/*"]);
  });
});
