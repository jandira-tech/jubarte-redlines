# Functions and methods in `src/comparer/mod.rs`

16 non-test, 5 test. Part of the [symbol manifest](../README.md).

#### `compare_bodies`  (L85-95, pub)
`pub fn compare_bodies( dom: &mut Dom, body1: NodeId, body2: NodeId, settings: &WmlComparerSettings, ) -> NodeId`

- **Does:** Compare two content-parent bodies (already accepted/clean) and produce a redline `<w:document>` node in `dom`.
- **Calls (crate):** `create_comparison_unit_atom_list` (comparer/atomize.rs:307) L91,92; `correlate_atoms` (comparer/lcs.rs:69) L93; `produce_document` (comparer/produce.rs:65) L94
- **Called by:** none found in src

#### `compare_bodies_faithful`  (L115-132, pub)
`pub fn compare_bodies_faithful( dom: &mut Dom, source_root1: NodeId, source_root2: NodeId, body1: NodeId, body2: NodeId, settings: &WmlComparerSettings, ) -> NodeId`

- **Does:** M4.I — faithful end-to-end body compare: the real WmlComparer produce path (atomize → hash → `GetComparisonUnitList` → LCS → `MarkRows` → `Flatten` → `AssembleAncestorUnids` → `CoalesceRecurse` → `MarkContent` → `IgnorePt14` → `Conjoin` → `FixUpRevisionIds` → `RemoveScratch`).
- **Calls (crate):** `compare_bodies_faithful_with_notes` (comparer/mod.rs:179) L123
- **Called by:** `coverage_real_revision_merge_matrix_tests::exercise` (comparer/finalize.rs:24981); `coverage_real_revision_merge_matrix_tests::public_two_original_one_revised_windows_keep_complete_sources_after_all_word_phases` (comparer/finalize.rs:26519); `coverage_real_revision_merge_matrix_tests::repeated_authored_prefixes_keep_deleted_paragraphs_before_their_unchanged_table` (comparer/finalize.rs:26657); `coverage_real_revision_merge_matrix_tests::unshared_wrap_gap_keeps_each_sources_preserved_terminal_space` (comparer/finalize.rs:26763); `paired_row_mark_ownership_tests::public_empty_table_comparison_preserves_complete_authored_rows_and_pilcrows` (comparer/finalize.rs:29031); `closing_authored_section_boundary_tests::public_added_and_unchanged_authored_sections_keep_complete_source_bodies` (comparer/finalize.rs:29400); `body_section_source_ownership_tests::faithful_final_body_sections_never_become_paragraph_breaks_even_with_identical_authored_geometry` (comparer/mod.rs:1744); `body_section_source_ownership_tests::final_section_change_history_keeps_its_owner_and_identical_genuine_breaks` (comparer/mod.rs:1793); `body_section_source_ownership_tests::authored_body_paragraph_section_before_an_ending_table_is_never_the_final_body_owner` (comparer/mod.rs:1931); `content_control_source_order_regressions::introducing_or_removing_a_control_keeps_replacement_words_in_both_source_positions` (comparer/produce.rs:1957); `content_control_source_order_regressions::equal_control_ancestry_keeps_its_wrapper_and_authored_properties` (comparer/produce.rs:1996); `repeated_control_tag_regressions::repeated_tag_outer_wrapper_never_replaces_retained_inner_control_metadata` (comparer/produce.rs:2347); +2 more

#### `compare_bodies_faithful_with_notes`  (L179-198, pub)
`pub fn compare_bodies_faithful_with_notes( dom: &mut Dom, source_root1: NodeId, source_root2: NodeId, body1: NodeId, body2: NodeId, settings: &WmlComparerSettings, notes: Option<&mut NotesContext>, ) -> NodeId`

