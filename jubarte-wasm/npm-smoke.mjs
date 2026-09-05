// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

// Smoke test for the assembled npm/ package: full AND slim Node builds.
//
//   node jubarte-wasm/npm-smoke.mjs
//
// Run from the repo root after ./build-npm.sh. Verifies that the slim build
// carries the whole redline surface (compare / accept / reject / list) minus
// only the PDF exports, and that the two builds produce identical redlines.
import { readFileSync } from "node:fs";
import { strict as assert } from "node:assert";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const full = require("./npm/node/jubarte_wasm.js");
const slim = require("./npm/node-slim/jubarte_wasm.js");

const FIX = new URL("../tests/fixtures/redline/", import.meta.url);
const original = readFileSync(new URL("original.docx", FIX));
const modified = readFileSync(new URL("modified.docx", FIX));

for (const [name, mod] of [["full", full], ["slim", slim]]) {
  for (const fn of ["compareDocuments", "acceptRevisions", "rejectRevisions", "getRevisions", "initPanicHook"]) {
    assert.equal(typeof mod[fn], "function", `${name} build must export ${fn}`);
  }
  const redline = mod.compareDocuments(original, modified, "smoke");
  assert.equal(redline[0], 0x50, `${name}: redline is a zip`);
  const revs = JSON.parse(mod.getRevisions(redline));
  assert.ok(revs.length > 0, `${name}: revisions listed`);
  assert.equal(JSON.parse(mod.getRevisions(mod.acceptRevisions(redline))).length, 0, `${name}: accept drains revisions`);
  assert.equal(JSON.parse(mod.getRevisions(mod.rejectRevisions(redline))).length, 0, `${name}: reject drains revisions`);
}

// PDF surface: full-only.
assert.equal(typeof full.docxToPdf, "function", "full build exports docxToPdf");
assert.equal(typeof full.pdfPageCount, "function", "full build exports pdfPageCount");
assert.equal(typeof slim.docxToPdf, "undefined", "slim build must NOT export docxToPdf");
assert.equal(typeof slim.pdfPageCount, "undefined", "slim build must NOT export pdfPageCount");

const pdf = full.docxToPdf(full.compareDocuments(original, modified, "smoke"));
assert.equal(Buffer.from(pdf.slice(0, 5)).toString(), "%PDF-", "docxToPdf emits a PDF");
assert.ok(full.pdfPageCount(pdf) > 0, "pdfPageCount sees pages");

// Same engine, same content: the OPC writer's zip *entry order* is not
// deterministic across instances, so compare size + revision content, not
// raw bytes.
const a = full.compareDocuments(original, modified, "smoke");
const b = slim.compareDocuments(original, modified, "smoke");
assert.equal(a.length, b.length, "slim and full redlines hold the same parts");
assert.equal(slim.getRevisions(b), full.getRevisions(a), "slim and full redlines carry identical revisions");

console.log("npm-smoke: all checks passed (full + slim)");
