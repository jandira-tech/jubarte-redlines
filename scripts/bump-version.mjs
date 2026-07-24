#!/usr/bin/env bun
// Bump the crate version in every hard-coded location, in one shot.
//
//   bun scripts/bump-version.mjs 0.2.0
//
// Touches: Cargo.toml ([package] version), CHANGELOG.md is NOT auto-written —
// add the Keep-a-Changelog section yourself, then commit + tag.
// See VERSIONING.md.

import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const next = process.argv[2];
if (!/^\d+\.\d+\.\d+$/.test(next ?? "")) {
  console.error(`usage: bun scripts/bump-version.mjs <x.y.z>   (got: ${next ?? "nothing"})`);
  process.exit(1);
}

const cargoPath = join(root, "Cargo.toml");
const cargo = readFileSync(cargoPath, "utf8");
const m = cargo.match(/^version = "(\d+\.\d+\.\d+)"$/m);
if (!m) {
  console.error(`could not find [package] version in ${cargoPath}`);
  process.exit(1);
}
const prev = m[1];
if (prev === next) {
  console.error(`version is already ${next} — nothing to do`);
  process.exit(1);
}

const cargoNext = cargo.replace(
  /^version = "\d+\.\d+\.\d+"$/m,
  `version = "${next}"`,
);
writeFileSync(cargoPath, cargoNext);

// Optional: README badge / install line if present
const readmePath = join(root, "README.md");
try {
  const readme = readFileSync(readmePath, "utf8");
  const readmeNext = readme
    .replace(
      new RegExp(`jubarte\\s*=\\s*"${prev.replace(/\./g, "\\.")}"`, "g"),
      `jubarte = "${next}"`,
    )
    .replace(
      new RegExp(`jubarte@${prev.replace(/\./g, "\\.")}`, "g"),
      `jubarte@${next}`,
    );
  if (readmeNext !== readme) writeFileSync(readmePath, readmeNext);
} catch {
  /* no README or no pins */
}

console.log(`bumped jubarte ${prev} → ${next}`);
console.log("next:");
console.log("  1. Edit CHANGELOG.md (## [x.y.z] - YYYY-MM-DD)");
console.log("  2. cargo build --release --bin jubarte");
console.log("  3. git commit -am 'chore(release): v" + next + "'");
console.log("  4. git tag -a v" + next + " -m 'v" + next + "'");
console.log("  5. Install binary into neurotic_docx_bench (see VERSIONING.md)");
