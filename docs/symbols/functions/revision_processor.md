# Functions and methods in `src/revision_processor.rs`

92 non-test, 55 test. Part of the [symbol manifest](../README.md).

#### `tracked_revisions_elements`  (L80-82, pub)
`pub fn tracked_revisions_elements() -> Vec<XName>`

- **Does:** The element names whose presence indicates the document carries tracked revisions (one per local name in the `w:` namespace).
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`, `W::name`
- **Called by:** none found in src

#### `element_has_tracked_revisions`  (L90-106, pub)
`pub fn element_has_tracked_revisions(dom: &Dom, root: NodeId) -> bool`

- **Does:** True if `root` (or any descendant) is a tracked-revision element.
- **Calls (crate):** `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:91) L105
- **Called by:** `is_restorable_layout_blank` (comparer/finalize.rs:3818); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209); `accept_move_from_move_to_transform` (revision_processor.rs:129); `accept_all_other_revisions_transform` (revision_processor.rs:229); `accept_revisions_for_element` (revision_processor.rs:459); `reject_revisions_document` (revision_processor.rs:729); `accept_revisions_for_part_content` (revision_processor.rs:3252); `public_block_control_ownership_tests::nested_block_controls_keep_each_owner_and_transformed_paragraph_payload` (revision_processor.rs:4495); `public_block_control_ownership_tests::nested_block_controls_keep_outer_table_owner_and_nested_cell_geometry` (revision_processor.rs:4569)

#### `element_has_tracked_revisions::{fn walk}`  (L91-104)
`fn walk(dom: &Dom, id: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L92; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L93; `XName::local_name` (xmllinq/mod.rs:208) (by name) L94; `Dom::child_count` (xmllinq/mod.rs:622) L98; `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:91) L99; `Dom::child_at` (xmllinq/mod.rs:632) L99
- **Calls (std/external/unresolved):** `.contains`
- **Called by:** `element_has_tracked_revisions` (revision_processor.rs:90); `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:91); `element_or_desc_has_name` (revision_processor.rs:184); `element_or_desc_has_name::{fn walk}` (revision_processor.rs:185); `has_deleted_or_moved_from_paragraph_mark` (revision_processor.rs:1771); `has_deleted_or_moved_from_paragraph_mark::{fn walk}` (revision_processor.rs:1772); `subtree_needs_adjacent_table_merge` (revision_processor.rs:2909); `subtree_needs_adjacent_table_merge::{fn walk}` (revision_processor.rs:2910); `has_empty_table_cell` (revision_processor.rs:3159); `has_empty_table_cell::{fn walk}` (revision_processor.rs:3162)

#### `has_child`  (L109-111)
`fn has_child(dom: &Dom, el: NodeId, name: &XName) -> bool`

- **Does:** Does `el` have a child element named `local` (in the W namespace)?
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L110
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `has_path` (revision_processor.rs:114)

#### `has_path`  (L114-118)
`fn has_path(dom: &Dom, el: NodeId, a: &XName, b: &XName) -> bool`

- **Does:** Path check `el / a / b` exists (a, b are direct-child element names).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L115; `has_child` (revision_processor.rs:109) L117
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `Some`
- **Called by:** `accept_all_other_revisions_transform` (revision_processor.rs:229)

#### `accept_move_from_move_to_transform`  (L129-172, pub)
`pub fn accept_move_from_move_to_transform(dom: &mut Dom, node: NodeId) -> Vec<NodeId>`

- **Does:** Port of `AcceptMoveFromMoveToTransform` — unwrap `w:moveTo`, drop `w:moveFrom`.
- **Calls (crate):** `element_has_tracked_revisions` (revision_processor.rs:90) L131; `take_subtree` (revision_processor.rs:176) L132,135,154; `Dom::is_element` (xmllinq/mod.rs:502) L134; `Dom::name` (xmllinq/mod.rs:526) L137,150,151; `Dom::nodes` (xmllinq/mod.rs:604) L140,166; `accept_move_from_move_to_transform` (revision_processor.rs:129) L141,167; `Dom::parent` (xmllinq/mod.rs:597) L149,151; `hoist_range_markers_from` (revision_processor.rs:2074) L159; `Dom::new_element` (xmllinq/mod.rs:478) L162; `Dom::attributes` (xmllinq/mod.rs:885) L163; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L164; `Dom::add` (xmllinq/mod.rs:1058) L168
- **Calls (std/external/unresolved):** `.and_then`, `.extend`, `.is_some_and`, `.unwrap`, `Some`, `Vec::new`, `W::move_from`, `W::move_to`, `W::p_pr`, `W::r_pr`
- **Macros:** `vec!`
- **Called by:** `accept_move_from_move_to_transform` (revision_processor.rs:129); `accept_revisions_for_element` (revision_processor.rs:459); `accept_revisions_for_part_content` (revision_processor.rs:3252)

#### `take_subtree`  (L176-181)
`fn take_subtree(dom: &mut Dom, node: NodeId) -> Vec<NodeId>`

- **Does:** Detach `node` from its parent (if any) and return it for reparenting.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L177; `Dom::remove` (xmllinq/mod.rs:1102) L178
- **Calls (std/external/unresolved):** `.is_some`
- **Macros:** `vec!`
- **Called by:** `accept_move_from_move_to_transform` (revision_processor.rs:129); `accept_all_other_revisions_transform` (revision_processor.rs:229)

#### `element_or_desc_has_name`  (L184-198)
`fn element_or_desc_has_name(dom: &Dom, root: NodeId, name: &XName) -> bool`

- **Does:** Non-allocating presence of `name` on `root` or any descendant element.
- **Calls (crate):** `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:91) L197
- **Called by:** `fix_up_deleted_or_inserted_field_codes_transform` (revision_processor.rs:1037); `accept_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1355); `accept_deleted_cells_transform` (revision_processor.rs:2627); `table_has_revision_marks` (revision_processor.rs:2827)

#### `element_or_desc_has_name::{fn walk}`  (L185-196)
`fn walk(dom: &Dom, id: NodeId, name: &XName) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L186; `Dom::child_count` (xmllinq/mod.rs:622) L189; `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:91) L191; `Dom::child_at` (xmllinq/mod.rs:632) L191
- **Calls (std/external/unresolved):** `.as_ref`, `Some`
- **Called by:** `element_has_tracked_revisions` (revision_processor.rs:90); `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:91); `element_or_desc_has_name` (revision_processor.rs:184); `element_or_desc_has_name::{fn walk}` (revision_processor.rs:185); `has_deleted_or_moved_from_paragraph_mark` (revision_processor.rs:1771); `has_deleted_or_moved_from_paragraph_mark::{fn walk}` (revision_processor.rs:1772); `subtree_needs_adjacent_table_merge` (revision_processor.rs:2909); `subtree_needs_adjacent_table_merge::{fn walk}` (revision_processor.rs:2910); `has_empty_table_cell` (revision_processor.rs:3159); `has_empty_table_cell::{fn walk}` (revision_processor.rs:3162)

#### `has_run_content`  (L202-215)
`fn has_run_content(dom: &Dom, element: NodeId) -> bool`

- **Does:** True iff a transformed element still carries run/inline content (port of `HasRunContent`) — used to decide whether an emptied `w:hyperlink` survives.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L212; `Dom::name` (xmllinq/mod.rs:526) L214
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.into_iter`, `.is_some_and`, `W::del`, `W::fld_simple`, `W::hyperlink`, `W::ins`, `W::r`, `W::sdt`, `W::smart_tag`
- **Called by:** `accept_all_other_revisions_transform` (revision_processor.rs:229)

#### `accept_all_other_revisions_transform`  (L229-446, pub)
`pub fn accept_all_other_revisions_transform(dom: &mut Dom, node: NodeId) -> Vec<NodeId>`

- **Does:** Port of `AcceptAllOtherRevisionsTransform` — accept inserts (unwrap `w:ins`), drop deletions and revision-range markers, accept formatting- change markers, handle deleted rows/tables, cell merges, and empty hyperlink shells.
- **Calls (crate):** `element_has_tracked_revisions` (revision_processor.rs:90) L231; `take_subtree` (revision_processor.rs:176) L232,235; `Dom::is_element` (xmllinq/mod.rs:502) L234,345; `Dom::name` (xmllinq/mod.rs:526) L237,314,331,398; `Dom::nodes` (xmllinq/mod.rs:604) L242,344,421,440; `accept_all_other_revisions_transform` (revision_processor.rs:229) L243,422,441; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L263,282,321,332; `XName::local_name` (xmllinq/mod.rs:208) (by name) L263,282,324,329,332; `Dom::descendants` (xmllinq/mod.rs:668) L292,297,301,305; `Dom::name_is` (xmllinq/mod.rs:535) L295,303,307,310,343; `Dom::ancestors` (xmllinq/mod.rs:770) L306; `Dom::text_value` (xmllinq/mod.rs:547) L315,346; `Dom::elements` (xmllinq/mod.rs:640) L335,341,358,374; `hoist_range_markers_from` (revision_processor.rs:2074) L352,391,431; `has_path` (revision_processor.rs:114) L361,368,378; `Dom::parent` (xmllinq/mod.rs:597) L396; `Dom::attribute` (xmllinq/mod.rs:874) L401; `Dom::new_element` (xmllinq/mod.rs:478) L403,408,417,436; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L404,409,419,438; `Dom::attributes` (xmllinq/mod.rs:885) L418,437; `Dom::add` (xmllinq/mod.rs:1058) L423,442; `has_run_content` (revision_processor.rs:202) L426
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.as_deref`, `.collect`, `.contains`, `.ends_with`, `.extend`, `.filter`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.map`, `.to_string`, `.trim`, `.unwrap`, `M::name`, `Some`, `Vec::new`, `W::cell_merge`, `W::del`, `W::hyperlink`, `W::ins`, `W::tbl`, `W::tc_pr`, `W::tr_pr`, `W::tr`, `W::v_merge`, `W::val`
- **Macros:** `vec!`
- **Called by:** `accept_all_other_revisions_transform` (revision_processor.rs:229); `accept_revisions_for_element` (revision_processor.rs:459); `all_para_content_is_deleted` (revision_processor.rs:1717); `accept_revisions_for_part_content` (revision_processor.rs:3252); `revision_boundary_coverage_tests::accept_cell_merge_converts_restart_and_continue_only_inside_cell_properties` (revision_processor.rs:4034); `revision_boundary_coverage_tests::internally_deleted_math_does_not_consume_independent_anchors_controls_or_empty_branches` (revision_processor.rs:4171); `revision_boundary_coverage_tests::wholly_deleted_math_hoists_owned_range_metadata_instead_of_discarding_it` (revision_processor.rs:4234); `revision_boundary_coverage_tests::accept_deleted_fraction_drops_its_math_payload_and_preserves_live_fraction` (revision_processor.rs:4262)

#### `accept_revisions_for_element`  (L459-489, pub)
`pub fn accept_revisions_for_element(dom: &mut Dom, element: NodeId) -> NodeId`

- **Does:** Port of `AcceptRevisionsForElement` (the documented simple-revision entry point, RevisionProcessor.ts:1004): RemoveRsid → AcceptMoveFromMoveTo → AcceptAllOtherRevisions → strip PT.UniqueId/RunIds → drop empty w:numPr.
- **Calls (crate):** `element_has_tracked_revisions` (revision_processor.rs:90) L460; `remove_rsid_transform` (markup_simplifier.rs:154) L461; `accept_move_from_move_to_transform` (revision_processor.rs:129) L463; `accept_all_other_revisions_transform` (revision_processor.rs:229) L466; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L476; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L477,478; `Dom::descendants` (xmllinq/mod.rs:668) L483; `Dom::has_elements` (xmllinq/mod.rs:862) L484; `Dom::remove` (xmllinq/mod.rs:1102) L485
- **Calls (std/external/unresolved):** `.expect`, `.len`, `PT::run_ids`, `PT::unique_id`, `Some`, `W::num_pr`
- **Macros:** `debug_assert_eq!`
- **Called by:** `revision_boundary_coverage_tests::acceptance_removes_empty_numbering_but_keeps_live_numbering_and_clears_atom_annotations` (revision_processor.rs:4333)

#### `accept_revisions_document`  (L502-504, pub)
`pub fn accept_revisions_document(dom: &mut Dom, root: NodeId) -> NodeId`

- **Does:** `AcceptRevisionsDocument` at element scope — accept all tracked revisions in the `root` element's subtree, returning the new root.
- **Calls (crate):** `accept_revisions_for_part_content` (revision_processor.rs:3252) L503
- **Called by:** `coverage_round_two_tests::wholesale_mesh_preserves_each_authored_terminal_whitespace_and_format::{fn source_projection}` (comparer/finalize.rs:16892); `coverage_round_two_tests::wholesale_mesh_without_terminal_whitespace_keeps_exact_supported_word_streams::{fn owned}` (comparer/finalize.rs:16943); `coverage_round_next_tests::trailing_deletion_peel_truth_table_for_shared_long_and_copula_paths::{fn projected_body}` (comparer/finalize.rs:19534); `deep_production_path_tests::title_spacers_require_unique_revised_two_blank_pretable_context` (comparer/finalize.rs:21795); `deep_production_path_tests::restored_title_spacers_preserve_each_revised_blank_template` (comparer/finalize.rs:21887); `deep_production_path_tests::short_title_splice_crosses_only_tables_whose_every_row_is_structurally_inserted::{fn projected}` (comparer/finalize.rs:22443); `deep_production_path_tests::deletion_peel_keeps_retained_source_boundary_and_inserted_continuations::{fn paragraphs}` (comparer/finalize.rs:22617); `deep_production_path_tests::final_deletion_peel_transfers_boundary_only_with_equal_properties_and_text_payload::{fn projected}` (comparer/finalize.rs:22683); `deep_production_path_tests::deletion_peel_keeps_nontext_only_original_anchors_in_their_source_paragraph::{fn projected}` (comparer/finalize.rs:22774); `deep_production_path_tests::deletion_peel_keeps_punctuation_and_whitespace_only_original_anchors::{fn projection}` (comparer/finalize.rs:22833); `deep_production_path_tests::deletion_peel_keeps_original_closing_boundary_when_all_original_words_are_deleted::{fn projection}` (comparer/finalize.rs:22880); `deep_production_path_tests::wholesale_deleted_body_peel_transfers_its_retained_closing_boundary::{fn projection}` (comparer/finalize.rs:22925); +37 more

#### `effective_parent`  (L508-513)
`fn effective_parent(dom: &Dom, node: NodeId) -> Option<NodeId>`

- **Does:** The "skipping" parent used by `ReverseRevisionsTransform` — nearest ancestor whose name is not `w:sdtContent`/`w:sdt`/`w:smartTag`.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L510; `Dom::name` (xmllinq/mod.rs:526) L512
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.into_iter`, `.is_some_and`, `W::sdt_content`, `W::sdt`, `W::smart_tag`
- **Called by:** `reverse_revisions_transform` (revision_processor.rs:523)

