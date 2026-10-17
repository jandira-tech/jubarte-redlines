# Functions and methods in `src/comments.rs`

40 non-test, 18 test. Part of the [symbol manifest](../README.md).

#### `CommentError::code`  (L93-98, pub)
`pub fn code(&self) -> &'static str`

- **Does:** Stable error code, as the edit API reports it: the admission code, or `INVALID_DOCUMENT` for an unreadable package.
- **Calls (crate):** `.code` (ambiguous, 4 candidates) L95
- **Called by:** none found in src

#### `<CommentError as fmt::Display>::fmt`  (L102-107, impl fmt::Display)
`fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`

- **Does:** _no doc comment_
- **Macros:** `write!`
- **Called by:** none found in src

#### `<CommentError as From>::from`  (L113-118, impl From)
`fn from(error: crate::inspect::InspectError) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.to_string`, `CommentError::Admission`, `CommentError::Package`
- **Called by:** none found in src

#### `list_comments`  (L130-132, pub)
`pub fn list_comments(docx: &[u8]) -> Result<Vec<CommentRecord>, CommentError>`

- **Does:** Every comment in `comments.xml` order, with its thread, resolution and anchor.
- **Calls (crate):** `comments_in` (comments.rs:136) L131; `Opened::open` (inspect.rs:525) L131
- **Called by:** `run_comments` (bin/jubarte.rs:415); `public_memory_anchor_boundary_tests::block_and_point_comments_preserve_exact_unicode_anchor_context_and_package` (comments.rs:1371); `authored_thread_reference_contract_tests::cyclic_and_missing_parent_metadata_terminate_without_discarding_comment_owners` (comments.rs:1548); `coverage_real_revision_merge_matrix_tests::organic_source_snapshot` (comparer/finalize.rs:26876); `public_memory_validator_owner_boundary_tests::repairing_comment_paragraph_ids_retains_reply_graph_source_text_and_all_relationships` (validate.rs:2498)

#### `comments_in`  (L136-156, pub(crate))
`pub(crate) fn comments_in(opened: &mut Opened) -> Result<Vec<CommentRecord>, CommentError>`

- **Does:** [`list_comments`] of a package already open.
- **Calls (crate):** `CommentFamily::load` (comments.rs:429) L137; `Anchors::scan` (comments.rs:241) L142,145; `Opened::story_parts` (inspect.rs:641) L143; `parse_part` (inspect.rs:693) L144; `CommentFamily::order` (comments.rs:534) (by name) L147; `.record` (ambiguous: `CommentFamily::record` (comments.rs:578); `FontIndex::record` (convert/font.rs:3192)) L151; `Anchors::fill` (comments.rs:342) L152
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.is_empty`, `.map_err`, `.map`, `Anchors::default`, `Ok`, `Vec::new`
- **Called by:** `list_comments` (comments.rs:130); `snapshot` (inspect.rs:362)

#### `comments_text`  (L162-183, pub)
`pub fn comments_text(comments: &[CommentRecord]) -> String`

- **Does:** `jubarte comments`' listing: one line per comment, `c{id}\t{paragraph} \t{author}\t{text}\ton {anchor}[\treply to c{parent}][\tresolved]` (`-` for an absent paragraph; text and anchor quoted, their first 60 and 40 characters), then `{n} comment(s)`.
- **Calls (std/external/unresolved):** `.as_deref`, `.chars`, `.collect`, `.len`, `.map_or_else`, `.map`, `.push_str`, `.take`, `.to_string`, `.unwrap_or_default`, `String::new`
- **Macros:** `format!`
- **Called by:** `run_comments` (bin/jubarte.rs:415)

#### `select_comments`  (L188-223, pub)
`pub fn select_comments( records: Vec<CommentRecord>, author: Option<&str>, latest: bool, ) -> Vec<CommentRecord>`

- **Does:** Filter a listing: `author` keeps that author's comments (exact match); `latest` keeps one comment per thread, the newest by `date` (later in the listing on a tie), threads in the order their first comment appears.
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.find`, `.get`, `.insert`, `.into_iter`, `.is_none_or`, `.iter_mut`, `.iter`, `.map`, `.push`, `HashSet::new`, `Vec::new`, `root_of`
- **Called by:** `run_comments` (bin/jubarte.rs:415); `public_memory_anchor_boundary_tests::latest_thread_selection_preserves_complete_winning_source_records_and_thread_order` (comments.rs:1460); `authored_thread_reference_contract_tests::cyclic_and_missing_parent_metadata_terminate_without_discarding_comment_owners` (comments.rs:1548)

