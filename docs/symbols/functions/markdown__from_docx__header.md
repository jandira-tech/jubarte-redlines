# Functions and methods in `src/markdown/from_docx/header.rs`

41 non-test, 19 test. Part of the [symbol manifest](../README.md).

#### `kv`  (L157-168)
`fn kv(out: &mut String, key_value: &str, comment: Option<&str>)`

- **Does:** `key: value` padded to 35 columns then `# comment`; a key/value longer than 33 characters takes two spaces.
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.filter`, `.is_empty`, `.push_str`, `.push`
- **Macros:** `format!`
- **Called by:** `render` (markdown/from_docx/header.rs:306); `story_kv` (markdown/from_docx/header.rs:1080); `first_section_stories` (markdown/from_docx/header.rs:1126); `part_two` (markdown/from_docx/header.rs:1286); `tests::a_long_value_aligns_by_characters_not_bytes` (markdown/from_docx/header.rs:2085); `tests::an_empty_comment_leaves_no_hash` (markdown/from_docx/header.rs:2107)

#### `scalar`  (L174-199, pub(crate))
`pub(crate) fn scalar(value: &str, flow: bool) -> String`

- **Does:** `value` as a YAML scalar: plain when every YAML reader, 1.1 or 1.2, takes it back as the same string, else double-quoted (a JSON string is a valid YAML one).
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.chars`, `.contains`, `.ends_with`, `.is_ascii_digit`, `.is_empty`, `.starts_with`, `.to_ascii_lowercase`, `.to_string`, `.trim`, `.unwrap_or_else`, `serde_json::to_string`
- **Macros:** `format!`
- **Called by:** `render` (markdown/from_docx/header.rs:306); `paragraph_entries` (markdown/from_docx/header.rs:1035); `story_tail` (markdown/from_docx/header.rs:1057); `story_kv` (markdown/from_docx/header.rs:1080); `entry` (markdown/from_docx/header.rs:1098); `section_entry` (markdown/from_docx/header.rs:1184); `part_two` (markdown/from_docx/header.rs:1286); `tests::number_and_time_like_values_are_quoted` (markdown/from_docx/header.rs:2199)

#### `plural`  (L201-207)
`fn plural(n: usize, one: &str, many: &str) -> String`

- **Does:** _no doc comment_
- **Macros:** `format!`
- **Called by:** `Ids::table_paragraph` (edit/origin.rs:95) (?); `revisions_comment` (markdown/from_docx/header.rs:217); `render` (markdown/from_docx/header.rs:306)

#### `join`  (L209-211)
`fn join(items: &[String]) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.join`
- **Called by:** `tests::splice_marks_a_tracked_break_and_extends_adjacent_spans` (markdown/from_docx/critic.rs:1962) (?); `tests::a_span_after_a_code_fence_opens_on_the_next_block_s_line` (markdown/from_docx/critic.rs:1992) (?); `tests::a_plain_span_across_paragraphs_names_every_piece_s_id` (markdown/from_docx/critic.rs:2043) (?); `tests::a_tracked_break_extends_spans_only_by_the_same_person` (markdown/from_docx/critic.rs:2299) (?); `revisions_comment` (markdown/from_docx/header.rs:217); `render` (markdown/from_docx/header.rs:306)

#### `revisions_comment`  (L217-304)
`fn revisions_comment(f: &Facts) -> String`

