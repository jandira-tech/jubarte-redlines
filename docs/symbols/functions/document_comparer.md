# Functions and methods in `src/document_comparer.rs`

107 non-test, 129 test. Part of the [symbol manifest](../README.md).

#### `carry_revised_part_relationships`  (L25-64)
`fn carry_revised_part_relationships( out: &mut PartFs, part_a: &str, (pkg2, part_b): (&PartFs, &str), xml: &str, ) -> Option<String>`

- **Does:** `xml` (part `part_b` of `pkg2`) with each relationship it references carried onto `part_a` of `out` under a fresh id (an image with A's bytes reuses A's relationship), so B's content means what it meant in B once it is diffed into A's part.
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L31; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L32; `Dom::root` (xmllinq/mod.rs:822) L33; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L36; `Dom::attributes` (xmllinq/mod.rs:899) L37; `Dom::name` (xmllinq/mod.rs:526) L41; `required_rel_type_suffix` (comparer/parts.rs:127) L43; `XName::local_name` (xmllinq/mod.rs:208) (by name) L43; `carry_relationship` (comparer/parts.rs:517) L48; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L60; `Dom::serialize_document` (xmllinq/mod.rs:1338) L63
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.contains`, `.ends_with`, `.get`, `.insert`, `.is_none_or`, `Some`, `std::collections::HashMap::new`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::carry_relationships_reuses_minted_id_and_refuses_unresolved_references` (document_comparer.rs:9852); `comparer_relationship_contract_tests::repeated_hyperlinks_share_one_carried_relationship_and_keep_independent_source_text_and_format` (document_comparer.rs:12086)

#### `part_rels_agree`  (L70-110)
`fn part_rels_agree( (pkg1, part_a): (&PartFs, &str), (pkg2, part_b): (&PartFs, &str), xml: &str, ) -> bool`

- **Does:** Every relationship `xml` (part `part_b` of `pkg2`) references resolves in `part_a`'s rels to the same type, mode and target, and an internal target to the same bytes.
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L75; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L76; `Dom::root` (xmllinq/mod.rs:822) L77; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L80; `Dom::attributes` (xmllinq/mod.rs:899) L83; `PartFs::read_rels_for` (opc/mod.rs:991) (by name) L90; `find` (ambiguous: `find` (markdown/pages.rs:452); `find` (markdown/unified.rs:474)) L97; `PartFs::part_bytes` (opc/mod.rs:628) (by name) L106,107; `PartFs::resolve_rel_target` (opc/mod.rs:829) (by name) L106,107
- **Calls (std/external/unresolved):** `.all`, `.as_deref`, `.cloned`, `.collect`, `.contains`, `.filter`, `.find`, `.flat_map`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.map`, `Some`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::relationship_agreement_refuses_missing_mismatched_and_dangling_targets` (document_comparer.rs:9808); `comparer_relationship_contract_tests::referenced_relationship_agreement_requires_matching_type_mode_and_payload_or_external_target` (document_comparer.rs:11995)

#### `header_footer_refs`  (L116-122)
`fn header_footer_refs(pkg: &PartFs) -> Vec<(String, String, String)>`

- **Does:** The header/footer parts a document references, as (kind, type, part-name): kind ∈ {"header","footer"}, type ∈ {"default","even","first"}.
- **Calls (crate):** `header_footer_refs_by_section` (document_comparer.rs:129) L117
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`
- **Called by:** `mark_a_only_hf_content_as_deleted` (document_comparer.rs:5571); `coverage_boundary_tests::section_story_references_resolve_actual_opc_targets_before_legacy_word_paths` (document_comparer.rs:11412)

#### `header_footer_refs_by_section`  (L129-183)
`fn header_footer_refs_by_section(pkg: &PartFs) -> (usize, Vec<SectionRef>)`

- **Does:** A document's section count and [`header_footer_refs`], each tagged with the index of the section that references it.
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:1070) L130; `PartFs::part_string` (opc/mod.rs:636) L133; `PartFs::read_rels_for` (opc/mod.rs:991) L136; `Dom::new` (xmllinq/mod.rs:387) L144; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L145; `Dom::root` (xmllinq/mod.rs:822) L146; `section_props` (document_comparer.rs:187) L149; `Dom::descendants` (xmllinq/mod.rs:668) L155; `Dom::attribute` (xmllinq/mod.rs:874) L156,164; `Dom::parent` (xmllinq/mod.rs:597) L160; `PartFs::resolve_rel_target` (opc/mod.rs:829) L167; `PartFs::part_bytes` (opc/mod.rs:628) L171,173
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.collect`, `.get`, `.is_some`, `.iter`, `.len`, `.map`, `.position`, `.push`, `.starts_with`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `R::name`, `Some`, `Vec::new`, `W::name`
- **Called by:** `header_footer_refs` (document_comparer.rs:116); `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::empty_missing_and_non_xml_package_parts_are_safe_noops` (document_comparer.rs:10069); `coverage_boundary_tests::section_story_references_resolve_actual_opc_targets_before_legacy_word_paths` (document_comparer.rs:11412)

#### `section_props`  (L187-195)
`fn section_props(dom: &Dom, root: NodeId) -> Vec<NodeId>`

- **Does:** A document's sections in order: each paragraph mark's `w:sectPr`, then the body's own.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L188; `Dom::parent` (xmllinq/mod.rs:597) L191; `Dom::name_is` (xmllinq/mod.rs:535) L192
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_some_and`, `Some`, `W::body`, `W::p_pr`, `W::sect_pr`
- **Called by:** `header_footer_refs_by_section` (document_comparer.rs:129); `section_pairing` (document_comparer.rs:202)

#### `section_pairing`  (L202-228)
`fn section_pairing( redline_xml: &str, sections_a: usize, sections_b: usize, ) -> Option<Vec<Option<usize>>>`

- **Does:** The revised section each original section became, read off the redline's section breaks: the section of a deleted break has none (Word leaves its headers as they were, probes hs1/hs3), an inserted break's is the revision's alone.
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L207; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L208; `Dom::root` (xmllinq/mod.rs:822) L209; `Dom::parent` (xmllinq/mod.rs:597) L211; `Dom::name_is` (xmllinq/mod.rs:535) L212; `Dom::element` (xmllinq/mod.rs:655) L213,214; `section_props` (document_comparer.rs:187) L218; `marked` (ambiguous, 5 candidates) L219,220
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.is_some_and`, `.is_some`, `.len`, `.push`, `.then_some`, `Vec::new`, `W::del`, `W::ins`, `W::p_pr`, `W::r_pr`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::section_pairing_accounts_for_inserted_deleted_and_recorded_breaks` (document_comparer.rs:8980)

#### `pair_header_footer`  (L238-264)
`fn pair_header_footer( pkg1: &PartFs, pkg2: &PartFs, refs_b: &[SectionRef], (kind, ty, part_a): (&str, &str, &str), at: usize, section: Option<Option<usize>>, ) -> Option<String>`

- **Does:** The revised part a header/footer part is diffed against, or none when A's part is unchanged in B.
- **Calls (crate):** `PartFs::part_bytes` (opc/mod.rs:628) L251,254
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.collect`, `.filter`, `.find`, `.get`, `.iter`, `.last`, `.map`, `.or`, `.rev`, `.to_string`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::header_pairing_uses_surviving_section_inheritance_and_content_identity` (document_comparer.rs:9706)

#### `find_normal_style`  (L271-296)
`fn find_normal_style(dom: &Dom, styles_root: NodeId) -> Option<NodeId>`

- **Does:** The `w:style[@w:type='paragraph' @w:styleId='Normal']` element under a styles root, falling back to the `w:default="1"` paragraph style.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L272; `Dom::attribute` (xmllinq/mod.rs:874) L275,280,285,292; `Dom::element` (xmllinq/mod.rs:655) L291
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.copied`, `.eq_ignore_ascii_case`, `.filter`, `.find`, `.into_iter`, `.is_some_and`, `.iter`, `.or_else`, `Some`, `W::name`, `W::val`
- **Called by:** `merge_normal_style_spacing` (document_comparer.rs:777); `cascade_normal_change_to_based_styles` (document_comparer.rs:1124); `inherited_only_style_changes` (document_comparer.rs:1601); `merge_revised_style_definitions` (document_comparer.rs:2160); `normalize_word_paragraph_style_line` (document_comparer.rs:2948); `align_paragraph_style_fonts_with_normal` (document_comparer.rs:3053); `prune_normal_rpr_context_equal_attrs` (document_comparer.rs:3822); `bake_bothsides_dd_disabling_neutralizers` (document_comparer.rs:3909); `merge_normal_style_rpr` (document_comparer.rs:4343); `tests::normal_takes_b_theme_fonts_from_its_doc_defaults` (document_comparer.rs:8382); `tests::normal_is_found_by_name_when_no_style_is_the_default` (document_comparer.rs:8424); `tests::live_normal_attrs` (document_comparer.rs:8484); +5 more

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
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::paragraph_spacing_tracks_table_precedence_and_missing_ancestors` (document_comparer.rs:8996)

#### `next_free_revision_id`  (L469-481)
`fn next_free_revision_id(dom: &Dom, styles_root: NodeId) -> u32`

- **Does:** The next free revision id under `styles_root`: one greater than the maximum numeric `w:id` on any revision record present (0 when none).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L472; `Dom::attribute` (xmllinq/mod.rs:874) L473
- **Calls (std/external/unresolved):** `.max`, `.parse`, `.saturating_add`, `Some`, `W::id`, `W::name`
- **Called by:** `style_record_id` (document_comparer.rs:491); `mark_adopted_hf_content_as_inserted` (document_comparer.rs:5452); `delete_story_closing_mark` (document_comparer.rs:5593); `mark_hf_part_content_as_deleted` (document_comparer.rs:5666); `compare_documents_impl` (document_comparer.rs:6520); `tests::next_free_revision_id_is_max_plus_one_across_change_families` (document_comparer.rs:8175); `tests::next_free_revision_id_counts_list_move_range_and_cell_carriers` (document_comparer.rs:8203); `coverage_boundary_tests::revision_ids_ignore_missing_and_nonnumeric_values_and_saturate` (document_comparer.rs:10839)

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
- **Called by:** `doc_default_ppr_delta` (document_comparer.rs:566); `coverage_package_source_matrix_tests::defaults_delta_distinguishes_implicit_on_off_and_explicit_overrides` (document_comparer.rs:11780)

#### `doc_default_ppr_delta`  (L566-680)
`fn doc_default_ppr_delta( dom: &mut Dom, out_root: NodeId, b_root: NodeId, b_style: Option<NodeId>, ) -> DocDefaultPprDelta`

- **Does:** The docDefaults paragraph properties the after/before/line rule of [`merge_normal_style_spacing`] does not cover, as Word writes them into the live Normal.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L573,574,575,579,581; `Dom::elements` (xmllinq/mod.rs:640) L585,629; `Dom::name` (xmllinq/mod.rs:526) L586,630; `XName::local_name` (xmllinq/mod.rs:208) (by name) L587,655; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L588,610,656; `coverage_boundary_tests::child` (document_comparer.rs:8826) L598,599,650; `style_prop_signature` (document_comparer.rs:1485) L600; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L604; `Dom::new_element` (xmllinq/mod.rs:478) L606,631; `Dom::attributes` (xmllinq/mod.rs:899) L609; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L611,616,621,632,640; `on_off_is_off` (document_comparer.rs:544) L624,637; `Dom::add` (xmllinq/mod.rs:1074) L633; `Dom::attributes_iter` (xmllinq/mod.rs:886) L654; `Dom::attribute` (xmllinq/mod.rs:874) L665
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_str`, `.contains`, `.flatten`, `.into_iter`, `.iter`, `.map`, `.or_else`, `.push`, `.retain_mut`, `.to_string`, `.unwrap_or_else`, `Some`, `String::new`, `Vec::new`, `W::name`, `W::p_pr`, `W::val`, `dd_ppr`, `get`, `sp`
- **Macros:** `matches!`
- **Called by:** `merge_normal_style_spacing` (document_comparer.rs:777); `coverage_package_source_matrix_tests::defaults_delta_distinguishes_implicit_on_off_and_explicit_overrides` (document_comparer.rs:11780); `namespace_declared_default_property_owner_tests::locally_declared_namespace_metadata_never_becomes_a_paragraph_property_delta` (document_comparer.rs:12545)

#### `sync_normal_ppr_with_revised`  (L686-758)
`fn sync_normal_ppr_with_revised(dom: &mut Dom, ppr: NodeId, b_style: Option<NodeId>)`

- **Does:** M64/M70/M72: after the spacing merge, Normal's other declared pPr children follow the revision — B's `w:ind` replaces A's, A-only children drop into the pPrChange record, B's own children are copied in schema order.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L692,693,694,700,742,751; `Dom::remove` (xmllinq/mod.rs:1118) L695,738; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L698,754; `Dom::nodes` (xmllinq/mod.rs:604) L702; `Dom::add_before_self` (xmllinq/mod.rs:1130) L708; `Dom::add` (xmllinq/mod.rs:1074) L709; `Dom::add_first` (xmllinq/mod.rs:1101) L712; `Dom::elements` (xmllinq/mod.rs:640) L727,744; `Dom::name` (xmllinq/mod.rs:526) L731,746; `XName::local_name` (xmllinq/mod.rs:208) (by name) L750; `insert_child_by_rank` (document_comparer.rs:1327) L755
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.collect`, `.copied`, `.filter`, `.get`, `.into_iter`, `.is_some`, `.iter`, `.position`, `.to_string`, `W::name`
- **Called by:** `merge_normal_style_spacing` (document_comparer.rs:777); `coverage_package_source_matrix_tests::normal_sync_preserves_history_and_replaces_indent_in_schema_order` (document_comparer.rs:11852)

#### `merge_normal_style_spacing`  (L777-1109)
`fn merge_normal_style_spacing( dom: &mut Dom, out_root: NodeId, b_root: NodeId, settings: &WmlComparerSettings, ) -> bool`

- **Does:** M-PAG mechanism 2: rewrite the output stylesheet's Normal to B's target spacing with a `w:pPrChange` holding A's old pPr.
- **Calls (crate):** `find_normal_style` (document_comparer.rs:271) L783,803; `normal_spacing` (document_comparer.rs:299) L787,863; `docdefaults_ppr_spacing` (document_comparer.rs:336) L795,861; `b_docdefaults_ppr_spacing` (document_comparer.rs:324) L807,862; `Dom::element` (xmllinq/mod.rs:655) L808,809,811,814,867,868...; `spacing_before_attr` (document_comparer.rs:311) L876,877,878; `doc_default_ppr_delta` (document_comparer.rs:566) L932; `Dom::elements` (xmllinq/mod.rs:640) L941; `Dom::name` (xmllinq/mod.rs:526) L945; `style_prop_signature` (document_comparer.rs:1485) L948; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L985; `Dom::descendants` (xmllinq/mod.rs:668) L986; `Dom::remove` (xmllinq/mod.rs:1118) L987,1066,1086; `Dom::new_element` (xmllinq/mod.rs:478) L992,994,1012,1026,1055,1075...; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L996,999,1002,1037,1061,1063...; `Dom::add` (xmllinq/mod.rs:1074) L1004,1016,1106,1107; `Dom::add_before_self` (xmllinq/mod.rs:1130) L1015; `Dom::add_first` (xmllinq/mod.rs:1101) L1027,1056; `set` (ambiguous: `deeper_boundary_tests::set` (comparer/comments.rs:1863); `set` (comparer/tables.rs:29)) L1043,1044,1045,1046; `sync_normal_ppr_with_revised` (document_comparer.rs:686) L1047,1069; `insert_child_by_rank` (document_comparer.rs:1327) L1076,1088; `style_record_id` (document_comparer.rs:491) L1094
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_ref`, `.as_str`, `.clone`, `.collect`, `.filter`, `.into_iter`, `.into`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.map`, `.or_else`, `.sort`, `.then`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `Some`, `String::new`, `W::name`, `W::p_pr`, `b_eff`, `ctx`, `dd_val`, `pick`, `rest_signature`, `sp_node`, `stored_sp_node`, `stored`
- **Macros:** `matches!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `tests::merge_normals` (document_comparer.rs:8473); `tests::normal_takes_b_doc_default_indent_and_justification` (document_comparer.rs:8667); `tests::normal_neutralizes_a_doc_default_indent_and_justification` (document_comparer.rs:8709); `tests::normal_clears_a_doc_default_borders` (document_comparer.rs:8762)

#### `cascade_normal_change_to_based_styles`  (L1124-1284)
`fn cascade_normal_change_to_based_styles( dom: &mut Dom, styles_root: NodeId, settings: &WmlComparerSettings, ) -> bool`

