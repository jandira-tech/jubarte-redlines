# Functions and methods in `src/comparer/textbox.rs`

8 non-test, 3 test. Part of the [symbol manifest](../README.md).

#### `diff_inside_replaced_text_boxes`  (L33-79, pub)
`pub fn diff_inside_replaced_text_boxes( dom: &mut Dom, root: NodeId, settings: &WmlComparerSettings, )`

- **Does:** Replace each deleted shape that sits beside an inserted shape differing from it only in text-box text with the inserted shape, each of its text boxes holding a compare of the old story against the new one.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L39; `Dom::ancestors` (xmllinq/mod.rs:770) L43; `Dom::name_is` (xmllinq/mod.rs:535) L45,53; `revision_clusters` (comparer/textbox.rs:83) L49; `Dom::elements` (xmllinq/mod.rs:640) L54; `shape_run` (comparer/textbox.rs:108) L55; `shell` (comparer/textbox.rs:168) L62,65; `Dom::name` (xmllinq/mod.rs:526) L65; `redline_boxes` (comparer/textbox.rs:204) L72; `detach_run` (comparer/textbox.rs:127) L73; `lift_run` (comparer/textbox.rs:139) L74
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.filter_map`, `.filter`, `.find`, `.flat_map`, `.into_iter`, `.iter`, `.len`, `Some`, `W::del`, `W::ins`, `W::name`, `W::p`, `boxes`
- **Called by:** `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209); `complete_shape_revision_contract_tests::matching_opaque_shapes_keep_every_neighbor_when_revision_wrappers_split` (comparer/textbox.rs:260); `complete_shape_revision_contract_tests::unrelated_or_mixed_shape_carriers_retain_complete_source_revisions` (comparer/textbox.rs:310)

#### `revision_clusters`  (L83-104)
`fn revision_clusters(dom: &Dom, p: NodeId) -> Vec<Vec<NodeId>>`

- **Does:** Each maximal run of adjacent `w:ins`/`w:del` children of a paragraph that holds both kinds.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L88,89,96; `Dom::elements` (xmllinq/mod.rs:640) L95; `flush` (markdown/view_stories.rs:755) (by name) L99,102
- **Calls (std/external/unresolved):** `.any`, `.clear`, `.iter`, `.push`, `Vec::new`, `W::del`, `W::ins`, `std::mem::take`
- **Called by:** `diff_inside_replaced_text_boxes` (comparer/textbox.rs:33)

#### `shape_run`  (L108-124)
`fn shape_run(dom: &Dom, run: NodeId) -> Option<(NodeId, NodeId)>`

