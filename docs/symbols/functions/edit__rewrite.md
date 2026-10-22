# Functions and methods in `src/edit/rewrite.rs`

7 non-test, 12 test. Part of the [symbol manifest](../README.md).

#### `kept`  (L18-20)
`fn kept(c: char) -> bool`

- **Does:** A character that holds a place in the projection but is no run text: a symbol (`w:sym`, U+FFFC).
- **Calls:** none
- **Called by:** `run_flags` (bin/jubarte.rs:2511) (?); `render_best` (comparer/cross_para.rs:689) (?); `coverage_boundary_tests::page_selection_sorts_deduplicates_and_reports_the_largest_invalid_page` (convert/mod.rs:47672) (?); `split_around_kept` (edit/rewrite.rs:178)

#### `units`  (L34-62)
`fn units(text: &str, collapse: bool) -> Vec<Unit>`

- **Does:** The old text in units, each keyed as it reads (a curly quote as the straight one, a non-breaking space as a space, a zero-width character as nothing), so a look-alike never differs: words and marks as they are, any run of whitespace as one space when `collapse` (a tab or a line break equals a space), symbols and text...
- **Calls (crate):** `word_tokens` (util/words.rs:65) L37
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.collect`, `.filter_map`, `.is_empty`, `.last_mut`, `.len`, `.push`, `.to_string`, `Vec::new`
- **Called by:** `source_paragraph_promotion_boundary_tests::cached_distinct_paragraph_owners_disambiguate_group_and_word_gaps` (comparer/moves.rs:1366) (?); `source_paragraph_promotion_boundary_tests::repeated_deleted_prefix_matches_following_distinct_source_owners` (comparer/moves.rs:1510) (?); `ranges` (edit/rewrite.rs:88)

#### `breaks`  (L65-67)
`fn breaks(whitespace: &str) -> impl Iterator<Item = char> + '_`

- **Does:** The tabs and line breaks of some whitespace, in order.
- **Calls (std/external/unresolved):** `.chars`, `.filter`
- **Macros:** `matches!`
- **Called by:** `ranges` (edit/rewrite.rs:88); `without_layout` (markdown/from_docx/mod.rs:1408) (?)

#### `rewrite_ranges`  (L77-79, pub(super))
`pub(super) fn rewrite_ranges(old: &str, new: &str) -> Vec<(usize, usize, String)>`

- **Does:** Byte ranges of `old` and their replacements that make it read as `new`, in order and without overlaps, word by word.
- **Calls (crate):** `ranges` (edit/rewrite.rs:88) L78
- **Called by:** `tests::one_word_is_one_edit` (edit/rewrite.rs:234); `tests::a_decomposed_word_is_replaced_whole_with_its_accent` (edit/rewrite.rs:242); `tests::insertions_at_the_start_and_end` (edit/rewrite.rs:253); `tests::unchanged_text_and_whitespace_of_another_kind_need_nothing` (edit/rewrite.rs:260); `tests::tabs_breaks_and_symbols_stay` (edit/rewrite.rs:268); `tests::new_text_writes_its_tabs_and_breaks` (edit/rewrite.rs:286); `tests::look_alikes_read_as_the_plain_character_and_stay` (edit/rewrite.rs:302); `tests::everything_can_go` (edit/rewrite.rs:366); `insertion_replacement` (edit/tracked.rs:987); `Transaction::resolve_rewrite` (edit.rs:4608)

#### `replace_ranges`  (L84-86, pub(super))
`pub(super) fn replace_ranges(old: &str, new: &str) -> Vec<(usize, usize, String)>`

- **Does:** [`rewrite_ranges`] for a `replace`: whitespace compares as written, so a double space or a tab the anchor names is replaced as asked, while a look-alike of the replacement's character still stays.
- **Calls (crate):** `ranges` (edit/rewrite.rs:88) L85
- **Called by:** `tests::replace_ranges_compare_whitespace_as_written` (edit/rewrite.rs:315); `tests::a_joined_replacement_keeps_its_gaps_look_alike` (edit/rewrite.rs:330); `tests::replace_ranges_join_changes_only_whitespace_separates` (edit/rewrite.rs:347); `Transaction::replace_pieces` (edit.rs:4549)

#### `ranges`  (L88-172)
`fn ranges(old: &str, new: &str, collapse: bool) -> Vec<(usize, usize, String)>`

- **Does:** _no doc comment_
- **Calls (crate):** `units` (edit/rewrite.rs:34) L89,90; `breaks` (edit/rewrite.rs:65) L136,137; `split_around_kept` (edit/rewrite.rs:178) L139,169
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.as_slice`, `.as_str`, `.as_tag_tuple`, `.checked_sub`, `.clone`, `.collect`, `.contains`, `.drain`, `.eq`, `.expect`, `.extend`, `.find`, `.get`, `.is_empty`, `.is_some`, `.iter`, `.len`, `.map_or`, `.map`, `.next`, `.push`, `.trim`, `.zip`, `Vec::new`, `capture_diff_slices`
- **Called by:** `rewrite_ranges` (edit/rewrite.rs:77); `replace_ranges` (edit/rewrite.rs:84)

#### `split_around_kept`  (L178-218)
`fn split_around_kept( old: &str, start: usize, end: usize, replacement: &str, ) -> Vec<(usize, usize, String)>`

- **Does:** `[start, end)` of `old` with `replacement`, split so that no piece covers a symbol.
- **Calls (crate):** `kept` (edit/rewrite.rs:18) L188
- **Calls (std/external/unresolved):** `.char_indices`, `.collect`, `.concat`, `.filter_map`, `.filter`, `.into_iter`, `.is_empty`, `.len_utf8`, `.len`, `.map`, `.push`, `.split`, `.take`, `.then_some`, `.to_string`, `.unwrap_or_default`, `.zip`, `Some`, `Vec::new`
- **Called by:** `ranges` (edit/rewrite.rs:88)

## Test functions (12)

- `tests::apply` (L225): _no doc comment_
- `tests::one_word_is_one_edit` (L234): _no doc comment_
- `tests::a_decomposed_word_is_replaced_whole_with_its_accent` (L242): _no doc comment_
- `tests::insertions_at_the_start_and_end` (L253): _no doc comment_
- `tests::unchanged_text_and_whitespace_of_another_kind_need_nothing` (L260): _no doc comment_
- `tests::tabs_breaks_and_symbols_stay` (L268): _no doc comment_
- `tests::new_text_writes_its_tabs_and_breaks` (L286): _no doc comment_
- `tests::look_alikes_read_as_the_plain_character_and_stay` (L302): _no doc comment_
- `tests::replace_ranges_compare_whitespace_as_written` (L315): _no doc comment_
- `tests::a_joined_replacement_keeps_its_gaps_look_alike` (L330): Changes a whitespace gap joins into one keep the gap as the document writes it: a non-breaking space the replacement types as a plain one stays (#1384 review).
- `tests::replace_ranges_join_changes_only_whitespace_separates` (L347): _no doc comment_
- `tests::everything_can_go` (L366): _no doc comment_
