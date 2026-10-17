# Functions and methods in `src/append.rs`

37 non-test, 13 test. Part of the [symbol manifest](../README.md).

#### `AppendError::code`  (L115-121, pub)
`pub fn code(&self) -> &'static str`

- **Does:** Stable machine code: the admission code, `INVALID_PACKAGE` or `INVALID_DOCUMENT`.
- **Calls (crate):** `.code` (ambiguous, 4 candidates) L117
- **Called by:** none found in src

#### `<AppendError as fmt::Display>::fmt`  (L125-132, impl fmt::Display)
`fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`

- **Does:** _no doc comment_
- **Calls (crate):** `.code` (ambiguous, 4 candidates) L129
- **Macros:** `write!`
- **Called by:** none found in src

#### `append_documents`  (L152-300, pub)
`pub fn append_documents( a: &[u8], b: &[u8], options: &AppendOptions, ) -> Result<Appended, AppendError>`

- **Does:** Append B's body after A's.
- **Calls (crate):** `open` (append.rs:316) L157,158; `PartFs::parts` (opc/mod.rs:532) (by name) L160; `main_part` (append.rs:323) L161,162; `Dom::new` (xmllinq/mod.rs:387) L163; `parse` (append.rs:330) L164,165; `body` (append.rs:349) L166,167; `final_sect_pr` (append.rs:354) L168,169; `merge_namespace_declarations` (append.rs:365) L170; `Dom::new_element` (xmllinq/mod.rs:478) L174; `Dom::elements` (xmllinq/mod.rs:640) L175,242,265; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L177,184; `Dom::add` (xmllinq/mod.rs:1058) L178,243; `drop_comments` (append.rs:405) L190,193,207; `offset_annotation_ids` (append.rs:495) L195,198; `drop_colliding_paragraph_ids` (append.rs:523) L200; `carry_notes` (append.rs:667) L205; `CommentFamily::load` (comments.rs:429) L217; `invalid` (append.rs:309) L217; `stage_comments` (append.rs:557) L219; `carry_styles_and_numbering` (append.rs:861) L227; `carry_part_relationships` (comparer/parts.rs:576) L229,231,234,251; `PartFs::set_part` (opc/mod.rs:510) (by name) L245,263; `Dom::serialize_document` (xmllinq/mod.rs:1322) L247,263; `CommentFamily::part_name` (comments.rs:664) (by name) L253; `drop_comments_in_new_parts` (append.rs:450) L260; `join` (append.rs:1264) L262; `Dom::attribute` (xmllinq/mod.rs:874) L266; `CommentFamily::adopt` (comments.rs:687) (by name) L273; `Dom::serialize_element` (xmllinq/mod.rs:1299) L274; `.store` (ambiguous: `CommentFamily::store` (comments.rs:798); `Parsed::store` (revision_processor/comments.rs:51); `Part::store` (scrub.rs:202)) L281; `related` (append.rs:760) L282; `merge_part_namespaces` (append.rs:1081) L283; `PartFs::to_zip` (opc/mod.rs:548) (by name) L296; `package` (append.rs:302) L296,298; `fix_up_drawing_ids_in_package` (comparer/fixups.rs:121) L297
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.cloned`, `.collect`, `.extend`, `.flatten`, `.get`, `.into_bytes`, `.into_iter`, `.iter`, `.map_err`, `.map`, `.ok`, `.parse`, `.push`, `.then_some`, `.unwrap_or_default`, `Ok`, `Some`, `Vec::new`, `W::body`, `W::id`, `W::name`
- **Macros:** `format!`, `vec!`
- **Called by:** `public_memory_append_owner_boundary_tests::public_append_keeps_source_property_history_and_fresh_annotation_owners_at_every_join_boundary` (append.rs:1511); `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1594); `run_append` (bin/jubarte.rs:3357)

