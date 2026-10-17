# Functions and methods in `src/comparer/footnotes.rs`

29 non-test, 15 test. Part of the [symbol manifest](../README.md).

#### `set_id`  (L28-30)
`fn set_id(dom: &mut Dom, e: NodeId, v: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::set_attribute_value` (xmllinq/mod.rs:961) L29
- **Calls (std/external/unresolved):** `Some`, `W::id`
- **Called by:** `change_footnote_endnote_references_to_unique_range` (comparer/footnotes.rs:38)

#### `change_footnote_endnote_references_to_unique_range`  (L38-95, pub)
`pub fn change_footnote_endnote_references_to_unique_range( dom: &mut Dom, main_root: NodeId, fn_root: Option<NodeId>, en_root: Option<NodeId>, start: i32, has_log: bool, ) -> Result<Vec<String>, String>`

- **Does:** M4.H.4 — `ChangeFootnoteEndnoteReferencesToUniqueRange` (WmlComparer.ts :2135): renumber each `w:footnoteReference`/`w:endnoteReference` (document order) and its definition to a unique monotonic range from `start`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L48; `Dom::name` (xmllinq/mod.rs:526) L51,59; `Dom::attribute` (xmllinq/mod.rs:874) L57,68; `Dom::elements` (xmllinq/mod.rs:640) L66; `set_id` (comparer/footnotes.rs:28) L72,73; `XName::local_name` (xmllinq/mod.rs:208) (by name) L79,85; `Dom::remove` (xmllinq/mod.rs:1102) L92
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.collect`, `.filter`, `.find`, `.into_iter`, `.is_some_and`, `.push`, `.to_string`, `.unwrap_or`, `.unwrap`, `.zip`, `Err`, `Ok`, `Some`, `Vec::new`, `W::endnote`, `W::footnote`, `W::id`, `W::name`
- **Macros:** `format!`
- **Called by:** `pre_process_markup` (document_comparer.rs:4713)

#### `mandatory_separator_notes`  (L99-129, pub)
`pub fn mandatory_separator_notes(dom: &mut Dom, is_footnote: bool) -> Vec<NodeId>`

- **Does:** M4.H.4 — `MandatorySeparatorNotes` (:2077): the `separator` (id -1) and `continuationSeparator` (id 0) notes a footnotes/endnotes part must contain.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L107,110,111,112,118,119; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L108,109,113,114,115; `Dom::add` (xmllinq/mod.rs:1058) L116,117,120,121,122
- **Calls (std/external/unresolved):** `.clone`, `Some`, `W::endnote`, `W::footnote`, `W::id`, `W::name`, `W::p_pr`, `W::p`, `W::r`, `mk`
- **Macros:** `vec!`
- **Called by:** `deterministic_note_boundary_tests::both_separator_kinds_keep_exact_ids_spacing_and_marker_payload` (comparer/footnotes.rs:1323)

#### `fill_in_empty_footnotes_endnotes`  (L133-171, pub)
`pub fn fill_in_empty_footnotes_endnotes(dom: &mut Dom, notes_root: NodeId, is_footnote: bool)`

- **Does:** M4.H.4 — `FillInEmptyFootnotesEndnotes` (:1053): give every element-less `w:footnote`/`w:endnote` a stock reference paragraph.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L149; `Dom::has_elements` (xmllinq/mod.rs:862) L151; `Dom::new_element` (xmllinq/mod.rs:478) L154,155,156,160,161,162...; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L157,163; `Dom::add` (xmllinq/mod.rs:1058) L158,159,164,165,167,168...
- **Calls (std/external/unresolved):** `Some`, `W::endnote`, `W::footnote`, `W::name`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `W::val`
- **Called by:** `pre_process_markup` (document_comparer.rs:4713)

#### `content_contains_footnote_endnote_references_that_have_revisions`  (L176-212, pub)
`pub fn content_contains_footnote_endnote_references_that_have_revisions( dom: &Dom, element: NodeId, fn_root: Option<NodeId>, en_root: Option<NodeId>, ) -> bool`

- **Does:** M4.H.4 — `ContentContainsFootnoteEndnoteReferencesThatHaveRevisions` (:1101): true if any footnote/endnote referenced under `element` has a definition (in the delta parts) containing `w:ins`/`w:del`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L186,202; `Dom::name` (xmllinq/mod.rs:526) L187,205; `Dom::attribute` (xmllinq/mod.rs:874) L195,200; `Dom::elements` (xmllinq/mod.rs:640) L197
- **Calls (std/external/unresolved):** `.any`, `.find`, `.into_iter`, `.is_some_and`, `Some`, `W::del`, `W::endnote`, `W::footnote`, `W::id`, `W::ins`, `W::name`
- **Called by:** `deterministic_note_boundary_tests::referenced_endnote_deletion_counts_but_unreferenced_note_revisions_do_not` (comparer/footnotes.rs:1363)

#### `defined_style_ids`  (L217-225, pub)
`pub fn defined_style_ids(dom: &Dom, styles_root: NodeId) -> std::collections::HashSet<String>`

- **Does:** Collect every non-empty `w:styleId` defined under a `w:styles` root.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L220; `Dom::attribute` (xmllinq/mod.rs:874) L222
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.filter`, `.into_iter`, `.is_empty`, `.map`, `.to_string`, `Some`, `W::name`
- **Called by:** `defined_style_ids` (document_comparer.rs:4685); `compare_documents_impl` (document_comparer.rs:6501)

