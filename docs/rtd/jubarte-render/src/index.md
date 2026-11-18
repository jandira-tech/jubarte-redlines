<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# jubarte-render

The render product of the jubarte engine: it lays a Word document out page
by page and writes a PDF, or PNG pages, with Word-faithful layout and no
Word or LibreOffice process. Everything is in `convert`:
`docx_to_pdf`, `docx_to_pdf_with`, `docx_to_pdf_report`, `docx_to_png`,
`render` (one layout pass for PDF, PNGs and a report together),
`docx_render_report`, `layout_facts` and `diff_render`.

The pure option types (`PdfOptions`, `RevisionStyle`, `RevisionPalette`,
`CommentPlacement`, `RenderRequest`) live in jubarte-opc's
`render_options` and are re-exported here under `convert`.

The fidelity benchmarks measure the `word` revision style against
Microsoft Word's own Save as PDF, page by page.

## Where it sits

{{#include ../../shared/layers.md}}

## When to depend on it directly

Depend on `jubarte-render` to render documents and nothing else. It sits
beside jubarte-redlines, not above it: to render a redline, compare with
jubarte-redlines and pass the bytes here, or use the
[`jubarte`](https://jubarte.readthedocs.io/en/latest/) facade. jubarte-doc,
jubarte-markdown and jubarte-edit reach it through their optional `render`
feature.
