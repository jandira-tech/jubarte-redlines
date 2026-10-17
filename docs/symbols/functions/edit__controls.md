# Functions and methods in `src/edit/controls.rs`

15 non-test, 7 test. Part of the [symbol manifest](../README.md).

#### `Transaction::resolve_fill_control`  (L72-202, pub(super))
`pub(super) fn resolve_fill_control( &self, id: &str, request: &FillRequest<'_>, ) -> Resolution<Vec<Resolved>>`

- **Does:** _no doc comment_
- **Calls (crate):** `err` (edit.rs:1401) L91; `Transaction::select_control` (edit/controls.rs:205) L93; `Dom::element` (xmllinq/mod.rs:655) (by name) L129; `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L130; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L132; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L135; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L147,158; `Transaction::fill_value` (edit/controls.rs:313) L165; `excerpt` (edit.rs:5435) L176,177; `paragraph_id` (ids.rs:410) L189
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.clone`, `.cloned`, `.collect`, `.contains`, `.enumerate`, `.filter`, `.first`, `.into_iter`, `.into`, `.is_empty`, `.is_some_and`, `.iter`, `.map`, `.to_string`, `Box::new`, `Err`, `Ok`, `Some`, `String::new`, `W::name`, `W::p`, `W::sdt_content`, `W::sect_pr`, `W::tbl`, `fail`
- **Macros:** `format!`, `vec!`
- **Called by:** `Transaction::resolve` (edit.rs:2375)

#### `Transaction::select_control`  (L205-294)
`fn select_control( &self, selector: &ControlSelector, ) -> Result<usize, (&'static str, String, usize)>`

- **Does:** The control index, or `(code, message, matches)`.
- **Calls (crate):** `Origin::control_ids` (edit/origin.rs:377) (by name) L214; `Origin::control_source` (edit/origin.rs:382) (by name) L219; `Transaction::cell_control` (edit/controls.rs:298) L226; `.control` (ambiguous: `Origin::control` (edit/origin.rs:390); `Writer::control` (markdown/math.rs:1685)) L229
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.as_ref`, `.as_slice`, `.as_str`, `.collect`, `.filter`, `.get`, `.iter`, `.join`, `.len`, `.map`, `.ok`, `.parse`, `.position`, `.strip_prefix`, `.to_string`, `.unwrap_or_default`, `.unwrap_or`, `Err`, `Ok`, `Some`, `name_of`
- **Macros:** `format!`
- **Called by:** `Transaction::resolve_fill_control` (edit/controls.rs:72)

#### `Transaction::cell_control`  (L298-310)
`fn cell_control(&self, id: &str) -> Option<String>`

- **Does:** `t0.row1.cell2[.pK].ccN` as the `pM.ccN` of the paragraph its cell spelling names; `None` for any other id.
- **Calls (crate):** `.ids` (ambiguous: `ChangeFilter::ids` (changes.rs:117); `Origin::ids` (edit/origin.rs:397); `Transaction::ids` (edit.rs:3763)) L305; `Transaction::ids` (edit.rs:3763) L306; `.paragraph_id` (ambiguous: `Story::paragraph_id` (audit.rs:324); `Ids::paragraph_id` (edit/origin.rs:48); `Transaction::paragraph_id` (edit.rs:3876)) L308
- **Calls (std/external/unresolved):** `.ok`, `.parse`, `.rsplit_once`, `.starts_with`, `Some`
- **Macros:** `format!`
- **Called by:** `Transaction::select_control` (edit/controls.rs:205)

#### `Transaction::fill_value`  (L313-417)
`fn fill_value( &self, record: &crate::inspect::ContentControl, sdt: NodeId, request: &FillRequest<'_>, ) -> Result<FillValue, String>`

- **Does:** Check the value form against the control and compute what to write.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) (by name) L348,361,362,383,385,409...; `check_text` (edit.rs:5368) L353; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L364; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L366,367,391,397,411; `parse_date` (edit/controls.rs:663) L406; `format_date` (edit/controls.rs:723) L415
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.count`, `.filter`, `.is_control`, `.is_some`, `.iter`, `.join`, `.map_or_else`, `.ok_or_else`, `.ok`, `.or_else`, `.or`, `.to_string`, `.unwrap_or_default`, `.unwrap_or`, `Err`, `FillValue::Text`, `Ok`, `Some`, `W14::name`, `W::name`, `W::sdt_pr`, `W::val`, `form_for`, `u32::from_str_radix`, `wrong`
- **Macros:** `format!`, `matches!`
- **Called by:** `Transaction::resolve_fill_control` (edit/controls.rs:72)

#### `Transaction::check_control_conflicts`  (L420-479, pub(super))
`pub(super) fn check_control_conflicts(&self, deleted: &[usize]) -> Result<(), EditError>`

- **Does:** Fills against each other and against paragraph operations.
- **Calls (crate):** `Transaction::conflict` (edit.rs:4477) L438,443,468,475; `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L441
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.collect`, `.contains`, `.enumerate`, `.filter_map`, `.iter`, `.max`, `Err`, `Ok`, `Some`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `Transaction::check_conflicts` (edit.rs:4044)

#### `Transaction::apply_fills`  (L483-500, pub(super))
`pub(super) fn apply_fills(&mut self)`

- **Does:** Apply step 1b: write every fill.
- **Calls (crate):** `fill` (edit/controls.rs:504) L498
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.filter_map`, `.iter`, `Some`
- **Called by:** `Transaction::apply` (edit.rs:4489)

