# Functions and methods in `src/document_comparer.rs`

106 non-test, 127 test. Part of the [symbol manifest](../README.md).

#### `carry_revised_part_relationships`  (L25-64)
`fn carry_revised_part_relationships( out: &mut PartFs, part_a: &str, (pkg2, part_b): (&PartFs, &str), xml: &str, ) -> Option<String>`

- **Does:** `xml` (part `part_b` of `pkg2`) with each relationship it references carried onto `part_a` of `out` under a fresh id (an image with A's bytes reuses A's relationship), so B's content means what it meant in B once it is diffed into A's part.
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L31; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L32; `Dom::root` (xmllinq/mod.rs:822) L33; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L36; `Dom::attributes` (xmllinq/mod.rs:885) L37; `Dom::name` (xmllinq/mod.rs:526) L41; `required_rel_type_suffix` (comparer/parts.rs:127) L43; `XName::local_name` (xmllinq/mod.rs:208) (by name) L43; `carry_relationship` (comparer/parts.rs:517) L48; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L60; `Dom::serialize_document` (xmllinq/mod.rs:1322) L63
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.contains`, `.ends_with`, `.get`, `.insert`, `.is_none_or`, `Some`, `std::collections::HashMap::new`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::carry_relationships_reuses_minted_id_and_refuses_unresolved_references` (document_comparer.rs:9745); `comparer_relationship_contract_tests::repeated_hyperlinks_share_one_carried_relationship_and_keep_independent_source_text_and_format` (document_comparer.rs:11957)

#### `part_rels_agree`  (L70-110)
`fn part_rels_agree( (pkg1, part_a): (&PartFs, &str), (pkg2, part_b): (&PartFs, &str), xml: &str, ) -> bool`

- **Does:** Every relationship `xml` (part `part_b` of `pkg2`) references resolves in `part_a`'s rels to the same type, mode and target, and an internal target to the same bytes.
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L75; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L76; `Dom::root` (xmllinq/mod.rs:822) L77; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L80; `Dom::attributes` (xmllinq/mod.rs:885) L83; `PartFs::read_rels_for` (opc/mod.rs:756) (by name) L90; `find` (ambiguous: `find` (markdown/pages.rs:451); `find` (markdown/unified.rs:464)) L97; `PartFs::part_bytes` (opc/mod.rs:493) (by name) L106,107; `PartFs::resolve_rel_target` (opc/mod.rs:600) (by name) L106,107
- **Calls (std/external/unresolved):** `.all`, `.as_deref`, `.cloned`, `.collect`, `.contains`, `.filter`, `.find`, `.flat_map`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.map`, `Some`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::relationship_agreement_refuses_missing_mismatched_and_dangling_targets` (document_comparer.rs:9701); `comparer_relationship_contract_tests::referenced_relationship_agreement_requires_matching_type_mode_and_payload_or_external_target` (document_comparer.rs:11866)

#### `header_footer_refs`  (L116-122)
`fn header_footer_refs(pkg: &PartFs) -> Vec<(String, String, String)>`

- **Does:** The header/footer parts a document references, as (kind, type, part-name): kind ∈ {"header","footer"}, type ∈ {"default","even","first"}.
- **Calls (crate):** `header_footer_refs_by_section` (document_comparer.rs:129) L117
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`
- **Called by:** `mark_a_only_hf_content_as_deleted` (document_comparer.rs:5563); `coverage_boundary_tests::section_story_references_resolve_actual_opc_targets_before_legacy_word_paths` (document_comparer.rs:11293)

#### `header_footer_refs_by_section`  (L129-183)
`fn header_footer_refs_by_section(pkg: &PartFs) -> (usize, Vec<SectionRef>)`

- **Does:** A document's section count and [`header_footer_refs`], each tagged with the index of the section that references it.
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:829) L130; `PartFs::part_string` (opc/mod.rs:498) L133; `PartFs::read_rels_for` (opc/mod.rs:756) L136; `Dom::new` (xmllinq/mod.rs:387) L144; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L145; `Dom::root` (xmllinq/mod.rs:822) L146; `section_props` (document_comparer.rs:187) L149; `Dom::descendants` (xmllinq/mod.rs:668) L155; `Dom::attribute` (xmllinq/mod.rs:874) L156,164; `Dom::parent` (xmllinq/mod.rs:597) L160; `PartFs::resolve_rel_target` (opc/mod.rs:600) L167; `PartFs::part_bytes` (opc/mod.rs:493) L171,173
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.collect`, `.get`, `.is_some`, `.iter`, `.len`, `.map`, `.position`, `.push`, `.starts_with`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `R::name`, `Some`, `Vec::new`, `W::name`
- **Called by:** `header_footer_refs` (document_comparer.rs:116); `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::empty_missing_and_non_xml_package_parts_are_safe_noops` (document_comparer.rs:9962); `coverage_boundary_tests::section_story_references_resolve_actual_opc_targets_before_legacy_word_paths` (document_comparer.rs:11293)

#### `section_props`  (L187-195)
`fn section_props(dom: &Dom, root: NodeId) -> Vec<NodeId>`

- **Does:** A document's sections in order: each paragraph mark's `w:sectPr`, then the body's own.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L188; `Dom::parent` (xmllinq/mod.rs:597) L191; `Dom::name_is` (xmllinq/mod.rs:535) L192
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_some_and`, `Some`, `W::body`, `W::p_pr`, `W::sect_pr`
- **Called by:** `header_footer_refs_by_section` (document_comparer.rs:129); `section_pairing` (document_comparer.rs:202)

#### `section_pairing`  (L202-228)
`fn section_pairing( redline_xml: &str, sections_a: usize, sections_b: usize, ) -> Option<Vec<Option<usize>>>`

- **Does:** The revised section each original section became, read off the redline's section breaks: the section of a deleted break has none (Word leaves its headers as they were, probes hs1/hs3), an inserted break's is the revision's alone.
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L207; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L208; `Dom::root` (xmllinq/mod.rs:822) L209; `Dom::parent` (xmllinq/mod.rs:597) L211; `Dom::name_is` (xmllinq/mod.rs:535) L212; `Dom::element` (xmllinq/mod.rs:655) L213,214; `section_props` (document_comparer.rs:187) L218; `marked` (ambiguous, 5 candidates) L219,220
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.is_some_and`, `.is_some`, `.len`, `.push`, `.then_some`, `Vec::new`, `W::del`, `W::ins`, `W::p_pr`, `W::r_pr`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::section_pairing_accounts_for_inserted_deleted_and_recorded_breaks` (document_comparer.rs:8873)

#### `pair_header_footer`  (L238-264)
`fn pair_header_footer( pkg1: &PartFs, pkg2: &PartFs, refs_b: &[SectionRef], (kind, ty, part_a): (&str, &str, &str), at: usize, section: Option<Option<usize>>, ) -> Option<String>`

- **Does:** The revised part a header/footer part is diffed against, or none when A's part is unchanged in B.
- **Calls (crate):** `PartFs::part_bytes` (opc/mod.rs:493) L251,254
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.collect`, `.filter`, `.find`, `.get`, `.iter`, `.last`, `.map`, `.or`, `.rev`, `.to_string`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::header_pairing_uses_surviving_section_inheritance_and_content_identity` (document_comparer.rs:9599)

#### `find_normal_style`  (L271-296)
`fn find_normal_style(dom: &Dom, styles_root: NodeId) -> Option<NodeId>`

- **Does:** The `w:style[@w:type='paragraph' @w:styleId='Normal']` element under a styles root, falling back to the `w:default="1"` paragraph style.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L272; `Dom::attribute` (xmllinq/mod.rs:874) L275,280,285,292; `Dom::element` (xmllinq/mod.rs:655) L291
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.copied`, `.eq_ignore_ascii_case`, `.filter`, `.find`, `.into_iter`, `.is_some_and`, `.iter`, `.or_else`, `Some`, `W::name`, `W::val`
- **Called by:** `merge_normal_style_spacing` (document_comparer.rs:777); `cascade_normal_change_to_based_styles` (document_comparer.rs:1124); `inherited_only_style_changes` (document_comparer.rs:1601); `merge_revised_style_definitions` (document_comparer.rs:2160); `normalize_word_paragraph_style_line` (document_comparer.rs:2948); `align_paragraph_style_fonts_with_normal` (document_comparer.rs:3053); `prune_normal_rpr_context_equal_attrs` (document_comparer.rs:3822); `bake_bothsides_dd_disabling_neutralizers` (document_comparer.rs:3909); `merge_normal_style_rpr` (document_comparer.rs:4343); `tests::normal_takes_b_theme_fonts_from_its_doc_defaults` (document_comparer.rs:8330); `tests::normal_is_found_by_name_when_no_style_is_the_default` (document_comparer.rs:8372); `tests::live_normal_attrs` (document_comparer.rs:8432); +5 more

#### `normal_spacing`  (L299-304)
`fn normal_spacing(dom: &Dom, style: NodeId) -> Option<(String, String, String)>`

- **Does:** A style's `pPr/spacing` as (after, line, lineRule), each None when absent.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L300,301; `Dom::attribute` (xmllinq/mod.rs:874) L302
- **Calls (std/external/unresolved):** `.to_string`, `.unwrap_or`, `Some`, `W::name`, `get`
- **Called by:** `merge_normal_style_spacing` (document_comparer.rs:777)

#### `spacing_before_attr`  (L311-314)
`fn spacing_before_attr(dom: &Dom, sp_holder: Option<NodeId>) -> Option<String>`

- **Does:** `before` companion to [`normal_spacing`]/[`docdefaults_ppr_spacing`] — M479: the Normal-merge truth table was derived over {after, line} only; an A-docDefaults `before` (paragraph_spacing_missing: before=240) leaked through unneutralized, adding 12pt above every Normal paragraph.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L313
- **Calls (std/external/unresolved):** `.map`, `W::name`
- **Called by:** `merge_normal_style_spacing` (document_comparer.rs:777)

#### `b_docdefaults_ppr_spacing`  (L324-330)
`fn b_docdefaults_ppr_spacing(dom: &Dom, b_root: NodeId) -> Option<(String, String, String)>`

- **Does:** B's `docdefaults_ppr_spacing`, with Word's factory spacing standing in when B has no `w:docDefaults` at all.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L325; `docdefaults_ppr_spacing` (document_comparer.rs:336) L329
- **Calls (std/external/unresolved):** `.is_none`, `.to_string`, `Some`, `W::name`
- **Called by:** `merge_normal_style_spacing` (document_comparer.rs:777)

#### `docdefaults_ppr_spacing`  (L336-343)
`fn docdefaults_ppr_spacing(dom: &Dom, styles_root: NodeId) -> Option<(String, String, String)>`

- **Does:** The stylesheet's `docDefaults/pPrDefault/pPr/spacing` as (after, line, lineRule), each "" when absent.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L337,338,339,340; `Dom::attribute` (xmllinq/mod.rs:874) L341
- **Calls (std/external/unresolved):** `.to_string`, `.unwrap_or`, `Some`, `W::name`, `get`
- **Called by:** `b_docdefaults_ppr_spacing` (document_comparer.rs:324); `merge_normal_style_spacing` (document_comparer.rs:777)

#### `effective_para_spacing`  (L358-436)
`fn effective_para_spacing( dom: &Dom, styles_root: NodeId, by_id: &std::collections::HashMap<String, NodeId>, style_id: &str, table_style: Option<&str>, ) -> ([String; 4], [bool; 4], [bool; 4])`

- **Does:** M487 — effective paragraph spacing for `style_id` under one stylesheet: (after, before, line, lineRule), each resolved through the basedOn chain then docDefaults, with OOXML implicit defaults materialized ("0", "0", "240", "auto") so two stylesheets always compare attr-by-attr.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L371,372,382,393,394,395...; `Dom::attribute` (xmllinq/mod.rs:874) L376,384,399,403; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L404
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.clone`, `.copied`, `.enumerate`, `.get`, `.is_none`, `.is_some_and`, `.iter`, `.map`, `.or`, `.to_string`, `Some`, `W::name`, `W::p_pr`, `W::val`, `chain_vals`, `std::array::from_fn`
- **Macros:** `matches!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::paragraph_spacing_tracks_table_precedence_and_missing_ancestors` (document_comparer.rs:8889)

#### `next_free_revision_id`  (L469-481)
`fn next_free_revision_id(dom: &Dom, styles_root: NodeId) -> u32`

- **Does:** The next free revision id under `styles_root`: one greater than the maximum numeric `w:id` on any revision record present (0 when none).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L472; `Dom::attribute` (xmllinq/mod.rs:874) L473
- **Calls (std/external/unresolved):** `.max`, `.parse`, `.saturating_add`, `Some`, `W::id`, `W::name`
- **Called by:** `style_record_id` (document_comparer.rs:491); `mark_adopted_hf_content_as_inserted` (document_comparer.rs:5444); `delete_story_closing_mark` (document_comparer.rs:5585); `mark_hf_part_content_as_deleted` (document_comparer.rs:5658); `compare_documents_impl` (document_comparer.rs:6501); `tests::next_free_revision_id_is_max_plus_one_across_change_families` (document_comparer.rs:8123); `tests::next_free_revision_id_counts_list_move_range_and_cell_carriers` (document_comparer.rs:8151); `coverage_boundary_tests::revision_ids_ignore_missing_and_nonnumeric_values_and_saturate` (document_comparer.rs:10726)

#### `style_record_id`  (L491-513)
`fn style_record_id( dom: &Dom, styles_root: NodeId, block: NodeId, settings: &WmlComparerSettings, ) -> String`

- **Does:** The `w:id` of a new `w:pPrChange`/`w:rPrChange` under `block`.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L498,499,509; `Dom::parent` (xmllinq/mod.rs:597) L501; `Dom::name_is` (xmllinq/mod.rs:535) L502; `Dom::element` (xmllinq/mod.rs:655) L507,508; `next_free_revision_id` (document_comparer.rs:469) L512
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.filter`, `.find_map`, `.into_iter`, `.map`, `.to_string`, `.unwrap_or_else`, `Some`, `W::author`, `W::date`, `W::id`, `W::name`, `ours`
- **Called by:** `merge_normal_style_spacing` (document_comparer.rs:777); `cascade_normal_change_to_based_styles` (document_comparer.rs:1124); `append_style_change_record` (document_comparer.rs:2103); `merge_normal_style_rpr` (document_comparer.rs:4343)

#### `on_off_is_off`  (L544-546)
`fn on_off_is_off(dom: &Dom, el: NodeId) -> bool`

- **Does:** An on/off element set to off.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) (by name) L545
- **Calls (std/external/unresolved):** `Some`, `W::val`
- **Macros:** `matches!`
- **Called by:** `doc_default_ppr_delta` (document_comparer.rs:566); `coverage_package_source_matrix_tests::defaults_delta_distinguishes_implicit_on_off_and_explicit_overrides` (document_comparer.rs:11659)

#### `doc_default_ppr_delta`  (L566-680)
`fn doc_default_ppr_delta( dom: &mut Dom, out_root: NodeId, b_root: NodeId, b_style: Option<NodeId>, ) -> DocDefaultPprDelta`

- **Does:** The docDefaults paragraph properties the after/before/line rule of [`merge_normal_style_spacing`] does not cover, as Word writes them into the live Normal.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L573,574,575,579,581; `Dom::elements` (xmllinq/mod.rs:640) L585,629; `Dom::name` (xmllinq/mod.rs:526) L586,630; `XName::local_name` (xmllinq/mod.rs:208) (by name) L587,655; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L588,610,656; `coverage_boundary_tests::child` (document_comparer.rs:8774) L598,599,650; `style_prop_signature` (document_comparer.rs:1485) L600; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L604; `Dom::new_element` (xmllinq/mod.rs:478) L606,631; `Dom::attributes` (xmllinq/mod.rs:885) L609,654; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L611,616,621,632,640; `on_off_is_off` (document_comparer.rs:544) L624,637; `Dom::add` (xmllinq/mod.rs:1058) L633; `Dom::attribute` (xmllinq/mod.rs:874) L665
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_str`, `.contains`, `.flatten`, `.into_iter`, `.iter`, `.map`, `.or_else`, `.push`, `.retain_mut`, `.to_string`, `.unwrap_or_else`, `Some`, `String::new`, `Vec::new`, `W::name`, `W::p_pr`, `W::val`, `dd_ppr`, `get`, `sp`
- **Macros:** `matches!`
- **Called by:** `merge_normal_style_spacing` (document_comparer.rs:777); `coverage_package_source_matrix_tests::defaults_delta_distinguishes_implicit_on_off_and_explicit_overrides` (document_comparer.rs:11659); `namespace_declared_default_property_owner_tests::locally_declared_namespace_metadata_never_becomes_a_paragraph_property_delta` (document_comparer.rs:12423)

#### `sync_normal_ppr_with_revised`  (L686-758)
`fn sync_normal_ppr_with_revised(dom: &mut Dom, ppr: NodeId, b_style: Option<NodeId>)`

- **Does:** M64/M70/M72: after the spacing merge, Normal's other declared pPr children follow the revision — B's `w:ind` replaces A's, A-only children drop into the pPrChange record, B's own children are copied in schema order.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L692,693,694,700,742,751; `Dom::remove` (xmllinq/mod.rs:1102) L695,738; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L698,754; `Dom::nodes` (xmllinq/mod.rs:604) L702; `Dom::add_before_self` (xmllinq/mod.rs:1114) L708; `Dom::add` (xmllinq/mod.rs:1058) L709; `Dom::add_first` (xmllinq/mod.rs:1085) L712; `Dom::elements` (xmllinq/mod.rs:640) L727,744; `Dom::name` (xmllinq/mod.rs:526) L731,746; `XName::local_name` (xmllinq/mod.rs:208) (by name) L750; `insert_child_by_rank` (document_comparer.rs:1327) L755
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.collect`, `.copied`, `.filter`, `.get`, `.into_iter`, `.is_some`, `.iter`, `.position`, `.to_string`, `W::name`
- **Called by:** `merge_normal_style_spacing` (document_comparer.rs:777); `coverage_package_source_matrix_tests::normal_sync_preserves_history_and_replaces_indent_in_schema_order` (document_comparer.rs:11731)

#### `merge_normal_style_spacing`  (L777-1109)
`fn merge_normal_style_spacing( dom: &mut Dom, out_root: NodeId, b_root: NodeId, settings: &WmlComparerSettings, ) -> bool`

