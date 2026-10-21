# Functions and methods in `src/xmllinq/mod.rs`

105 non-test, 6 test. Part of the [symbol manifest](../README.md).

#### `intern_str`  (L75-91)
`fn intern_str(s: &str) -> Arc<str>`

- **Does:** Return a shared interned `Arc<str>` for `s`, allocating only on first sight.
- **Calls (std/external/unresolved):** `.borrow_mut`, `.borrow`, `.clone`, `.get`, `.insert`, `.len`, `.with`, `Arc::from`
- **Called by:** `XNamespace::get` (xmllinq/mod.rs:108); `XName::get` (xmllinq/mod.rs:180)

#### `XNamespace::get`  (L108-112, pub)
`pub fn get(namespace_name: &str) -> XNamespace`

- **Does:** Construct a namespace, interning `namespace_name` for sharing across the thread (a no-op once the pool is saturated).
- **Calls (crate):** `intern_str` (xmllinq/mod.rs:75) L110
- **Called by:** `fix_strict_validity_artifacts` (comparer/finalize.rs:11326); `coverage_final_batch_finalize_tests::inherited_invalidity_repairs_only_wml_schema_defects` (comparer/finalize.rs:24583); `deep_lcs_source_conservation_tests::opaque_object_and_foreign_namespace_payloads_are_contentful_without_text` (comparer/lcs.rs:17317); `package_tree` (debug/diff.rs:303); `holds_watermark` (edit/watermark.rs:156); `XNamespace::none` (xmllinq/mod.rs:116); `XNamespace::xmlns` (xmllinq/mod.rs:122); `XNamespace::xml` (xmllinq/mod.rs:128); `XName::get` (xmllinq/mod.rs:180); `resolve` (xmllinq/parse.rs:837); `tag_policy_tests::a_word_gfxdata_attribute_is_admitted` (xmllinq/parse.rs:1451); `tests::word_2016_plus_namespaces_serialize_with_word_prefixes` (xmllinq/serialize.rs:859); +2 more

#### `XNamespace::none`  (L116-118, pub)
`pub fn none() -> XNamespace`

- **Does:** The empty namespace (no URI).
- **Calls (crate):** `XNamespace::get` (xmllinq/mod.rs:108) L117
- **Called by:** `choice_requires_understood` (comparer/finalize.rs:1479); `fix_strict_validity_artifacts` (comparer/finalize.rs:11326); `coverage_final_batch_finalize_tests::inherited_invalidity_repairs_only_wml_schema_defects` (comparer/finalize.rs:24583); `tests::attribute_and_text_whitespace_survive_serialization_round_trips` (xmllinq/serialize.rs:1022); `xml_structure_namespace_boundary_tests::structure_hash_keeps_mc_required_namespace_ownership_while_omitting_payload` (xmllinq/serialize.rs:1070)

#### `XNamespace::xmlns`  (L122-124, pub)
`pub fn xmlns() -> XNamespace`

- **Does:** The reserved `xmlns:` declarations namespace (`http://www.w3.org/2000/xmlns/`).
- **Calls (crate):** `XNamespace::get` (xmllinq/mod.rs:108) L123
- **Called by:** `declare_ignorable` (comments.rs:1199); `coalesce` (comparer/atomize.rs:557); `preserve_cloned_namespace_context` (comparer/comments.rs:1052); `ignore_pt14_namespace` (comparer/finalize.rs:862); `declare_extension_namespaces_ignorable` (comparer/finalize.rs:942); `bind_compatibility_prefixes` (comparer/finalize.rs:1026); `coverage_final_batch_finalize_tests::compatibility_bindings_keep_conflicts_and_recognize_attribute_only_extensions` (comparer/finalize.rs:24643); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215); `produce_document` (comparer/produce.rs:65); `keyed_groups` (comparer/revisions.rs:88); `compare_documents_impl` (document_comparer.rs:6520); `Transaction::mark_header` (edit/watermark.rs:285); +2 more

#### `XNamespace::xml`  (L128-130, pub)
`pub fn xml() -> XNamespace`

- **Does:** The reserved `xml:` declarations namespace (`http://www.w3.org/XML/1998/namespace`).
- **Calls (crate):** `XNamespace::get` (xmllinq/mod.rs:108) L129
- **Called by:** `fill_comment_paragraph` (comments.rs:1129); `coalesce_recurse` (comparer/atomize.rs:575); `coverage_atomizer_owned_boundaries::note_definition_owns_every_character_and_mark_but_the_part_is_a_path_stop` (comparer/atomize.rs:827); `coverage_atomizer_field_chain_boundaries::header_and_footer_roots_stop_paths_and_preserve_each_whitespace_boundary` (comparer/atomize.rs:1123); `split_seg` (comparer/comments.rs:568); `deeper_boundary_tests::styled_unicode_split_preserves_properties_and_trailing_nodes` (comparer/comments.rs:2098); `coalesce_adjacent_runs` (comparer/finalize.rs:1222); `rebuild_title_free_mesh` (comparer/finalize.rs:2813); `add_revision_text_run` (comparer/finalize.rs:2863); `mesh_short_label_shared_eq_with_settings` (comparer/finalize.rs:7204); `rebuild_body_free_mesh_lcs` (comparer/finalize.rs:13851); `m460_push_eq_text` (comparer/finalize.rs:14292); +28 more

#### `XNamespace::name`  (L134-136, pub)
`pub fn name(&self, local: &str) -> XName`

- **Does:** Build the [`XName`] `{ns}local` — i.e.
- **Calls (crate):** `XName::get` (xmllinq/mod.rs:180) L135
- **Called by:** `declare_ignorable` (comments.rs:1199); `declare_extension_namespaces_ignorable` (comparer/finalize.rs:942); `bind_compatibility_prefixes` (comparer/finalize.rs:1026); `fix_strict_validity_artifacts::{fn remap}` (comparer/finalize.rs:11395); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215); `Transaction::mark_header` (edit/watermark.rs:285); `tag_policy_tests::a_word_gfxdata_attribute_is_admitted` (xmllinq/parse.rs:1451)

#### `XNamespace::namespace_name`  (L140-142, pub)
`pub fn namespace_name(&self) -> &str`

- **Does:** The namespace URI as a borrowed string.
- **Calls:** none
- **Called by:** `w_vals` (append.rs:818) (?); `set_section_type` (append.rs:1331) (?); `public_memory_append_owner_boundary_tests::semantic` (append.rs:1454) (?); `walk` (changes/logical.rs:399) (?); `audit_stories` (changes.rs:168) (?); `carriers` (changes.rs:1124) (?); `in_recorded_properties` (changes.rs:1406) (?); `recorded_marks` (changes.rs:1419) (?); `comment_text` (comments.rs:1101) (?); `preserve_cloned_namespace_context` (comparer/comments.rs:1052) (?); `deeper_boundary_tests::namespace_conflicts_reuse_aliases_or_skip_occupied_generated_prefixes` (comparer/comments.rs:2317) (?); `public_option_source_ownership_tests::canonical` (comparer/cross_para.rs:2479) (?); +141 more

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
- **Called by:** `merge_namespace_declarations` (append.rs:371); `tests::ignorable_prefixes_merge_only_with_the_same_namespace` (append.rs:1423); `Package::image_no_descr` (audit.rs:558); `decorative` (audit.rs:942); `resolve_for_edit_or_cli` (changes.rs:922); `carriers` (changes.rs:1124); `CommentFamily::read_meta` (comments.rs:485); `CommentFamily::reported_date` (comments.rs:637); `CommentFamily::thread_state` (comments.rs:716); `CommentFamily::set_date_utc` (comments.rs:726); `CommentFamily::adopt` (comments.rs:743); `CommentFamily::store` (comments.rs:854); +49 more

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
- **Called by:** `elements` (admission.rs:967) (?); `is_comment_anchor` (append.rs:423) (?); `is_annotation` (append.rs:494); `w_vals` (append.rs:818) (?); `set_section_type` (append.rs:1331) (?); `tests::section_type_sits_after_header_references` (append.rs:1404) (?); `public_memory_append_owner_boundary_tests::semantic` (append.rs:1454) (?); `decorative` (audit.rs:942) (?); `walk` (changes/logical.rs:399) (?); `audit_stories` (changes.rs:168) (?); `audit_owner` (changes.rs:283) (?); `resolve_for_edit_or_cli` (changes.rs:922) (?); +262 more

#### `XName::namespace`  (L213-215, pub)
`pub fn namespace(&self) -> &XNamespace`

- **Does:** The namespace half of the expanded name.
- **Calls:** none
- **Called by:** `Build::noise` (debug/diff.rs:504); `Build::namespace_note` (debug/diff.rs:564) (?); `Build::attrs` (debug/diff.rs:593) (?); `effective_chain_props` (document_comparer.rs:1696) (?); `complete_attributes` (document_comparer.rs:2054) (?)

#### `XName::namespace_name`  (L219-221, pub)
`pub fn namespace_name(&self) -> &str`

- **Does:** The namespace URI as a borrowed string.
- **Calls (crate):** `XNamespace::namespace_name` (xmllinq/mod.rs:140) L220
- **Called by:** `is_comment_anchor` (append.rs:423); `is_annotation` (append.rs:494); `w_vals` (append.rs:818) (?); `set_section_type` (append.rs:1331) (?); `public_memory_append_owner_boundary_tests::semantic` (append.rs:1454) (?); `walk` (changes/logical.rs:399) (?); `audit_stories` (changes.rs:168) (?); `carriers` (changes.rs:1124) (?); `in_recorded_properties` (changes.rs:1406) (?); `recorded_marks` (changes.rs:1419) (?); `comment_text` (comments.rs:1101) (?); `preserve_cloned_namespace_context` (comparer/comments.rs:1052) (?); +156 more

#### `XName::clark`  (L226-232, pub)
`pub fn clark(&self) -> String`

