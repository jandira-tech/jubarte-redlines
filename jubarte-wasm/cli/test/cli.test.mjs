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
import zlib from "node:zlib";

const here = path.dirname(fileURLToPath(import.meta.url));
const cliDir = path.resolve(here, "..");
const bin = path.join(cliDir, "bin", "jubarte-redlines.mjs");
const repo = path.resolve(cliDir, "..", "..");
const pair = path.join(repo, "tests/corpus/batch_to_fix/pairs/41_heading_1_bold_demo_id_paraid_overflow_heading_1_style_demo_id_paraid_overflow");
const tracked = path.join(repo, "tests/fixtures/from-docx/critic/tracked-changes.docx");
const untracked = path.join(repo, "tests/fixtures/from-docx/critic/no-tracked-changes.docx");
let tmp;
// The checked-in binding has #1255's JSON errors and table picks once it is
// rebuilt from a source that has them; until then their tests skip.
const binding = createRequire(import.meta.url)(path.join(cliDir, "..", "npm"));
const jsonErrors = typeof binding.runtimeJsonError === "function";
const inputNotices = typeof binding.inputNotice === "function";
const inputValidity = typeof binding.inputValidityLine === "function";
// #1690's flags (`changes --author`, `read --tables -o`) parse once the
// binding is rebuilt from a source that has them; until then their tests skip.
const parses = (argv) => JSON.parse(binding.parseCli(JSON.stringify(argv))).exit_code === 0;
const authorFilter = parses(["changes", "a.docx", "--author", "Ann"]);
const tablesOutput = parses(["read", "a.docx", "--tables", "-o", "t.md"]);

/** Every stdout line of `changes --json` is a record (#1925). */
function records(out) {
  const rows = out.split("\n").filter(Boolean).map((line) => JSON.parse(line));
  assert.ok(rows.every((row) => ["kind", "id", "paragraph"].every((field) => field in row)), out);
  return rows;
}

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
  return runEnv({}, ...args);
}

function runEnv(env, ...args) {
  const r = spawnSync(process.execPath, [bin, ...args], {
    encoding: "utf8",
    cwd: tmp,
    env: { ...process.env, ...env },
  });
  return { code: r.status, out: r.stdout, err: r.stderr };
}

// The checked-in binding's parser predates #1690 and refuses `--tables -o`
// ("--tables prints the grids; drop -o"). Until the binding is rebuilt from
// a source that has the flag, the host's own behaviour is exercised through
// a copy of the bin whose binding parses it: the parser's own result for
// `--tables`, with the output the flag names. Once the binding has the
// flag, the bin itself is the one under test.
function tablesBin() {
  const host = path.join(tmp, "tables-host");
  const modules = path.join(host, "node_modules", "jubarte-wasm");
  fs.mkdirSync(modules, { recursive: true });
  fs.writeFileSync(path.join(host, "jubarte-redlines.mjs"), fs.readFileSync(bin));
  fs.writeFileSync(path.join(modules, "package.json"), JSON.stringify({ name: "jubarte-wasm", main: "index.js" }));
  fs.writeFileSync(path.join(modules, "index.js"), [
    `const real = require(${JSON.stringify(path.join(cliDir, "..", "npm"))});`,
    "const parse = real.parseCli;",
    "module.exports = {",
    "  ...real,",
    "  parseCli(argv, ...rest) {",
    "    const args = JSON.parse(argv);",
    "    const out = args.indexOf('-o');",
    "    if (!args.includes('--tables') || out === -1) return parse(argv, ...rest);",
    "    const stripped = args.filter((_, i) => i !== out && i !== out + 1);",
    "    const parsed = JSON.parse(parse(JSON.stringify(stripped), ...rest));",
    "    parsed.args.output = args[out + 1];",
    "    parsed.args.output_format = 'md';",
    "    return JSON.stringify(parsed);",
    "  },",
    "};",
  ].join("\n"));
  return path.join(host, "jubarte-redlines.mjs");
}

/** `read --tables` with the flag the checked-in parser may not know yet. */
function runTables(...args) {
  const target = tablesOutput ? bin : tablesBin();
  const r = spawnSync(process.execPath, [target, ...args], { encoding: "utf8", cwd: tmp });
  return { code: r.status, out: r.stdout, err: r.stderr };
}

// Integration-only syscall hooks: Node's own mock utility delegates to the
// real filesystem, but lets us stop at an exact boundary without timing races.
function runWithPlanHook(plan, hook, args, context) {
  const source = `
    import assert from "node:assert/strict";
    import fs from "node:fs";
    import { mock } from "node:test";
    const plan = ${JSON.stringify(plan)};
    const verify = (${hook.toString()})(fs, mock, assert, plan, ${JSON.stringify(context) ?? "undefined"});
    process.argv = [process.execPath, ${JSON.stringify(bin)}, ...${JSON.stringify(args)}];
    try {
      await import(${JSON.stringify(bin)});
      verify();
      process.exitCode = 0;
    } finally {
      mock.restoreAll();
    }
  `;
  const r = spawnSync(process.execPath, ["--input-type=module", "-e", source], { encoding: "utf8", cwd: tmp });
  return { code: r.status, out: r.stdout, err: r.stderr };
}

/// One part of a .docx, by name, read from its central directory.
function part(file, name) {
  const zip = fs.readFileSync(file);
  let at = zip.lastIndexOf(Buffer.from([0x50, 0x4b, 0x05, 0x06]));
  const count = zip.readUInt16LE(at + 10);
  at = zip.readUInt32LE(at + 16);
  for (let i = 0; i < count; i++) {
    const method = zip.readUInt16LE(at + 10);
    const size = zip.readUInt32LE(at + 20);
    const nameLength = zip.readUInt16LE(at + 28);
    const skip = zip.readUInt16LE(at + 30) + zip.readUInt16LE(at + 32);
    const local = zip.readUInt32LE(at + 42);
    if (zip.toString("utf8", at + 46, at + 46 + nameLength) === name) {
      const start = local + 30 + zip.readUInt16LE(local + 26) + zip.readUInt16LE(local + 28);
      const data = zip.subarray(start, start + size);
      return (method === 8 ? zlib.inflateRawSync(data) : data).toString("utf8");
    }
    at += 46 + nameLength + skip;
  }
  throw new Error(`${file} has no ${name}`);
}

function copy(from, name) {
  const to = path.join(tmp, name);
  fs.copyFileSync(from, to);
  return to;
}

test("field refresh reads its input first and overwrites nothing without --force", () => {
  const out = path.join(tmp, "field-refresh.docx");
  const report = path.join(tmp, "field-refresh.json");
  fs.writeFileSync(out, "preserve document");
  fs.writeFileSync(report, "preserve report");
  for (const extra of [[], ["--to", "docx"], ["--track-changes", "accept"], ["--track-changes", "reject"], ["--report", report, "--force"]]) {
    const result = run("convert", "missing.docx", "-o", out, "--update-fields", ...extra);
    assert.equal(result.code, 1, result.err);
    assert.equal(result.err, "error: reading missing.docx: No such file or directory (os error 2)\n");
    assert.equal(result.out, "");
  }
  const kept = run("convert", untracked, "-o", out, "--update-fields");
  assert.equal(kept.code, 2, kept.err);
  assert.match(kept.err, /OUTPUT_EXISTS/);
  assert.equal(fs.readFileSync(out, "utf8"), "preserve document");
  assert.equal(fs.readFileSync(report, "utf8"), "preserve report");
  const written = run("convert", untracked, "-o", out, "--update-fields", "--report", report, "--force");
  assert.equal(written.code, 0, written.err);
  assert.equal(typeof JSON.parse(fs.readFileSync(report, "utf8")).page_count, "number");
  assert.equal(fs.readFileSync(out).subarray(0, 2).toString(), "PK");
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
    // Under --json the usage error is a JSON line on stdout too (#1255).
    if (jsonErrors && args.includes("--json")) assert.equal(JSON.parse(result.out).error.code, "USAGE");
    else assert.equal(result.out, "");
    assert.ok(!fs.existsSync(path.join(tmp, "unused.docx")));
  }
  // A retired name keeps its note when the run fails: npm has no validate.
  const audit = run("audit", "missing.docx");
  assert.equal(audit.code, 2, audit.err);
  assert.ok(audit.err.startsWith("note: audit is now validate --rules a11y,style,structure; audit is removed in 0.14\nerror: "), audit.err);
  // `revisions` reads as a file name again; a note names `changes` first.
  const note = "note: revisions is no longer a task, so it reads as a file name; use changes\n";
  assert.ok(run("revisions", "missing.docx", "--json").err.startsWith(note));
  const legacy = run("revisions", "missing.docx");
  assert.equal(legacy.code, 1, legacy.err);
  assert.ok(legacy.err.startsWith(note), legacy.err);
  assert.match(legacy.err, /reading revisions/);
});

test("redline writes a Word redline and refuses to overwrite it", () => {
  const out = path.join(tmp, "redline.docx");
  const r = run(
    "redline", path.join(pair, "base.docx"), path.join(pair, "next.docx"),
    "-d", "1970-01-01T00:00:00Z", "-o", out, "--author", "Legal",
  );
  assert.equal(r.code, 0, r.err);
  assert.equal(r.out, "");
  assert.match(r.err, /^wrote .*redline\.docx \(docx, \d+ bytes\)\n$/);
  assert.deepEqual([...fs.readFileSync(out).subarray(0, 2)], [0x50, 0x4b]);
  const changes = records(run("changes", out, "--json").out);
  assert.ok(changes.length > 0 && changes.every((c) => c.author === "Legal"));
  const again = run("compare", path.join(pair, "base.docx"), path.join(pair, "next.docx"), "-o", out);
  assert.equal(again.code, 2);
  assert.match(again.err, /OUTPUT_EXISTS: output '.*' already exists \(use --force to overwrite\)/);
  assert.equal(run(
    "compare", path.join(pair, "base.docx"), path.join(pair, "next.docx"),
    "-d", "1970-01-01T00:00:00Z", "-o", out, "--force",
  ).code, 0);
});

test("redline prints the view without -o and writes nothing", () => {
  const a = copy(path.join(pair, "base.docx"), "a.docx");
  const b = copy(path.join(pair, "next.docx"), "b.docx");
  const result = run("redline", a, b, "-d", "1970-01-01T00:00:00Z");
  assert.equal(result.code, 0, result.err);
  assert.ok(result.out.startsWith(`---\nsource: ${path.join(tmp, "a_v_b.docx")} (not written; -o keeps it)\n`), result.out);
  assert.ok(!fs.existsSync(path.join(tmp, "a_v_b.docx")));
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
  assert.match(listed.out, /^r0\tdeletion\ttext\tBo Chen\tp1\t" to people that"\n/);
  assert.match(listed.out, /\n\d+ change\(s\)\n$/);
  const rows = records(run("changes", tracked, "--json").out);
  assert.equal(rows[0].kind, "deletion");
  // `revisions` left the CLI: `changes` is the one listing.
  const gone = run("revisions", tracked);
  assert.notEqual(gone.code, 0);
  assert.doesNotMatch(gone.out, /revision\(s\)/);
  const kept = path.join(tmp, "kept.docx");
  assert.equal(run("accept", tracked, "-o", kept, "--kind", "deletion").code, 0);
  const left = records(run("changes", kept, "--json").out);
  assert.ok(left.length > 0 && left.every((c) => c.kind !== "deletion"));
  const base = path.join(tmp, "base.docx");
  assert.equal(run("reject", tracked, "-o", base).code, 0);
  assert.equal(run("changes", base).out, "0 change(s)\n");
});

test("accept/reject -o naming the input refuses in favour of --in-place (#1998)", () => {
  const file = path.join(tmp, "contract.docx");
  fs.copyFileSync(tracked, file);
  const packed = fs.readFileSync(file);
  for (const args of [
      ["accept", file, "-o", file],
    ["accept", file, "-o", file, "--force"],
    ["reject", file, "-o", file, "--force"],
    // Another spelling of the same file is the same refusal.
    ["accept", file, "-o", `${tmp}/./contract.docx`],
    ]) {
      const r = run(...args);
    assert.equal(r.code, 2, `${args.join(" ")}: ${r.err}`);
    assert.match(r.err, /^error: OUTPUT_ALIAS: the output '.*' is the same file as the input '.*'; use --in-place\n$/);
    assert.deepEqual(fs.readFileSync(file), packed);
    }
    // `-o` naming another file still writes it.
  const out = path.join(tmp, "other.docx");
  assert.equal(run("accept", file, "-o", out).code, 0);
  assert.deepEqual(fs.readFileSync(file), packed);
  });