#### `fill`  (L504-555)
`fn fill(dom: &mut Dom, sdt: NodeId, block: bool, value: &FillValue)`

- **Does:** Write `value` into the control `sdt`.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L505,513,516,528,534,535; `Dom::new_element` (xmllinq/mod.rs:478) L508,519,538,549; `Dom::add_first` (xmllinq/mod.rs:1085) L509,539; `Dom::remove` (xmllinq/mod.rs:1102) L514; `Dom::add` (xmllinq/mod.rs:1058) L520; `first_run_properties` (edit/controls.rs:626) L524; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L529,543; `set_glyph_font` (edit/controls.rs:647) L550; `write_content` (edit/controls.rs:586) L551,554
- **Calls (std/external/unresolved):** `.clone`, `.to_string`, `.unwrap_or_else`, `Some`, `W14::name`, `W::name`, `W::r_pr`, `W::sdt_content`, `W::sdt_pr`
- **Called by:** `demo_title_pilcrow_history_tests::nested_mark_effect_changes_keep_both_source_projections_and_disabled_tracking` (comparer/finalize.rs:28648) (?); `finalizer_merge_source_guard_tests::pilcrow_format_equality_includes_nested_fill_payload_but_ignores_source_scratch` (comparer/finalize.rs:31772) (?); `Transaction::apply_fills` (edit/controls.rs:483); `control_source_boundary_tests::filling_empty_authored_controls_creates_only_missing_metadata_and_payload` (edit/controls.rs:828)

#### `take_range_markers`  (L564-581)
`fn take_range_markers(dom: &mut Dom, content: NodeId) -> (Vec<NodeId>, Vec<NodeId>)`

- **Does:** The range markers under `content`, detached: `(starts, ends)` in document order.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L566; `Dom::name` (xmllinq/mod.rs:526) L567; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L568; `XName::local_name` (xmllinq/mod.rs:208) (by name) L571,573; `Dom::remove` (xmllinq/mod.rs:1102) L578
- **Calls (std/external/unresolved):** `.chain`, `.contains`, `.iter`, `.push`, `Vec::new`
- **Called by:** `write_content` (edit/controls.rs:586)

#### `write_content`  (L586-622)
`fn write_content(dom: &mut Dom, content: NodeId, block: bool, rpr: Option<NodeId>, text: &str)`

