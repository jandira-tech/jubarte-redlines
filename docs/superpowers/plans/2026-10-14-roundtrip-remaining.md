<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# What the Markdown round trip still loses: plan (2026-10-14)

## Where these numbers come from

They come from the strict test `md_fixed_point markdown_and_docx_round_trip_exactly`,
run on the #476 branch (nested lists by indentation).
- **Totals:** 31 of 73 round trips are not exact. Ids: 6935 correct, 155 wrong,
  40 lost, 540 new. docx: 267 differences.
- **How the rows were built:** every `✗` line was grouped into a family. The rows
  below are ordered by how many files each one touches. A row names the earlier
  report's item (`O*`, `D*` in `jubarte-loop/roundtrip_report/`) where there is one.
- **Process:** each row is its own PR, test first (red, then green). Run
  `markdown_and_docx_round_trip_exactly` before and after, and record the new
  totals in the PR. Any view change regenerates the committed views
  (`tests/fixtures/md-roundtrip/views.txt`).

## 1. Notes parts that hold only separators (18 files lose endnotes.xml, 16 footnotes.xml)

Status: next.
- **Cause.** The writer makes a notes part only when the body holds a note
  (`footnotes`, `src/markdown/package.rs:894`). A Word document that once had
  notes, or came from a template that has them, keeps `footnotes.xml` and
  `endnotes.xml` with only the separator and continuation separator. The view
  says nothing about them, so they are lost.
- **Fix.** The `notes:` header line (`src/markdown/from_docx/header.rs:1066`)
  names the empty notes parts the document holds, `empty footnotes, endnotes`, in
  a clause beside its `fmt` clause. The writer then makes those parts with Word's
  separators (`ensure_notes_part`, `package.rs:856`). A part that holds a note
  needs no clause: the note implies it.
- **Not chosen:**
  - Writing both parts always would add parts to documents that never had them.
  - Calling the two equivalent in the comparator would hide a real difference.
- **Tests:**
  - a docx with separator-only `footnotes.xml` and `endnotes.xml` keeps both
    after the round trip;
  - a docx without them gains none;
  - a docx with one footnote and an empty `endnotes.xml` keeps `endnotes.xml`.
  - The separators' own changes (`notes: {fmt: …}`, 5 files `notes lost`) are
    row 6.

## 2. Paragraph styles of headers, footers, comments and notes (≈25 files)

Status: after 1. O5 and O11.
- **Cause.** The writer stamps each story's default style on all its paragraphs:
  `header`, `footer`, `annotation text`, `footnote text`, `endnote text`. The view
  never prints a story paragraph's style.
- **What breaks:**
  - Source paragraphs with no style (Normal) gain one: 7 `footer1` `new: pstyle
    "footer"`, and 10 comment parts gain `annotation text`.
  - Source paragraphs with another style lose it: `List Paragraph`,
    `Text Heading 2`, `line number`, `Endnote`, `Footnote Text`.
- **Fix:** the view names a story paragraph's style only when it is not the
  story's default, and the writer stamps the default only where the view is
  silent.
  - A header or footer entry gains `style: X` when all its paragraphs share
    style X. When they differ, it gains `styles: [X, Y]`, aligned with `text`.
    Normal is written `Normal`.
  - Comments: the `comments:` header line gains a document-wide `style X` clause
    when the comment paragraphs are not `CommentText` (a document's comments
    come from one tool). One comment that differs gets a `cN style X` exception
    in that line.
  - Notes: the same as comments, on the `notes:` line.
- **Tests:** one per story kind. A source paragraph with no style, and one with a
  non-default style, keep their styles. A story whose paragraphs all use the
  default prints no clause, so today's views stay byte for byte.

## 3. Page count (10 files)

Status: measure after 2, 7 and 9. A page count that differs after the round trip
is a symptom: styles (2), sections (7) and floating objects (9) all move text. A
file that still differs after those gets its own diagnosis with
`jubarte debug A.docx B.docx --check render`.

## 4. Change counts in the `revisions:` line (9 files; authors' counts 6 files)

Status: open. O1.
- **What happens.** The round trip groups marks into changes differently: for
  example 1549 changes become 1497, with 52 fewer insertions; and 52 insertions
  become 47 insertions plus 5 table rows.
- **Where.** Inserted table rows, and insertions next to them, come back as row
  changes. Author revision counts are off by one or two.