- **Does:** The `revisions:` line's comment: the body's marks and the changes the view shows them as, then formatting (body and styles) and each other story's count: `9 marks shown as 5 changes: 1 insertion, 4 substitutions; 2 formatting; 1 in header1`.
- **Calls (crate):** `change_id` (ids.rs:187) L220; `story` (ambiguous, 6 candidates) L225,231,239,261; `plural` (markdown/from_docx/header.rs:201) L283,286,292; `join` (markdown/from_docx/header.rs:209) L293
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.collect`, `.contains_key`, `.contains`, `.count`, `.entry`, `.filter_map`, `.filter`, `.find`, `.get`, `.into_iter`, `.is_empty`, `.iter`, `.join`, `.len`, `.map_or_else`, `.map`, `.or_default`, `.push`, `.to_string`, `HashMap::new`, `Vec::new`
- **Macros:** `format!`, `matches!`
- **Called by:** `render` (markdown/from_docx/header.rs:306)

#### `render`  (L306-557, pub(crate))
`pub(crate) fn render(f: &Facts) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `kv` (markdown/from_docx/header.rs:157) L308,313,315,344,349,358...; `scalar` (markdown/from_docx/header.rs:174) L310,315,451,468,473,515; `join` (markdown/from_docx/header.rs:209) L331,439,460,516; `plural` (markdown/from_docx/header.rs:201) L354,355,363,390,459,503...; `revisions_comment` (markdown/from_docx/header.rs:217) L384; `style_token` (markdown/view_notes.rs:1185) L417,424; `Handles::unique_date` (markdown/from_docx/agent.rs:53) L508; `Handles::date_range` (markdown/from_docx/agent.rs:61) L510; `part_two` (markdown/from_docx/header.rs:1286) L554
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.as_str`, `.checked_sub`, `.cloned`, `.collect`, `.contains`, `.count`, `.ends_with`, `.extend`, `.filter_map`, `.filter`, `.find`, `.fold`, `.get`, `.is_empty`, `.is_none`, `.iter_mut`, `.iter`, `.len`, `.map_or_else`, `.map_or`, `.map`, `.push_str`, `.push`, `.split_once`, `.then`, `.to_string`, `Some`, `String::from`, `String::new`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

#### `twips`  (L561-567)
`fn twips(e: &Element, attr: &str) -> Option<f64>`

- **Does:** A twips-valued attribute; Strict files write the universal measure (`72pt`, `1in`) where Transitional writes the integer.
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:286) L562; `universal_measure_twips` (util/mod.rs:42) L565
- **Calls (std/external/unresolved):** `.and_then`, `.ok`, `.or_else`, `.parse`
- **Called by:** `paragraph_block` (convert/mod.rs:11573) (?); `text_width` (fields.rs:1135) (?); `page_setup` (markdown/from_docx/header.rs:570); `apply` (markdown/from_docx/header.rs:744); `IdLine::clause` (markdown/view_notes.rs:1349) (?)

#### `page_setup`  (L570-689)
`fn page_setup(sect: &Element) -> String`

- **Does:** `Letter portrait, margins 1in, header/footer 0.5in` for a `w:sectPr`.
- **Calls (crate):** `Element::child` (markdown/from_docx/ooxml.rs:315) L571,599,628,646,680; `twips` (markdown/from_docx/header.rs:561) L573,574,600,615,616,623...; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L576,629,638,647,669,680; `inches` (markdown/from_docx/agent.rs:1206) L591,592,636; `side` (markdown/borders.rs:189) (by name) L601
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.find`, `.is_some_and`, `.iter`, `.ok`, `.parse`, `.push_str`, `.round`, `.then_some`, `.to_string`, `.trim_end_matches`, `.unwrap_or`, `Some`, `i64::from`, `inch`
- **Macros:** `format!`, `matches!`
- **Called by:** `part_two` (markdown/from_docx/header.rs:1286); `tests::a_section_s_universal_measures_read_as_twips` (markdown/from_docx/header.rs:1971); `tests::an_mm_spelled_page_is_the_page_its_twips_twin_names` (markdown/from_docx/header.rs:2000); `tests::word_s_paper_list_names_its_sizes` (markdown/from_docx/header.rs:2018)

