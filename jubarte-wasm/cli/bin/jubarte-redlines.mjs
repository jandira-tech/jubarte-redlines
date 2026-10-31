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
// Exit codes: 0 success, 1 error (I/O, engine), 2 usage (a flag, a
// combination, an existing output), 3 edit plan refused (its per-operation
// report is on stdout; nothing was written).

import { randomUUID } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

const require = createRequire(import.meta.url);
const wasm = require("jubarte-wasm");

const PROG = "jubarte-redlines";
const EXIT_OK = 0;
const EXIT_ERROR = 1;
const EXIT_USAGE = 2;
const EXIT_PLAN_REFUSED = 3;

/** A user-facing failure; printed as `error: ...`, exit 1. */
class CliError extends Error {}

/** A usage refusal: a flag, a combination or an existing output; exit 2. */
class UsageError extends Error {}

/** An engine refusal that exits 3, as `read --for-edit`'s. */
class RefusalError extends Error {}

// strerror's text for the errors a file task meets, so an I/O error reads
// as the binary's: `No such file or directory (os error 2)`.
const STRERROR = {
  ENOENT: "No such file or directory", EACCES: "Permission denied", EPERM: "Operation not permitted",
  EISDIR: "Is a directory", ENOTDIR: "Not a directory", EEXIST: "File exists", ENOSPC: "No space left on device",
  EROFS: "Read-only file system", ENAMETOOLONG: "File name too long", ELOOP: "Too many levels of symbolic links",
};

/** An I/O error as Rust words it; Node's own message where it has no errno. */
function osError(e) {
  const text = STRERROR[e?.code];
  return text && Number.isInteger(e.errno) && process.platform !== "win32" ? `${text} (os error ${-e.errno})` : e.message;
}

/** An error's message without the engine's `jubarte-wasm: ` prefix. */
const messageOf = (e) => String(e instanceof Error ? e.message : e).replace(/^jubarte-wasm: /, "");

function read(file, forceKind) {
  let bytes;
  try {
    bytes = fs.readFileSync(file);
  } catch (e) {
    throw new CliError(`reading ${file}: ${osError(e)}`);
  }
  // Ask the same clap parser for declared input metadata on other commands.
  // Its format grammar stays in the core; unknown suffixes use ZIP sniffing.
  if (forceKind === undefined) {
    const parsed = JSON.parse(wasm.parseCli(JSON.stringify(["diff", "--", file, file])));
    forceKind = parsed.args.old_format;
  }
  const word = (forceKind ?? (bytes.subarray(0, 4).equals(ZIP_MAGIC) ? "docx" : "md")) === "docx";
  // The binary's line for what no task reads: a .doc or RTF file, or Word
  // that is no ZIP package (`CODE: FILE is …`).
  const refused = wasm.refusedInput(file, bytes, word);
  if (refused) throw new CliError(refused);
  if (!word && bytes.subarray(0, 3).equals(Buffer.from([0xef, 0xbb, 0xbf]))) {
    return bytes.subarray(3);
  }
  return bytes;
}

/** JSON snapshots embed validity (#1309); JSON Lines send a quietable
 * JSON status to stderr (#1925). Older bindings keep the text notice. */
const jsonValidity = typeof wasm.inputValidityLine === "function";

/** `read`'s bytes for `read` and `changes`; unless `-q`, one stderr line
 * says how many validity findings the input has, as the binary's does,
 * but for a JSON read (`json`), which carries them. (A binding older than
 * #1256 has no inputNotice and says nothing.) */
function readInput(file, o, json = false) {
  const docx = read(file);
  if (!o.quiet && !(json && jsonValidity) && wasm.inputNotice) {
    const notice = wasm.inputNotice(docx, file);
    if (notice) console.error(notice);
  }
  return docx;
}

/** The no-clobber refusal, as the binary words and codes it: a usage
 * refusal (exit 2) with the OUTPUT_EXISTS code. */
function ensureWritable(file, force) {
  if (fs.existsSync(file) && !force) {
    throw new UsageError(`OUTPUT_EXISTS: output '${file}' already exists (use --force to overwrite)`);
  }
}

/** Whether two paths name the same file: the same device and inode when
 * both exist (a link to the input is the input), else the same path once
 * resolved (`out.md` and `./out.md` are one file). A file system that
 * reports no inode (0) compares by path. */
function sameFile(a, b) {
  try {
    const left = fs.statSync(a, { bigint: true });
    const right = fs.statSync(b, { bigint: true });
    if (left.ino !== 0n && right.ino !== 0n) return left.dev === right.dev && left.ino === right.ino;
  } catch {}
  return path.resolve(a) === path.resolve(b);
}

/** The refusal of an output that names its own input, as the binary words
 * and codes it: a usage refusal (exit 2) with the OUTPUT_ALIAS code.
 * --force authorizes replacing an output, never an input. */
