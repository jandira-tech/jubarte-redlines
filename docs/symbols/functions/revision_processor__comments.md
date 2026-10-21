# Functions and methods in `src/revision_processor/comments.rs`

13 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `Parsed::load`  (L41-52, pub(super))
`pub(super) fn load(pkg: &PartFs, name: &str) -> Option<Self>`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L42; `Dom::new` (xmllinq/mod.rs:387) L43; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L44; `Dom::root` (xmllinq/mod.rs:822) L45
- **Calls (std/external/unresolved):** `.to_string`, `Some`
- **Called by:** `prune_orphan_comments` (revision_processor/comments.rs:274); `prune_by_para` (revision_processor/comments.rs:411); `prune_people` (revision_processor/comments.rs:467); `prune_orphan_notes` (revision_processor/notes.rs:24); `tidy` (revision_processor/word_save.rs:25); `drop_unreferenced_headers_footers` (revision_processor/word_save.rs:59); `non_paragraph_styles` (revision_processor/word_save.rs:88)

#### `Parsed::store`  (L54-59, pub(super))
`pub(super) fn store(self, pkg: &mut PartFs)`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::set_part` (opc/mod.rs:661) L55; `Dom::serialize_document` (xmllinq/mod.rs:1338) L57
- **Calls (std/external/unresolved):** `.into_bytes`
- **Called by:** `append_documents` (append.rs:152) (?); `cli_main` (bin/jubarte.rs:3935) (?); `Transaction::write_comments_part` (edit.rs:6714) (?); `threads` (markdown/package.rs:74) (?); `active::reset` (perf.rs:240) (?); `active::set_atom_count` (perf.rs:277) (?); `active::set_unit_count` (perf.rs:281) (?); `prune_orphan_comments` (revision_processor/comments.rs:274) (?); `prune_by_para` (revision_processor/comments.rs:411) (?); `prune_people` (revision_processor/comments.rs:467) (?); `prune_orphan_notes` (revision_processor/notes.rs:24) (?); `tidy` (revision_processor/word_save.rs:25) (?); +4 more

#### `Parsed::w`  (L62-64, pub(super))
`pub(super) fn w(&self, local: &str) -> Vec<NodeId>`

- **Does:** Descendants named `local` in the W namespace.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L63
- **Calls (std/external/unresolved):** `Some`, `W::name`
- **Called by:** `prune_orphan_comments` (revision_processor/comments.rs:274) (?); `collapse_cut_ranges` (revision_processor/comments.rs:361); `prune_orphan_notes` (revision_processor/notes.rs:24) (?); `tidy` (revision_processor/word_save.rs:25) (?); `drop_unreferenced_headers_footers` (revision_processor/word_save.rs:59) (?); `non_paragraph_styles` (revision_processor/word_save.rs:88) (?)

#### `drop_wholly_deleted_comments`  (L71-91, pub(super))
`pub(super) fn drop_wholly_deleted_comments(dom: &mut Dom, root: NodeId)`

- **Does:** Remove every comment range whose entire content is deleted, including its reference run.
- **Calls (crate):** `wholly_deleted_ranges` (revision_processor/bookmarks.rs:32) L72; `Dom::descendants` (xmllinq/mod.rs:668) L82; `Dom::attribute` (xmllinq/mod.rs:874) L83; `remove_marker` (revision_processor/comments.rs:258) L87
- **Calls (std/external/unresolved):** `.contains`, `.is_empty`, `.is_some_and`, `Some`, `W::id`, `W::name`
- **Called by:** `accept_content` (revision_processor.rs:3481)

#### `keep_wholly_deleted_comments`  (L96-236, pub(super))
`pub(super) fn keep_wholly_deleted_comments(dom: &mut Dom, root: NodeId) -> Vec<String>`

- **Does:** Collapse wholly removed ranges at their original position before deletion.
- **Calls (crate):** `wholly_deleted_ranges` (revision_processor/bookmarks.rs:32) L97; `Dom::descendants` (xmllinq/mod.rs:668) L103,106,202; `Dom::attribute` (xmllinq/mod.rs:874) L107,125,203; `Dom::name` (xmllinq/mod.rs:526) L124,128,133,147,179; `is_character` (revision_processor/bookmarks.rs:112) L129; `Dom::parent` (xmllinq/mod.rs:597) L132,138,213; `removed_container` (revision_processor/comments.rs:240) L144,147,162; `Dom::new_element` (xmllinq/mod.rs:478) L174,199,212; `Dom::elements` (xmllinq/mod.rs:640) L176; `Dom::add_before_self` (xmllinq/mod.rs:1130) L181,200,222,225; `Dom::add` (xmllinq/mod.rs:1074) L183,217,220; `Dom::element` (xmllinq/mod.rs:655) L214; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L216,219; `remove_marker` (revision_processor/comments.rs:258) L221; `Dom::remove` (xmllinq/mod.rs:1118) L224,229,233
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_str`, `.contains`, `.copied`, `.expect`, `.extend`, `.filter`, `.find`, `.into_iter`, `.is_none`, `.is_some_and`, `.is_some`, `.iter_mut`, `.iter`, `.map`, `.or_else`, `.position`, `.push`, `.rev`, `.take_while`, `.unwrap_or`, `Some`, `Vec::new`, `W::id`, `W::name`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `survives`
- **Macros:** `vec!`
- **Called by:** `accept_content` (revision_processor.rs:3481)

