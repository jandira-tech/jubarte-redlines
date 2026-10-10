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
import { execFileSync } from "node:child_process";

const require = createRequire(import.meta.url);
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

/** A host capability mismatch; rejected before file I/O, exit 2. */
class UsageError extends Error {}

function read(file, forceKind) {
  let bytes;
  try {
    bytes = fs.readFileSync(file);
  } catch (e) {
    throw new CliError(`reading ${file}: ${e.message}`);
  }
  if (bytes.subarray(0, OLE_MAGIC.length).equals(OLE_MAGIC)) {
    throw new CliError(`${file} is a Word 97-2003 (.doc) or encrypted document; open it in Word and save it as .docx without a password`);
  }
  // Ask the same clap parser for declared input metadata on other commands.
  // Its format grammar stays in the core; unknown suffixes use ZIP sniffing.
  if (forceKind === undefined) {
    const parsed = JSON.parse(wasm.parseCli(JSON.stringify(["diff", "--", file, file])));
    forceKind = parsed.args.old_format;
  }
  if (forceKind === "docx" && !bytes.subarray(0, 4).equals(Buffer.from([0x50, 0x4b, 0x03, 0x04]))) {
    throw new CliError(`reading ${file}: invalid DOCX (expected a ZIP package)`);
  }
  if (forceKind !== "docx" && bytes.subarray(0, 3).equals(Buffer.from([0xef, 0xbb, 0xbf]))) {
    return bytes.subarray(3);
  }
  return bytes;
}

function ensureWritable(file, force) {
  if (fs.existsSync(file) && !force) {
    throw new CliError(`output '${file}' already exists (use --force to overwrite)`);
  }
}

const ZIP_MAGIC = Buffer.from([0x50, 0x4b, 0x03, 0x04]);

/** The format a file's name declares, or null when only its bytes tell. */
function namedKind(file) {
  return JSON.parse(wasm.parseCli(JSON.stringify(["diff", "--", file, file]))).args.old_format;
}

/** The input's kind: the declared or named format, else a ZIP is Word. */
function kindOf(file, bytes, declared) {
  return declared ?? namedKind(file) ?? (bytes.subarray(0, 4).equals(ZIP_MAGIC) ? "docx" : "md");
}

/** Word output from convert is a Markdown conversion. */
const convertsToDocx = (o) => o.to === "docx" || (o.to == null && path.extname(o.output ?? "").toLowerCase() === ".docx");
const docxNeedsMarkdown = () => new UsageError("--to docx requires Markdown input in the npm CLI");

/** Markdown bytes as text; malformed UTF-8 is refused, never replaced. */
function markdownText(file, bytes) {
  try {
    return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  } catch {
    throw new CliError(`${file}: Markdown must be UTF-8`);
  }
}

