# Functions and methods in `src/markdown/diff.rs`

46 non-test, 39 test. Part of the [symbol manifest](../README.md).

#### `diff_markdown`  (L86-107, pub)
`pub fn diff_markdown(old: &str, new: &str) -> String`

- **Does:** The two text snapshots, `old` and `new`, as one Markdown document whose CriticMarkup holds the changes: accepting them all gives `new`, rejecting them all gives `old`.
- **Calls (crate):** `lines` (markdown/diff.rs:109) L89,90; `Output::line` (markdown/diff.rs:389) L98; `escape` (markdown/diff.rs:294) L98; `hunk` (markdown/diff.rs:477) L102; `Output::finish` (markdown/diff.rs:408) L106
- **Calls (std/external/unresolved):** `.as_tag_tuple`, `.ends_with`, `.replace`, `Output::default`, `capture_diff_slices`
- **Called by:** `tests::a_decomposed_accent_is_part_of_the_word_replaced` (markdown/diff.rs:1407); `tests::delimiters_in_the_text_are_escaped` (markdown/diff.rs:1495); `tests::identical_documents_come_back_unchanged` (markdown/diff.rs:1504); `tests::blocks_added_or_removed_keep_their_markers_outside` (markdown/diff.rs:1511); `tests::a_line_added_inside_a_paragraph_takes_its_line_break` (markdown/diff.rs:1527); `tests::a_changed_marker_replaces_the_block` (markdown/diff.rs:1536); `tests::a_table_row_changes_cell_by_cell` (markdown/diff.rs:1561); `tests::unrelated_lines_are_replaced_whole` (markdown/diff.rs:1573); `tests::round_trip` (markdown/diff.rs:1580); `tests::a_changed_code_span_is_wrapped_whole` (markdown/diff.rs:1595); `tests::a_changed_image_is_wrapped_whole` (markdown/diff.rs:1610); `tests::a_changed_equation_is_wrapped_whole` (markdown/diff.rs:1628); +16 more

#### `lines`  (L109-115)
`fn lines(text: &str) -> Vec<&str>`

- **Does:** _no doc comment_
- **Calls (crate):** `opaque_units` (markdown/diff.rs:120) L114
- **Calls (std/external/unresolved):** `.is_empty`, `.strip_suffix`, `.unwrap_or`, `Vec::new`
- **Called by:** `diff_markdown` (markdown/diff.rs:86); `tests::long_lines_pair_in_linear_time` (markdown/diff.rs:1978); `patch_markdown` (markdown/unified.rs:222)

#### `opaque_units`  (L120-178)
`fn opaque_units(text: &str) -> Vec<&str>`

- **Does:** The physical lines of `text`, except that a code span, equation, image, autolink or HTML comment that spans a line break keeps its lines in one unit: [`atoms`] then reads the whole construct, not a line of it.
- **Calls (crate):** `opens_amount` (markdown/diff.rs:1127) L136; `end_of` (markdown/write.rs:6371) (by name) L167,170,174
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.extend`, `.filter`, `.get`, `.into_offset_iter`, `.last_mut`, `.len`, `.map`, `.match_indices`, `.max`, `.partition_point`, `.push`, `.sort_unstable`, `Parser::new_ext`, `Vec::new`, `line_of`
- **Macros:** `matches!`, `vec!`
- **Called by:** `lines` (markdown/diff.rs:109)

#### `marker_len`  (L183-229, pub(super))
`pub(super) fn marker_len(line: &str) -> usize`

- **Does:** The length of a line's block markers: indentation, `>` quote markers, a list marker with its task box, or a heading's `#`s.
- **Calls (std/external/unresolved):** `.as_bytes`, `.bytes`, `.contains`, `.count`, `.find`, `.first`, `.get`, `.iter`, `.len`, `.starts_with`, `.take_while`, `Some`, `usize::from`
- **Macros:** `matches!`
- **Called by:** `continues` (markdown/diff.rs:233); `plain_paragraph` (markdown/diff.rs:618); `body_words` (markdown/diff.rs:836); `marker_shape` (markdown/diff.rs:840); `paired` (markdown/diff.rs:892); `group` (markdown/diff.rs:1298); `tests::markers_are_found` (markdown/diff.rs:1373); `plain` (markdown/unified.rs:492); `visible` (markdown/unified.rs:1128); `starts_block` (markdown/unified.rs:1444); `blocky` (markdown/write.rs:409); `distribute` (markdown/write.rs:425)