- **Does:** M-PAG mechanism 2: rewrite the output stylesheet's Normal to B's target spacing with a `w:pPrChange` holding A's old pPr.
- **Calls (crate):** `find_normal_style` (document_comparer.rs:271) L783,803; `normal_spacing` (document_comparer.rs:299) L787,863; `docdefaults_ppr_spacing` (document_comparer.rs:336) L795,861; `b_docdefaults_ppr_spacing` (document_comparer.rs:324) L807,862; `Dom::element` (xmllinq/mod.rs:655) L808,809,811,814,867,868...; `spacing_before_attr` (document_comparer.rs:311) L876,877,878; `doc_default_ppr_delta` (document_comparer.rs:566) L932; `Dom::elements` (xmllinq/mod.rs:640) L941; `Dom::name` (xmllinq/mod.rs:526) L945; `style_prop_signature` (document_comparer.rs:1485) L948; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L985; `Dom::descendants` (xmllinq/mod.rs:668) L986; `Dom::remove` (xmllinq/mod.rs:1102) L987,1066,1086; `Dom::new_element` (xmllinq/mod.rs:478) L992,994,1012,1026,1055,1075...; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L996,999,1002,1037,1061,1063...; `Dom::add` (xmllinq/mod.rs:1058) L1004,1016,1106,1107; `Dom::add_before_self` (xmllinq/mod.rs:1114) L1015; `Dom::add_first` (xmllinq/mod.rs:1085) L1027,1056; `set` (ambiguous: `deeper_boundary_tests::set` (comparer/comments.rs:1863); `set` (comparer/tables.rs:29)) L1043,1044,1045,1046; `sync_normal_ppr_with_revised` (document_comparer.rs:686) L1047,1069; `insert_child_by_rank` (document_comparer.rs:1327) L1076,1088; `style_record_id` (document_comparer.rs:491) L1094
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_ref`, `.as_str`, `.clone`, `.collect`, `.filter`, `.into_iter`, `.into`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.map`, `.or_else`, `.sort`, `.then`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `Some`, `String::new`, `W::name`, `W::p_pr`, `b_eff`, `ctx`, `dd_val`, `pick`, `rest_signature`, `sp_node`, `stored_sp_node`, `stored`
- **Macros:** `matches!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `tests::merge_normals` (document_comparer.rs:8421); `tests::normal_takes_b_doc_default_indent_and_justification` (document_comparer.rs:8615); `tests::normal_neutralizes_a_doc_default_indent_and_justification` (document_comparer.rs:8657); `tests::normal_clears_a_doc_default_borders` (document_comparer.rs:8710)

#### `cascade_normal_change_to_based_styles`  (L1124-1284)
`fn cascade_normal_change_to_based_styles( dom: &mut Dom, styles_root: NodeId, settings: &WmlComparerSettings, ) -> bool`

- **Does:** M111 — cascade Normal's pPrChange/rPrChange onto basedOn=Normal styles.
- **Calls (crate):** `find_normal_style` (document_comparer.rs:271) L1129; `Dom::element` (xmllinq/mod.rs:655) L1132,1134,1135,1137,1142,1143...; `Dom::elements` (xmllinq/mod.rs:640) L1148,1179,1260; `Dom::attribute` (xmllinq/mod.rs:874) L1150,1153,1158,1194,1195,1198; `Dom::new_element` (xmllinq/mod.rs:478) L1168,1177,1209,1233,1239; `Dom::add_before_self` (xmllinq/mod.rs:1114) L1170; `Dom::add` (xmllinq/mod.rs:1058) L1171,1187,1191,1222,1223,1234...; `Dom::name` (xmllinq/mod.rs:526) L1180,1183,1261; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L1186,1190,1252,1265; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1202,1205,1211,1212,1217,1241...; `style_record_id` (document_comparer.rs:491) L1210,1240; `Dom::descendants` (xmllinq/mod.rs:668) L1254; `Dom::remove` (xmllinq/mod.rs:1102) L1255,1273; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1270,1271,1275; `complete_attributes` (document_comparer.rs:2054) L1271; `add_rpr_child_in_order` (document_comparer.rs:3309) L1275
- **Calls (std/external/unresolved):** `.and_then`, `.contains`, `.is_none`, `.map`, `.to_string`, `Some`, `W::name`, `W::val`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::normal_cascade_preserves_partial_spacing_owners_without_inventing_metrics` (document_comparer.rs:10330); `coverage_boundary_tests::normal_history_cascade_keeps_own_font_language_and_spacing_slots_and_existing_records` (document_comparer.rs:11146)

#### `insert_child_by_rank`  (L1327-1343)
`fn insert_child_by_rank( dom: &mut Dom, parent: NodeId, child: NodeId, local: &str, rank_of: &dyn Fn(&str) -> usize, )`

- **Does:** Insert `child` under `parent` at the position `order` gives its `local` name: immediately before the first existing child that sorts after it, else appended.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1335; `Dom::name` (xmllinq/mod.rs:526) L1336; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1337; `Dom::add_before_self` (xmllinq/mod.rs:1114) L1340; `Dom::add` (xmllinq/mod.rs:1058) L1341
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.is_some_and`, `rank_of`
- **Called by:** `sync_normal_ppr_with_revised` (document_comparer.rs:686); `merge_normal_style_spacing` (document_comparer.rs:777); `restore_deleted_paragraph_spacing` (document_comparer.rs:1415); `merge_revised_style_definitions` (document_comparer.rs:2160); `factory_scaffold_bake_b_styles` (document_comparer.rs:2616); `normalize_word_paragraph_style_line` (document_comparer.rs:2948); `bake_bothsides_dd_disabling_neutralizers` (document_comparer.rs:3909); `resolve_redefined_style_metrics` (document_comparer.rs:4063); `compare_documents_impl` (document_comparer.rs:6501)

#### `style_child_rank`  (L1346-1351)
`fn style_child_rank(local: &str) -> usize`

- **Does:** Rank of a `w:style` child in [`STYLE_CHILD_ORDER`].
- **Calls (std/external/unresolved):** `.iter`, `.position`, `.unwrap_or`
- **Called by:** none found in src

#### `ppr_child_rank`  (L1354-1360)
`fn ppr_child_rank(local: &str) -> usize`

- **Does:** Rank of a `w:pPr` child in [`crate::comparer::order_tables::PPR_ORDER`].
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.map`, `.unwrap_or`
- **Called by:** none found in src

#### `merge_revised_font_table`  (L1384-1413)
`fn merge_revised_font_table(out_xml: &str, b_xml: &str) -> Option<String>`

- **Does:** M492: deleted paragraphs keep their A-ORIGINAL direct spacing.
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L1387; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L1388,1389; `Dom::root` (xmllinq/mod.rs:822) L1390; `Dom::elements` (xmllinq/mod.rs:640) L1391,1397,1402; `Dom::attribute` (xmllinq/mod.rs:874) L1394,1398; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L1401; `Dom::name` (xmllinq/mod.rs:526) L1403; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1404; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1404; `Dom::remove` (xmllinq/mod.rs:1102) L1406; `Dom::add` (xmllinq/mod.rs:1058) L1409; `Dom::serialize_element` (xmllinq/mod.rs:1299) L1412
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter_map`, `.into_iter`, `.is_none_or`, `.is_some_and`, `.map`, `.starts_with`, `.then`, `Some`, `W::name`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `tests::revised_fonts_join_the_font_table` (document_comparer.rs:8082); `coverage_boundary_tests::empty_missing_and_non_xml_package_parts_are_safe_noops` (document_comparer.rs:9962)

#### `restore_deleted_paragraph_spacing`  (L1415-1471)
`fn restore_deleted_paragraph_spacing(a_xml: &str, out_xml: &str) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L1419,1443; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L1420,1444; `Dom::root` (xmllinq/mod.rs:822) L1421,1445; `Dom::descendants` (xmllinq/mod.rs:668) L1422,1447; `Dom::attribute` (xmllinq/mod.rs:874) L1423,1448; `Dom::element` (xmllinq/mod.rs:655) L1426,1427,1454,1457,1459,1460; `Dom::attributes` (xmllinq/mod.rs:885) L1429; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1432; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1433; `Dom::new_element` (xmllinq/mod.rs:478) L1463; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1465; `insert_child_by_rank` (document_comparer.rs:1327) L1467; `Dom::serialize_element` (xmllinq/mod.rs:1299) L1470
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.get`, `.insert`, `.into_iter`, `.is_empty`, `.is_some_and`, `.is_some`, `.map`, `.then`, `.to_string`, `Some`, `W::name`, `W::p_pr`, `W::p`, `W::r_pr`, `crate::namespaces::W14::name`, `std::collections::HashMap::new`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `tests::restored_deleted_spacing_leaves_scratch_unids_behind` (document_comparer.rs:8044); `coverage_boundary_tests::empty_missing_and_non_xml_package_parts_are_safe_noops` (document_comparer.rs:9962); `coverage_boundary_tests::deleted_spacing_restoration_refuses_unmarked_missing_and_already_spaced_paragraphs` (document_comparer.rs:10809)

#### `is_style_prop_noise`  (L1475-1480)
`fn is_style_prop_noise(name: &crate::xmllinq::XName) -> bool`

- **Does:** Children of a style's `w:pPr`/`w:rPr` that are not formatting: revision records, section properties, and revision-save ids.
- **Calls (crate):** `XName::local_name` (xmllinq/mod.rs:208) (by name) L1477
- **Macros:** `matches!`
- **Called by:** `declared_props_signature` (document_comparer.rs:1516); `effective_chain_props` (document_comparer.rs:1696); `complete_from_defaults` (document_comparer.rs:2023); `merge_revised_style_definitions` (document_comparer.rs:2160)

#### `style_prop_signature`  (L1485-1511)
`fn style_prop_signature(dom: &Dom, node: NodeId) -> String`

- **Does:** Order-independent signature of one formatting element: expanded name, its non-`rsid` attributes sorted by name, then its children's signatures in document order (child order carries meaning inside `w:tabs`, `w:pBdr`, …).
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L1486; `XName::clark` (xmllinq/mod.rs:226) (by name) L1489,1497; `Dom::attributes` (xmllinq/mod.rs:885) L1490; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1494; `Dom::is_namespace_declaration` (xmllinq/mod.rs:996) L1495; `Dom::elements` (xmllinq/mod.rs:640) L1506; `style_prop_signature` (document_comparer.rs:1485) L1508
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.map`, `.push_str`, `.push`, `.sort`, `.starts_with`, `.to_ascii_lowercase`, `String::new`
- **Called by:** `doc_default_ppr_delta` (document_comparer.rs:566); `merge_normal_style_spacing` (document_comparer.rs:777); `style_prop_signature` (document_comparer.rs:1485); `declared_props_signature` (document_comparer.rs:1516); `effective_chain_props` (document_comparer.rs:1696); `coverage_boundary_tests::style_cycles_stop_and_self_based_style_does_not_hide_own_properties` (document_comparer.rs:8952)

#### `declared_props_signature`  (L1516-1530)
`fn declared_props_signature(dom: &Dom, style: NodeId, local: &str) -> String`

- **Does:** Signature of a style's DECLARED `w:pPr`/`w:rPr` block: the formatting children it writes itself, ignoring everything it inherits.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1517; `Dom::elements` (xmllinq/mod.rs:640) L1521; `Dom::name` (xmllinq/mod.rs:526) L1522; `is_style_prop_noise` (document_comparer.rs:1475) L1523; `style_prop_signature` (document_comparer.rs:1485) L1526
- **Calls (std/external/unresolved):** `.join`, `.push`, `.sort`, `String::new`, `Vec::new`, `W::name`
- **Called by:** `inherited_only_style_changes` (document_comparer.rs:1601); `merge_revised_style_definitions` (document_comparer.rs:2160)

#### `effective_style_props`  (L1563-1572)
`fn effective_style_props( dom: &Dom, styles_root: NodeId, by_id: &std::collections::HashMap<String, NodeId>, style: NodeId, local: &str, default_local: &str, ) -> std::collections::BTreeMap<String, String>`

- **Does:** A style's EFFECTIVE `w:pPr`/`w:rPr`, resolved the way Word resolves a style chain: `w:docDefaults` first, then each `w:basedOn` ancestor from the root of the chain downwards, then the style's own declared properties.
- **Calls (crate):** `effective_chain_props` (document_comparer.rs:1696) L1571
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `own_style_props` (document_comparer.rs:1578); `inherited_only_style_changes` (document_comparer.rs:1601); `merge_revised_style_definitions` (document_comparer.rs:2160); `coverage_boundary_tests::effective_run_properties_merge_independent_slots_and_clear_alternatives` (document_comparer.rs:8915); `coverage_boundary_tests::style_cycles_stop_and_self_based_style_does_not_hide_own_properties` (document_comparer.rs:8952)

#### `own_style_props`  (L1578-1595)
`fn own_style_props( dom: &Dom, styles_root: NodeId, by_id: &std::collections::HashMap<String, NodeId>, style: NodeId, local: &str, default_local: &str, ) -> std::collections::BTreeMap<String, String>`

- **Does:** What `style` declares beyond its parent: each effective property of [`effective_style_props`] whose value its `basedOn` parent (the docDefaults for a root style) does not already give.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1586; `Dom::attribute` (xmllinq/mod.rs:874) L1588; `effective_chain_props` (document_comparer.rs:1696) L1591; `effective_style_props` (document_comparer.rs:1563) L1592
- **Calls (std/external/unresolved):** `.and_then`, `.copied`, `.filter`, `.get`, `.retain`, `Some`, `W::name`, `W::val`
- **Called by:** `inherited_only_style_changes` (document_comparer.rs:1601); `coverage_boundary_tests::effective_run_properties_merge_independent_slots_and_clear_alternatives` (document_comparer.rs:8915); `coverage_boundary_tests::style_cycles_stop_and_self_based_style_does_not_hide_own_properties` (document_comparer.rs:8952)

#### `inherited_only_style_changes`  (L1601-1667)
`fn inherited_only_style_changes( dom: &Dom, out_root: NodeId, a_root: NodeId, b_root: NodeId, ) -> std::collections::HashSet<(NodeId, &'static str)>`

- **Does:** Freeze the style blocks whose declared difference is only inherited.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1608,1620,1633; `Dom::attribute` (xmllinq/mod.rs:874) L1612; `style_match_key` (document_comparer.rs:1843) L1621,1637; `find_normal_style` (document_comparer.rs:271) L1631; `Dom::element` (xmllinq/mod.rs:655) L1648,1649; `declared_props_signature` (document_comparer.rs:1516) L1652,1653; `effective_style_props` (document_comparer.rs:1563) L1654,1655; `own_style_props` (document_comparer.rs:1578) L1659,1660
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.entry`, `.filter_map`, `.get`, `.insert`, `.into_iter`, `.is_some_and`, `.is_some`, `.or_insert`, `.to_string`, `Some`, `W::name`, `by_id`, `by_key`, `std::collections::HashMap::new`, `std::collections::HashSet::new`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::inherited_only_records_are_suppressed_but_inbound_history_survives` (document_comparer.rs:8975)

#### `remove_inherited_only_style_records`  (L1672-1692)
`fn remove_inherited_only_style_records( dom: &mut Dom, suppressed: &std::collections::HashSet<(NodeId, &'static str)>, ) -> bool`

- **Does:** Remove newly synthesized inherited-only history after live metrics have been normalized.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1683,1685; `Dom::remove` (xmllinq/mod.rs:1102) L1687
- **Calls (std/external/unresolved):** `.and_then`, `W::name`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::inherited_only_records_are_suppressed_but_inbound_history_survives` (document_comparer.rs:8975)

#### `effective_chain_props`  (L1696-1778)
`fn effective_chain_props( dom: &Dom, styles_root: NodeId, by_id: &std::collections::HashMap<String, NodeId>, start: Option<NodeId>, local: &str, default_local: &str, ) -> std::collections::BTreeMap<String, String>`

- **Does:** [`effective_style_props`] from `start` up its `basedOn` chain; `None` resolves the docDefaults alone.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1713,1757,1759,1760,1765; `Dom::attribute` (xmllinq/mod.rs:874) L1715; `Dom::elements` (xmllinq/mod.rs:640) L1726; `Dom::name` (xmllinq/mod.rs:526) L1727; `is_style_prop_noise` (document_comparer.rs:1475) L1728; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1731,1737,1742,1744; `XName::clark` (xmllinq/mod.rs:226) (by name) L1732,1735,1748,1753; `style_prop_signature` (document_comparer.rs:1485) L1732; `Dom::attributes` (xmllinq/mod.rs:885) L1736; `Dom::is_namespace_declaration` (xmllinq/mod.rs:996) L1738; `XName::namespace` (xmllinq/mod.rs:213) (by name) L1748
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.contains`, `.copied`, `.entry`, `.get`, `.insert`, `.into_iter`, `.join`, `.len`, `.map`, `.name`, `.or_default`, `.push`, `.remove`, `.reverse`, `.starts_with`, `.to_ascii_lowercase`, `Props::new`, `Slots::new`, `Vec::new`, `W::name`, `W::val`, `absorb`, `std::collections::HashSet::new`
- **Macros:** `format!`
- **Called by:** `effective_style_props` (document_comparer.rs:1563); `own_style_props` (document_comparer.rs:1578); `authored_style_chain_boundary_tests::inherited_style_font_alternatives_and_missing_blocks_keep_exact_declarations` (document_comparer.rs:12048); `authored_style_chain_boundary_tests::based_on_cycles_stop_after_unique_authored_owners_with_nearest_properties_winning` (document_comparer.rs:12093); `authored_style_chain_boundary_tests::bounded_style_walk_reads_exactly_the_nearest_thirty_two_owners` (document_comparer.rs:12115)

#### `styles_the_revision_lacks`  (L1790-1810)
`fn styles_the_revision_lacks( dom: &mut Dom, out_root: NodeId, b_root: NodeId, ) -> Vec<((String, String), NodeId)>`

- **Does:** The key a style is matched across the two stylesheets by: `(type, name)`, falling back to the styleId when the style carries no `w:name`.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1795,1800; `style_match_key` (document_comparer.rs:1843) L1798,1803; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L1808
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter_map`, `.filter`, `.into_iter`, `.map`, `Some`, `W::name`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::absent_revised_styles_restore_distinct_twins_and_skip_removed_targets` (document_comparer.rs:9154)

#### `restore_styles`  (L1814-1838)
`fn restore_styles( dom: &mut Dom, styles_root: NodeId, saved: Vec<((String, String), NodeId)>, ) -> bool`

