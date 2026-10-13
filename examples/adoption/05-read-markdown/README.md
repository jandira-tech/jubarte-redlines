<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Reading a .docx: pandoc vs jubarte

Anthropic's docx skill reads a document with `pandoc -t markdown file.docx`;
the adoption page (`docs/adoption/anthropic-docx-skill.md`) offers
`jubarte read file.docx` as the replacement. This folder runs both on the
same input — a two-page services agreement with two heading levels, bold and
italic runs, a bulleted list, a 4x3 table, a header, a footer and one
footnote — and shows what each loses.

## The input

`input.docx` is written by `make_input.py`: the body comes from python-docx
1.2.0 (headings, runs, list, table, header, footer), and the footnote is
injected into the saved package with the standard library's `zipfile`,
because python-docx has no footnote API. `jubarte validate input.docx`
reports no findings on it.

## The commands

```bash
# The substituted tool (Anthropic skill, read step):
pandoc -t markdown input.docx -o read_pandoc.md

# The replacement:
jubarte read input.docx > read_jubarte.md
jubarte convert input.docx -t md > read_jubarte_convert.md
```

## Tool versions used here

- jubarte 0.11.2 (binary at `/Users/arthrod/temp/T/jr-adopt-bin/jubarte`);
  `read_jubarte.md` regenerated with jubarte 0.12.0 + the 0.13 CLI changes
  (`read` replaced `text`, 2026-10-12)
- pandoc 3.11
- python-docx 1.2.0, Python 3.14.7 (input only; not a reader below)

## Outputs

| File | Made by |
|---|---|
| `make_input.py` / `input.docx` | the input and the script that writes it |
| `read_pandoc.md` | `pandoc -t markdown` |
| `read_jubarte.md` | `jubarte read` |
| `read_jubarte_convert.md` | `jubarte convert input.docx -t md` |
| `versions.txt` | tool versions, written by `run.sh` |

## What each reader keeps

| Input feature | pandoc `-t markdown` | `jubarte read` | `jubarte convert -t md` |
|---|---|---|---|
| Heading 1/2 | `#`, `##` | `#`, `##` (the YAML header maps them to `Heading1`/`Heading2`) | `#`, `##` |
| `**bold**`, `*italic*` | kept | kept | kept |
| Bulleted list | `- ` items | `- ` items, each under its id line (`<!-- p5 ListBullet bullet -->`) | `- ` items |
| Table | grid table, rows and columns kept | GFM pipe table, kept, under `<!-- t0 4x3, cells p13-p24 by row -->` | GFM pipe table, kept |
| Footnote | `[^1]` marker at the anchor + definition | `[^1]` marker at the anchor + definition under `<!-- footnotes.p0 -->` | `[^1]` marker + definition |
| Header | lost (silently) | kept in the YAML header: `headers: default: {id: header1, text: ...}` | lost (silently) |
| Footer | lost (silently) | kept in the YAML header: `footers: default: {id: footer1, text: ...}` | lost (silently) |
| Paragraph ids | none | `<!-- pN -->` before every paragraph | none |
| Pages | none | `<!-- page N of M -->` from the layout | `<!-- page N of M -->` |
| Punctuation | escaped: `\"`, `\'`, `\_\_\_`; wrapped near 72 columns | as in the document, but `_` escaped (`\_\_\_`) | as in the document |

Verified in the outputs: `grep -c "Draft 3" read_pandoc.md` is 0 (the
header text is gone); the same grep on `read_jubarte.md` finds it.

## Verdict

For the body text, pandoc's Markdown and both jubarte readers now agree on
the structure: heading and list syntax, a table that keeps its rows and
columns, and the footnote at its anchor. `jubarte convert input.docx -t md`
also drops the header and footer, exactly as pandoc does. `jubarte read`
keeps them, in its YAML header, together with the page setup, the styles
behind `#` and `##`, the page breaks from the layout, and the `<!-- pN -->`
ids that edits anchor to. For the Anthropic skill's actual purpose —
handing document text to a model — pandoc silently loses the header and
footer; `jubarte read` shows them and loses nothing this folder measures.
The adoption page's description of `jubarte read` matches what this folder
measured.

Until 0.12 this folder ran `jubarte text`, which printed `[body:p:N Style]`
tags instead of Markdown headings and lists and flattened the table to one
paragraph per cell; `read` replaced it.
