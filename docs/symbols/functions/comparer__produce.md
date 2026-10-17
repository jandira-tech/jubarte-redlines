# Functions and methods in `src/comparer/produce.rs`

31 non-test, 30 test. Part of the [symbol manifest](../README.md).

#### `delete_text_in_opaque`  (L41-50)
`fn delete_text_in_opaque(dom: &mut Dom, node: NodeId, status: CorrelationStatus)`

- **Does:** Deleted opaque subtrees (drawings, text boxes, `mc:AlternateContent`) are cloned verbatim, so their nested text still reads `w:t`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L46; `Dom::set_name` (xmllinq/mod.rs:539) L47
- **Calls (std/external/unresolved):** `.clone`, `Some`, `W::del_text`, `W::t`
- **Macros:** `matches!`
- **Called by:** `coalesce_recurse` (comparer/produce.rs:912); `opaque_text_tests::deleted_opaque_text_becomes_deltext` (comparer/produce.rs:1817); `opaque_text_tests::moved_source_opaque_text_stays_t_like_word` (comparer/produce.rs:1825); `opaque_text_tests::non_deleted_opaque_text_stays_t` (comparer/produce.rs:1837); `opaque_text_tests::instr_text_is_untouched` (comparer/produce.rs:1855)

#### `next_rev_id`  (L52-54)
`fn next_rev_id() -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.fetch_add`, `.to_string`
- **Called by:** `wrap_run` (comparer/produce.rs:167)

#### `produce_document`  (L65-101, pub)
`pub fn produce_document( dom: &mut Dom, tagged: &[TaggedAtom], settings: &WmlComparerSettings, ) -> NodeId`

- **Does:** Build the redline `<w:document>` node from the tagged atom stream.
- **Calls (crate):** `Dom::new_document` (xmllinq/mod.rs:472) L70; `Dom::new_element` (xmllinq/mod.rs:478) L71,74; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L72,73; `XNamespace::xmlns` (xmllinq/mod.rs:122) L72,73; `Dom::name_is` (xmllinq/mod.rs:535) L79; `build_paragraph` (comparer/produce.rs:104) L81,94; `Dom::add` (xmllinq/mod.rs:1058) L82,95,98,99
- **Calls (std/external/unresolved):** `.clear`, `.clone`, `.is_empty`, `.name`, `.push`, `Some`, `Vec::new`, `W::body`, `W::document`, `W::p_pr`
- **Called by:** `compare_bodies` (comparer/mod.rs:79)

#### `build_paragraph`  (L104-152)
`fn build_paragraph( dom: &mut Dom, run_atoms: &[TaggedAtom], ppr_atom: &TaggedAtom, settings: &WmlComparerSettings, ) -> NodeId`

- **Does:** Build one `<w:p>` from its run atoms + the paragraph-mark atom.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L110; `Dom::name_is` (xmllinq/mod.rs:535) L113; `Dom::has_elements` (xmllinq/mod.rs:862) L114; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L116; `Dom::add` (xmllinq/mod.rs:1058) L117,138,142,147; `group_adjacent` (util/group_adjacent.rs:20) L121; `Dom::name` (xmllinq/mod.rs:526) L127; `Dom::value_str` (xmllinq/mod.rs:1179) L130; `wrap_run` (comparer/produce.rs:167) L137,141; `build_text_run` (comparer/produce.rs:155) L146
- **Calls (std/external/unresolved):** `.cloned`, `.collect`, `.filter`, `.into_owned`, `.is_empty`, `.iter`, `.map`, `Some`, `W::del_text`, `W::p_pr`, `W::p`, `W::t`
- **Called by:** `produce_document` (comparer/produce.rs:65)

#### `build_text_run`  (L155-164)
`fn build_text_run(dom: &mut Dom, text: &str, deleted: bool) -> NodeId`

- **Does:** Build `<w:r><w:t>text</w:t></w:r>` (or delText when `deleted`).
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L156,157; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L159; `XNamespace::xml` (xmllinq/mod.rs:128) L159; `Dom::add_text` (xmllinq/mod.rs:1071) L161; `Dom::add` (xmllinq/mod.rs:1058) L162
- **Calls (std/external/unresolved):** `.ends_with`, `.name`, `.starts_with`, `Some`, `W::del_text`, `W::r`, `W::t`
- **Called by:** `build_paragraph` (comparer/produce.rs:104); `wrap_run` (comparer/produce.rs:167)

