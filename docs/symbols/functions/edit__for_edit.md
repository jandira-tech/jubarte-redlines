# Functions and methods in `src/edit/for_edit.rs`

31 non-test, 2 test. Part of the [symbol manifest](../README.md).

#### `<ForEditRequest as Default>::default`  (L84-107, impl Default)
`fn default() -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `tests::cursor_binds_frozen_membership_independently_of_request_filters` (edit/for_edit.rs:1811); `tests::cursor_binds_source_digest_independently_of_selection` (edit/for_edit.rs:1829); `tests::invalid_attribution_never_produces_a_saved_request` (edit/template.rs:306)

#### `TextValue::complete`  (L136-142)
`fn complete(text: String) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `TextValue::bounded` (edit/for_edit.rs:146); `make_row` (edit/for_edit.rs:1293)

#### `TextValue::bounded`  (L146-170)
`fn bounded(text: &str, chars: usize) -> Self`

- **Does:** The value bounded to `chars` Unicode scalar positions; a preview keeps `text` absent so it cannot pose as a complete payload.
- **Calls (crate):** `TextValue::complete` (edit/for_edit.rs:136) L163
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.ends_with`, `.extend`, `.is_some_and`, `.is_whitespace`, `.len`, `.next`, `.take_while`, `.take`, `.to_string`, `Some`
- **Called by:** `bound_row` (edit/for_edit.rs:1266); `context_of` (edit/for_edit.rs:1505)

#### `TextValue::chars`  (L173-178)
`fn chars(&self) -> usize`

- **Does:** Unicode scalar count of the emitted text.
- **Calls (std/external/unresolved):** `.as_deref`, `.chars`, `.count`, `.map_or`, `.or`
- **Called by:** `text_chars` (edit/for_edit.rs:1261); `context_of` (edit/for_edit.rs:1505)

#### `existing_revisions_name`  (L445-452)
`fn existing_revisions_name(policy: ExistingRevisions) -> &'static str`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `basis_digest` (edit/for_edit.rs:456); `for_edit` (edit/for_edit.rs:1581)

#### `basis_digest`  (L456-491)
`fn basis_digest(source_digest: &str, request: &ForEditRequest, selection: &Selection) -> String`

- **Does:** Everything a cursor binds: the source digest, the frozen selection, the filters that produced it, the codec, and the working policy.
- **Calls (crate):** `existing_revisions_name` (edit/for_edit.rs:445) L470; `source_sha256` (inspect.rs:299) (by name) L486
- **Calls (std/external/unresolved):** `.as_bytes`, `.clone`, `.dedup`, `.expect`, `.sort`, `serde_json::to_string`
- **Macros:** `serde_json::json!`
- **Called by:** `for_edit` (edit/for_edit.rs:1581); `tests::cursor_binds_frozen_membership_independently_of_request_filters` (edit/for_edit.rs:1811); `tests::cursor_binds_source_digest_independently_of_selection` (edit/for_edit.rs:1829)

#### `cursor_token`  (L495-500)
`fn cursor_token(basis: &str, offset: usize) -> String`

- **Does:** The cursor token: a version, the bound basis digest and the next index into the frozen selection, base64url-encoded.
- **Calls (std/external/unresolved):** `.encode`, `.expect`, `serde_json::to_string`
- **Macros:** `serde_json::json!`
- **Called by:** `for_edit` (edit/for_edit.rs:1581); `tests::cursor_binds_frozen_membership_independently_of_request_filters` (edit/for_edit.rs:1811); `tests::cursor_binds_source_digest_independently_of_selection` (edit/for_edit.rs:1829)

#### `cursor_offset`  (L503-533)
`fn cursor_offset(token: &str, basis: &str) -> Result<usize, EditError>`

