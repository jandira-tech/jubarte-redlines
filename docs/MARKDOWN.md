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
followed by its number, author and date (`{++new++}{r5 Ana Lima
(2026-09-29T14:05:00Z)}`; the brace tag attributes the change; `{>>…<<}` denotes a comment). Fields print as their text, and
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
sign and date their changes with `--author` (default `Jubarte`) and
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
| table | Word's thin grid (`TableGrid`, single 0.5pt borders all round, as Insert Table makes it) unless the view names another style: the legend's `table:` for every table, a table line's `style X` for its own. `TableNormal` is Word's Normal Table, written with no `tblStyle`; the view names it for a table without a style. The borders a table carries itself come back as they were: `borders single 0.5pt 000000` (`w:tblBorders`; sides that share a border group as `top,bottom single 1.5pt`, groups joined by ` + `, a side turned off is `none`, a group naming no sides covers all six) `row-borders top double 0.5pt FF0000 in t0.row1 t0.row3` (a row's own `w:tblPrEx/w:tblBorders`; groups joined by `; `) and `cell-borders top,left,bottom,right single 1pt CCCCCC in all` (`w:tcBorders`, for `all` cells, a row `t0.row1` or a cell `t0.row1.cell2`; groups joined by `; `). A border in a theme colour adds it after its fallback colour: `single 1pt 2F5496 theme accent1 tint 99 shade BF` (`w:themeColor`, `w:themeTint`, `w:themeShade`). Word's border effects follow it: `single 1pt 2F5496 shadow frame` (`w:shadow`, `w:frame`). A source that defines `TableGrid` with one single border other than 0.5pt keeps it: the legend says `table: TableGrid, all borders 1.5pt`; one whose borders vary names each side (`table: TableGrid, borders top double 3pt bottom double 3pt`). Header row repeated, column alignment kept |
| `[text](url)`, `[text](#anchor)` | hyperlink; `#the-intro` lands on the heading whose GitHub slug it is, through a bookmark (`_` before a digit, cut to 40 characters); a wholly deleted heading, an empty one or one in a footnote has none, and a link to a heading that is not there is written with a warning. Reading Word back, reader and writer share github-slugger's names (a repeat `-1`, `-2`): only the exact name the writer gives a heading is its own bookmark and prints no anchor; any other (`title-01`, a lone `title-1`) stays an `<a id>` anchor, and a heading in a table cell claims no slug. A title (`[t](u "Title")`) is the hyperlink's tooltip, and `<a href="u">t</a>` is a link too |
| `[^note]` | footnote; a label defined twice (a footnote and an endnote both `[^1]`) is two notes: its first reference names the first definition and its next the second, which takes the next free number, with a warning |
| `![alt](file.png)` | picture (PNG, JPEG, GIF, BMP, TIFF), at most the text width; else its alt text |
| `![alt][dN]` with `[dN]: drawing:PAYLOAD` | a drawing the readable forms cannot say — a shape, text box, connector, VML picture or `mc:AlternateContent`. The reference's name is the drawing's number in reading order; the definition sits with the others after a `<!-- drawings -->` marker at the end of the view, so the body text stays free of payloads. `PAYLOAD` is URL-safe base64 of a JSON envelope the reader writes: the element's XML, its relationships, and every part they reach. The writer splices it back verbatim (fresh relationship and `docPr` ids, each part re-written); a payload that does not decode is dropped with a warning. A content control inside it bound to a custom XML store (a cover page's Abstract) gets its store back when the main part reaches none: the writer builds it from the bindings it writes, each simple path of prefixed names (`/ns0:CoverPageProperties[1]/ns0:Abstract[1]`) holding what a plain text, date or list control shows, since Word fills a bound control from its store. Any other binding (rich text, a picture, a check box, another path, a field or symbol in the text) is dropped, so Word keeps what the control shows, and `jubarte validate` reports a binding whose store the main part does not reach as `DATA_BINDING_WITHOUT_STORE` (a warning: Word opens the file and shows the control empty). The older inline form `![alt](drawing:PAYLOAD)` reads the same |
| `![alt][imageN]` with `[imageN]: <data:mime;base64,…>` and `<!-- imageN drawing:PAYLOAD -->` | a raster picture (PNG, JPEG, GIF, BMP, TIFF), inline or floating. The `data:` definition holds the image's bytes; the backing comment the payload of everything else (anchor, wrap, `docPr`). Written back, the bytes fill the payload's image part; an edited `data:` writes the new image. Two identical payloads share one name and one definition. Without its backing comment the `data:` picture is a plain inline image |
| ` ```svg dN ` fence with `<!-- dN drawing:PAYLOAD -->` | an `asvg:svgBlip` the paragraph holds alone, as its source code: the fence's text is the SVG part, the backing comment the payload with the PNG fallback inside. An edited source writes the new SVG into the part and keeps the fallback |
| ` ```mermaid dN ` fence with `<!-- dN drawing:PAYLOAD -->` | a chart the paragraph holds alone, as its data: a `pie` fence for a pie chart, `xychart-beta` for bar and line. The backing comment holds the payload and a hash of the printed text: a fence that comes back unchanged writes the chart part byte for byte; an edited one rewrites the part's categories, values and title in place — styling, axes and everything else the Mermaid cannot say stay the part's own. A chart that shares its paragraph with text keeps `![alt][dN]` instead |
| `---` | a paragraph with a bottom border |
| `<br>`, `<sup>`, `<sub>`, `<u>` | line break, superscript, subscript, underline (in a cell or a note of the tracked view, `<br>` ends a paragraph and `<br/>` is a line break; see below) |
| `<b>`/`<strong>`, `<i>`/`<em>`, `<s>`/`<del>`, `<code>`, `<mark class="hl-<colour>">` | bold, italic, strike, `VerbatimChar`, highlight (yellow by default; the colour Word holds through `class="hl-<colour>"`, the names the table above lists); attributes and case do not matter, a tag left open closes with its block, and any other tag keeps its text and is named once in a warning |
| `x ^2^`, `x ~2~` | superscript, subscript, at word edges only (pulldown-cmark opens neither inside a word: `H~2~O` is text) |
| `$x^2$`, `$$\frac{a}{b}$$` | an equation (`m:oMath`; display math in an `m:oMathPara`), see below |

A body text box's words are editable Markdown paragraphs after its anchor,
between `<!-- text-box dN.0 -->` and `<!-- /text-box -->`. The suffix
numbers boxes in that drawing from zero; these markers do not contain
any document words or consume paragraph ids. Keep the markers when
editing, including when adding or removing paragraphs. The drawing's
reference definition holds its frame and a text-free template. Matching
Markdown and word hashes restore the template's original structure (apart
from package relationship and drawing ids); edited paragraphs rebuild the
box content inside the same frame. The payload cannot override edited words.
A box whose words carry no change of their own prints them with its
anchor's tag (`{++box ++}{r7 @B}` inside an inserted drawing). That copy of
a single id is the anchor's mark, which already holds the box, so the
writer writes the words bare, with no id of their own and no warning.

