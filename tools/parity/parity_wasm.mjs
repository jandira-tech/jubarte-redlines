#!/usr/bin/env node
// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//
// WASM side of the jubarte parity check (driven by parity_check.py).
//
//   node parity_wasm.mjs --module <jubarte_wasm.js | dir> --jobs jobs.json --out results.json
//
// --module accepts:
//   - the jubarte_wasm.js file itself (wasm-pack --target nodejs output, or
//     jubarte-wasm/npm/node/jubarte_wasm.js — the auto-initializing Node CJS
//     build documented in jubarte-wasm/npm/README.md "Usage — Node" and
//     docs/javascript.md, and exercised the same way by npm-smoke.mjs);
//   - a directory holding it: jubarte-wasm/npm, jubarte-wasm/pkg,
//     node_modules/jubarte-wasm, or a scratch dir with node_modules/ inside.
//
// jobs.json (same schema the Python driver in parity_check.py consumes):
//   { "author": "Redline", "date": "1970-01-01T00:00:00Z",
//     "jobs": [
//       { "id": "row40-compare", "kind": "compare", "base": "…docx", "next": "…docx", "out": "…docx" },
//       { "id": "row40-pdf", "kind": "topdf", "docx": "…docx", "out": "…pdf",
//         "compress": false, "revisions": "word" } ] }
// An empty jobs list is a probe: the module loads, its facts are reported.
//
// Exports used (jubarte-wasm/npm/node/jubarte_wasm.d.ts:165,203):
//   compareDocuments(original, modified, author, inputLimitsJson?) → Uint8Array
//   docxToPdf(docx, compress?, revisions?, revisionPalette?) → Uint8Array   (full build)
// `date` is NOT a parameter of the WASM binding (jubarte-wasm/src/lib.rs:86-99):
// it always compares under WmlComparerSettings::default(), whose
// date_time_for_revisions is the engine epoch DEFAULT_DATE
// (src/comparer/mod.rs:1381-1384) — the same date parity_check.py pins on the
// CLI (--date) and the Python binding (date=…), so no normalisation is needed.
// A slim build exports no docxToPdf (jubarte-wasm/npm/README.md "Slim builds");
// topdf jobs then fail per-job with a clear message and hasDocxToPdf is false
// in the results, which parity_check.py reports instead of failing silently.
//
// v2 (audit F10): the results now carry the build identity parity_check.py
// records — the absolute path of the .wasm file beside the module, the
// package directory (nearest package.json named jubarte-wasm), and
// ENGINE_COMMIT.txt next to that package.json when one exists
// (tools/parity/build.sh writes it into a wasm-pack out-dir; jubarte-wasm's
// build-npm.sh ships it inside jubarte-wasm/npm). A bare wasm-pack pkg/
// without ENGINE_COMMIT.txt reports engineCommit: null, which
// parity_check.py --expect-commit rejects.

import { readFileSync, writeFileSync, existsSync, statSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

function arg(name) {
  const i = process.argv.indexOf(`--${name}`);
  if (i === -1 || i + 1 >= process.argv.length) return undefined;
  return process.argv[i + 1];
}

/** Resolve --module to a jubarte_wasm.js file (see header for accepted forms). */
function resolveModule(p) {
  const st = statSync(p);
  if (st.isFile()) return p;
  const candidates = [
    path.join(p, "node", "jubarte_wasm.js"),       // jubarte-wasm/npm
    path.join(p, "jubarte_wasm.js"),               // wasm-pack pkg/
    path.join(p, "node_modules", "jubarte-wasm"),  // a scratch npm install
    path.join(p, "jubarte-wasm"),                  // node_modules parent dir
  ];
  for (const c of candidates) if (existsSync(c)) return resolveModule(c);
  throw new Error(`no jubarte_wasm.js under ${p} (looked in: ${candidates.join(", ")})`);
}

// absolute: require() takes a bare relative path for a package name (#1799)
const modulePath = resolveModule(path.resolve(arg("module")));
const require = createRequire(import.meta.url);
const mod = require(modulePath);
if (typeof mod.initPanicHook === "function") mod.initPanicHook();

/** Nearest package.json named jubarte-wasm above the module (its version
 *  identifies the build; ENGINE_COMMIT.txt beside it binds it to a commit). */
function packageFacts(dir) {
  let cur = dir;
  while (true) {
    const pj = path.join(cur, "package.json");
    if (existsSync(pj)) {
      try {
        const parsed = JSON.parse(readFileSync(pj, "utf8"));
        if (parsed.name === "jubarte-wasm") {
          const commitFile = path.join(cur, "ENGINE_COMMIT.txt");
          const commit = existsSync(commitFile)
            ? readFileSync(commitFile, "utf8").trim()
            : null;
          return { packageVersion: parsed.version, packageDir: cur,
                   engineCommitFile: commit ? commitFile : null, engineCommit: commit };
        }
      } catch { /* keep walking */ }
    }
    const parent = path.dirname(cur);
    if (parent === cur) return {};
    cur = parent;
  }
}

/** The .wasm beside the module (wasm-bindgen's <crate>_bg.wasm); its sha256
 *  and mtime are the build identity parity_check.py records. */
function wasmFileOf(moduleFile) {
  const candidate = moduleFile.replace(/\.js$/, "_bg.wasm");
  return existsSync(candidate) ? path.resolve(candidate) : null;
}

const facts = packageFacts(path.dirname(modulePath));
const wasmFile = wasmFileOf(modulePath);
const hasCompare = typeof mod.compareDocuments === "function";
const hasPdf = typeof mod.docxToPdf === "function";
if (!hasCompare) {
  console.error(`${modulePath} exports no compareDocuments — not a jubarte-wasm Node build?`);
  process.exit(2);
}

const job = JSON.parse(readFileSync(arg("jobs"), "utf8"));
const results = {
  module: modulePath,
  wasmFile,
  packageVersion: facts.packageVersion,
  packageDir: facts.packageDir ?? null,
  engineCommitFile: facts.engineCommitFile ?? null,
  engineCommit: facts.engineCommit ?? null,
  hasCompareDocuments: hasCompare,
  hasDocxToPdf: hasPdf,
  jobs: [],
};

for (const j of job.jobs) {
  const r = { id: j.id, ok: true };
  try {
    let out;
    if (j.kind === "compare") {
      // author is pinned by parity_check.py; the date is the engine epoch
      // inside the binding (see header).
      out = mod.compareDocuments(readFileSync(j.base), readFileSync(j.next), job.author);
    } else if (j.kind === "topdf") {
      if (!hasPdf) throw new Error("this build exports no docxToPdf (slim) — PDF parity unavailable for wasm");
      out = mod.docxToPdf(readFileSync(j.docx), j.compress ?? false, j.revisions ?? "conventional");
    } else {
      throw new Error(`unknown kind: ${j.kind}`);
    }
    writeFileSync(j.out, out);
    r.bytes = out.length;
  } catch (e) {
    r.ok = false;
    r.error = String(e && e.message ? e.message : e).split("\n")[0];
  }
  results.jobs.push(r);
}

writeFileSync(arg("out"), JSON.stringify(results, null, 1));
console.log(JSON.stringify(results));