#### `style_by_id`  (L713-717)
`fn style_by_id<'a>(styles: &'a Element, id: &str) -> Option<&'a Element>`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::children_named` (markdown/from_docx/ooxml.rs:308) L714; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L716
- **Calls (std/external/unresolved):** `.find`, `Some`
- **Called by:** `resolve` (markdown/from_docx/header.rs:800); `named` (markdown/from_docx/header.rs:941); `table_style_line` (markdown/from_docx/header.rs:961)

#### `font_of`  (L719-742)
`fn font_of(rpr: &Element, theme: Option<&Element>) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::child` (markdown/from_docx/ooxml.rs:315) L720; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L721,724,726,737; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:315); `Scope::child` (xmllinq/serialize.rs:225)) L737
- **Calls (std/external/unresolved):** `.find_all`, `.first`, `.map`, `.or_else`, `.starts_with`, `.to_string`, `Some`, `Vec::new`
- **Called by:** `apply` (markdown/from_docx/header.rs:744)

#### `apply`  (L744-798)
`fn apply(r: &mut Resolved, ppr: Option<&Element>, rpr: Option<&Element>, theme: Option<&Element>)`

- **Does:** _no doc comment_
- **Calls (crate):** `font_of` (markdown/from_docx/header.rs:719) L747; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L753,774,794; `universal_measure_half_points` (util/mod.rs:47) L756; `twips` (markdown/from_docx/header.rs:561) L766,769,772,782,785,788
- **Calls (std/external/unresolved):** `.and_then`, `.child`, `.is_none`, `.is_some`, `.map`, `.ok`, `.or_else`, `.parse`, `.to_string`, `.toggle`, `.unwrap_or`, `Some`
- **Called by:** `run_resolution` (bin/jubarte.rs:361) (?); `reresolve_theme_color_hexes` (document_comparer.rs:6004) (?); `resolve` (markdown/from_docx/header.rs:800)

#### `resolve`  (L800-833)
`fn resolve(styles: Option<&Element>, theme: Option<&Element>, id: &str) -> Resolved`

- **Does:** _no doc comment_
- **Calls (crate):** `style_by_id` (markdown/from_docx/header.rs:713) L805,815; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L805,821; `apply` (markdown/from_docx/header.rs:744) L818,825; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:315); `Scope::child` (xmllinq/serialize.rs:225)) L818,819; `Element::path` (markdown/from_docx/ooxml.rs:320) (by name) L827,828
- **Calls (std/external/unresolved):** `.and_then`, `.child`, `.is_some_and`, `.map`, `.take`, `.to_string`, `Resolved::default`, `Some`
- **Called by:** `part_two` (markdown/from_docx/header.rs:1286); `tests::the_legend_names_an_indent_contextual_spacing_and_keep_lines` (markdown/from_docx/header.rs:2255); `tests::a_style_s_universal_measures_read_as_their_integers` (markdown/from_docx/header.rs:2299)

#### `points`  (L835-838)
`fn points(twips: f64) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.to_string`, `.trim_end_matches`
- **Macros:** `format!`
- **Called by:** `describe` (markdown/from_docx/header.rs:854)

#### `align_word`  (L840-847)
`fn align_word(align: Option<&str>) -> &'static str`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `describe` (markdown/from_docx/header.rs:854)

#### `describe`  (L854-937)
`fn describe(r: &Resolved, base: Option<&Resolved>) -> String`

- **Does:** The default style's line prints everything; a heading line always prints its font and size, then only what differs from `base` (the default style), so the legend reads as deviations.
- **Calls (crate):** `points` (markdown/from_docx/header.rs:835) L881,886,898; `inches` (markdown/from_docx/agent.rs:1206) L912,918,924; `align_word` (markdown/from_docx/header.rs:840) L930,931,932
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.filter`, `.is_none_or`, `.is_some`, `.join`, `.push_str`, `.push`, `.to_string`, `.trim_end_matches`, `.unwrap_or_else`, `.unwrap_or`, `Vec::new`, `differs`, `pick`
- **Macros:** `format!`
- **Called by:** `part_two` (markdown/from_docx/header.rs:1286); `tests::the_legend_names_an_indent_contextual_spacing_and_keep_lines` (markdown/from_docx/header.rs:2255); `tests::a_style_s_universal_measures_read_as_their_integers` (markdown/from_docx/header.rs:2299)

#### `named`  (L941-949)
`fn named(styles: Option<&Element>, id: &str, expected: &str) -> String`

- **Does:** `, named Text Heading 2`: the name of style `id` when it is not the one the writer would give it (`expected`).
- **Calls (crate):** `style_by_id` (markdown/from_docx/header.rs:713) L943; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:315); `Scope::child` (xmllinq/serialize.rs:225)) L944; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L945
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.map`, `.unwrap_or_default`
- **Macros:** `format!`
- **Called by:** `holds_watermark` (edit/watermark.rs:156) (?); `classify` (edit/whole.rs:396) (?); `part_two` (markdown/from_docx/header.rs:1286); `story` (markdown/xml.rs:587) (?)

#### `quoted_name`  (L953-959)
`fn quoted_name(name: &str) -> String`

- **Does:** A legend `named` value: quoted (a JSON string) when it holds a comma or a quote, so the clause's `, ` separator cannot split it (#549).
- **Calls (std/external/unresolved):** `.contains`, `.to_string`, `.unwrap_or_else`, `serde_json::to_string`
- **Macros:** `format!`
- **Called by:** `table_style_line` (markdown/from_docx/header.rs:961)

