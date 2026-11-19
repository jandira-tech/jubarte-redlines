<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

The workspace publishes nine crates at one version; each pins the others it
uses exactly (`=`), so they always move together. Each layer reaches only
downward:

| Layer | Crate | Holds | Uses |
|---|---|---|---|
| 1 | [`jubarte-dom`](https://jubarte.readthedocs.io/projects/jubarte-dom/en/latest/) | the arena XML DOM (`xmllinq`), WordprocessingML names, ids, input limits | nothing in the workspace |
| 2 | [`jubarte-opc`](https://jubarte.readthedocs.io/projects/jubarte-opc/en/latest/) | the OPC package (`PartFs`), input admission, settings, styles, numbering, the shared Word tables, the render option types | dom |
| 3 | [`jubarte-redlines`](https://jubarte.readthedocs.io/projects/jubarte-redlines/en/latest/) | compare two documents; accept and reject revisions | dom, opc |
| 3 | [`jubarte-render`](https://jubarte.readthedocs.io/projects/jubarte-render/en/latest/) | DOCX to PDF and PNG | dom, opc |
| 4 | [`jubarte-doc`](https://jubarte.readthedocs.io/projects/jubarte-doc/en/latest/) | changes, comments, validate, audit, scrub, append, fields, inspect, debug | dom, opc, redlines; render (feature `render`) |
| 4 | [`jubarte-markdown`](https://jubarte.readthedocs.io/projects/jubarte-markdown/en/latest/) | Markdown in and out, CriticMarkup, Word 97-2003 `.doc` reading | dom, opc, redlines, doc; render (feature `render`) |
| 4 | [`jubarte-edit`](https://jubarte.readthedocs.io/projects/jubarte-edit/en/latest/) | edit plans written as tracked changes, text diffs, capabilities, publication | dom, opc, redlines, doc, markdown; render (feature `render`) |
| 5 | [`jubarte-cli`](https://jubarte.readthedocs.io/projects/jubarte-cli/en/latest/) | the clap command line and `self-update` | every crate above |
| 6 | [`jubarte`](https://jubarte.readthedocs.io/en/latest/) | the facade: every module under `jubarte::`, and the `jubarte` binary | every crate above |
