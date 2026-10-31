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
| `jubarte convert R.docx --revisions conventional` | default | Litera Compare's marks: red strike, blue underline, and a move in green, double-struck where it left and double-underlined where it landed; every revised run is marked. |
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

### 5. Revised field results Word computes (PAGE, NUMPAGES, STYLEREF, ...)

- **What happens.** Word paints a revised field's computed number in the
  run's own colour, without the revision ink or underline, although the text
  around it is marked. An inserted field whose result Save as PDF computes
  again (NUMPAGES, SECTION, STYLEREF, REF, DATE, SEQ) is painted the same way;
  DOCPROPERTY, AUTHOR, FILENAME and QUOTE keep their cached result, inked
  (Word 16 probe fldrev_p1, 2026-10-02). In the body, a deleted field of any
  kind, PAGE included, keeps its cached result, inked and struck; the footer
  cases this rule came from (d45aa3d5, 06063858) paint a deleted PAGE's
  computed number black. We still treat a deleted body PAGE like a footer
  one (open).
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
  validator-clean, and taken out for that reason. These are the `w:id`
  source keys; the agent-facing `cN`/`rN` labels make the same choice under
  the id contract — a canonical source id stays the label, while a
  duplicated, missing or noncanonical key gets a deterministically
  allocated label
  (`docs/superpowers/specs/2026-10-17-short-and-long-id-contract.md`; the
  allocation half is not implemented yet — a keyless revision still prints
  `noid`).
- **Status.** Not copied.

### 10. Watermark shape size and colour

- **What happens.** Word's Insert > Watermark sizes the text shape from
  its own measurement of the text. On a portrait Letter page with the
  default "CONFIDENTIAL" sample it writes `width:527.85pt;height:131.95pt`
  and the default colour as `fillcolor="silver"` (a Word-saved header
  quoted on Stack Overflow, question 70081174). Its numbers for A4 and
  other page sizes were not measured for this change.
- **What we do.** The `watermark` edit operation writes 527.85pt by
  131.95pt on a portrait Letter page. On any other page it writes 0.9 of
  the text width (page width less the left and right margins) at Word's
  4:1 ratio, for example 406.17pt by 101.54pt on A4 with one-inch
  margins. The colour is always written as `#RRGGBB`.
- **Which is better.** Neither for the shape: `fitshape="t"` stretches the
  text to the box either way, so only the box's size on the page differs.
  `#C0C0C0` and `silver` are the same colour.
- **Status.** Letter copied; other sizes approximate until measured
  against Word.

### 10a. Comment balloons and sticky notes (PDF)

- **What happens.** Word's Save as PDF paints each comment as a balloon
  in the grey markup pane ("Commented [initials n]: " bold, then the
  comment's text, in a box filled with the author's tint and stroked in
  the author's ink, a dotted connector from the end of the commented
  range, the range itself tinted) and writes no PDF annotation for it.
  The geometry is measured on 466 balloons of 152 corpus documents
  (`neurotic_docx_bench/scripts/word_balloon_spec.py`, 2026-10-03) and
  recorded on `paint_comment_balloons` in `src/convert/mod.rs`.
- **What we do.** Every style paints the balloon the same way and, as
  Word does, writes no PDF annotation. `PdfOptions::comment_notes` (the
  hidden `--comment-notes` flag) adds a `/Text` annotation at each
  anchor for the exceptional reader who wants one.
- **Which is better.** Word. The balloon is the print, and a viewer
  paints a `/Text` annotation as a note icon over the commented words,
  hiding them. Word's
  author colours cannot be copied: the same author gets up to six
  palette colours across the corpus (Word's reviewer table is per
  session), so jubarte colours authors in order of first appearance.
  Balloon numbers count through the document whatever the author; a
  reply takes its thread's number with "R" and its rank, the parent
  counting as 1 (RW1, KB2, JW2R2, RW3R3). A resolved comment
  (`w15:done`) is faded: text BFBFBF, stroke in the author's tint, fill
  16 % of that, the range under the pale fill.
- **Status.** Copied under `RevisionStyle::Word`. A comment's own run
  formatting (one corpus document sets 12pt Verdana bold inside a
  comment) is not painted yet: the balloon shows its text in the Balloon
  Text size. The bench's r4/r5
  probe documents (tracking on, no revisions) show a black
  changed-line bar beside the commented paragraph; three real documents
  with tracking on and live balloons show none, so no bar is drawn.

