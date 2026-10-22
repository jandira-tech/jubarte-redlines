# Functions and methods in `src/comments.rs`

43 non-test, 19 test. Part of the [symbol manifest](../README.md).

#### `CommentError::code`  (L95-100, pub)
`pub fn code(&self) -> &'static str`

- **Does:** Stable error code, as the edit API reports it: the admission code, or `INVALID_DOCUMENT` for an unreadable package.
- **Calls (crate):** `.code` (ambiguous, 4 candidates) L97
- **Called by:** none found in src

#### `<CommentError as fmt::Display>::fmt`  (L104-109, impl fmt::Display)
`fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`

- **Does:** _no doc comment_
- **Macros:** `write!`
- **Called by:** none found in src

#### `<CommentError as From>::from`  (L115-120, impl From)
`fn from(error: crate::inspect::InspectError) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.to_string`, `CommentError::Admission`, `CommentError::Package`
- **Called by:** none found in src

#### `list_comments`  (L132-134, pub)
`pub fn list_comments(docx: &[u8]) -> Result<Vec<CommentRecord>, CommentError>`

- **Does:** Every comment in `comments.xml` order, with its thread, resolution and anchor.
- **Calls (crate):** `comments_in` (comments.rs:138) L133; `Opened::open` (inspect.rs:735) L133
- **Called by:** `run_comments` (bin/jubarte.rs:527); `settlement_notes` (bin/jubarte.rs:2120); `resolve_with_report` (changes.rs:822); `public_memory_anchor_boundary_tests::block_and_point_comments_preserve_exact_unicode_anchor_context_and_package` (comments.rs:1427); `public_memory_anchor_boundary_tests::reversed_ranges_record_both_endpoints` (comments.rs:1516); `authored_thread_reference_contract_tests::cyclic_and_missing_parent_metadata_terminate_without_discarding_comment_owners` (comments.rs:1629); `coverage_real_revision_merge_matrix_tests::organic_source_snapshot` (comparer/finalize.rs:26897); `AnchorChanges::records` (edit/comment_settlement.rs:78); `changed_selection` (edit/for_edit.rs:703); `for_edit` (edit/for_edit.rs:1581); `tests::inline_insert_edges_and_paragraph_sides_use_explicit_kinds` (edit/inline.rs:535); `comments` (edit/v2_apply.rs:647); +1 more

#### `comments_in`  (L138-158, pub(crate))
`pub(crate) fn comments_in(opened: &mut Opened) -> Result<Vec<CommentRecord>, CommentError>`

- **Does:** [`list_comments`] of a package already open.
- **Calls (crate):** `CommentFamily::load` (comments.rs:457) L139; `Anchors::scan` (comments.rs:251) L144,147; `Opened::story_parts` (inspect.rs:861) L145; `parse_part` (inspect.rs:926) L146; `CommentFamily::order` (comments.rs:568) (by name) L149; `.record` (ambiguous: `CommentFamily::record` (comments.rs:612); `FontIndex::record` (convert/font.rs:3192)) L153; `Anchors::fill` (comments.rs:352) L154
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.is_empty`, `.map_err`, `.map`, `Anchors::default`, `Ok`, `Vec::new`
- **Called by:** `list_comments` (comments.rs:132); `snapshot` (inspect.rs:403)

#### `comments_text`  (L164-185, pub)
`pub fn comments_text(comments: &[CommentRecord]) -> String`

- **Does:** `jubarte comments`' listing: one line per comment, `c{id}\t{paragraph} \t{author}\t{text}\ton {anchor}[\treply to c{parent}][\tresolved]` (`-` for an absent paragraph; text and anchor quoted, their first 60 and 40 characters), then `{n} comment(s)`.
- **Calls (std/external/unresolved):** `.as_deref`, `.chars`, `.collect`, `.len`, `.map_or_else`, `.map`, `.push_str`, `.take`, `.to_string`, `.unwrap_or_default`, `String::new`
- **Macros:** `format!`
- **Called by:** `run_comments` (bin/jubarte.rs:527)

#### `select_comments`  (L190-233, pub)
`pub fn select_comments( records: Vec<CommentRecord>, author: Option<&str>, latest: bool, ) -> Vec<CommentRecord>`

- **Does:** Filter a listing: `author` keeps that author's comments (exact match); `latest` keeps one comment per thread, the newest by `date` (later in the listing on a tie), threads in the order their first comment appears.
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.collect`, `.filter`, `.find`, `.get`, `.insert`, `.into_iter`, `.is_none_or`, `.iter_mut`, `.iter`, `.map`, `.push`, `HashSet::new`, `Vec::new`, `root_of`
- **Called by:** `run_comments` (bin/jubarte.rs:527); `public_memory_anchor_boundary_tests::latest_thread_selection_preserves_complete_winning_source_records_and_thread_order` (comments.rs:1541); `authored_thread_reference_contract_tests::cyclic_and_missing_parent_metadata_terminate_without_discarding_comment_owners` (comments.rs:1629)

