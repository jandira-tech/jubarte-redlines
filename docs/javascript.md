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

### `diffDocumentsUnified`

```typescript
diffDocumentsUnified(old: Uint8Array, new: Uint8Array, oldName?: string, newName?: string, context?: number): string
```

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

### `diffDocumentsCritic`

```typescript
diffDocumentsCritic(old: Uint8Array, _new: Uint8Array, author?: string | null, date?: string | null): string
```

The complete document as CriticMarkup; existing paragraph patches stay separate.

### `diffDocumentsView`

```typescript
diffDocumentsView(old: Uint8Array, _new: Uint8Array, options_json?: string | null): string
```

Document review view. `optionsJson` is a strict camelCase object with
`format` (github, word, normal, context, side-by-side), `oldName`,
`newName`, `context` (u32), `acceptChanges`, `fullLines`, `oldFormat`
and `newFormat` (docx/md). Defaults use the core display window; Word
always accepts both inputs' revisions before creating new CriticMarkup.

### `documentMarkdown`

```typescript
documentMarkdown(docx: Uint8Array): string
```

Body paragraphs as Markdown, each preceded by its `[body:p:N]` id: the
coordinates an edit plan uses.

Mirrors `jubarte::inspect::markdown`.

### `documentMarkdownWithChanges`

```typescript
documentMarkdownWithChanges(docx: Uint8Array, track_changes: string): string
```

Markdown without paragraph ids, with tracked changes kept or resolved.

### `docxToPdf`

```typescript
docxToPdf(docx: Uint8Array, compress?: boolean | null, revisions?: string | null, revision_palette?: string | null, move_comments?: boolean | null, changed_only?: boolean | null): Uint8Array
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
`moveComments` (optional, default `false`) lists the comments after the
last page instead of in balloons beside the text; `changedOnly`
(optional, default `false`) keeps only the pages a tracked change
touches (a document without changes keeps its first page).

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

### `parseCli`

```typescript
parseCli(arguments_json: string, program?: string | null, supported_json?: string | null): string
```

Shared clap parsing, with no filesystem, clock or process access.

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

### `redlineDocuments`

```typescript
redlineDocuments(old: Uint8Array, _new: Uint8Array, author: string, date: string): Uint8Array
```

DOCX/Markdown comparison written as a Word redline, for host CLI I/O.

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
Read, edit, compare and render Word documents

Usage: jubarte-redlines [OPTIONS] [ORIGINAL] [MODIFIED]
       jubarte-redlines <COMMAND>

Tasks:
  compare       Compare documents and write a Word redline [alias: redline]
  revisions     List the tracked revisions in a redline .docx
  changes       List tracked changes with IDs for accept, reject and edit plans
  accept        Accept all tracked changes, or select by ID, author or kind
  reject        Reject all tracked changes, or select by ID, author or kind
  convert       Convert Word or Markdown to DOCX, PDF, PNG or Markdown
  diff          Review differences as GitHub, word, normal, context or side-by-side text
  inspect       Inspect document facts, paragraphs, styles and tables
  text          Read Markdown with edit IDs `[body:p:N]`, or with tracked marks
  edit          Apply a JSON edit plan; write clean copy, redline and report (refusal:
                exit 3)
  capabilities  What this binary can do, for agents choosing an operation
  help          Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Compare options:
  -b, --original <FILE>
          Original/base document (overrides the positional ORIGINAL)

  -m, --modified <FILE>
          Modified document (overrides the positional MODIFIED)

  -o, --output <FILE>
          Output path [default: <original-dir>/<original>_v_<modified>.docx]. A `.md`
          output writes the changes as CriticMarkup (both documents Markdown)

  -a, --author <NAME>
          Author name recorded on the revisions

          [default: Redline]

  -d, --date <ISO8601>
          Revision timestamp (ISO 8601); pinned for reproducible output

          [default: 1970-01-01T00:00:00Z]

      --force
          Overwrite the output file if it already exists

  -q, --quiet
          Do not print the success message

      --detail-threshold <RATIO>
          Word-match detail, from 0 to 1 [default: 0.02; powertools: 0.15]

      --mode <MODE>
          Compare like Microsoft Word or Open-Xml-PowerTools

          Possible values:
          - word:       Microsoft Word Compare's layout: word-level detail, replaced
            paragraphs merged, Word's alignment passes
          - powertools: Open-Xml-PowerTools: coarse paragraph fallback (threshold 0.15),
            no Word alignment passes

          [default: word]

      --powertools-faithful
          Same as --mode powertools

  [MODIFIED]
          The modified document (.docx or Markdown)

  [ORIGINAL]
          The original / base document (.docx or Markdown)

Examples:
  jubarte-redlines compare old.docx new.docx -o redline.docx
  jubarte-redlines inspect contract.docx --json
  jubarte-redlines diff old.docx new.docx --format github
  jubarte-redlines convert contract.docx -o contract.pdf

Run jubarte-redlines <task> --help for task options.
```