- **Does:** Put each saved style back in place of the one its key finds.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1822; `style_match_key` (document_comparer.rs:1843) L1825; `Dom::serialize_element` (xmllinq/mod.rs:1299) L1829; `Dom::replace_with` (xmllinq/mod.rs:1149) L1830
- **Calls (std/external/unresolved):** `.as_ref`, `.contains`, `.find`, `.insert`, `.into_iter`, `Some`, `W::name`, `std::collections::HashSet::new`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::absent_revised_styles_restore_distinct_twins_and_skip_removed_targets` (document_comparer.rs:9154)

#### `style_match_key`  (L1843-1858)
`fn style_match_key(dom: &Dom, style: NodeId) -> Option<(String, String)>`

- **Does:** The key Word pairs two stylesheets' styles by: the type, and the name (the id when it has none).
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L1844,1850,1851; `Dom::element` (xmllinq/mod.rs:655) L1848; `is_built_in` (builtin_styles.rs:394) L1852
- **Calls (std/external/unresolved):** `.and_then`, `.or_else`, `.to_ascii_lowercase`, `.to_string`, `.unwrap_or`, `Some`, `W::name`, `W::val`
- **Called by:** `inherited_only_style_changes` (document_comparer.rs:1601); `styles_the_revision_lacks` (document_comparer.rs:1790); `restore_styles` (document_comparer.rs:1814); `complete_based_style_change_records` (document_comparer.rs:1951); `merge_revised_style_definitions` (document_comparer.rs:2160); `copy_missing_styles_by_name` (document_comparer.rs:3623); `resolve_redefined_style_metrics` (document_comparer.rs:4063); `compare_documents_impl` (document_comparer.rs:6501)

#### `complete_root_style_change_records`  (L1873-1943)
`fn complete_root_style_change_records( dom: &mut Dom, styles_root: NodeId, settings: &WmlComparerSettings, ) -> bool`

- **Does:** Word writes a root style's recorded old properties out in full against the docDefaults (c719b900f0, f1257ca7ea, 221577c35b, 4eff11f045): the old block holds every property the docDefaults set, at the docDefaults value where the original's style leaves it to them, with `w:rFonts`, `w:lang` and `w:spacing` completed att...
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1878,1881,1882,1890,1891,1899...; `Dom::elements` (xmllinq/mod.rs:640) L1883,1894,1924; `Dom::attribute` (xmllinq/mod.rs:874) L1895; `record` (ambiguous: `residual_current_point_contract_tests::record` (convert/metafile.rs:2128); `record` (edit/record.rs:181)) L1902,1910; `Dom::new_element` (xmllinq/mod.rs:478) L1918,1923; `add_child_in_rank_order` (document_comparer.rs:2089) L1919; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L1925; `Dom::add` (xmllinq/mod.rs:1058) L1926; `append_style_change_record` (document_comparer.rs:2103) L1928; `complete_from_defaults` (document_comparer.rs:2023) L1939
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.filter`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.unwrap_or`, `Some`, `W::name`, `defaults`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::root_records_gain_missing_companion_and_keep_recorded_original` (document_comparer.rs:9109)

#### `complete_based_style_change_records`  (L1951-2008)
`fn complete_based_style_change_records(dom: &mut Dom, styles_root: NodeId, a_root: NodeId) -> bool`

- **Does:** Complete the old records of each paragraph style with a `w:basedOn` from what the original's docDefaults and chain gave the same style (matched by type and name), so Reject All gives back the original's look rather than Word's built-ins: see [`crate::revision_processor::style_records::complete_from_original_chain`].
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1952,1960,1975; `style_match_key` (document_comparer.rs:1843) L1955,1961,1984; `Dom::element` (xmllinq/mod.rs:655) L1965,1967,1980,1990,1992,1993; `Dom::attribute` (xmllinq/mod.rs:874) L1976; `complete_from_original_chain` (revision_processor/style_records.rs:448) L1997
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.contains`, `.entry`, `.filter_map`, `.get`, `.insert`, `.into_iter`, `.is_none`, `.is_some_and`, `.is_some`, `.or_default`, `.unwrap_or`, `Some`, `W::name`, `std::collections::HashMap::new`
- **Called by:** `stylesheet_completion_matches_type_and_name_and_only_changes_old_records` (document_comparer/based_style_record_tests.rs:418); `stylesheet_completion_ignores_ineligible_styles_and_incomplete_records` (document_comparer/based_style_record_tests.rs:461); `recorded_ancestors_are_matched_in_original_ids_and_tracked_per_block` (document_comparer/based_style_record_tests.rs:495); `compare_documents_impl` (document_comparer.rs:6501)

#### `complete_from_defaults`  (L2023-2050)
`fn complete_from_defaults(dom: &mut Dom, old: NodeId, defaults: NodeId, block_local: &str) -> bool`

- **Does:** Give the recorded `old` block every property of the docDefaults `defaults` block it lacks, and complete its [`RECORD_COMPLETED_PROPS`].
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L2025; `Dom::name` (xmllinq/mod.rs:526) L2026; `is_style_prop_noise` (document_comparer.rs:1475) L2029; `XName::local_name` (xmllinq/mod.rs:208) (by name) L2032; `Dom::element` (xmllinq/mod.rs:655) L2033; `complete_attributes` (document_comparer.rs:2054) L2035; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L2039; `add_rpr_child_in_order` (document_comparer.rs:3309) L2041; `add_child_in_rank_order` (document_comparer.rs:2089) L2043
- **Calls (std/external/unresolved):** `.as_str`, `.contains`, `.to_string`
- **Called by:** `complete_root_style_change_records` (document_comparer.rs:1873); `coverage_boundary_tests::recorded_defaults_complete_slots_without_overwriting_explicit_values` (document_comparer.rs:9048)

#### `complete_attributes`  (L2054-2085)
`fn complete_attributes(dom: &mut Dom, recorded: NodeId, default: NodeId, local: &str) -> bool`

- **Does:** Give `recorded` each attribute of `default` it lacks, unless it names an alternative for that slot (an explicit `w:ascii` over `w:asciiTheme`).
- **Calls (crate):** `Dom::attributes` (xmllinq/mod.rs:885) L2067; `Dom::is_namespace_declaration` (xmllinq/mod.rs:996) L2068; `Dom::attribute` (xmllinq/mod.rs:874) L2068,2076; `XName::local_name` (xmllinq/mod.rs:208) (by name) L2073; `XName::namespace` (xmllinq/mod.rs:213) (by name) L2076; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L2080
- **Calls (std/external/unresolved):** `.any`, `.as_slice`, `.collect`, `.contains`, `.filter`, `.flat_map`, `.is_some`, `.iter`, `.map`, `.name`, `Some`, `Vec::new`
- **Called by:** `cascade_normal_change_to_based_styles` (document_comparer.rs:1124); `complete_from_defaults` (document_comparer.rs:2023)

#### `add_child_in_rank_order`  (L2089-2099)
`fn add_child_in_rank_order(dom: &mut Dom, parent: NodeId, child: NodeId, rank: fn(&str) -> usize)`

- **Does:** Insert `child` under `parent` after the last existing child that `rank` puts before it, or first.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L2090,2094; `rank` (ambiguous: `wml_order_elements_per_standard::{fn rank}` (comparer/finalize.rs:10487); `add_numbering_child_in_schema_order::{fn rank}` (comparer/footnotes.rs:293)) L2090,2094; `XName::local_name` (xmllinq/mod.rs:208) (by name) L2090,2094; `Dom::elements` (xmllinq/mod.rs:640) L2091; `Dom::add_after_self` (xmllinq/mod.rs:1131) L2096; `Dom::add_first` (xmllinq/mod.rs:1085) L2097
- **Calls (std/external/unresolved):** `.into_iter`, `.is_some_and`, `.map_or`, `.rfind`
- **Called by:** `complete_root_style_change_records` (document_comparer.rs:1873); `complete_from_defaults` (document_comparer.rs:2023)

#### `append_style_change_record`  (L2103-2126)
`fn append_style_change_record( dom: &mut Dom, styles_root: NodeId, block: NodeId, old: NodeId, change_local: &str, settings: &WmlComparerSettings, )`

- **Does:** Wrap `old` (a `w:pPr`/`w:rPr` clone) in a `w:pPrChange`/`w:rPrChange` record and append it to `block`, which is where CT_PPr / CT_RPr put it.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L2111; `style_record_id` (document_comparer.rs:491) L2112; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L2113,2114,2119; `Dom::add` (xmllinq/mod.rs:1058) L2124,2125
- **Calls (std/external/unresolved):** `Some`, `W::name`
- **Called by:** `complete_root_style_change_records` (document_comparer.rs:1873); `merge_revised_style_definitions` (document_comparer.rs:2160); `factory_scaffold_bake_b_styles` (document_comparer.rs:2616); `coverage_boundary_tests::inherited_only_records_are_suppressed_but_inbound_history_survives` (document_comparer.rs:8975); `coverage_boundary_tests::a_styles_two_records_share_one_id` (document_comparer.rs:9011)

#### `merge_revised_style_definitions`  (L2160-2600)
`fn merge_revised_style_definitions( dom: &mut Dom, out_root: NodeId, b_root: NodeId, settings: &WmlComparerSettings, a_declared_keys: &std::collections::HashSet<(String, String)>, inherited_only: &std::collections::HashSet<(NodeId, &'static str)>, ) -> bool`

- **Does:** Workstream S — adopt the REVISED document's definition of every style both documents define, recording the ORIGINAL definition on the `w:style` itself.
- **Calls (crate):** `find_normal_style` (document_comparer.rs:271) L2170,2171; `Dom::elements` (xmllinq/mod.rs:640) L2174,2184,2191,2253,2270,2364...; `Dom::attribute` (xmllinq/mod.rs:874) L2176,2316,2323,2349,2361,2367...; `style_match_key` (document_comparer.rs:1843) L2185,2195,2473; `declared_props_signature` (document_comparer.rs:1516) L2208,2209; `effective_style_props` (document_comparer.rs:1563) L2214,2215; `Dom::element` (xmllinq/mod.rs:655) L2229,2231,2238,2251,2269,2298...; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L2240,2275,2504,2510,2530,2589; `Dom::descendants` (xmllinq/mod.rs:668) L2241,2497,2590; `Dom::remove` (xmllinq/mod.rs:1102) L2242,2259,2591; `Dom::new_element` (xmllinq/mod.rs:478) L2246,2264,2505,2516,2540,2549...; `Dom::name` (xmllinq/mod.rs:526) L2256,2271; `is_style_prop_noise` (document_comparer.rs:1475) L2256,2272; `insert_child_by_rank` (document_comparer.rs:1327) L2265,2517,2562,2576; `Dom::add` (xmllinq/mod.rs:1058) L2276,2511,2533,2542,2551; `append_style_change_record` (document_comparer.rs:2103) L2279,2553,2593; `Dom::attributes` (xmllinq/mod.rs:885) L2307; `Dom::add_before_self` (xmllinq/mod.rs:1114) L2532; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L2541,2550,2581,2585
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.clone`, `.collect`, `.contains`, `.copied`, `.entry`, `.filter_map`, `.filter`, `.first`, `.get`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.map`, `.or_insert`, `.or`, `.push`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `Some`, `Vec::new`, `W14::name`, `W::name`, `W::p_pr`, `W::r_pr`, `W::val`, `attr_map`, `b_chain_has_rpr_elem`, `b_chain_has_sp_attr`, `b_chain_has`, `chain_has_sp_attr`, `dd_ppr_spacing`, `dd_rpr`, `index_by_id`, `lig_or`, `sp_val`, `std::collections::HashMap::new`, `val_or`
- **Macros:** `format!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::revised_style_merge_tracks_both_blocks_without_copying_revised_history` (document_comparer.rs:10741)

#### `factory_scaffold_bake_b_styles`  (L2616-2929)
`fn factory_scaffold_bake_b_styles( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, ) -> bool`

- **Does:** M462 — A has no styles part: rewrite the adopted B stylesheet so its docDefaults become Word's FACTORY defaults, baking each style's B-effective metrics into the style itself.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L2624,2626,2627,2628,2637,2638...; `Dom::attribute` (xmllinq/mod.rs:874) L2630,2642,2651,2698,2707,2713...; `Dom::attributes` (xmllinq/mod.rs:885) L2655; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L2662,2784,2791,2880; `Dom::remove` (xmllinq/mod.rs:1102) L2669,2786,2793; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L2671,2672; `Dom::new_element` (xmllinq/mod.rs:478) L2673,2677,2822,2830,2871,2884...; `Dom::root` (xmllinq/mod.rs:822) L2674,2678; `Dom::add` (xmllinq/mod.rs:1058) L2675,2679,2892; `Dom::add_first` (xmllinq/mod.rs:1085) L2681,2682,2881; `Dom::elements` (xmllinq/mod.rs:640) L2695; `Dom::descendants` (xmllinq/mod.rs:668) L2785,2792,2902; `insert_child_by_rank` (document_comparer.rs:1327) L2823,2831,2872,2908,2918; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L2836,2839,2885,2891; `add_rpr_child_in_order` (document_comparer.rs:3309) L2886; `append_style_change_record` (document_comparer.rs:2103) L2913,2923
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.clone`, `.collect`, `.filter_map`, `.flatten`, `.get`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.map`, `.push`, `.then_some`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `Some`, `Vec::new`, `W14::name`, `W::name`, `W::p_pr`, `W::r_pr`, `W::val`, `b_rv`, `b_sp`, `chain_has_rfonts`, `chain_lig`, `chain_rpr`, `chain_spacing`
- **Macros:** `format!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501)

#### `normalize_word_paragraph_style_line`  (L2948-3038)
`fn normalize_word_paragraph_style_line(dom: &mut Dom, styles_root: NodeId) -> bool`

- **Does:** M79 — Word-mode single-line normalization on paragraph styles.
- **Calls (crate):** `find_normal_style` (document_comparer.rs:271) L2953; `Dom::element` (xmllinq/mod.rs:655) L2954,2955,3000,3009,3018; `Dom::attribute` (xmllinq/mod.rs:874) L2962,2965,2988,2989,2993,3019; `Dom::elements` (xmllinq/mod.rs:640) L2984; `Dom::new_element` (xmllinq/mod.rs:478) L3007,3026; `Dom::add_before_self` (xmllinq/mod.rs:1114) L3010; `Dom::add` (xmllinq/mod.rs:1058) L3012; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L3021,3022,3027,3028,3029; `insert_child_by_rank` (document_comparer.rs:1327) L3033
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.collect`, `.contains`, `.filter`, `.into_iter`, `.is_none`, `.is_some`, `.to_string`, `.unwrap_or`, `Some`, `W::name`, `W::p_pr`, `W::r_pr`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `word_style_line_owned_boundary_tests::word_single_line_style_propagation_keeps_authored_history_and_unrelated_properties` (document_comparer.rs:12688)

#### `align_paragraph_style_fonts_with_normal`  (L3053-3137)
`fn align_paragraph_style_fonts_with_normal(dom: &mut Dom, styles_root: NodeId) -> bool`

- **Does:** M80 — Word-mode paragraph style rFonts alignment with Normal.
- **Calls (crate):** `find_normal_style` (document_comparer.rs:271) L3054; `Dom::element` (xmllinq/mod.rs:655) L3057,3060,3105,3108,3118,3121; `Dom::attribute` (xmllinq/mod.rs:874) L3063,3072,3090,3091,3096,3124; `Dom::elements` (xmllinq/mod.rs:640) L3086; `Dom::new_element` (xmllinq/mod.rs:478) L3111; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L3113,3131,3132; `add_rpr_child_in_order` (document_comparer.rs:3309) L3115
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.contains`, `.filter_map`, `.filter`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some`, `.map`, `.to_string`, `.unwrap_or`, `Some`, `W::name`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501)

#### `has_default_table_style`  (L3156-3168)
`fn has_default_table_style(styles_xml: &str) -> bool`

- **Does:** A style tree's `docDefaults/rPrDefault/rPr` node, if present.
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L3157; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L3158; `Dom::root` (xmllinq/mod.rs:822) L3159; `Dom::elements` (xmllinq/mod.rs:640) L3162; `Dom::attribute` (xmllinq/mod.rs:874) L3165; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L3166
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `Some`, `W::name`
- **Macros:** `matches!`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209); `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::empty_missing_and_non_xml_package_parts_are_safe_noops` (document_comparer.rs:9962)

#### `default_paragraph_line`  (L3170-3208)
`fn default_paragraph_line(styles_xml: &str) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L3171; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L3172; `Dom::root` (xmllinq/mod.rs:822) L3173; `Dom::elements` (xmllinq/mod.rs:640) L3177; `Dom::attribute` (xmllinq/mod.rs:874) L3182,3186,3190,3200; `Dom::element` (xmllinq/mod.rs:655) L3185,3195,3198,3203,3205,3206; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L3191
- **Calls (std/external/unresolved):** `.and_then`, `.copied`, `.find`, `.iter`, `.map`, `.to_string`, `.unwrap_or_else`, `Some`, `W::name`, `W::p_pr`, `W::val`, `line_of`
- **Macros:** `matches!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::empty_missing_and_non_xml_package_parts_are_safe_noops` (document_comparer.rs:9962)

#### `rpr_default`  (L3210-3214)
`fn rpr_default(dom: &Dom, styles_root: NodeId) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L3211,3212,3213
- **Calls (std/external/unresolved):** `W::name`
- **Called by:** `effective_normal_rpr_metrics` (document_comparer.rs:3224); `prune_normal_rpr_context_equal_attrs` (document_comparer.rs:3822); `bake_bothsides_dd_disabling_neutralizers` (document_comparer.rs:3909); `resolve_redefined_style_metrics` (document_comparer.rs:4063); `merge_normal_style_rpr` (document_comparer.rs:4343)

#### `effective_normal_rpr_metrics`  (L3224-3259)
`fn effective_normal_rpr_metrics( dom: &Dom, styles_root: NodeId, normal: Option<NodeId>, ) -> ([FontSlot; 4], [Option<String>; 2])`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L3229,3233,3247; `rpr_default` (document_comparer.rs:3210) L3230; `Dom::attribute` (xmllinq/mod.rs:874) L3236,3237,3248
- **Calls (std/external/unresolved):** `.and_then`, `.is_some`, `.map`, `.to_string`, `Some`, `W::name`, `W::val`, `sz_val`
- **Called by:** `merge_normal_style_rpr` (document_comparer.rs:4343)

#### `add_rpr_child_in_order`  (L3309-3336)
`fn add_rpr_child_in_order(dom: &mut Dom, rpr: NodeId, child: NodeId, local: &str)`