#### `wrap_run`  (L167-186)
`fn wrap_run( dom: &mut Dom, text: &str, deleted: bool, settings: &WmlComparerSettings, status: CorrelationStatus, ) -> NodeId`

- **Does:** Wrap a run in `<w:ins>`/`<w:del>` with id/author/date.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L179; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L180,181,182; `next_rev_id` (comparer/produce.rs:52) L180; `build_text_run` (comparer/produce.rs:155) L183; `Dom::add` (xmllinq/mod.rs:1058) L184
- **Calls (std/external/unresolved):** `Some`, `W::author`, `W::date`, `W::del`, `W::id`, `W::ins`
- **Macros:** `matches!`
- **Called by:** `build_paragraph` (comparer/produce.rs:104)

#### `flatten_atoms`  (L196-201)
`fn flatten_atoms(units: &[ComparisonUnit]) -> Vec<ComparisonUnitAtom>`

- **Does:** _no doc comment_
- **Calls (crate):** `ComparisonUnit::descendant_atoms` (comparer/atoms.rs:406) (by name) L199
- **Calls (std/external/unresolved):** `.cloned`, `.collect`, `.flat_map`, `.into_iter`, `.iter`
- **Called by:** `flatten_to_comparison_unit_atom_list` (comparer/produce.rs:225)

#### `atom_has_predelete_orig`  (L206-214)
`fn atom_has_predelete_orig(dom: &Dom, atom: &ComparisonUnitAtom) -> bool`

- **Does:** True when a before-side atom carries `pt:PreDelete="orig"` on itself or an ancestor (word-mode flattened A-only pre-existing deletion).
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L208,213
- **Calls (std/external/unresolved):** `.any`, `.iter`, `PT::name`, `Some`
- **Called by:** `flatten_to_comparison_unit_atom_list` (comparer/produce.rs:225)

#### `flatten_to_comparison_unit_atom_list`  (L225-278, pub)
`pub fn flatten_to_comparison_unit_atom_list( dom: &Dom, seqs: &[CorrelatedSequence], ) -> Vec<ComparisonUnitAtom>`

- **Does:** M4.E.1 — `FlattenToComparisonUnitAtomList` (WmlComparer.ts :4141): nested correlated tree → flat status-tagged atom list.
- **Calls (crate):** `flatten_atoms` (comparer/produce.rs:196) L233,234,261,268; `atom_has_predelete_orig` (comparer/produce.rs:206) L239
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.clone`, `.iter`, `.push`, `.unwrap_or`, `.zip`, `Some`, `Vec::new`, `std::sync::Arc::new`
- **Macros:** `panic!`
- **Called by:** `process_footnote_endnote` (comparer/footnotes.rs:1115); `deep_lcs_source_conservation_tests::split_changed_prose_with_deleted_table_keeps_every_space_and_mark_atom` (comparer/lcs.rs:15858); `horizontal_partition_tests::actual_atomized_split_with_deleted_table_conserves_source_through_production` (comparer/lcs_table.rs:1187); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `same_authored_control`  (L284-302)
`fn same_authored_control(dom: &Dom, a: NodeId, b: NodeId) -> bool`

- **Does:** Authored control identifiers distinguish an existing inner control from a newly introduced outer wrapper.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L286,291; `Dom::attribute` (xmllinq/mod.rs:874) L292
- **Calls (std/external/unresolved):** `.and_then`, `.is_empty`, `W::id`, `W::name`, `W::val`, `value`
- **Called by:** `authored_control_match_strength` (comparer/produce.rs:307); `authored_control_identity_regressions::nonempty_tags_override_regenerated_ids_and_empty_tags_allow_fallbacks` (comparer/produce.rs:2279)

#### `authored_control_match_strength`  (L307-324)
`fn authored_control_match_strength(dom: &Dom, a: NodeId, b: NodeId) -> u8`

- **Does:** Resolve repeated authored tags by preferring the retained control's exact id within the same transparent ancestry segment.
- **Calls (crate):** `same_authored_control` (comparer/produce.rs:284) L308; `Dom::element` (xmllinq/mod.rs:655) L312,313; `Dom::attribute` (xmllinq/mod.rs:874) L314
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.is_empty`, `W::id`, `W::name`, `W::val`, `control_id`
- **Called by:** `align_content_control_ancestry` (comparer/produce.rs:331)

