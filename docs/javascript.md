<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Node and browser guide

Using jubarte from JavaScript/TypeScript. Two npm packages share the engine
(both WebAssembly builds of the Rust crate, running in-process — no binary
download, no server round-trip):

| Package | What it is |
|---|---|
| [`jubarte-wasm`](https://www.npmjs.com/package/jubarte-wasm) | the library: compare, changes, revisions, PDF, inspection, edit plans |
| [`jubarte-redlines`](https://www.npmjs.com/package/jubarte-redlines) | the CLI: `npx jubarte-redlines …`, same shared command set |

For the other surfaces see [docs/rust.md](rust.md) and
[docs/python.md](python.md).

## Library usage

Node (CommonJS, auto-initializing; ESM import works the same way):

```js
const { compareDocuments, initPanicHook } = require("jubarte-wasm");
const fs = require("node:fs");

initPanicHook(); // optional: route wasm panics to console.error

const redline = compareDocuments(
  fs.readFileSync("original.docx"),
  fs.readFileSync("modified.docx"),
  "Reviewer",
);
fs.writeFileSync("redline.docx", redline); // opens clean in Microsoft Word
```

Browser / bundlers (ES module with explicit async init):

```js
import init, { compareDocuments, initPanicHook } from "jubarte-wasm/web";

await init(); // fetches jubarte_wasm_bg.wasm relative to the module URL
initPanicHook();
```

Vite, webpack 5 and other bundlers that understand
`new URL("...", import.meta.url)` bundle the `.wasm` automatically; you can
also pass the source yourself: `await init({ module_or_path: url })`.

### Entry points

| Entry point | Build | Contents |
|---|---|---|
| `jubarte-wasm` / `jubarte-wasm/node` | full, Node CJS | all functions |
| `jubarte-wasm/web` | full, browser ESM | all functions |
| `jubarte-wasm/slim` / `jubarte-wasm/node-slim` | slim, Node CJS | everything except `docxToPdf` / `pdfPageCount` |
| `jubarte-wasm/web-slim` | slim, browser ESM | same export set as `node-slim` |

The slim builds drop the PDF engine and its embedded Carlito/Liberation
fonts, shrinking the wasm from ~13 MB to ~3.9 MB; redline output is
identical.

## API reference

Generated from `jubarte-wasm/npm/node/jubarte_wasm.d.ts` — the exact
signatures and doc comments the package ships.

<!-- gen:wasm-api:start -->
Reflects `jubarte-wasm/npm/node/jubarte_wasm.d.ts` (the full Node build; the slim entry points drop `docxToPdf` and `pdfPageCount`).

### `AppendOutput`

```typescript
class AppendOutput {
    readonly docx: Uint8Array
    readonly warnings: string
}
```

What `appendDocuments` returns.

#### `AppendOutput.docx`

```typescript
readonly docx: Uint8Array
```

The joined document.

#### `AppendOutput.warnings`

```typescript
readonly warnings: string
```

What was not carried, as a JSON array of `CODE: message` strings
(`COMMENTS_DROPPED: ...`).

### `EditOutput`

```typescript
class EditOutput {
    readonly clean: Uint8Array | undefined
    readonly json: string
    readonly ok: boolean
    readonly patch: string | undefined
    readonly redline: Uint8Array | undefined
}
```

What `applyEditPlan` and
`previewEditPlan` return. A refused plan is data, not
an exception, so every operation's outcome stays readable.

#### `EditOutput.clean`

```typescript
readonly clean: Uint8Array | undefined
```

The edited document without tracked changes; `undefined` on refusal
and for previews.

#### `EditOutput.json`

```typescript
readonly json: string
```

The report JSON when `ok`, else the error JSON (`code`, `operation`,
`message`, `outcomes`).

#### `EditOutput.ok`

```typescript
readonly ok: boolean
```

`true` when the plan was applied (or resolved, for a preview).

#### `EditOutput.patch`

```typescript
readonly patch: string | undefined
```

The changes the redline tracks as a patch (see
`diffDocuments`), by the plan's author and date;
`undefined` on refusal and for previews.

#### `EditOutput.redline`

```typescript
readonly redline: Uint8Array | undefined
```

The source compared against the clean copy (Word tracked changes);
`undefined` on refusal and for previews.

### `FieldsOutput`

```typescript
class FieldsOutput {
    readonly docx: Uint8Array
    readonly json: string
}
```

What `updateFields` returns.

#### `FieldsOutput.docx`

```typescript
readonly docx: Uint8Array
```

The document with refreshed field results.

#### `FieldsOutput.json`

```typescript
readonly json: string
```

`{"page_count", "fields": [{"kind", "code", "paragraph", "old", "new"}]}`.

### `RepairOutput`

```typescript
class RepairOutput {
    readonly docx: Uint8Array
    readonly json: string
}
```

Output of `repairDocument`.

#### `RepairOutput.docx`

```typescript
readonly docx: Uint8Array
```

The package with every repairable finding fixed.

#### `RepairOutput.json`

```typescript
readonly json: string
```

`{"repaired": [...], "remaining": [...]}`: the findings fixed and the
ones the output still has.

### `acceptChanges`

```typescript
acceptChanges(docx: Uint8Array, filter_json: string): Uint8Array
```

Accept the changes `filterJson` selects and keep the rest tracked, as
Word's Accept This Change does. The filter is `{"ids": [...],
"authors": [...], "kinds": [...]}`: a change is selected when it matches
every list given (`{}` selects every change; an empty list, none).

Mirrors `jubarte::changes::accept_changes`.

### `acceptRevisions`

```typescript
acceptRevisions(docx: Uint8Array): Uint8Array
```

Accept every tracked revision (package-wide) → clean DOCX bytes.

Mirrors `jubarte::document_comparer::accept_revisions`.

### `appendDocuments`

```typescript
appendDocuments(a: Uint8Array, b: Uint8Array, options_json?: string | null): AppendOutput
```

Append B after A, carrying B's images, links, headers, styles, lists and
notes. `optionsJson` is `{"section_break": "next_page" | "continuous" |
"none", "keep_sections": bool, "comments": "drop" | "carry"}`, each
optional; B's comments are dropped (warned) unless `"carry"`.

Mirrors `jubarte::append::append_documents`.

### `applyEditPlan`

```typescript
applyEditPlan(docx: Uint8Array, plan_json: string): EditOutput
```

Apply an edit plan (JSON) to a DOCX: the clean copy, the Word redline and
the per-operation report.

Mirrors `jubarte::edit::apply_plan_json`.

### `auditDocument`

```typescript
auditDocument(docx: Uint8Array, rules?: string | null): string
```

Audit findings as JSON `{findings, rules, layout}` (see `jubarte audit`).
`rules` is a comma-separated list of rule sets (`a11y`, `style`,
`structure`) or codes; omitted or empty runs every rule. The slim build
has no layout pass: it leaves `FONT_SUBSTITUTED` out (naming it is an
error) and does not compare `NUMPAGES` caches with a page count.

### `auditTracked`

```typescript
auditTracked(original: Uint8Array, edited: Uint8Array, author: string): string
```

Every text change from `original` to `edited` must be a revision by
`author`; the findings (`UNTRACKED_EDIT`, `FOREIGN_AUTHOR`) as a JSON
array.

Mirrors `jubarte::validate::audit_tracked`.

### `capabilities`

```typescript
capabilities(): string
```

What this build can do, as JSON (`runtime: "wasm"`): PDF and field
refresh only in the full build, PNG never.

Mirrors `jubarte::capabilities::capabilities`.

### `compareDocuments`

```typescript
compareDocuments(original: Uint8Array, modified: Uint8Array, author: string, input_limits_json?: string | null): Uint8Array
```

Compare two DOCX packages (bytes) → redline DOCX bytes (`w:ins`/`w:del`).

Mirrors `jubarte::document_comparer::compare_documents`.
`inputLimitsJson` (optional) overrides the admission budget key by key:
`{"max_compressed_bytes", "max_entries", "max_part_bytes",
"max_uncompressed_bytes", "max_xml_depth"}`. A package past the budget
throws with `INPUT_LIMIT`; an unknown key throws `invalid input limits`.
The default budget allows 2 GiB inflated, more than a 32-bit WASM heap
holds, so browser hosts should lower it.

### `diffDocuments`

```typescript
diffDocuments(old: Uint8Array, _new: Uint8Array, author: string, date: string, columns?: number | null, old_name?: string | null, new_name?: string | null): string
```

The changes from `old` to `new` as a patch, JSON `{"text", "hunks":
[{"at", "removed", "text"}]}`: only the changed paragraphs, each whole,
with `[-old-]{+new+}` changes and CriticMarkup comments, at its
`body:p:N` id in a Word document or `line:N` in Markdown.

Each side is a `.docx` package or UTF-8 Markdown
(`new TextEncoder().encode(text)`). `author` and `date` (ISO 8601) own
the changes; `columns` wraps the lines (72 by default, 0 does not);
the names default to `old.docx`/`old.md` and `new.docx`/`new.md`.

Mirrors `jubarte::markdown::patch_documents`.

### `documentMarkdown`

```typescript
documentMarkdown(docx: Uint8Array): string
```

Body paragraphs as Markdown, each preceded by its `[body:p:N]` id: the
coordinates an edit plan uses.

Mirrors `jubarte::inspect::markdown`.

### `docxToPdf`

```typescript
docxToPdf(docx: Uint8Array, compress?: boolean | null, revisions?: string | null, revision_palette?: string | null): Uint8Array
```

Render a DOCX package (bytes) → PDF bytes (Word-style layout).

Mirrors `jubarte::convert::docx_to_pdf`. Fonts come from the embedded
Carlito / Liberation set; the native system/cloud font overrides are
no-ops under wasm (no filesystem), which only changes glyph sourcing,
never layout metrics.
`compress` (optional, default `false`) deflates the PDF's streams
(`/FlateDecode`): much smaller output, no longer plain text.
`revisions` (optional, default `"conventional"`) paints tracked changes:
`"conventional"`, `"word"` (Microsoft Word's markup) or `"custom"` with
`revisionPalette` (`"deleted=#AA0000:strike,..."`).

### `editReportJsonl`

```typescript
editReportJsonl(report_json: string): string
```

The JSON-lines form of a report (`load`, one `op` per operation,
`summary`), for agent logs.

### `getRevisions`

```typescript
getRevisions(docx: Uint8Array, input_limits_json?: string | null): string
```

List the tracked revisions in a DOCX as a JSON array string — the same
object shape as the CLI `jubarte revisions --json` lines
(`type`/`author`/`date`/`part`/`moveGroupId`/`isMoveSource`/`formatChange`/`text`).

Mirrors `jubarte::document_comparer::get_revisions` with default settings,
serialized by the shared `revisions_to_json`. `inputLimitsJson` as in
`compareDocuments`.

### `initPanicHook`

```typescript
initPanicHook(): void
```

One-shot init: panic hook → `console.error`. Safe to call multiple times.

### `inspectDocument`

```typescript
inspectDocument(docx: Uint8Array): string
```

The inspection snapshot as JSON: `schema_version`, `source_sha256`,
`summary` and `paragraphs` (ids, text, style, formatting spans,
limitations). Oversized or malformed packages are refused before parsing.

Mirrors `jubarte::inspect::inspect_json`.

### `listChanges`

```typescript
listChanges(docx: Uint8Array): string
```

List the tracked changes one by one as a JSON array string, each with the
id `acceptChanges` / `rejectChanges` select by (the same objects as
`jubarte changes --json`: `id`, `kind`, `target`, `author`, `date`,
`text`, `move_name`, `move_side`, `inside`).

Mirrors `jubarte::changes::list_changes`.

### `listComments`

```typescript
listComments(docx: Uint8Array, author?: string | null, latest?: boolean | null): string
```

List every comment as a JSON array string (the objects `jubarte comments
--json` prints: `id`, `author`, `initials`, `date`, `text`, `parent`,
`done`, `paragraph`, `anchor_text`, `before`, `after`). `author` keeps
one author's comments; `latest` keeps the newest comment of each thread.

Mirrors `jubarte::comments::list_comments` and `select_comments`.

### `markdownToDocx`

```typescript
markdownToDocx(text: string, options_json?: string | null, reference?: Uint8Array | null): Uint8Array
```

Markdown with CriticMarkup → DOCX bytes, as `jubarte convert draft.md`.

`optionsJson` (every field optional): `page` (`"letter"` default, or
`"a4"`), `author` (`"Redline"`), `date` (fixed epoch, so the same Markdown
writes the same bytes), `critic` (`true`: CriticMarkup becomes tracked
changes and comments) and `track_changes` (or `trackChanges`: `"all"`,
`"accept"`, `"reject"`). An unknown field is an error. `reference`, a
`.docx`, lends its styles and page setup, and then `page` is ignored.
Images are written as their alt text, and the engine's warnings are not
returned.

### `pdfPageCount`

```typescript
pdfPageCount(pdf: Uint8Array): number
```

Number of pages in a PDF (cheap object scan; `0` if the bytes are not a
readable PDF).

Mirrors `jubarte::convert::pdf_page_count`.

### `previewEditPlan`

```typescript
previewEditPlan(docx: Uint8Array, plan_json: string): EditOutput
```

Resolve every operation of an edit plan without producing documents.

Mirrors `jubarte::edit::preview_plan`.

### `rejectChanges`

```typescript
rejectChanges(docx: Uint8Array, filter_json: string): Uint8Array
```

Reject the changes `filterJson` selects and keep the rest tracked
(filter as in `acceptChanges`).

Mirrors `jubarte::changes::reject_changes`.

### `rejectRevisions`

```typescript
rejectRevisions(docx: Uint8Array): Uint8Array
```

Reject every tracked revision (package-wide) → base DOCX bytes.

Mirrors `jubarte::document_comparer::reject_revisions`.

### `repairDocument`

```typescript
repairDocument(docx: Uint8Array): RepairOutput
```

The package with every repairable finding fixed, with the findings it
fixed and could not fix in `json`.

Mirrors `jubarte::validate::repair`.

### `scrubDocument`

```typescript
scrubDocument(docx: Uint8Array, options_json?: string | null): Uint8Array
```

Remove who touched a document: author names (as one alias), rsids, the
people and dates in the document properties, and comments.
`optionsJson` is `{"author_alias": string, "rsids": bool, "docprops":
bool, "comments": bool}`, a field left out off; without it, everything
goes under the alias `Author`.

Mirrors `jubarte::scrub::scrub`.

### `sourceSha256`

```typescript
sourceSha256(docx: Uint8Array): string
```

SHA-256 (lowercase hex) of the bytes: the `source_sha256` guard an edit
plan carries.

Mirrors `jubarte::inspect::source_sha256`.

### `updateFields`

```typescript
updateFields(docx: Uint8Array): FieldsOutput
```

Refresh the cached results of `PAGEREF`, `REF`, `NUMPAGES`, `SEQ` and
`TOC` fields from jubarte's layout (page numbers are jubarte's, not
Word's). Full build only: it needs the layout the PDF export links.

Mirrors `jubarte::fields::update_fields`.

### `validateDocument`

```typescript
validateDocument(docx: Uint8Array): string
```

Word-validity findings beyond the schema as a JSON array (`code`,
`part`, `path`, `message`, `word_fatal`, `repairable`); `[]` is a pass.

Mirrors `jubarte::validate::validate`.
<!-- gen:wasm-api:end -->

## CLI usage

```sh
npx jubarte-redlines redline original.docx modified.docx -o redline.docx --author Legal
npx jubarte-redlines changes redline.docx
npx jubarte-redlines accept redline.docx -o clean.docx --kind formatting
npx jubarte-redlines reject redline.docx -o original-again.docx
npx jubarte-redlines convert redline.docx --revisions word
npx jubarte-redlines edit contract.docx --plan plan.json --out-dir review
```

The shared command set, messages and exit codes match the Python wheel and
the Rust binary (0 success, 1 error, 2 usage, 3 edit plan refused), but the
flags differ per surface: PNG pages, `--date` and the other render-side or
PDF-producing flags need the Python or Rust build, and defaults differ (the
Rust binary's `--author` is `Redline`; the others ship `jubarte`). The
generated references below are authoritative. OLE `.doc` and encrypted
files are refused with a clear message before parsing.

### CLI reference

Generated from the real `--help` output of `jubarte-redlines.mjs`.

<!-- gen:cli-npm:start -->
#### `jubarte-redlines`

```text
$ jubarte-redlines --help
usage: jubarte-redlines <command> [options]

DOCX compare, tracked editing, inspection and rendering (the jubarte engine, WebAssembly build).

commands:
  redline, compare  two documents into a Word tracked-changes document
  changes           list each tracked change with the id accept/reject --id and edit plans take
  revisions         list tracked revisions
  accept            accept tracked changes (all, or the ones selected)
  reject            reject tracked changes (all, or the ones selected)
  text              Markdown with [body:p:N] ids, the coordinates an edit plan uses
  inspect           paragraph ids, formatting spans, limitations and package facts
  convert           DOCX to PDF
  edit              apply an edit plan: clean.docx, redline.docx, patch.diff, report.jsonl
  capabilities      what this build can do

  jubarte-redlines <command> --help    a command's options
  jubarte-redlines --version
```

#### `jubarte-redlines redline`

```text
$ jubarte-redlines redline --help
usage: jubarte-redlines redline ORIGINAL MODIFIED [options]

two documents into a Word tracked-changes document

  -o, --output OUTPUT  [default: <original>_v_<modified>.docx]
      --author AUTHOR  who the revisions are by [default: jubarte]
      --force          overwrite an existing output
```

#### `jubarte-redlines compare`

```text
$ jubarte-redlines compare --help
usage: jubarte-redlines compare ORIGINAL MODIFIED [options]

two documents into a Word tracked-changes document

  -o, --output OUTPUT  [default: <original>_v_<modified>.docx]
      --author AUTHOR  who the revisions are by [default: jubarte]
      --force          overwrite an existing output
```

#### `jubarte-redlines changes`

```text
$ jubarte-redlines changes --help
usage: jubarte-redlines changes FILE [options]

list each tracked change with the id accept/reject --id and edit plans take

      --json  one JSON object per line
```

#### `jubarte-redlines revisions`

```text
$ jubarte-redlines revisions --help
usage: jubarte-redlines revisions FILE [options]

list tracked revisions

      --json  one JSON object per line
```

#### `jubarte-redlines accept`

```text
$ jubarte-redlines accept --help
usage: jubarte-redlines accept FILE [options]

accept tracked changes (all, or the ones selected)

  -o, --output OUTPUT  output path (required)
      --force          overwrite an existing output
      --id ID          only this change (body:rev:12); repeatable
      --author AUTHOR  only changes by this author; repeatable
      --kind KIND      only changes of this kind (insertion, deletion, move, formatting); repeatable
```

#### `jubarte-redlines reject`

```text
$ jubarte-redlines reject --help
usage: jubarte-redlines reject FILE [options]

reject tracked changes (all, or the ones selected)

  -o, --output OUTPUT  output path (required)
      --force          overwrite an existing output
      --id ID          only this change (body:rev:12); repeatable
      --author AUTHOR  only changes by this author; repeatable
      --kind KIND      only changes of this kind (insertion, deletion, move, formatting); repeatable
```

#### `jubarte-redlines text`

```text
$ jubarte-redlines text --help
usage: jubarte-redlines text FILE [options]

Markdown with [body:p:N] ids, the coordinates an edit plan uses
```

#### `jubarte-redlines inspect`

```text
$ jubarte-redlines inspect --help
usage: jubarte-redlines inspect FILE [options]

paragraph ids, formatting spans, limitations and package facts

      --json  emit the JSON snapshot
```

#### `jubarte-redlines convert`

```text
$ jubarte-redlines convert --help
usage: jubarte-redlines convert FILE [options]

DOCX to PDF

  -o, --output OUTPUT                      PDF path [default: <stem>.pdf beside the input]
      --force                              overwrite an existing output
      --pdf                                write the PDF (the default)
      --png                                not in this build: use uvx jubarte-redlines or the jubarte binary
      --compress                           deflate PDF streams
      --revisions REVISIONS                how tracked changes are painted: conventional, word or custom [default: conventional]
      --revision-palette REVISION_PALETTE  marks for --revisions custom, e.g. deleted=#AA0000:strike,...
```

#### `jubarte-redlines edit`

```text
$ jubarte-redlines edit --help
usage: jubarte-redlines edit FILE [options]

apply an edit plan: clean.docx, redline.docx, patch.diff, report.jsonl

      --plan PLAN        PLAN.json (required)
      --out-dir OUT_DIR  DIR (required)
      --dry-run          resolve and report only; write nothing
      --force            replace an existing output directory's files
  -q, --quiet            print nothing on success
```

#### `jubarte-redlines capabilities`

```text
$ jubarte-redlines capabilities --help
usage: jubarte-redlines capabilities [options]

what this build can do

      --json  (the output is JSON either way)
```
<!-- gen:cli-npm:end -->

## TypeScript

Every entry point ships its `.d.ts` (wasm-bindgen output carrying the Rust
doc comments). Per-release copies are snapshotted in
[docs/api/](api/) (`jubarte-wasm-{node,web,node-slim,web-slim}-v*.d.ts`).

## Building from source

`jubarte-wasm/build-npm.sh` rebuilds all four flavors with wasm-pack, copies
them into `npm/` and stamps `ENGINE_COMMIT.txt`; `node jubarte-wasm/npm-smoke.mjs`
smoke-tests the assembly. Publishing happens through `scripts/release.sh`
(`jubarte-wasm` first, then `jubarte-redlines`); the package version tracks
the embedded engine version. Details: [VERSIONING](../VERSIONING.md).

## Keeping this page current

`scripts/gen_docs.sh` regenerates both generated blocks; the CI docs job
(`docs.yml`) fails when the committed page drifts. The curated sections
mirror [`jubarte-wasm/npm/README.md`](../jubarte-wasm/npm/README.md) and
[`jubarte-wasm/cli/README.md`](../jubarte-wasm/cli/README.md) — those ship
inside the packages; update them too when you change an example (nothing
automates that sync yet).
