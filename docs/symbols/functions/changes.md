# Functions and methods in `src/changes.rs`

51 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `Change::element_ids`  (L118-122, pub)
`pub fn element_ids(&self) -> impl Iterator<Item = &str>`

- **Does:** The ids of the elements this change is made of, in reading order: its [`Self::members`]', or its own for a change of one element.
- **Calls (std/external/unresolved):** `.as_str`, `.chain`, `.into_iter`, `.is_empty`, `.iter`, `.map`, `.then_some`
- **Called by:** `revision_counts` (edit/tracked.rs:404) (?)

#### `audit_stories`  (L168-279, pub(crate))
`pub(crate) fn audit_stories(docx: &[u8]) -> Result<Vec<AuditStory>, ChangeError>`

- **Does:** Read paragraph text including deletions, and document positions for the same carriers as `list_changes`.
- **Calls (crate):** `open` (changes.rs:1031) L169; `main_part` (changes.rs:1039) L170; `revision_bearing_parts` (revision_processor.rs:3716) L172; `PartFs::part_string` (opc/mod.rs:636) (by name) L173; `Dom::new` (xmllinq/mod.rs:387) L176; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L177; `Dom::root` (xmllinq/mod.rs:822) L178; `story_id` (changes.rs:1046) L181; `carriers` (changes.rs:1124) L182; `Dom::name` (xmllinq/mod.rs:526) L196,216,242; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L197; `XName::local_name` (xmllinq/mod.rs:208) (by name) L199,218,242; `Dom::value` (xmllinq/mod.rs:1185) L221; `Dom::child_count` (xmllinq/mod.rs:622) L226; `Dom::child_at` (xmllinq/mod.rs:632) L227; `story_paragraph_nodes` (inspect.rs:1178) L230; `paragraph_id` (ids.rs:410) L233; `text_of` (changes.rs:1518) L233; `paragraph_index` (changes.rs:1377) L235; `paragraph_of` (changes.rs:1388) L240; `audit_owner` (changes.rs:283) L245,258
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.enumerate`, `.filter_map`, `.filter`, `.get_mut`, `.insert`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.iter`, `.map`, `.pop`, `.push`, `.rev`, `.to_string`, `.unwrap`, `HashMap::new`, `Ok`, `Some`, `String::new`, `Vec::new`
- **Macros:** `matches!`, `vec!`
- **Called by:** `check_foreign_revisions` (validate.rs:3541)

#### `audit_owner`  (L283-307)
`fn audit_owner(dom: &Dom, node: NodeId) -> String`

- **Does:** Owner paths use semantic ids where OOXML defines them, otherwise the element's ordinal among same-named siblings.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L285; `Dom::name` (xmllinq/mod.rs:526) L286,300; `XName::local_name` (xmllinq/mod.rs:208) (by name) L289; `Dom::attribute` (xmllinq/mod.rs:874) L293; `Dom::nodes_before_self` (xmllinq/mod.rs:846) L297
- **Calls (std/external/unresolved):** `.as_ref`, `.count`, `.filter`, `.find_map`, `.into_iter`, `.join`, `.map`, `.push`, `.rev`, `.to_string`, `.unwrap_or_else`, `Some`, `Vec::new`, `W::name`
- **Macros:** `format!`
- **Called by:** `audit_stories` (changes.rs:168)

#### `ChangeFilter::ids`  (L330-335, pub)
`pub fn ids<I: IntoIterator<Item = S>, S: Into<String>>(ids: I) -> Self`

- **Does:** Build a filter that selects the changes `ids` names (`r12`, `header1.r1`, `r1+2`).
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`, `Self::default`, `Some`
- **Called by:** `plan_from_flags` (edit/flags.rs:85); `tests::a_silent_reject_is_not_recorded_into_a_visible_one` (edit/flags.rs:1042); `tests::recording_reads_ids_and_names_as_the_plan_spells_them` (edit/flags.rs:1064); `tests::revisions_go_to_resolve_revisions` (edit/flags.rs:1506); `for_edit` (edit/for_edit.rs:1581); `SourceModel::decisions` (edit/v2_apply.rs:1235); `resolve_selected` (edit.rs:1485); `deeper_boundary_tests::revision_selection_empty_and_reject_only_preserve_unselected_change` (edit.rs:8914)

#### `ChangeFilter::normalized`  (L349-377, pub)
`pub fn normalized(&self) -> Result<Self, ChangeError>`

- **Does:** The filter with every id in its settled short spelling, the form [`Change::id`] holds and [`Self::check`] and [`Self::matches`] compare: `r12`, `header1.r1`, a pair `r1+2` as its two ids, each once.
- **Calls (crate):** `parse_change_ids` (ids.rs:232) L364; `ChangeId::short` (ids.rs:155) (by name) L367
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.contains`, `.into`, `.is_some_and`, `.len`, `.map_err`, `.push`, `.to_string`, `ChangeError::InvalidId`, `Err`, `Ok`, `Some`, `Vec::with_capacity`
- **Called by:** `run_resolution` (bin/jubarte.rs:458); `check_whole_changes` (changes.rs:596); `resolve_with_report` (changes.rs:822); `resolve_for_edit_or_cli` (changes.rs:922); `record_into` (edit/flags.rs:690) (?); `record` (edit/record.rs:191) (?); `named_ids` (edit.rs:1472) (?)

