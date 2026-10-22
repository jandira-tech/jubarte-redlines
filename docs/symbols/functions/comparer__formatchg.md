# Functions and methods in `src/comparer/formatchg.rs`

20 non-test, 11 test. Part of the [symbol manifest](../README.md).

#### `get_run_properties_from_atom`  (L31-38, pub)
`pub fn get_run_properties_from_atom(dom: &Dom, atom: &ComparisonUnitAtom) -> Option<NodeId>`

- **Does:** Nearest `w:r` ancestor's `w:rPr` element for an atom, or `None` if no such ancestor exists.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L36; `Dom::element` (xmllinq/mod.rs:655) L37
- **Calls (std/external/unresolved):** `.copied`, `.find`, `.iter`, `W::r_pr`, `W::r`
- **Called by:** `detect_format_changes_impl` (comparer/formatchg.rs:406)

#### `is_rsid_attr`  (L40-42)
`fn is_rsid_attr(n: &XName) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `XName::local_name` (xmllinq/mod.rs:208) L41
- **Calls (std/external/unresolved):** `.starts_with`, `.to_lowercase`
- **Called by:** `canonical_rpr_spec` (comparer/formatchg.rs:53); `prop_signature` (comparer/formatchg.rs:194)

#### `canonical_rpr_spec`  (L53-84)
`fn canonical_rpr_spec(src: &Dom, rpr: Option<NodeId>) -> Vec<CanonicalRprChild>`

- **Does:** `NormalizeRunProperties` (read half) — extract `rpr`'s canonical child spec from `src` with PURE READS: drop `w:rPrChange` + pt-ns children; children sorted by local name; each child keeps name + filtered (no rsid*, no pt) attrs sorted by local name; nested sub-elements dropped.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L56; `Dom::name` (xmllinq/mod.rs:526) L60,68,69,73; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L61,77; `XName::local_name` (xmllinq/mod.rs:208) (by name) L70,79; `Dom::attributes` (xmllinq/mod.rs:899) L74; `is_rsid_attr` (comparer/formatchg.rs:40) L77
- **Calls (std/external/unresolved):** `.cmp`, `.collect`, `.filter`, `.into_iter`, `.push`, `.sort_by`, `.unwrap`, `Vec::new`, `W::name`
- **Called by:** `normalize_run_properties` (comparer/formatchg.rs:110); `normalized_rpr_serialized` (comparer/formatchg.rs:132)

#### `build_canonical_rpr`  (L90-100)
`fn build_canonical_rpr(dst: &mut Dom, spec: &[CanonicalRprChild]) -> NodeId`

- **Does:** `NormalizeRunProperties` (build half) — materialize a canonical `w:rPr` from a spec into `dst`, returning its NodeId.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L91,93; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L95; `Dom::add` (xmllinq/mod.rs:1074) L97
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `Some`, `W::r_pr`
- **Called by:** `normalize_run_properties` (comparer/formatchg.rs:110); `normalized_rpr_serialized` (comparer/formatchg.rs:132)

#### `normalize_run_properties`  (L110-113, pub)
`pub fn normalize_run_properties(dom: &mut Dom, rpr: Option<NodeId>) -> NodeId`

- **Does:** Build a canonical `w:rPr` in the same arena `dom`: drop `w:rPrChange` and any `pt:`-namespace children, sort the surviving children by local name, and sort each child's attributes the same way.
- **Calls (crate):** `canonical_rpr_spec` (comparer/formatchg.rs:53) L111; `build_canonical_rpr` (comparer/formatchg.rs:90) L112
- **Called by:** `are_run_properties_equal` (comparer/formatchg.rs:118); `format_change_cache_tests::direct` (comparer/formatchg.rs:558)

#### `are_run_properties_equal`  (L118-122, pub)
`pub fn are_run_properties_equal(dom: &mut Dom, a: Option<NodeId>, b: Option<NodeId>) -> bool`

- **Does:** True when two `w:rPr` elements serialize to the same canonical form (after [`normalize_run_properties`]).
- **Calls (crate):** `normalize_run_properties` (comparer/formatchg.rs:110) L119,120; `Dom::serialize_element` (xmllinq/mod.rs:1315) L121
- **Called by:** `paragraph_mark_properties_equal` (comparer/finalize.rs:3322); `detect_format_changes_impl` (comparer/formatchg.rs:406)

#### `normalized_rpr_serialized`  (L132-157)
`fn normalized_rpr_serialized( dom: &Dom, scratch: &mut Dom, cache: &mut std::collections::HashMap<Option<NodeId>, String>, rpr: Option<NodeId>, ) -> String`

- **Does:** Cached `serialize(normalize_run_properties(rpr))` keyed by the `rPr` NodeId.
- **Calls (crate):** `canonical_rpr_spec` (comparer/formatchg.rs:53) L150; `Dom::with_scratch` (xmllinq/mod.rs:449) L151; `build_canonical_rpr` (comparer/formatchg.rs:90) L152; `Dom::serialize_element` (xmllinq/mod.rs:1315) (by name) L153
- **Calls (std/external/unresolved):** `.clone`, `.get`, `.insert`
- **Called by:** `detect_format_changes_impl` (comparer/formatchg.rs:406); `format_change_cache_tests::format_changes_cache_matches_direct` (comparer/formatchg.rs:564); `format_change_cache_tests::normalized_rpr_serialization_reclaims_scratch_nodes` (comparer/formatchg.rs:608)

#### `friendly_property_name`  (L162-192, pub)
`pub fn friendly_property_name(local: &str) -> String`

- **Does:** Map a WordprocessingML run-property local name to the friendly name the Python and WASM revisions-binding returns (`bold`, `italic`, `strikethrough`, `fontSize`, `color`, …).
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `get_changed_property_names` (comparer/formatchg.rs:217); `extract_property_dictionary` (comparer/revisions.rs:303)

#### `prop_signature`  (L194-212)
`fn prop_signature(dom: &mut Dom, prop: NodeId) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L199; `Dom::new_element` (xmllinq/mod.rs:478) L200; `Dom::attributes` (xmllinq/mod.rs:899) L201; `is_rsid_attr` (comparer/formatchg.rs:40) L204; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L204; `XName::local_name` (xmllinq/mod.rs:208) (by name) L207; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L209; `Dom::serialize_element` (xmllinq/mod.rs:1315) L211
- **Calls (std/external/unresolved):** `.cmp`, `.collect`, `.filter`, `.into_iter`, `.sort_by`, `.unwrap`, `Some`
- **Called by:** `get_changed_property_names` (comparer/formatchg.rs:217); `para_property_signature` (comparer/formatchg.rs:297)

