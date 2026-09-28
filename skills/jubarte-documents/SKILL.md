---
name: jubarte-documents
description: "Use this skill whenever the user wants to read, edit, redline, comment on, compare, accept/reject, or render Word documents (.docx). Triggers: 'Word doc', '.docx', 'tracked changes', 'redline', 'compare these documents', 'accept all changes', 'render to PDF', 'what does this contract say', 'comment on clause', or a request to change specific clauses of a .docx as tracked changes. One engine (jubarte) does the reading, the editing, the clean copy, the PDF and the page images; no pandoc, LibreOffice or Poppler. Creating a brand-new .docx from scratch still uses the docx npm library (section 5). Do NOT use for PDFs, spreadsheets, Google Docs, or legacy .doc files."
license: Proprietary. LICENSE.txt has complete terms
---

# DOCX with jubarte: read, edit as tracked changes, verify

`jubarte` is one binary (or `python -m jubarte_redlines`, the same commands
and files; only compare differs, see below) that reads a `.docx` into addressable paragraphs, applies a
plan of exact edits as Word tracked changes with comments, produces the clean
copy, and renders pages to PDF and PNG from its own layout engine. You never
touch `word/document.xml`.

| Task | Command |
|---|---|
| Read | `jubarte text file.docx` (Markdown with `[body:p:N]` ids) or `jubarte inspect file.docx --json` |
| Edit (tracked changes + comments) | `jubarte edit file.docx --plan plan.json --out-dir review --pdf --png` |
| Look at pages | `jubarte convert file.docx --png --dpi 100` then `Read` the PNGs |
| Page count / page text | `jubarte convert file.docx --png --report pages.json` |
| Compare two versions | `jubarte a.docx b.docx -o redline.docx --author "Name"` (Python: `python -m jubarte_redlines compare a.docx b.docx -o redline.docx --author "Name"`) |
| Clean copy of a redline | `jubarte accept redline.docx -o clean.docx` (or `reject`) |
| What can this build do | `jubarte capabilities --json` |

Python: `import jubarte_redlines as jubarte; doc = jubarte.read("file.docx")`,
then `doc.markdown()`, `doc.inspect()`, `doc.edit(plan)`, `doc.to_png()`,
`doc.render()`, `doc.compare(other, author=...)`, `doc.accept()`.

## 1. Read before you edit

```bash
jubarte text contract.docx
```

Every paragraph prints as `[body:p:12] (a) **Confidentiality.** You will ...`.
The id is the coordinate an edit uses. `**bold**`, `*italic*` and
`==highlight==` are the document's direct run formatting, so you can see where
a bold heading run ends. `jubarte inspect contract.docx --json` gives the same
paragraphs as data (`text`, `style`, `numbered`, `in_table`, `runs` with char
offsets, `limitations`) plus `summary` (tables, comments, revisions, headers,
footnotes) and `source_sha256`.

Gotchas:
- Only the body story is addressable. Headers, footers, footnotes and text
  boxes are counted in `summary` but not printed and not editable.
- `limitations` on a paragraph (`field`, `hyperlink`, `content_control`,
  `sym`, `drawing`, `revision`) tell you which ranges an edit will refuse.
- Text is exact: tabs stay `\t`, smart quotes stay `“ ”`, a Symbol-font
  bullet is U+FFFC. Copy anchors from the output, do not retype them.
- If `summary.revisions > 0` the document already has tracked changes. An
  edit plan refuses it unless you set `"existing_revisions": "accept"` (or
  `"reject"`), which flattens first and reports `base_sha256`.

## 2. Edit with a plan

A plan is JSON: author, optional date, the source hash, and operations. Each
operation names one paragraph and one exact anchor; anything ambiguous fails
the whole plan and nothing is written.

```json
{
  "schema_version": 1,
  "source_sha256": "<from jubarte inspect --json>",
  "author": "Claude",
  "date": "2026-09-25T12:00:00Z",
  "operations": [
    {"id": "pronoun", "kind": "replace", "paragraph": {"contains": "signs in his or her"},
     "find": "his or her", "replacement": "an"},
    {"id": "recipients", "kind": "insert", "paragraph": {"starts_with": "(a) Confidentiality."},
     "after": "retained experts, ", "text": "court reporters, ",
     "comment": "Former 4(d) folded in here."},
    {"id": "survival", "kind": "comment", "paragraph": "body:p:88", "find": "Sections 1(g), ",
     "text": "Added 2(c) so the deletion duty survives termination."},
    {"kind": "delete_paragraph", "paragraph": {"starts_with": "(d) Onward Disclosure."}},
    {"kind": "insert_paragraph", "paragraph": {"starts_with": "(f) Notice of Inability"},
     "position": "after",
     "runs": [{"text": "(g) "}, {"text": "Automated Tools. ", "bold": true},
              {"text": "You will not upload the Information to any AI service."}],
     "comment": "New; delete if too aggressive."}
  ]
}
```

```bash
jubarte edit contract.docx --plan plan.json --out-dir review --pdf --png --dpi 100
```