#### `package`  (L302-307)
`fn package(document: &'static str, error: &OpcError) -> AppendError`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `append_documents` (append.rs:152); `open` (append.rs:316); `public_memory_append_owner_boundary_tests::public_append_keeps_source_property_history_and_fresh_annotation_owners_at_every_join_boundary` (append.rs:1511); `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1594)

#### `invalid`  (L309-314)
`fn invalid(document: &'static str, message: impl Into<String>) -> AppendError`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`
- **Called by:** `declared_entry_count` (admission.rs:470) (?); `check_xml_depth` (admission.rs:678) (?); `append_documents` (append.rs:152); `main_part` (append.rs:323); `parse` (append.rs:330); `body` (append.rs:349); `stage_comments` (append.rs:557); `tests::error_codes_and_messages_name_the_document` (append.rs:1381); `picture` (edit/images.rs:86) (?); `load_stories` (edit.rs:1311) (?); `apply_markdown` (markdown/patch.rs:121) (?)

#### `open`  (L316-321)
`fn open(document: &'static str, bytes: &[u8]) -> Result<PartFs, AppendError>`

- **Does:** _no doc comment_
- **Calls (crate):** `admit` (admission.rs:310) L317; `<InputLimits as Default>::default` (admission.rs:56) L317; `strict_to_transitional_docx` (strict_translation.rs:743) L319; `PartFs::open` (opc/mod.rs:456) L320; `package` (append.rs:302) L320
- **Calls (std/external/unresolved):** `.map_err`
- **Called by:** `append_documents` (append.rs:152); `tests::error_codes_and_messages_name_the_document` (append.rs:1381); `Origin::build` (edit/origin.rs:256) (?); `record_mid_changes` (edit/sections.rs:453) (?); `rewrite` (edit/whole.rs:130) (?); `collect_fields` (fields.rs:281) (?)

#### `main_part`  (L323-327)
`fn main_part(document: &'static str, pkg: &PartFs) -> Result<String, AppendError>`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:829) L324; `PartFs::part_bytes` (opc/mod.rs:493) L325; `invalid` (append.rs:309) L326
- **Calls (std/external/unresolved):** `.filter`, `.is_some`, `.ok_or_else`
- **Called by:** `append_documents` (append.rs:152)

#### `parse`  (L330-347)
`fn parse( document: &'static str, dom: &mut Dom, pkg: &PartFs, part: &str, ) -> Result<(NodeId, NodeId), AppendError>`

- **Does:** Parse `part` of `pkg` into `dom`: `(document node, root element)`.
- **Calls (crate):** `PartFs::part_bytes` (opc/mod.rs:493) L336; `invalid` (append.rs:309) L338,339,341,345; `validate_xml` (xmllinq/parse.rs:792) L340; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L342; `Dom::root` (xmllinq/mod.rs:822) L343
- **Calls (std/external/unresolved):** `.map_err`, `.ok_or_else`, `Ok`, `std::str::from_utf8`
- **Macros:** `format!`
- **Called by:** `append_documents` (append.rs:152); `stage_comments` (append.rs:557); `carry_notes` (append.rs:667); `carry_styles_and_numbering` (append.rs:861); `merge_part_namespaces` (append.rs:1081)

#### `body`  (L349-352)
`fn body(document: &'static str, dom: &Dom, root: NodeId) -> Result<NodeId, AppendError>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L350; `invalid` (append.rs:309) L351
- **Calls (std/external/unresolved):** `.ok_or_else`, `W::body`
- **Called by:** `append_documents` (append.rs:152)

#### `final_sect_pr`  (L354-359)
`fn final_sect_pr(dom: &Dom, body: NodeId) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L355; `Dom::name_is` (xmllinq/mod.rs:535) L358
- **Calls (std/external/unresolved):** `.copied`, `.filter`, `.last`, `W::sect_pr`
- **Called by:** `append_documents` (append.rs:152)

#### `merge_namespace_declarations`  (L365-400)
`fn merge_namespace_declarations(dom: &mut Dom, to: NodeId, from: NodeId)`