#### `get_changed_property_names`  (L217-250, pub)
`pub fn get_changed_property_names( dom: &mut Dom, old: Option<NodeId>, new: Option<NodeId>, ) -> Vec<String>`

- **Does:** Friendly names of the run-property children that differ between `old` and `new` (presence or value, by canonical serialization).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L225; `Dom::name` (xmllinq/mod.rs:526) L226; `XName::clark` (xmllinq/mod.rs:226) (by name) L228; `collect` (ambiguous: `tree_scope::{fn collect}` (markdown/from_docx/drawing.rs:472); `collect` (markdown/from_docx/header.rs:1467)) L234,235; `prop_signature` (comparer/formatchg.rs:194) L242; `friendly_property_name` (comparer/formatchg.rs:162) L246; `XName::local_name` (xmllinq/mod.rs:208) (by name) L246; `XName::from_clark` (xmllinq/mod.rs:195) L246
- **Calls (std/external/unresolved):** `.chain`, `.cloned`, `.collect`, `.dedup`, `.get`, `.insert`, `.keys`, `.push`, `.sort`, `.unwrap`, `BTreeMap::new`, `Vec::new`, `W::name`
- **Called by:** `detect_format_changes_impl` (comparer/formatchg.rs:406)

#### `is_para_comparison_noise`  (L259-267)
`fn is_para_comparison_noise(dom: &Dom, child: NodeId) -> bool`

