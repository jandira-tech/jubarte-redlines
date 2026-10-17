# Functions and methods in `src/edit/anchor.rs`

8 non-test, 16 test. Part of the [symbol manifest](../README.md).

#### `probe_compare`  (L43-43)
`fn probe_compare()`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `v2_candidates` (edit/anchor.rs:118)

#### `reads_as`  (L46-61)
`fn reads_as(c: char) -> Option<char>`

- **Does:** What `c` reads as, or `None` for a character that shows nothing.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `reading` (edit/anchor.rs:65); `shown` (edit/anchor.rs:95)

#### `reading`  (L65-75)
`fn reading(text: &str) -> (String, Vec<(usize, usize)>)`

- **Does:** `text` as it reads, with the byte range of `text` each byte of it comes from.
- **Calls (crate):** `reads_as` (edit/anchor.rs:46) L69
- **Calls (std/external/unresolved):** `.char_indices`, `.extend`, `.len_utf8`, `.len`, `.push`, `String::with_capacity`, `Vec::with_capacity`, `std::iter::repeat_n`
- **Called by:** `reading_hits` (edit/anchor.rs:79)

#### `reading_hits`  (L79-91, pub(super))
`pub(super) fn reading_hits(text: &str, find: &str) -> Vec<(usize, usize)>`

- **Does:** Byte ranges of `text` that read as `find`, in order (overlapping ones too); none when `find` reads as nothing.
- **Calls (crate):** `reading` (edit/anchor.rs:65) L80,84
- **Calls (std/external/unresolved):** `.char_indices`, `.collect`, `.filter`, `.is_empty`, `.len`, `.map`, `.starts_with`, `Vec::new`
- **Called by:** `tests::text_is_found_by_what_it_reads_as` (edit/anchor.rs:285); `locate` (edit.rs:5475)

#### `shown`  (L95-111, pub(super))
`pub(super) fn shown(text: &str) -> String`

- **Does:** `text` quoted, with every character that looks like another or shows nothing written as `\u{…}`, so an agent sees why its anchor missed.
- **Calls (crate):** `reads_as` (edit/anchor.rs:46) L103
- **Calls (std/external/unresolved):** `.chars`, `.is_alphanumeric`, `.is_control`, `.push_str`, `.push`, `Some`, `String::from`, `from`
- **Macros:** `format!`
- **Called by:** `v2_select` (edit/anchor.rs:182); `v2_select_all` (edit/anchor.rs:238); `tests::lookalikes_and_invisible_characters_are_shown_escaped` (edit/anchor.rs:343); `locate` (edit.rs:5475); `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:144) (?)

#### `v2_candidates`  (L118-171, pub)
`pub fn v2_candidates(haystack: &str, anchor: &str) -> Vec<(usize, usize)>`

- **Does:** Every exact occurrence of `anchor` in `haystack`, including overlaps, as UTF-8 byte ranges in start order.
- **Calls (crate):** `probe_compare` (edit/anchor.rs:36) L132,136,151,158
- **Calls (std/external/unresolved):** `.as_bytes`, `.enumerate`, `.is_empty`, `.iter`, `.len`, `.push`, `.skip`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `v2_select` (edit/anchor.rs:182); `v2_select_all` (edit/anchor.rs:238); `tests::v2_counts_every_exact_start_position_including_overlaps` (edit/anchor.rs:353); `tests::v2_overlapping_search_matches_naive_scan_on_a_deterministic_corpus` (edit/anchor.rs:559); `tests::v2_overlapping_search_probes_in_linear_time` (edit/anchor.rs:610)

#### `v2_select`  (L182-226, pub)
`pub fn v2_select( haystack: &str, anchor: &str, occurrence: Option<usize>, ) -> Result<(usize, usize), EditError>`

- **Does:** Select the unique v2 literal candidate, or the 1-based `occurrence` within the complete candidate list, including overlapping ranges.
- **Calls (crate):** `err` (edit.rs:1401) L188,191,197,207,217; `v2_candidates` (edit/anchor.rs:118) L193; `shown` (edit/anchor.rs:95) L196,205,215
- **Calls (std/external/unresolved):** `.as_slice`, `.is_empty`, `.len`, `Err`, `Ok`, `Some`
- **Macros:** `format!`
- **Called by:** `tests::v2_selects_a_unique_anchor_and_refuses_with_the_count_when_repeated` (edit/anchor.rs:372); `tests::v2_occurrence_picks_the_nth_candidate_and_refuses_past_the_last` (edit/anchor.rs:390); `tests::v2_empty_anchor_is_an_invalid_edit` (edit/anchor.rs:456); `tests::v2_literal_anchors_preserve_exact_characters` (edit/anchor.rs:468); `tests::v2_selection_uses_utf8_byte_ranges` (edit/anchor.rs:496); `tests::v2_missing_anchors_refuse_with_zero_candidates` (edit/anchor.rs:515); `tests::v2_zero_occurrence_is_invalid_even_without_candidates` (edit/anchor.rs:531)

#### `v2_select_all`  (L238-277, pub)
`pub fn v2_select_all( haystack: &str, anchor: &str, expected: usize, ) -> Result<Vec<(usize, usize)>, EditError>`

- **Does:** Select every exact v2 candidate, requiring the complete count to equal `expected` and every range to be disjoint.
- **Calls (crate):** `err` (edit.rs:1401) L244,250,258,268; `v2_candidates` (edit/anchor.rs:118) L246; `shown` (edit/anchor.rs:95) L249,257,267
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.len`, `.windows`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `tests::v2_all_matches_takes_the_complete_disjoint_candidate_set` (edit/anchor.rs:408); `tests::v2_empty_anchor_is_an_invalid_edit` (edit/anchor.rs:456); `tests::v2_selection_uses_utf8_byte_ranges` (edit/anchor.rs:496); `tests::v2_missing_anchors_refuse_with_zero_candidates` (edit/anchor.rs:515); `tests::v2_overlap_message_states_the_condition_without_naming_a_flag` (edit/anchor.rs:640)

## Test functions (16)

- `probe_compare` (L36): Count one byte comparison.
- `tests::text_is_found_by_what_it_reads_as` (L285): _no doc comment_
- `tests::lookalikes_and_invisible_characters_are_shown_escaped` (L343): _no doc comment_
- `tests::v2_counts_every_exact_start_position_including_overlaps` (L353): _no doc comment_
- `tests::v2_selects_a_unique_anchor_and_refuses_with_the_count_when_repeated` (L372): _no doc comment_
- `tests::v2_occurrence_picks_the_nth_candidate_and_refuses_past_the_last` (L390): _no doc comment_
- `tests::v2_all_matches_takes_the_complete_disjoint_candidate_set` (L408): _no doc comment_
- `tests::v2_empty_anchor_is_an_invalid_edit` (L456): _no doc comment_
- `tests::v2_literal_anchors_preserve_exact_characters` (L468): _no doc comment_
- `tests::v2_selection_uses_utf8_byte_ranges` (L496): _no doc comment_
- `tests::v2_missing_anchors_refuse_with_zero_candidates` (L515): _no doc comment_
- `tests::v2_zero_occurrence_is_invalid_even_without_candidates` (L531): _no doc comment_
- `tests::naive_v2_candidates` (L546): The quadratic reference: `starts_with` re-scanned at every char index.
- `tests::v2_overlapping_search_matches_naive_scan_on_a_deterministic_corpus` (L559): _no doc comment_
- `tests::v2_overlapping_search_probes_in_linear_time` (L610): _no doc comment_
- `tests::v2_overlap_message_states_the_condition_without_naming_a_flag` (L640): _no doc comment_