- **Does:** The clark-notation form: `"{ns}local"` for a non-empty namespace, or just `"local"` for the empty one.
- **Calls (std/external/unresolved):** `.is_empty`, `.to_string`
- **Macros:** `format!`
- **Called by:** `get_changed_property_names` (comparer/formatchg.rs:217) (?); `assert_tree` (document_comparer/based_style_record_tests.rs:34) (?); `style_prop_signature` (document_comparer.rs:1485) (?); `effective_chain_props` (document_comparer.rs:1696) (?); `tests::restored_deleted_spacing_leaves_scratch_unids_behind` (document_comparer.rs:8100) (?); `coverage_boundary_tests::effective_run_properties_merge_independent_slots_and_clear_alternatives` (document_comparer.rs:9022) (?); `coverage_boundary_tests::style_cycles_stop_and_self_based_style_does_not_hide_own_properties` (document_comparer.rs:9059) (?); `authored_style_chain_boundary_tests::leaf` (document_comparer.rs:12161) (?); `authored_style_chain_boundary_tests::attrs` (document_comparer.rs:12168) (?); `authored_style_chain_boundary_tests::inherited_style_font_alternatives_and_missing_blocks_keep_exact_declarations` (document_comparer.rs:12177) (?); `authored_style_chain_boundary_tests::based_on_cycles_stop_after_unique_authored_owners_with_nearest_properties_winning` (document_comparer.rs:12222) (?); `authored_style_chain_boundary_tests::bounded_style_walk_reads_exactly_the_nearest_thirty_two_owners` (document_comparer.rs:12244) (?); +5 more

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
- **Called by:** `append_documents` (append.rs:152); `drop_comments_in_new_parts` (append.rs:456); `insert_lists` (append.rs:1165); `nsids` (append.rs:1232); `tests::nsid_collisions_take_the_next_free_value` (append.rs:1357); `tests::section_type_sits_after_header_references` (append.rs:1404); `tests::ignorable_prefixes_merge_only_with_the_same_namespace` (append.rs:1423); `public_memory_append_owner_boundary_tests::view` (append.rs:1509); `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1599); `audit_stories` (changes.rs:168); `for_each_story` (changes.rs:657); `comment_changes_in` (changes.rs:1003); +754 more

#### `Dom::alloc`  (L394-398)
`fn alloc(&mut self, kind: NodeKind) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `NodeData::new` (xmllinq/mod.rs:350) L396
- **Calls (std/external/unresolved):** `.len`, `.push`, `NodeId`
- **Called by:** `Dom::new_document` (xmllinq/mod.rs:472); `Dom::new_element` (xmllinq/mod.rs:478); `Dom::new_text` (xmllinq/mod.rs:483); `Dom::new_comment` (xmllinq/mod.rs:487); `Dom::new_pi` (xmllinq/mod.rs:492); `Dom::clone_subtree` (xmllinq/mod.rs:1231)

#### `Dom::node_count`  (L402-404, pub)
`pub fn node_count(&self) -> usize`

- **Does:** Number of nodes currently in the arena.
- **Calls (std/external/unresolved):** `.len`
- **Called by:** `coverage_real_revision_merge_matrix_tests::paired_property_comparison_never_grows_the_persistent_dom_or_changes_sources` (comparer/finalize.rs:26414); `finalizer_remaining_source_boundary_tests::paired_format_equality_is_complete_and_uses_no_production_arena_nodes` (comparer/finalize.rs:31206) (?); `public_body_heuristic_story_scope_tests::public_body_layout_heuristics_preserve_complete_independent_header_and_footer_parts` (comparer/finalize.rs:32488); `interleave_terminal_source_ownership_tests::remains_source_owned` (comparer/finalize.rs:33328) (?); `interleave_terminal_source_ownership_tests::split_inserted_title_fragments_do_not_take_another_authors_pending_revision` (comparer/finalize.rs:33433) (?); `whitespace_source_mark_carrier_tests::whitespace_fold_preserves_selected_source_mark_shells_and_every_body_owner` (comparer/finalize.rs:33565) (?); `fragmented_source_label_boundary_tests::fragmented_labels_keep_literal_source_chars_formats_and_revision_history` (comparer/finalize.rs:33728) (?); `fragmented_source_label_boundary_tests::label_format_boundaries_and_disabled_history_leave_complete_original_sources_intact` (comparer/finalize.rs:33882) (?); `fragmented_source_label_boundary_tests::public_label_zip_preserves_already_adjacent_spacer_and_missing_layout_ownership` (comparer/finalize.rs:34014) (?); `fragmented_source_label_boundary_tests::wholesale_glue_only_lcs_preserves_crossed_significant_source_anchors` (comparer/finalize.rs:34112) (?); `nested_deleted_text_source_ownership_tests::outer_deletion_preserves_independently_owned_nested_revision_text_and_fields` (comparer/finalize.rs:34181) (?); `reached_word_document_policy_tests::document_policy_reaches_numbering_relatedness_and_demo_intro_boundaries` (comparer/finalize.rs:34432); +5 more

#### `Dom::node_capacity`  (L413-415, pub)
`pub fn node_capacity(&self) -> usize`

- **Does:** Allocated node-slot capacity of the arena's backing `Vec`.
- **Calls (std/external/unresolved):** `.capacity`
- **Called by:** `coverage_real_revision_merge_matrix_tests::paired_property_comparison_never_grows_the_persistent_dom_or_changes_sources` (comparer/finalize.rs:26414); `format_change_cache_tests::detect_format_changes_never_grows_production_arena_capacity` (comparer/formatchg.rs:651)

#### `Dom::with_scratch`  (L449-459, pub)
`pub fn with_scratch<R>(&mut self, f: impl FnOnce(&mut Dom) -> R) -> R`

- **Does:** Run `f` and then reclaim every node it allocated by truncating the arena back to its pre-call length.
- **Calls (std/external/unresolved):** `.is_empty`, `.len`, `.retain`, `.truncate`, `f`
- **Called by:** `paired_replacement_has_identical_properties` (comparer/finalize.rs:8529); `normalized_rpr_serialized` (comparer/formatchg.rs:132)

#### `Dom::data`  (L461-463)
`fn data(&self, id: NodeId) -> &NodeData`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `tests::hidden_text_paints_nothing_but_reads_back` (convert/raster.rs:748) (?); `raster_payload_boundary_tests::image_alpha_composites_into_white_and_degenerate_placement_is_identity` (convert/raster.rs:952) (?); `raster_payload_boundary_tests::rgb_missing_alpha_defaults_only_missing_pixels_to_opaque` (convert/raster.rs:995) (?); `raster_payload_boundary_tests::empty_shape_payloads_preserve_pixels_and_negative_rectangle_extents_are_normalized` (convert/raster.rs:1060) (?); `residual_display_contract_tests::empty_glyphs_and_nonpositive_display_sizes_never_load_or_paint_a_face` (convert/raster.rs:1154) (?); `residual_display_contract_tests::collapsed_vector_subpaths_and_rejected_encoded_picture_payloads_preserve_all_pixels` (convert/raster.rs:1274) (?); `residual_display_contract_tests::memory_png_roundtrip_preserves_exact_unrotated_page_size_and_pixels` (convert/raster.rs:1390) (?); `Dom::is_element` (xmllinq/mod.rs:502) (?); `Dom::is_text` (xmllinq/mod.rs:506) (?); `Dom::is_document` (xmllinq/mod.rs:511) (?); `Dom::is_comment` (xmllinq/mod.rs:515) (?); `Dom::is_pi` (xmllinq/mod.rs:519) (?); +36 more

#### `Dom::data_mut`  (L464-466)
`fn data_mut(&mut self, id: NodeId) -> &mut NodeData`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `paint_image` (convert/raster.rs:543) (?); `Dom::set_name` (xmllinq/mod.rs:539); `Dom::set_text_value` (xmllinq/mod.rs:555); `Dom::set_declaration` (xmllinq/mod.rs:589); `Dom::set_attributes` (xmllinq/mod.rs:935); `Dom::set_attribute_value` (xmllinq/mod.rs:977); `Dom::detach` (xmllinq/mod.rs:1018); `Dom::add` (xmllinq/mod.rs:1074); `Dom::add_text` (xmllinq/mod.rs:1087); `Dom::add_first` (xmllinq/mod.rs:1101); `Dom::remove_nodes` (xmllinq/mod.rs:1110); `Dom::add_before_self` (xmllinq/mod.rs:1130); +3 more

#### `Dom::new_document`  (L472-474, pub)
`pub fn new_document(&mut self) -> NodeId`

- **Does:** Allocate a Document (root container) node, with no declaration.
- **Calls (crate):** `Dom::alloc` (xmllinq/mod.rs:394) L473
- **Called by:** `coalesce` (comparer/atomize.rs:557); `deeper_boundary_tests::projection_rejects_document_text_and_a_detached_leaf_without_a_run` (comparer/comments.rs:2540); `produce_document` (comparer/produce.rs:65); `Parser::parse_document` (xmllinq/parse.rs:654)

#### `Dom::new_element`  (L478-480, pub)
`pub fn new_element(&mut self, name: XName) -> NodeId`

- **Does:** Allocate an element node with the given expanded name.
- **Calls (crate):** `Dom::alloc` (xmllinq/mod.rs:394) L479
- **Called by:** `append_documents` (append.rs:152); `stage_comments` (append.rs:563); `carry_notes` (append.rs:673); `join` (append.rs:1270); `set_section_type` (append.rs:1331); `Anchors::scan` (comments.rs:251); `CommentFamily::add` (comments.rs:648) (?); `CommentFamily::set_date_utc` (comments.rs:726); `CommentFamily::adopt` (comments.rs:743) (?); `CommentFamily::set_text` (comments.rs:793) (?); `CommentFamily::store` (comments.rs:854); `CommentFamily::store_people` (comments.rs:1033); +285 more

#### `Dom::new_text`  (L483-485, pub)
`pub fn new_text(&mut self, value: &str) -> NodeId`

- **Does:** Allocate a text node carrying `value`.
- **Calls (crate):** `Dom::alloc` (xmllinq/mod.rs:394) L484
- **Calls (std/external/unresolved):** `.to_string`, `NodeKind::Text`
- **Called by:** `split_seg` (comparer/comments.rs:568); `deeper_boundary_tests::projection_rejects_document_text_and_a_detached_leaf_without_a_run` (comparer/comments.rs:2540); `coverage_tests::revision_wrapper_requires_consistent_hyperlink_stamps` (comparer/finalize.rs:15859); `clone_internal_unsalted` (comparer/preprocess.rs:245) (?); `coverage_boundary_tests::style_cycles_stop_and_self_based_style_does_not_hide_own_properties` (document_comparer.rs:9059); `tests::settled_paragraph_unprojected_content_is_not_content_free` (edit/origin.rs:1101); `Dom::add_text` (xmllinq/mod.rs:1087); `Parser::parse_element` (xmllinq/parse.rs:709)

#### `Dom::new_comment`  (L487-489, pub)
`pub fn new_comment(&mut self, value: &str) -> NodeId`

- **Does:** Allocate a comment node (`<!-- value -->`).
- **Calls (crate):** `Dom::alloc` (xmllinq/mod.rs:394) L488
- **Calls (std/external/unresolved):** `.to_string`, `NodeKind::Comment`
- **Called by:** `tests::settled_paragraph_unprojected_content_is_not_content_free` (edit/origin.rs:1101); `Parser::parse_document` (xmllinq/parse.rs:654); `Parser::parse_element` (xmllinq/parse.rs:709)

