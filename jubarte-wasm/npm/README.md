# jubarte-wasm

Word-mode **DOCX redline** engine, compiled to WebAssembly.

This package is the WASM binding of
[**jubarte-redlines**](https://github.com/jandira-tech/jubarte-redlines) — a
lossless, Word-compatible tracked-changes engine written in Rust. It compares
two `.docx` files and produces a redline `.docx` with native Word revisions
(`w:ins` / `w:del`), the same output model Microsoft Word itself uses. It can
also accept or reject all tracked revisions in a document, list them as
JSON, render any DOCX to PDF (Word-style layout, embedded
Carlito/Liberation fonts), and let an agent read a document by paragraph id
and apply an edit plan that returns a clean copy, a Word redline and a
per-operation report.

Everything runs in-process — no Word, no LibreOffice, no server round-trip.
Ships prebuilt binaries for **Node** (CommonJS, auto-initializing) and the
**browser / bundlers** (ES module with explicit init), each in two flavors:
the **full** build (compare + PDF, ~10 MB wasm) and a **slim** build
(everything except PDF rendering, ~2.4 MB wasm) for
bundle-size-sensitive deployments.

## Install

```bash
npm install jubarte-wasm
```

## Usage — Node

The Node build initializes automatically on require/import:

```js
const { compareDocuments, initPanicHook } = require("jubarte-wasm");
// or: import { compareDocuments, initPanicHook } from "jubarte-wasm";
const fs = require("node:fs");

initPanicHook(); // optional: route wasm panics to console.error

const original = fs.readFileSync("original.docx");
const modified = fs.readFileSync("modified.docx");

const redline = compareDocuments(original, modified, "Author Name");
fs.writeFileSync("redline.docx", redline); // opens clean in Microsoft Word
```

## Usage — browser / bundlers

The web build is an ES module with an explicit async init:

```js
import init, { compareDocuments, initPanicHook } from "jubarte-wasm/web";

await init(); // fetches jubarte_wasm_bg.wasm relative to the module URL
initPanicHook();

const redline = compareDocuments(originalBytes, modifiedBytes, "Author Name");
// redline: Uint8Array — serve it as a .docx download
```

Vite, webpack 5, and other bundlers that understand
`new URL("...", import.meta.url)` will bundle the `.wasm` file automatically.
You can also pass the wasm source yourself: `await init({ module_or_path: url })`.

## Slim builds (no PDF)

If you don't need PDF rendering, the slim entry points drop `docxToPdf` /
`pdfPageCount` — and with them the PDF engine and its embedded
Carlito/Liberation fonts — shrinking the wasm from ~10 MB to ~2.4 MB.
Redline output carries the same parts and revisions as the full build.

```js
const { compareDocuments } = require("jubarte-wasm/slim");   // Node
import init, { compareDocuments } from "jubarte-wasm/web-slim"; // browser
```

| Entry point | Build | Contents |
|---|---|---|
| `jubarte-wasm` / `jubarte-wasm/node` | full, Node CJS | all functions |
| `jubarte-wasm/web` | full, browser ESM | all functions |
| `jubarte-wasm/slim` / `jubarte-wasm/node-slim` | slim, Node CJS | everything except `docxToPdf` / `pdfPageCount` (so also `inspectDocument`, `documentMarkdown`, `sourceSha256`, `applyEditPlan`, `previewEditPlan`, `editReportJsonl`, `capabilities`) |
| `jubarte-wasm/web-slim` | slim, browser ESM | everything except `docxToPdf` / `pdfPageCount` (same export set as `node-slim`) |

## API

Document parameters and returns are `Uint8Array` holding complete `.docx`
(or, for `docxToPdf`, `.pdf`) packages; `getRevisions` returns a JSON
`string`, `pdfPageCount` a `number`, and `initPanicHook` returns nothing.

| Function | Signature | Description |
|---|---|---|
| `compareDocuments` | `(original, modified, author) → Uint8Array` | Compare two DOCX files → redline DOCX with tracked changes attributed to `author`. |
| `acceptRevisions` | `(docx) → Uint8Array` | Accept every tracked revision → clean DOCX. |
| `rejectRevisions` | `(docx) → Uint8Array` | Reject every tracked revision → base DOCX. |
| `getRevisions` | `(docx) → string` | List tracked revisions as a JSON array string (`type` / `author` / `date` / `part` / `moveGroupId` / `isMoveSource` / `formatChange` / `text`). |
| `listChanges` | `(docx) → string` | Each tracked change as a JSON array string: `id` (`body:rev:12`), `kind`, `target`, `author`, `date`, `text`, `move_name`, `move_side`, `inside`. |
| `acceptChanges` | `(docx, filterJson) → Uint8Array` | Accept the changes `filterJson` selects (`{"ids": [...], "authors": [...], "kinds": [...]}`: every list given must match; `{}` selects all, an empty list none) and keep the rest tracked. |
| `rejectChanges` | `(docx, filterJson) → Uint8Array` | Reject the changes `filterJson` selects and keep the rest tracked. |
| `docxToPdf` | `(docx, compress?, revisions?, revision_palette?) → Uint8Array` | Render a DOCX → PDF (Word-style layout). `compress` (default `false`) deflates the PDF's streams; `revisions` (default `"conventional"`) paints tracked changes — `"conventional"`, `"word"`, or `"custom"` with `revision_palette` (`"deleted=#AA0000:strike,..."`). Fonts come from the embedded Carlito/Liberation set. *Full builds only.* |
| `pdfPageCount` | `(pdf) → number` | Page count of a PDF (`0` if the bytes are not a readable PDF). *Full builds only.* |
| `initPanicHook` | `() → void` | Route wasm panics to `console.error`. Safe to call multiple times. |
| `inspectDocument` | `(docx) → string` | Inspection snapshot as JSON: `source_sha256`, `summary`, `paragraphs` with `body:p:N` ids, text, style, formatting spans and limitations, `stories` (headers, footers, notes) with `header1:p:N`-style ids, and `tables` (each body table's cells with their paragraph ids and text, `header_rows`, `widths_dxa`). |
| `documentMarkdown` | `(docx) → string` | Body, then every header, footer and notes story, as Markdown with a `[body:p:N]` / `[header1:p:N]` id before every paragraph. |
| `sourceSha256` | `(docx) → string` | SHA-256 of the bytes: the `source_sha256` guard an edit plan carries. |
| `applyEditPlan` | `(docx, planJson) → EditOutput` | Apply an edit plan (`replace`, `insert`, `delete`, `comment`, `insert_paragraph`, `delete_paragraph`, `format_paragraph`, `merge_paragraphs`, `rewrite`, `insert_table`, `list`; `replace`/`insert` take an optional run `format`; `replace` takes `whole: true` for one deletion then one insertion; `insert_paragraph` takes `like` to copy another paragraph's properties; `delete_paragraph` takes a `comment` on the deleted text). `ok`, `clean`, `redline`, and `json` (the report, or the refusal with `code` and every operation's outcome). |
| `previewEditPlan` | `(docx, planJson) → EditOutput` | Resolve every operation without producing documents. |
| `editReportJsonl` | `(reportJson) → string` | A report as JSON lines (`load`, one `op` per operation, `summary`). |
| `capabilities` | `() → string` | What this build can do, as JSON (`runtime: "wasm"`, operations, edit kinds, input budgets). |

Errors (invalid/corrupt DOCX, unsupported constructs) are thrown as JS
exceptions with a `jubarte-wasm: …` message. Edit plans are the exception:
a refused plan is returned as data (`ok: false`) so its per-operation
outcomes stay readable.

`inspectDocument`, `documentMarkdown` and the edit functions refuse a
package before parsing it when it exceeds the input budgets `capabilities`
reports (64 MiB file, 10,000 entries, 64 MiB per inflated part, 256 MiB in
total, XML nesting 256) or is not a Word package. The refusal codes are
`INPUT_LIMIT`, `DUPLICATE_PART`, `UNSUPPORTED_PACKAGE`, `INVALID_PACKAGE`
and `INVALID_XML`.

```js
const { inspectDocument, applyEditPlan } = require("jubarte-wasm");
const snap = JSON.parse(inspectDocument(bytes));
const out = applyEditPlan(bytes, JSON.stringify({
  schema_version: 1,
  source_sha256: snap.source_sha256,
  author: "Reviewer",
  operations: [{ kind: "replace", paragraph: { id: "body:p:3" }, find: "30 days", replacement: "45 days" }],
}));
if (out.ok) { writeFileSync("redline.docx", out.redline); }
else { console.error(JSON.parse(out.json).code); }
```

## Versioning

The package version tracks the embedded **jubarte-redlines** engine version;
adapter-only releases (new bindings over the same engine) bump the patch
level past it.
The exact engine commit each build was produced from is recorded in
`ENGINE_COMMIT.txt` inside the package.

## License

[AGPL-3.0-only](https://www.gnu.org/licenses/agpl-3.0.html) ©
Jandira Technologies, LLC. If AGPL does not fit your use case, contact the
authors about commercial licensing via the
[GitHub repository](https://github.com/jandira-tech/jubarte-redlines).
