# Functions and methods in `src/revision_processor.rs`

112 non-test, 55 test. Part of the [symbol manifest](../README.md).

#### `tracked_revisions_elements`  (L79-81, pub)
`pub fn tracked_revisions_elements() -> Vec<XName>`

- **Does:** The element names whose presence indicates the document carries tracked revisions (one per local name in the `w:` namespace).
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`, `W::name`
- **Called by:** `decisions` (edit/source_refs.rs:25); `has_history` (edit/v2_apply.rs:659); `SourceIndex::add_revision_part` (edit/v2_apply.rs:753)

#### `element_has_tracked_revisions`  (L89-105, pub)
`pub fn element_has_tracked_revisions(dom: &Dom, root: NodeId) -> bool`

- **Does:** True if `root` (or any descendant) is a tracked-revision element.
- **Calls (crate):** `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90) L104
- **Called by:** `is_restorable_layout_blank` (comparer/finalize.rs:3859); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215); `accept_move_from_move_to_transform` (revision_processor.rs:128); `accept_all_other_revisions_transform` (revision_processor.rs:377); `accept_revisions_for_element` (revision_processor.rs:481); `reject_content` (revision_processor.rs:755); `accept_content` (revision_processor.rs:3510); `resolve_with_comments` (revision_processor.rs:3900); `public_block_control_ownership_tests::nested_block_controls_keep_each_owner_and_transformed_paragraph_payload` (revision_processor.rs:4913); `public_block_control_ownership_tests::nested_block_controls_keep_outer_table_owner_and_nested_cell_geometry` (revision_processor.rs:4987)

#### `element_has_tracked_revisions::{fn walk}`  (L90-103)
`fn walk(dom: &Dom, id: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L91; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L92; `XName::local_name` (xmllinq/mod.rs:208) (by name) L93; `Dom::child_count` (xmllinq/mod.rs:622) L97; `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90) L98; `Dom::child_at` (xmllinq/mod.rs:632) L98
- **Calls (std/external/unresolved):** `.contains`
- **Called by:** `element_has_tracked_revisions` (revision_processor.rs:89); `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90); `element_or_desc_has_name` (revision_processor.rs:183); `element_or_desc_has_name::{fn walk}` (revision_processor.rs:184); `has_deleted_or_moved_from_paragraph_mark` (revision_processor.rs:1995); `has_deleted_or_moved_from_paragraph_mark::{fn walk}` (revision_processor.rs:1996); `subtree_needs_adjacent_table_merge` (revision_processor.rs:3165); `subtree_needs_adjacent_table_merge::{fn walk}` (revision_processor.rs:3166); `has_empty_table_cell` (revision_processor.rs:3413); `has_empty_table_cell::{fn walk}` (revision_processor.rs:3416)

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
- **Called by:** `removed_container` (revision_processor/comments.rs:240); `try_accept_math_fraction` (revision_processor.rs:288); `try_accept_deleted_row` (revision_processor.rs:301)

#### `accept_move_from_move_to_transform`  (L128-171, pub)
`pub fn accept_move_from_move_to_transform(dom: &mut Dom, node: NodeId) -> Vec<NodeId>`

- **Does:** Port of `AcceptMoveFromMoveToTransform` — unwrap `w:moveTo`, drop `w:moveFrom`.
- **Calls (crate):** `element_has_tracked_revisions` (revision_processor.rs:89) L130; `take_subtree` (revision_processor.rs:175) L131,134,153; `Dom::is_element` (xmllinq/mod.rs:502) L133; `Dom::name` (xmllinq/mod.rs:526) L136,149,150; `Dom::nodes` (xmllinq/mod.rs:604) L139,165; `accept_move_from_move_to_transform` (revision_processor.rs:128) L140,166; `Dom::parent` (xmllinq/mod.rs:597) L148,150; `hoist_range_markers_from` (revision_processor.rs:2314) L158; `Dom::new_element` (xmllinq/mod.rs:478) L161; `Dom::attributes` (xmllinq/mod.rs:899) L162; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L163; `Dom::add` (xmllinq/mod.rs:1074) L167
- **Calls (std/external/unresolved):** `.and_then`, `.extend`, `.is_some_and`, `.unwrap`, `Some`, `Vec::new`, `W::move_from`, `W::move_to`, `W::p_pr`, `W::r_pr`
- **Macros:** `vec!`
- **Called by:** `accept_move_from_move_to_transform` (revision_processor.rs:128); `accept_revisions_for_element` (revision_processor.rs:481); `accept_content` (revision_processor.rs:3510)

#### `take_subtree`  (L175-180)
`fn take_subtree(dom: &mut Dom, node: NodeId) -> Vec<NodeId>`

- **Does:** Detach `node` from its parent (if any) and return it for reparenting.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L176; `Dom::remove` (xmllinq/mod.rs:1118) L177
- **Calls (std/external/unresolved):** `.is_some`
- **Macros:** `vec!`
- **Called by:** `accept_move_from_move_to_transform` (revision_processor.rs:128); `accept_all_other_revisions_transform` (revision_processor.rs:377)

#### `element_or_desc_has_name`  (L183-197)
`fn element_or_desc_has_name(dom: &Dom, root: NodeId, name: &XName) -> bool`

- **Does:** Non-allocating presence of `name` on `root` or any descendant element.
- **Calls (crate):** `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90) L196
- **Called by:** `fix_up_deleted_or_inserted_field_codes_transform` (revision_processor.rs:1080); `accept_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1579); `accept_deleted_cells_transform` (revision_processor.rs:2867); `table_has_revision_marks` (revision_processor.rs:3083)

#### `element_or_desc_has_name::{fn walk}`  (L184-195)
`fn walk(dom: &Dom, id: NodeId, name: &XName) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L185; `Dom::child_count` (xmllinq/mod.rs:622) L188; `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90) L190; `Dom::child_at` (xmllinq/mod.rs:632) L190
- **Calls (std/external/unresolved):** `.as_ref`, `Some`
- **Called by:** `element_has_tracked_revisions` (revision_processor.rs:89); `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90); `element_or_desc_has_name` (revision_processor.rs:183); `element_or_desc_has_name::{fn walk}` (revision_processor.rs:184); `has_deleted_or_moved_from_paragraph_mark` (revision_processor.rs:1995); `has_deleted_or_moved_from_paragraph_mark::{fn walk}` (revision_processor.rs:1996); `subtree_needs_adjacent_table_merge` (revision_processor.rs:3165); `subtree_needs_adjacent_table_merge::{fn walk}` (revision_processor.rs:3166); `has_empty_table_cell` (revision_processor.rs:3413); `has_empty_table_cell::{fn walk}` (revision_processor.rs:3416)

#### `has_run_content`  (L201-214)
`fn has_run_content(dom: &Dom, element: NodeId) -> bool`

- **Does:** True iff a transformed element still carries run/inline content (port of `HasRunContent`) — used to decide whether an emptied `w:hyperlink` survives.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L211; `Dom::name` (xmllinq/mod.rs:526) L213
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.into_iter`, `.is_some_and`, `W::del`, `W::fld_simple`, `W::hyperlink`, `W::ins`, `W::r`, `W::sdt`, `W::smart_tag`
- **Called by:** `try_accept_hyperlink` (revision_processor.rs:345)

#### `try_accept_wholly_revised_equation`  (L216-286)
`fn try_accept_wholly_revised_equation( dom: &mut Dom, node: NodeId, name: &XName, ) -> Option<Vec<NodeId>>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L222,227,231,235; `Dom::name_is` (xmllinq/mod.rs:535) L225,233,237,240,273; `Dom::ancestors` (xmllinq/mod.rs:770) L236; `Dom::name` (xmllinq/mod.rs:526) L244,261; `Dom::text_value` (xmllinq/mod.rs:547) L245,276; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L251; `XName::local_name` (xmllinq/mod.rs:208) (by name) L254,259; `XName::namespace_name` (xmllinq/mod.rs:219) L262; `XName::local_name` (xmllinq/mod.rs:208) L262; `Dom::elements` (xmllinq/mod.rs:640) L265,271; `Dom::nodes` (xmllinq/mod.rs:604) L274; `Dom::is_element` (xmllinq/mod.rs:502) L275; `hoist_range_markers_from` (revision_processor.rs:2314) L282
- **Calls (std/external/unresolved):** `.all`, `.any`, `.collect`, `.ends_with`, `.filter`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.trim`, `M::name`, `Some`, `W::del`
- **Called by:** `accept_all_other_revisions_transform` (revision_processor.rs:377)

#### `try_accept_math_fraction`  (L288-299)
`fn try_accept_math_fraction(dom: &mut Dom, node: NodeId, name: &XName) -> Option<Vec<NodeId>>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L290; `has_path` (revision_processor.rs:113) L293
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `M::name`, `Some`, `W::del`
- **Macros:** `vec!`
- **Called by:** `accept_all_other_revisions_transform` (revision_processor.rs:377)

#### `try_accept_deleted_row`  (L301-306)
`fn try_accept_deleted_row(dom: &mut Dom, node: NodeId, name: &XName) -> Option<Vec<NodeId>>`

- **Does:** _no doc comment_
- **Calls (crate):** `has_path` (revision_processor.rs:113) L302
- **Calls (std/external/unresolved):** `Some`, `W::del`, `W::tr_pr`, `W::tr`
- **Macros:** `vec!`
- **Called by:** `accept_all_other_revisions_transform` (revision_processor.rs:377)

#### `try_accept_deleted_table`  (L308-313)
`fn try_accept_deleted_table(dom: &mut Dom, node: NodeId, name: &XName) -> Option<Vec<NodeId>>`

- **Does:** _no doc comment_
- **Calls (crate):** `table_rows_all_deleted` (revision_processor.rs:1970) L309
- **Calls (std/external/unresolved):** `Some`, `W::tbl`
- **Macros:** `vec!`
- **Called by:** `accept_all_other_revisions_transform` (revision_processor.rs:377)

#### `try_accept_deleted_text`  (L315-320)
`fn try_accept_deleted_text(dom: &mut Dom, node: NodeId, name: &XName) -> Option<Vec<NodeId>>`

- **Does:** _no doc comment_
- **Calls (crate):** `hoist_range_markers_from` (revision_processor.rs:2314) L317
- **Calls (std/external/unresolved):** `Some`, `W::del`
- **Called by:** `accept_all_other_revisions_transform` (revision_processor.rs:377)

#### `try_accept_cell_merge`  (L322-343)
`fn try_accept_cell_merge(dom: &mut Dom, node: NodeId, name: &XName) -> Option<Vec<NodeId>>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L324; `Dom::name` (xmllinq/mod.rs:526) L326; `Dom::attribute` (xmllinq/mod.rs:874) L329; `Dom::new_element` (xmllinq/mod.rs:478) L331,336; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L332,337
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.is_some_and`, `.map`, `.to_string`, `Some`, `W::cell_merge`, `W::tc_pr`, `W::v_merge`, `W::val`
- **Macros:** `vec!`
- **Called by:** `accept_all_other_revisions_transform` (revision_processor.rs:377)

#### `try_accept_hyperlink`  (L345-363)
`fn try_accept_hyperlink(dom: &mut Dom, node: NodeId, name: &XName) -> Option<Vec<NodeId>>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L347; `Dom::attributes` (xmllinq/mod.rs:899) L348; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L349; `Dom::nodes` (xmllinq/mod.rs:604) L351; `accept_all_other_revisions_transform` (revision_processor.rs:377) L352; `Dom::add` (xmllinq/mod.rs:1074) L353; `has_run_content` (revision_processor.rs:201) L356; `hoist_range_markers_from` (revision_processor.rs:2314) L359
- **Calls (std/external/unresolved):** `.clone`, `Some`, `W::hyperlink`
- **Macros:** `vec!`
- **Called by:** `accept_all_other_revisions_transform` (revision_processor.rs:377)

#### `accept_all_other_revisions_transform`  (L377-468, pub)
`pub fn accept_all_other_revisions_transform(dom: &mut Dom, node: NodeId) -> Vec<NodeId>`

- **Does:** Port of `AcceptAllOtherRevisionsTransform` — accept inserts (unwrap `w:ins`), drop deletions and revision-range markers, accept formatting- change markers, handle deleted rows/tables, cell merges, and empty hyperlink shells.
- **Calls (crate):** `element_has_tracked_revisions` (revision_processor.rs:89) L379; `take_subtree` (revision_processor.rs:175) L380,383; `Dom::is_element` (xmllinq/mod.rs:502) L382; `Dom::name` (xmllinq/mod.rs:526) L385; `Dom::nodes` (xmllinq/mod.rs:604) L391,462; `accept_all_other_revisions_transform` (revision_processor.rs:377) L392,463; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L412,431; `XName::local_name` (xmllinq/mod.rs:208) (by name) L412,431; `try_accept_wholly_revised_equation` (revision_processor.rs:216) L435; `try_accept_math_fraction` (revision_processor.rs:288) L438; `try_accept_deleted_row` (revision_processor.rs:301) L441; `try_accept_deleted_table` (revision_processor.rs:308) L444; `try_accept_deleted_text` (revision_processor.rs:315) L447; `try_accept_cell_merge` (revision_processor.rs:322) L450; `try_accept_hyperlink` (revision_processor.rs:345) L453; `Dom::new_element` (xmllinq/mod.rs:478) L458; `Dom::attributes` (xmllinq/mod.rs:899) L459; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L460; `Dom::add` (xmllinq/mod.rs:1074) L464
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.extend`, `.unwrap`, `Some`, `Vec::new`, `W::ins`
- **Macros:** `vec!`
- **Called by:** `try_accept_hyperlink` (revision_processor.rs:345); `accept_all_other_revisions_transform` (revision_processor.rs:377); `accept_revisions_for_element` (revision_processor.rs:481); `all_para_content_is_deleted` (revision_processor.rs:1941); `accept_content` (revision_processor.rs:3510); `revision_boundary_coverage_tests::accept_cell_merge_converts_restart_and_continue_only_inside_cell_properties` (revision_processor.rs:4453); `revision_boundary_coverage_tests::internally_deleted_math_does_not_consume_independent_anchors_controls_or_empty_branches` (revision_processor.rs:4590); `revision_boundary_coverage_tests::wholly_deleted_math_hoists_owned_range_metadata_instead_of_discarding_it` (revision_processor.rs:4652); `revision_boundary_coverage_tests::accept_deleted_fraction_drops_its_math_payload_and_preserves_live_fraction` (revision_processor.rs:4680)

