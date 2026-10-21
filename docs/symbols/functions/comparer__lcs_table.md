# Functions and methods in `src/comparer/lcs_table.rs`

20 non-test, 38 test. Part of the [symbol manifest](../README.md).

#### `as_group`  (L21-26)
`fn as_group(u: &ComparisonUnit) -> Option<&ComparisonUnitGroup>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `mark_word_table_mesh_context` (comparer/lcs_table.rs:381); `do_lcs_algorithm_for_table` (comparer/lcs_table.rs:626); `mark_rows_as_deleted_or_inserted` (comparer/lcs_table.rs:705); `horizontal_partition_tests::row_unit` (comparer/lcs_table.rs:1383); `nested_lifecycle_owner_regressions::group_owned_by` (comparer/lcs_table.rs:1412); `m42_word_context_boundaries::merged_package_context_requires_every_observed_partition_and_merge_direction` (comparer/lcs_table.rs:1539); `deterministic_table_dispatch_tests::source` (comparer/lcs_table.rs:1620); `deterministic_table_dispatch_tests::row_lcs_ties_and_both_one_sided_remainders_conserve_actual_atom_ownership` (comparer/lcs_table.rs:1692); `deterministic_table_dispatch_tests::seven_aligned_rows_use_content_lcs_while_six_use_positional_pairs` (comparer/lcs_table.rs:1772); `deterministic_table_dispatch_tests::vertical_merge_structure_dispatch_preserves_same_partition_and_each_remainder` (comparer/lcs_table.rs:1817); `shared_word_alignment_tests::row_tokens_keep_cell_paragraph_and_tab_boundaries_but_join_run_fragments` (comparer/lcs_table.rs:1972)

#### `ancestor_named`  (L29-43)
`fn ancestor_named( dom: &Dom, g: &ComparisonUnitGroup, name: &crate::xmllinq::XName, ) -> Option<NodeId>`

- **Does:** `w:tbl`/`w:tr` ancestor of a group's FIRST descendant atom.
- **Calls (crate):** `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) L35; `Dom::name` (xmllinq/mod.rs:526) L42
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.copied`, `.find`, `.first`, `.iter`, `.rev`, `ComparisonUnit::Group`, `Some`
- **Called by:** `table_has_merges` (comparer/lcs_table.rs:235); `mark_word_table_mesh_context` (comparer/lcs_table.rs:381); `do_lcs_algorithm_for_table` (comparer/lcs_table.rs:626); `m42_word_context_boundaries::merged_package_context_requires_every_observed_partition_and_merge_direction` (comparer/lcs_table.rs:1539)

#### `apply_lcs_to_table_rows`  (L51-130, pub)
`pub fn apply_lcs_to_table_rows( rows1: &[ComparisonUnit], rows2: &[ComparisonUnit], ) -> Vec<CorrelatedSequence>`

- **Does:** M4.D.1 — `ApplyLcsToTableRows` (WmlComparer.ts :8241): classic DP LCS over rows keyed on [`ComparisonUnit::sha1`], emitting Deleted/Inserted/Unknown sequences in document order.
- **Calls (crate):** `ComparisonUnit::sha1` (comparer/atoms.rs:355) (by name) L60,72; `CorrelatedSequence::deleted` (comparer/atoms.rs:518) L96; `CorrelatedSequence::inserted` (comparer/atoms.rs:527) L102; `CorrelatedSequence::paired` (comparer/atoms.rs:505) L108
- **Calls (std/external/unresolved):** `.clone`, `.len`, `.max`, `.push`, `.reverse`, `Vec::new`
- **Macros:** `unreachable!`, `vec!`
- **Called by:** `do_lcs_algorithm_for_table` (comparer/lcs_table.rs:626); `deterministic_table_dispatch_tests::row_lcs_ties_and_both_one_sided_remainders_conserve_actual_atom_ownership` (comparer/lcs_table.rs:1692)

#### `align_rows_by_shared_words`  (L147-203, pub(crate))
`pub(crate) fn align_rows_by_shared_words( old: &[std::collections::HashSet<String>], new: &[std::collections::HashSet<String>], ) -> Vec<WordRowOp>`

- **Does:** M4.D.5 — Word's row alignment for paired tables with no identical rows (Task 22, redline `3d4318d7e9`): `score(i, j)` is the number of shared case-insensitive alphanumeric word tokens; global max-weight monotone matching with gap cost 0; the backtrack deletes an old row on a tie, inserts a new row on a tie, and pairs...
- **Calls (std/external/unresolved):** `.chain`, `.copied`, `.count`, `.intersection`, `.iter`, `.len`, `.max`, `.min`, `.push`, `.sort_unstable`, `Vec::new`, `WordRowOp::Delete`, `WordRowOp::Insert`, `WordRowOp::Pair`, `score`
- **Macros:** `debug_assert_eq!`, `vec!`
- **Called by:** `shared_word_row_sequences` (comparer/lcs_table.rs:247); `shared_word_alignment_tests::anchors_win_over_positional_heads_with_inserted_gap_rows` (comparer/lcs_table.rs:1896); `shared_word_alignment_tests::zero_score_tables_pair_heads_positionally` (comparer/lcs_table.rs:1927); `shared_word_alignment_tests::deletion_wins_ties_against_pairing` (comparer/lcs_table.rs:1939); `shared_word_alignment_tests::insertion_wins_ties_against_pairing` (comparer/lcs_table.rs:1951); `shared_word_alignment_tests::heavier_anchor_pull_beats_earlier_single` (comparer/lcs_table.rs:1963)

#### `row_word_tokens`  (L206-233)
`fn row_word_tokens(dom: &Dom, row: &ComparisonUnit) -> std::collections::HashSet<String>`

- **Does:** The joined `w:t` text of a row as lowercase alphanumeric word tokens.
- **Calls (crate):** `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) L209; `Dom::name_is` (xmllinq/mod.rs:535) L210,216,222,223,224; `Dom::value_str` (xmllinq/mod.rs:1195) L220
- **Calls (std/external/unresolved):** `.collect`, `.copied`, `.filter`, `.find`, `.is_alphanumeric`, `.is_empty`, `.is_some`, `.iter`, `.map`, `.push_str`, `.push`, `.rev`, `.split`, `.to_lowercase`, `String::new`, `W::name`, `W::p`, `W::t`
- **Called by:** `shared_word_row_sequences` (comparer/lcs_table.rs:247); `shared_word_alignment_tests::row_tokens_keep_cell_paragraph_and_tab_boundaries_but_join_run_fragments` (comparer/lcs_table.rs:1972)

