# Functions and methods in `src/markdown/view_styles.rs`

25 non-test, 15 test. Part of the [symbol manifest](../README.md).

#### `with_style_changes`  (L40-89, pub(super))
`pub(super) fn with_style_changes(styles: &str, changes: &[(String, Vec<FormatMark>)]) -> String`

- **Does:** `styles` with a tracked change on each holder the view's `style_changes:` names: one `w:pPrChange` and `w:rPrChange` per tag a style's entry gives, a `w:rPrChange` on `docDefaults`, each carrying an empty previous properties element as the view names no payload.
- **Calls (crate):** `escape` (markdown/xml.rs:426) L47,55,62; `with_defaults_change` (markdown/view_styles.rs:95) L59; `change` (ambiguous: `change` (markdown/diff.rs:900); `change` (markdown/view_stories.rs:696)) L59,74,76,81,83
- **Calls (std/external/unresolved):** `.and_then`, `.find`, `.insert_str`, `.is_empty`, `.len`, `.map_or_else`, `.map`, `.push_str`, `.replace_range`, `.rfind`, `.to_string`, `.unwrap_or`, `String::new`
- **Macros:** `format!`
- **Called by:** `assemble` (markdown/package.rs:97); `tests::defaults_change` (markdown/view_styles.rs:885)

#### `with_defaults_change`  (L95-145)
`fn with_defaults_change(out: &mut String, carrier: &str)`

- **Does:** `carrier` (a `w:rPrChange`) last in the run defaults' `w:rPr`, making whatever of `w:docDefaults` / `w:rPrDefault` / `w:rPr` the part lacks where the schema puts it: `w:docDefaults` first in `w:styles`, `w:rPrDefault` first in it.
- **Calls (crate):** `open_tag` (markdown/view_styles.rs:158) L98,99,120,134
- **Calls (std/external/unresolved):** `.filter`, `.find`, `.insert_str`, `.len`, `.map`, `.replace_range`, `.rfind`
- **Macros:** `format!`
- **Called by:** `with_style_changes` (markdown/view_styles.rs:40)

#### `open_tag`  (L158-174)
`fn open_tag(xml: &str, name: &str, range: std::ops::Range<usize>) -> Option<OpenTag>`

- **Does:** The first `<w:name …>` start tag in `range` of `xml`, attributes and all; `<w:nameMore` is another element.
- **Calls (std/external/unresolved):** `.ends_with`, `.find`, `.len`, `.map`, `.starts_with`, `Some`
- **Macros:** `format!`
- **Called by:** `with_defaults_change` (markdown/view_styles.rs:95)

#### `points`  (L214-218)
`fn points(value: &str, scale: f64) -> Option<u32>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`, `.is_finite`, `.ok`, `.parse`, `.round`, `.strip_suffix`, `.then`
- **Called by:** `font_clause` (markdown/view_styles.rs:232); `spec` (markdown/view_styles.rs:250)

#### `twips_in`  (L221-229)
`fn twips_in(value: &str) -> Option<i64>`

- **Does:** `0.5in` as twips.
- **Calls (std/external/unresolved):** `.abs`, `.is_finite`, `.ok`, `.parse`, `.round`, `.strip_suffix`, `.then_some`
- **Called by:** `spec` (markdown/view_styles.rs:250)

#### `font_clause`  (L232-247)
`fn font_clause(clause: &str) -> Option<(String, bool, bool, u32)>`

- **Does:** `Times New Roman bold italic 11pt`: the font, bold, italic and size.
- **Calls (crate):** `points` (markdown/view_styles.rs:214) L234
- **Calls (std/external/unresolved):** `.is_empty`, `.rsplit_once`, `.strip_suffix`, `.then`, `.to_string`
- **Called by:** `spec` (markdown/view_styles.rs:250)

#### `spec`  (L250-319)
`fn spec(id: &str, heading: Option<u32>, default: bool, value: &str) -> Option<StyleSpec>`

- **Does:** The style a legend value describes; `heading` is the `#` count.
- **Calls (crate):** `font_clause` (markdown/view_styles.rs:232) L259; `points` (markdown/view_styles.rs:214) L283,284,291,292; `twips_in` (markdown/view_styles.rs:221) L285,286,287
- **Calls (std/external/unresolved):** `.contains`, `.is_finite`, `.is_some`, `.next_if`, `.next`, `.ok`, `.parse`, `.peekable`, `.round`, `.rsplit_once`, `.split_once`, `.split`, `.to_string`, `Some`
- **Called by:** `legend` (markdown/view_styles.rs:456)

#### `without_comment`  (L324-339, pub(super))
`pub(super) fn without_comment(line: &str) -> &str`