/** Markdown output is CriticMarkup of two Markdown documents. */
function markdownNeedsBoth(sides) {
  const word = sides.find(([, kind]) => kind !== "md");
  if (word) {
    throw new CliError(`Markdown output needs both documents in Markdown (${word[0]} is Word): write a Word redline (-o FILE.docx) or a PDF (-o FILE.pdf) instead`);
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

const COMMANDS = {
  compare: {
    run(name, [original, modified], o) {
      const fallback = path.join(path.dirname(original), `${stem(original)}_v_${stem(modified)}.docx`);
      const [a, b] = [read(original, o.old_format), read(modified, o.new_format)];
      // The shorthand `A B` without -o prints the redline's agent view.
      if (o.output == null && o.view != null) {
        return void printView(wasm.redlineDocuments(a, b, o.author, o.date), o.view, `${fallback} (not written; -o keeps it)`);
      }
      const output = o.output ?? fallback;
      if (o.output_format === "md") markdownNeedsBoth([[original, kindOf(original, a, o.old_format)], [modified, kindOf(modified, b, o.new_format)]]);
      ensureWritable(output, o.force);
      const redline = o.output_format === "md"
        ? wasm.diffDocumentsCritic(a, b, o.author, o.date)
        : wasm.redlineDocuments(a, b, o.author, o.date);
      write(output, redline);
      if (!o.quiet) console.log(`wrote ${output} (${redline.length} bytes)`);
    },
  },
  diff: {
    run(_, [old, next], o) {
      const view = ["github", "word", "normal", "context", "side-by-side"].includes(o.format);
      const a = read(old, o.old_format), b = read(next, o.new_format);
      const kind = (declared, bytes) => declared ?? (bytes.subarray(0, 4).equals(ZIP_MAGIC) ? "docx" : "md");
      const oldFormat = kind(o.old_format, a), newFormat = kind(o.new_format, b);
      // A side read as Markdown must be text, whichever engine call follows.
      if (oldFormat === "md") markdownText(old, a);
      if (newFormat === "md") markdownText(next, b);
      let output = o.output;
      const to = o.to ?? o.output_format ?? (oldFormat === "md" && newFormat === "md" ? "md" : "docx");
      if (!view && output == null && to !== "md") output = path.join(path.dirname(old), `${stem(old)}_v_${stem(next)}.${to}`);
      if (!view && output != null && to === "md") markdownNeedsBoth([[old, oldFormat], [next, newFormat]]);
      if (output != null) ensureWritable(output, o.force);
      if (view) {
        const text = wasm.diffDocumentsView(a, b, JSON.stringify({
          format: o.format, oldName: old, newName: next, context: o.context,
          acceptChanges: o.accept_changes, fullLines: o.full_lines, oldFormat, newFormat,
        }));
        if (output != null) {
          write(output, text);
          console.error(`wrote ${output} (${Buffer.byteLength(text, "utf8")} bytes)`);
        } else process.stdout.write(text);
        return;
      }
      const author = o.author ?? defaultAuthor();
      const date = o.date ?? new Date().toISOString().replace(/\.\d{3}Z$/, "Z");
      const text = o.format === "critic" ? wasm.diffDocumentsCritic(a, b, author, date)
        : JSON.parse(wasm.diffDocuments(a, b, author, date, o.columns, path.basename(old), path.basename(next))).text;
      if (output != null) {
        let data = text;
        if (to === "md") data = wasm.diffDocumentsCritic(a, b, author, date);
        if (["docx", "pdf"].includes(to)) {
          data = wasm.redlineDocuments(a, b, author, date);
          if (to === "pdf") data = wasm.docxToPdf(data, false, o.revisions, paletteOf(o), Boolean(o.move_comments), Boolean(o.changed_only));
        }
        write(output, data);
        // As natively: critic says only what it wrote; a patch keeps stdout.
        const wrote = `wrote ${output} (${typeof data === "string" ? Buffer.byteLength(data, "utf8") : data.length} bytes)`;
        if (o.format === "critic") return void console.log(wrote);
        console.error(wrote);
      }
      process.stdout.write(text);
    },
  },
  changes: {
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
  read: {
    run(_, [file], o) {
      printView(read(file), o, path.basename(file));
    },
  },
  inspect: {
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
    run(_, [file], o) {
      const palette = paletteOf(o);
      let docx = read(file);
      if (kindOf(file, docx) === "md") {
        docx = wasm.markdownToDocx(markdownText(file, docx), JSON.stringify({ page: o.page, author: o.author, date: o.date, critic: !o.no_critic, track_changes: o.track_changes }), o.reference_doc == null ? undefined : read(o.reference_doc));
        if (!o.pdf && (o.to === "docx" || (o.to == null && (o.output == null || path.extname(o.output).toLowerCase() === ".docx")))) {
          if (o.move_comments || o.changed_only) {
            throw new CliError("--move-comments / --changed-only applies to PDF or PNG output only");
          }
          const output = o.output ?? path.join(path.dirname(file), `${stem(file)}.docx`);
          ensureWritable(output, o.force);
          write(output, docx);
          console.log(`wrote ${output} (${docx.length} bytes)`);
          return;
        }
      } else if (convertsToDocx(o)) {
        // A name that said nothing; its bytes are Word.
        throw docxNeedsMarkdown();
      } else if (o.track_changes === "accept") {
        docx = wasm.acceptRevisions(docx);
      } else if (o.track_changes === "reject") {
        docx = wasm.rejectRevisions(docx);
      }
      const output = o.output ?? path.join(path.dirname(file), `${stem(file)}.pdf`);
      ensureWritable(output, o.force);
      const pdf = wasm.docxToPdf(docx, Boolean(o.compress), o.revisions, palette, Boolean(o.move_comments), Boolean(o.changed_only));
      write(output, pdf);
      console.log(`wrote ${output} (${pdf.length} bytes, ${plural(wasm.pdfPageCount(pdf), "page")})`);
    },
  },
  edit: { run: (name, [file], o) => runEdit("edit", file, o) },
  add: { run: (name, [file], o) => runEdit("add", file, o) },
  capabilities: {
    run() {
      console.log(JSON.stringify(JSON.parse(wasm.capabilities()), null, 2));
    },
  },
};

/** `--plan`'s text, or the plan the operation flags describe, and its notes. */
function editPlan(verb, docx, o) {
  if (o.plan != null) {
    try {
      return { plan: fs.readFileSync(o.plan, "utf8"), notes: [] };
    } catch (e) {
      throw new CliError(`reading ${o.plan}: ${e.message}`);
    }
  }
  try {
    const options = { author: o.author, date: o.datetime ?? undefined, existingRevisions: o.existing_revisions };
    return JSON.parse(wasm.flagPlan(verb, JSON.stringify(o.operations ?? []), docx, JSON.stringify(options)));
  } catch (e) {
    throw new UsageError(String(e?.message ?? e).replace(/^jubarte-wasm: /, ""));
  }
}

function runEdit(verb, file, o) {
  const docx = read(file);
  const { plan, notes } = editPlan(verb, docx, o);
  const editing = Boolean(o.editing_mode);
  if (editing && JSON.parse(plan).existing_revisions === "keep") {
    throw new UsageError("--editing-mode needs a document without tracked changes; it has some, so pass --existing-revisions accept or reject");
  }
  const outDir = o.out_dir ?? path.join(path.dirname(file), `${stem(file)}.edit`);
  if (!o.dry_run) {
    if (fs.existsSync(outDir) && !o.force) {
      throw new CliError(`output directory '${outDir}' already exists (use --force to replace its files)`);
    }
    if (path.resolve(outDir) === path.dirname(path.resolve(file))) {
      throw new CliError("--out-dir must not be the input's own directory");
    }
  }
  const result = o.dry_run ? wasm.previewEditPlan(docx, plan) : wasm.applyEditPlan(docx, plan);
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
  if (o.dry_run) return void process.stdout.write(jsonl);
  const lines = jsonl.trimEnd().split("\n");
  const summary = lines.pop();
  const outputs = [["clean.docx", result.clean]];
  if (!editing) {
    outputs.push(["redline.docx", result.redline]);
    // jubarte-wasm 0.10.1 has no patch; later builds carry it.
    if (typeof result.patch === "string") outputs.push(["patch.diff", Buffer.from(result.patch, "utf8")]);
  }
  fs.mkdirSync(outDir, { recursive: true });
  const saved = outputs.map(([name, data]) => {
    write(path.join(outDir, name), data);
    return { f: name, bytes: data.length, sha256: wasm.sourceSha256(data) };
  });
  lines.push(JSON.stringify({ ev: "save", dir: outDir, outputs: saved }), summary);
  write(path.join(outDir, "report.jsonl"), `${lines.join("\n")}\n`);
  if (o.quiet) return;
  console.log(summary);
  const names = [...outputs.map(([n]) => n), "report.jsonl"];
  console.log(`wrote ${outDir} (${names.length} files: ${names.join(", ")})`);
  for (const note of notes) console.log(`note: ${note}`);
  const report = JSON.parse(result.json);
  for (const outcome of report.operations ?? []) {
    if (outcome.anchor_given != null && outcome.anchor_read_as != null) {
      console.log(`note: ${outcome.id}: anchor ${JSON.stringify(outcome.anchor_given)} read as ${JSON.stringify(outcome.anchor_read_as)} (Markdown marks are not document text)`);
    }
  }
  const shown = path.join(outDir, editing ? "clean.docx" : "redline.docx");
  process.stdout.write(wasm.changedView(result.redline, report.author, editing, shown));
}

/** The agent view of `docx` with the read options `o`; `source` is its header name. */
function printView(docx, o, source) {
  const view = JSON.parse(wasm.readView(docx, JSON.stringify({
    trackChanges: o.track_changes ?? undefined, comments: o.comments, dates: o.dates,
    pageMarkers: !o.no_page_markers, paragraphs: o.paragraphs ?? undefined,
    head: o.head ?? undefined, tail: o.tail ?? undefined,
    changed: o.changed || undefined, by: o.by ?? undefined, source,
  })));
  for (const warning of view.warnings) console.error(`warning: ${warning}`);
  process.stdout.write(view.markdown);
}

function resolution(accept) {
  return {
    run(name, [file], o) {
      const filter = {};
      for (const key of ["ids", "authors", "kinds"]) if (o[key]?.length) filter[key] = o[key];
      const docx = read(file);
      ensureWritable(o.output, o.force);
      const out = (accept ? wasm.acceptChanges : wasm.rejectChanges)(docx, JSON.stringify(filter));
      write(o.output, out);
      console.log(`wrote ${o.output} (${out.length} bytes)`);
    },
  };
}

function paletteOf(o) {
  return o.revision_palette;
}

function stem(file) {
  return path.basename(file, path.extname(file));
}

function defaultAuthor() {
  try {
    return execFileSync("git", ["config", "user.name"], { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] }).trim() || "Redline";
  } catch {
    return "Redline";
  }
}

// -- shared clap parser and host capabilities --------------------------------

function validateHost(name, o) {
  const reject = (flag) => { throw new UsageError(`--${flag.replaceAll("_", "-")} is not supported by the npm CLI`); };
  if (["compare", "diff"].includes(name)) {
    if ((o.mode ?? "word") !== "word" || o.powertools_faithful) reject("mode powertools");
    if (o.detail_threshold != null) reject("detail_threshold");
    if (o.no_paragraph_merge) reject("no_paragraph_merge");
  }
  if (name === "inspect" && o.tables) reject("tables");
  if (["convert", "diff"].includes(name)) {
    for (const flag of [...(name === "convert" ? ["from"] : []), "resource_path", "timeout", "fail_on_substitution", "no_page_markers"]) {
      if (o[flag] != null && o[flag] !== false) reject(flag);
    }
    if (o.png || o.to === "png" || path.extname(o.output ?? "").toLowerCase() === ".png") {
      throw new UsageError("PNG pages need the Python or Rust build (uvx jubarte-redlines convert --png)");
    }
  }
  if (name === "convert") {
    for (const flag of ["dpi", "pages", "report", "font_report"]) {
      if (flag === "dpi" ? o.dpi !== 96 : o[flag] != null) reject(flag);
    }
    const extension = path.extname(o.output ?? "").toLowerCase();
    if (o.to === "md" || (o.to == null && [".md", ".markdown", ".txt", ".mdown", ".mkd", ".mkdn"].includes(extension))) {
      throw new UsageError("Markdown output with page markers is not supported by the npm CLI; use read --track-changes");
    }
    // A Word name fails before any read; a name that says nothing is
    // sniffed like every other input, in `convert`.
    if (convertsToDocx(o) && namedKind(o.file) === "docx") throw docxNeedsMarkdown();
  }
  if (name === "edit") {
    for (const flag of ["pdf", "png"]) if (o[flag]) reject(flag);
    if (o.dpi !== 96) reject("dpi");
    if (o.revisions !== "conventional" || o.revision_palette != null) reject("revisions");
  }
  if (name === "diff") {
    for (const flag of ["reference_doc", "critic"]) if (o[flag] != null && o[flag] !== false) reject(flag);
    if (!Number.isInteger(o.context) || o.context < 0 || o.context > 0xffffffff) {
      throw new UsageError("--context must be in the u32 range (0..4294967295)");
    }
  }
}

function main(argv) {
  try {
    const parsed = JSON.parse(wasm.parseCli(JSON.stringify(argv), PROG, JSON.stringify(Object.keys(COMMANDS))));
    if ("text" in parsed) {
      (parsed.stream === "stderr" ? process.stderr : process.stdout).write(parsed.text);
      return parsed.exit_code;
    }
    const name = parsed.command, o = parsed.args;
    validateHost(name, o);
    const positionals = name === "compare" ? [o.original, o.modified] : name === "diff" ? [o.old, o.new] : o.file == null ? [] : [o.file];
    return COMMANDS[name].run(name, positionals, o) ?? EXIT_OK;
  } catch (e) {
    console.error(`error: ${e instanceof Error ? e.message : String(e)}`);
    return e instanceof UsageError ? EXIT_USAGE : EXIT_ERROR;
  }
}

process.exitCode = main(process.argv.slice(2));
