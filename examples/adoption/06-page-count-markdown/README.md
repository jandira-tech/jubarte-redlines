<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# 06: which page is this paragraph on? (Markdown with page markers)

An agent that reads a contract as Markdown cannot say "clause 6, on page
2": Markdown has no pages. The skills read with `pandoc -t markdown` and,
to count pages, convert to PDF with LibreOffice and run `pdfinfo`.

`jubarte convert input.docx -t md` does both in one call. It lays the
document out (the same pass that writes its PDF), then puts
`<!-- page N of M -->` before the first block that starts on each page.
The markers are HTML comments, so Markdown readers drop them, and the
Markdown converts back to the same `.docx` text. `--no-page-markers` leaves
them out, and skips the layout pass.

```bash
pandoc -t markdown input.docx -o read_pandoc.md          # no pages
soffice --headless --convert-to pdf input.docx && pdfinfo input.pdf   # Pages: 3

jubarte convert input.docx -t md -o read_jubarte.md      # pages in the text
```

Run everything with `JUBARTE=/path/to/jubarte bash run.sh`.

## Outputs

| File | Tool | What it shows |
|---|---|---|
| `read_pandoc.md` | pandoc 3.11 | The text, with no page information |
| `page_count_soffice.txt` | LibreOffice 26.8 + pdfinfo | `Pages: 3` |
| `read_jubarte.md` | jubarte | The text with `<!-- page 1 of 3 -->`, `<!-- page 2 of 3 -->` before "6. Clause 6" and `<!-- page 3 of 3 -->` before "11. Clause 11" |
| `page_markers_jubarte.txt` | jubarte | The three markers |
| `page_count_jubarte.txt` | jubarte PDF + pdfinfo | `Pages: 3`: the marker count is the PDF's page count |

## Verdict (2026-10-04)

- Both tools put page 2 at "6. Clause 6" and page 3 at "11. Clause 11"
  (checked with `pdftotext -f 2 -l 2` on each PDF); the markers sit there.
- A marker goes only before a block, never inside a table or a list. A
  page that starts in the middle of a long table or paragraph is named at
  the next block, so a page can have no marker of its own. On an 11-page
  test document with long tables (`tests/corpus/fresh_docx_fixtures_and_redlines/docx_lots_of_comments_addition_removal.docx`)
  10 of 11 pages get a marker; page 11 begins inside a table.
- The pages are jubarte's layout, which targets Word's. On a dense
  document they can differ from Word's by a page, as the render pages say.
- pandoc gives no page information at all, so a skill needs a second tool
  (LibreOffice, then pdfinfo) for the page count alone.

`tests/adoption.rs` (`markdown_page_markers_count_the_pdf_pages`) checks
that the markers count the PDF's pages, rise in order, and vanish when the
Markdown converts back.