- **Does:** Declare on `to`'s root the prefixes `from`'s root declares and `to` lacks, and add `from`'s ignorable prefixes to `to`'s `mc:Ignorable`, so copied markup keeps the prefixes its `mc:Choice/@Requires` and `mc:Ignorable` lists name.
- **Calls (crate):** `Dom::attributes` (xmllinq/mod.rs:885) L366,372; `Dom::is_namespace_declaration` (xmllinq/mod.rs:996) L369,373; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L374,398; `Dom::attribute` (xmllinq/mod.rs:874) L378,381,390,392; `XName::get` (xmllinq/mod.rs:180) L389
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.contains`, `.filter`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.join`, `.map`, `.push`, `.split_whitespace`, `.to_string`, `.unwrap_or`, `MC::name`, `Some`
- **Called by:** `append_documents` (append.rs:152); `carry_notes` (append.rs:667); `carry_styles_and_numbering` (append.rs:861); `merge_part_namespaces` (append.rs:1081); `tests::ignorable_prefixes_merge_only_with_the_same_namespace` (append.rs:1417)

#### `drop_comments`  (L405-407)
`fn drop_comments(dom: &mut Dom, root: NodeId) -> usize`

- **Does:** Remove comment anchors and references under `root` (a run left holding only its properties goes too); returns how many distinct comments they named.
- **Calls (crate):** `drop_comments_where` (append.rs:426) L406
- **Called by:** `append_documents` (append.rs:152); `drop_comments_in_new_parts` (append.rs:450)

#### `comment_anchors`  (L410-415)
`fn comment_anchors(dom: &Dom, root: NodeId) -> Vec<NodeId>`

- **Does:** The comment anchors and references under `root`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L411; `Dom::name` (xmllinq/mod.rs:526) L413; `is_comment_anchor` (append.rs:417) L413
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_some_and`
- **Called by:** `drop_comments_where` (append.rs:426); `stage_comments` (append.rs:557)

#### `is_comment_anchor`  (L417-423)
`fn is_comment_anchor(name: &XName) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L418; `XName::local_name` (xmllinq/mod.rs:208) (by name) L420
- **Macros:** `matches!`
- **Called by:** `comment_anchors` (append.rs:410)

#### `drop_comments_where`  (L426-446)
`fn drop_comments_where(dom: &mut Dom, root: NodeId, drop: impl Fn(&str) -> bool) -> usize`

- **Does:** [`drop_comments`] for the anchors whose id `drop` accepts.
- **Calls (crate):** `comment_anchors` (append.rs:410) L428; `Dom::attribute` (xmllinq/mod.rs:874) L429; `Dom::parent` (xmllinq/mod.rs:597) L434; `Dom::remove` (xmllinq/mod.rs:1102) L435,442; `Dom::name_is` (xmllinq/mod.rs:535) L436,440; `Dom::elements` (xmllinq/mod.rs:640) L437
- **Calls (std/external/unresolved):** `.all`, `.filter`, `.insert`, `.iter`, `.len`, `.to_string`, `.unwrap_or`, `HashSet::new`, `W::id`, `W::r_pr`, `W::r`, `drop`
- **Called by:** `drop_comments` (append.rs:405); `stage_comments` (append.rs:557)

#### `drop_comments_in_new_parts`  (L450-473)
`fn drop_comments_in_new_parts(dest: &mut PartFs, before: &HashSet<String>) -> usize`

- **Does:** Comments in header and footer parts copied by this append (parts not in `before`): their anchors name comments A does not have.
- **Calls (crate):** `PartFs::parts` (opc/mod.rs:532) L452; `PartFs::part_string` (opc/mod.rs:498) L456; `Dom::new` (xmllinq/mod.rs:387) L462; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L463; `Dom::root` (xmllinq/mod.rs:822) L464; `drop_comments` (append.rs:405) L465; `PartFs::set_part` (opc/mod.rs:510) L468; `Dom::serialize_document` (xmllinq/mod.rs:1322) L468
- **Calls (std/external/unresolved):** `.contains`, `.ends_with`, `.into_bytes`
- **Called by:** `append_documents` (append.rs:152)

