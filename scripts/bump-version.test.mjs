// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

// The README pin rewrite of scripts/bump-version.mjs: a version string is
// interpolated into a RegExp, so it must be escaped whole — a `$` in it
// anchors the pattern and leaves the pin stale. Run: node --test scripts/
import { test } from "node:test";
import assert from "node:assert/strict";
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
