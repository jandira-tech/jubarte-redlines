<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
SPDX-License-Identifier: AGPL-3.0-only
-->

# When Word draws no comment balloon: reviewed 2026-09-29

Measured on live Word for Mac 16.114 through `scripts/word_pdf.py` in
`neurotic_docx_bench`. A balloon is a `Commented [` label in the exported PDF text.
The probes and their Word PDFs are the `comment_balloons_0929` set of
`neurotic_docx_bench/corpus/word` (origin `grok_run/comment_balloons_0929`, whose
README has the round-by-round table and the generators).

## Why it matters

37 of the 151 corpus documents that hold comments have no balloon at all in Word's
PDF, and no markup pane (the page stays 612 pt wide). jubarte's redlines of the same
pairs do get balloons, so Word widens those pages and the redline scores low for a
layout choice, not for a wrong change.

## The rule

A comment gets a balloon when

1. the body references it (`w:commentReference`), and
2. its `w:commentRangeEnd`, if it has one, is **live**: inside a `w:p`, with content
   before it in that paragraph (`w:t` or `w:delText` with text, `w:drawing`, `w:pict`,
   `w:object`, `w:sym`, `w:tab`, or another comment's `w:commentReference`), or with
   its own `w:commentRangeStart` before it in that paragraph (an empty range followed
   by its reference is a reference alone; round 6, 2026-10-03), and
3. for a reply (`w15:paraIdParent` in `commentsExtended.xml`, matched to the parent's
   last `w14:paraId`), its parent gets one.

A range end at body level (between blocks, or right after `</w:tbl>`) or first in its
paragraph is **dead**: that comment and its replies get no balloon. Where the
`w:commentReference` sits does not matter (R5), and neither do settings, content types,
rels, the comments parts or styles (AB2 to AB4 part swaps).

Applied to the 151 corpus documents with comments, the rule predicts Word's balloon
count exactly for all 151 and zero for all 37 zero-balloon documents
(`comment_balloons_0929/survey6.py`). The own-start clause is what the first reading
(149/151) lacked: `6ef6726c28` (`<start 11/><end 11/><ref 11/>`, Word 1) and
`1672057675` (`<start 0/><start 1/><end 0/><ref 0/><end 1/><ref 1/>`, Word 4). Round 6
(`round6.py`, 13 shapes in Word 16 on 2026-10-03) confirms it: an empty range gets its
balloon whether its start and end are adjacent, an empty `w:t` run sits between, the
reference is styled or plain, another comment's stray start precedes, the paragraph is
the first, the last or the only one; the end-first-in-paragraph control stays dead.
The 2026-09-29 probe that read the empty-`w:t` case as dead was wrong.

## What jubarte does today

Measured on the 2611-pair run `neurotic_docx_bench/results/redlines_0929_full`
(jubarte 0.10.0, 86b6b5d3, `jubarte-rust-inproc`); 174 of those pairs have comments in
Word's compare.

### 1. Range ends move to a live position (0.9.3 and 0.10.0)

Word's compare keeps the source's range-end layout. On
`b175a00954_file_27` vs `f32428a03a_file_28` the base has six range ends, all dead
(body level after a table, or a reply's end right after its parent's reference), and
Word's compare keeps all six dead. jubarte 0.10.0 writes the same six ends inside the
paragraph after the deleted text, all live, so Word draws balloons its own compare does
not have.

Across the run, Word's compare has dead range ends in 26 pairs, and jubarte has more
live range ends than Word in 31.

**To match Word:** keep each `w:commentRangeEnd` where the source put it, at body level
or first in its paragraph included.

### 2. Repeated comments are collapsed (0.9.3; fewer cases in 0.10.0)

jubarte 0.9.3 (673aff74) kept 4 of the 6 comments of the pair above; the two it
dropped (296, 297) repeat an earlier comment and its reply. 0.10.0 keeps all six there.

0.10.0 still writes fewer comments than Word, and fewer than the side they come from,
on six pairs. In every one, the source repeats identical (author, text) comments:

| pair (first compare id) | source comments | distinct (author, text) on the larger side | Word | jubarte 0.10.0 |
|---|---:|---:|---:|---:|
| `file_190` vs `file_191` (01f3deda92) | 15 (next) | 3 | 15 | 6 |
| `file_191` vs `file_192` (0cbac1d96f) | 15 + 2 | 3 | 17 | 8 |
| `verdana_italic_centered_demo_id_paraid_overflow` (4f0a3847e3) | 15 (next) | 3 | 15 | 6 |
| `vfdsdfcacawesd_suggesting_mixed_edits` (eb2f3c86b5) | 15 (base) | 3 | 15 | 6 |
| `docx_lots_of_comments_addition_removal` (3fcadf4761) | 6 + 6 | 4 | 6 | 4 |
| `docx_lots_of_comments_addition` (e57ad0eaa4) | 6 + 4 | 4 | 6 | 4 |

This is a correlation from the XML, not a proven mechanism: no A/B has isolated it yet.
Word keeps every repeated comment.

### Not jubarte's

Word's compare sometimes keeps fewer comments than the sources hold (`file_145` vs
`file_146`: 4 in the next, 2 in Word's compare), and a few Word compares hold more
comments than base and next together (`sample_document_word_repair_of_our_output_iter2`:
1 + 1, Word 8). The second group points at compares whose recorded base or next is not
what Word compared; check them before scoring comment counts against them.
