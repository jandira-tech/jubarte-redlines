# Functions and methods in `src/edit/controls.rs`

15 non-test, 7 test. Part of the [symbol manifest](../README.md).

#### `Transaction::resolve_fill_control`  (L73-204, pub(super))
`pub(super) fn resolve_fill_control( &self, id: &str, request: &FillRequest<'_>, ) -> Resolution<Vec<Resolved>>`

- **Does:** _no doc comment_
- **Calls (crate):** `err` (edit.rs:1770) L93; `Transaction::select_control` (edit/controls.rs:207) L95; `Dom::element` (xmllinq/mod.rs:655) (by name) L131; `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L132; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L134; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L137; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L149,160; `Transaction::fill_value` (edit/controls.rs:318) L167; `excerpt` (edit.rs:7161) L178,179; `paragraph_id` (ids.rs:410) L191
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.clone`, `.cloned`, `.collect`, `.contains`, `.enumerate`, `.filter`, `.first`, `.into_iter`, `.into`, `.is_empty`, `.is_some_and`, `.iter`, `.map`, `.to_string`, `Box::new`, `Err`, `Ok`, `Some`, `String::new`, `W::name`, `W::p`, `W::sdt_content`, `W::sect_pr`, `W::tbl`, `fail`
- **Macros:** `format!`, `vec!`
- **Called by:** `Transaction::resolve` (edit.rs:3267)

#### `Transaction::select_control`  (L207-299)
`fn select_control( &self, selector: &ControlSelector, ) -> Result<usize, (&'static str, String, usize)>`

- **Does:** The control index, or `(code, message, matches)`.
- **Calls (crate):** `Origin::control_ids` (edit/origin.rs:368) (by name) L216; `Origin::control_source` (edit/origin.rs:373) (by name) L221; `source_only` (edit/origin.rs:637) L226; `Transaction::cell_control` (edit/controls.rs:303) L231; `.control` (ambiguous: `Origin::control` (edit/origin.rs:381); `Writer::control` (markdown/math.rs:1685)) L234
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.as_ref`, `.as_slice`, `.as_str`, `.collect`, `.filter`, `.get`, `.iter`, `.join`, `.len`, `.map`, `.ok`, `.parse`, `.position`, `.strip_prefix`, `.to_string`, `.unwrap_or_default`, `.unwrap_or`, `Err`, `Ok`, `Some`, `name_of`
- **Macros:** `format!`
- **Called by:** `Transaction::resolve_fill_control` (edit/controls.rs:73)

#### `Transaction::cell_control`  (L303-315)
`fn cell_control(&self, id: &str) -> Option<String>`

- **Does:** `t0.row1.cell2[.pK].ccN` as the `pM.ccN` of the paragraph its cell spelling names; `None` for any other id.
- **Calls (crate):** `.ids` (ambiguous, 6 candidates) L310; `Transaction::ids` (edit.rs:4931) L311; `.paragraph_id` (ambiguous: `Story::paragraph_id` (audit.rs:324); `Ids::paragraph_id` (edit/origin.rs:48); `Transaction::paragraph_id` (edit.rs:5085)) L313
- **Calls (std/external/unresolved):** `.ok`, `.parse`, `.rsplit_once`, `.starts_with`, `Some`
- **Macros:** `format!`
- **Called by:** `Transaction::select_control` (edit/controls.rs:207)

#### `Transaction::fill_value`  (L318-422)
`fn fill_value( &self, record: &crate::inspect::ContentControl, sdt: NodeId, request: &FillRequest<'_>, ) -> Result<FillValue, String>`

- **Does:** Check the value form against the control and compute what to write.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) (by name) L353,366,367,388,390,414...; `check_text` (edit.rs:7094) L358; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L369; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L371,372,396,402,416; `parse_date` (edit/controls.rs:666) L411; `format_date` (edit/controls.rs:726) L420
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.count`, `.filter`, `.is_control`, `.is_some`, `.iter`, `.join`, `.map_or_else`, `.ok_or_else`, `.ok`, `.or_else`, `.or`, `.to_string`, `.unwrap_or_default`, `.unwrap_or`, `Err`, `FillValue::Text`, `Ok`, `Some`, `W14::name`, `W::name`, `W::sdt_pr`, `W::val`, `form_for`, `u32::from_str_radix`, `wrong`
- **Macros:** `format!`, `matches!`
- **Called by:** `Transaction::resolve_fill_control` (edit/controls.rs:73)

#### `Transaction::check_control_conflicts`  (L425-484, pub(super))
`pub(super) fn check_control_conflicts(&self, deleted: &[usize]) -> Result<(), EditError>`

- **Does:** Fills against each other and against paragraph operations.
- **Calls (crate):** `Transaction::conflict` (edit.rs:5870) L443,448,473,480; `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L446
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.collect`, `.contains`, `.enumerate`, `.filter_map`, `.iter`, `.max`, `Err`, `Ok`, `Some`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `Transaction::check_conflicts_with_comments` (edit.rs:5410)

#### `Transaction::apply_fills`  (L488-505, pub(super))
`pub(super) fn apply_fills(&mut self)`

- **Does:** Apply step 1b: write every fill.
- **Calls (crate):** `fill` (edit/controls.rs:509) L503
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.filter_map`, `.iter`, `Some`
- **Called by:** `Transaction::apply` (edit.rs:6430)

