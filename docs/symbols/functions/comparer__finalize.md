# Functions and methods in `src/comparer/finalize.rs`

269 non-test, 493 test. Part of the [symbol manifest](../README.md).

#### `begin_para_classification_cache`  (L49-52, pub)
`pub fn begin_para_classification_cache()`

- **Does:** Enable the per-paragraph pure-del / mixed classification caches the `M4.F` peels share.
- **Calls (std/external/unresolved):** `.borrow_mut`, `.with`, `HashMap::new`, `Some`
- **Called by:** `coverage_tests::classification_cache_hits_and_resets_agree_with_uncached_results` (comparer/finalize.rs:15676); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `end_para_classification_cache`  (L57-60, pub)
`pub fn end_para_classification_cache()`

- **Does:** Drop the thread-local pure-del / mixed paragraph classification caches; called by the pipeline at the end of the peel ladder so the next document run starts with empty caches.
- **Calls (std/external/unresolved):** `.borrow_mut`, `.with`
- **Called by:** `coverage_tests::classification_cache_hits_and_resets_agree_with_uncached_results` (comparer/finalize.rs:15676); `<Reset as Drop>::drop` (comparer/finalize.rs:15679); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `descendants_trimmed`  (L63-75)
`fn descendants_trimmed(dom: &Dom, node: NodeId, stop: &XName) -> Vec<NodeId>`

- **Does:** `DescendantsTrimmed(node, stop)` — descendants, not recursing into `stop`.
- **Calls (crate):** `descendants_trimmed::{fn walk}` (comparer/finalize.rs:65) L73
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `mark_content_transform` (comparer/finalize.rs:192)

#### `descendants_trimmed::{fn walk}`  (L65-72)
`fn walk(dom: &Dom, node: NodeId, stop: &XName, out: &mut Vec<NodeId>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L66; `Dom::is_element` (xmllinq/mod.rs:502) L68; `Dom::name` (xmllinq/mod.rs:526) L68; `descendants_trimmed::{fn walk}` (comparer/finalize.rs:65) L69
- **Calls (std/external/unresolved):** `.as_ref`, `.push`, `Some`
- **Called by:** `descendants_trimmed` (comparer/finalize.rs:63); `descendants_trimmed::{fn walk}` (comparer/finalize.rs:65); `convert_run_text_to_del_text` (comparer/finalize.rs:102); `convert_run_text_to_del_text::{fn walk}` (comparer/finalize.rs:103); `enforce_deleted_text_kinds` (comparer/finalize.rs:14834); `enforce_deleted_text_kinds::{fn walk}` (comparer/finalize.rs:14835)

#### `is_run_status_carrier`  (L77-95)
`fn is_run_status_carrier(dom: &Dom, d: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L78
- **Calls (std/external/unresolved):** `.contains`, `MC::name`, `W::del_text`, `W::name`, `W::pict`, `W::t`
- **Called by:** `mark_content_transform` (comparer/finalize.rs:192)

#### `convert_run_text_to_del_text`  (L102-124)
`fn convert_run_text_to_del_text(dom: &mut Dom, run: NodeId)`

- **Does:** `w:t` inside `w:del` / `w:moveFrom` is non-conformant — Word expects `w:delText` (and `w:delInstrText` for field instructions; Word shows the repair dialog on `w:instrText` under `w:del` — strict01 TOC evidence, and Word's own redlines emit delInstrText).
- **Calls (crate):** `descendants_trimmed::{fn walk}` (comparer/finalize.rs:65) L123
- **Called by:** `mark_content_transform` (comparer/finalize.rs:192); `coverage_tests::deleted_text_conversion_stops_at_nested_revision_owners` (comparer/finalize.rs:15801)

#### `convert_run_text_to_del_text::{fn walk}`  (L103-122)
`fn walk(dom: &mut Dom, node: NodeId)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L104; `Dom::is_element` (xmllinq/mod.rs:502) L105; `Dom::name` (xmllinq/mod.rs:526) L108; `Dom::set_name` (xmllinq/mod.rs:539) L117,118; `descendants_trimmed::{fn walk}` (comparer/finalize.rs:65) L119
- **Calls (std/external/unresolved):** `.as_ref`, `W::del_text`, `W::del`, `W::ins`, `W::instr_text`, `W::name`, `W::t`
- **Called by:** `descendants_trimmed` (comparer/finalize.rs:63); `descendants_trimmed::{fn walk}` (comparer/finalize.rs:65); `convert_run_text_to_del_text` (comparer/finalize.rs:102); `convert_run_text_to_del_text::{fn walk}` (comparer/finalize.rs:103); `enforce_deleted_text_kinds` (comparer/finalize.rs:14834); `enforce_deleted_text_kinds::{fn walk}` (comparer/finalize.rs:14835)

#### `rev_el`  (L126-133)
`fn rev_el(dom: &mut Dom, name: XName, settings: &WmlComparerSettings, id_gen: &mut u32) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L127; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L128,129,131
- **Calls (std/external/unresolved):** `.to_string`, `Some`, `W::author`, `W::date`, `W::id`
- **Called by:** `mark_content_transform` (comparer/finalize.rs:192); `merged_closing_carrier_takes_source_properties` (comparer/finalize.rs:3080); `convert_stamped_preins` (comparer/finalize.rs:10282); `wrap_bare_del_text_runs` (comparer/finalize.rs:10771); `delete_stories_of_deleted_text_boxes` (comparer/finalize.rs:10812); `mark_paragraph_mark_deleted` (comparer/finalize.rs:10903); `convert_stamped_predeletes` (comparer/finalize.rs:11136); `fold_midstream_demo_title_into_numbered_heading_with_source_properties` (comparer/finalize.rs:12619); `mark_tabbed_deleted_titles` (comparer/finalize.rs:15150); `append_empty_ppr_change` (comparer/finalize.rs:15221); `unmerge_comment_list_item` (comparer/finalize.rs:15233)

#### `copy_attrs`  (L136-143)
`fn copy_attrs(dom: &mut Dom, src: NodeId, dst: NodeId, skip_pt: bool)`

- **Does:** Clone `src`'s attributes onto `dst` (optionally skipping the pt namespace).
- **Calls (crate):** `Dom::attributes` (xmllinq/mod.rs:885) L137; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L138; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L141
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `rebuild_run` (comparer/finalize.rs:162); `mark_content_transform` (comparer/finalize.rs:192); `conjoin_transform` (comparer/finalize.rs:585); `coalesce_adjacent_runs` (comparer/finalize.rs:1180); `simplify_move_transform` (comparer/finalize.rs:6813); `split_insertion_at` (comparer/finalize.rs:15028)

#### `copy_move_id_attrs`  (L150-159)
`fn copy_move_id_attrs(dom: &mut Dom, src: NodeId, dst: NodeId)`

- **Does:** Copy only `w:author`, `w:date`, `w:id` from `src` onto `dst` — matches the oracle's `SimplifyMoveMarkupToDelIns` (WmlComparer.cs:2874-2878), which passes `moveFrom.Attribute(W.author) / (W.date) / (W.id)` explicitly when constructing the replacement `w:del` / `w:ins`.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L154; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L156
- **Calls (std/external/unresolved):** `.map`, `Some`, `W::author`, `W::date`, `W::id`
- **Called by:** `simplify_move_transform` (comparer/finalize.rs:6813)

#### `rebuild_run`  (L162-177)
`fn rebuild_run( dom: &mut Dom, element: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, skip_pt: bool, ) -> NodeId`

- **Does:** Rebuild a run, recursing its children through the transform.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L169; `copy_attrs` (comparer/finalize.rs:136) L170; `Dom::nodes` (xmllinq/mod.rs:604) L171; `mark_content_transform` (comparer/finalize.rs:192) L172; `Dom::add` (xmllinq/mod.rs:1058) L173
- **Calls (std/external/unresolved):** `W::r`
- **Called by:** `mark_content_transform` (comparer/finalize.rs:192)

#### `mark_content_transform`  (L192-463, pub)
`pub fn mark_content_transform( dom: &mut Dom, node: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, ) -> Vec<NodeId>`

- **Does:** `MarkContentAsDeletedOrInsertedTransform` (WmlComparer.cs:2568) — the recursive body of `M4.F.1/F.2/F.3`.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L198; `Dom::clone_subtree` (xmllinq/mod.rs:1215) (by name) L199,245; `Dom::name` (xmllinq/mod.rs:526) L201; `Dom::elements` (xmllinq/mod.rs:640) L205; `Dom::attribute` (xmllinq/mod.rs:874) L206,215,267,284,340,363...; `Dom::new_element` (xmllinq/mod.rs:478) L221,238,321,333,345,351...; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L222,223,224,229,239,322...; `rev_el` (comparer/finalize.rs:126) L234,247,296,326,449; `complex_field_runs` (comparer/finalize.rs:14706) L235,253; `Dom::add` (xmllinq/mod.rs:1058) L236,254,301,332,356,357...; `descendants_trimmed` (comparer/finalize.rs:63) L261; `is_run_status_carrier` (comparer/finalize.rs:77) L263; `rebuild_run` (comparer/finalize.rs:162) L277,297,327,341; `convert_run_text_to_del_text` (comparer/finalize.rs:102) L299; `Dom::element` (xmllinq/mod.rs:655) L342,392,400,441; `Dom::add_first` (xmllinq/mod.rs:1085) L346,396,445; `parse_rpr` (comparer/finalize.rs:466) L350,401; `copy_attrs` (comparer/finalize.rs:136) L367,455; `Dom::nodes` (xmllinq/mod.rs:604) L368,456; `mark_content_transform` (comparer/finalize.rs:192) L369,457; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L382,435; `parse_ppr` (comparer/finalize.rs:499) L429
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.as_str`, `.collect`, `.filter`, `.find_map`, `.into_iter`, `.is_empty`, `.is_none`, `.iter`, `.len`, `.map`, `.push`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `.unwrap`, `PT::name`, `PT::status`, `Some`, `Vec::new`, `W::author`, `W::date`, `W::del`, `W::id`, `W::ins`, `W::move_from_range_end`, `W::move_to_range_end`, `W::name`, `W::p_pr_change`, `W::p_pr`, `W::r_pr`, `W::r`, `move_name`
- **Macros:** `panic!`, `vec!`
- **Called by:** `rebuild_run` (comparer/finalize.rs:162); `mark_content_transform` (comparer/finalize.rs:192); `mark_content_as_deleted_or_inserted` (comparer/finalize.rs:517); `run_format_history_regressions::one_new_run_preserves_each_original_format_boundary_when_rejected` (comparer/produce.rs:2064); `run_format_history_regressions::inline_containers_preserve_each_original_format_boundary_when_rejected` (comparer/produce.rs:2145); `nontext_run_format_history_tests::changed_nontext_run_payload_keeps_complete_original_and_revised_properties` (comparer/produce.rs:2684)

#### `parse_rpr`  (L466-475)
`fn parse_rpr(dom: &mut Dom, s: Option<&str>) -> NodeId`

- **Does:** Parse an `OldRPr` attribute string into a `w:rPr` element (empty on failure).
- **Calls (crate):** `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L469; `Dom::root` (xmllinq/mod.rs:822) L470; `drop_pt_bookkeeping` (comparer/finalize.rs:481) L471; `Dom::new_element` (xmllinq/mod.rs:478) L474
- **Calls (std/external/unresolved):** `W::r_pr`
- **Called by:** `mark_content_transform` (comparer/finalize.rs:192)

#### `drop_pt_bookkeeping`  (L481-496)
`fn drop_pt_bookkeeping(dom: &mut Dom, root: NodeId) -> NodeId`

- **Does:** Clone a parsed old-properties element without the scratch `pt14:*` attributes it was serialized with, nor the `xmlns:ns0` declarations the standalone serialization bound them to (they would otherwise survive into the redline).
- **Calls (crate):** `Dom::clone_subtree` (xmllinq/mod.rs:1215) L482; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L483; `Dom::attr_count` (xmllinq/mod.rs:895) L484; `Dom::attr_at` (xmllinq/mod.rs:906) L485; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L487; `Dom::is_namespace_declaration` (xmllinq/mod.rs:996) L487; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L492
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.filter`, `.map`
- **Called by:** `parse_rpr` (comparer/finalize.rs:466); `parse_ppr` (comparer/finalize.rs:499); `closing_paragraph_properties` (comparer/finalize.rs:3051)

#### `parse_ppr`  (L499-507)
`fn parse_ppr(dom: &mut Dom, s: Option<&str>) -> NodeId`

- **Does:** Parse an `OldPPr` attribute string into a `w:pPr` element (empty on failure).
- **Calls (crate):** `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L501; `Dom::root` (xmllinq/mod.rs:822) L502; `drop_pt_bookkeeping` (comparer/finalize.rs:481) L503; `Dom::new_element` (xmllinq/mod.rs:478) L506
- **Calls (std/external/unresolved):** `W::p_pr`
- **Called by:** `mark_content_transform` (comparer/finalize.rs:192)

#### `mark_content_as_deleted_or_inserted`  (L517-527, pub)
`pub fn mark_content_as_deleted_or_inserted( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, ) -> NodeId`

- **Does:** M4.F — run [`mark_content_transform`] over `root` and return the new root.
- **Calls (crate):** `mark_content_transform` (comparer/finalize.rs:192) L523
- **Calls (std/external/unresolved):** `.expect`, `.into_iter`, `.next`
- **Called by:** `coverage_round_next_tests::moved_empty_field_becomes_a_complex_field_in_the_move` (comparer/finalize.rs:20144); `coverage_round_next_tests::deleted_or_inserted_empty_field_becomes_a_complex_field_in_the_revision` (comparer/finalize.rs:20183); `finalize_notes_part` (comparer/footnotes.rs:716); `horizontal_partition_tests::actual_atomized_split_with_deleted_table_conserves_source_through_production` (comparer/lcs_table.rs:1187); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `ppr_has_structural_props`  (L531-546)
`fn ppr_has_structural_props(dom: &Dom, ppr: NodeId) -> bool`

- **Does:** True when pPr carries structural body props (pStyle/numPr/spacing/jc/ind/…).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L532; `Dom::name` (xmllinq/mod.rs:526) L533; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L539
- **Calls (std/external/unresolved):** `W::p_pr_change`, `W::r_pr`, `W::sect_pr`
- **Called by:** `conjoin_transform` (comparer/finalize.rs:585); `fold_whitespace_pure_ins_into_following_pure_del` (comparer/finalize.rs:4211); `merge_replaced_in_container` (comparer/finalize.rs:8621); `coverage_round_next_tests::structural_property_classifiers_ignore_history_and_require_one_alignment` (comparer/finalize.rs:20063)

#### `ppr_is_jc_only`  (L549-572)
`fn ppr_is_jc_only(dom: &Dom, ppr: NodeId) -> bool`

- **Does:** True when the only structural child of pPr is `w:jc` (center-align demos).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L551; `Dom::name` (xmllinq/mod.rs:526) L552; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L558
- **Calls (std/external/unresolved):** `W::jc_el`, `W::p_pr_change`, `W::r_pr`, `W::sect_pr`
- **Called by:** `merge_replaced_in_container` (comparer/finalize.rs:8621); `coverage_round_next_tests::structural_property_classifiers_ignore_history_and_require_one_alignment` (comparer/finalize.rs:20063)

#### `conjoin_transform`  (L585-692)
`fn conjoin_transform(dom: &mut Dom, node: NodeId, author: &str, date: &str) -> NodeId`

- **Does:** M4.F.4 — `ConjoinTransform` (:2947): collapse a `w:p` with ≥2 `w:pPr` into one cleaned pPr + all non-pPr children.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L586; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L587,631,656; `Dom::name` (xmllinq/mod.rs:526) L589,638,660,678; `Dom::elements` (xmllinq/mod.rs:640) L590,591,636,637,643,658...; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L594,600,620,626; `ppr_has_structural_props` (comparer/finalize.rs:531) L607,609,610; `Dom::remove` (xmllinq/mod.rs:1102) L640,644,662; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L648,657,667,668,669; `Dom::element` (xmllinq/mod.rs:655) L654; `Dom::new_element` (xmllinq/mod.rs:478) L666,674,685; `Dom::add` (xmllinq/mod.rs:1058) L670,671,676,680,689; `copy_attrs` (comparer/finalize.rs:136) L675,686; `conjoin_transform` (comparer/finalize.rs:585) L679,688; `Dom::nodes` (xmllinq/mod.rs:604) L687
- **Calls (std/external/unresolved):** `.copied`, `.find`, `.is_empty`, `.is_none`, `.iter`, `.len`, `.position`, `.saturating_sub`, `.unwrap`, `PT::status`, `Some`, `W::author`, `W::date`, `W::del`, `W::id`, `W::ins`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`
- **Macros:** `matches!`
- **Called by:** `conjoin_transform` (comparer/finalize.rs:585); `conjoin_paragraph_marks` (comparer/finalize.rs:702)

#### `conjoin_paragraph_marks`  (L702-713, pub)
`pub fn conjoin_paragraph_marks( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, ) -> NodeId`

- **Does:** M4.F.4 — apply the Conjoin transform to `root`.
- **Calls (crate):** `conjoin_transform` (comparer/finalize.rs:585) L707
- **Called by:** `coverage_round_next_tests::conjoin_property_selection_retains_format_and_does_not_duplicate_history` (comparer/finalize.rs:20088); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `fix_up_revision_ids`  (L725-811, pub)
`pub fn fix_up_revision_ids(dom: &mut Dom, roots: &[NodeId])`

- **Does:** M4.F.5 — `FixUpRevisionIds` (WmlComparer.cs:2769): renumber every `w:id` of every revision element under each `root` from `1`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L760; `Dom::name` (xmllinq/mod.rs:526) L761,798; `Dom::attribute` (xmllinq/mod.rs:874) L765,795; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L809
- **Calls (std/external/unresolved):** `.and_then`, `.contains`, `.get`, `.insert`, `.map`, `.ok`, `.parse`, `.push`, `.to_string`, `.unwrap`, `Some`, `Vec::new`, `W::cell_del`, `W::cell_ins`, `W::del`, `W::id`, `W::ins`, `W::move_from_range_end`, `W::move_to_range_end`, `W::name`, `W::p_pr_change`, `alloc`, `std::collections::HashMap::new`, `std::collections::HashSet::new`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209); `compare_documents_impl` (document_comparer.rs:6501)

#### `ignore_pt14_namespace`  (L819-831, pub)
`pub fn ignore_pt14_namespace(dom: &mut Dom, root: NodeId)`

- **Does:** M4.F.7 — `IgnorePt14Namespace` (WmlComparer.cs:2912): declare `xmlns:pt14` on `root` and add `pt14` to `mc:Ignorable` (idempotent).
- **Calls (crate):** `XNamespace::xmlns` (xmllinq/mod.rs:122) L820; `Dom::attribute` (xmllinq/mod.rs:874) L821,825; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L822,829
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.is_none`, `.join`, `.name`, `.push`, `.split_whitespace`, `.to_string`, `.unwrap_or`, `MC::name`, `Some`
- **Called by:** `finalize_notes_part` (comparer/footnotes.rs:716); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209); `pre_process_markup` (document_comparer.rs:4713)

#### `declare_extension_namespaces_ignorable`  (L899-958, pub)
`pub fn declare_extension_namespaces_ignorable(dom: &mut Dom, root: NodeId)`

- **Does:** Elements merged in from the other document (B's numbering definitions, styles, notes) keep their Word 2010+ extension attributes but not the source root's `mc:Ignorable` contract, so the part claims markup it never declared ignorable (`w15:restartNumberingAfterBreak`, `w16cid:durableId` under a Mac-era numbering root).
- **Calls (crate):** `XNamespace::xmlns` (xmllinq/mod.rs:122) L900; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L902; `Dom::name` (xmllinq/mod.rs:526) L911; `existing_mark_layout_ownership_tests::mark` (comparer/finalize.rs:32802) L912,916; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L912,916; `Dom::attributes` (xmllinq/mod.rs:885) L914,949; `Dom::is_namespace_declaration` (xmllinq/mod.rs:996) L915,952; `Dom::attribute` (xmllinq/mod.rs:874) L924,935,948; `XNamespace::name` (xmllinq/mod.rs:134) L935,940,947; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L940,954,956
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.collect`, `.contains`, `.enumerate`, `.is_none`, `.iter`, `.join`, `.len`, `.map`, `.position`, `.push`, `.split_whitespace`, `.to_string`, `.unwrap_or`, `MC::name`, `Some`
- **Called by:** `coverage_final_batch_finalize_tests::compatibility_bindings_keep_conflicts_and_recognize_attribute_only_extensions` (comparer/finalize.rs:24621); `compare_documents_impl` (document_comparer.rs:6501)

#### `bind_compatibility_prefixes`  (L983-1009, pub)
`pub fn bind_compatibility_prefixes(dom: &mut Dom, root: NodeId)`

- **Does:** Binds every prefix an MC prefix list names (`mc:Choice/@Requires`, `mc:Ignorable`, `mc:ProcessContent`, …) where the list names it.
- **Calls (crate):** `XNamespace::xmlns` (xmllinq/mod.rs:122) L985; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L986; `Dom::name` (xmllinq/mod.rs:526) L987; `Dom::attributes` (xmllinq/mod.rs:885) L990; `is_namespace_prefix_list` (xmllinq/serialize.rs:365) L993; `prefix_in_scope` (comparer/finalize.rs:1448) L999; `well_known_namespace` (xmllinq/serialize.rs:190) L1002; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1005; `XNamespace::name` (xmllinq/mod.rs:134) L1005
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.map_or`, `.map`, `.split_once`, `.split_whitespace`, `.to_string`, `Some`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `repair` (validate.rs:1649)

#### `remove_powertools_scratch_markup`  (L1018-1030, pub)
`pub fn remove_powertools_scratch_markup(dom: &mut Dom, root: NodeId)`

- **Does:** Strip every `pt:*` attribute across `root` and its descendants.
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L1019; `Dom::attributes` (xmllinq/mod.rs:885) L1020; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1024; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1027
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.map`
- **Called by:** `make_empty_pure_ins_para` (comparer/finalize.rs:3783); `paired_replacement_has_identical_properties` (comparer/finalize.rs:8524); `sectpr_identity` (comparer/finalize.rs:11604); `final_peel_has_identical_closing_properties` (comparer/finalize.rs:12491); `coverage_real_revision_merge_matrix_tests::projected` (comparer/finalize.rs:24791); `coverage_real_revision_merge_matrix_tests::guarded_multi_original_complete_projection` (comparer/finalize.rs:25599); `coverage_real_revision_merge_matrix_tests::organic_source_snapshot` (comparer/finalize.rs:26876); `paired_row_mark_ownership_tests::public_empty_table_comparison_preserves_complete_authored_rows_and_pilcrows::{fn source_table}` (comparer/finalize.rs:29014); `closing_authored_section_boundary_tests::projection` (comparer/finalize.rs:29175); `finalize_notes_part` (comparer/footnotes.rs:716); `paired_note_format_history_tests::paired_note_content_format_history_respects_word_and_faithful_modes` (comparer/footnotes.rs:1970); `nested_paragraph_property_source_history_tests::clean` (comparer/formatchg.rs:788); +6 more

#### `xml_space_attr`  (L1040-1046)
`fn xml_space_attr(text: &str) -> Option<&'static str>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chars`, `.is_whitespace`, `.last`, `.next`, `Some`
- **Called by:** `coalesce_adjacent_runs` (comparer/finalize.rs:1180); `split_insertion_at` (comparer/finalize.rs:15028)

#### `rpr_string`  (L1048-1053)
`fn rpr_string(dom: &Dom, r: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1049; `Dom::serialize_element` (xmllinq/mod.rs:1299) L1050
- **Calls (std/external/unresolved):** `String::new`, `W::r_pr`
- **Called by:** `coalesce_key` (comparer/finalize.rs:1055); `mesh_short_label_shared_eq_with_settings::{fn label_run}` (comparer/finalize.rs:7317)

#### `coalesce_key`  (L1055-1151)
`fn coalesce_key(dom: &Dom, ce: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L1056; `Dom::elements` (xmllinq/mod.rs:640) L1060,1103,1106,1109,1119,1130; `Dom::name_is` (xmllinq/mod.rs:535) L1063,1107; `Dom::attribute` (xmllinq/mod.rs:874) L1068,1086,1117,1118,1145; `rpr_string` (comparer/finalize.rs:1048) L1089; `Dom::element` (xmllinq/mod.rs:655) L1090,1093,1112,1123; `Dom::serialize_element` (xmllinq/mod.rs:1299) L1124
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.count`, `.filter_map`, `.filter`, `.flat_map`, `.into_iter`, `.is_some`, `.iter`, `.join`, `.map`, `.to_string`, `.unwrap_or`, `PT::name`, `Some`, `W::author`, `W::date`, `W::del_text`, `W::del`, `W::instr_text`, `W::r_pr`, `W::r`, `W::t`
- **Macros:** `format!`
- **Called by:** `coalesce_adjacent_runs` (comparer/finalize.rs:1180)

#### `run_text_concat`  (L1153-1162)
`fn run_text_concat(dom: &Dom, r: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1155; `Dom::name` (xmllinq/mod.rs:526) L1156; `Dom::value_str` (xmllinq/mod.rs:1179) L1158
- **Calls (std/external/unresolved):** `.push_str`, `.unwrap`, `String::new`, `W::del_text`, `W::instr_text`, `W::t`
- **Called by:** `coalesce_adjacent_runs` (comparer/finalize.rs:1180)

#### `coalesce_adjacent_runs`  (L1180-1244, pub)
`pub fn coalesce_adjacent_runs(dom: &mut Dom, container: NodeId) -> NodeId`

- **Does:** Merge adjacent `w:r` and `w:del` runs of identical text + formatting into one — port of `CoalesceAdjacentRunsWithIdenticalFormatting` (PtOpenXmlUtil.ts:4458).
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L1181,1197; `Dom::elements` (xmllinq/mod.rs:640) L1182; `coalesce_key` (comparer/finalize.rs:1055) L1183; `Dom::new_element` (xmllinq/mod.rs:478) L1185,1199,1210,1218,1220,1228; `copy_attrs` (comparer/finalize.rs:136) L1186,1200,1219,1222; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L1190,1202,1224,1238; `Dom::add` (xmllinq/mod.rs:1058) L1191,1203,1215,1216,1225,1233...; `run_text_concat` (comparer/finalize.rs:1153) L1195; `Dom::element` (xmllinq/mod.rs:655) L1201,1205,1221,1223; `xml_space_attr` (comparer/finalize.rs:1040) L1211,1229; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1212,1230; `XNamespace::xml` (xmllinq/mod.rs:128) L1212,1230; `Dom::add_text` (xmllinq/mod.rs:1071) L1214,1232
- **Calls (std/external/unresolved):** `.collect`, `.is_some`, `.iter`, `.map`, `.name`, `.unwrap`, `Some`, `W::del_text`, `W::del`, `W::instr_text`, `W::r_pr`, `W::r`, `W::t`, `crate::util::group_adjacent`
- **Called by:** `coalesce_all_paragraphs` (comparer/finalize.rs:1461)

#### `resolve_alternate_content`  (L1259-1303, pub)
`pub fn resolve_alternate_content(dom: &mut Dom, root: NodeId)`

- **Does:** Resolve feature-gating `mc:AlternateContent` against the MCE rule (ISO/IEC 29500-3 §10.1.2).
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L1260; `Dom::parent` (xmllinq/mod.rs:597) L1262; `Dom::descendants` (xmllinq/mod.rs:668) L1266,1267; `Dom::ancestors_and_self` (xmllinq/mod.rs:787) L1277; `Dom::name_is` (xmllinq/mod.rs:535) L1280; `Dom::elements` (xmllinq/mod.rs:640) L1293; `choice_requires_understood` (comparer/finalize.rs:1437) L1296; `Dom::element` (xmllinq/mod.rs:655) L1297; `Dom::nodes` (xmllinq/mod.rs:604) L1299; `Dom::replace_with` (xmllinq/mod.rs:1149) L1300
- **Calls (std/external/unresolved):** `.any`, `.find`, `.into_iter`, `.is_empty`, `.is_none`, `.or_else`, `MC::name`, `Some`, `W::drawing`, `W::pict`
- **Called by:** `coverage_tests::alternate_content_selects_first_bound_choice_or_fallback` (comparer/finalize.rs:15905); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `sanitize_sdt_properties`  (L1312-1347, pub)
`pub fn sanitize_sdt_properties(dom: &mut Dom, root: NodeId)`

- **Does:** Strip non-standard `w:`-namespace children from `<w:sdtPr>`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1337; `Dom::elements` (xmllinq/mod.rs:640) L1338; `Dom::name` (xmllinq/mod.rs:526) L1339; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1340; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1341; `Dom::remove` (xmllinq/mod.rs:1102) L1343
- **Calls (std/external/unresolved):** `.contains`, `Some`, `W::name`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `unwrap_content_controls`  (L1358-1403, pub)
`pub fn unwrap_content_controls(dom: &mut Dom, root: NodeId)`

- **Does:** Unwrap every `w:sdt` under `root` to its `sdtContent` children, deepest first.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1360,1369; `Dom::parent` (xmllinq/mod.rs:597) L1381,1391; `Dom::element` (xmllinq/mod.rs:655) L1384; `Dom::nodes` (xmllinq/mod.rs:604) L1385; `Dom::remove` (xmllinq/mod.rs:1102) L1392,1396; `Dom::add_before_self` (xmllinq/mod.rs:1114) L1393
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.copied`, `.filter`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `Some`, `Vec::new`, `W::sdt_content`, `W::sdt`
- **Called by:** `unwrap_content_controls_in_pure_revisions` (comparer/finalize.rs:1412); `coverage_tests::content_controls_unwrap_deepest_first_and_keep_text_order` (comparer/finalize.rs:15875)

#### `unwrap_content_controls_in_pure_revisions`  (L1412-1431, pub)
`pub fn unwrap_content_controls_in_pure_revisions(dom: &mut Dom, root: NodeId)`

- **Does:** M390 — [`unwrap_content_controls`], but only under pure-I / pure-D / MIX residual paragraphs.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1413; `Dom::descendants` (xmllinq/mod.rs:668) L1416,1418,1420,1426; `para_mark_revision` (comparer/finalize.rs:7626) L1419,1421; `unwrap_content_controls` (comparer/finalize.rs:1358) L1429
- **Calls (std/external/unresolved):** `.is_empty`, `Some`, `W::body`, `W::del`, `W::ins`, `W::p`, `W::sdt`
- **Called by:** `coverage_tests::residual_control_unwrap_keeps_equal_form_controls` (comparer/finalize.rs:15890); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `choice_requires_understood`  (L1437-1444)
`fn choice_requires_understood(dom: &Dom, choice: NodeId) -> bool`

- **Does:** True if every namespace prefix listed in a `mc:Choice`'s `@Requires` is bound by an in-scope `xmlns:*` declaration.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L1438; `XNamespace::none` (xmllinq/mod.rs:116) L1438; `prefix_in_scope` (comparer/finalize.rs:1448) L1443
- **Calls (std/external/unresolved):** `.all`, `.name`, `.split_whitespace`
- **Called by:** `resolve_alternate_content` (comparer/finalize.rs:1259)

#### `prefix_in_scope`  (L1448-1454)
`fn prefix_in_scope(dom: &Dom, node: NodeId, prefix: &str) -> bool`

- **Does:** True if `prefix` is bound by an `xmlns:prefix` declaration on `node` or any of its ancestors (XML namespace scoping).
- **Calls (crate):** `Dom::ancestors_and_self` (xmllinq/mod.rs:787) L1449; `Dom::attributes` (xmllinq/mod.rs:885) L1450; `Dom::is_namespace_declaration` (xmllinq/mod.rs:996) L1452; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1452
- **Calls (std/external/unresolved):** `.any`, `.iter`
- **Called by:** `bind_compatibility_prefixes` (comparer/finalize.rs:983); `choice_requires_understood` (comparer/finalize.rs:1437)

#### `coalesce_all_paragraphs`  (L1461-1472, pub)
`pub fn coalesce_all_paragraphs(dom: &mut Dom, root: NodeId)`

- **Does:** Apply [`coalesce_adjacent_runs`] to every `w:p` under `root`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1462; `coalesce_adjacent_runs` (comparer/finalize.rs:1180) L1464; `Dom::nodes` (xmllinq/mod.rs:604) L1465; `Dom::remove_nodes` (xmllinq/mod.rs:1094) L1467; `Dom::add` (xmllinq/mod.rs:1058) L1469
- **Calls (std/external/unresolved):** `Some`, `W::p`
- **Called by:** `finalize_notes_part` (comparer/footnotes.rs:716); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `fix_paragraph_mark_revision_order`  (L1484-1515, pub)
`pub fn fix_paragraph_mark_revision_order(dom: &mut Dom, root: NodeId)`

- **Does:** Enforce the OOXML child order for paragraph-mark revision markers.
- **Calls (crate):** `XName::local_name` (xmllinq/mod.rs:208) (by name) L1487; `Dom::descendants` (xmllinq/mod.rs:668) L1491; `Dom::element` (xmllinq/mod.rs:655) L1492,1506,1508; `Dom::elements` (xmllinq/mod.rs:640) L1496; `is_marker` (markdown/from_docx/revise.rs:46) (by name) L1499; `Dom::remove` (xmllinq/mod.rs:1102) L1502,1509; `Dom::add_first` (xmllinq/mod.rs:1085) L1503; `Dom::add_before_self` (xmllinq/mod.rs:1114) L1511; `Dom::add` (xmllinq/mod.rs:1058) L1512
- **Calls (std/external/unresolved):** `.as_ref`, `.collect`, `.filter`, `.into_iter`, `.map`, `.name`, `.or_else`, `.rev`, `Some`, `W::name`, `W::p_pr_change`, `W::p_pr`, `W::sect_pr`
- **Macros:** `matches!`
- **Called by:** `finalize_notes_part` (comparer/footnotes.rs:716); `produce_note_redline` (comparer/footnotes.rs:1015); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `move_paragraph_properties_first`  (L1524-1542, pub)
`pub fn move_paragraph_properties_first(dom: &mut Dom, node: NodeId)`

- **Does:** Move every paragraph's `w:pPr` to the FIRST child position (OOXML schema requires `w:pPr` first under `w:p`).
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L1525,1529; `Dom::nodes` (xmllinq/mod.rs:604) L1526,1537; `Dom::remove` (xmllinq/mod.rs:1102) L1533; `Dom::add_first` (xmllinq/mod.rs:1085) L1534; `Dom::is_element` (xmllinq/mod.rs:502) L1538; `move_paragraph_properties_first` (comparer/finalize.rs:1524) L1539
- **Calls (std/external/unresolved):** `.as_ref`, `.iter`, `.position`, `Some`, `W::p_pr`, `W::p`
- **Called by:** `move_paragraph_properties_first` (comparer/finalize.rs:1524); `finalize_notes_part` (comparer/footnotes.rs:716); `produce_note_redline` (comparer/footnotes.rs:1015); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209); `coverage_ruby_math_ownership::ruby_reassembly_retains_reading_base_and_property_owners` (comparer/produce.rs:2538)

#### `unwrap_hyperlinks_to_styled_runs`  (L1555-1594, pub)
`pub fn unwrap_hyperlinks_to_styled_runs(dom: &mut Dom, root: NodeId)`

- **Does:** Word Compare unwraps TOC/body `w:hyperlink` wrappers into runs with `w:rStyle w:val="Hyperlink"` (file_21: Word has 0 hyperlinks + 251 Hyperlink-styled runs; we kept 133 live hyperlinks).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1556,1563; `Dom::attribute` (xmllinq/mod.rs:874) L1559; `Dom::element` (xmllinq/mod.rs:655) L1565,1573; `Dom::new_element` (xmllinq/mod.rs:478) L1568,1574; `Dom::add_first` (xmllinq/mod.rs:1085) L1569,1576; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1575; `wrap_revised_hyperlink_as_field` (comparer/finalize.rs:1599) L1585; `Dom::nodes` (xmllinq/mod.rs:604) L1587; `Dom::remove` (xmllinq/mod.rs:1102) L1589,1592; `Dom::add_before_self` (xmllinq/mod.rs:1114) L1590
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_none`, `R::name`, `Some`, `W::hyperlink`, `W::name`, `W::r_pr`, `W::r`, `W::val`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `wrap_revised_hyperlink_as_field`  (L1599-1691)
`fn wrap_revised_hyperlink_as_field(dom: &mut Dom, hl: NodeId)`

- **Does:** M470 helper — when `hl`'s text content is wholly deleted (or wholly inserted), synthesize the HYPERLINK field wrapper runs inside the same revision context.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L1600,1647; `Dom::descendants` (xmllinq/mod.rs:668) L1603,1610,1611; `Dom::value_str` (xmllinq/mod.rs:1179) L1604; `Dom::ancestors_and_self` (xmllinq/mod.rs:787) L1605; `Dom::name_is` (xmllinq/mod.rs:535) L1608,1614,1618; `Dom::parent` (xmllinq/mod.rs:597) L1612; `Dom::new_element` (xmllinq/mod.rs:478) L1635,1636,1642,1643; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1637; `Dom::add` (xmllinq/mod.rs:1058) L1638,1652,1673,1684,1685,1686...; `Dom::add_text` (xmllinq/mod.rs:1071) L1651; `Dom::add_before_self` (xmllinq/mod.rs:1114) L1659,1660,1661,1679,1680,1681; `Dom::add_after_self` (xmllinq/mod.rs:1131) L1662; `Dom::elements` (xmllinq/mod.rs:640) L1668,1677; `Dom::add_first` (xmllinq/mod.rs:1085) L1670,1671,1672
- **Calls (std/external/unresolved):** `.any`, `.copied`, `.filter`, `.first`, `.is_empty`, `.is_some`, `.iter`, `.last`, `.map`, `.push_str`, `.to_string`, `.trim`, `Some`, `W::del_text`, `W::del`, `W::ins`, `W::name`, `W::r`, `W::t`, `mk_fld`
- **Macros:** `format!`
- **Called by:** `unwrap_hyperlinks_to_styled_runs` (comparer/finalize.rs:1555); `deep_production_path_tests::revised_internal_hyperlink_fields_preserve_result_text_and_revision_attribution` (comparer/finalize.rs:22560)

#### `fold_short_list_label_into_empty_pure_del`  (L1702-1768, pub)
`pub fn fold_short_list_label_into_empty_pure_del(dom: &mut Dom, root: NodeId)`

- **Does:** M440 — fold a short pure-I numbered list label (`a` / `test` / `b`, ≤ 2 atoms) sitting directly before a mark-only pure-D into a MIX `w:del` on the next content pure-D.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1703,1744,1747; `Dom::elements` (xmllinq/mod.rs:640) L1707,1749,1755; `Dom::name_is` (xmllinq/mod.rs:535) L1710,1716,1738,1756; `para_is_pure_inserted` (comparer/finalize.rs:4175) L1719; `para_is_pure_deleted` (comparer/finalize.rs:2858) L1719,1738; `para_has_real_del` (comparer/finalize.rs:7849) L1723,1738; `para_has_no_text` (comparer/finalize.rs:3459) L1723; `para_mark_revision` (comparer/finalize.rs:7626) L1726; `para_has_live_numpr` (comparer/finalize.rs:7990) L1730; `para_word_atom_count` (comparer/finalize.rs:8008) L1733; `Dom::remove` (xmllinq/mod.rs:1102) L1745,1760; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L1748; `Dom::add_before_self` (xmllinq/mod.rs:1114) L1750; `Dom::add` (xmllinq/mod.rs:1058) L1752,1757
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.contains`, `.copied`, `.filter`, `.first`, `.into_iter`, `.iter`, `.len`, `.saturating_sub`, `W::body`, `W::del`, `W::p_pr`, `W::p`, `W::sect_pr`
- **Called by:** `coverage_tests::short_list_label_folds_only_into_empty_deleted_mark_with_deleted_tail` (comparer/finalize.rs:16446); `finalizer_distinct_boundary_contract_tests::the_short_list_label_uses_only_an_empty_deleted_carrier_with_continuation` (comparer/finalize.rs:30275); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `strip_trailing_bare_empty_after_pure_i_dominant`  (L1787-1839, pub)
`pub fn strip_trailing_bare_empty_after_pure_i_dominant(dom: &mut Dom, root: NodeId)`

- **Does:** M448 (diff_after8 × doc_with_spacing ~75.6 / docxodus 100): Word ends pure-I-dominant body with pure-D residual only (`…IDD`).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1788,1813,1816; `Dom::elements` (xmllinq/mod.rs:640) L1791; `Dom::name_is` (xmllinq/mod.rs:535) L1794,1800,1820,1826; `para_has_no_text` (comparer/finalize.rs:3459) L1803; `Dom::descendants` (xmllinq/mod.rs:668) L1806,1807; `para_is_pure_deleted` (comparer/finalize.rs:2858) L1820,1831; `para_is_pure_inserted` (comparer/finalize.rs:4175) L1829; `Dom::remove` (xmllinq/mod.rs:1102) L1838
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter`, `.into_iter`, `.is_empty`, `.is_none`, `.len`, `Some`, `W::body`, `W::del`, `W::ins`, `W::p_pr`, `W::p`, `W::sect_pr`, `W::spacing_el`
- **Called by:** `coverage_tests::trailing_spaced_empty_requires_insert_dominance_and_deleted_predecessor` (comparer/finalize.rs:16578); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `promote_live_numpr_on_pure_d_from_pprchange`  (L1866-1966, pub)
`pub fn promote_live_numpr_on_pure_d_from_pprchange(dom: &mut Dom, root: NodeId)`

- **Does:** M442 (list_spacer1 × list_with_break_exported_broken residual ~80.8 / docxodus 94.6): last pure-D "14.11…" has Word live `numPr` (numId from the pure-I list residual) + `pPrChange` parking base StandardL1/numId=0.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1867,1890,1891,1892,1923,1924...; `Dom::elements` (xmllinq/mod.rs:640) L1878,1907; `Dom::name_is` (xmllinq/mod.rs:535) L1879,1908; `para_is_pure_inserted` (comparer/finalize.rs:4175) L1879; `para_has_live_numpr` (comparer/finalize.rs:7990) L1882; `para_word_atom_count` (comparer/finalize.rs:8008) L1885; `Dom::attribute` (xmllinq/mod.rs:874) L1893,1925,1953; `para_is_pure_deleted` (comparer/finalize.rs:2858) L1911; `para_has_real_del` (comparer/finalize.rs:7849) L1911; `para_revision_body_text` (comparer/finalize.rs:7854) L1916; `body_looks_like_list_residual_label` (comparer/finalize.rs:1991) L1917; `Dom::new_element` (xmllinq/mod.rs:478) L1956,1957,1959; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1958,1960; `Dom::add` (xmllinq/mod.rs:1058) L1961,1962; `Dom::add_first` (xmllinq/mod.rs:1085) L1964
- **Calls (std/external/unresolved):** `.and_then`, `.contains`, `.is_none`, `.is_some_and`, `.is_some`, `.starts_with`, `.to_ascii_lowercase`, `.to_string`, `.unwrap_or`, `Some`, `W::body`, `W::name`, `W::num_pr`, `W::p_pr_change`, `W::p_pr`, `W::p_style`, `W::p`, `W::val`
- **Called by:** `coverage_tests::promoted_list_numbering_uses_inserted_id_and_deleted_level` (comparer/finalize.rs:16629); `finalizer_distinct_boundary_contract_tests::historical_legal_numbering_uses_the_donor_id_and_keeps_authored_old_level` (comparer/finalize.rs:30321); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `body_looks_like_section_number`  (L1971-1987)
`fn body_looks_like_section_number(text: &str) -> bool`

- **Does:** True when body starts like a legal section id: one+ digits, a dot, one+ digits (e.g.
- **Calls (crate):** `Parser::peek` (markdown/math.rs:804) (by name) L1975,1986
- **Calls (std/external/unresolved):** `.chars`, `.is_ascii_digit`, `.next`, `.peekable`, `.trim_start`, `Some`
- **Macros:** `matches!`
- **Called by:** `body_looks_like_list_residual_label` (comparer/finalize.rs:1991); `coverage_tests::section_and_list_residual_label_boundaries` (comparer/finalize.rs:15699)

#### `body_looks_like_list_residual_label`  (L1991-2009)
`fn body_looks_like_list_residual_label(text: &str) -> bool`

- **Does:** list_spacer section ids **or** list_def residual labels ("Num 4") without live ListParagraph style.
- **Calls (crate):** `body_looks_like_section_number` (comparer/finalize.rs:1971) L1992
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.is_ascii_digit`, `.is_empty`, `.next`, `.split_whitespace`, `.strip_prefix`, `.to_ascii_lowercase`, `.trim`, `.unwrap`
- **Called by:** `promote_live_numpr_on_pure_d_from_pprchange` (comparer/finalize.rs:1866); `coverage_tests::section_and_list_residual_label_boundaries` (comparer/finalize.rs:15699)

#### `para_side_word_count`  (L2013-2031)
`fn para_side_word_count(dom: &Dom, p: NodeId, ins_side: bool) -> usize`

- **Does:** Word-count of `w:t` under pure-I runs (`ins_side`) or `w:delText` under pure-D runs (`!ins_side`) inside a paragraph (MIX-aware).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2016,2018,2024; `Dom::value_str` (xmllinq/mod.rs:1179) L2019,2025
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.is_empty`, `.push_str`, `.push`, `.split_whitespace`, `Some`, `String::new`, `W::del_text`, `W::del`, `W::ins`, `W::t`
- **Called by:** `last_pure_del_spacing_to_pprchange` (comparer/finalize.rs:5164); `promote_live_jc_from_pprchange_on_body_mix` (comparer/finalize.rs:6511); `park_jc_on_first_short_title_mix_from_body` (comparer/finalize.rs:13103)

#### `para_keeps_unchanged_words`  (L2034-2043)
`fn para_keeps_unchanged_words(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when a paragraph holds live text outside any `w:ins`/`w:del`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2035; `Dom::value_str` (xmllinq/mod.rs:1179) L2036; `Dom::ancestors` (xmllinq/mod.rs:770) L2037; `Dom::name_is` (xmllinq/mod.rs:535) L2041
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_empty`, `.take_while`, `.trim`, `Some`, `W::del`, `W::ins`, `W::t`
- **Called by:** `last_pure_del_spacing_to_pprchange` (comparer/finalize.rs:5164)

#### `ensure_pure_i_list_snug_spacing`  (L2071-2130, pub)
`pub fn ensure_pure_i_list_snug_spacing(dom: &mut Dom, root: NodeId)`

- **Does:** M439 (list_def_mix × list_numbering_reimport ~50.5 / docxodus 90): pure-I list items with live `numPr` and **no** spacing inherit bloated package `pPrDefault` (before=240 after=240 line=288) under LO.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2077; `para_is_pure_inserted` (comparer/finalize.rs:4175) L2078; `Dom::element` (xmllinq/mod.rs:655) L2081,2084,2087,2090,2111,2114...; `Dom::attribute` (xmllinq/mod.rs:874) L2091; `para_word_atom_count` (comparer/finalize.rs:8008) L2097; `para_revision_body_text` (comparer/finalize.rs:7854) L2100; `Dom::new_element` (xmllinq/mod.rs:478) L2117; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L2118,2119,2120,2121; `Dom::add_before_self` (xmllinq/mod.rs:1114) L2123,2125; `Dom::add` (xmllinq/mod.rs:1058) L2127
- **Calls (std/external/unresolved):** `.chars`, `.clone`, `.copied`, `.count`, `.entry`, `.eq_ignore_ascii_case`, `.get`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.or_insert`, `.push`, `.to_string`, `.trim`, `.unwrap_or`, `Some`, `Vec::new`, `W::name`, `W::num_pr`, `W::p_pr_change`, `W::p_pr`, `W::p_style`, `W::p`, `W::r_pr`, `W::spacing_el`, `W::val`, `std::collections::HashMap::new`
- **Called by:** `coverage_round_next_tests::list_snug_spacing_requires_three_identical_short_tokens_and_respects_schema_tail` (comparer/finalize.rs:19872); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `normalize_incomplete_spacing`  (L2176-2229, pub)
`pub fn normalize_incomplete_spacing(dom: &mut Dom, root: NodeId)`

- **Does:** Word-parity for incomplete body spacing (C3 / C5 layout).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2183; `Dom::element` (xmllinq/mod.rs:655) L2184,2187,2188; `Dom::attribute` (xmllinq/mod.rs:874) L2191,2192,2193,2194; `Dom::remove` (xmllinq/mod.rs:1102) L2214; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L2218,2219,2220,2221,2223,2227
- **Calls (std/external/unresolved):** `.is_empty`, `.is_some`, `.push`, `.unwrap_or`, `Some`, `Vec::new`, `W::name`, `W::num_pr`, `W::p_pr`, `W::p`, `W::spacing_el`
- **Called by:** `closing_mark_takes_revised_properties` (comparer/finalize.rs:3158); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `strip_redundant_demo_default_spacing`  (L2266-2347, pub)
`pub fn strip_redundant_demo_default_spacing( dom: &mut Dom, root: NodeId, default_lines: (Option<&str>, Option<&str>), )`

- **Does:** Word Compare omits body-level `w:spacing` that only restates the common demo-doc default (line=276, optional after=200 / lineRule=auto).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2273; `Dom::element` (xmllinq/mod.rs:655) L2274,2277,2310,2311,2322,2323...; `Dom::attribute` (xmllinq/mod.rs:874) L2280,2281,2282,2283,2327; `para_is_pure_deleted` (comparer/finalize.rs:2858) L2301,2339; `para_is_pure_inserted` (comparer/finalize.rs:4175) L2312; `para_word_atom_count` (comparer/finalize.rs:8008) L2316; `Dom::remove` (xmllinq/mod.rs:1102) L2345
- **Calls (std/external/unresolved):** `.all`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some`, `.iter`, `.parse`, `.push`, `.unwrap_or`, `Some`, `Vec::new`, `W::name`, `W::p_pr_change`, `W::p_pr`, `W::p_style`, `W::p`, `W::spacing_el`
- **Called by:** `closing_mark_takes_revised_properties` (comparer/finalize.rs:3158); `coverage_round_next_tests::default_spacing_strips_only_the_selected_source_default_and_exact_shape` (comparer/finalize.rs:20215); `coverage_round_next_tests::deleted_heading_spacing_threshold_keeps_missing_metrics_and_real_styles` (comparer/finalize.rs:20250); `demo_title_pilcrow_history_tests::authored_title_snapshot_survives_demo_default_layout_normalization` (comparer/finalize.rs:28801); `finalizer_distinct_boundary_contract_tests::demo_spacing_respects_authored_metrics_history_and_source_defaults` (comparer/finalize.rs:29577); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `strip_redundant_normal_pstyle_and_bidi`  (L2362-2406, pub)
`pub fn strip_redundant_normal_pstyle_and_bidi(dom: &mut Dom, root: NodeId)`

- **Does:** M367 (sdts×shape_group −4.3): Word omits body `w:pStyle=Normal` and `w:bidi=0` on pure-inserted paragraphs (shape_group writes both; Word redline is bare mark-only pPr).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2364,2393; `para_is_pure_inserted` (comparer/finalize.rs:4175) L2365,2394; `Dom::element` (xmllinq/mod.rs:655) L2368,2371,2377,2397; `Dom::attribute` (xmllinq/mod.rs:874) L2372,2378; `Dom::parent` (xmllinq/mod.rs:597) L2386; `Dom::remove` (xmllinq/mod.rs:1102) L2387,2404; `Dom::elements` (xmllinq/mod.rs:640) L2398
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`, `.is_empty`, `.is_some`, `.push`, `.unwrap_or`, `Some`, `Vec::new`, `W::name`, `W::p_pr`, `W::p_style`, `W::p`, `W::val`
- **Called by:** `finalizer_distinct_boundary_contract_tests::explicit_default_direction_is_removed_only_from_inserted_layout` (comparer/finalize.rs:29518); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `strip_list_layout_from_mid_pure_del`  (L2421-2502, pub)
`pub fn strip_list_layout_from_mid_pure_del(dom: &mut Dom, root: NodeId)`

- **Does:** M376 — strip list-shape spacing / `w:jc` / mark fonts that a mid pure-D residual absorbed from a preceding pure-I list stream.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L2422,2440,2456,2460,2463,2474...; `Dom::elements` (xmllinq/mod.rs:640) L2425,2486; `Dom::name_is` (xmllinq/mod.rs:535) L2428,2437,2453; `para_is_pure_inserted` (comparer/finalize.rs:4175) L2437; `spacing_is_list_single_line` (comparer/finalize.rs:2506) L2441,2470; `para_is_pure_deleted` (comparer/finalize.rs:2858) L2453; `Dom::attribute` (xmllinq/mod.rs:874) L2464,2479; `Dom::name` (xmllinq/mod.rs:526) L2487; `Dom::parent` (xmllinq/mod.rs:597) L2498; `Dom::remove` (xmllinq/mod.rs:1102) L2499
- **Calls (std/external/unresolved):** `.collect`, `.eq_ignore_ascii_case`, `.filter`, `.into_iter`, `.is_some_and`, `.is_some`, `.len`, `.push`, `.unwrap_or`, `Vec::new`, `W::body`, `W::del`, `W::ins`, `W::jc_el`, `W::num_pr`, `W::p_pr`, `W::p_style`, `W::p`, `W::r_pr`, `W::sect_pr`, `W::spacing_el`, `W::val`
- **Called by:** `coverage_round_next_tests::list_contamination_cleanup_keeps_real_lists_and_alignment` (comparer/finalize.rs:19844); `finalizer_distinct_boundary_contract_tests::list_layout_contamination_does_not_erase_real_list_or_terminal_ownership` (comparer/finalize.rs:30139); `mid_list_optional_source_property_tests::list_layout_cleanup_handles_absent_optional_properties_without_inventing_owners` (comparer/finalize.rs:34412); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `spacing_is_list_single_line`  (L2506-2518)
`fn spacing_is_list_single_line(dom: &Dom, ppr: NodeId) -> bool`

- **Does:** `after=0` (or absent) + `line=240` + `lineRule=auto` — pure-I list residual single-line shape (broken_complex_list / Word list compare).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L2507; `Dom::attribute` (xmllinq/mod.rs:874) L2510,2511,2512,2513
- **Calls (std/external/unresolved):** `.is_empty`, `.unwrap_or`, `W::name`, `W::spacing_el`
- **Called by:** `strip_list_layout_from_mid_pure_del` (comparer/finalize.rs:2421)

#### `free_mesh_shared_title_token_in_mix`  (L2531-2714, pub)
`pub fn free_mesh_shared_title_token_in_mix(dom: &mut Dom, root: NodeId)`

- **Does:** M377 — free-mesh one shared significant `len≥4` token out of a short-title MIX as `Equal` (re-emitting it on both sides instead of ins+deleting it).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L2532,2687,2688; `Dom::elements` (xmllinq/mod.rs:640) L2535,2550; `Dom::name_is` (xmllinq/mod.rs:535) L2538,2541,2553; `Dom::descendants` (xmllinq/mod.rs:668) L2544,2545,2598,2604; `Dom::name` (xmllinq/mod.rs:526) L2563; `holds_only_text` (comparer/finalize.rs:13578) L2576; `Dom::attribute` (xmllinq/mod.rs:874) L2588,2671,2674,2677; `Dom::value_str` (xmllinq/mod.rs:1179) L2599,2605; `alnum_tokens` (comparer/finalize.rs:2717) L2611,2612; `trailing_alnum_token` (comparer/finalize.rs:7583) L2650; `split_around_token` (comparer/finalize.rs:2728) L2658; `strip_trailing_alnum_token` (comparer/finalize.rs:7601) L2661; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L2689; `Dom::parent` (xmllinq/mod.rs:597) L2693; `Dom::remove` (xmllinq/mod.rs:1102) L2694; `rebuild_title_free_mesh` (comparer/finalize.rs:2771) L2700
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.as_str`, `.collect`, `.contains`, `.count`, `.eq_ignore_ascii_case`, `.filter`, `.first`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.last`, `.len`, `.map`, `.push_str`, `.push`, `.to_string`, `Some`, `String::new`, `Vec::new`, `W::author`, `W::body`, `W::date`, `W::del_text`, `W::del`, `W::id`, `W::ins`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `W::sect_pr`, `W::t`, `author_of`
- **Called by:** `deep_production_path_tests::shared_title_mesh_executes_wholesale_and_confetti_rebuilds_with_exact_views` (comparer/finalize.rs:22204); `finalizer_distinct_boundary_contract_tests::a_short_title_mesh_cannot_consume_foreign_revision_provenance_or_unrelated_titles` (comparer/finalize.rs:30634); `interleave_terminal_source_ownership_tests::split_inserted_title_fragments_do_not_take_another_authors_pending_revision` (comparer/finalize.rs:33227); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `alnum_tokens`  (L2717-2722)
`fn alnum_tokens(text: &str) -> Vec<String>`

- **Does:** Alnum tokens lowercased.
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.is_alphanumeric`, `.is_empty`, `.map`, `.split`, `.to_ascii_lowercase`
- **Called by:** `free_mesh_shared_title_token_in_mix` (comparer/finalize.rs:2531); `free_mesh_wholesale_body_mix` (comparer/finalize.rs:13611); `free_mesh_bookended_ins_del` (comparer/finalize.rs:14051)

#### `split_around_token`  (L2728-2759)
`fn split_around_token(text: &str, label: &str) -> Option<(String, String)>`

- **Does:** Split `text` into (before, after) around first case-insensitive alnum token matching `label`.
- **Calls (std/external/unresolved):** `.char_indices`, `.collect`, `.eq_ignore_ascii_case`, `.is_alphanumeric`, `.is_empty`, `.len`, `.to_string`, `.trim`, `Some`
- **Called by:** `free_mesh_shared_title_token_in_mix` (comparer/finalize.rs:2531); `coverage_tests::token_splitting_preserves_separators_and_original_case` (comparer/finalize.rs:15722)

#### `rebuild_title_free_mesh`  (L2771-2819)
`fn rebuild_title_free_mesh( dom: &mut Dom, p: NodeId, ins_before: &str, del_prefix: &str, eq_label: &str, ins_after: &str, stamp: RevStamp<'_>, )`

- **Does:** Rebuild short-title MIX free-mesh body on `p`.
- **Calls (crate):** `add_revision_text_run` (comparer/finalize.rs:2821) L2785,2789,2817; `Dom::new_element` (xmllinq/mod.rs:478) L2804,2809; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L2806; `Dom::add` (xmllinq/mod.rs:1058) L2807,2813,2814; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L2811; `XNamespace::xml` (xmllinq/mod.rs:128) L2811; `Dom::add_text` (xmllinq/mod.rs:1071) L2812
- **Calls (std/external/unresolved):** `.is_empty`, `.name`, `.trim_end`, `Some`, `W::del`, `W::ins`, `W::r`, `W::t`
- **Macros:** `format!`
- **Called by:** `free_mesh_shared_title_token_in_mix` (comparer/finalize.rs:2531)

#### `add_revision_text_run`  (L2821-2854)
`fn add_revision_text_run( dom: &mut Dom, p: NodeId, wrapper: crate::xmllinq::XName, text: &str, deleted: bool, stamp: RevStamp<'_>, next_id: &mut u32, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L2835,2841,2846; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L2836,2837,2840,2848; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L2843; `Dom::add` (xmllinq/mod.rs:1058) L2844,2851,2852,2853; `XNamespace::xml` (xmllinq/mod.rs:128) L2848; `Dom::add_text` (xmllinq/mod.rs:1071) L2850
- **Calls (std/external/unresolved):** `.ends_with`, `.name`, `.starts_with`, `.to_string`, `Some`, `W::author`, `W::date`, `W::del_text`, `W::id`, `W::r`, `W::t`
- **Called by:** `rebuild_title_free_mesh` (comparer/finalize.rs:2771)

#### `para_is_pure_deleted`  (L2858-2871)
`fn para_is_pure_deleted(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when a paragraph has deleted content, no live (non-del) `w:t` text, and no `w:ins` — pure deleted body paragraph.
- **Calls (crate):** `para_is_pure_deleted_uncached` (comparer/finalize.rs:2873) L2864
- **Calls (std/external/unresolved):** `.and_then`, `.as_mut`, `.as_ref`, `.borrow_mut`, `.borrow`, `.copied`, `.get`, `.insert`, `.with`
- **Called by:** `fold_short_list_label_into_empty_pure_del` (comparer/finalize.rs:1702); `strip_trailing_bare_empty_after_pure_i_dominant` (comparer/finalize.rs:1787); `promote_live_numpr_on_pure_d_from_pprchange` (comparer/finalize.rs:1866); `strip_redundant_demo_default_spacing` (comparer/finalize.rs:2266); `strip_list_layout_from_mid_pure_del` (comparer/finalize.rs:2421); `strip_trailing_empty_pure_del_mark` (comparer/finalize.rs:2909); `story_final_kept_blank` (comparer/finalize.rs:3501); `relocate_title_page_last_empty_after_pure_dels` (comparer/finalize.rs:3612); `ensure_empty_pure_i_before_short_title_del` (comparer/finalize.rs:3910); `strip_empty_pure_ins_before_trailing_pure_dels` (comparer/finalize.rs:4062); `fold_whitespace_pure_ins_into_following_pure_del` (comparer/finalize.rs:4211); `fold_leading_ins_from_mix_into_preceding_pure_del` (comparer/finalize.rs:4950); +17 more

#### `para_is_pure_deleted_uncached`  (L2873-2900)
`fn para_is_pure_deleted_uncached(dom: &Dom, p: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2874,2878,2881; `Dom::ancestors_and_self` (xmllinq/mod.rs:787) L2883; `Dom::name` (xmllinq/mod.rs:526) L2884; `Dom::value` (xmllinq/mod.rs:1169) L2893
- **Calls (std/external/unresolved):** `.as_ref`, `.is_empty`, `.trim`, `Some`, `W::del`, `W::ins`, `W::t`
- **Called by:** `para_is_pure_deleted` (comparer/finalize.rs:2858)

#### `strip_trailing_empty_pure_del_mark`  (L2909-2949, pub)
`pub fn strip_trailing_empty_pure_del_mark(dom: &mut Dom, root: NodeId)`

- **Does:** M69 — clear the paragraph-mark `w:del` on the final body paragraph when it is a content-free pure-D.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L2910,2936,2937,2938; `Dom::elements` (xmllinq/mod.rs:640) L2913,2941,2945; `Dom::name_is` (xmllinq/mod.rs:535) L2915,2918; `Dom::descendants` (xmllinq/mod.rs:668) L2922,2923,2932; `para_is_pure_deleted` (comparer/finalize.rs:2858) L2928; `para_mark_revision` (comparer/finalize.rs:7626) L2928; `Dom::remove` (xmllinq/mod.rs:1102) L2939,2942,2946
- **Calls (std/external/unresolved):** `.find`, `.is_empty`, `.iter`, `.rev`, `Some`, `W::body`, `W::del_text`, `W::del`, `W::ins`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`, `W::t`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `trailing_empty_spacing_to_pprchange`  (L2967-3034, pub)
`pub fn trailing_empty_spacing_to_pprchange( dom: &mut Dom, root: NodeId, revised_tail_spacing: Option<&[(String, String, String)]>, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** M92 — trailing empty body paragraph: Word records live `w:spacing` under `w:pPrChange` (file_30 last empty after list residual).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L2974,2993,2996,2999,3029; `Dom::elements` (xmllinq/mod.rs:640) L2977,3006,3030; `Dom::name_is` (xmllinq/mod.rs:535) L2978,2981; `para_has_no_text` (comparer/finalize.rs:3459) L2984; `Dom::descendants` (xmllinq/mod.rs:668) L2988,2989; `sorted_attributes` (comparer/finalize.rs:3436) L3002; `Dom::name` (xmllinq/mod.rs:526) L3007; `Dom::new_element` (xmllinq/mod.rs:478) L3017,3021; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L3018; `Dom::add` (xmllinq/mod.rs:1058) L3019,3026,3027; `Dom::remove` (xmllinq/mod.rs:1102) L3020,3032; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L3022,3024,3025
- **Calls (std/external/unresolved):** `.find`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.rev`, `.to_string`, `Some`, `W::author`, `W::body`, `W::date`, `W::del`, `W::id`, `W::ins`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`, `W::spacing_el`
- **Called by:** `coverage_round_next_tests::trailing_blank_spacing_records_history_only_when_source_differs` (comparer/finalize.rs:19901); `finalizer_distinct_boundary_contract_tests::the_final_empty_spacing_history_has_one_source_owner` (comparer/finalize.rs:29711); `public_body_heuristic_story_scope_tests::public_body_layout_heuristics_preserve_complete_independent_header_and_footer_parts` (comparer/finalize.rs:32251); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `closing_paragraph_properties`  (L3051-3064, pub)
`pub fn closing_paragraph_properties(dom: &mut Dom, body: NodeId) -> Option<NodeId>`

- **Does:** The paragraph properties of the story's closing paragraph, cloned without scratch attributes before atomize consumes `body`: `None` when the story does not end on a paragraph, an empty `w:pPr` when the paragraph has none.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L3052; `Dom::name_is` (xmllinq/mod.rs:535) L3056,3057; `Dom::element` (xmllinq/mod.rs:655) L3060; `drop_pt_bookkeeping` (comparer/finalize.rs:481) L3061; `Dom::new_element` (xmllinq/mod.rs:478) L3062
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.rev`, `Some`, `W::p_pr`, `W::p`, `W::sect_pr`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `merged_closing_carrier_takes_source_properties`  (L3080-3141, pub)
`pub fn merged_closing_carrier_takes_source_properties( dom: &mut Dom, root: NodeId, original: Option<NodeId>, revised: Option<NodeId>, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** Faithful production can conjoin unrelated terminal deletion/insertion streams into the original paragraph.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L3094,3108,3119,3121; `Dom::elements` (xmllinq/mod.rs:640) L3097,3127,3132; `Dom::name_is` (xmllinq/mod.rs:535) L3101,3105,3128; `para_mark_revision` (comparer/finalize.rs:7626) L3105; `content_revision_marks` (comparer/finalize.rs:6184) L3111; `project_para_properties_for_change` (comparer/formatchg.rs:329) L3114,3115; `normalize_para_properties` (comparer/formatchg.rs:349) L3116,3117,3118,3123; `Dom::remove` (xmllinq/mod.rs:1102) L3129,3133; `Dom::add` (xmllinq/mod.rs:1058) L3134,3138,3139; `rev_el` (comparer/finalize.rs:126) L3137
- **Calls (std/external/unresolved):** `.and_then`, `.find`, `.into_iter`, `.is_some_and`, `.rev`, `W::body`, `W::del`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`
- **Called by:** `finalizer_distinct_boundary_contract_tests::faithful_merged_closing_carrier_records_both_complete_property_sources` (comparer/finalize.rs:30192); `public_body_heuristic_story_scope_tests::public_body_layout_heuristics_preserve_complete_independent_header_and_footer_parts` (comparer/finalize.rs:32251); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `closing_mark_takes_revised_properties`  (L3158-3274, pub)
`pub fn closing_mark_takes_revised_properties( dom: &mut Dom, root: NodeId, original: Option<NodeId>, revised: Option<NodeId>, default_lines: (Option<&str>, Option<&str>), settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** The story's closing mark is never revised: accepted, it closes the revision; rejected, the original.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L3172,3173,3197,3264; `Dom::add` (xmllinq/mod.rs:1058) L3174,3176,3258,3261,3269,3270; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L3175; `normalize_universal_measures` (comparer/finalize.rs:11485) L3177,3273; `strip_redundant_demo_default_spacing` (comparer/finalize.rs:2266) L3178; `normalize_incomplete_spacing` (comparer/finalize.rs:2176) L3179; `Dom::element` (xmllinq/mod.rs:655) L3180,3194,3202,3206,3224,3226...; `Dom::elements` (xmllinq/mod.rs:640) L3183,3213,3214,3252,3256; `Dom::name_is` (xmllinq/mod.rs:535) L3187,3191; `Dom::add_first` (xmllinq/mod.rs:1085) L3198; `closing_mark_takes_revised_mark_formatting` (comparer/finalize.rs:3358) L3219; `normalize_para_properties` (comparer/formatchg.rs:349) L3220,3221,3223,3228; `project_para_properties_for_change` (comparer/formatchg.rs:329) L3222,3255; `sectpr_identity` (comparer/finalize.rs:11604) L3239; `Dom::remove` (xmllinq/mod.rs:1102) L3253,3257; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L3265,3267,3268
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.collect`, `.filter_map`, `.find`, `.into_iter`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.rev`, `.to_string`, `Some`, `W::author`, `W::body`, `W::date`, `W::id`, `W::ins`, `W::name`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`
- **Called by:** `closing_layout_history_regressions::already_revised_live_closing_layout_still_records_original_properties` (comparer/finalize.rs:23657); `closing_authored_section_boundary_tests::section_addition_guard_respects_tracking_source_presence_and_exact_live_identity` (comparer/finalize.rs:29200); `closing_authored_section_boundary_tests::matching_closing_history_keeps_its_existing_revision_owner_and_id` (comparer/finalize.rs:29315); `public_body_heuristic_story_scope_tests::public_body_layout_heuristics_preserve_complete_independent_header_and_footer_parts` (comparer/finalize.rs:32251); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `paragraph_mark_properties_equal`  (L3280-3352)
`fn paragraph_mark_properties_equal( dom: &mut Dom, old: Option<NodeId>, new: Option<NodeId>, ) -> bool`

- **Does:** Keep the established leaf-property normalization, but paragraph marks also own nested Office effects. _(from a code comment)_
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L3288; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L3291; `Dom::serialize_element` (xmllinq/mod.rs:1299) L3291; `Dom::root` (xmllinq/mod.rs:822) L3292; `are_run_properties_equal` (comparer/formatchg.rs:118) L3297; `Dom::elements` (xmllinq/mod.rs:640) L3338; `Dom::name_is` (xmllinq/mod.rs:535) L3340; `Dom::name` (xmllinq/mod.rs:526) L3341; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L3343; `Dom::has_elements` (xmllinq/mod.rs:862) L3344; `paragraph_mark_properties_equal::{fn signature}` (comparer/finalize.rs:3300) L3346; `text_lines::{fn nested}` (debug.rs:1566) (by name) L3351
- **Calls (std/external/unresolved):** `.collect`, `.expect`, `.filter`, `.flat_map`, `.into_iter`, `.is_some_and`, `.map`, `.sort`, `W::r_pr_change`
- **Called by:** `closing_mark_takes_revised_mark_formatting` (comparer/finalize.rs:3358); `mesh_short_label_shared_eq_with_settings` (comparer/finalize.rs:7199); `fold_midstream_demo_title_into_numbered_heading_with_source_properties` (comparer/finalize.rs:12619); `finalizer_merge_source_guard_tests::pilcrow_format_equality_includes_nested_fill_payload_but_ignores_source_scratch` (comparer/finalize.rs:31772)

#### `paragraph_mark_properties_equal::{fn signature}`  (L3300-3334)
`fn signature(dom: &Dom, node: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::attributes` (xmllinq/mod.rs:885) L3301; `Dom::is_namespace_declaration` (xmllinq/mod.rs:996) L3305; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L3306,3311,3323; `XName::local_name` (xmllinq/mod.rs:208) (by name) L3307,3312; `Dom::nodes` (xmllinq/mod.rs:604) L3318; `Dom::name` (xmllinq/mod.rs:526) L3322; `Dom::is_element` (xmllinq/mod.rs:502) L3324; `Dom::text_value` (xmllinq/mod.rs:547) L3325; `paragraph_mark_properties_equal::{fn signature}` (comparer/finalize.rs:3300) L3327; `Dom::text_value` (xmllinq/mod.rs:547) (by name) L3332
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.is_some_and`, `.map`, `.name`, `.sort`, `.starts_with`, `.to_ascii_lowercase`, `.to_string`, `.trim`, `.unwrap_or_default`
- **Macros:** `format!`
- **Called by:** `paragraph_mark_properties_equal` (comparer/finalize.rs:3280); `paragraph_mark_properties_equal::{fn signature}` (comparer/finalize.rs:3300)

#### `closing_mark_takes_revised_mark_formatting`  (L3358-3409)
`fn closing_mark_takes_revised_mark_formatting( dom: &mut Dom, ppr: NodeId, revised: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** The closing mark's own formatting follows the same way: Word writes the revised closing paragraph's mark `rPr` live and records the original's in a `rPrChange` (b4cd671041: `sz=28` live over the original's `b rFonts=Arial sz=24 u`).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L3368,3369,3372,3403; `paragraph_mark_properties_equal` (comparer/finalize.rs:3280) L3373; `Dom::new_element` (xmllinq/mod.rs:478) L3376,3390,3392; `Dom::elements` (xmllinq/mod.rs:640) L3378; `Dom::name_is` (xmllinq/mod.rs:535) L3381; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L3383,3389; `Dom::add` (xmllinq/mod.rs:1058) L3384,3397,3398,3406; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L3393,3395,3396; `Dom::replace_with` (xmllinq/mod.rs:1149) L3400; `Dom::add_before_self` (xmllinq/mod.rs:1114) L3405
- **Calls (std/external/unresolved):** `.any`, `.find_map`, `.is_some_and`, `.is_some`, `.iter`, `.to_string`, `Some`, `W::author`, `W::date`, `W::id`, `W::name`, `W::p_pr_change`, `W::r_pr`, `W::sect_pr`
- **Called by:** `closing_mark_takes_revised_properties` (comparer/finalize.rs:3158); `mesh_short_label_shared_eq_with_settings` (comparer/finalize.rs:7199); `fold_midstream_demo_title_into_numbered_heading_with_source_properties` (comparer/finalize.rs:12619); `demo_title_pilcrow_history_tests::closing_mark_history_preserves_nested_effects_and_ignores_only_existing_normalization_noise` (comparer/finalize.rs:28715)

#### `last_body_para_spacing`  (L3422-3433, pub)
`pub fn last_body_para_spacing(dom: &Dom, body: NodeId) -> Option<Vec<(String, String, String)>>`

- **Does:** Sorted `(namespace, local name, value)` attributes of the last body paragraph's `w:spacing` under `body`, if it has one.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L3423; `Dom::name_is` (xmllinq/mod.rs:535) L3427,3428; `Dom::element` (xmllinq/mod.rs:655) L3431; `sorted_attributes` (comparer/finalize.rs:3436) L3432
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.rev`, `Some`, `W::p_pr`, `W::p`, `W::sect_pr`, `W::spacing_el`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `sorted_attributes`  (L3436-3452)
`fn sorted_attributes(dom: &Dom, n: NodeId) -> Vec<(String, String, String)>`

- **Does:** Sorted `w:` attributes (scratch ids such as `pt:Unid` excluded).
- **Calls (crate):** `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L3437,3441,3444; `Dom::attributes` (xmllinq/mod.rs:885) L3438; `XName::local_name` (xmllinq/mod.rs:208) (by name) L3445
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.map`, `.sort`, `.to_string`, `W::p`
- **Called by:** `trailing_empty_spacing_to_pprchange` (comparer/finalize.rs:2967); `last_body_para_spacing` (comparer/finalize.rs:3422); `coverage_round_next_tests::trailing_blank_spacing_records_history_only_when_source_differs` (comparer/finalize.rs:19901)

#### `para_has_no_text`  (L3459-3468)
`fn para_has_no_text(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when a paragraph has no `w:t` / `w:delText` content.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L3460,3461,3462,3463,3464,3465...
- **Calls (std/external/unresolved):** `.is_empty`, `M::name`, `Some`, `W::del_text`, `W::drawing`, `W::name`, `W::pict`, `W::t`
- **Called by:** `fold_short_list_label_into_empty_pure_del` (comparer/finalize.rs:1702); `strip_trailing_bare_empty_after_pure_i_dominant` (comparer/finalize.rs:1787); `trailing_empty_spacing_to_pprchange` (comparer/finalize.rs:2967); `para_is_empty_pure_ins` (comparer/finalize.rs:3542); `is_restorable_layout_blank` (comparer/finalize.rs:3818); `revised_two_blank_table_titles` (comparer/finalize.rs:3848); `ensure_empty_pure_i_before_short_title_del` (comparer/finalize.rs:3910); `strip_last_pure_del_mark_only_ppr` (comparer/finalize.rs:5081); `mixed_spacing_to_following_empty` (comparer/finalize.rs:5267); `park_mixed_spacing_onto_trailing_pure_del` (comparer/finalize.rs:5602); `fold_short_label_ins_into_del_with_settings` (comparer/finalize.rs:7053); `merge_replaced_in_container` (comparer/finalize.rs:8621); +3 more

#### `para_has_omath`  (L3474-3477)
`fn para_has_omath(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when a paragraph carries OMML math (`m:oMath` / `m:oMathPara`).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L3475,3476
- **Calls (std/external/unresolved):** `.is_empty`, `M::name`, `Some`
- **Called by:** `para_is_visually_blank` (comparer/finalize.rs:3487); `relocate_title_page_last_empty_after_pure_dels` (comparer/finalize.rs:3612); `merge_replaced_in_container` (comparer/finalize.rs:8621)

#### `para_is_visually_blank`  (L3487-3494)
`fn para_is_visually_blank(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when a paragraph renders as a blank line: whitespace-only text and no non-text ink (math, drawings, pictures, objects, breaks).
- **Calls (crate):** `para_body_text_is_whitespace_only` (comparer/finalize.rs:4159) L3488; `para_has_omath` (comparer/finalize.rs:3474) L3489; `Dom::descendants` (xmllinq/mod.rs:668) L3490,3491,3492,3493
- **Calls (std/external/unresolved):** `.is_empty`, `Some`, `W::drawing`, `W::name`, `W::pict`
- **Called by:** `story_final_kept_blank` (comparer/finalize.rs:3501); `fold_whitespace_pure_ins_into_following_pure_del` (comparer/finalize.rs:4211); `merge_replaced_in_container` (comparer/finalize.rs:8621)

#### `story_final_kept_blank`  (L3501-3518)
`fn story_final_kept_blank(dom: &Dom, kids: &[NodeId]) -> usize`

- **Does:** 1 when the body ends on a live, blank paragraph after a pure-D run: the story-final mark the unrelated-sources pairing kept (A's and B's final empty paragraphs).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L3506,3507; `para_mark_revision` (comparer/finalize.rs:7626) L3508,3509; `Dom::name_is` (xmllinq/mod.rs:535) L3512,3515; `para_is_visually_blank` (comparer/finalize.rs:3487) L3513; `deep_production_path_tests::live` (comparer/finalize.rs:20901) L3514; `para_is_pure_deleted` (comparer/finalize.rs:2858) L3516
- **Calls (std/external/unresolved):** `.is_empty`, `Some`, `W::del`, `W::ins`, `W::p`, `usize::from`
- **Called by:** `strip_empty_pure_ins_before_trailing_pure_dels` (comparer/finalize.rs:4062); `fold_whitespace_pure_ins_into_following_pure_del` (comparer/finalize.rs:4211)

#### `para_has_repeated_phrase`  (L3524-3539)
`fn para_has_repeated_phrase(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when body text repeats a ≥4-word phrase (hummingbird wrap fingerprint).
- **Calls (crate):** `para_revision_body_text` (comparer/finalize.rs:7854) L3525
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.insert`, `.is_empty`, `.join`, `.len`, `.saturating_sub`, `.split_whitespace`, `.to_ascii_lowercase`, `std::collections::HashSet::new`
- **Called by:** `should_fold_multi_del_at_document_scale` (comparer/finalize.rs:8020); `merge_replaced_in_container` (comparer/finalize.rs:8621)

#### `para_is_empty_pure_ins`  (L3542-3566)
`fn para_is_empty_pure_ins(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when `p` is an empty pure-inserted paragraph (no text, has ins, no del).
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L3543; `para_has_no_text` (comparer/finalize.rs:3459) L3546; `Dom::descendants` (xmllinq/mod.rs:668) L3550,3555,3560,3561; `para_mark_revision` (comparer/finalize.rs:7626) L3550,3555
- **Calls (std/external/unresolved):** `.is_empty`, `Some`, `W::del`, `W::drawing`, `W::ins`, `W::p`, `W::tbl`
- **Called by:** `strip_trailing_empty_pure_ins` (comparer/finalize.rs:3579); `strip_empty_pure_ins_before_trailing_pure_dels` (comparer/finalize.rs:4062)

#### `strip_trailing_empty_pure_ins`  (L3579-3590, pub)
`pub fn strip_trailing_empty_pure_ins(dom: &mut Dom, root: NodeId)`

- **Does:** M83a — drop trailing empty pure-inserted body paragraph (file_23).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L3580; `Dom::elements` (xmllinq/mod.rs:640) L3583; `Dom::name_is` (xmllinq/mod.rs:535) L3584; `para_is_empty_pure_ins` (comparer/finalize.rs:3542) L3587; `Dom::remove` (xmllinq/mod.rs:1102) L3588
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.rev`, `W::body`, `W::sect_pr`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `relocate_title_page_last_empty_after_pure_dels`  (L3612-3732, pub)
`pub fn relocate_title_page_last_empty_after_pure_dels(dom: &mut Dom, root: NodeId)`

- **Does:** M438 (doc_with_spaces × doc_with_spacing ~67 / docxodus 100): Word title-page pure-I next × short pure-D base is `I…date e×6 DD E` (trailing bare empty).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L3613,3646; `Dom::elements` (xmllinq/mod.rs:640) L3616,3719; `Dom::name_is` (xmllinq/mod.rs:535) L3619,3626,3634,3645,3656,3678...; `para_is_pure_deleted` (comparer/finalize.rs:2858) L3627,3634; `para_body_text_is_whitespace_only` (comparer/finalize.rs:4159) L3628,3658,3680,3708; `para_is_pure_inserted` (comparer/finalize.rs:4175) L3646,3657,3679,3707; `para_has_omath` (comparer/finalize.rs:3474) L3659; `Dom::descendants` (xmllinq/mod.rs:668) L3660,3661; `para_revision_body_text` (comparer/finalize.rs:7854) L3685; `strip_para_revision_marks` (comparer/finalize.rs:3735) L3713; `Dom::remove` (xmllinq/mod.rs:1102) L3723; `Dom::parent` (xmllinq/mod.rs:597) L3725; `Dom::add_before_self` (xmllinq/mod.rs:1114) L3726; `Dom::add` (xmllinq/mod.rs:1058) L3729
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.collect`, `.contains`, `.count`, `.filter`, `.find`, `.into_iter`, `.is_ascii_digit`, `.is_empty`, `.is_some`, `.iter`, `.len`, `.position`, `.rev`, `.saturating_sub`, `.to_ascii_lowercase`, `Some`, `W::body`, `W::drawing`, `W::name`, `W::p_pr`, `W::p`, `W::sect_pr`
- **Called by:** `coverage_round_two_tests::title_page_relocation_preserves_six_spacers_and_moves_only_last` (comparer/finalize.rs:17092); `finalizer_distinct_boundary_contract_tests::title_page_empty_relocation_is_scoped_to_an_authored_cover_tail` (comparer/finalize.rs:30585); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `strip_para_revision_marks`  (L3735-3780)
`fn strip_para_revision_marks(dom: &mut Dom, p: NodeId)`

- **Does:** Drop ins/del wrappers and para-mark revisions from a paragraph (bare EQ/empty).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L3737,3756,3768,3776; `Dom::descendants` (xmllinq/mod.rs:668) L3744,3746; `Dom::parent` (xmllinq/mod.rs:597) L3752; `Dom::remove` (xmllinq/mod.rs:1102) L3758,3762,3774,3777; `Dom::add_before_self` (xmllinq/mod.rs:1114) L3759; `Dom::element` (xmllinq/mod.rs:655) L3765,3766
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.dedup`, `.extend`, `.filter`, `.into_iter`, `.is_empty`, `.is_none`, `.name`, `.sort_by_key`, `Some`, `W::del`, `W::ins`, `W::p_pr`, `W::r_pr`, `std::cmp::Reverse`
- **Macros:** `matches!`
- **Called by:** `relocate_title_page_last_empty_after_pure_dels` (comparer/finalize.rs:3612); `fold_midstream_demo_title_into_numbered_heading_with_source_properties::{fn clean_mark_properties}` (comparer/finalize.rs:12735)

#### `make_empty_pure_ins_para`  (L3783-3813)
`fn make_empty_pure_ins_para( dom: &mut Dom, template: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, ) -> NodeId`

- **Does:** Restore a revised layout blank with its own properties and a new insert mark.
- **Calls (crate):** `Dom::clone_subtree` (xmllinq/mod.rs:1215) L3789; `remove_powertools_scratch_markup` (comparer/finalize.rs:1018) L3790; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L3794,3807,3809,3810; `Dom::element` (xmllinq/mod.rs:655) L3796,3801; `Dom::new_element` (xmllinq/mod.rs:478) L3797,3802,3806; `Dom::add_first` (xmllinq/mod.rs:1085) L3798,3811; `Dom::add` (xmllinq/mod.rs:1058) L3803
- **Calls (std/external/unresolved):** `.to_string`, `.unwrap_or_else`, `Some`, `W14::name`, `W::author`, `W::date`, `W::id`, `W::ins`, `W::p_pr`, `W::r_pr`
- **Called by:** `ensure_empty_pure_i_before_short_title_del` (comparer/finalize.rs:3910)

#### `is_restorable_layout_blank`  (L3818-3835)
`fn is_restorable_layout_blank(dom: &Dom, p: NodeId) -> bool`

- **Does:** Only layout properties and property-only empty runs are safe to restore. _(from a code comment)_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L3819,3823,3826,3829; `para_has_no_text` (comparer/finalize.rs:3459) L3819; `Dom::elements` (xmllinq/mod.rs:640) L3822,3827; `Dom::descendants` (xmllinq/mod.rs:668) L3824; `element_has_tracked_revisions` (revision_processor.rs:90) L3825,3830
- **Calls (std/external/unresolved):** `.all`, `.into_iter`, `.is_empty`, `Some`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `W::sect_pr`
- **Called by:** `revised_two_blank_table_titles` (comparer/finalize.rs:3848)

#### `revised_two_blank_table_titles`  (L3848-3889, pub)
`pub fn revised_two_blank_table_titles(dom: &mut Dom, body: NodeId) -> HashMap<String, [NodeId; 2]>`

- **Does:** Titles that uniquely identify the revised source's two-blank pretable layout.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L3849; `Dom::name_is` (xmllinq/mod.rs:535) L3858,3864,3873,3878; `para_has_no_text` (comparer/finalize.rs:3459) L3858,3864; `revised_two_blank_table_titles::{fn text}` (comparer/finalize.rs:3850) L3859,3867; `is_restorable_layout_blank` (comparer/finalize.rs:3818) L3873; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L3882,3883
- **Calls (std/external/unresolved):** `.entry`, `.enumerate`, `.get`, `.insert`, `.is_empty`, `.is_some_and`, `.iter`, `.or_default`, `.trim`, `HashMap::::new`, `HashMap::new`, `Some`, `W::p`, `W::tbl`
- **Called by:** `deep_production_path_tests::spacer_context` (comparer/finalize.rs:21687); `finalizer_remaining_source_boundary_tests::two_authored_layout_blanks_are_restored_only_at_the_shared_table_boundary` (comparer/finalize.rs:31268); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `revised_two_blank_table_titles::{fn text}`  (L3850-3855)
`fn text(dom: &Dom, p: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L3851; `Dom::value` (xmllinq/mod.rs:1169) L3853
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`, `Some`, `W::t`
- **Called by:** `revised_two_blank_table_titles` (comparer/finalize.rs:3848); `coverage_tests::deleted_text_conversion_stops_at_nested_revision_owners` (comparer/finalize.rs:15801); `coverage_tests::content_controls_unwrap_deepest_first_and_keep_text_order` (comparer/finalize.rs:15875); `coverage_tests::residual_control_unwrap_keeps_equal_form_controls` (comparer/finalize.rs:15890); `coverage_tests::alternate_content_selects_first_bound_choice_or_fallback` (comparer/finalize.rs:15905); `coverage_tests::cleanup_spacing_removes_only_redundant_mixed_history` (comparer/finalize.rs:16232); `coverage_tests::cleanup_default_alignment_keeps_real_layout_history` (comparer/finalize.rs:16272); `coverage_tests::cleanup_deleted_spacing_promotes_mid_body_but_preserves_last_history` (comparer/finalize.rs:16306); `coverage_tests::short_label_free_mesh_preserves_deleted_prefix_and_inserted_whitespace` (comparer/finalize.rs:16368); `coverage_tests::residual_label_zip_adopts_deleted_list_layout_and_moves_empty_spacer` (comparer/finalize.rs:16416); `coverage_tests::short_list_label_folds_only_into_empty_deleted_mark_with_deleted_tail` (comparer/finalize.rs:16446); `coverage_tests::bookended_free_mesh_rebuilds_word_order_and_preserves_run_properties` (comparer/finalize.rs:16496); +6 more

#### `ensure_empty_pure_i_before_short_title_del`  (L3910-4040, pub)
`pub fn ensure_empty_pure_i_before_short_title_del( dom: &mut Dom, root: NodeId, revised_titles: &HashMap<String, [NodeId; 2]>, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** M392 (file_36×file_37 residual ~66): Word keeps empty pure-I spacers between content pure-I short title and pure-D short title before a table.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L3917,3996,4008,4016; `Dom::elements` (xmllinq/mod.rs:640) L3921,4000; `Dom::name_is` (xmllinq/mod.rs:535) L3924,3930,3990; `para_is_pure_inserted` (comparer/finalize.rs:4175) L3933,3993; `para_is_pure_deleted` (comparer/finalize.rs:2858) L3933,3978,3992; `para_body_text_is_whitespace_only` (comparer/finalize.rs:4159) L3937; `Dom::descendants` (xmllinq/mod.rs:668) L3940; `Dom::value` (xmllinq/mod.rs:1169) L3943; `para_word_atom_count` (comparer/finalize.rs:8008) L3949; `para_body_alnum_len` (comparer/finalize.rs:7884) L3950; `para_looks_like_demo_title` (comparer/finalize.rs:7901) L3951; `para_has_heading_or_title_style` (comparer/finalize.rs:7918) L3952; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L3969,3974; `para_has_no_text` (comparer/finalize.rs:3459) L3970,3991; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L3973,3976; `make_empty_pure_ins_para` (comparer/finalize.rs:3783) L3985; `Dom::add_before_self` (xmllinq/mod.rs:1114) L3986,4001; `Dom::new_element` (xmllinq/mod.rs:478) L3999,4011,4017; `Dom::add` (xmllinq/mod.rs:1058) L4003; `Dom::add_first` (xmllinq/mod.rs:1085) L4012,4030; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L4018,4020,4025
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.contains`, `.copied`, `.filter`, `.first`, `.get`, `.into_iter`, `.is_empty`, `.is_none`, `.len`, `.map`, `.saturating_sub`, `.to_string`, `Some`, `W::author`, `W::body`, `W::date`, `W::del`, `W::id`, `W::ins`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`, `W::t`, `W::tbl`
- **Macros:** `matches!`
- **Called by:** `deep_production_path_tests::table_title_spacers_stamp_only_new_marks_and_keep_existing_empty_properties` (comparer/finalize.rs:21697); `deep_production_path_tests::title_spacers_require_unique_revised_two_blank_pretable_context` (comparer/finalize.rs:21795); `deep_production_path_tests::restored_title_spacers_preserve_each_revised_blank_template` (comparer/finalize.rs:21887); `deep_production_path_tests::title_spacer_context_declines_owned_nonlayout_payload` (comparer/finalize.rs:21931); `deep_production_path_tests::title_spacer_injection_requires_a_paired_table_and_a_truly_blank_separator` (comparer/finalize.rs:21977); `finalizer_remaining_source_boundary_tests::two_authored_layout_blanks_are_restored_only_at_the_shared_table_boundary` (comparer/finalize.rs:31268); `public_body_heuristic_story_scope_tests::public_body_layout_heuristics_preserve_complete_independent_header_and_footer_parts` (comparer/finalize.rs:32251); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `strip_empty_pure_ins_before_trailing_pure_dels`  (L4062-4156, pub)
`pub fn strip_empty_pure_ins_before_trailing_pure_dels(dom: &mut Dom, root: NodeId)`

- **Does:** M85a — drop empty pure-ins sitting immediately before a trailing pure-del run.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L4063; `Dom::elements` (xmllinq/mod.rs:640) L4066; `Dom::name_is` (xmllinq/mod.rs:535) L4069,4078,4126,4132,4136; `para_is_pure_deleted` (comparer/finalize.rs:2858) L4078,4132; `para_is_empty_pure_ins` (comparer/finalize.rs:3542) L4090,4152; `story_final_kept_blank` (comparer/finalize.rs:3501) L4098; `para_word_atom_count` (comparer/finalize.rs:8008) L4115; `para_body_alnum_len` (comparer/finalize.rs:7884) L4116; `para_looks_like_demo_title` (comparer/finalize.rs:7901) L4119; `para_has_heading_or_title_style` (comparer/finalize.rs:7918) L4120; `Dom::remove` (xmllinq/mod.rs:1102) L4153
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.contains`, `.count`, `.filter`, `.flatten`, `.get`, `.into_iter`, `.is_some_and`, `.iter`, `.len`, `.saturating_sub`, `.take_while`, `W::body`, `W::p`, `W::sect_pr`, `W::tbl`
- **Called by:** `coverage_round_next_tests::title_spacers_are_kept_once_and_prose_spacers_are_removed` (comparer/finalize.rs:19815); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `para_body_text_is_whitespace_only`  (L4159-4172)
`fn para_body_text_is_whitespace_only(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when every `w:t` / `w:delText` under `p` is empty or whitespace.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L4162; `Dom::value` (xmllinq/mod.rs:1169) L4164
- **Calls (std/external/unresolved):** `.is_empty`, `.trim`, `Some`, `W::del_text`, `W::t`
- **Called by:** `para_is_visually_blank` (comparer/finalize.rs:3487); `relocate_title_page_last_empty_after_pure_dels` (comparer/finalize.rs:3612); `ensure_empty_pure_i_before_short_title_del` (comparer/finalize.rs:3910); `fold_whitespace_pure_ins_into_following_pure_del` (comparer/finalize.rs:4211); `fold_short_label_ins_into_del_with_settings` (comparer/finalize.rs:7053); `merge_replaced_in_container` (comparer/finalize.rs:8621); `coverage_tests::residual_label_zip_adopts_deleted_list_layout_and_moves_empty_spacer` (comparer/finalize.rs:16416)

#### `para_is_pure_inserted`  (L4175-4186)
`fn para_is_pure_inserted(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when `p` is pure-inserted (has ins, no del content/mark).
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L4176; `Dom::descendants` (xmllinq/mod.rs:668) L4180,4185; `para_mark_revision` (comparer/finalize.rs:7626) L4180,4185
- **Calls (std/external/unresolved):** `.is_empty`, `Some`, `W::del`, `W::ins`, `W::p`
- **Called by:** `fold_short_list_label_into_empty_pure_del` (comparer/finalize.rs:1702); `strip_trailing_bare_empty_after_pure_i_dominant` (comparer/finalize.rs:1787); `promote_live_numpr_on_pure_d_from_pprchange` (comparer/finalize.rs:1866); `ensure_pure_i_list_snug_spacing` (comparer/finalize.rs:2071); `strip_redundant_demo_default_spacing` (comparer/finalize.rs:2266); `strip_redundant_normal_pstyle_and_bidi` (comparer/finalize.rs:2362); `strip_list_layout_from_mid_pure_del` (comparer/finalize.rs:2421); `relocate_title_page_last_empty_after_pure_dels` (comparer/finalize.rs:3612); `ensure_empty_pure_i_before_short_title_del` (comparer/finalize.rs:3910); `fold_whitespace_pure_ins_into_following_pure_del` (comparer/finalize.rs:4211); `residual_short_label_zip_with_settings` (comparer/finalize.rs:6925); `fold_short_label_ins_into_del_with_settings` (comparer/finalize.rs:7053); +11 more

#### `fold_whitespace_pure_ins_into_following_pure_del`  (L4211-4553, pub)
`pub fn fold_whitespace_pure_ins_into_following_pure_del(dom: &mut Dom, root: NodeId)`

- **Does:** M86 — fold whitespace-only pure-ins into the following pure-del (file_88).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L4212,4409,4412,4447,4448,4457...; `Dom::elements` (xmllinq/mod.rs:640) L4220,4252,4289,4462,4468,4495...; `Dom::name_is` (xmllinq/mod.rs:535) L4223,4228,4236,4255,4259,4269...; `para_is_pure_inserted` (comparer/finalize.rs:4175) L4229,4237,4270,4280,4307,4433; `para_body_text_is_whitespace_only` (comparer/finalize.rs:4159) L4230,4238,4261,4310,4350,4434; `para_is_pure_deleted` (comparer/finalize.rs:2858) L4260,4307,4372,4418; `para_is_visually_blank` (comparer/finalize.rs:3487) L4271; `story_final_kept_blank` (comparer/finalize.rs:3501) L4281; `Dom::descendants` (xmllinq/mod.rs:668) L4317,4318,4319,4320,4332,4340...; `para_word_atom_count` (comparer/finalize.rs:8008) L4364,4401; `para_body_alnum_len` (comparer/finalize.rs:7884) L4365; `para_looks_like_demo_title` (comparer/finalize.rs:7901) L4366; `para_has_heading_or_title_style` (comparer/finalize.rs:7918) L4367; `ppr_has_structural_props` (comparer/finalize.rs:531) L4411,4414; `para_is_field_residue` (comparer/finalize.rs:7664) L4446,4515; `Dom::attribute` (xmllinq/mod.rs:874) L4449,4517; `Dom::remove` (xmllinq/mod.rs:1102) L4458,4473,4484,4496,4545; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L4461,4492,4503,4525,4529; `Dom::add_before_self` (xmllinq/mod.rs:1114) L4463,4533; `Dom::add` (xmllinq/mod.rs:1058) L4465,4470,4493,4504,4526,4535...; `Dom::new_element` (xmllinq/mod.rs:478) L4524
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.contains`, `.copied`, `.count`, `.filter`, `.first`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.position`, `.rev`, `.saturating_sub`, `.skip`, `.starts_with`, `.to_ascii_lowercase`, `.unwrap_or`, `Some`, `W::body`, `W::del`, `W::drawing`, `W::ins`, `W::name`, `W::p_pr`, `W::p_style`, `W::p`, `W::pict`, `W::r_pr`, `W::sect_pr`, `W::t`, `W::val`
- **Called by:** `coverage_round_two_tests::whitespace_fold_adopts_list_layout_and_replaces_inserted_mark` (comparer/finalize.rs:17509); `coverage_round_two_tests::whitespace_field_residue_never_inherits_heading_style` (comparer/finalize.rs:17554); `coverage_round_three_tests::runless_empty_carrier_preserves_cover_spacers_and_long_prose` (comparer/finalize.rs:18794); `coverage_round_three_tests::trailing_whitespace_spacing_folds_only_after_contentful_insertions` (comparer/finalize.rs:18830); `coverage_round_three_tests::whitespace_carriers_without_mark_shell_adopt_deleted_pilcrow` (comparer/finalize.rs:18868); `deep_production_path_tests::whitespace_mark_shell_variants_keep_existing_format_without_inventing_a_mark` (comparer/finalize.rs:21612); `deep_production_path_tests::whitespace_spacing_guard_searches_past_noninserted_predecessors` (comparer/finalize.rs:21660); `deep_production_path_tests::complete_field_carriers_keep_title_deletions_as_mark_only_properties` (comparer/finalize.rs:23480); `finalizer_distinct_boundary_contract_tests::whitespace_layout_before_a_deleted_block_is_not_empty_content` (comparer/finalize.rs:30060); `whitespace_source_mark_carrier_tests::whitespace_fold_preserves_selected_source_mark_shells_and_every_body_owner` (comparer/finalize.rs:33360); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `rotate_ins_mark_del_only_paragraph`  (L4576-4702, pub)
`pub fn rotate_ins_mark_del_only_paragraph(dom: &mut Dom, root: NodeId)`

- **Does:** M471 — first slice of the mesh off-by-one rotation (diff_after19 × diff_after2 oracle).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L4577,4656; `Dom::elements` (xmllinq/mod.rs:640) L4581,4592,4621,4633,4667,4681; `Dom::name_is` (xmllinq/mod.rs:535) L4584,4595,4601,4607,4624,4626...; `para_mark_revision` (comparer/finalize.rs:7626) L4589,4618,4630; `Dom::remove` (xmllinq/mod.rs:1102) L4660,4661,4666,4680,4692; `Dom::add_first` (xmllinq/mod.rs:1085) L4662,4663; `Dom::add_before_self` (xmllinq/mod.rs:1114) L4673,4687; `Dom::add` (xmllinq/mod.rs:1058) L4674,4688,4693
- **Calls (std/external/unresolved):** `.any`, `.array_windows`, `.collect`, `.copied`, `.filter`, `.find`, `.into_iter`, `.is_empty`, `.iter`, `.len`, `.push`, `.rev`, `Vec::new`, `W::body`, `W::del`, `W::ins`, `W::p_pr`, `W::p`
- **Called by:** `deep_production_path_tests::three_paragraph_rotation_trades_full_properties_and_preserves_each_revision` (comparer/finalize.rs:22017); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `restamp_stranded_del_mark_onto_del_only_paragraph`  (L4734-4813, pub)
`pub fn restamp_stranded_del_mark_onto_del_only_paragraph(dom: &mut Dom, root: NodeId)`

- **Does:** M473 — sibling of the M471 rotation: the deletion mark lands one slot late.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L4735,4751,4784,4799; `Dom::elements` (xmllinq/mod.rs:640) L4739,4752,4756,4776,4787,4791; `Dom::name_is` (xmllinq/mod.rs:535) L4742,4759,4761,4767,4779,4788...; `para_mark_revision` (comparer/finalize.rs:7626) L4748,4773; `Dom::descendants` (xmllinq/mod.rs:668) L4765; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L4798; `Dom::add_before_self` (xmllinq/mod.rs:1114) L4801; `Dom::remove` (xmllinq/mod.rs:1102) L4802; `Dom::add_first` (xmllinq/mod.rs:1085) L4804
- **Calls (std/external/unresolved):** `.any`, `.array_windows`, `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.iter`, `.len`, `W::body`, `W::del_text`, `W::del`, `W::ins`, `W::p_pr`, `W::p`, `W::r_pr`
- **Called by:** `coverage_round_two_tests::stranded_mark_is_cloned_without_removing_empty_shell` (comparer/finalize.rs:17239); `finalizer_distinct_boundary_contract_tests::a_stranded_mark_can_be_copied_without_deleting_its_layout_shell` (comparer/finalize.rs:30495); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `split_head_short_title_long_del_mix`  (L4829-4934, pub)
`pub fn split_head_short_title_long_del_mix(dom: &mut Dom, root: NodeId)`

- **Does:** M469 — split a head title MIX: B's SHORT inserted title (≤4 tokens) paired with A's LONG deleted opening (≥10 tokens, no shared significant token) is Word's [pure-INS title] + [style-less MARK-DEL paragraph] (ooxml_rfonts_rstyle × sd_2672_rtl_table oracle: bare pPr, no pStyle — the deleted text renders at body size, n...
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L4830; `Dom::elements` (xmllinq/mod.rs:640) L4833,4842; `Dom::name_is` (xmllinq/mod.rs:535) L4836,4845,4856,4861; `para_mark_revision` (comparer/finalize.rs:7626) L4848,4849; `Dom::descendants` (xmllinq/mod.rs:668) L4874,4906; `Dom::value_str` (xmllinq/mod.rs:1179) L4875; `text_of` (changes.rs:806) (by name) L4880,4881; `shared_word_alignment_tests::toks` (comparer/lcs_table.rs:1884) L4888,4889; `Dom::attribute` (xmllinq/mod.rs:874) L4904,4905,4910; `Dom::new_element` (xmllinq/mod.rs:478) L4915,4916,4917,4918; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L4919,4921,4924; `Dom::add` (xmllinq/mod.rs:1058) L4926,4927,4928,4931; `Dom::remove` (xmllinq/mod.rs:1102) L4930; `Dom::add_after_self` (xmllinq/mod.rs:1131) L4933
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_str`, `.collect`, `.contains`, `.filter_map`, `.filter`, `.find`, `.into_iter`, `.is_alphanumeric`, `.is_empty`, `.iter`, `.len`, `.map`, `.max`, `.ok`, `.parse`, `.push_str`, `.push`, `.split`, `.to_string`, `.unwrap_or`, `Some`, `String::new`, `Vec::new`, `W::author`, `W::body`, `W::date`, `W::del_text`, `W::del`, `W::id`, `W::ins`, `W::p_pr`, `W::p`, `W::r_pr`, `W::t`
- **Called by:** `coverage_round_two_tests::short_head_split_preserves_title_style_and_deleted_wrapper_identity` (comparer/finalize.rs:17176); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `fold_leading_ins_from_mix_into_preceding_pure_del`  (L4950-5066, pub)
`pub fn fold_leading_ins_from_mix_into_preceding_pure_del(dom: &mut Dom, root: NodeId)`

- **Does:** M105 — fold the leading insertion of a MIX paragraph back onto a preceding pure-D, leaving the MIX as pure-D + the trailing tokens of the MIX.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L4951,5051,5053,5054; `Dom::elements` (xmllinq/mod.rs:640) L4955,4987,5027; `Dom::name_is` (xmllinq/mod.rs:535) L4958,4964,4990,4993,5030; `para_is_pure_deleted` (comparer/finalize.rs:2858) L4967; `Dom::descendants` (xmllinq/mod.rs:668) L4971,4972,5005; `para_mark_revision` (comparer/finalize.rs:7626) L4973; `para_revision_body_text` (comparer/finalize.rs:7854) L4978; `Dom::value_str` (xmllinq/mod.rs:1179) L5006; `Dom::parent` (xmllinq/mod.rs:597) L5032,5038; `Dom::remove` (xmllinq/mod.rs:1102) L5035; `Dom::add_before_self` (xmllinq/mod.rs:1114) L5039; `Dom::add` (xmllinq/mod.rs:1058) L5041,5044
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.count`, `.filter`, `.find`, `.intersection`, `.into_iter`, `.is_alphanumeric`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.len`, `.map`, `.push_str`, `.push`, `.saturating_sub`, `.split`, `.to_ascii_lowercase`, `.trim`, `Some`, `String::new`, `Vec::new`, `W::body`, `W::del`, `W::ins`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`, `W::t`
- **Called by:** `deep_production_path_tests::leading_subtitle_fold_moves_multiple_wrappers_and_retains_deleted_layout` (comparer/finalize.rs:22127); `deep_production_path_tests::leading_subtitle_gate_requires_disjoint_significant_tokens_and_eight_token_cap` (comparer/finalize.rs:22166); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `strip_last_pure_del_mark_only_ppr`  (L5081-5114, pub)
`pub fn strip_last_pure_del_mark_only_ppr(dom: &mut Dom, root: NodeId)`

- **Does:** M85b — last pure-del with content: Word often omits mark-only `pPr/rPr/del` (bare `<w:del>…</w:del>` only).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L5082,5096; `Dom::elements` (xmllinq/mod.rs:640) L5085,5101,5109; `Dom::name_is` (xmllinq/mod.rs:535) L5086,5089,5106,5110; `para_is_pure_deleted` (comparer/finalize.rs:2858) L5089; `para_has_no_text` (comparer/finalize.rs:3459) L5093; `Dom::remove` (xmllinq/mod.rs:1102) L5113
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.len`, `.rev`, `W::body`, `W::del`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `para_is_mixed_revision`  (L5117-5136)
`fn para_is_mixed_revision(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when paragraph body has both ins and del content (mixed residual).
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L5118; `Dom::descendants` (xmllinq/mod.rs:668) L5126,5128; `para_mark_revision` (comparer/finalize.rs:7626) L5126,5128
- **Calls (std/external/unresolved):** `.and_then`, `.as_mut`, `.as_ref`, `.borrow_mut`, `.borrow`, `.copied`, `.get`, `.insert`, `.is_empty`, `.with`, `Some`, `W::del`, `W::ins`, `W::p`
- **Called by:** `last_pure_del_spacing_to_pprchange` (comparer/finalize.rs:5164); `mixed_spacing_to_following_empty` (comparer/finalize.rs:5267); `cleanup_spacing_and_default_jc` (comparer/finalize.rs:5381); `park_mixed_spacing_onto_trailing_pure_del` (comparer/finalize.rs:5602); `park_mixed_numpr_onto_trailing_empty_pure_del` (comparer/finalize.rs:5742); `strip_empty_pprchange_on_mix_with_live_jc` (comparer/finalize.rs:5932); `promote_heading_spacing_from_pprchange_on_last_mix` (comparer/finalize.rs:6426); `promote_live_jc_from_pprchange_on_body_mix` (comparer/finalize.rs:6511); `strip_last_pure_del_mark_when_pprchange` (comparer/finalize.rs:6593); `park_jc_on_first_short_title_mix_from_body` (comparer/finalize.rs:13103); `ensure_empty_pprchange_on_live_heading_spacing` (comparer/finalize.rs:13215); `ensure_empty_pprchange_on_eq_with_live_jc` (comparer/finalize.rs:13403); +15 more

#### `last_pure_del_spacing_to_pprchange`  (L5164-5253, pub)
`pub fn last_pure_del_spacing_to_pprchange( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** M83b / M87 / M91 / M93 / M94 — last pure-deleted **or mixed** body paragraph: Word moves live layout props into `w:pPrChange` and drops the mark-only del shell.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L5170,5212,5215,5233; `Dom::elements` (xmllinq/mod.rs:640) L5173; `Dom::name_is` (xmllinq/mod.rs:535) L5174,5177; `para_is_mixed_revision` (comparer/finalize.rs:5117) L5184; `para_is_pure_deleted` (comparer/finalize.rs:2858) L5185; `para_keeps_unchanged_words` (comparer/finalize.rs:2034) L5200; `para_side_word_count` (comparer/finalize.rs:2013) L5203,5204; `project_para_properties_for_change` (comparer/formatchg.rs:329) L5242; `Dom::remove` (xmllinq/mod.rs:1102) L5244; `Dom::new_element` (xmllinq/mod.rs:478) L5246; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L5247,5249,5250; `Dom::add` (xmllinq/mod.rs:1058) L5251,5252
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.is_empty`, `.is_some`, `.iter`, `.push`, `.rev`, `.to_string`, `Some`, `Vec::new`, `W::author`, `W::body`, `W::date`, `W::id`, `W::jc_el`, `W::name`, `W::num_pr`, `W::p_pr_change`, `W::p_pr`, `W::p_style`, `W::p`, `W::sect_pr`, `W::spacing_el`
- **Macros:** `vec!`
- **Called by:** `parked_paragraph_history_regressions::parked_layout_restores_properties_that_remain_live_when_rejected` (comparer/finalize.rs:23595); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `mixed_spacing_to_following_empty`  (L5267-5365, pub)
`pub fn mixed_spacing_to_following_empty( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** M98b (file_167): mixed I+D residual followed by empty trailing para — Word keeps del mark on the mixed (no live spacing) and parks the layout spacing on the empty para as live spacing + `pPrChange` (empty old).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L5273,5307,5310,5314,5326,5332...; `Dom::elements` (xmllinq/mod.rs:640) L5276,5318,5360; `Dom::name_is` (xmllinq/mod.rs:535) L5279,5287,5301; `para_is_mixed_revision` (comparer/finalize.rs:5117) L5290; `para_has_no_text` (comparer/finalize.rs:3459) L5293; `Dom::descendants` (xmllinq/mod.rs:668) L5302,5303; `Dom::new_element` (xmllinq/mod.rs:478) L5317,5333,5334,5346,5352; `Dom::add_before_self` (xmllinq/mod.rs:1114) L5319; `Dom::add` (xmllinq/mod.rs:1058) L5321,5339,5340,5347,5357; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L5327; `Dom::add_first` (xmllinq/mod.rs:1085) L5328; `Dom::remove` (xmllinq/mod.rs:1102) L5330,5361; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L5335,5337,5338,5353,5355,5356
- **Calls (std/external/unresolved):** `.collect`, `.copied`, `.filter`, `.first`, `.into_iter`, `.is_empty`, `.is_none`, `.len`, `.to_string`, `Some`, `W::author`, `W::body`, `W::date`, `W::del`, `W::id`, `W::ins`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`, `W::spacing_el`, `W::tbl`
- **Called by:** `coverage_round_two_tests::mixed_spacing_moves_to_blank_and_preserves_existing_history` (comparer/finalize.rs:17280); `coverage_round_two_tests::mixed_spacing_does_not_cross_live_blocks_or_fill_textual_paragraphs` (comparer/finalize.rs:17340); `coverage_real_revision_merge_matrix_tests::word_m98b_spacing_and_history_match_the_complete_format_golden` (comparer/finalize.rs:26601); `existing_mark_layout_ownership_tests::moving_mixed_spacing_keeps_existing_inserted_and_deleted_mark_owners` (comparer/finalize.rs:32828); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `cleanup_spacing_and_default_jc`  (L5381-5549, pub)
`pub fn cleanup_spacing_and_default_jc(dom: &mut Dom, root: NodeId)`

- **Does:** M228 / M226 / M231 single body walk (perf: avoid 3× O(n_p) on large docs).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L5382,5400,5405,5412,5421,5450...; `Dom::elements` (xmllinq/mod.rs:640) L5385,5414,5430,5453,5484,5518; `Dom::name_is` (xmllinq/mod.rs:535) L5388,5397,5417,5456; `Dom::descendants` (xmllinq/mod.rs:668) L5395; `Dom::attribute` (xmllinq/mod.rs:874) L5406,5422,5473,5474,5479,5480...; `Dom::remove` (xmllinq/mod.rs:1102) L5408,5424,5441,5496,5539,5547; `Dom::name` (xmllinq/mod.rs:526) L5431,5485,5519; `XName::local_name` (xmllinq/mod.rs:208) (by name) L5434,5488,5522; `para_is_mixed_revision` (comparer/finalize.rs:5117) L5466; `para_is_pure_deleted` (comparer/finalize.rs:2858) L5514; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L5545; `Dom::add_before_self` (xmllinq/mod.rs:1114) L5546
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.enumerate`, `.filter`, `.find`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.len`, `.unwrap_or`, `Some`, `W::body`, `W::del`, `W::jc_el`, `W::name`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::sect_pr`, `W::spacing_el`, `W::val`
- **Called by:** `promote_mid_pure_del_spacing_from_pprchange` (comparer/finalize.rs:5561); `strip_default_left_jc` (comparer/finalize.rs:5573); `strip_redundant_equal_spacing_pprchange` (comparer/finalize.rs:5583); `coverage_tests::cleanup_spacing_removes_only_redundant_mixed_history` (comparer/finalize.rs:16232); `coverage_tests::cleanup_default_alignment_keeps_real_layout_history` (comparer/finalize.rs:16272); `coverage_tests::cleanup_deleted_spacing_promotes_mid_body_but_preserves_last_history` (comparer/finalize.rs:16306); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `promote_mid_pure_del_spacing_from_pprchange`  (L5561-5563, pub)
`pub fn promote_mid_pure_del_spacing_from_pprchange(dom: &mut Dom, root: NodeId)`

- **Does:** Thin wrappers kept for call-site clarity / tests.
- **Calls (crate):** `cleanup_spacing_and_default_jc` (comparer/finalize.rs:5381) L5562
- **Called by:** none found in src

#### `strip_default_left_jc`  (L5573-5575, pub)
`pub fn strip_default_left_jc(dom: &mut Dom, root: NodeId)`

- **Does:** Strip default left `jc` restatements (legacy name; shares cleanup pass).
- **Calls (crate):** `cleanup_spacing_and_default_jc` (comparer/finalize.rs:5381) L5574
- **Called by:** none found in src

#### `strip_redundant_equal_spacing_pprchange`  (L5583-5585, pub)
`pub fn strip_redundant_equal_spacing_pprchange(dom: &mut Dom, root: NodeId)`

- **Does:** Strip redundant equal-spacing `pPrChange` (legacy name; shares cleanup pass).
- **Calls (crate):** `cleanup_spacing_and_default_jc` (comparer/finalize.rs:5381) L5584
- **Called by:** none found in src

#### `park_mixed_spacing_onto_trailing_pure_del`  (L5602-5728, pub)
`pub fn park_mixed_spacing_onto_trailing_pure_del( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** M221 (green_underline×heading_1_bold ~56): MIX residual carries B Heading spacing (before=400 after=120 line=240) while following pure-D bullets have none.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L5608,5624,5627,5648,5650,5680...; `Dom::elements` (xmllinq/mod.rs:640) L5611,5684,5723; `Dom::name_is` (xmllinq/mod.rs:535) L5614,5621,5632; `para_is_mixed_revision` (comparer/finalize.rs:5117) L5621; `para_is_pure_deleted` (comparer/finalize.rs:2858) L5632; `para_has_no_text` (comparer/finalize.rs:3459) L5645; `para_body_alnum_len` (comparer/finalize.rs:7884) L5658; `body_token_set` (comparer/finalize.rs:7865) L5659; `para_revision_body_text` (comparer/finalize.rs:7854) L5659; `Dom::attribute` (xmllinq/mod.rs:874) L5666,5670; `Dom::new_element` (xmllinq/mod.rs:478) L5683,5696,5697,5710,5716; `Dom::add_before_self` (xmllinq/mod.rs:1114) L5685; `Dom::add` (xmllinq/mod.rs:1058) L5687,5702,5703,5711,5721; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L5692; `Dom::add_first` (xmllinq/mod.rs:1085) L5693; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L5698,5700,5701,5717,5719,5720; `Dom::remove` (xmllinq/mod.rs:1102) L5706,5724
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.contains`, `.copied`, `.filter`, `.first`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some`, `.len`, `.ok`, `.parse`, `.to_string`, `.unwrap_or`, `Some`, `W::author`, `W::body`, `W::date`, `W::del`, `W::id`, `W::ins`, `W::name`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`, `W::spacing_el`
- **Called by:** `coverage_round_two_tests::heading_spacing_parks_on_last_short_deleted_residual` (comparer/finalize.rs:17359); `coverage_round_two_tests::spacing_park_rejects_count_length_and_heading_thresholds` (comparer/finalize.rs:17404); `existing_mark_layout_ownership_tests::parking_heading_spacing_keeps_existing_lifetime_and_history_provenance` (comparer/finalize.rs:32906); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `park_mixed_numpr_onto_trailing_empty_pure_del`  (L5742-5836, pub)
`pub fn park_mixed_numpr_onto_trailing_empty_pure_del( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** M230 (bullet_list_bold×bullet_list ~83): MIX residual carries live `numPr` (B list item "Grapes") while trailing pure-D empties from A have none.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L5748,5764,5767,5787,5789,5799...; `Dom::elements` (xmllinq/mod.rs:640) L5751,5803; `Dom::name_is` (xmllinq/mod.rs:535) L5754,5761,5772; `para_is_mixed_revision` (comparer/finalize.rs:5117) L5761; `para_is_pure_deleted` (comparer/finalize.rs:2858) L5772; `Dom::new_element` (xmllinq/mod.rs:478) L5802,5824,5825; `Dom::add_before_self` (xmllinq/mod.rs:1114) L5804,5817,5819; `Dom::add` (xmllinq/mod.rs:1058) L5806,5830,5831; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L5814; `Dom::add_first` (xmllinq/mod.rs:1085) L5821; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L5826,5828,5829; `Dom::remove` (xmllinq/mod.rs:1102) L5833
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.contains`, `.copied`, `.filter`, `.first`, `.into_iter`, `.is_none`, `.is_some`, `.len`, `.to_string`, `Some`, `W::author`, `W::body`, `W::date`, `W::id`, `W::num_pr`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`
- **Called by:** `coverage_round_two_tests::mixed_numbering_moves_before_last_deletion_mark_or_history` (comparer/finalize.rs:17441); `coverage_round_two_tests::numbering_park_preserves_short_long_and_already_numbered_runs` (comparer/finalize.rs:17492); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `last_pure_del_inherit_prev_jc`  (L5842-5917, pub)
`pub fn last_pure_del_inherit_prev_jc(dom: &mut Dom, root: NodeId)`

- **Does:** M102c (file_148): last pure-del with `pPrChange(spacing)` and no live `jc` inherits live `jc` from a preceding body para that has center align (Word pure-D of A line-spacing sentence keeps B's center mark even when the immediate prev mixed residual only has spacing).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L5843,5858,5861,5866,5873,5881...; `Dom::elements` (xmllinq/mod.rs:640) L5846,5869,5895; `Dom::name_is` (xmllinq/mod.rs:535) L5849,5855,5871,5888,5896,5899; `para_is_pure_deleted` (comparer/finalize.rs:2858) L5855; `Dom::descendants` (xmllinq/mod.rs:668) L5876; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L5911; `Dom::add_before_self` (xmllinq/mod.rs:1114) L5913; `Dom::add_first` (xmllinq/mod.rs:1085) L5915
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.find`, `.into_iter`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.or_else`, `.rev`, `Some`, `W::body`, `W::jc_el`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::sect_pr`, `W::spacing_el`
- **Called by:** `existing_mark_layout_ownership_tests::alignment_donor_search_crosses_default_property_paragraphs_without_claiming_their_formats` (comparer/finalize.rs:32991); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `strip_empty_pprchange_on_mix_with_live_jc`  (L5932-5992, pub)
`pub fn strip_empty_pprchange_on_mix_with_live_jc(dom: &mut Dom, root: NodeId)`

- **Does:** M451 (center_alignment_2 × center_alignment ~81.5 / docxodus 100): mid MIX free-mesh carries **empty** `pPrChange` alongside live `jc` where Word uses live `jc` + rPr/ins mark only.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L5933,5955,5956,5957,5962,5966...; `Dom::elements` (xmllinq/mod.rs:640) L5936,5977; `Dom::name_is` (xmllinq/mod.rs:535) L5939,5946,5953; `para_is_mixed_revision` (comparer/finalize.rs:5117) L5946; `Dom::name` (xmllinq/mod.rs:526) L5978; `Dom::remove` (xmllinq/mod.rs:1102) L5990
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.enumerate`, `.filter`, `.find`, `.into_iter`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `W::body`, `W::jc_el`, `W::name`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `strip_propertyless_ppr`  (L6003-6042, pub)
`pub fn strip_propertyless_ppr(dom: &mut Dom, root: NodeId)`

- **Does:** Drop a `w:pPr` that records nothing Word keeps.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L6004,6009,6018; `Dom::descendants` (xmllinq/mod.rs:668) L6007; `Dom::elements` (xmllinq/mod.rs:640) L6012,6019,6038; `Dom::name_is` (xmllinq/mod.rs:535) L6017,6028; `Dom::name` (xmllinq/mod.rs:526) L6021; `Dom::remove` (xmllinq/mod.rs:1102) L6036,6039
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.is_some_and`, `.iter`, `.push`, `Some`, `Vec::new`, `W::body`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209); `compare_documents_impl` (document_comparer.rs:6501)

#### `pg_num_type_is_implicit_decimal`  (L6048-6056, pub(crate))
`pub(crate) fn pg_num_type_is_implicit_decimal(dom: &Dom, n: NodeId) -> bool`

- **Does:** `w:pgNumType` that only restates Word's implicit `decimal` format.
- **Calls (crate):** `Dom::attributes` (xmllinq/mod.rs:885) L6049; `XName::local_name` (xmllinq/mod.rs:208) (by name) L6050
- **Called by:** `strip_unrecorded_word_defaults` (comparer/finalize.rs:6077); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `strip_unrecorded_word_defaults`  (L6077-6147, pub)
`pub fn strip_unrecorded_word_defaults(dom: &mut Dom, root: NodeId, original_line: Option<&str>)`

- **Does:** Markup Word's redline leaves out once the text already matches.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L6079,6109,6119,6122,6123,6124; `Dom::elements` (xmllinq/mod.rs:640) L6082; `Dom::name_is` (xmllinq/mod.rs:535) L6086; `Dom::descendants` (xmllinq/mod.rs:668) L6088,6093,6098,6103,6108,6116; `Dom::attribute` (xmllinq/mod.rs:874) L6089,6110; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L6094,6099; `pg_num_type_is_implicit_decimal` (comparer/finalize.rs:6048) L6104; `spacing_is_only_line_276` (comparer/finalize.rs:6149) L6126; `content_revision_marks` (comparer/finalize.rs:6184) L6127,6136; `pilcrow_ins_only` (comparer/finalize.rs:6169) L6135; `Dom::remove` (xmllinq/mod.rs:1102) L6142,6145
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.is_none_or`, `.is_some`, `.push`, `.rev`, `.unwrap_or`, `Some`, `Vec::new`, `W::body`, `W::del`, `W::jc_el`, `W::name`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`, `W::spacing_el`, `W::val`
- **Macros:** `matches!`
- **Called by:** `coverage_round_next_tests::unrecorded_spacing_guards_preserve_all_explicit_vertical_metrics` (comparer/finalize.rs:20277); `split_paragraph_boundary_regressions::mixed_inserted_boundary_joins_original_text_and_closing_layout_on_reject` (comparer/finalize.rs:23730); `public_body_heuristic_story_scope_tests::public_body_layout_heuristics_preserve_complete_independent_header_and_footer_parts` (comparer/finalize.rs:32251); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209); `compare_documents_impl` (document_comparer.rs:6501)

#### `spacing_is_only_line_276`  (L6149-6166)
`fn spacing_is_only_line_276(dom: &Dom, sp: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L6150,6161
- **Calls (std/external/unresolved):** `.is_some`, `Some`, `W::name`
- **Called by:** `strip_unrecorded_word_defaults` (comparer/finalize.rs:6077)

#### `pilcrow_ins_only`  (L6169-6180)
`fn pilcrow_ins_only(dom: &Dom, ppr: NodeId) -> bool`

- **Does:** `w:pPr` whose only child is `w:rPr/w:ins`.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L6170,6178; `Dom::name_is` (xmllinq/mod.rs:535) L6175,6179
- **Calls (std/external/unresolved):** `.len`, `W::ins`, `W::r_pr`
- **Called by:** `strip_unrecorded_word_defaults` (comparer/finalize.rs:6077)

#### `content_revision_marks`  (L6184-6200)
`fn content_revision_marks(dom: &Dom, p: NodeId, ppr: NodeId) -> (bool, bool)`

- **Does:** Whether the paragraph's content, outside its `w:pPr`, holds `w:ins` and `w:del`.
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L6186; `Dom::descendants` (xmllinq/mod.rs:668) L6189; `Dom::name_is` (xmllinq/mod.rs:535) L6193,6195
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.into_iter`, `W::del`, `W::ins`
- **Called by:** `merged_closing_carrier_takes_source_properties` (comparer/finalize.rs:3080); `strip_unrecorded_word_defaults` (comparer/finalize.rs:6077)

#### `align_word_table_and_comment_chrome`  (L6215-6322, pub)
`pub fn align_word_table_and_comment_chrome(dom: &mut Dom, root: NodeId)`

- **Does:** Table and comment-reference chrome Word writes on every matching redline in this corpus.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L6216,6230,6249,6252,6259,6268...; `Dom::descendants` (xmllinq/mod.rs:668) L6219,6247,6289; `Dom::ancestors` (xmllinq/mod.rs:770) L6224; `Dom::new_element` (xmllinq/mod.rs:478) L6233,6271,6272,6274,6301,6307...; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L6234,6243,6275,6276,6308,6313...; `insert_tblpr_child_in_order` (comparer/finalize.rs:10449) L6245; `Dom::attribute` (xmllinq/mod.rs:874) L6260; `Dom::elements` (xmllinq/mod.rs:640) L6266,6282; `Dom::add` (xmllinq/mod.rs:1058) L6277,6279,6285,6309,6314,6319; `Dom::add_before_self` (xmllinq/mod.rs:1114) L6281,6283; `Dom::ancestors_and_self` (xmllinq/mod.rs:787) L6291; `Dom::name_is` (xmllinq/mod.rs:535) L6294; `Dom::add_first` (xmllinq/mod.rs:1085) L6302
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.copied`, `.find`, `.first`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `Some`, `W::body`, `W::name`, `W::r_pr`, `W::r`, `W::tbl_pr`, `W::tbl`, `W::tr_pr`, `W::tr`, `W::val`
- **Called by:** `coverage_round_next_tests::table_chrome_adds_look_before_history_and_handles_all_row_positions` (comparer/finalize.rs:19944); `coverage_round_next_tests::comment_chrome_fills_each_missing_property_independently_and_ignores_orphans` (comparer/finalize.rs:19966); `coverage_round_next_tests::compatibility_defaults_keep_nonzero_vertical_margins_and_existing_look` (comparer/finalize.rs:20560); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209); `compare_documents_impl` (document_comparer.rs:6501)

#### `rewrite_inserted_external_hyperlinks`  (L6331-6391, pub)
`pub fn rewrite_inserted_external_hyperlinks( dom: &mut Dom, root: NodeId, id_to_target: &HashMap<String, String>, base_targets: &HashSet<String>, )`

- **Does:** A wholly inserted external hyperlink whose URL is not in the base document becomes Word's `HYPERLINK` field.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L6337; `Dom::attribute` (xmllinq/mod.rs:874) L6339,6342; `hyperlink_is_wholly_inserted` (comparer/finalize.rs:6393) L6351; `field_char_run` (comparer/finalize.rs:6409) L6355,6360,6361; `Dom::new_element` (xmllinq/mod.rs:478) L6356,6357; `Dom::add_text` (xmllinq/mod.rs:1071) L6358; `Dom::add` (xmllinq/mod.rs:1058) L6359,6375; `Dom::parent` (xmllinq/mod.rs:597) L6362; `Dom::name_is` (xmllinq/mod.rs:535) L6362; `Dom::add_before_self` (xmllinq/mod.rs:1114) L6364,6365,6366,6387; `Dom::add_after_self` (xmllinq/mod.rs:1131) L6367; `Dom::elements` (xmllinq/mod.rs:640) L6370; `Dom::add_first` (xmllinq/mod.rs:1085) L6372,6373,6374; `Dom::nodes` (xmllinq/mod.rs:604) L6384; `Dom::remove` (xmllinq/mod.rs:1102) L6386,6389
- **Calls (std/external/unresolved):** `.cloned`, `.contains`, `.first`, `.get`, `.is_some_and`, `.is_some`, `.last`, `.map`, `R::name`, `Some`, `W::hyperlink`, `W::ins`, `W::name`, `W::r`
- **Macros:** `format!`
- **Called by:** `coverage_final_batch_finalize_tests::external_link_rewrite_matrix_requires_new_target_and_wholly_inserted_content` (comparer/finalize.rs:24380); `compare_documents_impl` (document_comparer.rs:6501)

#### `hyperlink_is_wholly_inserted`  (L6393-6407)
`fn hyperlink_is_wholly_inserted(dom: &Dom, hl: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L6394,6401,6402; `Dom::value_str` (xmllinq/mod.rs:1179) L6395; `Dom::ancestors_and_self` (xmllinq/mod.rs:787) L6396; `Dom::name_is` (xmllinq/mod.rs:535) L6399,6403; `Dom::parent` (xmllinq/mod.rs:597) L6403
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.is_some_and`, `.iter`, `.trim`, `Some`, `W::del_text`, `W::ins`, `W::t`
- **Called by:** `rewrite_inserted_external_hyperlinks` (comparer/finalize.rs:6331)

#### `field_char_run`  (L6409-6415)
`fn field_char_run(dom: &mut Dom, ty: &str) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L6410,6411; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L6412; `Dom::add` (xmllinq/mod.rs:1058) L6413
- **Calls (std/external/unresolved):** `Some`, `W::name`, `W::r`
- **Called by:** `rewrite_inserted_external_hyperlinks` (comparer/finalize.rs:6331)

#### `promote_heading_spacing_from_pprchange_on_last_mix`  (L6426-6496, pub)
`pub fn promote_heading_spacing_from_pprchange_on_last_mix(dom: &mut Dom, root: NodeId)`

- **Does:** M450 (calibri_font × calibri_heading_2_right ~82.5 / docxodus 100): last MIX free-mesh parks **Heading residual** spacing (`before≥200` + `line=240`) into `pPrChange` only; Word keeps it **live** with an empty `pPrChange`.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L6427,6441,6445,6448,6451,6454...; `Dom::elements` (xmllinq/mod.rs:640) L6430; `Dom::name_is` (xmllinq/mod.rs:535) L6433,6438,6470; `para_is_mixed_revision` (comparer/finalize.rs:5117) L6438; `Dom::attribute` (xmllinq/mod.rs:874) L6458,6462,6479,6483; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L6492; `Dom::add_before_self` (xmllinq/mod.rs:1114) L6493; `Dom::remove` (xmllinq/mod.rs:1102) L6495
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.filter`, `.into_iter`, `.is_some`, `.last`, `.len`, `.ok`, `.parse`, `.unwrap_or`, `W::body`, `W::name`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::sect_pr`, `W::spacing_el`
- **Called by:** `coverage_round_two_tests::heading_spacing_promotion_requires_prior_live_heading_and_preserves_old_shell` (comparer/finalize.rs:17961); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `promote_live_jc_from_pprchange_on_body_mix`  (L6511-6581, pub)
`pub fn promote_live_jc_from_pprchange_on_body_mix( dom: &mut Dom, root: NodeId, revised_unaligned: &HashSet<String>, )`

- **Does:** M449 (right_aligned_italic × right_alignment ~75.9 / docxodus 100): mid-body MIX free-mesh parks `jc` into `pPrChange` only; Word keeps **live** `jc` on the body MIX.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L6516,6537,6543,6544,6548,6551...; `Dom::elements` (xmllinq/mod.rs:640) L6519,6567; `Dom::name_is` (xmllinq/mod.rs:535) L6522,6529; `para_is_mixed_revision` (comparer/finalize.rs:5117) L6529; `para_side_word_count` (comparer/finalize.rs:2013) L6532,6533; `Dom::attribute` (xmllinq/mod.rs:874) L6557; `accepted_text` (comparer/finalize.rs:13333) L6558; `Dom::new_element` (xmllinq/mod.rs:478) L6562; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L6563; `Dom::add_before_self` (xmllinq/mod.rs:1114) L6564; `Dom::name` (xmllinq/mod.rs:526) L6568; `Dom::remove` (xmllinq/mod.rs:1102) L6578
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter`, `.into_iter`, `.is_empty`, `.is_some`, `.len`, `.to_string`, `.unwrap_or`, `Some`, `W::body`, `W::jc_el`, `W::name`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`, `W::val`
- **Called by:** `coverage_round_two_tests::old_body_alignment_promotes_only_for_aligned_revised_paragraphs` (comparer/finalize.rs:17994); `public_body_heuristic_story_scope_tests::public_body_layout_heuristics_preserve_complete_independent_header_and_footer_parts` (comparer/finalize.rs:32251); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `strip_last_pure_del_mark_when_pprchange`  (L6593-6629, pub)
`pub fn strip_last_pure_del_mark_when_pprchange(dom: &mut Dom, root: NodeId)`

- **Does:** M87b / M94 — last pure-del **or mixed** that already has `pPrChange`: drop mark-only `rPr/del` (Word keeps pPrChange only — file_55 "b", file_139 mixed).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L6594,6607,6610,6614,6615; `Dom::elements` (xmllinq/mod.rs:640) L6597,6618,6624; `Dom::name_is` (xmllinq/mod.rs:535) L6598,6601; `para_is_pure_deleted` (comparer/finalize.rs:2858) L6604; `para_is_mixed_revision` (comparer/finalize.rs:5117) L6604; `Dom::remove` (xmllinq/mod.rs:1102) L6621,6623,6625
- **Calls (std/external/unresolved):** `.find`, `.is_empty`, `.is_none`, `.iter`, `.len`, `.rev`, `W::body`, `W::del`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`
- **Called by:** `finalizer_distinct_boundary_contract_tests::a_terminal_property_history_replaces_the_mark_but_keeps_its_formats` (comparer/finalize.rs:30534); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `reorder_replacements_ins_before_del`  (L6652-6680, pub)
`pub fn reorder_replacements_ins_before_del(dom: &mut Dom, node: NodeId)`

- **Does:** Word emits the INSERTION (new text) before the DELETION (old text) at a replacement site; our LCS produces delete-then-insert.
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L6656,6675; `revision_wrapper` (comparer/finalize.rs:6707) L6662; `fields_complete` (comparer/finalize.rs:6684) L6665,6666; `Dom::remove` (xmllinq/mod.rs:1102) L6668; `Dom::add_before_self` (xmllinq/mod.rs:1114) L6669; `Dom::is_element` (xmllinq/mod.rs:502) L6676; `reorder_replacements_ins_before_del` (comparer/finalize.rs:6652) L6677
- **Calls (std/external/unresolved):** `.len`, `Some`, `W::name`
- **Macros:** `matches!`
- **Called by:** `reorder_replacements_ins_before_del` (comparer/finalize.rs:6652); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209); `compare_documents_impl` (document_comparer.rs:6501)

#### `fields_complete`  (L6684-6699)
`fn fields_complete(dom: &Dom, node: NodeId, fld_char: &XName) -> bool`

- **Does:** Every field char under `node` belongs to a field that begins and ends under it: no `fldChar` at all, or begins and ends that nest and balance.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L6686; `Dom::attribute` (xmllinq/mod.rs:874) L6687
- **Calls (std/external/unresolved):** `Some`, `W::name`
- **Called by:** `reorder_replacements_ins_before_del` (comparer/finalize.rs:6652); `coverage_tests::field_completeness_requires_balanced_order_not_just_counts` (comparer/finalize.rs:15813); `deep_production_path_tests::complete_field_carriers_keep_title_deletions_as_mark_only_properties` (comparer/finalize.rs:23480)

#### `revision_wrapper`  (L6707-6742)
`fn revision_wrapper(dom: &Dom, node: NodeId) -> Option<(bool, String, String)>`

- **Does:** A node that is one revision for the replacement swap: a `w:ins` (`true`) or `w:del` (`false`) wrapper, or a `w:hyperlink` whose element children are all one of them by one author and date (Word's redline of super_editor hyperlink_multiple_runs × diff_before10, corpus a098bd5ad9, writes "One two" inserted before the de...
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L6708,6731; `Dom::name` (xmllinq/mod.rs:526) L6712,6726; `Dom::attribute` (xmllinq/mod.rs:874) L6719,6720; `Dom::nodes` (xmllinq/mod.rs:604) L6730
- **Calls (std/external/unresolved):** `.as_ref`, `.to_string`, `.unwrap_or`, `Some`, `W::author`, `W::date`, `W::del`, `W::ins`, `W::name`, `own`
- **Called by:** `reorder_replacements_ins_before_del` (comparer/finalize.rs:6652); `coverage_tests::revision_wrapper_requires_consistent_hyperlink_stamps` (comparer/finalize.rs:15837)

#### `coalesce_adjacent_revisions`  (L6759-6793, pub)
`pub fn coalesce_adjacent_revisions(dom: &mut Dom, node: NodeId)`

- **Does:** Merge consecutive sibling `w:ins` (and consecutive `w:del`) wrappers that share the same `w:author` and `w:date` into a single wrapper, moving the later wrapper's child runs into the first and removing the emptied wrapper.
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L6762,6775,6788; `Dom::is_element` (xmllinq/mod.rs:502) L6766,6789; `Dom::name` (xmllinq/mod.rs:526) L6769; `Dom::attribute` (xmllinq/mod.rs:874) L6770,6771,6772,6773; `Dom::remove` (xmllinq/mod.rs:1102) L6776,6779; `Dom::add` (xmllinq/mod.rs:1058) L6777; `coalesce_adjacent_revisions` (comparer/finalize.rs:6759) L6790
- **Calls (std/external/unresolved):** `.map`, `.name`, `.to_string`, `Some`, `W::author`, `W::date`, `W::del`, `W::ins`
- **Macros:** `matches!`
- **Called by:** `coalesce_adjacent_revisions` (comparer/finalize.rs:6759); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `simplify_move_markup_to_del_ins`  (L6809-6811, pub)
`pub fn simplify_move_markup_to_del_ins(dom: &mut Dom, root: NodeId) -> NodeId`

- **Does:** M4.F — `SimplifyMoveMarkupToDelIns` (:3309), gated by settings.simplify_move_markup (default off).
- **Calls (crate):** `simplify_move_transform` (comparer/finalize.rs:6813) L6810
- **Called by:** `finalizer_distinct_boundary_contract_tests::move_simplification_changes_only_lifetime_names_and_range_markers` (comparer/finalize.rs:29943); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `simplify_move_transform`  (L6813-6851)
`fn simplify_move_transform(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L6814,6839; `Dom::new_element` (xmllinq/mod.rs:478) L6828; `copy_move_id_attrs` (comparer/finalize.rs:150) L6833; `copy_attrs` (comparer/finalize.rs:136) L6835; `Dom::nodes` (xmllinq/mod.rs:604) L6837; `Dom::is_element` (xmllinq/mod.rs:502) L6838; `simplify_move_transform` (comparer/finalize.rs:6813) L6843; `Dom::add` (xmllinq/mod.rs:1058) L6844,6847; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L6846
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.unwrap`, `W::del`, `W::ins`, `W::move_from_range_end`, `W::move_to_range_end`, `W::name`
- **Called by:** `simplify_move_markup_to_del_ins` (comparer/finalize.rs:6809); `simplify_move_transform` (comparer/finalize.rs:6813)

#### `merge_replaced_paragraphs`  (L6868-6870, pub)
`pub fn merge_replaced_paragraphs(dom: &mut Dom, root: NodeId, comparer_author: &str)`

- **Does:** Word-alignment mode (settings-gated, never in PowerTools-faithful runs): Word's Compare merges a fully-replaced paragraph PAIR into one paragraph — inserted runs first, then deleted runs, under the INSERTED paragraph's properties (evidence: the `_word_redline` fixture corpus, e.g.
- **Calls (crate):** `merge_replaced_paragraphs_in` (comparer/finalize.rs:6890) L6869
- **Called by:** `coverage_round_three_tests::same_after_merge` (comparer/finalize.rs:18250); `coverage_round_three_tests::paired_normalization_preserves_prior_pilcrow_provenance_and_only_unmarks_its_own_pair` (comparer/finalize.rs:18258); `coverage_round_three_tests::related_one_for_one_adopts_exact_structural_properties_and_inserted_attributes` (comparer/finalize.rs:18333); `coverage_round_three_tests::field_replacement_keeps_bare_properties_instead_of_deleted_heading` (comparer/finalize.rs:18383); `coverage_round_three_tests::bare_related_pair_does_not_invent_paragraph_properties` (comparer/finalize.rs:18403); `coverage_round_three_tests::deleted_then_multiple_inserted_folds_only_last_and_adopts_heading` (comparer/finalize.rs:18418); `coverage_round_three_tests::last_inserted_list_adopts_deleted_listparagraph_and_numbering` (comparer/finalize.rs:18456); `coverage_round_three_tests::alignment_only_carrier_adopts_deleted_spacing_exactly` (comparer/finalize.rs:18486); `coverage_round_three_tests::story_tail_fusion_crosses_deleted_table_and_keeps_deleted_layout` (comparer/finalize.rs:18581); `coverage_round_three_tests::multiple_mark_only_deletions_fold_once_and_preserve_remaining_empty` (comparer/finalize.rs:18659); `coverage_round_three_tests::empty_heading_residual_adopts_heading_for_last_list_carrier` (comparer/finalize.rs:18689); `coverage_round_three_tests::carried_foreign_insertions_reorder_without_becoming_replacement_deletions` (comparer/finalize.rs:18709); +1 more

#### `merge_replaced_paragraphs_in`  (L6890-6905, pub)
`pub fn merge_replaced_paragraphs_in( dom: &mut Dom, root: NodeId, comparer_author: &str, story_final_paired: bool, )`

- **Does:** [`merge_replaced_paragraphs`] for a body whose two final paragraph marks are paired (`story_final_paired`): the revised document ended on an empty paragraph, so its last content paragraph closes on its own inserted mark and Word does not fold it into the first deleted paragraph (math_radical_tests × multi_image_types).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L6896; `Dom::descendants` (xmllinq/mod.rs:668) L6899; `merge_replaced_in_container` (comparer/finalize.rs:8621) L6903
- **Calls (std/external/unresolved):** `.collect`, `.extend`, `.into_iter`, `Some`, `W::body`, `W::name`, `W::sdt_content`
- **Called by:** `merge_replaced_paragraphs` (comparer/finalize.rs:6868); `coverage_round_three_tests::paired_body_tail_stays_separate_but_nested_stories_still_fold` (comparer/finalize.rs:18523); `coverage_round_three_tests::shared_title_head_junction_precedes_remaining_inserted_stream` (comparer/finalize.rs:18548); `coverage_round_three_tests::story_tail_fusion_stops_at_page_breaks_section_breaks_and_live_tables` (comparer/finalize.rs:18623); `coverage_round_three_tests::unmatched_field_blocks_shared_title_head_junction` (comparer/finalize.rs:18726); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `residual_short_label_zip_with_settings`  (L6925-7033, pub)
`pub fn residual_short_label_zip_with_settings( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, )`

- **Does:** Fold residual shared labels using the configured formatting-history policy.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L6930; `Dom::elements` (xmllinq/mod.rs:640) L6934; `Dom::name_is` (xmllinq/mod.rs:535) L6937,6942,6947; `para_is_pure_inserted` (comparer/finalize.rs:4175) L6942; `para_is_pure_deleted` (comparer/finalize.rs:2858) L6947; `para_body_is_very_short` (comparer/finalize.rs:7894) L6956,7009; `para_revision_body_text` (comparer/finalize.rs:7854) L6959,6974,7020; `Dom::parent` (xmllinq/mod.rs:597) L6971,6992,7008,7017; `fold_short_label_ins_into_del_with_settings` (comparer/finalize.rs:7053) L6995,7025; `para_body_alnum_len` (comparer/finalize.rs:7884) L7010
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.contains`, `.copied`, `.filter`, `.insert`, `.into_iter`, `.is_alphanumeric`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.last`, `.len`, `.map`, `.split`, `.to_ascii_lowercase`, `Some`, `W::body`, `W::p`, `W::sect_pr`, `std::collections::HashSet::new`
- **Called by:** `residual_short_label_zip` (comparer/finalize.rs:6918); `short_label_format_history_tests::configured_label_tracking_preserves_sources_and_respects_disabled_changes` (comparer/finalize.rs:24148); `public_body_heuristic_story_scope_tests::public_body_layout_heuristics_preserve_complete_independent_header_and_footer_parts` (comparer/finalize.rs:32251); `fragmented_source_label_boundary_tests::public_label_zip_preserves_already_adjacent_spacer_and_missing_layout_ownership` (comparer/finalize.rs:33810); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `fold_short_label_ins_into_del_with_settings`  (L7053-7171)
`fn fold_short_label_ins_into_del_with_settings( dom: &mut Dom, ip: NodeId, dp: NodeId, settings: &WmlComparerSettings, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L7060,7113,7118,7130,7141,7142...; `Dom::elements` (xmllinq/mod.rs:640) L7061,7080,7103,7108,7148,7157; `Dom::name_is` (xmllinq/mod.rs:535) L7064,7106,7111; `para_is_pure_inserted` (comparer/finalize.rs:4175) L7065; `para_has_no_text` (comparer/finalize.rs:3459) L7066; `para_body_text_is_whitespace_only` (comparer/finalize.rs:4159) L7066; `Dom::element` (xmllinq/mod.rs:655) L7075,7076,7088,7089,7094,7095...; `Dom::new_element` (xmllinq/mod.rs:478) L7079; `Dom::add_before_self` (xmllinq/mod.rs:1114) L7081,7119,7156,7163; `Dom::add` (xmllinq/mod.rs:1058) L7083,7092,7098,7121,7124,7165...; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L7091,7097; `Dom::remove` (xmllinq/mod.rs:1102) L7116,7128,7131,7153; `mesh_short_label_shared_eq_with_settings` (comparer/finalize.rs:7199) L7137
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.copied`, `.filter`, `.find`, `.first`, `.get`, `.into_iter`, `.is_none`, `.is_some`, `.iter`, `.nth`, `.position`, `.skip_while`, `Some`, `W::name`, `W::p_pr`, `W::p`, `W::spacing_el`
- **Called by:** `residual_short_label_zip_with_settings` (comparer/finalize.rs:6925); `fold_short_label_ins_into_del` (comparer/finalize.rs:7048)

#### `mesh_short_label_shared_eq_with_settings`  (L7199-7580)
`fn mesh_short_label_shared_eq_with_settings( dom: &mut Dom, p: NodeId, settings: &WmlComparerSettings, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L7207,7220,7226,7342,7347,7423...; `Dom::name_is` (xmllinq/mod.rs:535) L7208,7269,7282,7286,7290,7307...; `Dom::parent` (xmllinq/mod.rs:597) L7209,7213,7454,7502,7504,7511...; `Dom::ancestors` (xmllinq/mod.rs:770) L7210,7365; `Dom::elements` (xmllinq/mod.rs:640) L7219,7225,7265,7285,7289,7294...; `Dom::value_str` (xmllinq/mod.rs:1179) L7221,7227,7373,7392,7427,7544...; `trailing_alnum_token` (comparer/finalize.rs:7583) L7252; `Dom::name` (xmllinq/mod.rs:526) L7276,7296,7558; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L7297; `XName::local_name` (xmllinq/mod.rs:208) (by name) L7299; `Dom::nodes` (xmllinq/mod.rs:604) L7302; `mesh_short_label_shared_eq_with_settings::{fn label_run}` (comparer/finalize.rs:7317) L7351,7352; `Dom::element` (xmllinq/mod.rs:655) L7358,7359,7465; `paragraph_mark_properties_equal` (comparer/finalize.rs:3280) L7361; `max_numeric_id` (comparer/finalize.rs:14901) L7366; `strip_trailing_alnum_token` (comparer/finalize.rs:7601) L7374; `Dom::set_value` (xmllinq/mod.rs:1203) L7375,7412,7442; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L7377,7414,7444; `XNamespace::xml` (xmllinq/mod.rs:128) L7377,7414,7444; `Dom::new_element` (xmllinq/mod.rs:478) L7464,7470; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L7466; `Dom::add` (xmllinq/mod.rs:1058) L7467,7472,7484,7527; `closing_mark_takes_revised_mark_formatting` (comparer/finalize.rs:3358) L7469; `Dom::add_text` (xmllinq/mod.rs:1071) L7471; `Dom::add_after_self` (xmllinq/mod.rs:1131) L7474; `Dom::add_before_self` (xmllinq/mod.rs:1114) L7482,7525; `Dom::remove` (xmllinq/mod.rs:1102) L7523,7545,7562,7577
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.chars`, `.collect`, `.copied`, `.count`, `.ends_with`, `.enumerate`, `.eq_ignore_ascii_case`, `.filter_map`, `.filter`, `.find`, `.first`, `.flat_map`, `.get`, `.into_iter`, `.into_owned`, `.is_alphanumeric`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some`, `.is_whitespace`, `.iter`, `.last`, `.len`, `.map`, `.min`, `.name`, `.position`, `.push_str`, `.push`, `.remove`, `.rev`, `.saturating_add`, `.split`, `.starts_with`, `.then_some`, `.to_ascii_lowercase`, `.to_string`, `.unwrap_or`, `.unwrap`, `.windows`, `Some`, `String::new`, `Vec::new`, `W::del_text`, `W::del`, `W::ins`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `W::t`
- **Macros:** `matches!`
- **Called by:** `fold_short_label_ins_into_del_with_settings` (comparer/finalize.rs:7053); `mesh_short_label_shared_eq` (comparer/finalize.rs:7194); `short_label_format_history_tests::configured_label_tracking_preserves_sources_and_respects_disabled_changes` (comparer/finalize.rs:24148); `demo_title_pilcrow_history_tests::sharing_a_label_with_nested_effects_obeys_disabled_format_tracking` (comparer/finalize.rs:28751); `finalizer_distinct_boundary_contract_tests::equal_shared_labels_cannot_cross_opaque_or_historical_run_ownership` (comparer/finalize.rs:30001); `fragmented_source_label_boundary_tests::fragmented_labels_keep_literal_source_chars_formats_and_revision_history` (comparer/finalize.rs:33524); `fragmented_source_label_boundary_tests::label_format_boundaries_and_disabled_history_leave_complete_original_sources_intact` (comparer/finalize.rs:33678)

#### `mesh_short_label_shared_eq_with_settings::{fn label_run}`  (L7317-7338)
`fn label_run(dom: &Dom, leaves: &[NodeId], mut bytes: usize) -> Option<NodeId>`

- **Does:** A shared label may span text leaves, but must not cross a property boundary on either source side. _(from a code comment)_
- **Calls (crate):** `Dom::value_str` (xmllinq/mod.rs:1179) L7320; `Dom::parent` (xmllinq/mod.rs:597) L7324; `rpr_string` (comparer/finalize.rs:1048) L7326
- **Calls (std/external/unresolved):** `.len`, `.saturating_sub`, `Some`
- **Called by:** `mesh_short_label_shared_eq_with_settings` (comparer/finalize.rs:7199)

#### `trailing_alnum_token`  (L7583-7597)
`fn trailing_alnum_token(text: &str) -> Option<String>`

- **Does:** Last alphanumeric token of `text` with original casing (for EQ peel).
- **Calls (std/external/unresolved):** `.chars`, `.is_alphanumeric`, `.is_empty`, `.push`, `Some`, `String::new`, `std::mem::take`
- **Called by:** `free_mesh_shared_title_token_in_mix` (comparer/finalize.rs:2531); `mesh_short_label_shared_eq_with_settings` (comparer/finalize.rs:7199); `strip_trailing_alnum_token` (comparer/finalize.rs:7601); `coverage_tests::token_splitting_preserves_separators_and_original_case` (comparer/finalize.rs:15722)

#### `strip_trailing_alnum_token`  (L7601-7619)
`fn strip_trailing_alnum_token(text: &str, label: &str) -> Option<String>`

- **Does:** If `text` ends with alnum token matching `label` (case-insensitive), return the prefix with that token removed (separators before it kept).
- **Calls (crate):** `trailing_alnum_token` (comparer/finalize.rs:7583) L7602
- **Calls (std/external/unresolved):** `.char_indices`, `.collect`, `.eq_ignore_ascii_case`, `.is_alphanumeric`, `.len`, `.to_string`, `Some`
- **Called by:** `free_mesh_shared_title_token_in_mix` (comparer/finalize.rs:2531); `mesh_short_label_shared_eq_with_settings` (comparer/finalize.rs:7199); `coverage_tests::token_splitting_preserves_separators_and_original_case` (comparer/finalize.rs:15722)

#### `para_mark_revision`  (L7626-7630)
`fn para_mark_revision(dom: &Dom, p: NodeId, rev: &crate::xmllinq::XName) -> bool`

- **Does:** True when `w:pPr/w:rPr` carries a paragraph-mark revision (`w:del`/`w:ins`).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L7627,7628,7629
- **Calls (std/external/unresolved):** `.and_then`, `.is_some_and`, `.is_some`, `W::p_pr`, `W::r_pr`
- **Called by:** `unwrap_content_controls_in_pure_revisions` (comparer/finalize.rs:1412); `fold_short_list_label_into_empty_pure_del` (comparer/finalize.rs:1702); `strip_trailing_empty_pure_del_mark` (comparer/finalize.rs:2909); `merged_closing_carrier_takes_source_properties` (comparer/finalize.rs:3080); `story_final_kept_blank` (comparer/finalize.rs:3501); `para_is_empty_pure_ins` (comparer/finalize.rs:3542); `para_is_pure_inserted` (comparer/finalize.rs:4175); `rotate_ins_mark_del_only_paragraph` (comparer/finalize.rs:4576); `restamp_stranded_del_mark_onto_del_only_paragraph` (comparer/finalize.rs:4734); `split_head_short_title_long_del_mix` (comparer/finalize.rs:4829); `fold_leading_ins_from_mix_into_preceding_pure_del` (comparer/finalize.rs:4950); `para_is_mixed_revision` (comparer/finalize.rs:5117); +43 more

#### `fold_crosses_a_field`  (L7637-7657)
`fn fold_crosses_a_field(dom: &Dom, d: NodeId, carrier: NodeId, inss: &[NodeId]) -> bool`

- **Does:** Folding `d` into `carrier` appends d's body after the carrier's, above every inserted paragraph between them.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L7642,7656; `Dom::attribute` (xmllinq/mod.rs:874) L7643
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.iter`, `.skip_while`, `.skip`, `Some`, `W::name`
- **Called by:** `merge_replaced_in_container` (comparer/finalize.rs:8621); `coverage_tests::field_fold_rejects_crossing_but_allows_balanced_deleted_fields` (comparer/finalize.rs:16705)

#### `para_is_field_residue`  (L7664-7671)
`fn para_is_field_residue(dom: &Dom, p: NodeId) -> bool`

- **Does:** M474 — a "field residue" paragraph (M360's TOC field-end fragments) carries fldChar but NO visible text.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L7665,7668; `Dom::value_str` (xmllinq/mod.rs:1179) L7670
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.iter`, `.trim`, `Some`, `W::name`, `W::t`
- **Called by:** `fold_whitespace_pure_ins_into_following_pure_del` (comparer/finalize.rs:4211); `merge_replaced_in_container` (comparer/finalize.rs:8621); `coverage_tests::field_fold_rejects_crossing_but_allows_balanced_deleted_fields` (comparer/finalize.rs:16705)

#### `accumulate_para_child_class`  (L7678-7686)
`fn accumulate_para_child_class( dom: &Dom, c: NodeId, ins: &mut bool, del: &mut bool, plain: &mut bool, )`

- **Does:** Some(false) = fully deleted, Some(true) = fully inserted, None = neither.
- **Calls (crate):** `accumulate_para_child_class_with_moves` (comparer/finalize.rs:7688) L7685
- **Called by:** `para_replacement_class_with_moves` (comparer/finalize.rs:7742)

#### `accumulate_para_child_class_with_moves`  (L7688-7736)
`fn accumulate_para_child_class_with_moves( dom: &Dom, c: NodeId, ins: &mut bool, del: &mut bool, plain: &mut bool, include_moves: bool, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L7696; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L7702; `XName::local_name` (xmllinq/mod.rs:208) (by name) L7703; `Dom::elements` (xmllinq/mod.rs:640) L7730; `accumulate_para_child_class_with_moves` (comparer/finalize.rs:7688) L7731
- **Calls (std/external/unresolved):** `W::del`, `W::hyperlink`, `W::ins`, `W::name`, `W::p_pr`
- **Called by:** `accumulate_para_child_class` (comparer/finalize.rs:7678); `accumulate_para_child_class_with_moves` (comparer/finalize.rs:7688); `para_replacement_class_with_moves` (comparer/finalize.rs:7742)

#### `para_replacement_class`  (L7738-7740)
`fn para_replacement_class(dom: &Dom, p: NodeId) -> Option<bool>`

- **Does:** _no doc comment_
- **Calls (crate):** `para_replacement_class_with_moves` (comparer/finalize.rs:7742) L7739
- **Called by:** `reorder_replaced_blocks::{fn block_class}` (comparer/finalize.rs:10076); `splice_trailing_short_pure_dels_midstream` (comparer/finalize.rs:11798); `coverage_tests::replacement_classification_is_transparent_only_for_hyperlinks` (comparer/finalize.rs:15535); `coverage_tests::row_move_classification_requires_matching_structural_paragraph_marks` (comparer/finalize.rs:15607)

#### `para_replacement_class_with_moves`  (L7742-7781)
`fn para_replacement_class_with_moves(dom: &Dom, p: NodeId, include_moves: bool) -> Option<bool>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L7743; `Dom::elements` (xmllinq/mod.rs:640) L7747; `accumulate_para_child_class_with_moves` (comparer/finalize.rs:7688) L7749; `accumulate_para_child_class` (comparer/finalize.rs:7678) L7751; `para_mark_revision` (comparer/finalize.rs:7626) L7755,7757,7776; `Dom::descendants` (xmllinq/mod.rs:668) L7776
- **Calls (std/external/unresolved):** `.is_empty`, `Some`, `W::del`, `W::ins`, `W::name`, `W::p`
- **Called by:** `para_replacement_class` (comparer/finalize.rs:7738); `mark_fully_revised_rows` (comparer/finalize.rs:11666); `coverage_tests::row_move_classification_requires_matching_structural_paragraph_marks` (comparer/finalize.rs:15607)

#### `para_class_carried_aware`  (L7789-7844)
`fn para_class_carried_aware(dom: &Dom, p: NodeId, comparer_author: &str) -> Option<bool>`

- **Does:** Like [`para_replacement_class`], but a `w:ins` authored by someone OTHER than the comparer (a CARRIED pre-existing insertion, D1 machinery) counts toward the DELETED side: GT places doc-A blocks whose only live content is carried foreign-author ins inside the trailing deleted cluster (contract-review-suggesting-mixed-...
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L7790; `Dom::elements` (xmllinq/mod.rs:640) L7794,7811; `Dom::name` (xmllinq/mod.rs:526) L7795,7812; `Dom::attribute` (xmllinq/mod.rs:874) L7802,7817; `para_mark_revision` (comparer/finalize.rs:7626) L7833,7835
- **Calls (std/external/unresolved):** `.unwrap_or`, `Some`, `W::author`, `W::del`, `W::hyperlink`, `W::ins`, `W::name`, `W::p_pr`, `W::p`
- **Called by:** `merge_replaced_in_container` (comparer/finalize.rs:8621); `coverage_tests::replacement_classification_is_transparent_only_for_hyperlinks` (comparer/finalize.rs:15535)

#### `para_has_real_del`  (L7849-7851)
`fn para_has_real_del(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when the paragraph holds a REAL deletion (any `w:del` child) — a carried-only "deleted-side" paragraph must not 1v1-merge into a mixed paragraph (its content is pending ins, not replaced text).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L7850
- **Calls (std/external/unresolved):** `.is_empty`, `Some`, `W::del`
- **Called by:** `fold_short_list_label_into_empty_pure_del` (comparer/finalize.rs:1702); `promote_live_numpr_on_pure_d_from_pprchange` (comparer/finalize.rs:1866); `merge_replaced_in_container` (comparer/finalize.rs:8621)

#### `para_revision_body_text`  (L7854-7863)
`fn para_revision_body_text(dom: &Dom, p: NodeId) -> String`

- **Does:** Live + deleted body text of a paragraph (for relatedness gates).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L7857; `Dom::value_str` (xmllinq/mod.rs:1179) L7858
- **Calls (std/external/unresolved):** `.push_str`, `.push`, `Some`, `String::new`, `W::del_text`, `W::name`
- **Called by:** `promote_live_numpr_on_pure_d_from_pprchange` (comparer/finalize.rs:1866); `ensure_pure_i_list_snug_spacing` (comparer/finalize.rs:2071); `para_has_repeated_phrase` (comparer/finalize.rs:3524); `relocate_title_page_last_empty_after_pure_dels` (comparer/finalize.rs:3612); `fold_leading_ins_from_mix_into_preceding_pure_del` (comparer/finalize.rs:4950); `park_mixed_spacing_onto_trailing_pure_del` (comparer/finalize.rs:5602); `residual_short_label_zip_with_settings` (comparer/finalize.rs:6925); `para_body_is_digits_only` (comparer/finalize.rs:7874); `para_body_alnum_len` (comparer/finalize.rs:7884); `para_looks_like_demo_title` (comparer/finalize.rs:7901); `should_fold_ins_del_pair` (comparer/finalize.rs:7997); `para_word_atom_count` (comparer/finalize.rs:8008); +13 more

#### `body_token_set`  (L7865-7870)
`fn body_token_set(text: &str) -> std::collections::HashSet<String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.is_alphanumeric`, `.is_empty`, `.map`, `.split`, `.to_ascii_lowercase`
- **Called by:** `park_mixed_spacing_onto_trailing_pure_del` (comparer/finalize.rs:5602); `body_text_jaccard` (comparer/finalize.rs:7966); `should_fold_multi_del_at_document_scale` (comparer/finalize.rs:8020); `merge_replaced_in_container` (comparer/finalize.rs:8621); `deep_production_path_tests::punctuation_only_items_are_contentful_and_can_supply_empty_token_relatedness` (comparer/finalize.rs:23344)

#### `para_body_is_digits_only`  (L7874-7881)
`fn para_body_is_digits_only(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when paragraph body text (live + delText) is only digits/whitespace (file_166 pure-I "24" — corpus fragment, not content fold bait).
- **Calls (crate):** `para_revision_body_text` (comparer/finalize.rs:7854) L7875
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.is_ascii_digit`, `.is_empty`, `.is_whitespace`, `.trim`
- **Called by:** `merge_replaced_in_container` (comparer/finalize.rs:8621); `coverage_tests::relatedness_and_short_title_gates_use_tokens_not_substrings` (comparer/finalize.rs:15763)

#### `para_body_alnum_len`  (L7884-7889)
`fn para_body_alnum_len(dom: &Dom, p: NodeId) -> usize`

- **Does:** Alphanumeric character count of para body text (revision-aware).
- **Calls (crate):** `para_revision_body_text` (comparer/finalize.rs:7854) L7885
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.filter`, `.is_alphanumeric`
- **Called by:** `ensure_empty_pure_i_before_short_title_del` (comparer/finalize.rs:3910); `strip_empty_pure_ins_before_trailing_pure_dels` (comparer/finalize.rs:4062); `fold_whitespace_pure_ins_into_following_pure_del` (comparer/finalize.rs:4211); `park_mixed_spacing_onto_trailing_pure_del` (comparer/finalize.rs:5602); `residual_short_label_zip_with_settings` (comparer/finalize.rs:6925); `para_body_is_very_short` (comparer/finalize.rs:7894); `merge_replaced_in_container` (comparer/finalize.rs:8621); `splice_trailing_short_pure_dels_midstream` (comparer/finalize.rs:11798); `restore_short_del_before_long_ins` (comparer/finalize.rs:12180)

#### `para_body_is_very_short`  (L7894-7896)
`fn para_body_is_very_short(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when body alphanumerics are 1–2 chars ("a", "x", "OK").
- **Calls (crate):** `para_body_alnum_len` (comparer/finalize.rs:7884) L7895
- **Calls (std/external/unresolved):** `.contains`
- **Called by:** `residual_short_label_zip_with_settings` (comparer/finalize.rs:6925); `merge_replaced_in_container` (comparer/finalize.rs:8621); `coverage_tests::relatedness_and_short_title_gates_use_tokens_not_substrings` (comparer/finalize.rs:15763)

#### `para_looks_like_demo_title`  (L7901-7915)
`fn para_looks_like_demo_title(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when body looks like a short demo title (≤8 tokens, last significant token is `Demo`).
- **Calls (crate):** `para_revision_body_text` (comparer/finalize.rs:7854) L7902
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.count`, `.eq_ignore_ascii_case`, `.filter`, `.find`, `.is_alphanumeric`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.rev`, `.split`
- **Called by:** `ensure_empty_pure_i_before_short_title_del` (comparer/finalize.rs:3910); `strip_empty_pure_ins_before_trailing_pure_dels` (comparer/finalize.rs:4062); `fold_whitespace_pure_ins_into_following_pure_del` (comparer/finalize.rs:4211); `merge_replaced_in_container` (comparer/finalize.rs:8621); `capture_deleted_demo_title_properties` (comparer/finalize.rs:12579); `fold_midstream_demo_title_into_numbered_heading_with_source_properties` (comparer/finalize.rs:12619); `coverage_tests::relatedness_and_short_title_gates_use_tokens_not_substrings` (comparer/finalize.rs:15763)

#### `para_has_heading_or_title_style`  (L7918-7928)
`fn para_has_heading_or_title_style(dom: &Dom, p: NodeId) -> bool`

- **Does:** Live `w:pStyle` is Title / Heading* (Heading1, Heading2, …).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L7919,7922; `Dom::attribute` (xmllinq/mod.rs:874) L7925
- **Calls (std/external/unresolved):** `.starts_with`, `.to_ascii_lowercase`, `.unwrap_or`, `W::p_pr`, `W::p_style`, `W::val`
- **Called by:** `ensure_empty_pure_i_before_short_title_del` (comparer/finalize.rs:3910); `strip_empty_pure_ins_before_trailing_pure_dels` (comparer/finalize.rs:4062); `fold_whitespace_pure_ins_into_following_pure_del` (comparer/finalize.rs:4211); `should_fold_multi_del_at_document_scale` (comparer/finalize.rs:8020); `merge_replaced_in_container` (comparer/finalize.rs:8621); `deep_production_path_tests::long_content_cannot_reach_back_across_an_empty_tail_without_a_head_match` (comparer/finalize.rs:21141); `deep_production_path_tests::deletion_first_short_residual_and_field_carrier_keep_source_properties` (comparer/finalize.rs:21279); `deep_production_path_tests::heading_skip_is_limited_to_short_multitoken_headings_and_nonlist_carriers` (comparer/finalize.rs:21436)

#### `para_has_heading_style`  (L7931-7940)
`fn para_has_heading_style(dom: &Dom, p: NodeId) -> bool`

- **Does:** Live `w:pStyle` is Heading* only (not Title).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L7932,7935; `Dom::attribute` (xmllinq/mod.rs:874) L7938
- **Calls (std/external/unresolved):** `.starts_with`, `.to_ascii_lowercase`, `.unwrap_or`, `W::p_pr`, `W::p_style`, `W::val`
- **Called by:** `merge_replaced_in_container` (comparer/finalize.rs:8621); `deep_production_path_tests::styled_inserted_titles_keep_unrelated_short_base_titles_except_demos` (comparer/finalize.rs:21391)

#### `short_title_shares_sig_token`  (L7944-7960)
`fn short_title_shares_sig_token(ins_text: &str, del_text: &str) -> bool`

- **Does:** Shared significant token (len≥4) between short-title pure-D and first pure-I (M322 head-junction).
- **Calls (crate):** `memoized_moves_tests::sig` (comparer/moves.rs:882) (by name) L7957,7958
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.filter`, `.intersection`, `.is_alphanumeric`, `.is_empty`, `.is_some`, `.iter`, `.len`, `.map`, `.next`, `.split`, `.to_ascii_lowercase`
- **Called by:** `merge_replaced_in_container` (comparer/finalize.rs:8621); `coverage_tests::relatedness_and_short_title_gates_use_tokens_not_substrings` (comparer/finalize.rs:15763)

#### `body_text_jaccard`  (L7966-7975)
`fn body_text_jaccard(a: &str, b: &str) -> f64`

- **Does:** Token Jaccard of two body strings.
- **Calls (crate):** `body_token_set` (comparer/finalize.rs:7865) L7967,7968
- **Calls (std/external/unresolved):** `.count`, `.intersection`, `.is_empty`, `.union`
- **Called by:** `should_fold_ins_del_pair` (comparer/finalize.rs:7997); `coverage_tests::relatedness_and_short_title_gates_use_tokens_not_substrings` (comparer/finalize.rs:15763); `deep_production_path_tests::midstory_checklist_cell_guard_distinguishes_titles_at_a_weak_head_junction` (comparer/finalize.rs:21577)

#### `para_has_live_numpr`  (L7990-7995)
`fn para_has_live_numpr(dom: &Dom, p: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L7991,7994
- **Calls (std/external/unresolved):** `.is_some`, `W::num_pr`, `W::p_pr`
- **Called by:** `fold_short_list_label_into_empty_pure_del` (comparer/finalize.rs:1702); `promote_live_numpr_on_pure_d_from_pprchange` (comparer/finalize.rs:1866); `should_fold_multi_del_at_document_scale` (comparer/finalize.rs:8020); `merge_replaced_in_container` (comparer/finalize.rs:8621); `deep_production_path_tests::prior_nested_cluster_keeps_short_labels_but_allows_a_longer_carrier` (comparer/finalize.rs:21108); `deep_production_path_tests::list_carriers_override_the_long_prose_list_skip_and_retain_their_geometry` (comparer/finalize.rs:21482)

#### `should_fold_ins_del_pair`  (L7997-8005)
`fn should_fold_ins_del_pair(dom: &Dom, ins_p: NodeId, del_p: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `para_revision_body_text` (comparer/finalize.rs:7854) L7998,7999; `body_text_jaccard` (comparer/finalize.rs:7966) L8004
- **Calls (std/external/unresolved):** `.is_empty`, `.trim`
- **Called by:** `should_fold_multi_del_at_document_scale` (comparer/finalize.rs:8020); `merge_replaced_in_container` (comparer/finalize.rs:8621); `coverage_tests::sole_pair_relatedness_boundary_is_inclusive` (comparer/finalize.rs:16210); `deep_production_path_tests::punctuation_only_items_are_contentful_and_can_supply_empty_token_relatedness` (comparer/finalize.rs:23344)

#### `para_word_atom_count`  (L8008-8011)
`fn para_word_atom_count(dom: &Dom, p: NodeId) -> usize`

- **Does:** Word-atom count for body text under a paragraph (whitespace-split tokens).
- **Calls (crate):** `para_revision_body_text` (comparer/finalize.rs:7854) L8009
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.is_empty`, `.split_whitespace`
- **Called by:** `fold_short_list_label_into_empty_pure_del` (comparer/finalize.rs:1702); `promote_live_numpr_on_pure_d_from_pprchange` (comparer/finalize.rs:1866); `ensure_pure_i_list_snug_spacing` (comparer/finalize.rs:2071); `strip_redundant_demo_default_spacing` (comparer/finalize.rs:2266); `ensure_empty_pure_i_before_short_title_del` (comparer/finalize.rs:3910); `strip_empty_pure_ins_before_trailing_pure_dels` (comparer/finalize.rs:4062); `fold_whitespace_pure_ins_into_following_pure_del` (comparer/finalize.rs:4211); `should_fold_multi_del_at_document_scale` (comparer/finalize.rs:8020); `merge_replaced_in_container` (comparer/finalize.rs:8621); `interleave_list_cluster_after_coalesce` (comparer/finalize.rs:9876)

#### `should_fold_multi_del_at_document_scale`  (L8020-8338)
`fn should_fold_multi_del_at_document_scale( dom: &Dom, container: NodeId, last_ins: NodeId, first_del: NodeId, inss: &[NodeId], dels: &[NodeId], ) -> bool`

- **Does:** Document-scale gate for multi-del boundary fold (plan B1 / C1).
- **Calls (crate):** `para_revision_body_text` (comparer/finalize.rs:7854) L8032,8067,8076,8079,8108,8113...; `should_fold_ins_del_pair` (comparer/finalize.rs:7997) L8033,8215,8250,8267,8277,8312; `Dom::element` (xmllinq/mod.rs:655) L8041,8042; `Dom::attribute` (xmllinq/mod.rs:874) L8043; `para_has_heading_or_title_style` (comparer/finalize.rs:7918) L8050; `para_word_atom_count` (comparer/finalize.rs:8008) L8051,8069,8072,8156,8160,8164...; `para_has_live_numpr` (comparer/finalize.rs:7990) L8083,8084,8151,8172; `body_token_set` (comparer/finalize.rs:7865) L8123,8127; `para_has_repeated_phrase` (comparer/finalize.rs:3524) L8266; `Dom::elements` (xmllinq/mod.rs:640) L8329; `Dom::name_is` (xmllinq/mod.rs:535) L8332
- **Calls (std/external/unresolved):** `.all`, `.any`, `.chars`, `.collect`, `.contains`, `.copied`, `.count`, `.filter`, `.intersection`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.map`, `.max`, `.min`, `.next`, `.saturating_mul`, `.split_whitespace`, `.starts_with`, `.sum`, `.to_ascii_lowercase`, `.trim`, `.union`, `.unwrap_or`, `W::p_pr`, `W::p_style`, `W::p`, `W::val`, `all_short`, `list_frac`
- **Called by:** `merge_replaced_in_container` (comparer/finalize.rs:8621); `coverage_tests::fold_gate` (comparer/finalize.rs:15958); `coverage_tests::document_scale_list_and_cover_special_cases` (comparer/finalize.rs:16053); `deep_production_path_tests::numbered_wholesale_gate_requires_the_last_new_item_and_half_of_old_items_numbered` (comparer/finalize.rs:23307); `deep_production_path_tests::punctuation_only_items_are_contentful_and_can_supply_empty_token_relatedness` (comparer/finalize.rs:23344); `finalizer_remaining_source_boundary_tests::document_scale_folds_preserve_authored_sources_and_observed_word_exceptions` (comparer/finalize.rs:30780); `reached_word_document_policy_tests::document_policy_reaches_numbering_relatedness_and_demo_intro_boundaries` (comparer/finalize.rs:34228)

#### `gap_ends_story`  (L8341-8343)
`fn gap_ends_story(dom: &Dom, rest: &[NodeId]) -> bool`

- **Does:** Only the section properties follow a replace gap: it ends the story.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L8342
- **Calls (std/external/unresolved):** `.all`, `.iter`, `W::sect_pr`
- **Called by:** `merge_replaced_in_container` (comparer/finalize.rs:8621)

#### `gap_precedes_live_story_tail`  (L8355-8379)
`fn gap_precedes_live_story_tail( dom: &Dom, rest: &[NodeId], classes: &[Option<bool>], fold_boundary: bool, ) -> bool`

- **Does:** A replace gap followed by deleted blocks and then the story's final paragraph, holding nothing live or inserted (empty, or the original's deleted last words; its mark is the story's final pilcrow, which Word never deletes, so a later pass restores it), while the final marks are no empty pair (`fold_boundary`): the rev...
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L8362,8369; `para_mark_revision` (comparer/finalize.rs:7626) L8370; `Dom::descendants` (xmllinq/mod.rs:668) L8371,8372; `Dom::value_str` (xmllinq/mod.rs:1179) L8375; `table_is_deleted` (comparer/finalize.rs:8416) L8378
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.filter`, `.is_empty`, `.iter`, `.len`, `.split_last`, `.trim`, `Some`, `W::ins`, `W::p`, `W::sect_pr`, `W::t`
- **Called by:** `merge_replaced_in_container` (comparer/finalize.rs:8621)

#### `para_carries_page_break`  (L8384-8391)
`fn para_carries_page_break(dom: &Dom, p: NodeId) -> bool`

- **Does:** A page break keeps its paragraph whole: Word never fuses it, so the break keeps paginating.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L8385,8386; `Dom::descendants` (xmllinq/mod.rs:668) L8387; `Dom::attribute` (xmllinq/mod.rs:874) L8390
- **Calls (std/external/unresolved):** `.any`, `.is_some_and`, `.is_some`, `.iter`, `Some`, `W::name`, `W::p_pr`
- **Called by:** `merge_replaced_in_container` (comparer/finalize.rs:8621); `coverage_tests::field_fold_rejects_crossing_but_allows_balanced_deleted_fields` (comparer/finalize.rs:16705); `deep_production_path_tests::explicit_page_breaks_block_final_story_fusion_on_either_source_side` (comparer/finalize.rs:23266)

#### `fuse_story_tail_into_deleted`  (L8395-8411)
`fn fuse_story_tail_into_deleted(dom: &mut Dom, last_ins: NodeId, first_del: NodeId)`

- **Does:** Word's story-tail fusion: the revised last paragraph's runs open the first deleted paragraph, which keeps its own properties and deleted mark.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L8396,8400; `Dom::name_is` (xmllinq/mod.rs:535) L8399,8401; `Dom::remove` (xmllinq/mod.rs:1102) L8404,8410; `Dom::add_before_self` (xmllinq/mod.rs:1114) L8406; `Dom::add` (xmllinq/mod.rs:1058) L8407
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `W::p_pr`
- **Called by:** `merge_replaced_in_container` (comparer/finalize.rs:8621)

#### `table_is_deleted`  (L8416-8426)
`fn table_is_deleted(dom: &Dom, tbl: NodeId) -> bool`

- **Does:** Every paragraph of the table is deleted, marks included, and nothing in it is live or inserted (the rows get their deleted marks later, from `mark_fully_revised_rows`).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L8417,8421,8422; `Dom::name_is` (xmllinq/mod.rs:535) L8418; `para_mark_revision` (comparer/finalize.rs:7626) L8420; `Dom::value_str` (xmllinq/mod.rs:1179) L8425
- **Calls (std/external/unresolved):** `.all`, `.is_empty`, `.iter`, `.trim`, `Some`, `W::del`, `W::ins`, `W::p`, `W::t`, `W::tbl`
- **Called by:** `gap_precedes_live_story_tail` (comparer/finalize.rs:8355)

#### `paragraph_has_live_empty_pilcrow`  (L8431-8462)
`fn paragraph_has_live_empty_pilcrow(dom: &Dom, p: NodeId) -> bool`

- **Does:** A live empty pilcrow, optionally carrying structurally inert runs.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L8432,8440,8443,8447,8450; `para_mark_revision` (comparer/finalize.rs:7626) L8435; `Dom::elements` (xmllinq/mod.rs:640) L8439,8446,8451; `Dom::value_str` (xmllinq/mod.rs:1179) L8448; `Dom::name` (xmllinq/mod.rs:526) L8452; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L8453; `XName::local_name` (xmllinq/mod.rs:208) (by name) L8455; `Dom::nodes` (xmllinq/mod.rs:604) L8458
- **Calls (std/external/unresolved):** `.all`, `.any`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `W::del`, `W::ins`, `W::move_from`, `W::move_to`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `W::t`
- **Macros:** `matches!`
- **Called by:** `gap_has_live_blank_cell_tail` (comparer/finalize.rs:8469); `story_tail_has_paired_original_boundary` (comparer/finalize.rs:8606); `merge_replaced_in_container` (comparer/finalize.rs:8621); `coverage_real_revision_merge_matrix_tests::multiple_original_paragraphs_before_a_live_blank_control_tail_keep_exact_phase_payloads` (comparer/finalize.rs:25406); `coverage_real_revision_merge_matrix_tests::multiple_revised_paragraphs_with_distinct_tail_properties_keep_word_deleted_carrier` (comparer/finalize.rs:25538); `coverage_real_revision_merge_matrix_tests::multiple_original_blank_tail_checks_every_later_pilcrow_provenance` (comparer/finalize.rs:25645); `coverage_real_revision_merge_matrix_tests::multiple_original_blank_tail_preserves_first_original_section_and_property_history` (comparer/finalize.rs:25780); `coverage_real_revision_merge_matrix_tests::multiple_revised_blank_tail_pairs_only_the_matching_final_revised_paragraph` (comparer/finalize.rs:26001); `coverage_real_revision_merge_matrix_tests::empty_pilcrow_proof_retains_inert_formatting_and_declines_owned_payloads` (comparer/finalize.rs:26443)

#### `gap_has_live_blank_cell_tail`  (L8469-8475)
`fn gap_has_live_blank_cell_tail(dom: &Dom, container: NodeId, rest: &[NodeId]) -> bool`

- **Does:** An explicit live empty cell paragraph owns a separate final pilcrow on both sources.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L8470; `paragraph_has_live_empty_pilcrow` (comparer/finalize.rs:8431) L8474
- **Calls (std/external/unresolved):** `.all`, `.is_empty`, `.iter`, `W::tc`
- **Called by:** `merge_replaced_in_container` (comparer/finalize.rs:8621)

#### `paired_replacement_marks_belong_to_comparer`  (L8480-8517)
`fn paired_replacement_marks_belong_to_comparer( dom: &Dom, deleted: NodeId, inserted: NodeId, comparer_author: &str, ) -> bool`

- **Does:** Only comparison-owned pilcrows may become an unmarked paired boundary.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L8492,8495; `Dom::elements` (xmllinq/mod.rs:640) L8498,8499; `Dom::attribute` (xmllinq/mod.rs:874) L8503,8513,8514; `Dom::descendants` (xmllinq/mod.rs:668) L8506; `Dom::ancestors` (xmllinq/mod.rs:770) L8509
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.len`, `.unwrap_or`, `Some`, `W::author`, `W::date`, `W::del`, `W::ins`, `W::p_pr`, `W::r_pr`
- **Called by:** `story_tail_has_paired_original_boundary` (comparer/finalize.rs:8606); `merge_replaced_in_container` (comparer/finalize.rs:8621); `finalizer_remaining_source_boundary_tests::paired_lifetimes_require_complete_matching_comparison_provenance` (comparer/finalize.rs:31108)

#### `paired_replacement_has_identical_properties`  (L8524-8600)
`fn paired_replacement_has_identical_properties( dom: &mut Dom, deleted: NodeId, inserted: NodeId, ) -> bool`

- **Does:** A related 1v1 pair with identical complete paragraph properties owns a single live pilcrow in Word's mixed paragraph.
- **Calls (crate):** `para_mark_revision` (comparer/finalize.rs:7626) L8529; `Dom::element` (xmllinq/mod.rs:655) L8534,8535,8540,8541,8545,8565; `Dom::elements` (xmllinq/mod.rs:640) L8546,8567,8571; `Dom::name_is` (xmllinq/mod.rs:535) L8549; `Dom::new` (xmllinq/mod.rs:387) L8557; `Dom::serialize_element` (xmllinq/mod.rs:1299) L8559,8596; `Dom::with_scratch` (xmllinq/mod.rs:449) L8560; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L8561; `Dom::root` (xmllinq/mod.rs:822) L8562; `Dom::remove` (xmllinq/mod.rs:1102) L8568,8572; `remove_powertools_scratch_markup` (comparer/finalize.rs:1018) L8575; `Dom::attributes` (xmllinq/mod.rs:885) L8579,8585; `Dom::is_namespace_declaration` (xmllinq/mod.rs:996) L8583; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L8594
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.expect`, `.filter`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.map`, `Some`, `W::del`, `W::ins`, `W::move_from`, `W::move_to`, `W::p_pr_change`, `W::p_pr`, `W::r_pr_change`, `W::r_pr`, `W::sect_pr`
- **Called by:** `story_tail_has_paired_original_boundary` (comparer/finalize.rs:8606); `merge_replaced_in_container` (comparer/finalize.rs:8621); `coverage_real_revision_merge_matrix_tests::multiple_original_blank_tail_preserves_first_original_section_and_property_history` (comparer/finalize.rs:25780); `coverage_real_revision_merge_matrix_tests::paired_property_comparison_never_grows_the_persistent_dom_or_changes_sources` (comparer/finalize.rs:26393); `finalizer_remaining_source_boundary_tests::paired_format_equality_is_complete_and_uses_no_production_arena_nodes` (comparer/finalize.rs:31191)

#### `story_tail_has_paired_original_boundary`  (L8606-8619)
`fn story_tail_has_paired_original_boundary( dom: &mut Dom, deleted: &[NodeId], inserted: NodeId, rest: &[NodeId], comparer_author: &str, ) -> bool`

- **Does:** A shared blank cannot supply the pilcrow for revised text whose source had its own paragraph properties.
- **Calls (crate):** `paragraph_has_live_empty_pilcrow` (comparer/finalize.rs:8431) L8613; `paired_replacement_marks_belong_to_comparer` (comparer/finalize.rs:8480) L8616; `paired_replacement_has_identical_properties` (comparer/finalize.rs:8524) L8618
- **Calls (std/external/unresolved):** `.all`, `.is_empty`, `.iter`
- **Macros:** `matches!`
- **Called by:** `merge_replaced_in_container` (comparer/finalize.rs:8621); `coverage_real_revision_merge_matrix_tests::multiple_original_blank_tail_checks_every_later_pilcrow_provenance` (comparer/finalize.rs:25645); `coverage_real_revision_merge_matrix_tests::multiple_original_blank_tail_preserves_first_original_section_and_property_history` (comparer/finalize.rs:25780)

#### `merge_replaced_in_container`  (L8621-9855)
`fn merge_replaced_in_container( dom: &mut Dom, container: NodeId, comparer_author: &str, fold_boundary: bool, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L8628,8702,8705,8821,8837,8842...; `para_class_carried_aware` (comparer/finalize.rs:7789) L8631; `gap_has_live_blank_cell_tail` (comparer/finalize.rs:8469) L8654,8973; `para_has_real_del` (comparer/finalize.rs:7849) L8673,8712,8756,9018,9023,9060; `gap_precedes_live_story_tail` (comparer/finalize.rs:8355) L8674,8977; `para_has_no_text` (comparer/finalize.rs:3459) L8675,8978,9020,9025,9044,9117...; `para_mark_revision` (comparer/finalize.rs:7626) L8676,8909,8979,9019,9024,9030...; `para_carries_page_break` (comparer/finalize.rs:8384) L8677,8678,8980,8981; `Dom::element` (xmllinq/mod.rs:655) L8679,8681,8698,8700,8723,8724...; `Dom::remove` (xmllinq/mod.rs:1102) L8684,8703,8706,8765,8817,8834...; `Dom::add_before_self` (xmllinq/mod.rs:1114) L8685,8766,8822,8860,9799,9820; `paragraph_has_live_empty_pilcrow` (comparer/finalize.rs:8431) L8688; `paired_replacement_marks_belong_to_comparer` (comparer/finalize.rs:8480) L8689,8910; `paired_replacement_has_identical_properties` (comparer/finalize.rs:8524) L8695,8911; `fuse_story_tail_into_deleted` (comparer/finalize.rs:8395) L8696,8993; `Dom::attribute` (xmllinq/mod.rs:874) L8726,8884,9050,9108,9620,9630...; `para_revision_body_text` (comparer/finalize.rs:7854) L8735,8797,9184,9185,9208,9209...; `Dom::parent` (xmllinq/mod.rs:597) L8762,8794,8795,8858; `gap_ends_story` (comparer/finalize.rs:8341) L8791,9228; `body_token_set` (comparer/finalize.rs:7865) L8797,9492,9493,9519,9520,9573...; `para_body_alnum_len` (comparer/finalize.rs:7884) L8799,9548,9570,9585,9595,9616...; `should_fold_ins_del_pair` (comparer/finalize.rs:7997) L8801,8857,9153,9175,9229,9274...; `para_is_field_residue` (comparer/finalize.rs:7664) L8813,8881,9783; `para_has_heading_or_title_style` (comparer/finalize.rs:7918) L8814,9058,9509,9593,9689,9750; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L8820,8893,8898,8902,9797,9809; `Dom::add` (xmllinq/mod.rs:1058) L8824,8844,8894,8899,8903,8925...; `Dom::name_is` (xmllinq/mod.rs:535) L8843,8906,8924,8929,9011,9563...; `Dom::new_element` (xmllinq/mod.rs:478) L8865; `Dom::attributes` (xmllinq/mod.rs:885) L8866; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L8867; `ppr_has_structural_props` (comparer/finalize.rs:531) L8874,8877,9724,9727; `Dom::replace_with` (xmllinq/mod.rs:1149) L8933; `story_tail_has_paired_original_boundary` (comparer/finalize.rs:8606) L8986; `Dom::descendants` (xmllinq/mod.rs:668) L9011,9364,9365,9366,9367,9407...; `para_word_atom_count` (comparer/finalize.rs:8008) L9118,9154,9155,9156,9176,9177...; `para_has_live_numpr` (comparer/finalize.rs:7990) L9119,9617,9627,9762,9775,9776; `para_body_text_is_whitespace_only` (comparer/finalize.rs:4159) L9152,9174,9244,9276,9406; `short_title_shares_sig_token` (comparer/finalize.rs:7944) L9186,9210; `fold_crosses_a_field` (comparer/finalize.rs:7637) L9206,9215; `para_has_omath` (comparer/finalize.rs:3474) L9245; `Dom::name` (xmllinq/mod.rs:526) L9260; `para_has_repeated_phrase` (comparer/finalize.rs:3524) L9319; `para_is_visually_blank` (comparer/finalize.rs:3487) L9350,9354; `should_fold_multi_del_at_document_scale` (comparer/finalize.rs:8020) L9390,9397; `para_body_is_digits_only` (comparer/finalize.rs:7874) L9422,9594; `para_looks_like_demo_title` (comparer/finalize.rs:7901) L9516,9549,9586; `para_body_is_very_short` (comparer/finalize.rs:7894) L9534,9714; `para_is_pure_inserted` (comparer/finalize.rs:4175) L9564; `para_has_heading_style` (comparer/finalize.rs:7931) L9688,9716; `ppr_is_jc_only` (comparer/finalize.rs:549) L9734
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.chars`, `.collect`, `.contains`, `.copied`, `.count`, `.eq_ignore_ascii_case`, `.filter`, `.find`, `.first`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some_and`, `.is_some`, `.is_whitespace`, `.iter`, `.last`, `.len`, `.map`, `.ok`, `.parse`, `.rev`, `.split_last`, `.starts_with`, `.take_while`, `.to_ascii_lowercase`, `.to_vec`, `.trim`, `.unwrap_or`, `.zip`, `Some`, `W::del`, `W::drawing`, `W::ins`, `W::move_from`, `W::move_to`, `W::name`, `W::num_pr`, `W::p_pr`, `W::p_style`, `W::p`, `W::pict`, `W::r_pr`, `W::sect_pr`, `W::spacing_el`, `W::tbl`, `W::val`
- **Macros:** `matches!`
- **Called by:** `merge_replaced_paragraphs_in` (comparer/finalize.rs:6890); `deep_production_path_tests::merge` (comparer/finalize.rs:20974); `coverage_real_revision_merge_matrix_tests::exercise` (comparer/finalize.rs:24959); `coverage_real_revision_merge_matrix_tests::live_blank_cell_tail_keeps_independent_replacement_paragraph_owners` (comparer/finalize.rs:25314); `coverage_real_revision_merge_matrix_tests::a_related_replacement_before_live_content_uses_a_paired_unmarked_pilcrow` (comparer/finalize.rs:25355); `coverage_real_revision_merge_matrix_tests::multiple_original_paragraphs_before_a_live_blank_control_tail_keep_exact_phase_payloads` (comparer/finalize.rs:25406); `coverage_real_revision_merge_matrix_tests::multiple_revised_paragraphs_with_distinct_tail_properties_keep_word_deleted_carrier` (comparer/finalize.rs:25538); `coverage_real_revision_merge_matrix_tests::multiple_original_blank_tail_checks_every_later_pilcrow_provenance` (comparer/finalize.rs:25645); `coverage_real_revision_merge_matrix_tests::multiple_original_blank_tail_preserves_first_original_section_and_property_history` (comparer/finalize.rs:25780); `coverage_real_revision_merge_matrix_tests::multiple_revised_blank_tail_pairs_only_the_matching_final_revised_paragraph` (comparer/finalize.rs:26001); `coverage_real_revision_merge_matrix_tests::a_single_replacement_before_a_live_blank_control_tail_keeps_a_paired_pilcrow` (comparer/finalize.rs:26220); `coverage_real_revision_merge_matrix_tests::a_sole_original_and_multiple_revised_paragraphs_keep_live_blank_tail_ownership` (comparer/finalize.rs:26255); +9 more

#### `interleave_list_cluster_after_coalesce`  (L9876-10065, pub)
`pub fn interleave_list_cluster_after_coalesce(dom: &mut Dom, root: NodeId)`

- **Does:** Word-alignment mode: at a multi-block replacement region — a run of fully-deleted block elements immediately followed by a run of fully- inserted ones, where either run contains a NON-paragraph block (table/ sdt) — Word emits the INSERTED block first (ole-object_ooxml-style-link: Word page 1 is the new document's head...
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L9877,9953,9954,9955,10016,10018...; `Dom::elements` (xmllinq/mod.rs:640) L9880,10040; `Dom::name_is` (xmllinq/mod.rs:535) L9883,9891,10043; `para_is_pure_inserted` (comparer/finalize.rs:4175) L9894; `para_is_pure_deleted` (comparer/finalize.rs:2858) L9895; `para_has_no_text` (comparer/finalize.rs:3459) L9900,10011; `Dom::attribute` (xmllinq/mod.rs:874) L9956,10020; `para_word_atom_count` (comparer/finalize.rs:8008) L9968; `para_revision_body_text` (comparer/finalize.rs:7854) L9983; `Dom::remove` (xmllinq/mod.rs:1102) L10045; `Dom::add_before_self` (xmllinq/mod.rs:1114) L10058; `Dom::add` (xmllinq/mod.rs:1058) L10062
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.as_bytes`, `.clone`, `.collect`, `.enumerate`, `.extend`, `.filter`, `.find`, `.first`, `.into_iter`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.map`, `.ok`, `.parse`, `.push`, `.split_whitespace`, `.to_vec`, `.unwrap_or`, `Some`, `Vec::new`, `Vec::with_capacity`, `W::body`, `W::name`, `W::num_pr`, `W::p_pr`, `W::p`, `W::sect_pr`, `W::val`
- **Called by:** `coverage_round_three_tests::nested_list_interleave_preserves_exact_node_order_and_trailing_anchors` (comparer/finalize.rs:18973); `deep_production_path_tests::nested_interleave_keeps_mark_only_items_and_defaults_invalid_terminal_levels` (comparer/finalize.rs:23121); `deep_production_path_tests::nested_interleave_uniform_tokens_are_case_insensitive_and_late_mismatches_reorder` (comparer/finalize.rs:23164); `finalizer_distinct_boundary_contract_tests::list_cluster_reorders_only_the_observed_nested_word_family` (comparer/finalize.rs:30386); `interleave_terminal_source_ownership_tests::remains_source_owned` (comparer/finalize.rs:33122); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `reorder_replaced_blocks`  (L10075-10168, pub)
`pub fn reorder_replaced_blocks(dom: &mut Dom, root: NodeId)`

- **Does:** Word-alignment mode: at a multi-block replacement region — a run of fully-deleted block elements immediately followed by a run of fully- inserted ones, where either run contains a NON-paragraph block (table/ sdt) — Word emits the INSERTED block first (ole-object_ooxml-style-link: Word page 1 is the new document's head...
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L10123; `Dom::descendants` (xmllinq/mod.rs:668) L10126; `Dom::elements` (xmllinq/mod.rs:640) L10128; `reorder_replaced_blocks::{fn block_class}` (comparer/finalize.rs:10076) L10129; `Dom::name_is` (xmllinq/mod.rs:535) L10130; `Dom::remove` (xmllinq/mod.rs:1102) L10163; `Dom::add_before_self` (xmllinq/mod.rs:1114) L10164
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.extend`, `.iter`, `.len`, `.map`, `.push`, `Some`, `Vec::new`, `W::body`, `W::name`, `W::p`
- **Called by:** `coverage_round_next_tests::block_reordering_requires_uniform_revision_rows_and_nonempty_sdt_content` (comparer/finalize.rs:20663); `consecutive_block_splice_tests::adjacent_inserted_prefix_preserves_table_midsplice_and_both_complete_projections` (comparer/finalize.rs:28413); `consecutive_block_splice_tests::live_boundary_prevents_an_earlier_insertion_from_claiming_the_splice` (comparer/finalize.rs:28451); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `reorder_replaced_blocks::{fn block_class}`  (L10076-10118)
`fn block_class(dom: &Dom, el: NodeId) -> Option<bool>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L10077; `para_replacement_class` (comparer/finalize.rs:7738) L10079; `Dom::descendants` (xmllinq/mod.rs:668) L10082; `Dom::element` (xmllinq/mod.rs:655) L10088,10089,10091,10103; `Dom::elements` (xmllinq/mod.rs:640) L10104; `reorder_replaced_blocks::{fn block_class}` (comparer/finalize.rs:10076) L10110
- **Calls (std/external/unresolved):** `.get_or_insert`, `.is_empty`, `.is_some`, `Some`, `W::del`, `W::ins`, `W::name`, `W::p`, `W::sdt_content`, `W::sdt`, `W::tbl`
- **Called by:** `reorder_replaced_blocks` (comparer/finalize.rs:10075); `reorder_replaced_blocks::{fn block_class}` (comparer/finalize.rs:10076)

#### `drop_sectpr_from_deleted_marks`  (L10179-10204, pub)
`pub fn drop_sectpr_from_deleted_marks( dom: &mut Dom, root: NodeId, genuine: &std::collections::HashSet<String>, )`

- **Does:** Word-alignment mode: drop the embedded `pPr/sectPr` from a paragraph whose paragraph mark is `w:del`-marked.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L10185; `Dom::element` (xmllinq/mod.rs:655) L10186,10188; `Dom::elements` (xmllinq/mod.rs:640) L10192; `sectpr_identity` (comparer/finalize.rs:11604) L10198; `Dom::remove` (xmllinq/mod.rs:1102) L10201
- **Calls (std/external/unresolved):** `.contains`, `.is_some_and`, `.is_some`, `Some`, `W::del`, `W::p_pr`, `W::r_pr`, `W::sect_pr`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `flatten_tracked_insertions_stamped`  (L10215-10267, pub)
`pub fn flatten_tracked_insertions_stamped(dom: &mut Dom, body: NodeId)`

- **Does:** Word-alignment mode — flatten doc A's PRE-EXISTING tracked INSERTIONS with a stamp.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L10219,10240; `Dom::parent` (xmllinq/mod.rs:597) L10223; `Dom::attribute` (xmllinq/mod.rs:874) L10228,10229,10245; `Dom::nodes` (xmllinq/mod.rs:604) L10230; `Dom::name_is` (xmllinq/mod.rs:535) L10237,10253; `Dom::ancestors` (xmllinq/mod.rs:770) L10250; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L10257,10259,10262; `Dom::replace_with` (xmllinq/mod.rs:1149) L10265
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.flat_map`, `.is_none`, `.is_some`, `.iter`, `.map`, `.reverse`, `.to_string`, `PT::name`, `Some`, `W::author`, `W::date`, `W::del`, `W::ins`, `W::r`
- **Macros:** `vec!`
- **Called by:** `finalizer_distinct_boundary_contract_tests::preexisting_insertions_keep_innermost_provenance_across_real_wrappers` (comparer/finalize.rs:29765); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `convert_stamped_preins`  (L10282-10375, pub)
`pub fn convert_stamped_preins( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** Produce-side counterpart of [`flatten_tracked_insertions_stamped`]: every run stamped `pt:PreIns` re-emerges as a PENDING INSERTION with its original author — its `w:del` wrapper (the diff saw A-only text absent from B) renames/splits to `w:ins` and delText reverts to w:t; Equal runs get a fresh `w:ins`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L10289,10316,10319; `Dom::attribute` (xmllinq/mod.rs:874) L10292,10296,10299; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L10295,10302,10303,10306,10309,10336...; `Dom::set_name` (xmllinq/mod.rs:539) L10317,10320,10335; `Dom::parent` (xmllinq/mod.rs:597) L10323; `Dom::name` (xmllinq/mod.rs:526) L10326; `Dom::elements` (xmllinq/mod.rs:640) L10332; `rev_el` (comparer/finalize.rs:126) L10342,10367; `Dom::remove` (xmllinq/mod.rs:1102) L10345,10357,10363,10371; `Dom::add` (xmllinq/mod.rs:1058) L10346,10358,10372; `Dom::add_after_self` (xmllinq/mod.rs:1131) L10347,10360; `Dom::new_element` (xmllinq/mod.rs:478) L10350; `Dom::attributes` (xmllinq/mod.rs:885) L10351; `Dom::add_before_self` (xmllinq/mod.rs:1114) L10370
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.len`, `.map`, `.position`, `.to_string`, `.to_vec`, `.unwrap_or`, `PT::name`, `Some`, `W::author`, `W::date`, `W::del_text`, `W::del`, `W::id`, `W::ins`, `W::instr_text`, `W::name`, `W::r`, `W::t`, `restamp`, `restore_text_kinds`
- **Called by:** `coverage_round_two_tests::carried_insert_and_delete_split_preserves_neighbors_and_attribution` (comparer/finalize.rs:17891); `coverage_final_batch_finalize_tests::carried_revision_matrix_preserves_order_formatting_and_original_attribution` (comparer/finalize.rs:24252); `finalizer_distinct_boundary_contract_tests::a_pending_insertion_splits_deleted_siblings_without_stealing_their_payloads` (comparer/finalize.rs:29857); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `ensure_default_page_size`  (L10388-10421, pub)
`pub fn ensure_default_page_size(dom: &mut Dom, root: NodeId)`

- **Does:** Word-alignment mode (settings-gated): Word's Compare output always carries a page setup — inputs with no `sectPr`/`pgSz` are normalized to Word's default Letter geometry (`w:pgSz w=12240 h=15840`; evidence: the `_word_redline` corpus, e.g.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L10389,10393,10401; `Dom::new_element` (xmllinq/mod.rs:478) L10396,10402; `Dom::add` (xmllinq/mod.rs:1058) L10397,10418; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L10403,10404; `Dom::elements` (xmllinq/mod.rs:640) L10408; `Dom::name` (xmllinq/mod.rs:526) L10409; `XName::local_name` (xmllinq/mod.rs:208) (by name) L10411; `Dom::add_before_self` (xmllinq/mod.rs:1114) L10417
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.is_none`, `.is_some_and`, `Some`, `W::body`, `W::name`, `W::sect_pr`
- **Macros:** `matches!`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `insert_tblpr_child_in_order`  (L10449-10465)
`fn insert_tblpr_child_in_order(dom: &mut Dom, tblpr: NodeId, child: NodeId, rank: i32)`

- **Does:** Insert `child` among `tblpr`'s children so every later sibling outranks it.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L10450; `Dom::name` (xmllinq/mod.rs:526) L10451; `XName::local_name` (xmllinq/mod.rs:208) (by name) L10455; `Dom::add_before_self` (xmllinq/mod.rs:1114) L10462; `Dom::add` (xmllinq/mod.rs:1058) L10463
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.iter`, `.map_or`, `.map`, `.unwrap_or`
- **Called by:** `align_word_table_and_comment_chrome` (comparer/finalize.rs:6215); `synthesize_table_cell_margins` (comparer/finalize.rs:11249)

#### `wml_order_elements_per_standard`  (L10483-10718, pub)
`pub fn wml_order_elements_per_standard(dom: &mut Dom, root: NodeId)`

- **Does:** Faithful port of `WordprocessingMLUtil.WmlOrderElementsPerStandard` (PtOpenXmlUtil.cs:1440), called by the C# produce path (WmlComparer.cs :1893) and previously MISSING from the port: property-container children (pPr/rPr/tblPr/tcPr/tcBorders/tblBorders/pBdr) are stable-sorted into standard schema order (unknown names...
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L10665; `Dom::name` (xmllinq/mod.rs:526) L10666; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L10667; `XName::local_name` (xmllinq/mod.rs:208) (by name) L10670; `Dom::nodes` (xmllinq/mod.rs:604) L10672,10693; `Dom::is_element` (xmllinq/mod.rs:502) L10673,10694; `Dom::remove` (xmllinq/mod.rs:1102) L10674,10688,10695,10712; `Dom::elements` (xmllinq/mod.rs:640) L10677,10698; `wml_order_elements_per_standard::{fn rank}` (comparer/finalize.rs:10487) L10681; `Dom::add` (xmllinq/mod.rs:1058) L10689,10713; `Dom::name_is` (xmllinq/mod.rs:535) L10701,10705,10710
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.chain`, `.clone`, `.collect`, `.contains`, `.enumerate`, `.into_iter`, `.is_some_and`, `.is_sorted_by_key`, `.iter`, `.map`, `.partition`, `.position`, `.sort_by_key`, `.to_string`, `W::p_pr`, `W::r_pr`
- **Called by:** `enforce_part_schema_order` (comparer/finalize.rs:10736); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `wml_order_elements_per_standard::{fn rank}`  (L10487-10653)
`fn rank(dom: &Dom, container: &str, e: NodeId) -> i32`

- **Does:** Rank tables (PtOpenXmlUtil.cs:1273-1439). _(from a code comment)_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L10488; `XName::local_name` (xmllinq/mod.rs:208) (by name) L10489; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L10490
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.map_or`
- **Called by:** `wml_order_elements_per_standard` (comparer/finalize.rs:10483); `add_child_in_rank_order` (document_comparer.rs:2089) (?)

#### `enforce_part_schema_order`  (L10736-10748, pub)
`pub fn enforce_part_schema_order(dom: &mut Dom, root: NodeId)`

- **Does:** Schema order for a whole package part, run by the package validity sweep after every other pass has had its say — so a property appended late (a style merged from B, a `w:jc` added after the body's ordering pass) still lands where the schema wants it.
- **Calls (crate):** `wml_order_elements_per_standard` (comparer/finalize.rs:10483) L10737; `Dom::descendants` (xmllinq/mod.rs:668) L10739; `Dom::elements` (xmllinq/mod.rs:640) L10740,10741; `Dom::name` (xmllinq/mod.rs:526) L10742; `Dom::remove` (xmllinq/mod.rs:1102) L10743
- **Calls (std/external/unresolved):** `.contains`, `.is_some_and`, `Some`, `W::p_pr_change`, `W::p_pr`, `W::r_pr`, `W::sect_pr`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501)

#### `wrap_bare_del_text_runs`  (L10771-10801, pub)
`pub fn wrap_bare_del_text_runs( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** Wrap any run holding `w:delText` that is NOT directly inside a legal deletion container in a fresh `w:del` — Word shows the repair dialog on bare delText runs (THE residual strict01 trigger: `delete_text_in_opaque` / the deleted-run rebuild rename nested text-box `w:t`s to delText without wrapping the nested runs; Wor...
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L10779; `Dom::element` (xmllinq/mod.rs:655) L10783; `Dom::parent` (xmllinq/mod.rs:597) L10786; `Dom::name` (xmllinq/mod.rs:526) L10789; `rev_el` (comparer/finalize.rs:126) L10795; `Dom::add_before_self` (xmllinq/mod.rs:1114) L10796; `Dom::remove` (xmllinq/mod.rs:1102) L10797; `Dom::add` (xmllinq/mod.rs:1058) L10798; `delete_stories_of_deleted_text_boxes` (comparer/finalize.rs:10812) L10800
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.filter`, `.into_iter`, `.is_none`, `Some`, `W::del_text`, `W::del`, `W::name`, `W::r`
- **Called by:** `coverage_round_three_tests::deleted_textbox_story_marks_runs_and_pilcrows_in_schema_order` (comparer/finalize.rs:19020); `coverage_round_three_tests::nested_deleted_textboxes_use_their_own_anchor_and_preserve_existing_marks` (comparer/finalize.rs:19077); `coverage_round_three_tests::deleted_textbox_missing_attribution_uses_pinned_settings` (comparer/finalize.rs:19112); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `delete_stories_of_deleted_text_boxes`  (L10812-10899)
`fn delete_stories_of_deleted_text_boxes( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** A text box's `w:txbxContent` is a story of its own: the `w:del` around its anchor run does not reach into it.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L10823,10828,10835,10840; `Dom::name` (xmllinq/mod.rs:526) L10825,10837; `Dom::descendants` (xmllinq/mod.rs:668) L10845,10850,10885,10888,10892; `Dom::nodes_before_self` (xmllinq/mod.rs:846) L10857; `Dom::is_element` (xmllinq/mod.rs:502) L10861; `Dom::name_is` (xmllinq/mod.rs:535) L10865; `Dom::attribute` (xmllinq/mod.rs:874) L10866,10873; `rev_el` (comparer/finalize.rs:126) L10871; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L10874,10877,10886; `Dom::add_before_self` (xmllinq/mod.rs:1114) L10878; `Dom::remove` (xmllinq/mod.rs:1102) L10882; `Dom::add` (xmllinq/mod.rs:1058) L10883; `Dom::ancestors` (xmllinq/mod.rs:770) L10893; `mark_paragraph_mark_deleted` (comparer/finalize.rs:10903) L10896
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.first`, `.into_iter`, `.is_empty`, `.is_some`, `.map`, `.rev`, `PT::name`, `Some`, `W::author`, `W::date`, `W::del`, `W::ins`, `W::name`, `W::p`, `W::r`, `anchor_del`, `bare_in`
- **Called by:** `wrap_bare_del_text_runs` (comparer/finalize.rs:10771); `coverage_round_next_tests::deleted_textbox_attribution_is_inherited_per_attribute_and_runs_join_across_text` (comparer/finalize.rs:20010)

#### `mark_paragraph_mark_deleted`  (L10903-10945)
`fn mark_paragraph_mark_deleted( dom: &mut Dom, p: NodeId, like: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** Give `p`'s paragraph mark a `w:del` (attribution copied from `like`) unless the mark already carries a revision.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L10910,10918,10935; `Dom::new_element` (xmllinq/mod.rs:478) L10913,10921; `Dom::add_first` (xmllinq/mod.rs:1085) L10914,10944; `Dom::elements` (xmllinq/mod.rs:640) L10923; `Dom::name_is` (xmllinq/mod.rs:535) L10926; `Dom::add_before_self` (xmllinq/mod.rs:1114) L10928; `Dom::add` (xmllinq/mod.rs:1058) L10929; `rev_el` (comparer/finalize.rs:126) L10938; `Dom::attribute` (xmllinq/mod.rs:874) L10940; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L10941
- **Calls (std/external/unresolved):** `.any`, `.find`, `.into_iter`, `.is_some`, `.iter`, `.map`, `Some`, `W::author`, `W::date`, `W::del`, `W::ins`, `W::name`, `W::p_pr_change`, `W::p_pr`, `W::r_pr`, `W::sect_pr`
- **Called by:** `delete_stories_of_deleted_text_boxes` (comparer/finalize.rs:10812); `coverage_round_next_tests::paragraph_deletion_mark_inserts_before_each_schema_tail_and_preserves_format` (comparer/finalize.rs:20042)

#### `pending_deletion_texts`  (L10976-10985, pub)
`pub fn pending_deletion_texts(dom: &Dom, body: NodeId) -> std::collections::HashSet<String>`

- **Does:** Collect the concatenated visible text of every `w:del` wrapper under `body` (both `w:delText` and `w:delInstrText`).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L10978; `pending_deletion_fingerprint` (comparer/finalize.rs:10994) L10979
- **Calls (std/external/unresolved):** `.insert`, `.is_empty`, `Some`, `W::del`, `std::collections::HashSet::new`
- **Called by:** `coverage_round_next_tests::deleted_history_flattening_uses_both_field_and_text_fingerprint` (comparer/finalize.rs:20324); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `pending_deletion_fingerprint`  (L10994-11005)
`fn pending_deletion_fingerprint(dom: &Dom, del: NodeId) -> String`

- **Does:** Fingerprint of one `w:del` wrapper's pending-deletion text, gathering BOTH `w:delText` (run text) and `w:delInstrText` (field-instruction text).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L10997; `Dom::name` (xmllinq/mod.rs:526) L11000; `Dom::value` (xmllinq/mod.rs:1169) L11003
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_some_and`, `.map`, `W::del_text`, `W::name`
- **Called by:** `pending_deletion_texts` (comparer/finalize.rs:10976); `flatten_tracked_deletions` (comparer/finalize.rs:11030)

#### `flatten_tracked_deletions`  (L11030-11118, pub)
`pub fn flatten_tracked_deletions( dom: &mut Dom, body: NodeId, side: FlattenSide, other_side_pending: Option<&std::collections::HashSet<String>>, )`

- **Does:** Word-alignment mode: flatten pre-existing tracked deletions before diffing — unwrap `w:del`, convert `w:delText`/`w:delInstrText` back to `w:t`/`w:instrText` — so the old text re-enters the diff and comes out marked deleted: VISIBLE (struck through) like Word's redline, instead of vanishing via accept-before-diff.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L11040,11054,11082,11085; `Dom::element` (xmllinq/mod.rs:655) L11041,11042; `Dom::remove` (xmllinq/mod.rs:1102) L11044; `Dom::parent` (xmllinq/mod.rs:597) L11058; `Dom::name_is` (xmllinq/mod.rs:535) L11060,11068,11106; `Dom::nodes` (xmllinq/mod.rs:604) L11064; `Dom::is_element` (xmllinq/mod.rs:502) L11068; `pending_deletion_fingerprint` (comparer/finalize.rs:10994) L11081; `Dom::set_name` (xmllinq/mod.rs:539) L11083,11086; `Dom::attribute` (xmllinq/mod.rs:874) L11088,11089; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L11107,11109,11112; `Dom::replace_with` (xmllinq/mod.rs:1149) L11116
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.contains`, `.is_some_and`, `.iter`, `.map`, `.to_string`, `PT::name`, `Some`, `W::author`, `W::date`, `W::del_text`, `W::del`, `W::instr_text`, `W::name`, `W::p_pr`, `W::r_pr`, `W::r`, `W::t`
- **Called by:** `coverage_round_next_tests::deleted_history_flattening_uses_both_field_and_text_fingerprint` (comparer/finalize.rs:20324); `coverage_round_next_tests::revised_complex_deletion_stays_pending_while_original_restores_nested_text` (comparer/finalize.rs:20630); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `convert_stamped_predeletes`  (L11136-11227, pub)
`pub fn convert_stamped_predeletes( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** Word-alignment mode, produce-side counterpart of the doc-B stamp in [`flatten_tracked_deletions`]: every run stamped `pt:PreDelete` re-emerges as a PENDING DELETION — its `w:ins` wrapper (the diff saw B-only text) becomes `w:del`, Equal runs get a fresh `w:del` — so B's pre-existing deletions render struck through lik...
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L11143,11220,11223; `Dom::attribute` (xmllinq/mod.rs:874) L11146,11150,11153; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L11149,11156,11157,11163,11166,11199...; `Dom::parent` (xmllinq/mod.rs:597) L11169; `Dom::name` (xmllinq/mod.rs:526) L11172; `Dom::elements` (xmllinq/mod.rs:640) L11176,11182; `Dom::set_name` (xmllinq/mod.rs:539) L11184,11221,11224; `rev_el` (comparer/finalize.rs:126) L11190,11214; `Dom::remove` (xmllinq/mod.rs:1102) L11192,11204,11210,11217; `Dom::add` (xmllinq/mod.rs:1058) L11193,11205,11218; `Dom::add_after_self` (xmllinq/mod.rs:1131) L11194,11207; `Dom::new_element` (xmllinq/mod.rs:478) L11197; `Dom::attributes` (xmllinq/mod.rs:885) L11198; `Dom::add_before_self` (xmllinq/mod.rs:1114) L11216
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.len`, `.map`, `.position`, `.to_string`, `.to_vec`, `.unwrap_or`, `PT::name`, `Some`, `W::author`, `W::date`, `W::del_text`, `W::del`, `W::id`, `W::ins`, `W::instr_text`, `W::name`, `W::r`, `W::t`, `restamp`
- **Called by:** `coverage_round_two_tests::carried_insert_and_delete_split_preserves_neighbors_and_attribution` (comparer/finalize.rs:17891); `coverage_round_next_tests::predelete_conversion_missing_attribution_defaults_without_changing_neighbor_runs` (comparer/finalize.rs:20375); `coverage_final_batch_finalize_tests::carried_revision_matrix_preserves_order_formatting_and_original_attribution` (comparer/finalize.rs:24252); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `synthesize_table_cell_margins`  (L11249-11299, pub)
`pub fn synthesize_table_cell_margins( dom: &mut Dom, root: NodeId, default_table_styles: (bool, bool), )`

- **Does:** Word-alignment mode: Word's Compare synthesizes near-zero cell margins on the tables it emits — `w:tblInd w=10` and `w:tblCellMar left/right w=10` — present in NEITHER input nor Word's TableNormal default of 108 (forensics pair #5, helvetica_hr-onboarding).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L11260; `Dom::element` (xmllinq/mod.rs:655) L11262,11268,11274,11275,11286,11290; `Dom::elements` (xmllinq/mod.rs:640) L11271; `synthesize_table_cell_margins::{fn dxa}` (comparer/finalize.rs:11254) L11287,11292,11294; `insert_tblpr_child_in_order` (comparer/finalize.rs:10449) L11288,11296; `Dom::new_element` (xmllinq/mod.rs:478) L11291; `Dom::add` (xmllinq/mod.rs:1058) L11293,11295
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `Some`, `W::del`, `W::name`, `W::tbl`, `W::tr_pr`, `W::tr`
- **Called by:** `coverage_round_three_tests::bordered_table_geometry_uses_row_revision_source_and_schema_order` (comparer/finalize.rs:19134); `coverage_round_three_tests::table_geometry_preserves_explicit_values_and_fills_only_missing_properties` (comparer/finalize.rs:19181); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `synthesize_table_cell_margins::{fn dxa}`  (L11254-11259)
`fn dxa(dom: &mut Dom, name: &str, w: &str) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L11255; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L11256,11257
- **Calls (std/external/unresolved):** `Some`, `W::name`
- **Called by:** `synthesize_table_cell_margins` (comparer/finalize.rs:11249)

#### `fix_strict_validity_artifacts`  (L11321-11470, pub)
`pub fn fix_strict_validity_artifacts(dom: &mut Dom, root: NodeId)`

- **Does:** Word-validity artifacts beyond schema ordering (OpenXmlValidator-driven; Word's own redlines validate clean of all of these): - `w:cnfStyle` without the REQUIRED `w:val`: Strict expresses the flags as individual boolean attributes; synthesize the Transitional 12-bit bitmask (firstRow lastRow firstColumn lastColumn odd...
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L11336,11353,11405,11423,11431; `Dom::attribute` (xmllinq/mod.rs:874) L11337,11369,11407,11432; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L11345; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L11350,11373,11433,11434,11467; `Dom::value` (xmllinq/mod.rs:1169) L11354; `Dom::set_value` (xmllinq/mod.rs:1203) L11359; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L11363,11451; `XNamespace::none` (xmllinq/mod.rs:116) L11388; `XNamespace::get` (xmllinq/mod.rs:108) L11413; `fix_strict_validity_artifacts::{fn remap}` (comparer/finalize.rs:11390) L11414; `Dom::set_name` (xmllinq/mod.rs:539) L11424; `Dom::name` (xmllinq/mod.rs:526) L11452; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L11453; `Dom::attributes` (xmllinq/mod.rs:885) L11456
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter_map`, `.into_iter`, `.is_some_and`, `.is_some`, `.map_or`, `.map`, `.name`, `.ok`, `.parse`, `.push`, `.round`, `.starts_with`, `.strip_suffix`, `.to_string`, `.trim`, `Some`, `String::with_capacity`, `W14::name`, `W::drawing`, `W::name`, `W::val`, `WP14::name`, `crate::namespaces::A::name`, `crate::namespaces::WNE::name`, `u32::from_str_radix`
- **Macros:** `matches!`
- **Called by:** `coverage_round_next_tests::strict_shape_repair_stops_at_nested_drawing_and_keeps_bad_percent_text` (comparer/finalize.rs:20645); `compare_documents_impl` (document_comparer.rs:6501)

#### `fix_strict_validity_artifacts::{fn remap}`  (L11390-11404)
`fn remap(dom: &mut Dom, node: NodeId, wp_uri: &str, target: &XNamespace, drawing: &XName)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L11391; `Dom::is_element` (xmllinq/mod.rs:502) L11392; `Dom::name` (xmllinq/mod.rs:526) L11395; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L11399; `Dom::set_name` (xmllinq/mod.rs:539) L11400; `XNamespace::name` (xmllinq/mod.rs:134) L11400; `XName::local_name` (xmllinq/mod.rs:208) (by name) L11400; `fix_strict_validity_artifacts::{fn remap}` (comparer/finalize.rs:11390) L11402
- **Called by:** `fix_strict_validity_artifacts` (comparer/finalize.rs:11321); `fix_strict_validity_artifacts::{fn remap}` (comparer/finalize.rs:11390)

#### `normalize_universal_measures`  (L11485-11487, pub)
`pub fn normalize_universal_measures(dom: &mut Dom, root: NodeId)`

- **Does:** Canonicalize ST_UniversalMeasure values (`612pt`, `1in`, `2.54cm`, …) to plain twips on the twips-typed layout attributes.
- **Calls (crate):** `normalize_measures` (comparer/finalize.rs:11497) L11486
- **Called by:** `closing_mark_takes_revised_properties` (comparer/finalize.rs:3158); `sectpr_identity` (comparer/finalize.rs:11604); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209); `compare_documents_impl` (document_comparer.rs:6501)

#### `normalize_unit_measures`  (L11493-11495, pub)
`pub fn normalize_unit_measures(dom: &mut Dom, root: NodeId)`

- **Does:** [`normalize_universal_measures`] without the fractional-plain-value rounding: only unit-bearing spellings ("612pt") are rewritten.
- **Calls (crate):** `normalize_measures` (comparer/finalize.rs:11497) L11494
- **Called by:** `parse` (debug/render.rs:104)

#### `normalize_measures`  (L11497-11583)
`fn normalize_measures(dom: &mut Dom, root: NodeId, round_plain_fractional: bool)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L11559; `Dom::name` (xmllinq/mod.rs:526) L11560; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L11561; `XName::local_name` (xmllinq/mod.rs:208) (by name) L11566; `Dom::attribute` (xmllinq/mod.rs:874) L11575; `normalize_measures::{fn to_twips}` (comparer/finalize.rs:11543) L11577; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L11579
- **Calls (std/external/unresolved):** `.and_then`, `.find`, `.iter`, `.map`, `.to_string`, `Some`, `W::name`
- **Called by:** `normalize_universal_measures` (comparer/finalize.rs:11485); `normalize_unit_measures` (comparer/finalize.rs:11493)

#### `normalize_measures::{fn to_twips}`  (L11543-11558)
`fn to_twips(v: &str, round_plain_fractional: bool) -> Option<i64>`

- **Does:** _no doc comment_
- **Calls (crate):** `universal_measure_twips` (util/mod.rs:42) L11544
- **Calls (std/external/unresolved):** `.contains`, `.map`, `.ok`, `.parse`, `.round`, `.trim`, `Some`
- **Called by:** `normalize_measures` (comparer/finalize.rs:11497)

#### `sectpr_identity`  (L11604-11620, pub)
`pub fn sectpr_identity(dom: &mut Dom, sp: NodeId) -> String`

- **Does:** Serialize a sectPr with every `pt:*` bookkeeping attribute AND every `rsid*` attribute stripped, and measurement units canonicalized to twips, so the identity comparison in [`drop_hoisted_sectpr_artifacts`] is immune to what the pipeline changes between capture and check: unid stamping adds pt attrs, the pre-diff acce...
- **Calls (crate):** `Dom::clone_subtree` (xmllinq/mod.rs:1215) L11605; `remove_powertools_scratch_markup` (comparer/finalize.rs:1018) L11606; `normalize_universal_measures` (comparer/finalize.rs:11485) L11607; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L11608; `Dom::attributes` (xmllinq/mod.rs:885) L11609; `XName::local_name` (xmllinq/mod.rs:208) (by name) L11613; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L11616; `Dom::serialize_element` (xmllinq/mod.rs:1299) L11619
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.map`, `.starts_with`
- **Called by:** `closing_mark_takes_revised_properties` (comparer/finalize.rs:3158); `drop_sectpr_from_deleted_marks` (comparer/finalize.rs:10179); `drop_hoisted_sectpr_artifacts` (comparer/finalize.rs:11637); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `drop_hoisted_sectpr_artifacts`  (L11637-11650, pub)
`pub fn drop_hoisted_sectpr_artifacts( dom: &mut Dom, root: NodeId, genuine: &std::collections::HashSet<String>, )`

- **Does:** Word-alignment mode (settings-gated): drop pPr-embedded `sectPr`s that were NOT pPr-embedded in either input ("genuine" mid-section breaks are captured from the inputs before atomize hoists each doc's final body sectPr into its last paragraph — that hoist artifact must not become a mid-document page break; evidence: 1...
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L11643; `Dom::elements` (xmllinq/mod.rs:640) L11644; `sectpr_identity` (comparer/finalize.rs:11604) L11645; `Dom::remove` (xmllinq/mod.rs:1102) L11646
- **Calls (std/external/unresolved):** `.contains`, `Some`, `W::p_pr`, `W::sect_pr`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `mark_fully_revised_rows`  (L11666-11780, pub)
`pub fn mark_fully_revised_rows( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** Word-alignment mode (settings-gated): Word marks a fully-deleted table row with `w:del` inside `w:trPr` (and fully-inserted rows with `w:ins`), in addition to the cell-content revisions — the accept pipeline then removes the row (and the whole table once every row is marked).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L11674,11680,11704,11764; `Dom::elements` (xmllinq/mod.rs:640) L11679,11700,11761; `para_replacement_class_with_moves` (comparer/finalize.rs:7742) L11681; `Dom::name_is` (xmllinq/mod.rs:535) L11701; `Dom::value` (xmllinq/mod.rs:1169) L11702; `Dom::name` (xmllinq/mod.rs:526) L11706; `para_mark_revision` (comparer/finalize.rs:7626) L11721,11754,11756,11757; `Dom::element` (xmllinq/mod.rs:655) L11733,11746,11748,11770; `Dom::new_element` (xmllinq/mod.rs:478) L11736,11773; `Dom::add_first` (xmllinq/mod.rs:1085) L11737; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L11774,11775,11777; `add_row_mark` (comparer/lcs_table.rs:778) L11778
- **Calls (std/external/unresolved):** `.all`, `.any`, `.chain`, `.contains`, `.flat_map`, `.get_or_insert`, `.into_iter`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.to_string`, `.trim`, `MC::name`, `Some`, `W::author`, `W::date`, `W::del`, `W::id`, `W::ins`, `W::move_from`, `W::move_to`, `W::name`, `W::object`, `W::p_pr`, `W::p`, `W::pict`, `std::iter::once`
- **Called by:** `coverage_round_three_tests::revision_rows_preserve_cell_geometry_and_are_stamped_once` (comparer/finalize.rs:19263); `row_move_ownership_regressions::moving_cell_content_without_paragraph_ownership_keeps_existing_rows` (comparer/finalize.rs:23530); `row_move_ownership_regressions::moved_content_with_matching_paragraph_mark_tracks_row_lifecycle` (comparer/finalize.rs:23574); `row_nontext_ownership_regressions::unchanged_nontext_payload_prevents_a_moved_neighbor_from_owning_its_row` (comparer/finalize.rs:23804); `paired_row_mark_ownership_tests::paired_rows_with_surviving_marks_keep_empty_revised_cells_and_full_original_history` (comparer/finalize.rs:28899); `paired_row_mark_ownership_tests::unchanged_empty_pilcrows_keep_row_ownership_without_row_property_history` (comparer/finalize.rs:28940); `paired_row_mark_ownership_tests::new_empty_rows_with_owned_pilcrows_still_receive_their_row_lifetime` (comparer/finalize.rs:28990); `paired_row_mark_ownership_tests::explicit_row_lifetime_is_not_overridden_by_a_surviving_cell_mark` (comparer/finalize.rs:29074); `paired_row_mark_ownership_tests::explicit_row_lifetime_preserves_opposite_content_revisions_and_property_history` (comparer/finalize.rs:29087); `live_opaque_row_source_boundary_tests::unchanged_opaque_cell_owners_prevent_row_lifetime_synthesis_without_text_caches` (comparer/finalize.rs:34493); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `splice_trailing_short_pure_dels_midstream`  (L11798-11940, pub)
`pub fn splice_trailing_short_pure_dels_midstream(dom: &mut Dom, root: NodeId)`

- **Does:** Whole-doc short-base × long-next: insert-all-next + delete-all-base trails pure-D.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L11799,11918,11920,11921; `Dom::elements` (xmllinq/mod.rs:640) L11802,11915; `Dom::name_is` (xmllinq/mod.rs:535) L11805,11815,11831,11833,11848,11893...; `para_is_pure_deleted` (comparer/finalize.rs:2858) L11815,11881,11894; `para_is_pure_inserted` (comparer/finalize.rs:4175) L11832; `Dom::descendants` (xmllinq/mod.rs:668) L11834,11835,11852; `Dom::value_str` (xmllinq/mod.rs:1179) L11853; `para_body_alnum_len` (comparer/finalize.rs:7884) L11884,11895; `para_replacement_class` (comparer/finalize.rs:7738) L11910; `Dom::remove` (xmllinq/mod.rs:1102) L11933; `Dom::add_after_self` (xmllinq/mod.rs:1131) L11937
- **Calls (std/external/unresolved):** `.all`, `.any`, `.chars`, `.collect`, `.contains`, `.count`, `.enumerate`, `.filter`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.or`, `.push_str`, `.starts_with`, `.to_ascii_lowercase`, `.to_vec`, `.trim_start`, `Some`, `String::new`, `W::body`, `W::del`, `W::ins`, `W::name`, `W::p`, `W::sect_pr`, `W::t`, `W::tbl`, `W::tr`, `disappears_on_rejection`
- **Macros:** `vec!`
- **Called by:** `deep_production_path_tests::short_residual_splice_prefers_tip_then_toc_and_moves_only_the_short_numbered_title` (comparer/finalize.rs:22404); `deep_production_path_tests::short_title_splice_crosses_only_tables_whose_every_row_is_structurally_inserted` (comparer/finalize.rs:22442); `deep_production_path_tests::short_residual_splice_majority_and_length_boundaries_do_not_invent_reordering` (comparer/finalize.rs:22509); `deep_production_path_tests::late_tip_overrides_earlier_toc_and_numbered_splice_anchors` (comparer/finalize.rs:23371); `finalizer_remaining_source_boundary_tests::midstream_deleted_blocks_cross_only_payload_that_disappears_on_rejection` (comparer/finalize.rs:31038); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `split_digits_ins_from_mixed_title`  (L11954-12024, pub)
`pub fn split_digits_ins_from_mixed_title(dom: &mut Dom, root: NodeId)`

- **Does:** M147 — MIX that is **only** leading pure-ins digits ("24") + pure-del demo title body (1_5_line_spacing×24): Word keeps pure-I "24" then pure-D title as separate paragraphs.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L11955; `Dom::elements` (xmllinq/mod.rs:640) L11958,11972; `Dom::name_is` (xmllinq/mod.rs:535) L11961,11964,11977,11980,12007,12010; `Dom::descendants` (xmllinq/mod.rs:668) L11967,11968,11993,12007,12010; `Dom::value_str` (xmllinq/mod.rs:1179) L11994; `Dom::new_element` (xmllinq/mod.rs:478) L12015; `Dom::parent` (xmllinq/mod.rs:597) L12017; `Dom::remove` (xmllinq/mod.rs:1102) L12018; `Dom::add` (xmllinq/mod.rs:1058) L12019; `Dom::add_before_self` (xmllinq/mod.rs:1114) L12022
- **Calls (std/external/unresolved):** `.all`, `.any`, `.chars`, `.collect`, `.filter`, `.into_iter`, `.is_ascii_digit`, `.is_empty`, `.is_some`, `.is_whitespace`, `.iter`, `.push_str`, `.push`, `.trim`, `Some`, `String::new`, `Vec::new`, `W::body`, `W::del`, `W::ins`, `W::p_pr`, `W::p`, `W::sect_pr`, `W::t`
- **Called by:** `coverage_round_next_tests::digit_title_split_moves_all_leading_fragments_and_keeps_original_properties` (comparer/finalize.rs:19412); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `peel_trailing_ins_from_mix_into_following_pure_del`  (L12041-12166, pub)
`pub fn peel_trailing_ins_from_mix_into_following_pure_del(dom: &mut Dom, root: NodeId)`

- **Does:** M144 — MIX with trailing pure-ins + following pure-D body: when the trailing insert and pure-D share a connector/content word (italic×justified: both have `for`), peel the trailing ins into the pure-D as a mixed para via simple word interleave (Word: del prefix + Equal` for ` + ins phrase + del rest).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L12042; `Dom::elements` (xmllinq/mod.rs:640) L12046,12080,12144; `Dom::name_is` (xmllinq/mod.rs:535) L12049,12055,12084,12087,12093,12147; `para_is_pure_deleted` (comparer/finalize.rs:2858) L12058; `Dom::descendants` (xmllinq/mod.rs:668) L12061,12062,12095,12118; `para_mark_revision` (comparer/finalize.rs:7626) L12063; `para_revision_body_text` (comparer/finalize.rs:7854) L12068; `Dom::value_str` (xmllinq/mod.rs:1179) L12096,12119; `Dom::name` (xmllinq/mod.rs:526) L12110; `Dom::parent` (xmllinq/mod.rs:597) L12149; `Dom::remove` (xmllinq/mod.rs:1102) L12152; `Dom::add_before_self` (xmllinq/mod.rs:1114) L12154; `Dom::add` (xmllinq/mod.rs:1058) L12156
- **Calls (std/external/unresolved):** `.all`, `.any`, `.as_str`, `.chars`, `.collect`, `.contains`, `.count`, `.filter`, `.find`, `.into_iter`, `.is_alphanumeric`, `.is_empty`, `.is_none`, `.iter`, `.len`, `.map`, `.push_str`, `.push`, `.rev`, `.reverse`, `.saturating_sub`, `.split`, `.to_ascii_lowercase`, `Some`, `String::new`, `Vec::new`, `W::body`, `W::del`, `W::ins`, `W::p_pr`, `W::p`, `W::r`, `W::sect_pr`, `W::t`
- **Called by:** `coverage_round_next_tests::trailing_insertion_peel_has_inclusive_token_bounds_and_keeps_punctuation` (comparer/finalize.rs:19463); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `restore_short_del_before_long_ins`  (L12180-12249, pub)
`pub fn restore_short_del_before_long_ins(dom: &mut Dom, root: NodeId)`

- **Does:** M159 (text_highlight×times): after merge reorders unrelated 1v1 D-then-I to I-before-D, restore Word order when pure-I is longer than a short pure-D and they sit after a MIX title: `… MIX | INS | DEL | MIX …` → `… MIX | DEL | INS | MIX …`.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L12181; `Dom::elements` (xmllinq/mod.rs:640) L12185; `Dom::name_is` (xmllinq/mod.rs:535) L12188,12194,12208; `para_is_pure_inserted` (comparer/finalize.rs:4175) L12197; `para_is_pure_deleted` (comparer/finalize.rs:2858) L12197; `Dom::descendants` (xmllinq/mod.rs:668) L12212,12214; `para_mark_revision` (comparer/finalize.rs:7626) L12213,12215; `para_revision_body_text` (comparer/finalize.rs:7854) L12223; `para_body_alnum_len` (comparer/finalize.rs:7884) L12231,12232; `Dom::parent` (xmllinq/mod.rs:597) L12237; `Dom::remove` (xmllinq/mod.rs:1102) L12240; `Dom::add_before_self` (xmllinq/mod.rs:1114) L12241
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter`, `.into_iter`, `.is_empty`, `.is_none`, `.len`, `.saturating_sub`, `.to_ascii_lowercase`, `Some`, `W::body`, `W::del`, `W::ins`, `W::p`, `W::sect_pr`
- **Called by:** `coverage_round_next_tests::short_deletion_restore_checks_filename_and_alnum_length_boundaries` (comparer/finalize.rs:19635); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `peel_trailing_del_from_mix_into_following_pure_ins`  (L12261-12470, pub)
`pub fn peel_trailing_del_from_mix_into_following_pure_ins(dom: &mut Dom, root: NodeId)`

- **Does:** M154 (justified_underline×justify_2): trailing `w:del` on a MIX followed by pure-I body — peel the del into the pure-I so both become MIX.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L12262,12345,12429,12431,12432,12437...; `Dom::elements` (xmllinq/mod.rs:640) L12266,12303; `Dom::name_is` (xmllinq/mod.rs:535) L12269,12275,12307,12310,12315; `para_is_pure_inserted` (comparer/finalize.rs:4175) L12278; `Dom::descendants` (xmllinq/mod.rs:668) L12281,12283,12317,12366,12369; `para_mark_revision` (comparer/finalize.rs:7626) L12282,12296; `peel_has_text_only_body` (comparer/finalize.rs:12473) L12287; `final_peel_has_identical_closing_properties` (comparer/finalize.rs:12491) L12299; `Dom::value_str` (xmllinq/mod.rs:1179) L12318,12367,12370; `trailing_equal_period` (comparer/finalize.rs:12875) L12325,12336; `Dom::parent` (xmllinq/mod.rs:597) L12325,12344,12457; `text_leaves` (comparer/finalize.rs:12898) L12339; `Dom::serialize_element` (xmllinq/mod.rs:1299) L12346; `run_properties` (markdown/xml.rs:1806) (by name) L12352; `Dom::name` (xmllinq/mod.rs:526) L12358; `para_revision_body_text` (comparer/finalize.rs:7854) L12382; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L12434; `max_numeric_id` (comparer/finalize.rs:14901) L12435; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L12436; `Dom::new_element` (xmllinq/mod.rs:478) L12440,12448; `Dom::add_first` (xmllinq/mod.rs:1085) L12441,12453; `Dom::add` (xmllinq/mod.rs:1058) L12449,12461; `Dom::remove` (xmllinq/mod.rs:1102) L12460
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_str`, `.chars`, `.collect`, `.contains`, `.copied`, `.count`, `.expect`, `.filter`, `.first`, `.into_iter`, `.is_alphanumeric`, `.is_empty`, `.is_none`, `.is_some_and`, `.iter`, `.last`, `.len`, `.map`, `.push_str`, `.push`, `.rev`, `.reverse`, `.saturating_add`, `.saturating_sub`, `.split`, `.to_ascii_lowercase`, `.to_string`, `.unwrap_or_default`, `Some`, `String::new`, `Vec::new`, `W::body`, `W::del_text`, `W::del`, `W::id`, `W::ins`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `W::sect_pr`, `W::t`
- **Called by:** `coverage_round_next_tests::trailing_deletion_peel_truth_table_for_shared_long_and_copula_paths` (comparer/finalize.rs:19533); `deep_production_path_tests::deletion_peel_keeps_retained_source_boundary_and_inserted_continuations` (comparer/finalize.rs:22616); `deep_production_path_tests::final_deletion_peel_transfers_boundary_only_with_equal_properties_and_text_payload` (comparer/finalize.rs:22682); `deep_production_path_tests::deletion_peel_keeps_nontext_only_original_anchors_in_their_source_paragraph` (comparer/finalize.rs:22773); `deep_production_path_tests::deletion_peel_keeps_punctuation_and_whitespace_only_original_anchors` (comparer/finalize.rs:22832); `deep_production_path_tests::deletion_peel_keeps_original_closing_boundary_when_all_original_words_are_deleted` (comparer/finalize.rs:22879); `deep_production_path_tests::wholesale_deleted_body_peel_transfers_its_retained_closing_boundary` (comparer/finalize.rs:22924); `deep_production_path_tests::deletion_peel_does_not_move_an_original_suffix_past_equal_nonperiod_text` (comparer/finalize.rs:22977); `deep_production_path_tests::borrowed_period_peel_keeps_the_original_period_run_formatting` (comparer/finalize.rs:23024); `deep_production_path_tests::same_format_borrowed_period_peel_keeps_literal_text_and_character_properties` (comparer/finalize.rs:23061); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `peel_has_text_only_body`  (L12473-12486)
`fn peel_has_text_only_body(dom: &Dom, node: NodeId) -> bool`

- **Does:** Keep nontext payload under its original paragraph ownership during M154.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L12474; `Dom::name` (xmllinq/mod.rs:526) L12477; `Dom::elements` (xmllinq/mod.rs:640) L12483; `peel_has_text_only_body` (comparer/finalize.rs:12473) L12485
- **Calls (std/external/unresolved):** `.all`, `.contains`, `.into_iter`, `W::del_text`, `W::del`, `W::ins`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `W::t`
- **Called by:** `peel_trailing_del_from_mix_into_following_pure_ins` (comparer/finalize.rs:12261); `peel_has_text_only_body` (comparer/finalize.rs:12473); `final_peel_has_identical_closing_properties` (comparer/finalize.rs:12491)

#### `final_peel_has_identical_closing_properties`  (L12491-12547)
`fn final_peel_has_identical_closing_properties( dom: &mut Dom, mix: NodeId, inserted: NodeId, ) -> bool`

- **Does:** Only the final inserted carrier can take a source continuation without losing its paragraph formatting.
- **Calls (crate):** `peel_has_text_only_body` (comparer/finalize.rs:12473) L12496; `Dom::element` (xmllinq/mod.rs:655) L12499,12500,12504,12505,12509,12524...; `Dom::elements` (xmllinq/mod.rs:640) L12510,12537,12540; `Dom::name_is` (xmllinq/mod.rs:535) L12518; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L12532,12534; `Dom::new_element` (xmllinq/mod.rs:478) L12533; `Dom::remove` (xmllinq/mod.rs:1102) L12538,12541; `remove_powertools_scratch_markup` (comparer/finalize.rs:1018) L12544; `Dom::serialize_element` (xmllinq/mod.rs:1299) L12546
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.chain`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.map`, `.unwrap_or_else`, `Some`, `W::del`, `W::ins`, `W::move_from`, `W::name`, `W::p_pr_change`, `W::p_pr`, `W::r_pr`, `W::sect_pr`
- **Called by:** `peel_trailing_del_from_mix_into_following_pure_ins` (comparer/finalize.rs:12261)

#### `capture_deleted_demo_title_properties`  (L12579-12594, pub(crate))
`pub(crate) fn capture_deleted_demo_title_properties( dom: &mut Dom, root: NodeId, ) -> std::collections::HashMap<NodeId, NodeId>`

- **Does:** Snapshot every pure-deleted paragraph that looks like a "Demo" title's `w:pPr` BEFORE [`strip_redundant_demo_default_spacing`] runs, so M143's fold can re-attach authored Heading properties inside `w:pPrChange`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L12583; `para_is_pure_deleted` (comparer/finalize.rs:2858) L12586; `para_looks_like_demo_title` (comparer/finalize.rs:7901) L12587; `Dom::element` (xmllinq/mod.rs:655) L12588; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L12590
- **Calls (std/external/unresolved):** `.insert`, `Some`, `W::p_pr`, `W::p`, `std::collections::HashMap::new`
- **Called by:** `demo_title_pilcrow_history_tests::authored_title_snapshot_survives_demo_default_layout_normalization` (comparer/finalize.rs:28801); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `fold_midstream_demo_title_into_numbered_heading_with_settings`  (L12597-12610, pub)
`pub fn fold_midstream_demo_title_into_numbered_heading_with_settings( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** M143 with explicit settings and id-gen — production entry point.
- **Calls (crate):** `fold_midstream_demo_title_into_numbered_heading_with_source_properties` (comparer/finalize.rs:12619) L12603
- **Calls (std/external/unresolved):** `std::collections::HashMap::new`
- **Called by:** `fold_midstream_demo_title_into_numbered_heading` (comparer/finalize.rs:12562); `coverage_round_next_tests::demo_title_fold_history_preserves_complete_old_layout_and_current_attribution` (comparer/finalize.rs:19711); `demo_title_pilcrow_history_tests::title_fold_retains_complete_pilcrow_formats_and_configured_history` (comparer/finalize.rs:28532); `demo_title_pilcrow_history_tests::nested_mark_effect_changes_keep_both_source_projections_and_disabled_tracking` (comparer/finalize.rs:28648); `demo_title_pilcrow_history_tests::title_fold_preserves_existing_mark_history_and_section_break_ownership` (comparer/finalize.rs:28768); `fragmented_source_label_boundary_tests::midstream_heading_search_preserves_controls_live_anchors_and_identical_property_owners` (comparer/finalize.rs:33722)

#### `fold_midstream_demo_title_into_numbered_heading_with_source_properties`  (L12619-12821, pub)
`pub fn fold_midstream_demo_title_into_numbered_heading_with_source_properties( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, source_properties: &std::collections::HashMap<NodeId, NodeId>, )`

- **Does:** M143 fold that re-attaches captured authored Heading properties inside `w:pPrChange` so the deleted-title's formatting history survives the merge.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L12626,12721,12722,12727,12728,12730...; `Dom::elements` (xmllinq/mod.rs:640) L12629,12769,12810,12814; `Dom::name_is` (xmllinq/mod.rs:535) L12632,12642,12654,12663,12687,12690...; `Dom::descendants` (xmllinq/mod.rs:668) L12643,12644,12648,12649,12698; `Dom::value_str` (xmllinq/mod.rs:1179) L12647,12650,12699; `Dom::ancestors_and_self` (xmllinq/mod.rs:787) L12651; `para_is_pure_deleted` (comparer/finalize.rs:2858) L12663; `para_looks_like_demo_title` (comparer/finalize.rs:7901) L12666; `Dom::name` (xmllinq/mod.rs:526) L12669; `para_has_no_text` (comparer/finalize.rs:3459) L12670; `Dom::parent` (xmllinq/mod.rs:597) L12680,12713; `para_is_pure_inserted` (comparer/finalize.rs:4175) L12693; `fold_midstream_demo_title_into_numbered_heading_with_source_properties::{fn clean_mark_properties}` (comparer/finalize.rs:12735) L12744,12745; `paragraph_mark_properties_equal` (comparer/finalize.rs:3280) L12749; `project_para_properties_for_change` (comparer/formatchg.rs:329) L12759; `Dom::new_element` (xmllinq/mod.rs:478) L12760,12762,12786; `Dom::add_first` (xmllinq/mod.rs:1085) L12763,12787; `normalize_para_properties` (comparer/formatchg.rs:349) L12766,12767; `Dom::remove` (xmllinq/mod.rs:1102) L12770,12791,12811,12819; `rev_el` (comparer/finalize.rs:126) L12772; `Dom::add` (xmllinq/mod.rs:1058) L12773,12774,12798,12816; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L12794; `Dom::add_before_self` (xmllinq/mod.rs:1114) L12796; `closing_mark_takes_revised_mark_formatting` (comparer/finalize.rs:3358) L12802
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.collect`, `.copied`, `.count`, `.enumerate`, `.filter`, `.find`, `.flatten`, `.get`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.map`, `.or_else`, `.push_str`, `.push`, `.rev`, `.saturating_sub`, `.starts_with`, `.trim_start`, `.trim`, `.unwrap_or_else`, `Some`, `String::new`, `Vec::new`, `W::body`, `W::del_text`, `W::ins`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr_change`, `W::r_pr`, `W::sect_pr`, `W::t`, `W::tbl`
- **Called by:** `fold_midstream_demo_title_into_numbered_heading_with_settings` (comparer/finalize.rs:12597); `demo_title_pilcrow_history_tests::authored_title_snapshot_survives_demo_default_layout_normalization` (comparer/finalize.rs:28801); `public_body_heuristic_story_scope_tests::public_body_layout_heuristics_preserve_complete_independent_header_and_footer_parts` (comparer/finalize.rs:32251); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `fold_midstream_demo_title_into_numbered_heading_with_source_properties::{fn clean_mark_properties}`  (L12735-12743)
`fn clean_mark_properties(dom: &mut Dom, properties: Option<NodeId>) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L12736,12739; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L12738; `Dom::add` (xmllinq/mod.rs:1058) L12740; `strip_para_revision_marks` (comparer/finalize.rs:3735) L12741
- **Calls (std/external/unresolved):** `.map`, `.unwrap_or_else`, `W::p_pr`, `W::p`
- **Called by:** `fold_midstream_demo_title_into_numbered_heading_with_source_properties` (comparer/finalize.rs:12619)

#### `repair_borrowed_sentence_period`  (L12834-12873, pub)
`pub fn repair_borrowed_sentence_period( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** Move a paragraph-final `.` whose shared equal segment is genuinely shared (the next paragraph starts with equal text) onto the side that already finished its sentence, and append a `.` to the continuation.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L12840; `Dom::elements` (xmllinq/mod.rs:640) L12843; `Dom::name_is` (xmllinq/mod.rs:535) L12845; `trailing_equal_period` (comparer/finalize.rs:12875) L12848; `text_leaves` (comparer/finalize.rs:12898) L12851; `text_side` (comparer/finalize.rs:12905) L12854; `side_text` (comparer/finalize.rs:12923) L12858,12859; `side_needs_borrowed_period` (comparer/finalize.rs:12887) L12860,12861; `wrap_run` (comparer/finalize.rs:12933) L12870; `append_period` (comparer/finalize.rs:12953) L12871
- **Calls (std/external/unresolved):** `.array_windows`, `.first`, `.is_some_and`, `W::body`, `W::del`, `W::ins`, `W::p`
- **Called by:** `coverage_round_next_tests::borrowed_period_moves_to_finished_side_and_finishes_the_continuation` (comparer/finalize.rs:20519); `deep_production_path_tests::borrowed_period_peel_keeps_the_original_period_run_formatting` (comparer/finalize.rs:23024); `deep_production_path_tests::same_format_borrowed_period_peel_keeps_literal_text_and_character_properties` (comparer/finalize.rs:23061); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `trailing_equal_period`  (L12875-12885)
`fn trailing_equal_period(dom: &Dom, p: NodeId) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `text_leaves` (comparer/finalize.rs:12898) L12876; `Dom::name_is` (xmllinq/mod.rs:535) L12877,12881; `text_side` (comparer/finalize.rs:12905) L12877; `Dom::value_str` (xmllinq/mod.rs:1179) L12877; `Dom::parent` (xmllinq/mod.rs:597) L12880,12881
- **Calls (std/external/unresolved):** `.last`, `Some`, `W::r`, `W::t`
- **Called by:** `peel_trailing_del_from_mix_into_following_pure_ins` (comparer/finalize.rs:12261); `repair_borrowed_sentence_period` (comparer/finalize.rs:12834)

#### `side_needs_borrowed_period`  (L12887-12896)
`fn side_needs_borrowed_period(text: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chars`, `.is_empty`, `.is_some_and`, `.last`, `.next`, `.starts_with`, `.trim_end`, `Some`
- **Macros:** `matches!`
- **Called by:** `repair_borrowed_sentence_period` (comparer/finalize.rs:12834)

#### `text_leaves`  (L12898-12903)
`fn text_leaves(dom: &Dom, p: NodeId) -> Vec<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L12899; `Dom::name_is` (xmllinq/mod.rs:535) L12901
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `W::del_text`, `W::t`
- **Called by:** `peel_trailing_del_from_mix_into_following_pure_ins` (comparer/finalize.rs:12261); `repair_borrowed_sentence_period` (comparer/finalize.rs:12834); `trailing_equal_period` (comparer/finalize.rs:12875); `side_text` (comparer/finalize.rs:12923); `append_period` (comparer/finalize.rs:12953); `table_has_both_sides` (comparer/finalize.rs:13032); `block_is_original_only` (comparer/finalize.rs:13054); `collapsed_text` (comparer/finalize.rs:13068); `split_insertion_at` (comparer/finalize.rs:15028); `coverage_round_two_tests::wholesale_mesh_emits_exact_shared_words_and_styled_residuals` (comparer/finalize.rs:16832); `coverage_round_two_tests::insertion_split_handles_unicode_run_boundaries_and_multi_leaf_runs` (comparer/finalize.rs:17838)

#### `text_side`  (L12905-12921)
`fn text_side(dom: &Dom, t: NodeId, stop: NodeId) -> &'static str`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L12906,12913,12916; `Dom::ancestors_and_self` (xmllinq/mod.rs:787) L12909
- **Calls (std/external/unresolved):** `W::del_text`, `W::del`, `W::ins`, `W::name`
- **Called by:** `repair_borrowed_sentence_period` (comparer/finalize.rs:12834); `trailing_equal_period` (comparer/finalize.rs:12875); `side_text` (comparer/finalize.rs:12923); `append_period` (comparer/finalize.rs:12953); `table_has_both_sides` (comparer/finalize.rs:13032); `block_is_original_only` (comparer/finalize.rs:13054)

#### `side_text`  (L12923-12931)
`fn side_text(dom: &Dom, p: NodeId, side: &str) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `text_leaves` (comparer/finalize.rs:12898) L12925; `text_side` (comparer/finalize.rs:12905) L12926; `Dom::value_str` (xmllinq/mod.rs:1179) L12927
- **Calls (std/external/unresolved):** `.push_str`, `String::new`
- **Called by:** `repair_borrowed_sentence_period` (comparer/finalize.rs:12834)

#### `wrap_run`  (L12933-12951)
`fn wrap_run( dom: &mut Dom, t: NodeId, wrapper_name: crate::xmllinq::XName, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L12940; `Dom::new_element` (xmllinq/mod.rs:478) L12943; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L12944,12946,12947; `Dom::add_before_self` (xmllinq/mod.rs:1114) L12948; `Dom::remove` (xmllinq/mod.rs:1102) L12949; `Dom::add` (xmllinq/mod.rs:1058) L12950
- **Calls (std/external/unresolved):** `.to_string`, `Some`, `W::author`, `W::date`, `W::id`
- **Called by:** `repair_borrowed_sentence_period` (comparer/finalize.rs:12834)

#### `append_period`  (L12953-12963)
`fn append_period(dom: &mut Dom, p: NodeId, side: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `text_leaves` (comparer/finalize.rs:12898) L12954; `text_side` (comparer/finalize.rs:12905) L12957; `Dom::value_str` (xmllinq/mod.rs:1179) (by name) L12961; `Dom::set_value` (xmllinq/mod.rs:1203) L12962
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.rev`
- **Macros:** `format!`
- **Called by:** `repair_borrowed_sentence_period` (comparer/finalize.rs:12834)

#### `restore_echoed_heading_deletions_before_table`  (L12979-13030, pub)
`pub fn restore_echoed_heading_deletions_before_table(dom: &mut Dom, root: NodeId)`

- **Does:** The comments redline aligns B's capability table with A's and emits A's preceding section (Parity through the old "3.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L12980; `Dom::elements` (xmllinq/mod.rs:640) L12983; `table_has_both_sides` (comparer/finalize.rs:13032) L12987; `para_is_pure_inserted` (comparer/finalize.rs:4175) L12992; `block_is_original_only` (comparer/finalize.rs:13054) L13000; `collapsed_text` (comparer/finalize.rs:13068) L13009; `paragraph_texts` (comparer/finalize.rs:13076) L13013; `Dom::remove` (xmllinq/mod.rs:1102) L13026; `Dom::add_before_self` (xmllinq/mod.rs:1114) L13027
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.filter`, `.is_empty`, `.iter`, `.len`, `.map`, `.push`, `.to_vec`, `Vec::new`, `W::body`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `table_has_both_sides`  (L13032-13049)
`fn table_has_both_sides(dom: &Dom, n: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L13033; `text_leaves` (comparer/finalize.rs:12898) L13038; `text_side` (comparer/finalize.rs:12905) L13039; `Dom::value_str` (xmllinq/mod.rs:1179) L13040,13041
- **Calls (std/external/unresolved):** `.is_empty`, `.trim`, `W::tbl`
- **Called by:** `restore_echoed_heading_deletions_before_table` (comparer/finalize.rs:12979)

#### `block_is_original_only`  (L13054-13066)
`fn block_is_original_only(dom: &Dom, n: NodeId) -> bool`

- **Does:** Original-only body block: deleted or moved-from text, and no inserted or moved-to text.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L13055; `Dom::descendants` (xmllinq/mod.rs:668) L13058,13059; `text_leaves` (comparer/finalize.rs:12898) L13063; `Dom::value_str` (xmllinq/mod.rs:1179) L13065; `text_side` (comparer/finalize.rs:12905) L13065
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_empty`, `.trim`, `Some`, `W::ins`, `W::name`, `W::p`, `W::tbl`
- **Called by:** `restore_echoed_heading_deletions_before_table` (comparer/finalize.rs:12979)

#### `collapsed_text`  (L13068-13074)
`fn collapsed_text(dom: &Dom, n: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `text_leaves` (comparer/finalize.rs:12898) L13070; `Dom::value_str` (xmllinq/mod.rs:1179) L13071
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.filter`, `.is_whitespace`, `.push_str`, `String::new`
- **Called by:** `restore_echoed_heading_deletions_before_table` (comparer/finalize.rs:12979); `paragraph_texts` (comparer/finalize.rs:13076)

#### `paragraph_texts`  (L13076-13086)
`fn paragraph_texts(dom: &Dom, block: NodeId) -> Vec<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L13077; `collapsed_text` (comparer/finalize.rs:13068) L13078,13083; `Dom::descendants` (xmllinq/mod.rs:668) L13081
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.map`, `Some`, `Vec::new`, `W::p`
- **Macros:** `vec!`
- **Called by:** `restore_echoed_heading_deletions_before_table` (comparer/finalize.rs:12979)

#### `park_jc_on_first_short_title_mix_from_body`  (L13103-13199, pub)
`pub fn park_jc_on_first_short_title_mix_from_body( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** M452 (right_aligned_italic × right_alignment_2 ~84.3 residual): short **title** MIX free-mesh has no pPr; Word parks original `jc` into `pPrChange` only (no live jc).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L13109,13139,13154,13157,13166,13169; `Dom::elements` (xmllinq/mod.rs:640) L13112,13144; `Dom::name_is` (xmllinq/mod.rs:535) L13115,13123,13163; `para_is_mixed_revision` (comparer/finalize.rs:5117) L13126; `para_side_word_count` (comparer/finalize.rs:2013) L13134,13135; `Dom::new_element` (xmllinq/mod.rs:478) L13142,13188,13189,13192; `Dom::add_before_self` (xmllinq/mod.rs:1114) L13145; `Dom::add` (xmllinq/mod.rs:1058) L13147,13191,13197,13198; `Dom::attribute` (xmllinq/mod.rs:874) L13172; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L13190,13193,13195,13196
- **Calls (std/external/unresolved):** `.collect`, `.copied`, `.filter`, `.first`, `.into_iter`, `.is_empty`, `.is_some`, `.len`, `.to_string`, `Some`, `W::author`, `W::body`, `W::date`, `W::id`, `W::jc_el`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::sect_pr`, `W::val`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `ensure_empty_pprchange_on_live_heading_spacing`  (L13215-13302, pub)
`pub fn ensure_empty_pprchange_on_live_heading_spacing( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** M453 (calibri_font × calibri_heading_2_right mid residual): Word keeps live Heading residual spacing **and** an empty `pPrChange` shell on mid MIX.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L13221,13240,13243,13246,13269,13272...; `Dom::elements` (xmllinq/mod.rs:640) L13224,13250; `Dom::name_is` (xmllinq/mod.rs:535) L13227,13237,13266; `Dom::name` (xmllinq/mod.rs:526) L13251; `para_is_mixed_revision` (comparer/finalize.rs:5117) L13266; `Dom::attribute` (xmllinq/mod.rs:874) L13286,13290; `Dom::new_element` (xmllinq/mod.rs:478) L13293,13294; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L13295,13297,13298; `Dom::add` (xmllinq/mod.rs:1058) L13299,13300
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.filter`, `.into_iter`, `.is_none`, `.is_some`, `.len`, `.ok`, `.parse`, `.to_string`, `.unwrap_or`, `Some`, `W::author`, `W::body`, `W::date`, `W::id`, `W::jc_el`, `W::name`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`, `W::spacing_el`
- **Called by:** `coverage_round_two_tests::live_heading_history_is_added_only_when_last_history_is_empty` (comparer/finalize.rs:18129); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `unaligned_paragraph_texts`  (L13314-13329, pub)
`pub fn unaligned_paragraph_texts(dom: &Dom, body: NodeId) -> HashSet<String>`

- **Does:** The text of each paragraph that sets no `w:jc`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L13315,13323; `Dom::element` (xmllinq/mod.rs:655) L13318,13319; `Dom::value` (xmllinq/mod.rs:1169) L13325
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.filter`, `.into_iter`, `.is_none`, `.map`, `Some`, `W::jc_el`, `W::p_pr`, `W::p`, `W::t`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `accepted_text`  (L13333-13345)
`fn accepted_text(dom: &Dom, kids: &[NodeId], p: NodeId) -> String`

- **Does:** The text `p` holds once accepted: its revised text after that of the paragraphs before it whose deleted marks merge them into `p`.
- **Calls (crate):** `revised_text` (comparer/finalize.rs:13348) L13335,13343; `Dom::name_is` (xmllinq/mod.rs:535) L13339; `para_mark_revision` (comparer/finalize.rs:7626) L13339
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map_or`, `.map`, `.position`, `.rposition`, `W::del`, `W::p`
- **Called by:** `promote_live_jc_from_pprchange_on_body_mix` (comparer/finalize.rs:6511)

#### `revised_text`  (L13348-13359)
`fn revised_text(dom: &Dom, p: NodeId) -> String`

- **Does:** A redline paragraph's revised text: its live and inserted `w:t`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L13349; `Dom::ancestors` (xmllinq/mod.rs:770) L13352; `Dom::name_is` (xmllinq/mod.rs:535) L13355; `Dom::value` (xmllinq/mod.rs:1169) L13357
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.filter`, `.into_iter`, `.map`, `.take_while`, `Some`, `W::del`, `W::t`
- **Called by:** `accepted_text` (comparer/finalize.rs:13333)

#### `paragraph_alignments`  (L13369-13382, pub)
`pub fn paragraph_alignments(dom: &Dom, body: NodeId) -> HashSet<(String, String)>`

- **Does:** Each paragraph's text and `w:jc` value, for paragraphs with a `w:jc`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L13370,13374; `Dom::element` (xmllinq/mod.rs:655) L13373; `Dom::value` (xmllinq/mod.rs:1169) L13377; `Dom::attribute` (xmllinq/mod.rs:874) L13379
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.into_iter`, `.map`, `.to_string`, `.unwrap_or`, `Some`, `W::jc_el`, `W::p_pr`, `W::p`, `W::t`, `W::val`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `ensure_empty_pprchange_on_eq_with_live_jc`  (L13403-13466, pub)
`pub fn ensure_empty_pprchange_on_eq_with_live_jc( dom: &mut Dom, root: NodeId, original: &HashSet<(String, String)>, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** M454 (center_alignment_2 residual ~87 → 100): Word EQ title keeps live `jc` + empty `pPrChange`.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L13410,13436,13439,13451,13454; `Dom::elements` (xmllinq/mod.rs:640) L13413; `Dom::name_is` (xmllinq/mod.rs:535) L13416,13419; `para_is_mixed_revision` (comparer/finalize.rs:5117) L13422; `para_is_pure_inserted` (comparer/finalize.rs:4175) L13423; `para_is_pure_deleted` (comparer/finalize.rs:2858) L13424; `Dom::descendants` (xmllinq/mod.rs:668) L13429,13430,13442; `para_mark_revision` (comparer/finalize.rs:7626) L13431,13432; `Dom::value` (xmllinq/mod.rs:1169) L13445; `Dom::attribute` (xmllinq/mod.rs:874) L13447; `Dom::new_element` (xmllinq/mod.rs:478) L13457,13458; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L13459,13461,13462; `Dom::add` (xmllinq/mod.rs:1058) L13463,13464
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter`, `.into_iter`, `.is_empty`, `.is_some`, `.map`, `.to_string`, `.unwrap_or`, `Some`, `W::author`, `W::body`, `W::date`, `W::del`, `W::id`, `W::ins`, `W::jc_el`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`, `W::t`, `W::val`
- **Called by:** `finalizer_distinct_boundary_contract_tests::an_equal_alignment_change_is_attributed_once_and_never_claims_source_history` (comparer/finalize.rs:29656); `public_body_heuristic_story_scope_tests::public_body_layout_heuristics_preserve_complete_independent_header_and_footer_parts` (comparer/finalize.rs:32251); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `strip_leading_del_echoing_prev_pure_i`  (L13481-13570, pub)
`pub fn strip_leading_del_echoing_prev_pure_i(dom: &mut Dom, root: NodeId)`

- **Does:** M458 (right_aligned_italic residual ~88 after M151 peel): M151 pure-I's B1 ("This document…") then free-meshes A1×B2, which leaves an **early del "This"** on the body MIX (often after a short leading ins "All") that Word does not show.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L13482; `Dom::elements` (xmllinq/mod.rs:640) L13485,13514; `Dom::name_is` (xmllinq/mod.rs:535) L13488,13493,13517,13525; `para_is_pure_inserted` (comparer/finalize.rs:4175) L13496; `para_is_mixed_revision` (comparer/finalize.rs:5117) L13496; `para_mark_revision` (comparer/finalize.rs:7626) L13503; `para_revision_body_text` (comparer/finalize.rs:7854) L13506; `Dom::descendants` (xmllinq/mod.rs:668) L13530,13541; `Dom::value_str` (xmllinq/mod.rs:1179) L13531,13542; `Dom::remove` (xmllinq/mod.rs:1102) L13568
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.contains`, `.filter`, `.find`, `.into_iter`, `.is_alphanumeric`, `.is_empty`, `.is_none`, `.iter`, `.len`, `.map`, `.split`, `.take`, `.to_ascii_lowercase`, `Some`, `W::body`, `W::del_text`, `W::del`, `W::p_pr`, `W::p`, `W::sect_pr`, `W::t`
- **Called by:** `coverage_round_two_tests::leading_deleted_echo_is_removed_only_with_one_or_two_tokens` (comparer/finalize.rs:18040); `deleted_carrier_echo_source_owner_tests::deleted_original_carrier_keeps_echo_prefix_and_both_complete_source_projections` (comparer/finalize.rs:34664); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `holds_only_text`  (L13578-13591)
`fn holds_only_text(dom: &Dom, node: NodeId) -> bool`

- **Does:** Whether a paragraph child holds only text: a run, or a revision wrapper of runs, whose content is run properties and `w:t`/`w:delText`.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L13580,13582,13583,13584,13587; `Dom::elements` (xmllinq/mod.rs:640) L13581,13590; `text_run` (convert/math_layout.rs:530) (by name) L13588
- **Calls (std/external/unresolved):** `.all`, `.into_iter`, `W::del_text`, `W::r_pr`, `W::r`, `W::t`
- **Called by:** `free_mesh_shared_title_token_in_mix` (comparer/finalize.rs:2531); `free_mesh_wholesale_body_mix` (comparer/finalize.rs:13611); `free_mesh_bookended_ins_del` (comparer/finalize.rs:14051)

#### `free_mesh_wholesale_body_mix`  (L13611-13762, pub)
`pub fn free_mesh_wholesale_body_mix(dom: &mut Dom, root: NodeId)`

- **Does:** M462 (center_aligned_bold body2 residual after M461): wholesale body MIX free-mesh with coverage-gated word-LCS.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L13612,13739,13741; `Dom::elements` (xmllinq/mod.rs:640) L13615,13624; `Dom::name_is` (xmllinq/mod.rs:535) L13618,13621,13627; `para_is_mixed_revision` (comparer/finalize.rs:5117) L13621; `Dom::name` (xmllinq/mod.rs:526) L13636; `Dom::descendants` (xmllinq/mod.rs:668) L13644,13666,13670; `Dom::value_str` (xmllinq/mod.rs:1179) L13645,13667,13671; `holds_only_text` (comparer/finalize.rs:13578) L13659; `alnum_tokens` (comparer/finalize.rs:2717) L13682,13683; `split_words_preserve` (comparer/finalize.rs:13764) L13712,13713; `word_lcs_indices_eligible` (comparer/finalize.rs:13804) L13717; `Dom::attribute` (xmllinq/mod.rs:874) L13731,13734; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L13742; `Dom::parent` (xmllinq/mod.rs:597) L13744; `Dom::remove` (xmllinq/mod.rs:1102) L13745; `rebuild_body_free_mesh_lcs` (comparer/finalize.rs:13829) L13748
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.as_str`, `.chars`, `.collect`, `.contains`, `.count`, `.ends_with`, `.filter`, `.intersection`, `.into_iter`, `.is_alphanumeric`, `.is_empty`, `.is_some`, `.iter`, `.len`, `.map`, `.min`, `.push_str`, `.push`, `.to_ascii_lowercase`, `.to_string`, `.trim_end`, `Some`, `String::new`, `Vec::new`, `W::author`, `W::body`, `W::date`, `W::del_text`, `W::del`, `W::ins`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `W::sect_pr`, `W::t`, `is_sig`
- **Called by:** `coverage_round_two_tests::wholesale_mesh_emits_exact_shared_words_and_styled_residuals` (comparer/finalize.rs:16832); `coverage_round_two_tests::wholesale_mesh_preserves_each_authored_terminal_whitespace_and_format` (comparer/finalize.rs:16891); `coverage_round_two_tests::wholesale_mesh_without_terminal_whitespace_keeps_exact_supported_word_streams` (comparer/finalize.rs:16942); `fragmented_source_label_boundary_tests::wholesale_glue_only_lcs_preserves_crossed_significant_source_anchors` (comparer/finalize.rs:33908); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `split_words_preserve`  (L13764-13769)
`fn split_words_preserve(text: &str) -> Vec<String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.is_alphanumeric`, `.is_empty`, `.map`, `.split`, `.to_string`
- **Called by:** `free_mesh_wholesale_body_mix` (comparer/finalize.rs:13611)

#### `word_lcs_indices`  (L13771-13800)
`fn word_lcs_indices(a: &[String], b: &[String]) -> Vec<(usize, usize)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.len`, `.max`, `.push`, `.reverse`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `word_lcs_indices_eligible` (comparer/finalize.rs:13804)

#### `word_lcs_indices_eligible`  (L13804-13824)
`fn word_lcs_indices_eligible(a: &[String], b: &[String], boiler: &[&str]) -> Vec<(usize, usize)>`

- **Does:** LCS only on significant (len≥4 non-boiler) or short glue words; map indices back to the full word lists.
- **Calls (crate):** `word_lcs_indices` (comparer/finalize.rs:13771) L13820
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.contains`, `.filter`, `.into_iter`, `.is_empty`, `.iter`, `.len`, `.map`, `Vec::new`, `eligible`
- **Called by:** `free_mesh_wholesale_body_mix` (comparer/finalize.rs:13611)

#### `rebuild_body_free_mesh_lcs`  (L13829-14032)
`fn rebuild_body_free_mesh_lcs( dom: &mut Dom, p: NodeId, ins_words: &[String], del_words: &[String], lcs: &[(usize, usize)], stamp: RevStamp<'_>, trailing_period: bool, )`

- **Does:** Rebuild free-mesh from word LCS.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L13868,13873,13882,13887,13892,13902...; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L13870,13889,13909,14024; `Dom::add` (xmllinq/mod.rs:1058) L13871,13878,13879,13890,13897,13898...; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L13875,13883,13885,13886,13894,13903...; `XNamespace::xml` (xmllinq/mod.rs:128) L13875,13894,13914; `Dom::add_text` (xmllinq/mod.rs:1071) L13877,13896,13916,14028; `push_run` (markdown/patch.rs:794) (by name) L13930,13954,13973,13992,14006
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.ends_with`, `.get`, `.is_empty`, `.join`, `.len`, `.name`, `.push`, `.starts_with`, `.to_string`, `.to_vec`, `Some`, `Vec::new`, `W::author`, `W::date`, `W::del_text`, `W::del`, `W::id`, `W::ins`, `W::r`, `W::t`
- **Macros:** `format!`
- **Called by:** `free_mesh_wholesale_body_mix` (comparer/finalize.rs:13611); `coverage_round_two_tests::lcs_rebuild_covers_equal_prefixes_and_both_unmatched_tails` (comparer/finalize.rs:17052)

#### `free_mesh_bookended_ins_del`  (L14051-14238, pub)
`pub fn free_mesh_bookended_ins_del(dom: &mut Dom, root: NodeId)`

- **Does:** M460 (right_align_bold×right_aligned_italic ~81.3): MIX with EQ bookends (`This ` … `.`) around a single wholesale ins+del that share one significant mid token (`right`).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L14052,14184,14187,14189; `Dom::elements` (xmllinq/mod.rs:640) L14055,14064; `Dom::name_is` (xmllinq/mod.rs:535) L14058,14061,14067,14183; `para_is_mixed_revision` (comparer/finalize.rs:5117) L14061; `Dom::name` (xmllinq/mod.rs:526) L14077,14196; `Dom::descendants` (xmllinq/mod.rs:668) L14084,14107,14111; `Dom::value_str` (xmllinq/mod.rs:1179) L14085,14108,14112; `holds_only_text` (comparer/finalize.rs:13578) L14100; `alnum_tokens` (comparer/finalize.rs:2717) L14114,14115; `split_around_whole_word` (comparer/finalize.rs:14242) L14156,14159; `Dom::attribute` (xmllinq/mod.rs:874) L14172,14175; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L14185,14190,14208,14209; `Dom::parent` (xmllinq/mod.rs:597) L14211; `Dom::remove` (xmllinq/mod.rs:1102) L14212; `Dom::add` (xmllinq/mod.rs:1058) L14216,14235; `m460_push_rev_text` (comparer/finalize.rs:14288) L14229,14230,14232,14233; `m460_push_eq_text` (comparer/finalize.rs:14270) L14231
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.as_str`, `.chars`, `.collect`, `.contains`, `.copied`, `.entry`, `.filter`, `.find`, `.first`, `.get`, `.into_iter`, `.is_alphanumeric`, `.is_empty`, `.is_some`, `.iter`, `.keys`, `.len`, `.map`, `.or_default`, `.push_str`, `.push`, `.sort_by_key`, `.to_string`, `Some`, `String::new`, `Vec::new`, `W::author`, `W::body`, `W::date`, `W::del_text`, `W::del`, `W::ins`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `W::sect_pr`, `W::t`, `std::cmp::Reverse`, `std::collections::HashMap::new`
- **Called by:** `coverage_tests::bookended_free_mesh_rebuilds_word_order_and_preserves_run_properties` (comparer/finalize.rs:16496); `deep_production_path_tests::bookended_mesh_handles_boundary_anchors_and_the_forty_token_limit` (comparer/finalize.rs:22279); `deep_production_path_tests::suffix_only_bookend_mesh_uses_pinned_defaults_and_keeps_the_original_suffix` (comparer/finalize.rs:23441); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `split_around_whole_word`  (L14242-14268)
`fn split_around_whole_word(text: &str, word: &str) -> Option<(String, String)>`

- **Does:** Split `text` around the first whole-word match of `word` (case-insensitive).
- **Calls (std/external/unresolved):** `.and_then`, `.chars`, `.find`, `.get`, `.is_alphanumeric`, `.is_some_and`, `.len`, `.next`, `.to_ascii_lowercase`, `.to_string`, `Some`
- **Called by:** `free_mesh_bookended_ins_del` (comparer/finalize.rs:14051); `coverage_tests::token_splitting_preserves_separators_and_original_case` (comparer/finalize.rs:15722)

#### `m460_push_eq_text`  (L14270-14286)
`fn m460_push_eq_text(dom: &mut Dom, p: NodeId, text: &str, sample_rpr: &Option<NodeId>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L14274,14279; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L14276; `Dom::add` (xmllinq/mod.rs:1058) L14277,14284,14285; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L14281; `XNamespace::xml` (xmllinq/mod.rs:128) L14281; `Dom::add_text` (xmllinq/mod.rs:1071) L14283
- **Calls (std/external/unresolved):** `.ends_with`, `.is_empty`, `.name`, `.starts_with`, `Some`, `W::r`, `W::t`
- **Called by:** `free_mesh_bookended_ins_del` (comparer/finalize.rs:14051)

#### `m460_push_rev_text`  (L14288-14344)
`fn m460_push_rev_text( dom: &mut Dom, p: NodeId, kind: &str, text: &str, stamp: RevStamp<'_>, next_id: &mut u32, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L14307,14312,14313,14323,14328,14333; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L14308,14310,14311,14315,14324,14326...; `XNamespace::xml` (xmllinq/mod.rs:128) L14315,14335; `Dom::add_text` (xmllinq/mod.rs:1071) L14317,14337; `Dom::add` (xmllinq/mod.rs:1058) L14318,14319,14320,14331,14338,14339...; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L14330
- **Calls (std/external/unresolved):** `.contains`, `.ends_with`, `.is_empty`, `.name`, `.starts_with`, `.to_string`, `Some`, `W::author`, `W::date`, `W::del_text`, `W::del`, `W::id`, `W::ins`, `W::r`, `W::t`
- **Called by:** `free_mesh_bookended_ins_del` (comparer/finalize.rs:14051)

#### `peel_trailing_for_word_onto_next_mix`  (L14359-14494, pub)
`pub fn peel_trailing_for_word_onto_next_mix(dom: &mut Dom, root: NodeId)`

- **Does:** M464 (center_bold ~83.6 residual): Word free-meshes trailing ` for <word>` from a mid-body MIX ins onto the following MIX as `EQ[ for ]|INS[<word> ]`, trimming the following del's trailing ` for`.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L14360,14446,14448,14449,14452,14454; `Dom::elements` (xmllinq/mod.rs:640) L14363,14378,14383; `Dom::name_is` (xmllinq/mod.rs:535) L14366,14371,14372,14381,14386,14389...; `para_is_mixed_revision` (comparer/finalize.rs:5117) L14373,14374; `Dom::descendants` (xmllinq/mod.rs:668) L14402,14406; `Dom::value_str` (xmllinq/mod.rs:1179) L14403,14407; `split_trailing_for_word` (comparer/finalize.rs:14497) L14409; `Dom::attribute` (xmllinq/mod.rs:874) L14434,14437; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L14451,14455,14466,14480; `rewrite_rev_text` (comparer/finalize.rs:14523) L14458,14460; `Dom::new_element` (xmllinq/mod.rs:478) L14464,14469,14474,14478,14483; `Dom::add` (xmllinq/mod.rs:1058) L14467,14472,14481,14487,14488; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L14470,14475,14476,14477,14485; `XNamespace::xml` (xmllinq/mod.rs:128) L14470,14485; `Dom::add_text` (xmllinq/mod.rs:1071) L14471,14486; `Dom::add_after_self` (xmllinq/mod.rs:1131) L14491,14492
- **Calls (std/external/unresolved):** `.and_then`, `.as_bytes`, `.collect`, `.copied`, `.ends_with`, `.filter`, `.find`, `.first`, `.into_iter`, `.is_empty`, `.iter`, `.len`, `.map`, `.name`, `.or_else`, `.push_str`, `.rev`, `.saturating_sub`, `.to_ascii_lowercase`, `.to_string`, `.trim_end`, `.trim`, `Some`, `String::new`, `W::author`, `W::body`, `W::date`, `W::del_text`, `W::del`, `W::id`, `W::ins`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `W::sect_pr`, `W::t`
- **Macros:** `format!`
- **Called by:** `coverage_round_next_tests::for_word_peel_preserves_run_format_and_uses_equal_format_for_shared_connector` (comparer/finalize.rs:20463); `deep_production_path_tests::connector_peel_preserves_both_sources_and_falls_back_to_inserted_run_format` (comparer/finalize.rs:22331); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `split_trailing_for_word`  (L14497-14520)
`fn split_trailing_for_word(text: &str) -> Option<(String, String)>`

- **Does:** Split trailing ` for <word>` (case-insensitive).
- **Calls (std/external/unresolved):** `.all`, `.any`, `.chars`, `.is_alphanumeric`, `.is_empty`, `.is_whitespace`, `.rfind`, `.to_ascii_lowercase`, `.to_string`, `.trim_end`, `.trim`, `Some`
- **Called by:** `peel_trailing_for_word_onto_next_mix` (comparer/finalize.rs:14359)

#### `rewrite_rev_text`  (L14523-14559)
`fn rewrite_rev_text(dom: &mut Dom, rev: NodeId, new_text: &str, is_del: bool)`

- **Does:** Replace all text under an ins/del with `new_text` (single run).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L14524,14528,14530; `Dom::parent` (xmllinq/mod.rs:597) L14533,14539; `Dom::remove` (xmllinq/mod.rs:1102) L14534,14540; `Dom::new_element` (xmllinq/mod.rs:478) L14545,14550,14552; `Dom::add` (xmllinq/mod.rs:1058) L14546,14558; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L14555; `XNamespace::xml` (xmllinq/mod.rs:128) L14555; `Dom::add_text` (xmllinq/mod.rs:1071) L14557
- **Calls (std/external/unresolved):** `.contains`, `.ends_with`, `.first`, `.is_some`, `.iter`, `.name`, `.skip`, `.starts_with`, `.to_vec`, `Some`, `W::del_text`, `W::r`, `W::t`
- **Called by:** `peel_trailing_for_word_onto_next_mix` (comparer/finalize.rs:14359)

#### `hoist_hyperlinks_out_of_revisions`  (L14594-14626, pub)
`pub fn hoist_hyperlinks_out_of_revisions(dom: &mut Dom, root: NodeId)`

- **Does:** `w:hyperlink` is not in CT_RunTrackChange's content model: a `w:ins`/`w:del` holds runs, not a hyperlink wrapper.
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L14597; `Dom::attribute` (xmllinq/mod.rs:874) L14601; `Dom::descendants` (xmllinq/mod.rs:668) L14612; `Dom::elements` (xmllinq/mod.rs:640) L14614; `Dom::name` (xmllinq/mod.rs:526) L14616; `split_revision_around_hyperlinks` (comparer/finalize.rs:14630) L14623
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.collect`, `.contains`, `.filter_map`, `.filter`, `.flat_map`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.max`, `.ok`, `.parse`, `.unwrap_or`, `Some`, `W::hyperlink`, `W::id`, `W::name`
- **Called by:** `coverage_final_batch_finalize_tests::hyperlink_hoisting_preserves_empty_wrappers_and_run_order_across_revisions` (comparer/finalize.rs:24460); `coverage_final_batch_finalize_tests::empty_field_in_a_revision_keeps_its_flags_and_padded_code` (comparer/finalize.rs:24544); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209); `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::one_sided_story_fields_join_the_story_revision` (document_comparer.rs:9424)

#### `split_revision_around_hyperlinks`  (L14630-14701)
`fn split_revision_around_hyperlinks(dom: &mut Dom, rev: NodeId, next_id: &mut u32)`

- **Does:** One step of [`hoist_hyperlinks_out_of_revisions`]: replace `rev` in its parent with the alternating sequence of same-shaped revisions and hyperlinks.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L14631,14673; `Dom::attributes` (xmllinq/mod.rs:885) L14635; `Dom::nodes` (xmllinq/mod.rs:604) L14636,14677; `Dom::new_element` (xmllinq/mod.rs:478) L14641; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L14643,14645; `Dom::remove` (xmllinq/mod.rs:1102) L14652,14681,14700; `Dom::name_is` (xmllinq/mod.rs:535) L14653; `Dom::elements` (xmllinq/mod.rs:640) L14653; `complex_field_runs` (comparer/finalize.rs:14706) L14658; `Dom::add_before_self` (xmllinq/mod.rs:1114) L14663,14686,14693; `Dom::add` (xmllinq/mod.rs:1058) L14669,14682,14684,14698
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.is_empty`, `.is_some_and`, `.to_string`, `Some`, `W::del`, `W::hyperlink`, `W::id`, `W::name`, `like_rev`
- **Called by:** `hoist_hyperlinks_out_of_revisions` (comparer/finalize.rs:14594)

#### `complex_field_runs`  (L14706-14734)
`fn complex_field_runs(dom: &mut Dom, field: NodeId, deleted: bool) -> Vec<NodeId>`

- **Does:** An empty `w:fldSimple` as the runs of its complex form: `begin` (with the field's `w:dirty` / `w:fldLock`), the code (`w:delInstrText` when `deleted`), `separate` and `end`.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L14708,14709,14720,14721; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L14710,14717,14727; `Dom::add` (xmllinq/mod.rs:1058) L14711,14730; `Dom::attribute` (xmllinq/mod.rs:874) L14716,14722; `XNamespace::xml` (xmllinq/mod.rs:128) L14727; `Dom::add_text` (xmllinq/mod.rs:1071) L14729
- **Calls (std/external/unresolved):** `.map`, `.name`, `.to_string`, `.trim`, `.unwrap_or`, `Some`, `W::name`, `W::r`, `char_run`
- **Macros:** `vec!`
- **Called by:** `mark_content_transform` (comparer/finalize.rs:192); `split_revision_around_hyperlinks` (comparer/finalize.rs:14630)

#### `repair_inherited_invalidity`  (L14767-14806, pub)
`pub fn repair_inherited_invalidity(dom: &mut Dom, root: NodeId)`

- **Does:** Repair invalidity we did not create but would otherwise ship.
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L14768,14782,14789; `Dom::name` (xmllinq/mod.rs:526) L14769,14799,14801; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L14770,14774; `Dom::attributes` (xmllinq/mod.rs:885) L14773; `Dom::is_namespace_declaration` (xmllinq/mod.rs:996) L14775; `XName::local_name` (xmllinq/mod.rs:208) (by name) L14776; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L14778,14784,14791; `Dom::attribute` (xmllinq/mod.rs:874) L14783,14790; `Dom::descendants` (xmllinq/mod.rs:668) L14796; `Dom::parent` (xmllinq/mod.rs:597) L14797,14800; `Dom::remove` (xmllinq/mod.rs:1102) L14803
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.filter`, `.is_empty`, `.is_none`, `.is_some_and`, `Some`, `W::name`, `W::r_pr`, `W::val`
- **Called by:** `coverage_final_batch_finalize_tests::inherited_invalidity_repairs_only_wml_schema_defects` (comparer/finalize.rs:24561); `compare_documents_impl` (document_comparer.rs:6501)

#### `enforce_deleted_text_kinds`  (L14834-14857, pub)
`pub fn enforce_deleted_text_kinds(dom: &mut Dom, root: NodeId)`

- **Does:** Enforce the deleted-text invariant everywhere, not just on runs that happened to go through `convert_run_text_to_del_text`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L14854; `descendants_trimmed::{fn walk}` (comparer/finalize.rs:65) L14855
- **Calls (std/external/unresolved):** `Some`, `W::del`
- **Called by:** `nested_deleted_text_source_ownership_tests::outer_deletion_preserves_independently_owned_nested_revision_text_and_fields` (comparer/finalize.rs:33977); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209); `compare_documents_impl` (document_comparer.rs:6501)

#### `enforce_deleted_text_kinds::{fn walk}`  (L14835-14853)
`fn walk(dom: &mut Dom, node: NodeId)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L14836; `Dom::is_element` (xmllinq/mod.rs:502) L14837; `Dom::name` (xmllinq/mod.rs:526) L14840; `Dom::set_name` (xmllinq/mod.rs:539) L14848,14849; `descendants_trimmed::{fn walk}` (comparer/finalize.rs:65) L14850
- **Calls (std/external/unresolved):** `.as_ref`, `W::del_text`, `W::ins`, `W::instr_text`, `W::name`, `W::t`
- **Called by:** `descendants_trimmed` (comparer/finalize.rs:63); `descendants_trimmed::{fn walk}` (comparer/finalize.rs:65); `convert_run_text_to_del_text` (comparer/finalize.rs:102); `convert_run_text_to_del_text::{fn walk}` (comparer/finalize.rs:103); `enforce_deleted_text_kinds` (comparer/finalize.rs:14834); `enforce_deleted_text_kinds::{fn walk}` (comparer/finalize.rs:14835)

#### `align_remaining_ladder_rungs`  (L14886-14899, pub)
`pub fn align_remaining_ladder_rungs(dom: &mut Dom, root: NodeId, sources: &LadderSources<'_>)`

- **Does:** Match the six output shapes the parity ladder still counted after the operation sequence already matched — a trailing revised insertion split out of the comparison insertion, a deleted pilcrow on a tab-joined swallowed title, a commented list item unmerged from the deletion, a duplicated Normal-styled deletion, spacin...
- **Calls (crate):** `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L14887,14888; `Dom::root` (xmllinq/mod.rs:822) L14889; `max_numeric_id` (comparer/finalize.rs:14901) L14892; `split_trailing_insertions` (comparer/finalize.rs:15011) L14893; `mark_tabbed_deleted_titles` (comparer/finalize.rs:15150) L14894; `unmerge_comment_list_item` (comparer/finalize.rs:15233) L14895; `drop_duplicate_normal_style` (comparer/finalize.rs:15299) L14896; `move_spacing_onto_bare_deletion` (comparer/finalize.rs:15364) L14897; `stamp_trailing_numbering` (comparer/finalize.rs:15446) L14898
- **Calls (std/external/unresolved):** `.saturating_add`
- **Called by:** `ladder_suffix_source_boundary_tests::public_ladder_restores_suffix_author_at_unicode_and_run_boundaries` (comparer/finalize.rs:34088); `compare_documents_impl` (document_comparer.rs:6501)

#### `max_numeric_id`  (L14901-14911)
`fn max_numeric_id(dom: &Dom, root: NodeId) -> u32`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L14903; `Dom::attribute` (xmllinq/mod.rs:874) L14904
- **Calls (std/external/unresolved):** `.max`, `.parse`, `W::id`
- **Called by:** `mesh_short_label_shared_eq_with_settings` (comparer/finalize.rs:7199); `peel_trailing_del_from_mix_into_following_pure_ins` (comparer/finalize.rs:12261); `fold_midstream_demo_title_into_numbered_heading` (comparer/finalize.rs:12562); `align_remaining_ladder_rungs` (comparer/finalize.rs:14886); `short_label_format_history_tests::configured_label_tracking_preserves_sources_and_respects_disabled_changes` (comparer/finalize.rs:24148)

#### `body_paragraphs`  (L14913-14915)
`fn body_paragraphs(dom: &Dom, root: NodeId) -> Vec<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L14914
- **Calls (std/external/unresolved):** `Some`, `W::p`
- **Called by:** `suffix_hits` (comparer/finalize.rs:14986); `mark_tabbed_deleted_titles` (comparer/finalize.rs:15150); `unmerge_comment_list_item` (comparer/finalize.rs:15233); `drop_duplicate_normal_style` (comparer/finalize.rs:15299); `move_spacing_onto_bare_deletion` (comparer/finalize.rs:15364); `stamp_trailing_numbering` (comparer/finalize.rs:15446)

#### `markup_text`  (L14918-14932)
`fn markup_text(dom: &Dom, node: NodeId) -> String`

- **Does:** Concatenated `w:t` and `w:delText`, with nothing inserted between nodes.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L14923; `Dom::value` (xmllinq/mod.rs:1169) L14924; `Dom::descendants` (xmllinq/mod.rs:668) L14928
- **Calls (std/external/unresolved):** `.push_str`, `String::new`, `W::del_text`, `W::t`, `consider`
- **Called by:** `suffix_hits` (comparer/finalize.rs:14986); `split_trailing_insertions` (comparer/finalize.rs:15011); `mark_tabbed_deleted_titles` (comparer/finalize.rs:15150); `content_revisions` (comparer/finalize.rs:15189); `drop_duplicate_normal_style` (comparer/finalize.rs:15299); `move_spacing_onto_bare_deletion` (comparer/finalize.rs:15364); `stamp_trailing_numbering` (comparer/finalize.rs:15446); `coverage_round_two_tests::segments` (comparer/finalize.rs:16796); `coverage_round_two_tests::title_page_relocation_preserves_six_spacers_and_moves_only_last` (comparer/finalize.rs:17092); `coverage_round_two_tests::stranded_mark_is_cloned_without_removing_empty_shell` (comparer/finalize.rs:17239); `coverage_round_two_tests::heading_spacing_parks_on_last_short_deleted_residual` (comparer/finalize.rs:17359); `coverage_round_two_tests::mixed_numbering_moves_before_last_deletion_mark_or_history` (comparer/finalize.rs:17441); +11 more

#### `inside`  (L14934-14942)
`fn inside(dom: &Dom, mut node: NodeId, ancestor: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L14935
- **Called by:** `content_revisions` (comparer/finalize.rs:15189); `locate` (edit.rs:5475) (?)

#### `inside_table`  (L14944-14953)
`fn inside_table(dom: &Dom, node: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L14947; `Dom::parent` (xmllinq/mod.rs:597) L14950
- **Calls (std/external/unresolved):** `Some`, `W::tbl`
- **Called by:** `mark_tabbed_deleted_titles` (comparer/finalize.rs:15150)

#### `run_holding`  (L14955-14966)
`fn run_holding(dom: &Dom, node: NodeId, ins: NodeId) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L14961; `Dom::parent` (xmllinq/mod.rs:597) L14964
- **Calls (std/external/unresolved):** `Some`, `W::r`
- **Called by:** `split_insertion_at` (comparer/finalize.rs:15028)

#### `split_at_chars`  (L14968-14977)
`fn split_at_chars(value: &str, chars: usize) -> (String, String)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.char_indices`, `.enumerate`, `.len`, `.to_string`
- **Called by:** `split_insertion_at` (comparer/finalize.rs:15028)

#### `suffix_hits`  (L14986-15009)
`fn suffix_hits(dom: &Dom, revised: NodeId) -> Vec<SuffixHit>`

- **Does:** _no doc comment_
- **Calls (crate):** `body_paragraphs` (comparer/finalize.rs:14913) L14988; `Dom::descendants` (xmllinq/mod.rs:668) L14989; `markup_text` (comparer/finalize.rs:14918) L14993,14994; `Dom::attribute` (xmllinq/mod.rs:874) L15002,15005
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.ends_with`, `.is_empty`, `.len`, `.map`, `.push`, `Some`, `Vec::new`, `W::author`, `W::date`, `W::ins`
- **Called by:** `split_trailing_insertions` (comparer/finalize.rs:15011)

#### `split_trailing_insertions`  (L15011-15026)
`fn split_trailing_insertions(dom: &mut Dom, root: NodeId, revised: NodeId, id_gen: &mut u32)`

- **Does:** _no doc comment_
- **Calls (crate):** `suffix_hits` (comparer/finalize.rs:14986) L15012; `Dom::descendants` (xmllinq/mod.rs:668) L15016; `markup_text` (comparer/finalize.rs:14918) L15018; `split_insertion_at` (comparer/finalize.rs:15028) L15024
- **Calls (std/external/unresolved):** `.clone`, `.find`, `.is_empty`, `.iter`, `Some`, `W::ins`
- **Called by:** `align_remaining_ladder_rungs` (comparer/finalize.rs:14886)

#### `split_insertion_at`  (L15028-15138)
`fn split_insertion_at( dom: &mut Dom, ins: NodeId, cut: usize, author: &Option<String>, date: &Option<String>, id_gen: &mut u32, ) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `text_leaves` (comparer/finalize.rs:12898) L15036,15124; `Dom::value` (xmllinq/mod.rs:1169) L15041,15062; `split_at_chars` (comparer/finalize.rs:14968) L15062; `Dom::name` (xmllinq/mod.rs:526) L15063; `Dom::set_value` (xmllinq/mod.rs:1203) L15066,15069; `Dom::new_element` (xmllinq/mod.rs:478) L15067,15087; `copy_attrs` (comparer/finalize.rs:136) L15068,15088; `XNamespace::xml` (xmllinq/mod.rs:128) L15072; `xml_space_attr` (comparer/finalize.rs:1040) L15074; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L15075,15090,15093,15095; `Dom::add_after_self` (xmllinq/mod.rs:1131) L15078,15136; `run_holding` (comparer/finalize.rs:14955) L15100,15110; `Dom::remove` (xmllinq/mod.rs:1102) L15113,15121,15125,15131; `Dom::add` (xmllinq/mod.rs:1058) L15114,15127,15132; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L15117; `Dom::nodes` (xmllinq/mod.rs:604) L15118,15129,15130
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.contains`, `.copied`, `.count`, `.enumerate`, `.filter`, `.insert`, `.into_iter`, `.is_empty`, `.iter`, `.len`, `.name`, `.push`, `.take`, `.to_string`, `.to_vec`, `Some`, `Vec::new`, `W::author`, `W::date`, `W::id`, `W::ins`
- **Called by:** `split_trailing_insertions` (comparer/finalize.rs:15011); `coverage_round_two_tests::insertion_split_handles_unicode_run_boundaries_and_multi_leaf_runs` (comparer/finalize.rs:17838)

#### `has_deleted_mark`  (L15140-15148)
`fn has_deleted_mark(dom: &Dom, paragraph: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L15141,15144,15147
- **Calls (std/external/unresolved):** `.is_some`, `W::del`, `W::p_pr`, `W::r_pr`
- **Called by:** `mark_tabbed_deleted_titles` (comparer/finalize.rs:15150); `drop_duplicate_normal_style` (comparer/finalize.rs:15299)

#### `mark_tabbed_deleted_titles`  (L15150-15187)
`fn mark_tabbed_deleted_titles( dom: &mut Dom, root: NodeId, original: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** _no doc comment_
- **Calls (crate):** `body_paragraphs` (comparer/finalize.rs:14913) L15158,15169; `inside_table` (comparer/finalize.rs:14944) L15159; `markup_text` (comparer/finalize.rs:14918) L15162; `has_deleted_mark` (comparer/finalize.rs:15140) L15170; `Dom::element` (xmllinq/mod.rs:655) L15170; `content_revisions` (comparer/finalize.rs:15189) L15173; `Dom::descendants` (xmllinq/mod.rs:668) L15177; `Dom::new_element` (xmllinq/mod.rs:478) L15180,15181; `rev_el` (comparer/finalize.rs:126) L15182; `Dom::add` (xmllinq/mod.rs:1058) L15183,15184; `Dom::add_first` (xmllinq/mod.rs:1085) L15185
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.insert`, `.is_empty`, `.is_some`, `.iter`, `.to_string`, `.trim`, `HashSet::new`, `Some`, `W::del`, `W::name`, `W::p_pr`, `W::r_pr`
- **Called by:** `align_remaining_ladder_rungs` (comparer/finalize.rs:14886); `coverage_round_two_tests::tabbed_title_mark_matches_original_body_and_excludes_table_only_titles` (comparer/finalize.rs:18067)

#### `content_revisions`  (L15189-15206)
`fn content_revisions(dom: &Dom, paragraph: NodeId) -> (bool, bool, Vec<String>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L15190; `Dom::descendants` (xmllinq/mod.rs:668) L15194; `inside` (comparer/finalize.rs:14934) L15195; `Dom::name_is` (xmllinq/mod.rs:535) L15198,15200; `markup_text` (comparer/finalize.rs:14918) L15202
- **Calls (std/external/unresolved):** `.is_some_and`, `.push`, `.to_string`, `.trim`, `Vec::new`, `W::del`, `W::ins`, `W::p_pr`
- **Called by:** `mark_tabbed_deleted_titles` (comparer/finalize.rs:15150)

#### `spacing_only_change`  (L15208-15219)
`fn spacing_only_change(dom: &Dom, change: NodeId) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L15209,15213; `Dom::name_is` (xmllinq/mod.rs:535) L15214
- **Calls (std/external/unresolved):** `.len`, `Some`, `W::name`, `W::p_pr`
- **Called by:** `unmerge_comment_list_item` (comparer/finalize.rs:15233)

#### `append_empty_ppr_change`  (L15221-15231)
`fn append_empty_ppr_change( dom: &mut Dom, ppr: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** _no doc comment_
- **Calls (crate):** `rev_el` (comparer/finalize.rs:126) L15227; `Dom::new_element` (xmllinq/mod.rs:478) L15228; `Dom::add` (xmllinq/mod.rs:1058) L15229,15230
- **Calls (std/external/unresolved):** `W::p_pr_change`, `W::p_pr`
- **Called by:** `unmerge_comment_list_item` (comparer/finalize.rs:15233); `move_spacing_onto_bare_deletion` (comparer/finalize.rs:15364); `stamp_trailing_numbering` (comparer/finalize.rs:15446)

#### `unmerge_comment_list_item`  (L15233-15289)
`fn unmerge_comment_list_item( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** _no doc comment_
- **Calls (crate):** `body_paragraphs` (comparer/finalize.rs:14913) L15241; `Dom::elements` (xmllinq/mod.rs:640) L15243,15244; `Dom::descendants` (xmllinq/mod.rs:668) L15249; `Dom::name_is` (xmllinq/mod.rs:535) L15252; `Dom::element` (xmllinq/mod.rs:655) L15256,15259,15262,15268; `spacing_only_change` (comparer/finalize.rs:15208) L15265; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L15271,15272; `Dom::add` (xmllinq/mod.rs:1058) L15273,15277,15278,15282,15284,15286; `Dom::remove` (xmllinq/mod.rs:1102) L15274,15285; `Dom::new_element` (xmllinq/mod.rs:478) L15275,15280,15281; `rev_el` (comparer/finalize.rs:126) L15276; `append_empty_ppr_change` (comparer/finalize.rs:15221) L15283; `Dom::add_after_self` (xmllinq/mod.rs:1131) L15287
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_none`, `.is_some`, `.len`, `Some`, `W::del`, `W::ins`, `W::name`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`
- **Called by:** `align_remaining_ladder_rungs` (comparer/finalize.rs:14886); `coverage_round_two_tests::comment_list_unmerge_retains_numbering_and_clones_spacing_to_both_parts` (comparer/finalize.rs:17631); `coverage_round_two_tests::comment_unmerge_rejects_incomplete_or_substantive_property_history` (comparer/finalize.rs:17686)

#### `change_is_empty`  (L15291-15297)
`fn change_is_empty(dom: &Dom, change: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L15292; `Dom::elements` (xmllinq/mod.rs:640) L15295,15296
- **Calls (std/external/unresolved):** `.is_empty`, `.len`, `Some`, `W::p_pr_change`, `W::p_pr`
- **Called by:** `drop_duplicate_normal_style` (comparer/finalize.rs:15299); `coverage_round_two_tests::mixed_spacing_moves_to_blank_and_preserves_existing_history` (comparer/finalize.rs:17280); `coverage_round_two_tests::mixed_numbering_moves_before_last_deletion_mark_or_history` (comparer/finalize.rs:17441); `coverage_round_two_tests::comment_list_unmerge_retains_numbering_and_clones_spacing_to_both_parts` (comparer/finalize.rs:17631); `coverage_round_two_tests::bare_deletion_receives_unique_revised_spacing_with_auto_line_rule` (comparer/finalize.rs:17717); `coverage_round_two_tests::trailing_normal_empty_adopts_source_numbering_and_empty_history` (comparer/finalize.rs:17780); `coverage_round_two_tests::heading_spacing_promotion_requires_prior_live_heading_and_preserves_old_shell` (comparer/finalize.rs:17961); `coverage_round_two_tests::live_heading_history_is_added_only_when_last_history_is_empty` (comparer/finalize.rs:18129)

#### `drop_duplicate_normal_style`  (L15299-15340)
`fn drop_duplicate_normal_style(dom: &mut Dom, root: NodeId)`

- **Does:** _no doc comment_
- **Calls (crate):** `body_paragraphs` (comparer/finalize.rs:14913) L15301; `Dom::element` (xmllinq/mod.rs:655) L15303,15332,15334; `Dom::elements` (xmllinq/mod.rs:640) L15306,15317,15320; `Dom::name_is` (xmllinq/mod.rs:535) L15307,15313; `change_is_empty` (comparer/finalize.rs:15291) L15317; `markup_text` (comparer/finalize.rs:14918) L15324,15330; `has_deleted_mark` (comparer/finalize.rs:15140) L15331; `Dom::remove` (xmllinq/mod.rs:1102) L15337
- **Calls (std/external/unresolved):** `.any`, `.copied`, `.enumerate`, `.find`, `.is_empty`, `.is_none_or`, `.is_none`, `.iter`, `.len`, `.trim`, `Some`, `W::del`, `W::ins`, `W::name`, `W::p_pr_change`, `W::p_pr`
- **Called by:** `align_remaining_ladder_rungs` (comparer/finalize.rs:14886); `coverage_round_two_tests::duplicate_deleted_default_style_drops_only_after_matching_marked_text` (comparer/finalize.rs:18105)

#### `spacing_signature`  (L15342-15352)
`fn spacing_signature(dom: &Dom, paragraph: NodeId) -> Option<Vec<(String, String)>>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L15343,15344; `Dom::attributes` (xmllinq/mod.rs:885) L15345; `XName::local_name` (xmllinq/mod.rs:208) (by name) L15348
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.is_empty`, `.map`, `.sort`, `.to_string`, `Some`, `W::name`, `W::p_pr`
- **Called by:** `move_spacing_onto_bare_deletion` (comparer/finalize.rs:15364)

#### `direct_spacing`  (L15354-15357)
`fn direct_spacing(dom: &Dom, paragraph: NodeId) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L15355,15356
- **Calls (std/external/unresolved):** `W::name`, `W::p_pr`
- **Called by:** `move_spacing_onto_bare_deletion` (comparer/finalize.rs:15364)

#### `move_spacing_onto_bare_deletion`  (L15364-15444)
`fn move_spacing_onto_bare_deletion( dom: &mut Dom, root: NodeId, revised: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** _no doc comment_
- **Calls (crate):** `body_paragraphs` (comparer/finalize.rs:14913) L15372,15398; `spacing_signature` (comparer/finalize.rs:15342) L15373,15413; `markup_text` (comparer/finalize.rs:14918) L15382,15406,15425; `direct_spacing` (comparer/finalize.rs:15354) L15387; `Dom::elements` (xmllinq/mod.rs:640) L15401,15402,15420,15423; `Dom::element` (xmllinq/mod.rs:655) L15426; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L15434; `Dom::attribute` (xmllinq/mod.rs:874) L15437; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L15438; `Dom::new_element` (xmllinq/mod.rs:478) L15440; `Dom::add` (xmllinq/mod.rs:1058) L15441; `append_empty_ppr_change` (comparer/finalize.rs:15221) L15442; `Dom::add_first` (xmllinq/mod.rs:1085) L15443
- **Calls (std/external/unresolved):** `.any`, `.as_ref`, `.entry`, `.enumerate`, `.get`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.len`, `.or_default`, `.push`, `.to_string`, `.trim`, `HashMap::new`, `Some`, `Vec::new`, `W::del`, `W::ins`, `W::name`, `W::p_pr`
- **Called by:** `align_remaining_ladder_rungs` (comparer/finalize.rs:14886); `coverage_round_two_tests::bare_deletion_receives_unique_revised_spacing_with_auto_line_rule` (comparer/finalize.rs:17717); `coverage_round_two_tests::bare_deletion_spacing_rejects_ambiguity_reverse_order_and_equal_spacing` (comparer/finalize.rs:17755)

#### `stamp_trailing_numbering`  (L15446-15493)
`fn stamp_trailing_numbering( dom: &mut Dom, root: NodeId, revised: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** _no doc comment_
- **Calls (crate):** `body_paragraphs` (comparer/finalize.rs:14913) L15455,15464,15483; `Dom::element` (xmllinq/mod.rs:655) L15458,15459,15471; `markup_text` (comparer/finalize.rs:14918) L15468,15485; `Dom::elements` (xmllinq/mod.rs:640) L15474; `Dom::name_is` (xmllinq/mod.rs:535) L15475; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L15480; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L15489; `Dom::remove` (xmllinq/mod.rs:1102) L15490; `Dom::add_first` (xmllinq/mod.rs:1085) L15491; `append_empty_ppr_change` (comparer/finalize.rs:15221) L15492
- **Calls (std/external/unresolved):** `.and_then`, `.find_map`, `.into_iter`, `.is_empty`, `.is_some_and`, `.last`, `.len`, `.trim`, `Some`, `W::name`, `W::p_pr`, `W::val`
- **Macros:** `matches!`
- **Called by:** `align_remaining_ladder_rungs` (comparer/finalize.rs:14886); `coverage_round_two_tests::trailing_normal_empty_adopts_source_numbering_and_empty_history` (comparer/finalize.rs:17780); `coverage_round_two_tests::trailing_numbering_preserves_nonempty_custom_and_source_final_blanks` (comparer/finalize.rs:17813)

## Test functions (493)

- `residual_short_label_zip` (L6918): M369 — greedy order-preserving zip of residual short-label pure-I paragraphs (e.g.
- `fold_short_label_ins_into_del` (L7048): Fold pure-I short label into pure-D list residual.
- `short_label_fixture_settings` (L7178): M375 — residual short-label MIX free-meshes the shared trailing label as unrevised EQ.
- `mesh_short_label_shared_eq` (L7194): _no doc comment_
- `fold_midstream_demo_title_into_numbered_heading` (L12562): M143 — fold a mid-stream pure-D short "Demo" title among the pure-I body of a long revised document into the first numbered heading instead of leaving it as a free pure-D.
- `coverage_tests::xml` (L15499): _no doc comment_
- `coverage_tests::paragraphs` (L15509): _no doc comment_
- `coverage_tests::text` (L15513): _no doc comment_
- `coverage_tests::revised` (L15520): _no doc comment_
- `coverage_tests::assert_unchanged` (L15527): _no doc comment_
- `coverage_tests::replacement_classification_is_transparent_only_for_hyperlinks` (L15535): _no doc comment_
- `coverage_tests::row_move_classification_requires_matching_structural_paragraph_marks` (L15607): _no doc comment_
- `coverage_tests::pure_and_mixed_paragraph_classification_preserves_live_content` (L15639): _no doc comment_
- `coverage_tests::classification_cache_hits_and_resets_agree_with_uncached_results` (L15676): _no doc comment_
- `<Reset as Drop>::drop` (L15679): _no doc comment_
- `coverage_tests::section_and_list_residual_label_boundaries` (L15699): _no doc comment_
- `coverage_tests::token_splitting_preserves_separators_and_original_case` (L15722): _no doc comment_
- `coverage_tests::relatedness_and_short_title_gates_use_tokens_not_substrings` (L15763): _no doc comment_
- `coverage_tests::deleted_text_conversion_stops_at_nested_revision_owners` (L15801): _no doc comment_
- `coverage_tests::field_completeness_requires_balanced_order_not_just_counts` (L15813): _no doc comment_
- `coverage_tests::revision_wrapper_requires_consistent_hyperlink_stamps` (L15837): _no doc comment_
- `coverage_tests::content_controls_unwrap_deepest_first_and_keep_text_order` (L15875): _no doc comment_
- `coverage_tests::residual_control_unwrap_keeps_equal_form_controls` (L15890): _no doc comment_
- `coverage_tests::alternate_content_selects_first_bound_choice_or_fallback` (L15905): _no doc comment_
- `coverage_tests::fold_gate` (L15958): _no doc comment_
- `coverage_tests::document_scale_fold_respects_absolute_ratio_and_fraction_boundaries` (L15992): _no doc comment_
- `coverage_tests::document_scale_list_and_cover_special_cases` (L16053): _no doc comment_
- `coverage_tests::sole_pair_relatedness_boundary_is_inclusive` (L16210): _no doc comment_
- `coverage_tests::cleanup_spacing_removes_only_redundant_mixed_history` (L16232): _no doc comment_
- `coverage_tests::cleanup_default_alignment_keeps_real_layout_history` (L16272): _no doc comment_
- `coverage_tests::cleanup_deleted_spacing_promotes_mid_body_but_preserves_last_history` (L16306): _no doc comment_
- `coverage_tests::short_label_free_mesh_preserves_deleted_prefix_and_inserted_whitespace` (L16368): _no doc comment_
- `coverage_tests::residual_label_zip_adopts_deleted_list_layout_and_moves_empty_spacer` (L16416): _no doc comment_
- `coverage_tests::short_list_label_folds_only_into_empty_deleted_mark_with_deleted_tail` (L16446): _no doc comment_
- `coverage_tests::bookended_free_mesh_rebuilds_word_order_and_preserves_run_properties` (L16496): _no doc comment_
- `coverage_tests::bookended_free_mesh_rejects_ambiguous_and_nontext_anchors` (L16530): _no doc comment_
- `coverage_tests::trailing_spaced_empty_requires_insert_dominance_and_deleted_predecessor` (L16578): _no doc comment_
- `coverage_tests::promoted_list_numbering_uses_inserted_id_and_deleted_level` (L16629): _no doc comment_
- `coverage_tests::field_fold_rejects_crossing_but_allows_balanced_deleted_fields` (L16705): _no doc comment_
- `coverage_round_two_tests::xml` (L16751): _no doc comment_
- `coverage_round_two_tests::ps` (L16762): _no doc comment_
- `coverage_round_two_tests::revision` (L16766): _no doc comment_
- `coverage_round_two_tests::pure` (L16773): _no doc comment_
- `coverage_round_two_tests::mixed` (L16780): _no doc comment_
- `coverage_round_two_tests::settings` (L16788): _no doc comment_
- `coverage_round_two_tests::segments` (L16796): _no doc comment_
- `coverage_round_two_tests::stamped` (L16815): _no doc comment_
- `coverage_round_two_tests::unchanged` (L16824): _no doc comment_
- `coverage_round_two_tests::wholesale_mesh_emits_exact_shared_words_and_styled_residuals` (L16832): _no doc comment_
- `coverage_round_two_tests::wholesale_mesh_preserves_each_authored_terminal_whitespace_and_format` (L16891): _no doc comment_
- `coverage_round_two_tests::wholesale_mesh_preserves_each_authored_terminal_whitespace_and_format::{fn source_projection}` (L16892): _no doc comment_
- `coverage_round_two_tests::wholesale_mesh_without_terminal_whitespace_keeps_exact_supported_word_streams` (L16942): _no doc comment_
- `coverage_round_two_tests::wholesale_mesh_without_terminal_whitespace_keeps_exact_supported_word_streams::{fn owned}` (L16943): _no doc comment_
- `coverage_round_two_tests::wholesale_mesh_keeps_low_overlap_and_nontext_changes_verbatim` (L17008): _no doc comment_
- `coverage_round_two_tests::lcs_rebuild_covers_equal_prefixes_and_both_unmatched_tails` (L17052): _no doc comment_
- `coverage_round_two_tests::title_page_relocation_preserves_six_spacers_and_moves_only_last` (L17092): _no doc comment_
- `coverage_round_two_tests::title_page_relocation_respects_cover_and_layout_boundaries` (L17141): _no doc comment_
- `coverage_round_two_tests::short_head_split_preserves_title_style_and_deleted_wrapper_identity` (L17176): _no doc comment_
- `coverage_round_two_tests::short_head_split_rejects_shared_words_marks_and_equal_interleaving` (L17215): _no doc comment_
- `coverage_round_two_tests::stranded_mark_is_cloned_without_removing_empty_shell` (L17239): _no doc comment_
- `coverage_round_two_tests::stranded_mark_rejects_layout_or_body_content_in_shell` (L17262): _no doc comment_
- `coverage_round_two_tests::mixed_spacing_moves_to_blank_and_preserves_existing_history` (L17280): _no doc comment_
- `coverage_round_two_tests::mixed_spacing_does_not_cross_live_blocks_or_fill_textual_paragraphs` (L17340): _no doc comment_
- `coverage_round_two_tests::heading_spacing_parks_on_last_short_deleted_residual` (L17359): _no doc comment_
- `coverage_round_two_tests::spacing_park_rejects_count_length_and_heading_thresholds` (L17404): _no doc comment_
- `coverage_round_two_tests::mixed_numbering_moves_before_last_deletion_mark_or_history` (L17441): _no doc comment_
- `coverage_round_two_tests::numbering_park_preserves_short_long_and_already_numbered_runs` (L17492): _no doc comment_
- `coverage_round_two_tests::whitespace_fold_adopts_list_layout_and_replaces_inserted_mark` (L17509): _no doc comment_
- `coverage_round_two_tests::whitespace_field_residue_never_inherits_heading_style` (L17554): _no doc comment_
- `coverage_round_two_tests::whitespace_fold_keeps_layout_payloads_long_prose_and_cover_spacers` (L17581): _no doc comment_
- `coverage_round_two_tests::comment_list_unmerge_retains_numbering_and_clones_spacing_to_both_parts` (L17631): _no doc comment_
- `coverage_round_two_tests::comment_unmerge_rejects_incomplete_or_substantive_property_history` (L17686): _no doc comment_
- `coverage_round_two_tests::source` (L17708): _no doc comment_
- `coverage_round_two_tests::bare_deletion_receives_unique_revised_spacing_with_auto_line_rule` (L17717): _no doc comment_
- `coverage_round_two_tests::bare_deletion_spacing_rejects_ambiguity_reverse_order_and_equal_spacing` (L17755): _no doc comment_
- `coverage_round_two_tests::trailing_normal_empty_adopts_source_numbering_and_empty_history` (L17780): _no doc comment_
- `coverage_round_two_tests::trailing_numbering_preserves_nonempty_custom_and_source_final_blanks` (L17813): _no doc comment_
- `coverage_round_two_tests::insertion_split_handles_unicode_run_boundaries_and_multi_leaf_runs` (L17838): _no doc comment_
- `coverage_round_two_tests::carried_insert_and_delete_split_preserves_neighbors_and_attribution` (L17891): _no doc comment_
- `coverage_round_two_tests::heading_spacing_promotion_requires_prior_live_heading_and_preserves_old_shell` (L17961): _no doc comment_
- `coverage_round_two_tests::old_body_alignment_promotes_only_for_aligned_revised_paragraphs` (L17994): _no doc comment_
- `coverage_round_two_tests::leading_deleted_echo_is_removed_only_with_one_or_two_tokens` (L18040): _no doc comment_
- `coverage_round_two_tests::tabbed_title_mark_matches_original_body_and_excludes_table_only_titles` (L18067): _no doc comment_
- `coverage_round_two_tests::duplicate_deleted_default_style_drops_only_after_matching_marked_text` (L18105): _no doc comment_
- `coverage_round_two_tests::live_heading_history_is_added_only_when_last_history_is_empty` (L18129): _no doc comment_
- `coverage_round_three_tests::document` (L18180): Pure in-memory WordprocessingML fixtures. _(from a code comment)_
- `coverage_round_three_tests::rev` (L18192): _no doc comment_
- `coverage_round_three_tests::para` (L18199): _no doc comment_
- `coverage_round_three_tests::bare` (L18206): _no doc comment_
- `coverage_round_three_tests::paragraphs` (L18210): _no doc comment_
- `coverage_round_three_tests::body_segments` (L18214): _no doc comment_
- `coverage_round_three_tests::expect_segments` (L18227): _no doc comment_
- `coverage_round_three_tests::props_xml` (L18237): _no doc comment_
- `coverage_round_three_tests::settings` (L18242): _no doc comment_
- `coverage_round_three_tests::same_after_merge` (L18250): _no doc comment_
- `coverage_round_three_tests::paired_normalization_preserves_prior_pilcrow_provenance_and_only_unmarks_its_own_pair` (L18258): _no doc comment_
- `coverage_round_three_tests::related_one_for_one_adopts_exact_structural_properties_and_inserted_attributes` (L18333): _no doc comment_
- `coverage_round_three_tests::field_replacement_keeps_bare_properties_instead_of_deleted_heading` (L18383): _no doc comment_
- `coverage_round_three_tests::bare_related_pair_does_not_invent_paragraph_properties` (L18403): _no doc comment_
- `coverage_round_three_tests::deleted_then_multiple_inserted_folds_only_last_and_adopts_heading` (L18418): _no doc comment_
- `coverage_round_three_tests::last_inserted_list_adopts_deleted_listparagraph_and_numbering` (L18456): _no doc comment_
- `coverage_round_three_tests::alignment_only_carrier_adopts_deleted_spacing_exactly` (L18486): _no doc comment_
- `coverage_round_three_tests::paired_body_tail_stays_separate_but_nested_stories_still_fold` (L18523): _no doc comment_
- `coverage_round_three_tests::shared_title_head_junction_precedes_remaining_inserted_stream` (L18548): _no doc comment_
- `coverage_round_three_tests::story_tail_fusion_crosses_deleted_table_and_keeps_deleted_layout` (L18581): _no doc comment_
- `coverage_round_three_tests::story_tail_fusion_stops_at_page_breaks_section_breaks_and_live_tables` (L18623): _no doc comment_
- `coverage_round_three_tests::multiple_mark_only_deletions_fold_once_and_preserve_remaining_empty` (L18659): _no doc comment_
- `coverage_round_three_tests::empty_heading_residual_adopts_heading_for_last_list_carrier` (L18689): _no doc comment_
- `coverage_round_three_tests::carried_foreign_insertions_reorder_without_becoming_replacement_deletions` (L18709): _no doc comment_
- `coverage_round_three_tests::unmatched_field_blocks_shared_title_head_junction` (L18726): _no doc comment_
- `coverage_round_three_tests::digits_and_short_list_residuals_keep_their_own_inserted_paragraph` (L18754): _no doc comment_
- `coverage_round_three_tests::long_prose_does_not_inherit_multiword_list_geometry` (L18767): _no doc comment_
- `coverage_round_three_tests::runless_empty_carrier_preserves_cover_spacers_and_long_prose` (L18794): _no doc comment_
- `coverage_round_three_tests::trailing_whitespace_spacing_folds_only_after_contentful_insertions` (L18830): _no doc comment_
- `coverage_round_three_tests::whitespace_carriers_without_mark_shell_adopt_deleted_pilcrow` (L18868): _no doc comment_
- `coverage_round_three_tests::fragmented_two_character_label_meshes_and_retains_whitespace_and_punctuation` (L18890): _no doc comment_
- `coverage_round_three_tests::label_mesh_removes_empty_inserted_wrapper_and_keeps_deleted_list_layout` (L18909): _no doc comment_
- `coverage_round_three_tests::positional_label_zip_keeps_distinct_labels_and_existing_geometry` (L18948): _no doc comment_
- `coverage_round_three_tests::nested_list_interleave_preserves_exact_node_order_and_trailing_anchors` (L18973): _no doc comment_
- `coverage_round_three_tests::deleted_textbox_story_marks_runs_and_pilcrows_in_schema_order` (L19020): _no doc comment_
- `coverage_round_three_tests::nested_deleted_textboxes_use_their_own_anchor_and_preserve_existing_marks` (L19077): _no doc comment_
- `coverage_round_three_tests::deleted_textbox_missing_attribution_uses_pinned_settings` (L19112): _no doc comment_
- `coverage_round_three_tests::bordered_table_geometry_uses_row_revision_source_and_schema_order` (L19134): _no doc comment_
- `coverage_round_three_tests::table_geometry_preserves_explicit_values_and_fills_only_missing_properties` (L19181): _no doc comment_
- `coverage_round_three_tests::large_wholesale_replacement_keeps_all_paragraphs_at_scale_boundary` (L19217): _no doc comment_
- `coverage_round_three_tests::short_wholesale_lists_keep_both_sides_when_old_count_is_twice_new` (L19233): _no doc comment_
- `coverage_round_three_tests::inserted_break_never_folds_into_deleted_drawing_pict_or_object` (L19247): _no doc comment_
- `coverage_round_three_tests::revision_rows_preserve_cell_geometry_and_are_stamped_once` (L19263): _no doc comment_
- `coverage_round_next_tests::document` (L19330): Coverage: lines and branches await the parent's instrumented run. _(from a code comment)_
- `coverage_round_next_tests::revision` (L19341): _no doc comment_
- `coverage_round_next_tests::pure` (L19348): _no doc comment_
- `coverage_round_next_tests::marked` (L19352): _no doc comment_
- `coverage_round_next_tests::settings` (L19359): _no doc comment_
- `coverage_round_next_tests::same_tree` (L19369): Compare the complete tree, including every attribute, text leaf and child position. _(from a code comment)_
- `coverage_round_next_tests::expect_body` (L19399): _no doc comment_
- `coverage_round_next_tests::unchanged` (L19404): _no doc comment_
- `coverage_round_next_tests::digit_title_split_moves_all_leading_fragments_and_keeps_original_properties` (L19412): _no doc comment_
- `coverage_round_next_tests::digit_split_rejects_nonascii_empty_and_later_insertions` (L19432): _no doc comment_
- `coverage_round_next_tests::trailing_insertion_peel_has_inclusive_token_bounds_and_keeps_punctuation` (L19463): _no doc comment_
- `coverage_round_next_tests::trailing_insertion_peel_rejects_exact_guard_boundaries` (L19486): _no doc comment_
- `coverage_round_next_tests::trailing_deletion_peel_truth_table_for_shared_long_and_copula_paths` (L19533): _no doc comment_
- `coverage_round_next_tests::trailing_deletion_peel_truth_table_for_shared_long_and_copula_paths::{fn projected_body}` (L19534): _no doc comment_
- `coverage_round_next_tests::short_deletion_restore_checks_filename_and_alnum_length_boundaries` (L19635): _no doc comment_
- `coverage_round_next_tests::demo_fold_strips_only_revised_pilcrow_and_uses_nearest_numbered_heading` (L19673): _no doc comment_
- `coverage_round_next_tests::demo_title_fold_history_preserves_complete_old_layout_and_current_attribution` (L19711): _no doc comment_
- `coverage_round_next_tests::demo_fold_heading_text_limits_and_table_barriers_are_exact` (L19756): _no doc comment_
- `coverage_round_next_tests::title_spacers_are_kept_once_and_prose_spacers_are_removed` (L19815): _no doc comment_
- `coverage_round_next_tests::list_contamination_cleanup_keeps_real_lists_and_alignment` (L19844): _no doc comment_
- `coverage_round_next_tests::list_snug_spacing_requires_three_identical_short_tokens_and_respects_schema_tail` (L19872): _no doc comment_
- `coverage_round_next_tests::trailing_blank_spacing_records_history_only_when_source_differs` (L19901): _no doc comment_
- `coverage_round_next_tests::table_chrome_adds_look_before_history_and_handles_all_row_positions` (L19944): _no doc comment_
- `coverage_round_next_tests::comment_chrome_fills_each_missing_property_independently_and_ignores_orphans` (L19966): _no doc comment_
- `coverage_round_next_tests::deleted_textbox_attribution_is_inherited_per_attribute_and_runs_join_across_text` (L20010): _no doc comment_
- `coverage_round_next_tests::paragraph_deletion_mark_inserts_before_each_schema_tail_and_preserves_format` (L20042): _no doc comment_
- `coverage_round_next_tests::structural_property_classifiers_ignore_history_and_require_one_alignment` (L20063): _no doc comment_
- `coverage_round_next_tests::conjoin_property_selection_retains_format_and_does_not_duplicate_history` (L20088): _no doc comment_
- `coverage_round_next_tests::moved_empty_field_becomes_a_complex_field_in_the_move` (L20144): An empty field is a whole atom (no run), so the run-level status pass never wrapped it: a deleted `PAGE` stayed live in its deleted paragraph and Word 16.115 hung on the footer (b15, 0.12.0 sample).
- `coverage_round_next_tests::deleted_or_inserted_empty_field_becomes_a_complex_field_in_the_revision` (L20183): _no doc comment_
- `coverage_round_next_tests::default_spacing_strips_only_the_selected_source_default_and_exact_shape` (L20215): _no doc comment_
- `coverage_round_next_tests::deleted_heading_spacing_threshold_keeps_missing_metrics_and_real_styles` (L20250): _no doc comment_
- `coverage_round_next_tests::unrecorded_spacing_guards_preserve_all_explicit_vertical_metrics` (L20277): _no doc comment_
- `coverage_round_next_tests::deleted_history_flattening_uses_both_field_and_text_fingerprint` (L20324): _no doc comment_
- `coverage_round_next_tests::predelete_conversion_missing_attribution_defaults_without_changing_neighbor_runs` (L20375): _no doc comment_
- `coverage_round_next_tests::short_label_mesh_rejects_empty_oversized_and_mismatching_labels_verbatim` (L20413): _no doc comment_
- `coverage_round_next_tests::label_fold_creates_missing_properties_without_inventing_source_geometry` (L20436): _no doc comment_
- `coverage_round_next_tests::for_word_peel_preserves_run_format_and_uses_equal_format_for_shared_connector` (L20463): _no doc comment_
- `coverage_round_next_tests::for_word_guard_matrix_requires_residual_and_leading_deleted_carrier` (L20488): _no doc comment_
- `coverage_round_next_tests::borrowed_period_moves_to_finished_side_and_finishes_the_continuation` (L20519): _no doc comment_
- `coverage_round_next_tests::compatibility_defaults_keep_nonzero_vertical_margins_and_existing_look` (L20560): _no doc comment_
- `coverage_round_next_tests::shared_title_mesh_rejects_author_mismatch_repeats_and_nonfinal_shared_tokens` (L20581): _no doc comment_
- `coverage_round_next_tests::revised_complex_deletion_stays_pending_while_original_restores_nested_text` (L20630): _no doc comment_
- `coverage_round_next_tests::strict_shape_repair_stops_at_nested_drawing_and_keeps_bad_percent_text` (L20645): _no doc comment_
- `coverage_round_next_tests::block_reordering_requires_uniform_revision_rows_and_nonempty_sdt_content` (L20663): _no doc comment_
- `fragmented_label_regression_tests::projection` (L20710): _no doc comment_
- `fragmented_label_regression_tests::shared_label_fragments_preserve_both_revision_projections` (L20725): _no doc comment_
- `fragmented_label_regression_tests::unsupported_fragment_tail_cannot_partially_erase_a_deletion` (L20747): _no doc comment_
- `fragmented_label_regression_tests::label_mesh_preserves_punctuation_styles_and_nontext_payloads` (L20758): _no doc comment_
- `fragmented_label_regression_tests::interleaved_revision_blocks_preserve_insertion_order` (L20789): _no doc comment_
- `fragmented_label_regression_tests::shared_label_mesh_keeps_nested_revision_hyperlink_and_control_ownership` (L20805): _no doc comment_
- `fragmented_label_regression_tests::short_label_mesh_keeps_unrelated_empty_run_properties` (L20829): _no doc comment_
- `fragmented_label_regression_tests::paragraph_mark_revision_does_not_block_plain_shared_label_meshing` (L20845): _no doc comment_
- `deep_production_path_tests::document` (L20871): _no doc comment_
- `deep_production_path_tests::revision` (L20881): _no doc comment_
- `deep_production_path_tests::para` (L20888): _no doc comment_
- `deep_production_path_tests::empty` (L20895): _no doc comment_
- `deep_production_path_tests::live` (L20901): _no doc comment_
- `deep_production_path_tests::blocks` (L20905): _no doc comment_
- `deep_production_path_tests::projection` (L20914): Independent source projections: paragraph properties are not body text, and a rejected/accepted revision excludes its entire nested subtree. _(from a code comment)_
- `deep_production_path_tests::views` (L20928): _no doc comment_
- `deep_production_path_tests::body_revisions` (L20932): _no doc comment_
- `deep_production_path_tests::properties` (L20947): _no doc comment_
- `deep_production_path_tests::segments` (L20951): _no doc comment_
- `deep_production_path_tests::expect_segments` (L20964): _no doc comment_
- `deep_production_path_tests::merge` (L20974): _no doc comment_
- `deep_production_path_tests::unchanged` (L21000): _no doc comment_
- `deep_production_path_tests::legal_and_memo_midsplices_keep_each_source_paragraph_in_order` (L21010): _no doc comment_
- `deep_production_path_tests::nested_level_parsing_controls_interleave_without_changing_markup` (L21047): _no doc comment_
- `deep_production_path_tests::legal_midsplice_count_boundaries_reorder_outside_the_guard` (L21084): _no doc comment_
- `deep_production_path_tests::prior_nested_cluster_keeps_short_labels_but_allows_a_longer_carrier` (L21108): _no doc comment_
- `deep_production_path_tests::long_content_cannot_reach_back_across_an_empty_tail_without_a_head_match` (L21141): _no doc comment_
- `deep_production_path_tests::math_shells_are_fold_carriers_and_plain_layout_empties_are_preserved` (L21209): _no doc comment_
- `deep_production_path_tests::inserted_cover_tail_and_short_residual_exceptions_are_distinct` (L21244): _no doc comment_
- `deep_production_path_tests::deletion_first_short_residual_and_field_carrier_keep_source_properties` (L21279): _no doc comment_
- `deep_production_path_tests::empty_deleted_tail_is_not_consumed_after_long_inserted_streams` (L21320): _no doc comment_
- `deep_production_path_tests::short_label_long_prose_gate_distinguishes_plain_demo_and_ooxml_intros` (L21333): _no doc comment_
- `deep_production_path_tests::styled_inserted_titles_keep_unrelated_short_base_titles_except_demos` (L21391): _no doc comment_
- `deep_production_path_tests::long_demo_stream_gate_has_an_exact_ten_paragraph_boundary` (L21420): _no doc comment_
- `deep_production_path_tests::heading_skip_is_limited_to_short_multitoken_headings_and_nonlist_carriers` (L21436): _no doc comment_
- `deep_production_path_tests::list_carriers_override_the_long_prose_list_skip_and_retain_their_geometry` (L21482): _no doc comment_
- `deep_production_path_tests::wholesale_scale_gate_preserves_large_math_and_drawing_replacements` (L21527): _no doc comment_
- `deep_production_path_tests::midstory_checklist_cell_guard_distinguishes_titles_at_a_weak_head_junction` (L21577): _no doc comment_
- `deep_production_path_tests::whitespace_fold_keeps_a_whitespace_only_deleted_source_paragraph` (L21604): _no doc comment_
- `deep_production_path_tests::whitespace_mark_shell_variants_keep_existing_format_without_inventing_a_mark` (L21612): _no doc comment_
- `deep_production_path_tests::whitespace_spacing_guard_searches_past_noninserted_predecessors` (L21660): _no doc comment_
- `deep_production_path_tests::spacer_context` (L21687): _no doc comment_
- `deep_production_path_tests::table_title_spacers_stamp_only_new_marks_and_keep_existing_empty_properties` (L21697): _no doc comment_
- `deep_production_path_tests::title_spacers_require_unique_revised_two_blank_pretable_context` (L21795): _no doc comment_
- `deep_production_path_tests::restored_title_spacers_preserve_each_revised_blank_template` (L21887): _no doc comment_
- `deep_production_path_tests::title_spacer_context_declines_owned_nonlayout_payload` (L21931): _no doc comment_
- `deep_production_path_tests::title_spacer_injection_requires_a_paired_table_and_a_truly_blank_separator` (L21977): _no doc comment_
- `deep_production_path_tests::three_paragraph_rotation_trades_full_properties_and_preserves_each_revision` (L22017): _no doc comment_
- `deep_production_path_tests::rotation_rejects_interleaved_first_runs_and_nonfinal_middle_deletions` (L22075): _no doc comment_
- `deep_production_path_tests::leading_subtitle_fold_moves_multiple_wrappers_and_retains_deleted_layout` (L22127): _no doc comment_
- `deep_production_path_tests::leading_subtitle_gate_requires_disjoint_significant_tokens_and_eight_token_cap` (L22166): _no doc comment_
- `deep_production_path_tests::shared_title_mesh_executes_wholesale_and_confetti_rebuilds_with_exact_views` (L22204): _no doc comment_
- `deep_production_path_tests::bookended_mesh_handles_boundary_anchors_and_the_forty_token_limit` (L22279): _no doc comment_
- `deep_production_path_tests::connector_peel_preserves_both_sources_and_falls_back_to_inserted_run_format` (L22331): _no doc comment_
- `deep_production_path_tests::splice_fragment` (L22386): _no doc comment_
- `deep_production_path_tests::short_residual_splice_prefers_tip_then_toc_and_moves_only_the_short_numbered_title` (L22404): _no doc comment_
- `deep_production_path_tests::short_title_splice_crosses_only_tables_whose_every_row_is_structurally_inserted` (L22442): _no doc comment_
- `deep_production_path_tests::short_title_splice_crosses_only_tables_whose_every_row_is_structurally_inserted::{fn projected}` (L22443): _no doc comment_
- `deep_production_path_tests::short_residual_splice_majority_and_length_boundaries_do_not_invent_reordering` (L22509): _no doc comment_
- `deep_production_path_tests::revised_internal_hyperlink_fields_preserve_result_text_and_revision_attribution` (L22560): _no doc comment_
- `deep_production_path_tests::deletion_peel_keeps_retained_source_boundary_and_inserted_continuations` (L22616): _no doc comment_
- `deep_production_path_tests::deletion_peel_keeps_retained_source_boundary_and_inserted_continuations::{fn paragraphs}` (L22617): _no doc comment_
- `deep_production_path_tests::final_deletion_peel_transfers_boundary_only_with_equal_properties_and_text_payload` (L22682): _no doc comment_
- `deep_production_path_tests::final_deletion_peel_transfers_boundary_only_with_equal_properties_and_text_payload::{fn projected}` (L22683): _no doc comment_
- `deep_production_path_tests::deletion_peel_keeps_nontext_only_original_anchors_in_their_source_paragraph` (L22773): _no doc comment_
- `deep_production_path_tests::deletion_peel_keeps_nontext_only_original_anchors_in_their_source_paragraph::{fn projected}` (L22774): _no doc comment_
- `deep_production_path_tests::deletion_peel_keeps_punctuation_and_whitespace_only_original_anchors` (L22832): _no doc comment_
- `deep_production_path_tests::deletion_peel_keeps_punctuation_and_whitespace_only_original_anchors::{fn projection}` (L22833): _no doc comment_
- `deep_production_path_tests::deletion_peel_keeps_original_closing_boundary_when_all_original_words_are_deleted` (L22879): _no doc comment_
- `deep_production_path_tests::deletion_peel_keeps_original_closing_boundary_when_all_original_words_are_deleted::{fn projection}` (L22880): _no doc comment_
- `deep_production_path_tests::wholesale_deleted_body_peel_transfers_its_retained_closing_boundary` (L22924): _no doc comment_
- `deep_production_path_tests::wholesale_deleted_body_peel_transfers_its_retained_closing_boundary::{fn projection}` (L22925): _no doc comment_
- `deep_production_path_tests::deletion_peel_does_not_move_an_original_suffix_past_equal_nonperiod_text` (L22977): _no doc comment_
- `deep_production_path_tests::deletion_peel_does_not_move_an_original_suffix_past_equal_nonperiod_text::{fn projection}` (L22978): _no doc comment_
- `deep_production_path_tests::borrowed_period_peel_keeps_the_original_period_run_formatting` (L23024): _no doc comment_
- `deep_production_path_tests::borrowed_period_peel_keeps_the_original_period_run_formatting::{fn projection}` (L23025): _no doc comment_
- `deep_production_path_tests::same_format_borrowed_period_peel_keeps_literal_text_and_character_properties` (L23061): _no doc comment_
- `deep_production_path_tests::same_format_borrowed_period_peel_keeps_literal_text_and_character_properties::{fn projection}` (L23062): _no doc comment_
- `deep_production_path_tests::nested_interleave_keeps_mark_only_items_and_defaults_invalid_terminal_levels` (L23121): _no doc comment_
- `deep_production_path_tests::nested_interleave_uniform_tokens_are_case_insensitive_and_late_mismatches_reorder` (L23164): _no doc comment_
- `deep_production_path_tests::nested_interleave_needs_a_complete_nested_cluster_followed_by_a_top_level_item` (L23194): _no doc comment_
- `deep_production_path_tests::final_story_fusion_keeps_the_original_deleted_tail_and_its_metadata` (L23215): _no doc comment_
- `deep_production_path_tests::explicit_page_breaks_block_final_story_fusion_on_either_source_side` (L23266): _no doc comment_
- `deep_production_path_tests::numbered_wholesale_gate_requires_the_last_new_item_and_half_of_old_items_numbered` (L23307): _no doc comment_
- `deep_production_path_tests::punctuation_only_items_are_contentful_and_can_supply_empty_token_relatedness` (L23344): _no doc comment_
- `deep_production_path_tests::late_tip_overrides_earlier_toc_and_numbered_splice_anchors` (L23371): _no doc comment_
- `deep_production_path_tests::plain_label_mesh_may_share_across_multiple_contiguous_revision_wrappers` (L23408): _no doc comment_
- `deep_production_path_tests::suffix_only_bookend_mesh_uses_pinned_defaults_and_keeps_the_original_suffix` (L23441): _no doc comment_
- `deep_production_path_tests::complete_field_carriers_keep_title_deletions_as_mark_only_properties` (L23480): _no doc comment_
- `row_move_ownership_regressions::moved_row` (L23520): _no doc comment_
- `row_move_ownership_regressions::moving_cell_content_without_paragraph_ownership_keeps_existing_rows` (L23530): _no doc comment_
- `row_move_ownership_regressions::moved_content_with_matching_paragraph_mark_tracks_row_lifecycle` (L23574): _no doc comment_
- `parked_paragraph_history_regressions::parked_layout_restores_properties_that_remain_live_when_rejected` (L23595): _no doc comment_
- `closing_layout_history_regressions::already_revised_live_closing_layout_still_records_original_properties` (L23657): _no doc comment_
- `split_paragraph_boundary_regressions::mixed_inserted_boundary_joins_original_text_and_closing_layout_on_reject` (L23730): _no doc comment_
- `row_nontext_ownership_regressions::document` (L23793): _no doc comment_
- `row_nontext_ownership_regressions::unchanged_nontext_payload_prevents_a_moved_neighbor_from_owning_its_row` (L23804): _no doc comment_
- `row_nontext_ownership_regressions::package` (L23845): _no doc comment_
- `row_nontext_ownership_regressions::row_payload` (L23860): _no doc comment_
- `row_nontext_ownership_regressions::public_compare_keeps_unchanged_tab_and_break_cells_when_text_moves_into_their_row` (L23873): _no doc comment_
- `short_label_format_history_tests::document` (L23942): _no doc comment_
- `short_label_format_history_tests::properties` (L23952): _no doc comment_
- `short_label_format_history_tests::resolved` (L23962): _no doc comment_
- `short_label_format_history_tests::shared_label_tracks_complete_old_and_revised_run_formats_and_layout` (L23991): _no doc comment_
- `short_label_format_history_tests::formerly_unsupported_bold_to_italic_label_preserves_the_exact_fixture` (L24064): _no doc comment_
- `short_label_format_history_tests::fragmented_label_may_share_only_with_consistent_properties_on_each_side` (L24089): _no doc comment_
- `short_label_format_history_tests::label_property_boundaries_history_and_opaque_properties_stay_unchanged` (L24108): _no doc comment_
- `short_label_format_history_tests::configured_label_tracking_preserves_sources_and_respects_disabled_changes` (L24148): _no doc comment_
- `coverage_final_batch_finalize_tests::xml` (L24233): _no doc comment_
- `coverage_final_batch_finalize_tests::visible` (L24243): _no doc comment_
- `coverage_final_batch_finalize_tests::carried_revision_matrix_preserves_order_formatting_and_original_attribution` (L24252): _no doc comment_
- `coverage_final_batch_finalize_tests::external_link_rewrite_matrix_requires_new_target_and_wholly_inserted_content` (L24380): _no doc comment_
- `coverage_final_batch_finalize_tests::hyperlink_hoisting_preserves_empty_wrappers_and_run_order_across_revisions` (L24460): _no doc comment_
- `coverage_final_batch_finalize_tests::empty_field_in_a_revision_keeps_its_flags_and_padded_code` (L24544): _no doc comment_
- `coverage_final_batch_finalize_tests::inherited_invalidity_repairs_only_wml_schema_defects` (L24561): _no doc comment_
- `coverage_final_batch_finalize_tests::compatibility_bindings_keep_conflicts_and_recognize_attribute_only_extensions` (L24621): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::properties` (L24687): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::revision_para` (L24706): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::authored_paragraph` (L24747): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::document` (L24761): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::projected` (L24791): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::text` (L24863): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::compact_source_difference` (L24897): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::exercise` (L24959): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::full_production_compare_restores_original_and_revised_run_and_paragraph_formatting` (L25195): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::public_cell_replacement_keeps_both_source_paragraph_properties_at_a_blank_tail` (L25258): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::live_blank_cell_tail_keeps_independent_replacement_paragraph_owners` (L25314): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::a_related_replacement_before_live_content_uses_a_paired_unmarked_pilcrow` (L25355): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::multiple_original_paragraphs_before_a_live_blank_control_tail_keep_exact_phase_payloads` (L25406): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::multiple_revised_paragraphs_with_distinct_tail_properties_keep_word_deleted_carrier` (L25538): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::guarded_multi_original_complete_projection` (L25599): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::guarded_multi_original_complete_projection::{fn semantic}` (L25604): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::multiple_original_blank_tail_checks_every_later_pilcrow_provenance` (L25645): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::multiple_original_blank_tail_preserves_first_original_section_and_property_history` (L25780): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::multiple_revised_blank_tail_pairs_only_the_matching_final_revised_paragraph` (L26001): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::public_compare_multi_original_blank_tail_recovers_complete_clean_sources` (L26091): A raw D/I phase fixture is not an untracked document: its deleted paragraph marks can make revision projection promote the shared blank out of sdtContent. _(from a code comment)_
- `coverage_real_revision_merge_matrix_tests::public_compare_multi_original_blank_tail_recovers_complete_clean_sources::{fn clean_paragraph}` (L26092): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::public_compare_multi_original_blank_tail_recovers_complete_clean_sources::{fn package}` (L26097): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::public_compare_multi_original_blank_tail_recovers_complete_clean_sources::{fn complete_source}` (L26127): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::a_single_replacement_before_a_live_blank_control_tail_keeps_a_paired_pilcrow` (L26220): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::a_sole_original_and_multiple_revised_paragraphs_keep_live_blank_tail_ownership` (L26255): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::a_sole_original_and_multiple_revised_paragraphs_keep_live_blank_tail_ownership::{fn count_projected_paragraphs}` (L26256): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::live_blank_cell_tail_with_inert_run_keeps_exact_pilcrow_owners` (L26344): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::paired_property_comparison_never_grows_the_persistent_dom_or_changes_sources` (L26393): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::empty_pilcrow_proof_retains_inert_formatting_and_declines_owned_payloads` (L26443): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::public_two_original_one_revised_windows_keep_complete_sources_after_all_word_phases` (L26498): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::word_m98b_spacing_and_history_match_the_complete_format_golden` (L26601): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::repeated_authored_prefixes_keep_deleted_paragraphs_before_their_unchanged_table` (L26636): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::repeated_authored_prefixes_keep_deleted_paragraphs_before_their_unchanged_table::{fn owned}` (L26637): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::unshared_wrap_gap_keeps_each_sources_preserved_terminal_space` (L26742): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::unshared_wrap_gap_keeps_each_sources_preserved_terminal_space::{fn owned}` (L26743): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::organic_source_snapshot` (L26876): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::organic_paragraph` (L26959): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::wholesale_boundary_package` (L27003): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::wholesale_boundary_paragraph` (L27012): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::whole_story_title_label_heading_and_list_boundaries_keep_source_ownership` (L27034): These packages intentionally omit organic_package's equal anchors. _(from a code comment)_
- `coverage_real_revision_merge_matrix_tests::terminal_layout_paragraph` (L27452): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::public_terminal_field_math_and_break_boundaries_preserve_all_source_owners` (L27484): M360 field residues and M431 layout breaks have source payload even without cached text. _(from a code comment)_
- `coverage_real_revision_merge_matrix_tests::public_cross_paragraph_field_ends_keep_exact_source_instruction_and_cache_owners` (L27706): TOC-style complex fields legitimately cross paragraph boundaries. _(from a code comment)_
- `coverage_real_revision_merge_matrix_tests::organic_table` (L27881): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::organic_table_with_rows` (L27885): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::organic_package` (L27913): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::organic_heterogeneous_public_packages_preserve_owned_sources_across_modes` (L27987): These literal clean packages exercise heterogeneous organic production paths rather than supplying private D/I intermediate states.
- `coverage_real_revision_merge_matrix_tests::literal_legacy_table_geometry` (L28197): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::public_word_paired_table_width_and_grid_changes_restore_both_authored_geometries` (L28223): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::public_faithful_paired_table_geometry_has_an_exact_legacy_omission_golden` (L28292): WmlComparer v10.0.0 ReconstructElement (6731-6751) copies the matched ancestor's table properties/grid without an old table-level history. _(from a code comment)_
- `consecutive_block_splice_tests::paragraph` (L28356): _no doc comment_
- `consecutive_block_splice_tests::parse` (L28363): _no doc comment_
- `consecutive_block_splice_tests::canonical` (L28376): Namespace declarations can move when independently cloned revision projections are serialized. _(from a code comment)_
- `consecutive_block_splice_tests::resolved` (L28402): _no doc comment_
- `consecutive_block_splice_tests::adjacent_inserted_prefix_preserves_table_midsplice_and_both_complete_projections` (L28413): _no doc comment_
- `consecutive_block_splice_tests::live_boundary_prevents_an_earlier_insertion_from_claiming_the_splice` (L28451): _no doc comment_
- `demo_title_pilcrow_history_tests::paragraph` (L28475): _no doc comment_
- `demo_title_pilcrow_history_tests::source` (L28481): _no doc comment_
- `demo_title_pilcrow_history_tests::projection` (L28508): _no doc comment_
- `demo_title_pilcrow_history_tests::title_fold_retains_complete_pilcrow_formats_and_configured_history` (L28532): _no doc comment_
- `demo_title_pilcrow_history_tests::effect_projection` (L28607): _no doc comment_
- `demo_title_pilcrow_history_tests::effect_projection::{fn canonical}` (L28608): _no doc comment_
- `demo_title_pilcrow_history_tests::nested_mark_effect_changes_keep_both_source_projections_and_disabled_tracking` (L28648): _no doc comment_
- `demo_title_pilcrow_history_tests::closing_mark_history_preserves_nested_effects_and_ignores_only_existing_normalization_noise` (L28715): _no doc comment_
- `demo_title_pilcrow_history_tests::sharing_a_label_with_nested_effects_obeys_disabled_format_tracking` (L28751): _no doc comment_
- `demo_title_pilcrow_history_tests::title_fold_preserves_existing_mark_history_and_section_break_ownership` (L28768): _no doc comment_
- `demo_title_pilcrow_history_tests::authored_title_snapshot_survives_demo_default_layout_normalization` (L28801): _no doc comment_
- `paired_row_mark_ownership_tests::canonical` (L28850): _no doc comment_
- `paired_row_mark_ownership_tests::project` (L28878): _no doc comment_
- `paired_row_mark_ownership_tests::paired_rows_with_surviving_marks_keep_empty_revised_cells_and_full_original_history` (L28899): _no doc comment_
- `paired_row_mark_ownership_tests::unchanged_empty_pilcrows_keep_row_ownership_without_row_property_history` (L28940): _no doc comment_
- `paired_row_mark_ownership_tests::new_empty_rows_with_owned_pilcrows_still_receive_their_row_lifetime` (L28990): _no doc comment_
- `paired_row_mark_ownership_tests::public_empty_table_comparison_preserves_complete_authored_rows_and_pilcrows` (L29013): _no doc comment_
- `paired_row_mark_ownership_tests::public_empty_table_comparison_preserves_complete_authored_rows_and_pilcrows::{fn source_table}` (L29014): _no doc comment_
- `paired_row_mark_ownership_tests::explicit_row_lifetime_is_not_overridden_by_a_surviving_cell_mark` (L29074): _no doc comment_
- `paired_row_mark_ownership_tests::explicit_row_lifetime_preserves_opposite_content_revisions_and_property_history` (L29087): _no doc comment_
- `closing_authored_section_boundary_tests::parse` (L29134): _no doc comment_
- `closing_authored_section_boundary_tests::section` (L29139): _no doc comment_
- `closing_authored_section_boundary_tests::props` (L29143): _no doc comment_
- `closing_authored_section_boundary_tests::canonical` (L29150): _no doc comment_
- `closing_authored_section_boundary_tests::projection` (L29175): _no doc comment_
- `closing_authored_section_boundary_tests::section_addition_guard_respects_tracking_source_presence_and_exact_live_identity` (L29200): _no doc comment_
- `closing_authored_section_boundary_tests::matching_closing_history_keeps_its_existing_revision_owner_and_id` (L29315): _no doc comment_
- `closing_authored_section_boundary_tests::public_added_and_unchanged_authored_sections_keep_complete_source_bodies` (L29383): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::settings` (L29435): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::package` (L29442): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::semantic` (L29448): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::expected` (L29474): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::run` (L29482): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::para` (L29491): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::marked` (L29494): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::stable` (L29507): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::explicit_default_direction_is_removed_only_from_inserted_layout` (L29518): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::demo_spacing_respects_authored_metrics_history_and_source_defaults` (L29577): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::an_equal_alignment_change_is_attributed_once_and_never_claims_source_history` (L29656): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::the_final_empty_spacing_history_has_one_source_owner` (L29711): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::preexisting_insertions_keep_innermost_provenance_across_real_wrappers` (L29765): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::expected_run_format` (L29848): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::a_pending_insertion_splits_deleted_siblings_without_stealing_their_payloads` (L29857): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::move_simplification_changes_only_lifetime_names_and_range_markers` (L29943): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::equal_shared_labels_cannot_cross_opaque_or_historical_run_ownership` (L30001): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::whitespace_layout_before_a_deleted_block_is_not_empty_content` (L30060): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::list_layout_contamination_does_not_erase_real_list_or_terminal_ownership` (L30139): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::faithful_merged_closing_carrier_records_both_complete_property_sources` (L30192): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::the_short_list_label_uses_only_an_empty_deleted_carrier_with_continuation` (L30275): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::historical_legal_numbering_uses_the_donor_id_and_keeps_authored_old_level` (L30321): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::list_cluster_reorders_only_the_observed_nested_word_family` (L30386): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::a_stranded_mark_can_be_copied_without_deleting_its_layout_shell` (L30495): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::a_terminal_property_history_replaces_the_mark_but_keeps_its_formats` (L30534): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::title_page_empty_relocation_is_scoped_to_an_authored_cover_tail` (L30585): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::a_short_title_mesh_cannot_consume_foreign_revision_provenance_or_unrelated_titles` (L30634): _no doc comment_
- `finalizer_remaining_source_boundary_tests::package` (L30678): _no doc comment_
- `finalizer_remaining_source_boundary_tests::semantic` (L30689): _no doc comment_
- `finalizer_remaining_source_boundary_tests::run` (L30715): _no doc comment_
- `finalizer_remaining_source_boundary_tests::para` (L30724): _no doc comment_
- `finalizer_remaining_source_boundary_tests::marked` (L30730): _no doc comment_
- `finalizer_remaining_source_boundary_tests::assert_body` (L30744): _no doc comment_
- `finalizer_remaining_source_boundary_tests::words` (L30769): _no doc comment_
- `finalizer_remaining_source_boundary_tests::document_scale_folds_preserve_authored_sources_and_observed_word_exceptions` (L30780): The public Word policy distinguishes local edits from wholesale gaps. _(from a code comment)_
- `finalizer_remaining_source_boundary_tests::midstream_deleted_blocks_cross_only_payload_that_disappears_on_rejection` (L31038): _no doc comment_
- `finalizer_remaining_source_boundary_tests::paired_lifetimes_require_complete_matching_comparison_provenance` (L31108): _no doc comment_
- `finalizer_remaining_source_boundary_tests::paired_format_equality_is_complete_and_uses_no_production_arena_nodes` (L31191): _no doc comment_
- `finalizer_remaining_source_boundary_tests::two_authored_layout_blanks_are_restored_only_at_the_shared_table_boundary` (L31268): _no doc comment_
- `finalizer_merge_source_guard_tests::package` (L31424): _no doc comment_
- `finalizer_merge_source_guard_tests::semantic` (L31431): _no doc comment_
- `finalizer_merge_source_guard_tests::run` (L31457): _no doc comment_
- `finalizer_merge_source_guard_tests::para` (L31466): _no doc comment_
- `finalizer_merge_source_guard_tests::marked` (L31472): _no doc comment_
- `finalizer_merge_source_guard_tests::assert_body` (L31486): _no doc comment_
- `finalizer_merge_source_guard_tests::words` (L31511): _no doc comment_
- `finalizer_merge_source_guard_tests::related_replacement_selects_the_documented_structural_carrier_without_losing_run_sources` (L31522): M88 keeps original structural paragraph properties when the new mark is bare. _(from a code comment)_
- `finalizer_merge_source_guard_tests::an_unresolved_field_residue_preserves_m360s_bare_heading_carrier` (L31600): _no doc comment_
- `finalizer_merge_source_guard_tests::legal_and_memo_midstory_block_ownership_is_retained_at_exact_boundaries` (L31639): _no doc comment_
- `finalizer_merge_source_guard_tests::a_carried_original_insertion_keeps_its_author_and_never_becomes_a_real_deletion` (L31713): _no doc comment_
- `finalizer_merge_source_guard_tests::pilcrow_format_equality_includes_nested_fill_payload_but_ignores_source_scratch` (L31772): _no doc comment_
- `finalizer_reached_predicate_boundary_tests::package` (L31857): _no doc comment_
- `finalizer_reached_predicate_boundary_tests::semantic` (L31868): _no doc comment_
- `finalizer_reached_predicate_boundary_tests::run` (L31894): _no doc comment_
- `finalizer_reached_predicate_boundary_tests::para` (L31903): _no doc comment_
- `finalizer_reached_predicate_boundary_tests::marked` (L31909): _no doc comment_
- `finalizer_reached_predicate_boundary_tests::assert_body` (L31923): _no doc comment_
- `finalizer_reached_predicate_boundary_tests::a_later_revised_block_protects_the_long_intro_checklist_boundary` (L31954): Midstory unrelated replacements keep their complete source carriers. _(from a code comment)_
- `finalizer_reached_predicate_boundary_tests::a_short_residual_label_does_not_steal_long_bookmark_or_heading_properties` (L32100): The original bookmark prose deliberately includes the inserted label. _(from a code comment)_
- `public_body_heuristic_story_scope_tests::canonical` (L32220): _no doc comment_
- `public_body_heuristic_story_scope_tests::public_body_layout_heuristics_preserve_complete_independent_header_and_footer_parts` (L32251): These are valid header/footer part roots accepted by the public DOM helpers, not body-pipeline fixtures. _(from a code comment)_
- `default_source_property_carrier_tests::package` (L32548): _no doc comment_
- `default_source_property_carrier_tests::semantic` (L32559): _no doc comment_
- `default_source_property_carrier_tests::run` (L32585): _no doc comment_
- `default_source_property_carrier_tests::assert_body` (L32594): _no doc comment_
- `default_source_property_carrier_tests::properties` (L32598): _no doc comment_
- `default_source_property_carrier_tests::related_one_to_one_pairs_keep_the_exact_default_or_structural_source_carrier` (L32619): Clean Word paragraphs may omit pPr entirely. _(from a code comment)_
- `default_source_property_carrier_tests::a_sole_original_closes_multi_inserted_default_paragraphs_without_inventing_properties` (L32667): Single-paragraph Word GT closes a wholesale replacement in the last revised paragraph. _(from a code comment)_
- `existing_mark_layout_ownership_tests::package` (L32731): _no doc comment_
- `existing_mark_layout_ownership_tests::semantic` (L32742): _no doc comment_
- `existing_mark_layout_ownership_tests::run` (L32768): _no doc comment_
- `existing_mark_layout_ownership_tests::assert_body` (L32777): _no doc comment_
- `existing_mark_layout_ownership_tests::mark` (L32802): _no doc comment_
- `existing_mark_layout_ownership_tests::history` (L32806): _no doc comment_
- `existing_mark_layout_ownership_tests::fresh_history` (L32811): _no doc comment_
- `existing_mark_layout_ownership_tests::settings` (L32816): _no doc comment_
- `existing_mark_layout_ownership_tests::moving_mixed_spacing_keeps_existing_inserted_and_deleted_mark_owners` (L32828): Word M221 moves a MIX's spacing to the authored following blank. _(from a code comment)_
- `existing_mark_layout_ownership_tests::parking_heading_spacing_keeps_existing_lifetime_and_history_provenance` (L32906): M221's other observed Word shape parks heading spacing on the final deleted residual. _(from a code comment)_
- `existing_mark_layout_ownership_tests::alignment_donor_search_crosses_default_property_paragraphs_without_claiming_their_formats` (L32991): _no doc comment_
- `interleave_terminal_source_ownership_tests::package` (L33061): _no doc comment_
- `interleave_terminal_source_ownership_tests::semantic` (L33070): _no doc comment_
- `interleave_terminal_source_ownership_tests::owned` (L33098): Empty text is an authored blank paragraph with a revision on its pilcrow, rather than an empty text node masquerading as a list item. _(from a code comment)_
- `interleave_terminal_source_ownership_tests::remains_source_owned` (L33122): _no doc comment_
- `interleave_terminal_source_ownership_tests::terminal_revision_runs_and_incomplete_interleave_keep_all_source_positions` (L33163): _no doc comment_
- `interleave_terminal_source_ownership_tests::an_authored_inserted_blank_is_not_a_short_list_label` (L33192): _no doc comment_
- `interleave_terminal_source_ownership_tests::a_deleted_blank_numbered_paragraph_does_not_claim_a_nested_content_cluster` (L33210): _no doc comment_
- `interleave_terminal_source_ownership_tests::split_inserted_title_fragments_do_not_take_another_authors_pending_revision` (L33227): _no doc comment_
- `whitespace_source_mark_carrier_tests::package` (L33280): _no doc comment_
- `whitespace_source_mark_carrier_tests::semantic` (L33289): _no doc comment_
- `whitespace_source_mark_carrier_tests::revision` (L33324): _no doc comment_
- `whitespace_source_mark_carrier_tests::check_complete` (L33330): _no doc comment_
- `whitespace_source_mark_carrier_tests::whitespace_fold_preserves_selected_source_mark_shells_and_every_body_owner` (L33360): Word M86/M88 folds actual inserted spaces into the residual deletion. _(from a code comment)_
- `fragmented_source_label_boundary_tests::package` (L33444): _no doc comment_
- `fragmented_source_label_boundary_tests::semantic` (L33453): _no doc comment_
- `fragmented_source_label_boundary_tests::run` (L33490): _no doc comment_
- `fragmented_source_label_boundary_tests::wrapper` (L33494): _no doc comment_
- `fragmented_source_label_boundary_tests::assert_complete` (L33500): _no doc comment_
- `fragmented_source_label_boundary_tests::fragmented_labels_keep_literal_source_chars_formats_and_revision_history` (L33524): _no doc comment_
- `fragmented_source_label_boundary_tests::label_format_boundaries_and_disabled_history_leave_complete_original_sources_intact` (L33678): _no doc comment_
- `fragmented_source_label_boundary_tests::midstream_heading_search_preserves_controls_live_anchors_and_identical_property_owners` (L33722): _no doc comment_
- `fragmented_source_label_boundary_tests::public_label_zip_preserves_already_adjacent_spacer_and_missing_layout_ownership` (L33810): Word M374 keeps the deleted list owner, donates only missing scalar layout properties, and leaves an already adjacent inserted spacer in source order.
- `fragmented_source_label_boundary_tests::wholesale_glue_only_lcs_preserves_crossed_significant_source_anchors` (L33908): A common significant word may exist on both sides while the longest ordered match contains only glue words.
- `nested_deleted_text_source_ownership_tests::package` (L33950): _no doc comment_
- `nested_deleted_text_source_ownership_tests::revision` (L33964): _no doc comment_
- `nested_deleted_text_source_ownership_tests::field` (L33969): _no doc comment_
- `nested_deleted_text_source_ownership_tests::outer_deletion_preserves_independently_owned_nested_revision_text_and_fields` (L33977): _no doc comment_
- `ladder_suffix_source_boundary_tests::xml` (L34066): _no doc comment_
- `ladder_suffix_source_boundary_tests::package` (L34072): _no doc comment_
- `ladder_suffix_source_boundary_tests::carrier` (L34078): _no doc comment_
- `ladder_suffix_source_boundary_tests::public_ladder_restores_suffix_author_at_unicode_and_run_boundaries` (L34088): _no doc comment_
- `reached_word_document_policy_tests::paragraph` (L34207): _no doc comment_
- `reached_word_document_policy_tests::document_policy_reaches_numbering_relatedness_and_demo_intro_boundaries` (L34228): _no doc comment_
- `mid_list_optional_source_property_tests::package` (L34399): _no doc comment_
- `mid_list_optional_source_property_tests::list_layout_cleanup_handles_absent_optional_properties_without_inventing_owners` (L34412): M376 removes copied list layout from a middle prose deletion.
- `live_opaque_row_source_boundary_tests::unchanged_opaque_cell_owners_prevent_row_lifetime_synthesis_without_text_caches` (L34493): Nontext source owners are real content, even when their cached result is empty. _(from a code comment)_
- `deleted_carrier_echo_source_owner_tests::parse` (L34581): _no doc comment_
- `deleted_carrier_echo_source_owner_tests::paragraph` (L34587): _no doc comment_
- `deleted_carrier_echo_source_owner_tests::canonical` (L34592): _no doc comment_
- `deleted_carrier_echo_source_owner_tests::events` (L34622): _no doc comment_
- `deleted_carrier_echo_source_owner_tests::deleted_original_carrier_keeps_echo_prefix_and_both_complete_source_projections` (L34664): _no doc comment_