#### `ChangeFilter::check`  (L387-402, pub)
`pub fn check(&self, listed: &[Change]) -> Result<(), ChangeError>`

- **Does:** The first error the filter's ids meet in `listed`: an id that names no change, or one that names several.
- **Calls (std/external/unresolved):** `.clone`, `.cloned`, `.collect`, `.filter`, `.flatten`, `.into_iter`, `.iter`, `.len`, `ChangeError::UnknownChange`, `Err`, `Ok`
- **Called by:** `resolve_for_edit_or_cli` (changes.rs:922)

#### `ChangeFilter::matches`  (L405-415, pub)
`pub fn matches(&self, change: &Change) -> bool`

- **Does:** Whether `change` is selected.
- **Calls (std/external/unresolved):** `.as_ref`, `.contains`, `.is_none_or`, `.is_some_and`
- **Called by:** `resolve_with_report` (changes.rs:822); `resolve_for_edit_or_cli` (changes.rs:922); `public_memory_anchor_boundary_tests::block_and_point_comments_preserve_exact_unicode_anchor_context_and_package` (comments.rs:1427) (?); `tests::regression_tests::omitted_cell_and_row_end_tags_keep_the_table` (convert/altchunk.rs:875) (?); `tests::regression_tests::comments_scripts_and_styles_do_not_become_document_text` (convert/altchunk.rs:994) (?); `tests::regression_tests::html_tables_keep_spans_empty_cells_and_header_formatting` (convert/altchunk.rs:1026) (?); `tests::regression_tests::invalid_or_zero_colspan_falls_back_to_one_column` (convert/altchunk.rs:1047) (?); `collect_hf_runs` (convert/mod.rs:21559) (?); `Layout::emit_picture_break_lines` (convert/mod.rs:27833) (?); `converter_semantic_boundary_tests::attached_template_replaces_only_default_paragraph_style_and_inserts_when_absent` (convert/mod.rs:56017) (?); `residual_byte_and_geometry_contract_tests::unicode_cmap_chunks_preserve_every_cid_and_utf16_surrogate_exactly` (convert/pdf.rs:2708) (?); `residual_byte_and_geometry_contract_tests::authored_width_scales_close_text_objects_without_leaking_into_following_glyphs` (convert/pdf.rs:2883) (?); +20 more

#### `<ChangeError as fmt::Display>::fmt`  (L451-505, impl fmt::Display)
`fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`

- **Does:** _no doc comment_
- **Calls (crate):** `kind_name` (changes.rs:510) L476
- **Calls (std/external/unresolved):** `.all`, `.as_deref`, `.collect`, `.iter`, `.join`, `.len`, `.map`, `.unwrap_or_default`, `.write_str`
- **Macros:** `format!`, `matches!`, `write!`
- **Called by:** none found in src

#### `kind_name`  (L510-518)
`fn kind_name(kind: ChangeKind) -> &'static str`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `<ChangeError as fmt::Display>::fmt` (changes.rs:451); `changes_text` (changes.rs:526); `outcome_of` (edit/skip.rs:224) (?); `failed_plan_json` (edit/skip.rs:279) (?); `keep_notes` (edit/skip.rs:320) (?); `untracked_kind` (edit/tracked.rs:79) (?); `outcome` (edit/v2_apply.rs:1634) (?)

#### `changes_text`  (L526-555, pub)
`pub fn changes_text(changes: &[Change]) -> String`

- **Does:** `jubarte changes`' listing: one line per change, `{ref}\t{kind}\t{target} \t{author}\t{paragraph}\t{text}[\t({n} elements)][\tinside {ref}][\tholds comment {c}'s mark]` (`-` for an absent author or paragraph; the text quoted, its first 60 characters; `({n} elements)` for a logical change Word wrote as several; `holds...
- **Calls (crate):** `kind_name` (changes.rs:510) L548
- **Calls (std/external/unresolved):** `.as_deref`, `.as_slice`, `.chars`, `.collect`, `.join`, `.len`, `.map`, `.push_str`, `.take`, `.unwrap_or_default`, `.unwrap_or`, `String::new`
- **Macros:** `format!`
- **Called by:** `run_changes` (bin/jubarte.rs:509)

#### `list_changes`  (L564-566, pub)
`pub fn list_changes(docx: &[u8]) -> Result<Vec<Change>, ChangeError>`

- **Does:** Every tracked change, story by story (body, headers, footers, endnotes, footnotes, styles), in document order.
- **Calls (crate):** `changes_in` (changes.rs:643) L565; `open` (changes.rs:1031) L565
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `plan_from_flags` (edit/flags.rs:85); `changed_selection` (edit/for_edit.rs:703); `tests::inline_decisions_freeze_original_ids_and_working_rewrite_hash` (edit/inline.rs:612); `resolution_lines` (edit/resolution.rs:238); `revision_counts` (edit/tracked.rs:404); `public_adjacent_run_replacement_owner_tests::public_keep_replacement_across_adjacent_unicode_runs_restores_every_source_owner` (edit/tracked.rs:2174); `changes` (edit/v2_apply.rs:644); `lower` (edit/visible.rs:95); `Transaction::resolve_visible` (edit/visible.rs:348); `resolve_selected` (edit.rs:1485); `revisions_left` (edit.rs:1571); `deeper_boundary_tests::revision_selection_empty_and_reject_only_preserve_unselected_change` (edit.rs:8914); +2 more

