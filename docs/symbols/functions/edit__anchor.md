# Functions and methods in `src/edit/anchor.rs`

12 non-test, 16 test. Part of the [symbol manifest](../README.md).

#### `probe_compare`  (L43-43)
`fn probe_compare()`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `v2_candidates` (edit/anchor.rs:232)

#### `reads_as`  (L46-61, pub(super))
`pub(super) fn reads_as(c: char) -> Option<char>`

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
- **Called by:** `tests::text_is_found_by_what_it_reads_as` (edit/anchor.rs:387); `removed_holder` (edit/hint.rs:144); `locate` (edit.rs:7156)

#### `shown`  (L95-111, pub(super))
`pub(super) fn shown(text: &str) -> String`

- **Does:** `text` quoted, with every character that looks like another or shows nothing written as `\u{…}`, so an agent sees why its anchor missed.
- **Calls (crate):** `reads_as` (edit/anchor.rs:46) L103
- **Calls (std/external/unresolved):** `.chars`, `.is_alphanumeric`, `.is_control`, `.push_str`, `.push`, `Some`, `String::from`, `from`
- **Macros:** `format!`
- **Called by:** `near_matches` (edit/anchor.rs:187); `missing` (edit/anchor.rs:217); `v2_select` (edit/anchor.rs:296); `v2_select_all` (edit/anchor.rs:345); `tests::lookalikes_and_invisible_characters_are_shown_escaped` (edit/anchor.rs:445); `refused` (edit/anchor_error.rs:462) (?); `quoted_anchor` (edit/anchor_error.rs:591) (?); `tests::a_pasted_clause_anchor_is_quoted_bounded` (edit/anchor_error.rs:715) (?); `locate` (edit.rs:7156)

#### `NearMatches::copy_hint`  (L139-149, pub)
`pub fn copy_hint(&self) -> String`