- **Does:** [`compare_bodies_faithful`] with a [`NotesContext`]: when `Some`, footnote/endnote definitions are processed per correlated reference (`ProcessFootnoteEndnote`) and the output notes parts are renumbered (`RectifyFootnoteEndnoteIds`) before the pipeline ends.
- **Calls (crate):** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215) L188
- **Calls (std/external/unresolved):** `.unwrap_or_else`
- **Macros:** `panic!`
- **Called by:** `compare_bodies_faithful` (comparer/mod.rs:115)

#### `try_compare_bodies_faithful_with_notes`  (L215-1270, pub)
`pub fn try_compare_bodies_faithful_with_notes( dom: &mut Dom, source_root1: NodeId, source_root2: NodeId, body1: NodeId, body2: NodeId, settings: &WmlComparerSettings, notes: Option<&mut NotesContext>, ) -> Result<NodeId, footnotes::RectifyError>`

- **Does:** [`compare_bodies_faithful_with_notes`] that returns the notes-layout failure as a `super::footnotes::RectifyError` instead of panicking; the package-level comparer uses this so a malformed notes part is an `Err`, never an abort of the host.
- **Calls (crate):** `last_body_para_spacing` (comparer/finalize.rs:3463) L246; `closing_paragraph_properties` (comparer/finalize.rs:3093) L250,251; `paragraph_alignments` (comparer/finalize.rs:13391) L255; `unaligned_paragraph_texts` (comparer/finalize.rs:13336) L257; `Dom::element` (xmllinq/mod.rs:655) L260,304,332,334,379,390...; `Dom::descendants` (xmllinq/mod.rs:668) L261,288,482,856; `word_default_sectpr` (comparer/mod.rs:1594) L271; `Dom::new_element` (xmllinq/mod.rs:478) L276,377,421,432,464,755...; `Dom::attributes` (xmllinq/mod.rs:899) L277,772; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L278,321,402,422,427,455...; `Dom::elements` (xmllinq/mod.rs:640) L293,351,483,497,863; `Dom::attribute` (xmllinq/mod.rs:874) L295,311,391,452,453,783...; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L298,305,345,380,435,565...; `Dom::add` (xmllinq/mod.rs:1074) L299,324,357,381,436,439...; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L318,399; `pg_num_type_is_implicit_decimal` (comparer/finalize.rs:6053) L342; `Dom::name` (xmllinq/mod.rs:526) L352; `XName::local_name` (xmllinq/mod.rs:208) (by name) L353; `Dom::add_before_self` (xmllinq/mod.rs:1130) L356,467; `Dom::remove` (xmllinq/mod.rs:1118) L393,410,859,864; `cols_is_word_default` (comparer/mod.rs:1564) L409; `canonicalize_attr_order` (comparer/mod.rs:1539) L417; `sectpr_identity` (comparer/finalize.rs:11609) L418,484; `resolve_alternate_content` (comparer/finalize.rs:1301) L509,510; `sanitize_sdt_properties` (comparer/finalize.rs:1354) L515,516; `coalesce_all_paragraphs` (comparer/finalize.rs:1503) L525,526,800; `flatten_tracked_insertions_stamped` (comparer/finalize.rs:10220) L539; `pending_deletion_texts` (comparer/finalize.rs:10981) L540; `flatten_tracked_deletions` (comparer/finalize.rs:11035) L541,552; `assign_to_all_elements` (unid.rs:60) L554,555; `element_has_tracked_revisions` (revision_processor.rs:89) L561,562; `accept_revisions_document` (revision_processor.rs:524) L566,618,619; `hash_block_level_content` (comparer/preprocess.rs:1317) L567,575,586,594,642,651; `reject_revisions_document` (revision_processor.rs:751) L585; `revised_two_blank_table_titles` (comparer/finalize.rs:3889) L622; `ParagraphPropertySources::capture` (comparer/finalize.rs:13105) L626; `add_sha1_hash_to_block_level_content` (comparer/preprocess.rs:1259) L663,669; `create_comparison_unit_atom_list` (comparer/atomize.rs:307) L676,677; `get_comparison_unit_list` (comparer/units.rs:140) L678,679; `detect_unrelated_sources_word_mode` (comparer/lcs.rs:6062) L702; `lcs` (comparer/lcs.rs:10084) L707,711; `detect_unrelated_sources` (comparer/lcs.rs:5620) L710; `pair_story_final_marks` (comparer/lcs.rs:5786) L715; `Dom::name_is` (xmllinq/mod.rs:535) L718; `restream_cross_paragraph_regions` (comparer/cross_para.rs:138) L719; `final_empty_paragraphs_paired` (comparer/mod.rs:1660) L726; `unit_closes_story` (comparer/lcs.rs:3501) L736; `promote_skip_ahead_equals_with_source` (comparer/moves.rs:270) L740; `mark_rows_as_deleted_or_inserted` (comparer/lcs_table.rs:705) L743; `flatten_to_comparison_unit_atom_list` (comparer/produce.rs:225) L745; `detect_moves_in_atom_list` (comparer/moves.rs:470) L748; `detect_format_changes_in_atom_list` (comparer/formatchg.rs:383) L749; `assemble_ancestor_unids` (comparer/produce.rs:470) L750; `produce_new_wml_markup_from_correlated_sequence` (comparer/produce.rs:1768) L752; `XNamespace::xmlns` (xmllinq/mod.rs:122) L758,766; `XNamespace::name` (xmllinq/mod.rs:134) L767; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L777; `XNamespace::namespace_name` (xmllinq/mod.rs:140) L777; `mark_content_as_deleted_or_inserted` (comparer/finalize.rs:517) L799; `ignore_pt14_namespace` (comparer/finalize.rs:862) L801; `process_footnote_endnote` (comparer/footnotes.rs:1115) L807; `rectify_footnote_endnote_ids` (comparer/footnotes.rs:673) L812; `conjoin_paragraph_marks` (comparer/finalize.rs:745) L829; `merged_closing_carrier_takes_source_properties` (comparer/finalize.rs:3122) L831; `fix_up_revision_ids` (comparer/finalize.rs:768) L840,1261; `wml_order_elements_per_standard` (comparer/finalize.rs:10488) L845; `fix_up_footnotes_endnotes_with_custom_markers` (comparer/footnotes.rs:887) L880; `simplify_move_markup_to_del_ins` (comparer/finalize.rs:6814) L884; `fix_up_doc_pr_ids` (comparer/fixups.rs:32) L893,1266; `fix_up_shape_ids` (comparer/fixups.rs:54) L894,1267; `fix_up_shape_type_ids` (comparer/fixups.rs:75) L895,1268; `convert_stamped_predeletes` (comparer/finalize.rs:11141) L899; `convert_stamped_preins` (comparer/finalize.rs:10287) L900; `remove_powertools_scratch_markup` (comparer/finalize.rs:1060) L902; `normalize_universal_measures` (comparer/finalize.rs:11490) L906; `move_paragraph_properties_first` (comparer/finalize.rs:1566) L909; `capture_deleted_demo_title_properties` (comparer/finalize.rs:12584) L911; `strip_redundant_demo_default_spacing` (comparer/finalize.rs:2308) L925; `strip_redundant_normal_pstyle_and_bidi` (comparer/finalize.rs:2404) L932; `normalize_incomplete_spacing` (comparer/finalize.rs:2218) L934; `ensure_pure_i_list_snug_spacing` (comparer/finalize.rs:2113) L936; `unwrap_hyperlinks_to_styled_runs` (comparer/finalize.rs:1597) L941; `unwrap_content_controls_in_pure_revisions` (comparer/finalize.rs:1454) L944; `strip_trailing_empty_pure_del_mark` (comparer/finalize.rs:2951) L946; `trailing_empty_spacing_to_pprchange` (comparer/finalize.rs:3009) L948,1056; `strip_trailing_empty_pure_ins` (comparer/finalize.rs:3620) L958,1114; `fold_whitespace_pure_ins_into_following_pure_del` (comparer/finalize.rs:4252) L966,1117; `ensure_empty_pure_i_before_short_title_del` (comparer/finalize.rs:3951) L969,1120; `strip_empty_pure_ins_before_trailing_pure_dels` (comparer/finalize.rs:4103) L979,1128; `strip_last_pure_del_mark_only_ppr` (comparer/finalize.rs:5122) L982,1103; `strip_last_pure_del_mark_when_pprchange` (comparer/finalize.rs:6598) L983,1105; `fix_paragraph_mark_revision_order` (comparer/finalize.rs:1526) L988; `reorder_replacements_ins_before_del` (comparer/finalize.rs:6657) L990; `coalesce_adjacent_revisions` (comparer/finalize.rs:6764) L994; `wrap_bare_del_text_runs` (comparer/finalize.rs:10776) L998; `reorder_replaced_blocks` (comparer/finalize.rs:10080) L1002; `splice_trailing_short_pure_dels_midstream` (comparer/finalize.rs:11803) L1005; `interleave_list_cluster_after_coalesce` (comparer/finalize.rs:9881) L1008,1194; `merge_replaced_paragraphs_in` (comparer/finalize.rs:6895) L1011; `restore_short_del_before_long_ins` (comparer/finalize.rs:12185) L1019; `split_digits_ins_from_mixed_title` (comparer/finalize.rs:11959) L1021; `fold_midstream_demo_title_into_numbered_heading_with_source_properties` (comparer/finalize.rs:12624) L1024; `drop_sectpr_from_deleted_marks` (comparer/finalize.rs:10184) L1031; `drop_hoisted_sectpr_artifacts` (comparer/finalize.rs:11642) L1032; `mark_fully_revised_rows` (comparer/finalize.rs:11671) L1033; `has_default_table_style` (document_comparer.rs:3156) L1038,1039; `synthesize_table_cell_margins` (comparer/finalize.rs:11254) L1041; `ensure_default_page_size` (comparer/finalize.rs:10393) L1042; `begin_para_classification_cache` (comparer/finalize.rs:49) L1045; `last_pure_del_spacing_to_pprchange` (comparer/finalize.rs:5205) L1050; `cleanup_spacing_and_default_jc` (comparer/finalize.rs:5422) L1054; `mixed_spacing_to_following_empty` (comparer/finalize.rs:5308) L1064; `park_mixed_spacing_onto_trailing_pure_del` (comparer/finalize.rs:5607) L1066; `park_mixed_numpr_onto_trailing_empty_pure_del` (comparer/finalize.rs:5747) L1068; `last_pure_del_inherit_prev_jc` (comparer/finalize.rs:5847) L1070; `promote_live_jc_from_pprchange_on_body_mix` (comparer/finalize.rs:6516) L1072; `park_jc_on_first_short_title_mix_from_body` (comparer/finalize.rs:13148) L1075; `promote_heading_spacing_from_pprchange_on_last_mix` (comparer/finalize.rs:6431) L1084; `ensure_empty_pprchange_on_live_heading_spacing` (comparer/finalize.rs:13237) L1087; `ensure_empty_pprchange_on_eq_with_live_jc` (comparer/finalize.rs:13425) L1090; `strip_empty_pprchange_on_mix_with_live_jc` (comparer/finalize.rs:5937) L1098; `end_para_classification_cache` (comparer/finalize.rs:57) L1106; `relocate_title_page_last_empty_after_pure_dels` (comparer/finalize.rs:3653) L1134; `fold_short_list_label_into_empty_pure_del` (comparer/finalize.rs:1744) L1137; `promote_live_numpr_on_pure_d_from_pprchange` (comparer/finalize.rs:1908) L1140; `strip_trailing_bare_empty_after_pure_i_dominant` (comparer/finalize.rs:1829) L1150; `split_head_short_title_long_del_mix` (comparer/finalize.rs:4870) L1155; `fold_leading_ins_from_mix_into_preceding_pure_del` (comparer/finalize.rs:4991) L1158; `peel_trailing_ins_from_mix_into_following_pure_del` (comparer/finalize.rs:12046) L1161; `peel_trailing_del_from_mix_into_following_pure_ins` (comparer/finalize.rs:12266) L1163; `strip_leading_del_echoing_prev_pure_i` (comparer/finalize.rs:13503) L1166; `residual_short_label_zip_with_settings` (comparer/finalize.rs:6930) L1170; `free_mesh_shared_title_token_in_mix` (comparer/finalize.rs:2573) L1173; `free_mesh_bookended_ins_del` (comparer/finalize.rs:14073) L1176; `free_mesh_wholesale_body_mix` (comparer/finalize.rs:13633) L1183; `peel_trailing_for_word_onto_next_mix` (comparer/finalize.rs:14381) L1192; `rotate_ins_mark_del_only_paragraph` (comparer/finalize.rs:4617) L1198; `restamp_stranded_del_mark_onto_del_only_paragraph` (comparer/finalize.rs:4775) L1202; `strip_list_layout_from_mid_pure_del` (comparer/finalize.rs:2463) L1205; `repair_borrowed_sentence_period` (comparer/finalize.rs:12839) L1208; `restore_echoed_heading_deletions_before_table` (comparer/finalize.rs:12984) L1211; `closing_mark_takes_revised_properties` (comparer/finalize.rs:3200) L1222; `strip_propertyless_ppr` (comparer/finalize.rs:6008) L1234; `strip_unrecorded_word_defaults` (comparer/finalize.rs:6082) L1238; `align_word_table_and_comment_chrome` (comparer/finalize.rs:6220) L1241; `diff_inside_replaced_text_boxes` (comparer/textbox.rs:33) L1246; `hoist_hyperlinks_out_of_revisions` (comparer/finalize.rs:14616) L1253; `enforce_deleted_text_kinds` (comparer/finalize.rs:14856) L1257
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.collect`, `.copied`, `.extend`, `.find_map`, `.find`, `.insert`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.join`, `.last`, `.map`, `.name`, `.or_else`, `.rev`, `.split_whitespace`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `Ok`, `Some`, `W::body`, `W::document`, `W::id`, `W::name`, `W::p_pr`, `W::sect_pr`, `W::val`, `crate::namespaces::MC::ns`, `crate::namespaces::PT::default_line`, `crate::namespaces::PT::has_default_table_style`, `crate::namespaces::PT::name`, `default_line`, `finalize::ParagraphPropertySources::default`, `geometry_of`, `last_sect`, `std::collections::BTreeSet::new`, `std::collections::HashMap::new`, `std::collections::HashSet::new`
- **Macros:** `matches!`
- **Called by:** `compare_bodies_faithful_with_notes` (comparer/mod.rs:179); `compare_documents_impl` (document_comparer.rs:6521)