#### `list_logical_changes`  (L579-581, pub)
`pub fn list_logical_changes(docx: &[u8]) -> Result<Vec<Change>, ChangeError>`

- **Does:** Every logical change, story by story, in document order: the changes `jubarte changes` lists.
- **Calls (crate):** `logical_changes_in` (changes.rs:584) L580; `open` (changes.rs:1031) L580
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `run_changes` (bin/jubarte.rs:509); `revision_counts` (edit/tracked.rs:404); `revision_counts` (edit.rs:2396)

#### `logical_changes_in`  (L584-590, pub(crate))
`pub(crate) fn logical_changes_in(pkg: &PartFs) -> Vec<Change>`

- **Does:** The logical changes of an opened package ([`list_logical_changes`]).
- **Calls (crate):** `for_each_story` (changes.rs:657) L586; `logical` (changes/logical.rs:90) L587; `carriers` (changes.rs:1124) L587
- **Calls (std/external/unresolved):** `.extend`, `Vec::new`
- **Called by:** `list_logical_changes` (changes.rs:579); `check_whole_changes` (changes.rs:596); `list_logical_changes_tolerant` (changes.rs:613); `resolve_for_edit_or_cli` (changes.rs:922); `snapshot` (inspect.rs:403); `Opened::summary` (inspect.rs:765)

#### `check_whole_changes`  (L596-602, pub(crate))
`pub(crate) fn check_whole_changes(docx: &[u8], filter: &ChangeFilter) -> Result<(), ChangeError>`

- **Does:** Refuse ids that name only some marks of a logical change ([`ChangeError::PartOfChange`]): the check `accept` and `reject` make, for an edit plan's `resolve_revisions` and the edit flags to make on their source before they resolve anything.
- **Calls (crate):** `ChangeFilter::normalized` (changes.rs:349) L597; `check_whole` (changes/logical.rs:517) L601; `logical_changes_in` (changes.rs:584) L601; `open` (changes.rs:1031) L601
- **Calls (std/external/unresolved):** `.is_none`, `Ok`
- **Called by:** `lower` (edit/visible.rs:95); `resolve_selected` (edit.rs:1485)

#### `list_changes_tolerant`  (L607-609, pub(crate))
`pub(crate) fn list_changes_tolerant(docx: &[u8]) -> Result<Vec<Change>, ChangeError>`

- **Does:** [`list_changes`] for a view that already renders the package: no admission, and a part that does not parse lists nothing, so the view's count fails only where the package itself does.
- **Calls (crate):** `changes_in` (changes.rs:643) L608; `tolerant` (changes.rs:620) L608
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `resolve_with_report` (changes.rs:822); `AnchorChanges::records` (edit/comment_settlement.rs:78); `convert_marks` (markdown/from_docx/mod.rs:108)

#### `list_logical_changes_tolerant`  (L613-615, pub(crate))
`pub(crate) fn list_logical_changes_tolerant(docx: &[u8]) -> Result<Vec<Change>, ChangeError>`

- **Does:** [`list_logical_changes`] as [`list_changes_tolerant`] lists the marks: the changes a view's tags name.
- **Calls (crate):** `logical_changes_in` (changes.rs:584) L614; `tolerant` (changes.rs:620) L614
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `member_wholes` (changes.rs:788); `convert` (markdown/from_docx/mod.rs:96)

#### `tolerant`  (L620-637)
`fn tolerant(docx: &[u8]) -> Result<PartFs, ChangeError>`

- **Does:** The package a view reads, opened without admission.
- **Calls (crate):** `strict_to_transitional_docx` (strict_translation.rs:749) L623; `PartFs::open` (opc/mod.rs:585) L625; `revision_bearing_parts` (revision_processor.rs:3716) L626; `PartFs::part_string` (opc/mod.rs:636) (by name) L627; `nests_deeper` (xmllinq/parse.rs:151) (by name) L629
- **Calls (std/external/unresolved):** `.is_some_and`, `.map_err`, `.to_string`, `ChangeError::Package`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `list_changes_tolerant` (changes.rs:607); `list_logical_changes_tolerant` (changes.rs:613)

#### `changes_in`  (L643-653, pub(crate))
`pub(crate) fn changes_in(pkg: &PartFs) -> Vec<Change>`

- **Does:** The changes of an opened (transitional) package, as [`list_changes`] lists them.
- **Calls (crate):** `for_each_story` (changes.rs:657) L645; `carriers` (changes.rs:1124) L647
- **Calls (std/external/unresolved):** `.extend`, `.filter_map`, `.into_iter`, `Vec::new`
- **Called by:** `list_changes` (changes.rs:564); `list_changes_tolerant` (changes.rs:607); `resolve_for_edit_or_cli` (changes.rs:922); `settle` (edit.rs:1617); `table_line` (markdown/from_docx/agent.rs:2378)

