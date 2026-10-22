# Functions and methods in `src/markdown/borders.rs`

14 non-test, 4 test. Part of the [symbol manifest](../README.md).

#### `Border::new`  (L51-71, pub(crate))
`pub(crate) fn new( val: &str, size: Option<&str>, color: Option<&str>, space: Option<&str>, ) -> Self`

- **Does:** A border from its element's attributes.
- **Calls (std/external/unresolved):** `.and_then`, `.eq_ignore_ascii_case`, `.filter`, `.into`, `.map`, `.ok`, `.parse`
- **Macros:** `matches!`
- **Called by:** `table` (debug/render.rs:869); `tests::a_border_prints_and_reads_back` (markdown/borders.rs:291); `tests::all_six_sides_drop_their_names_and_groups_keep_their_order` (markdown/borders.rs:322); `tests::the_writer_writes_word_s_attributes` (markdown/borders.rs:367); `own_borders` (markdown/from_docx/agent.rs:2174)

#### `Border::with_theme`  (L74-87, pub(crate))
`pub(crate) fn with_theme( mut self, theme: Option<&str>, tint: Option<&str>, shade: Option<&str>, ) -> Self`

- **Does:** The border with its element's theme colour attributes.
- **Calls (crate):** `is_theme_name` (markdown/borders.rs:263) L83; `is_hex_byte` (markdown/borders.rs:268) L84,85
- **Calls (std/external/unresolved):** `.filter`, `.map`
- **Called by:** `own_borders` (markdown/from_docx/agent.rs:2174) (?)

#### `Border::text`  (L89-113)
`fn text(&self) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `points` (markdown/borders.rs:273) L93
- **Calls (std/external/unresolved):** `.clone`, `.push_str`, `.push`
- **Macros:** `format!`
- **Called by:** none found in src

#### `Border::parse`  (L115-155)
`fn parse(words: &[&str]) -> Option<Self>`

- **Does:** _no doc comment_
- **Calls (crate):** `is_theme_name` (markdown/borders.rs:263) L135; `is_hex_byte` (markdown/borders.rs:268) L138; `eighths` (markdown/borders.rs:278) L147
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.filter`, `.is_ascii_alphanumeric`, `.is_ascii_hexdigit`, `.iter`, `.len`, `.next`, `.ok`, `.parse`, `.split_first`, `.strip_suffix`, `.to_string`, `Some`
- **Called by:** `parse` (markdown/borders.rs:232)

#### `Border::xml`  (L158-181)
`fn xml(&self, side: &str) -> String`

- **Does:** The side's element as the writer writes it.
- **Calls (std/external/unresolved):** `.push_str`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `xml` (markdown/borders.rs:254) (?); `creator_created` (markdown/from_docx/mod.rs:1386) (?)

#### `side`  (L189-196, pub(crate))
`pub(crate) fn side(name: &str, sides: &[&'static str]) -> Option<&'static str>`

- **Does:** The side Word names `name`, as the view names it (`start` is `left`, `end` is `right`), when `sides` has it.
- **Calls (std/external/unresolved):** `.copied`, `.find`, `.iter`
- **Called by:** `check_whole` (changes/logical.rs:517) (?); `member_tokens` (comparer/cross_para.rs:597) (?); `resolve_paragraph_window` (comparer/lcs.rs:9735) (?); `inline_effect_pt` (convert/mod.rs:18193) (?); `table` (debug/render.rs:869); `plan_from_flags` (edit/flags.rs:85) (?); `Recorded::summary` (edit/record.rs:91) (?); `Geometry::of` (edit/sections.rs:151) (?); `tests::start_and_end_are_left_and_right_and_text_that_does_not_read_is_none` (markdown/borders.rs:351); `own_borders` (markdown/from_docx/agent.rs:2174); `text_width` (markdown/from_docx/header.rs:629) (?); `page_setup` (markdown/from_docx/header.rs:640) (?); +1 more

#### `ordered`  (L200-205, pub(crate))
`pub(crate) fn ordered(found: &[(&'static str, Border)], sides: &[&'static str]) -> Sides`

- **Does:** `found` (side, border) in schema order, a side named twice (`left` and `start`) kept once, the first time.
- **Calls (std/external/unresolved):** `.cloned`, `.collect`, `.filter_map`, `.find`, `.iter`
- **Called by:** `table` (debug/render.rs:869); `parse` (markdown/borders.rs:232); `own_borders` (markdown/from_docx/agent.rs:2174)