#### `strip_unresolved_style_refs`  (L233-255, pub)
`pub fn strip_unresolved_style_refs( dom: &mut Dom, root: NodeId, defined: &std::collections::HashSet<String>, ) -> usize`

- **Does:** Drop `w:pStyle` / `w:rStyle` whose `@w:val` is not in `defined`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L241; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L245; `Dom::remove` (xmllinq/mod.rs:1102) L250
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter`, `.into_iter`, `.is_empty`, `Some`, `W::name`, `W::val`
- **Macros:** `matches!`
- **Called by:** `pre_process_markup` (document_comparer.rs:4713); `compare_documents_impl` (document_comparer.rs:6501)

#### `copy_missing_styles`  (L267-287, pub)
`pub fn copy_missing_styles(dom: &mut Dom, to_root: NodeId, from_root: NodeId)`

- **Does:** M4.H.8 — `CopyMissingStylesFromOneDocToAnother` (WmlComparer.ts :2547): copy each `w:style` from `from_root` whose `styleId` `to_root` lacks, dropping its `w:default` attribute (so a copy doesn't add a second default style of the same kind).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L270,275; `Dom::attribute` (xmllinq/mod.rs:874) L273,277; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L283; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L284; `Dom::add` (xmllinq/mod.rs:1058) L285
- **Calls (std/external/unresolved):** `.collect`, `.insert`, `.into_iter`, `.map`, `.to_string`, `.unwrap_or`, `Some`, `W::name`
- **Called by:** `copy_missing_styles_by_name` (document_comparer.rs:3623); `compare_documents_impl` (document_comparer.rs:6501)

#### `add_numbering_child_in_schema_order`  (L292-315)
`fn add_numbering_child_in_schema_order(dom: &mut Dom, numbering: NodeId, child: NodeId)`

- **Does:** E.1 — `AddNumberingChildInSchemaOrder` (:2295): insert into `w:numbering` respecting `numPicBullet* abstractNum* num* numIdMacAtCleanup?` — place BEFORE the first existing child of a higher rank.
- **Calls (crate):** `add_numbering_child_in_schema_order::{fn rank}` (comparer/footnotes.rs:293) L306,310; `Dom::elements` (xmllinq/mod.rs:640) L307; `Dom::add_before_self` (xmllinq/mod.rs:1114) L312; `Dom::add` (xmllinq/mod.rs:1058) L313
- **Calls (std/external/unresolved):** `.find`, `.into_iter`
- **Called by:** `copy_missing_numbering` (comparer/footnotes.rs:364); `synthesize_dangling_numbering` (comparer/footnotes.rs:533)

#### `add_numbering_child_in_schema_order::{fn rank}`  (L293-305)
`fn rank(dom: &Dom, e: NodeId) -> i32`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L294; `XName::local_name` (xmllinq/mod.rs:208) (by name) L296
- **Calls (std/external/unresolved):** `.as_str`, `.map`, `.to_string`, `.unwrap_or_default`
- **Called by:** `add_numbering_child_in_schema_order` (comparer/footnotes.rs:292); `add_child_in_rank_order` (document_comparer.rs:2089) (?)

#### `get_int_attribute`  (L319-321)
`fn get_int_attribute(dom: &Dom, e: NodeId, name: &XName) -> Option<i32>`

- **Does:** E.1 — `GetIntAttribute` (:2315): parse an integer attribute, None when missing or unparseable.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L320
- **Calls (std/external/unresolved):** `.and_then`, `.ok`, `.parse`
- **Called by:** `copy_missing_numbering` (comparer/footnotes.rs:364); `synthesize_dangling_numbering` (comparer/footnotes.rs:533)

#### `normalized_abstract_num_signature`  (L326-344)
`fn normalized_abstract_num_signature(dom: &mut Dom, abstract_num: NodeId) -> String`

- **Does:** E.1 — `NormalizeAbstractNumForComparison` (:2331) + `XNode.DeepEquals`: clone minus the abstractNumId attribute and the (first) nsid/tmpl children, compared by serialization.
- **Calls (crate):** `Dom::clone_subtree` (xmllinq/mod.rs:1215) L327; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L328,333; `Dom::element` (xmllinq/mod.rs:655) L339; `Dom::remove` (xmllinq/mod.rs:1102) L340; `Dom::serialize_element` (xmllinq/mod.rs:1299) L343
- **Calls (std/external/unresolved):** `W::name`, `crate::namespaces::W15::name`
- **Called by:** `copy_missing_numbering` (comparer/footnotes.rs:364)

#### `copy_missing_numbering`  (L364-522, pub)
`pub fn copy_missing_numbering( dom: &mut Dom, to_root: NodeId, from_root: NodeId, ) -> (std::collections::HashMap<String, String>, Vec<NodeId>)`

- **Does:** E.1 — `CopyMissingNumberingFromOneDocToAnother` (WmlComparer.ts :2143): content-dedup (an `abstractNum` whose normalized content already exists is REUSED), id-remapped (colliding `abstractNumId` / `numId` get fresh ids past the destination maximum, with `num` references rewired), and inserted in schema order.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L375,381,391,404,410,424...; `get_int_attribute` (comparer/footnotes.rs:319) L378,384,394,405,419,427...; `normalized_abstract_num_signature` (comparer/footnotes.rs:326) L408,413; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L437,466,501,512; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L438,467,474,502,504,516; `add_numbering_child_in_schema_order` (comparer/footnotes.rs:292) L440,468,507,518; `Dom::descendants` (xmllinq/mod.rs:668) L441; `Dom::element` (xmllinq/mod.rs:655) L482,494,503,514
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.find`, `.get`, `.insert`, `.into_iter`, `.map`, `.max`, `.push`, `.saturating_add`, `.to_string`, `.unwrap_or`, `Some`, `Vec::new`, `W::name`, `W::val`, `std::collections::HashMap::new`
- **Called by:** `deterministic_note_boundary_tests::picture_bullet_copies_reuse_source_identity_and_advance_collision_watermark` (comparer/footnotes.rs:1610); `compare_documents_impl` (document_comparer.rs:6501)