#### `for_each_story`  (L657-674)
`fn for_each_story(pkg: &PartFs, mut each: impl FnMut(&Dom, NodeId, &str))`

- **Does:** Each listed story's parsed part, in [`list_changes`] order: every revision-bearing part but the comments part.
- **Calls (crate):** `main_part` (changes.rs:1039) L658; `comments_part` (revision_processor.rs:3708) L659; `revision_bearing_parts` (revision_processor.rs:3716) L660; `PartFs::part_string` (opc/mod.rs:636) L664; `Dom::new` (xmllinq/mod.rs:387) L667; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L668; `Dom::root` (xmllinq/mod.rs:822) L669; `story_id` (changes.rs:1046) L672
- **Calls (std/external/unresolved):** `.as_ref`, `Some`, `each`
- **Called by:** `logical_changes_in` (changes.rs:584); `changes_in` (changes.rs:643)

#### `accept_changes`  (L689-691, pub)
`pub fn accept_changes(docx: &[u8], filter: &ChangeFilter) -> Result<Vec<u8>, ChangeError>`

- **Does:** Accept the changes `filter` selects; the others stay tracked.
- **Calls (crate):** `resolve` (changes.rs:872) L690
- **Calls (std/external/unresolved):** `.map`
- **Called by:** `public_adjacent_run_replacement_owner_tests::public_keep_replacement_across_adjacent_unicode_runs_restores_every_source_owner` (edit/tracked.rs:2174)

#### `reject_changes`  (L699-701, pub)
`pub fn reject_changes(docx: &[u8], filter: &ChangeFilter) -> Result<Vec<u8>, ChangeError>`

- **Does:** Reject the changes `filter` selects; the others stay tracked.
- **Calls (crate):** `resolve` (changes.rs:872) L700
- **Calls (std/external/unresolved):** `.map`
- **Called by:** `public_adjacent_run_replacement_owner_tests::public_keep_replacement_across_adjacent_unicode_runs_restores_every_source_owner` (edit/tracked.rs:2174); `audit_tracked` (validate.rs:3027)

#### `why_removed`  (L731-783, pub(crate))
`pub(crate) fn why_removed( changes: &[Change], thread: &[String], resolution: Resolution, settled: Option<&[String]>, wholes: &HashMap<String, String>, ) -> String`

