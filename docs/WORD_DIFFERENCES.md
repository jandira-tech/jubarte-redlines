<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Where jubarte's redline differs from Word's

Microsoft Word Compare is the reference. This page lists what still differs
between `jubarte A.docx B.docx` and Word's own redline of the same pair, says
which side we think is better, and names the mode that picks each behavior.

The evidence is the parity ladder (`tools/parity_ladder.py sweep`, baseline
`tools/parity_baseline.tsv`, 146 findings on 2026-09-28) over the
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
  pairs where we record none, and we record it in 4 pairs where Word
  doesn't. We no longer record one on a paragraph whose properties did not
  change (a story's last paragraph with a few words revised, an unchanged
  justified paragraph); Word never does.
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

### 6. Comments Word draws no balloon for

- **What happens.** Word's Save as PDF draws no balloon for a comment whose
  `w:commentRangeEnd` is dead, that is, at body or cell level, or before any
  content in its paragraph. A reply shares its parent's fate. When no
  balloon is left, Word also drops the grey markup pane, and the page stays
  full width. Live Word 16.114 on 2026-09-29: the rule predicts the balloon
  count of 149 of the 151 corpus documents with comments
  (`comment_balloons_0929` in neurotic_docx_bench,
  [WORD_COMMENT_BALLOONS.md](WORD_COMMENT_BALLOONS.md)).
- **What we do.** Only `--revisions word` copies this
  (`word_balloon_comments` in `src/convert/mod.rs`). The other styles draw
  every comment the body references.
- **Which is better.** Ours; Word's version hides the comment.

### 7. An autofit table Word widens past the page (PDF)

- **What happens.** In `tracking_without_comments/f94aeed5f5` and the
  other 333bfe069a redlines, the first table has `tblW` auto,
  `jc=center`, and a `tblPrChange` that keeps an older `tblLayout fixed`.
  Its rows also carry `trPrChange` and its cells `tcPrChange`. Word's Save
  as PDF lays the table out fixed at its cells' `tcW`, 1047pt wide on an
  A4 page, so its left columns print past the page's left edge and their
  labels ("Dersin Kodu", …) can't be read.
- **The rule.** Word does this only when both hold: the old properties
  say `tblLayout fixed`, and some row or cell of the table carries a
  property change (even an empty `trPrChange`). With only the
  `tblPrChange`, or without the old layout, Word fits the table to the
  page. The page parts, styles and compatibility mode (12 or 15) do not
  matter. Word 16 probes 33c, 33d and 33e, 2026-10-02.
- **What we do.** Only `--revisions word` copies it
  (`old_layout_still_fixed` in `src/convert/mod.rs`). Our own styles keep
  the table's live autofit, inside the margins.
- **Which is better.** Ours. Word's output loses text off the page.

### 8. Renumbered list items (PDF)

- **What happens.** Word numbers a revised list twice: once for the
  original document and once for the revised one. When an item keeps its
  paragraph mark but its two numbers differ, Word paints the old number
  plain, then the new one inked and underlined in an author colour of its
  own ("1.2.", "3.1."). The text then tabs on to the next stop. Word 16
  probes lbl0930 and lbl0930b, 2026-10-01.
- **What we do.** Only `--revisions word` copies the number pair. Our own
  styles show one number: the revised one, or the original one for an
  item whose mark is deleted. Both modes ink an inserted or deleted mark's
  label as Word does.
- **Which is better.** Ours to read. Word's pair does show where a list
  renumbered, and it pushes the item's text one tab stop right. This is
  Arthur's call; flipping the default is one condition in
  `revise_list_label`.

### 9. Annotation ids in a redline

- **What happens.** Word saves its redline with every comment, bookmark
  and revision numbered on one counter, 0, 1, …, in reading order. A note
  counts at its reference, and a header or footer at its section's
  reference. So a redline renumbers the comments of both documents.
  Evidence (Word 16 Compare, 2026-10-01):
  - docx_lots_of_comments × _addition: 180 annotations, in order, with
    comments 0 1 3 4 19 20.
  - _addition_redline × _removal_v_addition (no difference): comments
    0 1 3 4 10 11 from source ids 2 3 9 10 294 295.
  - Corpus redlines: file_195's header revision takes 1 at the first
    section, and 0048ba31dd's footnote insertion 16 between the body's
    15 and 17.
