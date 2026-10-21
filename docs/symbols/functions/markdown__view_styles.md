# Functions and methods in `src/markdown/view_styles.rs`

27 non-test, 20 test. Part of the [symbol manifest](../README.md).

#### `with_style_changes`  (L45-95, pub(super))
`pub(super) fn with_style_changes(styles: &str, changes: &[(String, Vec<FormatMark>)]) -> String`

- **Does:** `styles` with a tracked change on each holder the view's `style_changes:` names.
- **Calls (crate):** `date_attribute` (markdown/xml.rs:2921) L49; `escape` (markdown/xml.rs:601) L56,63; `with_defaults_change` (markdown/view_styles.rs:101) L60; `change` (ambiguous: `tests::change` (changes/logical.rs:593); `change` (markdown/diff.rs:909); `change` (markdown/view_stories.rs:1227)) L60,76,78,86,88
- **Calls (std/external/unresolved):** `.and_then`, `.enumerate`, `.find`, `.insert_str`, `.iter`, `.len`, `.map_or_else`, `.map`, `.push_str`, `.replace_range`, `.rfind`, `.to_string`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `assemble` (markdown/package.rs:103); `tests::defaults_change` (markdown/view_styles.rs:1104)

#### `with_defaults_change`  (L101-151)
`fn with_defaults_change(out: &mut String, carrier: &str)`

- **Does:** `carrier` (a `w:rPrChange`) last in the run defaults' `w:rPr`, making whatever of `w:docDefaults` / `w:rPrDefault` / `w:rPr` the part lacks where the schema puts it: `w:docDefaults` first in `w:styles`, `w:rPrDefault` first in it.
- **Calls (crate):** `open_tag` (markdown/view_styles.rs:164) L104,105,126,140
- **Calls (std/external/unresolved):** `.filter`, `.find`, `.insert_str`, `.len`, `.map`, `.replace_range`, `.rfind`
- **Macros:** `format!`
- **Called by:** `with_style_changes` (markdown/view_styles.rs:45)

#### `open_tag`  (L164-180)
`fn open_tag(xml: &str, name: &str, range: std::ops::Range<usize>) -> Option<OpenTag>`

- **Does:** The first `<w:name …>` start tag in `range` of `xml`, attributes and all; `<w:nameMore` is another element.
- **Calls (std/external/unresolved):** `.ends_with`, `.find`, `.len`, `.map`, `.starts_with`, `Some`
- **Macros:** `format!`
- **Called by:** `with_defaults_change` (markdown/view_styles.rs:101); `with_default_spacing` (markdown/view_styles.rs:645)

#### `points`  (L222-226)
`fn points(value: &str, scale: f64) -> Option<u32>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`, `.is_finite`, `.ok`, `.parse`, `.round`, `.strip_suffix`, `.then`
- **Called by:** `font_clause` (markdown/view_styles.rs:240); `spec` (markdown/view_styles.rs:258); `Spacing::clause` (markdown/xml.rs:293) (?)

#### `twips_in`  (L229-237, pub(super))
`pub(super) fn twips_in(value: &str) -> Option<i64>`

- **Does:** `0.5in` as twips.
- **Calls (std/external/unresolved):** `.abs`, `.is_finite`, `.ok`, `.parse`, `.round`, `.strip_suffix`, `.then_some`
- **Called by:** `spec` (markdown/view_styles.rs:258); `TableGeometry::clause` (markdown/xml.rs:247)

#### `font_clause`  (L240-255)
`fn font_clause(clause: &str) -> Option<(String, bool, bool, u32)>`

- **Does:** `Times New Roman bold italic 11pt`: the font, bold, italic and size.
- **Calls (crate):** `points` (markdown/view_styles.rs:222) L242
- **Calls (std/external/unresolved):** `.is_empty`, `.rsplit_once`, `.strip_suffix`, `.then`, `.to_string`
- **Called by:** `spec` (markdown/view_styles.rs:258)

#### `spec`  (L258-330)
`fn spec(id: &str, heading: Option<u32>, default: bool, value: &str) -> Option<StyleSpec>`

