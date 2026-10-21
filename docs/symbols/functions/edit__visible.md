# Functions and methods in `src/edit/visible.rs`

20 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `Visible::verb`  (L33-38, pub(super))
`pub(super) fn verb(self) -> &'static str`

- **Does:** The operation kind's name.
- **Calls:** none
- **Called by:** `named` (edit/hint.rs:259) (?); `check_keep` (edit/visible.rs:71) (?); `Transaction::resolve_visible` (edit/visible.rs:348)

#### `Visible::past`  (L40-45)
`fn past(self) -> &'static str`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `Transaction::resolve_visible` (edit/visible.rs:348) (?)

#### `Visible::takes`  (L48-53)
`fn takes(self) -> &'static str`

- **Does:** What it takes, as the refusals say it.
- **Calls:** none
- **Called by:** `Transaction::resolve_visible` (edit/visible.rs:348) (?)

#### `of`  (L57-67, pub(super))
`pub(super) fn of(kind: &OperationKind) -> Option<(Visible, &str, Option<&str>)>`

- **Does:** The visible reject `kind` asks for, with its revision and comment.
- **Calls (std/external/unresolved):** `.as_deref`, `Some`
- **Called by:** `check_keep` (edit/visible.rs:71)

#### `check_keep`  (L71-89, pub(super))
`pub(super) fn check_keep(plan: &EditPlan) -> Result<(), EditError>`

- **Does:** `INVALID_PLAN` for a strike or a restore in a plan that does not keep the source's revisions: the change it names would already be settled.
- **Calls (crate):** `of` (edit/visible.rs:57) L75; `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L84; `.verb` (ambiguous: `Held::verb` (edit/hint.rs:72); `Visible::verb` (edit/visible.rs:33)) L87
- **Calls (std/external/unresolved):** `.find_map`, `.iter`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `Transaction::start` (edit.rs:2680)

#### `lower`  (L95-131, pub(super))
`pub(super) fn lower(source: &[u8], plan: &EditPlan) -> Result<Option<EditPlan>, EditError>`

- **Does:** `plan` with `resolve_revisions.reject.visible` lowered: each change the reject side selects becomes a `strike` (an insertion) or a `restore` (a deletion) after the plan's operations, in document order, and the reject side settles nothing.
- **Calls (crate):** `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L100; `check_whole_changes` (changes.rs:596) L107; `list_changes` (changes.rs:564) L109; `selected_ids` (edit.rs:1450) L110; `reject_kind` (edit/visible.rs:136) L122
- **Calls (std/external/unresolved):** `.any`, `.as_ref`, `.clone`, `.filter`, `.iter`, `.map_err`, `.map`, `.push`, `Err`, `Ok`, `Some`
- **Called by:** `prepare` (edit/prepared.rs:211); `prepare_skipping` (edit/skip.rs:43); `replies_on_deleted_paragraphs` (edit.rs:1588); `check_plan_quoting` (edit.rs:2349)

#### `reject_kind`  (L136-152, pub(super))
`pub(super) fn reject_kind( listed: &[Change], revision: String, comment: Option<String>, ) -> OperationKind`

- **Does:** The visible reject of `revision`: a `restore` of a deletion or of a move's source, a `strike` of anything else, an id `listed` lacks included (the strike then refuses it as unknown).
- **Calls (std/external/unresolved):** `.find`, `.is_some_and`, `.iter`, `Some`
- **Macros:** `matches!`
- **Called by:** `plan_from_flags` (edit/flags.rs:85); `lower` (edit/visible.rs:95)

#### `local`  (L185-189)
`fn local(dom: &Dom, node: NodeId) -> String`

- **Does:** `node`'s local name, empty for a text node.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L186; `XName::local_name` (xmllinq/mod.rs:208) (by name) L187
- **Calls (std/external/unresolved):** `.map`, `.to_string`, `.unwrap_or_default`
- **Called by:** `not_plain` (edit/visible.rs:192); `unsupported_child` (edit/visible.rs:210); `holds_words` (edit/visible.rs:242); `Transaction::resolve_visible` (edit/visible.rs:348)

#### `not_plain`  (L192-206)
`fn not_plain(dom: &Dom, run: NodeId) -> Option<&'static str>`

- **Does:** What in `run` is not plain text, as a refusal names it.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L193; `local` (edit/visible.rs:185) L194
- **Calls (std/external/unresolved):** `.as_str`, `.contains`, `.find_map`, `.into_iter`, `Some`
- **Called by:** `unsupported_child` (edit/visible.rs:210)

#### `unsupported_child`  (L210-222)
`fn unsupported_child(dom: &Dom, container: NodeId, nested: &[&str]) -> Option<&'static str>`