- **Does:** `anchor removed by accepting r13`: the verb of `resolution`, and the change of `changes` (the source's) that held the reference mark of a comment of `thread` (the root's first, then each reply's).
- **Calls (crate):** `named` (ambiguous, 5 candidates) L766,774
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_ref`, `.clone`, `.collect`, `.contains`, `.filter`, `.find_map`, `.find`, `.get`, `.is_some`, `.iter`, `.map`, `.unwrap_or`, `Some`, `removes`
- **Macros:** `format!`
- **Called by:** `resolve_with_report` (changes.rs:822); `AnchorChanges::records` (edit/comment_settlement.rs:78)

#### `member_wholes`  (L788-798, pub(crate))
`pub(crate) fn member_wholes(docx: &[u8]) -> Result<HashMap<String, String>, ChangeError>`

- **Does:** Each mark of a change of several marks, by its id, with the id of that change: a removed comment's reason names the whole change, as `changes` lists it and a selection names it (#1261).
- **Calls (crate):** `list_logical_changes_tolerant` (changes.rs:613) L789
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.flat_map`, `.into_iter`, `.map`, `Ok`
- **Called by:** `resolve_with_report` (changes.rs:822); `AnchorChanges::records` (edit/comment_settlement.rs:78)

#### `accept_changes_with_report`  (L804-809, pub)
`pub fn accept_changes_with_report( docx: &[u8], filter: &ChangeFilter, ) -> Result<ResolvedChanges, ChangeError>`

- **Does:** Accept selected changes and report comments that disappeared with them.
- **Calls (crate):** `resolve_with_report` (changes.rs:822) L808
- **Called by:** none found in src

#### `reject_changes_with_report`  (L815-820, pub)
`pub fn reject_changes_with_report( docx: &[u8], filter: &ChangeFilter, ) -> Result<ResolvedChanges, ChangeError>`

- **Does:** Reject selected changes and report comments that disappeared with them.
- **Calls (crate):** `resolve_with_report` (changes.rs:822) L819
- **Called by:** none found in src

#### `resolve_with_report`  (L822-870)
`fn resolve_with_report( docx: &[u8], filter: &ChangeFilter, resolution: Resolution, ) -> Result<ResolvedChanges, ChangeError>`

- **Does:** _no doc comment_
- **Calls (crate):** `resolve` (changes.rs:872) L827; `list_comments` (comments.rs:132) L837; `list_changes_tolerant` (changes.rs:607) L838; `member_wholes` (changes.rs:788) L839; `ChangeFilter::normalized` (changes.rs:349) L840; `ChangeFilter::matches` (changes.rs:405) L843; `why_removed` (changes.rs:731) L861
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.cloned`, `.collect`, `.contains`, `.filter`, `.find`, `.is_empty`, `.iter`, `.map_err`, `.map`, `.push`, `.to_string`, `ChangeError::Package`, `Ok`, `Some`, `Vec::new`
- **Macros:** `format!`, `vec!`
- **Called by:** `accept_changes_with_report` (changes.rs:804); `reject_changes_with_report` (changes.rs:815)

#### `resolve`  (L872-879)
`fn resolve( docx: &[u8], filter: &ChangeFilter, resolution: Resolution, ) -> Result<(Vec<u8>, Vec<String>, usize), ChangeError>`

- **Does:** _no doc comment_
- **Calls (crate):** `resolve_for_edit_or_cli` (changes.rs:922) L877
- **Calls (std/external/unresolved):** `.map`
- **Called by:** `run_convert_any` (bin/jubarte.rs:3458); `accept_changes` (changes.rs:689); `reject_changes` (changes.rs:699); `resolve_with_report` (changes.rs:822)

#### `accept_elements`  (L890-892, pub)
`pub fn accept_elements(docx: &[u8], filter: &ChangeFilter) -> Result<Vec<u8>, ChangeError>`

- **Does:** Accept the elements `filter` selects, each `w:ins`, `w:del` or other mark by its own id ([`list_changes`]), even when it is only part of a logical change: for tools that work mark by mark, such as a comparison with Word's own handling of each element.
- **Calls (crate):** `resolve_for_edit_or_cli` (changes.rs:922) L891
- **Calls (std/external/unresolved):** `.map`
- **Called by:** `patch_own_changes` (markdown/unified.rs:308)

#### `reject_elements`  (L900-902, pub)
`pub fn reject_elements(docx: &[u8], filter: &ChangeFilter) -> Result<Vec<u8>, ChangeError>`

- **Does:** Reject the elements `filter` selects, as [`accept_elements`] accepts them.
- **Calls (crate):** `resolve_for_edit_or_cli` (changes.rs:922) L901
- **Calls (std/external/unresolved):** `.map`
- **Called by:** none found in src

#### `resolve_for_edit`  (L909-920, pub(crate))
`pub(crate) fn resolve_for_edit( docx: &[u8], filter: &ChangeFilter, resolution: Resolution, keep_comments: bool, ) -> Result<CommentResolution, ChangeError>`

- **Does:** Settlement driven by an edit plan, with stable comment ids and opt-in empty anchors.
- **Calls (crate):** `resolve_for_edit_or_cli` (changes.rs:922) L918
- **Calls (std/external/unresolved):** `.map`, `Some`
- **Called by:** `apply_bound` (edit/v2_apply.rs:121); `resolve_selected` (edit.rs:1485); `settle` (edit.rs:1617)

#### `resolve_for_edit_or_cli`  (L922-999)
`fn resolve_for_edit_or_cli( docx: &[u8], filter: &ChangeFilter, resolution: Resolution, edit: Option<bool>, whole: bool, ) -> Result<ResolutionOutcome, ChangeError>`

- **Does:** _no doc comment_
- **Calls (crate):** `ChangeFilter::normalized` (changes.rs:349) L929; `open` (changes.rs:1031) L930; `changes_in` (changes.rs:643) L931; `ChangeFilter::check` (changes.rs:387) L932; `check_whole` (changes/logical.rs:517) L934; `logical_changes_in` (changes.rs:584) L934; `comment_changes_in` (changes.rs:1003) L938,989; `ChangeFilter::matches` (changes.rs:405) L943,957; `main_part` (changes.rs:1039) L952; `carriers` (changes.rs:1124) L954; `story_id` (changes.rs:1046) L954; `deleted_text` (changes.rs:1504) L965; `recorded_marks` (changes.rs:1419) L966; `Dom::set_name` (xmllinq/mod.rs:539) (by name) L971; `XName::get` (xmllinq/mod.rs:180) L971; `XName::local_name` (xmllinq/mod.rs:208) (by name) L971; `resolve_package_for_edit` (revision_processor.rs:3889) L976; `resolve_package` (revision_processor.rs:3879) L983; `PartFs::to_zip` (opc/mod.rs:749) (by name) L990
- **Calls (std/external/unresolved):** `.any`, `.as_ref`, `.chain`, `.collect`, `.contains`, `.filter`, `.flat_map`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.map_err`, `.map`, `.name`, `.to_string`, `.to_vec`, `ChangeError::Package`, `Ok`, `Some`, `Vec::new`, `std::iter::once`
- **Called by:** `resolve` (changes.rs:872); `accept_elements` (changes.rs:890); `reject_elements` (changes.rs:900); `resolve_for_edit` (changes.rs:909)

#### `comment_changes_in`  (L1003-1019)
`fn comment_changes_in(pkg: &PartFs) -> Vec<Change>`

- **Does:** The listed changes of the comments part only (empty when the package has none). The same changes are in [`changes_in`] as `comments.rN`.
- **Calls (crate):** `comments_part` (revision_processor.rs:3708) L1004; `PartFs::part_string` (opc/mod.rs:636) L1007; `Dom::new` (xmllinq/mod.rs:387) L1010; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L1011; `Dom::root` (xmllinq/mod.rs:822) L1012; `carriers` (changes.rs:1124) L1015
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.into_iter`, `Vec::new`
- **Called by:** `resolve_for_edit_or_cli` (changes.rs:922)

#### `open`  (L1031-1037)
`fn open(docx: &[u8]) -> Result<PartFs, ChangeError>`

- **Does:** The package, Strict normalized to transitional as `inspect` reads it.
- **Calls (crate):** `admit_package` (document_comparer.rs:4927) L1032; `strict_to_transitional_docx` (strict_translation.rs:749) L1034; `PartFs::open` (opc/mod.rs:585) L1036
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`, `ChangeError::Package`
- **Called by:** `audit_stories` (changes.rs:168); `list_changes` (changes.rs:564); `list_logical_changes` (changes.rs:579); `check_whole_changes` (changes.rs:596); `resolve_for_edit_or_cli` (changes.rs:922); `Origin::build` (edit/origin.rs:260) (?); `restore` (edit/ranges.rs:432) (?); `record_mid_changes` (edit/sections.rs:453) (?); `rewrite` (edit/whole.rs:130) (?); `collect_fields` (fields.rs:281) (?)

#### `main_part`  (L1039-1042)
`fn main_part(pkg: &PartFs) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:1070) L1040
- **Calls (std/external/unresolved):** `.to_string`, `.unwrap_or_else`
- **Called by:** `audit_stories` (changes.rs:168); `for_each_story` (changes.rs:657); `resolve_for_edit_or_cli` (changes.rs:922)

#### `story_id`  (L1046-1051)
`fn story_id(main: &str, part: &str) -> String`

- **Does:** `body` for the main part, else the part's file stem (`header1`, `footnotes`, `styles`).
- **Calls (crate):** `part_story` (ids.rs:392) L1050
- **Calls (std/external/unresolved):** `.to_string`, `.trim_start_matches`
- **Called by:** `audit_stories` (changes.rs:168); `for_each_story` (changes.rs:657); `resolve_for_edit_or_cli` (changes.rs:922)

#### `revision_date`  (L1082-1090, pub(crate))
`pub(crate) fn revision_date<'a>( mut attribute: impl FnMut(&str, &str) -> Option<&'a str>, ) -> Option<String>`

- **Does:** The recorded instant with the author's offset, else the stored legacy date.
- **Calls (crate):** `attribute` (markdown/unified.rs:756) (by name) L1085,1089; `reported_date` (changes.rs:1093) L1089
- **Called by:** `carriers` (changes.rs:1124); `get_revisions_from_body` (comparer/revisions.rs:151); `get_revisions_from_note_definitions` (comparer/revisions.rs:226); `get_format_change_revisions` (comparer/revisions.rs:366); `Element::revision_date` (markdown/from_docx/ooxml.rs:354)

#### `reported_date`  (L1093-1121, pub(crate))
`pub(crate) fn reported_date(local: Option<&str>, utc: Option<&str>) -> Option<String>`

- **Does:** Revisions and comments share Word's wall-clock/UTC pair encoding.
- **Calls (crate):** `timestamp_order_key` (convert/mod.rs:12427) L1095,1100; `timestamp_with_offset` (convert/mod.rs:12443) L1116
- **Calls (std/external/unresolved):** `.abs`, `.and_then`, `.ends_with`, `.filter`, `.map_or`, `.map`, `.next`, `.split`, `.strip_prefix`, `.to_string`, `.unwrap_or_default`, `Some`
- **Called by:** `revision_date` (changes.rs:1082); `CommentFamily::reported_date` (comments.rs:637)

#### `carriers`  (L1124-1243)
`fn carriers(dom: &Dom, root: NodeId, story: &str) -> Vec<Carrier>`

- **Does:** The revision elements under `root`, in document order.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1131; `Dom::name` (xmllinq/mod.rs:526) L1132; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1135; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1138; `Dom::attribute` (xmllinq/mod.rs:874) L1139,1142,1209,1210; `in_recorded_properties` (changes.rs:1406) L1175; `target` (changes.rs:1480) L1182; `paragraph_move` (changes.rs:1443) L1191; `owner_text` (changes.rs:1528) L1198; `text_of` (changes.rs:1518) L1199; `ChangeId::short` (ids.rs:155) (by name) L1205,1206; `ChangeId::new` (ids.rs:142) L1205,1206; `revision_date` (changes.rs:1082) L1210; `XName::get` (xmllinq/mod.rs:180) L1210; `fold_twins` (changes.rs:1280) L1225; `fold_style_pairs` (changes.rs:1342) L1226; `paragraph_index` (changes.rs:1377) L1231; `held_comment_marks` (changes.rs:1249) L1234; `holder` (changes.rs:1455) L1235; `paragraph_of` (changes.rs:1388) L1238; `paragraph_id` (ids.rs:410) L1239
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.as_str`, `.clone`, `.collect`, `.contains`, `.filter_map`, `.iter`, `.last`, `.map_or_else`, `.map_or`, `.map`, `.or`, `.push`, `.remove`, `.rposition`, `.then`, `.to_string`, `.unwrap_or`, `Group::Move`, `Group::Own`, `Some`, `String::new`, `Vec::new`, `W::name`
- **Macros:** `format!`
- **Called by:** `audit_stories` (changes.rs:168); `logical_changes_in` (changes.rs:584); `changes_in` (changes.rs:643); `resolve_for_edit_or_cli` (changes.rs:922); `comment_changes_in` (changes.rs:1003)

