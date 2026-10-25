<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# jubarte's Markdown dialect on one page

> **Identity note (re-checked 2026-10-19, `aba48bd5`):** every id form this
> page lists — `pN`, `tN`, `tN.rowR.cellC`, `headerN`/`footerN`,
> `part.pN`, `rN`, `story.rN`, `r1+2`, `cN`, `cN.text`, closed `pN-pM`
> ranges — is the current short grammar and survives into the 17 October
> 2026 short-and-long-ID contract v3
> (`docs/superpowers/specs/2026-10-17-short-and-long-id-contract.md`,
> documentation-only in `4612a155`, **not implemented**). Today's
> divergences from that contract live outside this page's examples: `noid`
> for an unnumbered change, `r43+43` for carriers sharing one `w:id`, a
> story's paragraph 0 printed as the bare story alias (`header1`, not
> `header1.p0`), the colon fallback (`body:rev:007`) for values with no
> short spelling, nested `.pN` inside cells and text boxes
> (`t0.row1.cell2.p1` — the contract's `psN`), open ranges `p17-`/`-p3`
> (`read -p` still parses them transitionally), and schema-1 JSON (`id`,
> `ref`, `at`, numeric comment ids) where the contract wants paired
> `short_id`/`long_id` under `schema_version: 2`.

CommonMark + GitHub tables, strikethrough, task lists and footnotes, then
CriticMarkup for changes, then the agent view's YAML header and id lines.
[MARKDOWN.md](MARKDOWN.md) has the details.

**Text and blocks**

| Write | Word | Notes |
| --- | --- | --- |
| `**b**` `*i*` `~~s~~` `` `c` `` | bold, italic, strike, `VerbatimChar` | `__b__` is bold too (CommonMark); a `VerbatimChar` run reads back as `` `c` ``, a `SourceCode` paragraph as a fence |
| `==h==` | highlight (yellow) | the agent view prints a highlight as `<mark>h</mark>`, naming any colour but yellow on the tag (`<mark class="hl-green">h</mark>`; `{==…==}` is a comment's scope), strike as `~~s~~` and underline as `<u>u</u>` |
| `{DATE \@ "MMMM d, yyyy"}March 1, 2026{/}` | field: its code, then its result | ECMA-376 field names only, closed by `{/}` in the paragraph, or in a later one when the code begins its paragraph's text (a table of contents: the paragraphs between are its result); `{Client Name}` stays text, `\{PAGE}` and `\{/}` are literal. The agent view prints body fields this way, except hyperlinks (links) and fields with a field in their code, which show their result |
| `<u>u</u>` `<sup>2</sup>` `<sub>2</sub>` `<br>` | underline, superscript, subscript, line break | what the reader prints |
| `<b>` `<i>` `<s>` `<code>` `<mark class="hl-<colour>">` (and `<strong>` `<em>` `<del>`), `<a href>` | bold, italic, strike, code, highlight (yellow by default, the colour the class names), link | read only; other tags warn, and an HTML block keeps only its text |
| `x ^2^`, `x ~2~` | superscript, subscript | word edges only: `H~2~O` stays text |
| `$x$`, `$$x$$` | equation (`m:oMath`, `m:oMathPara`) | LaTeX both ways; `{++…++}` inside marks a change in the math; see MARKDOWN.md, Equations |
| `#`–`######` | `Heading1`–`Heading6` | or the `styles:` legend's heading style |
| `- a` `* a` `+ a` / `1. a` `1) a` / `- [ ] a` | bullets / numbers (start kept) / task | `ListParagraph` |
| `> q`, fenced or indented code | `Quote`, `SourceCode` | |
| `[t](url)` `[t](#a)` `![alt](f.png)` `[^n]` | link, picture, footnote | `#a` is a heading's GitHub slug |
| `![alt][dN]` | shape, text box, connector, VML, `mc:AlternateContent` | the reference definition `[dN]: drawing:…` at the end (after `<!-- drawings -->`) holds a verbatim payload of the element's XML, its relationships and the parts they reach; spliced back whole |
| `![alt][imageN]` | raster picture (PNG, JPEG, GIF, BMP, TIFF), inline or anchored | `[imageN]: <data:mime;base64,…>` holds the bytes, `<!-- imageN drawing:… -->` the anchor's payload; without the comment the `data:` bytes are a plain picture |
| ` ```svg dN ` fence | an `asvg:svgBlip` alone in its paragraph | the fence holds the SVG source; `<!-- dN drawing:… -->` keeps the payload, PNG fallback inside; an edited source writes the new SVG into the part |
| ` ```mermaid dN ` fence | a chart (`c:chartSpace` pie, bar or line) alone in its paragraph | the fence holds `pie`/`xychart-beta`; `<!-- dN drawing:… -->` keeps the payload and a hash of the printed text — unchanged writes the part back verbatim, edited rewrites its categories, values and title |
| `---`, GFM table | bordered paragraph, `TableGrid` table | header row repeats |

**Changes and comments (CriticMarkup, read before the Markdown)**

| Write | Word |
| --- | --- |
| `{++new++}` `{--old--}` `{~~old~>new~~}` | insertion, deletion, substitution |
| `{==text==}{>>note<<}`, `{>>note<<}` | comment on text (its scope), comment at a point |
| `{==text==}` alone | highlight |
| `\{++` | literal text (also `--no-critic`) |

A change across `\n\n` holds the paragraph mark; a block that is one change
whole is added or removed whole; a row whose every cell is one change is an
inserted or deleted row.

**Agent view (`jubarte read`): tags**

| Form | Meaning |
| --- | --- |
| `{>>r12 @AC<<}`, `{>>r1+2 @AC<<}` | revision `r12` (its `w:id` is 12); two marks of one change |
| `{~~a~>b~~}{>>r1 @AC<<}{>>r2 @JD<<}` | substitution by two authors, deleted side first |
| `{>>c5 @AC: text<<}`, `re c5`, `resolved` | comment, reply, resolved thread |
| `header1.r3 @AC`, `footnotes.r7 @AC` | change in another story |
| `@??` | a mark with no author's name; written back as an empty `w:author` |
| no tag | new, by the plan author |

**Id lines**: `<!-- pN[ Style][, clause…] -->` above every body paragraph

`left|center|right|justify` · `first-line|hanging|left|right Xin` ·
`num "1)"` | `bullet` · `level N` · `page-break` |
`page-break-ins|page-break-del TAG` · `section-break` ·
`break-ins|break-del TAGS` · `fmt TAGS` · `rev TAGS` · `comments cN` ·
`in cN` · `empty` (`<!-- p4-p7 empty -->`)

Table: `<!-- tN [center|right] RxC, cells pA-pB by row, pN center, pN Style, header row repeats,
style S, merged tN.rowR.cellC across N down N, gaps tN.rowR before N after N,
break-ins TAGS in pN, rev …, fmt … -->` (`cells tN.row0 pA-pB tN.row1 …` when
a cell holds other than one paragraph).
`pN center|right|left|justify` is a cell paragraph aligned against its column's
`:-`/`:-:`/`-:` (most of the column decides); `pN Style` (`pA-pB Style` for
consecutive ones) is a cell paragraph in a style other than the default (a
style id that is a clause word or holds a space, `,`, `"` or `\` prints
quoted, `"My Style"`, on table and paragraph id lines alike);
`tN center` places the table. Notes: `<!-- footnotes.p0 -->`, `<!-- endnotes.p0 Endnote -->` (paragraph styles other than Word's own, one per paragraph). Pages: `<!-- page 2 of 5 -->`.

**YAML header**

```yaml
comments: 2 threads open, style Normal, c6 style CommentText   # paragraph styles; CommentText unless named
authors:
  AC: Ann Counsel                    # the handle tags name
page:      Letter portrait, margins 1in, header/footer 0.5in   # columns 2, column gap 0.25in, column line, line grid 15.6pt
styles:
  Normal: Calibri 11pt, after 8pt, line 1.08, left   # default style
  "#": Heading1, Calibri bold 16pt, before 12pt, keep-next, keep-lines
  ListParagraph: Calibri 11pt, indent 0.5in, contextual spacing, named List Paragraph
headers:
  first: none                        # or {id: header2, text: "{PAGE}", center}
  default: {id: header1, text: Smith & Co, right}
footers:                             # the same keys; even: turns on even/odd
  default: {id: footer2, text: "Page {PAGE}"}
  even: {id: footer1, text: [A, B], styles: [Footer, Normal]}  # Header/Footer unless style:/styles: says
  first:                             # paragraph by paragraph, when one has its own colour,
    id: footer3                      # borders or alignment
    p0: {text: "docen · output", center, color: "595959", border: bottom single 0.5pt 8496B0 space 4pt}
    p1: {text: "{PAGE}"}
sections:
  2: {p11-p40, page: A4 landscape, headers: {first: none}}
```

Read back: `page:`, `styles:`, `headers:`/`footers:`, `sections:` and the
`comments:` line's style clauses are honoured in any YAML header; id lines and tags only under `view:`. A
change or comment naming no author or date takes `-a`/`-d`, else the front
matter's `author:` and `last_modified:` (or `modified:`, `date:`,
`created:`), else the `--reference-doc`'s `dc:creator` and
`dcterms:created`, else `Redline` and now.
`  # …` after a value is a comment unless it sits inside quotes.

**Escapes the view prints**: a line starting `# ` `- ` `> ` `1. `; `*`
`` ` `` `\` `~~` `==` `[^` tag-like `<`; `_`, `~`, `^` at a word edge.
