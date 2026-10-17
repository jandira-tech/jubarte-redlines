# Functions and methods in `src/comparer/bookmarks.rs`

12 non-test, 3 test. Part of the [symbol manifest](../README.md).

#### `paragraph_of`  (L66-75)
`fn paragraph_of(dom: &Dom, mut node: NodeId) -> Option<NodeId>`

- **Does:** The nearest enclosing paragraph of `node`, if any.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L68; `Dom::name_is` (xmllinq/mod.rs:535) L69
- **Calls (std/external/unresolved):** `Some`, `W::p`
- **Called by:** `Anchors::scan` (comments.rs:241) (?); `extract` (comparer/bookmarks.rs:82)

#### `extract`  (L82-157)
`fn extract(pkg: &PartFs, main: &str) -> (Vec<char>, Vec<Marker>)`

- **Does:** The source body's text projection (every `w:t` character, as [`collect_segments`] counts them) and its bookmarks in start order.
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:498) L83; `Dom::new` (xmllinq/mod.rs:387) L86; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L87; `Dom::root` (xmllinq/mod.rs:822) L89; `Dom::element` (xmllinq/mod.rs:655) L89; `Dom::descendant_nodes` (xmllinq/mod.rs:730) L100,129; `Dom::is_text` (xmllinq/mod.rs:506) L101; `Dom::parent` (xmllinq/mod.rs:597) L102; `Dom::name_is` (xmllinq/mod.rs:535) L102,133; `Dom::text_value` (xmllinq/mod.rs:547) L103; `paragraph_of` (comparer/bookmarks.rs:66) L104,125,150; `Dom::name` (xmllinq/mod.rs:526) L110; `Dom::attribute` (xmllinq/mod.rs:874) L111,115,121,122; `Dom::value` (xmllinq/mod.rs:1169) L133; `Dom::ancestors` (xmllinq/mod.rs:770) L143
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.contains`, `.extend`, `.filter`, `.insert`, `.into_iter`, `.into_values`, `.is_empty`, `.is_none_or`, `.is_some_and`, `.len`, `.map`, `.push`, `.remove`, `.skip_while`, `.skip`, `.sort_by_key`, `.to_string`, `.unwrap_or`, `HashMap::new`, `HashSet::new`, `Some`, `Vec::new`, `W::body`, `W::name`, `W::t`
- **Called by:** `carry_matching_bookmarks` (comparer/bookmarks.rs:418)

#### `find_from`  (L160-166)
`fn find_from(haystack: &[char], needle: &[char], from: usize) -> Option<usize>`

- **Does:** First occurrence of `needle` at or after `from`, wrapping to the start.
- **Calls (std/external/unresolved):** `.chain`, `.checked_sub`, `.find`, `.len`, `.min`
- **Called by:** `map_point` (comparer/bookmarks.rs:173)

#### `map_point`  (L173-192)
`fn map_point(src: &[char], merged: &[char], o: usize, hint: usize, after: bool) -> Option<usize>`

- **Does:** Map source offset `o` onto the merged projection by the text around it, with shrinking context windows, searching forward from `hint`.
- **Calls (crate):** `find_from` (comparer/bookmarks.rs:160) L187
- **Calls (std/external/unresolved):** `.chain`, `.concat`, `.into_iter`, `.is_empty`, `.len`, `.min`, `.saturating_sub`, `Some`
- **Called by:** `inject_side` (comparer/bookmarks.rs:266)

#### `outermost_sdt`  (L195-197)
`fn outermost_sdt(dom: &Dom, node: NodeId) -> Option<NodeId>`

- **Does:** The outermost content control around `node`.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L196
- **Calls (std/external/unresolved):** `.copied`, `.last`, `Some`, `W::name`
- **Called by:** `leave_foreign_controls` (comparer/bookmarks.rs:202)

#### `leave_foreign_controls`  (L202-228)
`fn leave_foreign_controls(dom: &mut Dom, m: &Marker, start: NodeId, end: NodeId, empty: bool)`

- **Does:** Move a placed pair out of the content controls its source bookmark was not in: the start before its outermost control, the end after its own.
- **Calls (crate):** `outermost_sdt` (comparer/bookmarks.rs:195) L207,220,224; `Dom::remove` (xmllinq/mod.rs:1102) L210,211,221,225; `Dom::add_before_self` (xmllinq/mod.rs:1114) L213,222; `Dom::add_after_self` (xmllinq/mod.rs:1131) L215,217,226
- **Called by:** `inject_side` (comparer/bookmarks.rs:266)

#### `new_markers`  (L231-243)
`fn new_markers(dom: &mut Dom, m: &Marker, id: u32) -> (NodeId, NodeId)`

- **Does:** The `w:bookmarkStart`/`w:bookmarkEnd` pair for `m` under `id`.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L233,240; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L234,235,237,238,241
- **Calls (std/external/unresolved):** `.to_string`, `Some`, `W::name`
- **Called by:** `inject_side` (comparer/bookmarks.rs:266)

#### `leave_inline_revision`  (L247-263)
`fn leave_inline_revision(dom: &mut Dom, endpoint: NodeId, after: bool)`

- **Does:** Keep a boundary endpoint beside the outermost inline revision containing its adjacent run, on the same side.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L249; `Dom::name_is` (xmllinq/mod.rs:535) L252,253; `Dom::remove` (xmllinq/mod.rs:1102) L256; `Dom::add_after_self` (xmllinq/mod.rs:1131) L258; `Dom::add_before_self` (xmllinq/mod.rs:1114) L260
- **Calls (std/external/unresolved):** `.any`, `.filter`, `.into_iter`, `.iter`, `.last`, `.take_while`, `W::del`, `W::ins`, `W::name`, `W::p`
- **Called by:** `inject_side` (comparer/bookmarks.rs:266)

#### `inject_side`  (L266-337)
`fn inject_side( dom: &mut Dom, result_root: NodeId, (src, markers): (&[char], &[&Marker]), b_side: bool, author: &str, next_id: &mut u32, )`

- **Does:** Place one side's bookmarks on the merged body's projection of that side.
- **Calls (crate):** `collect_segments` (comparer/comments.rs:477) L278; `map_point` (comparer/bookmarks.rs:173) L290,303; `new_markers` (comparer/bookmarks.rs:231) L309; `place_after_offset` (comparer/comments.rs:691) L317,326,332; `leave_inline_revision` (comparer/bookmarks.rs:247) L318,324; `place_before_offset` (comparer/comments.rs:665) L320,323,329; `Dom::add_after_self` (xmllinq/mod.rs:1131) L330; `Dom::add_before_self` (xmllinq/mod.rs:1114) L333; `leave_foreign_controls` (comparer/bookmarks.rs:202) L335
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.filter`, `.is_empty`, `.unwrap_or`
- **Called by:** `carry_matching_bookmarks` (comparer/bookmarks.rs:418); `tests::check_boundary_revision` (comparer/bookmarks.rs:466)

