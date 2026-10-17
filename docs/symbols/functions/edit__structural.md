# Functions and methods in `src/edit/structural.rs`

15 non-test, 8 test. Part of the [symbol manifest](../README.md).

#### `check_rows`  (L28-66, pub(super))
`pub(super) fn check_rows(rows: &[Vec<String>], widths: Option<&[u32]>) -> Result<usize, String>`

- **Does:** The column count of `rows`, checked: at least one row, every row the same nonzero length, plain cell text, and `widths` (when given) one positive width per column.
- **Calls (crate):** `check_text` (edit.rs:5368) L49
- **Calls (std/external/unresolved):** `.any`, `.enumerate`, `.first`, `.into`, `.iter`, `.len`, `.map_err`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `tests::rows_are_checked_for_shape_and_widths` (edit/structural.rs:495); `Transaction::resolve_one` (edit.rs:2528)

#### `text_width`  (L70-86, pub(super))
`pub(super) fn text_width(dom: &Dom, body: NodeId) -> u32`

- **Does:** The body's text width: the final section's page width less its side margins (Letter with 1-inch margins when unset).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L71; `Dom::name_is` (xmllinq/mod.rs:535) L75; `Dom::element` (xmllinq/mod.rs:655) L78; `Dom::attribute` (xmllinq/mod.rs:874) L79; `read` (ambiguous: `read` (legacy_doc.rs:180); `read` (markdown/mod.rs:749); `read` (markdown/write.rs:125)) L83,84
- **Calls (std/external/unresolved):** `.and_then`, `.find`, `.into_iter`, `.max`, `.ok`, `.parse`, `.rev`, `.saturating_sub`, `.unwrap_or`, `W::name`, `W::sect_pr`
- **Called by:** `tests::text_width_reads_the_final_section` (edit/structural.rs:548); `Transaction::resolve_one` (edit.rs:2528)

#### `table_styles`  (L89-114, pub(super))
`pub(super) fn table_styles(opened: &Opened) -> Vec<(String, String)>`

- **Does:** `(styleId, name)` of every table style in the document's styles part.
- **Calls (crate):** `Opened::related` (inspect.rs:669) L90; `PartFs::part_string` (opc/mod.rs:498) L93; `Dom::new` (xmllinq/mod.rs:387) L96; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L97; `Dom::root` (xmllinq/mod.rs:822) L98; `Dom::elements` (xmllinq/mod.rs:640) L101; `Dom::attribute` (xmllinq/mod.rs:874) L103,105,108; `Dom::element` (xmllinq/mod.rs:655) L106
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.filter_map`, `.filter`, `.into_iter`, `.next`, `.to_string`, `.unwrap_or`, `Some`, `Vec::new`, `W::name`, `W::val`
- **Called by:** `Transaction::resolve_one` (edit.rs:2528)

#### `resolve_table_style`  (L119-147, pub(super))
`pub(super) fn resolve_table_style( styles: &[(String, String)], requested: &str, ) -> Result<(String, bool), String>`

- **Does:** A table style id from its id or name (case-insensitive) among the document's table styles, or a built-in one the engine adds; the flag is true when the definition must be added.
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.chain`, `.clone`, `.collect`, `.eq_ignore_ascii_case`, `.find`, `.iter`, `.join`, `.map`, `.take`, `.to_string`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `tests::table_styles_resolve_by_id_name_or_builtin` (edit/structural.rs:529); `Transaction::resolve_one` (edit.rs:2528)

#### `build_table`  (L153-217, pub(super))
`pub(super) fn build_table( dom: &mut Dom, anchor: NodeId, rows: &[Vec<String>], header_row: bool, widths: &[u32], style: &str, ) -> NodeId`

- **Does:** A `w:tbl` of `rows`: `style`, a DXA width that is the sum of `widths`, a grid column and a DXA `w:tcW` per column, `w:tblHeader` on the first row when `header_row`, and one paragraph per cell that takes the anchor's paragraph style and run formatting.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L162,169; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L164; `Dom::add` (xmllinq/mod.rs:1058) L166,213; `edit_core::{fn element}` (markdown/package.rs:1347) L170,171,173,179,193,195...; `cell_paragraph` (edit/structural.rs:220) L212
- **Calls (std/external/unresolved):** `.enumerate`, `.iter`, `.sum`, `.to_string`, `.zip`, `Some`, `W::name`, `W::tbl`
- **Called by:** `emit` (edit/tracked.rs:275); `Transaction::apply` (edit.rs:4489)

#### `cell_paragraph`  (L220-237)
`fn cell_paragraph(dom: &mut Dom, anchor: NodeId, text: &str) -> NodeId`

- **Does:** A cell paragraph modeled on `anchor`, keeping only its paragraph style.
- **Calls (crate):** `build_paragraph` (edit.rs:6280) L225; `Dom::element` (xmllinq/mod.rs:655) L226; `Dom::elements` (xmllinq/mod.rs:640) L227,232; `Dom::name_is` (xmllinq/mod.rs:535) L228; `Dom::remove` (xmllinq/mod.rs:1102) L229,233
- **Calls (std/external/unresolved):** `.is_empty`, `.to_string`, `RunSpec::default`, `W::p_pr`, `W::p_style`
- **Called by:** `build_table` (edit/structural.rs:153)