#### `WmlComparerSettings::new`  (L1442-1447, pub)
`pub fn new(mode: CompareMode) -> Self`

- **Does:** The preset for `mode`.
- **Calls (crate):** `<WmlComparerSettings as Default>::default` (comparer/mod.rs:1493) L1444; `WmlComparerSettings::powertools_faithful` (comparer/mod.rs:1482) L1445
- **Called by:** none found in src

#### `WmlComparerSettings::with_author`  (L1451-1454, pub)
`pub fn with_author(mut self, author: impl Into<String>) -> Self`

- **Does:** Set the `w:author` stamped on every revision.
- **Calls (std/external/unresolved):** `.into`
- **Called by:** none found in src

#### `WmlComparerSettings::with_date`  (L1458-1461, pub)
`pub fn with_date(mut self, date: impl Into<String>) -> Self`

- **Does:** Set the `w:date` stamped on every revision (ISO-8601).
- **Calls (std/external/unresolved):** `.into`
- **Called by:** none found in src

#### `WmlComparerSettings::with_detail_threshold`  (L1465-1468, pub)
`pub fn with_detail_threshold(mut self, threshold: f64) -> Self`

- **Does:** Set [`Self::detail_threshold`].
- **Calls:** none
- **Called by:** none found in src

#### `WmlComparerSettings::with_input_limits`  (L1472-1475, pub)
`pub fn with_input_limits(mut self, limits: InputLimits) -> Self`

