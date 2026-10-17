# Functions and methods in `src/edit/origin.rs`

14 non-test, 2 test. Part of the [symbol manifest](../README.md).

#### `Ids::paragraph_id`  (L48-88, pub(super))
`pub(super) fn paragraph_id(&self, id: &str) -> Option<Result<String, String>>`

- **Does:** The paragraph id [`crate::ids::paragraph_id`] prints for what `id` names, as the agent view prints them: `p3` and `header1.p1` as given, `header1.p0` → `header1` (the part stem, Word's own numbering), `t0.row1.cell2[.p1]` → that cell's first (or K-th) paragraph (`p40`).
- **Calls (crate):** `parse_location` (ids.rs:326) L50,71; `paragraph_id` (ids.rs:410) L65,78; `public_memory_anchor_boundary_tests::paragraph` (comments.rs:1365) L67,76; `Ids::table_paragraph` (edit/origin.rs:95) L68
- **Calls (std/external/unresolved):** `.as_slice`, `.as_str`, `.contains`, `.err`, `.join`, `.map_or_else`, `.map`, `.parse`, `.to_string`, `Err`, `Ok`, `Some`
- **Macros:** `format!`
- **Called by:** `Package::heading_skip` (audit.rs:532) (?); `Package::literal_bullet` (audit.rs:636) (?); `Package::empty_spacers` (audit.rs:661) (?); `Transaction::cell_control` (edit/controls.rs:298) (?); `Ids::position` (edit/origin.rs:168); `paragraph_ids` (edit.rs:1365); `Transaction::resolve_comment_text` (edit.rs:2242) (?); `Transaction::resolve_one` (edit.rs:2528) (?); `Transaction::resolve_list` (edit.rs:3563) (?); `Transaction::select` (edit.rs:3778) (?)

#### `Ids::table_paragraph`  (L95-163)
`fn table_paragraph( &self, head: &str, n: usize, row: usize, cell: usize, index: usize, ) -> Result<String, String>`

- **Does:** Table `n`, row `row`, cell `cell`: the id of that cell's `index`-th own paragraph.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L106; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L109,120,131,142; `nearest` (ambiguous: `nearest` (capabilities/discovery.rs:161); `nearest` (debug/render.rs:375); `nearest` (inspect/tables.rs:43)) L112,123,134; `in_text_box` (inspect.rs:721) L112,147; `plural` (markdown/from_docx/header.rs:201) (by name) L117,128,139,152; `Ids::id` (edit/origin.rs:213) L162
- **Calls (std/external/unresolved):** `.collect`, `.copied`, `.filter`, `.first`, `.get`, `.into_iter`, `.is_none`, `.iter`, `.len`, `.ok_or_else`, `.position`, `Err`, `Ok`, `Some`, `W::name`, `W::p`, `W::tbl`, `W::tc`
- **Macros:** `format!`
- **Called by:** `Ids::paragraph_id` (edit/origin.rs:48)

#### `Ids::position`  (L168-210, pub(super))
`pub(super) fn position(&self, selector: &Selector) -> Result<Result<usize, usize>, Miss>`

- **Does:** The paragraph a positional selector names, as an index into `paragraph_nodes`; `None` for a text selector, and the story it searches.
- **Calls (crate):** `Ids::paragraph_id` (edit/origin.rs:48) L172; `paragraph_place` (ids.rs:417) L177
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.collect`, `.filter`, `.get`, `.iter`, `.join`, `.len`, `.position`, `.to_string`, `.unwrap_or`, `Err`, `Ok`, `Some`, `not_found`
- **Macros:** `format!`
- **Called by:** none found in src

#### `Ids::id`  (L213-216, pub(super))
`pub(super) fn id(&self, global: usize) -> String`

- **Does:** The id of a paragraph ([`crate::ids::paragraph_id`]).
- **Calls (crate):** `paragraph_id` (ids.rs:410) L215
- **Called by:** `Ids::table_paragraph` (edit/origin.rs:95); `Origin::follow` (edit/origin.rs:410) (?); `Origin::moves` (edit/origin.rs:438) (?)

#### `Origin::had_comment`  (L248-251, pub(super))
`pub(super) fn had_comment(&self, id: u32) -> bool`

- **Does:** Whether the source had comment `id`: settling the plan's revisions removes a comment whose anchor was text they took away.
- **Calls (crate):** `CommentFamily::load` (comments.rs:429) L249
- **Calls (std/external/unresolved):** `.contains`, `.is_ok_and`
- **Called by:** `Transaction::resolve_thread` (edit.rs:3342) (?)

#### `Origin::build`  (L256-374, pub(super))
`pub(super) fn build( source: &[u8], plan: &EditPlan, settled_ids: &[String], settled_story: &[(usize, usize)], ) -> Result<Self, EditError>`

- **Does:** Map the paragraphs of `source` onto those of the document settling it by `plan` gives, whose stories are `settled_ids` and whose paragraphs are `settled_story`.
- **Calls (crate):** `Opened::open` (inspect.rs:525) L262; `open` (ambiguous: `open` (append.rs:316); `open` (changes.rs:391)) L263,272,308; `load_stories` (edit.rs:1311) L268,277,313; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L279,292; `Dom::set_attribute_value` (xmllinq/mod.rs:961) (by name) L280,284; `body_control_nodes` (inspect.rs:756) L290,339; `tag_control` (edit/origin.rs:529) L297; `Dom::serialize_document` (xmllinq/mod.rs:1322) (by name) L301; `PartFs::set_part` (opc/mod.rs:510) (by name) L302; `PartFs::to_zip` (opc/mod.rs:548) (by name) L304; `err` (edit.rs:1401) L307,320; `settle` (edit.rs:1260) L308; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L329,347; `collect_controls` (inspect.rs:764) L334; `Dom::element` (xmllinq/mod.rs:655) (by name) L343,346; `fates` (edit/origin.rs:462) L357; `project_paragraph` (inspect.rs:1024) L361,362
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.checked_sub`, `.clone`, `.collect`, `.enumerate`, `.eq`, `.get_mut`, `.into_bytes`, `.into_iter`, `.iter`, `.len`, `.map_err`, `.map_or`, `.map`, `.ok`, `.parse`, `.position`, `.then_some`, `.to_string`, `Err`, `Ok`, `Some`, `W14::name`, `W::name`, `W::p`, `W::sdt_pr`, `W::sdt`, `W::val`, `usize::from_str_radix`
- **Macros:** `format!`, `vec!`
- **Called by:** `Transaction::start` (edit.rs:2004); `run` (update.rs:186) (?)

#### `Origin::control_ids`  (L377-379, pub(super))
`pub(super) fn control_ids(&self) -> &[String]`

- **Does:** The source's body content control ids, in document order.
- **Calls:** none
- **Called by:** `Transaction::select_control` (edit/controls.rs:205) (?)

#### `Origin::control_source`  (L382-385, pub(super))
`pub(super) fn control_source(&self, post: usize) -> Option<&str>`

- **Does:** The source id of settled control `post`, when it came from one.
- **Calls (std/external/unresolved):** `.as_str`, `.iter`, `.position`, `Some`
- **Called by:** `Transaction::select_control` (edit/controls.rs:205) (?)

#### `Origin::control`  (L390-395, pub(super))
`pub(super) fn control(&self, id: &str) -> Option<Result<usize, String>>`

- **Does:** The settled control (its index among the settled body's controls) the source control `id` became; `None` when the source has no control `id`.
- **Calls (std/external/unresolved):** `.iter`, `.ok_or_else`, `.position`, `Some`
- **Macros:** `format!`
- **Called by:** `Transaction::select_control` (edit/controls.rs:205) (?)

#### `Origin::ids`  (L397-405, pub(super))
`pub(super) fn ids(&self) -> Ids<'_>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`
- **Called by:** `validate_matches` (cli.rs:2566) (?); `Transaction::cell_control` (edit/controls.rs:298) (?); `Origin::follow` (edit/origin.rs:410); `Origin::moves` (edit/origin.rs:438); `Transaction::select` (edit.rs:3778) (?)

#### `Origin::follow`  (L410-434, pub(super))
`pub(super) fn follow( &self, source: usize, merged_ok: bool, settled_id: impl Fn(usize) -> String, ) -> Result<usize, Miss>`

- **Does:** The settled paragraph source paragraph `source` became.
- **Calls (crate):** `Ids::id` (edit/origin.rs:213) (by name) L416; `Origin::ids` (edit/origin.rs:397) L416
- **Calls (std/external/unresolved):** `.into`, `Err`, `Ok`, `settled_id`
- **Macros:** `format!`
- **Called by:** `Transaction::select` (edit.rs:3778) (?)

#### `Origin::moves`  (L438-452, pub(super))
`pub(super) fn moves(&self, settled_id: impl Fn(usize) -> String) -> Vec<super::ParagraphMove>`

- **Does:** Every source id that names another paragraph, or none, once the plan's revisions are settled, in source order.
- **Calls (crate):** `Origin::ids` (edit/origin.rs:397) L439; `Ids::id` (edit/origin.rs:213) (by name) L444
- **Calls (std/external/unresolved):** `.as_deref`, `.as_str`, `.collect`, `.enumerate`, `.filter_map`, `.iter`, `.then_some`, `Some`, `settled_id`
- **Called by:** `Transaction::report` (edit.rs:2346) (?)

#### `fates`  (L462-526)
`fn fates( tags: &[Option<usize>], source_story: impl Fn(usize) -> String, settled_story: impl Fn(usize) -> String, source_text: impl Fn(usize) -> String, settled_text: impl Fn(usize) -> String, count: usize, ) -> Vec<Fate>`

- **Does:** Where each of `count` source paragraphs went, from the source tag each settled paragraph carries (`tags`).
- **Calls (std/external/unresolved):** `.as_slice`, `.clone`, `.collect`, `.enumerate`, `.filter`, `.find_map`, `.find`, `.get`, `.is_empty`, `.is_some`, `.iter`, `.len`, `.map_or`, `.rev`, `.unwrap_or`, `Fate::Kept`, `Fate::Merged`, `Some`, `settled_story`, `settled_text`, `source_story`, `source_text`
- **Macros:** `vec!`
- **Called by:** `Origin::build` (edit/origin.rs:256); `tests::untagged_paragraphs_are_merges_of_the_text_that_vanished_around_them` (edit/origin.rs:555)

#### `tag_control`  (L529-547)
`fn tag_control(dom: &mut Dom, sdt: NodeId, tag: usize)`

- **Does:** Write `tag` into the control's `w:sdtPr/w:id`, creating them if needed.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L530,538; `Dom::new_element` (xmllinq/mod.rs:478) L533,541; `Dom::add_first` (xmllinq/mod.rs:1085) L534,542; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L546
- **Calls (std/external/unresolved):** `.to_string`, `Some`, `W::name`, `W::sdt_pr`, `W::val`
- **Called by:** `Origin::build` (edit/origin.rs:256)

## Test functions (2)

- `tests::untagged_paragraphs_are_merges_of_the_text_that_vanished_around_them` (L555): _no doc comment_
- `tests::untagged_paragraphs_are_merges_of_the_text_that_vanished_around_them::{fn texts}` (L557): _no doc comment_
