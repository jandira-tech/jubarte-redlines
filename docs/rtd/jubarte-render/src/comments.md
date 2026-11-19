<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Comments

`PdfOptions::comments` (`CommentPlacement`) places a document's comments:

- `CommentPlacement::Margin` (the default): balloons in a pane beside the
  page, joined to the commented text, as Word's Save as PDF prints them. A
  resolved comment is faded and a reply is numbered under its thread.
- `CommentPlacement::End`: the page keeps its own width; the commented
  text keeps its tint and a `[JR1]`-style marker, and the comments are
  listed after the last page (the CLI's `--move-comments`).

No style puts a note icon in the text: a viewer paints the icon over the
words and hides them. `PdfOptions::comment_notes` (the CLI's hidden
`--comment-notes`) attaches each comment as a PDF note at its anchor; it is
off for every style and stays an explicit opt-in.
