# Functions and methods in `src/edit/whole.rs`

10 non-test, 7 test. Part of the [symbol manifest](../README.md).

#### `mark`  (L53-85, pub(super))
`pub(super) fn mark( dom: &mut Dom, paragraph: NodeId, (start, end): (usize, usize), (op, part): (usize, Option<String>), (find, replacement): (&str, &str), ) -> Option<Mark>`

- **Does:** Wrap the new text at projection offsets `start..end` of `paragraph` in a helper bookmark.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L60; `Dom::descendants` (xmllinq/mod.rs:668) L65; `Dom::attribute` (xmllinq/mod.rs:874) L68; `Dom::new_element` (xmllinq/mod.rs:478) L73,76; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L74,75,77; `wrap_range` (edit.rs:5960) L78
- **Calls (std/external/unresolved):** `.copied`, `.filter_map`, `.into_iter`, `.last`, `.map_or`, `.max`, `.ok`, `.parse`, `.then`, `.to_string`, `.unwrap_or`, `Some`, `W::bookmark_end`, `W::bookmark_start`, `W::id`, `W::name`
- **Macros:** `format!`
- **Called by:** `tests::a_dropped_bookmark_keeps_the_word_level_diff` (edit/whole.rs:482); `tests::a_span_crossing_a_hyperlink_is_left_alone` (edit/whole.rs:493); `tests::a_container_split_by_the_span_gets_a_second_id` (edit/whole.rs:507); `Transaction::apply` (edit.rs:4489) (?)

#### `strip`  (L89-117, pub(super))
`pub(super) fn strip(dom: &mut Dom, root: NodeId)`

- **Does:** Remove every helper bookmark under `root`, and any revision container left empty by that.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L90,99; `is_helper` (edit/whole.rs:119) L93; `Dom::attribute` (xmllinq/mod.rs:874) L97,103; `Dom::parent` (xmllinq/mod.rs:597) L108; `Dom::remove` (xmllinq/mod.rs:1102) L109,114; `Dom::name_is` (xmllinq/mod.rs:535) L111; `Dom::nodes` (xmllinq/mod.rs:604) L112
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.contains`, `.filter_map`, `.filter`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.map`, `Some`, `W::bookmark_end`, `W::bookmark_start`, `W::del`, `W::id`, `W::ins`
- **Called by:** `step_h` (comparer/lcs.rs:3621) (?); `align_content_control_ancestry` (comparer/produce.rs:331) (?); `rewrite` (edit/whole.rs:130); `tests::strip_removes_helper_bookmarks_and_the_containers_they_emptied` (edit/whole.rs:542); `Transaction::finish` (edit.rs:5077)

#### `is_helper`  (L119-122)
`fn is_helper(dom: &Dom, bookmark: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L120
- **Calls (std/external/unresolved):** `.is_some_and`, `.starts_with`, `W::name`
- **Called by:** `strip` (edit/whole.rs:89)

#### `rewrite`  (L130-197, pub(super))
`pub(super) fn rewrite( redline: &[u8], (base, marked): (&[u8], &[u8]), marks: &[Mark], author: &str, date: &str, ) -> Result<(Vec<u8>, Vec<Fallback>), EditError>`

- **Does:** Rewrite each mark's span in `redline` as one deletion then one insertion and drop every helper bookmark.
- **Calls (crate):** `Opened::open` (inspect.rs:525) L138; `err` (edit.rs:1401) L138,145,149,195; `PartFs::open` (opc/mod.rs:456) L145; `open` (ambiguous: `open` (append.rs:316); `open` (changes.rs:391)) L147; `PartFs::part_string` (opc/mod.rs:498) (by name) L150; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) (by name) L151; `.root` (ambiguous: `Dom::root` (xmllinq/mod.rs:822); `Scope::root` (xmllinq/serialize.rs:211)) L152; `carry_matching_bookmarks` (comparer/bookmarks.rs:418) L155; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L168; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L169; `rewrite_one` (edit/whole.rs:235) L183; `strip` (edit/whole.rs:89) L188; `Dom::serialize_document` (xmllinq/mod.rs:1322) (by name) L189; `PartFs::set_part` (opc/mod.rs:510) (by name) L190; `PartFs::to_zip` (opc/mod.rs:548) (by name) L192
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.collect`, `.filter_map`, `.find`, `.flat_map`, `.into_bytes`, `.is_empty`, `.iter`, `.map_err`, `.map_or`, `.max`, `.ok_or_else`, `.ok`, `.parse`, `.push`, `.starts_with`, `.to_string`, `Ok`, `Some`, `Vec::new`, `W::id`
- **Macros:** `format!`, `vec!`
- **Called by:** `apply_plan` (edit.rs:1621)