#### `Dom::new_pi`  (L492-497, pub)
`pub fn new_pi(&mut self, target: &str, data: &str) -> NodeId`

- **Does:** Allocate a processing instruction node (`<?target data?>`).
- **Calls (crate):** `Dom::alloc` (xmllinq/mod.rs:394) L493
- **Calls (std/external/unresolved):** `.to_string`, `Box::new`, `NodeKind::Pi`
- **Called by:** `Parser::parse_pi_node` (xmllinq/parse.rs:808)

#### `Dom::is_element`  (L502-504, pub)
`pub fn is_element(&self, id: NodeId) -> bool`

- **Does:** True if `id` is an element node.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) (by name) L503
- **Macros:** `matches!`
- **Called by:** `public_memory_append_owner_boundary_tests::semantic` (append.rs:1454); `extract_events` (comparer/comments.rs:260); `descendants_trimmed::{fn walk}` (comparer/finalize.rs:65); `convert_run_text_to_del_text::{fn walk}` (comparer/finalize.rs:103); `mark_content_transform` (comparer/finalize.rs:192); `conjoin_transform` (comparer/finalize.rs:628); `move_paragraph_properties_first` (comparer/finalize.rs:1566); `paragraph_mark_properties_equal::{fn signature}` (comparer/finalize.rs:3342); `reorder_replacements_ins_before_del` (comparer/finalize.rs:6657); `revision_wrapper` (comparer/finalize.rs:6712); `coalesce_adjacent_revisions` (comparer/finalize.rs:6764); `simplify_move_transform` (comparer/finalize.rs:6818); +94 more

#### `Dom::is_text`  (L506-508, pub)
`pub fn is_text(&self, id: NodeId) -> bool`

- **Does:** True if `id` is a text node.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) (by name) L507
- **Calls (std/external/unresolved):** `NodeKind::Text`
- **Macros:** `matches!`
- **Called by:** `extract` (comparer/bookmarks.rs:92); `extract_events` (comparer/comments.rs:260); `collect_segments` (comparer/comments.rs:477); `strip_whitespace_in_clone_text` (comparer/preprocess.rs:96); `clone_internal_unsalted` (comparer/preprocess.rs:245); `text_lines::{fn runs}` (debug.rs:1688); `alternate_content_paragraph_owner_tests::semantic` (revision_processor.rs:5470); `xml_holds` (scrub.rs:528); `emit` (xmllinq/serialize.rs:668); `serialize_document` (xmllinq/serialize.rs:805)

#### `Dom::is_document`  (L511-513, pub)
`pub fn is_document(&self, id: NodeId) -> bool`

- **Does:** True if `id` is the document container (allocation created with [`Self::new_document`] or via [`Self::parse_xdocument`]).
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) (by name) L512
- **Macros:** `matches!`
- **Called by:** `Dom::document` (xmllinq/mod.rs:809); `Dom::validate_attachment` (xmllinq/mod.rs:1041)

#### `Dom::is_comment`  (L515-517, pub)
`pub fn is_comment(&self, id: NodeId) -> bool`

- **Does:** True if `id` is a comment node.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) (by name) L516
- **Calls (std/external/unresolved):** `NodeKind::Comment`
- **Macros:** `matches!`
- **Called by:** `xml_holds` (scrub.rs:528); `emit` (xmllinq/serialize.rs:668); `serialize_document` (xmllinq/serialize.rs:805)

#### `Dom::is_pi`  (L519-521, pub)
`pub fn is_pi(&self, id: NodeId) -> bool`

- **Does:** True if `id` is a processing-instruction node.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) (by name) L520
- **Calls (std/external/unresolved):** `NodeKind::Pi`
- **Macros:** `matches!`
- **Called by:** `xml_holds` (scrub.rs:528); `xml_lexical_boundary_contract_tests::legacy_declaration_defaults_keep_readable_roots_without_inventing_metadata` (xmllinq/parse.rs:1351) (?); `emit` (xmllinq/serialize.rs:668); `serialize_document` (xmllinq/serialize.rs:805); `tests::document_serialization_preserves_declarations_comments_and_processing_instructions` (xmllinq/serialize.rs:885)

#### `Dom::name`  (L526-531, pub)
`pub fn name(&self, id: NodeId) -> Option<XName>`

- **Does:** The element's expanded name, or `None` for non-element nodes.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L527
- **Calls (std/external/unresolved):** `.clone`, `Some`
- **Called by:** `comment_anchors` (append.rs:416); `offset_annotation_ids` (append.rs:501); `w_vals` (append.rs:818); `add_in_numbering_order` (append.rs:1208); `set_section_type` (append.rs:1331); `tests::section_type_sits_after_header_references` (append.rs:1404); `decorative` (audit.rs:942); `walk` (changes/logical.rs:399); `audit_stories` (changes.rs:168); `audit_owner` (changes.rs:283); `carriers` (changes.rs:1124); `fold_style_pairs` (changes.rs:1342); +333 more

#### `Dom::name_is`  (L535-537, pub)
`pub fn name_is(&self, id: NodeId, name: &XName) -> bool`

- **Does:** True if `id` is an element whose name equals `name`.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) (by name) L536
- **Macros:** `matches!`
- **Called by:** `final_sect_pr` (append.rs:360); `drop_comments_where` (append.rs:432); `Package::fields` (audit.rs:764) (?); `fold_twins` (changes.rs:1280); `fold_style_pairs` (changes.rs:1342); `CommentFamily::set_text` (comments.rs:793) (?); `revision_tracking_element_from_ancestors` (comparer/atomize.rs:112); `atom_hash` (comparer/atomize.rs:163); `coverage_atomizer_owned_boundaries::note_definition_owns_every_character_and_mark_but_the_part_is_a_path_stop` (comparer/atomize.rs:827) (?); `coverage_atomizer_field_chain_boundaries::header_and_footer_roots_stop_paths_and_preserve_each_whitespace_boundary` (comparer/atomize.rs:1123) (?); `paragraph_of` (comparer/bookmarks.rs:76); `extract` (comparer/bookmarks.rs:92); +364 more

#### `Dom::set_name`  (L539-543, pub)
`pub fn set_name(&mut self, id: NodeId, new_name: XName)`

- **Does:** Set the element's expanded name.
- **Calls (crate):** `Dom::data_mut` (xmllinq/mod.rs:464) L540
- **Called by:** `resolve_for_edit_or_cli` (changes.rs:922) (?); `convert_run_text_to_del_text::{fn walk}` (comparer/finalize.rs:103); `convert_stamped_preins` (comparer/finalize.rs:10287); `flatten_tracked_deletions` (comparer/finalize.rs:11035); `convert_stamped_predeletes` (comparer/finalize.rs:11141); `fix_strict_validity_artifacts` (comparer/finalize.rs:11326); `fix_strict_validity_artifacts::{fn remap}` (comparer/finalize.rs:11395); `enforce_deleted_text_kinds::{fn walk}` (comparer/finalize.rs:14857); `delete_text_in_opaque` (comparer/produce.rs:41); `document_text_view` (debug.rs:2181); `mark_hf_part_content_as_deleted` (document_comparer.rs:5666); `rename_deleted_text` (edit/tracked.rs:1521); +5 more

#### `Dom::text_value`  (L547-552, pub)
`pub fn text_value(&self, id: NodeId) -> Option<&str>`

- **Does:** The payload of a text or comment node as a borrowed string, or `None` for any other node kind.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L548
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `public_memory_append_owner_boundary_tests::semantic` (append.rs:1454) (?); `empty_text` (changes/logical.rs:487); `extract` (comparer/bookmarks.rs:92); `extract_events` (comparer/comments.rs:260); `collect_segments` (comparer/comments.rs:477); `public_option_source_ownership_tests::canonical` (comparer/cross_para.rs:2479) (?); `paragraph_mark_properties_equal::{fn signature}` (comparer/finalize.rs:3342); `coverage_round_next_tests::same_tree` (comparer/finalize.rs:19391) (?); `coverage_real_revision_merge_matrix_tests::guarded_multi_original_complete_projection::{fn semantic}` (comparer/finalize.rs:25626) (?); `consecutive_block_splice_tests::canonical` (comparer/finalize.rs:28397) (?); `demo_title_pilcrow_history_tests::effect_projection::{fn canonical}` (comparer/finalize.rs:28628) (?); `paired_row_mark_ownership_tests::canonical` (comparer/finalize.rs:28869) (?); +50 more

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
- **Called by:** `emit` (xmllinq/serialize.rs:668); `serialize_document` (xmllinq/serialize.rs:805); `tests::document_serialization_preserves_declarations_comments_and_processing_instructions` (xmllinq/serialize.rs:885)

#### `Dom::pi_data`  (L572-577, pub)
`pub fn pi_data(&self, id: NodeId) -> Option<&str>`

- **Does:** The PI data (everything between the target and `?>`), or `None` for non-PI nodes.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L573
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `xml_holds` (scrub.rs:528); `emit` (xmllinq/serialize.rs:668); `serialize_document` (xmllinq/serialize.rs:805); `tests::document_serialization_preserves_declarations_comments_and_processing_instructions` (xmllinq/serialize.rs:885)

#### `Dom::declaration`  (L581-586, pub)
`pub fn declaration(&self, id: NodeId) -> Option<&XDeclaration>`

- **Does:** The XML declaration attached to a Document node, or `None` for any other node kind.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L582
- **Calls (std/external/unresolved):** `.as_deref`
- **Called by:** `xml_lexical_boundary_contract_tests::legacy_declaration_defaults_keep_readable_roots_without_inventing_metadata` (xmllinq/parse.rs:1351); `serialize_document` (xmllinq/serialize.rs:805)

#### `Dom::set_declaration`  (L589-593, pub)
`pub fn set_declaration(&mut self, id: NodeId, decl: Option<XDeclaration>)`

- **Does:** Attach or clear the XML declaration on a Document node.
- **Calls (crate):** `Dom::data_mut` (xmllinq/mod.rs:464) L590
- **Calls (std/external/unresolved):** `.map`
- **Called by:** `Parser::parse_document` (xmllinq/parse.rs:654); `tests::document_serialization_preserves_declarations_comments_and_processing_instructions` (xmllinq/serialize.rs:885)

#### `Dom::parent`  (L597-599, pub)
`pub fn parent(&self, id: NodeId) -> Option<NodeId>`