#### `held_comment_marks`  (L1249-1269)
`fn held_comment_marks(dom: &Dom, e: NodeId, target: &str) -> Vec<String>`

- **Does:** The comments whose reference mark the content of change `e` holds (`c4`, `c5`), in document order: what resolving the change so its content goes removes.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L1252,1253; `Dom::descendants` (xmllinq/mod.rs:668) L1258; `Dom::attribute` (xmllinq/mod.rs:874) L1261
- **Calls (std/external/unresolved):** `.contains`, `.copied`, `.first`, `.map`, `.push`, `.unwrap_or_default`, `Some`, `Vec::new`, `W::id`, `W::name`, `W::tc`, `W::tr`
- **Macros:** `format!`
- **Called by:** `carriers` (changes.rs:1124)

#### `fold_twins`  (L1280-1335)
`fn fold_twins(dom: &Dom, carriers: &mut [Carrier])`

- **Does:** Word stores a text box twice, in an `mc:AlternateContent`'s `mc:Choice` and its `mc:Fallback`, and gives the Fallback copy of each change its own consecutive `w:id`.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L1288,1289,1303; `Dom::name_is` (xmllinq/mod.rs:535) L1297; `Dom::parent` (xmllinq/mod.rs:597) L1310; `Dom::element` (xmllinq/mod.rs:655) L1310; `key` (ambiguous: `same_path::{fn key}` (bin/jubarte.rs:201); `key` (edit/ranges.rs:25); `key` (markdown/patch.rs:316)) L1325
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.clone`, `.collect`, `.contains`, `.enumerate`, `.extend`, `.filter`, `.is_empty`, `.is_some`, `.iter`, `.len`, `.map`, `.position`, `.push`, `.sort_by_key`, `MC::name`, `Some`, `Vec::new`, `std::cmp::Reverse`, `under`
- **Called by:** `carriers` (changes.rs:1124)

#### `fold_style_pairs`  (L1342-1373)
`fn fold_style_pairs(dom: &Dom, carriers: &mut [Carrier])`

- **Does:** Word records a style's paragraph and run formatting changes (`w:style/w:pPr/w:pPrChange`, `w:style/w:rPr/w:rPrChange`) under one `w:id`: one change.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L1346,1347; `Dom::name` (xmllinq/mod.rs:526) L1349; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1350; `Dom::name_is` (xmllinq/mod.rs:535) L1351; `change_id` (ids.rs:187) L1362
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.get`, `.insert`, `.iter_mut`, `.map_or_else`, `.then_some`, `HashMap::new`, `W::name`, `own_style`
- **Called by:** `carriers` (changes.rs:1124)

