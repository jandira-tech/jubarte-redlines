# Functions and methods in `src/markdown/diff.rs`

46 non-test, 39 test. Part of the [symbol manifest](../README.md).

#### `diff_markdown`  (L86-107, pub)
`pub fn diff_markdown(old: &str, new: &str) -> String`

- **Does:** The two text snapshots, `old` and `new`, as one Markdown document whose CriticMarkup holds the changes: accepting them all gives `new`, rejecting them all gives `old`.
- **Calls (crate):** `lines` (markdown/diff.rs:109) L89,90; `Output::line` (markdown/diff.rs:380) L98; `escape` (markdown/diff.rs:294) L98; `hunk` (markdown/diff.rs:468) L102; `Output::finish` (markdown/diff.rs:399) L106
- **Calls (std/external/unresolved):** `.as_tag_tuple`, `.ends_with`, `.replace`, `Output::default`, `capture_diff_slices`
- **Called by:** `tests::a_decomposed_accent_is_part_of_the_word_replaced` (markdown/diff.rs:1398); `tests::delimiters_in_the_text_are_escaped` (markdown/diff.rs:1486); `tests::identical_documents_come_back_unchanged` (markdown/diff.rs:1495); `tests::blocks_added_or_removed_keep_their_markers_outside` (markdown/diff.rs:1502); `tests::a_line_added_inside_a_paragraph_takes_its_line_break` (markdown/diff.rs:1518); `tests::a_changed_marker_replaces_the_block` (markdown/diff.rs:1527); `tests::a_table_row_changes_cell_by_cell` (markdown/diff.rs:1552); `tests::unrelated_lines_are_replaced_whole` (markdown/diff.rs:1564); `tests::round_trip` (markdown/diff.rs:1571); `tests::a_changed_code_span_is_wrapped_whole` (markdown/diff.rs:1586); `tests::a_changed_image_is_wrapped_whole` (markdown/diff.rs:1601); `tests::a_changed_equation_is_wrapped_whole` (markdown/diff.rs:1619); +16 more

#### `lines`  (L109-115)
`fn lines(text: &str) -> Vec<&str>`

- **Does:** _no doc comment_
- **Calls (crate):** `opaque_units` (markdown/diff.rs:120) L114
- **Calls (std/external/unresolved):** `.is_empty`, `.strip_suffix`, `.unwrap_or`, `Vec::new`
- **Called by:** `diff_markdown` (markdown/diff.rs:86); `tests::long_lines_pair_in_linear_time` (markdown/diff.rs:1969); `patch_markdown` (markdown/unified.rs:222)

#### `opaque_units`  (L120-178)
`fn opaque_units(text: &str) -> Vec<&str>`

- **Does:** The physical lines of `text`, except that a code span, equation, image, autolink or HTML comment that spans a line break keeps its lines in one unit: [`atoms`] then reads the whole construct, not a line of it.
- **Calls (crate):** `opens_amount` (markdown/diff.rs:1118) L136; `end_of` (markdown/write.rs:5508) (by name) L167,170,174
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.extend`, `.filter`, `.get`, `.into_offset_iter`, `.last_mut`, `.len`, `.map`, `.match_indices`, `.max`, `.partition_point`, `.push`, `.sort_unstable`, `Parser::new_ext`, `Vec::new`, `line_of`
- **Macros:** `matches!`, `vec!`
- **Called by:** `lines` (markdown/diff.rs:109)

#### `marker_len`  (L183-229, pub(super))
`pub(super) fn marker_len(line: &str) -> usize`

- **Does:** The length of a line's block markers: indentation, `>` quote markers, a list marker with its task box, or a heading's `#`s.
- **Calls (std/external/unresolved):** `.as_bytes`, `.bytes`, `.contains`, `.count`, `.find`, `.first`, `.get`, `.iter`, `.len`, `.starts_with`, `.take_while`, `Some`, `usize::from`
- **Macros:** `matches!`
- **Called by:** `continues` (markdown/diff.rs:233); `plain_paragraph` (markdown/diff.rs:609); `body_words` (markdown/diff.rs:827); `marker_shape` (markdown/diff.rs:831); `paired` (markdown/diff.rs:883); `group` (markdown/diff.rs:1289); `tests::markers_are_found` (markdown/diff.rs:1364); `plain` (markdown/unified.rs:482); `visible` (markdown/unified.rs:920); `starts_block` (markdown/unified.rs:1236); `blocky` (markdown/write.rs:226); `distribute` (markdown/write.rs:242)