- **Does:** The parent of `id`, or `None` for the document root.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L598
- **Called by:** `drop_comments_where` (append.rs:432); `Package::empty_spacers` (audit.rs:661) (?); `absolute` (bin/jubarte.rs:168) (?); `run_convert` (bin/jubarte.rs:646) (?); `publish_edit` (bin/jubarte.rs:1402) (?); `run_debug_diff` (bin/jubarte.rs:3871) (?); `fold_twins` (changes.rs:1280); `fold_style_pairs` (changes.rs:1342); `target` (changes.rs:1480); `owner_text` (changes.rs:1528); `resolve_plan` (cli.rs:613) (?); `paragraph_of` (comparer/bookmarks.rs:76); +179 more

#### `Dom::nodes`  (L604-606, pub)
`pub fn nodes(&self, id: NodeId) -> Vec<NodeId>`

- **Does:** Direct children in document order, all node kinds (a clone of the internal content vec).
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L605
- **Calls (std/external/unresolved):** `.clone`
- **Called by:** `public_memory_append_owner_boundary_tests::semantic` (append.rs:1454) (?); `empty_text` (changes/logical.rs:487); `CommentFamily::set_text` (comments.rs:793) (?); `split_seg` (comparer/comments.rs:568); `split_run_before_leaf` (comparer/comments.rs:633); `public_option_source_ownership_tests::canonical` (comparer/cross_para.rs:2479); `descendants_trimmed::{fn walk}` (comparer/finalize.rs:65); `convert_run_text_to_del_text::{fn walk}` (comparer/finalize.rs:103); `rebuild_run` (comparer/finalize.rs:162); `mark_content_transform` (comparer/finalize.rs:192); `conjoin_transform` (comparer/finalize.rs:628); `resolve_alternate_content` (comparer/finalize.rs:1301); +116 more

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
- **Called by:** `audit_stories` (changes.rs:168); `recurse_children` (comparer/atomize.rs:393); `clone_block_level_content_for_hashing::{fn strip_nsdecls}` (comparer/preprocess.rs:134); `clone_children` (comparer/preprocess.rs:193); `clone_internal_unsalted` (comparer/preprocess.rs:245); `element_children` (comparer/preprocess.rs:506); `element_children_named` (comparer/preprocess.rs:519); `is_single_t_run` (comparer/preprocess.rs:544); `collect_simple_p_fragments` (comparer/preprocess.rs:775); `has_element_child` (comparer/preprocess.rs:835); `element_or_desc_has_predelete_orig` (comparer/preprocess.rs:1004); `stream_tbl_body` (comparer/preprocess.rs:1023); +54 more

#### `Dom::child_at`  (L632-634, pub)
`pub fn child_at(&self, id: NodeId, i: usize) -> NodeId`

- **Does:** The `i`-th direct child of `id` (all node kinds).
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L633
- **Called by:** `audit_stories` (changes.rs:168); `recurse_children` (comparer/atomize.rs:393); `clone_block_level_content_for_hashing::{fn strip_nsdecls}` (comparer/preprocess.rs:134); `clone_children` (comparer/preprocess.rs:193); `clone_internal_unsalted` (comparer/preprocess.rs:245); `element_children` (comparer/preprocess.rs:506); `element_children_named` (comparer/preprocess.rs:519); `is_single_t_run` (comparer/preprocess.rs:544); `collect_simple_p_fragments` (comparer/preprocess.rs:775); `has_element_child` (comparer/preprocess.rs:835); `element_or_desc_has_predelete_orig` (comparer/preprocess.rs:1004); `stream_tbl_body` (comparer/preprocess.rs:1023); +53 more

#### `Dom::elements`  (L640-651, pub)
`pub fn elements(&self, id: NodeId, filter: Option<&XName>) -> Vec<NodeId>`

- **Does:** Direct child elements, optionally filtered by expanded name.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L641,645
- **Calls (std/external/unresolved):** `.collect`, `.copied`, `.filter`, `.iter`
- **Called by:** `append_documents` (append.rs:152); `final_sect_pr` (append.rs:360); `drop_comments_where` (append.rs:432); `stage_comments` (append.rs:563); `carry_notes` (append.rs:673); `index_by_attribute` (append.rs:848); `ListCopy::build` (append.rs:1117); `insert_lists` (append.rs:1165); `add_in_numbering_order` (append.rs:1208); `join` (append.rs:1270); `set_section_type` (append.rs:1331); `tests::section_type_sits_after_header_references` (append.rs:1404); +610 more

#### `Dom::element`  (L655-661, pub)
`pub fn element(&self, id: NodeId, filter: &XName) -> Option<NodeId>`

- **Does:** The first direct child element whose name matches `filter`, or `None`.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L656; `Dom::data` (xmllinq/mod.rs:461) (by name) L660
- **Calls (std/external/unresolved):** `.copied`, `.find`, `.iter`
- **Macros:** `matches!`
- **Called by:** `body` (append.rs:355); `style_key` (append.rs:831); `abstract_of` (append.rs:1226); `unique_nsid` (append.rs:1247); `tests::nsid_collisions_take_the_next_free_value` (append.rs:1357); `tests::section_type_sits_after_header_references` (append.rs:1404) (?); `public_memory_append_owner_boundary_tests::view` (append.rs:1509); `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1599) (?); `Package::read` (audit.rs:354) (?); `Package::read_styles` (audit.rs:406); `Package::paragraph_style` (audit.rs:493); `Package::numbered` (audit.rs:513); +620 more

#### `Dom::descendants`  (L668-672, pub)
`pub fn descendants(&self, id: NodeId, filter: Option<&XName>) -> Vec<NodeId>`

- **Does:** Every descendant element of `id`, in pre-order.
- **Calls (crate):** `Dom::for_each_descendant_element` (xmllinq/mod.rs:677) L670
- **Calls (std/external/unresolved):** `.push`, `Vec::new`
- **Called by:** `comment_anchors` (append.rs:416); `offset_annotation_ids` (append.rs:501); `drop_colliding_paragraph_ids` (append.rs:529); `carry_notes` (append.rs:673); `ListCopy::build` (append.rs:1117); `nsids` (append.rs:1232); `Package::read_styles` (audit.rs:406); `Package::image_no_descr` (audit.rs:558) (?); `Package::table_no_header_row` (audit.rs:586) (?); `Package::direct_formatting` (audit.rs:714) (?); `Package::fields` (audit.rs:764) (?); `decorative` (audit.rs:942); +783 more

#### `Dom::for_each_descendant_element`  (L677-687, pub)
`pub fn for_each_descendant_element( &self, id: NodeId, filter: Option<&XName>, mut visit: impl FnMut(NodeId), )`

- **Does:** Visit every descendant element of `id` in document order without allocating a result `Vec` (DOM-ITER-03).
- **Calls (crate):** `Dom::find_descendant_element` (xmllinq/mod.rs:692) L683; `declare::{fn visit}` (markdown/from_docx/drawing.rs:498) (by name) L684
- **Called by:** `do_lcs_algorithm` (comparer/lcs.rs:2494); `test_for_invalid_content` (comparer/preprocess.rs:602); `add_sha1_hash_to_block_level_content` (comparer/preprocess.rs:1259); `hash_block_level_content` (comparer/preprocess.rs:1317); `Dom::descendants` (xmllinq/mod.rs:668); `Dom::for_each_descendant_and_self` (xmllinq/mod.rs:753)

#### `Dom::find_descendant_element`  (L692-725, pub)
`pub fn find_descendant_element( &self, id: NodeId, filter: Option<&XName>, mut pred: impl FnMut(NodeId) -> bool, ) -> Option<NodeId>`

- **Does:** First descendant element (in pre-order) that passes both the name `filter` and the `pred` predicate; the walk stops there instead of listing the whole subtree first.
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L701; `Dom::child_at` (xmllinq/mod.rs:632) L706; `Dom::is_element` (xmllinq/mod.rs:502) L708; `Dom::name_is` (xmllinq/mod.rs:535) L717
- **Calls (std/external/unresolved):** `.last_mut`, `.pop`, `.push`, `Some`, `pred`
- **Macros:** `vec!`
- **Called by:** `first_named` (convert/mod.rs:4143); `para_formatting_changed` (convert/mod.rs:11491); `first_named_any` (convert/mod.rs:19979); `wholly_deleted_endpoints` (revision_processor/bookmarks.rs:47); `Dom::for_each_descendant_element` (xmllinq/mod.rs:677)

#### `Dom::descendant_nodes`  (L730-734, pub)
`pub fn descendant_nodes(&self, id: NodeId) -> Vec<NodeId>`

- **Does:** Every descendant node (not just elements), in pre-order: self is NOT included.
- **Calls (crate):** `Dom::walk_descendant_nodes` (xmllinq/mod.rs:735) L732
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `extract` (comparer/bookmarks.rs:92); `extract_events` (comparer/comments.rs:260); `collect_segments` (comparer/comments.rs:477); `strip_whitespace_in_clone_text` (comparer/preprocess.rs:96); `xml_holds` (scrub.rs:528); `tests::document_serialization_preserves_declarations_comments_and_processing_instructions` (xmllinq/serialize.rs:885)

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
- **Called by:** `w_vals` (append.rs:818); `rewrite_para_id_references` (comparer/comments.rs:988); `rewrite_durable_id_references` (comparer/comments.rs:1003); `preserve_cloned_namespace_context` (comparer/comments.rs:1052); `drop_pt_bookkeeping` (comparer/finalize.rs:481); `declare_extension_namespaces_ignorable` (comparer/finalize.rs:942); `bind_compatibility_prefixes` (comparer/finalize.rs:1026); `remove_powertools_scratch_markup` (comparer/finalize.rs:1060); `resolve_alternate_content` (comparer/finalize.rs:1301); `content_revision_marks` (comparer/finalize.rs:6189); `wml_order_elements_per_standard` (comparer/finalize.rs:10488); `fix_strict_validity_artifacts` (comparer/finalize.rs:11326); +28 more

#### `Dom::for_each_descendant_and_self`  (L753-765, pub)
`pub fn for_each_descendant_and_self( &self, id: NodeId, filter: Option<&XName>, mut visit: impl FnMut(NodeId), )`

- **Does:** Non-allocating `DescendantsAndSelf` walk (DOM-ITER-03).
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L759; `declare::{fn visit}` (markdown/from_docx/drawing.rs:498) (by name) L762; `Dom::for_each_descendant_element` (xmllinq/mod.rs:677) L764
- **Calls (std/external/unresolved):** `.is_none_or`
- **Called by:** `remove_existing_powertools_markup` (comparer/preprocess.rs:572); `Dom::descendants_and_self` (xmllinq/mod.rs:746)

#### `Dom::ancestors`  (L770-782, pub)
`pub fn ancestors(&self, id: NodeId, filter: Option<&XName>) -> Vec<NodeId>`

- **Does:** Ancestor elements from the nearest parent up to (but not including) the document, optionally filtered by name.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L772,774,779
- **Calls (std/external/unresolved):** `.is_none_or`, `.push`, `Vec::new`
- **Called by:** `Story::location` (audit.rs:317); `Package::empty_spacers` (audit.rs:661) (?); `Package::direct_formatting` (audit.rs:714) (?); `under_fallback` (audit.rs:954); `audit_owner` (changes.rs:283); `held_comment_marks` (changes.rs:1249); `fold_twins` (changes.rs:1280); `paragraph_of` (changes.rs:1388); `in_recorded_properties` (changes.rs:1406); `paragraph_move` (changes.rs:1443); `holder` (changes.rs:1455); `deleted_text` (changes.rs:1504); +108 more

#### `Dom::ancestors_and_self`  (L787-805, pub)
`pub fn ancestors_and_self(&self, id: NodeId, filter: Option<&XName>) -> Vec<NodeId>`

- **Does:** `Self` then its ancestor elements, optionally filtered by name.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L791,796; `Dom::is_element` (xmllinq/mod.rs:502) L797
- **Calls (std/external/unresolved):** `.is_none_or`, `.push`, `Some`, `Vec::new`
- **Called by:** `resolve_alternate_content` (comparer/finalize.rs:1301); `prefix_in_scope` (comparer/finalize.rs:1490); `wrap_revised_hyperlink_as_field` (comparer/finalize.rs:1641); `para_is_pure_deleted_uncached` (comparer/finalize.rs:2915); `align_word_table_and_comment_chrome` (comparer/finalize.rs:6220); `hyperlink_is_wholly_inserted` (comparer/finalize.rs:6398); `fold_midstream_demo_title_into_numbered_heading_with_source_properties` (comparer/finalize.rs:12624); `text_side` (comparer/finalize.rs:12910); `deep_production_path_tests::body_revisions` (comparer/finalize.rs:20954); `paragraph_id` (fields.rs:932) (?); `add_content_controls_with_owners` (revision_processor.rs:2393)

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
- **Called by:** `parse` (append.rs:336); `drop_comments_in_new_parts` (append.rs:456); `insert_lists` (append.rs:1165); `nsids` (append.rs:1232); `tests::nsid_collisions_take_the_next_free_value` (append.rs:1357); `tests::section_type_sits_after_header_references` (append.rs:1404); `tests::ignorable_prefixes_merge_only_with_the_same_namespace` (append.rs:1423); `public_memory_append_owner_boundary_tests::view` (append.rs:1509); `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1599); `audit_stories` (changes.rs:168); `for_each_story` (changes.rs:657); `comment_changes_in` (changes.rs:1003); +492 more