function refuseOutputAlias(output, input) {
  if (sameFile(output, input)) {
    throw new UsageError(`OUTPUT_ALIAS: the output '${output}' is the same file as the input '${input}'`);
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

/** Markdown bytes as text; malformed UTF-8 is refused, never replaced. */
function markdownText(file, bytes) {
  try {
    return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  } catch {
    throw new CliError(`${file}: Markdown must be UTF-8`);
  }
}

/** Writes `data` to `file` through a staged file beside it, renamed over
 * it: a link is replaced, never written through, and an input that shares
 * the output's file keeps its bytes. A replaced regular file keeps its
 * mode, as the binary's does. The staged name is random and created only
 * if absent, so a link planted beside the output is never followed
 * (#1698). */
function write(file, data) {
  const partial = path.join(path.dirname(file), `.${path.basename(file)}.${randomUUID()}.partial`);
  let created = false;
  try {
    let mode;
    try {
      const old = fs.lstatSync(file);
      if (old.isFile()) mode = old.mode & 0o777;
    } catch {}
    const fd = fs.openSync(partial, "wx");
    created = true;
    try {
      fs.writeFileSync(fd, data);
    } finally {
      fs.closeSync(fd);
    }
    if (mode !== undefined) fs.chmodSync(partial, mode);
    fs.renameSync(partial, file);
  } catch (e) {
    if (created) {
      try {
        fs.rmSync(partial, { force: true });
      } catch {}
    }
    throw new CliError(`writing ${file}: ${osError(e)}`);
  }
}

/** `--in-place` follows a symlink: the link stays, and the target keeps its
 * mode. `write` on any other path still replaces the directory entry. */
function writeThrough(file, data) {
  let target = file;
  try {
    if (fs.lstatSync(file).isSymbolicLink()) target = fs.realpathSync(file);
  } catch (e) {
    if (e.code !== "ENOENT") throw new CliError(`resolving ${file}: ${osError(e)}`);
  }
  write(target, data);
}

/** `wrote FILE in place (replaced sha256 <8 hex>…)`, plus the markup detail. */
function inPlaceLine(file, bytes, detail) {
  const prefix = wasm.sourceSha256(bytes).slice(0, 8);
  const extra = detail ? `; ${detail}` : "";
  return `wrote ${file} in place (replaced sha256 ${prefix}\u2026)${extra}`;
}

/** The `; 1 change by "GiantCo"` fragment. Null when the write added no markup. */
function markupDetail(result, o) {
  let report;
  try {
    report = JSON.parse(result.json);
  } catch {
    return null;
  }
  const editing = Boolean(o.editing_mode);
  const changes = editing ? 0 : (report.revisions?.total ?? 0);
  const comments = editing ? 0 : (report.comments_added ?? 0);
  if (!changes && !comments) return null;
  const parts = [];
  if (changes) parts.push(`${changes} change${changes === 1 ? "" : "s"}`);
  if (comments) parts.push(`${comments} comment${comments === 1 ? "" : "s"}`);
  let detail = `${parts.join(", ")} by "${report.author}"`;
  if (!o.author_given) detail += ": no -a given";
  return detail;
}

/** A value with every object's keys sorted, as the binary's JSON prints them. */
function sortedKeys(value) {
  if (Array.isArray(value)) return value.map(sortedKeys);
  if (value === null || typeof value !== "object") return value;
  return Object.fromEntries(Object.keys(value).sort().map((key) => [key, sortedKeys(value[key])]));
}

/** One JSON line as the binary prints it: compact, keys sorted. */
const jsonl = (row) => JSON.stringify(sortedKeys(row));

/** The engine's `wrote` line on stderr, unless `-q`. */
function wrote(o, file, format, size, pages) {
  if (!o.quiet) console.error(wasm.wroteLine(file, format, size, pages));
}

/** `readJson`'s `validityFile` for FILE as typed, when the binding takes it. */
function validity(file) {
  return file != null && jsonValidity ? { validityFile: file } : {};
}

/** The read options that select paragraphs, and the text --track-changes
 * settles, as `readJson` takes them. */
function selection(o = {}) {
  return { trackChanges: o.track_changes ?? undefined, paragraphs: o.paragraphs ?? undefined, head: o.head ?? undefined, tail: o.tail ?? undefined, changed: Boolean(o.changed), by: o.by ?? undefined };
}

/** `read --for-edit`'s working-document JSON, from the flags the shared
 * parser stored; `validityFile` is FILE as typed. Usage failures are
 * `jubarte-wasm: USAGE: …`, refusals `jubarte-wasm: CODE: …`. */
function forEditJson(docx, o, validityFile) {
  try {
    return wasm.forEdit(docx, JSON.stringify({
      paragraphs: o.paragraphs ?? undefined, head: o.head ?? undefined, tail: o.tail ?? undefined,
      find: o.find ?? undefined, regex: o.regex ?? undefined, ignoreCase: Boolean(o.ignore_case),
      changed: Boolean(o.changed), by: o.by ?? undefined,
      context: o.context ?? undefined, limit: o.limit ?? undefined, maxChars: o.max_chars ?? undefined,
      all: Boolean(o.all), cursor: o.cursor ?? undefined,
      acceptChange: o.accept_change?.length ? o.accept_change : undefined,
      rejectChange: o.reject_change?.length ? o.reject_change : undefined,
      existingRevisions: o.existing_revisions ?? undefined, track: o.track ?? undefined,
      validityFile,
    }));
  } catch (e) {
    const usage = messageOf(e);
    if (usage.startsWith("USAGE: ")) throw new UsageError(usage.slice("USAGE: ".length));
    throw new RefusalError(usage);
  }
}

/** `-o` in the format its extension names, as the binary's writer: the Word
 * package, its agent view (.md, headed `source`), its snapshot (.json) or
 * its PDF. `view` is read's options; the defaults elsewhere. `validityFile`
 * puts the input's validity in a snapshot's summary, as `read --json`
 * prints it. */
function writeResult(o, docx, source, view, validityFile) {
  const output = o.output, kind = o.output_format ?? "docx";
  if (kind === "pdf") return renderTo(o, docx, output);
  ensureWritable(output, o.force);
  const data = kind === "md" ? viewText(docx, view ?? {}, source)
    : kind === "json" && view?.for_edit ? forEditJson(docx, view, validityFile)
    : kind === "json" ? `${wasm.readJson(docx, JSON.stringify({ ...selection(view), ...validity(validityFile) }))}\n`
    : docx;
  write(output, data);
  wrote(o, output, kind, typeof data === "string" ? Buffer.byteLength(data, "utf8") : data.length);
}

/** `docx` drawn as the PDF `output`, with the task's render flags. */
function renderTo(o, docx, output) {
  ensureWritable(output, o.force);
  const pdf = wasm.docxToPdf(docx, Boolean(o.compress), o.revisions ?? undefined, paletteOf(o), Boolean(o.move_comments), Boolean(o.changed_pages));
  write(output, pdf);
  wrote(o, output, "pdf", pdf.length, wasm.pdfPageCount(pdf));
}

/** The name a task's .md view gives the Word file it did not write: -o as
 * typed, with .docx for its extension. */
const notWritten = (output) => `${output.slice(0, output.length - path.extname(output).length)}.docx (not written)`;

// -- commands -----------------------------------------------------------------

const COMMANDS = {
  compare: {
    run(name, [original, modified], o) {
      const fallback = path.join(path.dirname(original), `${stem(original)}_v_${stem(modified)}.docx`);
      const [a, b] = [read(original, o.old_format), read(modified, o.new_format)];
      // Without -o, compare (and the shorthand `A B`) prints the redline's
      // agent view.
      if (o.output == null) {
        return void printView(wasm.redlineDocuments(a, b, o.author, o.datetime), o.view, `${fallback} (not written; -o keeps it)`);
      }
      const output = o.output;
      // A .md of two Markdown documents is CriticMarkup; of anything else,
      // the redline's view.
      if (o.output_format === "md" && kindOf(original, a, o.old_format) === "md" && kindOf(modified, b, o.new_format) === "md") {
        if (o.view_given) throw new UsageError("read options shape the view; two Markdown documents write CriticMarkup to a .md");
        ensureWritable(output, o.force);
        const critic = wasm.diffDocumentsCritic(a, b, o.author, o.datetime);
        write(output, critic);
        return void wrote(o, output, "md", Buffer.byteLength(critic, "utf8"));
      }
      ensureWritable(output, o.force);
      writeResult(o, wasm.redlineDocuments(a, b, o.author, o.datetime), `${fallback} (not written; -o keeps it)`, o.view);
    },
  },
  diff: {
    // Text only, as natively: the patch, a text view, or (two Markdown
    // documents) CriticMarkup, on stdout or in -o's file.
    run(_, [old, next], o) {
      // An existing output is refused before any work.
      if (o.output != null) ensureWritable(o.output, o.force);
      const a = read(old, o.old_format), b = read(next, o.new_format);
      const kind = (declared, bytes) => declared ?? (bytes.subarray(0, 4).equals(ZIP_MAGIC) ? "docx" : "md");
      const oldFormat = kind(o.old_format, a), newFormat = kind(o.new_format, b);
      // A side read as Markdown must be text, whichever engine call follows.
      if (oldFormat === "md") markdownText(old, a);
      if (newFormat === "md") markdownText(next, b);
      let text;
      if (["github", "word", "normal", "context", "side-by-side"].includes(o.format)) {
        text = wasm.diffDocumentsView(a, b, JSON.stringify({
          format: o.format, oldName: old, newName: next, context: o.context,
          acceptChanges: o.accept_changes, fullLines: o.full_lines, oldFormat, newFormat,
        }));
      } else {
        const date = o.datetime; // the shared parser dated the run (D17)
        if (o.format === "critic") {
          if (oldFormat !== "md" || newFormat !== "md") {
            throw new CliError("--format critic needs both documents in Markdown; `jubarte compare OLD NEW` prints their redline as the agent view");
          }
          text = wasm.diffDocumentsCritic(a, b, o.author, date);
        } else text = JSON.parse(wasm.diffDocuments(a, b, o.author, date, o.columns, old, next)).text;
      }
      if (o.output == null) return void process.stdout.write(text);
      write(o.output, text);
      if (!o.quiet) console.error(`wrote ${o.output} (md, ${Buffer.byteLength(text, "utf8")} bytes)`);
    },
  },
  changes: {
    run(_, [file], o) {
      const docx = readInput(file, o, o.json);
      // --author: only those authors' changes, exact names (#1690).
      const authors = o.author?.length ? o.author : undefined;
      if (!o.json) return void process.stdout.write(wasm.changesText(docx, authors));
      for (const change of JSON.parse(wasm.listChanges(docx, authors))) {
        console.log(JSON.stringify(Object.fromEntries(Object.entries(change).filter(([, v]) => v !== null))));
      }
      if (jsonValidity && !o.quiet) console.error(wasm.inputValidityLine(docx, file));
    },
  },
  accept: resolution(true),
  reject: resolution(false),
  read: {
    run(_, [file], o) {
      // The output is refused before the input is read, as the binary does:
      // an existing output is a usage refusal, not an error the input's
      // reading would have found first (#1690).
      if (o.output != null && !o.for_edit) {
        // An output that names the input would destroy it: the tables view
        // of a document written over itself.
        if (o.tables) refuseOutputAlias(o.output, file);
        ensureWritable(o.output, o.force);
      }
      const docx = readInput(file, o, (o.json || o.for_edit) && o.output == null);
      if (o.for_edit) {
        if (o.output != null) return void writeResult(o, docx, path.basename(file), o, file);
        return void process.stdout.write(forEditJson(docx, o, file));
      }
      if (o.tables) {
        // -o FILE writes the grids it would print (#1690).
        const emit = (text) => {
          if (o.output == null) return void process.stdout.write(text);
          write(o.output, text);
          if (!o.quiet) console.error(`wrote ${o.output} (tables, ${Buffer.byteLength(text, "utf8")} bytes)`);
        };
        // (A binding older than #1255 has no tablesView; its parser
        // refuses -p and --track-changes here.)
        if (!wasm.tablesView) return void emit(wasm.tablesText(docx));
        // With -p, the grids of the tables it reaches, then its view; a
        // pick that reaches no table is a usage error.
        let view;
        try {
          view = JSON.parse(wasm.tablesView(docx, JSON.stringify({
            trackChanges: o.track_changes ?? undefined, comments: o.comments, dates: o.dates,
            pageMarkers: !o.no_page_markers, paragraphs: o.paragraphs ?? undefined, source: path.basename(file),
          })));
        } catch (e) {
          const usage = /^jubarte-wasm: USAGE: (.*)$/s.exec(String(e?.message ?? e));
          if (usage) throw new UsageError(usage[1]);
          throw e;
        }
        for (const warning of view.warnings) console.error(`warning: ${warning}`);
        return void emit(view.markdown);
      }
      if (o.output != null) return void writeResult(o, docx, path.basename(file), o, file);
      if (o.json) return void console.log(wasm.readJson(docx, JSON.stringify({ ...selection(o), source: path.basename(file), ...validity(file) })));
      printView(docx, o, path.basename(file));
    },
  },
  convert: {
    run(_, [file], o) {
      let docx = read(file);
      if (kindOf(file, docx) === "md") {
        const text = markdownText(file, docx);
        const reference = o.reference_doc == null ? undefined : read(o.reference_doc);
        // What names no one: the flags, else the front matter, else the
        // reference's creator and creation time, else Redline and now.
        let author = o.author;
        let date = o.datetime;
        if (author == null || date == null) {
          const [stampAuthor, stampDate] = JSON.parse(wasm.markdownStamp(text, reference));
          author ??= stampAuthor ?? "Redline";
          date ??= stampDate ?? new Date().toISOString().replace(/\.\d{3}Z$/, "Z");
        }
        docx = wasm.markdownToDocx(text, JSON.stringify({ page: o.page, author, date, critic: !o.no_critic, track_changes: o.track_changes, tracking: o.tracking == null ? undefined : o.tracking === "on", scholar: Boolean(o.scholar) }), reference);
        if (!o.pdf && o.output_format !== "json" && (o.to === "docx" || (o.to == null && (o.output == null || path.extname(o.output).toLowerCase() === ".docx")))) {
          if (o.move_comments || o.changed_pages) {
            throw new CliError("--move-comments / --changed-pages applies to PDF or PNG output only");
          }
          const output = o.output ?? path.join(path.dirname(file), `${stem(file)}.docx`);
          ensureWritable(output, o.force);
          write(output, docx);
          return void wrote(o, output, "docx", docx.length);
        }
      } else if (convertsToDocx(o)) {
        return void wordToWord(file, docx, o);
      } else if (o.to === "md" || (o.to == null && o.output_format === "md")) {
        return void wordToMarkdown(docx, o);
      } else if (o.track_changes === "accept") {
        docx = wasm.acceptRevisions(docx);
      } else if (o.track_changes === "reject") {
        docx = wasm.rejectRevisions(docx);
      }
      if (o.output_format === "json") return void writeResult(o, docx, path.basename(file));
      renderTo(o, docx, o.output ?? path.join(path.dirname(file), `${stem(file)}.pdf`));
    },
  },
  edit: { run: (name, [file], o) => runEdit("edit", file, o) },
  add: { run: (name, [file], o) => runEdit("add", file, o) },
  capabilities: {
    run(_, __, o) {
      if (o.matrix || o.operation != null || o.field != null) {
        if (o.json) return console.log(JSON.stringify(JSON.parse(wasm.capabilityQuery(o.operation, o.field)), null, 2));
        return process.stdout.write(wasm.capabilityQueryText(o.operation, o.field));
      }
      const manifest = cliManifest();
      if (o.json) return console.log(JSON.stringify(manifest, null, 2));
      process.stdout.write(wasm.capabilitiesText(JSON.stringify(manifest)));
    },
  },
};

/** The engine's manifest narrowed to this CLI: the tasks it has, and no
 * operation it refuses (PNG, page ranges, append, scrub, validate, repair,
 * audit, comments, visual diffs). */
function cliManifest() {
  const manifest = JSON.parse(wasm.capabilities());
  for (const op of ["append", "scrub", "validate", "repair", "audit", "comments", "diff_render", "page_ranges", "png"]) {
    manifest.operations[op] = false;
  }
  for (const task of Object.keys(manifest.formats)) if (!(task in COMMANDS)) delete manifest.formats[task];
  return manifest;
}

/** `convert` of Word to Word, as the binary: the changes accepted or
 * rejected, the field results refreshed, or both. */
function wordToWord(file, docx, o) {
  const resolve = o.track_changes === "accept" ? wasm.acceptRevisions : o.track_changes === "reject" ? wasm.rejectRevisions : null;
  if (!resolve && !o.update_fields) {
    throw new CliError(`${file} is already Word: give --track-changes accept or reject, --update-fields, or another --to`);
  }
  if (o.output == null) throw new CliError("--output is required to write Word from Word");
  if (o.report != null) ensureWritable(o.report, o.force);
  ensureWritable(o.output, o.force);
  let out = resolve ? resolve(docx) : docx;
  if (o.update_fields) {
    const updated = wasm.updateFields(out);
    const report = JSON.parse(updated.json);
    for (const field of report.fields) {
      console.log(`${field.paragraph}\t${field.kind}\t${JSON.stringify(field.old)} -> ${JSON.stringify(field.new)}`);
    }
    console.error(`${report.fields.length} field(s) written; ${report.page_count} page(s)`);
    if (o.report != null) write(o.report, updated.json);
    out = updated.docx;
  }
  write(o.output, out);
  wrote(o, o.output, "docx", out.length);
}

/** `convert` of Word to Markdown, as the binary: the paged view (page
 * markers unless --no-page-markers), on stdout without -o. */
function wordToMarkdown(docx, o) {
  const { markdown, warning } = JSON.parse(wasm.convertMarkdown(docx, o.track_changes ?? "all", !o.no_page_markers, o.revisions ?? undefined, paletteOf(o)));
  if (warning) console.error(`warning: ${warning}`);
  if (o.output == null) return void process.stdout.write(markdown);
  ensureWritable(o.output, o.force);
  write(o.output, markdown);
  wrote(o, o.output, "md", Buffer.byteLength(markdown, "utf8"));
}

/** An edited document's PDF and its page facts, as `edit --pdf` draws it. */
function editPdf(docx, o) {
  const rendered = wasm.renderPdf(docx, o.revisions ?? undefined, paletteOf(o));
  return { pdf: rendered.pdf, ...JSON.parse(rendered.json) };
}

/** PLAN.json's text, strictly UTF-8 as the binary reads it; `undefined`
 * when there is no such file and `missing` allows it. */
function readPlanText(plan, missing = false) {
  let bytes;
  try {
    bytes = fs.readFileSync(plan);
  } catch (e) {
    if (missing && e.code === "ENOENT") return undefined;
    throw new CliError(`reading ${plan}: ${osError(e)}`);
  }
  try {
    return new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(bytes);
  } catch {
    throw new CliError(`reading ${plan}: stream did not contain valid UTF-8`);
  }
}

/** The guard is on: `--keep-others`, or exactly `JUBARTE_KEEP_OTHERS=1`.
 * Read here, in the CLI, and nowhere in the library. */
function keepOthers(o) {
  return Boolean(o.keep_others) || process.env.JUBARTE_KEEP_OTHERS === "1";
}

/** `--plan`'s text, or the plan the operation flags describe, and its
 * notes. `--rewrite-others-comments` still sets its field (a no-op).
 * `--keep-others` and `JUBARTE_KEEP_OTHERS=1` set `"others": "keep"`. */
function editPlan(verb, docx, o) {
  const given = editPlanAsGiven(verb, docx, o);
  const rewrite = Boolean(o.rewrite_others_comments);
  const keep = keepOthers(o);
  if (!rewrite && !keep) return given;
  let parsed;
  try {
    parsed = JSON.parse(given.plan);
  } catch {
    return given; // reported as INVALID_PLAN when applied
  }
  if (parsed === null || typeof parsed !== "object" || Array.isArray(parsed)) return given;
  if (rewrite) parsed.rewrite_others_comments = true;
  if (keep) parsed.others = "keep";
  return { plan: JSON.stringify(parsed), notes: given.notes };
}

function editPlanAsGiven(verb, docx, o) {
  if (o.plan != null) return { plan: readPlanText(o.plan), notes: [] };
  try {
    const options = { author: o.author, date: o.datetime ?? undefined, existingRevisions: o.existing_revisions };
    return JSON.parse(wasm.flagPlan(verb, JSON.stringify(o.operations ?? []), docx, JSON.stringify(options)));
  } catch (e) {
    throw new UsageError(String(e?.message ?? e).replace(/^jubarte-wasm: /, ""));
  }
}

/** Whether the plan keeps the source's revisions; a plan that is not a JSON
 * object answers no, and the engine then reports it as INVALID_PLAN. */
/** A refused plan's per-operation report on stdout, the error on stderr. */
function printRefusal(result) {
  const error = JSON.parse(result.json);
  (error.outcomes ?? []).forEach((outcome, i) => {
    const row = { ev: "op", i: i + 1, id: outcome.id, op: outcome.kind, status: outcome.status, matches: outcome.matches };
    for (const [key, field] of [["at", "paragraph"], ["ctx", "context"], ["code", "code"], ["message", "message"]]) {
      if (outcome[field] != null) row[key] = outcome[field];
    }
    console.log(jsonl(row));
  });
  console.log(jsonl({ ev: "summary", status: "failed", code: error.code, operation: error.operation ?? null, message: error.message }));
  console.error(`error: plan refused: ${error.code}${error.operation ? ` (${error.operation})` : ""}: ${error.message}`);
  return EXIT_PLAN_REFUSED;
}

/** `--plan PLAN.json` with operation flags: record them into the plan,
 * checked against FILE first; a refusal leaves PLAN.json as it was. The
 * shared parser has refused the flags that do not go with `--plan`. */
function recordPlan(verb, file, docx, o) {
  const held = readPlanText(o.plan, true);
  const options = {
    author: o.author, date: o.datetime ?? undefined, existingRevisions: o.existing_revisions,
    file, plan: o.plan, defaultPlan: Boolean(o.plan_default), dryRun: Boolean(o.dry_run),
    rewriteOthersComments: Boolean(o.rewrite_others_comments),
  };
  // Only when set: an engine that predates the field rejects an unknown key.
  if (keepOthers(o)) options.keepOthers = true;
  const answer = JSON.parse(wasm.recordPlan(verb, JSON.stringify(o.operations), docx, JSON.stringify(options), held));
  if (!answer.ok) {
    if (answer.usage) throw new UsageError(answer.message);
    console.error(`error: ${o.plan}: ${answer.message}`);
    return EXIT_PLAN_REFUSED;
  }
  const { recorded } = answer;
  const preview = wasm.previewEditPlan(docx, recorded.plan);
  if (!preview.ok) return printRefusal(preview);
  if (!o.quiet) for (const note of recorded.notes) console.log(`note: ${note}`);
  if (!o.dry_run) {
    // Through a symbolic link to the file it names, written yet or not; via a hidden partial
    // file named for this process, removed if the write fails.
    let target = o.plan;
    try {
      target = fs.realpathSync(o.plan);
    } catch {
      // A link to a plan not written yet names where it goes.
      try {
        target = path.resolve(path.dirname(o.plan), fs.readlinkSync(o.plan));
      } catch {}
    }
    const partial = path.join(path.dirname(target), `.${path.basename(target)}.${process.pid}.partial`);
    try {
      fs.writeFileSync(partial, recorded.json);
      fs.renameSync(partial, target);
    } catch (e) {
      fs.rmSync(partial, { force: true });
      throw new CliError(`writing ${o.plan}: ${osError(e)}`);
    }
  }
  if (!o.quiet) console.log(recorded.summary);
  return EXIT_OK;
}

// lstat refuses links even where O_NOFOLLOW is unavailable; compare the
// opened file with the name using bigint identities to avoid rounding inos.
function planIdentityMatches(fd, name) {
  const opened = fs.fstatSync(fd, { bigint: true });
  const named = fs.lstatSync(name, { bigint: true });
  if (named.isSymbolicLink()) throw Object.assign(new Error("symbolic link"), { code: "ELOOP" });
  return (opened.ino === 0n && named.ino === 0n) || (opened.dev === named.dev && opened.ino === named.ino);
}

function runEdit(verb, file, o) {
  const docx = read(file);
  if (o.plan != null && o.operations?.length) return recordPlan(verb, file, docx, o);
  // What --consume deletes once the edit is written: the plan file as it
  // was read, never a link to it or an output written over it. Open once
  // and read the handle, so `plan` names this file — not another placed on
  // the name between a check and the read; O_NOFOLLOW makes the open itself
  // refuse a symbolic link.
  let planFd, consumed;
  if (o.consume && !o.dry_run) {
    try {
      planFd = fs.openSync(o.plan, (fs.constants.O_NOFOLLOW ?? 0) | fs.constants.O_RDONLY);
      if (!planIdentityMatches(planFd, o.plan)) {
        throw new Error("the plan name changed while it was being read");
      }
      consumed = fs.readFileSync(planFd);
    } catch (e) {
      if (planFd !== undefined) try { fs.closeSync(planFd); } catch {}
      if (e?.code === "ELOOP") {
        throw new UsageError(`--consume deletes ${o.plan}, a symbolic link; pass the file it names`);
      }
      throw new CliError(`reading ${o.plan}: ${osError(e)}`);
    }
  }
  try {
    const code = applyEdit(verb, file, docx, o);
    if ((code ?? EXIT_OK) === EXIT_OK && consumed) {
      const kept = (why) => new CliError(`the plan was applied, but ${o.plan} was not deleted: ${why}`);
      let unchanged = false;
      try {
        // The original read advanced the fd: reread positionally, including
        // one extra byte so an append cannot compare equal to the plan.
        const current = Buffer.alloc(consumed.length + 1);
        let length = 0;
        while (length < current.length) {
          const read = fs.readSync(planFd, current, length, current.length - length, length);
          if (read === 0) break;
          length += read;
        }
        unchanged = current.subarray(0, length).equals(consumed) && planIdentityMatches(planFd, o.plan);
      } catch {}
      if (!unchanged) throw kept("it no longer holds the plan (an output was written over it?)");
      // Unavoidable race window between the last lstat and unlinkSync:
      // Node has no unlink-by-handle API.
      try {
        fs.unlinkSync(o.plan);
      } catch (e) {
        throw kept(osError(e));
      }
    }
    return code;
  } finally {
    if (planFd !== undefined) fs.closeSync(planFd);
  }
}

function applyEdit(verb, file, docx, o) {
  const { plan, notes } = editPlan(verb, docx, o);
  // A plan that does not read is one line, as the binary prints it; no
  // report, and nothing is written.
  const invalid = o.plan != null ? wasm.planError(plan) : undefined;
  if (invalid) {
    console.error(`error: ${invalid}`);
    return EXIT_PLAN_REFUSED;
  }
  const editing = Boolean(o.editing_mode);
  // Changes the plan itself accepts or rejects (-p r12 --accept) do not count.
  const conflict = editing ? wasm.editingModeConflict(plan, docx) : undefined;
  if (conflict) throw new UsageError(conflict);
  // The plan first: a refused plan prints its whole report, whatever the
  // outputs (D15).
  const result = o.dry_run ? wasm.previewEditPlan(docx, plan) : wasm.applyEditPlan(docx, plan, path.basename(file));
  if (!result.ok) return printRefusal(result);
  const report = wasm.editReportJsonl(result.json);
  if (o.dry_run) return void process.stdout.write(report);
  // The shared parser resolved the target: `output` for the one document
  // (FILE_modified.docx by default), `out_dir` for the bundle.
  const outDir = o.out_dir;
  if (outDir == null) return writeEdited(o.output, result, notes, o, docx, plan);
  if (fs.existsSync(outDir) && !o.force) {
    throw new UsageError(`OUTPUT_EXISTS: output '${outDir}' already exists (use --force to overwrite)`);
  }
  if (path.resolve(outDir) === path.dirname(path.resolve(file))) {
    throw new CliError("--out-dir must not be the input's own directory");
  }
  const lines = report.trimEnd().split("\n");
  const summary = lines.pop();
  // Render before creating the directory, so a failed render leaves no
  // partial bundle; the `render` line goes before the summary.
  const renders = o.pdf ? (editing ? [["clean", result.clean]] : [["redline", result.redline], ["clean", result.clean]]).map(([name, doc]) => [name, editPdf(doc, o)]) : [];
  if (renders.length) {
    lines.push(jsonl({
      ev: "render", engine: `jubarte ${JSON.parse(wasm.capabilities()).engine_version}`,
      pages: Object.fromEntries(renders.map(([name, r]) => [name, r.page_count])),
      page_starts: Object.fromEntries(renders.map(([name, r]) => [name, r.page_starts])),
    }));
  }
  const outputs = [["clean.docx", result.clean]];
  if (!editing) {
    outputs.push(["redline.docx", result.redline]);
    // jubarte-wasm 0.10.1 has no patch; later builds carry it.
    if (typeof result.patch === "string") outputs.push(["patch.diff", Buffer.from(result.patch, "utf8")]);
  }
  for (const [name, r] of renders) outputs.push([`${name}.pdf`, Buffer.from(r.pdf)]);
  fs.mkdirSync(outDir, { recursive: true });
  const saved = outputs.map(([name, data]) => {
    write(path.join(outDir, name), data);
    return { f: name, bytes: data.length, sha256: wasm.sourceSha256(data) };
  });
  lines.push(jsonl({ ev: "save", dir: outDir, outputs: saved }), summary);
  write(path.join(outDir, "report.jsonl"), `${lines.join("\n")}\n`);
  if (o.quiet) return;
  // A pure accept/reject prints the resolved lines on stdout, nothing
  // else; the bundle's report.jsonl still holds the summary.
  if (isPureResolution(plan, result.json)) {
    try {
      const resolved = JSON.parse(result.json).resolved_revisions ?? {};
      const text = wasm.resolutionText(docx, JSON.stringify({ accepted: resolved.accepted ?? [], rejected: resolved.rejected ?? [] }));
      const names = [...outputs.map(([n]) => n), "report.jsonl"];
      console.error(`wrote ${outDir} (${names.length} files: ${names.join(", ")})`);
      for (const note of editNotes(notes, result)) console.error(note);
      process.stdout.write(text);
      return;
    } catch {}
  }
  console.log(summary);
  const names = [...outputs.map(([n]) => n), "report.jsonl"];
  // The status lines on stderr (D6); the summary and the view are data.
  console.error(`wrote ${outDir} (${names.length} files: ${names.join(", ")})`);
  for (const note of editNotes(notes, result)) console.error(note);
  printResolutionOrView(docx, plan, result, editing, path.join(outDir, editing ? "clean.docx" : "redline.docx"));
}

/** Whether the plan resolved only revisions with some resolved: a pure
 * `--accept`/`--reject` prints the resolved lines, not the view. */
function isPureResolution(planJson, reportJson) {
  try {
    const plan = JSON.parse(planJson);
    if (!Array.isArray(plan.operations) || plan.operations.length !== 0) return false;
    if (plan.resolve_revisions == null) return false;
    const resolved = JSON.parse(reportJson).resolved_revisions ?? {};
    return Boolean((resolved.accepted?.length ?? 0) + (resolved.rejected?.length ?? 0));
  } catch {
    return false;
  }
}

/** The resolved lines for a pure accept/reject, or the changed view
 * otherwise. Failures fall back to the view, as the binary does. */
function printResolutionOrView(docx, planJson, result, editing, shown) {
  if (isPureResolution(planJson, result.json)) {
    try {
      const resolved = JSON.parse(result.json).resolved_revisions ?? {};
      const text = wasm.resolutionText(docx, JSON.stringify({ accepted: resolved.accepted ?? [], rejected: resolved.rejected ?? [] }));
      process.stdout.write(text);
      return;
    } catch {}
  }
  printChangedView(result, editing, shown);
}

/** The `note:` lines of an applied edit: the flags' notes, the plan's (a field
 * read as another), then each anchor read without its Markdown marks. */
function editNotes(notes, result) {
  const report = JSON.parse(result.json);
  const lines = [...notes, ...(report.notes ?? [])].map((note) => `note: ${note}`);
  for (const outcome of report.operations ?? []) {
    if (outcome.anchor_given != null && outcome.anchor_read_as != null) {
      lines.push(`note: ${outcome.id}: anchor ${JSON.stringify(outcome.anchor_given)} read as ${JSON.stringify(outcome.anchor_read_as)} (Markdown marks are not document text)`);
    }
  }
  return lines;
}

/** The changed blocks' view of the written `shown`. The files are written:
 * a view that cannot be read back is a warning, not a failed edit. */
function printChangedView(result, editing, shown) {
  let view;
  try {
    view = wasm.changedView(result.redline, JSON.parse(result.json).author, editing, shown);
  } catch (error) {
    console.error(`warning: the changed blocks cannot be shown: ${error.message ?? error}`);
    return;
  }
  process.stdout.write(view);
}

/** `edit` and `add` without --out-dir, as the binary: the redline (in
 * editing mode, the result) as one .docx at `output`, the `wrote` line and
 * notes on stderr, the changed blocks' view of that file on stdout (a pure
 * accept/reject prints the resolved lines). */
function writeEdited(output, result, notes, o, docx, plan) {
  const editing = Boolean(o.editing_mode);
  const document = editing ? result.clean : result.redline;
  // FILE itself (X_modified.docx edited again, or -o FILE) exists, so it is
  // replaced only with --force (D15). `--in-place` authorizes the input
  // only; a PDF that already exists still needs `--force`.
  const pdfPath = o.pdf ? `${output.slice(0, output.length - path.extname(output).length)}.pdf` : null;
  const replaces = (taken) => Boolean(o.force) || (Boolean(o.in_place) && taken === output);
  for (const taken of [output, pdfPath]) {
    if (taken != null && fs.existsSync(taken) && !replaces(taken)) {
      throw new UsageError(`OUTPUT_EXISTS: output '${taken}' already exists (use --force to overwrite)`);
    }
  }
  // Render before writing, so a failed render writes nothing.
  const rendered = o.pdf ? editPdf(document, o) : null;
  const outputs = [[output, document]];
  if (rendered) outputs.push([pdfPath, rendered.pdf]);
  // Companions first. A failure there leaves the input byte-identical.
  if (o.in_place && outputs.length > 1) {
    outputs.push(outputs.shift());
  }
  for (const [file, data] of outputs) {
    if (o.in_place && file === output) {
      if (docx != null && !fs.readFileSync(file).equals(docx)) {
        throw new RefusalError(`SOURCE_CHANGED: '${file}' changed since the edit was prepared; nothing was published`);
      }
      writeThrough(file, data);
    } else {
      write(file, data);
    }
  }
  if (o.quiet) return;
  if (o.in_place) {
    const detail = markupDetail(result, o);
    console.error(inPlaceLine(output, docx ?? document, detail));
    if (detail && !o.author_given) {
      console.error(`warning: no -a given; the new markup is signed "${JSON.parse(result.json).author}"`);
    }
  } else {
    console.error(`wrote ${output} (docx, ${document.length} bytes)`);
  }
  if (rendered) console.error(wasm.wroteLine(pdfPath, "pdf", rendered.pdf.length, rendered.page_count));
  for (const note of editNotes(notes, result)) console.error(note);
  if (docx != null && plan != null) {
    printResolutionOrView(docx, plan, result, editing, output);
    return;
  }
  printChangedView(result, editing, output);
}

/** The agent view of `docx` with the read options `o`; `source` is its header name. */
function printView(docx, o, source) {
  process.stdout.write(viewText(docx, o, source));
}

/** `printView`'s Markdown, its warnings said on stderr. */
function viewText(docx, o, source) {
  const view = JSON.parse(wasm.readView(docx, JSON.stringify({
    trackChanges: o.track_changes ?? undefined, comments: o.comments, dates: o.dates,
    pageMarkers: !o.no_page_markers, paragraphs: o.paragraphs ?? undefined,
    head: o.head ?? undefined, tail: o.tail ?? undefined,
    changed: o.changed || undefined, by: o.by ?? undefined, source,
  })));
  for (const warning of view.warnings) console.error(`warning: ${warning}`);
  return view.markdown;
}

function resolution(accept) {
  return {
    run(name, [file], o) {
      const filter = {};
      for (const key of ["ids", "authors", "kinds"]) if (o[key]?.length) filter[key] = o[key];
      // An id that is no change id is a usage error, before any I/O.
      try {
        wasm.changeFilter(JSON.stringify(filter));
      } catch (e) {
        throw new UsageError(messageOf(e));
      }
      // A DOCX keeps that name or not. `--in-place` always writes the
      // package, so a `.json` or `.md` name is not an output format.
      // Sniff the bytes: the name's declared kind would refuse the package.
      const docx = o.in_place ? read(file, null) : read(file);
      if (!o.in_place) ensureWritable(o.output, o.force);
      let out;
      try {
        out = (accept ? wasm.acceptChanges : wasm.rejectChanges)(docx, JSON.stringify(filter));
      } catch (e) {
        // An id that names two changes (a duplicate w:id) is one too.
        const message = messageOf(e);
        if (/^\S+ names \d+ changes/.test(message)) throw new UsageError(message);
        throw new CliError(`${name} failed: ${message}`);
      }
      if (o.in_place) {
        const current = fs.readFileSync(file);
        if (!current.equals(docx)) {
          throw new RefusalError(`SOURCE_CHANGED: '${file}' changed since the edit was prepared; nothing was published`);
        }
        const target = o.output ?? file;
        writeThrough(target, out);
        if (!o.quiet) console.error(inPlaceLine(target, docx, null));
        return;
      }
      writeResult(o, out, notWritten(o.output));
    },
  };
}

function paletteOf(o) {
  return o.revision_palette;
}

function stem(file) {
  return path.basename(file, path.extname(file));
}

// -- shared clap parser and host capabilities --------------------------------

function validateHost(name, o) {
  const reject = (flag) => { throw new UsageError(`--${flag.replaceAll("_", "-")} is not supported by the npm CLI`); };
  if (name === "compare") {
    if ((o.mode ?? "word") !== "word" || o.powertools_faithful) reject("mode powertools");
    if (o.detail_threshold != null) reject("detail_threshold");
    if (o.no_paragraph_merge) reject("no_paragraph_merge");
  }
  // PNG pages and --pages need the native renderer (D14).
  if (name !== "edit" && name !== "convert" && o.output_format === "png") {
    throw new UsageError("-o FILE.png is not supported by the npm CLI: PNG pages need the Python or Rust build (uvx jubarte-redlines)");
  }
  if (name !== "convert" && o.pages != null) reject("pages");
  if (name === "compare") for (const flag of ["reference_doc", "critic", "resource_path"]) if (o[flag] != null && o[flag] !== false) reject(flag);
  if (name === "convert") {
    for (const flag of ["from", "resource_path", "timeout", "fail_on_substitution"]) {
      if (o[flag] != null && o[flag] !== false) reject(flag);
    }
    if (o.png || o.to === "png" || path.extname(o.output ?? "").toLowerCase() === ".png") {
      throw new UsageError("PNG pages need the Python or Rust build (uvx jubarte-redlines convert --png)");
    }
  }
  if (name === "convert") {
    for (const flag of ["dpi", "pages"]) {
      if (flag === "dpi" ? o.dpi !== 96 : o[flag] != null) reject(flag);
    }
  }
  if (name === "convert") {
    if (o.font_report != null) reject("font_report");
    if (o.report != null && !o.update_fields) reject("report");
    // Markdown to Markdown resolves CriticMarkup natively; this CLI has no
    // such engine call.
    const markdownOut = o.to === "md" || (o.to == null && o.output_format === "md");
    if (markdownOut && namedKind(o.file) === "md") {
      throw new UsageError("Markdown to Markdown is not supported by the npm CLI: `uvx jubarte-redlines convert` resolves CriticMarkup");
    }
  }
  if (name === "edit") {
    if (o.png) reject("png");
  }
  if (name === "diff") {
    if (!Number.isInteger(o.context) || o.context < 0 || o.context > 0xffffffff) {
      throw new UsageError("--context must be in the u32 range (0..4294967295)");
    }
  }
}

/** `--json` before any `--`: an error prints on stdout as JSON too. */
function main(argv) {
  try {
    const parsed = JSON.parse(wasm.parseCli(JSON.stringify(argv), PROG, JSON.stringify(Object.keys(COMMANDS))));
    if ("text" in parsed) {
      for (const note of parsed.notes ?? []) console.error(note);
      if (parsed.json != null) process.stdout.write(parsed.json);
      (parsed.stream === "stderr" ? process.stderr : process.stdout).write(parsed.text);
      return parsed.exit_code;
    }
    for (const note of parsed.notes ?? []) console.error(note);
    const name = parsed.command, o = parsed.args;
    validateHost(name, o);
    const positionals = name === "compare" ? [o.original, o.modified] : name === "diff" ? [o.old, o.new] : o.file == null ? [] : [o.file];
    return COMMANDS[name].run(name, positionals, o) ?? EXIT_OK;
  } catch (e) {
    const code = e instanceof UsageError ? EXIT_USAGE : e instanceof RefusalError ? EXIT_PLAN_REFUSED : EXIT_ERROR;
    // A binding older than #1255 has no runtimeJsonError: stdout stays empty.
    if (wasm.runtimeJsonError && wasm.wantsJson(argv)) process.stdout.write(wasm.runtimeJsonError(messageOf(e), code));
    console.error(`error: ${messageOf(e)}`);
    return code;
  }
}

process.exitCode = main(process.argv.slice(2));
