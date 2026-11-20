// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

// The published manifest: both command names run the one entry point (plan
// T5.2). `jubarte` was checked free on npm before it was added (`npm view
// jubarte` answered E404); a bin name is per install, so it names no other
// package either way.
import { test } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const cliDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const manifest = JSON.parse(fs.readFileSync(path.join(cliDir, "package.json"), "utf8"));

test("the package installs both `jubarte` and `jubarte-redlines` commands on the one entry point", () => {
  assert.deepEqual(manifest.bin, {
    jubarte: "bin/jubarte-redlines.mjs",
    "jubarte-redlines": "bin/jubarte-redlines.mjs",
  });
  for (const target of new Set(Object.values(manifest.bin))) {
    assert.ok(fs.existsSync(path.join(cliDir, target)), `${target} exists`);
    assert.ok(manifest.files.some((entry) => target.startsWith(entry)), `${target} ships in files`);
  }
});
