# Functions and methods in `src/comparer/mod.rs`

16 non-test, 5 test. Part of the [symbol manifest](../README.md).

#### `compare_bodies`  (L79-89, pub)
`pub fn compare_bodies( dom: &mut Dom, body1: NodeId, body2: NodeId, settings: &WmlComparerSettings, ) -> NodeId`

- **Does:** Compare two content-parent bodies (already accepted/clean) and produce a redline `<w:document>` node in `dom`.
- **Calls (crate):** `create_comparison_unit_atom_list` (comparer/atomize.rs:307) L85,86; `correlate_atoms` (comparer/lcs.rs:69) L87; `produce_document` (comparer/produce.rs:65) L88
- **Called by:** none found in src

#### `compare_bodies_faithful`  (L109-126, pub)
`pub fn compare_bodies_faithful( dom: &mut Dom, source_root1: NodeId, source_root2: NodeId, body1: NodeId, body2: NodeId, settings: &WmlComparerSettings, ) -> NodeId`

- **Does:** M4.I — faithful end-to-end body compare: the real WmlComparer produce path (atomize → hash → `GetComparisonUnitList` → LCS → `MarkRows` → `Flatten` → `AssembleAncestorUnids` → `CoalesceRecurse` → `MarkContent` → `IgnorePt14` → `Conjoin` → `FixUpRevisionIds` → `RemoveScratch`).
- **Calls (crate):** `compare_bodies_faithful_with_notes` (comparer/mod.rs:173) L117
- **Called by:** `coverage_real_revision_merge_matrix_tests::exercise` (comparer/finalize.rs:24959); `coverage_real_revision_merge_matrix_tests::public_two_original_one_revised_windows_keep_complete_sources_after_all_word_phases` (comparer/finalize.rs:26498); `coverage_real_revision_merge_matrix_tests::repeated_authored_prefixes_keep_deleted_paragraphs_before_their_unchanged_table` (comparer/finalize.rs:26636); `coverage_real_revision_merge_matrix_tests::unshared_wrap_gap_keeps_each_sources_preserved_terminal_space` (comparer/finalize.rs:26742); `paired_row_mark_ownership_tests::public_empty_table_comparison_preserves_complete_authored_rows_and_pilcrows` (comparer/finalize.rs:29013); `closing_authored_section_boundary_tests::public_added_and_unchanged_authored_sections_keep_complete_source_bodies` (comparer/finalize.rs:29383); `body_section_source_ownership_tests::faithful_final_body_sections_never_become_paragraph_breaks_even_with_identical_authored_geometry` (comparer/mod.rs:1726); `body_section_source_ownership_tests::final_section_change_history_keeps_its_owner_and_identical_genuine_breaks` (comparer/mod.rs:1775); `body_section_source_ownership_tests::authored_body_paragraph_section_before_an_ending_table_is_never_the_final_body_owner` (comparer/mod.rs:1913); `content_control_source_order_regressions::introducing_or_removing_a_control_keeps_replacement_words_in_both_source_positions` (comparer/produce.rs:1958); `content_control_source_order_regressions::equal_control_ancestry_keeps_its_wrapper_and_authored_properties` (comparer/produce.rs:1997); `repeated_control_tag_regressions::repeated_tag_outer_wrapper_never_replaces_retained_inner_control_metadata` (comparer/produce.rs:2348); +2 more

#### `compare_bodies_faithful_with_notes`  (L173-192, pub)
`pub fn compare_bodies_faithful_with_notes( dom: &mut Dom, source_root1: NodeId, source_root2: NodeId, body1: NodeId, body2: NodeId, settings: &WmlComparerSettings, notes: Option<&mut NotesContext>, ) -> NodeId`