#### `Anchors::scan`  (L251-350)
`fn scan(&mut self, story_id: &str, dom: &mut Dom, root: NodeId, body: bool)`

- **Does:** Locate every marker of one story.
- **Calls (crate):** `body_paragraph_nodes` (inspect.rs:988) L254; `story_paragraph_nodes` (inspect.rs:1178) L256; `Dom::descendants` (xmllinq/mod.rs:668) L263,279; `Dom::ancestors` (xmllinq/mod.rs:770) L270; `Dom::attribute` (xmllinq/mod.rs:874) L280; `paragraph_of` (ambiguous: `paragraph_of` (changes.rs:1388); `paragraph_of` (comparer/bookmarks.rs:76)) L283; `Dom::new_element` (xmllinq/mod.rs:478) L297,298; `Dom::add_text` (xmllinq/mod.rs:1087) L299; `Dom::add` (xmllinq/mod.rs:1074) L300; `Dom::add_before_self` (xmllinq/mod.rs:1130) L301; `project_paragraph` (inspect.rs:1265) L322
- **Calls (std/external/unresolved):** `.chars`, `.clone`, `.collect`, `.copied`, `.entry`, `.enumerate`, `.find_map`, `.get`, `.insert`, `.into_iter`, `.iter`, `.len`, `.map`, `.or_insert`, `.position`, `.push`, `.rposition`, `.to_string`, `.unwrap_or`, `HashMap::new`, `Some`, `String::with_capacity`, `Vec::new`, `Vec::with_capacity`, `W::id`, `W::name`, `W::p`, `W::r`, `W::t`, `char::from_u32`
- **Called by:** `comments_in` (comments.rs:138)

#### `Anchors::fill`  (L352-404)
`fn fill(&self, record: &mut CommentRecord)`

- **Does:** _no doc comment_
- **Calls (crate):** `paragraph_id` (ids.rs:410) L357; `paragraph_id` (ids.rs:410) L363,364,381,382,400; `tail` (comments.rs:408) L366,395
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.copied`, `.get`, `.push_str`, `.push`, `.take`, `.then`, `.to_string`, `Some`
- **Called by:** `comments_in` (comments.rs:138); `bidi_visual_line` (convert/mod.rs:32736) (?); `subset_keep_gids` (convert/pdf.rs:1185) (?); `raster_payload_boundary_tests::white` (convert/raster.rs:910) (?); `residual_display_contract_tests::white` (convert/raster.rs:1142) (?); `without_emphasis` (markdown/anchor.rs:147) (?); `wrap_markers` (markdown/from_docx/critic.rs:1040) (?); `foreign_alignment_budget_tests::foreign_alignment_dense_fallback_is_order_stable` (validate.rs:4682) (?)

#### `tail`  (L408-411)
`fn tail(text: &str, n: usize) -> String`

- **Does:** The last `n` characters of `text`.
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.count`, `.saturating_sub`, `.skip`
- **Called by:** `Anchors::fill` (comments.rs:352); `tests::tail_counts_characters_not_bytes` (comments.rs:1234)

#### `CommentFamily::load`  (L457-483, pub(crate))
`pub(crate) fn load(pkg: &PartFs, main: &str) -> Result<Self, String>`

- **Does:** Load the family the main part `main` relates to.
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L458; `related_part` (comments.rs:1083) L461; `PartFs::part_string` (opc/mod.rs:636) L464; `validate_xml` (xmllinq/parse.rs:1052) L467; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L468; `Dom::root` (xmllinq/mod.rs:822) L469
- **Calls (std/external/unresolved):** `.enumerate`, `.iter`, `.map_err`, `.ok_or_else`, `.read_meta`, `BTreeMap::new`, `HashSet::new`, `Ok`, `Some`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `append_documents` (append.rs:152); `stage_comments` (append.rs:563); `comments_in` (comments.rs:138); `people_and_text_boundary_tests::people_updates_preserve_existing_presence_and_add_each_remaining_author_once` (comments.rs:1273); `people_and_text_boundary_tests::removing_the_last_person_removes_its_part_type_and_relationship` (comments.rs:1309); `people_and_text_boundary_tests::unchanged_people_are_byte_identical_and_new_authors_do_not_create_a_people_part` (comments.rs:1332); `people_and_text_boundary_tests::adopted_comments_preserve_text_and_drop_old_paragraph_ids` (comments.rs:1372); `authored_thread_reference_contract_tests::family` (comments.rs:1585); `Origin::had_comment` (edit/origin.rs:252); `SourceModel::entity` (edit/v2_apply.rs:836); `SourceModel::operation` (edit/v2_apply.rs:1388); `Transaction::start` (edit.rs:2680); +3 more

