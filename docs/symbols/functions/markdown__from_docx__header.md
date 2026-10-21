# Functions and methods in `src/markdown/from_docx/header.rs`

44 non-test, 21 test. Part of the [symbol manifest](../README.md).

#### `kv`  (L168-179)
`fn kv(out: &mut String, key_value: &str, comment: Option<&str>)`

- **Does:** `key: value` padded to 35 columns then `# comment`; a key/value longer than 33 characters takes two spaces.
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.filter`, `.is_empty`, `.push_str`, `.push`
- **Macros:** `format!`
- **Called by:** `render` (markdown/from_docx/header.rs:363); `story_kv` (markdown/from_docx/header.rs:1271); `first_section_stories` (markdown/from_docx/header.rs:1317); `part_two` (markdown/from_docx/header.rs:1480); `tests::a_long_value_aligns_by_characters_not_bytes` (markdown/from_docx/header.rs:2228); `tests::an_empty_comment_leaves_no_hash` (markdown/from_docx/header.rs:2250)

#### `scalar`  (L185-210, pub(crate))
`pub(crate) fn scalar(value: &str, flow: bool) -> String`

- **Does:** `value` as a YAML scalar: plain when every YAML reader, 1.1 or 1.2, takes it back as the same string, else double-quoted (a JSON string is a valid YAML one).
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.chars`, `.contains`, `.ends_with`, `.is_ascii_digit`, `.is_empty`, `.starts_with`, `.to_ascii_lowercase`, `.to_string`, `.trim`, `.unwrap_or_else`, `serde_json::to_string`
- **Macros:** `format!`
- **Called by:** `render` (markdown/from_docx/header.rs:363); `paragraph_entries` (markdown/from_docx/header.rs:1215); `story_tail` (markdown/from_docx/header.rs:1248); `story_kv` (markdown/from_docx/header.rs:1271); `entry` (markdown/from_docx/header.rs:1289); `section_entry` (markdown/from_docx/header.rs:1375); `part_two` (markdown/from_docx/header.rs:1480); `tests::number_and_time_like_values_are_quoted` (markdown/from_docx/header.rs:2342)

#### `plural`  (L212-218)
`fn plural(n: usize, one: &str, many: &str) -> String`

- **Does:** _no doc comment_
- **Macros:** `format!`
- **Called by:** `Ids::table_paragraph` (edit/origin.rs:98) (?); `revisions_comment` (markdown/from_docx/header.rs:230); `render` (markdown/from_docx/header.rs:363)

#### `join`  (L220-222)
`fn join(items: &[String]) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.join`
- **Called by:** `tests::splice_marks_a_tracked_break_and_extends_adjacent_spans` (markdown/from_docx/critic.rs:1989) (?); `tests::a_span_after_a_code_fence_opens_on_the_next_block_s_line` (markdown/from_docx/critic.rs:2019) (?); `tests::a_plain_span_across_paragraphs_names_every_piece_s_id` (markdown/from_docx/critic.rs:2070) (?); `tests::a_tracked_break_extends_spans_only_by_the_same_person` (markdown/from_docx/critic.rs:2368) (?); `revisions_comment` (markdown/from_docx/header.rs:230); `render` (markdown/from_docx/header.rs:363)

#### `revisions_comment`  (L230-361)
`fn revisions_comment(f: &Facts) -> String`