- **Does:** [`compare_bodies_faithful`] with a [`NotesContext`]: when `Some`, footnote/endnote definitions are processed per correlated reference (`ProcessFootnoteEndnote`) and the output notes parts are renumbered (`RectifyFootnoteEndnoteIds`) before the pipeline ends.
- **Calls (crate):** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209) L182
- **Calls (std/external/unresolved):** `.unwrap_or_else`
- **Macros:** `panic!`
- **Called by:** `compare_bodies_faithful` (comparer/mod.rs:109)

#### `try_compare_bodies_faithful_with_notes`  (L209-1252, pub)
`pub fn try_compare_bodies_faithful_with_notes( dom: &mut Dom, source_root1: NodeId, source_root2: NodeId, body1: NodeId, body2: NodeId, settings: &WmlComparerSettings, notes: Option<&mut NotesContext>, ) -> Result<NodeId, footnotes::RectifyError>`

- **Does:** [`compare_bodies_faithful_with_notes`] that returns the notes-layout failure as a `super::footnotes::RectifyError` instead of panicking; the package-level comparer uses this so a malformed notes part is an `Err`, never an abort of the host.
- **Calls (crate):** `last_body_para_spacing` (comparer/finalize.rs:3422) L240; `closing_paragraph_properties` (comparer/finalize.rs:3051) L244,245; `paragraph_alignments` (comparer/finalize.rs:13369) L249; `unaligned_paragraph_texts` (comparer/finalize.rs:13314) L251; `Dom::element` (xmllinq/mod.rs:655) L254,298,326,328,373,384...; `Dom::descendants` (xmllinq/mod.rs:668) L255,282,476,843; `word_default_sectpr` (comparer/mod.rs:1576) L265; `Dom::new_element` (xmllinq/mod.rs:478) L270,371,415,426,458,742...; `Dom::attributes` (xmllinq/mod.rs:885) L271,759; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L272,315,396,416,421,449...; `Dom::elements` (xmllinq/mod.rs:640) L287,345,477,491,850; `Dom::attribute` (xmllinq/mod.rs:874) L289,305,385,446,447,770...; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L292,299,339,374,429,559...; `Dom::add` (xmllinq/mod.rs:1058) L293,318,351,375,430,433...; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L312,393; `pg_num_type_is_implicit_decimal` (comparer/finalize.rs:6048) L336; `Dom::name` (xmllinq/mod.rs:526) L346; `XName::local_name` (xmllinq/mod.rs:208) (by name) L347; `Dom::add_before_self` (xmllinq/mod.rs:1114) L350,461; `Dom::remove` (xmllinq/mod.rs:1102) L387,404,846,851; `cols_is_word_default` (comparer/mod.rs:1546) L403; `canonicalize_attr_order` (comparer/mod.rs:1521) L411; `sectpr_identity` (comparer/finalize.rs:11604) L412,478; `resolve_alternate_content` (comparer/finalize.rs:1259) L503,504; `sanitize_sdt_properties` (comparer/finalize.rs:1312) L509,510; `coalesce_all_paragraphs` (comparer/finalize.rs:1461) L519,520,787; `flatten_tracked_insertions_stamped` (comparer/finalize.rs:10215) L533; `pending_deletion_texts` (comparer/finalize.rs:10976) L534; `flatten_tracked_deletions` (comparer/finalize.rs:11030) L535,546; `assign_to_all_elements` (unid.rs:60) L548,549; `element_has_tracked_revisions` (revision_processor.rs:90) L555,556; `accept_revisions_document` (revision_processor.rs:502) L560,612,613; `hash_block_level_content` (comparer/preprocess.rs:1317) L561,569,580,588,629,638; `reject_revisions_document` (revision_processor.rs:729) L579; `revised_two_blank_table_titles` (comparer/finalize.rs:3848) L616; `add_sha1_hash_to_block_level_content` (comparer/preprocess.rs:1259) L650,656; `create_comparison_unit_atom_list` (comparer/atomize.rs:307) L663,664; `get_comparison_unit_list` (comparer/units.rs:140) L665,666; `detect_unrelated_sources_word_mode` (comparer/lcs.rs:6062) L689; `lcs` (comparer/lcs.rs:10084) L694,698; `detect_unrelated_sources` (comparer/lcs.rs:5620) L697; `pair_story_final_marks` (comparer/lcs.rs:5786) L702; `Dom::name_is` (xmllinq/mod.rs:535) L705; `restream_cross_paragraph_regions` (comparer/cross_para.rs:138) L706; `final_empty_paragraphs_paired` (comparer/mod.rs:1642) L713; `unit_closes_story` (comparer/lcs.rs:3501) L723; `promote_skip_ahead_equals_with_source` (comparer/moves.rs:270) L727; `mark_rows_as_deleted_or_inserted` (comparer/lcs_table.rs:705) L730; `flatten_to_comparison_unit_atom_list` (comparer/produce.rs:225) L732; `detect_moves_in_atom_list` (comparer/moves.rs:470) L735; `detect_format_changes_in_atom_list` (comparer/formatchg.rs:383) L736; `assemble_ancestor_unids` (comparer/produce.rs:470) L737; `produce_new_wml_markup_from_correlated_sequence` (comparer/produce.rs:1769) L739; `XNamespace::xmlns` (xmllinq/mod.rs:122) L745,753; `XNamespace::name` (xmllinq/mod.rs:134) L754; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L764; `XNamespace::namespace_name` (xmllinq/mod.rs:140) L764; `mark_content_as_deleted_or_inserted` (comparer/finalize.rs:517) L786; `ignore_pt14_namespace` (comparer/finalize.rs:819) L788; `process_footnote_endnote` (comparer/footnotes.rs:1115) L794; `rectify_footnote_endnote_ids` (comparer/footnotes.rs:673) L799; `conjoin_paragraph_marks` (comparer/finalize.rs:702) L816; `merged_closing_carrier_takes_source_properties` (comparer/finalize.rs:3080) L818; `fix_up_revision_ids` (comparer/finalize.rs:725) L827,1243; `wml_order_elements_per_standard` (comparer/finalize.rs:10483) L832; `fix_up_footnotes_endnotes_with_custom_markers` (comparer/footnotes.rs:887) L867; `simplify_move_markup_to_del_ins` (comparer/finalize.rs:6809) L871; `fix_up_doc_pr_ids` (comparer/fixups.rs:32) L880,1248; `fix_up_shape_ids` (comparer/fixups.rs:54) L881,1249; `fix_up_shape_type_ids` (comparer/fixups.rs:75) L882,1250; `convert_stamped_predeletes` (comparer/finalize.rs:11136) L886; `convert_stamped_preins` (comparer/finalize.rs:10282) L887; `remove_powertools_scratch_markup` (comparer/finalize.rs:1018) L889; `normalize_universal_measures` (comparer/finalize.rs:11485) L893; `move_paragraph_properties_first` (comparer/finalize.rs:1524) L896; `capture_deleted_demo_title_properties` (comparer/finalize.rs:12579) L898; `strip_redundant_demo_default_spacing` (comparer/finalize.rs:2266) L912; `strip_redundant_normal_pstyle_and_bidi` (comparer/finalize.rs:2362) L919; `normalize_incomplete_spacing` (comparer/finalize.rs:2176) L921; `ensure_pure_i_list_snug_spacing` (comparer/finalize.rs:2071) L923; `unwrap_hyperlinks_to_styled_runs` (comparer/finalize.rs:1555) L928; `unwrap_content_controls_in_pure_revisions` (comparer/finalize.rs:1412) L931; `strip_trailing_empty_pure_del_mark` (comparer/finalize.rs:2909) L933; `trailing_empty_spacing_to_pprchange` (comparer/finalize.rs:2967) L935,1043; `strip_trailing_empty_pure_ins` (comparer/finalize.rs:3579) L945,1096; `fold_whitespace_pure_ins_into_following_pure_del` (comparer/finalize.rs:4211) L953,1099; `ensure_empty_pure_i_before_short_title_del` (comparer/finalize.rs:3910) L956,1102; `strip_empty_pure_ins_before_trailing_pure_dels` (comparer/finalize.rs:4062) L966,1110; `strip_last_pure_del_mark_only_ppr` (comparer/finalize.rs:5081) L969,1085; `strip_last_pure_del_mark_when_pprchange` (comparer/finalize.rs:6593) L970,1087; `fix_paragraph_mark_revision_order` (comparer/finalize.rs:1484) L975; `reorder_replacements_ins_before_del` (comparer/finalize.rs:6652) L977; `coalesce_adjacent_revisions` (comparer/finalize.rs:6759) L981; `wrap_bare_del_text_runs` (comparer/finalize.rs:10771) L985; `reorder_replaced_blocks` (comparer/finalize.rs:10075) L989; `splice_trailing_short_pure_dels_midstream` (comparer/finalize.rs:11798) L992; `interleave_list_cluster_after_coalesce` (comparer/finalize.rs:9876) L995,1176; `merge_replaced_paragraphs_in` (comparer/finalize.rs:6890) L998; `restore_short_del_before_long_ins` (comparer/finalize.rs:12180) L1006; `split_digits_ins_from_mixed_title` (comparer/finalize.rs:11954) L1008; `fold_midstream_demo_title_into_numbered_heading_with_source_properties` (comparer/finalize.rs:12619) L1011; `drop_sectpr_from_deleted_marks` (comparer/finalize.rs:10179) L1018; `drop_hoisted_sectpr_artifacts` (comparer/finalize.rs:11637) L1019; `mark_fully_revised_rows` (comparer/finalize.rs:11666) L1020; `has_default_table_style` (document_comparer.rs:3156) L1025,1026; `synthesize_table_cell_margins` (comparer/finalize.rs:11249) L1028; `ensure_default_page_size` (comparer/finalize.rs:10388) L1029; `begin_para_classification_cache` (comparer/finalize.rs:49) L1032; `last_pure_del_spacing_to_pprchange` (comparer/finalize.rs:5164) L1037; `cleanup_spacing_and_default_jc` (comparer/finalize.rs:5381) L1041; `mixed_spacing_to_following_empty` (comparer/finalize.rs:5267) L1051; `park_mixed_spacing_onto_trailing_pure_del` (comparer/finalize.rs:5602) L1053; `park_mixed_numpr_onto_trailing_empty_pure_del` (comparer/finalize.rs:5742) L1055; `last_pure_del_inherit_prev_jc` (comparer/finalize.rs:5842) L1057; `promote_live_jc_from_pprchange_on_body_mix` (comparer/finalize.rs:6511) L1059; `park_jc_on_first_short_title_mix_from_body` (comparer/finalize.rs:13103) L1063; `promote_heading_spacing_from_pprchange_on_last_mix` (comparer/finalize.rs:6426) L1066; `ensure_empty_pprchange_on_live_heading_spacing` (comparer/finalize.rs:13215) L1069; `ensure_empty_pprchange_on_eq_with_live_jc` (comparer/finalize.rs:13403) L1072; `strip_empty_pprchange_on_mix_with_live_jc` (comparer/finalize.rs:5932) L1080; `end_para_classification_cache` (comparer/finalize.rs:57) L1088; `relocate_title_page_last_empty_after_pure_dels` (comparer/finalize.rs:3612) L1116; `fold_short_list_label_into_empty_pure_del` (comparer/finalize.rs:1702) L1119; `promote_live_numpr_on_pure_d_from_pprchange` (comparer/finalize.rs:1866) L1122; `strip_trailing_bare_empty_after_pure_i_dominant` (comparer/finalize.rs:1787) L1132; `split_head_short_title_long_del_mix` (comparer/finalize.rs:4829) L1137; `fold_leading_ins_from_mix_into_preceding_pure_del` (comparer/finalize.rs:4950) L1140; `peel_trailing_ins_from_mix_into_following_pure_del` (comparer/finalize.rs:12041) L1143; `peel_trailing_del_from_mix_into_following_pure_ins` (comparer/finalize.rs:12261) L1145; `strip_leading_del_echoing_prev_pure_i` (comparer/finalize.rs:13481) L1148; `residual_short_label_zip_with_settings` (comparer/finalize.rs:6925) L1152; `free_mesh_shared_title_token_in_mix` (comparer/finalize.rs:2531) L1155; `free_mesh_bookended_ins_del` (comparer/finalize.rs:14051) L1158; `free_mesh_wholesale_body_mix` (comparer/finalize.rs:13611) L1165; `peel_trailing_for_word_onto_next_mix` (comparer/finalize.rs:14359) L1174; `rotate_ins_mark_del_only_paragraph` (comparer/finalize.rs:4576) L1180; `restamp_stranded_del_mark_onto_del_only_paragraph` (comparer/finalize.rs:4734) L1184; `strip_list_layout_from_mid_pure_del` (comparer/finalize.rs:2421) L1187; `repair_borrowed_sentence_period` (comparer/finalize.rs:12834) L1190; `restore_echoed_heading_deletions_before_table` (comparer/finalize.rs:12979) L1193; `closing_mark_takes_revised_properties` (comparer/finalize.rs:3158) L1204; `strip_propertyless_ppr` (comparer/finalize.rs:6003) L1216; `strip_unrecorded_word_defaults` (comparer/finalize.rs:6077) L1220; `align_word_table_and_comment_chrome` (comparer/finalize.rs:6215) L1223; `diff_inside_replaced_text_boxes` (comparer/textbox.rs:33) L1228; `hoist_hyperlinks_out_of_revisions` (comparer/finalize.rs:14594) L1235; `enforce_deleted_text_kinds` (comparer/finalize.rs:14834) L1239
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.collect`, `.copied`, `.extend`, `.find_map`, `.find`, `.insert`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.join`, `.last`, `.map`, `.name`, `.or_else`, `.rev`, `.split_whitespace`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `Ok`, `Some`, `W::body`, `W::document`, `W::id`, `W::name`, `W::p_pr`, `W::sect_pr`, `W::val`, `crate::namespaces::MC::ns`, `crate::namespaces::PT::default_line`, `crate::namespaces::PT::has_default_table_style`, `crate::namespaces::PT::name`, `default_line`, `geometry_of`, `last_sect`, `std::collections::BTreeSet::new`, `std::collections::HashMap::new`, `std::collections::HashSet::new`
- **Macros:** `matches!`
- **Called by:** `compare_bodies_faithful_with_notes` (comparer/mod.rs:173); `compare_documents_impl` (document_comparer.rs:6501)