- **Does:** Children of `w:pPr` that do not participate in paragraph-format comparison (docxodus `IsParaComparisonNoiseChild`).
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L260; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L266
- **Calls (std/external/unresolved):** `W::name`, `W::r_pr`
- **Called by:** `projected_ppr_jc` (comparer/formatchg.rs:270); `normalize_para_properties_without_jc` (comparer/formatchg.rs:313); `project_para_properties_for_change` (comparer/formatchg.rs:329); `normalize_para_properties` (comparer/formatchg.rs:349)

#### `projected_ppr_jc`  (L270-283)
`fn projected_ppr_jc(dom: &Dom, ppr: NodeId) -> Option<NodeId>`

- **Does:** First non-default `w:jc` child of a projected pPr, if any.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L271; `is_para_comparison_noise` (comparer/formatchg.rs:259) L272; `Dom::name_is` (xmllinq/mod.rs:535) L275; `Dom::attribute` (xmllinq/mod.rs:874) L276
- **Calls (std/external/unresolved):** `.unwrap_or`, `Some`, `W::name`, `W::val`
- **Called by:** `project_jc_only_from` (comparer/formatchg.rs:286); `detect_format_changes_impl` (comparer/formatchg.rs:406)

#### `project_jc_only_from`  (L286-292)
`fn project_jc_only_from(dom: &mut Dom, ppr: NodeId) -> Option<NodeId>`

- **Does:** Project only the non-default `w:jc` from old pPr (justify/center removal class).
- **Calls (crate):** `projected_ppr_jc` (comparer/formatchg.rs:270) L287; `Dom::new_element` (xmllinq/mod.rs:478) L288; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L289; `Dom::add` (xmllinq/mod.rs:1074) L290
- **Calls (std/external/unresolved):** `Some`, `W::p_pr`
- **Called by:** `detect_format_changes_impl` (comparer/formatchg.rs:406)

#### `para_property_signature`  (L297-310)
`fn para_property_signature(dom: &mut Dom, property: NodeId) -> String`

- **Does:** Paragraph properties such as numPr, tabs and pBdr own nested values.
- **Calls (crate):** `prop_signature` (comparer/formatchg.rs:194) L298; `Dom::elements` (xmllinq/mod.rs:640) L299; `Dom::name` (xmllinq/mod.rs:526) L300; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L302; `para_property_signature` (comparer/formatchg.rs:297) L306
- **Calls (std/external/unresolved):** `.is_some_and`, `.len`, `.push_str`
- **Macros:** `format!`
- **Called by:** `para_property_signature` (comparer/formatchg.rs:297); `normalize_para_properties_without_jc` (comparer/formatchg.rs:313); `normalize_para_properties` (comparer/formatchg.rs:349)

#### `normalize_para_properties_without_jc`  (L313-326)
`fn normalize_para_properties_without_jc(dom: &mut Dom, ppr: NodeId) -> String`

- **Does:** Signature of projected pPr with jc children ignored (for partial-removal gate).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L315; `is_para_comparison_noise` (comparer/formatchg.rs:259) L316; `Dom::name_is` (xmllinq/mod.rs:535) L319; `para_property_signature` (comparer/formatchg.rs:297) L322
- **Calls (std/external/unresolved):** `.join`, `.push`, `.sort`, `Vec::new`, `W::name`
- **Called by:** `detect_format_changes_impl` (comparer/formatchg.rs:406)

#### `project_para_properties_for_change`  (L329-346, pub(crate))
`pub(crate) fn project_para_properties_for_change(dom: &mut Dom, ppr: NodeId) -> NodeId`