#### `is_annotation`  (L488-490)
`fn is_annotation(name: &XName) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L489; `XName::local_name` (xmllinq/mod.rs:208) L489
- **Calls (std/external/unresolved):** `.contains`
- **Called by:** `offset_annotation_ids` (append.rs:495)

#### `offset_annotation_ids`  (L495-519)
`fn offset_annotation_ids(dom: &mut Dom, existing: NodeId, staged: NodeId)`

- **Does:** Shift every numeric annotation id under `staged` (revisions, bookmarks, move ranges, permissions) past the largest one under `existing`: ids are unique per story, and a start keeps its end's id.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L496,507; `Dom::name` (xmllinq/mod.rs:526) L499,508; `is_annotation` (append.rs:488) L499,508; `Dom::attribute` (xmllinq/mod.rs:874) L500,511; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L516
- **Calls (std/external/unresolved):** `.and_then`, `.filter_map`, `.filter`, `.into_iter`, `.is_some_and`, `.max`, `.ok`, `.parse`, `.to_string`, `.unwrap_or`, `Some`, `W::id`
- **Called by:** `append_documents` (append.rs:152); `carry_notes` (append.rs:667)

#### `drop_colliding_paragraph_ids`  (L523-539)
`fn drop_colliding_paragraph_ids(dom: &mut Dom, existing: NodeId, staged: NodeId)`

- **Does:** Drop `w14:paraId`/`w14:textId` from B's paragraphs whose paragraph id A already uses; Word assigns new ones.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L525,530; `Dom::attribute` (xmllinq/mod.rs:874) L528,531; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L535,536
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter_map`, `.into_iter`, `.is_some_and`, `.map`, `Some`, `W14::name`, `W::p`
- **Called by:** `append_documents` (append.rs:152)

#### `stage_comments`  (L557-630)
`fn stage_comments( src: &PartFs, b_main: &str, dom: &mut Dom, family: &CommentFamily, a_root: NodeId, staged: NodeId, ) -> Result<(Option<StagedComments>, usize), AppendError>`

- **Does:** Give the comments anchored under `staged` ids past every comment id of A (`family` and the anchors under `a_root`), rewrite the anchors, and stage copies of those comments.
- **Calls (crate):** `CommentFamily::load` (comments.rs:429) L565; `invalid` (append.rs:309) L565,599; `drop_comments_where` (append.rs:426) L567; `comment_anchors` (append.rs:410) L568,576; `Dom::attribute` (xmllinq/mod.rs:874) L571,578,590,603; `CommentFamily::order` (comments.rs:534) L579; `CommentFamily::order` (comments.rs:534) (by name) L583; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L595,611; `related` (append.rs:760) L599; `parse` (append.rs:330) L600; `Dom::new_element` (xmllinq/mod.rs:478) L601; `Dom::elements` (xmllinq/mod.rs:640) L602; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L610; `Dom::add` (xmllinq/mod.rs:1058) L612; `CommentFamily::thread_state` (comments.rs:672) (by name) L617
- **Calls (std/external/unresolved):** `.and_then`, `.chain`, `.collect`, `.contains`, `.copied`, `.filter_map`, `.get`, `.insert`, `.into_iter`, `.is_empty`, `.is_ok_and`, `.iter`, `.map_err`, `.map_or`, `.max`, `.ok_or_else`, `.ok`, `.parse`, `.to_string`, `HashMap::new`, `Ok`, `Some`, `W::id`, `W::name`, `defined`
- **Called by:** `append_documents` (append.rs:152)

#### `carry_notes`  (L667-756)
`fn carry_notes( kind: &NoteKind, dest: &mut PartFs, a_main: &str, src: &PartFs, b_main: &str, dom: &mut Dom, staged: NodeId, ) -> Result<Option<CarriedNotes>, AppendError>`

