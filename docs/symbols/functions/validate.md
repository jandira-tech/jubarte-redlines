# Functions and methods in `src/validate.rs`

62 non-test, 29 test. Part of the [symbol manifest](../README.md).

#### `Finding::new`  (L118-138, pub)
`pub fn new( code: &str, part: &str, location: impl Into<String>, message: impl Into<String>, ) -> Self`

- **Does:** A `word` finding with `word_fatal` and `repairable` taken from the code's table ([`traits`]) and the severity from `word_fatal`.
- **Calls (crate):** `traits` (validate.rs:143) L124
- **Calls (std/external/unresolved):** `.clone`, `.into`, `.to_string`
- **Called by:** `findings` (debug.rs:2397); `check_style_names` (validate.rs:466); `check_runs_inside_paragraphs` (validate.rs:499); `check_content_types_and_xml` (validate.rs:596); `check_relationship_integrity` (validate.rs:646); `check_picture_bullets` (validate.rs:743); `collect_ids` (validate.rs:788); `check_para_text_id_bounds` (validate.rs:877); `check_del_text_under_del` (validate.rs:898); `check_deleted_text_has_deletion` (validate.rs:953); `check_bookmarks_outside_single_value_controls` (validate.rs:1017); `comment_finding` (validate.rs:1092); +4 more

#### `traits`  (L143-177, pub)
`pub fn traits(code: &str) -> (bool, bool)`

- **Does:** `(word_fatal, repairable)` for a code; an unknown code is a lead with no repair.
- **Calls:** none
- **Called by:** `Finding::new` (validate.rs:118)

#### `<ValidateError as fmt::Display>::fmt`  (L227-236, impl fmt::Display)
`fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`

- **Does:** _no doc comment_
- **Macros:** `write!`
- **Called by:** none found in src

#### `<ValidateError as From>::from`  (L242-244, impl From)
`fn from(e: AdmissionError) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Self::Refused`
- **Called by:** none found in src

#### `<ValidateError as From>::from`  (L248-250, impl From)
`fn from(e: ChangeError) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Self::Changes`
- **Called by:** none found in src

#### `<ValidateError as From>::from`  (L254-256, impl From)
`fn from(e: InspectError) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Self::Inspect`
- **Called by:** none found in src

#### `<ValidateError as From>::from`  (L260-262, impl From)
`fn from(e: crate::opc::OpcError) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.to_string`, `Self::Package`
- **Called by:** none found in src

#### `validate_with`  (L290-331, pub)
`pub fn validate_with( docx: &[u8], rules: &[&str], original: Option<(&[u8], &str)>, layout: Option<crate::audit::LayoutPass<'_>>, ) -> Result<Vec<Finding>, ValidateError>`

- **Does:** [`validate`] with rule sets: `word` (the default), `edits`, and the [`crate::audit`] sets or codes (`a11y`, `style`, `structure`, `LITERAL_BULLET`, ...).
- **Calls (crate):** `is_rule` (audit.rs:156) L303; `unknown_rule` (validate.rs:334) L304,325; `validate` (validate.rs:420) L309; `audit_tracked` (validate.rs:2036) L317; `audit_report_with` (audit.rs:177) L321
- **Calls (std/external/unresolved):** `.extend`, `.into_iter`, `.is_empty`, `.is_some`, `.map_err`, `.map`, `.push`, `.to_string`, `Err`, `Ok`, `ValidateError::Inspect`, `ValidateError::Layout`, `ValidateError::Usage`, `Vec::new`
- **Called by:** `run_validate` (bin/jubarte.rs:2793)

#### `unknown_rule`  (L334-341)
`fn unknown_rule(rule: &str) -> String`

- **Does:** The usage message for a rule that is neither a set nor an audit code.
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.join`, `.map`
- **Macros:** `format!`
- **Called by:** `validate_with` (validate.rs:290)

#### `fails`  (L345-351, pub)
`pub fn fails(findings: &[Finding], strict: bool) -> bool`

- **Does:** True when `findings` fail a run: any `word` or `edits` finding, any `error`, or (`strict`) any `warning`.
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.iter`
- **Macros:** `matches!`
- **Called by:** `run_validate` (bin/jubarte.rs:2793)

#### `findings_text`  (L359-386, pub)
`pub fn findings_text(findings: &[Finding]) -> String`