#### `Dom::index_in_parent`  (L830-834)
`fn index_in_parent(&self, id: NodeId) -> Option<(NodeId, usize)>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L831,832
- **Calls (std/external/unresolved):** `.iter`, `.position`, `Some`
- **Called by:** `Dom::nodes_after_self` (xmllinq/mod.rs:838); `Dom::nodes_before_self` (xmllinq/mod.rs:846); `Dom::detach` (xmllinq/mod.rs:1018); `Dom::add_before_self` (xmllinq/mod.rs:1130); `Dom::add_after_self` (xmllinq/mod.rs:1147); `Dom::replace_with` (xmllinq/mod.rs:1165)

#### `Dom::nodes_after_self`  (L838-843, pub)
`pub fn nodes_after_self(&self, id: NodeId) -> Vec<NodeId>`

- **Does:** Following siblings of `id` (all node kinds), in document order.
- **Calls (crate):** `Dom::index_in_parent` (xmllinq/mod.rs:830) L839; `Dom::data` (xmllinq/mod.rs:461) L840
- **Calls (std/external/unresolved):** `.to_vec`, `Vec::new`
- **Called by:** `lift_out_of_deletion` (edit.rs:6776); `split_deletion_before` (edit.rs:6821); `write_result` (fields.rs:824); `write_toc` (fields.rs:1154); `elements_after_self` (revision_processor.rs:889); `accept_deleted_cells_transform` (revision_processor.rs:2838); `Dom::next_element` (xmllinq/mod.rs:855)

#### `Dom::nodes_before_self`  (L846-851, pub)
`pub fn nodes_before_self(&self, id: NodeId) -> Vec<NodeId>`

- **Does:** Preceding siblings of `id` (all node kinds), in document order.
- **Calls (crate):** `Dom::index_in_parent` (xmllinq/mod.rs:830) L847; `Dom::data` (xmllinq/mod.rs:461) L848
- **Calls (std/external/unresolved):** `.to_vec`, `Vec::new`
- **Called by:** `audit_owner` (changes.rs:283); `delete_stories_of_deleted_text_boxes` (comparer/finalize.rs:10817); `separate` (edit/structural.rs:242); `previous_paragraph` (edit/tracked.rs:1659); `Transaction::body_level_marker_beside` (edit.rs:2897) (?); `write_result` (fields.rs:824); `write_toc` (fields.rs:1154); `get_paragraph_info` (revision_processor.rs:966); `content_elements_before_self` (revision_processor.rs:1002)

#### `Dom::next_element`  (L855-859, pub)
`pub fn next_element(&self, id: NodeId) -> Option<NodeId>`

- **Does:** First following sibling element, or `None` if there is no element sibling after `id`.
- **Calls (crate):** `Dom::nodes_after_self` (xmllinq/mod.rs:838) L856; `Dom::is_element` (xmllinq/mod.rs:502) L858
- **Calls (std/external/unresolved):** `.find`, `.into_iter`
- **Called by:** `fix_up_footnotes_endnotes_with_custom_markers` (comparer/footnotes.rs:887); `coverage_custom_note_owner_boundaries::custom_marker_fixup_moves_only_matching_leaf_and_keeps_all_other_owners` (comparer/footnotes.rs:1708); `separate` (edit/structural.rs:242); `check` (edit/tracked.rs:107) (?); `emit` (edit/tracked.rs:452) (?); `emit_text` (edit/tracked.rs:1035); `Transaction::resolve_visible` (edit/visible.rs:348) (?); `Transaction::body_level_marker_beside` (edit.rs:2897) (?); `Transaction::resolve_comment_text` (edit.rs:3057) (?); `Transaction::merge_partner` (edit.rs:4859) (?); `lift_out_of_deletion` (edit.rs:6776); `merge_into` (edit.rs:8156); +2 more

#### `Dom::has_elements`  (L862-864, pub)
`pub fn has_elements(&self, id: NodeId) -> bool`

- **Does:** True if `id` has at least one direct child element.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L863; `Dom::is_element` (xmllinq/mod.rs:502) L863
- **Calls (std/external/unresolved):** `.any`, `.iter`
- **Called by:** `paragraph_mark_properties_equal` (comparer/finalize.rs:3322); `fill_in_empty_footnotes_endnotes` (comparer/footnotes.rs:133); `deterministic_note_boundary_tests::properties` (comparer/footnotes.rs:1291) (?); `build_paragraph` (comparer/produce.rs:104); `get_property_value` (comparer/revisions.rs:291); `check_text_content` (edit/v2.rs:847); `mark_run` (edit.rs:7632); `accept_revisions_for_element` (revision_processor.rs:452); `accept_content` (revision_processor.rs:3481)

#### `Dom::has_attributes`  (L867-869, pub)
`pub fn has_attributes(&self, id: NodeId) -> bool`

- **Does:** True if `id` carries at least one attribute.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L868
- **Calls (std/external/unresolved):** `.is_empty`
- **Called by:** `coverage_real_revision_merge_matrix_tests::projected` (comparer/finalize.rs:24813); `coverage_real_revision_merge_matrix_tests::guarded_multi_original_complete_projection::{fn semantic}` (comparer/finalize.rs:25626); `demo_title_pilcrow_history_tests::projection` (comparer/finalize.rs:28528); `get_property_value` (comparer/revisions.rs:291); `prune_normal_rpr_context_equal_attrs` (document_comparer.rs:3822); `resolve_redefined_style_metrics` (document_comparer.rs:4063); `coverage_package_source_matrix_tests::view` (document_comparer.rs:11533); `drop_slot` (revision_processor/style_records.rs:758); `complete_field_instruction_metadata_boundary_tests::repaired_complete_field_codes_preserve_source_comments_processing_instructions_and_formats` (revision_processor.rs:5384) (?)

#### `Dom::attribute`  (L874-880, pub)
`pub fn attribute(&self, id: NodeId, name: &XName) -> Option<&str>`

- **Does:** The value of attribute `name` on `id`, or `None` if `id` is not an element or has no such attribute.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L875
- **Calls (std/external/unresolved):** `.as_str`, `.find`, `.iter`, `.map`
- **Called by:** `append_documents` (append.rs:152); `merge_namespace_declarations` (append.rs:371); `drop_comments_where` (append.rs:432); `offset_annotation_ids` (append.rs:501); `drop_colliding_paragraph_ids` (append.rs:529); `stage_comments` (append.rs:563); `carry_notes` (append.rs:673); `w_vals` (append.rs:818); `style_key` (append.rs:831); `index_by_attribute` (append.rs:848); `ListCopy::build` (append.rs:1117); `insert_lists` (append.rs:1165); +584 more

#### `Dom::attributes_iter`  (L886-894, pub)
`pub fn attributes_iter( &self, id: NodeId, ) -> impl ExactSizeIterator<Item = (&XName, &str)> + '_`

- **Does:** All attributes on `id`, in declaration order, as borrowed `(name, value)` pairs.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L890
- **Calls (std/external/unresolved):** `.as_str`, `.iter`, `.map`
- **Called by:** `public_memory_append_owner_boundary_tests::semantic` (append.rs:1454); `preserve_cloned_namespace_context` (comparer/comments.rs:1052); `union_comments_xml` (comparer/comments.rs:1190); `drop_orphans` (comparer/comments.rs:1416); `public_option_source_ownership_tests::canonical` (comparer/cross_para.rs:2479); `public_option_source_ownership_tests::source_events::{fn walk}` (comparer/cross_para.rs:2512) (?); `declare_extension_namespaces_ignorable` (comparer/finalize.rs:942); `bind_compatibility_prefixes` (comparer/finalize.rs:1026); `paragraph_mark_properties_equal::{fn signature}` (comparer/finalize.rs:3342); `pg_num_type_is_implicit_decimal` (comparer/finalize.rs:6053); `coverage_real_revision_merge_matrix_tests::guarded_multi_original_complete_projection::{fn semantic}` (comparer/finalize.rs:25626); `coverage_real_revision_merge_matrix_tests::public_compare_multi_original_blank_tail_recovers_complete_clean_sources::{fn complete_source}` (comparer/finalize.rs:26148) (?); +44 more