- **Does:** M111 — cascade Normal's pPrChange/rPrChange onto basedOn=Normal styles.
- **Calls (crate):** `find_normal_style` (document_comparer.rs:271) L1129; `Dom::element` (xmllinq/mod.rs:655) L1132,1134,1135,1137,1142,1143...; `Dom::elements` (xmllinq/mod.rs:640) L1148,1179,1260; `Dom::attribute` (xmllinq/mod.rs:874) L1150,1153,1158,1194,1195,1198; `Dom::new_element` (xmllinq/mod.rs:478) L1168,1177,1209,1233,1239; `Dom::add_before_self` (xmllinq/mod.rs:1130) L1170; `Dom::add` (xmllinq/mod.rs:1074) L1171,1187,1191,1222,1223,1234...; `Dom::name` (xmllinq/mod.rs:526) L1180,1183,1261; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1186,1190,1252,1265; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1202,1205,1211,1212,1217,1241...; `style_record_id` (document_comparer.rs:491) L1210,1240; `Dom::descendants` (xmllinq/mod.rs:668) L1254; `Dom::remove` (xmllinq/mod.rs:1118) L1255,1273; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1270,1271,1275; `complete_attributes` (document_comparer.rs:2054) L1271; `add_rpr_child_in_order` (document_comparer.rs:3309) L1275
- **Calls (std/external/unresolved):** `.and_then`, `.contains`, `.is_none`, `.map`, `.to_string`, `Some`, `W::name`, `W::val`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::normal_cascade_preserves_partial_spacing_owners_without_inventing_metrics` (document_comparer.rs:10443); `coverage_boundary_tests::normal_history_cascade_keeps_own_font_language_and_spacing_slots_and_existing_records` (document_comparer.rs:11259)

#### `insert_child_by_rank`  (L1327-1343)
`fn insert_child_by_rank( dom: &mut Dom, parent: NodeId, child: NodeId, local: &str, rank_of: &dyn Fn(&str) -> usize, )`

- **Does:** Insert `child` under `parent` at the position `order` gives its `local` name: immediately before the first existing child that sorts after it, else appended.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1335; `Dom::name` (xmllinq/mod.rs:526) L1336; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1337; `Dom::add_before_self` (xmllinq/mod.rs:1130) L1340; `Dom::add` (xmllinq/mod.rs:1074) L1341
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.is_some_and`, `rank_of`
- **Called by:** `sync_normal_ppr_with_revised` (document_comparer.rs:686); `merge_normal_style_spacing` (document_comparer.rs:777); `restore_deleted_paragraph_spacing` (document_comparer.rs:1415); `merge_revised_style_definitions` (document_comparer.rs:2160); `factory_scaffold_bake_b_styles` (document_comparer.rs:2616); `normalize_word_paragraph_style_line` (document_comparer.rs:2948); `bake_bothsides_dd_disabling_neutralizers` (document_comparer.rs:3909); `resolve_redefined_style_metrics` (document_comparer.rs:4063); `compare_documents_impl` (document_comparer.rs:6520)

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
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L1387; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L1388,1389; `Dom::root` (xmllinq/mod.rs:822) L1390; `Dom::elements` (xmllinq/mod.rs:640) L1391,1397,1402; `Dom::attribute` (xmllinq/mod.rs:874) L1394,1398; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1401; `Dom::name` (xmllinq/mod.rs:526) L1403; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1404; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1404; `Dom::remove` (xmllinq/mod.rs:1118) L1406; `Dom::add` (xmllinq/mod.rs:1074) L1409; `Dom::serialize_element` (xmllinq/mod.rs:1315) L1412
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter_map`, `.into_iter`, `.is_none_or`, `.is_some_and`, `.map`, `.starts_with`, `.then`, `Some`, `W::name`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `tests::revised_fonts_join_the_font_table` (document_comparer.rs:8134); `coverage_boundary_tests::empty_missing_and_non_xml_package_parts_are_safe_noops` (document_comparer.rs:10069)

#### `restore_deleted_paragraph_spacing`  (L1415-1471)
`fn restore_deleted_paragraph_spacing(a_xml: &str, out_xml: &str) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L1419,1443; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L1420,1444; `Dom::root` (xmllinq/mod.rs:822) L1421,1445; `Dom::descendants` (xmllinq/mod.rs:668) L1422,1447; `Dom::attribute` (xmllinq/mod.rs:874) L1423,1448; `Dom::element` (xmllinq/mod.rs:655) L1426,1427,1454,1457,1459,1460; `Dom::attributes` (xmllinq/mod.rs:899) L1429; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1432; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1433; `Dom::new_element` (xmllinq/mod.rs:478) L1463; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1465; `insert_child_by_rank` (document_comparer.rs:1327) L1467; `Dom::serialize_element` (xmllinq/mod.rs:1315) L1470
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.get`, `.insert`, `.into_iter`, `.is_empty`, `.is_some_and`, `.is_some`, `.map`, `.then`, `.to_string`, `Some`, `W::name`, `W::p_pr`, `W::p`, `W::r_pr`, `crate::namespaces::W14::name`, `std::collections::HashMap::new`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `tests::restored_deleted_spacing_leaves_scratch_unids_behind` (document_comparer.rs:8100); `coverage_boundary_tests::empty_missing_and_non_xml_package_parts_are_safe_noops` (document_comparer.rs:10069); `coverage_boundary_tests::deleted_spacing_restoration_refuses_unmarked_missing_and_already_spaced_paragraphs` (document_comparer.rs:10922)

#### `is_style_prop_noise`  (L1475-1480)
`fn is_style_prop_noise(name: &crate::xmllinq::XName) -> bool`

- **Does:** Children of a style's `w:pPr`/`w:rPr` that are not formatting: revision records, section properties, and revision-save ids.
- **Calls (crate):** `XName::local_name` (xmllinq/mod.rs:208) (by name) L1477
- **Macros:** `matches!`
- **Called by:** `declared_props_signature` (document_comparer.rs:1516); `effective_chain_props` (document_comparer.rs:1696); `complete_from_defaults` (document_comparer.rs:2023); `merge_revised_style_definitions` (document_comparer.rs:2160)

#### `style_prop_signature`  (L1485-1511)
`fn style_prop_signature(dom: &Dom, node: NodeId) -> String`

- **Does:** Order-independent signature of one formatting element: expanded name, its non-`rsid` attributes sorted by name, then its children's signatures in document order (child order carries meaning inside `w:tabs`, `w:pBdr`, …).
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L1486; `XName::clark` (xmllinq/mod.rs:226) (by name) L1489,1497; `Dom::attributes` (xmllinq/mod.rs:899) L1490; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1494; `Dom::is_namespace_declaration` (xmllinq/mod.rs:1012) L1495; `Dom::elements` (xmllinq/mod.rs:640) L1506; `style_prop_signature` (document_comparer.rs:1485) L1508
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.map`, `.push_str`, `.push`, `.sort`, `.starts_with`, `.to_ascii_lowercase`, `String::new`
- **Called by:** `doc_default_ppr_delta` (document_comparer.rs:566); `merge_normal_style_spacing` (document_comparer.rs:777); `style_prop_signature` (document_comparer.rs:1485); `declared_props_signature` (document_comparer.rs:1516); `effective_chain_props` (document_comparer.rs:1696); `coverage_boundary_tests::style_cycles_stop_and_self_based_style_does_not_hide_own_properties` (document_comparer.rs:9059)

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
- **Called by:** `own_style_props` (document_comparer.rs:1578); `inherited_only_style_changes` (document_comparer.rs:1601); `merge_revised_style_definitions` (document_comparer.rs:2160); `coverage_boundary_tests::effective_run_properties_merge_independent_slots_and_clear_alternatives` (document_comparer.rs:9022); `coverage_boundary_tests::style_cycles_stop_and_self_based_style_does_not_hide_own_properties` (document_comparer.rs:9059)

#### `own_style_props`  (L1578-1595)
`fn own_style_props( dom: &Dom, styles_root: NodeId, by_id: &std::collections::HashMap<String, NodeId>, style: NodeId, local: &str, default_local: &str, ) -> std::collections::BTreeMap<String, String>`

- **Does:** What `style` declares beyond its parent: each effective property of [`effective_style_props`] whose value its `basedOn` parent (the docDefaults for a root style) does not already give.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1586; `Dom::attribute` (xmllinq/mod.rs:874) L1588; `effective_chain_props` (document_comparer.rs:1696) L1591; `effective_style_props` (document_comparer.rs:1563) L1592
- **Calls (std/external/unresolved):** `.and_then`, `.copied`, `.filter`, `.get`, `.retain`, `Some`, `W::name`, `W::val`
- **Called by:** `inherited_only_style_changes` (document_comparer.rs:1601); `coverage_boundary_tests::effective_run_properties_merge_independent_slots_and_clear_alternatives` (document_comparer.rs:9022); `coverage_boundary_tests::style_cycles_stop_and_self_based_style_does_not_hide_own_properties` (document_comparer.rs:9059)

#### `inherited_only_style_changes`  (L1601-1667)
`fn inherited_only_style_changes( dom: &Dom, out_root: NodeId, a_root: NodeId, b_root: NodeId, ) -> std::collections::HashSet<(NodeId, &'static str)>`

- **Does:** Freeze the style blocks whose declared difference is only inherited.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1608,1620,1633; `Dom::attribute` (xmllinq/mod.rs:874) L1612; `style_match_key` (document_comparer.rs:1843) L1621,1637; `find_normal_style` (document_comparer.rs:271) L1631; `Dom::element` (xmllinq/mod.rs:655) L1648,1649; `declared_props_signature` (document_comparer.rs:1516) L1652,1653; `effective_style_props` (document_comparer.rs:1563) L1654,1655; `own_style_props` (document_comparer.rs:1578) L1659,1660
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.entry`, `.filter_map`, `.get`, `.insert`, `.into_iter`, `.is_some_and`, `.is_some`, `.or_insert`, `.to_string`, `Some`, `W::name`, `by_id`, `by_key`, `std::collections::HashMap::new`, `std::collections::HashSet::new`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::inherited_only_records_are_suppressed_but_inbound_history_survives` (document_comparer.rs:9082)

#### `remove_inherited_only_style_records`  (L1672-1692)
`fn remove_inherited_only_style_records( dom: &mut Dom, suppressed: &std::collections::HashSet<(NodeId, &'static str)>, ) -> bool`

- **Does:** Remove newly synthesized inherited-only history after live metrics have been normalized.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1683,1685; `Dom::remove` (xmllinq/mod.rs:1118) L1687
- **Calls (std/external/unresolved):** `.and_then`, `W::name`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::inherited_only_records_are_suppressed_but_inbound_history_survives` (document_comparer.rs:9082)

#### `effective_chain_props`  (L1696-1778)
`fn effective_chain_props( dom: &Dom, styles_root: NodeId, by_id: &std::collections::HashMap<String, NodeId>, start: Option<NodeId>, local: &str, default_local: &str, ) -> std::collections::BTreeMap<String, String>`

- **Does:** [`effective_style_props`] from `start` up its `basedOn` chain; `None` resolves the docDefaults alone.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1713,1757,1759,1760,1765; `Dom::attribute` (xmllinq/mod.rs:874) L1715; `Dom::elements` (xmllinq/mod.rs:640) L1726; `Dom::name` (xmllinq/mod.rs:526) L1727; `is_style_prop_noise` (document_comparer.rs:1475) L1728; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1731,1737,1742,1744; `XName::clark` (xmllinq/mod.rs:226) (by name) L1732,1735,1748,1753; `style_prop_signature` (document_comparer.rs:1485) L1732; `Dom::attributes` (xmllinq/mod.rs:899) L1736; `Dom::is_namespace_declaration` (xmllinq/mod.rs:1012) L1738; `XName::namespace` (xmllinq/mod.rs:213) (by name) L1748
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.contains`, `.copied`, `.entry`, `.get`, `.insert`, `.into_iter`, `.join`, `.len`, `.map`, `.name`, `.or_default`, `.push`, `.remove`, `.reverse`, `.starts_with`, `.to_ascii_lowercase`, `Props::new`, `Slots::new`, `Vec::new`, `W::name`, `W::val`, `absorb`, `std::collections::HashSet::new`
- **Macros:** `format!`
- **Called by:** `effective_style_props` (document_comparer.rs:1563); `own_style_props` (document_comparer.rs:1578); `authored_style_chain_boundary_tests::inherited_style_font_alternatives_and_missing_blocks_keep_exact_declarations` (document_comparer.rs:12177); `authored_style_chain_boundary_tests::based_on_cycles_stop_after_unique_authored_owners_with_nearest_properties_winning` (document_comparer.rs:12222); `authored_style_chain_boundary_tests::bounded_style_walk_reads_exactly_the_nearest_thirty_two_owners` (document_comparer.rs:12244)

#### `styles_the_revision_lacks`  (L1790-1810)
`fn styles_the_revision_lacks( dom: &mut Dom, out_root: NodeId, b_root: NodeId, ) -> Vec<((String, String), NodeId)>`

- **Does:** The key a style is matched across the two stylesheets by: `(type, name)`, falling back to the styleId when the style carries no `w:name`.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1795,1800; `style_match_key` (document_comparer.rs:1843) L1798,1803; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1808
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter_map`, `.filter`, `.into_iter`, `.map`, `Some`, `W::name`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::absent_revised_styles_restore_distinct_twins_and_skip_removed_targets` (document_comparer.rs:9261)

#### `restore_styles`  (L1814-1838)
`fn restore_styles( dom: &mut Dom, styles_root: NodeId, saved: Vec<((String, String), NodeId)>, ) -> bool`

- **Does:** Put each saved style back in place of the one its key finds.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1822; `style_match_key` (document_comparer.rs:1843) L1825; `Dom::serialize_element` (xmllinq/mod.rs:1315) L1829; `Dom::replace_with` (xmllinq/mod.rs:1165) L1830
- **Calls (std/external/unresolved):** `.as_ref`, `.contains`, `.find`, `.insert`, `.into_iter`, `Some`, `W::name`, `std::collections::HashSet::new`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::absent_revised_styles_restore_distinct_twins_and_skip_removed_targets` (document_comparer.rs:9261)

#### `style_match_key`  (L1843-1858)
`fn style_match_key(dom: &Dom, style: NodeId) -> Option<(String, String)>`

- **Does:** The key Word pairs two stylesheets' styles by: the type, and the name (the id when it has none).
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L1844,1850,1851; `Dom::element` (xmllinq/mod.rs:655) L1848; `is_built_in` (builtin_styles.rs:394) L1852
- **Calls (std/external/unresolved):** `.and_then`, `.or_else`, `.to_ascii_lowercase`, `.to_string`, `.unwrap_or`, `Some`, `W::name`, `W::val`
- **Called by:** `inherited_only_style_changes` (document_comparer.rs:1601); `styles_the_revision_lacks` (document_comparer.rs:1790); `restore_styles` (document_comparer.rs:1814); `complete_based_style_change_records` (document_comparer.rs:1951); `merge_revised_style_definitions` (document_comparer.rs:2160); `copy_missing_styles_by_name` (document_comparer.rs:3623); `resolve_redefined_style_metrics` (document_comparer.rs:4063); `compare_documents_impl` (document_comparer.rs:6520)

#### `complete_root_style_change_records`  (L1873-1943)
`fn complete_root_style_change_records( dom: &mut Dom, styles_root: NodeId, settings: &WmlComparerSettings, ) -> bool`

- **Does:** Word writes a root style's recorded old properties out in full against the docDefaults (c719b900f0, f1257ca7ea, 221577c35b, 4eff11f045): the old block holds every property the docDefaults set, at the docDefaults value where the original's style leaves it to them, with `w:rFonts`, `w:lang` and `w:spacing` completed att...
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1878,1881,1882,1890,1891,1899...; `Dom::elements` (xmllinq/mod.rs:640) L1883,1894,1924; `Dom::attribute` (xmllinq/mod.rs:874) L1895; `record` (ambiguous: `residual_current_point_contract_tests::record` (convert/metafile.rs:2128); `record` (edit/record.rs:191)) L1902,1910; `Dom::new_element` (xmllinq/mod.rs:478) L1918,1923; `add_child_in_rank_order` (document_comparer.rs:2089) L1919; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1925; `Dom::add` (xmllinq/mod.rs:1074) L1926; `append_style_change_record` (document_comparer.rs:2103) L1928; `complete_from_defaults` (document_comparer.rs:2023) L1939
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.filter`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.unwrap_or`, `Some`, `W::name`, `defaults`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::root_records_gain_missing_companion_and_keep_recorded_original` (document_comparer.rs:9216)

#### `complete_based_style_change_records`  (L1951-2008)
`fn complete_based_style_change_records(dom: &mut Dom, styles_root: NodeId, a_root: NodeId) -> bool`

