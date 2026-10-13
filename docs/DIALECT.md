<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# jubarte's Markdown dialect on one page

CommonMark + GitHub tables, strikethrough, task lists and footnotes, then
CriticMarkup for changes, then the agent view's YAML header and id lines.
[MARKDOWN.md](MARKDOWN.md) has the details.

**Text and blocks**

| Write | Word | Notes |
| --- | --- | --- |
| `**b**` `*i*` `~~s~~` `` `c` `` | bold, italic, strike, `VerbatimChar` | `__b__` is bold too (CommonMark) |
| `<u>u</u>` `<sup>2</sup>` `<sub>2</sub>` `<br>` | underline, superscript, subscript, line break | what the reader prints |
| `x ^2^`, `x ~2~` | superscript, subscript | word edges only: `H~2~O` stays text |
| `$x$`, `$$x$$` | text today | Word equations read out as LaTeX; not yet written back |
| `#`–`######` | `Heading1`–`Heading6` | or the `styles:` legend's heading style |
| `- a` `* a` `+ a` / `1. a` `1) a` / `- [ ] a` | bullets / numbers (start kept) / task | `ListParagraph` |
| `> q`, fenced code | `Quote`, `SourceCode` | |
| `[t](url)` `[t](#a)` `![alt](f.png)` `[^n]` | link, picture, footnote | `#a` is a heading's GitHub slug |
| `---`, GFM table | bordered paragraph, `TableGrid` table | header row repeats |

**Changes and comments (CriticMarkup, read before the Markdown)**

| Write | Word |
| --- | --- |
| `{++new++}` `{--old--}` `{~~old~>new~~}` | insertion, deletion, substitution |
| `{==text==}{>>note<<}`, `{>>note<<}` | comment on text, comment at a point |
| `{==text==}` alone | highlight |
| `\{++` | literal text (also `--no-critic`) |

A change across `\n\n` holds the paragraph mark; a block that is one change
whole is added or removed whole; a row whose every cell is one change is an
inserted or deleted row.

**Agent view (`jubarte read`): tags**

| Form | Meaning |
| --- | --- |
| `{>>r12 @AC<<}`, `{>>r1+2 @AC<<}` | revision `body:rev:12`; two marks of one change |
| `{~~a~>b~~}{>>r1 @AC<<}{>>r2 @JD<<}` | substitution by two authors, deleted side first |
| `{>>c5 @AC: text<<}`, `re c5`, `resolved` | comment, reply, resolved thread |
| `header1.r3 @AC`, `footnotes.r7 @AC` | change in another story |
| no tag | new, by the plan author |

**Id lines**: `<!-- pN[ Style][, clause…] -->` above every body paragraph

`left|center|right|justify` · `first-line|hanging|left|right Xin` ·
`num "1)"` | `bullet` · `level N` · `page-break` · `section-break` ·
`break-ins|break-del TAGS` · `fmt TAGS` · `rev TAGS` · `comments cN` ·
`in cN` · `empty` (`<!-- p4-p7 empty -->`)

Table: `<!-- tN [center|right] RxC, cells pA-pB by row, header row repeats,
merged tN.rowR.cellC across N down N, break-ins TAGS in pN, rev …, fmt … -->`.
Notes: `<!-- footnotes.p0 -->`. Pages: `<!-- page 2 of 5 -->`.

**YAML header**

```yaml
authors:
  AC: Ann Counsel                    # the handle tags name
page:      Letter portrait, margins 1in, header/footer 0.5in   # columns 2, column gap 0.25in
styles:
  Normal: Calibri 11pt, after 8pt, line 1.08, left   # default style
  "#": Heading1, Calibri bold 16pt, before 12pt, keep-next
headers:
  first: none                        # or {id: header2, text: "{PAGE}", center}
  default: {id: header1, text: Smith & Co, right}
footers:                             # the same keys; even: turns on even/odd
  default: {id: footer2, text: "Page {PAGE}"}
sections:
  2: {p11-p40, page: A4 landscape, headers: {first: none}}
```

Read back: `page:`, `styles:`, `headers:`/`footers:` and `sections:` are
honoured in any YAML header; id lines and tags only under `view:`.
`  # …` after a value is a comment unless it sits inside quotes.

**Escapes the view prints**: a line starting `# ` `- ` `> ` `1. `; `*`
`` ` `` `\` `~~` `==` `[^` tag-like `<`; `_`, `~`, `^` at a word edge.
