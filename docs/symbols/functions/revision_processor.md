# Functions and methods in `src/revision_processor.rs`

105 non-test, 55 test. Part of the [symbol manifest](../README.md).

#### `tracked_revisions_elements`  (L79-81, pub)
`pub fn tracked_revisions_elements() -> Vec<XName>`

- **Does:** The element names whose presence indicates the document carries tracked revisions (one per local name in the `w:` namespace).
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`, `W::name`
- **Called by:** `decisions` (edit/source_refs.rs:25); `has_history` (edit/v2_apply.rs:659); `SourceIndex::add_revision_part` (edit/v2_apply.rs:753)

#### `element_has_tracked_revisions`  (L89-105, pub)
`pub fn element_has_tracked_revisions(dom: &Dom, root: NodeId) -> bool`

- **Does:** True if `root` (or any descendant) is a tracked-revision element.
- **Calls (crate):** `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90) L104
- **Called by:** `is_restorable_layout_blank` (comparer/finalize.rs:3859); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215); `accept_move_from_move_to_transform` (revision_processor.rs:128); `accept_all_other_revisions_transform` (revision_processor.rs:228); `accept_revisions_for_element` (revision_processor.rs:452); `reject_content` (revision_processor.rs:726); `accept_content` (revision_processor.rs:3481); `resolve_with_comments` (revision_processor.rs:3871); `public_block_control_ownership_tests::nested_block_controls_keep_each_owner_and_transformed_paragraph_payload` (revision_processor.rs:4884); `public_block_control_ownership_tests::nested_block_controls_keep_outer_table_owner_and_nested_cell_geometry` (revision_processor.rs:4958)

#### `element_has_tracked_revisions::{fn walk}`  (L90-103)
`fn walk(dom: &Dom, id: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L91; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L92; `XName::local_name` (xmllinq/mod.rs:208) (by name) L93; `Dom::child_count` (xmllinq/mod.rs:622) L97; `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90) L98; `Dom::child_at` (xmllinq/mod.rs:632) L98
- **Calls (std/external/unresolved):** `.contains`
- **Called by:** `element_has_tracked_revisions` (revision_processor.rs:89); `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90); `element_or_desc_has_name` (revision_processor.rs:183); `element_or_desc_has_name::{fn walk}` (revision_processor.rs:184); `has_deleted_or_moved_from_paragraph_mark` (revision_processor.rs:1966); `has_deleted_or_moved_from_paragraph_mark::{fn walk}` (revision_processor.rs:1967); `subtree_needs_adjacent_table_merge` (revision_processor.rs:3136); `subtree_needs_adjacent_table_merge::{fn walk}` (revision_processor.rs:3137); `has_empty_table_cell` (revision_processor.rs:3384); `has_empty_table_cell::{fn walk}` (revision_processor.rs:3387)

#### `has_child`  (L108-110)
`fn has_child(dom: &Dom, el: NodeId, name: &XName) -> bool`

- **Does:** Does `el` have a child element named `local` (in the W namespace)?
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L109
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `has_path` (revision_processor.rs:113)

#### `has_path`  (L113-117)
`fn has_path(dom: &Dom, el: NodeId, a: &XName, b: &XName) -> bool`

- **Does:** Path check `el / a / b` exists (a, b are direct-child element names).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L114; `has_child` (revision_processor.rs:108) L116
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `Some`
- **Called by:** `removed_container` (revision_processor/comments.rs:240); `accept_all_other_revisions_transform` (revision_processor.rs:228)

#### `accept_move_from_move_to_transform`  (L128-171, pub)
`pub fn accept_move_from_move_to_transform(dom: &mut Dom, node: NodeId) -> Vec<NodeId>`

- **Does:** Port of `AcceptMoveFromMoveToTransform` — unwrap `w:moveTo`, drop `w:moveFrom`.
- **Calls (crate):** `element_has_tracked_revisions` (revision_processor.rs:89) L130; `take_subtree` (revision_processor.rs:175) L131,134,153; `Dom::is_element` (xmllinq/mod.rs:502) L133; `Dom::name` (xmllinq/mod.rs:526) L136,149,150; `Dom::nodes` (xmllinq/mod.rs:604) L139,165; `accept_move_from_move_to_transform` (revision_processor.rs:128) L140,166; `Dom::parent` (xmllinq/mod.rs:597) L148,150; `hoist_range_markers_from` (revision_processor.rs:2285) L158; `Dom::new_element` (xmllinq/mod.rs:478) L161; `Dom::attributes` (xmllinq/mod.rs:899) L162; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L163; `Dom::add` (xmllinq/mod.rs:1074) L167
- **Calls (std/external/unresolved):** `.and_then`, `.extend`, `.is_some_and`, `.unwrap`, `Some`, `Vec::new`, `W::move_from`, `W::move_to`, `W::p_pr`, `W::r_pr`
- **Macros:** `vec!`
- **Called by:** `accept_move_from_move_to_transform` (revision_processor.rs:128); `accept_revisions_for_element` (revision_processor.rs:452); `accept_content` (revision_processor.rs:3481)

#### `take_subtree`  (L175-180)
`fn take_subtree(dom: &mut Dom, node: NodeId) -> Vec<NodeId>`

- **Does:** Detach `node` from its parent (if any) and return it for reparenting.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L176; `Dom::remove` (xmllinq/mod.rs:1118) L177
- **Calls (std/external/unresolved):** `.is_some`
- **Macros:** `vec!`
- **Called by:** `accept_move_from_move_to_transform` (revision_processor.rs:128); `accept_all_other_revisions_transform` (revision_processor.rs:228)

#### `element_or_desc_has_name`  (L183-197)
`fn element_or_desc_has_name(dom: &Dom, root: NodeId, name: &XName) -> bool`

- **Does:** Non-allocating presence of `name` on `root` or any descendant element.
- **Calls (crate):** `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90) L196
- **Called by:** `fix_up_deleted_or_inserted_field_codes_transform` (revision_processor.rs:1051); `accept_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1550); `accept_deleted_cells_transform` (revision_processor.rs:2838); `table_has_revision_marks` (revision_processor.rs:3054)

#### `element_or_desc_has_name::{fn walk}`  (L184-195)
`fn walk(dom: &Dom, id: NodeId, name: &XName) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L185; `Dom::child_count` (xmllinq/mod.rs:622) L188; `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90) L190; `Dom::child_at` (xmllinq/mod.rs:632) L190
- **Calls (std/external/unresolved):** `.as_ref`, `Some`
- **Called by:** `element_has_tracked_revisions` (revision_processor.rs:89); `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90); `element_or_desc_has_name` (revision_processor.rs:183); `element_or_desc_has_name::{fn walk}` (revision_processor.rs:184); `has_deleted_or_moved_from_paragraph_mark` (revision_processor.rs:1966); `has_deleted_or_moved_from_paragraph_mark::{fn walk}` (revision_processor.rs:1967); `subtree_needs_adjacent_table_merge` (revision_processor.rs:3136); `subtree_needs_adjacent_table_merge::{fn walk}` (revision_processor.rs:3137); `has_empty_table_cell` (revision_processor.rs:3384); `has_empty_table_cell::{fn walk}` (revision_processor.rs:3387)

#### `has_run_content`  (L201-214)
`fn has_run_content(dom: &Dom, element: NodeId) -> bool`

- **Does:** True iff a transformed element still carries run/inline content (port of `HasRunContent`) — used to decide whether an emptied `w:hyperlink` survives.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L211; `Dom::name` (xmllinq/mod.rs:526) L213
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.into_iter`, `.is_some_and`, `W::del`, `W::fld_simple`, `W::hyperlink`, `W::ins`, `W::r`, `W::sdt`, `W::smart_tag`
- **Called by:** `accept_all_other_revisions_transform` (revision_processor.rs:228)

#### `accept_all_other_revisions_transform`  (L228-439, pub)
`pub fn accept_all_other_revisions_transform(dom: &mut Dom, node: NodeId) -> Vec<NodeId>`

- **Does:** Port of `AcceptAllOtherRevisionsTransform` — accept inserts (unwrap `w:ins`), drop deletions and revision-range markers, accept formatting- change markers, handle deleted rows/tables, cell merges, and empty hyperlink shells.
- **Calls (crate):** `element_has_tracked_revisions` (revision_processor.rs:89) L230; `take_subtree` (revision_processor.rs:175) L231,234; `Dom::is_element` (xmllinq/mod.rs:502) L233,345; `Dom::name` (xmllinq/mod.rs:526) L236,314,331,391; `Dom::nodes` (xmllinq/mod.rs:604) L242,344,414,433; `accept_all_other_revisions_transform` (revision_processor.rs:228) L243,415,434; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L263,282,321,332; `XName::local_name` (xmllinq/mod.rs:208) (by name) L263,282,324,329,332; `Dom::descendants` (xmllinq/mod.rs:668) L292,297,301,305; `Dom::name_is` (xmllinq/mod.rs:535) L295,303,307,310,343; `Dom::ancestors` (xmllinq/mod.rs:770) L306; `Dom::text_value` (xmllinq/mod.rs:547) L315,346; `Dom::elements` (xmllinq/mod.rs:640) L335,341,358; `hoist_range_markers_from` (revision_processor.rs:2285) L352,384,424; `has_path` (revision_processor.rs:113) L361,368; `table_rows_all_deleted` (revision_processor.rs:1941) L373; `Dom::parent` (xmllinq/mod.rs:597) L389; `Dom::attribute` (xmllinq/mod.rs:874) L394; `Dom::new_element` (xmllinq/mod.rs:478) L396,401,410,429; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L397,402,412,431; `Dom::attributes` (xmllinq/mod.rs:899) L411,430; `Dom::add` (xmllinq/mod.rs:1074) L416,435; `has_run_content` (revision_processor.rs:201) L419
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.as_deref`, `.collect`, `.contains`, `.ends_with`, `.extend`, `.filter`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.map`, `.to_string`, `.trim`, `.unwrap`, `M::name`, `Some`, `Vec::new`, `W::cell_merge`, `W::del`, `W::hyperlink`, `W::ins`, `W::tbl`, `W::tc_pr`, `W::tr_pr`, `W::tr`, `W::v_merge`, `W::val`
- **Macros:** `vec!`
- **Called by:** `accept_all_other_revisions_transform` (revision_processor.rs:228); `accept_revisions_for_element` (revision_processor.rs:452); `all_para_content_is_deleted` (revision_processor.rs:1912); `accept_content` (revision_processor.rs:3481); `revision_boundary_coverage_tests::accept_cell_merge_converts_restart_and_continue_only_inside_cell_properties` (revision_processor.rs:4424); `revision_boundary_coverage_tests::internally_deleted_math_does_not_consume_independent_anchors_controls_or_empty_branches` (revision_processor.rs:4561); `revision_boundary_coverage_tests::wholly_deleted_math_hoists_owned_range_metadata_instead_of_discarding_it` (revision_processor.rs:4623); `revision_boundary_coverage_tests::accept_deleted_fraction_drops_its_math_payload_and_preserves_live_fraction` (revision_processor.rs:4651)

#### `accept_revisions_for_element`  (L452-482, pub)
`pub fn accept_revisions_for_element(dom: &mut Dom, element: NodeId) -> NodeId`

- **Does:** Port of `AcceptRevisionsForElement` (the documented simple-revision entry point, RevisionProcessor.ts:1004): RemoveRsid → AcceptMoveFromMoveTo → AcceptAllOtherRevisions → strip PT.UniqueId/RunIds → drop empty w:numPr.
- **Calls (crate):** `element_has_tracked_revisions` (revision_processor.rs:89) L453; `remove_rsid_transform` (markup_simplifier.rs:154) L454; `accept_move_from_move_to_transform` (revision_processor.rs:128) L456; `accept_all_other_revisions_transform` (revision_processor.rs:228) L459; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L469; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L470,471; `Dom::descendants` (xmllinq/mod.rs:668) L476; `Dom::has_elements` (xmllinq/mod.rs:862) L477; `Dom::remove` (xmllinq/mod.rs:1118) L478
- **Calls (std/external/unresolved):** `.expect`, `.len`, `PT::run_ids`, `PT::unique_id`, `Some`, `W::num_pr`
- **Macros:** `debug_assert_eq!`
- **Called by:** `revision_boundary_coverage_tests::acceptance_removes_empty_numbering_but_keeps_live_numbering_and_clears_atom_annotations` (revision_processor.rs:4722)

#### `accept_revisions_document`  (L495-497, pub)
`pub fn accept_revisions_document(dom: &mut Dom, root: NodeId) -> NodeId`

- **Does:** `AcceptRevisionsDocument` at element scope — accept all tracked revisions in the `root` element's subtree, returning the new root.
- **Calls (crate):** `accept_revisions_for_part_content` (revision_processor.rs:3477) L496
- **Called by:** `coverage_round_two_tests::wholesale_mesh_preserves_each_authored_terminal_whitespace_and_format::{fn source_projection}` (comparer/finalize.rs:16914); `coverage_round_two_tests::wholesale_mesh_without_terminal_whitespace_keeps_exact_supported_word_streams::{fn owned}` (comparer/finalize.rs:16965); `coverage_round_next_tests::trailing_deletion_peel_truth_table_for_shared_long_and_copula_paths::{fn projected_body}` (comparer/finalize.rs:19556); `deep_production_path_tests::title_spacers_require_unique_revised_two_blank_pretable_context` (comparer/finalize.rs:21817); `deep_production_path_tests::restored_title_spacers_preserve_each_revised_blank_template` (comparer/finalize.rs:21909); `deep_production_path_tests::short_title_splice_crosses_only_tables_whose_every_row_is_structurally_inserted::{fn projected}` (comparer/finalize.rs:22465); `deep_production_path_tests::deletion_peel_keeps_retained_source_boundary_and_inserted_continuations::{fn paragraphs}` (comparer/finalize.rs:22639); `deep_production_path_tests::final_deletion_peel_transfers_boundary_only_with_equal_properties_and_text_payload::{fn projected}` (comparer/finalize.rs:22705); `deep_production_path_tests::deletion_peel_keeps_nontext_only_original_anchors_in_their_source_paragraph::{fn projected}` (comparer/finalize.rs:22796); `deep_production_path_tests::deletion_peel_keeps_punctuation_and_whitespace_only_original_anchors::{fn projection}` (comparer/finalize.rs:22855); `deep_production_path_tests::deletion_peel_keeps_original_closing_boundary_when_all_original_words_are_deleted::{fn projection}` (comparer/finalize.rs:22902); `deep_production_path_tests::wholesale_deleted_body_peel_transfers_its_retained_closing_boundary::{fn projection}` (comparer/finalize.rs:22947); +37 more

#### `effective_parent`  (L501-506)
`fn effective_parent(dom: &Dom, node: NodeId) -> Option<NodeId>`

- **Does:** The "skipping" parent used by `ReverseRevisionsTransform` — nearest ancestor whose name is not `w:sdtContent`/`w:sdt`/`w:smartTag`.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L503; `Dom::name` (xmllinq/mod.rs:526) L505
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.into_iter`, `.is_some_and`, `W::sdt_content`, `W::sdt`, `W::smart_tag`
- **Called by:** `reverse_revisions_transform` (revision_processor.rs:516)