- **Does:** Copy the notes B's staged content references into A's notes part (creating it, with separators, when A has none) under ids past A's, and rewrite the references.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L677; `related` (append.rs:760) L682,692; `parse` (append.rs:330) L683,712; `Dom::remove` (xmllinq/mod.rs:1102) L688,745; `free_sibling` (append.rs:771) L695; `add_related_part` (append.rs:790) L708; `merge_namespace_declarations` (append.rs:365) L713; `Dom::elements` (xmllinq/mod.rs:640) L714,722; `Dom::attribute` (xmllinq/mod.rs:874) L717,725,730; `Dom::new_element` (xmllinq/mod.rs:478) L727; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L734; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L737,744; `Dom::add` (xmllinq/mod.rs:1058) L738; `offset_annotation_ids` (append.rs:495) L748
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.filter_map`, `.get`, `.insert`, `.into_iter`, `.is_empty`, `.map`, `.max`, `.ok`, `.parse`, `.to_string`, `.unwrap_or`, `HashMap::new`, `Ok`, `Some`, `W::id`, `W::name`
- **Macros:** `format!`
- **Called by:** `append_documents` (append.rs:152)

#### `related`  (L760-768)
`fn related(pkg: &PartFs, main: &str, kind: &str) -> Option<String>`

- **Does:** The internal part `main` relates to with a relationship type ending in `kind`, when the part exists.
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:756) L761; `PartFs::resolve_rel_target` (opc/mod.rs:600) L766; `PartFs::part_bytes` (opc/mod.rs:493) L767
- **Calls (std/external/unresolved):** `.as_deref`, `.ends_with`, `.find`, `.is_some`, `.iter`, `.then_some`, `Some`
- **Called by:** `append_documents` (append.rs:152); `stage_comments` (append.rs:557); `carry_notes` (append.rs:667); `carry_styles_and_numbering` (append.rs:861)

#### `free_sibling`  (L771-788)
`fn free_sibling(pkg: &PartFs, main: &str, name: &str) -> String`

- **Does:** `name` beside `main`, or `name` with a number when that is taken.
- **Calls (crate):** `path` (debug.rs:324) (by name) L781,784; `PartFs::part_bytes` (opc/mod.rs:493) L783
- **Calls (std/external/unresolved):** `.is_empty`, `.is_some`, `.map_or`, `.rsplit_once`, `.to_string`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `carry_notes` (append.rs:667); `carry_styles_and_numbering` (append.rs:861); `tests::free_sibling_numbers_a_taken_name` (append.rs:1366)

#### `add_related_part`  (L790-805)
`fn add_related_part( pkg: &mut PartFs, main: &str, part: &str, rel: &str, content_type: &str, xml: String, )`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::add_document_relationship` (opc/mod.rs:772) L798; `relative_rel_target` (opc/mod.rs:129) L801; `PartFs::add_content_type_override` (opc/mod.rs:616) L803; `PartFs::set_part` (opc/mod.rs:510) L804
- **Calls (std/external/unresolved):** `.into_bytes`
- **Macros:** `format!`
- **Called by:** `carry_notes` (append.rs:667); `carry_styles_and_numbering` (append.rs:861)

#### `w_vals`  (L812-821)
`fn w_vals(dom: &Dom, root: NodeId, locals: &[&str]) -> Vec<(NodeId, String)>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L813; `Dom::name` (xmllinq/mod.rs:526) L816; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L817; `XName::local_name` (xmllinq/mod.rs:208) (by name) L817; `Dom::attribute` (xmllinq/mod.rs:874) L819
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter_map`, `.filter`, `.into_iter`, `.is_some_and`, `.to_string`, `Some`, `W::val`
- **Called by:** `carry_styles_and_numbering` (append.rs:861); `ListCopy::build` (append.rs:1111)

#### `style_key`  (L825-840)
`fn style_key(dom: &Dom, style: NodeId) -> Option<(String, String)>`

- **Does:** Word's pairing key for a style: type and name (the id when unnamed), built-in names folded to lower case.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L826,832,833; `Dom::element` (xmllinq/mod.rs:655) L830; `is_built_in` (builtin_styles.rs:394) L834
- **Calls (std/external/unresolved):** `.and_then`, `.or_else`, `.to_ascii_lowercase`, `.to_string`, `.unwrap_or`, `Some`, `W::name`, `W::val`
- **Called by:** `carry_styles_and_numbering` (append.rs:861)