Writes `review/clean.docx` (edits applied, no tracked changes),
`review/redline.docx` (Word tracked changes by `author`, comments attached),
`review/report.jsonl`, and with the flags `redline.pdf`, `clean.pdf`,
`redline-page-NN.png`, `clean-page-NN.png`. Exit 0 means every operation
matched exactly once. Exit 3 means the plan was refused: the report on stdout
says which operation and why (`ANCHOR_NOT_FOUND`, `AMBIGUOUS_ANCHOR` with the
match count, `OVERLAPPING_EDITS`, `UNSUPPORTED_STRUCTURE`, `STALE_SOURCE`,
`EXISTING_REVISIONS`, `INVALID_PLAN`); fix the plan and rerun. Use `--dry-run`
to see the report without writing.

Operation kinds: `replace`, `insert` (one of `after`, `before`,
`position: start|end`), `delete`, `comment` (`find` optional: whole
paragraph), `insert_paragraph` (`runs` with `bold`/`italic`/`underline`/
`highlight`; copies the anchor's paragraph properties), `delete_paragraph`.
Paragraph selectors: `"body:p:N"`, `{"index": N}`, `{"starts_with": "..."}`,
`{"contains": "..."}`; the last two must match exactly one paragraph.

Gotchas:
- `find` must occur exactly once in that paragraph; overlapping occurrences
  count (`"aa"` occurs twice in `"aaa"`). Widen the anchor instead of
  guessing.
- Inserted text takes the formatting of the run it lands in (`after` and
  `end` extend the preceding run; `before` and `start` join the following
  one). To insert bold or highlighted text, use `insert_paragraph` runs or
  put the text in plain and add a `comment` asking the reviewer to format.
- Run text is plain: no `\t` or `\n` inside `text`/`replacement`. A range
  that crosses a tab, a break, a field, a hyperlink or a content control is
  refused (`UNSUPPORTED_STRUCTURE`); edit the words on either side.
- Two inserts at the same position keep plan order. A replace and an insert
  inside its range conflict.
- `delete_paragraph` refuses a paragraph that carries a section break or is
  the only paragraph of a table cell.
- The redline is produced by comparing the source with the clean copy, the
  way Word Compare does. A long replacement therefore appears as a
  word-level diff against the old text, not as one deletion plus one
  insertion. The `ctx` field in the report shows exactly what you asked for.
- Comments on inserted text sit inside the insertion in the redline (Word
  shows them normally). The current PDF/PNG renderer paints balloons for
  comments on inserted paragraphs but not yet for comments inside inserted
  runs; the comments are in the file.
- Merging two paragraphs and tracked paragraph-formatting changes are not
  operations yet; say so rather than hand-rolling XML.

## 3. Verify

```bash
jubarte convert review/redline.docx --png --dpi 100 --report pages.json
```

`Read` the PNGs to look at the pages. `pages.json` has `page_count` and each
page's painted text, so you can say which page a clause starts on without
opening anything. The `render` line in `report.jsonl` already lists page
counts and page starts for both outputs when you passed `--pdf` or `--png`.

Gotchas:
- Page count is the renderer's layout, not Word's; treat a one-page
  difference between renderer and Word as possible on dense documents.
- `jubarte accept review/redline.docx -o check.docx` then `jubarte text
  check.docx` must equal `jubarte text review/clean.docx`. That is the
  every-edit-is-tracked check; it replaces `validate.py --author`.

## 4. Compare, accept, reject

```bash
jubarte original.docx revised.docx -o redline.docx --author "Legal"
# Python CLI: compare is a subcommand there
python -m jubarte_redlines compare original.docx revised.docx -o redline.docx --author "Legal"
jubarte revisions redline.docx --json
jubarte accept redline.docx -o clean.docx
jubarte reject redline.docx -o base.docx
```

Accepting a deleted paragraph mark joins that paragraph to the next one, as
Word does; a paragraph whose runs are all deleted disappears.

## 5. Create a new document (docx-js)

`docx` (npm) is preinstalled; write a script and `require('docx')`. Footguns:

- Page size defaults to A4. US Letter: `page: { size: { width: 12240, height: 15840 } }` (DXA; 1440 = 1″).
- Landscape: portrait dimensions plus `orientation: PageOrientation.LANDSCAPE`.
- Tables need `columnWidths` on the table and `width` on every cell, both `WidthType.DXA`; widths must sum to the table width. Shading uses `ShadingType.CLEAR`.
- Lists: a `numbering` config with `LevelFormat.BULLET`, never a literal `•`.
- `ImageRun` requires `type:`; `PageBreak` goes inside a `Paragraph`; never `\n`, use separate `Paragraph`s.
- TOC needs built-in `HeadingLevel.*` (or `outlineLevel` on custom styles).
- Horizontal rule: a paragraph bottom border, not a table. Dot leaders: `PositionalTab`.

Then verify with `jubarte convert output.docx --png --dpi 100` and `Read` the pages.

## Dependencies

`jubarte` (single binary) or `pip install jubarte-redlines` (`python -m
jubarte_redlines`, same commands; compare is `compare A B` there) · `docx` (npm) for new documents. Legacy
`.doc` is not read; ask for a `.docx`.