- **Does:** What among `container`'s children is not a run of plain text, a marker or one of `nested`, as a refusal names it.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L211; `local` (edit/visible.rs:185) L213; `not_plain` (edit/visible.rs:192) L214
- **Calls (std/external/unresolved):** `.as_str`, `.contains`, `.find_map`, `.into_iter`, `Some`
- **Called by:** `restorable` (edit/visible.rs:257); `Transaction::resolve_visible` (edit/visible.rs:348)

#### `in_paragraph`  (L226-228)
`fn in_paragraph(dom: &Dom, parent: NodeId) -> bool`

- **Does:** Whether a deletion whose parent is `parent` sits where restore puts text back: in a paragraph, or in an insertion directly inside one.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L227; `container` (edit/visible.rs:232) L227
- **Calls (std/external/unresolved):** `W::p`
- **Called by:** `restorable` (edit/visible.rs:257); `Transaction::resolve_visible` (edit/visible.rs:348)

#### `container`  (L232-238)
`fn container(dom: &Dom, parent: NodeId) -> NodeId`

- **Does:** What holds a deletion whose parent is `parent`: that parent, or past an insertion the one holding it.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L233; `Dom::parent` (xmllinq/mod.rs:597) L234
- **Calls (std/external/unresolved):** `.unwrap_or`, `W::ins`
- **Called by:** `in_paragraph` (edit/visible.rs:226); `Transaction::resolve_visible` (edit/visible.rs:348)

#### `holds_words`  (L242-251)
`fn holds_words(dom: &Dom, del: NodeId) -> bool`

- **Does:** Whether `del` holds words to put back: deleted text, or a glyph typed in one of its runs (a tab, a break, a symbol, a hyphen).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L243; `local` (edit/visible.rs:185) L244; `Dom::parent` (xmllinq/mod.rs:597) L248; `Dom::name_is` (xmllinq/mod.rs:535) L248; `Dom::value` (xmllinq/mod.rs:1185) L249
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.into_iter`, `.is_empty`, `.is_some_and`, `W::r`
- **Macros:** `matches!`
- **Called by:** `restorable` (edit/visible.rs:257); `Transaction::resolve_visible` (edit/visible.rs:348)

#### `restorable`  (L257-263, pub(super))
`pub(super) fn restorable(dom: &Dom, del: NodeId, stretches: usize) -> bool`

- **Does:** Whether restore takes `del`, a `w:del` its revision spreads over `stretches` places: one stretch of plain deleted words in a paragraph.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L259; `Dom::parent` (xmllinq/mod.rs:597) L260; `in_paragraph` (edit/visible.rs:226) L260; `holds_words` (edit/visible.rs:242) L261; `unsupported_child` (edit/visible.rs:210) L262
- **Calls (std/external/unresolved):** `.is_none`, `.is_some_and`, `W::name`
- **Called by:** `Transaction::revision_hint` (edit.rs:5259)

#### `copy_runs`  (L267-283, pub(super))
`pub(super) fn copy_runs(dom: &mut Dom, del: NodeId) -> Vec<NodeId>`

- **Does:** Detached copies of `del`'s runs as inserted text: `w:delText` read as `w:t`, the run formatting without another revision's marks.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L269,271; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L270; `Dom::name_is` (xmllinq/mod.rs:535) L272,274; `Dom::set_name` (xmllinq/mod.rs:539) L273; `clean_rpr` (edit/tracked.rs:1705) L275; `Dom::add_before_self` (xmllinq/mod.rs:1130) L276; `Dom::remove` (xmllinq/mod.rs:1118) L277
- **Calls (std/external/unresolved):** `.push`, `Some`, `Vec::new`, `W::del_text`, `W::r_pr`, `W::r`, `W::t`
- **Called by:** `emit` (edit/tracked.rs:452); `restore_plain` (edit/visible.rs:311); `Transaction::resolve_visible` (edit/visible.rs:348)

#### `revision_in`  (L287-294, pub(super))
`pub(super) fn revision_in(dom: &Dom, paragraph: NodeId, local: &str, wid: &str) -> Option<NodeId>`

- **Does:** The run-level revision `local` (`ins`, `del`) with `w:id` `wid` in `paragraph`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L288; `Dom::attribute` (xmllinq/mod.rs:874) L291; `Dom::parent` (xmllinq/mod.rs:597) L292; `Dom::name_is` (xmllinq/mod.rs:535) L292
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.is_some_and`, `Some`, `W::id`, `W::name`, `W::r_pr`
- **Called by:** `emit` (edit/tracked.rs:452); `restore_plain` (edit/visible.rs:311)

#### `comment_around`  (L299-307, pub(super))
`pub(super) fn comment_around(dom: &mut Dom, first: NodeId, last: NodeId, id: u32)`