#### `table_has_merges`  (L235-241)
`fn table_has_merges(dom: &Dom, rows: &[ComparisonUnit]) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `ancestor_named` (comparer/lcs_table.rs:29) L239; `contains_merged` (comparer/lcs_table.rs:287) L240
- **Calls (std/external/unresolved):** `.filter_map`, `.find_map`, `.is_some_and`, `.iter`, `W::name`
- **Called by:** `shared_word_row_sequences` (comparer/lcs_table.rs:247)

#### `shared_word_row_sequences`  (L247-285, pub(crate))
`pub(crate) fn shared_word_row_sequences( dom: &Dom, rows1: &[ComparisonUnit], rows2: &[ComparisonUnit], ) -> Option<Vec<CorrelatedSequence>>`

- **Does:** Word's shared-word row alignment owns this table pair when Word mode is on, both sides have rows, no row is identical (the `sha1` LCS owns that case), and neither table uses merges (those keep their dedicated positional routes).
- **Calls (crate):** `table_has_merges` (comparer/lcs_table.rs:235) L255; `ComparisonUnit::sha1` (comparer/atoms.rs:355) (by name) L260; `row_word_tokens` (comparer/lcs_table.rs:206) L264,265; `align_rows_by_shared_words` (comparer/lcs_table.rs:147) L272; `CorrelatedSequence::paired` (comparer/atoms.rs:505) L275; `CorrelatedSequence::deleted` (comparer/atoms.rs:518) L280; `CorrelatedSequence::inserted` (comparer/atoms.rs:527) L281
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.collect`, `.into_iter`, `.is_empty`, `.iter`, `.map`, `Some`
- **Macros:** `vec!`
- **Called by:** `do_lcs_algorithm_for_table` (comparer/lcs_table.rs:626)

#### `contains_merged`  (L287-290)
`fn contains_merged(dom: &Dom, tbl: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L288,289
- **Calls (std/external/unresolved):** `.is_empty`, `Some`, `W::name`
- **Called by:** `table_has_merges` (comparer/lcs_table.rs:235); `do_lcs_algorithm_for_table` (comparer/lcs_table.rs:626)

#### `positional_rows`  (L292-304)
`fn positional_rows(rows1: &[ComparisonUnit], rows2: &[ComparisonUnit]) -> Vec<CorrelatedSequence>`

- **Does:** _no doc comment_
- **Calls (crate):** `CorrelatedSequence::paired` (comparer/atoms.rs:505) L297
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.iter`, `.map`, `.zip`
- **Macros:** `vec!`
- **Called by:** `do_lcs_algorithm_for_table` (comparer/lcs_table.rs:626)