#### `reverse_revisions_transform`  (L516-589)
`fn reverse_revisions_transform(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** Port of `ReverseRevisionsForPartTransform` — invert the *sense* of each revision (del↔ins, moveFrom↔moveTo, delText→t, custom-xml ranges, …), context-aware via the effective parent.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L517; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L518; `Dom::name` (xmllinq/mod.rs:526) L520,522,525; `effective_parent` (revision_processor.rs:501) L521; `Dom::parent` (xmllinq/mod.rs:597) L524; `Dom::new_element` (xmllinq/mod.rs:478) L530,534,538,541; `rebuild_named` (revision_processor.rs:592) L554,557,584,588; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L579; `XName::local_name` (xmllinq/mod.rs:208) (by name) L582
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.find`, `.is_some_and`, `.iter`, `.unwrap`, `Some`, `W::del`, `W::ins`, `W::name`, `W::p_pr`, `W::r_pr`, `W::tr_pr`
- **Called by:** `rebuild_named` (revision_processor.rs:592); `reject_content` (revision_processor.rs:726)

#### `rebuild_named`  (L592-604)
`fn rebuild_named(dom: &mut Dom, new_name: XName, src: NodeId, keep_attrs: bool) -> NodeId`

- **Does:** Build `<new_name attrs?
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L593; `Dom::attributes` (xmllinq/mod.rs:899) L595; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L596; `Dom::nodes` (xmllinq/mod.rs:604) L599; `reverse_revisions_transform` (revision_processor.rs:516) L600; `Dom::add` (xmllinq/mod.rs:1074) L601
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `reverse_revisions_transform` (revision_processor.rs:516)

#### `reject_revisions_for_part_transform`  (L612-709)
`fn reject_revisions_for_part_transform(dom: &mut Dom, node: NodeId) -> Option<NodeId>`

- **Does:** Port of `RejectRevisionsForPartTransform` — revert the *non-invertible* revisions (property changes) and drop inserted structural markers.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L613; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L614,640,647,664,677; `Dom::name` (xmllinq/mod.rs:526) L616,658; `Dom::element` (xmllinq/mod.rs:655) L619,636,639,645,673,675...; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L632; `XName::local_name` (xmllinq/mod.rs:208) (by name) L635; `Dom::new_element` (xmllinq/mod.rs:478) L641,678,699; `Dom::add` (xmllinq/mod.rs:1074) L648,705; `Dom::elements` (xmllinq/mod.rs:640) L654,689; `Dom::add_first` (xmllinq/mod.rs:1101) L665; `carry_kept_revisions` (revision_processor.rs:3568) L668,680; `reject_revisions_for_part_transform` (revision_processor.rs:612) L669,681,704; `Dom::attributes` (xmllinq/mod.rs:899) L700; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L701; `Dom::nodes` (xmllinq/mod.rs:604) L703
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.filter`, `.find`, `.into_iter`, `.is_some_and`, `.is_some`, `.iter`, `.rev`, `.unwrap`, `Some`, `W::cell_del`, `W::cell_ins`, `W::cell_merge`, `W::ins`, `W::name`, `W::num_pr`, `W::numbering_change`, `W::p_pr`, `W::r_pr_change`, `W::r_pr`, `W::sect_pr`, `W::tc_pr`, `W::tc`
- **Called by:** `reject_revisions_for_part_transform` (revision_processor.rs:612); `reject_content` (revision_processor.rs:726); `revision_boundary_coverage_tests::rejecting_structural_markers_drops_inserted_numbering_and_cell_without_losing_neighbors` (revision_processor.rs:4665)

#### `reject_revisions_document`  (L722-724, pub)
`pub fn reject_revisions_document(dom: &mut Dom, root: NodeId) -> NodeId`

- **Does:** `RejectRevisionsDocument` at element scope: revert property changes, invert the sense of every remaining revision, strip rsids, then accept.
- **Calls (crate):** `reject_content` (revision_processor.rs:726) L723
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `coverage_round_two_tests::wholesale_mesh_preserves_each_authored_terminal_whitespace_and_format::{fn source_projection}` (comparer/finalize.rs:16914); `coverage_round_two_tests::wholesale_mesh_without_terminal_whitespace_keeps_exact_supported_word_streams::{fn owned}` (comparer/finalize.rs:16965); `coverage_round_next_tests::trailing_deletion_peel_truth_table_for_shared_long_and_copula_paths::{fn projected_body}` (comparer/finalize.rs:19556); `deep_production_path_tests::title_spacers_require_unique_revised_two_blank_pretable_context` (comparer/finalize.rs:21817); `deep_production_path_tests::short_title_splice_crosses_only_tables_whose_every_row_is_structurally_inserted` (comparer/finalize.rs:22464); `deep_production_path_tests::short_title_splice_crosses_only_tables_whose_every_row_is_structurally_inserted::{fn projected}` (comparer/finalize.rs:22465); `deep_production_path_tests::deletion_peel_keeps_retained_source_boundary_and_inserted_continuations::{fn paragraphs}` (comparer/finalize.rs:22639); `deep_production_path_tests::final_deletion_peel_transfers_boundary_only_with_equal_properties_and_text_payload::{fn projected}` (comparer/finalize.rs:22705); `deep_production_path_tests::deletion_peel_keeps_nontext_only_original_anchors_in_their_source_paragraph::{fn projected}` (comparer/finalize.rs:22796); `deep_production_path_tests::deletion_peel_keeps_punctuation_and_whitespace_only_original_anchors::{fn projection}` (comparer/finalize.rs:22855); `deep_production_path_tests::deletion_peel_keeps_original_closing_boundary_when_all_original_words_are_deleted::{fn projection}` (comparer/finalize.rs:22902); `deep_production_path_tests::wholesale_deleted_body_peel_transfers_its_retained_closing_boundary::{fn projection}` (comparer/finalize.rs:22947); +38 more

#### `reject_content`  (L726-747)
`fn reject_content( dom: &mut Dom, root: NodeId, keep_comments: bool, kept: &mut Vec<String>, ) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `element_has_tracked_revisions` (revision_processor.rs:89) L732; `remove_rsid_transform` (markup_simplifier.rs:154) L733,745; `reject_revisions_for_part_transform` (revision_processor.rs:612) L736; `normalize_orphan_moves` (revision_processor.rs:1174) L743; `reverse_revisions_transform` (revision_processor.rs:516) L744; `accept_content` (revision_processor.rs:3481) L746
- **Calls (std/external/unresolved):** `.expect`
- **Called by:** `reject_revisions_document` (revision_processor.rs:722); `resolve_with_comments` (revision_processor.rs:3871)

#### `descendant_and_self_tags`  (L781-823, pub)
`pub fn descendant_and_self_tags(dom: &Dom, element: NodeId) -> Vec<Tag>`

- **Does:** A.0 — `DescendantAndSelfTags` (RevisionProcessor.cs :2397): stream `element` and its descendants as open / empty / close tags in document order.
- **Calls (crate):** `element_children_vec` (revision_processor.rs:826) L789,805; `Dom::child_count` (xmllinq/mod.rs:622) L794
- **Calls (std/external/unresolved):** `.last_mut`, `.last`, `.len`, `.pop`, `.push`
- **Macros:** `vec!`
- **Called by:** `wholly_deleted_endpoints` (revision_processor/bookmarks.rs:47); `normalize_orphan_moves` (revision_processor.rs:1174); `move_from_range_deleted_elements` (revision_processor.rs:1341); `accept_deleted_and_moved_from_content_controls` (revision_processor.rs:1647)

#### `element_children_vec`  (L826-836)
`fn element_children_vec(dom: &Dom, id: NodeId) -> Vec<NodeId>`

- **Does:** Direct element children (document order) without the `elements()` filter path.
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L828; `Dom::child_at` (xmllinq/mod.rs:632) L830; `Dom::is_element` (xmllinq/mod.rs:502) L831
- **Calls (std/external/unresolved):** `.push`, `Vec::new`
- **Called by:** `descendant_and_self_tags` (revision_processor.rs:781); `iterate_block_content_elements` (revision_processor.rs:901)

#### `first_block_content`  (L856-886)
`fn first_block_content(dom: &Dom, roots: &[NodeId]) -> Option<NodeId>`

- **Does:** First paragraph, table, or opaque equation block among `roots`' descendants-and-self, document order.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L859; `Dom::child_count` (xmllinq/mod.rs:622) L869; `Dom::child_at` (xmllinq/mod.rs:632) L874; `Dom::is_element` (xmllinq/mod.rs:502) L876
- **Calls (std/external/unresolved):** `.is_some_and`, `.last_mut`, `.pop`, `.push`, `M::name`, `Some`, `W::p`, `W::tbl`, `is_block`
- **Macros:** `vec!`
- **Called by:** `iterate_block_content_elements` (revision_processor.rs:901)

#### `elements_after_self`  (L889-894)
`fn elements_after_self(dom: &Dom, id: NodeId) -> Vec<NodeId>`

- **Does:** Element siblings after `id`, document order.
- **Calls (crate):** `Dom::nodes_after_self` (xmllinq/mod.rs:838) L890; `Dom::is_element` (xmllinq/mod.rs:502) L892
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`
- **Called by:** `iterate_block_content_elements` (revision_processor.rs:901)

#### `iterate_block_content_elements`  (L901-937, pub)
`pub fn iterate_block_content_elements(dom: &Dom, element: NodeId) -> Vec<BlockContentInfo>`

- **Does:** A.0 — `IterateBlockContentElements` (:1909) + `AnnotateBlockContentElements` (:1855): the doc-order chain of paragraphs, tables, and equation blocks under `element`, linked prev/this/next.
- **Calls (crate):** `element_children_vec` (revision_processor.rs:826) L903; `first_block_content` (revision_processor.rs:856) L907,915; `elements_after_self` (revision_processor.rs:889) L915; `Dom::parent` (xmllinq/mod.rs:597) L919
- **Calls (std/external/unresolved):** `.collect`, `.copied`, `.get`, `.is_empty`, `.last`, `.len`, `.map`, `.push`, `.then`, `.unwrap`, `Some`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `accept_paragraph_marks_with_owners` (revision_processor.rs:2006); `body_equation_paragraph_mark_ownership_tests::equation_is_an_opaque_block_barrier_for_deleted_paragraph_marks` (revision_processor.rs:5082)

#### `block_level_content_containers`  (L944-955)
`fn block_level_content_containers() -> [XName; 8]`

- **Does:** `W.BlockLevelContentContainers` (PtOpenXmlUtil.cs :5569), plus `w:comment`: the C# pipeline never visits `comments.xml`, but Word's Accept/Reject All does, and a deleted or inserted paragraph mark inside a comment merges the comment's own paragraphs — never across the `w:comment` boundary.
- **Calls (std/external/unresolved):** `W::body`, `W::endnote`, `W::footnote`, `W::ftr`, `W::hdr`, `W::name`, `W::tc`, `W::txbx_content`
- **Called by:** `get_paragraph_info` (revision_processor.rs:966); `accept_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1550); `accept_paragraph_marks_with_owners` (revision_processor.rs:2006)

#### `get_paragraph_info`  (L966-998, pub)
`pub fn get_paragraph_info(dom: &Dom, content_element: NodeId) -> BlockContentInfo`

- **Does:** A.0 — `GetParagraphInfo` (:2917) + `InitializeParagraphInfo` (:2890): for a child of a block-level content container, `this` = the first descendant-or-self in {`w:p`, `w:tc`, `w:txbxContent`} — nulled when that first hit is a `tc`/`txbxContent` (means "no own paragraph") — and `previous` = the previous element sibling...
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L967; `Dom::name` (xmllinq/mod.rs:526) L970,980,982; `block_level_content_containers` (revision_processor.rs:944) L972; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L977; `Dom::nodes_before_self` (xmllinq/mod.rs:846) L987; `Dom::is_element` (xmllinq/mod.rs:502) L991
- **Calls (std/external/unresolved):** `.contains`, `.expect`, `.find`, `.into_iter`, `.is_some_and`, `.rev`, `W::p`, `W::tc`, `W::txbx_content`
- **Macros:** `assert!`
- **Called by:** `accept_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1550)

#### `content_elements_before_self`  (L1002-1008, pub)
`pub fn content_elements_before_self(dom: &Dom, element: NodeId) -> Vec<NodeId>`

- **Does:** A.0 — `ContentElementsBeforeSelf` (:2926): previous element siblings, nearest first.
- **Calls (crate):** `Dom::nodes_before_self` (xmllinq/mod.rs:846) L1003; `Dom::is_element` (xmllinq/mod.rs:502) L1006
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.rev`
- **Called by:** `accept_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1550); `accept_deleted_cells_transform` (revision_processor.rs:2838)