#### `fill`  (L509-560)
`fn fill(dom: &mut Dom, sdt: NodeId, block: bool, value: &FillValue)`

- **Does:** Write `value` into the control `sdt`.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L510,518,521,533,539,540; `Dom::new_element` (xmllinq/mod.rs:478) L513,524,543,554; `Dom::add_first` (xmllinq/mod.rs:1101) L514,544; `Dom::remove` (xmllinq/mod.rs:1118) L519; `Dom::add` (xmllinq/mod.rs:1074) L525; `first_run_properties` (edit/controls.rs:629) L529; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L534,548; `set_glyph_font` (edit/controls.rs:650) L555; `write_content` (edit/controls.rs:589) L556,559
- **Calls (std/external/unresolved):** `.clone`, `.to_string`, `.unwrap_or_else`, `Some`, `W14::name`, `W::name`, `W::r_pr`, `W::sdt_content`, `W::sdt_pr`
- **Called by:** `demo_title_pilcrow_history_tests::nested_mark_effect_changes_keep_both_source_projections_and_disabled_tracking` (comparer/finalize.rs:28667) (?); `finalizer_merge_source_guard_tests::pilcrow_format_equality_includes_nested_fill_payload_but_ignores_source_scratch` (comparer/finalize.rs:31786) (?); `Transaction::apply_fills` (edit/controls.rs:488); `control_source_boundary_tests::filling_empty_authored_controls_creates_only_missing_metadata_and_payload` (edit/controls.rs:831); `with_operation_targets` (edit/v2_apply.rs:97) (?)

#### `take_range_markers`  (L566-584)
`fn take_range_markers(dom: &mut Dom, content: NodeId) -> (Vec<NodeId>, Vec<NodeId>)`

- **Does:** The range markers under `content`, detached: `(starts, ends)` in document order.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L568; `Dom::name` (xmllinq/mod.rs:526) L569; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L570; `XName::local_name` (xmllinq/mod.rs:208) (by name) L573,575; `Dom::remove` (xmllinq/mod.rs:1118) L581
- **Calls (std/external/unresolved):** `.chain`, `.contains`, `.iter`, `.push`, `Vec::new`
- **Called by:** `write_content` (edit/controls.rs:589)

#### `write_content`  (L589-625)
`fn write_content(dom: &mut Dom, content: NodeId, block: bool, rpr: Option<NodeId>, text: &str)`

- **Does:** Replace `content`'s children with one run (or one paragraph holding it, keeping the first paragraph's properties, for a block-level control).
- **Calls (crate):** `take_range_markers` (edit/controls.rs:566) L590; `Dom::new_element` (xmllinq/mod.rs:478) L591,597,608; `Dom::elements` (xmllinq/mod.rs:640) L593,610; `Dom::add` (xmllinq/mod.rs:1074) L595,600,615,617,623; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L598; `XNamespace::xml` (xmllinq/mod.rs:128) L598; `Dom::add_text` (xmllinq/mod.rs:1087) L599; `Dom::descendants` (xmllinq/mod.rs:668) L601; `Dom::element` (xmllinq/mod.rs:655) L604; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L605; `Dom::remove_nodes` (xmllinq/mod.rs:1110) L606; `Dom::name_is` (xmllinq/mod.rs:535) L611; `Dom::remove` (xmllinq/mod.rs:1118) L612
- **Calls (std/external/unresolved):** `.and_then`, `.chain`, `.first`, `.into_iter`, `.is_empty`, `.map`, `.name`, `Some`, `W::p_pr_change`, `W::p_pr`, `W::p`, `W::r`, `W::sect_pr`, `W::t`
- **Called by:** `fill` (edit/controls.rs:509)

