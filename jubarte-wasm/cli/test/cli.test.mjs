// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

// `npx jubarte-redlines …` end to end: the bin over the checked-in
// jubarte-wasm build (../npm), with the commands, flags, messages and exit
// codes of `uvx jubarte-redlines`. Run: node --test jubarte-wasm/cli/test/
import { test, before } from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const cliDir = path.resolve(here, "..");
const bin = path.join(cliDir, "bin", "jubarte-redlines.mjs");
const repo = path.resolve(cliDir, "..", "..");
const pair = path.join(repo, "tests/corpus/batch_to_fix/pairs/41_heading_1_bold_demo_id_paraid_overflow_heading_1_style_demo_id_paraid_overflow");
const tracked = path.join(repo, "tests/fixtures/from-docx/critic/tracked-changes.docx");
const untracked = path.join(repo, "tests/fixtures/from-docx/critic/no-tracked-changes.docx");
let tmp;

before(() => {
  // The published package resolves jubarte-wasm from npm; here it is the
  // build checked in beside this package.
  const link = path.join(cliDir, "node_modules", "jubarte-wasm");
  if (!fs.existsSync(link)) {
    fs.mkdirSync(path.dirname(link), { recursive: true });
    fs.symlinkSync(path.join("..", "..", "npm"), link, "junction");
  }
  tmp = fs.mkdtempSync(path.join(os.tmpdir(), "jubarte-cli-"));
});

function run(...args) {
  const r = spawnSync(process.execPath, [bin, ...args], { encoding: "utf8", cwd: tmp });
  return { code: r.status, out: r.stdout, err: r.stderr };
}

function copy(from, name) {
  const to = path.join(tmp, name);
  fs.copyFileSync(from, to);
  return to;
}

test("redline writes a Word redline and refuses to overwrite it", () => {
  const out = path.join(tmp, "redline.docx");
  const r = run("redline", path.join(pair, "base.docx"), path.join(pair, "next.docx"), "-o", out, "--author", "Legal");
  assert.equal(r.code, 0, r.err);
  assert.match(r.out, /^wrote .*redline\.docx \(\d+ bytes\)\n$/);
  assert.deepEqual([...fs.readFileSync(out).subarray(0, 2)], [0x50, 0x4b]);
  const changes = run("changes", out, "--json").out.trim().split("\n").map((l) => JSON.parse(l));
  assert.ok(changes.length > 0 && changes.every((c) => c.author === "Legal"));
  const again = run("compare", path.join(pair, "base.docx"), path.join(pair, "next.docx"), "-o", out);
  assert.equal(again.code, 1);
  assert.match(again.err, /already exists \(use --force to overwrite\)/);
  assert.equal(run("compare", path.join(pair, "base.docx"), path.join(pair, "next.docx"), "-o", out, "--force").code, 0);
});

test("redline names its output after both inputs by default", () => {
  const a = copy(path.join(pair, "base.docx"), "a.docx");
  const b = copy(path.join(pair, "next.docx"), "b.docx");
  assert.equal(run("redline", a, b).code, 0);
  assert.ok(fs.existsSync(path.join(tmp, "a_v_b.docx")));
});

test("a legacy .doc is refused with a save-as hint", () => {
  const doc = path.join(tmp, "b.doc");
  const ole = Buffer.alloc(4096);
  Buffer.from([0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]).copy(ole);
  fs.writeFileSync(doc, ole);
  const r = run("redline", path.join(pair, "base.docx"), doc, "-o", path.join(tmp, "never.docx"));
  assert.equal(r.code, 1);
  assert.match(r.err, /b\.doc is a Word 97-2003 \(\.doc\) or encrypted document; open it in Word and save it as \.docx without a password/);
  assert.ok(!fs.existsSync(path.join(tmp, "never.docx")));
});

test("changes, revisions, accept and reject by kind", () => {
  const listed = run("changes", tracked);
  assert.equal(listed.code, 0, listed.err);
  assert.match(listed.out, /^body:rev:0\tdeletion\ttext\tBo Chen\t" to people that"/);
  assert.match(listed.out, /\n\d+ change\(s\)\n$/);
  const revisions = run("revisions", tracked, "--json").out.trim().split("\n").map((l) => JSON.parse(l));
  assert.equal(revisions[0].type, "Deleted");
  const kept = path.join(tmp, "kept.docx");
  assert.equal(run("accept", tracked, "-o", kept, "--kind", "deletion").code, 0);
  const left = run("changes", kept, "--json").out.trim().split("\n").map((l) => JSON.parse(l));
  assert.ok(left.length > 0 && left.every((c) => c.kind !== "deletion"));
  const base = path.join(tmp, "base.docx");
  assert.equal(run("reject", tracked, "-o", base).code, 0);
  assert.equal(run("changes", base).out, "0 change(s)\n");
});