#### `accept_revisions_for_element`  (L481-511, pub)
`pub fn accept_revisions_for_element(dom: &mut Dom, element: NodeId) -> NodeId`

- **Does:** Port of `AcceptRevisionsForElement` (the documented simple-revision entry point, RevisionProcessor.ts:1004): RemoveRsid → AcceptMoveFromMoveTo → AcceptAllOtherRevisions → strip PT.UniqueId/RunIds → drop empty w:numPr.
- **Calls (crate):** `element_has_tracked_revisions` (revision_processor.rs:89) L482; `remove_rsid_transform` (markup_simplifier.rs:154) L483; `accept_move_from_move_to_transform` (revision_processor.rs:128) L485; `accept_all_other_revisions_transform` (revision_processor.rs:377) L488; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L498; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L499,500; `Dom::descendants` (xmllinq/mod.rs:668) L505; `Dom::has_elements` (xmllinq/mod.rs:862) L506; `Dom::remove` (xmllinq/mod.rs:1118) L507
- **Calls (std/external/unresolved):** `.expect`, `.len`, `PT::run_ids`, `PT::unique_id`, `Some`, `W::num_pr`
- **Macros:** `debug_assert_eq!`
- **Called by:** `revision_boundary_coverage_tests::acceptance_removes_empty_numbering_but_keeps_live_numbering_and_clears_atom_annotations` (revision_processor.rs:4751)

#### `accept_revisions_document`  (L524-526, pub)
`pub fn accept_revisions_document(dom: &mut Dom, root: NodeId) -> NodeId`

- **Does:** `AcceptRevisionsDocument` at element scope — accept all tracked revisions in the `root` element's subtree, returning the new root.
- **Calls (crate):** `accept_revisions_for_part_content` (revision_processor.rs:3506) L525
- **Called by:** `coverage_round_two_tests::wholesale_mesh_preserves_each_authored_terminal_whitespace_and_format::{fn source_projection}` (comparer/finalize.rs:16914); `coverage_round_two_tests::wholesale_mesh_without_terminal_whitespace_keeps_exact_supported_word_streams::{fn owned}` (comparer/finalize.rs:16965); `coverage_round_next_tests::trailing_deletion_peel_truth_table_for_shared_long_and_copula_paths::{fn projected_body}` (comparer/finalize.rs:19556); `deep_production_path_tests::title_spacers_require_unique_revised_two_blank_pretable_context` (comparer/finalize.rs:21817); `deep_production_path_tests::restored_title_spacers_preserve_each_revised_blank_template` (comparer/finalize.rs:21909); `deep_production_path_tests::short_title_splice_crosses_only_tables_whose_every_row_is_structurally_inserted::{fn projected}` (comparer/finalize.rs:22465); `deep_production_path_tests::deletion_peel_keeps_retained_source_boundary_and_inserted_continuations::{fn paragraphs}` (comparer/finalize.rs:22639); `deep_production_path_tests::final_deletion_peel_transfers_boundary_only_with_equal_properties_and_text_payload::{fn projected}` (comparer/finalize.rs:22705); `deep_production_path_tests::deletion_peel_keeps_nontext_only_original_anchors_in_their_source_paragraph::{fn projected}` (comparer/finalize.rs:22796); `deep_production_path_tests::deletion_peel_keeps_punctuation_and_whitespace_only_original_anchors::{fn projection}` (comparer/finalize.rs:22855); `deep_production_path_tests::deletion_peel_keeps_original_closing_boundary_when_all_original_words_are_deleted::{fn projection}` (comparer/finalize.rs:22902); `deep_production_path_tests::wholesale_deleted_body_peel_transfers_its_retained_closing_boundary::{fn projection}` (comparer/finalize.rs:22947); +37 more

#### `effective_parent`  (L530-535)
`fn effective_parent(dom: &Dom, node: NodeId) -> Option<NodeId>`

- **Does:** The "skipping" parent used by `ReverseRevisionsTransform` — nearest ancestor whose name is not `w:sdtContent`/`w:sdt`/`w:smartTag`.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L532; `Dom::name` (xmllinq/mod.rs:526) L534
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.into_iter`, `.is_some_and`, `W::sdt_content`, `W::sdt`, `W::smart_tag`
- **Called by:** `reverse_revisions_transform` (revision_processor.rs:545)

#### `reverse_revisions_transform`  (L545-618)
`fn reverse_revisions_transform(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** Port of `ReverseRevisionsForPartTransform` — invert the *sense* of each revision (del↔ins, moveFrom↔moveTo, delText→t, custom-xml ranges, …), context-aware via the effective parent.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L546; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L547; `Dom::name` (xmllinq/mod.rs:526) L549,551,554; `effective_parent` (revision_processor.rs:530) L550; `Dom::parent` (xmllinq/mod.rs:597) L553; `Dom::new_element` (xmllinq/mod.rs:478) L559,563,567,570; `rebuild_named` (revision_processor.rs:621) L583,586,613,617; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L608; `XName::local_name` (xmllinq/mod.rs:208) (by name) L611
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.find`, `.is_some_and`, `.iter`, `.unwrap`, `Some`, `W::del`, `W::ins`, `W::name`, `W::p_pr`, `W::r_pr`, `W::tr_pr`
- **Called by:** `rebuild_named` (revision_processor.rs:621); `reject_content` (revision_processor.rs:755)

#### `rebuild_named`  (L621-633)
`fn rebuild_named(dom: &mut Dom, new_name: XName, src: NodeId, keep_attrs: bool) -> NodeId`

- **Does:** Build `<new_name attrs?
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L622; `Dom::attributes` (xmllinq/mod.rs:899) L624; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L625; `Dom::nodes` (xmllinq/mod.rs:604) L628; `reverse_revisions_transform` (revision_processor.rs:545) L629; `Dom::add` (xmllinq/mod.rs:1074) L630
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `reverse_revisions_transform` (revision_processor.rs:545)

#### `reject_revisions_for_part_transform`  (L641-738)
`fn reject_revisions_for_part_transform(dom: &mut Dom, node: NodeId) -> Option<NodeId>`

- **Does:** Port of `RejectRevisionsForPartTransform` — revert the *non-invertible* revisions (property changes) and drop inserted structural markers.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L642; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L643,669,676,693,706; `Dom::name` (xmllinq/mod.rs:526) L645,687; `Dom::element` (xmllinq/mod.rs:655) L648,665,668,674,702,704...; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L661; `XName::local_name` (xmllinq/mod.rs:208) (by name) L664; `Dom::new_element` (xmllinq/mod.rs:478) L670,707,728; `Dom::add` (xmllinq/mod.rs:1074) L677,734; `Dom::elements` (xmllinq/mod.rs:640) L683,718; `Dom::add_first` (xmllinq/mod.rs:1101) L694; `carry_kept_revisions` (revision_processor.rs:3597) L697,709; `reject_revisions_for_part_transform` (revision_processor.rs:641) L698,710,733; `Dom::attributes` (xmllinq/mod.rs:899) L729; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L730; `Dom::nodes` (xmllinq/mod.rs:604) L732
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.filter`, `.find`, `.into_iter`, `.is_some_and`, `.is_some`, `.iter`, `.rev`, `.unwrap`, `Some`, `W::cell_del`, `W::cell_ins`, `W::cell_merge`, `W::ins`, `W::name`, `W::num_pr`, `W::numbering_change`, `W::p_pr`, `W::r_pr_change`, `W::r_pr`, `W::sect_pr`, `W::tc_pr`, `W::tc`
- **Called by:** `reject_revisions_for_part_transform` (revision_processor.rs:641); `reject_content` (revision_processor.rs:755); `revision_boundary_coverage_tests::rejecting_structural_markers_drops_inserted_numbering_and_cell_without_losing_neighbors` (revision_processor.rs:4694)

#### `reject_revisions_document`  (L751-753, pub)
`pub fn reject_revisions_document(dom: &mut Dom, root: NodeId) -> NodeId`

- **Does:** `RejectRevisionsDocument` at element scope: revert property changes, invert the sense of every remaining revision, strip rsids, then accept.
- **Calls (crate):** `reject_content` (revision_processor.rs:755) L752
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `coverage_round_two_tests::wholesale_mesh_preserves_each_authored_terminal_whitespace_and_format::{fn source_projection}` (comparer/finalize.rs:16914); `coverage_round_two_tests::wholesale_mesh_without_terminal_whitespace_keeps_exact_supported_word_streams::{fn owned}` (comparer/finalize.rs:16965); `coverage_round_next_tests::trailing_deletion_peel_truth_table_for_shared_long_and_copula_paths::{fn projected_body}` (comparer/finalize.rs:19556); `deep_production_path_tests::title_spacers_require_unique_revised_two_blank_pretable_context` (comparer/finalize.rs:21817); `deep_production_path_tests::short_title_splice_crosses_only_tables_whose_every_row_is_structurally_inserted` (comparer/finalize.rs:22464); `deep_production_path_tests::short_title_splice_crosses_only_tables_whose_every_row_is_structurally_inserted::{fn projected}` (comparer/finalize.rs:22465); `deep_production_path_tests::deletion_peel_keeps_retained_source_boundary_and_inserted_continuations::{fn paragraphs}` (comparer/finalize.rs:22639); `deep_production_path_tests::final_deletion_peel_transfers_boundary_only_with_equal_properties_and_text_payload::{fn projected}` (comparer/finalize.rs:22705); `deep_production_path_tests::deletion_peel_keeps_nontext_only_original_anchors_in_their_source_paragraph::{fn projected}` (comparer/finalize.rs:22796); `deep_production_path_tests::deletion_peel_keeps_punctuation_and_whitespace_only_original_anchors::{fn projection}` (comparer/finalize.rs:22855); `deep_production_path_tests::deletion_peel_keeps_original_closing_boundary_when_all_original_words_are_deleted::{fn projection}` (comparer/finalize.rs:22902); `deep_production_path_tests::wholesale_deleted_body_peel_transfers_its_retained_closing_boundary::{fn projection}` (comparer/finalize.rs:22947); +38 more

#### `reject_content`  (L755-776)
`fn reject_content( dom: &mut Dom, root: NodeId, keep_comments: bool, kept: &mut Vec<String>, ) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `element_has_tracked_revisions` (revision_processor.rs:89) L761; `remove_rsid_transform` (markup_simplifier.rs:154) L762,774; `reject_revisions_for_part_transform` (revision_processor.rs:641) L765; `normalize_orphan_moves` (revision_processor.rs:1203) L772; `reverse_revisions_transform` (revision_processor.rs:545) L773; `accept_content` (revision_processor.rs:3510) L775
- **Calls (std/external/unresolved):** `.expect`
- **Called by:** `reject_revisions_document` (revision_processor.rs:751); `resolve_with_comments` (revision_processor.rs:3900)

#### `descendant_and_self_tags`  (L810-852, pub)
`pub fn descendant_and_self_tags(dom: &Dom, element: NodeId) -> Vec<Tag>`

- **Does:** A.0 — `DescendantAndSelfTags` (RevisionProcessor.cs :2397): stream `element` and its descendants as open / empty / close tags in document order.
- **Calls (crate):** `element_children_vec` (revision_processor.rs:855) L818,834; `Dom::child_count` (xmllinq/mod.rs:622) L823
- **Calls (std/external/unresolved):** `.last_mut`, `.last`, `.len`, `.pop`, `.push`
- **Macros:** `vec!`
- **Called by:** `wholly_deleted_endpoints` (revision_processor/bookmarks.rs:47); `normalize_orphan_moves` (revision_processor.rs:1203); `move_from_range_deleted_elements` (revision_processor.rs:1370); `accept_deleted_and_moved_from_content_controls` (revision_processor.rs:1676)

#### `element_children_vec`  (L855-865)
`fn element_children_vec(dom: &Dom, id: NodeId) -> Vec<NodeId>`

- **Does:** Direct element children (document order) without the `elements()` filter path.
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L857; `Dom::child_at` (xmllinq/mod.rs:632) L859; `Dom::is_element` (xmllinq/mod.rs:502) L860
- **Calls (std/external/unresolved):** `.push`, `Vec::new`
- **Called by:** `descendant_and_self_tags` (revision_processor.rs:810); `iterate_block_content_elements` (revision_processor.rs:930)

#### `first_block_content`  (L885-915)
`fn first_block_content(dom: &Dom, roots: &[NodeId]) -> Option<NodeId>`

- **Does:** First paragraph, table, or opaque equation block among `roots`' descendants-and-self, document order.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L888; `Dom::child_count` (xmllinq/mod.rs:622) L898; `Dom::child_at` (xmllinq/mod.rs:632) L903; `Dom::is_element` (xmllinq/mod.rs:502) L905
- **Calls (std/external/unresolved):** `.is_some_and`, `.last_mut`, `.pop`, `.push`, `M::name`, `Some`, `W::p`, `W::tbl`, `is_block`
- **Macros:** `vec!`
- **Called by:** `iterate_block_content_elements` (revision_processor.rs:930)

#### `elements_after_self`  (L918-923)
`fn elements_after_self(dom: &Dom, id: NodeId) -> Vec<NodeId>`

- **Does:** Element siblings after `id`, document order.
- **Calls (crate):** `Dom::nodes_after_self` (xmllinq/mod.rs:838) L919; `Dom::is_element` (xmllinq/mod.rs:502) L921
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`
- **Called by:** `iterate_block_content_elements` (revision_processor.rs:930)

#### `iterate_block_content_elements`  (L930-966, pub)
`pub fn iterate_block_content_elements(dom: &Dom, element: NodeId) -> Vec<BlockContentInfo>`

- **Does:** A.0 — `IterateBlockContentElements` (:1909) + `AnnotateBlockContentElements` (:1855): the doc-order chain of paragraphs, tables, and equation blocks under `element`, linked prev/this/next.
- **Calls (crate):** `element_children_vec` (revision_processor.rs:855) L932; `first_block_content` (revision_processor.rs:885) L936,944; `elements_after_self` (revision_processor.rs:918) L944; `Dom::parent` (xmllinq/mod.rs:597) L948
- **Calls (std/external/unresolved):** `.collect`, `.copied`, `.get`, `.is_empty`, `.last`, `.len`, `.map`, `.push`, `.then`, `.unwrap`, `Some`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `accept_paragraph_marks_with_owners` (revision_processor.rs:2035); `body_equation_paragraph_mark_ownership_tests::equation_is_an_opaque_block_barrier_for_deleted_paragraph_marks` (revision_processor.rs:5111)

#### `block_level_content_containers`  (L973-984)
`fn block_level_content_containers() -> [XName; 8]`

- **Does:** `W.BlockLevelContentContainers` (PtOpenXmlUtil.cs :5569), plus `w:comment`: the C# pipeline never visits `comments.xml`, but Word's Accept/Reject All does, and a deleted or inserted paragraph mark inside a comment merges the comment's own paragraphs — never across the `w:comment` boundary.
- **Calls (std/external/unresolved):** `W::body`, `W::endnote`, `W::footnote`, `W::ftr`, `W::hdr`, `W::name`, `W::tc`, `W::txbx_content`
- **Called by:** `get_paragraph_info` (revision_processor.rs:995); `accept_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1579); `accept_paragraph_marks_with_owners` (revision_processor.rs:2035)

