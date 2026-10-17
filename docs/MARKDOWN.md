<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Markdown and CriticMarkup

jubarte reads Markdown alongside Word documents:

| Task | CLI | Library (`jubarte::markdown`) |
| --- | --- | --- |
| Markdown to Word, CriticMarkup as tracked changes | `jubarte convert draft.md` | `markdown_to_docx` |
| Word to Markdown: the agent view | `jubarte convert FILE.docx -t md` | `read` |
| Word to plain CommonMark, tracked changes and comments as CriticMarkup | `jubarte convert FILE.docx -t md --commonmark` | `docx_to_markdown` |
| Accept or reject CriticMarkup in Markdown | `jubarte convert draft.md -t md --track-changes accept` | `resolve_critic` |
| The changes between any two documents as a patch | `jubarte diff old new` | `patch_documents` |
| Two Markdown documents as CriticMarkup (pandiff) | `jubarte diff old.md new.md --format critic` | `diff_markdown` |
| Any two documents as a Word redline | `jubarte compare a b -o redline.docx` | `redline` |
| A Markdown edit applied to a Word document | `jubarte compare contract.docx edited.md -o redline.docx` | `apply_markdown` |

`jubarte convert FILE.docx -t md` prints the [agent view](#agent-view), as
`jubarte read FILE` does: the Markdown with a YAML header, an id line before
every paragraph and an id on every change and comment, which comes back
whole through `jubarte convert FILE.md`. `-o FILE.md` writes it;
`--track-changes accept` or `reject` writes the text after Word's Accept
All or Reject All.

`--commonmark` writes plain CommonMark instead, for readers outside this
dialect: each tracked change and comment as CriticMarkup, a change
followed by its number, author and date (`{++new++}{>>r5 Ana Lima
(2026-09-29T14:05:00Z)<<}`; the number tells the writer the note names who
made the change, not a comment on it). Fields print as their text, and
underline, highlight and strike are left out. Pictures become their alt
text. Hidden text (`w:vanish` on the run, or on its character style
directly or through `basedOn`) prints nothing, in the body, table cells,
notes and headers.

## Markdown to Word

`jubarte convert draft.md` writes `draft.docx` next to the Markdown
(`-o` names another file, `-o draft.pdf` or `-t png` renders it). The
Markdown is CommonMark with GitHub's tables, strikethrough, task lists and
footnotes, and a YAML front matter's `title:` and `author:` become the
document's properties. In a table row an even backslash run before a pipe
(`\\|`) is escaped backslashes and then the cell separator, as GFM reads
it; a pipe behind an odd run (`\|`) is a literal pipe in the cell. Its
`track_changes: on` (or `off`) sets Word's Track
Changes switch in the output; `--tracking on|off` overrides the key. With
neither, the output keeps the `--reference-doc`'s switch, else tracking is
off. Tracked changes the Markdown holds never turn the switch on by
themselves.

A change or comment that names no author or date (`{++new++}`, an
untagged `{>>note<<}`) takes them, in this order, from:

1. `-a/--author` and `-d/--datetime`;
2. the front matter: `author:`, and `last_modified:`, else `modified:`,
   `date:` or `created:` (a timestamp, or a bare day for its midnight; any
   other value is named in a warning);
3. the `--reference-doc`'s `dc:creator` and `dcterms:created`
   (`docProps/core.xml`);
4. `Redline` and now. Nothing is read from the file's owner or
   modification time.

A page break is a line of its own: `\newpage`, `\pagebreak`,
`\clearpage` or `\cleardoublepage` (LaTeX's, as Pandoc manuscripts write
them), or `<!-- page-break -->`, the line the plain Markdown of `convert
--to md` prints for one. The paragraph above it holds the break, as Word
puts one; at the top of the document an empty paragraph does. In code the
commands stay text. The agent view says the same with its id line's
`page-break` clause.

`--scholar` (Python `from_markdown(scholar=True)`, wasm `scholar: true`)
sets a manuscript as LaTeX's article class does: Latin Modern Roman 10pt
on 12pt lines, justified, 6pt between paragraphs, bold headings at 14.4,
12 and 10pt with LaTeX's spacing, and lines ending in a line break (an
author block) left unstretched (`w:doNotExpandShiftReturn`). The page
number goes in the footer, centered, from page 2. It only fills in what the
front matter leaves out: the `styles:` entries it names (a default style of
its own keeps `Normal`), and its `headers:`/`footers:` (no page number is
added when they hold `{PAGE}`; with headers of their own the number starts
on page 1, since a title page would hide them there). Latin Modern Roman is
TeX's font and is seldom installed: Word and jubarte draw it in a
substitute (Cambria on macOS) until it is.

An author the `authors:` table names, and a date a tag carries, come before
all of these. Pin `-d 1970-01-01T00:00:00Z` for byte-identical output from
identical inputs. This represents the document, so its own metadata
counts; `compare`, `edit`, `add` and `diff` change a file instead, and
sign and date their changes with `--author` (default `Redline`) and
`-d/--datetime` (default now) only, never the files' metadata.

Library calls keep that epoch instead of now and read no reference
document's metadata: `DocxOptions::default()` is `Redline` at
1970-01-01T00:00:00Z, and the front matter's `author:` and date beat the
options' `author` and `date` unless `author_fixed` or `date_fixed` is set.
Python's `from_markdown` and WASM's `markdownToDocx` set them for an
`author` or `date` they are given. `markdown::docx_stamp(docx)` returns a
`.docx`'s first non-empty `dc:creator` and its `dcterms:created`, as the
CLI reads them.

