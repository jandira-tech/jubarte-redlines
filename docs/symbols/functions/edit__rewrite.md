# Functions and methods in `src/edit/rewrite.rs`

5 non-test, 8 test. Part of the [symbol manifest](../README.md).

#### `kept`  (L14-16)
`fn kept(c: char) -> bool`

- **Does:** A character that holds a place in the projection but is no run text: a symbol (`w:sym`, U+FFFC).
- **Calls:** none
- **Called by:** `run_flags` (bin/jubarte.rs:1804) (?); `render_best` (comparer/cross_para.rs:689) (?); `coverage_boundary_tests::page_selection_sorts_deduplicates_and_reports_the_largest_invalid_page` (convert/mod.rs:47636) (?); `split_around_kept` (edit/rewrite.rs:117)

#### `units`  (L28-59)
`fn units(text: &str) -> Vec<Unit>`

- **Does:** The old text in units: words and marks as they are, any run of whitespace as one space (a tab or a line break equals a space), symbols left out.
- **Calls (crate):** `word_tokens` (util/words.rs:65) L31
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.last_mut`, `.len`, `.push`, `.to_string`, `Vec::new`
- **Called by:** `source_paragraph_promotion_boundary_tests::cached_distinct_paragraph_owners_disambiguate_group_and_word_gaps` (comparer/moves.rs:1366) (?); `source_paragraph_promotion_boundary_tests::repeated_deleted_prefix_matches_following_distinct_source_owners` (comparer/moves.rs:1510) (?); `rewrite_ranges` (edit/rewrite.rs:72)

#### `breaks`  (L62-64)
`fn breaks(whitespace: &str) -> impl Iterator<Item = char> + '_`

- **Does:** The tabs and line breaks of some whitespace, in order.
- **Calls (std/external/unresolved):** `.chars`, `.filter`
- **Macros:** `matches!`
- **Called by:** `rewrite_ranges` (edit/rewrite.rs:72); `without_layout` (markdown/from_docx/mod.rs:1324) (?)

#### `rewrite_ranges`  (L72-111, pub(super))
`pub(super) fn rewrite_ranges(old: &str, new: &str) -> Vec<(usize, usize, String)>`

- **Does:** Byte ranges of `old` and their replacements that make it read as `new`, in order and without overlaps, word by word.
- **Calls (crate):** `units` (edit/rewrite.rs:28) L73,74; `breaks` (edit/rewrite.rs:62) L84,85; `split_around_kept` (edit/rewrite.rs:117) L87,108
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.as_tag_tuple`, `.checked_sub`, `.collect`, `.eq`, `.extend`, `.get`, `.is_empty`, `.is_some`, `.iter`, `.map_or`, `.map`, `.next`, `.zip`, `Vec::new`, `capture_diff_slices`
- **Called by:** `tests::one_word_is_one_edit` (edit/rewrite.rs:173); `tests::a_decomposed_word_is_replaced_whole_with_its_accent` (edit/rewrite.rs:181); `tests::insertions_at_the_start_and_end` (edit/rewrite.rs:192); `tests::unchanged_text_and_whitespace_of_another_kind_need_nothing` (edit/rewrite.rs:199); `tests::tabs_breaks_and_symbols_stay` (edit/rewrite.rs:207); `tests::new_text_writes_its_tabs_and_breaks` (edit/rewrite.rs:225); `tests::everything_can_go` (edit/rewrite.rs:241); `Transaction::resolve_rewrite` (edit.rs:3466)

#### `split_around_kept`  (L117-157)
`fn split_around_kept( old: &str, start: usize, end: usize, replacement: &str, ) -> Vec<(usize, usize, String)>`

- **Does:** `[start, end)` of `old` with `replacement`, split so that no piece covers a symbol.
- **Calls (crate):** `kept` (edit/rewrite.rs:14) L127
- **Calls (std/external/unresolved):** `.char_indices`, `.collect`, `.concat`, `.filter_map`, `.filter`, `.into_iter`, `.is_empty`, `.len_utf8`, `.len`, `.map`, `.push`, `.split`, `.take`, `.then_some`, `.to_string`, `.unwrap_or_default`, `.zip`, `Some`, `Vec::new`
- **Called by:** `rewrite_ranges` (edit/rewrite.rs:72)

## Test functions (8)

- `tests::apply` (L164): _no doc comment_
- `tests::one_word_is_one_edit` (L173): _no doc comment_
- `tests::a_decomposed_word_is_replaced_whole_with_its_accent` (L181): _no doc comment_
- `tests::insertions_at_the_start_and_end` (L192): _no doc comment_
- `tests::unchanged_text_and_whitespace_of_another_kind_need_nothing` (L199): _no doc comment_
- `tests::tabs_breaks_and_symbols_stay` (L207): _no doc comment_
- `tests::new_text_writes_its_tabs_and_breaks` (L225): _no doc comment_
- `tests::everything_can_go` (L241): _no doc comment_