#### `paragraph_index`  (L1377-1383)
`fn paragraph_index(dom: &Dom, root: NodeId) -> HashMap<NodeId, usize>`

- **Does:** Each listed paragraph of the story under `root` (the body's or a header's, footer's or notes part's, as `inspect` numbers them) by node.
- **Calls (crate):** `story_paragraph_nodes` (inspect.rs:1178) L1378
- **Calls (std/external/unresolved):** `.collect`, `.enumerate`, `.into_iter`, `.map`
- **Called by:** `audit_stories` (changes.rs:168); `carriers` (changes.rs:1124)

#### `paragraph_of`  (L1388-1402)
`fn paragraph_of(dom: &Dom, e: NodeId, index: &HashMap<NodeId, usize>) -> Option<usize>`

- **Does:** The paragraph a revision element sits in: the nearest listed paragraph around it (a text box's paragraphs are not listed, so its anchor's), or the first listed paragraph of the row, cell or table around it.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L1389; `Dom::name` (xmllinq/mod.rs:526) L1396; `Dom::descendants` (xmllinq/mod.rs:668) L1398
- **Calls (std/external/unresolved):** `.chain`, `.contains`, `.copied`, `.filter`, `.find_map`, `.get`, `.into_iter`, `.is_some_and`, `.iter`, `Some`, `W::p`, `W::tbl`, `W::tc`, `W::tr`, `std::iter::once`
- **Called by:** `audit_stories` (changes.rs:168); `carriers` (changes.rs:1124); `Anchors::scan` (comments.rs:251) (?); `Transaction::resolve_visible` (edit/visible.rs:348) (?)

#### `in_recorded_properties`  (L1406-1411)
`fn in_recorded_properties(dom: &Dom, e: NodeId) -> bool`