#### `align_content_control_ancestry`  (L331-449)
`fn align_content_control_ancestry(dom: &Dom, atoms: &mut [ComparisonUnitAtom])`

- **Does:** Flatten only unmatched content-control ancestry in correlated paragraphs.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L335,350,359,367,383,398...; `authored_control_match_strength` (comparer/produce.rs:307) L352,360,368; `Dom::name` (xmllinq/mod.rs:526) L406; `strip` (edit/whole.rs:89) (by name) L444,446
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.contains`, `.copied`, `.extend`, `.filter`, `.get`, `.into`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.map`, `.max`, `.push`, `.take_while`, `.unwrap_or`, `Vec::new`, `W::p`, `W::sdt_content`, `W::sdt`, `is_control`, `std::collections::HashSet::new`, `std::sync::Arc::make_mut`
- **Called by:** `assemble_ancestor_unids` (comparer/produce.rs:470); `reordered_retained_control_identity_regression::both_retained_controls_cannot_be_stripped_when_their_nesting_order_changes` (comparer/produce.rs:2448); `reordered_retained_control_identity_regression::stronger_unilateral_id_match_beats_opposite_regenerated_id_tag_match` (comparer/produce.rs:2468)

#### `is_ppr_atom`  (L451-453)
`fn is_ppr_atom(dom: &Dom, atom: &ComparisonUnitAtom) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L452
- **Calls (std/external/unresolved):** `W::p_pr`
- **Called by:** `assemble_ancestor_unids` (comparer/produce.rs:470)

#### `atom_in_textbox`  (L454-459)
`fn atom_in_textbox(dom: &Dom, atom: &ComparisonUnitAtom) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L458
- **Calls (std/external/unresolved):** `.any`, `.as_ref`, `.iter`, `Some`, `W::txbx_content`
- **Called by:** `assemble_ancestor_unids` (comparer/produce.rs:470)

#### `assemble_ancestor_unids`  (L470-666, pub)
`pub fn assemble_ancestor_unids(dom: &mut Dom, atoms: &mut [ComparisonUnitAtom])`

- **Does:** M4.E.2 — `AssembleAncestorUnidsInOrderToRebuildXmlTreeProperly` (WmlComparer.ts :3974).
- **Calls (crate):** `align_content_control_ancestry` (comparer/produce.rs:331) L471; `is_ppr_atom` (comparer/produce.rs:451) L479,540,614; `atom_in_textbox` (comparer/produce.rs:454) L480,540,615; `Dom::attribute` (xmllinq/mod.rs:874) L495,513,522,626; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L502,526; `Dom::name` (xmllinq/mod.rs:526) L511,579; `generate_unid` (unid.rs:47) L525
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.as_ref`, `.clone`, `.collect`, `.copied`, `.expect`, `.filter_map`, `.first_mut`, `.first`, `.into`, `.iter_mut`, `.iter`, `.last`, `.len`, `.map`, `.push`, `.rev`, `.skip`, `.to_string`, `.to_vec`, `.unwrap_or_default`, `.zip`, `PT::unid`, `Some`, `W::endnote`, `W::footnote`, `std::sync::Arc::clone`, `std::sync::Arc::ptr_eq`, `unid_or_mint`
- **Called by:** `produce_note_redline` (comparer/footnotes.rs:1015); `horizontal_partition_tests::actual_atomized_split_with_deleted_table_conserves_source_through_production` (comparer/lcs_table.rs:1187); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209); `coverage_ruby_math_ownership::ruby_reassembly_retains_reading_base_and_property_owners` (comparer/produce.rs:2538)

#### `xml_space_attr`  (L671-677)
`fn xml_space_attr(text: &str) -> Option<&'static str>`

- **Does:** `GetXmlSpaceAttribute` — `Some("preserve")` when leading/trailing whitespace.
- **Calls (std/external/unresolved):** `.chars`, `.is_whitespace`, `.last`, `.next`, `Some`
- **Called by:** `coalesce_recurse` (comparer/produce.rs:912)