#### `transform_instr_text_to_del_instr_text`  (L1014-1040)
`fn transform_instr_text_to_del_instr_text(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** A.1 — `TransformInstrTextToDelInstrText` (:1433): rename `w:instrText` to `w:delInstrText` (attrs + nodes carried as-is), rebuilding everything else.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L1015; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1016,1026; `Dom::name` (xmllinq/mod.rs:526) L1018; `Dom::new_element` (xmllinq/mod.rs:478) L1020,1031; `Dom::attributes` (xmllinq/mod.rs:899) L1021,1032; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1022,1033; `Dom::nodes` (xmllinq/mod.rs:604) L1025,1035; `Dom::add` (xmllinq/mod.rs:1074) L1027,1037; `transform_instr_text_to_del_instr_text` (revision_processor.rs:1014) L1036
- **Calls (std/external/unresolved):** `.unwrap`, `Some`, `W::del_instr_text`, `W::instr_text`
- **Called by:** `transform_instr_text_to_del_instr_text` (revision_processor.rs:1014); `fix_up_deleted_or_inserted_field_codes_transform` (revision_processor.rs:1051)

#### `fix_up_deleted_or_inserted_field_codes_transform`  (L1051-1135, pub)
`pub fn fix_up_deleted_or_inserted_field_codes_transform(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** A.1 — `FixUpDeletedOrInsertedFieldCodesTransform` (:1354): inside each `w:p`, group-adjacent the child elements by kind — 2 = `w:del` holding `w:r/w:fldChar`, 3 = `w:ins` holding `w:r/w:fldChar`, 4 = `w:r` with `w:instrText`, 1 = other.
- **Calls (crate):** `element_or_desc_has_name` (revision_processor.rs:183) L1055; `Dom::is_element` (xmllinq/mod.rs:502) L1058; `Dom::name` (xmllinq/mod.rs:526) L1061,1064; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L1066; `Dom::element` (xmllinq/mod.rs:655) (by name) L1068; `Dom::element` (xmllinq/mod.rs:655) L1074; `Dom::elements` (xmllinq/mod.rs:640) L1080; `Dom::new_element` (xmllinq/mod.rs:478) L1084,1101,1108,1126; `Dom::attributes` (xmllinq/mod.rs:899) L1085,1127; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1086,1128; `fix_up_deleted_or_inserted_field_codes_transform` (revision_processor.rs:1051) L1092,1110,1116,1131; `Dom::add` (xmllinq/mod.rs:1074) L1093,1104,1106,1111,1113,1117...; `transform_instr_text_to_del_instr_text` (revision_processor.rs:1014) L1103; `Dom::nodes` (xmllinq/mod.rs:604) L1130
- **Calls (std/external/unresolved):** `.any`, `.enumerate`, `.into_iter`, `.is_some`, `.iter`, `.len`, `.unwrap`, `Some`, `W::del`, `W::fld_char`, `W::ins`, `W::instr_text`, `W::p`, `W::r`, `crate::util::group_adjacent`, `flanked_by`, `holds_fld_char`, `key_of`
- **Macros:** `unreachable!`
- **Called by:** `fix_up_deleted_or_inserted_field_codes_transform` (revision_processor.rs:1051); `accept_content` (revision_processor.rs:3481); `complete_field_instruction_metadata_boundary_tests::repaired_complete_field_codes_preserve_source_comments_processing_instructions_and_formats` (revision_processor.rs:5384)

#### `normalize_orphan_moves`  (L1174-1281)
`fn normalize_orphan_moves(dom: &mut Dom, root: NodeId, pass: OrphanMovePass) -> NodeId`

- **Does:** #1235, #1243: a `w:moveTo` / `w:moveFrom` is an ORPHAN when no range of the other kind shares its `w:name` in the same part — a bare `moveTo` has no `w:name` at all, so it is an orphan unless it sits inside a paired `moveToRange`.
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L1178,1205; `Dom::name` (xmllinq/mod.rs:526) L1179,1206,1224; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1180,1225; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1183,1228; `Dom::attribute` (xmllinq/mod.rs:874) L1185,1190,1230,1231,1235,1239...; `descendant_and_self_tags` (revision_processor.rs:781) L1222; `normalize_orphan_moves_transform` (revision_processor.rs:1286) L1279
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.expect`, `.insert`, `.into_iter`, `.is_empty`, `.is_some_and`, `.remove`, `.to_string`, `.unwrap_or`, `.values`, `HashMap::new`, `HashSet::new`, `W::id`, `W::move_from`, `W::move_to`, `W::name`
- **Called by:** `reject_content` (revision_processor.rs:726); `accept_content` (revision_processor.rs:3481)

#### `normalize_orphan_moves_transform`  (L1286-1316)
`fn normalize_orphan_moves_transform( dom: &mut Dom, node: NodeId, to_ins: &HashSet<NodeId>, to_del: &HashSet<NodeId>, drop_markers: &HashSet<NodeId>, ) -> Option<NodeId>`

- **Does:** Rebuild for [`normalize_orphan_moves`]: orphan `moveTo`/`moveFrom` re-emerge as `w:ins`/`w:del` (attributes preserved), orphan range markers vanish, everything else rebuilds recursively.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L1293; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1294; `Dom::name` (xmllinq/mod.rs:526) L1304; `Dom::new_element` (xmllinq/mod.rs:478) L1306; `Dom::attributes` (xmllinq/mod.rs:899) L1307; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1308; `Dom::nodes` (xmllinq/mod.rs:604) L1310; `normalize_orphan_moves_transform` (revision_processor.rs:1286) L1311; `Dom::add` (xmllinq/mod.rs:1074) L1312
- **Calls (std/external/unresolved):** `.contains`, `.unwrap`, `Some`, `W::del`, `W::ins`
- **Called by:** `normalize_orphan_moves` (revision_processor.rs:1174); `normalize_orphan_moves_transform` (revision_processor.rs:1286)

#### `accept_move_from_ranges`  (L1329-1336, pub)
`pub fn accept_move_from_ranges(dom: &mut Dom, document: NodeId) -> NodeId`

- **Does:** A.2 — `AcceptMoveFromRanges` (:1530): walk the tag stream; while one or more `moveFromRangeStart` ids are open, record every other element's open tag (start-list) and close tag (end-list; empty elements land in both).
- **Calls (crate):** `move_from_range_deleted_elements` (revision_processor.rs:1341) L1330; `accept_move_from_ranges_transform` (revision_processor.rs:1433) L1334
- **Calls (std/external/unresolved):** `.expect`, `.is_empty`
- **Called by:** `accept_content` (revision_processor.rs:3481); `revision_boundary_coverage_tests::move_range_that_enters_table_keeps_container_properties_and_surviving_content` (revision_processor.rs:4401); `revision_boundary_coverage_tests::unmatched_or_contentless_move_ranges_preserve_identity_and_properties` (revision_processor.rs:4746)

#### `move_from_range_deleted_elements`  (L1341-1416)
`fn move_from_range_deleted_elements(dom: &Dom, document: NodeId) -> HashSet<NodeId>`

- **Does:** Elements wholly removed by completed move-from ranges.
- **Calls (crate):** `descendant_and_self_tags` (revision_processor.rs:781) L1352; `Dom::name` (xmllinq/mod.rs:526) L1353,1408; `Dom::attribute` (xmllinq/mod.rs:874) L1355,1363; `is_inline_range_marker` (revision_processor.rs:2259) L1408; `is_container_property` (revision_processor.rs:1421) L1409,1413; `Dom::ancestors` (xmllinq/mod.rs:770) L1410
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.contains`, `.extend`, `.filter`, `.insert`, `.into_iter`, `.is_empty`, `.is_some_and`, `.push`, `.remove`, `.to_string`, `.unwrap_or`, `.unwrap`, `.values_mut`, `HashMap::new`, `Vec::new`, `W::id`, `W::move_from_range_end`, `W::move_from_range_start`
- **Called by:** `wholly_deleted_endpoints` (revision_processor/bookmarks.rs:47); `accept_move_from_ranges` (revision_processor.rs:1329)

#### `is_container_property`  (L1421-1429)
`fn is_container_property(dom: &Dom, e: NodeId) -> bool`

- **Does:** Is `e` the property element of its container (a paragraph's pPr, a table's tblPr or tblGrid, a row's trPr or tblPrEx, a cell's tcPr, a content control's sdtPr or sdtEndPr)?
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L1422; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1423; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1425
- **Calls (std/external/unresolved):** `.is_some_and`
- **Macros:** `matches!`
- **Called by:** `move_from_range_deleted_elements` (revision_processor.rs:1341)

#### `accept_move_from_ranges_transform`  (L1433-1460)
`fn accept_move_from_ranges_transform( dom: &mut Dom, node: NodeId, to_delete: &std::collections::HashSet<NodeId>, ) -> Option<NodeId>`

- **Does:** A.2 — `AcceptMoveFromRangesTransform` (:2629): rebuild, dropping the elements marked for deletion (and thereby their subtrees).
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L1438; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1439; `Dom::new_element` (xmllinq/mod.rs:478) L1444; `Dom::name` (xmllinq/mod.rs:526) L1444; `Dom::attributes` (xmllinq/mod.rs:899) L1445; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1446; `Dom::nodes` (xmllinq/mod.rs:604) L1448; `accept_move_from_ranges_transform` (revision_processor.rs:1433) L1449; `Dom::add` (xmllinq/mod.rs:1074) L1450,1455; `hoist_range_markers_from` (revision_processor.rs:2285) L1454
- **Calls (std/external/unresolved):** `.contains`, `.unwrap`, `Some`
- **Called by:** `accept_move_from_ranges` (revision_processor.rs:1329); `accept_move_from_ranges_transform` (revision_processor.rs:1433)

#### `collapse_paragraph_transform`  (L1467-1490, pub)
`pub fn collapse_paragraph_transform(dom: &mut Dom, node: NodeId) -> Vec<NodeId>`

- **Does:** A.3 — `CollapseParagraphTransform` (:1821): a `w:p` collapses to its element children minus `w:pPr` (clones — C# XLinq re-parents by cloning); other elements rebuild recursively; non-elements pass through.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L1468; `Dom::clone_subtree` (xmllinq/mod.rs:1231) (by name) L1469; `Dom::name` (xmllinq/mod.rs:526) L1471,1476; `Dom::elements` (xmllinq/mod.rs:640) L1473; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1478; `Dom::new_element` (xmllinq/mod.rs:478) L1480; `Dom::attributes` (xmllinq/mod.rs:899) L1481; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1482; `Dom::nodes` (xmllinq/mod.rs:604) L1484; `collapse_paragraph_transform` (revision_processor.rs:1467) L1485; `Dom::add` (xmllinq/mod.rs:1074) L1486
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.map`, `.unwrap`, `Some`, `W::p_pr`, `W::p`
- **Macros:** `vec!`
- **Called by:** `collapse_paragraph_transform` (revision_processor.rs:1467); `coalesque_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1498); `accept_paragraph_marks_with_owners` (revision_processor.rs:2006)

#### `coalesque_paragraph_end_tags_in_move_from_transform`  (L1498-1532, pub)
`pub fn coalesque_paragraph_end_tags_in_move_from_transform( dom: &mut Dom, node: NodeId, group: &[NodeId], ) -> NodeId`

- **Does:** A.3 — `CoalesqueParagraphEndTagsInMoveFromTransform` (:2645): produce the first group member, with its (first descendant) paragraph replaced by one carrying the paragraph's own children plus the COLLAPSED content of the group's subsequent members.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L1503; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1504,1513; `Dom::name` (xmllinq/mod.rs:526) L1506; `Dom::new_element` (xmllinq/mod.rs:478) L1508,1523; `Dom::attributes` (xmllinq/mod.rs:899) L1509,1524; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1510,1525; `Dom::elements` (xmllinq/mod.rs:640) L1512; `Dom::add` (xmllinq/mod.rs:1074) L1514,1518,1529; `collapse_paragraph_transform` (revision_processor.rs:1467) L1517; `Dom::nodes` (xmllinq/mod.rs:604) L1527; `coalesque_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1498) L1528
- **Calls (std/external/unresolved):** `.iter`, `.skip`, `.unwrap`, `Some`, `W::p`
- **Called by:** `coalesque_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1498); `accept_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1550)

#### `accept_paragraph_end_tags_in_move_from_transform`  (L1550-1636, pub)
`pub fn accept_paragraph_end_tags_in_move_from_transform(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** A.3 — `AcceptParagraphEndTagsInMoveFromTransform` (:1610): group a content container's children by whether their paragraph mark sits in an OPEN moveFrom range (`moveFromRangeStart` child without `moveFromRangeEnd`), or they are a `w:p` directly following such a paragraph.
- **Calls (crate):** `element_or_desc_has_name` (revision_processor.rs:183) L1552; `Dom::is_element` (xmllinq/mod.rs:502) L1555; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1556,1602; `Dom::name` (xmllinq/mod.rs:526) L1558,1580; `block_level_content_containers` (revision_processor.rs:944) L1559; `Dom::elements` (xmllinq/mod.rs:640) L1563,1588; `get_paragraph_info` (revision_processor.rs:966) L1567,1574,1579; `content_elements_before_self` (revision_processor.rs:1002) L1573; `Dom::new_element` (xmllinq/mod.rs:478) L1595,1617,1627; `Dom::attributes` (xmllinq/mod.rs:899) L1596,1618,1628; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1597,1619,1629; `Dom::add` (xmllinq/mod.rs:1074) L1603,1610,1623,1633; `coalesque_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1498) L1609; `Dom::nodes` (xmllinq/mod.rs:604) L1621,1631; `accept_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1550) L1622,1632
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.into_iter`, `.is_empty`, `.is_some`, `.len`, `.unwrap`, `Some`, `W::move_from_range_end`, `W::move_from_range_start`, `W::p`, `crate::util::group_adjacent`, `key_of`, `mark_in_open_range`
- **Called by:** `accept_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1550); `accept_content` (revision_processor.rs:3481)