#### `Stamp::container`  (L207-214)
`fn container(&mut self, dom: &mut Dom, name: crate::xmllinq::XName) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L208; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L209,211,212
- **Calls (std/external/unresolved):** `.to_string`, `Some`, `W::author`, `W::date`, `W::id`
- **Called by:** `emit` (edit/tracked.rs:275) (?); `insert_paragraph_content` (edit/tracked.rs:574) (?); `insert_table_content` (edit/tracked.rs:593) (?); `track_properties` (edit/tracked.rs:633) (?); `track_run_format` (edit/tracked.rs:669) (?); `emit_text` (edit/tracked.rs:732) (?); `wrap_deleted` (edit/tracked.rs:949) (?); `mark` (edit/tracked.rs:1012) (?); `rewrite_one` (edit/whole.rs:235) (?)

#### `rewrite_one`  (L235-365)
`fn rewrite_one( dom: &mut Dom, body: NodeId, mark: &Mark, mut stamp: Stamp<'_>, ) -> Result<(), String>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L241,296; `Dom::attribute` (xmllinq/mod.rs:874) L244,246,263; `Dom::ancestors` (xmllinq/mod.rs:770) L250; `flatten` (edit/whole.rs:367) L255; `Dom::name_is` (xmllinq/mod.rs:535) L263; `side_text` (edit/whole.rs:433) L274,280; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L295; `Dom::set_name` (xmllinq/mod.rs:539) L297; `.container` (ambiguous: `Stamp::container` (edit/tracked.rs:40); `Stamp::container` (edit/whole.rs:207)) L307,308; `Dom::remove` (xmllinq/mod.rs:1102) L314,317; `Dom::add` (xmllinq/mod.rs:1058) L325,328,330,331,338,357...; `Dom::new_element` (xmllinq/mod.rs:478) L349; `Dom::name` (xmllinq/mod.rs:526) L349; `Dom::attributes` (xmllinq/mod.rs:885) L350; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L351,353
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.clone`, `.collect`, `.contains`, `.copied`, `.count`, `.drain`, `.enumerate`, `.expect`, `.filter_map`, `.filter`, `.find`, `.first`, `.flat_map`, `.insert`, `.into_iter`, `.iter`, `.len`, `.map`, `.ok_or_else`, `.ok_or`, `.position`, `.push`, `.rev`, `.take_while`, `.to_string`, `.unwrap_or_default`, `Err`, `HashSet::new`, `Ok`, `Some`, `Vec::new`, `W::bookmark_end`, `W::bookmark_start`, `W::del_text`, `W::del`, `W::id`, `W::ins`, `W::name`, `W::p`, `W::t`
- **Macros:** `format!`, `unreachable!`
- **Called by:** `rewrite` (edit/whole.rs:130); `tests::run` (edit/whole.rs:468)

#### `flatten`  (L367-394)
`fn flatten(dom: &Dom, p: NodeId) -> Vec<Item>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L369,385; `Dom::name_is` (xmllinq/mod.rs:535) L370,373,375; `classify` (edit/whole.rs:396) L380,388
- **Calls (std/external/unresolved):** `.push`, `Some`, `Vec::new`, `W::del`, `W::ins`, `W::p_pr`
- **Called by:** `rewrite_one` (edit/whole.rs:235)

#### `classify`  (L396-415)
`fn classify(dom: &Dom, node: NodeId, side: Kind) -> Kind`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L397,398,404; `run_text` (edit/whole.rs:418) L399; `Dom::elements` (xmllinq/mod.rs:640) L401; `named` (ambiguous: `coverage_round_three_tests::coverage_round_five_tests::named` (convert/mod.rs:50359); `named` (markdown/from_docx/header.rs:941); `named` (markdown/write.rs:5387)) L404,410
- **Calls (std/external/unresolved):** `.all`, `.any`, `.into_iter`, `.is_some`, `.iter`, `W::name`, `W::r_pr`, `W::r`
- **Called by:** `flatten` (edit/whole.rs:367)

#### `run_text`  (L418-430)
`fn run_text(dom: &Dom, run: NodeId) -> Option<String>`

- **Does:** A run's text when it holds nothing but text.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L420; `Dom::name_is` (xmllinq/mod.rs:535) L421,423,424; `Dom::value` (xmllinq/mod.rs:1169) L422
- **Calls (std/external/unresolved):** `.push_str`, `Some`, `String::new`, `W::del_text`, `W::name`, `W::r_pr`, `W::t`
- **Called by:** `classify` (edit/whole.rs:396); `side_text` (edit/whole.rs:433)

#### `side_text`  (L433-439)
`fn side_text(dom: &Dom, items: &[Item], side: Kind) -> String`

- **Does:** Text of the kept runs plus the runs of one side.
- **Calls (crate):** `run_text` (edit/whole.rs:418) L437
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.filter`, `.iter`
- **Called by:** `rewrite_one` (edit/whole.rs:235)

## Test functions (7)

- `tests::body` (L448): _no doc comment_
- `tests::mark` (L458): _no doc comment_
- `tests::run` (L468): _no doc comment_
- `tests::a_dropped_bookmark_keeps_the_word_level_diff` (L482): _no doc comment_
- `tests::a_span_crossing_a_hyperlink_is_left_alone` (L493): _no doc comment_
- `tests::a_container_split_by_the_span_gets_a_second_id` (L507): The span starts inside a comparer `w:ins` that also holds text before it; the part after the span keeps its container under a new id.
- `tests::strip_removes_helper_bookmarks_and_the_containers_they_emptied` (L542): _no doc comment_