#### `Anchors::scan`  (L241-340)
`fn scan(&mut self, story_id: &str, dom: &mut Dom, root: NodeId, body: bool)`

- **Does:** Locate every marker of one story.
- **Calls (crate):** `body_paragraph_nodes` (inspect.rs:747) L244; `story_paragraph_nodes` (inspect.rs:937) L246; `Dom::descendants` (xmllinq/mod.rs:668) L253,269; `Dom::ancestors` (xmllinq/mod.rs:770) L260; `Dom::attribute` (xmllinq/mod.rs:874) L270; `paragraph_of` (ambiguous: `paragraph_of` (changes.rs:676); `paragraph_of` (comparer/bookmarks.rs:66)) L273; `Dom::new_element` (xmllinq/mod.rs:478) L287,288; `Dom::add_text` (xmllinq/mod.rs:1071) L289; `Dom::add` (xmllinq/mod.rs:1058) L290; `Dom::add_before_self` (xmllinq/mod.rs:1114) L291; `project_paragraph` (inspect.rs:1024) L312
- **Calls (std/external/unresolved):** `.chars`, `.clone`, `.collect`, `.copied`, `.entry`, `.enumerate`, `.find_map`, `.get`, `.insert`, `.into_iter`, `.iter`, `.len`, `.map`, `.or_insert`, `.position`, `.push`, `.rposition`, `.to_string`, `.unwrap_or`, `HashMap::new`, `Some`, `String::with_capacity`, `Vec::new`, `Vec::with_capacity`, `W::id`, `W::name`, `W::p`, `W::r`, `W::t`, `char::from_u32`
- **Called by:** `comments_in` (comments.rs:136)

#### `Anchors::fill`  (L342-376)
`fn fill(&self, record: &mut CommentRecord)`

- **Does:** _no doc comment_
- **Calls (crate):** `paragraph_id` (ids.rs:410) L347; `paragraph_id` (ids.rs:410) L353,354,372; `tail` (comments.rs:380) L367
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.copied`, `.get`, `.push_str`, `.push`, `.take`, `.then`, `.to_string`, `Some`
- **Called by:** `comments_in` (comments.rs:136); `bidi_visual_line` (convert/mod.rs:32700) (?); `subset_keep_gids` (convert/pdf.rs:1185) (?); `raster_payload_boundary_tests::white` (convert/raster.rs:910) (?); `residual_display_contract_tests::white` (convert/raster.rs:1142) (?); `without_emphasis` (markdown/anchor.rs:139) (?); `wrap_markers` (markdown/from_docx/critic.rs:937) (?)

#### `tail`  (L380-383)
`fn tail(text: &str, n: usize) -> String`

- **Does:** The last `n` characters of `text`.
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.count`, `.saturating_sub`, `.skip`
- **Called by:** `Anchors::fill` (comments.rs:342); `tests::tail_counts_characters_not_bytes` (comments.rs:1178)

#### `CommentFamily::load`  (L429-455, pub(crate))
`pub(crate) fn load(pkg: &PartFs, main: &str) -> Result<Self, String>`

- **Does:** Load the family the main part `main` relates to.
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L430; `related_part` (comments.rs:1027) L433; `PartFs::part_string` (opc/mod.rs:498) L436; `validate_xml` (xmllinq/parse.rs:792) L439; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L440; `Dom::root` (xmllinq/mod.rs:822) L441
- **Calls (std/external/unresolved):** `.enumerate`, `.iter`, `.map_err`, `.ok_or_else`, `.read_meta`, `BTreeMap::new`, `HashSet::new`, `Ok`, `Some`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `append_documents` (append.rs:152); `stage_comments` (append.rs:557); `comments_in` (comments.rs:136); `people_and_text_boundary_tests::people_updates_preserve_existing_presence_and_add_each_remaining_author_once` (comments.rs:1217); `people_and_text_boundary_tests::removing_the_last_person_removes_its_part_type_and_relationship` (comments.rs:1253); `people_and_text_boundary_tests::unchanged_people_are_byte_identical_and_new_authors_do_not_create_a_people_part` (comments.rs:1276); `people_and_text_boundary_tests::adopted_comments_preserve_text_and_drop_old_paragraph_ids` (comments.rs:1316); `authored_thread_reference_contract_tests::family` (comments.rs:1504); `Origin::had_comment` (edit/origin.rs:248); `Transaction::start` (edit.rs:2004); `Transaction::write_comments_part` (edit.rs:5190); `threads` (markdown/package.rs:73)