#### `WmlComparerSettings::new`  (L1424-1429, pub)
`pub fn new(mode: CompareMode) -> Self`

- **Does:** The preset for `mode`.
- **Calls (crate):** `<WmlComparerSettings as Default>::default` (comparer/mod.rs:1475) L1426; `WmlComparerSettings::powertools_faithful` (comparer/mod.rs:1464) L1427
- **Called by:** none found in src

#### `WmlComparerSettings::with_author`  (L1433-1436, pub)
`pub fn with_author(mut self, author: impl Into<String>) -> Self`

- **Does:** Set the `w:author` stamped on every revision.
- **Calls (std/external/unresolved):** `.into`
- **Called by:** none found in src

#### `WmlComparerSettings::with_date`  (L1440-1443, pub)
`pub fn with_date(mut self, date: impl Into<String>) -> Self`

- **Does:** Set the `w:date` stamped on every revision (ISO-8601).
- **Calls (std/external/unresolved):** `.into`
- **Called by:** none found in src

#### `WmlComparerSettings::with_detail_threshold`  (L1447-1450, pub)
`pub fn with_detail_threshold(mut self, threshold: f64) -> Self`

- **Does:** Set [`Self::detail_threshold`].
- **Calls:** none
- **Called by:** none found in src

#### `WmlComparerSettings::with_input_limits`  (L1454-1457, pub)
`pub fn with_input_limits(mut self, limits: InputLimits) -> Self`