#### `continues`  (L233-236, pub(super))
`pub(super) fn continues(line: &str) -> bool`

- **Does:** Whether a line continues the paragraph of a non-blank line before it: it has text and no block marker, and is not a table row or code fence.
- **Calls (crate):** `marker_len` (markdown/diff.rs:183) L235; `table_row` (markdown/diff.rs:249) L235
- **Calls (std/external/unresolved):** `.is_empty`, `.starts_with`, `.trim_start`
- **Called by:** `plain_paragraph` (markdown/diff.rs:618); `group` (markdown/diff.rs:1298); `blocks` (markdown/unified.rs:1059); `distribute` (markdown/write.rs:425) (?)

#### `open_paragraph`  (L240-246, pub(super))
`pub(super) fn open_paragraph(line: &str) -> bool`

- **Does:** Whether text may continue after a line: not blank, not a heading, a table row or a code fence.
- **Calls (crate):** `table_row` (markdown/diff.rs:249) L245
- **Calls (std/external/unresolved):** `.is_empty`, `.starts_with`, `.trim_start`
- **Called by:** `group` (markdown/diff.rs:1298); `blocks` (markdown/unified.rs:1059)

#### `table_row`  (L249-251, pub(super))
`pub(super) fn table_row(line: &str) -> bool`

- **Does:** A table row: the line starts with a pipe.
- **Calls (std/external/unresolved):** `.starts_with`, `.trim_start`
- **Called by:** `continues` (markdown/diff.rs:233); `open_paragraph` (markdown/diff.rs:240); `group` (markdown/diff.rs:1298); `visible` (markdown/unified.rs:1128); `wrap` (markdown/unified.rs:1301); `blocky` (markdown/write.rs:409); `distribute` (markdown/write.rs:425)

#### `table_row_change`  (L255-285, pub(super))
`pub(super) fn table_row_change(line: &str, open: &str, close: &str) -> String`

- **Does:** A table row added or removed whole: each cell's text is one change, so the row keeps its cells (a table drops text outside them).
- **Calls (crate):** `escape` (markdown/diff.rs:294) L268; `flush` (markdown/view_stories.rs:1334) (by name) L276,283
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.clear`, `.is_empty`, `.len`, `.push_str`, `.push`, `.trim_end`, `.trim_start`, `.trim`, `String::new`
- **Macros:** `matches!`
- **Called by:** `group` (markdown/diff.rs:1298); `tests::a_table_row_changes_cell_by_cell` (markdown/diff.rs:1561); `distribute` (markdown/write.rs:425)

#### `escape`  (L294-311)
`fn escape(text: &str) -> Cow<'_, str>`

- **Does:** CriticMarkup delimiters in text, escaped so they stay text.
- **Calls (crate):** `text_has_delimiter` (markdown/diff.rs:330) L295; `escape_points` (markdown/diff.rs:338) L298
- **Calls (std/external/unresolved):** `.is_empty`, `.len`, `.push_str`, `.push`, `Cow::Borrowed`, `Cow::Owned`, `String::with_capacity`
- **Called by:** `diff_markdown` (markdown/diff.rs:86); `table_row_change` (markdown/diff.rs:255); `reflow_body` (markdown/diff.rs:556); `paired` (markdown/diff.rs:892); `change` (markdown/diff.rs:909); `words_diff` (markdown/diff.rs:920); `group` (markdown/diff.rs:1298); `tests::delimiters_in_the_text_are_escaped` (markdown/diff.rs:1495); `tests::escape_counts_backslashes_and_skips_code` (markdown/diff.rs:1647); `tests::escape_skips_the_readers_verbatim_ranges` (markdown/diff.rs:1869); `tests::escape_is_linear_on_backtick_runs` (markdown/diff.rs:1936)

#### `text_has_delimiter`  (L330-335)
`fn text_has_delimiter(text: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `attribution_tag` (markdown/critic.rs:1330) L334
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.is_some`, `.iter`, `.match_indices`
- **Called by:** `escape` (markdown/diff.rs:294)