test("errors read as the binary's: bad ids are usage, I/O as Rust words it, sorted report keys", () => {
  const out = path.join(tmp, "never-accepted.docx");
  const bad = run("accept", tracked, "-o", out, "--id", "c5");
  assert.equal(bad.code, 2, bad.err);
  assert.equal(bad.err, "error: c5: expected a change id such as r12, r1+2 or header1.r1\n");
  // The id is checked before any I/O, as natively.
  assert.equal(run("reject", "missing.docx", "-o", out, "--id", "c5").code, 2);
  const unknown = run("accept", tracked, "-o", out, "--id", "r99");
  assert.equal(unknown.code, 1);
  assert.equal(unknown.err, "error: accept failed: no tracked change r99\n");
  assert.ok(!fs.existsSync(out));
  assert.equal(run("reject", tracked, "-o", out, "--id", "r99").err, "error: reject failed: no tracked change r99\n");
  const picked = run("read", tracked, "-p", "c5");
  assert.equal(picked.err, "error: c5: expected pN, pN-pM or tN\n");
  for (const command of ["changes", "read"]) {
    const missing = run(command, "missing.docx");
    assert.equal(missing.code, 1);
    assert.equal(missing.err, "error: reading missing.docx: No such file or directory (os error 2)\n");
  }
  // A refused plan's report lines are compact with sorted keys.
  const refused = run("edit", untracked, "-p", "p99", "--content", "x", "-o", out);
  assert.equal(refused.code, 3, refused.err);
  for (const line of refused.out.trim().split("\n")) {
    const row = JSON.parse(line);
    assert.equal(line, JSON.stringify(Object.fromEntries(Object.entries(row).sort(([a], [b]) => (a < b ? -1 : 1)))));
  }
});

test("read (alias text), inspect, capabilities and convert", () => {
  assert.equal(JSON.parse(run("read", untracked, "--json").out).schema_version, 3);
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
  // `inspect` is `read` now, with a note.
  const retired = run("inspect", untracked);
  assert.equal(retired.out, run("read", untracked).out);
  assert.equal(retired.err, "note: inspect is now read; inspect is removed in 0.14\n");
  const snapshot = JSON.parse(run("inspect", untracked, "--json").out);
  assert.equal(snapshot.schema_version, 3);
  assert.ok(Array.isArray(snapshot.changes) && Array.isArray(snapshot.comments));
  assert.deepEqual(JSON.parse(run("read", untracked, "--json").out), snapshot);
  const wasm = createRequire(bin)("jubarte-wasm");
  // The grids and a selected snapshot, as the binary prints them.
  const tables = run("read", untracked, "--tables");
  assert.equal(tables.code, 0, tables.err);
  assert.match(tables.out, /^table 0: 2x2 header_rows=0 widths=4818,4820\np4=Name\tp5=Value\n/);
  const headed = JSON.parse(run("read", untracked, "--json", "--head", "1").out);
  assert.equal(headed.paragraphs.length, 1);
  // The engine's writer: -o's extension picks the task's Word file, its
  // view (.md, headed by the Word name it did not write), its snapshot
  // (.json) or its drawing (.pdf); the wrote line is on stderr.
  for (const [args, format, check] of [
    [["accept", tracked, "-o", path.join(tmp, "a.pdf")], "pdf", (bytes) => bytes.subarray(0, 5).toString() === "%PDF-"],
    [["reject", tracked, "-o", path.join(tmp, "s.md")], "md", (bytes) => bytes.toString().startsWith(`---\nsource: ${path.join(tmp, "s.docx")} (not written)\n`)],
    [["read", untracked, "-o", path.join(tmp, "v.md")], "md", (bytes) => bytes.toString().startsWith(`---\nsource: ${path.basename(untracked)}\n`)],
    [[untracked, "-o", path.join(tmp, "v.json"), "-p", "p0"], "json", (bytes) => JSON.parse(bytes).paragraphs.length === 1],
    [["convert", tracked, "-o", path.join(tmp, "c.json")], "json", (bytes) => JSON.parse(bytes).changes.length === 5],
    [["accept", tracked, "-o", path.join(tmp, "writer-a.docx")], "docx", (bytes) => bytes.subarray(0, 2).toString() === "PK"],
  ]) {
    const written = run(...args);
    const out = args[args.indexOf("-o") + 1];
    assert.equal(written.code, 0, `${args}: ${written.err}`);
    assert.equal(written.out, "", `${args}`);
    const bytes = fs.readFileSync(out);
    const pages = format === "pdf" ? `, ${wasm.pdfPageCount(bytes)} page` : "";
    assert.match(written.err, new RegExp(`^wrote ${out.replaceAll(".", "\\.")} \\(${format}, ${bytes.length} bytes${pages}s?\\)\n$`), `${args}`);
    assert.ok(check(bytes), `${args}`);
  }
  const manifest = JSON.parse(run("capabilities", "--json").out);
  assert.equal(manifest.runtime, "wasm");
  assert.ok(!manifest.formats.convert.includes(".png"), manifest.formats.convert);
  const summary = run("capabilities").out;
  assert.match(summary, /^engine: jubarte \S+ \(wasm\)\ncompare: yes\n/);
  assert.match(summary, /\npng: no\n/);
  assert.match(summary, /run with --json for the manifest\n$/);
  const lines = summary.split("\n");
  const ops = Object.entries(manifest.operations);
  assert.deepEqual(
    lines.slice(1, ops.length + 1).sort(),
    ops.map(([name, on]) => `${name}: ${on ? "yes" : "no"}`).sort(),
  );
  const formats = lines.find((line) => line.startsWith("formats: ")).slice("formats: ".length);
  assert.deepEqual(
    formats.split("; ").sort(),
    Object.entries(manifest.formats).map(([task, ext]) => `${task} ${ext.join(" ")}`).sort(),
  );
  const pdf = path.join(tmp, "out.pdf");
  const r = run("convert", tracked, "-o", pdf, "--revisions", "word", "--compress");
  assert.equal(r.code, 0, r.err);
  assert.equal(r.out, "");
  assert.match(r.err, /^wrote .*out\.pdf \(pdf, \d+ bytes, \d+ pages?\)\n$/);
  assert.equal(fs.readFileSync(pdf).subarray(0, 5).toString(), "%PDF-");
  const png = run("convert", tracked, "--png");
  assert.equal(png.code, 2);
  assert.match(png.err, /PNG pages need the Python or Rust build/);
  assert.match(run("convert", tracked, "--revisions", "custom").err, /--revision-palette/);
});

test("an unreadable input reads as the binary says, on every task (#454)", () => {
  fs.writeFileSync(path.join(tmp, "fake.docx"), "hello, not a zip");
  fs.copyFileSync(untracked, path.join(tmp, "good.docx"));
  const expected = "error: INVALID_PACKAGE: fake.docx is not a ZIP file: no end-of-central-directory record\n";
  for (const args of [
    ["read", "fake.docx"], ["read", "fake.docx", "--json"], ["read", "fake.docx", "--tables"],
    ["changes", "fake.docx"], ["accept", "fake.docx", "-o", "unread-a.docx"], ["reject", "fake.docx", "-o", "unread-r.docx"],
    ["convert", "fake.docx", "-o", "unread.pdf"], ["compare", "fake.docx", "good.docx"],
    ["compare", "good.docx", "fake.docx", "-o", "unread-c.docx"], ["diff", "fake.docx", "good.docx"],
    ["edit", "fake.docx", "-p", "p0", "--delete"], ["add", "fake.docx", "-p", "p0", "--content", "x"],
  ]) {
    const r = run(...args);
    assert.equal(r.code, 1, args.join(" "));
    assert.equal(r.err, expected, args.join(" "));
    if (args.includes("--json") && jsonErrors) {
      // With --json, stdout says it as JSON too (#1255).
      assert.deepEqual(JSON.parse(r.out), { error: { code: "INVALID_PACKAGE", message: expected.slice("error: ".length, -1) } });
    } else {
      assert.equal(r.out, "", args.join(" "));
    }
  }
  // A Word 97-2003 file names its code first, as the binary does.
  fs.writeFileSync(path.join(tmp, "old.docx"), Buffer.concat([Buffer.from([0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]), Buffer.alloc(512)]));
  assert.match(run("read", "old.docx").err, /^error: LEGACY_DOC: old\.docx is /);
});

test("an invalid plan prints the binary's one line and no report (#454)", () => {
  const bad = path.join(tmp, "bad-plan.json");
  fs.writeFileSync(bad, '{"edits": 3}');
  const r = run("edit", untracked, "--plan", bad, "--out-dir", path.join(tmp, "never"));
  assert.equal(r.code, 3, r.err);
  assert.equal(r.out, "");
  assert.match(r.err, /^error: INVALID_PLAN: schema_version is required \(1 for this build\)/);
  assert.ok(!fs.existsSync(path.join(tmp, "never")));
});

test("Word to Word convert resolves changes and refreshes fields, as the binary does (#454)", () => {
  const accepted = path.join(tmp, "conv-accepted.docx");
  const r = run("convert", tracked, "-o", accepted, "--track-changes", "accept");
  assert.equal(r.code, 0, r.err);
  assert.match(r.err, /^wrote .*conv-accepted\.docx \(docx, \d+ bytes\)\n$/);
  assert.equal(run("changes", accepted).out, "0 change(s)\n");
  const rejected = path.join(tmp, "conv-rejected.docx");
  assert.equal(run("convert", tracked, "-o", rejected, "--track-changes", "reject").code, 0);
  assert.equal(run("changes", rejected).out, "0 change(s)\n");
  const updated = path.join(tmp, "conv-fields.docx");
  const fields = run("convert", untracked, "-o", updated, "--update-fields");
  assert.equal(fields.code, 0, fields.err);
  assert.match(fields.err, /^\d+ field\(s\) written; \d+ page\(s\)\nwrote /);
  const plain = run("convert", untracked, "-o", path.join(tmp, "conv-plain.docx"));
  assert.equal(plain.code, 1);
  assert.match(plain.err, /is already Word: give --track-changes accept or reject, --update-fields, or another --to\n$/);
});

test("convert --to md writes the paged Markdown, as the binary does (#454)", () => {
  const out = path.join(tmp, "conv-view.md");
  const r = run("convert", tracked, "-o", out, "--track-changes", "accept");
  assert.equal(r.code, 0, r.err);
  assert.match(r.err, /^wrote .*conv-view\.md \(md, \d+ bytes\)\n$/);
  assert.match(fs.readFileSync(out, "utf8"), /^<!-- page 1 of \d+ -->\n/);
  const bare = path.join(tmp, "conv-bare.txt");
  assert.equal(run("convert", untracked, "-o", bare, "--no-page-markers").code, 0);
  assert.doesNotMatch(fs.readFileSync(bare, "utf8"), /<!-- page /);
});

test("edit --pdf writes the PDF beside the document, as the binary does (#454)", () => {
  const out = path.join(tmp, "edit-pdf.docx");
  const r = run("edit", untracked, "-p", "p0", "--content", "Z", "-a", "Jubarte", "-o", out, "--pdf", "--revisions", "word", "--dpi", "50");
  assert.equal(r.code, 0, r.err);
  assert.match(r.err, /wrote .*edit-pdf\.pdf \(pdf, \d+ bytes, \d+ pages?\)\n/);
  assert.ok(fs.readFileSync(path.join(tmp, "edit-pdf.pdf")).subarray(0, 4).equals(Buffer.from("%PDF")));
  const dir = path.join(tmp, "edit-pdf-bundle");
  assert.equal(run("edit", untracked, "-p", "p0", "--content", "Z", "-a", "Jubarte", "--out-dir", dir, "--pdf").code, 0);
  for (const name of ["redline.pdf", "clean.pdf"]) assert.ok(fs.existsSync(path.join(dir, name)), name);
});

test("capabilities claims only what this CLI does (#454, #447, #453)", () => {
  const manifest = JSON.parse(run("capabilities", "--json").out);
  for (const op of ["append", "scrub", "validate", "repair", "audit", "comments", "diff_render", "page_ranges", "png"]) {
    assert.equal(manifest.operations[op], false, op);
  }
  for (const op of ["compare", "accept_revisions", "edit", "markdown", "markdown_to_docx", "patch", "pdf", "fields"]) {
    assert.equal(manifest.operations[op], true, op);
  }
  assert.deepEqual(Object.keys(manifest.formats).sort(), ["accept", "add", "compare", "convert", "diff", "edit", "read", "reject"]);
  const summary = run("capabilities").out;
  for (const op of ["append", "scrub", "validate", "audit"]) assert.match(summary, new RegExp(`\\n${op}: no\\n`), op);
  assert.doesNotMatch(summary, /append \./);
});

