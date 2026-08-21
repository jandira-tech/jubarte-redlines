# jubarte-wasm

Word-mode **DOCX redline** engine, compiled to WebAssembly.

This package is the WASM binding of
[**jubarte-redlines**](https://github.com/jandira-tech/jubarte-redlines) — a
lossless, Word-compatible tracked-changes engine written in Rust. It compares
two `.docx` files and produces a redline `.docx` with native Word revisions
(`w:ins` / `w:del`), the same output model Microsoft Word itself uses. It can
also accept or reject all tracked revisions in a document, list them as
JSON, and render any DOCX to PDF (Word-style layout, embedded
Carlito/Liberation fonts).

Everything runs in-process — no Word, no LibreOffice, no server round-trip.
Ships prebuilt binaries for **Node** (CommonJS, auto-initializing) and the
**browser / bundlers** (ES module with explicit init), each in two flavors:
the **full** build (compare + PDF, ~10 MB wasm) and a **slim** build
(compare-only, ~2.4 MB wasm) for bundle-size-sensitive deployments.

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

## Slim builds (compare-only)

If you don't need PDF rendering, the slim entry points drop `docxToPdf` /
`pdfPageCount` — and with them the PDF engine and its embedded
Carlito/Liberation fonts — shrinking the wasm from ~10 MB to ~2.4 MB.
Redline output is byte-identical to the full build.

```js
const { compareDocuments } = require("jubarte-wasm/slim");   // Node
import init, { compareDocuments } from "jubarte-wasm/web-slim"; // browser
```

| Entry point | Build | Contents |
|---|---|---|
| `jubarte-wasm` / `jubarte-wasm/node` | full, Node CJS | all functions |
| `jubarte-wasm/web` | full, browser ESM | all functions |
| `jubarte-wasm/slim` / `jubarte-wasm/node-slim` | slim, Node CJS | compare/accept/reject/list only |
| `jubarte-wasm/web-slim` | slim, browser ESM | compare/accept/reject/list only |

## API

All byte parameters and returns are `Uint8Array` holding complete `.docx`
packages.

| Function | Signature | Description |
|---|---|---|
| `compareDocuments` | `(original, modified, author) → Uint8Array` | Compare two DOCX files → redline DOCX with tracked changes attributed to `author`. |
| `acceptRevisions` | `(docx) → Uint8Array` | Accept every tracked revision → clean DOCX. |
| `rejectRevisions` | `(docx) → Uint8Array` | Reject every tracked revision → base DOCX. |
| `getRevisions` | `(docx) → string` | List tracked revisions as a JSON array string (`type` / `author` / `date` / `part` / `moveGroupId` / `isMoveSource` / `formatChange` / `text`). |
| `docxToPdf` | `(docx) → Uint8Array` | Render a DOCX → PDF (Word-style layout). Fonts come from the embedded Carlito/Liberation set. *Full builds only.* |
| `pdfPageCount` | `(pdf) → number` | Page count of a PDF (`0` if the bytes are not a readable PDF). *Full builds only.* |
| `initPanicHook` | `() → void` | Route wasm panics to `console.error`. Safe to call multiple times. |

Errors (invalid/corrupt DOCX, unsupported constructs) are thrown as JS
exceptions with a `jubarte-wasm: …` message.

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