- **Fix.** Find, per file, the first mark whose grouping differs (the
  `revisions:` comment's per-kind counts name the family). Make the writer emit
  the grouping the view printed: a row insertion stays an insertion when the view
  tagged it so.
- **Test:** the three smallest files listed, with their `revisions:` lines
  matching.

## 5. Highlights lost or added (7 files lost, 6 new)

Status: open.
- **Examples:** `Cyan highlight` and `Green highlight` lost; `yellow-highlighted`
  ×10 and ×12 lost; `highlighted text` ×4 new.
- **Likely cause.** A highlight colour other than the one the view prints (`==`
  is yellow), or `w:shd` shading read as a highlight, or a highlight inside a
  comment range read as the range.
- **Fix:** print the colour when it is not yellow (`{==text==}{>>hl cyan<<}` or an
  HTML `<mark>` with a class; choose against the dialect in `docs/DIALECT.md`),
  and keep shading apart from highlight.
- **Test:** each colour Word offers, round-tripped.

## 6. Header lines lost (5 files `notes`, 4 `style_changes`, 5 `styles.*`, 1 each `unplaced`, `bibliography`, `tags`, `layout`)

Status: open. O6.
- **What happens.** The view prints these YAML lines, and the writer reads them
  back only partly.
- **Lines to cover:**
  - `notes: {fmt: footnotes.r1 @LW …}`: changes to the separators;
  - `style_changes`: `w:rPrChange`/`w:pPrChange` inside styles;
  - `styles.ListParagraph`: a style the body no longer uses;
  - `unplaced`;
  - Pandoc's `bibliography`, `tags` and `layout`.
- **Fix:** one sub-PR per line. The writer restores each, or the view stops
  printing what cannot come back, and says so in `docs/MARKDOWN.md`.

## 7. Section properties (≈10 files)

Status: open. O12.
- **What is lost:**
  - the order of header and footer references inside `w:sectPr`;
  - `w:titlePg`;
  - `w:type` (`nextPage`, `continuous`);
  - point-unit page sizes.
- **Fix:**
  - Write references in Word's order (header then footer, each
    default/even/first as the source).
  - Carry `type` and `titlePg` through the `sections:` line, which already has
    page sizes.
- **Test:** each break type and `titlePg`, round-tripped, with Word opening the
  result.

## 8. Table styles renamed `TableGrid` → `TableGrid0` (4 files, 24 tables)

Status: open. O7.
- **Cause.** The writer adds its own `TableGrid` definition when the source's is
  not in the view, then de-duplicates the name.
- **Fix:** reuse the style the view names when its definition comes from the
  view's `styles:` line or from Word's built-in list
  (`src/markdown/word_table_styles.tsv`). Never rename.
- **Test:** `ids-revisions.md`'s 18 tables keep `TableGrid`.

## 9. Floating objects (≈6 files)

Status: open. O4 and O8.
- **What is lost:**
  - anchored pictures (`wrapNone`/`wrapSquare`/`wrapTight`, behind text) in the
    body and headers;
  - frames in headers and footers (`framePr` with `xAlign=right`);
  - VML `pict`;
  - `mc:AlternateContent`;
  - inline pictures in headers.
- **Fix:** the view keeps each as an opaque, id-tagged object that the writer
  copies back unchanged (the docx is the source; Markdown carries a reference).
  Frames get a clause on their paragraph's id line.
- **Tests:** one per kind, with Word opening the result and a Word side-by-side
  of the page.

## 10. Paragraph and table text (120 paragraphs, 11 tables in 4 files)

Status: open.
- **Where.** Mostly `Strict01.docx` (81 wrong: Strict OOXML namespaces) and
  `ac6cd8d92f.docx` (49).
- **Fix:** diagnose each file. Read the first wrong id's `want`/`got`, then the
  XML.
  - Strict01 probably needs the Strict-namespace reader on the round-trip path.
  - ac6cd8d92f needs its own reading.
- **Test:** the file's first wrong paragraph.

## 11. Revision order (1 file)

Status: open. O2.
- **What happens.** An insertion-then-deletion pair comes back as
  deletion-then-insertion in the body and the endnotes.
- **Rule.** The view pairs only insertion then deletion
  (`{~~old~>new~~}`). A deletion then insertion is two changes (`d7b87793`).
- **Fix:** the writer writes a substitution's insertion first.
- **Test:** the file's pair.

## 12. Pandoc manuscripts (4 files, 483 new ids)

Status: decision D2, Arthur's.
- **What happens.** `dolphins.md`, `eLife.00047.md`, `git_manuscript.md` and
  `list_of_figures.md` are plain Pandoc Markdown without id lines or a view
  header. The round trip prints a full view, so every id is new.
- **Choices:**
  - compare them with a looser test (text and structure, not ids);
  - or treat them as `convert` inputs only and drop them from this test.