#### `accept_deleted_and_moved_from_content_controls`  (L1647-1766, pub)
`pub fn accept_deleted_and_moved_from_content_controls(dom: &mut Dom, root: NodeId) -> NodeId`

- **Does:** A.4 — `AcceptDeletedAndMovedFromContentControls` (:2491): walk the tag stream tracking two range kinds.
- **Calls (crate):** `descendant_and_self_tags` (revision_processor.rs:781) L1665; `Dom::name` (xmllinq/mod.rs:526) L1666; `Dom::attribute` (xmllinq/mod.rs:874) L1668,1676,1684,1692; `accept_deleted_and_moved_from_content_controls_transform` (revision_processor.rs:1771) L1758
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.contains`, `.extend`, `.filter`, `.insert`, `.into_iter`, `.is_empty`, `.len`, `.push`, `.remove`, `.to_string`, `.unwrap_or`, `.unwrap`, `.values_mut`, `HashMap::new`, `Vec::new`, `W::id`, `W::move_from_range_end`, `W::move_from_range_start`, `W::name`, `W::sdt`
- **Macros:** `debug_assert_eq!`
- **Called by:** `accept_content` (revision_processor.rs:3481); `revision_boundary_coverage_tests::custom_xml_deleted_control_unwraps_nested_controls_but_retains_order_and_anchors` (revision_processor.rs:4329); `revision_boundary_coverage_tests::custom_xml_move_deletes_whole_control_and_plain_content_but_keeps_outside_control` (revision_processor.rs:4360); `revision_boundary_coverage_tests::unmatched_custom_ranges_leave_control_payload_and_parent_links_untouched` (revision_processor.rs:4381)

#### `accept_deleted_and_moved_from_content_controls_transform`  (L1771-1811)
`fn accept_deleted_and_moved_from_content_controls_transform( dom: &mut Dom, node: NodeId, to_collapse: &std::collections::HashSet<NodeId>, to_delete: &std::collections::HashSet<NodeId>, ) -> Vec<NodeId>`

- **Does:** A.4 — `AcceptDeletedAndMovedFromContentControlsTransform` (:2468): splice a collapsed sdt's `sdtContent` child nodes (transformed) in its place, drop deleted elements, rebuild the rest.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L1777; `Dom::clone_subtree` (xmllinq/mod.rs:1231) (by name) L1778; `Dom::name` (xmllinq/mod.rs:526) L1780; `Dom::element` (xmllinq/mod.rs:655) L1782; `Dom::nodes` (xmllinq/mod.rs:604) L1786,1803; `accept_deleted_and_moved_from_content_controls_transform` (revision_processor.rs:1771) L1787,1805; `Dom::new_element` (xmllinq/mod.rs:478) L1799; `Dom::attributes` (xmllinq/mod.rs:899) L1800; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1801; `Dom::add` (xmllinq/mod.rs:1074) L1807
- **Calls (std/external/unresolved):** `.contains`, `.expect`, `.extend`, `.unwrap`, `Some`, `Vec::new`, `W::sdt_content`, `W::sdt`
- **Macros:** `vec!`
- **Called by:** `accept_deleted_and_moved_from_content_controls` (revision_processor.rs:1647); `accept_deleted_and_moved_from_content_controls_transform` (revision_processor.rs:1771)

#### `is_run_content`  (L1817-1860)
`fn is_run_content(name: &XName) -> Option<bool>`

- **Does:** A.5a — `IsRunContent` (:2356): `Some(true)` = run-level content, `Some(false)` = marker/non-content, `None` = unknown (C# throws).
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L1824,1829,1832; `XName::local_name` (xmllinq/mod.rs:208) L1830,1835
- **Calls (std/external/unresolved):** `.contains`, `Some`, `crate::namespaces::MC::name`
- **Called by:** `all_para_content_is_deleted` (revision_processor.rs:1912)

#### `collapse_transform`  (L1868-1904)
`fn collapse_transform(dom: &mut Dom, node: NodeId) -> Vec<NodeId>`

- **Does:** A.5a — `CollapseTransform` (:2331): splice `w:dir`/`w:bdr`/`w:ins`/ `w:moveTo`/`w:smartTag` to their element children and `w:sdt` to its `sdtContent` element children — ONE level, un-recursed, exactly like the C# (`return element.Elements()`); drop `w:pPr`; rebuild everything else recursively.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L1869; `Dom::clone_subtree` (xmllinq/mod.rs:1231) (by name) L1870; `Dom::name` (xmllinq/mod.rs:526) L1872; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1873; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1875; `Dom::elements` (xmllinq/mod.rs:640) L1879,1884,1885; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1880,1886; `Dom::new_element` (xmllinq/mod.rs:478) L1894; `Dom::attributes` (xmllinq/mod.rs:899) L1895; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1896; `Dom::nodes` (xmllinq/mod.rs:604) L1898; `collapse_transform` (revision_processor.rs:1868) L1899; `Dom::add` (xmllinq/mod.rs:1074) L1900
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`, `.push`, `.unwrap`, `Some`, `Vec::new`, `W::p_pr`, `W::sdt_content`, `W::sdt`
- **Macros:** `matches!`, `vec!`
- **Called by:** `collapse_transform` (revision_processor.rs:1868); `all_para_content_is_deleted` (revision_processor.rs:1912)

#### `all_para_content_is_deleted`  (L1912-1938)
`fn all_para_content_is_deleted(dom: &mut Dom, p: NodeId) -> bool`

- **Does:** A.5a — `AllParaContentIsDeleted` (:2310): after collapsing wrappers, does the paragraph hold NO run-level content?
- **Calls (crate):** `collapse_transform` (revision_processor.rs:1868) L1913; `Dom::elements` (xmllinq/mod.rs:640) L1916; `Dom::name` (xmllinq/mod.rs:526) L1917,1928; `accept_all_other_revisions_transform` (revision_processor.rs:228) L1925; `is_run_content` (revision_processor.rs:1817) L1929,1935; `XName::clark` (xmllinq/mod.rs:226) (by name) L1930,1936
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_some_and`, `.len`, `.unwrap_or_else`, `.unwrap`, `M::name`
- **Macros:** `debug_assert_eq!`, `panic!`
- **Called by:** `accept_paragraph_marks_with_owners` (revision_processor.rs:2006)

#### `table_rows_all_deleted`  (L1941-1948)
`fn table_rows_all_deleted(dom: &Dom, tbl: NodeId) -> bool`

- **Does:** Does every row of `tbl` carry `trPr/w:del` (and is there a row)?
- **Calls (crate):** `wrapped_children` (revision_processor.rs:2992) L1942; `Dom::element` (xmllinq/mod.rs:655) L1945,1946
- **Calls (std/external/unresolved):** `.all`, `.into_iter`, `.is_empty`, `.is_some_and`, `.is_some`, `W::del`, `W::tr_pr`, `W::tr`
- **Called by:** `removed_container` (revision_processor/comments.rs:240); `accept_all_other_revisions_transform` (revision_processor.rs:228); `accept_paragraph_marks_with_owners` (revision_processor.rs:2006)

#### `paragraph_mark_is_deleted_or_moved_from`  (L1952-1962)
`fn paragraph_mark_is_deleted_or_moved_from(dom: &Dom, p: NodeId) -> bool`

- **Does:** `p / pPr / rPr / (del | moveFrom)` — is the paragraph mark deleted or moved from?
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1954,1955; `Dom::elements` (xmllinq/mod.rs:640) L1957; `Dom::name` (xmllinq/mod.rs:526) L1958
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.into_iter`, `.is_some_and`, `W::del`, `W::move_from`, `W::p_pr`, `W::r_pr`
- **Called by:** `wholly_deleted_endpoints` (revision_processor/bookmarks.rs:47); `carry_vanishing_section_references` (revision_processor/sections.rs:31); `has_deleted_or_moved_from_paragraph_mark::{fn walk}` (revision_processor.rs:1967); `accept_paragraph_marks_with_owners` (revision_processor.rs:2006)

#### `has_deleted_or_moved_from_paragraph_mark`  (L1966-1982)
`fn has_deleted_or_moved_from_paragraph_mark(dom: &Dom, root: NodeId) -> bool`

- **Does:** ACCEPT-SKIP-A5: non-allocating presence of any paragraph whose mark is deleted or moved-from under `root`.
- **Calls (crate):** `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90) L1981
- **Called by:** `accept_deleted_and_move_from_paragraph_marks` (revision_processor.rs:2726); `body_equation_paragraph_mark_ownership_tests::public_accept_and_reject_preserve_unchanged_body_math_among_revised_marks` (revision_processor.rs:5099)

#### `has_deleted_or_moved_from_paragraph_mark::{fn walk}`  (L1967-1980)
`fn walk(dom: &Dom, id: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L1968; `paragraph_mark_is_deleted_or_moved_from` (revision_processor.rs:1952) L1969; `Dom::child_count` (xmllinq/mod.rs:622) L1973; `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90) L1975; `Dom::child_at` (xmllinq/mod.rs:632) L1975
- **Calls (std/external/unresolved):** `.as_ref`, `Some`, `W::p`
- **Called by:** `element_has_tracked_revisions` (revision_processor.rs:89); `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90); `element_or_desc_has_name` (revision_processor.rs:183); `element_or_desc_has_name::{fn walk}` (revision_processor.rs:184); `has_deleted_or_moved_from_paragraph_mark` (revision_processor.rs:1966); `has_deleted_or_moved_from_paragraph_mark::{fn walk}` (revision_processor.rs:1967); `subtree_needs_adjacent_table_merge` (revision_processor.rs:3136); `subtree_needs_adjacent_table_merge::{fn walk}` (revision_processor.rs:3137); `has_empty_table_cell` (revision_processor.rs:3384); `has_empty_table_cell::{fn walk}` (revision_processor.rs:3387)

#### `accept_deleted_and_move_from_paragraph_marks_transform`  (L1997-2002, pub)
`pub fn accept_deleted_and_move_from_paragraph_marks_transform( dom: &mut Dom, node: NodeId, ) -> NodeId`

- **Does:** A.5a — `AcceptDeletedAndMoveFromParagraphMarksTransform` (:2119): the 3-state grouping machine over a container's block-content chain.
- **Calls (crate):** `accept_paragraph_marks_with_owners` (revision_processor.rs:2006) L2001
- **Calls (std/external/unresolved):** `HashMap::new`
- **Called by:** `body_equation_paragraph_mark_ownership_tests::equation_is_an_opaque_block_barrier_for_deleted_paragraph_marks` (revision_processor.rs:5082); `deleted_title_equation_barrier_tests::deleted_title_before_equation_is_removed_without_consuming_the_equation` (revision_processor.rs:5306)

#### `accept_paragraph_marks_with_owners`  (L2006-2243)
`fn accept_paragraph_marks_with_owners( dom: &mut Dom, node: NodeId, paragraph_owners: &mut HashMap<NodeId, Vec<NodeId>>, ) -> NodeId`

- **Does:** Original paragraph owners are private reconstruction evidence. _(from a code comment)_
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L2011; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L2012,2107,2180,2196,2239; `Dom::name` (xmllinq/mod.rs:526) L2014,2043,2075,2114,2139,2158...; `block_level_content_containers` (revision_processor.rs:944) L2015; `Dom::new_element` (xmllinq/mod.rs:478) L2016,2105,2159,2175; `Dom::attributes` (xmllinq/mod.rs:899) L2017,2160,2176; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L2018,2161,2177; `Dom::nodes` (xmllinq/mod.rs:604) L2020,2163; `accept_paragraph_marks_with_owners` (revision_processor.rs:2006) L2021,2164; `Dom::add` (xmllinq/mod.rs:1074) L2022,2108,2119,2165,2181,2197...; `Dom::name_is` (xmllinq/mod.rs:535) L2024,2127,2167; `Dom::element` (xmllinq/mod.rs:655) L2031; `iterate_block_content_elements` (revision_processor.rs:901) L2036; `paragraph_mark_is_deleted_or_moved_from` (revision_processor.rs:1952) L2045,2132; `table_rows_all_deleted` (revision_processor.rs:1941) L2076; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L2082; `Dom::elements` (xmllinq/mod.rs:640) L2106,2179,2188; `collapse_paragraph_transform` (revision_processor.rs:1467) L2118; `all_para_content_is_deleted` (revision_processor.rs:1912) L2143; `hoist_range_markers_from` (revision_processor.rs:2285) L2150; `is_body_level_range_marker` (revision_processor.rs:2246) L2195; `Dom::ancestors` (xmllinq/mod.rs:770) L2225
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.collect`, `.contains`, `.copied`, `.enumerate`, `.filter`, `.insert`, `.into_iter`, `.is_none`, `.is_some_and`, `.iter`, `.last`, `.len`, `.position`, `.push`, `.unwrap`, `.zip`, `HashSet::new`, `M::name`, `Some`, `Vec::new`, `Vec::with_capacity`, `W::body`, `W::p_pr`, `W::p`, `W::sect_pr`, `W::tbl`, `W::tc_pr`, `crate::util::group_adjacent`
- **Macros:** `vec!`
- **Called by:** `accept_deleted_and_move_from_paragraph_marks_transform` (revision_processor.rs:1997); `accept_paragraph_marks_with_owners` (revision_processor.rs:2006); `accept_deleted_and_move_from_paragraph_marks` (revision_processor.rs:2726)

#### `is_body_level_range_marker`  (L2246-2257)
`fn is_body_level_range_marker(name: &XName) -> bool`

- **Does:** Body/cell-level markers that must survive block-content rebuild (not p/tbl).
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L2247; `XName::local_name` (xmllinq/mod.rs:208) (by name) L2249
- **Macros:** `matches!`
- **Called by:** `accept_paragraph_marks_with_owners` (revision_processor.rs:2006)

#### `is_inline_range_marker`  (L2259-2270)
`fn is_inline_range_marker(name: &XName) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L2260; `XName::local_name` (xmllinq/mod.rs:208) (by name) L2262
- **Macros:** `matches!`
- **Called by:** `move_from_range_deleted_elements` (revision_processor.rs:1341); `hoist_range_markers_from` (revision_processor.rs:2285)

