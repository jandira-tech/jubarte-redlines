# Functions and methods in `src/comparer/atomize.rs`

16 non-test, 14 test. Part of the [symbol manifest](../README.md).

#### `verify_no_invalid_content`  (L44-53, pub)
`pub fn verify_no_invalid_content(dom: &Dom, content_parent: NodeId) -> Result<(), String>`

- **Does:** `VerifyNoInvalidContent` (WmlComparer.ts :8678): error if any descendant of `content_parent` is a member of [`super::tables::INVALID_ELEMENTS`] (`altChunk`, `customXml`, `subDoc`, the customXml range markers).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L45; `Dom::name` (xmllinq/mod.rs:526) L46; `XName::local_name` (xmllinq/mod.rs:208) (by name) L49
- **Calls (std/external/unresolved):** `.contains`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `create_comparison_unit_atom_list` (comparer/atomize.rs:307)

#### `move_last_sectpr_into_last_paragraph`  (L63-106, pub)
`pub fn move_last_sectpr_into_last_paragraph( dom: &mut Dom, content_parent: NodeId, ) -> Result<(), String>`

- **Does:** `MoveLastSectPrIntoLastParagraph` (WmlComparer.ts :8819): move a trailing body-level `w:sectPr` into the last paragraph's `w:pPr`.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L67,75,102; `Dom::descendants` (xmllinq/mod.rs:668) L80; `Dom::ancestors` (xmllinq/mod.rs:770) L86; `Dom::element` (xmllinq/mod.rs:655) L92; `Dom::new_element` (xmllinq/mod.rs:478) L95; `Dom::add_first` (xmllinq/mod.rs:1101) L96; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L100; `Dom::add` (xmllinq/mod.rs:1074) L101; `Dom::remove` (xmllinq/mod.rs:1118) L103
- **Calls (std/external/unresolved):** `.copied`, `.find`, `.first`, `.into_iter`, `.is_empty`, `.last`, `.len`, `.or_else`, `.rev`, `.to_string`, `Err`, `Ok`, `Some`, `W::p_pr`, `W::p`, `W::sect_pr`, `W::tc`
- **Called by:** `create_comparison_unit_atom_list` (comparer/atomize.rs:307); `final_section_table_ownership_tests::a_table_only_story_keeps_its_final_section_outside_all_cell_paragraphs` (comparer/atomize.rs:747); `final_section_table_ownership_tests::block_control_paragraph_still_owns_the_closing_section_instead_of_later_table_cells` (comparer/atomize.rs:771); `final_section_table_ownership_tests::existing_cell_section_properties_and_history_are_never_replaced_by_body_geometry` (comparer/atomize.rs:794)

#### `revision_tracking_element_from_ancestors`  (L112-132)
`fn revision_tracking_element_from_ancestors( dom: &Dom, content: NodeId, ancestors: &[NodeId], ) -> Option<NodeId>`

- **Does:** D.1 — `GetRevisionTrackingElementFromAncestors` (:8945): the `w:del`/ `w:ins`/`w:moveFrom`/`w:moveTo` element that gives an atom its status.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L117; `Dom::elements` (xmllinq/mod.rs:640) L118,119; `Dom::name` (xmllinq/mod.rs:526) L120,129
- **Calls (std/external/unresolved):** `.copied`, `.find`, `.iter`, `.unwrap`, `Some`, `W::del`, `W::ins`, `W::move_from`, `W::move_to`, `W::p_pr`, `W::r_pr`
- **Called by:** `push_atom` (comparer/atomize.rs:346)

#### `status_from_rev_track_element`  (L136-154)
`fn status_from_rev_track_element(dom: &Dom, rte: Option<NodeId>) -> CorrelationStatus`

- **Does:** D.1 — the ComparisonUnitAtom ctor's status mapping (:8909): derive the correlation status FROM the revision tracking element's name.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L140
- **Calls (std/external/unresolved):** `.unwrap`, `W::del`, `W::ins`, `W::move_from`, `W::move_to`
- **Called by:** `push_atom` (comparer/atomize.rs:346)