#### `Dom::attributes`  (L899-905, pub)
`pub fn attributes(&self, id: NodeId) -> Vec<(XName, String)>`

- **Does:** All attributes on `id`, in declaration order, as `(name, value)` owned pairs.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L900
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.iter`, `.map`
- **Called by:** `elements` (admission.rs:967) (?); `merge_namespace_declarations` (append.rs:371); `merge_part_namespaces` (append.rs:1087); `coalesce_recurse` (comparer/atomize.rs:575); `extract` (comparer/bookmarks.rs:92); `rewrite_para_id_references` (comparer/comments.rs:988); `rewrite_durable_id_references` (comparer/comments.rs:1003); `namespace_declarations` (comparer/comments.rs:1018); `preserve_cloned_namespace_context` (comparer/comments.rs:1052); `union_comments_xml` (comparer/comments.rs:1190); `drop_orphans` (comparer/comments.rs:1416); `public_option_source_ownership_tests::canonical` (comparer/cross_para.rs:2479); +100 more

#### `Dom::attr_count`  (L909-911, pub)
`pub fn attr_count(&self, id: NodeId) -> usize`

- **Does:** Number of attributes on `id`.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L910
- **Calls (std/external/unresolved):** `.len`
- **Called by:** `drop_pt_bookkeeping` (comparer/finalize.rs:481); `clone_block_level_content_for_hashing::{fn strip_nsdecls}` (comparer/preprocess.rs:134); `new_with_filtered_attrs` (comparer/preprocess.rs:162); `clone_internal_unsalted` (comparer/preprocess.rs:245); `remove_existing_powertools_markup` (comparer/preprocess.rs:572); `filtered_p_attr_xml` (comparer/preprocess.rs:742); `filtered_leaf_attr_xml` (comparer/preprocess.rs:846); `clone_for_structure_hash` (comparer/preprocess.rs:1221); `xml_holds` (scrub.rs:528); `attribute_by_local` (validate.rs:2490); `namespace_in_scope` (validate.rs:2595); `tag_policy_tests::the_dom_parser_keeps_every_attribute_and_its_order` (xmllinq/parse.rs:1575); +8 more

#### `Dom::attr_at`  (L920-923, pub)
`pub fn attr_at(&self, id: NodeId, i: usize) -> (&XName, &str)`

- **Does:** The `i`-th attribute on `id`, as borrowed `(name, value)` (no allocations; DOM-ITER-01 backs the serializer hot path).
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L921
- **Calls (std/external/unresolved):** `.as_str`
- **Called by:** `drop_pt_bookkeeping` (comparer/finalize.rs:481); `clone_block_level_content_for_hashing::{fn strip_nsdecls}` (comparer/preprocess.rs:134); `new_with_filtered_attrs` (comparer/preprocess.rs:162); `clone_internal_unsalted` (comparer/preprocess.rs:245); `remove_existing_powertools_markup` (comparer/preprocess.rs:572); `filtered_p_attr_xml` (comparer/preprocess.rs:742); `filtered_leaf_attr_xml` (comparer/preprocess.rs:846); `clone_for_structure_hash` (comparer/preprocess.rs:1221); `xml_holds` (scrub.rs:528); `attribute_by_local` (validate.rs:2490); `namespace_in_scope` (validate.rs:2595); `tag_policy_tests::the_dom_parser_keeps_every_attribute_and_its_order` (xmllinq/parse.rs:1575); +7 more

#### `Dom::set_attributes`  (L935-971, pub(crate))
`pub(crate) fn set_attributes(&mut self, id: NodeId, attrs: Vec<(XName, String)>)`

- **Does:** Insert a batch of attributes in declaration order.
- **Calls (crate):** `Dom::data_mut` (xmllinq/mod.rs:464) L936; `bump_attr_name_probes` (xmllinq/mod.rs:283) L962
- **Calls (std/external/unresolved):** `.clone`, `.entry`, `.enumerate`, `.find`, `.get`, `.insert`, `.is_empty`, `.iter_mut`, `.iter`, `.len`, `.or_insert`, `.push`, `HashMap::with_capacity`
- **Called by:** `new_markers` (comparer/bookmarks.rs:264); `Parser::parse_element` (xmllinq/parse.rs:709)

#### `Dom::set_attribute_value`  (L977-1006, pub)
`pub fn set_attribute_value(&mut self, id: NodeId, name: &XName, value: Option<&str>)`

- **Does:** Add or update an attribute.
- **Calls (crate):** `Dom::data_mut` (xmllinq/mod.rs:464) L978; `bump_attr_name_probes` (xmllinq/mod.rs:283) L986
- **Calls (std/external/unresolved):** `.clone`, `.enumerate`, `.iter`, `.position`, `.push`, `.retain`, `.to_string`, `Some`
- **Called by:** `merge_namespace_declarations` (append.rs:371); `offset_annotation_ids` (append.rs:501); `drop_colliding_paragraph_ids` (append.rs:529); `stage_comments` (append.rs:563); `carry_notes` (append.rs:673); `carry_styles_and_numbering` (append.rs:867); `ListCopy::build` (append.rs:1117); `unique_nsid` (append.rs:1247); `join` (append.rs:1270); `set_section_type` (append.rs:1331); `CommentFamily::add` (comments.rs:648) (?); `CommentFamily::set_date_utc` (comments.rs:726); +272 more

#### `Dom::is_namespace_declaration`  (L1012-1015, pub)
`pub fn is_namespace_declaration(&self, name: &XName) -> bool`

- **Does:** True if `name` looks like an `xmlns` / `xmlns:*` namespace declaration: namespace URI `http://www.w3.org/2000/xmlns/`, or no namespace with local name `xmlns`.
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L1013,1014; `XName::local_name` (xmllinq/mod.rs:208) L1014
- **Calls (std/external/unresolved):** `.is_empty`
- **Called by:** `merge_namespace_declarations` (append.rs:371); `public_memory_append_owner_boundary_tests::semantic` (append.rs:1454); `namespace_declarations` (comparer/comments.rs:1018); `preserve_cloned_namespace_context` (comparer/comments.rs:1052); `public_option_source_ownership_tests::canonical` (comparer/cross_para.rs:2479); `public_option_source_ownership_tests::source_events::{fn walk}` (comparer/cross_para.rs:2512); `drop_pt_bookkeeping` (comparer/finalize.rs:481); `declare_extension_namespaces_ignorable` (comparer/finalize.rs:942); `prefix_in_scope` (comparer/finalize.rs:1490); `paragraph_mark_properties_equal::{fn signature}` (comparer/finalize.rs:3342); `paired_replacement_has_identical_properties` (comparer/finalize.rs:8529); `repair_inherited_invalidity` (comparer/finalize.rs:14789); +50 more

#### `Dom::detach`  (L1018-1023)
`fn detach(&mut self, id: NodeId)`

- **Does:** ── mutation ────────────────────────────────────────────────────────────── _(from a code comment)_
- **Calls (crate):** `Dom::index_in_parent` (xmllinq/mod.rs:830) L1019; `Dom::data_mut` (xmllinq/mod.rs:464) L1020,1021
- **Calls (std/external/unresolved):** `.remove`
- **Called by:** `Dom::remove` (xmllinq/mod.rs:1118)

#### `Dom::materialize`  (L1029-1035)
`fn materialize(&mut self, node: NodeId) -> NodeId`

- **Does:** If `node` already has a parent, deep-clone it (LINQ-to-XML semantics); otherwise return it as-is.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L1030; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1031
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `Dom::add` (xmllinq/mod.rs:1074); `Dom::add_first` (xmllinq/mod.rs:1101); `Dom::add_before_self` (xmllinq/mod.rs:1130); `Dom::add_after_self` (xmllinq/mod.rs:1147); `Dom::replace_with` (xmllinq/mod.rs:1165)

#### `Dom::validate_attachment`  (L1041-1051)
`fn validate_attachment(&self, parent: NodeId, node: NodeId)`

- **Does:** Centralized validation before a node is attached to a parent: `parent` must be a container (`Element` or `Document`), `node` must not be the same as `parent`, and an unparented `node` must not be an ancestor of `parent` (cycle under descendant).
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L1042; `Dom::is_document` (xmllinq/mod.rs:511) L1042; `Dom::data` (xmllinq/mod.rs:461) L1048; `Dom::is_ancestor_of` (xmllinq/mod.rs:1054) L1048
- **Calls (std/external/unresolved):** `.is_none`
- **Macros:** `panic!`
- **Called by:** `Dom::add` (xmllinq/mod.rs:1074); `Dom::add_text` (xmllinq/mod.rs:1087); `Dom::add_first` (xmllinq/mod.rs:1101); `Dom::add_before_self` (xmllinq/mod.rs:1130); `Dom::add_after_self` (xmllinq/mod.rs:1147); `Dom::replace_with` (xmllinq/mod.rs:1165); `Dom::clone_subtree` (xmllinq/mod.rs:1231)

#### `Dom::is_ancestor_of`  (L1054-1063)
`fn is_ancestor_of(&self, ancestor: NodeId, descendant: NodeId) -> bool`

- **Does:** True when `ancestor` is strictly above `descendant` on the parent chain.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L1055,1060
- **Called by:** `Dom::validate_attachment` (xmllinq/mod.rs:1041)

#### `Dom::add`  (L1074-1079, pub)
`pub fn add(&mut self, parent: NodeId, node: NodeId)`

- **Does:** Append `node` as the last child of `parent`.
- **Calls (crate):** `Dom::materialize` (xmllinq/mod.rs:1029) L1075; `Dom::validate_attachment` (xmllinq/mod.rs:1041) L1076; `Dom::data_mut` (xmllinq/mod.rs:464) L1077,1078
- **Calls (std/external/unresolved):** `.push`, `Some`
- **Called by:** `append_documents` (append.rs:152); `stage_comments` (append.rs:563); `carry_notes` (append.rs:673); `carry_styles_and_numbering` (append.rs:867); `add_in_numbering_order` (append.rs:1208); `join` (append.rs:1270); `set_section_type` (append.rs:1331); `Anchors::scan` (comments.rs:251); `CommentFamily::add` (comments.rs:648); `CommentFamily::store` (comments.rs:854); `CommentFamily::store_people` (comments.rs:1033); `fill_comment_paragraph` (comments.rs:1129); +239 more

