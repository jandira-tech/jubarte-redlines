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
- **A whole story replaced pairs them too** (`tests/m_whole_story_final_marks.rs`).
  With no paragraph kept, Word still joins the revision's last words to
  the original's first deleted paragraph. In 1053 of Word's own redlines
  in the bench corpus (no tables, the revision ending on text), the last
  inserted paragraph never keeps an inserted mark. A revision that ends
  on an empty paragraph pairs that paragraph instead, and its last words
  keep their inserted mark (m308, m309).
  - Over 2472 corpus pairs, 101 more now hold the paragraph-count check
    and none breaks. Against 2198 of Word's own redlines, 57 come out
    closer to Word's shape and none further from it.
- **Tables bound the join.** If the original's first deleted block is a
  table, the revision's last words have no paragraph to join, and Word
  keeps their inserted mark.
  - A revision that ends on an empty paragraph after a table pairs that
    paragraph with the original's closing mark. The original's last
    paragraph is deleted into it (multi_section × nested_table_rowspan).
  - An original that ends that way keeps its empty paragraph live after
    the deleted table (nested_table_rowspan × numbered_list). Accepting
    Word's own redline then keeps that empty paragraph: this is Word's
    behaviour, and we copy it.
- **A paragraph that closes a block content control does not close the
  story.** The story's own closing paragraph follows it, so that
  paragraph's mark is not paired.
- **An original spliced into the middle of the revision keeps its
  place** (employment × lease: the original's text after "3. Rent").
  Only a deleted-first tail is turned insert-first.

## Styles

- **Built-in style names pair in any case, custom names only exactly**
  (`src/builtin_styles.rs`; `tests/m_styles_matched_by_name.rs`
  `case_twin_styles_stay_apart`). Across Word's own redlines in the bench
  corpus, a revision's `normal`, `Caption` or `Footnote Reference` merged
  with the original's `Normal`, `caption` or `footnote reference` (54 of
  54), while `Subsection`, `Definition`, `Clause` and `Schedule Heading`
  stayed beside `subsection`, `definition`, `clause` and `Schedule heading`
  (6 of 6). One stylesheet holds such case twins too (`Indent(A)` beside
  `Indent(a)`, 72 redlines). Built-in means one of the 376 latent-style
  names Word 16 writes into every stylesheet.
  - Keyed in lowercase, the twins collided: 45 corpus pairs wrote one style
    id twice, and a hash-ordered pick baked the wrong twin's spacing onto
    inserted paragraphs in one compare out of six (35266bcd04 ×
    355857f6ac). Of 90 Word redlines whose sources hold twins, 23 now come
    closer to Word's styles and none moves away.

## Open

- **Comment ids of a revision that carries tracked changes.** Compare
  accepts such inputs first, and Accept renumbers bookmarks and comments
  from one counter as Word saves them. Word's redlines disagree on whether
  Compare does the same: lots_of_comments × addition keeps the revision's
  sparse 19 and 20 (`m35_comments` W1 fails on this), addition_redline ×
  removal_v_addition comes out dense (0 1 3 4 10 11, which we match), and
  addition_removal × addition_redline gives 64 and 65, which neither model
  explains. Ids are invisible and valid either way; left as is.
