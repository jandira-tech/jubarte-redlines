# Functions and methods in `src/edit/tracked.rs`

29 non-test, 12 test. Part of the [symbol manifest](../README.md).

#### `Stamp::container`  (L40-47)
`fn container(&mut self, dom: &mut Dom, name: XName) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L41; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L42,44,45
- **Calls (std/external/unresolved):** `.to_string`, `Some`, `W::author`, `W::date`, `W::id`
- **Called by:** `emit` (edit/tracked.rs:275) (?); `insert_paragraph_content` (edit/tracked.rs:574) (?); `insert_table_content` (edit/tracked.rs:593) (?); `track_properties` (edit/tracked.rs:633) (?); `track_run_format` (edit/tracked.rs:669) (?); `emit_text` (edit/tracked.rs:732) (?); `wrap_deleted` (edit/tracked.rs:949) (?); `mark` (edit/tracked.rs:1012) (?); `rewrite_one` (edit/whole.rs:235) (?)

#### `untracked_kind`  (L53-56)
`fn untracked_kind(tx: &Transaction<'_>, operation: usize) -> Option<&'static str>`

- **Does:** The operation kind of `resolved` when the emitter cannot write it as a tracked change: such a plan goes through the comparer, or is refused under `keep`, never left out of the redline.
- **Calls (crate):** `super::kind_name` (ambiguous: `kind_name` (changes.rs:246); `kind_name` (edit.rs:5294)) L54; `operation_supports_keep` (edit.rs:1489) L55
- **Calls (std/external/unresolved):** `.then_some`
- **Called by:** `emits_all` (edit/tracked.rs:59); `check` (edit/tracked.rs:84)

#### `emits_all`  (L59-63, pub(super))
`pub(super) fn emits_all(tx: &Transaction<'_>) -> bool`

- **Does:** Whether the emitter writes every operation of `tx` as tracked changes.
- **Calls (crate):** `untracked_kind` (edit/tracked.rs:53) L62
- **Calls (std/external/unresolved):** `.all`, `.is_none`, `.iter`
- **Called by:** `Transaction::emits_directly` (edit.rs:2125)

#### `formatting_tracked`  (L67-78)
`fn formatting_tracked(dom: &Dom, paragraph: NodeId, start: usize, end: usize) -> bool`

- **Does:** Whether a run holding `[start, end)` of `paragraph` has a tracked formatting change.
- **Calls (crate):** `project_paragraph` (inspect.rs:1024) L68; `Dom::element` (xmllinq/mod.rs:655) L74,75
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.filter`, `.is_some`, `.iter`, `W::r_pr_change`, `W::r_pr`
- **Called by:** `check` (edit/tracked.rs:84)

#### `check`  (L84-163, pub(super))
`pub(super) fn check(tx: &Transaction<'_>) -> Result<(), EditError>`

- **Does:** Refusals that only `keep` needs, checked on the untouched source after resolution: an operation the emitter cannot track, and a paragraph- or run-level change on top of another revision of the same paragraph mark or properties.
- **Calls (crate):** `Transaction::conflict` (edit.rs:4477) L90; `untracked_kind` (edit/tracked.rs:53) L96; `holds_revision` (edit/tracked.rs:981) L107; `Dom::next_element` (xmllinq/mod.rs:855) (by name) L111; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L113; `previous_paragraph` (edit/tracked.rs:989) L114; `mark_has` (edit/tracked.rs:968) L114,124; `Dom::element` (xmllinq/mod.rs:655) (by name) L131,133,142,143; `formatting_tracked` (edit/tracked.rs:67) L153
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.clone`, `.into`, `.is_none_or`, `.is_some_and`, `.is_some`, `.iter`, `Err`, `Ok`, `Some`, `W::p_pr_change`, `W::p_pr`, `W::p`, `refuse`
- **Macros:** `format!`
- **Called by:** `apply_plan` (edit.rs:1621); `resolved` (edit.rs:1732)

#### `result`  (L167-179, pub(super))
`pub(super) fn result(tx: &Transaction<'_>, clean: Vec<u8>) -> Result<EditResult, EditError>`

- **Does:** The result under `keep`: the clean copy as `finish` made it, the redline by emission, and revision counts of the plan's own changes.
- **Calls (crate):** `redline` (edit/tracked.rs:185) L168; `Transaction::report` (edit.rs:2346) L169; `paragraphs` (inspect.rs:284) L170; `revision_counts` (edit/tracked.rs:237) L173
- **Calls (std/external/unresolved):** `.len`, `.map`, `.unwrap_or`, `Ok`
- **Called by:** `apply_plan` (edit.rs:1621)

#### `redline`  (L185-232)
`fn redline(tx: &Transaction<'_>) -> Result<(Vec<u8>, u64), EditError>`

- **Does:** Replay `tx`'s resolved operations on a fresh copy of its base as tracked changes.
- **Calls (crate):** `Transaction::start` (edit.rs:2004) L194; `Transaction::deletion_comment` (edit.rs:4974) L204; `first_free_id` (edit/tracked.rs:250) L218; `emit` (edit/tracked.rs:275) L221; `thread_markers` (edit/tracked.rs:701) L222; `separate_tracked` (edit/tracked.rs:615) L226; `Transaction::finish` (edit.rs:5077) L230
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.enumerate`, `.filter_map`, `.iter`, `.map`, `.push`, `.sort_by_key`, `.to_string`, `Ok`, `Some`
- **Called by:** `result` (edit/tracked.rs:167)

#### `revision_counts`  (L237-246)
`fn revision_counts(redline: &[u8], author: &str, date: &str, floor: u64) -> RevisionCounts`

- **Does:** Counts of the redline's changes stamped with `author` and `date` whose `w:id` is at least `floor`: the edit's own.
- **Calls (crate):** `list_changes` (changes.rs:287) L238; `RevisionCounts::from_changes` (edit.rs:959) L239; `change_id` (ids.rs:187) L242
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.filter`, `.is_some_and`, `.iter`, `.ok`, `.parse`, `.unwrap_or_default`, `Some`
- **Called by:** `result` (edit/tracked.rs:167)

#### `first_free_id`  (L250-271)
`fn first_free_id(opened: &Opened, comments: &[(u32, String)]) -> u64`

- **Does:** One past the highest `w:id` in every revision-bearing part and among the plan's comments, so a new revision id collides with nothing.
- **Calls (crate):** `revision_bearing_parts` (revision_processor.rs:3403) L252; `PartFs::part_string` (opc/mod.rs:498) L253; `Dom::new` (xmllinq/mod.rs:387) L256; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L257; `Dom::root` (xmllinq/mod.rs:822) L258; `Dom::descendants` (xmllinq/mod.rs:668) L261; `Dom::attribute` (xmllinq/mod.rs:874) L262
- **Calls (std/external/unresolved):** `.and_then`, `.iter`, `.map_or`, `.map`, `.max`, `.ok`, `.parse`, `Some`, `W::id`, `u64::from`
- **Called by:** `redline` (edit/tracked.rs:185)

#### `emit`  (L275-571)
`fn emit( t: &mut Transaction<'_>, comment_ids: &BTreeMap<usize, u32>, stamp: &mut Stamp<'_>, ) -> Vec<NodeId>`

- **Does:** The steps of `Transaction::apply`, in its order, as tracked changes.
- **Calls (crate):** `track_run_format` (edit/tracked.rs:669) L293; `project_paragraph` (inspect.rs:1024) L346,446,548; `apply_text_edit` (edit.rs:5611) L347; `emit_text` (edit/tracked.rs:732) L356; `new_position` (edit.rs:5448) L362,367,368; `anchor_comment` (edit.rs:5809) L374,447,549; `anchor_span` (edit.rs:5865) L384; `toc_paragraphs` (edit.rs:6236) L405; `build_paragraph` (edit.rs:6280) L406; `insert_paragraph_content` (edit/tracked.rs:574) L409; `build_table` (edit/structural.rs:153) L425; `insert_table_content` (edit/tracked.rs:593) L426; `Dom::add_after_self` (xmllinq/mod.rs:1131) (by name) L438; `Dom::add_before_self` (xmllinq/mod.rs:1114) (by name) L441; `track_properties` (edit/tracked.rs:633) L462,498; `format_paragraph` (edit.rs:6064) L463; `next_numbering_ids` (edit/structural.rs:350) L468; `abstract_num` (markdown/xml.rs:2109) L485; `.format` (ambiguous: `ListKind::format` (edit.rs:642); `RunSpec::format` (edit.rs:891); `Writer::format` (markdown/write.rs:2950)) L487; `num` (markdown/xml.rs:2154) L489; `number_paragraph` (edit/structural.rs:435) L499; `Dom::new_element` (xmllinq/mod.rs:478) (by name) L522,531; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L523; `Dom::element` (xmllinq/mod.rs:655) (by name) L526; `clean_rpr` (edit/tracked.rs:1035) L528; `.add` (ambiguous, 5 candidates) L529,533,535,536; `set_text` (edit.rs:5721) L532; `.container` (ambiguous: `Stamp::container` (edit/tracked.rs:40); `Stamp::container` (edit/whole.rs:207)) L534; `mark` (edit/tracked.rs:1012) L538,567; `delete_runs` (edit/tracked.rs:923) L551; `Dom::next_element` (xmllinq/mod.rs:855) (by name) L554; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L556; `previous_paragraph` (edit/tracked.rs:989) L560,562; `mark_has` (edit/tracked.rs:968) L561
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.chain`, `.clone`, `.collect`, `.copied`, `.entry`, `.filter_map`, `.filter`, `.get`, `.insert`, `.is_empty`, `.is_some_and`, `.is_some`, `.iter`, `.keys`, `.last`, `.len`, `.or_default`, `.push`, `.remove`, `.rev`, `.sort_by_key`, `.to_string`, `.unwrap_or_default`, `.unwrap_or`, `BTreeMap::new`, `Some`, `Vec::new`, `W::del`, `W::ins`, `W::p`, `W::r_pr`, `W::r`, `W::t`
- **Macros:** `matches!`, `vec!`
- **Called by:** `segment_region` (comparer/cross_para.rs:1503) (?); `tests::a_comment_on_a_vertical_page_turns_with_its_text` (convert/pdf.rs:2304) (?); `redline` (edit/tracked.rs:185); `Pieces::feed` (markdown/critic.rs:1711) (?)

#### `insert_paragraph_content`  (L574-589)
`fn insert_paragraph_content(dom: &mut Dom, paragraph: NodeId, stamp: &mut Stamp<'_>)`

- **Does:** Wrap a new paragraph's content in one `w:ins` and mark it inserted.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L575; `Dom::name_is` (xmllinq/mod.rs:535) L578; `.container` (ambiguous: `Stamp::container` (edit/tracked.rs:40); `Stamp::container` (edit/whole.rs:207)) L581; `Dom::add_before_self` (xmllinq/mod.rs:1114) L582; `Dom::remove` (xmllinq/mod.rs:1102) L584; `Dom::add` (xmllinq/mod.rs:1058) L585; `mark` (edit/tracked.rs:1012) L588
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.first`, `.into_iter`, `W::ins`, `W::p_pr`
- **Called by:** `emit` (edit/tracked.rs:275); `insert_table_content` (edit/tracked.rs:593)

#### `insert_table_content`  (L593-612)
`fn insert_table_content(dom: &mut Dom, table: NodeId, stamp: &mut Stamp<'_>)`

- **Does:** Mark every row of a new table inserted (`w:trPr/w:ins`, after the row's other properties) and every cell paragraph's runs and mark inserted.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L594; `Dom::element` (xmllinq/mod.rs:655) L595,599; `Dom::new_element` (xmllinq/mod.rs:478) L598; `Dom::add_after_self` (xmllinq/mod.rs:1131) L600; `Dom::add_first` (xmllinq/mod.rs:1085) L601; `.container` (ambiguous: `Stamp::container` (edit/tracked.rs:40); `Stamp::container` (edit/whole.rs:207)) L606; `Dom::add` (xmllinq/mod.rs:1058) L607; `Dom::descendants` (xmllinq/mod.rs:668) L608; `insert_paragraph_content` (edit/tracked.rs:574) L609
- **Calls (std/external/unresolved):** `Some`, `W::ins`, `W::name`, `W::p`, `W::tr`
- **Called by:** `emit` (edit/tracked.rs:275)

#### `separate_tracked`  (L615-628)
`fn separate_tracked(dom: &mut Dom, table: NodeId, stamp: &mut Stamp<'_>)`

- **Does:** `structural::separate`, with each paragraph it adds marked inserted.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L616; `Dom::elements` (xmllinq/mod.rs:640) L618,622; `separate` (edit/structural.rs:242) L620; `mark` (edit/tracked.rs:1012) L624
- **Calls (std/external/unresolved):** `.contains`, `.map`, `.unwrap_or_default`, `Some`, `W::ins`, `W::p`
- **Called by:** `redline` (edit/tracked.rs:185)

#### `track_properties`  (L633-665)
`fn track_properties( dom: &mut Dom, paragraph: NodeId, stamp: &mut Stamp<'_>, change: impl FnOnce(&mut Dom), )`

- **Does:** Run `change` on `paragraph`'s properties and track it: a `w:pPrChange` holding the old `w:pPr`, unless the plan already tracked a change there (that one holds the original).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L639,641,644; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L645; `Dom::new_element` (xmllinq/mod.rs:478) L646; `Dom::elements` (xmllinq/mod.rs:640) L648; `Dom::name_is` (xmllinq/mod.rs:535) L649,650,651; `Dom::remove` (xmllinq/mod.rs:1102) L653; `change` (ambiguous: `change` (markdown/diff.rs:900); `change` (markdown/view_stories.rs:696)) L658; `paragraph_properties` (edit/tracked.rs:998) L660; `.container` (ambiguous: `Stamp::container` (edit/tracked.rs:40); `Stamp::container` (edit/whole.rs:207)) L661; `Dom::add` (xmllinq/mod.rs:1058) L662; `insert_ppr_child` (edit.rs:6163) L663
- **Calls (std/external/unresolved):** `.and_then`, `.is_some`, `.then`, `W::p_pr_change`, `W::p_pr`, `W::r_pr`, `W::sect_pr`
- **Called by:** `emit` (edit/tracked.rs:275); `tracked_source_property_boundary_tests::property_history_keeps_original_layout_and_leaves_live_marks_and_sections_on_the_source` (edit/tracked.rs:1237)

#### `track_run_format`  (L669-697)
`fn track_run_format( dom: &mut Dom, paragraph: NodeId, (start, end): (usize, usize), format: &super::RunFormat, stamp: &mut Stamp<'_>, )`

- **Does:** Format `range` of `paragraph` as `Transaction::apply` does, each changed run's old properties in a `w:rPrChange`, the last child of its `w:rPr`.
- **Calls (crate):** `runs_in_range` (edit.rs:6035) L676; `Dom::element` (xmllinq/mod.rs:655) L677; `Dom::new_element` (xmllinq/mod.rs:478) L680; `Dom::add_first` (xmllinq/mod.rs:1085) L681; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L685; `apply_run_format` (edit.rs:6355) L686; `Dom::serialize_element` (xmllinq/mod.rs:1299) L687; `Dom::elements` (xmllinq/mod.rs:640) L688; `Dom::remove` (xmllinq/mod.rs:1102) L689; `.container` (ambiguous: `Stamp::container` (edit/tracked.rs:40); `Stamp::container` (edit/whole.rs:207)) L693; `Dom::add` (xmllinq/mod.rs:1058) L694,695
- **Calls (std/external/unresolved):** `.is_empty`, `W::r_pr_change`, `W::r_pr`
- **Called by:** `emit` (edit/tracked.rs:275)

#### `thread_markers`  (L701-727)
`fn thread_markers(t: &mut Transaction<'_>, comment_ids: &BTreeMap<usize, u32>)`

- **Does:** Reply markers beside their comment's, and deleted comments' markers removed, as `Transaction::apply` does last.
- **Calls (crate):** `place_reply_markers` (edit.rs:5894) L710; `CommentFamily::with_replies` (comments.rs:563) (by name) L717; `remove_comment_markers` (edit.rs:5932) L722
- **Calls (std/external/unresolved):** `.as_ref`, `.collect`, `.get`, `.iter`, `.map`, `.unwrap_or_default`
- **Called by:** `redline` (edit/tracked.rs:185)

#### `emit_text`  (L732-822)
`fn emit_text(dom: &mut Dom, paragraph: NodeId, edit: &ScheduledEdit, stamp: &mut Stamp<'_>)`

- **Does:** Emit one text edit: the range's runs in `w:del`, then the replacement in a `w:ins` run formatted like the first deleted run, or like the run the insertion attaches to.
- **Calls (crate):** `isolate_range` (edit/tracked.rs:826) L736; `Dom::next_element` (xmllinq/mod.rs:855) L740; `.container` (ambiguous: `Stamp::container` (edit/tracked.rs:40); `Stamp::container` (edit/whole.rs:207)) L748,815; `Dom::add_before_self` (xmllinq/mod.rs:1114) L749,818; `Dom::remove` (xmllinq/mod.rs:1102) L751,807; `Dom::add` (xmllinq/mod.rs:1058) L752,809,814,816,820; `rename_deleted_text` (edit/tracked.rs:911) L753; `project_paragraph` (inspect.rs:1024) L759,764; `attach_segment` (edit.rs:5588) L760; `split_run_at` (edit.rs:5771) L763; `isolate` (edit/tracked.rs:874) L787; `Dom::new_element` (xmllinq/mod.rs:478) L793,799,812; `Dom::element` (xmllinq/mod.rs:655) L795; `clean_rpr` (edit/tracked.rs:1035) L796; `apply_run_format` (edit.rs:6355) L804; `Dom::elements` (xmllinq/mod.rs:640) L806; `set_text` (edit.rs:5721) L813; `Dom::add_after_self` (xmllinq/mod.rs:1131) L819
- **Calls (std/external/unresolved):** `.and_then`, `.cloned`, `.copied`, `.expect`, `.find`, `.first`, `.is_empty`, `.iter`, `.last_mut`, `.last`, `.map`, `.push`, `.rev`, `Some`, `Vec::new`, `W::del`, `W::ins`, `W::r_pr`, `W::r`, `W::t`
- **Macros:** `vec!`
- **Called by:** `emit` (edit/tracked.rs:275)

#### `isolate_range`  (L826-862)
`fn isolate_range(dom: &mut Dom, paragraph: NodeId, start: usize, end: usize) -> Vec<NodeId>`

- **Does:** Split runs so that each `w:t` of `[start, end)` sits alone in its run (beside the run's `w:rPr`); the runs, in document order.
- **Calls (crate):** `project_paragraph` (inspect.rs:1024) L828,839; `split_run_at` (edit.rs:5771) L835; `alone` (edit/tracked.rs:865) L849; `isolate` (edit/tracked.rs:874) L850
- **Calls (std/external/unresolved):** `.cloned`, `.collect`, `.contains`, `.filter_map`, `.filter`, `.find`, `.iter`, `.push`, `Some`, `Vec::new`
- **Called by:** `emit_text` (edit/tracked.rs:732)

#### `alone`  (L865-869)
`fn alone(dom: &Dom, run: NodeId, t: NodeId) -> bool`

- **Does:** Whether `t` is the only child of `run` besides its `w:rPr`.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L866; `Dom::name_is` (xmllinq/mod.rs:535) L868
- **Calls (std/external/unresolved):** `.all`, `.into_iter`, `W::r_pr`
- **Called by:** `isolate_range` (edit/tracked.rs:826); `tests::isolate_moves_neighbours_into_copies` (edit/tracked.rs:1059)

#### `isolate`  (L874-907)
`fn isolate(dom: &mut Dom, run: NodeId, t: NodeId)`

- **Does:** Move the children of `run` before `t` into a copy of the run placed before it, and those after `t` into a copy placed after it; each copy keeps the run's `w:rPr`.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L875,892; `Dom::name_is` (xmllinq/mod.rs:535) L879; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L891; `Dom::remove` (xmllinq/mod.rs:1102) L895,904; `Dom::add_before_self` (xmllinq/mod.rs:1114) L899; `Dom::add_after_self` (xmllinq/mod.rs:1131) L901
- **Calls (std/external/unresolved):** `.collect`, `.enumerate`, `.filter`, `.into_iter`, `.is_empty`, `.iter`, `.map`, `.position`, `W::r_pr`, `content`
- **Called by:** `emit_text` (edit/tracked.rs:732); `isolate_range` (edit/tracked.rs:826); `tests::isolate_moves_neighbours_into_copies` (edit/tracked.rs:1059)

#### `rename_deleted_text`  (L911-919)
`fn rename_deleted_text(dom: &mut Dom, run: NodeId)`

- **Does:** `w:t` to `w:delText` and `w:instrText` to `w:delInstrText` in a run that now sits inside a `w:del`.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L912; `Dom::name_is` (xmllinq/mod.rs:535) L913,915; `Dom::set_name` (xmllinq/mod.rs:539) L914,916
- **Calls (std/external/unresolved):** `W::del_text`, `W::name`, `W::t`
- **Called by:** `emit_text` (edit/tracked.rs:732); `wrap_deleted` (edit/tracked.rs:949)

#### `delete_runs`  (L923-947)
`fn delete_runs(dom: &mut Dom, container: NodeId, stamp: &mut Stamp<'_>)`

- **Does:** Every run of `container`, at any depth of hyperlinks, content controls, smart tags and simple fields, inside a `w:del`; consecutive runs share one.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L925; `Dom::name_is` (xmllinq/mod.rs:535) L926; `wrap_deleted` (edit/tracked.rs:949) L930,946; `Dom::name` (xmllinq/mod.rs:526) L931; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L931; `XName::local_name` (xmllinq/mod.rs:208) (by name) L934; `delete_runs` (edit/tracked.rs:923) L936,940; `Dom::element` (xmllinq/mod.rs:655) L939
- **Calls (std/external/unresolved):** `.filter`, `.push`, `Vec::new`, `W::r`, `W::sdt_content`, `std::mem::take`
- **Called by:** `emit` (edit/tracked.rs:275); `delete_runs` (edit/tracked.rs:923); `tests::delete_runs_groups_siblings_and_recurses_into_hyperlinks` (edit/tracked.rs:1107); `nested_run_deletion_source_boundary_tests::deleting_supported_nested_run_containers_preserves_complete_rejected_sources` (edit/tracked.rs:1172)

#### `wrap_deleted`  (L949-960)
`fn wrap_deleted(dom: &mut Dom, group: Vec<NodeId>, stamp: &mut Stamp<'_>)`

- **Does:** _no doc comment_
- **Calls (crate):** `.container` (ambiguous: `Stamp::container` (edit/tracked.rs:40); `Stamp::container` (edit/whole.rs:207)) L953; `Dom::add_before_self` (xmllinq/mod.rs:1114) L954; `Dom::remove` (xmllinq/mod.rs:1102) L956; `Dom::add` (xmllinq/mod.rs:1058) L957; `rename_deleted_text` (edit/tracked.rs:911) L958
- **Calls (std/external/unresolved):** `.first`, `W::del`
- **Called by:** `delete_runs` (edit/tracked.rs:923)

#### `mark_has`  (L968-978)
`fn mark_has(dom: &Dom, paragraph: NodeId, locals: &[&str]) -> bool`

- **Does:** Whether `paragraph`'s mark (`w:pPr/w:rPr`) carries one of `locals`.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L969,970; `Dom::elements` (xmllinq/mod.rs:640) L972; `Dom::name` (xmllinq/mod.rs:526) L973; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L974; `XName::local_name` (xmllinq/mod.rs:208) (by name) L974
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.contains`, `.into_iter`, `.is_some_and`, `W::p_pr`, `W::r_pr`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501) (?); `check` (edit/tracked.rs:84); `emit` (edit/tracked.rs:275); `mark` (edit/tracked.rs:1012); `tests::a_deleted_mark_goes_after_an_inserted_one_and_is_not_doubled` (edit/tracked.rs:1081)

#### `holds_revision`  (L981-986)
`fn holds_revision(dom: &Dom, paragraph: NodeId) -> bool`

- **Does:** Whether `paragraph` holds an inserted, deleted or moved run or mark.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L982; `Dom::name` (xmllinq/mod.rs:526) L983; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L984; `XName::local_name` (xmllinq/mod.rs:208) (by name) L984
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.into_iter`, `.is_some_and`
- **Called by:** `check` (edit/tracked.rs:84); `tests::a_deleted_mark_goes_after_an_inserted_one_and_is_not_doubled` (edit/tracked.rs:1081)

#### `previous_paragraph`  (L989-995)
`fn previous_paragraph(dom: &Dom, node: NodeId) -> Option<NodeId>`

- **Does:** The paragraph right before `node` among its siblings.
- **Calls (crate):** `Dom::nodes_before_self` (xmllinq/mod.rs:846) L990; `Dom::is_element` (xmllinq/mod.rs:502) L993; `Dom::name_is` (xmllinq/mod.rs:535) L994
- **Calls (std/external/unresolved):** `.filter`, `.find`, `.into_iter`, `.rev`, `W::p`
- **Called by:** `check` (edit/tracked.rs:84); `emit` (edit/tracked.rs:275)

#### `paragraph_properties`  (L998-1007)
`fn paragraph_properties(dom: &mut Dom, paragraph: NodeId) -> NodeId`

- **Does:** `paragraph`'s `w:pPr`, created when missing.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L999; `Dom::new_element` (xmllinq/mod.rs:478) L1002; `Dom::add_first` (xmllinq/mod.rs:1085) L1003
- **Calls (std/external/unresolved):** `W::p_pr`
- **Called by:** `track_properties` (edit/tracked.rs:633); `mark` (edit/tracked.rs:1012); `tracked_source_property_boundary_tests::property_history_keeps_original_layout_and_leaves_live_marks_and_sections_on_the_source` (edit/tracked.rs:1237)

#### `mark`  (L1012-1031)
`fn mark(dom: &mut Dom, paragraph: NodeId, name: XName, stamp: &mut Stamp<'_>)`

- **Does:** Mark `paragraph`'s mark inserted or deleted (`w:pPr/w:rPr/w:ins|w:del`, first in `w:rPr` as the schema orders them, a deletion after an insertion).
- **Calls (crate):** `mark_has` (edit/tracked.rs:968) L1014; `paragraph_properties` (edit/tracked.rs:998) L1017; `Dom::element` (xmllinq/mod.rs:655) L1018,1027; `Dom::new_element` (xmllinq/mod.rs:478) L1021; `insert_ppr_child` (edit.rs:6163) L1022; `.container` (ambiguous: `Stamp::container` (edit/tracked.rs:40); `Stamp::container` (edit/whole.rs:207)) L1026; `Dom::add_after_self` (xmllinq/mod.rs:1131) L1028; `Dom::add_first` (xmllinq/mod.rs:1085) L1029
- **Calls (std/external/unresolved):** `W::del`, `W::ins`, `W::r_pr`
- **Called by:** `emit` (edit/tracked.rs:275); `insert_paragraph_content` (edit/tracked.rs:574); `separate_tracked` (edit/tracked.rs:615); `tests::a_deleted_mark_goes_after_an_inserted_one_and_is_not_doubled` (edit/tracked.rs:1081); `nested_run_deletion_source_boundary_tests::deleting_supported_nested_run_containers_preserves_complete_rejected_sources` (edit/tracked.rs:1172)

#### `clean_rpr`  (L1035-1046)
`fn clean_rpr(dom: &mut Dom, rpr: NodeId) -> NodeId`

- **Does:** A copy of a run's `w:rPr` for new text: without another revision's format change or mark revisions, whose ids must stay unique.
- **Calls (crate):** `Dom::clone_subtree` (xmllinq/mod.rs:1215) L1036; `Dom::elements` (xmllinq/mod.rs:640) L1037; `Dom::name_is` (xmllinq/mod.rs:535) L1038,1039,1040; `Dom::remove` (xmllinq/mod.rs:1102) L1042
- **Calls (std/external/unresolved):** `W::del`, `W::ins`, `W::r_pr_change`
- **Called by:** `emit` (edit/tracked.rs:275); `emit_text` (edit/tracked.rs:732); `tracked_source_property_boundary_tests::inserted_run_format_clones_drop_only_foreign_change_records` (edit/tracked.rs:1321)

## Test functions (12)

- `tests::paragraph` (L1053): _no doc comment_
- `tests::isolate_moves_neighbours_into_copies` (L1059): _no doc comment_
- `tests::a_deleted_mark_goes_after_an_inserted_one_and_is_not_doubled` (L1081): _no doc comment_
- `tests::delete_runs_groups_siblings_and_recurses_into_hyperlinks` (L1107): _no doc comment_
- `nested_run_deletion_source_boundary_tests::semantic` (L1135): _no doc comment_
- `nested_run_deletion_source_boundary_tests::deleting_supported_nested_run_containers_preserves_complete_rejected_sources` (L1172): _no doc comment_
- `tracked_source_property_boundary_tests::paragraph` (L1231): _no doc comment_
- `tracked_source_property_boundary_tests::property_history_keeps_original_layout_and_leaves_live_marks_and_sections_on_the_source` (L1237): _no doc comment_
- `tracked_source_property_boundary_tests::inserted_run_format_clones_drop_only_foreign_change_records` (L1321): _no doc comment_
- `public_adjacent_run_replacement_owner_tests::tree` (L1356): _no doc comment_
- `public_adjacent_run_replacement_owner_tests::source_events` (L1384): _no doc comment_
- `public_adjacent_run_replacement_owner_tests::public_keep_replacement_across_adjacent_unicode_runs_restores_every_source_owner` (L1422): _no doc comment_