- **Does:** `line` without the comment the view prints after a value: a `#` after two spaces, outside a quoted string (`text: "ACME #1"` keeps its `#`; so does `title: Issue #5`).
- **Calls (std/external/unresolved):** `.char_indices`
- **Called by:** `first_section` (markdown/view_stories.rs:510); `legend_line` (markdown/view_styles.rs:383); `legend` (markdown/view_styles.rs:456); `tests::a_comment_ends_at_a_hash_outside_quotes` (markdown/view_styles.rs:1063); `scholar` (markdown/write.rs:319); `Writer::metadata` (markdown/write.rs:4075)

#### `unquote`  (L342-348)
`fn unquote(value: &str) -> Option<String>`

- **Does:** A YAML scalar as the legend prints it: plain, or a JSON string.
- **Calls (std/external/unresolved):** `.ok`, `.starts_with`, `.to_string`, `Some`, `serde_json::from_str`
- **Called by:** `legend_line` (markdown/view_styles.rs:383); `table_style_entry` (markdown/view_styles.rs:408); `legend` (markdown/view_styles.rs:456)

#### `table_style`  (L356-361, pub(super))
`pub(super) fn table_style(yaml: &str) -> Option<(String, Option<u32>, Option<String>)>`

- **Does:** The styles the YAML header's `styles:` block describes: the default style's line (`Normal: …`, its id quoted when it must be), each heading's (`"#": Heading1, …`) and each other style's (`TOC2: …`), after the default's.
- **Calls (crate):** `legend_line` (markdown/view_styles.rs:383) L357
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.map`
- **Called by:** `Writer::metadata` (markdown/write.rs:4075)

#### `table_legend`  (L366-379, pub(super))
`pub(super) fn table_legend( yaml: &str, ) -> std::collections::BTreeMap<String, (Option<u32>, Option<String>)>`

- **Does:** The `tables:` legend's entries, keyed by their style id: the styles of a document whose tables differ, each with the border and the name its entry states.
- **Calls (crate):** `legend_line` (markdown/view_styles.rs:383) L370; `table_style_entry` (markdown/view_styles.rs:408) L374
- **Calls (std/external/unresolved):** `.as_str`, `.insert`, `std::collections::BTreeMap::new`
- **Called by:** `Writer::metadata` (markdown/write.rs:4075)

#### `legend_line`  (L383-400)
`fn legend_line(yaml: &str, key: &str) -> Option<String>`

- **Does:** The `styles:` block's `key: ` line's value, unwrapped from its YAML quoting: `table: X, named My X`'s, `tables: …`'s.
- **Calls (crate):** `without_comment` (markdown/view_styles.rs:324) L390; `unquote` (markdown/view_styles.rs:342) L394
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.lines`, `.starts_with`, `.strip_prefix`, `.trim_end`, `.trim`, `Some`
- **Called by:** `table_style` (markdown/view_styles.rs:356); `table_legend` (markdown/view_styles.rs:366)

#### `table_style_entry`  (L408-435)
`fn table_style_entry(value: &str) -> Option<(TableStyleEntry, &str)>`

- **Does:** _no doc comment_
- **Calls (crate):** `take_clause` (markdown/view_styles.rs:440) L409,414; `unquote` (markdown/view_styles.rs:342) L410,426
- **Calls (std/external/unresolved):** `.and_then`, `.contains`, `.filter`, `.is_empty`, `.is_none`, `.ok`, `.parse`, `.round`, `.strip_prefix`, `.strip_suffix`, `.trim`, `Some`
- **Called by:** `table_legend` (markdown/view_styles.rs:366)

#### `take_clause`  (L440-454)
`fn take_clause(value: &str) -> (String, &str)`

- **Does:** The value's first `, `-separated clause, a quoted one whole (a `"` opens a JSON string, whose `, ` is the name's own), and what follows the separator.
- **Calls (std/external/unresolved):** `.char_indices`, `.starts_with`, `.to_string`
- **Called by:** `table_style_entry` (markdown/view_styles.rs:408)

#### `legend`  (L456-492, pub(super))
`pub(super) fn legend(yaml: &str) -> Vec<StyleSpec>`