#### `atom_hash`  (L163-204)
`fn atom_hash(dom: &Dom, content: NodeId, settings: &WmlComparerSettings) -> AtomHash`

- **Does:** `GetSha1HashStringForElement` + the atom hash (localName + normalized text).
- **Calls (crate):** `Dom::value` (xmllinq/mod.rs:1185) L164; `Dom::name` (xmllinq/mod.rs:526) L173; `XName::local_name` (xmllinq/mod.rs:208) (by name) L175; `Dom::attribute` (xmllinq/mod.rs:874) L178,184,193; `AtomHash::from_hex` (comparer/atoms.rs:59) L179; `Dom::name_is` (xmllinq/mod.rs:535) L191; `AtomHash::of_bytes` (comparer/atoms.rs:49) L203
- **Calls (std/external/unresolved):** `.as_bytes`, `.map`, `.push_str`, `.push`, `.replace`, `.to_string`, `.to_uppercase`, `.unwrap_or_default`, `PT::sha1_hash`, `W::name`
- **Macros:** `format!`
- **Called by:** `push_atom` (comparer/atomize.rs:346); `break_payload_identity_regressions::break_defaults_match_but_authored_kind_and_clearance_do_not` (comparer/atomize.rs:703); `coverage_atomizer_owned_boundaries::original_deletion_salt_follows_its_owner_and_never_salts_revised_or_inserted_content` (comparer/atomize.rs:918); `coverage_atomizer_owned_boundaries::stamped_hash_is_authoritative_while_fresh_text_normalization_is_option_owned` (comparer/atomize.rs:1030); `coverage_atomizer_field_chain_boundaries::fields_without_cached_results_still_own_their_complete_nested_instruction_chain` (comparer/atomize.rs:1080); `coverage_atomizer_leaf_stamp_boundaries::opaque_leaf_stamps_salt_original_deletions_once_and_preserve_the_authored_payload` (comparer/atomize.rs:1213)

#### `salt_field_results`  (L217-287)
`fn salt_field_results(dom: &Dom, list: &mut [ComparisonUnitAtom])`

- **Does:** Salts every atom of a complex field with the codes of the fields around it.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L223,252; `Dom::attribute` (xmllinq/mod.rs:874) L224; `kind` (ambiguous: `tests::kind` (admission.rs:1443); `tests::kind` (edit/flags.rs:894); `tests::kind` (markdown/from_docx/critic.rs:1518)) L237,267; `Dom::value` (xmllinq/mod.rs:1185) L253; `AtomHash::to_hex_string` (comparer/atoms.rs:70) (by name) L280; `AtomHash::of_bytes` (comparer/atoms.rs:49) L281
- **Calls (std/external/unresolved):** `.as_bytes`, `.enumerate`, `.filter`, `.is_empty`, `.is_some_and`, `.iter_mut`, `.iter`, `.last_mut`, `.len`, `.pop`, `.push_str`, `.push`, `.trim`, `Some`, `String::from`, `String::new`, `Vec::new`, `W::name`
- **Macros:** `vec!`
- **Called by:** `create_comparison_unit_atom_list` (comparer/atomize.rs:307)

#### `create_comparison_unit_atom_list`  (L307-323, pub)
`pub fn create_comparison_unit_atom_list( dom: &mut Dom, content_parent: NodeId, settings: &WmlComparerSettings, ) -> Vec<ComparisonUnitAtom>`