test("capability query: --matrix and --operation answer through the wasm engine", () => {
  const wasm = createRequire(import.meta.url)("../../npm/node/jubarte_wasm.js");
  // The CLI's --matrix is the engine's capabilityQuery with no arguments.
  const matrix = JSON.parse(run("capabilities", "--matrix", "--json").out);
  const direct = JSON.parse(wasm.capabilityQuery(undefined, undefined));
  assert.equal(matrix.runtime, "wasm");
  assert.deepEqual(matrix.edit_operations, direct.edit_operations);
  // #1252's strike and restore join the kinds once the binding has them.
  assert.equal(matrix.edit_operations.length, matrix.edit_operations.includes("strike") ? 26 : 24);
  assert.deepEqual(matrix.edit_plan_versions, [1]);
  // Every advertised kind answers a focused query as a dispatched operation.
  for (const kind of matrix.edit_operations) {
    const answer = JSON.parse(run("capabilities", "--operation", kind, "--json").out);
    assert.equal(answer.supported, true, kind);
    const v1 = answer.profiles.find((p) => p.plan_version === 1);
    assert.equal(v1.dispatched, true, kind);
  }
  // A name the engine refuses is not advertised, and the query says so.
  assert.ok(!matrix.edit_operations.includes("definitely_not_an_operation"));
  const refused = JSON.parse(wasm.capabilityQuery("definitely_not_an_operation", undefined));
  assert.equal(refused.supported, false);
  assert.equal(refused.reason, "unsupported operation");
  assert.equal(refused.nearest.length, 3);
  // Field queries suggest the nearest valid name.
  const near = JSON.parse(wasm.capabilityQuery("reply_comment", "tex"));
  assert.equal(near.supported, false);
  assert.equal(near.nearest[0], "text");
  // The text rendering names the runtime without JSON quotes.
  assert.match(wasm.capabilityQueryText("reply_comment", undefined), /^runtime: /);
});

test("convert --scholar and --tracking, as the binary has them", () => {
  const source = path.join(tmp, "paper.md");
  fs.writeFileSync(source, "# Title\n\nBody.\n");
  const out = path.join(tmp, "paper.docx");
  assert.equal(run("convert", source, "-o", out, "--scholar", "--tracking", "on").code, 0);
  assert.match(part(out, "word/styles.xml"), /Latin Modern Roman/);
  assert.match(part(out, "word/settings.xml"), /<w:trackRevisions/);
  assert.match(part(out, "word/document.xml"), /<w:titlePg\/>/);
  assert.equal(run("convert", source, "-o", out, "--force").code, 0);
  assert.doesNotMatch(part(out, "word/styles.xml"), /Latin Modern Roman/);
  assert.doesNotMatch(part(out, "word/settings.xml"), /<w:trackRevisions/);
});

test("a replaced output keeps its mode, as the binary's does (#1698)", () => {
  const source = path.join(tmp, "mode.md");
  fs.writeFileSync(source, "Body.\n");
  const out = path.join(tmp, "mode.docx");
  fs.writeFileSync(out, "old");
  fs.chmodSync(out, 0o640);
  const replaced = run("convert", source, "-o", out, "--force");
  assert.equal(replaced.code, 0, replaced.err);
  assert.equal(fs.statSync(out).mode & 0o777, 0o640);
  assert.deepEqual(fs.readdirSync(tmp).filter((n) => n.endsWith(".partial")), []);
});

