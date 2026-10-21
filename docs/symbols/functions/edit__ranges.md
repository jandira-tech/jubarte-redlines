# Functions and methods in `src/edit/ranges.rs`

9 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `is_marker`  (L16-20)
`fn is_marker(dom: &Dom, node: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L19
- **Calls (std/external/unresolved):** `.any`, `.iter`, `W::name`
- **Called by:** `fix_paragraph_mark_revision_order` (comparer/finalize.rs:1526) (?); `Layout::line_face_metrics` (convert/mod.rs:23868) (?); `key` (edit/ranges.rs:25); `restore_empty` (edit/ranges.rs:103); `restore_textless_permissions` (edit/ranges.rs:323)

#### `key`  (L25-57)
`fn key(dom: &Dom, node: NodeId, keep_markers: bool, content_only: bool) -> String`

- **Does:** Namespace/prefix-independent structural key, excluding carried markers and volatile producer ids.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L26; `is_marker` (edit/ranges.rs:16) L29; `Dom::name` (xmllinq/mod.rs:526) L32; `Dom::text_value` (xmllinq/mod.rs:547) L33; `Dom::nodes` (xmllinq/mod.rs:604) L35; `key` (edit/ranges.rs:25) L38; `Dom::attributes` (xmllinq/mod.rs:899) L40; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L44,45,51; `XName::local_name` (xmllinq/mod.rs:208) (by name) L46,47,51
- **Calls (std/external/unresolved):** `.cmp`, `.collect`, `.contains`, `.filter`, `.into_iter`, `.is_empty`, `.map`, `.sort_by`, `.starts_with`, `.to_string`, `.unwrap_or`, `String::new`, `W::p_pr`, `W::r_pr`
- **Macros:** `format!`, `matches!`
- **Called by:** `earliest` (changes/logical.rs:383) (?); `fold_twins` (changes.rs:1280) (?); `step_h` (comparer/lcs.rs:3621) (?); `has_common_run_ge` (comparer/lcs.rs:5736) (?); `group_by_key_stable` (comparer/produce.rs:692) (?); `key` (edit/ranges.rs:25); `correspondence` (edit/ranges.rs:68)

#### `unchanged`  (L59-66)
`fn unchanged( dom: &Dom, from: &[NodeId], to: &[NodeId], keep_markers: bool, ) -> HashMap<NodeId, NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `correspondence` (edit/ranges.rs:68) L65
- **Called by:** `restore_empty` (edit/ranges.rs:103)

#### `correspondence`  (L68-98)
`fn correspondence( dom: &Dom, from: &[NodeId], to: &[NodeId], keep_markers: bool, content_only: bool, ) -> HashMap<NodeId, NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `key` (edit/ranges.rs:25) L77,81
- **Calls (std/external/unresolved):** `.as_tag_tuple`, `.collect`, `.copied`, `.flat_map`, `.into_iter`, `.iter`, `.map`, `.zip`, `Vec::new`, `capture_diff_slices`
- **Called by:** `unchanged` (edit/ranges.rs:59); `restore_textless_permissions` (edit/ranges.rs:323)

#### `restore_empty`  (L103-253)
`fn restore_empty(dom: &mut Dom, original: NodeId, revised: NodeId, output: NodeId)`

- **Does:** Find original empty paragraphs among the paragraphs left unchanged in both the edited copy and the comparison.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L104,105,106,111,137,163; `unchanged` (edit/ranges.rs:59) L107,108; `Dom::value` (xmllinq/mod.rs:1185) L114; `Dom::elements` (xmllinq/mod.rs:640) L122,210; `is_marker` (edit/ranges.rs:16) L123,142,212; `Dom::attribute` (xmllinq/mod.rs:874) L138,149,164,168,191,219; `Dom::name` (xmllinq/mod.rs:526) L144,218,229; `Dom::name_is` (xmllinq/mod.rs:535) L148,167,176; `Dom::remove` (xmllinq/mod.rs:1118) L241; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L243; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L245; `Dom::add_before_self` (xmllinq/mod.rs:1130) L248; `Dom::add` (xmllinq/mod.rs:1074) L250
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.clone`, `.cloned`, `.collect`, `.contains`, `.entry`, `.filter`, `.get_mut`, `.get`, `.insert`, `.into_iter`, `.is_empty`, `.is_none_or`, `.iter`, `.keys`, `.map`, `.max`, `.or_default`, `.parse`, `.push_back`, `.push`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `.unwrap`, `HashMap::new`, `Some`, `Vec::new`, `W::bookmark_end`, `W::bookmark_start`, `W::id`, `W::name`, `W::p`, `W::t`
- **Called by:** `restore` (edit/ranges.rs:432)

#### `needs_restore`  (L257-318)
`fn needs_restore(xml: &str) -> bool`

- **Does:** A streaming eligibility scan keeps ordinary bookmarked text paragraphs off the restoration path.
- **Calls (crate):** `XName::local_name` (xmllinq/mod.rs:208) (by name) L275,286,292
- **Calls (std/external/unresolved):** `.as_ref`, `.contains`, `.is_empty`, `.is_some_and`, `.last_mut`, `.pop`, `.push`, `.read_resolved_event`, `Vec::new`, `quick_xml::NsReader::from_str`, `quick_xml::name::ResolveResult::Bound`
- **Macros:** `matches!`
- **Called by:** `restore` (edit/ranges.rs:432)