#### `table_style_line`  (L961-1017)
`fn table_style_line(styles: Option<&Element>, id: &str) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `style_by_id` (markdown/from_docx/header.rs:713) L965,976; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:315); `Scope::child` (xmllinq/serialize.rs:225)) L966,993; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L967,995,996; `word_table_style_name` (markdown/package.rs:676) L970; `quoted_name` (markdown/from_docx/header.rs:953) L973; `Element::path` (markdown/from_docx/ooxml.rs:320) (by name) L977
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.as_ref`, `.clone`, `.cloned`, `.collect`, `.filter`, `.find_map`, `.first`, `.flatten`, `.iter`, `.map`, `.parse`, `.to_string`, `.trim_end_matches`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `Some`
- **Macros:** `format!`
- **Called by:** `part_two` (markdown/from_docx/header.rs:1286); `tests::a_table_style_s_start_end_borders_read_as_left_right` (markdown/from_docx/header.rs:2057)

#### `story_style`  (L1021-1023, pub(crate))
`pub(crate) fn story_style(kind: &str) -> &'static str`

- **Does:** The style the writer gives a `header` or `footer` paragraph the view names no style for: Word's own.
- **Calls:** none
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

#### `by_paragraph`  (L1027-1031)
`fn by_paragraph(s: &StoryFact) -> bool`

- **Does:** A story prints paragraph by paragraph (`p0: {text: …, color: …}`) when one carries its own colour or borders, or their alignments differ.
- **Calls (std/external/unresolved):** `.any`, `.is_some`, `.iter`
- **Called by:** `story_kv` (markdown/from_docx/header.rs:1080); `entry` (markdown/from_docx/header.rs:1098)

#### `paragraph_entries`  (L1035-1054)
`fn paragraph_entries(s: &StoryFact) -> Vec<String>`

- **Does:** `p0: {text: …, center, color: "595959", border: bottom single 0.5pt}`, one per paragraph.
- **Calls (crate):** `scalar` (markdown/from_docx/header.rs:174) L1041,1046,1049
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.enumerate`, `.map`, `.push_str`, `.zip`, `std::iter::once`
- **Macros:** `format!`
- **Called by:** `story_kv` (markdown/from_docx/header.rs:1080); `entry` (markdown/from_docx/header.rs:1098)

#### `story_tail`  (L1057-1076)
`fn story_tail(s: &StoryFact) -> Vec<String>`

- **Does:** A story's entries after its text: its styles, `inactive`, its changes.
- **Calls (crate):** `scalar` (markdown/from_docx/header.rs:174) L1061,1063,1072
- **Calls (std/external/unresolved):** `.as_slice`, `.collect`, `.is_empty`, `.iter`, `.join`, `.map`, `.push`, `.to_string`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `story_kv` (markdown/from_docx/header.rs:1080); `entry` (markdown/from_docx/header.rs:1098)

#### `story_kv`  (L1080-1096)
`fn story_kv(out: &mut String, key: &str, s: &StoryFact, comment: Option<&str>)`

- **Does:** A story's entry in the first section's block: one line, or, paragraph by paragraph, `key:` then its entries indented below.
- **Calls (crate):** `by_paragraph` (markdown/from_docx/header.rs:1027) L1081; `kv` (markdown/from_docx/header.rs:157) L1082,1085; `entry` (markdown/from_docx/header.rs:1098) L1082; `scalar` (markdown/from_docx/header.rs:174) L1086; `paragraph_entries` (markdown/from_docx/header.rs:1035) L1087; `story_tail` (markdown/from_docx/header.rs:1057) L1087
- **Calls (std/external/unresolved):** `.chain`, `.into_iter`, `.push_str`, `.to_string`
- **Macros:** `format!`
- **Called by:** `first_section_stories` (markdown/from_docx/header.rs:1126)

#### `entry`  (L1098-1123)
`fn entry(s: &StoryFact) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `by_paragraph` (markdown/from_docx/header.rs:1027) L1099; `scalar` (markdown/from_docx/header.rs:174) L1100,1107,1111,1115; `paragraph_entries` (markdown/from_docx/header.rs:1035) L1101; `story_tail` (markdown/from_docx/header.rs:1057) L1102,1119
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.is_empty`, `.join`, `.map`, `.push_str`, `std::iter::once`
- **Macros:** `format!`
- **Called by:** `story_kv` (markdown/from_docx/header.rs:1080); `section_entry` (markdown/from_docx/header.rs:1184)

#### `first_section_stories`  (L1126-1180)
`fn first_section_stories(out: &mut String, section: &SectionFact)`

- **Does:** The `headers:` / `footers:` blocks of the first section.
- **Calls (crate):** `find` (ambiguous: `find` (markdown/pages.rs:451); `find` (markdown/unified.rs:464)) L1135,1154,1163,1167; `story_kv` (markdown/from_docx/header.rs:1080) L1136,1156,1164,1168; `kv` (markdown/from_docx/header.rs:157) L1142,1165
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.find`, `.is_empty`, `.iter`, `.push_str`, `Some`
- **Macros:** `format!`
- **Called by:** `part_two` (markdown/from_docx/header.rs:1286)

