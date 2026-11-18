<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Node and browser

`npm install jubarte-wasm` (Node 18 or later). The package is the engine
compiled to WebAssembly; `jubarte-wasm/slim` leaves the PDF renderer out,
and `jubarte-wasm/web` and `jubarte-wasm/web-slim` are the browser
builds. The `jubarte-redlines` npm package is the command line over it
(`npx jubarte-redlines`).

```js
const { compareDocuments, docxToPdf } = require("jubarte-wasm");

const redline = compareDocuments(originalBytes, modifiedBytes, "Reviewer");
const pdf = docxToPdf(redline, false, "word");
```

WebAssembly has no installed fonts: `docxToPdf` paints with the fonts a
document embeds and the bundled Carlito and Liberation faces.

Both blocks below are generated from the shipped declaration file and the
npm CLI's `--help`.

## API reference

{{#include ../../../javascript.md:wasm-api}}

## Command line

{{#include ../../../javascript.md:cli-npm}}
