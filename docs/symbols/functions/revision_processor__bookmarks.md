# Functions and methods in `src/revision_processor/bookmarks.rs`

8 non-test, 6 test. Part of the [symbol manifest](../README.md).

#### `drop_wholly_deleted_bookmarks`  (L23-29, pub(super))
`pub(super) fn drop_wholly_deleted_bookmarks(dom: &mut Dom, root: NodeId)`

- **Does:** Remove, before the accept transforms run, every bookmark whose span held only deleted characters.
- **Calls (crate):** `wholly_deleted_endpoints` (revision_processor/bookmarks.rs:47) L25; `Dom::remove` (xmllinq/mod.rs:1118) L27
- **Calls (std/external/unresolved):** `W::name`
- **Called by:** `accept_content` (revision_processor.rs:3510)

#### `wholly_deleted_ranges`  (L32-42, pub(super))
`pub(super) fn wholly_deleted_ranges( dom: &Dom, root: NodeId, start: &XName, end: &XName, ) -> HashSet<String>`

- **Does:** Comments also need the id to prune their separate reference and definition.
- **Calls (crate):** `wholly_deleted_endpoints` (revision_processor/bookmarks.rs:47) L38; `Dom::attribute` (xmllinq/mod.rs:874) L40
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.into_iter`, `.map`, `W::id`
- **Called by:** `drop_wholly_deleted_comments` (revision_processor/comments.rs:71); `keep_wholly_deleted_comments` (revision_processor/comments.rs:96)

#### `wholly_deleted_endpoints`  (L47-109, pub(super))
`pub(super) fn wholly_deleted_endpoints( dom: &Dom, root: NodeId, start: &XName, end: &XName, ) -> HashSet<NodeId>`

- **Does:** Nonempty annotation ranges whose characters all disappear when accepting.
- **Calls (crate):** `Dom::find_descendant_element` (xmllinq/mod.rs:692) L53; `move_from_range_deleted_elements` (revision_processor.rs:1370) L59; `Dom::ancestors` (xmllinq/mod.rs:770) L62; `descendant_and_self_tags` (revision_processor.rs:810) L72; `Dom::name` (xmllinq/mod.rs:526) L74; `paragraph_mark_is_deleted_or_moved_from` (revision_processor.rs:1981) L81; `in_deleted_row` (revision_processor/bookmarks.rs:138) L82; `Dom::attribute` (xmllinq/mod.rs:874) L85,90; `is_character` (revision_processor/bookmarks.rs:112) L98; `in_deleted_content` (revision_processor/bookmarks.rs:130) L99
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.contains`, `.entry`, `.extend`, `.flatten`, `.get_mut`, `.into_iter`, `.is_none`, `.or_default`, `.push`, `.to_string`, `.values_mut`, `HashMap::new`, `HashSet::new`, `Some`, `W::id`, `W::p`, `removed_by_range`, `usize::from`
- **Called by:** `drop_wholly_deleted_bookmarks` (revision_processor/bookmarks.rs:23); `wholly_deleted_ranges` (revision_processor/bookmarks.rs:32); `drop_wholly_deleted_permissions` (revision_processor.rs:2304)

#### `is_character`  (L112-127, pub(super))
`pub(super) fn is_character(dom: &Dom, e: NodeId, name: &XName) -> bool`

- **Does:** Does `e` stand for at least one character of the story?
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L113,116; `XName::local_name` (xmllinq/mod.rs:208) L114,119; `Dom::value_str` (xmllinq/mod.rs:1195) L114,120; `Dom::parent` (xmllinq/mod.rs:597) L122; `Dom::name` (xmllinq/mod.rs:526) L122
- **Calls (std/external/unresolved):** `.and_then`, `.is_empty`, `Some`, `W::r`
- **Called by:** `wholly_deleted_endpoints` (revision_processor/bookmarks.rs:47); `keep_wholly_deleted_comments` (revision_processor/comments.rs:96)

#### `in_deleted_content`  (L130-136)
`fn in_deleted_content(dom: &Dom, e: NodeId) -> bool`

- **Does:** Is `e` inside deleted or moved-from content, or a deleted row?
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L132; `Dom::name` (xmllinq/mod.rs:526) L133; `row_is_deleted` (revision_processor/bookmarks.rs:144) L134
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_some_and`, `W::del`, `W::move_from`, `W::tr`
- **Called by:** `wholly_deleted_endpoints` (revision_processor/bookmarks.rs:47)

#### `in_deleted_row`  (L138-142)
`fn in_deleted_row(dom: &Dom, p: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L139; `row_is_deleted` (revision_processor/bookmarks.rs:144) L141
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `Some`, `W::tr`
- **Called by:** `wholly_deleted_endpoints` (revision_processor/bookmarks.rs:47)

#### `row_is_deleted`  (L144-147)
`fn row_is_deleted(dom: &Dom, tr: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L145,146
- **Calls (std/external/unresolved):** `.is_some_and`, `.is_some`, `W::del`, `W::tr_pr`
- **Called by:** `in_deleted_content` (revision_processor/bookmarks.rs:130); `in_deleted_row` (revision_processor/bookmarks.rs:138)

#### `drop_unpaired_bookmarks`  (L152-172, pub(super))
`pub(super) fn drop_unpaired_bookmarks(dom: &mut Dom, root: NodeId)`

- **Does:** Remove the bookmark markers left without their partner: Word's range model has no half bookmark, and the accept transforms can drop one side with the content around it (a deleted row, a moved-from run).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L156,166; `Dom::attribute` (xmllinq/mod.rs:874) L158,167; `reordered_retained_control_identity_regression::ids` (comparer/produce.rs:2433) (by name) L161; `Dom::remove` (xmllinq/mod.rs:1118) L168
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter_map`, `.into_iter`, `.is_some_and`, `.map`, `Some`, `W::id`, `W::name`
- **Called by:** `accept_content` (revision_processor.rs:3510)

## Test functions (6)

- `move_range_bookmark_regressions::accept` (L181): _no doc comment_
- `move_range_bookmark_regressions::names` (L192): _no doc comment_
- `move_range_bookmark_regressions::range_only_moved_nonempty_bookmark_disappears_but_original_empty_anchor_survives` (L200): _no doc comment_
- `move_range_bookmark_regressions::partially_surviving_bookmark_keeps_both_anchors_after_range_only_move` (L214): _no doc comment_
- `move_range_bookmark_regressions::unmatched_move_range_keeps_nonempty_bookmark_and_its_source_content` (L231): _no doc comment_
- `move_range_bookmark_regressions::entering_only_paragraph_properties_does_not_delete_a_bookmarks_live_character` (L241): _no doc comment_
