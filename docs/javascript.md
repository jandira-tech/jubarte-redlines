<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Node and browser guide

Revision tags use the [settled Markdown dialect](MARKDOWN.md#tags): separate ids,
same-author deletion-first substitutions, old-side `re` parents and inline `¶` marks.

Using jubarte from JavaScript/TypeScript. Two npm packages share the engine
(both WebAssembly builds of the Rust crate, running in-process — no binary
download, no server round-trip):

| Package | What it is |
|---|---|
| [`jubarte-wasm`](https://www.npmjs.com/package/jubarte-wasm) | the library: compare, changes, revisions, PDF, inspection, edit plans |
| [`jubarte-redlines`](https://www.npmjs.com/package/jubarte-redlines) | the CLI: `npx jubarte-redlines …`, ten of the native tasks (see [CLI usage](#cli-usage)) |

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
| `jubarte-wasm/slim` / `jubarte-wasm/node-slim` | slim, Node CJS | everything except `docxToPdf`, `pdfPageCount` and `updateFields` |
| `jubarte-wasm/web-slim` | slim, browser ESM | same export set as `node-slim` |

The slim builds drop the PDF engine and its embedded Carlito/Liberation
fonts; redline output is identical. The size saving is now small: in
0.12.0 the full wasm is 17.47 MB and the slim one 17.45 MB.

## API reference

The snapshot uses `schema_version: 3`. `changes` lists logical changes:
a change Word wrote as several marks is one entry with its `members`,
and a deletion followed by its author's insertion is one `replacement`.
`summary.tables` and `tables[]`
count top-level body tables only; each `index` is the view's `tN`.
Every table has a `nested` array (empty for a leaf), containing its cell tables.
A nested entry has `parent` (for example `t0.row2.cell0`) and no `index`.
A deeper parent includes the cell-local table ordinal, for example
`t0.row2.cell0.t0.row0.cell1`. Paragraph ids and edit-plan schemas are unchanged.

`--tables` numbers only top-level tables as `table N`, matching `tN` in
the view and `-p tN`. Nested tables follow their containing cell row,
indented two spaces per nesting level and labelled `nested in CELL_PATH`.
For example, `nested in t0.row2.cell0`; a further level uses
`t0.row2.cell0.t0.row0.cell1`. `--tables -p t0` includes this whole tree.

Generated from `jubarte-wasm/npm/node/jubarte_wasm.d.ts` — the exact
signatures and doc comments the package ships.

The identity fields the models carry retain their existing wire shape: `id` and
`ref` both hold the short id (`r12`, `header1.r1`, `c5` — the colon
fallback `body:rev:007` only when no short spelling reads back; `9872a374`,
`52e96ced`), comment ids are numeric and `Snapshot.schema_version == 3` for logical changes over the table-tree shape.
The 17 October 2026 short-and-long-ID contract
([superpowers/specs/2026-10-17-short-and-long-id-contract.md](superpowers/specs/2026-10-17-short-and-long-id-contract.md),
staged in
[superpowers/plans/2026-10-16-id-contract-implementation.md](superpowers/plans/2026-10-16-id-contract-implementation.md))
replaces them with paired `short_id`/`long_id` references at the planned identity-schema
cutover and retires the transitional spellings — `noid`, colon ids
(`body:rev:12`), nested `.pN` inside cells and text boxes
(`t0.row1.cell2.p1`), open ranges (`p17-`, `-p3`). It is not implemented:
the fields the wire carries today are the ones named above.

<!-- gen:wasm-api:start -->
Reflects `jubarte-wasm/npm/node/jubarte_wasm.d.ts` (the full Node build; the slim entry points drop `docxToPdf` and `pdfPageCount`).

### `diffDocumentsUnified`

```typescript
diffDocumentsUnified(old: Uint8Array, next: Uint8Array, oldName?: string, newName?: string, context?: number): string
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

### `RenderOutput`

```typescript
class RenderOutput {
    readonly json: string
    readonly pdf: Uint8Array
}
```

What `renderPdf` returns.

#### `RenderOutput.json`

```typescript
readonly json: string
```

`{"page_count", "page_starts"}`: the pages written and each one's
first line (60 characters at most), as an edit bundle's `render`
line lists them.

#### `RenderOutput.pdf`

```typescript
readonly pdf: Uint8Array
```

The PDF.

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
applyEditPlan(docx: Uint8Array, plan_json: string, name?: string | null): EditOutput
```

Apply an edit plan (JSON) to a DOCX: the clean copy, the Word redline,
the patch of the plan's changes and the per-operation report. `name`
(default `document.docx`) is the file the patch names; under
`existing_revisions: keep` the patch shows the plan's own changes, not
the other party's the redline also holds.

Mirrors `jubarte::edit::apply_plan_json`.

### `auditDocument`

```typescript
auditDocument(docx: Uint8Array, rules?: string | null): string
```

Audit findings as JSON `{findings, rules, layout}` (see `jubarte validate --rules`).
`rules` is a comma-separated list of rule sets (`a11y`, `style`,
`structure`) or codes; omitted or empty runs every rule. The slim build
has no layout pass: it leaves `FONT_SUBSTITUTED` out (naming it is an
error) and does not compare `NUMPAGES` caches with a page count.

### `auditTracked`

```typescript
auditTracked(original: Uint8Array, edited: Uint8Array, author: string): string
```

Check that every text change from `original` to `edited` is a revision
by `author`. Returns the findings (`UNTRACKED_EDIT`, `FOREIGN_AUTHOR`,
`FOREIGN_COMMENT` for another author's comment gone, reworded, resolved
or reopened, or a new comment not signed `author`, and
`PERMISSION_RANGE_DROPPED` for a permission endpoint of `original` that
`edited` lost) as a JSON array.

Mirrors `jubarte::validate::audit_tracked`.

### `capabilities`

```typescript
capabilities(): string
```

What this build can do, as JSON (`runtime: "wasm"`): PDF and field
refresh only in the full build, PNG never.

Mirrors `jubarte::capabilities::capabilities`.

### `capabilitiesText`

```typescript
capabilitiesText(manifest_json?: string | null): string
```

[`capabilities`] as the text summary `jubarte capabilities` prints; of
`manifestJson` when given (a manifest a host narrowed to what it does),
else of this build's.

Mirrors `jubarte::capabilities::capabilities_text`.

### `capabilityQuery`

```typescript
capabilityQuery(operation?: string | null, field?: string | null): string
```

Focused edit discovery; no filters returns this runtime's edit support matrix.

### `capabilityQueryText`

```typescript
capabilityQueryText(operation?: string | null, field?: string | null): string
```

Render a focused query in the same text form as the native CLI.

### `changeFilter`

```typescript
changeFilter(filter_json: string): string
```

`filterJson` (as `acceptChanges` takes it) with each id
in its settled spelling (`r1+2` → `r1`, `r2`). An id that is no change
id, the expired `body:rev:1` among them, throws `jubarte-wasm: c5:
expected a change id …` (naming `r1` for `body:rev:1`), which the
binary reports as a usage error before reading the document.

Mirrors `jubarte::changes::ChangeFilter::normalized`.

### `changedView`

```typescript
changedView(docx: Uint8Array, author: string, accepted: boolean, source?: string | null): string
```

The blocks of `docx` (a redline) carrying `author`'s marks, as the agent
view `edit` and `add` print (every changed block when none does);
`accepted` reads its accepted text, `source` is the name printed as
`source:`.

Mirrors `jubarte::markdown::changed_view`.

### `changesText`

```typescript
changesText(docx: Uint8Array, authors?: string[] | null): string
```

The listing `jubarte changes` prints: a line per change with its short
id (`r0`), kind, target, author, paragraph and text (then `(N elements)`
for a change Word wrote as several marks), then `N change(s)`.
`authors` filters as in `listChanges`.

Mirrors `jubarte::changes::changes_text`.

### `commentsText`

```typescript
commentsText(docx: Uint8Array, author?: string | null, latest?: boolean | null): string
```

The listing `jubarte comments` prints: a line per comment with its short
id (`c5`), paragraph, author, text and anchor, then `N comment(s)`.
`author` and `latest` filter as in `listComments`.

Mirrors `jubarte::comments::comments_text`.

### `compareDocuments`

```typescript
compareDocuments(original: Uint8Array, modified: Uint8Array, author: string, input_limits_json?: string | null): Uint8Array
```

Compare two DOCX packages (bytes) → redline DOCX bytes (`w:ins`/`w:del`).

Mirrors `jubarte::document_comparer::compare_documents`.
`inputLimitsJson` (optional) overrides the admission budget key by key:
`{"max_compressed_bytes", "max_entries", "max_part_bytes",
"max_uncompressed_bytes", "max_xml_depth", "max_xml_elements",
"max_story_units"}`. A package past the budget
throws with `INPUT_LIMIT`; an unknown key throws `invalid input limits`.
The default budget allows 2 GiB inflated, more than a 32-bit WASM heap
holds, so browser hosts should lower it.

### `convertMarkdown`

```typescript
convertMarkdown(docx: Uint8Array, track_changes: string, page_markers: boolean, revisions?: string | null, revision_palette?: string | null): string
```

A Word document as `convert --to md` writes it: Markdown with its
changes kept, accepted or rejected (`trackChanges`: `all`, `accept`,
`reject`), and unless `pageMarkers` is false `<!-- page N of M -->` lines
from the layout, its changes painted as `revisions`. JSON
`{"markdown", "warning"}`: the warning says why the markers are missing.
Full build only.

Mirrors `jubarte::markdown::with_page_markers`.

### `diffDocuments`

```typescript
diffDocuments(old: Uint8Array, next: Uint8Array, author: string, date: string, columns?: number | null, old_name?: string | null, new_name?: string | null): string
```

The changes from `old` to `next` as a patch, JSON `{"text", "hunks":
[{"at", "removed", "text"}]}`: only the changed paragraphs, each whole,
with `[-old-]{+new+}` changes and CriticMarkup comments, at its
`pN` id in a Word document or `line:N` in Markdown. When a side is a
Word document, a replacement inside a paragraph reads insertion first,
`{+new+}[-old-]`, as Word Compare writes it.

Each side is a `.docx` package or UTF-8 Markdown
(`new TextEncoder().encode(text)`). `author` and `date` (ISO 8601) own
the changes; `columns` wraps the lines (72 by default, 0 does not);
the names default to `old.docx`/`old.md` and `new.docx`/`new.md`.

Mirrors `jubarte::markdown::patch_documents`.

### `diffDocumentsCritic`

```typescript
diffDocumentsCritic(old: Uint8Array, next: Uint8Array, author?: string | null, date?: string | null): string
```

The complete document as CriticMarkup; existing paragraph patches stay separate.

### `diffDocumentsView`

```typescript
diffDocumentsView(old: Uint8Array, next: Uint8Array, options_json?: string | null): string
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

Body paragraphs as Markdown, each preceded by its `[pN]` id: the
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
touches, the pages a comment is anchored on and each section's first
page on which a revised header or footer paints (else the first page).

### `editReportJsonl`

```typescript
editReportJsonl(report_json: string): string
```

The JSON-lines form of a report (`load`, one `op` per operation,
`summary`), for agent logs.

### `editingModeConflict`

```typescript
editingModeConflict(plan: string, docx: Uint8Array): string | undefined
```

Why `--editing-mode` cannot apply `plan` to `docx`: the tracked changes
it keeps and does not itself accept or reject, or a reply to a comment
on a paragraph it deletes; `undefined` when neither.

Mirrors `jubarte::edit::flags::editing_mode_conflict`.

### `flagPlan`

```typescript
flagPlan(verb: string, operations_json: string, docx: Uint8Array, options_json?: string | null): string
```

The plan the `edit`/`add` operation flags describe, as JSON `{plan,
notes}`: `plan` is the plan's JSON for `applyEditPlan`,
`notes` the `op-N: …` lines to print. `verb` is `edit` or `add`;
`operationsJson` the parsed command's `operations` array. Options (JSON,
all optional): `author` (default `Redline`), `date`,
`existingRevisions` (auto, keep, accept, reject, refuse; default auto).
An error is a usage error.

Mirrors `jubarte::edit::flags::plan_from_flags`.

### `forEdit`

```typescript
forEdit(docx: Uint8Array, options_json?: string | null): string
```

The working-document JSON `jubarte read --for-edit` prints
(`jubarte-read-for-edit-v1`): source hash, paired target ids, exact
working text with `before_sha256`, and per-operation readiness. Options
(JSON, all optional): `paragraphs` (`"p3,p10-p20,t0"`), `head`, `tail`,
`find`/`regex` (+ `ignoreCase`) on the original final text, `changed`
with `by`, `context` (read-only original neighbours), `limit`,
`maxChars`, `all`, `cursor`, `acceptChange`/`rejectChange` (revision
ids), `existingRevisions` (`keep`, `accept`, `reject`, `refuse`,
`auto`) and `track` (`on`, `off`); `validityFile`, the path as the
command line spells it, adds `input_validity`. A `USAGE` failure throws
`jubarte-wasm: USAGE: …`, as the CLI exits 2; the other refusals throw
`jubarte-wasm: CODE: …`, as the CLI exits 3.

Mirrors `jubarte::edit::for_edit::for_edit_json`.

### `getRevisions`

```typescript
getRevisions(docx: Uint8Array, input_limits_json?: string | null): string
```

List the tracked revisions in a DOCX as a JSON array string, grouped as
Docxodus's `GetRevisions` groups them; each object has
(`type`/`author`/`date`/`part`/`moveGroupId`/`isMoveSource`/`formatChange`/`text`).

Mirrors `jubarte::document_comparer::get_revisions` with default settings,
serialized by the shared `revisions_to_json`. `inputLimitsJson` as in
`compareDocuments`.

### `initPanicHook`

```typescript
initPanicHook(): void
```

One-shot init: panic hook → `console.error`. Safe to call multiple times.

### `inputNotice`

```typescript
inputNotice(docx: Uint8Array, file: string): string | undefined
```

The one stderr line `read` and `changes` give an input with validity
findings (`input has 2 validity warnings; jubarte validate FILE lists
them`), or `undefined` when there is none or the package cannot be
validated; `file` is the path as the command line spells it.

Mirrors `jubarte::validate::input_notice`.

### `inputValidityLine`

```typescript
inputValidityLine(docx: Uint8Array, file: string): string
```

The JSON status `changes --json` and `comments --json` print on stderr
unless quiet, with the input's validity findings counted (`{"input_validity":{"errors":0,"warnings":2,
"next":"jubarte validate FILE"}}`); `file` is the path as the command
line spells it. A package that cannot be validated counts none.

Mirrors `jubarte::validate::input_validity_line`.

### `inspectDocument`

```typescript
inspectDocument(docx: Uint8Array): string
```

The inspection snapshot as JSON: `schema_version`, `source_sha256`,
`summary`, `paragraphs` (ids, text, style, formatting spans,
limitations) and top-level `tables` with recursive `nested` entries.
Snapshot schema 3 lists logical changes (one entry, with `members`, for
a change Word wrote as several marks), counts top-level tables, and gives
nested entries `parent` cell paths and no `index`. Oversized or malformed packages are refused before parsing.

Mirrors `jubarte::inspect::inspect_json`.

### `listChanges`

```typescript
listChanges(docx: Uint8Array, authors?: string[] | null): string
```

List the tracked changes as a JSON array string, one object per
logical change: a change Word wrote as several marks is one object
with a composite id (`r1+2`) and its `members`, and an author's
deletion followed by their insertion is one `replacement`. Each `id`
is one `acceptChanges` / `rejectChanges` select by. The objects are
the ones `jubarte changes --json` prints: `id`, `ref` (the same id),
`kind` (`insertion`, `deletion`, `move`, `formatting` or
`replacement`), `target`, `author`, `date`, `text`, `paragraph`,
`move_name`, `move_side`, `inside`, `holds_comments` when the change
holds comment marks, and `members` (each mark's `id`, `kind`,
`target`, `date` and `text`) for a change of several marks.
`authors` keeps those authors' changes (exact names), as `jubarte
changes --author`; omitted or empty, every change.

Mirrors `jubarte::changes::list_logical_changes` and `by_authors`.

### `listComments`

```typescript
listComments(docx: Uint8Array, author?: string | null, latest?: boolean | null): string
```

List every comment as a JSON array string (the objects `jubarte comments
--json` prints: `id`, `author`, `initials`, `date`, `text`, `parent`,
`done`, `paragraph`, `last_paragraph` when the range ends in a later
paragraph, `anchor_text`, `before`, `after`). `author` keeps one
author's comments; `latest` keeps the newest comment of each thread.

Mirrors `jubarte::comments::list_comments` and `select_comments`.

### `markdownStamp`

```typescript
markdownStamp(text: string, reference?: Uint8Array | null): string
```

The author and timestamp a Markdown document gives the changes and
comments that name none, as JSON `[author, date]` (`null` for each it
does not tell): its front matter, else the reference `.docx`'s creator
and creation time. The CLI falls back to `Redline` and now.

### `markdownToDocx`

```typescript
markdownToDocx(text: string, options_json?: string | null, reference?: Uint8Array | null): Uint8Array
```

Markdown with CriticMarkup → DOCX bytes, as `jubarte convert draft.md`.

`optionsJson` (every field optional): `page` (`"letter"` default, or
`"a4"`), `author` and `date` (for a change or comment that names none;
given, they beat the front matter's `author:` and `last_modified:`,
`modified:`, `date:`, `created:`, which beat `"Redline"` and the fixed
epoch, so the same Markdown writes the same bytes; the CLI dates them now
instead), `critic` (`true`: CriticMarkup becomes tracked changes and
comments), `track_changes` (or `trackChanges`: `"all"`,
`"accept"`, `"reject"`), `tracking` (`true`/`false`: Word's Track
Changes switch in the output, whatever the front matter's
`track_changes:` says) and `scholar` (`true`: set as LaTeX's article
class, page numbers from page 2, as `--scholar`). An unknown field is
an error. `reference`, a
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

### `planError`

```typescript
planError(plan_json: string): string | undefined
```

Why `planJson` is no edit plan, as the binary prints it (`INVALID_PLAN:
…`); `undefined` for a plan. A plan that parses can still be refused when
applied; that refusal carries the per-operation report.

Mirrors `jubarte::edit::EditPlan::from_json`.

### `previewEditPlan`

```typescript
previewEditPlan(docx: Uint8Array, plan_json: string): EditOutput
```

Resolve every operation of an edit plan without producing documents.

Mirrors `jubarte::edit::preview_plan`.

### `readJson`

```typescript
readJson(docx: Uint8Array, options_json?: string | null): string
```

The snapshot `jubarte read --json` prints, with its `changes` and
`comments`. Options (JSON, all optional) select as in
`readView`: `paragraphs`, `head`, `tail`, `changed`, `by`;
`source` names the document; `trackChanges` `accept` or `reject`
settles the text and names the choice in `track_changes`;
`validityFile`, the path as the command line spells it, adds
`summary.input_validity`, the input's validity findings counted. The
snapshot keeps the selected paragraphs and their changes and comments.

Mirrors `jubarte::markdown::read_snapshot`.

### `readView`

```typescript
readView(docx: Uint8Array, options_json?: string | null): string
```

The agent view (`read`) as JSON `{markdown, warnings}`: a YAML header,
then Markdown with `<!-- pN -->` id lines and every change and comment
with its id. Options (JSON, all optional): `trackChanges` (all, accept,
reject), `comments` (inline, none), `dates`, `pageMarkers` (default
true: the layout pass numbers the pages), `paragraphs` (`"p3,p10-p20,t0"`),
`head`, `tail`, `changed` (only the blocks with marks) with `by` (one
author's: a handle or a full name), `source` (the name printed as
`source:`).

Mirrors `jubarte::markdown::read`.

### `recordPlan`

```typescript
recordPlan(verb: string, operations_json: string, docx: Uint8Array, options_json: string, held?: string | null): string
```

`edit`/`add --plan PLAN.json` with operation flags: the operations
recorded into `held` (PLAN.json's text, absent before the first
recording), as JSON `{ok: true, recorded: {plan, json, notes,
summary}}`: preview `plan`, write `json`, print `notes` and `summary`.
On failure `{ok: false, usage, message}`: a usage error (exit 2), or
`held` is not a plan (exit 3). Options (JSON): `author`, `date`,
`existingRevisions`, `file`, `plan`, `defaultPlan` (a bare `--plan`),
`dryRun`, `rewriteOthersComments`.

Mirrors `jubarte::edit::record::record`.

### `redlineDocuments`

```typescript
redlineDocuments(old: Uint8Array, next: Uint8Array, author: string, date: string): Uint8Array
```

DOCX/Markdown comparison written as a Word redline, for host CLI I/O.

### `refusedInput`

```typescript
refusedInput(path: string, bytes: Uint8Array, word: boolean): string | undefined
```

The error line for an input no task reads, as the binary prints it:
`LEGACY_DOC: a.docx is a Word 97-2003 …`, or (`word`) `INVALID_PACKAGE:
a.docx is not a ZIP file: …`; `undefined` when the bytes may be read.

Mirrors `jubarte::cli::refused_input`.

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

### `renderPdf`

```typescript
renderPdf(docx: Uint8Array, revisions?: string | null, revision_palette?: string | null): RenderOutput
```

An edited document's PDF, compressed, its changes painted as
`revisions` (`conventional`, `word` or `custom` with
`revisionPalette`), with the page facts `edit --pdf` reports. Full build
only.

Mirrors `jubarte::convert::render`.

### `repairDocument`

```typescript
repairDocument(docx: Uint8Array): RepairOutput
```

The package with every repairable finding fixed, with the findings it
fixed and could not fix in `json`.

Mirrors `jubarte::validate::repair`.

### `resolutionText`

```typescript
resolutionText(docx: Uint8Array, resolved_json: string): string
```

The lines `edit -p rN --accept/--reject` prints for a pure resolution:
one per resolved change from `source` (the bytes before resolution) and
`resolvedJson` (`{"accepted": [...], "rejected": [...]}` short ids).

Mirrors `jubarte::edit::resolution::resolution_text`.

### `runtimeJsonError`

```typescript
runtimeJsonError(message: string, exit: number): string
```

The line an error prints on stdout under `--json`:
`{"error":{"code":…,"message":…}}`, its code the message's leading
`CODE:`, else `USAGE` for an exit 2 and `ERROR` for any other.

Mirrors `jubarte::cli::runtime_json_error`.

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

### `tablesText`

```typescript
tablesText(docx: Uint8Array, options_json?: string | null): string
```

The grids `jubarte read --tables` prints: a `table N: RxC …` line per
top-level body table, then a line per row of `ids=text` cells; nested
tables follow their cell row, indented two spaces per level and labelled
`nested in CELL_PATH`. `no tables` prints when there are none. Options (JSON, all optional): `trackChanges` (`reject`
reads the rejected text).

Mirrors `jubarte::markdown::read_tables` without a selection.

### `tablesView`

```typescript
tablesView(docx: Uint8Array, options_json?: string | null): string
```

What `jubarte read --tables` prints, as JSON `{markdown, warnings}`:
the grids (`tablesText`), or with `paragraphs` (`"t0"`, `"p11"`,
`"p3-p20"`) the grids of the tables the view's selection reaches (a
`pN` inside a table, that table), each with the tables nested in it,
then an empty line and the view `readView` prints of that selection.
Options (JSON, all optional): `trackChanges`, `comments`, `dates`,
`pageMarkers`, `paragraphs`, `source`, as `readView` reads them. A pick
that reaches no table throws `jubarte-wasm: USAGE: …`, as the CLI
exits 2.

Mirrors `jubarte::markdown::read_tables`.

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

Word-validity findings beyond the schema as a JSON array (`code`, `set`,
`severity`, `part`, `location`, `ref`, `message`, `word_fatal`,
`repairable`; `ref` is the short id the CLI prints for `location`);
`[]` is a pass.

Mirrors `jubarte::validate::validate`.

### `wantsJson`

```typescript
wantsJson(_arguments: string[]): boolean
```

Whether a command line (without the program name) asks for JSON: a
real `--json` flag before any `--`, read as the binary reads it (the
`--json` in `--text --json` is a value). An error then prints its
[`runtime_json_error`] line on stdout too.

Mirrors `jubarte::cli::wants_json`.

### `wroteLine`

```typescript
wroteLine(path: string, format: string, size: number, pages?: number | null): string
```

The line a writing task prints on stderr for the file it wrote:
`wrote r.pdf (pdf, 48213 bytes, 3 pages)`. `format` is `docx`, `md`,
`pdf`, `png` or `json`; `pages` is a PDF's.

Mirrors `jubarte::cli::wrote_line`.
<!-- gen:wasm-api:end -->

### Auditing existing revision history

`auditTracked`: Existing foreign revisions may remain: `FOREIGN_AUTHOR` reports
foreign history added, changed or removed against `original`. Paragraphs
align by full text, including deleted text, within each story. An untracked
paragraph merge or split around another author's revisions reports those
revisions as new and gone alongside `UNTRACKED_EDIT`. Groups with
the same author, kind, target, date and move side are compared in document
order within the aligned paragraph or non-paragraph owner (for example, a
style id). Revision ids do not define identity. Split carriers combine only
when they touch, with no visible content or still-present revision between
them. A boundary explained solely by a missing separator revision adds no
finding beyond that separator. Changed revisions say `differs from the
original`; new revisions say `is new`. These messages name ids and authors,
never revision text.

## CLI usage

```sh
npx jubarte-redlines redline original.docx modified.docx -o redline.docx --author Legal
npx jubarte-redlines changes redline.docx
npx jubarte-redlines accept redline.docx -o clean.docx --kind formatting
npx jubarte-redlines reject redline.docx -o original-again.docx
npx jubarte-redlines convert redline.docx --revisions word
npx jubarte-redlines edit contract.docx --plan plan.json --out-dir review
```

The npm CLI parses with the native CLI's own parser and has its tasks
`compare`, `changes`, `accept`, `reject`, `convert`, `diff`, `read`, `edit`,
`add` and `capabilities`; `visual-diff`, `comments`, `append`, `validate` and
`scrub` need the Python or Rust build. Messages and exit codes match the
Python wheel and the Rust binary (0 success, 1 error, 2 usage, 3 edit plan
refused). An existing output without `--force` is that usage refusal, on
every task that writes: `error: OUTPUT_EXISTS: output '…' already exists
(use --force to overwrite)`, the path spelled with `/` on every OS (#1692).
A flag the npm CLI cannot honour exits 2 naming it: PNG output,
`--pages`, `--dpi`, `--update-fields`, `--report`, `--font-report`,
`--timeout`, `--fail-on-substitution`, `convert --from`, `--resource-path`
and Markdown output (`-t md`; `read` prints the agent view), `edit
--pdf`/`--png`/`--revisions`, and `compare --mode powertools`,
`--detail-threshold`, `--no-paragraph-merge`, `--reference-doc` and
`--critic`. The generated references below are authoritative. OLE `.doc`
and encrypted files are refused with a clear message before parsing.

### CLI reference

Generated from the real `--help` output of `jubarte-redlines.mjs`.

The native `jubarte` binary offers `--keep-nested-comment` on `accept`,
`reject` and `convert --track-changes accept|reject`. These bindings do not
yet dispatch that option or return its kept-thread receipts; use the native
binary when retention is needed.

<!-- gen:cli-npm:start -->
#### `jubarte-redlines`

```text
$ jubarte-redlines --help
jubarte-redlines 0.16.0
Read, edit, compare and render Word documents

Usage: jubarte-redlines [OPTIONS]
       jubarte-redlines <COMMAND>

Tasks:
  compare       Compare documents: print the redline as the agent view, or write it with
                -o [alias: redline]
  changes       List tracked changes with IDs for accept, reject and edit plans
  accept        Accept all tracked changes, or select by ID, author or kind
  reject        Reject all tracked changes, or select by ID, author or kind
  convert       Convert Word or Markdown to DOCX, PDF, PNG or Markdown
  diff          Review differences as GitHub, word, normal, context or side-by-side text
  read          Read the agent view: YAML header, `<!-- pN -->` id lines, changes and
                comments with ids
  edit          Edit a document: -p WHERE with --anchor, --content, --delete, --resolve,
                --reply, --accept, --reject, --strike, --restore or --style (several -p
                per command), or a JSON plan. Writes FILE_modified.docx (--in-place
                replaces FILE; --out-dir: the bundle), then prints SUCCESS and a fenced
                before/after diff of the full agent views (refusal: exit 3)
  add           Add a paragraph, a comment or a reply: -p WHERE --content TEXT (several
                -p per command). Writes FILE_modified.docx, as edit does (--in-place
                replaces FILE), and prints SUCCESS and a fenced before/after diff of the
                full agent views
  capabilities  What this binary can do, for agents choosing an operation
  help          Print this message or the help of the given subcommand(s)

Options:
  -q, --quiet    No status on success: no `wrote` lines, SUCCESS block or edit summary.
                 Files are still written; a reading task (`read`, `changes`, `compare`
                 without -o, `diff`, `capabilities`) still prints what it read, and
                 warnings and errors still print. `read`, `changes` and `comments` drop
                 their stderr validity status (JSON under `changes --json` and `comments
                 --json`). `read --json` and `read --for-edit` keep validity counts in
                 their output, -q or not
  -h, --help     Print help
  -V, --version  Print version

Examples:
  jubarte-redlines compare old.docx new.docx -o redline.docx
  jubarte-redlines contract.docx
  jubarte-redlines read contract.docx --json
  jubarte-redlines diff old.docx new.docx --format github
  jubarte-redlines convert contract.docx -o contract.pdf

Run jubarte-redlines <task> --help for task options.
```

#### `jubarte-redlines compare`

```text
$ jubarte-redlines compare --help
Compare documents: print the redline as the agent view, or write it with -o

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
          Write the redline to FILE instead of printing its agent view; the extension
          picks the format: .docx (also any other), .md (the agent view; CriticMarkup
          when both documents are Markdown), .pdf, .png (one file per page) or .json
          (the snapshot)

  -a, --author <NAME>
          Author name recorded on the revisions

          [default: Jubarte]

  -d, --datetime <ISO8601>
          Revision timestamp (ISO 8601) [default: now, UTC]; pin it for reproducible
          output

      --force
          Overwrite the output file if it already exists

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

  -q, --quiet
          No status on success: no `wrote` lines, SUCCESS block or edit summary. Files
          are still written; a reading task (`read`, `changes`, `compare` without -o,
          `diff`, `capabilities`) still prints what it read, and warnings and errors
          still print. `read`, `changes` and `comments` drop their stderr validity
          status (JSON under `changes --json` and `comments --json`). `read --json` and
          `read --for-edit` keep validity counts in their output, -q or not

  -h, --help
          Print help (see a summary with '-h')

Markdown inputs:
      --reference-doc <FILE>
          Two Markdown documents written as Word take styles, page setup, headers and
          footers from this .docx

      --critic
          Read CriticMarkup in the Markdown documents as tracked changes. By default a
          document compared is text

      --resource-path <DIR>
          Where images named by the Markdown are found (default: the original's
          directory when it is Markdown, else the modified's). Only images that resolve
          inside that directory are read, symlinks resolved; a URL or a path out of it
          is written as its alt text, with a warning that says why

Rendering (-o FILE.pdf or FILE.png):
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

      --dpi <DPI>
          PNG resolution in dots per inch (1-1200)

          [default: 96]

      --move-comments
          List the comments after the last page instead of in balloons beside the text.
          The commented text keeps its tint and a `[JR1]` marker, and the pages keep
          their own width

      --changed-pages
          Keep only the pages a change touches: a body or note revision, a comment
          anchor, or the first page on which a revised header or footer paints (the
          first page when none does). Page numbers stay the document's

      --pages <SPEC>
          Only these pages, by the document's page numbers: `3`, `1-3,7`. Layout still
          covers the whole document; the PDF carries these pages and PNGs are named by
          them (`STEM-page-03.png`)

Read options:
      --track-changes <CHOICE>
          Tracked changes inline (all, the default), or the text with every change
          accepted or rejected (also spelled accepted, rejected); the id lines then list
          what changed. --json and --tables read the accepted text, or under reject the
          rejected one, every paragraph keeping its id; the snapshot names an accept or
          a reject in `track_changes`

          Possible values:
          - all:    Keep them: CriticMarkup becomes Word tracked changes and comments
          - accept: Accept every change
          - reject: Reject every change

      --comments <MODE>
          Comments inline (default) or hidden, with their ids on the id line of the
          paragraph that holds them; ignored by --json, --for-edit and the --tables
          grids

          Possible values:
          - inline: Comments inline, with their ids
          - none:   Comments hidden; their ids on the id lines

          [default: inline]

      --dates
          Timestamps on the notes of an author whose changes do not all share one (the
          header shows an author's single timestamp); ignored by --json, --for-edit and
          the --tables grids

      --no-page-markers
          Skip the layout pass; page count from Word's cached breaks, no `<!-- page N of
          M -->` lines; ignored by --json, --for-edit and the --tables grids

  -p, --paragraphs <SPEC>
          Only these blocks: `p5`, `p4-p7`, `t0`, comma-separated (open-ended `p17-` /
          `-p3` spellings still parse, transitionally; the 17 October 2026 contract
          retires them — spell the end, `p17-p20`). With --tables, select top-level
          tables (including their nested tables) by the same tN as the view, or by a pN
          inside them; a pick that reaches no table is refused. Print their grid, then a
          blank line and the same view as read -p SPEC. Repeat it to add blocks: `-p p3
          -p p8` reads as `-p p3,p8`

          [alias: --select]

      --head <N>
          Only the first N blocks (a table is one block)

      --tail <N>
          Only the last N blocks

      --changed
          Only the blocks with a tracked change or a comment

      --by <AUTHOR>
          With --changed: only the blocks with this author's marks (a handle such as AC,
          or the full name)

      --json
          Print the snapshot as JSON instead of the view: `schema_version`,
          `source_sha256`, `summary`, `paragraphs`, `stories`, `tables`, `controls`,
          `changes` and `comments`. -p, --head, --tail, --changed and --by keep the body
          paragraphs they select, and every change and comment that starts in one of
          them, whoever made it; the rest of the snapshot stays whole. --track-changes
          accept or reject settles the text

      --tables
          Print each top-level body table as a grid: a `table N: ROWSxCOLS header_rows=H
          widths=W,...` line, then one line per row of tab-separated `ids=text` cells.
          Use -p t0,t2 or -p pN (a cell paragraph) to select top-level tables and all
          their nested tables, in document order, each once. tN is the view's top-level
          index; nested entries follow their cell row, indented two spaces per level,
          labelled `nested in t0.row2.cell0` (recursively, e.g.
          `t0.row2.cell0.t0.row0.cell1`). A pN or a range selects the tables that hold
          its paragraphs; one that reaches no table is refused. With -p, follow the
          selected grids with a blank line and the same view as read -p SPEC, including
          its YAML header. Without -p, print only the inventory. --track-changes reject
          reads the rejected text

      --for-edit
          The working document template/apply derive, as JSON: source hash, paired
          target ids, exact logical working text with `before_sha256` and readiness per
          operation and range. The selection (-p/--select, --head, --tail, --changed,
          --by, --find, --regex) resolves on the original source and freezes before the
          decisions settle it. Given filters narrow together: a paragraph is selected
          only when every filter given matches it, and an empty result names the filters
          applied in `coverage.omissions`

      --accept-change <ID>
          --for-edit: the working view after accepting this revision (repeatable; it
          does not modify FILE)

      --reject-change <ID>
          --for-edit: the working view after rejecting this revision (repeatable; it
          does not modify FILE)

      --existing-revisions <MODE>
          --for-edit: the remaining-revision policy of the working basis (default keep,
          matching new plans)

          Possible values:
          - auto:   `keep` when the file has tracked changes, else the comparer path
          - keep:   Keep them; the new edits are tracked beside them
          - accept: Accept them first
          - reject: Reject them first
          - refuse: Refuse a file that has them

      --track <CHOICE>
          --for-edit: readiness for tracked (on, the default) or direct (off) new edits

          Possible values:
          - on:  The new edits are tracked
          - off: The new edits land directly

      --find <TEXT>
          Print matching body paragraphs only, with the same Markdown and id lines as
          -p. Searches original final logical text, as --for-edit does; combined filters
          narrow. Like -p, the view cannot select paragraphs in headers, footers,
          footnotes, endnotes and comments. Use --for-edit for header/footer/note
          paragraphs. No match prints nothing and reports it on stderr

      --regex <PATTERN>
          Search with a regular expression instead of literal text (view or --for-edit).
          Give PATTERN here and omit --find

      --ignore-case
          Case-insensitive --find/--regex (view or --for-edit)

      --context <N>
          --for-edit: this many read-only original-source neighbours around each row

      --limit <N>
          --for-edit: primary records a page returns (default 100)

      --max-chars <N>
          --for-edit: the text budget a page shares (default 24000); a truncated row's
          preview is never complete text

      --cursor <TOKEN>
          --for-edit: continue an earlier bounded query (bound to its source, selection,
          filters and working basis)

      --all
          --for-edit: no presentation limits (admission limits still hold)

Examples:
  jubarte compare old.docx new.docx                 the redline's agent view
  jubarte compare old.docx new.docx -o redline.docx
  jubarte compare old.docx new.docx -o redline.pdf --changed-pages
  jubarte compare old.md new.md -o changes.md       CriticMarkup
  jubarte compare -b old.docx -m new.docx --author Legal -o r.docx
```

#### `jubarte-redlines changes`

```text
$ jubarte-redlines changes --help
List tracked changes with IDs for accept, reject and edit plans.

One line per logical change: the marks Word wrote for one edit are one change with a
joined id (`r7+8+9+10`), its text followed by `(N elements)`. A revision inside a
comment body is `comments.r9`.

Usage: jubarte-redlines changes [OPTIONS] <FILE>

Arguments:
  <FILE>
          The document (.docx)

Options:
      --json
          Emit one record per line on stdout (records only). The input_validity status
          is one JSON line on stderr; -q silences it

      --author <NAME>
          Only listed changes by this author (exact match); repeat for several

  -q, --quiet
          No status on success: no `wrote` lines, SUCCESS block or edit summary. Files
          are still written; a reading task (`read`, `changes`, `compare` without -o,
          `diff`, `capabilities`) still prints what it read, and warnings and errors
          still print. `read`, `changes` and `comments` drop their stderr validity
          status (JSON under `changes --json` and `comments --json`). `read --json` and
          `read --for-edit` keep validity counts in their output, -q or not

  -h, --help
          Print help (see a summary with '-h')
```

#### `jubarte-redlines accept`

```text
$ jubarte-redlines accept --help
Accept all tracked changes, or select by ID, author or kind.

Every part counts: the body, headers, footers, notes and comment bodies. A comment-body
revision is `comments.r9`, the id `changes` lists, and an id, author or kind selection
settles it.

Usage: jubarte-redlines accept [OPTIONS] <FILE>

Arguments:
  <FILE>
          The document (.docx) whose revisions to accept

Options:
      --in-place
          Replace FILE through the same atomic write as -o. A refused resolution leaves
          FILE unchanged. Not with -o

  -o, --output <FILE>
          Output path; its extension picks the format: .docx (also any other), .md (the
          agent view), .pdf, .png (one file per page) or .json (the snapshot)

      --force
          Overwrite the output file if it already exists

      --keep-nested-comment
          Keep comments and replies whose whole anchor is removed, on an empty anchor

      --id <ID>
          Only this change: `r12` (as the view and `jubarte changes` print it),
          `header1.r1`, `comments.r9`, or `r1+2` (both). Repeatable

      --author <NAME>
          Only changes by this author. Repeatable

      --kind <KIND>
          Only changes of this kind. Repeatable

          Possible values:
          - insertion:  Inserted text or structural elements
          - deletion:   Deleted text or structural elements
          - move:       Content moved between document locations
          - formatting: Changes to text or paragraph formatting

  -q, --quiet
          No status on success: no `wrote` lines, SUCCESS block or edit summary. Files
          are still written; a reading task (`read`, `changes`, `compare` without -o,
          `diff`, `capabilities`) still prints what it read, and warnings and errors
          still print. `read`, `changes` and `comments` drop their stderr validity
          status (JSON under `changes --json` and `comments --json`). `read --json` and
          `read --for-edit` keep validity counts in their output, -q or not

  -h, --help
          Print help (see a summary with '-h')

Rendering (-o FILE.pdf or FILE.png):
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

      --dpi <DPI>
          PNG resolution in dots per inch (1-1200)

          [default: 96]

      --move-comments
          List the comments after the last page instead of in balloons beside the text.
          The commented text keeps its tint and a `[JR1]` marker, and the pages keep
          their own width

      --changed-pages
          Keep only the pages a change touches: a body or note revision, a comment
          anchor, or the first page on which a revised header or footer paints (the
          first page when none does). Page numbers stay the document's

      --pages <SPEC>
          Only these pages, by the document's page numbers: `3`, `1-3,7`. Layout still
          covers the whole document; the PDF carries these pages and PNGs are named by
          them (`STEM-page-03.png`)
```

#### `jubarte-redlines reject`

```text
$ jubarte-redlines reject --help
Reject all tracked changes, or select by ID, author or kind.

Every part counts: the body, headers, footers, notes and comment bodies. A comment-body
revision is `comments.r9`, the id `changes` lists, and an id, author or kind selection
settles it.

Usage: jubarte-redlines reject [OPTIONS] <FILE>

Arguments:
  <FILE>
          The document (.docx) whose revisions to reject

Options:
      --in-place
          Replace FILE through the same atomic write as -o. A refused resolution leaves
          FILE unchanged. Not with -o

  -o, --output <FILE>
          Output path; its extension picks the format: .docx (also any other), .md (the
          agent view), .pdf, .png (one file per page) or .json (the snapshot)

      --force
          Overwrite the output file if it already exists

      --keep-nested-comment
          Keep comments and replies whose whole anchor is removed, on an empty anchor

      --id <ID>
          Only this change: `r12` (as the view and `jubarte changes` print it),
          `header1.r1`, `comments.r9`, or `r1+2` (both). Repeatable

      --author <NAME>
          Only changes by this author. Repeatable

      --kind <KIND>
          Only changes of this kind. Repeatable

          Possible values:
          - insertion:  Inserted text or structural elements
          - deletion:   Deleted text or structural elements
          - move:       Content moved between document locations
          - formatting: Changes to text or paragraph formatting

  -q, --quiet
          No status on success: no `wrote` lines, SUCCESS block or edit summary. Files
          are still written; a reading task (`read`, `changes`, `compare` without -o,
          `diff`, `capabilities`) still prints what it read, and warnings and errors
          still print. `read`, `changes` and `comments` drop their stderr validity
          status (JSON under `changes --json` and `comments --json`). `read --json` and
          `read --for-edit` keep validity counts in their output, -q or not

  -h, --help
          Print help (see a summary with '-h')

Rendering (-o FILE.pdf or FILE.png):
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

      --dpi <DPI>
          PNG resolution in dots per inch (1-1200)

          [default: 96]

      --move-comments
          List the comments after the last page instead of in balloons beside the text.
          The commented text keeps its tint and a `[JR1]` marker, and the pages keep
          their own width

      --changed-pages
          Keep only the pages a change touches: a body or note revision, a comment
          anchor, or the first page on which a revised header or footer paints (the
          first page when none does). Page numbers stay the document's

      --pages <SPEC>
          Only these pages, by the document's page numbers: `3`, `1-3,7`. Layout still
          covers the whole document; the PDF carries these pages and PNGs are named by
          them (`STEM-page-03.png`)
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
          Output path; its extension picks the format: .docx, .md, .pdf, .png (pages
          named `<stem>-page-NN.png` beside it) or .json (the snapshot of the result).
          Without it: `<stem>.pdf` next to a .docx, `<stem>.docx` next to Markdown,
          `<stem>.json` for `--to json`; Markdown output goes to stdout

      --force
          Overwrite the output file if it already exists

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

      --keep-nested-comment
          Keep comments and replies whose whole anchor is removed; requires
          --track-changes accept or reject

      --no-critic
          Markdown: read `{++`, `{--` and the other CriticMarkup delimiters as text

      --reference-doc <FILE>
          Markdown to Word: take styles, numbering, page setup, headers and footers from
          this .docx (pandoc's --reference-doc)

      --resource-path <DIR>
          Markdown to Word: where images are found [default: the Markdown file's
          directory]. Only images that resolve inside that directory are read, symlinks
          resolved; a URL or a path out of it is written as its alt text, with a warning
          that says why

  -a, --author <NAME>
          Markdown to Word: author of the tracked changes and comments that name none
          [default: the front matter's `author:`, else the --reference-doc's creator,
          else Jubarte]

  -d, --datetime <ISO8601>
          Markdown to Word: timestamp (ISO 8601) of the tracked changes and comments
          that carry none [default: the front matter's
          `last_modified:`/`modified:`/`date:`/`created:`, else the --reference-doc's
          creation time, else now]; pin it (`-d 1970-01-01T00:00:00Z`) for reproducible
          output

      --page <SIZE>
          Markdown to Word: the page size when there is no --reference-doc (one-inch
          margins either way); a reference's page setup wins

          Possible values:
          - letter: US Letter, 8.5 by 11 inches
          - a4:     ISO A4, 210 by 297 mm

          [default: letter]

      --tracking <SWITCH>
          Markdown to Word: turn Word's Track Changes on or off in the .docx written,
          whatever the Markdown's `track_changes:` key says [default: the key; without
          one, the --reference-doc's setting, else off]

          Possible values:
          - on:  New edits in Word are tracked
          - off: New edits in Word are not tracked

      --scholar
          Markdown to Word: set a manuscript as LaTeX's article class does (Latin Modern
          Roman 10pt on 12pt lines, justified, bold headings) and number its pages from
          page 2, centered in the footer. The front matter's own `styles:` entries,
          headers and footers win

      --no-page-markers
          Word to Markdown: leave out the `<!-- page N of M -->` lines, and the layout
          pass that places them

      --commonmark
          Word to Markdown: write plain CommonMark with CriticMarkup for readers outside
          jubarte's dialect, instead of the agent view (`jubarte read`'s YAML header and
          id lines). Fields print as their text, and underline, highlight and strike are
          left out

  -q, --quiet
          No status on success: no `wrote` lines, SUCCESS block or edit summary. Files
          are still written; a reading task (`read`, `changes`, `compare` without -o,
          `diff`, `capabilities`) still prints what it read, and warnings and errors
          still print. `read`, `changes` and `comments` drop their stderr validity
          status (JSON under `changes --json` and `comments --json`). `read --json` and
          `read --for-edit` keep validity counts in their output, -q or not

  -h, --help
          Print help (see a summary with '-h')

Formats:
  -f, --from <FORMAT>
          Input format [default: from the file: .md and .markdown are Markdown, a zip is
          Word]

          [possible values: docx, md, markdown]

  -t, --to <FORMAT>
          Output format [default: from --output, else pdf for Word and docx for
          Markdown]

          Possible values:
          - docx: Word (.docx)
          - md:   Markdown: CommonMark with GitHub tables, task lists and footnotes, and
            CriticMarkup
          - pdf:  PDF, laid out as Word does
          - png:  PNG pages
          - json: The JSON snapshot (`read --json`'s object): written, never read.
            `diff` refuses it (`json_not_yet`)

Rendering (-o FILE.pdf or FILE.png):
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

      --dpi <DPI>
          PNG resolution in dots per inch (1-1200)

          [default: 96]

      --move-comments
          List the comments after the last page instead of in balloons beside the text.
          The commented text keeps its tint and a `[JR1]` marker, and the pages keep
          their own width

      --changed-pages
          Keep only the pages a change touches: a body or note revision, a comment
          anchor, or the first page on which a revised header or footer paints (the
          first page when none does). Page numbers stay the document's

      --pages <SPEC>
          Only these pages, by the document's page numbers: `3`, `1-3,7`. Layout still
          covers the whole document; the PDF carries these pages and PNGs are named by
          them (`STEM-page-03.png`)

      --fail-on-substitution
          Exit 4 when a requested font was substituted (listed on stderr and in
          --report). Every output is still written. Exit status: 0 ok, 1 error, 4 a
          requested font was substituted

      --update-fields
          Refresh the cached results of PAGEREF, REF, NUMPAGES, SEQ and TOC fields from
          jubarte's layout (TOCs rebuilt from the headings) in the .docx written, after
          --track-changes. Field codes stay, so Word can update them again; page numbers
          are jubarte's layout, not Word's (docs/WORD_DIFFERENCES.md). Prints one line
          per field; --report writes `{page_count, fields}`

      --timeout <SECONDS>
          Give up after this many seconds: exit 124 (as `timeout(1)`) with nothing more
          written. An output being written at that moment may be left partial

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
  -q, --quiet
          No status on success: no `wrote` lines, SUCCESS block or edit summary. Files
          are still written; a reading task (`read`, `changes`, `compare` without -o,
          `diff`, `capabilities`) still prints what it read, and warnings and errors
          still print. `read`, `changes` and `comments` drop their stderr validity
          status (JSON under `changes --json` and `comments --json`). `read --json` and
          `read --for-edit` keep validity counts in their output, -q or not

  -h, --help
          Print help (see a summary with '-h')

Review:
  -o, --output <FILE>
          Write the view to FILE (.md, .markdown, .txt, .patch or .diff; stdout
          otherwise)

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

  -a, --author <NAME>
          Who made the changes: the patch's owner

          [default: Jubarte]

  -d, --datetime <ISO8601>
          Revision timestamp (ISO 8601) [default: now, UTC]; pin it for reproducible
          output

Formats:
  -t, --to <FORMAT>
          Output format: md only (the patch, CriticMarkup or a text view are all text)

  -f, --from <FORMAT>
          Input format of both documents [default: from each file]

          [possible values: docx, md, markdown]

Examples:
  jubarte diff old.docx new.docx --format github   Git/GitHub patch on stdout
  jubarte diff old.md new.md                       the patch on stdout
  jubarte diff old.md new.md --format critic       CriticMarkup on stdout, as pandiff
  jubarte diff old.docx new.docx -o changes.patch  the patch in a file
  jubarte compare old.md new.md -o changes.docx    Word tracked changes (compare writes
  redlines)
  jubarte compare contract.docx edited.md -o redline.pdf
      the Markdown's edits painted on the Word document

GIT:
  git config --global difftool.jubarte.cmd 'jubarte diff "$LOCAL" "$REMOTE"'
  git difftool -t jubarte -y -- '*.md'
```

#### `jubarte-redlines read`

```text
$ jubarte-redlines read --help
Read the agent view: YAML header, `<!-- pN -->` id lines, changes and comments with ids

A YAML header, then Markdown with an `<!-- pN -->` id line before every paragraph,
tracked changes as CriticMarkup followed by their ids (`{++text++}{r12 @AC}`) and
comments with theirs (`{>>c5 @AC: …<<}`).

Usage: jubarte-redlines read [OPTIONS] <FILE>

Arguments:
  <FILE>
          The document (.docx) to read

Options:
  -o, --output <FILE>
          Write the view to FILE.md, or the snapshot to FILE.json, instead of printing
          it

      --force
          Overwrite the output file if it already exists

  -q, --quiet
          No status on success: no `wrote` lines, SUCCESS block or edit summary. Files
          are still written; a reading task (`read`, `changes`, `compare` without -o,
          `diff`, `capabilities`) still prints what it read, and warnings and errors
          still print. `read`, `changes` and `comments` drop their stderr validity
          status (JSON under `changes --json` and `comments --json`). `read --json` and
          `read --for-edit` keep validity counts in their output, -q or not

  -h, --help
          Print help (see a summary with '-h')

Read options:
      --track-changes <CHOICE>
          Tracked changes inline (all, the default), or the text with every change
          accepted or rejected (also spelled accepted, rejected); the id lines then list
          what changed. --json and --tables read the accepted text, or under reject the
          rejected one, every paragraph keeping its id; the snapshot names an accept or
          a reject in `track_changes`

          Possible values:
          - all:    Keep them: CriticMarkup becomes Word tracked changes and comments
          - accept: Accept every change
          - reject: Reject every change

      --comments <MODE>
          Comments inline (default) or hidden, with their ids on the id line of the
          paragraph that holds them; ignored by --json, --for-edit and the --tables
          grids

          Possible values:
          - inline: Comments inline, with their ids
          - none:   Comments hidden; their ids on the id lines

          [default: inline]

      --dates
          Timestamps on the notes of an author whose changes do not all share one (the
          header shows an author's single timestamp); ignored by --json, --for-edit and
          the --tables grids

      --no-page-markers
          Skip the layout pass; page count from Word's cached breaks, no `<!-- page N of
          M -->` lines; ignored by --json, --for-edit and the --tables grids

  -p, --paragraphs <SPEC>
          Only these blocks: `p5`, `p4-p7`, `t0`, comma-separated (open-ended `p17-` /
          `-p3` spellings still parse, transitionally; the 17 October 2026 contract
          retires them — spell the end, `p17-p20`). With --tables, select top-level
          tables (including their nested tables) by the same tN as the view, or by a pN
          inside them; a pick that reaches no table is refused. Print their grid, then a
          blank line and the same view as read -p SPEC. Repeat it to add blocks: `-p p3
          -p p8` reads as `-p p3,p8`

          [alias: --select]

      --head <N>
          Only the first N blocks (a table is one block)

      --tail <N>
          Only the last N blocks

      --changed
          Only the blocks with a tracked change or a comment

      --by <AUTHOR>
          With --changed: only the blocks with this author's marks (a handle such as AC,
          or the full name)

      --json
          Print the snapshot as JSON instead of the view: `schema_version`,
          `source_sha256`, `summary`, `paragraphs`, `stories`, `tables`, `controls`,
          `changes` and `comments`. -p, --head, --tail, --changed and --by keep the body
          paragraphs they select, and every change and comment that starts in one of
          them, whoever made it; the rest of the snapshot stays whole. --track-changes
          accept or reject settles the text

      --tables
          Print each top-level body table as a grid: a `table N: ROWSxCOLS header_rows=H
          widths=W,...` line, then one line per row of tab-separated `ids=text` cells.
          Use -p t0,t2 or -p pN (a cell paragraph) to select top-level tables and all
          their nested tables, in document order, each once. tN is the view's top-level
          index; nested entries follow their cell row, indented two spaces per level,
          labelled `nested in t0.row2.cell0` (recursively, e.g.
          `t0.row2.cell0.t0.row0.cell1`). A pN or a range selects the tables that hold
          its paragraphs; one that reaches no table is refused. With -p, follow the
          selected grids with a blank line and the same view as read -p SPEC, including
          its YAML header. Without -p, print only the inventory. --track-changes reject
          reads the rejected text

      --for-edit
          The working document template/apply derive, as JSON: source hash, paired
          target ids, exact logical working text with `before_sha256` and readiness per
          operation and range. The selection (-p/--select, --head, --tail, --changed,
          --by, --find, --regex) resolves on the original source and freezes before the
          decisions settle it. Given filters narrow together: a paragraph is selected
          only when every filter given matches it, and an empty result names the filters
          applied in `coverage.omissions`

      --accept-change <ID>
          --for-edit: the working view after accepting this revision (repeatable; it
          does not modify FILE)

      --reject-change <ID>
          --for-edit: the working view after rejecting this revision (repeatable; it
          does not modify FILE)

      --existing-revisions <MODE>
          --for-edit: the remaining-revision policy of the working basis (default keep,
          matching new plans)

          Possible values:
          - auto:   `keep` when the file has tracked changes, else the comparer path
          - keep:   Keep them; the new edits are tracked beside them
          - accept: Accept them first
          - reject: Reject them first
          - refuse: Refuse a file that has them

      --track <CHOICE>
          --for-edit: readiness for tracked (on, the default) or direct (off) new edits

          Possible values:
          - on:  The new edits are tracked
          - off: The new edits land directly

      --find <TEXT>
          Print matching body paragraphs only, with the same Markdown and id lines as
          -p. Searches original final logical text, as --for-edit does; combined filters
          narrow. Like -p, the view cannot select paragraphs in headers, footers,
          footnotes, endnotes and comments. Use --for-edit for header/footer/note
          paragraphs. No match prints nothing and reports it on stderr

      --regex <PATTERN>
          Search with a regular expression instead of literal text (view or --for-edit).
          Give PATTERN here and omit --find

      --ignore-case
          Case-insensitive --find/--regex (view or --for-edit)

      --context <N>
          --for-edit: this many read-only original-source neighbours around each row

      --limit <N>
          --for-edit: primary records a page returns (default 100)

      --max-chars <N>
          --for-edit: the text budget a page shares (default 24000); a truncated row's
          preview is never complete text

      --cursor <TOKEN>
          --for-edit: continue an earlier bounded query (bound to its source, selection,
          filters and working basis)

      --all
          --for-edit: no presentation limits (admission limits still hold)
```

#### `jubarte-redlines edit`

```text
$ jubarte-redlines edit --help
Edit a document: -p WHERE with --anchor, --content, --delete, --resolve, --reply,
--accept, --reject, --strike, --restore or --style (several -p per command), or a JSON
plan. Writes FILE_modified.docx (--in-place replaces FILE; --out-dir: the bundle), then
prints SUCCESS and a fenced before/after diff of the full agent views (refusal: exit 3)

Usage: jubarte-redlines edit [OPTIONS] <FILE>

Arguments:
  <FILE>
          The source document (.docx). Left unchanged unless --in-place

Options:
      --plan [<PLAN.json>]
          Edit plan JSON: a list of operations of any kind (see `jubarte capabilities
          --json`). Alone, it is applied to FILE; with operation flags, they are
          recorded into it (created on first use) and FILE is left as it is. Without a
          value: FILE_plan.json beside FILE

      --consume
          Delete PLAN.json once it has been applied and the outputs are written (with
          --plan and no operation flags). Kept when --skip-failed left operations out

  -p, --location <WHERE>
          Where: p12, header1, footer2.p1, t0.row1.cell2, c5 (a comment), c5.text (the
          text c5 covers) or r12 (a revision, with --accept, --reject, --strike or
          --restore). Each -p starts an operation; the flags after it belong to it

      --anchor <TEXT>
          Text inside WHERE the operation applies to (must occur once, unless
          --occurrence picks a match)

      --occurrence <N>
          Pick the Nth anchor match (1-based), after --anchor in this -p group

      --content <TEXT>
          New text: replaces the anchor; rewrites the paragraph without one; on c5, the
          comment's new text

      --delete
          Delete the anchor, or the whole paragraph (or comment) without one

      --resolve
          Resolve comment c5 (with -p c5)

      --reply <TEXT>
          Reply to comment c5 (with -p c5), as `add -p c5 --content`

      --keep-others
          Refuse to accept or reject another author's tracked change, and to delete,
          resolve, reopen or reword their comment. Restore, strike, a comment and a
          reply stay allowed. JUBARTE_KEEP_OTHERS=1 sets the same guard. Another author
          is anyone other than --author (Jubarte when --author is left out)

      --accept
          Accept revision r12 (with -p r12; r1+2 for several); prints the resolved
          change (`accepted r1+2 in p3: "from" > "to"`)

      --reject
          Reject revision r12 (with -p r12; r1+2 for several); prints the resolved
          change (`rejected r0 in p3: "from" > "to"`)

      --strike
          Strike another author's insertion r12 (with -p r12; r10+11 for a change of
          several marks, a replacement's insertion only): our tracked deletion inside
          it, so the decision shows. --content is one comment on the struck words

      --restore
          Restore another author's deletion r12 (with -p r12; r10+11 for a change of
          several marks, a replacement's deletion only): their deletion stays and our
          tracked insertion of its words follows it. --content is one comment on the
          restored words

      --visible
          With --reject: strike each insertion and restore each deletion instead of
          removing the change without a trace

      --style <SPEC>
          Formatting for the anchored text: bold, italic, underline, strike, caps,
          highlight=yellow, font=Calibri, size=11, color=FF0000; any other value is a
          paragraph style (Heading2)

  -a, --author <NAME>
          Author of the changes and comments (operation flags; a plan names its own). A
          write that adds markup needs it spelled here or in JUBARTE_AUTHOR

          [default: Jubarte]

  -d, --datetime <ISO8601>
          Revision timestamp (ISO 8601) [default: now, UTC]; pin it for reproducible
          output

      --suggesting-mode
          The edits are tracked changes (the default): the file written is the redline
          and the view shows the marks

      --editing-mode
          The edits land directly: the file written is the result, and the view shows it
          with rev tags on the id lines

  -q, --quiet
          No status on success: no `wrote` lines, SUCCESS block or edit summary. Files
          are still written; a reading task (`read`, `changes`, `compare` without -o,
          `diff`, `capabilities`) still prints what it read, and warnings and errors
          still print. `read`, `changes` and `comments` drop their stderr validity
          status (JSON under `changes --json` and `comments --json`). `read --json` and
          `read --for-edit` keep validity counts in their output, -q or not

      --existing-revisions <MODE>
          What to do when FILE already holds tracked changes: auto keeps them and tracks
          the new edits beside them; a clean file goes through the comparer

          Possible values:
          - auto:   `keep` when the file has tracked changes, else the comparer path
          - keep:   Keep them; the new edits are tracked beside them
          - accept: Accept them first
          - reject: Reject them first
          - refuse: Refuse a file that has them

          [default: auto]

      --in-place
          Replace FILE through the same atomic write as -o. A refused edit or a failed
          check leaves FILE unchanged. Not with -o or --out-dir

  -o, --output <FILE.docx>
          The edited document, a .docx (default: `<FILE's
          directory>/<stem>_modified.docx`). Any other extension is written as a .docx
          too; .md, .json, .pdf and .png name the task that writes them

      --out-dir [<DIR>]
          Write the bundle instead: clean.docx, redline.docx, patch.diff and
          report.jsonl (clean.docx and report.jsonl in editing mode) in DIR (default:
          `<FILE's directory>/<stem>.edit`)

      --force
          Replace an existing output (the bundle directory's files)

      --dry-run
          Resolve and report only; write nothing. When every operation passes, stderr
          ends with the command that applies it (`next:`)

      --skip-failed
          Apply every operation that passes; leave out the ones that fail and every
          operation of their "group". Exit 0 when one landed. The failed ones go to
          PLAN.failed.json (FILE_plan.failed.json for operation flags): a plan to fix
          and run on the output

      --require-change
          Refuse when the edit would change nothing in the document
          (NO_CHANGES_REQUESTED); reports, renders and bundles do not count as a change

      --pdf
          Also write the PDF beside the document (the bundle: redline.pdf and clean.pdf)

      --png
          Also write PNG pages beside the document, STEM-page-NN.png (the bundle:
          redline-page-NN.png and clean-page-NN.png)

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

Examples:
  jubarte edit a.docx -p p12 --anchor "thirty days" --content "forty-five days" -a NAME
  jubarte edit a.docx -p p12 --anchor thirty --content "thirty (30)" -a NAME   an
  insertion
  jubarte edit a.docx -p p7 --anchor "at its sole discretion" --delete -p p9 --delete -a
  NAME
  jubarte edit a.docx -p p3 --content "The parties agree as follows." -a NAME    rewrite
  jubarte edit a.docx -p p4 --anchor Fees --style bold -p p5 --style Heading2 -a NAME
  jubarte edit a.docx -p c5 --content "Agreed." -p c7 --resolve -a NAME
  jubarte edit a.docx -p c5 --reply "Agreed, will fix." -a NAME
  jubarte edit a.docx -p c5.text --content "net 30" -a NAME   the text c5 covers
  jubarte edit a.docx -p r3 --accept -p header1.r1 --reject
  jubarte edit a.docx -p p3 --delete -o b.docx --pdf -a NAME   b.docx and b.pdf
  jubarte edit a.docx --plan plan.json --out-dir review   the bundle in review/
  jubarte edit a.docx --plan plan.json -p p3 --content "New."   record into plan.json
  jubarte edit a.docx --plan plan.json --consume   apply plan.json, then delete it

WHERE is an id from `jubarte read`: p12, header1, footer2.p1,
t0.row1.cell2, c5 (a comment), c5.text (the text it covers) or r12
(a revision).

A plan (--plan plan.json) is a list of operations, applied all or none;
`jubarte capabilities --json` lists every operation kind:
  {"schema_version": 1, "source_sha256": "<sha256 from jubarte read>", "author":
  "Claude",
   "operations": [
    {"kind": "replace", "paragraph": "p1", "find": "his or her", "replacement":
    "their"},
    {"kind": "delete_paragraph", "paragraph": "p2"}]}

Desired-text plans (PlanV2), a separate workflow:
  jubarte edit template FILE --select p12,p20-p25 -o edits.json   the paragraphs as they
  read
  (edit the desired text in edits.json)
  jubarte edit apply FILE --plan edits.json -o OUTPUT.docx        tracked changes for
  the difference

Typed actions: jubarte edit replace|rewrite|insert|delete FILE [OPTIONS]
  Inline --save-plan FILE saves without applying; writes require -o or --out-dir.
  Run jubarte edit ACTION --help for its grammar.
Routing: valid legacy calls, including `edit replace -p p0 --content TEXT`,
  retain v1 semantics. Use ./replace (or ./rewrite, ./insert, ./delete, ./apply)
  to explicitly name a legacy input whose name equals an action.
```

#### `jubarte-redlines add`

```text
$ jubarte-redlines add --help
Add a paragraph, a comment or a reply: -p WHERE --content TEXT (several -p per command).
Writes FILE_modified.docx, as edit does (--in-place replaces FILE), and prints SUCCESS
and a fenced before/after diff of the full agent views

Usage: jubarte-redlines add [OPTIONS] <FILE>

Arguments:
  <FILE>
          The source document (.docx). Left unchanged unless --in-place

Options:
      --plan [<PLAN.json>]
          Edit plan JSON. With operation flags, they are recorded into it (created on
          first use) and FILE is left as it is; alone, it is applied to FILE, as `edit
          --plan` does. Without a value: FILE_plan.json beside FILE

      --consume
          Delete PLAN.json once it has been applied and the outputs are written (with
          --plan and no operation flags)

  -p, --location <WHERE>
          Where: p12, header1, footer2.p1, t0.row1.cell2, or c5 for a reply. Each -p
          starts an operation; the flags after it belong to it

      --anchor <TEXT>
          Text inside WHERE to comment on (must occur once, unless --occurrence picks a
          match)

      --occurrence <N>
          Pick the Nth anchor match (1-based), after --anchor in this -p group

      --content <TEXT>
          The paragraph, comment or reply text

      --reply <TEXT>
          Reply to comment c5 (with -p c5); the same as --content there

      --keep-others
          Refuse to accept or reject another author's tracked change, and to delete,
          resolve, reopen or reword their comment, when this command applies or records
          a plan. Restore, strike, a comment and a reply stay allowed.
          JUBARTE_KEEP_OTHERS=1 sets the same guard. Another author is anyone other than
          --author (Jubarte when --author is left out)

      --before
          A new paragraph before WHERE instead of after it

      --comment
          A comment on the whole paragraph (with --anchor the comment sits on the anchor
          and this flag is implied)

      --style <SPEC>
          Formatting for the new paragraph's text (bold, italic, underline,
          highlight=yellow) or its paragraph style (Heading2)

  -a, --author <NAME>
          Author of the changes and comments (operation flags; a plan names its own). A
          write that adds markup needs it spelled here or in JUBARTE_AUTHOR

          [default: Jubarte]

  -d, --datetime <ISO8601>
          Revision timestamp (ISO 8601) [default: now, UTC]; pin it for reproducible
          output

      --suggesting-mode
          The edits are tracked changes (the default): the file written is the redline
          and the view shows the marks

      --editing-mode
          The edits land directly: the file written is the result, and the view shows it
          with rev tags on the id lines

      --existing-revisions <MODE>
          What to do when FILE already holds tracked changes: auto keeps them and tracks
          the new edits beside them; a clean file goes through the comparer

          Possible values:
          - auto:   `keep` when the file has tracked changes, else the comparer path
          - keep:   Keep them; the new edits are tracked beside them
          - accept: Accept them first
          - reject: Reject them first
          - refuse: Refuse a file that has them

          [default: auto]

      --in-place
          Replace FILE through the same atomic write as -o. A refused edit or a failed
          check leaves FILE unchanged. Not with -o or --out-dir

  -o, --output <FILE.docx>
          The edited document, a .docx (default: `<FILE's
          directory>/<stem>_modified.docx`). Any other extension is written as a .docx
          too; .md, .json, .pdf and .png name the task that writes them

      --out-dir [<DIR>]
          Write the bundle instead: clean.docx, redline.docx, patch.diff and
          report.jsonl (clean.docx and report.jsonl in editing mode) in DIR (default:
          `<FILE's directory>/<stem>.edit`)

      --force
          Replace an existing output (the bundle directory's files)

  -q, --quiet
          No status on success: no `wrote` lines, SUCCESS block or edit summary. Files
          are still written; a reading task (`read`, `changes`, `compare` without -o,
          `diff`, `capabilities`) still prints what it read, and warnings and errors
          still print. `read`, `changes` and `comments` drop their stderr validity
          status (JSON under `changes --json` and `comments --json`). `read --json` and
          `read --for-edit` keep validity counts in their output, -q or not

  -h, --help
          Print help (see a summary with '-h')

Examples:
  jubarte add a.docx -p p12 --content "Time is of the essence." -a NAME   new paragraph
  after p12
  jubarte add a.docx -p p12 --content Recitals --before --style Heading2 -a NAME
  jubarte add a.docx -p p5 --anchor "monthly fee" --content "Net of taxes?" -a NAME   a
  comment
  jubarte add a.docx -p p5 --comment --content "Whole clause needs a cap." -a NAME
  jubarte add a.docx -p c5 --content "Agreed, will fix." -a NAME           a reply
  jubarte add a.docx -p c5 --reply "Agreed, will fix." -a NAME             the same
  reply
  jubarte add a.docx --plan plan.json -p p5 --comment --content "Cap?"   record into
  plan.json
```

#### `jubarte-redlines capabilities`

```text
$ jubarte-redlines capabilities --help
What this binary can do, for agents choosing an operation

Usage: jubarte-redlines capabilities [OPTIONS]

Options:
      --json                   The manifest or focused answer as JSON instead of text
      --operation <OPERATION>  Discover one edit kind, including its fields and
                               execution conditions
      --field <FIELD>          Discover one field, optionally within --operation
      --matrix                 Publish operation × target × policy edit support
  -q, --quiet                  No status on success: no `wrote` lines, SUCCESS block or
                               edit summary. Files are still written; a reading task
                               (`read`, `changes`, `compare` without -o, `diff`,
                               `capabilities`) still prints what it read, and warnings
                               and errors still print. `read`, `changes` and `comments`
                               drop their stderr validity status (JSON under `changes
                               --json` and `comments --json`). `read --json` and `read
                               --for-edit` keep validity counts in their output, -q or
                               not
  -h, --help                   Print help
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