- **Does:** One line per finding, then a summary.
- **Calls (crate):** `count` (ambiguous: `count` (cli.rs:1741); `count` (markdown/from_docx/agent.rs:1227)) L380,381,382
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.count`, `.filter`, `.is_empty`, `.iter`, `.len`, `.push_str`, `String::new`
- **Macros:** `format!`, `matches!`
- **Called by:** `run_validate` (bin/jubarte.rs:2793)

#### `<Finding as From>::from`  (L392-410, impl From)
`fn from(f: crate::audit::AuditFinding) -> Self`

- **Does:** The audit's finding in the shared shape: its part is the story's part for a paragraph id (`p3` → `word/document.xml`), else the location itself (a part name); it is never Word-fatal or repairable.
- **Calls (crate):** `paragraph_place` (ids.rs:417) L393; `story_part` (validate.rs:2016) L394
- **Calls (std/external/unresolved):** `.clone`, `.is_some`
- **Called by:** none found in src

#### `validate`  (L420-441, pub)
`pub fn validate(docx: &[u8]) -> Result<Vec<Finding>, ValidateError>`

- **Does:** Findings sorted by part, location, code and message (the same order every run); an empty vector is a pass.
- **Calls (crate):** `admit` (admission.rs:310) L421; `<InputLimits as Default>::default` (admission.rs:56) L421; `PartFs::open` (opc/mod.rs:456) L422; `ring1` (validate.rs:445) L423; `findings` (debug.rs:2397) L424
- **Calls (std/external/unresolved):** `.as_str`, `.cmp`, `.contains`, `.extend`, `.filter`, `.into_iter`, `.map_err`, `.sort_by`, `Ok`
- **Called by:** `scrub` (scrub.rs:109); `validate_with` (validate.rs:290); `repair` (validate.rs:1649); `public_memory_validator_owner_boundary_tests::missing_opaque_part_type_has_exact_public_diagnostics_and_no_speculative_repair` (validate.rs:2459); `public_memory_validator_owner_boundary_tests::repairing_comment_paragraph_ids_retains_reply_graph_source_text_and_all_relationships` (validate.rs:2498); `retained_namespace_finding_source_contract_tests::repair_reports_unbound_unknown_prefixes_without_discarding_owned_story_content` (validate.rs:2879); `duplicate_drawing_owned_repair_contract_tests::duplicate_drawing_repair_renumbers_only_ids_and_preserves_every_shape_owner` (validate.rs:2982)

#### `ring1`  (L445-460, pub)
`pub fn ring1(pkg: &PartFs) -> Vec<Finding>`

- **Does:** The Ring-1 invariants alone, on an opened package: no admission, no triage checks.
- **Calls (crate):** `check_content_types_and_xml` (validate.rs:596) L447; `check_relationship_integrity` (validate.rs:646) L448; `check_picture_bullets` (validate.rs:743) L449; `check_revision_and_drawing_ids` (validate.rs:767) L450; `check_para_text_id_bounds` (validate.rs:877) L451; `check_del_text_under_del` (validate.rs:898) L452; `check_deleted_text_has_deletion` (validate.rs:953) L453; `check_bookmarks_outside_single_value_controls` (validate.rs:1017) L454; `check_comment_graph` (validate.rs:1096) L455; `check_namespace_qname_contexts` (validate.rs:590) L456; `check_runs_inside_paragraphs` (validate.rs:499) L457; `check_style_names` (validate.rs:466) L458
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `row_nontext_ownership_regressions::public_compare_keeps_unchanged_tab_and_break_cells_when_text_moves_into_their_row` (comparer/finalize.rs:23873); `coverage_real_revision_merge_matrix_tests::public_compare_multi_original_blank_tail_recovers_complete_clean_sources` (comparer/finalize.rs:26091); `tests::empty_paragraphs_are_kept_in_the_docx_and_left_out_of_markdown` (legacy_doc.rs:2090); `tests::only_rows_marked_as_headers_repeat_as_headers` (legacy_doc.rs:2207); `tests::a_blank_paragraph_keeps_its_line_breaks_in_the_docx` (legacy_doc.rs:2368); `tests::a_heading_keeps_its_line_breaks_in_the_docx` (legacy_doc.rs:2399); `tests::title_and_headings_7_to_9_keep_their_word_styles` (legacy_doc.rs:2528); `validate` (validate.rs:420); `public_memory_validator_owner_boundary_tests::missing_opaque_part_type_has_exact_public_diagnostics_and_no_speculative_repair` (validate.rs:2459); `authored_comment_family_validation_contract_tests::auxiliary_comment_parts_without_their_source_definition_are_reported_individually` (validate.rs:2761)

#### `check_style_names`  (L466-492)
`fn check_style_names(pkg: &PartFs, out: &mut Vec<Finding>)`

- **Does:** A style's `w:name` holds no comma: the schema's pattern is `[^,]*` (Word reads a comma there as the start of an alias list), while its `w:styleId` may hold one.
- **Calls (crate):** `xml_parts` (validate.rs:578) L467; `parse_part` (validate.rs:570) L468; `XName::local_name` (xmllinq/mod.rs:208) (by name) L471,478; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L474; `Dom::parent` (xmllinq/mod.rs:597) (by name) L475; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L479; `Finding::new` (validate.rs:118) L483; `element_path` (validate.rs:546) L486
- **Calls (std/external/unresolved):** `.and_then`, `.contains`, `.is_none_or`, `.is_some_and`, `.name`, `.push`, `Some`, `W::name`, `W::val`
- **Macros:** `format!`
- **Called by:** `ring1` (validate.rs:445)

#### `check_runs_inside_paragraphs`  (L499-542)
`fn check_runs_inside_paragraphs(pkg: &PartFs, out: &mut Vec<Finding>)`

- **Does:** A run outside any paragraph: Word loads the document empty.
- **Calls (crate):** `xml_parts` (validate.rs:578) L513; `parse_part` (validate.rs:570) L514; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L517; `Dom::parent` (xmllinq/mod.rs:597) (by name) L518,530; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L521,526; `Finding::new` (validate.rs:118) L533; `element_path` (validate.rs:546) L536
- **Calls (std/external/unresolved):** `.any`, `.iter`, `.push`, `Some`, `W::name`, `W::p`, `W::r`
- **Macros:** `format!`
- **Called by:** `ring1` (validate.rs:445)

#### `element_path`  (L546-568)
`fn element_path(dom: &Dom, node: NodeId) -> String`

- **Does:** `w:body[0]/w:p[3]/w:r[2]`: conventional prefix and local name with the index among same-named siblings, root excluded.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L550,551; `Dom::name` (xmllinq/mod.rs:526) L554; `Dom::elements` (xmllinq/mod.rs:640) L555; `well_known_prefix` (xmllinq/serialize.rs:181) L560; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L560; `XName::local_name` (xmllinq/mod.rs:208) (by name) L563
- **Calls (std/external/unresolved):** `.is_none`, `.iter`, `.join`, `.map`, `.position`, `.push`, `.reverse`, `.unwrap_or_default`, `.unwrap_or`, `Some`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `check_style_names` (validate.rs:466); `check_runs_inside_paragraphs` (validate.rs:499); `check_picture_bullets` (validate.rs:743); `collect_ids` (validate.rs:788); `check_del_text_under_del` (validate.rs:898); `check_deleted_text_has_deletion` (validate.rs:953); `check_bookmarks_outside_single_value_controls` (validate.rs:1017); `check_namespace_qname_context` (validate.rs:1588)

#### `parse_part`  (L570-576)
`fn parse_part(pkg: &PartFs, name: &str) -> Option<(Dom, NodeId)>`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:498) L571; `Dom::new` (xmllinq/mod.rs:387) L572; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L573; `Dom::root` (xmllinq/mod.rs:822) L574
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `Package::read` (audit.rs:354) (?); `check_style_names` (validate.rs:466); `check_runs_inside_paragraphs` (validate.rs:499); `check_picture_bullets` (validate.rs:743); `check_revision_and_drawing_ids` (validate.rs:767); `check_del_text_under_del` (validate.rs:898); `check_deleted_text_has_deletion` (validate.rs:953); `check_comment_graph` (validate.rs:1096); `check_comments_extended` (validate.rs:1368); `check_comments_ids` (validate.rs:1415); `check_comments_extensible` (validate.rs:1468); `check_namespace_qname_context` (validate.rs:1588); +1 more

#### `xml_parts`  (L578-583)
`fn xml_parts(pkg: &PartFs) -> Vec<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::parts` (opc/mod.rs:532) L579
- **Calls (std/external/unresolved):** `.collect`, `.ends_with`, `.filter`, `.into_iter`
- **Called by:** `check_style_names` (validate.rs:466); `check_runs_inside_paragraphs` (validate.rs:499); `check_namespace_qname_contexts` (validate.rs:590); `check_revision_and_drawing_ids` (validate.rs:767); `check_para_text_id_bounds` (validate.rs:877); `check_del_text_under_del` (validate.rs:898); `check_deleted_text_has_deletion` (validate.rs:953); `check_bookmarks_outside_single_value_controls` (validate.rs:1017); `renumber_out_of_range_para_ids` (validate.rs:1905)

#### `check_namespace_qname_contexts`  (L590-594)
`fn check_namespace_qname_contexts(pkg: &PartFs, out: &mut Vec<Finding>)`

- **Does:** Every prefix an MC prefix list names (`mc:Choice/@Requires`, `mc:Ignorable`, …) is bound where it is named, in every XML part: Word rejects a header whose `mc:Choice Requires="wps"` names an unbound `wps`.
- **Calls (crate):** `xml_parts` (validate.rs:578) L591; `check_namespace_qname_context` (validate.rs:1588) L592
- **Called by:** `ring1` (validate.rs:445)

#### `check_content_types_and_xml`  (L596-642)
`fn check_content_types_and_xml(pkg: &PartFs, out: &mut Vec<Finding>)`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::parts` (opc/mod.rs:532) L597; `PartFs::content_type_for` (opc/mod.rs:608) L599; `Finding::new` (validate.rs:118) L603,613,629; `PartFs::part_string` (opc/mod.rs:498) L612
- **Calls (std/external/unresolved):** `.clear`, `.config_mut`, `.ends_with`, `.is_none`, `.push`, `.read_event_into`, `.starts_with`, `.trim_text`, `Reader::from_str`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `ring1` (validate.rs:445)