#### `first_run_properties`  (L629-647)
`fn first_run_properties(dom: &mut Dom, content: NodeId) -> Option<NodeId>`

- **Does:** A copy of the first run's properties in `content`, without the placeholder style and revision marks.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L630; `Dom::element` (xmllinq/mod.rs:655) L633; `Dom::clone_subtree` (xmllinq/mod.rs:1231) L634; `Dom::elements` (xmllinq/mod.rs:640) L635; `Dom::name_is` (xmllinq/mod.rs:535) L636,639,640,641; `Dom::attribute` (xmllinq/mod.rs:874) L637; `Dom::remove` (xmllinq/mod.rs:1118) L643
- **Calls (std/external/unresolved):** `.find_map`, `.into_iter`, `Some`, `W::del`, `W::ins`, `W::name`, `W::r_pr_change`, `W::r_pr`, `W::r`, `W::val`
- **Called by:** `fill` (edit/controls.rs:509)

#### `set_glyph_font`  (L650-663)
`fn set_glyph_font(dom: &mut Dom, rpr: NodeId, font: &str)`

- **Does:** Point every script's font at `font`, as Word does for a checkbox glyph.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L651; `Dom::new_element` (xmllinq/mod.rs:478) L654; `insert_rpr_child` (edit.rs:8544) L655; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L660,662
- **Calls (std/external/unresolved):** `Some`, `W::name`
- **Called by:** `fill` (edit/controls.rs:509)

#### `parse_date`  (L666-680)
`fn parse_date(date: &str) -> Option<(i32, u32, u32)>`

- **Does:** `YYYY-MM-DD` with a real calendar day.
- **Calls (crate):** `days_in_month` (edit/controls.rs:682) L679
- **Calls (std/external/unresolved):** `.all`, `.as_bytes`, `.bytes`, `.contains`, `.is_ascii_digit`, `.len`, `.ok`, `.parse`, `.then_some`, `.then`, `digits`, `i32::try_from`
- **Called by:** `Transaction::fill_value` (edit/controls.rs:318); `tests::dates_parse_only_real_days` (edit/controls.rs:773)

#### `days_in_month`  (L682-689)
`fn days_in_month(y: i32, m: u32) -> u32`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `parse_date` (edit/controls.rs:666)

#### `weekday`  (L692-697)
`fn weekday(y: i32, m: u32, d: u32) -> usize`

- **Does:** Day of the week, 0 = Sunday (Sakamoto's method).
- **Calls (std/external/unresolved):** `.rem_euclid`
- **Called by:** `format_date` (edit/controls.rs:726)

#### `format_date`  (L726-765)
`fn format_date(pattern: &str, y: i32, m: u32, d: u32) -> String`

- **Does:** Format a date with a Word date pattern (`yyyy`, `yy`, `MMMM`, `MMM`, `MM`, `M`, `dddd`, `ddd`, `dd`, `d`; `'quoted'` text is literal; other characters are copied).
- **Calls (crate):** `weekday` (edit/controls.rs:692) L757,758
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.count`, `.extend`, `.iter`, `.len`, `.map_or`, `.position`, `.push_str`, `.rem_euclid`, `.take_while`, `.to_string`, `String::new`, `std::iter::repeat_n`
- **Macros:** `format!`
- **Called by:** `Transaction::fill_value` (edit/controls.rs:318); `tests::word_date_patterns_format_in_english` (edit/controls.rs:792)

## Test functions (7)

- `tests::dates_parse_only_real_days` (L773): _no doc comment_
- `tests::word_date_patterns_format_in_english` (L792): _no doc comment_
- `control_source_boundary_tests::parse` (L809): _no doc comment_
- `control_source_boundary_tests::package` (L815): _no doc comment_
- `control_source_boundary_tests::plan` (L827): _no doc comment_
- `control_source_boundary_tests::filling_empty_authored_controls_creates_only_missing_metadata_and_payload` (L831): _no doc comment_
- `control_source_boundary_tests::public_block_controls_refuse_section_loss_and_both_nested_fill_orders` (L885): _no doc comment_