#### `reverse_revisions_transform`  (L523-596)
`fn reverse_revisions_transform(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** Port of `ReverseRevisionsForPartTransform` — invert the *sense* of each revision (del↔ins, moveFrom↔moveTo, delText→t, custom-xml ranges, …), context-aware via the effective parent.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L524; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L525; `Dom::name` (xmllinq/mod.rs:526) L527,529,532; `effective_parent` (revision_processor.rs:508) L528; `Dom::parent` (xmllinq/mod.rs:597) L531; `Dom::new_element` (xmllinq/mod.rs:478) L537,541,545,548; `rebuild_named` (revision_processor.rs:599) L561,564,591,595; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L586; `XName::local_name` (xmllinq/mod.rs:208) (by name) L589
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.find`, `.is_some_and`, `.iter`, `.unwrap`, `Some`, `W::del`, `W::ins`, `W::name`, `W::p_pr`, `W::r_pr`, `W::tr_pr`
- **Called by:** `rebuild_named` (revision_processor.rs:599); `reject_revisions_document` (revision_processor.rs:729)

#### `rebuild_named`  (L599-611)
`fn rebuild_named(dom: &mut Dom, new_name: XName, src: NodeId, keep_attrs: bool) -> NodeId`

- **Does:** Build `<new_name attrs?
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L600; `Dom::attributes` (xmllinq/mod.rs:885) L602; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L603; `Dom::nodes` (xmllinq/mod.rs:604) L606; `reverse_revisions_transform` (revision_processor.rs:523) L607; `Dom::add` (xmllinq/mod.rs:1058) L608
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `reverse_revisions_transform` (revision_processor.rs:523)

#### `reject_revisions_for_part_transform`  (L619-716)
`fn reject_revisions_for_part_transform(dom: &mut Dom, node: NodeId) -> Option<NodeId>`

- **Does:** Port of `RejectRevisionsForPartTransform` — revert the *non-invertible* revisions (property changes) and drop inserted structural markers.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L620; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L621,647,654,671,684; `Dom::name` (xmllinq/mod.rs:526) L623,665; `Dom::element` (xmllinq/mod.rs:655) L626,643,646,652,680,682...; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L639; `XName::local_name` (xmllinq/mod.rs:208) (by name) L642; `Dom::new_element` (xmllinq/mod.rs:478) L648,685,706; `Dom::add` (xmllinq/mod.rs:1058) L655,712; `Dom::elements` (xmllinq/mod.rs:640) L661,696; `Dom::add_first` (xmllinq/mod.rs:1085) L672; `carry_kept_revisions` (revision_processor.rs:3315) L675,687; `reject_revisions_for_part_transform` (revision_processor.rs:619) L676,688,711; `Dom::attributes` (xmllinq/mod.rs:885) L707; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L708; `Dom::nodes` (xmllinq/mod.rs:604) L710
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.filter`, `.find`, `.into_iter`, `.is_some_and`, `.is_some`, `.iter`, `.rev`, `.unwrap`, `Some`, `W::cell_del`, `W::cell_ins`, `W::cell_merge`, `W::ins`, `W::name`, `W::num_pr`, `W::numbering_change`, `W::p_pr`, `W::r_pr_change`, `W::r_pr`, `W::sect_pr`, `W::tc_pr`, `W::tc`
- **Called by:** `reject_revisions_for_part_transform` (revision_processor.rs:619); `reject_revisions_document` (revision_processor.rs:729); `revision_boundary_coverage_tests::rejecting_structural_markers_drops_inserted_numbering_and_cell_without_losing_neighbors` (revision_processor.rs:4276)

#### `reject_revisions_document`  (L729-738, pub)
`pub fn reject_revisions_document(dom: &mut Dom, root: NodeId) -> NodeId`

- **Does:** `RejectRevisionsDocument` at element scope: revert property changes, invert the sense of every remaining revision, strip rsids, then accept.
- **Calls (crate):** `element_has_tracked_revisions` (revision_processor.rs:90) L730; `remove_rsid_transform` (markup_simplifier.rs:154) L731,736; `reject_revisions_for_part_transform` (revision_processor.rs:619) L734; `reverse_revisions_transform` (revision_processor.rs:523) L735; `accept_revisions_for_part_content` (revision_processor.rs:3252) L737
- **Calls (std/external/unresolved):** `.expect`
- **Called by:** `coverage_round_two_tests::wholesale_mesh_preserves_each_authored_terminal_whitespace_and_format::{fn source_projection}` (comparer/finalize.rs:16892); `coverage_round_two_tests::wholesale_mesh_without_terminal_whitespace_keeps_exact_supported_word_streams::{fn owned}` (comparer/finalize.rs:16943); `coverage_round_next_tests::trailing_deletion_peel_truth_table_for_shared_long_and_copula_paths::{fn projected_body}` (comparer/finalize.rs:19534); `deep_production_path_tests::title_spacers_require_unique_revised_two_blank_pretable_context` (comparer/finalize.rs:21795); `deep_production_path_tests::short_title_splice_crosses_only_tables_whose_every_row_is_structurally_inserted` (comparer/finalize.rs:22442); `deep_production_path_tests::short_title_splice_crosses_only_tables_whose_every_row_is_structurally_inserted::{fn projected}` (comparer/finalize.rs:22443); `deep_production_path_tests::deletion_peel_keeps_retained_source_boundary_and_inserted_continuations::{fn paragraphs}` (comparer/finalize.rs:22617); `deep_production_path_tests::final_deletion_peel_transfers_boundary_only_with_equal_properties_and_text_payload::{fn projected}` (comparer/finalize.rs:22683); `deep_production_path_tests::deletion_peel_keeps_nontext_only_original_anchors_in_their_source_paragraph::{fn projected}` (comparer/finalize.rs:22774); `deep_production_path_tests::deletion_peel_keeps_punctuation_and_whitespace_only_original_anchors::{fn projection}` (comparer/finalize.rs:22833); `deep_production_path_tests::deletion_peel_keeps_original_closing_boundary_when_all_original_words_are_deleted::{fn projection}` (comparer/finalize.rs:22880); `deep_production_path_tests::wholesale_deleted_body_peel_transfers_its_retained_closing_boundary::{fn projection}` (comparer/finalize.rs:22925); +39 more

#### `descendant_and_self_tags`  (L772-814, pub)
`pub fn descendant_and_self_tags(dom: &Dom, element: NodeId) -> Vec<Tag>`

- **Does:** A.0 — `DescendantAndSelfTags` (RevisionProcessor.cs :2397): stream `element` and its descendants as open / empty / close tags in document order.
- **Calls (crate):** `element_children_vec` (revision_processor.rs:817) L780,796; `Dom::child_count` (xmllinq/mod.rs:622) L785
- **Calls (std/external/unresolved):** `.last_mut`, `.last`, `.len`, `.pop`, `.push`
- **Macros:** `vec!`
- **Called by:** `drop_wholly_deleted_bookmarks` (revision_processor/bookmarks.rs:23); `move_from_range_deleted_elements` (revision_processor.rs:1146); `accept_deleted_and_moved_from_content_controls` (revision_processor.rs:1452)

#### `element_children_vec`  (L817-827)
`fn element_children_vec(dom: &Dom, id: NodeId) -> Vec<NodeId>`

- **Does:** Direct element children (document order) without the `elements()` filter path.
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L819; `Dom::child_at` (xmllinq/mod.rs:632) L821; `Dom::is_element` (xmllinq/mod.rs:502) L822
- **Calls (std/external/unresolved):** `.push`, `Vec::new`
- **Called by:** `descendant_and_self_tags` (revision_processor.rs:772); `iterate_block_content_elements` (revision_processor.rs:892)

#### `first_block_content`  (L847-877)
`fn first_block_content(dom: &Dom, roots: &[NodeId]) -> Option<NodeId>`

- **Does:** First paragraph, table, or opaque equation block among `roots`' descendants-and-self, document order.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L850; `Dom::child_count` (xmllinq/mod.rs:622) L860; `Dom::child_at` (xmllinq/mod.rs:632) L865; `Dom::is_element` (xmllinq/mod.rs:502) L867
- **Calls (std/external/unresolved):** `.is_some_and`, `.last_mut`, `.pop`, `.push`, `M::name`, `Some`, `W::p`, `W::tbl`, `is_block`
- **Macros:** `vec!`
- **Called by:** `iterate_block_content_elements` (revision_processor.rs:892)

#### `elements_after_self`  (L880-885)
`fn elements_after_self(dom: &Dom, id: NodeId) -> Vec<NodeId>`

- **Does:** Element siblings after `id`, document order.
- **Calls (crate):** `Dom::nodes_after_self` (xmllinq/mod.rs:838) L881; `Dom::is_element` (xmllinq/mod.rs:502) L883
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`
- **Called by:** `iterate_block_content_elements` (revision_processor.rs:892)

#### `iterate_block_content_elements`  (L892-928, pub)
`pub fn iterate_block_content_elements(dom: &Dom, element: NodeId) -> Vec<BlockContentInfo>`

- **Does:** A.0 — `IterateBlockContentElements` (:1909) + `AnnotateBlockContentElements` (:1855): the doc-order chain of paragraphs, tables, and equation blocks under `element`, linked prev/this/next.
- **Calls (crate):** `element_children_vec` (revision_processor.rs:817) L894; `first_block_content` (revision_processor.rs:847) L898,906; `elements_after_self` (revision_processor.rs:880) L906; `Dom::parent` (xmllinq/mod.rs:597) L910
- **Calls (std/external/unresolved):** `.collect`, `.copied`, `.get`, `.is_empty`, `.last`, `.len`, `.map`, `.push`, `.then`, `.unwrap`, `Some`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `accept_paragraph_marks_with_owners` (revision_processor.rs:1811); `body_equation_paragraph_mark_ownership_tests::equation_is_an_opaque_block_barrier_for_deleted_paragraph_marks` (revision_processor.rs:4694)

#### `block_level_content_containers`  (L931-941)
`fn block_level_content_containers() -> [XName; 7]`

- **Does:** `W.BlockLevelContentContainers` (PtOpenXmlUtil.cs :5569).
- **Calls (std/external/unresolved):** `W::body`, `W::endnote`, `W::footnote`, `W::ftr`, `W::hdr`, `W::tc`, `W::txbx_content`
- **Called by:** `get_paragraph_info` (revision_processor.rs:952); `accept_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1355); `accept_paragraph_marks_with_owners` (revision_processor.rs:1811)

