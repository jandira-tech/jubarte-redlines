# Functions and methods in `src/xmllinq/mod.rs`

104 non-test, 5 test. Part of the [symbol manifest](../README.md).

#### `intern_str`  (L75-91)
`fn intern_str(s: &str) -> Arc<str>`

- **Does:** Return a shared interned `Arc<str>` for `s`, allocating only on first sight.
- **Calls (std/external/unresolved):** `.borrow_mut`, `.borrow`, `.clone`, `.get`, `.insert`, `.len`, `.with`, `Arc::from`
- **Called by:** `XNamespace::get` (xmllinq/mod.rs:108); `XName::get` (xmllinq/mod.rs:180)

#### `XNamespace::get`  (L108-112, pub)
`pub fn get(namespace_name: &str) -> XNamespace`

- **Does:** Construct a namespace, interning `namespace_name` for sharing across the thread (a no-op once the pool is saturated).
- **Calls (crate):** `intern_str` (xmllinq/mod.rs:75) L110
- **Called by:** `fix_strict_validity_artifacts` (comparer/finalize.rs:11321); `coverage_final_batch_finalize_tests::inherited_invalidity_repairs_only_wml_schema_defects` (comparer/finalize.rs:24561); `deep_lcs_source_conservation_tests::opaque_object_and_foreign_namespace_payloads_are_contentful_without_text` (comparer/lcs.rs:17317); `package_tree` (debug/diff.rs:303); `holds_watermark` (edit/watermark.rs:156); `XNamespace::none` (xmllinq/mod.rs:116); `XNamespace::xmlns` (xmllinq/mod.rs:122); `XNamespace::xml` (xmllinq/mod.rs:128); `XName::get` (xmllinq/mod.rs:180); `resolve` (xmllinq/parse.rs:644); `tag_policy_tests::a_word_gfxdata_attribute_is_admitted` (xmllinq/parse.rs:1046); `tests::word_2016_plus_namespaces_serialize_with_word_prefixes` (xmllinq/serialize.rs:847); +2 more

#### `XNamespace::none`  (L116-118, pub)
`pub fn none() -> XNamespace`

- **Does:** The empty namespace (no URI).
- **Calls (crate):** `XNamespace::get` (xmllinq/mod.rs:108) L117
- **Called by:** `choice_requires_understood` (comparer/finalize.rs:1437); `fix_strict_validity_artifacts` (comparer/finalize.rs:11321); `coverage_final_batch_finalize_tests::inherited_invalidity_repairs_only_wml_schema_defects` (comparer/finalize.rs:24561); `xml_structure_namespace_boundary_tests::structure_hash_keeps_mc_required_namespace_ownership_while_omitting_payload` (xmllinq/serialize.rs:1035)

#### `XNamespace::xmlns`  (L122-124, pub)
`pub fn xmlns() -> XNamespace`

- **Does:** The reserved `xmlns:` declarations namespace (`http://www.w3.org/2000/xmlns/`).
- **Calls (crate):** `XNamespace::get` (xmllinq/mod.rs:108) L123
- **Called by:** `declare_ignorable` (comments.rs:1143); `coalesce` (comparer/atomize.rs:557); `preserve_cloned_namespace_context` (comparer/comments.rs:1052); `ignore_pt14_namespace` (comparer/finalize.rs:819); `declare_extension_namespaces_ignorable` (comparer/finalize.rs:899); `bind_compatibility_prefixes` (comparer/finalize.rs:983); `coverage_final_batch_finalize_tests::compatibility_bindings_keep_conflicts_and_recognize_attribute_only_extensions` (comparer/finalize.rs:24621); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209); `produce_document` (comparer/produce.rs:65); `keyed_groups` (comparer/revisions.rs:88); `compare_documents_impl` (document_comparer.rs:6501); `Transaction::mark_header` (edit/watermark.rs:284); +2 more

#### `XNamespace::xml`  (L128-130, pub)
`pub fn xml() -> XNamespace`

- **Does:** The reserved `xml:` declarations namespace (`http://www.w3.org/XML/1998/namespace`).
- **Calls (crate):** `XNamespace::get` (xmllinq/mod.rs:108) L129
- **Called by:** `fill_comment_paragraph` (comments.rs:1073); `coalesce_recurse` (comparer/atomize.rs:575); `coverage_atomizer_owned_boundaries::note_definition_owns_every_character_and_mark_but_the_part_is_a_path_stop` (comparer/atomize.rs:827); `coverage_atomizer_field_chain_boundaries::header_and_footer_roots_stop_paths_and_preserve_each_whitespace_boundary` (comparer/atomize.rs:1123); `split_seg` (comparer/comments.rs:568); `deeper_boundary_tests::styled_unicode_split_preserves_properties_and_trailing_nodes` (comparer/comments.rs:2098); `coalesce_adjacent_runs` (comparer/finalize.rs:1180); `rebuild_title_free_mesh` (comparer/finalize.rs:2771); `add_revision_text_run` (comparer/finalize.rs:2821); `mesh_short_label_shared_eq_with_settings` (comparer/finalize.rs:7199); `rebuild_body_free_mesh_lcs` (comparer/finalize.rs:13829); `m460_push_eq_text` (comparer/finalize.rs:14270); +28 more

#### `XNamespace::name`  (L134-136, pub)
`pub fn name(&self, local: &str) -> XName`

- **Does:** Build the [`XName`] `{ns}local` — i.e.
- **Calls (crate):** `XName::get` (xmllinq/mod.rs:180) L135
- **Called by:** `declare_ignorable` (comments.rs:1143); `declare_extension_namespaces_ignorable` (comparer/finalize.rs:899); `bind_compatibility_prefixes` (comparer/finalize.rs:983); `fix_strict_validity_artifacts::{fn remap}` (comparer/finalize.rs:11390); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209); `Transaction::mark_header` (edit/watermark.rs:284); `tag_policy_tests::a_word_gfxdata_attribute_is_admitted` (xmllinq/parse.rs:1046)

#### `XNamespace::namespace_name`  (L140-142, pub)
`pub fn namespace_name(&self) -> &str`

- **Does:** The namespace URI as a borrowed string.
- **Calls:** none
- **Called by:** `w_vals` (append.rs:812) (?); `set_section_type` (append.rs:1325) (?); `public_memory_append_owner_boundary_tests::semantic` (append.rs:1448) (?); `carriers` (changes.rs:441) (?); `in_recorded_properties` (changes.rs:694) (?); `recorded_marks` (changes.rs:707) (?); `comment_text` (comments.rs:1045) (?); `preserve_cloned_namespace_context` (comparer/comments.rs:1052) (?); `deeper_boundary_tests::namespace_conflicts_reuse_aliases_or_skip_occupied_generated_prefixes` (comparer/comments.rs:2317) (?); `public_option_source_ownership_tests::canonical` (comparer/cross_para.rs:2479) (?); `public_option_source_ownership_tests::source_events::{fn walk}` (comparer/cross_para.rs:2513) (?); `copy_attrs` (comparer/finalize.rs:136) (?); +123 more

#### `<XNamespace as PartialEq>::eq`  (L146-152, impl PartialEq)
`fn eq(&self, other: &Self) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_ref`, `Arc::ptr_eq`
- **Called by:** none found in src

#### `<XNamespace as std::hash::Hash>::hash`  (L156-158, impl std::hash::Hash)
`fn hash<H: std::hash::Hasher>(&self, state: &mut H)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_ref`, `.hash`
- **Called by:** `<XName as std::hash::Hash>::hash` (xmllinq/mod.rs:245)

#### `<XNamespace as std::fmt::Debug>::fmt`  (L161-163, impl std::fmt::Debug)
`fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_ref`
- **Macros:** `write!`
- **Called by:** none found in src

#### `XName::get`  (L180-185, pub)
`pub fn get(local_name: &str, namespace_name: &str) -> XName`

- **Does:** Construct an expanded name in the given namespace.
- **Calls (crate):** `intern_str` (xmllinq/mod.rs:75) L182; `XNamespace::get` (xmllinq/mod.rs:108) L183
- **Called by:** `merge_namespace_declarations` (append.rs:365); `tests::ignorable_prefixes_merge_only_with_the_same_namespace` (append.rs:1417); `Package::image_no_descr` (audit.rs:558); `decorative` (audit.rs:942); `resolve` (changes.rs:349); `CommentFamily::read_meta` (comments.rs:457); `CommentFamily::thread_state` (comments.rs:672); `CommentFamily::adopt` (comments.rs:687); `CommentFamily::store` (comments.rs:798); `deeper_boundary_tests::reference_rewriting_changes_only_matching_identity_attributes` (comparer/comments.rs:2276); `deeper_boundary_tests::namespace_conflicts_reuse_aliases_or_skip_occupied_generated_prefixes` (comparer/comments.rs:2317); `deeper_boundary_tests::comments_ids_accepts_missing_durable_id_without_inventing_one` (comparer/comments.rs:2445); +38 more

#### `XName::from_clark`  (L195-205, pub)
`pub fn from_clark(expanded: &str) -> XName`

- **Does:** Parse clark notation: `"{ns}local"` becomes a qualified [`XName`], a bare `"local"` becomes one in the empty namespace.
- **Calls (crate):** `XName::get` (xmllinq/mod.rs:180) L201,203
- **Calls (std/external/unresolved):** `.filter`, `.find`, `.starts_with`, `.unwrap_or_else`
- **Macros:** `panic!`
- **Called by:** `get_changed_property_names` (comparer/formatchg.rs:217)

#### `XName::local_name`  (L208-210, pub)
`pub fn local_name(&self) -> &str`

- **Does:** The local part of the name, as a borrowed string.
- **Calls:** none
- **Called by:** `elements` (admission.rs:782) (?); `is_comment_anchor` (append.rs:417) (?); `is_annotation` (append.rs:488); `w_vals` (append.rs:812) (?); `set_section_type` (append.rs:1325) (?); `tests::section_type_sits_after_header_references` (append.rs:1398) (?); `public_memory_append_owner_boundary_tests::semantic` (append.rs:1448) (?); `decorative` (audit.rs:942) (?); `resolve` (changes.rs:349) (?); `carriers` (changes.rs:441) (?); `fold_style_pairs` (changes.rs:630) (?); `in_recorded_properties` (changes.rs:694) (?); +232 more

#### `XName::namespace`  (L213-215, pub)
`pub fn namespace(&self) -> &XNamespace`

- **Does:** The namespace half of the expanded name.
- **Calls:** none
- **Called by:** `Build::noise` (debug/diff.rs:504); `Build::namespace_note` (debug/diff.rs:564) (?); `Build::attrs` (debug/diff.rs:593) (?); `effective_chain_props` (document_comparer.rs:1696) (?); `complete_attributes` (document_comparer.rs:2054) (?)

#### `XName::namespace_name`  (L219-221, pub)
`pub fn namespace_name(&self) -> &str`

