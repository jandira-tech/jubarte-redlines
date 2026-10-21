# Functions and methods in `src/convert/altchunk.rs`

25 non-test, 34 test. Part of the [symbol manifest](../README.md).

#### `expand`  (L15-44, pub(crate))
`pub(crate) fn expand(xml: &str, load: impl Fn(&str) -> Option<Vec<u8>>) -> String`

- **Does:** `xml` with every `w:altChunk` replaced by its chunk's paragraphs.
- **Calls (crate):** `attr` (convert/altchunk.rs:47) L34; `drawing_tests::load` (convert/mod.rs:38689) L35; `chunk_html` (convert/altchunk.rs:70) L36; `html_to_wml` (convert/altchunk.rs:604) L37
- **Calls (std/external/unresolved):** `.and_then`, `.ends_with`, `.find`, `.len`, `.map_or`, `.map`, `.push_str`, `.unwrap_or_default`, `String::with_capacity`
- **Called by:** `tests::an_alt_chunk_is_replaced_by_its_paragraphs` (convert/altchunk.rs:793); `tests::regression_tests::expansion_keeps_surrounding_content_and_chunk_order` (convert/altchunk.rs:824); `tests::regression_tests::expansion_without_chunks_does_not_load_any_parts` (convert/altchunk.rs:839); `tests::regression_tests::missing_id_does_not_load_an_unrelated_part` (convert/altchunk.rs:845); `imported_source_contract_tests::memory_story_chunk_relationships_expand_exactly_without_mutating_package_or_source` (convert/altchunk.rs:1093); `imported_source_contract_tests::valid_html_declarations_headings_and_self_closed_spans_keep_exact_source_formats` (convert/altchunk.rs:1265); `with_layout` (convert/mod.rs:769)

#### `attr`  (L47-67)
`fn attr(tag: &str, name: &str) -> Option<String>`

- **Does:** A tag's attribute value, any prefix (`r:id` for `id`).
- **Calls (std/external/unresolved):** `.as_bytes`, `.find`, `.get`, `.is_whitespace`, `.len`, `.map_or`, `.to_string`, `Some`
- **Macros:** `format!`, `matches!`
- **Called by:** `CommentFamily::record` (comments.rs:612) (?); `expand` (convert/altchunk.rs:15); `Builder::start` (convert/altchunk.rs:520); `imported_source_contract_tests::quoted_and_unquoted_html_attribute_boundaries_select_only_the_authored_class` (convert/altchunk.rs:1135); `page_frame_key` (convert/mod.rs:9233) (?); `body_frame_key` (convert/mod.rs:9255) (?); `tests::normal_takes_b_theme_fonts_from_its_doc_defaults` (document_comparer.rs:8382) (?); `tests::normal_takes_b_doc_default_indent_and_justification` (document_comparer.rs:8667) (?); `Transaction::opaque_structure` (edit.rs:5211) (?)

#### `chunk_html`  (L70-93)
`fn chunk_html(bytes: &[u8]) -> Option<String>`

