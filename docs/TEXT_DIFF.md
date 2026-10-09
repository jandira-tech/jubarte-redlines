<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Reviewing document differences

```sh
jubarte diff a.docx b.docx --format github
jubarte diff a.docx b.docx --format github -U0 --accept-changes
jubarte diff a.docx b.docx --format word
jubarte diff a.docx b.docx --format normal --accept-changes
jubarte diff a.docx b.docx --format context -U0
jubarte diff a.docx b.docx --format side-by-side
```

| Format | View | Input tracked changes |
| --- | --- | --- |
| `github` (`unified`, `text`) | Git headers, `@@` ranges, `-`/`+` lines | Preserved inside each line |
| `word` | Changed text with fresh `{--old--}{++new++}` marks | **All changes accepted in both documents first** |
| `normal` | Line addresses (`a`/`d`/`c`), `<`/`>` lines, no context | Preserved |
| `context` | Old/new range blocks, `!`/`+`/`-` prefixes | Preserved |
| `side-by-side` | Old/new columns, `\|`/`<`/`>` separator | Preserved |
| `critic` | Current document content represented with tracked marks | Existing representation API |
| `patch` | Existing paragraph patch with edit IDs | Existing paragraph comparison API |

`--accept-changes` accepts every input revision for line-based views too.
Accepting changes keeps insertions and removes deletions. This can make a
clause disappear from the comparison even when its historical marks differ.
It also loses provenance: word diff shows the new comparison, not who proposed
an earlier edit. `critic` remains a document representation; it does not become
a synonym for word diff.

`--context N` (or `-UN`) controls unchanged lines around GitHub and context
hunks, default 3. Zero shows only changes. An unchanged gap greater than twice
the context size splits hunks. Hunk addresses count complete snapshot lines,
before display clipping, or accepted snapshot lines with `--accept-changes`.

Long lines show a 70-character window starting 35 characters before the
first change, bounded at the start of the line. `…` indicates hidden text.
Positions count Unicode characters rather than UTF-8 bytes. This is a review
view: clipping can hide later edits on the same line. Use `--full-lines` to
show all content and produce an unabridged textual patch. Short lines stay
complete. Full extraction happens before display clipping; there is no hunk cap.

In the `word` view the window counts text, never the marks' delimiters. A
mark it reaches keeps its delimiters and loses only text, so every mark a line
opens, it closes: `{++ c d e …++}`, never `{++ c d…`. In `side-by-side` the
old column is padded to its widest cell, so every `|`, `<` and `>` and every
new cell start in the same column.

Text views go to stdout and create no Word file. `-o review.patch` writes only
the named text file, with a status message on stderr. `.patch`, `.diff`, `.txt`,
`.md` and `.markdown` are supported; existing files require `--force`. Equal
snapshots produce empty output. Explicit rendering or redline options that do
not apply to a text view are usage errors before any input is read.

## What is compared

DOCX text comes from the lines underneath `jubarte debug A B -c text`, including
body paragraphs, tables and rows, nested text boxes, headers, footers, footnotes,
endnotes and comments. Text views use CriticMarkup additions `{++text++}` and
deletions `{--text--}` inside those lines. GitHub `-` and `+` prefixes are a
separate outer layer. A tracked deletion remains a line until it is accepted.

Headers and footers are labelled by their first section role. Renaming a
header part alone does not create a textual change. Other stories retain part
names and paragraph order. Arbitrary section insertion, note/comment reordering
or role changes can affect those labels. The resolved main document and declared
story XML are validated; malformed input is an error, not an empty snapshot.

These are differences between extracted text snapshots, not binary DOCX patches.
Formatting, images, field instructions, comment authors and range anchors are
outside this view. Use `debug -c runs`, `debug -c xml`, `debug diff` or
`diff-render` for those comparisons. Use `compare A B -o redline.docx` to produce
a Word document with tracked changes.

## Python

```python
from jubarte_redlines import read, diff

print(read("a.docx").diff(read("b.docx"), format="github").text)
print(diff("Due in {~~30~>45~~} days.\n", "Due in 60 days.\n",
           format="word", full_lines=True).text)
```

The result uses the existing `Diff` type. `text` contains the selected view;
`hunks` is empty because paragraph locators and line ranges are different
coordinate systems. `context` must be an integer in 0..4294967295; booleans,
negative or fractional values and larger integers are rejected.

## JavaScript and browser WASM

```js
const review = wasm.diffDocumentsView(beforeBytes, afterBytes, JSON.stringify({
  format: "word", oldName: "a.docx", newName: "b.docx", fullLines: true,
  oldFormat: "docx", newFormat: "docx"
}));
```

Inputs are DOCX `Uint8Array` or UTF-8 Markdown bytes. Explicit input formats
prevent damaged DOCX bytes from being mistaken for Markdown. Full and slim
builds expose this API. `diffDocumentsUnified` remains the complete unified
patch API; `diffDocuments` retains paragraph-patch JSON.

Rust exposes `TextOptions`, `TextFormat`, `diff_text_view` and
`diff_documents_view` alongside the complete snapshot/unified APIs. All three
CLIs use the same clap derives through `cli::parse_json`; options, enum values,
defaults and usage errors are parsed without document or host I/O.