- **Does:** The `revisions:` line's comment: the body's marks and the changes the view shows them as, then formatting (body and styles) and each other story's changes, with its marks when they differ: `9 marks shown as 5 changes: 1 insertion, 4 substitutions; 2 formatting; 3 in header1 (7 marks)`.
- **Calls (crate):** `change_id` (ids.rs:187) L233; `story` (ambiguous, 6 candidates) L238,244,252,274,301; `plural` (markdown/from_docx/header.rs:212) L334,337,345,357; `join` (markdown/from_docx/header.rs:220) L341,346
- **Calls (std/external/unresolved):** `.as_ref`, `.as_str`, `.clone`, `.collect`, `.contains_key`, `.contains`, `.copied`, `.count`, `.entry`, `.filter_map`, `.filter`, `.find`, `.flatten`, `.get`, `.into_iter`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.join`, `.len`, `.map_or_else`, `.map_or`, `.map`, `.or_default`, `.push`, `.then`, `.to_string`, `.unwrap_or`, `HashMap::new`, `Some`, `Vec::new`
- **Macros:** `format!`, `matches!`
- **Called by:** `render` (markdown/from_docx/header.rs:363)

#### `render`  (L363-614, pub(crate))
`pub(crate) fn render(f: &Facts) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `kv` (markdown/from_docx/header.rs:168) L365,370,372,401,406,415...; `scalar` (markdown/from_docx/header.rs:185) L367,372,508,525,530,572; `join` (markdown/from_docx/header.rs:220) L388,496,517,573; `plural` (markdown/from_docx/header.rs:212) L411,412,420,447,516,560...; `revisions_comment` (markdown/from_docx/header.rs:230) L441; `style_token` (markdown/view_notes.rs:1246) L474,481; `Handles::unique_date` (markdown/from_docx/agent.rs:58) L565; `Handles::latest_date` (markdown/from_docx/agent.rs:75) L567; `part_two` (markdown/from_docx/header.rs:1480) L611
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.as_str`, `.checked_sub`, `.cloned`, `.collect`, `.contains`, `.count`, `.ends_with`, `.extend`, `.filter_map`, `.filter`, `.find`, `.fold`, `.get`, `.is_empty`, `.is_none`, `.iter_mut`, `.iter`, `.len`, `.map_or_else`, `.map_or`, `.map`, `.push_str`, `.push`, `.split_once`, `.then`, `.to_string`, `Some`, `String::from`, `String::new`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `twips`  (L618-624)
`fn twips(e: &Element, attr: &str) -> Option<f64>`

- **Does:** A twips-valued attribute; Strict files write the universal measure (`72pt`, `1in`) where Transitional writes the integer.
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:327) L619; `universal_measure_twips` (util/mod.rs:47) L622
- **Calls (std/external/unresolved):** `.and_then`, `.ok`, `.or_else`, `.parse`
- **Called by:** `paragraph_block` (convert/mod.rs:11574) (?); `text_width` (fields.rs:1135) (?); `spacing_clauses` (markdown/from_docx/agent.rs:1606) (?); `geometry_clauses` (markdown/from_docx/agent.rs:2286) (?); `text_width` (markdown/from_docx/header.rs:629); `page_setup` (markdown/from_docx/header.rs:640); `apply` (markdown/from_docx/header.rs:816); `IdLine::clause` (markdown/view_notes.rs:1415) (?); `TableGeometry::clause` (markdown/xml.rs:247) (?)

#### `text_width`  (L629-637, pub(crate))
`pub(crate) fn text_width(sect: Option<&Element>) -> u32`

- **Does:** The page width less the side margins of `sect`, in twips, as the writer computes it (`package.rs`): what it splits a table's columns over when the table line gives no widths (#1472).
- **Calls (crate):** `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L630,631; `twips` (markdown/from_docx/header.rs:618) L632,633; `side` (markdown/borders.rs:189) (by name) L634
- **Calls (std/external/unresolved):** `.and_then`, `.max`, `.round`, `.unwrap_or`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `page_setup`  (L640-759)
`fn page_setup(sect: &Element) -> String`

- **Does:** `Letter portrait, margins 1in, header/footer 0.5in` for a `w:sectPr`.
- **Calls (crate):** `Element::child` (markdown/from_docx/ooxml.rs:386) L641,669,698,716,750; `twips` (markdown/from_docx/header.rs:618) L643,644,670,685,686,693...; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L646,699,708,717,739,750; `inches` (markdown/from_docx/agent.rs:1667) L661,662,706; `side` (markdown/borders.rs:189) (by name) L671
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.find`, `.is_some_and`, `.iter`, `.ok`, `.parse`, `.push_str`, `.round`, `.then_some`, `.to_string`, `.trim_end_matches`, `.unwrap_or`, `Some`, `i64::from`, `inch`
- **Macros:** `format!`, `matches!`
- **Called by:** `part_two` (markdown/from_docx/header.rs:1480); `tests::a_section_s_universal_measures_read_as_twips` (markdown/from_docx/header.rs:2111); `tests::an_mm_spelled_page_is_the_page_its_twips_twin_names` (markdown/from_docx/header.rs:2140); `tests::word_s_paper_list_names_its_sizes` (markdown/from_docx/header.rs:2158)

#### `style_by_id`  (L785-789)
`fn style_by_id<'a>(styles: &'a Element, id: &str) -> Option<&'a Element>`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::children_named` (markdown/from_docx/ooxml.rs:379) L786; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L788
- **Calls (std/external/unresolved):** `.find`, `Some`
- **Called by:** `resolve` (markdown/from_docx/header.rs:875); `named` (markdown/from_docx/header.rs:1071); `table_style_line` (markdown/from_docx/header.rs:1091)

#### `font_of`  (L791-814)
`fn font_of(rpr: &Element, theme: Option<&Element>) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::child` (markdown/from_docx/ooxml.rs:386) L792; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L793,796,798,809; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L809
- **Calls (std/external/unresolved):** `.find_all`, `.first`, `.map`, `.or_else`, `.starts_with`, `.to_string`, `Some`, `Vec::new`
- **Called by:** `apply` (markdown/from_docx/header.rs:816)

#### `apply`  (L816-873)
`fn apply(r: &mut Resolved, ppr: Option<&Element>, rpr: Option<&Element>, theme: Option<&Element>)`