#### `escape_points`  (L338-369)
`fn escape_points(text: &str) -> Vec<usize>`

- **Does:** Byte positions where a backslash must be inserted.
- **Calls (crate):** `verbatim_ranges` (markdown/critic.rs:1815) L339; `escaped_at` (markdown/diff.rs:372) L354,360; `attribution_tag` (markdown/critic.rs:1330) L355
- **Calls (std/external/unresolved):** `.as_bytes`, `.dedup`, `.get`, `.is_some_and`, `.is_some`, `.len`, `.push`, `.sort_unstable`, `.starts_with`, `Vec::new`
- **Called by:** `escape` (markdown/diff.rs:294)

#### `escaped_at`  (L372-374)
`fn escaped_at(text: &str, at: usize) -> bool`

- **Does:** Whether `at` follows an odd number of backslashes.
- **Calls (std/external/unresolved):** `.bytes`, `.count`, `.rev`, `.take_while`
- **Called by:** `escape_points` (markdown/diff.rs:338)

#### `Output::line`  (L389-399)
`fn line(&mut self, line: &str)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.push_str`, `.push`, `.take`, `.to_string`
- **Called by:** `plan_value` (edit.rs:1971) (?); `diff_markdown` (markdown/diff.rs:86); `hunk` (markdown/diff.rs:477); `paired` (markdown/diff.rs:892); `group` (markdown/diff.rs:1298)

#### `Output::append`  (L402-406)
`fn append(&mut self, text: &str)`

- **Does:** Appends to the last line without a line break.
- **Calls (std/external/unresolved):** `.push_str`
- **Called by:** `stamp_confetti_then_replace` (comparer/lcs.rs:1541) (?); `walk_container` (convert/mod.rs:8907) (?); `table_block` (convert/mod.rs:13095) (?); `Layout::wrap_hanging_or_first` (convert/mod.rs:25759) (?); `Layout::wrap_para_runs` (convert/mod.rs:26005) (?); `apply_plan_json_with` (edit.rs:2284) (?); `group` (markdown/diff.rs:1298)

#### `Output::finish`  (L408-431)
`fn finish(mut self, old_trailing: bool, new_trailing: bool) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Output::add_trailing` (markdown/diff.rs:436) L428
- **Calls (std/external/unresolved):** `.push_str`, `.push`, `.take`
- **Called by:** `diff_markdown` (markdown/diff.rs:86)

#### `Output::add_trailing`  (L436-453)
`fn add_trailing(&mut self, inserted: bool)`

- **Does:** The final line break changed sides.
- **Calls (std/external/unresolved):** `.ends_with`, `.insert`, `.len`, `.map_or`, `.push_str`, `.rfind`
- **Macros:** `format!`
- **Called by:** `Output::finish` (markdown/diff.rs:408)

#### `Side::delimiters`  (L463-468)
`fn delimiters(self) -> (&'static str, &'static str)`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `group` (markdown/diff.rs:1298); `render` (markdown/from_docx/critic.rs:916) (?); `splice` (markdown/from_docx/critic.rs:1344) (?); `whole_change` (markdown/from_docx/critic.rs:1397) (?)

#### `hunk`  (L477-514)
`fn hunk(out: &mut Output, olds: &[&str], news: &[&str])`

