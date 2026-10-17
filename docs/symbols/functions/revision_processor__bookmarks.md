# Functions and methods in `src/revision_processor/bookmarks.rs`

6 non-test, 6 test. Part of the [symbol manifest](../README.md).

#### `drop_wholly_deleted_bookmarks`  (L23-89, pub(super))
`pub(super) fn drop_wholly_deleted_bookmarks(dom: &mut Dom, root: NodeId)`

- **Does:** Remove, before the accept transforms run, every bookmark whose span held only deleted characters.
- **Calls (crate):** `Dom::find_descendant_element` (xmllinq/mod.rs:692) L25; `move_from_range_deleted_elements` (revision_processor.rs:1146) L31; `Dom::ancestors` (xmllinq/mod.rs:770) L34; `descendant_and_self_tags` (revision_processor.rs:772) L43; `Dom::name` (xmllinq/mod.rs:526) L45; `paragraph_mark_is_deleted_or_moved_from` (revision_processor.rs:1757) L52; `in_deleted_row` (revision_processor/bookmarks.rs:118) L53; `Dom::attribute` (xmllinq/mod.rs:874) L56,61,84; `is_character` (revision_processor/bookmarks.rs:92) L69; `in_deleted_content` (revision_processor/bookmarks.rs:110) L70; `Dom::descendants` (xmllinq/mod.rs:668) L83; `Dom::remove` (xmllinq/mod.rs:1102) L85
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.insert`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some_and`, `.remove`, `.to_string`, `.values_mut`, `HashMap::new`, `HashSet::new`, `Some`, `W::id`, `W::name`, `W::p`, `removed_by_range`, `usize::from`
- **Called by:** `accept_revisions_for_part_content` (revision_processor.rs:3252)

#### `is_character`  (L92-107)
`fn is_character(dom: &Dom, e: NodeId, name: &XName) -> bool`

- **Does:** Does `e` stand for at least one character of the story?
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L93,96; `XName::local_name` (xmllinq/mod.rs:208) L94,99; `Dom::value_str` (xmllinq/mod.rs:1179) L94,100; `Dom::parent` (xmllinq/mod.rs:597) L102; `Dom::name` (xmllinq/mod.rs:526) L102
- **Calls (std/external/unresolved):** `.and_then`, `.is_empty`, `Some`, `W::r`
- **Called by:** `drop_wholly_deleted_bookmarks` (revision_processor/bookmarks.rs:23)

#### `in_deleted_content`  (L110-116)
`fn in_deleted_content(dom: &Dom, e: NodeId) -> bool`

- **Does:** Is `e` inside deleted or moved-from content, or a deleted row?
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L112; `Dom::name` (xmllinq/mod.rs:526) L113; `row_is_deleted` (revision_processor/bookmarks.rs:124) L114
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_some_and`, `W::del`, `W::move_from`, `W::tr`
- **Called by:** `drop_wholly_deleted_bookmarks` (revision_processor/bookmarks.rs:23)

#### `in_deleted_row`  (L118-122)
`fn in_deleted_row(dom: &Dom, p: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L119; `row_is_deleted` (revision_processor/bookmarks.rs:124) L121
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `Some`, `W::tr`
- **Called by:** `drop_wholly_deleted_bookmarks` (revision_processor/bookmarks.rs:23)

#### `row_is_deleted`  (L124-127)
`fn row_is_deleted(dom: &Dom, tr: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L125,126
- **Calls (std/external/unresolved):** `.is_some_and`, `.is_some`, `W::del`, `W::tr_pr`
- **Called by:** `in_deleted_content` (revision_processor/bookmarks.rs:110); `in_deleted_row` (revision_processor/bookmarks.rs:118)

#### `drop_unpaired_bookmarks`  (L132-152, pub(super))
`pub(super) fn drop_unpaired_bookmarks(dom: &mut Dom, root: NodeId)`

- **Does:** Remove the bookmark markers left without their partner: Word's range model has no half bookmark, and the accept transforms can drop one side with the content around it (a deleted row, a moved-from run).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L136,146; `Dom::attribute` (xmllinq/mod.rs:874) L138,147; `reordered_retained_control_identity_regression::ids` (comparer/produce.rs:2434) (by name) L141; `Dom::remove` (xmllinq/mod.rs:1102) L148
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter_map`, `.into_iter`, `.is_some_and`, `.map`, `Some`, `W::id`, `W::name`
- **Called by:** `accept_revisions_for_part_content` (revision_processor.rs:3252)

## Test functions (6)

- `move_range_bookmark_regressions::accept` (L161): _no doc comment_
- `move_range_bookmark_regressions::names` (L172): _no doc comment_
- `move_range_bookmark_regressions::range_only_moved_nonempty_bookmark_disappears_but_original_empty_anchor_survives` (L180): _no doc comment_
- `move_range_bookmark_regressions::partially_surviving_bookmark_keeps_both_anchors_after_range_only_move` (L194): _no doc comment_
- `move_range_bookmark_regressions::unmatched_move_range_keeps_nonempty_bookmark_and_its_source_content` (L211): _no doc comment_
- `move_range_bookmark_regressions::entering_only_paragraph_properties_does_not_delete_a_bookmarks_live_character` (L221): _no doc comment_