#### `section_entry`  (L1184-1229)
`fn section_entry( section: &SectionFact, previous: &SectionFact, first: &SectionFact, handles: Option<&Handles>, ) -> String`

- **Does:** `{p2-p3, page: …, columns 2, headers: {default: {…}}}`: what a later section changes against the one before it (page setup against the first).
- **Calls (crate):** `section_changes` (markdown/from_docx/header.rs:1233) L1197; `scalar` (markdown/from_docx/header.rs:174) L1198; `entry` (markdown/from_docx/header.rs:1098) L1214
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_ref`, `.collect`, `.filter`, `.insert`, `.is_empty`, `.iter`, `.join`, `.map`, `.push`, `.to_string`, `setup`
- **Macros:** `format!`, `vec!`
- **Called by:** `part_two` (markdown/from_docx/header.rs:1286)

#### `section_changes`  (L1233-1240)
`fn section_changes(section: &SectionFact, handles: &Handles) -> Option<String>`

- **Does:** The tracked view's tags for a section's own changes (`w:sectPrChange`): `r9 @AC`.
- **Calls (crate):** `Element::children_named` (markdown/from_docx/ooxml.rs:308) (by name) L1235; `tag_of` (markdown/from_docx/agent.rs:170) L1237; `format_tags` (markdown/from_docx/agent.rs:320) L1239
- **Calls (std/external/unresolved):** `.as_ref`, `.collect`, `.is_empty`, `.map`, `.then`
- **Called by:** `section_entry` (markdown/from_docx/header.rs:1184); `part_two` (markdown/from_docx/header.rs:1286)

#### `style_changes`  (L1245-1270)
`fn style_changes(styles: &Element, handles: &Handles) -> Vec<(String, String)>`

- **Does:** `Normal: styles.r0 @AS, Heading1: styles.r3 @AS`: the styles part's tracked changes by the style that holds them (`docDefaults` for the document defaults), in the tracked view.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L1247; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L1248; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L1250; `collect` (markdown/from_docx/header.rs:1273) L1257; `tag_of` (markdown/from_docx/agent.rs:170) L1260; `format_tags` (markdown/from_docx/agent.rs:320) L1266
- **Calls (std/external/unresolved):** `.contains`, `.is_empty`, `.push`, `.to_string`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `part_two` (markdown/from_docx/header.rs:1286)

#### `collect`  (L1273-1284)
`fn collect<'e>(e: &'e Element, out: &mut Vec<&'e Element>)`

- **Does:** The elements under `e` that record a tracked change.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L1274; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L1275; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L1277; `collect` (markdown/from_docx/header.rs:1273) L1282
- **Calls (std/external/unresolved):** `.ends_with`, `.is_some`, `.push`
- **Macros:** `matches!`
- **Called by:** `get_changed_property_names` (comparer/formatchg.rs:217) (?); `style_changes` (markdown/from_docx/header.rs:1245); `collect` (markdown/from_docx/header.rs:1273)

#### `part_two`  (L1286-1517)
`fn part_two(out: &mut String, f: &Facts)`

- **Does:** _no doc comment_
- **Calls (crate):** `page_setup` (markdown/from_docx/header.rs:570) L1290; `section_changes` (markdown/from_docx/header.rs:1233) L1292; `kv` (markdown/from_docx/header.rs:157) L1296,1313,1335,1353,1378,1397...; `resolve` (markdown/from_docx/header.rs:800) L1300,1343,1361,1386; `style_changes` (markdown/from_docx/header.rs:1245) L1308; `scalar` (markdown/from_docx/header.rs:174) L1317,1318,1340,1357,1358,1382...; `describe` (markdown/from_docx/header.rs:854) L1321,1343,1361,1386; `named` (markdown/from_docx/header.rs:941) L1322,1344,1362,1387; `table_style_line` (markdown/from_docx/header.rs:961) L1401,1412; `first_section_stories` (markdown/from_docx/header.rs:1126) L1496; `section_entry` (markdown/from_docx/header.rs:1184) L1505
- **Calls (std/external/unresolved):** `.any`, `.as_ref`, `.as_slice`, `.as_str`, `.clone`, `.collect`, `.enumerate`, `.filter`, `.first`, `.into_iter`, `.is_empty`, `.is_none`, `.is_ok`, `.iter`, `.join`, `.len`, `.map_or_else`, `.map_or`, `.map`, `.or_else`, `.parse`, `.position`, `.push_str`, `.push`, `.repeat`, `.skip`, `.sort_by_key`, `.strip_prefix`, `.then_some`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `Some`, `Vec::new`, `order`
- **Macros:** `format!`
- **Called by:** `render` (markdown/from_docx/header.rs:306)