#### `separate`  (L242-261, pub(super))
`pub(super) fn separate(dom: &mut Dom, table: NodeId)`

- **Does:** Give `table` a paragraph after it unless one follows, and one before it when a table precedes it: a story must not end in a table, and Word joins two tables that touch.
- **Calls (crate):** `Dom::next_element` (xmllinq/mod.rs:855) L243,245; `is_range_markup` (edit.rs:5280) L244,256; `Dom::name_is` (xmllinq/mod.rs:535) L247,257; `Dom::new_element` (xmllinq/mod.rs:478) L248,258; `Dom::add_after_self` (xmllinq/mod.rs:1131) L249; `Dom::nodes_before_self` (xmllinq/mod.rs:846) L251; `Dom::is_element` (xmllinq/mod.rs:502) L255; `Dom::add_before_self` (xmllinq/mod.rs:1114) L259
- **Calls (std/external/unresolved):** `.filter`, `.find`, `.into_iter`, `.is_some_and`, `.rev`, `W::p`, `W::tbl`
- **Called by:** `separate_tracked` (edit/tracked.rs:615); `Transaction::apply` (edit.rs:4489)

#### `add_styles`  (L267-275, pub(super))
`pub(super) fn add_styles(pkg: &mut PartFs, main: &str, wanted: &BTreeSet<String>)`

- **Does:** Add the definitions of `wanted` styles the styles part lacks (making the part when there is none).
- **Calls (crate):** `styles_part` (markdown/package.rs:308) L268; `PartFs::part_string` (opc/mod.rs:498) L269; `with_definitions` (edit/structural.rs:279) L271; `PartFs::set_part` (opc/mod.rs:510) L273
- **Calls (std/external/unresolved):** `.and_then`, `.into_bytes`
- **Called by:** `Transaction::finish` (edit.rs:5077)

#### `with_definitions`  (L279-340)
`fn with_definitions(source: &str, wanted: &BTreeSet<String>) -> Option<String>`

- **Does:** `source` (a styles part) with the definitions [`add_styles`] adds, or `None` when it adds none.
- **Calls (crate):** `Dom::new` (xmllinq/mod.rs:387) L280; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L281; `Dom::root` (xmllinq/mod.rs:822) L282; `Dom::elements` (xmllinq/mod.rs:640) L286,291; `Dom::attribute` (xmllinq/mod.rs:874) L289,302,303,316,320; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L296; `definition` (edit/structural.rs:470) L312,326; `Dom::element` (xmllinq/mod.rs:655) L315; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L323; `Dom::add` (xmllinq/mod.rs:1058) L327,335; `Dom::remove` (xmllinq/mod.rs:1102) L330; `Dom::serialize_document` (xmllinq/mod.rs:1322) L339
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.collect`, `.contains`, `.filter_map`, `.filter`, `.find`, `.insert`, `.into_iter`, `.iter`, `.map`, `.then`, `.to_string`, `.unwrap_or`, `Some`, `W::name`, `W::val`
- **Macros:** `matches!`
- **Called by:** `add_styles` (edit/structural.rs:267); `tests::a_missing_base_follows_the_documents_default_style` (edit/structural.rs:563); `tests::present_styles_are_left_alone` (edit/structural.rs:576); `tests::a_missing_base_without_a_default_is_added` (edit/structural.rs:587)

#### `numbering_tree`  (L343-346)
`fn numbering_tree(dom: &mut Dom, xml: &str) -> Option<(NodeId, NodeId)>`

- **Does:** The numbering part's parsed tree: its document and root element.
- **Calls (crate):** `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L344; `Dom::root` (xmllinq/mod.rs:822) L345
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `next_numbering_ids` (edit/structural.rs:350); `splice_numbering` (edit/structural.rs:377)

#### `next_numbering_ids`  (L350-371, pub(super))
`pub(super) fn next_numbering_ids(opened: &Opened) -> (u32, u32)`

- **Does:** One past the largest `w:abstractNumId` and `w:numId` in the document's numbering part (`(0, 1)` without one: `numId` 0 means "no list").
- **Calls (crate):** `Opened::related` (inspect.rs:669) L351; `PartFs::part_string` (opc/mod.rs:498) L355; `Dom::new` (xmllinq/mod.rs:387) L356; `numbering_tree` (edit/structural.rs:343) L357; `Dom::elements` (xmllinq/mod.rs:640) L361; `Dom::attribute` (xmllinq/mod.rs:874) L363
- **Calls (std/external/unresolved):** `.and_then`, `.filter_map`, `.into_iter`, `.map_or`, `.max`, `.next`, `.ok`, `.parse`, `.saturating_add`, `Some`, `W::name`, `next`
- **Called by:** `emit` (edit/tracked.rs:275); `Transaction::apply` (edit.rs:4489)