#### `CommentFamily::read_meta`  (L485-549)
`fn read_meta(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) (by name) L492; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L494,508,518,522,524; `rows` (ambiguous: `shared_word_alignment_tests::rows` (comparer/lcs_table.rs:1887); `rows` (convert/word_subst.rs:130); `tests::rows` (edit/structural.rs:487)) L498,499,500; `XName::get` (xmllinq/mod.rs:180) L499,502,503,518; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L507; `last_para_id` (comments.rs:1093) L514
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.collect`, `.copied`, `.eq_ignore_ascii_case`, `.filter_map`, `.filter`, `.get_mut`, `.get`, `.insert`, `.into_iter`, `.is_some_and`, `.map_or_else`, `.map`, `.ok`, `.parse`, `.push`, `.to_string`, `HashMap::new`, `Some`, `Vec::new`, `W15::name`, `W::id`, `W::name`
- **Called by:** none found in src

#### `CommentFamily::is_empty`  (L552-554, pub(crate))
`pub(crate) fn is_empty(&self) -> bool`

- **Does:** No comment is defined.
- **Calls (std/external/unresolved):** `.is_empty`
- **Called by:** none found in src

#### `CommentFamily::contains`  (L557-559, pub(crate))
`pub(crate) fn contains(&self, id: u32) -> bool`

- **Does:** Whether comment `id` is defined.
- **Calls (std/external/unresolved):** `.contains_key`
- **Called by:** none found in src

#### `CommentFamily::author`  (L562-565, pub(crate))
`pub(crate) fn author(&self, id: u32) -> Option<&str>`

- **Does:** The `w:author` comment `id` is recorded under, when it names one.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L564
- **Calls (std/external/unresolved):** `.get`, `W::name`
- **Called by:** `Transaction::not_own_comment` (edit.rs:3260)

#### `CommentFamily::order`  (L568-582, pub(crate))
`pub(crate) fn order(&self) -> Vec<u32>`