#### `CommentFamily::read_meta`  (L457-521)
`fn read_meta(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) (by name) L464; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L466,480,490,494,496; `rows` (ambiguous: `shared_word_alignment_tests::rows` (comparer/lcs_table.rs:1888); `rows` (convert/word_subst.rs:130); `tests::rows` (edit/structural.rs:487)) L470,471,472; `XName::get` (xmllinq/mod.rs:180) L471,474,475,490; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L479; `last_para_id` (comments.rs:1037) L486
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.collect`, `.copied`, `.eq_ignore_ascii_case`, `.filter_map`, `.filter`, `.get_mut`, `.get`, `.insert`, `.into_iter`, `.is_some_and`, `.map_or_else`, `.map`, `.ok`, `.parse`, `.push`, `.to_string`, `HashMap::new`, `Some`, `Vec::new`, `W15::name`, `W::id`, `W::name`
- **Called by:** none found in src

#### `CommentFamily::is_empty`  (L524-526, pub(crate))
`pub(crate) fn is_empty(&self) -> bool`

- **Does:** No comment is defined.
- **Calls (std/external/unresolved):** `.is_empty`
- **Called by:** none found in src

#### `CommentFamily::contains`  (L529-531, pub(crate))
`pub(crate) fn contains(&self, id: u32) -> bool`

- **Does:** Whether comment `id` is defined.
- **Calls (std/external/unresolved):** `.contains_key`
- **Called by:** none found in src

#### `CommentFamily::order`  (L534-548, pub(crate))
`pub(crate) fn order(&self) -> Vec<u32>`

