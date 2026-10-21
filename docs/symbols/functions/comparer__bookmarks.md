# Functions and methods in `src/comparer/bookmarks.rs`

16 non-test, 3 test. Part of the [symbol manifest](../README.md).

#### `paragraph_of`  (L76-85)
`fn paragraph_of(dom: &Dom, mut node: NodeId) -> Option<NodeId>`

- **Does:** The nearest enclosing paragraph of `node`, if any.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L78; `Dom::name_is` (xmllinq/mod.rs:535) L79
- **Calls (std/external/unresolved):** `Some`, `W::p`
- **Called by:** `Anchors::scan` (comments.rs:251) (?); `extract` (comparer/bookmarks.rs:92); `inject_side` (comparer/bookmarks.rs:309); `Transaction::resolve_visible` (edit/visible.rs:348) (?)

#### `extract`  (L92-190)
`fn extract(pkg: &PartFs, main: &str, permission: bool) -> (Vec<char>, Vec<Marker>)`

- **Does:** The source body's text projection (every `w:t` character, as [`collect_segments`] counts them) and its bookmarks in start order.
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L93; `Dom::new` (xmllinq/mod.rs:387) L96; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L97; `Dom::root` (xmllinq/mod.rs:822) L99; `Dom::element` (xmllinq/mod.rs:655) L99; `Dom::descendant_nodes` (xmllinq/mod.rs:730) L116,150; `Dom::is_text` (xmllinq/mod.rs:506) L117; `Dom::parent` (xmllinq/mod.rs:597) L118,167,183; `Dom::name_is` (xmllinq/mod.rs:535) L118,154,167,183; `Dom::text_value` (xmllinq/mod.rs:547) L119; `paragraph_of` (comparer/bookmarks.rs:76) L120,146,182; `Dom::name` (xmllinq/mod.rs:526) L126; `Dom::attribute` (xmllinq/mod.rs:874) L127,134,142,143; `Dom::value` (xmllinq/mod.rs:1185) L154; `Dom::attributes` (xmllinq/mod.rs:899) L160,178; `Dom::ancestors` (xmllinq/mod.rs:770) L170
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.contains`, `.entry`, `.extend`, `.filter`, `.insert`, `.into_iter`, `.into_values`, `.is_empty`, `.is_none_or`, `.is_some_and`, `.len`, `.map`, `.or_default`, `.push`, `.remove`, `.skip_while`, `.skip`, `.sort_by_key`, `.then`, `.to_string`, `.unwrap_or`, `HashMap::new`, `HashSet::new`, `Some`, `Vec::new`, `W::body`, `W::name`, `W::t`
- **Called by:** `carry_matching_bookmarks` (comparer/bookmarks.rs:483); `carry_permission_union` (comparer/bookmarks.rs:594); `carry_permissions` (comparer/bookmarks.rs:745)

#### `find_from`  (L193-199)
`fn find_from(haystack: &[char], needle: &[char], from: usize) -> Option<usize>`

- **Does:** First occurrence of `needle` at or after `from`, wrapping to the start.
- **Calls (std/external/unresolved):** `.chain`, `.checked_sub`, `.find`, `.len`, `.min`
- **Called by:** `map_point` (comparer/bookmarks.rs:206)

#### `map_point`  (L206-225)
`fn map_point(src: &[char], merged: &[char], o: usize, hint: usize, after: bool) -> Option<usize>`

- **Does:** Map source offset `o` onto the merged projection by the text around it, with shrinking context windows, searching forward from `hint`.
- **Calls (crate):** `find_from` (comparer/bookmarks.rs:193) L220
- **Calls (std/external/unresolved):** `.chain`, `.concat`, `.into_iter`, `.is_empty`, `.len`, `.min`, `.saturating_sub`, `Some`
- **Called by:** `inject_side` (comparer/bookmarks.rs:309)

#### `outermost_sdt`  (L228-230)
`fn outermost_sdt(dom: &Dom, node: NodeId) -> Option<NodeId>`

- **Does:** The outermost content control around `node`.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L229
- **Calls (std/external/unresolved):** `.copied`, `.last`, `Some`, `W::name`
- **Called by:** `leave_foreign_controls` (comparer/bookmarks.rs:235)

#### `leave_foreign_controls`  (L235-261)
`fn leave_foreign_controls(dom: &mut Dom, m: &Marker, start: NodeId, end: NodeId, empty: bool)`

- **Does:** Move a placed pair out of the content controls its source bookmark was not in: the start before its outermost control, the end after its own.
- **Calls (crate):** `outermost_sdt` (comparer/bookmarks.rs:228) L240,253,257; `Dom::remove` (xmllinq/mod.rs:1118) L243,244,254,258; `Dom::add_before_self` (xmllinq/mod.rs:1130) L246,255; `Dom::add_after_self` (xmllinq/mod.rs:1147) L248,250,259
- **Called by:** `inject_side` (comparer/bookmarks.rs:309)

#### `new_markers`  (L264-286)
`fn new_markers(dom: &mut Dom, m: &Marker, id: u32) -> (NodeId, NodeId)`

- **Does:** The `w:bookmarkStart`/`w:bookmarkEnd` pair for `m` under `id`.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L266,268,276,283; `Dom::set_attributes` (xmllinq/mod.rs:935) L267,269; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L271,277,278,280,281,284
- **Calls (std/external/unresolved):** `.clone`, `.is_empty`, `.to_string`, `Some`, `W::id`, `W::name`
- **Called by:** `inject_side` (comparer/bookmarks.rs:309)

#### `leave_inline_revision`  (L290-306)
`fn leave_inline_revision(dom: &mut Dom, endpoint: NodeId, after: bool)`

- **Does:** Keep a boundary endpoint beside the outermost inline revision containing its adjacent run, on the same side.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L292; `Dom::name_is` (xmllinq/mod.rs:535) L295,296; `Dom::remove` (xmllinq/mod.rs:1118) L299; `Dom::add_after_self` (xmllinq/mod.rs:1147) L301; `Dom::add_before_self` (xmllinq/mod.rs:1130) L303
- **Calls (std/external/unresolved):** `.any`, `.filter`, `.into_iter`, `.iter`, `.last`, `.take_while`, `W::del`, `W::ins`, `W::name`, `W::p`
- **Called by:** `inject_side` (comparer/bookmarks.rs:309); `leave_permission_revision_boundary` (comparer/bookmarks.rs:551)

#### `inject_side`  (L309-402)
`fn inject_side( dom: &mut Dom, result_root: NodeId, (src, markers): (&[char], &[&Marker]), b_side: bool, author: &str, next_id: &mut u32, mut placed: impl FnMut(usize, NodeId, NodeId), )`

- **Does:** Place one side's bookmarks on the merged body's projection of that side.
- **Calls (crate):** `collect_segments` (comparer/comments.rs:477) L322; `map_point` (comparer/bookmarks.rs:206) L334,347; `new_markers` (comparer/bookmarks.rs:264) L353; `place_after_offset` (comparer/comments.rs:691) L361,370,376; `leave_inline_revision` (comparer/bookmarks.rs:290) L362,368; `place_before_offset` (comparer/comments.rs:665) L364,367,373; `Dom::add_after_self` (xmllinq/mod.rs:1147) L374,393; `Dom::add_before_self` (xmllinq/mod.rs:1130) L377,395; `leave_foreign_controls` (comparer/bookmarks.rs:235) L379; `paragraph_of` (comparer/bookmarks.rs:76) L390; `Dom::remove` (xmllinq/mod.rs:1118) L391
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.enumerate`, `.filter`, `.is_empty`, `.iter`, `.unwrap_or`, `placed`
- **Called by:** `carry_matching_bookmarks` (comparer/bookmarks.rs:483); `carry_permission_union` (comparer/bookmarks.rs:594); `carry_permissions` (comparer/bookmarks.rs:745); `tests::check_boundary_revision` (comparer/bookmarks.rs:819)