#### `check_relationship_integrity`  (L646-739)
`fn check_relationship_integrity(pkg: &PartFs, out: &mut Vec<Finding>)`

- **Does:** Every `r:id` / `r:embed` / `r:link` in a part resolves in that part's `.rels`; no duplicate rIds; no dangling internal targets.
- **Calls (crate):** `PartFs::repaired_duplicate_ids` (opc/mod.rs:488) L649; `Finding::new` (validate.rs:118) L655,676,700,721; `PartFs::parts` (opc/mod.rs:532) L662; `PartFs::part_string` (opc/mod.rs:498) L666; `PartFs::read_rels_for` (opc/mod.rs:756) L669; `PartFs::resolve_rel_target` (opc/mod.rs:600) L709; `PartFs::part_bytes` (opc/mod.rs:493) L710,711
- **Calls (std/external/unresolved):** `.as_deref`, `.as_str`, `.clone`, `.contains`, `.ends_with`, `.find`, `.get`, `.insert`, `.is_empty`, `.is_none`, `.len`, `.push`, `.starts_with`, `.trim_start_matches`, `HashMap::new`, `HashSet::new`, `Some`
- **Macros:** `format!`
- **Called by:** `ring1` (validate.rs:445); `validation_owner_boundary_tests::relationship_target_diagnostics_distinguish_owned_payloads_from_unresolved_and_external_targets` (validate.rs:2335)

#### `check_picture_bullets`  (L743-765)
`fn check_picture_bullets(pkg: &PartFs, out: &mut Vec<Finding>)`