- **Does:** Comment ids in `comments.xml` order.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L538; `Dom::attribute` (xmllinq/mod.rs:874) L542
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.contains_key`, `.filter_map`, `.filter`, `.into_iter`, `.ok`, `.parse`, `Some`, `Vec::new`, `W::id`, `W::name`
- **Called by:** `stage_comments` (append.rs:557); `comments_in` (comments.rs:136) (?); `CommentFamily::add` (comments.rs:604); `CommentFamily::store` (comments.rs:798)

#### `CommentFamily::thread_root`  (L551-560, pub(crate))
`pub(crate) fn thread_root(&self, mut id: u32) -> u32`

- **Does:** The first comment of `id`'s thread.
- **Calls (std/external/unresolved):** `.and_then`, `.get`, `.insert`, `HashSet::new`
- **Called by:** `CommentFamily::add` (comments.rs:604); `CommentFamily::set_thread` (comments.rs:763); `authored_thread_reference_contract_tests::cyclic_and_missing_parent_metadata_terminate_without_discarding_comment_owners` (comments.rs:1548) (?)

#### `CommentFamily::with_replies`  (L563-576, pub(crate))
`pub(crate) fn with_replies(&self, id: u32) -> Vec<u32>`

- **Does:** `id` and every comment that replies to it, directly or not.
- **Calls (std/external/unresolved):** `.contains`, `.len`, `.push`, `Some`
- **Macros:** `vec!`
- **Called by:** `CommentFamily::set_done` (comments.rs:774); `CommentFamily::remove` (comments.rs:783); `authored_thread_reference_contract_tests::cyclic_and_missing_parent_metadata_terminate_without_discarding_comment_owners` (comments.rs:1548) (?); `thread_markers` (edit/tracked.rs:701) (?); `Transaction::resolve_thread` (edit.rs:3342) (?); `Transaction::check_thread_ops` (edit.rs:4326) (?); `Transaction::apply` (edit.rs:4489) (?)

#### `CommentFamily::record`  (L578-600)
`fn record(&self, id: u32) -> CommentRecord`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L581; `attr` (ambiguous: `attr` (convert/altchunk.rs:47); `attr` (debug/diff.rs:468); `attr` (debug.rs:523)) L588,589,590; `comment_text` (comments.rs:1045) L591
- **Calls (std/external/unresolved):** `.map`, `.unwrap_or_default`, `String::new`, `W::name`
- **Macros:** `format!`
- **Called by:** `comments_in` (comments.rs:136) (?)

#### `CommentFamily::add`  (L604-641, pub(crate))
`pub(crate) fn add(&mut self, new: &NewComment<'_>)`

- **Does:** Add a comment; a reply goes right after the last comment of its thread and hangs off the thread's first comment.
- **Calls (crate):** `CommentFamily::comments_root` (comments.rs:644) L605; `Dom::new_element` (xmllinq/mod.rs:478) (by name) L607,612; `Dom::set_attribute_value` (xmllinq/mod.rs:961) (by name) L608,609,610,611; `fill_comment_paragraph` (comments.rs:1073) L613; `.add` (ambiguous, 5 candidates) L614; `CommentFamily::thread_root` (comments.rs:551) L615,620; `CommentFamily::order` (comments.rs:534) L617; `Dom::add_after_self` (xmllinq/mod.rs:1131) L624; `Dom::add` (xmllinq/mod.rs:1058) L625
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.find`, `.insert`, `.into_iter`, `.iter`, `.map`, `.push`, `.rev`, `.to_string`, `Some`, `W::author`, `W::date`, `W::id`, `W::name`, `W::p`
- **Called by:** none found in src

#### `CommentFamily::comments_root`  (L644-660)
`fn comments_root(&mut self) -> NodeId`

- **Does:** The root of `comments.xml`, created when the family has none.
- **Calls (crate):** `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L648; `Dom::root` (xmllinq/mod.rs:822) L653
- **Calls (std/external/unresolved):** `.expect`, `.to_string`, `Some`
- **Macros:** `format!`
- **Called by:** `CommentFamily::add` (comments.rs:604); `CommentFamily::adopt` (comments.rs:687)

#### `CommentFamily::part_name`  (L664-668, pub(crate))
`pub(crate) fn part_name(&self) -> String`

- **Does:** The comments part's name: the one the main part relates to, or the name [`Self::store`] gives a new one.
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.map_or_else`, `.to_string`
- **Called by:** `append_documents` (append.rs:152) (?)

#### `CommentFamily::thread_state`  (L672-679, pub(crate))
`pub(crate) fn thread_state(&self, id: u32) -> Option<(Option<u32>, bool, Option<String>)>`

- **Does:** Comment `id`'s thread parent, resolution and UTC date (`w16cex:dateUtc`), when `id` is defined.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L676; `XName::get` (xmllinq/mod.rs:180) L676
- **Calls (std/external/unresolved):** `.and_then`, `.get`, `.map`, `Some`
- **Called by:** `stage_comments` (append.rs:557) (?); `people_and_text_boundary_tests::adopted_comments_preserve_text_and_drop_old_paragraph_ids` (comments.rs:1316) (?)

#### `CommentFamily::adopt`  (L687-733, pub(crate))
`pub(crate) fn adopt( &mut self, xml: &str, id: u32, parent: Option<u32>, done: bool, date_utc: Option<&str>, )`

- **Does:** Add a comment copied whole from another package (its formatting, line breaks and links stay): `xml` is its `w:comment` element with the namespaces it uses declared on it.
- **Calls (crate):** `CommentFamily::comments_root` (comments.rs:644) L695; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) (by name) L697; `.root` (ambiguous: `Dom::root` (xmllinq/mod.rs:822); `Scope::root` (xmllinq/serialize.rs:211)) L698; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L700; `Dom::set_attribute_value` (xmllinq/mod.rs:961) (by name) L705,707,708,713; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L706; `.add` (ambiguous, 5 candidates) L710; `Dom::new_element` (xmllinq/mod.rs:478) (by name) L712; `XName::get` (xmllinq/mod.rs:180) L712,713; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L716
- **Calls (std/external/unresolved):** `.and_then`, `.contains_key`, `.contains`, `.copied`, `.filter`, `.first`, `.insert`, `.map`, `.push`, `.remove`, `.to_string`, `Some`, `W14::name`, `W::author`, `W::id`, `W::name`, `W::p`
- **Macros:** `format!`
- **Called by:** `append_documents` (append.rs:152) (?); `people_and_text_boundary_tests::adopted_comments_preserve_text_and_drop_old_paragraph_ids` (comments.rs:1316) (?)

#### `CommentFamily::set_text`  (L737-758, pub(crate))
`pub(crate) fn set_text(&mut self, id: u32, text: &str)`

- **Does:** Replace a comment's text.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) (by name) L743; `Dom::new_element` (xmllinq/mod.rs:478) (by name) L747; `.nodes` (ambiguous: `Writer::nodes` (markdown/math.rs:1728); `Dom::nodes` (xmllinq/mod.rs:604)) L748,751; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L752; `fill_comment_paragraph` (comments.rs:1073) L756; `.add` (ambiguous, 5 candidates) L757
- **Calls (std/external/unresolved):** `.copied`, `.get`, `.last`, `.remove`, `.unwrap_or_else`, `Some`, `W::p_pr`, `W::p`
- **Called by:** `Transaction::write_comments_part` (edit.rs:5190) (?)

#### `CommentFamily::set_thread`  (L763-771, pub(crate))
`pub(crate) fn set_thread(&mut self, id: u32, parent: Option<u32>, done: bool)`

- **Does:** Set comment `id`'s thread: the comment it replies to (it hangs off that comment's thread root, Word threads being one level deep) and whether it is resolved.
- **Calls (crate):** `CommentFamily::thread_root` (comments.rs:551) L766
- **Calls (std/external/unresolved):** `.contains_key`, `.filter`, `.get_mut`, `.map`
- **Called by:** `threads` (markdown/package.rs:73) (?)

#### `CommentFamily::set_done`  (L774-780, pub(crate))
`pub(crate) fn set_done(&mut self, id: u32, done: bool)`

- **Does:** Mark `id` and its replies resolved (or open again).
- **Calls (crate):** `CommentFamily::with_replies` (comments.rs:563) L775
- **Calls (std/external/unresolved):** `.get_mut`
- **Called by:** `Transaction::write_comments_part` (edit.rs:5190) (?); `threads` (markdown/package.rs:73) (?)

#### `CommentFamily::remove`  (L783-794, pub(crate))
`pub(crate) fn remove(&mut self, id: u32) -> Vec<u32>`

- **Does:** Remove `id` and its replies; the removed ids.
- **Calls (crate):** `CommentFamily::with_replies` (comments.rs:563) L784; `Dom::attribute` (xmllinq/mod.rs:874) L787; `Dom::remove` (xmllinq/mod.rs:1102) L790
- **Calls (std/external/unresolved):** `.insert`, `.remove`, `.to_string`, `W::author`
- **Called by:** none found in src

#### `CommentFamily::store`  (L798-920, pub(crate))
`pub(crate) fn store(mut self, pkg: &mut PartFs, main: &str)`

- **Does:** Write the family back into `pkg`: every part present and consistent, or none of them when no comment is left; `people.xml` follows.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L802,873,899; `PartFs::remove_part` (opc/mod.rs:527) L809; `PartFs::remove_content_type_override` (opc/mod.rs:668) L810; `PartFs::remove_relationships_by_type` (opc/mod.rs:675) L811; `CommentFamily::store_people` (comments.rs:977) L813,919; `CommentFamily::stamp_para_ids` (comments.rs:942) L816; `CommentFamily::order` (comments.rs:534) L817; `last_para_id` (comments.rs:1037) L820; `CommentFamily::ensure_part` (comments.rs:923) L823,849,854; `clear_rows` (comments.rs:1092) L829,859,860; `Dom::new_element` (xmllinq/mod.rs:478) L834,887,895; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L837,842,844,888,890,896...; `Dom::add` (xmllinq/mod.rs:1058) L846,892,907; `XName::get` (xmllinq/mod.rs:180) L859,863,865,887,889,895...; `is_hex8` (comments.rs:1098) L875; `allocate_hex8` (comments.rs:1103) L883; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L900; `Dom::serialize_document` (xmllinq/mod.rs:1322) L911; `PartFs::set_part` (opc/mod.rs:510) L912; `related_part` (comments.rs:1027) L913; `relative_rel_target` (opc/mod.rs:129) L914; `PartFs::add_document_relationship` (opc/mod.rs:772) L915; `PartFs::add_content_type_override` (opc/mod.rs:616) L916
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.as_ref`, `.clone`, `.cloned`, `.collect`, `.enumerate`, `.filter_map`, `.get`, `.insert`, `.into_bytes`, `.is_empty`, `.is_none`, `.iter`, `.map_or_else`, `.map`, `.then_some`, `.to_ascii_uppercase`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `.values`, `HashSet::new`, `Some`, `W15::name`, `W::author`, `W::date`, `u32::from_str_radix`
- **Macros:** `format!`
- **Called by:** `append_documents` (append.rs:152) (?); `Transaction::write_comments_part` (edit.rs:5190) (?); `threads` (markdown/package.rs:73) (?); `active::reset` (perf.rs:240) (?); `active::set_atom_count` (perf.rs:277) (?); `active::set_unit_count` (perf.rs:281) (?); `renumber` (revision_processor/annotation_ids.rs:22) (?); `prune_orphan_comments` (revision_processor/comments.rs:66) (?); `prune_by_para` (revision_processor/comments.rs:195) (?); `prune_people` (revision_processor/comments.rs:251) (?); `prune_orphan_notes` (revision_processor/notes.rs:24) (?); `tidy` (revision_processor/word_save.rs:25) (?); +4 more

#### `CommentFamily::ensure_part`  (L923-938)
`fn ensure_part(&mut self, k: usize, xml: impl Fn(&str) -> String) -> NodeId`

- **Does:** Create part `k` from `xml(mc namespace)` when missing; its root.
- **Calls (crate):** `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L927; `deeper_boundary_tests::xml` (comparer/comments.rs:1829) L929; `Dom::root` (xmllinq/mod.rs:822) L931
- **Calls (std/external/unresolved):** `.expect`, `.to_string`, `Some`
- **Macros:** `format!`
- **Called by:** `CommentFamily::store` (comments.rs:798)

#### `CommentFamily::stamp_para_ids`  (L942-973)
`fn stamp_para_ids(&mut self, pkg: &PartFs)`

- **Does:** Give every comment paragraph without one a `w14:paraId` unique in the package, and declare `w14` on the comments root.
- **Calls (crate):** `package_para_ids` (comments.rs:1119) L949; `Dom::descendants` (xmllinq/mod.rs:668) L950; `Dom::attribute` (xmllinq/mod.rs:874) L951,962; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L956; `allocate_hex8` (comments.rs:1103) L968; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L969; `declare_ignorable` (comments.rs:1143) L972
- **Calls (std/external/unresolved):** `.clone`, `.enumerate`, `.insert`, `.into_iter`, `.is_some`, `.to_ascii_uppercase`, `.wrapping_add`, `.wrapping_mul`, `Some`, `W14::name`, `W::p`
- **Called by:** `CommentFamily::store` (comments.rs:798)

#### `CommentFamily::store_people`  (L977-1023)
`fn store_people(&self, pkg: &mut PartFs, main: &str, remaining: &HashSet<String>)`

- **Does:** Drop `people.xml` rows of removed authors who no longer comment, and add new authors, when the part exists; never create it.
- **Calls (crate):** `related_part` (comments.rs:1027) L978; `PartFs::part_string` (opc/mod.rs:498) L981; `Dom::new` (xmllinq/mod.rs:387) L984; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L985; `Dom::root` (xmllinq/mod.rs:822) L986; `Dom::descendants` (xmllinq/mod.rs:668) L990,1016; `Dom::attribute` (xmllinq/mod.rs:874) L991; `Dom::remove` (xmllinq/mod.rs:1102) L993; `Dom::new_element` (xmllinq/mod.rs:478) L1003,1005; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1004,1006,1007; `Dom::add` (xmllinq/mod.rs:1058) L1008,1009; `PartFs::remove_part` (opc/mod.rs:527) L1017; `PartFs::remove_content_type_override` (opc/mod.rs:668) L1018; `PartFs::remove_relationships_by_type` (opc/mod.rs:675) L1019; `PartFs::set_part` (opc/mod.rs:510) L1021; `Dom::serialize_document` (xmllinq/mod.rs:1322) L1021
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.insert`, `.into_bytes`, `.is_empty`, `.to_string`, `.unwrap_or`, `HashSet::new`, `Some`, `W15::name`
- **Macros:** `format!`
- **Called by:** `CommentFamily::store` (comments.rs:798); `people_and_text_boundary_tests::people_updates_preserve_existing_presence_and_add_each_remaining_author_once` (comments.rs:1217) (?); `people_and_text_boundary_tests::removing_the_last_person_removes_its_part_type_and_relationship` (comments.rs:1253) (?); `people_and_text_boundary_tests::unchanged_people_are_byte_identical_and_new_authors_do_not_create_a_people_part` (comments.rs:1276) (?)

#### `related_part`  (L1027-1034)
`fn related_part(pkg: &PartFs, main: &str, rel_type: &str) -> Option<String>`

- **Does:** The part the main part relates to with exactly `rel_type`.
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:756) L1028; `PartFs::resolve_rel_target` (opc/mod.rs:600) L1032; `PartFs::part_string` (opc/mod.rs:498) L1033
- **Calls (std/external/unresolved):** `.as_deref`, `.filter`, `.find`, `.is_some`, `.iter`, `.map`, `Some`
- **Called by:** `CommentFamily::load` (comments.rs:429); `CommentFamily::store` (comments.rs:798); `CommentFamily::store_people` (comments.rs:977)