#### `index_by_attribute`  (L842-855)
`fn index_by_attribute( dom: &Dom, root: Option<NodeId>, element: &str, attribute: &str, ) -> HashMap<String, NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L849; `Dom::attribute` (xmllinq/mod.rs:874) L851
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.into_iter`, `.map`, `.to_string`, `.unwrap_or_default`, `Some`, `W::name`
- **Called by:** `carry_styles_and_numbering` (append.rs:861)

#### `carry_styles_and_numbering`  (L861-1076)
`fn carry_styles_and_numbering( dest: &mut PartFs, a_main: &str, src: &PartFs, b_main: &str, dom: &mut Dom, roots: &[NodeId], ) -> Result<(), AppendError>`

- **Does:** Carry the styles and lists the staged `roots` use: a B style pairs with A's style of the same type and name, else is copied (renamed `{id}B` when A uses its id) with the styles it is based on, links to and is followed by; each B list is copied with ids past A's.
- **Calls (crate):** `related` (append.rs:760) L869,873,877,1014,1035,1047...; `parse` (append.rs:330) L870,874,880,979; `index_by_attribute` (append.rs:842) L882,883,888,889; `style_key` (append.rs:825) L886,918; `w_vals` (append.rs:812) L899,906,935,936,949,991...; `abstract_of` (append.rs:1220) L947,1020; `free_sibling` (append.rs:771) L966; `add_related_part` (append.rs:790) L967; `merge_namespace_declarations` (append.rs:365) L984; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L987; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L988,990,993,1006,1068; `Dom::add` (xmllinq/mod.rs:1058) L996; `PartFs::part_string` (opc/mod.rs:498) L1015; `nsids` (append.rs:1226) L1016; `PartFs::part_bytes` (opc/mod.rs:493) L1036,1048; `append_numbering` (markdown/package.rs:984) L1038; `ListCopy::build` (append.rs:1111) L1042; `insert_lists` (append.rs:1159) L1054; `merge_part_namespaces` (append.rs:1081) L1060; `PartFs::set_part` (opc/mod.rs:510) L1073; `Dom::serialize_document` (xmllinq/mod.rs:1322) L1073
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.clone`, `.cloned`, `.collect`, `.contains_key`, `.contains`, `.copied`, `.extend`, `.filter_map`, `.filter`, `.get`, `.insert`, `.into_bytes`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.keys`, `.map`, `.pop`, `.push`, `.to_vec`, `.transpose`, `.unwrap_or_default`, `HashMap::new`, `Ok`, `Some`, `Vec::new`, `W::name`, `W::val`
- **Macros:** `format!`
- **Called by:** `append_documents` (append.rs:152)

#### `merge_part_namespaces`  (L1081-1094)
`fn merge_part_namespaces( dest: &mut PartFs, part: &str, dom: &mut Dom, from: NodeId, ) -> Result<(), AppendError>`

- **Does:** Merge `from`'s namespace declarations and ignorable prefixes onto the root of `dest`'s `part`, which received markup copied from under `from`; the part is rewritten only when its root changed.
- **Calls (crate):** `parse` (append.rs:330) L1087; `Dom::attributes` (xmllinq/mod.rs:885) L1088,1090; `merge_namespace_declarations` (append.rs:365) L1089; `PartFs::set_part` (opc/mod.rs:510) L1091; `Dom::serialize_document` (xmllinq/mod.rs:1322) L1091
- **Calls (std/external/unresolved):** `.into_bytes`, `Ok`
- **Called by:** `append_documents` (append.rs:152); `carry_styles_and_numbering` (append.rs:861)

#### `ListCopy::build`  (L1111-1154)
`fn build( &self, dom: &mut Dom, first_abstract: u32, first_num: u32, ) -> (String, String, HashMap<String, String>)`