- **What we do.** Comments keep the ids their documents gave them, in both
  modes and whether or not a source carries tracked changes. Revisions
  number around them, and bookmarks above them.
- **Which is better.** Ours. Ids never reach the page, and `edit` reports
  each comment by the id its redline holds. A reading-order pass was built,
  validator-clean, and taken out for that reason.
- **Status.** Not copied.

### 10. Field results jubarte writes (`jubarte fields update`, `update_fields`)

- **What happens.** Word's Update Field recomputes `PAGEREF`, `REF`,
  `NUMPAGES`, `SEQ` and `TOC` results from Word's own pagination.
- **What we do.** `jubarte fields update`, an edit plan's
  `"update_fields": true`, Python `Document.update_fields()` and WASM
  `updateFields` write those results from jubarte's layout, the one
  `jubarte convert` paints with its default options. The page numbers are
  ours, not Word's: they match Word wherever our pagination does, and are off
  wherever it is (the PDF scores measure how often). The field codes stay,
  so Word's Update Field in the saved file replaces our numbers with its
  own.
- **Evidence.** Three corpus documents carry a TOC whose result Word wrote
  (2026-10-02, `fields update --json` against the cached result):
  `behavior__pageref_standalone_uppercase_h_7701e07f` (7 entries) and
  `behavior__sd_2447_toc_tab_alignment_8319c14c` (8 entries) match it
  exactly, text and page numbers. In `strict01.docx` Word's cached TOC
  lists 4 headings, all on page 1, where ours lists the 10 heading
  paragraphs the body holds. Whether that cache is stale was not checked
  in Word.
- **Not written yet** (the cached result stays as it was):
  - `PAGE`, which differs on every page;
  - TOC switches other than `\o`, `\h`, `\u`, `\z`, `\w`, `\x` (`\t`,
    `\b`, `\c`, `\f`, `\n`, `\p`, ...), and a TOC in a `w:fldSimple`;
  - `PAGEREF \p`; `REF` with `\n`, `\r`, `\w`, `\p`, `\t`, `\d`, or a
    bookmark that spans paragraphs; `SEQ \s` and every later field of that
    identifier;
  - number formats other than Arabic (`\* roman`, `\#`, `\@`);
  - a `PAGEREF` to a bookmark outside any paragraph or in a header, which
    the layout does not page. A `PAGEREF` to a bookmark the document lacks
    gets Word's "Error! Bookmark not defined.", and a `REF` Word's "Error!
    Reference source not found.".
- **Other differences.**
  - TOC entries carry the heading's text without its list number ("1.").
  - The layout runs once, before the results are written, so a TOC long
    enough to push the headings after it to a later page lists their
    earlier pages. Word repaginates as it updates.
  - Word's entries in `behavior__pageref_standalone_uppercase_h` mark every
    run `w:noProof`, the tab and page-number runs `w:webHidden`, and the
    text run with the `Hyperlink` character style. Ours carry none of
    these. Both put the right tab with its dot leader on the entry
    paragraph, 10 twips inside the text width.
- **Which is better.** Word's, when a Word is at hand. Ours is for a file
  that has to read right without one: a generated report, a TOC
  placeholder, a PDF made headless.

## Accept All / Reject All: where Word's result is worse (not copied yet)

`jubarte accept` / `jubarte reject` (and per-change accept/reject) follow
Word's Accept All / Reject All wherever Word keeps what the revision says.
The cases below are where Word loses or invents something. We do the
sensible thing today; each belongs in a future Word mode, never in the
default. Found by accepting our redlines in Word and comparing with Word's
accept of its own redline (`_to_improve_accepted_changes`, 2026-09-29).
Where Word's reject strays, the tie-breaker is Word's PDF of A itself: a
reject should give A back.