A text box nested in another box has an ordinary `![alt][dN]` token at its
position inside the enclosing box's region. Its own regions follow the
closing marker of the enclosing region, in depth-first order. Regions
never nest; their existing `dN.k` names associate them with their tokens.
Both boxes' words are editable. Keep each inner token: conversion refuses
a missing or repeated nested token and names the affected box. An unchanged
round trip preserves the nested drawing's properties and nesting.

Word cannot open a text box nested in another box's readable content;
`jubarte validate` reports `NESTED_TEXT_BOX` (Word-fatal, no automatic repair).
A nest only in an ignored `mc:Fallback` does not trigger this check.

When the Choice and Fallback copies of a nested drawing do not correspond,
the enclosing drawing stays opaque. Its nested words remain readable, but
conversion refuses edits to that box and explains the Choice/Fallback
mismatch. Region names `dN.k` identify box stories; they are separate from
payload-internal `dP.n` relationship symbols.

Conversion refuses a missing, renamed, annotated, duplicated or overlapping
marker region when the frame needs its words. The error names the box.
Keep an empty marker region to remove all its words. Markers in code or
escaped as text are ordinary words. Older payloads that still hold the box's
words, including header and footer boxes, need no marker region.

Images are read relative to the Markdown file, or to `--resource-path`;
URLs are not fetched. After `%XX` escapes are decoded, a name that is a
URL scheme is refused, and the path a local name resolves to must stay
inside that directory once symlinks are resolved: an absolute path, a
`..` walk or a link out of it is refused too. A refused image is
written as its alt text, with a warning that says why, and nothing
outside the directory is read. A document that names an image in
another folder needs `--resource-path` at or above it.

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
`--keep-nested-comment` preserves comments and replies whose whole range
settlement removes, on an empty untracked anchor at the removed text.
`accept` and `reject` offer the same opt-in. Partial overlaps keep their
existing behavior. Without `--track-changes accept` or `reject`, convert
refuses the keep flag with exit 2.
`--no-critic` reads the delimiters as text. A delimiter behind a backslash
(`\{++`) or without its partner is text too, and says so in a warning.