- **Does:** The offset a continuation resumes at; its bound basis must still hold.
- **Calls (crate):** `decode` (ambiguous, 4 candidates) L511; `super::err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L512,519,526
- **Calls (std/external/unresolved):** `.as_str`, `.as_u64`, `.decode`, `.is_string`, `.is_u64`, `.ok_or_else`, `.ok`, `.unwrap_or`, `Err`, `Ok`, `Some`, `serde_json::from_slice`
- **Called by:** `for_edit` (edit/for_edit.rs:1581); `tests::cursor_binds_frozen_membership_independently_of_request_filters` (edit/for_edit.rs:1811); `tests::cursor_binds_source_digest_independently_of_selection` (edit/for_edit.rs:1829)

#### `Selection::build`  (L547-651)
`fn build( request: &ForEditRequest, s_opened: &Opened, s_loaded: &Loaded, source: &[u8], ) -> Result<Self, EditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `super::err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L562; `id_selection` (edit/for_edit.rs:658) L578; `text_selection` (edit/for_edit.rs:663) L618; `changed_selection` (edit/for_edit.rs:703) L622
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.collect`, `.contains`, `.count`, `.dedup`, `.entry`, `.enumerate`, `.filter`, `.is_empty`, `.is_some`, `.iter`, `.len`, `.map_or_else`, `.map`, `.next`, `.ok_or_else`, `.or_default`, `.position`, `.push`, `.retain`, `.saturating_sub`, `.to_string`, `.values_mut`, `HashMap::new`, `Ok`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `task_usage` (cli.rs:3755) (?); `for_edit` (edit/for_edit.rs:1581); `run` (update.rs:186) (?)

#### `id_selection`  (L658-660)
`fn id_selection(spec: &str, source: &[u8], closed_only: bool) -> Result<HashSet<usize>, EditError>`

- **Does:** The paragraphs `spec` names: `pN`/`pN-pM` closed ranges and `tN` whole tables, against the body's own numbering on S; the transitional open-ended `pN-` / `-pN` spellings parse unless the caller requires closed ranges.
- **Calls (crate):** `SourceModel::selected_paragraphs` (edit/v2_apply.rs:1020) (by name) L659; `SourceModel::new` (edit/v2_apply.rs:814) L659
- **Called by:** `Selection::build` (edit/for_edit.rs:547)

#### `text_selection`  (L663-699)
`fn text_selection( request: &ForEditRequest, dom: &Dom, paragraph_nodes: &[NodeId], ) -> Result<HashSet<usize>, EditError>`

- **Does:** The paragraphs whose original final logical text the search matches.
- **Calls (crate):** `super::err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L690; `project_paragraph` (inspect.rs:1265) (by name) L696
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.collect`, `.contains`, `.enumerate`, `.filter`, `.is_match`, `.iter`, `.map_err`, `.map`, `.to_lowercase`, `.to_string`, `.unwrap_or`, `Box::new`, `Ok`, `matcher`, `regex::Regex::new`
- **Macros:** `format!`
- **Called by:** `Selection::build` (edit/for_edit.rs:547)

#### `changed_selection`  (L703-764)
`fn changed_selection( request: &ForEditRequest, s_loaded: &Loaded, source: &[u8], ) -> Result<HashSet<usize>, EditError>`

- **Does:** The paragraphs an original revision or comment intersects; `--by` keeps only marks of that author.
- **Calls (crate):** `paragraph_place` (ids.rs:417) L715; `list_changes` (changes.rs:564) L741; `super::err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L742,752; `tests::by` (markdown/from_docx/mod.rs:6361) (by name) L744,754; `list_comments` (comments.rs:132) L751; `.code` (ambiguous, 4 candidates) L752; `comment_paragraphs` (edit/for_edit.rs:770) L757
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.as_ref`, `.chain`, `.clone`, `.collect`, `.enumerate`, `.get`, `.insert`, `.is_empty`, `.is_some_and`, `.iter`, `.map_err`, `.map`, `.position`, `.strip_prefix`, `.to_string`, `.trim`, `.unwrap_or`, `HashSet::new`, `Ok`, `Some`, `global`, `place`
- **Called by:** `Selection::build` (edit/for_edit.rs:547)

#### `comment_paragraphs`  (L770-798)
`fn comment_paragraphs( comment: &crate::comments::CommentRecord, s_loaded: &Loaded, ) -> (Vec<(usize, usize)>, bool)`