- **Does:** Comment ids in `comments.xml` order.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L572; `Dom::attribute` (xmllinq/mod.rs:874) L576
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.contains_key`, `.filter_map`, `.filter`, `.into_iter`, `.ok`, `.parse`, `Some`, `Vec::new`, `W::id`, `W::name`
- **Called by:** `stage_comments` (append.rs:563); `comments_in` (comments.rs:138) (?); `CommentFamily::add` (comments.rs:648); `CommentFamily::store` (comments.rs:854); `check_comment_anchors` (edit/v2_apply.rs:1913) (?)

#### `CommentFamily::thread_root`  (L585-594, pub(crate))
`pub(crate) fn thread_root(&self, mut id: u32) -> u32`

- **Does:** The first comment of `id`'s thread.
- **Calls (std/external/unresolved):** `.and_then`, `.get`, `.insert`, `HashSet::new`
- **Called by:** `CommentFamily::add` (comments.rs:648); `CommentFamily::set_thread` (comments.rs:819); `authored_thread_reference_contract_tests::cyclic_and_missing_parent_metadata_terminate_without_discarding_comment_owners` (comments.rs:1629) (?); `compose_threads` (edit/v2_apply.rs:1821) (?)

#### `CommentFamily::with_replies`  (L597-610, pub(crate))
`pub(crate) fn with_replies(&self, id: u32) -> Vec<u32>`

- **Does:** `id` and every comment that replies to it, directly or not.
- **Calls (std/external/unresolved):** `.contains`, `.len`, `.push`, `Some`
- **Macros:** `vec!`
- **Called by:** `CommentFamily::set_done` (comments.rs:830); `CommentFamily::remove` (comments.rs:839); `authored_thread_reference_contract_tests::cyclic_and_missing_parent_metadata_terminate_without_discarding_comment_owners` (comments.rs:1629) (?); `thread_markers` (edit/tracked.rs:956) (?); `SourceModel::operation` (edit/v2_apply.rs:1388) (?); `check_comment_anchors` (edit/v2_apply.rs:1913) (?); `Transaction::resolve_thread` (edit.rs:4379) (?); `Transaction::check_thread_ops` (edit.rs:5715) (?); `Transaction::apply_reply_markers` (edit.rs:6381) (?)

#### `CommentFamily::record`  (L612-634)
`fn record(&self, id: u32) -> CommentRecord`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L615; `attr` (ambiguous: `attr` (convert/altchunk.rs:47); `attr` (debug/diff.rs:468); `attr` (debug.rs:641)) L622,623; `CommentFamily::reported_date` (comments.rs:637) L624; `comment_text` (comments.rs:1101) L625
- **Calls (std/external/unresolved):** `.map`, `.unwrap_or_default`, `String::new`, `W::name`
- **Macros:** `format!`
- **Called by:** `comments_in` (comments.rs:138) (?)

#### `CommentFamily::reported_date`  (L637-644, pub(crate))
`pub(crate) fn reported_date(&self, id: u32) -> Option<String>`

- **Does:** The comment instant, sharing the revision reader's offset and fallback rule.
- **Calls (crate):** `reported_date` (changes.rs:1093) L639; `Dom::attribute` (xmllinq/mod.rs:874) L640,642; `XName::get` (xmllinq/mod.rs:180) L642
- **Calls (std/external/unresolved):** `.and_then`, `.get`, `W::date`
- **Called by:** `CommentFamily::record` (comments.rs:612); `convert_marks` (markdown/from_docx/mod.rs:108) (?)

#### `CommentFamily::add`  (L648-685, pub(crate))
`pub(crate) fn add(&mut self, new: &NewComment<'_>)`

- **Does:** Add a comment; a reply goes right after the last comment of its thread and hangs off the thread's first comment.
- **Calls (crate):** `CommentFamily::comments_root` (comments.rs:688) L649; `Dom::new_element` (xmllinq/mod.rs:478) (by name) L651,656; `Dom::set_attribute_value` (xmllinq/mod.rs:977) (by name) L652,653,654,655; `fill_comment_paragraph` (comments.rs:1129) L657; `.add` (ambiguous, 5 candidates) L658; `CommentFamily::thread_root` (comments.rs:585) L659,664; `CommentFamily::order` (comments.rs:568) L661; `Dom::add_after_self` (xmllinq/mod.rs:1147) L668; `Dom::add` (xmllinq/mod.rs:1074) L669
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.find`, `.insert`, `.into_iter`, `.iter`, `.map`, `.push`, `.rev`, `.to_string`, `Some`, `W::author`, `W::date`, `W::id`, `W::name`, `W::p`
- **Called by:** none found in src

#### `CommentFamily::comments_root`  (L688-704)
`fn comments_root(&mut self) -> NodeId`

- **Does:** The root of `comments.xml`, created when the family has none.
- **Calls (crate):** `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L692; `Dom::root` (xmllinq/mod.rs:822) L697
- **Calls (std/external/unresolved):** `.expect`, `.to_string`, `Some`
- **Macros:** `format!`
- **Called by:** `CommentFamily::add` (comments.rs:648); `CommentFamily::adopt` (comments.rs:743)

#### `CommentFamily::part_name`  (L708-712, pub(crate))
`pub(crate) fn part_name(&self) -> String`

- **Does:** The comments part's name: the one the main part relates to, or the name [`Self::store`] gives a new one.
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.map_or_else`, `.to_string`
- **Called by:** `append_documents` (append.rs:152) (?)

#### `CommentFamily::thread_state`  (L716-723, pub(crate))
`pub(crate) fn thread_state(&self, id: u32) -> Option<(Option<u32>, bool, Option<String>)>`

- **Does:** Comment `id`'s thread parent, resolution and UTC date (`w16cex:dateUtc`), when `id` is defined.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L720; `XName::get` (xmllinq/mod.rs:180) L720
- **Calls (std/external/unresolved):** `.and_then`, `.get`, `.map`, `Some`
- **Called by:** `stage_comments` (append.rs:563) (?); `people_and_text_boundary_tests::adopted_comments_preserve_text_and_drop_old_paragraph_ids` (comments.rs:1372) (?); `compose_threads` (edit/v2_apply.rs:1821) (?)

#### `CommentFamily::set_date_utc`  (L726-735, pub(crate))
`pub(crate) fn set_date_utc(&mut self, id: u32, date: &str)`

- **Does:** Set a linked UTC date before storing the existing comment family.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L732; `XName::get` (xmllinq/mod.rs:180) L732,734; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L733
- **Calls (std/external/unresolved):** `.get_mut`, `.get_or_insert_with`, `Some`
- **Called by:** `threads` (markdown/package.rs:74) (?)

#### `CommentFamily::adopt`  (L743-789, pub(crate))
`pub(crate) fn adopt( &mut self, xml: &str, id: u32, parent: Option<u32>, done: bool, date_utc: Option<&str>, )`

- **Does:** Add a comment copied whole from another package (its formatting, line breaks and links stay): `xml` is its `w:comment` element with the namespaces it uses declared on it.
- **Calls (crate):** `CommentFamily::comments_root` (comments.rs:688) L751; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) (by name) L753; `.root` (ambiguous: `Dom::root` (xmllinq/mod.rs:822); `Scope::root` (xmllinq/serialize.rs:211)) L754; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L756; `Dom::set_attribute_value` (xmllinq/mod.rs:977) (by name) L761,763,764,769; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L762; `.add` (ambiguous, 5 candidates) L766; `Dom::new_element` (xmllinq/mod.rs:478) (by name) L768; `XName::get` (xmllinq/mod.rs:180) L768,769; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L772
- **Calls (std/external/unresolved):** `.and_then`, `.contains_key`, `.contains`, `.copied`, `.filter`, `.first`, `.insert`, `.map`, `.push`, `.remove`, `.to_string`, `Some`, `W14::name`, `W::author`, `W::id`, `W::name`, `W::p`
- **Macros:** `format!`
- **Called by:** `append_documents` (append.rs:152) (?); `people_and_text_boundary_tests::adopted_comments_preserve_text_and_drop_old_paragraph_ids` (comments.rs:1372) (?)

#### `CommentFamily::set_text`  (L793-814, pub(crate))
`pub(crate) fn set_text(&mut self, id: u32, text: &str)`

- **Does:** Replace a comment's text.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) (by name) L799; `Dom::new_element` (xmllinq/mod.rs:478) (by name) L803; `.nodes` (ambiguous: `Writer::nodes` (markdown/math.rs:1728); `Dom::nodes` (xmllinq/mod.rs:604)) L804,807; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L808; `fill_comment_paragraph` (comments.rs:1129) L812; `.add` (ambiguous, 5 candidates) L813
- **Calls (std/external/unresolved):** `.copied`, `.get`, `.last`, `.remove`, `.unwrap_or_else`, `Some`, `W::p_pr`, `W::p`
- **Called by:** `Transaction::write_comments_part` (edit.rs:6761) (?)

#### `CommentFamily::set_thread`  (L819-827, pub(crate))
`pub(crate) fn set_thread(&mut self, id: u32, parent: Option<u32>, done: bool)`

- **Does:** Set comment `id`'s thread: the comment it replies to (it hangs off that comment's thread root, Word threads being one level deep) and whether it is resolved.
- **Calls (crate):** `CommentFamily::thread_root` (comments.rs:585) L822
- **Calls (std/external/unresolved):** `.contains_key`, `.filter`, `.get_mut`, `.map`
- **Called by:** `threads` (markdown/package.rs:74) (?)

#### `CommentFamily::set_done`  (L830-836, pub(crate))
`pub(crate) fn set_done(&mut self, id: u32, done: bool)`

- **Does:** Mark `id` and its replies resolved (or open again).
- **Calls (crate):** `CommentFamily::with_replies` (comments.rs:597) L831
- **Calls (std/external/unresolved):** `.get_mut`
- **Called by:** `Transaction::write_comments_part` (edit.rs:6761) (?); `threads` (markdown/package.rs:74) (?)

#### `CommentFamily::remove`  (L839-850, pub(crate))
`pub(crate) fn remove(&mut self, id: u32) -> Vec<u32>`

- **Does:** Remove `id` and its replies; the removed ids.
- **Calls (crate):** `CommentFamily::with_replies` (comments.rs:597) L840; `Dom::attribute` (xmllinq/mod.rs:874) L843; `Dom::remove` (xmllinq/mod.rs:1118) L846
- **Calls (std/external/unresolved):** `.insert`, `.remove`, `.to_string`, `W::author`
- **Called by:** none found in src

#### `CommentFamily::store`  (L854-976, pub(crate))
`pub(crate) fn store(mut self, pkg: &mut PartFs, main: &str)`

- **Does:** Write the family back into `pkg`: every part present and consistent, or none of them when no comment is left; `people.xml` follows.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L858,929,955; `PartFs::remove_part` (opc/mod.rs:710) L865; `PartFs::remove_content_type_override` (opc/mod.rs:897) L866; `PartFs::remove_relationships_by_type` (opc/mod.rs:904) L867; `CommentFamily::store_people` (comments.rs:1033) L869,975; `CommentFamily::stamp_para_ids` (comments.rs:998) L872; `CommentFamily::order` (comments.rs:568) L873; `last_para_id` (comments.rs:1093) L876; `CommentFamily::ensure_part` (comments.rs:979) L879,905,910; `clear_rows` (comments.rs:1148) L885,915,916; `Dom::new_element` (xmllinq/mod.rs:478) L890,943,951; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L893,898,900,944,946,952...; `Dom::add` (xmllinq/mod.rs:1074) L902,948,963; `XName::get` (xmllinq/mod.rs:180) L915,919,921,943,945,951...; `is_hex8` (comments.rs:1154) L931; `allocate_hex8` (comments.rs:1159) L939; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L956; `Dom::serialize_document` (xmllinq/mod.rs:1338) L967; `PartFs::set_part` (opc/mod.rs:661) L968; `related_part` (comments.rs:1083) L969; `relative_rel_target` (opc/mod.rs:180) L970; `PartFs::add_document_relationship` (opc/mod.rs:1003) L971; `PartFs::add_content_type_override` (opc/mod.rs:845) L972
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.as_ref`, `.clone`, `.cloned`, `.collect`, `.enumerate`, `.filter_map`, `.get`, `.insert`, `.into_bytes`, `.is_empty`, `.is_none`, `.iter`, `.map_or_else`, `.map`, `.then_some`, `.to_ascii_uppercase`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `.values`, `HashSet::new`, `Some`, `W15::name`, `W::author`, `W::date`, `u32::from_str_radix`
- **Macros:** `format!`
- **Called by:** `append_documents` (append.rs:152) (?); `cli_main` (bin/jubarte.rs:3935) (?); `Transaction::write_comments_part` (edit.rs:6761) (?); `threads` (markdown/package.rs:74) (?); `active::reset` (perf.rs:240) (?); `active::set_atom_count` (perf.rs:277) (?); `active::set_unit_count` (perf.rs:281) (?); `prune_orphan_comments` (revision_processor/comments.rs:274) (?); `prune_by_para` (revision_processor/comments.rs:411) (?); `prune_people` (revision_processor/comments.rs:467) (?); `prune_orphan_notes` (revision_processor/notes.rs:24) (?); `tidy` (revision_processor/word_save.rs:25) (?); +4 more

#### `CommentFamily::ensure_part`  (L979-994)
`fn ensure_part(&mut self, k: usize, xml: impl Fn(&str) -> String) -> NodeId`

- **Does:** Create part `k` from `xml(mc namespace)` when missing; its root.
- **Calls (crate):** `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L983; `deeper_boundary_tests::xml` (comparer/comments.rs:1829) L985; `Dom::root` (xmllinq/mod.rs:822) L987
- **Calls (std/external/unresolved):** `.expect`, `.to_string`, `Some`
- **Macros:** `format!`
- **Called by:** `CommentFamily::store` (comments.rs:854)

#### `CommentFamily::stamp_para_ids`  (L998-1029)
`fn stamp_para_ids(&mut self, pkg: &PartFs)`

- **Does:** Give every comment paragraph without one a `w14:paraId` unique in the package, and declare `w14` on the comments root.
- **Calls (crate):** `package_para_ids` (comments.rs:1175) L1005; `Dom::descendants` (xmllinq/mod.rs:668) L1006; `Dom::attribute` (xmllinq/mod.rs:874) L1007,1018; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L1012; `allocate_hex8` (comments.rs:1159) L1024; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1025; `declare_ignorable` (comments.rs:1199) L1028
- **Calls (std/external/unresolved):** `.clone`, `.enumerate`, `.insert`, `.into_iter`, `.is_some`, `.to_ascii_uppercase`, `.wrapping_add`, `.wrapping_mul`, `Some`, `W14::name`, `W::p`
- **Called by:** `CommentFamily::store` (comments.rs:854)

#### `CommentFamily::store_people`  (L1033-1079)
`fn store_people(&self, pkg: &mut PartFs, main: &str, remaining: &HashSet<String>)`

- **Does:** Drop `people.xml` rows of removed authors who no longer comment, and add new authors, when the part exists; never create it.
- **Calls (crate):** `related_part` (comments.rs:1083) L1034; `PartFs::part_string` (opc/mod.rs:636) L1037; `Dom::new` (xmllinq/mod.rs:387) L1040; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L1041; `Dom::root` (xmllinq/mod.rs:822) L1042; `Dom::descendants` (xmllinq/mod.rs:668) L1046,1072; `Dom::attribute` (xmllinq/mod.rs:874) L1047; `Dom::remove` (xmllinq/mod.rs:1118) L1049; `Dom::new_element` (xmllinq/mod.rs:478) L1059,1061; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1060,1062,1063; `Dom::add` (xmllinq/mod.rs:1074) L1064,1065; `PartFs::remove_part` (opc/mod.rs:710) L1073; `PartFs::remove_content_type_override` (opc/mod.rs:897) L1074; `PartFs::remove_relationships_by_type` (opc/mod.rs:904) L1075; `PartFs::set_part` (opc/mod.rs:661) L1077; `Dom::serialize_document` (xmllinq/mod.rs:1338) L1077
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.insert`, `.into_bytes`, `.is_empty`, `.to_string`, `.unwrap_or`, `HashSet::new`, `Some`, `W15::name`
- **Macros:** `format!`
- **Called by:** `CommentFamily::store` (comments.rs:854); `people_and_text_boundary_tests::people_updates_preserve_existing_presence_and_add_each_remaining_author_once` (comments.rs:1273) (?); `people_and_text_boundary_tests::removing_the_last_person_removes_its_part_type_and_relationship` (comments.rs:1309) (?); `people_and_text_boundary_tests::unchanged_people_are_byte_identical_and_new_authors_do_not_create_a_people_part` (comments.rs:1332) (?)

#### `related_part`  (L1083-1090)
`fn related_part(pkg: &PartFs, main: &str, rel_type: &str) -> Option<String>`

- **Does:** The part the main part relates to with exactly `rel_type`.
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:991) L1084; `PartFs::resolve_rel_target` (opc/mod.rs:829) L1088; `PartFs::part_string` (opc/mod.rs:636) L1089
- **Calls (std/external/unresolved):** `.as_deref`, `.filter`, `.find`, `.is_some`, `.iter`, `.map`, `Some`
- **Called by:** `CommentFamily::load` (comments.rs:457); `CommentFamily::store` (comments.rs:854); `CommentFamily::store_people` (comments.rs:1033)

