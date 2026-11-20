<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Word to Markdown

`docx_to_markdown(docx, &MarkdownOptions)` converts a `.docx` to Markdown.
By default tracked changes and comments come back as CriticMarkup;
`track_changes` set to `Accept` or `Reject` writes the text after Word's
Accept All or Reject All instead. `extract_media` names a folder for the
document's pictures, as pandoc's `--extract-media`. With `ids` on, the
output is the agent view: a YAML header, an id line before every block
(`<!-- p3 -->`), and Word's ids on revisions and comments.

`read(docx, &ReadOptions)` is that agent view with `jubarte read`'s
defaults. `read_tables` prints the grids of the tables a selection reaches
before that view, and `changed_view` only the blocks that carry one
author's marks.

`diff_markdown(old, new)` compares two Markdown texts and writes the
difference as CriticMarkup:

```text
Payment in {~~30~>45~~} days.
```

The example on [Markdown to Word](to-docx.md) runs both directions.
