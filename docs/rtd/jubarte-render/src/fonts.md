<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Fonts

Layout follows the fonts a document asks for. For each family and style it
uses, the renderer records which step found the face (a `FontStep`):

| Step | The face came from |
|---|---|
| `Embedded` | a font the document embeds |
| `Explicit` | the family itself, installed on the system or in jubarte's font folder |
| `AltName` | the `w:altName` the document's font table gives |
| `Theme` | a theme font slot |
| `WordSubstitution` | the face Word substitutes for the family |
| `OpenFallback` | a bundled metric-compatible face: Carlito for Calibri, Liberation Sans for Arial, Liberation Serif for Times New Roman, Liberation Mono for Courier New |
| `Generic` | the generic family the font table names (roman, swiss, modern, fixed) |
| `Unknown` | the last resort for a family nothing else knows |

`docx_to_pdf_report` returns the PDF with this font report
(one `FontReportEntry` per family and style); `font_report_json`
writes it as JSON. `jubarte debug FILE --check render` prints the fonts
asked for and what was loaded for each.

## Where fonts come from

| Variable | Effect |
|---|---|
| `JUBARTE_FONT_DIR` | jubarte's own font folder; without it, `~/Library/Application Support/jubarte/fonts` (macOS), `$XDG_DATA_HOME/jubarte/fonts` or `~/.local/share/jubarte/fonts` (Linux), `%APPDATA%\jubarte\fonts` (Windows) |
| `JUBARTE_FONT_INDEX` | the persistent font index, which remembers where each family was found; empty or `off` turns it off. Without it, `font-index.tsv` beside the font folder |

From a source checkout, `scripts/install.sh --fonts-only`
(`scripts/install.ps1` on Windows) installs the supplemental fonts into
that folder.

WebAssembly has no installed fonts: there, `docxToPdf` paints with the fonts
a document embeds and the bundled faces, so Chinese, Japanese or Korean text
without an embedded font is not painted.