#### `last_para_id`  (L1093-1097)
`fn last_para_id(dom: &Dom, comment: NodeId) -> Option<String>`

- **Does:** `w14:paraId` of a comment's last paragraph.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1094; `Dom::attribute` (xmllinq/mod.rs:874) L1095
- **Calls (std/external/unresolved):** `.copied`, `.last`, `.map`, `Some`, `W14::name`, `W::p`
- **Called by:** `CommentFamily::read_meta` (comments.rs:485); `CommentFamily::store` (comments.rs:854); `people_and_text_boundary_tests::comment_projection_ignores_tab_stops_and_foreign_elements_but_keeps_breaks` (comments.rs:1360); `people_and_text_boundary_tests::adopted_comments_preserve_text_and_drop_old_paragraph_ids` (comments.rs:1372)

#### `comment_text`  (L1101-1125)
`fn comment_text(dom: &Dom, comment: NodeId) -> String`

- **Does:** A comment's text: `w:t` as is, breaks and paragraphs as `\n`, tabs as `\t`.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L1103,1111; `Dom::name` (xmllinq/mod.rs:526) L1112; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1112; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1115; `Dom::value` (xmllinq/mod.rs:1185) L1116; `Dom::ancestors` (xmllinq/mod.rs:770) L1118
- **Calls (std/external/unresolved):** `.enumerate`, `.filter`, `.into_iter`, `.is_empty`, `.push_str`, `.push`, `Some`, `String::new`, `W::p_pr`, `W::p`
- **Called by:** `CommentFamily::record` (comments.rs:612); `people_and_text_boundary_tests::comment_projection_ignores_tab_stops_and_foreign_elements_but_keeps_breaks` (comments.rs:1360); `people_and_text_boundary_tests::adopted_comments_preserve_text_and_drop_old_paragraph_ids` (comments.rs:1372)