#### `get_paragraph_info`  (L952-984, pub)
`pub fn get_paragraph_info(dom: &Dom, content_element: NodeId) -> BlockContentInfo`

- **Does:** A.0 — `GetParagraphInfo` (:2917) + `InitializeParagraphInfo` (:2890): for a child of a block-level content container, `this` = the first descendant-or-self in {`w:p`, `w:tc`, `w:txbxContent`} — nulled when that first hit is a `tc`/`txbxContent` (means "no own paragraph") — and `previous` = the previous element sibling...
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L953; `Dom::name` (xmllinq/mod.rs:526) L956,966,968; `block_level_content_containers` (revision_processor.rs:931) L958; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L963; `Dom::nodes_before_self` (xmllinq/mod.rs:846) L973; `Dom::is_element` (xmllinq/mod.rs:502) L977
- **Calls (std/external/unresolved):** `.contains`, `.expect`, `.find`, `.into_iter`, `.is_some_and`, `.rev`, `W::p`, `W::tc`, `W::txbx_content`
- **Macros:** `assert!`
- **Called by:** `accept_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1355)

#### `content_elements_before_self`  (L988-994, pub)
`pub fn content_elements_before_self(dom: &Dom, element: NodeId) -> Vec<NodeId>`

- **Does:** A.0 — `ContentElementsBeforeSelf` (:2926): previous element siblings, nearest first.
- **Calls (crate):** `Dom::nodes_before_self` (xmllinq/mod.rs:846) L989; `Dom::is_element` (xmllinq/mod.rs:502) L992
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.rev`
- **Called by:** `accept_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1355); `accept_deleted_cells_transform` (revision_processor.rs:2627)

#### `transform_instr_text_to_del_instr_text`  (L1000-1026)
`fn transform_instr_text_to_del_instr_text(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** A.1 — `TransformInstrTextToDelInstrText` (:1433): rename `w:instrText` to `w:delInstrText` (attrs + nodes carried as-is), rebuilding everything else.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L1001; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L1002,1012; `Dom::name` (xmllinq/mod.rs:526) L1004; `Dom::new_element` (xmllinq/mod.rs:478) L1006,1017; `Dom::attributes` (xmllinq/mod.rs:885) L1007,1018; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1008,1019; `Dom::nodes` (xmllinq/mod.rs:604) L1011,1021; `Dom::add` (xmllinq/mod.rs:1058) L1013,1023; `transform_instr_text_to_del_instr_text` (revision_processor.rs:1000) L1022
- **Calls (std/external/unresolved):** `.unwrap`, `Some`, `W::del_instr_text`, `W::instr_text`
- **Called by:** `transform_instr_text_to_del_instr_text` (revision_processor.rs:1000); `fix_up_deleted_or_inserted_field_codes_transform` (revision_processor.rs:1037)

#### `fix_up_deleted_or_inserted_field_codes_transform`  (L1037-1121, pub)
`pub fn fix_up_deleted_or_inserted_field_codes_transform(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** A.1 — `FixUpDeletedOrInsertedFieldCodesTransform` (:1354): inside each `w:p`, group-adjacent the child elements by kind — 2 = `w:del` holding `w:r/w:fldChar`, 3 = `w:ins` holding `w:r/w:fldChar`, 4 = `w:r` with `w:instrText`, 1 = other.
- **Calls (crate):** `element_or_desc_has_name` (revision_processor.rs:184) L1041; `Dom::is_element` (xmllinq/mod.rs:502) L1044; `Dom::name` (xmllinq/mod.rs:526) L1047,1050; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L1052; `Dom::element` (xmllinq/mod.rs:655) (by name) L1054; `Dom::element` (xmllinq/mod.rs:655) L1060; `Dom::elements` (xmllinq/mod.rs:640) L1066; `Dom::new_element` (xmllinq/mod.rs:478) L1070,1087,1094,1112; `Dom::attributes` (xmllinq/mod.rs:885) L1071,1113; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1072,1114; `fix_up_deleted_or_inserted_field_codes_transform` (revision_processor.rs:1037) L1078,1096,1102,1117; `Dom::add` (xmllinq/mod.rs:1058) L1079,1090,1092,1097,1099,1103...; `transform_instr_text_to_del_instr_text` (revision_processor.rs:1000) L1089; `Dom::nodes` (xmllinq/mod.rs:604) L1116
- **Calls (std/external/unresolved):** `.any`, `.enumerate`, `.into_iter`, `.is_some`, `.iter`, `.len`, `.unwrap`, `Some`, `W::del`, `W::fld_char`, `W::ins`, `W::instr_text`, `W::p`, `W::r`, `crate::util::group_adjacent`, `flanked_by`, `holds_fld_char`, `key_of`
- **Macros:** `unreachable!`
- **Called by:** `fix_up_deleted_or_inserted_field_codes_transform` (revision_processor.rs:1037); `accept_revisions_for_part_content` (revision_processor.rs:3252); `complete_field_instruction_metadata_boundary_tests::repaired_complete_field_codes_preserve_source_comments_processing_instructions_and_formats` (revision_processor.rs:4997)

#### `accept_move_from_ranges`  (L1134-1141, pub)
`pub fn accept_move_from_ranges(dom: &mut Dom, document: NodeId) -> NodeId`

- **Does:** A.2 — `AcceptMoveFromRanges` (:1530): walk the tag stream; while one or more `moveFromRangeStart` ids are open, record every other element's open tag (start-list) and close tag (end-list; empty elements land in both).
- **Calls (crate):** `move_from_range_deleted_elements` (revision_processor.rs:1146) L1135; `accept_move_from_ranges_transform` (revision_processor.rs:1238) L1139
- **Calls (std/external/unresolved):** `.expect`, `.is_empty`
- **Called by:** `accept_revisions_for_part_content` (revision_processor.rs:3252); `revision_boundary_coverage_tests::move_range_that_enters_table_keeps_container_properties_and_surviving_content` (revision_processor.rs:4011); `revision_boundary_coverage_tests::unmatched_or_contentless_move_ranges_preserve_identity_and_properties` (revision_processor.rs:4357)

#### `move_from_range_deleted_elements`  (L1146-1221)
`fn move_from_range_deleted_elements(dom: &Dom, document: NodeId) -> HashSet<NodeId>`

- **Does:** Elements wholly removed by completed move-from ranges.
- **Calls (crate):** `descendant_and_self_tags` (revision_processor.rs:772) L1157; `Dom::name` (xmllinq/mod.rs:526) L1158,1213; `Dom::attribute` (xmllinq/mod.rs:874) L1160,1168; `is_comment_or_bookmark_range_marker` (revision_processor.rs:2064) L1213; `is_container_property` (revision_processor.rs:1226) L1214,1218; `Dom::ancestors` (xmllinq/mod.rs:770) L1215
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.contains`, `.extend`, `.filter`, `.insert`, `.into_iter`, `.is_empty`, `.is_some_and`, `.push`, `.remove`, `.to_string`, `.unwrap_or`, `.unwrap`, `.values_mut`, `HashMap::new`, `Vec::new`, `W::id`, `W::move_from_range_end`, `W::move_from_range_start`
- **Called by:** `drop_wholly_deleted_bookmarks` (revision_processor/bookmarks.rs:23); `accept_move_from_ranges` (revision_processor.rs:1134)

#### `is_container_property`  (L1226-1234)
`fn is_container_property(dom: &Dom, e: NodeId) -> bool`

- **Does:** Is `e` the property element of its container (a paragraph's pPr, a table's tblPr or tblGrid, a row's trPr or tblPrEx, a cell's tcPr, a content control's sdtPr or sdtEndPr)?
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L1227; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1228; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1230
- **Calls (std/external/unresolved):** `.is_some_and`
- **Macros:** `matches!`
- **Called by:** `move_from_range_deleted_elements` (revision_processor.rs:1146)

#### `accept_move_from_ranges_transform`  (L1238-1265)
`fn accept_move_from_ranges_transform( dom: &mut Dom, node: NodeId, to_delete: &std::collections::HashSet<NodeId>, ) -> Option<NodeId>`

- **Does:** A.2 — `AcceptMoveFromRangesTransform` (:2629): rebuild, dropping the elements marked for deletion (and thereby their subtrees).
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L1243; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L1244; `Dom::new_element` (xmllinq/mod.rs:478) L1249; `Dom::name` (xmllinq/mod.rs:526) L1249; `Dom::attributes` (xmllinq/mod.rs:885) L1250; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1251; `Dom::nodes` (xmllinq/mod.rs:604) L1253; `accept_move_from_ranges_transform` (revision_processor.rs:1238) L1254; `Dom::add` (xmllinq/mod.rs:1058) L1255,1260; `hoist_range_markers_from` (revision_processor.rs:2074) L1259
- **Calls (std/external/unresolved):** `.contains`, `.unwrap`, `Some`
- **Called by:** `accept_move_from_ranges` (revision_processor.rs:1134); `accept_move_from_ranges_transform` (revision_processor.rs:1238)

#### `collapse_paragraph_transform`  (L1272-1295, pub)
`pub fn collapse_paragraph_transform(dom: &mut Dom, node: NodeId) -> Vec<NodeId>`

- **Does:** A.3 — `CollapseParagraphTransform` (:1821): a `w:p` collapses to its element children minus `w:pPr` (clones — C# XLinq re-parents by cloning); other elements rebuild recursively; non-elements pass through.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L1273; `Dom::clone_subtree` (xmllinq/mod.rs:1215) (by name) L1274; `Dom::name` (xmllinq/mod.rs:526) L1276,1281; `Dom::elements` (xmllinq/mod.rs:640) L1278; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L1283; `Dom::new_element` (xmllinq/mod.rs:478) L1285; `Dom::attributes` (xmllinq/mod.rs:885) L1286; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1287; `Dom::nodes` (xmllinq/mod.rs:604) L1289; `collapse_paragraph_transform` (revision_processor.rs:1272) L1290; `Dom::add` (xmllinq/mod.rs:1058) L1291
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.map`, `.unwrap`, `Some`, `W::p_pr`, `W::p`
- **Macros:** `vec!`
- **Called by:** `collapse_paragraph_transform` (revision_processor.rs:1272); `coalesque_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1303); `accept_paragraph_marks_with_owners` (revision_processor.rs:1811)

#### `coalesque_paragraph_end_tags_in_move_from_transform`  (L1303-1337, pub)
`pub fn coalesque_paragraph_end_tags_in_move_from_transform( dom: &mut Dom, node: NodeId, group: &[NodeId], ) -> NodeId`

- **Does:** A.3 — `CoalesqueParagraphEndTagsInMoveFromTransform` (:2645): produce the first group member, with its (first descendant) paragraph replaced by one carrying the paragraph's own children plus the COLLAPSED content of the group's subsequent members.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L1308; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L1309,1318; `Dom::name` (xmllinq/mod.rs:526) L1311; `Dom::new_element` (xmllinq/mod.rs:478) L1313,1328; `Dom::attributes` (xmllinq/mod.rs:885) L1314,1329; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1315,1330; `Dom::elements` (xmllinq/mod.rs:640) L1317; `Dom::add` (xmllinq/mod.rs:1058) L1319,1323,1334; `collapse_paragraph_transform` (revision_processor.rs:1272) L1322; `Dom::nodes` (xmllinq/mod.rs:604) L1332; `coalesque_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1303) L1333
- **Calls (std/external/unresolved):** `.iter`, `.skip`, `.unwrap`, `Some`, `W::p`
- **Called by:** `coalesque_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1303); `accept_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1355)

#### `accept_paragraph_end_tags_in_move_from_transform`  (L1355-1441, pub)
`pub fn accept_paragraph_end_tags_in_move_from_transform(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** A.3 — `AcceptParagraphEndTagsInMoveFromTransform` (:1610): group a content container's children by whether their paragraph mark sits in an OPEN moveFrom range (`moveFromRangeStart` child without `moveFromRangeEnd`), or they are a `w:p` directly following such a paragraph.
- **Calls (crate):** `element_or_desc_has_name` (revision_processor.rs:184) L1357; `Dom::is_element` (xmllinq/mod.rs:502) L1360; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L1361,1407; `Dom::name` (xmllinq/mod.rs:526) L1363,1385; `block_level_content_containers` (revision_processor.rs:931) L1364; `Dom::elements` (xmllinq/mod.rs:640) L1368,1393; `get_paragraph_info` (revision_processor.rs:952) L1372,1379,1384; `content_elements_before_self` (revision_processor.rs:988) L1378; `Dom::new_element` (xmllinq/mod.rs:478) L1400,1422,1432; `Dom::attributes` (xmllinq/mod.rs:885) L1401,1423,1433; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1402,1424,1434; `Dom::add` (xmllinq/mod.rs:1058) L1408,1415,1428,1438; `coalesque_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1303) L1414; `Dom::nodes` (xmllinq/mod.rs:604) L1426,1436; `accept_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1355) L1427,1437
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.into_iter`, `.is_empty`, `.is_some`, `.len`, `.unwrap`, `Some`, `W::move_from_range_end`, `W::move_from_range_start`, `W::p`, `crate::util::group_adjacent`, `key_of`, `mark_in_open_range`
- **Called by:** `accept_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1355); `accept_revisions_for_part_content` (revision_processor.rs:3252)

