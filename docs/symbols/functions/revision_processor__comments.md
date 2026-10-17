# Functions and methods in `src/revision_processor/comments.rs`

9 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `Parsed::load`  (L38-49, pub(super))
`pub(super) fn load(pkg: &PartFs, name: &str) -> Option<Self>`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:498) L39; `Dom::new` (xmllinq/mod.rs:387) L40; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L41; `Dom::root` (xmllinq/mod.rs:822) L42
- **Calls (std/external/unresolved):** `.to_string`, `Some`
- **Called by:** `renumber` (revision_processor/annotation_ids.rs:22); `prune_orphan_comments` (revision_processor/comments.rs:66); `prune_by_para` (revision_processor/comments.rs:195); `prune_people` (revision_processor/comments.rs:251); `prune_orphan_notes` (revision_processor/notes.rs:24); `tidy` (revision_processor/word_save.rs:25); `drop_unreferenced_headers_footers` (revision_processor/word_save.rs:59); `non_paragraph_styles` (revision_processor/word_save.rs:88)

#### `Parsed::store`  (L51-56, pub(super))
`pub(super) fn store(self, pkg: &mut PartFs)`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::set_part` (opc/mod.rs:510) L52; `Dom::serialize_document` (xmllinq/mod.rs:1322) L54
- **Calls (std/external/unresolved):** `.into_bytes`
- **Called by:** `append_documents` (append.rs:152) (?); `Transaction::write_comments_part` (edit.rs:5190) (?); `threads` (markdown/package.rs:73) (?); `active::reset` (perf.rs:240) (?); `active::set_atom_count` (perf.rs:277) (?); `active::set_unit_count` (perf.rs:281) (?); `renumber` (revision_processor/annotation_ids.rs:22) (?); `prune_orphan_comments` (revision_processor/comments.rs:66) (?); `prune_by_para` (revision_processor/comments.rs:195) (?); `prune_people` (revision_processor/comments.rs:251) (?); `prune_orphan_notes` (revision_processor/notes.rs:24) (?); `tidy` (revision_processor/word_save.rs:25) (?); +4 more

#### `Parsed::w`  (L59-61, pub(super))
`pub(super) fn w(&self, local: &str) -> Vec<NodeId>`

- **Does:** Descendants named `local` in the W namespace.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L60
- **Calls (std/external/unresolved):** `Some`, `W::name`
- **Called by:** `renumber` (revision_processor/annotation_ids.rs:22) (?); `prune_orphan_comments` (revision_processor/comments.rs:66) (?); `collapse_cut_ranges` (revision_processor/comments.rs:145); `prune_orphan_notes` (revision_processor/notes.rs:24) (?); `tidy` (revision_processor/word_save.rs:25) (?); `drop_unreferenced_headers_footers` (revision_processor/word_save.rs:59) (?); `non_paragraph_styles` (revision_processor/word_save.rs:88) (?)

#### `prune_orphan_comments`  (L66-138, pub(crate))
`pub(crate) fn prune_orphan_comments(pkg: &mut PartFs, story_parts: &[String])`

- **Does:** Drop the comments Accept / Reject All left without a reference, prune the comment family and `people.xml` (module docs).
- **Calls (crate):** `Parsed::load` (revision_processor/comments.rs:38) L67,72; `Parsed::w` (revision_processor/comments.rs:59) (by name) L78,90,112; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L80,91,94,99,113; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L98; `collapse_cut_ranges` (revision_processor/comments.rs:145) L119; `.store` (ambiguous: `CommentFamily::store` (comments.rs:798); `Parsed::store` (revision_processor/comments.rs:51); `Part::store` (scrub.rs:202)) L121,134; `PartFs::main_document_part` (opc/mod.rs:829) L125; `remove_part` (revision_processor/comments.rs:279) L130; `prune_by_para` (revision_processor/comments.rs:195) L135; `prune_people` (revision_processor/comments.rs:251) L137
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter_map`, `.flat_map`, `.insert`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.map`, `.remove`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `HashSet::new`, `Some`, `W14::name`, `W::id`, `W::name`, `W::p`
- **Called by:** `resolve` (revision_processor.rs:3503)

#### `collapse_cut_ranges`  (L145-185)
`fn collapse_cut_ranges(s: &mut Parsed) -> bool`

- **Does:** A comment whose reference stayed while resolved content took one end of its range (resolving some changes and keeping others can cut a range that way) gets the missing marker back where the range was cut short: the start just before the surviving end, the end just before the reference's run.
- **Calls (crate):** `Parsed::w` (revision_processor/comments.rs:59) L148,155,165; `Dom::attribute` (xmllinq/mod.rs:874) L150,156,167; `Dom::parent` (xmllinq/mod.rs:597) (by name) L159; `Dom::name` (xmllinq/mod.rs:526) L162; `marker` (revision_processor/comments.rs:187) L169,179; `Dom::add_before_self` (xmllinq/mod.rs:1114) L170,180
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.contains`, `.filter_map`, `.filter`, `.find`, `.into_iter`, `.map`, `.unwrap_or`, `Some`, `W::id`, `W::r`, `ids_of`
- **Called by:** `prune_orphan_comments` (revision_processor/comments.rs:66)