#### `removed_container`  (L240-254)
`fn removed_container(dom: &Dom, node: NodeId) -> Option<NodeId>`

- **Does:** The outermost container removed by the trailing accept pass.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L244; `has_path` (revision_processor.rs:113) L245,246; `table_rows_all_deleted` (revision_processor.rs:1941) L247; `Dom::parent` (xmllinq/mod.rs:597) L251
- **Calls (std/external/unresolved):** `Some`, `W::cell_del`, `W::del`, `W::tbl`, `W::tc_pr`, `W::tc`, `W::tr_pr`, `W::tr`
- **Called by:** `keep_wholly_deleted_comments` (revision_processor/comments.rs:96)

#### `remove_marker`  (L258-269)
`fn remove_marker(dom: &mut Dom, marker: NodeId)`

- **Does:** A reference-only run goes with its marker; a run containing other content keeps that content.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L259; `Dom::name` (xmllinq/mod.rs:526) L259,265; `Dom::remove` (xmllinq/mod.rs:1118) L260,267; `Dom::elements` (xmllinq/mod.rs:640) L262
- **Calls (std/external/unresolved):** `.all`, `.filter`, `.into_iter`, `Some`, `W::r_pr`, `W::r`
- **Called by:** `drop_wholly_deleted_comments` (revision_processor/comments.rs:71); `keep_wholly_deleted_comments` (revision_processor/comments.rs:96); `prune_orphan_comments` (revision_processor/comments.rs:274)

#### `prune_orphan_comments`  (L274-354, pub(crate))
`pub(crate) fn prune_orphan_comments(pkg: &mut PartFs, story_parts: &[String]) -> Vec<String>`

- **Does:** Drop the comments Accept / Reject All left without a reference, prune the comment family and `people.xml` (module docs).
- **Calls (crate):** `Parsed::load` (revision_processor/comments.rs:41) L275,285; `Parsed::w` (revision_processor/comments.rs:62) (by name) L291,304,327; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L293,305,308,314,328; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L313; `remove_marker` (revision_processor/comments.rs:258) L329; `collapse_cut_ranges` (revision_processor/comments.rs:361) L334; `.store` (ambiguous: `CommentFamily::store` (comments.rs:854); `Parsed::store` (revision_processor/comments.rs:54); `Part::store` (scrub.rs:203)) L336,349; `PartFs::main_document_part` (opc/mod.rs:1070) L340; `remove_part` (revision_processor/comments.rs:495) L345; `prune_by_para` (revision_processor/comments.rs:411) L350; `prune_people` (revision_processor/comments.rs:467) L352
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.contains`, `.filter_map`, `.filter`, `.flat_map`, `.insert`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.map`, `.push`, `.remove`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `HashSet::new`, `Some`, `Vec::new`, `W14::name`, `W::id`, `W::name`, `W::p`
- **Called by:** `resolve_with_comments` (revision_processor.rs:3871)