#### `drop_wholly_deleted_permissions`  (L2275-2281)
`fn drop_wholly_deleted_permissions(dom: &mut Dom, root: NodeId)`

- **Does:** Permission ranges follow Word's character-range rule, including paragraph marks: discard both endpoints only when their nonempty span disappears.
- **Calls (crate):** `wholly_deleted_endpoints` (revision_processor/bookmarks.rs:47) L2277; `Dom::remove` (xmllinq/mod.rs:1118) L2279
- **Calls (std/external/unresolved):** `W::name`
- **Called by:** `accept_content` (revision_processor.rs:3481)

#### `hoist_range_markers_from`  (L2285-2296)
`fn hoist_range_markers_from(dom: &mut Dom, node: NodeId) -> Vec<NodeId>`

- **Does:** Pull comment, bookmark and permission markers out of a subtree being discarded (e.g.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2287; `Dom::name` (xmllinq/mod.rs:526) L2288; `is_inline_range_marker` (revision_processor.rs:2259) L2291; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L2292
- **Calls (std/external/unresolved):** `.push`, `Vec::new`
- **Called by:** `accept_move_from_move_to_transform` (revision_processor.rs:128); `accept_all_other_revisions_transform` (revision_processor.rs:228); `accept_move_from_ranges_transform` (revision_processor.rs:1433); `accept_paragraph_marks_with_owners` (revision_processor.rs:2006)

#### `annotate_run_elements_with_id`  (L2302-2307, pub)
`pub fn annotate_run_elements_with_id(dom: &mut Dom, element: NodeId)`

- **Does:** A.5b — `AnnotateRunElementsWithId` (:1935): number every descendant `w:r` with `pt:UniqueId` 0..
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2304; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L2305
- **Calls (std/external/unresolved):** `.to_string`, `.zip`, `PT::unique_id`, `Some`, `W::r`
- **Called by:** `accept_deleted_and_move_from_paragraph_marks` (revision_processor.rs:2726)

#### `annotate_paragraph_elements_with_id`  (L2314-2319)
`fn annotate_paragraph_elements_with_id(dom: &mut Dom, element: NodeId)`

- **Does:** Number every descendant `w:p` (`pt:UniqueId` `p0`, `p1`, …) so A.5b can find a control's paragraphs when it has no run to anchor on (an empty control keeps its paragraph mark, and so its place).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2316; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L2317
- **Calls (std/external/unresolved):** `.zip`, `PT::unique_id`, `Some`, `W::p`
- **Macros:** `format!`
- **Called by:** `accept_deleted_and_move_from_paragraph_marks` (revision_processor.rs:2726)

#### `descendants_trimmed`  (L2323-2335)
`fn descendants_trimmed(dom: &Dom, element: NodeId, trim: &XName) -> Vec<NodeId>`

- **Does:** Descendants of `element` in document order, not descending INTO elements named `trim` (the `DescendantsTrimmed` helper the annotators use).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L2325,2329; `Dom::name` (xmllinq/mod.rs:526) L2328
- **Calls (std/external/unresolved):** `.as_ref`, `.collect`, `.into_iter`, `.pop`, `.push`, `.rev`, `Some`, `Vec::new`
- **Called by:** `annotate_content_controls_with_run_ids` (revision_processor.rs:2340)

#### `annotate_content_controls_with_run_ids`  (L2340-2353, pub)
`pub fn annotate_content_controls_with_run_ids(dom: &mut Dom, element: NodeId)`

- **Does:** A.5b — `AnnotateContentControlsWithRunIds` (:1945): give every descendant `w:sdt` a `pt:RunIds` (comma-joined `pt:UniqueId`s of its runs, trimmed at `w:txbxContent`) and its own `pt:UniqueId` (in place).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2344; `descendants_trimmed` (revision_processor.rs:2323) L2345; `Dom::name` (xmllinq/mod.rs:526) L2347; `Dom::attribute` (xmllinq/mod.rs:874) L2348; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L2350,2351
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.filter`, `.into_iter`, `.join`, `.map`, `.to_string`, `.zip`, `PT::run_ids`, `PT::unique_id`, `Some`, `W::r`, `W::sdt`, `W::txbx_content`
- **Called by:** `accept_deleted_and_move_from_paragraph_marks` (revision_processor.rs:2726)

#### `order_sdt`  (L2356-2368)
`fn order_sdt(name: &XName) -> i32`

- **Does:** `Order_sdt` (:2090): schema order for rebuilt `w:sdt` children.
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L2357; `XName::local_name` (xmllinq/mod.rs:208) L2360
- **Called by:** `add_content_controls_with_owners` (revision_processor.rs:2393)

#### `add_block_level_content_controls`  (L2385-2391, pub)
`pub fn add_block_level_content_controls( dom: &mut Dom, new_document: NodeId, original: NodeId, ) -> NodeId`

- **Does:** A.5b — `AddBlockLevelContentControls` (:1964): re-create the `w:sdt` wrappers the paragraph-mark transform stripped.
- **Calls (crate):** `add_content_controls_with_owners` (revision_processor.rs:2393) L2390
- **Called by:** none found in src

#### `add_content_controls_with_owners`  (L2393-2717)
`fn add_content_controls_with_owners( dom: &mut Dom, new_document: NodeId, original: NodeId, paragraph_owners: Option<&HashMap<NodeId, Vec<NodeId>>>, ) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2405,2406,2418,2425,2442,2459...; `Dom::attribute` (xmllinq/mod.rs:874) L2409,2419,2426,2432,2436,2445...; `Dom::element` (xmllinq/mod.rs:655) L2463,2464,2466,2467,2490,2527; `Dom::ancestors` (xmllinq/mod.rs:770) L2474,2493,2502,2563,2569,2612...; `Dom::name` (xmllinq/mod.rs:526) L2478,2497,2511,2552,2562,2567...; `fragmented_source_label_boundary_tests::wrapper` (comparer/finalize.rs:33698) (by name) L2497,2511; `Dom::parent` (xmllinq/mod.rs:597) L2508,2630; `Dom::ancestors_and_self` (xmllinq/mod.rs:787) L2539,2628; `Dom::elements` (xmllinq/mod.rs:640) L2637,2667,2679,2697; `Dom::new_element` (xmllinq/mod.rs:478) L2663,2689,2693; `Dom::attributes` (xmllinq/mod.rs:899) L2664,2690; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L2665,2691; `order_sdt` (revision_processor.rs:2356) L2668,2704; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L2670,2702; `Dom::add` (xmllinq/mod.rs:1074) L2671,2695,2706,2709,2711,2713; `Dom::add_before_self` (xmllinq/mod.rs:1130) L2673; `Dom::remove` (xmllinq/mod.rs:1118) L2674,2687
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.as_ref`, `.collect`, `.contains`, `.copied`, `.count`, `.entry`, `.expect`, `.filter_map`, `.filter`, `.find`, `.first`, `.get`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_some_and`, `.is_some`, `.iter`, `.last`, `.map`, `.nth`, `.or_insert`, `.position`, `.push`, `.retain`, `.reverse`, `.sort_by_key`, `.split`, `.take_while`, `.to_string`, `.to_vec`, `.unwrap_or`, `.unwrap`, `HashMap::new`, `PT::run_ids`, `PT::unique_id`, `Some`, `Vec::new`, `W::del`, `W::ins`, `W::move_from`, `W::move_to`, `W::name`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `W::sdt_content`, `W::sdt`, `W::tbl`, `child_containing`
- **Called by:** `add_block_level_content_controls` (revision_processor.rs:2385); `accept_deleted_and_move_from_paragraph_marks` (revision_processor.rs:2726)

#### `accept_deleted_and_move_from_paragraph_marks`  (L2726-2737, pub)
`pub fn accept_deleted_and_move_from_paragraph_marks(dom: &mut Dom, element: NodeId) -> NodeId`

- **Does:** A.5b — `AcceptDeletedAndMoveFromParagraphMarks` (:2098): annotate runs and content controls (on the ORIGINAL, in place), run the A.5a transform, then re-wrap the content controls the transform stripped.
- **Calls (crate):** `has_deleted_or_moved_from_paragraph_mark` (revision_processor.rs:1966) L2728; `annotate_run_elements_with_id` (revision_processor.rs:2302) L2731; `annotate_paragraph_elements_with_id` (revision_processor.rs:2314) L2732; `annotate_content_controls_with_run_ids` (revision_processor.rs:2340) L2733; `accept_paragraph_marks_with_owners` (revision_processor.rs:2006) L2735; `add_content_controls_with_owners` (revision_processor.rs:2393) L2736
- **Calls (std/external/unresolved):** `HashMap::new`, `Some`
- **Called by:** `accept_content` (revision_processor.rs:3481)

#### `a6_block_level_elements`  (L2743-2754)
`fn a6_block_level_elements() -> [XName; 8]`

- **Does:** `BlockLevelElements` (:2766) — the direct-cell-child names that make a cell non-empty for [`remove_rows_left_empty_by_move_from`].
- **Calls (std/external/unresolved):** `M::name`, `W::del`, `W::ins`, `W::move_to`, `W::p`, `W::sdt`, `W::tbl`
- **Called by:** `remove_rows_left_empty_by_move_from_inner` (revision_processor.rs:2764)

#### `remove_rows_left_empty_by_move_from`  (L2760-2762, pub)
`pub fn remove_rows_left_empty_by_move_from(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** A.6 — `RemoveRowsLeftEmptyByMoveFrom` (:2777): drop every `w:tr` whose cells all lost their block-level content to an accepted moveFrom; rebuild everything else.
- **Calls (crate):** `remove_rows_left_empty_by_move_from_inner` (revision_processor.rs:2764) L2761
- **Calls (std/external/unresolved):** `.expect`
- **Called by:** `accept_content` (revision_processor.rs:3481)

#### `remove_rows_left_empty_by_move_from_inner`  (L2764-2797)
`fn remove_rows_left_empty_by_move_from_inner(dom: &mut Dom, node: NodeId) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L2765; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L2766; `Dom::name` (xmllinq/mod.rs:526) L2768,2774; `a6_block_level_elements` (revision_processor.rs:2743) L2770; `Dom::elements` (xmllinq/mod.rs:640) L2771,2772; `Dom::element` (xmllinq/mod.rs:655) L2780,2793; `Dom::new_element` (xmllinq/mod.rs:478) L2781; `Dom::attributes` (xmllinq/mod.rs:899) L2782; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L2783; `Dom::nodes` (xmllinq/mod.rs:604) L2785; `remove_rows_left_empty_by_move_from_inner` (revision_processor.rs:2764) L2786; `Dom::add` (xmllinq/mod.rs:1074) L2787
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.into_iter`, `.is_none`, `.is_some_and`, `.is_some`, `.unwrap`, `Some`, `W::tbl`, `W::tc`, `W::tr`
- **Called by:** `remove_rows_left_empty_by_move_from` (revision_processor.rs:2760); `remove_rows_left_empty_by_move_from_inner` (revision_processor.rs:2764)

#### `order_tc_pr`  (L2802-2823)
`fn order_tc_pr(name: &XName) -> i32`

- **Does:** `Order_tcPr` (:1466): schema order for rebuilt `w:tcPr` children.
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L2803; `XName::local_name` (xmllinq/mod.rs:208) L2806
- **Called by:** `accept_deleted_cells_transform` (revision_processor.rs:2838); `merge_adjacent_tables` (revision_processor.rs:3198)

#### `accept_deleted_cells_transform`  (L2838-2942, pub)
`pub fn accept_deleted_cells_transform(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** A.7 — `AcceptDeletedCellsTransform` (:2674): inside each `w:tr`, group adjacent children by (deleted-cell?, anchor), where a cell is "deleted" when it carries `w:cellDel` OR the next `w:tc` after it does, and the anchor is the nearest preceding (or self) `w:tc` WITHOUT `cellDel`.
- **Calls (crate):** `element_or_desc_has_name` (revision_processor.rs:183) L2841; `Dom::is_element` (xmllinq/mod.rs:502) L2844; `Dom::name` (xmllinq/mod.rs:526) L2847,2870,2872,2875,2917,2922...; `Dom::new_element` (xmllinq/mod.rs:478) L2849,2882,2911,2924,2928; `Dom::attributes` (xmllinq/mod.rs:899) L2850,2883; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L2851,2884,2912; `Dom::nodes` (xmllinq/mod.rs:604) L2853; `accept_deleted_cells_transform` (revision_processor.rs:2838) L2854; `Dom::add` (xmllinq/mod.rs:1074) L2855,2890,2926,2929,2937,2939; `Dom::descendants` (xmllinq/mod.rs:668) L2862; `Dom::elements` (xmllinq/mod.rs:640) L2864,2914,2930; `Dom::nodes_after_self` (xmllinq/mod.rs:838) L2867; `content_elements_before_self` (revision_processor.rs:1002) L2874; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L2889,2920,2936; `Dom::element` (xmllinq/mod.rs:655) L2901,2904; `Dom::attribute` (xmllinq/mod.rs:874) L2906; `order_tc_pr` (revision_processor.rs:2802) L2922
- **Calls (std/external/unresolved):** `.and_then`, `.chain`, `.clone`, `.collect`, `.expect`, `.filter`, `.find`, `.into_iter`, `.is_empty`, `.is_some_and`, `.len`, `.ok`, `.parse`, `.push`, `.sort_by_key`, `.to_string`, `.unwrap_or`, `.unwrap`, `Some`, `W::cell_del`, `W::grid_span`, `W::tc_pr`, `W::tc`, `W::tr`, `W::val`, `crate::util::group_adjacent`, `has_cell_del`, `key_of`, `std::iter::once`
- **Macros:** `vec!`
- **Called by:** `accept_deleted_cells_transform` (revision_processor.rs:2838); `accept_content` (revision_processor.rs:3481); `revision_boundary_coverage_tests::accepting_deleted_cells_keeps_anchor_payload_and_expands_existing_span` (revision_processor.rs:4238); `revision_boundary_coverage_tests::deleted_cell_anchor_defaults_bad_span_and_orders_all_schema_properties` (revision_processor.rs:4274)

#### `own_tbl_pr_ex`  (L2948-2967)
`fn own_tbl_pr_ex(dom: &Dom, first: NodeId, member: NodeId) -> Vec<NodeId>`

- **Does:** R8: the `tblPrEx`-able properties of `member` that `first` (whose `tblPr` the merged table keeps) does not share.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L2950; `Dom::elements` (xmllinq/mod.rs:640) L2952; `tbl_pr_ex_rank` (revision_processor.rs:3087) L2954; `Dom::serialize_element` (xmllinq/mod.rs:1315) L2961,2965
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter`, `.into_iter`, `.is_some`, `.map`, `.unwrap_or_default`, `W::tbl_pr`, `ex_of`
- **Called by:** `merge_adjacent_tables` (revision_processor.rs:3198)

#### `row_tbl_pr_ex`  (L2971-2989)
`fn row_tbl_pr_ex(dom: &mut Dom, tr: NodeId, own: &[NodeId]) -> NodeId`

- **Does:** R8: row `tr`'s `tblPrEx` with its table's `own` properties under the row's own, in schema order.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L2972; `Dom::elements` (xmllinq/mod.rs:640) L2974; `Dom::name` (xmllinq/mod.rs:526) L2978; `tbl_pr_ex_rank` (revision_processor.rs:3087) L2982; `Dom::new_element` (xmllinq/mod.rs:478) L2983; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L2985; `Dom::add` (xmllinq/mod.rs:1074) L2986
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.iter`, `.len`, `.map`, `.push`, `.sort_by_key`, `.unwrap_or_default`, `.unwrap_or`, `W::name`
- **Called by:** `merge_adjacent_tables` (revision_processor.rs:3198)

#### `wrapped_children`  (L2992-3005)
`fn wrapped_children(dom: &Dom, parent: NodeId, name: &XName) -> Vec<NodeId>`

- **Does:** Matching children in order, through wrappers but not into nested tables.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L2994; `Dom::name_is` (xmllinq/mod.rs:535) L2995; `Dom::name` (xmllinq/mod.rs:526) L2997,2998,2999; `wrapped_children` (revision_processor.rs:2992) L3001
- **Calls (std/external/unresolved):** `.extend`, `.push`, `Some`, `Vec::new`, `W::name`
- **Called by:** `table_rows_all_deleted` (revision_processor.rs:1941); `wrapped_children` (revision_processor.rs:2992); `fix_widths` (revision_processor.rs:3011); `merge_adjacent_tables` (revision_processor.rs:3198)

#### `fix_widths`  (L3011-3050)
`fn fix_widths(dom: &mut Dom, tbl: NodeId) -> NodeId`

- **Does:** A.8 — `FixWidths` (:1484): clone the table and rewrite each `w:tcW`'s `w:w` to the sum of the grid columns its cell spans (per the ORIGINAL table's `tblGrid`).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L3012,3015; `Dom::attribute` (xmllinq/mod.rs:874) L3017,3030,3035; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L3022; `wrapped_children` (revision_processor.rs:2992) L3023,3025; `Dom::element` (xmllinq/mod.rs:655) L3027,3029,3032,3034; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L3045
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.enumerate`, `.expect`, `.filter`, `.flat_map`, `.into_iter`, `.is_some`, `.iter`, `.len`, `.map`, `.ok`, `.parse`, `.sum`, `.to_string`, `.unwrap_or`, `Some`, `W::grid_span`, `W::name`, `W::tc_pr`, `W::tc`, `W::tr`, `W::val`, `std::cmp::min`
- **Called by:** `merge_adjacent_tables` (revision_processor.rs:3198); `revision_boundary_coverage_tests::fix_widths_clamps_spans_and_leaves_widthless_cells_out_of_cursor` (revision_processor.rs:4090)

#### `table_has_revision_marks`  (L3054-3061)
`fn table_has_revision_marks(dom: &Dom, tbl: NodeId) -> bool`

- **Does:** True when a table subtree carries row/cell/run revision marks.
- **Calls (crate):** `element_or_desc_has_name` (revision_processor.rs:183) L3056
- **Calls (std/external/unresolved):** `W::name`
- **Called by:** `subtree_needs_adjacent_table_merge::{fn walk}` (revision_processor.rs:3137); `merge_adjacent_tables` (revision_processor.rs:3198)

#### `tbl_pr_ex_rank`  (L3087-3092)
`fn tbl_pr_ex_rank(dom: &Dom, e: NodeId) -> Option<usize>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L3088; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L3089; `XName::local_name` (xmllinq/mod.rs:208) (by name) L3090
- **Calls (std/external/unresolved):** `.flatten`, `.iter`, `.position`, `.then`
- **Called by:** `own_tbl_pr_ex` (revision_processor.rs:2948); `row_tbl_pr_ex` (revision_processor.rs:2971); `whole_table_key` (revision_processor.rs:3098)

#### `whole_table_key`  (L3098-3113)
`fn whole_table_key(dom: &Dom, tbl: NodeId) -> String`

- **Does:** A table's whole-table properties (its `tblPr` minus what `tblPrEx` can carry), serialized: Word joins adjacent tables whose keys agree.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L3100; `Dom::elements` (xmllinq/mod.rs:640) L3101,3107,3108; `tbl_pr_ex_rank` (revision_processor.rs:3087) L3102; `Dom::serialize_element` (xmllinq/mod.rs:1315) L3103,3109
- **Calls (std/external/unresolved):** `.is_none`, `.push_str`, `Some`, `String::from`, `W::name`, `W::tbl_pr`
- **Called by:** `table_merge_key` (revision_processor.rs:3116)

#### `table_merge_key`  (L3116-3132)
`fn table_merge_key(dom: &Dom, e: NodeId, mode: TableMerge) -> String`

- **Does:** The grouping key of A.8 for `e` (empty: not a table).
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L3117; `whole_table_key` (revision_processor.rs:3098) L3121; `Dom::elements` (xmllinq/mod.rs:640) L3123; `Dom::element` (xmllinq/mod.rs:655) L3126
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_some`, `.to_string`, `Some`, `String::new`, `W::name`, `W::tbl_pr`, `W::tbl`
- **Called by:** `subtree_needs_adjacent_table_merge::{fn walk}` (revision_processor.rs:3137); `merge_adjacent_tables` (revision_processor.rs:3198)

#### `subtree_needs_adjacent_table_merge`  (L3136-3168)
`fn subtree_needs_adjacent_table_merge(dom: &Dom, root: NodeId, mode: TableMerge) -> bool`

- **Does:** True if any element under `root` has ≥2 adjacent direct `w:tbl` children that A.8 merges in `mode`.
- **Calls (crate):** `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90) L3167
- **Called by:** `merge_adjacent_tables` (revision_processor.rs:3198)