- **Does:** _no doc comment_
- **Calls (crate):** `font_of` (markdown/from_docx/header.rs:791) L819; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L825,846,869; `universal_measure_half_points` (util/mod.rs:52) L828; `twips` (markdown/from_docx/header.rs:618) L838,841,844,854,857,860...
- **Calls (std/external/unresolved):** `.and_then`, `.child`, `.is_none`, `.is_some`, `.map`, `.ok`, `.or_else`, `.parse`, `.to_string`, `.toggle`, `.unwrap_or`, `Some`
- **Called by:** `run_resolution` (bin/jubarte.rs:458) (?); `reresolve_theme_color_hexes` (document_comparer.rs:6012) (?); `resolve` (markdown/from_docx/header.rs:875)

#### `resolve`  (L875-908)
`fn resolve(styles: Option<&Element>, theme: Option<&Element>, id: &str) -> Resolved`

- **Does:** _no doc comment_
- **Calls (crate):** `style_by_id` (markdown/from_docx/header.rs:785) L880,890; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L880,896; `apply` (markdown/from_docx/header.rs:816) L893,900; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L893,894; `Element::path` (markdown/from_docx/ooxml.rs:391) (by name) L902,903
- **Calls (std/external/unresolved):** `.and_then`, `.child`, `.is_some_and`, `.map`, `.take`, `.to_string`, `Resolved::default`, `Some`
- **Called by:** `paragraph_style_sizes` (markdown/from_docx/header.rs:921); `part_two` (markdown/from_docx/header.rs:1480); `tests::the_legend_names_an_indent_contextual_spacing_and_keep_lines` (markdown/from_docx/header.rs:2398); `tests::the_legend_names_a_right_indent` (markdown/from_docx/header.rs:2443); `tests::a_zero_space_or_a_single_line_apart_from_the_base_is_named` (markdown/from_docx/header.rs:2472); `tests::a_style_s_universal_measures_read_as_their_integers` (markdown/from_docx/header.rs:2506)

#### `is_paragraph_style`  (L913-915, pub(crate))
`pub(crate) fn is_paragraph_style(style: &Element) -> bool`

- **Does:** Whether `style` is a paragraph style: ECMA-376 `w:style` reads "If this attribute is not specified, then the default value shall be assumed to be paragraph" (#1485 review).
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:327) L914
- **Calls (std/external/unresolved):** `.is_none_or`
- **Called by:** `paragraph_style_sizes` (markdown/from_docx/header.rs:921); `convert_marks` (markdown/from_docx/mod.rs:108); `Styles::parse` (markdown/from_docx/mod.rs:1809); `story_frame` (markdown/from_docx/mod.rs:4880)

#### `paragraph_style_sizes`  (L921-942, pub(crate))
`pub(crate) fn paragraph_style_sizes( styles: Option<&Element>, default_style: Option<&str>, ) -> (HashMap<String, u32>, u32)`

- **Does:** Each paragraph style's text size through its `basedOn` chain and the document defaults, in half-points, and the size of a paragraph with no `w:pStyle` (the default paragraph style's): what a paragraph's own `size` clause is measured against (#1474).
- **Calls (crate):** `resolve` (markdown/from_docx/header.rs:875) L926; `Element::children_named` (markdown/from_docx/ooxml.rs:379) (by name) L936; `is_paragraph_style` (markdown/from_docx/header.rs:913) L937; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L938; `own_size::{fn size}` (markdown/from_docx/agent.rs:1116) L939,941
- **Calls (std/external/unresolved):** `.clamp`, `.collect`, `.filter_map`, `.filter`, `.flat_map`, `.into_iter`, `.is_finite`, `.map`, `.round`, `.to_string`, `.unwrap_or`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `points`  (L944-947)
`fn points(twips: f64) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.to_string`, `.trim_end_matches`
- **Macros:** `format!`
- **Called by:** `describe` (markdown/from_docx/header.rs:963)

#### `align_word`  (L949-956)
`fn align_word(align: Option<&str>) -> &'static str`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `describe` (markdown/from_docx/header.rs:963)

#### `describe`  (L963-1067)
`fn describe(r: &Resolved, base: Option<&Resolved>) -> String`

- **Does:** The default style's line prints everything; a heading line always prints its font and size, then only what differs from `base` (the default style), so the legend reads as deviations.
- **Calls (crate):** `points` (markdown/from_docx/header.rs:944) L1003,1021; `inches` (markdown/from_docx/agent.rs:1667) L1036,1042,1048,1054; `align_word` (markdown/from_docx/header.rs:949) L1060,1061,1062
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.is_none_or`, `.is_some`, `.join`, `.push_str`, `.push`, `.to_string`, `.trim_end_matches`, `.unwrap_or_else`, `.unwrap_or`, `Vec::new`, `differs`, `pick`
- **Macros:** `format!`
- **Called by:** `part_two` (markdown/from_docx/header.rs:1480); `tests::the_legend_names_an_indent_contextual_spacing_and_keep_lines` (markdown/from_docx/header.rs:2398); `tests::the_legend_names_a_right_indent` (markdown/from_docx/header.rs:2443); `tests::a_zero_space_or_a_single_line_apart_from_the_base_is_named` (markdown/from_docx/header.rs:2472); `tests::a_style_s_universal_measures_read_as_their_integers` (markdown/from_docx/header.rs:2506)

#### `named`  (L1071-1079)
`fn named(styles: Option<&Element>, id: &str, expected: &str) -> String`

- **Does:** `, named Text Heading 2`: the name of style `id` when it is not the one the writer would give it (`expected`).
- **Calls (crate):** `style_by_id` (markdown/from_docx/header.rs:785) L1073; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L1074; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L1075
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.map`, `.unwrap_or_default`
- **Macros:** `format!`
- **Called by:** `part_two` (markdown/from_docx/header.rs:1480)

