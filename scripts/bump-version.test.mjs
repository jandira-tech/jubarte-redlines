// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

// The README pin rewrite of scripts/bump-version.mjs: a version string is
// interpolated into a RegExp, so it must be escaped whole — a `$` in it
// anchors the pattern and leaves the pin stale. Run: node --test scripts/
import { test } from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { bumpReadmePins, escapeRegExp } from "./bump-version.mjs";

test("escapeRegExp makes any string match itself, backslash included", () => {
  for (const text of ['1.\\0.$"', "0.12.0", "[a-z]+\\d{2}"]) {
    assert.ok(new RegExp(`^${escapeRegExp(text)}$`).test(text), text);
  }
});

test("a pin holding regex metacharacters is still found and rewritten", () => {
  // Backslash, dot, dollar and quote in one pin (CodeQL #1059,
  // js/incomplete-sanitization): escaping only the dots let `$` anchor the
  // pattern, so the pin was left exactly as it was.
  const hostile = '1.\\0.$"';
  const readme = `jubarte = "${hostile}"\njubarte@${hostile}\n`;
  const bumped = bumpReadmePins(readme, hostile, "2.0.0");
  assert.ok(bumped.includes('jubarte = "2.0.0"'), bumped);
  assert.ok(bumped.includes("jubarte@2.0.0"), bumped);
  assert.ok(!bumped.includes(hostile), bumped);
});

test("an ordinary version moves every README pin", () => {
  const readme = [
    'jubarte = "0.11.0"',
    "jubarte@0.11.0",
    "https://badge.socket.dev/cargo/package/jubarte-redlines/0.11.0",
    'jubarte-redlines = { version = "0.11", default-features = false }',
  ].join("\n");
  assert.equal(
    bumpReadmePins(readme, "0.11.0", "0.12.0"),
    [
      'jubarte = "0.12.0"',
      "jubarte@0.12.0",
      "https://badge.socket.dev/cargo/package/jubarte-redlines/0.12.0",
      'jubarte-redlines = { version = "0.12", default-features = false }',
    ].join("\n"),
  );
});

function stageCodemod(t) {
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "jubarte-bump-"));
  t.after(() => fs.rmSync(scratch, { recursive: true, force: true }));
  const root = path.join(scratch, "real");
  fs.mkdirSync(path.join(root, "scripts"), { recursive: true });
  fs.copyFileSync(fileURLToPath(new URL("./bump-version.mjs", import.meta.url)), path.join(root, "scripts/bump-version.mjs"));
  fs.writeFileSync(path.join(root, "Cargo.toml"), '[package]\nname = "jubarte-redlines"\nversion = "0.12.0"\n');
  fs.writeFileSync(path.join(root, "README.md"), 'jubarte = "0.12.0"\njubarte@0.12.0\n');
  fs.writeFileSync(path.join(root, "gemini-extension.json"), '{"version":"0.12.0"}\n');
  return { scratch, root };
}

for (const flags of [[], ["--preserve-symlinks-main"]]) {
  test(`the codemod bumps through a symlinked directory ${flags.join(" ")}`, (t) => {
    const { scratch, root } = stageCodemod(t);
    const link = path.join(scratch, "linked");
    fs.symlinkSync(root, link, "junction");
    const result = spawnSync(process.execPath, [...flags, path.join(link, "scripts/bump-version.mjs"), "9.9.9"], {
      encoding: "utf8", env: { ...process.env, JUBARTE_RELEASE_SH: "1" },
    });
    assert.equal(result.status, 0, result.stderr);
    assert.match(fs.readFileSync(path.join(root, "Cargo.toml"), "utf8"), /^version = "9\.9\.9"$/m);
    assert.equal(fs.readFileSync(path.join(root, "README.md"), "utf8"), 'jubarte = "9.9.9"\njubarte@9.9.9\n');
    assert.equal(JSON.parse(fs.readFileSync(path.join(root, "gemini-extension.json"), "utf8")).version, "9.9.9");
  });
}

for (const importer of ["eval", "script"]) {
  test(`importing the codemod from ${importer} does not run main`, (t) => {
    const { root } = stageCodemod(t);
    const source = `import ${JSON.stringify(pathToFileURL(path.join(root, "scripts/bump-version.mjs")).href)};`;
    const args = ["--input-type=module", "-e", source];
    if (importer === "script") {
      const entry = path.join(root, "importer.mjs");
      fs.writeFileSync(entry, source);
      args.splice(0, args.length, entry, "9.9.9");
    }
    const result = spawnSync(process.execPath, args, { encoding: "utf8" });
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stdout + result.stderr, "");
    assert.match(fs.readFileSync(path.join(root, "Cargo.toml"), "utf8"), /^version = "0\.12\.0"$/m);
  });
}
