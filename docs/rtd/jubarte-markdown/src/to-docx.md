<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Markdown to Word

`markdown_to_docx(markdown, &DocxOptions)` writes CommonMark as a `.docx`
and returns its bytes with any warnings (an image written as its alt text,
for example). CriticMarkup is read before the Markdown, as its
specification asks, so a change can cross emphasis, links and paragraph
breaks:

| CriticMarkup | Becomes |
|---|---|
| `{++text++}` | a tracked insertion |
| `{--text--}` | a tracked deletion |
| `{~~old~>new~~}` | a tracked deletion followed by an insertion |
| `{==text==}{>>note<<}` | a comment on `text` |
| `{>>note<<}` | a comment at that point |

`DocxOptions` sets the author and date of the changes and comments (the
date is fixed by default, so the same Markdown writes the same bytes),
whether CriticMarkup is read at all (`critic`), whether the changes stay
tracked or are accepted or rejected (`track_changes`), and a reference
`.docx` whose styles, numbering, page setup, headers and footers the output
takes, as pandoc's `--reference-doc`.

```rust,ignore
{{#include ../../../../crates/jubarte-markdown/examples/markdown.rs:example}}
```

Run it with `cargo run -p jubarte-markdown --example markdown OUT.docx`.
The read-back shows each change with its id, author and date:

```text
Payment is due in {~~thirty~>forty-five~~}{r1+2 Reviewer (1970-01-01T00:00:00Z)} days.
```

From the command line, `jubarte convert draft.md -o draft.docx` (or
`.pdf`, `.png`). The full dialect, front matter included, is
`docs/MARKDOWN.md` in the repository.