test("text, inspect, capabilities and convert", () => {
  assert.match(run("text", untracked).out, /^\[body:p:0[\] ]/);
  const summary = run("inspect", untracked);
  assert.match(summary.out, /^sha256: [0-9a-f]{64}\nparagraphs: \d+ /);
  assert.match(summary.out, /\nbody:p:0\t\[/);
  assert.equal(JSON.parse(run("inspect", untracked, "--json").out).schema_version, 1);
  assert.equal(JSON.parse(run("capabilities").out).runtime, "wasm");
  const pdf = path.join(tmp, "out.pdf");
  const r = run("convert", tracked, "-o", pdf, "--revisions", "word", "--compress");
  assert.equal(r.code, 0, r.err);
  assert.match(r.out, /^wrote .*out\.pdf \(\d+ bytes, \d+ pages?\)\n$/);
  assert.equal(fs.readFileSync(pdf).subarray(0, 5).toString(), "%PDF-");
  const png = run("convert", tracked, "--png");
  assert.equal(png.code, 2);
  assert.match(png.err, /PNG pages need the Python or Rust build/);
  assert.match(run("convert", tracked, "--revisions", "custom").err, /--revision-palette/);
});

test("edit writes the bundle, and a refused plan exits 3", () => {
  const first = run("inspect", untracked, "--json");
  const para = JSON.parse(first.out).paragraphs.find((p) => p.text.length > 3);
  const word = para.text.split(" ")[0];
  const plan = path.join(tmp, "plan.json");
  fs.writeFileSync(plan, JSON.stringify({ schema_version: 1, author: "Claude", date: "2026-09-25T12:00:00Z", operations: [{ id: "w", kind: "replace", paragraph: { index: para.index }, find: word, replacement: "Changed" }] }));
  const dir = path.join(tmp, "review");
  const r = run("edit", untracked, "--plan", plan, "--out-dir", dir);
  assert.equal(r.code, 0, r.err);
  for (const name of ["clean.docx", "redline.docx", "report.jsonl"]) assert.ok(fs.existsSync(path.join(dir, name)), name);
  const rows = fs.readFileSync(path.join(dir, "report.jsonl"), "utf8").trim().split("\n").map((l) => JSON.parse(l));
  assert.equal(rows.at(-1).ev, "summary");
  assert.equal(run("edit", untracked, "--plan", plan, "--out-dir", dir).code, 1);
  const refused = run("edit", tracked, "--plan", plan, "--out-dir", path.join(tmp, "refused"));
  assert.equal(refused.code, 3);
  assert.equal(JSON.parse(refused.out.trim().split("\n").at(-1)).code, "EXISTING_REVISIONS");
  assert.ok(!fs.existsSync(path.join(tmp, "refused")));
});

test("help, version and usage errors", () => {
  const help = run("--help");
  assert.equal(help.code, 0);
  assert.match(help.out, /Usage: jubarte-redlines/);
  assert.match(help.out, /compare/);
  assert.match(run("redline", "--help").out, /Usage: jubarte-redlines (?:redline|compare)/);
  assert.match(run("--version").out, /^jubarte-redlines \d+\.\d+\.\d+/);
  assert.equal(run().code, 2);
  assert.equal(run("frobnicate").code, 2);
  const bad = run("redline", "only-one.docx");
  assert.equal(bad.code, 2);
  assert.match(bad.err, /required|MODIFIED/);
  assert.equal(run("redline", "a", "b", "--nope").code, 2);
});

test("github aliases write text without implicit Word output (integration)", () => {
  const a = copy(path.join(pair, "base.docx"), "unified-a.docx");
  const b = copy(path.join(pair, "next.docx"), "unified-b.docx");
  const first = run("diff", a, b, "--format", "github", "--context", "0");
  assert.equal(first.code, 0, first.err);
  assert.match(first.out, /^diff --git /);
  assert.ok(!fs.existsSync(path.join(tmp, "unified-a_v_unified-b.docx")));
  for (const format of ["unified", "text"]) {
    const out = path.join(tmp, `${format}.patch`);
    const result = run("diff", a, b, "--format", format, "--context", "0", "-o", out);
    assert.equal(result.code, 0, result.err);
    assert.equal(fs.readFileSync(out, "utf8"), first.out);
  }
});

test("format contradictions and unsupported flags fail before I/O (integration)", () => {
  const output = path.join(tmp, "never-github.docx");
  for (const extra of [["--format", "github"], ["--format", "github", "--to", "docx"], ["--format", "github", "--context", "-1"]]) {
    const result = run("diff", "missing-a.docx", "missing-b.docx", "-o", output, ...extra);
    assert.equal(result.code, 2, result.err);
    assert.doesNotMatch(result.err, /reading/);
    assert.ok(!fs.existsSync(output));
  }
  for (const args of [["inspect", "missing.docx", "--tables"], ["convert", "missing.docx", "--timeout", "1"], ["compare", "a", "b", "--mode", "powertools"]]) {
    const result = run(...args);
    assert.equal(result.code, 2, result.err);
    assert.doesNotMatch(result.err, /reading/);
  }
});

test("Markdown paragraph/critic output and shorthand comparison (integration)", () => {
  const a = path.join(tmp, "short-a.md"), b = path.join(tmp, "short-b.md");
  fs.writeFileSync(a, "Due in 30 days.\n");
  fs.writeFileSync(b, "Due in 45 days.\n");
  const patch = run("diff", a, b, "--author", "Legal", "--date", "2026-09-30T14:05:00Z");
  assert.equal(patch.code, 0, patch.err);
  assert.match(patch.out, /\[-30-\]\{\+45\+\}/);
  assert.equal(run("diff", a, b, "--format", "critic").out, "Due in {~~30~>45~~} days.\n");
  const compared = run(a, b, "--quiet");
  assert.equal(compared.code, 0, compared.err);
  assert.equal(compared.out, "");
  assert.equal(fs.readFileSync(path.join(tmp, "short-a_v_short-b.docx")).subarray(0, 2).toString(), "PK");
  const converted = run("convert", a, "--to", "docx");
  assert.equal(converted.code, 0, converted.err);
  assert.equal(fs.readFileSync(path.join(tmp, "short-a.docx")).subarray(0, 2).toString(), "PK");
  const accepted = run("text", tracked, "--track-changes", "accept");
  assert.equal(accepted.code, 0, accepted.err);
  assert.doesNotMatch(accepted.out, /\[body:p:/);
});

test("all views write only the requested text file with status on stderr (integration)", () => {
  const a = path.join(tmp, "view-old.txt"), b = path.join(tmp, "view-new.md");
  fs.writeFileSync(a, "Due in 30 days.\n");
  fs.writeFileSync(b, "Due in 45 days.\n");
  for (const format of ["github", "unified", "text", "word", "normal", "context", "side-by-side"]) {
    const stdout = run("diff", a, b, "--format", format, "--full-lines", "-U", "0");
    assert.equal(stdout.code, 0, stdout.err);
    assert.ok(stdout.out.includes("30") && stdout.out.includes("45"));
    if (format === "github") assert.ok(stdout.out.includes(a) && stdout.out.includes(b));
    const out = path.join(tmp, `view-${format}.patch`);
    const saved = run("diff", a, b, "--format", format, "--full-lines", "-U", "0", "-o", out);
    assert.equal(saved.code, 0, saved.err);
    assert.equal(saved.out, "");
    assert.match(saved.err, /wrote/);
    assert.equal(fs.readFileSync(out, "utf8"), stdout.out);
  }
  assert.ok(!fs.existsSync(path.join(tmp, "view-old_v_view-new.docx")));
});

test("patch output txt inference and unknown suffix fallback match native (integration)", () => {
  const a = path.join(tmp, "inference-old.md"), b = path.join(tmp, "inference-new.md");
  fs.writeFileSync(a, "Due in 30 days.\n");
  fs.writeFileSync(b, "Due in 45 days.\n");
  for (const suffix of ["txt", "unknown"]) {
    const out = path.join(tmp, `inference.${suffix}`), result = run("diff", a, b, "-o", out);
    assert.equal(result.code, 0, result.err);
    assert.equal(fs.readFileSync(out, "utf8"), "Due in {~~30~>45~~} days.\n");
    assert.match(result.out, /\[-30-\]\{\+45\+\}/);
  }
});

test("declared docx UTF-8 never silently compares as Markdown (integration)", () => {
  const a = path.join(tmp, "pretend.docx"), b = path.join(tmp, "pretend-new.md");
  fs.writeFileSync(a, "UTF-8 masquerading as DOCX\n");
  fs.writeFileSync(b, "new\n");
  for (const format of ["patch", "critic", "github", "word", "normal", "context", "side-by-side"]) {
    const result = run("diff", a, b, "--format", format);
    assert.equal(result.code, 1, result.err);
    assert.equal(result.out, "");
    assert.ok(!fs.existsSync(path.join(tmp, "pretend_v_pretend-new.docx")));
  }
  const out = path.join(tmp, "pretend-redline.docx");
  assert.equal(run("compare", a, b, "-o", out).code, 1);
  assert.ok(!fs.existsSync(out));
});

test("CLI accepts both revision histories and word always accepts (integration)", () => {
  const a = path.join(tmp, "history-old.md"), b = path.join(tmp, "history-new.md");
  fs.writeFileSync(a, "Due in {~~30~>45~~} days.\n");
  fs.writeFileSync(b, "Due in {~~60~>45~~} days.\n");
  const preserved = run("diff", a, b, "--format", "github");
  assert.equal(preserved.code, 0, preserved.err);
  assert.ok(preserved.out.includes("30") && preserved.out.includes("60"));
  const accepted = run("diff", a, b, "--format", "github", "--accept-changes");
  assert.equal(accepted.code, 0, accepted.err);
  assert.equal(accepted.out, "");
  const word = run("diff", a, b, "--format", "word");
  assert.equal(word.code, 0, word.err);
  assert.ok(word.out.includes("45") && !word.out.includes("30") && !word.out.includes("60"));
});

test("unknown output suffix defaults to Word for Word inputs (integration)", () => {
  const out = path.join(tmp, "word-fallback.unknown");
  const result = run("diff", path.join(pair, "base.docx"), path.join(pair, "next.docx"), "-o", out);
  assert.equal(result.code, 0, result.err);
  assert.equal(fs.readFileSync(out).subarray(0, 4).toString("hex"), "504b0304");
  assert.ok(result.out.length > 0);
});
