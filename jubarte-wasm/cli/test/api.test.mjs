// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

// Pure byte fixtures from the engine's Markdown builder, no filesystem I/O.
import { test } from "node:test";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
const require = createRequire(import.meta.url);
const wasm = require("../../npm/node/jubarte_wasm.js");
const bytes = (text) => new TextEncoder().encode(text);

test("unified patch has all eight hunks and unabridged Unicode lines", () => {
  const old = Array.from({ length: 8 }, (_, i) => `old ${i} ${"é".repeat(300)}\nseparator ${i}\n`).join("");
  const patch = wasm.diffDocumentsUnified(bytes(old), bytes(old.replaceAll("old ", "new ")), "old.md", "new.md", 0);
  assert.match(patch, /^diff --git a\/old\.md b\/new\.md\n--- a\/old\.md\n\+\+\+ b\/new\.md\n/);
  assert.equal(patch.match(/@@ -/g).length, 8);
  for (let i = 0; i < 8; i++) assert.ok(patch.includes(`-old ${i} ${"é".repeat(300)}\n+new ${i} ${"é".repeat(300)}\n`));
});

test("default context, labels, empty patches and existing tracked marks", () => {
  const patch = wasm.diffDocumentsUnified(bytes("Intro\nOld\nEnd\n"), bytes("Intro\nNew\nEnd\n"));
  assert.ok(patch.includes("--- a/old.md\n+++ b/new.md\n"));
  assert.ok(patch.includes("@@ -1,3 +1,3 @@\n Intro\n-Old\n+New\n End\n"));
  assert.equal(wasm.diffDocumentsUnified(bytes("same\n"), bytes("same\n")), "");
  const old = wasm.markdownToDocx("Due in {~~30~>45~~} days.\n");
  const next = wasm.markdownToDocx("Due in {~~30~>60~~} days.\n");
  const tracked = wasm.diffDocumentsUnified(old, next);
  assert.ok(tracked.includes("[-30-]{+45+}") && tracked.includes("[-30-]{+60+}"), tracked);
  assert.equal(wasm.diffDocumentsUnified(old, old), "");
});

test("unified context rejects JavaScript coercions and u32 overflow", () => {
  for (const context of [true, false, -1, 1.5, 2 ** 32, 1e100, "3", null, NaN, Infinity]) {
    assert.throws(() => wasm.diffDocumentsUnified(bytes("a\n"), bytes("b\n"), undefined, undefined, context), /context/);
  }
  assert.ok(wasm.diffDocumentsUnified(bytes("a\nb\n"), bytes("a\nc\n"), undefined, undefined, 2 ** 32 - 1).includes(" a\n"));
});

test("shared clap JSON accepts aliases/defaults and prunes unsupported commands", () => {
  const parsed = JSON.parse(wasm.parseCli(JSON.stringify(["diff", "a.docx", "b.docx", "--format", "unified"])));
  assert.equal(parsed.exit_code, 0);
  assert.equal(parsed.command, "diff");
  assert.equal(parsed.args.format, "github");
  assert.equal(parsed.args.context, 3);
  const help = JSON.parse(wasm.parseCli('["--help"]', "web-jubarte", '["diff"]'));
  assert.equal(help.stream, "stdout");
  assert.match(help.text, /web-jubarte/);
  assert.doesNotMatch(help.text, /self-update/);
  assert.equal(JSON.parse(wasm.parseCli('["inspect","a.docx"]', undefined, '["diff"]')).exit_code, 2);
  for (const args of ["{}", '[1]', "invalid"]) assert.throws(() => wasm.parseCli(args));
  assert.throws(() => wasm.parseCli("[]", undefined, "{}"));
});