- **Does:** The namespace URI as a borrowed string.
- **Calls (crate):** `XNamespace::namespace_name` (xmllinq/mod.rs:140) L220
- **Called by:** `is_comment_anchor` (append.rs:417); `is_annotation` (append.rs:488); `w_vals` (append.rs:812) (?); `set_section_type` (append.rs:1325) (?); `public_memory_append_owner_boundary_tests::semantic` (append.rs:1448) (?); `carriers` (changes.rs:441) (?); `in_recorded_properties` (changes.rs:694) (?); `recorded_marks` (changes.rs:707) (?); `comment_text` (comments.rs:1045) (?); `preserve_cloned_namespace_context` (comparer/comments.rs:1052) (?); `deeper_boundary_tests::namespace_conflicts_reuse_aliases_or_skip_occupied_generated_prefixes` (comparer/comments.rs:2317) (?); `public_option_source_ownership_tests::canonical` (comparer/cross_para.rs:2479) (?); +137 more

#### `XName::clark`  (L226-232, pub)
`pub fn clark(&self) -> String`

- **Does:** The clark-notation form: `"{ns}local"` for a non-empty namespace, or just `"local"` for the empty one.
- **Calls (std/external/unresolved):** `.is_empty`, `.to_string`
- **Macros:** `format!`
- **Called by:** `get_changed_property_names` (comparer/formatchg.rs:217) (?); `assert_tree` (document_comparer/based_style_record_tests.rs:34) (?); `style_prop_signature` (document_comparer.rs:1485) (?); `effective_chain_props` (document_comparer.rs:1696) (?); `tests::restored_deleted_spacing_leaves_scratch_unids_behind` (document_comparer.rs:8044) (?); `coverage_boundary_tests::effective_run_properties_merge_independent_slots_and_clear_alternatives` (document_comparer.rs:8915) (?); `coverage_boundary_tests::style_cycles_stop_and_self_based_style_does_not_hide_own_properties` (document_comparer.rs:8952) (?); `authored_style_chain_boundary_tests::leaf` (document_comparer.rs:12032) (?); `authored_style_chain_boundary_tests::attrs` (document_comparer.rs:12039) (?); `authored_style_chain_boundary_tests::inherited_style_font_alternatives_and_missing_blocks_keep_exact_declarations` (document_comparer.rs:12048) (?); `authored_style_chain_boundary_tests::based_on_cycles_stop_after_unique_authored_owners_with_nearest_properties_winning` (document_comparer.rs:12093) (?); `authored_style_chain_boundary_tests::bounded_style_walk_reads_exactly_the_nearest_thirty_two_owners` (document_comparer.rs:12115) (?); +5 more

#### `<XName as PartialEq>::eq`  (L236-241, impl PartialEq)
`fn eq(&self, other: &Self) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_ref`, `Arc::ptr_eq`
- **Called by:** none found in src

#### `<XName as std::hash::Hash>::hash`  (L245-248, impl std::hash::Hash)
`fn hash<H: std::hash::Hasher>(&self, state: &mut H)`

- **Does:** _no doc comment_
- **Calls (crate):** `<XNamespace as std::hash::Hash>::hash` (xmllinq/mod.rs:156) L247
- **Calls (std/external/unresolved):** `.as_ref`, `.hash`
- **Called by:** none found in src

#### `<XName as std::fmt::Debug>::fmt`  (L251-253, impl std::fmt::Debug)
`fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`

- **Does:** _no doc comment_
- **Calls (crate):** `XName::clark` (xmllinq/mod.rs:226) (by name) L252
- **Macros:** `write!`
- **Called by:** none found in src

#### `NodeData::new`  (L350-357)
`fn new(kind: NodeKind) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `Dom::alloc` (xmllinq/mod.rs:394)

#### `Dom::new`  (L387-392, pub)
`pub fn new() -> Self`

- **Does:** Construct an empty arena.
- **Calls (std/external/unresolved):** `HashMap::new`, `Vec::new`
- **Called by:** `append_documents` (append.rs:152); `drop_comments_in_new_parts` (append.rs:450); `insert_lists` (append.rs:1159); `nsids` (append.rs:1226); `tests::nsid_collisions_take_the_next_free_value` (append.rs:1351); `tests::section_type_sits_after_header_references` (append.rs:1398); `tests::ignorable_prefixes_merge_only_with_the_same_namespace` (append.rs:1417); `public_memory_append_owner_boundary_tests::view` (append.rs:1504); `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1594); `changes_in` (changes.rs:302); `CommentFamily::load` (comments.rs:429); `CommentFamily::store_people` (comments.rs:977); +725 more