#### `story_paragraphs`  (L1522-1524, pub(crate))
`pub(crate) fn story_paragraphs(root: &Element) -> Vec<&Element>`

- **Does:** A header/footer part's paragraphs in document order, numbered as `edit` numbers a story (`inspect::story_paragraph_nodes`): every `w:p` outside text boxes, table cells and content controls included.
- **Calls (crate):** `paragraphs` (markdown/from_docx/agent.rs:1444) L1523
- **Called by:** `tests::story_paragraphs_agree_with_the_paragraphs_edit_resolves` (markdown/from_docx/header.rs:1957); `tests::a_story_field_prints_its_whole_code_and_its_result` (markdown/from_docx/header.rs:2114); `tests::a_story_field_without_a_result_prints_its_bare_code` (markdown/from_docx/header.rs:2130); `tests::a_page_field_in_a_block_content_control_is_found` (markdown/from_docx/header.rs:2146); `tests::runs_inside_inline_wrappers_count_and_deletions_do_not` (markdown/from_docx/header.rs:2163); `tests::paragraphs_number_as_edit_does_tables_in_and_text_boxes_out` (markdown/from_docx/header.rs:2177); `tests::a_break_in_a_header_line_is_a_newline` (markdown/from_docx/header.rs:2223); `tests::a_nested_field_keeps_the_outer_code` (markdown/from_docx/header.rs:2239); `convert` (markdown/from_docx/mod.rs:96)

#### `story_text_with`  (L1559-1908, pub(crate))
`pub(crate) fn story_text_with( p: &Element, view: StoryView, draw: &mut dyn FnMut(&Element, bool) -> Option<String>, ) -> String`

- **Does:** [`story_text`], calling `draw` for each `w:drawing`, `w:pict`, `w:object` and drawing-holding `mc:AlternateContent` element: its Markdown (`None` prints nothing).
- **Calls (crate):** `story_text_with::{fn walk}` (markdown/from_docx/header.rs:1638) L1906
- **Calls (std/external/unresolved):** `.to_string`, `.trim`, `String::new`, `Vec::new`
- **Called by:** `story_text` (markdown/from_docx/header.rs:1550); `convert` (markdown/from_docx/mod.rs:96)

#### `story_text_with::{fn field_end}`  (L1610-1621)
`fn field_end(out: &str, at: usize, instr: &str, nested: bool) -> &'static str`

- **Does:** A printed field's end: `{/}`, or nothing when it has no result and its code is a bare name, which the writer reads as `{PAGE}`.
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.is_ascii_digit`, `.is_ascii_uppercase`, `.is_empty`, `.len`, `.trim`
- **Called by:** `story_text_with::{fn walk}` (markdown/from_docx/header.rs:1638); `story_text_with::{fn run_content}` (markdown/from_docx/header.rs:1736)

#### `story_text_with::{fn inserting}`  (L1623-1629)
`fn inserting(local: &str) -> Option<bool>`

- **Does:** `ins`/`moveTo` → `Some(true)`, `del`/`moveFrom` → `Some(false)`.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `story_text_with::{fn walk}` (markdown/from_docx/header.rs:1638)

#### `story_text_with::{fn code}`  (L1630-1637)
`fn code(instr: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.next`, `.split_whitespace`, `.to_uppercase`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `tests::a_code_block_breaks_only_the_delimiter_shapes_it_holds` (markdown/from_docx/critic.rs:2463) (?); `story_text_with::{fn walk}` (markdown/from_docx/header.rs:1638); `story_text_with::{fn run_content}` (markdown/from_docx/header.rs:1736)

#### `story_text_with::{fn walk}`  (L1638-1733)
`fn walk( parent: &Element, s: &mut State, draw: &mut dyn FnMut(&Element, bool) -> Option<String>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L1643; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L1644; `capturable_alternate` (markdown/from_docx/drawing.rs:1290) L1650; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:315); `Scope::child` (xmllinq/serialize.rs:225)) L1656,1713,1723; `story_text_with::{fn walk}` (markdown/from_docx/header.rs:1638) L1657,1668,1673,1694,1711,1714; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L1661,1705; `printed_code` (markdown/from_docx/mod.rs:2175) L1662; `story_text_with::{fn field_end}` (markdown/from_docx/header.rs:1610) L1670; `story_text_with::{fn code}` (markdown/from_docx/header.rs:1630) L1672; `story_text_with::{fn inserting}` (markdown/from_docx/header.rs:1623) L1676,1677; `tag_of` (markdown/from_docx/agent.rs:170) L1699; `State::change` (markdown/from_docx/header.rs:1820) L1700; `hidden_run` (markdown/from_docx/agent.rs:133) L1723; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L1726; `story_text_with::{fn run_content}` (markdown/from_docx/header.rs:1736) L1727
- **Calls (std/external/unresolved):** `.filter`, `.is_empty`, `.is_some`, `.len`, `.map`, `.or_else`, `.push_str`, `.split_off`, `.unwrap_or`, `Some`, `draw`, `std::mem::replace`
- **Macros:** `format!`, `vec!`
- **Called by:** `story_text_with` (markdown/from_docx/header.rs:1559); `story_text_with::{fn walk}` (markdown/from_docx/header.rs:1638)