#### `collapse_cut_ranges`  (L361-401)
`fn collapse_cut_ranges(s: &mut Parsed) -> bool`

- **Does:** A comment whose reference stayed while resolved content took one end of its range (resolving some changes and keeping others can cut a range that way) gets the missing marker back where the range was cut short: the start just before the surviving end, the end just before the reference's run.
- **Calls (crate):** `Parsed::w` (revision_processor/comments.rs:62) L364,371,381; `Dom::attribute` (xmllinq/mod.rs:874) L366,372,383; `.parent` (ambiguous: `Marks::parent` (markdown/view_stories.rs:450); `Dom::parent` (xmllinq/mod.rs:597)) L375; `Dom::name` (xmllinq/mod.rs:526) L378; `marker` (revision_processor/comments.rs:403) L385,395; `Dom::add_before_self` (xmllinq/mod.rs:1130) L386,396
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.contains`, `.filter_map`, `.filter`, `.find`, `.into_iter`, `.map`, `.unwrap_or`, `Some`, `W::id`, `W::r`, `ids_of`
- **Called by:** `prune_orphan_comments` (revision_processor/comments.rs:274)

#### `marker`  (L403-407)
`fn marker(dom: &mut Dom, local: &str, cid: &str) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L404; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L405
- **Calls (std/external/unresolved):** `Some`, `W::id`, `W::name`
- **Called by:** `step_out_of_deletion_end` (comparer/comments.rs:898) (?); `mark_comment_anchors` (convert/mod.rs:7877) (?); `lift_out_of_deletion` (edit.rs:6776); `closing_comment_references` (edit.rs:7548); `collapse_cut_ranges` (revision_processor/comments.rs:361)

#### `prune_by_para`  (L411-464)
`fn prune_by_para(pkg: &mut PartFs, dropped_paras: &HashSet<String>)`

- **Does:** Drop the commentsExtended / commentsIds entries of the dropped comments' paragraphs and the commentsExtensible entries of their durable ids.
- **Calls (crate):** `Parsed::load` (revision_processor/comments.rs:41) L415,429,448; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L417,431,450; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L418,435,440,454; `.store` (ambiguous: `CommentFamily::store` (comments.rs:854); `Parsed::store` (revision_processor/comments.rs:54); `Part::store` (scrub.rs:203)) L426,446,462; `XName::get` (xmllinq/mod.rs:180) L430,433,449,452
- **Calls (std/external/unresolved):** `.contains`, `.insert`, `.is_empty`, `.is_some_and`, `.remove`, `.to_string`, `HashSet::new`, `Some`, `W15::name`
- **Called by:** `prune_orphan_comments` (revision_processor/comments.rs:274)

#### `prune_people`  (L467-492)
`fn prune_people(pkg: &mut PartFs, main: &str, authors: &HashSet<String>)`

- **Does:** Keep the people who authored a comment left; drop the part when none is.
- **Calls (crate):** `Parsed::load` (revision_processor/comments.rs:41) L468; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L473; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L477; `remove_part` (revision_processor/comments.rs:495) L488; `.store` (ambiguous: `CommentFamily::store` (comments.rs:854); `Parsed::store` (revision_processor/comments.rs:54); `Part::store` (scrub.rs:203)) L490
- **Calls (std/external/unresolved):** `.contains`, `.is_some_and`, `.remove`, `Some`, `W15::name`
- **Called by:** `prune_orphan_comments` (revision_processor/comments.rs:274)

#### `remove_part`  (L495-502)
`fn remove_part(pkg: &mut PartFs, main: &str, part: &str, rel_type: &str)`

- **Does:** Remove `part`, the main part's relationship to it and its content type.
- **Calls (crate):** `PartFs::part_bytes` (opc/mod.rs:628) L496; `PartFs::remove_part` (opc/mod.rs:710) L499; `PartFs::remove_relationships_by_type` (opc/mod.rs:904) L500; `PartFs::remove_content_type_override` (opc/mod.rs:897) L501
- **Calls (std/external/unresolved):** `.is_none`
- **Macros:** `format!`
- **Called by:** `prune_orphan_comments` (revision_processor/comments.rs:274); `prune_people` (revision_processor/comments.rs:467)