#### `continues`  (L233-236, pub(super))
`pub(super) fn continues(line: &str) -> bool`

- **Does:** Whether a line continues the paragraph of a non-blank line before it: it has text and no block marker, and is not a table row or code fence.
- **Calls (crate):** `marker_len` (markdown/diff.rs:183) L235; `table_row` (markdown/diff.rs:249) L235
- **Calls (std/external/unresolved):** `.is_empty`, `.starts_with`, `.trim_start`
- **Called by:** `plain_paragraph` (markdown/diff.rs:609); `group` (markdown/diff.rs:1289); `blocks` (markdown/unified.rs:851); `distribute` (markdown/write.rs:242) (?)

#### `open_paragraph`  (L240-246, pub(super))
`pub(super) fn open_paragraph(line: &str) -> bool`

- **Does:** Whether text may continue after a line: not blank, not a heading, a table row or a code fence.
- **Calls (crate):** `table_row` (markdown/diff.rs:249) L245
- **Calls (std/external/unresolved):** `.is_empty`, `.starts_with`, `.trim_start`
- **Called by:** `group` (markdown/diff.rs:1289); `blocks` (markdown/unified.rs:851)

#### `table_row`  (L249-251, pub(super))
`pub(super) fn table_row(line: &str) -> bool`

- **Does:** A table row: the line starts with a pipe.
- **Calls (std/external/unresolved):** `.starts_with`, `.trim_start`
- **Called by:** `continues` (markdown/diff.rs:233); `open_paragraph` (markdown/diff.rs:240); `group` (markdown/diff.rs:1289); `visible` (markdown/unified.rs:920); `wrap` (markdown/unified.rs:1093); `blocky` (markdown/write.rs:226); `distribute` (markdown/write.rs:242)

#### `table_row_change`  (L255-285, pub(super))
`pub(super) fn table_row_change(line: &str, open: &str, close: &str) -> String`

- **Does:** A table row added or removed whole: each cell's text is one change, so the row keeps its cells (a table drops text outside them).
- **Calls (crate):** `escape` (markdown/diff.rs:294) L268; `flush` (markdown/view_stories.rs:755) (by name) L276,283
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.clear`, `.is_empty`, `.len`, `.push_str`, `.push`, `.trim_end`, `.trim_start`, `.trim`, `String::new`
- **Macros:** `matches!`
- **Called by:** `group` (markdown/diff.rs:1289); `tests::a_table_row_changes_cell_by_cell` (markdown/diff.rs:1552); `distribute` (markdown/write.rs:242)

#### `escape`  (L294-311)
`fn escape(text: &str) -> Cow<'_, str>`

- **Does:** CriticMarkup delimiters in text, escaped so they stay text.
- **Calls (crate):** `text_has_delimiter` (markdown/diff.rs:330) L295; `escape_points` (markdown/diff.rs:335) L298
- **Calls (std/external/unresolved):** `.is_empty`, `.len`, `.push_str`, `.push`, `Cow::Borrowed`, `Cow::Owned`, `String::with_capacity`
- **Called by:** `diff_markdown` (markdown/diff.rs:86); `table_row_change` (markdown/diff.rs:255); `reflow_body` (markdown/diff.rs:547); `paired` (markdown/diff.rs:883); `change` (markdown/diff.rs:900); `words_diff` (markdown/diff.rs:911); `group` (markdown/diff.rs:1289); `tests::delimiters_in_the_text_are_escaped` (markdown/diff.rs:1486); `tests::escape_counts_backslashes_and_skips_code` (markdown/diff.rs:1638); `tests::escape_skips_the_readers_verbatim_ranges` (markdown/diff.rs:1860); `tests::escape_is_linear_on_backtick_runs` (markdown/diff.rs:1927)

#### `text_has_delimiter`  (L330-332)
`fn text_has_delimiter(text: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.iter`
- **Called by:** `escape` (markdown/diff.rs:294)

