<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# jubarte-render

The render product of the [jubarte
engine](https://github.com/jandira-tech/jubarte-redlines): DOCX (and
Markdown) to PDF and PNG with Word-faithful layout.

```rust
let pdf = jubarte_render::convert::docx_to_pdf_with(
    &docx, jubarte_render::convert::PdfOptions::default())?;
```

The 16 embedded font programs (Carlito and the Liberation families)
ship inside the crate; the pure option types are jubarte-opc's
`render_options`, re-exported at their historical `convert::` paths.

License: AGPL-3.0-only (see [LICENSE](LICENSE)); the bundled fonts
carry their own notices beside them under `assets/fonts/`.