- **Does:** The style a legend value describes; `heading` is the `#` count.
- **Calls (crate):** `font_clause` (markdown/view_styles.rs:240) L267; `points` (markdown/view_styles.rs:222) L292,293,302,303; `twips_in` (markdown/view_styles.rs:229) L294,296,297,298
- **Calls (std/external/unresolved):** `.contains`, `.is_finite`, `.is_some`, `.next_if`, `.next`, `.ok`, `.parse`, `.peekable`, `.round`, `.rsplit_once`, `.split_once`, `.split`, `.to_string`, `Some`
- **Called by:** `legend` (markdown/view_styles.rs:545)

#### `without_comment`  (L335-350, pub(super))
`pub(super) fn without_comment(line: &str) -> &str`

- **Does:** `line` without the comment the view prints after a value: a `#` after two spaces, outside a quoted string (`text: "ACME #1"` keeps its `#`; so does `title: Issue #5`).
- **Calls (std/external/unresolved):** `.char_indices`
- **Called by:** `first_section` (markdown/view_stories.rs:839); `legend_line` (markdown/view_styles.rs:468); `legend` (markdown/view_styles.rs:545); `tests::a_comment_ends_at_a_hash_outside_quotes` (markdown/view_styles.rs:1282); `scholar` (markdown/write.rs:502); `Writer::metadata` (markdown/write.rs:4453)

#### `unquote`  (L353-359)
`fn unquote(value: &str) -> Option<String>`

- **Does:** A YAML scalar as the legend prints it: plain, or a JSON string.
- **Calls (std/external/unresolved):** `.ok`, `.starts_with`, `.to_string`, `Some`, `serde_json::from_str`
- **Called by:** `legend_line` (markdown/view_styles.rs:468); `table_style_entry` (markdown/view_styles.rs:493); `legend` (markdown/view_styles.rs:545)

#### `border_sides`  (L396-437)
`fn border_sides(clause: &str) -> Option<Vec<BorderSide>>`

- **Does:** A `borders …` clause's sides (`borders bottom double 0.75pt`): a side name, its value, then a size in points, a `#RRGGBB` colour and a `space Npt`, each optional.
- **Calls (crate):** `Parser::peek` (markdown/math.rs:804) (by name) L413
- **Calls (std/external/unresolved):** `.all`, `.any`, `.chars`, `.contains`, `.filter`, `.is_ascii_alphanumeric`, `.is_ascii_hexdigit`, `.is_empty`, `.is_none`, `.iter`, `.len`, `.next`, `.ok`, `.parse`, `.peekable`, `.push`, `.round`, `.split`, `.strip_prefix`, `.strip_suffix`, `.then_some`, `.to_string`, `Some`, `Vec::new`
- **Called by:** `table_style_entry` (markdown/view_styles.rs:493)

#### `table_style`  (L441-446, pub(super))
`pub(super) fn table_style(yaml: &str) -> Option<(String, Option<TableBorders>, Option<String>)>`

- **Does:** The legend's `table:` style, which every table takes: its id, the borders its entry states and the name it gives the style.
- **Calls (crate):** `legend_line` (markdown/view_styles.rs:468) L442
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.map`
- **Called by:** `Writer::metadata` (markdown/write.rs:4453)

#### `table_legend`  (L451-464, pub(super))
`pub(super) fn table_legend( yaml: &str, ) -> std::collections::BTreeMap<String, (Option<TableBorders>, Option<String>)>`

- **Does:** The `tables:` legend's entries, keyed by their style id: the styles of a document whose tables differ, each with the border and the name its entry states.
- **Calls (crate):** `legend_line` (markdown/view_styles.rs:468) L455; `table_style_entry` (markdown/view_styles.rs:493) L459
- **Calls (std/external/unresolved):** `.as_str`, `.insert`, `std::collections::BTreeMap::new`
- **Called by:** `Writer::metadata` (markdown/write.rs:4453)

#### `legend_line`  (L468-485)
`fn legend_line(yaml: &str, key: &str) -> Option<String>`

- **Does:** The `styles:` block's `key: ` line's value, unwrapped from its YAML quoting: `table: X, named My X`'s, `tables: …`'s.
- **Calls (crate):** `without_comment` (markdown/view_styles.rs:335) L475; `unquote` (markdown/view_styles.rs:353) L479
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.lines`, `.starts_with`, `.strip_prefix`, `.trim_end`, `.trim`, `Some`
- **Called by:** `table_style` (markdown/view_styles.rs:441); `table_legend` (markdown/view_styles.rs:451)