- **Does:** Replace `content`'s children with one run (or one paragraph holding it, keeping the first paragraph's properties, for a block-level control).
- **Calls (crate):** `take_range_markers` (edit/controls.rs:564) L587; `Dom::new_element` (xmllinq/mod.rs:478) L588,594,605; `Dom::elements` (xmllinq/mod.rs:640) L590,607; `Dom::add` (xmllinq/mod.rs:1058) L592,597,612,614,620; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L595; `XNamespace::xml` (xmllinq/mod.rs:128) L595; `Dom::add_text` (xmllinq/mod.rs:1071) L596; `Dom::descendants` (xmllinq/mod.rs:668) L598; `Dom::element` (xmllinq/mod.rs:655) L601; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L602; `Dom::remove_nodes` (xmllinq/mod.rs:1094) L603; `Dom::name_is` (xmllinq/mod.rs:535) L608; `Dom::remove` (xmllinq/mod.rs:1102) L609
- **Calls (std/external/unresolved):** `.and_then`, `.chain`, `.first`, `.into_iter`, `.is_empty`, `.map`, `.name`, `Some`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r`, `W::sect_pr`, `W::t`
- **Called by:** `fill` (edit/controls.rs:504)

#### `first_run_properties`  (L626-644)
`fn first_run_properties(dom: &mut Dom, content: NodeId) -> Option<NodeId>`

- **Does:** A copy of the first run's properties in `content`, without the placeholder style and revision marks.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L627; `Dom::element` (xmllinq/mod.rs:655) L630; `Dom::clone_subtree` (xmllinq/mod.rs:1215) L631; `Dom::elements` (xmllinq/mod.rs:640) L632; `Dom::name_is` (xmllinq/mod.rs:535) L633,636,637,638; `Dom::attribute` (xmllinq/mod.rs:874) L634; `Dom::remove` (xmllinq/mod.rs:1102) L640
- **Calls (std/external/unresolved):** `.find_map`, `.into_iter`, `Some`, `W::del`, `W::ins`, `W::name`, `W::r_pr_change`, `W::r_pr`, `W::r`, `W::val`
- **Called by:** `fill` (edit/controls.rs:504)

#### `set_glyph_font`  (L647-660)
`fn set_glyph_font(dom: &mut Dom, rpr: NodeId, font: &str)`

- **Does:** Point every script's font at `font`, as Word does for a checkbox glyph.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L648; `Dom::new_element` (xmllinq/mod.rs:478) L651; `insert_rpr_child` (edit.rs:6525) L652; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L657,659
- **Calls (std/external/unresolved):** `Some`, `W::name`
- **Called by:** `fill` (edit/controls.rs:504)

#### `parse_date`  (L663-677)
`fn parse_date(date: &str) -> Option<(i32, u32, u32)>`

- **Does:** `YYYY-MM-DD` with a real calendar day.
- **Calls (crate):** `days_in_month` (edit/controls.rs:679) L676
- **Calls (std/external/unresolved):** `.all`, `.as_bytes`, `.bytes`, `.contains`, `.is_ascii_digit`, `.len`, `.ok`, `.parse`, `.then_some`, `.then`, `digits`, `i32::try_from`
- **Called by:** `Transaction::fill_value` (edit/controls.rs:313); `tests::dates_parse_only_real_days` (edit/controls.rs:770)

#### `days_in_month`  (L679-686)
`fn days_in_month(y: i32, m: u32) -> u32`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `parse_date` (edit/controls.rs:663)

#### `weekday`  (L689-694)
`fn weekday(y: i32, m: u32, d: u32) -> usize`

- **Does:** Day of the week, 0 = Sunday (Sakamoto's method).
- **Calls (std/external/unresolved):** `.rem_euclid`
- **Called by:** `format_date` (edit/controls.rs:723)

#### `format_date`  (L723-762)
`fn format_date(pattern: &str, y: i32, m: u32, d: u32) -> String`

- **Does:** Format a date with a Word date pattern (`yyyy`, `yy`, `MMMM`, `MMM`, `MM`, `M`, `dddd`, `ddd`, `dd`, `d`; `'quoted'` text is literal; other characters are copied).
- **Calls (crate):** `weekday` (edit/controls.rs:689) L754,755
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.count`, `.extend`, `.iter`, `.len`, `.map_or`, `.position`, `.push_str`, `.rem_euclid`, `.take_while`, `.to_string`, `String::new`, `std::iter::repeat_n`
- **Macros:** `format!`
- **Called by:** `Transaction::fill_value` (edit/controls.rs:313); `tests::word_date_patterns_format_in_english` (edit/controls.rs:789)

## Test functions (7)

- `tests::dates_parse_only_real_days` (L770): _no doc comment_
- `tests::word_date_patterns_format_in_english` (L789): _no doc comment_
- `control_source_boundary_tests::parse` (L806): _no doc comment_
- `control_source_boundary_tests::package` (L812): _no doc comment_
- `control_source_boundary_tests::plan` (L824): _no doc comment_
- `control_source_boundary_tests::filling_empty_authored_controls_creates_only_missing_metadata_and_payload` (L828): _no doc comment_
- `control_source_boundary_tests::public_block_controls_refuse_section_loss_and_both_nested_fill_orders` (L882): _no doc comment_
