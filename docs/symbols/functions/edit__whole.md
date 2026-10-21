# Functions and methods in `src/edit/whole.rs`

11 non-test, 7 test. Part of the [symbol manifest](../README.md).

#### `mark`  (L53-85, pub(super))
`pub(super) fn mark( dom: &mut Dom, paragraph: NodeId, (start, end): (usize, usize), (op, part): (usize, Option<String>), (find, replacement): (&str, &str), ) -> Option<Mark>`

- **Does:** Wrap the new text at projection offsets `start..end` of `paragraph` in a helper bookmark.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L60; `Dom::descendants` (xmllinq/mod.rs:668) L65; `Dom::attribute` (xmllinq/mod.rs:874) L68; `Dom::new_element` (xmllinq/mod.rs:478) L73,76; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L74,75,77; `wrap_range` (edit.rs:7932) L78
- **Calls (std/external/unresolved):** `.copied`, `.filter_map`, `.into_iter`, `.last`, `.map_or`, `.max`, `.ok`, `.parse`, `.then`, `.to_string`, `.unwrap_or`, `Some`, `W::bookmark_end`, `W::bookmark_start`, `W::id`, `W::name`
- **Macros:** `format!`
- **Called by:** `tests::a_dropped_bookmark_keeps_the_word_level_diff` (edit/whole.rs:513); `tests::a_span_crossing_a_hyperlink_is_left_alone` (edit/whole.rs:524); `tests::a_container_split_by_the_span_gets_a_second_id` (edit/whole.rs:538); `Transaction::apply` (edit.rs:5882) (?)

#### `strip`  (L89-117, pub(super))
`pub(super) fn strip(dom: &mut Dom, root: NodeId)`

- **Does:** Remove every helper bookmark under `root`, and any revision container left empty by that.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L90,99; `is_helper` (edit/whole.rs:119) L93; `Dom::attribute` (xmllinq/mod.rs:874) L97,103; `Dom::parent` (xmllinq/mod.rs:597) L108; `Dom::remove` (xmllinq/mod.rs:1118) L109,114; `Dom::name_is` (xmllinq/mod.rs:535) L111; `Dom::nodes` (xmllinq/mod.rs:604) L112
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.contains`, `.filter_map`, `.filter`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.map`, `Some`, `W::bookmark_end`, `W::bookmark_start`, `W::del`, `W::id`, `W::ins`
- **Called by:** `step_h` (comparer/lcs.rs:3621) (?); `align_content_control_ancestry` (comparer/produce.rs:331) (?); `rewrite` (edit/whole.rs:130); `tests::strip_removes_helper_bookmarks_and_the_containers_they_emptied` (edit/whole.rs:573); `Transaction::finish` (edit.rs:6598)

#### `is_helper`  (L119-122)
`fn is_helper(dom: &Dom, bookmark: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L120
- **Calls (std/external/unresolved):** `.is_some_and`, `.starts_with`, `W::name`
- **Called by:** `strip` (edit/whole.rs:89)

#### `rewrite`  (L130-192, pub(super))
`pub(super) fn rewrite( redline: &[u8], (base, marked): (&[u8], &[u8]), marks: &[Mark], author: &str, date: &str, ) -> Result<(Vec<u8>, Vec<Fallback>), EditError>`

- **Does:** Rewrite each mark's span in `redline` as one deletion then one insertion and drop every helper bookmark.
- **Calls (crate):** `Opened::open` (inspect.rs:735) L138; `err` (edit.rs:1770) L138,145,149,190; `PartFs::open` (opc/mod.rs:585) L145; `open` (ambiguous: `open` (append.rs:316); `open` (changes.rs:1031); `open` (edit/anchor_error.rs:229)) L147; `PartFs::part_string` (opc/mod.rs:636) (by name) L150; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) (by name) L151; `.root` (ambiguous: `Dom::root` (xmllinq/mod.rs:822); `Scope::root` (xmllinq/serialize.rs:211)) L152; `carry_matching_bookmarks` (comparer/bookmarks.rs:483) L155; `RevisionIds::new` (revision_processor.rs:3777) L166; `rewrite_one` (edit/whole.rs:229) L178; `strip` (edit/whole.rs:89) L183; `Dom::serialize_document` (xmllinq/mod.rs:1338) (by name) L184; `PartFs::set_part` (opc/mod.rs:661) (by name) L185; `PartFs::to_zip` (opc/mod.rs:749) (by name) L187
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.collect`, `.filter_map`, `.find`, `.into_bytes`, `.is_empty`, `.iter`, `.map_err`, `.map_or`, `.ok_or_else`, `.push`, `.starts_with`, `.to_string`, `Ok`, `Some`, `Vec::new`
- **Macros:** `format!`, `vec!`
- **Called by:** `produce` (edit/prepared.rs:928)

#### `Stamp::container`  (L202-208)
`fn container(&mut self, dom: &mut Dom, name: crate::xmllinq::XName) -> NodeId`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L203; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L204,205,206; `RevisionIds::allocate` (revision_processor.rs:3808) L204
- **Calls (std/external/unresolved):** `.to_string`, `Some`, `W::author`, `W::date`, `W::id`
- **Called by:** `emit` (edit/tracked.rs:452) (?); `insert_paragraph_content` (edit/tracked.rs:829) (?); `insert_table_content` (edit/tracked.rs:848) (?); `track_properties` (edit/tracked.rs:888) (?); `track_run_format` (edit/tracked.rs:924) (?); `emit_text` (edit/tracked.rs:1035) (?); `wrap_deleted` (edit/tracked.rs:1609) (?); `mark` (edit/tracked.rs:1682) (?); `rewrite_one` (edit/whole.rs:229) (?)