#### `accept_deleted_and_moved_from_content_controls`  (L1452-1571, pub)
`pub fn accept_deleted_and_moved_from_content_controls(dom: &mut Dom, root: NodeId) -> NodeId`

- **Does:** A.4 — `AcceptDeletedAndMovedFromContentControls` (:2491): walk the tag stream tracking two range kinds.
- **Calls (crate):** `descendant_and_self_tags` (revision_processor.rs:772) L1470; `Dom::name` (xmllinq/mod.rs:526) L1471; `Dom::attribute` (xmllinq/mod.rs:874) L1473,1481,1489,1497; `accept_deleted_and_moved_from_content_controls_transform` (revision_processor.rs:1576) L1563
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.contains`, `.extend`, `.filter`, `.insert`, `.into_iter`, `.is_empty`, `.len`, `.push`, `.remove`, `.to_string`, `.unwrap_or`, `.unwrap`, `.values_mut`, `HashMap::new`, `Vec::new`, `W::id`, `W::move_from_range_end`, `W::move_from_range_start`, `W::name`, `W::sdt`
- **Macros:** `debug_assert_eq!`
- **Called by:** `accept_revisions_for_part_content` (revision_processor.rs:3252); `revision_boundary_coverage_tests::custom_xml_deleted_control_unwraps_nested_controls_but_retains_order_and_anchors` (revision_processor.rs:3939); `revision_boundary_coverage_tests::custom_xml_move_deletes_whole_control_and_plain_content_but_keeps_outside_control` (revision_processor.rs:3970); `revision_boundary_coverage_tests::unmatched_custom_ranges_leave_control_payload_and_parent_links_untouched` (revision_processor.rs:3991)

#### `accept_deleted_and_moved_from_content_controls_transform`  (L1576-1616)
`fn accept_deleted_and_moved_from_content_controls_transform( dom: &mut Dom, node: NodeId, to_collapse: &std::collections::HashSet<NodeId>, to_delete: &std::collections::HashSet<NodeId>, ) -> Vec<NodeId>`

- **Does:** A.4 — `AcceptDeletedAndMovedFromContentControlsTransform` (:2468): splice a collapsed sdt's `sdtContent` child nodes (transformed) in its place, drop deleted elements, rebuild the rest.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L1582; `Dom::clone_subtree` (xmllinq/mod.rs:1215) (by name) L1583; `Dom::name` (xmllinq/mod.rs:526) L1585; `Dom::element` (xmllinq/mod.rs:655) L1587; `Dom::nodes` (xmllinq/mod.rs:604) L1591,1608; `accept_deleted_and_moved_from_content_controls_transform` (revision_processor.rs:1576) L1592,1610; `Dom::new_element` (xmllinq/mod.rs:478) L1604; `Dom::attributes` (xmllinq/mod.rs:885) L1605; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1606; `Dom::add` (xmllinq/mod.rs:1058) L1612
- **Calls (std/external/unresolved):** `.contains`, `.expect`, `.extend`, `.unwrap`, `Some`, `Vec::new`, `W::sdt_content`, `W::sdt`
- **Macros:** `vec!`
- **Called by:** `accept_deleted_and_moved_from_content_controls` (revision_processor.rs:1452); `accept_deleted_and_moved_from_content_controls_transform` (revision_processor.rs:1576)

#### `is_run_content`  (L1622-1665)
`fn is_run_content(name: &XName) -> Option<bool>`

- **Does:** A.5a — `IsRunContent` (:2356): `Some(true)` = run-level content, `Some(false)` = marker/non-content, `None` = unknown (C# throws).
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L1629,1634,1637; `XName::local_name` (xmllinq/mod.rs:208) L1635,1640
- **Calls (std/external/unresolved):** `.contains`, `Some`, `crate::namespaces::MC::name`
- **Called by:** `all_para_content_is_deleted` (revision_processor.rs:1717)

#### `collapse_transform`  (L1673-1709)
`fn collapse_transform(dom: &mut Dom, node: NodeId) -> Vec<NodeId>`

- **Does:** A.5a — `CollapseTransform` (:2331): splice `w:dir`/`w:bdr`/`w:ins`/ `w:moveTo`/`w:smartTag` to their element children and `w:sdt` to its `sdtContent` element children — ONE level, un-recursed, exactly like the C# (`return element.Elements()`); drop `w:pPr`; rebuild everything else recursively.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L1674; `Dom::clone_subtree` (xmllinq/mod.rs:1215) (by name) L1675; `Dom::name` (xmllinq/mod.rs:526) L1677; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1678; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1680; `Dom::elements` (xmllinq/mod.rs:640) L1684,1689,1690; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L1685,1691; `Dom::new_element` (xmllinq/mod.rs:478) L1699; `Dom::attributes` (xmllinq/mod.rs:885) L1700; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1701; `Dom::nodes` (xmllinq/mod.rs:604) L1703; `collapse_transform` (revision_processor.rs:1673) L1704; `Dom::add` (xmllinq/mod.rs:1058) L1705
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`, `.push`, `.unwrap`, `Some`, `Vec::new`, `W::p_pr`, `W::sdt_content`, `W::sdt`
- **Macros:** `matches!`, `vec!`
- **Called by:** `collapse_transform` (revision_processor.rs:1673); `all_para_content_is_deleted` (revision_processor.rs:1717)

#### `all_para_content_is_deleted`  (L1717-1743)
`fn all_para_content_is_deleted(dom: &mut Dom, p: NodeId) -> bool`

- **Does:** A.5a — `AllParaContentIsDeleted` (:2310): after collapsing wrappers, does the paragraph hold NO run-level content?
- **Calls (crate):** `collapse_transform` (revision_processor.rs:1673) L1718; `Dom::elements` (xmllinq/mod.rs:640) L1721; `Dom::name` (xmllinq/mod.rs:526) L1722,1733; `accept_all_other_revisions_transform` (revision_processor.rs:229) L1730; `is_run_content` (revision_processor.rs:1622) L1734,1740; `XName::clark` (xmllinq/mod.rs:226) (by name) L1735,1741
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_some_and`, `.len`, `.unwrap_or_else`, `.unwrap`, `M::name`
- **Macros:** `debug_assert_eq!`, `panic!`
- **Called by:** `accept_paragraph_marks_with_owners` (revision_processor.rs:1811)

#### `table_rows_all_deleted`  (L1746-1753)
`fn table_rows_all_deleted(dom: &Dom, tbl: NodeId) -> bool`

- **Does:** Does every row of `tbl` carry `trPr/w:del` (and is there a row)?
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1747; `Dom::element` (xmllinq/mod.rs:655) L1750,1751
- **Calls (std/external/unresolved):** `.all`, `.into_iter`, `.is_empty`, `.is_some_and`, `.is_some`, `Some`, `W::del`, `W::tr_pr`, `W::tr`
- **Called by:** `accept_paragraph_marks_with_owners` (revision_processor.rs:1811)

#### `paragraph_mark_is_deleted_or_moved_from`  (L1757-1767)
`fn paragraph_mark_is_deleted_or_moved_from(dom: &Dom, p: NodeId) -> bool`

- **Does:** `p / pPr / rPr / (del | moveFrom)` — is the paragraph mark deleted or moved from?
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1759,1760; `Dom::elements` (xmllinq/mod.rs:640) L1762; `Dom::name` (xmllinq/mod.rs:526) L1763
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.into_iter`, `.is_some_and`, `W::del`, `W::move_from`, `W::p_pr`, `W::r_pr`
- **Called by:** `drop_wholly_deleted_bookmarks` (revision_processor/bookmarks.rs:23); `carry_vanishing_section_references` (revision_processor/sections.rs:31); `has_deleted_or_moved_from_paragraph_mark::{fn walk}` (revision_processor.rs:1772); `accept_paragraph_marks_with_owners` (revision_processor.rs:1811)

#### `has_deleted_or_moved_from_paragraph_mark`  (L1771-1787)
`fn has_deleted_or_moved_from_paragraph_mark(dom: &Dom, root: NodeId) -> bool`

- **Does:** ACCEPT-SKIP-A5: non-allocating presence of any paragraph whose mark is deleted or moved-from under `root`.
- **Calls (crate):** `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:91) L1786
- **Called by:** `accept_deleted_and_move_from_paragraph_marks` (revision_processor.rs:2515); `body_equation_paragraph_mark_ownership_tests::public_accept_and_reject_preserve_unchanged_body_math_among_revised_marks` (revision_processor.rs:4711)

#### `has_deleted_or_moved_from_paragraph_mark::{fn walk}`  (L1772-1785)
`fn walk(dom: &Dom, id: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L1773; `paragraph_mark_is_deleted_or_moved_from` (revision_processor.rs:1757) L1774; `Dom::child_count` (xmllinq/mod.rs:622) L1778; `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:91) L1780; `Dom::child_at` (xmllinq/mod.rs:632) L1780
- **Calls (std/external/unresolved):** `.as_ref`, `Some`, `W::p`
- **Called by:** `element_has_tracked_revisions` (revision_processor.rs:90); `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:91); `element_or_desc_has_name` (revision_processor.rs:184); `element_or_desc_has_name::{fn walk}` (revision_processor.rs:185); `has_deleted_or_moved_from_paragraph_mark` (revision_processor.rs:1771); `has_deleted_or_moved_from_paragraph_mark::{fn walk}` (revision_processor.rs:1772); `subtree_needs_adjacent_table_merge` (revision_processor.rs:2909); `subtree_needs_adjacent_table_merge::{fn walk}` (revision_processor.rs:2910); `has_empty_table_cell` (revision_processor.rs:3159); `has_empty_table_cell::{fn walk}` (revision_processor.rs:3162)

#### `accept_deleted_and_move_from_paragraph_marks_transform`  (L1802-1807, pub)
`pub fn accept_deleted_and_move_from_paragraph_marks_transform( dom: &mut Dom, node: NodeId, ) -> NodeId`

- **Does:** A.5a — `AcceptDeletedAndMoveFromParagraphMarksTransform` (:2119): the 3-state grouping machine over a container's block-content chain.
- **Calls (crate):** `accept_paragraph_marks_with_owners` (revision_processor.rs:1811) L1806
- **Calls (std/external/unresolved):** `HashMap::new`
- **Called by:** `body_equation_paragraph_mark_ownership_tests::equation_is_an_opaque_block_barrier_for_deleted_paragraph_marks` (revision_processor.rs:4694); `deleted_title_equation_barrier_tests::deleted_title_before_equation_is_removed_without_consuming_the_equation` (revision_processor.rs:4919)

#### `accept_paragraph_marks_with_owners`  (L1811-2048)
`fn accept_paragraph_marks_with_owners( dom: &mut Dom, node: NodeId, paragraph_owners: &mut HashMap<NodeId, Vec<NodeId>>, ) -> NodeId`

