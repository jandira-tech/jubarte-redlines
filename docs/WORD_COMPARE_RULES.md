<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
SPDX-License-Identifier: AGPL-3.0-only
-->

# Word Compare rules we reconstructed — 2026-10-01

Microsoft Word's Compare Documents is the reference for `jubarte` redlines.
Each rule below was probed on Word 16 for Mac through `word_redline.py` in
`neurotic_docx_bench`, and checked with `check_redline_identity.py`. Every
probe is kept as a fixture pair next to Word's own redline
(`tests/fixtures/word_probes/<topic>/<name>_{a,b,word_redline}.docx`), and a
test compares our changes with Word's.

## Words

- **A word is a maximal run of one character class**, c885710a
  (`src/comparer/units.rs` `word_class`; fixtures `tokens/classes_*`,
  where 154 sentences give Word's 308 revisions).
  - Letters form one class. The apostrophe and ª, µ, º join them.
  - Signs form another: ASCII punctuation, U+2010–205E, currency, arrows,
    math, box drawing, CJK punctuation, and super- and subscripts or
    number forms that are not letters. `x−y` is three words; `−−−` and
    `).` are one each.
  - Some scripts are set apart from Latin and from each other, each a
    class of its own: Thai, Lao, Georgian, Hangul, Ethiopic, Cherokee,
    Canadian syllabics, Khmer, hiragana and katakana, among others.
    `アあい` is two words.
- Open: Word keeps a run of Han ideographs whole, even under zh-CN or
  ja-JP tags; we still split each ideograph. Thai tagged th-TH is
  dictionary-segmented.

## Long documents

- **An edit in every paragraph stays in its paragraph, however long the
  story**, d1468253 (fixtures `every_paragraph/case28`, `case40`,
  `seed_word`). Word gives 2 revisions per paragraph at 28 and at 40
  paragraphs. We used to replace everything past ~27 paragraphs. Two
  gates measured the longest common run against the whole story: the
  unrelated-sources shortcut, and the 2% detail threshold. Both now keep
  their anchors when most paragraphs pair in order, each with one that
  shares at least half its words.
- **A paragraph two unrelated documents share is an anchor**
  (`final_marks/kept_junk`). Forty unrelated paragraphs around one shared
  `(dolore)` come out as an insert and a delete on each side of it.

## Final marks

- **The two stories' closing paragraph marks always pair**, 35d4004c
  (fixtures `final_marks/kept_title`, `kept_title3`). After a kept title
  over a rewritten body, Word inserts the new paragraphs and then deletes
  the old ones. The last inserted paragraph joins the first deleted one
  under its deleted mark. If any other mark took the revision's closing
  mark, accepting the redline would leave an empty paragraph.
  - Check without Word: accepting a redline must give the revision's
    paragraph count, and rejecting it the original's. Across 182 corpus
    pairs, 6 more pairs now hold this. 38 still break it in both builds;
    those are open.
