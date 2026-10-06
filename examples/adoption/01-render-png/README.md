<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# 01 — Render a .docx to a PNG page

Task: rasterize page 1 of `input.docx` (a two-page services agreement
built from `input.md`) at 72 dpi, and report the document's page count.

## Substituted tools: soffice + pdftoppm (+ pdfinfo for the page count)

Three processes, a private LibreOffice profile, and Poppler:

```sh
soffice -env:UserInstallation=file:///tmp/lo_adopt_01 --headless \
    --convert-to pdf --outdir . input.docx     # -> input.pdf
pdfinfo input.pdf | grep '^Pages:'             # -> page count
pdftoppm -png -r 72 -singlefile -f 1 -l 1 input.pdf render_page_1_soffice
```

## jubarte

One command; the page report carries the page count and each page's text:

```sh
jubarte convert input.docx --png --dpi 72 --report pages.json
# writes input-page-01.png, input-page-02.png and pages.json
```

## Tool versions (measured in this folder)

| Tool | Version |
|---|---|
| LibreOffice | 26.8.0.3 (soffice) |
| Poppler | 26.09.0 (pdftoppm, pdfinfo) |
| jubarte | 0.11.2 |

## Outputs

| File | Made by |
|---|---|
| `input.docx` | `jubarte convert input.md -o input.docx --force` |
| `render_page_1_soffice.png` | soffice + pdftoppm, page 1, 72 dpi |
| `render_page_1_jubarte.png` | jubarte page 1 (`input-page-01.png` renamed) |
| `page_count_soffice.txt` | `pdfinfo input.pdf` |
| `page_count_jubarte.txt` | `page_count` from `pages.json` |
| `pages.json` | jubarte page report (page_count, per-page text, fonts) |

Regenerate everything with `JUBARTE=/path/to/jubarte bash run.sh`
(the soffice steps print `skip: soffice not installed` and continue if
LibreOffice or Poppler is missing).

## What the two page-1 images show

Both are 612x792 px (US Letter at 72 dpi) with the text block between
columns 72 and 540 (the one-inch margins). Measured ink rows: soffice
86–698, jubarte 89–702; roughly 30 text lines each — the same heading,
list, table and paragraphs land on page 1 in both. Glyph shapes and
anti-aliasing differ (LibreOffice renders Calibri through its own font
handling; jubarte draws the same Calibri faces with its own rasterizer),
so the images are visually similar, not pixel-identical. Page counts
agree: 2 pages in `page_count_soffice.txt` and `page_count_jubarte.txt`.

## Verdict

For "render this docx and tell me the page count" the jubarte call
replaces all three commands and the temporary profile, adds a page
report pdftoppm has no equivalent of, and matches soffice on page size,
margins, page break position and page count here. It is worse in one
small way: the pages come out named `<stem>-page-NN.png`, so a script
that wants its own names still renames them (this folder does), and
jubarte printed two harmless `Fontconfig warning: no <cachedir>` lines
on stderr on this machine, which a caller must not mistake for failure.