#### `table_style_entry`  (L493-524)
`fn table_style_entry(value: &str) -> Option<(TableStyleEntry, &str)>`

- **Does:** _no doc comment_
- **Calls (crate):** `take_clause` (markdown/view_styles.rs:529) L494,499; `unquote` (markdown/view_styles.rs:353) L495,515; `border_sides` (markdown/view_styles.rs:396) L509
- **Calls (std/external/unresolved):** `.and_then`, `.contains`, `.filter`, `.is_empty`, `.is_none`, `.ok`, `.parse`, `.round`, `.strip_prefix`, `.strip_suffix`, `.trim`, `Some`, `TableBorders::All`, `TableBorders::Sides`
- **Called by:** `table_legend` (markdown/view_styles.rs:451)

#### `take_clause`  (L529-543)
`fn take_clause(value: &str) -> (String, &str)`

- **Does:** The value's first `, `-separated clause, a quoted one whole (a `"` opens a JSON string, whose `, ` is the name's own), and what follows the separator.
- **Calls (std/external/unresolved):** `.char_indices`, `.starts_with`, `.to_string`
- **Called by:** `table_style_entry` (markdown/view_styles.rs:493)

#### `legend`  (L545-581, pub(super))
`pub(super) fn legend(yaml: &str) -> Vec<StyleSpec>`

- **Does:** _no doc comment_
- **Calls (crate):** `without_comment` (markdown/view_styles.rs:335) L557; `unquote` (markdown/view_styles.rs:353) L561,570; `spec` (markdown/view_styles.rs:258) L570,576
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.as_str`, `.bytes`, `.contains`, `.extend`, `.filter`, `.iter`, `.len`, `.lines`, `.ok`, `.split_once`, `.starts_with`, `.trim_end`, `.trim`, `Some`, `Vec::new`, `u32::try_from`
- **Macros:** `matches!`
- **Called by:** `tests::a_style_s_universal_measures_read_as_their_integers` (markdown/from_docx/header.rs:2506) (?); `tests::every_other_style_and_its_name_read_and_are_defined` (markdown/view_styles.rs:1167); `tests::a_character_style_comes_back_as_one` (markdown/view_styles.rs:1206); `tests::a_character_style_keeps_its_font_without_a_paragraph_base` (markdown/view_styles.rs:1221); `tests::the_legend_reads` (markdown/view_styles.rs:1247); `tests::what_does_not_read_is_left_out` (markdown/view_styles.rs:1293); `tests::a_default_style_with_another_id_takes_the_default` (markdown/view_styles.rs:1315); `tests::a_definition_replaces_the_styles_own` (markdown/view_styles.rs:1334); `tests::an_indent_contextual_spacing_and_keep_lines_are_written` (markdown/view_styles.rs:1383); `tests::a_right_indent_is_written_and_a_bare_right_still_aligns` (markdown/view_styles.rs:1415); `tests::a_theme_font_in_the_defaults_is_not_the_named_font` (markdown/view_styles.rs:1444); `tests::a_style_omits_properties_the_defaults_already_give` (markdown/view_styles.rs:1456); +5 more

#### `apply`  (L586-623, pub(super))
`pub(super) fn apply(styles: &str, specs: &[StyleSpec]) -> String`

- **Does:** `styles` (a styles part) with each style of `specs` defined as the legend describes it, in place of its own definition.
- **Calls (crate):** `document_defaults` (markdown/view_styles.rs:832) L587; `with_default_spacing` (markdown/view_styles.rs:645) L600; `without_style` (markdown/view_styles.rs:727) L607; `definition` (markdown/view_styles.rs:939) L617
- **Calls (std/external/unresolved):** `.as_ref`, `.as_str`, `.clone_from`, `.filter`, `.find`, `.insert_str`, `.iter`, `.map`, `.replace`, `.rfind`, `.to_string`, `.unwrap_or`
- **Called by:** `run_resolution` (bin/jubarte.rs:458) (?); `reresolve_theme_color_hexes` (document_comparer.rs:6013) (?); `assemble` (markdown/package.rs:103); `tests::every_other_style_and_its_name_read_and_are_defined` (markdown/view_styles.rs:1167); `tests::a_character_style_comes_back_as_one` (markdown/view_styles.rs:1206); `tests::a_character_style_keeps_its_font_without_a_paragraph_base` (markdown/view_styles.rs:1221); `tests::a_default_style_with_another_id_takes_the_default` (markdown/view_styles.rs:1315); `tests::a_definition_replaces_the_styles_own` (markdown/view_styles.rs:1334); `tests::an_indent_contextual_spacing_and_keep_lines_are_written` (markdown/view_styles.rs:1383); `tests::a_right_indent_is_written_and_a_bare_right_still_aligns` (markdown/view_styles.rs:1415); `tests::a_theme_font_in_the_defaults_is_not_the_named_font` (markdown/view_styles.rs:1444); `tests::a_style_omits_properties_the_defaults_already_give` (markdown/view_styles.rs:1456); +4 more

#### `with_default_spacing`  (L645-724)
`fn with_default_spacing(out: &mut String, look: &Given)`

- **Does:** `out` with `look`'s spacing as the document's paragraph default (`w:docDefaults/w:pPrDefault/w:pPr/w:spacing`) in place of the spacing it had, each element made where the part lacks it: `w:docDefaults` first in `w:styles`, `w:pPrDefault` after `w:rPrDefault`, the spacing where CT_PPrBase puts it.
- **Calls (crate):** `open_tag` (markdown/view_styles.rs:164) L657,658,679,693,707,720
- **Calls (std/external/unresolved):** `.filter_map`, `.filter`, `.find`, `.insert_str`, `.iter`, `.len`, `.map_or`, `.map`, `.min`, `.replace_range`, `.rfind`, `.unwrap_or`, `String::from`
- **Macros:** `format!`, `write!`
- **Called by:** `apply` (markdown/view_styles.rs:586)

#### `without_style`  (L727-746)
`fn without_style(styles: &str, id: &str) -> String`

- **Does:** `styles` without the definition of style `id`.
- **Calls (crate):** `escape` (markdown/xml.rs:601) L728
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.len`, `.map`, `.push_str`, `String::with_capacity`
- **Macros:** `format!`
- **Called by:** `apply` (markdown/view_styles.rs:586)