- **Does:** _no doc comment_
- **Calls (crate):** `without_comment` (markdown/view_styles.rs:324) L468; `unquote` (markdown/view_styles.rs:342) L472,481; `spec` (markdown/view_styles.rs:250) L481,487
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.as_str`, `.bytes`, `.contains`, `.extend`, `.filter`, `.iter`, `.len`, `.lines`, `.ok`, `.split_once`, `.starts_with`, `.trim_end`, `.trim`, `Some`, `Vec::new`, `u32::try_from`
- **Macros:** `matches!`
- **Called by:** `tests::a_style_s_universal_measures_read_as_their_integers` (markdown/from_docx/header.rs:2299) (?); `tests::every_other_style_and_its_name_read_and_are_defined` (markdown/view_styles.rs:948); `tests::a_character_style_comes_back_as_one` (markdown/view_styles.rs:987); `tests::a_character_style_keeps_its_font_without_a_paragraph_base` (markdown/view_styles.rs:1002); `tests::the_legend_reads` (markdown/view_styles.rs:1028); `tests::what_does_not_read_is_left_out` (markdown/view_styles.rs:1074); `tests::a_default_style_with_another_id_takes_the_default` (markdown/view_styles.rs:1096); `tests::a_definition_replaces_the_styles_own` (markdown/view_styles.rs:1115); `tests::an_indent_contextual_spacing_and_keep_lines_are_written` (markdown/view_styles.rs:1159); `tests::a_theme_font_in_the_defaults_is_not_the_named_font` (markdown/view_styles.rs:1191); `tests::a_style_omits_properties_the_defaults_already_give` (markdown/view_styles.rs:1203); `tests::a_run_property_is_written_when_it_differs_from_the_base` (markdown/view_styles.rs:1265); +1 more

#### `apply`  (L497-525, pub(super))
`pub(super) fn apply(styles: &str, specs: &[StyleSpec]) -> String`

- **Does:** `styles` (a styles part) with each style of `specs` defined as the legend describes it, in place of its own definition.
- **Calls (crate):** `document_defaults` (markdown/view_styles.rs:630) L498; `without_style` (markdown/view_styles.rs:528) L509; `definition` (markdown/view_styles.rs:734) L519
- **Calls (std/external/unresolved):** `.as_ref`, `.as_str`, `.filter`, `.find`, `.insert_str`, `.is_some`, `.iter`, `.map`, `.replace`, `.rfind`, `.to_string`, `.unwrap_or`
- **Called by:** `run_resolution` (bin/jubarte.rs:361) (?); `reresolve_theme_color_hexes` (document_comparer.rs:6004) (?); `assemble` (markdown/package.rs:97); `tests::every_other_style_and_its_name_read_and_are_defined` (markdown/view_styles.rs:948); `tests::a_character_style_comes_back_as_one` (markdown/view_styles.rs:987); `tests::a_character_style_keeps_its_font_without_a_paragraph_base` (markdown/view_styles.rs:1002); `tests::a_default_style_with_another_id_takes_the_default` (markdown/view_styles.rs:1096); `tests::a_definition_replaces_the_styles_own` (markdown/view_styles.rs:1115); `tests::an_indent_contextual_spacing_and_keep_lines_are_written` (markdown/view_styles.rs:1159); `tests::a_theme_font_in_the_defaults_is_not_the_named_font` (markdown/view_styles.rs:1191); `tests::a_style_omits_properties_the_defaults_already_give` (markdown/view_styles.rs:1203); `tests::a_run_property_is_written_when_it_differs_from_the_base` (markdown/view_styles.rs:1265)

#### `without_style`  (L528-547)
`fn without_style(styles: &str, id: &str) -> String`

- **Does:** `styles` without the definition of style `id`.
- **Calls (crate):** `escape` (markdown/xml.rs:426) L529
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.len`, `.map`, `.push_str`, `String::with_capacity`
- **Macros:** `format!`
- **Called by:** `apply` (markdown/view_styles.rs:497)

#### `Given::implicit`  (L570-588)
`fn implicit() -> Self`

- **Does:** Word's own values when `docDefaults` names nothing.
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `document_defaults` (markdown/view_styles.rs:630)

#### `effective`  (L593-612)
`fn effective(spec: &StyleSpec) -> Given`

- **Does:** The legend's value for the default style: an omitted spacing is zero or single, an omitted alignment is left, and an omitted indent is none.
- **Calls (std/external/unresolved):** `.clone`, `.to_string`, `.unwrap_or`, `Some`
- **Called by:** none found in src

#### `whole`  (L614-619)
`fn whole(element: &Element, attr: &str) -> Option<u32>`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:286) L615
- **Calls (std/external/unresolved):** `.contains`, `.is_finite`, `.ok`, `.parse`, `.round`, `.then_some`
- **Called by:** `detect_unrelated_sources_word_mode` (comparer/lcs.rs:6062) (?); `document_defaults` (markdown/view_styles.rs:630)

#### `signed`  (L621-626)
`fn signed(element: &Element, attr: &str) -> Option<i64>`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:286) L622
- **Calls (std/external/unresolved):** `.abs`, `.is_finite`, `.ok`, `.parse`, `.round`, `.then_some`
- **Called by:** `document_defaults` (markdown/view_styles.rs:630)

