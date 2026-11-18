<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

The render product of the [jubarte]({{repository}}) DOCX engine: it lays
a Word document out page by page and writes a PDF, or one PNG per page. The
layout engine is its own and aims at Microsoft Word's output; it does not
call Word or LibreOffice. Tracked changes and comments are drawn in one of
three styles: jubarte's conventional colours (the default), Word's, or a
custom palette.

## Where it sits

`jubarte-dom` → `jubarte-opc` → `jubarte-redlines` / `jubarte-render` →
`jubarte-doc` / `jubarte-markdown` / `jubarte-edit` → `jubarte-cli` → `jubarte`

`jubarte-render` sits on `jubarte-dom` and `jubarte-opc`, beside
`jubarte-redlines`. The document, Markdown and edit layers use it behind
their `render` features.

## When to use this crate

Depend on `jubarte-render` when DOCX to PDF or PNG is what you need. It does
not compare documents and does not read Markdown: Markdown reaches PDF
through `jubarte-markdown`, which writes a DOCX first. For comparing and
rendering together, depend on the facade crate [`jubarte`](https://crates.io/crates/jubarte).

## Install

```sh
cargo add jubarte-render@{{version}}
```

or in `Cargo.toml`:

```toml
[dependencies]
jubarte-render = "{{version}}"
```

The jubarte crates are released together at one version and pin each other
exactly, so depend on the same version of every jubarte crate you use.
Minimum supported Rust version: 1.94.

## Cargo features

None is on by default.

| Feature | What it adds |
|---|---|
| `perf-profile` | Forwards to `jubarte-opc/perf-profile`: stage counters and timers. Diagnostic only. |
| `test-support` | Hidden helpers for jubarte's own tests; not a public API. |

## Example

```rust,no_run
use jubarte_render::convert::{PdfOptions, RevisionStyle, docx_to_pdf, docx_to_pdf_with, docx_to_png};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let docx = std::fs::read("redline.docx")?;

    // Default options: tracked changes in the conventional colours.
    std::fs::write("redline.pdf", docx_to_pdf(&docx)?)?;

    // Tracked changes drawn the way Microsoft Word prints them.
    let word = PdfOptions { revisions: RevisionStyle::Word, ..PdfOptions::default() };
    std::fs::write("redline-word.pdf", docx_to_pdf_with(&docx, word)?)?;

    // One PNG per page, at 96 dpi.
    for (n, png) in docx_to_png(&docx, PdfOptions::default(), 96.0)?.iter().enumerate() {
        std::fs::write(format!("page-{}.png", n + 1), png)?;
    }
    Ok(())
}
```

## Modules

- `convert`: DOCX to PDF and PNG (`docx_to_pdf`, `docx_to_pdf_with`,
  `docx_to_png`, `docx_to_pdf_report`), the options (`PdfOptions`,
  `RevisionStyle`, `CommentPlacement`), page diffs (`diff_render`) and the
  font report.
- Re-exported from the layers below: `opc` and `admission` (`jubarte-opc`);
  `xmllinq`, `namespaces` and `util` (`jubarte-dom`).

## Fonts

A run is drawn with the font the document embeds for it, else the installed
font of that name, else the substitutes Word would use, else a face bundled
in this crate that keeps the original's metrics (Carlito for Calibri,
Liberation Sans for Arial, Liberation Serif for Times New Roman, Liberation
Mono for Courier New). A substitute can change the layout, so install the
fonts your documents name for output closest to Word's. `JUBARTE_FONT_DIR` adds a font folder, and `JUBARTE_FONT_INDEX` moves
the persistent font index (`off` disables it).

The crate bundles 16 font programs under `assets/fonts/`: Carlito and the
Liberation Sans, Serif and Mono families, each in four styles. They are not
covered by the AGPL: both families are under the SIL Open Font License 1.1,
with their copyright notices in
[`LICENSE-Carlito`](assets/fonts/LICENSE-Carlito) and
[`LICENSE-Liberation`](assets/fonts/LICENSE-Liberation), which ship in the
crate beside the fonts.

## License

The code is [AGPL-3.0-only](LICENSE); the bundled fonts keep the licenses
named above. Review the AGPL's requirements before you embed the crate in a
product or a network service.