| Markdown | Word |
| --- | --- |
| `#` to `######` | `Heading1` to `Heading6` |
| paragraph | `Normal` |
| `**bold**`, `*italic*`, `~~struck~~` | bold, italic, strikethrough |
| `==marked==` | yellow highlight; `==` beside a space, inside a longer `=` run, escaped or unclosed in its paragraph is text. The agent view prints a highlight of any colour as `<mark>marked</mark>`, so `{==…==}` is only ever a comment's scope, and strike as `~~struck~~`; the spaces at a highlighted, underlined or struck stretch's edges stay inside its marks as `&#32;`. A highlight whose `w:highlight w:val` is anything other than yellow names the colour on the `<mark>` tag (`<mark class="hl-green">…</mark>`, `<mark class="hl-cyan">…</mark>` …) so the round-trip keeps the colour: Word's 16 named highlight colours round-trip (`yellow`, `green`, `cyan`, `red`, `blue`, `magenta`, `darkYellow`, `darkGreen`, `darkCyan`, `darkBlue`, `darkRed`, `darkMagenta`, `darkGray`, `lightGray`, `black`, `white`); a class the writer does not recognise falls back to yellow |
| `{REF _Ref1 \h}Section 2{/}` | a field (`fldChar begin`, `instrText`, `separate`, the result, `end`). The code must start with a field name ECMA-376 lists (any case) and close with `{/}` in the same paragraph, so a placeholder such as `{Client Name}` stays text; `\{` makes a brace literal. A code that begins its paragraph's text (after any `#`, list or `>` marker) may close in a later paragraph, the paragraphs between being its result: a table of contents, `{TOC \o "1-3"}` at the start of its first entry and `{/}` after its last. The agent view prints a body field this way when it has no field inside its code, is not a hyperlink, and either closes in its paragraph or begins its first paragraph with only paragraphs (no table, no tracked change around its marks) up to its end; otherwise it prints the result only |
| `` `code` `` and fenced code | `VerbatimChar` and `SourceCode` (pandoc's names) |
| `- item`, `1. item` | numbered paragraphs (`ListParagraph`); an ordered list starts at its first number |
| `> quote` | `Quote` |
| table | Word's thin grid (`TableGrid`, single 0.5pt borders all round, as Insert Table makes it) unless the view names another style: the legend's `table:` for every table, a table line's `style X` for its own. `TableNormal` is Word's Normal Table, written with no `tblStyle`; the view names it for a table without a style. The borders a table carries itself come back as they were: `borders single 0.5pt 000000` (`w:tblBorders`; sides that share a border group as `top,bottom single 1.5pt`, groups joined by ` + `, a side turned off is `none`, a group naming no sides covers all six) `row-borders top double 0.5pt FF0000 in t0.row1 t0.row3` (a row's own `w:tblPrEx/w:tblBorders`; groups joined by `; `) and `cell-borders top,left,bottom,right single 1pt CCCCCC in all` (`w:tcBorders`, for `all` cells, a row `t0.row1` or a cell `t0.row1.cell2`; groups joined by `; `). A border in a theme colour adds it after its fallback colour: `single 1pt 2F5496 theme accent1 tint 99 shade BF` (`w:themeColor`, `w:themeTint`, `w:themeShade`). A source that defines `TableGrid` with one single border other than 0.5pt keeps it: the legend says `table: TableGrid, all borders 1.5pt`. Header row repeated, column alignment kept |
| `[text](url)`, `[text](#anchor)` | hyperlink; `#the-intro` lands on the heading whose GitHub slug it is, through a bookmark (`_` before a digit, cut to 40 characters); a wholly deleted heading, an empty one or one in a footnote has none, and a link to a heading that is not there is written with a warning. Reading Word back, reader and writer share github-slugger's names (a repeat `-1`, `-2`): only the exact name the writer gives a heading is its own bookmark and prints no anchor; any other (`title-01`, a lone `title-1`) stays an `<a id>` anchor, and a heading in a table cell claims no slug. A title (`[t](u "Title")`) is the hyperlink's tooltip, and `<a href="u">t</a>` is a link too |
| `[^note]` | footnote; a label defined twice (a footnote and an endnote both `[^1]`) is two notes: its first reference names the first definition and its next the second, which takes the next free number, with a warning |
| `![alt](file.png)` | picture (PNG, JPEG, GIF, BMP, TIFF), at most the text width; else its alt text |
| `![alt][dN]` with `[dN]: drawing:PAYLOAD` | a drawing the readable forms cannot say — a shape, text box, connector, VML picture or `mc:AlternateContent`. The reference's name is the drawing's number in reading order; the definition sits with the others after a `<!-- drawings -->` marker at the end of the view, so the body text stays free of payloads. `PAYLOAD` is URL-safe base64 of a JSON envelope the reader writes: the element's XML, its relationships, and every part they reach. The writer splices it back verbatim (fresh relationship and `docPr` ids, each part re-written); a payload that does not decode is dropped with a warning. The older inline form `![alt](drawing:PAYLOAD)` reads the same |
| `![alt][imageN]` with `[imageN]: <data:mime;base64,…>` and `<!-- imageN drawing:PAYLOAD -->` | a raster picture (PNG, JPEG, GIF, BMP, TIFF), inline or floating. The `data:` definition holds the image's bytes; the backing comment the payload of everything else (anchor, wrap, `docPr`). Written back, the bytes fill the payload's image part; an edited `data:` writes the new image. Two identical payloads share one name and one definition. Without its backing comment the `data:` picture is a plain inline image |
| ` ```svg dN ` fence with `<!-- dN drawing:PAYLOAD -->` | an `asvg:svgBlip` the paragraph holds alone, as its source code: the fence's text is the SVG part, the backing comment the payload with the PNG fallback inside. An edited source writes the new SVG into the part and keeps the fallback |
| ` ```mermaid dN ` fence with `<!-- dN drawing:PAYLOAD -->` | a chart the paragraph holds alone, as its data: a `pie` fence for a pie chart, `xychart-beta` for bar and line. The backing comment holds the payload and a hash of the printed text: a fence that comes back unchanged writes the chart part byte for byte; an edited one rewrites the part's categories, values and title in place — styling, axes and everything else the Mermaid cannot say stay the part's own. A chart that shares its paragraph with text keeps `![alt][dN]` instead |
| `---` | a paragraph with a bottom border |
| `<br>`, `<sup>`, `<sub>`, `<u>` | line break, superscript, subscript, underline (in a cell or a note of the tracked view, `<br>` ends a paragraph and `<br/>` is a line break; see below) |
| `<b>`/`<strong>`, `<i>`/`<em>`, `<s>`/`<del>`, `<code>`, `<mark class="hl-<colour>">` | bold, italic, strike, `VerbatimChar`, highlight (yellow by default; the colour Word holds through `class="hl-<colour>"`, the names the table above lists); attributes and case do not matter, a tag left open closes with its block, and any other tag keeps its text and is named once in a warning |
| `x ^2^`, `x ~2~` | superscript, subscript, at word edges only (pulldown-cmark opens neither inside a word: `H~2~O` is text) |
| `$x^2$`, `$$\frac{a}{b}$$` | an equation (`m:oMath`; display math in an `m:oMathPara`), see below |

Images are read relative to the Markdown file, or to `--resource-path`;
URLs are not fetched.

`--reference-doc house.docx` works as pandoc's: the output takes the
reference's styles, numbering, page setup, headers and footers, and none of
its text. Styles the reference lacks are added with jubarte's definitions.
A pandoc reference document works, since the style ids are Word's built-in
ones plus pandoc's for code.

Without a reference, `--page letter` (the default) or `--page a4` picks the
page size; both keep one-inch margins, where Word's own A4 template uses
2 cm. With a reference, its page setup wins and `--page a4` prints a
warning.

The same writer is in Python and JavaScript:

```python
import jubarte_redlines

doc = jubarte_redlines.from_markdown("Due in {~~30~>45~~} days.", page="a4")
doc.to_bytes()  # the .docx
```

```js
const docx = markdownToDocx("Due in {~~30~>45~~} days.", JSON.stringify({ page: "a4" }));
```

`python -m jubarte_redlines convert draft.md` writes `draft.docx` with the
same flags. Neither binding reads image files: pictures are written as
their alt text.

### Equations

LaTeX between `$` (inline) or `$$` (display) becomes Word's own equation,
and the reader prints Word's equations back in the same LaTeX, so an
equation round-trips: Word's own files (`tests/fixtures/math`) come back
with every structure Word wrote.

| LaTeX | Word |
| --- | --- |
| `\frac{a}{b}`, `\binom{n}{k}`, `\sqrt[3]{x}` | fraction, bar-less fraction in brackets, radical |
| `x^2`, `x_i`, `x_i^2`, `\prescript{a}{b}{X}` | scripts; a script's base is the character, group or structure before it |
| `f'`, `f''`, `f'^2` | a prime superscript (′ ″ ‴), as Word builds `f'`; it reads back as the apostrophes |
| `(…)`, `[…]`, `\{…\}`; `\left⟨ a \middle\| b \right.` | a delimiter: balanced brackets build one, as Word builds `f(x)`; `\left` names any other; `\bigl(` … `\Biggr)` are their bracket, as Word's delimiter grows to its content |
| `\sin x`, `\lim_{n\to\infty} a_n`, `\operatorname{argmax}_x f` | a function, its limit under it |
| `\sum_{i=1}^n x_i`, `\int_0^1`, `\iint`, `\bigcup` … | n-ary operator; it applies to the next character or group and its scripts |
| `\hat`, `\tilde`, `\bar`, `\dot`, `\ddot`, `\vec`; `\overline`, `\underline` | accent; bar |
| `\underbrace{x}_{n}`, `\overbrace{x}^{n}`, `\underset{a}{b}`, `\overset{a}{b}` | brace and limit, stacked limit |
| `\begin{matrix}`, `pmatrix`, `bmatrix`, `array` …; `aligned`, `cases` | matrix (`array`'s column spec is dropped); equation array, aligned at each `&` as Word aligns it |
| `\mathrm`, `\mathbf`, `\boldsymbol`, `\mathbb`, `\mathcal`, `\text` | the run's math style, script or ordinary text |
| `\alpha` … `\Omega`, `\pm`, `\le`, `\to`, `\infty` … | the character itself |
| `\not\in`, `\not=`, `\not\perp` | the negated character (∉, ≠), else the character with U+0338 over it |
| `\bmod`, `\pmod{n}`, `\hspace{…}` | upright mod; (mod n); a space |
| `\lparen`, `\lbrack`, `\lbrace`, `\backslash` | a bracket or backslash that is text |

A top-level `\\` breaks the equation into lines of an equation array:
Word draws the equations of one paragraph side by side. Spaces inside
text stay, as Word keeps them; at an argument's edges they go, as LaTeX
ignores them. An unknown command stays as its text, with the braces of
an argument right after it (`\foo{x}`). Inside the math,
`{++…++}` and `{--…--}` mark the runs they hold as inserted or deleted,
the way Word marks a change inside an equation; an equation inserted or
deleted whole reads as `{++$E=mc^2$++}`. Every run is set in Cambria
Math, as Word sets it.

## CriticMarkup

[CriticMarkup](https://criticmarkup.com) marks changes in plain text. jubarte
reads it before the Markdown, as the specification asks, so a change can
cross emphasis, links and paragraph breaks.

| CriticMarkup | Word |
| --- | --- |
| `{++new++}` | inserted text |
| `{--old--}` | deleted text |
| `{~~old~>new~~}` | deleted, then inserted text |
| `{==text==}{>>note<<}` | a comment on `text` (`text` is the comment's scope, Word's `commentRangeStart` to `commentRangeEnd`) |
| `{++new++}{>>note<<}` | a comment on the change |
| `{>>note<<}` alone | a comment at that point |
| `{>>note<<}` in a footnote, header or footer | Word allows comments only in the body: in a footnote the comment moves to the note's reference mark, in a header or footer it is dropped and its text kept; both warn |
| `{==text==}` alone | highlighted text |

A change or comment with no author or date of its own takes them as
[Markdown to Word](#markdown-to-word) lists.

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
(`\{++`) or without its partner is text too, and says so in a warning.

A comment or highlight whole inside a change is text to the change's
own delimiters: `{~~a{>>x ~> y<<} b~>c~~}` substitutes `a b` for `c` and
keeps the comment's `~>`, `{++a{>>b++}c++}` inserts `ac` around a
comment that holds `b++}`. A blank line inside `{>>…<<}` is a paragraph
break in the comment's text, not in the paragraph it sits on.

A raw HTML block keeps only its text, and CriticMarkup written in that
text is read as anywhere else: `<div>{++x++}</div>` inserts `x`.
A numeric character reference naming a stand-in character
(`&#x2E0E;`, `&#xF0000;`) is literal text wherever the parser decodes
it — paragraph text, a comment's body, an HTML block — because
references decode only after CriticMarkup was read.

## Agent view

`jubarte read FILE.docx` prints the agent view: a YAML header
with the facts an editor needs, then the document as Markdown and
CriticMarkup, with an id line before every paragraph and an id on every
tracked change and comment. The ids are the ones `edit`, `add`,
`accept --id` and `reject --id` take, and `changes` and `comments` print.
The `source:` line names the document with `/` between the path's parts
on every OS — it is an identifier an agent reads back, as are the paths
in `--json` output and the bundle's `report.jsonl`; the status lines on
stderr (`wrote …`) keep the OS's own spelling.

```text
---
source: received.docx
sha256: e42d9d7cec67a3ceff4714cddd5e2306b23c8fc973a9d262f4aa637041c67e68
view: tracked                      # revisions as CriticMarkup, comments inline
track_changes: off                 # w:trackRevisions not set; new edits are not tracked unless edit sets it
revisions: 9                       # 9 marks shown as 5 changes: 1 insertion, 4 substitutions
comments: 2 threads open, c6 style Normal  # 3 comments: c5 (+ reply c6), c11
authors:
  document_owner: Arthur Souza Rodrigues  # cp:lastModifiedBy; no dc:creator
  AC: Ann Counsel                  # 9 revisions, 2 comments, 2026-10-01T09:00:00Z
  AS: Arthur Souza Rodrigues       # 1 comment, 2026-10-09T16:13:00Z
body: p0-p20, 1 table, 2 pages     # pages from layout
page: Letter portrait, margins 1in, header/footer 0.5in
styles:
  Normal: Calibri 11pt, after 8pt, line 1.08, left  # default; unannotated paragraphs use it
  "#": Heading1, Calibri bold 16pt, before 12pt, after 4pt, keep-next, keep-lines
  "##": Heading2, Calibri bold 14pt, before 12pt, after 4pt, keep-next, keep-lines
headers:
  first: {id: header2, text: DRAFT}  # page 1 only (different first page)
  default: {id: header1, text: SIGNATURE PAGE, right}
footers:
  first: none                      # page 1 shows no page number
  default: {id: footer2, text: "{PAGE}"}
  even: {id: footer1, text: "{PAGE}", inactive}  # defined, but even/odd headers are off
---

<!-- page 1 of 2 -->

<!-- p0 center -->
# Consulting Agreement

<!-- p1 justify -->
This Consulting Agreement (the “Agreement”) is made between Harbor Point Analytics LLC ("Consultant") and Juniper Freight Co. ("Client").

<!-- p2 -->
## 1. Services

<!-- p3 -->
Consultant will deliver the services in each Statement of Work, including {++quarterly++}{>>r0 @AC<<} route-cost reports and a {~~weekly~>monthly~~}{>>r2+1 @AC<<} review call.
```

`--track-changes accept|reject` prints the document with every change
resolved, `--comments none` leaves the comments out of the text (their ids
move to the id lines), `--dates` puts timestamps in the notes of authors
with several, `-p p3,p10-p20,t0`, `--head N` and `--tail N` print a
selection, and `--no-page-markers` skips the layout pass.

### Id lines

One per body paragraph, on the line above it: a head, one space, then
clauses joined by `, ` (no clauses: `<!-- p2 -->`).

```text
head:     p{N}[ {Style}][ outline {n}]
clauses:  {align} · first-line {x}in · hanging {x}in · left {x}in · right {x}in · num "{label}" | bullet · level {n} · page-break | page-break-ins {tag} | page-break-del {tag} · section-break · break-ins {tags} · break-del {tags} · fmt {tags} · rev {tags} · comments c{ids} · in c{ids}
examples: <!-- p0 center -->   <!-- p18 first-line 0.5in, comments c11 -->   <!-- p0 Quote justify, hanging 0.25in, left 0.5in -->   <!-- p3 rev r0 @AC; r2+1 @AC -->
```

- `N` counts every `w:p` under `w:body` in document order, table cells
  included, text boxes excluded.
- `Style` prints whenever the paragraph's own `w:pPr` states a `w:pStyle`,
  the document's default and `Normal` included: the statement is the
  document's, not the view's to drop. A heading's own `Heading{level}` is
  the one exception, the `#` count says it — an empty paragraph has no
  `#`, so even that one prints (`<!-- p19 Heading2 empty -->`).
- `outline {n}` prints when the paragraph states its `w:outlineLvl` with
  no style at all: `# Text` alone would give the writer's `HeadingN`
  where the document had none (the writer then writes the `w:outlineLvl`
  alone). `n` runs 1 to 9; past the heading styles (`outline 7` to
  `outline 9`) no `#` count reaches the level, so the paragraph stays
  body text and the clause alone carries it.
- `align` prints only when the paragraph sets `w:jc` itself: `center`,
  `right`, `justify`, else `left`. Indents come from the paragraph's own
  `w:ind`, in inches, with as many decimals (two to four) as it takes to
  read back the same twips (`0.7875in` is 1134).
- `num "1)"` is the label Word prints (the level's `w:lvlText`, each `%k`
  in its level's number format: `1)`, `a)`, `ii.`, `1.2`; a `"` or `\` in
  it is escaped with `\`). A level text naming a deeper level (`%3` on
  level 2) gets no label, and no list indent, from Word: the view shows
  a plain paragraph. `bullet` is a
  bulleted item. Labels are not document text, and a list body keeps its
  Markdown marker (`1.`). An item past its list's first level (Word's
  `w:ilvl`) is indented to the content column of the item it nests under,
  and its id line with it, so the Markdown reads as a nested list:

  ```markdown
  <!-- p14 ListNumber num "2." -->
  2. Whisk dry ingredients.

     <!-- p15 ListNumber bullet -->
     - Sift the flour twice.
  ```

  `level 1` (to `level 8`) follows the label only when the indentation
  cannot say the level: a list that starts past its first level, or an
  item that skips one. A skipped level adds no indentation (four columns
  past an item's content would read as code). Reading a view back, an
  explicit `level N` wins over the indentation.
- `page-break`: the paragraph holds `w:br w:type="page"`. In the tracked
  view, a page break inside an insertion or deletion names that change
  instead: `page-break-del r282 @SW` (it would otherwise be a `rev` tag,
  which does not say what the change holds). The clause holds one break,
  written back at the paragraph's end: a second page break in the same
  paragraph, or a break between its runs, does not keep its own place yet.
  `section-break`:
  its `w:pPr` holds `w:sectPr`; reading the view back, it takes the page
  the YAML header names for its section (`page:` for the first,
  `sections:` for the others). A page that states how its section starts
  (`w:type`) says so: `starts nextPage` — Word's default, but the
  statement is the document's — or `starts continuous`, `evenPage`,
  `oddPage`; the writer puts the `w:type` first in its `w:sectPr`. A
  page's text columns are its clauses
  too: `columns 2`, `column gap 0.25in` when not Word's 0.5in, and
  `column line` for a rule between them (`w:cols w:sep`). A grid the lines
  snap to (`w:docGrid` with a type) is a clause as well: `line grid 15.6pt`,
  `line and char grid 18pt` or `char grid 18pt` (a character grid with no
  line pitch is plain `char grid`), then `char space N` when a character
  grid sets one. A pitch with no grid type is not printed:
  Word does not snap to it.
- `break-ins`/`break-del`: a tracked paragraph mark. `fmt`: a formatting
  change (`w:rPrChange`, `w:pPrChange`).
- `rev`: in the accept and reject views, every revision the paragraph
  held before resolution, formatting changes and paragraph marks included.
  A paragraph joined into the one before it hands its revisions over. In
  the tracked view, the insertions, deletions and moves its text has no
  tag for: a deleted picture, line break, field character or empty run
  (a tracked page break has its own clause).
- `comments`: with `--comments none` only, the comments whose ranges or
  references the paragraph holds.
- `in c5`: with comments inline, the comment threads whose range runs on
  past the paragraph. CriticMarkup cannot span paragraphs, so such a range
  repeats `{==…==}` in each and shows its note only where it ends; the
  clause says which thread a scope belongs to.
- Empty paragraphs: `<!-- p19 empty -->`, consecutive ones
  `<!-- p19-p23 empty -->`. An empty paragraph that holds a tracked mark, a
  formatting change, a resolved revision, a hidden comment or a stated
  style keeps its own line: `<!-- p4 empty, break-ins r3 @AC -->`,
  `<!-- p19 TitlePage empty -->`; a range of styled ones
  (`<!-- p19-p23 TitlePage empty -->`) stands for every paragraph it
  numbers, each with what the line says.
- Page markers: `<!-- page N of M -->` before the first block that starts on
  page N, `<!-- page 1 of M -->` first. They come from the layout pass.
  With `--no-page-markers` none print; through the library without layout
  pages they come from Word's cached breaks (`w:lastRenderedPageBreak`),
  else from hard page breaks and from section breaks that start a page.

A table gets one line: `<!-- t0 center 3x2, cells p8-p13 by row, header row
repeats -->`, `cells t0.row0 p8-p10 t0.row1 p11-p14` when a cell holds other
than one paragraph, or when a row whose cells and gaps take more columns
than the grid has holds an empty cell the view could not tell from padding
(the writer adds such a row's missing cells back empty), then
`p10 center` for each cell paragraph aligned against its column (the delimiter row's `:-`, `:-:` or `-:` is the
alignment most of a column's cell paragraphs have; left and none count
alike, a tie or justified text prints `-`; the `center` or `right` after
`t0` places the table itself on the page), then `merged t0.row0.cell1 across 2; t0.row1.cell0
down 3` for a cell spanning grid columns or a vertical merge and the rows
it covers (the view prints a span's other columns as empty cells, and
leaves out empty columns at the right; the reader counts both back, and a
merge keeps its span even past the room a row's gap leaves),
`gaps t0.row1 before 1 after 2` for the grid columns a row leaves out
(Word's `w:gridBefore`/`w:gridAfter`, and a row shorter than the grid),
written back exactly as it says: empty Markdown cells past the line's
`RxC` columns are padding, and the grid has exactly those columns,
`break-ins r12 @AC in p9`,
`break-del …`, `rev … in pN` (resolved views; `rev r5 @AC in t0.row1.cell2`
for a cell's own insertion, deletion or merge in the tracked view, and
`rev r6 @AC in t0.row1` for an inserted or deleted row with no text),
`fmt r12 @AS` for the table's own property changes (`in t0.row1`,
`in t0.row1.cell2` for a row's or cell's), with `--comments none`
`comments c9 in p11`, and `in c5` for a comment range open across it.
`tN` counts top-level body tables.

`styles:` gives the default paragraph style's whole look (font, size,
spacing, alignment), each heading level's style with what differs from
it, then every other paragraph style the body uses, keyed by its id
(`TOC2: Times New Roman 12pt, after 6pt`), described the same way. A line
names only what its style sets apart from the style it is based on:
font and size, `before`/`after`, `line`, alignment, `keep-next`,
`keep-lines`, `indent Xin` with `first-line`/`hanging Xin`, and
`contextual spacing` (no space between paragraphs of that style, as List
Paragraph has). A line ends `, named Text Heading 2` when the style's name
is not its id (for a `HeadingN` heading, not `heading N`). The `table:`
line of the legend ends `, named Normal Table` the same way when the
style's name is not the one the writer would give its id; a style the
writer knows as Word's own (Table Grid, the Grid and List Tables, the
older Light Shading and Medium List families) needs no `named`, and is
written as Word defines it, shading bands and all — but its `named`,
when the view gives one, renames it: the document renamed the built-in.
A table style's name that holds a comma or a quote quotes in the legend
(`named "Foo, named Bar"`, a JSON string), so the clause's grammar
stays unambiguous — and the writer writes no comma into a style's
`w:name` (Word forbids one, reading it as an alias list): the comma
becomes a space, and the conversion says so in a warning.
When the tables differ the legend is plural, one entry per style in
first-use order (`tables: TableNormal, Ledger, borders vary`), each
entry saying what the singular `table:` line says (the borders clause,
the name). Converting the
view back defines each style as its line says, and a style Word knows
(`ListParagraph`, `Quote`) keeps Word's own gallery settings.

Each footnote and endnote definition follows the id line of its
paragraphs: `<!-- footnotes.p0 -->`, or `<!-- endnotes.p0-p2 -->` for a note
of three paragraphs, which the definition joins with `<br>` (a table in a
note reads as its cells' paragraphs), and a line break inside one of them
reads `<br/>`. A note whose paragraphs are not in Word's own footnote or
endnote text names their styles after the id, one per paragraph, the last
for every paragraph after it: `<!-- endnotes.p0 Endnote -->` for a note in
LibreOffice's `Endnote`, `<!-- footnotes.p1-p2 FootnoteText, "My Note" -->`
for one whose second paragraph is in `My Note` (an id that is no plain word
is quoted). A silent line is Word's own style, spelled as Word spells it
(`footnote text`); a document that renames it (`Footnote Text`) names the
style, and the styles legend the name, so the writer defines it as
spelled. A table cell does the same: `<br>` ends one of the cell's
paragraphs, `<br/>` is a line break (`w:br`) within one. A view written
before that distinction still converts: when the id or table line's
paragraph count and the `<br>`s disagree, the writer reads every `<br>`
there as a line break (for a note, with a warning). A `<br>` prints inside a change only when its paragraph's
mark is tracked: `this<br>{++added++}` is an inserted paragraph after an
untracked mark, `{++A<br>B++}` an insertion that takes the mark with it. `edit -p footnotes.p0` edits that
note's text. A note Word inserted or deleted whole, its reference with it,
reads `{++[^1]: text++}{>>r10 @AL<<}`, and the marks its own text holds
close the definition: `<!-- rev footnotes.r11 @AL; footnotes.r12 @AL -->`.
A label defined once and referenced twice is one note; a label defined
twice is two (see the table above).

`sha256:` is the SHA-256 of the bytes the view read: the file, or for
`jubarte A B` the redline it prints in memory. Compare it with the file on
disk before editing to know the view is current.

Every change `jubarte changes` lists prints its tag somewhere in the
tracked view, so an agent can name it. Changes with no body text show in
the header: a section's own change on `page:` (`page: Letter portrait; fmt
r9 @AC`) or as `fmt:` in its `sections:` entry; style changes as
`style_changes: {Normal: styles.r3 @AS}`, a key of its own beside
`styles:`, so no style id can collide with it; a header's or footer's
changes its text does not tag as `rev:`/`fmt:` in its entry; and changes in
notes no definition shows (Word's separators) as `notes: {fmt: …}`;
converting back puts a part's `fmt` changes on its separator, then its
continuation separator (the view does not say which a lone one was on;
its tag, author and date come back). The clause also updates a reference
file's separators; removing it removes their old formatting-change marks.
A `fmt` clause alone creates the notes part it names. A
note's own changes its text cannot show close its definition line:
`[^1]: Source <!-- fmt footnotes.r7 @AC -->`. A first-page header or footer
prints as `first: {…, inactive}` when different-first-page is off.
A header's or footer's paragraphs take Word's Header or Footer style unless
the entry names theirs: `style: Normal` for all, or `styles: [Footer,
ListParagraph]`, one per paragraph from the part's first through its
last. A header's or footer's `text` runs from its first paragraph with
text through the part's last, so empty paragraphs after the text stay
(`text: [DRAFT, ""]`): Word lays each out as a line of the header. A paragraph
with no style of its own prints the document's default style (`Normal` in
most documents), which writes no `w:pStyle`; any other, `Normal` under
another default too, writes one.
A comment's paragraphs take Word's CommentText style unless the
`comments:` line names theirs: `style Normal` for every comment, and
`c6 style CommentText` for one that differs (a style id that is no plain
word is quoted, as on id lines). A comment's style is its first
paragraph's, and all its paragraphs take it.
A table cell's paragraph has no id line, so the table line names its
formatting changes: `fmt r6 @AC in p2`. Whatever change no other line
shows (a change in a text box with no text to tag, or in Word's Fallback
copy of one with no twin in the copy the view shows) is listed by
paragraph on the header's `unplaced:` line: `unplaced: [r3 @AC in p0]`.
A change inside another (a cell's text in an inserted row) reads from
its container's tag.

The header's `comments:` line counts open and resolved threads, and apart
from them a comment no story anchors (no range or reference in the body or
a note): `1 thread open, 1 unanchored`. Word draws no balloon for it. A
comment anchored in a header or footer is unanchored too: Word shows a
comment only from a reference in the main story.

### Tags

A tag is `r<ids> @<handle>`: `r0 @AC`, or `r1+2 @AC` for two Word marks of one
logical change. The number is the change's `w:id` when that is one canonical
decimal: `r12` is the change with `w:id` 12; a change in
another story names it, `footnotes.r3 @AC` or `header1.r1 @AC`. Several
authors' tags on one id line are joined by `; ` (`rev r1 @AC; r2 @JD`).
Three fallback spellings are transitional: two marks Word wrote with one
`w:id` print the number twice (`r43+43`), a mark with no `w:id` prints
`noid`, and one whose value no short spelling reads back prints its colon
id (`body:rev:007`). The 17 October 2026 short-and-long-ID contract
([superpowers/specs/2026-10-17-short-and-long-id-contract.md](superpowers/specs/2026-10-17-short-and-long-id-contract.md),
§4) retires all three — `rN` becomes an allocated label that keeps the
source number only when it is canonical, inside Word's `int` range and
unambiguous, so a pair sharing one `w:id` prints `r0+1` — but the change is
staged in the contract's plan, not landed.

```text
{++quarterly++}{>>r0 @AC<<}
{--ninety days--}{>>r7 @AC<<}
{~~weekly~>monthly~~}{>>r2+1 @AC<<}          one author: one note, deleted ids first
{~~old~>new~~}{>>r2 @JD; r1 @AC<<}            two authors: deleted side first
{++bold words++}{>>r7+8 @AC<<}                 neighbouring marks by one author
{++quarterly++}{>>r0 @AC 2026-10-03T14:05:00Z<<}   with --dates
{--deleted--}{>>r3 @BO on r1 @AL<<}         a change that was another's only content
```

A change that was the only content of another — `A` inserted text, `B`
later deleted it — has nothing of the outer change left to split around
it, so its note names the outer one after ` on ` and the outer change's
own tag, each with its own timestamp as notes print them: `r3 @BO on
r1 @AL`, or `r3 Ben Ode (2026-09-30T09:00:00Z) on r1 Ana Lima
(2026-09-29T14:05:00Z)` in the plain view. The writer puts the change
back inside the one it names. A row's or cell's own mark is never
named: the writer rebuilds those where they stand. Deeper nests chain
the same way, innermost first (`r5 @CD on r3 @BO on r1 @AL`); Word
refuses a revision nested in its own kind, so the writer rebuilds only
the first outer a chain names.

A plain note's grammar is exact about where the name ends and the date
begins: the date is the last parenthesised group, so a name may hold
parentheses of its own (`r1 Ana (consultant) (2026-09-29T14:05:00Z)`
names `Ana (consultant)`). In a note that names another change after
` on `, each name spells its own parentheses percent-wise — `(` as `%28`,
`)` as `%29`, `%` itself as `%25` (`r3 Ben Ode (…) on r1 Ana %28consultant%29
(…)`) — so only the dates' parentheses are bare and the parts of the
note split without doubt: the last ` on ` whose left side parses as one
part and whose right side as the chain the rest names wins, so a name
holding ` on r1 Kim` of its own stays one name. A part with a bare
parenthesis in its name makes the note no fold: it reads as the change's
plain attribution, or as a comment.

Neighbouring marks of one kind by one author join, then an insertion
directly followed by a deletion pairs as a substitution, left to right:
Word's Compare writes the insertion first. A deletion followed by an
insertion stays two changes. A bookmark the view prints (`<a id>`) and a
comment range it prints (`{==…==}`) break adjacency; proofing marks, other
bookmarks and comment markers, and runs that print nothing (empty,
hidden, a lone soft hyphen or `w:sym`, a page break) do not. A comment
range on one side only, ending inside the insertion as Word writes it,
keeps the pair one substitution both ways:
`{~~old~>{==new==}~~}{>>c0 …<<}`. Changes inside links, content controls and
simple fields count, and headers and footers pair the same way
(`{~~old~>new~~}{>>header1.r91+90 @AL<<}`). The text inside the delimiters
is the document's text exactly.

Converting back, a substitution writes its insertion, then its deletion.
Its one tag lists the deleted side's ids, then the inserted side's: where
the numbers drop (`r94+95+92+93`) the inserted side begins, else the
deleted side takes one id per run it has. A side naming more ids than it
has runs splits its text, after its last space or before its last
character, so every id comes back on its own side.

`@??` is the handle of a mark with no author's name (the attribute missing
or empty); it is a tag, not a comment, and converts back to an empty
`w:author`, which Word opens cleanly (checked 2026-10-14). Its `authors:`
line, `??: ""`, keeps the date its note gives.

A comment is `{==anchored text==}{>>c5 @AC: text<<}`; a reply names its
thread root, `{>>c6 @AS re c5: Disagree.<<}`; a resolved root reads
`{>>c5 @AC resolved: text<<}`.

A handle is the comment `w:initials` that author wrote, else the initials of
the name's words, letters and digits only; a collision appends a digit. The
header's `authors:` line gives each handle's name, counts and its
timestamp, or the day range when the author has several
(`2026-03-02..2026-03-04`). Converting back, the author's undated marks
and comments take the first day at `00:00:00Z`, and one of them, when
another keeps the first day, takes the last day at `23:59:59Z`: an undated
formatting mark, else the last undated comment, else the last change's
marks (each cell of an inserted row the view tags), else the author's
header and footer marks. One mark keeps only the first day. A date the
view writes out, as on a comment, is never rewritten. A bare day
(`# 1 comment, 2026-10-09`) reads as its midnight; a range that runs
backwards is read as no date, with a warning ("its days run backwards").

`document_owner:` is the `dc:creator` that first names someone (its
spaces kept, the XML's indentation not), else `cp:lastModifiedBy`; `none`
names nobody. Converting back writes it to `docProps/core.xml`, unless
that part is cut short, which is left as it is. An empty or null one
(`document_owner:`, `~`, `null`) is ignored with a warning that it names
no one. A comment after `authors:` (`authors: # legend`) keeps the
block.

### Reading back

`convert` reads an agent view back into Word. A change followed by a
tagged note (`{>>r12 @AC<<}`) keeps its id, `r12`
(`footnotes.r12` under a footnote id line), the name the `authors:`
line gives its handle, and its date: the note's own timestamp, else the
handle's from `authors:`. A note that names another change after ` on `
(`{--x--}{>>r3 @BO on r1 @AL<<}`, the plain view the same by author and
date) puts the change it follows back inside that one: `w:ins r1`
wrapping `w:del r3`, each with its own id, author and date, resolved
through `authors:` like any tag. A tagged comment note keeps its id,
author and
date; `re cN` names its thread root; `resolved` resolves the thread. A
change or note with no tag is new and takes the author and date
[Markdown to Word](#markdown-to-word) lists (`-a/--author` and
`-d/--datetime`, the front matter's, the `--reference-doc`'s creator and
creation time, else `Redline` and now). `<!-- pN -->` lines are alignment
keys and are never text.

The header is honoured for `page:`, the `styles:` lines it names,
`authors:` with its `document_owner:`, and `headers:`/`footers:` text,
alignment and `{PAGE}`/`{NUMPAGES}`/`{DATE}` fields. Each story becomes the
part its id names (`header2.p1` is paragraph 1 of `header2.xml`), `first:`
turns on the title page unless it is `inactive`, an active `even:` turns on
even and odd headers, and a `sections:` entry gives its section the page
and stories it names. With no header at all: Letter portrait, one-inch
margins, Normal Calibri 11pt and the writer's other defaults.

A header's or footer's field with no result is its bare name, `{PAGE}`,
where the body writes `{PAGE}{/}`: Word computes the page number on each
page, so a story's view keeps no result for it. Inside another field's
result a field always ends with `{/}` (`{PAGE}{NUMPAGES}{/}{/}`), and
story text that would read as a field (`{SELLER}`, `{1999}`, `{/}`) is
escaped (`\{SELLER}`).

Not read back yet: `source:` names no base document, an untagged note
after a tagged one on the same anchor is a new comment, not a reply, and
the applier (`jubarte compare contract.docx view.md`, `apply_markdown`)
reads CriticMarkup as text. The contract that governs the missing mode is
§7, "Anchored Markdown edits", of
[superpowers/specs/2026-10-17-short-and-long-id-contract.md](superpowers/specs/2026-10-17-short-and-long-id-contract.md)
(moved there from the 13 October settlement's §5.3): when `source:` and
`sha256:` name a file beside the Markdown and the digest matches,
`convert` anchors on it (`--standalone` opts out, a mismatch is
`STALE_SOURCE`), an id line resolves through the pinned snapshot's
inventory, a block without one is new, a source block the Markdown drops
is deleted, and everything the view does not state comes from `source:`
byte for byte — a tagged change is kept when unchanged, accepted or
rejected when the agent removed its text. It is staged in
docs/superpowers/plans/2026-10-16-id-contract-implementation.md, not
implemented.

### Literal marks and anchors

Text that reads like a field code is escaped with `\{` (`\{PAGE}`), in the
body and in headers and footers, and text that reads like a mark has its
delimiter broken (`{\++`); converting back restores both.

A code fence has its own rule, because Markdown reads no backslash escape
there while CriticMarkup still marks tracked changes in code: the delimiter
itself carries the escape. A delimiter shape with a run of backslashes at
its break point (`{` + the run + `++`, `++` + the run + `}`, `~` + the run
+ `>`) is written with the run made odd — literal `{++` prints as `{\++`,
literal `{\++` as `{\\\++` — and reading halves an odd run back
(`{\++` in a fence is the text `{++`). Every other backslash in a fence is
text as written, so hand-written `C:\\path`, `\\d+` and LaTeX `\\` reach
Word byte-identical; a delimiter behind an even run (`{\\++`) is text as
written too and opens nothing, and a bare delimiter is the change itself,
never escaped by what precedes it. CommonMark's indented code blocks (four
spaces or more) hold their text verbatim the same way: nothing opens there
and no fence sits inside one.

Document text that CommonMark would read as markup is escaped in every
view, the agent view and `convert --commonmark` alike: a paragraph that
starts with `# `, `- `, `> ` or `1. ` (also after a line break), and `*`,
`` ` ``, `\`, `~~`, `==`, `[^`, a tag-like `<` and a word-edge `_`, `~` or
`^` anywhere (`\# Not a heading`, `\*\*stars\*\*`). The escapes read back
as the characters.

An `edit` anchor copied out of the view may carry marks that are not
document text (`# Fees`, `**secret**`, a CriticMarkup note). The literal
anchor is tried first; when it does not occur, the anchor without its marks
(escapes resolved) is tried, and the report's `op` line records
`anchor_given` and `anchor_read_as`. The CLI prints
`note: op-1: anchor "# Fees" read as "Fees" (Markdown marks are not document
text)`. `read --changed` keeps the blocks with a change or a comment, every
paragraph of a comment range included; `--by AUTHOR` (a handle or a full
name) keeps one author's. A selection keeps the footnotes its blocks cite;
`--changed` also keeps a changed footnote and names it in `range:`
(`changed (p4, [^2])`).

### Commands

```text
jubarte read FILE                      the view; jubarte FILE is the same
  -p p5,p12-p20,t0   --head N   --tail N   --changed [--by AC]
  --track-changes accept|reject   --comments none   --dates   --no-page-markers
  --json (the snapshot)   --tables (the grids)   -o FILE.md|FILE.json
jubarte A B                            accept both sides' changes, compare, print the redline's view
                                       (read options apply; -o FILE writes the file instead)
jubarte compare A B                    the same; -o FILE writes the redline (.docx, .pdf, .png, .md)
jubarte diff A B                       the paragraph patch (text only; -o FILE.patch keeps it)

jubarte edit FILE -p WHERE --anchor FIND --content WITH        replace FIND (an insertion when WITH keeps FIND at its start or end)
jubarte edit FILE -p WHERE --content TEXT                      rewrite the paragraph
jubarte edit FILE -p WHERE [--anchor FIND] --delete            delete FIND, or the paragraph
jubarte edit FILE -p WHERE [--anchor FIND] --style SPEC        format FIND, or set the paragraph style
jubarte edit FILE -p c5 --content TEXT | --delete | --resolve  a comment
jubarte edit FILE -p c5 --reply TEXT                           a reply
jubarte edit FILE -p c5.text --content TEXT                    replace the text c5 covers
jubarte edit FILE -p r12 --accept | --reject                   resolve one tracked change (prints `accepted r12 in p3: "from" > "to"`)
jubarte add  FILE -p WHERE --content TEXT [--before]           a new paragraph after (or before) WHERE
jubarte add  FILE -p WHERE --anchor FIND --content TEXT        a comment on FIND (--comment: on the whole paragraph)
jubarte add  FILE -p c5 --content TEXT                         a reply (also --reply TEXT)
```

The printed view is the output of `jubarte FILE` and `jubarte A B`, so `-q`
does not hide it (it silences the `wrote …` line of `-o`). Options go after
the task: `jubarte --head 2 read a.docx` is refused, since the parser would
take `read` for a document.

`WHERE` is an id the view prints (`p12`, `header1`, `footer2.p1`,
`footnotes.p0`, `t0.row1.cell2`, `c5`, `c5.text`, `r12`); `edit` plans take
the same ids. No id needs quoting in bash, zsh or csh. One command carries
several operations: every `-p` starts one, and the flags after it belong to
it. `--plan PLAN.json` stays for batches and the other operation kinds, and
excludes the operation flags.

The command writes one file, `<stem>_modified.docx` beside FILE (`-o
FILE.docx` names it; an existing one needs `--force`): the redline in
suggesting mode, the edited document in editing mode. Editing
`X_modified.docx` again writes `X_modified.docx`. `--out-dir [DIR]` writes
the bundle instead, in `<stem>.edit` by default.

The options apply to every operation of the command: `--author NAME`
(default `Redline`), `--datetime ISO8601` (default now, UTC),
`--suggesting-mode` (the default: tracked changes; the bundle holds
`redline.docx`, `clean.docx`, `patch.diff` and `report.jsonl`) or
`--editing-mode` (the edits land directly; the bundle holds `clean.docx`
and `report.jsonl` only), `--existing-revisions auto|keep|accept|reject|refuse`
(default `auto`: `keep` when the file has tracked changes; `--editing-mode`
refuses `keep`, so pass `accept` or `reject` for such a file). `--plan`
without a value reads or records `<stem>_plan.json` beside FILE.
`--style` takes `bold`, `italic`, `underline`, `strike`, `caps`,
`highlight=yellow`, `font=Calibri`, `size=11` and `color=FF0000`; any other
value is a paragraph style (`Heading2`). After applying, the command prints
its notes, then the changed blocks: `read redline.docx --changed --by
<author>`, or every changed block when the author has none (as after
resolving another author's comment). A pure `--accept`/`--reject` (no
content operation) prints one line per resolved change instead
(`accepted r1+2 in p3: "{~~weekly~>monthly~~}" > "monthly"`).
Anchors match literally first, then without their Markdown marks, with a
note.

### Limits of the agent view

- Tracked paragraph marks inside comment bodies are not shown.
- Headers and footers are header lines (`headers:`, `footers:`), not body
  blocks: when one holds a change `--changed` selects, the `range:` line
  says to read those lines (with no block left when only they changed).
  A story whose paragraphs carry their own text colour (`w:color`, when
  every text run sets the same one) or borders (`w:pBdr`, in the table
  border grammar), or differ in alignment, prints paragraph by paragraph:
  `id: header1`, then `p0: {text: …, center, color: "595959", border:
  bottom single 0.5pt 8496B0 space 4pt}` for each, as a block in the first
  section and as a flow map in `sections:`. A colour that opens with a
  digit is quoted, since YAML reads `000000` as the number 0. Alignment is
  `center`, `right`, or `left`/`justify` when the paragraph states it
  (`w:jc`), so a centred Header style does not centre it; no word means the
  paragraph names none.
- Page markers need the layout pass; `--no-page-markers` skips it. When
  the layout fails, the markers come from Word's cached page breaks, else
  from hard breaks and section starts. That fallback does not mix the two
  (a page break added after Word last saved turns no page) and does not
  add the blank page an odd- or even-page section start can need.
- A table is one block for `-p`, `--head` and `--tail`.
- With comments inline, a comment range between two changes by one author
  splits their note (`{>>r7 @AC<<}` … `{>>r8 @AC<<}`): CriticMarkup cannot
  nest a comment's scope inside a change. The resolved views say `rev r7+8 @AC`;
  both ids name the same revisions.

## Diffs

`jubarte diff OLD NEW` prints the changes as a patch, as `git diff
--word-diff` does, for any two documents, Word or Markdown:

```text
$ jubarte diff old.md new.md -a "Arthur Rodrigues"
--- a/old.md
+++ b/new.md	Arthur Rodrigues	2026-09-30T14:05:00Z
@@ [line:3] @@
Payment is due in [-30-]{+45+} days.

@@ [line:5] @@
- Delivery{+
- Returns+}
```

- Only the changed paragraphs are shown, each whole. `@@ [line:N] @@` is
  the line a Markdown paragraph starts on; in a Word document it is the
  paragraph's id, `@@ [p12] @@` (or `header1`, `header1.p2`,
  `footnotes.p3`, ...), the id edit plans take (`jubarte read` prints it as
  `<!-- p12 -->`). It is the
  paragraph in the new version; `@@ -[p12] @@` is a paragraph
  removed, in the old one. A change in a text box is shown at the
  paragraph that holds the box.
- A change is `[-old-]{+new+}`. Comments and their scopes stay CriticMarkup:
  `{==text==}{>>Name (date): comment<<}`.
- The `+++` line names who made the changes and when, once. A change by
  anyone else (a tracked change already in a Word document) is followed
  by `{>>Name (date)<<}`, and every comment carries its author and date.
  A substitution whose two sides two people made is two changes, each
  named after itself: `[-old-]{>>Ana (date)<<}{+new+}{>>Bo (date)<<}`.
  `-a/--author` defaults to Redline; `-d/--datetime` to now.
- Changes grow to whole words, numbers, formatting spans and links, so
  `[-**twelve**-]{+**eighteen**+}` rather than a change inside the bold.
  Positions and counts are in characters: nothing is tokenized.
- Lines wrap at 72 columns, never inside a change's delimiters, code, a
  link or an autolink; `--columns N` changes it and `--columns 0` does not
  wrap. Nothing is printed when nothing changed.

`diff` writes text only. `-o changes.patch` (or `.md`, `.txt`, `.diff`)
writes what it would print, says so on stderr and prints nothing; an
existing file needs `--force`. Without `-o` nothing is written. The Word
redline, its PDF and its pages are `compare`'s: `jubarte compare old new
-o changes.docx` (or `.pdf`, `.png`); `diff -o changes.docx` exits 2
saying so.

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

- Delivery{++
- Returns++}
- Warranty
```

Lines are aligned first, then words within the lines that pair up; changes
separated only by spaces merge, so a rewritten phrase is one change. On
lines both versions share, list, heading, quote and footnote markers stay
outside the change. A marker that itself changed — a renumbered item, `#`
becoming `##`, a task box ticked — takes the whole block with it into one
substitution, and a block added or removed whole is one change over its
text and line breaks, so accepting and rejecting rebuild both documents
byte for byte. Word writes such a change as structure, never as text: a
deleted heading is a deleted Heading paragraph, a ticked task box changes
the box itself. A table row added or removed is marked cell by cell, since
a table drops text outside its cells. An ordered list's numbers do not
steer the line pairing: Word numbers items itself, and an item renumbered
by an insertion above is changed whole. CriticMarkup delimiters already
in either document are escaped, so they stay text. The patch is built from
this alignment.

For a Word redline (`jubarte compare old.md new.md -o r.docx`) the two
Markdown documents are written to Word and compared by the same engine as
two `.docx` files, so moves and Word's grouping of changes apply.
`--reference-doc` styles both.

### Git

```sh
git config --global difftool.jubarte.cmd 'jubarte diff "$LOCAL" "$REMOTE"'
git difftool -t jubarte -y -- '*.md'
```

## Word against Markdown

`jubarte compare contract.docx edited.md -o redline.docx` is the redline
of an edit made in Markdown, on
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

Reversed (`jubarte compare edited.md contract.docx`), the Markdown is applied
the same way and the redline runs from it to the Word document.

Not applied, and reported as warnings: footnote text (only the body is
edited) and table rows or columns added. Emphasis changed in the Markdown
is not applied to an existing paragraph; a new paragraph takes the
Markdown's bold and italic. A document with tracked changes is read with
them accepted.

## Limits

- `jubarte diff --format critic` needs two Markdown documents; with a
  Word side it exits 1, and `jubarte compare OLD NEW` prints the redline
  as the agent view, CriticMarkup included.
- Definition lists are not written. A raw HTML block (`<div>` up to a
  blank line) keeps only its text, a paragraph per block element, with a
  warning.
- The PDF renderer sets an equation's text on one line: fractions,
  radicals, matrices and limits are not laid out yet.
- Changes have one author and date; CriticMarkup has no syntax for more.

Related: [CSHARP_MARKDOWN_PROJECTION_MAPPING.md](CSHARP_MARKDOWN_PROJECTION_MAPPING.md)
maps Docxodus's C# Markdown projection onto this crate — a study with open
questions, not a shipped feature.