test("convert --move-comments and --changed-only", () => {
  const pages = (r) => {
    assert.equal(r.code, 0, r.err);
    return Number(/, (\d+) pages?\)/.exec(r.err)[1]);
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
  const first = run("read", untracked, "--json");
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
  assert.equal(run("edit", untracked, "--plan", plan, "--out-dir", dir).code, 2);
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
  const para = JSON.parse(run("read", untracked, "--json").out).paragraphs.find((p) => p.text.length > 3);
  const word = para.text.split(" ")[0];
  const dir = path.join(tmp, "flags");
  const r = run("edit", untracked, "-p", `p${para.index}`, "--anchor", word, "--content", "Changed", "--author", "Ann Counsel", "--out-dir", dir);
  assert.equal(r.code, 0, r.err);
  for (const name of ["clean.docx", "redline.docx", "patch.diff", "report.jsonl"]) assert.ok(fs.existsSync(path.join(dir, name)), name);
  // The patch names the edited file, as the binary's bundle does.
  const base = path.basename(untracked);
  assert.ok(fs.readFileSync(path.join(dir, "patch.diff"), "utf8").startsWith(`--- a/${base}\n+++ b/${base}\tAnn Counsel\t`));
  assert.match(r.out, new RegExp(`\\nrange: changed by @AC \\(p${para.index}\\) of p0-`));
  assert.ok(r.out.includes(`{~~${word}~>Changed~~}`), r.out);
  assert.doesNotMatch(r.out, /\n@@ /);
  // A recording takes no --out-dir; a flag before any -p and a bare -p are usage errors.
  for (const bad of [["--plan", "x.json", "-p", "p1", "--delete"], ["--anchor", "a", "-p", "p1", "--content", "b"], ["-p", "p1"]]) {
    assert.equal(run("edit", untracked, ...bad, "--out-dir", path.join(tmp, "flags-bad")).code, 2, bad.join(" "));
  }
  const comment = run("add", untracked, "-p", `p${para.index}`, "--anchor", word, "--content", "Why?", "--author", "Ann Counsel", "--out-dir", path.join(tmp, "flags-comment"));
  assert.equal(comment.code, 0, comment.err);
  assert.ok(comment.out.includes(`{==${word}==}{>>c0 @AC: Why?<<}`), comment.out);
  const editing = run("add", untracked, "-p", `p${para.index}`, "--content", "Recitals", "--editing-mode", "--out-dir", path.join(tmp, "flags-editing"));
  assert.equal(editing.code, 0, editing.err);
  assert.ok(!fs.existsSync(path.join(tmp, "flags-editing", "redline.docx")));
  // The new paragraph after a heading keeps the heading's style here (`# Recitals`).
  assert.match(editing.out, /\n(# )?Recitals\n/);
  assert.ok(!editing.out.includes("{++"), editing.out);
});

// An engine built before #1264 refuses the plan field the flag sets.
const ownershipShipped = binding.planError(JSON.stringify({
  schema_version: 1, author: "A", rewrite_others_comments: false,
  operations: [{ kind: "delete", paragraph: "p0", find: "x" }],
})) === undefined;
test("edit allows another author's comment unless JUBARTE_KEEP_OTHERS=1 (#1892)", {
  skip: !ownershipShipped && "the checked-in engine predates #1264; it ships with the next release",
}, () => {
  const para = JSON.parse(run("read", untracked, "--json").out).paragraphs.find((p) => p.text.length > 3);
  const word = para.text.split(" ")[0];
  const commented = path.join(tmp, "ownership-commented.docx");
  const added = run("add", untracked, "-p", `p${para.index}`, "--anchor", word, "--content", "Why?", "--author", "Ann Counsel", "-o", commented);
  assert.equal(added.code, 0, added.err);
  const out = path.join(tmp, "ownership-edited.docx");
  const edit = ["edit", commented, "-p", "c0", "--content", "Why not?", "--author", "Bob Lee", "-o", out];
  const allowed = run(...edit);
  assert.equal(allowed.code, 0, allowed.err);
  assert.ok(allowed.err.includes('note: op-1 (edit_comment): c0 is by "Ann Counsel"; their words were changed under their name'), allowed.err);
  assert.ok(allowed.out.includes("{>>c0 @AC: Why not?<<}"), allowed.out);
  const kept = path.join(tmp, "ownership-kept.docx");
  const refused = runEnv({ JUBARTE_KEEP_OTHERS: "1" }, ...edit.slice(0, -2), "-o", kept);
  assert.equal(refused.code, 3, refused.err);
  assert.ok(refused.err.includes('COMMENT_NOT_YOURS (op-1): c0 is by "Ann Counsel"; reply instead'), refused.err);
  assert.ok(!fs.existsSync(kept));
});

test("edit and add write FILE_modified.docx, as the binary does (D15)", () => {
  const para = JSON.parse(run("read", untracked, "--json").out).paragraphs.find((p) => p.text.length > 3);
  const word = para.text.split(" ")[0];
  const file = copy(untracked, "single.docx");
  const written = path.join(tmp, "single_modified.docx");
  const edit = ["edit", file, "-p", `p${para.index}`, "--anchor", word, "--content", "Changed", "-a", "Jubarte"];
  const r = run(...edit);
  assert.equal(r.code, 0, r.err);
  assert.ok(fs.existsSync(written) && !fs.existsSync(path.join(tmp, "single.edit")));
  assert.ok(r.out.includes(`\nsource: ${written}\n`), r.out);
  assert.ok(r.err.startsWith(`wrote ${written} (docx, `), r.err);
  // An existing output is a usage error until --force.
  const again = run(...edit);
  assert.equal(again.code, 2, again.err);
  assert.match(again.err, /already exists \(use --force to overwrite\)/);
  const quiet = run(...edit, "--force", "-q");
  assert.equal(quiet.code, 0, quiet.err);
  assert.equal(quiet.out + quiet.err, "");
  // -q also goes before the task.
  const quietFirst = run("-q", ...edit, "--force");
  assert.equal(quietFirst.code, 0, quietFirst.err);
  assert.equal(quietFirst.out + quietFirst.err, "");
  // -o names the one .docx; any other extension, or -o with --out-dir, is a usage error.
  const named = path.join(tmp, "single-named.docx");
  assert.equal(run(...edit, "-o", named).code, 0);
  assert.ok(fs.existsSync(named));
  for (const bad of [["-o", path.join(tmp, "single.pdf")], ["-o", named, "--out-dir", path.join(tmp, "single-r")]]) {
    assert.equal(run(...edit, ...bad).code, 2, bad.join(" "));
  }
  assert.ok(!fs.existsSync(path.join(tmp, "single.pdf")) && !fs.existsSync(path.join(tmp, "single-r")));
  // --out-dir alone writes the bundle beside FILE.
  const bundle = run(...edit, "--out-dir");
  assert.equal(bundle.code, 0, bundle.err);
  assert.ok(fs.existsSync(path.join(tmp, "single.edit", "redline.docx")));
  // The plan runs first: a refused plan reports even over an existing output.
  const refused = run("edit", file, "-p", `p${para.index}`, "--anchor", "absent words", "--content", "x");
  assert.equal(refused.code, 3, refused.err);
  assert.match(refused.out, /absent words/);
  // The bundle's wrote line is a status line, on stderr.
  assert.ok(bundle.err.startsWith(`wrote ${path.join(tmp, "single.edit")} (`), bundle.err);
  assert.doesNotMatch(bundle.out, /\nwrote /);
  // A _modified FILE keeps its name: add edits it in place, with --force.
  const add = ["add", written, "-p", `p${para.index}`, "--content", "New.", "-a", "Jubarte"];
  assert.equal(run(...add).code, 2);
  const forced = run(...add, "--force");
  assert.equal(forced.code, 0, forced.err);
  assert.ok(forced.out.includes(`\nsource: ${written}\n`), forced.out);
});

test("a bare --plan is FILE_plan.json, as the binary has it (D16)", () => {
  const para = JSON.parse(run("read", untracked, "--json").out).paragraphs.find((p) => p.text.length > 3);
  const file = copy(untracked, "bare.docx");
  const plan = path.join(tmp, "bare_plan.json");
  const recorded = run("edit", file, "--plan", "-p", `p${para.index}`, "--content", "Changed.");
  assert.equal(recorded.code, 0, recorded.err);
  assert.ok(fs.existsSync(plan), recorded.out);
  assert.ok(recorded.out.trim().endsWith(`apply with: jubarte edit ${file} --plan`), recorded.out);
  const applied = run("edit", file, "--plan", "--consume", "-q");
  assert.equal(applied.code, 0, applied.err);
  assert.ok(!fs.existsSync(plan) && fs.existsSync(path.join(tmp, "bare_modified.docx")));
  const swallowed = run("edit", "--plan", file, "-p", "p1", "--content", "x");
  assert.equal(swallowed.code, 2);
  assert.match(swallowed.err, /names a Word document; put FILE before --plan/);
});

test("edit takes the text a comment covers (c0.text), and read names note paragraphs", () => {
  const file = copy(untracked, "covered.docx");
  const commented = path.join(tmp, "covered-c.docx");
  const out = path.join(tmp, "covered-out.docx");
  const add = run("add", file, "-p", "p8", "--anchor", "Last", "--content", "Check", "-a", "Jubarte", "-o", commented);
  assert.equal(add.code, 0, add.err);
  const edit = run("edit", commented, "-p", "c0.text", "--content", "Final", "-a", "Jubarte", "-o", out);
  assert.equal(edit.code, 0, edit.err);
  const view = run("read", out);
  assert.equal(view.code, 0, view.err);
  // edit writes the deletion before the insertion; read pairs only insertion-first, so two changes.
  assert.match(view.out, /\{==\{~~Last~>Final~~\}\{r\d+\+\d+ @J\}==\}/);
  assert.match(view.out, /<!-- footnotes\.p0 -->\n\[\^1\]: The note text\./);
  const outside = run("edit", commented, "-p", "c0.text", "--anchor", "paragraph", "--content", "x", "-o", path.join(tmp, "covered-no.docx"));
  assert.equal(outside.code, 3, outside.err);
  assert.match(outside.err + outside.out, /not in the text c0 covers/);
});

test("edit and add --plan record into one plan that --consume deletes once applied", () => {
  const paras = JSON.parse(run("read", untracked, "--json").out).paragraphs.filter((p) => p.text.length > 3);
  const [para, other, third] = paras;
  const word = para.text.split(" ")[0];
  const file = copy(untracked, "record.docx");
  const plan = path.join(tmp, "record-plan.json");
  const before = fs.readFileSync(file);
  const first = run("edit", file, "--plan", plan, "-p", `p${para.index}`, "--anchor", word, "--content", "Changed");
  assert.equal(first.code, 0, first.err);
  assert.equal(first.out.trim(), `recorded 1 operation in ${plan}: 1 operation, 0 revisions to accept, 0 to reject; apply with: jubarte edit ${file} --plan ${plan}`);
  const second = run("add", file, "--plan", plan, "-p", `p${other.index}`, "--comment", "--content", "Why?");
  assert.equal(second.code, 0, second.err);
  assert.match(second.out, /^recorded 1 operation in .*: 2 operations/);
  assert.deepEqual(fs.readFileSync(file), before, "recording edits nothing");
  const held = fs.readFileSync(plan, "utf8");
  assert.ok(!held.includes("null"), held);
  assert.equal(JSON.parse(held).operations.length, 2);
  // What applies later is refused while recording; what is recorded is refused while applying.
  const later = run("edit", file, "--plan", plan, "-p", "p1", "--content", "x", "--out-dir", "d");
  assert.equal(later.code, 2);
  assert.match(later.err, /--out-dir takes effect when the plan is applied/);
  const author = run("edit", file, "--plan", plan, "--author", "Bob");
  assert.equal(author.code, 2);
  assert.match(author.err, /--author belongs to recorded operations/);
  const bob = run("edit", file, "--plan", plan, "-p", "p1", "--content", "x", "--author", "Bob");
  assert.equal(bob.code, 2);
  assert.match(bob.err, /the plan's author is "Jubarte"/);
  const missing = run("edit", file, "--plan", plan, "-p", `p${para.index}`, "--anchor", "nowhere at all", "--content", "x");
  assert.equal(missing.code, 3, missing.out);
  assert.match(missing.out, /"status":"failed"/);
  const dry = run("edit", file, "--plan", plan, "--dry-run", "-p", `p${third.index}`, "--content", "Checked only.");
  assert.equal(dry.code, 0, dry.err);
  assert.match(dry.out, /^would record 1 operation in .*: 3 operations/);
  assert.equal(fs.readFileSync(plan, "utf8"), held, "a refused or dry recording keeps the plan");
  const render = run("edit", file, "--plan", plan, "-p", "p1", "--content", "x", "--revisions", "word");
  assert.equal(render.code, 2);
  assert.match(render.err, /--revisions takes effect when the plan is applied/);
  const preview = run("edit", file, "--plan", plan, "--consume", "--dry-run");
  assert.equal(preview.code, 0, preview.err);
  assert.ok(fs.existsSync(plan), "a dry run consumes nothing");
  const link = path.join(tmp, "record-link.json");
  fs.symlinkSync(plan, link);
  const linked = run("edit", file, "--plan", link, "--consume");
  assert.equal(linked.code, 2);
  assert.match(linked.err, /a symbolic link; pass the file it names/);
  assert.equal(fs.readFileSync(plan, "utf8"), held);
  assert.deepEqual(fs.readdirSync(tmp).filter((n) => n.endsWith(".partial")), []);
  // A link to a plan not written yet creates the file it names.
  const next = path.join(tmp, "record-next.json");
  fs.symlinkSync("record-next-target.json", next);
  assert.equal(run("edit", file, "--plan", next, "-p", "p1", "--content", "x").code, 0);
  assert.ok(fs.lstatSync(next).isSymbolicLink() && fs.existsSync(path.join(tmp, "record-next-target.json")));
  // A plan that is not UTF-8 is an I/O error, as the binary reads it (pi review 06c parity F1).
  const bad = path.join(tmp, "record-bad.json");
  fs.writeFileSync(bad, Buffer.from([0xff, 0xfe, 0x7b, 0x7d]));
  for (const args of [["-p", "p1", "--content", "x"], []]) {
    const r = run("edit", file, "--plan", bad, ...args);
    assert.equal(r.code, 1, r.err);
    assert.match(r.err, /stream did not contain valid UTF-8/);
  }
  const dir = path.join(tmp, "record-out");
  const applied = run("edit", file, "--plan", plan, "--consume", "--out-dir", dir);
  assert.equal(applied.code, 0, applied.err);
  assert.ok(!fs.existsSync(plan), "--consume deletes the applied plan");
  const view = run("read", path.join(dir, "redline.docx")).out;
  assert.ok(view.includes(`{~~${word}~>Changed~~}`) && view.includes("Why?"), view);
});

test("--consume closes the plan descriptor when applyEdit throws (integration)", () => {
  const plan = path.join(tmp, "consume-throws.json");
  const out = path.join(tmp, "consume-throws.docx");
  const recorded = run("edit", untracked, "--plan", plan, "-p", "p1", "--content", "Changed.");
  assert.equal(recorded.code, 0, recorded.err);
  fs.writeFileSync(out, "existing output");
  const result = runWithPlanHook(plan, (fs, mock, assert, plan) => {
    let planFd;
    const open = fs.openSync;
    mock.method(fs, "openSync", (name, ...args) => {
      const fd = open(name, ...args);
      if (name === plan && planFd === undefined) planFd = fd;
      return fd;
    });
    return () => {
      assert.equal(process.exitCode, 2, "applyEdit must throw the existing-output usage error");
      assert.notEqual(planFd, undefined, "the plan was opened");
      assert.throws(() => fs.fstatSync(planFd), { code: "EBADF" }, "the plan descriptor must be closed on throw");
    };
  }, ["edit", untracked, "--plan", plan, "--consume", "-o", out]);
  assert.equal(result.code, 0, result.err);
  assert.match(result.err, /already exists \(use --force to overwrite\)/);
  assert.ok(fs.existsSync(plan), "a failed application keeps its plan");
  assert.equal(fs.readFileSync(out, "utf8"), "existing output");
});

test("--consume refuses an identical-bytes replacement between open and read (integration)", () => {
  const plan = path.join(tmp, "consume-open-swap.json");
  const out = path.join(tmp, "consume-open-swap.docx");
  assert.equal(run("edit", untracked, "--plan", plan, "-p", "p1", "--content", "Changed.").code, 0);
  const held = fs.readFileSync(plan);
  const result = runWithPlanHook(plan, (fs, mock, assert, plan) => {
    const open = fs.openSync;
    let swapped = false, planFd;
    const original = fs.readFileSync(plan);
    mock.method(fs, "openSync", (name, ...args) => {
      const fd = open(name, ...args);
      if (name === plan && !swapped) {
        swapped = true;
        planFd = fd;
        fs.writeFileSync(`${plan}.replacement`, original);
        fs.renameSync(`${plan}.replacement`, plan);
        assert.notEqual(fs.fstatSync(fd, { bigint: true }).ino, fs.lstatSync(plan, { bigint: true }).ino);
      }
      return fd;
    });
    return () => {
      assert.ok(swapped);
      assert.equal(process.exitCode, 1, "a changed plan identity must be refused before applyEdit");
      assert.throws(() => fs.fstatSync(planFd), { code: "EBADF" });
    };
  }, ["edit", untracked, "--plan", plan, "--consume", "-o", out, "-q"]);
  assert.equal(result.code, 0, result.err);
  assert.match(result.err, /reading .*: .*changed while it was being read/);
  assert.deepEqual(fs.readFileSync(plan), held);
  assert.ok(!fs.existsSync(out));
});

for (const change of ["rename", "overwrite", "append"]) {
  test(`--consume keeps a plan changed by ${change} after applying (integration)`, () => {
    const plan = path.join(tmp, `consume-${change}.json`);
    const out = path.join(tmp, `consume-${change}.docx`);
    assert.equal(run("edit", untracked, "--plan", plan, "-p", "p1", "--content", "Changed.").code, 0);
    const held = fs.readFileSync(plan);
    // The CLI writes synchronously, through a staged file renamed over the
    // output. Interpose only at that rename to reproduce another actor
    // replacing the plan, including an identical-bytes rename.
    const hook = (fs, mock, assert, plan, change) => {
      const write = fs.writeFileSync;
      const rename = fs.renameSync;
      let changed = false, planFd;
      const open = fs.openSync;
      mock.method(fs, "openSync", (name, ...args) => {
        const fd = open(name, ...args);
        if (name === plan && planFd === undefined) planFd = fd;
        return fd;
      });
      mock.method(fs, "renameSync", (from, to, ...rest) => {
        const result = rename(from, to, ...rest);
        if (!changed && String(to).endsWith(".docx")) {
          changed = true;
          assert.doesNotThrow(() => fs.fstatSync(planFd), "keep the descriptor open until deletion");
          const original = fs.readFileSync(plan);
          const before = fs.lstatSync(plan, { bigint: true });
          if (change === "rename") {
            write(`${plan}.replacement`, original);
            fs.renameSync(`${plan}.replacement`, plan);
            assert.notEqual(fs.lstatSync(plan, { bigint: true }).ino, before.ino);
          } else if (change === "append") {
            fs.appendFileSync(plan, " ");
          } else {
            write(plan, Buffer.alloc(original.length, 0x20));
            assert.equal(fs.lstatSync(plan, { bigint: true }).ino, before.ino);
          }
        }
        return result;
      });
      return () => {
        assert.ok(changed, "the edit wrote its output");
        assert.equal(process.exitCode, 1, "a changed plan must be kept");
        assert.throws(() => fs.fstatSync(planFd), { code: "EBADF" });
      };
    };
    const result = runWithPlanHook(plan, hook,
      ["edit", untracked, "--plan", plan, "--consume", "-o", out, "-q"], change);
    assert.equal(result.code, 0, result.err);
    assert.match(result.err, /it no longer holds the plan \(an output was written over it\?\)/);
    assert.ok(fs.existsSync(out), "the edit committed before the refusal to delete");
    const current = fs.readFileSync(plan);
    if (change === "rename") assert.deepEqual(current, held, "identical bytes on a new inode are preserved");
    else assert.ok(!current.equals(held), "changed content is preserved");
  });
}

test("--consume holds its descriptor through unlink and rereads from offset zero (integration)", () => {
  const plan = path.join(tmp, "consume-unlink.json");
  const out = path.join(tmp, "consume-unlink.docx");
  assert.equal(run("edit", untracked, "--plan", plan, "-p", "p1", "--content", "Changed.").code, 0);
  const result = runWithPlanHook(plan, (fs, mock, assert, plan) => {
    const open = fs.openSync, read = fs.readSync, unlink = fs.unlinkSync;
    let planFd, positional = false, deleted = false;
    mock.method(fs, "openSync", (name, ...args) => {
      const fd = open(name, ...args);
      if (name === plan && planFd === undefined) planFd = fd;
      return fd;
    });
    mock.method(fs, "readSync", (fd, buffer, offset, length, position) => {
      if (fd === planFd && position === 0) positional = true;
      return read(fd, buffer, offset, length, position);
    });
    mock.method(fs, "unlinkSync", (name) => {
      if (name === plan) {
        assert.doesNotThrow(() => fs.fstatSync(planFd), "the plan fd must stay open until unlink");
        assert.ok(positional, "the consumed fd must be reread from position zero");
        deleted = true;
      }
      return unlink(name);
    });
    return () => {
      assert.equal(process.exitCode, 0);
      assert.ok(deleted, "the unchanged plan must be consumed");
      assert.throws(() => fs.fstatSync(planFd), { code: "EBADF" });
    };
  }, ["edit", untracked, "--plan", plan, "--consume", "-o", out, "-q"]);
  assert.equal(result.code, 0, result.err);
  assert.ok(!fs.existsSync(plan));
});

for (const identity of ["zero-inos", "different-dev"]) {
  test(`--consume identity handles ${identity} (integration)`, () => {
    const plan = path.join(tmp, `consume-${identity}.json`);
    const out = path.join(tmp, `consume-${identity}.docx`);
    assert.equal(run("edit", untracked, "--plan", plan, "-p", "p1", "--content", "Changed.").code, 0);
    const result = runWithPlanHook(plan, (fs, mock, assert, plan, identity) => {
      const open = fs.openSync, fstat = fs.fstatSync, lstat = fs.lstatSync;
      let planFd, checks = 0;
      mock.method(fs, "openSync", (name, ...args) => {
        const fd = open(name, ...args);
        if (name === plan && planFd === undefined) planFd = fd;
        return fd;
      });
      mock.method(fs, "fstatSync", (fd, options) => {
        const stat = fstat(fd, options);
        if (fd === planFd && options?.bigint) {
          stat.ino = identity === "zero-inos" ? 0n : 9007199254740993n;
          stat.dev = 9007199254740992n;
        }
        return stat;
      });
      mock.method(fs, "lstatSync", (name, options) => {
        const stat = lstat(name, options);
        if (name === plan) {
          assert.equal(options?.bigint, true);
          checks++;
          stat.ino = identity === "zero-inos" ? 0n : 9007199254740993n;
          stat.dev = 9007199254740993n;
        }
        return stat;
      });
      return () => {
        assert.equal(process.exitCode, identity === "zero-inos" ? 0 : 1);
        assert.equal(checks, identity === "zero-inos" ? 2 : 1, "identity is checked on open and before delete");
        assert.throws(() => fs.fstatSync(planFd), { code: "EBADF" });
      };
    }, ["edit", untracked, "--plan", plan, "--consume", "-o", out, "-q"], identity);
    assert.equal(result.code, 0, result.err);
    assert.equal(fs.existsSync(plan), identity !== "zero-inos");
    assert.equal(fs.existsSync(out), identity === "zero-inos");
  });
}

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
  for (const args of [["convert", "missing.docx", "--timeout", "1"], ["compare", "a", "b", "--mode", "powertools"]]) {
    const result = run(...args);
    assert.equal(result.code, 2, result.err);
    assert.doesNotMatch(result.err, /reading/);
  }
});

test("Markdown paragraph/critic output and shorthand comparison (integration)", () => {
  const a = path.join(tmp, "short-a.md"), b = path.join(tmp, "short-b.md");
  fs.writeFileSync(a, "Due in 30 days.\n");
  fs.writeFileSync(b, "Due in 45 days.\n");
  const patch = run("diff", a, b, "--author", "Legal", "--datetime", "2026-09-30T14:05:00Z");
  assert.equal(patch.code, 0, patch.err);
  assert.match(patch.out, /\[-30-\]\{\+45\+\}/);
  assert.equal(run("diff", a, b, "--format", "critic").out, "Due in {~~30~>45~~} days.\n");
  // The shorthand prints the redline's agent view and writes nothing; -o writes it.
  const word = path.join(tmp, "short-a_v_short-b.docx");
  const shown = run(a, b, "-d", "1970-01-01T00:00:00Z");
  assert.equal(shown.code, 0, shown.err);
  assert.ok(shown.out.startsWith(`---\nsource: ${word} (not written; -o keeps it)\n`), shown.out);
  // Word comparison keeps insertion before deletion, as the native view does.
  const replacement = (handle) => new RegExp(`\\{\\+\\+45\\+\\+\\}\\{r\\d+\\+\\d+ @${handle}\\}\\{--30--\\}\\{r\\d+\\+\\d+ @${handle}\\}`);
  assert.match(shown.out, replacement("J"));
  assert.ok(!fs.existsSync(word));
  // The view is the output, so --quiet does not hide it (pi review av5 F6);
  // read options apply to it.
  assert.match(run(a, b, "--quiet", "-d", "1970-01-01T00:00:00Z").out, replacement("J"));
  const acceptedView = run(a, b, "--track-changes", "accept", "-d", "1970-01-01T00:00:00Z");
  assert.equal(acceptedView.code, 0, acceptedView.err);
  assert.ok(!acceptedView.out.includes("{~~") && acceptedView.out.includes("<!-- p0 rev r"), acceptedView.out);
  const compared = run(a, b, "-d", "1970-01-01T00:00:00Z", "-o", word, "--quiet");
  assert.equal(compared.code, 0, compared.err);
  assert.equal(compared.out, "");
  assert.equal(fs.readFileSync(word).subarray(0, 2).toString(), "PK");
  // --changed --by keeps the author's block, by handle or by name (pi
  // review av3 F8: only the (none) path was tested).
  const signed = path.join(tmp, "signed.docx");
  assert.equal(run(a, b, "-d", "1970-01-01T00:00:00Z", "-o", signed, "--author", "Ann Counsel", "--quiet").code, 0);
  for (const by of ["AC", "Ann Counsel"]) {
    const mine = run("read", signed, "--changed", "--by", by, "--no-page-markers");
    assert.equal(mine.code, 0, mine.err);
    assert.ok(mine.out.includes("\nrange: changed by @AC (p0) of p0"), mine.out);
    assert.match(mine.out, replacement("AC"));
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
    // The file holds the patch; stdout holds nothing.
    assert.match(fs.readFileSync(out, "utf8"), /\[-30-\]\{\+45\+\}/);
    assert.equal(result.out, "");
    assert.ok(result.err.startsWith(`wrote ${out} (md, `), result.err);
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

test("unknown output suffix writes the patch for Word inputs (integration)", () => {
  const out = path.join(tmp, "word-fallback.unknown");
  const result = run("diff", path.join(pair, "base.docx"), path.join(pair, "next.docx"), "-o", out);
  assert.equal(result.code, 0, result.err);
  assert.match(fs.readFileSync(out, "utf8"), /^--- a\//);
  assert.equal(result.out, "");
});


test("explicit Markdown input and UTF-8 BOM share native text behavior", () => {
  const a = path.join(tmp, "declared-old.docx"), b = path.join(tmp, "declared-new.docx");
  fs.writeFileSync(a, "\ufeffDue 30 days.\n");
  fs.writeFileSync(b, "Due 45 days.\n");
  const result = run("diff", a, b, "--from", "md", "--format", "word", "--full-lines");
  assert.equal(result.code, 0, result.err);
  assert.equal(result.out, "Due {~~30~>45~~} days.\n");
});


test("a redline's PDF forwards page options; diff writes none (integration)", () => {
  const wasm = createRequire(bin)("jubarte-wasm");
  const long = Array.from({ length: 120 }, (_, i) => `Paragraph ${i}.`).join("\n\n");
  const a = path.join(tmp, "page-options-old.md"), b = path.join(tmp, "page-options-new.md");
  fs.writeFileSync(a, long);
  fs.writeFileSync(b, long.replace("Paragraph 0.", "Paragraph zero."));
  const refused = run("diff", a, b, "-o", path.join(tmp, "diff-refused.pdf"));
  assert.equal(refused.code, 2, refused.err);
  assert.match(refused.err, /`compare OLD NEW -o .*diff-refused\.pdf`/);
  const redline = path.join(tmp, "page-options-redline.docx");
  assert.equal(run("compare", a, b, "-d", "1970-01-01T00:00:00Z", "-o", redline).code, 0);
  // Without -d the revisions are dated now (D17): the shared parser fills it.
  const stamp = () => new Date().toISOString().replace(/\.\d{3}Z$/, "Z");
  const before = stamp();
  const dated = path.join(tmp, "page-options-now.docx");
  assert.equal(run("compare", a, b, "-o", dated).code, 0);
  const after = stamp();
  const dates = records(run("changes", dated, "--json").out).map((change) => change.date);
  assert.ok(dates.length > 0 && dates.every((d) => before <= d && d <= after), `${before} ${dates} ${after}`);
  const pinned = records(run("changes", redline, "--json").out).map((change) => change.date);
  assert.deepEqual([...new Set(pinned)], ["1970-01-01T00:00:00Z"]);
  const all = path.join(tmp, "diff-all.pdf"), kept = path.join(tmp, "diff-kept.pdf");
  for (const [output, flags] of [[all, []], [kept, ["--changed-only"]]]) {
    const result = run("convert", redline, "-o", output, ...flags);
    assert.equal(result.code, 0, result.err);
  }
  assert.ok(wasm.pdfPageCount(fs.readFileSync(all)) > 1);
  assert.equal(wasm.pdfPageCount(fs.readFileSync(kept)), 1);
  const end = path.join(tmp, "diff-end.pdf");
  const moved = run("convert", tracked, "-o", end, "--move-comments");
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
  assert.equal(docx.out, "");
  assert.match(docx.err, /^wrote .*reference-draft\.docx \(docx, \d+ bytes\)\n$/);
  assert.match(run("text", path.join(tmp, "reference-draft.docx")).out, /A clause\./);
  const pdf = run("convert", source, "--pdf");
  assert.equal(pdf.code, 0, pdf.err);
  assert.equal(fs.readFileSync(path.join(tmp, "reference-draft.pdf")).subarray(0, 5).toString(), "%PDF-");
});

test("comparison refuses Markdown output of Word inputs and sniffs unknown input suffixes (integration)", () => {
  // As natively, a .md output is CriticMarkup of two Markdown documents.
  const markdown = path.join(tmp, "compare-output.md");
  // A .md of Word inputs is the redline's view, headed by the Word file
  // compare did not write.
  const result = run("compare", path.join(pair, "base.docx"), path.join(pair, "next.docx"), "-o", markdown, "-d", "1970-01-01T00:00:00Z");
  assert.equal(result.code, 0, result.err);
  assert.match(fs.readFileSync(markdown, "utf8"), new RegExp(`^---\nsource: ${path.join(pair, "base_v_next.docx").replaceAll(".", "\\.")} \\(not written; -o keeps it\\)\n`));
  const a = copy(path.join(pair, "base.docx"), "sniff-a.bin");
  const b = copy(path.join(pair, "next.docx"), "sniff-b.bin");
  const inferred = run("diff", a, b);
  assert.equal(inferred.code, 0, inferred.err);
  assert.match(inferred.out, /^--- a\//);
  assert.ok(!fs.existsSync(path.join(tmp, "sniff-a_v_sniff-b.docx")));
});

test("edit previews write nothing and quiet application still writes its report (integration)", () => {
  const plan = path.join(tmp, "coverage-edit-plan.json");
  const paragraph = JSON.parse(run("read", untracked, "--json").out).paragraphs.find((p) => p.text.length > 3);
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
  // The plan runs first (a refused plan reports whatever the outputs), so the
  // directory check needs a plan the document takes.
  const own = run("edit", untracked, "--plan", plan, "--out-dir", path.dirname(untracked), "--force");
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
    ["--critic"], ["--reference-doc", "r.docx"], ["--resource-path", "."],
    ["-o", "r.png"], ["-o", "r.pdf", "--pages", "1"],
  ];
  for (const flags of comparison) {
    const result = run("compare", "missing-a.docx", "missing-b.docx", ...flags);
    assert.equal(result.code, 2, `${flags}: ${result.err}`);
    assert.match(result.err, /not supported/);
    assert.doesNotMatch(result.err, /reading/);
  }
  for (const flags of [["--from", "docx"], ["--resource-path", "."], ["--fail-on-substitution"], ["--dpi", "120"], ["--pages", "1"], ["--report", "report.json"], ["--font-report", "fonts.json"]]) {
    const result = run("convert", "missing.docx", ...flags);
    assert.equal(result.code, 2, `${flags}: ${result.err}`);
    assert.doesNotMatch(result.err, /reading/);
  }
  // Markdown to Markdown has no engine call here; it says where it runs.
  const markdown = run("convert", "missing.md", "-o", "output.md");
  assert.equal(markdown.code, 2, markdown.err);
  assert.match(markdown.err, /uvx jubarte-redlines convert/);
  for (const flags of [["--png"]]) {
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
  // A Word file under a name that says nothing is Word, and Word to Word
  // needs something to do, as the binary says; nothing is written.
  const zip = copy(untracked, "SNIFF-WORD"), out = path.join(tmp, "sniff-word.docx");
  const refused = run("convert", zip, "-o", out);
  assert.equal(refused.code, 1, refused.err);
  assert.match(refused.err, /SNIFF-WORD is already Word: give --track-changes accept or reject, --update-fields, or another --to\n$/);
  assert.ok(!fs.existsSync(out));
});

test("compare reports the bytes a Markdown redline takes (integration)", () => {
  // Codex on #381: the count is UTF-8 bytes on disk, not UTF-16 code units.
  const a = path.join(tmp, "bytes-a.md"), b = path.join(tmp, "bytes-b.md"), out = path.join(tmp, "bytes-redline.md");
  fs.writeFileSync(a, "Prazo de 30 días — ok.\n");
  fs.writeFileSync(b, "Prazo de 45 días — ok.\n");
  const result = run("compare", a, b, "-d", "1970-01-01T00:00:00Z", "-o", out);
  assert.equal(result.code, 0, result.err);
  const fd = fs.openSync(out, "r");
  const size = fs.fstatSync(fd).size;
  assert.ok(size > fs.readFileSync(fd, "utf8").length);
  fs.closeSync(fd);
  assert.equal(result.out, "");
  assert.equal(result.err, `wrote ${out} (md, ${size} bytes)\n`);
});

test("diff, compare and convert follow the native input and output contract (integration)", () => {
  const a = path.join(tmp, "contract-a.md"), b = path.join(tmp, "contract-b.md");
  fs.writeFileSync(a, "Due in 30 days.\n");
  fs.writeFileSync(b, "Due in 45 days.\n");
  const word = path.join(tmp, "contract-a.docx");
  assert.equal(run("convert", a, "--to", "docx", "-o", word).code, 0);

  // diff -o FILE.md of a Word side writes the patch (compare's .md is the
  // redline's view); diff writes no Word.
  const written = path.join(tmp, "contract-d.md");
  const patched = run("diff", word, b, "-o", written);
  assert.equal(patched.code, 0, patched.err);
  assert.match(fs.readFileSync(written, "utf8"), /\{\+45\+\}\[-30-\]/);
  for (const [args, code, message] of [
    [["diff", word, b, "-o", path.join(tmp, "contract-patch.docx")], 2, /`compare OLD NEW -o /],
  ]) {
    const result = run(...args);
    assert.equal(result.code, code, result.out);
    assert.match(result.err, message);
    assert.ok(!fs.existsSync(args.at(-1)));
  }

  // --format critic needs two Markdown documents.
  const critic = run("diff", word, b, "--format", "critic");
  assert.equal(critic.code, 1, critic.err);
  assert.equal(critic.out, "");
  assert.match(critic.err, /--format critic needs both documents in Markdown/);

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
  const first = run("read", untracked, "--json");
  const para = JSON.parse(first.out).paragraphs.find((p) => p.text.length > 3);
  const plan = path.join(tmp, "plan-broken-view.json");
  fs.writeFileSync(plan, JSON.stringify({ schema_version: 1, author: "Claude", date: "2026-09-25T12:00:00Z", operations: [{ kind: "replace", paragraph: { index: para.index }, find: para.text.split(" ")[0], replacement: "Changed" }] }));
  const dir = path.join(tmp, "review-broken-view");
  const r = spawnSync(process.execPath, ["--require", preload, bin, "edit", untracked, "--plan", plan, "--out-dir", dir], { encoding: "utf8", cwd: tmp });
  assert.equal(r.status, 0, r.stderr);
  assert.ok(fs.existsSync(path.join(dir, "redline.docx")));
  assert.match(r.stderr, /warning: the changed blocks cannot be shown: read back failed/);
});

test("pr392: editing mode preserves its complete bundle after a view failure", (t) => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "jubarte-view-failure-"));
  t.after(() => fs.rmSync(dir, { recursive: true, force: true }));
  const preload = path.join(dir, "broken-view.cjs");
  const called = path.join(dir, "called.json");
  fs.writeFileSync(preload,
    `const fs = require("node:fs");\n` +
    `const wasm = require(${JSON.stringify(path.join(cliDir, "node_modules", "jubarte-wasm"))});\n` +
    `wasm.changedView = (_bytes, author, accepted, source) => {\n` +
    `  fs.writeFileSync(${JSON.stringify(called)}, JSON.stringify({author, accepted, source}));\n` +
    `  throw "view unavailable";\n` +
    `};\n`);
  // Empty plans are valid and isolate the post-write view from edit semantics.
  const plan = path.join(dir, "plan.json");
  fs.writeFileSync(plan, JSON.stringify({ schema_version: 1, author: "Ann Counsel", date: "2026-10-01T09:00:00Z", operations: [] }));
  for (const quiet of [false, true]) {
    fs.rmSync(called, { force: true });
    const out = path.join(dir, quiet ? "quiet" : "shown");
    const r = spawnSync(process.execPath, ["--require", preload, bin, "edit", untracked, "--plan", plan, "--editing-mode", "--out-dir", out, ...(quiet ? ["--quiet"] : [])], { encoding: "utf8", timeout: 10000 });
    assert.ifError(r.error);
    assert.equal(r.status, 0, r.stderr);
    assert.deepEqual(fs.readdirSync(out).sort(), ["clean.docx", "report.jsonl"]);
    assert.equal(fs.readFileSync(path.join(out, "clean.docx")).subarray(0, 2).toString(), "PK");
    const report = fs.readFileSync(path.join(out, "report.jsonl"), "utf8").trim().split("\n").map(JSON.parse);
    const saved = report.find((row) => row.ev === "save");
    assert.deepEqual(saved.outputs.map((output) => output.f), ["clean.docx"]);
    assert.equal(saved.outputs[0].bytes, fs.statSync(path.join(out, "clean.docx")).size);
    assert.equal(report.at(-1).ev, "summary");
    assert.equal(report.at(-1).status, "ok");
    if (quiet) {
      assert.equal(r.stdout, "");
      assert.equal(r.stderr, "");
      assert.ok(!fs.existsSync(called), "quiet mode must not attempt the view");
    } else {
      assert.deepEqual(JSON.parse(fs.readFileSync(called, "utf8")), { author: "Ann Counsel", accepted: true, source: path.join(out, "clean.docx") });
      assert.match(r.stderr, /warning: the changed blocks cannot be shown: view unavailable/);
      assert.match(r.stdout, /"ev":"summary"/);
      assert.doesNotMatch(r.stdout, /---\nsource:/);
    }
  }
});

test("agent shorthand reads an in-memory redline without touching an existing default output", () => {
  const a = path.join(tmp, "sentinel-old.md"), b = path.join(tmp, "sentinel-new.md");
  fs.writeFileSync(a, "Unchanged intro.\n\nDue in 30 days.\n");
  fs.writeFileSync(b, "Unchanged intro.\n\nDue in 45 days.\n");
  const fallback = path.join(tmp, "sentinel-old_v_sentinel-new.docx");
  const sentinel = Buffer.from("existing output must survive");
  fs.writeFileSync(fallback, sentinel);
  for (const [mode, kept, removed] of [["accept", "45", "30"], ["reject", "30", "45"]]) {
    const result = run(a, b, "--track-changes", mode, "--changed", "--by", "Legal", "--author", "Legal", "--no-page-markers", "-d", "1970-01-01T00:00:00Z");
    assert.equal(result.code, 0, result.err);
    assert.ok(result.out.startsWith(`---\nsource: ${fallback} (not written; -o keeps it)\n`));
    assert.ok(result.out.includes(`Due in ${kept} days.`), result.out);
    assert.ok(!result.out.includes(`Due in ${removed} days.`), result.out);
    assert.doesNotMatch(result.out, /Unchanged intro|<!-- page /);
    assert.deepEqual(fs.readFileSync(fallback), sentinel);
  }
  // compare prints the same view; only -o writes, and never over a file.
  const explicit = run("compare", a, b, "-d", "1970-01-01T00:00:00Z");
  assert.equal(explicit.code, 0, explicit.err);
  assert.ok(explicit.out.startsWith(`---\nsource: ${fallback} (not written; -o keeps it)\n`));
  const written = run("compare", a, b, "-o", fallback);
  assert.equal(written.code, 2);
  assert.match(written.err, /OUTPUT_EXISTS: output '.*' already exists \(use --force to overwrite\)/);
  assert.deepEqual(fs.readFileSync(fallback), sentinel);
  // Two Markdown documents written to a .md are CriticMarkup, which view
  // options cannot shape; the host reads that from the bytes, as natively.
  const plainA = path.join(tmp, "sentinel-old"), plainB = path.join(tmp, "sentinel-new.json");
  fs.copyFileSync(a, plainA);
  fs.copyFileSync(b, plainB);
  const critic = path.join(tmp, "sentinel-critic.md");
  const made = run("compare", plainA, plainB, "-d", "1970-01-01T00:00:00Z", "-o", critic);
  assert.equal(made.code, 0, made.err);
  assert.match(fs.readFileSync(critic, "utf8"), /\{~~30~>45~~\}/);
  const refused = run("compare", plainA, plainB, "-o", path.join(tmp, "sentinel-h.md"), "--head", "1");
  assert.equal(refused.code, 2, refused.err);
  assert.match(refused.err, /two Markdown documents write CriticMarkup/);
});

test("agent editing mode rejects keep before creating output even in dry run", () => {
  for (const task of ["edit", "add"]) {
    for (const dryRun of task === "edit" ? [false, true] : [false]) {
      const out = path.join(tmp, `keep-${task}-${dryRun}`);
      const result = run(task, tracked, "-p", "p0", "--content", "Replacement", "--existing-revisions", "keep", "--editing-mode", "--out-dir", out, ...(dryRun ? ["--dry-run"] : []));
      assert.equal(result.code, 2, result.err);
      assert.match(result.err, /--existing-revisions accept or reject/);
      assert.equal(result.out, "");
      assert.equal(fs.existsSync(out), false);
    }
  }
});

test("convert signs what names no one by the flags, the front matter, the reference, else Redline and now", () => {
  const reference = path.join(repo, "tests/fixtures/md-roundtrip/review/ref-stamp.docx");
  const stamped = (markdown, ...extra) => {
    const md = path.join(tmp, "stamp.md");
    const out = path.join(tmp, "stamp.docx");
    fs.writeFileSync(md, markdown);
    const r = run("convert", md, "-o", out, "--force", ...extra);
    assert.equal(r.code, 0, r.err);
    const changes = run("changes", out, "--json");
    assert.equal(changes.code, 0, changes.err);
    return records(changes.out)[0];
  };
  const front = "---\nauthor: Fran Front\nlast_modified: 2025-01-02T03:04:05Z\n---\n\nSome {++x++} here.\n";
  let change = stamped(front, "--reference-doc", reference);
  assert.deepEqual([change.author, change.date], ["Fran Front", "2025-01-02T03:04:05Z"]);
  change = stamped("Some {++x++} here.\n", "--reference-doc", reference);
  assert.deepEqual([change.author, change.date], ["Cleo Core", "2025-03-04T05:06:07Z"]);
  change = stamped(front, "-a", "Cli Name", "-d", "2024-02-03T04:05:06Z");
  assert.deepEqual([change.author, change.date], ["Cli Name", "2024-02-03T04:05:06Z"]);
  const now = () => new Date().toISOString().replace(/\.\d{3}Z$/, "Z");
  const before = now();
  change = stamped("Some {++x++} here.\n");
  const after = now();
  assert.equal(change.author, "Redline");
  assert.ok(before <= change.date && change.date <= after, `${before} <= ${change.date} <= ${after}`);
});

// bebbbfff pinned these as refusals; the npm CLI now writes Markdown, PDF
// and JSON, so only PNG is refused. Either way a failed run leaves the
// existing output alone, --force or not.
for (const command of ["accept", "reject"]) {
  test(`writer refuses only PNG on ${command}, and a failed run keeps the output`, () => {
    for (const extension of ["md", "markdown", "PDF", "png", "JSON"]) {
      const output = path.join(tmp, `unsupported-${command}.${extension}`);
      fs.writeFileSync(output, "preserve output");
      const result = run(command, "missing.docx", "-o", output, "--force", "-q");
      assert.equal(result.out, "");
      if (extension === "png") {
        assert.equal(result.code, 2, `${extension}: ${result.err}`);
        assert.ok(result.err.includes("-o FILE.png is not supported by the npm CLI"), result.err);
        assert.ok(!fs.existsSync(output.replace(/\.png$/, "-page-01.png")));
      } else {
        assert.equal(result.code, 1, `${extension}: ${result.err}`);
        assert.match(result.err, /reading missing\.docx/);
      }
      assert.equal(fs.readFileSync(output, "utf8"), "preserve output");
    }
  });
}

test("read, its alias and the shorthand create no output when the source fails", () => {
  for (const prefix of [["read", "missing.docx"], ["text", "missing.docx"], ["missing.docx"]]) {
    for (const extension of ["md", "JSON", "unknown"]) {
      const output = path.join(tmp, `unsupported-read.${extension}`);
      const result = run(...prefix, "-o", output, "--force", "-q");
      assert.equal(result.code, 1, result.err);
      assert.equal(result.out, "");
      assert.match(result.err, /reading missing\.docx/);
      assert.ok(!fs.existsSync(output));
    }
  }
});

test("convert JSON, by format or extension, keeps the output when the source fails", () => {
  const output = path.join(tmp, "unsupported-convert.unknown");
  fs.writeFileSync(output, "preserve output");
  for (const flags of [["--to", "json"], ["-o", "unsupported-convert.JSON"], ["--to", "json", "-o", output]]) {
    const result = run("convert", "missing.docx", ...flags, "--force", "-q");
    assert.equal(result.code, 1, result.err);
    assert.equal(result.out, "");
    assert.match(result.err, /reading missing\.docx/);
    assert.equal(fs.readFileSync(output, "utf8"), "preserve output");
    assert.ok(!fs.existsSync(path.join(tmp, "missing.json")));
    assert.ok(!fs.existsSync(path.join(tmp, "unsupported-convert.JSON")));
  }
});

// The checked-in engine is rebuilt only at release (ENGINE_COMMIT.txt), so a
// CLI feature that needs a newer export waits for the next release to run.
const resolutionTextShipped = typeof createRequire(bin)("jubarte-wasm").resolutionText === "function";
test("edit -p rN --accept/--reject prints the resolved changes, as the binary does", {
  skip: !resolutionTextShipped && "the checked-in engine predates resolutionText; it ships with the next release",
}, () => {
  const received = path.join(repo, "tests/fixtures/agent-view/received.docx");
  const src = copy(received, "resolve-received.docx");
  const out = path.join(tmp, "resolve-out.docx");
  const single = run("edit", src, "-p", "r0", "--accept", "-o", out, "--datetime", "2026-10-11T00:00:00Z");
  assert.equal(single.code, 0, single.err);
  assert.equal(single.out, 'accepted r0 in p3: "{++quarterly++}" > "quarterly"\n');
  assert.match(single.err, new RegExp(`^wrote ${out.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")} \\(docx, `));
  const pair = run("edit", src, "-p", "r1+2", "--accept", "-o", path.join(tmp, "resolve-pair.docx"), "--datetime", "2026-10-11T00:00:00Z");
  assert.equal(pair.code, 0, pair.err);
  assert.equal(pair.out, 'accepted r1+2 in p3: "{~~weekly~>monthly~~}" > "monthly"\n');
});

test("read --json and --tables take the view's options; errors under --json are JSON (#1255)", { skip: !jsonErrors && "the checked-in binding predates #1255" }, () => {
  const plain = run("read", tracked, "--json");
  assert.equal(plain.code, 0, plain.err);
  for (const extra of [["--no-page-markers"], ["--comments", "none"], ["--dates"]]) {
    assert.equal(run("read", tracked, "--json", ...extra).out, plain.out, extra.join(" "));
  }
  const rejected = JSON.parse(run("read", tracked, "--json", "--track-changes", "rejected").out);
  assert.equal(rejected.track_changes, "reject");
  assert.equal(rejected.paragraphs.length, JSON.parse(plain.out).paragraphs.length);
  const usage = run("read", tracked, "--json", "--bogus");
  assert.equal(usage.code, 2);
  const error = JSON.parse(usage.out).error;
  assert.equal(error.code, "USAGE");
  assert.equal(error.usage, "jubarte-redlines read [OPTIONS] <FILE>");
  // #1161: the grids of the tables -p reaches, an empty line, its view.
  const grids = run("read", untracked, "--tables");
  const view = run("read", untracked, "-p", "t0");
  const table = run("read", untracked, "--tables", "-p", "t0");
  assert.equal(table.code, 0, table.err);
  assert.match(table.out, /^table 0: /);
  assert.ok(table.out.endsWith(`\n${view.out}`), table.out);
  assert.ok(grids.out.startsWith(table.out.slice(0, -view.out.length - 1)), table.out);
  // p5 is a cell of t0; p0 is outside every table.
  assert.equal(run("read", untracked, "--tables", "-p", "p5").out, `${grids.out}\n${run("read", untracked, "-p", "p5").out}`);
  const outside = run("read", untracked, "--tables", "-p", "p0");
  assert.equal(outside.code, 2);
  assert.match(outside.err, /p0 is not in a table; --tables -p takes tN, or a pN inside a table/);
  assert.equal(outside.out, "");
  const missing = run("read", untracked, "--tables", "-p", "t9");
  assert.equal(missing.code, run("read", untracked, "-p", "t9").code);
  assert.match(missing.err, /t9 is not a table of this document/);
});

test("JSON snapshots carry validity; JSON Lines send status to stderr (#1925)", { skip: !inputValidity && "the checked-in binding predates #1309" }, () => {
  copy(path.join(repo, "tests/fixtures/agent-view/received.docx"), "received-1309.docx");
  const wanted = { errors: 0, warnings: 2, next: "jubarte validate received-1309.docx" };
  for (const quiet of [[], ["-q"]]) {
    const read = run("read", "received-1309.docx", "--json", ...quiet);
    assert.equal(read.code, 0, read.err);
    assert.equal(read.err, "");
    assert.deepEqual(JSON.parse(read.out).summary.input_validity, wanted);
    const changes = run("changes", "received-1309.docx", "--json", ...quiet);
    assert.equal(changes.code, 0, changes.err);
    assert.equal(changes.err, quiet.length ? "" : JSON.stringify({ input_validity: wanted }) + "\n");
    const lines = changes.out.trim().split("\n");
    assert.ok(lines.every((line) => ["kind", "id", "paragraph"].every((field) => field in JSON.parse(line))), changes.out);
  }
  // -o FILE.json writes what --json prints; the stderr line stays.
  const printed = run("read", "received-1309.docx", "--json").out;
  const written = run("read", "received-1309.docx", "-o", "received-1309.json", "--force");
  assert.equal(written.code, 0, written.err);
  assert.equal(fs.readFileSync(path.join(tmp, "received-1309.json"), "utf8"), printed);
  assert.match(written.err, /^input has 2 validity warnings; jubarte validate received-1309.docx lists them\n/);
  for (const file of [tracked, untracked]) {
    const quiet = run("changes", file, "--json", "-q");
    assert.equal(quiet.code, 0, quiet.err);
    assert.equal(quiet.err, "");
    assert.ok(records(quiet.out).every((row) => "kind" in row), quiet.out);
  }
  // A clean input counts zero, with nothing to run next.
  assert.deepEqual(JSON.parse(run("read", untracked, "--json").out).summary.input_validity, { errors: 0, warnings: 0 });
});

test("read and changes name the input's validity findings once; -q drops it (#1256)", { skip: !inputNotices && "the checked-in binding predates #1256" }, () => {
  copy(path.join(repo, "tests/fixtures/agent-view/received.docx"), "received-1256.docx");
  const line = "input has 2 validity warnings; jubarte validate received-1256.docx lists them\n";
  // A JSON read carries the counts instead once the binding has them (#1309).
  const reads = [["read", "received-1256.docx", "--no-page-markers"], ["changes", "received-1256.docx"]];
  if (!inputValidity) reads.push(["read", "received-1256.docx", "--json"]);
  for (const args of reads) {
    const r = run(...args);
    assert.equal(r.code, 0, r.err);
    assert.equal(r.err, line, args.join(" "));
    assert.doesNotMatch(r.out, /validity/);
  }
  for (const args of [["-q", "read", "received-1256.docx", "--no-page-markers"], ["changes", "received-1256.docx", "-q"], ["changes", untracked]]) {
    const r = run(...args);
    assert.equal(r.code, 0, r.err);
    assert.equal(r.err, "", args.join(" "));
  }
});

// `read --for-edit` (V2-10): the working-document JSON the binary prints,
// decisions and refusals with its exit codes.
test("read --for-edit prints the binary's working-document JSON", () => {
  const src = copy(path.join(repo, "tests/fixtures/agent-view/received.docx"), "received-for-edit.docx");
  const plain = run("read", "received-for-edit.docx", "--for-edit");
  assert.equal(plain.code, 0, plain.err);
  const result = JSON.parse(plain.out);
  assert.equal(result.schema, "jubarte-read-for-edit-v1");
  assert.deepEqual(result.settled, { changed: false });
  assert.equal(result.basis.existing_revisions, "keep");
  assert.equal(result.basis.track, "on");
  assert.equal(result.paragraphs.length, 28);
  assert.deepEqual(result.input_validity, {
    errors: 0, warnings: 2, next: "jubarte validate received-for-edit.docx",
  });
  const p3 = result.paragraphs.find((p) => p.target.short_id === "p3");
  assert.match(p3.original.text, /^Consultant will deliver/);

  // --reject-change r0 settles the insertion out of the working text.
  const decided = run("read", "received-for-edit.docx", "--for-edit", "--reject-change", "r0");
  assert.equal(decided.code, 0, decided.err);
  const rejected = JSON.parse(decided.out);
  assert.deepEqual(rejected.basis.reject, ["r0"]);
  assert.deepEqual(rejected.settled, { changed: true, rejected: ["r0"] });
  const rejectedP3 = rejected.paragraphs.find((p) => p.target.short_id === "p3");
  assert.equal(
    rejectedP3.working.text,
    "Consultant will deliver the services in each Statement of Work, including  route-cost reports and a monthly review call.",
  );

  // An unknown change id is a plan refusal (exit 3), as the binary's.
  const unknown = run("read", "received-for-edit.docx", "--for-edit", "--accept-change", "r99");
  assert.equal(unknown.code, 3, unknown.err);
  assert.equal(unknown.out, "");
  assert.match(unknown.err, /error: UNKNOWN_CHANGE: resolve_revisions: no tracked change r99/);

  // -o writes the same JSON; stdout is free, so the input's validity
  // notice and the wrote line stay on stderr.
  const out = path.join(tmp, "fe.json");
  const written = run("read", "received-for-edit.docx", "--for-edit", "-o", "fe.json");
  assert.equal(written.code, 0, written.err);
  assert.equal(written.out, "");
  // One read: the size the wrote line names and the JSON are the same bytes.
  const bytes = fs.readFileSync(out);
  assert.equal(
    written.err,
    `input has 2 validity warnings; jubarte validate received-for-edit.docx lists them\n` +
      `wrote fe.json (json, ${bytes.length} bytes)\n`,
  );
  assert.deepEqual(JSON.parse(bytes.toString("utf8")), result);
  void src;
});

test("read --for-edit usage errors as the binary", () => {
  copy(path.join(repo, "tests/fixtures/agent-view/received.docx"), "received-for-edit-usage.docx");
  for (const args of [
    ["read", "received-for-edit-usage.docx", "--accept-change", "r0"],
    ["read", "received-for-edit-usage.docx", "--for-edit", "-o", "fe.md"],
  ]) {
    const r = run(...args);
    assert.equal(r.code, 2, `${args}: ${r.out} ${r.err}`);
    assert.match(r.err, /error:/);
  }
  // --json no longer conflicts with --for-edit (#1889).
  assert.equal(run("read", "received-for-edit-usage.docx", "--for-edit", "--json").code, 0);
});

test("changes --author keeps those authors' changes, as the binary (#1690)", { skip: !authorFilter && "the checked-in binding predates #1690" }, () => {
  const bo = run("changes", tracked, "--author", "Bo Chen");
  assert.equal(bo.code, 0, bo.err);
  const lines = bo.out.trimEnd().split("\n");
  assert.equal(lines.at(-1), "3 change(s)", bo.out);
  assert.ok(lines.slice(0, -1).every((line) => line.split("\t")[3] === "Bo Chen"), bo.out);
  const both = run("changes", tracked, "--author", "Bo Chen", "--author", "Ana Lima");
  assert.equal(both.out, run("changes", tracked).out);
  assert.equal(run("changes", tracked, "--author", "Bo").out, "0 change(s)\n");
  const rows = records(run("changes", tracked, "--json", "--author", "Ana Lima").out);
  assert.ok(rows.length === 2 && rows.every((c) => c.author === "Ana Lima"), JSON.stringify(rows));
});

test("read --tables -o writes the grids it prints, as the binary (#1690)", { skip: !tablesOutput && "the checked-in binding predates #1690" }, () => {
  const grids = run("read", untracked, "--tables");
  assert.equal(grids.code, 0, grids.err);
  const out = path.join(tmp, "grids.md");
  const wrote = run("read", untracked, "--tables", "-o", out);
  assert.equal(wrote.code, 0, wrote.err);
  assert.equal(wrote.out, "");
  assert.equal(fs.readFileSync(out, "utf8"), grids.out);
  assert.match(wrote.err, new RegExp(`wrote ${out.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")} \\(tables, ${fs.statSync(out).size} bytes\\)\n$`));
  // Refused as the binary refuses it: exit 2 with the OUTPUT_EXISTS code (#1692).
  const again = run("read", untracked, "--tables", "-o", out);
  assert.equal(again.code, 2, again.err);
  assert.equal(again.err, `error: OUTPUT_EXISTS: output '${out}' already exists (use --force to overwrite)\n`);
  const json = run("read", untracked, "--tables", "-o", path.join(tmp, "grids.json"));
  assert.equal(json.code, 2, json.err);
  assert.match(json.err, /--tables writes the grids as text/);
});

test("an existing output is a usage refusal with the OUTPUT_EXISTS code (#1698)", () => {
  const out = path.join(tmp, "taken.md");
  fs.writeFileSync(out, "keep me\n");
  // The binary's wording and code: exit 2, `OUTPUT_EXISTS:`.
  const again = run("compare", path.join(pair, "base.docx"), path.join(pair, "next.docx"), "-o", out);
  assert.equal(again.code, 2, again.err);
  assert.equal(again.err, `error: OUTPUT_EXISTS: output '${out}' already exists (use --force to overwrite)\n`);
  assert.equal(fs.readFileSync(out, "utf8"), "keep me\n");
  // Every task that writes says it the same way.
  for (const args of [
    ["accept", tracked, "-o", out],
    ["reject", tracked, "-o", out],
    ["convert", tracked, "-o", out],
    ["diff", tracked, tracked, "-o", out],
  ]) {
    const refused = run(...args);
    assert.equal(refused.code, 2, `${args.join(" ")}: ${refused.err}`);
    assert.match(refused.err, /^error: OUTPUT_EXISTS: output '.*' already exists \(use --force to overwrite\)\n$/);
  }
  assert.equal(fs.readFileSync(out, "utf8"), "keep me\n");
  // --force writes it.
  const docx = path.join(tmp, "taken.docx");
  const forced = run("accept", tracked, "-o", docx, "--force");
  assert.equal(forced.code, 0, forced.err);
  assert.deepEqual(fs.readFileSync(docx).subarray(0, 2), Buffer.from("PK"));
  // The `/` spelling on every OS, as the binary's messages (#568, #1692):
  // a name that holds a backslash is shown with a slash.
  const odd = path.join(tmp, "odd\\name.md");
  fs.writeFileSync(odd, "keep me\n");
  const backslash = run("convert", tracked, "-o", odd);
  assert.equal(backslash.code, 2, backslash.err);
  assert.equal(backslash.err, `error: OUTPUT_EXISTS: output '${odd.replaceAll("\\", "/")}' already exists (use --force to overwrite)\n`);
  assert.equal(fs.readFileSync(odd, "utf8"), "keep me\n");
});

test("read refuses an existing output before it reads the input (#1698)", () => {
  const out = path.join(tmp, "taken-view.md");
  fs.writeFileSync(out, "keep me\n");
  // The output is refused first: a missing input does not turn the refusal
  // into a reading error.
  const missing = run("read", "missing.docx", "-o", out);
  assert.equal(missing.code, 2, missing.err);
  assert.equal(missing.err, `error: OUTPUT_EXISTS: output '${out}' already exists (use --force to overwrite)\n`);
  assert.equal(fs.readFileSync(out, "utf8"), "keep me\n");
  // The tables view refuses it the same way, before it reads the input.
  const tables = runTables("read", "missing.docx", "--tables", "-o", out);
  assert.equal(tables.code, 2, tables.err);
  assert.equal(tables.err, `error: OUTPUT_EXISTS: output '${out}' already exists (use --force to overwrite)\n`);
  assert.equal(fs.readFileSync(out, "utf8"), "keep me\n");
});

test("read --tables -o refuses an output that names the input (#1698)", () => {
  // An extensionless input: the tables view would destroy the document.
  const input = path.join(tmp, "extensionless-input");
  fs.copyFileSync(untracked, input);
  const before = fs.readFileSync(input);
  const refused = runTables("read", input, "--tables", "-o", input, "--force");
  assert.equal(refused.code, 2, refused.err);
  assert.equal(refused.err, `error: OUTPUT_ALIAS: the output '${input}' is the same file as the input '${input}'\n`);
  assert.deepEqual(fs.readFileSync(input), before, "the input keeps its bytes");
  // A link to the input is the input.
  const link = path.join(tmp, "link-to-input.md");
  fs.symlinkSync(input, link);
  const throughLink = runTables("read", input, "--tables", "-o", link, "--force");
  assert.equal(throughLink.code, 2, throughLink.err);
  assert.match(throughLink.err, /^error: OUTPUT_ALIAS: the output '.*' is the same file as the input '.*'\n$/);
  assert.deepEqual(fs.readFileSync(input), before, "the input keeps its bytes");
  // Another spelling of the same path is the same file.
  const spelled = runTables("read", input, "--tables", "-o", `${tmp}/./extensionless-input`, "--force");
  assert.equal(spelled.code, 2, spelled.err);
  assert.match(spelled.err, /OUTPUT_ALIAS/);
  assert.deepEqual(fs.readFileSync(input), before, "the input keeps its bytes");
});

test("read --tables -o replaces a link, never writing through it (#1698)", () => {
  const referent = path.join(tmp, "referent.md");
  fs.writeFileSync(referent, "keep me\n");
  const link = path.join(tmp, "link.md");
  fs.symlinkSync(referent, link);
  const written = runTables("read", untracked, "--tables", "-o", link, "--force");
  assert.equal(written.code, 0, written.err);
  assert.equal(fs.readFileSync(referent, "utf8"), "keep me\n", "the link's referent keeps its bytes");
  // Read before checking the link (CodeQL js/file-system-race, #1803).
  const replaced = fs.readFileSync(link, "utf8");
  assert.ok(!fs.lstatSync(link).isSymbolicLink(), "the link itself is replaced");
  assert.match(replaced, /^table 0: /);
  // A hard link to the input is the input: it is refused, not written
  // through, and the document keeps its bytes.
  const source = path.join(tmp, "hard-link-source.docx");
  fs.copyFileSync(untracked, source);
  const before = fs.readFileSync(source);
  const hard = path.join(tmp, "hard-link.md");
  fs.linkSync(source, hard);
  const refused = runTables("read", source, "--tables", "-o", hard, "--force");
  assert.equal(refused.code, 2, refused.err);
  assert.match(refused.err, /^error: OUTPUT_ALIAS: the output '.*' is the same file as the input '.*'\n$/);
  assert.deepEqual(fs.readFileSync(source), before, "the input keeps its bytes");
  assert.deepEqual(fs.readFileSync(hard), before, "the hard link keeps the input's bytes");
});

// The checked-in npm wasm predates this checkout's parser, so `accept
// FILE --in-place` is an unknown flag there. The runner still honours
// `args.in_place` once the parser reports it. Until `jubarte-wasm/build-npm.sh`
// rebuilds npm/ from a clean tree, these tests drive that runner through a
// parseCli shim, the same way `--tables -o` does. The real bin's refusal is
// its own test.
const inPlaceParsed = parses(["accept", "a.docx", "--in-place"]);

function inPlaceBin() {
  const host = path.join(tmp, "in-place-host");
  const script = path.join(host, "jubarte-redlines.mjs");
  if (fs.existsSync(script)) return script;
  const modules = path.join(host, "node_modules", "jubarte-wasm");
  fs.mkdirSync(modules, { recursive: true });
  fs.writeFileSync(script, fs.readFileSync(bin));
  fs.writeFileSync(path.join(modules, "package.json"), JSON.stringify({ name: "jubarte-wasm", main: "index.js" }));
  fs.writeFileSync(path.join(modules, "index.js"), [
    `const real = require(${JSON.stringify(path.join(cliDir, "..", "npm"))});`,
    `const fs = require("node:fs");`,
    "const parse = real.parseCli;",
    "module.exports = {",
    "  ...real,",
    "  parseCli(argv, ...rest) {",
    "    const args = JSON.parse(argv);",
    "    const direct = JSON.parse(parse(argv, ...rest));",
    "    if (!(direct.exit_code && direct.exit_code !== 0)) return JSON.stringify(direct);",
    "    const at = args.indexOf('--in-place');",
    "    if (at === -1) return JSON.stringify(direct);",
    "    const stripped = args.filter((_, i) => i !== at);",
    "    if ((stripped[0] === 'accept' || stripped[0] === 'reject') && !stripped.includes('-o') && !stripped.includes('--output')) {",
    "      stripped.splice(1, 0, '-o', stripped[1]);",
    "    }",
    "    const parsed = JSON.parse(parse(JSON.stringify(stripped), ...rest));",
    "    if (parsed.exit_code && parsed.exit_code !== 0) return JSON.stringify(parsed);",
    "    parsed.args.in_place = true;",
    "    if (parsed.args.author_given == null) parsed.args.author_given = args.includes('-a') || args.includes('--author');",
    "    if (parsed.command === 'accept' || parsed.command === 'reject') {",
    "      parsed.args.output = parsed.args.file;",
    "      parsed.args.output_format = 'docx';",
    "    } else if (parsed.command === 'edit' || parsed.command === 'add') {",
    "      parsed.args.output = parsed.args.file;",
    "    }",
    "    return JSON.stringify(parsed);",
    "  },",
    "  acceptChanges(docx, filter) {",
    "    if (process.env.JUBARTE_IN_PLACE_POISON) fs.writeFileSync(process.env.JUBARTE_IN_PLACE_POISON, 'poisoned-source');",
    "    return real.acceptChanges(docx, filter);",
    "  },",
    "};",
  ].join("\n"));
  return script;
}

function runInPlace(env, ...args) {
  const r = spawnSync(process.execPath, [inPlaceBin(), ...args], {
    encoding: "utf8",
    cwd: tmp,
    env: { ...process.env, ...env },
  });
  return { code: r.status, out: r.stdout, err: r.stderr };
}

function editAnchor() {
  const para = JSON.parse(run("read", untracked, "--json").out).paragraphs.find((p) => p.text.length > 3);
  return { index: para.index, word: para.text.split(" ")[0] };
}

test("the checked-in parser refuses --in-place until npm/ is rebuilt", { skip: inPlaceParsed && "this npm build already parses --in-place" }, () => {
  const refused = run("accept", "a.docx", "--in-place");
  assert.equal(refused.code, 2, refused.err);
  assert.match(refused.err, /--in-place/);
  assert.doesNotMatch(refused.err, /in place \(replaced sha256/);
});

test("accept --in-place of a misnamed docx stays a docx (#1888)", () => {
  for (const name of ["contract.json", "notes.md"]) {
    const file = copy(tracked, name);
    const before = fs.readFileSync(file);
    const accepted = runInPlace({}, "accept", file, "--in-place");
    assert.equal(accepted.code, 0, accepted.err);
    assert.ok(fs.readFileSync(file).subarray(0, 4).equals(Buffer.from("PK\x03\x04")), name);
    assert.ok(!fs.readFileSync(file).equals(before), name);
    assert.ok(accepted.err.includes(`wrote ${file} in place (replaced sha256 `), accepted.err);
    assert.match(accepted.err, /replaced sha256 [0-9a-f]{8}\u2026/);
    assert.doesNotMatch(accepted.err, /\{"schema_version"/);
  }
});

test("edit --in-place does not clobber a pdf, and a failed companion leaves the input (#1888)", () => {
  const { index, word } = editAnchor();
  const file = copy(untracked, "pdf-in.docx");
  const pdf = path.join(tmp, "pdf-in.pdf");
  fs.writeFileSync(pdf, "keep-pdf");
  const before = fs.readFileSync(file);
  const refused = runInPlace({ JUBARTE_AUTHOR: "Jubarte" }, "edit", file, "-p", `p${index}`, "--anchor", word, "--content", "Changed", "--in-place", "--pdf");
  assert.equal(refused.code, 2, refused.err);
  assert.match(refused.err, /OUTPUT_EXISTS/);
  assert.match(refused.err, /pdf-in\.pdf/);
  assert.deepEqual(fs.readFileSync(file), before);
  assert.equal(fs.readFileSync(pdf, "utf8"), "keep-pdf");

  const companion = copy(untracked, "comp.docx");
  const blocked = path.join(tmp, "comp.pdf");
  fs.mkdirSync(blocked);
  fs.writeFileSync(path.join(blocked, "keep"), "keep");
  const held = fs.readFileSync(companion);
  const failed = runInPlace({ JUBARTE_AUTHOR: "Jubarte" }, "edit", companion, "-p", `p${index}`, "--anchor", word, "--content", "Changed", "--in-place", "--pdf", "--force");
  assert.notEqual(failed.code, 0, failed.err);
  assert.deepEqual(fs.readFileSync(companion), held, failed.err);
  assert.equal(fs.readFileSync(path.join(blocked, "keep"), "utf8"), "keep");
});

test("edit --in-place follows a symlink and keeps its mode (#1888)", () => {
  const { index, word } = editAnchor();
  const real = path.join(tmp, "real.docx");
  fs.copyFileSync(untracked, real);
  fs.chmodSync(real, 0o640);
  const link = path.join(tmp, "link.docx");
  fs.symlinkSync("real.docx", link);
  const before = fs.readFileSync(real);
  const edited = runInPlace({}, "edit", link, "-p", `p${index}`, "--anchor", word, "--content", "Changed", "-a", "GiantCo", "--in-place");
  assert.equal(edited.code, 0, edited.err);
  assert.ok(fs.lstatSync(link).isSymbolicLink(), edited.err);
  assert.equal(fs.readlinkSync(link), "real.docx");
  assert.equal(fs.statSync(real).mode & 0o777, 0o640);
  assert.ok(!fs.readFileSync(real).equals(before));
  assert.match(edited.err, /wrote .*link\.docx in place \(replaced sha256 [0-9a-f]{8}\u2026\); \d+ changes? by "GiantCo"/);
  assert.doesNotMatch(edited.err, /no -a given/);
});

test("accept --in-place rechecks the source before replacing (#1888)", () => {
  const file = copy(tracked, "poison.docx");
  const poisoned = runInPlace({ JUBARTE_IN_PLACE_POISON: file }, "accept", file, "--in-place");
  assert.equal(poisoned.code, 3, poisoned.err);
  assert.match(poisoned.err, /SOURCE_CHANGED:/);
  assert.match(poisoned.err, /nothing was published/);
  assert.equal(fs.readFileSync(file, "utf8"), "poisoned-source");
  assert.doesNotMatch(poisoned.err, /in place \(replaced sha256/);
});