#### `status_str`  (L679-689)
`fn status_str(s: CorrelationStatus) -> &'static str`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `tag_status` (comparer/produce.rs:724); `coalesce_recurse` (comparer/produce.rs:912)

#### `group_by_key_stable`  (L692-721)
`fn group_by_key_stable<'a, K: Eq + std::hash::Hash + Clone>( items: &[&'a ComparisonUnitAtom], key: impl Fn(&ComparisonUnitAtom) -> K, ) -> Vec<(K, Vec<&'a ComparisonUnitAtom>)>`

- **Does:** Stable first-key-seen bucket grouping (port of `groupByKey`).
- **Calls (crate):** `key` (ambiguous: `same_path::{fn key}` (bin/jubarte.rs:159); `key` (markdown/patch.rs:310)) L706; `.key` (ambiguous: `Sha1Keyed::key` (comparer/atoms.rs:257); `FaceId::key` (convert/font.rs:564)) L708
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.entry`, `.get_mut`, `.insert`, `.into_iter`, `.map`, `.push`, `.remove`, `.unwrap`, `Vec::new`, `std::collections::HashMap::new`
- **Macros:** `vec!`
- **Called by:** `coalesce_recurse` (comparer/produce.rs:912)

#### `tag_status`  (L724-759)
`fn tag_status(dom: &mut Dom, node: NodeId, status: CorrelationStatus, atom: &ComparisonUnitAtom)`

- **Does:** Add the `pt:Status` (+ move/format) attributes to a constructed node.
- **Calls (crate):** `Dom::set_attribute_value` (xmllinq/mod.rs:961) L727,730,733,735,736,744...; `status_str` (comparer/produce.rs:679) L733; `Dom::serialize_element` (xmllinq/mod.rs:1299) L747,752
- **Calls (std/external/unresolved):** `.as_deref`, `.to_string`, `.unwrap_or`, `PT::name`, `PT::status`, `Some`
- **Called by:** `coalesce_recurse` (comparer/produce.rs:912)

#### `is_txbx_from_level`  (L761-767)
`fn is_txbx_from_level(dom: &Dom, atom: &ComparisonUnitAtom, level: usize) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L766
- **Calls (std/external/unresolved):** `.any`, `.as_ref`, `.iter`, `.skip`, `Some`, `W::txbx_content`
- **Called by:** `coalesce_recurse` (comparer/produce.rs:912)

#### `convert_outer_math_wraps_to_internal`  (L778-821, pub)
`pub fn convert_outer_math_wraps_to_internal( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, )`

- **Does:** M463 — final-serialization pass: every `w:ins` / `w:del` whose element content is exactly OMML math (`m:oMath` / `m:oMathPara`) is unwrapped, and the revision state is rewritten INSIDE the math the way Word Compare writes it.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L789,798; `Dom::attribute` (xmllinq/mod.rs:874) L790; `Dom::elements` (xmllinq/mod.rs:640) L802,810; `Dom::name` (xmllinq/mod.rs:526) L806; `mark_math_revisions_internally` (comparer/produce.rs:832) L812; `Dom::remove` (xmllinq/mod.rs:1102) L815,818; `Dom::add_before_self` (xmllinq/mod.rs:1114) L816
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.contains`, `.filter`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.max`, `.parse`, `Some`, `W::del`, `W::id`, `W::ins`, `crate::namespaces::M::name`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501)