#### `splice_numbering`  (L377-419, pub(super))
`pub(super) fn splice_numbering( pkg: &mut PartFs, main: &str, abstracts: &[String], nums: &[String], )`

- **Does:** Add `abstracts` (`w:abstractNum`) and `nums` (`w:num`) to the numbering part, making it when there is none, in schema order: every `w:abstractNum` before every `w:num`, and both before `w:numIdMacAtCleanup`.
- **Calls (crate):** `numbering_part` (markdown/package.rs:922) L383; `PartFs::part_string` (opc/mod.rs:498) L384; `Dom::new` (xmllinq/mod.rs:387) L387; `numbering_tree` (edit/structural.rs:343) L388,397; `Dom::elements` (xmllinq/mod.rs:640) L401,406; `Dom::name` (xmllinq/mod.rs:526) L402; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L403; `XName::local_name` (xmllinq/mod.rs:208) (by name) L403; `Dom::remove` (xmllinq/mod.rs:1102) L407; `Dom::name_is` (xmllinq/mod.rs:535) L408; `Dom::add_before_self` (xmllinq/mod.rs:1114) L414; `Dom::add` (xmllinq/mod.rs:1058) L415; `PartFs::set_part` (opc/mod.rs:510) L418; `Dom::serialize_document` (xmllinq/mod.rs:1322) L418
- **Calls (std/external/unresolved):** `.concat`, `.contains`, `.find`, `.into_bytes`, `.into_iter`, `.is_some_and`, `W::name`, `first`
- **Macros:** `format!`
- **Called by:** `Transaction::finish` (edit.rs:5077)

#### `direct_num_id`  (L422-430, pub(super))
`pub(super) fn direct_num_id(dom: &Dom, paragraph: NodeId) -> Option<String>`

- **Does:** The `w:numId` of `paragraph`'s direct numbering, unless it is 0 (none).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L423,425,426; `Dom::attribute` (xmllinq/mod.rs:874) L428
- **Calls (std/external/unresolved):** `.and_then`, `.then`, `.to_string`, `W::name`, `W::num_pr`, `W::p_pr`, `W::val`
- **Called by:** `numbering_source_property_boundary_tests::new_numbering_adds_a_style_only_when_the_authored_paragraph_has_none` (edit/structural.rs:608); `Transaction::resolve_list` (edit.rs:3563)

#### `number_paragraph`  (L435-467, pub(super))
`pub(super) fn number_paragraph( dom: &mut Dom, paragraph: NodeId, level: u32, num_id: &str, list_style: &str, ) -> bool`

- **Does:** Number `paragraph` at `level` of list `num_id`, replacing any direct numbering; a paragraph without a style takes `list_style`.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L442,450,460; `Dom::new_element` (xmllinq/mod.rs:478) L445,453,455,463; `Dom::add_first` (xmllinq/mod.rs:1085) L446; `Dom::remove` (xmllinq/mod.rs:1102) L451; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L456,464; `Dom::add` (xmllinq/mod.rs:1058) L457; `insert_ppr_child` (edit.rs:6163) L459,465
- **Calls (std/external/unresolved):** `.is_some`, `.to_string`, `Some`, `W::name`, `W::num_pr`, `W::p_pr`, `W::p_style`, `W::val`
- **Called by:** `numbering_source_property_boundary_tests::new_numbering_adds_a_style_only_when_the_authored_paragraph_has_none` (edit/structural.rs:608); `emit` (edit/tracked.rs:275); `Transaction::apply` (edit.rs:4489)

#### `definition`  (L470-480)
`fn definition(dom: &mut Dom, id: &str) -> Option<NodeId>`

- **Does:** The engine's definition of style `id`, parsed into `dom` and detached.
- **Calls (crate):** `style_definition` (markdown/xml.rs:2267) L471; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L472; `Dom::root` (xmllinq/mod.rs:822) L476; `Dom::element` (xmllinq/mod.rs:655) L477; `Dom::remove` (xmllinq/mod.rs:1102) L478
- **Calls (std/external/unresolved):** `Some`, `W::name`
- **Macros:** `format!`
- **Called by:** `deeper_boundary_tests::equal_comment_bodies_keep_distinct_dates_and_initials` (comparer/comments.rs:2585) (?); `deeper_boundary_tests::carry_comments_remaps_colliding_anchor_ids_and_preserves_definition_metadata` (comparer/comments.rs:2623) (?); `with_definitions` (edit/structural.rs:279)

## Test functions (8)

- `tests::rows` (L487): _no doc comment_
- `tests::rows_are_checked_for_shape_and_widths` (L495): _no doc comment_
- `tests::table_styles_resolve_by_id_name_or_builtin` (L529): _no doc comment_
- `tests::text_width_reads_the_final_section` (L548): _no doc comment_
- `tests::a_missing_base_follows_the_documents_default_style` (L563): _no doc comment_
- `tests::present_styles_are_left_alone` (L576): _no doc comment_
- `tests::a_missing_base_without_a_default_is_added` (L587): _no doc comment_
- `numbering_source_property_boundary_tests::new_numbering_adds_a_style_only_when_the_authored_paragraph_has_none` (L608): _no doc comment_