- **Does:** Set the budget the inputs are admitted under ([`Self::input_limits`]).
- **Calls:** none
- **Called by:** none found in src

#### `WmlComparerSettings::powertools_faithful`  (L1482-1489, pub)
`pub fn powertools_faithful() -> Self`

- **Does:** The PowerTools-faithful preset: coarse paragraph fallback (`detail_threshold 0.15`, the C# library default) and none of the Word-visual alignment passes.
- **Calls (crate):** `<WmlComparerSettings as Default>::default` (comparer/mod.rs:1493) L1487
- **Called by:** `comparer_settings` (bin/jubarte.rs:2837); `break_payload_identity_regressions::break_defaults_match_but_authored_kind_and_clearance_do_not` (comparer/atomize.rs:703); `row_nontext_ownership_regressions::public_compare_keeps_unchanged_tab_and_break_cells_when_text_moves_into_their_row` (comparer/finalize.rs:23895); `coverage_real_revision_merge_matrix_tests::exercise` (comparer/finalize.rs:24981); `coverage_real_revision_merge_matrix_tests::public_cell_replacement_keeps_both_source_paragraph_properties_at_a_blank_tail` (comparer/finalize.rs:25280); `coverage_real_revision_merge_matrix_tests::public_compare_multi_original_blank_tail_recovers_complete_clean_sources` (comparer/finalize.rs:26112); `coverage_real_revision_merge_matrix_tests::public_two_original_one_revised_windows_keep_complete_sources_after_all_word_phases` (comparer/finalize.rs:26519); `coverage_real_revision_merge_matrix_tests::repeated_authored_prefixes_keep_deleted_paragraphs_before_their_unchanged_table` (comparer/finalize.rs:26657); `coverage_real_revision_merge_matrix_tests::unshared_wrap_gap_keeps_each_sources_preserved_terminal_space` (comparer/finalize.rs:26763); `coverage_real_revision_merge_matrix_tests::whole_story_title_label_heading_and_list_boundaries_keep_source_ownership` (comparer/finalize.rs:27055); `coverage_real_revision_merge_matrix_tests::public_terminal_field_math_and_break_boundaries_preserve_all_source_owners` (comparer/finalize.rs:27505); `coverage_real_revision_merge_matrix_tests::public_cross_paragraph_field_ends_keep_exact_source_instruction_and_cache_owners` (comparer/finalize.rs:27727); +24 more

#### `<WmlComparerSettings as Default>::default`  (L1493-1533, impl Default)
`fn default() -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `InputLimits::compare` (admission.rs:87) L1529
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.to_string`
- **Called by:** `comparer_settings` (bin/jubarte.rs:2837); `break_payload_identity_regressions::break_defaults_match_but_authored_kind_and_clearance_do_not` (comparer/atomize.rs:703); `coverage_atomizer_owned_boundaries::note_definition_owns_every_character_and_mark_but_the_part_is_a_path_stop` (comparer/atomize.rs:827); `coverage_atomizer_owned_boundaries::original_deletion_salt_follows_its_owner_and_never_salts_revised_or_inserted_content` (comparer/atomize.rs:918); `coverage_atomizer_owned_boundaries::tracked_payload_status_and_paragraph_mark_status_have_independent_owners` (comparer/atomize.rs:973); `coverage_atomizer_owned_boundaries::stamped_hash_is_authoritative_while_fresh_text_normalization_is_option_owned` (comparer/atomize.rs:1030); `coverage_atomizer_field_chain_boundaries::field_atoms` (comparer/atomize.rs:1065); `coverage_atomizer_field_chain_boundaries::fields_without_cached_results_still_own_their_complete_nested_instruction_chain` (comparer/atomize.rs:1080); `coverage_atomizer_field_chain_boundaries::header_and_footer_roots_stop_paths_and_preserve_each_whitespace_boundary` (comparer/atomize.rs:1123); `coverage_atomizer_leaf_stamp_boundaries::opaque_leaf_stamps_salt_original_deletions_once_and_preserve_the_authored_payload` (comparer/atomize.rs:1213); `public_option_source_ownership_tests::public_boundary_shifts_preserve_all_source_owners_with_lexical_options` (comparer/cross_para.rs:2702); `public_option_source_ownership_tests::public_comparison_options_preserve_reordered_paragraphs_formats_and_note_owners` (comparer/cross_para.rs:2772); +184 more

#### `canonicalize_attr_order`  (L1539-1559)
`fn canonicalize_attr_order(dom: &mut Dom, root: NodeId)`

- **Does:** Rewrite attributes into local-name order so two sections with the same properties compare equal.
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L1540; `Dom::attributes` (xmllinq/mod.rs:899) L1542; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1547,1548; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1549; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1553,1556
- **Calls (std/external/unresolved):** `.clone`, `.cmp`, `.collect`, `.iter`, `.len`, `.map`, `.sort_by`, `.then_with`, `Some`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `cols_is_word_default`  (L1564-1588)
`fn cols_is_word_default(dom: &Dom, cols: NodeId) -> bool`

- **Does:** True for a `w:cols` that states only Word's defaults: one column, 720 twips between columns, equal widths, no children.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1565; `Dom::attributes` (xmllinq/mod.rs:899) L1568; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1571,1577; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1572,1573
- **Calls (std/external/unresolved):** `.as_str`, `.is_empty`, `.starts_with`
- **Macros:** `matches!`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `word_default_sectpr`  (L1594-1630)
`fn word_default_sectpr(dom: &mut Dom, base: NodeId) -> NodeId`

- **Does:** Word's default section, as Word writes it for a document that has no `w:sectPr`: Letter, one-inch margins, half-inch header and footer, one column, 18pt line pitch.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L1596,1616; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1618,1627; `Dom::add` (xmllinq/mod.rs:1074) L1620; `Dom::element` (xmllinq/mod.rs:655) L1623,1625; `Dom::attribute` (xmllinq/mod.rs:874) L1624
- **Calls (std/external/unresolved):** `.is_some`, `Some`, `W::name`, `W::sect_pr`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `CompareContext::new`  (L1651-1656, pub)
`pub fn new(settings: WmlComparerSettings) -> Self`

- **Does:** Build a fresh context with the supplied settings and an empty [`ComparisonLog`].
- **Calls (crate):** `ComparisonLog::new` (comparison_log.rs:43) L1654
- **Called by:** none found in src

#### `final_empty_paragraphs_paired`  (L1660-1679)
`fn final_empty_paragraphs_paired(dom: &Dom, seqs: &[atoms::CorrelatedSequence]) -> bool`

- **Does:** The last sequence pairs the two stories' final paragraphs, both empty.
- **Calls (crate):** `story_closing_paragraph` (comparer/lcs.rs:3506) L1669; `Dom::descendants` (xmllinq/mod.rs:668) L1671; `Dom::value_str` (xmllinq/mod.rs:1195) L1673
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.as_deref`, `.is_empty`, `.is_some_and`, `.iter`, `.last`, `.trim`, `Some`, `W::t`, `empty_final`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

## Test functions (5)

- `body_section_source_ownership_tests::source_view` (L1687): _no doc comment_
- `body_section_source_ownership_tests::source` (L1728): _no doc comment_
- `body_section_source_ownership_tests::faithful_final_body_sections_never_become_paragraph_breaks_even_with_identical_authored_geometry` (L1744): _no doc comment_
- `body_section_source_ownership_tests::final_section_change_history_keeps_its_owner_and_identical_genuine_breaks` (L1793): _no doc comment_
- `body_section_source_ownership_tests::authored_body_paragraph_section_before_an_ending_table_is_never_the_final_body_owner` (L1931): _no doc comment_