#### `settle_present_bookmarks`  (L342-383)
`fn settle_present_bookmarks(dom: &mut Dom, result_root: NodeId) -> (HashSet<String>, u32)`

- **Does:** Drop the `_GoBack` bookmarks that rode in with opaque content (textboxes) and renumber the others above every `w:id` in the body.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L349; `Dom::attribute` (xmllinq/mod.rs:874) L350,355,369; `Dom::name_is` (xmllinq/mod.rs:535) L354,362; `Dom::remove` (xmllinq/mod.rs:1102) L373; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L380
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.contains`, `.entry`, `.extend`, `.insert`, `.map`, `.max`, `.ok`, `.or_insert_with`, `.parse`, `.push`, `.to_string`, `.unwrap_or`, `HashMap::new`, `HashSet::new`, `Some`, `Vec::new`, `W::name`
- **Called by:** `carry_matching_bookmarks` (comparer/bookmarks.rs:418)

#### `carry_bookmarks`  (L395-410, pub)
`pub fn carry_bookmarks( dom: &mut Dom, result_root: NodeId, (pkg1, main1): (&PartFs, &str), (pkg2, main2): (&PartFs, &str), author: &str, )`

- **Does:** Carry both source documents' bookmarks into the redline body, then renumber ids above every `w:id` already present so they do not collide with revisions or comments (Word keeps bookmark and revision ids disjoint).
- **Calls (crate):** `carry_matching_bookmarks` (comparer/bookmarks.rs:418) L402
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501)

#### `carry_matching_bookmarks`  (L418-459, pub)
`pub fn carry_matching_bookmarks( dom: &mut Dom, result_root: NodeId, (pkg1, main1): (&PartFs, &str), (pkg2, main2): (&PartFs, &str), author: &str, keep: impl Fn(&str) -> bool, )`

- **Does:** [`carry_bookmarks`] for the bookmarks whose name passes `keep`, applied to one part root (the body, or a single header, footer or notes part).
- **Calls (crate):** `extract` (comparer/bookmarks.rs:82) L426,427; `settle_present_bookmarks` (comparer/bookmarks.rs:342) L430; `inject_side` (comparer/bookmarks.rs:266) L443,451
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.contains`, `.filter`, `.is_empty`, `.iter`, `.map`, `.retain`, `keep`
- **Called by:** `carry_bookmarks` (comparer/bookmarks.rs:395); `rewrite` (edit/whole.rs:130)

## Test functions (3)

- `tests::check_boundary_revision` (L466): _no doc comment_
- `tests::trailing_start_leaves_outermost_inline_revision_on_same_side` (L536): _no doc comment_
- `tests::leading_end_leaves_outermost_inline_revision_on_same_side` (L541): _no doc comment_
