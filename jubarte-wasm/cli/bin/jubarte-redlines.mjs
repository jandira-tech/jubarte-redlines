#!/usr/bin/env node
// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

// `npx jubarte-redlines`: the commands of `uvx jubarte-redlines` (and the
// `jubarte` binary) over the jubarte-wasm package, with the same names,
// flags, output files, messages and exit codes.
//
//   npx jubarte-redlines redline a.docx b.docx -o redline.docx
//
// Exit codes: 0 success, 1 error (I/O, engine, existing output), 2 usage,
// 3 edit plan refused (its per-operation report is on stdout; nothing was
// written).

import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { parseArgs } from "node:util";

const require = createRequire(import.meta.url);
const pkg = require("../package.json");
const wasm = require("jubarte-wasm");

const PROG = "jubarte-redlines";
const EXIT_OK = 0;
const EXIT_ERROR = 1;
const EXIT_USAGE = 2;
const EXIT_PLAN_REFUSED = 3;

// The first bytes of an OLE compound file: a Word 97-2003 .doc, or a
// password-encrypted document of any Word version.
const OLE_MAGIC = Buffer.from([0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]);

/** A user-facing failure; printed as `error: ...`, exit 1. */
class CliError extends Error {}

/** A usage mistake; printed with the command's usage line, exit 2. */
class UsageError extends Error {}

function read(file) {
  let bytes;
  try {
    bytes = fs.readFileSync(file);
  } catch (e) {
    throw new CliError(`reading ${file}: ${e.message}`);
  }
  if (bytes.subarray(0, OLE_MAGIC.length).equals(OLE_MAGIC)) {
    throw new CliError(`${file} is a Word 97-2003 (.doc) or encrypted document; open it in Word and save it as .docx without a password`);
  }
  return bytes;
}

function ensureWritable(file, force) {
  if (fs.existsSync(file) && !force) {
    throw new CliError(`output '${file}' already exists (use --force to overwrite)`);
  }
}

function write(file, data) {
  try {
    fs.writeFileSync(file, data);
  } catch (e) {
    throw new CliError(`writing ${file}: ${e.message}`);
  }
}

function jsonLines(json) {
  for (const row of JSON.parse(json)) console.log(JSON.stringify(row));
}

const plural = (n, word) => `${n} ${word}${n === 1 ? "" : "s"}`;

// -- commands -----------------------------------------------------------------

const REVISION_FLAGS = {
  revisions: { type: "string", default: "conventional", help: "how tracked changes are painted: conventional, word or custom" },
  "revision-palette": { type: "string", help: "marks for --revisions custom, e.g. deleted=#AA0000:strike,..." },
};

