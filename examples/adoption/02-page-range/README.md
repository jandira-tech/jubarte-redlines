<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# 02 — Render only a range of pages

Task: rasterize pages 2 and 3 only of `input.docx` (a three-page
maintenance statement built from `input.md`) at 72 dpi.

## Substituted tools: soffice + pdftoppm

LibreOffice has no page-range render, so the substitute first renders
every page to PDF and pdftoppm then picks the range out of it:

```sh
soffice -env:UserInstallation=file:///tmp/lo_adopt_02 --headless \
    --convert-to pdf --outdir . input.docx     # renders all 3 pages
pdftoppm -png -r 72 -f 2 -l 3 input.pdf range  # -> range-2.png range-3.png
```

## jubarte

One command; only pages 2 and 3 are rasterized and written (layout still
runs over the whole document):

```sh
jubarte convert input.docx --png --pages 2-3 --dpi 72 --report pages.json
# writes input-page-02.png, input-page-03.png and pages.json
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
| `range_page_2_soffice.png`, `range_page_3_soffice.png` | soffice + `pdftoppm -f 2 -l 3` |
| `range_page_2_jubarte.png`, `range_page_3_jubarte.png` | jubarte `--pages 2-3` (renamed from `input-page-0N.png`) |
| `page_count_soffice.txt`, `page_count_jubarte.txt` | `pdfinfo` / `page_count` from `pages.json` (both 3) |
| `out_of_range.txt` | the out-of-range check described below, run by `run.sh` |
| `pages.json` | jubarte page report |

Regenerate everything with `JUBARTE=/path/to/jubarte bash run.sh`.

## What the images show

Same page size (612x792), same margins (ink between columns 72 and 540),
and the same page break: page 2 carries 28 text lines on both sides
(ink rows 72–708 soffice, 72–707 jubarte) and page 3 carries 8 lines on
both sides (ink rows 72–230 soffice, 72–232 jubarte). The content of
each page is the same; glyph shapes and anti-aliasing differ, so the
pairs are visually similar, not pixel-identical.

## Out-of-range behaviour (measured here, recorded in `out_of_range.txt`)

`pdftoppm -f 2 -l 4` on this 3-page PDF exits 0 and silently writes
only the pages that exist (2 files). `jubarte convert --pages 2-4`
exits 1 with `page 4 is out of range: the document has 3 pages` and
writes no PNGs. A silent clamp is convenient; a refusal cannot hide a
wrong page spec. Neither behaviour is wrong — they differ, and a caller
migrating from pdftoppm must handle jubarte's exit 1.

## Verdict

jubarte replaces the soffice-then-pdftoppm pair for page-range rendering
with one command that skips rasterizing pages you did not ask for (the
soffice path must still render the whole document to PDF first), and it
agrees with soffice here on page count and on where pages 2 and 3
break. It is worse in two small ways: the output name is fixed to
`<stem>-page-NN.png`, so scripts that want their own names still rename
files (this folder does), and an out-of-range spec is a hard error
rather than pdftoppm's silent clamp.