#### `Dom::alloc`  (L394-398)
`fn alloc(&mut self, kind: NodeKind) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `NodeData::new` (xmllinq/mod.rs:350) L396
- **Calls (std/external/unresolved):** `.len`, `.push`, `NodeId`
- **Called by:** `Dom::new_document` (xmllinq/mod.rs:472); `Dom::new_element` (xmllinq/mod.rs:478); `Dom::new_text` (xmllinq/mod.rs:483); `Dom::new_comment` (xmllinq/mod.rs:487); `Dom::new_pi` (xmllinq/mod.rs:492); `Dom::clone_subtree` (xmllinq/mod.rs:1215)

#### `Dom::node_count`  (L402-404, pub)
`pub fn node_count(&self) -> usize`

- **Does:** Number of nodes currently in the arena.
- **Calls (std/external/unresolved):** `.len`
- **Called by:** `coverage_real_revision_merge_matrix_tests::paired_property_comparison_never_grows_the_persistent_dom_or_changes_sources` (comparer/finalize.rs:26393); `finalizer_remaining_source_boundary_tests::paired_format_equality_is_complete_and_uses_no_production_arena_nodes` (comparer/finalize.rs:31191) (?); `public_body_heuristic_story_scope_tests::public_body_layout_heuristics_preserve_complete_independent_header_and_footer_parts` (comparer/finalize.rs:32251); `interleave_terminal_source_ownership_tests::remains_source_owned` (comparer/finalize.rs:33122) (?); `interleave_terminal_source_ownership_tests::split_inserted_title_fragments_do_not_take_another_authors_pending_revision` (comparer/finalize.rs:33227) (?); `whitespace_source_mark_carrier_tests::whitespace_fold_preserves_selected_source_mark_shells_and_every_body_owner` (comparer/finalize.rs:33360) (?); `fragmented_source_label_boundary_tests::fragmented_labels_keep_literal_source_chars_formats_and_revision_history` (comparer/finalize.rs:33524) (?); `fragmented_source_label_boundary_tests::label_format_boundaries_and_disabled_history_leave_complete_original_sources_intact` (comparer/finalize.rs:33678) (?); `fragmented_source_label_boundary_tests::public_label_zip_preserves_already_adjacent_spacer_and_missing_layout_ownership` (comparer/finalize.rs:33810) (?); `fragmented_source_label_boundary_tests::wholesale_glue_only_lcs_preserves_crossed_significant_source_anchors` (comparer/finalize.rs:33908) (?); `nested_deleted_text_source_ownership_tests::outer_deletion_preserves_independently_owned_nested_revision_text_and_fields` (comparer/finalize.rs:33977) (?); `reached_word_document_policy_tests::document_policy_reaches_numbering_relatedness_and_demo_intro_boundaries` (comparer/finalize.rs:34228); +5 more

#### `Dom::node_capacity`  (L413-415, pub)
`pub fn node_capacity(&self) -> usize`

- **Does:** Allocated node-slot capacity of the arena's backing `Vec`.
- **Calls (std/external/unresolved):** `.capacity`
- **Called by:** `coverage_real_revision_merge_matrix_tests::paired_property_comparison_never_grows_the_persistent_dom_or_changes_sources` (comparer/finalize.rs:26393); `format_change_cache_tests::detect_format_changes_never_grows_production_arena_capacity` (comparer/formatchg.rs:651)

#### `Dom::with_scratch`  (L449-459, pub)
`pub fn with_scratch<R>(&mut self, f: impl FnOnce(&mut Dom) -> R) -> R`

- **Does:** Run `f` and then reclaim every node it allocated by truncating the arena back to its pre-call length.
- **Calls (std/external/unresolved):** `.is_empty`, `.len`, `.retain`, `.truncate`, `f`
- **Called by:** `paired_replacement_has_identical_properties` (comparer/finalize.rs:8524); `normalized_rpr_serialized` (comparer/formatchg.rs:132)

#### `Dom::data`  (L461-463)
`fn data(&self, id: NodeId) -> &NodeData`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `tests::hidden_text_paints_nothing_but_reads_back` (convert/raster.rs:748) (?); `raster_payload_boundary_tests::image_alpha_composites_into_white_and_degenerate_placement_is_identity` (convert/raster.rs:952) (?); `raster_payload_boundary_tests::rgb_missing_alpha_defaults_only_missing_pixels_to_opaque` (convert/raster.rs:995) (?); `raster_payload_boundary_tests::empty_shape_payloads_preserve_pixels_and_negative_rectangle_extents_are_normalized` (convert/raster.rs:1060) (?); `residual_display_contract_tests::empty_glyphs_and_nonpositive_display_sizes_never_load_or_paint_a_face` (convert/raster.rs:1154) (?); `residual_display_contract_tests::collapsed_vector_subpaths_and_rejected_encoded_picture_payloads_preserve_all_pixels` (convert/raster.rs:1274) (?); `residual_display_contract_tests::memory_png_roundtrip_preserves_exact_unrotated_page_size_and_pixels` (convert/raster.rs:1390) (?); `Dom::is_element` (xmllinq/mod.rs:502) (?); `Dom::is_text` (xmllinq/mod.rs:506) (?); `Dom::is_document` (xmllinq/mod.rs:511) (?); `Dom::is_comment` (xmllinq/mod.rs:515) (?); `Dom::is_pi` (xmllinq/mod.rs:519) (?); +35 more

#### `Dom::data_mut`  (L464-466)
`fn data_mut(&mut self, id: NodeId) -> &mut NodeData`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `paint_image` (convert/raster.rs:543) (?); `Dom::set_name` (xmllinq/mod.rs:539); `Dom::set_text_value` (xmllinq/mod.rs:555); `Dom::set_declaration` (xmllinq/mod.rs:589); `Dom::set_attributes` (xmllinq/mod.rs:921); `Dom::set_attribute_value` (xmllinq/mod.rs:961); `Dom::detach` (xmllinq/mod.rs:1002); `Dom::add` (xmllinq/mod.rs:1058); `Dom::add_text` (xmllinq/mod.rs:1071); `Dom::add_first` (xmllinq/mod.rs:1085); `Dom::remove_nodes` (xmllinq/mod.rs:1094); `Dom::add_before_self` (xmllinq/mod.rs:1114); +3 more

#### `Dom::new_document`  (L472-474, pub)
`pub fn new_document(&mut self) -> NodeId`

- **Does:** Allocate a Document (root container) node, with no declaration.
- **Calls (crate):** `Dom::alloc` (xmllinq/mod.rs:394) L473
- **Called by:** `coalesce` (comparer/atomize.rs:557); `deeper_boundary_tests::projection_rejects_document_text_and_a_detached_leaf_without_a_run` (comparer/comments.rs:2540); `produce_document` (comparer/produce.rs:65); `Parser::parse_document` (xmllinq/parse.rs:450)

#### `Dom::new_element`  (L478-480, pub)
`pub fn new_element(&mut self, name: XName) -> NodeId`

- **Does:** Allocate an element node with the given expanded name.
- **Calls (crate):** `Dom::alloc` (xmllinq/mod.rs:394) L479
- **Called by:** `append_documents` (append.rs:152); `stage_comments` (append.rs:557); `carry_notes` (append.rs:667); `join` (append.rs:1264); `set_section_type` (append.rs:1325); `Anchors::scan` (comments.rs:241); `CommentFamily::add` (comments.rs:604) (?); `CommentFamily::adopt` (comments.rs:687) (?); `CommentFamily::set_text` (comments.rs:737) (?); `CommentFamily::store` (comments.rs:798); `CommentFamily::store_people` (comments.rs:977); `fill_comment_paragraph` (comments.rs:1073); +273 more

#### `Dom::new_text`  (L483-485, pub)
`pub fn new_text(&mut self, value: &str) -> NodeId`

- **Does:** Allocate a text node carrying `value`.
- **Calls (crate):** `Dom::alloc` (xmllinq/mod.rs:394) L484
- **Calls (std/external/unresolved):** `.to_string`, `NodeKind::Text`
- **Called by:** `split_seg` (comparer/comments.rs:568); `deeper_boundary_tests::projection_rejects_document_text_and_a_detached_leaf_without_a_run` (comparer/comments.rs:2540); `coverage_tests::revision_wrapper_requires_consistent_hyperlink_stamps` (comparer/finalize.rs:15837); `clone_internal_unsalted` (comparer/preprocess.rs:245) (?); `coverage_boundary_tests::style_cycles_stop_and_self_based_style_does_not_hide_own_properties` (document_comparer.rs:8952); `Dom::add_text` (xmllinq/mod.rs:1071); `Parser::parse_element` (xmllinq/parse.rs:505)

#### `Dom::new_comment`  (L487-489, pub)
`pub fn new_comment(&mut self, value: &str) -> NodeId`

- **Does:** Allocate a comment node (`<!-- value -->`).
- **Calls (crate):** `Dom::alloc` (xmllinq/mod.rs:394) L488
- **Calls (std/external/unresolved):** `.to_string`, `NodeKind::Comment`
- **Called by:** `Parser::parse_document` (xmllinq/parse.rs:450); `Parser::parse_element` (xmllinq/parse.rs:505)

#### `Dom::new_pi`  (L492-497, pub)
`pub fn new_pi(&mut self, target: &str, data: &str) -> NodeId`

- **Does:** Allocate a processing instruction node (`<?target data?>`).
- **Calls (crate):** `Dom::alloc` (xmllinq/mod.rs:394) L493
- **Calls (std/external/unresolved):** `.to_string`, `Box::new`, `NodeKind::Pi`
- **Called by:** `Parser::parse_pi_node` (xmllinq/parse.rs:623)

#### `Dom::is_element`  (L502-504, pub)
`pub fn is_element(&self, id: NodeId) -> bool`

- **Does:** True if `id` is an element node.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) (by name) L503
- **Macros:** `matches!`
- **Called by:** `public_memory_append_owner_boundary_tests::semantic` (append.rs:1448); `extract_events` (comparer/comments.rs:260); `descendants_trimmed::{fn walk}` (comparer/finalize.rs:65); `convert_run_text_to_del_text::{fn walk}` (comparer/finalize.rs:103); `mark_content_transform` (comparer/finalize.rs:192); `conjoin_transform` (comparer/finalize.rs:585); `move_paragraph_properties_first` (comparer/finalize.rs:1524); `paragraph_mark_properties_equal::{fn signature}` (comparer/finalize.rs:3300); `reorder_replacements_ins_before_del` (comparer/finalize.rs:6652); `revision_wrapper` (comparer/finalize.rs:6707); `coalesce_adjacent_revisions` (comparer/finalize.rs:6759); `simplify_move_transform` (comparer/finalize.rs:6813); +91 more

#### `Dom::is_text`  (L506-508, pub)
`pub fn is_text(&self, id: NodeId) -> bool`

- **Does:** True if `id` is a text node.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) (by name) L507
- **Calls (std/external/unresolved):** `NodeKind::Text`
- **Macros:** `matches!`
- **Called by:** `extract` (comparer/bookmarks.rs:82); `extract_events` (comparer/comments.rs:260); `collect_segments` (comparer/comments.rs:477); `strip_whitespace_in_clone_text` (comparer/preprocess.rs:96); `clone_internal_unsalted` (comparer/preprocess.rs:245); `text_lines::{fn runs}` (debug.rs:1518); `alternate_content_paragraph_owner_tests::semantic` (revision_processor.rs:5083); `xml_holds` (scrub.rs:477); `emit` (xmllinq/serialize.rs:656); `serialize_document` (xmllinq/serialize.rs:793)

#### `Dom::is_document`  (L511-513, pub)
`pub fn is_document(&self, id: NodeId) -> bool`

- **Does:** True if `id` is the document container (allocation created with [`Self::new_document`] or via [`Self::parse_xdocument`]).
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) (by name) L512
- **Macros:** `matches!`
- **Called by:** `Dom::document` (xmllinq/mod.rs:809); `Dom::validate_attachment` (xmllinq/mod.rs:1025)

#### `Dom::is_comment`  (L515-517, pub)
`pub fn is_comment(&self, id: NodeId) -> bool`

- **Does:** True if `id` is a comment node.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) (by name) L516
- **Calls (std/external/unresolved):** `NodeKind::Comment`
- **Macros:** `matches!`
- **Called by:** `xml_holds` (scrub.rs:477); `emit` (xmllinq/serialize.rs:656); `serialize_document` (xmllinq/serialize.rs:793)

#### `Dom::is_pi`  (L519-521, pub)
`pub fn is_pi(&self, id: NodeId) -> bool`

- **Does:** True if `id` is a processing-instruction node.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) (by name) L520
- **Calls (std/external/unresolved):** `NodeKind::Pi`
- **Macros:** `matches!`
- **Called by:** `xml_holds` (scrub.rs:477); `xml_lexical_boundary_contract_tests::legacy_declaration_defaults_keep_readable_roots_without_inventing_metadata` (xmllinq/parse.rs:946) (?); `emit` (xmllinq/serialize.rs:656); `serialize_document` (xmllinq/serialize.rs:793); `tests::document_serialization_preserves_declarations_comments_and_processing_instructions` (xmllinq/serialize.rs:873)

#### `Dom::name`  (L526-531, pub)
`pub fn name(&self, id: NodeId) -> Option<XName>`

- **Does:** The element's expanded name, or `None` for non-element nodes.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L527
- **Calls (std/external/unresolved):** `.clone`, `Some`
- **Called by:** `comment_anchors` (append.rs:410); `offset_annotation_ids` (append.rs:495); `w_vals` (append.rs:812); `add_in_numbering_order` (append.rs:1202); `set_section_type` (append.rs:1325); `tests::section_type_sits_after_header_references` (append.rs:1398); `decorative` (audit.rs:942); `carriers` (changes.rs:441); `fold_style_pairs` (changes.rs:630); `paragraph_of` (changes.rs:676); `in_recorded_properties` (changes.rs:694); `recorded_marks` (changes.rs:707); +300 more

#### `Dom::name_is`  (L535-537, pub)
`pub fn name_is(&self, id: NodeId, name: &XName) -> bool`

- **Does:** True if `id` is an element whose name equals `name`.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) (by name) L536
- **Macros:** `matches!`
- **Called by:** `final_sect_pr` (append.rs:354); `drop_comments_where` (append.rs:426); `Package::fields` (audit.rs:764) (?); `fold_twins` (changes.rs:568); `fold_style_pairs` (changes.rs:630); `CommentFamily::set_text` (comments.rs:737) (?); `revision_tracking_element_from_ancestors` (comparer/atomize.rs:112); `atom_hash` (comparer/atomize.rs:163); `coverage_atomizer_owned_boundaries::note_definition_owns_every_character_and_mark_but_the_part_is_a_path_stop` (comparer/atomize.rs:827) (?); `coverage_atomizer_field_chain_boundaries::header_and_footer_roots_stop_paths_and_preserve_each_whitespace_boundary` (comparer/atomize.rs:1123) (?); `paragraph_of` (comparer/bookmarks.rs:66); `extract` (comparer/bookmarks.rs:82); +329 more

#### `Dom::set_name`  (L539-543, pub)
`pub fn set_name(&mut self, id: NodeId, new_name: XName)`

- **Does:** Set the element's expanded name.
- **Calls (crate):** `Dom::data_mut` (xmllinq/mod.rs:464) L540
- **Called by:** `resolve` (changes.rs:349) (?); `convert_run_text_to_del_text::{fn walk}` (comparer/finalize.rs:103); `convert_stamped_preins` (comparer/finalize.rs:10282); `flatten_tracked_deletions` (comparer/finalize.rs:11030); `convert_stamped_predeletes` (comparer/finalize.rs:11136); `fix_strict_validity_artifacts` (comparer/finalize.rs:11321); `fix_strict_validity_artifacts::{fn remap}` (comparer/finalize.rs:11390); `enforce_deleted_text_kinds::{fn walk}` (comparer/finalize.rs:14835); `delete_text_in_opaque` (comparer/produce.rs:41); `document_text_view` (debug.rs:2011); `mark_hf_part_content_as_deleted` (document_comparer.rs:5658); `rename_deleted_text` (edit/tracked.rs:911); +3 more

#### `Dom::text_value`  (L547-552, pub)
`pub fn text_value(&self, id: NodeId) -> Option<&str>`

- **Does:** The payload of a text or comment node as a borrowed string, or `None` for any other node kind.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L548
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `public_memory_append_owner_boundary_tests::semantic` (append.rs:1448) (?); `extract` (comparer/bookmarks.rs:82); `extract_events` (comparer/comments.rs:260); `collect_segments` (comparer/comments.rs:477); `public_option_source_ownership_tests::canonical` (comparer/cross_para.rs:2479) (?); `paragraph_mark_properties_equal::{fn signature}` (comparer/finalize.rs:3300); `coverage_round_next_tests::same_tree` (comparer/finalize.rs:19369) (?); `coverage_real_revision_merge_matrix_tests::guarded_multi_original_complete_projection::{fn semantic}` (comparer/finalize.rs:25604) (?); `consecutive_block_splice_tests::canonical` (comparer/finalize.rs:28376) (?); `demo_title_pilcrow_history_tests::effect_projection::{fn canonical}` (comparer/finalize.rs:28608) (?); `paired_row_mark_ownership_tests::canonical` (comparer/finalize.rs:28850) (?); `closing_authored_section_boundary_tests::canonical` (comparer/finalize.rs:29150) (?); +47 more

#### `Dom::set_text_value`  (L555-560, pub)
`pub fn set_text_value(&mut self, id: NodeId, value: &str)`

- **Does:** Replace the payload of a text or comment node.
- **Calls (crate):** `Dom::data_mut` (xmllinq/mod.rs:464) L556
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `strip_whitespace_in_clone_text` (comparer/preprocess.rs:96)

#### `Dom::pi_target`  (L564-569, pub)
`pub fn pi_target(&self, id: NodeId) -> Option<&str>`

- **Does:** The PI target (the part before the first whitespace inside `<?target?>`), or `None` for non-PI nodes.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L565
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `emit` (xmllinq/serialize.rs:656); `serialize_document` (xmllinq/serialize.rs:793); `tests::document_serialization_preserves_declarations_comments_and_processing_instructions` (xmllinq/serialize.rs:873)

#### `Dom::pi_data`  (L572-577, pub)
`pub fn pi_data(&self, id: NodeId) -> Option<&str>`

- **Does:** The PI data (everything between the target and `?>`), or `None` for non-PI nodes.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L573
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `xml_holds` (scrub.rs:477); `emit` (xmllinq/serialize.rs:656); `serialize_document` (xmllinq/serialize.rs:793); `tests::document_serialization_preserves_declarations_comments_and_processing_instructions` (xmllinq/serialize.rs:873)

#### `Dom::declaration`  (L581-586, pub)
`pub fn declaration(&self, id: NodeId) -> Option<&XDeclaration>`

- **Does:** The XML declaration attached to a Document node, or `None` for any other node kind.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L582
- **Calls (std/external/unresolved):** `.as_deref`
- **Called by:** `xml_lexical_boundary_contract_tests::legacy_declaration_defaults_keep_readable_roots_without_inventing_metadata` (xmllinq/parse.rs:946); `serialize_document` (xmllinq/serialize.rs:793)

#### `Dom::set_declaration`  (L589-593, pub)
`pub fn set_declaration(&mut self, id: NodeId, decl: Option<XDeclaration>)`

- **Does:** Attach or clear the XML declaration on a Document node.
- **Calls (crate):** `Dom::data_mut` (xmllinq/mod.rs:464) L590
- **Calls (std/external/unresolved):** `.map`
- **Called by:** `Parser::parse_document` (xmllinq/parse.rs:450); `tests::document_serialization_preserves_declarations_comments_and_processing_instructions` (xmllinq/serialize.rs:873)

#### `Dom::parent`  (L597-599, pub)
`pub fn parent(&self, id: NodeId) -> Option<NodeId>`

- **Does:** The parent of `id`, or `None` for the document root.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L598
- **Called by:** `drop_comments_where` (append.rs:426); `Package::empty_spacers` (audit.rs:661) (?); `run_convert` (bin/jubarte.rs:507) (?); `run_edit` (bin/jubarte.rs:953) (?); `run_debug_diff` (bin/jubarte.rs:2836) (?); `fold_twins` (changes.rs:568); `fold_style_pairs` (changes.rs:630); `target` (changes.rs:768); `owner_text` (changes.rs:816); `resolve_plan` (cli.rs:401) (?); `paragraph_of` (comparer/bookmarks.rs:66); `extract` (comparer/bookmarks.rs:82); +155 more

#### `Dom::nodes`  (L604-606, pub)
`pub fn nodes(&self, id: NodeId) -> Vec<NodeId>`

- **Does:** Direct children in document order, all node kinds (a clone of the internal content vec).
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L605
- **Calls (std/external/unresolved):** `.clone`
- **Called by:** `public_memory_append_owner_boundary_tests::semantic` (append.rs:1448) (?); `CommentFamily::set_text` (comments.rs:737) (?); `split_seg` (comparer/comments.rs:568); `split_run_before_leaf` (comparer/comments.rs:633); `public_option_source_ownership_tests::canonical` (comparer/cross_para.rs:2479); `descendants_trimmed::{fn walk}` (comparer/finalize.rs:65); `convert_run_text_to_del_text::{fn walk}` (comparer/finalize.rs:103); `rebuild_run` (comparer/finalize.rs:162); `mark_content_transform` (comparer/finalize.rs:192); `conjoin_transform` (comparer/finalize.rs:585); `resolve_alternate_content` (comparer/finalize.rs:1259); `unwrap_content_controls` (comparer/finalize.rs:1358); +108 more

#### `Dom::first_node`  (L609-611, pub)
`pub fn first_node(&self, id: NodeId) -> Option<NodeId>`

- **Does:** The first direct child of `id`, or `None` if it has none.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L610
- **Calls (std/external/unresolved):** `.copied`, `.first`
- **Called by:** none found in src

#### `Dom::last_node`  (L613-615, pub)
`pub fn last_node(&self, id: NodeId) -> Option<NodeId>`

- **Does:** The last direct child of `id`, or `None` if it has none.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L614
- **Calls (std/external/unresolved):** `.copied`, `.last`
- **Called by:** none found in src

#### `Dom::child_count`  (L622-624, pub)
`pub fn child_count(&self, id: NodeId) -> usize`

- **Does:** Number of direct children of `id` (all node kinds).
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L623
- **Calls (std/external/unresolved):** `.len`
- **Called by:** `recurse_children` (comparer/atomize.rs:393); `clone_block_level_content_for_hashing::{fn strip_nsdecls}` (comparer/preprocess.rs:134); `clone_children` (comparer/preprocess.rs:193); `clone_internal_unsalted` (comparer/preprocess.rs:245); `element_children` (comparer/preprocess.rs:506); `element_children_named` (comparer/preprocess.rs:519); `is_single_t_run` (comparer/preprocess.rs:544); `collect_simple_p_fragments` (comparer/preprocess.rs:775); `has_element_child` (comparer/preprocess.rs:835); `element_or_desc_has_predelete_orig` (comparer/preprocess.rs:1004); `stream_tbl_body` (comparer/preprocess.rs:1023); `stream_tr_body` (comparer/preprocess.rs:1048); +53 more

#### `Dom::child_at`  (L632-634, pub)
`pub fn child_at(&self, id: NodeId, i: usize) -> NodeId`

- **Does:** The `i`-th direct child of `id` (all node kinds).
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L633
- **Called by:** `recurse_children` (comparer/atomize.rs:393); `clone_block_level_content_for_hashing::{fn strip_nsdecls}` (comparer/preprocess.rs:134); `clone_children` (comparer/preprocess.rs:193); `clone_internal_unsalted` (comparer/preprocess.rs:245); `element_children` (comparer/preprocess.rs:506); `element_children_named` (comparer/preprocess.rs:519); `is_single_t_run` (comparer/preprocess.rs:544); `collect_simple_p_fragments` (comparer/preprocess.rs:775); `has_element_child` (comparer/preprocess.rs:835); `element_or_desc_has_predelete_orig` (comparer/preprocess.rs:1004); `stream_tbl_body` (comparer/preprocess.rs:1023); `stream_tr_body` (comparer/preprocess.rs:1048); +52 more

#### `Dom::elements`  (L640-651, pub)
`pub fn elements(&self, id: NodeId, filter: Option<&XName>) -> Vec<NodeId>`

- **Does:** Direct child elements, optionally filtered by expanded name.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L641,645
- **Calls (std/external/unresolved):** `.collect`, `.copied`, `.filter`, `.iter`
- **Called by:** `append_documents` (append.rs:152); `final_sect_pr` (append.rs:354); `drop_comments_where` (append.rs:426); `stage_comments` (append.rs:557); `carry_notes` (append.rs:667); `index_by_attribute` (append.rs:842); `ListCopy::build` (append.rs:1111); `insert_lists` (append.rs:1159); `add_in_numbering_order` (append.rs:1202); `join` (append.rs:1264); `set_section_type` (append.rs:1325); `tests::section_type_sits_after_header_references` (append.rs:1398); +577 more

#### `Dom::element`  (L655-661, pub)
`pub fn element(&self, id: NodeId, filter: &XName) -> Option<NodeId>`

- **Does:** The first direct child element whose name matches `filter`, or `None`.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L656; `Dom::data` (xmllinq/mod.rs:461) (by name) L660
- **Calls (std/external/unresolved):** `.copied`, `.find`, `.iter`
- **Macros:** `matches!`
- **Called by:** `body` (append.rs:349); `style_key` (append.rs:825); `abstract_of` (append.rs:1220); `unique_nsid` (append.rs:1241); `tests::nsid_collisions_take_the_next_free_value` (append.rs:1351); `tests::section_type_sits_after_header_references` (append.rs:1398) (?); `public_memory_append_owner_boundary_tests::view` (append.rs:1504); `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1594) (?); `Package::read` (audit.rs:354) (?); `Package::read_styles` (audit.rs:406); `Package::paragraph_style` (audit.rs:493); `Package::numbered` (audit.rs:513); +607 more