- **Does:** Comment `id` from `first` to `last`, siblings in that order: the start marker before `first`, the end marker and the reference run after `last`.
- **Calls (crate):** `comment_marker` (edit.rs:7821) (by name) L301,302; `comment_reference_run` (edit.rs:7827) (by name) L303; `Dom::add_before_self` (xmllinq/mod.rs:1130) L304; `Dom::add_after_self` (xmllinq/mod.rs:1147) L305,306
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `emit` (edit/tracked.rs:452); `Transaction::apply` (edit.rs:5882)

#### `restore_plain`  (L311-324, pub(super))
`pub(super) fn restore_plain( dom: &mut Dom, paragraph: NodeId, wid: &str, ) -> Option<(NodeId, NodeId)>`

- **Does:** The clean copy's restore: plain copies of their deletion `wid`'s runs right after it, which stays.
- **Calls (crate):** `revision_in` (edit/visible.rs:287) L316; `copy_runs` (edit/visible.rs:267) L317; `Dom::add_after_self` (xmllinq/mod.rs:1147) L320
- **Calls (std/external/unresolved):** `.first`, `Some`
- **Called by:** `Transaction::apply` (edit.rs:5882)

#### `offset_before`  (L328-340)
`fn offset_before(tx: &Transaction<'_>, para: usize, node: NodeId) -> usize`

- **Does:** The projection offset at `node` in paragraph `para`: the end of the last projected piece before it.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) (by name) L330; `index` (ambiguous: `coverage_boundary_tests::index` (document_comparer.rs:8844); `index` (legacy_doc.rs:616)) L332,336; `run_of` (edit.rs:7980) (by name) L336
- **Calls (std/external/unresolved):** `.filter`, `.iter`, `.map`, `.max`, `.position`, `.unwrap_or`
- **Called by:** `Transaction::resolve_visible` (edit/visible.rs:348)

#### `Transaction::resolve_visible`  (L348-672, pub(super))
`pub(super) fn resolve_visible( &mut self, id: &str, visible: Visible, revision: &str, comment: Option<&str>, ) -> Result<(Vec<Resolved>, EditOutcome), Failure>`

- **Does:** Resolve a `strike` or a `restore` of `revision`: the text deletions that strike their insertion, or the copy that restores their deletion.
- **Calls (crate):** `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L370; `Visible::verb` (edit/visible.rs:33) L372; `parse_change_ids` (ids.rs:232) L373; `ChangeId::short` (ids.rs:155) (by name) L388; `Visible::past` (edit/visible.rs:40) (by name) L395; `check_comment` (edit.rs:7087) (by name) L401; `list_changes` (changes.rs:564) L404; `unknown` (ambiguous: `coverage_round_three_tests::unknown` (comparer/lcs.rs:11929); `coverage_round_four_tests::unknown` (comparer/lcs.rs:12930); `coverage_round_next_tests::unknown` (comparer/lcs.rs:14299)) L407,499; `Visible::takes` (edit/visible.rs:48) (by name) L465,494; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L506,626; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L510,629; `.parent` (ambiguous: `Marks::parent` (markdown/view_stories.rs:450); `Dom::parent` (xmllinq/mod.rs:597)) L511,557,614; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L511; `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L515; `paragraph_of` (ambiguous: `paragraph_of` (changes.rs:1388); `paragraph_of` (comparer/bookmarks.rs:76)) L519,546; `Transaction::paragraph_id` (edit.rs:5085) L526; `unsupported_child` (edit/visible.rs:210) L541,644; `run_of` (edit.rs:7980) (by name) L556; `editable_insertion` (edit.rs:7269) (by name) L560; `Dom::next_element` (xmllinq/mod.rs:855) (by name) L568; `excerpt` (edit.rs:7114) (by name) L604,659; `in_paragraph` (edit/visible.rs:226) L615; `local` (edit/visible.rs:185) L616; `container` (edit/visible.rs:232) L616; `holds_words` (edit/visible.rs:242) L631; `offset_before` (edit/visible.rs:328) L649; `Dom::new_element` (xmllinq/mod.rs:478) (by name) L653; `copy_runs` (edit/visible.rs:267) L654; `.add` (ambiguous, 5 candidates) L655; `project_paragraph` (inspect.rs:1265) (by name) L657
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.as_slice`, `.as_str`, `.clone`, `.collect`, `.contains`, `.expect`, `.extend`, `.filter_map`, `.filter`, `.find_map`, `.find`, `.first`, `.into_iter`, `.into`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.join`, `.last_mut`, `.last`, `.len`, `.map_err`, `.map`, `.position`, `.push_str`, `.push`, `.replace`, `.to_string`, `.trim`, `.unwrap_or`, `Box::new`, `Err`, `Ok`, `Some`, `String::new`, `Vec::new`, `W::id`, `W::name`, `W::p`, `W::r_pr`, `fail`, `unsupported`
- **Macros:** `format!`, `vec!`
- **Called by:** `Transaction::resolve` (edit.rs:3267)