#### `jubarte-redlines compare`

```text
$ jubarte-redlines compare --help
Compare documents and write a Word redline

Usage: jubarte-redlines compare [OPTIONS] [ORIGINAL] [MODIFIED]

Arguments:
  [ORIGINAL]
          The original / base document (.docx or Markdown)

  [MODIFIED]
          The modified document (.docx or Markdown)

Options:
  -b, --original <FILE>
          Original/base document (overrides the positional ORIGINAL)

  -m, --modified <FILE>
          Modified document (overrides the positional MODIFIED)

  -o, --output <FILE>
          Output path [default: <original-dir>/<original>_v_<modified>.docx]. A `.md`
          output writes the changes as CriticMarkup (both documents Markdown)

  -a, --author <NAME>
          Author name recorded on the revisions

          [default: Redline]

  -d, --date <ISO8601>
          Revision timestamp (ISO 8601); pinned for reproducible output

          [default: 1970-01-01T00:00:00Z]

      --force
          Overwrite the output file if it already exists

  -q, --quiet
          Do not print the success message

      --detail-threshold <RATIO>
          Word-match detail, from 0 to 1 [default: 0.02; powertools: 0.15]

      --mode <MODE>
          Compare like Microsoft Word or Open-Xml-PowerTools

          Possible values:
          - word:       Microsoft Word Compare's layout: word-level detail, replaced
            paragraphs merged, Word's alignment passes
          - powertools: Open-Xml-PowerTools: coarse paragraph fallback (threshold 0.15),
            no Word alignment passes

          [default: word]

      --powertools-faithful
          Same as --mode powertools

  -h, --help
          Print help (see a summary with '-h')

Examples:
  jubarte compare old.docx new.docx -o redline.docx
  jubarte compare -b old.docx -m new.docx --author Legal
```

#### `jubarte-redlines revisions`

```text
$ jubarte-redlines revisions --help
List the tracked revisions in a redline .docx

Usage: jubarte-redlines revisions [OPTIONS] <FILE>

Arguments:
  <FILE>  The redline document (.docx)

Options:
      --json  Emit the list as JSON lines instead of a human summary
  -h, --help  Print help
```

#### `jubarte-redlines changes`

```text
$ jubarte-redlines changes --help
List tracked changes with IDs for accept, reject and edit plans

Usage: jubarte-redlines changes [OPTIONS] <FILE>

Arguments:
  <FILE>  The document (.docx)

Options:
      --json  Emit one JSON object per line
  -h, --help  Print help
```

#### `jubarte-redlines accept`

```text
$ jubarte-redlines accept --help
Accept all tracked changes, or select by ID, author or kind

Usage: jubarte-redlines accept [OPTIONS] --output <FILE> <FILE>

Arguments:
  <FILE>
          The document (.docx) whose revisions to accept

Options:
  -o, --output <FILE>
          Output path

      --force
          Overwrite the output file if it already exists

      --id <ID>
          Only this change (`body:rev:12`, as `jubarte changes` lists it). Repeatable

      --author <NAME>
          Only changes by this author. Repeatable

      --kind <KIND>
          Only changes of this kind. Repeatable

          Possible values:
          - insertion:  Inserted text or structural elements
          - deletion:   Deleted text or structural elements
          - move:       Content moved between document locations
          - formatting: Changes to text or paragraph formatting

  -h, --help
          Print help (see a summary with '-h')
```

#### `jubarte-redlines reject`

```text
$ jubarte-redlines reject --help
Reject all tracked changes, or select by ID, author or kind

Usage: jubarte-redlines reject [OPTIONS] --output <FILE> <FILE>

Arguments:
  <FILE>
          The document (.docx) whose revisions to reject

Options:
  -o, --output <FILE>
          Output path

      --force
          Overwrite the output file if it already exists

      --id <ID>
          Only this change (`body:rev:12`, as `jubarte changes` lists it). Repeatable

      --author <NAME>
          Only changes by this author. Repeatable

      --kind <KIND>
          Only changes of this kind. Repeatable

          Possible values:
          - insertion:  Inserted text or structural elements
          - deletion:   Deleted text or structural elements
          - move:       Content moved between document locations
          - formatting: Changes to text or paragraph formatting

  -h, --help
          Print help (see a summary with '-h')
```