#### `positional_rows_with_remainder`  (L312-332)
`fn positional_rows_with_remainder( rows1: &[ComparisonUnit], rows2: &[ComparisonUnit], ) -> Vec<CorrelatedSequence>`

- **Does:** Positional row pairing for unequal row counts: zip `min(len)` pairs as Unknown (cell-level LCS follows), then pure del/ins for the longer side's remainder.
- **Calls (crate):** `CorrelatedSequence::paired` (comparer/atoms.rs:505) L319; `CorrelatedSequence::deleted` (comparer/atoms.rs:518) L326; `CorrelatedSequence::inserted` (comparer/atoms.rs:527) L329
- **Calls (std/external/unresolved):** `.clone`, `.len`, `.min`, `.push`, `Vec::with_capacity`
- **Macros:** `vec!`
- **Called by:** `do_lcs_algorithm_for_table` (comparer/lcs_table.rs:626)

#### `row_container`  (L337-344, pub(crate))
`pub(crate) fn row_container(dom: &Dom, row: &ComparisonUnit) -> Option<NodeId>`

- **Does:** Physical cell boundaries, independent of vertical merges and nested tables.
- **Calls (crate):** `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) L338; `Dom::name_is` (xmllinq/mod.rs:535) L341
- **Calls (std/external/unresolved):** `.all`, `.contains`, `.copied`, `.find`, `.first`, `.iter`, `.rev`, `Some`, `W::tr`
- **Called by:** `resolve_correlated_sequences` (comparer/lcs.rs:10016); `horizontal_cell_partition` (comparer/lcs_table.rs:346); `row_pair_uses_word_table_mesh_context` (comparer/lcs_table.rs:530)

#### `horizontal_cell_partition`  (L346-365)
`fn horizontal_cell_partition(dom: &Dom, row: &ComparisonUnit) -> Option<Vec<u64>>`

- **Does:** _no doc comment_
- **Calls (crate):** `row_container` (comparer/lcs_table.rs:337) L347; `Dom::elements` (xmllinq/mod.rs:640) L350; `Dom::element` (xmllinq/mod.rs:655) L353,355; `Dom::attribute` (xmllinq/mod.rs:874) L356
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.into_iter`, `.map`, `.max`, `.ok`, `.parse`, `.unwrap_or`, `Some`, `W::grid_span`, `W::tc_pr`, `W::tc`, `W::val`, `u64::from`
- **Called by:** `rows_preserving_horizontal_partitions_inner` (comparer/lcs_table.rs:573); `horizontal_partition_tests::same_horizontal_partition_keeps_vertical_merge_pairing_and_normalizes_zero_span` (comparer/lcs_table.rs:1393)

#### `mark_word_table_mesh_context`  (L381-528, pub(crate))
`pub(crate) fn mark_word_table_mesh_context( dom: &mut Dom, original: &[ComparisonUnit], revised: &[ComparisonUnit], family: WordTableMeshContext, )`

- **Does:** _no doc comment_
- **Calls (crate):** `as_group` (comparer/lcs_table.rs:21) L391; `ancestor_named` (comparer/lcs_table.rs:29) L393; `tables` (ambiguous: `tables` (inspect.rs:389); `tables` (markdown/from_docx/agent.rs:1890)) L398,399; `Dom::elements` (xmllinq/mod.rs:640) L401,405,422,426,503,504...; `Dom::descendants` (xmllinq/mod.rs:668) L403,424; `Dom::element` (xmllinq/mod.rs:655) L408,411,412,429,431,436; `Dom::attribute` (xmllinq/mod.rs:874) L432,437,519,520; `horizontal_partition_tests::geometry` (comparer/lcs_table.rs:879) L473; `Dom::name_is` (xmllinq/mod.rs:535) L506,507,511; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L526,527
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.collect`, `.filter_map`, `.flatten`, `.is_empty`, `.is_none`, `.iter`, `.len`, `.map`, `.ok`, `.parse`, `.then`, `.unwrap_or`, `.zip`, `PT::name`, `PT::unid`, `Some`, `W::grid_span`, `W::name`, `W::p_pr`, `W::p`, `W::tbl`, `W::tc_pr`, `W::tc`, `W::tr`, `W::val`, `matches`, `merged_geometry`
- **Macros:** `format!`, `matches!`
- **Called by:** `detect_unrelated_sources_word_mode_inner` (comparer/lcs.rs:6215); `m42_word_context_boundaries::merged_package_context_requires_every_observed_partition_and_merge_direction` (comparer/lcs_table.rs:1539)

#### `row_pair_uses_word_table_mesh_context`  (L530-543)
`fn row_pair_uses_word_table_mesh_context( dom: &Dom, original: &ComparisonUnit, revised: &ComparisonUnit, ) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `row_container` (comparer/lcs_table.rs:337) L537; `Dom::ancestors` (xmllinq/mod.rs:770) L538; `Dom::attribute` (xmllinq/mod.rs:874) L540; `context` (edit.rs:7147) (by name) L542
- **Calls (std/external/unresolved):** `.find_map`, `.into_iter`, `.is_some_and`, `PT::name`, `Some`, `W::tbl`
- **Called by:** `rows_preserving_horizontal_partitions_inner` (comparer/lcs_table.rs:573)