- **Does:** `CreateComparisonUnitAtomList(contentParent)` — assign `pt:Unid` to every element under `content_parent`, move the trailing body `w:sectPr` into the last paragraph's `w:pPr` (when present), then flatten the tree into a per-character / per-leaf atom stream.
- **Calls (crate):** `verify_no_invalid_content` (comparer/atomize.rs:44) L312; `assign_to_all_elements` (unid.rs:60) L313; `move_last_sectpr_into_last_paragraph` (comparer/atomize.rs:63) L314; `recurse` (comparer/atomize.rs:410) L320; `salt_field_results` (comparer/atomize.rs:217) L321
- **Calls (std/external/unresolved):** `.expect`, `Vec::new`
- **Called by:** `coverage_atomizer_owned_boundaries::note_definition_owns_every_character_and_mark_but_the_part_is_a_path_stop` (comparer/atomize.rs:827); `coverage_atomizer_owned_boundaries::original_deletion_salt_follows_its_owner_and_never_salts_revised_or_inserted_content` (comparer/atomize.rs:918); `coverage_atomizer_owned_boundaries::tracked_payload_status_and_paragraph_mark_status_have_independent_owners` (comparer/atomize.rs:973); `coverage_atomizer_field_chain_boundaries::field_atoms` (comparer/atomize.rs:1065); `coverage_atomizer_field_chain_boundaries::header_and_footer_roots_stop_paths_and_preserve_each_whitespace_boundary` (comparer/atomize.rs:1123); `coverage_atomizer_leaf_stamp_boundaries::opaque_leaf_stamps_salt_original_deletions_once_and_preserve_the_authored_payload` (comparer/atomize.rs:1213); `public_option_source_ownership_tests::sourced_cross_paragraph_windows_decline_quadratic_tables_without_mutating_owners::{fn member}` (comparer/cross_para.rs:3257); `process_footnote_endnote` (comparer/footnotes.rs:1115); `nested_paragraph_property_source_history_tests::owned_mark` (comparer/formatchg.rs:775); `coverage_real_source_route_matrix_tests::source` (comparer/lcs.rs:18303); `horizontal_partition_tests::actual_atomized_split_with_deleted_table_conserves_source_through_production` (comparer/lcs_table.rs:1186); `horizontal_partition_tests::row_unit` (comparer/lcs_table.rs:1383); +10 more

#### `annotate_element_with_props`  (L327-344)
`fn annotate_element_with_props( dom: &mut Dom, element: NodeId, list: &mut Vec<ComparisonUnitAtom>, child_property_names: Option<&[XName]>, settings: &WmlComparerSettings, path: &mut Vec<NodeId>, )`

- **Does:** `AnnotateElementWithProps` (:8971) — recurse into child elements, skipping the declared property children (which Coalesce re-attaches structurally).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L335; `Dom::name` (xmllinq/mod.rs:526) L336; `recurse` (comparer/atomize.rs:410) L341
- **Calls (std/external/unresolved):** `.contains`
- **Called by:** `recurse` (comparer/atomize.rs:410)

#### `push_atom`  (L346-378)
`fn push_atom( dom: &Dom, content: NodeId, ancestors: &Arc<[NodeId]>, list: &mut Vec<ComparisonUnitAtom>, settings: &WmlComparerSettings, )`

- **Does:** _no doc comment_
- **Calls (crate):** `atom_hash` (comparer/atomize.rs:163) L353; `Dom::attribute` (xmllinq/mod.rs:874) L362,365; `AtomHash::of_bytes` (comparer/atoms.rs:49) L370; `AtomHash::to_hex_string` (comparer/atoms.rs:70) (by name) L370; `ComparisonUnitAtom::new` (comparer/atoms.rs:198) L373; `revision_tracking_element_from_ancestors` (comparer/atomize.rs:112) L375; `status_from_rev_track_element` (comparer/atomize.rs:136) L376
- **Calls (std/external/unresolved):** `.any`, `.as_bytes`, `.as_ref`, `.iter`, `.push`, `Arc::clone`, `PT::name`, `Some`
- **Macros:** `format!`
- **Called by:** `recurse` (comparer/atomize.rs:410)

#### `chain_with`  (L382-387)
`fn chain_with(path: &[NodeId], element: NodeId) -> Arc<[NodeId]>`

- **Does:** Chain for an atom at `element`: `path` (ancestors excluding body) + `element`.
- **Calls (std/external/unresolved):** `.extend_from_slice`, `.len`, `.push`, `Arc::from`, `Vec::with_capacity`
- **Called by:** `recurse` (comparer/atomize.rs:410)

