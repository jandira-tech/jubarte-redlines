---
name: jubarte-documents
description: "Use this skill whenever the user wants to read, edit, redline, comment on, compare, accept/reject, or render Word documents (.docx). Triggers: 'Word doc', '.docx', 'tracked changes', 'redline', 'compare these documents', 'accept all changes', 'render to PDF', 'what does this contract say', 'comment on clause', or a request to change specific clauses of a .docx as tracked changes. One engine (jubarte) does the reading, the editing, the clean copy, the PDF and the page images; no pandoc, LibreOffice or Poppler. Creating a brand-new .docx from scratch still uses the docx npm library (section 5). Do NOT use for PDFs, spreadsheets, Google Docs, or legacy .doc files."
license: AGPL-3.0-only
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
| Will Word open it, is every edit tracked | `jubarte validate redline.docx --original file.docx --author "Name"` (`--repair fixed.docx` fixes what it can) |
| What can this build do | `jubarte capabilities --json` |

Python: `import jubarte_redlines as jubarte; doc = jubarte.read("file.docx")`,
then `doc.markdown()`, `doc.inspect()`, `doc.edit(plan)`, `doc.to_png()`,
`doc.render()`, `doc.compare(other, author=...)`, `doc.accept()`,
`doc.validate()`, `doc.repair()`, `doc.audit_tracked(original, author=...)`.

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
footnotes) and `source_sha256`. After the body, `jubarte text` prints each
header, footer and notes part as its own story (`[header1:p:0] ...`,
`[footnotes:p:0] ...`), and `inspect` lists them under `stories`.
`jubarte comments FILE --json` lists every comment with its thread
(`parent`, `done`) and the anchored text with its surroundings
(`anchor_text`, `before`, `after`, `paragraph`); `--author NAME` keeps one
author's, `--latest` the newest of each thread.
`tables` reads each body table as a grid: `rows` of cells, each cell with
its `paragraph_ids` and `text`, plus `header_rows` and `widths_dxa`. Edit a
cell through its paragraph id. `jubarte inspect contract.docx --tables`
prints the same grids.
`inspect` also lists the body's content controls under `controls`: `id`
(`body:sdt:N`), `tag`, `alias`, `kind` (`text`, `rich_text`, `drop_down`,
`combo_box`, `date`, `checkbox`, `picture`, ...), `text`, `paragraph_ids`,
`locked`, `choices` (list values), `checked` and `placeholder`.

Gotchas:
- Headers, footers, footnotes and endnotes are editable stories (the kinds
  `jubarte capabilities --json` lists under `limits.stories`); text boxes
  are not stories: their text is omitted and not editable, and the owner
  paragraph carries `text_box_omitted`. Comments can
  only be anchored in the body (Word cannot anchor one in a header).
- `limitations` on a paragraph (`field`, `hyperlink`, `content_control`,
  `sym`, `drawing`, `revision`) tell you which ranges an edit will refuse.
- Text is exact: tabs stay `\t`, smart quotes stay `“ ”`, a Symbol-font
  bullet is U+FFFC. Copy anchors from the output, do not retype them.
- If `summary.revisions > 0` the document already has tracked changes.
  `"existing_revisions": "keep"` leaves their changes tracked and adds
  yours beside them (what Word does when you type on a received redline);
  `"accept"` or `"reject"` flatten first and report `base_sha256`; the
  default refuses. Under `keep` you cannot edit text inside their
  insertions or deletions, nor delete, merge or reformat a paragraph whose
  mark or properties they changed; resolve those changes first with
  `resolve_revisions`.
- To keep some of them, list them with `jubarte changes FILE --json` (one
  change per line: `id` such as `body:rev:12`, `kind`, `target`, `author`,
  `text`, `inside`) and resolve a selection, either directly
  (`jubarte accept FILE -o OUT --id body:rev:12 --author "Ann"`, flags
  repeat, a change must match every flag) or in the plan:
  `"resolve_revisions": {"accept": {"ids": ["body:rev:12"]}, "reject":
  {"authors": ["Bob"]}}`. `{}` selects every change and an empty list none;
  the changes it leaves still follow `existing_revisions`. Both sides of a
  move resolve together; a change whose `inside` holder is resolved away
  goes with it.

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
match count, `OVERLAPPING_EDITS`, `UNSUPPORTED_STRUCTURE`, `UNSUPPORTED_IMAGE`, `STALE_SOURCE`,
`EXISTING_REVISIONS`, `REVISION_CONFLICT`, `UNKNOWN_CHANGE`,
`UNKNOWN_COMMENT`, `COMMENT_NOT_IN_BODY`, `REDACTION_LEAK`, `UNSUPPORTED`,
`INVALID_PLAN`, `INVALID_EDIT`, `LOCKED_CONTROL`);
fix the plan and rerun. Use `--dry-run`
to see the report without writing.

