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
text. `jubarte text FILE` prints the body with paragraph ids, for edit
plans.

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
  ...), the id `jubarte text` prints and edit plans take. It is the
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

Related: [CSHARP_MARKDOWN_PROJECTION_MAPPING.md](../CSHARP_MARKDOWN_PROJECTION_MAPPING.md)
maps Docxodus's C# Markdown projection onto this crate — a study with open
questions, not a shipped feature.