- **Does:** Every `w:lvlPicBulletId` names a `w:numPicBullet` of the same numbering part: Word refuses the whole package otherwise ("document loaded empty").
- **Calls (crate):** `parse_part` (validate.rs:570) L745; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L748; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L751,754; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L753; `Finding::new` (validate.rs:118) L757; `element_path` (validate.rs:546) L760
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter_map`, `.into_iter`, `.push`, `Some`, `W::name`, `W::val`
- **Macros:** `format!`
- **Called by:** `ring1` (validate.rs:445)

#### `check_revision_and_drawing_ids`  (L767-776)
`fn check_revision_and_drawing_ids(pkg: &PartFs, out: &mut Vec<Finding>)`

- **Does:** _no doc comment_
- **Calls (crate):** `xml_parts` (validate.rs:578) L768; `parse_part` (validate.rs:570) L769; `collect_ids` (validate.rs:788) L774
- **Calls (std/external/unresolved):** `HashSet::new`
- **Called by:** `ring1` (validate.rs:445)

#### `collect_ids`  (L788-837)
`fn collect_ids( dom: &Dom, root: NodeId, rev_ids: &mut HashSet<String>, docpr_ids: &mut HashSet<String>, out: &mut Vec<Finding>, part: &str, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L796; `Dom::name` (xmllinq/mod.rs:526) L797; `XName::local_name` (xmllinq/mod.rs:208) (by name) L800,822; `Dom::attribute` (xmllinq/mod.rs:874) L804,819; `Finding::new` (validate.rs:118) L807,828; `element_path` (validate.rs:546) L810,831; `XName::get` (xmllinq/mod.rs:180) L815; `Dom::attributes` (xmllinq/mod.rs:885) L820
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.find`, `.insert`, `.into_iter`, `.map`, `.or_else`, `.push`, `W::name`
- **Macros:** `format!`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501) (?); `check_revision_and_drawing_ids` (validate.rs:767)

#### `para_id_values`  (L842-856)
`fn para_id_values(xml: &str) -> Vec<u32>`

- **Does:** `(attribute, value)` pairs of `w14:paraId` / `w14:textId` values at or above Word's `0x80000000` bound, in `xml`.
- **Calls (std/external/unresolved):** `.find`, `.len`, `.push`, `Vec::new`, `u32::from_str_radix`
- **Called by:** `renumber_out_of_range_para_ids` (validate.rs:1905); `validation_boundary_tests::paragraph_ids_ignore_invalid_or_unterminated_values_and_find_both_attributes` (validate.rs:2163)

#### `out_of_range_para_ids`  (L858-875)
`fn out_of_range_para_ids(xml: &str) -> Vec<(&'static str, String)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.find`, `.len`, `.push`, `.to_string`, `Vec::new`, `u32::from_str_radix`
- **Called by:** `check_para_text_id_bounds` (validate.rs:877); `renumber_out_of_range_para_ids` (validate.rs:1905); `validation_boundary_tests::paragraph_ids_ignore_invalid_or_unterminated_values_and_find_both_attributes` (validate.rs:2163)

#### `check_para_text_id_bounds`  (L877-893)
`fn check_para_text_id_bounds(pkg: &PartFs, out: &mut Vec<Finding>)`

- **Does:** _no doc comment_
- **Calls (crate):** `xml_parts` (validate.rs:578) L878; `PartFs::part_string` (opc/mod.rs:498) L879; `out_of_range_para_ids` (validate.rs:858) L882; `Finding::new` (validate.rs:118) L883
- **Calls (std/external/unresolved):** `.push`
- **Macros:** `format!`
- **Called by:** `ring1` (validate.rs:445)

#### `check_del_text_under_del`  (L898-931)
`fn check_del_text_under_del(pkg: &PartFs, out: &mut Vec<Finding>)`

- **Does:** `w:del` must carry `w:delText` (never `w:t`).
- **Calls (crate):** `xml_parts` (validate.rs:578) L899; `parse_part` (validate.rs:570) L900; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L903,904,917,918; `ancestor_has` (validate.rs:1043) L905,919; `Finding::new` (validate.rs:118) L908,922; `element_path` (validate.rs:546) L911,925
- **Calls (std/external/unresolved):** `.push`, `Some`, `W::del`, `W::name`, `W::t`
- **Macros:** `format!`
- **Called by:** `ring1` (validate.rs:445)

#### `covered_by_deletion`  (L935-945)
`fn covered_by_deletion(dom: &Dom, node: NodeId) -> bool`

- **Does:** True when a `w:del` or `w:moveFrom` covers `node` in its own story (a text box's `w:txbxContent` is a separate story).
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L936,941; `Dom::name` (xmllinq/mod.rs:526) L938; `XName::local_name` (xmllinq/mod.rs:208) (by name) L938
- **Calls (std/external/unresolved):** `.as_ref`, `.map`
- **Called by:** `check_deleted_text_has_deletion` (validate.rs:953); `fix_deleted_text` (validate.rs:1757); `validation_boundary_tests::deletion_scope_stops_at_textboxes_and_nested_insertions_stay_live` (validate.rs:2180)

#### `check_deleted_text_has_deletion`  (L953-973)
`fn check_deleted_text_has_deletion(pkg: &PartFs, out: &mut Vec<Finding>)`

- **Does:** `w:delText` / `w:delInstrText` must sit under a `w:del` (or `w:moveFrom`) in its OWN story.
- **Calls (crate):** `xml_parts` (validate.rs:578) L954; `parse_part` (validate.rs:570) L955; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L959; `covered_by_deletion` (validate.rs:935) L960; `Finding::new` (validate.rs:118) L961; `element_path` (validate.rs:546) L964
- **Calls (std/external/unresolved):** `.push`, `Some`, `W::name`
- **Macros:** `format!`
- **Called by:** `ring1` (validate.rs:445)

#### `bookmarks_in_single_value_controls`  (L988-1009)
`fn bookmarks_in_single_value_controls( dom: &Dom, root: NodeId, ) -> Vec<(NodeId, &'static str, String)>`

- **Does:** `(bookmark mark, kind, control)` for every bookmark start or end inside a single-value content control.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L994; `Dom::ancestors` (xmllinq/mod.rs:770) L995; `Dom::element` (xmllinq/mod.rs:655) L996; `Dom::elements` (xmllinq/mod.rs:640) L999; `Dom::name` (xmllinq/mod.rs:526) L1000; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1000
- **Calls (std/external/unresolved):** `.as_str`, `.contains`, `.filter_map`, `.find`, `.flat_map`, `.into_iter`, `.map`, `.push`, `.to_string`, `Some`, `Vec::new`, `W::name`
- **Called by:** `check_bookmarks_outside_single_value_controls` (validate.rs:1017); `drop_bookmarks_in_single_value_controls` (validate.rs:1793); `validation_boundary_tests::bookmarks_are_forbidden_in_each_single_value_control_but_allowed_in_rich_text` (validate.rs:2204)

#### `check_bookmarks_outside_single_value_controls`  (L1017-1041)
`fn check_bookmarks_outside_single_value_controls(pkg: &PartFs, out: &mut Vec<Finding>)`

- **Does:** No bookmark start or end in a plain-text, dropdown, combo box, date, picture or checkbox content control.
- **Calls (crate):** `xml_parts` (validate.rs:578) L1018; `PartFs::part_string` (opc/mod.rs:498) L1019; `Dom::new` (xmllinq/mod.rs:387) L1025; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L1026; `Dom::root` (xmllinq/mod.rs:822) L1027; `bookmarks_in_single_value_controls` (validate.rs:988) L1030; `Finding::new` (validate.rs:118) L1031; `element_path` (validate.rs:546) L1034
- **Calls (std/external/unresolved):** `.contains`, `.push`
- **Macros:** `format!`
- **Called by:** `ring1` (validate.rs:445)

#### `ancestor_has`  (L1043-1057)
`fn ancestor_has(dom: &Dom, node: NodeId, stop: NodeId, local: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L1044,1054; `Dom::name` (xmllinq/mod.rs:526) L1049; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1050
- **Called by:** `check_del_text_under_del` (validate.rs:898); `fix_deleted_text` (validate.rs:1757)

#### `main_part`  (L1084-1089)
`fn main_part(pkg: &PartFs) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:829) L1085; `PartFs::part_bytes` (opc/mod.rs:493) L1086
- **Calls (std/external/unresolved):** `.map`, `.or_else`, `.to_string`
- **Called by:** `check_comment_graph` (validate.rs:1096); `repair` (validate.rs:1649)

#### `comment_finding`  (L1092-1094)
`fn comment_finding(out: &mut Vec<Finding>, code: &str, part: &str, message: String)`

- **Does:** A finding of the comment family: `code` on `part`, no path.
- **Calls (crate):** `Finding::new` (validate.rs:118) L1093
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `check_comment_graph` (validate.rs:1096); `check_comment_family_packaging` (validate.rs:1272); `check_hex_id` (validate.rs:1329); `check_comments_extended` (validate.rs:1368); `check_comments_ids` (validate.rs:1415); `check_comments_extensible` (validate.rs:1468); `check_exact_key_set` (validate.rs:1526); `check_parent_cycles` (validate.rs:1551)

#### `check_comment_graph`  (L1096-1251)
`fn check_comment_graph(pkg: &PartFs, out: &mut Vec<Finding>)`

- **Does:** _no doc comment_
- **Calls (crate):** `main_part` (validate.rs:1084) L1097; `parse_part` (validate.rs:570) L1100,1107; `comment_id_counts` (validate.rs:1256) L1103,1104,1105,1135; `comment_finding` (validate.rs:1092) L1114,1124,1151,1168,1185,1213...; `PartFs::part_bytes` (opc/mod.rs:493) L1123,1198; `check_comment_family_packaging` (validate.rs:1272) L1194; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L1201; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L1202,1207; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L1206; `check_hex_id` (validate.rs:1329) L1211; `check_comments_extended` (validate.rs:1368) L1235; `check_comments_ids` (validate.rs:1415) L1236; `check_comments_extensible` (validate.rs:1468) L1237; `check_parent_cycles` (validate.rs:1551) L1249
- **Calls (std/external/unresolved):** `.any`, `.as_ref`, `.as_str`, `.clone`, `.contains_key`, `.contains`, `.filter`, `.insert`, `.is_empty`, `.is_some`, `.iter`, `.keys`, `.last`, `.push`, `.to_ascii_uppercase`, `.unwrap_or`, `.values`, `HashSet::new`, `Some`, `Vec::new`, `W14::name`, `W::id`, `W::name`, `W::p`
- **Macros:** `format!`
- **Called by:** `ring1` (validate.rs:445); `authored_comment_family_validation_contract_tests::auxiliary_comment_parts_without_their_source_definition_are_reported_individually` (validate.rs:2761)

#### `comment_id_counts`  (L1256-1270)
`fn comment_id_counts(dom: &Dom, root: NodeId, local: &str) -> HashMap<String, usize>`

- **Does:** How often each id occurs on `local` elements.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1258; `Dom::ancestors` (xmllinq/mod.rs:770) L1259; `Dom::attribute` (xmllinq/mod.rs:874) L1265
- **Calls (std/external/unresolved):** `.entry`, `.is_empty`, `.or_default`, `.to_string`, `HashMap::new`, `MC::name`, `Some`, `W::id`, `W::name`
- **Called by:** `check_comment_graph` (validate.rs:1096); `repair` (validate.rs:1649)

#### `check_comment_family_packaging`  (L1272-1327)
`fn check_comment_family_packaging(pkg: &PartFs, main: &str, out: &mut Vec<Finding>)`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:756) L1273; `PartFs::part_bytes` (opc/mod.rs:493) L1275; `PartFs::content_type_for` (opc/mod.rs:608) L1282; `comment_finding` (validate.rs:1092) L1283,1291,1307,1319; `PartFs::resolve_rel_target` (opc/mod.rs:600) L1302
- **Calls (std/external/unresolved):** `.as_deref`, `.collect`, `.filter`, `.flat_map`, `.into_iter`, `.is_empty`, `.is_some`, `.len`, `.to_string`, `.trim_start_matches`, `Some`
- **Macros:** `format!`
- **Called by:** `check_comment_graph` (validate.rs:1096); `authored_comment_family_validation_contract_tests::each_comment_family_relationship_requires_its_exact_owned_part_and_content_type` (validate.rs:2785)

#### `check_hex_id`  (L1329-1361)
`fn check_hex_id( label: &str, value: &str, word_para_bound: bool, part: &str, out: &mut Vec<Finding>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `comment_finding` (validate.rs:1092) L1345,1354
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.contains`, `.is_ascii_hexdigit`, `.len`, `.ok`, `u32::from_str_radix`
- **Macros:** `format!`
- **Called by:** `check_comment_graph` (validate.rs:1096); `check_comments_extended` (validate.rs:1368); `check_comments_ids` (validate.rs:1415); `check_comments_extensible` (validate.rs:1468); `validation_boundary_tests::metadata_ids_check_width_hex_digits_and_the_word_boundary_independently` (validate.rs:2126)

#### `check_comments_extended`  (L1368-1413)
`fn check_comments_extended( pkg: &PartFs, last_para_ids: &HashSet<String>, out: &mut Vec<Finding>, ) -> Option<ExtendedGraph>`

- **Does:** _no doc comment_
- **Calls (crate):** `parse_part` (validate.rs:570) L1374; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L1377; `attribute_by_local` (validate.rs:1516) L1378,1397; `comment_finding` (validate.rs:1092) L1379,1390,1401; `check_hex_id` (validate.rs:1329) L1388,1399; `check_exact_key_set` (validate.rs:1526) L1411
- **Calls (std/external/unresolved):** `.clone`, `.insert`, `.to_ascii_uppercase`, `.to_string`, `HashMap::new`, `HashSet::new`, `Some`
- **Macros:** `format!`
- **Called by:** `check_comment_graph` (validate.rs:1096); `validation_boundary_tests::comment_auxiliary_parts_reject_missing_duplicate_and_unmatched_ids` (validate.rs:2225)

#### `check_comments_ids`  (L1415-1466)
`fn check_comments_ids( pkg: &PartFs, last_para_ids: &HashSet<String>, out: &mut Vec<Finding>, ) -> Option<HashSet<String>>`

- **Does:** _no doc comment_
- **Calls (crate):** `parse_part` (validate.rs:570) L1421; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L1424; `attribute_by_local` (validate.rs:1516) L1425,1444; `comment_finding` (validate.rs:1092) L1426,1437,1445,1456; `check_hex_id` (validate.rs:1329) L1435,1454; `check_exact_key_set` (validate.rs:1526) L1464
- **Calls (std/external/unresolved):** `.insert`, `.to_ascii_uppercase`, `.to_string`, `HashSet::new`, `Some`
- **Macros:** `format!`
- **Called by:** `check_comment_graph` (validate.rs:1096); `validation_boundary_tests::comment_auxiliary_parts_reject_missing_duplicate_and_unmatched_ids` (validate.rs:2225)

#### `check_comments_extensible`  (L1468-1514)
`fn check_comments_extensible( pkg: &PartFs, expected_durable_ids: Option<&HashSet<String>>, out: &mut Vec<Finding>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `parse_part` (validate.rs:570) L1474; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L1478; `attribute_by_local` (validate.rs:1516) L1479; `comment_finding` (validate.rs:1092) L1480,1491,1507; `check_hex_id` (validate.rs:1329) L1489; `check_exact_key_set` (validate.rs:1526) L1500
- **Calls (std/external/unresolved):** `.insert`, `.to_ascii_uppercase`, `.to_string`, `HashSet::new`
- **Macros:** `format!`
- **Called by:** `check_comment_graph` (validate.rs:1096); `validation_boundary_tests::comment_auxiliary_parts_reject_missing_duplicate_and_unmatched_ids` (validate.rs:2225)

#### `attribute_by_local`  (L1516-1524)
`fn attribute_by_local<'a>(dom: &'a Dom, element: NodeId, local: &str) -> Option<&'a str>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::attr_count` (xmllinq/mod.rs:895) L1517; `Dom::attr_at` (xmllinq/mod.rs:906) L1518; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1519
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `check_comments_extended` (validate.rs:1368); `check_comments_ids` (validate.rs:1415); `check_comments_extensible` (validate.rs:1468)