#### `Dom::descendants`  (L668-672, pub)
`pub fn descendants(&self, id: NodeId, filter: Option<&XName>) -> Vec<NodeId>`

- **Does:** Every descendant element of `id`, in pre-order.
- **Calls (crate):** `Dom::for_each_descendant_element` (xmllinq/mod.rs:677) L670
- **Calls (std/external/unresolved):** `.push`, `Vec::new`
- **Called by:** `comment_anchors` (append.rs:410); `offset_annotation_ids` (append.rs:495); `drop_colliding_paragraph_ids` (append.rs:523); `carry_notes` (append.rs:667); `ListCopy::build` (append.rs:1111); `nsids` (append.rs:1226); `Package::read_styles` (audit.rs:406); `Package::image_no_descr` (audit.rs:558) (?); `Package::table_no_header_row` (audit.rs:586) (?); `Package::direct_formatting` (audit.rs:714) (?); `Package::fields` (audit.rs:764) (?); `decorative` (audit.rs:942); +740 more

#### `Dom::for_each_descendant_element`  (L677-687, pub)
`pub fn for_each_descendant_element( &self, id: NodeId, filter: Option<&XName>, mut visit: impl FnMut(NodeId), )`

- **Does:** Visit every descendant element of `id` in document order without allocating a result `Vec` (DOM-ITER-03).
- **Calls (crate):** `Dom::find_descendant_element` (xmllinq/mod.rs:692) L683; `declare::{fn visit}` (markdown/from_docx/drawing.rs:344) (by name) L684
- **Called by:** `do_lcs_algorithm` (comparer/lcs.rs:2494); `test_for_invalid_content` (comparer/preprocess.rs:602); `add_sha1_hash_to_block_level_content` (comparer/preprocess.rs:1259); `hash_block_level_content` (comparer/preprocess.rs:1317); `Dom::descendants` (xmllinq/mod.rs:668); `Dom::for_each_descendant_and_self` (xmllinq/mod.rs:753)

#### `Dom::find_descendant_element`  (L692-725, pub)
`pub fn find_descendant_element( &self, id: NodeId, filter: Option<&XName>, mut pred: impl FnMut(NodeId) -> bool, ) -> Option<NodeId>`