- **Does:** Project old-side pPr children for `w:pPrChange` (CT_PPrBase noise-stripped).
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L330; `Dom::elements` (xmllinq/mod.rs:640) L331; `is_para_comparison_noise` (comparer/formatchg.rs:259) L332; `Dom::name_is` (xmllinq/mod.rs:535) L336; `Dom::attribute` (xmllinq/mod.rs:874) L337; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L342; `Dom::add` (xmllinq/mod.rs:1074) L343
- **Calls (std/external/unresolved):** `.unwrap_or`, `W::name`, `W::p_pr`, `W::val`
- **Called by:** `merged_closing_carrier_takes_source_properties` (comparer/finalize.rs:3122); `closing_mark_takes_revised_properties` (comparer/finalize.rs:3200); `last_pure_del_spacing_to_pprchange` (comparer/finalize.rs:5205); `fold_midstream_demo_title_into_numbered_heading_with_source_properties` (comparer/finalize.rs:12624); `ParagraphPropertySources::capture` (comparer/finalize.rs:13105); `detect_format_changes_impl` (comparer/formatchg.rs:406)

#### `normalize_para_properties`  (L349-365, pub(crate))
`pub(crate) fn normalize_para_properties(dom: &mut Dom, ppr: NodeId) -> String`

- **Does:** Canonical form of comparable pPr children for equality (local name + attrs).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L351; `is_para_comparison_noise` (comparer/formatchg.rs:259) L352; `Dom::name_is` (xmllinq/mod.rs:535) L355; `Dom::attribute` (xmllinq/mod.rs:874) L356; `para_property_signature` (comparer/formatchg.rs:297) L361
- **Calls (std/external/unresolved):** `.join`, `.push`, `.sort`, `.unwrap_or`, `Vec::new`, `W::name`, `W::val`
- **Called by:** `merged_closing_carrier_takes_source_properties` (comparer/finalize.rs:3122); `closing_mark_takes_revised_properties` (comparer/finalize.rs:3200); `fold_midstream_demo_title_into_numbered_heading_with_source_properties` (comparer/finalize.rs:12624); `park_jc_on_first_short_title_mix_from_body` (comparer/finalize.rs:13148); `parked_paragraph_history_regressions::parked_layout_restores_properties_that_remain_live_when_rejected` (comparer/finalize.rs:23617); `closing_layout_history_regressions::already_revised_live_closing_layout_still_records_original_properties` (comparer/finalize.rs:23679); `split_paragraph_boundary_regressions::mixed_inserted_boundary_joins_original_text_and_closing_layout_on_reject` (comparer/finalize.rs:23752); `are_para_properties_equal` (comparer/formatchg.rs:367); `detect_format_changes_impl` (comparer/formatchg.rs:406)