- **Does:** Insert `child` (a new rPr child element named `local`) under `rpr` in EG_RPrBase order: immediately after the last existing predecessor in the schema sequence, or first when no predecessor is present.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L3314; `Dom::name` (xmllinq/mod.rs:526) L3318; `XName::local_name` (xmllinq/mod.rs:208) (by name) L3321; `Dom::add_after_self` (xmllinq/mod.rs:1131) L3333; `Dom::add_first` (xmllinq/mod.rs:1085) L3334
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.find`, `.into_iter`, `.iter`, `.map`, `.position`, `.rev`, `.unwrap_or`, `Some`
- **Called by:** `cascade_normal_change_to_based_styles` (document_comparer.rs:1124); `complete_from_defaults` (document_comparer.rs:2023); `factory_scaffold_bake_b_styles` (document_comparer.rs:2616); `align_paragraph_style_fonts_with_normal` (document_comparer.rs:3053); `bake_bothsides_dd_disabling_neutralizers` (document_comparer.rs:3909); `resolve_redefined_style_metrics` (document_comparer.rs:4063); `merge_normal_style_rpr` (document_comparer.rs:4343); `tests::add_rpr_child_keeps_sz_in_schema_order_after_position` (document_comparer.rs:8174)

#### `adopt_revised_styles_chrome`  (L3348-3393)
`fn adopt_revised_styles_chrome(out: &mut PartFs, pkg2: &PartFs, out_main: &str)`

- **Does:** M-PAG mechanism 2b / M71: when the output Normal's effective run metrics differ from the REVISED document's, rewrite Normal's rPr to B's effective values with a `w:rPrChange` holding the old rPr.
- **Calls (crate):** `PartFs::part_bytes` (opc/mod.rs:493) L3378,3381; `PartFs::set_part` (opc/mod.rs:510) L3384; `PartFs::add_content_type_override` (opc/mod.rs:616) L3385; `PartFs::read_rels_for` (opc/mod.rs:756) L3386; `PartFs::add_document_relationship` (opc/mod.rs:772) L3390
- **Calls (std/external/unresolved):** `.any`, `.is_some_and`, `.is_some`, `.iter`, `.map`
- **Macros:** `format!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::revised_chrome_preserves_existing_parts_and_repairs_relationships_idempotently` (document_comparer.rs:10664)

#### `ensure_factory_package_chrome`  (L3402-3506)
`fn ensure_factory_package_chrome(out: &mut PartFs, out_main: &str)`

- **Does:** Minimal Word-like package chrome for thin demo packages (C5).
- **Calls (crate):** `PartFs::part_bytes` (opc/mod.rs:493) L3404,3429,3452; `PartFs::set_part` (opc/mod.rs:510) L3412,3435,3459,3480; `PartFs::add_content_type_override` (opc/mod.rs:616) L3413,3436,3460,3486; `PartFs::read_rels_for` (opc/mod.rs:756) L3417,3440,3464,3490; `PartFs::add_document_relationship` (opc/mod.rs:772) L3421,3444,3468,3504; `referenced_theme_part` (document_comparer.rs:3511) L3478; `PartFs::remove_relationships_by_type` (opc/mod.rs:675) L3501; `relative_rel_target` (opc/mod.rs:129) L3503
- **Calls (std/external/unresolved):** `.any`, `.as_bytes`, `.clone`, `.collect`, `.ends_with`, `.filter`, `.is_none`, `.is_some_and`, `.iter`, `.map`, `.to_vec`, `.unwrap_or_default`
- **Macros:** `format!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::revised_chrome_preserves_existing_parts_and_repairs_relationships_idempotently` (document_comparer.rs:10664)

#### `referenced_theme_part`  (L3511-3518)
`fn referenced_theme_part(out: &PartFs, main: &str) -> Option<String>`

- **Does:** The theme part `main` references, when the package holds it.
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:756) L3512; `PartFs::resolve_rel_target` (opc/mod.rs:600) L3516; `PartFs::part_bytes` (opc/mod.rs:493) L3517
- **Calls (std/external/unresolved):** `.ends_with`, `.filter`, `.find`, `.is_some`, `.iter`, `.map`
- **Called by:** `ensure_factory_package_chrome` (document_comparer.rs:3402); `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::revised_chrome_preserves_existing_parts_and_repairs_relationships_idempotently` (document_comparer.rs:10664)

#### `word_canonical_style_id`  (L3527-3615)
`fn word_canonical_style_id(name: &str) -> String`

- **Does:** Word-canonical `w:styleId` for a human-readable `w:name` (C3 / C5).
- **Calls (std/external/unresolved):** `.as_str`, `.chars`, `.collect`, `.filter`, `.into`, `.is_alphanumeric`, `.is_empty`, `.map`, `.next`, `.split`, `.to_ascii_lowercase`, `.to_uppercase`, `.trim`, `String::new`
- **Called by:** `canonicalize_style_ids` (document_comparer.rs:3667); `tests::word_canonical_style_id_preserves_toc_builtins` (document_comparer.rs:8274); `tests::word_canonical_style_id_keeps_builtin_ids_that_differ_from_their_names` (document_comparer.rs:8291); `tests::word_canonical_style_id_drops_punctuation` (document_comparer.rs:8318)

#### `copy_missing_styles_by_name`  (L3623-3659)
`fn copy_missing_styles_by_name( dom: &mut Dom, to_root: NodeId, from_root: NodeId, ) -> std::collections::HashMap<String, String>`

- **Does:** Word mode: copy the revised stylesheet's styles that the output lacks, pairing styles by type and name as Word does, not by id.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L3632,3639; `style_match_key` (document_comparer.rs:1843) L3633,3640; `Dom::attribute` (xmllinq/mod.rs:874) L3633,3641; `Dom::new_element` (xmllinq/mod.rs:478) L3652; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L3654; `Dom::add` (xmllinq/mod.rs:1058) L3655; `copy_missing_styles` (comparer/footnotes.rs:267) L3657
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.entry`, `.get`, `.insert`, `.or_insert_with`, `.push`, `.to_string`, `Some`, `Vec::new`, `W::name`, `std::collections::HashMap::new`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501)

#### `canonicalize_style_ids`  (L3667-3733)
`fn canonicalize_style_ids( dom: &mut Dom, styles_root: NodeId, ) -> std::collections::HashMap<String, String>`

- **Does:** Rename `w:styleId` values to Word-canonical ids derived from `w:name`.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L3674; `Dom::attribute` (xmllinq/mod.rs:874) L3679,3684,3709,3725; `Dom::element` (xmllinq/mod.rs:655) L3682; `word_canonical_style_id` (document_comparer.rs:3527) L3689; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L3719,3728; `Dom::descendants` (xmllinq/mod.rs:668) L3724
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_str`, `.clone`, `.collect`, `.contains`, `.get`, `.insert`, `.is_empty`, `.iter`, `.map`, `.push`, `.to_string`, `Some`, `Vec::new`, `W::name`, `W::val`, `std::collections::HashMap::new`, `std::collections::HashSet::new`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::canonical_style_ids_refuse_collisions_and_remap_only_known_references` (document_comparer.rs:9187)

#### `remap_style_refs`  (L3736-3757)
`fn remap_style_refs( dom: &mut Dom, root: NodeId, renames: &std::collections::HashMap<String, String>, ) -> usize`

- **Does:** Apply a styleId rename map to `pStyle` / `rStyle` / `tblStyle` under `root`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L3747; `Dom::attribute` (xmllinq/mod.rs:874) L3748; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L3751
- **Calls (std/external/unresolved):** `.get`, `.is_empty`, `Some`, `W::name`, `W::val`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::canonical_style_ids_refuse_collisions_and_remap_only_known_references` (document_comparer.rs:9187)

#### `materialize_builtin_paragraph_defaults`  (L3764-3781)
`fn materialize_builtin_paragraph_defaults(dom: &mut Dom, root: NodeId) -> bool`

- **Does:** Word reads a docDefaults with no pPrDefault as its built-in paragraph defaults, 160 after and 278 auto lines, while a present empty pPrDefault means single spacing (ceaf5f4).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L3765,3768; `Dom::new_element` (xmllinq/mod.rs:478) L3771,3775,3777; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L3773; `Dom::add` (xmllinq/mod.rs:1058) L3776,3778,3779
- **Calls (std/external/unresolved):** `.is_some`, `Some`, `W::name`, `W::p_pr`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::style_structure_adoption_preserves_existing_defaults_and_empty_explicit_spacing` (document_comparer.rs:9233)

#### `adopt_missing_styles_structure`  (L3786-3806)
`fn adopt_missing_styles_structure(dom: &mut Dom, out_root: NodeId, b_root: NodeId) -> bool`

- **Does:** When the A-based styles part lacks `docDefaults` or `latentStyles`, copy them from B (Word redlines always carry both when B has a full stylesheet).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L3790,3793,3798; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L3796; `Dom::add_before_self` (xmllinq/mod.rs:1114) L3799; `Dom::add_first` (xmllinq/mod.rs:1085) L3801
- **Calls (std/external/unresolved):** `.is_some`, `W::name`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::style_structure_adoption_preserves_existing_defaults_and_empty_explicit_spacing` (document_comparer.rs:9233)

#### `prune_normal_rpr_context_equal_attrs`  (L3822-3887)
`fn prune_normal_rpr_context_equal_attrs(dom: &mut Dom, out_root: NodeId) -> bool`

- **Does:** header/footer→Normal (footer knife-edge line box).
- **Calls (crate):** `find_normal_style` (document_comparer.rs:271) L3823; `Dom::element` (xmllinq/mod.rs:655) L3826,3831,3837,3849,3852,3860...; `rpr_default` (document_comparer.rs:3210) L3829; `Dom::attribute` (xmllinq/mod.rs:874) L3832,3840,3850,3853,3872,3875; `Dom::remove` (xmllinq/mod.rs:1102) L3843,3856,3882; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L3877; `Dom::attributes` (xmllinq/mod.rs:885) L3881
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.as_str`, `.is_empty`, `.map`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `Some`, `W14::name`, `W::name`, `W::val`, `dd_val`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501)

#### `bake_bothsides_dd_disabling_neutralizers`  (L3909-4048)
`fn bake_bothsides_dd_disabling_neutralizers( dom: &mut Dom, out_root: NodeId, b_root: NodeId, ) -> bool`

- **Does:** M480b — docDefaults-delta DISABLING neutralizers on both-sides merged styles.
- **Calls (crate):** `rpr_default` (document_comparer.rs:3210) L3917,3918; `Dom::element` (xmllinq/mod.rs:655) L3921,3926,3948,3959,3961,4000...; `Dom::attribute` (xmllinq/mod.rs:874) L3922,3927,3945,3950,3994,4009; `find_normal_style` (document_comparer.rs:271) L3941; `Dom::elements` (xmllinq/mod.rs:640) L3942,3971; `Dom::new_element` (xmllinq/mod.rs:478) L4015,4021,4037; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L4016,4022; `insert_child_by_rank` (document_comparer.rs:1327) L4038; `add_rpr_child_in_order` (document_comparer.rs:3309) L4043; `XName::local_name` (xmllinq/mod.rs:208) (by name) L4043
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.copied`, `.filter_map`, `.get`, `.into_iter`, `.is_none_or`, `.is_some_and`, `.is_some`, `.map`, `.parse`, `.sort_by_key`, `.to_string`, `.unwrap_or`, `Some`, `W14::name`, `W::name`, `W::p_pr`, `W::r_pr`, `W::val`, `based_of`, `chain_declares`, `kern_of`, `lig_of`, `mk`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::disabling_default_neutralizers_preserve_tracked_history_at_inheritance_limit` (document_comparer.rs:10247)

#### `resolve_redefined_style_metrics`  (L4063-4341)
`fn resolve_redefined_style_metrics(dom: &mut Dom, out_root: NodeId, b_root: NodeId) -> bool`

- **Does:** Word's redefined paragraph styles hold B's effective metrics as a delta against the output context.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L4066,4075,4183; `Dom::attribute` (xmllinq/mod.rs:874) L4068,4082,4136,4137,4155,4164...; `coverage_boundary_tests::index` (document_comparer.rs:8792) L4071,4072; `style_match_key` (document_comparer.rs:1843) L4076,4204; `Dom::element` (xmllinq/mod.rs:655) L4081,4102,4103,4104,4109,4116...; `rpr_default` (document_comparer.rs:3210) L4126; `Dom::new_element` (xmllinq/mod.rs:478) L4239,4252,4288,4313,4323; `insert_child_by_rank` (document_comparer.rs:1327) L4240,4314,4324; `Dom::add_first` (xmllinq/mod.rs:1085) L4253; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L4265,4266,4283,4289,4332; `Dom::attributes` (xmllinq/mod.rs:885) L4270,4336; `Dom::remove` (xmllinq/mod.rs:1102) L4271,4278,4337; `add_rpr_child_in_order` (document_comparer.rs:3309) L4290
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.as_str`, `.clone`, `.collect`, `.contains`, `.copied`, `.entry`, `.filter_map`, `.find_map`, `.get`, `.into_iter`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.map`, `.or_else`, `.or_insert`, `.push`, `.sort_by_key`, `.then_some`, `.then`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `.zip`, `Some`, `Vec::new`, `W::name`, `W::p_pr`, `W::r_pr`, `W::val`, `chain`, `dd_ppr`, `font_slot`, `holders`, `line_val`, `parent`, `ppr_of`, `rpr_of`, `run_val`, `slot_key`, `spacing_val`, `std::collections::HashMap::new`
- **Macros:** `format!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `redefined_style_chain_source_owner_tests::redefined_style_metrics_bound_cycles_depth_and_default_owners_without_source_history_loss` (document_comparer.rs:12361)

#### `merge_normal_style_rpr`  (L4343-4599)
`fn merge_normal_style_rpr( dom: &mut Dom, out_root: NodeId, b_root: NodeId, settings: &WmlComparerSettings, ) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `find_normal_style` (document_comparer.rs:271) L4349,4352; `Dom::element` (xmllinq/mod.rs:655) L4362,4376,4389,4400,4419,4429...; `effective_normal_rpr_metrics` (document_comparer.rs:3224) L4367,4368; `rpr_default` (document_comparer.rs:3210) L4378,4428,4471,4506; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L4381,4473,4488,4494; `Dom::descendants` (xmllinq/mod.rs:668) L4382; `Dom::remove` (xmllinq/mod.rs:1102) L4383,4433,4476,4571; `Dom::new_element` (xmllinq/mod.rs:478) L4387,4392,4403,4440,4525,4538...; `Dom::add` (xmllinq/mod.rs:1058) L4395,4489,4544,4596,4597; `Dom::add_first` (xmllinq/mod.rs:1085) L4404; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L4409,4410,4445,4526,4539,4576...; `add_rpr_child_in_order` (document_comparer.rs:3309) L4441,4498,4527,4578; `Dom::elements` (xmllinq/mod.rs:640) L4463,4480; `Dom::name` (xmllinq/mod.rs:526) L4464,4482; `XName::local_name` (xmllinq/mod.rs:208) (by name) L4465,4493; `Dom::replace_with` (xmllinq/mod.rs:1149) L4474,4497; `Dom::attribute` (xmllinq/mod.rs:874) L4513,4518,4530,4535,4553; `style_record_id` (document_comparer.rs:491) L4584
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.clone`, `.collect`, `.filter_map`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.map`, `.or_else`, `.or`, `.then_some`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `.zip`, `Some`, `W14::name`, `W::name`, `W::p_pr`, `W::val`, `dd_elem`, `lang_attr`, `metric`, `structured`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `tests::normal_takes_b_theme_fonts_from_its_doc_defaults` (document_comparer.rs:8330); `tests::merge_normals` (document_comparer.rs:8421); `tests::normal_takes_b_run_defaults_when_only_b_paragraph_props_are_stored` (document_comparer.rs:8565); `coverage_boundary_tests::normal_run_adoption_preserves_complete_old_properties_and_revised_slot_ownership` (document_comparer.rs:10865)

#### `normalize_strict_namespaces`  (L4607-4625)
`fn normalize_strict_namespaces(xml: &str) -> std::borrow::Cow<'_, str>`

- **Does:** Merge every direct `w:body` child of `root` into one body and return it.
- **Calls (std/external/unresolved):** `.contains`, `.replace`, `std::borrow::Cow::Borrowed`, `std::borrow::Cow::Owned`
- **Called by:** `main_part_xml` (document_comparer.rs:6467); `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::strict_namespace_normalization_rewrites_only_the_three_ooxml_families` (document_comparer.rs:10034)

#### `merged_body`  (L4630-4659)
`fn merged_body(dom: &mut Dom, root: NodeId) -> Option<NodeId>`

- **Does:** Some producers emit multiple `w:body` elements (invalid, but real — e.g.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L4631; `Dom::nodes` (xmllinq/mod.rs:604) L4640; `Dom::remove` (xmllinq/mod.rs:1102) L4641,4656; `Dom::is_element` (xmllinq/mod.rs:502) L4642; `Dom::name` (xmllinq/mod.rs:526) L4642; `Dom::add` (xmllinq/mod.rs:1058) L4650,4653
- **Calls (std/external/unresolved):** `.as_ref`, `.copied`, `.first`, `.len`, `.push`, `Some`, `Vec::new`, `W::body`, `W::name`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::merged_bodies_preserve_content_order_and_only_last_section_properties` (document_comparer.rs:8839)

#### `wrap_bare_breaks`  (L4666-4682)
`fn wrap_bare_breaks(dom: &mut Dom, root: NodeId)`

- **Does:** Wrap each `w:br` a producer left directly under a paragraph (the schema forbids it) in a run of its own with no properties, `<w:r><w:br/></w:r>`, the way Word reads it.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L4667; `Dom::parent` (xmllinq/mod.rs:597) L4671; `Dom::name` (xmllinq/mod.rs:526) L4672; `Dom::new_element` (xmllinq/mod.rs:478) L4677; `Dom::add_before_self` (xmllinq/mod.rs:1114) L4678; `Dom::remove` (xmllinq/mod.rs:1102) L4679; `Dom::add` (xmllinq/mod.rs:1058) L4680
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.filter`, `.into_iter`, `.is_some_and`, `Some`, `W::name`, `W::p`, `W::r`
- **Called by:** `pre_process_markup` (document_comparer.rs:4713)

#### `defined_style_ids`  (L4685-4697)
`fn defined_style_ids(pkg: &PartFs, main: &str) -> Option<std::collections::HashSet<String>>`

- **Does:** The styleIds the package's stylesheet defines, or `None` without one.
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:756) L4686; `PartFs::resolve_rel_target` (opc/mod.rs:600) L4690; `PartFs::part_string` (opc/mod.rs:498) L4692; `Dom::new` (xmllinq/mod.rs:387) L4693; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L4694; `Dom::root` (xmllinq/mod.rs:822) L4695; `defined_style_ids` (comparer/footnotes.rs:217) L4696
- **Calls (std/external/unresolved):** `.and_then`, `.ends_with`, `.find`, `.is_none`, `.iter`, `.map`, `Some`
- **Called by:** `pre_process_markup` (document_comparer.rs:4713)

#### `pre_process_markup`  (L4713-4894, pub)
`pub fn pre_process_markup( pkg: &mut PartFs, starting_id_for_footnotes_endnotes: i32, ) -> Result<Vec<String>, OpcError>`