Operation kinds: `replace`, `insert` (one of `after`, `before`,
`position: start|end`), `delete`, `comment` (`find` optional: whole
paragraph; `through` instead of `find` covers every paragraph from
`paragraph` to that one), `reply_comment` (`comment_id`, `text`),
`resolve_comment` (`comment_id`, `done` default true; false reopens),
`edit_comment` (`comment_id`, `text`), `delete_comment` (`comment_id`;
replies and anchors go with it), `insert_paragraph` (`runs` with `bold`/`italic`/`underline`/
`highlight`; copies the anchor's paragraph properties, or those of the
paragraph `like` selects), `delete_paragraph` (optional `comment`),
`format_paragraph` (any of `style` (id or name), `alignment`
`left|center|right|justify`, `line_spacing` as a multiple such as `1.15`,
`space_before`/`space_after` in points), `fill_control` (see below),
`merge_paragraphs` (joins the next
paragraph onto this one; optional `separator`, usually `" "`), `rewrite`
(`text`: the paragraph's whole new text; only the words that differ are
edited, so the rest keeps its runs and formatting), `insert_table`
(`rows`: cell text row by row, every row the same length; optional
`position` `before|after`, `header_row`, `widths_dxa` per column in
twentieths of a point (the text width split evenly when omitted), and
`style`, a table style id or name, `TableGrid` by default), `list`
(`paragraphs`: a list of selectors, not `paragraph`; `kind_of_list`
`bullet|decimal|lower_letter`, `level` 0 to 8, `restart` true by default),
`watermark` (`text`; optional `color` as six hex digits, `diagonal`,
`font`; no paragraph: writes Word's own diagonal text watermark into every
default header; one per document), `format_run` (`find`
plus `format`: restyles existing text as a tracked formatting change;
`occurrence`, 1-based, picks one of several matches), `insert_footnote`
(`after` plus the note's `text`; body paragraphs only; optional
`occurrence`), `insert_image` (`image_base64` of a PNG, JPEG, GIF, BMP or
TIFF file, `position` `before|after`, optional `content_type`, `width_emu`
with 914400 per inch, `alt`; body paragraphs only), `page_setup` (no
`paragraph`; `section` `last|all`, `page` `letter|a4|{"width_dxa",
"height_dxa"}`, `orientation` `portrait|landscape`, `margins_dxa` with any
of top, right, bottom, left, header, footer, 1440 per inch), `redact` (`find`: replaced with one block `█` per
character in the clean copy and the redline alike, untracked; optional
`occurrence`), `settings` (any of `track_revisions`, `update_fields` as
booleans and `protection` `{"edit": "readOnly|comments|trackedChanges|forms|none",
"enforcement": true}`; no paragraph; one per plan; this `update_fields`
asks Word to recompute fields on open, the plan's top-level one writes
jubarte's results now). `replace` and
`insert` take an optional `format` (`bold`/`italic`/`underline`/`highlight`,
`font`, `size_pt`, `color` as `FF0000` or `auto`, `strike`, `caps`) that
applies to the new text only. `replace` takes `"whole": true` to show
the change as the whole old text deleted, then the whole new text inserted.
Paragraph selectors: `"body:p:N"` (or `"header1:p:0"`, `"footnotes:p:2"`),
`{"index": N}`, `{"starts_with": "..."}`, `{"contains": "..."}`; the last two
must match exactly one paragraph. Those three search the body unless they
name a story: `{"story": "footer1", "contains": "Page"}`.

Gotchas:
- `find` must occur exactly once in that paragraph unless you give
  `occurrence` (1-based); the refusal says how many times it occurs.
  Overlapping occurrences count (`"aa"` occurs twice in `"aaa"`).
- Inserted text takes the formatting of the run it lands in (`after` and
  `end` extend the preceding run; `before` and `start` join the following
  one). To insert bold or highlighted text, give the operation a `format`
  (`{"bold": true}`, `{"highlight": "yellow"}`); only the new text changes.
- Run text is plain: no `\t` or `\n` inside `text`/`replacement`. A range
  that crosses a tab, a break, a field, a hyperlink or a content control is
  refused (`UNSUPPORTED_STRUCTURE`); edit the words on either side.
- Two inserts at the same position keep plan order. A replace and an insert
  inside its range conflict.
- `rewrite` needs no anchors: give the paragraph's new text. Tabs, breaks
  and symbols stay where they are (write a tab or a break as a space); new
  words take the formatting of the run before them. It is refused where a
  `replace` would be: a changed word inside a link or a field.
- `delete_paragraph` refuses a paragraph that carries a section break or is
  the only paragraph of a table cell.
- A `delete_paragraph` `comment` sits on the deleted text in the redline
  only; the clean copy has no paragraph to hold it. When an identical
  paragraph is next to it, the comparer may delete the twin instead, and the
  plan is refused (`UNSUPPORTED_STRUCTURE`) rather than leave the comment on
  text that stays.
- The redline is produced by comparing the source with the clean copy, the
  way Word Compare does. A long replacement therefore appears as a
  word-level diff against the old text. Give the `replace` `"whole": true`
  to show one deletion followed by one insertion instead, as typing over
  the selection with Track Changes on would; if the comparer's diff cannot
  be regrouped, the operation stays word-level and its report line carries
  a `message` saying why. The `ctx` field in the report shows exactly what
  you asked for.
- Comments on inserted text sit inside the insertion in the redline (Word
  shows them normally). The current PDF/PNG renderer paints balloons for
  comments on inserted paragraphs but not yet for comments inside inserted
  runs; the comments are in the file.
- `format_paragraph` is a tracked property change: the redline keeps the
  old style, alignment and spacing for reject. An unknown style is refused
  with `UNKNOWN_STYLE` and the list of defined style ids.
- `watermark` is header content, not a tracked change: the clean copy and
  the redline both carry it untracked. A first section without a default
  header gets one; a later section without its own inherits the previous
  header, as in Word. A document that already holds a watermark, or a
  second `watermark` in the plan, is refused (`UNSUPPORTED_STRUCTURE`).
- `redact` removes the text from both documents and is no tracked change,
  so the other side never sees what was there. The plan is refused with
  `REDACTION_LEAK` when the text still occurs anywhere in either output (a
  comment, another paragraph, a header, the document properties); the
  message names the parts, never the text. Redact every copy in the same
  plan. A short `find` can also match an unrelated attribute value and be
  refused: the check fails closed.
- `settings` writes `word/settings.xml` (created when missing) in schema
  order, in the clean copy and the redline alike: settings are not
  revisions. `false` removes `w:trackRevisions` or `w:updateFields`;
  `"edit": "none"` removes the restriction. `protection` has no password
  (`password` is refused with `UNSUPPORTED`): it is Word's "enforce
  without password", which any user can turn off. Two `settings` in one
  plan are `OVERLAPPING_EDITS`.
- `merge_paragraphs` keeps the second paragraph's properties (what Word's
  accept of a deleted paragraph mark does); the redline deletes the first
  paragraph's mark and inserts only the separator, as Word Compare shows a
  join. It refuses a paragraph that carries a section break or is not
  followed by a plain paragraph. Do not format, delete or insert after a
  paragraph you merge in the same plan (`OVERLAPPING_EDITS`).