#### `mark_math_revisions_internally`  (L832-906)
`fn mark_math_revisions_internally( dom: &mut Dom, math_root: NodeId, rev_name: &crate::xmllinq::XName, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** Rewrite a `m:oMath(Para)` subtree so its revision state lives INSIDE the math, the way Word Compare writes it: - every `m:r` moves its children (m:rPr?, w:rPr?, m:t…) into a `w:ins`/`w:del` child; a `w:rPr` with Cambria Math rFonts is materialized when the run stores none (Word always writes it); - every `m:ctrlPr` mo...
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L843,844,851; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L845,846,852,854,855; `Dom::add` (xmllinq/mod.rs:1058) L847,884,899,904; `Dom::descendants` (xmllinq/mod.rs:668) L858,863; `Dom::elements` (xmllinq/mod.rs:640) L870,877,891,897; `Dom::name` (xmllinq/mod.rs:526) L873,881,882,894; `Dom::remove` (xmllinq/mod.rs:1102) L883; `Dom::add_after_self` (xmllinq/mod.rs:1131) L896; `Dom::add_before_self` (xmllinq/mod.rs:1114) L898
- **Calls (std/external/unresolved):** `.any`, `.as_ref`, `.chain`, `.clone`, `.collect`, `.copied`, `.find`, `.first`, `.into_iter`, `.is_some_and`, `.iter`, `.map`, `.to_string`, `Some`, `W::author`, `W::date`, `W::del`, `W::id`, `W::ins`, `W::name`, `W::r_pr`, `cambria`, `crate::namespaces::M::name`, `new_mark`
- **Called by:** `convert_outer_math_wraps_to_internal` (comparer/produce.rs:778); `coverage_ruby_math_ownership::existing_math_revisions_keep_attribution_and_new_siblings_are_marked_once` (comparer/produce.rs:2587)

#### `coalesce_recurse`  (L912-1247, pub)
`pub fn coalesce_recurse( dom: &mut Dom, atoms: &[&ComparisonUnitAtom], level: usize, settings: &WmlComparerSettings, id_gen: &mut u32, ) -> Vec<NodeId>`

- **Does:** M4.E.3–E.7 — `CoalesceRecurse` (WmlComparer.ts :6024).
- **Calls (crate):** `group_by_key_stable` (comparer/produce.rs:692) L928; `Dom::name` (xmllinq/mod.rs:526) L940,957; `XName::local_name` (xmllinq/mod.rs:208) (by name) L942,1034; `group_adjacent` (util/group_adjacent.rs:20) L960; `is_txbx_from_level` (comparer/produce.rs:761) L969; `status_str` (comparer/produce.rs:679) L972; `Dom::name_is` (xmllinq/mod.rs:535) L982; `Dom::new_element` (xmllinq/mod.rs:478) L998,1025,1080,1168,1202; `Dom::attributes` (xmllinq/mod.rs:885) L999,1026,1203; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1000,1032,1204; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1001,1005,1043,1083,1169,1174...; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L1009,1047,1053,1095,1110,1126...; `tag_status` (comparer/produce.rs:724) L1010,1054,1081,1096,1111,1127...; `Dom::add` (xmllinq/mod.rs:1058) L1011,1015,1048,1055,1059,1181...; `coalesce_recurse` (comparer/produce.rs:912) L1014,1058; `Dom::element` (xmllinq/mod.rs:655) L1046; `Dom::value_str` (xmllinq/mod.rs:1179) L1073; `xml_space_attr` (comparer/produce.rs:671) L1082; `XNamespace::xml` (xmllinq/mod.rs:128) L1083; `Dom::add_text` (xmllinq/mod.rs:1071) L1085; `delete_text_in_opaque` (comparer/produce.rs:41) L1097,1128,1141; `Dom::elements` (xmllinq/mod.rs:640) L1107; `Dom::nodes` (xmllinq/mod.rs:604) L1211; `reconstruct_element` (comparer/produce.rs:1251) L1243
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.clone`, `.cloned`, `.collect`, `.contains`, `.filter`, `.get`, `.into_iter`, `.into_owned`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.map`, `.name`, `.push`, `.to_string`, `.unwrap_or_default`, `.unwrap`, `PT::unid`, `Some`, `String::new`, `Vec::new`, `W::author`, `W::date`, `W::del_text`, `W::del`, `W::drawing`, `W::id`, `W::ins`, `W::move_from`, `W::move_to`, `W::name`, `W::object`, `W::p`, `W::pict`, `W::r_pr`, `W::r`, `W::sdt`, `W::t`, `W::tbl`, `W::tc`, `W::tr`, `crate::namespaces::M::name`, `crate::namespaces::MC::name`
- **Macros:** `matches!`
- **Called by:** `coalesce_recurse` (comparer/produce.rs:912); `reconstruct_element` (comparer/produce.rs:1251); `produce_new_wml_markup_from_correlated_sequence` (comparer/produce.rs:1769); `run_format_history_regressions::one_new_run_preserves_each_original_format_boundary_when_rejected` (comparer/produce.rs:2064); `run_format_history_regressions::inline_containers_preserve_each_original_format_boundary_when_rejected` (comparer/produce.rs:2145); `nontext_run_format_history_tests::changed_nontext_run_payload_keeps_complete_original_and_revised_properties` (comparer/produce.rs:2684)

#### `reconstruct_element`  (L1251-1460)
`fn reconstruct_element( dom: &mut Dom, g: &[&ComparisonUnitAtom], ancestor: NodeId, props: &[&str], level: usize, settings: &WmlComparerSettings, id_gen: &mut u32, ) -> NodeId`

- **Does:** M4.E.6 — `ReconstructElement` (:6984): rebuild a container element, hoisting the named property children first, then the recursively-coalesced children.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L1260,1324; `Dom::name_is` (xmllinq/mod.rs:535) L1271; `group_adjacent` (util/group_adjacent.rs:20) L1274; `coalesce_recurse` (comparer/produce.rs:912) L1291,1295; `Dom::new_element` (xmllinq/mod.rs:478) L1297,1399,1427; `Dom::attributes` (xmllinq/mod.rs:885) L1298; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1299,1400,1402,1407,1434; `Dom::elements` (xmllinq/mod.rs:640) L1303,1309,1361,1364,1375; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L1304,1310,1365,1394,1423; `Dom::add` (xmllinq/mod.rs:1058) L1305,1311,1366,1412,1413,1436...; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1324; `Dom::remove` (xmllinq/mod.rs:1102) L1362,1376,1415,1439; `children_signature` (comparer/produce.rs:1608) L1383; `Dom::element` (xmllinq/mod.rs:655) L1391,1392,1420,1421; `record_revised_row_cell_props` (comparer/produce.rs:1468) L1448; `rebuild_degenerate_grid` (comparer/produce.rs:1669) L1457
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.as_ref`, `.clone`, `.cloned`, `.copied`, `.extend`, `.filter`, `.find_map`, `.get`, `.is_some_and`, `.iter`, `.or_else`, `.to_string`, `.unwrap`, `Some`, `Vec::new`, `W::author`, `W::date`, `W::id`, `W::name`, `W::pict`, `W::r`, `W::tbl_pr`, `W::tbl`, `W::tc`, `W::tr`, `changed`, `crate::namespaces::VML::name`, `is_tbl`, `strip_change`
- **Called by:** `coalesce_recurse` (comparer/produce.rs:912)