#### `check_exact_key_set`  (L1526-1549)
`fn check_exact_key_set( label: &str, actual: &HashSet<String>, expected: &HashSet<String>, part: &str, out: &mut Vec<Finding>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `comment_finding` (validate.rs:1092) L1534,1542
- **Calls (std/external/unresolved):** `.difference`
- **Macros:** `format!`
- **Called by:** `check_comments_extended` (validate.rs:1368); `check_comments_ids` (validate.rs:1415); `check_comments_extensible` (validate.rs:1468)

#### `check_parent_cycles`  (L1551-1568)
`fn check_parent_cycles(parents: &HashMap<String, String>, out: &mut Vec<Finding>)`

- **Does:** _no doc comment_
- **Calls (crate):** `comment_finding` (validate.rs:1092) L1557
- **Calls (std/external/unresolved):** `.clone`, `.get`, `.insert`, `.keys`, `HashSet::new`
- **Macros:** `format!`
- **Called by:** `check_comment_graph` (validate.rs:1096); `validation_boundary_tests::graph_cycles_are_reported_without_confusing_acyclic_chains` (validate.rs:2265)

#### `is_namespace_qname_list`  (L1572-1586)
`fn is_namespace_qname_list(element: &XName, name: &XName) -> bool`

- **Does:** An unqualified `Requires` is a prefix list only on `mc:Choice`; on any other element (a custom XML part) it is application data.
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L1573,1575,1577; `XName::local_name` (xmllinq/mod.rs:208) L1574,1576; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1579
- **Calls (std/external/unresolved):** `.is_empty`
- **Macros:** `matches!`
- **Called by:** `check_namespace_qname_context` (validate.rs:1588); `validation_boundary_tests::namespace_prefix_lists_distinguish_application_data_bound_and_unknown_prefixes` (validate.rs:2285)

#### `check_namespace_qname_context`  (L1588-1619)
`fn check_namespace_qname_context(pkg: &PartFs, part: &str, out: &mut Vec<Finding>)`

- **Does:** _no doc comment_
- **Calls (crate):** `parse_part` (validate.rs:570) L1589; `Dom::descendants_and_self` (xmllinq/mod.rs:746) (by name) L1592; `Dom::attributes` (xmllinq/mod.rs:885) (by name) L1596; `is_namespace_qname_list` (validate.rs:1572) L1597; `namespace_in_scope` (validate.rs:1621) L1602; `Finding::new` (validate.rs:118) L1603; `element_path` (validate.rs:546) L1606; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1609; `well_known_namespace` (xmllinq/serialize.rs:190) L1613
- **Calls (std/external/unresolved):** `.is_none`, `.is_some`, `.map_or`, `.name`, `.push`, `.split_once`, `.split_whitespace`
- **Macros:** `format!`
- **Called by:** `check_namespace_qname_contexts` (validate.rs:590); `validation_boundary_tests::namespace_prefix_lists_distinguish_application_data_bound_and_unknown_prefixes` (validate.rs:2285)

#### `namespace_in_scope`  (L1621-1633)
`fn namespace_in_scope<'a>(dom: &'a Dom, element: NodeId, prefix: &str) -> Option<&'a str>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::attr_count` (xmllinq/mod.rs:895) L1624; `Dom::attr_at` (xmllinq/mod.rs:906) L1625; `Dom::is_namespace_declaration` (xmllinq/mod.rs:996) L1626; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1626; `Dom::parent` (xmllinq/mod.rs:597) L1630
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `check_namespace_qname_context` (validate.rs:1588)