#### `get_paragraph_info`  (L995-1027, pub)
`pub fn get_paragraph_info(dom: &Dom, content_element: NodeId) -> BlockContentInfo`

- **Does:** A.0 — `GetParagraphInfo` (:2917) + `InitializeParagraphInfo` (:2890): for a child of a block-level content container, `this` = the first descendant-or-self in {`w:p`, `w:tc`, `w:txbxContent`} — nulled when that first hit is a `tc`/`txbxContent` (means "no own paragraph") — and `previous` = the previous element sibling...
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L996; `Dom::name` (xmllinq/mod.rs:526) L999,1009,1011; `block_level_content_containers` (revision_processor.rs:973) L1001; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L1006; `Dom::nodes_before_self` (xmllinq/mod.rs:846) L1016; `Dom::is_element` (xmllinq/mod.rs:502) L1020
- **Calls (std/external/unresolved):** `.contains`, `.expect`, `.find`, `.into_iter`, `.is_some_and`, `.rev`, `W::p`, `W::tc`, `W::txbx_content`
- **Macros:** `assert!`
- **Called by:** `accept_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1579)

#### `content_elements_before_self`  (L1031-1037, pub)
`pub fn content_elements_before_self(dom: &Dom, element: NodeId) -> Vec<NodeId>`

- **Does:** A.0 — `ContentElementsBeforeSelf` (:2926): previous element siblings, nearest first.
- **Calls (crate):** `Dom::nodes_before_self` (xmllinq/mod.rs:846) L1032; `Dom::is_element` (xmllinq/mod.rs:502) L1035
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.rev`
- **Called by:** `accept_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1579); `accept_deleted_cells_transform` (revision_processor.rs:2867)

#### `transform_instr_text_to_del_instr_text`  (L1043-1069)
`fn transform_instr_text_to_del_instr_text(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** A.1 — `TransformInstrTextToDelInstrText` (:1433): rename `w:instrText` to `w:delInstrText` (attrs + nodes carried as-is), rebuilding everything else.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L1044; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1045,1055; `Dom::name` (xmllinq/mod.rs:526) L1047; `Dom::new_element` (xmllinq/mod.rs:478) L1049,1060; `Dom::attributes` (xmllinq/mod.rs:899) L1050,1061; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1051,1062; `Dom::nodes` (xmllinq/mod.rs:604) L1054,1064; `Dom::add` (xmllinq/mod.rs:1074) L1056,1066; `transform_instr_text_to_del_instr_text` (revision_processor.rs:1043) L1065
- **Calls (std/external/unresolved):** `.unwrap`, `Some`, `W::del_instr_text`, `W::instr_text`
- **Called by:** `transform_instr_text_to_del_instr_text` (revision_processor.rs:1043); `fix_up_deleted_or_inserted_field_codes_transform` (revision_processor.rs:1080)

#### `fix_up_deleted_or_inserted_field_codes_transform`  (L1080-1164, pub)
`pub fn fix_up_deleted_or_inserted_field_codes_transform(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** A.1 — `FixUpDeletedOrInsertedFieldCodesTransform` (:1354): inside each `w:p`, group-adjacent the child elements by kind — 2 = `w:del` holding `w:r/w:fldChar`, 3 = `w:ins` holding `w:r/w:fldChar`, 4 = `w:r` with `w:instrText`, 1 = other.
- **Calls (crate):** `element_or_desc_has_name` (revision_processor.rs:183) L1084; `Dom::is_element` (xmllinq/mod.rs:502) L1087; `Dom::name` (xmllinq/mod.rs:526) L1090,1093; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L1095; `Dom::element` (xmllinq/mod.rs:655) (by name) L1097; `Dom::element` (xmllinq/mod.rs:655) L1103; `Dom::elements` (xmllinq/mod.rs:640) L1109; `Dom::new_element` (xmllinq/mod.rs:478) L1113,1130,1137,1155; `Dom::attributes` (xmllinq/mod.rs:899) L1114,1156; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1115,1157; `fix_up_deleted_or_inserted_field_codes_transform` (revision_processor.rs:1080) L1121,1139,1145,1160; `Dom::add` (xmllinq/mod.rs:1074) L1122,1133,1135,1140,1142,1146...; `transform_instr_text_to_del_instr_text` (revision_processor.rs:1043) L1132; `Dom::nodes` (xmllinq/mod.rs:604) L1159
- **Calls (std/external/unresolved):** `.any`, `.enumerate`, `.into_iter`, `.is_some`, `.iter`, `.len`, `.unwrap`, `Some`, `W::del`, `W::fld_char`, `W::ins`, `W::instr_text`, `W::p`, `W::r`, `crate::util::group_adjacent`, `flanked_by`, `holds_fld_char`, `key_of`
- **Macros:** `unreachable!`
- **Called by:** `fix_up_deleted_or_inserted_field_codes_transform` (revision_processor.rs:1080); `accept_content` (revision_processor.rs:3510); `complete_field_instruction_metadata_boundary_tests::repaired_complete_field_codes_preserve_source_comments_processing_instructions_and_formats` (revision_processor.rs:5413)

#### `normalize_orphan_moves`  (L1203-1310)
`fn normalize_orphan_moves(dom: &mut Dom, root: NodeId, pass: OrphanMovePass) -> NodeId`

- **Does:** #1235, #1243: a `w:moveTo` / `w:moveFrom` is an ORPHAN when no range of the other kind shares its `w:name` in the same part — a bare `moveTo` has no `w:name` at all, so it is an orphan unless it sits inside a paired `moveToRange`.
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L1207,1234; `Dom::name` (xmllinq/mod.rs:526) L1208,1235,1253; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1209,1254; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1212,1257; `Dom::attribute` (xmllinq/mod.rs:874) L1214,1219,1259,1260,1264,1268...; `descendant_and_self_tags` (revision_processor.rs:810) L1251; `normalize_orphan_moves_transform` (revision_processor.rs:1315) L1308
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.expect`, `.insert`, `.into_iter`, `.is_empty`, `.is_some_and`, `.remove`, `.to_string`, `.unwrap_or`, `.values`, `HashMap::new`, `HashSet::new`, `W::id`, `W::move_from`, `W::move_to`, `W::name`
- **Called by:** `reject_content` (revision_processor.rs:755); `accept_content` (revision_processor.rs:3510)

#### `normalize_orphan_moves_transform`  (L1315-1345)
`fn normalize_orphan_moves_transform( dom: &mut Dom, node: NodeId, to_ins: &HashSet<NodeId>, to_del: &HashSet<NodeId>, drop_markers: &HashSet<NodeId>, ) -> Option<NodeId>`

- **Does:** Rebuild for [`normalize_orphan_moves`]: orphan `moveTo`/`moveFrom` re-emerge as `w:ins`/`w:del` (attributes preserved), orphan range markers vanish, everything else rebuilds recursively.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L1322; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1323; `Dom::name` (xmllinq/mod.rs:526) L1333; `Dom::new_element` (xmllinq/mod.rs:478) L1335; `Dom::attributes` (xmllinq/mod.rs:899) L1336; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1337; `Dom::nodes` (xmllinq/mod.rs:604) L1339; `normalize_orphan_moves_transform` (revision_processor.rs:1315) L1340; `Dom::add` (xmllinq/mod.rs:1074) L1341
- **Calls (std/external/unresolved):** `.contains`, `.unwrap`, `Some`, `W::del`, `W::ins`
- **Called by:** `normalize_orphan_moves` (revision_processor.rs:1203); `normalize_orphan_moves_transform` (revision_processor.rs:1315)

#### `accept_move_from_ranges`  (L1358-1365, pub)
`pub fn accept_move_from_ranges(dom: &mut Dom, document: NodeId) -> NodeId`

- **Does:** A.2 — `AcceptMoveFromRanges` (:1530): walk the tag stream; while one or more `moveFromRangeStart` ids are open, record every other element's open tag (start-list) and close tag (end-list; empty elements land in both).
- **Calls (crate):** `move_from_range_deleted_elements` (revision_processor.rs:1370) L1359; `accept_move_from_ranges_transform` (revision_processor.rs:1462) L1363
- **Calls (std/external/unresolved):** `.expect`, `.is_empty`
- **Called by:** `accept_content` (revision_processor.rs:3510); `revision_boundary_coverage_tests::move_range_that_enters_table_keeps_container_properties_and_surviving_content` (revision_processor.rs:4430); `revision_boundary_coverage_tests::unmatched_or_contentless_move_ranges_preserve_identity_and_properties` (revision_processor.rs:4775)

#### `move_from_range_deleted_elements`  (L1370-1445)
`fn move_from_range_deleted_elements(dom: &Dom, document: NodeId) -> HashSet<NodeId>`

- **Does:** Elements wholly removed by completed move-from ranges.
- **Calls (crate):** `descendant_and_self_tags` (revision_processor.rs:810) L1381; `Dom::name` (xmllinq/mod.rs:526) L1382,1437; `Dom::attribute` (xmllinq/mod.rs:874) L1384,1392; `is_inline_range_marker` (revision_processor.rs:2288) L1437; `is_container_property` (revision_processor.rs:1450) L1438,1442; `Dom::ancestors` (xmllinq/mod.rs:770) L1439
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.contains`, `.extend`, `.filter`, `.insert`, `.into_iter`, `.is_empty`, `.is_some_and`, `.push`, `.remove`, `.to_string`, `.unwrap_or`, `.unwrap`, `.values_mut`, `HashMap::new`, `Vec::new`, `W::id`, `W::move_from_range_end`, `W::move_from_range_start`
- **Called by:** `wholly_deleted_endpoints` (revision_processor/bookmarks.rs:47); `accept_move_from_ranges` (revision_processor.rs:1358)

#### `is_container_property`  (L1450-1458)
`fn is_container_property(dom: &Dom, e: NodeId) -> bool`

- **Does:** Is `e` the property element of its container (a paragraph's pPr, a table's tblPr or tblGrid, a row's trPr or tblPrEx, a cell's tcPr, a content control's sdtPr or sdtEndPr)?
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L1451; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1452; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1454
- **Calls (std/external/unresolved):** `.is_some_and`
- **Macros:** `matches!`
- **Called by:** `move_from_range_deleted_elements` (revision_processor.rs:1370)

#### `accept_move_from_ranges_transform`  (L1462-1489)
`fn accept_move_from_ranges_transform( dom: &mut Dom, node: NodeId, to_delete: &std::collections::HashSet<NodeId>, ) -> Option<NodeId>`

- **Does:** A.2 — `AcceptMoveFromRangesTransform` (:2629): rebuild, dropping the elements marked for deletion (and thereby their subtrees).
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L1467; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1468; `Dom::new_element` (xmllinq/mod.rs:478) L1473; `Dom::name` (xmllinq/mod.rs:526) L1473; `Dom::attributes` (xmllinq/mod.rs:899) L1474; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1475; `Dom::nodes` (xmllinq/mod.rs:604) L1477; `accept_move_from_ranges_transform` (revision_processor.rs:1462) L1478; `Dom::add` (xmllinq/mod.rs:1074) L1479,1484; `hoist_range_markers_from` (revision_processor.rs:2314) L1483
- **Calls (std/external/unresolved):** `.contains`, `.unwrap`, `Some`
- **Called by:** `accept_move_from_ranges` (revision_processor.rs:1358); `accept_move_from_ranges_transform` (revision_processor.rs:1462)

#### `collapse_paragraph_transform`  (L1496-1519, pub)
`pub fn collapse_paragraph_transform(dom: &mut Dom, node: NodeId) -> Vec<NodeId>`

- **Does:** A.3 — `CollapseParagraphTransform` (:1821): a `w:p` collapses to its element children minus `w:pPr` (clones — C# XLinq re-parents by cloning); other elements rebuild recursively; non-elements pass through.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L1497; `Dom::clone_subtree` (xmllinq/mod.rs:1231) (by name) L1498; `Dom::name` (xmllinq/mod.rs:526) L1500,1505; `Dom::elements` (xmllinq/mod.rs:640) L1502; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1507; `Dom::new_element` (xmllinq/mod.rs:478) L1509; `Dom::attributes` (xmllinq/mod.rs:899) L1510; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1511; `Dom::nodes` (xmllinq/mod.rs:604) L1513; `collapse_paragraph_transform` (revision_processor.rs:1496) L1514; `Dom::add` (xmllinq/mod.rs:1074) L1515
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.map`, `.unwrap`, `Some`, `W::p_pr`, `W::p`
- **Macros:** `vec!`
- **Called by:** `collapse_paragraph_transform` (revision_processor.rs:1496); `coalesque_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1527); `accept_paragraph_marks_with_owners` (revision_processor.rs:2035)

#### `coalesque_paragraph_end_tags_in_move_from_transform`  (L1527-1561, pub)
`pub fn coalesque_paragraph_end_tags_in_move_from_transform( dom: &mut Dom, node: NodeId, group: &[NodeId], ) -> NodeId`

- **Does:** A.3 — `CoalesqueParagraphEndTagsInMoveFromTransform` (:2645): produce the first group member, with its (first descendant) paragraph replaced by one carrying the paragraph's own children plus the COLLAPSED content of the group's subsequent members.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L1532; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1533,1542; `Dom::name` (xmllinq/mod.rs:526) L1535; `Dom::new_element` (xmllinq/mod.rs:478) L1537,1552; `Dom::attributes` (xmllinq/mod.rs:899) L1538,1553; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1539,1554; `Dom::elements` (xmllinq/mod.rs:640) L1541; `Dom::add` (xmllinq/mod.rs:1074) L1543,1547,1558; `collapse_paragraph_transform` (revision_processor.rs:1496) L1546; `Dom::nodes` (xmllinq/mod.rs:604) L1556; `coalesque_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1527) L1557
- **Calls (std/external/unresolved):** `.iter`, `.skip`, `.unwrap`, `Some`, `W::p`
- **Called by:** `coalesque_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1527); `accept_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1579)