test("paragraph patches and CriticMarkup remain separate APIs", () => {
  const old = bytes("Due in 30 days.\n"), next = bytes("Due in 45 days.\n");
  const paragraph = JSON.parse(wasm.diffDocuments(old, next, "A", "2026-09-30T14:05:00Z"));
  assert.equal(paragraph.hunks[0].at, "line:1");
  assert.match(paragraph.text, /\[-30-\]\{\+45\+\}/);
  assert.equal(wasm.diffDocumentsCritic(old, next), "Due in {~~30~>45~~} days.\n");
  const redline = wasm.redlineDocuments(old, next, "Legal", "2026-09-30T14:05:00Z");
  const revisions = JSON.parse(wasm.getRevisions(redline));
  assert.ok(revisions.length > 0 && revisions.every((r) => r.author === "Legal" && r.date === "2026-09-30T14:05:00Z"));
  assert.match(wasm.documentMarkdownWithChanges(redline, "accept"), /45/);
  assert.doesNotMatch(wasm.documentMarkdownWithChanges(redline, "accept"), /30|\[body:p:/);
  assert.match(wasm.documentMarkdownWithChanges(redline, "reject"), /30/);
});

test("document views select formats and preserve the legacy full unified API", () => {
  for (const format of ["github", "word", "normal", "context", "side-by-side"]) {
    const text = wasm.diffDocumentsView(bytes("Due in 30 days.\n"), bytes("Due in 45 days.\n"), JSON.stringify({ format, context: 0 }));
    assert.ok(text.includes("30") && text.includes("45"), text);
    if (format === "word") assert.ok(text.includes("{~~30~>45~~}"), text);
  }
  const prefix = "é🙂".repeat(100), old = bytes(`${prefix} old tail\n`), next = bytes(`${prefix} new tail\n`);
  for (const format of ["github", "normal", "context", "side-by-side"]) {
    const clipped = wasm.diffDocumentsView(old, next, JSON.stringify({ format }));
    const full = wasm.diffDocumentsView(old, next, JSON.stringify({ format, fullLines: true }));
    assert.ok(!clipped.includes(prefix) && full.includes(prefix));
    assert.ok(full.includes("old tail") && full.includes("new tail"));
    assert.ok(!clipped.includes("�") && clipped.length < full.length);
  }
  assert.ok(wasm.diffDocumentsUnified(old, next).includes(prefix));
  const labeled = wasm.diffDocumentsView(bytes("a\nb\n"), bytes("a\nc\n"), JSON.stringify({ oldName: "folder/before.md", newName: "folder/after.md", context: 2 ** 32 - 1 }));
  assert.ok(labeled.includes("--- a/folder/before.md\n+++ b/folder/after.md\n") && labeled.includes(" a\n"));
});

test("word view always accepts both histories and other views accept on request", () => {
  const old = wasm.markdownToDocx("Due in {~~30~>45~~} days.\n");
  const next = wasm.markdownToDocx("Due in {~~60~>45~~} days.\n");
  for (const format of ["github", "normal", "context", "side-by-side"]) {
    assert.ok(wasm.diffDocumentsView(old, next, JSON.stringify({ format })));
    const clean = wasm.markdownToDocx("Due in 45 days.\n");
    const accepted = wasm.diffDocumentsView(old, next, JSON.stringify({ format, acceptChanges: true }));
    assert.equal(accepted, wasm.diffDocumentsView(clean, clean, JSON.stringify({ format })));
    assert.ok(!accepted.includes("30") && !accepted.includes("60"));
  }
  const word = wasm.diffDocumentsView(old, next, '{"format":"word"}');
  assert.equal(word, "");
  assert.doesNotMatch(word, /\{(?:~~|\+\+|--)/);
  const changed = wasm.diffDocumentsView(old, wasm.markdownToDocx("Due in {~~60~>90~~} days.\n"), '{"format":"word"}');
  assert.ok(changed.includes("{~~45~>90~~}") && !changed.includes("30") && !changed.includes("60"));
});

test("view JSON strictly validates all typed options and explicit sides", () => {
  const old = bytes("a\n"), next = bytes("b\n");
  for (const context of [true, false, -1, 1.5, 2 ** 32, "3", null]) {
    assert.throws(() => wasm.diffDocumentsView(old, next, JSON.stringify({ context })), /context/);
  }
  for (const options of [{ format: "critic" }, { format: "unified" }, { format: null }, { oldFormat: "txt" }, { newFormat: "word" }, { oldName: 1 }, { acceptChanges: null }, { fullLines: "true" }, { typo: 1 }]) {
    assert.throws(() => wasm.diffDocumentsView(old, next, JSON.stringify(options)));
  }
  for (const json of ["{", "null", "[]", ""]) assert.throws(() => wasm.diffDocumentsView(old, next, json));
  assert.throws(() => wasm.diffDocumentsView(old, next, '{"oldFormat":"docx"}'), /docx|DOCX|ZIP|zip/);
  assert.throws(() => wasm.diffDocumentsView(old, next, '{"newFormat":"docx"}'), /docx|DOCX|ZIP|zip/);
  assert.throws(() => wasm.diffDocumentsView(new Uint8Array([255]), next, '{"oldFormat":"md"}'), /UTF-8/);
  assert.ok(wasm.diffDocumentsView(old, next, '{"oldFormat":"md","newFormat":"md"}'));
  const docx = wasm.markdownToDocx("a\n");
  assert.equal(wasm.diffDocumentsView(docx, docx, '{"oldFormat":"docx","newFormat":"docx"}'), "");
  assert.equal(wasm.diffDocumentsView(old, old), "");
});

test("shared parser emits adapter format metadata and new view switches", () => {
  for (const [suffix, expected] of [["txt", "md"], ["mdown", "md"], ["unknown", null]]) {
    const parsed = JSON.parse(wasm.parseCli(JSON.stringify(["diff", "a.txt", "b.markdown", "-o", `out.${suffix}`])));
    assert.equal(parsed.args.old_format, "md");
    assert.equal(parsed.args.new_format, "md");
    assert.equal(parsed.args.output_format, expected);
    assert.equal(parsed.args.accept_changes, false);
    assert.equal(parsed.args.full_lines, false);
  }
  const parsed = JSON.parse(wasm.parseCli('["diff","a.docx","b.docx","--format","word","--accept-changes","--full-lines","-U","0"]'));
  assert.equal(parsed.args.format, "word");
  assert.equal(parsed.args.accept_changes, true);
  assert.equal(parsed.args.full_lines, true);
});