#### `Given::implicit`  (L770-789)
`fn implicit() -> Self`

- **Does:** Word's own values when `docDefaults` names nothing.
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `document_defaults` (markdown/view_styles.rs:832)

#### `effective`  (L794-814)
`fn effective(spec: &StyleSpec) -> Given`

- **Does:** The legend's value for the default style: an omitted spacing is zero or single, an omitted alignment is left, and an omitted indent is none.
- **Calls (std/external/unresolved):** `.clone`, `.to_string`, `.unwrap_or`, `Some`
- **Called by:** none found in src

#### `whole`  (L816-821)
`fn whole(element: &Element, attr: &str) -> Option<u32>`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:327) L817
- **Calls (std/external/unresolved):** `.contains`, `.is_finite`, `.ok`, `.parse`, `.round`, `.then_some`
- **Called by:** `detect_unrelated_sources_word_mode` (comparer/lcs.rs:6062) (?); `document_defaults` (markdown/view_styles.rs:832)

#### `signed`  (L823-828)
`fn signed(element: &Element, attr: &str) -> Option<i64>`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:327) L824
- **Calls (std/external/unresolved):** `.abs`, `.is_finite`, `.ok`, `.parse`, `.round`, `.then_some`
- **Called by:** `document_defaults` (markdown/view_styles.rs:832)

#### `document_defaults`  (L832-902)
`fn document_defaults(styles: &str) -> Given`

- **Does:** `docDefaults` over Word's implicit values.
- **Calls (crate):** `Given::implicit` (markdown/view_styles.rs:770) L833; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L837,843,851,862,874,897; `Element::path` (markdown/from_docx/ooxml.rs:391) (by name) L840,861; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L844,847,871,897; `whole` (markdown/view_styles.rs:816) L851,863,866,869; `Element::toggle` (markdown/from_docx/ooxml.rs:431) (by name) L854,857,888,891,894; `signed` (markdown/view_styles.rs:823) L875,878,881,884
- **Calls (std/external/unresolved):** `.and_then`, `.as_bytes`, `.flatten`, `.is_none`, `.map`, `.or_else`, `.then`, `.to_string`, `.unwrap_or`, `Some`, `super::from_docx::parse_xml`
- **Called by:** `apply` (markdown/view_styles.rs:586)