#### `subtree_needs_adjacent_table_merge::{fn walk}`  (L3137-3166)
`fn walk(dom: &Dom, id: NodeId, mode: TableMerge) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L3138; `Dom::elements` (xmllinq/mod.rs:640) L3142; `table_merge_key` (revision_processor.rs:3116) L3146; `table_has_revision_marks` (revision_processor.rs:3054) L3157; `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90) L3165
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_empty`, `String::new`
- **Called by:** `element_has_tracked_revisions` (revision_processor.rs:89); `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90); `element_or_desc_has_name` (revision_processor.rs:183); `element_or_desc_has_name::{fn walk}` (revision_processor.rs:184); `has_deleted_or_moved_from_paragraph_mark` (revision_processor.rs:1966); `has_deleted_or_moved_from_paragraph_mark::{fn walk}` (revision_processor.rs:1967); `subtree_needs_adjacent_table_merge` (revision_processor.rs:3136); `subtree_needs_adjacent_table_merge::{fn walk}` (revision_processor.rs:3137); `has_empty_table_cell` (revision_processor.rs:3384); `has_empty_table_cell::{fn walk}` (revision_processor.rs:3387)

#### `merge_adjacent_tables_transform`  (L3186-3188, pub)
`pub fn merge_adjacent_tables_transform(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** A.8 — `MergeAdjacentTablesTransform` (:464): where an element has direct `w:tbl` children, merge each run of ≥2 adjacent tables sharing the same bidiVisual-kind into one table: `tblPr` from the first, `tblGrid` = the diffs of the union of every member's cumulative grid widths, and each member's rows re-fit (`FixWidths...
- **Calls (crate):** `merge_adjacent_tables` (revision_processor.rs:3198) L3187
- **Called by:** `accept_content` (revision_processor.rs:3481); `revision_boundary_coverage_tests::revised_table_union_preserves_rows_payload_and_unrelated_clean_groups` (revision_processor.rs:4024); `revision_boundary_coverage_tests::nested_merge_clones_widthless_cell_and_keeps_control_identity` (revision_processor.rs:4131); `revision_boundary_coverage_tests::table_merge_identity_respects_whole_properties_grid_history_and_bidi` (revision_processor.rs:4196)

#### `merge_adjacent_tables_like_word`  (L3194-3196)
`fn merge_adjacent_tables_like_word(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** R8 — Word's save after Accept / Reject All: adjacent tables alike in every whole-table property are one table in Word's model (72cc9f4ac6, 3d4318d7e9); a later member's own `tblPrEx`-able properties ride on its rows.
- **Calls (crate):** `merge_adjacent_tables` (revision_processor.rs:3198) L3195
- **Called by:** `tidy` (revision_processor/word_save.rs:25); `revision_boundary_coverage_tests::word_save_merges_row_exceptions_with_row_values_taking_precedence` (revision_processor.rs:4155); `revision_boundary_coverage_tests::table_merge_identity_respects_whole_properties_grid_history_and_bidi` (revision_processor.rs:4196)

#### `merge_adjacent_tables`  (L3198-3378)
`fn merge_adjacent_tables(dom: &mut Dom, node: NodeId, mode: TableMerge) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `subtree_needs_adjacent_table_merge` (revision_processor.rs:3136) L3200; `Dom::is_element` (xmllinq/mod.rs:502) L3203; `Dom::name` (xmllinq/mod.rs:526) L3206,3301,3302,3335,3345,3355...; `Dom::element` (xmllinq/mod.rs:655) L3208,3311,3313; `Dom::new_element` (xmllinq/mod.rs:478) L3209,3223,3261,3266,3269,3283...; `Dom::attributes` (xmllinq/mod.rs:899) L3210,3224,3284; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L3211,3225,3270,3285,3342; `Dom::nodes` (xmllinq/mod.rs:604) L3213; `merge_adjacent_tables` (revision_processor.rs:3198) L3214; `Dom::add` (xmllinq/mod.rs:1074) L3215,3231,3239,3264,3271,3273...; `Dom::elements` (xmllinq/mod.rs:640) L3220,3247,3248,3262,3295,3300...; `table_merge_key` (revision_processor.rs:3116) L3221; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L3230,3238,3263,3296,3306,3338...; `table_has_revision_marks` (revision_processor.rs:3054) L3236; `Dom::attribute` (xmllinq/mod.rs:874) L3249,3314; `own_tbl_pr_ex` (revision_processor.rs:2948) L3277; `fix_widths` (revision_processor.rs:3011) L3281; `wrapped_children` (revision_processor.rs:2992) L3282,3310; `row_tbl_pr_ex` (revision_processor.rs:2971) L3288; `order_tc_pr` (revision_processor.rs:2802) L3345; `Dom::replace_with` (xmllinq/mod.rs:1165) L3361,3363
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.chain`, `.clone`, `.collect`, `.copied`, `.count`, `.dedup`, `.enumerate`, `.expect`, `.filter`, `.flat_map`, `.into_iter`, `.is_empty`, `.is_none`, `.iter`, `.len`, `.ok`, `.parse`, `.position`, `.push`, `.sort_by_key`, `.sort_unstable`, `.take_while`, `.to_string`, `.unwrap`, `Some`, `Vec::new`, `W::grid_span`, `W::name`, `W::tbl_pr`, `W::tbl`, `W::tc_pr`, `W::tc`, `W::tr_pr`, `W::tr`, `W::val`, `crate::util::group_adjacent`, `std::iter::once`
- **Called by:** `merge_adjacent_tables_transform` (revision_processor.rs:3186); `merge_adjacent_tables_like_word` (revision_processor.rs:3194); `merge_adjacent_tables` (revision_processor.rs:3198)

#### `has_empty_table_cell`  (L3384-3412)
`fn has_empty_table_cell(dom: &Dom, root: NodeId) -> bool`

- **Does:** True if any `w:tc` under `root` has no element children other than `w:tcPr` (the A.9 empty-cell predicate).
- **Calls (crate):** `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90) L3411
- **Calls (std/external/unresolved):** `W::tc_pr`, `W::tc`
- **Called by:** `accept_content` (revision_processor.rs:3481)