#### `accept_paragraph_end_tags_in_move_from_transform`  (L1579-1665, pub)
`pub fn accept_paragraph_end_tags_in_move_from_transform(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** A.3 — `AcceptParagraphEndTagsInMoveFromTransform` (:1610): group a content container's children by whether their paragraph mark sits in an OPEN moveFrom range (`moveFromRangeStart` child without `moveFromRangeEnd`), or they are a `w:p` directly following such a paragraph.
- **Calls (crate):** `element_or_desc_has_name` (revision_processor.rs:183) L1581; `Dom::is_element` (xmllinq/mod.rs:502) L1584; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1585,1631; `Dom::name` (xmllinq/mod.rs:526) L1587,1609; `block_level_content_containers` (revision_processor.rs:973) L1588; `Dom::elements` (xmllinq/mod.rs:640) L1592,1617; `get_paragraph_info` (revision_processor.rs:995) L1596,1603,1608; `content_elements_before_self` (revision_processor.rs:1031) L1602; `Dom::new_element` (xmllinq/mod.rs:478) L1624,1646,1656; `Dom::attributes` (xmllinq/mod.rs:899) L1625,1647,1657; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1626,1648,1658; `Dom::add` (xmllinq/mod.rs:1074) L1632,1639,1652,1662; `coalesque_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1527) L1638; `Dom::nodes` (xmllinq/mod.rs:604) L1650,1660; `accept_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1579) L1651,1661
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.into_iter`, `.is_empty`, `.is_some`, `.len`, `.unwrap`, `Some`, `W::move_from_range_end`, `W::move_from_range_start`, `W::p`, `crate::util::group_adjacent`, `key_of`, `mark_in_open_range`
- **Called by:** `accept_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1579); `accept_content` (revision_processor.rs:3510)

#### `accept_deleted_and_moved_from_content_controls`  (L1676-1795, pub)
`pub fn accept_deleted_and_moved_from_content_controls(dom: &mut Dom, root: NodeId) -> NodeId`

- **Does:** A.4 — `AcceptDeletedAndMovedFromContentControls` (:2491): walk the tag stream tracking two range kinds.
- **Calls (crate):** `descendant_and_self_tags` (revision_processor.rs:810) L1694; `Dom::name` (xmllinq/mod.rs:526) L1695; `Dom::attribute` (xmllinq/mod.rs:874) L1697,1705,1713,1721; `accept_deleted_and_moved_from_content_controls_transform` (revision_processor.rs:1800) L1787
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.contains`, `.extend`, `.filter`, `.insert`, `.into_iter`, `.is_empty`, `.len`, `.push`, `.remove`, `.to_string`, `.unwrap_or`, `.unwrap`, `.values_mut`, `HashMap::new`, `Vec::new`, `W::id`, `W::move_from_range_end`, `W::move_from_range_start`, `W::name`, `W::sdt`
- **Macros:** `debug_assert_eq!`
- **Called by:** `accept_content` (revision_processor.rs:3510); `revision_boundary_coverage_tests::custom_xml_deleted_control_unwraps_nested_controls_but_retains_order_and_anchors` (revision_processor.rs:4358); `revision_boundary_coverage_tests::custom_xml_move_deletes_whole_control_and_plain_content_but_keeps_outside_control` (revision_processor.rs:4389); `revision_boundary_coverage_tests::unmatched_custom_ranges_leave_control_payload_and_parent_links_untouched` (revision_processor.rs:4410)

#### `accept_deleted_and_moved_from_content_controls_transform`  (L1800-1840)
`fn accept_deleted_and_moved_from_content_controls_transform( dom: &mut Dom, node: NodeId, to_collapse: &std::collections::HashSet<NodeId>, to_delete: &std::collections::HashSet<NodeId>, ) -> Vec<NodeId>`

- **Does:** A.4 — `AcceptDeletedAndMovedFromContentControlsTransform` (:2468): splice a collapsed sdt's `sdtContent` child nodes (transformed) in its place, drop deleted elements, rebuild the rest.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L1806; `Dom::clone_subtree` (xmllinq/mod.rs:1231) (by name) L1807; `Dom::name` (xmllinq/mod.rs:526) L1809; `Dom::element` (xmllinq/mod.rs:655) L1811; `Dom::nodes` (xmllinq/mod.rs:604) L1815,1832; `accept_deleted_and_moved_from_content_controls_transform` (revision_processor.rs:1800) L1816,1834; `Dom::new_element` (xmllinq/mod.rs:478) L1828; `Dom::attributes` (xmllinq/mod.rs:899) L1829; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1830; `Dom::add` (xmllinq/mod.rs:1074) L1836
- **Calls (std/external/unresolved):** `.contains`, `.expect`, `.extend`, `.unwrap`, `Some`, `Vec::new`, `W::sdt_content`, `W::sdt`
- **Macros:** `vec!`
- **Called by:** `accept_deleted_and_moved_from_content_controls` (revision_processor.rs:1676); `accept_deleted_and_moved_from_content_controls_transform` (revision_processor.rs:1800)

#### `is_run_content`  (L1846-1889)
`fn is_run_content(name: &XName) -> Option<bool>`

- **Does:** A.5a — `IsRunContent` (:2356): `Some(true)` = run-level content, `Some(false)` = marker/non-content, `None` = unknown (C# throws).
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L1853,1858,1861; `XName::local_name` (xmllinq/mod.rs:208) L1859,1864
- **Calls (std/external/unresolved):** `.contains`, `Some`, `crate::namespaces::MC::name`
- **Called by:** `all_para_content_is_deleted` (revision_processor.rs:1941)

#### `collapse_transform`  (L1897-1933)
`fn collapse_transform(dom: &mut Dom, node: NodeId) -> Vec<NodeId>`

- **Does:** A.5a — `CollapseTransform` (:2331): splice `w:dir`/`w:bdr`/`w:ins`/ `w:moveTo`/`w:smartTag` to their element children and `w:sdt` to its `sdtContent` element children — ONE level, un-recursed, exactly like the C# (`return element.Elements()`); drop `w:pPr`; rebuild everything else recursively.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L1898; `Dom::clone_subtree` (xmllinq/mod.rs:1231) (by name) L1899; `Dom::name` (xmllinq/mod.rs:526) L1901; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1902; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1904; `Dom::elements` (xmllinq/mod.rs:640) L1908,1913,1914; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1909,1915; `Dom::new_element` (xmllinq/mod.rs:478) L1923; `Dom::attributes` (xmllinq/mod.rs:899) L1924; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1925; `Dom::nodes` (xmllinq/mod.rs:604) L1927; `collapse_transform` (revision_processor.rs:1897) L1928; `Dom::add` (xmllinq/mod.rs:1074) L1929
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`, `.push`, `.unwrap`, `Some`, `Vec::new`, `W::p_pr`, `W::sdt_content`, `W::sdt`
- **Macros:** `matches!`, `vec!`
- **Called by:** `collapse_transform` (revision_processor.rs:1897); `all_para_content_is_deleted` (revision_processor.rs:1941)

#### `all_para_content_is_deleted`  (L1941-1967)
`fn all_para_content_is_deleted(dom: &mut Dom, p: NodeId) -> bool`

- **Does:** A.5a — `AllParaContentIsDeleted` (:2310): after collapsing wrappers, does the paragraph hold NO run-level content?
- **Calls (crate):** `collapse_transform` (revision_processor.rs:1897) L1942; `Dom::elements` (xmllinq/mod.rs:640) L1945; `Dom::name` (xmllinq/mod.rs:526) L1946,1957; `accept_all_other_revisions_transform` (revision_processor.rs:377) L1954; `is_run_content` (revision_processor.rs:1846) L1958,1964; `XName::clark` (xmllinq/mod.rs:226) (by name) L1959,1965
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_some_and`, `.len`, `.unwrap_or_else`, `.unwrap`, `M::name`
- **Macros:** `debug_assert_eq!`, `panic!`
- **Called by:** `accept_paragraph_marks_with_owners` (revision_processor.rs:2035)

#### `table_rows_all_deleted`  (L1970-1977)
`fn table_rows_all_deleted(dom: &Dom, tbl: NodeId) -> bool`

- **Does:** Does every row of `tbl` carry `trPr/w:del` (and is there a row)?
- **Calls (crate):** `wrapped_children` (revision_processor.rs:3021) L1971; `Dom::element` (xmllinq/mod.rs:655) L1974,1975
- **Calls (std/external/unresolved):** `.all`, `.into_iter`, `.is_empty`, `.is_some_and`, `.is_some`, `W::del`, `W::tr_pr`, `W::tr`
- **Called by:** `removed_container` (revision_processor/comments.rs:240); `try_accept_deleted_table` (revision_processor.rs:308); `accept_paragraph_marks_with_owners` (revision_processor.rs:2035)

#### `paragraph_mark_is_deleted_or_moved_from`  (L1981-1991)
`fn paragraph_mark_is_deleted_or_moved_from(dom: &Dom, p: NodeId) -> bool`

- **Does:** `p / pPr / rPr / (del | moveFrom)` — is the paragraph mark deleted or moved from?
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1983,1984; `Dom::elements` (xmllinq/mod.rs:640) L1986; `Dom::name` (xmllinq/mod.rs:526) L1987
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.into_iter`, `.is_some_and`, `W::del`, `W::move_from`, `W::p_pr`, `W::r_pr`
- **Called by:** `wholly_deleted_endpoints` (revision_processor/bookmarks.rs:47); `carry_vanishing_section_references` (revision_processor/sections.rs:31); `has_deleted_or_moved_from_paragraph_mark::{fn walk}` (revision_processor.rs:1996); `accept_paragraph_marks_with_owners` (revision_processor.rs:2035)

#### `has_deleted_or_moved_from_paragraph_mark`  (L1995-2011)
`fn has_deleted_or_moved_from_paragraph_mark(dom: &Dom, root: NodeId) -> bool`

- **Does:** ACCEPT-SKIP-A5: non-allocating presence of any paragraph whose mark is deleted or moved-from under `root`.
- **Calls (crate):** `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90) L2010
- **Called by:** `accept_deleted_and_move_from_paragraph_marks` (revision_processor.rs:2755); `body_equation_paragraph_mark_ownership_tests::public_accept_and_reject_preserve_unchanged_body_math_among_revised_marks` (revision_processor.rs:5128)

#### `has_deleted_or_moved_from_paragraph_mark::{fn walk}`  (L1996-2009)
`fn walk(dom: &Dom, id: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L1997; `paragraph_mark_is_deleted_or_moved_from` (revision_processor.rs:1981) L1998; `Dom::child_count` (xmllinq/mod.rs:622) L2002; `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90) L2004; `Dom::child_at` (xmllinq/mod.rs:632) L2004
- **Calls (std/external/unresolved):** `.as_ref`, `Some`, `W::p`
- **Called by:** `element_has_tracked_revisions` (revision_processor.rs:89); `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90); `element_or_desc_has_name` (revision_processor.rs:183); `element_or_desc_has_name::{fn walk}` (revision_processor.rs:184); `has_deleted_or_moved_from_paragraph_mark` (revision_processor.rs:1995); `has_deleted_or_moved_from_paragraph_mark::{fn walk}` (revision_processor.rs:1996); `subtree_needs_adjacent_table_merge` (revision_processor.rs:3165); `subtree_needs_adjacent_table_merge::{fn walk}` (revision_processor.rs:3166); `has_empty_table_cell` (revision_processor.rs:3413); `has_empty_table_cell::{fn walk}` (revision_processor.rs:3416)

#### `accept_deleted_and_move_from_paragraph_marks_transform`  (L2026-2031, pub)
`pub fn accept_deleted_and_move_from_paragraph_marks_transform( dom: &mut Dom, node: NodeId, ) -> NodeId`

- **Does:** A.5a — `AcceptDeletedAndMoveFromParagraphMarksTransform` (:2119): the 3-state grouping machine over a container's block-content chain.
- **Calls (crate):** `accept_paragraph_marks_with_owners` (revision_processor.rs:2035) L2030
- **Calls (std/external/unresolved):** `HashMap::new`
- **Called by:** `body_equation_paragraph_mark_ownership_tests::equation_is_an_opaque_block_barrier_for_deleted_paragraph_marks` (revision_processor.rs:5111); `deleted_title_equation_barrier_tests::deleted_title_before_equation_is_removed_without_consuming_the_equation` (revision_processor.rs:5335)

#### `accept_paragraph_marks_with_owners`  (L2035-2272)
`fn accept_paragraph_marks_with_owners( dom: &mut Dom, node: NodeId, paragraph_owners: &mut HashMap<NodeId, Vec<NodeId>>, ) -> NodeId`

- **Does:** Original paragraph owners are private reconstruction evidence. _(from a code comment)_
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L2040; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L2041,2136,2209,2225,2268; `Dom::name` (xmllinq/mod.rs:526) L2043,2072,2104,2143,2168,2187...; `block_level_content_containers` (revision_processor.rs:973) L2044; `Dom::new_element` (xmllinq/mod.rs:478) L2045,2134,2188,2204; `Dom::attributes` (xmllinq/mod.rs:899) L2046,2189,2205; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L2047,2190,2206; `Dom::nodes` (xmllinq/mod.rs:604) L2049,2192; `accept_paragraph_marks_with_owners` (revision_processor.rs:2035) L2050,2193; `Dom::add` (xmllinq/mod.rs:1074) L2051,2137,2148,2194,2210,2226...; `Dom::name_is` (xmllinq/mod.rs:535) L2053,2156,2196; `Dom::element` (xmllinq/mod.rs:655) L2060; `iterate_block_content_elements` (revision_processor.rs:930) L2065; `paragraph_mark_is_deleted_or_moved_from` (revision_processor.rs:1981) L2074,2161; `table_rows_all_deleted` (revision_processor.rs:1970) L2105; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L2111; `Dom::elements` (xmllinq/mod.rs:640) L2135,2208,2217; `collapse_paragraph_transform` (revision_processor.rs:1496) L2147; `all_para_content_is_deleted` (revision_processor.rs:1941) L2172; `hoist_range_markers_from` (revision_processor.rs:2314) L2179; `is_body_level_range_marker` (revision_processor.rs:2275) L2224; `Dom::ancestors` (xmllinq/mod.rs:770) L2254
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.collect`, `.contains`, `.copied`, `.enumerate`, `.filter`, `.insert`, `.into_iter`, `.is_none`, `.is_some_and`, `.iter`, `.last`, `.len`, `.position`, `.push`, `.unwrap`, `.zip`, `HashSet::new`, `M::name`, `Some`, `Vec::new`, `Vec::with_capacity`, `W::body`, `W::p_pr`, `W::p`, `W::sect_pr`, `W::tbl`, `W::tc_pr`, `crate::util::group_adjacent`
- **Macros:** `vec!`
- **Called by:** `accept_deleted_and_move_from_paragraph_marks_transform` (revision_processor.rs:2026); `accept_paragraph_marks_with_owners` (revision_processor.rs:2035); `accept_deleted_and_move_from_paragraph_marks` (revision_processor.rs:2755)

