<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# PDF and PNG

`docx_to_pdf(docx)` renders with `PdfOptions::default()`;
`docx_to_pdf_with(docx, options)` takes the options:

| `PdfOptions` field | Default | Effect |
|---|---|---|
| `compress` | off | deflate content streams; a text-heavy document shrinks to about a seventh. Off, each page is plain text a diff can read |
| `revisions` | `Conventional` | how tracked changes are painted ([Revision styles](revision-styles.md)) |
| `comments` | `Margin` | balloons beside the page, or a listing after the last page ([Comments](comments.md)) |
| `comment_notes` | off | also attach each comment as a PDF note at its anchor; an explicit opt-in |
| `changed_only` | off | lay out the whole document, then keep only the pages a tracked change touches, the pages a comment is anchored on, and the first page of each section where a revised header or footer paints; page numbers stay the document's |

`docx_to_png(docx, options, dpi)` returns one PNG per page: 96 DPI is
screen resolution, 150 reads comfortably, 300 is print, and `MAX_PNG_DPI`
(1200) is the ceiling. `render` does one layout pass and writes any of the
PDF, the PNGs and a report, optionally for chosen pages only
(`RenderRequest`). `pdf_page_count` counts the pages of a PDF.

```rust,ignore
{{#include ../../../../crates/jubarte-render/examples/to_pdf.rs:example}}
```

Run it with `cargo run -p jubarte-render --example to_pdf OUT_DIR`:

```text
conventional: 1 page(s), 6791 bytes
word: 1 page(s), 6797 bytes
custom: 1 page(s), 6791 bytes
png: 1 page(s)
```

From the command line:

```sh
jubarte convert redline.docx -o redline.pdf --revisions word --compress
jubarte convert redline.docx -o redline.png --dpi 120   # redline-page-01.png, ...
```

Errors are a `ConvertError`: a package that cannot be opened, one without
a main document, a PDF that cannot be assembled, or a PNG resolution out of
range.