#### `jubarte-redlines convert`

```text
$ jubarte-redlines convert --help
Convert Word or Markdown to DOCX, PDF, PNG or Markdown

Usage: jubarte-redlines convert [OPTIONS] <FILE>

Arguments:
  <FILE>
          The document to convert: .docx, Markdown (.md, .markdown), or a Word 97-2003
          .doc (read into a .docx first)

Options:
  -o, --output <FILE>
          Output path [default: <stem>.pdf next to a .docx, <stem>.docx next to
          Markdown; Markdown output goes to stdout]. PNG pages are named
          <stem>-page-NN.png beside it

      --force
          Overwrite the output file if it already exists

      --pdf
          Write the PDF (the default when neither --pdf nor --png is given)

      --png
          Rasterize every page to PNG (<stem>-page-NN.png)

      --report <FILE>
          Write a JSON page report (`{page_count, pages:[{index,text}], fonts}`)

      --compress
          Deflate the PDF's streams (`/FlateDecode`). Much smaller output; the trade is
          that the page content is no longer plain text, so it cannot be read with
          `strings` or `grep`

      --font-report <FILE>
          Write a JSON font-resolution report (`[{requested, step, physical, bold,
          italic, synthetic, substituted}, …]`) for this document (plan Step 2f)

      --track-changes <CHOICE>
          Keep tracked changes (all), or write the document with every change accepted
          or rejected (pandoc's flag): CriticMarkup in Markdown, Word's revisions in a
          .docx. With --to md, the Markdown itself is resolved

          Possible values:
          - all:    Keep them: CriticMarkup becomes Word tracked changes and comments
          - accept: Accept every change
          - reject: Reject every change

          [default: all]

      --no-critic
          Markdown: read `{++`, `{--` and the other CriticMarkup delimiters as text

      --reference-doc <FILE>
          Markdown to Word: take styles, numbering, page setup, headers and footers from
          this .docx (pandoc's --reference-doc)

      --resource-path <DIR>
          Markdown to Word: where images are found [default: the Markdown file's
          directory]

  -a, --author <NAME>
          Markdown to Word: author of the tracked changes and comments

          [default: Redline]

  -d, --date <ISO8601>
          Markdown to Word: their date (ISO 8601); pinned for reproducible output

          [default: 1970-01-01T00:00:00Z]

      --page <SIZE>
          Markdown to Word: the page size when there is no --reference-doc (one-inch
          margins either way); a reference's page setup wins

          Possible values:
          - letter: US Letter, 8.5 by 11 inches
          - a4:     ISO A4, 210 by 297 mm

          [default: letter]

      --no-page-markers
          Word to Markdown: leave out the `<!-- page N of M -->` lines, and the layout
          pass that places them

      --pages <SPEC>
          Rasterize only these pages, counted from 1: `3`, `1-3,7`. Layout still runs
          over the whole document. Needs PNG output

      --fail-on-substitution
          Exit 4 when a requested font was substituted (listed on stderr and in
          --report). Every output is still written. Exit status: 0 ok, 1 error, 4 a
          requested font was substituted

      --timeout <SECONDS>
          Give up after this many seconds: exit 124 (as `timeout(1)`) with nothing more
          written. An output being written at that moment may be left partial

  -h, --help
          Print help (see a summary with '-h')

Rendering:
      --dpi <DPI>
          PNG resolution in dots per inch (1-1200)

          [default: 96]

Revision marks:
      --revisions <REVISIONS>
          How tracked changes are painted: `conventional` (deletions red struck through,
          insertions blue underlined, moves green: double-struck where they left,
          double-underlined where they landed), `word` (what Microsoft Word's Save as
          PDF paints), or `custom` (see --revision-palette)

          Possible values:
          - conventional: Red strike, blue underline, green double marks for moves
          - word:         Microsoft Word's own markup
          - custom:       --revision-palette

          [default: conventional]

      --revision-palette <SPEC>
          Marks for --revisions custom: `kind=#RRGGBB[:lines],...` with kinds deleted,
          inserted, moved-from, moved-to and lines strike, double-strike, underline,
          double-underline, plain. Kinds left out keep their conventional mark

Formats:
  -f, --from <FORMAT>
          Input format [default: from the file: .md and .markdown are Markdown, a zip is
          Word]

          Possible values:
          - docx: Word (.docx)
          - md:   Markdown: CommonMark with GitHub tables, task lists and footnotes, and
            CriticMarkup
          - pdf:  PDF, laid out as Word does
          - png:  PNG pages

  -t, --to <FORMAT>
          Output format [default: from --output, else pdf for Word and docx for
          Markdown]

          Possible values:
          - docx: Word (.docx)
          - md:   Markdown: CommonMark with GitHub tables, task lists and footnotes, and
            CriticMarkup
          - pdf:  PDF, laid out as Word does
          - png:  PNG pages

Examples:
  jubarte convert contract.docx                   PDF, Word-style layout
  jubarte convert draft.md                        draft.docx, CriticMarkup as tracked
  changes
  jubarte convert draft.md -o draft.pdf           the changes painted in a PDF
  jubarte convert draft.md --reference-doc house.docx -o draft.docx
  jubarte convert draft.md -t md --track-changes accept   the text with every change
  accepted
  jubarte convert contract.docx -t md             Markdown with <!-- page N of M -->
  lines
  jubarte convert old.doc                         old.docx (text, headings, lists,
  tables)
  jubarte convert notes.md --no-critic            {++ and the other delimiters as text
```

#### `jubarte-redlines diff`

```text
$ jubarte-redlines diff --help
Review differences as GitHub, word, normal, context or side-by-side text

Usage: jubarte-redlines diff [OPTIONS] <OLD> <NEW>

Arguments:
  <OLD>
          The old document: .docx or Markdown

  <NEW>
          The new document: .docx or Markdown

Options:
  -h, --help
          Print help (see a summary with '-h')

Review:
  -o, --output <FILE>
          Write to FILE. Text views default to stdout and write only text. Patch/critic
          infer Word, Markdown, PDF or PNG from the extension

      --format <FORMAT>
          Choose the review view; word accepts ALL input changes first

          Possible values:
          - patch:        The changed paragraphs, as `git diff --word-diff` with
            CriticMarkup comments and highlights
          - critic:       CriticMarkup: current document text with tracked marks
          - github:       Git/GitHub unified text; preserves each document's tracked
            marks
          - word:         Fresh word-level CriticMarkup after accepting ALL changes in
            both inputs
          - normal:       Normal diff with line addresses and no context (a/d/c, < and
            >)
          - context:      Context diff with old/new ranges and !, + and - prefixes
          - side-by-side: Old and new lines in parallel columns, with |, < and > markers

          [default: patch]

  -U, --context <LINES>
          Unchanged lines around GitHub or context hunks; -U0 shows changes only

          [default: 3]

      --accept-changes
          Accept both documents' changes before comparing. Word format always does this

      --full-lines
          Show complete lines instead of a 70-character window around changes

      --force
          Overwrite the output file if it already exists

Paragraph patch:
      --columns <N>
          Wrap the patch's lines at this many columns; 0 does not wrap

          [default: 72]

Formats:
  -t, --to <FORMAT>
          Output format, when --output does not say

          Possible values:
          - docx: Word (.docx)
          - md:   Markdown: CommonMark with GitHub tables, task lists and footnotes, and
            CriticMarkup
          - pdf:  PDF, laid out as Word does
          - png:  PNG pages

  -f, --from <FORMAT>
          Input format of both documents [default: from each file]

          [possible values: docx, md, markdown]

Word redline:
  -a, --author <NAME>
          Who made the changes: the patch's owner and the revisions' author [default:
          `git config user.name`, else Redline]

  -d, --date <ISO8601>
          When (ISO 8601) [default: now]; pin it for reproducible output

      --mode <MODE>
          Whose redline to reproduce (see `jubarte --help`)

          Possible values:
          - word:       Microsoft Word Compare's layout: word-level detail, replaced
            paragraphs merged, Word's alignment passes
          - powertools: Open-Xml-PowerTools: coarse paragraph fallback (threshold 0.15),
            no Word alignment passes

          [default: word]

      --detail-threshold <RATIO>
          LCS detail threshold (see `jubarte --help`)

      --reference-doc <FILE>
          Two Markdown documents written as Word take styles, page setup, headers and
          footers from this .docx

      --critic
          Read CriticMarkup in the Markdown documents as tracked changes (Word output).
          By default a document compared is text

      --resource-path <DIR>
          Where images named by the Markdown are found [default: each Markdown file's
          directory]

Revision marks:
      --revisions <REVISIONS>
          How tracked changes are painted in PDF or PNG output (see `convert --help`)

          Possible values:
          - conventional: Red strike, blue underline, green double marks for moves
          - word:         Microsoft Word's own markup
          - custom:       --revision-palette

          [default: conventional]

      --revision-palette <SPEC>
          Marks for --revisions custom (see `convert --help`)

Examples:
  jubarte diff old.docx new.docx --format github   Git/GitHub patch on stdout
  jubarte diff old.md new.md                       the patch on stdout
  jubarte diff old.md new.md --format critic       CriticMarkup on stdout, as pandiff
  jubarte diff old.md new.md -o changes.docx       Word tracked changes
  jubarte diff old.md new.md -o changes.pdf        the changes painted in a PDF
  jubarte diff contract.docx edited.md -o redline.docx
      the Markdown's edits as tracked changes on the Word document

GIT:
  git config --global difftool.jubarte.cmd 'jubarte diff "$LOCAL" "$REMOTE"'
  git difftool -t jubarte -y -- '*.md'
```

#### `jubarte-redlines inspect`

```text
$ jubarte-redlines inspect --help
Inspect document facts, paragraphs, styles and tables

Usage: jubarte-redlines inspect [OPTIONS] <FILE>

Arguments:
  <FILE>  The document (.docx) to read

Options:
      --json    Emit the snapshot as JSON (`schema_version`, `source_sha256`, `summary`,
                `paragraphs`, `stories`, `tables`) instead of a human summary
      --tables  Print each body table as a grid instead of the paragraphs: a `table N:
                ROWSxCOLS header_rows=H widths=W,...` line, then one line per row of
                tab-separated `ids=text` cells
  -h, --help    Print help
```

#### `jubarte-redlines text`

```text
$ jubarte-redlines text --help
Read Markdown with edit IDs `[body:p:N]`, or with tracked marks

Usage: jubarte-redlines text [OPTIONS] <FILE>

Arguments:
  <FILE>
          The document (.docx) to read

Options:
      --track-changes <CHOICE>
          Print the document as Markdown with its tracked changes as CriticMarkup (all),
          or with every change accepted or rejected, like `convert --to md`. The output
          then has no `[body:p:N]` ids

          Possible values:
          - all:    Keep them: CriticMarkup becomes Word tracked changes and comments
          - accept: Accept every change
          - reject: Reject every change

  -h, --help
          Print help (see a summary with '-h')
```

#### `jubarte-redlines edit`

```text
$ jubarte-redlines edit --help
Apply a JSON edit plan; write clean copy, redline and report (refusal: exit 3)

Usage: jubarte-redlines edit [OPTIONS] --plan <PLAN.json> --out-dir <DIR> <FILE>

Arguments:
  <FILE>
          The source document (.docx). Never modified

Options:
      --plan <PLAN.json>
          Edit plan JSON (see `jubarte capabilities --json` for the kinds)

      --out-dir <DIR>
          Directory to create for clean.docx, redline.docx, report.jsonl

      --dry-run
          Resolve and report only; write nothing

      --force
          Replace an existing output directory's files

      --pdf
          Also write redline.pdf and clean.pdf

      --png
          Also write redline-page-NN.png and clean-page-NN.png

  -q, --quiet
          Print nothing on success (patch.diff and report.jsonl are still written)

  -h, --help
          Print help (see a summary with '-h')

Rendering:
      --dpi <DPI>
          PNG resolution in dots per inch (1-1200)

          [default: 96]

Revision marks:
      --revisions <REVISIONS>
          How tracked changes are painted in the redline PDF/PNG

          Possible values:
          - conventional: Red strike, blue underline, green double marks for moves
          - word:         Microsoft Word's own markup
          - custom:       --revision-palette

          [default: conventional]

      --revision-palette <SPEC>
          Marks for --revisions custom (see `convert --help`)
```

#### `jubarte-redlines capabilities`

```text
$ jubarte-redlines capabilities --help
What this binary can do, for agents choosing an operation

Usage: jubarte-redlines capabilities [OPTIONS]

Options:
      --json  Emit JSON (the default output is JSON too; the flag documents intent)
  -h, --help  Print help
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
the embedded engine version. Details: [VERSIONING](VERSIONING.md).

## Keeping this page current

`scripts/gen_docs.sh` regenerates both generated blocks; the CI docs job
(`docs.yml`) fails when the committed page drifts. The curated sections
mirror [`jubarte-wasm/npm/README.md`](../jubarte-wasm/npm/README.md) and
[`jubarte-wasm/cli/README.md`](../jubarte-wasm/cli/README.md) — those ship
inside the packages; update them too when you change an example (nothing
automates that sync yet).