#### `is_body_level_range_marker`  (L2275-2286)
`fn is_body_level_range_marker(name: &XName) -> bool`

- **Does:** Body/cell-level markers that must survive block-content rebuild (not p/tbl).
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L2276; `XName::local_name` (xmllinq/mod.rs:208) (by name) L2278
- **Macros:** `matches!`
- **Called by:** `accept_paragraph_marks_with_owners` (revision_processor.rs:2035)

#### `is_inline_range_marker`  (L2288-2299)
`fn is_inline_range_marker(name: &XName) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L2289; `XName::local_name` (xmllinq/mod.rs:208) (by name) L2291
- **Macros:** `matches!`
- **Called by:** `move_from_range_deleted_elements` (revision_processor.rs:1370); `hoist_range_markers_from` (revision_processor.rs:2314)

#### `drop_wholly_deleted_permissions`  (L2304-2310)
`fn drop_wholly_deleted_permissions(dom: &mut Dom, root: NodeId)`

- **Does:** Permission ranges follow Word's character-range rule, including paragraph marks: discard both endpoints only when their nonempty span disappears.
- **Calls (crate):** `wholly_deleted_endpoints` (revision_processor/bookmarks.rs:47) L2306; `Dom::remove` (xmllinq/mod.rs:1118) L2308
- **Calls (std/external/unresolved):** `W::name`
- **Called by:** `accept_content` (revision_processor.rs:3510)

#### `hoist_range_markers_from`  (L2314-2325)
`fn hoist_range_markers_from(dom: &mut Dom, node: NodeId) -> Vec<NodeId>`

- **Does:** Pull comment, bookmark and permission markers out of a subtree being discarded (e.g.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2316; `Dom::name` (xmllinq/mod.rs:526) L2317; `is_inline_range_marker` (revision_processor.rs:2288) L2320; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L2321
- **Calls (std/external/unresolved):** `.push`, `Vec::new`
- **Called by:** `accept_move_from_move_to_transform` (revision_processor.rs:128); `try_accept_wholly_revised_equation` (revision_processor.rs:216); `try_accept_deleted_text` (revision_processor.rs:315); `try_accept_hyperlink` (revision_processor.rs:345); `accept_move_from_ranges_transform` (revision_processor.rs:1462); `accept_paragraph_marks_with_owners` (revision_processor.rs:2035)

#### `annotate_run_elements_with_id`  (L2331-2336, pub)
`pub fn annotate_run_elements_with_id(dom: &mut Dom, element: NodeId)`

- **Does:** A.5b — `AnnotateRunElementsWithId` (:1935): number every descendant `w:r` with `pt:UniqueId` 0..
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2333; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L2334
- **Calls (std/external/unresolved):** `.to_string`, `.zip`, `PT::unique_id`, `Some`, `W::r`
- **Called by:** `accept_deleted_and_move_from_paragraph_marks` (revision_processor.rs:2755)

#### `annotate_paragraph_elements_with_id`  (L2343-2348)
`fn annotate_paragraph_elements_with_id(dom: &mut Dom, element: NodeId)`

- **Does:** Number every descendant `w:p` (`pt:UniqueId` `p0`, `p1`, …) so A.5b can find a control's paragraphs when it has no run to anchor on (an empty control keeps its paragraph mark, and so its place).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2345; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L2346
- **Calls (std/external/unresolved):** `.zip`, `PT::unique_id`, `Some`, `W::p`
- **Macros:** `format!`
- **Called by:** `accept_deleted_and_move_from_paragraph_marks` (revision_processor.rs:2755)

#### `descendants_trimmed`  (L2352-2364)
`fn descendants_trimmed(dom: &Dom, element: NodeId, trim: &XName) -> Vec<NodeId>`

- **Does:** Descendants of `element` in document order, not descending INTO elements named `trim` (the `DescendantsTrimmed` helper the annotators use).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L2354,2358; `Dom::name` (xmllinq/mod.rs:526) L2357
- **Calls (std/external/unresolved):** `.as_ref`, `.collect`, `.into_iter`, `.pop`, `.push`, `.rev`, `Some`, `Vec::new`
- **Called by:** `annotate_content_controls_with_run_ids` (revision_processor.rs:2369)

#### `annotate_content_controls_with_run_ids`  (L2369-2382, pub)
`pub fn annotate_content_controls_with_run_ids(dom: &mut Dom, element: NodeId)`

- **Does:** A.5b — `AnnotateContentControlsWithRunIds` (:1945): give every descendant `w:sdt` a `pt:RunIds` (comma-joined `pt:UniqueId`s of its runs, trimmed at `w:txbxContent`) and its own `pt:UniqueId` (in place).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2373; `descendants_trimmed` (revision_processor.rs:2352) L2374; `Dom::name` (xmllinq/mod.rs:526) L2376; `Dom::attribute` (xmllinq/mod.rs:874) L2377; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L2379,2380
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.filter`, `.into_iter`, `.join`, `.map`, `.to_string`, `.zip`, `PT::run_ids`, `PT::unique_id`, `Some`, `W::r`, `W::sdt`, `W::txbx_content`
- **Called by:** `accept_deleted_and_move_from_paragraph_marks` (revision_processor.rs:2755)

#### `order_sdt`  (L2385-2397)
`fn order_sdt(name: &XName) -> i32`

- **Does:** `Order_sdt` (:2090): schema order for rebuilt `w:sdt` children.
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L2386; `XName::local_name` (xmllinq/mod.rs:208) L2389
- **Called by:** `add_content_controls_with_owners` (revision_processor.rs:2422)

#### `add_block_level_content_controls`  (L2414-2420, pub)
`pub fn add_block_level_content_controls( dom: &mut Dom, new_document: NodeId, original: NodeId, ) -> NodeId`

- **Does:** A.5b — `AddBlockLevelContentControls` (:1964): re-create the `w:sdt` wrappers the paragraph-mark transform stripped.
- **Calls (crate):** `add_content_controls_with_owners` (revision_processor.rs:2422) L2419
- **Called by:** none found in src