- **Does:** C.1/C.2 — `WmlComparer.PreProcessMarkup` (:434) at package level: `ChangeFootnoteEndnoteReferencesToUniqueRange` (:1627) then `AddFootnotesEndnotesParts` (:1604).
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:829) L4717; `PartFs::read_rels_for` (opc/mod.rs:756) L4724,4861; `PartFs::resolve_rel_target` (opc/mod.rs:600) L4730,4731,4869; `PartFs::part_string` (opc/mod.rs:498) L4737,4740,4741,4874; `Dom::new` (xmllinq/mod.rs:387) L4743,4877; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L4744,4748,4750,4878; `Dom::root` (xmllinq/mod.rs:822) L4745,4749,4751,4879; `Dom::descendants` (xmllinq/mod.rs:668) L4758,4883; `Dom::name` (xmllinq/mod.rs:526) L4761; `change_footnote_endnote_references_to_unique_range` (comparer/footnotes.rs:38) L4766; `defined_style_ids` (document_comparer.rs:4685) L4778; `wrap_bare_breaks` (document_comparer.rs:4666) L4780; `strip_unresolved_style_refs` (comparer/footnotes.rs:233) L4786; `assign_to_all_elements` (unid.rs:60) L4794,4797; `ignore_pt14_namespace` (comparer/finalize.rs:819) L4795,4798; `fill_in_empty_footnotes_endnotes` (comparer/footnotes.rs:133) L4806,4809; `PartFs::set_part` (opc/mod.rs:510) L4813,4816,4820,4842,4890; `Dom::serialize_document` (xmllinq/mod.rs:1322) L4813,4816,4820,4890; `PartFs::add_document_relationship` (opc/mod.rs:772) L4843; `PartFs::add_content_type_override` (opc/mod.rs:616) L4848; `Dom::remove` (xmllinq/mod.rs:1102) L4888
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.clone`, `.collect`, `.ends_with`, `.filter`, `.flatten`, `.into_bytes`, `.into_iter`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.map_err`, `.map`, `.next`, `.push`, `.rsplit_once`, `.rsplit`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `Ok`, `Some`, `Vec::new`, `W::name`, `crate::namespaces::C::name`
- **Macros:** `format!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::empty_missing_and_non_xml_package_parts_are_safe_noops` (document_comparer.rs:9962); `coverage_boundary_tests::preprocessing_refuses_orphan_notes_before_mutating_package` (document_comparer.rs:9993)

#### `invalid_content`  (L4899-4901)
`fn invalid_content(msg: String) -> OpcError`

- **Does:** Markup the engine refuses (C# throws DocxodusException): an `Err` the caller can handle, never a panic, which would abort a WASM instance.
- **Calls (std/external/unresolved):** `OpcError::Io`, `std::io::Error::new`
- **Called by:** none found in src

#### `admit_input`  (L4907-4920)
`fn admit_input( side: &str, bytes: &[u8], limits: crate::admission::InputLimits, ) -> Result<(), OpcError>`

- **Does:** Admit one compare input under `limits`.
- **Calls (crate):** `admit` (admission.rs:310) L4912; `refused` (opc/mod.rs:76) L4915
- **Calls (std/external/unresolved):** `.map_err`, `.map`
- **Macros:** `format!`
- **Called by:** `admit_package` (document_comparer.rs:4924); `get_revisions` (document_comparer.rs:6285); `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::admission_preserves_typed_refusal_and_identifies_document_side` (document_comparer.rs:10008)

#### `admit_package`  (L4924-4926, pub(crate))
`pub(crate) fn admit_package(bytes: &[u8]) -> Result<(), OpcError>`

- **Does:** Admit a single package under the compare budget before a public entry point opens it: `PartFs::open` inflates every entry without a bound.
- **Calls (crate):** `admit_input` (document_comparer.rs:4907) L4925; `InputLimits::compare` (admission.rs:75) L4925
- **Called by:** `open` (changes.rs:391); `with_layout` (convert/mod.rs:769); `accept_revisions` (document_comparer.rs:4959); `reject_revisions` (document_comparer.rs:4976); `coverage_boundary_tests::admission_preserves_typed_refusal_and_identifies_document_side` (document_comparer.rs:10008)

#### `accept_revisions`  (L4959-4964, pub)
`pub fn accept_revisions(docx: &[u8]) -> Result<Vec<u8>, OpcError>`

- **Does:** A.11 — `RevisionProcessor.AcceptRevisions` byte facade: accept every tracked revision across main + headers/footers + notes + styles parts and return the result as a `.docx` byte vector.
- **Calls (crate):** `admit_package` (document_comparer.rs:4924) L4960; `PartFs::open` (opc/mod.rs:456) L4961; `accept_revisions_package` (revision_processor.rs:3465) L4962; `PartFs::to_zip` (opc/mod.rs:548) L4963
- **Called by:** `run_convert_any` (bin/jubarte.rs:2477); `public_option_source_ownership_tests::public_boundary_shifts_preserve_all_source_owners_with_lexical_options` (comparer/cross_para.rs:2705); `public_option_source_ownership_tests::public_comparison_options_preserve_reordered_paragraphs_formats_and_note_owners` (comparer/cross_para.rs:2775); `public_option_source_ownership_tests::public_reordered_bulk_paragraphs_preserve_sources_on_move_thrash_boundaries` (comparer/cross_para.rs:2938); `public_option_source_ownership_tests::strict_and_transitional_packages_preserve_complete_authored_body_ownership` (comparer/cross_para.rs:3034); `public_option_source_ownership_tests::zero_word_move_threshold_preserves_authored_nontext_and_separator_payloads` (comparer/cross_para.rs:3181); `public_option_source_ownership_tests::public_compound_and_one_sided_paragraph_windows_preserve_complete_sources` (comparer/cross_para.rs:3328); `row_nontext_ownership_regressions::public_compare_keeps_unchanged_tab_and_break_cells_when_text_moves_into_their_row` (comparer/finalize.rs:23873); `coverage_real_revision_merge_matrix_tests::public_compare_multi_original_blank_tail_recovers_complete_clean_sources` (comparer/finalize.rs:26091); `coverage_real_revision_merge_matrix_tests::whole_story_title_label_heading_and_list_boundaries_keep_source_ownership` (comparer/finalize.rs:27034); `coverage_real_revision_merge_matrix_tests::public_terminal_field_math_and_break_boundaries_preserve_all_source_owners` (comparer/finalize.rs:27484); `coverage_real_revision_merge_matrix_tests::public_cross_paragraph_field_ends_keep_exact_source_instruction_and_cache_owners` (comparer/finalize.rs:27706); +24 more

#### `accept_source_revisions`  (L4968-4972)
`fn accept_source_revisions(docx: &[u8]) -> Result<Vec<u8>, OpcError>`

- **Does:** [`accept_revisions`] keeping bookmark and comment ids: a redline names its comments by the ids the documents gave them, as `edit` reports them.
- **Calls (crate):** `PartFs::open` (opc/mod.rs:456) L4969; `accept_revisions_package_keeping_ids` (revision_processor.rs:3471) L4970; `PartFs::to_zip` (opc/mod.rs:548) L4971
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_package_source_matrix_tests::accepted_source_revision_converges_to_identical_final_before_comparison` (document_comparer.rs:11811)

#### `reject_revisions`  (L4976-4981, pub)
`pub fn reject_revisions(docx: &[u8]) -> Result<Vec<u8>, OpcError>`

- **Does:** A.11 — `RevisionProcessor.RejectRevisions` byte facade: reject every tracked revision across main + headers/footers + notes + styles parts.
- **Calls (crate):** `admit_package` (document_comparer.rs:4924) L4977; `PartFs::open` (opc/mod.rs:456) L4978; `reject_revisions_package` (revision_processor.rs:3480) L4979; `PartFs::to_zip` (opc/mod.rs:548) L4980
- **Called by:** `run_convert_any` (bin/jubarte.rs:2477); `public_option_source_ownership_tests::public_boundary_shifts_preserve_all_source_owners_with_lexical_options` (comparer/cross_para.rs:2705); `public_option_source_ownership_tests::public_comparison_options_preserve_reordered_paragraphs_formats_and_note_owners` (comparer/cross_para.rs:2775); `public_option_source_ownership_tests::public_reordered_bulk_paragraphs_preserve_sources_on_move_thrash_boundaries` (comparer/cross_para.rs:2938); `public_option_source_ownership_tests::strict_and_transitional_packages_preserve_complete_authored_body_ownership` (comparer/cross_para.rs:3034); `public_option_source_ownership_tests::zero_word_move_threshold_preserves_authored_nontext_and_separator_payloads` (comparer/cross_para.rs:3181); `public_option_source_ownership_tests::public_compound_and_one_sided_paragraph_windows_preserve_complete_sources` (comparer/cross_para.rs:3328); `row_nontext_ownership_regressions::public_compare_keeps_unchanged_tab_and_break_cells_when_text_moves_into_their_row` (comparer/finalize.rs:23873); `coverage_real_revision_merge_matrix_tests::public_compare_multi_original_blank_tail_recovers_complete_clean_sources` (comparer/finalize.rs:26091); `coverage_real_revision_merge_matrix_tests::whole_story_title_label_heading_and_list_boundaries_keep_source_ownership` (comparer/finalize.rs:27034); `coverage_real_revision_merge_matrix_tests::public_terminal_field_math_and_break_boundaries_preserve_all_source_owners` (comparer/finalize.rs:27484); `coverage_real_revision_merge_matrix_tests::public_cross_paragraph_field_ends_keep_exact_source_instruction_and_cache_owners` (comparer/finalize.rs:27706); +23 more

#### `unique_part_name`  (L4990-4992)
`fn unique_part_name(out: &PartFs, want: &str, bytes: &[u8]) -> String`

- **Does:** Collision-proof target name for copying `want` (with `bytes`) into `out`: free name or byte-identical existing part → `want` unchanged; an existing part with DIFFERENT content → the first free or identical candidate: a header or footer (`header1.xml`) takes the next number of its kind (`header3.xml`), a name the short...
- **Calls (crate):** `unique_part_name_where` (document_comparer.rs:4997) L4991
- **Called by:** `adopt_revised_header_footer` (document_comparer.rs:5166); `adopt_b_notes_when_a_lacks_separators` (document_comparer.rs:5863); `coverage_boundary_tests::a_colliding_header_or_footer_takes_the_first_free_number` (document_comparer.rs:9642); `coverage_boundary_tests::unique_part_names_handle_binary_collisions_reuse_and_flat_names` (document_comparer.rs:9668)

#### `unique_part_name_where`  (L4997-5040)
`fn unique_part_name_where( out: &PartFs, want: &str, bytes: &[u8], same: &dyn Fn(&str) -> bool, ) -> String`

- **Does:** [`unique_part_name`], reusing an identical part only when `same` also holds for it (a header's relationships, so identical XML that shows another image is another part).
- **Calls (crate):** `PartFs::part_bytes` (opc/mod.rs:493) L5003,5016,5032; `detach_breaks::{fn same}` (markdown/unified.rs:755) (by name) L5005,5018,5034
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.bytes`, `.find`, `.into_iter`, `.is_ascii_digit`, `.is_empty`, `.is_some_and`, `.rsplit_once`, `.strip_prefix`, `.strip_suffix`, `.to_string`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `unique_part_name` (document_comparer.rs:4990); `adopt_revised_header_footer` (document_comparer.rs:5166)

#### `strip_final_sectpr_inherited_header_footer`  (L5053-5115)
`fn strip_final_sectpr_inherited_header_footer( dom: &mut Dom, result_root: NodeId, revised_final: &std::collections::HashSet<(bool, String)>, )`

- **Does:** Word Compare leaves the body-level final `sectPr` without headerReference/footerReference when an earlier mid-body section break already defines the same (kind, type) slot — later sections inherit.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L5061,5065; `Dom::descendants` (xmllinq/mod.rs:668) L5070; `Dom::elements` (xmllinq/mod.rs:640) L5074,5094; `Dom::name` (xmllinq/mod.rs:526) L5075,5095; `Dom::attribute` (xmllinq/mod.rs:874) L5083,5103; `Dom::remove` (xmllinq/mod.rs:1102) L5113
- **Calls (std/external/unresolved):** `.contains`, `.insert`, `.is_empty`, `.push`, `.to_string`, `.unwrap_or`, `Some`, `Vec::new`, `W::body`, `W::name`, `std::collections::HashSet::new`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::final_header_slots_strip_only_inherited_non_revised_references` (document_comparer.rs:9555)

#### `final_header_footer_slots`  (L5119-5150)
`fn final_header_footer_slots(pkg: &PartFs) -> std::collections::HashSet<(bool, String)>`

- **Does:** The (is header, `w:type`) header/footer slots the body-level final `sectPr` of `pkg`'s main document sets explicitly.
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:829) L5120; `PartFs::part_string` (opc/mod.rs:498) L5123; `Dom::new` (xmllinq/mod.rs:387) L5126; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L5127; `Dom::root` (xmllinq/mod.rs:822) L5128; `Dom::element` (xmllinq/mod.rs:655) L5130,5131; `Dom::elements` (xmllinq/mod.rs:640) L5135; `Dom::name` (xmllinq/mod.rs:526) L5138; `Dom::attribute` (xmllinq/mod.rs:874) L5146
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.filter_map`, `.into_iter`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `Default::default`, `Some`, `W::body`, `W::name`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::final_header_slots_strip_only_inherited_non_revised_references` (document_comparer.rs:9555); `coverage_boundary_tests::empty_missing_and_non_xml_package_parts_are_safe_noops` (document_comparer.rs:9962)

#### `adopt_revised_header_footer`  (L5166-5438)
`fn adopt_revised_header_footer( dom: &mut Dom, result_root: NodeId, pkg2: &PartFs, out: &mut PartFs, out_main: &str, settings: &WmlComparerSettings, )`

- **Does:** Word-alignment mode (settings-gated): Word's Compare presents the REVISED document's headers/footers as a UNION per (kind, `w:type`) slot.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L5176,5179,5250,5253; `Dom::descendants` (xmllinq/mod.rs:668) L5184,5255; `Dom::elements` (xmllinq/mod.rs:640) L5186,5205,5269; `Dom::name` (xmllinq/mod.rs:526) L5188,5209,5272,5279; `Dom::attribute` (xmllinq/mod.rs:874) L5196,5217,5280,5289; `PartFs::main_document_part` (opc/mod.rs:829) L5239; `PartFs::part_string` (opc/mod.rs:498) L5242; `Dom::new` (xmllinq/mod.rs:387) L5245; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L5246; `Dom::root` (xmllinq/mod.rs:822) L5247; `PartFs::read_rels_for` (opc/mod.rs:756) L5259,5351; `PartFs::resolve_rel_target` (opc/mod.rs:600) L5295,5364; `PartFs::part_bytes` (opc/mod.rs:493) L5296,5331,5365,5367; `PartFs::parts` (opc/mod.rs:532) L5305; `PartFs::read_rels_for` (opc/mod.rs:756) (by name) L5317; `rels_of` (debug.rs:723) (by name) L5324,5326; `unique_part_name_where` (document_comparer.rs:4997) L5325; `PartFs::set_part` (opc/mod.rs:510) L5333,5368,5416; `mark_adopted_hf_content_as_inserted` (document_comparer.rs:5444) L5337; `PartFs::add_content_type_override` (opc/mod.rs:616) L5344; `unique_part_name` (document_comparer.rs:4990) L5366; `relative_rel_target` (opc/mod.rs:129) L5373,5427; `PartFs::add_content_type_default` (opc/mod.rs:623) L5391; `PartFs::add_document_relationship` (opc/mod.rs:772) L5428; `Dom::new_element` (xmllinq/mod.rs:478) L5429; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L5434,5435; `Dom::add_first` (xmllinq/mod.rs:1085) L5436
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.as_str`, `.clone`, `.collect`, `.contains_key`, `.contains`, `.copied`, `.ends_with`, `.filter_map`, `.filter`, `.flat_map`, `.get`, `.insert`, `.into_bytes`, `.into_iter`, `.is_none`, `.is_some_and`, `.iter`, `.last`, `.map`, `.next`, `.or_else`, `.push_str`, `.replace`, `.rsplit_once`, `.rsplit`, `.to_ascii_lowercase`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `.values`, `R::name`, `Some`, `String::from`, `W::body`, `W::name`, `body_slots`, `std::collections::HashMap::new`, `xe`
- **Macros:** `format!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::final_header_footer_adoption_respects_existing_body_owners_and_cached_donor_parts` (document_comparer.rs:10052); `coverage_boundary_tests::header_adoption_preserves_colliding_media_and_uses_correct_content_types` (document_comparer.rs:10403); `coverage_boundary_tests::a_renamed_header_copy_never_hides_another_header_of_the_revision` (document_comparer.rs:10502); `coverage_boundary_tests::a_reused_header_keeps_its_own_relationships` (document_comparer.rs:10555); `coverage_boundary_tests::header_adoption_refuses_unresolvable_references_and_missing_story_structure` (document_comparer.rs:10593); `adopted_story_relationship_owner_tests::adopted_root_and_nested_stories_keep_colliding_media_relationships_resolvable` (document_comparer.rs:12527)

#### `mark_adopted_hf_content_as_inserted`  (L5444-5556)
`fn mark_adopted_hf_content_as_inserted( out: &mut PartFs, part: &str, settings: &WmlComparerSettings, )`

- **Does:** M378 — wrap body content of a newly adopted B-only header/footer as pure-I.
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:498) L5449; `Dom::new` (xmllinq/mod.rs:387) L5452; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L5453; `Dom::root` (xmllinq/mod.rs:822) L5454; `next_free_revision_id` (document_comparer.rs:469) L5458; `Dom::descendants` (xmllinq/mod.rs:668) L5461,5464,5465; `Dom::elements` (xmllinq/mod.rs:640) L5469,5539; `Dom::name` (xmllinq/mod.rs:526) L5472,5479; `XName::local_name` (xmllinq/mod.rs:208) (by name) L5487; `Dom::new_element` (xmllinq/mod.rs:478) L5493,5521,5532; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L5494,5495,5498,5533,5534,5537; `Dom::parent` (xmllinq/mod.rs:597) L5501,5510; `Dom::add_before_self` (xmllinq/mod.rs:1114) L5502,5524,5540; `Dom::add` (xmllinq/mod.rs:1058) L5504,5507,5514,5526,5542; `Dom::remove` (xmllinq/mod.rs:1102) L5513; `Dom::element` (xmllinq/mod.rs:655) L5517,5518,5523,5531; `mark_story_rows` (document_comparer.rs:5747) L5547; `Dom::new_element` (xmllinq/mod.rs:478) (by name) L5548; `Dom::set_attribute_value` (xmllinq/mod.rs:961) (by name) L5549,5550,5551; `PartFs::set_part` (opc/mod.rs:510) L5555; `Dom::serialize_element` (xmllinq/mod.rs:1299) L5555
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.collect`, `.copied`, `.filter`, `.first`, `.into_bytes`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.to_string`, `Some`, `W::author`, `W::date`, `W::del`, `W::hyperlink`, `W::id`, `W::ins`, `W::name`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`
- **Called by:** `adopt_revised_header_footer` (document_comparer.rs:5166); `coverage_boundary_tests::adopted_header_marks_content_and_paragraph_properties_with_pinned_provenance` (document_comparer.rs:9281); `coverage_boundary_tests::one_sided_header_footer_tables_mark_every_row` (document_comparer.rs:9358); `coverage_boundary_tests::one_sided_story_marks_take_ids_above_the_story_s_own` (document_comparer.rs:9396); `coverage_boundary_tests::one_sided_story_fields_join_the_story_revision` (document_comparer.rs:9424); `coverage_boundary_tests::empty_missing_and_non_xml_package_parts_are_safe_noops` (document_comparer.rs:9962); `coverage_boundary_tests::header_insertion_wraps_supported_owned_payload_and_retains_noncontent_and_foreign_revisions` (document_comparer.rs:11078); `isolated_header_container_revision_boundary_tests::pure_header_container_payloads_recover_every_authored_property_and_fallback` (document_comparer.rs:12199)

#### `mark_a_only_hf_content_as_deleted`  (L5563-5579)
`fn mark_a_only_hf_content_as_deleted( out: &mut PartFs, pkg1: &PartFs, pkg2: &PartFs, settings: &WmlComparerSettings, )`

- **Does:** M383 — when the revised document has **no** headers/footers but the original does, Word marks all original HF content pure-D.
- **Calls (crate):** `header_footer_refs` (document_comparer.rs:116) L5569,5571; `mark_hf_part_content_as_deleted` (document_comparer.rs:5658) L5577
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.insert`, `.iter`, `std::collections::HashSet::new`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::original_footer_deletion_is_per_kind_and_duplicate_references_are_processed_once` (document_comparer.rs:10628)

#### `delete_story_closing_mark`  (L5585-5647)
`fn delete_story_closing_mark(dom: &mut Dom, story: NodeId, settings: &WmlComparerSettings)`

- **Does:** Word's redline of a header/footer whose revised story ends with a table: the original's closing paragraph, emptied by the diff, loses its mark, since the revised story needs none, and keeps its own properties unrecorded (bc0135eaa1).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L5586,5620,5625; `Dom::name_is` (xmllinq/mod.rs:535) L5595; `Dom::descendants` (xmllinq/mod.rs:668) L5598; `Dom::element` (xmllinq/mod.rs:655) L5602,5610,5612,5617,5618; `Dom::new_element` (xmllinq/mod.rs:478) L5605,5636,5641; `Dom::add_first` (xmllinq/mod.rs:1085) L5606,5646; `Dom::remove` (xmllinq/mod.rs:1102) L5619,5622; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L5626; `Dom::add_before_self` (xmllinq/mod.rs:1114) L5628; `Dom::add` (xmllinq/mod.rs:1058) L5629,5637; `next_free_revision_id` (document_comparer.rs:469) L5642; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L5643,5644,5645
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.last`, `.map`, `.to_string`, `.unwrap_or_default`, `Some`, `W::author`, `W::date`, `W::del`, `W::id`, `W::ins`, `W::name`, `W::p_pr`, `W::p`, `W::r_pr`, `W::t`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::story_closing_mark_restores_original_props_and_does_not_double_mark` (document_comparer.rs:9472); `coverage_boundary_tests::closing_mark_refuses_live_content_and_handles_empty_or_unformatted_stories` (document_comparer.rs:9497)

#### `mark_hf_part_content_as_deleted`  (L5658-5740)
`fn mark_hf_part_content_as_deleted(out: &mut PartFs, part: &str, settings: &WmlComparerSettings)`

- **Does:** Word's redline of a header/footer the revised document drops: the content goes pure-D and every paragraph mark but the story's last is deleted (a paragraph without `pPr` included).
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:498) L5659; `Dom::new` (xmllinq/mod.rs:387) L5662; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L5663; `Dom::root` (xmllinq/mod.rs:822) L5664; `next_free_revision_id` (document_comparer.rs:469) L5667; `Dom::new_element` (xmllinq/mod.rs:478) L5671; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L5672,5673,5674; `Dom::name` (xmllinq/mod.rs:526) L5678,5694,5697,5720; `Dom::elements` (xmllinq/mod.rs:640) L5683,5691,5721,5731; `Dom::descendants` (xmllinq/mod.rs:668) L5684,5686,5687,5713; `XName::local_name` (xmllinq/mod.rs:208) (by name) L5705; `revision` (ambiguous, 5 candidates) L5708,5730,5738; `Dom::add_before_self` (xmllinq/mod.rs:1114) L5709,5732; `Dom::remove` (xmllinq/mod.rs:1102) L5711; `Dom::set_name` (xmllinq/mod.rs:539) L5714; `Dom::add` (xmllinq/mod.rs:1058) L5716,5733; `Dom::parent` (xmllinq/mod.rs:597) L5719; `reset_to_blank_paragraph` (document_comparer.rs:5761) L5724; `child_or_first` (document_comparer.rs:5829) L5727; `child_before` (document_comparer.rs:5843) L5728; `Dom::element` (xmllinq/mod.rs:655) L5729; `mark_story_rows` (document_comparer.rs:5747) L5738; `PartFs::set_part` (opc/mod.rs:510) L5739; `Dom::serialize_element` (xmllinq/mod.rs:1299) L5739
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.collect`, `.copied`, `.filter`, `.first`, `.into_bytes`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.iter`, `.last`, `.to_string`, `Some`, `W::author`, `W::date`, `W::del`, `W::hyperlink`, `W::id`, `W::ins`, `W::name`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `W::t`
- **Called by:** `mark_a_only_hf_content_as_deleted` (document_comparer.rs:5563); `coverage_boundary_tests::deleted_footer_preserves_last_mark_and_records_original_properties` (document_comparer.rs:9307); `coverage_boundary_tests::one_sided_header_footer_tables_mark_every_row` (document_comparer.rs:9358); `coverage_boundary_tests::one_sided_story_marks_take_ids_above_the_story_s_own` (document_comparer.rs:9396); `coverage_boundary_tests::one_sided_story_fields_join_the_story_revision` (document_comparer.rs:9424); `coverage_boundary_tests::deletion_skips_inbound_changes_and_preserves_existing_blank_history` (document_comparer.rs:9453); `coverage_boundary_tests::empty_missing_and_non_xml_package_parts_are_safe_noops` (document_comparer.rs:9962); `isolated_header_container_revision_boundary_tests::pure_header_container_payloads_recover_every_authored_property_and_fallback` (document_comparer.rs:12199)

#### `mark_story_rows`  (L5747-5756)
`fn mark_story_rows(dom: &mut Dom, story: NodeId, mut mark: impl FnMut(&mut Dom) -> NodeId)`

- **Does:** Mark every table row of a one-sided header/footer story with the story's revision (`w:del` dropped, `w:ins` adopted), as Word's redline does.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L5748; `row_properties` (comparer/lcs_table.rs:763) L5749; `Dom::element` (xmllinq/mod.rs:655) L5750; `existing_mark_layout_ownership_tests::mark` (comparer/finalize.rs:32802) L5753; `add_row_mark` (comparer/lcs_table.rs:778) L5754
- **Calls (std/external/unresolved):** `.is_some`, `Some`, `W::del`, `W::ins`, `W::name`
- **Called by:** `mark_adopted_hf_content_as_inserted` (document_comparer.rs:5444); `mark_hf_part_content_as_deleted` (document_comparer.rs:5658)

#### `reset_to_blank_paragraph`  (L5761-5826)
`fn reset_to_blank_paragraph( dom: &mut Dom, p: NodeId, own_style: &str, revision: &mut impl FnMut(&mut Dom, XName) -> NodeId, )`

- **Does:** Reset the story's last paragraph to the blank one Word's redline keeps: no properties but the kind's own style, the old paragraph properties in a `pPrChange` and the old mark formatting in an `rPrChange`.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L5767,5782,5802,5815; `Dom::name` (xmllinq/mod.rs:526) L5771,5807; `XName::local_name` (xmllinq/mod.rs:208) (by name) L5774,5789,5809; `Dom::attribute` (xmllinq/mod.rs:874) L5775; `Dom::elements` (xmllinq/mod.rs:640) L5777,5787,5803; `revision` (ambiguous, 5 candidates) L5783,5816; `Dom::new_element` (xmllinq/mod.rs:478) L5784,5817; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L5792; `Dom::add` (xmllinq/mod.rs:1058) L5793,5799,5800,5820,5822,5823; `Dom::remove` (xmllinq/mod.rs:1102) L5797,5819
- **Calls (std/external/unresolved):** `.as_deref`, `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.map`, `.name`, `.to_string`, `Some`, `W::name`, `W::p_pr`, `W::r_pr`, `W::val`, `keep`
- **Macros:** `matches!`
- **Called by:** `mark_hf_part_content_as_deleted` (document_comparer.rs:5658)

#### `child_or_first`  (L5829-5839)
`fn child_or_first(dom: &mut Dom, parent: NodeId, name: XName) -> NodeId`

- **Does:** `parent`'s `name` child, created as its first child when absent.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L5830; `Dom::new_element` (xmllinq/mod.rs:478) L5833; `Dom::elements` (xmllinq/mod.rs:640) L5834; `Dom::add_before_self` (xmllinq/mod.rs:1114) L5835; `Dom::add` (xmllinq/mod.rs:1058) L5836
- **Calls (std/external/unresolved):** `.copied`, `.first`
- **Called by:** `mark_hf_part_content_as_deleted` (document_comparer.rs:5658); `coverage_boundary_tests::child_creation_places_properties_before_schema_successors_and_reuses_nodes` (document_comparer.rs:9537)

#### `child_before`  (L5843-5857)
`fn child_before(dom: &mut Dom, parent: NodeId, name: XName, later: &[&str]) -> NodeId`

- **Does:** `parent`'s `name` child, created before the first of `later` (schema order) or last when none is present.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L5844; `Dom::new_element` (xmllinq/mod.rs:478) L5847; `Dom::elements` (xmllinq/mod.rs:640) L5848; `Dom::name` (xmllinq/mod.rs:526) L5849; `XName::local_name` (xmllinq/mod.rs:208) (by name) L5850; `Dom::add_before_self` (xmllinq/mod.rs:1114) L5853; `Dom::add` (xmllinq/mod.rs:1058) L5854
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.into_iter`, `.is_some_and`
- **Called by:** `mark_hf_part_content_as_deleted` (document_comparer.rs:5658); `coverage_boundary_tests::child_creation_places_properties_before_schema_successors_and_reuses_nodes` (document_comparer.rs:9537)