- **Does:** First descendant element (in pre-order) that passes both the name `filter` and the `pred` predicate; the walk stops there instead of listing the whole subtree first.
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L701; `Dom::child_at` (xmllinq/mod.rs:632) L706; `Dom::is_element` (xmllinq/mod.rs:502) L708; `Dom::name_is` (xmllinq/mod.rs:535) L717
- **Calls (std/external/unresolved):** `.last_mut`, `.pop`, `.push`, `Some`, `pred`
- **Macros:** `vec!`
- **Called by:** `first_named` (convert/mod.rs:4142); `para_formatting_changed` (convert/mod.rs:11490); `first_named_any` (convert/mod.rs:19943); `drop_wholly_deleted_bookmarks` (revision_processor/bookmarks.rs:23); `Dom::for_each_descendant_element` (xmllinq/mod.rs:677)

#### `Dom::descendant_nodes`  (L730-734, pub)
`pub fn descendant_nodes(&self, id: NodeId) -> Vec<NodeId>`

- **Does:** Every descendant node (not just elements), in pre-order: self is NOT included.
- **Calls (crate):** `Dom::walk_descendant_nodes` (xmllinq/mod.rs:735) L732
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `extract` (comparer/bookmarks.rs:82); `extract_events` (comparer/comments.rs:260); `collect_segments` (comparer/comments.rs:477); `strip_whitespace_in_clone_text` (comparer/preprocess.rs:96); `xml_holds` (scrub.rs:477); `tests::document_serialization_preserves_declarations_comments_and_processing_instructions` (xmllinq/serialize.rs:873)

#### `Dom::walk_descendant_nodes`  (L735-742)
`fn walk_descendant_nodes(&self, id: NodeId, out: &mut Vec<NodeId>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L736,738; `Dom::walk_descendant_nodes` (xmllinq/mod.rs:735) L739
- **Calls (std/external/unresolved):** `.is_empty`, `.push`
- **Called by:** `Dom::descendant_nodes` (xmllinq/mod.rs:730); `Dom::walk_descendant_nodes` (xmllinq/mod.rs:735)

#### `Dom::descendants_and_self`  (L746-750, pub)
`pub fn descendants_and_self(&self, id: NodeId, filter: Option<&XName>) -> Vec<NodeId>`

- **Does:** `Self` (if it is an element matching `filter`) then its descendants, in pre-order.
- **Calls (crate):** `Dom::for_each_descendant_and_self` (xmllinq/mod.rs:753) L748
- **Calls (std/external/unresolved):** `.push`, `Vec::new`
- **Called by:** `w_vals` (append.rs:812); `rewrite_para_id_references` (comparer/comments.rs:988); `rewrite_durable_id_references` (comparer/comments.rs:1003); `preserve_cloned_namespace_context` (comparer/comments.rs:1052); `drop_pt_bookkeeping` (comparer/finalize.rs:481); `declare_extension_namespaces_ignorable` (comparer/finalize.rs:899); `bind_compatibility_prefixes` (comparer/finalize.rs:983); `remove_powertools_scratch_markup` (comparer/finalize.rs:1018); `resolve_alternate_content` (comparer/finalize.rs:1259); `content_revision_marks` (comparer/finalize.rs:6184); `wml_order_elements_per_standard` (comparer/finalize.rs:10483); `fix_strict_validity_artifacts` (comparer/finalize.rs:11321); +24 more

#### `Dom::for_each_descendant_and_self`  (L753-765, pub)
`pub fn for_each_descendant_and_self( &self, id: NodeId, filter: Option<&XName>, mut visit: impl FnMut(NodeId), )`

- **Does:** Non-allocating `DescendantsAndSelf` walk (DOM-ITER-03).
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L759; `declare::{fn visit}` (markdown/from_docx/drawing.rs:344) (by name) L762; `Dom::for_each_descendant_element` (xmllinq/mod.rs:677) L764
- **Calls (std/external/unresolved):** `.is_none_or`
- **Called by:** `remove_existing_powertools_markup` (comparer/preprocess.rs:572); `Dom::descendants_and_self` (xmllinq/mod.rs:746)

#### `Dom::ancestors`  (L770-782, pub)
`pub fn ancestors(&self, id: NodeId, filter: Option<&XName>) -> Vec<NodeId>`

- **Does:** Ancestor elements from the nearest parent up to (but not including) the document, optionally filtered by name.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L772,774,779
- **Calls (std/external/unresolved):** `.is_none_or`, `.push`, `Vec::new`
- **Called by:** `Story::location` (audit.rs:317); `Package::empty_spacers` (audit.rs:661) (?); `Package::direct_formatting` (audit.rs:714) (?); `under_fallback` (audit.rs:954); `fold_twins` (changes.rs:568); `paragraph_of` (changes.rs:676); `in_recorded_properties` (changes.rs:694); `paragraph_move` (changes.rs:731); `holder` (changes.rs:743); `deleted_text` (changes.rs:792); `Anchors::scan` (comments.rs:241); `comment_text` (comments.rs:1045); +87 more

#### `Dom::ancestors_and_self`  (L787-805, pub)
`pub fn ancestors_and_self(&self, id: NodeId, filter: Option<&XName>) -> Vec<NodeId>`

- **Does:** `Self` then its ancestor elements, optionally filtered by name.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L791,796; `Dom::is_element` (xmllinq/mod.rs:502) L797
- **Calls (std/external/unresolved):** `.is_none_or`, `.push`, `Some`, `Vec::new`
- **Called by:** `resolve_alternate_content` (comparer/finalize.rs:1259); `prefix_in_scope` (comparer/finalize.rs:1448); `wrap_revised_hyperlink_as_field` (comparer/finalize.rs:1599); `para_is_pure_deleted_uncached` (comparer/finalize.rs:2873); `align_word_table_and_comment_chrome` (comparer/finalize.rs:6215); `hyperlink_is_wholly_inserted` (comparer/finalize.rs:6393); `fold_midstream_demo_title_into_numbered_heading_with_source_properties` (comparer/finalize.rs:12619); `text_side` (comparer/finalize.rs:12905); `deep_production_path_tests::body_revisions` (comparer/finalize.rs:20932); `paragraph_id` (fields.rs:932) (?); `add_content_controls_with_owners` (revision_processor.rs:2182)

#### `Dom::document`  (L809-818, pub)
`pub fn document(&self, id: NodeId) -> Option<NodeId>`

- **Does:** Walk the parent chain until a Document node is found; returns it, or `None` if `id` is detached from any document.
- **Calls (crate):** `Dom::is_document` (xmllinq/mod.rs:511) L812; `Dom::data` (xmllinq/mod.rs:461) L815
- **Calls (std/external/unresolved):** `Some`
- **Called by:** none found in src

#### `Dom::root`  (L822-828, pub)
`pub fn root(&self, doc: NodeId) -> Option<NodeId>`

- **Does:** The first child of `doc` that is an element, or `None` if the document is empty.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L823; `Dom::is_element` (xmllinq/mod.rs:502) L827
- **Calls (std/external/unresolved):** `.copied`, `.find`, `.iter`
- **Called by:** `parse` (append.rs:330); `drop_comments_in_new_parts` (append.rs:450); `insert_lists` (append.rs:1159); `nsids` (append.rs:1226); `tests::nsid_collisions_take_the_next_free_value` (append.rs:1351); `tests::section_type_sits_after_header_references` (append.rs:1398); `tests::ignorable_prefixes_merge_only_with_the_same_namespace` (append.rs:1417); `public_memory_append_owner_boundary_tests::view` (append.rs:1504); `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1594); `changes_in` (changes.rs:302); `CommentFamily::load` (comments.rs:429); `CommentFamily::comments_root` (comments.rs:644); +470 more

#### `Dom::index_in_parent`  (L830-834)
`fn index_in_parent(&self, id: NodeId) -> Option<(NodeId, usize)>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L831,832
- **Calls (std/external/unresolved):** `.iter`, `.position`, `Some`
- **Called by:** `Dom::nodes_after_self` (xmllinq/mod.rs:838); `Dom::nodes_before_self` (xmllinq/mod.rs:846); `Dom::detach` (xmllinq/mod.rs:1002); `Dom::add_before_self` (xmllinq/mod.rs:1114); `Dom::add_after_self` (xmllinq/mod.rs:1131); `Dom::replace_with` (xmllinq/mod.rs:1149)

#### `Dom::nodes_after_self`  (L838-843, pub)
`pub fn nodes_after_self(&self, id: NodeId) -> Vec<NodeId>`

- **Does:** Following siblings of `id` (all node kinds), in document order.
- **Calls (crate):** `Dom::index_in_parent` (xmllinq/mod.rs:830) L839; `Dom::data` (xmllinq/mod.rs:461) L840
- **Calls (std/external/unresolved):** `.to_vec`, `Vec::new`
- **Called by:** `write_result` (fields.rs:824); `write_toc` (fields.rs:1154); `elements_after_self` (revision_processor.rs:880); `accept_deleted_cells_transform` (revision_processor.rs:2627); `Dom::next_element` (xmllinq/mod.rs:855)

#### `Dom::nodes_before_self`  (L846-851, pub)
`pub fn nodes_before_self(&self, id: NodeId) -> Vec<NodeId>`

- **Does:** Preceding siblings of `id` (all node kinds), in document order.
- **Calls (crate):** `Dom::index_in_parent` (xmllinq/mod.rs:830) L847; `Dom::data` (xmllinq/mod.rs:461) L848
- **Calls (std/external/unresolved):** `.to_vec`, `Vec::new`
- **Called by:** `delete_stories_of_deleted_text_boxes` (comparer/finalize.rs:10812); `separate` (edit/structural.rs:242); `previous_paragraph` (edit/tracked.rs:989); `write_result` (fields.rs:824); `write_toc` (fields.rs:1154); `get_paragraph_info` (revision_processor.rs:952); `content_elements_before_self` (revision_processor.rs:988)

#### `Dom::next_element`  (L855-859, pub)
`pub fn next_element(&self, id: NodeId) -> Option<NodeId>`

- **Does:** First following sibling element, or `None` if there is no element sibling after `id`.
- **Calls (crate):** `Dom::nodes_after_self` (xmllinq/mod.rs:838) L856; `Dom::is_element` (xmllinq/mod.rs:502) L858
- **Calls (std/external/unresolved):** `.find`, `.into_iter`
- **Called by:** `fix_up_footnotes_endnotes_with_custom_markers` (comparer/footnotes.rs:887); `coverage_custom_note_owner_boundaries::custom_marker_fixup_moves_only_matching_leaf_and_keeps_all_other_owners` (comparer/footnotes.rs:1709); `separate` (edit/structural.rs:242); `check` (edit/tracked.rs:84) (?); `emit` (edit/tracked.rs:275) (?); `emit_text` (edit/tracked.rs:732); `Transaction::resolve_comment_text` (edit.rs:2242) (?); `Transaction::merge_partner` (edit.rs:3691) (?); `merge_into` (edit.rs:6184); `write_toc` (fields.rs:1154); `move_range_bookmark_regressions::range_only_moved_nonempty_bookmark_disappears_but_original_empty_anchor_survives` (revision_processor/bookmarks.rs:180) (?)

#### `Dom::has_elements`  (L862-864, pub)
`pub fn has_elements(&self, id: NodeId) -> bool`

- **Does:** True if `id` has at least one direct child element.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L863; `Dom::is_element` (xmllinq/mod.rs:502) L863
- **Calls (std/external/unresolved):** `.any`, `.iter`
- **Called by:** `paragraph_mark_properties_equal` (comparer/finalize.rs:3280); `fill_in_empty_footnotes_endnotes` (comparer/footnotes.rs:133); `deterministic_note_boundary_tests::properties` (comparer/footnotes.rs:1291) (?); `build_paragraph` (comparer/produce.rs:104); `get_property_value` (comparer/revisions.rs:287); `check_text_content` (edit/v2.rs:806); `accept_revisions_for_element` (revision_processor.rs:459); `accept_revisions_for_part_content` (revision_processor.rs:3252)

#### `Dom::has_attributes`  (L867-869, pub)
`pub fn has_attributes(&self, id: NodeId) -> bool`

- **Does:** True if `id` carries at least one attribute.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L868
- **Calls (std/external/unresolved):** `.is_empty`
- **Called by:** none found in src

#### `Dom::attribute`  (L874-880, pub)
`pub fn attribute(&self, id: NodeId, name: &XName) -> Option<&str>`

- **Does:** The value of attribute `name` on `id`, or `None` if `id` is not an element or has no such attribute.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L875
- **Calls (std/external/unresolved):** `.as_str`, `.find`, `.iter`, `.map`
- **Called by:** `append_documents` (append.rs:152); `merge_namespace_declarations` (append.rs:365); `drop_comments_where` (append.rs:426); `offset_annotation_ids` (append.rs:495); `drop_colliding_paragraph_ids` (append.rs:523); `stage_comments` (append.rs:557); `carry_notes` (append.rs:667); `w_vals` (append.rs:812); `style_key` (append.rs:825); `index_by_attribute` (append.rs:842); `ListCopy::build` (append.rs:1111); `insert_lists` (append.rs:1159); +537 more

#### `Dom::attributes`  (L885-891, pub)
`pub fn attributes(&self, id: NodeId) -> Vec<(XName, String)>`

- **Does:** All attributes on `id`, in declaration order, as `(name, value)` owned pairs.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L886
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.iter`, `.map`
- **Called by:** `elements` (admission.rs:782) (?); `merge_namespace_declarations` (append.rs:365); `merge_part_namespaces` (append.rs:1081); `public_memory_append_owner_boundary_tests::semantic` (append.rs:1448); `coalesce_recurse` (comparer/atomize.rs:575); `rewrite_para_id_references` (comparer/comments.rs:988); `rewrite_durable_id_references` (comparer/comments.rs:1003); `namespace_declarations` (comparer/comments.rs:1018); `preserve_cloned_namespace_context` (comparer/comments.rs:1052); `union_comments_xml` (comparer/comments.rs:1190); `drop_orphans` (comparer/comments.rs:1416); `public_option_source_ownership_tests::canonical` (comparer/cross_para.rs:2479); +145 more