- **Does:** Complete the old records of each paragraph style with a `w:basedOn` from what the original's docDefaults and chain gave the same style (matched by type and name), so Reject All gives back the original's look rather than Word's built-ins: see [`crate::revision_processor::style_records::complete_from_original_chain`].
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1952,1960,1975; `style_match_key` (document_comparer.rs:1843) L1955,1961,1984; `Dom::element` (xmllinq/mod.rs:655) L1965,1967,1980,1990,1992,1993; `Dom::attribute` (xmllinq/mod.rs:874) L1976; `complete_from_original_chain` (revision_processor/style_records.rs:448) L1997
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.contains`, `.entry`, `.filter_map`, `.get`, `.insert`, `.into_iter`, `.is_none`, `.is_some_and`, `.is_some`, `.or_default`, `.unwrap_or`, `Some`, `W::name`, `std::collections::HashMap::new`
- **Called by:** `stylesheet_completion_matches_type_and_name_and_only_changes_old_records` (document_comparer/based_style_record_tests.rs:417); `stylesheet_completion_ignores_ineligible_styles_and_incomplete_records` (document_comparer/based_style_record_tests.rs:460); `recorded_ancestors_are_matched_in_original_ids_and_tracked_per_block` (document_comparer/based_style_record_tests.rs:494); `compare_documents_impl` (document_comparer.rs:6520)

#### `complete_from_defaults`  (L2023-2050)
`fn complete_from_defaults(dom: &mut Dom, old: NodeId, defaults: NodeId, block_local: &str) -> bool`

- **Does:** Give the recorded `old` block every property of the docDefaults `defaults` block it lacks, and complete its [`RECORD_COMPLETED_PROPS`].
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L2025; `Dom::name` (xmllinq/mod.rs:526) L2026; `is_style_prop_noise` (document_comparer.rs:1475) L2029; `XName::local_name` (xmllinq/mod.rs:208) (by name) L2032; `Dom::element` (xmllinq/mod.rs:655) L2033; `complete_attributes` (document_comparer.rs:2054) L2035; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L2039; `add_rpr_child_in_order` (document_comparer.rs:3309) L2041; `add_child_in_rank_order` (document_comparer.rs:2089) L2043
- **Calls (std/external/unresolved):** `.as_str`, `.contains`, `.to_string`
- **Called by:** `complete_root_style_change_records` (document_comparer.rs:1873); `coverage_boundary_tests::recorded_defaults_complete_slots_without_overwriting_explicit_values` (document_comparer.rs:9155)

#### `complete_attributes`  (L2054-2085)
`fn complete_attributes(dom: &mut Dom, recorded: NodeId, default: NodeId, local: &str) -> bool`

- **Does:** Give `recorded` each attribute of `default` it lacks, unless it names an alternative for that slot (an explicit `w:ascii` over `w:asciiTheme`).
- **Calls (crate):** `Dom::attributes` (xmllinq/mod.rs:899) L2067; `Dom::is_namespace_declaration` (xmllinq/mod.rs:1012) L2068; `Dom::attribute` (xmllinq/mod.rs:874) L2068,2076; `XName::local_name` (xmllinq/mod.rs:208) (by name) L2073; `XName::namespace` (xmllinq/mod.rs:213) (by name) L2076; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L2080
- **Calls (std/external/unresolved):** `.any`, `.as_slice`, `.collect`, `.contains`, `.filter`, `.flat_map`, `.is_some`, `.iter`, `.map`, `.name`, `Some`, `Vec::new`
- **Called by:** `cascade_normal_change_to_based_styles` (document_comparer.rs:1124); `complete_from_defaults` (document_comparer.rs:2023)

#### `add_child_in_rank_order`  (L2089-2099)
`fn add_child_in_rank_order(dom: &mut Dom, parent: NodeId, child: NodeId, rank: fn(&str) -> usize)`

- **Does:** Insert `child` under `parent` after the last existing child that `rank` puts before it, or first.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L2090,2094; `rank` (ambiguous: `wml_order_elements_per_standard::{fn rank}` (comparer/finalize.rs:10492); `add_numbering_child_in_schema_order::{fn rank}` (comparer/footnotes.rs:293)) L2090,2094; `XName::local_name` (xmllinq/mod.rs:208) (by name) L2090,2094; `Dom::elements` (xmllinq/mod.rs:640) L2091; `Dom::add_after_self` (xmllinq/mod.rs:1147) L2096; `Dom::add_first` (xmllinq/mod.rs:1101) L2097
- **Calls (std/external/unresolved):** `.into_iter`, `.is_some_and`, `.map_or`, `.rfind`
- **Called by:** `complete_root_style_change_records` (document_comparer.rs:1873); `complete_from_defaults` (document_comparer.rs:2023)

#### `append_style_change_record`  (L2103-2126)
`fn append_style_change_record( dom: &mut Dom, styles_root: NodeId, block: NodeId, old: NodeId, change_local: &str, settings: &WmlComparerSettings, )`

- **Does:** Wrap `old` (a `w:pPr`/`w:rPr` clone) in a `w:pPrChange`/`w:rPrChange` record and append it to `block`, which is where CT_PPr / CT_RPr put it.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L2111; `style_record_id` (document_comparer.rs:491) L2112; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L2113,2114,2119; `Dom::add` (xmllinq/mod.rs:1074) L2124,2125
- **Calls (std/external/unresolved):** `Some`, `W::name`
- **Called by:** `complete_root_style_change_records` (document_comparer.rs:1873); `merge_revised_style_definitions` (document_comparer.rs:2160); `factory_scaffold_bake_b_styles` (document_comparer.rs:2616); `coverage_boundary_tests::inherited_only_records_are_suppressed_but_inbound_history_survives` (document_comparer.rs:9082); `coverage_boundary_tests::a_styles_two_records_share_one_id` (document_comparer.rs:9118)

#### `merge_revised_style_definitions`  (L2160-2600)
`fn merge_revised_style_definitions( dom: &mut Dom, out_root: NodeId, b_root: NodeId, settings: &WmlComparerSettings, a_declared_keys: &std::collections::HashSet<(String, String)>, inherited_only: &std::collections::HashSet<(NodeId, &'static str)>, ) -> bool`

- **Does:** Workstream S — adopt the REVISED document's definition of every style both documents define, recording the ORIGINAL definition on the `w:style` itself.
- **Calls (crate):** `find_normal_style` (document_comparer.rs:271) L2170,2171; `Dom::elements` (xmllinq/mod.rs:640) L2174,2184,2191,2253,2270,2364...; `Dom::attribute` (xmllinq/mod.rs:874) L2176,2316,2323,2349,2361,2367...; `style_match_key` (document_comparer.rs:1843) L2185,2195,2473; `declared_props_signature` (document_comparer.rs:1516) L2208,2209; `effective_style_props` (document_comparer.rs:1563) L2214,2215; `Dom::element` (xmllinq/mod.rs:655) L2229,2231,2238,2251,2269,2298...; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L2240,2275,2504,2510,2530,2589; `Dom::descendants` (xmllinq/mod.rs:668) L2241,2497,2590; `Dom::remove` (xmllinq/mod.rs:1118) L2242,2259,2591; `Dom::new_element` (xmllinq/mod.rs:478) L2246,2264,2505,2516,2540,2549...; `Dom::name` (xmllinq/mod.rs:526) L2256,2271; `is_style_prop_noise` (document_comparer.rs:1475) L2256,2272; `insert_child_by_rank` (document_comparer.rs:1327) L2265,2517,2562,2576; `Dom::add` (xmllinq/mod.rs:1074) L2276,2511,2533,2542,2551; `append_style_change_record` (document_comparer.rs:2103) L2279,2553,2593; `Dom::attributes` (xmllinq/mod.rs:899) L2307; `Dom::add_before_self` (xmllinq/mod.rs:1130) L2532; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L2541,2550,2581,2585
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.clone`, `.collect`, `.contains`, `.copied`, `.entry`, `.filter_map`, `.filter`, `.first`, `.get`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.map`, `.or_insert`, `.or`, `.push`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `Some`, `Vec::new`, `W14::name`, `W::name`, `W::p_pr`, `W::r_pr`, `W::val`, `attr_map`, `b_chain_has_rpr_elem`, `b_chain_has_sp_attr`, `b_chain_has`, `chain_has_sp_attr`, `dd_ppr_spacing`, `dd_rpr`, `index_by_id`, `lig_or`, `sp_val`, `std::collections::HashMap::new`, `val_or`
- **Macros:** `format!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::revised_style_merge_tracks_both_blocks_without_copying_revised_history` (document_comparer.rs:10854)

#### `factory_scaffold_bake_b_styles`  (L2616-2929)
`fn factory_scaffold_bake_b_styles( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, ) -> bool`

- **Does:** M462 — A has no styles part: rewrite the adopted B stylesheet so its docDefaults become Word's FACTORY defaults, baking each style's B-effective metrics into the style itself.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L2624,2626,2627,2628,2637,2638...; `Dom::attribute` (xmllinq/mod.rs:874) L2630,2642,2651,2698,2707,2713...; `Dom::attributes` (xmllinq/mod.rs:899) L2655; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L2662,2784,2791,2880; `Dom::remove` (xmllinq/mod.rs:1118) L2669,2786,2793; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L2671,2672; `Dom::new_element` (xmllinq/mod.rs:478) L2673,2677,2822,2830,2871,2884...; `Dom::root` (xmllinq/mod.rs:822) L2674,2678; `Dom::add` (xmllinq/mod.rs:1074) L2675,2679,2892; `Dom::add_first` (xmllinq/mod.rs:1101) L2681,2682,2881; `Dom::elements` (xmllinq/mod.rs:640) L2695; `Dom::descendants` (xmllinq/mod.rs:668) L2785,2792,2902; `insert_child_by_rank` (document_comparer.rs:1327) L2823,2831,2872,2908,2918; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L2836,2839,2885,2891; `add_rpr_child_in_order` (document_comparer.rs:3309) L2886; `append_style_change_record` (document_comparer.rs:2103) L2913,2923
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.clone`, `.collect`, `.filter_map`, `.flatten`, `.get`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.map`, `.push`, `.then_some`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `Some`, `Vec::new`, `W14::name`, `W::name`, `W::p_pr`, `W::r_pr`, `W::val`, `b_rv`, `b_sp`, `chain_has_rfonts`, `chain_lig`, `chain_rpr`, `chain_spacing`
- **Macros:** `format!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520)

#### `normalize_word_paragraph_style_line`  (L2948-3038)
`fn normalize_word_paragraph_style_line(dom: &mut Dom, styles_root: NodeId) -> bool`

- **Does:** M79 — Word-mode single-line normalization on paragraph styles.
- **Calls (crate):** `find_normal_style` (document_comparer.rs:271) L2953; `Dom::element` (xmllinq/mod.rs:655) L2954,2955,3000,3009,3018; `Dom::attribute` (xmllinq/mod.rs:874) L2962,2965,2988,2989,2993,3019; `Dom::elements` (xmllinq/mod.rs:640) L2984; `Dom::new_element` (xmllinq/mod.rs:478) L3007,3026; `Dom::add_before_self` (xmllinq/mod.rs:1130) L3010; `Dom::add` (xmllinq/mod.rs:1074) L3012; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L3021,3022,3027,3028,3029; `insert_child_by_rank` (document_comparer.rs:1327) L3033
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.collect`, `.contains`, `.filter`, `.into_iter`, `.is_none`, `.is_some`, `.to_string`, `.unwrap_or`, `Some`, `W::name`, `W::p_pr`, `W::r_pr`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `word_style_line_owned_boundary_tests::word_single_line_style_propagation_keeps_authored_history_and_unrelated_properties` (document_comparer.rs:12808)

#### `align_paragraph_style_fonts_with_normal`  (L3053-3137)
`fn align_paragraph_style_fonts_with_normal(dom: &mut Dom, styles_root: NodeId) -> bool`

- **Does:** M80 — Word-mode paragraph style rFonts alignment with Normal.
- **Calls (crate):** `find_normal_style` (document_comparer.rs:271) L3054; `Dom::element` (xmllinq/mod.rs:655) L3057,3060,3105,3108,3118,3121; `Dom::attribute` (xmllinq/mod.rs:874) L3063,3072,3090,3091,3096,3124; `Dom::elements` (xmllinq/mod.rs:640) L3086; `Dom::new_element` (xmllinq/mod.rs:478) L3111; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L3113,3131,3132; `add_rpr_child_in_order` (document_comparer.rs:3309) L3115
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.contains`, `.filter_map`, `.filter`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some`, `.map`, `.to_string`, `.unwrap_or`, `Some`, `W::name`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520)

#### `has_default_table_style`  (L3156-3168)
`fn has_default_table_style(styles_xml: &str) -> bool`

- **Does:** A style tree's `docDefaults/rPrDefault/rPr` node, if present.
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L3157; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L3158; `Dom::root` (xmllinq/mod.rs:822) L3159; `Dom::elements` (xmllinq/mod.rs:640) L3162; `Dom::attribute` (xmllinq/mod.rs:874) L3165; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L3166
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `Some`, `W::name`
- **Macros:** `matches!`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215); `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::empty_missing_and_non_xml_package_parts_are_safe_noops` (document_comparer.rs:10069)

#### `default_paragraph_line`  (L3170-3208)
`fn default_paragraph_line(styles_xml: &str) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L3171; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L3172; `Dom::root` (xmllinq/mod.rs:822) L3173; `Dom::elements` (xmllinq/mod.rs:640) L3177; `Dom::attribute` (xmllinq/mod.rs:874) L3182,3186,3190,3200; `Dom::element` (xmllinq/mod.rs:655) L3185,3195,3198,3203,3205,3206; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L3191
- **Calls (std/external/unresolved):** `.and_then`, `.copied`, `.find`, `.iter`, `.map`, `.to_string`, `.unwrap_or_else`, `Some`, `W::name`, `W::p_pr`, `W::val`, `line_of`
- **Macros:** `matches!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::empty_missing_and_non_xml_package_parts_are_safe_noops` (document_comparer.rs:10069)

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
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L3314; `Dom::name` (xmllinq/mod.rs:526) L3318; `XName::local_name` (xmllinq/mod.rs:208) (by name) L3321; `Dom::add_after_self` (xmllinq/mod.rs:1147) L3333; `Dom::add_first` (xmllinq/mod.rs:1101) L3334
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.find`, `.into_iter`, `.iter`, `.map`, `.position`, `.rev`, `.unwrap_or`, `Some`
- **Called by:** `cascade_normal_change_to_based_styles` (document_comparer.rs:1124); `complete_from_defaults` (document_comparer.rs:2023); `factory_scaffold_bake_b_styles` (document_comparer.rs:2616); `align_paragraph_style_fonts_with_normal` (document_comparer.rs:3053); `bake_bothsides_dd_disabling_neutralizers` (document_comparer.rs:3909); `resolve_redefined_style_metrics` (document_comparer.rs:4063); `merge_normal_style_rpr` (document_comparer.rs:4343); `tests::add_rpr_child_keeps_sz_in_schema_order_after_position` (document_comparer.rs:8226)

#### `adopt_revised_styles_chrome`  (L3348-3393)
`fn adopt_revised_styles_chrome(out: &mut PartFs, pkg2: &PartFs, out_main: &str)`

- **Does:** M-PAG mechanism 2b / M71: when the output Normal's effective run metrics differ from the REVISED document's, rewrite Normal's rPr to B's effective values with a `w:rPrChange` holding the old rPr.
- **Calls (crate):** `PartFs::part_bytes` (opc/mod.rs:628) L3378,3381; `PartFs::set_part` (opc/mod.rs:661) L3384; `PartFs::add_content_type_override` (opc/mod.rs:845) L3385; `PartFs::read_rels_for` (opc/mod.rs:991) L3386; `PartFs::add_document_relationship` (opc/mod.rs:1003) L3390
- **Calls (std/external/unresolved):** `.any`, `.is_some_and`, `.is_some`, `.iter`, `.map`
- **Macros:** `format!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::revised_chrome_preserves_existing_parts_and_repairs_relationships_idempotently` (document_comparer.rs:10777)

#### `ensure_factory_package_chrome`  (L3402-3506)
`fn ensure_factory_package_chrome(out: &mut PartFs, out_main: &str)`

- **Does:** Minimal Word-like package chrome for thin demo packages (C5).
- **Calls (crate):** `PartFs::part_bytes` (opc/mod.rs:628) L3404,3429,3452; `PartFs::set_part` (opc/mod.rs:661) L3412,3435,3459,3480; `PartFs::add_content_type_override` (opc/mod.rs:845) L3413,3436,3460,3486; `PartFs::read_rels_for` (opc/mod.rs:991) L3417,3440,3464,3490; `PartFs::add_document_relationship` (opc/mod.rs:1003) L3421,3444,3468,3504; `referenced_theme_part` (document_comparer.rs:3511) L3478; `PartFs::remove_relationships_by_type` (opc/mod.rs:904) L3501; `relative_rel_target` (opc/mod.rs:180) L3503
- **Calls (std/external/unresolved):** `.any`, `.as_bytes`, `.clone`, `.collect`, `.ends_with`, `.filter`, `.is_none`, `.is_some_and`, `.iter`, `.map`, `.to_vec`, `.unwrap_or_default`
- **Macros:** `format!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::revised_chrome_preserves_existing_parts_and_repairs_relationships_idempotently` (document_comparer.rs:10777)

#### `referenced_theme_part`  (L3511-3518)
`fn referenced_theme_part(out: &PartFs, main: &str) -> Option<String>`

- **Does:** The theme part `main` references, when the package holds it.
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:991) L3512; `PartFs::resolve_rel_target` (opc/mod.rs:829) L3516; `PartFs::part_bytes` (opc/mod.rs:628) L3517
- **Calls (std/external/unresolved):** `.ends_with`, `.filter`, `.find`, `.is_some`, `.iter`, `.map`
- **Called by:** `ensure_factory_package_chrome` (document_comparer.rs:3402); `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::revised_chrome_preserves_existing_parts_and_repairs_relationships_idempotently` (document_comparer.rs:10777)

#### `word_canonical_style_id`  (L3527-3615)
`fn word_canonical_style_id(name: &str) -> String`

- **Does:** Word-canonical `w:styleId` for a human-readable `w:name` (C3 / C5).
- **Calls (std/external/unresolved):** `.as_str`, `.chars`, `.collect`, `.filter`, `.into`, `.is_alphanumeric`, `.is_empty`, `.map`, `.next`, `.split`, `.to_ascii_lowercase`, `.to_uppercase`, `.trim`, `String::new`
- **Called by:** `canonicalize_style_ids` (document_comparer.rs:3667); `tests::word_canonical_style_id_preserves_toc_builtins` (document_comparer.rs:8326); `tests::word_canonical_style_id_keeps_builtin_ids_that_differ_from_their_names` (document_comparer.rs:8343); `tests::word_canonical_style_id_drops_punctuation` (document_comparer.rs:8370)

#### `copy_missing_styles_by_name`  (L3623-3659)
`fn copy_missing_styles_by_name( dom: &mut Dom, to_root: NodeId, from_root: NodeId, ) -> std::collections::HashMap<String, String>`

- **Does:** Word mode: copy the revised stylesheet's styles that the output lacks, pairing styles by type and name as Word does, not by id.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L3632,3639; `style_match_key` (document_comparer.rs:1843) L3633,3640; `Dom::attribute` (xmllinq/mod.rs:874) L3633,3641; `Dom::new_element` (xmllinq/mod.rs:478) L3652; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L3654; `Dom::add` (xmllinq/mod.rs:1074) L3655; `copy_missing_styles` (comparer/footnotes.rs:267) L3657
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.entry`, `.get`, `.insert`, `.or_insert_with`, `.push`, `.to_string`, `Some`, `Vec::new`, `W::name`, `std::collections::HashMap::new`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520)

#### `canonicalize_style_ids`  (L3667-3733)
`fn canonicalize_style_ids( dom: &mut Dom, styles_root: NodeId, ) -> std::collections::HashMap<String, String>`

- **Does:** Rename `w:styleId` values to Word-canonical ids derived from `w:name`.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L3674; `Dom::attribute` (xmllinq/mod.rs:874) L3679,3684,3709,3725; `Dom::element` (xmllinq/mod.rs:655) L3682; `word_canonical_style_id` (document_comparer.rs:3527) L3689; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L3719,3728; `Dom::descendants` (xmllinq/mod.rs:668) L3724
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_str`, `.clone`, `.collect`, `.contains`, `.get`, `.insert`, `.is_empty`, `.iter`, `.map`, `.push`, `.to_string`, `Some`, `Vec::new`, `W::name`, `W::val`, `std::collections::HashMap::new`, `std::collections::HashSet::new`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::canonical_style_ids_refuse_collisions_and_remap_only_known_references` (document_comparer.rs:9294)

#### `remap_style_refs`  (L3736-3757)
`fn remap_style_refs( dom: &mut Dom, root: NodeId, renames: &std::collections::HashMap<String, String>, ) -> usize`

- **Does:** Apply a styleId rename map to `pStyle` / `rStyle` / `tblStyle` under `root`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L3747; `Dom::attribute` (xmllinq/mod.rs:874) L3748; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L3751
- **Calls (std/external/unresolved):** `.get`, `.is_empty`, `Some`, `W::name`, `W::val`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::canonical_style_ids_refuse_collisions_and_remap_only_known_references` (document_comparer.rs:9294)