#### `recurse_children`  (L393-408)
`fn recurse_children( dom: &mut Dom, element: NodeId, exclude: Option<&XName>, list: &mut Vec<ComparisonUnitAtom>, settings: &WmlComparerSettings, path: &mut Vec<NodeId>, )`

- **Does:** Recurses into each named child of `element` except one named `exclude`.
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L401; `Dom::child_at` (xmllinq/mod.rs:632) L402; `Dom::name` (xmllinq/mod.rs:526) L403; `recurse` (comparer/atomize.rs:410) L404
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `recurse` (comparer/atomize.rs:410)

#### `recurse`  (L410-553)
`fn recurse( dom: &mut Dom, element: NodeId, list: &mut Vec<ComparisonUnitAtom>, settings: &WmlComparerSettings, path: &mut Vec<NodeId>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L417; `recurse_children` (comparer/atomize.rs:393) L432,438,445,452,472; `Dom::element` (xmllinq/mod.rs:655) L459; `Dom::new_element` (xmllinq/mod.rs:478) L462,484; `chain_with` (comparer/atomize.rs:382) L464,481,493,505,512,524; `push_atom` (comparer/atomize.rs:346) L465,486,494,506,513,525; `Dom::value` (xmllinq/mod.rs:1185) L479; `Dom::add_text` (xmllinq/mod.rs:1087) L485; `Dom::elements` (xmllinq/mod.rs:640) L523; `recursion_info` (comparer/tables.rs:215) L530; `annotate_element_with_props` (comparer/atomize.rs:327) L532,551
- **Calls (std/external/unresolved):** `.as_deref`, `.chars`, `.clone`, `.contains`, `.is_empty`, `.pop`, `.push`, `.to_string`, `MC::name`, `Some`, `W::body`, `W::del_text`, `W::endnote`, `W::footnote`, `W::name`, `W::object`, `W::p_pr`, `W::p`, `W::pict`, `W::r_pr`, `W::r`, `W::t`
- **Called by:** `create_comparison_unit_atom_list` (comparer/atomize.rs:307); `annotate_element_with_props` (comparer/atomize.rs:327); `recurse_children` (comparer/atomize.rs:393)

#### `coalesce`  (L557-571, pub)
`pub fn coalesce(dom: &mut Dom, atoms: &[ComparisonUnitAtom]) -> NodeId`

- **Does:** `Coalesce(atomList)` — rebuild a `<w:document><w:body>…` from the atom stream.
- **Calls (crate):** `Dom::new_document` (xmllinq/mod.rs:472) L558; `Dom::new_element` (xmllinq/mod.rs:478) L559,563; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L561,562; `XNamespace::xmlns` (xmllinq/mod.rs:122) L561,562; `coalesce_recurse` (comparer/atomize.rs:575) L564; `Dom::add` (xmllinq/mod.rs:1074) L566,568,569
- **Calls (std/external/unresolved):** `.name`, `Some`, `W::body`, `W::document`
- **Called by:** `coverage_atomizer_owned_boundaries::note_definition_owns_every_character_and_mark_but_the_part_is_a_path_stop` (comparer/atomize.rs:827); `coverage_atomizer_field_chain_boundaries::header_and_footer_roots_stop_paths_and_preserve_each_whitespace_boundary` (comparer/atomize.rs:1123)

#### `coalesce_recurse`  (L575-658)
`fn coalesce_recurse(dom: &mut Dom, atoms: &[ComparisonUnitAtom], level: usize) -> Vec<NodeId>`

- **Does:** Port of `CoalesceRecurseSimple` — regroup atoms by ancestor Unid at `level`, rebuild each ancestor element, recursing deeper.
- **Calls (crate):** `group_by_ancestor_unid` (comparer/atomize.rs:671) L577; `Dom::name` (xmllinq/mod.rs:526) L581,586,615; `group_adjacent` (util/group_adjacent.rs:20) L585,614; `Dom::new_element` (xmllinq/mod.rs:478) L588,617,629,647; `Dom::attributes` (xmllinq/mod.rs:899) L589,648; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L590,631,649; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L596,620,637; `Dom::add` (xmllinq/mod.rs:1074) L597,605,621,634,638,653; `coalesce_recurse` (comparer/atomize.rs:575) L603,651; `Dom::elements` (xmllinq/mod.rs:640) L619; `Dom::value_str` (xmllinq/mod.rs:1195) L627; `xml_space_attr` (comparer/atomize.rs:661) L630; `XNamespace::xml` (xmllinq/mod.rs:128) L631; `Dom::add_text` (xmllinq/mod.rs:1087) L633
- **Calls (std/external/unresolved):** `.clone`, `.cloned`, `.collect`, `.iter`, `.map`, `.name`, `.push`, `.unwrap`, `Some`, `Vec::new`, `W::del_text`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `W::t`
- **Called by:** `coalesce` (comparer/atomize.rs:557); `coalesce_recurse` (comparer/atomize.rs:575)

#### `xml_space_attr`  (L661-667)
`fn xml_space_attr(text: &str) -> Option<&'static str>`