#### `adopt_b_notes_when_a_lacks_separators`  (L5863-5993)
`fn adopt_b_notes_when_a_lacks_separators( out: &mut PartFs, pkg1: &PartFs, pkg2: &PartFs, out_main: &str, )`

- **Does:** M379 — when the original has no real footnotes/endnotes separators but the revised document does, copy B's notes part into the package.
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:498) L5881,5887,5894,5905; `Dom::new` (xmllinq/mod.rs:387) L5906; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L5907,5908; `Dom::root` (xmllinq/mod.rs:822) L5909; `Dom::elements` (xmllinq/mod.rs:640) L5915,5920; `is_structural_note` (comparer/footnotes.rs:787) L5918; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L5922; `Dom::add_before_self` (xmllinq/mod.rs:1114) L5924; `Dom::add` (xmllinq/mod.rs:1058) L5925; `PartFs::set_part` (opc/mod.rs:510) L5928,5932,5968,5990; `Dom::serialize_element` (xmllinq/mod.rs:1299) L5928; `PartFs::add_content_type_override` (opc/mod.rs:616) L5933; `PartFs::read_rels_for` (opc/mod.rs:756) L5935,5951; `relative_rel_target` (opc/mod.rs:129) L5941,5971; `PartFs::add_document_relationship` (opc/mod.rs:772) L5942; `PartFs::resolve_rel_target` (opc/mod.rs:600) L5964; `PartFs::part_bytes` (opc/mod.rs:493) L5965,5967; `unique_part_name` (document_comparer.rs:4990) L5966
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.clone`, `.collect`, `.contains`, `.copied`, `.ends_with`, `.filter`, `.first`, `.into_bytes`, `.into_iter`, `.is_none`, `.is_some_and`, `.iter`, `.map`, `.next`, `.push_str`, `.replace`, `.rsplit`, `.unwrap_or_default`, `.unwrap_or`, `Some`, `String::from`, `crate::namespaces::W::endnote`, `crate::namespaces::W::footnote`, `xe`
- **Macros:** `format!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::notes_merge_only_structural_definitions_ahead_of_rectified_content` (document_comparer.rs:9790); `coverage_boundary_tests::notes_copy_preserves_media_and_escapes_external_links_without_io` (document_comparer.rs:9831); `coverage_boundary_tests::notes_adoption_refuses_missing_separator_and_preserves_original_separator` (document_comparer.rs:9871); `coverage_boundary_tests::separator_media_collisions_keep_a_resolvable_relative_relationship` (document_comparer.rs:10832)

#### `reresolve_theme_color_hexes`  (L6004-6160)
`fn reresolve_theme_color_hexes(dom: &mut Dom, styles_root: NodeId, theme_xml: &str) -> bool`

- **Does:** M483 — re-cache `w:color` hex values against the package's theme.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L6119; `Dom::attribute` (xmllinq/mod.rs:874) L6121,6133,6136,6138; `Dom::name` (xmllinq/mod.rs:526) L6128; `apply` (ambiguous: `tests::apply` (edit/rewrite.rs:164); `apply` (markdown/from_docx/header.rs:744); `apply` (markdown/view_styles.rs:497)) L6137,6139; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L6154
- **Calls (std/external/unresolved):** `.abs_diff`, `.abs`, `.all`, `.as_ref`, `.clamp`, `.clone`, `.collect`, `.find`, `.floor`, `.iter`, `.len`, `.map_or`, `.map`, `.max`, `.min`, `.ok`, `.to_uppercase`, `.unwrap_or`, `Some`, `W::name`, `W::val`, `hue`, `slot_hex`, `slot_of`, `u32::from_str_radix`
- **Macros:** `format!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::theme_cache_recaching_preserves_independent_fill_color_and_tolerated_source_hexes` (document_comparer.rs:10963)

#### `custom_props_part`  (L6173-6180)
`fn custom_props_part(pkg: &PartFs) -> Option<String>`

- **Does:** The part the package's custom-properties relationship reaches.
- **Calls (crate):** `PartFs::package_relationships` (opc/mod.rs:782) L6174; `PartFs::part_bytes` (opc/mod.rs:493) L6179
- **Calls (std/external/unresolved):** `.filter`, `.find`, `.is_some`, `.iter`, `.map`, `.to_string`, `.trim_start_matches`
- **Called by:** `merge_custom_properties` (document_comparer.rs:6188); `coverage_boundary_tests::custom_property_adoption_refuses_unrelated_collision_and_malformed_xml` (document_comparer.rs:9927)

#### `merge_custom_properties`  (L6188-6248)
`fn merge_custom_properties(out: &mut PartFs, pkg2: &PartFs)`

- **Does:** Word's redline keeps the custom document properties of both sides: the revision's alone when the original has none (83 of the 96 Word redlines where only the revision has them), else the union, the original's value winning where both name a property (every conflict among the 47 with both).
- **Calls (crate):** `custom_props_part` (document_comparer.rs:6173) L6189,6195; `PartFs::part_string` (opc/mod.rs:498) L6192,6207; `PartFs::part_bytes` (opc/mod.rs:493) L6196; `PartFs::set_part` (opc/mod.rs:510) L6199,6246; `PartFs::add_content_type_override` (opc/mod.rs:616) L6200; `PartFs::add_package_relationship` (opc/mod.rs:787) L6204; `Dom::new` (xmllinq/mod.rs:387) L6210; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L6211,6212; `Dom::root` (xmllinq/mod.rs:822) L6213; `XName::get` (xmllinq/mod.rs:180) L6216,6217; `Dom::elements` (xmllinq/mod.rs:640) L6218,6232; `Dom::attribute` (xmllinq/mod.rs:874) L6221,6229,6233; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L6239; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L6241; `Dom::add` (xmllinq/mod.rs:1058) L6242; `Dom::serialize_document` (xmllinq/mod.rs:1322) L6246
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.insert`, `.into_bytes`, `.is_some`, `.iter`, `.map`, `.max`, `.ok`, `.parse`, `.to_lowercase`, `.to_string`, `.unwrap_or`, `Some`
- **Macros:** `format!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::custom_property_merge_is_case_insensitive_original_wins_and_new_ids_are_unique` (document_comparer.rs:9898); `coverage_boundary_tests::custom_property_adoption_refuses_unrelated_collision_and_malformed_xml` (document_comparer.rs:9927)

#### `repair_missing_core_relationships`  (L6250-6278)
`fn repair_missing_core_relationships(out: &mut PartFs, out_main: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::part_bytes` (opc/mod.rs:493) L6259; `PartFs::read_rels_for` (opc/mod.rs:756) L6262; `relative_rel_target` (opc/mod.rs:129) L6268; `PartFs::add_document_relationship` (opc/mod.rs:772) L6269
- **Calls (std/external/unresolved):** `.any`, `.ends_with`, `.is_none`, `.is_some_and`, `.iter`
- **Macros:** `format!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::revised_chrome_preserves_existing_parts_and_repairs_relationships_idempotently` (document_comparer.rs:10664)

#### `get_revisions`  (L6285-6340, pub)
`pub fn get_revisions( docx: &[u8], settings: &crate::comparer::WmlComparerSettings, ) -> Result<Vec<crate::comparer::WmlComparerRevision>, OpcError>`