#### `materialize_builtin_paragraph_defaults`  (L3764-3781)
`fn materialize_builtin_paragraph_defaults(dom: &mut Dom, root: NodeId) -> bool`

- **Does:** Word reads a docDefaults with no pPrDefault as its built-in paragraph defaults, 160 after and 278 auto lines, while a present empty pPrDefault means single spacing (ceaf5f4).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L3765,3768; `Dom::new_element` (xmllinq/mod.rs:478) L3771,3775,3777; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L3773; `Dom::add` (xmllinq/mod.rs:1074) L3776,3778,3779
- **Calls (std/external/unresolved):** `.is_some`, `Some`, `W::name`, `W::p_pr`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::style_structure_adoption_preserves_existing_defaults_and_empty_explicit_spacing` (document_comparer.rs:9340)

#### `adopt_missing_styles_structure`  (L3786-3806)
`fn adopt_missing_styles_structure(dom: &mut Dom, out_root: NodeId, b_root: NodeId) -> bool`

- **Does:** When the A-based styles part lacks `docDefaults` or `latentStyles`, copy them from B (Word redlines always carry both when B has a full stylesheet).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L3790,3793,3798; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L3796; `Dom::add_before_self` (xmllinq/mod.rs:1130) L3799; `Dom::add_first` (xmllinq/mod.rs:1101) L3801
- **Calls (std/external/unresolved):** `.is_some`, `W::name`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::style_structure_adoption_preserves_existing_defaults_and_empty_explicit_spacing` (document_comparer.rs:9340)

#### `prune_normal_rpr_context_equal_attrs`  (L3822-3887)
`fn prune_normal_rpr_context_equal_attrs(dom: &mut Dom, out_root: NodeId) -> bool`

- **Does:** header/footer→Normal (footer knife-edge line box).
- **Calls (crate):** `find_normal_style` (document_comparer.rs:271) L3823; `Dom::element` (xmllinq/mod.rs:655) L3826,3831,3837,3849,3852,3860...; `rpr_default` (document_comparer.rs:3210) L3829; `Dom::attribute` (xmllinq/mod.rs:874) L3832,3840,3850,3853,3872,3875; `Dom::remove` (xmllinq/mod.rs:1118) L3843,3856,3882; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L3877; `Dom::has_attributes` (xmllinq/mod.rs:867) L3881
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.as_str`, `.is_empty`, `.map`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `Some`, `W14::name`, `W::name`, `W::val`, `dd_val`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520)

#### `bake_bothsides_dd_disabling_neutralizers`  (L3909-4048)
`fn bake_bothsides_dd_disabling_neutralizers( dom: &mut Dom, out_root: NodeId, b_root: NodeId, ) -> bool`

- **Does:** M480b — docDefaults-delta DISABLING neutralizers on both-sides merged styles.
- **Calls (crate):** `rpr_default` (document_comparer.rs:3210) L3917,3918; `Dom::element` (xmllinq/mod.rs:655) L3921,3926,3948,3959,3961,4000...; `Dom::attribute` (xmllinq/mod.rs:874) L3922,3927,3945,3950,3994,4009; `find_normal_style` (document_comparer.rs:271) L3941; `Dom::elements` (xmllinq/mod.rs:640) L3942,3971; `Dom::new_element` (xmllinq/mod.rs:478) L4015,4021,4037; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L4016,4022; `insert_child_by_rank` (document_comparer.rs:1327) L4038; `add_rpr_child_in_order` (document_comparer.rs:3309) L4043; `XName::local_name` (xmllinq/mod.rs:208) (by name) L4043
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.copied`, `.filter_map`, `.get`, `.into_iter`, `.is_none_or`, `.is_some_and`, `.is_some`, `.map`, `.parse`, `.sort_by_key`, `.to_string`, `.unwrap_or`, `Some`, `W14::name`, `W::name`, `W::p_pr`, `W::r_pr`, `W::val`, `based_of`, `chain_declares`, `kern_of`, `lig_of`, `mk`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::disabling_default_neutralizers_preserve_tracked_history_at_inheritance_limit` (document_comparer.rs:10360)

#### `resolve_redefined_style_metrics`  (L4063-4341)
`fn resolve_redefined_style_metrics(dom: &mut Dom, out_root: NodeId, b_root: NodeId) -> bool`

- **Does:** Word's redefined paragraph styles hold B's effective metrics as a delta against the output context.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L4066,4075,4183; `Dom::attribute` (xmllinq/mod.rs:874) L4068,4082,4136,4137,4155,4164...; `coverage_boundary_tests::index` (document_comparer.rs:8844) L4071,4072; `style_match_key` (document_comparer.rs:1843) L4076,4204; `Dom::element` (xmllinq/mod.rs:655) L4081,4102,4103,4104,4109,4116...; `rpr_default` (document_comparer.rs:3210) L4126; `Dom::new_element` (xmllinq/mod.rs:478) L4239,4252,4288,4313,4323; `insert_child_by_rank` (document_comparer.rs:1327) L4240,4314,4324; `Dom::add_first` (xmllinq/mod.rs:1101) L4253; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L4265,4266,4283,4289,4332; `Dom::has_attributes` (xmllinq/mod.rs:867) L4270,4336; `Dom::remove` (xmllinq/mod.rs:1118) L4271,4278,4337; `add_rpr_child_in_order` (document_comparer.rs:3309) L4290
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.as_str`, `.clone`, `.collect`, `.contains`, `.copied`, `.entry`, `.filter_map`, `.find_map`, `.get`, `.into_iter`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.map`, `.or_else`, `.or_insert`, `.push`, `.sort_by_key`, `.then_some`, `.then`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `.zip`, `Some`, `Vec::new`, `W::name`, `W::p_pr`, `W::r_pr`, `W::val`, `chain`, `dd_ppr`, `font_slot`, `holders`, `line_val`, `parent`, `ppr_of`, `rpr_of`, `run_val`, `slot_key`, `spacing_val`, `std::collections::HashMap::new`
- **Macros:** `format!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `redefined_style_chain_source_owner_tests::redefined_style_metrics_bound_cycles_depth_and_default_owners_without_source_history_loss` (document_comparer.rs:12483)

#### `merge_normal_style_rpr`  (L4343-4599)
`fn merge_normal_style_rpr( dom: &mut Dom, out_root: NodeId, b_root: NodeId, settings: &WmlComparerSettings, ) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `find_normal_style` (document_comparer.rs:271) L4349,4352; `Dom::element` (xmllinq/mod.rs:655) L4362,4376,4389,4400,4419,4429...; `effective_normal_rpr_metrics` (document_comparer.rs:3224) L4367,4368; `rpr_default` (document_comparer.rs:3210) L4378,4428,4471,4506; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L4381,4473,4488,4494; `Dom::descendants` (xmllinq/mod.rs:668) L4382; `Dom::remove` (xmllinq/mod.rs:1118) L4383,4433,4476,4571; `Dom::new_element` (xmllinq/mod.rs:478) L4387,4392,4403,4440,4525,4538...; `Dom::add` (xmllinq/mod.rs:1074) L4395,4489,4544,4596,4597; `Dom::add_first` (xmllinq/mod.rs:1101) L4404; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L4409,4410,4445,4526,4539,4576...; `add_rpr_child_in_order` (document_comparer.rs:3309) L4441,4498,4527,4578; `Dom::elements` (xmllinq/mod.rs:640) L4463,4480; `Dom::name` (xmllinq/mod.rs:526) L4464,4482; `XName::local_name` (xmllinq/mod.rs:208) (by name) L4465,4493; `Dom::replace_with` (xmllinq/mod.rs:1165) L4474,4497; `Dom::attribute` (xmllinq/mod.rs:874) L4513,4518,4530,4535,4553; `style_record_id` (document_comparer.rs:491) L4584
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.clone`, `.collect`, `.filter_map`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.map`, `.or_else`, `.or`, `.then_some`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `.zip`, `Some`, `W14::name`, `W::name`, `W::p_pr`, `W::val`, `dd_elem`, `lang_attr`, `metric`, `structured`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `tests::normal_takes_b_theme_fonts_from_its_doc_defaults` (document_comparer.rs:8382); `tests::merge_normals` (document_comparer.rs:8473); `tests::normal_takes_b_run_defaults_when_only_b_paragraph_props_are_stored` (document_comparer.rs:8617); `coverage_boundary_tests::normal_run_adoption_preserves_complete_old_properties_and_revised_slot_ownership` (document_comparer.rs:10978)

#### `normalize_strict_namespaces`  (L4607-4625)
`fn normalize_strict_namespaces(xml: &str) -> std::borrow::Cow<'_, str>`

- **Does:** Merge every direct `w:body` child of `root` into one body and return it.
- **Calls (std/external/unresolved):** `.contains`, `.replace`, `std::borrow::Cow::Borrowed`, `std::borrow::Cow::Owned`
- **Called by:** `main_part_xml` (document_comparer.rs:6486); `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::strict_namespace_normalization_rewrites_only_the_three_ooxml_families` (document_comparer.rs:10147)

#### `merged_body`  (L4630-4659)
`fn merged_body(dom: &mut Dom, root: NodeId) -> Option<NodeId>`

- **Does:** Some producers emit multiple `w:body` elements (invalid, but real — e.g.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L4631; `Dom::nodes` (xmllinq/mod.rs:604) L4640; `Dom::remove` (xmllinq/mod.rs:1118) L4641,4656; `Dom::is_element` (xmllinq/mod.rs:502) L4642; `Dom::name` (xmllinq/mod.rs:526) L4642; `Dom::add` (xmllinq/mod.rs:1074) L4650,4653
- **Calls (std/external/unresolved):** `.as_ref`, `.copied`, `.first`, `.len`, `.push`, `Some`, `Vec::new`, `W::body`, `W::name`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::merged_bodies_preserve_content_order_and_only_last_section_properties` (document_comparer.rs:8946)

#### `wrap_bare_breaks`  (L4666-4682)
`fn wrap_bare_breaks(dom: &mut Dom, root: NodeId)`

- **Does:** Wrap each `w:br` a producer left directly under a paragraph (the schema forbids it) in a run of its own with no properties, `<w:r><w:br/></w:r>`, the way Word reads it.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L4667; `Dom::parent` (xmllinq/mod.rs:597) L4671; `Dom::name` (xmllinq/mod.rs:526) L4672; `Dom::new_element` (xmllinq/mod.rs:478) L4677; `Dom::add_before_self` (xmllinq/mod.rs:1130) L4678; `Dom::remove` (xmllinq/mod.rs:1118) L4679; `Dom::add` (xmllinq/mod.rs:1074) L4680
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.filter`, `.into_iter`, `.is_some_and`, `Some`, `W::name`, `W::p`, `W::r`
- **Called by:** `pre_process_markup` (document_comparer.rs:4713)

#### `defined_style_ids`  (L4685-4697)
`fn defined_style_ids(pkg: &PartFs, main: &str) -> Option<std::collections::HashSet<String>>`

- **Does:** The styleIds the package's stylesheet defines, or `None` without one.
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:991) L4686; `PartFs::resolve_rel_target` (opc/mod.rs:829) L4690; `PartFs::part_string` (opc/mod.rs:636) L4692; `Dom::new` (xmllinq/mod.rs:387) L4693; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L4694; `Dom::root` (xmllinq/mod.rs:822) L4695; `defined_style_ids` (comparer/footnotes.rs:217) L4696
- **Calls (std/external/unresolved):** `.and_then`, `.ends_with`, `.find`, `.is_none`, `.iter`, `.map`, `Some`
- **Called by:** `pre_process_markup` (document_comparer.rs:4713)

#### `pre_process_markup`  (L4713-4894, pub)
`pub fn pre_process_markup( pkg: &mut PartFs, starting_id_for_footnotes_endnotes: i32, ) -> Result<Vec<String>, OpcError>`

- **Does:** C.1/C.2 — `WmlComparer.PreProcessMarkup` (:434) at package level: `ChangeFootnoteEndnoteReferencesToUniqueRange` (:1627) then `AddFootnotesEndnotesParts` (:1604).
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:1070) L4717; `PartFs::read_rels_for` (opc/mod.rs:991) L4724,4861; `PartFs::resolve_rel_target` (opc/mod.rs:829) L4730,4731,4869; `PartFs::part_string` (opc/mod.rs:636) L4737,4740,4741,4874; `Dom::new` (xmllinq/mod.rs:387) L4743,4877; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L4744,4748,4750,4878; `Dom::root` (xmllinq/mod.rs:822) L4745,4749,4751,4879; `Dom::descendants` (xmllinq/mod.rs:668) L4758,4883; `Dom::name` (xmllinq/mod.rs:526) L4761; `change_footnote_endnote_references_to_unique_range` (comparer/footnotes.rs:38) L4766; `defined_style_ids` (document_comparer.rs:4685) L4778; `wrap_bare_breaks` (document_comparer.rs:4666) L4780; `strip_unresolved_style_refs` (comparer/footnotes.rs:233) L4786; `assign_to_all_elements` (unid.rs:60) L4794,4797; `ignore_pt14_namespace` (comparer/finalize.rs:862) L4795,4798; `fill_in_empty_footnotes_endnotes` (comparer/footnotes.rs:133) L4806,4809; `PartFs::set_part` (opc/mod.rs:661) L4813,4816,4820,4842,4890; `Dom::serialize_document` (xmllinq/mod.rs:1338) L4813,4816,4820,4890; `PartFs::add_document_relationship` (opc/mod.rs:1003) L4843; `PartFs::add_content_type_override` (opc/mod.rs:845) L4848; `Dom::remove` (xmllinq/mod.rs:1118) L4888
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.clone`, `.collect`, `.ends_with`, `.filter`, `.flatten`, `.into_bytes`, `.into_iter`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.map_err`, `.map`, `.next`, `.push`, `.rsplit_once`, `.rsplit`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `Ok`, `Some`, `Vec::new`, `W::name`, `crate::namespaces::C::name`
- **Macros:** `format!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::empty_missing_and_non_xml_package_parts_are_safe_noops` (document_comparer.rs:10069); `coverage_boundary_tests::preprocessing_refuses_orphan_notes_before_mutating_package` (document_comparer.rs:10100)

#### `invalid_content`  (L4899-4901)
`fn invalid_content(msg: String) -> OpcError`

