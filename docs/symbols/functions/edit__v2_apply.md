# Functions and methods in `src/edit/v2_apply.rs`

55 non-test, 17 test. Part of the [symbol manifest](../README.md).

#### `apply_plan`  (L80-82, pub)
`pub fn apply_plan(source: &[u8], plan: &PlanV2) -> Result<V2EditResult, EditError>`

- **Does:** Apply a bound PlanV2 in memory, using one immutable working baseline.
- **Calls (crate):** `apply_quoting` (edit/v2_apply.rs:86) L81
- **Called by:** `tests::effectful_apply_never_reopens_its_primary_for_the_correspondence` (edit/v2_apply.rs:2745); `tests::final_views_read_off_the_dom_match_a_reparse_of_the_emitted_primary` (edit/v2_apply.rs:2801)

#### `apply_quoting`  (L86-93, pub(super))
`pub(super) fn apply_quoting( source: &[u8], plan: &PlanV2, syntax: super::PlanSyntax, ) -> Result<V2EditResult, EditError>`

- **Does:** [`apply_plan`], its refusals quoting their routes in `syntax`: the plan's JSON, or the options of the typed command that built it.
- **Calls (crate):** `apply_bound` (edit/v2_apply.rs:121) L92; `with_operation_targets` (edit/v2_apply.rs:97) L92
- **Calls (std/external/unresolved):** `.map_err`
- **Called by:** `prepare_v2` (edit/prepared.rs:1058); `apply_plan` (edit/v2_apply.rs:80)

#### `with_operation_targets`  (L97-119)
`fn with_operation_targets(mut error: EditError, plan: &PlanV2) -> EditError`