#### `restore_textless_permissions`  (L323-368)
`fn restore_textless_permissions(dom: &mut Dom, revised: NodeId, output: NodeId)`

- **Does:** Text has no offset projection in an empty story.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L325; `Dom::remove` (xmllinq/mod.rs:1118) L326; `Dom::elements` (xmllinq/mod.rs:640) L331,337; `is_marker` (edit/ranges.rs:16) L335,340; `correspondence` (edit/ranges.rs:68) L342; `Dom::name_is` (xmllinq/mod.rs:535) L349,359; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L360; `Dom::add_before_self` (xmllinq/mod.rs:1130) L362; `Dom::add` (xmllinq/mod.rs:1074) L364
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.copied`, `.enumerate`, `.filter_map`, `.filter`, `.find`, `.get`, `.into_iter`, `.iter`, `.pop`, `.push`, `Some`, `W::name`, `W::p_pr`
- **Macros:** `vec!`
- **Called by:** `restore` (edit/ranges.rs:432)

#### `normalize_permission_ids`  (L374-429, pub(super))
`pub(super) fn normalize_permission_ids(bytes: &[u8]) -> Result<Option<Vec<u8>>, EditError>`

- **Does:** Give repeated source occurrences stable identities before any edit removes or moves nodes.
- **Calls (crate):** `PartFs::open` (opc/mod.rs:585) L375; `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L375,425; `PartFs::parts` (opc/mod.rs:733) (by name) L377; `PartFs::part_string` (opc/mod.rs:636) (by name) L378; `Dom::new` (xmllinq/mod.rs:387) L381; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L382; `Dom::root` (xmllinq/mod.rs:822) L383; `Dom::descendants` (xmllinq/mod.rs:668) L386; `Dom::attribute` (xmllinq/mod.rs:874) L389,396; `Dom::name_is` (xmllinq/mod.rs:535) L399,407; `fresh_permission_id` (comparer/bookmarks.rs:581) L403; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L413; `PartFs::set_part` (opc/mod.rs:661) (by name) L418; `Dom::serialize_document` (xmllinq/mod.rs:1338) L418; `PartFs::to_zip` (opc/mod.rs:749) (by name) L423
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.collect`, `.contains`, `.entry`, `.filter_map`, `.filter`, `.get_mut`, `.insert`, `.into_bytes`, `.iter`, `.map_err`, `.map`, `.or_default`, `.push`, `.to_string`, `HashMap::new`, `HashSet::new`, `Ok`, `Some`, `W::id`, `W::name`
- **Called by:** `Transaction::start` (edit.rs:2680)

#### `restore`  (L432-512, pub(super))
`pub(super) fn restore( redline: &[u8], original: &[u8], revised: &[u8], author: &str, ) -> Result<Vec<u8>, EditError>`

- **Does:** Apply to the body and all related header/footer/footnote/endnote parts.
- **Calls (crate):** `PartFs::open` (opc/mod.rs:585) L438; `err` (ambiguous: `err` (edit/prepared.rs:1047); `err` (edit.rs:1770); `err` (markdown/package.rs:98)) L438,511; `open` (ambiguous: `open` (append.rs:316); `open` (changes.rs:1031); `open` (edit/anchor_error.rs:229)) L439,440,441; `PartFs::main_document_part` (opc/mod.rs:1070) (by name) L442; `PartFs::read_rels_for` (opc/mod.rs:991) (by name) L447; `PartFs::resolve_rel_target` (opc/mod.rs:829) (by name) L458; `Dom::new` (xmllinq/mod.rs:387) L462; `PartFs::part_string` (opc/mod.rs:636) (by name) L465,468,474; `needs_restore` (edit/ranges.rs:257) L471; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L477,492,493; `Dom::root` (xmllinq/mod.rs:822) L478,494; `carry_permissions` (comparer/bookmarks.rs:745) L484; `Dom::descendants` (xmllinq/mod.rs:668) L495; `Dom::value` (xmllinq/mod.rs:1185) L498; `restore_textless_permissions` (edit/ranges.rs:323) L500; `restore_empty` (edit/ranges.rs:103) L502; `PartFs::set_part` (opc/mod.rs:661) (by name) L504; `Dom::serialize_document` (xmllinq/mod.rs:1338) L504; `PartFs::to_zip` (opc/mod.rs:749) (by name) L510
- **Calls (std/external/unresolved):** `.all`, `.as_deref`, `.clone`, `.contains`, `.dedup`, `.extend`, `.filter`, `.flat_map`, `.into_bytes`, `.into_iter`, `.is_empty`, `.map_err`, `.map`, `.next`, `.rsplit`, `.sort`, `.to_string`, `.to_vec`, `.unwrap_or_else`, `.zip`, `Ok`, `Some`, `W::t`
- **Macros:** `matches!`, `vec!`
- **Called by:** `publish_dir` (bin/jubarte.rs:2019) (?); `produce` (edit/prepared.rs:928)