#### `last_para_id`  (L1037-1041)
`fn last_para_id(dom: &Dom, comment: NodeId) -> Option<String>`

- **Does:** `w14:paraId` of a comment's last paragraph.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1038; `Dom::attribute` (xmllinq/mod.rs:874) L1039
- **Calls (std/external/unresolved):** `.copied`, `.last`, `.map`, `Some`, `W14::name`, `W::p`
- **Called by:** `CommentFamily::read_meta` (comments.rs:457); `CommentFamily::store` (comments.rs:798); `people_and_text_boundary_tests::comment_projection_ignores_tab_stops_and_foreign_elements_but_keeps_breaks` (comments.rs:1304); `people_and_text_boundary_tests::adopted_comments_preserve_text_and_drop_old_paragraph_ids` (comments.rs:1316)

#### `comment_text`  (L1045-1069)
`fn comment_text(dom: &Dom, comment: NodeId) -> String`

- **Does:** A comment's text: `w:t` as is, breaks and paragraphs as `\n`, tabs as `\t`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1047,1055; `Dom::name` (xmllinq/mod.rs:526) L1056; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1056; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1059; `Dom::value` (xmllinq/mod.rs:1169) L1060; `Dom::ancestors` (xmllinq/mod.rs:770) L1062
- **Calls (std/external/unresolved):** `.enumerate`, `.filter`, `.into_iter`, `.is_empty`, `.push_str`, `.push`, `Some`, `String::new`, `W::p_pr`, `W::p`
- **Called by:** `CommentFamily::record` (comments.rs:578); `people_and_text_boundary_tests::comment_projection_ignores_tab_stops_and_foreign_elements_but_keeps_breaks` (comments.rs:1304); `people_and_text_boundary_tests::adopted_comments_preserve_text_and_drop_old_paragraph_ids` (comments.rs:1316)