- **Does:** D.6 — `WmlComparer.GetRevisions` (:3940) byte facade: list every tracked revision in a redline `.docx` — main-part groups, footnote/endnote definition groups, `w:rPrChange` format changes, then (settings-gated) move detection.
- **Calls (crate):** `admit_input` (document_comparer.rs:4907) L6291; `PartFs::open` (opc/mod.rs:456) L6292; `PartFs::main_document_part` (opc/mod.rs:829) L6293; `PartFs::part_string` (opc/mod.rs:498) L6296,6315; `Dom::new` (xmllinq/mod.rs:387) L6299; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L6300,6316; `Dom::root` (xmllinq/mod.rs:822) L6301,6317; `test_for_invalid_content` (comparer/preprocess.rs:602) L6307; `remove_existing_powertools_markup` (comparer/preprocess.rs:572) L6308,6318; `notes_part_names` (document_comparer.rs:6444) L6310; `Dom::element` (xmllinq/mod.rs:655) L6324; `get_revisions_from_body` (comparer/revisions.rs:151) L6327; `get_revisions_from_note_definitions` (comparer/revisions.rs:224) L6329; `get_format_change_revisions` (comparer/revisions.rs:362) L6337; `detect_moves` (comparer/revisions.rs:410) L6338
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.extend`, `.map_err`, `.ok_or_else`, `.push`, `.to_string`, `.unwrap_or_else`, `Ok`, `OpcError::PartNotFound`, `Vec::new`, `W::body`, `W::endnote`, `W::footnote`
- **Macros:** `format!`, `vec!`
- **Called by:** none found in src

#### `revision_to_json`  (L6346-6383, pub)
`pub fn revision_to_json(r: &crate::comparer::WmlComparerRevision) -> String`

- **Does:** Serialize one revision to the stable JSON object shape of the wasm `getRevisions` and Python `get_revisions` bindings.
- **Calls (crate):** `revision_to_json::{fn esc}` (document_comparer.rs:6347) L6366,6373,6374,6375,6381
- **Calls (std/external/unresolved):** `.as_deref`, `.as_ref`, `.collect`, `.iter`, `.join`, `.map_or`, `.map`, `.to_string`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `tests::revision_json_matches_cli_shape_and_escapes_control_chars` (document_comparer.rs:8208)

#### `revision_to_json::{fn esc}`  (L6347-6361)
`fn esc(s: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chars`, `.len`, `.push_str`, `.push`, `String::with_capacity`
- **Macros:** `format!`
- **Called by:** `revision_to_json` (document_comparer.rs:6346)

#### `revisions_to_json`  (L6388-6391, pub)
`pub fn revisions_to_json(revs: &[crate::comparer::WmlComparerRevision]) -> String`

- **Does:** Serialize a revision list to a single JSON array string — the wasm `getRevisions` binding shape.
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.join`, `.map`
- **Macros:** `format!`
- **Called by:** `tests::revision_json_matches_cli_shape_and_escapes_control_chars` (document_comparer.rs:8208)

#### `compare_documents`  (L6394-6400, pub)
`pub fn compare_documents( original: &[u8], modified: &[u8], author: &str, ) -> Result<Vec<u8>, OpcError>`

- **Does:** `DocumentComparer.CompareDocuments(original, modified, author)`.
- **Calls (crate):** `compare_documents_with_options` (document_comparer.rs:6403) L6399
- **Called by:** none found in src

#### `compare_documents_with_options`  (L6403-6410, pub)
`pub fn compare_documents_with_options( original: &[u8], modified: &[u8], author: &str, date: &str, ) -> Result<Vec<u8>, OpcError>`

- **Does:** Compare with an explicit revision timestamp (for reproducible output).
- **Calls (crate):** `compare_documents_internal` (document_comparer.rs:6425) L6409
- **Called by:** `compare_documents` (document_comparer.rs:6394)

#### `compare_documents_with_settings`  (L6414-6420, pub)
`pub fn compare_documents_with_settings( original: &[u8], modified: &[u8], settings: &WmlComparerSettings, ) -> Result<Vec<u8>, OpcError>`

- **Does:** Compare with caller-supplied [`WmlComparerSettings`] (author/date/detail threshold/…).
- **Calls (crate):** `compare_documents_impl` (document_comparer.rs:6501) L6419
- **Called by:** `run` (bin/jubarte.rs:2095); `public_option_source_ownership_tests::public_boundary_shifts_preserve_all_source_owners_with_lexical_options` (comparer/cross_para.rs:2705); `public_option_source_ownership_tests::public_comparison_options_preserve_reordered_paragraphs_formats_and_note_owners` (comparer/cross_para.rs:2775); `public_option_source_ownership_tests::public_reordered_bulk_paragraphs_preserve_sources_on_move_thrash_boundaries` (comparer/cross_para.rs:2938); `public_option_source_ownership_tests::strict_and_transitional_packages_preserve_complete_authored_body_ownership` (comparer/cross_para.rs:3034); `public_option_source_ownership_tests::zero_word_move_threshold_preserves_authored_nontext_and_separator_payloads` (comparer/cross_para.rs:3181); `public_option_source_ownership_tests::public_compound_and_one_sided_paragraph_windows_preserve_complete_sources` (comparer/cross_para.rs:3328); `row_nontext_ownership_regressions::public_compare_keeps_unchanged_tab_and_break_cells_when_text_moves_into_their_row` (comparer/finalize.rs:23873); `coverage_real_revision_merge_matrix_tests::public_compare_multi_original_blank_tail_recovers_complete_clean_sources` (comparer/finalize.rs:26091); `coverage_real_revision_merge_matrix_tests::whole_story_title_label_heading_and_list_boundaries_keep_source_ownership` (comparer/finalize.rs:27034); `coverage_real_revision_merge_matrix_tests::public_terminal_field_math_and_break_boundaries_preserve_all_source_owners` (comparer/finalize.rs:27484); `coverage_real_revision_merge_matrix_tests::public_cross_paragraph_field_ends_keep_exact_source_instruction_and_cache_owners` (comparer/finalize.rs:27706); +17 more

#### `compare_documents_internal`  (L6425-6438, pub)
`pub fn compare_documents_internal( original: &[u8], modified: &[u8], author: &str, date: &str, pre_process_original: bool, ) -> Result<Vec<u8>, OpcError>`

- **Does:** `WmlComparer.CompareInternal` (:152).
- **Calls (crate):** `<WmlComparerSettings as Default>::default` (comparer/mod.rs:1475) L6435; `compare_documents_impl` (document_comparer.rs:6501) L6437
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `compare_documents_with_options` (document_comparer.rs:6403)

#### `notes_part_names`  (L6444-6463)
`fn notes_part_names(pkg: &PartFs) -> (String, String)`

- **Does:** Resolve the footnotes/endnotes part names via the main-document rels, falling back to the standard names.
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:829) L6445; `PartFs::read_rels_for` (opc/mod.rs:756) L6450; `PartFs::resolve_rel_target` (opc/mod.rs:600) L6456,6457
- **Calls (std/external/unresolved):** `.as_deref`, `.next`, `.rsplit`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `Some`
- **Called by:** `get_revisions` (document_comparer.rs:6285); `compare_documents_impl` (document_comparer.rs:6501)

#### `main_part_xml`  (L6467-6474)
`fn main_part_xml(docx: &[u8]) -> Option<String>`