### 11. Field results jubarte writes (`jubarte convert --update-fields`, `update_fields`)

- **What happens.** Word's Update Field recomputes `PAGEREF`, `REF`,
  `NUMPAGES`, `SEQ` and `TOC` results from Word's own pagination.
- **What we do.** `jubarte convert --update-fields`, an edit plan's
  `"update_fields": true`, Python `Document.update_fields()` and WASM
  `updateFields` write those results from jubarte's layout, the one
  `jubarte convert` paints with its default options. The page numbers are
  ours, not Word's: they match Word wherever our pagination does, and are off
  wherever it is (the PDF scores measure how often). The field codes stay,
  so Word's Update Field in the saved file replaces our numbers with its
  own.
- **Evidence.** Three corpus documents carry a TOC whose result Word wrote
  (2026-10-02, the field refresh's report, then `fields update --json`, against the cached result):
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
  - the number formats `\* Ordinal`, `\* CardText`, `\* Hex`, `\#` and
    `\@`, and any format on a `PAGEREF` whose page is labelled by its
    section ("iii", "2-1"). A last `\*` switch Word does not know gets
    Word's "Error! Unknown switch argument.";
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

### 12. Revised content controls (Compare, M390)

Word comparison mode (`jubarte A B`, the default) removes `w:sdt` wrappers
and their `w:sdtPr` metadata from paragraphs carrying inserted or deleted
content. The displayed text and tracked changes remain. Accept All and
Reject All therefore recover that text without the removed control's tag,
alias, lock or nesting. Controls in unchanged paragraphs keep their wrappers.
This reproduces the existing Word reference case `missing_sectpr × fields_test`,
pinned by `tests/m390_fields_test_unwraps_sdts.rs`; `KNOWN_ISSUES.md` item 7
records the same limitation for form fills.

PowerTools faithful mode (`--mode powertools`) skips M390 and can preserve
control ancestry around the recovered source text. The deterministic nested
control regressions in `tests/compare_boundary_matrix.rs` check both modes:
Word's explicit flattening and faithful mode's original/revised ID, tag and
alias ancestry. Use faithful mode when retaining revised form controls matters;
for `fill_control`, the clean edit output also retains the control.

### 13. Duplicate identical empty permission ranges (Compare)

- **What happens.** When both documents carry the same empty permission
  range (`w:permStart`/`w:permEnd` adjacent, at body level or inside a
  paragraph), Word's Compare writes both copies: two pairs at the same
  spot, each with its own document's id (#1215 synthetic pairs `wemptybody`,
  `wemptyinparagraph`, Word 16, 2026-10-18).
- **What we do.** The union writes one pair. Two empty ranges of one editor
  that sit at the same position touch, and touching ranges merge.
- **Which is better.** Ours. Both pairs grant nothing beyond the same empty
  spot, and writing one keeps a redline of a redline from growing a pair on
  every compare.

### 14. New text that replaces a body of tables (Compare, #1826)