- **Does:** One extra refusal line.
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.iter`, `.join`, `.map`
- **Macros:** `format!`
- **Called by:** `run` (bin/jubarte/edit_typed.rs:787) (?); `refused` (bin/jubarte.rs:2697) (?)

#### `whitespace_reading`  (L155-181)
`fn whitespace_reading(text: &str, document: bool) -> (String, Vec<(usize, usize)>)`

- **Does:** Collapse nonempty Unicode whitespace runs, mapping each output byte back to its complete input span.
- **Calls (std/external/unresolved):** `.char_indices`, `.expect`, `.extend`, `.is_whitespace`, `.last_mut`, `.len_utf8`, `.len`, `.push`, `String::with_capacity`, `Vec::with_capacity`, `std::iter::repeat_n`
- **Macros:** `matches!`
- **Called by:** `near_matches` (edit/anchor.rs:187)

#### `near_matches`  (L187-215, pub(super))
`pub(super) fn near_matches(text: &str, anchor: &str) -> Option<NearMatches>`

- **Does:** Explain a literal miss without selecting or editing anything.
- **Calls (crate):** `whitespace_reading` (edit/anchor.rs:155) L188,192; `v2_candidates` (edit/anchor.rs:232) L193; `shown` (edit/anchor.rs:95) L205
- **Calls (std/external/unresolved):** `.collect`, `.is_empty`, `.iter`, `.len`, `.map`, `.take`, `.to_string`, `Some`
- **Called by:** `missing` (edit/anchor.rs:217); `compile` (edit/inline.rs:38); `locate` (edit.rs:7156)

#### `missing`  (L217-225)
`fn missing(haystack: &str, anchor: &str) -> EditError`

- **Does:** _no doc comment_
- **Calls (crate):** `err` (edit.rs:1770) L218; `shown` (edit/anchor.rs:95) L221; `near_matches` (edit/anchor.rs:187) L223
- **Calls (std/external/unresolved):** `.map`
- **Macros:** `format!`
- **Called by:** `v2_select` (edit/anchor.rs:296); `v2_select_all` (edit/anchor.rs:345); `missing_foreign_markers` (validate.rs:3454) (?)

#### `v2_candidates`  (L232-285, pub)
`pub fn v2_candidates(haystack: &str, anchor: &str) -> Vec<(usize, usize)>`

- **Does:** Every exact occurrence of `anchor` in `haystack`, including overlaps, as UTF-8 byte ranges in start order.
- **Calls (crate):** `probe_compare` (edit/anchor.rs:36) L246,250,265,272
- **Calls (std/external/unresolved):** `.as_bytes`, `.enumerate`, `.is_empty`, `.iter`, `.len`, `.push`, `.skip`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `near_matches` (edit/anchor.rs:187); `v2_select` (edit/anchor.rs:296); `v2_select_all` (edit/anchor.rs:345); `tests::v2_counts_every_exact_start_position_including_overlaps` (edit/anchor.rs:455); `tests::v2_overlapping_search_matches_naive_scan_on_a_deterministic_corpus` (edit/anchor.rs:661); `tests::v2_overlapping_search_probes_in_linear_time` (edit/anchor.rs:712); `refused` (edit/anchor_error.rs:462) (?); `suggestions_elsewhere` (edit/anchor_error.rs:551) (?); `compile` (edit/inline.rs:38)

#### `v2_select`  (L296-333, pub)
`pub fn v2_select( haystack: &str, anchor: &str, occurrence: Option<usize>, ) -> Result<(usize, usize), EditError>`

- **Does:** Select the unique v2 literal candidate, or the 1-based `occurrence` within the complete candidate list, including overlapping ranges.
- **Calls (crate):** `err` (edit.rs:1770) L302,305,314,324; `v2_candidates` (edit/anchor.rs:232) L307; `missing` (edit/anchor.rs:217) L309; `shown` (edit/anchor.rs:95) L312,322
- **Calls (std/external/unresolved):** `.as_slice`, `.is_empty`, `.len`, `Err`, `Ok`, `Some`
- **Macros:** `format!`
- **Called by:** `tests::v2_selects_a_unique_anchor_and_refuses_with_the_count_when_repeated` (edit/anchor.rs:474); `tests::v2_occurrence_picks_the_nth_candidate_and_refuses_past_the_last` (edit/anchor.rs:492); `tests::v2_empty_anchor_is_an_invalid_edit` (edit/anchor.rs:558); `tests::v2_literal_anchors_preserve_exact_characters` (edit/anchor.rs:570); `tests::v2_selection_uses_utf8_byte_ranges` (edit/anchor.rs:598); `tests::v2_missing_anchors_refuse_with_zero_candidates` (edit/anchor.rs:617); `tests::v2_zero_occurrence_is_invalid_even_without_candidates` (edit/anchor.rs:633); `v2_resolve` (edit/anchor_error.rs:139) (?); `resolve_operation` (edit/v2_apply.rs:2084)

#### `v2_select_all`  (L345-379, pub)
`pub fn v2_select_all( haystack: &str, anchor: &str, expected: usize, ) -> Result<Vec<(usize, usize)>, EditError>`

- **Does:** Select every exact v2 candidate, requiring the complete count to equal `expected` and every range to be disjoint.
- **Calls (crate):** `err` (edit.rs:1770) L351,360,370; `v2_candidates` (edit/anchor.rs:232) L353; `missing` (edit/anchor.rs:217) L356; `shown` (edit/anchor.rs:95) L359,369
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.len`, `.windows`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `tests::v2_all_matches_takes_the_complete_disjoint_candidate_set` (edit/anchor.rs:510); `tests::v2_empty_anchor_is_an_invalid_edit` (edit/anchor.rs:558); `tests::v2_selection_uses_utf8_byte_ranges` (edit/anchor.rs:598); `tests::v2_missing_anchors_refuse_with_zero_candidates` (edit/anchor.rs:617); `tests::v2_overlap_message_states_the_condition_without_naming_a_flag` (edit/anchor.rs:742); `v2_resolve_all` (edit/anchor_error.rs:176) (?); `compile` (edit/inline.rs:38)

## Test functions (16)

- `probe_compare` (L36): Count one byte comparison.
- `tests::text_is_found_by_what_it_reads_as` (L387): _no doc comment_
- `tests::lookalikes_and_invisible_characters_are_shown_escaped` (L445): _no doc comment_
- `tests::v2_counts_every_exact_start_position_including_overlaps` (L455): _no doc comment_
- `tests::v2_selects_a_unique_anchor_and_refuses_with_the_count_when_repeated` (L474): _no doc comment_
- `tests::v2_occurrence_picks_the_nth_candidate_and_refuses_past_the_last` (L492): _no doc comment_
- `tests::v2_all_matches_takes_the_complete_disjoint_candidate_set` (L510): _no doc comment_
- `tests::v2_empty_anchor_is_an_invalid_edit` (L558): _no doc comment_
- `tests::v2_literal_anchors_preserve_exact_characters` (L570): _no doc comment_
- `tests::v2_selection_uses_utf8_byte_ranges` (L598): _no doc comment_
- `tests::v2_missing_anchors_refuse_with_zero_candidates` (L617): _no doc comment_
- `tests::v2_zero_occurrence_is_invalid_even_without_candidates` (L633): _no doc comment_
- `tests::naive_v2_candidates` (L648): The quadratic reference: `starts_with` re-scanned at every char index.
- `tests::v2_overlapping_search_matches_naive_scan_on_a_deterministic_corpus` (L661): _no doc comment_
- `tests::v2_overlapping_search_probes_in_linear_time` (L712): _no doc comment_
- `tests::v2_overlap_message_states_the_condition_without_naming_a_flag` (L742): _no doc comment_
