#!/usr/bin/env bun
// Bump the app version in every place it is hard-coded, in one shot.
//
//   bun run bump 0.3.0
//
// Touches: package.json, src-tauri/tauri.conf.json (JSON `.version`),
// src-tauri/Cargo.toml (the [package] `version` line), and src/index.html
// (the app-bar `vX.Y.Z` label). It does NOT write the CHANGELOG — add that
// entry yourself, then commit. See README → "Versioning & release".

import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

const next = process.argv[2];
if (!/^\d+\.\d+\.\d+$/.test(next ?? "")) {
  console.error(`usage: bun run bump <x.y.z>   (got: ${next ?? "nothing"})`);
  process.exit(1);
}

const pkgPath = join(root, "package.json");
const pkg = JSON.parse(readFileSync(pkgPath, "utf8"));
const prev = pkg.version;
if (prev === next) {
  console.error(`version is already ${next} — nothing to do`);
  process.exit(1);
}

// package.json + tauri.conf.json: set the `.version` field (JSON, key order kept).
pkg.version = next;
writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + "\n");

const confPath = join(root, "src-tauri", "tauri.conf.json");
const conf = JSON.parse(readFileSync(confPath, "utf8"));
conf.version = next;
writeFileSync(confPath, JSON.stringify(conf, null, 2) + "\n");

// Cargo.toml: only the standalone [package] `version = "x.y.z"` line (inline
// dependency versions like `version = "2"` are left alone).
const cargoPath = join(root, "src-tauri", "Cargo.toml");
const cargo = readFileSync(cargoPath, "utf8");
const cargoNext = cargo.replace(
  /^version = "\d+\.\d+\.\d+"$/m,
  `version = "${next}"`,
);
if (cargoNext === cargo) throw new Error(`could not find the package version line in ${cargoPath}`);
writeFileSync(cargoPath, cargoNext);

// index.html: the app-bar label `vX.Y.Z`.
const htmlPath = join(root, "src", "index.html");
const html = readFileSync(htmlPath, "utf8");
const htmlNext = html.replace(
  /(id="appbar-ver">)v\d+\.\d+\.\d+(<)/,
  `$1v${next}$2`,
);
if (htmlNext === html) throw new Error(`could not find the app-bar version label in ${htmlPath}`);
writeFileSync(htmlPath, htmlNext);

console.log(`bumped ${prev} → ${next}`);
console.log("next: add a CHANGELOG.md entry, then commit, then `bun run build`.");