#### `quoted_name`  (L1083-1089)
`fn quoted_name(name: &str) -> String`

- **Does:** A legend `named` value: quoted (a JSON string) when it holds a comma or a quote, so the clause's `, ` separator cannot split it (#549).
- **Calls (std/external/unresolved):** `.contains`, `.to_string`, `.unwrap_or_else`, `serde_json::to_string`
- **Macros:** `format!`
- **Called by:** `table_style_line` (markdown/from_docx/header.rs:1091)

#### `table_style_line`  (L1091-1166)
`fn table_style_line(styles: Option<&Element>, id: &str) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `style_by_id` (markdown/from_docx/header.rs:785) L1095,1106; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L1096,1123,1155; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L1097,1125,1126; `word_table_style_name` (markdown/package.rs:877) L1100; `quoted_name` (markdown/from_docx/header.rs:1083) L1103; `Element::path` (markdown/from_docx/ooxml.rs:391) (by name) L1107; `is_built_in_table_style` (markdown/package.rs:848) L1147; `border_side` (markdown/from_docx/header.rs:1171) L1156
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.as_ref`, `.clone`, `.cloned`, `.collect`, `.filter_map`, `.filter`, `.find_map`, `.first`, `.flatten`, `.is_empty`, `.iter`, `.join`, `.map`, `.parse`, `.to_string`, `.trim_end_matches`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `.zip`, `Some`
- **Macros:** `format!`
- **Called by:** `part_two` (markdown/from_docx/header.rs:1480); `tests::a_table_style_s_start_end_borders_read_as_left_right` (markdown/from_docx/header.rs:2197)

#### `border_side`  (L1171-1192)
`fn border_side(side: &str, b: &Element) -> Option<String>`

- **Does:** One side of a `borders …` clause: `bottom double 0.75pt`, with `#RRGGBB` for a colour that is not `auto` and `space Npt` for a spacing that is not zero.
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:327) L1172,1174,1178,1184
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.chars`, `.filter`, `.is_ascii_hexdigit`, `.is_empty`, `.len`, `.ok`, `.parse`, `.trim_end_matches`, `Some`
- **Macros:** `format!`, `write!`
- **Called by:** `table_style_line` (markdown/from_docx/header.rs:1091)

#### `story_style`  (L1196-1198, pub(crate))
`pub(crate) fn story_style(kind: &str) -> &'static str`

- **Does:** The style the writer gives a `header` or `footer` paragraph the view names no style for: Word's own.
- **Calls:** none
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `by_paragraph`  (L1203-1211)
`fn by_paragraph(s: &StoryFact) -> bool`

- **Does:** A story prints paragraph by paragraph (`p0: {text: …, color: …}`) when one carries its own colour, borders, frame or indent, or their alignments differ.
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.is_some`, `.iter`
- **Called by:** `story_kv` (markdown/from_docx/header.rs:1271); `entry` (markdown/from_docx/header.rs:1289)

#### `paragraph_entries`  (L1215-1245)
`fn paragraph_entries(s: &StoryFact) -> Vec<String>`

- **Does:** `p0: {text: …, center, color: "595959", border: bottom single 0.5pt, frame: {wrap: none, y: 1}}`, one per paragraph.
- **Calls (crate):** `scalar` (markdown/from_docx/header.rs:185) L1221,1234,1237
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.enumerate`, `.is_empty`, `.iter`, `.join`, `.map`, `.push_str`, `.zip`, `std::iter::once`
- **Macros:** `format!`
- **Called by:** `story_kv` (markdown/from_docx/header.rs:1271); `entry` (markdown/from_docx/header.rs:1289)

#### `story_tail`  (L1248-1267)
`fn story_tail(s: &StoryFact) -> Vec<String>`

- **Does:** A story's entries after its text: its styles, `inactive`, its changes.
- **Calls (crate):** `scalar` (markdown/from_docx/header.rs:185) L1252,1254,1263
- **Calls (std/external/unresolved):** `.as_slice`, `.collect`, `.is_empty`, `.iter`, `.join`, `.map`, `.push`, `.to_string`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `story_kv` (markdown/from_docx/header.rs:1271); `entry` (markdown/from_docx/header.rs:1289)

