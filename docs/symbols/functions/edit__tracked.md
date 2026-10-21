# Functions and methods in `src/edit/tracked.rs`

44 non-test, 12 test. Part of the [symbol manifest](../README.md).

#### `Stamp::container`  (L50-57)
`fn container(&mut self, dom: &mut Dom, name: XName) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L51; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L52,54,55
- **Calls (std/external/unresolved):** `.to_string`, `Some`, `W::author`, `W::date`, `W::id`
- **Called by:** `emit` (edit/tracked.rs:452) (?); `insert_paragraph_content` (edit/tracked.rs:829) (?); `insert_table_content` (edit/tracked.rs:848) (?); `track_properties` (edit/tracked.rs:888) (?); `track_run_format` (edit/tracked.rs:924) (?); `emit_text` (edit/tracked.rs:1035) (?); `wrap_deleted` (edit/tracked.rs:1609) (?); `mark` (edit/tracked.rs:1682) (?); `rewrite_one` (edit/whole.rs:229) (?)

#### `Stamp::insertion_origins`  (L59-73)
`fn insertion_origins(&self, dom: &Dom) -> BTreeMap<String, String>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L63; `Dom::attribute` (xmllinq/mod.rs:874) L66; `ChangeId::short` (ids.rs:155) (by name) L68; `ChangeId::new` (ids.rs:142) L68
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.contains`, `.filter_map`, `.iter`, `Some`, `W::id`
- **Called by:** `redline` (edit/tracked.rs:283)

#### `untracked_kind`  (L79-82)
`fn untracked_kind(tx: &Transaction<'_>, operation: usize) -> Option<&'static str>`

- **Does:** The operation kind of `resolved` when the emitter cannot write it as a tracked change: such a plan goes through the comparer, or is refused under `keep`, never left out of the redline.
- **Calls (crate):** `super::kind_name` (ambiguous: `kind_name` (changes.rs:510); `kind_name` (edit.rs:6943)) L80; `operation_supports_keep` (edit.rs:1912) L81
- **Calls (std/external/unresolved):** `.then_some`
- **Called by:** `emits_all` (edit/tracked.rs:85); `check` (edit/tracked.rs:107)

#### `emits_all`  (L85-89, pub(super))
`pub(super) fn emits_all(tx: &Transaction<'_>) -> bool`

- **Does:** Whether the emitter writes every operation of `tx` as tracked changes.
- **Calls (crate):** `untracked_kind` (edit/tracked.rs:79) L88
- **Calls (std/external/unresolved):** `.all`, `.is_none`, `.iter`
- **Called by:** `Transaction::emits_directly` (edit.rs:2828)

#### `formatting_tracked`  (L93-103)
`fn formatting_tracked(dom: &Dom, paragraph: NodeId, start: usize, end: usize) -> Option<NodeId>`

- **Does:** The tracked formatting change (`w:rPrChange`) of the first run holding `[start, end)` of `paragraph` that has one.
- **Calls (crate):** `project_paragraph` (inspect.rs:1265) L94; `Dom::element` (xmllinq/mod.rs:655) L100,101
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.find_map`, `.iter`, `W::r_pr_change`, `W::r_pr`
- **Called by:** `check` (edit/tracked.rs:107)

#### `check`  (L107-195, pub(super))
`pub(super) fn check(tx: &Transaction<'_>) -> Result<(), EditError>`

- **Does:** Refusals that only `keep` needs, checked on the untouched working source.
- **Calls (crate):** `Transaction::conflict` (edit.rs:5870) L114; `Transaction::attach_revision_refusal` (edit/refusal.rs:195) L117; `untracked_kind` (edit/tracked.rs:79) L120; `Transaction::revision_hint` (edit.rs:5259) L130; `moved_in` (edit/tracked.rs:1652) L134; `named` (ambiguous, 5 candidates) L135,146,157,165,175,183; `Dom::next_element` (xmllinq/mod.rs:855) (by name) L139; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L141; `previous_paragraph` (edit/tracked.rs:1659) L143; `mark_revision` (edit/tracked.rs:1634) L144,156; `paragraph_format_change` (edit/tracked.rs:1645) L164,173; `formatting_tracked` (edit/tracked.rs:93) L182
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.find_map`, `.into`, `.is_none_or`, `.iter`, `.map`, `.take`, `Err`, `Ok`, `Some`, `W::p`, `refuse`
- **Macros:** `format!`
- **Called by:** `prepare` (edit/prepared.rs:211); `apply_bound` (edit/v2_apply.rs:121); `resolved` (edit.rs:2358)

#### `result`  (L199-201, pub(super))
`pub(super) fn result(tx: &Transaction<'_>, clean: Vec<u8>) -> Result<EditResult, EditError>`

- **Does:** The result under `keep`: the clean copy as `finish` made it, the redline by emission, and revision counts of the plan's own changes.
- **Calls (crate):** `result_with_order` (edit/tracked.rs:252) L200
- **Called by:** `produce` (edit/prepared.rs:928)

#### `result_v2_with_paragraphs`  (L231-250, pub(super))
`pub(super) fn result_v2_with_paragraphs( tx: &Transaction<'_>, clean: Vec<u8>, ) -> Result<(EditResult, TrackedParagraphs), EditError>`

- **Does:** As [`result_v2`], also returning the tracked primary's paragraph walks (see [`TrackedParagraphs`]).
- **Calls (crate):** `redline` (edit/tracked.rs:283) L235; `Transaction::report` (edit.rs:3161) L236; `paragraphs` (inspect.rs:312) L237; `revision_counts` (edit/tracked.rs:404) L240; `splits` (edit/tracked.rs:373) L241
- **Calls (std/external/unresolved):** `.len`, `.map`, `.unwrap_or`, `Ok`
- **Called by:** `apply_bound` (edit/v2_apply.rs:121)

#### `result_with_order`  (L252-269)
`fn result_with_order( tx: &Transaction<'_>, clean: Vec<u8>, array_order: bool, ) -> Result<EditResult, EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `redline` (edit/tracked.rs:283) L257; `Transaction::report` (edit.rs:3161) L258; `paragraphs` (inspect.rs:312) L259; `revision_counts` (edit/tracked.rs:404) L262; `splits` (edit/tracked.rs:373) L263
- **Calls (std/external/unresolved):** `.len`, `.map`, `.unwrap_or`, `Ok`
- **Called by:** `result` (edit/tracked.rs:199)

#### `redline`  (L283-368)
`fn redline( tx: &Transaction<'_>, array_order: bool, final_views: bool, ) -> Result<(Vec<u8>, u64, TrackedParagraphs), EditError>`

- **Does:** Replay `tx`'s resolved operations on a fresh copy of its base as tracked changes.
- **Calls (crate):** `Transaction::start` (edit.rs:2680) L296; `Transaction::deleted_text_comment` (edit.rs:6456) L306; `Dom::descendants` (xmllinq/mod.rs:668) L319; `Dom::attribute` (xmllinq/mod.rs:874) L320; `ChangeId::short` (ids.rs:155) (by name) L325; `ChangeId::new` (ids.rs:142) L325; `first_free_id` (edit/tracked.rs:427) L335; `emit` (edit/tracked.rs:452) L339; `thread_markers` (edit/tracked.rs:956) L340; `separate_tracked` (edit/tracked.rs:870) L344; `Transaction::paragraph_walk` (edit.rs:5097) L349; `paragraph_final_state` (edit/v2_apply.rs:547) L353; `Stamp::insertion_origins` (edit/tracked.rs:59) L356; `Transaction::finish` (edit.rs:6598) L357
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.enumerate`, `.filter_map`, `.insert`, `.iter`, `.map`, `.push`, `.sort_by_key`, `.then`, `.to_string`, `HashMap::new`, `Ok`, `Some`, `W::id`, `W::ins`
- **Called by:** `result_v2_with_paragraphs` (edit/tracked.rs:231); `result_with_order` (edit/tracked.rs:252)

#### `splits`  (L373-398)
`fn splits(origins: &BTreeMap<String, String>) -> Vec<super::RevisionSplit>`

- **Does:** The source insertions the emission split, from each emitted piece's origin: a source id with more than one piece, or whose one piece took a new id.
- **Calls (crate):** `change_id` (ids.rs:187) L381
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.entry`, `.filter`, `.first`, `.into_iter`, `.len`, `.map`, `.ok`, `.or_default`, `.parse`, `.push`, `.sort_by_key`, `.to_string`, `BTreeMap::new`, `Some`, `order`
- **Called by:** `result_v2_with_paragraphs` (edit/tracked.rs:231); `result_with_order` (edit/tracked.rs:252)

#### `revision_counts`  (L404-423)
`fn revision_counts(redline: &[u8], author: &str, date: &str, floor: u64) -> RevisionCounts`

- **Does:** Counts of the redline's logical changes holding a mark stamped with `author` and `date` whose `w:id` is at least `floor`: the edit's own.
- **Calls (crate):** `list_changes` (changes.rs:564) L405; `change_id` (ids.rs:187) L411; `list_logical_changes` (changes.rs:579) L417; `RevisionCounts::from_changes` (edit.rs:1149) L418; `Change::element_ids` (changes.rs:118) (by name) L421
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.as_str`, `.collect`, `.contains`, `.filter`, `.is_some_and`, `.iter`, `.map`, `.ok`, `.parse`, `.unwrap_or_default`, `Some`
- **Called by:** `produce` (edit/prepared.rs:928) (?); `result_v2_with_paragraphs` (edit/tracked.rs:231); `result_with_order` (edit/tracked.rs:252)

#### `first_free_id`  (L427-448)
`fn first_free_id(opened: &Opened, comments: &[(u32, String)]) -> u64`

- **Does:** One past the highest `w:id` in every revision-bearing part and among the plan's comments, so a new revision id collides with nothing.
- **Calls (crate):** `revision_bearing_parts` (revision_processor.rs:3687) L429; `PartFs::part_string` (opc/mod.rs:636) L430; `Dom::new` (xmllinq/mod.rs:387) L433; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L434; `Dom::root` (xmllinq/mod.rs:822) L435; `Dom::descendants` (xmllinq/mod.rs:668) L438; `Dom::attribute` (xmllinq/mod.rs:874) L439
- **Calls (std/external/unresolved):** `.and_then`, `.iter`, `.map_or`, `.map`, `.max`, `.ok`, `.parse`, `Some`, `W::id`, `u64::from`
- **Called by:** `redline` (edit/tracked.rs:283)

#### `emit`  (L452-808)
`fn emit( t: &mut Transaction<'_>, comment_ids: &BTreeMap<usize, u32>, stamp: &mut Stamp<'_>, array_order: bool, ) -> Vec<NodeId>`

- **Does:** The steps of `Transaction::apply`, in its order, as tracked changes.
- **Calls (crate):** `track_run_format` (edit/tracked.rs:924) L471; `project_paragraph` (inspect.rs:1265) L518,550,683,785; `sort_edits` (edit.rs:7694) L522; `coalesce_insertions` (edit/tracked.rs:810) L532; `anchor_comment` (edit.rs:7781) L540,588,684,786; `apply_text_edit` (edit.rs:7403) L551; `insertion_replacement` (edit/tracked.rs:987) L564; `emit_text` (edit/tracked.rs:1035) L569; `new_position` (edit.rs:7127) L576,581,582; `anchor_span` (edit.rs:7837) L598; `revision_in` (edit/visible.rs:287) L609; `.container` (ambiguous: `Stamp::container` (edit/tracked.rs:50); `Stamp::container` (edit/whole.rs:202)) L612,771; `copy_runs` (edit/visible.rs:267) L613; `.add` (ambiguous, 5 candidates) L614,766,770,772,773; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L616,760; `comment_around` (edit/visible.rs:299) L620; `place_beside` (edit/tracked.rs:1386) L622; `toc_paragraphs` (edit.rs:8208) L642; `build_paragraph` (edit.rs:8252) L643; `insert_paragraph_content` (edit/tracked.rs:829) L646; `build_table` (edit/structural.rs:153) L662; `insert_table_content` (edit/tracked.rs:848) L663; `Dom::add_after_self` (xmllinq/mod.rs:1147) (by name) L675; `Dom::add_before_self` (xmllinq/mod.rs:1130) (by name) L678; `track_properties` (edit/tracked.rs:888) L699,735; `format_paragraph` (edit.rs:8036) L700; `next_numbering_ids` (edit/structural.rs:350) L705; `abstract_num` (markdown/xml.rs:2553) L722; `.format` (ambiguous: `ListKind::format` (edit.rs:822); `RunSpec::format` (edit.rs:1073); `Writer::format` (markdown/write.rs:3254)) L724; `num` (markdown/xml.rs:2598) L726; `number_paragraph` (edit/structural.rs:435) L736; `Dom::new_element` (xmllinq/mod.rs:478) (by name) L759,768; `Dom::element` (xmllinq/mod.rs:655) (by name) L763; `clean_rpr` (edit/tracked.rs:1705) L765; `set_text` (edit.rs:7623) L769; `mark` (edit/tracked.rs:1682) L775,804; `delete_runs` (edit/tracked.rs:1536) L788; `Dom::next_element` (xmllinq/mod.rs:855) (by name) L791; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L793; `previous_paragraph` (edit/tracked.rs:1659) L797,799; `mark_has` (edit/tracked.rs:1628) L798
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.as_mut`, `.chain`, `.clone`, `.collect`, `.copied`, `.entry`, `.filter_map`, `.filter`, `.first`, `.get`, `.insert`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.keys`, `.last`, `.len`, `.or_default`, `.push`, `.remove`, `.rev`, `.sort_by_key`, `.then`, `.to_string`, `.unwrap_or_default`, `.unwrap_or`, `BTreeMap::new`, `Some`, `Vec::new`, `W::del`, `W::ins`, `W::p`, `W::r_pr`, `W::r`, `W::t`
- **Macros:** `matches!`, `vec!`
- **Called by:** `segment_region` (comparer/cross_para.rs:1503) (?); `tests::a_comment_on_a_vertical_page_turns_with_its_text` (convert/pdf.rs:2304) (?); `redline` (edit/tracked.rs:283); `Pieces::feed` (markdown/critic.rs:1908) (?)

#### `coalesce_insertions`  (L810-826)
`fn coalesce_insertions(edits: &[ScheduledEdit]) -> Vec<ScheduledEdit>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone`, `.last_mut`, `.push_str`, `.push`, `Vec::new`
- **Called by:** `emit` (edit/tracked.rs:452)

#### `insert_paragraph_content`  (L829-844)
`fn insert_paragraph_content(dom: &mut Dom, paragraph: NodeId, stamp: &mut Stamp<'_>)`

- **Does:** Wrap a new paragraph's content in one `w:ins` and mark it inserted.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L830; `Dom::name_is` (xmllinq/mod.rs:535) L833; `.container` (ambiguous: `Stamp::container` (edit/tracked.rs:50); `Stamp::container` (edit/whole.rs:202)) L836; `Dom::add_before_self` (xmllinq/mod.rs:1130) L837; `Dom::remove` (xmllinq/mod.rs:1118) L839; `Dom::add` (xmllinq/mod.rs:1074) L840; `mark` (edit/tracked.rs:1682) L843
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.first`, `.into_iter`, `W::ins`, `W::p_pr`
- **Called by:** `emit` (edit/tracked.rs:452); `insert_table_content` (edit/tracked.rs:848)

#### `insert_table_content`  (L848-867)
`fn insert_table_content(dom: &mut Dom, table: NodeId, stamp: &mut Stamp<'_>)`

- **Does:** Mark every row of a new table inserted (`w:trPr/w:ins`, after the row's other properties) and every cell paragraph's runs and mark inserted.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L849; `Dom::element` (xmllinq/mod.rs:655) L850,854; `Dom::new_element` (xmllinq/mod.rs:478) L853; `Dom::add_after_self` (xmllinq/mod.rs:1147) L855; `Dom::add_first` (xmllinq/mod.rs:1101) L856; `.container` (ambiguous: `Stamp::container` (edit/tracked.rs:50); `Stamp::container` (edit/whole.rs:202)) L861; `Dom::add` (xmllinq/mod.rs:1074) L862; `Dom::descendants` (xmllinq/mod.rs:668) L863; `insert_paragraph_content` (edit/tracked.rs:829) L864
- **Calls (std/external/unresolved):** `Some`, `W::ins`, `W::name`, `W::p`, `W::tr`
- **Called by:** `emit` (edit/tracked.rs:452)

#### `separate_tracked`  (L870-883)
`fn separate_tracked(dom: &mut Dom, table: NodeId, stamp: &mut Stamp<'_>)`

- **Does:** `structural::separate`, with each paragraph it adds marked inserted.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L871; `Dom::elements` (xmllinq/mod.rs:640) L873,877; `separate` (edit/structural.rs:242) L875; `mark` (edit/tracked.rs:1682) L879
- **Calls (std/external/unresolved):** `.contains`, `.map`, `.unwrap_or_default`, `Some`, `W::ins`, `W::p`
- **Called by:** `redline` (edit/tracked.rs:283)

#### `track_properties`  (L888-920)
`fn track_properties( dom: &mut Dom, paragraph: NodeId, stamp: &mut Stamp<'_>, change: impl FnOnce(&mut Dom), )`

- **Does:** Run `change` on `paragraph`'s properties and track it: a `w:pPrChange` holding the old `w:pPr`, unless the plan already tracked a change there (that one holds the original).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L894,896,899; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L900; `Dom::new_element` (xmllinq/mod.rs:478) L901; `Dom::elements` (xmllinq/mod.rs:640) L903; `Dom::name_is` (xmllinq/mod.rs:535) L904,905,906; `Dom::remove` (xmllinq/mod.rs:1118) L908; `change` (ambiguous: `tests::change` (changes/logical.rs:593); `change` (markdown/diff.rs:909); `change` (markdown/view_stories.rs:1227)) L913; `paragraph_properties` (edit/tracked.rs:1668) L915; `.container` (ambiguous: `Stamp::container` (edit/tracked.rs:50); `Stamp::container` (edit/whole.rs:202)) L916; `Dom::add` (xmllinq/mod.rs:1074) L917; `insert_ppr_child` (edit.rs:8135) L918
- **Calls (std/external/unresolved):** `.and_then`, `.is_some`, `.then`, `W::p_pr_change`, `W::p_pr`, `W::r_pr`, `W::sect_pr`
- **Called by:** `emit` (edit/tracked.rs:452); `tracked_source_property_boundary_tests::property_history_keeps_original_layout_and_leaves_live_marks_and_sections_on_the_source` (edit/tracked.rs:1911)

#### `track_run_format`  (L924-952)
`fn track_run_format( dom: &mut Dom, paragraph: NodeId, (start, end): (usize, usize), format: &super::RunFormat, stamp: &mut Stamp<'_>, )`

- **Does:** Format `range` of `paragraph` as `Transaction::apply` does, each changed run's old properties in a `w:rPrChange`, the last child of its `w:rPr`.
- **Calls (crate):** `runs_in_range` (edit.rs:8007) L931; `Dom::element` (xmllinq/mod.rs:655) L932; `Dom::new_element` (xmllinq/mod.rs:478) L935; `Dom::add_first` (xmllinq/mod.rs:1101) L936; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L940; `apply_run_format` (edit.rs:8327) L941; `Dom::serialize_element` (xmllinq/mod.rs:1315) L942; `Dom::elements` (xmllinq/mod.rs:640) L943; `Dom::remove` (xmllinq/mod.rs:1118) L944; `.container` (ambiguous: `Stamp::container` (edit/tracked.rs:50); `Stamp::container` (edit/whole.rs:202)) L948; `Dom::add` (xmllinq/mod.rs:1074) L949,950
- **Calls (std/external/unresolved):** `.is_empty`, `W::r_pr_change`, `W::r_pr`
- **Called by:** `emit` (edit/tracked.rs:452)

#### `thread_markers`  (L956-982)
`fn thread_markers(t: &mut Transaction<'_>, comment_ids: &BTreeMap<usize, u32>)`

- **Does:** Reply markers beside their comment's, and deleted comments' markers removed, as `Transaction::apply` does last.
- **Calls (crate):** `place_reply_markers` (edit.rs:7866) L965; `CommentFamily::with_replies` (comments.rs:597) (by name) L972; `remove_comment_markers` (edit.rs:7904) L977
- **Calls (std/external/unresolved):** `.as_ref`, `.collect`, `.get`, `.iter`, `.map`, `.unwrap_or_default`
- **Called by:** `redline` (edit/tracked.rs:283)

#### `insertion_replacement`  (L987-1029)
`fn insertion_replacement(dom: &Dom, paragraph: NodeId, edit: &ScheduledEdit) -> Vec<ScheduledEdit>`

- **Does:** Reuse the word diff for default replacements inside an insertion.
- **Calls (crate):** `project_paragraph` (inspect.rs:1265) L993; `editable_insertion` (edit.rs:7269) L1001,1005; `rewrite_ranges` (edit/rewrite.rs:77) L1010
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.clone`, `.collect`, `.filter`, `.first`, `.into_iter`, `.is_none`, `.is_some`, `.iter`, `.map`, `.replace_range`, `.rev`, `.to_string`
- **Macros:** `vec!`
- **Called by:** `emit` (edit/tracked.rs:452)

#### `emit_text`  (L1035-1278)
`fn emit_text( dom: &mut Dom, paragraph: NodeId, edit: &ScheduledEdit, stamp: &mut Stamp<'_>, empty: Option<&mut super::EmptyPlacement>, )`

- **Does:** Emit one text edit: the range's runs in `w:del`, then the replacement in a `w:ins` run formatted like the first deleted run, or like the run the insertion attaches to.
- **Calls (crate):** `.container` (ambiguous: `Stamp::container` (edit/tracked.rs:50); `Stamp::container` (edit/whole.rs:202)) L1049,1145,1258; `.place` (ambiguous, 4 candidates) L1050; `mark_run` (edit.rs:7632) L1051; `Dom::add` (xmllinq/mod.rs:1074) L1052,1149,1215,1252,1257,1259...; `write_mark_run_text` (edit.rs:7650) L1053; `format_range` (edit.rs:7988) L1055,1090,1131; `project_paragraph` (inspect.rs:1265) L1059,1120,1156,1161; `attach_segment` (edit.rs:7301) L1061,1157; `editable_insertion` (edit.rs:7269) L1075,1079,1163; `Dom::attribute` (xmllinq/mod.rs:874) L1076,1118; `apply_text_edit` (edit.rs:7403) L1081,1128; `remove_empty_insertion` (edit/tracked.rs:1417) L1098,1142; `isolate_range` (edit/tracked.rs:1432) L1105; `Dom::next_element` (xmllinq/mod.rs:855) L1109; `Dom::parent` (xmllinq/mod.rs:597) L1117,1199; `Dom::name_is` (xmllinq/mod.rs:535) L1117,1199; `run_of` (edit.rs:7980) L1124; `Dom::add_before_self` (xmllinq/mod.rs:1130) L1146,1261; `Dom::remove` (xmllinq/mod.rs:1118) L1148,1211,1214,1250,1275; `rename_deleted_text` (edit/tracked.rs:1521) L1150; `split_run_at` (edit.rs:7743) L1160; `isolate` (edit/tracked.rs:1480) L1189; `insertion_boundary_anchor` (edit/tracked.rs:1283) L1190; `Dom::elements` (xmllinq/mod.rs:640) L1200,1249; `insertion_split_boundary` (edit/tracked.rs:1346) L1202; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1209; `Dom::nodes` (xmllinq/mod.rs:604) L1210; `outside_ranges` (edit/tracked.rs:1328) L1229; `Dom::new_element` (xmllinq/mod.rs:478) L1236,1242,1255; `Dom::element` (xmllinq/mod.rs:655) L1238; `clean_rpr` (edit/tracked.rs:1705) L1239; `apply_run_format` (edit.rs:8327) L1247; `set_text` (edit.rs:7623) L1256; `Dom::add_after_self` (xmllinq/mod.rs:1147) L1262,1276; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1273
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.as_str`, `.cloned`, `.collect`, `.contains`, `.copied`, `.enumerate`, `.expect`, `.filter`, `.find`, `.first`, `.get`, `.insert`, `.into_iter`, `.is_empty`, `.is_none`, `.iter`, `.last_mut`, `.last`, `.len`, `.map_or`, `.map`, `.position`, `.push`, `.rev`, `.to_string`, `Some`, `Vec::new`, `W::author`, `W::del`, `W::id`, `W::ins`, `W::r_pr`, `W::r`, `W::t`, `usize::from`
- **Macros:** `vec!`
- **Called by:** `emit` (edit/tracked.rs:452)

#### `insertion_boundary_anchor`  (L1283-1293, pub(super))
`pub(super) fn insertion_boundary_anchor(dom: &Dom, run: NodeId, before: bool) -> NodeId`

- **Does:** The sibling an insertion at `run`'s edge goes beside: past the range markers that open before the run (`before`) or close after it, so those ranges keep their source endpoints.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L1284; `Dom::elements` (xmllinq/mod.rs:640) L1287; `insertion_split_boundary` (edit/tracked.rs:1346) L1291
- **Calls (std/external/unresolved):** `.iter`, `.position`, `usize::from`
- **Called by:** `emit_text` (edit/tracked.rs:1035); `apply_text_edit` (edit.rs:7403)

#### `is_one_of`  (L1318-1321)
`fn is_one_of(dom: &Dom, node: NodeId, locals: &[&str]) -> bool`

- **Does:** Whether `node` is a `w:` element named one of `locals`.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L1319; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1320; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1320
- **Calls (std/external/unresolved):** `.contains`, `.is_some_and`
- **Called by:** `insertion_split_boundary` (edit/tracked.rs:1346); `remove_own_insertion` (edit/tracked.rs:1584); `moved_in` (edit/tracked.rs:1652)

#### `outside_ranges`  (L1328-1342, pub(super))
`pub(super) fn outside_ranges(dom: &Dom, anchor: NodeId, before: bool) -> NodeId`

- **Does:** The sibling of `anchor` that new words go `before` (or after) so they join no range they meet: the first of the openers right before `anchor`, or the last of the closers right after it, a comment's end with its reference.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L1329; `Dom::elements` (xmllinq/mod.rs:640) L1332; `insertion_split_boundary` (edit/tracked.rs:1346) L1336
- **Calls (std/external/unresolved):** `.iter`, `.position`, `usize::from`
- **Called by:** `emit_text` (edit/tracked.rs:1035); `insert_outside_ranges` (edit.rs:7359)

#### `insertion_split_boundary`  (L1346-1380)
`fn insertion_split_boundary(dom: &Dom, children: &[NodeId], mut at: usize, before: bool) -> usize`

- **Does:** Opening markers travel with the words to their right, closing markers with the words to their left.
- **Calls (crate):** `is_one_of` (edit/tracked.rs:1318) L1348,1359; `Dom::name_is` (xmllinq/mod.rs:535) L1355,1360,1370,1371; `Dom::attribute` (xmllinq/mod.rs:874) L1356,1366; `Dom::elements` (xmllinq/mod.rs:640) L1363,1369
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.get`, `.insert`, `.is_empty`, `.is_none_or`, `.iter`, `BTreeSet::new`, `Some`, `W::id`, `W::name`, `W::r_pr`, `W::r`
- **Called by:** `emit_text` (edit/tracked.rs:1035); `insertion_boundary_anchor` (edit/tracked.rs:1283); `outside_ranges` (edit/tracked.rs:1328); `place_beside` (edit/tracked.rs:1386)

#### `place_beside`  (L1386-1414)
`fn place_beside(dom: &mut Dom, del: NodeId, ours: NodeId, stamp: &mut Stamp<'_>)`

- **Does:** Our insertion `ours` right after deletion `del`.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L1387; `Dom::name_is` (xmllinq/mod.rs:535) L1387; `Dom::add_after_self` (xmllinq/mod.rs:1147) L1388,1397,1412; `Dom::elements` (xmllinq/mod.rs:640) L1391; `insertion_split_boundary` (edit/tracked.rs:1346) L1396; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1399; `Dom::nodes` (xmllinq/mod.rs:604) L1400; `Dom::remove` (xmllinq/mod.rs:1118) L1401,1404; `Dom::add` (xmllinq/mod.rs:1074) L1405; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1410
- **Calls (std/external/unresolved):** `.cloned`, `.filter`, `.get`, `.insert`, `.iter`, `.len`, `.map_or`, `.position`, `.to_string`, `Some`, `W::id`, `W::ins`
- **Called by:** `emit` (edit/tracked.rs:452)

#### `remove_empty_insertion`  (L1417-1428, pub(super))
`pub(super) fn remove_empty_insertion(dom: &mut Dom, ins: NodeId)`

- **Does:** Empty run shells must not keep an otherwise deleted own insertion alive.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1418,1419,1425; `Dom::name_is` (xmllinq/mod.rs:535) L1420; `Dom::value` (xmllinq/mod.rs:1185) L1420; `Dom::remove` (xmllinq/mod.rs:1118) L1422,1426
- **Calls (std/external/unresolved):** `.all`, `.is_empty`, `.iter`, `Some`, `W::r_pr`, `W::r`, `W::t`
- **Called by:** `emit_text` (edit/tracked.rs:1035); `Transaction::apply` (edit.rs:5882)

#### `isolate_range`  (L1432-1468)
`fn isolate_range(dom: &mut Dom, paragraph: NodeId, start: usize, end: usize) -> Vec<NodeId>`

- **Does:** Split runs so each text or glyph of `[start, end)` sits alone in its run (beside the run's `w:rPr`); return the runs in document order.
- **Calls (crate):** `project_paragraph` (inspect.rs:1265) L1434,1445; `split_run_at` (edit.rs:7743) L1441; `alone` (edit/tracked.rs:1471) L1455; `isolate` (edit/tracked.rs:1480) L1456
- **Calls (std/external/unresolved):** `.cloned`, `.collect`, `.contains`, `.filter`, `.find`, `.iter`, `.map`, `.push`, `Vec::new`
- **Called by:** `emit_text` (edit/tracked.rs:1035)

#### `alone`  (L1471-1475)
`fn alone(dom: &Dom, run: NodeId, t: NodeId) -> bool`

- **Does:** Whether `t` is the only child of `run` besides its `w:rPr`.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1472; `Dom::name_is` (xmllinq/mod.rs:535) L1474
- **Calls (std/external/unresolved):** `.all`, `.into_iter`, `W::r_pr`
- **Called by:** `isolate_range` (edit/tracked.rs:1432); `tests::isolate_moves_neighbours_into_copies` (edit/tracked.rs:1731)

#### `isolate`  (L1480-1517)
`fn isolate(dom: &mut Dom, run: NodeId, t: NodeId)`

- **Does:** Move the children of `run` before `t` into a copy of the run placed before it, and those after `t` into a copy placed after it; each copy keeps the run's `w:rPr`.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1481,1498; `Dom::name_is` (xmllinq/mod.rs:535) L1485; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1497; `Dom::remove` (xmllinq/mod.rs:1118) L1501,1514; `Dom::add_before_self` (xmllinq/mod.rs:1130) L1505; `Dom::add_after_self` (xmllinq/mod.rs:1147) L1507
- **Calls (std/external/unresolved):** `.collect`, `.enumerate`, `.filter`, `.into_iter`, `.is_empty`, `.iter`, `.map`, `.position`, `W::r_pr`, `content`
- **Called by:** `emit_text` (edit/tracked.rs:1035); `isolate_range` (edit/tracked.rs:1432); `tests::isolate_moves_neighbours_into_copies` (edit/tracked.rs:1731)

#### `rename_deleted_text`  (L1521-1529)
`fn rename_deleted_text(dom: &mut Dom, run: NodeId)`

- **Does:** `w:t` to `w:delText` and `w:instrText` to `w:delInstrText` in a run that now sits inside a `w:del`.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1522; `Dom::name_is` (xmllinq/mod.rs:535) L1523,1525; `Dom::set_name` (xmllinq/mod.rs:539) L1524,1526
- **Calls (std/external/unresolved):** `W::del_text`, `W::name`, `W::t`
- **Called by:** `emit_text` (edit/tracked.rs:1035); `wrap_deleted` (edit/tracked.rs:1609)

#### `delete_runs`  (L1536-1564)
`fn delete_runs(dom: &mut Dom, container: NodeId, stamp: &mut Stamp<'_>)`

- **Does:** Every run of `container`, at any depth of hyperlinks, content controls, smart tags and simple fields, inside a `w:del`; consecutive runs share one.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1538; `Dom::name_is` (xmllinq/mod.rs:535) L1539; `wrap_deleted` (edit/tracked.rs:1609) L1543,1563; `Dom::name` (xmllinq/mod.rs:526) L1544; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1544; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1547; `delete_runs` (edit/tracked.rs:1536) L1549,1553,1559; `Dom::element` (xmllinq/mod.rs:655) L1552; `Dom::attribute` (xmllinq/mod.rs:874) L1556; `remove_own_insertion` (edit/tracked.rs:1584) L1557
- **Calls (std/external/unresolved):** `.filter`, `.push`, `Some`, `Vec::new`, `W::author`, `W::r`, `W::sdt_content`, `std::mem::take`
- **Called by:** `emit` (edit/tracked.rs:452); `delete_runs` (edit/tracked.rs:1536); `remove_own_insertion` (edit/tracked.rs:1584); `tests::delete_runs_groups_siblings_and_recurses_into_hyperlinks` (edit/tracked.rs:1780); `nested_run_deletion_source_boundary_tests::deleting_supported_nested_run_containers_preserves_complete_rejected_sources` (edit/tracked.rs:1845)

#### `remove_own_insertion`  (L1584-1607)
`fn remove_own_insertion(dom: &mut Dom, ins: NodeId, stamp: &mut Stamp<'_>)`

- **Does:** The plan author's own pending insertion, deleted: its typed runs go outright, a run holding a field character or a reference is deleted inside it so no field or note loses a part, and the markers it held (bookmarks, comment ranges) stay where they were.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1585,1586,1595; `is_one_of` (edit/tracked.rs:1318) L1589,1597,1598,1599; `Dom::remove` (xmllinq/mod.rs:1118) L1591,1602,1605; `delete_runs` (edit/tracked.rs:1536) L1594; `Dom::add_before_self` (xmllinq/mod.rs:1130) L1603
- **Calls (std/external/unresolved):** `.all`, `.into_iter`, `.iter`, `Some`, `W::r`
- **Called by:** `delete_runs` (edit/tracked.rs:1536)

#### `wrap_deleted`  (L1609-1620)
`fn wrap_deleted(dom: &mut Dom, group: Vec<NodeId>, stamp: &mut Stamp<'_>)`

- **Does:** _no doc comment_
- **Calls (crate):** `.container` (ambiguous: `Stamp::container` (edit/tracked.rs:50); `Stamp::container` (edit/whole.rs:202)) L1613; `Dom::add_before_self` (xmllinq/mod.rs:1130) L1614; `Dom::remove` (xmllinq/mod.rs:1118) L1616; `Dom::add` (xmllinq/mod.rs:1074) L1617; `rename_deleted_text` (edit/tracked.rs:1521) L1618
- **Calls (std/external/unresolved):** `.first`, `W::del`
- **Called by:** `delete_runs` (edit/tracked.rs:1536)

#### `mark_has`  (L1628-1630)
`fn mark_has(dom: &Dom, paragraph: NodeId, locals: &[&str]) -> bool`

- **Does:** Whether `paragraph`'s mark (`w:pPr/w:rPr`) carries one of `locals`.
- **Calls (crate):** `mark_revision` (edit/tracked.rs:1634) L1629
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520) (?); `emit` (edit/tracked.rs:452); `mark` (edit/tracked.rs:1682); `tests::a_deleted_mark_goes_after_an_inserted_one_and_is_not_doubled` (edit/tracked.rs:1753)

#### `mark_revision`  (L1634-1642)
`fn mark_revision(dom: &Dom, paragraph: NodeId, locals: &[&str]) -> Option<NodeId>`

- **Does:** The revision among `locals` that `paragraph`'s mark (`w:pPr/w:rPr`) carries.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1635,1637; `Dom::elements` (xmllinq/mod.rs:640) L1638; `Dom::name` (xmllinq/mod.rs:526) L1639; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1640; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1640
- **Calls (std/external/unresolved):** `.and_then`, `.contains`, `.find`, `.into_iter`, `.is_some_and`, `W::p_pr`, `W::r_pr`
- **Called by:** `check` (edit/tracked.rs:107); `mark_has` (edit/tracked.rs:1628)

#### `paragraph_format_change`  (L1645-1648)
`fn paragraph_format_change(dom: &Dom, paragraph: NodeId) -> Option<NodeId>`

- **Does:** `paragraph`'s tracked properties change (`w:pPr/w:pPrChange`).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1646,1647
- **Calls (std/external/unresolved):** `.and_then`, `W::p_pr_change`, `W::p_pr`
- **Called by:** `check` (edit/tracked.rs:107)

#### `moved_in`  (L1652-1656)
`fn moved_in(dom: &Dom, node: NodeId) -> Option<NodeId>`

- **Does:** The first move (`w:moveFrom`, `w:moveTo`) `node` holds, in its runs or its mark.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1653; `is_one_of` (edit/tracked.rs:1318) L1655
- **Calls (std/external/unresolved):** `.find`, `.into_iter`
- **Called by:** `check` (edit/tracked.rs:107); `tests::a_deleted_mark_goes_after_an_inserted_one_and_is_not_doubled` (edit/tracked.rs:1753)

#### `previous_paragraph`  (L1659-1665)
`fn previous_paragraph(dom: &Dom, node: NodeId) -> Option<NodeId>`

- **Does:** The paragraph right before `node` among its siblings.
- **Calls (crate):** `Dom::nodes_before_self` (xmllinq/mod.rs:846) L1660; `Dom::is_element` (xmllinq/mod.rs:502) L1663; `Dom::name_is` (xmllinq/mod.rs:535) L1664
- **Calls (std/external/unresolved):** `.filter`, `.find`, `.into_iter`, `.rev`, `W::p`
- **Called by:** `check` (edit/tracked.rs:107); `emit` (edit/tracked.rs:452)

#### `paragraph_properties`  (L1668-1677)
`fn paragraph_properties(dom: &mut Dom, paragraph: NodeId) -> NodeId`

- **Does:** `paragraph`'s `w:pPr`, created when missing.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1669; `Dom::new_element` (xmllinq/mod.rs:478) L1672; `Dom::add_first` (xmllinq/mod.rs:1101) L1673
- **Calls (std/external/unresolved):** `W::p_pr`
- **Called by:** `track_properties` (edit/tracked.rs:888); `mark` (edit/tracked.rs:1682); `tracked_source_property_boundary_tests::property_history_keeps_original_layout_and_leaves_live_marks_and_sections_on_the_source` (edit/tracked.rs:1911)

#### `mark`  (L1682-1701)
`fn mark(dom: &mut Dom, paragraph: NodeId, name: XName, stamp: &mut Stamp<'_>)`

- **Does:** Mark `paragraph`'s mark inserted or deleted (`w:pPr/w:rPr/w:ins|w:del`, first in `w:rPr` as the schema orders them, a deletion after an insertion).
- **Calls (crate):** `mark_has` (edit/tracked.rs:1628) L1684; `paragraph_properties` (edit/tracked.rs:1668) L1687; `Dom::element` (xmllinq/mod.rs:655) L1688,1697; `Dom::new_element` (xmllinq/mod.rs:478) L1691; `insert_ppr_child` (edit.rs:8135) L1692; `.container` (ambiguous: `Stamp::container` (edit/tracked.rs:50); `Stamp::container` (edit/whole.rs:202)) L1696; `Dom::add_after_self` (xmllinq/mod.rs:1147) L1698; `Dom::add_first` (xmllinq/mod.rs:1101) L1699
- **Calls (std/external/unresolved):** `W::del`, `W::ins`, `W::r_pr`
- **Called by:** `emit` (edit/tracked.rs:452); `insert_paragraph_content` (edit/tracked.rs:829); `separate_tracked` (edit/tracked.rs:870); `tests::a_deleted_mark_goes_after_an_inserted_one_and_is_not_doubled` (edit/tracked.rs:1753); `nested_run_deletion_source_boundary_tests::deleting_supported_nested_run_containers_preserves_complete_rejected_sources` (edit/tracked.rs:1845)

#### `clean_rpr`  (L1705-1718, pub(super))
`pub(super) fn clean_rpr(dom: &mut Dom, rpr: NodeId) -> NodeId`

- **Does:** A copy of a run's `w:rPr` for new text: without another revision's format change or mark revisions, whose ids must stay unique.
- **Calls (crate):** `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1706; `Dom::elements` (xmllinq/mod.rs:640) L1707; `Dom::name_is` (xmllinq/mod.rs:535) L1708,1709,1710,1711,1712; `Dom::remove` (xmllinq/mod.rs:1118) L1714
- **Calls (std/external/unresolved):** `W::del`, `W::ins`, `W::name`, `W::r_pr_change`
- **Called by:** `emit` (edit/tracked.rs:452); `emit_text` (edit/tracked.rs:1035); `tracked_source_property_boundary_tests::inserted_run_format_clones_drop_only_foreign_change_records` (edit/tracked.rs:1996); `copy_runs` (edit/visible.rs:267); `insert_outside_ranges` (edit.rs:7359); `apply_text_edit` (edit.rs:7403); `mark_run` (edit.rs:7632)

## Test functions (12)

- `tests::paragraph` (L1725): _no doc comment_
- `tests::isolate_moves_neighbours_into_copies` (L1731): _no doc comment_
- `tests::a_deleted_mark_goes_after_an_inserted_one_and_is_not_doubled` (L1753): _no doc comment_
- `tests::delete_runs_groups_siblings_and_recurses_into_hyperlinks` (L1780): _no doc comment_
- `nested_run_deletion_source_boundary_tests::semantic` (L1809): _no doc comment_
- `nested_run_deletion_source_boundary_tests::deleting_supported_nested_run_containers_preserves_complete_rejected_sources` (L1845): _no doc comment_
- `tracked_source_property_boundary_tests::paragraph` (L1905): _no doc comment_
- `tracked_source_property_boundary_tests::property_history_keeps_original_layout_and_leaves_live_marks_and_sections_on_the_source` (L1911): _no doc comment_
- `tracked_source_property_boundary_tests::inserted_run_format_clones_drop_only_foreign_change_records` (L1996): _no doc comment_
- `public_adjacent_run_replacement_owner_tests::tree` (L2031): _no doc comment_
- `public_adjacent_run_replacement_owner_tests::source_events` (L2052): _no doc comment_
- `public_adjacent_run_replacement_owner_tests::public_keep_replacement_across_adjacent_unicode_runs_restores_every_source_owner` (L2090): _no doc comment_