#### `document_defaults`  (L630-697)
`fn document_defaults(styles: &str) -> Given`

- **Does:** `docDefaults` over Word's implicit values.
- **Calls (crate):** `Given::implicit` (markdown/view_styles.rs:570) L631; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:315); `Scope::child` (xmllinq/serialize.rs:225)) L635,641,649,660,672,692; `Element::path` (markdown/from_docx/ooxml.rs:320) (by name) L638,659; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L642,645,669,692; `whole` (markdown/view_styles.rs:614) L649,661,664,667; `Element::toggle` (markdown/from_docx/ooxml.rs:360) (by name) L652,655,683,686,689; `signed` (markdown/view_styles.rs:621) L673,676,679
- **Calls (std/external/unresolved):** `.and_then`, `.as_bytes`, `.flatten`, `.is_none`, `.map`, `.or_else`, `.then`, `.to_string`, `.unwrap_or`, `Some`, `super::from_docx::parse_xml`
- **Called by:** `apply` (markdown/view_styles.rs:497)

#### `indent_attr`  (L700-702)
`fn indent_attr(spec: Option<i64>, inherited: Option<i64>) -> Option<i64>`

- **Does:** An indent the legend sets and the style does not already inherit.
- **Calls (std/external/unresolved):** `.filter`, `Some`
- **Called by:** `definition` (markdown/view_styles.rs:734)

#### `gallery_flags`  (L707-729)
`fn gallery_flags(id: &str, name: &str) -> String`

- **Does:** The gallery flags (`uiPriority`, `semiHidden`, `unhideWhenUsed`, `qFormat`) Word gives built-in style `id` named `name`, in schema order; none for a style Word does not know.
- **Calls (crate):** `style_definition` (markdown/xml.rs:2267) L708
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.len`, `.map_or`, `.push_str`, `String::new`
- **Macros:** `format!`
- **Called by:** `definition` (markdown/view_styles.rs:734)

#### `definition`  (L734-878)
`fn definition(spec: &StyleSpec, base: Option<&str>, inherited: &Given) -> String`

- **Does:** The `w:style` element of `spec`, based on style `base` unless it is the default.
- **Calls (crate):** `escape` (markdown/xml.rs:426) L735,737,757,759,849; `gallery_flags` (markdown/view_styles.rs:707) L765; `indent_attr` (markdown/view_styles.rs:700) L813,814,815; `toggle` (ambiguous: `toggle` (edit.rs:6538); `toggle` (revision_processor/style_records.rs:176)) L862,865
- **Calls (std/external/unresolved):** `.as_deref`, `.as_str`, `.filter`, `.is_empty`, `.is_ok`, `.is_some_and`, `.is_some`, `.or`, `.parse`, `.push_str`, `.replace`, `.strip_prefix`, `.to_string`, `.unwrap_or`, `Some`, `String::new`
- **Macros:** `format!`
- **Called by:** `deeper_boundary_tests::equal_comment_bodies_keep_distinct_dates_and_initials` (comparer/comments.rs:2585) (?); `deeper_boundary_tests::carry_comments_remaps_colliding_anchor_ids_and_preserves_definition_metadata` (comparer/comments.rs:2623) (?); `apply` (markdown/view_styles.rs:497)

## Test functions (15)

- `tests::defaults_change` (L885): _no doc comment_
- `tests::a_defaults_change_lands_in_the_defaults_wherever_they_are` (L897): _no doc comment_
- `tests::every_other_style_and_its_name_read_and_are_defined` (L948): _no doc comment_
- `tests::a_character_style_comes_back_as_one` (L987): _no doc comment_
- `tests::a_character_style_keeps_its_font_without_a_paragraph_base` (L1002): _no doc comment_
- `tests::the_legend_reads` (L1028): _no doc comment_
- `tests::a_comment_ends_at_a_hash_outside_quotes` (L1063): _no doc comment_
- `tests::what_does_not_read_is_left_out` (L1074): _no doc comment_
- `tests::a_default_style_with_another_id_takes_the_default` (L1096): _no doc comment_
- `tests::a_definition_replaces_the_styles_own` (L1115): _no doc comment_
- `tests::style_element` (L1145): _no doc comment_
- `tests::an_indent_contextual_spacing_and_keep_lines_are_written` (L1159): _no doc comment_
- `tests::a_theme_font_in_the_defaults_is_not_the_named_font` (L1191): _no doc comment_
- `tests::a_style_omits_properties_the_defaults_already_give` (L1203): _no doc comment_
- `tests::a_run_property_is_written_when_it_differs_from_the_base` (L1265): _no doc comment_