- `insert_table` anchors on a body paragraph outside any table (nested
  tables are `UNSUPPORTED_STRUCTURE`). Cells take the anchor's paragraph
  style and run formatting. A table that would end the body or touch
  another table gets an empty paragraph beside it, as Word requires.
  Ragged rows, or `widths_dxa` whose count differs from the columns, are
  `INVALID_EDIT`; an unknown style is `UNKNOWN_STYLE`. `TableGrid` is
  added to the styles when the document lacks it.
- `list` numbers body paragraphs directly and gives `ListParagraph` to
  those without a style. `restart: false` continues the list of the
  nearest numbered paragraph before the first one, in that list's format,
  and is refused (`UNSUPPORTED_STRUCTURE`) when there is none. Do not
  delete, format or merge a paragraph you list in the same plan
  (`OVERLAPPING_EDITS`). The redline marks each paragraph's properties as
  changed, and also the `ListParagraph` definition when the plan added it.
- `insert_toc` (`position` before or after, `levels` 1 to 9, default 3,
  optional `title` styled `TOCHeading`) inserts a `TOC \o "1-N" \h \z \u`
  field in the body. Pair it with `"update_fields": true` at the top of the
  plan: the clean copy's TOC is then filled from the `Heading1`..`HeadingN`
  paragraphs, and every `PAGEREF`, `REF`, `NUMPAGES` and `SEQ` result is
  written, before the redline is compared; the report lists them under
  `fields`. Without it the TOC stays empty until Word updates its fields.
  On a document you are not editing, `jubarte fields update in.docx -o
  out.docx --json` does the same. Page numbers are jubarte's layout, which
  matches Word on most documents but is not Word
  (`docs/WORD_DIFFERENCES.md` section 11 in the jubarte repository).
  Field codes stay, so Word's Update Field still works.
  `update_fields` is refused (`INVALID_PLAN`) with `existing_revisions:
  "keep"`; `insert_toc` alone works there and is tracked as an insertion.