#### `rows_preserving_horizontal_partitions_with_word_context`  (L545-557, pub(crate))
`pub(crate) fn rows_preserving_horizontal_partitions_with_word_context( dom: &Dom, original: &[ComparisonUnit], revised: &[ComparisonUnit], settings: &WmlComparerSettings, ) -> Option<Vec<CorrelatedSequence>>`

- **Does:** _no doc comment_
- **Calls (crate):** `rows_preserving_horizontal_partitions_inner` (comparer/lcs_table.rs:573) L551
- **Called by:** `resolve_correlated_sequences` (comparer/lcs.rs:10016); `do_lcs_algorithm_for_table` (comparer/lcs_table.rs:626); `m42_word_context_boundaries::merged_package_context_requires_every_observed_partition_and_merge_direction` (comparer/lcs_table.rs:1539)

#### `rows_preserving_horizontal_partitions_inner`  (L573-619)
`fn rows_preserving_horizontal_partitions_inner( dom: &Dom, rows1: &[ComparisonUnit], rows2: &[ComparisonUnit], word_context: bool, ) -> Option<Vec<CorrelatedSequence>>`

- **Does:** _no doc comment_
- **Calls (crate):** `row_pair_uses_word_table_mesh_context` (comparer/lcs_table.rs:530) L580; `horizontal_cell_partition` (comparer/lcs_table.rs:346) L584,585; `CorrelatedSequence::inserted` (comparer/atoms.rs:527) L598,616; `CorrelatedSequence::deleted` (comparer/atoms.rs:518) L599,611; `CorrelatedSequence::paired` (comparer/atoms.rs:505) L601
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.extend`, `.iter`, `.len`, `.map`, `.min`, `.push`, `.zip`, `Some`, `Vec::new`, `differs`
- **Macros:** `vec!`
- **Called by:** `rows_preserving_horizontal_partitions_with_word_context` (comparer/lcs_table.rs:545); `rows_preserving_horizontal_partitions` (comparer/lcs_table.rs:565)

#### `do_lcs_algorithm_for_table`  (L626-699, pub)
`pub fn do_lcs_algorithm_for_table( dom: &Dom, cul1: &[ComparisonUnit], cul2: &[ComparisonUnit], settings: &WmlComparerSettings, ) -> Option<Vec<CorrelatedSequence>>`

- **Does:** M4.D.2/D.3 — `DoLcsAlgorithmForTable` (WmlComparer.ts :8348): single-table-vs-single-table dispatch.
- **Calls (crate):** `as_group` (comparer/lcs_table.rs:21) L632,633; `shared_word_row_sequences` (comparer/lcs_table.rs:247) L639; `rows_preserving_horizontal_partitions_with_word_context` (comparer/lcs_table.rs:545) L644; `ComparisonUnit::correlated_sha1` (comparer/atoms.rs:384) (by name) L655; `ComparisonUnit::sha1` (comparer/atoms.rs:355) (by name) L659; `apply_lcs_to_table_rows` (comparer/lcs_table.rs:51) L667; `positional_rows` (comparer/lcs_table.rs:292) L673,685; `ancestor_named` (comparer/lcs_table.rs:29) L679,680; `contains_merged` (comparer/lcs_table.rs:287) L679,680; `positional_rows_with_remainder` (comparer/lcs_table.rs:312) L691; `CorrelatedSequence::deleted` (comparer/atoms.rs:518) L694; `CorrelatedSequence::inserted` (comparer/atoms.rs:527) L695
- **Calls (std/external/unresolved):** `.all`, `.count`, `.filter`, `.first`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.to_vec`, `.zip`, `Some`, `W::name`
- **Macros:** `vec!`
- **Called by:** `step_h` (comparer/lcs.rs:3621); `deterministic_table_dispatch_tests::seven_aligned_rows_use_content_lcs_while_six_use_positional_pairs` (comparer/lcs_table.rs:1772); `deterministic_table_dispatch_tests::vertical_merge_structure_dispatch_preserves_same_partition_and_each_remainder` (comparer/lcs_table.rs:1817)