- **Does:** True when any package part's XML carries tracked-change markup that Word Compare would fold into the final view before diffing.
- **Calls (crate):** `PartFs::open` (opc/mod.rs:456) L6468; `PartFs::main_document_part` (opc/mod.rs:829) (by name) L6469; `PartFs::part_string` (opc/mod.rs:498) (by name) L6472; `normalize_strict_namespaces` (document_comparer.rs:4607) L6473
- **Calls (std/external/unresolved):** `.into_owned`, `.ok`, `.to_string`, `.unwrap_or_else`, `Some`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::admission_preserves_typed_refusal_and_identifies_document_side` (document_comparer.rs:10008)

#### `docx_has_tracked_changes`  (L6476-6499)
`fn docx_has_tracked_changes(docx: &[u8]) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::open` (opc/mod.rs:456) L6477; `PartFs::parts` (opc/mod.rs:532) (by name) L6480; `PartFs::part_string` (opc/mod.rs:498) (by name) L6484
- **Calls (std/external/unresolved):** `.contains`, `.ends_with`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501); `coverage_boundary_tests::admission_preserves_typed_refusal_and_identifies_document_side` (document_comparer.rs:10008); `coverage_boundary_tests::tracked_change_admission_scans_each_xml_story_and_refuses_binary_or_non_xml_carriers` (document_comparer.rs:11260); `coverage_package_source_matrix_tests::public_comparison_preserves_complete_sources_across_package_features` (document_comparer.rs:11595); `coverage_package_source_matrix_tests::accepted_source_revision_converges_to_identical_final_before_comparison` (document_comparer.rs:11811)

#### `compare_documents_impl`  (L6501-8015)
`fn compare_documents_impl( original: &[u8], modified: &[u8], settings: &WmlComparerSettings, pre_process_original: bool, ) -> Result<Vec<u8>, OpcError>`

- **Does:** _no doc comment_
- **Calls (crate):** `admit_input` (document_comparer.rs:4907) L6510,6533; `strict_to_transitional_docx_within` (strict_translation.rs:756) L6515,6534,6538; `docx_has_tracked_changes` (document_comparer.rs:6476) L6519,6564; `accept_source_revisions` (document_comparer.rs:4968) L6520,6566,6567; `fix_up_drawing_ids_in_package` (comparer/fixups.rs:121) L6525,6572; `main_part_xml` (document_comparer.rs:6467) L6548,6549; `PartFs::open` (opc/mod.rs:456) L6578,6579,6723,6725; `pre_process_markup` (document_comparer.rs:4713) L6586,6593; `PartFs::main_document_part` (opc/mod.rs:829) L6598,6601; `PartFs::part_string` (opc/mod.rs:498) L6605,6608,6756,6757,6829,7100...; `normalize_strict_namespaces` (document_comparer.rs:4607) L6613,6614,7857; `Dom::new` (xmllinq/mod.rs:387) L6617,6831,6904,7028,7052,7082...; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L6618,6619,6832,6833,6905,7029...; `Dom::root` (xmllinq/mod.rs:822) L6620,6623,6834,6906,7030,7054...; `merged_body` (document_comparer.rs:4630) L6626,6628; `normalize_universal_measures` (comparer/finalize.rs:11485) L6638,6675,7893; `notes_part_names` (document_comparer.rs:6444) L6652,6653; `compare_documents_impl::{fn parse_part_root}` (document_comparer.rs:6647) L6655,6656,6657,6658,6659,6660; `PartFs::part_string` (opc/mod.rs:498) (by name) L6682,6686,6829,6902,7079,7099...; `default_paragraph_line` (document_comparer.rs:3170) L6684,7918; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L6685,6690,6709,6710,6860,6888...; `has_default_table_style` (document_comparer.rs:3156) L6688; `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209) L6698; `PartFs::to_zip` (opc/mod.rs:548) L6725; `reconcile_dangling_relationships` (comparer/parts.rs:273) L6729; `adopt_revised_header_footer` (document_comparer.rs:5166) L6739; `strip_final_sectpr_inherited_header_footer` (document_comparer.rs:5053) L6742; `final_header_footer_slots` (document_comparer.rs:5119) L6745; `mark_a_only_hf_content_as_deleted` (document_comparer.rs:5563) L6748; `carry_comments` (comparer/comments.rs:1583) L6758; `fix_up_revision_ids` (comparer/finalize.rs:725) L6770; `reorder_replacements_ins_before_del` (comparer/finalize.rs:6652) L6778; `convert_outer_math_wraps_to_internal` (comparer/produce.rs:778) L6783; `carry_bookmarks` (comparer/bookmarks.rs:395) L6788; `fix_up_doc_pr_ids` (comparer/fixups.rs:32) L6799; `fix_up_shape_ids` (comparer/fixups.rs:54) L6800; `fix_up_shape_type_ids` (comparer/fixups.rs:75) L6801; `Dom::serialize_element` (xmllinq/mod.rs:1299) L6803,6985,6994,7033,7056,7092...; `PartFs::set_part` (opc/mod.rs:510) (by name) L6804,6985,6994,7005,7033,7056...; `defined_style_ids` (comparer/footnotes.rs:217) L6837,6992,7229; `materialize_builtin_paragraph_defaults` (document_comparer.rs:3764) L6838,6839; `copy_missing_styles_by_name` (document_comparer.rs:3623) L6840; `adopt_missing_styles_structure` (document_comparer.rs:3786) L6841; `canonicalize_style_ids` (document_comparer.rs:3667) L6842,7031,7055; `Dom::descendants` (xmllinq/mod.rs:668) L6855,6918,6922,7192,7195,7199...; `Dom::attribute` (xmllinq/mod.rs:874) L6857,6919,6943,7164,7401,7416...; `copy_missing_styles` (comparer/footnotes.rs:267) L6866; `copy_missing_numbering` (comparer/footnotes.rs:364) L6870; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L6875; `Dom::attributes` (xmllinq/mod.rs:885) L6876,7797; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L6877,7798; `carry_relationship` (comparer/parts.rs:517) L6880; `next_free_revision_id` (document_comparer.rs:469) L6921; `Dom::element` (xmllinq/mod.rs:655) L6923,6926,6928,6937,6939,6949...; `mark_has` (edit/tracked.rs:968) (by name) L6930,6933; `Dom::new_element` (xmllinq/mod.rs:478) L6951,6961,7547,7793; `Dom::elements` (xmllinq/mod.rs:640) L6952,7157,7310,7398,7413,7430...; `Dom::name` (xmllinq/mod.rs:526) L6953,7807; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L6958; `Dom::add` (xmllinq/mod.rs:1058) L6959,6978,6979,7811; `PartFs::add_content_type_override` (opc/mod.rs:616) (by name) L7006,7034,7628; `PartFs::read_rels_for` (opc/mod.rs:756) (by name) L7010,7038,7632,7924; `PartFs::add_document_relationship` (opc/mod.rs:772) (by name) L7014,7042,7636; `factory_scaffold_bake_b_styles` (document_comparer.rs:2616) L7032; `PartFs::parts` (opc/mod.rs:532) (by name) L7064,7236,7850,7873; `remap_style_refs` (document_comparer.rs:3736) L7091,7660; `merge_revised_font_table` (document_comparer.rs:1384) L7101; `adopt_revised_styles_chrome` (document_comparer.rs:3348) L7109; `ensure_factory_package_chrome` (document_comparer.rs:3402) L7110; `repair_missing_core_relationships` (document_comparer.rs:6250) L7111; `merge_custom_properties` (document_comparer.rs:6188) L7112; `restore_deleted_paragraph_spacing` (document_comparer.rs:1415) L7117; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L7140; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L7143; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L7146,7435; `Dom::name_is` (xmllinq/mod.rs:535) L7160,7777; `Dom::value_str` (xmllinq/mod.rs:1179) L7198,7467; `Dom::remove` (xmllinq/mod.rs:1102) L7208,7810; `strip_unresolved_style_refs` (comparer/footnotes.rs:233) L7258; `style_match_key` (document_comparer.rs:1843) L7312,7415,7424; `inherited_only_style_changes` (document_comparer.rs:1601) L7327; `styles_the_revision_lacks` (document_comparer.rs:1790) L7329; `merge_revised_style_definitions` (document_comparer.rs:2160) L7330; `merge_normal_style_spacing` (document_comparer.rs:777) L7338; `merge_normal_style_rpr` (document_comparer.rs:4343) L7344; `prune_normal_rpr_context_equal_attrs` (document_comparer.rs:3822) L7348; `bake_bothsides_dd_disabling_neutralizers` (document_comparer.rs:3909) L7352; `cascade_normal_change_to_based_styles` (document_comparer.rs:1124) L7355; `normalize_word_paragraph_style_line` (document_comparer.rs:2948) L7357; `align_paragraph_style_fonts_with_normal` (document_comparer.rs:3053) L7359; `resolve_redefined_style_metrics` (document_comparer.rs:4063) L7363; `complete_root_style_change_records` (document_comparer.rs:1873) L7366; `complete_based_style_change_records` (document_comparer.rs:1951) L7370; `remove_inherited_only_style_records` (document_comparer.rs:1672) L7372; `restore_styles` (document_comparer.rs:1814) L7373; `referenced_theme_part` (document_comparer.rs:3511) L7379; `reresolve_theme_color_hexes` (document_comparer.rs:6004) L7381; `coverage_boundary_tests::index` (document_comparer.rs:8792) L7405,7406; `Dom::ancestors` (xmllinq/mod.rs:770) L7492; `effective_para_spacing` (document_comparer.rs:358) L7500,7507; `insert_child_by_rank` (document_comparer.rs:1327) L7548; `synthesize_dangling_numbering` (comparer/footnotes.rs:533) L7620; `adopt_b_notes_when_a_lacks_separators` (document_comparer.rs:5863) L7687; `sync_settings_special_note_ids` (comparer/footnotes.rs:852) L7696,8000; `header_footer_refs_by_section` (document_comparer.rs:129) L7714,7715; `section_pairing` (document_comparer.rs:202) L7718; `pair_header_footer` (document_comparer.rs:238) L7745; `part_rels_agree` (document_comparer.rs:70) L7753; `carry_revised_part_relationships` (document_comparer.rs:25) L7756; `Dom::value` (xmllinq/mod.rs:1169) L7769; `text_of` (changes.rs:806) (by name) L7773,7831; `compare_bodies_faithful` (comparer/mod.rs:109) L7778; `XNamespace::xmlns` (xmllinq/mod.rs:122) L7796; `XNamespace::namespace_name` (xmllinq/mod.rs:140) L7798; `XName::local_name` (xmllinq/mod.rs:208) (by name) L7799; `delete_story_closing_mark` (document_comparer.rs:5585) L7814; `reconcile_part_relationships` (comparer/parts.rs:479) L7841; `PartFs::adopt_missing_content_types` (opc/mod.rs:631) (by name) L7844; `fix_strict_validity_artifacts` (comparer/finalize.rs:11321) L7894; `repair_inherited_invalidity` (comparer/finalize.rs:14767) L7898; `hoist_hyperlinks_out_of_revisions` (comparer/finalize.rs:14594) L7902; `enforce_deleted_text_kinds` (comparer/finalize.rs:14834) L7903; `remove_powertools_scratch_markup` (comparer/finalize.rs:1018) L7904; `strip_propertyless_ppr` (comparer/finalize.rs:6003) L7910; `strip_unrecorded_word_defaults` (comparer/finalize.rs:6077) L7919; `PartFs::read_rels_for` (opc/mod.rs:756) L7934; `rewrite_inserted_external_hyperlinks` (comparer/finalize.rs:6331) L7944; `align_word_table_and_comment_chrome` (comparer/finalize.rs:6215) L7950; `align_remaining_ladder_rungs` (comparer/finalize.rs:14886) L7953; `enforce_part_schema_order` (comparer/finalize.rs:10736) L7965; `declare_extension_namespaces_ignorable` (comparer/finalize.rs:899) L7966; `bind_compatibility_prefixes` (comparer/finalize.rs:983) L7967; `collect_ids` (ambiguous: `collect_ids` (debug.rs:970); `collect_ids` (validate.rs:788)) L7994,7995; `PartFs::to_zip` (opc/mod.rs:548) (by name) L8014
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.as_ref`, `.as_str`, `.clone`, `.cloned`, `.collect`, `.contains_key`, `.contains`, `.dedup`, `.ends_with`, `.entry`, `.enumerate`, `.extend`, `.filter_map`, `.filter`, `.find_map`, `.find`, `.first`, `.flat_map`, `.flatten`, `.fold`, `.get`, `.insert`, `.into_bytes`, `.into_iter`, `.into_owned`, `.into`, `.is_empty`, `.is_none`, `.is_ok`, `.is_some_and`, `.is_some`, `.iter`, `.last`, `.len`, `.map_err`, `.map_or_else`, `.map`, `.ok_or_else`, `.ok`, `.or_default`, `.or_else`, `.or_insert_with`, `.parse`, `.push`, `.sort`, `.starts_with`, `.take_while`, `.take`, `.then`, `.to_string`, `.trim`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `Ok`, `OpcError::PartNotFound`, `Some`, `Vec::new`, `W::body`, `W::del_text`, `W::endnote`, `W::footnote`, `W::id`, `W::name`, `W::object`, `W::p_pr`, `W::p`, `W::r_pr`, `W::t`, `W::tbl_pr`, `W::tbl`, `W::val`, `b_id_for`, `crate::namespaces::MC::name`, `crate::namespaces::PT::default_line`, `crate::namespaces::PT::has_default_table_style`, `crate::namespaces::VML::name`, `crate::namespaces::W14::name`, `crate::namespaces::WNE::name`, `crate::revision_processor::drop_unreferenced_headers_footers`, `dp`, `is_ins_empty`, `std::collections::HashMap::new`, `std::collections::HashSet::new`
- **Macros:** `format!`, `matches!`, `vec!`
- **Called by:** `compare_documents_with_settings` (document_comparer.rs:6414); `compare_documents_internal` (document_comparer.rs:6425)

#### `compare_documents_impl::{fn parse_part_root}`  (L6647-6651)
`fn parse_part_root(dom: &mut Dom, pkg: &PartFs, name: &str) -> Option<NodeId>`

- **Does:** B.4 — reference-driven notes processing: parse both documents' notes parts AND an independent copy of the original's parts (the withRevisions parts — C# gets them from the wmlResult clone of preprocessed source1) into the same Dom as the bodies. _(from a code comment)_
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:498) L6648; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L6649; `Dom::root` (xmllinq/mod.rs:822) L6650
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501)

## Test functions (127)

- `tests::parse` (L8031): _no doc comment_
- `tests::restored_deleted_spacing_leaves_scratch_unids_behind` (L8044): A's working copy carries `pt14:Unid` on its spacing; the restored spacing on the deleted paragraph must hold only the `w:` attributes.
- `tests::revised_fonts_join_the_font_table` (L8082): Fonts only the revised document declares join the output's font table (file_46 × file_47: without Liberation Serif's entry Word fell back to Times New Roman where its own redline used Hiragino Mincho).
- `tests::next_free_revision_id_is_max_plus_one_across_change_families` (L8123): `next_free_revision_id` must be one greater than the max numeric id on ANY `w:*Change` revision element in the stylesheet, never a hardcoded 1.
- `tests::next_free_revision_id_counts_list_move_range_and_cell_carriers` (L8151): Every `w:id` revision carrier raises the floor, not only the `*Change` records: a list, move-range or table-cell revision holding the next id would collide with a synthesized `w:pPrChange`.
- `tests::add_rpr_child_keeps_sz_in_schema_order_after_position` (L8174): sz must be inserted after position/kern, NOT immediately after rFonts — rFonts < color < spacing < w < kern < position < sz in EG_RPrBase.
- `tests::revision_json_matches_cli_shape_and_escapes_control_chars` (L8208): `revision_to_json` / `revisions_to_json` are the single serialization of the wasm `getRevisions` and Python `get_revisions` bindings: one object shape, full string escaping — backslash, quote, and EVERY control char < 0x20 (document text can carry tabs, CRs, vertical tabs).
- `tests::word_canonical_style_id_preserves_toc_builtins` (L8274): _no doc comment_
- `tests::word_canonical_style_id_keeps_builtin_ids_that_differ_from_their_names` (L8291): _no doc comment_
- `tests::word_canonical_style_id_drops_punctuation` (L8318): Word's style ids hold letters and digits only: "Normal (Web)" is `NormalWeb` (evals comments_doc × document).
- `tests::normal_takes_b_theme_fonts_from_its_doc_defaults` (L8330): B's docDefaults name their fonts by theme (`w:asciiTheme="minorHAnsi"`, the form Word itself writes) and A's name Times New Roman outright.
- `tests::normal_is_found_by_name_when_no_style_is_the_default` (L8372): LibreOffice names its Normal `style0` and marks no paragraph style as the default; Word still finds it by its name (multi_section_nested_ table_rowspan: the Word redline carries B's whole Normal, ours left it bare because B "had no Normal").
- `tests::stylesheet` (L8388): A stylesheet with a Normal; `dd` = (rPrDefault, pPrDefault) contents, `None` for no docDefaults at all.
- `tests::normal_pair` (L8404): _no doc comment_
- `tests::merge_normals` (L8421): Run `merge_normal_style_spacing` then `merge_normal_style_rpr` over A's and B's stylesheets; returns the dom and the output root.
- `tests::live_normal_attrs` (L8432): `(attribute, value)` pairs of the live Normal's `container/local`.
- `tests::attrs` (L8450): _no doc comment_
- `tests::normal_uses_word_factory_defaults_when_b_has_no_doc_defaults` (L8465): A LibreOffice B with no docDefaults at all: Word reads it with its own factory defaults (after 160, line 278, kern 2, standard contextual ligatures) and writes them into the live Normal (multi_section_nested_table_rowspan, table_bookmark_end × table_vmerge_colspan).
- `tests::normal_writes_the_implicit_complex_script_size` (L8500): B declares no complex-script size while A's docDefaults do: Word writes the implicit 20 half-points (17 of 17 corpus Word redlines).
- `tests::normal_writes_only_the_language_attributes_that_change` (L8522): Word writes only the language attributes B changes against A's docDefaults, reading a missing attribute as en-US / en-US / ar-SA (file_103 × file_104: eastAsia zh-CN alone; mcdoc_meeting_agenda: B silent, A zh-CN → eastAsia en-US).
- `tests::normal_takes_b_run_defaults_when_only_b_paragraph_props_are_stored` (L8565): file_103 × file_104: A's Normal is bare, B's carries only a pPr.
- `tests::live_normal_ppr` (L8591): Live Normal pPr, children in order, pPrChange left out.
- `tests::normal_takes_b_doc_default_indent_and_justification` (L8615): file_103 × file_104: B keeps indents, justification and line-unit spacing in its docDefaults, which the redline (A's docDefaults) loses.
- `tests::normal_neutralizes_a_doc_default_indent_and_justification` (L8657): file_104 × file_105, the reverse: A's docDefaults indent and justify, B's do not.
- `tests::normal_clears_a_doc_default_borders` (L8710): A's docDefaults draw paragraph borders, B's do not: Word writes a `nil` border on every edge A draws, even when no spacing changes (sd_1494_table_left_indent × sdpr_titleonly).
- `coverage_boundary_tests::xml` (L8760): _no doc comment_
- `coverage_boundary_tests::root` (L8769): _no doc comment_
- `coverage_boundary_tests::child` (L8774): _no doc comment_
- `coverage_boundary_tests::children` (L8778): _no doc comment_
- `coverage_boundary_tests::style` (L8785): _no doc comment_
- `coverage_boundary_tests::index` (L8792): _no doc comment_
- `coverage_boundary_tests::package` (L8799): _no doc comment_
- `coverage_boundary_tests::part_root` (L8815): _no doc comment_
- `coverage_boundary_tests::settings` (L8821): _no doc comment_
- `coverage_boundary_tests::assert_revision` (L8829): _no doc comment_
- `coverage_boundary_tests::merged_bodies_preserve_content_order_and_only_last_section_properties` (L8839): _no doc comment_
- `coverage_boundary_tests::section_pairing_accounts_for_inserted_deleted_and_recorded_breaks` (L8873): _no doc comment_
- `coverage_boundary_tests::paragraph_spacing_tracks_table_precedence_and_missing_ancestors` (L8889): _no doc comment_
- `coverage_boundary_tests::effective_run_properties_merge_independent_slots_and_clear_alternatives` (L8915): _no doc comment_
- `coverage_boundary_tests::style_cycles_stop_and_self_based_style_does_not_hide_own_properties` (L8952): _no doc comment_
- `coverage_boundary_tests::inherited_only_records_are_suppressed_but_inbound_history_survives` (L8975): _no doc comment_
- `coverage_boundary_tests::a_styles_two_records_share_one_id` (L9011): Word gives a style's `w:pPrChange` and `w:rPrChange` one id (199 of 199 corpus styles carrying both), so the pair is one change.
- `coverage_boundary_tests::recorded_defaults_complete_slots_without_overwriting_explicit_values` (L9048): _no doc comment_
- `coverage_boundary_tests::root_records_gain_missing_companion_and_keep_recorded_original` (L9109): _no doc comment_
- `coverage_boundary_tests::absent_revised_styles_restore_distinct_twins_and_skip_removed_targets` (L9154): _no doc comment_
- `coverage_boundary_tests::canonical_style_ids_refuse_collisions_and_remap_only_known_references` (L9187): _no doc comment_
- `coverage_boundary_tests::style_structure_adoption_preserves_existing_defaults_and_empty_explicit_spacing` (L9233): _no doc comment_
- `coverage_boundary_tests::adopted_header_marks_content_and_paragraph_properties_with_pinned_provenance` (L9281): _no doc comment_
- `coverage_boundary_tests::deleted_footer_preserves_last_mark_and_records_original_properties` (L9307): _no doc comment_
- `coverage_boundary_tests::one_sided_header_footer_tables_mark_every_row` (L9358): Word 16.115 hangs opening a redline whose footer table has every cell deleted but no row marked (b09, release 0.12.0 sample); Word's own redline of the pair deletes the row too.
- `coverage_boundary_tests::one_sided_story_marks_take_ids_above_the_story_s_own` (L9396): A one-sided story's new marks take ids above the ones it already carries, so a kept revision never shares its id with a new one.
- `coverage_boundary_tests::one_sided_story_fields_join_the_story_revision` (L9424): A paragraph whose only content is a field (`fldSimple`, empty or with a result) is content: left out, a dropped footer's NUMPAGES stayed live, the b15 shape Word 16.115 hung on.
- `coverage_boundary_tests::deletion_skips_inbound_changes_and_preserves_existing_blank_history` (L9453): _no doc comment_
- `coverage_boundary_tests::story_closing_mark_restores_original_props_and_does_not_double_mark` (L9472): _no doc comment_
- `coverage_boundary_tests::closing_mark_refuses_live_content_and_handles_empty_or_unformatted_stories` (L9497): _no doc comment_
- `coverage_boundary_tests::child_creation_places_properties_before_schema_successors_and_reuses_nodes` (L9537): _no doc comment_
- `coverage_boundary_tests::final_header_slots_strip_only_inherited_non_revised_references` (L9555): _no doc comment_
- `coverage_boundary_tests::header_pairing_uses_surviving_section_inheritance_and_content_identity` (L9599): _no doc comment_
- `coverage_boundary_tests::a_colliding_header_or_footer_takes_the_first_free_number` (L9642): _no doc comment_
- `coverage_boundary_tests::unique_part_names_handle_binary_collisions_reuse_and_flat_names` (L9668): _no doc comment_
- `coverage_boundary_tests::relationship_agreement_refuses_missing_mismatched_and_dangling_targets` (L9701): _no doc comment_
- `coverage_boundary_tests::carry_relationships_reuses_minted_id_and_refuses_unresolved_references` (L9745): _no doc comment_
- `coverage_boundary_tests::notes_merge_only_structural_definitions_ahead_of_rectified_content` (L9790): _no doc comment_
- `coverage_boundary_tests::notes_copy_preserves_media_and_escapes_external_links_without_io` (L9831): _no doc comment_
- `coverage_boundary_tests::notes_adoption_refuses_missing_separator_and_preserves_original_separator` (L9871): _no doc comment_
- `coverage_boundary_tests::custom_property_merge_is_case_insensitive_original_wins_and_new_ids_are_unique` (L9898): _no doc comment_
- `coverage_boundary_tests::custom_property_adoption_refuses_unrelated_collision_and_malformed_xml` (L9927): _no doc comment_
- `coverage_boundary_tests::empty_missing_and_non_xml_package_parts_are_safe_noops` (L9962): _no doc comment_
- `coverage_boundary_tests::preprocessing_refuses_orphan_notes_before_mutating_package` (L9993): _no doc comment_
- `coverage_boundary_tests::admission_preserves_typed_refusal_and_identifies_document_side` (L10008): _no doc comment_
- `coverage_boundary_tests::strict_namespace_normalization_rewrites_only_the_three_ooxml_families` (L10034): _no doc comment_
- `coverage_boundary_tests::final_header_footer_adoption_respects_existing_body_owners_and_cached_donor_parts` (L10052): _no doc comment_
- `coverage_boundary_tests::disabling_default_neutralizers_preserve_tracked_history_at_inheritance_limit` (L10247): _no doc comment_
- `coverage_boundary_tests::normal_cascade_preserves_partial_spacing_owners_without_inventing_metrics` (L10330): _no doc comment_
- `coverage_boundary_tests::header_adoption_preserves_colliding_media_and_uses_correct_content_types` (L10403): _no doc comment_
- `coverage_boundary_tests::a_renamed_header_copy_never_hides_another_header_of_the_revision` (L10502): _no doc comment_
- `coverage_boundary_tests::a_reused_header_keeps_its_own_relationships` (L10555): _no doc comment_
- `coverage_boundary_tests::header_adoption_refuses_unresolvable_references_and_missing_story_structure` (L10593): _no doc comment_
- `coverage_boundary_tests::original_footer_deletion_is_per_kind_and_duplicate_references_are_processed_once` (L10628): _no doc comment_
- `coverage_boundary_tests::revised_chrome_preserves_existing_parts_and_repairs_relationships_idempotently` (L10664): _no doc comment_
- `coverage_boundary_tests::revision_ids_ignore_missing_and_nonnumeric_values_and_saturate` (L10726): _no doc comment_
- `coverage_boundary_tests::revised_style_merge_tracks_both_blocks_without_copying_revised_history` (L10741): _no doc comment_
- `coverage_boundary_tests::deleted_spacing_restoration_refuses_unmarked_missing_and_already_spaced_paragraphs` (L10809): _no doc comment_
- `coverage_boundary_tests::separator_media_collisions_keep_a_resolvable_relative_relationship` (L10832): _no doc comment_
- `coverage_boundary_tests::normal_run_adoption_preserves_complete_old_properties_and_revised_slot_ownership` (L10865): _no doc comment_
- `coverage_boundary_tests::theme_cache_recaching_preserves_independent_fill_color_and_tolerated_source_hexes` (L10963): _no doc comment_
- `coverage_boundary_tests::header_insertion_wraps_supported_owned_payload_and_retains_noncontent_and_foreign_revisions` (L11078): _no doc comment_
- `coverage_boundary_tests::normal_history_cascade_keeps_own_font_language_and_spacing_slots_and_existing_records` (L11146): _no doc comment_
- `coverage_boundary_tests::tracked_change_admission_scans_each_xml_story_and_refuses_binary_or_non_xml_carriers` (L11260): _no doc comment_
- `coverage_boundary_tests::section_story_references_resolve_actual_opc_targets_before_legacy_word_paths` (L11293): _no doc comment_
- `coverage_package_source_matrix_tests::xml` (L11366): _no doc comment_
- `coverage_package_source_matrix_tests::root` (L11374): _no doc comment_
- `coverage_package_source_matrix_tests::signature` (L11381): Semantic properties include every attribute and child, independently of prefix declaration locations chosen when the OPC part is serialized. _(from a code comment)_
- `coverage_package_source_matrix_tests::view` (L11415): _no doc comment_
- `coverage_package_source_matrix_tests::add_part` (L11485): _no doc comment_
- `coverage_package_source_matrix_tests::package` (L11492): _no doc comment_
- `coverage_package_source_matrix_tests::public_comparison_preserves_complete_sources_across_package_features` (L11595): _no doc comment_
- `coverage_package_source_matrix_tests::defaults_delta_distinguishes_implicit_on_off_and_explicit_overrides` (L11659): _no doc comment_
- `coverage_package_source_matrix_tests::normal_sync_preserves_history_and_replaces_indent_in_schema_order` (L11731): _no doc comment_
- `coverage_package_source_matrix_tests::accepted_source_revision_converges_to_identical_final_before_comparison` (L11811): _no doc comment_
- `comparer_relationship_contract_tests::package` (L11862): _no doc comment_
- `comparer_relationship_contract_tests::referenced_relationship_agreement_requires_matching_type_mode_and_payload_or_external_target` (L11866): _no doc comment_
- `comparer_relationship_contract_tests::repeated_hyperlinks_share_one_carried_relationship_and_keep_independent_source_text_and_format` (L11957): _no doc comment_
- `authored_style_chain_boundary_tests::parse` (L12013): _no doc comment_
- `authored_style_chain_boundary_tests::leaf` (L12032): _no doc comment_
- `authored_style_chain_boundary_tests::attrs` (L12039): _no doc comment_
- `authored_style_chain_boundary_tests::inherited_style_font_alternatives_and_missing_blocks_keep_exact_declarations` (L12048): _no doc comment_
- `authored_style_chain_boundary_tests::based_on_cycles_stop_after_unique_authored_owners_with_nearest_properties_winning` (L12093): _no doc comment_
- `authored_style_chain_boundary_tests::bounded_style_walk_reads_exactly_the_nearest_thirty_two_owners` (L12115): _no doc comment_
- `isolated_header_container_revision_boundary_tests::semantic` (L12156): _no doc comment_
- `isolated_header_container_revision_boundary_tests::parsed` (L12192): _no doc comment_
- `isolated_header_container_revision_boundary_tests::pure_header_container_payloads_recover_every_authored_property_and_fallback` (L12199): _no doc comment_
- `redefined_style_chain_source_owner_tests::stylesheet` (L12313): _no doc comment_
- `redefined_style_chain_source_owner_tests::root` (L12355): _no doc comment_
- `redefined_style_chain_source_owner_tests::redefined_style_metrics_bound_cycles_depth_and_default_owners_without_source_history_loss` (L12361): _no doc comment_
- `namespace_declared_default_property_owner_tests::locally_declared_namespace_metadata_never_becomes_a_paragraph_property_delta` (L12423): _no doc comment_
- `adopted_story_relationship_owner_tests::signature` (L12478): _no doc comment_
- `adopted_story_relationship_owner_tests::parsed` (L12513): _no doc comment_
- `adopted_story_relationship_owner_tests::empty_main` (L12519): _no doc comment_
- `adopted_story_relationship_owner_tests::adopted_root_and_nested_stories_keep_colliding_media_relationships_resolvable` (L12527): _no doc comment_
- `word_style_line_owned_boundary_tests::parse` (L12659): _no doc comment_
- `word_style_line_owned_boundary_tests::semantic` (L12665): _no doc comment_
- `word_style_line_owned_boundary_tests::word_single_line_style_propagation_keeps_authored_history_and_unrelated_properties` (L12688): _no doc comment_
- `public_numbering_definition_source_owner_tests::canonical` (L12758): _no doc comment_
- `public_numbering_definition_source_owner_tests::owned_projection` (L12791): Numbering IDs are package-local references, not formatting. _(from a code comment)_
- `public_numbering_definition_source_owner_tests::package` (L12891): _no doc comment_
- `public_numbering_definition_source_owner_tests::public_colliding_list_definitions_preserve_each_source_level_override_and_paragraph_history` (L12941): _no doc comment_