const COMMANDS = {
  compare: {
    aliases: ["redline"],
    args: "ORIGINAL MODIFIED",
    help: "two documents into a Word tracked-changes document",
    options: {
      output: { type: "string", short: "o", help: "[default: <original>_v_<modified>.docx]" },
      author: { type: "string", default: "jubarte", help: "who the revisions are by" },
      force: { type: "boolean", help: "overwrite an existing output" },
    },
    run(name, [original, modified], o) {
      if (modified === undefined) throw new UsageError(`${name} needs ORIGINAL and MODIFIED`);
      const output = o.output ?? path.join(path.dirname(original), `${stem(original)}_v_${stem(modified)}.docx`);
      const [a, b] = [read(original), read(modified)];
      ensureWritable(output, o.force);
      const redline = wasm.compareDocuments(a, b, o.author);
      write(output, redline);
      console.log(`wrote ${output} (${redline.length} bytes)`);
    },
  },
  changes: {
    args: "FILE",
    help: "list each tracked change with the id accept/reject --id and edit plans take",
    options: { json: { type: "boolean", help: "one JSON object per line" } },
    run(_, [file], o) {
      const changes = JSON.parse(wasm.listChanges(read(file)));
      for (const change of changes) {
        if (o.json) {
          console.log(JSON.stringify(Object.fromEntries(Object.entries(change).filter(([, v]) => v !== null))));
          continue;
        }
        const inside = change.inside ? `\tinside ${change.inside}` : "";
        console.log(`${change.id}\t${change.kind}\t${change.target}\t${change.author || "-"}\t${JSON.stringify([...change.text].slice(0, 60).join(""))}${inside}`);
      }
      if (!o.json) console.log(`${changes.length} change(s)`);
    },
  },
  revisions: {
    args: "FILE",
    help: "list tracked revisions",
    options: { json: { type: "boolean", help: "one JSON object per line" } },
    run(_, [file], o) {
      const rows = JSON.parse(wasm.getRevisions(read(file)));
      if (o.json) return jsonLines(JSON.stringify(rows));
      for (const row of rows) {
        console.log(`${row.type}\t${row.author || "-"}\t${row.part}\t${JSON.stringify([...(row.text || "")].slice(0, 60).join(""))}`);
      }
      console.log(`${rows.length} revision(s)`);
    },
  },
  accept: resolution(true),
  reject: resolution(false),
  text: {
    args: "FILE",
    help: "Markdown with [body:p:N] ids, the coordinates an edit plan uses",
    options: {},
    run(_, [file]) {
      process.stdout.write(wasm.documentMarkdown(read(file)));
    },
  },
  inspect: {
    args: "FILE",
    help: "paragraph ids, formatting spans, limitations and package facts",
    options: { json: { type: "boolean", help: "emit the JSON snapshot" } },
    run(_, [file], o) {
      const json = wasm.inspectDocument(read(file));
      if (o.json) return console.log(json);
      const snap = JSON.parse(json);
      const s = snap.summary;
      console.log(`sha256: ${snap.source_sha256}`);
      console.log(
        `paragraphs: ${s.paragraphs}  tables: ${s.tables}  fields: ${s.fields}  sections: ${s.sections}  ` +
          `comments: ${s.comments}  revisions: ${s.revisions}  footnotes: ${s.footnotes}  endnotes: ${s.endnotes}  ` +
          `headers: ${s.headers}  footers: ${s.footers}  images: ${s.images}  numbering: ${s.list_numbering}  ` +
          `track_changes: ${s.track_changes}`,
      );
      for (const p of snap.paragraphs) {
        const flags = [p.style, p.numbered && "numbered", p.in_table && "table", p.page_break && "page-break", ...p.limitations].filter(Boolean);
        const chars = [...p.text];
        console.log(`${p.id}\t[${flags.join(",")}]\t${chars.slice(0, 80).join("")}${chars.length > 80 ? "…" : ""}`);
      }
    },
  },
  convert: {
    args: "FILE",
    help: "DOCX to PDF",
    options: {
      output: { type: "string", short: "o", help: "PDF path [default: <stem>.pdf beside the input]" },
      force: { type: "boolean", help: "overwrite an existing output" },
      pdf: { type: "boolean", help: "write the PDF (the default)" },
      png: { type: "boolean", help: "not in this build: use uvx jubarte-redlines or the jubarte binary" },
      compress: { type: "boolean", help: "deflate PDF streams" },
      ...REVISION_FLAGS,
      "move-comments": { type: "boolean", help: "list the comments after the last page instead of in balloons beside the text" },
      "changed-only": { type: "boolean", help: "keep only the pages a tracked change touches" },
    },
    run(_, [file], o) {
      if (o.png) throw new CliError("PNG pages need the Python or Rust build (uvx jubarte-redlines convert --png)");
      const palette = paletteOf(o);
      const output = o.output ?? path.join(path.dirname(file), `${stem(file)}.pdf`);
      const docx = read(file);
      ensureWritable(output, o.force);
      const pdf = wasm.docxToPdf(docx, Boolean(o.compress), o.revisions, palette, Boolean(o["move-comments"]), Boolean(o["changed-only"]));
      write(output, pdf);
      console.log(`wrote ${output} (${pdf.length} bytes, ${plural(wasm.pdfPageCount(pdf), "page")})`);
    },
  },
  edit: {
    args: "FILE",
    help: "apply an edit plan: clean.docx, redline.docx, patch.diff, report.jsonl",
    options: {
      plan: { type: "string", help: "PLAN.json (required)" },
      "out-dir": { type: "string", help: "DIR (required)" },
      "dry-run": { type: "boolean", help: "resolve and report only; write nothing" },
      force: { type: "boolean", help: "replace an existing output directory's files" },
      quiet: { type: "boolean", short: "q", help: "print nothing on success" },
    },
    run(name, [file], o) {
      if (o.plan === undefined || o["out-dir"] === undefined) throw new UsageError(`${name} needs --plan and --out-dir`);
      const docx = read(file);
      let plan;
      try {
        plan = fs.readFileSync(o.plan, "utf8");
      } catch (e) {
        throw new CliError(`reading ${o.plan}: ${e.message}`);
      }
      const outDir = o["out-dir"];
      if (!o["dry-run"]) {
        if (fs.existsSync(outDir) && !o.force) {
          throw new CliError(`output directory '${outDir}' already exists (use --force to replace its files)`);
        }
        if (path.resolve(outDir) === path.dirname(path.resolve(file))) {
          throw new CliError("--out-dir must not be the input's own directory");
        }
      }
      const result = o["dry-run"] ? wasm.previewEditPlan(docx, plan) : wasm.applyEditPlan(docx, plan);
      if (!result.ok) {
        const error = JSON.parse(result.json);
        (error.outcomes ?? []).forEach((outcome, i) => {
          const row = { ev: "op", i: i + 1, id: outcome.id, op: outcome.kind, status: outcome.status, matches: outcome.matches };
          for (const [key, field] of [["at", "paragraph"], ["ctx", "context"], ["code", "code"], ["message", "message"]]) {
            if (outcome[field] != null) row[key] = outcome[field];
          }
          console.log(JSON.stringify(row));
        });
        console.log(JSON.stringify({ ev: "summary", status: "failed", code: error.code, operation: error.operation, message: error.message }));
        console.error(`error: plan refused: ${error.code}${error.operation ? ` (${error.operation})` : ""}: ${error.message}`);
        return EXIT_PLAN_REFUSED;
      }
      const jsonl = wasm.editReportJsonl(result.json);
      if (o["dry-run"]) return void process.stdout.write(jsonl);
      const lines = jsonl.trimEnd().split("\n");
      const summary = lines.pop();
      // jubarte-wasm 0.10.1 has no patch; later builds carry it.
      const outputs = [["clean.docx", result.clean], ["redline.docx", result.redline]];
      if (typeof result.patch === "string") outputs.push(["patch.diff", Buffer.from(result.patch, "utf8")]);
      fs.mkdirSync(outDir, { recursive: true });
      const saved = outputs.map(([name, data]) => {
        write(path.join(outDir, name), data);
        return { f: name, bytes: data.length, sha256: wasm.sourceSha256(data) };
      });
      lines.push(JSON.stringify({ ev: "save", dir: outDir, outputs: saved }), summary);
      write(path.join(outDir, "report.jsonl"), `${lines.join("\n")}\n`);
      if (o.quiet) return;
      console.log(summary);
      console.log(`wrote ${outDir} (${outputs.length + 1} files: ${[...outputs.map(([n]) => n), "report.jsonl"].join(", ")})`);
      if (typeof result.patch === "string") process.stdout.write(result.patch);
    },
  },
  capabilities: {
    args: "",
    help: "what this build can do",
    options: { json: { type: "boolean", help: "(the output is JSON either way)" } },
    run() {
      console.log(JSON.stringify(JSON.parse(wasm.capabilities()), null, 2));
    },
  },
};

