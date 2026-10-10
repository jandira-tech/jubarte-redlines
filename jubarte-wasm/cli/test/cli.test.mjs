// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

// `npx jubarte-redlines …` end to end: the bin over the checked-in
// jubarte-wasm build (../npm), with the commands, flags, messages and exit
// codes of `uvx jubarte-redlines`. Run: node --test jubarte-wasm/cli/test/
import { test, before } from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";
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

test("field refresh is unsupported before reading or overwriting any file", () => {
  const out = path.join(tmp, "field-refresh.docx");
  const report = path.join(tmp, "field-refresh.json");
  fs.writeFileSync(out, "preserve document");
  fs.writeFileSync(report, "preserve report");
  for (const extra of [[], ["--to", "docx"], ["--track-changes", "accept"], ["--track-changes", "reject"], ["--report", report, "--force"]]) {
    const result = run("convert", "missing.docx", "-o", out, "--update-fields", ...extra);
    assert.equal(result.code, 2, result.err);
    assert.match(result.err, /--update-fields.*not supported/);
    assert.doesNotMatch(result.err, /reading|ENOENT|already exists/);
    assert.equal(result.out, "");
    assert.equal(fs.readFileSync(out, "utf8"), "preserve document");
    assert.equal(fs.readFileSync(report, "utf8"), "preserve report");
  }
});

test("removed command help and legacy listing flags are unavailable", () => {
  const help = run("--help");
  assert.equal(help.code, 0, help.err);
  assert.match(help.out, /\n  changes /);
  assert.doesNotMatch(help.out, /\n  (revisions|fields) /);
  for (const args of [["revisions", "missing.docx", "--json"], ["fields", "update", "missing.docx", "-o", "unused.docx"]]) {
    const result = run(...args);
    assert.equal(result.code, 2, result.err);
    assert.doesNotMatch(result.err, /reading|ENOENT/);
    assert.equal(result.out, "");
    assert.ok(!fs.existsSync(path.join(tmp, "unused.docx")));
  }
});

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

test("changes, accept and reject by kind", () => {
  const listed = run("changes", tracked);
  assert.equal(listed.code, 0, listed.err);
  assert.match(listed.out, /^body:rev:0\tdeletion\ttext\tBo Chen\t" to people that"/);
  assert.match(listed.out, /\n\d+ change\(s\)\n$/);
  const rows = run("changes", tracked, "--json").out.trim().split("\n").map((l) => JSON.parse(l));
  assert.equal(rows[0].kind, "deletion");
  // `revisions` left the CLI: `changes` is the one listing.
  const gone = run("revisions", tracked);
  assert.notEqual(gone.code, 0);
  assert.doesNotMatch(gone.out, /revision\(s\)/);
  const kept = path.join(tmp, "kept.docx");
  assert.equal(run("accept", tracked, "-o", kept, "--kind", "deletion").code, 0);
  const left = run("changes", kept, "--json").out.trim().split("\n").map((l) => JSON.parse(l));
  assert.ok(left.length > 0 && left.every((c) => c.kind !== "deletion"));
  const base = path.join(tmp, "base.docx");
  assert.equal(run("reject", tracked, "-o", base).code, 0);
  assert.equal(run("changes", base).out, "0 change(s)\n");
});