- **Does:** _no doc comment_
- **Calls (crate):** `reflow` (markdown/diff.rs:523) L478; `Output::line` (markdown/diff.rs:389) L480; `pair` (markdown/diff.rs:630) L484; `paired` (markdown/diff.rs:892) L489; `group` (markdown/diff.rs:1298) L508,509
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.iter`, `.len`, `.map_or`, `.position`, `.split`, `Entry::Pair`, `Some`, `only`
- **Macros:** `matches!`
- **Called by:** `diff_markdown` (markdown/diff.rs:86)

#### `reflow`  (L523-548)
`fn reflow(olds: &[&str], news: &[&str]) -> Option<String>`

- **Does:** A hard-wrapped paragraph whose lines all moved, as one word diff.
- **Calls (crate):** `plain_paragraph` (markdown/diff.rs:618) L524; `similarity` (markdown/diff.rs:859) L533; `words_diff` (markdown/diff.rs:920) L536; `ChangeFilter::matches` (changes.rs:405) (by name) L538; `reflow_body` (markdown/diff.rs:556) L547
- **Calls (std/external/unresolved):** `.count`, `.join`, `.len`, `.replace`, `Some`
- **Called by:** `hunk` (markdown/diff.rs:477)

#### `reflow_body`  (L556-607)
`fn reflow_body(old: &str, new: &str) -> String`

- **Does:** Word diff of a reflowed paragraph.
- **Calls (crate):** `tokens` (ambiguous: `lexical_pairing_contract_tests::tokens` (comparer/cross_para.rs:2144); `tokens` (markdown/math.rs:640)) L557,558; `reflow_body::{fn weigh}` (markdown/diff.rs:559) L572; `reflow_body::{fn owner}` (markdown/diff.rs:565) L573; `owned` (ambiguous, 4 candidates) L584,585; `widen` (markdown/diff.rs:1182) L597; `escape` (markdown/diff.rs:294) L601; `change` (markdown/diff.rs:909) L603
- **Calls (std/external/unresolved):** `.as_tag_tuple`, `.checked_sub`, `.collect`, `.concat`, `.into_iter`, `.into_owned`, `.last_mut`, `.map`, `.push_str`, `.push`, `Vec::new`, `capture_diff_slices`
- **Called by:** `reflow` (markdown/diff.rs:523)

#### `reflow_body::{fn weigh}`  (L559-564)
`fn weigh<'a>(tokens: &[&'a str]) -> Vec<&'a str>`

- **Does:** _no doc comment_
- **Calls (crate):** `weight` (markdown/diff.rs:610) L562
- **Calls (std/external/unresolved):** `.collect`, `.flat_map`, `.iter`, `std::iter::repeat_n`
- **Called by:** `reflow_body` (markdown/diff.rs:556)

#### `reflow_body::{fn owner}`  (L565-571)
`fn owner(tokens: &[&str]) -> Vec<usize>`

- **Does:** _no doc comment_
- **Calls (crate):** `weight` (markdown/diff.rs:610) L569
- **Calls (std/external/unresolved):** `.collect`, `.enumerate`, `.flat_map`, `.iter`, `std::iter::repeat_n`
- **Called by:** `reflow_body` (markdown/diff.rs:556)

#### `weight`  (L610-616)
`fn weight(token: &str) -> i32`

- **Does:** A word is worth more than the space or line break it moved across.
- **Calls (std/external/unresolved):** `.any`, `.chars`
- **Called by:** `pair_gap` (comparer/cross_para.rs:972) (?); `unit_lcs` (comparer/cross_para.rs:1384) (?); `char_weighted_lcs` (comparer/cross_para.rs:1996) (?); `reflow_body::{fn weigh}` (markdown/diff.rs:559); `reflow_body::{fn owner}` (markdown/diff.rs:565)