Content controls (form fields) are filled with `fill_control`, which names a
control instead of a paragraph and takes exactly one value:

```json
{"kind": "fill_control", "control": {"tag": "Name"}, "text": "Ada Lovelace"}
{"kind": "fill_control", "control": {"alias": "Country"}, "choice": "Brazil"}
{"kind": "fill_control", "control": "body:sdt:3", "checked": true}
{"kind": "fill_control", "control": {"tag": "Signed"}, "date": "2026-10-02"}
```

- `control` is an id from `inspect`'s `controls` or `{"tag": ...}` /
  `{"alias": ...}`; it must match one control (`ANCHOR_NOT_FOUND`,
  `AMBIGUOUS_ANCHOR` with the ids).
- `text` fills text, rich-text and combo-box controls; `choice` (a list
  item's value or display text, the display text is written) fills
  drop-downs and combo boxes; `checked` fills checkboxes with Word's glyph;
  `date` (`YYYY-MM-DD`) fills date pickers in the control's own date format,
  with English month and day names. Anything else is `INVALID_EDIT`, and a
  choice outside the list names the allowed values.
- The control keeps its tag, alias and lock; the placeholder flag goes, as
  when a person types into it. A block-level control becomes one paragraph
  with its first paragraph's properties.
- `LOCKED_CONTROL`: the control's content is locked. `UNSUPPORTED_STRUCTURE`:
  picture, group, repeating-section and building-block controls, controls
  around table rows or cells, and controls holding a footnote, endnote or
  comment reference. Bookmarks and comment ranges inside a control survive
  the fill around the new text.
- The clean copy is the filled form. In the redline the fill shows as tracked
  text without the control around it: the comparer unwraps controls in
  changed paragraphs, as Word Compare does (KNOWN_ISSUES.md #7).
  Under `"existing_revisions": "keep"` a fill is refused
  (`UNSUPPORTED_STRUCTURE`) until the tracked emitter writes fills; accept
  or reject the existing revisions first.
- Do not edit or delete a paragraph whose control you fill in the same plan
  (`OVERLAPPING_EDITS`); text outside a run-level control in the same
  paragraph can still be edited.

## 3. Verify

```bash
jubarte convert review/redline.docx --png --dpi 100 --report pages.json
```

`Read` the PNGs to look at the pages. `pages.json` has `page_count` and each
page's painted text, so you can say which page a clause starts on without
opening anything. The `render` line in `report.jsonl` already lists page
counts and page starts for both outputs when you passed `--pdf` or `--png`.

```bash
jubarte diff-render before.docx after.docx --out-dir diff
jubarte convert file.docx --png --pages 3-5
```

`jubarte diff-render before.docx after.docx --out-dir diff` writes only the
pages that changed with the change boxed, and `diff.json` with each page's
`changed_ratio`; it exits 5 when any page differs and 0 when none does.
`jubarte convert file.docx --png --pages 3-5` renders three pages from one
layout pass. Python: `jubarte_redlines.diff_render(a, b, dpi=100)` and
`Document.to_png(pages=[3, 4, 5])` (pages counted from 1).

Gotchas:
- Page count is the renderer's layout, not Word's; treat a one-page
  difference between renderer and Word as possible on dense documents.
- `--report` lists every font and whether it was substituted;
  `--fail-on-substitution` turns that into exit 4 for CI.
- `jubarte accept review/redline.docx -o check.docx` then `jubarte text
  check.docx` must equal `jubarte text review/clean.docx`. That is the
  every-edit-is-tracked check; it replaces `validate.py --author`. Under
  `keep`, accept only your own changes:
  `jubarte accept review/redline.docx --author Claude -o check.docx`
  (the author your plan names).
  `jubarte validate review/redline.docx --original contract.docx --author
  Claude` runs that check and the Word-validity check in one; it replaces
  `validate.py --original --author`. `--repair out.docx` fixes what it can
  and lists what it cannot.
- `jubarte audit file.docx --json` lists heading skips, images without alt
  text, tables without a header row, literal bullets, spacer paragraphs,
  stale TOC and page-count caches, and substituted fonts; `--strict` makes
  warnings fail. Each finding's `location` is the paragraph id to edit.

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

Either side may be Markdown (the `jubarte` binary): `jubarte contract.docx
edited.md -o redline.docx` applies the Markdown's edits to the Word document
and redlines only those, keeping empty paragraphs, fields and formatting.
`jubarte diff old new` prints the changes as a patch in the style of git
diff: only the changed paragraphs, each at its `[line:N]` (Markdown) or
`[body:p:N]` (Word) locator, with `[-old-]{+new+}` and CriticMarkup comments;
`--columns 0` stops the wrapping and `--format critic` prints two Markdown
documents as CriticMarkup (`{~~old~>new~~}`). `-o changes.docx` writes the
changes as tracked changes. `jubarte edit` writes the edit's patch as
`patch.diff` and prints it (`-q` prints nothing). See docs/MARKDOWN.md.

`jubarte append a.docx b.docx -o ab.docx` puts B after A on a new page;
images, links, styles, lists and notes come along. Comments are dropped
(warned as `COMMENTS_DROPPED`) unless `--carry-comments`, which brings the
comments B's body anchors with their threads and resolution (those in
notes, headers and footers are still dropped). More files fold left
(`append a b c`); `--section-break continuous` joins on the same page and
`--keep-sections` keeps B's page setup, headers and footers. A style A
already has (same type and name) keeps A's look. Python:
`Document.read("a.docx").append(Document.read("b.docx"))` returns
`Appended(document, warnings)`; WASM: `appendDocuments(a, b,
'{"section_break":"continuous","comments":"carry"}')`.

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

For prose, Markdown is shorter: `jubarte convert draft.md --reference-doc
house.docx -o draft.docx` takes the house styles, and CriticMarkup in the
Markdown (`{++added++}`, `{--removed--}`, `{==text==}{>>comment<<}`) becomes
tracked changes and comments.

## 6. Before sending a document out

`jubarte scrub in.docx -o out.docx` removes who touched a document: every
author (tracked changes, comments, `people.xml`) becomes "Author", rsids go, the document properties lose the creator, last editor,
revision number, dates, manager, company and custom properties, and
comments go. Text and tracked changes stay. `--author-alias NAME`,
`--rsids`, `--docprops` and `--comments` select only those. Python:
`Document.scrub(author_alias="Counsel", comments=False)`; WASM:
`scrubDocument(docx, '{"author_alias":"Counsel","rsids":true}')`.
Scrub refuses to write a package with a validity finding the input did not
have. Text to hide inside the document is a plan's `redact` (§2).

## Dependencies

`jubarte` (single binary) or `pip install jubarte-redlines` (`python -m
jubarte_redlines`, same commands; compare is `compare A B` there) · `docx` (npm) for new documents. As MCP tools (Claude Code, Codex, Gemini CLI):
`uvx --from 'jubarte-redlines[mcp]' jubarte-mcp --root .` (see `docs/adoption/mcp.md`). Legacy
`.doc` is refused with `LEGACY_DOC` (so is an encrypted document, which is
the same OLE container); convert it with Word, or ask for a `.docx`. RTF is
refused with `UNSUPPORTED_PACKAGE`.