#### `story_text_with::{fn run_content}`  (L1736-1815)
`fn run_content( child: &Element, s: &mut State, draw: &mut dyn FnMut(&Element, bool) -> Option<String>, )`

- **Does:** One child of a `w:r` (or of a run-level `mc:Choice`): text, field characters, breaks, and drawings `draw` renders.
- **Calls (crate):** `Element::local` (markdown/from_docx/ooxml.rs:277) L1741,1761; `holds_drawing` (markdown/from_docx/drawing.rs:1257) L1747; `Element::child` (markdown/from_docx/ooxml.rs:315) L1753; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L1754; `story_text_with::{fn run_content}` (markdown/from_docx/header.rs:1736) L1755; `Element::attr` (markdown/from_docx/ooxml.rs:286) L1762; `printed_code` (markdown/from_docx/mod.rs:2175) L1775; `story_text_with::{fn field_end}` (markdown/from_docx/header.rs:1610) L1788; `story_text_with::{fn code}` (markdown/from_docx/header.rs:1630) L1791; `Element::text` (markdown/from_docx/ooxml.rs:344) L1799,1806,1807; `State::push_text` (markdown/from_docx/header.rs:1875) L1806,1807
- **Calls (std/external/unresolved):** `.all`, `.filter`, `.is_empty`, `.iter`, `.last_mut`, `.len`, `.or_else`, `.pop`, `.push_str`, `.push`, `.to_string`, `Open::default`, `draw`
- **Called by:** `story_text_with::{fn walk}` (markdown/from_docx/header.rs:1638); `story_text_with::{fn run_content}` (markdown/from_docx/header.rs:1736)

#### `State::change`  (L1820-1870)
`fn change(&mut self, inserted: bool, side: Side)`

- **Does:** A tagged change, as the body prints its neighbours: marks of one kind by one author join, and an insertion directly followed by a deletion is one substitution.
- **Calls (crate):** `Shown::alone` (markdown/from_docx/header.rs:1884) L1841,1844; `join_tags` (markdown/from_docx/agent.rs:245) L1847,1853; `format_tag` (markdown/from_docx/agent.rs:273) L1853,1858,1863; `tag` (markdown/from_docx/critic.rs:74) L1853,1858,1863
- **Calls (std/external/unresolved):** `.as_mut`, `.as_ref`, `.expect`, `.extend`, `.filter`, `.is_none`, `.is_some_and`, `.is_some`, `.len`, `.push_str`, `.take`, `.truncate`, `Some`, `String::new`, `joins`
- **Macros:** `format!`
- **Called by:** `boundary_coverage_tests::empty_property_blocks_and_change_records_are_distinct` (debug/diff.rs:2385) (?); `story_text_with::{fn walk}` (markdown/from_docx/header.rs:1638)

#### `State::push_text`  (L1875-1881)
`fn push_text(&mut self, text: &str)`

- **Does:** Document text; the tagged view escapes what reads like a mark, and every view text that reads like a field code (`\{PAGE}`), as the body does.
- **Calls (crate):** `escape_field_codes` (markdown/from_docx/header.rs:1914) L1876; `escape` (markdown/from_docx/critic.rs:1009) L1878
- **Calls (std/external/unresolved):** `.push_str`
- **Called by:** `story_text_with::{fn run_content}` (markdown/from_docx/header.rs:1736)