- **Does:** Original paragraph owners are private reconstruction evidence. _(from a code comment)_
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L1816; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L1817,1912,1985,2001,2044; `Dom::name` (xmllinq/mod.rs:526) L1819,1848,1880,1919,1944,1963...; `block_level_content_containers` (revision_processor.rs:931) L1820; `Dom::new_element` (xmllinq/mod.rs:478) L1821,1910,1964,1980; `Dom::attributes` (xmllinq/mod.rs:885) L1822,1965,1981; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1823,1966,1982; `Dom::nodes` (xmllinq/mod.rs:604) L1825,1968; `accept_paragraph_marks_with_owners` (revision_processor.rs:1811) L1826,1969; `Dom::add` (xmllinq/mod.rs:1058) L1827,1913,1924,1970,1986,2002...; `Dom::name_is` (xmllinq/mod.rs:535) L1829,1932,1972; `Dom::element` (xmllinq/mod.rs:655) L1836; `iterate_block_content_elements` (revision_processor.rs:892) L1841; `paragraph_mark_is_deleted_or_moved_from` (revision_processor.rs:1757) L1850,1937; `table_rows_all_deleted` (revision_processor.rs:1746) L1881; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1887; `Dom::elements` (xmllinq/mod.rs:640) L1911,1984,1993; `collapse_paragraph_transform` (revision_processor.rs:1272) L1923; `all_para_content_is_deleted` (revision_processor.rs:1717) L1948; `hoist_range_markers_from` (revision_processor.rs:2074) L1955; `is_body_level_range_marker` (revision_processor.rs:2051) L2000; `Dom::ancestors` (xmllinq/mod.rs:770) L2030
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.collect`, `.contains`, `.copied`, `.enumerate`, `.filter`, `.insert`, `.into_iter`, `.is_none`, `.is_some_and`, `.iter`, `.last`, `.len`, `.position`, `.push`, `.unwrap`, `.zip`, `HashSet::new`, `M::name`, `Some`, `Vec::new`, `Vec::with_capacity`, `W::body`, `W::p_pr`, `W::p`, `W::sect_pr`, `W::tbl`, `W::tc_pr`, `crate::util::group_adjacent`
- **Macros:** `vec!`
- **Called by:** `accept_deleted_and_move_from_paragraph_marks_transform` (revision_processor.rs:1802); `accept_paragraph_marks_with_owners` (revision_processor.rs:1811); `accept_deleted_and_move_from_paragraph_marks` (revision_processor.rs:2515)

#### `is_body_level_range_marker`  (L2051-2062)
`fn is_body_level_range_marker(name: &XName) -> bool`

- **Does:** Body/cell-level markers that must survive block-content rebuild (not p/tbl).
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L2052; `XName::local_name` (xmllinq/mod.rs:208) (by name) L2054
- **Macros:** `matches!`
- **Called by:** `accept_paragraph_marks_with_owners` (revision_processor.rs:1811)

#### `is_comment_or_bookmark_range_marker`  (L2064-2070)
`fn is_comment_or_bookmark_range_marker(name: &XName) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L2065; `XName::local_name` (xmllinq/mod.rs:208) (by name) L2067
- **Macros:** `matches!`
- **Called by:** `move_from_range_deleted_elements` (revision_processor.rs:1146); `hoist_range_markers_from` (revision_processor.rs:2074)

#### `hoist_range_markers_from`  (L2074-2085)
`fn hoist_range_markers_from(dom: &mut Dom, node: NodeId) -> Vec<NodeId>`

- **Does:** Pull comment range and bookmark markers out of a subtree being discarded (e.g.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2076; `Dom::name` (xmllinq/mod.rs:526) L2077; `is_comment_or_bookmark_range_marker` (revision_processor.rs:2064) L2080; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L2081
- **Calls (std/external/unresolved):** `.push`, `Vec::new`
- **Called by:** `accept_move_from_move_to_transform` (revision_processor.rs:129); `accept_all_other_revisions_transform` (revision_processor.rs:229); `accept_move_from_ranges_transform` (revision_processor.rs:1238); `accept_paragraph_marks_with_owners` (revision_processor.rs:1811)

#### `annotate_run_elements_with_id`  (L2091-2096, pub)
`pub fn annotate_run_elements_with_id(dom: &mut Dom, element: NodeId)`

- **Does:** A.5b — `AnnotateRunElementsWithId` (:1935): number every descendant `w:r` with `pt:UniqueId` 0..
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2093; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L2094
- **Calls (std/external/unresolved):** `.to_string`, `.zip`, `PT::unique_id`, `Some`, `W::r`
- **Called by:** `accept_deleted_and_move_from_paragraph_marks` (revision_processor.rs:2515)

#### `annotate_paragraph_elements_with_id`  (L2103-2108)
`fn annotate_paragraph_elements_with_id(dom: &mut Dom, element: NodeId)`

- **Does:** Number every descendant `w:p` (`pt:UniqueId` `p0`, `p1`, …) so A.5b can find a control's paragraphs when it has no run to anchor on (an empty control keeps its paragraph mark, and so its place).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2105; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L2106
- **Calls (std/external/unresolved):** `.zip`, `PT::unique_id`, `Some`, `W::p`
- **Macros:** `format!`
- **Called by:** `accept_deleted_and_move_from_paragraph_marks` (revision_processor.rs:2515)

#### `descendants_trimmed`  (L2112-2124)
`fn descendants_trimmed(dom: &Dom, element: NodeId, trim: &XName) -> Vec<NodeId>`

- **Does:** Descendants of `element` in document order, not descending INTO elements named `trim` (the `DescendantsTrimmed` helper the annotators use).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L2114,2118; `Dom::name` (xmllinq/mod.rs:526) L2117
- **Calls (std/external/unresolved):** `.as_ref`, `.collect`, `.into_iter`, `.pop`, `.push`, `.rev`, `Some`, `Vec::new`
- **Called by:** `annotate_content_controls_with_run_ids` (revision_processor.rs:2129)

#### `annotate_content_controls_with_run_ids`  (L2129-2142, pub)
`pub fn annotate_content_controls_with_run_ids(dom: &mut Dom, element: NodeId)`

- **Does:** A.5b — `AnnotateContentControlsWithRunIds` (:1945): give every descendant `w:sdt` a `pt:RunIds` (comma-joined `pt:UniqueId`s of its runs, trimmed at `w:txbxContent`) and its own `pt:UniqueId` (in place).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2133; `descendants_trimmed` (revision_processor.rs:2112) L2134; `Dom::name` (xmllinq/mod.rs:526) L2136; `Dom::attribute` (xmllinq/mod.rs:874) L2137; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L2139,2140
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.filter`, `.into_iter`, `.join`, `.map`, `.to_string`, `.zip`, `PT::run_ids`, `PT::unique_id`, `Some`, `W::r`, `W::sdt`, `W::txbx_content`
- **Called by:** `accept_deleted_and_move_from_paragraph_marks` (revision_processor.rs:2515)

#### `order_sdt`  (L2145-2157)
`fn order_sdt(name: &XName) -> i32`

- **Does:** `Order_sdt` (:2090): schema order for rebuilt `w:sdt` children.
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L2146; `XName::local_name` (xmllinq/mod.rs:208) L2149
- **Called by:** `add_content_controls_with_owners` (revision_processor.rs:2182)

#### `add_block_level_content_controls`  (L2174-2180, pub)
`pub fn add_block_level_content_controls( dom: &mut Dom, new_document: NodeId, original: NodeId, ) -> NodeId`

- **Does:** A.5b — `AddBlockLevelContentControls` (:1964): re-create the `w:sdt` wrappers the paragraph-mark transform stripped.
- **Calls (crate):** `add_content_controls_with_owners` (revision_processor.rs:2182) L2179
- **Called by:** none found in src

#### `add_content_controls_with_owners`  (L2182-2506)
`fn add_content_controls_with_owners( dom: &mut Dom, new_document: NodeId, original: NodeId, paragraph_owners: Option<&HashMap<NodeId, Vec<NodeId>>>, ) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2194,2195,2207,2214,2231,2248...; `Dom::attribute` (xmllinq/mod.rs:874) L2198,2208,2215,2221,2225,2234...; `Dom::element` (xmllinq/mod.rs:655) L2252,2253,2255,2256,2279,2316; `Dom::ancestors` (xmllinq/mod.rs:770) L2263,2282,2291,2352,2358,2401...; `Dom::name` (xmllinq/mod.rs:526) L2267,2286,2300,2341,2351,2356...; `fragmented_source_label_boundary_tests::wrapper` (comparer/finalize.rs:33494) (by name) L2286,2300; `Dom::parent` (xmllinq/mod.rs:597) L2297,2419; `Dom::ancestors_and_self` (xmllinq/mod.rs:787) L2328,2417; `Dom::elements` (xmllinq/mod.rs:640) L2426,2456,2468,2486; `Dom::new_element` (xmllinq/mod.rs:478) L2452,2478,2482; `Dom::attributes` (xmllinq/mod.rs:885) L2453,2479; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L2454,2480; `order_sdt` (revision_processor.rs:2145) L2457,2493; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L2459,2491; `Dom::add` (xmllinq/mod.rs:1058) L2460,2484,2495,2498,2500,2502; `Dom::add_before_self` (xmllinq/mod.rs:1114) L2462; `Dom::remove` (xmllinq/mod.rs:1102) L2463,2476
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.as_ref`, `.collect`, `.contains`, `.copied`, `.count`, `.entry`, `.expect`, `.filter_map`, `.filter`, `.find`, `.first`, `.get`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_some_and`, `.is_some`, `.iter`, `.last`, `.map`, `.nth`, `.or_insert`, `.position`, `.push`, `.retain`, `.reverse`, `.sort_by_key`, `.split`, `.take_while`, `.to_string`, `.to_vec`, `.unwrap_or`, `.unwrap`, `HashMap::new`, `PT::run_ids`, `PT::unique_id`, `Some`, `Vec::new`, `W::del`, `W::ins`, `W::move_from`, `W::move_to`, `W::name`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `W::sdt_content`, `W::sdt`, `W::tbl`, `child_containing`
- **Called by:** `add_block_level_content_controls` (revision_processor.rs:2174); `accept_deleted_and_move_from_paragraph_marks` (revision_processor.rs:2515)

#### `accept_deleted_and_move_from_paragraph_marks`  (L2515-2526, pub)
`pub fn accept_deleted_and_move_from_paragraph_marks(dom: &mut Dom, element: NodeId) -> NodeId`

- **Does:** A.5b — `AcceptDeletedAndMoveFromParagraphMarks` (:2098): annotate runs and content controls (on the ORIGINAL, in place), run the A.5a transform, then re-wrap the content controls the transform stripped.
- **Calls (crate):** `has_deleted_or_moved_from_paragraph_mark` (revision_processor.rs:1771) L2517; `annotate_run_elements_with_id` (revision_processor.rs:2091) L2520; `annotate_paragraph_elements_with_id` (revision_processor.rs:2103) L2521; `annotate_content_controls_with_run_ids` (revision_processor.rs:2129) L2522; `accept_paragraph_marks_with_owners` (revision_processor.rs:1811) L2524; `add_content_controls_with_owners` (revision_processor.rs:2182) L2525
- **Calls (std/external/unresolved):** `HashMap::new`, `Some`
- **Called by:** `accept_revisions_for_part_content` (revision_processor.rs:3252)

#### `a6_block_level_elements`  (L2532-2543)
`fn a6_block_level_elements() -> [XName; 8]`

- **Does:** `BlockLevelElements` (:2766) — the direct-cell-child names that make a cell non-empty for [`remove_rows_left_empty_by_move_from`].
- **Calls (std/external/unresolved):** `M::name`, `W::del`, `W::ins`, `W::move_to`, `W::p`, `W::sdt`, `W::tbl`
- **Called by:** `remove_rows_left_empty_by_move_from_inner` (revision_processor.rs:2553)

#### `remove_rows_left_empty_by_move_from`  (L2549-2551, pub)
`pub fn remove_rows_left_empty_by_move_from(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** A.6 — `RemoveRowsLeftEmptyByMoveFrom` (:2777): drop every `w:tr` whose cells all lost their block-level content to an accepted moveFrom; rebuild everything else.
- **Calls (crate):** `remove_rows_left_empty_by_move_from_inner` (revision_processor.rs:2553) L2550
- **Calls (std/external/unresolved):** `.expect`
- **Called by:** `accept_revisions_for_part_content` (revision_processor.rs:3252)

#### `remove_rows_left_empty_by_move_from_inner`  (L2553-2586)
`fn remove_rows_left_empty_by_move_from_inner(dom: &mut Dom, node: NodeId) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L2554; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L2555; `Dom::name` (xmllinq/mod.rs:526) L2557,2563; `a6_block_level_elements` (revision_processor.rs:2532) L2559; `Dom::elements` (xmllinq/mod.rs:640) L2560,2561; `Dom::element` (xmllinq/mod.rs:655) L2569,2582; `Dom::new_element` (xmllinq/mod.rs:478) L2570; `Dom::attributes` (xmllinq/mod.rs:885) L2571; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L2572; `Dom::nodes` (xmllinq/mod.rs:604) L2574; `remove_rows_left_empty_by_move_from_inner` (revision_processor.rs:2553) L2575; `Dom::add` (xmllinq/mod.rs:1058) L2576
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.into_iter`, `.is_none`, `.is_some_and`, `.is_some`, `.unwrap`, `Some`, `W::tbl`, `W::tc`, `W::tr`
- **Called by:** `remove_rows_left_empty_by_move_from` (revision_processor.rs:2549); `remove_rows_left_empty_by_move_from_inner` (revision_processor.rs:2553)

#### `order_tc_pr`  (L2591-2612)
`fn order_tc_pr(name: &XName) -> i32`

- **Does:** `Order_tcPr` (:1466): schema order for rebuilt `w:tcPr` children.
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L2592; `XName::local_name` (xmllinq/mod.rs:208) L2595
- **Called by:** `accept_deleted_cells_transform` (revision_processor.rs:2627); `merge_adjacent_tables` (revision_processor.rs:2971)

#### `accept_deleted_cells_transform`  (L2627-2731, pub)
`pub fn accept_deleted_cells_transform(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** A.7 — `AcceptDeletedCellsTransform` (:2674): inside each `w:tr`, group adjacent children by (deleted-cell?, anchor), where a cell is "deleted" when it carries `w:cellDel` OR the next `w:tc` after it does, and the anchor is the nearest preceding (or self) `w:tc` WITHOUT `cellDel`.
- **Calls (crate):** `element_or_desc_has_name` (revision_processor.rs:184) L2630; `Dom::is_element` (xmllinq/mod.rs:502) L2633; `Dom::name` (xmllinq/mod.rs:526) L2636,2659,2661,2664,2706,2711...; `Dom::new_element` (xmllinq/mod.rs:478) L2638,2671,2700,2713,2717; `Dom::attributes` (xmllinq/mod.rs:885) L2639,2672; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L2640,2673,2701; `Dom::nodes` (xmllinq/mod.rs:604) L2642; `accept_deleted_cells_transform` (revision_processor.rs:2627) L2643; `Dom::add` (xmllinq/mod.rs:1058) L2644,2679,2715,2718,2726,2728; `Dom::descendants` (xmllinq/mod.rs:668) L2651; `Dom::elements` (xmllinq/mod.rs:640) L2653,2703,2719; `Dom::nodes_after_self` (xmllinq/mod.rs:838) L2656; `content_elements_before_self` (revision_processor.rs:988) L2663; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L2678,2709,2725; `Dom::element` (xmllinq/mod.rs:655) L2690,2693; `Dom::attribute` (xmllinq/mod.rs:874) L2695; `order_tc_pr` (revision_processor.rs:2591) L2711
- **Calls (std/external/unresolved):** `.and_then`, `.chain`, `.clone`, `.collect`, `.expect`, `.filter`, `.find`, `.into_iter`, `.is_empty`, `.is_some_and`, `.len`, `.ok`, `.parse`, `.push`, `.sort_by_key`, `.to_string`, `.unwrap_or`, `.unwrap`, `Some`, `W::cell_del`, `W::grid_span`, `W::tc_pr`, `W::tc`, `W::tr`, `W::val`, `crate::util::group_adjacent`, `has_cell_del`, `key_of`, `std::iter::once`
- **Macros:** `vec!`
- **Called by:** `accept_deleted_cells_transform` (revision_processor.rs:2627); `accept_revisions_for_part_content` (revision_processor.rs:3252); `revision_boundary_coverage_tests::accepting_deleted_cells_keeps_anchor_payload_and_expands_existing_span` (revision_processor.rs:3848); `revision_boundary_coverage_tests::deleted_cell_anchor_defaults_bad_span_and_orders_all_schema_properties` (revision_processor.rs:3884)

#### `own_tbl_pr_ex`  (L2737-2756)
`fn own_tbl_pr_ex(dom: &Dom, first: NodeId, member: NodeId) -> Vec<NodeId>`

- **Does:** R8: the `tblPrEx`-able properties of `member` that `first` (whose `tblPr` the merged table keeps) does not share.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L2739; `Dom::elements` (xmllinq/mod.rs:640) L2741; `tbl_pr_ex_rank` (revision_processor.rs:2860) L2743; `Dom::serialize_element` (xmllinq/mod.rs:1299) L2750,2754
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter`, `.into_iter`, `.is_some`, `.map`, `.unwrap_or_default`, `W::tbl_pr`, `ex_of`
- **Called by:** `merge_adjacent_tables` (revision_processor.rs:2971)

#### `row_tbl_pr_ex`  (L2760-2778)
`fn row_tbl_pr_ex(dom: &mut Dom, tr: NodeId, own: &[NodeId]) -> NodeId`

- **Does:** R8: row `tr`'s `tblPrEx` with its table's `own` properties under the row's own, in schema order.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L2761; `Dom::elements` (xmllinq/mod.rs:640) L2763; `Dom::name` (xmllinq/mod.rs:526) L2767; `tbl_pr_ex_rank` (revision_processor.rs:2860) L2771; `Dom::new_element` (xmllinq/mod.rs:478) L2772; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L2774; `Dom::add` (xmllinq/mod.rs:1058) L2775
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.iter`, `.len`, `.map`, `.push`, `.sort_by_key`, `.unwrap_or_default`, `.unwrap_or`, `W::name`
- **Called by:** `merge_adjacent_tables` (revision_processor.rs:2971)

#### `fix_widths`  (L2784-2823)
`fn fix_widths(dom: &mut Dom, tbl: NodeId) -> NodeId`

- **Does:** A.8 — `FixWidths` (:1484): clone the table and rewrite each `w:tcW`'s `w:w` to the sum of the grid columns its cell spans (per the ORIGINAL table's `tblGrid`).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L2785,2788,2796,2798; `Dom::attribute` (xmllinq/mod.rs:874) L2790,2803,2808; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L2795; `Dom::element` (xmllinq/mod.rs:655) L2800,2802,2805,2807; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L2818
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.enumerate`, `.expect`, `.filter`, `.flat_map`, `.into_iter`, `.is_some`, `.iter`, `.len`, `.map`, `.ok`, `.parse`, `.sum`, `.to_string`, `.unwrap_or`, `Some`, `W::grid_span`, `W::name`, `W::tc_pr`, `W::tc`, `W::tr`, `W::val`, `std::cmp::min`
- **Called by:** `merge_adjacent_tables` (revision_processor.rs:2971); `revision_boundary_coverage_tests::fix_widths_clamps_spans_and_leaves_widthless_cells_out_of_cursor` (revision_processor.rs:3700)

#### `table_has_revision_marks`  (L2827-2834)
`fn table_has_revision_marks(dom: &Dom, tbl: NodeId) -> bool`

- **Does:** True when a table subtree carries row/cell/run revision marks.
- **Calls (crate):** `element_or_desc_has_name` (revision_processor.rs:184) L2829
- **Calls (std/external/unresolved):** `W::name`
- **Called by:** `subtree_needs_adjacent_table_merge::{fn walk}` (revision_processor.rs:2910); `merge_adjacent_tables` (revision_processor.rs:2971)

#### `tbl_pr_ex_rank`  (L2860-2865)
`fn tbl_pr_ex_rank(dom: &Dom, e: NodeId) -> Option<usize>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L2861; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L2862; `XName::local_name` (xmllinq/mod.rs:208) (by name) L2863
- **Calls (std/external/unresolved):** `.flatten`, `.iter`, `.position`, `.then`
- **Called by:** `own_tbl_pr_ex` (revision_processor.rs:2737); `row_tbl_pr_ex` (revision_processor.rs:2760); `whole_table_key` (revision_processor.rs:2871)

#### `whole_table_key`  (L2871-2886)
`fn whole_table_key(dom: &Dom, tbl: NodeId) -> String`

- **Does:** A table's whole-table properties (its `tblPr` minus what `tblPrEx` can carry), serialized: Word joins adjacent tables whose keys agree.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L2873; `Dom::elements` (xmllinq/mod.rs:640) L2874,2880,2881; `tbl_pr_ex_rank` (revision_processor.rs:2860) L2875; `Dom::serialize_element` (xmllinq/mod.rs:1299) L2876,2882
- **Calls (std/external/unresolved):** `.is_none`, `.push_str`, `Some`, `String::from`, `W::name`, `W::tbl_pr`
- **Called by:** `table_merge_key` (revision_processor.rs:2889)

#### `table_merge_key`  (L2889-2905)
`fn table_merge_key(dom: &Dom, e: NodeId, mode: TableMerge) -> String`

- **Does:** The grouping key of A.8 for `e` (empty: not a table).
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L2890; `whole_table_key` (revision_processor.rs:2871) L2894; `Dom::elements` (xmllinq/mod.rs:640) L2896; `Dom::element` (xmllinq/mod.rs:655) L2899
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_some`, `.to_string`, `Some`, `String::new`, `W::name`, `W::tbl_pr`, `W::tbl`
- **Called by:** `subtree_needs_adjacent_table_merge::{fn walk}` (revision_processor.rs:2910); `merge_adjacent_tables` (revision_processor.rs:2971)

#### `subtree_needs_adjacent_table_merge`  (L2909-2941)
`fn subtree_needs_adjacent_table_merge(dom: &Dom, root: NodeId, mode: TableMerge) -> bool`

- **Does:** True if any element under `root` has ≥2 adjacent direct `w:tbl` children that A.8 merges in `mode`.
- **Calls (crate):** `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:91) L2940
- **Called by:** `merge_adjacent_tables` (revision_processor.rs:2971)