function resolution(accept) {
  const verb = accept ? "accept" : "reject";
  return {
    args: "FILE",
    help: `${verb} tracked changes (all, or the ones selected)`,
    options: {
      output: { type: "string", short: "o", help: "output path (required)" },
      force: { type: "boolean", help: "overwrite an existing output" },
      id: { type: "string", multiple: true, help: "only this change (body:rev:12); repeatable" },
      author: { type: "string", multiple: true, help: "only changes by this author; repeatable" },
      kind: { type: "string", multiple: true, help: "only changes of this kind (insertion, deletion, move, formatting); repeatable" },
    },
    run(name, [file], o) {
      if (o.output === undefined) throw new UsageError(`${name} needs -o/--output`);
      const filter = {};
      for (const key of ["id", "author", "kind"]) if (o[key]) filter[`${key}s`] = o[key];
      const docx = read(file);
      ensureWritable(o.output, o.force);
      const out = (accept ? wasm.acceptChanges : wasm.rejectChanges)(docx, JSON.stringify(filter));
      write(o.output, out);
      console.log(`wrote ${o.output} (${out.length} bytes)`);
    },
  };
}

function paletteOf(o) {
  if (!["conventional", "word", "custom"].includes(o.revisions)) {
    throw new UsageError(`--revisions must be conventional, word or custom, not '${o.revisions}'`);
  }
  const palette = o["revision-palette"];
  if (o.revisions === "custom" && palette === undefined) throw new CliError("--revisions custom needs --revision-palette");
  if (o.revisions !== "custom" && palette !== undefined) throw new CliError("--revision-palette needs --revisions custom");
  return palette;
}