- **Does:** The copies' markup numbered from `first_abstract` and `first_num`: `(abstracts, nums, B num id -> new num id)`.
- **Calls (crate):** `Dom::clone_subtree` (xmllinq/mod.rs:1215) L1123,1142; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1124,1132,1143,1149; `unique_nsid` (append.rs:1241) L1125; `Dom::descendants` (xmllinq/mod.rs:668) L1127; `Dom::remove` (xmllinq/mod.rs:1102) L1128; `w_vals` (append.rs:812) L1130; `Dom::serialize_element` (xmllinq/mod.rs:1299) L1135,1151; `Dom::elements` (xmllinq/mod.rs:640) L1144; `Dom::attribute` (xmllinq/mod.rs:874) L1145
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.clone`, `.cloned`, `.enumerate`, `.get`, `.insert`, `.iter`, `.push_str`, `.to_string`, `.unwrap_or`, `HashMap::new`, `Some`, `String::new`, `W::name`, `W::val`, `u32::try_from`
- **Called by:** `carry_styles_and_numbering` (append.rs:861); `insert_lists` (append.rs:1159); `run` (update.rs:186) (?)

#### `insert_lists`  (L1159-1199)
`fn insert_lists( dest: &mut PartFs, part: &str, dom: &mut Dom, lists: &ListCopy<'_>, ) -> Option<HashMap<String, String>>`

- **Does:** Add `lists` to the numbering `part` through the DOM, numbered past the part's own ids whatever its prefix, in schema order; returns the num map.
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:498) L1165; `Dom::new` (xmllinq/mod.rs:387) L1166; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L1167,1188; `Dom::root` (xmllinq/mod.rs:822) L1168,1189; `Dom::elements` (xmllinq/mod.rs:640) L1170,1192; `Dom::attribute` (xmllinq/mod.rs:874) L1174; `ListCopy::build` (append.rs:1111) L1186; `Dom::remove` (xmllinq/mod.rs:1102) L1193; `add_in_numbering_order` (append.rs:1202) L1194; `PartFs::set_part` (opc/mod.rs:510) L1197; `Dom::serialize_document` (xmllinq/mod.rs:1322) L1197
- **Calls (std/external/unresolved):** `.filter_map`, `.into_bytes`, `.into_iter`, `.max`, `.ok`, `.parse`, `.unwrap_or`, `Some`, `W::name`, `next`, `u32::try_from`
- **Macros:** `format!`
- **Called by:** `carry_styles_and_numbering` (append.rs:861)

#### `add_in_numbering_order`  (L1202-1218)
`fn add_in_numbering_order(dom: &mut Dom, numbering: NodeId, child: NodeId)`

- **Does:** `w:numbering` lists picture bullets, then abstracts, then nums.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L1203; `rank` (ambiguous, 4 candidates) L1209,1213; `Dom::elements` (xmllinq/mod.rs:640) L1210; `Dom::add_before_self` (xmllinq/mod.rs:1114) L1215; `Dom::add` (xmllinq/mod.rs:1058) L1216
- **Calls (std/external/unresolved):** `.as_ref`, `.find`, `.into_iter`, `.map`
- **Called by:** `insert_lists` (append.rs:1159)

#### `abstract_of`  (L1220-1223)
`fn abstract_of(dom: &Dom, num: NodeId) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1221; `Dom::attribute` (xmllinq/mod.rs:874) L1222
- **Calls (std/external/unresolved):** `.map`, `W::name`, `W::val`
- **Called by:** `carry_styles_and_numbering` (append.rs:861)

#### `nsids`  (L1226-1237)
`fn nsids(xml: &str) -> HashSet<String>`

- **Does:** The `w:nsid` values of a numbering part.
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L1227; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L1228; `Dom::root` (xmllinq/mod.rs:822) L1229; `Dom::descendants` (xmllinq/mod.rs:668) L1231; `Dom::attribute` (xmllinq/mod.rs:874) L1233
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.into_iter`, `.map`, `.unwrap_or_default`, `Some`, `W::name`, `W::val`
- **Called by:** `carry_styles_and_numbering` (append.rs:861)

#### `unique_nsid`  (L1241-1261)
`fn unique_nsid(dom: &mut Dom, abstract_num: NodeId, used: &mut HashSet<String>)`