#### `story_kv`  (L1271-1287)
`fn story_kv(out: &mut String, key: &str, s: &StoryFact, comment: Option<&str>)`

- **Does:** A story's entry in the first section's block: one line, or, paragraph by paragraph, `key:` then its entries indented below.
- **Calls (crate):** `by_paragraph` (markdown/from_docx/header.rs:1203) L1272; `kv` (markdown/from_docx/header.rs:168) L1273,1276; `entry` (markdown/from_docx/header.rs:1289) L1273; `scalar` (markdown/from_docx/header.rs:185) L1277; `paragraph_entries` (markdown/from_docx/header.rs:1215) L1278; `story_tail` (markdown/from_docx/header.rs:1248) L1278
- **Calls (std/external/unresolved):** `.chain`, `.into_iter`, `.push_str`, `.to_string`
- **Macros:** `format!`
- **Called by:** `first_section_stories` (markdown/from_docx/header.rs:1317)

#### `entry`  (L1289-1314)
`fn entry(s: &StoryFact) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `by_paragraph` (markdown/from_docx/header.rs:1203) L1290; `scalar` (markdown/from_docx/header.rs:185) L1291,1298,1302,1306; `paragraph_entries` (markdown/from_docx/header.rs:1215) L1292; `story_tail` (markdown/from_docx/header.rs:1248) L1293,1310
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.is_empty`, `.join`, `.map`, `.push_str`, `std::iter::once`
- **Macros:** `format!`
- **Called by:** `story_kv` (markdown/from_docx/header.rs:1271); `section_entry` (markdown/from_docx/header.rs:1375)

#### `first_section_stories`  (L1317-1371)
`fn first_section_stories(out: &mut String, section: &SectionFact)`

- **Does:** The `headers:` / `footers:` blocks of the first section.
- **Calls (crate):** `find` (ambiguous: `find` (markdown/pages.rs:452); `find` (markdown/unified.rs:474)) L1326,1345,1354,1358; `story_kv` (markdown/from_docx/header.rs:1271) L1327,1347,1355,1359; `kv` (markdown/from_docx/header.rs:168) L1333,1356
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.find`, `.is_empty`, `.iter`, `.push_str`, `Some`
- **Macros:** `format!`
- **Called by:** `part_two` (markdown/from_docx/header.rs:1480)

#### `section_entry`  (L1375-1420)
`fn section_entry( section: &SectionFact, previous: &SectionFact, first: &SectionFact, handles: Option<&Handles>, ) -> String`

- **Does:** `{p2-p3, page: …, columns 2, headers: {default: {…}}}`: what a later section changes against the one before it (page setup against the first).
- **Calls (crate):** `section_changes` (markdown/from_docx/header.rs:1424) L1388; `scalar` (markdown/from_docx/header.rs:185) L1389; `entry` (markdown/from_docx/header.rs:1289) L1405
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_ref`, `.collect`, `.filter`, `.insert`, `.is_empty`, `.iter`, `.join`, `.map`, `.push`, `.to_string`, `setup`
- **Macros:** `format!`, `vec!`
- **Called by:** `part_two` (markdown/from_docx/header.rs:1480)

#### `section_changes`  (L1424-1431)
`fn section_changes(section: &SectionFact, handles: &Handles) -> Option<String>`

- **Does:** The tracked view's tags for a section's own changes (`w:sectPrChange`): `r9 @AC`.
- **Calls (crate):** `Element::children_named` (markdown/from_docx/ooxml.rs:379) (by name) L1426; `required_date_tag_of` (markdown/from_docx/agent.rs:202) L1428; `format_tags` (markdown/from_docx/agent.rs:550) L1430
- **Calls (std/external/unresolved):** `.as_ref`, `.collect`, `.is_empty`, `.map`, `.then`
- **Called by:** `section_entry` (markdown/from_docx/header.rs:1375); `part_two` (markdown/from_docx/header.rs:1480)

#### `style_changes`  (L1436-1464)
`fn style_changes(styles: &Element, handles: &Handles) -> Vec<(String, String)>`

- **Does:** `Normal: styles.r0 @AS, Heading1: styles.r3 @AS`: the styles part's tracked changes by the style that holds them (`docDefaults` for the document defaults), in the tracked view.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L1438; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L1439; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L1441; `collect` (markdown/from_docx/header.rs:1467) L1448; `required_date_tag_of` (markdown/from_docx/agent.rs:202) L1453; `format_tags` (markdown/from_docx/agent.rs:550) L1460
- **Calls (std/external/unresolved):** `.contains`, `.is_empty`, `.push`, `.to_string`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `part_two` (markdown/from_docx/header.rs:1480)

#### `collect`  (L1467-1478)
`fn collect<'e>(e: &'e Element, out: &mut Vec<&'e Element>)`

