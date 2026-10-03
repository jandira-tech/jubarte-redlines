<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
SPDX-License-Identifier: AGPL-3.0-only
-->

# Word layout rules we reconstructed — reviewed 2026-09-26

Microsoft Word is the reference for `jubarte convert`. We do not have Word's
source. Every rule here was measured on live Word for Mac (the Quartz print
path, which `word_pdf.py` in `neurotic_docx_bench` drives), using the same
method each time:

1. Build a small synthetic `.docx` that isolates one variable.
2. Export it with Word.
3. Read the numbers from the PDF with pymupdf.

A corpus file is only the lead that points at a rule. The probe decides the
rule. Each rule names its probe and the commit that implements it. A rule is
Word's behaviour, not a heuristic: when a corpus score disagrees with a probe,
the probe wins.

This file covers the rules found while beating the English corpus: HF
`superdoc-dev/docx-corpus`, 500 + 500 files, plus 451 Word redlines of them.

> **Re-checked 2026-10-01 (0.10.1 + unreleased main):** still the live
> rulebook — each convert fix updates this file in the same commit. Pooled
> outcome vs Word's own exports: the results tables live in
> [neurotic_docx_bench](https://github.com/jandira-tech/neurotic_docx_bench/blob/main/RESULTS.md)
> (this repo's own RESULTS.md was dropped 2026-09-30, c86aba43). When last
> pooled, 2026-09-26, **jubarte 0.9.2 ranked #1 on both pools** — mean Jaccard
> 0.647 / median 0.732 over 2,102 clean docs, and 0.466 / 0.492 over 1,416
> redlined docs.

## Shading

### Pattern shading (`w:shd`), 22a7d85

`w:shd` paints according to its pattern (`w:val`), not according to `w:fill`
alone.

| `w:val` | Word paints |
|---|---|
| `clear` | `w:fill` |
| `solid` | `w:color`; if that is `auto`, black |
| `pctN` | N% of `w:color` over `w:fill`; `auto` colour = black, `auto` fill = white |
| `nil` | nothing |
| stripes, hatches | `w:fill` (not reconstructed yet) |

Probe values:

- `solid` with colour CC99FF on fill `auto` paints CC99FF.
- `pct50`, red on blue, paints (0.502, 0, 0.498).
- `pct25`, red on white, paints (1, 0.749, 0.749).
- `pct35` auto/auto paints 0.65 grey.

The same rule applies to cells, table-style conditions, paragraphs and runs.

### Automatic text on dark shading, 6457d28

`auto`-coloured text paints **white** when the shading behind it has a
Rec. 601 luma (0.299 R + 0.587 G + 0.114 B, on 0–255) **under 75**. From 75 up
it paints black.

- **Greys:** 4A4A4A → white, 4B4B4B → black.
- **Colours:** 007C00 (luma 72.8) → white, 008800 (79.8) → black.
- **Not Rec. 709:** Rec. 709 would put 007C00 at 88.7, yet Word paints it white.

It applies to shaded cells, shaded paragraphs and shaded runs.

It does **not** apply to:

- text under `w:highlight`, which stays black even on black highlight;
- text with an explicit colour, which never changes;
- revision text, which keeps its author colour on any fill. This was checked
  with a Word redline of shaded cells, from black to yellow.

## Revision colours ("by author"), 8cc34c4

Word colours each author from a 20-colour list, in order of first appearance.
Insertions and deletions of one author share a colour. The 21st author wraps
back to the first colour.

```
D13438 0078D4 5C2E91 498205 CC3595 7160E8 038387 6D5700 CF0F1F 4E6AED
B146C2 394146 0B6A0B CA5010 750B1C 5D5A58 881798 69797E 005B70 8E562E
```

- **Not keyed by name.** One Word session keeps a single author list across
  documents. In one export batch the same name took different colours
  (Reutersmon D13438 in one run, 5C2E91 in another). A fresh Word opening one
  document assigns in order of first appearance, as measured with a 30-author
  probe.
- **Deletions too.** A second author's deletion is 0078D4, not red.
- **Compare labels changes** with the revised file's `docProps` value
  `lastModifiedBy`.
- **In jubarte:** this is the `--revisions word` mode only. The default mode
  stays the conventional workshare blue/red/green.

## Lines and tabs

- **A right tab stop that the last tab misses**, 8ae6abb. A paragraph with a
  right stop is a TOC line only if its last tab actually lands on that stop.
  - Example (redline df4265bd): a hanging indent of 879 twips, a right stop at
    595, a left stop at 879, then tab, tab, sentence. The second tab lands on
    the left stop, so Word wraps the sentence at the margin.
  - We apply the check only to heads made of bare tabs. For a TOC title head
    ("3.1.⇥Engine Fuel.⇥49") we still place the first tab at the hanging indent,
    where Word uses a custom stop (open).
- **A centre or right tab sets its text back** by half or all of the following
  text's width, including when the line wraps (d2aa5db).
- **A picture after text** joins the text's last line when it fits. The line
  deepens to the picture, and the picture's bottom becomes the baseline
  (413ae00).
  - A floating picture in the same paragraph (`wp:anchor`, including
    `wrapNone`) takes no part in this. Probe: text followed by an inline
    picture, with and without an anchored picture ahead of it; Word gives the
    same line both times (loop 6).
- **A picture before text:**
  - a full-width picture takes the first line, and the text wraps under it;
  - a picture that fits opens the first line, and the text follows it or keeps
    its tab stop (6fd8f5e, d2aa5db).
- **A paragraph of only tabs is a line of its mark**, like one of only
  spaces. Word 16 probes tab1001: an 11pt or 24pt tab over an 8pt mark,
  deleted or not, gives an 8pt line. A tab before text does not size the
  line either. t899ef4's deleted tab paragraph stood 3.8pt too tall, and
  the drift cost page 2 twenty points (63.4 → 83.4).
- **A no-break space is a letter to the wrap**, ea41e341. U+00A0, U+2007 and
  U+202F never break a line, and one at a line's end takes room where a
  plain space hangs past the edge.
  - Word 16 probe d3a-d3c (2026-10-01): d328fa3674's cell "Množství
    pneumatik v tunách" ending in U+00A0 wraps "tunách"; with a plain space
    or none it fits.

## Redline markup

- **Deleted text is visible text.** A text box whose words are all
  `w:delText`:
  - is still a text box, not the picture it holds (4c0cf02);
  - still lays out paragraph by paragraph, one line per deleted paragraph,
    centred as styled (bb1600d).
- **A list label follows its paragraph mark, not its text** (Word 16
  probes lbl0930, lbl0930b, chg0930; 2026-10-01).
  - An inserted mark (`pPr/rPr/w:ins`) inks and underlines the label
    and its tab up to the text. A deleted mark inks and strikes them.
    Bullets behave the same way.
  - Inserted or deleted text under an unrevised mark leaves the label
    black.
  - A `w:pPrChange` that puts the paragraph into a list counts as an
    insertion of the label. It is inked in the change author's colour.
- **Word numbers a revised list twice.**
  - The original count skips inserted marks. The revised count skips
    deleted marks.
  - A deleted mark shows its original number. An inserted mark shows
    its revised number.
  - Any other paragraph whose two numbers differ shows the old number
    plain, then the new one inked and underlined. Its tab runs to the
    next stop past the pair: "1." at 90, "2." at 99.84, the text at
    144.
  - The new number takes the next author colour after the document's
    own authors (0B6A0B after msi's 394146).
  - A paragraph a `w:pPrChange` numbered was in no original list, so
    it shows no old number. en r 00f467c010 shows "2)", not "1)2)".
  - Only `RevisionStyle::Word` paints the number pair. Our own marks
    show only the revised number, or the original one for a deleted
    mark.
- **A tab's gap carries its run's underline and strike** (probe
  tab0930). This includes a trailing tab: "Nund<tab>" is underlined
  from 72 to 108. It also covers an inserted or deleted tab and the
  tab after a list label.

## Headers and footers

- **A justified header line spreads** when it wraps (6742f5c).
- **Space before a header's first paragraph:** an explicit `space-before` is
  kept, an automatic one is dropped (6742f5c).
- **A `type="first"` header or footer** shows only with `w:titlePg` (6fd8f5e).
- **A picture-only header paragraph** stacks its space before, the picture,
  and its space after. The body starts below all three when they overflow the
  top margin (b76f92a).
  - Probe: the first body baseline for none / before / after / both is
    67.92 / 73.2 / 73.2 / 79.2 in Word.
  - A right-aligned logo lowered this way stays right-aligned.
- **A header picture after text keeps the body rule**, 2c1f1cee. A picture
  that fits beside the text shares the text's last line: the line deepens to
  the picture and the picture's bottom is the baseline. A picture that does
  not fit leaves the text on line 1 and opens line 2 below it.
  - Probe: text bottom 43.0, picture 45.3–135.3, body 152.2.
  - jubarte used to put the picture first (redline 7429fdae's "RA ID" above
    its journal banner).
- **A page-wide square float in a compat-15 header pushes the header line
  under it**, acbac4d3. A header line overlapping a `wrapSquare` (not tight
  or through) float that spans the text column drops below the float plus
  its `distB`, and the body follows. Word 2010 and older modes leave the
  line over the float.
  - Word 16 probes u0–u4 (2026-10-01): the first body baseline is 125.3 in
    mode 15 and 100.3 in the legacy modes, for the same header.
  - 3936a8fe56's banner float now leaves its body where Word's starts.
- **A field is named by the first word of its instruction**, 8c003773. A
  result-less `INCLUDEPICTURE ".../page1image1105008"` paints nothing,
  and neither it nor a `PAGEREF` is a `PAGE` field. 2566689f0f's header
  painted "111" from three such pictures, which shifted every line of
  its page.

## Fonts: macOS system fonts

| Font | Word line pitch | First baseline | Notes |
|---|---|---|---|
| Helvetica | exactly 1.2 em | 0.975 em | hhea gives 1.0 em. Word's line is 1.2 em, and the ascent is the win ascent (0.95) plus the leftover leading. Implemented; part a 18f71536. |
| Futura | 1.32 em | 1.06 em | follows hhea |
| Palatino | 1.10 em | 0.82 em | follows hhea |

- **Mac Roman names count.** Apple's Futura.ttc names its family only in a
  Mac Roman name record, so it has to be decoded to find "Futura" at all.
  Part a 87098dc3 used to fall back to Arial and lose Word's second page.
- **Within a collection, the face at its style's normal width and weight
  wins.** Futura's upright face is Medium (weight 500). Papyrus.ttc lists
  Condensed before Regular. For Helvetica Neue, the Regular beats the Thin.

## Run fonts

- **An `asciiTheme` slot names the ascii face even beside an explicit
  `hAnsi`**, c3a0444d. With no `w:ascii`, `hAnsi` paints only the characters
  past U+007F.
  - 0fc80afa25: `asciiTheme="minorHAnsi" hAnsi="Arial"` paints its minutes
    in the theme's Calibri; Word's PDF holds no Arial at all.
- **A character style's `w:vertAlign` raises or lowers its runs.**
  - ece10bd712's note opens on a literal "1" in "footnote reference"
    (superscript), not a `w:footnoteRef`. Word paints it at 6.48pt,
    raised; we painted a 10pt digit on the baseline.
- **A list label takes the character style its paragraph mark names**
  (`pPr/rPr/w:rStyle`), then the level's `rPr`. Word 16 probe rsty0930
  (2026-10-01): with a Courier New 16pt green style, Word paints that
  "1." over plain Aptos text.
  - tb27bda's "5.2." labels are Times through Font Style12. We drew them
    in the theme's sans: 53.5 → 73.7 (Aspose 81.9).
- **So does an empty line's mark.** Word 16 probe ms1001: an empty
  paragraph whose mark names a TNR 24 character style is a TNR 24 line.
  The mark's own `w:sz` still wins over the style. tb27bda's
  Font Style11 marks are 13.8pt Times lines, not 14.58pt Verdana 12:
  73.7 → 85.0.
- **ASCII never takes the East Asian face.** Word 16 probes 2026-10-01: a
  ")" , "," or space between ideographs paints in the run's ascii font, even
  under `w:hint="eastAsia"` or a ja-JP `w:lang`. Curly quotes and the em
  dash take `hAnsi` in a plain run and the East Asian face under the hint
  or an East Asian `w:eastAsia` lang.
  - Test: `ascii_punctuation_and_spaces_between_ideographs_take_the_ascii_face`.
- **The autospace gap is not symmetric.** East Asian text followed by Latin
  gets 0.25 em. Latin text followed by East Asian gets half the Latin
  face's OS/2 `xAvgCharWidth`: at 12pt, Arial 2.648pt and Times New Roman
  2.405pt.
  - Test: `the_gap_after_latin_text_is_half_its_fonts_average_width`.

## Redline chrome

- **Change bars**, 5785e78. The bar stands 36pt out from the left margin, or
  at half the margin when that is further out.
  - Probe: margins of 30 / 60 / 90 / 120pt put the bar at 14.88 / 30 / 54 / 84.
  - With `w:evenAndOddHeaders`, the bar sits on the outside border: odd pages
    mirror it to the right, at page width − x.
- **Balloon pane**, 80d6f94. Only comments bring Word's grey balloon
  pasteboard, which also shrinks the page.
  - Probe, with `w:trackRevisions` on: 120 tracked insertions and deletions,
    with or without formatting changes, keep the full page. One comment brings
    the pane.

## Tables

- **A keep-with-next row**, cb7cb61, needs only the start of the next row,
  when that row may break across pages. A `cantSplit` next row still needs all
  of its height.
  - Example: redline a820a0da's label row stays on page 1 above a 690pt row
    that Word splits.
- **A row moves whole only when a cell opens on a `keepNext` paragraph.**
  `keepLines` on any paragraph, or `keepNext` further down the cell, still
  lets a row that does not fit break between its paragraphs.
  - Word 16 probes k1-k6 (compat 15 and legacy alike): keepLines on the
    first or the last paragraph splits, keepNext on the last splits,
    keepNext on the first moves the row (fixtures_500 000aba38).
  - _to_improve 2ad8d15e88: its reference list ends in a keepLines blank;
    moving the row made 3 pages to Word's 2.
- **An autofit table with a dxa width gives every column its longest word,
  even past the margin.** When the words together overrun the measure, each
  column shrinks or grows to its word and the table runs off the right
  margin. A pct or auto table stays at the measure and breaks the words
  (08c53c4f).
  - Word 16 probes a1-a7: two columns of 30 and 40 underscores make a
    469.2pt dxa table in a 468pt measure, but a 467.5pt pct or auto table
    that wraps one underscore each. ee7b597379's tcW 4321/720/4381/4381
    become 3945/1535/4304/989, the four longest words, and the table
    ends at 610pt.
- **Tables with nothing between them are one table**, f2fd0c8b. The second
  table's rows keep their own cell widths and start at the joined table's
  left edge, whatever their own `jc`. The joined table is aligned by its
  widest part.
  - Word 16 probe_adj (2026-10-01): a centred 453pt table then a 441pt one,
    centred or left-aligned, start at 79.2 (j1/j3); the 441pt table alone
    is centred at 85.4. The 441pt table first, then the 453pt one: both
    start at the 453pt table's centred edge (j14, j17).
  - A fixed-layout table after an autofit one keeps its own place (j6,
    j9; 0bf5192ed0's alternating tables). Row `jc` and tracked table, row
    or cell property changes play no part (j11-j13; 29379c0 bisect).
  - Open: Word shares one grid across the joined rows (29379c0's first
    rows take the third table's 2065/6997 columns, and one row is
    centred on its own); jubarte keeps each table's own grid.

## VML lines, e8b5bc6

- **A standalone `v:line`** takes its `from`/`to` as lengths in its anchor
  frame. Probe: `from="72pt,300pt" to="400pt,300pt"` strokes exactly there.
  It uses its `strokecolor` and `strokeweight`. A bare number is in pixels.
- **A `v:line` inside a `v:group`** takes group coordinates, using the group's
  `coordsize` and `coordorigin`, scaled onto the group's box. The group is what
  gets anchored.
- **A line that names no frame** is placed in VML's default "text" frame: its
  paragraph and column, not the page. Example: part a bc404781's form rules
  name only the horizontal frame, and Word hangs them from their paragraph.
- **Open:** a character-relative anchor (`mso-position-horizontal-relative:char`)
  starts at the anchor character's x. We still take it at the column edge.
- **An inline `v:rect` with `filled="f"`** (an old "Horizontal Line" without
  `o:hr`) is stroked at its own width, past the right margin when it is
  wider. Probes hr1001 h1–h7 and k1–k6, compat 15:
  - Its box is the rect plus a 1pt foot under it. A stroke of 2pt or more
    pads the box by half the stroke on every side instead (4pt: the rect is
    drawn 2pt in). An unstroked rect paints nothing and has no foot, but
    keeps its box.
  - Alone on its line, the line is the larger of the box and the single
    line of the `w:pict` run's font, and the box sits at the line's bottom.
    The paragraph mark plays no part: a TNR 12 mark, deleted or not, leaves
    a 1.1pt rule in a Verdana 10 run on a 12.15pt line. t3c1e5d page 2
    scored 64.5 → 86.5.
  - **Open:** a rect in the middle of a text line shares that line in Word
    (its outline 0.75pt above the baseline, text after it). We put it at the
    line's end, or on its own line.

## Pages and keep-with-next

- **`w:pgNumType w:start="0"` numbers the first page 0.** Word honours a
  zero start (Word 16 probe 2026-10-01); jubarte used to clamp it to 1,
  520ade18.

- **Parity blank page (fb241e2).** With `w:evenAndOddHeaders`, a section that
  restarts page numbering on the same parity as the previous page's number
  gets a blank page first, without headers or footers, so odd numbers stay on
  right-hand pages.
- **A closing bottom border must fit with its line.** When a paragraph ends
  its border group, Word fits its last line plus the border's space and
  width above the body floor, or opens the next page with the line. Word 16
  probes bd1001: 28pt left, an exact 27.5pt line fits alone, but not with a
  1pt + 0.75pt border; a 26pt line fits with that border (27.75pt) but not
  with a 4pt space. A border group split by the page break rules only under
  its last paragraph, on the new page. 83ba58bf48's bordered "Z á p i s"
  heads page 2 in Word.
- **keepNext with an inline picture or box**, 146c6b6d. A `keepNext`
  paragraph moves to the next page with the following paragraph when that
  paragraph's first line holds an inline picture or text box that doesn't
  fit. The line counts at the object's full height.
  - Live Word probe: a heading over a 600pt inline picture opens page 2.
  - Redline d20125ec: 11 pages, as in Word.

## Justified lines and hyphens

- **A numbered cell paragraph justifies its first line from the indent.**
  The label hangs, and the item text starts at the indent like the lines
  below. It spreads to the same right edge, even when label and text share
  a style and arrive as one run.
  - Probe c4num 2026-10-01 (216pt cell, Times 12): line 1 runs 113.28 to
    282.5 and the other lines 113.3 to 282.5. We painted the merged
    "1.\tword …" run whole: 5.4pt left of the indent and ragged.
- **A compat-15 justified line squeezes its spaces to keep a word, within
  two limits.** The overflow must be at most a quarter of the line's space
  width (e522f530, 00044aa0), and at most a third of the overflowing word
  plus two spaces: shrinking may take half of what moving the word would
  leave to stretch, its space included. Word 2010 mode (14) never squeezes.
  - Word 16 probes, Times 9 and 11, 8 to 39 spaces, last words of 2 to 9
    letters: "times" at 11pt is kept up to 9.7pt and moved at 9.92pt
    (limit 9.86, with 19 or 30 spaces alike); "measured" kept at 15.2,
    moved at 16.0 (limit 15.9); an 8-space line keeps "today" at 24% of
    its spaces and moves it at 30%. Mirror words (`iiiiimmmm`,
    `mmmmiiiii`) behave alike: width counts, not letters.
  - _to_improve d06f02170c: 11.07pt over at "times" moves it (6 pages to
    Word's 7). LibreOffice's Word-interop shrink has only the quarter
    (`nMinimum = 75` in `sw/source/core/text/portxt.cxx`).
  - Table cells squeeze too: 2ad8d15e88 keeps "… amacı ve önemi" on its
    justified cell line, its Verdana spaces at 2.5pt from 2.8.
- **`w:noBreakHyphen` paints a hyphen and never breaks the line.** Word's
  PDF holds a plain 0x2D: most faces (Arial, Calibri, Aptos) have no
  U+2011 glyph. d06f02170c's "self-incrimination".
- **Before compatibility mode 15, `w:compressPunctuation` narrows a
  line's spaces to keep its last word**, 4e5607bc, with three gates.
  - **Gate 1:** the document default (`w:docDefaults/w:rPrDefault`)
    names no fonts. A missing styles part or an empty docDefaults still
    squeezes. Probe_dd bisected 3f5209785f's "Duke" and "vocational"
    down to the docDefaults `w:rFonts`.
  - **Gate 2:** the squeeze is lost to balanced widths
    (`w:balanceSingleByteDoubleByteWidth`) under an East Asian
    `w:themeFontLang` (zh, ja, ko). 496e2984f7 keeps its lines whole.
    Either setting alone still squeezes, and run languages do not count.
  - **Gate 3:** only Times New Roman and Arial spaces narrow, by about a
    fifth. Times keeps the word at 19.97% of its spaces and moves it at
    20.3%; Arial ranges 17.8–21.3%. Calibri never narrows. Georgia
    (5–10%) and Courier New (22–28%) differ, and the mechanism is
    unknown: TNR's JSTF table (Arabic kashida only) and FreeType hinting
    were ruled out. Word really narrows the spaces; glyph advances stay
    linear.
  - Tracked changes, size, `w:enableOpenTypeFeatures` and run-level East
    Asian fonts or languages make no difference. Only 2 of ~7490 corpus
    documents meet all three gates.
  - Under compressed punctuation a hyphen holds its word on the line
    (496e2984f7, +0.361); this is not the squeeze.
- **Word paints a precomposed Latin, Greek or Cyrillic letter whole**,
  f3ca28a2. Cambria's `ccmp` would split "ě" into e plus a caron. Word's
  PDF of 3509b16c7d holds ě, č and ů as single glyphs, so `ccmp` stays
  off for those scripts and on for complex ones.

## Paragraph spacing

- **contextualSpacing drops only the flagged paragraph's share of the
  gap**, 0abf6bde. Two same-style paragraphs stand A's after plus B's
  before past it apart (the larger of the two); a flagged A loses its
  after, a flagged B its excess. Word 16 probe_cx (2026-10-01): flagged
  after 6 over plain before 20 = 14, flagged after 20 over plain before 6
  = 0, plain after 20 over flagged before 6 = 20, both flagged = 0. Body,
  cells, text boxes and headers share the rule. 6ef1820785's flagged
  lines over plain empty paragraphs kept 2pt each: 7 pages, as in Word.
- **Without HTML auto spacing the spacing adds up**, 6bb7f8b8. Under
  `w:compat/w:doNotUseHTMLParagraphAutoSpacing` two paragraphs stand the
  first's after plus the second's before apart, not the larger of the
  two, in the body and in cells.
  - Word 16 probe_sum (2026-10-01): exact 20pt lines with 6pt before and
    after step 26 without the flag and 32 with it. 4640e71ddd sets it:
    its list rows step 23.5 (20 + 1.8 + 1.8).
  - `w:beforeLines`/`w:afterLines` count hundredths of the docGrid pitch
    (12pt without a grid): 4640e71ddd steps 22.3 with its grid removed.
- **Auto spacing inherits attribute by attribute**, e12998d969. Only an
  explicit `w:beforeAutospacing`/`w:afterAutospacing` turns auto spacing
  on or off. A plain `w:before`/`w:after` from a derived style or the
  paragraph is kept as the fallback.
  - Word 16 probe asp0930 (2026-10-01): Normal sets `after=100
    afterAutospacing=1`.
    - Heading 2 based on it sets `after=80` and keeps the 14pt: line
      step 27.84.
    - A direct `after=0` keeps it too: 27.60.
    - `afterAutospacing="0"` alone brings back Normal's 5pt: 18.96.
  - e12998d969's headings sat 4pt over their text: 41.0 → 78.8 (Aspose's
    free converter 75.9).

## Footnotes cited in table cells

- **A note cited in a cell sits on the page of the row that cites it**,
  ece10bd712. The row's notes raise the floor before the row is placed.
  - A cantSplit row that no longer fits above that floor moves to the
    next page whole, and claims its notes there.
  - So does any other row with no head that fits above the floor.
  - A row that splits takes its cut at the raised floor.
  - Word: note and row both on page 1, "(9) Simulated" opens page 2.
    We dropped the note and kept the row.

## Endnotes

- **Each endnote mark reads its note's place in reference order**,
  9970aa9e, in the last section's `w:endnotePr` format (else the settings
  part's), lowerRoman from 1 by default. The note opens on the same mark
  (`w:endnoteRef`). Strict01 p13 reads "i This is an endnote."; 9134397db6
  "Ouchi.ii".
- **The separator note is a line of its own over the notes**, d7a8dcee.
  Its paragraph lays out as usual and its `w:separator` draws a 144pt
  black rule with its foot about 2.2pt over that line's baseline
  (Strict01 p13: rule 450.48, note baseline 435.84). It opens every run
  of notes, at docEnd or at a sectEnd section.
  - Open: on a page the notes continue onto, Word draws the
    `continuationSeparator` instead (9a1c0cc482 p3, 0.48pt there).
- **A reference to a missing endnote makes the document Word-invalid.**
  The middle of three `w:endnoteReference`s pointing at an id with no
  `w:endnote` never finishes opening in Word 16: word_pdf.py timed out
  twice, while the same file with every id resolved numbers i, ii, iii.
  Each reference keeps its slot in document order, and a missing body is
  not painted. No renumbering is invented around the hole.

## Breaks, typed labels and diagrams

- **A `w:br` run sizes only a line it stands alone on.** "Top" then a 20pt
  break run keeps Top's 11pt line; a 20pt break alone on its line sizes
  that empty line (Word 16 probe 2026-10-01). _to_improve e124592dd0's
  36pt break after a 28pt title made us a page longer than Word's 2;
  fixtures_500 00accd5b's 13.5pt break is the second of two, alone.
- **An underlined inline picture keeps its run's descent under it.** Word
  sets an inline picture on its line's baseline. When the picture's run is
  drawn underlined, either by its own `w:u` or as a tracked insertion, the
  line also keeps that run's descent below the baseline. Word 16 probes
  2026-10-01, with a 30pt picture:
  - in a TNR 12 run, the next baseline is 2.64pt lower, for single and
    double underlines alike;
  - in a 36pt run, 7.68pt lower;
  - a deleted or plain run adds nothing.

  _to_improve e1c745d784's inserted logo and map each lost 2.4pt, so page
  1 held a line Word sets on page 2.

  A VML picture (`w:pict`, or a `w:object` such as an embedded
  Word.Picture.8 whose preview is a `v:imagedata`) follows the same rule.
  Word 16 probes vo (2026-10-02, TNR 10, a 150x30pt picture) put the next
  baseline 2.16pt lower when the run is inserted.

  An inline VML picture is laid out in whole pixels at 143 dpi: Word
  takes the style's points to HIMETRIC (1/100 mm), then to pixels, then
  to twips, rounding each half up. A 150x30pt shape draws 150.05x30.2pt,
  so the next line sits 0.24pt lower than under a DrawingML picture; 37pt
  draws 36.75, 40pt 39.8, 151.3pt 151.55, whatever the image (probes
  vo3-vo5, 2026-10-02, 60 sizes). DrawingML keeps its extent
  (`vml_pixel_snap`).
  The 5a6c9a5c redline lost 2.2pt under its inserted diagram, so an
  inserted empty paragraph fitted above the footer where Word moves it to
  the next page: 33 pages against Word's 35.
- **A picture-only paragraph's breaks open lines under its pictures.**
  Each is an empty line sized by the break that ends it; the last is
  sized by the paragraph mark. Word 16 probes 2026-10-01, with a picture
  in a TNR 12 run followed by a break:
  - a 7pt mark adds a 7pt line (+8.16pt);
  - 12pt and 40pt marks add their own lines;
  - two breaks add a 12pt break line, then the 7pt mark line.

  _to_improve 66cfa52b0c's logo + break + 7pt mark lost that line.
- **A typed label its own tab places is tabbed text, not a hanging list
  marker.** English holdout c73c128db4's Defpara "⇥(a)⇥text" hangs 1616
  twips with a right stop at 1332 and a left one at 1616: Word right-aligns
  "(a)" on 66.6pt, starts and wraps the text on 80.8pt. The stop a tab
  aligns to ends the text it measures at the next tab, in the same run too.
  Pages 108 to Word's 104.
- **An inline SmartArt diagram takes the line of a picture its size.** No
  empty text line above it, no gap below, and a multiple's extra under it
  from the paragraph's run style (probe: 1.5 lines leave the next line at
  209.52 for both).
- **The run holding an inline shape sizes its line.** A paragraph whose
  Calibri run holds only a 144x0.48pt `wps` rule takes that run's line,
  with or without a trailing Arial 10 space: Word 16 probe_il (2026-10-01)
  puts the next paragraph 23.76 / 38.40 / 55.44pt under the one above at
  sz 20 / 44 / 72 (Calibri single lines). 3936a8fe56's rule above each
  heading is 14.49pt at line 259, not the space's 12.41.

## Page colour

- **`w:background` is not in Word's PDF.** Word leaves the page colour out even
  with `w:displayBackgroundShape` on. It prints page colour only when "Print
  background colors and images" is on, and that option is off by default.
  - Part a c301012f: ACB9CA page, white in Word's PDF.

## Floating tables

- **A full-width floating table keeps the text above its offset.** A
  `tblpPr` table anchored to the text (`vertAnchor="text"`) with a positive
  `tblpY` sits that far below its anchor paragraph's top. If the table leaves
  no room beside it, lines that end above the table keep their place. Only
  lines that reach the table's band go under it.
  - Part a 09d6d940: the anchor paragraph and a heading both sit in the 51pt
    above the table.
  - A table with room beside it still wraps from the anchor paragraph's first
    line (case45).

## Shapes in table cells

- **A shape anchored in a cell paints in the cell.** With `layoutInCell`, the
  shape's column is the cell's text area and its paragraph is the cell
  paragraph. A `wrapNone` shape overlays the cell without growing it.
  - Part a 1f3856c4: the flowchart's eight arrows sit in the empty gap cells.
    We used to drop every shape and text box anchored in a cell.
- **Text runs beside a square float at the cell's left.** Lines whose top
  is above the float's bottom start its width plus its right distance in;
  the rest return to the cell's text left. VML and DrawingML agree.
  - 3cccdeb956's header logo: the address lines sit 9pt right of it. We
    stacked them under the logo, which moved the body down a line.
  - Probes 2026-10-01 in a cell without a styles part: Word measures the
    offset from the text left but keeps the picture inside the cell's
    edges. `margin-left:-4.75pt` paints at the cell's left edge, and
    `margin-left:200pt` stops at the right edge with the text under it.
    That clamp, and text on the left of a float at the cell's right, are
    not reconstructed yet.
- **Block arrows point where their preset says.** `downArrow`, `upArrow` and
  `leftArrow` are not rotated `rightArrow`s. The shaft is the middle half
  across the arrow and the head is min(w, h)/2 long.
- **Outline width is `a:ln/@w` as written.** Word strokes 3175 EMU as 0.25pt.
  We used to clamp outlines to at least 0.4pt.

## Table cell margins

- **A table style's `tblCellMar` top and bottom pad every row.** Each edge the
  table's own `tblCellMar` doesn't name comes from its style.
  - Live Word probe: Table Grid with 57-twip top and bottom margins steps Arial
    10 rows 17.76pt apart instead of 12.
  - Part a 1f3856c4.
- **A compat-15 table sits by its rules' outer edge**, 1d415f4c. Left
  aligned, its left rule's outer edge is on the margin and the grid half
  that rule inside, for every row, ruled or not. Centred, the grid is
  centred and the rules straddle it. Text starts its margin, or half its
  own left rule when that is wider, past its grid line.
  - Word 16 probes (2026-10-01): d3e's 1pt rule at 70.8..71.76 with text
    at 76.3 in each row; r1-r8 paint the text at the same x for 0.5pt and
    3pt rules, table or cell borders, fixed or autofit layout.

- **A table with no default table style is not pulled and pads 0.5pt.**
  Without a `w:default="1"` table style (no styles part, PHPWord, docx
  editors), an unstyled table keeps its border on the margin in every
  compat mode, and an unnamed cell margin is 0.5pt, not 108 twips, on
  every side and inner column. A named `tblCellMar` still sets the pad
  and is still not pulled.
  - Word 16 probes 2026-10-01: n1-n6, p1-p4, v1-v3, w5. With no styles
    part, compat 14 puts the rule on 71.76..72.24 and the text at 72.48;
    compat 15 puts the text at 72.72; a 216-twip `tblCellMar` gives
    82.80. meeting_agenda_table and meeting_agenda_table_2 set "Time" at
    72.48. With Normal Table present (v4-v8, w1-w7) the 108-twip pull
    holds.
- **`w:start`/`w:end` are the cell's left and right margins.** They count
  in `tblCellMar`, `tcMar` and `tblPrEx`, and `start` beats a `left` beside
  it.
  - Probes s1-s5 2026-10-01: `start`=288 sets the text 14.4pt in.
    Cicero's start/end=160 tables (70fd78a4f8) sit 8pt in.

## Rows, groups and templates

- **A row holding a nested table taller than the page splits between the
  nested table's rows.** It does not move whole to the next page. We split
  only when the page stays at least three-quarters full.
  - Part b c8d1d38a.
- **An inline group's pictures hold its line.** The group's other shapes paint
  over the paragraph's start. They don't reserve a second box below the
  paragraph. If the line moves to a new page, the shapes move with it.
  - Part a 5fb9cedf: the logo group's dot pushed the heading 64pt down.
  - Redline 09d6d940.
- **`w:linkStyles` pulls the styles from Normal.dotm.** Word refreshes the
  styles from the attached template when it opens the file. The stock
  Normal.dotm has an empty Normal over docDefaults of the theme minor font,
  12pt, after=160, line=278. Only four styles are defined in it.
  - Part a 9b100bdc: Word sets 12pt on 16pt lines (56 pages). The file's own
    Normal is 11pt on 259 (we fitted 45 pages).
  - This applies only when the file names no `w:attachedTemplate`. When it
    names a .dotx this machine doesn't have, Word keeps the file's own styles
    (a a7110391, a4168b8a, b 2c352c83).
- **A `w:fldData` inside a `w:fldChar` is binary field data, never text.**
  - Part a c690df8d: its base64 EndNote records ran 11 pages to 132.
  - Part a a7444d0b: 18 pages to 153.
- **A deleted PAGE field is recomputed.** Its code sits in `w:delInstrText`.
  Word paints the current page, not the stale cached number.
  - Redline d45aa3d5.
  - Word mode only: the revised number is painted in the run's own colour
    with no insertion underline. That is a Word mistake, so our default
    revision style keeps the mark.
- **`a:blip/a:duotone` recolours a picture by luminance.** Each pixel becomes
  c1 + (c2 − c1) × Rec.709 luma. Rec.601 misses by 8 levels.
  - c1 takes its shade (linear sRGB) and satMod (HSL) transforms.
  - Part a c301012f: green 70AD47 under accent5, shade 45%, satMod 135%, to
    white paints A4B6D6.
- **A float's `wp:align` aligns within its `relativeFrom` frame.** page/left
  is x 0 and page/right ends at the page edge. Only margin, column and
  character align inside the margins.
  - Part a 1f3856c4: the letterhead at page/left and the footer logo at
    page/right sat 70.9pt in.
- **A numbering level's `w:pPr/w:jc` beats the paragraph style's `jc`.**
  This holds when the paragraph's `numPr` is direct and it carries no direct
  `jc`.
  - Part a 8aea3634: its numbered Titre1 (heading 1, centred) items sit left.
- **A negative numbering-level indent beats the paragraph style's.** Like a
  positive one, it applies when the paragraph's `numPr` is direct and no
  direct `w:ind` names it.
  - _to_improve 66cfa52b0c: the level's left=-131 hanging=360 over List
    Paragraph's 720 sets the text at 65.45 and the bullet at 47.5 in Word.
    We kept 108, so the bullets wrapped narrower and page 1 overflowed.
  - Open: an explicit level `left="0"` still reads as absent.
- **A topAndBottom float can hang below its anchor paragraph.** Any later
  line that meets its band starts under it, not only the anchor paragraph's
  own lines.
  - Part a 8aea3634: the rule 11.7pt under an empty paragraph sits above its
    heading. With the rule 0.5pt or 5pt tall, the gap from the rule's foot to
    the next rule stays 12.30pt.
- **An inline box in a textless paragraph shares the pictures' line.** An
  inline text box or group after inline pictures sits at the next tab stop on
  the pictures' line. Its foot is at the baseline plus its run's
  `w:position`, and it is not stacked as a block of its own.
  - Part b 212a1c9d: the title bar is beside the logo, x 177.4, its foot
    29.0pt over the logo's baseline.
- **An exact-height line holds an inline box without growing.**
  - Part a 8aea3634: an inline 0.5pt rule group in an exact 12pt paragraph
    leaves the next baseline where an empty paragraph would.
- **`a:prstClr` and `a:sysClr` colour shape fills and outlines.** `sysClr`
  paints its `lastClr`.
  - Part b 212a1c9d: the School Name / LEA Name form boxes are outlined
    0.5pt `prstClr` black.
- **Front floats stack by `relativeHeight`, pictures and boxes alike.** A
  lower front float anchored later paints under the higher ones already
  on the page; `behindDoc` floats stay under the text.
  - Part b e83fa17a page 4: the photos paint over the later, lower frame
    boxes (0.782 -> 0.846).
- **A page background adds an empty Normal paragraph to the header.** With
  `w:background`, Word's header story ends with one more empty paragraph
  (the background shape's anchor) that pushes the body when the header
  outgrows the top margin.
  - Part b af0035cc: a one-paragraph header (Normal, after=10 at 1.15)
    runs to 86.9pt and the title's baseline moves from 85.07 to 99.84.
  - With no header part, Word still lays one out: an empty Header paragraph
    (Word's latent Header is single-spaced with nothing after) and the
    background's Normal one. Part b f7143477's body starts at 63.6pt, below
    its 36pt top margin.

## Text boxes

- **A fitted box sizes a blank line by its mark.** An `a:spAutoFit` box
  grows by a blank paragraph's mark line, empty or holding only spaces.
  - Probes c5b/c5c 2026-10-01: a Times 20 mark adds 23.1pt (one 22.98pt
    line). We added a factory Calibri 11 line (13.4pt).
- **An outline insets the text by half its width.** A painted `a:ln`
  straddles the box edge. Its inner half adds to every `bodyPr` inset, so
  the text moves in and a fitted box grows by the whole width.
  - Probes c6/c7 2026-10-01, zero insets: a 4pt line moves "QQ" 1.9pt
    right and down (2.1pt left when right-aligned) and fits the box 3.8pt
    taller. A 1pt line gives 0.5 and 0.9. A fixed box insets its text the
    same way and keeps its height.
  - A theme `lnRef` outline does the same at the theme's width. We do not
    paint those, so we do not inset for them yet.

## Header STYLEREF fields

A `STYLEREF` in a header or footer shows the body text in its style
(paragraph or character style, named by name or id): the first such text
on its page, or the last with `\l`. A page without one shows the last
before it, and a page before any shows the first after it (probe sref_p1,
2026-10-02). 5a6c's running head reads s. 4 / s. 9 / s. 12 where the
cached result says s. 1 (`patch_stylerefs`).

## Metafile pictures

- A WMF's text records (`META_EXTTEXTOUT`, `META_TEXTOUT`) are text in
  Word's PDF, in the font the metafile selects. We paint them as text over
  the picture's raster (`paint_meta_texts`). Its `META_RECTANGLE` boxes
  fill with the brush and take the pen's outline; a `BS_NULL` brush fills
  nothing. 5a6c's reprint diagram showed only its "+" and arrow.
- An inline picture starts where its paragraph's first line starts: a
  hanging indent pulls it out unless a list marker fills that room (5a6c:
  left 1418, hanging 851 puts the diagram 28.35pt in).

## Text boxes holding tables

A table inside a text box lays out as a table, with its fills and rules,
between the box's paragraphs (5a6c's red "End-Point Assessment Recording
Forms" banner). A table style's `w:b w:val="0"` or `w:i w:val="0"` turns
that row's bold or italic off.

## Open, measured but not yet reconstructed

- **Word re-runs autofit on open.** A Word-saved `tblGrid` is that result,
  but a grid it did not compute is ignored: probe g12 (grid 1500/1800 over
  tblW 3000) keeps the 150pt table and takes 2.9pt from column 1 for a
  long word, while 6d73303ea5's saved 8752-twip grid over tblW 8138 paints
  as saved, widening only the two "Controls" columns to the word.
  jubarte keeps tblW in both.
- **A package with no styles part seems to give table cells no margins.**
  Probe g11 paints a cell's "A" at 72.5 on a 72pt margin, where jubarte,
  using the 108-twip Normal Table default, paints it at 77.5. Not yet
  probed apart from the glyph's side bearing.
- **Photo inside a deleted text box:** it does not paint yet (d20125ec).
- **Batch compares.** In `word_redline.py`'s default batch mode, "open produced
  2 new documents" happens about every other pair after a compare. Notes are in
  `neurotic_docx_bench/scripts/WORD_SCRIPTS_REVIEW_2026-09-25.md`.
- A picture inside a `v:group` is placed in the group's coordinate space:
  its unitless left/top/width/height are relative to `coordorigin` and
  scaled by the group's box / `coordsize` (nested groups repeat that), and
  the outermost group's margin and relative frame place the result.
  b 069252c3's org chart was painted at page (0,0) from the child's own
  unitless box; Word paints it at (128.5, 284.9).
- A VML shape's `<w10:wrap type=…>` child alone names its wrap:
  069252c3's topAndBottom group pushes the next paragraph to its band's
  bottom (Anchor 103.05 -> After 267.45, also with a negative z-index).
  `mso-wrap-style` wraps the lines of the shape's own text box, not the
  text around it. Word probes 2026-10-01 of 3cccdeb956's logo: with
  `mso-wrap-style:square` and no `w10:wrap` the text runs over the logo;
  with `mso-wrap-style:none` and `w10:wrap square` it wraps. Word ignores
  the wrap on a `v:line`: bc404781's wrapped form rules move no text.
- VML's wrap distance defaults to 9pt left and right and 0 above and
  below (`mso-wrap-distance-*`). The same probes put the text 9.08pt clear
  of the logo; an explicit `mso-wrap-distance-left:0` puts it flush.
- An inline VML shape (`w:pict` with no position) is sized by its style's
  width/height, and a text box no taller than its paragraph's line sits in
  that line instead of adding a line and then its own height: live Word
  2026-09-26 sets a 21x9pt box under a 12.7pt line on the line's bottom
  (069252c3's logo letters; its 12 pages became Word's 10).
- A row's `w:gridBefore`/`w:gridAfter` leave that many grid columns empty
  (width from `w:wBefore`/`w:wAfter`), borderless, before or after its
  cells: 08648d2f's form rows open one 7tw column in, and their cells now
  sit within 0.2pt of Word (they had slid left). The empty space never
  decides a row split: 2c352c83's gridAfter rows still move whole at a page
  end, keeping their top rule.
- A paragraph's `w:framePr` overlays its style chain's framePr attribute by
  attribute: e73ba1e0's date frame sets only x=9100/y=3182 and takes
  Marginalie's page anchors and width; its footer frame sets y=12182 and
  takes the style's x=9016 (Word: 455.04pt and 450.72pt). An unbordered
  frame's text starts on its x/y with no inset. A page-anchored text frame
  in a header or footer floats at its page position and leaves the band:
  e73ba1e0's 16-line address block no longer raises the footer, so page 1
  holds Word's text and the file is 2 pages, not 3.