test("read (alias text), inspect, capabilities and convert", () => {
  // `read` prints the agent view, as the binary and the Python CLI do.
  for (const command of ["read", "text"]) {
    const view = run(command, untracked, "--no-page-markers");
    assert.equal(view.code, 0, view.err);
    assert.match(view.out, new RegExp(`^---\\nsource: ${path.basename(untracked).replaceAll(".", "\\.")}\\n`));
    assert.match(view.out, /\n<!-- p0[ -]/);
    assert.doesNotMatch(view.out, /<!-- page /);
  }
  const paged = run("read", untracked);
  assert.equal(paged.code, 0, paged.err);
  assert.match(paged.out, /\n<!-- page 1 of \d+ -->\n/);
  const picked = run("read", untracked, "-p", "p0");
  assert.equal(picked.code, 0, picked.err);
  assert.doesNotMatch(picked.out, /<!-- p1[ -]/);
  // No marks in the document: --changed keeps nothing, and says so.
  const changed = run("read", untracked, "--changed", "--by", "AC");
  assert.equal(changed.code, 0, changed.err);
  assert.match(changed.out, /\nrange: changed by AC \(none\) of p0-/);
  assert.notEqual(run("read", untracked, "--by", "AC").code, 0);
  // The binary's other read flags parse and act the same way here.
  const head = run("read", untracked, "--head", "1", "--no-page-markers");
  assert.equal(head.code, 0, head.err);
  assert.match(head.out, /\nrange: head 1 \(p0\) of p0-/);
  assert.match(run("read", untracked, "--tail", "1", "--no-page-markers").out, /\nrange: tail 1 \(p/);
  for (const flags of [["--track-changes", "accept"], ["--track-changes", "reject"], ["--comments", "none"], ["--dates"]]) {
    const view = run("read", untracked, "--no-page-markers", ...flags);
    assert.equal(view.code, 0, `${flags}: ${view.err}`);
    assert.match(view.out, /\n<!-- p0[ -]/);
  }
  assert.notEqual(run("read", untracked, "--head", "1", "--tail", "1").code, 0);
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

test("convert --move-comments and --changed-only", () => {
  const pages = (r) => {
    assert.equal(r.code, 0, r.err);
    return Number(/, (\d+) pages?\)/.exec(r.out)[1]);
  };
  const out = path.join(tmp, "flags.pdf");
  assert.equal(pages(run("convert", tracked, "-o", out, "--force")), 1);
  assert.equal(pages(run("convert", tracked, "-o", out, "--force", "--move-comments")), 2, "the comments get a page after the last");
  // A long redline whose one change is on its first page.
  const wasm = createRequire(bin)("jubarte-wasm");
  const long = Array.from({ length: 120 }, (_, i) => `Paragraph ${i}.`).join("\n\n");
  const redline = path.join(tmp, "long.docx");
  fs.writeFileSync(redline, wasm.compareDocuments(wasm.markdownToDocx(long), wasm.markdownToDocx(long.replace("Paragraph 0.", "Paragraph zero.")), "Ann"));
  assert.ok(pages(run("convert", redline, "-o", out, "--force")) > 1);
  assert.equal(pages(run("convert", redline, "-o", out, "--force", "--changed-only")), 1);
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
  // Under --editing-mode, a plan that is not JSON or not an object is INVALID_PLAN, exit 3.
  for (const [name, text] of [["bad.json", "not json"], ["null.json", "null"]]) {
    const bad = path.join(tmp, name);
    fs.writeFileSync(bad, text);
    const out = path.join(tmp, `invalid-${name}`);
    const invalid = run("edit", untracked, "--plan", bad, "--editing-mode", "--out-dir", out);
    assert.equal(invalid.code, 3, invalid.err);
    assert.match(invalid.err, /INVALID_PLAN/);
    assert.ok(!fs.existsSync(out));
  }
});

test("edit and add by flags print the changed blocks, as the binary does", () => {
  const para = JSON.parse(run("inspect", untracked, "--json").out).paragraphs.find((p) => p.text.length > 3);
  const word = para.text.split(" ")[0];
  const dir = path.join(tmp, "flags");
  const r = run("edit", untracked, "-p", `p${para.index}`, "--anchor", word, "--content", "Changed", "--author", "Ann Counsel", "--out-dir", dir);
  assert.equal(r.code, 0, r.err);
  for (const name of ["clean.docx", "redline.docx", "patch.diff", "report.jsonl"]) assert.ok(fs.existsSync(path.join(dir, name)), name);
  assert.match(r.out, new RegExp(`\\nrange: changed by @AC \\(p${para.index}\\) of p0-`));
  assert.ok(r.out.includes(`{~~${word}~>Changed~~}`), r.out);
  assert.doesNotMatch(r.out, /\n@@ /);
  // --plan excludes the flags; a flag before any -p and a bare -p are usage errors.
  for (const bad of [["--plan", "x.json", "-p", "p1", "--delete"], ["--anchor", "a", "-p", "p1", "--content", "b"], ["-p", "p1"]]) {
    assert.equal(run("edit", untracked, ...bad, "--out-dir", path.join(tmp, "flags-bad")).code, 2, bad.join(" "));
  }
  const comment = run("add", untracked, "-p", `p${para.index}`, "--anchor", word, "--content", "Why?", "--author", "Ann Counsel", "--out-dir", path.join(tmp, "flags-comment"));
  assert.equal(comment.code, 0, comment.err);
  assert.ok(comment.out.includes(`{==${word}==}{>>#c0 @AC: Why?<<}`), comment.out);
  const editing = run("add", untracked, "-p", `p${para.index}`, "--content", "Recitals", "--editing-mode", "--out-dir", path.join(tmp, "flags-editing"));
  assert.equal(editing.code, 0, editing.err);
  assert.ok(!fs.existsSync(path.join(tmp, "flags-editing", "redline.docx")));
  // The new paragraph after a heading keeps the heading's style here (`# Recitals`).
  assert.match(editing.out, /\n(# )?Recitals\n/);
  assert.ok(!editing.out.includes("{++"), editing.out);
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
  for (const args of [["inspect", "missing.docx", "--tables"], ["convert", "missing.docx", "--timeout", "1"], ["convert", "missing.docx", "-o", "x.docx", "--update-fields"], ["compare", "a", "b", "--mode", "powertools"]]) {
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
  // The shorthand prints the redline's agent view and writes nothing; -o writes it.
  const word = path.join(tmp, "short-a_v_short-b.docx");
  const shown = run(a, b);
  assert.equal(shown.code, 0, shown.err);
  assert.ok(shown.out.startsWith(`---\nsource: ${word} (not written; -o keeps it)\n`), shown.out);
  assert.ok(shown.out.includes("{~~30~>45~~}"), shown.out);
  assert.ok(!fs.existsSync(word));
  // The view is the output, so --quiet does not hide it (pi review av5 F6);
  // read options apply to it.
  assert.ok(run(a, b, "--quiet").out.includes("{~~30~>45~~}"));
  const acceptedView = run(a, b, "--track-changes", "accept");
  assert.equal(acceptedView.code, 0, acceptedView.err);
  assert.ok(!acceptedView.out.includes("{~~") && acceptedView.out.includes("rev #"), acceptedView.out);
  const compared = run(a, b, "-o", word, "--quiet");
  assert.equal(compared.code, 0, compared.err);
  assert.equal(compared.out, "");
  assert.equal(fs.readFileSync(word).subarray(0, 2).toString(), "PK");
  // --changed --by keeps the author's block, by handle or by name (pi
  // review av3 F8: only the (none) path was tested).
  const signed = path.join(tmp, "signed.docx");
  assert.equal(run(a, b, "-o", signed, "--author", "Ann Counsel", "--quiet").code, 0);
  for (const by of ["AC", "Ann Counsel"]) {
    const mine = run("read", signed, "--changed", "--by", by, "--no-page-markers");
    assert.equal(mine.code, 0, mine.err);
    assert.ok(mine.out.includes("\nrange: changed by @AC (p0) of p0") && mine.out.includes("{~~30~>45~~}"), mine.out);
  }
  // One file is the agent view (read).
  const one = run(untracked, "--no-page-markers");
  assert.equal(one.code, 0, one.err);
  assert.equal(one.out, run("read", untracked, "--no-page-markers").out);
  assert.ok(one.out.startsWith("---\nsource: no-tracked-changes.docx\n"), one.out);
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
  assert.equal(word.out, "");
});

test("unknown output suffix defaults to Word for Word inputs (integration)", () => {
  const out = path.join(tmp, "word-fallback.unknown");
  const result = run("diff", path.join(pair, "base.docx"), path.join(pair, "next.docx"), "-o", out);
  assert.equal(result.code, 0, result.err);
  assert.equal(fs.readFileSync(out).subarray(0, 4).toString("hex"), "504b0304");
  assert.ok(result.out.length > 0);
});


test("explicit Markdown input and UTF-8 BOM share native text behavior", () => {
  const a = path.join(tmp, "declared-old.docx"), b = path.join(tmp, "declared-new.docx");
  fs.writeFileSync(a, "\ufeffDue 30 days.\n");
  fs.writeFileSync(b, "Due 45 days.\n");
  const result = run("diff", a, b, "--from", "md", "--format", "word", "--full-lines");
  assert.equal(result.code, 0, result.err);
  assert.equal(result.out, "Due {~~30~>45~~} days.\n");
});


test("diff PDF forwards page options (integration)", () => {
  const wasm = createRequire(bin)("jubarte-wasm");
  const long = Array.from({ length: 120 }, (_, i) => `Paragraph ${i}.`).join("\n\n");
  const a = path.join(tmp, "page-options-old.md"), b = path.join(tmp, "page-options-new.md");
  fs.writeFileSync(a, long);
  fs.writeFileSync(b, long.replace("Paragraph 0.", "Paragraph zero."));
  const all = path.join(tmp, "diff-all.pdf"), kept = path.join(tmp, "diff-kept.pdf");
  for (const [output, flags] of [[all, []], [kept, ["--changed-only"]]]) {
    const result = run("diff", a, b, "-o", output, ...flags);
    assert.equal(result.code, 0, result.err);
  }
  assert.ok(wasm.pdfPageCount(fs.readFileSync(all)) > 1);
  assert.equal(wasm.pdfPageCount(fs.readFileSync(kept)), 1);
  const end = path.join(tmp, "diff-end.pdf");
  const moved = run("diff", tracked, tracked, "-o", end, "--move-comments");
  assert.equal(moved.code, 0, moved.err);
  assert.equal(wasm.pdfPageCount(fs.readFileSync(end)), 2);
});

test("Markdown refuses render-only options (integration)", () => {
  const source = path.join(tmp, "page-options-draft.md");
  fs.writeFileSync(source, "# Draft\n");
  for (const flag of ["--move-comments", "--changed-only"]) {
    const result = run("convert", source, flag);
    assert.equal(result.code, 1, result.err);
    assert.match(result.err, /applies to PDF or PNG output only/);
  }
  assert.ok(!fs.existsSync(path.join(tmp, "page-options-draft.docx")));
});

test("I/O failures report the operation and leave no output (integration)", () => {
  const missing = run("text", "absent.docx");
  assert.equal(missing.code, 1);
  assert.match(missing.err, /reading absent\.docx:/);
  assert.equal(missing.out, "");
  const unwritable = path.join(tmp, "missing-parent", "accepted.docx");
  const failure = run("accept", tracked, "-o", unwritable);
  assert.equal(failure.code, 1);
  assert.match(failure.err, /writing .*accepted\.docx:/);
  assert.equal(failure.out, "");
  assert.ok(!fs.existsSync(unwritable));
});

test("tracked text projections retain their contracts (integration)", () => {
  for (const mode of ["accept", "reject"]) {
    const text = run("text", tracked, "--track-changes", mode);
    assert.equal(text.code, 0, text.err);
    assert.ok(text.out.length > 0);
    const output = path.join(tmp, `project-${mode}.pdf`);
    const pdf = run("convert", tracked, "--track-changes", mode, "-o", output);
    assert.equal(pdf.code, 0, pdf.err);
    assert.equal(fs.readFileSync(output).subarray(0, 5).toString(), "%PDF-");
  }
});

test("Markdown conversion supports a reference, inferred output and PDF (integration)", () => {
  const source = path.join(tmp, "reference-draft.md");
  fs.writeFileSync(source, "# Draft\n\nA clause.\n");
  const docx = run("convert", source, "--reference-doc", untracked);
  assert.equal(docx.code, 0, docx.err);
  assert.match(docx.out, /reference-draft\.docx/);
  assert.match(run("text", path.join(tmp, "reference-draft.docx")).out, /A clause\./);
  const pdf = run("convert", source, "--pdf");
  assert.equal(pdf.code, 0, pdf.err);
  assert.equal(fs.readFileSync(path.join(tmp, "reference-draft.pdf")).subarray(0, 5).toString(), "%PDF-");
});

test("comparison refuses Markdown output of Word inputs and sniffs unknown input suffixes (integration)", () => {
  // As natively, a .md output is CriticMarkup of two Markdown documents.
  const markdown = path.join(tmp, "compare-output.md");
  const result = run("compare", path.join(pair, "base.docx"), path.join(pair, "next.docx"), "-o", markdown);
  assert.equal(result.code, 1, result.out);
  assert.match(result.err, /Markdown output needs both documents in Markdown/);
  assert.ok(!fs.existsSync(markdown));
  const a = copy(path.join(pair, "base.docx"), "sniff-a.bin");
  const b = copy(path.join(pair, "next.docx"), "sniff-b.bin");
  const inferred = run("diff", a, b);
  assert.equal(inferred.code, 0, inferred.err);
  assert.equal(fs.readFileSync(path.join(tmp, "sniff-a_v_sniff-b.docx")).subarray(0, 4).toString("hex"), "504b0304");
});

test("edit previews write nothing and quiet application still writes its report (integration)", () => {
  const plan = path.join(tmp, "coverage-edit-plan.json");
  const paragraph = JSON.parse(run("inspect", untracked, "--json").out).paragraphs.find((p) => p.text.length > 3);
  fs.writeFileSync(plan, JSON.stringify({ schema_version: 1, author: "Legal", date: "2026-01-02T03:04:05Z", operations: [{ id: "replace", kind: "replace", paragraph: { index: paragraph.index }, find: paragraph.text.split(" ")[0], replacement: "Readers" }] }));
  const dir = path.join(tmp, "coverage-preview");
  const preview = run("edit", untracked, "--plan", plan, "--out-dir", dir, "--dry-run");
  assert.equal(preview.code, 0, preview.err);
  assert.match(preview.out, /"ev":"summary"/);
  assert.ok(!fs.existsSync(dir));
  const applied = run("edit", untracked, "--plan", plan, "--out-dir", dir, "--quiet");
  assert.equal(applied.code, 0, applied.err);
  assert.equal(applied.out, "");
  assert.ok(fs.readFileSync(path.join(dir, "report.jsonl"), "utf8").includes('"ev":"save"'));
  const own = run("edit", tracked, "--plan", plan, "--out-dir", path.dirname(tracked), "--force");
  assert.equal(own.code, 1);
  assert.match(own.err, /input's own directory/);
  const badPlan = path.join(tmp, "malformed-plan.json");

  const invalid = run("edit", tracked, "--plan", badPlan, "--out-dir", path.join(tmp, "bad-plan-output"));
  assert.equal(invalid.code, 1);
  assert.match(invalid.err, /reading .*malformed-plan\.json/);
});

test("unsupported host options are rejected before input I/O (integration)", () => {
  const comparison = [
    ["--powertools-faithful"], ["--detail-threshold", "0.1"], ["--no-paragraph-merge"],
  ];
  for (const flags of comparison) {
    const result = run("compare", "missing-a.docx", "missing-b.docx", ...flags);
    assert.equal(result.code, 2, `${flags}: ${result.err}`);
    assert.match(result.err, /not supported/);
    assert.doesNotMatch(result.err, /reading/);
  }
  for (const flags of [["--from", "docx"], ["--resource-path", "."], ["--fail-on-substitution"], ["--no-page-markers"], ["--dpi", "120"], ["--pages", "1"], ["--report", "report.json"], ["--font-report", "fonts.json"], ["--to", "md"], ["-o", "output.md"], ["--to", "docx"], ["-o", "output.docx"]]) {
    const result = run("convert", "missing.docx", ...flags);
    assert.equal(result.code, 2, `${flags}: ${result.err}`);
    assert.doesNotMatch(result.err, /reading/);
    // pi review av2 F16: the advice names a read that runs as written.
    if (flags.includes("md") || flags.includes("output.md")) {
      assert.match(result.err, /read FILE/);
      assert.doesNotMatch(result.err, /--track-changes/);
    }
  }
  for (const flags of [["--pdf"], ["--png"], ["--dpi", "120"], ["--revisions", "word"]]) {
    const result = run("edit", "missing.docx", "--plan", "missing.json", "--out-dir", "missing-dir", ...flags);
    assert.equal(result.code, 2, `${flags}: ${result.err}`);
    assert.match(result.err, /not supported/);
    assert.doesNotMatch(result.err, /reading/);
  }
  for (const flags of [["--reference-doc", "missing.docx"], ["--critic"]]) {
    const result = run("diff", "missing-a.docx", "missing-b.docx", ...flags);
    assert.equal(result.code, 2, `${flags}: ${result.err}`);
    assert.doesNotMatch(result.err, /reading/);
  }
});

test("convert to Word sniffs inputs whose name does not say Markdown (integration)", () => {
  for (const name of ["sniff-draft.txt", "sniff-notes.mkd", "SNIFF"]) {
    const file = path.join(tmp, name), out = path.join(tmp, `${name}.docx`);
    fs.writeFileSync(file, "# Notes\n\nDue in 30 days.\n");
    const result = run("convert", file, "-o", out);
    assert.equal(result.code, 0, `${name}: ${result.err}`);
    assert.equal(fs.readFileSync(out).subarray(0, 2).toString(), "PK");
  }
  // A Word file under a name that says nothing is still refused, unwritten.
  const zip = copy(untracked, "SNIFF-WORD"), out = path.join(tmp, "sniff-word.docx");
  const refused = run("convert", zip, "-o", out);
  assert.equal(refused.code, 2, refused.err);
  assert.match(refused.err, /--to docx requires Markdown input/);
  assert.ok(!fs.existsSync(out));
});

test("diff, compare and convert follow the native input and output contract (integration)", () => {
  const a = path.join(tmp, "contract-a.md"), b = path.join(tmp, "contract-b.md");
  fs.writeFileSync(a, "Due in 30 days.\n");
  fs.writeFileSync(b, "Due in 45 days.\n");
  const word = path.join(tmp, "contract-a.docx");
  assert.equal(run("convert", a, "--to", "docx", "-o", word).code, 0);

  // Markdown output needs both documents in Markdown, in diff and compare.
  for (const args of [["diff", word, b, "-o", path.join(tmp, "contract-d.md")], ["compare", word, b, "-o", path.join(tmp, "contract-c.md")]]) {
    const result = run(...args);
    assert.equal(result.code, 1, result.out);
    assert.match(result.err, /Markdown output needs both documents in Markdown/);
    assert.ok(!fs.existsSync(args.at(-1)));
  }

  // --format critic writing a redline says so and prints no CriticMarkup;
  // the patch keeps stdout to itself.
  const redline = path.join(tmp, "contract-critic.docx");
  const critic = run("diff", word, b, "--format", "critic", "-o", redline);
  assert.equal(critic.code, 0, critic.err);
  assert.doesNotMatch(critic.out, /\{~~/);
  assert.match(critic.out, /^wrote /);
  const patch = run("diff", word, b, "-o", path.join(tmp, "contract-patch.docx"));
  assert.equal(patch.code, 0, patch.err);
  assert.doesNotMatch(patch.out, /wrote/);
  assert.match(patch.err, /wrote/);

  // --from md holds for the paragraph patch: Word bytes are not UTF-8.
  const forced = run("diff", word, word, "--from", "md");
  assert.equal(forced.code, 1, forced.out);
  assert.match(forced.err, /Markdown must be UTF-8/);
  assert.equal(forced.out, "");

  // Markdown is told from the bytes when the name says nothing.
  const notes = path.join(tmp, "NOTES");
  fs.writeFileSync(notes, "Plain notes.\n");
  const converted = run("convert", notes, "-o", path.join(tmp, "notes.pdf"));
  assert.equal(converted.code, 0, converted.err);

  // Malformed UTF-8 is refused, not replaced.
  const broken = path.join(tmp, "broken.md");
  fs.writeFileSync(broken, Buffer.from([0x41, 0xff, 0x42, 0x0a]));
  const refused = run("convert", broken, "-o", path.join(tmp, "broken.pdf"));
  assert.equal(refused.code, 1, refused.out);
  assert.match(refused.err, /Markdown must be UTF-8/);
  assert.ok(!fs.existsSync(path.join(tmp, "broken.pdf")));
});

test("edit keeps its files when the view cannot be read back", () => {
  // pi review r392b tests F6: a preload makes changedView throw; the edit is
  // written, so the failure is a warning and the exit code stays 0.
  const preload = path.join(tmp, "broken-view.cjs");
  fs.writeFileSync(
    preload,
    `const wasm = require(${JSON.stringify(path.join(cliDir, "node_modules", "jubarte-wasm"))});\n` +
      `wasm.changedView = () => { throw new Error("read back failed"); };\n`,
  );
  const first = run("inspect", untracked, "--json");
  const para = JSON.parse(first.out).paragraphs.find((p) => p.text.length > 3);
  const plan = path.join(tmp, "plan-broken-view.json");
  fs.writeFileSync(plan, JSON.stringify({ schema_version: 1, author: "Claude", date: "2026-09-25T12:00:00Z", operations: [{ kind: "replace", paragraph: { index: para.index }, find: para.text.split(" ")[0], replacement: "Changed" }] }));
  const dir = path.join(tmp, "review-broken-view");
  const r = spawnSync(process.execPath, ["--require", preload, bin, "edit", untracked, "--plan", plan, "--out-dir", dir], { encoding: "utf8", cwd: tmp });
  assert.equal(r.status, 0, r.stderr);
  assert.ok(fs.existsSync(path.join(dir, "redline.docx")));
  assert.match(r.stderr, /warning: the changed blocks cannot be shown: read back failed/);
});