- **Does:** A refusal's comment route on the refused operation's own target: the hint quotes a placeholder, since only the plan holds the target.
- **Calls (crate):** `.target` (ambiguous: `EditOptions::target` (cli.rs:358); `Operation::target` (edit/v2.rs:585); `ThreadOp::target` (edit.rs:2567)) L107; `fill` (ambiguous: `fill` (edit/controls.rs:509); `fill` (markdown/from_docx/drawing.rs:929)) L112,115
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.contains`, `.get`, `.ok`, `.parse`, `.replace`, `.strip_prefix`, `Some`, `serde_json::to_string`
- **Macros:** `format!`
- **Called by:** `apply_quoting` (edit/v2_apply.rs:86)

#### `apply_bound`  (L121-357)
`fn apply_bound( source: &[u8], plan: &PlanV2, syntax: super::PlanSyntax, ) -> Result<V2EditResult, EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `PlanV2::validate` (edit/v2.rs:523) L126; `source_sha256` (inspect.rs:299) (by name) L127; `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L128,188,211,214,321; `SourceModel::new` (edit/v2_apply.rs:814) L134; `SourceModel::decisions` (edit/v2_apply.rs:1235) L139; `SourceModel::scope` (edit/v2_apply.rs:1328) L140; `SourceModel::check_context` (edit/v2_apply.rs:1356) L141; `SourceModel::operation` (edit/v2_apply.rs:1388) L147; `pointer` (edit/v2_apply.rs:650) L148; `legacy` (edit/v2_apply.rs:629) L153,223; `resolve_selected` (edit.rs:1485) L156; `changes` (edit/v2_apply.rs:644) L162; `comments` (edit/v2_apply.rs:647) L175; `has_history` (edit/v2_apply.rs:659) L187,213; `resolve_for_edit` (changes.rs:909) L204; `Transaction::start` (edit.rs:2680) L226; `Transaction::skip_comment_ids_of` (edit.rs:3213) L227; `ResolvedRevisions::is_empty` (edit.rs:271) L229,247,248; `Origin::build` (edit/origin.rs:260) L234; `AnchorChanges::records` (edit/comment_settlement.rs:78) (by name) L243; `resolve` (edit/v2_apply.rs:1656) L244; `check` (edit/tracked.rs:107) L245; `edited_revisions` (edit/v2_apply.rs:2344) L249; `Transaction::apply` (edit.rs:6430) L250; `check_dom_realized` (edit/prepared.rs:712) L251; `Transaction::finish` (edit.rs:6645) L252; `check_serialized_stories` (edit/prepared.rs:763) L253; `result_v2_with_paragraphs` (edit/tracked.rs:231) L261; `Transaction::report` (edit.rs:3161) L270; `Transaction::paragraph_walk` (edit.rs:5097) L272; `reset_reopened_packages` (inspect.rs:721) L290; `paragraph_final_state` (edit/v2_apply.rs:547) L303; `capture_correspondence_states` (edit/v2_apply.rs:37) L307; `expectation_correspondence` (edit/v2_apply.rs:371) L308; `identity_correspondence` (expect.rs:536) L320; `preserve_direct_history` (edit/v2_apply.rs:2412) L329
- **Calls (std/external/unresolved):** `.as_deref`, `.as_ref`, `.as_str`, `.clone`, `.collect`, `.contains`, `.enumerate`, `.extend`, `.filter`, `.is_empty`, `.iter`, `.map_err`, `.map`, `.retain`, `.to_string`, `.to_vec`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `BTreeMap::new`, `ChangeFilter::default`, `Err`, `Ok`, `ResolvedRevisions::default`, `Some`, `guard`
- **Macros:** `format!`
- **Called by:** `apply_quoting` (edit/v2_apply.rs:86)

#### `expectation_correspondence`  (L371-461)
`fn expectation_correspondence( model: &SourceModel, tx: &Transaction<'_>, primary_base: &[NodeId], primary_output: &[NodeId], primary_states: &[FinalView], view: v2::TextView, ) -> Result<crate::expect::SourceToOutputCorrespondence, EditError>`

- **Does:** Map every source paragraph onto the chosen primary's projection, from what preparation already knows: [`origin::Origin`] follows each source paragraph through the plan's revision settlement by its tag (merges and removals included), and the transaction's own paragraph nodes place the survivors in the emitted primary.
- **Calls (crate):** `SourceModel::ids` (edit/v2_apply.rs:827) L389; `Origin::fate` (edit/origin.rs:478) (by name) L399,405; `Ids::id` (edit/origin.rs:212) (by name) L404,432; `CorrespondenceEntry::split` (expect.rs:311) L407,421,438; `CorrespondenceEntry::one_to_one` (expect.rs:292) L412,435; `CorrespondenceEntry::merged` (expect.rs:301) L418; `refuse_removed_final_paragraphs` (edit/v2_apply.rs:486) L451; `SourceToOutputCorrespondence::from_entries` (expect.rs:343) L454; `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L455
- **Calls (std/external/unresolved):** `.and_then`, `.copied`, `.entry`, `.enumerate`, `.get`, `.insert`, `.iter`, `.len`, `.map_err`, `.or_default`, `.push`, `BTreeMap::new`, `Vec::with_capacity`, `output_of`, `std::collections::HashMap::with_capacity`
- **Macros:** `format!`, `vec!`
- **Called by:** `apply_bound` (edit/v2_apply.rs:121)

#### `refuse_removed_final_paragraphs`  (L486-543)
`fn refuse_removed_final_paragraphs( states: &[FinalView], entries: &mut [crate::expect::CorrespondenceEntry], ) -> Result<(), EditError>`

- **Does:** Rewrite one-to-one entries whose output paragraph is not a paragraph of the view the expectations judge in (#1084 review F2, proposal §10: "A missing or unresolved mapped paragraph is not a passing absence assertion").
- **Calls (std/external/unresolved):** `.as_slice`, `.clear`, `.find`, `.flatten`, `.is_some_and`, `.iter_mut`, `.iter`, `.len`, `.max`, `.push`, `.zip`, `Ok`, `Some`, `Vec::with_capacity`, `chain_end`, `usize::from`
- **Macros:** `vec!`
- **Called by:** `expectation_correspondence` (edit/v2_apply.rs:371)

#### `paragraph_final_state`  (L547-552, pub(super))
`pub(super) fn paragraph_final_state(dom: &Dom, paragraph: NodeId) -> FinalView`

- **Does:** One emitted primary paragraph's final-view disposition, read off the DOM that produced the primary's bytes.
- **Calls (crate):** `mark_deleted` (edit/v2_apply.rs:583) L549; `final_run` (edit/v2_apply.rs:599) L550
- **Called by:** `redline` (edit/tracked.rs:283); `apply_bound` (edit/v2_apply.rs:121); `reparsed_final_views` (edit/v2_apply.rs:563)

#### `mark_deleted`  (L583-594)
`fn mark_deleted(dom: &Dom, paragraph: NodeId) -> bool`

- **Does:** Whether the paragraph's mark (`w:pPr/w:rPr`) is tracked-deleted (`w:del` or `w:moveFrom`), the same elements `tracked::mark` writes.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L584,585; `Dom::elements` (xmllinq/mod.rs:640) L587; `Dom::name` (xmllinq/mod.rs:526) L588; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L589; `XName::local_name` (xmllinq/mod.rs:208) (by name) L590
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.is_some_and`, `.iter`, `W::p_pr`, `W::r_pr`
- **Macros:** `matches!`
- **Called by:** `paragraph_final_state` (edit/v2_apply.rs:547)

#### `final_run`  (L599-609)
`fn final_run(dom: &Dom, paragraph: NodeId) -> bool`

- **Does:** Whether any run of the paragraph survives the final view: a run outside every deletion container (`w:del`/`w:moveFrom`) that carries content besides its own properties.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L600; `inside_deletion` (edit/v2_apply.rs:613) L603; `Dom::elements` (xmllinq/mod.rs:640) L604; `Dom::name_is` (xmllinq/mod.rs:535) L607
- **Calls (std/external/unresolved):** `.any`, `.iter`, `Some`, `W::r_pr`, `W::r`
- **Called by:** `paragraph_final_state` (edit/v2_apply.rs:547)

#### `inside_deletion`  (L613-627)
`fn inside_deletion(dom: &Dom, paragraph: NodeId, node: NodeId) -> bool`

- **Does:** Whether `node` sits inside a deletion container between the paragraph and itself.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L615; `Dom::name` (xmllinq/mod.rs:526) L619; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L620; `XName::local_name` (xmllinq/mod.rs:208) (by name) L620
- **Calls (std/external/unresolved):** `.is_some_and`
- **Macros:** `matches!`
- **Called by:** `final_run` (edit/v2_apply.rs:599)

#### `legacy`  (L629-643)
`fn legacy(plan: &PlanV2) -> EditPlan`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone`, `.unwrap_or_default`, `.unwrap_or`, `Some`, `Vec::new`
- **Called by:** `apply_bound` (edit/v2_apply.rs:121); `tests::list_tables_are_read_within_their_fib_ranges` (legacy_doc.rs:2533) (?)

#### `changes`  (L644-646)
`fn changes(bytes: &[u8]) -> Result<Vec<Change>, EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `list_changes` (changes.rs:564) L645; `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L645
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`
- **Called by:** `apply_bound` (edit/v2_apply.rs:121); `has_history` (edit/v2_apply.rs:659); `SourceModel::new` (edit/v2_apply.rs:814); `compare_history` (edit/v2_apply.rs:2422)

#### `comments`  (L647-649)
`fn comments(bytes: &[u8]) -> Result<Vec<crate::comments::CommentRecord>, EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `list_comments` (comments.rs:132) L648; `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L648
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`
- **Called by:** `apply_bound` (edit/v2_apply.rs:121); `SourceModel::new` (edit/v2_apply.rs:814)

#### `pointer`  (L650-652)
`fn pointer(i: usize) -> String`

- **Does:** _no doc comment_
- **Macros:** `format!`
- **Called by:** `apply_bound` (edit/v2_apply.rs:121); `outcome` (edit/v2_apply.rs:1634); `resolve` (edit/v2_apply.rs:1656); `check_boundary_ownership` (edit/v2_apply.rs:1783); `tests::v2_comment_diagnosis_ignores_engine_message_wording` (edit/v2_apply.rs:3051)

#### `miss`  (L653-655)
`fn miss((code, message, _): origin::Miss) -> EditError`

- **Does:** _no doc comment_
- **Calls (crate):** `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L654
- **Called by:** `resolve` (edit/v2_apply.rs:1656)

#### `has_history`  (L659-691)
`fn has_history(bytes: &[u8]) -> Result<bool, EditError>`

- **Does:** The revision inventory is intentionally supplemented by the admitted XML: absence from list_changes is not proof of absence of an unknown revision.
- **Calls (crate):** `changes` (edit/v2_apply.rs:644) L660; `Opened::open` (inspect.rs:735) L663; `tracked_revisions_elements` (revision_processor.rs:79) L664; `PartFs::parts` (opc/mod.rs:733) (by name) L665; `PartFs::part_string` (opc/mod.rs:636) (by name) L669; `Dom::new` (xmllinq/mod.rs:387) L672; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L673; `Dom::root` (xmllinq/mod.rs:822) L674; `Dom::descendants` (xmllinq/mod.rs:668) L677; `Dom::name` (xmllinq/mod.rs:526) L678; `XName::local_name` (xmllinq/mod.rs:208) (by name) L679; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L680
- **Calls (std/external/unresolved):** `.chain`, `.contains`, `.ends_with`, `.is_empty`, `.map_err`, `Ok`, `std::iter::once`
- **Called by:** `apply_bound` (edit/v2_apply.rs:121)

#### `SourceIndex::new`  (L713-751)
`fn new(opened: &Opened, loaded: &Loaded, changes: &[Change]) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L720; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L721; `revision_bearing_parts` (revision_processor.rs:3716) L734; `Dom::root` (xmllinq/mod.rs:822) L739,745; `SourceIndex::add_revision_part` (edit/v2_apply.rs:753) L740,746; `PartFs::part_string` (opc/mod.rs:636) L742; `Dom::new` (xmllinq/mod.rs:387) L743; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L744
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.entry`, `.find`, `.iter`, `.or_default`, `.push`, `BTreeMap::::new`, `BTreeMap::new`, `Self::default`, `Some`, `W14::name`, `W::p`
- **Called by:** `SourceModel::new` (edit/v2_apply.rs:814)

#### `SourceIndex::add_revision_part`  (L753-774)
`fn add_revision_part(&mut self, dom: &Dom, root: NodeId, part: &str, main: &str, count: usize)`

- **Does:** _no doc comment_
- **Calls (crate):** `part_story` (ids.rs:392) L754; `tracked_revisions_elements` (revision_processor.rs:79) L758; `Dom::descendants` (xmllinq/mod.rs:668) L759; `Dom::name` (xmllinq/mod.rs:526) L760; `Dom::attribute` (xmllinq/mod.rs:874) L763; `revision_long_id` (edit/v2_apply.rs:780) L767; `XName::local_name` (xmllinq/mod.rs:208) (by name) L767
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.entry`, `.extend`, `.filter`, `.into`, `.or_default`, `.push`, `.to_string`, `W::id`, `std::iter::repeat_n`
- **Macros:** `vec!`
- **Called by:** `SourceIndex::new` (edit/v2_apply.rs:713)

#### `revision_long_id`  (L780-790, pub(super))
`pub(super) fn revision_long_id(part: &str, local: &str, story: &str, wid: &str) -> String`

- **Does:** The long id of the revision carrier `w:{local}` with `w:id` `wid` in `part`, as `story` prints it.
- **Calls (crate):** `revision_id` (edit/native_keys.rs:30) L781; `part_token` (edit/v2_apply.rs:1606) L782,786; `encode` (edit/v2_apply.rs:1595) L782; `ChangeId::new` (ids.rs:142) L787; `ChangeId::short` (ids.rs:155) (by name) L787
- **Calls (std/external/unresolved):** `.is_some`
- **Macros:** `format!`
- **Called by:** `SourceIndex::add_revision_part` (edit/v2_apply.rs:753); `Transaction::revision_hint` (edit.rs:5259)

#### `SourceModel::new`  (L814-826, pub(super))
`pub(super) fn new(bytes: &[u8]) -> Result<Self, EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `Opened::open` (inspect.rs:735) L815; `load_stories` (edit.rs:1680) (by name) L816; `changes` (edit/v2_apply.rs:644) L817; `SourceIndex::new` (edit/v2_apply.rs:713) L818; `comments` (edit/v2_apply.rs:647) L823
- **Calls (std/external/unresolved):** `.map_err`, `Ok`
- **Called by:** `id_selection` (edit/for_edit.rs:658); `apply_bound` (edit/v2_apply.rs:121); `tests::a_range_first_side_that_is_not_a_block_refuses_usage_not_a_panic` (edit/v2_apply.rs:2988)

#### `SourceModel::ids`  (L827-835)
`fn ids(&self) -> origin::Ids<'_>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.iter`, `.map`
- **Called by:** `expectation_correspondence` (edit/v2_apply.rs:371); `SourceModel::entity` (edit/v2_apply.rs:836); `SourceModel::selector_pair` (edit/v2_apply.rs:1169); `SourceModel::operation` (edit/v2_apply.rs:1388)

#### `SourceModel::entity`  (L836-937)
`fn entity(&self, target: &Target) -> Result<Entity, EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L838,875; `change_id` (ids.rs:187) L851; `CommentFamily::load` (comments.rs:457) L874; `SourceModel::container_node` (edit/v2_apply.rs:938) L896; `SourceModel::check_pair` (edit/v2_apply.rs:1215) L898,927; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L899; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L908; `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L914; `SourceModel::ids` (edit/v2_apply.rs:827) L922,931; `Ids::id` (edit/origin.rs:212) (by name) L931
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.collect`, `.contains`, `.enumerate`, `.filter`, `.find_map`, `.first`, `.get`, `.is_empty`, `.iter`, `.len`, `.map_err`, `.map`, `.ok_or_else`, `.parse`, `.position`, `.then_some`, `Entity::Comment`, `Entity::Container`, `Entity::Paragraph`, `Entity::Revision`, `Err`, `Ok`, `Selector::Name`, `Some`, `W::p`, `W::tc`, `bad`
- **Macros:** `format!`
- **Called by:** `SourceModel::select_parsed` (edit/v2_apply.rs:1116); `SourceModel::decisions` (edit/v2_apply.rs:1235); `SourceModel::scope` (edit/v2_apply.rs:1328); `SourceModel::check_context` (edit/v2_apply.rs:1356); `SourceModel::operation` (edit/v2_apply.rs:1388)

#### `SourceModel::container_node`  (L938-977)
`fn container_node( &self, si: usize, path: &[Step], short_id: &str, ) -> Option<crate::xmllinq::NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L959; `collect_controls` (inspect.rs:1005) L968; `body_control_nodes` (inspect.rs:997) L969
- **Calls (std/external/unresolved):** `.and_then`, `.copied`, `.get`, `.is_empty`, `.is_some_and`, `.iter`, `.last`, `.position`, `Some`, `Step::Cell`, `Step::ContentControl`, `Step::Row`, `W::name`, `W::tc`, `crate::inspect::top_level_table_nodes`
- **Macros:** `matches!`
- **Called by:** `SourceModel::entity` (edit/v2_apply.rs:836); `SourceModel::selector_pair` (edit/v2_apply.rs:1169)

#### `SourceModel::pair`  (L979-1013)
`fn pair( &self, short_id: &str, story: usize, node: crate::xmllinq::NodeId, canonical: Option<&str>, ) -> Result<Target, EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L989; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L990; `encode` (edit/v2_apply.rs:1595) L998,1000; `part_token` (edit/v2_apply.rs:1606) L1004; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1005
- **Calls (std/external/unresolved):** `.and_then`, `.get`, `.is_some_and`, `.name`, `.ok_or_else`, `.to_string`, `.unwrap_or_default`, `.unwrap_or`, `Ok`, `Some`, `W14::name`
- **Macros:** `format!`
- **Called by:** `make_row` (edit/for_edit.rs:1293) (?); `SourceModel::selector_pair` (edit/v2_apply.rs:1169); `SourceModel::check_pair` (edit/v2_apply.rs:1215)

#### `SourceModel::selected_paragraphs`  (L1020-1042, pub(super))
`pub(super) fn selected_paragraphs( &self, spec: &str, open_ranges: bool, ) -> Result<std::collections::HashSet<usize>, EditError>`

- **Does:** Resolve a printed source selector through the same compiled entity and descendant resolver used by apply's scope enforcement.
- **Calls (crate):** `SourceModel::close_open_range` (edit/v2_apply.rs:1048) L1028; `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L1033; `SourceModel::select_parsed` (edit/v2_apply.rs:1116) L1039
- **Calls (std/external/unresolved):** `.map_err`, `.map`, `.parse`, `.split`, `.to_string`, `Ok`, `std::collections::HashSet::new`
- **Called by:** `id_selection` (edit/for_edit.rs:658) (?)

#### `SourceModel::close_open_range`  (L1048-1073)
`fn close_open_range(&self, item: &str) -> Result<String, EditError>`

- **Does:** The closed spelling of an open range: `p17-` closes against the last paragraph of its story and `-p3` against its first.
- **Calls (crate):** `SourceModel::past_end_usage` (edit/v2_apply.rs:1093) L1051,1063; `paragraph_place` (ids.rs:417) L1054,1066; `SourceModel::story_paragraph_count` (edit/v2_apply.rs:1075) L1055; `close` (ambiguous, 9 candidates) L1057,1068; `paragraph_id` (ids.rs:410) L1067
- **Calls (std/external/unresolved):** `.checked_sub`, `.filter`, `.is_empty`, `.strip_prefix`, `.strip_suffix`, `.to_string`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `SourceModel::selected_paragraphs` (edit/v2_apply.rs:1020)

#### `SourceModel::story_paragraph_count`  (L1075-1087)
`fn story_paragraph_count(&self, story: &str) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.iter`, `.map_or`, `.position`
- **Called by:** `SourceModel::close_open_range` (edit/v2_apply.rs:1048)

#### `SourceModel::past_end_usage`  (L1093-1113)
`fn past_end_usage(&self, id: &str) -> Option<EditError>`

- **Does:** `USAGE` when `id` names a paragraph of a story this document has at an index past that story's end (`p99 is past the last paragraph pN`); `None` when `id` is not that shape.
- **Calls (crate):** `paragraph_place` (ids.rs:417) L1094; `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L1104; `paragraph_id` (ids.rs:410) L1109
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.iter`, `.position`, `.saturating_sub`, `.then`
- **Macros:** `format!`
- **Called by:** `SourceModel::close_open_range` (edit/v2_apply.rs:1048); `SourceModel::selector_miss` (edit/v2_apply.rs:1210)

#### `SourceModel::select_parsed`  (L1116-1167)
`fn select_parsed( &self, parsed: ShortId, out: &mut std::collections::HashSet<usize>, ) -> Result<(), EditError>`

- **Does:** The paragraphs one parsed selector names.
- **Calls (crate):** `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L1127,1130,1140,1143,1158; `SourceModel::selector_pair` (edit/v2_apply.rs:1169) L1135,1136,1151; `SourceModel::entity` (edit/v2_apply.rs:836) L1138,1152
- **Calls (std/external/unresolved):** `.clone`, `.extend`, `.insert`, `.last_mut`, `.to_string`, `Err`, `Ok`, `Step::Paragraph`
- **Called by:** `SourceModel::selected_paragraphs` (edit/v2_apply.rs:1020); `tests::a_range_first_side_that_is_not_a_block_refuses_usage_not_a_panic` (edit/v2_apply.rs:2988) (?)

#### `SourceModel::selector_pair`  (L1169-1205)
`fn selector_pair(&self, id: &str) -> Result<Target, EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L1171; `SourceModel::container_node` (edit/v2_apply.rs:938) L1190; `SourceModel::pair` (edit/v2_apply.rs:979) L1191,1199; `SourceModel::ids` (edit/v2_apply.rs:827) L1193,1198; `SourceModel::selector_miss` (edit/v2_apply.rs:1210) L1196; `Ids::id` (edit/origin.rs:212) (by name) L1198
- **Calls (std/external/unresolved):** `.as_str`, `.iter`, `.map_err`, `.ok_or_else`, `.parse`, `.position`, `.to_string`, `Err`, `Selector::Name`, `Some`, `bad`
- **Macros:** `format!`
- **Called by:** `SourceModel::select_parsed` (edit/v2_apply.rs:1116)

#### `SourceModel::selector_miss`  (L1210-1213)
`fn selector_miss(&self, id: &str, (code, message, _): origin::Miss) -> EditError`

- **Does:** A missing selector keeps its anchor code unless it names a paragraph past its story's end, which is a malformed selector (`USAGE`), not a missing document anchor.
- **Calls (crate):** `SourceModel::past_end_usage` (edit/v2_apply.rs:1093) L1211; `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L1212
- **Calls (std/external/unresolved):** `.unwrap_or_else`
- **Called by:** `SourceModel::selector_pair` (edit/v2_apply.rs:1169)

#### `SourceModel::check_pair`  (L1215-1234)
`fn check_pair( &self, target: &Target, story: usize, node: NodeId, canonical: Option<&str>, ) -> Result<(), EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `SourceModel::pair` (edit/v2_apply.rs:979) L1222; `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L1224
- **Calls (std/external/unresolved):** `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `SourceModel::entity` (edit/v2_apply.rs:836)

#### `SourceModel::decisions`  (L1235-1326)
`fn decisions(&self, plan: &PlanV2) -> Result<Decisions, EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `SourceModel::entity` (edit/v2_apply.rs:836) L1239; `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L1240,1251,1308; `select` (audit.rs:239) (by name) L1263,1264; `ChangeFilter::ids` (changes.rs:330) L1316
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.cloned`, `.collect`, `.contains`, `.extend`, `.filter`, `.find`, `.insert`, `.intersection`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.map`, `.next`, `.ok_or_else`, `.then`, `.union`, `BTreeSet::new`, `Err`, `Ok`, `Some`, `effects`, `filter`
- **Macros:** `format!`, `matches!`
- **Called by:** `apply_bound` (edit/v2_apply.rs:121)

#### `SourceModel::scope`  (L1328-1355)
`fn scope(&self, scope: &Scope) -> Result<Authority, EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `SourceModel::entity` (edit/v2_apply.rs:836) L1336; `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L1345
- **Calls (std/external/unresolved):** `.extend`, `.insert`, `BTreeSet::new`, `Err`, `Ok`
- **Macros:** `matches!`
- **Called by:** `apply_bound` (edit/v2_apply.rs:121)

#### `SourceModel::check_context`  (L1356-1387)
`fn check_context(&self, plan: &PlanV2) -> Result<(), EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `SourceModel::entity` (edit/v2_apply.rs:836) L1358; `logical_text` (edit/v2.rs:802) L1360; `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L1371,1379
- **Calls (std/external/unresolved):** `.clone`, `.find`, `.flatten`, `.is_none`, `.iter`, `.map`, `Err`, `Ok`, `Some`
- **Called by:** `apply_bound` (edit/v2_apply.rs:121)

#### `SourceModel::operation`  (L1388-1592)
`fn operation(&self, operation: &v2::Operation, scope: &Authority) -> Result<Mapped, EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `SourceModel::entity` (edit/v2_apply.rs:836) L1403,1516; `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L1409,1419,1434,1437,1517,1520...; `Ids::id` (edit/origin.rs:212) (by name) L1428,1533; `SourceModel::ids` (edit/v2_apply.rs:827) L1428,1533; `p` (ambiguous: `deep_lcs_source_conservation_tests::p` (comparer/lcs.rs:15395); `tests::p` (markdown/from_docx/mod.rs:5028)) L1447,1464,1478,1484,1494,1505...; `occurrence` (edit/v2.rs:492) (by name) L1450,1469,1480,1541; `attachment` (edit/v2.rs:495) (by name) L1451,1470,1502; `CommentFamily::load` (comments.rs:457) L1562; `CommentFamily::with_replies` (comments.rs:597) (by name) L1564
- **Calls (std/external/unresolved):** `.any`, `.as_ref`, `.clone`, `.collect`, `.contains`, `.first`, `.is_some_and`, `.iter`, `.len`, `.map_err`, `.map`, `.ok_or_else`, `.push`, `.unwrap_or`, `Err`, `Ok`, `Selector::Name`, `Some`, `Vec::new`, `c`, `usize::try_from`
- **Macros:** `vec!`
- **Called by:** `apply_bound` (edit/v2_apply.rs:121)

#### `encode`  (L1595-1605)
`fn encode(text: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.bytes`, `.collect`, `.is_ascii_alphanumeric`, `.map`, `.to_string`
- **Macros:** `format!`, `matches!`
- **Called by:** `revision_long_id` (edit/v2_apply.rs:780); `SourceModel::pair` (edit/v2_apply.rs:979)

#### `part_token`  (L1606-1632)
`fn part_token(part: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.bytes`, `.collect`, `.is_ascii_alphanumeric`, `.is_ok_and`, `.is_some_and`, `.iter`, `.map`, `.parse`, `.strip_prefix`, `.strip_suffix`, `.to_string`, `.trim_start_matches`
- **Macros:** `format!`, `matches!`
- **Called by:** `revision_long_id` (edit/v2_apply.rs:780); `SourceModel::pair` (edit/v2_apply.rs:979)

#### `outcome`  (L1634-1649)
`fn outcome(i: usize, kind: &OperationKind) -> EditOutcome`

- **Does:** _no doc comment_
- **Calls (crate):** `pointer` (edit/v2_apply.rs:650) L1636; `super::kind_name` (ambiguous: `kind_name` (changes.rs:510); `kind_name` (edit.rs:6990)) L1637
- **Calls (std/external/unresolved):** `.into`
- **Called by:** `CandidateChecks::verification` (edit/prepared.rs:94) (?); `resolve` (edit/v2_apply.rs:1656)

#### `mapped_paragraph`  (L1650-1655)
`fn mapped_paragraph(tx: &Transaction<'_>, source: usize) -> Result<usize, EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `Origin::exact` (edit/origin.rs:400) (by name) L1652
- **Calls (std/external/unresolved):** `.map_err`, `Ok`
- **Called by:** `resolve` (edit/v2_apply.rs:1656); `resolve_operation` (edit/v2_apply.rs:2084)

#### `resolve`  (L1656-1760)
`fn resolve(tx: &mut Transaction<'_>, plan: &PlanV2, mapped: &[Mapped]) -> Result<(), EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `mapped_paragraph` (edit/v2_apply.rs:1650) L1663,1681,1730; `Origin::follow` (edit/origin.rs:418) (by name) L1668; `Transaction::paragraph_id` (edit.rs:5085) L1668,1688; `miss` (edit/v2_apply.rs:653) L1670; `pointer` (edit/v2_apply.rs:650) L1676,1723,1734,1749; `logical_text` (edit/v2.rs:802) L1682; `source_sha256` (inspect.rs:299) (by name) L1683; `outcome` (edit/v2_apply.rs:1634) L1685,1719; `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L1709,1732,1747; `resolve_operation` (edit/v2_apply.rs:2084) L1721; `Transaction::attach_revision_refusal` (edit/refusal.rs:195) L1722; `Transaction::note_rewrites` (edit.rs:3243) L1739; `check_boundary_ownership` (edit/v2_apply.rs:1783) L1754; `compose_threads` (edit/v2_apply.rs:1821) L1755; `Transaction::check_conflicts_v2` (edit.rs:5406) L1756; `diagnose_conflict` (edit/v2_apply.rs:1762) L1757; `check_comment_anchors` (edit/v2_apply.rs:1913) L1759
- **Calls (std/external/unresolved):** `.any`, `.as_bytes`, `.chars`, `.clone`, `.collect`, `.enumerate`, `.extend`, `.first`, `.get`, `.insert`, `.into_iter`, `.into`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.map_err`, `.map`, `.push_str`, `.push`, `.take`, `.unwrap_or_default`, `.zip`, `BTreeMap::new`, `Err`, `Some`, `Vec::new`
- **Macros:** `format!`, `matches!`
- **Called by:** `apply_bound` (edit/v2_apply.rs:121)

#### `diagnose_conflict`  (L1762-1778)
`fn diagnose_conflict(tx: &Transaction<'_>, mut error: EditError, tracked: bool) -> EditError`

- **Does:** _no doc comment_
- **Calls (crate):** `check_comment_anchors` (edit/v2_apply.rs:1913) L1765
- **Calls (std/external/unresolved):** `.as_deref`, `.into`, `Some`
- **Called by:** `resolve` (edit/v2_apply.rs:1656); `tests::v2_comment_diagnosis_ignores_engine_message_wording` (edit/v2_apply.rs:3051)

#### `check_boundary_ownership`  (L1783-1816)
`fn check_boundary_ownership(tx: &Transaction<'_>) -> Result<(), EditError>`

- **Does:** Same-position inserts share array order only when they agree on the source side that owns them.
- **Calls (crate):** `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L1807; `pointer` (edit/v2_apply.rs:650) L1809
- **Calls (std/external/unresolved):** `.array_windows`, `.entry`, `.or_default`, `.push`, `.sort_by_key`, `.values_mut`, `BTreeMap::<usize, Vec>::new`, `Err`, `Ok`, `Some`
- **Called by:** `resolve` (edit/v2_apply.rs:1656)

#### `compose_threads`  (L1821-1896)
`fn compose_threads(tx: &mut Transaction<'_>) -> Result<(), EditError>`

- **Does:** All aliases of a thread share one deletion/conflict domain and one final state.
- **Calls (crate):** `CommentFamily::thread_root` (comments.rs:585) (by name) L1833,1841,1882; `.target` (ambiguous: `EditOptions::target` (cli.rs:358); `Operation::target` (edit/v2.rs:585); `ThreadOp::target` (edit.rs:2567)) L1841; `comment_error` (edit/v2_apply.rs:1898) L1843,1852; `CommentFamily::thread_state` (comments.rs:716) (by name) L1872
- **Calls (std/external/unresolved):** `.any`, `.as_ref`, `.collect`, `.contains`, `.entry`, `.expect`, `.filter_map`, `.filter`, `.insert`, `.into_iter`, `.iter`, `.map`, `.or_insert`, `.push`, `BTreeMap::new`, `BTreeSet::new`, `Err`, `Ok`, `Some`, `Vec::new`
- **Called by:** `resolve` (edit/v2_apply.rs:1656)

#### `comment_error`  (L1898-1903)
`fn comment_error(tx: &Transaction<'_>, i: usize, code: &str, message: &str) -> EditError`

- **Does:** _no doc comment_
- **Calls (crate):** `Transaction::conflict` (edit.rs:5870) L1899
- **Calls (std/external/unresolved):** `.into`, `Some`
- **Called by:** `compose_threads` (edit/v2_apply.rs:1821); `check_comment_anchor` (edit/v2_apply.rs:1981)

#### `check_comment_anchors`  (L1913-1979)
`fn check_comment_anchors(tx: &Transaction<'_>, tracked: bool) -> Result<(), EditError>`

- **Does:** A supported range keeps its endpoints through composition.
- **Calls (crate):** `CommentFamily::with_replies` (comments.rs:597) (by name) L1933; `CommentFamily::order` (comments.rs:568) (by name) L1938; `Transaction::comment_anchors` (edit.rs:2997) L1942; `check_comment_anchor` (edit/v2_apply.rs:1981) L1948,1969
- **Calls (std/external/unresolved):** `.as_ref`, `.collect`, `.contains`, `.entry`, `.expect`, `.filter_map`, `.flatten`, `.get`, `.is_empty`, `.iter`, `.len`, `.or_default`, `.push`, `.to_string`, `AnchorEdits::new`, `BTreeMap::new`, `Ok`, `Some`
- **Macros:** `matches!`
- **Called by:** `resolve` (edit/v2_apply.rs:1656); `diagnose_conflict` (edit/v2_apply.rs:1762); `tests::thread_only_and_unchanged_plans_do_not_scan_comment_anchors` (edit/v2_apply.rs:3005); `tests::existing_comment_anchors_scan_each_paragraph_once_per_apply` (edit/v2_apply.rs:3033)

#### `check_comment_anchor`  (L1981-2082)
`fn check_comment_anchor( tx: &Transaction<'_>, edits: &AnchorEdits<'_>, covered: super::Covered, owner: Option<usize>, full_paragraphs: bool, mut preserve_deleted: bool, ) -> Result<(), EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `comment_error` (edit/v2_apply.rs:1898) L2066,2074
- **Calls (std/external/unresolved):** `.find`, `.flatten`, `.get`, `.into_iter`, `.is_empty`, `.is_some`, `.len`, `.max`, `.min`, `.or`, `.unwrap_or`, `Err`, `Ok`, `Some`
- **Macros:** `matches!`
- **Called by:** `check_comment_anchors` (edit/v2_apply.rs:1913)

#### `resolve_operation`  (L2084-2243)
`fn resolve_operation( tx: &Transaction<'_>, operation: &v2::Operation, m: &Mapped, o: &mut EditOutcome, ) -> Result<Vec<Resolved>, EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `mapped_paragraph` (edit/v2_apply.rs:1650) L2094; `logical_text` (edit/v2.rs:802) L2096; `v2_select` (edit/anchor.rs:296) L2102; `occurrence` (edit/v2.rs:492) (by name) L2102; `Transaction::removed_anchor_in` (edit.rs:5354) L2104; `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L2107,2163; `Transaction::paragraph_id` (edit.rs:5085) L2112; `select` (audit.rs:239) (by name) L2123,2144,2158,2159,2211; `comment_component` (edit/v2_apply.rs:2244) L2125,2193,2215; `note` (ambiguous: `comment_listing_review_tests::note` (convert/mod.rs:46500); `note` (markdown/from_docx/critic.rs:1129)) L2125,2133,2176,2190; `text_component` (edit/v2_apply.rs:2269) L2127,2145,2170,2188; `exact_rewrite_ranges` (edit/v2_apply.rs:2316) L2187; `Transaction::resolve_thread` (edit.rs:4379) L2221; `Transaction::resolve_one` (edit.rs:3479) L2232
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.first`, `.flatten`, `.into`, `.is_empty`, `.len`, `.map_err`, `.map`, `.push`, `.then`, `.transpose`, `.unwrap_or`, `Err`, `Ok`, `Some`, `Vec::new`, `exact`, `usize::try_from`
- **Macros:** `vec!`
- **Called by:** `resolve` (edit/v2_apply.rs:1656)

#### `comment_component`  (L2244-2268)
`fn comment_component( tx: &Transaction<'_>, p: usize, start: usize, end: usize, note: Option<String>, ) -> Result<Vec<Resolved>, EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L2255,2261; `check_comment` (edit.rs:7134) L2261
- **Calls (std/external/unresolved):** `.map_err`, `Err`, `Ok`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `resolve_operation` (edit/v2_apply.rs:2084)

#### `text_component`  (L2269-2312)
`fn text_component( tx: &Transaction<'_>, p: usize, start: usize, end: usize, text: &str, note: Option<String>, attach: bool, ) -> Result<Resolved, EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `check_run_text` (edit.rs:7107) L2278; `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L2278,2281,2287,2293,2301; `check_comment` (edit.rs:7134) L2287; `Transaction::check_insert_position` (edit.rs:5381) L2291; `Transaction::check_range` (edit.rs:5160) L2300
- **Calls (std/external/unresolved):** `.into`, `.map_err`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `resolve_operation` (edit/v2_apply.rs:2084)

#### `exact_rewrite_ranges`  (L2316-2337)
`fn exact_rewrite_ranges(old: &str, new: &str) -> Vec<(usize, usize, String)>`

- **Does:** The same diff library as the legacy rewrite adapter, with exact character keys so whitespace/control changes cannot disappear through normalization.
- **Calls (std/external/unresolved):** `.as_tag_tuple`, `.chain`, `.char_indices`, `.chars`, `.collect`, `.filter_map`, `.into_iter`, `.iter`, `.len`, `.map`, `.then`, `similar::capture_diff_slices`, `std::iter::once`
- **Called by:** `resolve_operation` (edit/v2_apply.rs:2084)

#### `edited_revisions`  (L2344-2395)
`fn edited_revisions(tx: &Transaction<'_>) -> BTreeSet<String>`

- **Does:** The revisions the resolved operations reach: only insertion content a text operation touches may change, and a deleted paragraph reaches every revision it holds.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) (by name) L2354; `Held::of` (edit/hint.rs:58) L2355; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L2356; `ChangeId::short` (ids.rs:155) (by name) L2358,2390; `ChangeId::new` (ids.rs:142) L2358,2390; `attach_segment` (edit.rs:7348) L2375; `editable_insertion` (edit.rs:7316) L2386; `Dom::attribute` (xmllinq/mod.rs:874) L2387
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.insert`, `.into_iter`, `.is_some`, `.iter`, `BTreeSet::new`, `W::id`
- **Called by:** `apply_bound` (edit/v2_apply.rs:121)

#### `preserve_history`  (L2399-2407)
`fn preserve_history( working: &[u8], candidate: &[u8], edited: &BTreeSet<String>, author: &str, insertion_origins: &BTreeMap<String, String>, ) -> Result<(), EditError>`

- **Does:** Tracked edits retain another author's words, in order, either alive or in the editor's nested deletions.
- **Calls (crate):** `compare_history` (edit/v2_apply.rs:2422) L2406
- **Called by:** `tests::history_guard_rejects_words_dropped_without_a_nested_deletion` (edit/v2_apply.rs:2558); `tests::history_guard_checks_all_fragments_and_the_nested_deletion_author` (edit/v2_apply.rs:2603); `tests::history_guard_keeps_unedited_content_and_all_source_attribution` (edit/v2_apply.rs:2702)

#### `preserve_direct_history`  (L2412-2420)
`fn preserve_direct_history( working: &[u8], candidate: &[u8], edited: &BTreeSet<String>, author: &str, insertion_origins: &BTreeMap<String, String>, ) -> Result<(), EditError>`

- **Does:** The direct artifact intentionally applies new text without revision marks.
- **Calls (crate):** `compare_history` (edit/v2_apply.rs:2422) L2419
- **Called by:** `apply_bound` (edit/v2_apply.rs:121)

#### `compare_history`  (L2422-2549)
`fn compare_history( working: &[u8], candidate: &[u8], edited: &BTreeSet<String>, author: &str, insertion_origins: &BTreeMap<String, String>, tracked: bool, ) -> Result<(), EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L2431; `changes` (edit/v2_apply.rs:644) L2440; `existing_mark_layout_ownership_tests::history` (comparer/finalize.rs:33013) (by name) L2450,2455; `failure` (ambiguous: `failure` (bin/jubarte/edit_template.rs:19); `failure` (bin/jubarte/edit_typed.rs:17)) L2471,2484,2494,2518,2546
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.as_deref`, `.as_ref`, `.chain`, `.clone_from`, `.clone`, `.collect`, `.contains`, `.entry`, `.find`, `.get`, `.insert`, `.into_iter`, `.is_some_and`, `.iter_mut`, `.iter`, `.map`, `.ok_or_else`, `.or_default`, `.or_else`, `.push_str`, `.push`, `.retain_mut`, `BTreeMap::::new`, `BTreeSet::new`, `Err`, `Ok`, `Some`, `Vec::new`
- **Called by:** `preserve_history` (edit/v2_apply.rs:2399); `preserve_direct_history` (edit/v2_apply.rs:2412)

## Test functions (17)

- `capture_correspondence_states` (L37): Replace this thread's [`CORRESPONDENCE_STATES`] with the states the correspondence is about to read.
- `correspondence_states` (L44): Take this thread's [`CORRESPONDENCE_STATES`], clearing the slot.
- `reparsed_final_views` (L563): The emitted primary's final-view states, recomputed the way the correspondence computed them before #1087: reopen the bytes and walk their paragraphs again, in the checker's projection order (the body's, then each story part's).
- `tests::history_guard_rejects_words_dropped_without_a_nested_deletion` (L2558): _no doc comment_
- `tests::history_guard_ignores_a_comment_mark_inside_a_kept_revision` (L2579): _no doc comment_
- `tests::history_guard_checks_all_fragments_and_the_nested_deletion_author` (L2603): _no doc comment_
- `tests::history_guard_keeps_unedited_content_and_all_source_attribution` (L2702): _no doc comment_
- `tests::effectful_apply_never_reopens_its_primary_for_the_correspondence` (L2745): An effectful v2 apply builds its expectations correspondence without reopening the emitted primary: the final-view dispositions come from the DOM that produced the bytes, so no apply pays a second admission, unzip and parse of its own output for a correspondence only `--expect` reads (#1087).
- `tests::final_views_read_off_the_dom_match_a_reparse_of_the_emitted_primary` (L2801): The regression net behind #1087: the final-view states `apply_plan` hands the correspondence, read off the DOM that produced the primary, must equal a reparse of the emitted primary itself.
- `tests::net_plan` (L2892): One single-operation PlanV2 for the equivalence net.
- `tests::net_operation` (L2907): One operation of the equivalence net, addressed at `row`'s paired ids.
- `tests::many_comment_source` (L2960): _no doc comment_
- `tests::a_range_first_side_that_is_not_a_block_refuses_usage_not_a_panic` (L2988): _no doc comment_
- `tests::thread_only_and_unchanged_plans_do_not_scan_comment_anchors` (L3005): _no doc comment_
- `tests::existing_comment_anchors_scan_each_paragraph_once_per_apply` (L3033): _no doc comment_
- `tests::v2_comment_diagnosis_ignores_engine_message_wording` (L3051): _no doc comment_
- `tests::comment_inventory_preserves_first_complete_ranges_and_offsets` (L3072): _no doc comment_