#### `repair`  (L1649-1738, pub)
`pub fn repair(docx: &[u8]) -> Result<Repaired, ValidateError>`

- **Does:** The package with every repairable finding fixed, in a fixed order: unbound MC prefixes bound on the part root, `w:t`/`w:delText` swapped, bookmarks dropped from single-value controls, dangling relationship attributes dropped, duplicate drawing and revision ids renumbered, paragraph ids brought into Word's range, cells...
- **Calls (crate):** `validate` (validate.rs:420) L1650,1726; `fix_up_drawing_ids_in_package` (comparer/fixups.rs:121) L1672; `PartFs::open` (opc/mod.rs:456) L1676; `edit_part` (validate.rs:1741) L1678,1688,1691,1698,1703,1709...; `bind_compatibility_prefixes` (comparer/finalize.rs:983) L1679; `PartFs::read_rels_for` (opc/mod.rs:756) L1694; `drop_dangling_relationship_attributes` (validate.rs:1828) L1699; `renumber_out_of_range_para_ids` (validate.rs:1905) L1706; `main_part` (validate.rs:1084) L1712; `parse_part` (validate.rs:570) L1714; `comment_id_counts` (validate.rs:1256) L1716; `drop_orphan_comment_anchors` (validate.rs:1968) L1722; `PartFs::to_zip` (opc/mod.rs:548) L1725; `detach_breaks::{fn same}` (markdown/unified.rs:755) (by name) L1731
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.clone`, `.collect`, `.contains`, `.dedup`, `.filter`, `.into_iter`, `.into_keys`, `.iter`, `.map`, `.sort`, `.to_vec`, `.unwrap_or_default`, `Ok`, `Vec::new`, `has`, `parts_with`
- **Called by:** `run_validate` (bin/jubarte.rs:2793); `public_memory_validator_owner_boundary_tests::missing_opaque_part_type_has_exact_public_diagnostics_and_no_speculative_repair` (validate.rs:2459); `public_memory_validator_owner_boundary_tests::repairing_comment_paragraph_ids_retains_reply_graph_source_text_and_all_relationships` (validate.rs:2498); `retained_namespace_finding_source_contract_tests::repair_reports_unbound_unknown_prefixes_without_discarding_owned_story_content` (validate.rs:2879); `duplicate_drawing_owned_repair_contract_tests::duplicate_drawing_repair_renumbers_only_ids_and_preserves_every_shape_owner` (validate.rs:2982)

#### `edit_part`  (L1741-1752)
`fn edit_part(pkg: &mut PartFs, part: &str, f: impl FnOnce(&mut Dom, NodeId))`

- **Does:** Parse `part`, let `f` change it, write it back.
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:498) L1742; `Dom::new` (xmllinq/mod.rs:387) L1745; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L1746; `Dom::root` (xmllinq/mod.rs:822) L1747; `PartFs::set_part` (opc/mod.rs:510) L1751; `Dom::serialize_document` (xmllinq/mod.rs:1322) L1751
- **Calls (std/external/unresolved):** `.into_bytes`, `f`
- **Called by:** `repair` (validate.rs:1649); `validation_boundary_tests::absent_or_unreadable_parts_are_not_modified_by_edit_part` (validate.rs:2314)

#### `fix_deleted_text`  (L1757-1789)
`fn fix_deleted_text(dom: &mut Dom, root: NodeId)`

- **Does:** `w:t` and `w:instrText` under a deletion become `w:delText` and `w:delInstrText`; `w:delText` under a move source, and deleted text no deletion covers, become live text again.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1759,1761,1768,1769,1780; `ancestor_has` (validate.rs:1043) L1762,1770; `Dom::set_name` (xmllinq/mod.rs:539) L1776,1787; `covered_by_deletion` (validate.rs:935) L1781
- **Calls (std/external/unresolved):** `.push`, `Some`, `Vec::new`, `W::del`, `W::name`, `W::t`
- **Called by:** `validation_boundary_tests::deletion_scope_stops_at_textboxes_and_nested_insertions_stay_live` (validate.rs:2180)

#### `drop_bookmarks_in_single_value_controls`  (L1793-1809)
`fn drop_bookmarks_in_single_value_controls(dom: &mut Dom, root: NodeId)`

- **Does:** Every bookmark whose start or end sits in a single-value content control is dropped whole (both marks, wherever the other one is).
- **Calls (crate):** `bookmarks_in_single_value_controls` (validate.rs:988) L1794; `Dom::attribute` (xmllinq/mod.rs:874) L1796,1802; `Dom::descendants` (xmllinq/mod.rs:668) L1800; `Dom::remove` (xmllinq/mod.rs:1102) L1807
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter_map`, `.filter`, `.flat_map`, `.into_iter`, `.is_some_and`, `.map`, `Some`, `W::id`, `W::name`
- **Called by:** `validation_boundary_tests::bookmarks_are_forbidden_in_each_single_value_control_but_allowed_in_rich_text` (validate.rs:2204)