#### `fill_comment_paragraph`  (L1073-1090)
`fn fill_comment_paragraph(dom: &mut Dom, p: NodeId, text: &str)`

- **Does:** The annotation-reference run, then one run per line of `text`, a `w:br` before every line after the first.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L1074,1075,1079,1081,1084; `Dom::add` (xmllinq/mod.rs:1058) L1076,1077,1082,1087,1088; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1085; `XNamespace::xml` (xmllinq/mod.rs:128) L1085; `Dom::add_text` (xmllinq/mod.rs:1071) L1086
- **Calls (std/external/unresolved):** `.enumerate`, `.name`, `.split`, `Some`, `W::name`, `W::r`, `W::t`
- **Called by:** `CommentFamily::add` (comments.rs:604); `CommentFamily::set_text` (comments.rs:737)

#### `clear_rows`  (L1092-1096)
`fn clear_rows(dom: &mut Dom, root: NodeId, row: &XName)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1093; `Dom::remove` (xmllinq/mod.rs:1102) L1094
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `CommentFamily::store` (comments.rs:798)

#### `is_hex8`  (L1098-1100)
`fn is_hex8(value: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.is_ascii_hexdigit`, `.len`
- **Called by:** `CommentFamily::store` (comments.rs:798); `package_para_ids` (comments.rs:1119)

#### `allocate_hex8`  (L1103-1115)
`fn allocate_hex8(used: &mut HashSet<String>, seed: u32) -> String`

- **Does:** A paraId (or durable id) below `0x80000000`, nonzero, not in `used`, from `seed` on.
- **Calls (std/external/unresolved):** `.clone`, `.insert`
- **Macros:** `format!`
- **Called by:** `CommentFamily::store` (comments.rs:798); `CommentFamily::stamp_para_ids` (comments.rs:942); `tests::para_ids_stay_below_the_word_bound_and_skip_used_ones` (comments.rs:1169)

#### `package_para_ids`  (L1119-1139)
`fn package_para_ids(pkg: &PartFs, skip: &str) -> HashSet<String>`

- **Does:** Every `paraId="XXXXXXXX"` value in the package's XML parts other than `skip`, uppercased: Word wants paraIds unique across the document.
- **Calls (crate):** `PartFs::parts` (opc/mod.rs:532) L1121; `PartFs::part_string` (opc/mod.rs:498) L1125; `is_hex8` (comments.rs:1098) L1132
- **Calls (std/external/unresolved):** `.as_str`, `.ends_with`, `.find`, `.get`, `.insert`, `.to_ascii_uppercase`, `HashSet::new`
- **Called by:** `CommentFamily::stamp_para_ids` (comments.rs:942); `authored_thread_reference_contract_tests::paragraph_id_scan_reserves_only_complete_hex_ids_from_other_xml_parts` (comments.rs:1580)

#### `declare_ignorable`  (L1143-1161)
`fn declare_ignorable(dom: &mut Dom, root: NodeId, prefix: &str, uri: &str)`

- **Does:** Declare `prefix` on `root` and list it in `mc:Ignorable`, declaring `mc` too, unless already done.
- **Calls (crate):** `XNamespace::xmlns` (xmllinq/mod.rs:122) L1144; `Dom::attribute` (xmllinq/mod.rs:874) L1145,1148,1152; `XNamespace::name` (xmllinq/mod.rs:134) L1145,1146,1148,1149; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1146,1149,1159
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.is_none`, `.split_whitespace`, `.to_string`, `.trim`, `.unwrap_or`, `MC::name`, `Some`
- **Macros:** `format!`
- **Called by:** `CommentFamily::stamp_para_ids` (comments.rs:942); `tests::ignorable_prefixes_are_appended_once` (comments.rs:1184)

## Test functions (18)

- `tests::para_ids_stay_below_the_word_bound_and_skip_used_ones` (L1169): _no doc comment_
- `tests::tail_counts_characters_not_bytes` (L1178): _no doc comment_
- `tests::ignorable_prefixes_are_appended_once` (L1184): _no doc comment_
- `people_and_text_boundary_tests::pkg` (L1203): _no doc comment_
- `people_and_text_boundary_tests::people` (L1207): _no doc comment_
- `people_and_text_boundary_tests::people_updates_preserve_existing_presence_and_add_each_remaining_author_once` (L1217): _no doc comment_
- `people_and_text_boundary_tests::removing_the_last_person_removes_its_part_type_and_relationship` (L1253): _no doc comment_
- `people_and_text_boundary_tests::unchanged_people_are_byte_identical_and_new_authors_do_not_create_a_people_part` (L1276): _no doc comment_
- `people_and_text_boundary_tests::comment_projection_ignores_tab_stops_and_foreign_elements_but_keeps_breaks` (L1304): _no doc comment_
- `people_and_text_boundary_tests::adopted_comments_preserve_text_and_drop_old_paragraph_ids` (L1316): _no doc comment_
- `public_memory_anchor_boundary_tests::package` (L1347): _no doc comment_
- `public_memory_anchor_boundary_tests::snapshot` (L1353): _no doc comment_
- `public_memory_anchor_boundary_tests::paragraph` (L1365): _no doc comment_
- `public_memory_anchor_boundary_tests::block_and_point_comments_preserve_exact_unicode_anchor_context_and_package` (L1371): _no doc comment_
- `public_memory_anchor_boundary_tests::latest_thread_selection_preserves_complete_winning_source_records_and_thread_order` (L1460): _no doc comment_
- `authored_thread_reference_contract_tests::family` (L1504): _no doc comment_
- `authored_thread_reference_contract_tests::cyclic_and_missing_parent_metadata_terminate_without_discarding_comment_owners` (L1548): _no doc comment_
- `authored_thread_reference_contract_tests::paragraph_id_scan_reserves_only_complete_hex_ids_from_other_xml_parts` (L1580): _no doc comment_
