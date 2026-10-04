<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Reading a .docx: pandoc vs jubarte

Anthropic's docx skill reads a document with `pandoc -t markdown file.docx`;
the adoption page (`docs/adoption/anthropic-docx-skill.md`) offers
`jubarte text file.docx` as the replacement. This folder runs both on the
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
jubarte text input.docx > read_jubarte.md
jubarte convert input.docx -t md > read_jubarte_convert.md
```

## Tool versions used here

- jubarte 0.11.2 (binary at `/Users/arthrod/temp/T/jr-adopt-bin/jubarte`)
- pandoc 3.11
- python-docx 1.2.0, Python 3.14.7 (input only; not a reader below)

## Outputs

| File | Made by |
|---|---|
| `make_input.py` / `input.docx` | the input and the script that writes it |
| `read_pandoc.md` | `pandoc -t markdown` |
| `read_jubarte.md` | `jubarte text` |
| `read_jubarte_convert.md` | `jubarte convert input.docx -t md` |
| `versions.txt` | tool versions, written by `run.sh` |

## What each reader keeps

| Input feature | pandoc `-t markdown` | `jubarte text` | `jubarte convert -t md` |
|---|---|---|---|
| Heading 1/2 | `#`, `##` | tag only: `[body:p:2 Heading2]`, no `#` | `#`, `##` |
| `**bold**`, `*italic*` | kept | kept | kept |
| Bulleted list | `- ` items | one tagged paragraph per item, no `- ` | `- ` items |
| Table | grid table, rows and columns kept | lost: flattened to one paragraph per cell | GFM pipe table, kept |
| Footnote | `[^1]` marker at the anchor + definition | text kept as its own `[footnotes:p:0]` story; the anchor position is not marked | `[^1]` marker + definition |
| Header | lost (silently) | kept as `[header1:p:0]` story | lost (silently) |
| Footer | lost (silently) | kept as `[footer1:p:0]` story | lost (silently) |
| Paragraph ids | none | `[body:p:N]` before every paragraph | none |
| Punctuation | escaped: `\"`, `\'`, `\_\_\_`; wrapped near 72 columns | as in the document | as in the document |

Verified in the outputs: `grep -c "Draft 3" read_pandoc.md` is 0 (the
header text is gone); the same grep on `read_jubarte.md` finds it.

## Verdict

For the body text, pandoc's Markdown is the more standard rendering: real
heading and list syntax, a grid table, and the footnote at its anchor.
jubarte has two different answers. `jubarte convert input.docx -t md` matches
pandoc on the body — headings, list, a GFM table, the footnote, no escaped
punctuation — and also drops the header and footer, exactly as pandoc does.
`jubarte text` is not full Markdown: headings and list items come out as
style tags rather than `#`/`- `, and the table's structure is lost (each
cell becomes its own paragraph, in reading order). What `jubarte text` has
and pandoc has not: the header and footer stories, the footnote's text as
its own story, and the `[body:p:N]` ids that edit plans anchor to. For the
Anthropic skill's actual purpose — handing document text to a model — pandoc
silently loses the header and footer; jubarte's `text` shows them, at the
cost of the table structure. The adoption page's description of `jubarte
text` ("Markdown with a `[body:p:N]` id before every paragraph, `**bold**`
and `*italic*` from direct run formatting, then each header, footer and
notes story") matches what this folder measured; note that the page does not
claim `jubarte text` renders tables or heading syntax, and it does not.

No discrepancy between the adoption page and what jubarte did here.