#### `Shown::alone`  (L1884-1896)
`fn alone(start: usize, inserted: bool, side: Side) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `State::change` (markdown/from_docx/header.rs:1820)

#### `escape_field_codes`  (L1914-1939)
`fn escape_field_codes(text: &str) -> std::borrow::Cow<'_, str>`

- **Does:** `text` with a backslash before each `{` the writer would read as a field: one that opens what reads like a field code (`{PAGE \* roman}`), a bare name (`{SELLER}`, `{1999}`) or a result's end (`{/}`), so text typed in a header stays text.
- **Calls (crate):** `field_code_close` (markdown/critic.rs:1442) L1925; `is_field_code` (markdown/critic.rs:1426) L1926
- **Calls (std/external/unresolved):** `.all`, `.any`, `.bytes`, `.char_indices`, `.find`, `.is_ascii_digit`, `.is_ascii_uppercase`, `.is_some_and`, `.len`, `.push`, `.starts_with`, `String::with_capacity`, `opens`, `std::borrow::Cow::Borrowed`, `std::borrow::Cow::Owned`
- **Called by:** `State::push_text` (markdown/from_docx/header.rs:1875)

## Test functions (19)

- `story_text` (L1550): A header/footer paragraph's text with fields as the body prints them, `{CODE}result{/}` (`{PAGE}1{/}`, `{HYPERLINK "https://…"}site{/}`), and a code jubarte does not print whole as its name alone (`{XE}`), as is a code holding a nested field (`{IF}`).
- `tests::ftr` (L1948): _no doc comment_
- `tests::story_paragraphs_agree_with_the_paragraphs_edit_resolves` (L1957): Review t09-15: header and footer ids must number paragraphs exactly as `edit` resolves them, so the two walks are checked against each other.
- `tests::a_section_s_universal_measures_read_as_twips` (L1971): _no doc comment_
- `tests::an_mm_spelled_page_is_the_page_its_twips_twin_names` (L2000): An mm-spelled page is the page its twips twin names: 210mm x 297mm is A4 once rounded to the twips it is (11905.5 → 11906), not the `8.268x11.693in` a truncating cast reads from any producer whose spelling is not Word's own `595.3pt` (#531 review defect 6).
- `tests::word_s_paper_list_names_its_sizes` (L2018): Word's paper list: every size Word names and the writer writes back exactly is named by the rounded twips Word spells it in — the short side first, whatever way the sheet lies (#561 review defect 4).
- `tests::a_table_style_s_start_end_borders_read_as_left_right` (L2057): A strict table style names its side borders `start`/`end`: the legend reads them as the `left`/`right` they name, and one side a twip apart still varies.
- `tests::a_long_value_aligns_by_characters_not_bytes` (L2085): _no doc comment_
- `tests::an_empty_comment_leaves_no_hash` (L2107): _no doc comment_
- `tests::a_story_field_prints_its_whole_code_and_its_result` (L2114): _no doc comment_
- `tests::a_story_field_without_a_result_prints_its_bare_code` (L2130): _no doc comment_
- `tests::a_page_field_in_a_block_content_control_is_found` (L2146): _no doc comment_
- `tests::runs_inside_inline_wrappers_count_and_deletions_do_not` (L2163): _no doc comment_
- `tests::paragraphs_number_as_edit_does_tables_in_and_text_boxes_out` (L2177): _no doc comment_
- `tests::number_and_time_like_values_are_quoted` (L2199): pi review r392 F1/F6: values a YAML 1.1 or 1.2 reader takes as a number, a time or a date are quoted; prose that starts with a digit stays plain.
- `tests::a_break_in_a_header_line_is_a_newline` (L2223): pi review av2 F18: a line break inside a header paragraph is a line break in its text, not two words glued together.
- `tests::a_nested_field_keeps_the_outer_code` (L2239): pi review av2 F18: a field nested in another's instruction does not replace it; the outer field names the code, its result stays hidden.
- `tests::the_legend_names_an_indent_contextual_spacing_and_keep_lines` (L2255): A paragraph indent, contextual spacing and keep-lines are named; a style with none of them keeps today's line.
- `tests::a_style_s_universal_measures_read_as_their_integers` (L2299): A strict style carries its measures as universal measures (`11pt`, `1in`) and its font size in half-points either way: the legend reads the same as the transitional integers.