#### `has_empty_table_cell::{fn walk}`  (L3387-3410)
`fn walk(dom: &Dom, id: NodeId, tc: &XName, tcpr: &XName) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L3388,3395; `Dom::child_count` (xmllinq/mod.rs:622) L3393,3404; `Dom::child_at` (xmllinq/mod.rs:632) L3394,3405; `Dom::is_element` (xmllinq/mod.rs:502) L3395; `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90) L3405
- **Calls (std/external/unresolved):** `.is_some_and`
- **Called by:** `element_has_tracked_revisions` (revision_processor.rs:89); `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90); `element_or_desc_has_name` (revision_processor.rs:183); `element_or_desc_has_name::{fn walk}` (revision_processor.rs:184); `has_deleted_or_moved_from_paragraph_mark` (revision_processor.rs:1966); `has_deleted_or_moved_from_paragraph_mark::{fn walk}` (revision_processor.rs:1967); `subtree_needs_adjacent_table_merge` (revision_processor.rs:3136); `subtree_needs_adjacent_table_merge::{fn walk}` (revision_processor.rs:3137); `has_empty_table_cell` (revision_processor.rs:3384); `has_empty_table_cell::{fn walk}` (revision_processor.rs:3387)

#### `add_empty_paragraph_to_any_empty_cells`  (L3420-3434, pub)
`pub fn add_empty_paragraph_to_any_empty_cells(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** A.9 — `AddEmptyParagraphToAnyEmptyCells` (:1448): a `w:tc` with no element children other than `w:tcPr` gains an empty `w:p`.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L3421; `collect_empty_table_cells` (revision_processor.rs:3436) L3428; `Dom::new_element` (xmllinq/mod.rs:478) L3430; `Dom::add` (xmllinq/mod.rs:1074) L3431
- **Calls (std/external/unresolved):** `Vec::new`, `W::p`, `W::tc_pr`, `W::tc`
- **Called by:** `accept_content` (revision_processor.rs:3481)

#### `collect_empty_table_cells`  (L3436-3464)
`fn collect_empty_table_cells( dom: &Dom, id: NodeId, tc: &XName, tcpr: &XName, out: &mut Vec<NodeId>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L3443,3449; `Dom::child_count` (xmllinq/mod.rs:622) L3447,3458; `Dom::child_at` (xmllinq/mod.rs:632) L3448,3459; `Dom::is_element` (xmllinq/mod.rs:502) L3449,3460; `collect_empty_table_cells` (revision_processor.rs:3436) L3461
- **Calls (std/external/unresolved):** `.is_some_and`, `.push`
- **Called by:** `add_empty_paragraph_to_any_empty_cells` (revision_processor.rs:3420); `collect_empty_table_cells` (revision_processor.rs:3436)

#### `accept_revisions_for_part_content`  (L3477-3479, pub)
`pub fn accept_revisions_for_part_content(dom: &mut Dom, root: NodeId) -> NodeId`

- **Does:** A.10 — `AcceptRevisionsForPart` (:1314) at content scope: the exact 15-step transform order.
- **Calls (crate):** `accept_content` (revision_processor.rs:3481) L3478
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `document_text_view` (debug.rs:2181); `accept_revisions_document` (revision_processor.rs:495); `revision_boundary_coverage_tests::internally_deleted_equations_leave_no_math_shell_but_keep_live_and_empty_math` (revision_processor.rs:4460); `revision_boundary_coverage_tests::display_math_preserves_independent_empty_equation_and_mixed_run_payload` (revision_processor.rs:4540)

#### `accept_content`  (L3481-3560)
`fn accept_content( dom: &mut Dom, root: NodeId, keep_comments: bool, kept: &mut Vec<String>, ) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `element_has_tracked_revisions` (revision_processor.rs:89) L3487; `remove_rsid_transform` (markup_simplifier.rs:154) L3488; `normalize_orphan_moves` (revision_processor.rs:1174) L3497; `keep_wholly_deleted_comments` (revision_processor/comments.rs:96) L3499; `drop_wholly_deleted_comments` (revision_processor/comments.rs:71) L3501; `drop_wholly_deleted_bookmarks` (revision_processor/bookmarks.rs:23) L3502; `drop_wholly_deleted_permissions` (revision_processor.rs:2275) L3503; `fix_up_deleted_or_inserted_field_codes_transform` (revision_processor.rs:1051) L3504; `Dom::descendants` (xmllinq/mod.rs:668) L3505,3554; `accept_move_from_move_to_transform` (revision_processor.rs:128) L3507; `accept_move_from_ranges` (revision_processor.rs:1329) L3511; `accept_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1550) L3512; `accept_deleted_and_moved_from_content_controls` (revision_processor.rs:1647) L3513; `accept_deleted_and_move_from_paragraph_marks` (revision_processor.rs:2726) L3514; `remove_rows_left_empty_by_move_from` (revision_processor.rs:2760) L3516; `accept_all_other_revisions_transform` (revision_processor.rs:228) L3521; `accept_deleted_cells_transform` (revision_processor.rs:2838) L3525; `merge_adjacent_tables_transform` (revision_processor.rs:3186) L3526; `drop_unpaired_bookmarks` (revision_processor/bookmarks.rs:152) L3527; `has_empty_table_cell` (revision_processor.rs:3384) L3533; `add_empty_paragraph_to_any_empty_cells` (revision_processor.rs:3420) L3534; `Dom::name` (xmllinq/mod.rs:526) L3541; `ensure_comment_paragraphs` (revision_processor.rs:3588) L3542; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L3548; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L3549,3550; `Dom::has_elements` (xmllinq/mod.rs:862) L3555; `Dom::remove` (xmllinq/mod.rs:1118) L3556
- **Calls (std/external/unresolved):** `.expect`, `.extend`, `.is_empty`, `.len`, `PT::run_ids`, `PT::unique_id`, `Some`, `W::move_from`, `W::name`, `W::num_pr`
- **Macros:** `debug_assert_eq!`
- **Called by:** `reject_content` (revision_processor.rs:726); `accept_revisions_for_part_content` (revision_processor.rs:3477); `resolve_with_comments` (revision_processor.rs:3871)

#### `carry_kept_revisions`  (L3568-3584)
`fn carry_kept_revisions(dom: &mut Dom, live: NodeId, reverted: NodeId, first: bool)`

- **Does:** A property element `live` reverting to its saved copy `reverted` keeps the revisions left tracked in it (frozen by a selective resolution): a paragraph mark's insertion or deletion ahead of the run properties (`first`), a row's or a cell's marks after the rest.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L3569; `Dom::name` (xmllinq/mod.rs:526) L3572; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L3572; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L3574; `Dom::add_first` (xmllinq/mod.rs:1101) L3577; `Dom::add` (xmllinq/mod.rs:1074) L3581
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_some_and`, `.map`, `.rev`
- **Called by:** `reject_revisions_for_part_transform` (revision_processor.rs:612)

#### `ensure_comment_paragraphs`  (L3588-3600)
`fn ensure_comment_paragraphs(dom: &mut Dom, root: NodeId)`

- **Does:** Give every `w:comment` under a `w:comments` root without a block child (`w:p`, `w:tbl`, `w:sdt`, `w:customXml`) an empty paragraph (#1223).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L3590,3591; `Dom::name` (xmllinq/mod.rs:526) L3594; `Dom::new_element` (xmllinq/mod.rs:478) L3596; `Dom::add` (xmllinq/mod.rs:1074) L3597
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.into_iter`, `.is_some_and`, `Some`, `W::name`, `W::p`, `W::sdt`, `W::tbl`
- **Called by:** `accept_content` (revision_processor.rs:3481)

#### `accept_revisions_for_styles_transform`  (L3604-3622)
`fn accept_revisions_for_styles_transform(dom: &mut Dom, node: NodeId) -> Option<NodeId>`

- **Does:** A.11 — `AcceptRevisionsForStylesTransform` (:1300): drop `pPrChange`/ `rPrChange` from the styles part, rebuild the rest.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L3605; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L3606; `Dom::name` (xmllinq/mod.rs:526) L3608; `Dom::new_element` (xmllinq/mod.rs:478) L3612; `Dom::attributes` (xmllinq/mod.rs:899) L3613; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L3614; `Dom::nodes` (xmllinq/mod.rs:604) L3616; `accept_revisions_for_styles_transform` (revision_processor.rs:3604) L3617; `Dom::add` (xmllinq/mod.rs:1074) L3618
- **Calls (std/external/unresolved):** `.unwrap`, `Some`, `W::p_pr_change`, `W::r_pr_change`
- **Called by:** `accept_revisions_for_styles_transform` (revision_processor.rs:3604); `resolve_with_comments` (revision_processor.rs:3871); `revision_boundary_coverage_tests::missing_saved_style_properties_drop_changed_block_while_accept_keeps_live_properties` (revision_processor.rs:4688)

#### `reject_revisions_for_styles_transform`  (L3628-3655)
`fn reject_revisions_for_styles_transform(dom: &mut Dom, node: NodeId) -> Option<NodeId>`

- **Does:** A.11 — `RejectRevisionsForStylesTransform` (:391): a `pPr` holding a `pPrChange` reverts to the change's inner `pPr` (same for `rPr`); a change element without its inner properties drops the whole node (C# recurses null).
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L3629; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L3630; `Dom::name` (xmllinq/mod.rs:526) L3632; `Dom::element` (xmllinq/mod.rs:655) L3634,3636,3640,3642; `reject_revisions_for_styles_transform` (revision_processor.rs:3628) L3637,3643,3650; `Dom::new_element` (xmllinq/mod.rs:478) L3645; `Dom::attributes` (xmllinq/mod.rs:899) L3646; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L3647; `Dom::nodes` (xmllinq/mod.rs:604) L3649; `Dom::add` (xmllinq/mod.rs:1074) L3651
- **Calls (std/external/unresolved):** `.and_then`, `.unwrap`, `Some`, `W::p_pr_change`, `W::p_pr`, `W::r_pr_change`, `W::r_pr`
- **Called by:** `reject_revisions_for_styles_transform` (revision_processor.rs:3628); `resolve_with_comments` (revision_processor.rs:3871); `revision_boundary_coverage_tests::missing_saved_style_properties_drop_changed_block_while_accept_keeps_live_properties` (revision_processor.rs:4688)

#### `related_part`  (L3659-3669)
`fn related_part(pkg: &crate::opc::PartFs, rel: &str) -> Option<String>`

- **Does:** The part `rel` of the main part targets (`numbering`, `comments`), resolved against the main part; `None` when the package has none.
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:1070) L3660; `PartFs::read_rels_for` (opc/mod.rs:991) L3661; `PartFs::resolve_rel_target` (opc/mod.rs:829) L3668
- **Calls (std/external/unresolved):** `.as_deref`, `.find`, `.iter`, `.map`, `.next`, `.rsplit`, `Some`
- **Called by:** `numbering_part` (revision_processor.rs:3672); `comments_part` (revision_processor.rs:3679)

#### `numbering_part`  (L3672-3674)
`fn numbering_part(pkg: &crate::opc::PartFs) -> Option<String>`

- **Does:** The main document's numbering part, when it has one.
- **Calls (crate):** `related_part` (revision_processor.rs:3659) L3673
- **Called by:** `resolve_with_comments` (revision_processor.rs:3871)

#### `comments_part`  (L3679-3681, pub(crate))
`pub(crate) fn comments_part(pkg: &crate::opc::PartFs) -> Option<String>`

- **Does:** The main document's comments part (`word/comments.xml`), when it has one.
- **Calls (crate):** `related_part` (revision_processor.rs:3659) L3680
- **Called by:** `for_each_story` (changes.rs:657); `comment_changes_in` (changes.rs:1003); `decisions` (edit/source_refs.rs:25); `resolve_with_comments` (revision_processor.rs:3871)

#### `revision_bearing_parts`  (L3687-3728, pub(crate))
`pub(crate) fn revision_bearing_parts(pkg: &crate::opc::PartFs) -> Vec<(String, bool)>`

- **Does:** The parts `AcceptRevisions`/`RejectRevisions` (:1277/:31) walk, in the C# order — main, headers, footers, endnotes, footnotes, then styles (flagged) — with the comments part Word's own Accept/Reject All also settles (#1223) between the notes and the styles.
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:1070) L3688; `PartFs::read_rels_for` (opc/mod.rs:991) L3697; `PartFs::resolve_rel_target` (opc/mod.rs:829) L3711
- **Calls (std/external/unresolved):** `.as_deref`, `.chain`, `.into_iter`, `.next`, `.push`, `.rsplit`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `Some`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `audit_stories` (changes.rs:168); `tolerant` (changes.rs:620); `for_each_story` (changes.rs:657); `decisions` (edit/source_refs.rs:25); `first_free_id` (edit/tracked.rs:427); `SourceIndex::new` (edit/v2_apply.rs:713); `Transaction::finish` (edit.rs:6598); `RevisionIds::new` (revision_processor.rs:3777); `resolve_with_comments` (revision_processor.rs:3871); `remove_comments` (scrub.rs:221)

#### `process_part`  (L3730-3747)
`fn process_part<F>(pkg: &mut crate::opc::PartFs, part: &str, f: F) where F: FnOnce(&mut Dom, NodeId) -> Option<NodeId>,`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L3734; `Dom::new` (xmllinq/mod.rs:387) L3737; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L3738; `Dom::root` (xmllinq/mod.rs:822) L3739; `Dom::replace_with` (xmllinq/mod.rs:1165) L3745; `PartFs::set_part` (opc/mod.rs:661) L3746; `Dom::serialize_document` (xmllinq/mod.rs:1338) L3746
- **Calls (std/external/unresolved):** `.into_bytes`, `f`
- **Called by:** `resolve_with_comments` (revision_processor.rs:3871)

#### `accept_revisions_package`  (L3752-3754, pub)
`pub fn accept_revisions_package(pkg: &mut crate::opc::PartFs)`

- **Does:** A.11 — `AcceptRevisions` (:1277) at package scope: run the full part pipeline over main + headers + footers + endnotes + footnotes, and the styles transform over the styles part.
- **Calls (crate):** `resolve_package` (revision_processor.rs:3850) L3753
- **Called by:** `accept_revisions` (document_comparer.rs:4961); `accept_source_revisions` (document_comparer.rs:4972)

#### `reject_revisions_package`  (L3761-3763, pub)
`pub fn reject_revisions_package(pkg: &mut crate::opc::PartFs)`

- **Does:** A.11 — `RejectRevisions` (:31) at package scope: per content part, the revert → reverse → rsid-strip → full-accept composition (the C# phases the same steps across all parts; parts are independent, so per-part composition is equivalent); the styles part reverts its property changes then accepts the leftovers.
- **Calls (crate):** `resolve_package` (revision_processor.rs:3850) L3762
- **Called by:** `reject_revisions` (document_comparer.rs:4984)

#### `RevisionIds::new`  (L3777-3799, pub(crate))
`pub(crate) fn new(pkg: &crate::opc::PartFs) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `revision_bearing_parts` (revision_processor.rs:3687) L3779; `PartFs::part_string` (opc/mod.rs:636) L3780; `Dom::new` (xmllinq/mod.rs:387) L3783; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L3784; `Dom::root` (xmllinq/mod.rs:822) L3785; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L3786; `Dom::name` (xmllinq/mod.rs:526) L3787; `has_unique_revision_id` (validate.rs:1684) L3789; `Dom::attribute` (xmllinq/mod.rs:874) L3790; `RevisionIds::from_used` (revision_processor.rs:3801) L3798
- **Calls (std/external/unresolved):** `.and_then`, `.insert`, `.is_some_and`, `.ok`, `.parse`, `HashSet::new`, `W::id`
- **Called by:** `rewrite` (edit/whole.rs:130); `resolve_with_comments` (revision_processor.rs:3871)

#### `RevisionIds::from_used`  (L3801-3804, pub(crate))
`pub(crate) fn from_used(used: HashSet<u64>) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.iter`, `.map_or`, `.max`, `.saturating_add`
- **Called by:** `tests::run` (edit/whole.rs:499); `RevisionIds::new` (revision_processor.rs:3777)

#### `RevisionIds::allocate`  (L3808-3821, pub(crate))
`pub(crate) fn allocate(&mut self) -> u64`

- **Does:** Keep max + 1 until Word's signed 32-bit range is exhausted, then allocate the lowest unused id.
- **Calls (std/external/unresolved):** `.contains`, `.insert`
- **Macros:** `assert!`
- **Called by:** `Stamp::container` (edit/whole.rs:202); `rewrite_one` (edit/whole.rs:229); `clone_with_fresh_property_ids` (revision_processor.rs:3826)

#### `clone_with_fresh_property_ids`  (L3826-3841, pub(crate))
`pub(crate) fn clone_with_fresh_property_ids( dom: &mut Dom, source: NodeId, next_id: &mut RevisionIds, ) -> NodeId`

- **Does:** A copied property history is a distinct record.
- **Calls (crate):** `Dom::clone_subtree` (xmllinq/mod.rs:1231) L3831; `Dom::descendants` (xmllinq/mod.rs:668) L3832; `Dom::name` (xmllinq/mod.rs:526) L3833; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L3834; `XName::local_name` (xmllinq/mod.rs:208) (by name) L3835; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L3837; `RevisionIds::allocate` (revision_processor.rs:3808) L3837
- **Calls (std/external/unresolved):** `.chain`, `.is_some_and`, `.to_string`, `Some`, `W::id`, `std::iter::once`
- **Macros:** `matches!`
- **Called by:** `rewrite_one` (edit/whole.rs:229); `effective_rpr` (revision_processor/style_records.rs:585)

#### `resolve_package`  (L3850-3856, pub(crate))
`pub(crate) fn resolve_package( pkg: &mut crate::opc::PartFs, resolution: Resolution, freeze: Option<Freeze<'_>>, ) -> Vec<String>`

- **Does:** Accept or reject every revision `freeze` leaves in place.
- **Calls (crate):** `resolve_with_comments` (revision_processor.rs:3871) L3855
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `resolve_for_edit_or_cli` (changes.rs:922); `accept_revisions_package` (revision_processor.rs:3752); `reject_revisions_package` (revision_processor.rs:3761)

#### `resolve_package_for_edit`  (L3860-3869, pub(crate))
`pub(crate) fn resolve_package_for_edit( pkg: &mut crate::opc::PartFs, resolution: Resolution, freeze: Option<Freeze<'_>>, keep_comments: bool, ) -> (Vec<String>, Vec<String>)`

- **Does:** [`resolve_package`] for an edit plan, the only entry point that can rescue removed anchors.
- **Calls (crate):** `resolve_with_comments` (revision_processor.rs:3871) L3867
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `resolve_for_edit_or_cli` (changes.rs:922)

#### `resolve_with_comments`  (L3871-3939)
`fn resolve_with_comments( pkg: &mut crate::opc::PartFs, resolution: Resolution, freeze: Option<Freeze<'_>>, keep_comments: bool, kept_comments: &mut Vec<String>, ) -> Vec<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `revision_bearing_parts` (revision_processor.rs:3687) L3878; `RevisionIds::new` (revision_processor.rs:3777) L3879; `numbering_part` (revision_processor.rs:3672) L3880; `PartFs::part_string` (opc/mod.rs:636) L3881; `Levels::parse` (revision_processor/style_records.rs:280) L3882; `comments_part` (revision_processor.rs:3679) L3888; `process_part` (revision_processor.rs:3730) L3892; `element_has_tracked_revisions` (revision_processor.rs:89) L3893; `accept_revisions_for_styles_transform` (revision_processor.rs:3604) L3901,3912; `recorded_blocks` (revision_processor/style_records.rs:368) L3903; `reject_revisions_for_styles_transform` (revision_processor.rs:3628) L3904; `restore_against_built_ins` (revision_processor/style_records.rs:398) L3905; `carry_vanishing_section_references` (revision_processor/sections.rs:31) L3915; `accept_content` (revision_processor.rs:3481) L3918; `reject_content` (revision_processor.rs:726) L3921; `thaw` (revision_processor.rs:3942) L3926; `story_parts` (revision_processor.rs:3957) L3930; `prune_orphan_comments` (revision_processor/comments.rs:274) L3931; `prune_orphan_notes` (revision_processor/notes.rs:24) L3932; `tidy` (revision_processor/word_save.rs:25) L3937
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.clone`, `.collect`, `.filter`, `.into_iter`, `.map`, `.unwrap_or_default`, `Some`, `freeze`
- **Called by:** `resolve_package` (revision_processor.rs:3850); `resolve_package_for_edit` (revision_processor.rs:3860)

#### `thaw`  (L3942-3954)
`fn thaw(dom: &mut Dom, root: NodeId)`

- **Does:** Rename every frozen revision element under `root` back into `w:`.
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L3943; `Dom::name` (xmllinq/mod.rs:526) L3947; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L3948; `XName::local_name` (xmllinq/mod.rs:208) (by name) L3948; `Dom::set_name` (xmllinq/mod.rs:539) L3952
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.into_iter`, `.then`, `.to_string`, `W::name`
- **Called by:** `resolve_with_comments` (revision_processor.rs:3871)

#### `story_parts`  (L3957-3963)
`fn story_parts(parts: &[(String, bool)]) -> Vec<String>`

- **Does:** The content parts of [`revision_bearing_parts`] (styles left out).
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.filter`, `.iter`, `.map`
- **Called by:** `resolve_with_comments` (revision_processor.rs:3871)

## Test functions (55)

- `revision_boundary_coverage_tests::parse` (L3970): _no doc comment_
- `revision_boundary_coverage_tests::paragraph` (L3980): _no doc comment_
- `revision_boundary_coverage_tests::cell` (L3984): _no doc comment_
- `revision_boundary_coverage_tests::table` (L3991): _no doc comment_
- `revision_boundary_coverage_tests::local_children` (L4001): _no doc comment_
- `revision_boundary_coverage_tests::widths` (L4008): _no doc comment_
- `revision_boundary_coverage_tests::span` (L4016): _no doc comment_
- `revision_boundary_coverage_tests::revised_table_union_preserves_rows_payload_and_unrelated_clean_groups` (L4024): _no doc comment_
- `revision_boundary_coverage_tests::fix_widths_clamps_spans_and_leaves_widthless_cells_out_of_cursor` (L4090): _no doc comment_
- `revision_boundary_coverage_tests::nested_merge_clones_widthless_cell_and_keeps_control_identity` (L4131): _no doc comment_
- `revision_boundary_coverage_tests::word_save_merges_row_exceptions_with_row_values_taking_precedence` (L4155): _no doc comment_
- `revision_boundary_coverage_tests::table_merge_identity_respects_whole_properties_grid_history_and_bidi` (L4196): _no doc comment_
- `revision_boundary_coverage_tests::accepting_deleted_cells_keeps_anchor_payload_and_expands_existing_span` (L4238): _no doc comment_
- `revision_boundary_coverage_tests::deleted_cell_anchor_defaults_bad_span_and_orders_all_schema_properties` (L4274): _no doc comment_
- `revision_boundary_coverage_tests::custom_xml_deleted_control_unwraps_nested_controls_but_retains_order_and_anchors` (L4329): _no doc comment_
- `revision_boundary_coverage_tests::custom_xml_move_deletes_whole_control_and_plain_content_but_keeps_outside_control` (L4360): _no doc comment_
- `revision_boundary_coverage_tests::unmatched_custom_ranges_leave_control_payload_and_parent_links_untouched` (L4381): _no doc comment_
- `revision_boundary_coverage_tests::move_range_that_enters_table_keeps_container_properties_and_surviving_content` (L4401): _no doc comment_
- `revision_boundary_coverage_tests::accept_cell_merge_converts_restart_and_continue_only_inside_cell_properties` (L4424): _no doc comment_
- `revision_boundary_coverage_tests::internally_deleted_equations_leave_no_math_shell_but_keep_live_and_empty_math` (L4460): _no doc comment_
- `revision_boundary_coverage_tests::display_math_preserves_independent_empty_equation_and_mixed_run_payload` (L4540): _no doc comment_
- `revision_boundary_coverage_tests::internally_deleted_math_does_not_consume_independent_anchors_controls_or_empty_branches` (L4561): _no doc comment_
- `revision_boundary_coverage_tests::internally_deleted_math_does_not_consume_independent_anchors_controls_or_empty_branches::{fn semantic}` (L4562): _no doc comment_
- `revision_boundary_coverage_tests::wholly_deleted_math_hoists_owned_range_metadata_instead_of_discarding_it` (L4623): _no doc comment_
- `revision_boundary_coverage_tests::accept_deleted_fraction_drops_its_math_payload_and_preserves_live_fraction` (L4651): _no doc comment_
- `revision_boundary_coverage_tests::rejecting_structural_markers_drops_inserted_numbering_and_cell_without_losing_neighbors` (L4665): _no doc comment_
- `revision_boundary_coverage_tests::missing_saved_style_properties_drop_changed_block_while_accept_keeps_live_properties` (L4688): _no doc comment_
- `revision_boundary_coverage_tests::acceptance_removes_empty_numbering_but_keeps_live_numbering_and_clears_atom_annotations` (L4722): _no doc comment_
- `revision_boundary_coverage_tests::unmatched_or_contentless_move_ranges_preserve_identity_and_properties` (L4746): _no doc comment_
- `public_block_control_ownership_tests::package` (L4774): _no doc comment_
- `public_block_control_ownership_tests::resolved` (L4796): _no doc comment_
- `public_block_control_ownership_tests::assert_owner` (L4808): _no doc comment_
- `public_block_control_ownership_tests::public_mark_resolution_rewraps_transformed_paragraphs_with_original_control_metadata` (L4829): _no doc comment_
- `public_block_control_ownership_tests::public_mark_resolution_preserves_whole_table_payload_at_the_controls_block_level` (L4862): _no doc comment_
- `public_block_control_ownership_tests::nested_block_controls_keep_each_owner_and_transformed_paragraph_payload` (L4884): _no doc comment_
- `public_block_control_ownership_tests::nested_block_controls_keep_outer_table_owner_and_nested_cell_geometry` (L4958): _no doc comment_
- `body_equation_paragraph_mark_ownership_tests::equation` (L5014): _no doc comment_
- `body_equation_paragraph_mark_ownership_tests::semantic` (L5024): _no doc comment_
- `body_equation_paragraph_mark_ownership_tests::parse` (L5052): _no doc comment_
- `body_equation_paragraph_mark_ownership_tests::assert_math` (L5058): _no doc comment_
- `body_equation_paragraph_mark_ownership_tests::equation_is_an_opaque_block_barrier_for_deleted_paragraph_marks` (L5082): _no doc comment_
- `body_equation_paragraph_mark_ownership_tests::public_accept_and_reject_preserve_unchanged_body_math_among_revised_marks` (L5099): _no doc comment_
- `body_equation_paragraph_mark_ownership_tests::terminal_internal_equation_revisions_preserve_exact_surviving_paragraph_owners` (L5138): _no doc comment_
- `body_equation_paragraph_mark_ownership_tests::terminal_internal_equation_revisions_preserve_exact_surviving_paragraph_owners::{fn normalize_empty_properties}` (L5139): _no doc comment_
- `deleted_title_equation_barrier_tests::equation` (L5233): _no doc comment_
- `deleted_title_equation_barrier_tests::title` (L5250): _no doc comment_
- `deleted_title_equation_barrier_tests::parse` (L5271): _no doc comment_
- `deleted_title_equation_barrier_tests::semantic` (L5277): _no doc comment_
- `deleted_title_equation_barrier_tests::deleted_title_before_equation_is_removed_without_consuming_the_equation` (L5306): _no doc comment_
- `deleted_title_equation_barrier_tests::public_replaced_titles_and_equations_recover_complete_owned_sources` (L5323): _no doc comment_
- `complete_field_instruction_metadata_boundary_tests::repaired_complete_field_codes_preserve_source_comments_processing_instructions_and_formats` (L5384): _no doc comment_
- `alternate_content_paragraph_owner_tests::root` (L5465): _no doc comment_
- `alternate_content_paragraph_owner_tests::semantic` (L5470): _no doc comment_
- `alternate_content_paragraph_owner_tests::authored_property_only_alternatives_survive_but_wholly_deleted_wrappers_do_not` (L5498): _no doc comment_
- `alternate_content_paragraph_owner_tests::surviving_alternate_content_keeps_both_presentations_across_deleted_paragraph_marks` (L5544): _no doc comment_
