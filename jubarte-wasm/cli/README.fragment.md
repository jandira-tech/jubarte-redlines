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
npx jubarte-redlines reject redline.docx -o original-again.docx --id r12
npx jubarte-redlines convert redline.docx --revisions word
npx jubarte-redlines read contract.docx
npx jubarte-redlines edit contract.docx -p p12 --anchor "thirty" --content "sixty" -a NAME
npx jubarte-redlines edit contract.docx --plan plan.json --out-dir review
npx jubarte-redlines --help
```

`redline` is an alias of `compare`, which prints the redline as the agent
view without `-o`. `read --json` (paragraph ids and package facts, changes
and comments with their short ids) and `capabilities` (what this build can
do; `--json` the manifest) are also available. The
command set, messages and exit codes match `uvx jubarte-redlines` (Python)
and the `jubarte` binary (Rust): 0 success, 1 error, 2 usage, 3 edit plan
refused. `-o` writes what its extension names (`.docx`, the `.md` view,
the `.json` snapshot or a `.pdf`), as the binary does. `convert` also
turns Word into Word (`--track-changes accept|reject`, `--update-fields`)
and into its paged Markdown (`--to md`), and `edit --pdf` draws the
result beside it. PNG pages, `--pages`, Markdown-to-Markdown `convert`,
`comments`, `validate`, `append`, `scrub` and `visual-diff` need the
Python or Rust build.

Inputs are `.docx`: save a Word 97-2003 `.doc` as `.docx` first.

The native `jubarte` binary offers `--keep-nested-comment` on `accept`,
`reject` and `convert --track-changes accept|reject`. These bindings do not
yet dispatch that option or return its kept-thread receipts; use the native
binary when retention is needed.

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