#### `plain_paragraph`  (L618-626)
`fn plain_paragraph(lines: &[&str]) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `marker_len` (markdown/diff.rs:183) L622; `continues` (markdown/diff.rs:233) L625
- **Calls (std/external/unresolved):** `.all`, `.first`, `.is_empty`, `.iter`, `.starts_with`, `.trim`
- **Called by:** `reflow` (markdown/diff.rs:523)

#### `pair`  (L630-655)
`fn pair<'a>(olds: &[&'a str], news: &[&'a str]) -> Vec<Entry<'a>>`

- **Does:** Aligns the lines of a hunk by content: the similar lines with the same marker shapes pair up, in order; the rest are marked block by block.
- **Calls (crate):** `pair_by_similarity` (markdown/diff.rs:664) L638
- **Calls (std/external/unresolved):** `.chain`, `.extend`, `.into_iter`, `.iter`, `.len`, `.map`, `.push`, `Entry::Only`, `Entry::Pair`, `Vec::new`, `std::iter::once`
- **Macros:** `vec!`
- **Called by:** `hunk` (markdown/diff.rs:477)

#### `pair_by_similarity`  (L664-734)
`fn pair_by_similarity(olds: &[&str], news: &[&str]) -> Vec<(usize, usize)>`

- **Does:** The similar-line pairs of a hunk, in order.
- **Calls (crate):** `body_words` (markdown/diff.rs:836) L669; `intern` (markdown/diff.rs:829) L671; `marker_shape` (markdown/diff.rs:840) L699; `shape` (ambiguous: `coverage_round_three_tests::coverage_deep_layout_tests::shape` (convert/mod.rs:52371); `chart_cache_and_diagram_threshold_contract_tests::shape` (convert/mod.rs:63346); `tests::shape` (markdown/math.rs:1956)) L700; `dice` (markdown/diff.rs:738) L727; `weighted_pairs` (markdown/diff.rs:775) L733
- **Calls (std/external/unresolved):** `.chain`, `.clone`, `.collect`, `.copied`, `.dedup`, `.entry`, `.enumerate`, `.filter_map`, `.first`, `.flat_map`, `.get`, `.into_iter`, `.iter`, `.len`, `.map`, `.max`, `.or_default`, `.push`, `.sort_unstable`, `.split_at`, `.take_while`, `.to_vec`, `.truncate`, `Vec::new`, `Vec::with_capacity`, `sorted`, `std::collections::HashMap::new`
- **Called by:** `pair` (markdown/diff.rs:630)

#### `dice`  (L738-769)
`fn dice<T: Ord>(a: &[T], b: &[T]) -> f64`

- **Does:** Dice coefficient of two sorted id (or word) multisets: twice the size of their intersection, counting repeats, over the sizes summed.
- **Calls (std/external/unresolved):** `.cmp`, `.get`, `.is_empty`, `.len`, `.min`, `Some`
- **Called by:** `pair_by_similarity` (markdown/diff.rs:664)

#### `weighted_pairs`  (L775-827)
`fn weighted_pairs(pairs: &[(usize, usize, f64)], new_len: usize) -> Vec<(usize, usize)>`

- **Does:** The chain of `pairs` (old index, new index, score) with the greatest score sum whose indices strictly increase.
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.len`, `.map_or`, `.position`, `.push`, `.reverse`, `.sort_by_key`, `.wrapping_neg`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `pair_by_similarity` (markdown/diff.rs:664)

#### `intern`  (L829-832)
`fn intern<'a>(ids: &mut std::collections::HashMap<&'a str, u32>, word: &'a str) -> u32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.entry`, `.len`, `.or_insert`
- **Called by:** `emit` (convert/pdf.rs:380) (?); `pair_by_similarity` (markdown/diff.rs:664)

#### `body_words`  (L836-838)
`fn body_words(line: &str) -> Vec<&str>`

- **Does:** Words of a line after its block marker.
- **Calls (crate):** `words` (markdown/diff.rs:879) L837; `marker_len` (markdown/diff.rs:183) L837
- **Called by:** `pair_by_similarity` (markdown/diff.rs:664)

#### `marker_shape`  (L840-853)
`fn marker_shape(line: &str) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `marker_len` (markdown/diff.rs:183) L841
- **Calls (std/external/unresolved):** `.chars`, `.ends_with`, `.is_ascii_digit`, `.len`, `.push`, `String::with_capacity`
- **Called by:** `pair_by_similarity` (markdown/diff.rs:664)

#### `similarity`  (L859-877, pub(super))
`pub(super) fn similarity(old: &str, new: &str) -> f64`

- **Does:** The share of words two lines have in common (0 to 1): twice the longest common subsequence of their words over the sizes summed.
- **Calls (crate):** `words` (markdown/diff.rs:879) L860
- **Calls (std/external/unresolved):** `.as_tag_tuple`, `.filter`, `.is_empty`, `.iter`, `.len`, `.map`, `.min`, `.sum`, `capture_diff_slices`
- **Macros:** `matches!`
- **Called by:** `reflow` (markdown/diff.rs:523); `paired` (markdown/diff.rs:892); `align` (markdown/patch.rs:399)

#### `words`  (L879-884)
`fn words(line: &str) -> Vec<&str>`

- **Does:** _no doc comment_
- **Calls (crate):** `tokens` (ambiguous: `lexical_pairing_contract_tests::tokens` (comparer/cross_para.rs:2144); `tokens` (markdown/math.rs:640)) L880
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.collect`, `.filter`, `.into_iter`
- **Called by:** `body_words` (markdown/diff.rs:836); `similarity` (markdown/diff.rs:859)

#### `paired`  (L892-906)
`fn paired(out: &mut Output, old: &str, new: &str)`

- **Does:** A line in both versions: its markers, then its text word by word.
- **Calls (crate):** `marker_len` (markdown/diff.rs:183) L893; `Output::line` (markdown/diff.rs:389) L895,905; `change` (markdown/diff.rs:909) L895,901; `escape` (markdown/diff.rs:294) L899; `similarity` (markdown/diff.rs:859) L900; `words_diff` (markdown/diff.rs:920) L903
- **Calls (std/external/unresolved):** `.into_owned`, `.len`, `.push_str`
- **Called by:** `hunk` (markdown/diff.rs:477)

#### `change`  (L909-916)
`fn change(old: &str, new: &str) -> String`

- **Does:** A replacement, deletion or insertion of `old` by `new`.
- **Calls (crate):** `escape` (markdown/diff.rs:294) L912,913,914
- **Calls (std/external/unresolved):** `.is_empty`, `String::new`
- **Macros:** `format!`
- **Called by:** `coverage_round_three_tests::coverage_deep_layout_tests::wrapping_keeps_every_distinct_painted_format_across_an_unbroken_word` (convert/mod.rs:52974) (?); `track_properties` (edit/tracked.rs:888) (?); `reflow_body` (markdown/diff.rs:556); `paired` (markdown/diff.rs:892); `words_diff` (markdown/diff.rs:920); `with_style_changes` (markdown/view_styles.rs:45) (?)

#### `words_diff`  (L920-970)
`fn words_diff(old: &str, new: &str) -> String`

- **Does:** `old` against `new` word by word.
- **Calls (crate):** `tokens` (ambiguous: `lexical_pairing_contract_tests::tokens` (comparer/cross_para.rs:2144); `tokens` (markdown/math.rs:640)) L921,922; `widen` (markdown/diff.rs:1182) L960; `escape` (markdown/diff.rs:294) L964; `change` (markdown/diff.rs:909) L966
- **Calls (std/external/unresolved):** `.as_tag_tuple`, `.clone`, `.collect`, `.concat`, `.into_iter`, `.into_owned`, `.is_empty`, `.last_mut`, `.last`, `.len`, `.map`, `.push_str`, `.push`, `.trim`, `Some`, `Vec::new`, `capture_diff_slices`
- **Macros:** `matches!`
- **Called by:** `reflow` (markdown/diff.rs:523); `paired` (markdown/diff.rs:892); `tests::a_rewritten_phrase_is_one_change` (markdown/diff.rs:1397); `tests::a_decomposed_accent_is_part_of_the_word_replaced` (markdown/diff.rs:1407); `tests::a_change_inside_a_number_is_the_whole_number` (markdown/diff.rs:1428); `tests::a_formatting_change_is_the_whole_span` (markdown/diff.rs:1455); `tests::a_changed_link_target_is_the_whole_link` (markdown/diff.rs:1479)

#### `atoms`  (L992-997)
`fn atoms(text: &str) -> Vec<Atom>`

- **Does:** Numbers (`$5,000,000`, `04/20/26`, `3.5%`), the delimiters of emphasis and strikethrough, link syntax (all but its text), and a whole code span, image or equation.
- **Calls (crate):** `scan_atoms` (markdown/diff.rs:1004) L993; `numbers` (markdown/diff.rs:1134) L995
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.into_iter`
- **Called by:** `widen` (markdown/diff.rs:1182)

#### `scan_atoms`  (L1004-1103)
`fn scan_atoms(text: &str, offset: usize, math: bool) -> Vec<Atom>`

- **Does:** The parser's spans of `text`, a slice of a line at `offset`, with equations read (`math`) or not.
- **Calls (crate):** `extend` (markdown/diff.rs:1114) L1038,1061,1069,1076,1083,1088...; `is_atom_end` (markdown/diff.rs:1106) L1040; `opens_amount` (markdown/diff.rs:1127) L1078; `scan_atoms` (markdown/diff.rs:1004) L1087
- **Calls (std/external/unresolved):** `.extend`, `.into_offset_iter`, `.pop`, `.push`, `.unwrap_or`, `Parser::new_ext`, `Vec::new`, `shift`
- **Macros:** `matches!`, `vec!`
- **Called by:** `atoms` (markdown/diff.rs:992); `scan_atoms` (markdown/diff.rs:1004)

#### `is_atom_end`  (L1106-1111)
`fn is_atom_end(end: &TagEnd) -> bool`

- **Does:** Whether `end` closes a span [`atoms`] reads whole.
- **Macros:** `matches!`
- **Called by:** `scan_atoms` (markdown/diff.rs:1004)

#### `extend`  (L1114-1121)
`fn extend(open: &mut [OpenSpan], range: (usize, usize))`

- **Does:** Widens the innermost open span's text to cover `range`.
- **Calls (std/external/unresolved):** `.last_mut`, `.max`, `.min`, `Some`
- **Called by:** `scan_atoms` (markdown/diff.rs:1004)

#### `opens_amount`  (L1127-1130, pub(super))
`pub(super) fn opens_amount(text: &str, at: usize) -> bool`

- **Does:** Whether `at` is the `$` of a currency amount, with a digit right after it: [`numbers`] reads the sign as part of the amount, so this `$` opens no equation — even where the parser pairs it with a later `$` as math (`$250,000 … exceeds **$5,000,000**`).
- **Calls (std/external/unresolved):** `.as_bytes`, `.get`, `.is_some_and`, `Some`
- **Called by:** `opaque_units` (markdown/diff.rs:120); `scan_atoms` (markdown/diff.rs:1004); `inline_math_passes` (markdown/write.rs:226)

#### `numbers`  (L1134-1168)
`fn numbers(text: &str) -> Vec<Atom>`

- **Does:** Runs of digits joined by `,` `.` `/` `:` `-` between digits, with a currency sign before and a `%` after.
- **Calls (std/external/unresolved):** `.char_indices`, `.collect`, `.get`, `.is_ascii_digit`, `.is_some_and`, `.len`, `.map_or`, `.push`, `Vec::new`
- **Macros:** `matches!`, `vec!`
- **Called by:** `atoms` (markdown/diff.rs:992)

#### `touches`  (L1171-1177)
`fn touches(range: (usize, usize), span: (usize, usize)) -> bool`

- **Does:** Whether a change over `range` (empty: a point) touches `span`.
- **Calls:** none
- **Called by:** `widen` (markdown/diff.rs:1182)

#### `widen`  (L1182-1233)
`fn widen( mut segments: Vec<(bool, String, String)>, old: &str, new: &str, ) -> Vec<(bool, String, String)>`

- **Does:** Changes that touch a number, a formatting delimiter or a link's syntax, widened to all of it (in both versions), so `$5,000,000` is replaced whole and `*Buyer*` becoming `Buyer` reads as one change.
- **Calls (crate):** `atoms` (markdown/diff.rs:992) L1187; `touches` (markdown/diff.rs:1171) L1206; `take` (markdown/diff.rs:1239) L1223,1227
- **Calls (std/external/unresolved):** `.any`, `.filter`, `.fold`, `.iter`, `.len`, `.max`, `.min`, `cover`
- **Called by:** `reflow_body` (markdown/diff.rs:556); `words_diff` (markdown/diff.rs:920)

#### `take`  (L1239-1285)
`fn take( segments: &mut Vec<(bool, String, String)>, at: usize, length: usize, into_next: bool, ) -> bool`

- **Does:** Moves `length` bytes (or all) of the segment at `at` into the change next to it: its end into the change after it (`into_next`), or its start into the change before it.
- **Calls (std/external/unresolved):** `.ceil_char_boundary`, `.clone`, `.floor_char_boundary`, `.insert_str`, `.len`, `.push_str`, `.remove`, `.to_string`, `Some`
- **Called by:** `Builder::props` (convert/altchunk.rs:299) (?); `widen` (markdown/diff.rs:1182)

#### `group`  (L1298-1365)
`fn group(out: &mut Output, side: Side, lines: &[&str])`

- **Does:** Lines only in one version.
- **Calls (crate):** `Side::delimiters` (markdown/diff.rs:463) L1302; `open_paragraph` (markdown/diff.rs:240) L1307; `marker_len` (markdown/diff.rs:183) L1307; `continues` (markdown/diff.rs:233) L1307,1309; `escape` (markdown/diff.rs:294) L1310,1353,1355; `Output::append` (markdown/diff.rs:402) L1311,1340; `table_row` (markdown/diff.rs:249) L1319; `Output::line` (markdown/diff.rs:389) L1321,1360; `table_row_change` (markdown/diff.rs:255) L1321
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.count`, `.enumerate`, `.into_owned`, `.is_empty`, `.iter`, `.join`, `.len`, `.map`, `.push_str`, `.take_while`, `.take`, `.trim`, `Some`, `String::new`
- **Macros:** `format!`
- **Called by:** `hunk` (markdown/diff.rs:477)

## Test functions (39)

- `tests::markers_are_found` (L1373): _no doc comment_
- `tests::a_rewritten_phrase_is_one_change` (L1397): _no doc comment_
- `tests::a_decomposed_accent_is_part_of_the_word_replaced` (L1407): _no doc comment_
- `tests::a_change_inside_a_number_is_the_whole_number` (L1428): _no doc comment_
- `tests::a_formatting_change_is_the_whole_span` (L1455): _no doc comment_
- `tests::a_changed_link_target_is_the_whole_link` (L1479): _no doc comment_
- `tests::delimiters_in_the_text_are_escaped` (L1495): _no doc comment_
- `tests::identical_documents_come_back_unchanged` (L1504): _no doc comment_
- `tests::blocks_added_or_removed_keep_their_markers_outside` (L1511): _no doc comment_
- `tests::a_line_added_inside_a_paragraph_takes_its_line_break` (L1527): _no doc comment_
- `tests::a_changed_marker_replaces_the_block` (L1536): _no doc comment_
- `tests::a_table_row_changes_cell_by_cell` (L1561): _no doc comment_
- `tests::unrelated_lines_are_replaced_whole` (L1573): _no doc comment_
- `tests::round_trip` (L1580): _no doc comment_
- `tests::a_changed_code_span_is_wrapped_whole` (L1595): _no doc comment_
- `tests::a_changed_image_is_wrapped_whole` (L1610): _no doc comment_
- `tests::a_changed_equation_is_wrapped_whole` (L1628): _no doc comment_
- `tests::escape_counts_backslashes_and_skips_code` (L1647): _no doc comment_
- `tests::twenty_one_changed_lines_and_an_insertion_pair_by_content` (L1662): _no doc comment_
- `tests::a_large_changed_hunk_still_pairs_similar_lines` (L1689): _no doc comment_
- `tests::list_numbers_do_not_skew_pairing` (L1707): _no doc comment_
- `tests::a_list_item_turned_into_a_paragraph_is_not_glued` (L1718): _no doc comment_
- `tests::wrap_words` (L1730): Words of `text` wrapped to `width` columns, the way a hard wrap joins lines back with a space.
- `tests::one_word_added_to_a_hard_wrapped_paragraph_is_one_change` (L1750): _no doc comment_
- `tests::a_changed_list_number_is_marked` (L1774): _no doc comment_
- `tests::marker_shapes_change_whole` (L1782): _no doc comment_
- `tests::blank_lines_added_and_removed_are_marked` (L1793): _no doc comment_
- `tests::the_trailing_newline_is_part_of_the_change` (L1804): _no doc comment_
- `tests::a_deleted_table_row_takes_its_line_with_it` (L1812): _no doc comment_
- `tests::a_code_span_across_a_line_break_is_one_atom` (L1827): _no doc comment_
- `tests::a_multiline_display_equation_is_wrapped_whole` (L1834): _no doc comment_
- `tests::identical_documents_with_verbatim_delimiters_come_back_unchanged` (L1845): _no doc comment_
- `tests::verbatim_constructs_change_whole` (L1857): _no doc comment_
- `tests::escape_skips_the_readers_verbatim_ranges` (L1869): _no doc comment_
- `tests::accept_and_reject_rebuild_the_documents_exactly` (L1886): _no doc comment_
- `tests::escape_is_linear_on_backtick_runs` (L1936): _no doc comment_
- `tests::a_large_rewrap_stays_fast` (L1950): _no doc comment_
- `tests::long_lines_pair_in_linear_time` (L1978): _no doc comment_
- `tests::just_over_the_old_bound_pairs_by_content` (L1999): _no doc comment_
