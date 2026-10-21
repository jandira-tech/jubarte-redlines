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
- **Calls (crate):** `open` (append.rs:316) L157,158; `PartFs::parts` (opc/mod.rs:733) (by name) L160; `main_part` (append.rs:329) L161,162; `Dom::new` (xmllinq/mod.rs:387) L163; `parse` (append.rs:336) L164,165; `body` (append.rs:355) L166,167; `final_sect_pr` (append.rs:360) L168,169; `merge_namespace_declarations` (append.rs:371) L170; `Dom::new_element` (xmllinq/mod.rs:478) L174; `Dom::elements` (xmllinq/mod.rs:640) L175,242,265; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L177,184; `Dom::add` (xmllinq/mod.rs:1074) L178,243; `drop_comments` (append.rs:411) L190,193,207; `offset_annotation_ids` (append.rs:501) L195,198; `drop_colliding_paragraph_ids` (append.rs:529) L200; `carry_notes` (append.rs:673) L205; `CommentFamily::load` (comments.rs:457) L217; `invalid` (append.rs:309) L217; `stage_comments` (append.rs:563) L219; `carry_styles_and_numbering` (append.rs:867) L227; `carry_part_relationships` (comparer/parts.rs:576) L229,231,234,251; `PartFs::set_part` (opc/mod.rs:661) (by name) L245,263; `Dom::serialize_document` (xmllinq/mod.rs:1338) L247,263; `CommentFamily::part_name` (comments.rs:708) (by name) L253; `drop_comments_in_new_parts` (append.rs:456) L260; `join` (append.rs:1270) L262; `Dom::attribute` (xmllinq/mod.rs:874) L266; `CommentFamily::adopt` (comments.rs:743) (by name) L273; `Dom::serialize_element` (xmllinq/mod.rs:1315) L274; `.store` (ambiguous: `CommentFamily::store` (comments.rs:854); `Parsed::store` (revision_processor/comments.rs:54); `Part::store` (scrub.rs:203)) L281; `related` (append.rs:766) L282; `merge_part_namespaces` (append.rs:1087) L283; `PartFs::to_zip` (opc/mod.rs:749) (by name) L296; `package` (append.rs:302) L296,298; `fix_up_drawing_ids_in_package` (comparer/fixups.rs:121) L297
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.cloned`, `.collect`, `.extend`, `.flatten`, `.get`, `.into_bytes`, `.into_iter`, `.iter`, `.map_err`, `.map`, `.ok`, `.parse`, `.push`, `.then_some`, `.unwrap_or_default`, `Ok`, `Some`, `Vec::new`, `W::body`, `W::id`, `W::name`
- **Macros:** `format!`, `vec!`
- **Called by:** `public_memory_append_owner_boundary_tests::public_append_keeps_source_property_history_and_fresh_annotation_owners_at_every_join_boundary` (append.rs:1516); `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1599); `run_append` (bin/jubarte.rs:4433)