#### `escape_points`  (L335-360)
`fn escape_points(text: &str) -> Vec<usize>`

- **Does:** Byte positions where a backslash must be inserted.
- **Calls (crate):** `verbatim_ranges` (markdown/critic.rs:1621) L336; `escaped_at` (markdown/diff.rs:363) L351
- **Calls (std/external/unresolved):** `.as_bytes`, `.dedup`, `.get`, `.is_some_and`, `.len`, `.push`, `.sort_unstable`, `.starts_with`, `Vec::new`
- **Called by:** `escape` (markdown/diff.rs:294)

#### `escaped_at`  (L363-365)
`fn escaped_at(text: &str, at: usize) -> bool`

- **Does:** Whether `at` follows an odd number of backslashes.
- **Calls (std/external/unresolved):** `.bytes`, `.count`, `.rev`, `.take_while`
- **Called by:** `escape_points` (markdown/diff.rs:335)

#### `Output::line`  (L380-390)
`fn line(&mut self, line: &str)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.push_str`, `.push`, `.take`, `.to_string`
- **Called by:** `diff_markdown` (markdown/diff.rs:86); `hunk` (markdown/diff.rs:468); `paired` (markdown/diff.rs:883); `group` (markdown/diff.rs:1289)

#### `Output::append`  (L393-397)
`fn append(&mut self, text: &str)`

- **Does:** Appends to the last line without a line break.
- **Calls (std/external/unresolved):** `.push_str`
- **Called by:** `stamp_confetti_then_replace` (comparer/lcs.rs:1541) (?); `walk_container` (convert/mod.rs:8906) (?); `table_block` (convert/mod.rs:13059) (?); `Layout::wrap_hanging_or_first` (convert/mod.rs:25723) (?); `Layout::wrap_para_runs` (convert/mod.rs:25969) (?); `group` (markdown/diff.rs:1289)

#### `Output::finish`  (L399-422)
`fn finish(mut self, old_trailing: bool, new_trailing: bool) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Output::add_trailing` (markdown/diff.rs:427) L419
- **Calls (std/external/unresolved):** `.push_str`, `.push`, `.take`
- **Called by:** `diff_markdown` (markdown/diff.rs:86)

#### `Output::add_trailing`  (L427-444)
`fn add_trailing(&mut self, inserted: bool)`

- **Does:** The final line break changed sides.
- **Calls (std/external/unresolved):** `.ends_with`, `.insert`, `.len`, `.map_or`, `.push_str`, `.rfind`
- **Macros:** `format!`
- **Called by:** `Output::finish` (markdown/diff.rs:399)

#### `Side::delimiters`  (L454-459)
`fn delimiters(self) -> (&'static str, &'static str)`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `group` (markdown/diff.rs:1289); `render` (markdown/from_docx/critic.rs:836) (?); `splice` (markdown/from_docx/critic.rs:1197) (?); `whole_change` (markdown/from_docx/critic.rs:1369) (?)

#### `hunk`  (L468-505)
`fn hunk(out: &mut Output, olds: &[&str], news: &[&str])`