- **Does:** The `(story, local)` paragraphs a comment's range covers: min to max of its endpoints inside one story, and each endpoint separately when a malformed range crosses stories (OOXML cannot carry one, so the comment is attached where its markers are instead of dropped).
- **Calls (crate):** `paragraph_place` (ids.rs:417) L781
- **Calls (std/external/unresolved):** `.as_ref`, `.as_str`, `.collect`, `.enumerate`, `.get`, `.iter`, `.map`, `.max`, `.min`, `Some`, `Vec::new`, `place`
- **Macros:** `vec!`
- **Called by:** `changed_selection` (edit/for_edit.rs:703); `comments_by_paragraph` (edit/for_edit.rs:816)

#### `comment_ref`  (L801-812)
`fn comment_ref(comment: &crate::comments::CommentRecord) -> CommentRef`

- **Does:** The comment reference of `comment`.
- **Calls (std/external/unresolved):** `.to_string`
- **Macros:** `format!`
- **Called by:** `comments_by_paragraph` (edit/for_edit.rs:816)

#### `comments_by_paragraph`  (L816-840)
`fn comments_by_paragraph( comments: &[crate::comments::CommentRecord], s_loaded: &Loaded, ) -> (HashMap<usize, Vec<CommentRef>>, Vec<CommentRef>)`

- **Does:** The comments intersecting each global S paragraph index, and the comments no paragraph holds (no anchor in any story).
- **Calls (crate):** `comment_paragraphs` (edit/for_edit.rs:770) L824; `comment_ref` (edit/for_edit.rs:801) L826,829
- **Calls (std/external/unresolved):** `.clone`, `.entry`, `.is_empty`, `.iter`, `.or_default`, `.position`, `.push`, `.to_string`, `HashMap::new`, `Some`, `Vec::new`, `global`
- **Called by:** `for_edit` (edit/for_edit.rs:1581)

#### `NativeKeys::collect`  (L851-865)
`fn collect(dom: &Dom, stories: &[super::StoryPart]) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L856; `native_para_id` (edit/for_edit.rs:897) L857
- **Calls (std/external/unresolved):** `.collect`, `.entry`, `.iter`, `.map`, `.or_default`, `HashMap::new`, `Some`, `W::p`
- **Called by:** `for_edit` (edit/for_edit.rs:1581)

#### `NativeKeys::pair`  (L869-891)
`fn pair( &self, dom: &Dom, story: usize, part: &str, short_id: &str, paragraph: NodeId, ) -> TargetPair`

- **Does:** The pair `paragraph` gets: its paraId when it is a usable native key, else the `j:short` long form of its short ID.
- **Calls (crate):** `part_story` (ids.rs:392) L877; `native_para_id` (edit/for_edit.rs:897) L878; `Dom::attribute` (xmllinq/mod.rs:874) L881
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.get`, `.map`, `.to_string`, `Some`, `W14::name`
- **Macros:** `format!`
- **Called by:** `make_row` (edit/for_edit.rs:1293) (?); `context_of` (edit/for_edit.rs:1505)

#### `native_para_id`  (L897-899)
`fn native_para_id(dom: &Dom, paragraph: NodeId) -> Option<u32>`

- **Does:** The `w14:paraId` of `paragraph` when it is a key Word could have written, under the shared native-key policy: exactly eight ASCII hex digits in `1..=0x7FFFFFFF`.
- **Calls (crate):** `para_id` (edit/native_keys.rs:18) L898; `Dom::attribute` (xmllinq/mod.rs:874) L898
- **Calls (std/external/unresolved):** `W14::name`
- **Called by:** `NativeKeys::collect` (edit/for_edit.rs:851); `NativeKeys::pair` (edit/for_edit.rs:869)

#### `revision_pair`  (L904-922, pub(super))
`pub(super) fn revision_pair(dom: &Dom, el: NodeId, story: &str, part: &str) -> TargetPair`