#### `subtree_needs_adjacent_table_merge::{fn walk}`  (L2910-2939)
`fn walk(dom: &Dom, id: NodeId, mode: TableMerge) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L2911; `Dom::elements` (xmllinq/mod.rs:640) L2915; `table_merge_key` (revision_processor.rs:2889) L2919; `table_has_revision_marks` (revision_processor.rs:2827) L2930; `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:91) L2938
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_empty`, `String::new`
- **Called by:** `element_has_tracked_revisions` (revision_processor.rs:90); `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:91); `element_or_desc_has_name` (revision_processor.rs:184); `element_or_desc_has_name::{fn walk}` (revision_processor.rs:185); `has_deleted_or_moved_from_paragraph_mark` (revision_processor.rs:1771); `has_deleted_or_moved_from_paragraph_mark::{fn walk}` (revision_processor.rs:1772); `subtree_needs_adjacent_table_merge` (revision_processor.rs:2909); `subtree_needs_adjacent_table_merge::{fn walk}` (revision_processor.rs:2910); `has_empty_table_cell` (revision_processor.rs:3159); `has_empty_table_cell::{fn walk}` (revision_processor.rs:3162)

#### `merge_adjacent_tables_transform`  (L2959-2961, pub)
`pub fn merge_adjacent_tables_transform(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** A.8 — `MergeAdjacentTablesTransform` (:464): where an element has direct `w:tbl` children, merge each run of ≥2 adjacent tables sharing the same bidiVisual-kind into one table: `tblPr` from the first, `tblGrid` = the diffs of the union of every member's cumulative grid widths, and each member's rows re-fit (`FixWidths...
- **Calls (crate):** `merge_adjacent_tables` (revision_processor.rs:2971) L2960
- **Called by:** `accept_revisions_for_part_content` (revision_processor.rs:3252); `revision_boundary_coverage_tests::revised_table_union_preserves_rows_payload_and_unrelated_clean_groups` (revision_processor.rs:3634); `revision_boundary_coverage_tests::nested_merge_clones_widthless_cell_and_keeps_control_identity` (revision_processor.rs:3741); `revision_boundary_coverage_tests::table_merge_identity_respects_whole_properties_grid_history_and_bidi` (revision_processor.rs:3806)