- **Does:** _no doc comment_
- **Calls (crate):** `reflow` (markdown/diff.rs:514) L469; `Output::line` (markdown/diff.rs:380) L471; `pair` (markdown/diff.rs:621) L475; `paired` (markdown/diff.rs:883) L480; `group` (markdown/diff.rs:1289) L499,500
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.iter`, `.len`, `.map_or`, `.position`, `.split`, `Entry::Pair`, `Some`, `only`
- **Macros:** `matches!`
- **Called by:** `diff_markdown` (markdown/diff.rs:86)

#### `reflow`  (L514-539)
`fn reflow(olds: &[&str], news: &[&str]) -> Option<String>`

- **Does:** A hard-wrapped paragraph whose lines all moved, as one word diff.
- **Calls (crate):** `plain_paragraph` (markdown/diff.rs:609) L515; `similarity` (markdown/diff.rs:850) L524; `words_diff` (markdown/diff.rs:911) L527; `ChangeFilter::matches` (changes.rs:177) (by name) L529; `reflow_body` (markdown/diff.rs:547) L538
- **Calls (std/external/unresolved):** `.count`, `.join`, `.len`, `.replace`, `Some`
- **Called by:** `hunk` (markdown/diff.rs:468)

#### `reflow_body`  (L547-598)
`fn reflow_body(old: &str, new: &str) -> String`

- **Does:** Word diff of a reflowed paragraph.
- **Calls (crate):** `tokens` (ambiguous: `lexical_pairing_contract_tests::tokens` (comparer/cross_para.rs:2144); `tokens` (markdown/math.rs:640)) L548,549; `reflow_body::{fn weigh}` (markdown/diff.rs:550) L563; `reflow_body::{fn owner}` (markdown/diff.rs:556) L564; `owned` (ambiguous, 4 candidates) L575,576; `widen` (markdown/diff.rs:1173) L588; `escape` (markdown/diff.rs:294) L592; `change` (markdown/diff.rs:900) L594
- **Calls (std/external/unresolved):** `.as_tag_tuple`, `.checked_sub`, `.collect`, `.concat`, `.into_iter`, `.into_owned`, `.last_mut`, `.map`, `.push_str`, `.push`, `Vec::new`, `capture_diff_slices`
- **Called by:** `reflow` (markdown/diff.rs:514)

#### `reflow_body::{fn weigh}`  (L550-555)
`fn weigh<'a>(tokens: &[&'a str]) -> Vec<&'a str>`

- **Does:** _no doc comment_
- **Calls (crate):** `weight` (markdown/diff.rs:601) L553
- **Calls (std/external/unresolved):** `.collect`, `.flat_map`, `.iter`, `std::iter::repeat_n`
- **Called by:** `reflow_body` (markdown/diff.rs:547)

#### `reflow_body::{fn owner}`  (L556-562)
`fn owner(tokens: &[&str]) -> Vec<usize>`

- **Does:** _no doc comment_
- **Calls (crate):** `weight` (markdown/diff.rs:601) L560
- **Calls (std/external/unresolved):** `.collect`, `.enumerate`, `.flat_map`, `.iter`, `std::iter::repeat_n`
- **Called by:** `reflow_body` (markdown/diff.rs:547)

#### `weight`  (L601-607)
`fn weight(token: &str) -> i32`

- **Does:** A word is worth more than the space or line break it moved across.
- **Calls (std/external/unresolved):** `.any`, `.chars`
- **Called by:** `pair_gap` (comparer/cross_para.rs:972) (?); `unit_lcs` (comparer/cross_para.rs:1384) (?); `char_weighted_lcs` (comparer/cross_para.rs:1996) (?); `reflow_body::{fn weigh}` (markdown/diff.rs:550); `reflow_body::{fn owner}` (markdown/diff.rs:556)