function stem(file) {
  return path.basename(file, path.extname(file));
}

// -- usage --------------------------------------------------------------------

function names(name) {
  return [name, ...(COMMANDS[name].aliases ?? [])];
}

function usage() {
  const rows = Object.keys(COMMANDS).map((name) => [[...names(name)].reverse().join(", "), COMMANDS[name].help]);
  const width = Math.max(...rows.map(([n]) => n.length));
  return [
    `usage: ${PROG} <command> [options]`,
    "",
    "DOCX compare, tracked editing, inspection and rendering (the jubarte engine, WebAssembly build).",
    "",
    "commands:",
    ...rows.map(([n, h]) => `  ${n.padEnd(width)}  ${h}`),
    "",
    `  ${PROG} <command> --help    a command's options`,
    `  ${PROG} --version`,
    "",
  ].join("\n");
}

function commandUsage(typed, command) {
  const rows = Object.entries(command.options).map(([flag, spec]) => {
    const short = spec.short ? `-${spec.short}, ` : "    ";
    const value = spec.type === "string" ? ` ${flag.toUpperCase().replaceAll("-", "_")}` : "";
    const fallback = spec.default !== undefined ? ` [default: ${spec.default}]` : "";
    return [`${short}--${flag}${value}`, `${spec.help ?? ""}${fallback}`];
  });
  const width = Math.max(0, ...rows.map(([f]) => f.length));
  return [
    `usage: ${PROG} ${typed}${command.args ? ` ${command.args}` : ""} [options]`,
    "",
    command.help,
    "",
    ...rows.map(([f, h]) => `  ${f.padEnd(width)}  ${h}`),
    "",
  ].join("\n");
}

// -- main ---------------------------------------------------------------------

function main(argv) {
  const [typed, ...rest] = argv;
  if (typed === undefined) {
    process.stderr.write(usage());
    return EXIT_USAGE;
  }
  if (typed === "-h" || typed === "--help" || typed === "help") {
    process.stdout.write(usage());
    return EXIT_OK;
  }
  if (typed === "-V" || typed === "--version") {
    const engine = JSON.parse(wasm.capabilities()).engine_version;
    console.log(`${PROG} ${pkg.version} (engine ${engine}, wasm)`);
    return EXIT_OK;
  }
  const name = Object.keys(COMMANDS).find((n) => names(n).includes(typed));
  if (name === undefined) {
    process.stderr.write(usage());
    console.error(`${PROG}: error: unknown command '${typed}'`);
    return EXIT_USAGE;
  }
  const command = COMMANDS[name];
  if (rest.includes("-h") || rest.includes("--help")) {
    process.stdout.write(commandUsage(typed, command));
    return EXIT_OK;
  }
  try {
    const options = Object.fromEntries(Object.entries(command.options).map(([flag, { help, ...spec }]) => [flag, spec]));
    const { values, positionals } = parseArgs({ args: rest, options, allowPositionals: true, strict: true });
    const want = command.args ? command.args.split(" ").length : 0;
    if (positionals.length < want) throw new UsageError(`${typed} needs ${command.args.replace(" ", " and ")}`);
    if (positionals.length > want) throw new UsageError(`${typed}: unexpected argument '${positionals[want]}'`);
    return command.run(typed, positionals, values) ?? EXIT_OK;
  } catch (e) {
    if (e instanceof UsageError || e.code?.startsWith?.("ERR_PARSE_ARGS")) {
      process.stderr.write(`usage: ${PROG} ${typed}${command.args ? ` ${command.args}` : ""} [options]\n`);
      console.error(`${PROG} ${typed}: error: ${e.message}`);
      return EXIT_USAGE;
    }
    // CliError, engine errors (thrown as strings or Errors), I/O surprises.
    console.error(`error: ${e instanceof Error ? e.message : String(e)}`);
    return EXIT_ERROR;
  }
}

process.exitCode = main(process.argv.slice(2));