- **Does:** The elements under `e` that record a tracked change.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L1468; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L1469; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L1471; `collect` (markdown/from_docx/header.rs:1467) L1476
- **Calls (std/external/unresolved):** `.ends_with`, `.is_some`, `.push`
- **Macros:** `matches!`
- **Called by:** `get_changed_property_names` (comparer/formatchg.rs:217) (?); `style_changes` (markdown/from_docx/header.rs:1436); `collect` (markdown/from_docx/header.rs:1467)

#### `part_two`  (L1480-1712)
`fn part_two(out: &mut String, f: &Facts)`

- **Does:** _no doc comment_
- **Calls (crate):** `page_setup` (markdown/from_docx/header.rs:640) L1484; `section_changes` (markdown/from_docx/header.rs:1424) L1486; `kv` (markdown/from_docx/header.rs:168) L1490,1507,1529,1548,1573,1592...; `resolve` (markdown/from_docx/header.rs:875) L1494,1537,1556,1581; `style_changes` (markdown/from_docx/header.rs:1436) L1502; `scalar` (markdown/from_docx/header.rs:185) L1511,1512,1534,1552,1553,1577...; `describe` (markdown/from_docx/header.rs:963) L1515,1537,1556,1581; `named` (markdown/from_docx/header.rs:1071) L1516,1538,1557,1582; `table_style_line` (markdown/from_docx/header.rs:1091) L1596,1607; `first_section_stories` (markdown/from_docx/header.rs:1317) L1691; `section_entry` (markdown/from_docx/header.rs:1375) L1700
- **Calls (std/external/unresolved):** `.any`, `.as_ref`, `.as_slice`, `.as_str`, `.clone`, `.collect`, `.enumerate`, `.filter`, `.first`, `.get`, `.into_iter`, `.is_empty`, `.is_none`, `.is_ok`, `.iter`, `.join`, `.len`, `.map_or_else`, `.map_or`, `.map`, `.or_else`, `.parse`, `.position`, `.push_str`, `.push`, `.repeat`, `.skip`, `.sort_by_key`, `.strip_prefix`, `.then_some`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `Some`, `Vec::new`, `order`
- **Macros:** `format!`
- **Called by:** `render` (markdown/from_docx/header.rs:363)

#### `story_paragraphs`  (L1717-1719, pub(crate))
`pub(crate) fn story_paragraphs(root: &Element) -> Vec<&Element>`

- **Does:** A header/footer part's paragraphs in document order, numbered as `edit` numbers a story (`inspect::story_paragraph_nodes`): every `w:p` outside text boxes, table cells and content controls included.
- **Calls (crate):** `paragraphs` (markdown/from_docx/agent.rs:1908) L1718
- **Called by:** `tests::story_paragraphs_agree_with_the_paragraphs_edit_resolves` (markdown/from_docx/header.rs:2097); `tests::a_story_field_prints_its_whole_code_and_its_result` (markdown/from_docx/header.rs:2257); `tests::a_story_field_without_a_result_prints_its_bare_code` (markdown/from_docx/header.rs:2273); `tests::a_page_field_in_a_block_content_control_is_found` (markdown/from_docx/header.rs:2289); `tests::runs_inside_inline_wrappers_count_and_deletions_do_not` (markdown/from_docx/header.rs:2306); `tests::paragraphs_number_as_edit_does_tables_in_and_text_boxes_out` (markdown/from_docx/header.rs:2320); `tests::a_break_in_a_header_line_is_a_newline` (markdown/from_docx/header.rs:2366); `tests::a_nested_field_keeps_the_outer_code` (markdown/from_docx/header.rs:2382); `convert_marks` (markdown/from_docx/mod.rs:108)

#### `story_text_with`  (L1754-2048, pub(crate))
`pub(crate) fn story_text_with( p: &Element, view: StoryView, draw: &mut dyn FnMut(&Element, bool) -> Option<String>, ) -> String`

- **Does:** [`story_text`], calling `draw` for each `w:drawing`, `w:pict`, `w:object` and drawing-holding `mc:AlternateContent` element: its Markdown (`None` prints nothing).
- **Calls (crate):** `story_text_with::{fn walk}` (markdown/from_docx/header.rs:1819) L2029; `State::flush` (markdown/from_docx/header.rs:2004) L2031; `Critic::into_story` (markdown/from_docx/critic.rs:496) L2032; `Element::path` (markdown/from_docx/ooxml.rs:391) L2034; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L2036; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L2036; `format_tag` (markdown/from_docx/agent.rs:310) L2037; `required_date_tag_of` (markdown/from_docx/agent.rs:202) L2039
- **Calls (std/external/unresolved):** `.filter`, `.push_str`, `.to_string`, `.trim`, `StoryView::Tagged`, `String::new`, `Vec::new`, `super::critic::Critic::default`
- **Macros:** `format!`, `matches!`
- **Called by:** `story_text` (markdown/from_docx/header.rs:1745); `convert_marks` (markdown/from_docx/mod.rs:108)

#### `story_text_with::{fn field_end}`  (L1791-1802)
`fn field_end(out: &str, at: usize, instr: &str, nested: bool) -> &'static str`

