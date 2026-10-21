# Functions and methods in `src/comparer/finalize.rs`

269 non-test, 500 test. Part of the [symbol manifest](../README.md).

#### `begin_para_classification_cache`  (L49-52, pub)
`pub fn begin_para_classification_cache()`

- **Does:** Enable the per-paragraph pure-del / mixed classification caches the `M4.F` peels share.
- **Calls (std/external/unresolved):** `.borrow_mut`, `.with`, `HashMap::new`, `Some`
- **Called by:** `coverage_tests::classification_cache_hits_and_resets_agree_with_uncached_results` (comparer/finalize.rs:15698); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `end_para_classification_cache`  (L57-60, pub)
`pub fn end_para_classification_cache()`

- **Does:** Drop the thread-local pure-del / mixed paragraph classification caches; called by the pipeline at the end of the peel ladder so the next document run starts with empty caches.
- **Calls (std/external/unresolved):** `.borrow_mut`, `.with`
- **Called by:** `coverage_tests::classification_cache_hits_and_resets_agree_with_uncached_results` (comparer/finalize.rs:15698); `<Reset as Drop>::drop` (comparer/finalize.rs:15701); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

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
- **Called by:** `descendants_trimmed` (comparer/finalize.rs:63); `descendants_trimmed::{fn walk}` (comparer/finalize.rs:65); `convert_run_text_to_del_text` (comparer/finalize.rs:102); `convert_run_text_to_del_text::{fn walk}` (comparer/finalize.rs:103); `enforce_deleted_text_kinds` (comparer/finalize.rs:14856); `enforce_deleted_text_kinds::{fn walk}` (comparer/finalize.rs:14857)

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
- **Called by:** `mark_content_transform` (comparer/finalize.rs:192); `coverage_tests::deleted_text_conversion_stops_at_nested_revision_owners` (comparer/finalize.rs:15823)

#### `convert_run_text_to_del_text::{fn walk}`  (L103-122)
`fn walk(dom: &mut Dom, node: NodeId)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L104; `Dom::is_element` (xmllinq/mod.rs:502) L105; `Dom::name` (xmllinq/mod.rs:526) L108; `Dom::set_name` (xmllinq/mod.rs:539) L117,118; `descendants_trimmed::{fn walk}` (comparer/finalize.rs:65) L119
- **Calls (std/external/unresolved):** `.as_ref`, `W::del_text`, `W::del`, `W::ins`, `W::instr_text`, `W::name`, `W::t`
- **Called by:** `descendants_trimmed` (comparer/finalize.rs:63); `descendants_trimmed::{fn walk}` (comparer/finalize.rs:65); `convert_run_text_to_del_text` (comparer/finalize.rs:102); `convert_run_text_to_del_text::{fn walk}` (comparer/finalize.rs:103); `enforce_deleted_text_kinds` (comparer/finalize.rs:14856); `enforce_deleted_text_kinds::{fn walk}` (comparer/finalize.rs:14857)

#### `rev_el`  (L126-133)
`fn rev_el(dom: &mut Dom, name: XName, settings: &WmlComparerSettings, id_gen: &mut u32) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L127; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L128,129,131
- **Calls (std/external/unresolved):** `.to_string`, `Some`, `W::author`, `W::date`, `W::id`
- **Called by:** `mark_content_transform` (comparer/finalize.rs:192); `merged_closing_carrier_takes_source_properties` (comparer/finalize.rs:3122); `convert_stamped_preins` (comparer/finalize.rs:10287); `wrap_bare_del_text_runs` (comparer/finalize.rs:10776); `delete_stories_of_deleted_text_boxes` (comparer/finalize.rs:10817); `mark_paragraph_mark_deleted` (comparer/finalize.rs:10908); `convert_stamped_predeletes` (comparer/finalize.rs:11141); `fold_midstream_demo_title_into_numbered_heading_with_source_properties` (comparer/finalize.rs:12624); `mark_tabbed_deleted_titles` (comparer/finalize.rs:15172); `append_empty_ppr_change` (comparer/finalize.rs:15243); `unmerge_comment_list_item` (comparer/finalize.rs:15255)

#### `copy_attrs`  (L136-143)
`fn copy_attrs(dom: &mut Dom, src: NodeId, dst: NodeId, skip_pt: bool)`

- **Does:** Clone `src`'s attributes onto `dst` (optionally skipping the pt namespace).
- **Calls (crate):** `Dom::attributes` (xmllinq/mod.rs:899) L137; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L138; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L141
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `rebuild_run` (comparer/finalize.rs:162); `mark_content_transform` (comparer/finalize.rs:192); `conjoin_transform` (comparer/finalize.rs:628); `coalesce_adjacent_runs` (comparer/finalize.rs:1222); `simplify_move_transform` (comparer/finalize.rs:6818); `split_insertion_at` (comparer/finalize.rs:15050)

#### `copy_move_id_attrs`  (L150-159)
`fn copy_move_id_attrs(dom: &mut Dom, src: NodeId, dst: NodeId)`

- **Does:** Copy only `w:author`, `w:date`, `w:id` from `src` onto `dst` — matches the oracle's `SimplifyMoveMarkupToDelIns` (WmlComparer.cs:2874-2878), which passes `moveFrom.Attribute(W.author) / (W.date) / (W.id)` explicitly when constructing the replacement `w:del` / `w:ins`.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L154; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L156
- **Calls (std/external/unresolved):** `.map`, `Some`, `W::author`, `W::date`, `W::id`
- **Called by:** `simplify_move_transform` (comparer/finalize.rs:6818)

#### `rebuild_run`  (L162-177)
`fn rebuild_run( dom: &mut Dom, element: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, skip_pt: bool, ) -> NodeId`

- **Does:** Rebuild a run, recursing its children through the transform.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L169; `copy_attrs` (comparer/finalize.rs:136) L170; `Dom::nodes` (xmllinq/mod.rs:604) L171; `mark_content_transform` (comparer/finalize.rs:192) L172; `Dom::add` (xmllinq/mod.rs:1074) L173
- **Calls (std/external/unresolved):** `W::r`
- **Called by:** `mark_content_transform` (comparer/finalize.rs:192)

#### `mark_content_transform`  (L192-463, pub)
`pub fn mark_content_transform( dom: &mut Dom, node: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, ) -> Vec<NodeId>`

- **Does:** `MarkContentAsDeletedOrInsertedTransform` (WmlComparer.cs:2568) — the recursive body of `M4.F.1/F.2/F.3`.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L198; `Dom::clone_subtree` (xmllinq/mod.rs:1231) (by name) L199,245; `Dom::name` (xmllinq/mod.rs:526) L201; `Dom::elements` (xmllinq/mod.rs:640) L205; `Dom::attribute` (xmllinq/mod.rs:874) L206,215,267,284,340,363...; `Dom::new_element` (xmllinq/mod.rs:478) L221,238,321,333,345,351...; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L222,223,224,229,239,322...; `rev_el` (comparer/finalize.rs:126) L234,247,296,326,449; `complex_field_runs` (comparer/finalize.rs:14728) L235,253; `Dom::add` (xmllinq/mod.rs:1074) L236,254,301,332,356,357...; `descendants_trimmed` (comparer/finalize.rs:63) L261; `is_run_status_carrier` (comparer/finalize.rs:77) L263; `rebuild_run` (comparer/finalize.rs:162) L277,297,327,341; `convert_run_text_to_del_text` (comparer/finalize.rs:102) L299; `Dom::element` (xmllinq/mod.rs:655) L342,392,400,441; `Dom::add_first` (xmllinq/mod.rs:1101) L346,396,445; `parse_rpr` (comparer/finalize.rs:466) L350,401; `copy_attrs` (comparer/finalize.rs:136) L367,455; `Dom::nodes` (xmllinq/mod.rs:604) L368,456; `mark_content_transform` (comparer/finalize.rs:192) L369,457; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L382,435; `parse_ppr` (comparer/finalize.rs:499) L429
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.as_str`, `.collect`, `.filter`, `.find_map`, `.into_iter`, `.is_empty`, `.is_none`, `.iter`, `.len`, `.map`, `.push`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `.unwrap`, `PT::name`, `PT::status`, `Some`, `Vec::new`, `W::author`, `W::date`, `W::del`, `W::id`, `W::ins`, `W::move_from_range_end`, `W::move_to_range_end`, `W::name`, `W::p_pr_change`, `W::p_pr`, `W::r_pr`, `W::r`, `move_name`
- **Macros:** `panic!`, `vec!`
- **Called by:** `rebuild_run` (comparer/finalize.rs:162); `mark_content_transform` (comparer/finalize.rs:192); `mark_content_as_deleted_or_inserted` (comparer/finalize.rs:517); `run_format_history_regressions::one_new_run_preserves_each_original_format_boundary_when_rejected` (comparer/produce.rs:2063); `run_format_history_regressions::inline_containers_preserve_each_original_format_boundary_when_rejected` (comparer/produce.rs:2144); `nontext_run_format_history_tests::changed_nontext_run_payload_keeps_complete_original_and_revised_properties` (comparer/produce.rs:2681)

#### `parse_rpr`  (L466-475)
`fn parse_rpr(dom: &mut Dom, s: Option<&str>) -> NodeId`

- **Does:** Parse an `OldRPr` attribute string into a `w:rPr` element (empty on failure).
- **Calls (crate):** `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L469; `Dom::root` (xmllinq/mod.rs:822) L470; `drop_pt_bookkeeping` (comparer/finalize.rs:481) L471; `Dom::new_element` (xmllinq/mod.rs:478) L474
- **Calls (std/external/unresolved):** `W::r_pr`
- **Called by:** `mark_content_transform` (comparer/finalize.rs:192)

#### `drop_pt_bookkeeping`  (L481-496)
`fn drop_pt_bookkeeping(dom: &mut Dom, root: NodeId) -> NodeId`

- **Does:** Clone a parsed old-properties element without the scratch `pt14:*` attributes it was serialized with, nor the `xmlns:ns0` declarations the standalone serialization bound them to (they would otherwise survive into the redline).
- **Calls (crate):** `Dom::clone_subtree` (xmllinq/mod.rs:1231) L482; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L483; `Dom::attr_count` (xmllinq/mod.rs:909) L484; `Dom::attr_at` (xmllinq/mod.rs:920) L485; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L487; `Dom::is_namespace_declaration` (xmllinq/mod.rs:1012) L487; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L492
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.filter`, `.map`
- **Called by:** `parse_rpr` (comparer/finalize.rs:466); `parse_ppr` (comparer/finalize.rs:499); `closing_paragraph_properties` (comparer/finalize.rs:3093)

#### `parse_ppr`  (L499-507)
`fn parse_ppr(dom: &mut Dom, s: Option<&str>) -> NodeId`

- **Does:** Parse an `OldPPr` attribute string into a `w:pPr` element (empty on failure).
- **Calls (crate):** `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L501; `Dom::root` (xmllinq/mod.rs:822) L502; `drop_pt_bookkeeping` (comparer/finalize.rs:481) L503; `Dom::new_element` (xmllinq/mod.rs:478) L506
- **Calls (std/external/unresolved):** `W::p_pr`
- **Called by:** `mark_content_transform` (comparer/finalize.rs:192)

#### `mark_content_as_deleted_or_inserted`  (L517-530, pub)
`pub fn mark_content_as_deleted_or_inserted( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, ) -> NodeId`

- **Does:** M4.F — run [`mark_content_transform`] over `root` and return the new root.
- **Calls (crate):** `mark_content_transform` (comparer/finalize.rs:192) L523; `join_move_ranges` (comparer/finalize.rs:536) L528
- **Calls (std/external/unresolved):** `.expect`, `.into_iter`, `.next`
- **Called by:** `coverage_round_next_tests::moved_empty_field_becomes_a_complex_field_in_the_move` (comparer/finalize.rs:20166); `coverage_round_next_tests::deleted_or_inserted_empty_field_becomes_a_complex_field_in_the_revision` (comparer/finalize.rs:20205); `finalize_notes_part` (comparer/footnotes.rs:716); `horizontal_partition_tests::actual_atomized_split_with_deleted_table_conserves_source_through_production` (comparer/lcs_table.rs:1186); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `join_move_ranges`  (L536-570)
`fn join_move_ranges(dom: &mut Dom, root: NodeId)`

- **Does:** Marking emits a triple per run (and empty field), but a move names one range on each side.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L538,542; `Dom::attribute` (xmllinq/mod.rs:874) L545,549,552,560; `Dom::remove` (xmllinq/mod.rs:1118) L561,562; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L563
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.copied`, `.filter_map`, `.get_mut`, `.get`, `.insert`, `.into_iter`, `.len`, `.map`, `.to_string`, `.unwrap`, `HashMap::new`, `Some`, `W::id`, `W::name`
- **Macros:** `format!`
- **Called by:** `mark_content_as_deleted_or_inserted` (comparer/finalize.rs:517)

#### `ppr_has_structural_props`  (L574-589)
`fn ppr_has_structural_props(dom: &Dom, ppr: NodeId) -> bool`

- **Does:** True when pPr carries structural body props (pStyle/numPr/spacing/jc/ind/…).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L575; `Dom::name` (xmllinq/mod.rs:526) L576; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L582
- **Calls (std/external/unresolved):** `W::p_pr_change`, `W::r_pr`, `W::sect_pr`
- **Called by:** `conjoin_transform` (comparer/finalize.rs:628); `fold_whitespace_pure_ins_into_following_pure_del` (comparer/finalize.rs:4252); `merge_replaced_in_container` (comparer/finalize.rs:8626); `coverage_round_next_tests::structural_property_classifiers_ignore_history_and_require_one_alignment` (comparer/finalize.rs:20085)

#### `ppr_is_jc_only`  (L592-615)
`fn ppr_is_jc_only(dom: &Dom, ppr: NodeId) -> bool`

- **Does:** True when the only structural child of pPr is `w:jc` (center-align demos).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L594; `Dom::name` (xmllinq/mod.rs:526) L595; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L601
- **Calls (std/external/unresolved):** `W::jc_el`, `W::p_pr_change`, `W::r_pr`, `W::sect_pr`
- **Called by:** `merge_replaced_in_container` (comparer/finalize.rs:8626); `coverage_round_next_tests::structural_property_classifiers_ignore_history_and_require_one_alignment` (comparer/finalize.rs:20085)

#### `conjoin_transform`  (L628-735)
`fn conjoin_transform(dom: &mut Dom, node: NodeId, author: &str, date: &str) -> NodeId`

- **Does:** M4.F.4 — `ConjoinTransform` (:2947): collapse a `w:p` with ≥2 `w:pPr` into one cleaned pPr + all non-pPr children.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L629; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L630,674,699; `Dom::name` (xmllinq/mod.rs:526) L632,681,703,721; `Dom::elements` (xmllinq/mod.rs:640) L633,634,679,680,686,701...; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L637,643,663,669; `ppr_has_structural_props` (comparer/finalize.rs:574) L650,652,653; `Dom::remove` (xmllinq/mod.rs:1118) L683,687,705; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L691,700,710,711,712; `Dom::element` (xmllinq/mod.rs:655) L697; `Dom::new_element` (xmllinq/mod.rs:478) L709,717,728; `Dom::add` (xmllinq/mod.rs:1074) L713,714,719,723,732; `copy_attrs` (comparer/finalize.rs:136) L718,729; `conjoin_transform` (comparer/finalize.rs:628) L722,731; `Dom::nodes` (xmllinq/mod.rs:604) L730
- **Calls (std/external/unresolved):** `.copied`, `.find`, `.is_empty`, `.is_none`, `.iter`, `.len`, `.position`, `.saturating_sub`, `.unwrap`, `PT::status`, `Some`, `W::author`, `W::date`, `W::del`, `W::id`, `W::ins`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`
- **Macros:** `matches!`
- **Called by:** `conjoin_transform` (comparer/finalize.rs:628); `conjoin_paragraph_marks` (comparer/finalize.rs:745)

#### `conjoin_paragraph_marks`  (L745-756, pub)
`pub fn conjoin_paragraph_marks( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, ) -> NodeId`

- **Does:** M4.F.4 — apply the Conjoin transform to `root`.
- **Calls (crate):** `conjoin_transform` (comparer/finalize.rs:628) L750
- **Called by:** `coverage_round_next_tests::conjoin_property_selection_retains_format_and_does_not_duplicate_history` (comparer/finalize.rs:20110); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `fix_up_revision_ids`  (L768-854, pub)
`pub fn fix_up_revision_ids(dom: &mut Dom, roots: &[NodeId])`

- **Does:** M4.F.5 — `FixUpRevisionIds` (WmlComparer.cs:2769): renumber every `w:id` of every revision element under each `root` from `1`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L803; `Dom::name` (xmllinq/mod.rs:526) L804,841; `Dom::attribute` (xmllinq/mod.rs:874) L808,838; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L852
- **Calls (std/external/unresolved):** `.and_then`, `.contains`, `.get`, `.insert`, `.map`, `.ok`, `.parse`, `.push`, `.to_string`, `.unwrap`, `Some`, `Vec::new`, `W::cell_del`, `W::cell_ins`, `W::del`, `W::id`, `W::ins`, `W::move_from_range_end`, `W::move_to_range_end`, `W::name`, `W::p_pr_change`, `alloc`, `std::collections::HashMap::new`, `std::collections::HashSet::new`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215); `compare_documents_impl` (document_comparer.rs:6521); `typing_order` (edit/replacement_order.rs:82) (?)

#### `ignore_pt14_namespace`  (L862-874, pub)
`pub fn ignore_pt14_namespace(dom: &mut Dom, root: NodeId)`

- **Does:** M4.F.7 — `IgnorePt14Namespace` (WmlComparer.cs:2912): declare `xmlns:pt14` on `root` and add `pt14` to `mc:Ignorable` (idempotent).
- **Calls (crate):** `XNamespace::xmlns` (xmllinq/mod.rs:122) L863; `Dom::attribute` (xmllinq/mod.rs:874) L864,868; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L865,872
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.is_none`, `.join`, `.name`, `.push`, `.split_whitespace`, `.to_string`, `.unwrap_or`, `MC::name`, `Some`
- **Called by:** `finalize_notes_part` (comparer/footnotes.rs:716); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215); `pre_process_markup` (document_comparer.rs:4714)

#### `declare_extension_namespaces_ignorable`  (L942-1001, pub)
`pub fn declare_extension_namespaces_ignorable(dom: &mut Dom, root: NodeId)`

- **Does:** Elements merged in from the other document (B's numbering definitions, styles, notes) keep their Word 2010+ extension attributes but not the source root's `mc:Ignorable` contract, so the part claims markup it never declared ignorable (`w15:restartNumberingAfterBreak`, `w16cid:durableId` under a Mac-era numbering root).
- **Calls (crate):** `XNamespace::xmlns` (xmllinq/mod.rs:122) L943; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L945; `Dom::name` (xmllinq/mod.rs:526) L954; `existing_mark_layout_ownership_tests::mark` (comparer/finalize.rs:33009) L955,959; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L955,959; `Dom::attributes_iter` (xmllinq/mod.rs:886) L957; `Dom::is_namespace_declaration` (xmllinq/mod.rs:1012) L958,995; `Dom::attribute` (xmllinq/mod.rs:874) L967,978,991; `XNamespace::name` (xmllinq/mod.rs:134) L978,983,990; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L983,997,999; `Dom::attributes` (xmllinq/mod.rs:899) L992
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.collect`, `.contains`, `.enumerate`, `.is_none`, `.iter`, `.join`, `.len`, `.map`, `.position`, `.push`, `.split_whitespace`, `.to_string`, `.unwrap_or`, `MC::name`, `Some`
- **Called by:** `coverage_final_batch_finalize_tests::compatibility_bindings_keep_conflicts_and_recognize_attribute_only_extensions` (comparer/finalize.rs:24643); `compare_documents_impl` (document_comparer.rs:6521); `assemble` (markdown/package.rs:103)

#### `bind_compatibility_prefixes`  (L1026-1051, pub)
`pub fn bind_compatibility_prefixes(dom: &mut Dom, root: NodeId)`

- **Does:** Binds every prefix an MC prefix list names (`mc:Choice/@Requires`, `mc:Ignorable`, `mc:ProcessContent`, …) where the list names it.
- **Calls (crate):** `XNamespace::xmlns` (xmllinq/mod.rs:122) L1028; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L1029; `Dom::name` (xmllinq/mod.rs:526) L1030; `Dom::attributes_iter` (xmllinq/mod.rs:886) L1033; `is_namespace_prefix_list` (xmllinq/serialize.rs:365) L1035; `prefix_in_scope` (comparer/finalize.rs:1490) L1041; `well_known_namespace` (xmllinq/serialize.rs:190) L1044; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1047; `XNamespace::name` (xmllinq/mod.rs:134) L1047
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.is_empty`, `.map_or`, `.map`, `.split_once`, `.split_whitespace`, `.to_string`, `Some`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6521); `repair` (validate.rs:2623)

#### `remove_powertools_scratch_markup`  (L1060-1072, pub)
`pub fn remove_powertools_scratch_markup(dom: &mut Dom, root: NodeId)`

- **Does:** Strip every `pt:*` attribute across `root` and its descendants.
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L1061; `Dom::attributes` (xmllinq/mod.rs:899) L1062; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1066; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1069
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.map`
- **Called by:** `make_empty_pure_ins_para` (comparer/finalize.rs:3824); `paired_replacement_has_identical_properties` (comparer/finalize.rs:8529); `sectpr_identity` (comparer/finalize.rs:11609); `final_peel_has_identical_closing_properties` (comparer/finalize.rs:12496); `coverage_real_revision_merge_matrix_tests::projected` (comparer/finalize.rs:24813); `coverage_real_revision_merge_matrix_tests::guarded_multi_original_complete_projection` (comparer/finalize.rs:25621); `coverage_real_revision_merge_matrix_tests::organic_source_snapshot` (comparer/finalize.rs:26897); `paired_row_mark_ownership_tests::public_empty_table_comparison_preserves_complete_authored_rows_and_pilcrows::{fn source_table}` (comparer/finalize.rs:29032); `closing_authored_section_boundary_tests::projection` (comparer/finalize.rs:29192); `finalize_notes_part` (comparer/footnotes.rs:716); `paired_note_format_history_tests::paired_note_content_format_history_respects_word_and_faithful_modes` (comparer/footnotes.rs:1968); `nested_paragraph_property_source_history_tests::clean` (comparer/formatchg.rs:788); +6 more

#### `xml_space_attr`  (L1082-1088)
`fn xml_space_attr(text: &str) -> Option<&'static str>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chars`, `.is_whitespace`, `.last`, `.next`, `Some`
- **Called by:** `coalesce_adjacent_runs` (comparer/finalize.rs:1222); `split_insertion_at` (comparer/finalize.rs:15050)

#### `rpr_string`  (L1090-1095)
`fn rpr_string(dom: &Dom, r: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1091; `Dom::serialize_element` (xmllinq/mod.rs:1315) L1092
- **Calls (std/external/unresolved):** `String::new`, `W::r_pr`
- **Called by:** `coalesce_key` (comparer/finalize.rs:1097); `mesh_short_label_shared_eq_with_settings::{fn label_run}` (comparer/finalize.rs:7322)

#### `coalesce_key`  (L1097-1193)
`fn coalesce_key(dom: &Dom, ce: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L1098; `Dom::elements` (xmllinq/mod.rs:640) L1102,1145,1148,1151,1161,1172; `Dom::name_is` (xmllinq/mod.rs:535) L1105,1149; `Dom::attribute` (xmllinq/mod.rs:874) L1110,1128,1159,1160,1187; `rpr_string` (comparer/finalize.rs:1090) L1131; `Dom::element` (xmllinq/mod.rs:655) L1132,1135,1154,1165; `Dom::serialize_element` (xmllinq/mod.rs:1315) L1166
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.count`, `.filter_map`, `.filter`, `.flat_map`, `.into_iter`, `.is_some`, `.iter`, `.join`, `.map`, `.to_string`, `.unwrap_or`, `PT::name`, `Some`, `W::author`, `W::date`, `W::del_text`, `W::del`, `W::instr_text`, `W::r_pr`, `W::r`, `W::t`
- **Macros:** `format!`
- **Called by:** `coalesce_adjacent_runs` (comparer/finalize.rs:1222)

#### `run_text_concat`  (L1195-1204)
`fn run_text_concat(dom: &Dom, r: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1197; `Dom::name` (xmllinq/mod.rs:526) L1198; `Dom::value_str` (xmllinq/mod.rs:1195) L1200
- **Calls (std/external/unresolved):** `.push_str`, `.unwrap`, `String::new`, `W::del_text`, `W::instr_text`, `W::t`
- **Called by:** `coalesce_adjacent_runs` (comparer/finalize.rs:1222)

#### `coalesce_adjacent_runs`  (L1222-1286, pub)
`pub fn coalesce_adjacent_runs(dom: &mut Dom, container: NodeId) -> NodeId`

- **Does:** Merge adjacent `w:r` and `w:del` runs of identical text + formatting into one — port of `CoalesceAdjacentRunsWithIdenticalFormatting` (PtOpenXmlUtil.ts:4458).
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L1223,1239; `Dom::elements` (xmllinq/mod.rs:640) L1224; `coalesce_key` (comparer/finalize.rs:1097) L1225; `Dom::new_element` (xmllinq/mod.rs:478) L1227,1241,1252,1260,1262,1270; `copy_attrs` (comparer/finalize.rs:136) L1228,1242,1261,1264; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1232,1244,1266,1280; `Dom::add` (xmllinq/mod.rs:1074) L1233,1245,1257,1258,1267,1275...; `run_text_concat` (comparer/finalize.rs:1195) L1237; `Dom::element` (xmllinq/mod.rs:655) L1243,1247,1263,1265; `xml_space_attr` (comparer/finalize.rs:1082) L1253,1271; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1254,1272; `XNamespace::xml` (xmllinq/mod.rs:128) L1254,1272; `Dom::add_text` (xmllinq/mod.rs:1087) L1256,1274
- **Calls (std/external/unresolved):** `.collect`, `.is_some`, `.iter`, `.map`, `.name`, `.unwrap`, `Some`, `W::del_text`, `W::del`, `W::instr_text`, `W::r_pr`, `W::r`, `W::t`, `crate::util::group_adjacent`
- **Called by:** `coalesce_all_paragraphs` (comparer/finalize.rs:1503)

#### `resolve_alternate_content`  (L1301-1345, pub)
`pub fn resolve_alternate_content(dom: &mut Dom, root: NodeId)`

- **Does:** Resolve feature-gating `mc:AlternateContent` against the MCE rule (ISO/IEC 29500-3 §10.1.2).
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L1302; `Dom::parent` (xmllinq/mod.rs:597) L1304; `Dom::descendants` (xmllinq/mod.rs:668) L1308,1309; `Dom::ancestors_and_self` (xmllinq/mod.rs:787) L1319; `Dom::name_is` (xmllinq/mod.rs:535) L1322; `Dom::elements` (xmllinq/mod.rs:640) L1335; `choice_requires_understood` (comparer/finalize.rs:1479) L1338; `Dom::element` (xmllinq/mod.rs:655) L1339; `Dom::nodes` (xmllinq/mod.rs:604) L1341; `Dom::replace_with` (xmllinq/mod.rs:1165) L1342
- **Calls (std/external/unresolved):** `.any`, `.find`, `.into_iter`, `.is_empty`, `.is_none`, `.or_else`, `MC::name`, `Some`, `W::drawing`, `W::pict`
- **Called by:** `coverage_tests::alternate_content_selects_first_bound_choice_or_fallback` (comparer/finalize.rs:15927); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `sanitize_sdt_properties`  (L1354-1389, pub)
`pub fn sanitize_sdt_properties(dom: &mut Dom, root: NodeId)`

- **Does:** Strip non-standard `w:`-namespace children from `<w:sdtPr>`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1379; `Dom::elements` (xmllinq/mod.rs:640) L1380; `Dom::name` (xmllinq/mod.rs:526) L1381; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1382; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1383; `Dom::remove` (xmllinq/mod.rs:1118) L1385
- **Calls (std/external/unresolved):** `.contains`, `Some`, `W::name`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `unwrap_content_controls`  (L1400-1445, pub)
`pub fn unwrap_content_controls(dom: &mut Dom, root: NodeId)`

- **Does:** Unwrap every `w:sdt` under `root` to its `sdtContent` children, deepest first.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1402,1411; `Dom::parent` (xmllinq/mod.rs:597) L1423,1433; `Dom::element` (xmllinq/mod.rs:655) L1426; `Dom::nodes` (xmllinq/mod.rs:604) L1427; `Dom::remove` (xmllinq/mod.rs:1118) L1434,1438; `Dom::add_before_self` (xmllinq/mod.rs:1130) L1435
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.copied`, `.filter`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `Some`, `Vec::new`, `W::sdt_content`, `W::sdt`
- **Called by:** `unwrap_content_controls_in_pure_revisions` (comparer/finalize.rs:1454); `coverage_tests::content_controls_unwrap_deepest_first_and_keep_text_order` (comparer/finalize.rs:15897)

#### `unwrap_content_controls_in_pure_revisions`  (L1454-1473, pub)
`pub fn unwrap_content_controls_in_pure_revisions(dom: &mut Dom, root: NodeId)`

- **Does:** M390 — [`unwrap_content_controls`], but only under pure-I / pure-D / MIX residual paragraphs.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1455; `Dom::descendants` (xmllinq/mod.rs:668) L1458,1460,1462,1468; `para_mark_revision` (comparer/finalize.rs:7631) L1461,1463; `unwrap_content_controls` (comparer/finalize.rs:1400) L1471
- **Calls (std/external/unresolved):** `.is_empty`, `Some`, `W::body`, `W::del`, `W::ins`, `W::p`, `W::sdt`
- **Called by:** `coverage_tests::residual_control_unwrap_keeps_equal_form_controls` (comparer/finalize.rs:15912); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `choice_requires_understood`  (L1479-1486)
`fn choice_requires_understood(dom: &Dom, choice: NodeId) -> bool`

- **Does:** True if every namespace prefix listed in a `mc:Choice`'s `@Requires` is bound by an in-scope `xmlns:*` declaration.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L1480; `XNamespace::none` (xmllinq/mod.rs:116) L1480; `prefix_in_scope` (comparer/finalize.rs:1490) L1485
- **Calls (std/external/unresolved):** `.all`, `.name`, `.split_whitespace`
- **Called by:** `resolve_alternate_content` (comparer/finalize.rs:1301)

#### `prefix_in_scope`  (L1490-1496)
`fn prefix_in_scope(dom: &Dom, node: NodeId, prefix: &str) -> bool`

- **Does:** True if `prefix` is bound by an `xmlns:prefix` declaration on `node` or any of its ancestors (XML namespace scoping).
- **Calls (crate):** `Dom::ancestors_and_self` (xmllinq/mod.rs:787) L1491; `Dom::attributes` (xmllinq/mod.rs:899) L1492; `Dom::is_namespace_declaration` (xmllinq/mod.rs:1012) L1494; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1494
- **Calls (std/external/unresolved):** `.any`, `.iter`
- **Called by:** `bind_compatibility_prefixes` (comparer/finalize.rs:1026); `choice_requires_understood` (comparer/finalize.rs:1479)

#### `coalesce_all_paragraphs`  (L1503-1514, pub)
`pub fn coalesce_all_paragraphs(dom: &mut Dom, root: NodeId)`

- **Does:** Apply [`coalesce_adjacent_runs`] to every `w:p` under `root`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1504; `coalesce_adjacent_runs` (comparer/finalize.rs:1222) L1506; `Dom::nodes` (xmllinq/mod.rs:604) L1507; `Dom::remove_nodes` (xmllinq/mod.rs:1110) L1509; `Dom::add` (xmllinq/mod.rs:1074) L1511
- **Calls (std/external/unresolved):** `Some`, `W::p`
- **Called by:** `finalize_notes_part` (comparer/footnotes.rs:716); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `fix_paragraph_mark_revision_order`  (L1526-1557, pub)
`pub fn fix_paragraph_mark_revision_order(dom: &mut Dom, root: NodeId)`

- **Does:** Enforce the OOXML child order for paragraph-mark revision markers.
- **Calls (crate):** `XName::local_name` (xmllinq/mod.rs:208) (by name) L1529; `Dom::descendants` (xmllinq/mod.rs:668) L1533; `Dom::element` (xmllinq/mod.rs:655) L1534,1548,1550; `Dom::elements` (xmllinq/mod.rs:640) L1538; `is_marker` (ambiguous: `is_marker` (edit/ranges.rs:16); `is_marker` (markdown/from_docx/revise.rs:59)) L1541; `Dom::remove` (xmllinq/mod.rs:1118) L1544,1551; `Dom::add_first` (xmllinq/mod.rs:1101) L1545; `Dom::add_before_self` (xmllinq/mod.rs:1130) L1553; `Dom::add` (xmllinq/mod.rs:1074) L1554
- **Calls (std/external/unresolved):** `.as_ref`, `.collect`, `.filter`, `.into_iter`, `.map`, `.name`, `.or_else`, `.rev`, `Some`, `W::name`, `W::p_pr_change`, `W::p_pr`, `W::sect_pr`
- **Macros:** `matches!`
- **Called by:** `finalize_notes_part` (comparer/footnotes.rs:716); `produce_note_redline` (comparer/footnotes.rs:1015); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `move_paragraph_properties_first`  (L1566-1584, pub)
`pub fn move_paragraph_properties_first(dom: &mut Dom, node: NodeId)`

- **Does:** Move every paragraph's `w:pPr` to the FIRST child position (OOXML schema requires `w:pPr` first under `w:p`).
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L1567,1571; `Dom::nodes` (xmllinq/mod.rs:604) L1568,1579; `Dom::remove` (xmllinq/mod.rs:1118) L1575; `Dom::add_first` (xmllinq/mod.rs:1101) L1576; `Dom::is_element` (xmllinq/mod.rs:502) L1580; `move_paragraph_properties_first` (comparer/finalize.rs:1566) L1581
- **Calls (std/external/unresolved):** `.as_ref`, `.iter`, `.position`, `Some`, `W::p_pr`, `W::p`
- **Called by:** `move_paragraph_properties_first` (comparer/finalize.rs:1566); `finalize_notes_part` (comparer/footnotes.rs:716); `produce_note_redline` (comparer/footnotes.rs:1015); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215); `coverage_ruby_math_ownership::ruby_reassembly_retains_reading_base_and_property_owners` (comparer/produce.rs:2536)

#### `unwrap_hyperlinks_to_styled_runs`  (L1597-1636, pub)
`pub fn unwrap_hyperlinks_to_styled_runs(dom: &mut Dom, root: NodeId)`

- **Does:** Word Compare unwraps TOC/body `w:hyperlink` wrappers into runs with `w:rStyle w:val="Hyperlink"` (file_21: Word has 0 hyperlinks + 251 Hyperlink-styled runs; we kept 133 live hyperlinks).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1598,1605; `Dom::attribute` (xmllinq/mod.rs:874) L1601; `Dom::element` (xmllinq/mod.rs:655) L1607,1615; `Dom::new_element` (xmllinq/mod.rs:478) L1610,1616; `Dom::add_first` (xmllinq/mod.rs:1101) L1611,1618; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1617; `wrap_revised_hyperlink_as_field` (comparer/finalize.rs:1641) L1627; `Dom::nodes` (xmllinq/mod.rs:604) L1629; `Dom::remove` (xmllinq/mod.rs:1118) L1631,1634; `Dom::add_before_self` (xmllinq/mod.rs:1130) L1632
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_none`, `R::name`, `Some`, `W::hyperlink`, `W::name`, `W::r_pr`, `W::r`, `W::val`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `wrap_revised_hyperlink_as_field`  (L1641-1733)
`fn wrap_revised_hyperlink_as_field(dom: &mut Dom, hl: NodeId)`

- **Does:** M470 helper — when `hl`'s text content is wholly deleted (or wholly inserted), synthesize the HYPERLINK field wrapper runs inside the same revision context.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L1642,1689; `Dom::descendants` (xmllinq/mod.rs:668) L1645,1652,1653; `Dom::value_str` (xmllinq/mod.rs:1195) L1646; `Dom::ancestors_and_self` (xmllinq/mod.rs:787) L1647; `Dom::name_is` (xmllinq/mod.rs:535) L1650,1656,1660; `Dom::parent` (xmllinq/mod.rs:597) L1654; `Dom::new_element` (xmllinq/mod.rs:478) L1677,1678,1684,1685; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1679; `Dom::add` (xmllinq/mod.rs:1074) L1680,1694,1715,1726,1727,1728...; `Dom::add_text` (xmllinq/mod.rs:1087) L1693; `Dom::add_before_self` (xmllinq/mod.rs:1130) L1701,1702,1703,1721,1722,1723; `Dom::add_after_self` (xmllinq/mod.rs:1147) L1704; `Dom::elements` (xmllinq/mod.rs:640) L1710,1719; `Dom::add_first` (xmllinq/mod.rs:1101) L1712,1713,1714
- **Calls (std/external/unresolved):** `.any`, `.copied`, `.filter`, `.first`, `.is_empty`, `.is_some`, `.iter`, `.last`, `.map`, `.push_str`, `.to_string`, `.trim`, `Some`, `W::del_text`, `W::del`, `W::ins`, `W::name`, `W::r`, `W::t`, `mk_fld`
- **Macros:** `format!`
- **Called by:** `unwrap_hyperlinks_to_styled_runs` (comparer/finalize.rs:1597); `deep_production_path_tests::revised_internal_hyperlink_fields_preserve_result_text_and_revision_attribution` (comparer/finalize.rs:22582)

#### `fold_short_list_label_into_empty_pure_del`  (L1744-1810, pub)
`pub fn fold_short_list_label_into_empty_pure_del(dom: &mut Dom, root: NodeId)`

- **Does:** M440 — fold a short pure-I numbered list label (`a` / `test` / `b`, ≤ 2 atoms) sitting directly before a mark-only pure-D into a MIX `w:del` on the next content pure-D.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1745,1786,1789; `Dom::elements` (xmllinq/mod.rs:640) L1749,1791,1797; `Dom::name_is` (xmllinq/mod.rs:535) L1752,1758,1780,1798; `para_is_pure_inserted` (comparer/finalize.rs:4216) L1761; `para_is_pure_deleted` (comparer/finalize.rs:2900) L1761,1780; `para_has_real_del` (comparer/finalize.rs:7854) L1765,1780; `para_has_no_text` (comparer/finalize.rs:3500) L1765; `para_mark_revision` (comparer/finalize.rs:7631) L1768; `para_has_live_numpr` (comparer/finalize.rs:7995) L1772; `para_word_atom_count` (comparer/finalize.rs:8013) L1775; `Dom::remove` (xmllinq/mod.rs:1118) L1787,1802; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1790; `Dom::add_before_self` (xmllinq/mod.rs:1130) L1792; `Dom::add` (xmllinq/mod.rs:1074) L1794,1799
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.contains`, `.copied`, `.filter`, `.first`, `.into_iter`, `.iter`, `.len`, `.saturating_sub`, `W::body`, `W::del`, `W::p_pr`, `W::p`, `W::sect_pr`
- **Called by:** `coverage_tests::short_list_label_folds_only_into_empty_deleted_mark_with_deleted_tail` (comparer/finalize.rs:16468); `finalizer_distinct_boundary_contract_tests::the_short_list_label_uses_only_an_empty_deleted_carrier_with_continuation` (comparer/finalize.rs:30291); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `strip_trailing_bare_empty_after_pure_i_dominant`  (L1829-1881, pub)
`pub fn strip_trailing_bare_empty_after_pure_i_dominant(dom: &mut Dom, root: NodeId)`

- **Does:** M448 (diff_after8 × doc_with_spacing ~75.6 / docxodus 100): Word ends pure-I-dominant body with pure-D residual only (`…IDD`).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1830,1855,1858; `Dom::elements` (xmllinq/mod.rs:640) L1833; `Dom::name_is` (xmllinq/mod.rs:535) L1836,1842,1862,1868; `para_has_no_text` (comparer/finalize.rs:3500) L1845; `Dom::descendants` (xmllinq/mod.rs:668) L1848,1849; `para_is_pure_deleted` (comparer/finalize.rs:2900) L1862,1873; `para_is_pure_inserted` (comparer/finalize.rs:4216) L1871; `Dom::remove` (xmllinq/mod.rs:1118) L1880
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter`, `.into_iter`, `.is_empty`, `.is_none`, `.len`, `Some`, `W::body`, `W::del`, `W::ins`, `W::p_pr`, `W::p`, `W::sect_pr`, `W::spacing_el`
- **Called by:** `coverage_tests::trailing_spaced_empty_requires_insert_dominance_and_deleted_predecessor` (comparer/finalize.rs:16600); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `promote_live_numpr_on_pure_d_from_pprchange`  (L1908-2008, pub)
`pub fn promote_live_numpr_on_pure_d_from_pprchange(dom: &mut Dom, root: NodeId)`

- **Does:** M442 (list_spacer1 × list_with_break_exported_broken residual ~80.8 / docxodus 94.6): last pure-D "14.11…" has Word live `numPr` (numId from the pure-I list residual) + `pPrChange` parking base StandardL1/numId=0.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1909,1932,1933,1934,1965,1966...; `Dom::elements` (xmllinq/mod.rs:640) L1920,1949; `Dom::name_is` (xmllinq/mod.rs:535) L1921,1950; `para_is_pure_inserted` (comparer/finalize.rs:4216) L1921; `para_has_live_numpr` (comparer/finalize.rs:7995) L1924; `para_word_atom_count` (comparer/finalize.rs:8013) L1927; `Dom::attribute` (xmllinq/mod.rs:874) L1935,1967,1995; `para_is_pure_deleted` (comparer/finalize.rs:2900) L1953; `para_has_real_del` (comparer/finalize.rs:7854) L1953; `para_revision_body_text` (comparer/finalize.rs:7859) L1958; `body_looks_like_list_residual_label` (comparer/finalize.rs:2033) L1959; `Dom::new_element` (xmllinq/mod.rs:478) L1998,1999,2001; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L2000,2002; `Dom::add` (xmllinq/mod.rs:1074) L2003,2004; `Dom::add_first` (xmllinq/mod.rs:1101) L2006
- **Calls (std/external/unresolved):** `.and_then`, `.contains`, `.is_none`, `.is_some_and`, `.is_some`, `.starts_with`, `.to_ascii_lowercase`, `.to_string`, `.unwrap_or`, `Some`, `W::body`, `W::name`, `W::num_pr`, `W::p_pr_change`, `W::p_pr`, `W::p_style`, `W::p`, `W::val`
- **Called by:** `coverage_tests::promoted_list_numbering_uses_inserted_id_and_deleted_level` (comparer/finalize.rs:16651); `finalizer_distinct_boundary_contract_tests::historical_legal_numbering_uses_the_donor_id_and_keeps_authored_old_level` (comparer/finalize.rs:30337); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `body_looks_like_section_number`  (L2013-2029)
`fn body_looks_like_section_number(text: &str) -> bool`

- **Does:** True when body starts like a legal section id: one+ digits, a dot, one+ digits (e.g.
- **Calls (crate):** `Parser::peek` (markdown/math.rs:804) (by name) L2017,2028
- **Calls (std/external/unresolved):** `.chars`, `.is_ascii_digit`, `.next`, `.peekable`, `.trim_start`, `Some`
- **Macros:** `matches!`
- **Called by:** `body_looks_like_list_residual_label` (comparer/finalize.rs:2033); `coverage_tests::section_and_list_residual_label_boundaries` (comparer/finalize.rs:15721)

#### `body_looks_like_list_residual_label`  (L2033-2051)
`fn body_looks_like_list_residual_label(text: &str) -> bool`

- **Does:** list_spacer section ids **or** list_def residual labels ("Num 4") without live ListParagraph style.
- **Calls (crate):** `body_looks_like_section_number` (comparer/finalize.rs:2013) L2034
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.is_ascii_digit`, `.is_empty`, `.next`, `.split_whitespace`, `.strip_prefix`, `.to_ascii_lowercase`, `.trim`, `.unwrap`
- **Called by:** `promote_live_numpr_on_pure_d_from_pprchange` (comparer/finalize.rs:1908); `coverage_tests::section_and_list_residual_label_boundaries` (comparer/finalize.rs:15721)

#### `para_side_word_count`  (L2055-2073)
`fn para_side_word_count(dom: &Dom, p: NodeId, ins_side: bool) -> usize`

- **Does:** Word-count of `w:t` under pure-I runs (`ins_side`) or `w:delText` under pure-D runs (`!ins_side`) inside a paragraph (MIX-aware).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2058,2060,2066; `Dom::value_str` (xmllinq/mod.rs:1195) L2061,2067
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.is_empty`, `.push_str`, `.push`, `.split_whitespace`, `Some`, `String::new`, `W::del_text`, `W::del`, `W::ins`, `W::t`
- **Called by:** `last_pure_del_spacing_to_pprchange` (comparer/finalize.rs:5205); `promote_live_jc_from_pprchange_on_body_mix` (comparer/finalize.rs:6516); `park_jc_on_first_short_title_mix_from_body` (comparer/finalize.rs:13148)

#### `para_keeps_unchanged_words`  (L2076-2085)
`fn para_keeps_unchanged_words(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when a paragraph holds live text outside any `w:ins`/`w:del`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2077; `Dom::value_str` (xmllinq/mod.rs:1195) L2078; `Dom::ancestors` (xmllinq/mod.rs:770) L2079; `Dom::name_is` (xmllinq/mod.rs:535) L2083
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_empty`, `.take_while`, `.trim`, `Some`, `W::del`, `W::ins`, `W::t`
- **Called by:** `last_pure_del_spacing_to_pprchange` (comparer/finalize.rs:5205)

#### `ensure_pure_i_list_snug_spacing`  (L2113-2172, pub)
`pub fn ensure_pure_i_list_snug_spacing(dom: &mut Dom, root: NodeId)`

- **Does:** M439 (list_def_mix × list_numbering_reimport ~50.5 / docxodus 90): pure-I list items with live `numPr` and **no** spacing inherit bloated package `pPrDefault` (before=240 after=240 line=288) under LO.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2119; `para_is_pure_inserted` (comparer/finalize.rs:4216) L2120; `Dom::element` (xmllinq/mod.rs:655) L2123,2126,2129,2132,2153,2156...; `Dom::attribute` (xmllinq/mod.rs:874) L2133; `para_word_atom_count` (comparer/finalize.rs:8013) L2139; `para_revision_body_text` (comparer/finalize.rs:7859) L2142; `Dom::new_element` (xmllinq/mod.rs:478) L2159; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L2160,2161,2162,2163; `Dom::add_before_self` (xmllinq/mod.rs:1130) L2165,2167; `Dom::add` (xmllinq/mod.rs:1074) L2169
- **Calls (std/external/unresolved):** `.chars`, `.clone`, `.copied`, `.count`, `.entry`, `.eq_ignore_ascii_case`, `.get`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.or_insert`, `.push`, `.to_string`, `.trim`, `.unwrap_or`, `Some`, `Vec::new`, `W::name`, `W::num_pr`, `W::p_pr_change`, `W::p_pr`, `W::p_style`, `W::p`, `W::r_pr`, `W::spacing_el`, `W::val`, `std::collections::HashMap::new`
- **Called by:** `coverage_round_next_tests::list_snug_spacing_requires_three_identical_short_tokens_and_respects_schema_tail` (comparer/finalize.rs:19894); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `normalize_incomplete_spacing`  (L2218-2271, pub)
`pub fn normalize_incomplete_spacing(dom: &mut Dom, root: NodeId)`

- **Does:** Word-parity for incomplete body spacing (C3 / C5 layout).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2225; `Dom::element` (xmllinq/mod.rs:655) L2226,2229,2230; `Dom::attribute` (xmllinq/mod.rs:874) L2233,2234,2235,2236; `Dom::remove` (xmllinq/mod.rs:1118) L2256; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L2260,2261,2262,2263,2265,2269
- **Calls (std/external/unresolved):** `.is_empty`, `.is_some`, `.push`, `.unwrap_or`, `Some`, `Vec::new`, `W::name`, `W::num_pr`, `W::p_pr`, `W::p`, `W::spacing_el`
- **Called by:** `closing_mark_takes_revised_properties` (comparer/finalize.rs:3200); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `strip_redundant_demo_default_spacing`  (L2308-2389, pub)
`pub fn strip_redundant_demo_default_spacing( dom: &mut Dom, root: NodeId, default_lines: (Option<&str>, Option<&str>), )`

- **Does:** Word Compare omits body-level `w:spacing` that only restates the common demo-doc default (line=276, optional after=200 / lineRule=auto).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2315; `Dom::element` (xmllinq/mod.rs:655) L2316,2319,2352,2353,2364,2365...; `Dom::attribute` (xmllinq/mod.rs:874) L2322,2323,2324,2325,2369; `para_is_pure_deleted` (comparer/finalize.rs:2900) L2343,2381; `para_is_pure_inserted` (comparer/finalize.rs:4216) L2354; `para_word_atom_count` (comparer/finalize.rs:8013) L2358; `Dom::remove` (xmllinq/mod.rs:1118) L2387
- **Calls (std/external/unresolved):** `.all`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some`, `.iter`, `.parse`, `.push`, `.unwrap_or`, `Some`, `Vec::new`, `W::name`, `W::p_pr_change`, `W::p_pr`, `W::p_style`, `W::p`, `W::spacing_el`
- **Called by:** `closing_mark_takes_revised_properties` (comparer/finalize.rs:3200); `coverage_round_next_tests::default_spacing_strips_only_the_selected_source_default_and_exact_shape` (comparer/finalize.rs:20237); `coverage_round_next_tests::deleted_heading_spacing_threshold_keeps_missing_metrics_and_real_styles` (comparer/finalize.rs:20272); `demo_title_pilcrow_history_tests::authored_title_snapshot_survives_demo_default_layout_normalization` (comparer/finalize.rs:28820); `finalizer_distinct_boundary_contract_tests::demo_spacing_respects_authored_metrics_history_and_source_defaults` (comparer/finalize.rs:29593); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `strip_redundant_normal_pstyle_and_bidi`  (L2404-2448, pub)
`pub fn strip_redundant_normal_pstyle_and_bidi(dom: &mut Dom, root: NodeId)`

- **Does:** M367 (sdts×shape_group −4.3): Word omits body `w:pStyle=Normal` and `w:bidi=0` on pure-inserted paragraphs (shape_group writes both; Word redline is bare mark-only pPr).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2406,2435; `para_is_pure_inserted` (comparer/finalize.rs:4216) L2407,2436; `Dom::element` (xmllinq/mod.rs:655) L2410,2413,2419,2439; `Dom::attribute` (xmllinq/mod.rs:874) L2414,2420; `Dom::parent` (xmllinq/mod.rs:597) L2428; `Dom::remove` (xmllinq/mod.rs:1118) L2429,2446; `Dom::elements` (xmllinq/mod.rs:640) L2440
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`, `.is_empty`, `.is_some`, `.push`, `.unwrap_or`, `Some`, `Vec::new`, `W::name`, `W::p_pr`, `W::p_style`, `W::p`, `W::val`
- **Called by:** `finalizer_distinct_boundary_contract_tests::explicit_default_direction_is_removed_only_from_inserted_layout` (comparer/finalize.rs:29534); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `strip_list_layout_from_mid_pure_del`  (L2463-2544, pub)
`pub fn strip_list_layout_from_mid_pure_del(dom: &mut Dom, root: NodeId)`

- **Does:** M376 — strip list-shape spacing / `w:jc` / mark fonts that a mid pure-D residual absorbed from a preceding pure-I list stream.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L2464,2482,2498,2502,2505,2516...; `Dom::elements` (xmllinq/mod.rs:640) L2467,2528; `Dom::name_is` (xmllinq/mod.rs:535) L2470,2479,2495; `para_is_pure_inserted` (comparer/finalize.rs:4216) L2479; `spacing_is_list_single_line` (comparer/finalize.rs:2548) L2483,2512; `para_is_pure_deleted` (comparer/finalize.rs:2900) L2495; `Dom::attribute` (xmllinq/mod.rs:874) L2506,2521; `Dom::name` (xmllinq/mod.rs:526) L2529; `Dom::parent` (xmllinq/mod.rs:597) L2540; `Dom::remove` (xmllinq/mod.rs:1118) L2541
- **Calls (std/external/unresolved):** `.collect`, `.eq_ignore_ascii_case`, `.filter`, `.into_iter`, `.is_some_and`, `.is_some`, `.len`, `.push`, `.unwrap_or`, `Vec::new`, `W::body`, `W::del`, `W::ins`, `W::jc_el`, `W::num_pr`, `W::p_pr`, `W::p_style`, `W::p`, `W::r_pr`, `W::sect_pr`, `W::spacing_el`, `W::val`
- **Called by:** `coverage_round_next_tests::list_contamination_cleanup_keeps_real_lists_and_alignment` (comparer/finalize.rs:19866); `finalizer_distinct_boundary_contract_tests::list_layout_contamination_does_not_erase_real_list_or_terminal_ownership` (comparer/finalize.rs:30155); `mid_list_optional_source_property_tests::list_layout_cleanup_handles_absent_optional_properties_without_inventing_owners` (comparer/finalize.rs:34616); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `spacing_is_list_single_line`  (L2548-2560)
`fn spacing_is_list_single_line(dom: &Dom, ppr: NodeId) -> bool`

- **Does:** `after=0` (or absent) + `line=240` + `lineRule=auto` — pure-I list residual single-line shape (broken_complex_list / Word list compare).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L2549; `Dom::attribute` (xmllinq/mod.rs:874) L2552,2553,2554,2555
- **Calls (std/external/unresolved):** `.is_empty`, `.unwrap_or`, `W::name`, `W::spacing_el`
- **Called by:** `strip_list_layout_from_mid_pure_del` (comparer/finalize.rs:2463)

#### `free_mesh_shared_title_token_in_mix`  (L2573-2756, pub)
`pub fn free_mesh_shared_title_token_in_mix(dom: &mut Dom, root: NodeId)`

- **Does:** M377 — free-mesh one shared significant `len≥4` token out of a short-title MIX as `Equal` (re-emitting it on both sides instead of ins+deleting it).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L2574,2729,2730; `Dom::elements` (xmllinq/mod.rs:640) L2577,2592; `Dom::name_is` (xmllinq/mod.rs:535) L2580,2583,2595; `Dom::descendants` (xmllinq/mod.rs:668) L2586,2587,2640,2646; `Dom::name` (xmllinq/mod.rs:526) L2605; `holds_only_text` (comparer/finalize.rs:13600) L2618; `Dom::attribute` (xmllinq/mod.rs:874) L2630,2713,2716,2719; `Dom::value_str` (xmllinq/mod.rs:1195) L2641,2647; `alnum_tokens` (comparer/finalize.rs:2759) L2653,2654; `trailing_alnum_token` (comparer/finalize.rs:7588) L2692; `split_around_token` (comparer/finalize.rs:2770) L2700; `strip_trailing_alnum_token` (comparer/finalize.rs:7606) L2703; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L2731; `Dom::parent` (xmllinq/mod.rs:597) L2735; `Dom::remove` (xmllinq/mod.rs:1118) L2736; `rebuild_title_free_mesh` (comparer/finalize.rs:2813) L2742
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.as_str`, `.collect`, `.contains`, `.count`, `.eq_ignore_ascii_case`, `.filter`, `.first`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.last`, `.len`, `.map`, `.push_str`, `.push`, `.to_string`, `Some`, `String::new`, `Vec::new`, `W::author`, `W::body`, `W::date`, `W::del_text`, `W::del`, `W::id`, `W::ins`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `W::sect_pr`, `W::t`, `author_of`
- **Called by:** `deep_production_path_tests::shared_title_mesh_executes_wholesale_and_confetti_rebuilds_with_exact_views` (comparer/finalize.rs:22226); `finalizer_distinct_boundary_contract_tests::a_short_title_mesh_cannot_consume_foreign_revision_provenance_or_unrelated_titles` (comparer/finalize.rs:30650); `interleave_terminal_source_ownership_tests::split_inserted_title_fragments_do_not_take_another_authors_pending_revision` (comparer/finalize.rs:33433); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `alnum_tokens`  (L2759-2764)
`fn alnum_tokens(text: &str) -> Vec<String>`

- **Does:** Alnum tokens lowercased.
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.is_alphanumeric`, `.is_empty`, `.map`, `.split`, `.to_ascii_lowercase`
- **Called by:** `free_mesh_shared_title_token_in_mix` (comparer/finalize.rs:2573); `free_mesh_wholesale_body_mix` (comparer/finalize.rs:13633); `free_mesh_bookended_ins_del` (comparer/finalize.rs:14073)

#### `split_around_token`  (L2770-2801)
`fn split_around_token(text: &str, label: &str) -> Option<(String, String)>`

- **Does:** Split `text` into (before, after) around first case-insensitive alnum token matching `label`.
- **Calls (std/external/unresolved):** `.char_indices`, `.collect`, `.eq_ignore_ascii_case`, `.is_alphanumeric`, `.is_empty`, `.len`, `.to_string`, `.trim`, `Some`
- **Called by:** `free_mesh_shared_title_token_in_mix` (comparer/finalize.rs:2573); `coverage_tests::token_splitting_preserves_separators_and_original_case` (comparer/finalize.rs:15744)

#### `rebuild_title_free_mesh`  (L2813-2861)
`fn rebuild_title_free_mesh( dom: &mut Dom, p: NodeId, ins_before: &str, del_prefix: &str, eq_label: &str, ins_after: &str, stamp: RevStamp<'_>, )`

- **Does:** Rebuild short-title MIX free-mesh body on `p`.
- **Calls (crate):** `add_revision_text_run` (comparer/finalize.rs:2863) L2827,2831,2859; `Dom::new_element` (xmllinq/mod.rs:478) L2846,2851; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L2848; `Dom::add` (xmllinq/mod.rs:1074) L2849,2855,2856; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L2853; `XNamespace::xml` (xmllinq/mod.rs:128) L2853; `Dom::add_text` (xmllinq/mod.rs:1087) L2854
- **Calls (std/external/unresolved):** `.is_empty`, `.name`, `.trim_end`, `Some`, `W::del`, `W::ins`, `W::r`, `W::t`
- **Macros:** `format!`
- **Called by:** `free_mesh_shared_title_token_in_mix` (comparer/finalize.rs:2573)

#### `add_revision_text_run`  (L2863-2896)
`fn add_revision_text_run( dom: &mut Dom, p: NodeId, wrapper: crate::xmllinq::XName, text: &str, deleted: bool, stamp: RevStamp<'_>, next_id: &mut u32, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L2877,2883,2888; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L2878,2879,2882,2890; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L2885; `Dom::add` (xmllinq/mod.rs:1074) L2886,2893,2894,2895; `XNamespace::xml` (xmllinq/mod.rs:128) L2890; `Dom::add_text` (xmllinq/mod.rs:1087) L2892
- **Calls (std/external/unresolved):** `.ends_with`, `.name`, `.starts_with`, `.to_string`, `Some`, `W::author`, `W::date`, `W::del_text`, `W::id`, `W::r`, `W::t`
- **Called by:** `rebuild_title_free_mesh` (comparer/finalize.rs:2813)

#### `para_is_pure_deleted`  (L2900-2913)
`fn para_is_pure_deleted(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when a paragraph has deleted content, no live (non-del) `w:t` text, and no `w:ins` — pure deleted body paragraph.
- **Calls (crate):** `para_is_pure_deleted_uncached` (comparer/finalize.rs:2915) L2906
- **Calls (std/external/unresolved):** `.and_then`, `.as_mut`, `.as_ref`, `.borrow_mut`, `.borrow`, `.copied`, `.get`, `.insert`, `.with`
- **Called by:** `fold_short_list_label_into_empty_pure_del` (comparer/finalize.rs:1744); `strip_trailing_bare_empty_after_pure_i_dominant` (comparer/finalize.rs:1829); `promote_live_numpr_on_pure_d_from_pprchange` (comparer/finalize.rs:1908); `strip_redundant_demo_default_spacing` (comparer/finalize.rs:2308); `strip_list_layout_from_mid_pure_del` (comparer/finalize.rs:2463); `strip_trailing_empty_pure_del_mark` (comparer/finalize.rs:2951); `story_final_kept_blank` (comparer/finalize.rs:3542); `relocate_title_page_last_empty_after_pure_dels` (comparer/finalize.rs:3653); `ensure_empty_pure_i_before_short_title_del` (comparer/finalize.rs:3951); `strip_empty_pure_ins_before_trailing_pure_dels` (comparer/finalize.rs:4103); `fold_whitespace_pure_ins_into_following_pure_del` (comparer/finalize.rs:4252); `fold_leading_ins_from_mix_into_preceding_pure_del` (comparer/finalize.rs:4991); +17 more

#### `para_is_pure_deleted_uncached`  (L2915-2942)
`fn para_is_pure_deleted_uncached(dom: &Dom, p: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L2916,2920,2923; `Dom::ancestors_and_self` (xmllinq/mod.rs:787) L2925; `Dom::name` (xmllinq/mod.rs:526) L2926; `Dom::value` (xmllinq/mod.rs:1185) L2935
- **Calls (std/external/unresolved):** `.as_ref`, `.is_empty`, `.trim`, `Some`, `W::del`, `W::ins`, `W::t`
- **Called by:** `para_is_pure_deleted` (comparer/finalize.rs:2900)

#### `strip_trailing_empty_pure_del_mark`  (L2951-2991, pub)
`pub fn strip_trailing_empty_pure_del_mark(dom: &mut Dom, root: NodeId)`

- **Does:** M69 — clear the paragraph-mark `w:del` on the final body paragraph when it is a content-free pure-D.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L2952,2978,2979,2980; `Dom::elements` (xmllinq/mod.rs:640) L2955,2983,2987; `Dom::name_is` (xmllinq/mod.rs:535) L2957,2960; `Dom::descendants` (xmllinq/mod.rs:668) L2964,2965,2974; `para_is_pure_deleted` (comparer/finalize.rs:2900) L2970; `para_mark_revision` (comparer/finalize.rs:7631) L2970; `Dom::remove` (xmllinq/mod.rs:1118) L2981,2984,2988
- **Calls (std/external/unresolved):** `.find`, `.is_empty`, `.iter`, `.rev`, `Some`, `W::body`, `W::del_text`, `W::del`, `W::ins`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`, `W::t`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `trailing_empty_spacing_to_pprchange`  (L3009-3076, pub)
`pub fn trailing_empty_spacing_to_pprchange( dom: &mut Dom, root: NodeId, revised_tail_spacing: Option<&[(String, String, String)]>, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** M92 — trailing empty body paragraph: Word records live `w:spacing` under `w:pPrChange` (file_30 last empty after list residual).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L3016,3035,3038,3041,3071; `Dom::elements` (xmllinq/mod.rs:640) L3019,3048,3072; `Dom::name_is` (xmllinq/mod.rs:535) L3020,3023; `para_has_no_text` (comparer/finalize.rs:3500) L3026; `Dom::descendants` (xmllinq/mod.rs:668) L3030,3031; `sorted_attributes` (comparer/finalize.rs:3477) L3044; `Dom::name` (xmllinq/mod.rs:526) L3049; `Dom::new_element` (xmllinq/mod.rs:478) L3059,3063; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L3060; `Dom::add` (xmllinq/mod.rs:1074) L3061,3068,3069; `Dom::remove` (xmllinq/mod.rs:1118) L3062,3074; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L3064,3066,3067
- **Calls (std/external/unresolved):** `.find`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.rev`, `.to_string`, `Some`, `W::author`, `W::body`, `W::date`, `W::del`, `W::id`, `W::ins`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`, `W::spacing_el`
- **Called by:** `coverage_round_next_tests::trailing_blank_spacing_records_history_only_when_source_differs` (comparer/finalize.rs:19923); `finalizer_distinct_boundary_contract_tests::the_final_empty_spacing_history_has_one_source_owner` (comparer/finalize.rs:29727); `public_body_heuristic_story_scope_tests::configured_heuristics` (comparer/finalize.rs:32397); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `closing_paragraph_properties`  (L3093-3106, pub)
`pub fn closing_paragraph_properties(dom: &mut Dom, body: NodeId) -> Option<NodeId>`

- **Does:** The paragraph properties of the story's closing paragraph, cloned without scratch attributes before atomize consumes `body`: `None` when the story does not end on a paragraph, an empty `w:pPr` when the paragraph has none.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L3094; `Dom::name_is` (xmllinq/mod.rs:535) L3098,3099; `Dom::element` (xmllinq/mod.rs:655) L3102; `drop_pt_bookkeeping` (comparer/finalize.rs:481) L3103; `Dom::new_element` (xmllinq/mod.rs:478) L3104
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.rev`, `Some`, `W::p_pr`, `W::p`, `W::sect_pr`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `merged_closing_carrier_takes_source_properties`  (L3122-3183, pub)
`pub fn merged_closing_carrier_takes_source_properties( dom: &mut Dom, root: NodeId, original: Option<NodeId>, revised: Option<NodeId>, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** Faithful production can conjoin unrelated terminal deletion/insertion streams into the original paragraph.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L3136,3150,3161,3163; `Dom::elements` (xmllinq/mod.rs:640) L3139,3169,3174; `Dom::name_is` (xmllinq/mod.rs:535) L3143,3147,3170; `para_mark_revision` (comparer/finalize.rs:7631) L3147; `content_revision_marks` (comparer/finalize.rs:6189) L3153; `project_para_properties_for_change` (comparer/formatchg.rs:329) L3156,3157; `normalize_para_properties` (comparer/formatchg.rs:349) L3158,3159,3160,3165; `Dom::remove` (xmllinq/mod.rs:1118) L3171,3175; `Dom::add` (xmllinq/mod.rs:1074) L3176,3180,3181; `rev_el` (comparer/finalize.rs:126) L3179
- **Calls (std/external/unresolved):** `.and_then`, `.find`, `.into_iter`, `.is_some_and`, `.rev`, `W::body`, `W::del`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`
- **Called by:** `finalizer_distinct_boundary_contract_tests::faithful_merged_closing_carrier_records_both_complete_property_sources` (comparer/finalize.rs:30208); `public_body_heuristic_story_scope_tests::configured_heuristics` (comparer/finalize.rs:32397); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `closing_mark_takes_revised_properties`  (L3200-3316, pub)
`pub fn closing_mark_takes_revised_properties( dom: &mut Dom, root: NodeId, original: Option<NodeId>, revised: Option<NodeId>, default_lines: (Option<&str>, Option<&str>), settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** The story's closing mark is never revised: accepted, it closes the revision; rejected, the original.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L3214,3215,3239,3306; `Dom::add` (xmllinq/mod.rs:1074) L3216,3218,3300,3303,3311,3312; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L3217; `normalize_universal_measures` (comparer/finalize.rs:11490) L3219,3315; `strip_redundant_demo_default_spacing` (comparer/finalize.rs:2308) L3220; `normalize_incomplete_spacing` (comparer/finalize.rs:2218) L3221; `Dom::element` (xmllinq/mod.rs:655) L3222,3236,3244,3248,3266,3268...; `Dom::elements` (xmllinq/mod.rs:640) L3225,3255,3256,3294,3298; `Dom::name_is` (xmllinq/mod.rs:535) L3229,3233; `Dom::add_first` (xmllinq/mod.rs:1101) L3240; `closing_mark_takes_revised_mark_formatting` (comparer/finalize.rs:3399) L3261; `normalize_para_properties` (comparer/formatchg.rs:349) L3262,3263,3265,3270; `project_para_properties_for_change` (comparer/formatchg.rs:329) L3264,3297; `sectpr_identity` (comparer/finalize.rs:11609) L3281; `Dom::remove` (xmllinq/mod.rs:1118) L3295,3299; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L3307,3309,3310
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.collect`, `.filter_map`, `.find`, `.into_iter`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.rev`, `.to_string`, `Some`, `W::author`, `W::body`, `W::date`, `W::id`, `W::ins`, `W::name`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`
- **Called by:** `closing_layout_history_regressions::already_revised_live_closing_layout_still_records_original_properties` (comparer/finalize.rs:23679); `closing_authored_section_boundary_tests::section_addition_guard_respects_tracking_source_presence_and_exact_live_identity` (comparer/finalize.rs:29217); `closing_authored_section_boundary_tests::matching_closing_history_keeps_its_existing_revision_owner_and_id` (comparer/finalize.rs:29332); `public_body_heuristic_story_scope_tests::configured_heuristics` (comparer/finalize.rs:32397); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `paragraph_mark_properties_equal`  (L3322-3393)
`fn paragraph_mark_properties_equal( dom: &mut Dom, old: Option<NodeId>, new: Option<NodeId>, ) -> bool`

- **Does:** Keep the established leaf-property normalization, but paragraph marks also own nested Office effects. _(from a code comment)_
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L3330; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L3333; `Dom::serialize_element` (xmllinq/mod.rs:1315) L3333; `Dom::root` (xmllinq/mod.rs:822) L3334; `are_run_properties_equal` (comparer/formatchg.rs:118) L3339; `Dom::elements` (xmllinq/mod.rs:640) L3379; `Dom::name_is` (xmllinq/mod.rs:535) L3381; `Dom::name` (xmllinq/mod.rs:526) L3382; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L3384; `Dom::has_elements` (xmllinq/mod.rs:862) L3385; `paragraph_mark_properties_equal::{fn signature}` (comparer/finalize.rs:3342) L3387; `text_lines::{fn nested}` (debug.rs:1736) (by name) L3392
- **Calls (std/external/unresolved):** `.collect`, `.expect`, `.filter`, `.flat_map`, `.into_iter`, `.is_some_and`, `.map`, `.sort`, `W::r_pr_change`
- **Called by:** `closing_mark_takes_revised_mark_formatting` (comparer/finalize.rs:3399); `mesh_short_label_shared_eq_with_settings` (comparer/finalize.rs:7204); `fold_midstream_demo_title_into_numbered_heading_with_source_properties` (comparer/finalize.rs:12624); `finalizer_merge_source_guard_tests::pilcrow_format_equality_includes_nested_fill_payload_but_ignores_source_scratch` (comparer/finalize.rs:31786)

#### `paragraph_mark_properties_equal::{fn signature}`  (L3342-3375)
`fn signature(dom: &Dom, node: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::attributes_iter` (xmllinq/mod.rs:886) L3343; `Dom::is_namespace_declaration` (xmllinq/mod.rs:1012) L3346; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L3347,3352,3364; `XName::local_name` (xmllinq/mod.rs:208) (by name) L3348,3353; `Dom::nodes` (xmllinq/mod.rs:604) L3359; `Dom::name` (xmllinq/mod.rs:526) L3363; `Dom::is_element` (xmllinq/mod.rs:502) L3365; `Dom::text_value` (xmllinq/mod.rs:547) L3366; `paragraph_mark_properties_equal::{fn signature}` (comparer/finalize.rs:3342) L3368; `Dom::text_value` (xmllinq/mod.rs:547) (by name) L3373
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.is_some_and`, `.map`, `.name`, `.sort`, `.starts_with`, `.to_ascii_lowercase`, `.to_string`, `.trim`, `.unwrap_or_default`
- **Macros:** `format!`
- **Called by:** `paragraph_mark_properties_equal` (comparer/finalize.rs:3322); `paragraph_mark_properties_equal::{fn signature}` (comparer/finalize.rs:3342)

#### `closing_mark_takes_revised_mark_formatting`  (L3399-3450)
`fn closing_mark_takes_revised_mark_formatting( dom: &mut Dom, ppr: NodeId, revised: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** The closing mark's own formatting follows the same way: Word writes the revised closing paragraph's mark `rPr` live and records the original's in a `rPrChange` (b4cd671041: `sz=28` live over the original's `b rFonts=Arial sz=24 u`).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L3409,3410,3413,3444; `paragraph_mark_properties_equal` (comparer/finalize.rs:3322) L3414; `Dom::new_element` (xmllinq/mod.rs:478) L3417,3431,3433; `Dom::elements` (xmllinq/mod.rs:640) L3419; `Dom::name_is` (xmllinq/mod.rs:535) L3422; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L3424,3430; `Dom::add` (xmllinq/mod.rs:1074) L3425,3438,3439,3447; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L3434,3436,3437; `Dom::replace_with` (xmllinq/mod.rs:1165) L3441; `Dom::add_before_self` (xmllinq/mod.rs:1130) L3446
- **Calls (std/external/unresolved):** `.any`, `.find_map`, `.is_some_and`, `.is_some`, `.iter`, `.to_string`, `Some`, `W::author`, `W::date`, `W::id`, `W::name`, `W::p_pr_change`, `W::r_pr`, `W::sect_pr`
- **Called by:** `closing_mark_takes_revised_properties` (comparer/finalize.rs:3200); `mesh_short_label_shared_eq_with_settings` (comparer/finalize.rs:7204); `fold_midstream_demo_title_into_numbered_heading_with_source_properties` (comparer/finalize.rs:12624); `demo_title_pilcrow_history_tests::closing_mark_history_preserves_nested_effects_and_ignores_only_existing_normalization_noise` (comparer/finalize.rs:28734)

#### `last_body_para_spacing`  (L3463-3474, pub)
`pub fn last_body_para_spacing(dom: &Dom, body: NodeId) -> Option<Vec<(String, String, String)>>`

- **Does:** Sorted `(namespace, local name, value)` attributes of the last body paragraph's `w:spacing` under `body`, if it has one.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L3464; `Dom::name_is` (xmllinq/mod.rs:535) L3468,3469; `Dom::element` (xmllinq/mod.rs:655) L3472; `sorted_attributes` (comparer/finalize.rs:3477) L3473
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.rev`, `Some`, `W::p_pr`, `W::p`, `W::sect_pr`, `W::spacing_el`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `sorted_attributes`  (L3477-3493)
`fn sorted_attributes(dom: &Dom, n: NodeId) -> Vec<(String, String, String)>`

- **Does:** Sorted `w:` attributes (scratch ids such as `pt:Unid` excluded).
- **Calls (crate):** `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L3478,3482,3485; `Dom::attributes` (xmllinq/mod.rs:899) L3479; `XName::local_name` (xmllinq/mod.rs:208) (by name) L3486
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.map`, `.sort`, `.to_string`, `W::p`
- **Called by:** `trailing_empty_spacing_to_pprchange` (comparer/finalize.rs:3009); `last_body_para_spacing` (comparer/finalize.rs:3463); `coverage_round_next_tests::trailing_blank_spacing_records_history_only_when_source_differs` (comparer/finalize.rs:19923)

#### `para_has_no_text`  (L3500-3509)
`fn para_has_no_text(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when a paragraph has no `w:t` / `w:delText` content.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L3501,3502,3503,3504,3505,3506...
- **Calls (std/external/unresolved):** `.is_empty`, `M::name`, `Some`, `W::del_text`, `W::drawing`, `W::name`, `W::pict`, `W::t`
- **Called by:** `fold_short_list_label_into_empty_pure_del` (comparer/finalize.rs:1744); `strip_trailing_bare_empty_after_pure_i_dominant` (comparer/finalize.rs:1829); `trailing_empty_spacing_to_pprchange` (comparer/finalize.rs:3009); `para_is_empty_pure_ins` (comparer/finalize.rs:3583); `is_restorable_layout_blank` (comparer/finalize.rs:3859); `revised_two_blank_table_titles` (comparer/finalize.rs:3889); `ensure_empty_pure_i_before_short_title_del` (comparer/finalize.rs:3951); `strip_last_pure_del_mark_only_ppr` (comparer/finalize.rs:5122); `mixed_spacing_to_following_empty` (comparer/finalize.rs:5308); `park_mixed_spacing_onto_trailing_pure_del` (comparer/finalize.rs:5607); `fold_short_label_ins_into_del_with_settings` (comparer/finalize.rs:7058); `merge_replaced_in_container` (comparer/finalize.rs:8626); +3 more

#### `para_has_omath`  (L3515-3518)
`fn para_has_omath(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when a paragraph carries OMML math (`m:oMath` / `m:oMathPara`).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L3516,3517
- **Calls (std/external/unresolved):** `.is_empty`, `M::name`, `Some`
- **Called by:** `para_is_visually_blank` (comparer/finalize.rs:3528); `relocate_title_page_last_empty_after_pure_dels` (comparer/finalize.rs:3653); `merge_replaced_in_container` (comparer/finalize.rs:8626)

#### `para_is_visually_blank`  (L3528-3535)
`fn para_is_visually_blank(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when a paragraph renders as a blank line: whitespace-only text and no non-text ink (math, drawings, pictures, objects, breaks).
- **Calls (crate):** `para_body_text_is_whitespace_only` (comparer/finalize.rs:4200) L3529; `para_has_omath` (comparer/finalize.rs:3515) L3530; `Dom::descendants` (xmllinq/mod.rs:668) L3531,3532,3533,3534
- **Calls (std/external/unresolved):** `.is_empty`, `Some`, `W::drawing`, `W::name`, `W::pict`
- **Called by:** `story_final_kept_blank` (comparer/finalize.rs:3542); `fold_whitespace_pure_ins_into_following_pure_del` (comparer/finalize.rs:4252); `merge_replaced_in_container` (comparer/finalize.rs:8626)

#### `story_final_kept_blank`  (L3542-3559)
`fn story_final_kept_blank(dom: &Dom, kids: &[NodeId]) -> usize`

- **Does:** 1 when the body ends on a live, blank paragraph after a pure-D run: the story-final mark the unrelated-sources pairing kept (A's and B's final empty paragraphs).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L3547,3548; `para_mark_revision` (comparer/finalize.rs:7631) L3549,3550; `Dom::name_is` (xmllinq/mod.rs:535) L3553,3556; `para_is_visually_blank` (comparer/finalize.rs:3528) L3554; `deep_production_path_tests::live` (comparer/finalize.rs:20923) L3555; `para_is_pure_deleted` (comparer/finalize.rs:2900) L3557
- **Calls (std/external/unresolved):** `.is_empty`, `Some`, `W::del`, `W::ins`, `W::p`, `usize::from`
- **Called by:** `strip_empty_pure_ins_before_trailing_pure_dels` (comparer/finalize.rs:4103); `fold_whitespace_pure_ins_into_following_pure_del` (comparer/finalize.rs:4252)

#### `para_has_repeated_phrase`  (L3565-3580)
`fn para_has_repeated_phrase(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when body text repeats a ≥4-word phrase (hummingbird wrap fingerprint).
- **Calls (crate):** `para_revision_body_text` (comparer/finalize.rs:7859) L3566
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.insert`, `.is_empty`, `.join`, `.len`, `.saturating_sub`, `.split_whitespace`, `.to_ascii_lowercase`, `std::collections::HashSet::new`
- **Called by:** `should_fold_multi_del_at_document_scale` (comparer/finalize.rs:8025); `merge_replaced_in_container` (comparer/finalize.rs:8626)

#### `para_is_empty_pure_ins`  (L3583-3607)
`fn para_is_empty_pure_ins(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when `p` is an empty pure-inserted paragraph (no text, has ins, no del).
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L3584; `para_has_no_text` (comparer/finalize.rs:3500) L3587; `Dom::descendants` (xmllinq/mod.rs:668) L3591,3596,3601,3602; `para_mark_revision` (comparer/finalize.rs:7631) L3591,3596
- **Calls (std/external/unresolved):** `.is_empty`, `Some`, `W::del`, `W::drawing`, `W::ins`, `W::p`, `W::tbl`
- **Called by:** `strip_trailing_empty_pure_ins` (comparer/finalize.rs:3620); `strip_empty_pure_ins_before_trailing_pure_dels` (comparer/finalize.rs:4103)

#### `strip_trailing_empty_pure_ins`  (L3620-3631, pub)
`pub fn strip_trailing_empty_pure_ins(dom: &mut Dom, root: NodeId)`

- **Does:** M83a — drop trailing empty pure-inserted body paragraph (file_23).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L3621; `Dom::elements` (xmllinq/mod.rs:640) L3624; `Dom::name_is` (xmllinq/mod.rs:535) L3625; `para_is_empty_pure_ins` (comparer/finalize.rs:3583) L3628; `Dom::remove` (xmllinq/mod.rs:1118) L3629
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.rev`, `W::body`, `W::sect_pr`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `relocate_title_page_last_empty_after_pure_dels`  (L3653-3773, pub)
`pub fn relocate_title_page_last_empty_after_pure_dels(dom: &mut Dom, root: NodeId)`

- **Does:** M438 (doc_with_spaces × doc_with_spacing ~67 / docxodus 100): Word title-page pure-I next × short pure-D base is `I…date e×6 DD E` (trailing bare empty).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L3654,3687; `Dom::elements` (xmllinq/mod.rs:640) L3657,3760; `Dom::name_is` (xmllinq/mod.rs:535) L3660,3667,3675,3686,3697,3719...; `para_is_pure_deleted` (comparer/finalize.rs:2900) L3668,3675; `para_body_text_is_whitespace_only` (comparer/finalize.rs:4200) L3669,3699,3721,3749; `para_is_pure_inserted` (comparer/finalize.rs:4216) L3687,3698,3720,3748; `para_has_omath` (comparer/finalize.rs:3515) L3700; `Dom::descendants` (xmllinq/mod.rs:668) L3701,3702; `para_revision_body_text` (comparer/finalize.rs:7859) L3726; `strip_para_revision_marks` (comparer/finalize.rs:3776) L3754; `Dom::remove` (xmllinq/mod.rs:1118) L3764; `Dom::parent` (xmllinq/mod.rs:597) L3766; `Dom::add_before_self` (xmllinq/mod.rs:1130) L3767; `Dom::add` (xmllinq/mod.rs:1074) L3770
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.collect`, `.contains`, `.count`, `.filter`, `.find`, `.into_iter`, `.is_ascii_digit`, `.is_empty`, `.is_some`, `.iter`, `.len`, `.position`, `.rev`, `.saturating_sub`, `.to_ascii_lowercase`, `Some`, `W::body`, `W::drawing`, `W::name`, `W::p_pr`, `W::p`, `W::sect_pr`
- **Called by:** `coverage_round_two_tests::title_page_relocation_preserves_six_spacers_and_moves_only_last` (comparer/finalize.rs:17114); `finalizer_distinct_boundary_contract_tests::title_page_empty_relocation_is_scoped_to_an_authored_cover_tail` (comparer/finalize.rs:30601); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `strip_para_revision_marks`  (L3776-3821)
`fn strip_para_revision_marks(dom: &mut Dom, p: NodeId)`

- **Does:** Drop ins/del wrappers and para-mark revisions from a paragraph (bare EQ/empty).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L3778,3797,3809,3817; `Dom::descendants` (xmllinq/mod.rs:668) L3785,3787; `Dom::parent` (xmllinq/mod.rs:597) L3793; `Dom::remove` (xmllinq/mod.rs:1118) L3799,3803,3815,3818; `Dom::add_before_self` (xmllinq/mod.rs:1130) L3800; `Dom::element` (xmllinq/mod.rs:655) L3806,3807
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.dedup`, `.extend`, `.filter`, `.into_iter`, `.is_empty`, `.is_none`, `.name`, `.sort_by_key`, `Some`, `W::del`, `W::ins`, `W::p_pr`, `W::r_pr`, `std::cmp::Reverse`
- **Macros:** `matches!`
- **Called by:** `relocate_title_page_last_empty_after_pure_dels` (comparer/finalize.rs:3653); `fold_midstream_demo_title_into_numbered_heading_with_source_properties::{fn clean_mark_properties}` (comparer/finalize.rs:12740)

#### `make_empty_pure_ins_para`  (L3824-3854)
`fn make_empty_pure_ins_para( dom: &mut Dom, template: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, ) -> NodeId`

- **Does:** Restore a revised layout blank with its own properties and a new insert mark.
- **Calls (crate):** `Dom::clone_subtree` (xmllinq/mod.rs:1231) L3830; `remove_powertools_scratch_markup` (comparer/finalize.rs:1060) L3831; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L3835,3848,3850,3851; `Dom::element` (xmllinq/mod.rs:655) L3837,3842; `Dom::new_element` (xmllinq/mod.rs:478) L3838,3843,3847; `Dom::add_first` (xmllinq/mod.rs:1101) L3839,3852; `Dom::add` (xmllinq/mod.rs:1074) L3844
- **Calls (std/external/unresolved):** `.to_string`, `.unwrap_or_else`, `Some`, `W14::name`, `W::author`, `W::date`, `W::id`, `W::ins`, `W::p_pr`, `W::r_pr`
- **Called by:** `ensure_empty_pure_i_before_short_title_del` (comparer/finalize.rs:3951)

#### `is_restorable_layout_blank`  (L3859-3876)
`fn is_restorable_layout_blank(dom: &Dom, p: NodeId) -> bool`

- **Does:** Only layout properties and property-only empty runs are safe to restore. _(from a code comment)_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L3860,3864,3867,3870; `para_has_no_text` (comparer/finalize.rs:3500) L3860; `Dom::elements` (xmllinq/mod.rs:640) L3863,3868; `Dom::descendants` (xmllinq/mod.rs:668) L3865; `element_has_tracked_revisions` (revision_processor.rs:89) L3866,3871
- **Calls (std/external/unresolved):** `.all`, `.into_iter`, `.is_empty`, `Some`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `W::sect_pr`
- **Called by:** `revised_two_blank_table_titles` (comparer/finalize.rs:3889)

#### `revised_two_blank_table_titles`  (L3889-3930, pub)
`pub fn revised_two_blank_table_titles(dom: &mut Dom, body: NodeId) -> HashMap<String, [NodeId; 2]>`

- **Does:** Titles that uniquely identify the revised source's two-blank pretable layout.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L3890; `Dom::name_is` (xmllinq/mod.rs:535) L3899,3905,3914,3919; `para_has_no_text` (comparer/finalize.rs:3500) L3899,3905; `revised_two_blank_table_titles::{fn text}` (comparer/finalize.rs:3891) L3900,3908; `is_restorable_layout_blank` (comparer/finalize.rs:3859) L3914; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L3923,3924
- **Calls (std/external/unresolved):** `.entry`, `.enumerate`, `.get`, `.insert`, `.is_empty`, `.is_some_and`, `.iter`, `.or_default`, `.trim`, `HashMap::::new`, `HashMap::new`, `Some`, `W::p`, `W::tbl`
- **Called by:** `deep_production_path_tests::spacer_context` (comparer/finalize.rs:21709); `finalizer_remaining_source_boundary_tests::two_authored_layout_blanks_are_restored_only_at_the_shared_table_boundary` (comparer/finalize.rs:31283); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `revised_two_blank_table_titles::{fn text}`  (L3891-3896)
`fn text(dom: &Dom, p: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L3892; `Dom::value` (xmllinq/mod.rs:1185) L3894
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`, `Some`, `W::t`
- **Called by:** `revised_two_blank_table_titles` (comparer/finalize.rs:3889); `ParagraphPropertySources::for_mix` (comparer/finalize.rs:13124); `coverage_tests::deleted_text_conversion_stops_at_nested_revision_owners` (comparer/finalize.rs:15823); `coverage_tests::content_controls_unwrap_deepest_first_and_keep_text_order` (comparer/finalize.rs:15897); `coverage_tests::residual_control_unwrap_keeps_equal_form_controls` (comparer/finalize.rs:15912); `coverage_tests::alternate_content_selects_first_bound_choice_or_fallback` (comparer/finalize.rs:15927); `coverage_tests::cleanup_spacing_removes_only_redundant_mixed_history` (comparer/finalize.rs:16254); `coverage_tests::cleanup_default_alignment_keeps_real_layout_history` (comparer/finalize.rs:16294); `coverage_tests::cleanup_deleted_spacing_promotes_mid_body_but_preserves_last_history` (comparer/finalize.rs:16328); `coverage_tests::short_label_free_mesh_preserves_deleted_prefix_and_inserted_whitespace` (comparer/finalize.rs:16390); `coverage_tests::residual_label_zip_adopts_deleted_list_layout_and_moves_empty_spacer` (comparer/finalize.rs:16438); `coverage_tests::short_list_label_folds_only_into_empty_deleted_mark_with_deleted_tail` (comparer/finalize.rs:16468); +8 more

#### `ensure_empty_pure_i_before_short_title_del`  (L3951-4081, pub)
`pub fn ensure_empty_pure_i_before_short_title_del( dom: &mut Dom, root: NodeId, revised_titles: &HashMap<String, [NodeId; 2]>, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** M392 (file_36×file_37 residual ~66): Word keeps empty pure-I spacers between content pure-I short title and pure-D short title before a table.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L3958,4037,4049,4057; `Dom::elements` (xmllinq/mod.rs:640) L3962,4041; `Dom::name_is` (xmllinq/mod.rs:535) L3965,3971,4031; `para_is_pure_inserted` (comparer/finalize.rs:4216) L3974,4034; `para_is_pure_deleted` (comparer/finalize.rs:2900) L3974,4019,4033; `para_body_text_is_whitespace_only` (comparer/finalize.rs:4200) L3978; `Dom::descendants` (xmllinq/mod.rs:668) L3981; `Dom::value` (xmllinq/mod.rs:1185) L3984; `para_word_atom_count` (comparer/finalize.rs:8013) L3990; `para_body_alnum_len` (comparer/finalize.rs:7889) L3991; `para_looks_like_demo_title` (comparer/finalize.rs:7906) L3992; `para_has_heading_or_title_style` (comparer/finalize.rs:7923) L3993; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L4010,4015; `para_has_no_text` (comparer/finalize.rs:3500) L4011,4032; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L4014,4017; `make_empty_pure_ins_para` (comparer/finalize.rs:3824) L4026; `Dom::add_before_self` (xmllinq/mod.rs:1130) L4027,4042; `Dom::new_element` (xmllinq/mod.rs:478) L4040,4052,4058; `Dom::add` (xmllinq/mod.rs:1074) L4044; `Dom::add_first` (xmllinq/mod.rs:1101) L4053,4071; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L4059,4061,4066
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.contains`, `.copied`, `.filter`, `.first`, `.get`, `.into_iter`, `.is_empty`, `.is_none`, `.len`, `.map`, `.saturating_sub`, `.to_string`, `Some`, `W::author`, `W::body`, `W::date`, `W::del`, `W::id`, `W::ins`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`, `W::t`, `W::tbl`
- **Macros:** `matches!`
- **Called by:** `deep_production_path_tests::table_title_spacers_stamp_only_new_marks_and_keep_existing_empty_properties` (comparer/finalize.rs:21719); `deep_production_path_tests::title_spacers_require_unique_revised_two_blank_pretable_context` (comparer/finalize.rs:21817); `deep_production_path_tests::restored_title_spacers_preserve_each_revised_blank_template` (comparer/finalize.rs:21909); `deep_production_path_tests::title_spacer_context_declines_owned_nonlayout_payload` (comparer/finalize.rs:21953); `deep_production_path_tests::title_spacer_injection_requires_a_paired_table_and_a_truly_blank_separator` (comparer/finalize.rs:21999); `finalizer_remaining_source_boundary_tests::two_authored_layout_blanks_are_restored_only_at_the_shared_table_boundary` (comparer/finalize.rs:31283); `public_body_heuristic_story_scope_tests::configured_heuristics` (comparer/finalize.rs:32397); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `strip_empty_pure_ins_before_trailing_pure_dels`  (L4103-4197, pub)
`pub fn strip_empty_pure_ins_before_trailing_pure_dels(dom: &mut Dom, root: NodeId)`

- **Does:** M85a — drop empty pure-ins sitting immediately before a trailing pure-del run.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L4104; `Dom::elements` (xmllinq/mod.rs:640) L4107; `Dom::name_is` (xmllinq/mod.rs:535) L4110,4119,4167,4173,4177; `para_is_pure_deleted` (comparer/finalize.rs:2900) L4119,4173; `para_is_empty_pure_ins` (comparer/finalize.rs:3583) L4131,4193; `story_final_kept_blank` (comparer/finalize.rs:3542) L4139; `para_word_atom_count` (comparer/finalize.rs:8013) L4156; `para_body_alnum_len` (comparer/finalize.rs:7889) L4157; `para_looks_like_demo_title` (comparer/finalize.rs:7906) L4160; `para_has_heading_or_title_style` (comparer/finalize.rs:7923) L4161; `Dom::remove` (xmllinq/mod.rs:1118) L4194
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.contains`, `.count`, `.filter`, `.flatten`, `.get`, `.into_iter`, `.is_some_and`, `.iter`, `.len`, `.saturating_sub`, `.take_while`, `W::body`, `W::p`, `W::sect_pr`, `W::tbl`
- **Called by:** `coverage_round_next_tests::title_spacers_are_kept_once_and_prose_spacers_are_removed` (comparer/finalize.rs:19837); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `para_body_text_is_whitespace_only`  (L4200-4213)
`fn para_body_text_is_whitespace_only(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when every `w:t` / `w:delText` under `p` is empty or whitespace.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L4203; `Dom::value` (xmllinq/mod.rs:1185) L4205
- **Calls (std/external/unresolved):** `.is_empty`, `.trim`, `Some`, `W::del_text`, `W::t`
- **Called by:** `para_is_visually_blank` (comparer/finalize.rs:3528); `relocate_title_page_last_empty_after_pure_dels` (comparer/finalize.rs:3653); `ensure_empty_pure_i_before_short_title_del` (comparer/finalize.rs:3951); `fold_whitespace_pure_ins_into_following_pure_del` (comparer/finalize.rs:4252); `fold_short_label_ins_into_del_with_settings` (comparer/finalize.rs:7058); `merge_replaced_in_container` (comparer/finalize.rs:8626); `coverage_tests::residual_label_zip_adopts_deleted_list_layout_and_moves_empty_spacer` (comparer/finalize.rs:16438)

#### `para_is_pure_inserted`  (L4216-4227)
`fn para_is_pure_inserted(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when `p` is pure-inserted (has ins, no del content/mark).
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L4217; `Dom::descendants` (xmllinq/mod.rs:668) L4221,4226; `para_mark_revision` (comparer/finalize.rs:7631) L4221,4226
- **Calls (std/external/unresolved):** `.is_empty`, `Some`, `W::del`, `W::ins`, `W::p`
- **Called by:** `fold_short_list_label_into_empty_pure_del` (comparer/finalize.rs:1744); `strip_trailing_bare_empty_after_pure_i_dominant` (comparer/finalize.rs:1829); `promote_live_numpr_on_pure_d_from_pprchange` (comparer/finalize.rs:1908); `ensure_pure_i_list_snug_spacing` (comparer/finalize.rs:2113); `strip_redundant_demo_default_spacing` (comparer/finalize.rs:2308); `strip_redundant_normal_pstyle_and_bidi` (comparer/finalize.rs:2404); `strip_list_layout_from_mid_pure_del` (comparer/finalize.rs:2463); `relocate_title_page_last_empty_after_pure_dels` (comparer/finalize.rs:3653); `ensure_empty_pure_i_before_short_title_del` (comparer/finalize.rs:3951); `fold_whitespace_pure_ins_into_following_pure_del` (comparer/finalize.rs:4252); `residual_short_label_zip_with_settings` (comparer/finalize.rs:6930); `fold_short_label_ins_into_del_with_settings` (comparer/finalize.rs:7058); +11 more

#### `fold_whitespace_pure_ins_into_following_pure_del`  (L4252-4594, pub)
`pub fn fold_whitespace_pure_ins_into_following_pure_del(dom: &mut Dom, root: NodeId)`

- **Does:** M86 — fold whitespace-only pure-ins into the following pure-del (file_88).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L4253,4450,4453,4488,4489,4498...; `Dom::elements` (xmllinq/mod.rs:640) L4261,4293,4330,4503,4509,4536...; `Dom::name_is` (xmllinq/mod.rs:535) L4264,4269,4277,4296,4300,4310...; `para_is_pure_inserted` (comparer/finalize.rs:4216) L4270,4278,4311,4321,4348,4474; `para_body_text_is_whitespace_only` (comparer/finalize.rs:4200) L4271,4279,4302,4351,4391,4475; `para_is_pure_deleted` (comparer/finalize.rs:2900) L4301,4348,4413,4459; `para_is_visually_blank` (comparer/finalize.rs:3528) L4312; `story_final_kept_blank` (comparer/finalize.rs:3542) L4322; `Dom::descendants` (xmllinq/mod.rs:668) L4358,4359,4360,4361,4373,4381...; `para_word_atom_count` (comparer/finalize.rs:8013) L4405,4442; `para_body_alnum_len` (comparer/finalize.rs:7889) L4406; `para_looks_like_demo_title` (comparer/finalize.rs:7906) L4407; `para_has_heading_or_title_style` (comparer/finalize.rs:7923) L4408; `ppr_has_structural_props` (comparer/finalize.rs:574) L4452,4455; `para_is_field_residue` (comparer/finalize.rs:7669) L4487,4556; `Dom::attribute` (xmllinq/mod.rs:874) L4490,4558; `Dom::remove` (xmllinq/mod.rs:1118) L4499,4514,4525,4537,4586; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L4502,4533,4544,4566,4570; `Dom::add_before_self` (xmllinq/mod.rs:1130) L4504,4574; `Dom::add` (xmllinq/mod.rs:1074) L4506,4511,4534,4545,4567,4576...; `Dom::new_element` (xmllinq/mod.rs:478) L4565
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.contains`, `.copied`, `.count`, `.filter`, `.first`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.position`, `.rev`, `.saturating_sub`, `.skip`, `.starts_with`, `.to_ascii_lowercase`, `.unwrap_or`, `Some`, `W::body`, `W::del`, `W::drawing`, `W::ins`, `W::name`, `W::p_pr`, `W::p_style`, `W::p`, `W::pict`, `W::r_pr`, `W::sect_pr`, `W::t`, `W::val`
- **Called by:** `coverage_round_two_tests::whitespace_fold_adopts_list_layout_and_replaces_inserted_mark` (comparer/finalize.rs:17531); `coverage_round_two_tests::whitespace_field_residue_never_inherits_heading_style` (comparer/finalize.rs:17576); `coverage_round_three_tests::runless_empty_carrier_preserves_cover_spacers_and_long_prose` (comparer/finalize.rs:18816); `coverage_round_three_tests::trailing_whitespace_spacing_folds_only_after_contentful_insertions` (comparer/finalize.rs:18852); `coverage_round_three_tests::whitespace_carriers_without_mark_shell_adopt_deleted_pilcrow` (comparer/finalize.rs:18890); `deep_production_path_tests::whitespace_mark_shell_variants_keep_existing_format_without_inventing_a_mark` (comparer/finalize.rs:21634); `deep_production_path_tests::whitespace_spacing_guard_searches_past_noninserted_predecessors` (comparer/finalize.rs:21682); `deep_production_path_tests::complete_field_carriers_keep_title_deletions_as_mark_only_properties` (comparer/finalize.rs:23502); `finalizer_distinct_boundary_contract_tests::whitespace_layout_before_a_deleted_block_is_not_empty_content` (comparer/finalize.rs:30076); `whitespace_source_mark_carrier_tests::whitespace_fold_preserves_selected_source_mark_shells_and_every_body_owner` (comparer/finalize.rs:33565); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `rotate_ins_mark_del_only_paragraph`  (L4617-4743, pub)
`pub fn rotate_ins_mark_del_only_paragraph(dom: &mut Dom, root: NodeId)`

- **Does:** M471 — first slice of the mesh off-by-one rotation (diff_after19 × diff_after2 oracle).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L4618,4697; `Dom::elements` (xmllinq/mod.rs:640) L4622,4633,4662,4674,4708,4722; `Dom::name_is` (xmllinq/mod.rs:535) L4625,4636,4642,4648,4665,4667...; `para_mark_revision` (comparer/finalize.rs:7631) L4630,4659,4671; `Dom::remove` (xmllinq/mod.rs:1118) L4701,4702,4707,4721,4733; `Dom::add_first` (xmllinq/mod.rs:1101) L4703,4704; `Dom::add_before_self` (xmllinq/mod.rs:1130) L4714,4728; `Dom::add` (xmllinq/mod.rs:1074) L4715,4729,4734
- **Calls (std/external/unresolved):** `.any`, `.array_windows`, `.collect`, `.copied`, `.filter`, `.find`, `.into_iter`, `.is_empty`, `.iter`, `.len`, `.push`, `.rev`, `Vec::new`, `W::body`, `W::del`, `W::ins`, `W::p_pr`, `W::p`
- **Called by:** `deep_production_path_tests::three_paragraph_rotation_trades_full_properties_and_preserves_each_revision` (comparer/finalize.rs:22039); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `restamp_stranded_del_mark_onto_del_only_paragraph`  (L4775-4854, pub)
`pub fn restamp_stranded_del_mark_onto_del_only_paragraph(dom: &mut Dom, root: NodeId)`

- **Does:** M473 — sibling of the M471 rotation: the deletion mark lands one slot late.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L4776,4792,4825,4840; `Dom::elements` (xmllinq/mod.rs:640) L4780,4793,4797,4817,4828,4832; `Dom::name_is` (xmllinq/mod.rs:535) L4783,4800,4802,4808,4820,4829...; `para_mark_revision` (comparer/finalize.rs:7631) L4789,4814; `Dom::descendants` (xmllinq/mod.rs:668) L4806; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L4839; `Dom::add_before_self` (xmllinq/mod.rs:1130) L4842; `Dom::remove` (xmllinq/mod.rs:1118) L4843; `Dom::add_first` (xmllinq/mod.rs:1101) L4845
- **Calls (std/external/unresolved):** `.any`, `.array_windows`, `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.iter`, `.len`, `W::body`, `W::del_text`, `W::del`, `W::ins`, `W::p_pr`, `W::p`, `W::r_pr`
- **Called by:** `coverage_round_two_tests::stranded_mark_is_cloned_without_removing_empty_shell` (comparer/finalize.rs:17261); `finalizer_distinct_boundary_contract_tests::a_stranded_mark_can_be_copied_without_deleting_its_layout_shell` (comparer/finalize.rs:30511); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `split_head_short_title_long_del_mix`  (L4870-4975, pub)
`pub fn split_head_short_title_long_del_mix(dom: &mut Dom, root: NodeId)`

- **Does:** M469 — split a head title MIX: B's SHORT inserted title (≤4 tokens) paired with A's LONG deleted opening (≥10 tokens, no shared significant token) is Word's [pure-INS title] + [style-less MARK-DEL paragraph] (ooxml_rfonts_rstyle × sd_2672_rtl_table oracle: bare pPr, no pStyle — the deleted text renders at body size, n...
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L4871; `Dom::elements` (xmllinq/mod.rs:640) L4874,4883; `Dom::name_is` (xmllinq/mod.rs:535) L4877,4886,4897,4902; `para_mark_revision` (comparer/finalize.rs:7631) L4889,4890; `Dom::descendants` (xmllinq/mod.rs:668) L4915,4947; `Dom::value_str` (xmllinq/mod.rs:1195) L4916; `text_of` (changes.rs:1518) (by name) L4921,4922; `shared_word_alignment_tests::toks` (comparer/lcs_table.rs:1883) L4929,4930; `Dom::attribute` (xmllinq/mod.rs:874) L4945,4946,4951; `Dom::new_element` (xmllinq/mod.rs:478) L4956,4957,4958,4959; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L4960,4962,4965; `Dom::add` (xmllinq/mod.rs:1074) L4967,4968,4969,4972; `Dom::remove` (xmllinq/mod.rs:1118) L4971; `Dom::add_after_self` (xmllinq/mod.rs:1147) L4974
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_str`, `.collect`, `.contains`, `.filter_map`, `.filter`, `.find`, `.into_iter`, `.is_alphanumeric`, `.is_empty`, `.iter`, `.len`, `.map`, `.max`, `.ok`, `.parse`, `.push_str`, `.push`, `.split`, `.to_string`, `.unwrap_or`, `Some`, `String::new`, `Vec::new`, `W::author`, `W::body`, `W::date`, `W::del_text`, `W::del`, `W::id`, `W::ins`, `W::p_pr`, `W::p`, `W::r_pr`, `W::t`
- **Called by:** `coverage_round_two_tests::short_head_split_preserves_title_style_and_deleted_wrapper_identity` (comparer/finalize.rs:17198); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `fold_leading_ins_from_mix_into_preceding_pure_del`  (L4991-5107, pub)
`pub fn fold_leading_ins_from_mix_into_preceding_pure_del(dom: &mut Dom, root: NodeId)`

- **Does:** M105 — fold the leading insertion of a MIX paragraph back onto a preceding pure-D, leaving the MIX as pure-D + the trailing tokens of the MIX.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L4992,5092,5094,5095; `Dom::elements` (xmllinq/mod.rs:640) L4996,5028,5068; `Dom::name_is` (xmllinq/mod.rs:535) L4999,5005,5031,5034,5071; `para_is_pure_deleted` (comparer/finalize.rs:2900) L5008; `Dom::descendants` (xmllinq/mod.rs:668) L5012,5013,5046; `para_mark_revision` (comparer/finalize.rs:7631) L5014; `para_revision_body_text` (comparer/finalize.rs:7859) L5019; `Dom::value_str` (xmllinq/mod.rs:1195) L5047; `Dom::parent` (xmllinq/mod.rs:597) L5073,5079; `Dom::remove` (xmllinq/mod.rs:1118) L5076; `Dom::add_before_self` (xmllinq/mod.rs:1130) L5080; `Dom::add` (xmllinq/mod.rs:1074) L5082,5085
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.count`, `.filter`, `.find`, `.intersection`, `.into_iter`, `.is_alphanumeric`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.len`, `.map`, `.push_str`, `.push`, `.saturating_sub`, `.split`, `.to_ascii_lowercase`, `.trim`, `Some`, `String::new`, `Vec::new`, `W::body`, `W::del`, `W::ins`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`, `W::t`
- **Called by:** `deep_production_path_tests::leading_subtitle_fold_moves_multiple_wrappers_and_retains_deleted_layout` (comparer/finalize.rs:22149); `deep_production_path_tests::leading_subtitle_gate_requires_disjoint_significant_tokens_and_eight_token_cap` (comparer/finalize.rs:22188); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `strip_last_pure_del_mark_only_ppr`  (L5122-5155, pub)
`pub fn strip_last_pure_del_mark_only_ppr(dom: &mut Dom, root: NodeId)`

- **Does:** M85b — last pure-del with content: Word often omits mark-only `pPr/rPr/del` (bare `<w:del>…</w:del>` only).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L5123,5137; `Dom::elements` (xmllinq/mod.rs:640) L5126,5142,5150; `Dom::name_is` (xmllinq/mod.rs:535) L5127,5130,5147,5151; `para_is_pure_deleted` (comparer/finalize.rs:2900) L5130; `para_has_no_text` (comparer/finalize.rs:3500) L5134; `Dom::remove` (xmllinq/mod.rs:1118) L5154
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.len`, `.rev`, `W::body`, `W::del`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `para_is_mixed_revision`  (L5158-5177)
`fn para_is_mixed_revision(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when paragraph body has both ins and del content (mixed residual).
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L5159; `Dom::descendants` (xmllinq/mod.rs:668) L5167,5169; `para_mark_revision` (comparer/finalize.rs:7631) L5167,5169
- **Calls (std/external/unresolved):** `.and_then`, `.as_mut`, `.as_ref`, `.borrow_mut`, `.borrow`, `.copied`, `.get`, `.insert`, `.is_empty`, `.with`, `Some`, `W::del`, `W::ins`, `W::p`
- **Called by:** `last_pure_del_spacing_to_pprchange` (comparer/finalize.rs:5205); `mixed_spacing_to_following_empty` (comparer/finalize.rs:5308); `cleanup_spacing_and_default_jc` (comparer/finalize.rs:5422); `park_mixed_spacing_onto_trailing_pure_del` (comparer/finalize.rs:5607); `park_mixed_numpr_onto_trailing_empty_pure_del` (comparer/finalize.rs:5747); `strip_empty_pprchange_on_mix_with_live_jc` (comparer/finalize.rs:5937); `promote_heading_spacing_from_pprchange_on_last_mix` (comparer/finalize.rs:6431); `promote_live_jc_from_pprchange_on_body_mix` (comparer/finalize.rs:6516); `strip_last_pure_del_mark_when_pprchange` (comparer/finalize.rs:6598); `park_jc_on_first_short_title_mix_from_body` (comparer/finalize.rs:13148); `ensure_empty_pprchange_on_live_heading_spacing` (comparer/finalize.rs:13237); `ensure_empty_pprchange_on_eq_with_live_jc` (comparer/finalize.rs:13425); +15 more

#### `last_pure_del_spacing_to_pprchange`  (L5205-5294, pub)
`pub fn last_pure_del_spacing_to_pprchange( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** M83b / M87 / M91 / M93 / M94 — last pure-deleted **or mixed** body paragraph: Word moves live layout props into `w:pPrChange` and drops the mark-only del shell.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L5211,5253,5256,5274; `Dom::elements` (xmllinq/mod.rs:640) L5214; `Dom::name_is` (xmllinq/mod.rs:535) L5215,5218; `para_is_mixed_revision` (comparer/finalize.rs:5158) L5225; `para_is_pure_deleted` (comparer/finalize.rs:2900) L5226; `para_keeps_unchanged_words` (comparer/finalize.rs:2076) L5241; `para_side_word_count` (comparer/finalize.rs:2055) L5244,5245; `project_para_properties_for_change` (comparer/formatchg.rs:329) L5283; `Dom::remove` (xmllinq/mod.rs:1118) L5285; `Dom::new_element` (xmllinq/mod.rs:478) L5287; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L5288,5290,5291; `Dom::add` (xmllinq/mod.rs:1074) L5292,5293
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.is_empty`, `.is_some`, `.iter`, `.push`, `.rev`, `.to_string`, `Some`, `Vec::new`, `W::author`, `W::body`, `W::date`, `W::id`, `W::jc_el`, `W::name`, `W::num_pr`, `W::p_pr_change`, `W::p_pr`, `W::p_style`, `W::p`, `W::sect_pr`, `W::spacing_el`
- **Macros:** `vec!`
- **Called by:** `parked_paragraph_history_regressions::parked_layout_restores_properties_that_remain_live_when_rejected` (comparer/finalize.rs:23617); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `mixed_spacing_to_following_empty`  (L5308-5406, pub)
`pub fn mixed_spacing_to_following_empty( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** M98b (file_167): mixed I+D residual followed by empty trailing para — Word keeps del mark on the mixed (no live spacing) and parks the layout spacing on the empty para as live spacing + `pPrChange` (empty old).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L5314,5348,5351,5355,5367,5373...; `Dom::elements` (xmllinq/mod.rs:640) L5317,5359,5401; `Dom::name_is` (xmllinq/mod.rs:535) L5320,5328,5342; `para_is_mixed_revision` (comparer/finalize.rs:5158) L5331; `para_has_no_text` (comparer/finalize.rs:3500) L5334; `Dom::descendants` (xmllinq/mod.rs:668) L5343,5344; `Dom::new_element` (xmllinq/mod.rs:478) L5358,5374,5375,5387,5393; `Dom::add_before_self` (xmllinq/mod.rs:1130) L5360; `Dom::add` (xmllinq/mod.rs:1074) L5362,5380,5381,5388,5398; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L5368; `Dom::add_first` (xmllinq/mod.rs:1101) L5369; `Dom::remove` (xmllinq/mod.rs:1118) L5371,5402; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L5376,5378,5379,5394,5396,5397
- **Calls (std/external/unresolved):** `.collect`, `.copied`, `.filter`, `.first`, `.into_iter`, `.is_empty`, `.is_none`, `.len`, `.to_string`, `Some`, `W::author`, `W::body`, `W::date`, `W::del`, `W::id`, `W::ins`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`, `W::spacing_el`, `W::tbl`
- **Called by:** `coverage_round_two_tests::mixed_spacing_moves_to_blank_and_preserves_existing_history` (comparer/finalize.rs:17302); `coverage_round_two_tests::mixed_spacing_does_not_cross_live_blocks_or_fill_textual_paragraphs` (comparer/finalize.rs:17362); `coverage_real_revision_merge_matrix_tests::word_m98b_spacing_and_history_match_the_complete_format_golden` (comparer/finalize.rs:26622); `existing_mark_layout_ownership_tests::moving_mixed_spacing_keeps_existing_inserted_and_deleted_mark_owners` (comparer/finalize.rs:33035); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `cleanup_spacing_and_default_jc`  (L5422-5590, pub)
`pub fn cleanup_spacing_and_default_jc(dom: &mut Dom, root: NodeId)`

- **Does:** M228 / M226 / M231 single body walk (perf: avoid 3× O(n_p) on large docs).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L5423,5441,5446,5453,5462,5491...; `Dom::elements` (xmllinq/mod.rs:640) L5426,5455,5471,5494,5525,5559; `Dom::name_is` (xmllinq/mod.rs:535) L5429,5438,5458,5497; `Dom::descendants` (xmllinq/mod.rs:668) L5436; `Dom::attribute` (xmllinq/mod.rs:874) L5447,5463,5514,5515,5520,5521...; `Dom::remove` (xmllinq/mod.rs:1118) L5449,5465,5482,5537,5580,5588; `Dom::name` (xmllinq/mod.rs:526) L5472,5526,5560; `XName::local_name` (xmllinq/mod.rs:208) (by name) L5475,5529,5563; `para_is_mixed_revision` (comparer/finalize.rs:5158) L5507; `para_is_pure_deleted` (comparer/finalize.rs:2900) L5555; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L5586; `Dom::add_before_self` (xmllinq/mod.rs:1130) L5587
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.enumerate`, `.filter`, `.find`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.len`, `.unwrap_or`, `Some`, `W::body`, `W::del`, `W::jc_el`, `W::name`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::sect_pr`, `W::spacing_el`, `W::val`
- **Called by:** `coverage_tests::cleanup_spacing_removes_only_redundant_mixed_history` (comparer/finalize.rs:16254); `coverage_tests::cleanup_default_alignment_keeps_real_layout_history` (comparer/finalize.rs:16294); `coverage_tests::cleanup_deleted_spacing_promotes_mid_body_but_preserves_last_history` (comparer/finalize.rs:16328); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `park_mixed_spacing_onto_trailing_pure_del`  (L5607-5733, pub)
`pub fn park_mixed_spacing_onto_trailing_pure_del( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** M221 (green_underline×heading_1_bold ~56): MIX residual carries B Heading spacing (before=400 after=120 line=240) while following pure-D bullets have none.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L5613,5629,5632,5653,5655,5685...; `Dom::elements` (xmllinq/mod.rs:640) L5616,5689,5728; `Dom::name_is` (xmllinq/mod.rs:535) L5619,5626,5637; `para_is_mixed_revision` (comparer/finalize.rs:5158) L5626; `para_is_pure_deleted` (comparer/finalize.rs:2900) L5637; `para_has_no_text` (comparer/finalize.rs:3500) L5650; `para_body_alnum_len` (comparer/finalize.rs:7889) L5663; `body_token_set` (comparer/finalize.rs:7870) L5664; `para_revision_body_text` (comparer/finalize.rs:7859) L5664; `Dom::attribute` (xmllinq/mod.rs:874) L5671,5675; `Dom::new_element` (xmllinq/mod.rs:478) L5688,5701,5702,5715,5721; `Dom::add_before_self` (xmllinq/mod.rs:1130) L5690; `Dom::add` (xmllinq/mod.rs:1074) L5692,5707,5708,5716,5726; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L5697; `Dom::add_first` (xmllinq/mod.rs:1101) L5698; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L5703,5705,5706,5722,5724,5725; `Dom::remove` (xmllinq/mod.rs:1118) L5711,5729
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.contains`, `.copied`, `.filter`, `.first`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some`, `.len`, `.ok`, `.parse`, `.to_string`, `.unwrap_or`, `Some`, `W::author`, `W::body`, `W::date`, `W::del`, `W::id`, `W::ins`, `W::name`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`, `W::spacing_el`
- **Called by:** `coverage_round_two_tests::heading_spacing_parks_on_last_short_deleted_residual` (comparer/finalize.rs:17381); `coverage_round_two_tests::spacing_park_rejects_count_length_and_heading_thresholds` (comparer/finalize.rs:17426); `existing_mark_layout_ownership_tests::parking_heading_spacing_keeps_existing_lifetime_and_history_provenance` (comparer/finalize.rs:33113); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `park_mixed_numpr_onto_trailing_empty_pure_del`  (L5747-5841, pub)
`pub fn park_mixed_numpr_onto_trailing_empty_pure_del( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** M230 (bullet_list_bold×bullet_list ~83): MIX residual carries live `numPr` (B list item "Grapes") while trailing pure-D empties from A have none.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L5753,5769,5772,5792,5794,5804...; `Dom::elements` (xmllinq/mod.rs:640) L5756,5808; `Dom::name_is` (xmllinq/mod.rs:535) L5759,5766,5777; `para_is_mixed_revision` (comparer/finalize.rs:5158) L5766; `para_is_pure_deleted` (comparer/finalize.rs:2900) L5777; `Dom::new_element` (xmllinq/mod.rs:478) L5807,5829,5830; `Dom::add_before_self` (xmllinq/mod.rs:1130) L5809,5822,5824; `Dom::add` (xmllinq/mod.rs:1074) L5811,5835,5836; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L5819; `Dom::add_first` (xmllinq/mod.rs:1101) L5826; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L5831,5833,5834; `Dom::remove` (xmllinq/mod.rs:1118) L5838
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.contains`, `.copied`, `.filter`, `.first`, `.into_iter`, `.is_none`, `.is_some`, `.len`, `.to_string`, `Some`, `W::author`, `W::body`, `W::date`, `W::id`, `W::num_pr`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`
- **Called by:** `coverage_round_two_tests::mixed_numbering_moves_before_last_deletion_mark_or_history` (comparer/finalize.rs:17463); `coverage_round_two_tests::numbering_park_preserves_short_long_and_already_numbered_runs` (comparer/finalize.rs:17514); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `last_pure_del_inherit_prev_jc`  (L5847-5922, pub)
`pub fn last_pure_del_inherit_prev_jc(dom: &mut Dom, root: NodeId)`

- **Does:** M102c (file_148): last pure-del with `pPrChange(spacing)` and no live `jc` inherits live `jc` from a preceding body para that has center align (Word pure-D of A line-spacing sentence keeps B's center mark even when the immediate prev mixed residual only has spacing).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L5848,5863,5866,5871,5878,5886...; `Dom::elements` (xmllinq/mod.rs:640) L5851,5874,5900; `Dom::name_is` (xmllinq/mod.rs:535) L5854,5860,5876,5893,5901,5904; `para_is_pure_deleted` (comparer/finalize.rs:2900) L5860; `Dom::descendants` (xmllinq/mod.rs:668) L5881; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L5916; `Dom::add_before_self` (xmllinq/mod.rs:1130) L5918; `Dom::add_first` (xmllinq/mod.rs:1101) L5920
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.find`, `.into_iter`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.or_else`, `.rev`, `Some`, `W::body`, `W::jc_el`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::sect_pr`, `W::spacing_el`
- **Called by:** `existing_mark_layout_ownership_tests::alignment_donor_search_crosses_default_property_paragraphs_without_claiming_their_formats` (comparer/finalize.rs:33198); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `strip_empty_pprchange_on_mix_with_live_jc`  (L5937-5997, pub)
`pub fn strip_empty_pprchange_on_mix_with_live_jc(dom: &mut Dom, root: NodeId)`

- **Does:** M451 (center_alignment_2 × center_alignment ~81.5 / docxodus 100): mid MIX free-mesh carries **empty** `pPrChange` alongside live `jc` where Word uses live `jc` + rPr/ins mark only.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L5938,5960,5961,5962,5967,5971...; `Dom::elements` (xmllinq/mod.rs:640) L5941,5982; `Dom::name_is` (xmllinq/mod.rs:535) L5944,5951,5958; `para_is_mixed_revision` (comparer/finalize.rs:5158) L5951; `Dom::name` (xmllinq/mod.rs:526) L5983; `Dom::remove` (xmllinq/mod.rs:1118) L5995
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.enumerate`, `.filter`, `.find`, `.into_iter`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `W::body`, `W::jc_el`, `W::name`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `strip_propertyless_ppr`  (L6008-6047, pub)
`pub fn strip_propertyless_ppr(dom: &mut Dom, root: NodeId)`

- **Does:** Drop a `w:pPr` that records nothing Word keeps.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L6009,6014,6023; `Dom::descendants` (xmllinq/mod.rs:668) L6012; `Dom::elements` (xmllinq/mod.rs:640) L6017,6024,6043; `Dom::name_is` (xmllinq/mod.rs:535) L6022,6033; `Dom::name` (xmllinq/mod.rs:526) L6026; `Dom::remove` (xmllinq/mod.rs:1118) L6041,6044
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.is_some_and`, `.iter`, `.push`, `Some`, `Vec::new`, `W::body`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215); `compare_documents_impl` (document_comparer.rs:6521)

#### `pg_num_type_is_implicit_decimal`  (L6053-6061, pub(crate))
`pub(crate) fn pg_num_type_is_implicit_decimal(dom: &Dom, n: NodeId) -> bool`

- **Does:** `w:pgNumType` that only restates Word's implicit `decimal` format.
- **Calls (crate):** `Dom::attributes_iter` (xmllinq/mod.rs:886) L6054; `XName::local_name` (xmllinq/mod.rs:208) (by name) L6055
- **Called by:** `strip_unrecorded_word_defaults` (comparer/finalize.rs:6082); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `strip_unrecorded_word_defaults`  (L6082-6152, pub)
`pub fn strip_unrecorded_word_defaults(dom: &mut Dom, root: NodeId, original_line: Option<&str>)`

- **Does:** Markup Word's redline leaves out once the text already matches.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L6084,6114,6124,6127,6128,6129; `Dom::elements` (xmllinq/mod.rs:640) L6087; `Dom::name_is` (xmllinq/mod.rs:535) L6091; `Dom::descendants` (xmllinq/mod.rs:668) L6093,6098,6103,6108,6113,6121; `Dom::attribute` (xmllinq/mod.rs:874) L6094,6115; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L6099,6104; `pg_num_type_is_implicit_decimal` (comparer/finalize.rs:6053) L6109; `spacing_is_only_line_276` (comparer/finalize.rs:6154) L6131; `content_revision_marks` (comparer/finalize.rs:6189) L6132,6141; `pilcrow_ins_only` (comparer/finalize.rs:6174) L6140; `Dom::remove` (xmllinq/mod.rs:1118) L6147,6150
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.is_none_or`, `.is_some`, `.push`, `.rev`, `.unwrap_or`, `Some`, `Vec::new`, `W::body`, `W::del`, `W::jc_el`, `W::name`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`, `W::spacing_el`, `W::val`
- **Macros:** `matches!`
- **Called by:** `coverage_round_next_tests::unrecorded_spacing_guards_preserve_all_explicit_vertical_metrics` (comparer/finalize.rs:20299); `split_paragraph_boundary_regressions::mixed_inserted_boundary_joins_original_text_and_closing_layout_on_reject` (comparer/finalize.rs:23752); `public_body_heuristic_story_scope_tests::configured_heuristics` (comparer/finalize.rs:32397); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215); `compare_documents_impl` (document_comparer.rs:6521)

#### `spacing_is_only_line_276`  (L6154-6171)
`fn spacing_is_only_line_276(dom: &Dom, sp: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L6155,6166
- **Calls (std/external/unresolved):** `.is_some`, `Some`, `W::name`
- **Called by:** `strip_unrecorded_word_defaults` (comparer/finalize.rs:6082)

#### `pilcrow_ins_only`  (L6174-6185)
`fn pilcrow_ins_only(dom: &Dom, ppr: NodeId) -> bool`

- **Does:** `w:pPr` whose only child is `w:rPr/w:ins`.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L6175,6183; `Dom::name_is` (xmllinq/mod.rs:535) L6180,6184
- **Calls (std/external/unresolved):** `.len`, `W::ins`, `W::r_pr`
- **Called by:** `strip_unrecorded_word_defaults` (comparer/finalize.rs:6082)

#### `content_revision_marks`  (L6189-6205)
`fn content_revision_marks(dom: &Dom, p: NodeId, ppr: NodeId) -> (bool, bool)`

- **Does:** Whether the paragraph's content, outside its `w:pPr`, holds `w:ins` and `w:del`.
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L6191; `Dom::descendants` (xmllinq/mod.rs:668) L6194; `Dom::name_is` (xmllinq/mod.rs:535) L6198,6200
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.into_iter`, `W::del`, `W::ins`
- **Called by:** `merged_closing_carrier_takes_source_properties` (comparer/finalize.rs:3122); `strip_unrecorded_word_defaults` (comparer/finalize.rs:6082)

#### `align_word_table_and_comment_chrome`  (L6220-6327, pub)
`pub fn align_word_table_and_comment_chrome(dom: &mut Dom, root: NodeId)`

- **Does:** Table and comment-reference chrome Word writes on every matching redline in this corpus.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L6221,6235,6254,6257,6264,6273...; `Dom::descendants` (xmllinq/mod.rs:668) L6224,6252,6294; `Dom::ancestors` (xmllinq/mod.rs:770) L6229; `Dom::new_element` (xmllinq/mod.rs:478) L6238,6276,6277,6279,6306,6312...; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L6239,6248,6280,6281,6313,6318...; `insert_tblpr_child_in_order` (comparer/finalize.rs:10454) L6250; `Dom::attribute` (xmllinq/mod.rs:874) L6265; `Dom::elements` (xmllinq/mod.rs:640) L6271,6287; `Dom::add` (xmllinq/mod.rs:1074) L6282,6284,6290,6314,6319,6324; `Dom::add_before_self` (xmllinq/mod.rs:1130) L6286,6288; `Dom::ancestors_and_self` (xmllinq/mod.rs:787) L6296; `Dom::name_is` (xmllinq/mod.rs:535) L6299; `Dom::add_first` (xmllinq/mod.rs:1101) L6307
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.copied`, `.find`, `.first`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `Some`, `W::body`, `W::name`, `W::r_pr`, `W::r`, `W::tbl_pr`, `W::tbl`, `W::tr_pr`, `W::tr`, `W::val`
- **Called by:** `coverage_round_next_tests::table_chrome_adds_look_before_history_and_handles_all_row_positions` (comparer/finalize.rs:19966); `coverage_round_next_tests::comment_chrome_fills_each_missing_property_independently_and_ignores_orphans` (comparer/finalize.rs:19988); `coverage_round_next_tests::compatibility_defaults_keep_nonzero_vertical_margins_and_existing_look` (comparer/finalize.rs:20582); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215); `compare_documents_impl` (document_comparer.rs:6521)

#### `rewrite_inserted_external_hyperlinks`  (L6336-6396, pub)
`pub fn rewrite_inserted_external_hyperlinks( dom: &mut Dom, root: NodeId, id_to_target: &HashMap<String, String>, base_targets: &HashSet<String>, )`

- **Does:** A wholly inserted external hyperlink whose URL is not in the base document becomes Word's `HYPERLINK` field.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L6342; `Dom::attribute` (xmllinq/mod.rs:874) L6344,6347; `hyperlink_is_wholly_inserted` (comparer/finalize.rs:6398) L6356; `field_char_run` (comparer/finalize.rs:6414) L6360,6365,6366; `Dom::new_element` (xmllinq/mod.rs:478) L6361,6362; `Dom::add_text` (xmllinq/mod.rs:1087) L6363; `Dom::add` (xmllinq/mod.rs:1074) L6364,6380; `Dom::parent` (xmllinq/mod.rs:597) L6367; `Dom::name_is` (xmllinq/mod.rs:535) L6367; `Dom::add_before_self` (xmllinq/mod.rs:1130) L6369,6370,6371,6392; `Dom::add_after_self` (xmllinq/mod.rs:1147) L6372; `Dom::elements` (xmllinq/mod.rs:640) L6375; `Dom::add_first` (xmllinq/mod.rs:1101) L6377,6378,6379; `Dom::nodes` (xmllinq/mod.rs:604) L6389; `Dom::remove` (xmllinq/mod.rs:1118) L6391,6394
- **Calls (std/external/unresolved):** `.cloned`, `.contains`, `.first`, `.get`, `.is_some_and`, `.is_some`, `.last`, `.map`, `R::name`, `Some`, `W::hyperlink`, `W::ins`, `W::name`, `W::r`
- **Macros:** `format!`
- **Called by:** `coverage_final_batch_finalize_tests::external_link_rewrite_matrix_requires_new_target_and_wholly_inserted_content` (comparer/finalize.rs:24402); `compare_documents_impl` (document_comparer.rs:6521)

#### `hyperlink_is_wholly_inserted`  (L6398-6412)
`fn hyperlink_is_wholly_inserted(dom: &Dom, hl: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L6399,6406,6407; `Dom::value_str` (xmllinq/mod.rs:1195) L6400; `Dom::ancestors_and_self` (xmllinq/mod.rs:787) L6401; `Dom::name_is` (xmllinq/mod.rs:535) L6404,6408; `Dom::parent` (xmllinq/mod.rs:597) L6408
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.is_some_and`, `.iter`, `.trim`, `Some`, `W::del_text`, `W::ins`, `W::t`
- **Called by:** `rewrite_inserted_external_hyperlinks` (comparer/finalize.rs:6336)

#### `field_char_run`  (L6414-6420)
`fn field_char_run(dom: &mut Dom, ty: &str) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L6415,6416; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L6417; `Dom::add` (xmllinq/mod.rs:1074) L6418
- **Calls (std/external/unresolved):** `Some`, `W::name`, `W::r`
- **Called by:** `rewrite_inserted_external_hyperlinks` (comparer/finalize.rs:6336)

#### `promote_heading_spacing_from_pprchange_on_last_mix`  (L6431-6501, pub)
`pub fn promote_heading_spacing_from_pprchange_on_last_mix(dom: &mut Dom, root: NodeId)`

- **Does:** M450 (calibri_font × calibri_heading_2_right ~82.5 / docxodus 100): last MIX free-mesh parks **Heading residual** spacing (`before≥200` + `line=240`) into `pPrChange` only; Word keeps it **live** with an empty `pPrChange`.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L6432,6446,6450,6453,6456,6459...; `Dom::elements` (xmllinq/mod.rs:640) L6435; `Dom::name_is` (xmllinq/mod.rs:535) L6438,6443,6475; `para_is_mixed_revision` (comparer/finalize.rs:5158) L6443; `Dom::attribute` (xmllinq/mod.rs:874) L6463,6467,6484,6488; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L6497; `Dom::add_before_self` (xmllinq/mod.rs:1130) L6498; `Dom::remove` (xmllinq/mod.rs:1118) L6500
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.filter`, `.into_iter`, `.is_some`, `.last`, `.len`, `.ok`, `.parse`, `.unwrap_or`, `W::body`, `W::name`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::sect_pr`, `W::spacing_el`
- **Called by:** `coverage_round_two_tests::heading_spacing_promotion_requires_prior_live_heading_and_preserves_old_shell` (comparer/finalize.rs:17983); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `promote_live_jc_from_pprchange_on_body_mix`  (L6516-6586, pub)
`pub fn promote_live_jc_from_pprchange_on_body_mix( dom: &mut Dom, root: NodeId, revised_unaligned: &HashSet<String>, )`

- **Does:** M449 (right_aligned_italic × right_alignment ~75.9 / docxodus 100): mid-body MIX free-mesh parks `jc` into `pPrChange` only; Word keeps **live** `jc` on the body MIX.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L6521,6542,6548,6549,6553,6556...; `Dom::elements` (xmllinq/mod.rs:640) L6524,6572; `Dom::name_is` (xmllinq/mod.rs:535) L6527,6534; `para_is_mixed_revision` (comparer/finalize.rs:5158) L6534; `para_side_word_count` (comparer/finalize.rs:2055) L6537,6538; `Dom::attribute` (xmllinq/mod.rs:874) L6562; `accepted_text` (comparer/finalize.rs:13355) L6563; `Dom::new_element` (xmllinq/mod.rs:478) L6567; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L6568; `Dom::add_before_self` (xmllinq/mod.rs:1130) L6569; `Dom::name` (xmllinq/mod.rs:526) L6573; `Dom::remove` (xmllinq/mod.rs:1118) L6583
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter`, `.into_iter`, `.is_empty`, `.is_some`, `.len`, `.to_string`, `.unwrap_or`, `Some`, `W::body`, `W::jc_el`, `W::name`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`, `W::val`
- **Called by:** `coverage_round_two_tests::old_body_alignment_promotes_only_for_aligned_revised_paragraphs` (comparer/finalize.rs:18016); `public_body_heuristic_story_scope_tests::configured_heuristics` (comparer/finalize.rs:32397); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `strip_last_pure_del_mark_when_pprchange`  (L6598-6634, pub)
`pub fn strip_last_pure_del_mark_when_pprchange(dom: &mut Dom, root: NodeId)`

- **Does:** M87b / M94 — last pure-del **or mixed** that already has `pPrChange`: drop mark-only `rPr/del` (Word keeps pPrChange only — file_55 "b", file_139 mixed).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L6599,6612,6615,6619,6620; `Dom::elements` (xmllinq/mod.rs:640) L6602,6623,6629; `Dom::name_is` (xmllinq/mod.rs:535) L6603,6606; `para_is_pure_deleted` (comparer/finalize.rs:2900) L6609; `para_is_mixed_revision` (comparer/finalize.rs:5158) L6609; `Dom::remove` (xmllinq/mod.rs:1118) L6626,6628,6630
- **Calls (std/external/unresolved):** `.find`, `.is_empty`, `.is_none`, `.iter`, `.len`, `.rev`, `W::body`, `W::del`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`
- **Called by:** `finalizer_distinct_boundary_contract_tests::a_terminal_property_history_replaces_the_mark_but_keeps_its_formats` (comparer/finalize.rs:30550); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `reorder_replacements_ins_before_del`  (L6657-6685, pub)
`pub fn reorder_replacements_ins_before_del(dom: &mut Dom, node: NodeId)`

- **Does:** Word emits the INSERTION (new text) before the DELETION (old text) at a replacement site; our LCS produces delete-then-insert.
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L6661,6680; `revision_wrapper` (comparer/finalize.rs:6712) L6667; `fields_complete` (comparer/finalize.rs:6689) L6670,6671; `Dom::remove` (xmllinq/mod.rs:1118) L6673; `Dom::add_before_self` (xmllinq/mod.rs:1130) L6674; `Dom::is_element` (xmllinq/mod.rs:502) L6681; `reorder_replacements_ins_before_del` (comparer/finalize.rs:6657) L6682
- **Calls (std/external/unresolved):** `.len`, `Some`, `W::name`
- **Macros:** `matches!`
- **Called by:** `reorder_replacements_ins_before_del` (comparer/finalize.rs:6657); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215); `compare_documents_impl` (document_comparer.rs:6521)

#### `fields_complete`  (L6689-6704, pub(crate))
`pub(crate) fn fields_complete(dom: &Dom, node: NodeId, fld_char: &XName) -> bool`

- **Does:** Every field char under `node` belongs to a field that begins and ends under it: no `fldChar` at all, or begins and ends that nest and balance.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L6691; `Dom::attribute` (xmllinq/mod.rs:874) L6692
- **Calls (std/external/unresolved):** `Some`, `W::name`
- **Called by:** `reorder_replacements_ins_before_del` (comparer/finalize.rs:6657); `coverage_tests::field_completeness_requires_balanced_order_not_just_counts` (comparer/finalize.rs:15835); `deep_production_path_tests::complete_field_carriers_keep_title_deletions_as_mark_only_properties` (comparer/finalize.rs:23502); `reorder` (edit/replacement_order.rs:27) (?)

#### `revision_wrapper`  (L6712-6747, pub(crate))
`pub(crate) fn revision_wrapper(dom: &Dom, node: NodeId) -> Option<(bool, String, String)>`

- **Does:** A node that is one revision for the replacement swap: a `w:ins` (`true`) or `w:del` (`false`) wrapper, or a `w:hyperlink` whose element children are all one of them by one author and date (Word's redline of super_editor hyperlink_multiple_runs × diff_before10, corpus a098bd5ad9, writes "One two" inserted before the de...
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L6713,6736; `Dom::name` (xmllinq/mod.rs:526) L6717,6731; `Dom::attribute` (xmllinq/mod.rs:874) L6724,6725; `read_tables::{fn own}` (markdown/mod.rs:938) (by name) L6728,6739; `Dom::nodes` (xmllinq/mod.rs:604) L6735
- **Calls (std/external/unresolved):** `.as_ref`, `.to_string`, `.unwrap_or`, `Some`, `W::author`, `W::date`, `W::del`, `W::ins`, `W::name`
- **Called by:** `reorder_replacements_ins_before_del` (comparer/finalize.rs:6657); `coverage_tests::revision_wrapper_requires_consistent_hyperlink_stamps` (comparer/finalize.rs:15859); `reorder` (edit/replacement_order.rs:27) (?)

#### `coalesce_adjacent_revisions`  (L6764-6798, pub)
`pub fn coalesce_adjacent_revisions(dom: &mut Dom, node: NodeId)`

- **Does:** Merge consecutive sibling `w:ins` (and consecutive `w:del`) wrappers that share the same `w:author` and `w:date` into a single wrapper, moving the later wrapper's child runs into the first and removing the emptied wrapper.
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L6767,6780,6793; `Dom::is_element` (xmllinq/mod.rs:502) L6771,6794; `Dom::name` (xmllinq/mod.rs:526) L6774; `Dom::attribute` (xmllinq/mod.rs:874) L6775,6776,6777,6778; `Dom::remove` (xmllinq/mod.rs:1118) L6781,6784; `Dom::add` (xmllinq/mod.rs:1074) L6782; `coalesce_adjacent_revisions` (comparer/finalize.rs:6764) L6795
- **Calls (std/external/unresolved):** `.map`, `.name`, `.to_string`, `Some`, `W::author`, `W::date`, `W::del`, `W::ins`
- **Macros:** `matches!`
- **Called by:** `coalesce_adjacent_revisions` (comparer/finalize.rs:6764); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `simplify_move_markup_to_del_ins`  (L6814-6816, pub)
`pub fn simplify_move_markup_to_del_ins(dom: &mut Dom, root: NodeId) -> NodeId`

- **Does:** M4.F — `SimplifyMoveMarkupToDelIns` (:3309), gated by settings.simplify_move_markup (default off).
- **Calls (crate):** `simplify_move_transform` (comparer/finalize.rs:6818) L6815
- **Called by:** `finalizer_distinct_boundary_contract_tests::move_simplification_changes_only_lifetime_names_and_range_markers` (comparer/finalize.rs:29959); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `simplify_move_transform`  (L6818-6856)
`fn simplify_move_transform(dom: &mut Dom, node: NodeId) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L6819,6844; `Dom::new_element` (xmllinq/mod.rs:478) L6833; `copy_move_id_attrs` (comparer/finalize.rs:150) L6838; `copy_attrs` (comparer/finalize.rs:136) L6840; `Dom::nodes` (xmllinq/mod.rs:604) L6842; `Dom::is_element` (xmllinq/mod.rs:502) L6843; `simplify_move_transform` (comparer/finalize.rs:6818) L6848; `Dom::add` (xmllinq/mod.rs:1074) L6849,6852; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L6851
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.unwrap`, `W::del`, `W::ins`, `W::move_from_range_end`, `W::move_to_range_end`, `W::name`
- **Called by:** `simplify_move_markup_to_del_ins` (comparer/finalize.rs:6814); `simplify_move_transform` (comparer/finalize.rs:6818)

#### `merge_replaced_paragraphs`  (L6873-6875, pub)
`pub fn merge_replaced_paragraphs(dom: &mut Dom, root: NodeId, comparer_author: &str)`

- **Does:** Word-alignment mode (settings-gated, never in PowerTools-faithful runs): Word's Compare merges a fully-replaced paragraph PAIR into one paragraph — inserted runs first, then deleted runs, under the INSERTED paragraph's properties (evidence: the `_word_redline` fixture corpus, e.g.
- **Calls (crate):** `merge_replaced_paragraphs_in` (comparer/finalize.rs:6895) L6874
- **Called by:** `coverage_round_three_tests::same_after_merge` (comparer/finalize.rs:18272); `coverage_round_three_tests::paired_normalization_preserves_prior_pilcrow_provenance_and_only_unmarks_its_own_pair` (comparer/finalize.rs:18280); `coverage_round_three_tests::related_one_for_one_adopts_exact_structural_properties_and_inserted_attributes` (comparer/finalize.rs:18355); `coverage_round_three_tests::field_replacement_keeps_bare_properties_instead_of_deleted_heading` (comparer/finalize.rs:18405); `coverage_round_three_tests::bare_related_pair_does_not_invent_paragraph_properties` (comparer/finalize.rs:18425); `coverage_round_three_tests::deleted_then_multiple_inserted_folds_only_last_and_adopts_heading` (comparer/finalize.rs:18440); `coverage_round_three_tests::last_inserted_list_adopts_deleted_listparagraph_and_numbering` (comparer/finalize.rs:18478); `coverage_round_three_tests::alignment_only_carrier_adopts_deleted_spacing_exactly` (comparer/finalize.rs:18508); `coverage_round_three_tests::story_tail_fusion_crosses_deleted_table_and_keeps_deleted_layout` (comparer/finalize.rs:18603); `coverage_round_three_tests::multiple_mark_only_deletions_fold_once_and_preserve_remaining_empty` (comparer/finalize.rs:18681); `coverage_round_three_tests::empty_heading_residual_adopts_heading_for_last_list_carrier` (comparer/finalize.rs:18711); `coverage_round_three_tests::carried_foreign_insertions_reorder_without_becoming_replacement_deletions` (comparer/finalize.rs:18731); +1 more

#### `merge_replaced_paragraphs_in`  (L6895-6910, pub)
`pub fn merge_replaced_paragraphs_in( dom: &mut Dom, root: NodeId, comparer_author: &str, story_final_paired: bool, )`

- **Does:** [`merge_replaced_paragraphs`] for a body whose two final paragraph marks are paired (`story_final_paired`): the revised document ended on an empty paragraph, so its last content paragraph closes on its own inserted mark and Word does not fold it into the first deleted paragraph (math_radical_tests × multi_image_types).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L6901; `Dom::descendants` (xmllinq/mod.rs:668) L6904; `merge_replaced_in_container` (comparer/finalize.rs:8626) L6908
- **Calls (std/external/unresolved):** `.collect`, `.extend`, `.into_iter`, `Some`, `W::body`, `W::name`, `W::sdt_content`
- **Called by:** `merge_replaced_paragraphs` (comparer/finalize.rs:6873); `coverage_round_three_tests::paired_body_tail_stays_separate_but_nested_stories_still_fold` (comparer/finalize.rs:18545); `coverage_round_three_tests::shared_title_head_junction_precedes_remaining_inserted_stream` (comparer/finalize.rs:18570); `coverage_round_three_tests::story_tail_fusion_stops_at_page_breaks_section_breaks_and_live_tables` (comparer/finalize.rs:18645); `coverage_round_three_tests::unmatched_field_blocks_shared_title_head_junction` (comparer/finalize.rs:18748); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `residual_short_label_zip_with_settings`  (L6930-7038, pub)
`pub fn residual_short_label_zip_with_settings( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, )`

- **Does:** Fold residual shared labels using the configured formatting-history policy.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L6935; `Dom::elements` (xmllinq/mod.rs:640) L6939; `Dom::name_is` (xmllinq/mod.rs:535) L6942,6947,6952; `para_is_pure_inserted` (comparer/finalize.rs:4216) L6947; `para_is_pure_deleted` (comparer/finalize.rs:2900) L6952; `para_body_is_very_short` (comparer/finalize.rs:7899) L6961,7014; `para_revision_body_text` (comparer/finalize.rs:7859) L6964,6979,7025; `Dom::parent` (xmllinq/mod.rs:597) L6976,6997,7013,7022; `fold_short_label_ins_into_del_with_settings` (comparer/finalize.rs:7058) L7000,7030; `para_body_alnum_len` (comparer/finalize.rs:7889) L7015
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.contains`, `.copied`, `.filter`, `.insert`, `.into_iter`, `.is_alphanumeric`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.last`, `.len`, `.map`, `.split`, `.to_ascii_lowercase`, `Some`, `W::body`, `W::p`, `W::sect_pr`, `std::collections::HashSet::new`
- **Called by:** `residual_short_label_zip` (comparer/finalize.rs:6923); `short_label_format_history_tests::configured_label_tracking_preserves_sources_and_respects_disabled_changes` (comparer/finalize.rs:24170); `public_body_heuristic_story_scope_tests::configured_heuristics` (comparer/finalize.rs:32397); `fragmented_source_label_boundary_tests::public_label_zip_preserves_already_adjacent_spacer_and_missing_layout_ownership` (comparer/finalize.rs:34014); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `fold_short_label_ins_into_del_with_settings`  (L7058-7176)
`fn fold_short_label_ins_into_del_with_settings( dom: &mut Dom, ip: NodeId, dp: NodeId, settings: &WmlComparerSettings, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L7065,7118,7123,7135,7146,7147...; `Dom::elements` (xmllinq/mod.rs:640) L7066,7085,7108,7113,7153,7162; `Dom::name_is` (xmllinq/mod.rs:535) L7069,7111,7116; `para_is_pure_inserted` (comparer/finalize.rs:4216) L7070; `para_has_no_text` (comparer/finalize.rs:3500) L7071; `para_body_text_is_whitespace_only` (comparer/finalize.rs:4200) L7071; `Dom::element` (xmllinq/mod.rs:655) L7080,7081,7093,7094,7099,7100...; `Dom::new_element` (xmllinq/mod.rs:478) L7084; `Dom::add_before_self` (xmllinq/mod.rs:1130) L7086,7124,7161,7168; `Dom::add` (xmllinq/mod.rs:1074) L7088,7097,7103,7126,7129,7170...; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L7096,7102; `Dom::remove` (xmllinq/mod.rs:1118) L7121,7133,7136,7158; `mesh_short_label_shared_eq_with_settings` (comparer/finalize.rs:7204) L7142
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.copied`, `.filter`, `.find`, `.first`, `.get`, `.into_iter`, `.is_none`, `.is_some`, `.iter`, `.nth`, `.position`, `.skip_while`, `Some`, `W::name`, `W::p_pr`, `W::p`, `W::spacing_el`
- **Called by:** `residual_short_label_zip_with_settings` (comparer/finalize.rs:6930); `fold_short_label_ins_into_del` (comparer/finalize.rs:7053)

#### `mesh_short_label_shared_eq_with_settings`  (L7204-7585)
`fn mesh_short_label_shared_eq_with_settings( dom: &mut Dom, p: NodeId, settings: &WmlComparerSettings, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L7212,7225,7231,7347,7352,7428...; `Dom::name_is` (xmllinq/mod.rs:535) L7213,7274,7287,7291,7295,7312...; `Dom::parent` (xmllinq/mod.rs:597) L7214,7218,7459,7507,7509,7516...; `Dom::ancestors` (xmllinq/mod.rs:770) L7215,7370; `Dom::elements` (xmllinq/mod.rs:640) L7224,7230,7270,7290,7294,7299...; `Dom::value_str` (xmllinq/mod.rs:1195) L7226,7232,7378,7397,7432,7549...; `trailing_alnum_token` (comparer/finalize.rs:7588) L7257; `Dom::name` (xmllinq/mod.rs:526) L7281,7301,7563; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L7302; `XName::local_name` (xmllinq/mod.rs:208) (by name) L7304; `Dom::nodes` (xmllinq/mod.rs:604) L7307; `mesh_short_label_shared_eq_with_settings::{fn label_run}` (comparer/finalize.rs:7322) L7356,7357; `Dom::element` (xmllinq/mod.rs:655) L7363,7364,7470; `paragraph_mark_properties_equal` (comparer/finalize.rs:3322) L7366; `max_numeric_id` (comparer/finalize.rs:14923) L7371; `strip_trailing_alnum_token` (comparer/finalize.rs:7606) L7379; `Dom::set_value` (xmllinq/mod.rs:1219) L7380,7417,7447; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L7382,7419,7449; `XNamespace::xml` (xmllinq/mod.rs:128) L7382,7419,7449; `Dom::new_element` (xmllinq/mod.rs:478) L7469,7475; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L7471; `Dom::add` (xmllinq/mod.rs:1074) L7472,7477,7489,7532; `closing_mark_takes_revised_mark_formatting` (comparer/finalize.rs:3399) L7474; `Dom::add_text` (xmllinq/mod.rs:1087) L7476; `Dom::add_after_self` (xmllinq/mod.rs:1147) L7479; `Dom::add_before_self` (xmllinq/mod.rs:1130) L7487,7530; `Dom::remove` (xmllinq/mod.rs:1118) L7528,7550,7567,7582
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.chars`, `.collect`, `.copied`, `.count`, `.ends_with`, `.enumerate`, `.eq_ignore_ascii_case`, `.filter_map`, `.filter`, `.find`, `.first`, `.flat_map`, `.get`, `.into_iter`, `.into_owned`, `.is_alphanumeric`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some`, `.is_whitespace`, `.iter`, `.last`, `.len`, `.map`, `.min`, `.name`, `.position`, `.push_str`, `.push`, `.remove`, `.rev`, `.saturating_add`, `.split`, `.starts_with`, `.then_some`, `.to_ascii_lowercase`, `.to_string`, `.unwrap_or`, `.unwrap`, `.windows`, `Some`, `String::new`, `Vec::new`, `W::del_text`, `W::del`, `W::ins`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `W::t`
- **Macros:** `matches!`
- **Called by:** `fold_short_label_ins_into_del_with_settings` (comparer/finalize.rs:7058); `mesh_short_label_shared_eq` (comparer/finalize.rs:7199); `short_label_format_history_tests::configured_label_tracking_preserves_sources_and_respects_disabled_changes` (comparer/finalize.rs:24170); `demo_title_pilcrow_history_tests::sharing_a_label_with_nested_effects_obeys_disabled_format_tracking` (comparer/finalize.rs:28770); `finalizer_distinct_boundary_contract_tests::equal_shared_labels_cannot_cross_opaque_or_historical_run_ownership` (comparer/finalize.rs:30017); `fragmented_source_label_boundary_tests::fragmented_labels_keep_literal_source_chars_formats_and_revision_history` (comparer/finalize.rs:33728); `fragmented_source_label_boundary_tests::label_format_boundaries_and_disabled_history_leave_complete_original_sources_intact` (comparer/finalize.rs:33882)

#### `mesh_short_label_shared_eq_with_settings::{fn label_run}`  (L7322-7343)
`fn label_run(dom: &Dom, leaves: &[NodeId], mut bytes: usize) -> Option<NodeId>`

- **Does:** A shared label may span text leaves, but must not cross a property boundary on either source side. _(from a code comment)_
- **Calls (crate):** `Dom::value_str` (xmllinq/mod.rs:1195) L7325; `Dom::parent` (xmllinq/mod.rs:597) L7329; `rpr_string` (comparer/finalize.rs:1090) L7331
- **Calls (std/external/unresolved):** `.len`, `.saturating_sub`, `Some`
- **Called by:** `mesh_short_label_shared_eq_with_settings` (comparer/finalize.rs:7204)

#### `trailing_alnum_token`  (L7588-7602)
`fn trailing_alnum_token(text: &str) -> Option<String>`

- **Does:** Last alphanumeric token of `text` with original casing (for EQ peel).
- **Calls (std/external/unresolved):** `.chars`, `.is_alphanumeric`, `.is_empty`, `.push`, `Some`, `String::new`, `std::mem::take`
- **Called by:** `free_mesh_shared_title_token_in_mix` (comparer/finalize.rs:2573); `mesh_short_label_shared_eq_with_settings` (comparer/finalize.rs:7204); `strip_trailing_alnum_token` (comparer/finalize.rs:7606); `coverage_tests::token_splitting_preserves_separators_and_original_case` (comparer/finalize.rs:15744)

#### `strip_trailing_alnum_token`  (L7606-7624)
`fn strip_trailing_alnum_token(text: &str, label: &str) -> Option<String>`

- **Does:** If `text` ends with alnum token matching `label` (case-insensitive), return the prefix with that token removed (separators before it kept).
- **Calls (crate):** `trailing_alnum_token` (comparer/finalize.rs:7588) L7607
- **Calls (std/external/unresolved):** `.char_indices`, `.collect`, `.eq_ignore_ascii_case`, `.is_alphanumeric`, `.len`, `.to_string`, `Some`
- **Called by:** `free_mesh_shared_title_token_in_mix` (comparer/finalize.rs:2573); `mesh_short_label_shared_eq_with_settings` (comparer/finalize.rs:7204); `coverage_tests::token_splitting_preserves_separators_and_original_case` (comparer/finalize.rs:15744)

#### `para_mark_revision`  (L7631-7635)
`fn para_mark_revision(dom: &Dom, p: NodeId, rev: &crate::xmllinq::XName) -> bool`

- **Does:** True when `w:pPr/w:rPr` carries a paragraph-mark revision (`w:del`/`w:ins`).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L7632,7633,7634
- **Calls (std/external/unresolved):** `.and_then`, `.is_some_and`, `.is_some`, `W::p_pr`, `W::r_pr`
- **Called by:** `unwrap_content_controls_in_pure_revisions` (comparer/finalize.rs:1454); `fold_short_list_label_into_empty_pure_del` (comparer/finalize.rs:1744); `strip_trailing_empty_pure_del_mark` (comparer/finalize.rs:2951); `merged_closing_carrier_takes_source_properties` (comparer/finalize.rs:3122); `story_final_kept_blank` (comparer/finalize.rs:3542); `para_is_empty_pure_ins` (comparer/finalize.rs:3583); `para_is_pure_inserted` (comparer/finalize.rs:4216); `rotate_ins_mark_del_only_paragraph` (comparer/finalize.rs:4617); `restamp_stranded_del_mark_onto_del_only_paragraph` (comparer/finalize.rs:4775); `split_head_short_title_long_del_mix` (comparer/finalize.rs:4870); `fold_leading_ins_from_mix_into_preceding_pure_del` (comparer/finalize.rs:4991); `para_is_mixed_revision` (comparer/finalize.rs:5158); +43 more

#### `fold_crosses_a_field`  (L7642-7662)
`fn fold_crosses_a_field(dom: &Dom, d: NodeId, carrier: NodeId, inss: &[NodeId]) -> bool`

- **Does:** Folding `d` into `carrier` appends d's body after the carrier's, above every inserted paragraph between them.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L7647,7661; `Dom::attribute` (xmllinq/mod.rs:874) L7648
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.iter`, `.skip_while`, `.skip`, `Some`, `W::name`
- **Called by:** `merge_replaced_in_container` (comparer/finalize.rs:8626); `coverage_tests::field_fold_rejects_crossing_but_allows_balanced_deleted_fields` (comparer/finalize.rs:16727)

#### `para_is_field_residue`  (L7669-7676)
`fn para_is_field_residue(dom: &Dom, p: NodeId) -> bool`

- **Does:** M474 — a "field residue" paragraph (M360's TOC field-end fragments) carries fldChar but NO visible text.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L7670,7673; `Dom::value_str` (xmllinq/mod.rs:1195) L7675
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.iter`, `.trim`, `Some`, `W::name`, `W::t`
- **Called by:** `fold_whitespace_pure_ins_into_following_pure_del` (comparer/finalize.rs:4252); `merge_replaced_in_container` (comparer/finalize.rs:8626); `coverage_tests::field_fold_rejects_crossing_but_allows_balanced_deleted_fields` (comparer/finalize.rs:16727)

#### `accumulate_para_child_class`  (L7683-7691)
`fn accumulate_para_child_class( dom: &Dom, c: NodeId, ins: &mut bool, del: &mut bool, plain: &mut bool, )`

- **Does:** Some(false) = fully deleted, Some(true) = fully inserted, None = neither.
- **Calls (crate):** `accumulate_para_child_class_with_moves` (comparer/finalize.rs:7693) L7690
- **Called by:** `para_replacement_class_with_moves` (comparer/finalize.rs:7747)

#### `accumulate_para_child_class_with_moves`  (L7693-7741)
`fn accumulate_para_child_class_with_moves( dom: &Dom, c: NodeId, ins: &mut bool, del: &mut bool, plain: &mut bool, include_moves: bool, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L7701; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L7707; `XName::local_name` (xmllinq/mod.rs:208) (by name) L7708; `Dom::elements` (xmllinq/mod.rs:640) L7735; `accumulate_para_child_class_with_moves` (comparer/finalize.rs:7693) L7736
- **Calls (std/external/unresolved):** `W::del`, `W::hyperlink`, `W::ins`, `W::name`, `W::p_pr`
- **Called by:** `accumulate_para_child_class` (comparer/finalize.rs:7683); `accumulate_para_child_class_with_moves` (comparer/finalize.rs:7693); `para_replacement_class_with_moves` (comparer/finalize.rs:7747)

#### `para_replacement_class`  (L7743-7745)
`fn para_replacement_class(dom: &Dom, p: NodeId) -> Option<bool>`

- **Does:** _no doc comment_
- **Calls (crate):** `para_replacement_class_with_moves` (comparer/finalize.rs:7747) L7744
- **Called by:** `reorder_replaced_blocks::{fn block_class}` (comparer/finalize.rs:10081); `splice_trailing_short_pure_dels_midstream` (comparer/finalize.rs:11803); `coverage_tests::replacement_classification_is_transparent_only_for_hyperlinks` (comparer/finalize.rs:15557); `coverage_tests::row_move_classification_requires_matching_structural_paragraph_marks` (comparer/finalize.rs:15629)

#### `para_replacement_class_with_moves`  (L7747-7786)
`fn para_replacement_class_with_moves(dom: &Dom, p: NodeId, include_moves: bool) -> Option<bool>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L7748; `Dom::elements` (xmllinq/mod.rs:640) L7752; `accumulate_para_child_class_with_moves` (comparer/finalize.rs:7693) L7754; `accumulate_para_child_class` (comparer/finalize.rs:7683) L7756; `para_mark_revision` (comparer/finalize.rs:7631) L7760,7762,7781; `Dom::descendants` (xmllinq/mod.rs:668) L7781
- **Calls (std/external/unresolved):** `.is_empty`, `Some`, `W::del`, `W::ins`, `W::name`, `W::p`
- **Called by:** `para_replacement_class` (comparer/finalize.rs:7743); `mark_fully_revised_rows` (comparer/finalize.rs:11671); `coverage_tests::row_move_classification_requires_matching_structural_paragraph_marks` (comparer/finalize.rs:15629)

#### `para_class_carried_aware`  (L7794-7849)
`fn para_class_carried_aware(dom: &Dom, p: NodeId, comparer_author: &str) -> Option<bool>`

- **Does:** Like [`para_replacement_class`], but a `w:ins` authored by someone OTHER than the comparer (a CARRIED pre-existing insertion, D1 machinery) counts toward the DELETED side: GT places doc-A blocks whose only live content is carried foreign-author ins inside the trailing deleted cluster (contract-review-suggesting-mixed-...
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L7795; `Dom::elements` (xmllinq/mod.rs:640) L7799,7816; `Dom::name` (xmllinq/mod.rs:526) L7800,7817; `Dom::attribute` (xmllinq/mod.rs:874) L7807,7822; `para_mark_revision` (comparer/finalize.rs:7631) L7838,7840
- **Calls (std/external/unresolved):** `.unwrap_or`, `Some`, `W::author`, `W::del`, `W::hyperlink`, `W::ins`, `W::name`, `W::p_pr`, `W::p`
- **Called by:** `merge_replaced_in_container` (comparer/finalize.rs:8626); `coverage_tests::replacement_classification_is_transparent_only_for_hyperlinks` (comparer/finalize.rs:15557)

#### `para_has_real_del`  (L7854-7856)
`fn para_has_real_del(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when the paragraph holds a REAL deletion (any `w:del` child) — a carried-only "deleted-side" paragraph must not 1v1-merge into a mixed paragraph (its content is pending ins, not replaced text).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L7855
- **Calls (std/external/unresolved):** `.is_empty`, `Some`, `W::del`
- **Called by:** `fold_short_list_label_into_empty_pure_del` (comparer/finalize.rs:1744); `promote_live_numpr_on_pure_d_from_pprchange` (comparer/finalize.rs:1908); `merge_replaced_in_container` (comparer/finalize.rs:8626)

#### `para_revision_body_text`  (L7859-7868)
`fn para_revision_body_text(dom: &Dom, p: NodeId) -> String`

- **Does:** Live + deleted body text of a paragraph (for relatedness gates).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L7862; `Dom::value_str` (xmllinq/mod.rs:1195) L7863
- **Calls (std/external/unresolved):** `.push_str`, `.push`, `Some`, `String::new`, `W::del_text`, `W::name`
- **Called by:** `promote_live_numpr_on_pure_d_from_pprchange` (comparer/finalize.rs:1908); `ensure_pure_i_list_snug_spacing` (comparer/finalize.rs:2113); `para_has_repeated_phrase` (comparer/finalize.rs:3565); `relocate_title_page_last_empty_after_pure_dels` (comparer/finalize.rs:3653); `fold_leading_ins_from_mix_into_preceding_pure_del` (comparer/finalize.rs:4991); `park_mixed_spacing_onto_trailing_pure_del` (comparer/finalize.rs:5607); `residual_short_label_zip_with_settings` (comparer/finalize.rs:6930); `para_body_is_digits_only` (comparer/finalize.rs:7879); `para_body_alnum_len` (comparer/finalize.rs:7889); `para_looks_like_demo_title` (comparer/finalize.rs:7906); `should_fold_ins_del_pair` (comparer/finalize.rs:8002); `para_word_atom_count` (comparer/finalize.rs:8013); +13 more

#### `body_token_set`  (L7870-7875)
`fn body_token_set(text: &str) -> std::collections::HashSet<String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.is_alphanumeric`, `.is_empty`, `.map`, `.split`, `.to_ascii_lowercase`
- **Called by:** `park_mixed_spacing_onto_trailing_pure_del` (comparer/finalize.rs:5607); `body_text_jaccard` (comparer/finalize.rs:7971); `should_fold_multi_del_at_document_scale` (comparer/finalize.rs:8025); `merge_replaced_in_container` (comparer/finalize.rs:8626); `deep_production_path_tests::punctuation_only_items_are_contentful_and_can_supply_empty_token_relatedness` (comparer/finalize.rs:23366)

#### `para_body_is_digits_only`  (L7879-7886)
`fn para_body_is_digits_only(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when paragraph body text (live + delText) is only digits/whitespace (file_166 pure-I "24" — corpus fragment, not content fold bait).
- **Calls (crate):** `para_revision_body_text` (comparer/finalize.rs:7859) L7880
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.is_ascii_digit`, `.is_empty`, `.is_whitespace`, `.trim`
- **Called by:** `merge_replaced_in_container` (comparer/finalize.rs:8626); `coverage_tests::relatedness_and_short_title_gates_use_tokens_not_substrings` (comparer/finalize.rs:15785)

#### `para_body_alnum_len`  (L7889-7894)
`fn para_body_alnum_len(dom: &Dom, p: NodeId) -> usize`

- **Does:** Alphanumeric character count of para body text (revision-aware).
- **Calls (crate):** `para_revision_body_text` (comparer/finalize.rs:7859) L7890
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.filter`, `.is_alphanumeric`
- **Called by:** `ensure_empty_pure_i_before_short_title_del` (comparer/finalize.rs:3951); `strip_empty_pure_ins_before_trailing_pure_dels` (comparer/finalize.rs:4103); `fold_whitespace_pure_ins_into_following_pure_del` (comparer/finalize.rs:4252); `park_mixed_spacing_onto_trailing_pure_del` (comparer/finalize.rs:5607); `residual_short_label_zip_with_settings` (comparer/finalize.rs:6930); `para_body_is_very_short` (comparer/finalize.rs:7899); `merge_replaced_in_container` (comparer/finalize.rs:8626); `splice_trailing_short_pure_dels_midstream` (comparer/finalize.rs:11803); `restore_short_del_before_long_ins` (comparer/finalize.rs:12185)

#### `para_body_is_very_short`  (L7899-7901)
`fn para_body_is_very_short(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when body alphanumerics are 1–2 chars ("a", "x", "OK").
- **Calls (crate):** `para_body_alnum_len` (comparer/finalize.rs:7889) L7900
- **Calls (std/external/unresolved):** `.contains`
- **Called by:** `residual_short_label_zip_with_settings` (comparer/finalize.rs:6930); `merge_replaced_in_container` (comparer/finalize.rs:8626); `coverage_tests::relatedness_and_short_title_gates_use_tokens_not_substrings` (comparer/finalize.rs:15785)

#### `para_looks_like_demo_title`  (L7906-7920)
`fn para_looks_like_demo_title(dom: &Dom, p: NodeId) -> bool`

- **Does:** True when body looks like a short demo title (≤8 tokens, last significant token is `Demo`).
- **Calls (crate):** `para_revision_body_text` (comparer/finalize.rs:7859) L7907
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.count`, `.eq_ignore_ascii_case`, `.filter`, `.find`, `.is_alphanumeric`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.rev`, `.split`
- **Called by:** `ensure_empty_pure_i_before_short_title_del` (comparer/finalize.rs:3951); `strip_empty_pure_ins_before_trailing_pure_dels` (comparer/finalize.rs:4103); `fold_whitespace_pure_ins_into_following_pure_del` (comparer/finalize.rs:4252); `merge_replaced_in_container` (comparer/finalize.rs:8626); `capture_deleted_demo_title_properties` (comparer/finalize.rs:12584); `fold_midstream_demo_title_into_numbered_heading_with_source_properties` (comparer/finalize.rs:12624); `coverage_tests::relatedness_and_short_title_gates_use_tokens_not_substrings` (comparer/finalize.rs:15785)

#### `para_has_heading_or_title_style`  (L7923-7933)
`fn para_has_heading_or_title_style(dom: &Dom, p: NodeId) -> bool`

- **Does:** Live `w:pStyle` is Title / Heading* (Heading1, Heading2, …).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L7924,7927; `Dom::attribute` (xmllinq/mod.rs:874) L7930
- **Calls (std/external/unresolved):** `.starts_with`, `.to_ascii_lowercase`, `.unwrap_or`, `W::p_pr`, `W::p_style`, `W::val`
- **Called by:** `ensure_empty_pure_i_before_short_title_del` (comparer/finalize.rs:3951); `strip_empty_pure_ins_before_trailing_pure_dels` (comparer/finalize.rs:4103); `fold_whitespace_pure_ins_into_following_pure_del` (comparer/finalize.rs:4252); `should_fold_multi_del_at_document_scale` (comparer/finalize.rs:8025); `merge_replaced_in_container` (comparer/finalize.rs:8626); `deep_production_path_tests::long_content_cannot_reach_back_across_an_empty_tail_without_a_head_match` (comparer/finalize.rs:21163); `deep_production_path_tests::deletion_first_short_residual_and_field_carrier_keep_source_properties` (comparer/finalize.rs:21301); `deep_production_path_tests::heading_skip_is_limited_to_short_multitoken_headings_and_nonlist_carriers` (comparer/finalize.rs:21458)

#### `para_has_heading_style`  (L7936-7945)
`fn para_has_heading_style(dom: &Dom, p: NodeId) -> bool`

- **Does:** Live `w:pStyle` is Heading* only (not Title).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L7937,7940; `Dom::attribute` (xmllinq/mod.rs:874) L7943
- **Calls (std/external/unresolved):** `.starts_with`, `.to_ascii_lowercase`, `.unwrap_or`, `W::p_pr`, `W::p_style`, `W::val`
- **Called by:** `merge_replaced_in_container` (comparer/finalize.rs:8626); `deep_production_path_tests::styled_inserted_titles_keep_unrelated_short_base_titles_except_demos` (comparer/finalize.rs:21413)

#### `short_title_shares_sig_token`  (L7949-7965)
`fn short_title_shares_sig_token(ins_text: &str, del_text: &str) -> bool`

- **Does:** Shared significant token (len≥4) between short-title pure-D and first pure-I (M322 head-junction).
- **Calls (crate):** `memoized_moves_tests::sig` (comparer/moves.rs:882) (by name) L7962,7963
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.filter`, `.intersection`, `.is_alphanumeric`, `.is_empty`, `.is_some`, `.iter`, `.len`, `.map`, `.next`, `.split`, `.to_ascii_lowercase`
- **Called by:** `merge_replaced_in_container` (comparer/finalize.rs:8626); `coverage_tests::relatedness_and_short_title_gates_use_tokens_not_substrings` (comparer/finalize.rs:15785)

#### `body_text_jaccard`  (L7971-7980)
`fn body_text_jaccard(a: &str, b: &str) -> f64`

- **Does:** Token Jaccard of two body strings.
- **Calls (crate):** `body_token_set` (comparer/finalize.rs:7870) L7972,7973
- **Calls (std/external/unresolved):** `.count`, `.intersection`, `.is_empty`, `.union`
- **Called by:** `should_fold_ins_del_pair` (comparer/finalize.rs:8002); `coverage_tests::relatedness_and_short_title_gates_use_tokens_not_substrings` (comparer/finalize.rs:15785); `deep_production_path_tests::midstory_checklist_cell_guard_distinguishes_titles_at_a_weak_head_junction` (comparer/finalize.rs:21599)

#### `para_has_live_numpr`  (L7995-8000)
`fn para_has_live_numpr(dom: &Dom, p: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L7996,7999
- **Calls (std/external/unresolved):** `.is_some`, `W::num_pr`, `W::p_pr`
- **Called by:** `fold_short_list_label_into_empty_pure_del` (comparer/finalize.rs:1744); `promote_live_numpr_on_pure_d_from_pprchange` (comparer/finalize.rs:1908); `should_fold_multi_del_at_document_scale` (comparer/finalize.rs:8025); `merge_replaced_in_container` (comparer/finalize.rs:8626); `deep_production_path_tests::prior_nested_cluster_keeps_short_labels_but_allows_a_longer_carrier` (comparer/finalize.rs:21130); `deep_production_path_tests::list_carriers_override_the_long_prose_list_skip_and_retain_their_geometry` (comparer/finalize.rs:21504)

#### `should_fold_ins_del_pair`  (L8002-8010)
`fn should_fold_ins_del_pair(dom: &Dom, ins_p: NodeId, del_p: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `para_revision_body_text` (comparer/finalize.rs:7859) L8003,8004; `body_text_jaccard` (comparer/finalize.rs:7971) L8009
- **Calls (std/external/unresolved):** `.is_empty`, `.trim`
- **Called by:** `should_fold_multi_del_at_document_scale` (comparer/finalize.rs:8025); `merge_replaced_in_container` (comparer/finalize.rs:8626); `coverage_tests::sole_pair_relatedness_boundary_is_inclusive` (comparer/finalize.rs:16232); `deep_production_path_tests::punctuation_only_items_are_contentful_and_can_supply_empty_token_relatedness` (comparer/finalize.rs:23366)

#### `para_word_atom_count`  (L8013-8016)
`fn para_word_atom_count(dom: &Dom, p: NodeId) -> usize`

- **Does:** Word-atom count for body text under a paragraph (whitespace-split tokens).
- **Calls (crate):** `para_revision_body_text` (comparer/finalize.rs:7859) L8014
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.is_empty`, `.split_whitespace`
- **Called by:** `fold_short_list_label_into_empty_pure_del` (comparer/finalize.rs:1744); `promote_live_numpr_on_pure_d_from_pprchange` (comparer/finalize.rs:1908); `ensure_pure_i_list_snug_spacing` (comparer/finalize.rs:2113); `strip_redundant_demo_default_spacing` (comparer/finalize.rs:2308); `ensure_empty_pure_i_before_short_title_del` (comparer/finalize.rs:3951); `strip_empty_pure_ins_before_trailing_pure_dels` (comparer/finalize.rs:4103); `fold_whitespace_pure_ins_into_following_pure_del` (comparer/finalize.rs:4252); `should_fold_multi_del_at_document_scale` (comparer/finalize.rs:8025); `merge_replaced_in_container` (comparer/finalize.rs:8626); `interleave_list_cluster_after_coalesce` (comparer/finalize.rs:9881)

#### `should_fold_multi_del_at_document_scale`  (L8025-8343)
`fn should_fold_multi_del_at_document_scale( dom: &Dom, container: NodeId, last_ins: NodeId, first_del: NodeId, inss: &[NodeId], dels: &[NodeId], ) -> bool`

- **Does:** Document-scale gate for multi-del boundary fold (plan B1 / C1).
- **Calls (crate):** `para_revision_body_text` (comparer/finalize.rs:7859) L8037,8072,8081,8084,8113,8118...; `should_fold_ins_del_pair` (comparer/finalize.rs:8002) L8038,8220,8255,8272,8282,8317; `Dom::element` (xmllinq/mod.rs:655) L8046,8047; `Dom::attribute` (xmllinq/mod.rs:874) L8048; `para_has_heading_or_title_style` (comparer/finalize.rs:7923) L8055; `para_word_atom_count` (comparer/finalize.rs:8013) L8056,8074,8077,8161,8165,8169...; `para_has_live_numpr` (comparer/finalize.rs:7995) L8088,8089,8156,8177; `body_token_set` (comparer/finalize.rs:7870) L8128,8132; `para_has_repeated_phrase` (comparer/finalize.rs:3565) L8271; `Dom::elements` (xmllinq/mod.rs:640) L8334; `Dom::name_is` (xmllinq/mod.rs:535) L8337
- **Calls (std/external/unresolved):** `.all`, `.any`, `.chars`, `.collect`, `.contains`, `.copied`, `.count`, `.filter`, `.intersection`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.map`, `.max`, `.min`, `.next`, `.saturating_mul`, `.split_whitespace`, `.starts_with`, `.sum`, `.to_ascii_lowercase`, `.trim`, `.union`, `.unwrap_or`, `W::p_pr`, `W::p_style`, `W::p`, `W::val`, `all_short`, `list_frac`
- **Called by:** `merge_replaced_in_container` (comparer/finalize.rs:8626); `coverage_tests::fold_gate` (comparer/finalize.rs:15980); `coverage_tests::document_scale_list_and_cover_special_cases` (comparer/finalize.rs:16075); `deep_production_path_tests::numbered_wholesale_gate_requires_the_last_new_item_and_half_of_old_items_numbered` (comparer/finalize.rs:23329); `deep_production_path_tests::punctuation_only_items_are_contentful_and_can_supply_empty_token_relatedness` (comparer/finalize.rs:23366); `finalizer_remaining_source_boundary_tests::document_scale_folds_preserve_authored_sources_and_observed_word_exceptions` (comparer/finalize.rs:30795); `reached_word_document_policy_tests::document_policy_reaches_numbering_relatedness_and_demo_intro_boundaries` (comparer/finalize.rs:34432)

#### `gap_ends_story`  (L8346-8348)
`fn gap_ends_story(dom: &Dom, rest: &[NodeId]) -> bool`

- **Does:** Only the section properties follow a replace gap: it ends the story.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L8347
- **Calls (std/external/unresolved):** `.all`, `.iter`, `W::sect_pr`
- **Called by:** `merge_replaced_in_container` (comparer/finalize.rs:8626)

#### `gap_precedes_live_story_tail`  (L8360-8384)
`fn gap_precedes_live_story_tail( dom: &Dom, rest: &[NodeId], classes: &[Option<bool>], fold_boundary: bool, ) -> bool`

- **Does:** A replace gap followed by deleted blocks and then the story's final paragraph, holding nothing live or inserted (empty, or the original's deleted last words; its mark is the story's final pilcrow, which Word never deletes, so a later pass restores it), while the final marks are no empty pair (`fold_boundary`): the rev...
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L8367,8374; `para_mark_revision` (comparer/finalize.rs:7631) L8375; `Dom::descendants` (xmllinq/mod.rs:668) L8376,8377; `Dom::value_str` (xmllinq/mod.rs:1195) L8380; `table_is_deleted` (comparer/finalize.rs:8421) L8383
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.filter`, `.is_empty`, `.iter`, `.len`, `.split_last`, `.trim`, `Some`, `W::ins`, `W::p`, `W::sect_pr`, `W::t`
- **Called by:** `merge_replaced_in_container` (comparer/finalize.rs:8626)

#### `para_carries_page_break`  (L8389-8396)
`fn para_carries_page_break(dom: &Dom, p: NodeId) -> bool`

- **Does:** A page break keeps its paragraph whole: Word never fuses it, so the break keeps paginating.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L8390,8391; `Dom::descendants` (xmllinq/mod.rs:668) L8392; `Dom::attribute` (xmllinq/mod.rs:874) L8395
- **Calls (std/external/unresolved):** `.any`, `.is_some_and`, `.is_some`, `.iter`, `Some`, `W::name`, `W::p_pr`
- **Called by:** `merge_replaced_in_container` (comparer/finalize.rs:8626); `coverage_tests::field_fold_rejects_crossing_but_allows_balanced_deleted_fields` (comparer/finalize.rs:16727); `deep_production_path_tests::explicit_page_breaks_block_final_story_fusion_on_either_source_side` (comparer/finalize.rs:23288)

#### `fuse_story_tail_into_deleted`  (L8400-8416)
`fn fuse_story_tail_into_deleted(dom: &mut Dom, last_ins: NodeId, first_del: NodeId)`

- **Does:** Word's story-tail fusion: the revised last paragraph's runs open the first deleted paragraph, which keeps its own properties and deleted mark.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L8401,8405; `Dom::name_is` (xmllinq/mod.rs:535) L8404,8406; `Dom::remove` (xmllinq/mod.rs:1118) L8409,8415; `Dom::add_before_self` (xmllinq/mod.rs:1130) L8411; `Dom::add` (xmllinq/mod.rs:1074) L8412
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `W::p_pr`
- **Called by:** `merge_replaced_in_container` (comparer/finalize.rs:8626)

#### `table_is_deleted`  (L8421-8431)
`fn table_is_deleted(dom: &Dom, tbl: NodeId) -> bool`

- **Does:** Every paragraph of the table is deleted, marks included, and nothing in it is live or inserted (the rows get their deleted marks later, from `mark_fully_revised_rows`).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L8422,8426,8427; `Dom::name_is` (xmllinq/mod.rs:535) L8423; `para_mark_revision` (comparer/finalize.rs:7631) L8425; `Dom::value_str` (xmllinq/mod.rs:1195) L8430
- **Calls (std/external/unresolved):** `.all`, `.is_empty`, `.iter`, `.trim`, `Some`, `W::del`, `W::ins`, `W::p`, `W::t`, `W::tbl`
- **Called by:** `gap_precedes_live_story_tail` (comparer/finalize.rs:8360)

#### `paragraph_has_live_empty_pilcrow`  (L8436-8467)
`fn paragraph_has_live_empty_pilcrow(dom: &Dom, p: NodeId) -> bool`

- **Does:** A live empty pilcrow, optionally carrying structurally inert runs.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L8437,8445,8448,8452,8455; `para_mark_revision` (comparer/finalize.rs:7631) L8440; `Dom::elements` (xmllinq/mod.rs:640) L8444,8451,8456; `Dom::value_str` (xmllinq/mod.rs:1195) L8453; `Dom::name` (xmllinq/mod.rs:526) L8457; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L8458; `XName::local_name` (xmllinq/mod.rs:208) (by name) L8460; `Dom::nodes` (xmllinq/mod.rs:604) L8463
- **Calls (std/external/unresolved):** `.all`, `.any`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `W::del`, `W::ins`, `W::move_from`, `W::move_to`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `W::t`
- **Macros:** `matches!`
- **Called by:** `gap_has_live_blank_cell_tail` (comparer/finalize.rs:8474); `story_tail_has_paired_original_boundary` (comparer/finalize.rs:8611); `merge_replaced_in_container` (comparer/finalize.rs:8626); `coverage_real_revision_merge_matrix_tests::multiple_original_paragraphs_before_a_live_blank_control_tail_keep_exact_phase_payloads` (comparer/finalize.rs:25428); `coverage_real_revision_merge_matrix_tests::multiple_revised_paragraphs_with_distinct_tail_properties_keep_word_deleted_carrier` (comparer/finalize.rs:25560); `coverage_real_revision_merge_matrix_tests::multiple_original_blank_tail_checks_every_later_pilcrow_provenance` (comparer/finalize.rs:25666); `coverage_real_revision_merge_matrix_tests::multiple_original_blank_tail_preserves_first_original_section_and_property_history` (comparer/finalize.rs:25801); `coverage_real_revision_merge_matrix_tests::multiple_revised_blank_tail_pairs_only_the_matching_final_revised_paragraph` (comparer/finalize.rs:26022); `coverage_real_revision_merge_matrix_tests::empty_pilcrow_proof_retains_inert_formatting_and_declines_owned_payloads` (comparer/finalize.rs:26464)

#### `gap_has_live_blank_cell_tail`  (L8474-8480)
`fn gap_has_live_blank_cell_tail(dom: &Dom, container: NodeId, rest: &[NodeId]) -> bool`

- **Does:** An explicit live empty cell paragraph owns a separate final pilcrow on both sources.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L8475; `paragraph_has_live_empty_pilcrow` (comparer/finalize.rs:8436) L8479
- **Calls (std/external/unresolved):** `.all`, `.is_empty`, `.iter`, `W::tc`
- **Called by:** `merge_replaced_in_container` (comparer/finalize.rs:8626)

#### `paired_replacement_marks_belong_to_comparer`  (L8485-8522)
`fn paired_replacement_marks_belong_to_comparer( dom: &Dom, deleted: NodeId, inserted: NodeId, comparer_author: &str, ) -> bool`

- **Does:** Only comparison-owned pilcrows may become an unmarked paired boundary.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L8497,8500; `Dom::elements` (xmllinq/mod.rs:640) L8503,8504; `Dom::attribute` (xmllinq/mod.rs:874) L8508,8518,8519; `Dom::descendants` (xmllinq/mod.rs:668) L8511; `Dom::ancestors` (xmllinq/mod.rs:770) L8514
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.len`, `.unwrap_or`, `Some`, `W::author`, `W::date`, `W::del`, `W::ins`, `W::p_pr`, `W::r_pr`
- **Called by:** `story_tail_has_paired_original_boundary` (comparer/finalize.rs:8611); `merge_replaced_in_container` (comparer/finalize.rs:8626); `finalizer_remaining_source_boundary_tests::paired_lifetimes_require_complete_matching_comparison_provenance` (comparer/finalize.rs:31123)

#### `paired_replacement_has_identical_properties`  (L8529-8605)
`fn paired_replacement_has_identical_properties( dom: &mut Dom, deleted: NodeId, inserted: NodeId, ) -> bool`

- **Does:** A related 1v1 pair with identical complete paragraph properties owns a single live pilcrow in Word's mixed paragraph.
- **Calls (crate):** `para_mark_revision` (comparer/finalize.rs:7631) L8534; `Dom::element` (xmllinq/mod.rs:655) L8539,8540,8545,8546,8550,8570; `Dom::elements` (xmllinq/mod.rs:640) L8551,8572,8576; `Dom::name_is` (xmllinq/mod.rs:535) L8554; `Dom::new` (xmllinq/mod.rs:387) L8562; `Dom::serialize_element` (xmllinq/mod.rs:1315) L8564,8601; `Dom::with_scratch` (xmllinq/mod.rs:449) L8565; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L8566; `Dom::root` (xmllinq/mod.rs:822) L8567; `Dom::remove` (xmllinq/mod.rs:1118) L8573,8577; `remove_powertools_scratch_markup` (comparer/finalize.rs:1060) L8580; `Dom::attributes` (xmllinq/mod.rs:899) L8584,8590; `Dom::is_namespace_declaration` (xmllinq/mod.rs:1012) L8588; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L8599
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.expect`, `.filter`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.map`, `Some`, `W::del`, `W::ins`, `W::move_from`, `W::move_to`, `W::p_pr_change`, `W::p_pr`, `W::r_pr_change`, `W::r_pr`, `W::sect_pr`
- **Called by:** `story_tail_has_paired_original_boundary` (comparer/finalize.rs:8611); `merge_replaced_in_container` (comparer/finalize.rs:8626); `coverage_real_revision_merge_matrix_tests::multiple_original_blank_tail_preserves_first_original_section_and_property_history` (comparer/finalize.rs:25801); `coverage_real_revision_merge_matrix_tests::paired_property_comparison_never_grows_the_persistent_dom_or_changes_sources` (comparer/finalize.rs:26414); `finalizer_remaining_source_boundary_tests::paired_format_equality_is_complete_and_uses_no_production_arena_nodes` (comparer/finalize.rs:31206)

#### `story_tail_has_paired_original_boundary`  (L8611-8624)
`fn story_tail_has_paired_original_boundary( dom: &mut Dom, deleted: &[NodeId], inserted: NodeId, rest: &[NodeId], comparer_author: &str, ) -> bool`

- **Does:** A shared blank cannot supply the pilcrow for revised text whose source had its own paragraph properties.
- **Calls (crate):** `paragraph_has_live_empty_pilcrow` (comparer/finalize.rs:8436) L8618; `paired_replacement_marks_belong_to_comparer` (comparer/finalize.rs:8485) L8621; `paired_replacement_has_identical_properties` (comparer/finalize.rs:8529) L8623
- **Calls (std/external/unresolved):** `.all`, `.is_empty`, `.iter`
- **Macros:** `matches!`
- **Called by:** `merge_replaced_in_container` (comparer/finalize.rs:8626); `coverage_real_revision_merge_matrix_tests::multiple_original_blank_tail_checks_every_later_pilcrow_provenance` (comparer/finalize.rs:25666); `coverage_real_revision_merge_matrix_tests::multiple_original_blank_tail_preserves_first_original_section_and_property_history` (comparer/finalize.rs:25801)

#### `merge_replaced_in_container`  (L8626-9860)
`fn merge_replaced_in_container( dom: &mut Dom, container: NodeId, comparer_author: &str, fold_boundary: bool, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L8633,8707,8710,8826,8842,8847...; `para_class_carried_aware` (comparer/finalize.rs:7794) L8636; `gap_has_live_blank_cell_tail` (comparer/finalize.rs:8474) L8659,8978; `para_has_real_del` (comparer/finalize.rs:7854) L8678,8717,8761,9023,9028,9065; `gap_precedes_live_story_tail` (comparer/finalize.rs:8360) L8679,8982; `para_has_no_text` (comparer/finalize.rs:3500) L8680,8983,9025,9030,9049,9122...; `para_mark_revision` (comparer/finalize.rs:7631) L8681,8914,8984,9024,9029,9035...; `para_carries_page_break` (comparer/finalize.rs:8389) L8682,8683,8985,8986; `Dom::element` (xmllinq/mod.rs:655) L8684,8686,8703,8705,8728,8729...; `Dom::remove` (xmllinq/mod.rs:1118) L8689,8708,8711,8770,8822,8839...; `Dom::add_before_self` (xmllinq/mod.rs:1130) L8690,8771,8827,8865,9804,9825; `paragraph_has_live_empty_pilcrow` (comparer/finalize.rs:8436) L8693; `paired_replacement_marks_belong_to_comparer` (comparer/finalize.rs:8485) L8694,8915; `paired_replacement_has_identical_properties` (comparer/finalize.rs:8529) L8700,8916; `fuse_story_tail_into_deleted` (comparer/finalize.rs:8400) L8701,8998; `Dom::attribute` (xmllinq/mod.rs:874) L8731,8889,9055,9113,9625,9635...; `para_revision_body_text` (comparer/finalize.rs:7859) L8740,8802,9189,9190,9213,9214...; `Dom::parent` (xmllinq/mod.rs:597) L8767,8799,8800,8863; `gap_ends_story` (comparer/finalize.rs:8346) L8796,9233; `body_token_set` (comparer/finalize.rs:7870) L8802,9497,9498,9524,9525,9578...; `para_body_alnum_len` (comparer/finalize.rs:7889) L8804,9553,9575,9590,9600,9621...; `should_fold_ins_del_pair` (comparer/finalize.rs:8002) L8806,8862,9158,9180,9234,9279...; `para_is_field_residue` (comparer/finalize.rs:7669) L8818,8886,9788; `para_has_heading_or_title_style` (comparer/finalize.rs:7923) L8819,9063,9514,9598,9694,9755; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L8825,8898,8903,8907,9802,9814; `Dom::add` (xmllinq/mod.rs:1074) L8829,8849,8899,8904,8908,8930...; `Dom::name_is` (xmllinq/mod.rs:535) L8848,8911,8929,8934,9016,9568...; `Dom::new_element` (xmllinq/mod.rs:478) L8870; `Dom::attributes` (xmllinq/mod.rs:899) L8871; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L8872; `ppr_has_structural_props` (comparer/finalize.rs:574) L8879,8882,9729,9732; `Dom::replace_with` (xmllinq/mod.rs:1165) L8938; `story_tail_has_paired_original_boundary` (comparer/finalize.rs:8611) L8991; `Dom::descendants` (xmllinq/mod.rs:668) L9016,9369,9370,9371,9372,9412...; `para_word_atom_count` (comparer/finalize.rs:8013) L9123,9159,9160,9161,9181,9182...; `para_has_live_numpr` (comparer/finalize.rs:7995) L9124,9622,9632,9767,9780,9781; `para_body_text_is_whitespace_only` (comparer/finalize.rs:4200) L9157,9179,9249,9281,9411; `short_title_shares_sig_token` (comparer/finalize.rs:7949) L9191,9215; `fold_crosses_a_field` (comparer/finalize.rs:7642) L9211,9220; `para_has_omath` (comparer/finalize.rs:3515) L9250; `Dom::name` (xmllinq/mod.rs:526) L9265; `para_has_repeated_phrase` (comparer/finalize.rs:3565) L9324; `para_is_visually_blank` (comparer/finalize.rs:3528) L9355,9359; `should_fold_multi_del_at_document_scale` (comparer/finalize.rs:8025) L9395,9402; `para_body_is_digits_only` (comparer/finalize.rs:7879) L9427,9599; `para_looks_like_demo_title` (comparer/finalize.rs:7906) L9521,9554,9591; `para_body_is_very_short` (comparer/finalize.rs:7899) L9539,9719; `para_is_pure_inserted` (comparer/finalize.rs:4216) L9569; `para_has_heading_style` (comparer/finalize.rs:7936) L9693,9721; `ppr_is_jc_only` (comparer/finalize.rs:592) L9739
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.chars`, `.collect`, `.contains`, `.copied`, `.count`, `.eq_ignore_ascii_case`, `.filter`, `.find`, `.first`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some_and`, `.is_some`, `.is_whitespace`, `.iter`, `.last`, `.len`, `.map`, `.ok`, `.parse`, `.rev`, `.split_last`, `.starts_with`, `.take_while`, `.to_ascii_lowercase`, `.to_vec`, `.trim`, `.unwrap_or`, `.zip`, `Some`, `W::del`, `W::drawing`, `W::ins`, `W::move_from`, `W::move_to`, `W::name`, `W::num_pr`, `W::p_pr`, `W::p_style`, `W::p`, `W::pict`, `W::r_pr`, `W::sect_pr`, `W::spacing_el`, `W::tbl`, `W::val`
- **Macros:** `matches!`
- **Called by:** `merge_replaced_paragraphs_in` (comparer/finalize.rs:6895); `deep_production_path_tests::merge` (comparer/finalize.rs:20996); `coverage_real_revision_merge_matrix_tests::exercise` (comparer/finalize.rs:24981); `coverage_real_revision_merge_matrix_tests::live_blank_cell_tail_keeps_independent_replacement_paragraph_owners` (comparer/finalize.rs:25336); `coverage_real_revision_merge_matrix_tests::a_related_replacement_before_live_content_uses_a_paired_unmarked_pilcrow` (comparer/finalize.rs:25377); `coverage_real_revision_merge_matrix_tests::multiple_original_paragraphs_before_a_live_blank_control_tail_keep_exact_phase_payloads` (comparer/finalize.rs:25428); `coverage_real_revision_merge_matrix_tests::multiple_revised_paragraphs_with_distinct_tail_properties_keep_word_deleted_carrier` (comparer/finalize.rs:25560); `coverage_real_revision_merge_matrix_tests::multiple_original_blank_tail_checks_every_later_pilcrow_provenance` (comparer/finalize.rs:25666); `coverage_real_revision_merge_matrix_tests::multiple_original_blank_tail_preserves_first_original_section_and_property_history` (comparer/finalize.rs:25801); `coverage_real_revision_merge_matrix_tests::multiple_revised_blank_tail_pairs_only_the_matching_final_revised_paragraph` (comparer/finalize.rs:26022); `coverage_real_revision_merge_matrix_tests::a_single_replacement_before_a_live_blank_control_tail_keeps_a_paired_pilcrow` (comparer/finalize.rs:26241); `coverage_real_revision_merge_matrix_tests::a_sole_original_and_multiple_revised_paragraphs_keep_live_blank_tail_ownership` (comparer/finalize.rs:26276); +9 more

#### `interleave_list_cluster_after_coalesce`  (L9881-10070, pub)
`pub fn interleave_list_cluster_after_coalesce(dom: &mut Dom, root: NodeId)`

- **Does:** Word-alignment mode: at a multi-block replacement region — a run of fully-deleted block elements immediately followed by a run of fully- inserted ones, where either run contains a NON-paragraph block (table/ sdt) — Word emits the INSERTED block first (ole-object_ooxml-style-link: Word page 1 is the new document's head...
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L9882,9958,9959,9960,10021,10023...; `Dom::elements` (xmllinq/mod.rs:640) L9885,10045; `Dom::name_is` (xmllinq/mod.rs:535) L9888,9896,10048; `para_is_pure_inserted` (comparer/finalize.rs:4216) L9899; `para_is_pure_deleted` (comparer/finalize.rs:2900) L9900; `para_has_no_text` (comparer/finalize.rs:3500) L9905,10016; `Dom::attribute` (xmllinq/mod.rs:874) L9961,10025; `para_word_atom_count` (comparer/finalize.rs:8013) L9973; `para_revision_body_text` (comparer/finalize.rs:7859) L9988; `Dom::remove` (xmllinq/mod.rs:1118) L10050; `Dom::add_before_self` (xmllinq/mod.rs:1130) L10063; `Dom::add` (xmllinq/mod.rs:1074) L10067
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.as_bytes`, `.clone`, `.collect`, `.enumerate`, `.extend`, `.filter`, `.find`, `.first`, `.into_iter`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.map`, `.ok`, `.parse`, `.push`, `.split_whitespace`, `.to_vec`, `.unwrap_or`, `Some`, `Vec::new`, `Vec::with_capacity`, `W::body`, `W::name`, `W::num_pr`, `W::p_pr`, `W::p`, `W::sect_pr`, `W::val`
- **Called by:** `coverage_round_three_tests::nested_list_interleave_preserves_exact_node_order_and_trailing_anchors` (comparer/finalize.rs:18995); `deep_production_path_tests::nested_interleave_keeps_mark_only_items_and_defaults_invalid_terminal_levels` (comparer/finalize.rs:23143); `deep_production_path_tests::nested_interleave_uniform_tokens_are_case_insensitive_and_late_mismatches_reorder` (comparer/finalize.rs:23186); `finalizer_distinct_boundary_contract_tests::list_cluster_reorders_only_the_observed_nested_word_family` (comparer/finalize.rs:30402); `interleave_terminal_source_ownership_tests::remains_source_owned` (comparer/finalize.rs:33328); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `reorder_replaced_blocks`  (L10080-10173, pub)
`pub fn reorder_replaced_blocks(dom: &mut Dom, root: NodeId)`

- **Does:** Word-alignment mode: at a multi-block replacement region — a run of fully-deleted block elements immediately followed by a run of fully- inserted ones, where either run contains a NON-paragraph block (table/ sdt) — Word emits the INSERTED block first (ole-object_ooxml-style-link: Word page 1 is the new document's head...
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L10128; `Dom::descendants` (xmllinq/mod.rs:668) L10131; `Dom::elements` (xmllinq/mod.rs:640) L10133; `reorder_replaced_blocks::{fn block_class}` (comparer/finalize.rs:10081) L10134; `Dom::name_is` (xmllinq/mod.rs:535) L10135; `Dom::remove` (xmllinq/mod.rs:1118) L10168; `Dom::add_before_self` (xmllinq/mod.rs:1130) L10169
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.extend`, `.iter`, `.len`, `.map`, `.push`, `Some`, `Vec::new`, `W::body`, `W::name`, `W::p`
- **Called by:** `coverage_round_next_tests::block_reordering_requires_uniform_revision_rows_and_nonempty_sdt_content` (comparer/finalize.rs:20685); `consecutive_block_splice_tests::adjacent_inserted_prefix_preserves_table_midsplice_and_both_complete_projections` (comparer/finalize.rs:28433); `consecutive_block_splice_tests::live_boundary_prevents_an_earlier_insertion_from_claiming_the_splice` (comparer/finalize.rs:28471); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `reorder_replaced_blocks::{fn block_class}`  (L10081-10123)
`fn block_class(dom: &Dom, el: NodeId) -> Option<bool>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L10082; `para_replacement_class` (comparer/finalize.rs:7743) L10084; `Dom::descendants` (xmllinq/mod.rs:668) L10087; `Dom::element` (xmllinq/mod.rs:655) L10093,10094,10096,10108; `Dom::elements` (xmllinq/mod.rs:640) L10109; `reorder_replaced_blocks::{fn block_class}` (comparer/finalize.rs:10081) L10115
- **Calls (std/external/unresolved):** `.get_or_insert`, `.is_empty`, `.is_some`, `Some`, `W::del`, `W::ins`, `W::name`, `W::p`, `W::sdt_content`, `W::sdt`, `W::tbl`
- **Called by:** `reorder_replaced_blocks` (comparer/finalize.rs:10080); `reorder_replaced_blocks::{fn block_class}` (comparer/finalize.rs:10081)

#### `drop_sectpr_from_deleted_marks`  (L10184-10209, pub)
`pub fn drop_sectpr_from_deleted_marks( dom: &mut Dom, root: NodeId, genuine: &std::collections::HashSet<String>, )`

- **Does:** Word-alignment mode: drop the embedded `pPr/sectPr` from a paragraph whose paragraph mark is `w:del`-marked.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L10190; `Dom::element` (xmllinq/mod.rs:655) L10191,10193; `Dom::elements` (xmllinq/mod.rs:640) L10197; `sectpr_identity` (comparer/finalize.rs:11609) L10203; `Dom::remove` (xmllinq/mod.rs:1118) L10206
- **Calls (std/external/unresolved):** `.contains`, `.is_some_and`, `.is_some`, `Some`, `W::del`, `W::p_pr`, `W::r_pr`, `W::sect_pr`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `flatten_tracked_insertions_stamped`  (L10220-10272, pub)
`pub fn flatten_tracked_insertions_stamped(dom: &mut Dom, body: NodeId)`

- **Does:** Word-alignment mode — flatten doc A's PRE-EXISTING tracked INSERTIONS with a stamp.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L10224,10245; `Dom::parent` (xmllinq/mod.rs:597) L10228; `Dom::attribute` (xmllinq/mod.rs:874) L10233,10234,10250; `Dom::nodes` (xmllinq/mod.rs:604) L10235; `Dom::name_is` (xmllinq/mod.rs:535) L10242,10258; `Dom::ancestors` (xmllinq/mod.rs:770) L10255; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L10262,10264,10267; `Dom::replace_with` (xmllinq/mod.rs:1165) L10270
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.flat_map`, `.is_none`, `.is_some`, `.iter`, `.map`, `.reverse`, `.to_string`, `PT::name`, `Some`, `W::author`, `W::date`, `W::del`, `W::ins`, `W::r`
- **Macros:** `vec!`
- **Called by:** `finalizer_distinct_boundary_contract_tests::preexisting_insertions_keep_innermost_provenance_across_real_wrappers` (comparer/finalize.rs:29781); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `convert_stamped_preins`  (L10287-10380, pub)
`pub fn convert_stamped_preins( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** Produce-side counterpart of [`flatten_tracked_insertions_stamped`]: every run stamped `pt:PreIns` re-emerges as a PENDING INSERTION with its original author — its `w:del` wrapper (the diff saw A-only text absent from B) renames/splits to `w:ins` and delText reverts to w:t; Equal runs get a fresh `w:ins`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L10294,10321,10324; `Dom::attribute` (xmllinq/mod.rs:874) L10297,10301,10304; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L10300,10307,10308,10311,10314,10341...; `Dom::set_name` (xmllinq/mod.rs:539) L10322,10325,10340; `Dom::parent` (xmllinq/mod.rs:597) L10328; `Dom::name` (xmllinq/mod.rs:526) L10331; `Dom::elements` (xmllinq/mod.rs:640) L10337; `rev_el` (comparer/finalize.rs:126) L10347,10372; `Dom::remove` (xmllinq/mod.rs:1118) L10350,10362,10368,10376; `Dom::add` (xmllinq/mod.rs:1074) L10351,10363,10377; `Dom::add_after_self` (xmllinq/mod.rs:1147) L10352,10365; `Dom::new_element` (xmllinq/mod.rs:478) L10355; `Dom::attributes` (xmllinq/mod.rs:899) L10356; `Dom::add_before_self` (xmllinq/mod.rs:1130) L10375
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.len`, `.map`, `.position`, `.to_string`, `.to_vec`, `.unwrap_or`, `PT::name`, `Some`, `W::author`, `W::date`, `W::del_text`, `W::del`, `W::id`, `W::ins`, `W::instr_text`, `W::name`, `W::r`, `W::t`, `restamp`, `restore_text_kinds`
- **Called by:** `coverage_round_two_tests::carried_insert_and_delete_split_preserves_neighbors_and_attribution` (comparer/finalize.rs:17913); `coverage_final_batch_finalize_tests::carried_revision_matrix_preserves_order_formatting_and_original_attribution` (comparer/finalize.rs:24274); `finalizer_distinct_boundary_contract_tests::a_pending_insertion_splits_deleted_siblings_without_stealing_their_payloads` (comparer/finalize.rs:29873); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `ensure_default_page_size`  (L10393-10426, pub)
`pub fn ensure_default_page_size(dom: &mut Dom, root: NodeId)`

- **Does:** Word-alignment mode (settings-gated): Word's Compare output always carries a page setup — inputs with no `sectPr`/`pgSz` are normalized to Word's default Letter geometry (`w:pgSz w=12240 h=15840`; evidence: the `_word_redline` corpus, e.g.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L10394,10398,10406; `Dom::new_element` (xmllinq/mod.rs:478) L10401,10407; `Dom::add` (xmllinq/mod.rs:1074) L10402,10423; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L10408,10409; `Dom::elements` (xmllinq/mod.rs:640) L10413; `Dom::name` (xmllinq/mod.rs:526) L10414; `XName::local_name` (xmllinq/mod.rs:208) (by name) L10416; `Dom::add_before_self` (xmllinq/mod.rs:1130) L10422
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.is_none`, `.is_some_and`, `Some`, `W::body`, `W::name`, `W::sect_pr`
- **Macros:** `matches!`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `insert_tblpr_child_in_order`  (L10454-10470)
`fn insert_tblpr_child_in_order(dom: &mut Dom, tblpr: NodeId, child: NodeId, rank: i32)`

- **Does:** Insert `child` among `tblpr`'s children so every later sibling outranks it.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L10455; `Dom::name` (xmllinq/mod.rs:526) L10456; `XName::local_name` (xmllinq/mod.rs:208) (by name) L10460; `Dom::add_before_self` (xmllinq/mod.rs:1130) L10467; `Dom::add` (xmllinq/mod.rs:1074) L10468
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.iter`, `.map_or`, `.map`, `.unwrap_or`
- **Called by:** `align_word_table_and_comment_chrome` (comparer/finalize.rs:6220); `synthesize_table_cell_margins` (comparer/finalize.rs:11254)

#### `wml_order_elements_per_standard`  (L10488-10723, pub)
`pub fn wml_order_elements_per_standard(dom: &mut Dom, root: NodeId)`

- **Does:** Faithful port of `WordprocessingMLUtil.WmlOrderElementsPerStandard` (PtOpenXmlUtil.cs:1440), called by the C# produce path (WmlComparer.cs :1893) and previously MISSING from the port: property-container children (pPr/rPr/tblPr/tcPr/tcBorders/tblBorders/pBdr) are stable-sorted into standard schema order (unknown names...
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L10670; `Dom::name` (xmllinq/mod.rs:526) L10671; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L10672; `XName::local_name` (xmllinq/mod.rs:208) (by name) L10675; `Dom::nodes` (xmllinq/mod.rs:604) L10677,10698; `Dom::is_element` (xmllinq/mod.rs:502) L10678,10699; `Dom::remove` (xmllinq/mod.rs:1118) L10679,10693,10700,10717; `Dom::elements` (xmllinq/mod.rs:640) L10682,10703; `wml_order_elements_per_standard::{fn rank}` (comparer/finalize.rs:10492) L10686; `Dom::add` (xmllinq/mod.rs:1074) L10694,10718; `Dom::name_is` (xmllinq/mod.rs:535) L10706,10710,10715
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.chain`, `.clone`, `.collect`, `.contains`, `.enumerate`, `.into_iter`, `.is_some_and`, `.is_sorted_by_key`, `.iter`, `.map`, `.partition`, `.position`, `.sort_by_key`, `.to_string`, `W::p_pr`, `W::r_pr`
- **Called by:** `enforce_part_schema_order` (comparer/finalize.rs:10741); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `wml_order_elements_per_standard::{fn rank}`  (L10492-10658)
`fn rank(dom: &Dom, container: &str, e: NodeId) -> i32`

- **Does:** Rank tables (PtOpenXmlUtil.cs:1273-1439). _(from a code comment)_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L10493; `XName::local_name` (xmllinq/mod.rs:208) (by name) L10494; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L10495
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.map_or`
- **Called by:** `wml_order_elements_per_standard` (comparer/finalize.rs:10488); `add_child_in_rank_order` (document_comparer.rs:2089) (?)

#### `enforce_part_schema_order`  (L10741-10753, pub)
`pub fn enforce_part_schema_order(dom: &mut Dom, root: NodeId)`

- **Does:** Schema order for a whole package part, run by the package validity sweep after every other pass has had its say — so a property appended late (a style merged from B, a `w:jc` added after the body's ordering pass) still lands where the schema wants it.
- **Calls (crate):** `wml_order_elements_per_standard` (comparer/finalize.rs:10488) L10742; `Dom::descendants` (xmllinq/mod.rs:668) L10744; `Dom::elements` (xmllinq/mod.rs:640) L10745,10746; `Dom::name` (xmllinq/mod.rs:526) L10747; `Dom::remove` (xmllinq/mod.rs:1118) L10748
- **Calls (std/external/unresolved):** `.contains`, `.is_some_and`, `Some`, `W::p_pr_change`, `W::p_pr`, `W::r_pr`, `W::sect_pr`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6521)

#### `wrap_bare_del_text_runs`  (L10776-10806, pub)
`pub fn wrap_bare_del_text_runs( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** Wrap any run holding `w:delText` that is NOT directly inside a legal deletion container in a fresh `w:del` — Word shows the repair dialog on bare delText runs (THE residual strict01 trigger: `delete_text_in_opaque` / the deleted-run rebuild rename nested text-box `w:t`s to delText without wrapping the nested runs; Wor...
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L10784; `Dom::element` (xmllinq/mod.rs:655) L10788; `Dom::parent` (xmllinq/mod.rs:597) L10791; `Dom::name` (xmllinq/mod.rs:526) L10794; `rev_el` (comparer/finalize.rs:126) L10800; `Dom::add_before_self` (xmllinq/mod.rs:1130) L10801; `Dom::remove` (xmllinq/mod.rs:1118) L10802; `Dom::add` (xmllinq/mod.rs:1074) L10803; `delete_stories_of_deleted_text_boxes` (comparer/finalize.rs:10817) L10805
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.filter`, `.into_iter`, `.is_none`, `Some`, `W::del_text`, `W::del`, `W::name`, `W::r`
- **Called by:** `coverage_round_three_tests::deleted_textbox_story_marks_runs_and_pilcrows_in_schema_order` (comparer/finalize.rs:19042); `coverage_round_three_tests::nested_deleted_textboxes_use_their_own_anchor_and_preserve_existing_marks` (comparer/finalize.rs:19099); `coverage_round_three_tests::deleted_textbox_missing_attribution_uses_pinned_settings` (comparer/finalize.rs:19134); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `delete_stories_of_deleted_text_boxes`  (L10817-10904)
`fn delete_stories_of_deleted_text_boxes( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** A text box's `w:txbxContent` is a story of its own: the `w:del` around its anchor run does not reach into it.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L10828,10833,10840,10845; `Dom::name` (xmllinq/mod.rs:526) L10830,10842; `Dom::descendants` (xmllinq/mod.rs:668) L10850,10855,10890,10893,10897; `Dom::nodes_before_self` (xmllinq/mod.rs:846) L10862; `Dom::is_element` (xmllinq/mod.rs:502) L10866; `Dom::name_is` (xmllinq/mod.rs:535) L10870; `Dom::attribute` (xmllinq/mod.rs:874) L10871,10878; `rev_el` (comparer/finalize.rs:126) L10876; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L10879,10882,10891; `Dom::add_before_self` (xmllinq/mod.rs:1130) L10883; `Dom::remove` (xmllinq/mod.rs:1118) L10887; `Dom::add` (xmllinq/mod.rs:1074) L10888; `Dom::ancestors` (xmllinq/mod.rs:770) L10898; `mark_paragraph_mark_deleted` (comparer/finalize.rs:10908) L10901
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.first`, `.into_iter`, `.is_empty`, `.is_some`, `.map`, `.rev`, `PT::name`, `Some`, `W::author`, `W::date`, `W::del`, `W::ins`, `W::name`, `W::p`, `W::r`, `anchor_del`, `bare_in`
- **Called by:** `wrap_bare_del_text_runs` (comparer/finalize.rs:10776); `coverage_round_next_tests::deleted_textbox_attribution_is_inherited_per_attribute_and_runs_join_across_text` (comparer/finalize.rs:20032)

#### `mark_paragraph_mark_deleted`  (L10908-10950)
`fn mark_paragraph_mark_deleted( dom: &mut Dom, p: NodeId, like: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** Give `p`'s paragraph mark a `w:del` (attribution copied from `like`) unless the mark already carries a revision.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L10915,10923,10940; `Dom::new_element` (xmllinq/mod.rs:478) L10918,10926; `Dom::add_first` (xmllinq/mod.rs:1101) L10919,10949; `Dom::elements` (xmllinq/mod.rs:640) L10928; `Dom::name_is` (xmllinq/mod.rs:535) L10931; `Dom::add_before_self` (xmllinq/mod.rs:1130) L10933; `Dom::add` (xmllinq/mod.rs:1074) L10934; `rev_el` (comparer/finalize.rs:126) L10943; `Dom::attribute` (xmllinq/mod.rs:874) L10945; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L10946
- **Calls (std/external/unresolved):** `.any`, `.find`, `.into_iter`, `.is_some`, `.iter`, `.map`, `Some`, `W::author`, `W::date`, `W::del`, `W::ins`, `W::name`, `W::p_pr_change`, `W::p_pr`, `W::r_pr`, `W::sect_pr`
- **Called by:** `delete_stories_of_deleted_text_boxes` (comparer/finalize.rs:10817); `coverage_round_next_tests::paragraph_deletion_mark_inserts_before_each_schema_tail_and_preserves_format` (comparer/finalize.rs:20064)

#### `pending_deletion_texts`  (L10981-10990, pub)
`pub fn pending_deletion_texts(dom: &Dom, body: NodeId) -> std::collections::HashSet<String>`

- **Does:** Collect the concatenated visible text of every `w:del` wrapper under `body` (both `w:delText` and `w:delInstrText`).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L10983; `pending_deletion_fingerprint` (comparer/finalize.rs:10999) L10984
- **Calls (std/external/unresolved):** `.insert`, `.is_empty`, `Some`, `W::del`, `std::collections::HashSet::new`
- **Called by:** `coverage_round_next_tests::deleted_history_flattening_uses_both_field_and_text_fingerprint` (comparer/finalize.rs:20346); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `pending_deletion_fingerprint`  (L10999-11010)
`fn pending_deletion_fingerprint(dom: &Dom, del: NodeId) -> String`

- **Does:** Fingerprint of one `w:del` wrapper's pending-deletion text, gathering BOTH `w:delText` (run text) and `w:delInstrText` (field-instruction text).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L11002; `Dom::name` (xmllinq/mod.rs:526) L11005; `Dom::value` (xmllinq/mod.rs:1185) L11008
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_some_and`, `.map`, `W::del_text`, `W::name`
- **Called by:** `pending_deletion_texts` (comparer/finalize.rs:10981); `flatten_tracked_deletions` (comparer/finalize.rs:11035)

#### `flatten_tracked_deletions`  (L11035-11123, pub)
`pub fn flatten_tracked_deletions( dom: &mut Dom, body: NodeId, side: FlattenSide, other_side_pending: Option<&std::collections::HashSet<String>>, )`

- **Does:** Word-alignment mode: flatten pre-existing tracked deletions before diffing — unwrap `w:del`, convert `w:delText`/`w:delInstrText` back to `w:t`/`w:instrText` — so the old text re-enters the diff and comes out marked deleted: VISIBLE (struck through) like Word's redline, instead of vanishing via accept-before-diff.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L11045,11059,11087,11090; `Dom::element` (xmllinq/mod.rs:655) L11046,11047; `Dom::remove` (xmllinq/mod.rs:1118) L11049; `Dom::parent` (xmllinq/mod.rs:597) L11063; `Dom::name_is` (xmllinq/mod.rs:535) L11065,11073,11111; `Dom::nodes` (xmllinq/mod.rs:604) L11069; `Dom::is_element` (xmllinq/mod.rs:502) L11073; `pending_deletion_fingerprint` (comparer/finalize.rs:10999) L11086; `Dom::set_name` (xmllinq/mod.rs:539) L11088,11091; `Dom::attribute` (xmllinq/mod.rs:874) L11093,11094; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L11112,11114,11117; `Dom::replace_with` (xmllinq/mod.rs:1165) L11121
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.contains`, `.is_some_and`, `.iter`, `.map`, `.to_string`, `PT::name`, `Some`, `W::author`, `W::date`, `W::del_text`, `W::del`, `W::instr_text`, `W::name`, `W::p_pr`, `W::r_pr`, `W::r`, `W::t`
- **Called by:** `coverage_round_next_tests::deleted_history_flattening_uses_both_field_and_text_fingerprint` (comparer/finalize.rs:20346); `coverage_round_next_tests::revised_complex_deletion_stays_pending_while_original_restores_nested_text` (comparer/finalize.rs:20652); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `convert_stamped_predeletes`  (L11141-11232, pub)
`pub fn convert_stamped_predeletes( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** Word-alignment mode, produce-side counterpart of the doc-B stamp in [`flatten_tracked_deletions`]: every run stamped `pt:PreDelete` re-emerges as a PENDING DELETION — its `w:ins` wrapper (the diff saw B-only text) becomes `w:del`, Equal runs get a fresh `w:del` — so B's pre-existing deletions render struck through lik...
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L11148,11225,11228; `Dom::attribute` (xmllinq/mod.rs:874) L11151,11155,11158; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L11154,11161,11162,11168,11171,11204...; `Dom::parent` (xmllinq/mod.rs:597) L11174; `Dom::name` (xmllinq/mod.rs:526) L11177; `Dom::elements` (xmllinq/mod.rs:640) L11181,11187; `Dom::set_name` (xmllinq/mod.rs:539) L11189,11226,11229; `rev_el` (comparer/finalize.rs:126) L11195,11219; `Dom::remove` (xmllinq/mod.rs:1118) L11197,11209,11215,11222; `Dom::add` (xmllinq/mod.rs:1074) L11198,11210,11223; `Dom::add_after_self` (xmllinq/mod.rs:1147) L11199,11212; `Dom::new_element` (xmllinq/mod.rs:478) L11202; `Dom::attributes` (xmllinq/mod.rs:899) L11203; `Dom::add_before_self` (xmllinq/mod.rs:1130) L11221
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.len`, `.map`, `.position`, `.to_string`, `.to_vec`, `.unwrap_or`, `PT::name`, `Some`, `W::author`, `W::date`, `W::del_text`, `W::del`, `W::id`, `W::ins`, `W::instr_text`, `W::name`, `W::r`, `W::t`, `restamp`
- **Called by:** `coverage_round_two_tests::carried_insert_and_delete_split_preserves_neighbors_and_attribution` (comparer/finalize.rs:17913); `coverage_round_next_tests::predelete_conversion_missing_attribution_defaults_without_changing_neighbor_runs` (comparer/finalize.rs:20397); `coverage_final_batch_finalize_tests::carried_revision_matrix_preserves_order_formatting_and_original_attribution` (comparer/finalize.rs:24274); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `synthesize_table_cell_margins`  (L11254-11304, pub)
`pub fn synthesize_table_cell_margins( dom: &mut Dom, root: NodeId, default_table_styles: (bool, bool), )`

- **Does:** Word-alignment mode: Word's Compare synthesizes near-zero cell margins on the tables it emits — `w:tblInd w=10` and `w:tblCellMar left/right w=10` — present in NEITHER input nor Word's TableNormal default of 108 (forensics pair #5, helvetica_hr-onboarding).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L11265; `Dom::element` (xmllinq/mod.rs:655) L11267,11273,11279,11280,11291,11295; `Dom::elements` (xmllinq/mod.rs:640) L11276; `synthesize_table_cell_margins::{fn dxa}` (comparer/finalize.rs:11259) L11292,11297,11299; `insert_tblpr_child_in_order` (comparer/finalize.rs:10454) L11293,11301; `Dom::new_element` (xmllinq/mod.rs:478) L11296; `Dom::add` (xmllinq/mod.rs:1074) L11298,11300
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `Some`, `W::del`, `W::name`, `W::tbl`, `W::tr_pr`, `W::tr`
- **Called by:** `coverage_round_three_tests::bordered_table_geometry_uses_row_revision_source_and_schema_order` (comparer/finalize.rs:19156); `coverage_round_three_tests::table_geometry_preserves_explicit_values_and_fills_only_missing_properties` (comparer/finalize.rs:19203); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `synthesize_table_cell_margins::{fn dxa}`  (L11259-11264)
`fn dxa(dom: &mut Dom, name: &str, w: &str) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L11260; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L11261,11262
- **Calls (std/external/unresolved):** `Some`, `W::name`
- **Called by:** `synthesize_table_cell_margins` (comparer/finalize.rs:11254)

#### `fix_strict_validity_artifacts`  (L11326-11475, pub)
`pub fn fix_strict_validity_artifacts(dom: &mut Dom, root: NodeId)`

- **Does:** Word-validity artifacts beyond schema ordering (OpenXmlValidator-driven; Word's own redlines validate clean of all of these): - `w:cnfStyle` without the REQUIRED `w:val`: Strict expresses the flags as individual boolean attributes; synthesize the Transitional 12-bit bitmask (firstRow lastRow firstColumn lastColumn odd...
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L11341,11358,11410,11428,11436; `Dom::attribute` (xmllinq/mod.rs:874) L11342,11374,11412,11437; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L11350; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L11355,11378,11438,11439,11472; `Dom::value` (xmllinq/mod.rs:1185) L11359; `Dom::set_value` (xmllinq/mod.rs:1219) L11364; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L11368,11456; `XNamespace::none` (xmllinq/mod.rs:116) L11393; `XNamespace::get` (xmllinq/mod.rs:108) L11418; `fix_strict_validity_artifacts::{fn remap}` (comparer/finalize.rs:11395) L11419; `Dom::set_name` (xmllinq/mod.rs:539) L11429; `Dom::name` (xmllinq/mod.rs:526) L11457; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L11458; `Dom::attributes` (xmllinq/mod.rs:899) L11461
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter_map`, `.into_iter`, `.is_some_and`, `.is_some`, `.map_or`, `.map`, `.name`, `.ok`, `.parse`, `.push`, `.round`, `.starts_with`, `.strip_suffix`, `.to_string`, `.trim`, `Some`, `String::with_capacity`, `W14::name`, `W::drawing`, `W::name`, `W::val`, `WP14::name`, `crate::namespaces::A::name`, `crate::namespaces::WNE::name`, `u32::from_str_radix`
- **Macros:** `matches!`
- **Called by:** `coverage_round_next_tests::strict_shape_repair_stops_at_nested_drawing_and_keeps_bad_percent_text` (comparer/finalize.rs:20667); `compare_documents_impl` (document_comparer.rs:6521)

#### `fix_strict_validity_artifacts::{fn remap}`  (L11395-11409)
`fn remap(dom: &mut Dom, node: NodeId, wp_uri: &str, target: &XNamespace, drawing: &XName)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L11396; `Dom::is_element` (xmllinq/mod.rs:502) L11397; `Dom::name` (xmllinq/mod.rs:526) L11400; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L11404; `Dom::set_name` (xmllinq/mod.rs:539) L11405; `XNamespace::name` (xmllinq/mod.rs:134) L11405; `XName::local_name` (xmllinq/mod.rs:208) (by name) L11405; `fix_strict_validity_artifacts::{fn remap}` (comparer/finalize.rs:11395) L11407
- **Called by:** `fix_strict_validity_artifacts` (comparer/finalize.rs:11326); `fix_strict_validity_artifacts::{fn remap}` (comparer/finalize.rs:11395)

#### `normalize_universal_measures`  (L11490-11492, pub)
`pub fn normalize_universal_measures(dom: &mut Dom, root: NodeId)`

- **Does:** Canonicalize ST_UniversalMeasure values (`612pt`, `1in`, `2.54cm`, …) to plain twips on the twips-typed layout attributes.
- **Calls (crate):** `normalize_measures` (comparer/finalize.rs:11502) L11491
- **Called by:** `closing_mark_takes_revised_properties` (comparer/finalize.rs:3200); `sectpr_identity` (comparer/finalize.rs:11609); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215); `compare_documents_impl` (document_comparer.rs:6521)

#### `normalize_unit_measures`  (L11498-11500, pub)
`pub fn normalize_unit_measures(dom: &mut Dom, root: NodeId)`

- **Does:** [`normalize_universal_measures`] without the fractional-plain-value rounding: only unit-bearing spellings ("612pt") are rewritten.
- **Calls (crate):** `normalize_measures` (comparer/finalize.rs:11502) L11499
- **Called by:** `parse` (debug/render.rs:104)

#### `normalize_measures`  (L11502-11588)
`fn normalize_measures(dom: &mut Dom, root: NodeId, round_plain_fractional: bool)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L11564; `Dom::name` (xmllinq/mod.rs:526) L11565; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L11566; `XName::local_name` (xmllinq/mod.rs:208) (by name) L11571; `Dom::attribute` (xmllinq/mod.rs:874) L11580; `normalize_measures::{fn to_twips}` (comparer/finalize.rs:11548) L11582; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L11584
- **Calls (std/external/unresolved):** `.and_then`, `.find`, `.iter`, `.map`, `.to_string`, `Some`, `W::name`
- **Called by:** `normalize_universal_measures` (comparer/finalize.rs:11490); `normalize_unit_measures` (comparer/finalize.rs:11498)

#### `normalize_measures::{fn to_twips}`  (L11548-11563)
`fn to_twips(v: &str, round_plain_fractional: bool) -> Option<i64>`

- **Does:** _no doc comment_
- **Calls (crate):** `universal_measure_twips` (util/mod.rs:61) L11549
- **Calls (std/external/unresolved):** `.contains`, `.map`, `.ok`, `.parse`, `.round`, `.trim`, `Some`
- **Called by:** `normalize_measures` (comparer/finalize.rs:11502)

#### `sectpr_identity`  (L11609-11625, pub)
`pub fn sectpr_identity(dom: &mut Dom, sp: NodeId) -> String`

- **Does:** Serialize a sectPr with every `pt:*` bookkeeping attribute AND every `rsid*` attribute stripped, and measurement units canonicalized to twips, so the identity comparison in [`drop_hoisted_sectpr_artifacts`] is immune to what the pipeline changes between capture and check: unid stamping adds pt attrs, the pre-diff acce...
- **Calls (crate):** `Dom::clone_subtree` (xmllinq/mod.rs:1231) L11610; `remove_powertools_scratch_markup` (comparer/finalize.rs:1060) L11611; `normalize_universal_measures` (comparer/finalize.rs:11490) L11612; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L11613; `Dom::attributes` (xmllinq/mod.rs:899) L11614; `XName::local_name` (xmllinq/mod.rs:208) (by name) L11618; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L11621; `Dom::serialize_element` (xmllinq/mod.rs:1315) L11624
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.map`, `.starts_with`
- **Called by:** `closing_mark_takes_revised_properties` (comparer/finalize.rs:3200); `drop_sectpr_from_deleted_marks` (comparer/finalize.rs:10184); `drop_hoisted_sectpr_artifacts` (comparer/finalize.rs:11642); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `drop_hoisted_sectpr_artifacts`  (L11642-11655, pub)
`pub fn drop_hoisted_sectpr_artifacts( dom: &mut Dom, root: NodeId, genuine: &std::collections::HashSet<String>, )`

- **Does:** Word-alignment mode (settings-gated): drop pPr-embedded `sectPr`s that were NOT pPr-embedded in either input ("genuine" mid-section breaks are captured from the inputs before atomize hoists each doc's final body sectPr into its last paragraph — that hoist artifact must not become a mid-document page break; evidence: 1...
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L11648; `Dom::elements` (xmllinq/mod.rs:640) L11649; `sectpr_identity` (comparer/finalize.rs:11609) L11650; `Dom::remove` (xmllinq/mod.rs:1118) L11651
- **Calls (std/external/unresolved):** `.contains`, `Some`, `W::p_pr`, `W::sect_pr`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `mark_fully_revised_rows`  (L11671-11785, pub)
`pub fn mark_fully_revised_rows( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** Word-alignment mode (settings-gated): Word marks a fully-deleted table row with `w:del` inside `w:trPr` (and fully-inserted rows with `w:ins`), in addition to the cell-content revisions — the accept pipeline then removes the row (and the whole table once every row is marked).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L11679,11685,11709,11769; `Dom::elements` (xmllinq/mod.rs:640) L11684,11705,11766; `para_replacement_class_with_moves` (comparer/finalize.rs:7747) L11686; `Dom::name_is` (xmllinq/mod.rs:535) L11706; `Dom::value` (xmllinq/mod.rs:1185) L11707; `Dom::name` (xmllinq/mod.rs:526) L11711; `para_mark_revision` (comparer/finalize.rs:7631) L11726,11759,11761,11762; `Dom::element` (xmllinq/mod.rs:655) L11738,11751,11753,11775; `Dom::new_element` (xmllinq/mod.rs:478) L11741,11778; `Dom::add_first` (xmllinq/mod.rs:1101) L11742; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L11779,11780,11782; `add_row_mark` (comparer/lcs_table.rs:778) L11783
- **Calls (std/external/unresolved):** `.all`, `.any`, `.chain`, `.contains`, `.flat_map`, `.get_or_insert`, `.into_iter`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.to_string`, `.trim`, `MC::name`, `Some`, `W::author`, `W::date`, `W::del`, `W::id`, `W::ins`, `W::move_from`, `W::move_to`, `W::name`, `W::object`, `W::p_pr`, `W::p`, `W::pict`, `std::iter::once`
- **Called by:** `coverage_round_three_tests::revision_rows_preserve_cell_geometry_and_are_stamped_once` (comparer/finalize.rs:19285); `row_move_ownership_regressions::moving_cell_content_without_paragraph_ownership_keeps_existing_rows` (comparer/finalize.rs:23552); `row_move_ownership_regressions::moved_content_with_matching_paragraph_mark_tracks_row_lifecycle` (comparer/finalize.rs:23596); `row_nontext_ownership_regressions::unchanged_nontext_payload_prevents_a_moved_neighbor_from_owning_its_row` (comparer/finalize.rs:23826); `paired_row_mark_ownership_tests::paired_rows_with_surviving_marks_keep_empty_revised_cells_and_full_original_history` (comparer/finalize.rs:28917); `paired_row_mark_ownership_tests::unchanged_empty_pilcrows_keep_row_ownership_without_row_property_history` (comparer/finalize.rs:28958); `paired_row_mark_ownership_tests::new_empty_rows_with_owned_pilcrows_still_receive_their_row_lifetime` (comparer/finalize.rs:29008); `paired_row_mark_ownership_tests::explicit_row_lifetime_is_not_overridden_by_a_surviving_cell_mark` (comparer/finalize.rs:29092); `paired_row_mark_ownership_tests::explicit_row_lifetime_preserves_opposite_content_revisions_and_property_history` (comparer/finalize.rs:29105); `live_opaque_row_source_boundary_tests::unchanged_opaque_cell_owners_prevent_row_lifetime_synthesis_without_text_caches` (comparer/finalize.rs:34697); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `splice_trailing_short_pure_dels_midstream`  (L11803-11945, pub)
`pub fn splice_trailing_short_pure_dels_midstream(dom: &mut Dom, root: NodeId)`

- **Does:** Whole-doc short-base × long-next: insert-all-next + delete-all-base trails pure-D.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L11804,11923,11925,11926; `Dom::elements` (xmllinq/mod.rs:640) L11807,11920; `Dom::name_is` (xmllinq/mod.rs:535) L11810,11820,11836,11838,11853,11898...; `para_is_pure_deleted` (comparer/finalize.rs:2900) L11820,11886,11899; `para_is_pure_inserted` (comparer/finalize.rs:4216) L11837; `Dom::descendants` (xmllinq/mod.rs:668) L11839,11840,11857; `Dom::value_str` (xmllinq/mod.rs:1195) L11858; `para_body_alnum_len` (comparer/finalize.rs:7889) L11889,11900; `para_replacement_class` (comparer/finalize.rs:7743) L11915; `Dom::remove` (xmllinq/mod.rs:1118) L11938; `Dom::add_after_self` (xmllinq/mod.rs:1147) L11942
- **Calls (std/external/unresolved):** `.all`, `.any`, `.chars`, `.collect`, `.contains`, `.count`, `.enumerate`, `.filter`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.or`, `.push_str`, `.starts_with`, `.to_ascii_lowercase`, `.to_vec`, `.trim_start`, `Some`, `String::new`, `W::body`, `W::del`, `W::ins`, `W::name`, `W::p`, `W::sect_pr`, `W::t`, `W::tbl`, `W::tr`, `disappears_on_rejection`
- **Macros:** `vec!`
- **Called by:** `deep_production_path_tests::short_residual_splice_prefers_tip_then_toc_and_moves_only_the_short_numbered_title` (comparer/finalize.rs:22426); `deep_production_path_tests::short_title_splice_crosses_only_tables_whose_every_row_is_structurally_inserted` (comparer/finalize.rs:22464); `deep_production_path_tests::short_residual_splice_majority_and_length_boundaries_do_not_invent_reordering` (comparer/finalize.rs:22531); `deep_production_path_tests::late_tip_overrides_earlier_toc_and_numbered_splice_anchors` (comparer/finalize.rs:23393); `finalizer_remaining_source_boundary_tests::midstream_deleted_blocks_cross_only_payload_that_disappears_on_rejection` (comparer/finalize.rs:31053); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `split_digits_ins_from_mixed_title`  (L11959-12029, pub)
`pub fn split_digits_ins_from_mixed_title(dom: &mut Dom, root: NodeId)`

- **Does:** M147 — MIX that is **only** leading pure-ins digits ("24") + pure-del demo title body (1_5_line_spacing×24): Word keeps pure-I "24" then pure-D title as separate paragraphs.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L11960; `Dom::elements` (xmllinq/mod.rs:640) L11963,11977; `Dom::name_is` (xmllinq/mod.rs:535) L11966,11969,11982,11985,12012,12015; `Dom::descendants` (xmllinq/mod.rs:668) L11972,11973,11998,12012,12015; `Dom::value_str` (xmllinq/mod.rs:1195) L11999; `Dom::new_element` (xmllinq/mod.rs:478) L12020; `Dom::parent` (xmllinq/mod.rs:597) L12022; `Dom::remove` (xmllinq/mod.rs:1118) L12023; `Dom::add` (xmllinq/mod.rs:1074) L12024; `Dom::add_before_self` (xmllinq/mod.rs:1130) L12027
- **Calls (std/external/unresolved):** `.all`, `.any`, `.chars`, `.collect`, `.filter`, `.into_iter`, `.is_ascii_digit`, `.is_empty`, `.is_some`, `.is_whitespace`, `.iter`, `.push_str`, `.push`, `.trim`, `Some`, `String::new`, `Vec::new`, `W::body`, `W::del`, `W::ins`, `W::p_pr`, `W::p`, `W::sect_pr`, `W::t`
- **Called by:** `coverage_round_next_tests::digit_title_split_moves_all_leading_fragments_and_keeps_original_properties` (comparer/finalize.rs:19434); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `peel_trailing_ins_from_mix_into_following_pure_del`  (L12046-12171, pub)
`pub fn peel_trailing_ins_from_mix_into_following_pure_del(dom: &mut Dom, root: NodeId)`

- **Does:** M144 — MIX with trailing pure-ins + following pure-D body: when the trailing insert and pure-D share a connector/content word (italic×justified: both have `for`), peel the trailing ins into the pure-D as a mixed para via simple word interleave (Word: del prefix + Equal` for ` + ins phrase + del rest).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L12047; `Dom::elements` (xmllinq/mod.rs:640) L12051,12085,12149; `Dom::name_is` (xmllinq/mod.rs:535) L12054,12060,12089,12092,12098,12152; `para_is_pure_deleted` (comparer/finalize.rs:2900) L12063; `Dom::descendants` (xmllinq/mod.rs:668) L12066,12067,12100,12123; `para_mark_revision` (comparer/finalize.rs:7631) L12068; `para_revision_body_text` (comparer/finalize.rs:7859) L12073; `Dom::value_str` (xmllinq/mod.rs:1195) L12101,12124; `Dom::name` (xmllinq/mod.rs:526) L12115; `Dom::parent` (xmllinq/mod.rs:597) L12154; `Dom::remove` (xmllinq/mod.rs:1118) L12157; `Dom::add_before_self` (xmllinq/mod.rs:1130) L12159; `Dom::add` (xmllinq/mod.rs:1074) L12161
- **Calls (std/external/unresolved):** `.all`, `.any`, `.as_str`, `.chars`, `.collect`, `.contains`, `.count`, `.filter`, `.find`, `.into_iter`, `.is_alphanumeric`, `.is_empty`, `.is_none`, `.iter`, `.len`, `.map`, `.push_str`, `.push`, `.rev`, `.reverse`, `.saturating_sub`, `.split`, `.to_ascii_lowercase`, `Some`, `String::new`, `Vec::new`, `W::body`, `W::del`, `W::ins`, `W::p_pr`, `W::p`, `W::r`, `W::sect_pr`, `W::t`
- **Called by:** `coverage_round_next_tests::trailing_insertion_peel_has_inclusive_token_bounds_and_keeps_punctuation` (comparer/finalize.rs:19485); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `restore_short_del_before_long_ins`  (L12185-12254, pub)
`pub fn restore_short_del_before_long_ins(dom: &mut Dom, root: NodeId)`

- **Does:** M159 (text_highlight×times): after merge reorders unrelated 1v1 D-then-I to I-before-D, restore Word order when pure-I is longer than a short pure-D and they sit after a MIX title: `… MIX | INS | DEL | MIX …` → `… MIX | DEL | INS | MIX …`.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L12186; `Dom::elements` (xmllinq/mod.rs:640) L12190; `Dom::name_is` (xmllinq/mod.rs:535) L12193,12199,12213; `para_is_pure_inserted` (comparer/finalize.rs:4216) L12202; `para_is_pure_deleted` (comparer/finalize.rs:2900) L12202; `Dom::descendants` (xmllinq/mod.rs:668) L12217,12219; `para_mark_revision` (comparer/finalize.rs:7631) L12218,12220; `para_revision_body_text` (comparer/finalize.rs:7859) L12228; `para_body_alnum_len` (comparer/finalize.rs:7889) L12236,12237; `Dom::parent` (xmllinq/mod.rs:597) L12242; `Dom::remove` (xmllinq/mod.rs:1118) L12245; `Dom::add_before_self` (xmllinq/mod.rs:1130) L12246
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter`, `.into_iter`, `.is_empty`, `.is_none`, `.len`, `.saturating_sub`, `.to_ascii_lowercase`, `Some`, `W::body`, `W::del`, `W::ins`, `W::p`, `W::sect_pr`
- **Called by:** `coverage_round_next_tests::short_deletion_restore_checks_filename_and_alnum_length_boundaries` (comparer/finalize.rs:19657); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `peel_trailing_del_from_mix_into_following_pure_ins`  (L12266-12475, pub)
`pub fn peel_trailing_del_from_mix_into_following_pure_ins(dom: &mut Dom, root: NodeId)`

- **Does:** M154 (justified_underline×justify_2): trailing `w:del` on a MIX followed by pure-I body — peel the del into the pure-I so both become MIX.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L12267,12350,12434,12436,12437,12442...; `Dom::elements` (xmllinq/mod.rs:640) L12271,12308; `Dom::name_is` (xmllinq/mod.rs:535) L12274,12280,12312,12315,12320; `para_is_pure_inserted` (comparer/finalize.rs:4216) L12283; `Dom::descendants` (xmllinq/mod.rs:668) L12286,12288,12322,12371,12374; `para_mark_revision` (comparer/finalize.rs:7631) L12287,12301; `peel_has_text_only_body` (comparer/finalize.rs:12478) L12292; `final_peel_has_identical_closing_properties` (comparer/finalize.rs:12496) L12304; `Dom::value_str` (xmllinq/mod.rs:1195) L12323,12372,12375; `trailing_equal_period` (comparer/finalize.rs:12880) L12330,12341; `Dom::parent` (xmllinq/mod.rs:597) L12330,12349,12462; `text_leaves` (comparer/finalize.rs:12903) L12344; `Dom::serialize_element` (xmllinq/mod.rs:1315) L12351; `run_properties` (markdown/xml.rs:2175) (by name) L12357; `Dom::name` (xmllinq/mod.rs:526) L12363; `para_revision_body_text` (comparer/finalize.rs:7859) L12387; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L12439; `max_numeric_id` (comparer/finalize.rs:14923) L12440; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L12441; `Dom::new_element` (xmllinq/mod.rs:478) L12445,12453; `Dom::add_first` (xmllinq/mod.rs:1101) L12446,12458; `Dom::add` (xmllinq/mod.rs:1074) L12454,12466; `Dom::remove` (xmllinq/mod.rs:1118) L12465
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_str`, `.chars`, `.collect`, `.contains`, `.copied`, `.count`, `.expect`, `.filter`, `.first`, `.into_iter`, `.is_alphanumeric`, `.is_empty`, `.is_none`, `.is_some_and`, `.iter`, `.last`, `.len`, `.map`, `.push_str`, `.push`, `.rev`, `.reverse`, `.saturating_add`, `.saturating_sub`, `.split`, `.to_ascii_lowercase`, `.to_string`, `.unwrap_or_default`, `Some`, `String::new`, `Vec::new`, `W::body`, `W::del_text`, `W::del`, `W::id`, `W::ins`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `W::sect_pr`, `W::t`
- **Called by:** `coverage_round_next_tests::trailing_deletion_peel_truth_table_for_shared_long_and_copula_paths` (comparer/finalize.rs:19555); `deep_production_path_tests::deletion_peel_keeps_retained_source_boundary_and_inserted_continuations` (comparer/finalize.rs:22638); `deep_production_path_tests::final_deletion_peel_transfers_boundary_only_with_equal_properties_and_text_payload` (comparer/finalize.rs:22704); `deep_production_path_tests::deletion_peel_keeps_nontext_only_original_anchors_in_their_source_paragraph` (comparer/finalize.rs:22795); `deep_production_path_tests::deletion_peel_keeps_punctuation_and_whitespace_only_original_anchors` (comparer/finalize.rs:22854); `deep_production_path_tests::deletion_peel_keeps_original_closing_boundary_when_all_original_words_are_deleted` (comparer/finalize.rs:22901); `deep_production_path_tests::wholesale_deleted_body_peel_transfers_its_retained_closing_boundary` (comparer/finalize.rs:22946); `deep_production_path_tests::deletion_peel_does_not_move_an_original_suffix_past_equal_nonperiod_text` (comparer/finalize.rs:22999); `deep_production_path_tests::borrowed_period_peel_keeps_the_original_period_run_formatting` (comparer/finalize.rs:23046); `deep_production_path_tests::same_format_borrowed_period_peel_keeps_literal_text_and_character_properties` (comparer/finalize.rs:23083); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `peel_has_text_only_body`  (L12478-12491)
`fn peel_has_text_only_body(dom: &Dom, node: NodeId) -> bool`

- **Does:** Keep nontext payload under its original paragraph ownership during M154.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L12479; `Dom::name` (xmllinq/mod.rs:526) L12482; `Dom::elements` (xmllinq/mod.rs:640) L12488; `peel_has_text_only_body` (comparer/finalize.rs:12478) L12490
- **Calls (std/external/unresolved):** `.all`, `.contains`, `.into_iter`, `W::del_text`, `W::del`, `W::ins`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `W::t`
- **Called by:** `peel_trailing_del_from_mix_into_following_pure_ins` (comparer/finalize.rs:12266); `peel_has_text_only_body` (comparer/finalize.rs:12478); `final_peel_has_identical_closing_properties` (comparer/finalize.rs:12496)

#### `final_peel_has_identical_closing_properties`  (L12496-12552)
`fn final_peel_has_identical_closing_properties( dom: &mut Dom, mix: NodeId, inserted: NodeId, ) -> bool`

- **Does:** Only the final inserted carrier can take a source continuation without losing its paragraph formatting.
- **Calls (crate):** `peel_has_text_only_body` (comparer/finalize.rs:12478) L12501; `Dom::element` (xmllinq/mod.rs:655) L12504,12505,12509,12510,12514,12529...; `Dom::elements` (xmllinq/mod.rs:640) L12515,12542,12545; `Dom::name_is` (xmllinq/mod.rs:535) L12523; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L12537,12539; `Dom::new_element` (xmllinq/mod.rs:478) L12538; `Dom::remove` (xmllinq/mod.rs:1118) L12543,12546; `remove_powertools_scratch_markup` (comparer/finalize.rs:1060) L12549; `Dom::serialize_element` (xmllinq/mod.rs:1315) L12551
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.chain`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.map`, `.unwrap_or_else`, `Some`, `W::del`, `W::ins`, `W::move_from`, `W::name`, `W::p_pr_change`, `W::p_pr`, `W::r_pr`, `W::sect_pr`
- **Called by:** `peel_trailing_del_from_mix_into_following_pure_ins` (comparer/finalize.rs:12266)

#### `capture_deleted_demo_title_properties`  (L12584-12599, pub(crate))
`pub(crate) fn capture_deleted_demo_title_properties( dom: &mut Dom, root: NodeId, ) -> std::collections::HashMap<NodeId, NodeId>`

- **Does:** Snapshot every pure-deleted paragraph that looks like a "Demo" title's `w:pPr` BEFORE [`strip_redundant_demo_default_spacing`] runs, so M143's fold can re-attach authored Heading properties inside `w:pPrChange`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L12588; `para_is_pure_deleted` (comparer/finalize.rs:2900) L12591; `para_looks_like_demo_title` (comparer/finalize.rs:7906) L12592; `Dom::element` (xmllinq/mod.rs:655) L12593; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L12595
- **Calls (std/external/unresolved):** `.insert`, `Some`, `W::p_pr`, `W::p`, `std::collections::HashMap::new`
- **Called by:** `demo_title_pilcrow_history_tests::authored_title_snapshot_survives_demo_default_layout_normalization` (comparer/finalize.rs:28820); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `fold_midstream_demo_title_into_numbered_heading_with_settings`  (L12602-12615, pub)
`pub fn fold_midstream_demo_title_into_numbered_heading_with_settings( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** M143 with explicit settings and id-gen — production entry point.
- **Calls (crate):** `fold_midstream_demo_title_into_numbered_heading_with_source_properties` (comparer/finalize.rs:12624) L12608
- **Calls (std/external/unresolved):** `std::collections::HashMap::new`
- **Called by:** `fold_midstream_demo_title_into_numbered_heading` (comparer/finalize.rs:12567); `coverage_round_next_tests::demo_title_fold_history_preserves_complete_old_layout_and_current_attribution` (comparer/finalize.rs:19733); `demo_title_pilcrow_history_tests::title_fold_retains_complete_pilcrow_formats_and_configured_history` (comparer/finalize.rs:28552); `demo_title_pilcrow_history_tests::nested_mark_effect_changes_keep_both_source_projections_and_disabled_tracking` (comparer/finalize.rs:28667); `demo_title_pilcrow_history_tests::title_fold_preserves_existing_mark_history_and_section_break_ownership` (comparer/finalize.rs:28787); `fragmented_source_label_boundary_tests::midstream_heading_search_preserves_controls_live_anchors_and_identical_property_owners` (comparer/finalize.rs:33926)

#### `fold_midstream_demo_title_into_numbered_heading_with_source_properties`  (L12624-12826, pub)
`pub fn fold_midstream_demo_title_into_numbered_heading_with_source_properties( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, source_properties: &std::collections::HashMap<NodeId, NodeId>, )`

- **Does:** M143 fold that re-attaches captured authored Heading properties inside `w:pPrChange` so the deleted-title's formatting history survives the merge.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L12631,12726,12727,12732,12733,12735...; `Dom::elements` (xmllinq/mod.rs:640) L12634,12774,12815,12819; `Dom::name_is` (xmllinq/mod.rs:535) L12637,12647,12659,12668,12692,12695...; `Dom::descendants` (xmllinq/mod.rs:668) L12648,12649,12653,12654,12703; `Dom::value_str` (xmllinq/mod.rs:1195) L12652,12655,12704; `Dom::ancestors_and_self` (xmllinq/mod.rs:787) L12656; `para_is_pure_deleted` (comparer/finalize.rs:2900) L12668; `para_looks_like_demo_title` (comparer/finalize.rs:7906) L12671; `Dom::name` (xmllinq/mod.rs:526) L12674; `para_has_no_text` (comparer/finalize.rs:3500) L12675; `Dom::parent` (xmllinq/mod.rs:597) L12685,12718; `para_is_pure_inserted` (comparer/finalize.rs:4216) L12698; `fold_midstream_demo_title_into_numbered_heading_with_source_properties::{fn clean_mark_properties}` (comparer/finalize.rs:12740) L12749,12750; `paragraph_mark_properties_equal` (comparer/finalize.rs:3322) L12754; `project_para_properties_for_change` (comparer/formatchg.rs:329) L12764; `Dom::new_element` (xmllinq/mod.rs:478) L12765,12767,12791; `Dom::add_first` (xmllinq/mod.rs:1101) L12768,12792; `normalize_para_properties` (comparer/formatchg.rs:349) L12771,12772; `Dom::remove` (xmllinq/mod.rs:1118) L12775,12796,12816,12824; `rev_el` (comparer/finalize.rs:126) L12777; `Dom::add` (xmllinq/mod.rs:1074) L12778,12779,12803,12821; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L12799; `Dom::add_before_self` (xmllinq/mod.rs:1130) L12801; `closing_mark_takes_revised_mark_formatting` (comparer/finalize.rs:3399) L12807
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.collect`, `.copied`, `.count`, `.enumerate`, `.filter`, `.find`, `.flatten`, `.get`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.map`, `.or_else`, `.push_str`, `.push`, `.rev`, `.saturating_sub`, `.starts_with`, `.trim_start`, `.trim`, `.unwrap_or_else`, `Some`, `String::new`, `Vec::new`, `W::body`, `W::del_text`, `W::ins`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr_change`, `W::r_pr`, `W::sect_pr`, `W::t`, `W::tbl`
- **Called by:** `fold_midstream_demo_title_into_numbered_heading_with_settings` (comparer/finalize.rs:12602); `demo_title_pilcrow_history_tests::authored_title_snapshot_survives_demo_default_layout_normalization` (comparer/finalize.rs:28820); `public_body_heuristic_story_scope_tests::configured_heuristics` (comparer/finalize.rs:32397); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `fold_midstream_demo_title_into_numbered_heading_with_source_properties::{fn clean_mark_properties}`  (L12740-12748)
`fn clean_mark_properties(dom: &mut Dom, properties: Option<NodeId>) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L12741,12744; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L12743; `Dom::add` (xmllinq/mod.rs:1074) L12745; `strip_para_revision_marks` (comparer/finalize.rs:3776) L12746
- **Calls (std/external/unresolved):** `.map`, `.unwrap_or_else`, `W::p_pr`, `W::p`
- **Called by:** `fold_midstream_demo_title_into_numbered_heading_with_source_properties` (comparer/finalize.rs:12624)

#### `repair_borrowed_sentence_period`  (L12839-12878, pub)
`pub fn repair_borrowed_sentence_period( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** Move a paragraph-final `.` whose shared equal segment is genuinely shared (the next paragraph starts with equal text) onto the side that already finished its sentence, and append a `.` to the continuation.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L12845; `Dom::elements` (xmllinq/mod.rs:640) L12848; `Dom::name_is` (xmllinq/mod.rs:535) L12850; `trailing_equal_period` (comparer/finalize.rs:12880) L12853; `text_leaves` (comparer/finalize.rs:12903) L12856; `text_side` (comparer/finalize.rs:12910) L12859; `side_text` (comparer/finalize.rs:12928) L12863,12864; `side_needs_borrowed_period` (comparer/finalize.rs:12892) L12865,12866; `wrap_run` (comparer/finalize.rs:12938) L12875; `append_period` (comparer/finalize.rs:12958) L12876
- **Calls (std/external/unresolved):** `.array_windows`, `.first`, `.is_some_and`, `W::body`, `W::del`, `W::ins`, `W::p`
- **Called by:** `coverage_round_next_tests::borrowed_period_moves_to_finished_side_and_finishes_the_continuation` (comparer/finalize.rs:20541); `deep_production_path_tests::borrowed_period_peel_keeps_the_original_period_run_formatting` (comparer/finalize.rs:23046); `deep_production_path_tests::same_format_borrowed_period_peel_keeps_literal_text_and_character_properties` (comparer/finalize.rs:23083); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `trailing_equal_period`  (L12880-12890)
`fn trailing_equal_period(dom: &Dom, p: NodeId) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `text_leaves` (comparer/finalize.rs:12903) L12881; `Dom::name_is` (xmllinq/mod.rs:535) L12882,12886; `text_side` (comparer/finalize.rs:12910) L12882; `Dom::value_str` (xmllinq/mod.rs:1195) L12882; `Dom::parent` (xmllinq/mod.rs:597) L12885,12886
- **Calls (std/external/unresolved):** `.last`, `Some`, `W::r`, `W::t`
- **Called by:** `peel_trailing_del_from_mix_into_following_pure_ins` (comparer/finalize.rs:12266); `repair_borrowed_sentence_period` (comparer/finalize.rs:12839)

#### `side_needs_borrowed_period`  (L12892-12901)
`fn side_needs_borrowed_period(text: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chars`, `.is_empty`, `.is_some_and`, `.last`, `.next`, `.starts_with`, `.trim_end`, `Some`
- **Macros:** `matches!`
- **Called by:** `repair_borrowed_sentence_period` (comparer/finalize.rs:12839)

#### `text_leaves`  (L12903-12908)
`fn text_leaves(dom: &Dom, p: NodeId) -> Vec<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L12904; `Dom::name_is` (xmllinq/mod.rs:535) L12906
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `W::del_text`, `W::t`
- **Called by:** `peel_trailing_del_from_mix_into_following_pure_ins` (comparer/finalize.rs:12266); `repair_borrowed_sentence_period` (comparer/finalize.rs:12839); `trailing_equal_period` (comparer/finalize.rs:12880); `side_text` (comparer/finalize.rs:12928); `append_period` (comparer/finalize.rs:12958); `table_has_both_sides` (comparer/finalize.rs:13037); `block_is_original_only` (comparer/finalize.rs:13059); `collapsed_text` (comparer/finalize.rs:13073); `ParagraphPropertySources::for_mix` (comparer/finalize.rs:13124); `split_insertion_at` (comparer/finalize.rs:15050); `coverage_round_two_tests::wholesale_mesh_emits_exact_shared_words_and_styled_residuals` (comparer/finalize.rs:16854); `coverage_round_two_tests::insertion_split_handles_unicode_run_boundaries_and_multi_leaf_runs` (comparer/finalize.rs:17860)

#### `text_side`  (L12910-12926)
`fn text_side(dom: &Dom, t: NodeId, stop: NodeId) -> &'static str`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L12911,12918,12921; `Dom::ancestors_and_self` (xmllinq/mod.rs:787) L12914
- **Calls (std/external/unresolved):** `W::del_text`, `W::del`, `W::ins`, `W::name`
- **Called by:** `repair_borrowed_sentence_period` (comparer/finalize.rs:12839); `trailing_equal_period` (comparer/finalize.rs:12880); `side_text` (comparer/finalize.rs:12928); `append_period` (comparer/finalize.rs:12958); `table_has_both_sides` (comparer/finalize.rs:13037); `block_is_original_only` (comparer/finalize.rs:13059); `ParagraphPropertySources::for_mix` (comparer/finalize.rs:13124)

#### `side_text`  (L12928-12936)
`fn side_text(dom: &Dom, p: NodeId, side: &str) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `text_leaves` (comparer/finalize.rs:12903) L12930; `text_side` (comparer/finalize.rs:12910) L12931; `Dom::value_str` (xmllinq/mod.rs:1195) L12932
- **Calls (std/external/unresolved):** `.push_str`, `String::new`
- **Called by:** `repair_borrowed_sentence_period` (comparer/finalize.rs:12839)

#### `wrap_run`  (L12938-12956)
`fn wrap_run( dom: &mut Dom, t: NodeId, wrapper_name: crate::xmllinq::XName, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L12945; `Dom::new_element` (xmllinq/mod.rs:478) L12948; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L12949,12951,12952; `Dom::add_before_self` (xmllinq/mod.rs:1130) L12953; `Dom::remove` (xmllinq/mod.rs:1118) L12954; `Dom::add` (xmllinq/mod.rs:1074) L12955
- **Calls (std/external/unresolved):** `.to_string`, `Some`, `W::author`, `W::date`, `W::id`
- **Called by:** `repair_borrowed_sentence_period` (comparer/finalize.rs:12839)

#### `append_period`  (L12958-12968)
`fn append_period(dom: &mut Dom, p: NodeId, side: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `text_leaves` (comparer/finalize.rs:12903) L12959; `text_side` (comparer/finalize.rs:12910) L12962; `Dom::value_str` (xmllinq/mod.rs:1195) (by name) L12966; `Dom::set_value` (xmllinq/mod.rs:1219) L12967
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.rev`
- **Macros:** `format!`
- **Called by:** `repair_borrowed_sentence_period` (comparer/finalize.rs:12839)

#### `restore_echoed_heading_deletions_before_table`  (L12984-13035, pub)
`pub fn restore_echoed_heading_deletions_before_table(dom: &mut Dom, root: NodeId)`

- **Does:** The comments redline aligns B's capability table with A's and emits A's preceding section (Parity through the old "3.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L12985; `Dom::elements` (xmllinq/mod.rs:640) L12988; `table_has_both_sides` (comparer/finalize.rs:13037) L12992; `para_is_pure_inserted` (comparer/finalize.rs:4216) L12997; `block_is_original_only` (comparer/finalize.rs:13059) L13005; `collapsed_text` (comparer/finalize.rs:13073) L13014; `paragraph_texts` (comparer/finalize.rs:13081) L13018; `Dom::remove` (xmllinq/mod.rs:1118) L13031; `Dom::add_before_self` (xmllinq/mod.rs:1130) L13032
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.filter`, `.is_empty`, `.iter`, `.len`, `.map`, `.push`, `.to_vec`, `Vec::new`, `W::body`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `table_has_both_sides`  (L13037-13054)
`fn table_has_both_sides(dom: &Dom, n: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L13038; `text_leaves` (comparer/finalize.rs:12903) L13043; `text_side` (comparer/finalize.rs:12910) L13044; `Dom::value_str` (xmllinq/mod.rs:1195) L13045,13046
- **Calls (std/external/unresolved):** `.is_empty`, `.trim`, `W::tbl`
- **Called by:** `restore_echoed_heading_deletions_before_table` (comparer/finalize.rs:12984)

#### `block_is_original_only`  (L13059-13071)
`fn block_is_original_only(dom: &Dom, n: NodeId) -> bool`

- **Does:** Original-only body block: deleted or moved-from text, and no inserted or moved-to text.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L13060; `Dom::descendants` (xmllinq/mod.rs:668) L13063,13064; `text_leaves` (comparer/finalize.rs:12903) L13068; `Dom::value_str` (xmllinq/mod.rs:1195) L13070; `text_side` (comparer/finalize.rs:12910) L13070
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_empty`, `.trim`, `Some`, `W::ins`, `W::name`, `W::p`, `W::tbl`
- **Called by:** `restore_echoed_heading_deletions_before_table` (comparer/finalize.rs:12984)

#### `collapsed_text`  (L13073-13079)
`fn collapsed_text(dom: &Dom, n: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `text_leaves` (comparer/finalize.rs:12903) L13075; `Dom::value_str` (xmllinq/mod.rs:1195) L13076
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.filter`, `.is_whitespace`, `.push_str`, `String::new`
- **Called by:** `restore_echoed_heading_deletions_before_table` (comparer/finalize.rs:12984); `paragraph_texts` (comparer/finalize.rs:13081)

#### `paragraph_texts`  (L13081-13091)
`fn paragraph_texts(dom: &Dom, block: NodeId) -> Vec<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L13082; `collapsed_text` (comparer/finalize.rs:13073) L13083,13088; `Dom::descendants` (xmllinq/mod.rs:668) L13086
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.map`, `Some`, `Vec::new`, `W::p`
- **Macros:** `vec!`
- **Called by:** `restore_echoed_heading_deletions_before_table` (comparer/finalize.rs:12984)

#### `ParagraphPropertySources::capture`  (L13105-13122, pub)
`pub fn capture(dom: &mut Dom, original: NodeId, revised: NodeId) -> Self`

- **Does:** Save each input paragraph's own comparable properties, including an empty `w:pPr` for a paragraph that has none.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L13108; `revised_text` (comparer/finalize.rs:13370) L13109; `Dom::element` (xmllinq/mod.rs:655) L13110; `project_para_properties_for_change` (comparer/formatchg.rs:329) L13111; `Dom::new_element` (xmllinq/mod.rs:478) L13112
- **Calls (std/external/unresolved):** `.entry`, `.or_default`, `.push`, `.to_string`, `.trim`, `HashMap::new`, `Some`, `W::p_pr`, `W::p`, `capture`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `ParagraphPropertySources::for_mix`  (L13124-13141)
`fn for_mix(&self, dom: &Dom, p: NodeId) -> Option<(NodeId, NodeId)>`

- **Does:** _no doc comment_
- **Calls (crate):** `text_leaves` (comparer/finalize.rs:12903) L13126; `text_side` (comparer/finalize.rs:12910) L13128; `Dom::value` (xmllinq/mod.rs:1185) L13129; `revised_two_blank_table_titles::{fn text}` (comparer/finalize.rs:3891) L13134,13137
- **Calls (std/external/unresolved):** `.as_slice`, `.collect`, `.filter`, `.get`, `.into_iter`, `.map`, `.to_string`, `.trim`, `Some`
- **Called by:** `park_jc_on_first_short_title_mix_from_body` (comparer/finalize.rs:13148)

#### `park_jc_on_first_short_title_mix_from_body`  (L13148-13221, pub)
`pub fn park_jc_on_first_short_title_mix_from_body( dom: &mut Dom, root: NodeId, sources: &ParagraphPropertySources, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** M452: a short title MIX can lose its old alignment during layout peels.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L13158,13196,13202,13210; `Dom::elements` (xmllinq/mod.rs:640) L13161; `Dom::name_is` (xmllinq/mod.rs:535) L13164,13172; `para_is_mixed_revision` (comparer/finalize.rs:5158) L13175; `para_side_word_count` (comparer/finalize.rs:2055) L13183,13184; `ParagraphPropertySources::for_mix` (comparer/finalize.rs:13124) L13188; `normalize_para_properties` (comparer/formatchg.rs:349) L13191,13192; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L13199; `Dom::new_element` (xmllinq/mod.rs:478) L13205,13214; `Dom::add_first` (xmllinq/mod.rs:1101) L13206; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L13213; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L13215,13217,13218; `Dom::add` (xmllinq/mod.rs:1074) L13219,13220
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_some`, `.len`, `.to_string`, `Some`, `W::author`, `W::body`, `W::date`, `W::id`, `W::jc_el`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::sect_pr`, `W::val`
- **Macros:** `matches!`
- **Called by:** `public_body_heuristic_story_scope_tests::configured_heuristics` (comparer/finalize.rs:32397); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `ensure_empty_pprchange_on_live_heading_spacing`  (L13237-13324, pub)
`pub fn ensure_empty_pprchange_on_live_heading_spacing( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** M453 (calibri_font × calibri_heading_2_right mid residual): Word keeps live Heading residual spacing **and** an empty `pPrChange` shell on mid MIX.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L13243,13262,13265,13268,13291,13294...; `Dom::elements` (xmllinq/mod.rs:640) L13246,13272; `Dom::name_is` (xmllinq/mod.rs:535) L13249,13259,13288; `Dom::name` (xmllinq/mod.rs:526) L13273; `para_is_mixed_revision` (comparer/finalize.rs:5158) L13288; `Dom::attribute` (xmllinq/mod.rs:874) L13308,13312; `Dom::new_element` (xmllinq/mod.rs:478) L13315,13316; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L13317,13319,13320; `Dom::add` (xmllinq/mod.rs:1074) L13321,13322
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.filter`, `.into_iter`, `.is_none`, `.is_some`, `.len`, `.ok`, `.parse`, `.to_string`, `.unwrap_or`, `Some`, `W::author`, `W::body`, `W::date`, `W::id`, `W::jc_el`, `W::name`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`, `W::spacing_el`
- **Called by:** `coverage_round_two_tests::live_heading_history_is_added_only_when_last_history_is_empty` (comparer/finalize.rs:18151); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `unaligned_paragraph_texts`  (L13336-13351, pub)
`pub fn unaligned_paragraph_texts(dom: &Dom, body: NodeId) -> HashSet<String>`

- **Does:** The text of each paragraph that sets no `w:jc`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L13337,13345; `Dom::element` (xmllinq/mod.rs:655) L13340,13341; `Dom::value` (xmllinq/mod.rs:1185) L13347
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.filter`, `.into_iter`, `.is_none`, `.map`, `Some`, `W::jc_el`, `W::p_pr`, `W::p`, `W::t`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `accepted_text`  (L13355-13367)
`fn accepted_text(dom: &Dom, kids: &[NodeId], p: NodeId) -> String`

- **Does:** The text `p` holds once accepted: its revised text after that of the paragraphs before it whose deleted marks merge them into `p`.
- **Calls (crate):** `revised_text` (comparer/finalize.rs:13370) L13357,13365; `Dom::name_is` (xmllinq/mod.rs:535) L13361; `para_mark_revision` (comparer/finalize.rs:7631) L13361
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map_or`, `.map`, `.position`, `.rposition`, `W::del`, `W::p`
- **Called by:** `promote_live_jc_from_pprchange_on_body_mix` (comparer/finalize.rs:6516)

#### `revised_text`  (L13370-13381)
`fn revised_text(dom: &Dom, p: NodeId) -> String`

- **Does:** A redline paragraph's revised text: its live and inserted `w:t`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L13371; `Dom::ancestors` (xmllinq/mod.rs:770) L13374; `Dom::name_is` (xmllinq/mod.rs:535) L13377; `Dom::value` (xmllinq/mod.rs:1185) L13379
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.filter`, `.into_iter`, `.map`, `.take_while`, `Some`, `W::del`, `W::t`
- **Called by:** `ParagraphPropertySources::capture` (comparer/finalize.rs:13105); `accepted_text` (comparer/finalize.rs:13355)

#### `paragraph_alignments`  (L13391-13404, pub)
`pub fn paragraph_alignments(dom: &Dom, body: NodeId) -> HashSet<(String, String)>`

- **Does:** Each paragraph's text and `w:jc` value, for paragraphs with a `w:jc`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L13392,13396; `Dom::element` (xmllinq/mod.rs:655) L13395; `Dom::value` (xmllinq/mod.rs:1185) L13399; `Dom::attribute` (xmllinq/mod.rs:874) L13401
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.into_iter`, `.map`, `.to_string`, `.unwrap_or`, `Some`, `W::jc_el`, `W::p_pr`, `W::p`, `W::t`, `W::val`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `ensure_empty_pprchange_on_eq_with_live_jc`  (L13425-13488, pub)
`pub fn ensure_empty_pprchange_on_eq_with_live_jc( dom: &mut Dom, root: NodeId, original: &HashSet<(String, String)>, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** M454 (center_alignment_2 residual ~87 → 100): Word EQ title keeps live `jc` + empty `pPrChange`.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L13432,13458,13461,13473,13476; `Dom::elements` (xmllinq/mod.rs:640) L13435; `Dom::name_is` (xmllinq/mod.rs:535) L13438,13441; `para_is_mixed_revision` (comparer/finalize.rs:5158) L13444; `para_is_pure_inserted` (comparer/finalize.rs:4216) L13445; `para_is_pure_deleted` (comparer/finalize.rs:2900) L13446; `Dom::descendants` (xmllinq/mod.rs:668) L13451,13452,13464; `para_mark_revision` (comparer/finalize.rs:7631) L13453,13454; `Dom::value` (xmllinq/mod.rs:1185) L13467; `Dom::attribute` (xmllinq/mod.rs:874) L13469; `Dom::new_element` (xmllinq/mod.rs:478) L13479,13480; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L13481,13483,13484; `Dom::add` (xmllinq/mod.rs:1074) L13485,13486
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter`, `.into_iter`, `.is_empty`, `.is_some`, `.map`, `.to_string`, `.unwrap_or`, `Some`, `W::author`, `W::body`, `W::date`, `W::del`, `W::id`, `W::ins`, `W::jc_el`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`, `W::sect_pr`, `W::t`, `W::val`
- **Called by:** `finalizer_distinct_boundary_contract_tests::an_equal_alignment_change_is_attributed_once_and_never_claims_source_history` (comparer/finalize.rs:29672); `public_body_heuristic_story_scope_tests::configured_heuristics` (comparer/finalize.rs:32397); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `strip_leading_del_echoing_prev_pure_i`  (L13503-13592, pub)
`pub fn strip_leading_del_echoing_prev_pure_i(dom: &mut Dom, root: NodeId)`

- **Does:** M458 (right_aligned_italic residual ~88 after M151 peel): M151 pure-I's B1 ("This document…") then free-meshes A1×B2, which leaves an **early del "This"** on the body MIX (often after a short leading ins "All") that Word does not show.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L13504; `Dom::elements` (xmllinq/mod.rs:640) L13507,13536; `Dom::name_is` (xmllinq/mod.rs:535) L13510,13515,13539,13547; `para_is_pure_inserted` (comparer/finalize.rs:4216) L13518; `para_is_mixed_revision` (comparer/finalize.rs:5158) L13518; `para_mark_revision` (comparer/finalize.rs:7631) L13525; `para_revision_body_text` (comparer/finalize.rs:7859) L13528; `Dom::descendants` (xmllinq/mod.rs:668) L13552,13563; `Dom::value_str` (xmllinq/mod.rs:1195) L13553,13564; `Dom::remove` (xmllinq/mod.rs:1118) L13590
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.contains`, `.filter`, `.find`, `.into_iter`, `.is_alphanumeric`, `.is_empty`, `.is_none`, `.iter`, `.len`, `.map`, `.split`, `.take`, `.to_ascii_lowercase`, `Some`, `W::body`, `W::del_text`, `W::del`, `W::p_pr`, `W::p`, `W::sect_pr`, `W::t`
- **Called by:** `coverage_round_two_tests::leading_deleted_echo_is_removed_only_with_one_or_two_tokens` (comparer/finalize.rs:18062); `deleted_carrier_echo_source_owner_tests::deleted_original_carrier_keeps_echo_prefix_and_both_complete_source_projections` (comparer/finalize.rs:34867); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `holds_only_text`  (L13600-13613)
`fn holds_only_text(dom: &Dom, node: NodeId) -> bool`

- **Does:** Whether a paragraph child holds only text: a run, or a revision wrapper of runs, whose content is run properties and `w:t`/`w:delText`.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L13602,13604,13605,13606,13609; `Dom::elements` (xmllinq/mod.rs:640) L13603,13612; `text_run` (convert/math_layout.rs:530) (by name) L13610
- **Calls (std/external/unresolved):** `.all`, `.into_iter`, `W::del_text`, `W::r_pr`, `W::r`, `W::t`
- **Called by:** `free_mesh_shared_title_token_in_mix` (comparer/finalize.rs:2573); `free_mesh_wholesale_body_mix` (comparer/finalize.rs:13633); `free_mesh_bookended_ins_del` (comparer/finalize.rs:14073)

#### `free_mesh_wholesale_body_mix`  (L13633-13784, pub)
`pub fn free_mesh_wholesale_body_mix(dom: &mut Dom, root: NodeId)`

- **Does:** M462 (center_aligned_bold body2 residual after M461): wholesale body MIX free-mesh with coverage-gated word-LCS.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L13634,13761,13763; `Dom::elements` (xmllinq/mod.rs:640) L13637,13646; `Dom::name_is` (xmllinq/mod.rs:535) L13640,13643,13649; `para_is_mixed_revision` (comparer/finalize.rs:5158) L13643; `Dom::name` (xmllinq/mod.rs:526) L13658; `Dom::descendants` (xmllinq/mod.rs:668) L13666,13688,13692; `Dom::value_str` (xmllinq/mod.rs:1195) L13667,13689,13693; `holds_only_text` (comparer/finalize.rs:13600) L13681; `alnum_tokens` (comparer/finalize.rs:2759) L13704,13705; `split_words_preserve` (comparer/finalize.rs:13786) L13734,13735; `word_lcs_indices_eligible` (comparer/finalize.rs:13826) L13739; `Dom::attribute` (xmllinq/mod.rs:874) L13753,13756; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L13764; `Dom::parent` (xmllinq/mod.rs:597) L13766; `Dom::remove` (xmllinq/mod.rs:1118) L13767; `rebuild_body_free_mesh_lcs` (comparer/finalize.rs:13851) L13770
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.as_str`, `.chars`, `.collect`, `.contains`, `.count`, `.ends_with`, `.filter`, `.intersection`, `.into_iter`, `.is_alphanumeric`, `.is_empty`, `.is_some`, `.iter`, `.len`, `.map`, `.min`, `.push_str`, `.push`, `.to_ascii_lowercase`, `.to_string`, `.trim_end`, `Some`, `String::new`, `Vec::new`, `W::author`, `W::body`, `W::date`, `W::del_text`, `W::del`, `W::ins`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `W::sect_pr`, `W::t`, `is_sig`
- **Called by:** `coverage_round_two_tests::wholesale_mesh_emits_exact_shared_words_and_styled_residuals` (comparer/finalize.rs:16854); `coverage_round_two_tests::wholesale_mesh_preserves_each_authored_terminal_whitespace_and_format` (comparer/finalize.rs:16913); `coverage_round_two_tests::wholesale_mesh_without_terminal_whitespace_keeps_exact_supported_word_streams` (comparer/finalize.rs:16964); `fragmented_source_label_boundary_tests::wholesale_glue_only_lcs_preserves_crossed_significant_source_anchors` (comparer/finalize.rs:34112); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `split_words_preserve`  (L13786-13791)
`fn split_words_preserve(text: &str) -> Vec<String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.is_alphanumeric`, `.is_empty`, `.map`, `.split`, `.to_string`
- **Called by:** `free_mesh_wholesale_body_mix` (comparer/finalize.rs:13633)

#### `word_lcs_indices`  (L13793-13822)
`fn word_lcs_indices(a: &[String], b: &[String]) -> Vec<(usize, usize)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.len`, `.max`, `.push`, `.reverse`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `word_lcs_indices_eligible` (comparer/finalize.rs:13826)

#### `word_lcs_indices_eligible`  (L13826-13846)
`fn word_lcs_indices_eligible(a: &[String], b: &[String], boiler: &[&str]) -> Vec<(usize, usize)>`

- **Does:** LCS only on significant (len≥4 non-boiler) or short glue words; map indices back to the full word lists.
- **Calls (crate):** `word_lcs_indices` (comparer/finalize.rs:13793) L13842
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.contains`, `.filter`, `.into_iter`, `.is_empty`, `.iter`, `.len`, `.map`, `Vec::new`, `eligible`
- **Called by:** `free_mesh_wholesale_body_mix` (comparer/finalize.rs:13633)

#### `rebuild_body_free_mesh_lcs`  (L13851-14054)
`fn rebuild_body_free_mesh_lcs( dom: &mut Dom, p: NodeId, ins_words: &[String], del_words: &[String], lcs: &[(usize, usize)], stamp: RevStamp<'_>, trailing_period: bool, )`

- **Does:** Rebuild free-mesh from word LCS.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L13890,13895,13904,13909,13914,13924...; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L13892,13911,13931,14046; `Dom::add` (xmllinq/mod.rs:1074) L13893,13900,13901,13912,13919,13920...; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L13897,13905,13907,13908,13916,13925...; `XNamespace::xml` (xmllinq/mod.rs:128) L13897,13916,13936; `Dom::add_text` (xmllinq/mod.rs:1087) L13899,13918,13938,14050; `push_run` (markdown/patch.rs:813) (by name) L13952,13976,13995,14014,14028
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.ends_with`, `.get`, `.is_empty`, `.join`, `.len`, `.name`, `.push`, `.starts_with`, `.to_string`, `.to_vec`, `Some`, `Vec::new`, `W::author`, `W::date`, `W::del_text`, `W::del`, `W::id`, `W::ins`, `W::r`, `W::t`
- **Macros:** `format!`
- **Called by:** `free_mesh_wholesale_body_mix` (comparer/finalize.rs:13633); `coverage_round_two_tests::lcs_rebuild_covers_equal_prefixes_and_both_unmatched_tails` (comparer/finalize.rs:17074)

#### `free_mesh_bookended_ins_del`  (L14073-14260, pub)
`pub fn free_mesh_bookended_ins_del(dom: &mut Dom, root: NodeId)`

- **Does:** M460 (right_align_bold×right_aligned_italic ~81.3): MIX with EQ bookends (`This ` … `.`) around a single wholesale ins+del that share one significant mid token (`right`).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L14074,14206,14209,14211; `Dom::elements` (xmllinq/mod.rs:640) L14077,14086; `Dom::name_is` (xmllinq/mod.rs:535) L14080,14083,14089,14205; `para_is_mixed_revision` (comparer/finalize.rs:5158) L14083; `Dom::name` (xmllinq/mod.rs:526) L14099,14218; `Dom::descendants` (xmllinq/mod.rs:668) L14106,14129,14133; `Dom::value_str` (xmllinq/mod.rs:1195) L14107,14130,14134; `holds_only_text` (comparer/finalize.rs:13600) L14122; `alnum_tokens` (comparer/finalize.rs:2759) L14136,14137; `split_around_whole_word` (comparer/finalize.rs:14264) L14178,14181; `Dom::attribute` (xmllinq/mod.rs:874) L14194,14197; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L14207,14212,14230,14231; `Dom::parent` (xmllinq/mod.rs:597) L14233; `Dom::remove` (xmllinq/mod.rs:1118) L14234; `Dom::add` (xmllinq/mod.rs:1074) L14238,14257; `m460_push_rev_text` (comparer/finalize.rs:14310) L14251,14252,14254,14255; `m460_push_eq_text` (comparer/finalize.rs:14292) L14253
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.as_str`, `.chars`, `.collect`, `.contains`, `.copied`, `.entry`, `.filter`, `.find`, `.first`, `.get`, `.into_iter`, `.is_alphanumeric`, `.is_empty`, `.is_some`, `.iter`, `.keys`, `.len`, `.map`, `.or_default`, `.push_str`, `.push`, `.sort_by_key`, `.to_string`, `Some`, `String::new`, `Vec::new`, `W::author`, `W::body`, `W::date`, `W::del_text`, `W::del`, `W::ins`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `W::sect_pr`, `W::t`, `std::cmp::Reverse`, `std::collections::HashMap::new`
- **Called by:** `coverage_tests::bookended_free_mesh_rebuilds_word_order_and_preserves_run_properties` (comparer/finalize.rs:16518); `deep_production_path_tests::bookended_mesh_handles_boundary_anchors_and_the_forty_token_limit` (comparer/finalize.rs:22301); `deep_production_path_tests::suffix_only_bookend_mesh_uses_pinned_defaults_and_keeps_the_original_suffix` (comparer/finalize.rs:23463); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `split_around_whole_word`  (L14264-14290)
`fn split_around_whole_word(text: &str, word: &str) -> Option<(String, String)>`

- **Does:** Split `text` around the first whole-word match of `word` (case-insensitive).
- **Calls (std/external/unresolved):** `.and_then`, `.chars`, `.find`, `.get`, `.is_alphanumeric`, `.is_some_and`, `.len`, `.next`, `.to_ascii_lowercase`, `.to_string`, `Some`
- **Called by:** `free_mesh_bookended_ins_del` (comparer/finalize.rs:14073); `coverage_tests::token_splitting_preserves_separators_and_original_case` (comparer/finalize.rs:15744)

#### `m460_push_eq_text`  (L14292-14308)
`fn m460_push_eq_text(dom: &mut Dom, p: NodeId, text: &str, sample_rpr: &Option<NodeId>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L14296,14301; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L14298; `Dom::add` (xmllinq/mod.rs:1074) L14299,14306,14307; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L14303; `XNamespace::xml` (xmllinq/mod.rs:128) L14303; `Dom::add_text` (xmllinq/mod.rs:1087) L14305
- **Calls (std/external/unresolved):** `.ends_with`, `.is_empty`, `.name`, `.starts_with`, `Some`, `W::r`, `W::t`
- **Called by:** `free_mesh_bookended_ins_del` (comparer/finalize.rs:14073)

#### `m460_push_rev_text`  (L14310-14366)
`fn m460_push_rev_text( dom: &mut Dom, p: NodeId, kind: &str, text: &str, stamp: RevStamp<'_>, next_id: &mut u32, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L14329,14334,14335,14345,14350,14355; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L14330,14332,14333,14337,14346,14348...; `XNamespace::xml` (xmllinq/mod.rs:128) L14337,14357; `Dom::add_text` (xmllinq/mod.rs:1087) L14339,14359; `Dom::add` (xmllinq/mod.rs:1074) L14340,14341,14342,14353,14360,14361...; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L14352
- **Calls (std/external/unresolved):** `.contains`, `.ends_with`, `.is_empty`, `.name`, `.starts_with`, `.to_string`, `Some`, `W::author`, `W::date`, `W::del_text`, `W::del`, `W::id`, `W::ins`, `W::r`, `W::t`
- **Called by:** `free_mesh_bookended_ins_del` (comparer/finalize.rs:14073)

#### `peel_trailing_for_word_onto_next_mix`  (L14381-14516, pub)
`pub fn peel_trailing_for_word_onto_next_mix(dom: &mut Dom, root: NodeId)`

- **Does:** M464 (center_bold ~83.6 residual): Word free-meshes trailing ` for <word>` from a mid-body MIX ins onto the following MIX as `EQ[ for ]|INS[<word> ]`, trimming the following del's trailing ` for`.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L14382,14468,14470,14471,14474,14476; `Dom::elements` (xmllinq/mod.rs:640) L14385,14400,14405; `Dom::name_is` (xmllinq/mod.rs:535) L14388,14393,14394,14403,14408,14411...; `para_is_mixed_revision` (comparer/finalize.rs:5158) L14395,14396; `Dom::descendants` (xmllinq/mod.rs:668) L14424,14428; `Dom::value_str` (xmllinq/mod.rs:1195) L14425,14429; `split_trailing_for_word` (comparer/finalize.rs:14519) L14431; `Dom::attribute` (xmllinq/mod.rs:874) L14456,14459; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L14473,14477,14488,14502; `rewrite_rev_text` (comparer/finalize.rs:14545) L14480,14482; `Dom::new_element` (xmllinq/mod.rs:478) L14486,14491,14496,14500,14505; `Dom::add` (xmllinq/mod.rs:1074) L14489,14494,14503,14509,14510; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L14492,14497,14498,14499,14507; `XNamespace::xml` (xmllinq/mod.rs:128) L14492,14507; `Dom::add_text` (xmllinq/mod.rs:1087) L14493,14508; `Dom::add_after_self` (xmllinq/mod.rs:1147) L14513,14514
- **Calls (std/external/unresolved):** `.and_then`, `.as_bytes`, `.collect`, `.copied`, `.ends_with`, `.filter`, `.find`, `.first`, `.into_iter`, `.is_empty`, `.iter`, `.len`, `.map`, `.name`, `.or_else`, `.push_str`, `.rev`, `.saturating_sub`, `.to_ascii_lowercase`, `.to_string`, `.trim_end`, `.trim`, `Some`, `String::new`, `W::author`, `W::body`, `W::date`, `W::del_text`, `W::del`, `W::id`, `W::ins`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `W::sect_pr`, `W::t`
- **Macros:** `format!`
- **Called by:** `coverage_round_next_tests::for_word_peel_preserves_run_format_and_uses_equal_format_for_shared_connector` (comparer/finalize.rs:20485); `deep_production_path_tests::connector_peel_preserves_both_sources_and_falls_back_to_inserted_run_format` (comparer/finalize.rs:22353); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `split_trailing_for_word`  (L14519-14542)
`fn split_trailing_for_word(text: &str) -> Option<(String, String)>`

- **Does:** Split trailing ` for <word>` (case-insensitive).
- **Calls (std/external/unresolved):** `.all`, `.any`, `.chars`, `.is_alphanumeric`, `.is_empty`, `.is_whitespace`, `.rfind`, `.to_ascii_lowercase`, `.to_string`, `.trim_end`, `.trim`, `Some`
- **Called by:** `peel_trailing_for_word_onto_next_mix` (comparer/finalize.rs:14381)

#### `rewrite_rev_text`  (L14545-14581)
`fn rewrite_rev_text(dom: &mut Dom, rev: NodeId, new_text: &str, is_del: bool)`

- **Does:** Replace all text under an ins/del with `new_text` (single run).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L14546,14550,14552; `Dom::parent` (xmllinq/mod.rs:597) L14555,14561; `Dom::remove` (xmllinq/mod.rs:1118) L14556,14562; `Dom::new_element` (xmllinq/mod.rs:478) L14567,14572,14574; `Dom::add` (xmllinq/mod.rs:1074) L14568,14580; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L14577; `XNamespace::xml` (xmllinq/mod.rs:128) L14577; `Dom::add_text` (xmllinq/mod.rs:1087) L14579
- **Calls (std/external/unresolved):** `.contains`, `.ends_with`, `.first`, `.is_some`, `.iter`, `.name`, `.skip`, `.starts_with`, `.to_vec`, `Some`, `W::del_text`, `W::r`, `W::t`
- **Called by:** `peel_trailing_for_word_onto_next_mix` (comparer/finalize.rs:14381)

#### `hoist_hyperlinks_out_of_revisions`  (L14616-14648, pub)
`pub fn hoist_hyperlinks_out_of_revisions(dom: &mut Dom, root: NodeId)`

- **Does:** `w:hyperlink` is not in CT_RunTrackChange's content model: a `w:ins`/`w:del` holds runs, not a hyperlink wrapper.
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L14619; `Dom::attribute` (xmllinq/mod.rs:874) L14623; `Dom::descendants` (xmllinq/mod.rs:668) L14634; `Dom::elements` (xmllinq/mod.rs:640) L14636; `Dom::name` (xmllinq/mod.rs:526) L14638; `split_revision_around_hyperlinks` (comparer/finalize.rs:14652) L14645
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.collect`, `.contains`, `.filter_map`, `.filter`, `.flat_map`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.max`, `.ok`, `.parse`, `.unwrap_or`, `Some`, `W::hyperlink`, `W::id`, `W::name`
- **Called by:** `coverage_final_batch_finalize_tests::hyperlink_hoisting_preserves_empty_wrappers_and_run_order_across_revisions` (comparer/finalize.rs:24482); `coverage_final_batch_finalize_tests::empty_field_in_a_revision_keeps_its_flags_and_padded_code` (comparer/finalize.rs:24566); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215); `compare_documents_impl` (document_comparer.rs:6521); `coverage_boundary_tests::one_sided_story_fields_join_the_story_revision` (document_comparer.rs:9532)

#### `split_revision_around_hyperlinks`  (L14652-14723)
`fn split_revision_around_hyperlinks(dom: &mut Dom, rev: NodeId, next_id: &mut u32)`

- **Does:** One step of [`hoist_hyperlinks_out_of_revisions`]: replace `rev` in its parent with the alternating sequence of same-shaped revisions and hyperlinks.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L14653,14695; `Dom::attributes` (xmllinq/mod.rs:899) L14657; `Dom::nodes` (xmllinq/mod.rs:604) L14658,14699; `Dom::new_element` (xmllinq/mod.rs:478) L14663; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L14665,14667; `Dom::remove` (xmllinq/mod.rs:1118) L14674,14703,14722; `Dom::name_is` (xmllinq/mod.rs:535) L14675; `Dom::elements` (xmllinq/mod.rs:640) L14675; `complex_field_runs` (comparer/finalize.rs:14728) L14680; `Dom::add_before_self` (xmllinq/mod.rs:1130) L14685,14708,14715; `Dom::add` (xmllinq/mod.rs:1074) L14691,14704,14706,14720
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.is_empty`, `.is_some_and`, `.to_string`, `Some`, `W::del`, `W::hyperlink`, `W::id`, `W::name`, `like_rev`
- **Called by:** `hoist_hyperlinks_out_of_revisions` (comparer/finalize.rs:14616)

#### `complex_field_runs`  (L14728-14756)
`fn complex_field_runs(dom: &mut Dom, field: NodeId, deleted: bool) -> Vec<NodeId>`

- **Does:** An empty `w:fldSimple` as the runs of its complex form: `begin` (with the field's `w:dirty` / `w:fldLock`), the code (`w:delInstrText` when `deleted`), `separate` and `end`.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L14730,14731,14742,14743; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L14732,14739,14749; `Dom::add` (xmllinq/mod.rs:1074) L14733,14752; `Dom::attribute` (xmllinq/mod.rs:874) L14738,14744; `XNamespace::xml` (xmllinq/mod.rs:128) L14749; `Dom::add_text` (xmllinq/mod.rs:1087) L14751
- **Calls (std/external/unresolved):** `.map`, `.name`, `.to_string`, `.trim`, `.unwrap_or`, `Some`, `W::name`, `W::r`, `char_run`
- **Macros:** `vec!`
- **Called by:** `mark_content_transform` (comparer/finalize.rs:192); `split_revision_around_hyperlinks` (comparer/finalize.rs:14652)

#### `repair_inherited_invalidity`  (L14789-14828, pub)
`pub fn repair_inherited_invalidity(dom: &mut Dom, root: NodeId)`

- **Does:** Repair invalidity we did not create but would otherwise ship.
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L14790,14804,14811; `Dom::name` (xmllinq/mod.rs:526) L14791,14821,14823; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L14792,14796; `Dom::attributes` (xmllinq/mod.rs:899) L14795; `Dom::is_namespace_declaration` (xmllinq/mod.rs:1012) L14797; `XName::local_name` (xmllinq/mod.rs:208) (by name) L14798; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L14800,14806,14813; `Dom::attribute` (xmllinq/mod.rs:874) L14805,14812; `Dom::descendants` (xmllinq/mod.rs:668) L14818; `Dom::parent` (xmllinq/mod.rs:597) L14819,14822; `Dom::remove` (xmllinq/mod.rs:1118) L14825
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.filter`, `.is_empty`, `.is_none`, `.is_some_and`, `Some`, `W::name`, `W::r_pr`, `W::val`
- **Called by:** `coverage_final_batch_finalize_tests::inherited_invalidity_repairs_only_wml_schema_defects` (comparer/finalize.rs:24583); `compare_documents_impl` (document_comparer.rs:6521)

#### `enforce_deleted_text_kinds`  (L14856-14879, pub)
`pub fn enforce_deleted_text_kinds(dom: &mut Dom, root: NodeId)`

- **Does:** Enforce the deleted-text invariant everywhere, not just on runs that happened to go through `convert_run_text_to_del_text`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L14876; `descendants_trimmed::{fn walk}` (comparer/finalize.rs:65) L14877
- **Calls (std/external/unresolved):** `Some`, `W::del`
- **Called by:** `nested_deleted_text_source_ownership_tests::outer_deletion_preserves_independently_owned_nested_revision_text_and_fields` (comparer/finalize.rs:34181); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215); `compare_documents_impl` (document_comparer.rs:6521)

#### `enforce_deleted_text_kinds::{fn walk}`  (L14857-14875)
`fn walk(dom: &mut Dom, node: NodeId)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L14858; `Dom::is_element` (xmllinq/mod.rs:502) L14859; `Dom::name` (xmllinq/mod.rs:526) L14862; `Dom::set_name` (xmllinq/mod.rs:539) L14870,14871; `descendants_trimmed::{fn walk}` (comparer/finalize.rs:65) L14872
- **Calls (std/external/unresolved):** `.as_ref`, `W::del_text`, `W::ins`, `W::instr_text`, `W::name`, `W::t`
- **Called by:** `descendants_trimmed` (comparer/finalize.rs:63); `descendants_trimmed::{fn walk}` (comparer/finalize.rs:65); `convert_run_text_to_del_text` (comparer/finalize.rs:102); `convert_run_text_to_del_text::{fn walk}` (comparer/finalize.rs:103); `enforce_deleted_text_kinds` (comparer/finalize.rs:14856); `enforce_deleted_text_kinds::{fn walk}` (comparer/finalize.rs:14857)

#### `align_remaining_ladder_rungs`  (L14908-14921, pub)
`pub fn align_remaining_ladder_rungs(dom: &mut Dom, root: NodeId, sources: &LadderSources<'_>)`

- **Does:** Match the six output shapes the parity ladder still counted after the operation sequence already matched — a trailing revised insertion split out of the comparison insertion, a deleted pilcrow on a tab-joined swallowed title, a commented list item unmerged from the deletion, a duplicated Normal-styled deletion, spacin...
- **Calls (crate):** `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L14909,14910; `Dom::root` (xmllinq/mod.rs:822) L14911; `max_numeric_id` (comparer/finalize.rs:14923) L14914; `split_trailing_insertions` (comparer/finalize.rs:15033) L14915; `mark_tabbed_deleted_titles` (comparer/finalize.rs:15172) L14916; `unmerge_comment_list_item` (comparer/finalize.rs:15255) L14917; `drop_duplicate_normal_style` (comparer/finalize.rs:15321) L14918; `move_spacing_onto_bare_deletion` (comparer/finalize.rs:15386) L14919; `stamp_trailing_numbering` (comparer/finalize.rs:15468) L14920
- **Calls (std/external/unresolved):** `.saturating_add`
- **Called by:** `ladder_suffix_source_boundary_tests::public_ladder_restores_suffix_author_at_unicode_and_run_boundaries` (comparer/finalize.rs:34292); `compare_documents_impl` (document_comparer.rs:6521)

#### `max_numeric_id`  (L14923-14933)
`fn max_numeric_id(dom: &Dom, root: NodeId) -> u32`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L14925; `Dom::attribute` (xmllinq/mod.rs:874) L14926
- **Calls (std/external/unresolved):** `.max`, `.parse`, `W::id`
- **Called by:** `mesh_short_label_shared_eq_with_settings` (comparer/finalize.rs:7204); `peel_trailing_del_from_mix_into_following_pure_ins` (comparer/finalize.rs:12266); `fold_midstream_demo_title_into_numbered_heading` (comparer/finalize.rs:12567); `align_remaining_ladder_rungs` (comparer/finalize.rs:14908); `short_label_format_history_tests::configured_label_tracking_preserves_sources_and_respects_disabled_changes` (comparer/finalize.rs:24170)

#### `body_paragraphs`  (L14935-14937)
`fn body_paragraphs(dom: &Dom, root: NodeId) -> Vec<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L14936
- **Calls (std/external/unresolved):** `Some`, `W::p`
- **Called by:** `suffix_hits` (comparer/finalize.rs:15008); `mark_tabbed_deleted_titles` (comparer/finalize.rs:15172); `unmerge_comment_list_item` (comparer/finalize.rs:15255); `drop_duplicate_normal_style` (comparer/finalize.rs:15321); `move_spacing_onto_bare_deletion` (comparer/finalize.rs:15386); `stamp_trailing_numbering` (comparer/finalize.rs:15468)

#### `markup_text`  (L14940-14954)
`fn markup_text(dom: &Dom, node: NodeId) -> String`

- **Does:** Concatenated `w:t` and `w:delText`, with nothing inserted between nodes.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L14945; `Dom::value` (xmllinq/mod.rs:1185) L14946; `Dom::descendants` (xmllinq/mod.rs:668) L14950
- **Calls (std/external/unresolved):** `.push_str`, `String::new`, `W::del_text`, `W::t`, `consider`
- **Called by:** `suffix_hits` (comparer/finalize.rs:15008); `split_trailing_insertions` (comparer/finalize.rs:15033); `mark_tabbed_deleted_titles` (comparer/finalize.rs:15172); `content_revisions` (comparer/finalize.rs:15211); `drop_duplicate_normal_style` (comparer/finalize.rs:15321); `move_spacing_onto_bare_deletion` (comparer/finalize.rs:15386); `stamp_trailing_numbering` (comparer/finalize.rs:15468); `coverage_round_two_tests::segments` (comparer/finalize.rs:16818); `coverage_round_two_tests::title_page_relocation_preserves_six_spacers_and_moves_only_last` (comparer/finalize.rs:17114); `coverage_round_two_tests::stranded_mark_is_cloned_without_removing_empty_shell` (comparer/finalize.rs:17261); `coverage_round_two_tests::heading_spacing_parks_on_last_short_deleted_residual` (comparer/finalize.rs:17381); `coverage_round_two_tests::mixed_numbering_moves_before_last_deletion_mark_or_history` (comparer/finalize.rs:17463); +11 more

#### `inside`  (L14956-14964)
`fn inside(dom: &Dom, mut node: NodeId, ancestor: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L14957
- **Called by:** `content_revisions` (comparer/finalize.rs:15211); `locate` (edit.rs:7203) (?)

#### `inside_table`  (L14966-14975)
`fn inside_table(dom: &Dom, node: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L14969; `Dom::parent` (xmllinq/mod.rs:597) L14972
- **Calls (std/external/unresolved):** `Some`, `W::tbl`
- **Called by:** `mark_tabbed_deleted_titles` (comparer/finalize.rs:15172)

#### `run_holding`  (L14977-14988)
`fn run_holding(dom: &Dom, node: NodeId, ins: NodeId) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L14983; `Dom::parent` (xmllinq/mod.rs:597) L14986
- **Calls (std/external/unresolved):** `Some`, `W::r`
- **Called by:** `split_insertion_at` (comparer/finalize.rs:15050)

#### `split_at_chars`  (L14990-14999)
`fn split_at_chars(value: &str, chars: usize) -> (String, String)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.char_indices`, `.enumerate`, `.len`, `.to_string`
- **Called by:** `split_insertion_at` (comparer/finalize.rs:15050)

#### `suffix_hits`  (L15008-15031)
`fn suffix_hits(dom: &Dom, revised: NodeId) -> Vec<SuffixHit>`

- **Does:** _no doc comment_
- **Calls (crate):** `body_paragraphs` (comparer/finalize.rs:14935) L15010; `Dom::descendants` (xmllinq/mod.rs:668) L15011; `markup_text` (comparer/finalize.rs:14940) L15015,15016; `Dom::attribute` (xmllinq/mod.rs:874) L15024,15027
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.ends_with`, `.is_empty`, `.len`, `.map`, `.push`, `Some`, `Vec::new`, `W::author`, `W::date`, `W::ins`
- **Called by:** `split_trailing_insertions` (comparer/finalize.rs:15033)

#### `split_trailing_insertions`  (L15033-15048)
`fn split_trailing_insertions(dom: &mut Dom, root: NodeId, revised: NodeId, id_gen: &mut u32)`

- **Does:** _no doc comment_
- **Calls (crate):** `suffix_hits` (comparer/finalize.rs:15008) L15034; `Dom::descendants` (xmllinq/mod.rs:668) L15038; `markup_text` (comparer/finalize.rs:14940) L15040; `split_insertion_at` (comparer/finalize.rs:15050) L15046
- **Calls (std/external/unresolved):** `.clone`, `.find`, `.is_empty`, `.iter`, `Some`, `W::ins`
- **Called by:** `align_remaining_ladder_rungs` (comparer/finalize.rs:14908)

#### `split_insertion_at`  (L15050-15160)
`fn split_insertion_at( dom: &mut Dom, ins: NodeId, cut: usize, author: &Option<String>, date: &Option<String>, id_gen: &mut u32, ) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `text_leaves` (comparer/finalize.rs:12903) L15058,15146; `Dom::value` (xmllinq/mod.rs:1185) L15063,15084; `split_at_chars` (comparer/finalize.rs:14990) L15084; `Dom::name` (xmllinq/mod.rs:526) L15085; `Dom::set_value` (xmllinq/mod.rs:1219) L15088,15091; `Dom::new_element` (xmllinq/mod.rs:478) L15089,15109; `copy_attrs` (comparer/finalize.rs:136) L15090,15110; `XNamespace::xml` (xmllinq/mod.rs:128) L15094; `xml_space_attr` (comparer/finalize.rs:1082) L15096; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L15097,15112,15115,15117; `Dom::add_after_self` (xmllinq/mod.rs:1147) L15100,15158; `run_holding` (comparer/finalize.rs:14977) L15122,15132; `Dom::remove` (xmllinq/mod.rs:1118) L15135,15143,15147,15153; `Dom::add` (xmllinq/mod.rs:1074) L15136,15149,15154; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L15139; `Dom::nodes` (xmllinq/mod.rs:604) L15140,15151,15152
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.contains`, `.copied`, `.count`, `.enumerate`, `.filter`, `.insert`, `.into_iter`, `.is_empty`, `.iter`, `.len`, `.name`, `.push`, `.take`, `.to_string`, `.to_vec`, `Some`, `Vec::new`, `W::author`, `W::date`, `W::id`, `W::ins`
- **Called by:** `split_trailing_insertions` (comparer/finalize.rs:15033); `coverage_round_two_tests::insertion_split_handles_unicode_run_boundaries_and_multi_leaf_runs` (comparer/finalize.rs:17860)

#### `has_deleted_mark`  (L15162-15170)
`fn has_deleted_mark(dom: &Dom, paragraph: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L15163,15166,15169
- **Calls (std/external/unresolved):** `.is_some`, `W::del`, `W::p_pr`, `W::r_pr`
- **Called by:** `mark_tabbed_deleted_titles` (comparer/finalize.rs:15172); `drop_duplicate_normal_style` (comparer/finalize.rs:15321)

#### `mark_tabbed_deleted_titles`  (L15172-15209)
`fn mark_tabbed_deleted_titles( dom: &mut Dom, root: NodeId, original: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** _no doc comment_
- **Calls (crate):** `body_paragraphs` (comparer/finalize.rs:14935) L15180,15191; `inside_table` (comparer/finalize.rs:14966) L15181; `markup_text` (comparer/finalize.rs:14940) L15184; `has_deleted_mark` (comparer/finalize.rs:15162) L15192; `Dom::element` (xmllinq/mod.rs:655) L15192; `content_revisions` (comparer/finalize.rs:15211) L15195; `Dom::descendants` (xmllinq/mod.rs:668) L15199; `Dom::new_element` (xmllinq/mod.rs:478) L15202,15203; `rev_el` (comparer/finalize.rs:126) L15204; `Dom::add` (xmllinq/mod.rs:1074) L15205,15206; `Dom::add_first` (xmllinq/mod.rs:1101) L15207
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.insert`, `.is_empty`, `.is_some`, `.iter`, `.to_string`, `.trim`, `HashSet::new`, `Some`, `W::del`, `W::name`, `W::p_pr`, `W::r_pr`
- **Called by:** `align_remaining_ladder_rungs` (comparer/finalize.rs:14908); `coverage_round_two_tests::tabbed_title_mark_matches_original_body_and_excludes_table_only_titles` (comparer/finalize.rs:18089)

#### `content_revisions`  (L15211-15228)
`fn content_revisions(dom: &Dom, paragraph: NodeId) -> (bool, bool, Vec<String>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L15212; `Dom::descendants` (xmllinq/mod.rs:668) L15216; `inside` (comparer/finalize.rs:14956) L15217; `Dom::name_is` (xmllinq/mod.rs:535) L15220,15222; `markup_text` (comparer/finalize.rs:14940) L15224
- **Calls (std/external/unresolved):** `.is_some_and`, `.push`, `.to_string`, `.trim`, `Vec::new`, `W::del`, `W::ins`, `W::p_pr`
- **Called by:** `mark_tabbed_deleted_titles` (comparer/finalize.rs:15172)

#### `spacing_only_change`  (L15230-15241)
`fn spacing_only_change(dom: &Dom, change: NodeId) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L15231,15235; `Dom::name_is` (xmllinq/mod.rs:535) L15236
- **Calls (std/external/unresolved):** `.len`, `Some`, `W::name`, `W::p_pr`
- **Called by:** `unmerge_comment_list_item` (comparer/finalize.rs:15255)

#### `append_empty_ppr_change`  (L15243-15253)
`fn append_empty_ppr_change( dom: &mut Dom, ppr: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** _no doc comment_
- **Calls (crate):** `rev_el` (comparer/finalize.rs:126) L15249; `Dom::new_element` (xmllinq/mod.rs:478) L15250; `Dom::add` (xmllinq/mod.rs:1074) L15251,15252
- **Calls (std/external/unresolved):** `W::p_pr_change`, `W::p_pr`
- **Called by:** `unmerge_comment_list_item` (comparer/finalize.rs:15255); `move_spacing_onto_bare_deletion` (comparer/finalize.rs:15386); `stamp_trailing_numbering` (comparer/finalize.rs:15468)

#### `unmerge_comment_list_item`  (L15255-15311)
`fn unmerge_comment_list_item( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** _no doc comment_
- **Calls (crate):** `body_paragraphs` (comparer/finalize.rs:14935) L15263; `Dom::elements` (xmllinq/mod.rs:640) L15265,15266; `Dom::descendants` (xmllinq/mod.rs:668) L15271; `Dom::name_is` (xmllinq/mod.rs:535) L15274; `Dom::element` (xmllinq/mod.rs:655) L15278,15281,15284,15290; `spacing_only_change` (comparer/finalize.rs:15230) L15287; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L15293,15294; `Dom::add` (xmllinq/mod.rs:1074) L15295,15299,15300,15304,15306,15308; `Dom::remove` (xmllinq/mod.rs:1118) L15296,15307; `Dom::new_element` (xmllinq/mod.rs:478) L15297,15302,15303; `rev_el` (comparer/finalize.rs:126) L15298; `append_empty_ppr_change` (comparer/finalize.rs:15243) L15305; `Dom::add_after_self` (xmllinq/mod.rs:1147) L15309
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_none`, `.is_some`, `.len`, `Some`, `W::del`, `W::ins`, `W::name`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r_pr`
- **Called by:** `align_remaining_ladder_rungs` (comparer/finalize.rs:14908); `coverage_round_two_tests::comment_list_unmerge_retains_numbering_and_clones_spacing_to_both_parts` (comparer/finalize.rs:17653); `coverage_round_two_tests::comment_unmerge_rejects_incomplete_or_substantive_property_history` (comparer/finalize.rs:17708)

#### `change_is_empty`  (L15313-15319)
`fn change_is_empty(dom: &Dom, change: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L15314; `Dom::elements` (xmllinq/mod.rs:640) L15317,15318
- **Calls (std/external/unresolved):** `.is_empty`, `.len`, `Some`, `W::p_pr_change`, `W::p_pr`
- **Called by:** `drop_duplicate_normal_style` (comparer/finalize.rs:15321); `coverage_round_two_tests::mixed_spacing_moves_to_blank_and_preserves_existing_history` (comparer/finalize.rs:17302); `coverage_round_two_tests::mixed_numbering_moves_before_last_deletion_mark_or_history` (comparer/finalize.rs:17463); `coverage_round_two_tests::comment_list_unmerge_retains_numbering_and_clones_spacing_to_both_parts` (comparer/finalize.rs:17653); `coverage_round_two_tests::bare_deletion_receives_unique_revised_spacing_with_auto_line_rule` (comparer/finalize.rs:17739); `coverage_round_two_tests::trailing_normal_empty_adopts_source_numbering_and_empty_history` (comparer/finalize.rs:17802); `coverage_round_two_tests::heading_spacing_promotion_requires_prior_live_heading_and_preserves_old_shell` (comparer/finalize.rs:17983); `coverage_round_two_tests::live_heading_history_is_added_only_when_last_history_is_empty` (comparer/finalize.rs:18151)

#### `drop_duplicate_normal_style`  (L15321-15362)
`fn drop_duplicate_normal_style(dom: &mut Dom, root: NodeId)`

- **Does:** _no doc comment_
- **Calls (crate):** `body_paragraphs` (comparer/finalize.rs:14935) L15323; `Dom::element` (xmllinq/mod.rs:655) L15325,15354,15356; `Dom::elements` (xmllinq/mod.rs:640) L15328,15339,15342; `Dom::name_is` (xmllinq/mod.rs:535) L15329,15335; `change_is_empty` (comparer/finalize.rs:15313) L15339; `markup_text` (comparer/finalize.rs:14940) L15346,15352; `has_deleted_mark` (comparer/finalize.rs:15162) L15353; `Dom::remove` (xmllinq/mod.rs:1118) L15359
- **Calls (std/external/unresolved):** `.any`, `.copied`, `.enumerate`, `.find`, `.is_empty`, `.is_none_or`, `.is_none`, `.iter`, `.len`, `.trim`, `Some`, `W::del`, `W::ins`, `W::name`, `W::p_pr_change`, `W::p_pr`
- **Called by:** `align_remaining_ladder_rungs` (comparer/finalize.rs:14908); `coverage_round_two_tests::duplicate_deleted_default_style_drops_only_after_matching_marked_text` (comparer/finalize.rs:18127)

#### `spacing_signature`  (L15364-15374)
`fn spacing_signature(dom: &Dom, paragraph: NodeId) -> Option<Vec<(String, String)>>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L15365,15366; `Dom::attributes` (xmllinq/mod.rs:899) L15367; `XName::local_name` (xmllinq/mod.rs:208) (by name) L15370
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.is_empty`, `.map`, `.sort`, `.to_string`, `Some`, `W::name`, `W::p_pr`
- **Called by:** `move_spacing_onto_bare_deletion` (comparer/finalize.rs:15386)

#### `direct_spacing`  (L15376-15379)
`fn direct_spacing(dom: &Dom, paragraph: NodeId) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L15377,15378
- **Calls (std/external/unresolved):** `W::name`, `W::p_pr`
- **Called by:** `move_spacing_onto_bare_deletion` (comparer/finalize.rs:15386)

#### `move_spacing_onto_bare_deletion`  (L15386-15466)
`fn move_spacing_onto_bare_deletion( dom: &mut Dom, root: NodeId, revised: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** _no doc comment_
- **Calls (crate):** `body_paragraphs` (comparer/finalize.rs:14935) L15394,15420; `spacing_signature` (comparer/finalize.rs:15364) L15395,15435; `markup_text` (comparer/finalize.rs:14940) L15404,15428,15447; `direct_spacing` (comparer/finalize.rs:15376) L15409; `Dom::elements` (xmllinq/mod.rs:640) L15423,15424,15442,15445; `Dom::element` (xmllinq/mod.rs:655) L15448; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L15456; `Dom::attribute` (xmllinq/mod.rs:874) L15459; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L15460; `Dom::new_element` (xmllinq/mod.rs:478) L15462; `Dom::add` (xmllinq/mod.rs:1074) L15463; `append_empty_ppr_change` (comparer/finalize.rs:15243) L15464; `Dom::add_first` (xmllinq/mod.rs:1101) L15465
- **Calls (std/external/unresolved):** `.any`, `.as_ref`, `.entry`, `.enumerate`, `.get`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.len`, `.or_default`, `.push`, `.to_string`, `.trim`, `HashMap::new`, `Some`, `Vec::new`, `W::del`, `W::ins`, `W::name`, `W::p_pr`
- **Called by:** `align_remaining_ladder_rungs` (comparer/finalize.rs:14908); `coverage_round_two_tests::bare_deletion_receives_unique_revised_spacing_with_auto_line_rule` (comparer/finalize.rs:17739); `coverage_round_two_tests::bare_deletion_spacing_rejects_ambiguity_reverse_order_and_equal_spacing` (comparer/finalize.rs:17777)

#### `stamp_trailing_numbering`  (L15468-15515)
`fn stamp_trailing_numbering( dom: &mut Dom, root: NodeId, revised: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** _no doc comment_
- **Calls (crate):** `body_paragraphs` (comparer/finalize.rs:14935) L15477,15486,15505; `Dom::element` (xmllinq/mod.rs:655) L15480,15481,15493; `markup_text` (comparer/finalize.rs:14940) L15490,15507; `Dom::elements` (xmllinq/mod.rs:640) L15496; `Dom::name_is` (xmllinq/mod.rs:535) L15497; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L15502; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L15511; `Dom::remove` (xmllinq/mod.rs:1118) L15512; `Dom::add_first` (xmllinq/mod.rs:1101) L15513; `append_empty_ppr_change` (comparer/finalize.rs:15243) L15514
- **Calls (std/external/unresolved):** `.and_then`, `.find_map`, `.into_iter`, `.is_empty`, `.is_some_and`, `.last`, `.len`, `.trim`, `Some`, `W::name`, `W::p_pr`, `W::val`
- **Macros:** `matches!`
- **Called by:** `align_remaining_ladder_rungs` (comparer/finalize.rs:14908); `coverage_round_two_tests::trailing_normal_empty_adopts_source_numbering_and_empty_history` (comparer/finalize.rs:17802); `coverage_round_two_tests::trailing_numbering_preserves_nonempty_custom_and_source_final_blanks` (comparer/finalize.rs:17835)

## Test functions (500)

- `residual_short_label_zip` (L6923): M369 — greedy order-preserving zip of residual short-label pure-I paragraphs (e.g.
- `fold_short_label_ins_into_del` (L7053): Fold pure-I short label into pure-D list residual.
- `short_label_fixture_settings` (L7183): M375 — residual short-label MIX free-meshes the shared trailing label as unrevised EQ.
- `mesh_short_label_shared_eq` (L7199): _no doc comment_
- `fold_midstream_demo_title_into_numbered_heading` (L12567): M143 — fold a mid-stream pure-D short "Demo" title among the pure-I body of a long revised document into the first numbered heading instead of leaving it as a free pure-D.
- `coverage_tests::xml` (L15521): _no doc comment_
- `coverage_tests::paragraphs` (L15531): _no doc comment_
- `coverage_tests::text` (L15535): _no doc comment_
- `coverage_tests::revised` (L15542): _no doc comment_
- `coverage_tests::assert_unchanged` (L15549): _no doc comment_
- `coverage_tests::replacement_classification_is_transparent_only_for_hyperlinks` (L15557): _no doc comment_
- `coverage_tests::row_move_classification_requires_matching_structural_paragraph_marks` (L15629): _no doc comment_
- `coverage_tests::pure_and_mixed_paragraph_classification_preserves_live_content` (L15661): _no doc comment_
- `coverage_tests::classification_cache_hits_and_resets_agree_with_uncached_results` (L15698): _no doc comment_
- `<Reset as Drop>::drop` (L15701): _no doc comment_
- `coverage_tests::section_and_list_residual_label_boundaries` (L15721): _no doc comment_
- `coverage_tests::token_splitting_preserves_separators_and_original_case` (L15744): _no doc comment_
- `coverage_tests::relatedness_and_short_title_gates_use_tokens_not_substrings` (L15785): _no doc comment_
- `coverage_tests::deleted_text_conversion_stops_at_nested_revision_owners` (L15823): _no doc comment_
- `coverage_tests::field_completeness_requires_balanced_order_not_just_counts` (L15835): _no doc comment_
- `coverage_tests::revision_wrapper_requires_consistent_hyperlink_stamps` (L15859): _no doc comment_
- `coverage_tests::content_controls_unwrap_deepest_first_and_keep_text_order` (L15897): _no doc comment_
- `coverage_tests::residual_control_unwrap_keeps_equal_form_controls` (L15912): _no doc comment_
- `coverage_tests::alternate_content_selects_first_bound_choice_or_fallback` (L15927): _no doc comment_
- `coverage_tests::fold_gate` (L15980): _no doc comment_
- `coverage_tests::document_scale_fold_respects_absolute_ratio_and_fraction_boundaries` (L16014): _no doc comment_
- `coverage_tests::document_scale_list_and_cover_special_cases` (L16075): _no doc comment_
- `coverage_tests::sole_pair_relatedness_boundary_is_inclusive` (L16232): _no doc comment_
- `coverage_tests::cleanup_spacing_removes_only_redundant_mixed_history` (L16254): _no doc comment_
- `coverage_tests::cleanup_default_alignment_keeps_real_layout_history` (L16294): _no doc comment_
- `coverage_tests::cleanup_deleted_spacing_promotes_mid_body_but_preserves_last_history` (L16328): _no doc comment_
- `coverage_tests::short_label_free_mesh_preserves_deleted_prefix_and_inserted_whitespace` (L16390): _no doc comment_
- `coverage_tests::residual_label_zip_adopts_deleted_list_layout_and_moves_empty_spacer` (L16438): _no doc comment_
- `coverage_tests::short_list_label_folds_only_into_empty_deleted_mark_with_deleted_tail` (L16468): _no doc comment_
- `coverage_tests::bookended_free_mesh_rebuilds_word_order_and_preserves_run_properties` (L16518): _no doc comment_
- `coverage_tests::bookended_free_mesh_rejects_ambiguous_and_nontext_anchors` (L16552): _no doc comment_
- `coverage_tests::trailing_spaced_empty_requires_insert_dominance_and_deleted_predecessor` (L16600): _no doc comment_
- `coverage_tests::promoted_list_numbering_uses_inserted_id_and_deleted_level` (L16651): _no doc comment_
- `coverage_tests::field_fold_rejects_crossing_but_allows_balanced_deleted_fields` (L16727): _no doc comment_
- `coverage_round_two_tests::xml` (L16773): _no doc comment_
- `coverage_round_two_tests::ps` (L16784): _no doc comment_
- `coverage_round_two_tests::revision` (L16788): _no doc comment_
- `coverage_round_two_tests::pure` (L16795): _no doc comment_
- `coverage_round_two_tests::mixed` (L16802): _no doc comment_
- `coverage_round_two_tests::settings` (L16810): _no doc comment_
- `coverage_round_two_tests::segments` (L16818): _no doc comment_
- `coverage_round_two_tests::stamped` (L16837): _no doc comment_
- `coverage_round_two_tests::unchanged` (L16846): _no doc comment_
- `coverage_round_two_tests::wholesale_mesh_emits_exact_shared_words_and_styled_residuals` (L16854): _no doc comment_
- `coverage_round_two_tests::wholesale_mesh_preserves_each_authored_terminal_whitespace_and_format` (L16913): _no doc comment_
- `coverage_round_two_tests::wholesale_mesh_preserves_each_authored_terminal_whitespace_and_format::{fn source_projection}` (L16914): _no doc comment_
- `coverage_round_two_tests::wholesale_mesh_without_terminal_whitespace_keeps_exact_supported_word_streams` (L16964): _no doc comment_
- `coverage_round_two_tests::wholesale_mesh_without_terminal_whitespace_keeps_exact_supported_word_streams::{fn owned}` (L16965): _no doc comment_
- `coverage_round_two_tests::wholesale_mesh_keeps_low_overlap_and_nontext_changes_verbatim` (L17030): _no doc comment_
- `coverage_round_two_tests::lcs_rebuild_covers_equal_prefixes_and_both_unmatched_tails` (L17074): _no doc comment_
- `coverage_round_two_tests::title_page_relocation_preserves_six_spacers_and_moves_only_last` (L17114): _no doc comment_
- `coverage_round_two_tests::title_page_relocation_respects_cover_and_layout_boundaries` (L17163): _no doc comment_
- `coverage_round_two_tests::short_head_split_preserves_title_style_and_deleted_wrapper_identity` (L17198): _no doc comment_
- `coverage_round_two_tests::short_head_split_rejects_shared_words_marks_and_equal_interleaving` (L17237): _no doc comment_
- `coverage_round_two_tests::stranded_mark_is_cloned_without_removing_empty_shell` (L17261): _no doc comment_
- `coverage_round_two_tests::stranded_mark_rejects_layout_or_body_content_in_shell` (L17284): _no doc comment_
- `coverage_round_two_tests::mixed_spacing_moves_to_blank_and_preserves_existing_history` (L17302): _no doc comment_
- `coverage_round_two_tests::mixed_spacing_does_not_cross_live_blocks_or_fill_textual_paragraphs` (L17362): _no doc comment_
- `coverage_round_two_tests::heading_spacing_parks_on_last_short_deleted_residual` (L17381): _no doc comment_
- `coverage_round_two_tests::spacing_park_rejects_count_length_and_heading_thresholds` (L17426): _no doc comment_
- `coverage_round_two_tests::mixed_numbering_moves_before_last_deletion_mark_or_history` (L17463): _no doc comment_
- `coverage_round_two_tests::numbering_park_preserves_short_long_and_already_numbered_runs` (L17514): _no doc comment_
- `coverage_round_two_tests::whitespace_fold_adopts_list_layout_and_replaces_inserted_mark` (L17531): _no doc comment_
- `coverage_round_two_tests::whitespace_field_residue_never_inherits_heading_style` (L17576): _no doc comment_
- `coverage_round_two_tests::whitespace_fold_keeps_layout_payloads_long_prose_and_cover_spacers` (L17603): _no doc comment_
- `coverage_round_two_tests::comment_list_unmerge_retains_numbering_and_clones_spacing_to_both_parts` (L17653): _no doc comment_
- `coverage_round_two_tests::comment_unmerge_rejects_incomplete_or_substantive_property_history` (L17708): _no doc comment_
- `coverage_round_two_tests::source` (L17730): _no doc comment_
- `coverage_round_two_tests::bare_deletion_receives_unique_revised_spacing_with_auto_line_rule` (L17739): _no doc comment_
- `coverage_round_two_tests::bare_deletion_spacing_rejects_ambiguity_reverse_order_and_equal_spacing` (L17777): _no doc comment_
- `coverage_round_two_tests::trailing_normal_empty_adopts_source_numbering_and_empty_history` (L17802): _no doc comment_
- `coverage_round_two_tests::trailing_numbering_preserves_nonempty_custom_and_source_final_blanks` (L17835): _no doc comment_
- `coverage_round_two_tests::insertion_split_handles_unicode_run_boundaries_and_multi_leaf_runs` (L17860): _no doc comment_
- `coverage_round_two_tests::carried_insert_and_delete_split_preserves_neighbors_and_attribution` (L17913): _no doc comment_
- `coverage_round_two_tests::heading_spacing_promotion_requires_prior_live_heading_and_preserves_old_shell` (L17983): _no doc comment_
- `coverage_round_two_tests::old_body_alignment_promotes_only_for_aligned_revised_paragraphs` (L18016): _no doc comment_
- `coverage_round_two_tests::leading_deleted_echo_is_removed_only_with_one_or_two_tokens` (L18062): _no doc comment_
- `coverage_round_two_tests::tabbed_title_mark_matches_original_body_and_excludes_table_only_titles` (L18089): _no doc comment_
- `coverage_round_two_tests::duplicate_deleted_default_style_drops_only_after_matching_marked_text` (L18127): _no doc comment_
- `coverage_round_two_tests::live_heading_history_is_added_only_when_last_history_is_empty` (L18151): _no doc comment_
- `coverage_round_three_tests::document` (L18202): Pure in-memory WordprocessingML fixtures. _(from a code comment)_
- `coverage_round_three_tests::rev` (L18214): _no doc comment_
- `coverage_round_three_tests::para` (L18221): _no doc comment_
- `coverage_round_three_tests::bare` (L18228): _no doc comment_
- `coverage_round_three_tests::paragraphs` (L18232): _no doc comment_
- `coverage_round_three_tests::body_segments` (L18236): _no doc comment_
- `coverage_round_three_tests::expect_segments` (L18249): _no doc comment_
- `coverage_round_three_tests::props_xml` (L18259): _no doc comment_
- `coverage_round_three_tests::settings` (L18264): _no doc comment_
- `coverage_round_three_tests::same_after_merge` (L18272): _no doc comment_
- `coverage_round_three_tests::paired_normalization_preserves_prior_pilcrow_provenance_and_only_unmarks_its_own_pair` (L18280): _no doc comment_
- `coverage_round_three_tests::related_one_for_one_adopts_exact_structural_properties_and_inserted_attributes` (L18355): _no doc comment_
- `coverage_round_three_tests::field_replacement_keeps_bare_properties_instead_of_deleted_heading` (L18405): _no doc comment_
- `coverage_round_three_tests::bare_related_pair_does_not_invent_paragraph_properties` (L18425): _no doc comment_
- `coverage_round_three_tests::deleted_then_multiple_inserted_folds_only_last_and_adopts_heading` (L18440): _no doc comment_
- `coverage_round_three_tests::last_inserted_list_adopts_deleted_listparagraph_and_numbering` (L18478): _no doc comment_
- `coverage_round_three_tests::alignment_only_carrier_adopts_deleted_spacing_exactly` (L18508): _no doc comment_
- `coverage_round_three_tests::paired_body_tail_stays_separate_but_nested_stories_still_fold` (L18545): _no doc comment_
- `coverage_round_three_tests::shared_title_head_junction_precedes_remaining_inserted_stream` (L18570): _no doc comment_
- `coverage_round_three_tests::story_tail_fusion_crosses_deleted_table_and_keeps_deleted_layout` (L18603): _no doc comment_
- `coverage_round_three_tests::story_tail_fusion_stops_at_page_breaks_section_breaks_and_live_tables` (L18645): _no doc comment_
- `coverage_round_three_tests::multiple_mark_only_deletions_fold_once_and_preserve_remaining_empty` (L18681): _no doc comment_
- `coverage_round_three_tests::empty_heading_residual_adopts_heading_for_last_list_carrier` (L18711): _no doc comment_
- `coverage_round_three_tests::carried_foreign_insertions_reorder_without_becoming_replacement_deletions` (L18731): _no doc comment_
- `coverage_round_three_tests::unmatched_field_blocks_shared_title_head_junction` (L18748): _no doc comment_
- `coverage_round_three_tests::digits_and_short_list_residuals_keep_their_own_inserted_paragraph` (L18776): _no doc comment_
- `coverage_round_three_tests::long_prose_does_not_inherit_multiword_list_geometry` (L18789): _no doc comment_
- `coverage_round_three_tests::runless_empty_carrier_preserves_cover_spacers_and_long_prose` (L18816): _no doc comment_
- `coverage_round_three_tests::trailing_whitespace_spacing_folds_only_after_contentful_insertions` (L18852): _no doc comment_
- `coverage_round_three_tests::whitespace_carriers_without_mark_shell_adopt_deleted_pilcrow` (L18890): _no doc comment_
- `coverage_round_three_tests::fragmented_two_character_label_meshes_and_retains_whitespace_and_punctuation` (L18912): _no doc comment_
- `coverage_round_three_tests::label_mesh_removes_empty_inserted_wrapper_and_keeps_deleted_list_layout` (L18931): _no doc comment_
- `coverage_round_three_tests::positional_label_zip_keeps_distinct_labels_and_existing_geometry` (L18970): _no doc comment_
- `coverage_round_three_tests::nested_list_interleave_preserves_exact_node_order_and_trailing_anchors` (L18995): _no doc comment_
- `coverage_round_three_tests::deleted_textbox_story_marks_runs_and_pilcrows_in_schema_order` (L19042): _no doc comment_
- `coverage_round_three_tests::nested_deleted_textboxes_use_their_own_anchor_and_preserve_existing_marks` (L19099): _no doc comment_
- `coverage_round_three_tests::deleted_textbox_missing_attribution_uses_pinned_settings` (L19134): _no doc comment_
- `coverage_round_three_tests::bordered_table_geometry_uses_row_revision_source_and_schema_order` (L19156): _no doc comment_
- `coverage_round_three_tests::table_geometry_preserves_explicit_values_and_fills_only_missing_properties` (L19203): _no doc comment_
- `coverage_round_three_tests::large_wholesale_replacement_keeps_all_paragraphs_at_scale_boundary` (L19239): _no doc comment_
- `coverage_round_three_tests::short_wholesale_lists_keep_both_sides_when_old_count_is_twice_new` (L19255): _no doc comment_
- `coverage_round_three_tests::inserted_break_never_folds_into_deleted_drawing_pict_or_object` (L19269): _no doc comment_
- `coverage_round_three_tests::revision_rows_preserve_cell_geometry_and_are_stamped_once` (L19285): _no doc comment_
- `coverage_round_next_tests::document` (L19352): Coverage: lines and branches await the parent's instrumented run. _(from a code comment)_
- `coverage_round_next_tests::revision` (L19363): _no doc comment_
- `coverage_round_next_tests::pure` (L19370): _no doc comment_
- `coverage_round_next_tests::marked` (L19374): _no doc comment_
- `coverage_round_next_tests::settings` (L19381): _no doc comment_
- `coverage_round_next_tests::same_tree` (L19391): Compare the complete tree, including every attribute, text leaf and child position. _(from a code comment)_
- `coverage_round_next_tests::expect_body` (L19421): _no doc comment_
- `coverage_round_next_tests::unchanged` (L19426): _no doc comment_
- `coverage_round_next_tests::digit_title_split_moves_all_leading_fragments_and_keeps_original_properties` (L19434): _no doc comment_
- `coverage_round_next_tests::digit_split_rejects_nonascii_empty_and_later_insertions` (L19454): _no doc comment_
- `coverage_round_next_tests::trailing_insertion_peel_has_inclusive_token_bounds_and_keeps_punctuation` (L19485): _no doc comment_
- `coverage_round_next_tests::trailing_insertion_peel_rejects_exact_guard_boundaries` (L19508): _no doc comment_
- `coverage_round_next_tests::trailing_deletion_peel_truth_table_for_shared_long_and_copula_paths` (L19555): _no doc comment_
- `coverage_round_next_tests::trailing_deletion_peel_truth_table_for_shared_long_and_copula_paths::{fn projected_body}` (L19556): _no doc comment_
- `coverage_round_next_tests::short_deletion_restore_checks_filename_and_alnum_length_boundaries` (L19657): _no doc comment_
- `coverage_round_next_tests::demo_fold_strips_only_revised_pilcrow_and_uses_nearest_numbered_heading` (L19695): _no doc comment_
- `coverage_round_next_tests::demo_title_fold_history_preserves_complete_old_layout_and_current_attribution` (L19733): _no doc comment_
- `coverage_round_next_tests::demo_fold_heading_text_limits_and_table_barriers_are_exact` (L19778): _no doc comment_
- `coverage_round_next_tests::title_spacers_are_kept_once_and_prose_spacers_are_removed` (L19837): _no doc comment_
- `coverage_round_next_tests::list_contamination_cleanup_keeps_real_lists_and_alignment` (L19866): _no doc comment_
- `coverage_round_next_tests::list_snug_spacing_requires_three_identical_short_tokens_and_respects_schema_tail` (L19894): _no doc comment_
- `coverage_round_next_tests::trailing_blank_spacing_records_history_only_when_source_differs` (L19923): _no doc comment_
- `coverage_round_next_tests::table_chrome_adds_look_before_history_and_handles_all_row_positions` (L19966): _no doc comment_
- `coverage_round_next_tests::comment_chrome_fills_each_missing_property_independently_and_ignores_orphans` (L19988): _no doc comment_
- `coverage_round_next_tests::deleted_textbox_attribution_is_inherited_per_attribute_and_runs_join_across_text` (L20032): _no doc comment_
- `coverage_round_next_tests::paragraph_deletion_mark_inserts_before_each_schema_tail_and_preserves_format` (L20064): _no doc comment_
- `coverage_round_next_tests::structural_property_classifiers_ignore_history_and_require_one_alignment` (L20085): _no doc comment_
- `coverage_round_next_tests::conjoin_property_selection_retains_format_and_does_not_duplicate_history` (L20110): _no doc comment_
- `coverage_round_next_tests::moved_empty_field_becomes_a_complex_field_in_the_move` (L20166): An empty field is a whole atom (no run), so the run-level status pass never wrapped it: a deleted `PAGE` stayed live in its deleted paragraph and Word 16.115 hung on the footer (b15, 0.12.0 sample).
- `coverage_round_next_tests::deleted_or_inserted_empty_field_becomes_a_complex_field_in_the_revision` (L20205): _no doc comment_
- `coverage_round_next_tests::default_spacing_strips_only_the_selected_source_default_and_exact_shape` (L20237): _no doc comment_
- `coverage_round_next_tests::deleted_heading_spacing_threshold_keeps_missing_metrics_and_real_styles` (L20272): _no doc comment_
- `coverage_round_next_tests::unrecorded_spacing_guards_preserve_all_explicit_vertical_metrics` (L20299): _no doc comment_
- `coverage_round_next_tests::deleted_history_flattening_uses_both_field_and_text_fingerprint` (L20346): _no doc comment_
- `coverage_round_next_tests::predelete_conversion_missing_attribution_defaults_without_changing_neighbor_runs` (L20397): _no doc comment_
- `coverage_round_next_tests::short_label_mesh_rejects_empty_oversized_and_mismatching_labels_verbatim` (L20435): _no doc comment_
- `coverage_round_next_tests::label_fold_creates_missing_properties_without_inventing_source_geometry` (L20458): _no doc comment_
- `coverage_round_next_tests::for_word_peel_preserves_run_format_and_uses_equal_format_for_shared_connector` (L20485): _no doc comment_
- `coverage_round_next_tests::for_word_guard_matrix_requires_residual_and_leading_deleted_carrier` (L20510): _no doc comment_
- `coverage_round_next_tests::borrowed_period_moves_to_finished_side_and_finishes_the_continuation` (L20541): _no doc comment_
- `coverage_round_next_tests::compatibility_defaults_keep_nonzero_vertical_margins_and_existing_look` (L20582): _no doc comment_
- `coverage_round_next_tests::shared_title_mesh_rejects_author_mismatch_repeats_and_nonfinal_shared_tokens` (L20603): _no doc comment_
- `coverage_round_next_tests::revised_complex_deletion_stays_pending_while_original_restores_nested_text` (L20652): _no doc comment_
- `coverage_round_next_tests::strict_shape_repair_stops_at_nested_drawing_and_keeps_bad_percent_text` (L20667): _no doc comment_
- `coverage_round_next_tests::block_reordering_requires_uniform_revision_rows_and_nonempty_sdt_content` (L20685): _no doc comment_
- `fragmented_label_regression_tests::projection` (L20732): _no doc comment_
- `fragmented_label_regression_tests::shared_label_fragments_preserve_both_revision_projections` (L20747): _no doc comment_
- `fragmented_label_regression_tests::unsupported_fragment_tail_cannot_partially_erase_a_deletion` (L20769): _no doc comment_
- `fragmented_label_regression_tests::label_mesh_preserves_punctuation_styles_and_nontext_payloads` (L20780): _no doc comment_
- `fragmented_label_regression_tests::interleaved_revision_blocks_preserve_insertion_order` (L20811): _no doc comment_
- `fragmented_label_regression_tests::shared_label_mesh_keeps_nested_revision_hyperlink_and_control_ownership` (L20827): _no doc comment_
- `fragmented_label_regression_tests::short_label_mesh_keeps_unrelated_empty_run_properties` (L20851): _no doc comment_
- `fragmented_label_regression_tests::paragraph_mark_revision_does_not_block_plain_shared_label_meshing` (L20867): _no doc comment_
- `deep_production_path_tests::document` (L20893): _no doc comment_
- `deep_production_path_tests::revision` (L20903): _no doc comment_
- `deep_production_path_tests::para` (L20910): _no doc comment_
- `deep_production_path_tests::empty` (L20917): _no doc comment_
- `deep_production_path_tests::live` (L20923): _no doc comment_
- `deep_production_path_tests::blocks` (L20927): _no doc comment_
- `deep_production_path_tests::projection` (L20936): Independent source projections: paragraph properties are not body text, and a rejected/accepted revision excludes its entire nested subtree. _(from a code comment)_
- `deep_production_path_tests::views` (L20950): _no doc comment_
- `deep_production_path_tests::body_revisions` (L20954): _no doc comment_
- `deep_production_path_tests::properties` (L20969): _no doc comment_
- `deep_production_path_tests::segments` (L20973): _no doc comment_
- `deep_production_path_tests::expect_segments` (L20986): _no doc comment_
- `deep_production_path_tests::merge` (L20996): _no doc comment_
- `deep_production_path_tests::unchanged` (L21022): _no doc comment_
- `deep_production_path_tests::legal_and_memo_midsplices_keep_each_source_paragraph_in_order` (L21032): _no doc comment_
- `deep_production_path_tests::nested_level_parsing_controls_interleave_without_changing_markup` (L21069): _no doc comment_
- `deep_production_path_tests::legal_midsplice_count_boundaries_reorder_outside_the_guard` (L21106): _no doc comment_
- `deep_production_path_tests::prior_nested_cluster_keeps_short_labels_but_allows_a_longer_carrier` (L21130): _no doc comment_
- `deep_production_path_tests::long_content_cannot_reach_back_across_an_empty_tail_without_a_head_match` (L21163): _no doc comment_
- `deep_production_path_tests::math_shells_are_fold_carriers_and_plain_layout_empties_are_preserved` (L21231): _no doc comment_
- `deep_production_path_tests::inserted_cover_tail_and_short_residual_exceptions_are_distinct` (L21266): _no doc comment_
- `deep_production_path_tests::deletion_first_short_residual_and_field_carrier_keep_source_properties` (L21301): _no doc comment_
- `deep_production_path_tests::empty_deleted_tail_is_not_consumed_after_long_inserted_streams` (L21342): _no doc comment_
- `deep_production_path_tests::short_label_long_prose_gate_distinguishes_plain_demo_and_ooxml_intros` (L21355): _no doc comment_
- `deep_production_path_tests::styled_inserted_titles_keep_unrelated_short_base_titles_except_demos` (L21413): _no doc comment_
- `deep_production_path_tests::long_demo_stream_gate_has_an_exact_ten_paragraph_boundary` (L21442): _no doc comment_
- `deep_production_path_tests::heading_skip_is_limited_to_short_multitoken_headings_and_nonlist_carriers` (L21458): _no doc comment_
- `deep_production_path_tests::list_carriers_override_the_long_prose_list_skip_and_retain_their_geometry` (L21504): _no doc comment_
- `deep_production_path_tests::wholesale_scale_gate_preserves_large_math_and_drawing_replacements` (L21549): _no doc comment_
- `deep_production_path_tests::midstory_checklist_cell_guard_distinguishes_titles_at_a_weak_head_junction` (L21599): _no doc comment_
- `deep_production_path_tests::whitespace_fold_keeps_a_whitespace_only_deleted_source_paragraph` (L21626): _no doc comment_
- `deep_production_path_tests::whitespace_mark_shell_variants_keep_existing_format_without_inventing_a_mark` (L21634): _no doc comment_
- `deep_production_path_tests::whitespace_spacing_guard_searches_past_noninserted_predecessors` (L21682): _no doc comment_
- `deep_production_path_tests::spacer_context` (L21709): _no doc comment_
- `deep_production_path_tests::table_title_spacers_stamp_only_new_marks_and_keep_existing_empty_properties` (L21719): _no doc comment_
- `deep_production_path_tests::title_spacers_require_unique_revised_two_blank_pretable_context` (L21817): _no doc comment_
- `deep_production_path_tests::restored_title_spacers_preserve_each_revised_blank_template` (L21909): _no doc comment_
- `deep_production_path_tests::title_spacer_context_declines_owned_nonlayout_payload` (L21953): _no doc comment_
- `deep_production_path_tests::title_spacer_injection_requires_a_paired_table_and_a_truly_blank_separator` (L21999): _no doc comment_
- `deep_production_path_tests::three_paragraph_rotation_trades_full_properties_and_preserves_each_revision` (L22039): _no doc comment_
- `deep_production_path_tests::rotation_rejects_interleaved_first_runs_and_nonfinal_middle_deletions` (L22097): _no doc comment_
- `deep_production_path_tests::leading_subtitle_fold_moves_multiple_wrappers_and_retains_deleted_layout` (L22149): _no doc comment_
- `deep_production_path_tests::leading_subtitle_gate_requires_disjoint_significant_tokens_and_eight_token_cap` (L22188): _no doc comment_
- `deep_production_path_tests::shared_title_mesh_executes_wholesale_and_confetti_rebuilds_with_exact_views` (L22226): _no doc comment_
- `deep_production_path_tests::bookended_mesh_handles_boundary_anchors_and_the_forty_token_limit` (L22301): _no doc comment_
- `deep_production_path_tests::connector_peel_preserves_both_sources_and_falls_back_to_inserted_run_format` (L22353): _no doc comment_
- `deep_production_path_tests::splice_fragment` (L22408): _no doc comment_
- `deep_production_path_tests::short_residual_splice_prefers_tip_then_toc_and_moves_only_the_short_numbered_title` (L22426): _no doc comment_
- `deep_production_path_tests::short_title_splice_crosses_only_tables_whose_every_row_is_structurally_inserted` (L22464): _no doc comment_
- `deep_production_path_tests::short_title_splice_crosses_only_tables_whose_every_row_is_structurally_inserted::{fn projected}` (L22465): _no doc comment_
- `deep_production_path_tests::short_residual_splice_majority_and_length_boundaries_do_not_invent_reordering` (L22531): _no doc comment_
- `deep_production_path_tests::revised_internal_hyperlink_fields_preserve_result_text_and_revision_attribution` (L22582): _no doc comment_
- `deep_production_path_tests::deletion_peel_keeps_retained_source_boundary_and_inserted_continuations` (L22638): _no doc comment_
- `deep_production_path_tests::deletion_peel_keeps_retained_source_boundary_and_inserted_continuations::{fn paragraphs}` (L22639): _no doc comment_
- `deep_production_path_tests::final_deletion_peel_transfers_boundary_only_with_equal_properties_and_text_payload` (L22704): _no doc comment_
- `deep_production_path_tests::final_deletion_peel_transfers_boundary_only_with_equal_properties_and_text_payload::{fn projected}` (L22705): _no doc comment_
- `deep_production_path_tests::deletion_peel_keeps_nontext_only_original_anchors_in_their_source_paragraph` (L22795): _no doc comment_
- `deep_production_path_tests::deletion_peel_keeps_nontext_only_original_anchors_in_their_source_paragraph::{fn projected}` (L22796): _no doc comment_
- `deep_production_path_tests::deletion_peel_keeps_punctuation_and_whitespace_only_original_anchors` (L22854): _no doc comment_
- `deep_production_path_tests::deletion_peel_keeps_punctuation_and_whitespace_only_original_anchors::{fn projection}` (L22855): _no doc comment_
- `deep_production_path_tests::deletion_peel_keeps_original_closing_boundary_when_all_original_words_are_deleted` (L22901): _no doc comment_
- `deep_production_path_tests::deletion_peel_keeps_original_closing_boundary_when_all_original_words_are_deleted::{fn projection}` (L22902): _no doc comment_
- `deep_production_path_tests::wholesale_deleted_body_peel_transfers_its_retained_closing_boundary` (L22946): _no doc comment_
- `deep_production_path_tests::wholesale_deleted_body_peel_transfers_its_retained_closing_boundary::{fn projection}` (L22947): _no doc comment_
- `deep_production_path_tests::deletion_peel_does_not_move_an_original_suffix_past_equal_nonperiod_text` (L22999): _no doc comment_
- `deep_production_path_tests::deletion_peel_does_not_move_an_original_suffix_past_equal_nonperiod_text::{fn projection}` (L23000): _no doc comment_
- `deep_production_path_tests::borrowed_period_peel_keeps_the_original_period_run_formatting` (L23046): _no doc comment_
- `deep_production_path_tests::borrowed_period_peel_keeps_the_original_period_run_formatting::{fn projection}` (L23047): _no doc comment_
- `deep_production_path_tests::same_format_borrowed_period_peel_keeps_literal_text_and_character_properties` (L23083): _no doc comment_
- `deep_production_path_tests::same_format_borrowed_period_peel_keeps_literal_text_and_character_properties::{fn projection}` (L23084): _no doc comment_
- `deep_production_path_tests::nested_interleave_keeps_mark_only_items_and_defaults_invalid_terminal_levels` (L23143): _no doc comment_
- `deep_production_path_tests::nested_interleave_uniform_tokens_are_case_insensitive_and_late_mismatches_reorder` (L23186): _no doc comment_
- `deep_production_path_tests::nested_interleave_needs_a_complete_nested_cluster_followed_by_a_top_level_item` (L23216): _no doc comment_
- `deep_production_path_tests::final_story_fusion_keeps_the_original_deleted_tail_and_its_metadata` (L23237): _no doc comment_
- `deep_production_path_tests::explicit_page_breaks_block_final_story_fusion_on_either_source_side` (L23288): _no doc comment_
- `deep_production_path_tests::numbered_wholesale_gate_requires_the_last_new_item_and_half_of_old_items_numbered` (L23329): _no doc comment_
- `deep_production_path_tests::punctuation_only_items_are_contentful_and_can_supply_empty_token_relatedness` (L23366): _no doc comment_
- `deep_production_path_tests::late_tip_overrides_earlier_toc_and_numbered_splice_anchors` (L23393): _no doc comment_
- `deep_production_path_tests::plain_label_mesh_may_share_across_multiple_contiguous_revision_wrappers` (L23430): _no doc comment_
- `deep_production_path_tests::suffix_only_bookend_mesh_uses_pinned_defaults_and_keeps_the_original_suffix` (L23463): _no doc comment_
- `deep_production_path_tests::complete_field_carriers_keep_title_deletions_as_mark_only_properties` (L23502): _no doc comment_
- `row_move_ownership_regressions::moved_row` (L23542): _no doc comment_
- `row_move_ownership_regressions::moving_cell_content_without_paragraph_ownership_keeps_existing_rows` (L23552): _no doc comment_
- `row_move_ownership_regressions::moved_content_with_matching_paragraph_mark_tracks_row_lifecycle` (L23596): _no doc comment_
- `parked_paragraph_history_regressions::parked_layout_restores_properties_that_remain_live_when_rejected` (L23617): _no doc comment_
- `closing_layout_history_regressions::already_revised_live_closing_layout_still_records_original_properties` (L23679): _no doc comment_
- `split_paragraph_boundary_regressions::mixed_inserted_boundary_joins_original_text_and_closing_layout_on_reject` (L23752): _no doc comment_
- `row_nontext_ownership_regressions::document` (L23815): _no doc comment_
- `row_nontext_ownership_regressions::unchanged_nontext_payload_prevents_a_moved_neighbor_from_owning_its_row` (L23826): _no doc comment_
- `row_nontext_ownership_regressions::package` (L23867): _no doc comment_
- `row_nontext_ownership_regressions::row_payload` (L23882): _no doc comment_
- `row_nontext_ownership_regressions::public_compare_keeps_unchanged_tab_and_break_cells_when_text_moves_into_their_row` (L23895): _no doc comment_
- `short_label_format_history_tests::document` (L23964): _no doc comment_
- `short_label_format_history_tests::properties` (L23974): _no doc comment_
- `short_label_format_history_tests::resolved` (L23984): _no doc comment_
- `short_label_format_history_tests::shared_label_tracks_complete_old_and_revised_run_formats_and_layout` (L24013): _no doc comment_
- `short_label_format_history_tests::formerly_unsupported_bold_to_italic_label_preserves_the_exact_fixture` (L24086): _no doc comment_
- `short_label_format_history_tests::fragmented_label_may_share_only_with_consistent_properties_on_each_side` (L24111): _no doc comment_
- `short_label_format_history_tests::label_property_boundaries_history_and_opaque_properties_stay_unchanged` (L24130): _no doc comment_
- `short_label_format_history_tests::configured_label_tracking_preserves_sources_and_respects_disabled_changes` (L24170): _no doc comment_
- `coverage_final_batch_finalize_tests::xml` (L24255): _no doc comment_
- `coverage_final_batch_finalize_tests::visible` (L24265): _no doc comment_
- `coverage_final_batch_finalize_tests::carried_revision_matrix_preserves_order_formatting_and_original_attribution` (L24274): _no doc comment_
- `coverage_final_batch_finalize_tests::external_link_rewrite_matrix_requires_new_target_and_wholly_inserted_content` (L24402): _no doc comment_
- `coverage_final_batch_finalize_tests::hyperlink_hoisting_preserves_empty_wrappers_and_run_order_across_revisions` (L24482): _no doc comment_
- `coverage_final_batch_finalize_tests::empty_field_in_a_revision_keeps_its_flags_and_padded_code` (L24566): _no doc comment_
- `coverage_final_batch_finalize_tests::inherited_invalidity_repairs_only_wml_schema_defects` (L24583): _no doc comment_
- `coverage_final_batch_finalize_tests::compatibility_bindings_keep_conflicts_and_recognize_attribute_only_extensions` (L24643): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::properties` (L24709): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::revision_para` (L24728): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::authored_paragraph` (L24769): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::document` (L24783): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::projected` (L24813): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::text` (L24885): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::compact_source_difference` (L24919): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::exercise` (L24981): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::full_production_compare_restores_original_and_revised_run_and_paragraph_formatting` (L25217): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::public_cell_replacement_keeps_both_source_paragraph_properties_at_a_blank_tail` (L25280): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::live_blank_cell_tail_keeps_independent_replacement_paragraph_owners` (L25336): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::a_related_replacement_before_live_content_uses_a_paired_unmarked_pilcrow` (L25377): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::multiple_original_paragraphs_before_a_live_blank_control_tail_keep_exact_phase_payloads` (L25428): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::multiple_revised_paragraphs_with_distinct_tail_properties_keep_word_deleted_carrier` (L25560): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::guarded_multi_original_complete_projection` (L25621): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::guarded_multi_original_complete_projection::{fn semantic}` (L25626): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::multiple_original_blank_tail_checks_every_later_pilcrow_provenance` (L25666): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::multiple_original_blank_tail_preserves_first_original_section_and_property_history` (L25801): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::multiple_revised_blank_tail_pairs_only_the_matching_final_revised_paragraph` (L26022): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::public_compare_multi_original_blank_tail_recovers_complete_clean_sources` (L26112): A raw D/I phase fixture is not an untracked document: its deleted paragraph marks can make revision projection promote the shared blank out of sdtContent. _(from a code comment)_
- `coverage_real_revision_merge_matrix_tests::public_compare_multi_original_blank_tail_recovers_complete_clean_sources::{fn clean_paragraph}` (L26113): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::public_compare_multi_original_blank_tail_recovers_complete_clean_sources::{fn package}` (L26118): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::public_compare_multi_original_blank_tail_recovers_complete_clean_sources::{fn complete_source}` (L26148): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::a_single_replacement_before_a_live_blank_control_tail_keeps_a_paired_pilcrow` (L26241): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::a_sole_original_and_multiple_revised_paragraphs_keep_live_blank_tail_ownership` (L26276): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::a_sole_original_and_multiple_revised_paragraphs_keep_live_blank_tail_ownership::{fn count_projected_paragraphs}` (L26277): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::live_blank_cell_tail_with_inert_run_keeps_exact_pilcrow_owners` (L26365): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::paired_property_comparison_never_grows_the_persistent_dom_or_changes_sources` (L26414): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::empty_pilcrow_proof_retains_inert_formatting_and_declines_owned_payloads` (L26464): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::public_two_original_one_revised_windows_keep_complete_sources_after_all_word_phases` (L26519): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::word_m98b_spacing_and_history_match_the_complete_format_golden` (L26622): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::repeated_authored_prefixes_keep_deleted_paragraphs_before_their_unchanged_table` (L26657): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::repeated_authored_prefixes_keep_deleted_paragraphs_before_their_unchanged_table::{fn owned}` (L26658): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::unshared_wrap_gap_keeps_each_sources_preserved_terminal_space` (L26763): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::unshared_wrap_gap_keeps_each_sources_preserved_terminal_space::{fn owned}` (L26764): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::organic_source_snapshot` (L26897): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::organic_paragraph` (L26980): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::wholesale_boundary_package` (L27024): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::wholesale_boundary_paragraph` (L27033): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::whole_story_title_label_heading_and_list_boundaries_keep_source_ownership` (L27055): These packages intentionally omit organic_package's equal anchors. _(from a code comment)_
- `coverage_real_revision_merge_matrix_tests::terminal_layout_paragraph` (L27473): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::public_terminal_field_math_and_break_boundaries_preserve_all_source_owners` (L27505): M360 field residues and M431 layout breaks have source payload even without cached text. _(from a code comment)_
- `coverage_real_revision_merge_matrix_tests::public_cross_paragraph_field_ends_keep_exact_source_instruction_and_cache_owners` (L27727): TOC-style complex fields legitimately cross paragraph boundaries. _(from a code comment)_
- `coverage_real_revision_merge_matrix_tests::organic_table` (L27902): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::organic_table_with_rows` (L27906): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::organic_package` (L27934): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::organic_heterogeneous_public_packages_preserve_owned_sources_across_modes` (L28008): These literal clean packages exercise heterogeneous organic production paths rather than supplying private D/I intermediate states.
- `coverage_real_revision_merge_matrix_tests::literal_legacy_table_geometry` (L28218): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::public_word_paired_table_width_and_grid_changes_restore_both_authored_geometries` (L28244): _no doc comment_
- `coverage_real_revision_merge_matrix_tests::public_faithful_paired_table_geometry_has_an_exact_legacy_omission_golden` (L28313): WmlComparer v10.0.0 ReconstructElement (6731-6751) copies the matched ancestor's table properties/grid without an old table-level history. _(from a code comment)_
- `consecutive_block_splice_tests::paragraph` (L28377): _no doc comment_
- `consecutive_block_splice_tests::parse` (L28384): _no doc comment_
- `consecutive_block_splice_tests::canonical` (L28397): Namespace declarations can move when independently cloned revision projections are serialized. _(from a code comment)_
- `consecutive_block_splice_tests::resolved` (L28422): _no doc comment_
- `consecutive_block_splice_tests::adjacent_inserted_prefix_preserves_table_midsplice_and_both_complete_projections` (L28433): _no doc comment_
- `consecutive_block_splice_tests::live_boundary_prevents_an_earlier_insertion_from_claiming_the_splice` (L28471): _no doc comment_
- `demo_title_pilcrow_history_tests::paragraph` (L28495): _no doc comment_
- `demo_title_pilcrow_history_tests::source` (L28501): _no doc comment_
- `demo_title_pilcrow_history_tests::projection` (L28528): _no doc comment_
- `demo_title_pilcrow_history_tests::title_fold_retains_complete_pilcrow_formats_and_configured_history` (L28552): _no doc comment_
- `demo_title_pilcrow_history_tests::effect_projection` (L28627): _no doc comment_
- `demo_title_pilcrow_history_tests::effect_projection::{fn canonical}` (L28628): _no doc comment_
- `demo_title_pilcrow_history_tests::nested_mark_effect_changes_keep_both_source_projections_and_disabled_tracking` (L28667): _no doc comment_
- `demo_title_pilcrow_history_tests::closing_mark_history_preserves_nested_effects_and_ignores_only_existing_normalization_noise` (L28734): _no doc comment_
- `demo_title_pilcrow_history_tests::sharing_a_label_with_nested_effects_obeys_disabled_format_tracking` (L28770): _no doc comment_
- `demo_title_pilcrow_history_tests::title_fold_preserves_existing_mark_history_and_section_break_ownership` (L28787): _no doc comment_
- `demo_title_pilcrow_history_tests::authored_title_snapshot_survives_demo_default_layout_normalization` (L28820): _no doc comment_
- `paired_row_mark_ownership_tests::canonical` (L28869): _no doc comment_
- `paired_row_mark_ownership_tests::project` (L28896): _no doc comment_
- `paired_row_mark_ownership_tests::paired_rows_with_surviving_marks_keep_empty_revised_cells_and_full_original_history` (L28917): _no doc comment_
- `paired_row_mark_ownership_tests::unchanged_empty_pilcrows_keep_row_ownership_without_row_property_history` (L28958): _no doc comment_
- `paired_row_mark_ownership_tests::new_empty_rows_with_owned_pilcrows_still_receive_their_row_lifetime` (L29008): _no doc comment_
- `paired_row_mark_ownership_tests::public_empty_table_comparison_preserves_complete_authored_rows_and_pilcrows` (L29031): _no doc comment_
- `paired_row_mark_ownership_tests::public_empty_table_comparison_preserves_complete_authored_rows_and_pilcrows::{fn source_table}` (L29032): _no doc comment_
- `paired_row_mark_ownership_tests::explicit_row_lifetime_is_not_overridden_by_a_surviving_cell_mark` (L29092): _no doc comment_
- `paired_row_mark_ownership_tests::explicit_row_lifetime_preserves_opposite_content_revisions_and_property_history` (L29105): _no doc comment_
- `closing_authored_section_boundary_tests::parse` (L29152): _no doc comment_
- `closing_authored_section_boundary_tests::section` (L29157): _no doc comment_
- `closing_authored_section_boundary_tests::props` (L29161): _no doc comment_
- `closing_authored_section_boundary_tests::canonical` (L29168): _no doc comment_
- `closing_authored_section_boundary_tests::projection` (L29192): _no doc comment_
- `closing_authored_section_boundary_tests::section_addition_guard_respects_tracking_source_presence_and_exact_live_identity` (L29217): _no doc comment_
- `closing_authored_section_boundary_tests::matching_closing_history_keeps_its_existing_revision_owner_and_id` (L29332): _no doc comment_
- `closing_authored_section_boundary_tests::public_added_and_unchanged_authored_sections_keep_complete_source_bodies` (L29400): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::settings` (L29452): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::package` (L29459): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::semantic` (L29465): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::expected` (L29490): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::run` (L29498): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::para` (L29507): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::marked` (L29510): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::stable` (L29523): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::explicit_default_direction_is_removed_only_from_inserted_layout` (L29534): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::demo_spacing_respects_authored_metrics_history_and_source_defaults` (L29593): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::an_equal_alignment_change_is_attributed_once_and_never_claims_source_history` (L29672): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::the_final_empty_spacing_history_has_one_source_owner` (L29727): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::preexisting_insertions_keep_innermost_provenance_across_real_wrappers` (L29781): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::expected_run_format` (L29864): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::a_pending_insertion_splits_deleted_siblings_without_stealing_their_payloads` (L29873): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::move_simplification_changes_only_lifetime_names_and_range_markers` (L29959): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::equal_shared_labels_cannot_cross_opaque_or_historical_run_ownership` (L30017): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::whitespace_layout_before_a_deleted_block_is_not_empty_content` (L30076): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::list_layout_contamination_does_not_erase_real_list_or_terminal_ownership` (L30155): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::faithful_merged_closing_carrier_records_both_complete_property_sources` (L30208): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::the_short_list_label_uses_only_an_empty_deleted_carrier_with_continuation` (L30291): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::historical_legal_numbering_uses_the_donor_id_and_keeps_authored_old_level` (L30337): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::list_cluster_reorders_only_the_observed_nested_word_family` (L30402): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::a_stranded_mark_can_be_copied_without_deleting_its_layout_shell` (L30511): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::a_terminal_property_history_replaces_the_mark_but_keeps_its_formats` (L30550): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::title_page_empty_relocation_is_scoped_to_an_authored_cover_tail` (L30601): _no doc comment_
- `finalizer_distinct_boundary_contract_tests::a_short_title_mesh_cannot_consume_foreign_revision_provenance_or_unrelated_titles` (L30650): _no doc comment_
- `finalizer_remaining_source_boundary_tests::package` (L30694): _no doc comment_
- `finalizer_remaining_source_boundary_tests::semantic` (L30705): _no doc comment_
- `finalizer_remaining_source_boundary_tests::run` (L30730): _no doc comment_
- `finalizer_remaining_source_boundary_tests::para` (L30739): _no doc comment_
- `finalizer_remaining_source_boundary_tests::marked` (L30745): _no doc comment_
- `finalizer_remaining_source_boundary_tests::assert_body` (L30759): _no doc comment_
- `finalizer_remaining_source_boundary_tests::words` (L30784): _no doc comment_
- `finalizer_remaining_source_boundary_tests::document_scale_folds_preserve_authored_sources_and_observed_word_exceptions` (L30795): The public Word policy distinguishes local edits from wholesale gaps. _(from a code comment)_
- `finalizer_remaining_source_boundary_tests::midstream_deleted_blocks_cross_only_payload_that_disappears_on_rejection` (L31053): _no doc comment_
- `finalizer_remaining_source_boundary_tests::paired_lifetimes_require_complete_matching_comparison_provenance` (L31123): _no doc comment_
- `finalizer_remaining_source_boundary_tests::paired_format_equality_is_complete_and_uses_no_production_arena_nodes` (L31206): _no doc comment_
- `finalizer_remaining_source_boundary_tests::two_authored_layout_blanks_are_restored_only_at_the_shared_table_boundary` (L31283): _no doc comment_
- `finalizer_merge_source_guard_tests::package` (L31439): _no doc comment_
- `finalizer_merge_source_guard_tests::semantic` (L31446): _no doc comment_
- `finalizer_merge_source_guard_tests::run` (L31471): _no doc comment_
- `finalizer_merge_source_guard_tests::para` (L31480): _no doc comment_
- `finalizer_merge_source_guard_tests::marked` (L31486): _no doc comment_
- `finalizer_merge_source_guard_tests::assert_body` (L31500): _no doc comment_
- `finalizer_merge_source_guard_tests::words` (L31525): _no doc comment_
- `finalizer_merge_source_guard_tests::related_replacement_selects_the_documented_structural_carrier_without_losing_run_sources` (L31536): M88 keeps original structural paragraph properties when the new mark is bare. _(from a code comment)_
- `finalizer_merge_source_guard_tests::an_unresolved_field_residue_preserves_m360s_bare_heading_carrier` (L31614): _no doc comment_
- `finalizer_merge_source_guard_tests::legal_and_memo_midstory_block_ownership_is_retained_at_exact_boundaries` (L31653): _no doc comment_
- `finalizer_merge_source_guard_tests::a_carried_original_insertion_keeps_its_author_and_never_becomes_a_real_deletion` (L31727): _no doc comment_
- `finalizer_merge_source_guard_tests::pilcrow_format_equality_includes_nested_fill_payload_but_ignores_source_scratch` (L31786): _no doc comment_
- `finalizer_reached_predicate_boundary_tests::package` (L31871): _no doc comment_
- `finalizer_reached_predicate_boundary_tests::semantic` (L31882): _no doc comment_
- `finalizer_reached_predicate_boundary_tests::run` (L31907): _no doc comment_
- `finalizer_reached_predicate_boundary_tests::para` (L31916): _no doc comment_
- `finalizer_reached_predicate_boundary_tests::marked` (L31922): _no doc comment_
- `finalizer_reached_predicate_boundary_tests::assert_body` (L31936): _no doc comment_
- `finalizer_reached_predicate_boundary_tests::a_later_revised_block_protects_the_long_intro_checklist_boundary` (L31967): Midstory unrelated replacements keep their complete source carriers. _(from a code comment)_
- `finalizer_reached_predicate_boundary_tests::a_short_residual_label_does_not_steal_long_bookmark_or_heading_properties` (L32113): The original bookmark prose deliberately includes the inserted label. _(from a code comment)_
- `public_body_heuristic_story_scope_tests::canonical` (L32233): _no doc comment_
- `public_body_heuristic_story_scope_tests::body_heuristics` (L32263): The public body passes that take only the DOM and a root.
- `public_body_heuristic_story_scope_tests::configured_heuristics` (L32397): The public body passes that also take settings and an id counter.
- `public_body_heuristic_story_scope_tests::public_body_layout_heuristics_preserve_complete_independent_header_and_footer_parts` (L32488): These are valid header/footer part roots accepted by the public DOM helpers, not body-pipeline fixtures. _(from a code comment)_
- `public_body_heuristic_story_scope_tests::outline` (L32572): One element or text node per line, indented by depth, attributes sorted: the form the pass snapshots diff.
- `public_body_heuristic_story_scope_tests::synthetic_body` (L32603): A body with the revision shapes the passes look for: pure inserted and pure deleted titles, a mixed paragraph with a recorded property change, a field, a table and trailing empty paragraphs.
- `public_body_heuristic_story_scope_tests::document_xml` (L32636): The main document part of a fixture package, as XML text.
- `public_body_heuristic_story_scope_tests::pass_report` (L32644): What every public body pass does to `xml`: `unchanged`, or a unified diff of the outline before and after it (and the ids it took).
- `public_body_heuristic_story_scope_tests::finalize_pass_snapshots` (L32702): Snapshot of every public body pass on a small corpus: two of Word's own redlines and a synthetic body.
- `default_source_property_carrier_tests::package` (L32757): _no doc comment_
- `default_source_property_carrier_tests::semantic` (L32768): _no doc comment_
- `default_source_property_carrier_tests::run` (L32793): _no doc comment_
- `default_source_property_carrier_tests::assert_body` (L32802): _no doc comment_
- `default_source_property_carrier_tests::properties` (L32806): _no doc comment_
- `default_source_property_carrier_tests::related_one_to_one_pairs_keep_the_exact_default_or_structural_source_carrier` (L32827): Clean Word paragraphs may omit pPr entirely. _(from a code comment)_
- `default_source_property_carrier_tests::a_sole_original_closes_multi_inserted_default_paragraphs_without_inventing_properties` (L32875): Single-paragraph Word GT closes a wholesale replacement in the last revised paragraph. _(from a code comment)_
- `existing_mark_layout_ownership_tests::package` (L32939): _no doc comment_
- `existing_mark_layout_ownership_tests::semantic` (L32950): _no doc comment_
- `existing_mark_layout_ownership_tests::run` (L32975): _no doc comment_
- `existing_mark_layout_ownership_tests::assert_body` (L32984): _no doc comment_
- `existing_mark_layout_ownership_tests::mark` (L33009): _no doc comment_
- `existing_mark_layout_ownership_tests::history` (L33013): _no doc comment_
- `existing_mark_layout_ownership_tests::fresh_history` (L33018): _no doc comment_
- `existing_mark_layout_ownership_tests::settings` (L33023): _no doc comment_
- `existing_mark_layout_ownership_tests::moving_mixed_spacing_keeps_existing_inserted_and_deleted_mark_owners` (L33035): Word M221 moves a MIX's spacing to the authored following blank. _(from a code comment)_
- `existing_mark_layout_ownership_tests::parking_heading_spacing_keeps_existing_lifetime_and_history_provenance` (L33113): M221's other observed Word shape parks heading spacing on the final deleted residual. _(from a code comment)_
- `existing_mark_layout_ownership_tests::alignment_donor_search_crosses_default_property_paragraphs_without_claiming_their_formats` (L33198): _no doc comment_
- `interleave_terminal_source_ownership_tests::package` (L33268): _no doc comment_
- `interleave_terminal_source_ownership_tests::semantic` (L33277): _no doc comment_
- `interleave_terminal_source_ownership_tests::owned` (L33304): Empty text is an authored blank paragraph with a revision on its pilcrow, rather than an empty text node masquerading as a list item. _(from a code comment)_
- `interleave_terminal_source_ownership_tests::remains_source_owned` (L33328): _no doc comment_
- `interleave_terminal_source_ownership_tests::terminal_revision_runs_and_incomplete_interleave_keep_all_source_positions` (L33369): _no doc comment_
- `interleave_terminal_source_ownership_tests::an_authored_inserted_blank_is_not_a_short_list_label` (L33398): _no doc comment_
- `interleave_terminal_source_ownership_tests::a_deleted_blank_numbered_paragraph_does_not_claim_a_nested_content_cluster` (L33416): _no doc comment_
- `interleave_terminal_source_ownership_tests::split_inserted_title_fragments_do_not_take_another_authors_pending_revision` (L33433): _no doc comment_
- `whitespace_source_mark_carrier_tests::package` (L33486): _no doc comment_
- `whitespace_source_mark_carrier_tests::semantic` (L33495): _no doc comment_
- `whitespace_source_mark_carrier_tests::revision` (L33529): _no doc comment_
- `whitespace_source_mark_carrier_tests::check_complete` (L33535): _no doc comment_
- `whitespace_source_mark_carrier_tests::whitespace_fold_preserves_selected_source_mark_shells_and_every_body_owner` (L33565): Word M86/M88 folds actual inserted spaces into the residual deletion. _(from a code comment)_
- `fragmented_source_label_boundary_tests::package` (L33649): _no doc comment_
- `fragmented_source_label_boundary_tests::semantic` (L33658): _no doc comment_
- `fragmented_source_label_boundary_tests::run` (L33694): _no doc comment_
- `fragmented_source_label_boundary_tests::wrapper` (L33698): _no doc comment_
- `fragmented_source_label_boundary_tests::assert_complete` (L33704): _no doc comment_
- `fragmented_source_label_boundary_tests::fragmented_labels_keep_literal_source_chars_formats_and_revision_history` (L33728): _no doc comment_
- `fragmented_source_label_boundary_tests::label_format_boundaries_and_disabled_history_leave_complete_original_sources_intact` (L33882): _no doc comment_
- `fragmented_source_label_boundary_tests::midstream_heading_search_preserves_controls_live_anchors_and_identical_property_owners` (L33926): _no doc comment_
- `fragmented_source_label_boundary_tests::public_label_zip_preserves_already_adjacent_spacer_and_missing_layout_ownership` (L34014): Word M374 keeps the deleted list owner, donates only missing scalar layout properties, and leaves an already adjacent inserted spacer in source order.
- `fragmented_source_label_boundary_tests::wholesale_glue_only_lcs_preserves_crossed_significant_source_anchors` (L34112): A common significant word may exist on both sides while the longest ordered match contains only glue words.
- `nested_deleted_text_source_ownership_tests::package` (L34154): _no doc comment_
- `nested_deleted_text_source_ownership_tests::revision` (L34168): _no doc comment_
- `nested_deleted_text_source_ownership_tests::field` (L34173): _no doc comment_
- `nested_deleted_text_source_ownership_tests::outer_deletion_preserves_independently_owned_nested_revision_text_and_fields` (L34181): _no doc comment_
- `ladder_suffix_source_boundary_tests::xml` (L34270): _no doc comment_
- `ladder_suffix_source_boundary_tests::package` (L34276): _no doc comment_
- `ladder_suffix_source_boundary_tests::carrier` (L34282): _no doc comment_
- `ladder_suffix_source_boundary_tests::public_ladder_restores_suffix_author_at_unicode_and_run_boundaries` (L34292): _no doc comment_
- `reached_word_document_policy_tests::paragraph` (L34411): _no doc comment_
- `reached_word_document_policy_tests::document_policy_reaches_numbering_relatedness_and_demo_intro_boundaries` (L34432): _no doc comment_
- `mid_list_optional_source_property_tests::package` (L34603): _no doc comment_
- `mid_list_optional_source_property_tests::list_layout_cleanup_handles_absent_optional_properties_without_inventing_owners` (L34616): M376 removes copied list layout from a middle prose deletion.
- `live_opaque_row_source_boundary_tests::unchanged_opaque_cell_owners_prevent_row_lifetime_synthesis_without_text_caches` (L34697): Nontext source owners are real content, even when their cached result is empty. _(from a code comment)_
- `deleted_carrier_echo_source_owner_tests::parse` (L34785): _no doc comment_
- `deleted_carrier_echo_source_owner_tests::paragraph` (L34791): _no doc comment_
- `deleted_carrier_echo_source_owner_tests::canonical` (L34796): _no doc comment_
- `deleted_carrier_echo_source_owner_tests::events` (L34825): _no doc comment_
- `deleted_carrier_echo_source_owner_tests::deleted_original_carrier_keeps_echo_prefix_and_both_complete_source_projections` (L34867): _no doc comment_
