<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
SPDX-License-Identifier: AGPL-3.0-only
-->

# Word layout rules we reconstructed

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

## Redline markup

- **Deleted text is visible text.** A text box whose words are all
  `w:delText`:
  - is still a text box, not the picture it holds (4c0cf02);
  - still lays out paragraph by paragraph, one line per deleted paragraph,
    centred as styled (bb1600d).

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

## A header picture after text (measured, not yet implemented)

In a header paragraph, text followed by an inline picture keeps Word's body
rule:

- If the picture fits beside the text, it shares the line. The line deepens
  to the picture and the picture's bottom is the baseline. jubarte does this.
- If the picture does not fit, the text keeps line 1 and the picture opens
  line 2 below it.
  - Probe: text bottom 43.0, picture 45.3–135.3, body 152.2.
  - jubarte puts the picture first (redline 7429fdae's "RA ID" above its
    journal banner).

## Pages and keep-with-next

- **Parity blank page (fb241e2).** With `w:evenAndOddHeaders`, a section that
  restarts page numbering on the same parity as the previous page's number
  gets a blank page first, without headers or footers, so odd numbers stay on
  right-hand pages.
- **keepNext with an inline picture or box.** A `keepNext` paragraph moves to
  the next page with the following paragraph when that paragraph's first line
  holds an inline picture or text box that doesn't fit. The line counts at the
  object's full height.
  - Live Word probe: a heading over a 600pt inline picture opens page 2.
  - Redline d20125ec: 11 pages, as in Word.

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
- **A topAndBottom float can hang below its anchor paragraph.** Any later
  line that meets its band starts under it, not only the anchor paragraph's
  own lines.
  - Part a 8aea3634: the rule 11.7pt under an empty paragraph sits above its
    heading. With the rule 0.5pt or 5pt tall, the gap from the rule's foot to
    the next rule stays 12.30pt.

## Open, measured but not yet reconstructed

- **Photo inside a deleted text box:** it does not paint yet (d20125ec).
- **Batch compares.** In `word_redline.py`'s default batch mode, "open produced
  2 new documents" happens about every other pair after a compare. Notes are in
  `neurotic_docx_bench/scripts/WORD_SCRIPTS_REVIEW_2026-09-25.md`.