#### `Dom::attr_count`  (L895-897, pub)
`pub fn attr_count(&self, id: NodeId) -> usize`

- **Does:** Number of attributes on `id`.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L896
- **Calls (std/external/unresolved):** `.len`
- **Called by:** `drop_pt_bookkeeping` (comparer/finalize.rs:481); `clone_block_level_content_for_hashing::{fn strip_nsdecls}` (comparer/preprocess.rs:134); `new_with_filtered_attrs` (comparer/preprocess.rs:162); `clone_internal_unsalted` (comparer/preprocess.rs:245); `remove_existing_powertools_markup` (comparer/preprocess.rs:572); `filtered_p_attr_xml` (comparer/preprocess.rs:742); `filtered_leaf_attr_xml` (comparer/preprocess.rs:846); `clone_for_structure_hash` (comparer/preprocess.rs:1221); `xml_holds` (scrub.rs:477); `attribute_by_local` (validate.rs:1516); `namespace_in_scope` (validate.rs:1621); `tag_policy_tests::the_dom_parser_keeps_every_attribute_and_its_order` (xmllinq/parse.rs:1189); +5 more

#### `Dom::attr_at`  (L906-909, pub)
`pub fn attr_at(&self, id: NodeId, i: usize) -> (&XName, &str)`

- **Does:** The `i`-th attribute on `id`, as borrowed `(name, value)` (no allocations; DOM-ITER-01 backs the serializer hot path).
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L907
- **Calls (std/external/unresolved):** `.as_str`
- **Called by:** `drop_pt_bookkeeping` (comparer/finalize.rs:481); `clone_block_level_content_for_hashing::{fn strip_nsdecls}` (comparer/preprocess.rs:134); `new_with_filtered_attrs` (comparer/preprocess.rs:162); `clone_internal_unsalted` (comparer/preprocess.rs:245); `remove_existing_powertools_markup` (comparer/preprocess.rs:572); `filtered_p_attr_xml` (comparer/preprocess.rs:742); `filtered_leaf_attr_xml` (comparer/preprocess.rs:846); `clone_for_structure_hash` (comparer/preprocess.rs:1221); `xml_holds` (scrub.rs:477); `attribute_by_local` (validate.rs:1516); `namespace_in_scope` (validate.rs:1621); `tag_policy_tests::the_dom_parser_keeps_every_attribute_and_its_order` (xmllinq/parse.rs:1189); +4 more

#### `Dom::set_attributes`  (L921-955, pub(crate))
`pub(crate) fn set_attributes(&mut self, id: NodeId, attrs: Vec<(XName, String)>)`

- **Does:** Insert a batch of attributes in declaration order.
- **Calls (crate):** `Dom::data_mut` (xmllinq/mod.rs:464) L922
- **Calls (std/external/unresolved):** `.clone`, `.entry`, `.enumerate`, `.find`, `.get`, `.insert`, `.is_empty`, `.iter_mut`, `.iter`, `.len`, `.or_insert`, `.push`, `HashMap::with_capacity`
- **Called by:** `Parser::parse_element` (xmllinq/parse.rs:505)

#### `Dom::set_attribute_value`  (L961-990, pub)
`pub fn set_attribute_value(&mut self, id: NodeId, name: &XName, value: Option<&str>)`

- **Does:** Add or update an attribute.
- **Calls (crate):** `Dom::data_mut` (xmllinq/mod.rs:464) L962; `bump_attr_name_probes` (xmllinq/mod.rs:283) L970
- **Calls (std/external/unresolved):** `.clone`, `.enumerate`, `.iter`, `.position`, `.push`, `.retain`, `.to_string`, `Some`
- **Called by:** `merge_namespace_declarations` (append.rs:365); `offset_annotation_ids` (append.rs:495); `drop_colliding_paragraph_ids` (append.rs:523); `stage_comments` (append.rs:557); `carry_notes` (append.rs:667); `carry_styles_and_numbering` (append.rs:861); `ListCopy::build` (append.rs:1111); `unique_nsid` (append.rs:1241); `join` (append.rs:1264); `set_section_type` (append.rs:1325); `CommentFamily::add` (comments.rs:604) (?); `CommentFamily::adopt` (comments.rs:687) (?); +263 more

#### `Dom::is_namespace_declaration`  (L996-999, pub)
`pub fn is_namespace_declaration(&self, name: &XName) -> bool`

- **Does:** True if `name` looks like an `xmlns` / `xmlns:*` namespace declaration: namespace URI `http://www.w3.org/2000/xmlns/`, or no namespace with local name `xmlns`.
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L997,998; `XName::local_name` (xmllinq/mod.rs:208) L998
- **Calls (std/external/unresolved):** `.is_empty`
- **Called by:** `merge_namespace_declarations` (append.rs:365); `public_memory_append_owner_boundary_tests::semantic` (append.rs:1448); `namespace_declarations` (comparer/comments.rs:1018); `preserve_cloned_namespace_context` (comparer/comments.rs:1052); `public_option_source_ownership_tests::canonical` (comparer/cross_para.rs:2479); `public_option_source_ownership_tests::source_events::{fn walk}` (comparer/cross_para.rs:2513); `drop_pt_bookkeeping` (comparer/finalize.rs:481); `declare_extension_namespaces_ignorable` (comparer/finalize.rs:899); `prefix_in_scope` (comparer/finalize.rs:1448); `paragraph_mark_properties_equal::{fn signature}` (comparer/finalize.rs:3300); `paired_replacement_has_identical_properties` (comparer/finalize.rs:8524); `repair_inherited_invalidity` (comparer/finalize.rs:14767); +48 more

#### `Dom::detach`  (L1002-1007)
`fn detach(&mut self, id: NodeId)`

- **Does:** ── mutation ────────────────────────────────────────────────────────────── _(from a code comment)_
- **Calls (crate):** `Dom::index_in_parent` (xmllinq/mod.rs:830) L1003; `Dom::data_mut` (xmllinq/mod.rs:464) L1004,1005
- **Calls (std/external/unresolved):** `.remove`
- **Called by:** `Dom::remove` (xmllinq/mod.rs:1102)

#### `Dom::materialize`  (L1013-1019)
`fn materialize(&mut self, node: NodeId) -> NodeId`

- **Does:** If `node` already has a parent, deep-clone it (LINQ-to-XML semantics); otherwise return it as-is.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L1014; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L1015
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `Dom::add` (xmllinq/mod.rs:1058); `Dom::add_first` (xmllinq/mod.rs:1085); `Dom::add_before_self` (xmllinq/mod.rs:1114); `Dom::add_after_self` (xmllinq/mod.rs:1131); `Dom::replace_with` (xmllinq/mod.rs:1149)

#### `Dom::validate_attachment`  (L1025-1035)
`fn validate_attachment(&self, parent: NodeId, node: NodeId)`

- **Does:** Centralized validation before a node is attached to a parent: `parent` must be a container (`Element` or `Document`), `node` must not be the same as `parent`, and an unparented `node` must not be an ancestor of `parent` (cycle under descendant).
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L1026; `Dom::is_document` (xmllinq/mod.rs:511) L1026; `Dom::data` (xmllinq/mod.rs:461) L1032; `Dom::is_ancestor_of` (xmllinq/mod.rs:1038) L1032
- **Calls (std/external/unresolved):** `.is_none`
- **Macros:** `panic!`
- **Called by:** `Dom::add` (xmllinq/mod.rs:1058); `Dom::add_text` (xmllinq/mod.rs:1071); `Dom::add_first` (xmllinq/mod.rs:1085); `Dom::add_before_self` (xmllinq/mod.rs:1114); `Dom::add_after_self` (xmllinq/mod.rs:1131); `Dom::replace_with` (xmllinq/mod.rs:1149); `Dom::clone_subtree` (xmllinq/mod.rs:1215)

#### `Dom::is_ancestor_of`  (L1038-1047)
`fn is_ancestor_of(&self, ancestor: NodeId, descendant: NodeId) -> bool`

