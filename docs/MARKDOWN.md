<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Markdown and CriticMarkup

jubarte reads Markdown alongside Word documents:

| Task | CLI | Library (`jubarte::markdown`) |
| --- | --- | --- |
| Markdown to Word, CriticMarkup as tracked changes | `jubarte convert draft.md` | `markdown_to_docx` |
| Word to Markdown, tracked changes and comments as CriticMarkup | `jubarte convert FILE.docx -t md` | `docx_to_markdown` |
| Accept or reject CriticMarkup in Markdown | `jubarte convert draft.md -t md --track-changes accept` | `resolve_critic` |
| The changes between any two documents as a patch | `jubarte diff old new` | `patch_documents` |
| Two Markdown documents as CriticMarkup (pandiff) | `jubarte diff old.md new.md --format critic` | `diff_markdown` |
| Any two documents as a Word redline | `jubarte diff a b -o redline.docx`, `jubarte a b` | `redline` |
| A Markdown edit applied to a Word document | `jubarte diff contract.docx edited.md -o redline.docx` | `apply_markdown` |

`jubarte convert FILE.docx -t md` prints the document as Markdown, with
each tracked change and comment as CriticMarkup followed by its author and
date (`-o FILE.md` writes it; `--track-changes accept` or `reject` writes
the text after Word's Accept All or Reject All). Pictures become their alt
text. `jubarte read FILE` (alias `text`) prints the [agent view](#agent-view):
the same Markdown with a YAML header, an id line before every paragraph and
an id on every change and comment, for edit plans.

## Markdown to Word

`jubarte convert draft.md` writes `draft.docx` next to the Markdown
(`-o` names another file, `-o draft.pdf` or `-t png` renders it). The
Markdown is CommonMark with GitHub's tables, strikethrough, task lists and
footnotes, and a YAML front matter's `title:` and `author:` become the
document's properties.

| Markdown | Word |
| --- | --- |
| `#` to `######` | `Heading1` to `Heading6` |
| paragraph | `Normal` |
| `**bold**`, `*italic*`, `~~struck~~` | bold, italic, strikethrough |
| `` `code` `` and fenced code | `VerbatimChar` and `SourceCode` (pandoc's names) |
| `- item`, `1. item` | numbered paragraphs (`ListParagraph`); an ordered list starts at its first number |
| `> quote` | `Quote` |
| table | table in `TableGrid`, header row repeated, column alignment kept |
| `[text](url)`, `[text](#anchor)` | hyperlink |
| `[^note]` | footnote |
| `![alt](file.png)` | picture (PNG, JPEG, GIF, BMP, TIFF), at most the text width; else its alt text |
| `---` | a paragraph with a bottom border |
| `<br>`, `<sup>`, `<sub>`, `<u>` | line break, superscript, subscript, underline |

Images are read relative to the Markdown file, or to `--resource-path`;
URLs are not fetched.

`--reference-doc house.docx` works as pandoc's: the output takes the
reference's styles, numbering, page setup, headers and footers, and none of
its text. Styles the reference lacks are added with jubarte's definitions.
A pandoc reference document works, since the style ids are Word's built-in
ones plus pandoc's for code.

Without a reference, `--page letter` (the default) or `--page a4` picks the
page size; both keep one-inch margins, where Word's own A4 template uses
2 cm. With a reference, its page setup wins and `--page a4` prints a
warning.

The same writer is in Python and JavaScript:

```python
import jubarte_redlines

doc = jubarte_redlines.from_markdown("Due in {~~30~>45~~} days.", page="a4")
doc.to_bytes()  # the .docx
```

```js
const docx = markdownToDocx("Due in {~~30~>45~~} days.", JSON.stringify({ page: "a4" }));
```

`python -m jubarte_redlines convert draft.md` writes `draft.docx` with the
same flags. Neither binding reads image files: pictures are written as
their alt text.

## CriticMarkup

[CriticMarkup](https://criticmarkup.com) marks changes in plain text. jubarte
reads it before the Markdown, as the specification asks, so a change can
cross emphasis, links and paragraph breaks.

| CriticMarkup | Word |
| --- | --- |
| `{++new++}` | inserted text |
| `{--old--}` | deleted text |
| `{~~old~>new~~}` | deleted, then inserted text |
| `{==text==}{>>note<<}` | a comment on `text` |
| `{++new++}{>>note<<}` | a comment on the change |
| `{>>note<<}` alone | a comment at that point |
| `{==text==}` alone | highlighted text |

Every change and comment gets `--author` and `--date` (fixed by default,
so the same Markdown writes the same bytes).

Paragraphs follow Word's model, where a paragraph's mark (the end of the
paragraph) can be inserted or deleted too:

- A change that crosses a paragraph break holds that break: `A{++\n\nB++}`
  adds paragraph `B` after `A`; rejecting it leaves `A` alone.
- A block whose whole text is one change is added or removed whole, mark
  included: `{++New paragraph.++}` on its own line, `# {++New heading++}`,
  `- {--Old item--}`.
- Word cannot change the last paragraph's mark, so a last paragraph added
  or removed whole passes its change to the mark before it, as Word does.
- A table row whose every cell is one change whole is an inserted or
  deleted row.
- A footnote whose reference is inserted or deleted is inserted or deleted
  with it.

`--track-changes accept` or `reject` writes the document with every change
accepted or rejected, through the same engine as `jubarte accept` and
`jubarte reject`. With `-t md` the Markdown itself is resolved instead. The
flag works on a `.docx` too: `jubarte convert redline.docx --track-changes
accept` renders the accepted document to PDF.
`--no-critic` reads the delimiters as text. A delimiter behind a backslash
(`\{++`) or without its partner is text too.

## Agent view

`jubarte read FILE.docx` (alias `text`) prints the agent view: a YAML header
with the facts an editor needs, then the document as Markdown and
CriticMarkup, with an id line before every paragraph and an id on every
tracked change and comment. The ids are the ones `edit`, `accept --id`,
`reject --id` and `comments` take.

```text
---
source: received.docx
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it
revisions: 5                       # 1 insertion, 4 substitutions (9 Word marks)
comments: 2 threads open           # 3 comments: c5 (+ reply c6), c11
authors:
  document_owner: Arthur Souza Rodrigues  # cp:lastModifiedBy; no dc:creator
  AC: Ann Counsel                  # 5 revisions, 2 comments, 2026-10-01T09:00:00Z
  AS: Arthur Souza Rodrigues       # 1 comment, 2026-10-09T16:13:00Z
body: p0-p20, 1 table, 2 pages     # pages from layout
page: Letter portrait, margins 1in, header/footer 0.5in
styles:
  Normal: Calibri 11pt, after 8pt, line 1.08, left  # default; unannotated paragraphs use it
  "#": Heading1, Calibri bold 16pt, before 12pt, after 4pt, keep-next
  "##": Heading2, Calibri bold 14pt, before 12pt, after 4pt, keep-next
  table: TableGrid, all borders 0.5pt
headers:
  first: {id: header2, text: DRAFT}  # page 1 only (different first page)
  default: {id: header1, text: SIGNATURE PAGE, right}
footers:
  first: none                      # page 1 shows no page number
  default: {id: footer2, text: "{PAGE}"}
  even: {id: footer1, text: "{PAGE}", inactive}  # defined, but even/odd headers are off
---

<!-- page 1 of 2 -->

<!-- p0 center -->
# Consulting Agreement

<!-- p1 justify -->
This Consulting Agreement (the “Agreement”) is made between Harbor Point Analytics LLC ("Consultant") and Juniper Freight Co. ("Client").

<!-- p2 -->
## 1. Services

<!-- p3 -->
Consultant will deliver the services in each Statement of Work, including {++quarterly++}{>>#0 @AC<<} route-cost reports and a {~~weekly~>monthly~~}{>>#1+2 @AC<<} review call.
```

`--track-changes accept|reject` prints the document with every change
resolved, `--comments none` leaves the comments out of the text (their ids
move to the id lines), `--dates` puts timestamps in the notes of authors
with several, `-p p3,p10-p20,t0`, `--head N` and `--tail N` print a
selection, and `--no-page-markers` skips the layout pass.

### Id lines

One per body paragraph, on the line above it: a head, one space, then
clauses joined by `, ` (no clauses: `<!-- p2 -->`).

```text
head:     p{N}[ {Style}]
clauses:  {align} · first-line {x}in · hanging {x}in · left {x}in · right {x}in · num "{label}" | bullet · page-break · section-break · break-ins {tags} · break-del {tags} · fmt {tags} · rev {tags} · comments #c{ids}
examples: <!-- p0 center -->   <!-- p18 first-line 0.5in, comments #c11 -->   <!-- p0 Quote justify, hanging 0.25in, left 0.5in -->   <!-- p3 rev #0 @AC; #1+2 @AC -->
```

- `N` counts every `w:p` under `w:body` in document order, table cells
  included, text boxes excluded: `pN` is `body:p:N`.
- `Style` prints when it is not the default paragraph style and, for a
  heading, not `Heading{level}`.
- `align` prints only when the paragraph sets `w:jc` itself: `center`,
  `right`, `justify`, else `left`. Indents come from the paragraph's own
  `w:ind`, in inches.
- `num "1."` is the auto-number label; `bullet` a bulleted item. Labels are
  not document text.
- `page-break`: the paragraph holds `w:br w:type="page"`. `section-break`:
  its `w:pPr` holds `w:sectPr`.
- `break-ins`/`break-del`: a tracked paragraph mark. `fmt`: a formatting
  change (`w:rPrChange`, `w:pPrChange`).
- `rev`: in the accept and reject views only, every revision the paragraph
  held before resolution, formatting changes and paragraph marks included.
  A paragraph joined into the one before it hands its revisions over.
- `comments`: with `--comments none` only, the comments whose ranges or
  references the paragraph holds.
- Empty paragraphs: `<!-- p19 empty -->`, consecutive ones
  `<!-- p19-p23 empty -->`. An empty paragraph that holds a tracked mark, a
  formatting change, a resolved revision or a hidden comment keeps its own
  line: `<!-- p4 empty, break-ins #3 @AC -->`.
- Page markers: `<!-- page N of M -->` before the first block that starts on
  page N, `<!-- page 1 of M -->` first. They come from the layout pass.
  With `--no-page-markers` none print; through the library without layout
  pages they come from Word's cached breaks (`w:lastRenderedPageBreak`),
  else from hard page breaks and from section breaks that start a page.

A table gets one line: `<!-- t0 center 3x2, cells p8-p13 by row, header row
repeats -->`, `cells r0 p8-p10 r1 p11-p14` when a cell holds other than one
paragraph, then `merged cells`, `break-ins #12 @AC in p9`, `break-del …`,
`rev … in pN` (resolved views) and, with `--comments none`,
`comments #c9 in p11`. `tN` counts top-level body tables.

### Tags

A tag is `#<ids> @<handle>`: `#0 @AC`, or `#1+2 @AC` for two Word marks of one
logical change. The ids are `w:id` values: `#12` is `body:rev:12`. Several
authors' tags on one id line are joined by `; ` (`rev #1 @AC; #2 @JD`).

```text
{++quarterly++}{>>#0 @AC<<}
{--ninety days--}{>>#7 @AC<<}
{~~weekly~>monthly~~}{>>#1+2 @AC<<}          one author: one note
{~~old~>new~~}{>>#1 @AC<<}{>>#2 @JD<<}        two authors: deleted side first
{++bold words++}{>>#7+8 @AC<<}                 neighbouring marks by one author
{++quarterly++}{>>#0 @AC 2026-10-03T14:05:00Z<<}   with --dates
```

Neighbouring marks of one kind by one author join, then a deletion next to
an insertion pairs as a substitution, left to right. Bookmarks, proofing
marks and comment markers do not break adjacency, and changes inside links,
content controls and simple fields count. The text inside the delimiters is
the document's text exactly.

A comment is `{==anchored text==}{>>#c5 @AC: text<<}`; a reply names its
thread root, `{>>#c6 @AS re #c5: Disagree.<<}`; a resolved root reads
`{>>#c5 @AC resolved: text<<}`.

A handle is the comment `w:initials` that author wrote, else the initials of
the name's words, letters and digits only; a collision appends a digit. The
header's `authors:` line gives each handle's name, counts and its
timestamp, or the day range when the author has several.

### Reading back

An agent view is CriticMarkup plus id lines. The Markdown-to-Word applier
does not read it back yet; this is the contract it will honour. A
change followed by a tagged note (`{>>#12 @AC<<}`) is the document's revision
`body:rev:12` (`footnotes:rev:12` under a footnote id line): kept as it is
when unchanged, accepted or rejected when the agent removed its text. A
change with no note is new, by the plan author. A tagged comment note is the
document's comment; `re #cN` names its thread root; `resolved` resolves the
thread; a note with no tag is a new comment by the plan author, a reply when
it follows a tagged note on the same anchor. `<!-- pN -->` lines are
alignment keys and are never text.

Defaults when the Markdown is incomplete, in order: `source:` names a base
document, and everything not stated comes from it by id; a note's own
`@handle` and timestamp; the handle's single timestamp from `authors:`; if
`authors:` lists exactly one non-owner author, that author; else the author
`Modified User`; a missing date is the conversion time in UTC, with a
warning; `document_owner` missing is
`Original User`; with no header at all, Letter portrait, one-inch margins,
Normal Calibri 11pt and the writer's other defaults. A header without
`source:` is honoured for `page:`, the `styles:` lines it names, and
`headers:`/`footers:` text, alignment and `{PAGE}`/`{NUMPAGES}`/`{DATE}`
fields.

### Literal marks and anchors

Document text that CommonMark would read as markup is escaped in the agent
view: a paragraph that starts with `# `, `- `, `> ` or `1. ` (also after a
line break), and `*`, `` ` ``, `\`, `~~`, `==`, `[^`, a tag-like `<` and a
word-edge `_` anywhere (`\# Not a heading`, `\*\*stars\*\*`). The escapes
read back as the characters. `convert -t md` (ids off) does not escape.

An `edit` anchor copied out of the view may carry marks that are not
document text (`# Fees`, `**secret**`, a CriticMarkup note). The literal
anchor is tried first; when it does not occur, the anchor without its marks
(escapes resolved) is tried, and the report's `op` line records
`anchor_given` and `anchor_read_as`. The CLI prints
`note: op-1: anchor "# Fees" read as "Fees" (Markdown marks are not document
text)`. `read --changed` keeps the blocks with a change or a comment;
`--by AUTHOR` (a handle or a full name) keeps one author's.

### Commands

```text
jubarte read FILE                      the view (alias: text); jubarte FILE is the same
  -p p5,p12-p20,t0   --head N   --tail N   --changed [--by AC]
  --track-changes accept|reject   --comments none   --dates   --no-page-markers
jubarte A B                            accept both sides' changes, compare, print the redline's view
                                       (read options apply; -o FILE writes the file instead)
jubarte compare A B                    as before: writes <A>_v_<B>.docx

jubarte edit FILE -p WHERE --anchor FIND --content WITH        replace FIND (an insertion when WITH keeps FIND at its start or end)
jubarte edit FILE -p WHERE --content TEXT                      rewrite the paragraph
jubarte edit FILE -p WHERE [--anchor FIND] --delete            delete FIND, or the paragraph
jubarte edit FILE -p WHERE [--anchor FIND] --style SPEC        format FIND, or set the paragraph style
jubarte edit FILE -p c5 --content TEXT | --delete | --resolve  a comment
jubarte add  FILE -p WHERE --content TEXT [--before]           a new paragraph after (or before) WHERE
jubarte add  FILE -p WHERE --anchor FIND --content TEXT        a comment on FIND (--comment: on the whole paragraph)
jubarte add  FILE -p c5 --content TEXT                         a reply
```

The printed view is the output of `jubarte FILE` and `jubarte A B`, so `-q`
does not hide it (it silences the `wrote …` line of `-o`). Options go after
the task: `jubarte --head 2 read a.docx` is refused, since the parser would
take `read` for a document.

`WHERE` is an id the view prints (`p12`, `header1`, `footer2.p1`,
`t0.r1.c2`, `c5`); `edit` plans take the same ids. One command carries
several operations: every `-p` starts one, and the flags after it belong to
it. `--plan PLAN.json` stays for batches and the other operation kinds, and
excludes the operation flags.

The options apply to every operation of the command: `--author NAME`
(default `Modified User`), `--datetime ISO8601` (default now, UTC),
`--suggesting-mode` (the default: tracked changes; `redline.docx`,
`clean.docx`, `patch.diff` and `report.jsonl` are written) or
`--editing-mode` (the edits land directly: `clean.docx` and `report.jsonl`
only), `--existing-revisions auto|keep|accept|reject|refuse` (default `auto`:
`keep` when the file has tracked changes; `--editing-mode` refuses
`keep`, so pass `accept` or `reject` for such a file), `--out-dir DIR`
(default `<stem>.edit` beside the file; refused when it exists unless
`--force`).
`--style` takes `bold`, `italic`, `underline`, `strike`, `caps`,
`highlight=yellow`, `font=Calibri`, `size=11` and `color=FF0000`; any other
value is a paragraph style (`Heading2`). After applying, the command prints
its notes, then the changed blocks: `read redline.docx --changed --by
<author>`, or every changed block when the author has none (as after
resolving another author's comment). Anchors match literally first, then
without their Markdown marks, with a note.

### Limits of the agent view

- Tracked paragraph marks inside comment bodies are not shown.
- Headers and footers are header lines (`headers:`, `footers:`), not body
  blocks: a change only they hold leaves `--changed` and the view `edit`
  prints with no block, and the `range:` line says to read those lines.
- Page markers need the layout pass; `--no-page-markers` skips it. When
  the layout fails, the markers come from Word's cached page breaks, else
  from hard breaks and section starts. That fallback does not mix the two
  (a page break added after Word last saved turns no page) and does not
  add the blank page an odd- or even-page section start can need.
- A table is one block for `-p`, `--head` and `--tail`.
- With comments inline, a comment range between two changes by one author
  splits their note (`{>>#7 @AC<<}` … `{>>#8 @AC<<}`): CriticMarkup cannot
  nest a highlight inside a change. The resolved views say `rev #7+8 @AC`;
  both ids name the same revisions.

## Diffs

`jubarte diff OLD NEW` prints the changes as a patch, as `git diff
--word-diff` does, for any two documents, Word or Markdown:

```text
$ jubarte diff old.md new.md -a "Arthur Rodrigues"
--- a/old.md
+++ b/new.md	Arthur Rodrigues	2026-09-30T14:05:00Z
@@ [line:3] @@
Payment is due in [-30-]{+45+} days.

@@ [line:6] @@
- {+Returns+}
```

- Only the changed paragraphs are shown, each whole. `@@ [line:N] @@` is
  the line a Markdown paragraph starts on; in a Word document it is the
  paragraph's id, `@@ [body:p:N] @@` (or `header1:p:N`, `footnotes:p:N`,
  ...), the id edit plans take (`jubarte read` prints it as
  `<!-- p12 -->`). It is the
  paragraph in the new version; `@@ -[body:p:N] @@` is a paragraph
  removed, in the old one. A change in a text box is shown at the
  paragraph that holds the box.
- A change is `[-old-]{+new+}`. Highlights and comments stay CriticMarkup:
  `{==text==}{>>Name (date): comment<<}`.
- The `+++` line names who made the changes and when, once. A change by
  anyone else (a tracked change already in a Word document) is followed
  by `{>>Name (date)<<}`, and every comment carries its author and date.
  `-a/--author` defaults to `git config user.name`, else Redline;
  `-d/--date` to now.
- Changes grow to whole words, numbers, formatting spans and links, so
  `[-**twelve**-]{+**eighteen**+}` rather than a change inside the bold.
  Positions and counts are in characters: nothing is tokenized.
- Lines wrap at 72 columns, never inside a change's delimiters, code, a
  link or an autolink; `--columns N` changes it and `--columns 0` does not
  wrap. Nothing is printed when nothing changed.

The patch is printed whatever is written: `-o changes.docx` writes a Word
redline, `-o changes.md` the CriticMarkup below and `-o changes.pdf` the
redline painted (see `jubarte convert --help` for `--revisions`). What
`diff` wrote is said on stderr. With a Word side and no `-o`, the redline
goes to `<old>_v_<new>.docx` next to OLD, as before.

`jubarte edit` writes the same patch of its redline to `patch.diff` and
prints it after the report's summary; `-q` prints nothing. In Python,
`jubarte_redlines.diff(old, new)` and `Document.diff(other)` return it
(`str(diff)`, `diff.hunks`; a `diff` block in Jupyter) and an edit's result
has `.diff`; in WASM, `diffDocuments` and `applyEditPlan(...).patch`.

### CriticMarkup (pandiff)

`--format critic` prints two Markdown documents whole, with the changes
as CriticMarkup, as pandiff does: accepting them all gives `new.md`,
rejecting them all gives `old.md`.

```text
$ jubarte diff old.md new.md --format critic
# Terms

Payment is due in {~~30~>45~~} days.

- Delivery
- {++Returns++}
- Warranty
```

Lines are aligned first, then words within the lines that pair up; changes
separated only by spaces merge, so a rewritten phrase is one change. A
change never starts a line, so list, heading, quote and footnote markers
stay in place and both versions keep their structure. A block added or
removed whole is one change over its text, which Word writes as a
paragraph added or removed. A table row added or removed is marked cell by
cell, since a table drops text outside its cells. An ordered list's numbers
are not compared: Word numbers items itself. CriticMarkup delimiters
already in either document are escaped, so they stay text. The patch is
built from this alignment.

For Word output the two Markdown documents are written to Word and
compared by the same engine as two `.docx` files, so moves and Word's
grouping of changes apply. `--reference-doc` styles both.

### Git

```sh
git config --global difftool.jubarte.cmd 'jubarte diff "$LOCAL" "$REMOTE"'
git difftool -t jubarte -y -- '*.md'
```

## Word against Markdown

`jubarte diff contract.docx edited.md -o redline.docx` (or `jubarte
contract.docx edited.md`) is the redline of an edit made in Markdown, on
the Word document. The Markdown is not written to Word and compared, which
would lose what Markdown cannot say (empty paragraphs, fields, section
breaks, direct formatting) and show every loss as a change. Instead the
Markdown's edits are applied to the Word document (`apply_markdown`), and
the comparer compares the document before and after:

- The Markdown's blocks are aligned with the body's paragraphs by text
  (whitespace, symbols and typed numbering such as `1.` or `(a)` aside), so
  Markdown from any converter works.
- A paragraph whose text changed is rewritten word by word (the `rewrite`
  edit operation): unchanged words keep their runs and formatting, and
  tabs, breaks and fields stay.
- A block the document lacks becomes a paragraph next to its neighbours,
  with the properties of the nearest paragraph of its kind (heading, list
  item, plain), so a new list item is numbered and a new paragraph is not.
- A paragraph the Markdown lacks is deleted; a table cell's text is emptied.
- An edit inside a link or a field, which a rewrite cannot reach, replaces
  the paragraph whole.
- Empty paragraphs, which Markdown cannot hold, stay.

Reversed (`jubarte diff edited.md contract.docx`), the Markdown is applied
the same way and the redline runs from it to the Word document.

Not applied, and reported as warnings: footnote text (only the body is
edited) and table rows or columns added. Emphasis changed in the Markdown
is not applied to an existing paragraph; a new paragraph takes the
Markdown's bold and italic. A document with tracked changes is read with
them accepted.

## Limits

- `jubarte diff --format critic` with a Word side writes a Word redline
  rather than printing CriticMarkup; the patch (the default) prints it.
- Markdown math, definition lists and raw HTML blocks are not written.
- Changes have one author and date; CriticMarkup has no syntax for more.

Related: [CSHARP_MARKDOWN_PROJECTION_MAPPING.md](CSHARP_MARKDOWN_PROJECTION_MAPPING.md)
maps Docxodus's C# Markdown projection onto this crate — a study with open
questions, not a shipped feature.
