#!/usr/bin/env bun

// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

// ###########################################################################
// ##                                                                       ##
// ##   WARNING — scripts/release.sh IS THE SOURCE OF TRUTH FOR RELEASES.   ##
// ##                                                                       ##
// ##   This script is only the Cargo.toml + README codemod that            ##
// ##   release.sh calls in its version-sync step. On its own it does NOT   ##
// ##   bump jubarte-python, jubarte-wasm/npm or the four Cargo.lock        ##
// ##   files, write the six required release notes, run the gates, tag,   ##
// ##   push or publish. A version bumped here alone is a half release.     ##
// ##                                                                       ##
// ##   Release with:  scripts/release.sh x.y.z --changelog-summary "…" …   ##
// ##                  (see VERSIONING.md step 8; --dry-run rehearses it)   ##
// ##                                                                       ##
// ###########################################################################
//
// Bump the crate version in every hard-coded location, in one shot.
//
//   bun scripts/bump-version.mjs 0.2.0
//
// Touches: Cargo.toml ([package] version), the README's version pins,
// including the Socket badge (badge.socket.dev/cargo/package/jubarte-redlines/<version>),
// and gemini-extension.json's version.
// CHANGELOG.md is NOT auto-written — add the Keep-a-Changelog section
// yourself, then let scripts/release.sh commit, tag and publish.
// See VERSIONING.md.

import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

// release.sh sets JUBARTE_RELEASE_SH=1 when it calls this codemod; any other
// caller is running half a release and gets the banner.
const viaRelease = process.env.JUBARTE_RELEASE_SH === "1";
const banner = [
  "",
  "#".repeat(75),
  "##  WARNING: scripts/release.sh IS THE SOURCE OF TRUTH FOR RELEASES.",
  "##",
  "##  bump-version.mjs only rewrites Cargo.toml and the README pins. It does",
  "##  NOT sync jubarte-python, jubarte-wasm/npm or the Cargo.lock files, write",
  "##  the required release notes, run the gates, tag, push or publish.",
  "##",
  "##  Release with:  scripts/release.sh x.y.z --changelog-summary \"…\" …",
  "##                 (VERSIONING.md step 8; --dry-run rehearses it)",
  "#".repeat(75),
  "",
].join("\n");
if (!viaRelease) console.error(banner);
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
    )
    // Socket badge: image and link both pin the version; a stale pin
    // (a skipped bump) is corrected too, not only the previous version.
    .replace(
      /(badge\.socket\.dev\/cargo\/package\/jubarte-redlines\/)\d+\.\d+\.\d+/g,
      `$1${next}`,
    )
    // Library install pin: `version = "x.y"` (a caret requirement), so a
    // patch leaves it and a minor moves it. It stayed "0.9" through 0.10.1.
    .replace(
      /(jubarte-redlines\s*=\s*\{\s*version\s*=\s*")\d+\.\d+(")/g,
      `$1${next.split(".").slice(0, 2).join(".")}$2`,
    );
  if (readmeNext !== readme) writeFileSync(readmePath, readmeNext);
} catch {
  /* no README or no pins */
}

// Gemini CLI extension manifest: its version is the release's.
const geminiPath = join(root, "gemini-extension.json");
try {
  const gemini = readFileSync(geminiPath, "utf8");
  const geminiNext = gemini.replace(
    /("version"\s*:\s*")\d+\.\d+\.\d+(")/,
    `$1${next}$2`,
  );
  if (geminiNext !== gemini) writeFileSync(geminiPath, geminiNext);
} catch {
  /* no manifest */
}

console.log(`bumped jubarte ${prev} → ${next}`);
if (!viaRelease) {
  console.error(banner);
  console.error(
    `Do not commit or tag this by hand: run scripts/release.sh ${next} with its six required notes.`,
  );
}