#### `record_revised_row_cell_props`  (L1468-1570)
`fn record_revised_row_cell_props( dom: &mut Dom, ne: NodeId, g: &[&ComparisonUnitAtom], ancestor: NodeId, level: usize, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** A merged row or cell shows the revision's `trPr` / `tcPr` and records the original's in `trPrChange` / `tcPrChange` (ff42b4a7a3's rewritten header row; 117 of the 300 table-bearing Word redlines of 500_extra carry a `tcPrChange`).
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L1477,1485,1527,1546; `Dom::elements` (xmllinq/mod.rs:640) L1512,1526,1542,1552,1558; `Dom::remove` (xmllinq/mod.rs:1102) L1513,1531,1559; `Dom::element` (xmllinq/mod.rs:655) L1515,1522,1534; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L1516,1523; `place_row_cell_props` (comparer/produce.rs:1616) L1517,1538; `Dom::new_element` (xmllinq/mod.rs:478) L1524,1537,1563; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1529,1547; `props_signature` (comparer/produce.rs:1577) L1550,1555; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1564,1566,1567; `Dom::add` (xmllinq/mod.rs:1058) L1568,1569
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.collect`, `.copied`, `.filter`, `.find_map`, `.get`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.map`, `.to_string`, `Some`, `W::author`, `W::date`, `W::id`, `W::name`, `W::tr`
- **Macros:** `matches!`
- **Called by:** `reconstruct_element` (comparer/produce.rs:1251)

#### `props_signature`  (L1577-1605)
`fn props_signature(dom: &Dom, n: NodeId) -> String`

- **Does:** A property element's name, sorted attributes and children, recursively: the comparer's scratch `pt14:*` ids and rsids are no properties, so two clones of the same `w:tcW` compare equal whatever Unid each carries.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L1578; `Dom::attributes` (xmllinq/mod.rs:885) L1581; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1584,1588,1601; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1584,1589,1602; `Dom::elements` (xmllinq/mod.rs:640) L1594; `props_signature` (comparer/produce.rs:1577) L1597
- **Calls (std/external/unresolved):** `.collect`, `.concat`, `.filter`, `.into_iter`, `.map`, `.sort`, `.starts_with`, `String::new`
- **Macros:** `format!`
- **Called by:** `record_revised_row_cell_props` (comparer/produce.rs:1468); `props_signature` (comparer/produce.rs:1577); `children_signature` (comparer/produce.rs:1608)

#### `children_signature`  (L1608-1614)
`fn children_signature(dom: &Dom, n: NodeId, change: &str) -> String`

- **Does:** The signatures of `n`'s children but its own `w:{change}` record.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1609; `Dom::name` (xmllinq/mod.rs:526) L1611; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1611; `props_signature` (comparer/produce.rs:1577) L1612
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_none_or`, `.map`
- **Called by:** `reconstruct_element` (comparer/produce.rs:1251)