- **Does:** `GetXmlSpaceAttribute` — returns Some("preserve") when leading/trailing space.
- **Calls (std/external/unresolved):** `.ends_with`, `.starts_with`, `Some`
- **Called by:** `coalesce_recurse` (comparer/atomize.rs:575)

#### `group_by_ancestor_unid`  (L671-692)
`fn group_by_ancestor_unid( dom: &Dom, atoms: &[ComparisonUnitAtom], level: usize, ) -> Vec<Vec<ComparisonUnitAtom>>`

- **Does:** Group atoms by the `pt14:Unid` of their ancestor at `level`, preserving the order of first appearance (port of `groupByKey`).
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L682
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.contains_key`, `.entry`, `.into_iter`, `.map`, `.or_default`, `.push`, `.remove`, `.to_string`, `.unwrap_or`, `.unwrap`, `PT::unid`, `Vec::new`, `std::collections::HashMap::new`
- **Called by:** `coalesce_recurse` (comparer/atomize.rs:575)

## Test functions (14)

- `break_payload_identity_regressions::break_defaults_match_but_authored_kind_and_clearance_do_not` (L703): _no doc comment_
- `final_section_table_ownership_tests::parse` (L738): _no doc comment_
- `final_section_table_ownership_tests::a_table_only_story_keeps_its_final_section_outside_all_cell_paragraphs` (L747): _no doc comment_
- `final_section_table_ownership_tests::block_control_paragraph_still_owns_the_closing_section_instead_of_later_table_cells` (L771): _no doc comment_
- `final_section_table_ownership_tests::existing_cell_section_properties_and_history_are_never_replaced_by_body_geometry` (L794): _no doc comment_
- `coverage_atomizer_owned_boundaries::parse` (L815): _no doc comment_
- `coverage_atomizer_owned_boundaries::note_definition_owns_every_character_and_mark_but_the_part_is_a_path_stop` (L827): _no doc comment_
- `coverage_atomizer_owned_boundaries::original_deletion_salt_follows_its_owner_and_never_salts_revised_or_inserted_content` (L918): _no doc comment_
- `coverage_atomizer_owned_boundaries::tracked_payload_status_and_paragraph_mark_status_have_independent_owners` (L973): _no doc comment_
- `coverage_atomizer_owned_boundaries::stamped_hash_is_authoritative_while_fresh_text_normalization_is_option_owned` (L1030): _no doc comment_
- `coverage_atomizer_field_chain_boundaries::field_atoms` (L1065): _no doc comment_
- `coverage_atomizer_field_chain_boundaries::fields_without_cached_results_still_own_their_complete_nested_instruction_chain` (L1080): _no doc comment_
- `coverage_atomizer_field_chain_boundaries::header_and_footer_roots_stop_paths_and_preserve_each_whitespace_boundary` (L1123): _no doc comment_
- `coverage_atomizer_leaf_stamp_boundaries::opaque_leaf_stamps_salt_original_deletions_once_and_preserve_the_authored_payload` (L1213): _no doc comment_