- **Does:** The long ID of the revision element `el` (a `w:ins`, `w:del`, `w:moveFrom` or `w:moveTo`): its native `w:id` when usable, else the `j:short` form of its printed short ID.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L905; `XName::local_name` (xmllinq/mod.rs:208) (by name) L907; `Dom::attribute` (xmllinq/mod.rs:874) L909; `ChangeId::short` (ids.rs:155) (by name) L910; `ChangeId::new` (ids.rs:142) L910; `part_story` (ids.rs:392) L911; `revision_id` (edit/native_keys.rs:30) L912; `escape_value` (edit/for_edit.rs:927) L914
- **Calls (std/external/unresolved):** `.is_some`, `.map`, `.to_string`, `.unwrap_or_default`, `W::name`
- **Macros:** `format!`
- **Called by:** `classify` (edit/for_edit.rs:950); `obstacles` (edit/for_edit.rs:1041); `RevisionRefusal::new` (edit/refusal.rs:51); `decisions` (edit/source_refs.rs:25) (?)

#### `escape_value`  (L927-937)
`fn escape_value(text: &str) -> String`

- **Does:** The VALUE escaping of the long-ID contract: ASCII letters, digits, dot, underscore and colon stay literal; every other UTF-8 byte is a hyphen and two uppercase hex digits.
- **Calls (std/external/unresolved):** `.as_bytes`, `.is_ascii_alphanumeric`, `.len`, `.push_str`, `.push`, `String::with_capacity`, `char::from`
- **Macros:** `format!`, `matches!`
- **Called by:** `revision_pair` (edit/for_edit.rs:904)

#### `piece_element`  (L940-945)
`fn piece_element(piece: &Piece) -> NodeId`

- **Does:** The element a projection segment's text comes from.
- **Calls:** none
- **Called by:** `obstacles` (edit/for_edit.rs:1041)

#### `classify`  (L950-1032)
`fn classify( dom: &Dom, run: NodeId, paragraph: NodeId, story: &str, part: &str, ) -> (String, String, Option<TargetPair>, Option<String>)`

- **Does:** What an opaque segment sits inside, in order of strictness: a revision first (a decision frees it), then the structure that holds the text (no edit kind may cross it).
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L957; `Dom::name` (xmllinq/mod.rs:526) L961; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L964,972; `XName::local_name` (xmllinq/mod.rs:208) (by name) L975,977,1020; `revision_pair` (edit/for_edit.rs:904) L982; `Dom::attribute` (xmllinq/mod.rs:874) L983
- **Calls (std/external/unresolved):** `.map`, `.to_string`, `Some`, `W::name`
- **Macros:** `format!`
- **Called by:** `obstacles` (edit/for_edit.rs:1041)

#### `char_pos`  (L1035-1037)
`fn char_pos(text: &str, byte: usize) -> usize`

- **Does:** Byte offset to Unicode scalar position in `text`.
- **Calls (std/external/unresolved):** `.chars`, `.count`
- **Called by:** `obstacles` (edit/for_edit.rs:1041)

#### `obstacles`  (L1041-1258)
`fn obstacles( dom: &Dom, paragraph: NodeId, story: &str, part: &str, ) -> (Vec<Blocker>, Vec<RevisionRef>, Vec<RevisionRef>)`