- **Does:** A printed field's end: `{/}`, or nothing when it has no result and its code is a bare name, which the writer reads as `{PAGE}`.
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.is_ascii_digit`, `.is_ascii_uppercase`, `.is_empty`, `.len`, `.trim`
- **Called by:** `story_text_with::{fn walk}` (markdown/from_docx/header.rs:1819); `story_text_with::{fn run_content}` (markdown/from_docx/header.rs:1923)

#### `story_text_with::{fn inserting}`  (L1804-1810)
`fn inserting(local: &str) -> Option<bool>`

- **Does:** `ins`/`moveTo` → `Some(true)`, `del`/`moveFrom` → `Some(false)`.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `story_text_with::{fn walk}` (markdown/from_docx/header.rs:1819)

#### `story_text_with::{fn code}`  (L1811-1818)
`fn code(instr: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.next`, `.split_whitespace`, `.to_uppercase`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `tests::a_code_block_breaks_only_the_delimiter_shapes_it_holds` (markdown/from_docx/critic.rs:2541) (?); `story_text_with::{fn walk}` (markdown/from_docx/header.rs:1819); `story_text_with::{fn run_content}` (markdown/from_docx/header.rs:1923)

#### `story_text_with::{fn walk}`  (L1819-1920)
`fn walk( parent: &Element, s: &mut State, draw: &mut dyn FnMut(&Element, bool) -> Option<String>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L1824; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L1825; `capturable_alternate` (markdown/from_docx/drawing.rs:1516) L1831; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L1837,1900,1910; `story_text_with::{fn walk}` (markdown/from_docx/header.rs:1819) L1838,1849,1854,1891,1898,1901; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L1842; `printed_code` (markdown/from_docx/mod.rs:2258) L1843; `story_text_with::{fn field_end}` (markdown/from_docx/header.rs:1791) L1851; `story_text_with::{fn code}` (markdown/from_docx/header.rs:1811) L1853; `story_text_with::{fn inserting}` (markdown/from_docx/header.rs:1804) L1857,1858; `State::flush` (markdown/from_docx/header.rs:2004) L1879,1894; `required_date_tag_of` (markdown/from_docx/agent.rs:202) L1883; `Critic::open` (markdown/from_docx/critic.rs:436) L1885; `Critic::close` (markdown/from_docx/critic.rs:448) L1895; `hidden_run` (markdown/from_docx/agent.rs:157) L1910; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L1913; `story_text_with::{fn run_content}` (markdown/from_docx/header.rs:1923) L1914
- **Calls (std/external/unresolved):** `.filter`, `.is_empty`, `.is_some`, `.len`, `.or_else`, `.push_str`, `.unwrap_or`, `Some`, `draw`
- **Macros:** `format!`
- **Called by:** `story_text_with` (markdown/from_docx/header.rs:1754); `story_text_with::{fn walk}` (markdown/from_docx/header.rs:1819)

#### `story_text_with::{fn run_content}`  (L1923-2002)
`fn run_content( child: &Element, s: &mut State, draw: &mut dyn FnMut(&Element, bool) -> Option<String>, )`

- **Does:** One child of a `w:r` (or of a run-level `mc:Choice`): text, field characters, breaks, and drawings `draw` renders.
- **Calls (crate):** `Element::local` (markdown/from_docx/ooxml.rs:318) L1928,1948; `holds_drawing` (markdown/from_docx/drawing.rs:1483) L1934; `Element::child` (markdown/from_docx/ooxml.rs:386) L1940; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L1941; `story_text_with::{fn run_content}` (markdown/from_docx/header.rs:1923) L1942; `Element::attr` (markdown/from_docx/ooxml.rs:327) L1949; `printed_code` (markdown/from_docx/mod.rs:2258) L1962; `story_text_with::{fn field_end}` (markdown/from_docx/header.rs:1791) L1975; `story_text_with::{fn code}` (markdown/from_docx/header.rs:1811) L1978; `Element::text` (markdown/from_docx/ooxml.rs:415) L1986,1993,1994; `State::push_text` (markdown/from_docx/header.rs:2012) L1993,1994
- **Calls (std/external/unresolved):** `.all`, `.filter`, `.is_empty`, `.iter`, `.last_mut`, `.len`, `.or_else`, `.pop`, `.push_str`, `.push`, `.to_string`, `Open::default`, `draw`
- **Called by:** `story_text_with::{fn walk}` (markdown/from_docx/header.rs:1819); `story_text_with::{fn run_content}` (markdown/from_docx/header.rs:1923)

#### `State::flush`  (L2004-2007)
`fn flush(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Critic::raw` (markdown/from_docx/critic.rs:382) L2005
- **Calls (std/external/unresolved):** `.len`
- **Called by:** `story_text_with` (markdown/from_docx/header.rs:1754); `story_text_with::{fn walk}` (markdown/from_docx/header.rs:1819)

#### `State::push_text`  (L2012-2018)
`fn push_text(&mut self, text: &str)`

- **Does:** Document text; the tagged view escapes what reads like a mark, and every view text that reads like a field code (`\{PAGE}`), as the body does.
- **Calls (crate):** `escape_field_codes` (markdown/from_docx/header.rs:2054) L2013; `escape` (markdown/from_docx/critic.rs:1135) L2015
- **Calls (std/external/unresolved):** `.push_str`
- **Called by:** `story_text_with::{fn run_content}` (markdown/from_docx/header.rs:1923)

#### `escape_field_codes`  (L2054-2079)
`fn escape_field_codes(text: &str) -> std::borrow::Cow<'_, str>`

- **Does:** `text` with a backslash before each `{` the writer would read as a field: one that opens what reads like a field code (`{PAGE \* roman}`), a bare name (`{SELLER}`, `{1999}`) or a result's end (`{/}`), so text typed in a header stays text.
- **Calls (crate):** `field_code_close` (markdown/critic.rs:1636) L2065; `is_field_code` (markdown/critic.rs:1620) L2066
- **Calls (std/external/unresolved):** `.all`, `.any`, `.bytes`, `.char_indices`, `.find`, `.is_ascii_digit`, `.is_ascii_uppercase`, `.is_some_and`, `.len`, `.push`, `.starts_with`, `String::with_capacity`, `opens`, `std::borrow::Cow::Borrowed`, `std::borrow::Cow::Owned`
- **Called by:** `State::push_text` (markdown/from_docx/header.rs:2012)

## Test functions (21)

- `story_text` (L1745): A header/footer paragraph's text with fields as the body prints them, `{CODE}result{/}` (`{PAGE}1{/}`, `{HYPERLINK "https://…"}site{/}`), and a code jubarte does not print whole as its name alone (`{XE}`), as is a code holding a nested field (`{IF}`).
- `tests::ftr` (L2088): _no doc comment_
- `tests::story_paragraphs_agree_with_the_paragraphs_edit_resolves` (L2097): Review t09-15: header and footer ids must number paragraphs exactly as `edit` resolves them, so the two walks are checked against each other.
- `tests::a_section_s_universal_measures_read_as_twips` (L2111): _no doc comment_
- `tests::an_mm_spelled_page_is_the_page_its_twips_twin_names` (L2140): An mm-spelled page is the page its twips twin names: 210mm x 297mm is A4 once rounded to the twips it is (11905.5 → 11906), not the `8.268x11.693in` a truncating cast reads from any producer whose spelling is not Word's own `595.3pt` (#531 review defect 6).
- `tests::word_s_paper_list_names_its_sizes` (L2158): Word's paper list: every size Word names and the writer writes back exactly is named by the rounded twips Word spells it in — the short side first, whatever way the sheet lies (#561 review defect 4).
- `tests::a_table_style_s_start_end_borders_read_as_left_right` (L2197): A strict table style names its side borders `start`/`end`: the legend reads them as the `left`/`right` they name, and one side a twip apart still varies.
- `tests::a_long_value_aligns_by_characters_not_bytes` (L2228): _no doc comment_
- `tests::an_empty_comment_leaves_no_hash` (L2250): _no doc comment_
- `tests::a_story_field_prints_its_whole_code_and_its_result` (L2257): _no doc comment_
- `tests::a_story_field_without_a_result_prints_its_bare_code` (L2273): _no doc comment_
- `tests::a_page_field_in_a_block_content_control_is_found` (L2289): _no doc comment_
- `tests::runs_inside_inline_wrappers_count_and_deletions_do_not` (L2306): _no doc comment_
- `tests::paragraphs_number_as_edit_does_tables_in_and_text_boxes_out` (L2320): _no doc comment_
- `tests::number_and_time_like_values_are_quoted` (L2342): pi review r392 F1/F6: values a YAML 1.1 or 1.2 reader takes as a number, a time or a date are quoted; prose that starts with a digit stays plain.
- `tests::a_break_in_a_header_line_is_a_newline` (L2366): pi review av2 F18: a line break inside a header paragraph is a line break in its text, not two words glued together.
- `tests::a_nested_field_keeps_the_outer_code` (L2382): pi review av2 F18: a field nested in another's instruction does not replace it; the outer field names the code, its result stays hidden.
- `tests::the_legend_names_an_indent_contextual_spacing_and_keep_lines` (L2398): A paragraph indent, contextual spacing and keep-lines are named; a style with none of them keeps today's line.
- `tests::the_legend_names_a_right_indent` (L2443): A style's right indent is named, as `right 0.5in` (the id line's word): the legend dropped it, and the written text ran wider (#1489).
- `tests::a_zero_space_or_a_single_line_apart_from_the_base_is_named` (L2472): A strict style carries its measures as universal measures (`11pt`, `1in`) and its font size in half-points either way: the legend reads the same as the transitional integers.
- `tests::a_style_s_universal_measures_read_as_their_integers` (L2506): _no doc comment_