#### `fill_comment_paragraph`  (L1129-1146)
`fn fill_comment_paragraph(dom: &mut Dom, p: NodeId, text: &str)`

- **Does:** The annotation-reference run, then one run per line of `text`, a `w:br` before every line after the first.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L1130,1131,1135,1137,1140; `Dom::add` (xmllinq/mod.rs:1074) L1132,1133,1138,1143,1144; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1141; `XNamespace::xml` (xmllinq/mod.rs:128) L1141; `Dom::add_text` (xmllinq/mod.rs:1087) L1142
- **Calls (std/external/unresolved):** `.enumerate`, `.name`, `.split`, `Some`, `W::name`, `W::r`, `W::t`
- **Called by:** `CommentFamily::add` (comments.rs:648); `CommentFamily::set_text` (comments.rs:793)

#### `clear_rows`  (L1148-1152)
`fn clear_rows(dom: &mut Dom, root: NodeId, row: &XName)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1149; `Dom::remove` (xmllinq/mod.rs:1118) L1150
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `CommentFamily::store` (comments.rs:854)

#### `is_hex8`  (L1154-1156)
`fn is_hex8(value: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.is_ascii_hexdigit`, `.len`
- **Called by:** `CommentFamily::store` (comments.rs:854); `package_para_ids` (comments.rs:1175)

#### `allocate_hex8`  (L1159-1171)
`fn allocate_hex8(used: &mut HashSet<String>, seed: u32) -> String`

- **Does:** A paraId (or durable id) below `0x80000000`, nonzero, not in `used`, from `seed` on.
- **Calls (std/external/unresolved):** `.clone`, `.insert`
- **Macros:** `format!`
- **Called by:** `CommentFamily::store` (comments.rs:854); `CommentFamily::stamp_para_ids` (comments.rs:998); `tests::para_ids_stay_below_the_word_bound_and_skip_used_ones` (comments.rs:1225)

#### `package_para_ids`  (L1175-1195)
`fn package_para_ids(pkg: &PartFs, skip: &str) -> HashSet<String>`

- **Does:** Every `paraId="XXXXXXXX"` value in the package's XML parts other than `skip`, uppercased: Word wants paraIds unique across the document.
- **Calls (crate):** `PartFs::parts` (opc/mod.rs:733) L1177; `PartFs::part_string` (opc/mod.rs:636) L1181; `is_hex8` (comments.rs:1154) L1188
- **Calls (std/external/unresolved):** `.as_str`, `.ends_with`, `.find`, `.get`, `.insert`, `.to_ascii_uppercase`, `HashSet::new`
- **Called by:** `CommentFamily::stamp_para_ids` (comments.rs:998); `authored_thread_reference_contract_tests::paragraph_id_scan_reserves_only_complete_hex_ids_from_other_xml_parts` (comments.rs:1661)

#### `declare_ignorable`  (L1199-1217)
`fn declare_ignorable(dom: &mut Dom, root: NodeId, prefix: &str, uri: &str)`

- **Does:** Declare `prefix` on `root` and list it in `mc:Ignorable`, declaring `mc` too, unless already done.
- **Calls (crate):** `XNamespace::xmlns` (xmllinq/mod.rs:122) L1200; `Dom::attribute` (xmllinq/mod.rs:874) L1201,1204,1208; `XNamespace::name` (xmllinq/mod.rs:134) L1201,1202,1204,1205; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L1202,1205,1215
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.is_none`, `.split_whitespace`, `.to_string`, `.trim`, `.unwrap_or`, `MC::name`, `Some`
- **Macros:** `format!`
- **Called by:** `CommentFamily::stamp_para_ids` (comments.rs:998); `tests::ignorable_prefixes_are_appended_once` (comments.rs:1240)

## Test functions (19)

- `tests::para_ids_stay_below_the_word_bound_and_skip_used_ones` (L1225): _no doc comment_
- `tests::tail_counts_characters_not_bytes` (L1234): _no doc comment_
- `tests::ignorable_prefixes_are_appended_once` (L1240): _no doc comment_
- `people_and_text_boundary_tests::pkg` (L1259): _no doc comment_
- `people_and_text_boundary_tests::people` (L1263): _no doc comment_
- `people_and_text_boundary_tests::people_updates_preserve_existing_presence_and_add_each_remaining_author_once` (L1273): _no doc comment_
- `people_and_text_boundary_tests::removing_the_last_person_removes_its_part_type_and_relationship` (L1309): _no doc comment_
- `people_and_text_boundary_tests::unchanged_people_are_byte_identical_and_new_authors_do_not_create_a_people_part` (L1332): _no doc comment_
- `people_and_text_boundary_tests::comment_projection_ignores_tab_stops_and_foreign_elements_but_keeps_breaks` (L1360): _no doc comment_
- `people_and_text_boundary_tests::adopted_comments_preserve_text_and_drop_old_paragraph_ids` (L1372): _no doc comment_
- `public_memory_anchor_boundary_tests::package` (L1403): _no doc comment_
- `public_memory_anchor_boundary_tests::snapshot` (L1409): _no doc comment_
- `public_memory_anchor_boundary_tests::paragraph` (L1421): _no doc comment_
- `public_memory_anchor_boundary_tests::block_and_point_comments_preserve_exact_unicode_anchor_context_and_package` (L1427): _no doc comment_
- `public_memory_anchor_boundary_tests::reversed_ranges_record_both_endpoints` (L1516): _no doc comment_
- `public_memory_anchor_boundary_tests::latest_thread_selection_preserves_complete_winning_source_records_and_thread_order` (L1541): _no doc comment_
- `authored_thread_reference_contract_tests::family` (L1585): _no doc comment_
- `authored_thread_reference_contract_tests::cyclic_and_missing_parent_metadata_terminate_without_discarding_comment_owners` (L1629): _no doc comment_
- `authored_thread_reference_contract_tests::paragraph_id_scan_reserves_only_complete_hex_ids_from_other_xml_parts` (L1661): _no doc comment_