#### `describe`  (L209-228, pub(crate))
`pub(crate) fn describe(borders: &Sides, all: &[&'static str]) -> String`

- **Does:** The text a line prints for `borders`: groups of sides that share a border, in the order their first side comes.
- **Calls (crate):** `.text` (ambiguous, 10 candidates) L221,223
- **Calls (std/external/unresolved):** `.as_slice`, `.collect`, `.find`, `.iter_mut`, `.iter`, `.join`, `.map`, `.push`, `Vec::new`
- **Macros:** `format!`, `vec!`
- **Called by:** `table` (debug/render.rs:869); `tests::a_border_prints_and_reads_back` (markdown/borders.rs:291); `tests::all_six_sides_drop_their_names_and_groups_keep_their_order` (markdown/borders.rs:322); `border_clauses` (markdown/from_docx/agent.rs:2204); `convert_marks` (markdown/from_docx/mod.rs:108)

#### `parse`  (L232-250, pub(super))
`pub(super) fn parse(text: &str, all: &[&'static str]) -> Option<Sides>`

- **Does:** `text` (as [`describe`] prints it) read back; a group without sides covers `all`.
- **Calls (crate):** `Border::parse` (markdown/borders.rs:115) L242,243; `ordered` (markdown/borders.rs:200) L249
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.copied`, `.find`, `.first`, `.iter`, `.map`, `.push`, `.split_whitespace`, `.split`, `.to_vec`, `Some`, `Vec::new`
- **Called by:** `tests::a_border_prints_and_reads_back` (markdown/borders.rs:291); `tests::all_six_sides_drop_their_names_and_groups_keep_their_order` (markdown/borders.rs:322); `tests::start_and_end_are_left_and_right_and_text_that_does_not_read_is_none` (markdown/borders.rs:351); `table_line` (markdown/view_notes.rs:781); `ParaFormat::read` (markdown/view_stories.rs:94)

#### `xml`  (L254-259, pub(super))
`pub(super) fn xml(borders: &Sides) -> String`

- **Does:** The sides' elements, in schema order, for `w:tblBorders` or `w:tcBorders`.
- **Calls (crate):** `.xml` (ambiguous: `Border::xml` (markdown/borders.rs:158); `Package::xml` (markdown/from_docx/ooxml.rs:132); `XNamespace::xml` (xmllinq/mod.rs:128)) L257
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`
- **Called by:** `tests::the_writer_writes_word_s_attributes` (markdown/borders.rs:367); `story_part` (markdown/view_stories.rs:1710); `story` (markdown/xml.rs:786)

#### `is_theme_name`  (L263-265)
`fn is_theme_name(name: &str) -> bool`

- **Does:** A `w:themeColor` value as the view prints it: one word of letters and digits (`accent1`, `text1`, `followedHyperlink`).
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.is_ascii_alphanumeric`, `.is_empty`
- **Called by:** `Border::with_theme` (markdown/borders.rs:74); `Border::parse` (markdown/borders.rs:115)

#### `is_hex_byte`  (L268-270)
`fn is_hex_byte(value: &str) -> bool`

- **Does:** A `w:themeTint` or `w:themeShade` value: two hex digits.
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.is_ascii_hexdigit`, `.len`
- **Called by:** `Border::with_theme` (markdown/borders.rs:74); `Border::parse` (markdown/borders.rs:115)

#### `points`  (L273-276)
`fn points(eighths: u32) -> String`

- **Does:** Eighths of a point as points: `4` is `0.5`, `12` is `1.5`.
- **Calls (std/external/unresolved):** `.to_string`, `.trim_end_matches`, `from`
- **Macros:** `format!`
- **Called by:** `Border::text` (markdown/borders.rs:89); `Spacing::clause` (markdown/xml.rs:293) (?)

#### `eighths`  (L278-284)
`fn eighths(pt: f64) -> Option<u32>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`, `.round`, `.then_some`, `f64::from`
- **Called by:** `Border::parse` (markdown/borders.rs:115)

## Test functions (4)

- `tests::a_border_prints_and_reads_back` (L291): _no doc comment_
- `tests::all_six_sides_drop_their_names_and_groups_keep_their_order` (L322): _no doc comment_
- `tests::start_and_end_are_left_and_right_and_text_that_does_not_read_is_none` (L351): _no doc comment_
- `tests::the_writer_writes_word_s_attributes` (L367): _no doc comment_