#### `marker`  (L187-191)
`fn marker(dom: &mut Dom, local: &str, cid: &str) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L188; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L189
- **Calls (std/external/unresolved):** `Some`, `W::id`, `W::name`
- **Called by:** `step_out_of_deletion_end` (comparer/comments.rs:898) (?); `mark_comment_anchors` (convert/mod.rs:7876) (?); `collapse_cut_ranges` (revision_processor/comments.rs:145)

#### `prune_by_para`  (L195-248)
`fn prune_by_para(pkg: &mut PartFs, dropped_paras: &HashSet<String>)`

- **Does:** Drop the commentsExtended / commentsIds entries of the dropped comments' paragraphs and the commentsExtensible entries of their durable ids.
- **Calls (crate):** `Parsed::load` (revision_processor/comments.rs:38) L199,213,232; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L201,215,234; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L202,219,224,238; `.store` (ambiguous: `CommentFamily::store` (comments.rs:798); `Parsed::store` (revision_processor/comments.rs:51); `Part::store` (scrub.rs:202)) L210,230,246; `XName::get` (xmllinq/mod.rs:180) L214,217,233,236
- **Calls (std/external/unresolved):** `.contains`, `.insert`, `.is_empty`, `.is_some_and`, `.remove`, `.to_string`, `HashSet::new`, `Some`, `W15::name`
- **Called by:** `prune_orphan_comments` (revision_processor/comments.rs:66)

#### `prune_people`  (L251-276)
`fn prune_people(pkg: &mut PartFs, main: &str, authors: &HashSet<String>)`

- **Does:** Keep the people who authored a comment left; drop the part when none is.
- **Calls (crate):** `Parsed::load` (revision_processor/comments.rs:38) L252; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L257; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L261; `remove_part` (revision_processor/comments.rs:279) L272; `.store` (ambiguous: `CommentFamily::store` (comments.rs:798); `Parsed::store` (revision_processor/comments.rs:51); `Part::store` (scrub.rs:202)) L274
- **Calls (std/external/unresolved):** `.contains`, `.is_some_and`, `.remove`, `Some`, `W15::name`
- **Called by:** `prune_orphan_comments` (revision_processor/comments.rs:66)

#### `remove_part`  (L279-286)
`fn remove_part(pkg: &mut PartFs, main: &str, part: &str, rel_type: &str)`

- **Does:** Remove `part`, the main part's relationship to it and its content type.
- **Calls (crate):** `PartFs::part_bytes` (opc/mod.rs:493) L280; `PartFs::remove_part` (opc/mod.rs:527) L283; `PartFs::remove_relationships_by_type` (opc/mod.rs:675) L284; `PartFs::remove_content_type_override` (opc/mod.rs:668) L285
- **Calls (std/external/unresolved):** `.is_none`
- **Macros:** `format!`
- **Called by:** `prune_orphan_comments` (revision_processor/comments.rs:66); `prune_people` (revision_processor/comments.rs:251)