#### `mark_rows_as_deleted_or_inserted`  (L705-759, pub)
`pub fn mark_rows_as_deleted_or_inserted( dom: &mut Dom, settings: &WmlComparerSettings, sequences: &[CorrelatedSequence], next_id: &mut u32, )`

- **Does:** M4.D.4 — `MarkRowsAsDeletedOrInserted` (:4216): for each Deleted/Inserted sequence, add `w:del`/`w:ins` into each Row's `w:trPr` (created as first child if absent).
- **Calls (crate):** `as_group` (comparer/lcs_table.rs:21) L721; `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) (by name) L727; `Dom::name_is` (xmllinq/mod.rs:535) L733; `Dom::descendants` (xmllinq/mod.rs:668) L741; `row_properties` (comparer/lcs_table.rs:763) L749; `Dom::new_element` (xmllinq/mod.rs:478) L750; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L751,752,754; `add_row_mark` (comparer/lcs_table.rs:778) L755
- **Calls (std/external/unresolved):** `.all`, `.as_ref`, `.clone`, `.contains`, `.copied`, `.find`, `.first`, `.insert`, `.iter`, `.rev`, `.to_string`, `Some`, `W::author`, `W::date`, `W::del`, `W::id`, `W::ins`, `W::name`, `W::tbl`, `std::collections::HashSet::new`
- **Macros:** `vec!`
- **Called by:** `process_footnote_endnote` (comparer/footnotes.rs:1115); `horizontal_partition_tests::actual_atomized_split_with_deleted_table_conserves_source_through_production` (comparer/lcs_table.rs:1186); `nested_lifecycle_owner_regressions::lifecycle_units_mark_their_own_rows_even_when_the_first_atom_is_nested` (comparer/lcs_table.rs:1438); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `row_properties`  (L763-773, pub(crate))
`pub(crate) fn row_properties(dom: &mut Dom, tr: NodeId) -> NodeId`

- **Does:** The row's `w:trPr`, created when absent after a `w:tblPrEx` (CT_Row order), else as the row's first child.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L764,768; `Dom::new_element` (xmllinq/mod.rs:478) L767; `Dom::add_after_self` (xmllinq/mod.rs:1147) L769; `Dom::add_first` (xmllinq/mod.rs:1101) L770
- **Calls (std/external/unresolved):** `W::name`
- **Called by:** `mark_rows_as_deleted_or_inserted` (comparer/lcs_table.rs:705); `mark_story_rows` (document_comparer.rs:5756)

#### `add_row_mark`  (L778-783, pub(crate))
`pub(crate) fn add_row_mark(dom: &mut Dom, trpr: NodeId, rev: NodeId)`

- **Does:** Add a row's `w:ins` / `w:del` to its `w:trPr`: CT_TrPr keeps them ahead of a `w:trPrChange` (acce1b593c's header rows wrote the mark after it: the schema rejects that, and Word timed out saving the redline as PDF).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L779; `Dom::add_before_self` (xmllinq/mod.rs:1130) L780; `Dom::add` (xmllinq/mod.rs:1074) L781
- **Calls (std/external/unresolved):** `W::name`
- **Called by:** `mark_fully_revised_rows` (comparer/finalize.rs:11671); `mark_rows_as_deleted_or_inserted` (comparer/lcs_table.rs:705); `row_mark_tests::a_row_mark_goes_before_the_row_property_change` (comparer/lcs_table.rs:798); `mark_story_rows` (document_comparer.rs:5756)

## Test functions (38)

- `rows_preserving_horizontal_partitions` (L565): Horizontal repartitioning currently cannot track individual cell lifetimes safely: an unmatched cell's atoms lose their cell group, and closing marks can give it another cell's revised properties.
- `row_mark_tests::child_names` (L790): _no doc comment_
- `row_mark_tests::a_row_mark_goes_before_the_row_property_change` (L798): _no doc comment_
- `horizontal_partition_tests::cell` (L824): _no doc comment_
- `horizontal_partition_tests::table` (L829): _no doc comment_
- `horizontal_partition_tests::package` (L856): _no doc comment_
- `horizontal_partition_tests::prop` (L863): _no doc comment_
- `horizontal_partition_tests::geometry` (L879): _no doc comment_
- `horizontal_partition_tests::grids` (L918): _no doc comment_
- `horizontal_partition_tests::horizontal_repartition_roundtrips_source_cells_properties_and_nested_tables` (L939): _no doc comment_
- `horizontal_partition_tests::offset_table` (L994): _no doc comment_
- `horizontal_partition_tests::horizontal_row_offsets_roundtrip_exact_authored_grid_and_row_properties` (L1019): _no doc comment_
- `horizontal_partition_tests::horizontal_row_and_cell_lifetimes_survive_with_format_tracking_disabled` (L1061): _no doc comment_
- `horizontal_partition_tests::body_paragraph_texts` (L1126): _no doc comment_
- `horizontal_partition_tests::split_changed_prose_keeps_original_literal_space_and_tracks_its_new_boundary` (L1143): _no doc comment_
- `horizontal_partition_tests::actual_atomized_split_with_deleted_table_conserves_source_through_production` (L1186): _no doc comment_
- `horizontal_partition_tests::justified_underline_peel_preserves_both_sources_and_word_three_mix_shape` (L1315): _no doc comment_
- `horizontal_partition_tests::row_unit` (L1383): _no doc comment_
- `horizontal_partition_tests::same_horizontal_partition_keeps_vertical_merge_pairing_and_normalizes_zero_span` (L1393): _no doc comment_
- `nested_lifecycle_owner_regressions::group_owned_by` (L1412): _no doc comment_
- `nested_lifecycle_owner_regressions::lifecycle_units_mark_their_own_rows_even_when_the_first_atom_is_nested` (L1438): _no doc comment_
- `m42_word_context_boundaries::table` (L1498): _no doc comment_
- `m42_word_context_boundaries::units` (L1523): _no doc comment_
- `m42_word_context_boundaries::merged_package_context_requires_every_observed_partition_and_merge_direction` (L1539): _no doc comment_
- `deterministic_table_dispatch_tests::source` (L1620): _no doc comment_
- `deterministic_table_dispatch_tests::atoms` (L1657): _no doc comment_
- `deterministic_table_dispatch_tests::assert_sources` (L1664): _no doc comment_
- `deterministic_table_dispatch_tests::row_lcs_ties_and_both_one_sided_remainders_conserve_actual_atom_ownership` (L1692): _no doc comment_
- `deterministic_table_dispatch_tests::seven_aligned_rows_use_content_lcs_while_six_use_positional_pairs` (L1772): _no doc comment_
- `deterministic_table_dispatch_tests::vertical_merge_structure_dispatch_preserves_same_partition_and_each_remainder` (L1817): _no doc comment_
- `shared_word_alignment_tests::toks` (L1883): _no doc comment_
- `shared_word_alignment_tests::rows` (L1887): _no doc comment_
- `shared_word_alignment_tests::anchors_win_over_positional_heads_with_inserted_gap_rows` (L1896): Task 22 first table: 3 old rows against 21 new, anchors at new 0/3/7 (here 0/3/7 of 8).
- `shared_word_alignment_tests::zero_score_tables_pair_heads_positionally` (L1927): No shared token at all: heads pair positionally, the rest inserts.
- `shared_word_alignment_tests::deletion_wins_ties_against_pairing` (L1939): Deleting the old row beats pairing it when both reach the optimum (Task 22 second table tail: A18 stays deleted, B9 keeps A14).
- `shared_word_alignment_tests::insertion_wins_ties_against_pairing` (L1951): Symmetric inference (no direct Word case): inserting the new row beats pairing it on a tie.
- `shared_word_alignment_tests::heavier_anchor_pull_beats_earlier_single` (L1963): A higher-weight anchor pulls the alignment even when an earlier single-share pairing is available (Task 22: A10 takes B5 for 2).
- `shared_word_alignment_tests::row_tokens_keep_cell_paragraph_and_tab_boundaries_but_join_run_fragments` (L1972): _no doc comment_