#### `Dom::add_text`  (L1087-1093, pub)
`pub fn add_text(&mut self, parent: NodeId, value: &str) -> NodeId`

- **Does:** Allocate a new text child of `parent` carrying `value`.
- **Calls (crate):** `Dom::new_text` (xmllinq/mod.rs:483) L1088; `Dom::validate_attachment` (xmllinq/mod.rs:1041) L1089; `Dom::data_mut` (xmllinq/mod.rs:464) L1090,1091
- **Calls (std/external/unresolved):** `.push`, `Some`
- **Called by:** `Anchors::scan` (comments.rs:251); `fill_comment_paragraph` (comments.rs:1129); `recurse` (comparer/atomize.rs:410); `coalesce_recurse` (comparer/atomize.rs:575); `split_seg` (comparer/comments.rs:568); `deeper_boundary_tests::projection_rejects_document_text_and_a_detached_leaf_without_a_run` (comparer/comments.rs:2540); `coalesce_adjacent_runs` (comparer/finalize.rs:1222); `wrap_revised_hyperlink_as_field` (comparer/finalize.rs:1641); `rebuild_title_free_mesh` (comparer/finalize.rs:2813); `add_revision_text_run` (comparer/finalize.rs:2863); `rewrite_inserted_external_hyperlinks` (comparer/finalize.rs:6336); `mesh_short_label_shared_eq_with_settings` (comparer/finalize.rs:7204); +30 more

#### `Dom::add_first`  (L1101-1106, pub)
`pub fn add_first(&mut self, parent: NodeId, node: NodeId)`

- **Does:** Prepend `node` as the first child of `parent`.
- **Calls (crate):** `Dom::materialize` (xmllinq/mod.rs:1029) L1102; `Dom::validate_attachment` (xmllinq/mod.rs:1041) L1103; `Dom::data_mut` (xmllinq/mod.rs:464) L1104,1105
- **Calls (std/external/unresolved):** `.insert`, `Some`
- **Called by:** `move_last_sectpr_into_last_paragraph` (comparer/atomize.rs:63); `add_at_paragraph_start` (comparer/comments.rs:734); `mark_content_transform` (comparer/finalize.rs:192); `fix_paragraph_mark_revision_order` (comparer/finalize.rs:1526); `move_paragraph_properties_first` (comparer/finalize.rs:1566); `unwrap_hyperlinks_to_styled_runs` (comparer/finalize.rs:1597); `wrap_revised_hyperlink_as_field` (comparer/finalize.rs:1641); `promote_live_numpr_on_pure_d_from_pprchange` (comparer/finalize.rs:1908); `closing_mark_takes_revised_properties` (comparer/finalize.rs:3200); `make_empty_pure_ins_para` (comparer/finalize.rs:3824); `ensure_empty_pure_i_before_short_title_del` (comparer/finalize.rs:3951); `rotate_ins_mark_del_only_paragraph` (comparer/finalize.rs:4617); +55 more

#### `Dom::remove_nodes`  (L1110-1115, pub)
`pub fn remove_nodes(&mut self, id: NodeId)`

- **Does:** Detach every child of `id` (front matching LINQ-to-XML `RemoveNodes()`); their parent links are cleared.
- **Calls (crate):** `Dom::data_mut` (xmllinq/mod.rs:464) L1111,1113
- **Calls (std/external/unresolved):** `std::mem::take`
- **Called by:** `split_seg` (comparer/comments.rs:568); `coalesce_all_paragraphs` (comparer/finalize.rs:1503); `write_content` (edit/controls.rs:589); `write_result` (fields.rs:824); `reject_text` (inspect.rs:485); `Dom::set_value` (xmllinq/mod.rs:1219)

#### `Dom::remove`  (L1118-1120, pub)
`pub fn remove(&mut self, id: NodeId)`

- **Does:** Detach `id` from its parent.
- **Calls (crate):** `Dom::detach` (xmllinq/mod.rs:1018) L1119
- **Called by:** `drop_comments_where` (append.rs:432); `carry_notes` (append.rs:673); `ListCopy::build` (append.rs:1117); `insert_lists` (append.rs:1165); `join` (append.rs:1270); `set_section_type` (append.rs:1331); `CommentFamily::remove` (comments.rs:839); `CommentFamily::store_people` (comments.rs:1033); `clear_rows` (comments.rs:1148); `move_last_sectpr_into_last_paragraph` (comparer/atomize.rs:63); `leave_foreign_controls` (comparer/bookmarks.rs:235); `leave_inline_revision` (comparer/bookmarks.rs:290); +202 more

#### `Dom::add_before_self`  (L1130-1138, pub)
`pub fn add_before_self(&mut self, reference: NodeId, node: NodeId)`

- **Does:** Insert `node` as the preceding sibling of `reference`, which must have a parent.
- **Calls (crate):** `Dom::index_in_parent` (xmllinq/mod.rs:830) L1131; `Dom::materialize` (xmllinq/mod.rs:1029) L1134; `Dom::validate_attachment` (xmllinq/mod.rs:1041) L1135; `Dom::data_mut` (xmllinq/mod.rs:464) L1136,1137
- **Calls (std/external/unresolved):** `.expect`, `.insert`, `Some`
- **Called by:** `add_in_numbering_order` (append.rs:1208); `join` (append.rs:1270); `set_section_type` (append.rs:1331); `Anchors::scan` (comments.rs:251); `leave_foreign_controls` (comparer/bookmarks.rs:235); `leave_inline_revision` (comparer/bookmarks.rs:290); `inject_side` (comparer/bookmarks.rs:309); `place_before_offset` (comparer/comments.rs:665); `place_after_offset` (comparer/comments.rs:691); `inject_side` (comparer/comments.rs:787); `unwrap_content_controls` (comparer/finalize.rs:1400); `fix_paragraph_mark_revision_order` (comparer/finalize.rs:1526); +93 more

#### `Dom::add_after_self`  (L1147-1155, pub)
`pub fn add_after_self(&mut self, reference: NodeId, node: NodeId)`

- **Does:** Insert `node` as the following sibling of `reference`.
- **Calls (crate):** `Dom::index_in_parent` (xmllinq/mod.rs:830) L1148; `Dom::materialize` (xmllinq/mod.rs:1029) L1151; `Dom::validate_attachment` (xmllinq/mod.rs:1041) L1152; `Dom::data_mut` (xmllinq/mod.rs:464) L1153,1154
- **Calls (std/external/unresolved):** `.expect`, `.insert`, `Some`
- **Called by:** `CommentFamily::add` (comments.rs:648); `leave_foreign_controls` (comparer/bookmarks.rs:235); `leave_inline_revision` (comparer/bookmarks.rs:290); `inject_side` (comparer/bookmarks.rs:309); `split_seg` (comparer/comments.rs:568); `split_run_before_leaf` (comparer/comments.rs:633); `place_before_offset` (comparer/comments.rs:665); `place_after_offset` (comparer/comments.rs:691); `add_at_paragraph_start` (comparer/comments.rs:734); `inject_side` (comparer/comments.rs:787); `step_out_of_deletion_end` (comparer/comments.rs:898); `wrap_revised_hyperlink_as_field` (comparer/finalize.rs:1641); +45 more

#### `Dom::replace_with`  (L1165-1178, pub)
`pub fn replace_with(&mut self, reference: NodeId, nodes: &[NodeId])`

- **Does:** Replace `reference` with the sequence of `nodes` (in order) at the same position.
- **Calls (crate):** `Dom::index_in_parent` (xmllinq/mod.rs:830) L1166; `Dom::materialize` (xmllinq/mod.rs:1029) L1171; `Dom::validate_attachment` (xmllinq/mod.rs:1041) L1172; `Dom::data_mut` (xmllinq/mod.rs:464) L1173,1176,1177
- **Calls (std/external/unresolved):** `.expect`, `.len`, `.push`, `.splice`, `Some`, `Vec::with_capacity`
- **Called by:** `join` (append.rs:1270); `resolve_alternate_content` (comparer/finalize.rs:1301); `closing_mark_takes_revised_mark_formatting` (comparer/finalize.rs:3399); `merge_replaced_in_container` (comparer/finalize.rs:8626); `flatten_tracked_insertions_stamped` (comparer/finalize.rs:10220); `flatten_tracked_deletions` (comparer/finalize.rs:11035); `restore_styles` (document_comparer.rs:1814); `merge_normal_style_rpr` (document_comparer.rs:4343); `reject_text` (inspect.rs:485); `resync_linked_character_styles` (revision_processor/style_records.rs:543); `tidy` (revision_processor/word_save.rs:25) (?); `merge_adjacent_tables` (revision_processor.rs:3198); +1 more

#### `Dom::value`  (L1185-1190, pub)
`pub fn value(&self, id: NodeId) -> String`

- **Does:** Concatenated descendant text of `id` (LINQ-to-XML `element.Value`).
- **Calls (crate):** `Dom::value_str` (xmllinq/mod.rs:1195) L1186
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `audit_stories` (changes.rs:168); `text_of` (changes.rs:1518); `comment_text` (comments.rs:1101); `atom_hash` (comparer/atomize.rs:163); `salt_field_results` (comparer/atomize.rs:217); `recurse` (comparer/atomize.rs:410); `coverage_atomizer_field_chain_boundaries::header_and_footer_roots_stop_paths_and_preserve_each_whitespace_boundary` (comparer/atomize.rs:1123); `extract` (comparer/bookmarks.rs:92); `leave_permission_revision_boundary` (comparer/bookmarks.rs:551); `carry_permission_union` (comparer/bookmarks.rs:594); `comment_definition_fingerprint` (comparer/comments.rs:79); `split_seg` (comparer/comments.rs:568); +77 more

#### `Dom::value_str`  (L1195-1205, pub)
`pub fn value_str(&self, id: NodeId) -> std::borrow::Cow<'_, str>`

- **Does:** Borrowed text when `id` has exactly one direct text child; otherwise owned concatenated descendant text.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L1196,1198; `Dom::collect_text` (xmllinq/mod.rs:1207) L1203
- **Calls (std/external/unresolved):** `.as_str`, `.len`, `String::new`, `std::borrow::Cow::Borrowed`, `std::borrow::Cow::Owned`
- **Called by:** `Package::fields` (audit.rs:764) (?); `coalesce_recurse` (comparer/atomize.rs:575); `run_text_concat` (comparer/finalize.rs:1195); `wrap_revised_hyperlink_as_field` (comparer/finalize.rs:1641); `para_side_word_count` (comparer/finalize.rs:2055); `para_keeps_unchanged_words` (comparer/finalize.rs:2076); `free_mesh_shared_title_token_in_mix` (comparer/finalize.rs:2573); `split_head_short_title_long_del_mix` (comparer/finalize.rs:4870); `fold_leading_ins_from_mix_into_preceding_pure_del` (comparer/finalize.rs:4991); `hyperlink_is_wholly_inserted` (comparer/finalize.rs:6398); `mesh_short_label_shared_eq_with_settings` (comparer/finalize.rs:7204); `mesh_short_label_shared_eq_with_settings::{fn label_run}` (comparer/finalize.rs:7322); +69 more