#### `plain_paragraph`  (L609-617)
`fn plain_paragraph(lines: &[&str]) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `marker_len` (markdown/diff.rs:183) L613; `continues` (markdown/diff.rs:233) L616
- **Calls (std/external/unresolved):** `.all`, `.first`, `.is_empty`, `.iter`, `.starts_with`, `.trim`
- **Called by:** `reflow` (markdown/diff.rs:514)

#### `pair`  (L621-646)
`fn pair<'a>(olds: &[&'a str], news: &[&'a str]) -> Vec<Entry<'a>>`

- **Does:** Aligns the lines of a hunk by content: the similar lines with the same marker shapes pair up, in order; the rest are marked block by block.
- **Calls (crate):** `pair_by_similarity` (markdown/diff.rs:655) L629
- **Calls (std/external/unresolved):** `.chain`, `.extend`, `.into_iter`, `.iter`, `.len`, `.map`, `.push`, `Entry::Only`, `Entry::Pair`, `Vec::new`, `std::iter::once`
- **Macros:** `vec!`
- **Called by:** `vml_line_image` (convert/mod.rs:18609) (?); `vml_group_box` (convert/mod.rs:19263) (?); `Fib::parse` (legacy_doc.rs:895) (?); `hunk` (markdown/diff.rs:468); `render_emphasis` (markdown/from_docx/ooxml.rs:705) (?)

#### `pair_by_similarity`  (L655-725)
`fn pair_by_similarity(olds: &[&str], news: &[&str]) -> Vec<(usize, usize)>`

- **Does:** The similar-line pairs of a hunk, in order.
- **Calls (crate):** `body_words` (markdown/diff.rs:827) L660; `intern` (markdown/diff.rs:820) L662; `marker_shape` (markdown/diff.rs:831) L690; `shape` (ambiguous: `coverage_round_three_tests::coverage_deep_layout_tests::shape` (convert/mod.rs:52335); `chart_cache_and_diagram_threshold_contract_tests::shape` (convert/mod.rs:63310); `tests::shape` (markdown/math.rs:1956)) L691; `dice` (markdown/diff.rs:729) L718; `weighted_pairs` (markdown/diff.rs:766) L724
- **Calls (std/external/unresolved):** `.chain`, `.clone`, `.collect`, `.copied`, `.dedup`, `.entry`, `.enumerate`, `.filter_map`, `.first`, `.flat_map`, `.get`, `.into_iter`, `.iter`, `.len`, `.map`, `.max`, `.or_default`, `.push`, `.sort_unstable`, `.split_at`, `.take_while`, `.to_vec`, `.truncate`, `Vec::new`, `Vec::with_capacity`, `sorted`, `std::collections::HashMap::new`
- **Called by:** `pair` (markdown/diff.rs:621)

#### `dice`  (L729-760)
`fn dice<T: Ord>(a: &[T], b: &[T]) -> f64`

- **Does:** Dice coefficient of two sorted id (or word) multisets: twice the size of their intersection, counting repeats, over the sizes summed.
- **Calls (std/external/unresolved):** `.cmp`, `.get`, `.is_empty`, `.len`, `.min`, `Some`
- **Called by:** `pair_by_similarity` (markdown/diff.rs:655)

#### `weighted_pairs`  (L766-818)
`fn weighted_pairs(pairs: &[(usize, usize, f64)], new_len: usize) -> Vec<(usize, usize)>`

- **Does:** The chain of `pairs` (old index, new index, score) with the greatest score sum whose indices strictly increase.
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.len`, `.map_or`, `.position`, `.push`, `.reverse`, `.sort_by_key`, `.wrapping_neg`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `pair_by_similarity` (markdown/diff.rs:655)

#### `intern`  (L820-823)
`fn intern<'a>(ids: &mut std::collections::HashMap<&'a str, u32>, word: &'a str) -> u32`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.entry`, `.len`, `.or_insert`
- **Called by:** `emit` (convert/pdf.rs:380) (?); `pair_by_similarity` (markdown/diff.rs:655)

#### `body_words`  (L827-829)
`fn body_words(line: &str) -> Vec<&str>`

- **Does:** Words of a line after its block marker.
- **Calls (crate):** `words` (markdown/diff.rs:870) L828; `marker_len` (markdown/diff.rs:183) L828
- **Called by:** `pair_by_similarity` (markdown/diff.rs:655)

#### `marker_shape`  (L831-844)
`fn marker_shape(line: &str) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `marker_len` (markdown/diff.rs:183) L832
- **Calls (std/external/unresolved):** `.chars`, `.ends_with`, `.is_ascii_digit`, `.len`, `.push`, `String::with_capacity`
- **Called by:** `pair_by_similarity` (markdown/diff.rs:655)

#### `similarity`  (L850-868, pub(super))
`pub(super) fn similarity(old: &str, new: &str) -> f64`

- **Does:** The share of words two lines have in common (0 to 1): twice the longest common subsequence of their words over the sizes summed.
- **Calls (crate):** `words` (markdown/diff.rs:870) L851
- **Calls (std/external/unresolved):** `.as_tag_tuple`, `.filter`, `.is_empty`, `.iter`, `.len`, `.map`, `.min`, `.sum`, `capture_diff_slices`
- **Macros:** `matches!`
- **Called by:** `reflow` (markdown/diff.rs:514); `paired` (markdown/diff.rs:883); `align` (markdown/patch.rs:393)

#### `words`  (L870-875)
`fn words(line: &str) -> Vec<&str>`

- **Does:** _no doc comment_
- **Calls (crate):** `tokens` (ambiguous: `lexical_pairing_contract_tests::tokens` (comparer/cross_para.rs:2144); `tokens` (markdown/math.rs:640)) L871
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.collect`, `.filter`, `.into_iter`
- **Called by:** `body_words` (markdown/diff.rs:827); `similarity` (markdown/diff.rs:850)

#### `paired`  (L883-897)
`fn paired(out: &mut Output, old: &str, new: &str)`

- **Does:** A line in both versions: its markers, then its text word by word.
- **Calls (crate):** `marker_len` (markdown/diff.rs:183) L884; `Output::line` (markdown/diff.rs:380) L886,896; `change` (markdown/diff.rs:900) L886,892; `escape` (markdown/diff.rs:294) L890; `similarity` (markdown/diff.rs:850) L891; `words_diff` (markdown/diff.rs:911) L894
- **Calls (std/external/unresolved):** `.into_owned`, `.len`, `.push_str`
- **Called by:** `hunk` (markdown/diff.rs:468)

#### `change`  (L900-907)
`fn change(old: &str, new: &str) -> String`

- **Does:** A replacement, deletion or insertion of `old` by `new`.
- **Calls (crate):** `escape` (markdown/diff.rs:294) L903,904,905
- **Calls (std/external/unresolved):** `.is_empty`, `String::new`
- **Macros:** `format!`
- **Called by:** `coverage_round_three_tests::coverage_deep_layout_tests::wrapping_keeps_every_distinct_painted_format_across_an_unbroken_word` (convert/mod.rs:52938) (?); `track_properties` (edit/tracked.rs:633) (?); `reflow_body` (markdown/diff.rs:547); `paired` (markdown/diff.rs:883); `words_diff` (markdown/diff.rs:911); `with_style_changes` (markdown/view_styles.rs:40) (?)

#### `words_diff`  (L911-961)
`fn words_diff(old: &str, new: &str) -> String`

- **Does:** `old` against `new` word by word.
- **Calls (crate):** `tokens` (ambiguous: `lexical_pairing_contract_tests::tokens` (comparer/cross_para.rs:2144); `tokens` (markdown/math.rs:640)) L912,913; `widen` (markdown/diff.rs:1173) L951; `escape` (markdown/diff.rs:294) L955; `change` (markdown/diff.rs:900) L957
- **Calls (std/external/unresolved):** `.as_tag_tuple`, `.clone`, `.collect`, `.concat`, `.into_iter`, `.into_owned`, `.is_empty`, `.last_mut`, `.last`, `.len`, `.map`, `.push_str`, `.push`, `.trim`, `Some`, `Vec::new`, `capture_diff_slices`
- **Macros:** `matches!`
- **Called by:** `reflow` (markdown/diff.rs:514); `paired` (markdown/diff.rs:883); `tests::a_rewritten_phrase_is_one_change` (markdown/diff.rs:1388); `tests::a_decomposed_accent_is_part_of_the_word_replaced` (markdown/diff.rs:1398); `tests::a_change_inside_a_number_is_the_whole_number` (markdown/diff.rs:1419); `tests::a_formatting_change_is_the_whole_span` (markdown/diff.rs:1446); `tests::a_changed_link_target_is_the_whole_link` (markdown/diff.rs:1470)

#### `atoms`  (L983-988)
`fn atoms(text: &str) -> Vec<Atom>`

- **Does:** Numbers (`$5,000,000`, `04/20/26`, `3.5%`), the delimiters of emphasis and strikethrough, link syntax (all but its text), and a whole code span, image or equation.
- **Calls (crate):** `scan_atoms` (markdown/diff.rs:995) L984; `numbers` (markdown/diff.rs:1125) L986
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.into_iter`
- **Called by:** `widen` (markdown/diff.rs:1173)

#### `scan_atoms`  (L995-1094)
`fn scan_atoms(text: &str, offset: usize, math: bool) -> Vec<Atom>`

- **Does:** The parser's spans of `text`, a slice of a line at `offset`, with equations read (`math`) or not.
- **Calls (crate):** `extend` (markdown/diff.rs:1105) L1029,1052,1060,1067,1074,1079...; `is_atom_end` (markdown/diff.rs:1097) L1031; `opens_amount` (markdown/diff.rs:1118) L1069; `scan_atoms` (markdown/diff.rs:995) L1078
- **Calls (std/external/unresolved):** `.extend`, `.into_offset_iter`, `.pop`, `.push`, `.unwrap_or`, `Parser::new_ext`, `Vec::new`, `shift`
- **Macros:** `matches!`, `vec!`
- **Called by:** `atoms` (markdown/diff.rs:983); `scan_atoms` (markdown/diff.rs:995)

#### `is_atom_end`  (L1097-1102)
`fn is_atom_end(end: &TagEnd) -> bool`

- **Does:** Whether `end` closes a span [`atoms`] reads whole.
- **Macros:** `matches!`
- **Called by:** `scan_atoms` (markdown/diff.rs:995)

#### `extend`  (L1105-1112)
`fn extend(open: &mut [OpenSpan], range: (usize, usize))`

- **Does:** Widens the innermost open span's text to cover `range`.
- **Calls (std/external/unresolved):** `.last_mut`, `.max`, `.min`, `Some`
- **Called by:** `scan_atoms` (markdown/diff.rs:995)

#### `opens_amount`  (L1118-1121)
`fn opens_amount(text: &str, at: usize) -> bool`

- **Does:** Whether `at` is the `$` of a currency amount, with a digit right after it: [`numbers`] reads the sign as part of the amount, so this `$` opens no equation — even where the parser pairs it with a later `$` as math (`$250,000 … exceeds **$5,000,000**`).
- **Calls (std/external/unresolved):** `.as_bytes`, `.get`, `.is_some_and`, `Some`
- **Called by:** `opaque_units` (markdown/diff.rs:120); `scan_atoms` (markdown/diff.rs:995)

#### `numbers`  (L1125-1159)
`fn numbers(text: &str) -> Vec<Atom>`

- **Does:** Runs of digits joined by `,` `.` `/` `:` `-` between digits, with a currency sign before and a `%` after.
- **Calls (std/external/unresolved):** `.char_indices`, `.collect`, `.get`, `.is_ascii_digit`, `.is_some_and`, `.len`, `.map_or`, `.push`, `Vec::new`
- **Macros:** `matches!`, `vec!`
- **Called by:** `atoms` (markdown/diff.rs:983)

#### `touches`  (L1162-1168)
`fn touches(range: (usize, usize), span: (usize, usize)) -> bool`

- **Does:** Whether a change over `range` (empty: a point) touches `span`.
- **Calls:** none
- **Called by:** `widen` (markdown/diff.rs:1173)

#### `widen`  (L1173-1224)
`fn widen( mut segments: Vec<(bool, String, String)>, old: &str, new: &str, ) -> Vec<(bool, String, String)>`

- **Does:** Changes that touch a number, a formatting delimiter or a link's syntax, widened to all of it (in both versions), so `$5,000,000` is replaced whole and `*Buyer*` becoming `Buyer` reads as one change.
- **Calls (crate):** `atoms` (markdown/diff.rs:983) L1178; `touches` (markdown/diff.rs:1162) L1197; `take` (markdown/diff.rs:1230) L1214,1218
- **Calls (std/external/unresolved):** `.any`, `.filter`, `.fold`, `.iter`, `.len`, `.max`, `.min`, `cover`
- **Called by:** `reflow_body` (markdown/diff.rs:547); `words_diff` (markdown/diff.rs:911)

#### `take`  (L1230-1276)
`fn take( segments: &mut Vec<(bool, String, String)>, at: usize, length: usize, into_next: bool, ) -> bool`

- **Does:** Moves `length` bytes (or all) of the segment at `at` into the change next to it: its end into the change after it (`into_next`), or its start into the change before it.
- **Calls (std/external/unresolved):** `.ceil_char_boundary`, `.clone`, `.floor_char_boundary`, `.insert_str`, `.len`, `.push_str`, `.remove`, `.to_string`, `Some`
- **Called by:** `Builder::props` (convert/altchunk.rs:299) (?); `widen` (markdown/diff.rs:1173)

#### `group`  (L1289-1356)
`fn group(out: &mut Output, side: Side, lines: &[&str])`

- **Does:** Lines only in one version.
- **Calls (crate):** `Side::delimiters` (markdown/diff.rs:454) L1293; `open_paragraph` (markdown/diff.rs:240) L1298; `marker_len` (markdown/diff.rs:183) L1298; `continues` (markdown/diff.rs:233) L1298,1300; `escape` (markdown/diff.rs:294) L1301,1344,1346; `Output::append` (markdown/diff.rs:393) L1302,1331; `table_row` (markdown/diff.rs:249) L1310; `Output::line` (markdown/diff.rs:380) L1312,1351; `table_row_change` (markdown/diff.rs:255) L1312
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.count`, `.enumerate`, `.into_owned`, `.is_empty`, `.iter`, `.join`, `.len`, `.map`, `.push_str`, `.take_while`, `.take`, `.trim`, `Some`, `String::new`
- **Macros:** `format!`
- **Called by:** `hunk` (markdown/diff.rs:468)

## Test functions (39)

- `tests::markers_are_found` (L1364): _no doc comment_
- `tests::a_rewritten_phrase_is_one_change` (L1388): _no doc comment_
- `tests::a_decomposed_accent_is_part_of_the_word_replaced` (L1398): _no doc comment_
- `tests::a_change_inside_a_number_is_the_whole_number` (L1419): _no doc comment_
- `tests::a_formatting_change_is_the_whole_span` (L1446): _no doc comment_
- `tests::a_changed_link_target_is_the_whole_link` (L1470): _no doc comment_
- `tests::delimiters_in_the_text_are_escaped` (L1486): _no doc comment_
- `tests::identical_documents_come_back_unchanged` (L1495): _no doc comment_
- `tests::blocks_added_or_removed_keep_their_markers_outside` (L1502): _no doc comment_
- `tests::a_line_added_inside_a_paragraph_takes_its_line_break` (L1518): _no doc comment_
- `tests::a_changed_marker_replaces_the_block` (L1527): _no doc comment_
- `tests::a_table_row_changes_cell_by_cell` (L1552): _no doc comment_
- `tests::unrelated_lines_are_replaced_whole` (L1564): _no doc comment_
- `tests::round_trip` (L1571): _no doc comment_
- `tests::a_changed_code_span_is_wrapped_whole` (L1586): _no doc comment_
- `tests::a_changed_image_is_wrapped_whole` (L1601): _no doc comment_
- `tests::a_changed_equation_is_wrapped_whole` (L1619): _no doc comment_
- `tests::escape_counts_backslashes_and_skips_code` (L1638): _no doc comment_
- `tests::twenty_one_changed_lines_and_an_insertion_pair_by_content` (L1653): _no doc comment_
- `tests::a_large_changed_hunk_still_pairs_similar_lines` (L1680): _no doc comment_
- `tests::list_numbers_do_not_skew_pairing` (L1698): _no doc comment_
- `tests::a_list_item_turned_into_a_paragraph_is_not_glued` (L1709): _no doc comment_
- `tests::wrap_words` (L1721): Words of `text` wrapped to `width` columns, the way a hard wrap joins lines back with a space.
- `tests::one_word_added_to_a_hard_wrapped_paragraph_is_one_change` (L1741): _no doc comment_
- `tests::a_changed_list_number_is_marked` (L1765): _no doc comment_
- `tests::marker_shapes_change_whole` (L1773): _no doc comment_
- `tests::blank_lines_added_and_removed_are_marked` (L1784): _no doc comment_
- `tests::the_trailing_newline_is_part_of_the_change` (L1795): _no doc comment_
- `tests::a_deleted_table_row_takes_its_line_with_it` (L1803): _no doc comment_
- `tests::a_code_span_across_a_line_break_is_one_atom` (L1818): _no doc comment_
- `tests::a_multiline_display_equation_is_wrapped_whole` (L1825): _no doc comment_
- `tests::identical_documents_with_verbatim_delimiters_come_back_unchanged` (L1836): _no doc comment_
- `tests::verbatim_constructs_change_whole` (L1848): _no doc comment_
- `tests::escape_skips_the_readers_verbatim_ranges` (L1860): _no doc comment_
- `tests::accept_and_reject_rebuild_the_documents_exactly` (L1877): _no doc comment_
- `tests::escape_is_linear_on_backtick_runs` (L1927): _no doc comment_
- `tests::a_large_rewrap_stays_fast` (L1941): _no doc comment_
- `tests::long_lines_pair_in_linear_time` (L1969): _no doc comment_
- `tests::just_over_the_old_bound_pairs_by_content` (L1990): _no doc comment_