#### `settle_present_bookmarks`  (L407-448)
`fn settle_present_bookmarks(dom: &mut Dom, result_root: NodeId) -> (HashSet<String>, u32)`

- **Does:** Drop the `_GoBack` bookmarks that rode in with opaque content (textboxes) and renumber the others above every `w:id` in the body.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L414; `Dom::attribute` (xmllinq/mod.rs:874) L415,420,434; `Dom::name_is` (xmllinq/mod.rs:535) L419,427; `Dom::remove` (xmllinq/mod.rs:1118) L438; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L445
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.contains`, `.entry`, `.extend`, `.insert`, `.map`, `.max`, `.ok`, `.or_insert_with`, `.parse`, `.push`, `.to_string`, `.unwrap_or`, `HashMap::new`, `HashSet::new`, `Some`, `Vec::new`, `W::name`
- **Called by:** `carry_matching_bookmarks` (comparer/bookmarks.rs:483)

#### `carry_bookmarks`  (L460-475, pub)
`pub fn carry_bookmarks( dom: &mut Dom, result_root: NodeId, (pkg1, main1): (&PartFs, &str), (pkg2, main2): (&PartFs, &str), author: &str, )`

- **Does:** Carry both source documents' bookmarks into the redline body, then renumber ids above every `w:id` already present so they do not collide with revisions or comments (Word keeps bookmark and revision ids disjoint).
- **Calls (crate):** `carry_matching_bookmarks` (comparer/bookmarks.rs:483) L467
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520)

#### `carry_matching_bookmarks`  (L483-534, pub)
`pub fn carry_matching_bookmarks( dom: &mut Dom, result_root: NodeId, (pkg1, main1): (&PartFs, &str), (pkg2, main2): (&PartFs, &str), author: &str, keep: impl Fn(&str) -> bool, )`

- **Does:** [`carry_bookmarks`] for the bookmarks whose name passes `keep`, applied to one part root (the body, or a single header, footer or notes part).
- **Calls (crate):** `extract` (comparer/bookmarks.rs:92) L491,492; `settle_present_bookmarks` (comparer/bookmarks.rs:407) L503; `inject_side` (comparer/bookmarks.rs:309) L516,525
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.contains`, `.filter`, `.is_empty`, `.iter`, `.map`, `.retain`, `keep`
- **Called by:** `carry_bookmarks` (comparer/bookmarks.rs:460); `rewrite` (edit/whole.rs:130)

