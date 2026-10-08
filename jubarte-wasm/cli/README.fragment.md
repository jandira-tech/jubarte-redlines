<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

Compare two Word documents into a tracked-changes document that opens cleanly
in Microsoft Word; list, accept or reject the changes; apply edit plans;
render DOCX to PDF. Runs on the [`jubarte-wasm`](https://www.npmjs.com/package/jubarte-wasm)
WebAssembly build of the [jubarte](https://github.com/jandira-tech/jubarte-redlines)
engine: no Word, no LibreOffice, no network.

```sh
npx jubarte-redlines redline original.docx modified.docx -o redline.docx --author Legal
npx jubarte-redlines changes redline.docx
npx jubarte-redlines accept redline.docx -o clean.docx --kind formatting
npx jubarte-redlines reject redline.docx -o original-again.docx
npx jubarte-redlines convert redline.docx --revisions word
npx jubarte-redlines text contract.docx
npx jubarte-redlines edit contract.docx --plan plan.json --out-dir review
npx jubarte-redlines --help
```

`redline` is an alias of `compare`. `inspect` (paragraph ids and package
facts) and `capabilities` (what this build can do) are also available. The
command set, messages and exit codes match `uvx jubarte-redlines` (Python)
and the `jubarte` binary (Rust): 0 success, 1 error, 2 usage, 3 edit plan
refused; flags differ per surface, and the render-side/PDF-producing ones
(including PNG pages and `--date`) need the Python or Rust build.

Inputs are `.docx`: save a Word 97-2003 `.doc` as `.docx` first.

## Text comparisons

```sh
jubarte-redlines diff a.docx b.docx --format github
jubarte-redlines diff a.docx b.docx --format github -U0 --accept-changes
jubarte-redlines diff a.docx b.docx --format word
```

Line views (`github`, `normal`, `context`, `side-by-side`) retain each input’s
tracked marks. Word comparison accepts **all changes in both inputs first**,
then creates fresh CriticMarkup; earlier revision provenance disappears.
`critic` remains a representation of document content with its marks.
Long lines use a 70-character review window; `--full-lines` shows all text.
Text views support DOCX and Markdown and write stdout or an explicit `-o` text
file. CLI help and usage errors come from the shared clap-derived grammar.