- **Does:** True when `e` sits in the properties a formatting change records: a mark there is history, part of that change.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L1407; `Dom::name` (xmllinq/mod.rs:526) L1408; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1409; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1409
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.into_iter`, `.is_some_and`
- **Called by:** `carriers` (changes.rs:1124)

#### `recorded_marks`  (L1419-1438)
`fn recorded_marks(dom: &Dom, e: NodeId) -> Vec<NodeId>`

- **Does:** What a formatting change `e` records: the revision marks inside it (a cell's `w:cellIns`, `w:cellDel` or `w:cellMerge` in its `w:tcPrChange`) and the one formatting history the schema allows there, a list's `w:numberingChange` in its `w:pPrChange`.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L1420,1429; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1422,1432,1434; `Dom::descendants` (xmllinq/mod.rs:668) L1426; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1430
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter`, `.into_iter`, `.is_some_and`, `Vec::new`
- **Macros:** `matches!`
- **Called by:** `resolve_for_edit_or_cli` (changes.rs:922)

#### `paragraph_move`  (L1443-1449)
`fn paragraph_move(dom: &Dom, e: NodeId, start: &str) -> Option<String>`

- **Does:** The move a paragraph mark `e` belongs to: the last range of its side (`start`) opening inside its paragraph.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L1444; `Dom::descendants` (xmllinq/mod.rs:668) L1445; `Dom::attribute` (xmllinq/mod.rs:874) L1448
- **Calls (std/external/unresolved):** `.find_map`, `.into_iter`, `.map`, `.next`, `.rev`, `Some`, `W::name`, `W::p`
- **Called by:** `carriers` (changes.rs:1124)

#### `holder`  (L1455-1478)
`fn holder(dom: &Dom, e: NodeId, ids: &HashMap<NodeId, String>) -> Option<String>`

- **Does:** The id of the innermost change around `e` whose content holds it: an ancestor revision element, the row or cell an ancestor `w:tr` / `w:tc` marks inserted or deleted, or the mark of the paragraph whose `pPr` holds `e` (the paragraph's properties go with its mark).
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L1457; `Dom::name` (xmllinq/mod.rs:526) L1461,1475; `Dom::element` (xmllinq/mod.rs:655) L1471; `Dom::elements` (xmllinq/mod.rs:640) L1472; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1475
- **Calls (std/external/unresolved):** `.clone`, `.cloned`, `.contains`, `.filter`, `.find_map`, `.get`, `.into_iter`, `.is_some_and`, `Some`, `W::p_pr`, `W::r_pr`, `W::tc_pr`, `W::tc`, `W::tr_pr`, `W::tr`
- **Called by:** `carriers` (changes.rs:1124)

#### `target`  (L1480-1499)
`fn target(dom: &Dom, e: NodeId, local: &str) -> &'static str`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L1487,1488,1490; `Dom::name` (xmllinq/mod.rs:526) L1487,1491
- **Calls (std/external/unresolved):** `.and_then`, `.contains`, `.starts_with`, `Some`, `W::p_pr`, `W::r_pr`, `W::tr_pr`
- **Called by:** `carriers` (changes.rs:1124); `carry_with_own_relationships` (comparer/parts.rs:627) (?); `tests::rel_attributes_survive_round_trips_unescaped` (opc/mod.rs:1436) (?)

#### `deleted_text`  (L1504-1515)
`fn deleted_text(dom: &Dom, e: NodeId) -> Vec<NodeId>`

- **Does:** The `w:delText` / `w:delInstrText` a `w:del` holds itself (not through a nested `w:del`): the resolution rewrites deleted text wherever it sits, so a kept deletion keeps its text only when that text sits out too.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L1506,1512; `Dom::descendants` (xmllinq/mod.rs:668) L1510; `Dom::ancestors` (xmllinq/mod.rs:770) L1513
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.filter`, `.first`, `.into_iter`, `.is_some_and`, `Some`, `Vec::new`, `W::del`, `W::name`
- **Called by:** `resolve_for_edit_or_cli` (changes.rs:922)

#### `text_of`  (L1518-1525)
`fn text_of(dom: &Dom, e: NodeId) -> String`

- **Does:** Inserted, deleted or moved text under `e`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1520; `Dom::name` (xmllinq/mod.rs:526) L1522; `Dom::value` (xmllinq/mod.rs:1185) L1523
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_some_and`, `.map`, `W::name`, `W::t`
- **Called by:** `folded` (changes/logical.rs:316) (?); `audit_stories` (changes.rs:168); `carriers` (changes.rs:1124); `owner_text` (changes.rs:1528); `split_head_short_title_long_del_mix` (comparer/finalize.rs:4870) (?); `change_lines` (debug.rs:1480) (?); `compare_documents_impl` (document_comparer.rs:6521) (?); `ParaFormat::read` (markdown/view_stories.rs:94) (?); `story` (markdown/view_stories.rs:713) (?)

#### `owner_text`  (L1528-1534)
`fn owner_text(dom: &Dom, change: NodeId) -> String`

- **Does:** The text of the run or paragraph whose properties a change records.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L1529,1530; `Dom::name` (xmllinq/mod.rs:526) L1531; `text_of` (changes.rs:1518) L1532
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.is_some_and`, `.map`, `.unwrap_or_default`, `W::p`, `W::r`
- **Called by:** `carriers` (changes.rs:1124)