#### `are_para_properties_equal`  (L367-369)
`fn are_para_properties_equal(dom: &mut Dom, a: NodeId, b: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `normalize_para_properties` (comparer/formatchg.rs:349) L368
- **Called by:** `detect_format_changes_impl` (comparer/formatchg.rs:406)

#### `detect_format_changes_in_atom_list`  (L383-389, pub)
`pub fn detect_format_changes_in_atom_list( dom: &mut Dom, atoms: &mut [ComparisonUnitAtom], settings: &WmlComparerSettings, )`

- **Does:** M4.G.5 — `DetectFormatChangesInAtomList` (WmlComparer.ts :4824 / docxodus :7111): - `Equal` run atoms whose `w:rPr` differs → `FormatChanged` with the original's `w:rPr` serialized as `pt:OldRPr`.
- **Calls (crate):** `detect_format_changes_impl` (comparer/formatchg.rs:406) L388
- **Called by:** `process_footnote_endnote` (comparer/footnotes.rs:1115); `format_change_cache_tests::detect_format_changes_never_grows_production_arena_capacity` (comparer/formatchg.rs:651); `format_change_cache_tests::cached_format_changes_match_uncached` (comparer/formatchg.rs:710); `nested_paragraph_property_source_history_tests::nested_list_tab_and_border_values_record_complete_original_paragraph_properties` (comparer/formatchg.rs:795); `horizontal_partition_tests::actual_atomized_split_with_deleted_table_conserves_source_through_production` (comparer/lcs_table.rs:1186); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215)

#### `detect_format_changes_impl`  (L406-535)
`fn detect_format_changes_impl( dom: &mut Dom, atoms: &mut [ComparisonUnitAtom], settings: &WmlComparerSettings, use_cache: bool, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L425; `Dom::name_is` (xmllinq/mod.rs:535) L438,441,511,517; `are_para_properties_equal` (comparer/formatchg.rs:367) L441; `project_para_properties_for_change` (comparer/formatchg.rs:329) L443,444; `normalize_para_properties` (comparer/formatchg.rs:349) L445,446; `Dom::new_element` (xmllinq/mod.rs:478) L457; `projected_ppr_jc` (comparer/formatchg.rs:270) L459,460; `normalize_para_properties_without_jc` (comparer/formatchg.rs:313) L461,462; `project_jc_only_from` (comparer/formatchg.rs:286) L468; `get_run_properties_from_atom` (comparer/formatchg.rs:31) L480,481; `normalized_rpr_serialized` (comparer/formatchg.rs:132) L487,488; `are_run_properties_equal` (comparer/formatchg.rs:118) L490,522; `get_changed_property_names` (comparer/formatchg.rs:217) L493; `Dom::element` (xmllinq/mod.rs:655) L512,518
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.clone`, `.enumerate`, `.into`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.push`, `Some`, `Vec::new`, `W::p_pr`, `W::r_pr`, `std::collections::HashMap::new`
- **Macros:** `vec!`
- **Called by:** `detect_format_changes_in_atom_list` (comparer/formatchg.rs:383); `detect_format_changes_reference` (comparer/formatchg.rs:398)

## Test functions (11)

- `detect_format_changes_reference` (L398): Uncached oracle (pre-PR4 behaviour: `are_run_properties_equal` directly).
- `format_change_cache_tests::rpr_with` (L546): _no doc comment_
- `format_change_cache_tests::direct` (L558): _no doc comment_
- `format_change_cache_tests::format_changes_cache_matches_direct` (L564): _no doc comment_
- `format_change_cache_tests::normalized_rpr_serialization_reclaims_scratch_nodes` (L608): MEM-ATTRIBUTE-01 regression: normalizing an `rPr` is SCRATCH work whose only output is a `String`, so it must not leave throwaway nodes in the persistent arena.
- `format_change_cache_tests::detect_format_changes_never_grows_production_arena_capacity` (L651): FMT-SCRATCH-02 (MEM-ATTRIBUTE-01): rPr normalization must not merely RECLAIM its throwaway nodes from the production arena (that is FMT-SCRATCH-01) — it must never ALLOCATE into it at all.
- `format_change_cache_tests::run_atom` (L693): An Equal run atom whose nearest `w:r` carries a `w:rPr` with the given children — so `get_run_properties_from_atom` finds it.
- `format_change_cache_tests::cached_format_changes_match_uncached` (L710): The direct guard the review asked for: the cached run-property comparison must produce the EXACT same format-change retagging (status + changed property names) as the uncached `are_run_properties_equal` path, on the same atoms — covering adds, removes, value changes, reorders, and no-ops.
- `nested_paragraph_property_source_history_tests::owned_mark` (L775): _no doc comment_
- `nested_paragraph_property_source_history_tests::clean` (L788): _no doc comment_
- `nested_paragraph_property_source_history_tests::nested_list_tab_and_border_values_record_complete_original_paragraph_properties` (L795): _no doc comment_