#### `synthesize_dangling_numbering`  (L533-574, pub)
`pub fn synthesize_dangling_numbering(dom: &mut Dom, numbering_root: NodeId, dangling: &[String])`

- **Does:** Word-mode repair (beyond PowerTools): Word synthesizes a default decimal multilevel numbering definition when a document references a `w:numId` that no `w:num` defines — its document-open repair does exactly this (evidence: nested-table-rowspan_numbered-list — the revised fixture's dangling numId=2 gets a synthesized...
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L537; `get_int_attribute` (comparer/footnotes.rs:319) L540; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L559,569; `Dom::root` (xmllinq/mod.rs:822) L560,570; `add_numbering_child_in_schema_order` (comparer/footnotes.rs:292) L561,571
- **Calls (std/external/unresolved):** `.filter_map`, `.into_iter`, `.is_empty`, `.map_or`, `.max`, `.push_str`, `.saturating_add`, `Some`, `String::new`, `W::name`
- **Macros:** `format!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501)

#### `<RectifyError as std::fmt::Display>::fmt`  (L611-638, impl std::fmt::Display)
`fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`

- **Does:** _no doc comment_
- **Macros:** `write!`
- **Called by:** none found in src

#### `rectify_footnote_endnote_ids`  (L673-708, pub)
`pub fn rectify_footnote_endnote_ids( dom: &mut Dom, main_root: NodeId, footnotes: NotesSet, endnotes: NotesSet, settings: &WmlComparerSettings, id_gen: &mut u32, ) -> Result<(), RectifyError>`

- **Does:** M4.H.6/B.3 — `RectifyFootnoteEndnoteIds` (WmlComparer.ts :3309): in the output notes part keep only the separators (`-1` / `0`), then re-add the referenced definitions renumbered 1-based by reference order (after-part lookup wins over before), and finalize each with-revisions part: `MarkContentAsDeletedOrInserted` + `...
- **Calls (crate):** `plan_rectify` (comparer/footnotes.rs:745) L684,691; `apply_rectify` (comparer/footnotes.rs:798) L698,699; `finalize_notes_part` (comparer/footnotes.rs:716) L705
- **Calls (std/external/unresolved):** `.flatten`, `.into_iter`, `Ok`, `W::endnote`, `W::footnote`, `W::name`
- **Called by:** `deterministic_note_boundary_tests::rectification_reserves_notice_id_and_preserves_each_definition_payload` (comparer/footnotes.rs:1527); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `finalize_notes_part`  (L716-738)
`fn finalize_notes_part( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, id_gen: &mut u32, )`

- **Does:** The withRevisions notes-part finalization (C# :3433–:3450): MarkContentAsDeletedOrInserted (pt:Status → real `w:ins`/`w:del`), CoalesceAdjacentRunsWithIdenticalFormatting, WmlOrderElementsPerStandard (our produce-path equivalents: pPr-first + paragraph-mark revision order), IgnorePt14Namespace.
- **Calls (crate):** `mark_content_as_deleted_or_inserted` (comparer/finalize.rs:517) L722; `Dom::nodes` (xmllinq/mod.rs:604) L724,727; `Dom::remove` (xmllinq/mod.rs:1102) L725; `Dom::add` (xmllinq/mod.rs:1058) L728; `coalesce_all_paragraphs` (comparer/finalize.rs:1461) L731; `move_paragraph_properties_first` (comparer/finalize.rs:1524) L732; `fix_paragraph_mark_revision_order` (comparer/finalize.rs:1484) L733; `remove_powertools_scratch_markup` (comparer/finalize.rs:1018) L736; `ignore_pt14_namespace` (comparer/finalize.rs:819) L737
- **Called by:** `rectify_footnote_endnote_ids` (comparer/footnotes.rs:673); `paired_note_format_history_tests::paired_note_content_format_history_respects_word_and_faithful_modes` (comparer/footnotes.rs:1970)

#### `plan_rectify`  (L745-772)
`fn plan_rectify( dom: &Dom, main_root: NodeId, ref_name: &XName, def_name: &XName, notes: &NotesSet, ) -> Result<RectifyPlan, RectifyError>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L752; `Dom::attribute` (xmllinq/mod.rs:874) L755; `lookup_def` (comparer/footnotes.rs:774) L756,757
- **Calls (std/external/unresolved):** `.is_empty`, `.is_none`, `.len`, `.ok_or`, `.or_else`, `.push`, `.to_string`, `.unwrap_or`, `Err`, `Ok`, `Some`, `Vec::with_capacity`, `W::footnote`, `W::id`
- **Called by:** `rectify_footnote_endnote_ids` (comparer/footnotes.rs:673)

#### `lookup_def`  (L774-780)
`fn lookup_def(dom: &Dom, root: Option<NodeId>, def_name: &XName, old: &str) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L776; `Dom::attribute` (xmllinq/mod.rs:874) L778
- **Calls (std/external/unresolved):** `.and_then`, `.find`, `.into_iter`, `Some`, `W::id`
- **Called by:** `plan_rectify` (comparer/footnotes.rs:745); `deterministic_note_boundary_tests::referenced_endnote_deletion_counts_but_unreferenced_note_revisions_do_not` (comparer/footnotes.rs:1363); `deterministic_note_boundary_tests::rectification_reserves_notice_id_and_preserves_each_definition_payload` (comparer/footnotes.rs:1527)

#### `is_structural_note`  (L787-796, pub(crate))
`pub(crate) fn is_structural_note(dom: &Dom, note: NodeId) -> bool`

- **Does:** Structural (non-content) note definitions that must survive rectify.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L788; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L793
- **Calls (std/external/unresolved):** `Some`, `W::id`, `W::name`
- **Macros:** `matches!`
- **Called by:** `public_option_source_ownership_tests::public_comparison_options_preserve_reordered_paragraphs_formats_and_note_owners` (comparer/cross_para.rs:2775); `apply_rectify` (comparer/footnotes.rs:798); `deterministic_note_boundary_tests::both_separator_kinds_keep_exact_ids_spacing_and_marker_payload` (comparer/footnotes.rs:1323); `adopt_b_notes_when_a_lacks_separators` (document_comparer.rs:5863)

#### `apply_rectify`  (L798-846)
`fn apply_rectify(dom: &mut Dom, def_name: &XName, notes: &NotesSet, plan: &RectifyPlan)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L802,813; `is_structural_note` (comparer/footnotes.rs:787) L803,815; `Dom::remove` (xmllinq/mod.rs:1102) L804; `Dom::attribute` (xmllinq/mod.rs:874) L816; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L837,842; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L841; `Dom::add` (xmllinq/mod.rs:1058) L843
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.enumerate`, `.filter_map`, `.filter`, `.into_iter`, `.is_empty`, `.iter`, `.len`, `.map`, `.push`, `.to_string`, `.unwrap_or_default`, `Some`, `Vec::with_capacity`, `W::id`, `structural_ids`
- **Called by:** `rectify_footnote_endnote_ids` (comparer/footnotes.rs:673)

#### `sync_settings_special_note_ids`  (L852-878, pub)
`pub fn sync_settings_special_note_ids( settings_dom: &mut Dom, settings_root: NodeId, footnote_ids: &std::collections::HashSet<String>, endnote_ids: &std::collections::HashSet<String>, )`

- **Does:** Drop `w:footnote`/`w:endnote` children of `w:footnotePr`/`w:endnotePr` whose `@w:id` is not present in the corresponding notes part.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L864; `Dom::elements` (xmllinq/mod.rs:640) L865; `Dom::attribute` (xmllinq/mod.rs:874) L869; `Dom::remove` (xmllinq/mod.rs:1102) L874
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter`, `.into_iter`, `.is_empty`, `.unwrap_or`, `Some`, `W::id`, `W::name`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501)

#### `fix_up_footnotes_endnotes_with_custom_markers`  (L887-937, pub)
`pub fn fix_up_footnotes_endnotes_with_custom_markers(dom: &mut Dom, root: NodeId)`

- **Does:** M4.H.7 — `FixUpFootnotesEndnotesWithCustomMarkers` (WmlComparer.ts :2474 / :2484): a custom-marker footnote/endnote reference's `w:delText` / `w:t` must sit in the same run as the reference.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L891; `Dom::attribute` (xmllinq/mod.rs:874) L892; `Dom::parent` (xmllinq/mod.rs:597) L895,898; `Dom::name` (xmllinq/mod.rs:526) L899,902,921; `Dom::element` (xmllinq/mod.rs:655) L911,929; `Dom::next_element` (xmllinq/mod.rs:855) L914; `Dom::elements` (xmllinq/mod.rs:640) L926; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L933; `Dom::add` (xmllinq/mod.rs:1058) L934; `Dom::remove` (xmllinq/mod.rs:1102) L935
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.find_map`, `.flat_map`, `.into_iter`, `.is_some`, `.iter`, `Some`, `W::del`, `W::ins`, `W::name`, `W::r`, `W::t`
- **Called by:** `deterministic_note_boundary_tests::custom_markers_move_one_leaf_only_with_matching_revision_ownership` (comparer/footnotes.rs:1410); `coverage_custom_note_owner_boundaries::custom_marker_fixup_moves_only_matching_leaf_and_keeps_all_other_owners` (comparer/footnotes.rs:1709); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

#### `note_def_by_id`  (L942-946)
`fn note_def_by_id(dom: &Dom, root: NodeId, id: Option<&str>) -> Option<NodeId>`

- **Does:** Look up a note definition by id — C# `.Elements().FirstOrDefault(e => (string)e.Attribute(W.id) == id)`: ANY child element name, and a missing attribute matches a missing id (null == null).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L943; `Dom::attribute` (xmllinq/mod.rs:874) L945
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `W::id`
- **Called by:** `find_note_defs` (comparer/footnotes.rs:1051)

#### `replace_nodes`  (L950-957)
`fn replace_nodes(dom: &mut Dom, target: NodeId, source: NodeId)`

- **Does:** C# `ReplaceNodes`: clear `target`'s children, then append clones of `source`'s children.
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L951,954; `Dom::remove` (xmllinq/mod.rs:1102) L952; `Dom::add` (xmllinq/mod.rs:1058) L955
- **Called by:** `process_footnote_endnote` (comparer/footnotes.rs:1115)

#### `ensure_reference_marker`  (L967-1008)
`fn ensure_reference_marker(dom: &mut Dom, temp: NodeId, is_footnote: bool, direct_child_run: bool)`

- **Does:** The reference-marker guarantee (C# :3071–:3104, identical in all three branches): if the produced note content has no marker run, prepend a kind-correct one before the first run of the first paragraph.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L969,976,983,989; `Dom::elements` (xmllinq/mod.rs:640) L970,973; `Dom::attribute` (xmllinq/mod.rs:874) L974; `Dom::element` (xmllinq/mod.rs:655) L987; `Dom::new_element` (xmllinq/mod.rs:478) L999,1000,1001,1005; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1002; `Dom::add` (xmllinq/mod.rs:1058) L1003,1004,1006; `Dom::add_before_self` (xmllinq/mod.rs:1114) L1007
- **Calls (std/external/unresolved):** `.any`, `.copied`, `.find_map`, `.first`, `.flat_map`, `.into_iter`, `.is_empty`, `Some`, `W::name`, `W::p`, `W::r_pr`, `W::r`, `W::val`
- **Called by:** `produce_note_redline` (comparer/footnotes.rs:1015); `deterministic_note_boundary_tests::existing_note_reference_marker_and_nested_run_boundaries_are_respected` (comparer/footnotes.rs:1481)

#### `produce_note_redline`  (L1015-1039)
`fn produce_note_redline( dom: &mut Dom, flat: &mut [super::atoms::ComparisonUnitAtom], is_footnote: bool, direct_child_run: bool, settings: &WmlComparerSettings, id_gen: &mut u32, ) -> Option<NodeId>`

- **Does:** Shared tail of the three ProcessFootnoteEndnote branches: assemble unids, produce the redlined markup into a temp `w:body`, guarantee the reference marker, order per standard (our port's produce-path equivalents: pPr-first + paragraph-mark revision order — `WmlOrderElementsPerStandard` has no direct port), and return...
- **Calls (crate):** `assemble_ancestor_unids` (comparer/produce.rs:470) L1023; `produce_new_wml_markup_from_correlated_sequence` (comparer/produce.rs:1769) L1024; `Dom::new_element` (xmllinq/mod.rs:478) L1027; `Dom::add` (xmllinq/mod.rs:1058) L1029; `ensure_reference_marker` (comparer/footnotes.rs:967) L1031; `move_paragraph_properties_first` (comparer/finalize.rs:1524) L1032; `fix_paragraph_mark_revision_order` (comparer/finalize.rs:1484) L1033; `Dom::descendants` (xmllinq/mod.rs:668) L1036; `Dom::name` (xmllinq/mod.rs:526) L1038
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.is_some_and`, `W::body`, `W::endnote`, `W::footnote`
- **Called by:** `process_footnote_endnote` (comparer/footnotes.rs:1115)

#### `find_note_defs`  (L1051-1098)
`fn find_note_defs( dom: &Dom, notes: &super::NotesContext, is_footnote: bool, status: super::CorrelationStatus, before_id: Option<&str>, after_id: Option<&str>, ) -> Result<NoteDefs, RectifyError>`

- **Does:** Look up the definitions a reference's status processes: Equal needs both, Inserted the after one, Deleted the before one.
- **Calls (crate):** `note_def_by_id` (comparer/footnotes.rs:942) L1065
- **Calls (std/external/unresolved):** `.ok_or_else`, `.ok_or`, `.to_string`, `.unwrap_or_default`, `Err`, `Ok`, `Some`, `def`, `part`
- **Macros:** `format!`
- **Called by:** `process_footnote_endnote` (comparer/footnotes.rs:1115); `moved_reference_definition_owner_tests::moved_note_lookups_use_only_the_reference_source_or_destination_part` (comparer/footnotes.rs:1809)

#### `process_footnote_endnote`  (L1115-1277, pub)
`pub fn process_footnote_endnote( dom: &mut Dom, atoms: &[super::atoms::ComparisonUnitAtom], notes: &mut super::NotesContext, settings: &WmlComparerSettings, id_gen: &mut u32, ) -> Result<(), RectifyError>`

- **Does:** B.2 — `ProcessFootnoteEndnote` (:2944): for every footnote/endnote reference atom in the correlated body, process its definition keyed by the REFERENCE's correlation status — Equal → nested mini-compare of the two definitions written into the AFTER definition; Inserted → after-definition content re-emitted all-Inserte...
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L1131,1149; `Dom::attribute` (xmllinq/mod.rs:874) L1146,1148; `reordered_retained_control_identity_regression::ids` (comparer/produce.rs:2434) L1153,1165; `find_note_defs` (comparer/footnotes.rs:1051) L1154,1167; `add_sha1_hash_to_block_level_content` (comparer/preprocess.rs:1259) L1181,1187,1222,1252; `create_comparison_unit_atom_list` (comparer/atomize.rs:307) L1193,1195,1228,1258; `get_comparison_unit_list` (comparer/units.rs:140) L1194,1196,1229,1259; `lcs` (comparer/lcs.rs:10084) L1202; `mark_rows_as_deleted_or_inserted` (comparer/lcs_table.rs:705) L1203,1231,1261; `flatten_to_comparison_unit_atom_list` (comparer/produce.rs:225) L1204,1232,1262; `detect_format_changes_in_atom_list` (comparer/formatchg.rs:383) L1206; `produce_note_redline` (comparer/footnotes.rs:1015) L1211,1236,1268; `replace_nodes` (comparer/footnotes.rs:950) L1215,1238,1270; `CorrelatedSequence::inserted` (comparer/atoms.rs:527) L1230; `CorrelatedSequence::deleted` (comparer/atoms.rs:518) L1260
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.clone`, `.collect`, `.filter`, `.is_empty`, `.is_some_and`, `.iter`, `.map`, `.ok_or_else`, `.unwrap_or_default`, `Ok`, `W::id`, `W::name`
- **Macros:** `vec!`
- **Called by:** `paired_note_format_history_tests::paired_note_content_format_history_respects_word_and_faithful_modes` (comparer/footnotes.rs:1970); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

## Test functions (15)

- `deterministic_note_boundary_tests::parse` (L1284): _no doc comment_
- `deterministic_note_boundary_tests::properties` (L1291): _no doc comment_
- `deterministic_note_boundary_tests::settings` (L1314): _no doc comment_
- `deterministic_note_boundary_tests::both_separator_kinds_keep_exact_ids_spacing_and_marker_payload` (L1323): _no doc comment_
- `deterministic_note_boundary_tests::referenced_endnote_deletion_counts_but_unreferenced_note_revisions_do_not` (L1363): _no doc comment_
- `deterministic_note_boundary_tests::custom_markers_move_one_leaf_only_with_matching_revision_ownership` (L1410): _no doc comment_
- `deterministic_note_boundary_tests::existing_note_reference_marker_and_nested_run_boundaries_are_respected` (L1481): _no doc comment_
- `deterministic_note_boundary_tests::rectification_reserves_notice_id_and_preserves_each_definition_payload` (L1527): _no doc comment_
- `deterministic_note_boundary_tests::picture_bullet_copies_reuse_source_identity_and_advance_collision_watermark` (L1610): _no doc comment_
- `coverage_custom_note_owner_boundaries::custom_marker_fixup_moves_only_matching_leaf_and_keeps_all_other_owners` (L1709): _no doc comment_
- `moved_reference_definition_owner_tests::moved_note_lookups_use_only_the_reference_source_or_destination_part` (L1809): _no doc comment_
- `paired_note_format_history_tests::parse` (L1915): _no doc comment_
- `paired_note_format_history_tests::signature` (L1919): _no doc comment_
- `paired_note_format_history_tests::events` (L1941): _no doc comment_
- `paired_note_format_history_tests::paired_note_content_format_history_respects_word_and_faithful_modes` (L1970): _no doc comment_