#### `add_content_controls_with_owners`  (L2422-2746)
`fn add_content_controls_with_owners( dom: &mut Dom, new_document: NodeId, original: NodeId, paragraph_owners: Option<&HashMap<NodeId, Vec<NodeId>>>, ) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2434,2435,2447,2454,2471,2488...; `Dom::attribute` (xmllinq/mod.rs:874) L2438,2448,2455,2461,2465,2474...; `Dom::element` (xmllinq/mod.rs:655) L2492,2493,2495,2496,2519,2556; `Dom::ancestors` (xmllinq/mod.rs:770) L2503,2522,2531,2592,2598,2641...; `Dom::name` (xmllinq/mod.rs:526) L2507,2526,2540,2581,2591,2596...; `fragmented_source_label_boundary_tests::wrapper` (comparer/finalize.rs:33698) (by name) L2526,2540; `Dom::parent` (xmllinq/mod.rs:597) L2537,2659; `Dom::ancestors_and_self` (xmllinq/mod.rs:787) L2568,2657; `Dom::elements` (xmllinq/mod.rs:640) L2666,2696,2708,2726; `Dom::new_element` (xmllinq/mod.rs:478) L2692,2718,2722; `Dom::attributes` (xmllinq/mod.rs:899) L2693,2719; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L2694,2720; `order_sdt` (revision_processor.rs:2385) L2697,2733; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L2699,2731; `Dom::add` (xmllinq/mod.rs:1074) L2700,2724,2735,2738,2740,2742; `Dom::add_before_self` (xmllinq/mod.rs:1130) L2702; `Dom::remove` (xmllinq/mod.rs:1118) L2703,2716
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.as_ref`, `.collect`, `.contains`, `.copied`, `.count`, `.entry`, `.expect`, `.filter_map`, `.filter`, `.find`, `.first`, `.get`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_some_and`, `.is_some`, `.iter`, `.last`, `.map`, `.nth`, `.or_insert`, `.position`, `.push`, `.retain`, `.reverse`, `.sort_by_key`, `.split`, `.take_while`, `.to_string`, `.to_vec`, `.unwrap_or`, `.unwrap`, `HashMap::new`, `PT::run_ids`, `PT::unique_id`, `Some`, `Vec::new`, `W::del`, `W::ins`, `W::move_from`, `W::move_to`, `W::name`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `W::sdt_content`, `W::sdt`, `W::tbl`, `child_containing`
- **Called by:** `add_block_level_content_controls` (revision_processor.rs:2414); `accept_deleted_and_move_from_paragraph_marks` (revision_processor.rs:2755)

#### `accept_deleted_and_move_from_paragraph_marks`  (L2755-2766, pub)
`pub fn accept_deleted_and_move_from_paragraph_marks(dom: &mut Dom, element: NodeId) -> NodeId`

- **Does:** A.5b — `AcceptDeletedAndMoveFromParagraphMarks` (:2098): annotate runs and content controls (on the ORIGINAL, in place), run the A.5a transform, then re-wrap the content controls the transform stripped.
- **Calls (crate):** `has_deleted_or_moved_from_paragraph_mark` (revision_processor.rs:1995) L2757; `annotate_run_elements_with_id` (revision_processor.rs:2331) L2760; `annotate_paragraph_elements_with_id` (revision_processor.rs:2343) L2761; `annotate_content_controls_with_run_ids` (revision_processor.rs:2369) L2762; `accept_paragraph_marks_with_owners` (revision_processor.rs:2035) L2764; `add_content_controls_with_owners` (revision_processor.rs:2422) L2765
- **Calls (std/external/unresolved):** `HashMap::new`, `Some`
- **Called by:** `accept_content` (revision_processor.rs:3510)

#### `a6_block_level_elements`  (L2772-2783)
`fn a6_block_level_elements() -> [XName; 8]`

- **Does:** `BlockLevelElements` (:2766) — the direct-cell-child names that make a cell non-empty for [`remove_rows_left_empty_by_move_from`].
- **Calls (std/external/unresolved):** `M::name`, `W::del`, `W::ins`, `W::move_to`, `W::p`, `W::sdt`, `W::tbl`
- **Called by:** `remove_rows_left_empty_by_move_from_inner` (revision_processor.rs:2793)

#### `remove_rows_left_empty_by_move_from`  (L2789-2791, pub)
`pub fn remove_rows_left_empty_by_move_from(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** A.6 — `RemoveRowsLeftEmptyByMoveFrom` (:2777): drop every `w:tr` whose cells all lost their block-level content to an accepted moveFrom; rebuild everything else.
- **Calls (crate):** `remove_rows_left_empty_by_move_from_inner` (revision_processor.rs:2793) L2790
- **Calls (std/external/unresolved):** `.expect`
- **Called by:** `accept_content` (revision_processor.rs:3510)

#### `remove_rows_left_empty_by_move_from_inner`  (L2793-2826)
`fn remove_rows_left_empty_by_move_from_inner(dom: &mut Dom, node: NodeId) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L2794; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L2795; `Dom::name` (xmllinq/mod.rs:526) L2797,2803; `a6_block_level_elements` (revision_processor.rs:2772) L2799; `Dom::elements` (xmllinq/mod.rs:640) L2800,2801; `Dom::element` (xmllinq/mod.rs:655) L2809,2822; `Dom::new_element` (xmllinq/mod.rs:478) L2810; `Dom::attributes` (xmllinq/mod.rs:899) L2811; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L2812; `Dom::nodes` (xmllinq/mod.rs:604) L2814; `remove_rows_left_empty_by_move_from_inner` (revision_processor.rs:2793) L2815; `Dom::add` (xmllinq/mod.rs:1074) L2816
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.into_iter`, `.is_none`, `.is_some_and`, `.is_some`, `.unwrap`, `Some`, `W::tbl`, `W::tc`, `W::tr`
- **Called by:** `remove_rows_left_empty_by_move_from` (revision_processor.rs:2789); `remove_rows_left_empty_by_move_from_inner` (revision_processor.rs:2793)

#### `order_tc_pr`  (L2831-2852)
`fn order_tc_pr(name: &XName) -> i32`

- **Does:** `Order_tcPr` (:1466): schema order for rebuilt `w:tcPr` children.
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L2832; `XName::local_name` (xmllinq/mod.rs:208) L2835
- **Called by:** `accept_deleted_cells_transform` (revision_processor.rs:2867); `merge_adjacent_tables` (revision_processor.rs:3227)

#### `accept_deleted_cells_transform`  (L2867-2971, pub)
`pub fn accept_deleted_cells_transform(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** A.7 — `AcceptDeletedCellsTransform` (:2674): inside each `w:tr`, group adjacent children by (deleted-cell?, anchor), where a cell is "deleted" when it carries `w:cellDel` OR the next `w:tc` after it does, and the anchor is the nearest preceding (or self) `w:tc` WITHOUT `cellDel`.
- **Calls (crate):** `element_or_desc_has_name` (revision_processor.rs:183) L2870; `Dom::is_element` (xmllinq/mod.rs:502) L2873; `Dom::name` (xmllinq/mod.rs:526) L2876,2899,2901,2904,2946,2951...; `Dom::new_element` (xmllinq/mod.rs:478) L2878,2911,2940,2953,2957; `Dom::attributes` (xmllinq/mod.rs:899) L2879,2912; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L2880,2913,2941; `Dom::nodes` (xmllinq/mod.rs:604) L2882; `accept_deleted_cells_transform` (revision_processor.rs:2867) L2883; `Dom::add` (xmllinq/mod.rs:1074) L2884,2919,2955,2958,2966,2968; `Dom::descendants` (xmllinq/mod.rs:668) L2891; `Dom::elements` (xmllinq/mod.rs:640) L2893,2943,2959; `Dom::nodes_after_self` (xmllinq/mod.rs:838) L2896; `content_elements_before_self` (revision_processor.rs:1031) L2903; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L2918,2949,2965; `Dom::element` (xmllinq/mod.rs:655) L2930,2933; `Dom::attribute` (xmllinq/mod.rs:874) L2935; `order_tc_pr` (revision_processor.rs:2831) L2951
- **Calls (std/external/unresolved):** `.and_then`, `.chain`, `.clone`, `.collect`, `.expect`, `.filter`, `.find`, `.into_iter`, `.is_empty`, `.is_some_and`, `.len`, `.ok`, `.parse`, `.push`, `.sort_by_key`, `.to_string`, `.unwrap_or`, `.unwrap`, `Some`, `W::cell_del`, `W::grid_span`, `W::tc_pr`, `W::tc`, `W::tr`, `W::val`, `crate::util::group_adjacent`, `has_cell_del`, `key_of`, `std::iter::once`
- **Macros:** `vec!`
- **Called by:** `accept_deleted_cells_transform` (revision_processor.rs:2867); `accept_content` (revision_processor.rs:3510); `revision_boundary_coverage_tests::accepting_deleted_cells_keeps_anchor_payload_and_expands_existing_span` (revision_processor.rs:4267); `revision_boundary_coverage_tests::deleted_cell_anchor_defaults_bad_span_and_orders_all_schema_properties` (revision_processor.rs:4303)

#### `own_tbl_pr_ex`  (L2977-2996)
`fn own_tbl_pr_ex(dom: &Dom, first: NodeId, member: NodeId) -> Vec<NodeId>`

- **Does:** R8: the `tblPrEx`-able properties of `member` that `first` (whose `tblPr` the merged table keeps) does not share.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L2979; `Dom::elements` (xmllinq/mod.rs:640) L2981; `tbl_pr_ex_rank` (revision_processor.rs:3116) L2983; `Dom::serialize_element` (xmllinq/mod.rs:1315) L2990,2994
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter`, `.into_iter`, `.is_some`, `.map`, `.unwrap_or_default`, `W::tbl_pr`, `ex_of`
- **Called by:** `merge_adjacent_tables` (revision_processor.rs:3227)

#### `row_tbl_pr_ex`  (L3000-3018)
`fn row_tbl_pr_ex(dom: &mut Dom, tr: NodeId, own: &[NodeId]) -> NodeId`

- **Does:** R8: row `tr`'s `tblPrEx` with its table's `own` properties under the row's own, in schema order.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L3001; `Dom::elements` (xmllinq/mod.rs:640) L3003; `Dom::name` (xmllinq/mod.rs:526) L3007; `tbl_pr_ex_rank` (revision_processor.rs:3116) L3011; `Dom::new_element` (xmllinq/mod.rs:478) L3012; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L3014; `Dom::add` (xmllinq/mod.rs:1074) L3015
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.iter`, `.len`, `.map`, `.push`, `.sort_by_key`, `.unwrap_or_default`, `.unwrap_or`, `W::name`
- **Called by:** `merge_adjacent_tables` (revision_processor.rs:3227)

#### `wrapped_children`  (L3021-3034)
`fn wrapped_children(dom: &Dom, parent: NodeId, name: &XName) -> Vec<NodeId>`

- **Does:** Matching children in order, through wrappers but not into nested tables.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L3023; `Dom::name_is` (xmllinq/mod.rs:535) L3024; `Dom::name` (xmllinq/mod.rs:526) L3026,3027,3028; `wrapped_children` (revision_processor.rs:3021) L3030
- **Calls (std/external/unresolved):** `.extend`, `.push`, `Some`, `Vec::new`, `W::name`
- **Called by:** `table_rows_all_deleted` (revision_processor.rs:1970); `wrapped_children` (revision_processor.rs:3021); `fix_widths` (revision_processor.rs:3040); `merge_adjacent_tables` (revision_processor.rs:3227)

#### `fix_widths`  (L3040-3079)
`fn fix_widths(dom: &mut Dom, tbl: NodeId) -> NodeId`

- **Does:** A.8 — `FixWidths` (:1484): clone the table and rewrite each `w:tcW`'s `w:w` to the sum of the grid columns its cell spans (per the ORIGINAL table's `tblGrid`).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L3041,3044; `Dom::attribute` (xmllinq/mod.rs:874) L3046,3059,3064; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L3051; `wrapped_children` (revision_processor.rs:3021) L3052,3054; `Dom::element` (xmllinq/mod.rs:655) L3056,3058,3061,3063; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L3074
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.enumerate`, `.expect`, `.filter`, `.flat_map`, `.into_iter`, `.is_some`, `.iter`, `.len`, `.map`, `.ok`, `.parse`, `.sum`, `.to_string`, `.unwrap_or`, `Some`, `W::grid_span`, `W::name`, `W::tc_pr`, `W::tc`, `W::tr`, `W::val`, `std::cmp::min`
- **Called by:** `merge_adjacent_tables` (revision_processor.rs:3227); `revision_boundary_coverage_tests::fix_widths_clamps_spans_and_leaves_widthless_cells_out_of_cursor` (revision_processor.rs:4119)

#### `table_has_revision_marks`  (L3083-3090)
`fn table_has_revision_marks(dom: &Dom, tbl: NodeId) -> bool`

- **Does:** True when a table subtree carries row/cell/run revision marks.
- **Calls (crate):** `element_or_desc_has_name` (revision_processor.rs:183) L3085
- **Calls (std/external/unresolved):** `W::name`
- **Called by:** `subtree_needs_adjacent_table_merge::{fn walk}` (revision_processor.rs:3166); `merge_adjacent_tables` (revision_processor.rs:3227)

#### `tbl_pr_ex_rank`  (L3116-3121)
`fn tbl_pr_ex_rank(dom: &Dom, e: NodeId) -> Option<usize>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L3117; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L3118; `XName::local_name` (xmllinq/mod.rs:208) (by name) L3119
- **Calls (std/external/unresolved):** `.flatten`, `.iter`, `.position`, `.then`
- **Called by:** `own_tbl_pr_ex` (revision_processor.rs:2977); `row_tbl_pr_ex` (revision_processor.rs:3000); `whole_table_key` (revision_processor.rs:3127)

#### `whole_table_key`  (L3127-3142)
`fn whole_table_key(dom: &Dom, tbl: NodeId) -> String`

- **Does:** A table's whole-table properties (its `tblPr` minus what `tblPrEx` can carry), serialized: Word joins adjacent tables whose keys agree.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L3129; `Dom::elements` (xmllinq/mod.rs:640) L3130,3136,3137; `tbl_pr_ex_rank` (revision_processor.rs:3116) L3131; `Dom::serialize_element` (xmllinq/mod.rs:1315) L3132,3138
- **Calls (std/external/unresolved):** `.is_none`, `.push_str`, `Some`, `String::from`, `W::name`, `W::tbl_pr`
- **Called by:** `table_merge_key` (revision_processor.rs:3145)

#### `table_merge_key`  (L3145-3161)
`fn table_merge_key(dom: &Dom, e: NodeId, mode: TableMerge) -> String`

- **Does:** The grouping key of A.8 for `e` (empty: not a table).
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L3146; `whole_table_key` (revision_processor.rs:3127) L3150; `Dom::elements` (xmllinq/mod.rs:640) L3152; `Dom::element` (xmllinq/mod.rs:655) L3155
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_some`, `.to_string`, `Some`, `String::new`, `W::name`, `W::tbl_pr`, `W::tbl`
- **Called by:** `subtree_needs_adjacent_table_merge::{fn walk}` (revision_processor.rs:3166); `merge_adjacent_tables` (revision_processor.rs:3227)

#### `subtree_needs_adjacent_table_merge`  (L3165-3197)
`fn subtree_needs_adjacent_table_merge(dom: &Dom, root: NodeId, mode: TableMerge) -> bool`

- **Does:** True if any element under `root` has ≥2 adjacent direct `w:tbl` children that A.8 merges in `mode`.
- **Calls (crate):** `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90) L3196
- **Called by:** `merge_adjacent_tables` (revision_processor.rs:3227)

#### `subtree_needs_adjacent_table_merge::{fn walk}`  (L3166-3195)
`fn walk(dom: &Dom, id: NodeId, mode: TableMerge) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L3167; `Dom::elements` (xmllinq/mod.rs:640) L3171; `table_merge_key` (revision_processor.rs:3145) L3175; `table_has_revision_marks` (revision_processor.rs:3083) L3186; `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90) L3194
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_empty`, `String::new`
- **Called by:** `element_has_tracked_revisions` (revision_processor.rs:89); `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90); `element_or_desc_has_name` (revision_processor.rs:183); `element_or_desc_has_name::{fn walk}` (revision_processor.rs:184); `has_deleted_or_moved_from_paragraph_mark` (revision_processor.rs:1995); `has_deleted_or_moved_from_paragraph_mark::{fn walk}` (revision_processor.rs:1996); `subtree_needs_adjacent_table_merge` (revision_processor.rs:3165); `subtree_needs_adjacent_table_merge::{fn walk}` (revision_processor.rs:3166); `has_empty_table_cell` (revision_processor.rs:3413); `has_empty_table_cell::{fn walk}` (revision_processor.rs:3416)

#### `merge_adjacent_tables_transform`  (L3215-3217, pub)
`pub fn merge_adjacent_tables_transform(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** A.8 — `MergeAdjacentTablesTransform` (:464): where an element has direct `w:tbl` children, merge each run of ≥2 adjacent tables sharing the same bidiVisual-kind into one table: `tblPr` from the first, `tblGrid` = the diffs of the union of every member's cumulative grid widths, and each member's rows re-fit (`FixWidths...
- **Calls (crate):** `merge_adjacent_tables` (revision_processor.rs:3227) L3216
- **Called by:** `accept_content` (revision_processor.rs:3510); `revision_boundary_coverage_tests::revised_table_union_preserves_rows_payload_and_unrelated_clean_groups` (revision_processor.rs:4053); `revision_boundary_coverage_tests::nested_merge_clones_widthless_cell_and_keeps_control_identity` (revision_processor.rs:4160); `revision_boundary_coverage_tests::table_merge_identity_respects_whole_properties_grid_history_and_bidi` (revision_processor.rs:4225)

#### `merge_adjacent_tables_like_word`  (L3223-3225)
`fn merge_adjacent_tables_like_word(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** R8 — Word's save after Accept / Reject All: adjacent tables alike in every whole-table property are one table in Word's model (72cc9f4ac6, 3d4318d7e9); a later member's own `tblPrEx`-able properties ride on its rows.
- **Calls (crate):** `merge_adjacent_tables` (revision_processor.rs:3227) L3224
- **Called by:** `tidy` (revision_processor/word_save.rs:25); `revision_boundary_coverage_tests::word_save_merges_row_exceptions_with_row_values_taking_precedence` (revision_processor.rs:4184); `revision_boundary_coverage_tests::table_merge_identity_respects_whole_properties_grid_history_and_bidi` (revision_processor.rs:4225)

#### `merge_adjacent_tables`  (L3227-3407)
`fn merge_adjacent_tables(dom: &mut Dom, node: NodeId, mode: TableMerge) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `subtree_needs_adjacent_table_merge` (revision_processor.rs:3165) L3229; `Dom::is_element` (xmllinq/mod.rs:502) L3232; `Dom::name` (xmllinq/mod.rs:526) L3235,3330,3331,3364,3374,3384...; `Dom::element` (xmllinq/mod.rs:655) L3237,3340,3342; `Dom::new_element` (xmllinq/mod.rs:478) L3238,3252,3290,3295,3298,3312...; `Dom::attributes` (xmllinq/mod.rs:899) L3239,3253,3313; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L3240,3254,3299,3314,3371; `Dom::nodes` (xmllinq/mod.rs:604) L3242; `merge_adjacent_tables` (revision_processor.rs:3227) L3243; `Dom::add` (xmllinq/mod.rs:1074) L3244,3260,3268,3293,3300,3302...; `Dom::elements` (xmllinq/mod.rs:640) L3249,3276,3277,3291,3324,3329...; `table_merge_key` (revision_processor.rs:3145) L3250; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L3259,3267,3292,3325,3335,3367...; `table_has_revision_marks` (revision_processor.rs:3083) L3265; `Dom::attribute` (xmllinq/mod.rs:874) L3278,3343; `own_tbl_pr_ex` (revision_processor.rs:2977) L3306; `fix_widths` (revision_processor.rs:3040) L3310; `wrapped_children` (revision_processor.rs:3021) L3311,3339; `row_tbl_pr_ex` (revision_processor.rs:3000) L3317; `order_tc_pr` (revision_processor.rs:2831) L3374; `Dom::replace_with` (xmllinq/mod.rs:1165) L3390,3392
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.chain`, `.clone`, `.collect`, `.copied`, `.count`, `.dedup`, `.enumerate`, `.expect`, `.filter`, `.flat_map`, `.into_iter`, `.is_empty`, `.is_none`, `.iter`, `.len`, `.ok`, `.parse`, `.position`, `.push`, `.sort_by_key`, `.sort_unstable`, `.take_while`, `.to_string`, `.unwrap`, `Some`, `Vec::new`, `W::grid_span`, `W::name`, `W::tbl_pr`, `W::tbl`, `W::tc_pr`, `W::tc`, `W::tr_pr`, `W::tr`, `W::val`, `crate::util::group_adjacent`, `std::iter::once`
- **Called by:** `merge_adjacent_tables_transform` (revision_processor.rs:3215); `merge_adjacent_tables_like_word` (revision_processor.rs:3223); `merge_adjacent_tables` (revision_processor.rs:3227)

#### `has_empty_table_cell`  (L3413-3441)
`fn has_empty_table_cell(dom: &Dom, root: NodeId) -> bool`

- **Does:** True if any `w:tc` under `root` has no element children other than `w:tcPr` (the A.9 empty-cell predicate).
- **Calls (crate):** `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90) L3440
- **Calls (std/external/unresolved):** `W::tc_pr`, `W::tc`
- **Called by:** `accept_content` (revision_processor.rs:3510)

#### `has_empty_table_cell::{fn walk}`  (L3416-3439)
`fn walk(dom: &Dom, id: NodeId, tc: &XName, tcpr: &XName) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L3417,3424; `Dom::child_count` (xmllinq/mod.rs:622) L3422,3433; `Dom::child_at` (xmllinq/mod.rs:632) L3423,3434; `Dom::is_element` (xmllinq/mod.rs:502) L3424; `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90) L3434
- **Calls (std/external/unresolved):** `.is_some_and`
- **Called by:** `element_has_tracked_revisions` (revision_processor.rs:89); `element_has_tracked_revisions::{fn walk}` (revision_processor.rs:90); `element_or_desc_has_name` (revision_processor.rs:183); `element_or_desc_has_name::{fn walk}` (revision_processor.rs:184); `has_deleted_or_moved_from_paragraph_mark` (revision_processor.rs:1995); `has_deleted_or_moved_from_paragraph_mark::{fn walk}` (revision_processor.rs:1996); `subtree_needs_adjacent_table_merge` (revision_processor.rs:3165); `subtree_needs_adjacent_table_merge::{fn walk}` (revision_processor.rs:3166); `has_empty_table_cell` (revision_processor.rs:3413); `has_empty_table_cell::{fn walk}` (revision_processor.rs:3416)

#### `add_empty_paragraph_to_any_empty_cells`  (L3449-3463, pub)
`pub fn add_empty_paragraph_to_any_empty_cells(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** A.9 — `AddEmptyParagraphToAnyEmptyCells` (:1448): a `w:tc` with no element children other than `w:tcPr` gains an empty `w:p`.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L3450; `collect_empty_table_cells` (revision_processor.rs:3465) L3457; `Dom::new_element` (xmllinq/mod.rs:478) L3459; `Dom::add` (xmllinq/mod.rs:1074) L3460
- **Calls (std/external/unresolved):** `Vec::new`, `W::p`, `W::tc_pr`, `W::tc`
- **Called by:** `accept_content` (revision_processor.rs:3510)

#### `collect_empty_table_cells`  (L3465-3493)
`fn collect_empty_table_cells( dom: &Dom, id: NodeId, tc: &XName, tcpr: &XName, out: &mut Vec<NodeId>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L3472,3478; `Dom::child_count` (xmllinq/mod.rs:622) L3476,3487; `Dom::child_at` (xmllinq/mod.rs:632) L3477,3488; `Dom::is_element` (xmllinq/mod.rs:502) L3478,3489; `collect_empty_table_cells` (revision_processor.rs:3465) L3490
- **Calls (std/external/unresolved):** `.is_some_and`, `.push`
- **Called by:** `add_empty_paragraph_to_any_empty_cells` (revision_processor.rs:3449); `collect_empty_table_cells` (revision_processor.rs:3465)

#### `accept_revisions_for_part_content`  (L3506-3508, pub)
`pub fn accept_revisions_for_part_content(dom: &mut Dom, root: NodeId) -> NodeId`

- **Does:** A.10 — `AcceptRevisionsForPart` (:1314) at content scope: the exact 15-step transform order.
- **Calls (crate):** `accept_content` (revision_processor.rs:3510) L3507
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `document_text_view` (debug.rs:2181); `accept_revisions_document` (revision_processor.rs:524); `revision_boundary_coverage_tests::internally_deleted_equations_leave_no_math_shell_but_keep_live_and_empty_math` (revision_processor.rs:4489); `revision_boundary_coverage_tests::display_math_preserves_independent_empty_equation_and_mixed_run_payload` (revision_processor.rs:4569)

#### `accept_content`  (L3510-3589)
`fn accept_content( dom: &mut Dom, root: NodeId, keep_comments: bool, kept: &mut Vec<String>, ) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `element_has_tracked_revisions` (revision_processor.rs:89) L3516; `remove_rsid_transform` (markup_simplifier.rs:154) L3517; `normalize_orphan_moves` (revision_processor.rs:1203) L3526; `keep_wholly_deleted_comments` (revision_processor/comments.rs:96) L3528; `drop_wholly_deleted_comments` (revision_processor/comments.rs:71) L3530; `drop_wholly_deleted_bookmarks` (revision_processor/bookmarks.rs:23) L3531; `drop_wholly_deleted_permissions` (revision_processor.rs:2304) L3532; `fix_up_deleted_or_inserted_field_codes_transform` (revision_processor.rs:1080) L3533; `Dom::descendants` (xmllinq/mod.rs:668) L3534,3583; `accept_move_from_move_to_transform` (revision_processor.rs:128) L3536; `accept_move_from_ranges` (revision_processor.rs:1358) L3540; `accept_paragraph_end_tags_in_move_from_transform` (revision_processor.rs:1579) L3541; `accept_deleted_and_moved_from_content_controls` (revision_processor.rs:1676) L3542; `accept_deleted_and_move_from_paragraph_marks` (revision_processor.rs:2755) L3543; `remove_rows_left_empty_by_move_from` (revision_processor.rs:2789) L3545; `accept_all_other_revisions_transform` (revision_processor.rs:377) L3550; `accept_deleted_cells_transform` (revision_processor.rs:2867) L3554; `merge_adjacent_tables_transform` (revision_processor.rs:3215) L3555; `drop_unpaired_bookmarks` (revision_processor/bookmarks.rs:152) L3556; `has_empty_table_cell` (revision_processor.rs:3413) L3562; `add_empty_paragraph_to_any_empty_cells` (revision_processor.rs:3449) L3563; `Dom::name` (xmllinq/mod.rs:526) L3570; `ensure_comment_paragraphs` (revision_processor.rs:3617) L3571; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L3577; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L3578,3579; `Dom::has_elements` (xmllinq/mod.rs:862) L3584; `Dom::remove` (xmllinq/mod.rs:1118) L3585
- **Calls (std/external/unresolved):** `.expect`, `.extend`, `.is_empty`, `.len`, `PT::run_ids`, `PT::unique_id`, `Some`, `W::move_from`, `W::name`, `W::num_pr`
- **Macros:** `debug_assert_eq!`
- **Called by:** `reject_content` (revision_processor.rs:755); `accept_revisions_for_part_content` (revision_processor.rs:3506); `resolve_with_comments` (revision_processor.rs:3900)

#### `carry_kept_revisions`  (L3597-3613)
`fn carry_kept_revisions(dom: &mut Dom, live: NodeId, reverted: NodeId, first: bool)`

- **Does:** A property element `live` reverting to its saved copy `reverted` keeps the revisions left tracked in it (frozen by a selective resolution): a paragraph mark's insertion or deletion ahead of the run properties (`first`), a row's or a cell's marks after the rest.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L3598; `Dom::name` (xmllinq/mod.rs:526) L3601; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L3601; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L3603; `Dom::add_first` (xmllinq/mod.rs:1101) L3606; `Dom::add` (xmllinq/mod.rs:1074) L3610
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_some_and`, `.map`, `.rev`
- **Called by:** `reject_revisions_for_part_transform` (revision_processor.rs:641)

#### `ensure_comment_paragraphs`  (L3617-3629)
`fn ensure_comment_paragraphs(dom: &mut Dom, root: NodeId)`

- **Does:** Give every `w:comment` under a `w:comments` root without a block child (`w:p`, `w:tbl`, `w:sdt`, `w:customXml`) an empty paragraph (#1223).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L3619,3620; `Dom::name` (xmllinq/mod.rs:526) L3623; `Dom::new_element` (xmllinq/mod.rs:478) L3625; `Dom::add` (xmllinq/mod.rs:1074) L3626
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.into_iter`, `.is_some_and`, `Some`, `W::name`, `W::p`, `W::sdt`, `W::tbl`
- **Called by:** `accept_content` (revision_processor.rs:3510)

#### `accept_revisions_for_styles_transform`  (L3633-3651)
`fn accept_revisions_for_styles_transform(dom: &mut Dom, node: NodeId) -> Option<NodeId>`

- **Does:** A.11 — `AcceptRevisionsForStylesTransform` (:1300): drop `pPrChange`/ `rPrChange` from the styles part, rebuild the rest.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L3634; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L3635; `Dom::name` (xmllinq/mod.rs:526) L3637; `Dom::new_element` (xmllinq/mod.rs:478) L3641; `Dom::attributes` (xmllinq/mod.rs:899) L3642; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L3643; `Dom::nodes` (xmllinq/mod.rs:604) L3645; `accept_revisions_for_styles_transform` (revision_processor.rs:3633) L3646; `Dom::add` (xmllinq/mod.rs:1074) L3647
- **Calls (std/external/unresolved):** `.unwrap`, `Some`, `W::p_pr_change`, `W::r_pr_change`
- **Called by:** `accept_revisions_for_styles_transform` (revision_processor.rs:3633); `resolve_with_comments` (revision_processor.rs:3900); `revision_boundary_coverage_tests::missing_saved_style_properties_drop_changed_block_while_accept_keeps_live_properties` (revision_processor.rs:4717)

#### `reject_revisions_for_styles_transform`  (L3657-3684)
`fn reject_revisions_for_styles_transform(dom: &mut Dom, node: NodeId) -> Option<NodeId>`

- **Does:** A.11 — `RejectRevisionsForStylesTransform` (:391): a `pPr` holding a `pPrChange` reverts to the change's inner `pPr` (same for `rPr`); a change element without its inner properties drops the whole node (C# recurses null).
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L3658; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L3659; `Dom::name` (xmllinq/mod.rs:526) L3661; `Dom::element` (xmllinq/mod.rs:655) L3663,3665,3669,3671; `reject_revisions_for_styles_transform` (revision_processor.rs:3657) L3666,3672,3679; `Dom::new_element` (xmllinq/mod.rs:478) L3674; `Dom::attributes` (xmllinq/mod.rs:899) L3675; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L3676; `Dom::nodes` (xmllinq/mod.rs:604) L3678; `Dom::add` (xmllinq/mod.rs:1074) L3680
- **Calls (std/external/unresolved):** `.and_then`, `.unwrap`, `Some`, `W::p_pr_change`, `W::p_pr`, `W::r_pr_change`, `W::r_pr`
- **Called by:** `reject_revisions_for_styles_transform` (revision_processor.rs:3657); `resolve_with_comments` (revision_processor.rs:3900); `revision_boundary_coverage_tests::missing_saved_style_properties_drop_changed_block_while_accept_keeps_live_properties` (revision_processor.rs:4717)

#### `related_part`  (L3688-3698)
`fn related_part(pkg: &crate::opc::PartFs, rel: &str) -> Option<String>`

- **Does:** The part `rel` of the main part targets (`numbering`, `comments`), resolved against the main part; `None` when the package has none.
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:1070) L3689; `PartFs::read_rels_for` (opc/mod.rs:991) L3690; `PartFs::resolve_rel_target` (opc/mod.rs:829) L3697
- **Calls (std/external/unresolved):** `.as_deref`, `.find`, `.iter`, `.map`, `.next`, `.rsplit`, `Some`
- **Called by:** `numbering_part` (revision_processor.rs:3701); `comments_part` (revision_processor.rs:3708)

#### `numbering_part`  (L3701-3703)
`fn numbering_part(pkg: &crate::opc::PartFs) -> Option<String>`

- **Does:** The main document's numbering part, when it has one.
- **Calls (crate):** `related_part` (revision_processor.rs:3688) L3702
- **Called by:** `resolve_with_comments` (revision_processor.rs:3900)

#### `comments_part`  (L3708-3710, pub(crate))
`pub(crate) fn comments_part(pkg: &crate::opc::PartFs) -> Option<String>`

- **Does:** The main document's comments part (`word/comments.xml`), when it has one.
- **Calls (crate):** `related_part` (revision_processor.rs:3688) L3709
- **Called by:** `for_each_story` (changes.rs:657); `comment_changes_in` (changes.rs:1003); `decisions` (edit/source_refs.rs:25); `resolve_with_comments` (revision_processor.rs:3900)

#### `revision_bearing_parts`  (L3716-3757, pub(crate))
`pub(crate) fn revision_bearing_parts(pkg: &crate::opc::PartFs) -> Vec<(String, bool)>`

- **Does:** The parts `AcceptRevisions`/`RejectRevisions` (:1277/:31) walk, in the C# order — main, headers, footers, endnotes, footnotes, then styles (flagged) — with the comments part Word's own Accept/Reject All also settles (#1223) between the notes and the styles.
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:1070) L3717; `PartFs::read_rels_for` (opc/mod.rs:991) L3726; `PartFs::resolve_rel_target` (opc/mod.rs:829) L3740
- **Calls (std/external/unresolved):** `.as_deref`, `.chain`, `.into_iter`, `.next`, `.push`, `.rsplit`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `Some`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `audit_stories` (changes.rs:168); `tolerant` (changes.rs:620); `for_each_story` (changes.rs:657); `decisions` (edit/source_refs.rs:25); `first_free_id` (edit/tracked.rs:427); `SourceIndex::new` (edit/v2_apply.rs:713); `Transaction::finish` (edit.rs:6645); `RevisionIds::new` (revision_processor.rs:3806); `resolve_with_comments` (revision_processor.rs:3900); `remove_comments` (scrub.rs:221)

#### `process_part`  (L3759-3776)
`fn process_part<F>(pkg: &mut crate::opc::PartFs, part: &str, f: F) where F: FnOnce(&mut Dom, NodeId) -> Option<NodeId>,`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L3763; `Dom::new` (xmllinq/mod.rs:387) L3766; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L3767; `Dom::root` (xmllinq/mod.rs:822) L3768; `Dom::replace_with` (xmllinq/mod.rs:1165) L3774; `PartFs::set_part` (opc/mod.rs:661) L3775; `Dom::serialize_document` (xmllinq/mod.rs:1338) L3775
- **Calls (std/external/unresolved):** `.into_bytes`, `f`
- **Called by:** `resolve_with_comments` (revision_processor.rs:3900)

#### `accept_revisions_package`  (L3781-3783, pub)
`pub fn accept_revisions_package(pkg: &mut crate::opc::PartFs)`

- **Does:** A.11 — `AcceptRevisions` (:1277) at package scope: run the full part pipeline over main + headers + footers + endnotes + footnotes, and the styles transform over the styles part.
- **Calls (crate):** `resolve_package` (revision_processor.rs:3879) L3782
- **Called by:** `accept_revisions` (document_comparer.rs:4962); `accept_source_revisions` (document_comparer.rs:4973)

#### `reject_revisions_package`  (L3790-3792, pub)
`pub fn reject_revisions_package(pkg: &mut crate::opc::PartFs)`

- **Does:** A.11 — `RejectRevisions` (:31) at package scope: per content part, the revert → reverse → rsid-strip → full-accept composition (the C# phases the same steps across all parts; parts are independent, so per-part composition is equivalent); the styles part reverts its property changes then accepts the leftovers.
- **Calls (crate):** `resolve_package` (revision_processor.rs:3879) L3791
- **Called by:** `reject_revisions` (document_comparer.rs:4985)

#### `RevisionIds::new`  (L3806-3828, pub(crate))
`pub(crate) fn new(pkg: &crate::opc::PartFs) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `revision_bearing_parts` (revision_processor.rs:3716) L3808; `PartFs::part_string` (opc/mod.rs:636) L3809; `Dom::new` (xmllinq/mod.rs:387) L3812; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L3813; `Dom::root` (xmllinq/mod.rs:822) L3814; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L3815; `Dom::name` (xmllinq/mod.rs:526) L3816; `has_unique_revision_id` (validate.rs:1684) L3818; `Dom::attribute` (xmllinq/mod.rs:874) L3819; `RevisionIds::from_used` (revision_processor.rs:3830) L3827
- **Calls (std/external/unresolved):** `.and_then`, `.insert`, `.is_some_and`, `.ok`, `.parse`, `HashSet::new`, `W::id`
- **Called by:** `rewrite` (edit/whole.rs:130); `resolve_with_comments` (revision_processor.rs:3900)

#### `RevisionIds::from_used`  (L3830-3833, pub(crate))
`pub(crate) fn from_used(used: HashSet<u64>) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.iter`, `.map_or`, `.max`, `.saturating_add`
- **Called by:** `tests::run` (edit/whole.rs:499); `RevisionIds::new` (revision_processor.rs:3806)

#### `RevisionIds::allocate`  (L3837-3850, pub(crate))
`pub(crate) fn allocate(&mut self) -> u64`

- **Does:** Keep max + 1 until Word's signed 32-bit range is exhausted, then allocate the lowest unused id.
- **Calls (std/external/unresolved):** `.contains`, `.insert`
- **Macros:** `assert!`
- **Called by:** `Stamp::container` (edit/whole.rs:202); `rewrite_one` (edit/whole.rs:229); `clone_with_fresh_property_ids` (revision_processor.rs:3855)

#### `clone_with_fresh_property_ids`  (L3855-3870, pub(crate))
`pub(crate) fn clone_with_fresh_property_ids( dom: &mut Dom, source: NodeId, next_id: &mut RevisionIds, ) -> NodeId`

- **Does:** A copied property history is a distinct record.
- **Calls (crate):** `Dom::clone_subtree` (xmllinq/mod.rs:1231) L3860; `Dom::descendants` (xmllinq/mod.rs:668) L3861; `Dom::name` (xmllinq/mod.rs:526) L3862; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L3863; `XName::local_name` (xmllinq/mod.rs:208) (by name) L3864; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L3866; `RevisionIds::allocate` (revision_processor.rs:3837) L3866
- **Calls (std/external/unresolved):** `.chain`, `.is_some_and`, `.to_string`, `Some`, `W::id`, `std::iter::once`
- **Macros:** `matches!`
- **Called by:** `rewrite_one` (edit/whole.rs:229); `effective_rpr` (revision_processor/style_records.rs:585)

#### `resolve_package`  (L3879-3885, pub(crate))
`pub(crate) fn resolve_package( pkg: &mut crate::opc::PartFs, resolution: Resolution, freeze: Option<Freeze<'_>>, ) -> Vec<String>`

- **Does:** Accept or reject every revision `freeze` leaves in place.
- **Calls (crate):** `resolve_with_comments` (revision_processor.rs:3900) L3884
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `resolve_for_edit_or_cli` (changes.rs:922); `accept_revisions_package` (revision_processor.rs:3781); `reject_revisions_package` (revision_processor.rs:3790)

#### `resolve_package_for_edit`  (L3889-3898, pub(crate))
`pub(crate) fn resolve_package_for_edit( pkg: &mut crate::opc::PartFs, resolution: Resolution, freeze: Option<Freeze<'_>>, keep_comments: bool, ) -> (Vec<String>, Vec<String>)`

- **Does:** [`resolve_package`] for an edit plan, the only entry point that can rescue removed anchors.
- **Calls (crate):** `resolve_with_comments` (revision_processor.rs:3900) L3896
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `resolve_for_edit_or_cli` (changes.rs:922)

#### `resolve_with_comments`  (L3900-3968)
`fn resolve_with_comments( pkg: &mut crate::opc::PartFs, resolution: Resolution, freeze: Option<Freeze<'_>>, keep_comments: bool, kept_comments: &mut Vec<String>, ) -> Vec<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `revision_bearing_parts` (revision_processor.rs:3716) L3907; `RevisionIds::new` (revision_processor.rs:3806) L3908; `numbering_part` (revision_processor.rs:3701) L3909; `PartFs::part_string` (opc/mod.rs:636) L3910; `Levels::parse` (revision_processor/style_records.rs:280) L3911; `comments_part` (revision_processor.rs:3708) L3917; `process_part` (revision_processor.rs:3759) L3921; `element_has_tracked_revisions` (revision_processor.rs:89) L3922; `accept_revisions_for_styles_transform` (revision_processor.rs:3633) L3930,3941; `recorded_blocks` (revision_processor/style_records.rs:368) L3932; `reject_revisions_for_styles_transform` (revision_processor.rs:3657) L3933; `restore_against_built_ins` (revision_processor/style_records.rs:398) L3934; `carry_vanishing_section_references` (revision_processor/sections.rs:31) L3944; `accept_content` (revision_processor.rs:3510) L3947; `reject_content` (revision_processor.rs:755) L3950; `thaw` (revision_processor.rs:3971) L3955; `story_parts` (revision_processor.rs:3986) L3959; `prune_orphan_comments` (revision_processor/comments.rs:274) L3960; `prune_orphan_notes` (revision_processor/notes.rs:24) L3961; `tidy` (revision_processor/word_save.rs:25) L3966
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.clone`, `.collect`, `.filter`, `.into_iter`, `.map`, `.unwrap_or_default`, `Some`, `freeze`
- **Called by:** `resolve_package` (revision_processor.rs:3879); `resolve_package_for_edit` (revision_processor.rs:3889)

#### `thaw`  (L3971-3983)
`fn thaw(dom: &mut Dom, root: NodeId)`

- **Does:** Rename every frozen revision element under `root` back into `w:`.
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L3972; `Dom::name` (xmllinq/mod.rs:526) L3976; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L3977; `XName::local_name` (xmllinq/mod.rs:208) (by name) L3977; `Dom::set_name` (xmllinq/mod.rs:539) L3981
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.into_iter`, `.then`, `.to_string`, `W::name`
- **Called by:** `resolve_with_comments` (revision_processor.rs:3900)

#### `story_parts`  (L3986-3992)
`fn story_parts(parts: &[(String, bool)]) -> Vec<String>`

- **Does:** The content parts of [`revision_bearing_parts`] (styles left out).
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.filter`, `.iter`, `.map`
- **Called by:** `resolve_with_comments` (revision_processor.rs:3900)

## Test functions (55)

- `revision_boundary_coverage_tests::parse` (L3999): _no doc comment_
- `revision_boundary_coverage_tests::paragraph` (L4009): _no doc comment_
- `revision_boundary_coverage_tests::cell` (L4013): _no doc comment_
- `revision_boundary_coverage_tests::table` (L4020): _no doc comment_
- `revision_boundary_coverage_tests::local_children` (L4030): _no doc comment_
- `revision_boundary_coverage_tests::widths` (L4037): _no doc comment_
- `revision_boundary_coverage_tests::span` (L4045): _no doc comment_
- `revision_boundary_coverage_tests::revised_table_union_preserves_rows_payload_and_unrelated_clean_groups` (L4053): _no doc comment_
- `revision_boundary_coverage_tests::fix_widths_clamps_spans_and_leaves_widthless_cells_out_of_cursor` (L4119): _no doc comment_
- `revision_boundary_coverage_tests::nested_merge_clones_widthless_cell_and_keeps_control_identity` (L4160): _no doc comment_
- `revision_boundary_coverage_tests::word_save_merges_row_exceptions_with_row_values_taking_precedence` (L4184): _no doc comment_
- `revision_boundary_coverage_tests::table_merge_identity_respects_whole_properties_grid_history_and_bidi` (L4225): _no doc comment_
- `revision_boundary_coverage_tests::accepting_deleted_cells_keeps_anchor_payload_and_expands_existing_span` (L4267): _no doc comment_
- `revision_boundary_coverage_tests::deleted_cell_anchor_defaults_bad_span_and_orders_all_schema_properties` (L4303): _no doc comment_
- `revision_boundary_coverage_tests::custom_xml_deleted_control_unwraps_nested_controls_but_retains_order_and_anchors` (L4358): _no doc comment_
- `revision_boundary_coverage_tests::custom_xml_move_deletes_whole_control_and_plain_content_but_keeps_outside_control` (L4389): _no doc comment_
- `revision_boundary_coverage_tests::unmatched_custom_ranges_leave_control_payload_and_parent_links_untouched` (L4410): _no doc comment_
- `revision_boundary_coverage_tests::move_range_that_enters_table_keeps_container_properties_and_surviving_content` (L4430): _no doc comment_
- `revision_boundary_coverage_tests::accept_cell_merge_converts_restart_and_continue_only_inside_cell_properties` (L4453): _no doc comment_
- `revision_boundary_coverage_tests::internally_deleted_equations_leave_no_math_shell_but_keep_live_and_empty_math` (L4489): _no doc comment_
- `revision_boundary_coverage_tests::display_math_preserves_independent_empty_equation_and_mixed_run_payload` (L4569): _no doc comment_
- `revision_boundary_coverage_tests::internally_deleted_math_does_not_consume_independent_anchors_controls_or_empty_branches` (L4590): _no doc comment_
- `revision_boundary_coverage_tests::internally_deleted_math_does_not_consume_independent_anchors_controls_or_empty_branches::{fn semantic}` (L4591): _no doc comment_
- `revision_boundary_coverage_tests::wholly_deleted_math_hoists_owned_range_metadata_instead_of_discarding_it` (L4652): _no doc comment_
- `revision_boundary_coverage_tests::accept_deleted_fraction_drops_its_math_payload_and_preserves_live_fraction` (L4680): _no doc comment_
- `revision_boundary_coverage_tests::rejecting_structural_markers_drops_inserted_numbering_and_cell_without_losing_neighbors` (L4694): _no doc comment_
- `revision_boundary_coverage_tests::missing_saved_style_properties_drop_changed_block_while_accept_keeps_live_properties` (L4717): _no doc comment_
- `revision_boundary_coverage_tests::acceptance_removes_empty_numbering_but_keeps_live_numbering_and_clears_atom_annotations` (L4751): _no doc comment_
- `revision_boundary_coverage_tests::unmatched_or_contentless_move_ranges_preserve_identity_and_properties` (L4775): _no doc comment_
- `public_block_control_ownership_tests::package` (L4803): _no doc comment_
- `public_block_control_ownership_tests::resolved` (L4825): _no doc comment_
- `public_block_control_ownership_tests::assert_owner` (L4837): _no doc comment_
- `public_block_control_ownership_tests::public_mark_resolution_rewraps_transformed_paragraphs_with_original_control_metadata` (L4858): _no doc comment_
- `public_block_control_ownership_tests::public_mark_resolution_preserves_whole_table_payload_at_the_controls_block_level` (L4891): _no doc comment_
- `public_block_control_ownership_tests::nested_block_controls_keep_each_owner_and_transformed_paragraph_payload` (L4913): _no doc comment_
- `public_block_control_ownership_tests::nested_block_controls_keep_outer_table_owner_and_nested_cell_geometry` (L4987): _no doc comment_
- `body_equation_paragraph_mark_ownership_tests::equation` (L5043): _no doc comment_
- `body_equation_paragraph_mark_ownership_tests::semantic` (L5053): _no doc comment_
- `body_equation_paragraph_mark_ownership_tests::parse` (L5081): _no doc comment_
- `body_equation_paragraph_mark_ownership_tests::assert_math` (L5087): _no doc comment_
- `body_equation_paragraph_mark_ownership_tests::equation_is_an_opaque_block_barrier_for_deleted_paragraph_marks` (L5111): _no doc comment_
- `body_equation_paragraph_mark_ownership_tests::public_accept_and_reject_preserve_unchanged_body_math_among_revised_marks` (L5128): _no doc comment_
- `body_equation_paragraph_mark_ownership_tests::terminal_internal_equation_revisions_preserve_exact_surviving_paragraph_owners` (L5167): _no doc comment_
- `body_equation_paragraph_mark_ownership_tests::terminal_internal_equation_revisions_preserve_exact_surviving_paragraph_owners::{fn normalize_empty_properties}` (L5168): _no doc comment_
- `deleted_title_equation_barrier_tests::equation` (L5262): _no doc comment_
- `deleted_title_equation_barrier_tests::title` (L5279): _no doc comment_
- `deleted_title_equation_barrier_tests::parse` (L5300): _no doc comment_
- `deleted_title_equation_barrier_tests::semantic` (L5306): _no doc comment_
- `deleted_title_equation_barrier_tests::deleted_title_before_equation_is_removed_without_consuming_the_equation` (L5335): _no doc comment_
- `deleted_title_equation_barrier_tests::public_replaced_titles_and_equations_recover_complete_owned_sources` (L5352): _no doc comment_
- `complete_field_instruction_metadata_boundary_tests::repaired_complete_field_codes_preserve_source_comments_processing_instructions_and_formats` (L5413): _no doc comment_
- `alternate_content_paragraph_owner_tests::root` (L5494): _no doc comment_
- `alternate_content_paragraph_owner_tests::semantic` (L5499): _no doc comment_
- `alternate_content_paragraph_owner_tests::authored_property_only_alternatives_survive_but_wholly_deleted_wrappers_do_not` (L5527): _no doc comment_
- `alternate_content_paragraph_owner_tests::surviving_alternate_content_keeps_both_presentations_across_deleted_paragraph_marks` (L5573): _no doc comment_