#### `rewrite_one`  (L229-368)
`fn rewrite_one( dom: &mut Dom, body: NodeId, mark: &Mark, mut stamp: Stamp<'_>, ) -> Result<(), String>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L235,294; `Dom::attribute` (xmllinq/mod.rs:874) L238,240,257; `Dom::ancestors` (xmllinq/mod.rs:770) L244; `flatten` (edit/whole.rs:398) L249; `Dom::name_is` (xmllinq/mod.rs:535) L257; `side_text` (edit/whole.rs:464) L268,274; `clone_with_fresh_property_ids` (revision_processor.rs:3826) L289; `Dom::set_name` (xmllinq/mod.rs:539) L295; `settle_format` (edit/whole.rs:375) L297,298; `.container` (ambiguous: `Stamp::container` (edit/tracked.rs:50); `Stamp::container` (edit/whole.rs:202)) L307,308; `Dom::remove` (xmllinq/mod.rs:1118) L314,317; `Dom::add` (xmllinq/mod.rs:1074) L325,328,330,331,338,360...; `Dom::new_element` (xmllinq/mod.rs:478) L349; `Dom::name` (xmllinq/mod.rs:526) L349; `Dom::attributes` (xmllinq/mod.rs:899) L350; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L351,353; `RevisionIds::allocate` (revision_processor.rs:3808) L356
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.clone`, `.collect`, `.contains`, `.copied`, `.count`, `.drain`, `.enumerate`, `.expect`, `.filter_map`, `.filter`, `.find`, `.first`, `.flat_map`, `.insert`, `.into_iter`, `.iter`, `.len`, `.map`, `.ok_or_else`, `.ok_or`, `.position`, `.push`, `.rev`, `.take_while`, `.to_string`, `.unwrap_or_default`, `Err`, `HashSet::new`, `Ok`, `Some`, `Vec::new`, `W::bookmark_end`, `W::bookmark_start`, `W::del_text`, `W::del`, `W::id`, `W::ins`, `W::name`, `W::p`, `W::t`
- **Macros:** `format!`, `unreachable!`
- **Called by:** `rewrite` (edit/whole.rs:130); `tests::run` (edit/whole.rs:499)

#### `settle_format`  (L375-396)
`fn settle_format(dom: &mut Dom, run: NodeId, old: bool)`

- **Does:** A kept run's formatting change, settled for one side of the rewrite: the deleted copy (`old`) shows the text as the source had it, in the change's old properties, and the inserted run as it is now.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L376,379,382; `Dom::remove` (xmllinq/mod.rs:1118) L383,388,392; `Dom::elements` (xmllinq/mod.rs:640) L387,391; `Dom::add` (xmllinq/mod.rs:1074) L393
- **Calls (std/external/unresolved):** `W::r_pr_change`, `W::r_pr`
- **Called by:** `rewrite_one` (edit/whole.rs:229)

#### `flatten`  (L398-425)
`fn flatten(dom: &Dom, p: NodeId) -> Vec<Item>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L400,416; `Dom::name_is` (xmllinq/mod.rs:535) L401,404,406; `classify` (edit/whole.rs:427) L411,419
- **Calls (std/external/unresolved):** `.push`, `Some`, `Vec::new`, `W::del`, `W::ins`, `W::p_pr`
- **Called by:** `rewrite_one` (edit/whole.rs:229)

#### `classify`  (L427-446)
`fn classify(dom: &Dom, node: NodeId, side: Kind) -> Kind`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name_is` (xmllinq/mod.rs:535) L428,429,435; `run_text` (edit/whole.rs:449) L430; `Dom::elements` (xmllinq/mod.rs:640) L432; `named` (ambiguous, 5 candidates) L435,441
- **Calls (std/external/unresolved):** `.all`, `.any`, `.into_iter`, `.is_some`, `.iter`, `W::name`, `W::r_pr`, `W::r`
- **Called by:** `flatten` (edit/whole.rs:398)

#### `run_text`  (L449-461)
`fn run_text(dom: &Dom, run: NodeId) -> Option<String>`

- **Does:** A run's text when it holds nothing but text.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L451; `Dom::name_is` (xmllinq/mod.rs:535) L452,454,455; `Dom::value` (xmllinq/mod.rs:1185) L453
- **Calls (std/external/unresolved):** `.push_str`, `Some`, `String::new`, `W::del_text`, `W::name`, `W::r_pr`, `W::t`
- **Called by:** `classify` (edit/whole.rs:427); `side_text` (edit/whole.rs:464)

#### `side_text`  (L464-470)
`fn side_text(dom: &Dom, items: &[Item], side: Kind) -> String`

- **Does:** Text of the kept runs plus the runs of one side.
- **Calls (crate):** `run_text` (edit/whole.rs:449) L468
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.filter`, `.iter`
- **Called by:** `rewrite_one` (edit/whole.rs:229)

## Test functions (7)

- `tests::body` (L479): _no doc comment_
- `tests::mark` (L489): _no doc comment_
- `tests::run` (L499): _no doc comment_
- `tests::a_dropped_bookmark_keeps_the_word_level_diff` (L513): _no doc comment_
- `tests::a_span_crossing_a_hyperlink_is_left_alone` (L524): _no doc comment_
- `tests::a_container_split_by_the_span_gets_a_second_id` (L538): The span starts inside a comparer `w:ins` that also holds text before it; the part after the span keeps its container under a new id.
- `tests::strip_removes_helper_bookmarks_and_the_containers_they_emptied` (L573): _no doc comment_