- **Does:** Give `abstract_num` an `w:nsid` no list in `used` has: Word treats lists that share one as the same list.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1242; `Dom::attribute` (xmllinq/mod.rs:874) L1245; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1259
- **Calls (std/external/unresolved):** `.contains`, `.insert`, `.to_ascii_uppercase`, `.unwrap_or`, `.wrapping_add`, `Some`, `W::name`, `W::val`, `u32::from_str_radix`
- **Macros:** `format!`
- **Called by:** `ListCopy::build` (append.rs:1111); `tests::nsid_collisions_take_the_next_free_value` (append.rs:1351)

#### `join`  (L1264-1313)
`fn join( dom: &mut Dom, a_body: NodeId, a_sect: Option<NodeId>, staged: NodeId, b_final_sect: Option<NodeId>, options: &AppendOptions, )`

- **Does:** Put B's staged content after A's, before A's final section properties.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L1277,1278,1281,1289,1290,1291; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L1280; `Dom::add` (xmllinq/mod.rs:1058) L1283,1284,1293,1294,1304,1310; `set_section_type` (append.rs:1325) L1286; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1292; `Dom::elements` (xmllinq/mod.rs:640) L1299; `Dom::remove` (xmllinq/mod.rs:1102) L1301; `Dom::add_before_self` (xmllinq/mod.rs:1114) L1303; `Dom::replace_with` (xmllinq/mod.rs:1149) L1309
- **Calls (std/external/unresolved):** `.extend`, `.push`, `Some`, `Vec::new`, `W::name`, `W::p_pr`, `W::p`, `W::r`, `W::sect_pr`
- **Called by:** `append_documents` (append.rs:152); `tests::splice_marks_a_tracked_break_and_extends_adjacent_spans` (markdown/from_docx/critic.rs:1962) (?); `tests::a_span_after_a_code_fence_opens_on_the_next_block_s_line` (markdown/from_docx/critic.rs:1992) (?); `tests::a_plain_span_across_paragraphs_names_every_piece_s_id` (markdown/from_docx/critic.rs:2043) (?); `tests::a_tracked_break_extends_spans_only_by_the_same_person` (markdown/from_docx/critic.rs:2299) (?)

#### `set_section_type`  (L1325-1343)
`fn set_section_type(dom: &mut Dom, sect: NodeId, section_break: SectionBreak)`

- **Does:** How B's section starts: on a new page (Word's default, no `w:type`) or continuously.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1327,1335; `Dom::remove` (xmllinq/mod.rs:1102) L1328; `Dom::new_element` (xmllinq/mod.rs:478) L1333; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1334; `Dom::name` (xmllinq/mod.rs:526) L1336; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1337; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1337; `Dom::add_before_self` (xmllinq/mod.rs:1114) L1340; `Dom::add` (xmllinq/mod.rs:1058) L1341
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.into_iter`, `.is_some_and`, `Some`, `W::name`, `W::val`
- **Called by:** `join` (append.rs:1264); `tests::section_type_sits_after_header_references` (append.rs:1398)

## Test functions (13)

- `tests::nsid_collisions_take_the_next_free_value` (L1351): _no doc comment_
- `tests::free_sibling_numbers_a_taken_name` (L1366): _no doc comment_
- `tests::error_codes_and_messages_name_the_document` (L1381): _no doc comment_
- `tests::section_type_sits_after_header_references` (L1398): _no doc comment_
- `tests::ignorable_prefixes_merge_only_with_the_same_namespace` (L1417): _no doc comment_
- `tests::minimal` (L1437): _no doc comment_
- `public_memory_append_owner_boundary_tests::semantic` (L1448): _no doc comment_
- `public_memory_append_owner_boundary_tests::document` (L1474): _no doc comment_
- `public_memory_append_owner_boundary_tests::package` (L1482): _no doc comment_
- `public_memory_append_owner_boundary_tests::snapshot` (L1493): _no doc comment_
- `public_memory_append_owner_boundary_tests::view` (L1504): _no doc comment_
- `public_memory_append_owner_boundary_tests::public_append_keeps_source_property_history_and_fresh_annotation_owners_at_every_join_boundary` (L1511): _no doc comment_
- `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (L1594): _no doc comment_