- **Does:** The HTML of a chunk: an MHT's first `text/html` part, or the bytes.
- **Calls (crate):** `quoted_printable` (convert/altchunk.rs:121) L87; `base64_decode` (convert/altchunk.rs:97) L89
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.into_owned`, `.map_or`, `.map`, `.or_else`, `.rfind`, `.starts_with`, `.to_ascii_lowercase`, `.to_string`, `.unwrap_or`, `Some`, `String::from_utf8_lossy`
- **Called by:** `expand` (convert/altchunk.rs:15); `tests::an_mht_chunk_yields_its_quoted_printable_html` (convert/altchunk.rs:811); `tests::regression_tests::mime_without_an_html_part_or_header_separator_is_not_imported` (convert/altchunk.rs:855); `tests::regression_tests::mime_part_encoding_does_not_leak_from_a_previous_attachment` (convert/altchunk.rs:865); `tests::regression_tests::a_base64_html_part_is_decoded` (convert/altchunk.rs:898)

#### `base64_decode`  (L97-119)
`fn base64_decode(s: &str) -> Vec<u8>`

- **Does:** RFC 2045 base64: line breaks and other non-alphabet bytes are skipped, and decoding stops at the first `=` pad.
- **Calls (std/external/unresolved):** `.bytes`, `.len`, `.push`, `Vec::with_capacity`, `u32::from`
- **Called by:** `chunk_html` (convert/altchunk.rs:70)

#### `quoted_printable`  (L121-147)
`fn quoted_printable(s: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_bytes`, `.get`, `.into_owned`, `.len`, `.push`, `Some`, `String::from_utf8_lossy`, `Vec::with_capacity`, `u8::from_str_radix`
- **Macros:** `format!`
- **Called by:** `chunk_html` (convert/altchunk.rs:70); `tests::regression_tests::quoted_printable_handles_soft_breaks_utf8_and_incomplete_escapes` (convert/altchunk.rs:908)

#### `parse_css`  (L152-171)
`fn parse_css(css: &str) -> HashMap<String, Props>`

- **Does:** `selector { prop: value; … }` rules; a selector list shares its block.
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.entry`, `.extend`, `.filter_map`, `.is_empty`, `.map`, `.or_default`, `.split_once`, `.split`, `.to_ascii_lowercase`, `.to_string`, `.trim`, `HashMap::new`
- **Called by:** `html_to_wml` (convert/altchunk.rs:604)

#### `css_pt`  (L174-189)
`fn css_pt(v: &str) -> Option<f32>`

- **Does:** A CSS length in points ("14,4px" and "12pt" alike).
- **Calls (std/external/unresolved):** `.find`, `.is_ascii_alphabetic`, `.len`, `.ok`, `.parse`, `.replace`, `.split_at`, `.to_ascii_lowercase`, `.trim`, `.unwrap_or`, `Some`
- **Called by:** `Fmt::apply` (convert/altchunk.rs:202); `Builder::open_para` (convert/altchunk.rs:356); `tests::regression_tests::css_lengths_convert_physical_units_and_decimal_commas` (convert/altchunk.rs:934)

#### `Fmt::apply`  (L202-226)
`fn apply(&mut self, props: &Props)`

- **Does:** _no doc comment_
- **Calls (crate):** `css_pt` (convert/altchunk.rs:174) L212
- **Calls (std/external/unresolved):** `.and_then`, `.contains`, `.get`, `.is_empty`, `.is_ok_and`, `.next`, `.parse`, `.round`, `.split`, `.to_string`, `.trim_matches`, `.trim`, `.unwrap_or`, `Some`
- **Called by:** none found in src

#### `Fmt::rpr`  (L228-251)
`fn rpr(&self) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `xml_escape` (convert/altchunk.rs:765) L231
- **Calls (std/external/unresolved):** `.push_str`, `String::from`
- **Macros:** `write!`
- **Called by:** `Builder::flush` (convert/altchunk.rs:322) (?)

#### `Builder::fmt`  (L295-297)
`fn fmt(&self) -> Fmt`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone`, `.last`, `.map`, `.unwrap_or_default`
- **Called by:** `Builder::text` (convert/altchunk.rs:405); `Builder::start` (convert/altchunk.rs:520)

#### `Builder::props`  (L299-320)
`fn props(&self, tag: &str, class: Option<&str>, style: Option<&str>) -> Props`

- **Does:** _no doc comment_
- **Calls (crate):** `take` (markdown/diff.rs:1239) (by name) L306,310,311
- **Calls (std/external/unresolved):** `.clone`, `.extend`, `.filter_map`, `.get`, `.insert`, `.split_once`, `.split_whitespace`, `.split`, `.to_ascii_lowercase`, `.to_string`, `.trim`, `Props::new`
- **Macros:** `format!`
- **Called by:** `Builder::start` (convert/altchunk.rs:520)

#### `Builder::flush`  (L322-354)
`fn flush(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Fmt::rpr` (convert/altchunk.rs:228) (by name) L343,349; `xml_escape` (convert/altchunk.rs:765) L350
- **Calls (std/external/unresolved):** `.expect`, `.first_mut`, `.is_empty`, `.last_mut`, `.push_str`, `.take`, `.to_string`, `.trim_end`, `.trim_start`
- **Macros:** `write!`
- **Called by:** `Builder::open_para` (convert/altchunk.rs:356); `Builder::end_cell` (convert/altchunk.rs:433); `Builder::end_table` (convert/altchunk.rs:463); `Builder::start` (convert/altchunk.rs:520); `Builder::end` (convert/altchunk.rs:589); `html_to_wml` (convert/altchunk.rs:604)

#### `Builder::open_para`  (L356-403)
`fn open_para(&mut self, props: &Props)`

- **Does:** _no doc comment_
- **Calls (crate):** `Builder::flush` (convert/altchunk.rs:322) L357; `css_pt` (convert/altchunk.rs:174) L359,360,364,385
- **Calls (std/external/unresolved):** `.and_then`, `.ends_with`, `.filter`, `.get`, `.map_or`, `.map`, `.push_str`, `.round`, `.trim_end_matches`, `.trim_end`, `Some`, `String::new`, `Vec::new`
- **Macros:** `write!`
- **Called by:** `Builder::text` (convert/altchunk.rs:405); `Builder::start` (convert/altchunk.rs:520)

#### `Builder::text`  (L405-431)
`fn text(&mut self, raw: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `collapse_ws` (convert/altchunk.rs:704) L406; `decode_entities` (convert/altchunk.rs:724) L406; `Builder::open_para` (convert/altchunk.rs:356) L414; `Builder::fmt` (convert/altchunk.rs:295) L416
- **Calls (std/external/unresolved):** `.all`, `.as_mut`, `.ends_with`, `.expect`, `.is_empty`, `.is_none`, `.is_some_and`, `.iter`, `.last`, `.push`, `.starts_with`, `.to_string`, `.trim_start`, `.trim`, `Props::new`
- **Called by:** `html_to_wml` (convert/altchunk.rs:604)

#### `Builder::end_cell`  (L433-443)
`fn end_cell(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Builder::flush` (convert/altchunk.rs:322) L434
- **Calls (std/external/unresolved):** `.and_then`, `.last_mut`, `.len`, `.pop`, `.push`, `.unwrap_or_default`, `.unwrap_or`
- **Called by:** `Builder::close_open_cell` (convert/altchunk.rs:454); `Builder::end` (convert/altchunk.rs:589); `html_to_wml` (convert/altchunk.rs:604)

#### `Builder::in_open_cell`  (L446-450)
`fn in_open_cell(&self) -> bool`

- **Does:** Whether the innermost table has a cell open.
- **Calls (std/external/unresolved):** `.is_some_and`, `.last`, `.len`
- **Called by:** `Builder::close_open_cell` (convert/altchunk.rs:454); `Builder::end` (convert/altchunk.rs:589)

#### `Builder::close_open_cell`  (L454-461)
`fn close_open_cell(&mut self)`

- **Does:** Ends the innermost table's open cell, whose end tag HTML lets a document leave out: a new cell, a new row or the table's end does.
- **Calls (crate):** `Builder::in_open_cell` (convert/altchunk.rs:446) L455; `Builder::end_cell` (convert/altchunk.rs:433) L459
- **Calls (std/external/unresolved):** `.iter`, `.rposition`, `.truncate`
- **Called by:** `Builder::end_table` (convert/altchunk.rs:463); `Builder::start` (convert/altchunk.rs:520)

#### `Builder::end_table`  (L463-518)
`fn end_table(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Builder::close_open_cell` (convert/altchunk.rs:454) L464; `Builder::flush` (convert/altchunk.rs:322) L465
- **Calls (std/external/unresolved):** `.as_str`, `.contains`, `.expect`, `.iter`, `.last_mut`, `.map`, `.max`, `.pop`, `.push_str`, `.sum`, `.unwrap_or`, `String::from`, `String::new`
- **Macros:** `format!`, `write!`
- **Called by:** `Builder::end` (convert/altchunk.rs:589); `html_to_wml` (convert/altchunk.rs:604)

#### `Builder::start`  (L520-587)
`fn start(&mut self, tag: &str, open: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `attr` (convert/altchunk.rs:47) L521,522,545,563; `Builder::props` (convert/altchunk.rs:299) L523; `Builder::fmt` (convert/altchunk.rs:295) L526,538,568,576; `.apply` (ambiguous, 6 candidates) L530,572,583; `Builder::open_para` (convert/altchunk.rs:356) L532,536; `Builder::flush` (convert/altchunk.rs:322) L544,560; `Builder::close_open_cell` (convert/altchunk.rs:454) L553,559
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.as_mut`, `.contains`, `.into`, `.is_none`, `.is_some_and`, `.last_mut`, `.len`, `.max`, `.ok`, `.parse`, `.push`, `.starts_with`, `.to_string`, `.trim`, `.unwrap_or`, `Props::new`, `String::new`, `Vec::new`
- **Called by:** `html_to_wml` (convert/altchunk.rs:604)

#### `Builder::end`  (L589-600)
`fn end(&mut self, tag: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `Builder::flush` (convert/altchunk.rs:322) L594; `Builder::in_open_cell` (convert/altchunk.rs:446) L595; `Builder::end_cell` (convert/altchunk.rs:433) L595; `Builder::end_table` (convert/altchunk.rs:463) L597
- **Calls (std/external/unresolved):** `.contains`, `.iter`, `.rposition`, `.truncate`
- **Called by:** `html_to_wml` (convert/altchunk.rs:604)

#### `html_to_wml`  (L604-688, pub(crate))
`pub(crate) fn html_to_wml(html: &str) -> String`

- **Does:** The body WML of an HTML document.
- **Calls (crate):** `collect_between` (convert/altchunk.rs:690) L605; `parse_css` (convert/altchunk.rs:152) L606; `Builder::text` (convert/altchunk.rs:405) L628,632; `Builder::end` (convert/altchunk.rs:589) L672,676; `Builder::start` (convert/altchunk.rs:520) L674; `Builder::flush` (convert/altchunk.rs:322) L680; `Builder::end_cell` (convert/altchunk.rs:433) L682; `Builder::end_table` (convert/altchunk.rs:463) L685
- **Calls (std/external/unresolved):** `.and_then`, `.chars`, `.collect`, `.ends_with`, `.find`, `.into`, `.is_ascii_alphanumeric`, `.is_empty`, `.len`, `.map_or`, `.map`, `.pop`, `.starts_with`, `.take_while`, `.to_ascii_lowercase`, `.trim_start_matches`, `.trim`, `.unwrap_or_default`, `.unwrap_or`, `Some`, `String::new`, `Vec::new`
- **Macros:** `format!`, `vec!`
- **Called by:** `expand` (convert/altchunk.rs:15); `tests::a_class_rule_and_inline_bold_become_run_properties` (convert/altchunk.rs:778); `tests::regression_tests::omitted_cell_and_row_end_tags_keep_the_table` (convert/altchunk.rs:875); `tests::regression_tests::entities_preserve_unknown_and_invalid_unicode_references` (convert/altchunk.rs:919); `tests::regression_tests::nested_inline_formatting_is_restored_after_closing_tags` (convert/altchunk.rs:951); `tests::regression_tests::inline_css_overrides_class_and_tag_rules` (convert/altchunk.rs:967); `tests::regression_tests::paragraph_css_controls_spacing_alignment_and_indent` (convert/altchunk.rs:978); `tests::regression_tests::comments_scripts_and_styles_do_not_become_document_text` (convert/altchunk.rs:994); `tests::regression_tests::style_in_an_html_fragment_preserves_following_content` (convert/altchunk.rs:1010); `tests::regression_tests::html_tables_keep_spans_empty_cells_and_header_formatting` (convert/altchunk.rs:1026); `tests::regression_tests::invalid_or_zero_colspan_falls_back_to_one_column` (convert/altchunk.rs:1047); `imported_source_contract_tests::quoted_and_unquoted_html_attribute_boundaries_select_only_the_authored_class` (convert/altchunk.rs:1135); +9 more

#### `collect_between`  (L690-702)
`fn collect_between(s: &str, open: &str, close: &str) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.find`, `.is_empty`, `.len`, `.map_or`, `.push_str`, `.push`, `.then_some`, `.to_ascii_lowercase`, `String::new`
- **Called by:** `html_to_wml` (convert/altchunk.rs:604); `imported_source_contract_tests::empty_css_selectors_and_unclosed_style_blocks_do_not_steal_later_owned_rules` (convert/altchunk.rs:1304)

#### `collapse_ws`  (L704-722)
`fn collapse_ws(s: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chars`, `.is_whitespace`, `.len`, `.push`, `String::with_capacity`
- **Called by:** `Builder::text` (convert/altchunk.rs:405); `imported_source_contract_tests::paragraph_whitespace_empty_runs_and_leading_breaks_preserve_exact_imported_payloads` (convert/altchunk.rs:1226)

#### `decode_entities`  (L724-763, pub(crate))
`pub(crate) fn decode_entities(s: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.and_then`, `.contains`, `.filter`, `.find`, `.len`, `.map`, `.ok`, `.parse`, `.push_str`, `.push`, `.starts_with`, `.to_string`, `Some`, `String::with_capacity`, `u32::from_str_radix`
- **Called by:** `Builder::text` (convert/altchunk.rs:405); `tests::regression_tests::entities_preserve_unknown_and_invalid_unicode_references` (convert/altchunk.rs:919); `html_attribute` (markdown/write.rs:1346); `html_block_text` (markdown/write.rs:1424)

#### `xml_escape`  (L765-770)
`fn xml_escape(s: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.replace`
- **Called by:** `Fmt::rpr` (convert/altchunk.rs:228); `Builder::flush` (convert/altchunk.rs:322)

## Test functions (34)

- `tests::a_class_rule_and_inline_bold_become_run_properties` (L778): _no doc comment_
- `tests::an_alt_chunk_is_replaced_by_its_paragraphs` (L793): _no doc comment_
- `tests::an_mht_chunk_yields_its_quoted_printable_html` (L811): _no doc comment_
- `tests::regression_tests::expansion_keeps_surrounding_content_and_chunk_order` (L824): _no doc comment_
- `tests::regression_tests::expansion_without_chunks_does_not_load_any_parts` (L839): _no doc comment_
- `tests::regression_tests::missing_id_does_not_load_an_unrelated_part` (L845): _no doc comment_
- `tests::regression_tests::mime_without_an_html_part_or_header_separator_is_not_imported` (L855): _no doc comment_
- `tests::regression_tests::mime_part_encoding_does_not_leak_from_a_previous_attachment` (L865): _no doc comment_
- `tests::regression_tests::omitted_cell_and_row_end_tags_keep_the_table` (L875): _no doc comment_
- `tests::regression_tests::a_base64_html_part_is_decoded` (L898): _no doc comment_
- `tests::regression_tests::quoted_printable_handles_soft_breaks_utf8_and_incomplete_escapes` (L908): _no doc comment_
- `tests::regression_tests::entities_preserve_unknown_and_invalid_unicode_references` (L919): _no doc comment_
- `tests::regression_tests::css_lengths_convert_physical_units_and_decimal_commas` (L934): _no doc comment_
- `tests::regression_tests::nested_inline_formatting_is_restored_after_closing_tags` (L951): _no doc comment_
- `tests::regression_tests::inline_css_overrides_class_and_tag_rules` (L967): _no doc comment_
- `tests::regression_tests::paragraph_css_controls_spacing_alignment_and_indent` (L978): _no doc comment_
- `tests::regression_tests::comments_scripts_and_styles_do_not_become_document_text` (L994): _no doc comment_
- `tests::regression_tests::style_in_an_html_fragment_preserves_following_content` (L1010): _no doc comment_
- `tests::regression_tests::html_tables_keep_spans_empty_cells_and_header_formatting` (L1026): _no doc comment_
- `tests::regression_tests::invalid_or_zero_colspan_falls_back_to_one_column` (L1047): _no doc comment_
- `imported_source_contract_tests::plain` (L1070): _no doc comment_
- `imported_source_contract_tests::package` (L1076): _no doc comment_
- `imported_source_contract_tests::memory_story_chunk_relationships_expand_exactly_without_mutating_package_or_source` (L1093): _no doc comment_
- `imported_source_contract_tests::quoted_and_unquoted_html_attribute_boundaries_select_only_the_authored_class` (L1135): _no doc comment_
- `imported_source_contract_tests::css_font_weight_style_and_decoration_have_independent_literal_run_contracts` (L1159): _no doc comment_
- `imported_source_contract_tests::css_font_family_empty_fallback_and_numeric_lengths_preserve_owned_text` (L1196): _no doc comment_
- `imported_source_contract_tests::paragraph_whitespace_empty_runs_and_leading_breaks_preserve_exact_imported_payloads` (L1226): _no doc comment_
- `imported_source_contract_tests::valid_html_declarations_headings_and_self_closed_spans_keep_exact_source_formats` (L1265): _no doc comment_
- `imported_source_contract_tests::unclosed_html_table_cells_are_finalized_once_with_source_spans_and_empty_cells` (L1285): _no doc comment_
- `imported_source_contract_tests::empty_css_selectors_and_unclosed_style_blocks_do_not_steal_later_owned_rules` (L1304): _no doc comment_
- `residual_import_contract_tests::plain` (L1335): _no doc comment_
- `residual_import_contract_tests::paragraph_line_height_percentage_and_declined_numeric_forms_keep_exact_source_spacing` (L1342): _no doc comment_
- `residual_import_contract_tests::class_list_and_inline_override_preserve_complete_font_properties_and_owned_xml_text` (L1369): _no doc comment_
- `residual_import_contract_tests::ragged_authored_rows_share_one_grid_without_inventing_filler_cells_or_reordering_payload` (L1384): _no doc comment_