- **Does:** True when `ancestor` is strictly above `descendant` on the parent chain.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L1039,1044
- **Called by:** `Dom::validate_attachment` (xmllinq/mod.rs:1025)

#### `Dom::add`  (L1058-1063, pub)
`pub fn add(&mut self, parent: NodeId, node: NodeId)`

- **Does:** Append `node` as the last child of `parent`.
- **Calls (crate):** `Dom::materialize` (xmllinq/mod.rs:1013) L1059; `Dom::validate_attachment` (xmllinq/mod.rs:1025) L1060; `Dom::data_mut` (xmllinq/mod.rs:464) L1061,1062
- **Calls (std/external/unresolved):** `.push`, `Some`
- **Called by:** `append_documents` (append.rs:152); `stage_comments` (append.rs:557); `carry_notes` (append.rs:667); `carry_styles_and_numbering` (append.rs:861); `add_in_numbering_order` (append.rs:1202); `join` (append.rs:1264); `set_section_type` (append.rs:1325); `Anchors::scan` (comments.rs:241); `CommentFamily::add` (comments.rs:604); `CommentFamily::store` (comments.rs:798); `CommentFamily::store_people` (comments.rs:977); `fill_comment_paragraph` (comments.rs:1073); +222 more

#### `Dom::add_text`  (L1071-1077, pub)
`pub fn add_text(&mut self, parent: NodeId, value: &str) -> NodeId`

- **Does:** Allocate a new text child of `parent` carrying `value`.
- **Calls (crate):** `Dom::new_text` (xmllinq/mod.rs:483) L1072; `Dom::validate_attachment` (xmllinq/mod.rs:1025) L1073; `Dom::data_mut` (xmllinq/mod.rs:464) L1074,1075
- **Calls (std/external/unresolved):** `.push`, `Some`
- **Called by:** `Anchors::scan` (comments.rs:241); `fill_comment_paragraph` (comments.rs:1073); `recurse` (comparer/atomize.rs:410); `coalesce_recurse` (comparer/atomize.rs:575); `split_seg` (comparer/comments.rs:568); `deeper_boundary_tests::projection_rejects_document_text_and_a_detached_leaf_without_a_run` (comparer/comments.rs:2540); `coalesce_adjacent_runs` (comparer/finalize.rs:1180); `wrap_revised_hyperlink_as_field` (comparer/finalize.rs:1599); `rebuild_title_free_mesh` (comparer/finalize.rs:2771); `add_revision_text_run` (comparer/finalize.rs:2821); `rewrite_inserted_external_hyperlinks` (comparer/finalize.rs:6331); `mesh_short_label_shared_eq_with_settings` (comparer/finalize.rs:7199); +30 more

#### `Dom::add_first`  (L1085-1090, pub)
`pub fn add_first(&mut self, parent: NodeId, node: NodeId)`

- **Does:** Prepend `node` as the first child of `parent`.
- **Calls (crate):** `Dom::materialize` (xmllinq/mod.rs:1013) L1086; `Dom::validate_attachment` (xmllinq/mod.rs:1025) L1087; `Dom::data_mut` (xmllinq/mod.rs:464) L1088,1089
- **Calls (std/external/unresolved):** `.insert`, `Some`
- **Called by:** `move_last_sectpr_into_last_paragraph` (comparer/atomize.rs:63); `add_at_paragraph_start` (comparer/comments.rs:734); `mark_content_transform` (comparer/finalize.rs:192); `fix_paragraph_mark_revision_order` (comparer/finalize.rs:1484); `move_paragraph_properties_first` (comparer/finalize.rs:1524); `unwrap_hyperlinks_to_styled_runs` (comparer/finalize.rs:1555); `wrap_revised_hyperlink_as_field` (comparer/finalize.rs:1599); `promote_live_numpr_on_pure_d_from_pprchange` (comparer/finalize.rs:1866); `closing_mark_takes_revised_properties` (comparer/finalize.rs:3158); `make_empty_pure_ins_para` (comparer/finalize.rs:3783); `ensure_empty_pure_i_before_short_title_del` (comparer/finalize.rs:3910); `rotate_ins_mark_del_only_paragraph` (comparer/finalize.rs:4576); +53 more

#### `Dom::remove_nodes`  (L1094-1099, pub)
`pub fn remove_nodes(&mut self, id: NodeId)`

- **Does:** Detach every child of `id` (front matching LINQ-to-XML `RemoveNodes()`); their parent links are cleared.
- **Calls (crate):** `Dom::data_mut` (xmllinq/mod.rs:464) L1095,1097
- **Calls (std/external/unresolved):** `std::mem::take`
- **Called by:** `split_seg` (comparer/comments.rs:568); `coalesce_all_paragraphs` (comparer/finalize.rs:1461); `write_content` (edit/controls.rs:586); `write_result` (fields.rs:824); `Dom::set_value` (xmllinq/mod.rs:1203)

#### `Dom::remove`  (L1102-1104, pub)
`pub fn remove(&mut self, id: NodeId)`

- **Does:** Detach `id` from its parent.
- **Calls (crate):** `Dom::detach` (xmllinq/mod.rs:1002) L1103
- **Called by:** `drop_comments_where` (append.rs:426); `carry_notes` (append.rs:667); `ListCopy::build` (append.rs:1111); `insert_lists` (append.rs:1159); `join` (append.rs:1264); `set_section_type` (append.rs:1325); `CommentFamily::remove` (comments.rs:783); `CommentFamily::store_people` (comments.rs:977); `clear_rows` (comments.rs:1092); `move_last_sectpr_into_last_paragraph` (comparer/atomize.rs:63); `leave_foreign_controls` (comparer/bookmarks.rs:202); `leave_inline_revision` (comparer/bookmarks.rs:247); +182 more

#### `Dom::add_before_self`  (L1114-1122, pub)
`pub fn add_before_self(&mut self, reference: NodeId, node: NodeId)`

- **Does:** Insert `node` as the preceding sibling of `reference`, which must have a parent.
- **Calls (crate):** `Dom::index_in_parent` (xmllinq/mod.rs:830) L1115; `Dom::materialize` (xmllinq/mod.rs:1013) L1118; `Dom::validate_attachment` (xmllinq/mod.rs:1025) L1119; `Dom::data_mut` (xmllinq/mod.rs:464) L1120,1121
- **Calls (std/external/unresolved):** `.expect`, `.insert`, `Some`
- **Called by:** `add_in_numbering_order` (append.rs:1202); `join` (append.rs:1264); `set_section_type` (append.rs:1325); `Anchors::scan` (comments.rs:241); `leave_foreign_controls` (comparer/bookmarks.rs:202); `leave_inline_revision` (comparer/bookmarks.rs:247); `inject_side` (comparer/bookmarks.rs:266); `place_before_offset` (comparer/comments.rs:665); `place_after_offset` (comparer/comments.rs:691); `inject_side` (comparer/comments.rs:787); `unwrap_content_controls` (comparer/finalize.rs:1358); `fix_paragraph_mark_revision_order` (comparer/finalize.rs:1484); +84 more

#### `Dom::add_after_self`  (L1131-1139, pub)
`pub fn add_after_self(&mut self, reference: NodeId, node: NodeId)`

- **Does:** Insert `node` as the following sibling of `reference`.
- **Calls (crate):** `Dom::index_in_parent` (xmllinq/mod.rs:830) L1132; `Dom::materialize` (xmllinq/mod.rs:1013) L1135; `Dom::validate_attachment` (xmllinq/mod.rs:1025) L1136; `Dom::data_mut` (xmllinq/mod.rs:464) L1137,1138
- **Calls (std/external/unresolved):** `.expect`, `.insert`, `Some`
- **Called by:** `CommentFamily::add` (comments.rs:604); `leave_foreign_controls` (comparer/bookmarks.rs:202); `leave_inline_revision` (comparer/bookmarks.rs:247); `inject_side` (comparer/bookmarks.rs:266); `split_seg` (comparer/comments.rs:568); `split_run_before_leaf` (comparer/comments.rs:633); `place_before_offset` (comparer/comments.rs:665); `place_after_offset` (comparer/comments.rs:691); `add_at_paragraph_start` (comparer/comments.rs:734); `inject_side` (comparer/comments.rs:787); `step_out_of_deletion_end` (comparer/comments.rs:898); `wrap_revised_hyperlink_as_field` (comparer/finalize.rs:1599); +38 more

#### `Dom::replace_with`  (L1149-1162, pub)
`pub fn replace_with(&mut self, reference: NodeId, nodes: &[NodeId])`

- **Does:** Replace `reference` with the sequence of `nodes` (in order) at the same position.
- **Calls (crate):** `Dom::index_in_parent` (xmllinq/mod.rs:830) L1150; `Dom::materialize` (xmllinq/mod.rs:1013) L1155; `Dom::validate_attachment` (xmllinq/mod.rs:1025) L1156; `Dom::data_mut` (xmllinq/mod.rs:464) L1157,1160,1161
- **Calls (std/external/unresolved):** `.expect`, `.len`, `.push`, `.splice`, `Some`, `Vec::with_capacity`
- **Called by:** `join` (append.rs:1264); `resolve_alternate_content` (comparer/finalize.rs:1259); `closing_mark_takes_revised_mark_formatting` (comparer/finalize.rs:3358); `merge_replaced_in_container` (comparer/finalize.rs:8621); `flatten_tracked_insertions_stamped` (comparer/finalize.rs:10215); `flatten_tracked_deletions` (comparer/finalize.rs:11030); `restore_styles` (document_comparer.rs:1814); `merge_normal_style_rpr` (document_comparer.rs:4343); `resync_linked_character_styles` (revision_processor/style_records.rs:543); `tidy` (revision_processor/word_save.rs:25) (?); `process_part` (revision_processor.rs:3443)

#### `Dom::value`  (L1169-1174, pub)
`pub fn value(&self, id: NodeId) -> String`

- **Does:** Concatenated descendant text of `id` (LINQ-to-XML `element.Value`).
- **Calls (crate):** `Dom::value_str` (xmllinq/mod.rs:1179) L1170
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `text_of` (changes.rs:806); `comment_text` (comments.rs:1045); `atom_hash` (comparer/atomize.rs:163); `salt_field_results` (comparer/atomize.rs:217); `recurse` (comparer/atomize.rs:410); `coverage_atomizer_field_chain_boundaries::header_and_footer_roots_stop_paths_and_preserve_each_whitespace_boundary` (comparer/atomize.rs:1123); `extract` (comparer/bookmarks.rs:82); `comment_definition_fingerprint` (comparer/comments.rs:79); `split_seg` (comparer/comments.rs:568); `tests::texts_and_markers` (comparer/comments.rs:1699); `unit_text` (comparer/cross_para.rs:556); `public_option_source_ownership_tests::source_events::{fn walk}` (comparer/cross_para.rs:2513); +66 more

#### `Dom::value_str`  (L1179-1189, pub)
`pub fn value_str(&self, id: NodeId) -> std::borrow::Cow<'_, str>`