#### `Dom::collect_text`  (L1207-1215)
`fn collect_text(&self, id: NodeId, s: &mut String)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L1208,1209; `Dom::collect_text` (xmllinq/mod.rs:1207) L1211
- **Calls (std/external/unresolved):** `.push_str`
- **Called by:** `Element::collect_text` (markdown/from_docx/ooxml.rs:421) (?); `Dom::value_str` (xmllinq/mod.rs:1195); `Dom::collect_text` (xmllinq/mod.rs:1207)

#### `Dom::set_value`  (L1219-1222, pub)
`pub fn set_value(&mut self, id: NodeId, value: &str)`

- **Does:** Replace `id`'s children with a single text child carrying `value` (LINQ-to-XML `element.Value = value`).
- **Calls (crate):** `Dom::remove_nodes` (xmllinq/mod.rs:1110) L1220; `Dom::add_text` (xmllinq/mod.rs:1087) L1221
- **Called by:** `mesh_short_label_shared_eq_with_settings` (comparer/finalize.rs:7204); `fix_strict_validity_artifacts` (comparer/finalize.rs:11326); `append_period` (comparer/finalize.rs:12958); `split_insertion_at` (comparer/finalize.rs:15050); `format_change_cache_tests::run_atom` (comparer/formatchg.rs:693); `memoized_moves_tests::make_pool` (comparer/moves.rs:858); `set_text` (edit.rs:7623)

#### `Dom::clone_subtree`  (L1231-1258, pub)
`pub fn clone_subtree(&mut self, id: NodeId) -> NodeId`

- **Does:** Deep-clone the subtree rooted at `id` into the same arena and return the new root.
- **Calls (crate):** `Dom::data` (xmllinq/mod.rs:461) L1232,1242; `Dom::child_count` (xmllinq/mod.rs:622) L1241; `Dom::alloc` (xmllinq/mod.rs:394) L1243; `Dom::data_mut` (xmllinq/mod.rs:464) L1245,1254,1255; `Dom::child_at` (xmllinq/mod.rs:632) L1251; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1252; `Dom::validate_attachment` (xmllinq/mod.rs:1041) L1253
- **Calls (std/external/unresolved):** `.clone`, `.push`, `.reserve_exact`, `NodeKind::Comment`, `NodeKind::Pi`, `NodeKind::Text`, `Some`
- **Called by:** `append_documents` (append.rs:152); `stage_comments` (append.rs:563); `carry_notes` (append.rs:673); `carry_styles_and_numbering` (append.rs:867); `ListCopy::build` (append.rs:1117); `join` (append.rs:1270); `move_last_sectpr_into_last_paragraph` (comparer/atomize.rs:63); `coalesce_recurse` (comparer/atomize.rs:575); `split_seg` (comparer/comments.rs:568); `split_run_before_leaf` (comparer/comments.rs:633); `add_at_paragraph_start` (comparer/comments.rs:734); `union_comments_xml` (comparer/comments.rs:1190); +176 more

#### `Dom::add_annotation`  (L1270-1275, pub)
`pub fn add_annotation<T: Any + 'static>(&mut self, id: NodeId, annotation: T)`

- **Does:** Attach a typed annotation to `id`.
- **Calls (std/external/unresolved):** `.entry`, `.or_default`, `.push`, `Box::new`
- **Called by:** none found in src

#### `Dom::annotation`  (L1277-1282, pub)
`pub fn annotation<T: Any + 'static>(&self, id: NodeId) -> Option<&T>`

- **Does:** The first annotation of type `T` attached to `id`, or `None`.
- **Calls (std/external/unresolved):** `.downcast_ref`, `.find_map`, `.get`, `.iter`
- **Called by:** none found in src

#### `Dom::remove_annotations`  (L1286-1293, pub)
`pub fn remove_annotations<T: Any + 'static>(&mut self, id: NodeId)`

- **Does:** Drop every annotation of type `T` from `id`.
- **Calls (crate):** `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L1288
- **Calls (std/external/unresolved):** `.get_mut`, `.is_empty`, `.remove`, `.retain`
- **Called by:** none found in src

#### `Dom::parse_xdocument`  (L1299-1301, pub)
`pub fn parse_xdocument(&mut self, xml: &str) -> NodeId`

- **Does:** Parse a full XML string (with optional prolog) into a Document node allocated into this arena.
- **Calls (crate):** `parse_xdocument` (xmllinq/parse.rs:100) L1300
- **Called by:** `parse` (append.rs:336); `drop_comments_in_new_parts` (append.rs:456); `insert_lists` (append.rs:1165); `nsids` (append.rs:1232); `tests::nsid_collisions_take_the_next_free_value` (append.rs:1357); `tests::section_type_sits_after_header_references` (append.rs:1404); `tests::ignorable_prefixes_merge_only_with_the_same_namespace` (append.rs:1423); `public_memory_append_owner_boundary_tests::view` (append.rs:1509); `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1599); `audit_stories` (changes.rs:168); `for_each_story` (changes.rs:657); `comment_changes_in` (changes.rs:1003); +474 more

#### `Dom::try_parse_xdocument`  (L1310-1312, pub)
`pub fn try_parse_xdocument(&mut self, xml: &str) -> Result<NodeId, String>`

- **Does:** Checked variant of [`Self::parse_xdocument`]: refuses input that fails [`parse::validate_xml`] (including the tag-width and namespace budgets) before any DOM node is built.
- **Calls (crate):** `try_parse_xdocument` (xmllinq/parse.rs:121) L1311
- **Called by:** none found in src

#### `Dom::serialize_element`  (L1315-1317, pub)
`pub fn serialize_element(&self, el: NodeId) -> String`

- **Does:** Serialize an element subtree to an XML string.
- **Calls (crate):** `serialize_element` (xmllinq/serialize.rs:401) L1316
- **Called by:** `append_documents` (append.rs:152); `ListCopy::build` (append.rs:1117); `final_section_table_ownership_tests::a_table_only_story_keeps_its_final_section_outside_all_cell_paragraphs` (comparer/atomize.rs:747) (?); `final_section_table_ownership_tests::block_control_paragraph_still_owns_the_closing_section_instead_of_later_table_cells` (comparer/atomize.rs:771) (?); `final_section_table_ownership_tests::existing_cell_section_properties_and_history_are_never_replaced_by_body_geometry` (comparer/atomize.rs:794) (?); `coverage_atomizer_owned_boundaries::note_definition_owns_every_character_and_mark_but_the_part_is_a_path_stop` (comparer/atomize.rs:827) (?); `coverage_atomizer_owned_boundaries::tracked_payload_status_and_paragraph_mark_status_have_independent_owners` (comparer/atomize.rs:973) (?); `coverage_atomizer_leaf_stamp_boundaries::opaque_leaf_stamps_salt_original_deletions_once_and_preserve_the_authored_payload` (comparer/atomize.rs:1213); `union_comments_xml` (comparer/comments.rs:1190); `drop_orphans` (comparer/comments.rs:1416); `rpr_string` (comparer/finalize.rs:1090); `coalesce_key` (comparer/finalize.rs:1097); +274 more

#### `Dom::serialize_element_sha1_hex`  (L1325-1327, pub)
`pub fn serialize_element_sha1_hex(&self, el: NodeId) -> String`

- **Does:** SHA-1 (lowercase hex) of the serialized element with the first WML default-xmlns declaration stripped from the root open tag.
- **Calls (crate):** `serialize_element_sha1_hex` (xmllinq/serialize.rs:418) L1326
- **Called by:** `block_sha1` (comparer/preprocess.rs:641)

#### `Dom::serialize_element_structure_sha1_hex`  (L1333-1335, pub)
`pub fn serialize_element_structure_sha1_hex(&self, el: NodeId) -> String`

- **Does:** SHA-1 (lowercase hex) of the structural shape only: attributes and element nesting, no text / comment / PI children, with the same first-WML-default-xmlns strip as [`Self::serialize_element_sha1_hex`].
- **Calls (crate):** `serialize_element_structure_sha1_hex` (xmllinq/serialize.rs:432) L1334
- **Called by:** `structure_sha1` (comparer/preprocess.rs:1213)

#### `Dom::serialize_document`  (L1338-1340, pub)
`pub fn serialize_document(&self, doc: NodeId) -> String`

- **Does:** Serialize a whole document (declaration + children) to a string.
- **Calls (crate):** `serialize_document` (xmllinq/serialize.rs:805) L1339
- **Called by:** `append_documents` (append.rs:152); `drop_comments_in_new_parts` (append.rs:456); `carry_styles_and_numbering` (append.rs:867); `merge_part_namespaces` (append.rs:1087); `insert_lists` (append.rs:1165); `CommentFamily::store` (comments.rs:854); `CommentFamily::store_people` (comments.rs:1033); `carry_with_own_relationships` (comparer/parts.rs:627); `complete_shape_revision_contract_tests::matching_opaque_shapes_keep_every_neighbor_when_revision_wrappers_split` (comparer/textbox.rs:260) (?); `complete_shape_revision_contract_tests::unrelated_or_mixed_shape_carriers_retain_complete_source_revisions` (comparer/textbox.rs:310) (?); `raw_source_projection_contract_tests::with_source` (debug/diff.rs:2509); `carry_revised_part_relationships` (document_comparer.rs:25); +26 more

## Test functions (6)

- `bump_attr_name_probes` (L283): Add one to this thread's [`ATTR_NAME_PROBES`].
- `Dom::shrink_arena_to_fit` (L422): Shrink the arena's backing `Vec` capacity to its current length so any subsequent internal push provably reallocates.
- `node_layout_tests::node_data_excludes_annotations_vec` (L1355): ANN-01 mechanism counter.
- `node_layout_tests::node_kind_rare_variants_are_boxed` (L1373): NODE-KIND-01 mechanism counter.
- `intern_tests::identical_names_share_interned_arcs` (L1400): PARSE-01 mechanism proof.
- `attribute_iter_tests::attributes_iter_borrows_the_pairs_attributes_clones` (L1426): `attributes_iter` walks the same pairs as `attributes`, in the same declaration order, borrowing instead of cloning.
