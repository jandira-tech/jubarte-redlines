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
// SMOKE_FULL / SMOKE_SLIM point at fresh wasm-pack output (pkg/, pkg-slim/)
// before build-npm.sh has assembled npm/.
const full = require(process.env.SMOKE_FULL ?? "./npm/node/jubarte_wasm.js");
const slim = require(process.env.SMOKE_SLIM ?? "./npm/node-slim/jubarte_wasm.js");

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

// Agent surface (both builds): inspect, markdown, edit plans, capabilities.
for (const [name, mod] of [["full", full], ["slim", slim]]) {
  const snapshot = JSON.parse(mod.inspectDocument(original));
  assert.equal(snapshot.source_sha256, mod.sourceSha256(original), `${name}: snapshot hash`);
  assert.ok(snapshot.paragraphs.length > 0, `${name}: paragraphs inspected`);
  assert.match(mod.documentMarkdown(original), /\[body:p:0\]/, `${name}: markdown carries ids`);

  const first = snapshot.paragraphs.find((p) => p.text.trim().length > 0);
  const plan = JSON.stringify({
    schema_version: 1,
    source_sha256: snapshot.source_sha256,
    author: "smoke",
    date: "2026-09-28T00:00:00Z",
    operations: [{ id: "op-1", kind: "insert", paragraph: { id: first.id }, position: "end", text: " (smoke)" }],
  });
  const preview = mod.previewEditPlan(original, plan);
  assert.ok(preview.ok && preview.clean === undefined, `${name}: preview resolves without documents`);
  const out = mod.applyEditPlan(original, plan);
  assert.ok(out.ok, `${name}: plan applied: ${out.json}`);
  assert.equal(out.clean[0], 0x50, `${name}: clean copy is a zip`);
  assert.ok(JSON.parse(mod.getRevisions(out.redline)).length > 0, `${name}: redline tracks the edit`);
  assert.match(mod.editReportJsonl(out.json), /"summary"/, `${name}: report as JSON lines`);
  assert.ok(JSON.parse(mod.inspectDocument(out.clean)).paragraphs.some((p) => p.text.endsWith("(smoke)")), `${name}: edit landed`);

  const stale = mod.applyEditPlan(modified, plan);
  assert.equal(stale.ok, false, `${name}: stale source refused`);
  assert.equal(JSON.parse(stale.json).code, "STALE_SOURCE", `${name}: refusal code`);
  assert.equal(JSON.parse(mod.applyEditPlan(new Uint8Array([1, 2, 3]), plan).json).code, "STALE_SOURCE");

  const caps = JSON.parse(mod.capabilities());
  assert.equal(caps.runtime, "wasm");
  assert.equal(caps.operations.pdf, name === "full", `${name}: pdf capability matches the build`);
  assert.equal(caps.operations.png, false);
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