#### `leave_permission_revision_boundary`  (L551-578)
`fn leave_permission_revision_boundary(dom: &mut Dom, endpoint: NodeId, after: bool)`

- **Does:** A boundary of a whole changed run belongs outside its revision.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L552; `Dom::name_is` (xmllinq/mod.rs:535) L555,559,573; `Dom::descendants` (xmllinq/mod.rs:668) L563; `Dom::value` (xmllinq/mod.rs:1185) L573; `leave_inline_revision` (comparer/bookmarks.rs:290) L577
- **Calls (std/external/unresolved):** `.any`, `.filter`, `.into_iter`, `.is_empty`, `.iter`, `.last`, `.position`, `.take_while`, `W::del_text`, `W::del`, `W::ins`, `W::name`, `W::p`, `W::t`
- **Called by:** `carry_permission_union` (comparer/bookmarks.rs:594)

#### `fresh_permission_id`  (L581-588, pub(crate))
`pub(crate) fn fresh_permission_id(reserved: &mut HashSet<String>, next: &mut u64) -> String`

- **Does:** Allocate outside every source/output id reserved by the caller (#1190).
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.insert`, `.to_string`
- **Called by:** `carry_permission_union` (comparer/bookmarks.rs:594); `normalize_permission_ids` (edit/ranges.rs:374)

#### `carry_permission_union`  (L594-740, pub(crate))
`pub(crate) fn carry_permission_union( dom: &mut Dom, root: NodeId, original: (&PartFs, &str), revised: (&PartFs, &str), author: &str, )`

- **Does:** Word Compare unions permissions by editor and column scope, independently of source ids.
- **Calls (crate):** `PartFs::part_bytes` (opc/mod.rs:628) (by name) L602; `extract` (comparer/bookmarks.rs:92) L610,611; `Dom::descendants` (xmllinq/mod.rs:668) L623,664; `Dom::attribute` (xmllinq/mod.rs:874) L624,690; `Dom::name_is` (xmllinq/mod.rs:535) L631,670,676,678,694; `Dom::remove` (xmllinq/mod.rs:1118) L633,710,712,716; `inject_side` (comparer/bookmarks.rs:309) L639; `leave_permission_revision_boundary` (comparer/bookmarks.rs:551) L650,651; `Dom::parent` (xmllinq/mod.rs:597) L667; `Dom::value` (xmllinq/mod.rs:1185) L677; `Dom::ancestors` (xmllinq/mod.rs:770) L691; `fresh_permission_id` (comparer/bookmarks.rs:581) L733; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L737
- **Calls (std/external/unresolved):** `.any`, `.chain`, `.chars`, `.clone`, `.cmp`, `.collect`, `.contains`, `.count`, `.enumerate`, `.expect`, `.find`, `.insert`, `.into_iter`, `.is_some_and`, `.iter`, `.last_mut`, `.last`, `.map`, `.pop`, `.push`, `.retain`, `.sort_by_key`, `.sort_by`, `.to_string`, `HashMap::new`, `HashSet::new`, `Some`, `String::from_utf8_lossy`, `Vec::new`, `W::del_text`, `W::drawing`, `W::endnote`, `W::footnote`, `W::id`, `W::name`, `W::object`, `W::p`, `W::pict`, `W::t`, `W::tc`, `W::tr`, `has_permissions`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520)

#### `carry_permissions`  (L745-812, pub(crate))
`pub(crate) fn carry_permissions( dom: &mut Dom, root: NodeId, original: (&PartFs, &str), revised: (&PartFs, &str), author: &str, )`

- **Does:** Edit-only permission carry-over.
- **Calls (crate):** `extract` (comparer/bookmarks.rs:92) L752,753; `Dom::descendants` (xmllinq/mod.rs:668) L768,800; `Dom::name` (xmllinq/mod.rs:526) L769,802; `Dom::attribute` (xmllinq/mod.rs:874) L771,804; `inject_side` (comparer/bookmarks.rs:309) L782,791; `Dom::remove` (xmllinq/mod.rs:1118) L809
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.contains`, `.entry`, `.filter`, `.get_mut`, `.insert`, `.iter`, `.map`, `.or_insert`, `.retain`, `.to_string`, `.zip`, `HashMap::new`, `HashSet::new`, `W::id`, `W::name`
- **Called by:** `restore` (edit/ranges.rs:432)

## Test functions (3)

- `tests::check_boundary_revision` (L819): _no doc comment_
- `tests::trailing_start_leaves_outermost_inline_revision_on_same_side` (L895): _no doc comment_
- `tests::leading_end_leaves_outermost_inline_revision_on_same_side` (L900): _no doc comment_