- **Does:** Borrowed text when `id` has exactly one direct text child; otherwise owned concatenated descendant text.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L1180,1182; `Dom::collect_text` (xmllinq/mod.rs:1191) L1187
- **Calls (std/external/unresolved):** `.as_str`, `.len`, `String::new`, `std::borrow::Cow::Borrowed`, `std::borrow::Cow::Owned`
- **Called by:** `Package::fields` (audit.rs:764) (?); `coalesce_recurse` (comparer/atomize.rs:575); `run_text_concat` (comparer/finalize.rs:1153); `wrap_revised_hyperlink_as_field` (comparer/finalize.rs:1599); `para_side_word_count` (comparer/finalize.rs:2013); `para_keeps_unchanged_words` (comparer/finalize.rs:2034); `free_mesh_shared_title_token_in_mix` (comparer/finalize.rs:2531); `split_head_short_title_long_del_mix` (comparer/finalize.rs:4829); `fold_leading_ins_from_mix_into_preceding_pure_del` (comparer/finalize.rs:4950); `hyperlink_is_wholly_inserted` (comparer/finalize.rs:6393); `mesh_short_label_shared_eq_with_settings` (comparer/finalize.rs:7199); `mesh_short_label_shared_eq_with_settings::{fn label_run}` (comparer/finalize.rs:7317); +68 more

#### `Dom::collect_text`  (L1191-1199)
`fn collect_text(&self, id: NodeId, s: &mut String)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L1192,1193; `Dom::collect_text` (xmllinq/mod.rs:1191) L1195
- **Calls (std/external/unresolved):** `.push_str`
- **Called by:** `Element::collect_text` (markdown/from_docx/ooxml.rs:350) (?); `Dom::value_str` (xmllinq/mod.rs:1179); `Dom::collect_text` (xmllinq/mod.rs:1191)

#### `Dom::set_value`  (L1203-1206, pub)
`pub fn set_value(&mut self, id: NodeId, value: &str)`

- **Does:** Replace `id`'s children with a single text child carrying `value` (LINQ-to-XML `element.Value = value`).
- **Calls (crate):** `Dom::remove_nodes` (xmllinq/mod.rs:1094) L1204; `Dom::add_text` (xmllinq/mod.rs:1071) L1205
- **Called by:** `mesh_short_label_shared_eq_with_settings` (comparer/finalize.rs:7199); `fix_strict_validity_artifacts` (comparer/finalize.rs:11321); `append_period` (comparer/finalize.rs:12953); `split_insertion_at` (comparer/finalize.rs:15028); `format_change_cache_tests::run_atom` (comparer/formatchg.rs:693); `memoized_moves_tests::make_pool` (comparer/moves.rs:858); `set_text` (edit.rs:5721)

#### `Dom::clone_subtree`  (L1215-1242, pub)
`pub fn clone_subtree(&mut self, id: NodeId) -> NodeId`

- **Does:** Deep-clone the subtree rooted at `id` into the same arena and return the new root.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L1216,1226; `Dom::child_count` (xmllinq/mod.rs:622) L1225; `Dom::alloc` (xmllinq/mod.rs:394) L1227; `Dom::data_mut` (xmllinq/mod.rs:464) L1229,1238,1239; `Dom::child_at` (xmllinq/mod.rs:632) L1235; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L1236; `Dom::validate_attachment` (xmllinq/mod.rs:1025) L1237
- **Calls (std/external/unresolved):** `.clone`, `.push`, `.reserve_exact`, `NodeKind::Comment`, `NodeKind::Pi`, `NodeKind::Text`, `Some`
- **Called by:** `append_documents` (append.rs:152); `stage_comments` (append.rs:557); `carry_notes` (append.rs:667); `carry_styles_and_numbering` (append.rs:861); `ListCopy::build` (append.rs:1111); `join` (append.rs:1264); `move_last_sectpr_into_last_paragraph` (comparer/atomize.rs:63); `coalesce_recurse` (comparer/atomize.rs:575); `split_seg` (comparer/comments.rs:568); `split_run_before_leaf` (comparer/comments.rs:633); `add_at_paragraph_start` (comparer/comments.rs:734); `union_comments_xml` (comparer/comments.rs:1190); +169 more

#### `Dom::add_annotation`  (L1254-1259, pub)
`pub fn add_annotation<T: Any + 'static>(&mut self, id: NodeId, annotation: T)`

- **Does:** Attach a typed annotation to `id`.
- **Calls (std/external/unresolved):** `.entry`, `.or_default`, `.push`, `Box::new`
- **Called by:** none found in src

#### `Dom::annotation`  (L1261-1266, pub)
`pub fn annotation<T: Any + 'static>(&self, id: NodeId) -> Option<&T>`

- **Does:** The first annotation of type `T` attached to `id`, or `None`.
- **Calls (std/external/unresolved):** `.downcast_ref`, `.find_map`, `.get`, `.iter`
- **Called by:** none found in src

#### `Dom::remove_annotations`  (L1270-1277, pub)
`pub fn remove_annotations<T: Any + 'static>(&mut self, id: NodeId)`

- **Does:** Drop every annotation of type `T` from `id`.
- **Calls (crate):** `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L1272
- **Calls (std/external/unresolved):** `.get_mut`, `.is_empty`, `.remove`, `.retain`
- **Called by:** none found in src

#### `Dom::parse_xdocument`  (L1283-1285, pub)
`pub fn parse_xdocument(&mut self, xml: &str) -> NodeId`

- **Does:** Parse a full XML string (with optional prolog) into a Document node allocated into this arena.
- **Calls (crate):** `parse_xdocument` (xmllinq/parse.rs:101) L1284
- **Called by:** `parse` (append.rs:330); `drop_comments_in_new_parts` (append.rs:450); `insert_lists` (append.rs:1159); `nsids` (append.rs:1226); `tests::nsid_collisions_take_the_next_free_value` (append.rs:1351); `tests::section_type_sits_after_header_references` (append.rs:1398); `tests::ignorable_prefixes_merge_only_with_the_same_namespace` (append.rs:1417); `public_memory_append_owner_boundary_tests::view` (append.rs:1504); `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1594); `changes_in` (changes.rs:302); `CommentFamily::load` (comments.rs:429); `CommentFamily::comments_root` (comments.rs:644); +458 more

#### `Dom::try_parse_xdocument`  (L1294-1296, pub)
`pub fn try_parse_xdocument(&mut self, xml: &str) -> Result<NodeId, String>`

- **Does:** Checked variant of [`Self::parse_xdocument`]: refuses input that fails [`parse::validate_xml`] (including the tag-width and namespace budgets) before any DOM node is built.
- **Calls (crate):** `try_parse_xdocument` (xmllinq/parse.rs:122) L1295
- **Called by:** none found in src

#### `Dom::serialize_element`  (L1299-1301, pub)
`pub fn serialize_element(&self, el: NodeId) -> String`

- **Does:** Serialize an element subtree to an XML string.
- **Calls (crate):** `serialize_element` (xmllinq/serialize.rs:401) L1300
- **Called by:** `append_documents` (append.rs:152); `ListCopy::build` (append.rs:1111); `final_section_table_ownership_tests::a_table_only_story_keeps_its_final_section_outside_all_cell_paragraphs` (comparer/atomize.rs:747) (?); `final_section_table_ownership_tests::block_control_paragraph_still_owns_the_closing_section_instead_of_later_table_cells` (comparer/atomize.rs:771) (?); `final_section_table_ownership_tests::existing_cell_section_properties_and_history_are_never_replaced_by_body_geometry` (comparer/atomize.rs:794) (?); `coverage_atomizer_owned_boundaries::note_definition_owns_every_character_and_mark_but_the_part_is_a_path_stop` (comparer/atomize.rs:827) (?); `coverage_atomizer_owned_boundaries::tracked_payload_status_and_paragraph_mark_status_have_independent_owners` (comparer/atomize.rs:973) (?); `coverage_atomizer_leaf_stamp_boundaries::opaque_leaf_stamps_salt_original_deletions_once_and_preserve_the_authored_payload` (comparer/atomize.rs:1213); `union_comments_xml` (comparer/comments.rs:1190); `drop_orphans` (comparer/comments.rs:1416); `rpr_string` (comparer/finalize.rs:1048); `coalesce_key` (comparer/finalize.rs:1055); +274 more

#### `Dom::serialize_element_sha1_hex`  (L1309-1311, pub)
`pub fn serialize_element_sha1_hex(&self, el: NodeId) -> String`

- **Does:** SHA-1 (lowercase hex) of the serialized element with the first WML default-xmlns declaration stripped from the root open tag.
- **Calls (crate):** `serialize_element_sha1_hex` (xmllinq/serialize.rs:418) L1310
- **Called by:** `block_sha1` (comparer/preprocess.rs:641)

#### `Dom::serialize_element_structure_sha1_hex`  (L1317-1319, pub)
`pub fn serialize_element_structure_sha1_hex(&self, el: NodeId) -> String`

- **Does:** SHA-1 (lowercase hex) of the structural shape only: attributes and element nesting, no text / comment / PI children, with the same first-WML-default-xmlns strip as [`Self::serialize_element_sha1_hex`].
- **Calls (crate):** `serialize_element_structure_sha1_hex` (xmllinq/serialize.rs:432) L1318
- **Called by:** `structure_sha1` (comparer/preprocess.rs:1213)

#### `Dom::serialize_document`  (L1322-1324, pub)
`pub fn serialize_document(&self, doc: NodeId) -> String`

- **Does:** Serialize a whole document (declaration + children) to a string.
- **Calls (crate):** `serialize_document` (xmllinq/serialize.rs:793) L1323
- **Called by:** `append_documents` (append.rs:152); `drop_comments_in_new_parts` (append.rs:450); `carry_styles_and_numbering` (append.rs:861); `merge_part_namespaces` (append.rs:1081); `insert_lists` (append.rs:1159); `CommentFamily::store` (comments.rs:798); `CommentFamily::store_people` (comments.rs:977); `carry_with_own_relationships` (comparer/parts.rs:627); `complete_shape_revision_contract_tests::matching_opaque_shapes_keep_every_neighbor_when_revision_wrappers_split` (comparer/textbox.rs:260) (?); `complete_shape_revision_contract_tests::unrelated_or_mixed_shape_carriers_retain_complete_source_revisions` (comparer/textbox.rs:310) (?); `raw_source_projection_contract_tests::with_source` (debug/diff.rs:2509); `carry_revised_part_relationships` (document_comparer.rs:25); +21 more

## Test functions (5)

- `bump_attr_name_probes` (L283): Add one to this thread's [`ATTR_NAME_PROBES`].
- `Dom::shrink_arena_to_fit` (L422): Shrink the arena's backing `Vec` capacity to its current length so any subsequent internal push provably reallocates.
- `node_layout_tests::node_data_excludes_annotations_vec` (L1339): ANN-01 mechanism counter.
- `node_layout_tests::node_kind_rare_variants_are_boxed` (L1357): NODE-KIND-01 mechanism counter.
- `intern_tests::identical_names_share_interned_arcs` (L1384): PARSE-01 mechanism proof.