#### `merge_adjacent_tables_like_word`  (L2967-2969)
`fn merge_adjacent_tables_like_word(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** R8 — Word's save after Accept / Reject All: adjacent tables alike in every whole-table property are one table in Word's model (72cc9f4ac6, 3d4318d7e9); a later member's own `tblPrEx`-able properties ride on its rows.
- **Calls (crate):** `merge_adjacent_tables` (revision_processor.rs:2971) L2968
- **Called by:** `tidy` (revision_processor/word_save.rs:25); `revision_boundary_coverage_tests::word_save_merges_row_exceptions_with_row_values_taking_precedence` (revision_processor.rs:3765); `revision_boundary_coverage_tests::table_merge_identity_respects_whole_properties_grid_history_and_bidi` (revision_processor.rs:3806)

#### `merge_adjacent_tables`  (L2971-3153)
`fn merge_adjacent_tables(dom: &mut Dom, node: NodeId, mode: TableMerge) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `subtree_needs_adjacent_table_merge` (revision_processor.rs:2909) L2973; `Dom::is_element` (xmllinq/mod.rs:502) L2976; `Dom::name` (xmllinq/mod.rs:526) L2979,3068,3069,3119,3129,3139; `Dom::element` (xmllinq/mod.rs:655) L2981,3077,3079,3092,3094; `Dom::new_element` (xmllinq/mod.rs:478) L2982,2996,3034,3039,3042,3056...; `Dom::attributes` (xmllinq/mod.rs:885) L2983,2997,3057; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L2984,2998,3043,3058,3126; `Dom::nodes` (xmllinq/mod.rs:604) L2986; `merge_adjacent_tables` (revision_processor.rs:2971) L2987; `Dom::add` (xmllinq/mod.rs:1058) L2988,3004,3012,3037,3044,3046...; `Dom::elements` (xmllinq/mod.rs:640) L2993,3020,3021,3035,3055,3064...; `table_merge_key` (revision_processor.rs:2889) L2994; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L3003,3011,3036,3073,3083,3104...; `table_has_revision_marks` (revision_processor.rs:2827) L3009; `Dom::attribute` (xmllinq/mod.rs:874) L3022,3080,3095; `own_tbl_pr_ex` (revision_processor.rs:2737) L3050; `fix_widths` (revision_processor.rs:2784) L3054; `row_tbl_pr_ex` (revision_processor.rs:2760) L3061; `order_tc_pr` (revision_processor.rs:2591) L3129
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.chain`, `.clone`, `.collect`, `.copied`, `.count`, `.dedup`, `.enumerate`, `.expect`, `.filter`, `.flat_map`, `.into_iter`, `.is_empty`, `.is_none`, `.iter`, `.len`, `.ok`, `.parse`, `.position`, `.push`, `.sort_by_key`, `.sort_unstable`, `.take_while`, `.to_string`, `.unwrap_or`, `.unwrap`, `Some`, `Vec::new`, `W::grid_span`, `W::name`, `W::tbl_pr`, `W::tbl`, `W::tc_pr`, `W::tc`, `W::tr`, `W::val`, `crate::util::group_adjacent`, `std::iter::once`
- **Called by:** `merge_adjacent_tables_transform` (revision_processor.rs:2959); `merge_adjacent_tables_like_word` (revision_processor.rs:2967); `merge_adjacent_tables` (revision_processor.rs:2971)

#### `has_empty_table_cell`  (L3159-3187)
`fn has_empty_table_cell(dom: &Dom, root: NodeId) -> bool`

- **Does:** True if any `w:tc` under `root` has no element children other than `w:tcPr` (the A.9 empty-cell predicate).
- **Calls (crate):** `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:91) L3186
- **Calls (std/external/unresolved):** `W::tc_pr`, `W::tc`
- **Called by:** `accept_revisions_for_part_content` (revision_processor.rs:3252)

#### `has_empty_table_cell::{fn walk}`  (L3162-3185)
`fn walk(dom: &Dom, id: NodeId, tc: &XName, tcpr: &XName) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L3163,3170; `Dom::child_count` (xmllinq/mod.rs:622) L3168,3179; `Dom::child_at` (xmllinq/mod.rs:632) L3169,3180; `Dom::is_element` (xmllinq/mod.rs:502) L3170; `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:91) L3180
- **Calls (std/external/unresolved):** `.is_some_and`
- **Called by:** `element_has_tracked_revisions` (revision_processor.rs:90); `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:91); `element_or_desc_has_name` (revision_processor.rs:184); `element_or_desc_has_name::{fn walk}` (revision_processor.rs:185); `has_deleted_or_moved_from_paragraph_mark` (revision_processor.rs:1771); `has_deleted_or_moved_from_paragraph_mark::{fn walk}` (revision_processor.rs:1772); `subtree_needs_adjacent_table_merge` (revision_processor.rs:2909); `subtree_needs_adjacent_table_merge::{fn walk}` (revision_processor.rs:2910); `has_empty_table_cell` (revision_processor.rs:3159); `has_empty_table_cell::{fn walk}` (revision_processor.rs:3162)

#### `add_empty_paragraph_to_any_empty_cells`  (L3195-3209, pub)
`pub fn add_empty_paragraph_to_any_empty_cells(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** A.9 — `AddEmptyParagraphToAnyEmptyCells` (:1448): a `w:tc` with no element children other than `w:tcPr` gains an empty `w:p`.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L3196; `collect_empty_table_cells` (revision_processor.rs:3211) L3203; `Dom::new_element` (xmllinq/mod.rs:478) L3205; `Dom::add` (xmllinq/mod.rs:1058) L3206
- **Calls (std/external/unresolved):** `Vec::new`, `W::p`, `W::tc_pr`, `W::tc`
- **Called by:** `accept_revisions_for_part_content` (revision_processor.rs:3252)

#### `collect_empty_table_cells`  (L3211-3239)
`fn collect_empty_table_cells( dom: &Dom, id: NodeId, tc: &XName, tcpr: &XName, out: &mut Vec<NodeId>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L3218,3224; `Dom::child_count` (xmllinq/mod.rs:622) L3222,3233; `Dom::child_at` (xmllinq/mod.rs:632) L3223,3234; `Dom::is_element` (xmllinq/mod.rs:502) L3224,3235; `collect_empty_table_cells` (revision_processor.rs:3211) L3236
- **Calls (std/external/unresolved):** `.is_some_and`, `.push`
- **Called by:** `add_empty_paragraph_to_any_empty_cells` (revision_processor.rs:3195); `collect_empty_table_cells` (revision_processor.rs:3211)

#### `accept_revisions_for_part_content`  (L3252-3307, pub)
`pub fn accept_revisions_for_part_content(dom: &mut Dom, root: NodeId) -> NodeId`

- **Does:** A.10 — `AcceptRevisionsForPart` (:1314) at content scope: the exact 15-step transform order.
- **Calls (crate):** `element_has_tracked_revisions` (revision_processor.rs:90) L3253; `remove_rsid_transform` (markup_simplifier.rs:154) L3254; `drop_wholly_deleted_bookmarks` (revision_processor/bookmarks.rs:23) L3256; `fix_up_deleted_or_inserted_field_codes_transform` (revision_processor.rs:1037) L3257; `Dom::descendants` (xmllinq/mod.rs:668) L3258,3301; `accept_move_from_move_to_transform` (revision_processor.rs:129) L3260; `accept_move_from_ranges` (revision_processor.rs:1134) L3264; `accept_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1355) L3265; `accept_deleted_and_moved_from_content_controls` (revision_processor.rs:1452) L3266; `accept_deleted_and_move_from_paragraph_marks` (revision_processor.rs:2515) L3267; `remove_rows_left_empty_by_move_from` (revision_processor.rs:2549) L3269; `accept_all_other_revisions_transform` (revision_processor.rs:229) L3274; `accept_deleted_cells_transform` (revision_processor.rs:2627) L3278; `merge_adjacent_tables_transform` (revision_processor.rs:2959) L3279; `drop_unpaired_bookmarks` (revision_processor/bookmarks.rs:132) L3280; `has_empty_table_cell` (revision_processor.rs:3159) L3286; `add_empty_paragraph_to_any_empty_cells` (revision_processor.rs:3195) L3287; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L3295; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L3296,3297; `Dom::has_elements` (xmllinq/mod.rs:862) L3302; `Dom::remove` (xmllinq/mod.rs:1102) L3303
- **Calls (std/external/unresolved):** `.expect`, `.is_empty`, `.len`, `PT::run_ids`, `PT::unique_id`, `Some`, `W::move_from`, `W::num_pr`
- **Macros:** `debug_assert_eq!`
- **Called by:** `document_text_view` (debug.rs:2011); `accept_revisions_document` (revision_processor.rs:502); `reject_revisions_document` (revision_processor.rs:729); `resolve` (revision_processor.rs:3503); `revision_boundary_coverage_tests::internally_deleted_equations_leave_no_math_shell_but_keep_live_and_empty_math` (revision_processor.rs:4070); `revision_boundary_coverage_tests::display_math_preserves_independent_empty_equation_and_mixed_run_payload` (revision_processor.rs:4150)

#### `carry_kept_revisions`  (L3315-3331)
`fn carry_kept_revisions(dom: &mut Dom, live: NodeId, reverted: NodeId, first: bool)`