#### `drop_dangling_relationship_attributes`  (L1828-1852)
`fn drop_dangling_relationship_attributes(dom: &mut Dom, root: NodeId, ids: &HashSet<String>)`

- **Does:** Relationship attributes (`r:id`, `r:embed`, `r:link`, …) naming no relationship of the part are dropped, which is what stops Word's "unreadable content" repair.
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L1831; `Dom::attributes` (xmllinq/mod.rs:885) L1832; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1833; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1834,1837; `Dom::name` (xmllinq/mod.rs:526) L1835; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1847; `Dom::remove` (xmllinq/mod.rs:1102) L1850
- **Calls (std/external/unresolved):** `.contains`, `.is_empty`, `.is_some_and`, `.push`, `Vec::new`
- **Called by:** `repair` (validate.rs:1649); `paragraph_id_collision_owner_tests::dangling_relationship_repair_keeps_empty_known_and_nonrelationship_owners` (validate.rs:2704)

#### `renumber_duplicate_revision_ids`  (L1856-1900)
`fn renumber_duplicate_revision_ids(dom: &mut Dom, root: NodeId)`

- **Does:** A second use of a revision id is renumbered from the part's highest `w:id` upwards; a move range start takes its end along.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1858; `Dom::attribute` (xmllinq/mod.rs:874) L1861,1874,1891; `Dom::name` (xmllinq/mod.rs:526) L1869,1890; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1870,1890; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1898
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.contains`, `.enumerate`, `.filter_map`, `.find`, `.insert`, `.is_some_and`, `.iter`, `.map`, `.max`, `.ok`, `.parse`, `.push`, `.to_string`, `.unwrap_or`, `HashSet::new`, `Some`, `Vec::new`, `W::name`
- **Called by:** `literal_revision_repair_contract_tests::duplicate_move_ids_keep_exact_range_pairings_and_every_unrelated_source_node` (validate.rs:2598); `literal_revision_repair_contract_tests::repair_declines_missing_ids_and_unmatched_ranges_without_inventing_content` (validate.rs:2619)

#### `renumber_out_of_range_para_ids`  (L1905-1950)
`fn renumber_out_of_range_para_ids(pkg: &mut PartFs)`

- **Does:** Every `w14:paraId` / `w14:textId` at or above `0x80000000` is masked into Word's range, and every reference to it (`paraIdParent`, the comment parts) follows.
- **Calls (crate):** `xml_parts` (validate.rs:578) L1906; `PartFs::part_string` (opc/mod.rs:498) L1909,1916,1937; `para_id_values` (validate.rs:842) L1912; `out_of_range_para_ids` (validate.rs:858) L1919; `PartFs::set_part` (opc/mod.rs:510) L1947
- **Calls (std/external/unresolved):** `.clone`, `.contains_key`, `.contains`, `.extend`, `.insert`, `.into_bytes`, `.is_empty`, `.replace`, `.unwrap_or`, `HashMap::new`, `HashSet::new`, `u32::from_str_radix`
- **Macros:** `format!`
- **Called by:** `repair` (validate.rs:1649); `paragraph_id_collision_owner_tests::paragraph_id_collision_repair_preserves_all_owners_and_shared_references` (validate.rs:2655)

#### `end_cells_with_a_paragraph`  (L1954-1964)
`fn end_cells_with_a_paragraph(dom: &mut Dom, root: NodeId)`

- **Does:** A table cell whose last child (its properties aside) is not a paragraph gets an empty one.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1955; `cell_ends_in_paragraph` (debug.rs:288) L1958; `Dom::new_element` (xmllinq/mod.rs:478) L1961; `Dom::add` (xmllinq/mod.rs:1058) L1962
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `Some`, `W::name`, `W::p`
- **Called by:** none found in src

#### `drop_orphan_comment_anchors`  (L1968-2000)
`fn drop_orphan_comment_anchors(dom: &mut Dom, root: NodeId, defined: &HashSet<String>)`

- **Does:** Comment range marks and references naming no comment definition are dropped; a run left holding only its properties goes with its reference.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L1970; `Dom::descendants` (xmllinq/mod.rs:668) L1975,1981; `Dom::remove` (xmllinq/mod.rs:1102) L1979,1990,1997; `Dom::parent` (xmllinq/mod.rs:597) L1987; `Dom::name_is` (xmllinq/mod.rs:535) L1989,1995; `Dom::elements` (xmllinq/mod.rs:640) L1992
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.contains`, `.filter`, `.flat_map`, `.into_iter`, `.is_some_and`, `.iter`, `Some`, `W::id`, `W::name`, `orphan`
- **Called by:** `repair` (validate.rs:1649); `validation_owner_boundary_tests::orphan_comment_cleanup_preserves_live_references_and_neighbor_run_properties` (validate.rs:2384)