#### `place_row_cell_props`  (L1616-1621)
`fn place_row_cell_props(dom: &mut Dom, container: NodeId, pr: NodeId)`

- **Does:** Put a row's `trPr` after its `tblPrEx`, a cell's `tcPr` first.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1617; `Dom::add_after_self` (xmllinq/mod.rs:1131) L1618; `Dom::add_first` (xmllinq/mod.rs:1085) L1619
- **Calls (std/external/unresolved):** `W::name`
- **Called by:** `record_revised_row_cell_props` (comparer/produce.rs:1468)

#### `add_tblpr_child_in_order`  (L1648-1665)
`fn add_tblpr_child_in_order(dom: &mut Dom, tbl_pr: NodeId, child: NodeId, local: &str)`

- **Does:** Insert `child` (a new tblPr child named `local`) under `tbl_pr` in CT_TblPrBase order: after the last present predecessor, else first.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1653; `Dom::name` (xmllinq/mod.rs:526) L1654; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1657; `Dom::add_after_self` (xmllinq/mod.rs:1131) L1662; `Dom::add_first` (xmllinq/mod.rs:1085) L1663
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.is_some_and`, `.iter`, `.position`, `.rev`, `.unwrap_or`
- **Called by:** `rebuild_degenerate_grid` (comparer/produce.rs:1669); `tblpr_order_tests::tblw_inserted_after_tblppr_and_bidivisual` (comparer/produce.rs:1887)

#### `rebuild_degenerate_grid`  (L1669-1763)
`fn rebuild_degenerate_grid(dom: &mut Dom, tbl: NodeId, src_tbl: NodeId)`

- **Does:** M-TBL rule 4 — see call site.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1670,1682,1683,1703,1707,1751...; `Dom::elements` (xmllinq/mod.rs:640) L1673,1675,1679; `Dom::attribute` (xmllinq/mod.rs:874) L1684,1705,1708,1711,1720; `Dom::ancestors` (xmllinq/mod.rs:770) L1697; `Dom::descendants` (xmllinq/mod.rs:668) L1700; `Dom::remove` (xmllinq/mod.rs:1102) L1728; `Dom::new_element` (xmllinq/mod.rs:478) L1738,1755; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1739,1760,1761; `Dom::add_first` (xmllinq/mod.rs:1085) L1744; `add_tblpr_child_in_order` (comparer/produce.rs:1648) L1756
- **Calls (std/external/unresolved):** `.and_then`, `.copied`, `.filter_map`, `.filter`, `.first`, `.into_iter`, `.iter`, `.last`, `.len`, `.map`, `.max`, `.ok`, `.parse`, `.push`, `.rev`, `.sum`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `Some`, `Vec::with_capacity`, `W::grid_span`, `W::name`, `W::sect_pr`, `W::tbl_pr`, `W::tc_pr`, `W::tc`, `W::tr`, `W::val`
- **Called by:** `reconstruct_element` (comparer/produce.rs:1251)

#### `produce_new_wml_markup_from_correlated_sequence`  (L1769-1778, pub)
`pub fn produce_new_wml_markup_from_correlated_sequence( dom: &mut Dom, atoms: &[ComparisonUnitAtom], settings: &WmlComparerSettings, id_gen: &mut u32, ) -> Vec<NodeId>`

- **Does:** M4.E.8 — `ProduceNewWmlMarkupFromCorrelatedSequence` (WmlComparer.ts :5926): coalesce at level 0 and return the top-level children of the rebuilt `<w:body>`.
- **Calls (crate):** `coalesce_recurse` (comparer/produce.rs:912) L1777
- **Calls (std/external/unresolved):** `.collect`, `.iter`
- **Called by:** `produce_note_redline` (comparer/footnotes.rs:1015); `horizontal_partition_tests::actual_atomized_split_with_deleted_table_conserves_source_through_production` (comparer/lcs_table.rs:1187); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209); `coverage_ruby_math_ownership::ruby_reassembly_retains_reading_base_and_property_owners` (comparer/produce.rs:2538)

## Test functions (30)

- `opaque_text_tests::opaque_with_text` (L1791): `<w:drawing><w:r><w:t>txt</w:t></w:r></w:drawing>` — an opaque subtree.
- `opaque_text_tests::text_kind` (L1802): Local name of the first `w:t`/`w:delText` leaf under `node`.
- `opaque_text_tests::deleted_opaque_text_becomes_deltext` (L1817): _no doc comment_
- `opaque_text_tests::moved_source_opaque_text_stays_t_like_word` (L1825): _no doc comment_
- `opaque_text_tests::non_deleted_opaque_text_stays_t` (L1837): _no doc comment_
- `opaque_text_tests::instr_text_is_untouched` (L1855): _no doc comment_
- `tblpr_order_tests::tblw_inserted_after_tblppr_and_bidivisual` (L1887): _no doc comment_
- `content_control_source_order_regressions::document` (L1921): _no doc comment_
- `content_control_source_order_regressions::paragraph` (L1930): _no doc comment_
- `content_control_source_order_regressions::control` (L1934): _no doc comment_
- `content_control_source_order_regressions::text` (L1950): _no doc comment_
- `content_control_source_order_regressions::introducing_or_removing_a_control_keeps_replacement_words_in_both_source_positions` (L1958): _no doc comment_
- `content_control_source_order_regressions::equal_control_ancestry_keeps_its_wrapper_and_authored_properties` (L1997): _no doc comment_
- `run_format_history_regressions::run_snapshot` (L2043): _no doc comment_
- `run_format_history_regressions::one_new_run_preserves_each_original_format_boundary_when_rejected` (L2064): _no doc comment_
- `run_format_history_regressions::inline_containers_preserve_each_original_format_boundary_when_rejected` (L2145): _no doc comment_
- `authored_control_identity_regressions::nonempty_tags_override_regenerated_ids_and_empty_tags_allow_fallbacks` (L2279): _no doc comment_
- `repeated_control_tag_regressions::document` (L2325): _no doc comment_
- `repeated_control_tag_regressions::controlled` (L2334): _no doc comment_
- `repeated_control_tag_regressions::text` (L2340): _no doc comment_
- `repeated_control_tag_regressions::repeated_tag_outer_wrapper_never_replaces_retained_inner_control_metadata` (L2348): _no doc comment_
- `reordered_retained_control_identity_regression::source` (L2420): _no doc comment_
- `reordered_retained_control_identity_regression::ids` (L2434): _no doc comment_
- `reordered_retained_control_identity_regression::both_retained_controls_cannot_be_stripped_when_their_nesting_order_changes` (L2448): _no doc comment_
- `reordered_retained_control_identity_regression::stronger_unilateral_id_match_beats_opposite_regenerated_id_tag_match` (L2468): _no doc comment_
- `coverage_ruby_math_ownership::semantic` (L2505): _no doc comment_
- `coverage_ruby_math_ownership::ruby_reassembly_retains_reading_base_and_property_owners` (L2538): _no doc comment_
- `coverage_ruby_math_ownership::existing_math_revisions_keep_attribution_and_new_siblings_are_marked_once` (L2587): _no doc comment_
- `nontext_run_format_history_tests::signature` (L2660): _no doc comment_
- `nontext_run_format_history_tests::changed_nontext_run_payload_keeps_complete_original_and_revised_properties` (L2684): _no doc comment_