- **Does:** A property element `live` reverting to its saved copy `reverted` keeps the revisions left tracked in it (frozen by a selective resolution): a paragraph mark's insertion or deletion ahead of the run properties (`first`), a row's or a cell's marks after the rest.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L3316; `Dom::name` (xmllinq/mod.rs:526) L3319; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L3319; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L3321; `Dom::add_first` (xmllinq/mod.rs:1085) L3324; `Dom::add` (xmllinq/mod.rs:1058) L3328
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_some_and`, `.map`, `.rev`
- **Called by:** `reject_revisions_for_part_transform` (revision_processor.rs:619)

#### `accept_revisions_for_styles_transform`  (L3335-3353)
`fn accept_revisions_for_styles_transform(dom: &mut Dom, node: NodeId) -> Option<NodeId>`

- **Does:** A.11 — `AcceptRevisionsForStylesTransform` (:1300): drop `pPrChange`/ `rPrChange` from the styles part, rebuild the rest.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L3336; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L3337; `Dom::name` (xmllinq/mod.rs:526) L3339; `Dom::new_element` (xmllinq/mod.rs:478) L3343; `Dom::attributes` (xmllinq/mod.rs:885) L3344; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L3345; `Dom::nodes` (xmllinq/mod.rs:604) L3347; `accept_revisions_for_styles_transform` (revision_processor.rs:3335) L3348; `Dom::add` (xmllinq/mod.rs:1058) L3349
- **Calls (std/external/unresolved):** `.unwrap`, `Some`, `W::p_pr_change`, `W::r_pr_change`
- **Called by:** `accept_revisions_for_styles_transform` (revision_processor.rs:3335); `resolve` (revision_processor.rs:3503); `revision_boundary_coverage_tests::missing_saved_style_properties_drop_changed_block_while_accept_keeps_live_properties` (revision_processor.rs:4299)

#### `reject_revisions_for_styles_transform`  (L3359-3386)
`fn reject_revisions_for_styles_transform(dom: &mut Dom, node: NodeId) -> Option<NodeId>`

- **Does:** A.11 — `RejectRevisionsForStylesTransform` (:391): a `pPr` holding a `pPrChange` reverts to the change's inner `pPr` (same for `rPr`); a change element without its inner properties drops the whole node (C# recurses null).
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L3360; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L3361; `Dom::name` (xmllinq/mod.rs:526) L3363; `Dom::element` (xmllinq/mod.rs:655) L3365,3367,3371,3373; `reject_revisions_for_styles_transform` (revision_processor.rs:3359) L3368,3374,3381; `Dom::new_element` (xmllinq/mod.rs:478) L3376; `Dom::attributes` (xmllinq/mod.rs:885) L3377; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L3378; `Dom::nodes` (xmllinq/mod.rs:604) L3380; `Dom::add` (xmllinq/mod.rs:1058) L3382
- **Calls (std/external/unresolved):** `.and_then`, `.unwrap`, `Some`, `W::p_pr_change`, `W::p_pr`, `W::r_pr_change`, `W::r_pr`
- **Called by:** `reject_revisions_for_styles_transform` (revision_processor.rs:3359); `resolve` (revision_processor.rs:3503); `revision_boundary_coverage_tests::missing_saved_style_properties_drop_changed_block_while_accept_keeps_live_properties` (revision_processor.rs:4299)

#### `numbering_part`  (L3391-3401)
`fn numbering_part(pkg: &crate::opc::PartFs) -> Option<String>`

- **Does:** The parts `AcceptRevisions`/`RejectRevisions` (:1277/:31) walk, in the C# order: main, headers, footers, endnotes, footnotes, then styles (flagged).
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:829) L3392; `PartFs::read_rels_for` (opc/mod.rs:756) L3393; `PartFs::resolve_rel_target` (opc/mod.rs:600) L3400
- **Calls (std/external/unresolved):** `.as_deref`, `.find`, `.iter`, `.map`, `.next`, `.rsplit`, `Some`
- **Called by:** `resolve` (revision_processor.rs:3503)

#### `revision_bearing_parts`  (L3403-3441, pub(crate))
`pub(crate) fn revision_bearing_parts(pkg: &crate::opc::PartFs) -> Vec<(String, bool)>`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:829) L3404; `PartFs::read_rels_for` (opc/mod.rs:756) L3412; `PartFs::resolve_rel_target` (opc/mod.rs:600) L3425
- **Calls (std/external/unresolved):** `.as_deref`, `.chain`, `.into_iter`, `.next`, `.push`, `.rsplit`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `Some`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `changes_in` (changes.rs:302); `first_free_id` (edit/tracked.rs:250); `Transaction::finish` (edit.rs:5077); `resolve` (revision_processor.rs:3503); `remove_comments` (scrub.rs:220)

#### `process_part`  (L3443-3460)
`fn process_part<F>(pkg: &mut crate::opc::PartFs, part: &str, f: F) where F: FnOnce(&mut Dom, NodeId) -> Option<NodeId>,`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:498) L3447; `Dom::new` (xmllinq/mod.rs:387) L3450; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L3451; `Dom::root` (xmllinq/mod.rs:822) L3452; `Dom::replace_with` (xmllinq/mod.rs:1149) L3458; `PartFs::set_part` (opc/mod.rs:510) L3459; `Dom::serialize_document` (xmllinq/mod.rs:1322) L3459
- **Calls (std/external/unresolved):** `.into_bytes`, `f`
- **Called by:** `resolve` (revision_processor.rs:3503)

#### `accept_revisions_package`  (L3465-3467, pub)
`pub fn accept_revisions_package(pkg: &mut crate::opc::PartFs)`

- **Does:** A.11 — `AcceptRevisions` (:1277) at package scope: run the full part pipeline over main + headers + footers + endnotes + footnotes, and the styles transform over the styles part.
- **Calls (crate):** `resolve_package` (revision_processor.rs:3495) L3466
- **Called by:** `accept_revisions` (document_comparer.rs:4959)

#### `accept_revisions_package_keeping_ids`  (L3471-3473, pub)
`pub fn accept_revisions_package_keeping_ids(pkg: &mut crate::opc::PartFs)`

- **Does:** [`accept_revisions_package`] without renumbering bookmarks and comments, for a document about to be compared: the redline keeps its comment ids.
- **Calls (crate):** `resolve` (revision_processor.rs:3503) L3472
- **Called by:** `accept_source_revisions` (document_comparer.rs:4968)

#### `reject_revisions_package`  (L3480-3482, pub)
`pub fn reject_revisions_package(pkg: &mut crate::opc::PartFs)`

- **Does:** A.11 — `RejectRevisions` (:31) at package scope: per content part, the revert → reverse → rsid-strip → full-accept composition (the C# phases the same steps across all parts; parts are independent, so per-part composition is equivalent); the styles part reverts its property changes then accepts the leftovers.
- **Calls (crate):** `resolve_package` (revision_processor.rs:3495) L3481
- **Called by:** `reject_revisions` (document_comparer.rs:4976)

#### `resolve_package`  (L3495-3501, pub(crate))
`pub(crate) fn resolve_package( pkg: &mut crate::opc::PartFs, resolution: Resolution, freeze: Option<Freeze<'_>>, )`

- **Does:** Accept or reject every revision `freeze` leaves in place.
- **Calls (crate):** `resolve` (revision_processor.rs:3503) L3500
- **Called by:** `resolve` (changes.rs:349); `accept_revisions_package` (revision_processor.rs:3465); `reject_revisions_package` (revision_processor.rs:3480)

#### `resolve`  (L3503-3547)
`fn resolve( pkg: &mut crate::opc::PartFs, resolution: Resolution, freeze: Option<Freeze<'_>>, renumber: bool, )`

- **Does:** _no doc comment_
- **Calls (crate):** `revision_bearing_parts` (revision_processor.rs:3403) L3509; `numbering_part` (revision_processor.rs:3391) L3510; `PartFs::part_string` (opc/mod.rs:498) L3511; `Levels::parse` (revision_processor/style_records.rs:281) L3512; `process_part` (revision_processor.rs:3443) L3516; `accept_revisions_for_styles_transform` (revision_processor.rs:3335) L3521,3526; `recorded_blocks` (revision_processor/style_records.rs:369) L3523; `reject_revisions_for_styles_transform` (revision_processor.rs:3359) L3524; `restore_against_built_ins` (revision_processor/style_records.rs:399) L3525; `carry_vanishing_section_references` (revision_processor/sections.rs:31) L3529; `accept_revisions_for_part_content` (revision_processor.rs:3252) L3531; `reject_revisions_document` (revision_processor.rs:729) L3532; `thaw` (revision_processor.rs:3551) L3536; `story_parts` (revision_processor.rs:3567) L3540; `prune_orphan_comments` (revision_processor/comments.rs:66) L3541; `prune_orphan_notes` (revision_processor/notes.rs:24) L3542; `renumber` (revision_processor/annotation_ids.rs:22) L3544; `tidy` (revision_processor/word_save.rs:25) L3546
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.map`, `.unwrap_or_default`, `Some`, `freeze`
- **Called by:** `accept_revisions_package_keeping_ids` (revision_processor.rs:3471); `resolve_package` (revision_processor.rs:3495)

#### `thaw`  (L3551-3564)
`fn thaw(dom: &mut Dom, root: NodeId) -> bool`

- **Does:** Rename every frozen revision element under `root` back into `w:`; true when there was one.
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L3552; `Dom::name` (xmllinq/mod.rs:526) L3556; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L3557; `XName::local_name` (xmllinq/mod.rs:208) (by name) L3557; `Dom::set_name` (xmllinq/mod.rs:539) L3561
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.into_iter`, `.is_empty`, `.then`, `.to_string`, `W::name`
- **Called by:** `resolve` (revision_processor.rs:3503)

#### `story_parts`  (L3567-3573)
`fn story_parts(parts: &[(String, bool)]) -> Vec<String>`

- **Does:** The content parts of [`revision_bearing_parts`] (styles left out).
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.filter`, `.iter`, `.map`
- **Called by:** `resolve` (revision_processor.rs:3503)

## Test functions (55)

- `revision_boundary_coverage_tests::parse` (L3580): _no doc comment_
- `revision_boundary_coverage_tests::paragraph` (L3590): _no doc comment_
- `revision_boundary_coverage_tests::cell` (L3594): _no doc comment_
- `revision_boundary_coverage_tests::table` (L3601): _no doc comment_
- `revision_boundary_coverage_tests::local_children` (L3611): _no doc comment_
- `revision_boundary_coverage_tests::widths` (L3618): _no doc comment_
- `revision_boundary_coverage_tests::span` (L3626): _no doc comment_
- `revision_boundary_coverage_tests::revised_table_union_preserves_rows_payload_and_unrelated_clean_groups` (L3634): _no doc comment_
- `revision_boundary_coverage_tests::fix_widths_clamps_spans_and_leaves_widthless_cells_out_of_cursor` (L3700): _no doc comment_
- `revision_boundary_coverage_tests::nested_merge_clones_widthless_cell_and_keeps_control_identity` (L3741): _no doc comment_
- `revision_boundary_coverage_tests::word_save_merges_row_exceptions_with_row_values_taking_precedence` (L3765): _no doc comment_
- `revision_boundary_coverage_tests::table_merge_identity_respects_whole_properties_grid_history_and_bidi` (L3806): _no doc comment_
- `revision_boundary_coverage_tests::accepting_deleted_cells_keeps_anchor_payload_and_expands_existing_span` (L3848): _no doc comment_
- `revision_boundary_coverage_tests::deleted_cell_anchor_defaults_bad_span_and_orders_all_schema_properties` (L3884): _no doc comment_
- `revision_boundary_coverage_tests::custom_xml_deleted_control_unwraps_nested_controls_but_retains_order_and_anchors` (L3939): _no doc comment_
- `revision_boundary_coverage_tests::custom_xml_move_deletes_whole_control_and_plain_content_but_keeps_outside_control` (L3970): _no doc comment_
- `revision_boundary_coverage_tests::unmatched_custom_ranges_leave_control_payload_and_parent_links_untouched` (L3991): _no doc comment_
- `revision_boundary_coverage_tests::move_range_that_enters_table_keeps_container_properties_and_surviving_content` (L4011): _no doc comment_
- `revision_boundary_coverage_tests::accept_cell_merge_converts_restart_and_continue_only_inside_cell_properties` (L4034): _no doc comment_
- `revision_boundary_coverage_tests::internally_deleted_equations_leave_no_math_shell_but_keep_live_and_empty_math` (L4070): _no doc comment_
- `revision_boundary_coverage_tests::display_math_preserves_independent_empty_equation_and_mixed_run_payload` (L4150): _no doc comment_
- `revision_boundary_coverage_tests::internally_deleted_math_does_not_consume_independent_anchors_controls_or_empty_branches` (L4171): _no doc comment_
- `revision_boundary_coverage_tests::internally_deleted_math_does_not_consume_independent_anchors_controls_or_empty_branches::{fn semantic}` (L4172): _no doc comment_
- `revision_boundary_coverage_tests::wholly_deleted_math_hoists_owned_range_metadata_instead_of_discarding_it` (L4234): _no doc comment_
- `revision_boundary_coverage_tests::accept_deleted_fraction_drops_its_math_payload_and_preserves_live_fraction` (L4262): _no doc comment_
- `revision_boundary_coverage_tests::rejecting_structural_markers_drops_inserted_numbering_and_cell_without_losing_neighbors` (L4276): _no doc comment_
- `revision_boundary_coverage_tests::missing_saved_style_properties_drop_changed_block_while_accept_keeps_live_properties` (L4299): _no doc comment_
- `revision_boundary_coverage_tests::acceptance_removes_empty_numbering_but_keeps_live_numbering_and_clears_atom_annotations` (L4333): _no doc comment_
- `revision_boundary_coverage_tests::unmatched_or_contentless_move_ranges_preserve_identity_and_properties` (L4357): _no doc comment_
- `public_block_control_ownership_tests::package` (L4385): _no doc comment_
- `public_block_control_ownership_tests::resolved` (L4407): _no doc comment_
- `public_block_control_ownership_tests::assert_owner` (L4419): _no doc comment_
- `public_block_control_ownership_tests::public_mark_resolution_rewraps_transformed_paragraphs_with_original_control_metadata` (L4440): _no doc comment_
- `public_block_control_ownership_tests::public_mark_resolution_preserves_whole_table_payload_at_the_controls_block_level` (L4473): _no doc comment_
- `public_block_control_ownership_tests::nested_block_controls_keep_each_owner_and_transformed_paragraph_payload` (L4495): _no doc comment_
- `public_block_control_ownership_tests::nested_block_controls_keep_outer_table_owner_and_nested_cell_geometry` (L4569): _no doc comment_
- `body_equation_paragraph_mark_ownership_tests::equation` (L4625): _no doc comment_
- `body_equation_paragraph_mark_ownership_tests::semantic` (L4635): _no doc comment_
- `body_equation_paragraph_mark_ownership_tests::parse` (L4664): _no doc comment_
- `body_equation_paragraph_mark_ownership_tests::assert_math` (L4670): _no doc comment_
- `body_equation_paragraph_mark_ownership_tests::equation_is_an_opaque_block_barrier_for_deleted_paragraph_marks` (L4694): _no doc comment_
- `body_equation_paragraph_mark_ownership_tests::public_accept_and_reject_preserve_unchanged_body_math_among_revised_marks` (L4711): _no doc comment_
- `body_equation_paragraph_mark_ownership_tests::terminal_internal_equation_revisions_preserve_exact_surviving_paragraph_owners` (L4750): _no doc comment_
- `body_equation_paragraph_mark_ownership_tests::terminal_internal_equation_revisions_preserve_exact_surviving_paragraph_owners::{fn normalize_empty_properties}` (L4751): _no doc comment_
- `deleted_title_equation_barrier_tests::equation` (L4845): _no doc comment_
- `deleted_title_equation_barrier_tests::title` (L4862): _no doc comment_
- `deleted_title_equation_barrier_tests::parse` (L4883): _no doc comment_
- `deleted_title_equation_barrier_tests::semantic` (L4889): _no doc comment_
- `deleted_title_equation_barrier_tests::deleted_title_before_equation_is_removed_without_consuming_the_equation` (L4919): _no doc comment_
- `deleted_title_equation_barrier_tests::public_replaced_titles_and_equations_recover_complete_owned_sources` (L4936): _no doc comment_
- `complete_field_instruction_metadata_boundary_tests::repaired_complete_field_codes_preserve_source_comments_processing_instructions_and_formats` (L4997): _no doc comment_
- `alternate_content_paragraph_owner_tests::root` (L5078): _no doc comment_
- `alternate_content_paragraph_owner_tests::semantic` (L5083): _no doc comment_
- `alternate_content_paragraph_owner_tests::authored_property_only_alternatives_survive_but_wholly_deleted_wrappers_do_not` (L5112): _no doc comment_
- `alternate_content_paragraph_owner_tests::surviving_alternate_content_keeps_both_presentations_across_deleted_paragraph_marks` (L5158): _no doc comment_