Resolving the Markdown splices the kept side of every mark back into the
source and lets Markdown re-read it, which CommonMark's delimiter rules
do not always give back the same document: a kept `_b_` inside `*a…c*`
reads as literal underscores and `*a{++*b*++}c*` reads as a bare `b`,
while the same input written to Word and accepted there reads as one
italic `abc`. Every block holding a mark is therefore verified against
the md→docx reader's own runs. A block that already reads like them is
left byte-for-byte as it is; one that does not has the smallest inline
span holding the mark re-emitted from those runs — bold+italic `a`/`b`/`c`
as `**a*b*c**`, in the flavour the document already used — with the
block's own syntax (a heading marker, a list bullet, a table cell) and
its neighbours kept. The text a Markdown reader cannot model is never
re-emitted: a comment or a highlight inside a change, a change named as
nested inside another (`{~~old~>new~~}{r4+5 Bob re r3 Alice}`), an
anchor, a malformed delimiter and a mark holding a line break all keep
the resolved splice.

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
Consultant will deliver the services in each Statement of Work, including {++quarterly++}{r0 @AC} route-cost reports and a {~~weekly~>monthly~~}{r2+1 @AC} review call.
```

`revisions:` counts the body's marks, then the changes `jubarte changes`
lists for the body, kind by kind (#1356). That includes a change printed
only as another's parent (`re r7 @B`), and a row whose cells all print
its id counts once, as a table row. A selection counts the changes whose
ids it prints. Each other story then counts the changes `jubarte changes`
lists for it, with its marks when they differ: `3 in header1 (7 marks)`.

`--track-changes accept|reject` prints the document with every change
resolved, `--comments none` leaves the comments out of the text (their ids
move to the id lines), `--dates` puts timestamps in notes whose author's
recorded date forms differ, including offsets for the same instant.
`-p p3,p10-p20,t0`, `--head N` and `--tail N` print a selection, and
`--no-page-markers` skips the layout pass.

`--json` prints the snapshot and `--tables` the grids instead of the
view. Both read the accepted text; `--track-changes reject` reads the
text and formatting Word's Reject All leaves (`accepted` and `rejected`
spell the two choices too), every paragraph, row and cell keeping its id,
and the snapshot names an `accept` or `reject` in a top-level
`track_changes` (its `summary.track_changes` still says whether the
document tracks changes; `summary`, `changes` and `comments` describe the
file as stored). `--comments`, `--dates` and `--no-page-markers` shape
only the view, so `--json`, `--for-edit` and the grids of `--tables` take
them and change nothing.

`--tables -p` reads the selection as the view does: `t1` is the view's
`t1`, and a `pN` or a range inside a table reaches that table. It prints
the grids of the tables the selection reaches, each with the tables
nested in it, once and in document order, then an empty line and what
`read -p` prints for the same selection, unchanged. The grids' `table N`
counts nested tables too, so it can differ from `t1`. A pick that reaches
no table is a usage error (`p0 is not in a table; --tables -p takes tN,
or a pN inside a table`); what the view refuses (`t9 is not a table of
this document`) fails as `read -p` does.

With `--json` on the command line, an error also prints on stdout as one
line, `{"error":{"code":"USAGE","message":"unexpected argument '--bogus'
found","usage":"jubarte read [OPTIONS] <FILE>"}}`, so `| jq .error.code`
reads it from stdout alone; stderr and the exit code stay as they are.
stderr is text, the error alone, so pipe stdout alone, never `2>&1 |
jq`. Its `usage` is the task's, a line per form; the `Usage:` line on stderr is
clap's, which names the flags given. The code is the message's leading
code (`STALE_SOURCE`), else `USAGE` for an exit 2 and `ERROR` for any
other.

A JSON read names the input's own validity findings in its output, not
on stderr (#1309), `-q` or not: `read --json` in the snapshot's
`summary.input_validity`, `{"errors":0,"warnings":2,"next":"jubarte
validate FILE"}` (a clean input reads `{"errors":0,"warnings":0}`),
`read --for-edit` in its result's `input_validity` (#1340), and `changes
--json` and `comments --json` on their last line,
`{"input_validity":{…}}`, always there, after the records. `read -o
FILE.json` writes what `read --json` prints, and `read --for-edit -o
FILE.json` what `--for-edit` prints; with stdout free, the Markdown view
and `-o` keep the one stderr line (`input has 2 validity warnings;
jubarte validate FILE lists them`, which `-q` drops).

### Id lines

One per body paragraph, on the line above it: a head, one space, then
clauses joined by `, ` (no clauses: `<!-- p2 -->`).

```text
head:     p{N}[ {Style}][ outline {n}]
clauses:  {align} · first-line {x}in · hanging {x}in · left {x}in · right {x}in · before {x}pt | before auto · after {x}pt | after auto · line {r} | line {x}pt exact | line {x}pt atLeast · size {x}pt · num "{label}" | bullet · level {n} · page-break | page-break-ins {tag} | page-break-del {tag} · section-break · break-del {tags} · fmt {tags} · rev {tags} · comments c{ids} · in c{ids}
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
- `before`, `after` and `line` come from the paragraph's own `w:spacing`
  (its style's spacing is the legend's): `before 6pt`, `after 0pt` (a
  zero the paragraph states still prints: it overrides its style), in
  points with up to two decimals (a twip is a twentieth of a point; a
  Strict file's `12.945pt` reads as the nearest twip, as Word keeps it).
  Values past Word's 1584pt print and read back as the file has them.
  `before auto` and `after auto` are Word's automatic space
  (`w:beforeAutospacing`, which HTML paste writes): Word puts it in place
  of the value, so the view prints no value beside it.
  `line 1.15` is an `auto` line, a multiple of single spacing with the
  fewest decimals (two to four) that read back the same 240ths (`line
  1.038` is 249); `line 12pt exact` and `line 14pt atLeast` are the other
  two rules. An empty paragraph with its own spacing keeps its own id
  line: a range of empty ones would drop it. Body and table-cell
  paragraphs keep their spacing this way; a note's, a comment's or a text
  box's paragraph keeps only its style's for now, and spacing in lines
  (`w:beforeLines`, `w:afterLines`, for East Asian layout) is not kept.
- `size 9pt` is the paragraph's own text size: the one `w:sz` every run of
  its text states (`size 10.5pt`, in half points as Word keeps it), or,
  for a paragraph with no text, its mark's, when its style (through
  `basedOn` and the document defaults, Word's 10pt without any) gives
  another. Runs of different sizes, or a run that states none, print
  nothing, and neither does the mark of a paragraph with text. The writer
  puts the size on every run of the paragraph and on its mark (`w:sz` and
  `w:szCs`). An empty paragraph with its own size keeps its own id line:
  it is a line of that height. Body and table-cell paragraphs keep their
  size this way (a cell paragraph's on the table line, beside its
  spacing: `p8-p38 line 8.65pt exact size 9pt`); a note's, a comment's
  or a text box's paragraph keeps only its style's for now.
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
- `¶{rN @X}`: an inserted paragraph mark; `break-del`: a deleted paragraph mark. `fmt`: a formatting
  change (`w:rPrChange`, `w:pPrChange`).
- `rev`: in the accept and reject views, every revision the paragraph
  held before resolution, formatting changes and paragraph marks included.
  A paragraph joined into the one before it hands its revisions over. In
  the tracked view, the insertions, deletions and moves its text has no
  tag for: a deleted picture, line break, field character or empty run
  (a tracked page break has its own clause); `rev-del` names the
  deletions among them. A replacement with no text on either side (a
  field deleted and another put in its place) names each side by its own
  marks, `rev r4 @D, rev-del r3 @D`: either side is one `accept --id`
  takes.
- `comments`: with `--comments none` only, the comments whose ranges or
  references the paragraph holds.
- `in c5`: with comments inline, the comment threads whose range runs on
  past the paragraph. CriticMarkup cannot span paragraphs, so such a range
  repeats `{==…==}` in each and shows its note only where it ends; the
  clause says which thread a scope belongs to.
- Empty paragraphs: `<!-- p19 empty -->`, consecutive ones
  `<!-- p19-p23 empty -->`. An empty paragraph that holds a tracked mark, a
  formatting change, a resolved revision, a hidden comment or a stated
  style keeps its own line: `<!-- p4 -->` followed by `¶{r3 @AC}`,
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
`t0` places the table itself on the page), then `p8 Quote` for each cell
paragraph that states a style, then `p8-p10 after 0pt line 1 size 9pt`
for cell paragraphs with their own spacing or text size (the id line's
clauses joined by spaces, one range for neighbours that say the same), then the table's
geometry: `widths 1.5in 2in 3in` for the grid's columns (`w:gridCol`),
printed when they are not an even split of the text width (the body
section's page width less its margins); the reader writes the grid from
it, each cell's `w:tcW` from the columns it spans, and a row's gap
columns as `w:wBefore`/`w:wAfter`, while widths naming a column count
the grid does not have are dropped with a warning and the columns split
the text width evenly. `width auto`, `width 6in` or `width 80%` is the
table's preferred width (`w:tblW`); without one the table fills the text
width (`5000` fiftieths of a percent), so a table Word sizes to its
contents, or whose `w:tblW` gives no width or 0, says `width auto`; the
reader takes up to 22in or 600%, and the view prints no width past
either. `indent 0.5in` is its indent from the margin (`w:tblInd`), and
`layout fixed` keeps its column widths as set instead of fitting them to
the text (`w:tblLayout`). Only a numbered table has a table line, so the
geometry is a body table's: a table nested in a cell, or in a note,
header or footer, prints none and its widths are not kept. After that come `header row
repeats`, the table's `style` and its `borders`, then `merged t0.row0.cell1 across 2; t0.row1.cell0
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
spacing, alignment; the writer writes its spacing as the document's
default, `w:pPrDefault`, so a table style's own spacing still wins in
its cells), each heading level's style with what differs from
it, then every other paragraph style the body uses, a text box's
paragraphs included, keyed by its id
(`TOC2: Times New Roman 12pt, after 6pt`), described the same way. A line
names only what its style sets apart from the style it is based on:
font and size, `before`/`after` (`after 0pt` too, when the base has
some), `line` (`line 1` for single, when the base is not), alignment, `keep-next`,
`keep-lines`, `indent Xin` with `first-line`/`hanging Xin`, `right Xin`
for the right indent (the id line's word; `right` alone is the alignment),
and `contextual spacing` (no space between paragraphs of that style, as
List Paragraph has). A line ends `, named Text Heading 2` when the style's name
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
first-use order (`tables: TableNormal, Ledger, borders bottom double
0.75pt`), each entry saying what the singular `table:` line says (the
borders clause, the name). A style of the document's own whose borders
are not one single line all round names each side it sets, in Word's
order (top, left, bottom, right, insideH, insideV): the side, its
`w:val`, then its size in points, `#RRGGBB` for a colour other than
`auto` and `space Npt` for a spacing other than zero (`borders top
single 1.5pt #FF0000 space 2pt insideH nil`, the legend value quoted
for the `#`). Converting back writes exactly those sides. One of Word's
own styles says `borders vary` instead, since the writer writes it as
Word defines it; an older view's `borders vary` on another style still
reads, and the writer warns that it writes that style without borders. Converting the
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
is quoted). A silent line uses Word's own `FootnoteText` or `EndnoteText`
style. Its name matches `footnote text` or `endnote text` ignoring ASCII
case, under any source style id. Spaces remain significant: `Footnote Text`
matches, while `FootnoteText`, `Footnote  Text`, and names with leading or
trailing spaces do not. The legend keeps a case-only spelling difference
(`FootnoteText: …, named Footnote Text`), so the writer preserves the exact
name even with a silent id line. A localized source id with that name also
rebuilds as the canonical note style, with its spelling kept in the legend.
A real rename (`Notes`, LibreOffice's `Endnote`) keeps its explicit style
token. A table cell does the same: `<br>` ends one of the cell's
paragraphs, `<br/>` is a line break (`w:br`) within one. A view written
before that distinction still converts: when the id or table line's
paragraph count and the `<br>`s disagree, the writer reads every `<br>`
there as a line break (for a note, with a warning). A `<br>` prints inside a change only when its paragraph's
mark is tracked: `this<br>{++added++}` is an inserted paragraph after an
untracked mark, `{++A<br>B++}` an insertion that takes the mark with it. `edit -p footnotes.p0` edits that
note's text. A note Word inserted or deleted whole, its reference with it,
reads `{++[^1]: text++}{r10 @AL}`, and the marks its own text holds
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
A `fmt` clause alone creates the notes part it names. A `rev:` there
(an insertion or deletion in a separator) is not written, and says so. A
note's own changes its text cannot show close its definition line:
`[^1]: Source <!-- fmt footnotes.r7 @AC -->`. A first-page header or footer
prints as `first: {…, inactive}` when different-first-page is off.

A style's two `style_changes` tags name its paragraph-property and
run-property records, in that order. A single tag writes both records
with the same id, as Word does; reading them back prints one tag. Both
forms keep their record count and ids on further round trips. Duplicate
revision ids are checked and repaired within each record type in a part;
the accepted paragraph-property/run-property pairing is preserved. Run
revisions and row/paragraph-mark revisions also have separate id scopes,
even when their XML element names are the same.

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
{++quarterly++}{r0 @AC}
{--ninety days--}{r7 @AC}
{~~weekly~>monthly~~}{r2+1 @AC}       one author, deleted id first
{++an++}{r1+2 @AC}{--his--}{r1+2 @AC}  one author, insertion first
{--old--}{r2 @JD}{++new++}{r1 @AC}    different authors stay separate
{++bold ++}{r7+8 @AC}{++words++}{r7+8 @AC}  one change of two marks
{++new text++}{r5+6 @AC}¶{r5+6 @AC}  a change that ends its paragraph
{++quarterly++}{r0 @AC 2026-10-03T14:05:00Z}  with --dates
{--deleted--}{r3 @BO re r1 @AL}        deletion inside insertion r1
Text¶{r9 @AC}                        inserted paragraph mark
```

A change inside another — `A` inserted text, `B` later deleted some or
all of it — names the outer one after ` re ` and the outer change's own
tag, whether or not text of the outer change is left beside it (#1111),
each with its own timestamp as notes print them: `r3 @BO re
r1 @AL`, or `r3 Ben Ode (2026-09-30T09:00:00Z) re r1 Ana Lima
(2026-09-29T14:05:00Z)` in the plain view. The writer puts the change
back inside the one it names. A row's or cell's own mark is never
named: the writer rebuilds those where they stand, and the table line
that places the mark places the changes inside it (#1376). A formatting
change's `fmt` tag names no parent yet (#1377). Deeper nests chain
the same way, innermost first (`r5 @CD re r3 @BO re r1 @AL`). The body
writer currently rebuilds only the first outer a chain names; headers and
footers rebuild the full chain. Word opens `ins > del > ins` but misreads
reject-all; the chain-writing choice remains open in #1134.

Inside any `w:del` in the same story, at any depth, run text is
`w:delText` and field code is `w:delInstrText`, including inside a nested
`w:ins`. An insertion outside a deletion keeps `w:t` and `w:instrText`.

Revision `date` prefers the valid `w16du:dateUtc` instant, written in the author's local
offset when the document records both times (a whole-minute difference within ±14:00;
otherwise `Z`). Without a valid UTC date it is `w:date` as stored. Comment `date`
follows the same rule, using the `w16cex:dateUtc` instant linked through the comment
extensions and the author's local offset when both times are recorded; without a valid
UTC date it is `w:date` as stored.

A plain note's grammar is exact about where the name ends and the date
begins: the date is the last parenthesised group, so a name may hold
parentheses of its own (`r1 Ana (consultant) (2026-09-29T14:05:00Z)`
names `Ana (consultant)`). In a note that names another change after
` re `, each name spells its own parentheses percent-wise — `(` as `%28`,
`)` as `%29`, `%` itself as `%25` (`r3 Ben Ode (…) re r1 Ana %28consultant%29
(…)`) — so only the dates' parentheses are bare and the parts of the
note split without doubt: the last ` re ` whose left side parses as one
part and whose right side as the chain the rest names wins, so a name
holding ` re r1 Kim` of its own stays one name. A part with a bare
parenthesis in its name makes the note no fold: it reads as the change's
plain attribution, or as a comment.

A tag names the logical change its piece belongs to, the one `jubarte
changes` lists (#1261): the marks Word writes for one edit (one author's
touching insertions, or deletions, in a story, whatever their dates; a
deletion and that author's insertion after it; an insertion and that
author's deletion touching it; the paragraph mark the change ends with)
are one change with a joined id, and every piece prints it: `{++bold
++}{r7+8 @AC}{++words++}{r7+8 @AC}`. A replacement holds one run of
insertions and one of deletions, its ids in reading order (`r1+2` for an
insertion first, `r2+1` for a deletion first). The pieces keep their own
text, formatting and, under `--dates`, their own dates. Original text,
another author's deletion, a field, a move, a table cell or the end of a
paragraph separates two changes. So does original text the view prints
nothing for: hidden text, a soft hyphen, a symbol, a page break. Written
back, those two changes touch, so they read back as one (#1369). Another
author's insertion among the marks does not, unless the marks were made
inside it, or it sits between an insertion and the deletion after it,
which touch only with nothing between them. Empty runs, bookmarks,
comment ranges and Word's rendered-page-break hints separate nothing. When writing DOCX,
the pieces that print one joined id take its ids in reading order, one
each, so reading the DOCX back prints the same view. One id may appear on
several marks. When writing DOCX, adjacent pieces with the same single id,
kind, author and date become one revision element, including formatting
boundaries and changes nested inside that revision. Pieces separated by live text or a paragraph
boundary take distinct OOXML ids. The writer warns with the source line
and the replacement id; reading the DOCX back prints that new id. Reusing
an id with a different kind, author or date also requires a new id.
A substitution is one change: only a deletion directly followed by a live
insertion of the same change, as `changes` lists it, pairs as one. Two
changes print apart, each with its own tag (#1363): one author's deletion
and insertion with hidden text, a soft hyphen, a symbol or a page break
between them, a move and the insertion beside it, or a deletion and an
insertion that repeat one `w:id` (`{--old--}{r7 @B}{++new++}{r7 @B}`:
`changes` never joins a repeated id). Proofing markers,
bookmarks that print nothing, and empty runs may intervene; visible text
and rendered content interrupt adjacency. An insertion followed by a
deletion stays in document order, one change when they touch:
`{++an++}{r1+2 @AC}{--his--}{r1+2 @AC}`.
In `{~~old~>new~~}{r4+5 @B re r3 @A}`, `re` applies only to the old side;
the new side follows the parent insertion. Story prefixes remain intact.

Converting back writes the deleted side before the inserted side. The
writer uses `+` only across a substitution's `~>`; the reader also accepts
joined ids on other marks. Existing rules for splitting a side with
several ids remain in place. A three-level `re` chain is a spelling
change only: the rebuilding limit and the open #1134 decision are unchanged.

An inserted paragraph mark is `¶{rN @X}` at the end of its paragraph's text;
it has no `break-ins` clause on the id line. Deleted paragraph marks keep
the `break-del rN @X` clause. Formatting, page-break and row/cell clauses
keep their existing forms. Literal text that could read as a tag after a
closer or a pilcrow escapes its brace: `\{r3 @C}`. The reader restores
it as text. Attribution tags use braces; `{>>…<<}` is reserved for comments.
Saved views must migrate in one step; old wrappers no longer attribute revisions.

The reader refuses the old attribution form `{>>r1 @ASR<<}` immediately
after a `++}`, `--}` or `~~}` closer, including intervening `*`, `_` or
backtick emphasis markers. The error gives the line, old tag and replacement
`{r1 @ASR}`; an old ` on ` parent chain becomes ` re `. This includes
story-prefixed and substitution ids. To comment with that text, write
`{>>c1 @X: r1 @ASR<<}`. Free comments, explicit comment tags and old-looking
text away from that position remain comments.

`@??` is the handle of a mark with no author's name (the attribute missing
or empty); it is a tag, not a comment, and converts back to an empty
`w:author`, which Word opens cleanly (checked 2026-10-14). Its `authors:`
line, `??: ""`, keeps the date its note gives.

A comment is `{==anchored text==}{>>c5 @AC: text<<}`; a reply names its
thread root, `{>>c6 @AS re c5: Disagree.<<}`; a resolved root reads
`{>>c5 @AC resolved: text<<}`.

A note that prints outside the change holding its comment's reference
mark names that change last: `{==Kept.==}{>>c5 @AC in r13: text<<}{-- Gone.--}{r13 @JD}`,
or `<!-- p1 rev-del r13 @JD -->` with `{==text==}{>>c5 @AC in r13: text<<}`
when r13 holds nothing but the mark. Accepting a deletion that holds the
mark removes the thread, as Word does; rejecting an insertion that holds
it does the same. Converting back puts the mark inside r13 again. A note
inside its change (`{--{==Gone.==}{>>c5 @AC: text<<}--}{r13 @JD}`) needs no
`in`; neither does a mark outside any change. The view names one change:
a mark inside a move or inside two nested changes prints no `in`, and a
note whose paragraph has no r13 keeps its mark after its text, with a
warning.

A handle is the comment `w:initials` that author wrote, else the initials of
the name's words, letters and digits only; a collision appends a digit. The
header's `authors:` line gives each handle's name, counts and its
shared timestamp, or `latest 2026-10-17T16:59:00Z` when the author has
several timestamps. Revisions and comments both count. These dates report
an instant in the author's local offset when both times are recorded, for
example `2026-09-25T20:04:00-04:00`; otherwise they keep `w:date` as stored.
Shared and latest dates compare instants across offsets. Converting back,
notes without an inline date inherit a header timestamp when present;
explicit inline timestamps keep their own dates. An author with any undated
mark or comment (including replies, property changes, moves and other
stories) has neither a shared nor a latest timestamp in the generated
header. Each dated note then prints its own timestamp, even without
`--dates`, and undated notes print none, so writeback leaves their
`w:date` attribute absent. Older views with a shared or latest timestamp
still supply that timestamp to notes without dates. If every mark shares one
timestamp, reading the result prints it bare. A view date with an offset
writes its wall clock followed by `Z` into `w:date`, Word's encoding;
revision elements also store the instant in `w16du:dateUtc`, with the namespace
declared ignorable on the part root. A `Z` view date writes only `w:date`.
For replies or resolved comments, the writer already creates
`commentsExtensible.xml`; offset dates store their instant there, linked by
durable id. Plain comments keep their wall clock only and do not add this part.

Older views with a day range (`2026-03-02..2026-03-04`) still convert as
before: undated marks and comments take the first day at `00:00:00Z`, and
one of them, when another keeps the first day, takes the last day at `23:59:59Z`: an undated
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
tagged note (`{r12 @AC}`) keeps its id, `r12`
(`footnotes.r12` under a footnote id line), the name the `authors:`
line gives its handle, and its date: the note's own timestamp, else the
handle's from `authors:`. A note that names another change after ` re `
(`{--x--}{r3 @BO re r1 @AL}`, the plain view the same by author and
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
  --json (the snapshot)   --tables [-p t0] (the grids)   -o FILE.md|FILE.json
jubarte A B                            accept both sides' changes, compare, print the redline's view
                                       (read options apply; -o FILE writes the file instead)
jubarte compare A B                    the same; -o FILE writes the redline (.docx, .pdf, .png, .md)
jubarte diff A B                       the paragraph patch (text only; -o FILE.patch keeps it)

jubarte edit FILE -p WHERE --anchor FIND --content WITH        replace FIND (an insertion when WITH keeps FIND at its start or end)
jubarte edit FILE -p WHERE --content TEXT                      rewrite the paragraph
jubarte edit FILE -p WHERE [--anchor FIND] --delete            delete FIND, or the paragraph
jubarte edit FILE -p WHERE [--anchor FIND] --style SPEC        format FIND, or set the paragraph style
jubarte edit FILE -p c5 --content TEXT | --delete | --resolve  a comment (another author's: --rewrite-others-comments)
jubarte edit FILE -p c5 --reply TEXT                           a reply
jubarte edit FILE -p c5.text --content TEXT                    replace the text c5 covers
jubarte edit FILE -p r12 --accept | --reject                   resolve one tracked change (prints SUCCESS and a fenced diff containing the resolved text)
jubarte edit FILE -p r12 --strike | --restore                  reject another author's change visibly (--reject --visible picks by kind)
jubarte edit FILE --plan PLAN.json --skip-failed              apply what passes; the failed operations go to PLAN.failed.json
jubarte add  FILE -p WHERE --content TEXT [--before]           a new paragraph after (or before) WHERE
jubarte add  FILE -p WHERE --anchor FIND --content TEXT        a comment on FIND (--comment: on the whole paragraph)
jubarte add  FILE -p c5 --content TEXT                         a reply (also --reply TEXT)
```

`read --tables` prints the table inventory: a `table N: ROWSxCOLS
header_rows=H widths=…` line, then one line per row of tab-separated
`ids=text` cells. Add `-p t0,t2` to select the same top-level tables as
`read -p t0,t2`. The selected grids come first, then one empty line, then
exactly the view from `read -p t0,t2`, including its YAML header, tracked
changes and comments. Without `-p`, only the inventory prints.
Top-level labels `table N` match the view's `tN`. A nested table follows
its cell row, indented two spaces per level, labelled
`nested in t0.row2.cell0` (a deeper cell is
`t0.row2.cell0.t0.row0.cell1`). Each selected table includes its whole
nested tree, once in document order. A cell paragraph (`-p p11`) selects its
containing top-level table, including when the paragraph is nested.
Paragraph ranges (`-p p11-p13`) and lists may contain only paragraphs
inside tables: an outside paragraph refuses the entire selection with
exit 2. Table ranges (`t0-t2`) are not part of the `-p` grammar; list them
as `t0,t1,t2`. `--head`, `--tail`, `--changed` and `--by` cannot be combined
with `--tables`.

The snapshot uses `schema_version: 3`. `changes` lists logical changes:
a change Word wrote as several marks is one entry with its `members`,
and a deletion followed by its author's insertion is one `replacement`.
`summary.tables` and `tables[]`
count top-level body tables only; each `index` is the view's `tN`.
Every table has a `nested` array (empty for a leaf), containing its cell tables.
A nested entry has `parent` (for example `t0.row2.cell0`) and no `index`.
A deeper parent includes the cell-local table ordinal, for example
`t0.row2.cell0.t0.row0.cell1`. Paragraph ids and edit-plan schemas are unchanged.

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
(default `Jubarte`), `--datetime ISO8601` (default now, UTC),
`--suggesting-mode` (the default: tracked changes; the bundle holds
`redline.docx`, `clean.docx`, `patch.diff` and `report.jsonl`) or
`--editing-mode` (the edits land directly; the bundle holds `clean.docx`
and `report.jsonl` only), `--existing-revisions auto|keep|accept|reject|refuse`
(default `auto`: `keep` when the file has tracked changes; `--editing-mode`
refuses `keep`, so pass `accept` or `reject` for such a file). `--plan`
without a value reads or records `<stem>_plan.json` beside FILE.
`--style` takes `bold`, `italic`, `underline`, `strike`, `caps`,
`highlight=yellow`, `font=Calibri`, `size=11` and `color=FF0000`; any other
value is a paragraph style (`Heading2`). After applying, notes and `wrote …` stay on stderr.
Stdout prints SUCCESS and a fenced unified diff between the full input and
published output agent views, including source names and SHA-256 digests.
This also confirms pure `--accept`/`--reject` operations. Bundles retain
their summary line and print one block per DOCX; `-q` silences these outputs.
Anchors match literally first, then without their Markdown marks, with a
note.

When an anchor is refused with `ANCHOR_NOT_FOUND`, a whitespace-only near
occurrence can explain the miss. This is a report, never a selection: the
refusal message and exit 3 stay unchanged. Copy the document's exact characters
and submit a new request. For example, a single-space `services. Such` can
report `near: "services.  Such" ...; copy the text exactly`.

The optional `near` object accompanies the refused operation in the legacy
JSONL report and `error` in the typed JSON envelope. `total` counts all near
occurrences, including overlaps; `occurrences` contains at most five entries
in source order. Each entry has `text` (the exact document substring), `shown`
(the quoted text with visible escapes such as `\u{a0}`, `\t`, and `\n`) and
`offset` (a UTF-8 byte offset in the searched logical paragraph). A search
across paragraphs also includes each entry's `target` paragraph ID. There is
no `near` when the diagnostic search finds nothing.

The diagnostic search treats each nonempty run of `char::is_whitespace`
characters as one class, including spaces, NBSP, tabs, figure spaces, narrow
NBSP and soft breaks. It skips U+200B, U+200C, U+200D, U+2060 and U+FEFF only
in document text, to expose invisible characters an agent may have omitted.
They remain present in `text`; other characters, including punctuation and
soft hyphens, stay distinct. Typed v2 edits still require literal anchors.
The existing v1 Markdown/lookalike fallback keeps its versioned behavior;
near reporting adds no new successful match.

New words join no range they were not typed in. At a bookmark, a comment's
range or a permission range, an insertion goes before the markers that
open a range there and after the markers that close one, so in the clean
copy, after Accept All and after Reject All each range covers the words it
covered in the source. Words typed inside a range join it. A replacement
takes the place of words its range covers, and stays in that range. A
range at body level holds whole paragraphs, and its markers stay between
them. The redline of such an edit is written directly, as under `keep`.
A plan with `update_fields` goes through Compare, as before: a heading a
TOC lists holds its `_Toc` bookmark around its text, and words typed
before it refresh the TOC. So does a plan that also holds an operation
that cannot be written as a tracked change (`page_setup`,
`insert_footnote`, `insert_image`, `fill_control`). Compare can carry the
marker into the insertion, and the edit is then refused with
`CANDIDATE_INVALID`. Under `update_fields` it is refused too when Reject
All of the redline would drop a bookmark, a comment's range or a
permission range the source holds: a heading replaced whole loses its
`_Toc` bookmark that way, and the TOC would read "Error! Bookmark not
defined.". The refusal adds the route: drop `update_fields` and run
`jubarte convert FILE --update-fields` afterwards.

An empty paragraph (`<!-- p9 empty -->`) takes text as Word types it:
`edit FILE -p p9 --content "text"`, an `insert` at `position` `start` or
`end`, and a `rewrite` write one new run in your insertion. The run is
formatted as the paragraph mark (a copy of its `w:pPr/w:rPr`, without the
mark's own revisions or `w:oMath`, which only a mark takes), and the
paragraph's properties stay as they were. What the paragraph holds without
text (a field with no result, a bookmark's markers) stays where it is: an
`insert` at `start`, a `rewrite` and `--content` go before it, an `insert`
at `end` after it, so the words join no range they were not typed in.
Several inserts go to their edges, each edge in plan order (#1303); a v2
plan that mixes both edges of one empty paragraph is refused as ambiguous,
both edges being one point. A `format` on the operation applies over the
mark's formatting. In a table cell, a header or a footer too. Accept All
leaves one plain paragraph holding the text; Reject All, the empty
paragraph.

### Visible rejects

`--reject` is Word's Reject: another author's insertion vanishes and their
deletion silently comes back, and nothing in the document shows the
decision. `--strike` and `--restore` reject visibly, as your own tracked
change beside theirs:

- `-p r12 --strike` (`{"kind":"strike","revision":"r12"}`) deletes the
  words of another author's insertion, your tracked deletion inside it.
- `-p r12 --restore` (`{"kind":"restore","revision":"r12"}`) leaves their
  deletion as it is and inserts a copy of its words right after it, with
  their formatting.
- `-p r12 --reject --visible`, and `"resolve_revisions":{"reject":{"ids":[…],"visible":true}}`
  in a plan, strike each insertion and restore each deletion they name.

Both take a paragraph mark as well.

- **Strike an inserted mark** (another author split a paragraph). Your
  tracked deletion joins their insertion in the mark, as Word writes it.
  Accept All joins the two paragraphs, and the joined paragraph takes the
  second one's properties.
- **Restore a deleted mark** (another author joined two paragraphs). An
  empty paragraph whose mark is your insertion goes right after theirs,
  with their paragraph's properties and mark formatting. Accept All gives
  the words before the break their own paragraph again.

Both take a change as `jubarte changes` lists it, several marks under one
id included (`r10+11`, Word's two typing sessions; `r8+9`, words and their
paragraph's mark). A strike strikes every insertion mark of it and a
restore restores every deletion mark. A replacement's other half stays as
it is, with the note `struck r12; r11 is its deletion: restore it too to
reject the whole change`. A change with no mark of the kind is refused
(`r12+13 holds no insertion: restore it instead`), and so is a change
holding one of your own marks, naming that mark. `-p r11+12 --reject
--visible` gives a replacement its restore and its strike, in reading
order.

`--content` (a plan's `comment`) adds a comment. On words, it covers the
struck or restored words. On a paragraph mark, it covers the last word
before the break. A change of several marks takes one comment, from its
first struck or restored word to its last; a visible reject puts it on the
first of the restore and the strike. One comment covers one paragraph's
marks, so a hand-made id naming marks in two paragraphs takes none. Accept All reads as their change rejected, Reject All as the
document before both, and rejecting only your changes gives theirs back
intact. Both need the file's revisions kept (`--existing-revisions keep`,
the default for a file with tracked changes). Striking an insertion or a
mark that is already struck is `ok`, with the note `already struck`.

Your own revision of any kind is refused first (`OWN_REVISION`), with the
way to undo it. A revision with no text is refused as `EMPTY_REVISION`,
naming the comment marks it holds: there is nothing to restore. A
revision the same command also accepts or rejects is refused as
`REVISION_CONFLICT`. Each of the following is refused with a comment on
it offered instead, never a silent reject or a paragraph deletion:

- the source of a move, to restore, or its destination, to strike;
- a formatting change (there is no visible reject of formatting yet);
- a paragraph mark that ends a section, or that no paragraph follows
  directly in its container;
- to strike, an insertion holding anything but plain text, a field among
  them;
- to restore, a deletion split over several places, holding anything but
  plain text, or sitting in a hyperlink, content control or field. A
plan's `"reject":{"visible":true}` with no `ids`, `authors` or `kinds`
takes every revision, as a silent reject does. A `--reject` recorded into
a `--plan` whose reject is visible is refused: record it as `--reject
--visible`, or start a new plan.

### Refusals

Text inside another author's insertion is editable: the edit nests in
their insertion, and Accept All and Reject All give what each would
without it. Their insertion splits around the edit, and the report's
`summary` line names the pieces under its source id
(`"split_revisions":[{"from":"r3","to":["r3","r9"]}]`). Deleting a
paragraph nests the deletion in their insertions, leaves their deletions
as they are, and removes your own pending insertion outright. Text
another author deleted or moved, a paragraph holding a move, and a
paragraph or run whose mark or formatting change is still tracked, are
not. The refusal
(`UNSUPPORTED_STRUCTURE`, exit 3) names that revision by id, author and
text, then each way around it with what that way costs, in the syntax the
command used:

```text
error: plan refused: UNSUPPORTED_STRUCTURE (op-1): the text is inside r4, "bankers" deleted by "Bob".
  put it back visibly:  -p r4 --restore (their deletion stays, your insertion shows)
  let it go:            -p r4 --accept  (their words leave the document for good)
  put it back silently: -p r4 --reject  (their deletion disappears, no trace)
  or comment on it:     add -p p0 --anchor "…" --content "…"
```

Accepting or rejecting goes in the same command, before the edit
(`edit FILE -p r4 --reject -p p0 --anchor bankers --content brokers`). A
restore goes in a command of its own (an edit of its words in the same
command is refused, saying so): the next edit finds the restored words,
your own insertion, and rewrites them in place. A
`--plan` quotes `{"kind":"restore","revision":"r4"}`,
`"resolve_revisions":{"reject":{"ids":["r4"]}}` and a
`{"kind":"comment",…}` operation instead; `edit apply` quotes a whole
`"resolve_revisions":{"accept":[…],"reject":[…]}` naming the revision's
`short_id` and `long_id`, and a comment on the refused operation's own
`target`, but no restore: its plans have none yet. The visible route is
offered only where restore would land: not for a deletion split over
several places or holding a field, a picture or a note reference. Each JSON route pastes into its plan as written. An anchor that
misses the visible text but reaches into a deletion gets the same refusal,
not `ANCHOR_NOT_FOUND`, whether it is copied with the deletion's marks
(`{--bankers--}`) or without them, with a tab as `\t` or a curly quote
typed straight. A move offers "keep the move" and "undo it silently";
a formatting change, "keep their formatting" and "undo it silently". A
field, hyperlink or content control names itself instead.

### Plan errors

A `--plan` that does not read says where and what it takes
(`INVALID_PLAN`, exit 3). The operation is named by its `id`, else
`op-N`, then its kind, the field, the nearest field it takes and the list
of its fields:

```text
error: INVALID_PLAN (op-2): delete: unknown field "ocurrence"; did you mean "occurrence"? fields: comment, find, occurrence, paragraph
```

An unknown kind lists the kinds, a missing `schema_version` says which
this build reads, characters after the plan's object give their line and
column, a bare number for `paragraph` shows `{"index": N}`, a single
string for `resolve_revisions.accept.ids` shows it as an array, an `id`
that is no string shows one, and an alias beside the field it is read as
says to drop one. A byte order mark before the plan is ignored. Two
slips apply as meant, each with a `note:` line on stderr and in the
`summary` event's `notes` (left out when none applies): `anchor` for
`find` on `replace`, `delete` and `comment`, and `comment` for `text` on
`comment` and `reply_comment`.
`delete` and `rewrite` take a `comment` of their own: a deletion's sits
on the struck words, in the redline only, and a rewrite's covers the
paragraph. `text` on `replace` stays an error, naming `replacement`.

### Ids across a settle

A plan that settles revisions first (`existing_revisions` `accept` or
`reject`, or `resolve_revisions`) still names the paragraphs as its author
read them: `p3` is the source's `p3` even when settling moves it. A
paragraph whose mark the settle removes merges with its neighbour. An edit
that finds its text there follows it. A whole-paragraph operation
(`delete_paragraph`, `rewrite`, …) follows it when the merged paragraph
has no other source. When it has, the refusal names each paragraph in
its view and a route that reaches the merged one:

```text
error: PARAGRAPH_RESOLVED (op-1): p3 (as read) merges into p2 (after the plan's revisions settle) together with p4; an operation on a whole paragraph needs a paragraph with one source, so select it by its text instead ({"contains":"Three four."}), or accept the revisions first (`jubarte accept FILE -o OUT.docx`) and edit OUT.docx
```

The settled id is left out when it reads like one of the source ids the
message names (`p0 (as read) merges with p1 when the plan's revisions
settle`). The text route is offered to a `--plan` of the first version
only, quoting words no other paragraph of the story holds; a plan that
settles some revisions and keeps or rejects others gets "settle the
plan's revisions first and edit the result" instead of one command. A
paragraph whose text the settle removes is refused as gone. One whose own
text the settle changes (rejecting an insertion inside it, say) is still a
source of the merge that holds its words: an operation on its neighbour is
refused, not applied to both.

### Partial runs

A plan is applied whole or refused whole. `--skip-failed` applies every
operation that passes and leaves the failed ones out:

```text
$ jubarte edit in.docx --plan plan.json --skip-failed -o out.docx
wrote out.docx (docx, 6360 bytes)
sha256: a965bd39a2bb03491a52c57aca2dd1b1ac35556602ddbbe7b5726bfbb2fb4ade
skipped 2 of 4 operations; the fixes are in plan.failed.json
```

The output is the bytes the plan without the failed operations writes.
An operation is one unit, its `comment` included. Operations that stand
or fall together share a `group` (`"group":"spacing"`): when one fails,
the rest of its group is left out too, with the code `GROUP_FAILED`. The failed operations go to
`PLAN.failed.json` beside the plan (`FILE_plan.failed.json` beside FILE
when operation flags made the plan, and into the bundle with
`--out-dir`): a plan to fix and run on the document just written. Its
`source_sha256` is that document's, its `date` the run's, and each
operation carries a `why` the parser ignores
(`"why":"ANCHOR_NOT_FOUND: \"two spaces\" does not occur in the
paragraph"`). In suggesting mode its `existing_revisions` is `keep`: the
output's tracked changes are the plan's own, and `update_fields` is left
out with it (the skip line says so). The skip line is a warning: `-q`
keeps it. The report's `summary` says `"status":"partial"` with
`ops.skipped`, and each operation left out reports `skipped` with its
code. Never skipped: `STALE_SOURCE`, a `resolve_revisions` that fails,
and an operation id used twice. A plan whose every operation fails is
refused whole (exit 3, nothing written). `--skip-failed` applies a plan,
so it does not go with operation flags that record into `--plan`.

Every write prints the primary output's `sha256:` (the `summary`
event's `output_sha256`), the digest a follow-up plan's `source_sha256`
names. A `--dry-run` that passes whole ends with `next:`, the same
command without `--dry-run`, quoted for a POSIX shell (`-q` drops it); a
dry run that would skip operations prints none. A dry run that named no
output gets the default one spelled out (`-o <stem>_modified.docx`) and a
note that the input stays as it is, so the command run as printed writes
only that file. An `X_modified.docx` input is updated in place: its
`next:` names no output, and the note says so. When the output is
already there, a note says the next command replaces it, and `next:`
ends with `--force`.

### Limits of the agent view

- Tracked paragraph marks inside comment bodies are not shown.
- Headers and footers are header lines (`headers:`, `footers:`), not body
  blocks: when one holds a change `--changed` selects, the `range:` line
  says to read those lines (with no block left when only they changed).
  A story whose paragraphs carry their own text colour (`w:color`, when
  every text run sets the same one), borders (`w:pBdr`, in the table
  border grammar), frame or indent, or differ in alignment, prints
  paragraph by paragraph:
  `id: header1`, then `p0: {text: …, center, color: "595959", border:
  bottom single 0.5pt 8496B0 space 4pt}` for each, as a block in the first
  section and as a flow map in `sections:`. A colour that opens with a
  digit is quoted, since YAML reads `000000` as the number 0. Alignment is
  `center`, `right`, or `left`/`justify` when the paragraph states it
  (`w:jc`), so a centred Header style does not centre it; no word means the
  paragraph names none. A framed paragraph (Word's page-number frame)
  adds `frame: {wrap: none, hAnchor: margin, vAnchor: text, xAlign: right,
  y: 1}`: the attributes of `w:framePr` (CT_FramePr) and their values,
  lengths in twips, in the schema's order. It is the frame the paragraph
  lays out in, its style's under its own, and the writer puts it on the
  paragraph; `frame: {}` is an empty `w:framePr`. A frame naming another
  attribute, a value the schema does not allow, or the same attribute
  twice, does not read, and the story entry with it. A paragraph's own indent (`w:ind`) adds `indent:
  {first-line: 0.25in, right: 0.25in}` right after the alignment
  (`p1: {text: …, left, indent: {right: 0.25in}, color: …}`), its sides as
  the id lines name them (`first-line`, `hanging`, `left`, `right`; Strict's
  `start` and `end` read as `left` and `right`) in inches, negative for an
  outdent; a stated 0 is printed, since it overrides the style's. It is a
  map of its own because `left` and `right` alone are the alignment. A side
  it does not name or names twice, or a value that is not inches
  (`0.25in`) or is over 22in either way, does not read, and the story entry
  with it.
- Page markers need the layout pass; `--no-page-markers` skips it. With
  `--json`, `--tables`, `--for-edit` or `-o FILE.json`, it is dropped with a
  warning (`warning: --no-page-markers dropped: no page markers in JSON,
  proceeding`) and the output is unchanged: those outputs already have no
  layout pass or page markers.
  `--track-changes`, `--comments` and `--dates` shape the Markdown view and
  cannot be combined with those outputs.
  When the layout fails, the markers come from Word's cached page breaks, else
  from hard breaks and section starts. That fallback does not mix the two
  (a page break added after Word last saved turns no page) and does not
  add the blank page an odd- or even-page section start can need.
- A table is one block for `-p`, `--head` and `--tail`.
- A change Word wrote as several marks prints its joined id on each of its
  pieces, as `changes` lists it (`{++bold++}{r7+8 @AC}{++ words++}{r7+8
  @AC}`); a comment range between the marks does not part them. Separate
  changes keep separate tags, and resolved views name each change once:
  `rev r7+8 @AC; r9 @AC`.

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
- A change is `[-old-]{+new+}`; when a side is a Word document a
  replacement inside a paragraph keeps Word Compare's order, the insertion
  first (`{+new+}[-old-]`).
  Comments and their scopes stay CriticMarkup:
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
