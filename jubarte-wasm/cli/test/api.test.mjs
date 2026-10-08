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
