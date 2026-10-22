# Functions and methods in `src/edit/origin.rs`

29 non-test, 18 test. Part of the [symbol manifest](../README.md).

#### `Ids::paragraph_id`  (L48-91, pub(super))
`pub(super) fn paragraph_id(&self, id: &str) -> Option<Result<String, String>>`

- **Does:** The paragraph id [`crate::ids::paragraph_id`] prints for what `id` names, as the agent view prints them: `p3` and `header1.p1` as given, `header1.p0` → `header1` (the part stem, Word's own numbering), `t0.row1.cell2[.p1]` → that cell's first (or K-th) paragraph (`p40`).
- **Calls (crate):** `source_only` (edit/origin.rs:637) L49; `parse_location` (ids.rs:326) L53,74; `paragraph_id` (ids.rs:410) L68,81; `public_memory_anchor_boundary_tests::paragraph` (comments.rs:1421) L70,79; `Ids::table_paragraph` (edit/origin.rs:98) L71
- **Calls (std/external/unresolved):** `.as_slice`, `.as_str`, `.contains`, `.err`, `.join`, `.map_or_else`, `.map`, `.parse`, `.to_string`, `Err`, `Ok`, `Some`
- **Macros:** `format!`
- **Called by:** `Package::heading_skip` (audit.rs:532) (?); `Package::literal_bullet` (audit.rs:636) (?); `Package::empty_spacers` (audit.rs:661) (?); `Transaction::cell_control` (edit/controls.rs:303) (?); `Ids::position` (edit/origin.rs:167); `paragraph_ids` (edit.rs:1734); `Transaction::resolve_comment_text` (edit.rs:3057) (?); `Transaction::resolve_one_unreported` (edit.rs:3545) (?); `Transaction::resolve_list` (edit.rs:4730) (?); `Transaction::select` (edit.rs:4980) (?)

#### `Ids::table_paragraph`  (L98-162)
`fn table_paragraph( &self, head: &str, n: usize, row: usize, cell: usize, index: usize, ) -> Result<String, String>`

- **Does:** Table `n`, row `row`, cell `cell`: the id of that cell's `index`-th own paragraph.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L109; `plural` (ambiguous: `plural` (edit/anchor_error.rs:622); `plural` (markdown/from_docx/header.rs:212)) L116,127,138,151; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L119,130,141; `nearest` (edit.rs:2233) L122,133; `in_text_box` (inspect.rs:962) L146; `Ids::id` (edit/origin.rs:212) L161
- **Calls (std/external/unresolved):** `.collect`, `.copied`, `.filter`, `.first`, `.get`, `.into_iter`, `.iter`, `.len`, `.ok_or_else`, `.position`, `Err`, `Ok`, `Some`, `W::name`, `W::p`, `W::tbl`, `W::tc`, `crate::inspect::top_level_table_nodes`
- **Macros:** `format!`
- **Called by:** `Ids::paragraph_id` (edit/origin.rs:48)

#### `Ids::position`  (L167-209, pub(super))
`pub(super) fn position(&self, selector: &Selector) -> Result<Result<usize, usize>, Miss>`

- **Does:** The paragraph a positional selector names, as an index into `paragraph_nodes`; `None` for a text selector, and the story it searches.
- **Calls (crate):** `Ids::paragraph_id` (edit/origin.rs:48) L171; `paragraph_place` (ids.rs:417) L176
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.collect`, `.filter`, `.get`, `.iter`, `.join`, `.len`, `.position`, `.to_string`, `.unwrap_or`, `Err`, `Ok`, `Some`, `not_found`
- **Macros:** `format!`
- **Called by:** none found in src

#### `Ids::id`  (L212-215, pub(super))
`pub(super) fn id(&self, global: usize) -> String`

- **Does:** The id of a paragraph ([`crate::ids::paragraph_id`]).
- **Calls (crate):** `paragraph_id` (ids.rs:410) L214
- **Called by:** `Ids::table_paragraph` (edit/origin.rs:98); `Origin::exact` (edit/origin.rs:400) (?); `Origin::follow` (edit/origin.rs:418) (?); `Origin::moves` (edit/origin.rs:484) (?); `rename_positions` (edit/skip.rs:394) (?); `expectation_correspondence` (edit/v2_apply.rs:371) (?); `SourceModel::entity` (edit/v2_apply.rs:836) (?); `SourceModel::selector_pair` (edit/v2_apply.rs:1169) (?); `SourceModel::operation` (edit/v2_apply.rs:1388) (?)

#### `Origin::had_comment`  (L252-255, pub(super))
`pub(super) fn had_comment(&self, id: u32) -> bool`

- **Does:** Whether the source had comment `id`: settling the plan's revisions removes a comment whose anchor was text they took away.
- **Calls (crate):** `CommentFamily::load` (comments.rs:457) L253
- **Calls (std/external/unresolved):** `.contains`, `.is_ok_and`
- **Called by:** `Transaction::resolve_thread` (edit.rs:4379) (?)

#### `Origin::build`  (L260-365, pub(super))
`pub(super) fn build( source: &[u8], plan: &EditPlan, settled_ids: &[String], settled_story: &[(usize, usize)], ) -> Result<Self, EditError>`

- **Does:** Map the paragraphs of `source` onto those of the document settling it by `plan` gives, whose stories are `settled_ids` and whose paragraphs are `settled_story`.
- **Calls (crate):** `Opened::open` (inspect.rs:735) L266; `open` (ambiguous: `open` (append.rs:316); `open` (changes.rs:1031); `open` (edit/anchor_error.rs:229)) L267,273,293; `load_stories` (edit.rs:1680) L272,278,298; `tag_paragraphs` (edit/origin.rs:535) L279; `body_control_nodes` (inspect.rs:997) L282,328; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L284; `tag_control` (edit/origin.rs:885) L289; `package_of` (edit/origin.rs:550) L292; `settle` (edit.rs:1617) L293; `err` (edit.rs:1770) L305; `paragraph_tags` (edit/origin.rs:564) L311; `paragraph_holds_no_content` (edit/origin.rs:684) L314; `exact_correspondence` (edit/origin.rs:723) L316; `collect_controls` (inspect.rs:1005) L323; `Dom::element` (xmllinq/mod.rs:655) (by name) L332,335; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L336; `fates` (edit/origin.rs:788) L346; `project_paragraph` (inspect.rs:1265) L350,351; `settles_all` (edit/origin.rs:513) L360
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.checked_sub`, `.clone`, `.collect`, `.enumerate`, `.eq`, `.get_mut`, `.into_iter`, `.iter`, `.len`, `.map_err`, `.map_or`, `.map`, `.ok`, `.parse`, `.position`, `Err`, `Ok`, `Some`, `W::name`, `W::sdt_pr`, `W::sdt`, `W::val`
- **Macros:** `vec!`
- **Called by:** `task_usage` (cli.rs:3755) (?); `for_edit` (edit/for_edit.rs:1581); `apply_bound` (edit/v2_apply.rs:121); `Transaction::start` (edit.rs:2680); `run` (update.rs:225) (?)

#### `Origin::control_ids`  (L368-370, pub(super))
`pub(super) fn control_ids(&self) -> &[String]`

- **Does:** The source's body content control ids, in document order.
- **Calls:** none
- **Called by:** `Transaction::select_control` (edit/controls.rs:207) (?)

#### `Origin::control_source`  (L373-376, pub(super))
`pub(super) fn control_source(&self, post: usize) -> Option<&str>`

- **Does:** The source id of settled control `post`, when it came from one.
- **Calls (std/external/unresolved):** `.as_str`, `.iter`, `.position`, `Some`
- **Called by:** `Transaction::select_control` (edit/controls.rs:207) (?)

#### `Origin::control`  (L381-386, pub(super))
`pub(super) fn control(&self, id: &str) -> Option<Result<usize, String>>`

- **Does:** The settled control (its index among the settled body's controls) the source control `id` became; `None` when the source has no control `id`.
- **Calls (std/external/unresolved):** `.iter`, `.ok_or_else`, `.position`, `Some`
- **Macros:** `format!`
- **Called by:** `Transaction::select_control` (edit/controls.rs:207) (?)

#### `Origin::ids`  (L388-396, pub(super))
`pub(super) fn ids(&self) -> Ids<'_>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`
- **Called by:** `Origin::follow` (edit/origin.rs:418); `Origin::moves` (edit/origin.rs:484)

#### `Origin::exact`  (L400-411, pub(super))
`pub(super) fn exact(&self, source: usize) -> Result<usize, Miss>`

- **Does:** Follow only a unique surviving paragraph tag.
- **Calls (crate):** `.ids` (ambiguous, 6 candidates) L406; `Ids::id` (edit/origin.rs:212) (by name) L406
- **Calls (std/external/unresolved):** `.into`, `.ok_or_else`
- **Macros:** `format!`
- **Called by:** `for_edit` (edit/for_edit.rs:1581); `tests::split_correspondence_is_refused_by_origin` (edit/origin.rs:911); `mapped_paragraph` (edit/v2_apply.rs:1650) (?)

#### `Origin::follow`  (L418-474, pub(super))
`pub(super) fn follow( &self, source: usize, merged_ok: bool, settled_id: impl Fn(usize) -> String, by_text: impl Fn(usize) -> Option<String>, ) -> Result<usize, Miss>`

- **Does:** The settled paragraph source paragraph `source` became.
- **Calls (crate):** `Origin::ids` (edit/origin.rs:388) L425; `Ids::id` (edit/origin.rs:212) (by name) L426,432; `join_ids` (edit/origin.rs:502) L437
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter`, `.into`, `.is_empty`, `.len`, `.map`, `.to_string`, `Err`, `Fate::Merged`, `Ok`, `by_text`, `settled_id`
- **Macros:** `format!`
- **Called by:** `resolve` (edit/v2_apply.rs:1656) (?); `Transaction::select` (edit.rs:4980) (?)

#### `Origin::fate`  (L478-480, pub(super))
`pub(super) fn fate(&self, source: usize) -> Fate`

- **Does:** The [`Fate`] of source paragraph `source`: kept one to one, merged into a settled paragraph, or gone with its text.
- **Calls:** none
- **Called by:** `for_edit` (edit/for_edit.rs:1581); `expectation_correspondence` (edit/v2_apply.rs:371) (?)

#### `Origin::moves`  (L484-498, pub(super))
`pub(super) fn moves(&self, settled_id: impl Fn(usize) -> String) -> Vec<super::ParagraphMove>`

- **Does:** Every source id that names another paragraph, or none, once the plan's revisions are settled, in source order.
- **Calls (crate):** `Origin::ids` (edit/origin.rs:388) L485; `Ids::id` (edit/origin.rs:212) (by name) L490
- **Calls (std/external/unresolved):** `.as_deref`, `.as_str`, `.collect`, `.enumerate`, `.filter_map`, `.iter`, `.then_some`, `Some`, `settled_id`
- **Called by:** `Transaction::report` (edit.rs:3161) (?)

#### `join_ids`  (L502-508)
`fn join_ids(ids: &[String]) -> String`

- **Does:** `p4`, `p4 and p6`, `p4, p6 and p7`.
- **Calls (std/external/unresolved):** `.clone`, `.join`, `String::new`
- **Macros:** `format!`
- **Called by:** `Origin::follow` (edit/origin.rs:418); `tests::join_ids_lists_them_as_prose` (edit/origin.rs:1203)

#### `settles_all`  (L513-530)
`fn settles_all(plan: &EditPlan) -> Option<&'static str>`

- **Does:** `accept` when settling `plan`'s revisions accepts every one, `reject` when it rejects every one, `None` when it accepts some and rejects others, or rejects visibly.
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.is_none`, `.is_some_and`, `Some`, `every`
- **Called by:** `Origin::build` (edit/origin.rs:260); `tests::settles_all_names_one_command_only_when_every_revision_goes_one_way` (edit/origin.rs:1172)

#### `tag_paragraphs`  (L535-547)
`fn tag_paragraphs(opened: &mut Opened, stories: &[super::StoryPart], paragraph_nodes: &[NodeId])`

- **Does:** Tag `opened`'s paragraphs: in every story, the `w14:paraId` of paragraph g (`paragraph_nodes` order) is g + 1, and every other paraId goes, so no tag is ambiguous.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L538; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L539; `Dom::set_attribute_value` (xmllinq/mod.rs:977) (by name) L543
- **Calls (std/external/unresolved):** `.enumerate`, `.iter`, `Some`, `W14::name`, `W::p`
- **Macros:** `format!`
- **Called by:** `Origin::build` (edit/origin.rs:260); `written_places` (edit/origin.rs:649)

#### `package_of`  (L550-559)
`fn package_of(opened: &mut Opened, stories: &[super::StoryPart]) -> Result<Vec<u8>, EditError>`

- **Does:** `opened` as package bytes, its stories' DOMs written back.
- **Calls (crate):** `Dom::serialize_document` (xmllinq/mod.rs:1338) L552; `PartFs::set_part` (opc/mod.rs:661) L553; `PartFs::to_zip` (opc/mod.rs:749) (by name) L555; `err` (edit.rs:1770) L558
- **Calls (std/external/unresolved):** `.into_bytes`, `.map_err`, `.to_string`
- **Called by:** `Origin::build` (edit/origin.rs:260); `written_places` (edit/origin.rs:649)

#### `paragraph_tags`  (L564-574)
`fn paragraph_tags(dom: &Dom, nodes: &[NodeId], count: usize) -> Vec<Option<usize>>`

- **Does:** The source paragraph each of `nodes` carries the tag of ([`tag_paragraphs`]); `None` for one with no tag, or a tag no paragraph of `count` has.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L569
- **Calls (std/external/unresolved):** `.checked_sub`, `.collect`, `.iter`, `.map`, `.ok`, `.then_some`, `W14::name`, `usize::from_str_radix`
- **Called by:** `Origin::build` (edit/origin.rs:260); `written_places` (edit/origin.rs:649)

#### `Paragraphs::open`  (L585-598, pub(super))
`pub(super) fn open(docx: &[u8]) -> Result<Self, EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `Opened::open` (inspect.rs:735) L586; `load_stories` (edit.rs:1680) L591
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map_err`, `.map`, `Ok`
- **Called by:** `written_places` (edit/origin.rs:649); `rename_positions` (edit/skip.rs:394)

#### `Paragraphs::ids`  (L600-608, pub(super))
`pub(super) fn ids(&self) -> Ids<'_>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`
- **Called by:** none found in src

#### `Paragraphs::control_count`  (L612-622, pub(super))
`pub(super) fn control_count(&self, global: usize) -> usize`

- **Does:** How many content controls paragraph `global` holds: the `ccK` its `pN.ccK` ids count (text boxes aside, as `inspect` numbers them).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) (by name) L615; `in_text_box` (inspect.rs:962) L618; `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L619
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.first`, `.into_iter`, `Some`, `W::name`, `W::p`
- **Called by:** `rename_positions` (edit/skip.rs:394) (?)

#### `Paragraphs::place`  (L625-628, pub(super))
`pub(super) fn place(&self, global: usize) -> (&str, usize)`

- **Does:** The story id and the index in it of paragraph `global`.
- **Calls:** none
- **Called by:** none found in src

#### `source_only`  (L637-643, pub(super))
`pub(super) fn source_only(id: &str) -> Option<String>`

- **Does:** The refusal of a [`SOURCE_ONLY`] name; `None` for any other.
- **Calls (std/external/unresolved):** `.starts_with`, `.then`
- **Macros:** `format!`
- **Called by:** `Transaction::select_control` (edit/controls.rs:207); `Ids::paragraph_id` (edit/origin.rs:48)

#### `written_places`  (L649-677, pub(super))
`pub(super) fn written_places( source: &[u8], write: impl FnOnce(&[u8]) -> Result<Vec<u8>, EditError>, ) -> Result<(Vec<Option<usize>>, Paragraphs), EditError>`

- **Does:** Where each paragraph of `source` is in the document `write` makes of `source` with its paragraphs tagged: per source paragraph (`load_stories` order), its index among the written document's paragraphs, when exactly one of them carries its tag; and those paragraphs.
- **Calls (crate):** `Opened::open` (inspect.rs:735) L653; `load_stories` (edit.rs:1680) L658; `tag_paragraphs` (edit/origin.rs:535) L660; `Paragraphs::open` (edit/origin.rs:585) L661; `write` (markdown/view_stories.rs:1599) (by name) L661; `package_of` (edit/origin.rs:550) L661; `paragraph_tags` (edit/origin.rs:564) L662
- **Calls (std/external/unresolved):** `.enumerate`, `.iter_mut`, `.iter`, `.len`, `.map_err`, `.zip`, `Ok`, `Some`
- **Macros:** `vec!`
- **Called by:** `rename_positions` (edit/skip.rs:394) (?)

#### `paragraph_holds_no_content`  (L684-714)
`fn paragraph_holds_no_content(dom: &Dom, paragraph: NodeId) -> bool`

- **Does:** Whether a settled paragraph holds only properties and content-free markers.
- **Calls (crate):** `Dom::nodes` (xmllinq/mod.rs:604) L685,689; `Dom::name` (xmllinq/mod.rs:526) L686; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L687; `XName::local_name` (xmllinq/mod.rs:208) (by name) L688,691
- **Calls (std/external/unresolved):** `.all`, `.into_iter`, `.is_empty`, `.is_some_and`
- **Macros:** `matches!`
- **Called by:** `Origin::build` (edit/origin.rs:260); `tests::settled_paragraph_content_free_markers_hold_no_content` (edit/origin.rs:1068); `tests::settled_paragraph_unprojected_content_is_not_content_free` (edit/origin.rs:1101)

#### `exact_correspondence`  (L723-776)
`fn exact_correspondence( tags: &[Option<usize>], content_free: &[bool], settled_story: &[(usize, usize)], paragraph_story: &[(usize, usize)], sources: usize, ) -> Vec<Option<usize>>`

- **Does:** Follow unique source tags.
- **Calls (crate):** `story` (ambiguous, 6 candidates) L743,747
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.checked_sub`, `.enumerate`, `.filter`, `.flatten`, `.into_iter`, `.is_none`, `.is_some`, `.iter`, `.len`, `.rev`, `.take_while`, `.then_some`, `Some`, `explained`, `same_story`
- **Macros:** `vec!`
- **Called by:** `Origin::build` (edit/origin.rs:260); `tests::split_correspondence_is_refused_by_origin` (edit/origin.rs:911); `tests::exact_correspondence_keeps_unique_tags` (edit/origin.rs:945); `tests::exact_correspondence_refuses_merges_and_duplicate_tags` (edit/origin.rs:959); `tests::exact_correspondence_keeps_the_neighbours_of_a_merge` (edit/origin.rs:977); `tests::exact_correspondence_refuses_both_split_neighbours_in_the_same_story` (edit/origin.rs:1006); `tests::exact_correspondence_refuses_split_neighbours_at_story_edges` (edit/origin.rs:1020); `tests::exact_correspondence_keeps_neighbours_of_content_free_repairs` (edit/origin.rs:1049); `tests::exact_correspondence_checks_split_runs_in_each_story` (edit/origin.rs:1130); `tests::exact_correspondence_checks_split_runs_across_an_untagged_story` (edit/origin.rs:1144); `tests::exact_correspondence_keeps_the_neighbour_of_a_trailing_merge` (edit/origin.rs:1158)

#### `fates`  (L788-882)
`fn fates( tags: &[Option<usize>], source_story: impl Fn(usize) -> String, settled_story: impl Fn(usize) -> String, source_text: impl Fn(usize) -> String, settled_text: impl Fn(usize) -> String, count: usize, ) -> Vec<Fate>`

- **Does:** Where each of `count` source paragraphs went, from the source tag each settled paragraph carries (`tags`).
- **Calls (std/external/unresolved):** `.as_slice`, `.checked_sub`, `.clone`, `.collect`, `.enumerate`, `.filter`, `.find_map`, `.find`, `.get`, `.is_empty`, `.is_some`, `.iter`, `.len`, `.map_or`, `.map`, `.or_else`, `.rev`, `.then_some`, `.unwrap_or`, `Fate::Kept`, `Fate::Merged`, `Some`, `beside`, `merged_at`, `settled_story`, `settled_text`, `source_story`, `source_text`
- **Macros:** `vec!`
- **Called by:** `Origin::build` (edit/origin.rs:260); `tests::untagged_paragraphs_are_merges_of_the_text_that_vanished_around_them` (edit/origin.rs:1211); `tests::a_paragraph_whose_text_the_settle_changed_joins_the_merge_beside_it` (edit/origin.rs:1290)

#### `tag_control`  (L885-903)
`fn tag_control(dom: &mut Dom, sdt: NodeId, tag: usize)`

- **Does:** Write `tag` into the control's `w:sdtPr/w:id`, creating them if needed.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L886,894; `Dom::new_element` (xmllinq/mod.rs:478) L889,897; `Dom::add_first` (xmllinq/mod.rs:1101) L890,898; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L902
- **Calls (std/external/unresolved):** `.to_string`, `Some`, `W::name`, `W::sdt_pr`, `W::val`
- **Called by:** `Origin::build` (edit/origin.rs:260)

## Test functions (18)

- `tests::split_correspondence_is_refused_by_origin` (L911): _no doc comment_
- `tests::exact_correspondence_keeps_unique_tags` (L945): _no doc comment_
- `tests::exact_correspondence_refuses_merges_and_duplicate_tags` (L959): _no doc comment_
- `tests::exact_correspondence_keeps_the_neighbours_of_a_merge` (L977): _no doc comment_
- `tests::exact_correspondence_refuses_both_split_neighbours_in_the_same_story` (L1006): _no doc comment_
- `tests::exact_correspondence_refuses_split_neighbours_at_story_edges` (L1020): _no doc comment_
- `tests::exact_correspondence_keeps_neighbours_of_content_free_repairs` (L1049): _no doc comment_
- `tests::settled_paragraph_content_free_markers_hold_no_content` (L1068): _no doc comment_
- `tests::settled_paragraph_unprojected_content_is_not_content_free` (L1101): _no doc comment_
- `tests::exact_correspondence_checks_split_runs_in_each_story` (L1130): _no doc comment_
- `tests::exact_correspondence_checks_split_runs_across_an_untagged_story` (L1144): _no doc comment_
- `tests::exact_correspondence_keeps_the_neighbour_of_a_trailing_merge` (L1158): _no doc comment_
- `tests::settles_all_names_one_command_only_when_every_revision_goes_one_way` (L1172): _no doc comment_
- `tests::join_ids_lists_them_as_prose` (L1203): _no doc comment_
- `tests::untagged_paragraphs_are_merges_of_the_text_that_vanished_around_them` (L1211): _no doc comment_
- `tests::untagged_paragraphs_are_merges_of_the_text_that_vanished_around_them::{fn texts}` (L1213): _no doc comment_
- `tests::a_paragraph_whose_text_the_settle_changed_joins_the_merge_beside_it` (L1290): _no doc comment_
- `tests::a_paragraph_whose_text_the_settle_changed_joins_the_merge_beside_it::{fn texts}` (L1292): _no doc comment_