- **Does:** Set the budget the inputs are admitted under ([`Self::input_limits`]).
- **Calls:** none
- **Called by:** none found in src

#### `WmlComparerSettings::powertools_faithful`  (L1464-1471, pub)
`pub fn powertools_faithful() -> Self`

- **Does:** The PowerTools-faithful preset: coarse paragraph fallback (`detail_threshold 0.15`, the C# library default) and none of the Word-visual alignment passes.
- **Calls (crate):** `<WmlComparerSettings as Default>::default` (comparer/mod.rs:1475) L1469
- **Called by:** `comparer_settings` (bin/jubarte.rs:2070); `break_payload_identity_regressions::break_defaults_match_but_authored_kind_and_clearance_do_not` (comparer/atomize.rs:703); `row_nontext_ownership_regressions::public_compare_keeps_unchanged_tab_and_break_cells_when_text_moves_into_their_row` (comparer/finalize.rs:23873); `coverage_real_revision_merge_matrix_tests::exercise` (comparer/finalize.rs:24959); `coverage_real_revision_merge_matrix_tests::public_cell_replacement_keeps_both_source_paragraph_properties_at_a_blank_tail` (comparer/finalize.rs:25258); `coverage_real_revision_merge_matrix_tests::public_compare_multi_original_blank_tail_recovers_complete_clean_sources` (comparer/finalize.rs:26091); `coverage_real_revision_merge_matrix_tests::public_two_original_one_revised_windows_keep_complete_sources_after_all_word_phases` (comparer/finalize.rs:26498); `coverage_real_revision_merge_matrix_tests::repeated_authored_prefixes_keep_deleted_paragraphs_before_their_unchanged_table` (comparer/finalize.rs:26636); `coverage_real_revision_merge_matrix_tests::unshared_wrap_gap_keeps_each_sources_preserved_terminal_space` (comparer/finalize.rs:26742); `coverage_real_revision_merge_matrix_tests::whole_story_title_label_heading_and_list_boundaries_keep_source_ownership` (comparer/finalize.rs:27034); `coverage_real_revision_merge_matrix_tests::public_terminal_field_math_and_break_boundaries_preserve_all_source_owners` (comparer/finalize.rs:27484); `coverage_real_revision_merge_matrix_tests::public_cross_paragraph_field_ends_keep_exact_source_instruction_and_cache_owners` (comparer/finalize.rs:27706); +24 more

#### `<WmlComparerSettings as Default>::default`  (L1475-1515, impl Default)
`fn default() -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `InputLimits::compare` (admission.rs:75) L1511
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.to_string`
- **Called by:** `comparer_settings` (bin/jubarte.rs:2070); `break_payload_identity_regressions::break_defaults_match_but_authored_kind_and_clearance_do_not` (comparer/atomize.rs:703); `coverage_atomizer_owned_boundaries::note_definition_owns_every_character_and_mark_but_the_part_is_a_path_stop` (comparer/atomize.rs:827); `coverage_atomizer_owned_boundaries::original_deletion_salt_follows_its_owner_and_never_salts_revised_or_inserted_content` (comparer/atomize.rs:918); `coverage_atomizer_owned_boundaries::tracked_payload_status_and_paragraph_mark_status_have_independent_owners` (comparer/atomize.rs:973); `coverage_atomizer_owned_boundaries::stamped_hash_is_authoritative_while_fresh_text_normalization_is_option_owned` (comparer/atomize.rs:1030); `coverage_atomizer_field_chain_boundaries::field_atoms` (comparer/atomize.rs:1065); `coverage_atomizer_field_chain_boundaries::fields_without_cached_results_still_own_their_complete_nested_instruction_chain` (comparer/atomize.rs:1080); `coverage_atomizer_field_chain_boundaries::header_and_footer_roots_stop_paths_and_preserve_each_whitespace_boundary` (comparer/atomize.rs:1123); `coverage_atomizer_leaf_stamp_boundaries::opaque_leaf_stamps_salt_original_deletions_once_and_preserve_the_authored_payload` (comparer/atomize.rs:1213); `public_option_source_ownership_tests::public_boundary_shifts_preserve_all_source_owners_with_lexical_options` (comparer/cross_para.rs:2705); `public_option_source_ownership_tests::public_comparison_options_preserve_reordered_paragraphs_formats_and_note_owners` (comparer/cross_para.rs:2775); +181 more

#### `canonicalize_attr_order`  (L1521-1541)
`fn canonicalize_attr_order(dom: &mut Dom, root: NodeId)`

- **Does:** Rewrite attributes into local-name order so two sections with the same properties compare equal.
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L1522; `Dom::attributes` (xmllinq/mod.rs:885) L1524; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1529,1530; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1531; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1535,1538
- **Calls (std/external/unresolved):** `.clone`, `.cmp`, `.collect`, `.iter`, `.len`, `.map`, `.sort_by`, `.then_with`, `Some`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `cols_is_word_default`  (L1546-1570)
`fn cols_is_word_default(dom: &Dom, cols: NodeId) -> bool`

- **Does:** True for a `w:cols` that states only Word's defaults: one column, 720 twips between columns, equal widths, no children.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1547; `Dom::attributes` (xmllinq/mod.rs:885) L1550; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1553,1559; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1554,1555
- **Calls (std/external/unresolved):** `.as_str`, `.is_empty`, `.starts_with`
- **Macros:** `matches!`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `word_default_sectpr`  (L1576-1612)
`fn word_default_sectpr(dom: &mut Dom, base: NodeId) -> NodeId`

- **Does:** Word's default section, as Word writes it for a document that has no `w:sectPr`: Letter, one-inch margins, half-inch header and footer, one column, 18pt line pitch.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L1578,1598; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1600,1609; `Dom::add` (xmllinq/mod.rs:1058) L1602; `Dom::element` (xmllinq/mod.rs:655) L1605,1607; `Dom::attribute` (xmllinq/mod.rs:874) L1606
- **Calls (std/external/unresolved):** `.is_some`, `Some`, `W::name`, `W::sect_pr`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `CompareContext::new`  (L1633-1638, pub)
`pub fn new(settings: WmlComparerSettings) -> Self`

- **Does:** Build a fresh context with the supplied settings and an empty [`ComparisonLog`].
- **Calls (crate):** `ComparisonLog::new` (comparison_log.rs:43) L1636
- **Called by:** none found in src

#### `final_empty_paragraphs_paired`  (L1642-1661)
`fn final_empty_paragraphs_paired(dom: &Dom, seqs: &[atoms::CorrelatedSequence]) -> bool`

- **Does:** The last sequence pairs the two stories' final paragraphs, both empty.
- **Calls (crate):** `story_closing_paragraph` (comparer/lcs.rs:3506) L1651; `Dom::descendants` (xmllinq/mod.rs:668) L1653; `Dom::value_str` (xmllinq/mod.rs:1179) L1655
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.as_deref`, `.is_empty`, `.is_some_and`, `.iter`, `.last`, `.trim`, `Some`, `W::t`, `empty_final`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

## Test functions (5)

- `body_section_source_ownership_tests::source_view` (L1669): _no doc comment_
- `body_section_source_ownership_tests::source` (L1710): _no doc comment_
- `body_section_source_ownership_tests::faithful_final_body_sections_never_become_paragraph_breaks_even_with_identical_authored_geometry` (L1726): _no doc comment_
- `body_section_source_ownership_tests::final_section_change_history_keeps_its_owner_and_identical_genuine_breaks` (L1775): _no doc comment_
- `body_section_source_ownership_tests::authored_body_paragraph_section_before_an_ending_table_is_never_the_final_body_owner` (L1913): _no doc comment_
