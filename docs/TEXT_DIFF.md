<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Reviewing document text with a unified patch

Use `jubarte diff before.docx after.docx --format github` for a reviewable
Git/GitHub unified patch. `unified` and `text` are aliases. `--context N`
selects unchanged lines around each hunk (default 3, zero means changes only).
No output file is created unless `-o changes.patch` is supplied; `.diff`,
`.patch` and `.txt` are supported. An existing output requires `--force`.
Patch text goes to stdout; file-write messages and errors go to stderr.

The text comes from the lines underneath `jubarte debug A B -c text`, before
its clipping and hunk limits. The snapshot includes body paragraphs, tables
and rows, nested text boxes, headers, footers, footnotes, endnotes and comment
text. Existing insertions and deletions remain visible as `{+inserted+}` and
`[-deleted-]`; paragraph and row revision marks also remain. No revisions are
accepted or rejected to construct this view.

Headers and footers are labelled by their first section role, as the debug
comparison pairs them. Renaming `header1.xml` to `header9.xml` alone does not
create a textual change. Other stories are keyed by part name and retain
paragraph order. The view does not attempt to infer identity after arbitrary
section insertion, note/comment reordering or role changes.

A patch contains `diff --git`, `---` / `+++`, `@@` hunk ranges, unchanged
context, removals prefixed with `-`, and additions prefixed with `+`. Equal
snapshots give an empty patch. Unicode and long lines are preserved, and
filenames with control characters are Git quoted. Markdown inputs retain
their final-newline state; a missing final newline is reported normally.

These patches describe extracted text, not a binary DOCX ZIP. To produce a
Word tracked-changes document use `jubarte compare A B -o redline.docx`.
Formatting, images, field instructions, comment authors and range anchors
are outside the debug text view. Use `debug -c runs`, `debug -c xml`,
`debug diff` or `diff-render` for those comparisons.

CriticMarkup remains a representation of the current document with marks.
The existing `patch` and `critic` formats retain their existing APIs and
output rules. The new `github` format compares two textual representations
without rewriting either one's marks.

## Python

```python
from jubarte_redlines import read, diff

before = read("before.docx")
after = read("after.docx")
print(before.diff(after, format="github", context=3).text)
print(diff("Old\n", "New\n", format="github").text)
```

The result uses the existing `Diff` type. Its `text` is the unified patch;
`hunks` is empty because paragraph locators and unified line ranges are
different coordinate systems. Context must be an integer in 0..4294967295;
booleans, negative or fractional values and larger integers are rejected.

## JavaScript and browser WASM

```js
const patch = wasm.diffDocumentsUnified(beforeBytes, afterBytes,
  "before.docx", "after.docx", 3);
```

Each input is a DOCX `Uint8Array` or UTF-8 Markdown bytes. This API returns
the unified string; existing `diffDocuments` continues to return the
paragraph-patch JSON. Full and slim builds expose the same new text API.

The Rust `text_diff` module also provides `document_text`, `diff_text` and
`diff_documents`. All adapters use its formatter. Their argument parsing
uses the same clap declarations through `cli::parse_json`; that parser does
no document or host I/O.