- **Does:** Markup the engine refuses (C# throws DocxodusException): an `Err` the caller can handle, never a panic, which would abort a WASM instance.
- **Calls (std/external/unresolved):** `OpcError::Io`, `std::io::Error::new`
- **Called by:** none found in src

#### `admit_input`  (L4907-4920)
`fn admit_input( side: &str, bytes: &[u8], limits: crate::admission::InputLimits, ) -> Result<(), OpcError>`

- **Does:** Admit one compare input under `limits`.
- **Calls (crate):** `admit` (admission.rs:439) L4912; `refused` (opc/mod.rs:76) L4915
- **Calls (std/external/unresolved):** `.map_err`, `.map`
- **Macros:** `format!`
- **Called by:** `admit_package` (document_comparer.rs:4926); `get_revisions` (document_comparer.rs:6293); `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::admission_preserves_typed_refusal_and_identifies_document_side` (document_comparer.rs:10115)

#### `admit_package`  (L4926-4928, pub(crate))
`pub(crate) fn admit_package(bytes: &[u8]) -> Result<(), OpcError>`

- **Does:** Admit a single package under the compare budget before a public entry point opens it: the package constructor is bounded on its own (#645), and admission gives the typed refusal with the rules a budget alone does not check (part-name safety, encryption, XML depth, WML shape).
- **Calls (crate):** `admit_input` (document_comparer.rs:4907) L4927; `InputLimits::compare` (admission.rs:87) L4927
- **Called by:** `open` (changes.rs:1031); `with_layout` (convert/mod.rs:769); `accept_revisions` (document_comparer.rs:4961); `reject_revisions` (document_comparer.rs:4984); `coverage_boundary_tests::admission_preserves_typed_refusal_and_identifies_document_side` (document_comparer.rs:10115)

#### `accept_revisions`  (L4961-4966, pub)
`pub fn accept_revisions(docx: &[u8]) -> Result<Vec<u8>, OpcError>`

- **Does:** A.11 — `RevisionProcessor.AcceptRevisions` byte facade: accept every tracked revision across main + headers/footers + notes + comments + styles parts and return the result as a `.docx` byte vector.
- **Calls (crate):** `admit_package` (document_comparer.rs:4926) L4962; `PartFs::open` (opc/mod.rs:585) L4963; `accept_revisions_package` (revision_processor.rs:3752) L4964; `PartFs::to_zip` (opc/mod.rs:749) L4965
- **Called by:** `run_convert_any` (bin/jubarte.rs:3458); `public_option_source_ownership_tests::public_boundary_shifts_preserve_all_source_owners_with_lexical_options` (comparer/cross_para.rs:2702); `public_option_source_ownership_tests::public_comparison_options_preserve_reordered_paragraphs_formats_and_note_owners` (comparer/cross_para.rs:2772); `public_option_source_ownership_tests::public_reordered_bulk_paragraphs_preserve_sources_on_move_thrash_boundaries` (comparer/cross_para.rs:2935); `public_option_source_ownership_tests::strict_and_transitional_packages_preserve_complete_authored_body_ownership` (comparer/cross_para.rs:3031); `public_option_source_ownership_tests::zero_word_move_threshold_preserves_authored_nontext_and_separator_payloads` (comparer/cross_para.rs:3178); `public_option_source_ownership_tests::public_compound_and_one_sided_paragraph_windows_preserve_complete_sources` (comparer/cross_para.rs:3325); `row_nontext_ownership_regressions::public_compare_keeps_unchanged_tab_and_break_cells_when_text_moves_into_their_row` (comparer/finalize.rs:23895); `coverage_real_revision_merge_matrix_tests::public_compare_multi_original_blank_tail_recovers_complete_clean_sources` (comparer/finalize.rs:26112); `coverage_real_revision_merge_matrix_tests::whole_story_title_label_heading_and_list_boundaries_keep_source_ownership` (comparer/finalize.rs:27055); `coverage_real_revision_merge_matrix_tests::public_terminal_field_math_and_break_boundaries_preserve_all_source_owners` (comparer/finalize.rs:27505); `coverage_real_revision_merge_matrix_tests::public_cross_paragraph_field_ends_keep_exact_source_instruction_and_cache_owners` (comparer/finalize.rs:27727); +24 more

#### `accept_source_revisions`  (L4972-4979)
`fn accept_source_revisions( docx: &[u8], limits: crate::admission::InputLimits, ) -> Result<Vec<u8>, OpcError>`

- **Does:** [`accept_revisions`] under the caller's admission budget, which the compare flow already checked (#645).
- **Calls (crate):** `PartFs::open_within` (opc/mod.rs:593) L4976; `accept_revisions_package` (revision_processor.rs:3752) L4977; `PartFs::to_zip` (opc/mod.rs:749) L4978
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_package_source_matrix_tests::accepted_source_revision_converges_to_identical_final_before_comparison` (document_comparer.rs:11932)

#### `reject_revisions`  (L4984-4989, pub)
`pub fn reject_revisions(docx: &[u8]) -> Result<Vec<u8>, OpcError>`

- **Does:** A.11 — `RevisionProcessor.RejectRevisions` byte facade: reject every tracked revision across main + headers/footers + notes + comments + styles parts.
- **Calls (crate):** `admit_package` (document_comparer.rs:4926) L4985; `PartFs::open` (opc/mod.rs:585) L4986; `reject_revisions_package` (revision_processor.rs:3761) L4987; `PartFs::to_zip` (opc/mod.rs:749) L4988
- **Called by:** `run_convert_any` (bin/jubarte.rs:3458); `public_option_source_ownership_tests::public_boundary_shifts_preserve_all_source_owners_with_lexical_options` (comparer/cross_para.rs:2702); `public_option_source_ownership_tests::public_comparison_options_preserve_reordered_paragraphs_formats_and_note_owners` (comparer/cross_para.rs:2772); `public_option_source_ownership_tests::public_reordered_bulk_paragraphs_preserve_sources_on_move_thrash_boundaries` (comparer/cross_para.rs:2935); `public_option_source_ownership_tests::strict_and_transitional_packages_preserve_complete_authored_body_ownership` (comparer/cross_para.rs:3031); `public_option_source_ownership_tests::zero_word_move_threshold_preserves_authored_nontext_and_separator_payloads` (comparer/cross_para.rs:3178); `public_option_source_ownership_tests::public_compound_and_one_sided_paragraph_windows_preserve_complete_sources` (comparer/cross_para.rs:3325); `row_nontext_ownership_regressions::public_compare_keeps_unchanged_tab_and_break_cells_when_text_moves_into_their_row` (comparer/finalize.rs:23895); `coverage_real_revision_merge_matrix_tests::public_compare_multi_original_blank_tail_recovers_complete_clean_sources` (comparer/finalize.rs:26112); `coverage_real_revision_merge_matrix_tests::whole_story_title_label_heading_and_list_boundaries_keep_source_ownership` (comparer/finalize.rs:27055); `coverage_real_revision_merge_matrix_tests::public_terminal_field_math_and_break_boundaries_preserve_all_source_owners` (comparer/finalize.rs:27505); `coverage_real_revision_merge_matrix_tests::public_cross_paragraph_field_ends_keep_exact_source_instruction_and_cache_owners` (comparer/finalize.rs:27727); +23 more

#### `unique_part_name`  (L4998-5000)
`fn unique_part_name(out: &PartFs, want: &str, bytes: &[u8]) -> String`

- **Does:** Collision-proof target name for copying `want` (with `bytes`) into `out`: free name or byte-identical existing part → `want` unchanged; an existing part with DIFFERENT content → the first free or identical candidate: a header or footer (`header1.xml`) takes the next number of its kind (`header3.xml`), a name the short...
- **Calls (crate):** `unique_part_name_where` (document_comparer.rs:5005) L4999
- **Called by:** `adopt_revised_header_footer` (document_comparer.rs:5174); `adopt_b_notes_when_a_lacks_separators` (document_comparer.rs:5871); `coverage_boundary_tests::a_colliding_header_or_footer_takes_the_first_free_number` (document_comparer.rs:9749); `coverage_boundary_tests::unique_part_names_handle_binary_collisions_reuse_and_flat_names` (document_comparer.rs:9775)

#### `unique_part_name_where`  (L5005-5048)
`fn unique_part_name_where( out: &PartFs, want: &str, bytes: &[u8], same: &dyn Fn(&str) -> bool, ) -> String`

- **Does:** [`unique_part_name`], reusing an identical part only when `same` also holds for it (a header's relationships, so identical XML that shows another image is another part).
- **Calls (crate):** `PartFs::part_bytes` (opc/mod.rs:628) L5011,5024,5040; `detach_breaks::{fn same}` (markdown/unified.rs:876) (by name) L5013,5026,5042
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.bytes`, `.find`, `.into_iter`, `.is_ascii_digit`, `.is_empty`, `.is_some_and`, `.rsplit_once`, `.strip_prefix`, `.strip_suffix`, `.to_string`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `unique_part_name` (document_comparer.rs:4998); `adopt_revised_header_footer` (document_comparer.rs:5174)

#### `strip_final_sectpr_inherited_header_footer`  (L5061-5123)
`fn strip_final_sectpr_inherited_header_footer( dom: &mut Dom, result_root: NodeId, revised_final: &std::collections::HashSet<(bool, String)>, )`

- **Does:** Word Compare leaves the body-level final `sectPr` without headerReference/footerReference when an earlier mid-body section break already defines the same (kind, type) slot — later sections inherit.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L5069,5073; `Dom::descendants` (xmllinq/mod.rs:668) L5078; `Dom::elements` (xmllinq/mod.rs:640) L5082,5102; `Dom::name` (xmllinq/mod.rs:526) L5083,5103; `Dom::attribute` (xmllinq/mod.rs:874) L5091,5111; `Dom::remove` (xmllinq/mod.rs:1118) L5121
- **Calls (std/external/unresolved):** `.contains`, `.insert`, `.is_empty`, `.push`, `.to_string`, `.unwrap_or`, `Some`, `Vec::new`, `W::body`, `W::name`, `std::collections::HashSet::new`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::final_header_slots_strip_only_inherited_non_revised_references` (document_comparer.rs:9662)

#### `final_header_footer_slots`  (L5127-5158)
`fn final_header_footer_slots(pkg: &PartFs) -> std::collections::HashSet<(bool, String)>`

- **Does:** The (is header, `w:type`) header/footer slots the body-level final `sectPr` of `pkg`'s main document sets explicitly.
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:1070) L5128; `PartFs::part_string` (opc/mod.rs:636) L5131; `Dom::new` (xmllinq/mod.rs:387) L5134; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L5135; `Dom::root` (xmllinq/mod.rs:822) L5136; `Dom::element` (xmllinq/mod.rs:655) L5138,5139; `Dom::elements` (xmllinq/mod.rs:640) L5143; `Dom::name` (xmllinq/mod.rs:526) L5146; `Dom::attribute` (xmllinq/mod.rs:874) L5154
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.filter_map`, `.into_iter`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `Default::default`, `Some`, `W::body`, `W::name`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::final_header_slots_strip_only_inherited_non_revised_references` (document_comparer.rs:9662); `coverage_boundary_tests::empty_missing_and_non_xml_package_parts_are_safe_noops` (document_comparer.rs:10069)

#### `adopt_revised_header_footer`  (L5174-5446)
`fn adopt_revised_header_footer( dom: &mut Dom, result_root: NodeId, pkg2: &PartFs, out: &mut PartFs, out_main: &str, settings: &WmlComparerSettings, )`

- **Does:** Word-alignment mode (settings-gated): Word's Compare presents the REVISED document's headers/footers as a UNION per (kind, `w:type`) slot.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L5184,5187,5258,5261; `Dom::descendants` (xmllinq/mod.rs:668) L5192,5263; `Dom::elements` (xmllinq/mod.rs:640) L5194,5213,5277; `Dom::name` (xmllinq/mod.rs:526) L5196,5217,5280,5287; `Dom::attribute` (xmllinq/mod.rs:874) L5204,5225,5288,5297; `PartFs::main_document_part` (opc/mod.rs:1070) L5247; `PartFs::part_string` (opc/mod.rs:636) L5250; `Dom::new` (xmllinq/mod.rs:387) L5253; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L5254; `Dom::root` (xmllinq/mod.rs:822) L5255; `PartFs::read_rels_for` (opc/mod.rs:991) L5267,5359; `PartFs::resolve_rel_target` (opc/mod.rs:829) L5303,5372; `PartFs::part_bytes` (opc/mod.rs:628) L5304,5339,5373,5375; `PartFs::parts` (opc/mod.rs:733) L5313; `PartFs::read_rels_for` (opc/mod.rs:991) (by name) L5325; `rels_of` (debug.rs:845) (by name) L5332,5334; `unique_part_name_where` (document_comparer.rs:5005) L5333; `PartFs::set_part` (opc/mod.rs:661) L5341,5376,5424; `mark_adopted_hf_content_as_inserted` (document_comparer.rs:5452) L5345; `PartFs::add_content_type_override` (opc/mod.rs:845) L5352; `unique_part_name` (document_comparer.rs:4998) L5374; `relative_rel_target` (opc/mod.rs:180) L5381,5435; `PartFs::add_content_type_default` (opc/mod.rs:852) L5399; `PartFs::add_document_relationship` (opc/mod.rs:1003) L5436; `Dom::new_element` (xmllinq/mod.rs:478) L5437; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L5442,5443; `Dom::add_first` (xmllinq/mod.rs:1101) L5444
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.as_str`, `.clone`, `.collect`, `.contains_key`, `.contains`, `.copied`, `.ends_with`, `.filter_map`, `.filter`, `.flat_map`, `.get`, `.insert`, `.into_bytes`, `.into_iter`, `.is_none`, `.is_some_and`, `.iter`, `.last`, `.map`, `.next`, `.or_else`, `.push_str`, `.replace`, `.rsplit_once`, `.rsplit`, `.to_ascii_lowercase`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `.values`, `R::name`, `Some`, `String::from`, `W::body`, `W::name`, `body_slots`, `std::collections::HashMap::new`, `xe`
- **Macros:** `format!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::final_header_footer_adoption_respects_existing_body_owners_and_cached_donor_parts` (document_comparer.rs:10165); `coverage_boundary_tests::header_adoption_preserves_colliding_media_and_uses_correct_content_types` (document_comparer.rs:10516); `coverage_boundary_tests::a_renamed_header_copy_never_hides_another_header_of_the_revision` (document_comparer.rs:10615); `coverage_boundary_tests::a_reused_header_keeps_its_own_relationships` (document_comparer.rs:10668); `coverage_boundary_tests::header_adoption_refuses_unresolvable_references_and_missing_story_structure` (document_comparer.rs:10706); `adopted_story_relationship_owner_tests::adopted_root_and_nested_stories_keep_colliding_media_relationships_resolvable` (document_comparer.rs:12648)

#### `mark_adopted_hf_content_as_inserted`  (L5452-5564)
`fn mark_adopted_hf_content_as_inserted( out: &mut PartFs, part: &str, settings: &WmlComparerSettings, )`

- **Does:** M378 — wrap body content of a newly adopted B-only header/footer as pure-I.
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L5457; `Dom::new` (xmllinq/mod.rs:387) L5460; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L5461; `Dom::root` (xmllinq/mod.rs:822) L5462; `next_free_revision_id` (document_comparer.rs:469) L5466; `Dom::descendants` (xmllinq/mod.rs:668) L5469,5472,5473; `Dom::elements` (xmllinq/mod.rs:640) L5477,5547; `Dom::name` (xmllinq/mod.rs:526) L5480,5487; `XName::local_name` (xmllinq/mod.rs:208) (by name) L5495; `Dom::new_element` (xmllinq/mod.rs:478) L5501,5529,5540; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L5502,5503,5506,5541,5542,5545; `Dom::parent` (xmllinq/mod.rs:597) L5509,5518; `Dom::add_before_self` (xmllinq/mod.rs:1130) L5510,5532,5548; `Dom::add` (xmllinq/mod.rs:1074) L5512,5515,5522,5534,5550; `Dom::remove` (xmllinq/mod.rs:1118) L5521; `Dom::element` (xmllinq/mod.rs:655) L5525,5526,5531,5539; `mark_story_rows` (document_comparer.rs:5755) L5555; `Dom::new_element` (xmllinq/mod.rs:478) (by name) L5556; `Dom::set_attribute_value` (xmllinq/mod.rs:977) (by name) L5557,5558,5559; `PartFs::set_part` (opc/mod.rs:661) L5563; `Dom::serialize_element` (xmllinq/mod.rs:1315) L5563
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.collect`, `.copied`, `.filter`, `.first`, `.into_bytes`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.to_string`, `Some`, `W::author`, `W::date`, `W::del`, `W::hyperlink`, `W::id`, `W::ins`, `W::name`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`
- **Called by:** `adopt_revised_header_footer` (document_comparer.rs:5174); `coverage_boundary_tests::adopted_header_marks_content_and_paragraph_properties_with_pinned_provenance` (document_comparer.rs:9388); `coverage_boundary_tests::one_sided_header_footer_tables_mark_every_row` (document_comparer.rs:9465); `coverage_boundary_tests::one_sided_story_marks_take_ids_above_the_story_s_own` (document_comparer.rs:9503); `coverage_boundary_tests::one_sided_story_fields_join_the_story_revision` (document_comparer.rs:9531); `coverage_boundary_tests::empty_missing_and_non_xml_package_parts_are_safe_noops` (document_comparer.rs:10069); `coverage_boundary_tests::header_insertion_wraps_supported_owned_payload_and_retains_noncontent_and_foreign_revisions` (document_comparer.rs:11191); `isolated_header_container_revision_boundary_tests::pure_header_container_payloads_recover_every_authored_property_and_fallback` (document_comparer.rs:12321)

#### `mark_a_only_hf_content_as_deleted`  (L5571-5587)
`fn mark_a_only_hf_content_as_deleted( out: &mut PartFs, pkg1: &PartFs, pkg2: &PartFs, settings: &WmlComparerSettings, )`

- **Does:** M383 — when the revised document has **no** headers/footers but the original does, Word marks all original HF content pure-D.
- **Calls (crate):** `header_footer_refs` (document_comparer.rs:116) L5577,5579; `mark_hf_part_content_as_deleted` (document_comparer.rs:5666) L5585
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.insert`, `.iter`, `std::collections::HashSet::new`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::original_footer_deletion_is_per_kind_and_duplicate_references_are_processed_once` (document_comparer.rs:10741)

#### `delete_story_closing_mark`  (L5593-5655)
`fn delete_story_closing_mark(dom: &mut Dom, story: NodeId, settings: &WmlComparerSettings)`

- **Does:** Word's redline of a header/footer whose revised story ends with a table: the original's closing paragraph, emptied by the diff, loses its mark, since the revised story needs none, and keeps its own properties unrecorded (bc0135eaa1).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L5594,5628,5633; `Dom::name_is` (xmllinq/mod.rs:535) L5603; `Dom::descendants` (xmllinq/mod.rs:668) L5606; `Dom::element` (xmllinq/mod.rs:655) L5610,5618,5620,5625,5626; `Dom::new_element` (xmllinq/mod.rs:478) L5613,5644,5649; `Dom::add_first` (xmllinq/mod.rs:1101) L5614,5654; `Dom::remove` (xmllinq/mod.rs:1118) L5627,5630; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L5634; `Dom::add_before_self` (xmllinq/mod.rs:1130) L5636; `Dom::add` (xmllinq/mod.rs:1074) L5637,5645; `next_free_revision_id` (document_comparer.rs:469) L5650; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L5651,5652,5653
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.last`, `.map`, `.to_string`, `.unwrap_or_default`, `Some`, `W::author`, `W::date`, `W::del`, `W::id`, `W::ins`, `W::name`, `W::p_pr`, `W::p`, `W::r_pr`, `W::t`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::story_closing_mark_restores_original_props_and_does_not_double_mark` (document_comparer.rs:9579); `coverage_boundary_tests::closing_mark_refuses_live_content_and_handles_empty_or_unformatted_stories` (document_comparer.rs:9604)

#### `mark_hf_part_content_as_deleted`  (L5666-5748)
`fn mark_hf_part_content_as_deleted(out: &mut PartFs, part: &str, settings: &WmlComparerSettings)`

- **Does:** Word's redline of a header/footer the revised document drops: the content goes pure-D and every paragraph mark but the story's last is deleted (a paragraph without `pPr` included).
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L5667; `Dom::new` (xmllinq/mod.rs:387) L5670; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L5671; `Dom::root` (xmllinq/mod.rs:822) L5672; `next_free_revision_id` (document_comparer.rs:469) L5675; `Dom::new_element` (xmllinq/mod.rs:478) L5679; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L5680,5681,5682; `Dom::name` (xmllinq/mod.rs:526) L5686,5702,5705,5728; `Dom::elements` (xmllinq/mod.rs:640) L5691,5699,5729,5739; `Dom::descendants` (xmllinq/mod.rs:668) L5692,5694,5695,5721; `XName::local_name` (xmllinq/mod.rs:208) (by name) L5713; `revision` (ambiguous, 5 candidates) L5716,5738,5746; `Dom::add_before_self` (xmllinq/mod.rs:1130) L5717,5740; `Dom::remove` (xmllinq/mod.rs:1118) L5719; `Dom::set_name` (xmllinq/mod.rs:539) L5722; `Dom::add` (xmllinq/mod.rs:1074) L5724,5741; `Dom::parent` (xmllinq/mod.rs:597) L5727; `reset_to_blank_paragraph` (document_comparer.rs:5769) L5732; `child_or_first` (document_comparer.rs:5837) L5735; `child_before` (document_comparer.rs:5851) L5736; `Dom::element` (xmllinq/mod.rs:655) L5737; `mark_story_rows` (document_comparer.rs:5755) L5746; `PartFs::set_part` (opc/mod.rs:661) L5747; `Dom::serialize_element` (xmllinq/mod.rs:1315) L5747
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.collect`, `.copied`, `.filter`, `.first`, `.into_bytes`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.iter`, `.last`, `.to_string`, `Some`, `W::author`, `W::date`, `W::del`, `W::hyperlink`, `W::id`, `W::ins`, `W::name`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `W::t`
- **Called by:** `mark_a_only_hf_content_as_deleted` (document_comparer.rs:5571); `coverage_boundary_tests::deleted_footer_preserves_last_mark_and_records_original_properties` (document_comparer.rs:9414); `coverage_boundary_tests::one_sided_header_footer_tables_mark_every_row` (document_comparer.rs:9465); `coverage_boundary_tests::one_sided_story_marks_take_ids_above_the_story_s_own` (document_comparer.rs:9503); `coverage_boundary_tests::one_sided_story_fields_join_the_story_revision` (document_comparer.rs:9531); `coverage_boundary_tests::deletion_skips_inbound_changes_and_preserves_existing_blank_history` (document_comparer.rs:9560); `coverage_boundary_tests::empty_missing_and_non_xml_package_parts_are_safe_noops` (document_comparer.rs:10069); `isolated_header_container_revision_boundary_tests::pure_header_container_payloads_recover_every_authored_property_and_fallback` (document_comparer.rs:12321)

#### `mark_story_rows`  (L5755-5764)
`fn mark_story_rows(dom: &mut Dom, story: NodeId, mut mark: impl FnMut(&mut Dom) -> NodeId)`

- **Does:** Mark every table row of a one-sided header/footer story with the story's revision (`w:del` dropped, `w:ins` adopted), as Word's redline does.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L5756; `row_properties` (comparer/lcs_table.rs:763) L5757; `Dom::element` (xmllinq/mod.rs:655) L5758; `existing_mark_layout_ownership_tests::mark` (comparer/finalize.rs:33009) L5761; `add_row_mark` (comparer/lcs_table.rs:778) L5762
- **Calls (std/external/unresolved):** `.is_some`, `Some`, `W::del`, `W::ins`, `W::name`
- **Called by:** `mark_adopted_hf_content_as_inserted` (document_comparer.rs:5452); `mark_hf_part_content_as_deleted` (document_comparer.rs:5666)

#### `reset_to_blank_paragraph`  (L5769-5834)
`fn reset_to_blank_paragraph( dom: &mut Dom, p: NodeId, own_style: &str, revision: &mut impl FnMut(&mut Dom, XName) -> NodeId, )`

- **Does:** Reset the story's last paragraph to the blank one Word's redline keeps: no properties but the kind's own style, the old paragraph properties in a `pPrChange` and the old mark formatting in an `rPrChange`.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L5775,5790,5810,5823; `Dom::name` (xmllinq/mod.rs:526) L5779,5815; `XName::local_name` (xmllinq/mod.rs:208) (by name) L5782,5797,5817; `Dom::attribute` (xmllinq/mod.rs:874) L5783; `Dom::elements` (xmllinq/mod.rs:640) L5785,5795,5811; `revision` (ambiguous, 5 candidates) L5791,5824; `Dom::new_element` (xmllinq/mod.rs:478) L5792,5825; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L5800; `Dom::add` (xmllinq/mod.rs:1074) L5801,5807,5808,5828,5830,5831; `Dom::remove` (xmllinq/mod.rs:1118) L5805,5827
- **Calls (std/external/unresolved):** `.as_deref`, `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.map`, `.name`, `.to_string`, `Some`, `W::name`, `W::p_pr`, `W::r_pr`, `W::val`, `keep`
- **Macros:** `matches!`
- **Called by:** `mark_hf_part_content_as_deleted` (document_comparer.rs:5666)

#### `child_or_first`  (L5837-5847)
`fn child_or_first(dom: &mut Dom, parent: NodeId, name: XName) -> NodeId`

- **Does:** `parent`'s `name` child, created as its first child when absent.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L5838; `Dom::new_element` (xmllinq/mod.rs:478) L5841; `Dom::elements` (xmllinq/mod.rs:640) L5842; `Dom::add_before_self` (xmllinq/mod.rs:1130) L5843; `Dom::add` (xmllinq/mod.rs:1074) L5844
- **Calls (std/external/unresolved):** `.copied`, `.first`
- **Called by:** `mark_hf_part_content_as_deleted` (document_comparer.rs:5666); `coverage_boundary_tests::child_creation_places_properties_before_schema_successors_and_reuses_nodes` (document_comparer.rs:9644)

#### `child_before`  (L5851-5865)
`fn child_before(dom: &mut Dom, parent: NodeId, name: XName, later: &[&str]) -> NodeId`

- **Does:** `parent`'s `name` child, created before the first of `later` (schema order) or last when none is present.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L5852; `Dom::new_element` (xmllinq/mod.rs:478) L5855; `Dom::elements` (xmllinq/mod.rs:640) L5856; `Dom::name` (xmllinq/mod.rs:526) L5857; `XName::local_name` (xmllinq/mod.rs:208) (by name) L5858; `Dom::add_before_self` (xmllinq/mod.rs:1130) L5861; `Dom::add` (xmllinq/mod.rs:1074) L5862
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.into_iter`, `.is_some_and`
- **Called by:** `mark_hf_part_content_as_deleted` (document_comparer.rs:5666); `coverage_boundary_tests::child_creation_places_properties_before_schema_successors_and_reuses_nodes` (document_comparer.rs:9644)

#### `adopt_b_notes_when_a_lacks_separators`  (L5871-6001)
`fn adopt_b_notes_when_a_lacks_separators( out: &mut PartFs, pkg1: &PartFs, pkg2: &PartFs, out_main: &str, )`

- **Does:** M379 — when the original has no real footnotes/endnotes separators but the revised document does, copy B's notes part into the package.
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L5889,5895,5902,5913; `Dom::new` (xmllinq/mod.rs:387) L5914; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L5915,5916; `Dom::root` (xmllinq/mod.rs:822) L5917; `Dom::elements` (xmllinq/mod.rs:640) L5923,5928; `is_structural_note` (comparer/footnotes.rs:787) L5926; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L5930; `Dom::add_before_self` (xmllinq/mod.rs:1130) L5932; `Dom::add` (xmllinq/mod.rs:1074) L5933; `PartFs::set_part` (opc/mod.rs:661) L5936,5940,5976,5998; `Dom::serialize_element` (xmllinq/mod.rs:1315) L5936; `PartFs::add_content_type_override` (opc/mod.rs:845) L5941; `PartFs::read_rels_for` (opc/mod.rs:991) L5943,5959; `relative_rel_target` (opc/mod.rs:180) L5949,5979; `PartFs::add_document_relationship` (opc/mod.rs:1003) L5950; `PartFs::resolve_rel_target` (opc/mod.rs:829) L5972; `PartFs::part_bytes` (opc/mod.rs:628) L5973,5975; `unique_part_name` (document_comparer.rs:4998) L5974
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.clone`, `.collect`, `.contains`, `.copied`, `.ends_with`, `.filter`, `.first`, `.into_bytes`, `.into_iter`, `.is_none`, `.is_some_and`, `.iter`, `.map`, `.next`, `.push_str`, `.replace`, `.rsplit`, `.unwrap_or_default`, `.unwrap_or`, `Some`, `String::from`, `crate::namespaces::W::endnote`, `crate::namespaces::W::footnote`, `xe`
- **Macros:** `format!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::notes_merge_only_structural_definitions_ahead_of_rectified_content` (document_comparer.rs:9897); `coverage_boundary_tests::notes_copy_preserves_media_and_escapes_external_links_without_io` (document_comparer.rs:9938); `coverage_boundary_tests::notes_adoption_refuses_missing_separator_and_preserves_original_separator` (document_comparer.rs:9978); `coverage_boundary_tests::separator_media_collisions_keep_a_resolvable_relative_relationship` (document_comparer.rs:10945)

#### `reresolve_theme_color_hexes`  (L6012-6168)
`fn reresolve_theme_color_hexes(dom: &mut Dom, styles_root: NodeId, theme_xml: &str) -> bool`

- **Does:** M483 — re-cache `w:color` hex values against the package's theme.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L6127; `Dom::attribute` (xmllinq/mod.rs:874) L6129,6141,6144,6146; `Dom::name` (xmllinq/mod.rs:526) L6136; `apply` (ambiguous: `tests::apply` (edit/rewrite.rs:225); `apply` (markdown/from_docx/header.rs:816); `apply` (markdown/view_styles.rs:586)) L6145,6147; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L6162
- **Calls (std/external/unresolved):** `.abs_diff`, `.abs`, `.all`, `.as_ref`, `.clamp`, `.clone`, `.collect`, `.find`, `.floor`, `.iter`, `.len`, `.map_or`, `.map`, `.max`, `.min`, `.ok`, `.to_uppercase`, `.unwrap_or`, `Some`, `W::name`, `W::val`, `hue`, `slot_hex`, `slot_of`, `u32::from_str_radix`
- **Macros:** `format!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::theme_cache_recaching_preserves_independent_fill_color_and_tolerated_source_hexes` (document_comparer.rs:11076)

#### `custom_props_part`  (L6181-6188)
`fn custom_props_part(pkg: &PartFs) -> Option<String>`

- **Does:** The part the package's custom-properties relationship reaches.
- **Calls (crate):** `PartFs::package_relationships` (opc/mod.rs:1013) L6182; `PartFs::part_bytes` (opc/mod.rs:628) L6187
- **Calls (std/external/unresolved):** `.filter`, `.find`, `.is_some`, `.iter`, `.map`, `.to_string`, `.trim_start_matches`
- **Called by:** `merge_custom_properties` (document_comparer.rs:6196); `coverage_boundary_tests::custom_property_adoption_refuses_unrelated_collision_and_malformed_xml` (document_comparer.rs:10034)

#### `merge_custom_properties`  (L6196-6256)
`fn merge_custom_properties(out: &mut PartFs, pkg2: &PartFs)`

- **Does:** Word's redline keeps the custom document properties of both sides: the revision's alone when the original has none (83 of the 96 Word redlines where only the revision has them), else the union, the original's value winning where both name a property (every conflict among the 47 with both).
- **Calls (crate):** `custom_props_part` (document_comparer.rs:6181) L6197,6203; `PartFs::part_string` (opc/mod.rs:636) L6200,6215; `PartFs::part_bytes` (opc/mod.rs:628) L6204; `PartFs::set_part` (opc/mod.rs:661) L6207,6254; `PartFs::add_content_type_override` (opc/mod.rs:845) L6208; `PartFs::add_package_relationship` (opc/mod.rs:1018) L6212; `Dom::new` (xmllinq/mod.rs:387) L6218; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L6219,6220; `Dom::root` (xmllinq/mod.rs:822) L6221; `XName::get` (xmllinq/mod.rs:180) L6224,6225; `Dom::elements` (xmllinq/mod.rs:640) L6226,6240; `Dom::attribute` (xmllinq/mod.rs:874) L6229,6237,6241; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L6247; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L6249; `Dom::add` (xmllinq/mod.rs:1074) L6250; `Dom::serialize_document` (xmllinq/mod.rs:1338) L6254
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.insert`, `.into_bytes`, `.is_some`, `.iter`, `.map`, `.max`, `.ok`, `.parse`, `.to_lowercase`, `.to_string`, `.unwrap_or`, `Some`
- **Macros:** `format!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::custom_property_merge_is_case_insensitive_original_wins_and_new_ids_are_unique` (document_comparer.rs:10005); `coverage_boundary_tests::custom_property_adoption_refuses_unrelated_collision_and_malformed_xml` (document_comparer.rs:10034)

#### `repair_missing_core_relationships`  (L6258-6286)
`fn repair_missing_core_relationships(out: &mut PartFs, out_main: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::part_bytes` (opc/mod.rs:628) L6267; `PartFs::read_rels_for` (opc/mod.rs:991) L6270; `relative_rel_target` (opc/mod.rs:180) L6276; `PartFs::add_document_relationship` (opc/mod.rs:1003) L6277
- **Calls (std/external/unresolved):** `.any`, `.ends_with`, `.is_none`, `.is_some_and`, `.iter`
- **Macros:** `format!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::revised_chrome_preserves_existing_parts_and_repairs_relationships_idempotently` (document_comparer.rs:10777)

#### `get_revisions`  (L6293-6348, pub)
`pub fn get_revisions( docx: &[u8], settings: &crate::comparer::WmlComparerSettings, ) -> Result<Vec<crate::comparer::WmlComparerRevision>, OpcError>`

- **Does:** D.6 — `WmlComparer.GetRevisions` (:3940) byte facade: list every tracked revision in a redline `.docx` — main-part groups, footnote/endnote definition groups, `w:rPrChange` format changes, then (settings-gated) move detection.
- **Calls (crate):** `admit_input` (document_comparer.rs:4907) L6299; `PartFs::open_within` (opc/mod.rs:593) L6300; `PartFs::main_document_part` (opc/mod.rs:1070) L6301; `PartFs::part_string` (opc/mod.rs:636) L6304,6323; `Dom::new` (xmllinq/mod.rs:387) L6307; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L6308,6324; `Dom::root` (xmllinq/mod.rs:822) L6309,6325; `test_for_invalid_content` (comparer/preprocess.rs:602) L6315; `remove_existing_powertools_markup` (comparer/preprocess.rs:572) L6316,6326; `notes_part_names` (document_comparer.rs:6461) L6318; `Dom::element` (xmllinq/mod.rs:655) L6332; `get_revisions_from_body` (comparer/revisions.rs:151) L6335; `get_revisions_from_note_definitions` (comparer/revisions.rs:226) L6337; `get_format_change_revisions` (comparer/revisions.rs:366) L6345; `detect_moves` (comparer/revisions.rs:414) L6346
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.extend`, `.map_err`, `.ok_or_else`, `.push`, `.to_string`, `.unwrap_or_else`, `Ok`, `OpcError::PartNotFound`, `Vec::new`, `W::body`, `W::endnote`, `W::footnote`
- **Macros:** `format!`, `vec!`
- **Called by:** none found in src

#### `revision_to_json`  (L6354-6391, pub)
`pub fn revision_to_json(r: &crate::comparer::WmlComparerRevision) -> String`

- **Does:** Serialize one revision to the stable JSON object shape of the wasm `getRevisions` and Python `get_revisions` bindings.
- **Calls (crate):** `revision_to_json::{fn esc}` (document_comparer.rs:6355) L6374,6381,6382,6383,6389
- **Calls (std/external/unresolved):** `.as_deref`, `.as_ref`, `.collect`, `.iter`, `.join`, `.map_or`, `.map`, `.to_string`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `tests::revision_json_matches_cli_shape_and_escapes_control_chars` (document_comparer.rs:8260)

#### `revision_to_json::{fn esc}`  (L6355-6369)
`fn esc(s: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chars`, `.len`, `.push_str`, `.push`, `String::with_capacity`
- **Macros:** `format!`
- **Called by:** `revision_to_json` (document_comparer.rs:6354)

#### `revisions_to_json`  (L6396-6399, pub)
`pub fn revisions_to_json(revs: &[crate::comparer::WmlComparerRevision]) -> String`

- **Does:** Serialize a revision list to a single JSON array string — the wasm `getRevisions` binding shape.
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.join`, `.map`
- **Macros:** `format!`
- **Called by:** `tests::revision_json_matches_cli_shape_and_escapes_control_chars` (document_comparer.rs:8260)

#### `compare_documents`  (L6402-6408, pub)
`pub fn compare_documents( original: &[u8], modified: &[u8], author: &str, ) -> Result<Vec<u8>, OpcError>`

- **Does:** `DocumentComparer.CompareDocuments(original, modified, author)`.
- **Calls (crate):** `compare_documents_with_options` (document_comparer.rs:6411) L6407
- **Called by:** none found in src

#### `compare_documents_with_options`  (L6411-6418, pub)
`pub fn compare_documents_with_options( original: &[u8], modified: &[u8], author: &str, date: &str, ) -> Result<Vec<u8>, OpcError>`

- **Does:** Compare with an explicit revision timestamp (for reproducible output).
- **Calls (crate):** `compare_documents_internal` (document_comparer.rs:6442) L6417
- **Called by:** `compare_documents` (document_comparer.rs:6402)

#### `compare_documents_with_settings`  (L6422-6428, pub)
`pub fn compare_documents_with_settings( original: &[u8], modified: &[u8], settings: &WmlComparerSettings, ) -> Result<Vec<u8>, OpcError>`

- **Does:** Compare with caller-supplied [`WmlComparerSettings`] (author/date/detail threshold/…).
- **Calls (crate):** `compare_documents_impl` (document_comparer.rs:6520) L6427
- **Called by:** `run` (bin/jubarte.rs:2862); `public_option_source_ownership_tests::public_boundary_shifts_preserve_all_source_owners_with_lexical_options` (comparer/cross_para.rs:2702); `public_option_source_ownership_tests::public_comparison_options_preserve_reordered_paragraphs_formats_and_note_owners` (comparer/cross_para.rs:2772); `public_option_source_ownership_tests::public_reordered_bulk_paragraphs_preserve_sources_on_move_thrash_boundaries` (comparer/cross_para.rs:2935); `public_option_source_ownership_tests::strict_and_transitional_packages_preserve_complete_authored_body_ownership` (comparer/cross_para.rs:3031); `public_option_source_ownership_tests::zero_word_move_threshold_preserves_authored_nontext_and_separator_payloads` (comparer/cross_para.rs:3178); `public_option_source_ownership_tests::public_compound_and_one_sided_paragraph_windows_preserve_complete_sources` (comparer/cross_para.rs:3325); `row_nontext_ownership_regressions::public_compare_keeps_unchanged_tab_and_break_cells_when_text_moves_into_their_row` (comparer/finalize.rs:23895); `coverage_real_revision_merge_matrix_tests::public_compare_multi_original_blank_tail_recovers_complete_clean_sources` (comparer/finalize.rs:26112); `coverage_real_revision_merge_matrix_tests::whole_story_title_label_heading_and_list_boundaries_keep_source_ownership` (comparer/finalize.rs:27055); `coverage_real_revision_merge_matrix_tests::public_terminal_field_math_and_break_boundaries_preserve_all_source_owners` (comparer/finalize.rs:27505); `coverage_real_revision_merge_matrix_tests::public_cross_paragraph_field_ends_keep_exact_source_instruction_and_cache_owners` (comparer/finalize.rs:27727); +18 more

#### `compare_documents_for_edit`  (L6431-6437, pub(crate))
`pub(crate) fn compare_documents_for_edit( original: &[u8], modified: &[u8], settings: &WmlComparerSettings, ) -> Result<Vec<u8>, OpcError>`

- **Does:** Edit restores permissions under its own revised-by-id policy (#1176).
- **Calls (crate):** `compare_documents_impl` (document_comparer.rs:6520) L6436
- **Called by:** `coverage_boundary_tests::the_permission_union_is_a_no_op_without_ranges` (document_comparer.rs:8886); `coverage_boundary_tests::the_permission_union_is_a_no_op_with_bookmarks_only` (document_comparer.rs:8919); `produce` (edit/prepared.rs:928)

#### `compare_documents_internal`  (L6442-6455, pub)
`pub fn compare_documents_internal( original: &[u8], modified: &[u8], author: &str, date: &str, pre_process_original: bool, ) -> Result<Vec<u8>, OpcError>`

- **Does:** `WmlComparer.CompareInternal` (:152).
- **Calls (crate):** `<WmlComparerSettings as Default>::default` (comparer/mod.rs:1493) L6452; `compare_documents_impl` (document_comparer.rs:6520) L6454
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `compare_documents_with_options` (document_comparer.rs:6411)

#### `notes_part_names`  (L6461-6480)
`fn notes_part_names(pkg: &PartFs) -> (String, String)`

- **Does:** Resolve the footnotes/endnotes part names via the main-document rels, falling back to the standard names.
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:1070) L6462; `PartFs::read_rels_for` (opc/mod.rs:991) L6467; `PartFs::resolve_rel_target` (opc/mod.rs:829) L6473,6474
- **Calls (std/external/unresolved):** `.as_deref`, `.next`, `.rsplit`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `Some`
- **Called by:** `get_revisions` (document_comparer.rs:6293); `compare_documents_impl` (document_comparer.rs:6520)

#### `main_part_xml`  (L6486-6493)
`fn main_part_xml(docx: &[u8], limits: crate::admission::InputLimits) -> Option<String>`

- **Does:** True when any package part's XML carries tracked-change markup that Word Compare would fold into the final view before diffing.
- **Calls (crate):** `PartFs::open_within` (opc/mod.rs:593) L6487; `PartFs::main_document_part` (opc/mod.rs:1070) (by name) L6488; `PartFs::part_string` (opc/mod.rs:636) (by name) L6491; `normalize_strict_namespaces` (document_comparer.rs:4607) L6492
- **Calls (std/external/unresolved):** `.into_owned`, `.ok`, `.to_string`, `.unwrap_or_else`, `Some`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::admission_preserves_typed_refusal_and_identifies_document_side` (document_comparer.rs:10115)

#### `docx_has_tracked_changes`  (L6495-6518)
`fn docx_has_tracked_changes(docx: &[u8], limits: crate::admission::InputLimits) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::open_within` (opc/mod.rs:593) L6496; `PartFs::parts` (opc/mod.rs:733) (by name) L6499; `PartFs::part_string` (opc/mod.rs:636) (by name) L6503
- **Calls (std/external/unresolved):** `.contains`, `.ends_with`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520); `coverage_boundary_tests::admission_preserves_typed_refusal_and_identifies_document_side` (document_comparer.rs:10115); `coverage_boundary_tests::tracked_change_admission_scans_each_xml_story_and_refuses_binary_or_non_xml_carriers` (document_comparer.rs:11373); `coverage_package_source_matrix_tests::public_comparison_preserves_complete_sources_across_package_features` (document_comparer.rs:11713); `coverage_package_source_matrix_tests::accepted_source_revision_converges_to_identical_final_before_comparison` (document_comparer.rs:11932)

#### `compare_documents_impl`  (L6520-8071)
`fn compare_documents_impl( original: &[u8], modified: &[u8], settings: &WmlComparerSettings, pre_process_original: bool, union_permissions: bool, ) -> Result<Vec<u8>, OpcError>`

- **Does:** _no doc comment_
- **Calls (crate):** `admit_input` (document_comparer.rs:4907) L6530,6559; `strict_to_transitional_docx_within` (strict_translation.rs:769) L6535,6560,6565; `docx_has_tracked_changes` (document_comparer.rs:6495) L6541,6592,6593; `accept_source_revisions` (document_comparer.rs:4972) L6543,6595,6596; `fix_up_drawing_ids_in_package_within` (comparer/fixups.rs:126) L6548; `main_part_xml` (document_comparer.rs:6486) L6576,6577; `fix_up_drawing_ids_in_package` (comparer/fixups.rs:121) L6601; `PartFs::open_within` (opc/mod.rs:593) L6607,6608,6752,6754; `pre_process_markup` (document_comparer.rs:4713) L6615,6622; `PartFs::main_document_part` (opc/mod.rs:1070) L6627,6630; `PartFs::part_string` (opc/mod.rs:636) L6634,6637,6785,6786,6867,7138...; `normalize_strict_namespaces` (document_comparer.rs:4607) L6642,6643,7913; `Dom::new` (xmllinq/mod.rs:387) L6646,6869,6942,7066,7090,7120...; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L6647,6648,6870,6871,6943,7067...; `Dom::root` (xmllinq/mod.rs:822) L6649,6652,6872,6944,7068,7092...; `merged_body` (document_comparer.rs:4630) L6655,6657; `normalize_universal_measures` (comparer/finalize.rs:11490) L6667,6704,7949; `notes_part_names` (document_comparer.rs:6461) L6681,6682; `compare_documents_impl::{fn parse_part_root}` (document_comparer.rs:6676) L6684,6685,6686,6687,6688,6689; `PartFs::part_string` (opc/mod.rs:636) (by name) L6711,6715,6867,6940,7117,7137...; `default_paragraph_line` (document_comparer.rs:3170) L6713,7974; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L6714,6719,6738,6739,6898,6926...; `has_default_table_style` (document_comparer.rs:3156) L6717; `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215) L6727; `PartFs::to_zip` (opc/mod.rs:749) L6754; `reconcile_dangling_relationships` (comparer/parts.rs:273) L6758; `adopt_revised_header_footer` (document_comparer.rs:5174) L6768; `strip_final_sectpr_inherited_header_footer` (document_comparer.rs:5061) L6771; `final_header_footer_slots` (document_comparer.rs:5127) L6774; `mark_a_only_hf_content_as_deleted` (document_comparer.rs:5571) L6777; `carry_comments` (comparer/comments.rs:1583) L6787; `fix_up_revision_ids` (comparer/finalize.rs:768) L6799; `reorder_replacements_ins_before_del` (comparer/finalize.rs:6657) L6807; `convert_outer_math_wraps_to_internal` (comparer/produce.rs:778) L6812; `carry_bookmarks` (comparer/bookmarks.rs:460) L6817; `carry_permission_union` (comparer/bookmarks.rs:594) L6825,7709,7864; `fix_up_doc_pr_ids` (comparer/fixups.rs:32) L6837; `fix_up_shape_ids` (comparer/fixups.rs:54) L6838; `fix_up_shape_type_ids` (comparer/fixups.rs:75) L6839; `Dom::serialize_element` (xmllinq/mod.rs:1315) L6841,7023,7032,7071,7094,7130...; `PartFs::set_part` (opc/mod.rs:661) (by name) L6842,7023,7032,7043,7071,7094...; `defined_style_ids` (comparer/footnotes.rs:217) L6875,7030,7267; `materialize_builtin_paragraph_defaults` (document_comparer.rs:3764) L6876,6877; `copy_missing_styles_by_name` (document_comparer.rs:3623) L6878; `adopt_missing_styles_structure` (document_comparer.rs:3786) L6879; `canonicalize_style_ids` (document_comparer.rs:3667) L6880,7069,7093; `Dom::descendants` (xmllinq/mod.rs:668) L6893,6956,6960,7230,7233,7237...; `Dom::attribute` (xmllinq/mod.rs:874) L6895,6957,6981,7202,7439,7454...; `copy_missing_styles` (comparer/footnotes.rs:267) L6904; `copy_missing_numbering` (comparer/footnotes.rs:364) L6908; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L6913; `Dom::attributes` (xmllinq/mod.rs:899) L6914,7844; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L6915,7845; `carry_relationship` (comparer/parts.rs:517) L6918; `next_free_revision_id` (document_comparer.rs:469) L6959; `Dom::element` (xmllinq/mod.rs:655) L6961,6964,6966,6975,6977,6987...; `mark_has` (edit/tracked.rs:1628) (by name) L6968,6971; `Dom::new_element` (xmllinq/mod.rs:478) L6989,6999,7585,7840; `Dom::elements` (xmllinq/mod.rs:640) L6990,7195,7348,7436,7451,7468...; `Dom::name` (xmllinq/mod.rs:526) L6991,7854; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L6996; `Dom::add` (xmllinq/mod.rs:1074) L6997,7016,7017,7858; `PartFs::add_content_type_override` (opc/mod.rs:845) (by name) L7044,7072,7666; `PartFs::read_rels_for` (opc/mod.rs:991) (by name) L7048,7076,7670,7980; `PartFs::add_document_relationship` (opc/mod.rs:1003) (by name) L7052,7080,7674; `factory_scaffold_bake_b_styles` (document_comparer.rs:2616) L7070; `PartFs::parts` (opc/mod.rs:733) (by name) L7102,7274,7906,7929; `remap_style_refs` (document_comparer.rs:3736) L7129,7698; `merge_revised_font_table` (document_comparer.rs:1384) L7139; `adopt_revised_styles_chrome` (document_comparer.rs:3348) L7147; `ensure_factory_package_chrome` (document_comparer.rs:3402) L7148; `repair_missing_core_relationships` (document_comparer.rs:6258) L7149; `merge_custom_properties` (document_comparer.rs:6196) L7150; `restore_deleted_paragraph_spacing` (document_comparer.rs:1415) L7155; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L7178; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L7181; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L7184,7473; `Dom::name_is` (xmllinq/mod.rs:535) L7198,7824; `Dom::value_str` (xmllinq/mod.rs:1195) L7236,7505; `Dom::remove` (xmllinq/mod.rs:1118) L7246,7857; `strip_unresolved_style_refs` (comparer/footnotes.rs:233) L7296; `style_match_key` (document_comparer.rs:1843) L7350,7453,7462; `inherited_only_style_changes` (document_comparer.rs:1601) L7365; `styles_the_revision_lacks` (document_comparer.rs:1790) L7367; `merge_revised_style_definitions` (document_comparer.rs:2160) L7368; `merge_normal_style_spacing` (document_comparer.rs:777) L7376; `merge_normal_style_rpr` (document_comparer.rs:4343) L7382; `prune_normal_rpr_context_equal_attrs` (document_comparer.rs:3822) L7386; `bake_bothsides_dd_disabling_neutralizers` (document_comparer.rs:3909) L7390; `cascade_normal_change_to_based_styles` (document_comparer.rs:1124) L7393; `normalize_word_paragraph_style_line` (document_comparer.rs:2948) L7395; `align_paragraph_style_fonts_with_normal` (document_comparer.rs:3053) L7397; `resolve_redefined_style_metrics` (document_comparer.rs:4063) L7401; `complete_root_style_change_records` (document_comparer.rs:1873) L7404; `complete_based_style_change_records` (document_comparer.rs:1951) L7408; `remove_inherited_only_style_records` (document_comparer.rs:1672) L7410; `restore_styles` (document_comparer.rs:1814) L7411; `referenced_theme_part` (document_comparer.rs:3511) L7417; `reresolve_theme_color_hexes` (document_comparer.rs:6012) L7419; `coverage_boundary_tests::index` (document_comparer.rs:8844) L7443,7444; `Dom::ancestors` (xmllinq/mod.rs:770) L7530; `effective_para_spacing` (document_comparer.rs:358) L7538,7545; `insert_child_by_rank` (document_comparer.rs:1327) L7586; `synthesize_dangling_numbering` (comparer/footnotes.rs:533) L7658; `adopt_b_notes_when_a_lacks_separators` (document_comparer.rs:5871) L7734; `sync_settings_special_note_ids` (comparer/footnotes.rs:852) L7743,8056; `header_footer_refs_by_section` (document_comparer.rs:129) L7761,7762; `section_pairing` (document_comparer.rs:202) L7765; `pair_header_footer` (document_comparer.rs:238) L7792; `part_rels_agree` (document_comparer.rs:70) L7800; `carry_revised_part_relationships` (document_comparer.rs:25) L7803; `Dom::value` (xmllinq/mod.rs:1185) L7816; `text_of` (changes.rs:1518) (by name) L7820,7887; `compare_bodies_faithful` (comparer/mod.rs:115) L7825; `XNamespace::xmlns` (xmllinq/mod.rs:122) L7843; `XNamespace::namespace_name` (xmllinq/mod.rs:140) L7845; `XName::local_name` (xmllinq/mod.rs:208) (by name) L7846; `delete_story_closing_mark` (document_comparer.rs:5593) L7861; `reconcile_part_relationships` (comparer/parts.rs:479) L7897; `PartFs::adopt_missing_content_types` (opc/mod.rs:860) (by name) L7900; `fix_strict_validity_artifacts` (comparer/finalize.rs:11326) L7950; `repair_inherited_invalidity` (comparer/finalize.rs:14789) L7954; `hoist_hyperlinks_out_of_revisions` (comparer/finalize.rs:14616) L7958; `enforce_deleted_text_kinds` (comparer/finalize.rs:14856) L7959; `remove_powertools_scratch_markup` (comparer/finalize.rs:1060) L7960; `strip_propertyless_ppr` (comparer/finalize.rs:6008) L7966; `strip_unrecorded_word_defaults` (comparer/finalize.rs:6082) L7975; `PartFs::read_rels_for` (opc/mod.rs:991) L7990; `rewrite_inserted_external_hyperlinks` (comparer/finalize.rs:6336) L8000; `align_word_table_and_comment_chrome` (comparer/finalize.rs:6220) L8006; `align_remaining_ladder_rungs` (comparer/finalize.rs:14908) L8009; `enforce_part_schema_order` (comparer/finalize.rs:10741) L8021; `declare_extension_namespaces_ignorable` (comparer/finalize.rs:942) L8022; `bind_compatibility_prefixes` (comparer/finalize.rs:1026) L8023; `collect_ids` (ambiguous: `collect_ids` (debug.rs:1133); `collect_ids` (validate.rs:1718)) L8050,8051; `PartFs::to_zip` (opc/mod.rs:749) (by name) L8070
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.as_ref`, `.as_str`, `.clone`, `.cloned`, `.collect`, `.contains_key`, `.contains`, `.dedup`, `.ends_with`, `.entry`, `.enumerate`, `.extend`, `.filter_map`, `.filter`, `.find_map`, `.find`, `.first`, `.flat_map`, `.flatten`, `.fold`, `.get`, `.insert`, `.into_bytes`, `.into_iter`, `.into_owned`, `.into`, `.is_empty`, `.is_none`, `.is_ok`, `.is_some_and`, `.is_some`, `.iter`, `.last`, `.len`, `.map_err`, `.map_or_else`, `.map`, `.ok_or_else`, `.ok`, `.or_default`, `.or_else`, `.or_insert_with`, `.parse`, `.push`, `.sort`, `.starts_with`, `.take_while`, `.take`, `.then`, `.to_string`, `.trim`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `Ok`, `OpcError::PartNotFound`, `Some`, `Vec::new`, `W::body`, `W::del_text`, `W::endnote`, `W::footnote`, `W::id`, `W::name`, `W::object`, `W::p_pr`, `W::p`, `W::r_pr`, `W::t`, `W::tbl_pr`, `W::tbl`, `W::val`, `b_id_for`, `crate::namespaces::MC::name`, `crate::namespaces::PT::default_line`, `crate::namespaces::PT::has_default_table_style`, `crate::namespaces::VML::name`, `crate::namespaces::W14::name`, `crate::namespaces::WNE::name`, `crate::revision_processor::drop_unreferenced_headers_footers`, `dp`, `is_ins_empty`, `std::collections::HashMap::new`, `std::collections::HashSet::new`
- **Macros:** `format!`, `matches!`, `vec!`
- **Called by:** `compare_documents_with_settings` (document_comparer.rs:6422); `compare_documents_for_edit` (document_comparer.rs:6431); `compare_documents_internal` (document_comparer.rs:6442)

#### `compare_documents_impl::{fn parse_part_root}`  (L6676-6680)
`fn parse_part_root(dom: &mut Dom, pkg: &PartFs, name: &str) -> Option<NodeId>`

- **Does:** B.4 — reference-driven notes processing: parse both documents' notes parts AND an independent copy of the original's parts (the withRevisions parts — C# gets them from the wmlResult clone of preprocessed source1) into the same Dom as the bodies. _(from a code comment)_
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L6677; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L6678; `Dom::root` (xmllinq/mod.rs:822) L6679
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520)

## Test functions (129)

- `tests::parse` (L8087): _no doc comment_
- `tests::restored_deleted_spacing_leaves_scratch_unids_behind` (L8100): A's working copy carries `pt14:Unid` on its spacing; the restored spacing on the deleted paragraph must hold only the `w:` attributes.
- `tests::revised_fonts_join_the_font_table` (L8134): Fonts only the revised document declares join the output's font table (file_46 × file_47: without Liberation Serif's entry Word fell back to Times New Roman where its own redline used Hiragino Mincho).
- `tests::next_free_revision_id_is_max_plus_one_across_change_families` (L8175): `next_free_revision_id` must be one greater than the max numeric id on ANY `w:*Change` revision element in the stylesheet, never a hardcoded 1.
- `tests::next_free_revision_id_counts_list_move_range_and_cell_carriers` (L8203): Every `w:id` revision carrier raises the floor, not only the `*Change` records: a list, move-range or table-cell revision holding the next id would collide with a synthesized `w:pPrChange`.
- `tests::add_rpr_child_keeps_sz_in_schema_order_after_position` (L8226): sz must be inserted after position/kern, NOT immediately after rFonts — rFonts < color < spacing < w < kern < position < sz in EG_RPrBase.
- `tests::revision_json_matches_cli_shape_and_escapes_control_chars` (L8260): `revision_to_json` / `revisions_to_json` are the single serialization of the wasm `getRevisions` and Python `get_revisions` bindings: one object shape, full string escaping — backslash, quote, and EVERY control char < 0x20 (document text can carry tabs, CRs, vertical tabs).
- `tests::word_canonical_style_id_preserves_toc_builtins` (L8326): _no doc comment_
- `tests::word_canonical_style_id_keeps_builtin_ids_that_differ_from_their_names` (L8343): _no doc comment_
- `tests::word_canonical_style_id_drops_punctuation` (L8370): Word's style ids hold letters and digits only: "Normal (Web)" is `NormalWeb` (evals comments_doc × document).
- `tests::normal_takes_b_theme_fonts_from_its_doc_defaults` (L8382): B's docDefaults name their fonts by theme (`w:asciiTheme="minorHAnsi"`, the form Word itself writes) and A's name Times New Roman outright.
- `tests::normal_is_found_by_name_when_no_style_is_the_default` (L8424): LibreOffice names its Normal `style0` and marks no paragraph style as the default; Word still finds it by its name (multi_section_nested_ table_rowspan: the Word redline carries B's whole Normal, ours left it bare because B "had no Normal").
- `tests::stylesheet` (L8440): A stylesheet with a Normal; `dd` = (rPrDefault, pPrDefault) contents, `None` for no docDefaults at all.
- `tests::normal_pair` (L8456): _no doc comment_
- `tests::merge_normals` (L8473): Run `merge_normal_style_spacing` then `merge_normal_style_rpr` over A's and B's stylesheets; returns the dom and the output root.
- `tests::live_normal_attrs` (L8484): `(attribute, value)` pairs of the live Normal's `container/local`.
- `tests::attrs` (L8502): _no doc comment_
- `tests::normal_uses_word_factory_defaults_when_b_has_no_doc_defaults` (L8517): A LibreOffice B with no docDefaults at all: Word reads it with its own factory defaults (after 160, line 278, kern 2, standard contextual ligatures) and writes them into the live Normal (multi_section_nested_table_rowspan, table_bookmark_end × table_vmerge_colspan).
- `tests::normal_writes_the_implicit_complex_script_size` (L8552): B declares no complex-script size while A's docDefaults do: Word writes the implicit 20 half-points (17 of 17 corpus Word redlines).
- `tests::normal_writes_only_the_language_attributes_that_change` (L8574): Word writes only the language attributes B changes against A's docDefaults, reading a missing attribute as en-US / en-US / ar-SA (file_103 × file_104: eastAsia zh-CN alone; mcdoc_meeting_agenda: B silent, A zh-CN → eastAsia en-US).
- `tests::normal_takes_b_run_defaults_when_only_b_paragraph_props_are_stored` (L8617): file_103 × file_104: A's Normal is bare, B's carries only a pPr.
- `tests::live_normal_ppr` (L8643): Live Normal pPr, children in order, pPrChange left out.
- `tests::normal_takes_b_doc_default_indent_and_justification` (L8667): file_103 × file_104: B keeps indents, justification and line-unit spacing in its docDefaults, which the redline (A's docDefaults) loses.
- `tests::normal_neutralizes_a_doc_default_indent_and_justification` (L8709): file_104 × file_105, the reverse: A's docDefaults indent and justify, B's do not.
- `tests::normal_clears_a_doc_default_borders` (L8762): A's docDefaults draw paragraph borders, B's do not: Word writes a `nil` border on every edge A draws, even when no spacing changes (sd_1494_table_left_indent × sdpr_titleonly).
- `coverage_boundary_tests::xml` (L8812): _no doc comment_
- `coverage_boundary_tests::root` (L8821): _no doc comment_
- `coverage_boundary_tests::child` (L8826): _no doc comment_
- `coverage_boundary_tests::children` (L8830): _no doc comment_
- `coverage_boundary_tests::style` (L8837): _no doc comment_
- `coverage_boundary_tests::index` (L8844): _no doc comment_
- `coverage_boundary_tests::package` (L8851): _no doc comment_
- `coverage_boundary_tests::part_root` (L8867): _no doc comment_
- `coverage_boundary_tests::settings` (L8873): _no doc comment_
- `coverage_boundary_tests::the_permission_union_is_a_no_op_without_ranges` (L8886): #1190: `compare_documents_with_settings` and `compare_documents_for_edit` differ only in the permission union.
- `coverage_boundary_tests::the_permission_union_is_a_no_op_with_bookmarks_only` (L8919): #1215: bookmarks alone must not reach the permission union either.
- `coverage_boundary_tests::assert_revision` (L8936): _no doc comment_
- `coverage_boundary_tests::merged_bodies_preserve_content_order_and_only_last_section_properties` (L8946): _no doc comment_
- `coverage_boundary_tests::section_pairing_accounts_for_inserted_deleted_and_recorded_breaks` (L8980): _no doc comment_
- `coverage_boundary_tests::paragraph_spacing_tracks_table_precedence_and_missing_ancestors` (L8996): _no doc comment_
- `coverage_boundary_tests::effective_run_properties_merge_independent_slots_and_clear_alternatives` (L9022): _no doc comment_
- `coverage_boundary_tests::style_cycles_stop_and_self_based_style_does_not_hide_own_properties` (L9059): _no doc comment_
- `coverage_boundary_tests::inherited_only_records_are_suppressed_but_inbound_history_survives` (L9082): _no doc comment_
- `coverage_boundary_tests::a_styles_two_records_share_one_id` (L9118): Word gives a style's `w:pPrChange` and `w:rPrChange` one id (199 of 199 corpus styles carrying both), so the pair is one change.
- `coverage_boundary_tests::recorded_defaults_complete_slots_without_overwriting_explicit_values` (L9155): _no doc comment_
- `coverage_boundary_tests::root_records_gain_missing_companion_and_keep_recorded_original` (L9216): _no doc comment_
- `coverage_boundary_tests::absent_revised_styles_restore_distinct_twins_and_skip_removed_targets` (L9261): _no doc comment_
- `coverage_boundary_tests::canonical_style_ids_refuse_collisions_and_remap_only_known_references` (L9294): _no doc comment_
- `coverage_boundary_tests::style_structure_adoption_preserves_existing_defaults_and_empty_explicit_spacing` (L9340): _no doc comment_
- `coverage_boundary_tests::adopted_header_marks_content_and_paragraph_properties_with_pinned_provenance` (L9388): _no doc comment_
- `coverage_boundary_tests::deleted_footer_preserves_last_mark_and_records_original_properties` (L9414): _no doc comment_
- `coverage_boundary_tests::one_sided_header_footer_tables_mark_every_row` (L9465): Word 16.115 hangs opening a redline whose footer table has every cell deleted but no row marked (b09, release 0.12.0 sample); Word's own redline of the pair deletes the row too.
- `coverage_boundary_tests::one_sided_story_marks_take_ids_above_the_story_s_own` (L9503): A one-sided story's new marks take ids above the ones it already carries, so a kept revision never shares its id with a new one.
- `coverage_boundary_tests::one_sided_story_fields_join_the_story_revision` (L9531): A paragraph whose only content is a field (`fldSimple`, empty or with a result) is content: left out, a dropped footer's NUMPAGES stayed live, the b15 shape Word 16.115 hung on.
- `coverage_boundary_tests::deletion_skips_inbound_changes_and_preserves_existing_blank_history` (L9560): _no doc comment_
- `coverage_boundary_tests::story_closing_mark_restores_original_props_and_does_not_double_mark` (L9579): _no doc comment_
- `coverage_boundary_tests::closing_mark_refuses_live_content_and_handles_empty_or_unformatted_stories` (L9604): _no doc comment_
- `coverage_boundary_tests::child_creation_places_properties_before_schema_successors_and_reuses_nodes` (L9644): _no doc comment_
- `coverage_boundary_tests::final_header_slots_strip_only_inherited_non_revised_references` (L9662): _no doc comment_
- `coverage_boundary_tests::header_pairing_uses_surviving_section_inheritance_and_content_identity` (L9706): _no doc comment_
- `coverage_boundary_tests::a_colliding_header_or_footer_takes_the_first_free_number` (L9749): _no doc comment_
- `coverage_boundary_tests::unique_part_names_handle_binary_collisions_reuse_and_flat_names` (L9775): _no doc comment_
- `coverage_boundary_tests::relationship_agreement_refuses_missing_mismatched_and_dangling_targets` (L9808): _no doc comment_
- `coverage_boundary_tests::carry_relationships_reuses_minted_id_and_refuses_unresolved_references` (L9852): _no doc comment_
- `coverage_boundary_tests::notes_merge_only_structural_definitions_ahead_of_rectified_content` (L9897): _no doc comment_
- `coverage_boundary_tests::notes_copy_preserves_media_and_escapes_external_links_without_io` (L9938): _no doc comment_
- `coverage_boundary_tests::notes_adoption_refuses_missing_separator_and_preserves_original_separator` (L9978): _no doc comment_
- `coverage_boundary_tests::custom_property_merge_is_case_insensitive_original_wins_and_new_ids_are_unique` (L10005): _no doc comment_
- `coverage_boundary_tests::custom_property_adoption_refuses_unrelated_collision_and_malformed_xml` (L10034): _no doc comment_
- `coverage_boundary_tests::empty_missing_and_non_xml_package_parts_are_safe_noops` (L10069): _no doc comment_
- `coverage_boundary_tests::preprocessing_refuses_orphan_notes_before_mutating_package` (L10100): _no doc comment_
- `coverage_boundary_tests::admission_preserves_typed_refusal_and_identifies_document_side` (L10115): _no doc comment_
- `coverage_boundary_tests::strict_namespace_normalization_rewrites_only_the_three_ooxml_families` (L10147): _no doc comment_
- `coverage_boundary_tests::final_header_footer_adoption_respects_existing_body_owners_and_cached_donor_parts` (L10165): _no doc comment_
- `coverage_boundary_tests::disabling_default_neutralizers_preserve_tracked_history_at_inheritance_limit` (L10360): _no doc comment_
- `coverage_boundary_tests::normal_cascade_preserves_partial_spacing_owners_without_inventing_metrics` (L10443): _no doc comment_
- `coverage_boundary_tests::header_adoption_preserves_colliding_media_and_uses_correct_content_types` (L10516): _no doc comment_
- `coverage_boundary_tests::a_renamed_header_copy_never_hides_another_header_of_the_revision` (L10615): _no doc comment_
- `coverage_boundary_tests::a_reused_header_keeps_its_own_relationships` (L10668): _no doc comment_
- `coverage_boundary_tests::header_adoption_refuses_unresolvable_references_and_missing_story_structure` (L10706): _no doc comment_
- `coverage_boundary_tests::original_footer_deletion_is_per_kind_and_duplicate_references_are_processed_once` (L10741): _no doc comment_
- `coverage_boundary_tests::revised_chrome_preserves_existing_parts_and_repairs_relationships_idempotently` (L10777): _no doc comment_
- `coverage_boundary_tests::revision_ids_ignore_missing_and_nonnumeric_values_and_saturate` (L10839): _no doc comment_
- `coverage_boundary_tests::revised_style_merge_tracks_both_blocks_without_copying_revised_history` (L10854): _no doc comment_
- `coverage_boundary_tests::deleted_spacing_restoration_refuses_unmarked_missing_and_already_spaced_paragraphs` (L10922): _no doc comment_
- `coverage_boundary_tests::separator_media_collisions_keep_a_resolvable_relative_relationship` (L10945): _no doc comment_
- `coverage_boundary_tests::normal_run_adoption_preserves_complete_old_properties_and_revised_slot_ownership` (L10978): _no doc comment_
- `coverage_boundary_tests::theme_cache_recaching_preserves_independent_fill_color_and_tolerated_source_hexes` (L11076): _no doc comment_
- `coverage_boundary_tests::header_insertion_wraps_supported_owned_payload_and_retains_noncontent_and_foreign_revisions` (L11191): _no doc comment_
- `coverage_boundary_tests::normal_history_cascade_keeps_own_font_language_and_spacing_slots_and_existing_records` (L11259): _no doc comment_
- `coverage_boundary_tests::tracked_change_admission_scans_each_xml_story_and_refuses_binary_or_non_xml_carriers` (L11373): _no doc comment_
- `coverage_boundary_tests::section_story_references_resolve_actual_opc_targets_before_legacy_word_paths` (L11412): _no doc comment_
- `coverage_package_source_matrix_tests::xml` (L11485): _no doc comment_
- `coverage_package_source_matrix_tests::root` (L11493): _no doc comment_
- `coverage_package_source_matrix_tests::signature` (L11500): Semantic properties include every attribute and child, independently of prefix declaration locations chosen when the OPC part is serialized. _(from a code comment)_
- `coverage_package_source_matrix_tests::view` (L11533): _no doc comment_
- `coverage_package_source_matrix_tests::add_part` (L11603): _no doc comment_
- `coverage_package_source_matrix_tests::package` (L11610): _no doc comment_
- `coverage_package_source_matrix_tests::public_comparison_preserves_complete_sources_across_package_features` (L11713): _no doc comment_
- `coverage_package_source_matrix_tests::defaults_delta_distinguishes_implicit_on_off_and_explicit_overrides` (L11780): _no doc comment_
- `coverage_package_source_matrix_tests::normal_sync_preserves_history_and_replaces_indent_in_schema_order` (L11852): _no doc comment_
- `coverage_package_source_matrix_tests::accepted_source_revision_converges_to_identical_final_before_comparison` (L11932): _no doc comment_
- `comparer_relationship_contract_tests::package` (L11991): _no doc comment_
- `comparer_relationship_contract_tests::referenced_relationship_agreement_requires_matching_type_mode_and_payload_or_external_target` (L11995): _no doc comment_
- `comparer_relationship_contract_tests::repeated_hyperlinks_share_one_carried_relationship_and_keep_independent_source_text_and_format` (L12086): _no doc comment_
- `authored_style_chain_boundary_tests::parse` (L12142): _no doc comment_
- `authored_style_chain_boundary_tests::leaf` (L12161): _no doc comment_
- `authored_style_chain_boundary_tests::attrs` (L12168): _no doc comment_
- `authored_style_chain_boundary_tests::inherited_style_font_alternatives_and_missing_blocks_keep_exact_declarations` (L12177): _no doc comment_
- `authored_style_chain_boundary_tests::based_on_cycles_stop_after_unique_authored_owners_with_nearest_properties_winning` (L12222): _no doc comment_
- `authored_style_chain_boundary_tests::bounded_style_walk_reads_exactly_the_nearest_thirty_two_owners` (L12244): _no doc comment_
- `isolated_header_container_revision_boundary_tests::semantic` (L12285): _no doc comment_
- `isolated_header_container_revision_boundary_tests::parsed` (L12314): _no doc comment_
- `isolated_header_container_revision_boundary_tests::pure_header_container_payloads_recover_every_authored_property_and_fallback` (L12321): _no doc comment_
- `redefined_style_chain_source_owner_tests::stylesheet` (L12435): _no doc comment_
- `redefined_style_chain_source_owner_tests::root` (L12477): _no doc comment_
- `redefined_style_chain_source_owner_tests::redefined_style_metrics_bound_cycles_depth_and_default_owners_without_source_history_loss` (L12483): _no doc comment_
- `namespace_declared_default_property_owner_tests::locally_declared_namespace_metadata_never_becomes_a_paragraph_property_delta` (L12545): _no doc comment_
- `adopted_story_relationship_owner_tests::signature` (L12600): _no doc comment_
- `adopted_story_relationship_owner_tests::parsed` (L12634): _no doc comment_
- `adopted_story_relationship_owner_tests::empty_main` (L12640): _no doc comment_
- `adopted_story_relationship_owner_tests::adopted_root_and_nested_stories_keep_colliding_media_relationships_resolvable` (L12648): _no doc comment_
- `word_style_line_owned_boundary_tests::parse` (L12780): _no doc comment_
- `word_style_line_owned_boundary_tests::semantic` (L12786): _no doc comment_
- `word_style_line_owned_boundary_tests::word_single_line_style_propagation_keeps_authored_history_and_unrelated_properties` (L12808): _no doc comment_
- `public_numbering_definition_source_owner_tests::canonical` (L12878): _no doc comment_
- `public_numbering_definition_source_owner_tests::owned_projection` (L12910): Numbering IDs are package-local references, not formatting. _(from a code comment)_
- `public_numbering_definition_source_owner_tests::package` (L13010): _no doc comment_
- `public_numbering_definition_source_owner_tests::public_colliding_list_definitions_preserve_each_source_level_override_and_paragraph_history` (L13060): _no doc comment_