#### `all_paragraphs`  (L2005-2011)
`fn all_paragraphs(docx: &[u8]) -> Result<Vec<crate::inspect::Paragraph>, ValidateError>`

- **Does:** The body's paragraphs followed by every story's (`header1`, ...).
- **Calls (crate):** `paragraphs` (inspect.rs:284) L2006; `stories` (inspect.rs:312) L2007
- **Calls (std/external/unresolved):** `.extend`, `Ok`
- **Called by:** `audit_tracked` (validate.rs:2036)

#### `story_part`  (L2016-2024)
`fn story_part(id: &str) -> String`

- **Does:** The part a paragraph or change id lives in: the body's (`p3`, `r12`) is the main document, any other story's (`header1.p1`, `footnotes.r2`) `word/{story}.xml`.
- **Calls (crate):** `paragraph_place` (ids.rs:417) L2017; `change_id` (ids.rs:187) L2019
- **Calls (std/external/unresolved):** `.as_deref`, `.map`, `.or_else`, `.to_string`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `<Finding as From>::from` (validate.rs:392); `audit_tracked` (validate.rs:2036)

#### `audit_tracked`  (L2036-2096, pub)
`pub fn audit_tracked( original: &[u8], edited: &[u8], author: &str, ) -> Result<Vec<Finding>, ValidateError>`

- **Does:** `validate.py --original --author` without an XSD: every visible-text difference between `original` and `edited` must sit in a revision by `author`.
- **Calls (crate):** `list_changes` (changes.rs:287) L2042; `Finding::new` (validate.rs:118) L2047,2070,2084; `story_part` (validate.rs:2016) L2049,2086; `reject_changes` (changes.rs:345) L2064; `all_paragraphs` (validate.rs:2005) L2067,2068; `as_edits` (validate.rs:2099) L2080,2095
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.filter`, `.iter`, `.len`, `.push`, `.to_string`, `.unwrap_or_else`, `.zip`, `ChangeFilter::default`, `Ok`, `Some`, `Vec::new`
- **Macros:** `format!`, `vec!`
- **Called by:** `validate_with` (validate.rs:290)

#### `as_edits`  (L2099-2105)
`fn as_edits(mut findings: Vec<Finding>) -> Vec<Finding>`

- **Does:** [`audit_tracked`]'s findings belong to the `edits` set and fail a run.
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `audit_tracked` (validate.rs:2036)

## Test functions (29)

- `validation_boundary_tests::pkg` (L2112): _no doc comment_
- `validation_boundary_tests::xml` (L2116): _no doc comment_
- `validation_boundary_tests::metadata_ids_check_width_hex_digits_and_the_word_boundary_independently` (L2126): _no doc comment_
- `validation_boundary_tests::paragraph_ids_ignore_invalid_or_unterminated_values_and_find_both_attributes` (L2163): _no doc comment_
- `validation_boundary_tests::deletion_scope_stops_at_textboxes_and_nested_insertions_stay_live` (L2180): _no doc comment_
- `validation_boundary_tests::bookmarks_are_forbidden_in_each_single_value_control_but_allowed_in_rich_text` (L2204): _no doc comment_
- `validation_boundary_tests::comment_auxiliary_parts_reject_missing_duplicate_and_unmatched_ids` (L2225): _no doc comment_
- `validation_boundary_tests::graph_cycles_are_reported_without_confusing_acyclic_chains` (L2265): _no doc comment_
- `validation_boundary_tests::namespace_prefix_lists_distinguish_application_data_bound_and_unknown_prefixes` (L2285): _no doc comment_
- `validation_boundary_tests::absent_or_unreadable_parts_are_not_modified_by_edit_part` (L2314): _no doc comment_
- `validation_owner_boundary_tests::package` (L2331): _no doc comment_
- `validation_owner_boundary_tests::relationship_target_diagnostics_distinguish_owned_payloads_from_unresolved_and_external_targets` (L2335): _no doc comment_
- `validation_owner_boundary_tests::orphan_comment_cleanup_preserves_live_references_and_neighbor_run_properties` (L2384): _no doc comment_
- `public_memory_validator_owner_boundary_tests::package` (L2440): _no doc comment_
- `public_memory_validator_owner_boundary_tests::snapshot` (L2446): _no doc comment_
- `public_memory_validator_owner_boundary_tests::missing_opaque_part_type_has_exact_public_diagnostics_and_no_speculative_repair` (L2459): _no doc comment_
- `public_memory_validator_owner_boundary_tests::repairing_comment_paragraph_ids_retains_reply_graph_source_text_and_all_relationships` (L2498): _no doc comment_
- `literal_revision_repair_contract_tests::tree` (L2587): _no doc comment_
- `literal_revision_repair_contract_tests::duplicate_move_ids_keep_exact_range_pairings_and_every_unrelated_source_node` (L2598): _no doc comment_
- `literal_revision_repair_contract_tests::repair_declines_missing_ids_and_unmatched_ranges_without_inventing_content` (L2619): _no doc comment_
- `paragraph_id_collision_owner_tests::snapshot` (L2642): _no doc comment_
- `paragraph_id_collision_owner_tests::paragraph_id_collision_repair_preserves_all_owners_and_shared_references` (L2655): _no doc comment_
- `paragraph_id_collision_owner_tests::dangling_relationship_repair_keeps_empty_known_and_nonrelationship_owners` (L2704): _no doc comment_
- `authored_comment_family_validation_contract_tests::package` (L2742): _no doc comment_
- `authored_comment_family_validation_contract_tests::frozen` (L2748): _no doc comment_
- `authored_comment_family_validation_contract_tests::auxiliary_comment_parts_without_their_source_definition_are_reported_individually` (L2761): _no doc comment_
- `authored_comment_family_validation_contract_tests::each_comment_family_relationship_requires_its_exact_owned_part_and_content_type` (L2785): _no doc comment_
- `retained_namespace_finding_source_contract_tests::repair_reports_unbound_unknown_prefixes_without_discarding_owned_story_content` (L2879): _no doc comment_
- `duplicate_drawing_owned_repair_contract_tests::duplicate_drawing_repair_renumbers_only_ids_and_preserves_every_shape_owner` (L2982): _no doc comment_
