// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

// Refuses to publish an incomplete package. npm runs it before `npm publish`
// (package.json "prepublishOnly"); CI and scripts/release.sh run it too.
//
// The built files are not tracked, and npm drops a `files` entry that is
// missing without a word. So a package built with `build-npm.sh --only`, or
// one whose build skipped a target, would publish and be short. Every
// target's four files, the engine stamp and the license files must be on
// disk, and package.json must list each of them.

import { existsSync, readFileSync } from "node:fs";

const here = new URL("./", import.meta.url);
const targets = ["node", "web", "node-slim", "web-slim", "node-redlines", "web-redlines", "node-pdf", "web-pdf"];
const built = ["jubarte_wasm.js", "jubarte_wasm.d.ts", "jubarte_wasm_bg.wasm", "jubarte_wasm_bg.wasm.d.ts"];
const single = ["ENGINE_COMMIT.txt", "NOTICE", "LICENSE-OFL-1.1.txt"];

const listed = new Set(JSON.parse(readFileSync(new URL("package.json", here), "utf8")).files ?? []);
const problems = [];
for (const target of targets) {
  if (!listed.has(`${target}/`)) problems.push(`package.json "files" does not list ${target}/`);
  for (const name of [...built, "package.json"]) {
    if (!existsSync(new URL(`${target}/${name}`, here))) problems.push(`${target}/${name} is missing`);
  }
}
for (const name of single) {
  if (!listed.has(name)) problems.push(`package.json "files" does not list ${name}`);
  if (!existsSync(new URL(name, here))) problems.push(`${name} is missing`);
}
// npm always packs LICENSE and README.md.
for (const name of ["LICENSE", "README.md"]) {
  if (!existsSync(new URL(name, here))) problems.push(`${name} is missing`);
}
if (existsSync(new URL("ENGINE_COMMIT.txt", here))) {
  const stamp = readFileSync(new URL("ENGINE_COMMIT.txt", here), "utf8").trim();
  if (!/^[0-9a-f]{40}$/.test(stamp)) problems.push("ENGINE_COMMIT.txt does not hold a full commit hash");
}

if (problems.length > 0) {
  console.error("jubarte-wasm: this package is incomplete and must not be published:");
  for (const problem of problems) console.error(`  ${problem}`);
  console.error("Build every target from a clean tree with jubarte-wasm/build-npm.sh (no --only).");
  process.exit(1);
}
console.log(`jubarte-wasm: ${targets.length} targets complete`);
