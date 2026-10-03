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
import { inflateRawSync } from "node:zlib";

const require = createRequire(import.meta.url);
// SMOKE_FULL / SMOKE_SLIM point at fresh wasm-pack output (pkg/, pkg-slim/)
// before build-npm.sh has assembled npm/.
const full = require(process.env.SMOKE_FULL ?? "./npm/node/jubarte_wasm.js");
const slim = require(process.env.SMOKE_SLIM ?? "./npm/node-slim/jubarte_wasm.js");

const FIX = new URL("../tests/fixtures/redline/", import.meta.url);
const original = readFileSync(new URL("original.docx", FIX));
const modified = readFileSync(new URL("modified.docx", FIX));

for (const [name, mod] of [["full", full], ["slim", slim]]) {
  for (const fn of ["compareDocuments", "acceptRevisions", "rejectRevisions", "getRevisions", "listChanges", "acceptChanges", "rejectChanges", "initPanicHook"]) {
    assert.equal(typeof mod[fn], "function", `${name} build must export ${fn}`);
  }
  const redline = mod.compareDocuments(original, modified, "smoke");
  assert.equal(redline[0], 0x50, `${name}: redline is a zip`);
  const revs = JSON.parse(mod.getRevisions(redline));
  assert.ok(revs.length > 0, `${name}: revisions listed`);
  assert.equal(JSON.parse(mod.getRevisions(mod.acceptRevisions(redline))).length, 0, `${name}: accept drains revisions`);
  assert.equal(JSON.parse(mod.getRevisions(mod.rejectRevisions(redline))).length, 0, `${name}: reject drains revisions`);
  const changes = JSON.parse(mod.listChanges(redline));
  assert.ok(changes.length > 1 && changes[0].id.startsWith("body:rev:"), `${name}: changes listed by id`);
  const kept = JSON.parse(mod.listChanges(mod.acceptChanges(redline, JSON.stringify({ ids: [changes[0].id] }))));
  assert.ok(!kept.some((c) => c.id === changes[0].id) && kept.length > 0, `${name}: accepting one change keeps the rest`);
  assert.equal(JSON.parse(mod.listChanges(mod.rejectChanges(redline, "{}"))).length, 0, `${name}: {} rejects every change`);
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
  assert.equal(caps.operations.fields, name === "full", `${name}: fields capability matches the build`);
}

// Markdown creation (both builds): markdownToDocx(text, optionsJson, reference).
// A package part's text, read through the zip's central directory.
function partText(docx, wanted) {
  const buf = Buffer.from(docx);
  const eocd = buf.lastIndexOf(Buffer.from([0x50, 0x4b, 0x05, 0x06]));
  let at = buf.readUInt32LE(eocd + 16);
  for (let i = 0; i < buf.readUInt16LE(eocd + 10); i++) {
    const method = buf.readUInt16LE(at + 10);
    const size = buf.readUInt32LE(at + 20);
    const nameLength = buf.readUInt16LE(at + 28);
    const next = at + 46 + nameLength + buf.readUInt16LE(at + 30) + buf.readUInt16LE(at + 32);
    if (buf.toString("utf8", at + 46, at + 46 + nameLength) === wanted) {
      const local = buf.readUInt32LE(at + 42);
      const start = local + 30 + buf.readUInt16LE(local + 26) + buf.readUInt16LE(local + 28);
      const data = buf.subarray(start, start + size);
      return (method === 8 ? inflateRawSync(data) : data).toString("utf8");
    }
    at = next;
  }
  throw new Error(`no ${wanted}`);
}
const pageWidth = (docx) => /<w:pgSz w:w="(\d+)"/.exec(partText(docx, "word/document.xml"))[1];
for (const [name, mod] of [["full", full], ["slim", slim]]) {
  assert.equal(typeof mod.markdownToDocx, "function", `${name} build must export markdownToDocx`);
  const draft = "# Terms\n\nPayment is due in {~~30~>45~~} days.\n";
  const kept = mod.markdownToDocx(draft);
  assert.equal(kept[0], 0x50, `${name}: markdownToDocx writes a zip`);
  assert.match(mod.documentMarkdown(kept), /Payment is due in/, `${name}: the text is in the document`);
  assert.ok(JSON.parse(mod.getRevisions(kept)).length > 0, `${name}: CriticMarkup becomes tracked changes`);
  const accepted = mod.markdownToDocx(draft, JSON.stringify({ track_changes: "accept" }));
  assert.match(mod.documentMarkdown(accepted), /due in 45 days/, `${name}: accepted`);
  assert.equal(pageWidth(mod.markdownToDocx("Body.\n", "{}")), "12240", `${name}: US Letter by default`);
  const a4 = mod.markdownToDocx("Body.\n", JSON.stringify({ page: "a4" }));
  assert.equal(pageWidth(a4), "11906", `${name}: the a4 option writes an A4 page`);
  assert.equal(JSON.parse(mod.inspectDocument(a4)).paragraphs[0].text, "Body.", `${name}: A4 body`);
  assert.equal(pageWidth(mod.markdownToDocx("Body.\n", JSON.stringify({ page: "a4" }), a4)), "11906", `${name}: a reference lends its page`);
  assert.throws(() => mod.markdownToDocx("Body.\n", JSON.stringify({ page: "legal" })), /legal/, `${name}: unknown page refused`);
}

// PDF surface: full-only.
assert.equal(typeof full.docxToPdf, "function", "full build exports docxToPdf");
assert.equal(typeof full.pdfPageCount, "function", "full build exports pdfPageCount");
assert.equal(typeof slim.docxToPdf, "undefined", "slim build must NOT export docxToPdf");
assert.equal(typeof slim.pdfPageCount, "undefined", "slim build must NOT export pdfPageCount");
assert.equal(typeof slim.updateFields, "undefined", "slim build must NOT export updateFields");

const refreshed = full.updateFields(original);
assert.equal(refreshed.docx[0], 0x50, "updateFields returns a zip");
assert.ok(JSON.parse(refreshed.json).page_count > 0, "updateFields reports the page count");

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