- **Does:** A run whose only content is one shape (`mc:AlternateContent`, `w:drawing` or `w:pict`): `(run, shape)`.
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L109,115,120,121,122; `Dom::elements` (xmllinq/mod.rs:640) L112
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.then_some`, `MC::name`, `W::drawing`, `W::pict`, `W::r_pr`, `W::r`
- **Called by:** `diff_inside_replaced_text_boxes` (comparer/textbox.rs:33)

#### `detach_run`  (L127-135)
`fn detach_run(dom: &mut Dom, run: NodeId)`

- **Does:** Remove a run from its revision wrapper, and the wrapper once empty.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L128; `Dom::remove` (xmllinq/mod.rs:1102) L129,133; `Dom::elements` (xmllinq/mod.rs:640) L131
- **Calls (std/external/unresolved):** `.is_empty`
- **Called by:** `diff_inside_replaced_text_boxes` (comparer/textbox.rs:33)

#### `lift_run`  (L139-162)
`fn lift_run(dom: &mut Dom, run: NodeId)`

- **Does:** Move a run out of its revision wrapper to the same place, splitting the wrapper around it.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L140; `Dom::nodes` (xmllinq/mod.rs:604) L143; `Dom::new_element` (xmllinq/mod.rs:478) L147; `Dom::name` (xmllinq/mod.rs:526) L147; `Dom::attributes` (xmllinq/mod.rs:885) L148; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L149; `Dom::remove` (xmllinq/mod.rs:1102) L152,157,160; `Dom::add` (xmllinq/mod.rs:1058) L153; `Dom::add_after_self` (xmllinq/mod.rs:1131) L155,158; `Dom::elements` (xmllinq/mod.rs:640) L159
- **Calls (std/external/unresolved):** `.is_empty`, `.iter`, `.position`, `.unwrap_or_else`, `.unwrap_or`, `Some`
- **Called by:** `diff_inside_replaced_text_boxes` (comparer/textbox.rs:33)

#### `shell`  (L168-185)
`fn shell(dom: &mut Dom, shape: NodeId) -> String`

- **Does:** A shape with its text boxes emptied and the ids the id fixups renumber per copy left out (shape and drawing ids, and a VML shape's `#id` reference to its shapetype), so two boxes that differ only in text compare equal.
- **Calls (crate):** `Dom::clone_subtree` (xmllinq/mod.rs:1215) L169; `Dom::descendants` (xmllinq/mod.rs:668) L170; `Dom::nodes` (xmllinq/mod.rs:604) L171; `Dom::remove` (xmllinq/mod.rs:1102) L172; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L175; `Dom::attributes` (xmllinq/mod.rs:885) L176; `XName::local_name` (xmllinq/mod.rs:208) (by name) L177,178; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L180; `Dom::serialize_element` (xmllinq/mod.rs:1299) L184
- **Calls (std/external/unresolved):** `.starts_with`, `Some`, `W::name`
- **Macros:** `matches!`
- **Called by:** `diff_inside_replaced_text_boxes` (comparer/textbox.rs:33)

#### `story`  (L189-200)
`fn story(dom: &mut Dom, text_box: NodeId, accept: bool) -> NodeId`

- **Does:** One text box's paragraphs as a story of their own, with its revisions accepted (the new box) or rejected (the old box).
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L190; `Dom::elements` (xmllinq/mod.rs:640) L191; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L192; `Dom::add` (xmllinq/mod.rs:1058) L193; `accept_revisions_document` (revision_processor.rs:502) L196; `reject_revisions_document` (revision_processor.rs:729) L198
- **Calls (std/external/unresolved):** `W::name`
- **Called by:** `redline_boxes` (comparer/textbox.rs:204)

#### `redline_boxes`  (L204-248)
`fn redline_boxes(dom: &mut Dom, old: NodeId, new: NodeId, settings: &WmlComparerSettings) -> bool`

- **Does:** Fill each text box of `new` with a compare of `old`'s story against it.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L206,207; `Dom::nodes` (xmllinq/mod.rs:604) L215,239; `story` (comparer/textbox.rs:189) L219,220; `compare_bodies_faithful` (comparer/mod.rs:109) L221; `Dom::element` (xmllinq/mod.rs:655) L222; `Dom::elements` (xmllinq/mod.rs:640) L225; `Dom::name_is` (xmllinq/mod.rs:535) L228; `Dom::remove` (xmllinq/mod.rs:1102) L240,243; `Dom::add` (xmllinq/mod.rs:1058) L244
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_empty`, `.iter`, `.len`, `.push`, `.zip`, `Some`, `Vec::with_capacity`, `W::body`, `W::name`, `W::sect_pr`
- **Called by:** `diff_inside_replaced_text_boxes` (comparer/textbox.rs:33)

## Test functions (3)

- `complete_shape_revision_contract_tests::document` (L254): _no doc comment_
- `complete_shape_revision_contract_tests::matching_opaque_shapes_keep_every_neighbor_when_revision_wrappers_split` (L260): _no doc comment_
- `complete_shape_revision_contract_tests::unrelated_or_mixed_shape_carriers_retain_complete_source_revisions` (L310): _no doc comment_