#### `package`  (L302-307)
`fn package(document: &'static str, error: &OpcError) -> AppendError`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `append_documents` (append.rs:152); `open` (append.rs:316); `public_memory_append_owner_boundary_tests::public_append_keeps_source_property_history_and_fresh_annotation_owners_at_every_join_boundary` (append.rs:1516); `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1599)

#### `invalid`  (L309-314)
`fn invalid(document: &'static str, message: impl Into<String>) -> AppendError`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`
- **Called by:** `declared_entry_count` (admission.rs:571) (?); `check_xml_depth` (admission.rs:858) (?); `append_documents` (append.rs:152); `main_part` (append.rs:329); `parse` (append.rs:336); `body` (append.rs:355); `stage_comments` (append.rs:563); `tests::error_codes_and_messages_name_the_document` (append.rs:1387); `picture` (edit/images.rs:86) (?); `load_stories` (edit.rs:1680) (?); `check_operations` (edit.rs:2072) (?); `apply_markdown` (markdown/patch.rs:121) (?)

#### `open`  (L316-327)
`fn open(document: &'static str, bytes: &[u8]) -> Result<PartFs, AppendError>`

- **Does:** _no doc comment_
- **Calls (crate):** `admit` (admission.rs:439) L317; `<InputLimits as Default>::default` (admission.rs:64) L317; `strict_to_transitional_docx` (strict_translation.rs:749) L320; `PartFs::open` (opc/mod.rs:585) L326; `package` (append.rs:302) L326
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`
- **Called by:** `append_documents` (append.rs:152); `tests::error_codes_and_messages_name_the_document` (append.rs:1387); `Origin::build` (edit/origin.rs:260) (?); `restore` (edit/ranges.rs:432) (?); `record_mid_changes` (edit/sections.rs:453) (?); `rewrite` (edit/whole.rs:130) (?); `collect_fields` (fields.rs:281) (?)

#### `main_part`  (L329-333)
`fn main_part(document: &'static str, pkg: &PartFs) -> Result<String, AppendError>`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:1070) L330; `PartFs::part_bytes` (opc/mod.rs:628) L331; `invalid` (append.rs:309) L332
- **Calls (std/external/unresolved):** `.filter`, `.is_some`, `.ok_or_else`
- **Called by:** `append_documents` (append.rs:152)

#### `parse`  (L336-353)
`fn parse( document: &'static str, dom: &mut Dom, pkg: &PartFs, part: &str, ) -> Result<(NodeId, NodeId), AppendError>`

- **Does:** Parse `part` of `pkg` into `dom`: `(document node, root element)`.
- **Calls (crate):** `PartFs::part_bytes` (opc/mod.rs:628) L342; `invalid` (append.rs:309) L344,345,347,351; `validate_xml` (xmllinq/parse.rs:1043) L346; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L348; `Dom::root` (xmllinq/mod.rs:822) L349
- **Calls (std/external/unresolved):** `.map_err`, `.ok_or_else`, `Ok`, `std::str::from_utf8`
- **Macros:** `format!`
- **Called by:** `append_documents` (append.rs:152); `stage_comments` (append.rs:563); `carry_notes` (append.rs:673); `carry_styles_and_numbering` (append.rs:867); `merge_part_namespaces` (append.rs:1087)

#### `body`  (L355-358)
`fn body(document: &'static str, dom: &Dom, root: NodeId) -> Result<NodeId, AppendError>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L356; `invalid` (append.rs:309) L357
- **Calls (std/external/unresolved):** `.ok_or_else`, `W::body`
- **Called by:** `append_documents` (append.rs:152)

#### `final_sect_pr`  (L360-365)
`fn final_sect_pr(dom: &Dom, body: NodeId) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L361; `Dom::name_is` (xmllinq/mod.rs:535) L364
- **Calls (std/external/unresolved):** `.copied`, `.filter`, `.last`, `W::sect_pr`
- **Called by:** `append_documents` (append.rs:152)

#### `merge_namespace_declarations`  (L371-406)
`fn merge_namespace_declarations(dom: &mut Dom, to: NodeId, from: NodeId)`

- **Does:** Declare on `to`'s root the prefixes `from`'s root declares and `to` lacks, and add `from`'s ignorable prefixes to `to`'s `mc:Ignorable`, so copied markup keeps the prefixes its `mc:Choice/@Requires` and `mc:Ignorable` lists name.
- **Calls (crate):** `Dom::attributes` (xmllinq/mod.rs:899) L372,378; `Dom::is_namespace_declaration` (xmllinq/mod.rs:1012) L375,379; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L380,404; `Dom::attribute` (xmllinq/mod.rs:874) L384,387,396,398; `XName::get` (xmllinq/mod.rs:180) L395
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.contains`, `.filter`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.join`, `.map`, `.push`, `.split_whitespace`, `.to_string`, `.unwrap_or`, `MC::name`, `Some`
- **Called by:** `append_documents` (append.rs:152); `carry_notes` (append.rs:673); `carry_styles_and_numbering` (append.rs:867); `merge_part_namespaces` (append.rs:1087); `tests::ignorable_prefixes_merge_only_with_the_same_namespace` (append.rs:1423)

#### `drop_comments`  (L411-413)
`fn drop_comments(dom: &mut Dom, root: NodeId) -> usize`

- **Does:** Remove comment anchors and references under `root` (a run left holding only its properties goes too); returns how many distinct comments they named.
- **Calls (crate):** `drop_comments_where` (append.rs:432) L412
- **Called by:** `append_documents` (append.rs:152); `drop_comments_in_new_parts` (append.rs:456)

#### `comment_anchors`  (L416-421)
`fn comment_anchors(dom: &Dom, root: NodeId) -> Vec<NodeId>`

- **Does:** The comment anchors and references under `root`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L417; `Dom::name` (xmllinq/mod.rs:526) L419; `is_comment_anchor` (append.rs:423) L419
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_some_and`
- **Called by:** `drop_comments_where` (append.rs:432); `stage_comments` (append.rs:563)

#### `is_comment_anchor`  (L423-429)
`fn is_comment_anchor(name: &XName) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L424; `XName::local_name` (xmllinq/mod.rs:208) (by name) L426
- **Macros:** `matches!`
- **Called by:** `comment_anchors` (append.rs:416)

#### `drop_comments_where`  (L432-452)
`fn drop_comments_where(dom: &mut Dom, root: NodeId, drop: impl Fn(&str) -> bool) -> usize`

- **Does:** [`drop_comments`] for the anchors whose id `drop` accepts.
- **Calls (crate):** `comment_anchors` (append.rs:416) L434; `Dom::attribute` (xmllinq/mod.rs:874) L435; `Dom::parent` (xmllinq/mod.rs:597) L440; `Dom::remove` (xmllinq/mod.rs:1118) L441,448; `Dom::name_is` (xmllinq/mod.rs:535) L442,446; `Dom::elements` (xmllinq/mod.rs:640) L443
- **Calls (std/external/unresolved):** `.all`, `.filter`, `.insert`, `.iter`, `.len`, `.to_string`, `.unwrap_or`, `HashSet::new`, `W::id`, `W::r_pr`, `W::r`, `drop`
- **Called by:** `drop_comments` (append.rs:411); `stage_comments` (append.rs:563)

#### `drop_comments_in_new_parts`  (L456-479)
`fn drop_comments_in_new_parts(dest: &mut PartFs, before: &HashSet<String>) -> usize`

- **Does:** Comments in header and footer parts copied by this append (parts not in `before`): their anchors name comments A does not have.
- **Calls (crate):** `PartFs::parts` (opc/mod.rs:733) L458; `PartFs::part_string` (opc/mod.rs:636) L462; `Dom::new` (xmllinq/mod.rs:387) L468; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L469; `Dom::root` (xmllinq/mod.rs:822) L470; `drop_comments` (append.rs:411) L471; `PartFs::set_part` (opc/mod.rs:661) L474; `Dom::serialize_document` (xmllinq/mod.rs:1338) L474
- **Calls (std/external/unresolved):** `.contains`, `.ends_with`, `.into_bytes`
- **Called by:** `append_documents` (append.rs:152)

#### `is_annotation`  (L494-496)
`fn is_annotation(name: &XName) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L495; `XName::local_name` (xmllinq/mod.rs:208) L495
- **Calls (std/external/unresolved):** `.contains`
- **Called by:** `offset_annotation_ids` (append.rs:501)

#### `offset_annotation_ids`  (L501-525)
`fn offset_annotation_ids(dom: &mut Dom, existing: NodeId, staged: NodeId)`

- **Does:** Shift every numeric annotation id under `staged` (revisions, bookmarks, move ranges, permissions) past the largest one under `existing`: ids are unique per story, and a start keeps its end's id.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L502,513; `Dom::name` (xmllinq/mod.rs:526) L505,514; `is_annotation` (append.rs:494) L505,514; `Dom::attribute` (xmllinq/mod.rs:874) L506,517; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L522
- **Calls (std/external/unresolved):** `.and_then`, `.filter_map`, `.filter`, `.into_iter`, `.is_some_and`, `.max`, `.ok`, `.parse`, `.to_string`, `.unwrap_or`, `Some`, `W::id`
- **Called by:** `append_documents` (append.rs:152); `carry_notes` (append.rs:673)

#### `drop_colliding_paragraph_ids`  (L529-545)
`fn drop_colliding_paragraph_ids(dom: &mut Dom, existing: NodeId, staged: NodeId)`

- **Does:** Drop `w14:paraId`/`w14:textId` from B's paragraphs whose paragraph id A already uses; Word assigns new ones.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L531,536; `Dom::attribute` (xmllinq/mod.rs:874) L534,537; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L541,542
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter_map`, `.into_iter`, `.is_some_and`, `.map`, `Some`, `W14::name`, `W::p`
- **Called by:** `append_documents` (append.rs:152)

#### `stage_comments`  (L563-636)
`fn stage_comments( src: &PartFs, b_main: &str, dom: &mut Dom, family: &CommentFamily, a_root: NodeId, staged: NodeId, ) -> Result<(Option<StagedComments>, usize), AppendError>`

- **Does:** Give the comments anchored under `staged` ids past every comment id of A (`family` and the anchors under `a_root`), rewrite the anchors, and stage copies of those comments.
- **Calls (crate):** `CommentFamily::load` (comments.rs:457) L571; `invalid` (append.rs:309) L571,605; `drop_comments_where` (append.rs:432) L573; `comment_anchors` (append.rs:416) L574,582; `Dom::attribute` (xmllinq/mod.rs:874) L577,584,596,609; `CommentFamily::order` (comments.rs:568) L585; `CommentFamily::order` (comments.rs:568) (by name) L589; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L601,617; `related` (append.rs:766) L605; `parse` (append.rs:336) L606; `Dom::new_element` (xmllinq/mod.rs:478) L607; `Dom::elements` (xmllinq/mod.rs:640) L608; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L616; `Dom::add` (xmllinq/mod.rs:1074) L618; `CommentFamily::thread_state` (comments.rs:716) (by name) L623
- **Calls (std/external/unresolved):** `.and_then`, `.chain`, `.collect`, `.contains`, `.copied`, `.filter_map`, `.get`, `.insert`, `.into_iter`, `.is_empty`, `.is_ok_and`, `.iter`, `.map_err`, `.map_or`, `.max`, `.ok_or_else`, `.ok`, `.parse`, `.to_string`, `HashMap::new`, `Ok`, `Some`, `W::id`, `W::name`, `defined`
- **Called by:** `append_documents` (append.rs:152)

#### `carry_notes`  (L673-762)
`fn carry_notes( kind: &NoteKind, dest: &mut PartFs, a_main: &str, src: &PartFs, b_main: &str, dom: &mut Dom, staged: NodeId, ) -> Result<Option<CarriedNotes>, AppendError>`

- **Does:** Copy the notes B's staged content references into A's notes part (creating it, with separators, when A has none) under ids past A's, and rewrite the references.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L683; `related` (append.rs:766) L688,698; `parse` (append.rs:336) L689,718; `Dom::remove` (xmllinq/mod.rs:1118) L694,751; `free_sibling` (append.rs:777) L701; `add_related_part` (append.rs:796) L714; `merge_namespace_declarations` (append.rs:371) L719; `Dom::elements` (xmllinq/mod.rs:640) L720,728; `Dom::attribute` (xmllinq/mod.rs:874) L723,731,736; `Dom::new_element` (xmllinq/mod.rs:478) L733; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L740; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L743,750; `Dom::add` (xmllinq/mod.rs:1074) L744; `offset_annotation_ids` (append.rs:501) L754
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.filter_map`, `.get`, `.insert`, `.into_iter`, `.is_empty`, `.map`, `.max`, `.ok`, `.parse`, `.to_string`, `.unwrap_or`, `HashMap::new`, `Ok`, `Some`, `W::id`, `W::name`
- **Macros:** `format!`
- **Called by:** `append_documents` (append.rs:152)

#### `related`  (L766-774)
`fn related(pkg: &PartFs, main: &str, kind: &str) -> Option<String>`

- **Does:** The internal part `main` relates to with a relationship type ending in `kind`, when the part exists.
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:991) L767; `PartFs::resolve_rel_target` (opc/mod.rs:829) L772; `PartFs::part_bytes` (opc/mod.rs:628) L773
- **Calls (std/external/unresolved):** `.as_deref`, `.ends_with`, `.find`, `.is_some`, `.iter`, `.then_some`, `Some`
- **Called by:** `append_documents` (append.rs:152); `stage_comments` (append.rs:563); `carry_notes` (append.rs:673); `carry_styles_and_numbering` (append.rs:867); `stores` (markdown/data_stores.rs:155) (?)

#### `free_sibling`  (L777-794)
`fn free_sibling(pkg: &PartFs, main: &str, name: &str) -> String`

- **Does:** `name` beside `main`, or `name` with a number when that is taken.
- **Calls (crate):** `path` (debug.rs:428) (by name) L787,790; `PartFs::part_bytes` (opc/mod.rs:628) L789
- **Calls (std/external/unresolved):** `.is_empty`, `.is_some`, `.map_or`, `.rsplit_once`, `.to_string`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `carry_notes` (append.rs:673); `carry_styles_and_numbering` (append.rs:867); `tests::free_sibling_numbers_a_taken_name` (append.rs:1372)

#### `add_related_part`  (L796-811)
`fn add_related_part( pkg: &mut PartFs, main: &str, part: &str, rel: &str, content_type: &str, xml: String, )`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::add_document_relationship` (opc/mod.rs:1003) L804; `relative_rel_target` (opc/mod.rs:180) L807; `PartFs::add_content_type_override` (opc/mod.rs:845) L809; `PartFs::set_part` (opc/mod.rs:661) L810
- **Calls (std/external/unresolved):** `.into_bytes`
- **Macros:** `format!`
- **Called by:** `carry_notes` (append.rs:673); `carry_styles_and_numbering` (append.rs:867)

#### `w_vals`  (L818-827)
`fn w_vals(dom: &Dom, root: NodeId, locals: &[&str]) -> Vec<(NodeId, String)>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L819; `Dom::name` (xmllinq/mod.rs:526) L822; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L823; `XName::local_name` (xmllinq/mod.rs:208) (by name) L823; `Dom::attribute` (xmllinq/mod.rs:874) L825
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter_map`, `.filter`, `.into_iter`, `.is_some_and`, `.to_string`, `Some`, `W::val`
- **Called by:** `carry_styles_and_numbering` (append.rs:867); `ListCopy::build` (append.rs:1117)

#### `style_key`  (L831-846)
`fn style_key(dom: &Dom, style: NodeId) -> Option<(String, String)>`

- **Does:** Word's pairing key for a style: type and name (the id when unnamed), built-in names folded to lower case.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L832,838,839; `Dom::element` (xmllinq/mod.rs:655) L836; `is_built_in` (builtin_styles.rs:394) L840
- **Calls (std/external/unresolved):** `.and_then`, `.or_else`, `.to_ascii_lowercase`, `.to_string`, `.unwrap_or`, `Some`, `W::name`, `W::val`
- **Called by:** `carry_styles_and_numbering` (append.rs:867)

#### `index_by_attribute`  (L848-861)
`fn index_by_attribute( dom: &Dom, root: Option<NodeId>, element: &str, attribute: &str, ) -> HashMap<String, NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L855; `Dom::attribute` (xmllinq/mod.rs:874) L857
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.into_iter`, `.map`, `.to_string`, `.unwrap_or_default`, `Some`, `W::name`
- **Called by:** `carry_styles_and_numbering` (append.rs:867)

#### `carry_styles_and_numbering`  (L867-1082)
`fn carry_styles_and_numbering( dest: &mut PartFs, a_main: &str, src: &PartFs, b_main: &str, dom: &mut Dom, roots: &[NodeId], ) -> Result<(), AppendError>`

- **Does:** Carry the styles and lists the staged `roots` use: a B style pairs with A's style of the same type and name, else is copied (renamed `{id}B` when A uses its id) with the styles it is based on, links to and is followed by; each B list is copied with ids past A's.
- **Calls (crate):** `related` (append.rs:766) L875,879,883,1020,1041,1053...; `parse` (append.rs:336) L876,880,886,985; `index_by_attribute` (append.rs:848) L888,889,894,895; `style_key` (append.rs:831) L892,924; `w_vals` (append.rs:818) L905,912,941,942,955,997...; `abstract_of` (append.rs:1226) L953,1026; `free_sibling` (append.rs:777) L972; `add_related_part` (append.rs:796) L973; `merge_namespace_declarations` (append.rs:371) L990; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L993; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L994,996,999,1012,1074; `Dom::add` (xmllinq/mod.rs:1074) L1002; `PartFs::part_string` (opc/mod.rs:636) L1021; `nsids` (append.rs:1232) L1022; `PartFs::part_bytes` (opc/mod.rs:628) L1042,1054; `append_numbering` (markdown/package.rs:1185) L1044; `ListCopy::build` (append.rs:1117) L1048; `insert_lists` (append.rs:1165) L1060; `merge_part_namespaces` (append.rs:1087) L1066; `PartFs::set_part` (opc/mod.rs:661) L1079; `Dom::serialize_document` (xmllinq/mod.rs:1338) L1079
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.clone`, `.cloned`, `.collect`, `.contains_key`, `.contains`, `.copied`, `.extend`, `.filter_map`, `.filter`, `.get`, `.insert`, `.into_bytes`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.keys`, `.map`, `.pop`, `.push`, `.to_vec`, `.transpose`, `.unwrap_or_default`, `HashMap::new`, `Ok`, `Some`, `Vec::new`, `W::name`, `W::val`
- **Macros:** `format!`
- **Called by:** `append_documents` (append.rs:152)

#### `merge_part_namespaces`  (L1087-1100)
`fn merge_part_namespaces( dest: &mut PartFs, part: &str, dom: &mut Dom, from: NodeId, ) -> Result<(), AppendError>`

- **Does:** Merge `from`'s namespace declarations and ignorable prefixes onto the root of `dest`'s `part`, which received markup copied from under `from`; the part is rewritten only when its root changed.
- **Calls (crate):** `parse` (append.rs:336) L1093; `Dom::attributes` (xmllinq/mod.rs:899) L1094,1096; `merge_namespace_declarations` (append.rs:371) L1095; `PartFs::set_part` (opc/mod.rs:661) L1097; `Dom::serialize_document` (xmllinq/mod.rs:1338) L1097
- **Calls (std/external/unresolved):** `.into_bytes`, `Ok`
- **Called by:** `append_documents` (append.rs:152); `carry_styles_and_numbering` (append.rs:867)

#### `ListCopy::build`  (L1117-1160)
`fn build( &self, dom: &mut Dom, first_abstract: u32, first_num: u32, ) -> (String, String, HashMap<String, String>)`

- **Does:** The copies' markup numbered from `first_abstract` and `first_num`: `(abstracts, nums, B num id -> new num id)`.
- **Calls (crate):** `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1129,1148; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1130,1138,1149,1155; `unique_nsid` (append.rs:1247) L1131; `Dom::descendants` (xmllinq/mod.rs:668) L1133; `Dom::remove` (xmllinq/mod.rs:1118) L1134; `w_vals` (append.rs:818) L1136; `Dom::serialize_element` (xmllinq/mod.rs:1315) L1141,1157; `Dom::elements` (xmllinq/mod.rs:640) L1150; `Dom::attribute` (xmllinq/mod.rs:874) L1151
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.clone`, `.cloned`, `.enumerate`, `.get`, `.insert`, `.iter`, `.push_str`, `.to_string`, `.unwrap_or`, `HashMap::new`, `Some`, `String::new`, `W::name`, `W::val`, `u32::try_from`
- **Called by:** `carry_styles_and_numbering` (append.rs:867); `insert_lists` (append.rs:1165); `task_usage` (cli.rs:3755) (?); `run` (update.rs:186) (?)

#### `insert_lists`  (L1165-1205)
`fn insert_lists( dest: &mut PartFs, part: &str, dom: &mut Dom, lists: &ListCopy<'_>, ) -> Option<HashMap<String, String>>`

- **Does:** Add `lists` to the numbering `part` through the DOM, numbered past the part's own ids whatever its prefix, in schema order; returns the num map.
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L1171; `Dom::new` (xmllinq/mod.rs:387) L1172; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L1173,1194; `Dom::root` (xmllinq/mod.rs:822) L1174,1195; `Dom::elements` (xmllinq/mod.rs:640) L1176,1198; `Dom::attribute` (xmllinq/mod.rs:874) L1180; `ListCopy::build` (append.rs:1117) L1192; `Dom::remove` (xmllinq/mod.rs:1118) L1199; `add_in_numbering_order` (append.rs:1208) L1200; `PartFs::set_part` (opc/mod.rs:661) L1203; `Dom::serialize_document` (xmllinq/mod.rs:1338) L1203
- **Calls (std/external/unresolved):** `.filter_map`, `.into_bytes`, `.into_iter`, `.max`, `.ok`, `.parse`, `.unwrap_or`, `Some`, `W::name`, `next`, `u32::try_from`
- **Macros:** `format!`
- **Called by:** `carry_styles_and_numbering` (append.rs:867)

#### `add_in_numbering_order`  (L1208-1224)
`fn add_in_numbering_order(dom: &mut Dom, numbering: NodeId, child: NodeId)`

- **Does:** `w:numbering` lists picture bullets, then abstracts, then nums.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L1209; `rank` (ambiguous, 4 candidates) L1215,1219; `Dom::elements` (xmllinq/mod.rs:640) L1216; `Dom::add_before_self` (xmllinq/mod.rs:1130) L1221; `Dom::add` (xmllinq/mod.rs:1074) L1222
- **Calls (std/external/unresolved):** `.as_ref`, `.find`, `.into_iter`, `.map`
- **Called by:** `insert_lists` (append.rs:1165)

#### `abstract_of`  (L1226-1229)
`fn abstract_of(dom: &Dom, num: NodeId) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1227; `Dom::attribute` (xmllinq/mod.rs:874) L1228
- **Calls (std/external/unresolved):** `.map`, `W::name`, `W::val`
- **Called by:** `carry_styles_and_numbering` (append.rs:867)

#### `nsids`  (L1232-1243)
`fn nsids(xml: &str) -> HashSet<String>`

- **Does:** The `w:nsid` values of a numbering part.
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L1233; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L1234; `Dom::root` (xmllinq/mod.rs:822) L1235; `Dom::descendants` (xmllinq/mod.rs:668) L1237; `Dom::attribute` (xmllinq/mod.rs:874) L1239
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.into_iter`, `.map`, `.unwrap_or_default`, `Some`, `W::name`, `W::val`
- **Called by:** `carry_styles_and_numbering` (append.rs:867)

#### `unique_nsid`  (L1247-1267)
`fn unique_nsid(dom: &mut Dom, abstract_num: NodeId, used: &mut HashSet<String>)`

- **Does:** Give `abstract_num` an `w:nsid` no list in `used` has: Word treats lists that share one as the same list.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1248; `Dom::attribute` (xmllinq/mod.rs:874) L1251; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1265
- **Calls (std/external/unresolved):** `.contains`, `.insert`, `.to_ascii_uppercase`, `.unwrap_or`, `.wrapping_add`, `Some`, `W::name`, `W::val`, `u32::from_str_radix`
- **Macros:** `format!`
- **Called by:** `ListCopy::build` (append.rs:1117); `tests::nsid_collisions_take_the_next_free_value` (append.rs:1357)

#### `join`  (L1270-1319)
`fn join( dom: &mut Dom, a_body: NodeId, a_sect: Option<NodeId>, staged: NodeId, b_final_sect: Option<NodeId>, options: &AppendOptions, )`

- **Does:** Put B's staged content after A's, before A's final section properties.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L1283,1284,1287,1295,1296,1297; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L1286; `Dom::add` (xmllinq/mod.rs:1074) L1289,1290,1299,1300,1310,1316; `set_section_type` (append.rs:1331) L1292; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1298; `Dom::elements` (xmllinq/mod.rs:640) L1305; `Dom::remove` (xmllinq/mod.rs:1118) L1307; `Dom::add_before_self` (xmllinq/mod.rs:1130) L1309; `Dom::replace_with` (xmllinq/mod.rs:1165) L1315
- **Calls (std/external/unresolved):** `.extend`, `.push`, `Some`, `Vec::new`, `W::name`, `W::p_pr`, `W::p`, `W::r`, `W::sect_pr`
- **Called by:** `append_documents` (append.rs:152); `tests::splice_marks_a_tracked_break_and_extends_adjacent_spans` (markdown/from_docx/critic.rs:1989) (?); `tests::a_span_after_a_code_fence_opens_on_the_next_block_s_line` (markdown/from_docx/critic.rs:2019) (?); `tests::a_plain_span_across_paragraphs_names_every_piece_s_id` (markdown/from_docx/critic.rs:2070) (?); `tests::a_tracked_break_extends_spans_only_by_the_same_person` (markdown/from_docx/critic.rs:2368) (?)

#### `set_section_type`  (L1331-1349)
`fn set_section_type(dom: &mut Dom, sect: NodeId, section_break: SectionBreak)`

- **Does:** How B's section starts: on a new page (Word's default, no `w:type`) or continuously.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1333,1341; `Dom::remove` (xmllinq/mod.rs:1118) L1334; `Dom::new_element` (xmllinq/mod.rs:478) L1339; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1340; `Dom::name` (xmllinq/mod.rs:526) L1342; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1343; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1343; `Dom::add_before_self` (xmllinq/mod.rs:1130) L1346; `Dom::add` (xmllinq/mod.rs:1074) L1347
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.into_iter`, `.is_some_and`, `Some`, `W::name`, `W::val`
- **Called by:** `join` (append.rs:1270); `tests::section_type_sits_after_header_references` (append.rs:1404)

## Test functions (13)

- `tests::nsid_collisions_take_the_next_free_value` (L1357): _no doc comment_
- `tests::free_sibling_numbers_a_taken_name` (L1372): _no doc comment_
- `tests::error_codes_and_messages_name_the_document` (L1387): _no doc comment_
- `tests::section_type_sits_after_header_references` (L1404): _no doc comment_
- `tests::ignorable_prefixes_merge_only_with_the_same_namespace` (L1423): _no doc comment_
- `tests::minimal` (L1443): _no doc comment_
- `public_memory_append_owner_boundary_tests::semantic` (L1454): _no doc comment_
- `public_memory_append_owner_boundary_tests::document` (L1479): _no doc comment_
- `public_memory_append_owner_boundary_tests::package` (L1487): _no doc comment_
- `public_memory_append_owner_boundary_tests::snapshot` (L1498): _no doc comment_
- `public_memory_append_owner_boundary_tests::view` (L1509): _no doc comment_
- `public_memory_append_owner_boundary_tests::public_append_keeps_source_property_history_and_fresh_annotation_owners_at_every_join_boundary` (L1516): _no doc comment_
- `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (L1599): _no doc comment_