- **Does:** The blockers, revision references and mark revisions of a working paragraph, from its projection and XML.
- **Calls (crate):** `project_paragraph` (inspect.rs:1265) (by name) L1047; `Dom::descendants` (xmllinq/mod.rs:668) L1049,1163,1247; `revision_pair` (edit/for_edit.rs:904) L1065,1211,1236,1248; `Dom::attribute` (xmllinq/mod.rs:874) L1070,1240,1252; `piece_element` (edit/for_edit.rs:940) L1093,1201; `Dom::name_is` (xmllinq/mod.rs:535) L1095,1176; `close` (ambiguous, 9 candidates) L1096,1107,1146,1150; `char_pos` (edit/for_edit.rs:1035) L1096,1098,1107,1122,1135,1140...; `classify` (edit/for_edit.rs:950) L1113; `Dom::ancestors` (xmllinq/mod.rs:770) L1114,1127,1172; `Dom::name` (xmllinq/mod.rs:526) L1115,1128,1164,1225; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1116,1129,1167,1228; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1116,1129,1170,1231; `revision` (ambiguous, 6 candidates) L1118,1131,1214; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L1195; `Dom::element` (xmllinq/mod.rs:655) L1220,1222; `Dom::elements` (xmllinq/mod.rs:640) L1224
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_str`, `.chars`, `.clone`, `.collect`, `.copied`, `.count`, `.enumerate`, `.filter`, `.find`, `.get`, `.insert`, `.into_iter`, `.is_some_and`, `.iter`, `.len`, `.map`, `.min`, `.push`, `.take_while`, `.take`, `.to_string`, `.unwrap_or`, `HashSet::new`, `Some`, `Vec::new`, `W::name`, `W::p_pr`, `W::r_pr`
- **Macros:** `matches!`
- **Called by:** `make_row` (edit/for_edit.rs:1293); `context_of` (edit/for_edit.rs:1505)

#### `text_chars`  (L1261-1263)
`fn text_chars(row: &ForEditRow) -> usize`

- **Does:** The characters a row emits, for the text budget.
- **Calls (crate):** `TextValue::chars` (edit/for_edit.rs:173) L1262
- **Calls (std/external/unresolved):** `.as_ref`, `.chars`, `.map_or`
- **Called by:** `for_edit` (edit/for_edit.rs:1581)

#### `bound_row`  (L1266-1276)
`fn bound_row(row: &mut ForEditRow, budget: usize)`

- **Does:** Bound a row's text values to `budget` characters, working first.
- **Calls (crate):** `TextValue::bounded` (edit/for_edit.rs:146) L1270,1274
- **Calls (std/external/unresolved):** `.as_ref`, `.chars`, `.map_or`, `.saturating_sub`, `.take`
- **Called by:** `for_edit` (edit/for_edit.rs:1581)

#### `make_row`  (L1293-1392)
`fn make_row(g: usize, ctx: &RowCtx<'_>) -> ForEditRow`

- **Does:** Build one row of the result for original paragraph `g`.
- **Calls (crate):** `paragraph_id` (ids.rs:410) L1307,1322; `project_paragraph` (inspect.rs:1265) (by name) L1309,1339; `logical_text` (edit/v2.rs:802) L1310,1340; `source_sha256` (inspect.rs:299) (by name) L1312,1341; `TextValue::complete` (edit/for_edit.rs:136) L1346,1375; `obstacles` (edit/for_edit.rs:1041) L1350,1359; `readiness` (edit/for_edit.rs:1400) L1365; `before_sha256_present` (edit/for_edit.rs:1394) L1367; `.pair` (ambiguous: `NativeKeys::pair` (edit/for_edit.rs:869); `SourceModel::pair` (edit/v2_apply.rs:979)) L1372
- **Calls (std/external/unresolved):** `.as_bytes`, `.as_str`, `.chain`, `.cloned`, `.get`, `.into_iter`, `.is_none`, `.map`, `.ok`, `.push`, `.to_string`, `.unwrap_or_default`, `Fate::Kept`, `Some`, `Vec::new`
- **Macros:** `matches!`, `vec!`
- **Called by:** `for_edit` (edit/for_edit.rs:1581)

#### `before_sha256_present`  (L1394-1396)
`fn before_sha256_present(working: &Option<WorkingText>) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_ref`, `.is_some_and`, `.is_some`
- **Called by:** `make_row` (edit/for_edit.rs:1293)

#### `readiness`  (L1400-1501)
`fn readiness( fate: Fate, representable: bool, blockers: &[Blocker], mark_revisions: &[RevisionRef], ) -> Vec<Readiness>`

- **Does:** The readiness rows of one paragraph: range operations share the blockers; whole-paragraph operations add the mark revisions.
- **Calls (crate):** `pair_story_final_marks::{fn status}` (comparer/lcs.rs:5797) (by name) L1433,1451,1471,1481,1496
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.collect`, `.is_empty`, `.iter`, `.len`, `.push`, `.then`, `.to_string`, `Fate::Merged`, `Some`, `Vec::new`, `reason`
- **Macros:** `matches!`
- **Called by:** `make_row` (edit/for_edit.rs:1293)

#### `context_of`  (L1505-1569)
`fn context_of( g: usize, selection: &Selection, s_opened: &Opened, s_loaded: &Loaded, native: &NativeKeys, budget: &mut usize, ) -> Vec<SourceContextRow>`

- **Does:** The read-only context neighbours of selected paragraph `g`, charged to `budget`.
- **Calls (crate):** `paragraph_id` (ids.rs:410) L1538; `project_paragraph` (inspect.rs:1265) (by name) L1539; `logical_text` (edit/v2.rs:802) L1540; `obstacles` (edit/for_edit.rs:1041) L1546; `TextValue::bounded` (edit/for_edit.rs:146) L1551; `TextValue::chars` (edit/for_edit.rs:173) L1552; `NativeKeys::pair` (edit/for_edit.rs:869) L1555
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.contains`, `.extend`, `.into_iter`, `.is_ok`, `.iter`, `.position`, `.push`, `.saturating_add`, `.saturating_sub`, `.to_string`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `for_edit` (edit/for_edit.rs:1581)

#### `for_edit`  (L1581-1804, pub)
`pub fn for_edit(source: &[u8], request: &ForEditRequest) -> Result<ForEdit, EditError>`

- **Does:** Read `source` for editing: selection frozen on S, text and readiness derived from the working document `request` settles S to.
- **Calls (crate):** `super::err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L1583,1586,1658,1667; `source_sha256` (inspect.rs:299) (by name) L1588,1781; `Opened::open` (inspect.rs:735) L1589,1625; `ChangeFilter::ids` (changes.rs:330) L1599,1601; `settle` (ambiguous: `settle` (edit/hint.rs:301); `settle` (edit.rs:1617)) L1617; `load_stories` (edit.rs:1680) (by name) L1622,1626; `Origin::build` (edit/origin.rs:260) L1631; `Origin::exact` (edit/origin.rs:400) L1633; `Origin::fate` (edit/origin.rs:478) L1635; `NativeKeys::collect` (edit/for_edit.rs:851) L1648; `Selection::build` (edit/for_edit.rs:547) L1651; `basis_digest` (edit/for_edit.rs:456) L1652; `cursor_offset` (edit/for_edit.rs:503) L1654; `list_comments` (comments.rs:132) L1666; `.code` (ambiguous, 4 candidates) L1667; `comments_by_paragraph` (edit/for_edit.rs:816) L1668; `make_row` (edit/for_edit.rs:1293) L1693; `bound_row` (edit/for_edit.rs:1266) L1708; `text_chars` (edit/for_edit.rs:1261) L1709; `context_of` (edit/for_edit.rs:1505) L1713; `cursor_token` (edit/for_edit.rs:495) L1751; `existing_revisions_name` (edit/for_edit.rs:445) L1778
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.cloned`, `.collect`, `.dedup`, `.enumerate`, `.flatten`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some`, `.iter`, `.join`, `.len`, `.map_err`, `.map`, `.push`, `.saturating_sub`, `.skip`, `.sort`, `.then_some`, `.then`, `.to_string`, `.unwrap_or`, `Default::default`, `Err`, `Fate::Kept`, `Fate::Merged`, `Ok`, `Some`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `run_for_edit` (bin/jubarte.rs:961) (?); `compile` (edit/inline.rs:38) (?); `tests::review_11_prepare_v2_checks_serialized_text_like_v1` (edit/prepared.rs:1292); `generate` (edit/template.rs:84); `tests::final_views_read_off_the_dom_match_a_reparse_of_the_emitted_primary` (edit/v2_apply.rs:2801) (?)

## Test functions (2)

- `tests::cursor_binds_frozen_membership_independently_of_request_filters` (L1811): _no doc comment_
- `tests::cursor_binds_source_digest_independently_of_selection` (L1829): _no doc comment_