| # | Word does | We do | Seen in |
|---|---|---|---|
| A1 | Leaves phantom `pPrChange`/`rPrChange` records after Accept All (22/51 files) and Reject All (44/100) | No change records left | b42b and others |
| A2 | *Withdrawn 2026-09-29: not a Word defect.* Word records a style's old properties against its built-in defaults (Times New Roman, 10pt, single spacing), so an old rPr without `sz` meant 10pt, and writing sz=20 on reject gives the original back (b42b3ae070's Normal). Reject now does the same (R27, `tests/m_reject_word_parity.rs`); the redline side must record style changes the same way | — | b42b3ae070, c719b900f0, 1b4dd65cb9, 2288f27be1 |
| A3 | An outer `pPrChange` reject bleeds into a text-box paragraph | Text box keeps its own pPr | 618a11caa3 |
| A4 | Drops B's `hanging=360` from inserted numbered paragraphs that have no left indent | Keeps B's hanging indent | 3866 (74 paragraphs) |
| A5 | Drops B's run `sz`/`szCs` from text it threads into A's paragraphs (11pt instead of B's 12pt) | Keeps B's size | 440c |
| A6 | Drops `szCs`/`cs` from the paragraph-mark rPr | Keeps B's mark rPr | b4cd |
| A7 | Writes 333 "no difference" `tcPrChange` records | Only records real differences | 5b87 |
| A8 | Gives inserted or rebuilt cells the Word 2007 defaults (spacing 200/276, Calibri), which neither A nor B has | Keeps B's cell formatting | 5b87, ff42 (an inserted cell) |
| A9 | Gives a blank header or footer paragraph a leftover style (a lottery: garbage, Footer, AdoptionDate, BodyTextIndent, ListParagraph); in f125 that header grows and pushes the last line onto a second page | Keeps B's style | f125, b4cd, d8b0, ff42, 5b87 |
| A10 | Adds blank default header/footer/endnote parts, and header/footer parts neither side had. Not always invisible: a later section inherits the blank default header, and under a small top margin it pushes the body down (f8c1: top=284, header=142) | Writes only the parts the document uses, so the layout stays the revision's | 440c, f8c1 |
| A11 | Auto-creates `HeaderChar`/`FooterChar` styles | Adds no styles the document doesn't use | cda1 |
| A12 | Prunes style properties that repeat Normal | Keeps the style's own properties | several |
| A13 | Reject All of its own redline drops the paragraphs' direct spacing A had (after=0, line=240) and restores Normal to the docDefaults spacing (after=200, line=276), so A's 4 pages come back as 6 (33.79 against A's own PDF) | Reject gives back A's layout (99.96 against A's PDF) | 1b4d |
| A14 | Records an old rPr for a style only B has (`List Paragraph`: Arial, sz=18, neither side's value), which Reject All then writes | Leaves a style A lacks as B wrote it | 1b4d |
| A15 | Drops a deleted paragraph's direct properties that repeat the revision's Normal (`jc=both`, run `sz=20`), so Reject All loses the original's justification and 10pt size | Keeps the original's direct properties on deleted text | 2219 |

Word behaviour that *is* copied (it keeps what the revision says): see the
rules pinned in `tests/m_accept_word_parity.rs`, `tests/m_reject_word_parity.rs`
and `tests/m_accept_own_redlines.rs`.

## Shapes Word itself produces

These are in Word's own redlines, so jubarte producing them is not a bug
(check Word's redline with `jubarte debug WORD OURS` before fixing one):

- bookmarks crossing revision containers;
- deleted paragraph marks on the text-box paragraphs of deleted drawing
  canvases;
- a changed field laid out as the whole new field inserted, then the whole
  old field deleted;
- a changed multi-paragraph table of contents laid out as the whole new TOC,
  then the whole old TOC;
- two paragraphs joined into one laid out as the first paragraph's mark
  deleted and only the separator inserted, with the second paragraph's
  words left unchanged (checked in Word 16 on 2026-09-28; jubarte matches
  this in both modes since 0.10.0).

Word's redlines never contain a complex field whose begin and end are in
different revision states, or crossed fields. jubarte never emits either.

## Re-measuring

```text
python3 tools/parity_ladder.py sweep      # compare against tools/parity_baseline.tsv
uv run --project ../neurotic_docx_bench python tools/redline40/redline40.py run --label NAME --against baseline-0.9.3
```
