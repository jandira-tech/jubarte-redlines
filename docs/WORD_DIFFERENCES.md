<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Where jubarte's redline differs from Word's

Microsoft Word Compare is the reference. This page lists what still differs
between `jubarte A.docx B.docx` and Word's own redline of the same pair, says
which side we think is better, and names the mode that picks each behavior.

The evidence is the parity ladder (`tools/parity_ladder.py sweep`, baseline
`tools/parity_baseline.tsv`, 148 findings on 2026-09-28) over the
neurotic_docx_bench `word_based` corpus, plus the 40-pair Word-redline guard
(`tools/redline40`).

## Modes

| Command | Mode | What it reproduces |
|---|---|---|
| `jubarte A B` | `--mode word` (default) | Word Compare's layout: word-level detail inside paragraphs, replaced paragraphs merged the way Word merges them, Word's alignment passes. |
| `jubarte A B --mode powertools` | also `--powertools-faithful` | Open-Xml-PowerTools / Docxodus: coarse paragraph fallback (detail threshold 0.15), no Word alignment passes. |
| `jubarte convert R.docx --revisions conventional` | default | Red strike, blue double underline, green moves; every revised run is marked. |
| `jubarte convert R.docx --revisions word` | | What Word's Save as PDF paints, Word's own mistakes included (below). |
| `jubarte convert R.docx --revisions custom --revision-palette …` | | Your own marks. |

The rule for the two Word modes is that they copy Word even where Word is
wrong. Our improvements live only in the defaults (`conventional`), never in
`word`.

## Differences

### 1. Pre-existing tracked changes: Word keeps them, we accept them first

- **What happens.** When A or B already carries tracked changes, Word keeps
  them in its redline as history. jubarte accepts them first and compares
  the accepted documents.
- **Effect.** 29 of the 32 corpus pairs whose Word redline does not
  reconstruct A (reject all) or B (accept all) are pairs of this kind. For
  all 32, ours reconstructs both.
- **Which is better.** It depends on the reader. Word's output shows who
  proposed what before the comparison. Ours guarantees
  `accept(redline) ≡ B` and `reject(redline) ≡ A`, which is the contract
  agents and scripts rely on.
- **Status.** Open decision
  ([C4_preexisting_revisions_decision.md](C4_preexisting_revisions_decision.md),
  options A/B/C). Until it is decided, `--mode word` also accepts first. So
  this is the one place where `--mode word` does not yet follow Word.

The other 3 of the 32 are not differences:

- `verdana_font_demo_id_paraid_overflow_2 × …` and
  `verdana_bold_large_font_id_paraid_overflow × verdana_font_demo_…`: the
  Word references are stale. Their reject side reads "demonstrates … designed
  for screen readability at small sizes", which no current fixture contains.
  Word compared an older version of the file. The two sources have identical
  text, and ours correctly marks nothing.
- `mcdoc × meeting_agenda_table_2`: Word marks the VML `mc:Fallback` copy of
  a text box ("hello") deleted. The ladder's source walk skips fallbacks, so
  it counts the word as extra. Ours deletes the same text box.

### 2. How a changed paragraph is split into insertions and deletions (L1, 29 pairs)

- **What happens.** Word and jubarte usually mark the same words. The order
  and grouping of the inserted and deleted runs sometimes differ. Typical
  case: Word ends an insertion at a paragraph mark where we run on (`ours
  ins 'Titlestylecentered'`, `word ins 'Titlestyle'`).
- **Example.** The w23c case in `tests/m32_word_alignment.rs`: with five
  identical deleted paragraphs, both sides fuse the replacement into the
  first copy. Word also keeps a shared " sample " inside the fused paragraph
  as unchanged.
- **Which is better.** Word. The accepted and rejected text is identical;
  only the look of the redline differs. We close these case by case, each
  with a test against Word's own output.

### 3. Formatting-change records (L2/L3)

- **Paragraph formatting (`w:pPrChange`).** Word records the change in 8
  pairs where we record none, and we record it in 5 pairs where Word
  doesn't.
- **Table formatting (`w:tblPrChange`, `w:tblGridChange`).** We record more
  than Word does (5 pairs).
- **Row and cell formatting (`w:trPrChange`, `w:tcPrChange`).** Word records
  them in 3 pairs where we don't.
- **Other property differences.** A handful of property elements appear on
  one side only (`w:spacing`, `w:noProof`, `w:tblLook`, drawing extents).
  Most come from Word re-serialising the document it saved.
- **Which is better.** Word, where it records a real formatting change we
  miss. Neither side changes the accepted or rejected text.

### 4. Author colours

- **What happens.** Word colours revisions by author from a 20-colour
  palette. It assigns colours in the order authors first appear in the
  Word session, not by name, so the same author changes colour between
  sessions.
- **What we do.** `--revisions word` uses that palette in document order.
  `conventional` uses fixed red/blue/green, which reads the same every time.
- **Which is better.** We think `conventional`; Word's colours are not
  reproducible.

### 5. Revised field numbers (PAGE, NUMPAGES)

- **What happens.** Word paints a revised field's computed number in the
  run's own colour, without the revision ink or underline, although the text
  around it is marked.
- **What we do.** Only `--revisions word` copies this. Our own styles mark
  the number like any other revised text.
- **Which is better.** Ours; Word's version hides a revision.

## Shapes Word itself produces

These are in Word's own redlines, so jubarte producing them is not a bug
(check Word's redline with `jubarte debug WORD OURS` before fixing one):

- bookmarks crossing revision containers;
- deleted paragraph marks on the text-box paragraphs of deleted drawing
  canvases;
- a changed field laid out as the whole new field inserted, then the whole
  old field deleted;
- a changed multi-paragraph table of contents laid out as the whole new TOC,
  then the whole old TOC.

Word's redlines never contain a complex field whose begin and end are in
different revision states, or crossed fields. jubarte never emits either.

## Re-measuring

```text
python3 tools/parity_ladder.py sweep      # compare against tools/parity_baseline.tsv
uv run --project ../neurotic_docx_bench python tools/redline40/redline40.py run --label NAME --against baseline-0.9.3
```
