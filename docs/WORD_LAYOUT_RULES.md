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

## Fonts: Mac-only font metrics (measured, not yet implemented)

| Font | Word line pitch | First baseline | Notes |
|---|---|---|---|
| Helvetica | exactly 1.2 em | 0.975 em | hhea gives 1.0 em. Word's line is 1.2 em, and the ascent is the win ascent (0.95) plus the leftover leading. |
| Futura | 1.32 em | 1.06 em | follows hhea |
| Palatino | 1.10 em | 0.82 em | follows hhea |

Parked in `macnames-parked.patch` until the Helvetica line rule has a test.

## Open, measured but not yet reconstructed

- **Heading before a full-page box.** A `keepNext` heading followed by an inline
  box taller than the rest of the page moves to the next page, and the box
  follows on the page after (redline d20125ec).
- **Photo inside a deleted text box:** it does not paint yet (d20125ec).
- **Batch compares.** In `word_redline.py`'s default batch mode, "open produced
  2 new documents" happens about every other pair after a compare. Notes are in
  `neurotic_docx_bench/scripts/WORD_SCRIPTS_REVIEW_2026-09-25.md`.