- **What happens.** The original's body is `table, paragraph, table` and
  the revision replaces it with new text. Word writes the new text first,
  in a paragraph of its own with an inserted mark. The deleted tables
  follow, with the original's middle paragraph between them, its mark
  deleted (w06 `45c1c5abec_table_vmerge_colspan` → `a94f4c516b_text_box`,
  Word 16's redline in the corpus).
- **What we do.** The middle paragraph keeps its place between the tables,
  its mark deleted. The new text starts the paragraph after the second
  table, the one Accept All joins it to (#1851, difference 16), instead of
  opening the body ahead of the first table. Before #1826, Word mode moved
  the paragraph ahead of the first table, and rejecting fused the two
  tables into one.
- **Which is better.** Word's, for page flow: the new text opens the page.
  Both resolve correctly: rejecting gives the original's tables and the
  paragraph between them, and accepting gives the revision. Word's own
  redline of w06 crashes Word 16.115; ours opens (difference 16). Matching
  Word would mean splitting the paragraph into a new inserted paragraph and
  the deleted mark; that is open.

### 15. A header or footer only deleted section breaks reference (Compare, #1833)

- **What happens.** The revision drops a header or footer, and the only
  section breaks that reference it are breaks the redline deletes. Word
  deletes every run of the story but keeps its closing paragraph mark
  live, reset to a blank paragraph whose `pPrChange` holds the old
  properties. When that footer holds a PAGE or NUMPAGES field, Word 16.115
  crashes opening the redline, its own included (the `file_N` family, 9 of
  the 23 release-sample redlines of #1814; s548 is `df9ffe9a8c_file_16` →
  `ec900bd182_file_17`).
- **What we do.** In Word mode, such a story is deleted whole: every
  paragraph mark goes, the closing one included, and each paragraph keeps
  its own properties (no `pPrChange`). A content control's last paragraph
  keeps its mark, as before. A story a live section also references keeps
  Word's shape (s016).

  Deleting the marks alone opened 7 of the 9. s176 (`file_9` → `file_10`)
  and s472 (`file_27` → `file_28`) still crashed. A deleted break that
  references a footer with a PAGE or NUMPAGES field crashed Word whatever
  the story's text and marks were: live, deleted, or deleted with every
  mark. It also crashed when a live section referenced the footer too. The
  layout decides (round 18, 2026-10-25).

  So the page-number story the final section inherits through deleted
  breaks moves to the final section: the final section references it, and
  no deleted break does. A header without a page field stays on its
  break. Word opens all 9 that way.

  Six of the full corpus's 16 such crashers crash with every mark deleted
  (s176, s472, 703530e684, 6bd2deb635, 8568ca15a1, aed2c1780a), and Word's
  own redline of each of the six crashes Word too (round 23). Word's log
  names its table cache: `TchtCorruption_MissingTtp` at the end of a
  deleted row whose next paragraph's mark is deleted. Where that happens
  depends on the layout. No shape that keeps the reference on the break
  opened them. We tried ghost paragraphs (mark inserted and deleted) after
  or between the deleted tables, the field in a page-number content
  control, `w:fldSimple`, PAGE alone, NUMPAGES alone, and a transplant of
  Word's own body.
- **When it moves.** Two conditions hold the move back, so it never costs
  more than a blank:
  - The redline must delete a table row. Word's crash is in its table
    cache, and none of the 10 corpus redlines with the shape and no
    deleted row crashed.
  - No section before the story's first break may set the slot. Otherwise
    Reject All would show that section's footer on the moved break's pages:
    four 21-section trusts (e.g. 2fe0dc77f9, `24b9811b05_file_22` →
    `7226967373_file_23`) labelled exhibit G-2's pages "G-1". Word opens
    all four with the reference left on its break.

  Of the full corpus's 53 redlines with the shape, the move applies to 18:
  the 16 that crashed in 0.14.0, and two that delete rows.
- **Accept and reject.** Word's Accept All gives the revision: nothing
  shows the emptied story. Word's Reject All gives the original's text,
  and its header on every page. Where the footer moved, the page-number
  footer shows on the last section only: s548's pages 7–11, where the
  original shows it on all 11. The pages before carry no footer, never
  another section's. A `sectPrChange` cannot record header or footer
  references, so no redline can give them back to the deleted breaks.
  `jubarte accept` and `reject` give the same result.
- **Which is better.** Ours. Word crashes on its own shape, and a redline
  Word cannot open shows nothing. The cost is the blank rejected footer on
  the earlier sections of the redlines that moved. Stories the final
  section does not inherit, and stories held back, stay on their breaks.
  s356 has 19 deleted breaks, each with its own page-number footer, and
  Word opens it. `jubarte validate` reports those breaks as the
  `DELETED_SECTION_PAGE_FIELD` lead, a warning: Word crashed on 9 of the 11
  release-sample redlines with that shape.

### 16. Inserted words before whole-deleted tables (Compare, #1851)

- **What happens.** A body paragraph whose mark the revision deletes
  keeps, in its own `w:ins` runs, every word Accept All keeps of it. A
  whole-deleted table follows it right away (more may follow, with
  word-less paragraphs Accept All removes whole between them), then the
  paragraph whose mark survives, the one Accept All joins those words to.
  Word's own Compare writes this shape too: 15 of its 600 release-sample
  redlines have it. Word 16.115 crashed opening 6 of jubarte's 16 such
  redlines in the 0.14.0 release sample (w03, w06, w09, w10, w14, w17),
  its log naming its table cache (`TchtCorruption_*`) or `ErrorEid` 78
  (#1814).
- **What we do.** The `w:ins` runs move, in order, to the start of the
  paragraph they join, after its properties. The paragraph they leave
  keeps its deleted text and its deleted mark. The move is body-only. It
  never crosses a section break, a live block, a table with a live row,
  or a deleted paragraph that keeps words. `jubarte validate` reports the
  shape as the `INSERTED_TEXT_BEFORE_DELETED_TABLE` lead, a warning.

  The words stay where Word writes them when a single deleted table
  follows a paragraph whose own content is nothing but those insertions,
  the paragraph after the table keeps no words, and no deleted table
  precedes it (`employee_directory_table` →
  `employee_review_john_smith_suggesting_insertions`,
  `meeting_agenda_table` → `meeting_minutes_id_paraid_overflow`, #1926).
  A paragraph that also holds deleted words still moves, and so does one
  sitting between two deleted tables.
- **Not moved: deleted paragraphs before the table.** When word-less
  deleted paragraphs sit between the words and the first deleted table,
  the words keep Word's place. Word crashed on 2 of those 42 sample
  redlines, near the sample's 23 in 600. Moving them opened one more
  (s375, w13), but cost the others Word's placement: 14.8 bench points
  each on average against Word's own redlines (29 scored).
- **Accept and reject.** The documents do not change. Word 16.115's own
  Accept All and Reject All of the moved files give the revision's and
  the original's body text (fix23 rt30), and `jubarte reject` is
  byte-identical to before. `jubarte accept` of w09 is now right: before,
  the revision's last paragraph took the original's mark properties (bold,
  16 pt) and an empty paragraph followed it.
- **Selective accept and reject.** A selection can bring the words to the
  table. Resolving the deleted mark of an empty paragraph between them
  joins the two paragraphs, so the gapped shape becomes the adjacent one.
  Word's own redline of `4640e71ddd` → `1424386e9b` opens. With every
  other mark accepted, as `tests/m_selective_revisions.rs` does, Word
  16.115 crashed opening the result. A selective `accept` or `reject`
  (by id, author or kind, and an edit plan's) moves the words too,
  whenever its result holds the shape. Word opened the three such
  half-way documents of the bench's `rejected_tracking` set alone, all
  CLEAN, the crasher included (fix23 half1851).
- **What it costs.** The redline shows the new words after the deleted
  tables instead of before them, where Word's own redline shows them. In
  a long deleted region that is far down: s487's words move past 84
  blocks.
- **Which is better.** Ours, for opening; Word's order, for reading. On
  the release sample, the move changes 15 redlines, `word/document.xml`
  only. Each was opened alone in Word 16.115:
  - 13 are CLEAN, including 4 of the 6 among them that crashed before
    (s187, s278, s462, s487).
  - s116 and s257 still crash, as they did before.

  Against Word's own redlines, the 4 that open now score 180.5 bench
  points in all, where a crash scores 0. The 8 others scored lose 112.0
  (14.0 each on average; s040 loses 50.8, its words now under the table
  Word puts them above). Net, +68.5 points on the 600-pair sample.

  Putting the deleted blocks first instead also opened w02, w07, w08 and
  w16. It is not done: Word's own order is inserted-first in 78 of its 600
  sample redlines, and the swap would change 76 of ours to fix 3 or 4.
  Nothing structural tells the crashers apart either: s115 and s118 share
  one block pattern, and only one of them crashes.

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
| A16 | Leaves the `w:ins` on the body's last paragraph mark unresolved after Accept All and Reject All, with its date truncated to the minute. Only the body's last paragraph: Word resolves the same mark in headers, footers, footnotes, table cells, text boxes, a section-break paragraph, and resolves a `w:moveTo` mark on a last paragraph moved in from elsewhere. When the last paragraph's text is inserted too, Reject All keeps an empty paragraph that still carries the mark's `w:ins` (`A \| ''`) | Resolves the mark in every story: no revision left, same paragraphs. Reject All of an inserted last paragraph removes it, giving back the original (`A`) | #1155, #1224 (synthetic) |
| A17 | Renumbers every bookmark and comment the document keeps, 0, 1, … on one counter in reading order, whenever no revision is left (read off 51 files, R5) | Keeps each id the document gave it, so a reply or edit that names `c5` after the accept still reaches c5's thread (#1264). Ids never reach the page; `tests/m_accept_word_parity.rs` and the corpus test compare them by position | 2288f27be1, d6b1d609c1 |

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
  words left unchanged (checked in Word 16 on 2026-09-28).

Word's redlines never contain a complex field whose begin and end are in
different revision states, or crossed fields. jubarte never emits either.

## Re-measuring

```text
python3 tools/parity_ladder.py sweep      # compare against tools/parity_baseline.tsv
uv run --project ../neurotic_docx_bench python tools/redline40/redline40.py run --label NAME --against baseline-0.9.3
```

### Word keeps one cell count when a row's cells are repartitioned (#1817)

When a paired row's cell boundaries move (a cell split or merged, a column
added or deleted, a boundary shifted), Word 16's Compare meshes the row. The
two rows pair, the row keeps one cell count, and Word records no cell
insertion or deletion: the revision's extra cell holds inserted text, the
original's deleted text. Every cell of the row records its properties in a
`tcPrChange`, an unchanged cell its own, and no recorded span overflows its
side's grid. In 21 Word Compare redlines of such rows
(`jubarte-loop/fix23/ab1817`), Word wrote no `cellIns` or `cellDel`. Its own
Reject All of a split, and its Accept All of a merge, keep the extra cell,
empty; a shifted boundary comes back wrong both ways.

Word mode copies that mesh, and it is worse than the sources: accepting a
merge keeps an empty extra cell, and rejecting a split keeps one.
`--mode powertools` keeps complete inserted and deleted rows instead and
restores the authored cell counts, spans and text on both acceptance and
rejection; use it when exact table reconstruction matters. Word mode used
those row lifetimes too before #1817, outside five observed free-mesh routes,
and Word 16.115 would not open the deleted rows they wrote for the w04 and
w23 fixtures.

Word's mesh differs from jubarte's in four ways:

- Word derives new grid columns so the recorded spans fit (`i02a`: an
  authored `3120,6240` recorded as `3118,1667,4565`); jubarte keeps both
  authored grids and trims the spans that would overflow them, from the last
  cell back.
- Word lays the live grid out again whenever a cell of the original joins a
  row of the revision. A fixed table that declares a width in twips gets
  that width split evenly, even where the revision's grid already has the
  columns: `4000,5809` became `4905,4904`. An autofit table gets it split by
  its content (`4544,5265`). A fixed table without a width takes its cells'
  widths (`9809,9809`, wider than the page). A percentage also counts the
  cells' side margins. jubarte rebuilds only a grid with fewer columns than
  the row, and splits the declared width evenly whatever the layout, or the
  text width when there is none, without the margins (#1914, Word 16.115
  probes in `jubarte-loop/fix23/r55/syn`). Both sides now keep the declared
  width and give the original's cell the revision's cell properties.
- Word records `tblPrChange` and `tblGridChange` on 20 of the 21 tables, even
  when neither changed; jubarte records them only when they changed.
- Word pairs the cells of a row by their text when a column is added in the
  middle (`i09`) or one cell's text is divided between two (`i11`); jubarte
  pairs them in order. Open.

The observed references that first showed the mesh remain its regressions:
`sd_2766_pirates_tracked_changes_3285d875` →
`sd_1494_table_left_indent_11bb24c7`
(`737a91c72fe6eb78743835a3566c7af356c29f5b1915734a3b48a5ba142b85d4`, accepted
cell counts `[[4, 4], [3, 3]]` for the source's `[[3, 3], [3, 3]]`),
`pirates × table_border_widths` (`28d97cb1f1`),
`eigenpal_docx_editor_suggesting_mixed_edits × employee_directory_table_2`
(`b351836b34`) and `rFonts_rstyle_linked_combos × rtl_table` (`b26364fa4f`).
They are indexed by `corpus/word/notices/RENAMED.csv` in
`neurotic_docx_bench`. Faithful mode keeps exact authored cell partitions,
spans and text on both projections for all of them.

### Folded demo title paragraph history (M465)

The saved Word redline `615a0006cf` folds deleted “Double Spacing Bold Demo”
into the inserted “1. What this is” heading. The heading keeps the revised
bottom border and paragraph spacing live, but Word saves an empty
`pPrChange` rather than the original title's `spacing line="276"`.

Jubarte keeps that mixed paragraph shape and the revised heading layout.
When format tracking is enabled, its history contains the complete original
paragraph properties so rejection restores the authored title spacing.
Disabling format tracking retains the existing behavior without a property
history. This improves source reconstruction over Word's empty snapshot.