#### `indent_attr`  (L905-907)
`fn indent_attr(spec: Option<i64>, inherited: Option<i64>) -> Option<i64>`

- **Does:** An indent the legend sets and the style does not already inherit.
- **Calls (std/external/unresolved):** `.filter`, `Some`
- **Called by:** `definition` (markdown/view_styles.rs:939)

#### `gallery_flags`  (L912-934)
`fn gallery_flags(id: &str, name: &str) -> String`

- **Does:** The gallery flags (`uiPriority`, `semiHidden`, `unhideWhenUsed`, `qFormat`) Word gives built-in style `id` named `name`, in schema order; none for a style Word does not know.
- **Calls (crate):** `style_definition` (markdown/xml.rs:2711) L913
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.len`, `.map_or`, `.push_str`, `String::new`
- **Macros:** `format!`
- **Called by:** `definition` (markdown/view_styles.rs:939)

#### `definition`  (L939-1097)
`fn definition(spec: &StyleSpec, base: Option<&str>, inherited: &Given) -> String`

- **Does:** The `w:style` element of `spec`, based on style `base` unless it is the default.
- **Calls (crate):** `escape` (markdown/xml.rs:601) L940,942,962,964,1068; `gallery_flags` (markdown/view_styles.rs:912) L970; `indent_attr` (markdown/view_styles.rs:905) L1023,1025; `toggle` (ambiguous: `toggle` (edit.rs:8557); `toggle` (revision_processor/style_records.rs:176)) L1081,1084
- **Calls (std/external/unresolved):** `.as_deref`, `.as_str`, `.filter`, `.is_empty`, `.is_ok`, `.is_some_and`, `.is_some`, `.or`, `.parse`, `.push_str`, `.replace`, `.strip_prefix`, `.to_string`, `.unwrap_or`, `Some`, `String::new`, `indent`
- **Macros:** `format!`
- **Called by:** `deeper_boundary_tests::equal_comment_bodies_keep_distinct_dates_and_initials` (comparer/comments.rs:2585) (?); `deeper_boundary_tests::carry_comments_remaps_colliding_anchor_ids_and_preserves_definition_metadata` (comparer/comments.rs:2623) (?); `apply` (markdown/view_styles.rs:586)

## Test functions (20)

- `tests::defaults_change` (L1104): _no doc comment_
- `tests::a_defaults_change_lands_in_the_defaults_wherever_they_are` (L1116): _no doc comment_
- `tests::every_other_style_and_its_name_read_and_are_defined` (L1167): _no doc comment_
- `tests::a_character_style_comes_back_as_one` (L1206): _no doc comment_
- `tests::a_character_style_keeps_its_font_without_a_paragraph_base` (L1221): _no doc comment_
- `tests::the_legend_reads` (L1247): _no doc comment_
- `tests::a_comment_ends_at_a_hash_outside_quotes` (L1282): _no doc comment_
- `tests::what_does_not_read_is_left_out` (L1293): _no doc comment_
- `tests::a_default_style_with_another_id_takes_the_default` (L1315): _no doc comment_
- `tests::a_definition_replaces_the_styles_own` (L1334): _no doc comment_
- `tests::style_element` (L1369): _no doc comment_
- `tests::an_indent_contextual_spacing_and_keep_lines_are_written` (L1383): _no doc comment_
- `tests::a_right_indent_is_written_and_a_bare_right_still_aligns` (L1415): _no doc comment_
- `tests::a_theme_font_in_the_defaults_is_not_the_named_font` (L1444): _no doc comment_
- `tests::a_style_omits_properties_the_defaults_already_give` (L1456): _no doc comment_
- `tests::the_default_style_s_omitted_indent_is_none` (L1518): _no doc comment_
- `tests::a_run_property_is_written_when_it_differs_from_the_base` (L1545): _no doc comment_
- `tests::paragraph_defaults` (L1560): The `w:pPrDefault` of a styles part, or nothing.
- `tests::the_default_style_s_spacing_is_the_documents_default` (L1571): _no doc comment_
- `tests::the_default_spacing_lands_where_the_schema_puts_it` (L1612): _no doc comment_
