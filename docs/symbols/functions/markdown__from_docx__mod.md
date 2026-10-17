# Functions and methods in `src/markdown/from_docx/mod.rs`

123 non-test, 85 test. Part of the [symbol manifest](../README.md).

#### `<ConvertError as std::fmt::Display>::fmt`  (L71-75, impl std::fmt::Display)
`fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.write_str`
- **Called by:** none found in src

#### `part_stem`  (L92-94)
`fn part_stem(path: &str) -> String`

- **Does:** A part's file stem: `word/footnotes.xml` → `footnotes`, the story `jubarte changes` names its changes by.
- **Calls (crate):** `part_story` (ids.rs:392) L93
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

#### `convert`  (L96-1282, pub(crate))
`pub(crate) fn convert(bytes: &[u8], options: &Options) -> Result<Converted, ConvertError>`

- **Does:** _no doc comment_
- **Calls (crate):** `Package::open` (ambiguous: `Package::open` (debug.rs:171); `Package::open` (markdown/from_docx/ooxml.rs:38)) L97; `Package::main_part` (markdown/from_docx/ooxml.rs:140) L98; `Package::xml` (markdown/from_docx/ooxml.rs:93) L99,113,118,136,150,156...; `invalid` (markdown/from_docx/ooxml.rs:27) L101; `Package::rels` (markdown/from_docx/ooxml.rs:103) L107; `Rels::first_of_type` (markdown/from_docx/ooxml.rs:210) (by name) L109; `threads` (markdown/from_docx/agent.rs:2121) L125; `part_stem` (markdown/from_docx/mod.rs:92) L137,152,904; `ScopeMap::of` (markdown/from_docx/drawing.rs:161) L138,409,909; `Rels::targets_of_type` (markdown/from_docx/ooxml.rs:219) (by name) L145,148; `handles` (markdown/from_docx/agent.rs:704) L170; `Element::children_named` (markdown/from_docx/ooxml.rs:308) (by name) L185,252,268,313,413,692...; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L187,205,254,255,257,269...; `bookmark_targets` (markdown/from_docx/mod.rs:4527) L191,372; `body_headings` (markdown/from_docx/mod.rs:1606) L200; `own_slug` (markdown/from_docx/mod.rs:1624) L201; `Element::find_all` (markdown/from_docx/ooxml.rs:333) (by name) L203,419,421,729,860; `is_own_slug` (markdown/from_docx/mod.rs:1633) L206; `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L221,294,297; `stamp` (markdown/from_docx/agent.rs:509) L224; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:315); `Scope::child` (xmllinq/serialize.rs:225)) L235,366,428,679,717,855...; `page_counts` (markdown/from_docx/agent.rs:1659) L236,681; `page_sections` (markdown/from_docx/agent.rs:1463) L239; `resolve` (markdown/from_docx/revise.rs:20) L247,271,286; `Numbering::parse` (markdown/from_docx/mod.rs:1970) L264; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L293,881; `paragraphs` (markdown/from_docx/agent.rs:1444) L332,481,791; `Media::load` (markdown/from_docx/ooxml.rs:1052) L344; `table_styles` (markdown/from_docx/mod.rs:4442) L365,852; `custom_table_grid` (markdown/from_docx/mod.rs:4418) L367,371,852; `Blocks::new` (markdown/from_docx/ooxml.rs:1125) L429; `Writer::blocks` (markdown/from_docx/mod.rs:2392) L431; `Writer::flush_empty` (markdown/from_docx/mod.rs:2769) L432; `Writer::take_notes` (markdown/from_docx/mod.rs:2385) L434; `Blocks::push` (markdown/from_docx/ooxml.rs:1137) L435; `text_follows_reference` (markdown/from_docx/mod.rs:1566) L464; `Element::path` (markdown/from_docx/ooxml.rs:320) (by name) L494,732,795,815,865,914...; `Critic::into_inline` (markdown/from_docx/critic.rs:443) (by name) L535; `Writer::paragraph_inline` (markdown/from_docx/mod.rs:2964) L535; `.render` (ambiguous, 4 candidates) L539; `Writer::note_mark` (markdown/from_docx/mod.rs:3749) L552; `paragraph_mark_kind` (markdown/from_docx/mod.rs:1551) L556; `Inline::edges` (markdown/from_docx/ooxml.rs:574) (by name) L559; `join_paragraphs` (markdown/from_docx/critic.rs:1413) L573; `join_marked` (markdown/from_docx/critic.rs:1407) L575; `story_unshown` (markdown/from_docx/agent.rs:1081) L590,1059,1148; `Blocks::finish` (markdown/from_docx/ooxml.rs:1206) L627; `resolve_author` (markdown/from_docx/mod.rs:1287) L639; `revised_by` (markdown/from_docx/agent.rs:587) L644; `coverage_tests::revised` (comparer/finalize.rs:15520) (by name) L649,654; `.of` (ambiguous, 4 candidates) L664; `select_blocks` (markdown/from_docx/agent.rs:2421) L666; `collect_revisions` (markdown/from_docx/agent.rs:2176) L680; `without_layout` (markdown/from_docx/mod.rs:1324) L695; `named` (ambiguous: `coverage_round_three_tests::coverage_round_five_tests::named` (convert/mod.rs:50359); `named` (markdown/write.rs:5387)) L697; `comment_anchors` (markdown/from_docx/agent.rs:605) L706,708; `page_texts` (markdown/mod.rs:917) L770; `Styles::heading_level` (markdown/from_docx/mod.rs:1825) L798; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L882; `story_paragraphs` (markdown/from_docx/header.rs:1522) L908; `story_color` (markdown/from_docx/mod.rs:4599) L937; `own_borders` (markdown/from_docx/agent.rs:1701) L938; `describe` (markdown/borders.rs:209) L944; `story_text_with` (markdown/from_docx/header.rs:1559) L952; `Pending::push` (markdown/from_docx/drawing.rs:211) L953; `ScopeMap::scope` (markdown/from_docx/drawing.rs:187) L957; `story_style` (markdown/from_docx/header.rs:1021) L1018,1045; `story_paragraph_marks` (markdown/from_docx/agent.rs:1143) L1057; `short_paragraph` (ids.rs:435) L1073,1088,1089; `source_sha256` (inspect.rs:271) L1185; `list_changes_tolerant` (changes.rs:294) L1192; `.text` (ambiguous, 10 candidates) L1200; `render` (markdown/from_docx/header.rs:306) L1217,1235; `printed_change_ids` (markdown/from_docx/agent.rs:901) L1223; `unplaced_entry` (markdown/from_docx/agent.rs:1047) L1231; `note_label` (markdown/from_docx/agent.rs:2610) L1246,1251; `style_token` (markdown/view_notes.rs:1185) L1256; `content_types` (markdown/from_docx/drawing.rs:665) L1273; `fill` (markdown/from_docx/drawing.rs:703) L1274; `Media::into_files` (markdown/from_docx/ooxml.rs:1106) L1280
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.as_deref`, `.as_ref`, `.as_str`, `.chain`, `.checked_sub`, `.clear`, `.clone`, `.cloned`, `.collect`, `.contains`, `.dedup`, `.entry`, `.enumerate`, `.eq_ignore_ascii_case`, `.extend`, `.filter_map`, `.filter`, `.find_map`, `.find`, `.first`, `.flatten`, `.get`, `.hidden`, `.insert`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some_and`, `.is_some`, `.iter_mut`, `.iter`, `.join`, `.last`, `.len`, `.map_err`, `.map_or_else`, `.map_or`, `.map`, `.max_by_key`, `.max`, `.min`, `.ok_or_else`, `.ok`, `.or_default`, `.or_else`, `.or`, `.parse`, `.pop`, `.position`, `.push_str`, `.push`, `.remove`, `.replace`, `.rposition`, `.saturating_sub`, `.sort`, `.starts_with`, `.take`, `.then`, `.to_string`, `.to_vec`, `.trim_end`, `.trim_start_matches`, `.trim`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `.values`, `.zip`, `HashMap::new`, `HashSet::new`, `ListIndent::default`, `Ok`, `Some`, `String::new`, `Vec::new`, `agent::Handles::default`, `crate::convert::RevisionStyle::default`, `crate::markdown::paginate`, `header::Owner::Creator`, `header::Owner::LastModifiedBy`, `header::StoryView::Accept`, `header::StoryView::Reject`, `header::StoryView::Tagged`, `part`, `std::collections::BTreeMap::new`, `std::iter::once`, `std::mem::replace`, `style_name`, `super::write::Slugger::default`
- **Macros:** `format!`, `matches!`, `vec!`
- **Called by:** `tests::md` (markdown/from_docx/mod.rs:4685); `tests::resolved` (markdown/from_docx/mod.rs:6245); `tests::malformed_input_is_invalid_not_panic` (markdown/from_docx/mod.rs:6438); `markdown_source_owner_boundary_tests::style_title_outline_and_long_ancestry_follow_authored_nearest_properties` (markdown/from_docx/mod.rs:6469); `markdown_source_owner_boundary_tests::hidden_runs_cannot_leak_text_symbols_note_links_or_comment_markers` (markdown/from_docx/mod.rs:6506); `markdown_source_owner_boundary_tests::numbering_zero_and_out_of_range_overrides_do_not_change_source_order` (markdown/from_docx/mod.rs:6534); `docx_to_markdown` (markdown/mod.rs:641)

#### `resolve_author`  (L1287-1297)
`fn resolve_author(by: &str, handles: &agent::Handles) -> String`

- **Does:** `--by`'s author as `@HH` when it names a known author: a full name as stored, or a handle with or without its `@`.
- **Calls (std/external/unresolved):** `.find`, `.get`, `.map_or_else`, `.or_else`, `.strip_prefix`, `.to_string`, `.unwrap_or`, `.values`
- **Macros:** `format!`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

#### `creator_created`  (L1302-1319, pub(crate))
`pub(crate) fn creator_created(docx: &[u8]) -> (Option<String>, Option<String>)`

- **Does:** A `.docx`'s first non-empty `dc:creator` (its spaces kept, not the XML's indentation) and its `dcterms:created` when that is a timestamp; `None` for each the package does not tell.
- **Calls (crate):** `Package::open` (ambiguous: `Package::open` (debug.rs:171); `Package::open` (markdown/from_docx/ooxml.rs:38)) L1303; `.xml` (ambiguous: `Border::xml` (markdown/borders.rs:158); `Package::xml` (markdown/from_docx/ooxml.rs:93); `XNamespace::xml` (xmllinq/mod.rs:128)) L1305; `Element::children_named` (markdown/from_docx/ooxml.rs:308) (by name) L1309,1314; `without_layout` (markdown/from_docx/mod.rs:1324) L1313; `parse_datetime` (util/datetime.rs:11) L1317
- **Calls (std/external/unresolved):** `.and_then`, `.find_map`, `.find`, `.flatten`, `.is_empty`, `.map`, `.ok`, `.to_string`, `.trim`
- **Called by:** `docx_stamp` (markdown/mod.rs:572)

#### `without_layout`  (L1324-1338)
`fn without_layout(text: &str) -> &str`

- **Does:** `text` without an edge of whitespace that holds a line break: the indentation of pretty-printed XML (`Un-named\n `).
- **Calls (crate):** `breaks` (edit/rewrite.rs:62) (by name) L1327,1333
- **Calls (std/external/unresolved):** `.contains`, `.len`, `.trim_end`, `.trim_start`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96); `creator_created` (markdown/from_docx/mod.rs:1302); `tests::a_name_loses_the_xml_indentation_and_keeps_its_spaces` (markdown/from_docx/mod.rs:4651)

#### `fenced`  (L1342-1350)
`fn fenced(code: &str) -> String`

- **Does:** A code block's text in a fence of backticks, longer than any run of backticks that opens one of its lines.
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.lines`, `.map`, `.max`, `.repeat`, `.take_while`, `.trim_start`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2546)

#### `hard_breaks`  (L1356-1380)
`fn hard_breaks(text: &str) -> String`

- **Does:** A paragraph's line breaks (`w:br`) as Markdown hard breaks, which a bare newline is not: it reads as a space.
- **Calls (crate):** `unescaped_dollar` (markdown/from_docx/mod.rs:1415) L1360; `math_body` (markdown/from_docx/mod.rs:1431) L1371
- **Calls (std/external/unresolved):** `.is_empty`, `.len`, `.push_str`, `.replace`, `.starts_with`, `String::with_capacity`
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2546); `tests::a_break_inside_an_equation_stays_a_newline` (markdown/from_docx/mod.rs:4619)

#### `table_cell`  (L1389-1412)
`fn table_cell(value: &str, escaped: bool) -> String`

- **Does:** Escape a table cell for a Markdown pipe table.
- **Calls (crate):** `cell_math_kept` (markdown/from_docx/mod.rs:1466) L1391; `unescaped_dollar` (markdown/from_docx/mod.rs:1415) L1395; `escaped_cell_text` (markdown/from_docx/mod.rs:1446) L1405,1410; `one_line` (markdown/from_docx/mod.rs:1489) L1411
- **Calls (std/external/unresolved):** `.find`, `.len`, `.push_str`, `.replace`, `.starts_with`, `String::with_capacity`
- **Called by:** `markdown_table` (markdown/from_docx/mod.rs:1662); `tests::a_table_cell_keeps_its_equations_latex` (markdown/from_docx/mod.rs:4832); `tests::an_agent_view_cell_keeps_an_equations_norm_bars` (markdown/from_docx/mod.rs:4843)

#### `unescaped_dollar`  (L1415-1427)
`fn unescaped_dollar(value: &str) -> Option<usize>`

- **Does:** The byte offset of the first `$` no backslash escapes.
- **Calls (std/external/unresolved):** `.char_indices`, `.next`, `Some`
- **Called by:** `defuse` (markdown/from_docx/critic.rs:1093); `hard_breaks` (markdown/from_docx/mod.rs:1356); `table_cell` (markdown/from_docx/mod.rs:1389); `math_body` (markdown/from_docx/mod.rs:1431); `equation_split` (markdown/from_docx/ooxml.rs:973)

#### `math_body`  (L1431-1443)
`fn math_body(after_open: &str, fence: &str) -> Option<usize>`

- **Does:** The byte length of an equation's body after its opening fence, when `fence` (`$` or `$$`) closes it.
- **Calls (crate):** `unescaped_dollar` (markdown/from_docx/mod.rs:1415) L1434
- **Calls (std/external/unresolved):** `.starts_with`, `Some`
- **Called by:** `defuse` (markdown/from_docx/critic.rs:1093); `hard_breaks` (markdown/from_docx/mod.rs:1356); `equation_split` (markdown/from_docx/ooxml.rs:973)

#### `escaped_cell_text`  (L1446-1462)
`fn escaped_cell_text(value: &str) -> String`

- **Does:** Escaped cell text outside equations: escapes copied, bare `|` escaped.
- **Calls (std/external/unresolved):** `.chars`, `.len`, `.next`, `.push_str`, `.push`, `String::with_capacity`
- **Called by:** `table_cell` (markdown/from_docx/mod.rs:1389)

#### `cell_math_kept`  (L1466-1486)
`fn cell_math_kept(value: &str) -> String`

- **Does:** `table_cell` for unescaped text: backslashes doubled outside equations, kept inside them; `|` escaped everywhere, as a pipe table needs.
- **Calls (crate):** `one_line` (markdown/from_docx/mod.rs:1489) L1485
- **Calls (std/external/unresolved):** `.find`, `.len`, `.push_str`, `.replace`, `.starts_with`, `String::with_capacity`
- **Called by:** `table_cell` (markdown/from_docx/mod.rs:1389)

#### `one_line`  (L1489-1495)
`fn one_line(value: &str) -> String`

- **Does:** A cell's text on one line: Markdown table rows cannot break.
- **Calls (std/external/unresolved):** `.replace`, `.to_string`, `.trim`
- **Called by:** `table_cell` (markdown/from_docx/mod.rs:1389); `cell_math_kept` (markdown/from_docx/mod.rs:1466)

#### `majority`  (L1501-1527)
`fn majority(aligns: impl Iterator<Item = Option<&'static str>>) -> Option<&'static str>`

- **Does:** The alignment most of a column's cell paragraphs have, as its delimiter row writes it: none on a tie, for justified text, or when most have none.
- **Calls (std/external/unresolved):** `.filter`, `.find`, `.is_some`, `.iter_mut`, `.iter`, `.map`, `.max`, `.next`, `.push`, `.then_some`, `Vec::new`
- **Called by:** `Writer::column_alignments` (markdown/from_docx/mod.rs:3600)

#### `fold_tracked_breaks`  (L1536-1548)
`fn fold_tracked_breaks(parts: &mut [critic::Part])`

- **Does:** Rows as a Markdown pipe table; the first row is the header, and the delimiter row aligns each column as `aligns` says.
- **Calls (crate):** `whole_change` (markdown/from_docx/critic.rs:1369) L1542,1544
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.get`, `.len`, `.or_else`, `.take`
- **Called by:** `Writer::cell_text` (markdown/from_docx/mod.rs:3670)

#### `paragraph_mark_kind`  (L1551-1559)
`fn paragraph_mark_kind(p: &Element) -> Option<Mark>`

- **Does:** The tracked change on `p`'s own mark: `w:pPr/w:rPr/w:ins` or `w:del`.
- **Calls (crate):** `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L1552; `Element::path` (markdown/from_docx/ooxml.rs:320) L1552; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L1554
- **Calls (std/external/unresolved):** `.find_map`, `Some`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96); `Writer::cell_parts` (markdown/from_docx/mod.rs:3693)

#### `text_follows_reference`  (L1566-1602)
`fn text_follows_reference(note: &Element, changes: &[Change]) -> bool`

- **Does:** Whether `note` goes with its reference's change (`changes`, innermost last): it holds no tracked change of its own (resolving the reference resolves the note), or every run of it that holds text sits in a mark of that kind, Word's shape for a note inserted or deleted with its reference.
- **Calls (crate):** `text_follows_reference::{fn tracked}` (markdown/from_docx/mod.rs:1567) L1596; `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1572) L1600
- **Calls (std/external/unresolved):** `.find_map`, `.iter`, `.rev`, `Some`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

#### `text_follows_reference::{fn tracked}`  (L1567-1571)
`fn tracked(e: &Element) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L1568; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L1569; `text_follows_reference::{fn tracked}` (markdown/from_docx/mod.rs:1567) L1569
- **Calls (std/external/unresolved):** `.any`
- **Macros:** `matches!`
- **Called by:** `text_follows_reference` (markdown/from_docx/mod.rs:1566); `text_follows_reference::{fn tracked}` (markdown/from_docx/mod.rs:1567)

#### `text_follows_reference::{fn walk}`  (L1572-1586)
`fn walk(e: &Element, kind: &[&str], inside: bool, all: &mut bool)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L1573; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L1574,1578; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L1576; `.text` (ambiguous, 10 candidates) L1578; `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1572) L1583
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.is_empty`
- **Macros:** `matches!`
- **Called by:** `text_follows_reference` (markdown/from_docx/mod.rs:1566); `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1572); `body_headings` (markdown/from_docx/mod.rs:1606); `body_headings::{fn walk}` (markdown/from_docx/mod.rs:1607); `closing_fields` (markdown/from_docx/mod.rs:2070); `closing_fields::{fn walk}` (markdown/from_docx/mod.rs:2074); `spanning_fields` (markdown/from_docx/mod.rs:2121); `spanning_fields::{fn walk}` (markdown/from_docx/mod.rs:2125); `Writer::open_after` (markdown/from_docx/mod.rs:2848); `Writer::open_after::{fn walk}` (markdown/from_docx/mod.rs:2849); `shown_text` (markdown/from_docx/mod.rs:4460); `shown_text::{fn walk}` (markdown/from_docx/mod.rs:4461); +2 more

#### `body_headings`  (L1606-1620)
`fn body_headings<'e>(body: &'e Element, styles: &Styles) -> Vec<&'e Element>`

- **Does:** The paragraphs of `body` the view writes as blocks, in order, that are headings: not those in tables, which a cell writes as text.
- **Calls (crate):** `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1572) L1618
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

#### `body_headings::{fn walk}`  (L1607-1616)
`fn walk<'e>(e: &'e Element, styles: &Styles, out: &mut Vec<&'e Element>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L1608; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L1609; `heading_level` (markdown/from_docx/mod.rs:1642) L1610; `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1572) L1612
- **Calls (std/external/unresolved):** `.is_some`, `.push`
- **Called by:** `text_follows_reference` (markdown/from_docx/mod.rs:1566); `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1572); `body_headings` (markdown/from_docx/mod.rs:1606); `body_headings::{fn walk}` (markdown/from_docx/mod.rs:1607); `closing_fields` (markdown/from_docx/mod.rs:2070); `closing_fields::{fn walk}` (markdown/from_docx/mod.rs:2074); `spanning_fields` (markdown/from_docx/mod.rs:2121); `spanning_fields::{fn walk}` (markdown/from_docx/mod.rs:2125); `Writer::open_after` (markdown/from_docx/mod.rs:2848); `Writer::open_after::{fn walk}` (markdown/from_docx/mod.rs:2849); `shown_text` (markdown/from_docx/mod.rs:4460); `shown_text::{fn walk}` (markdown/from_docx/mod.rs:4461); +2 more

#### `own_slug`  (L1624-1628)
`fn own_slug(slugger: &mut super::write::Slugger, p: &Element) -> Option<String>`

- **Does:** The name the writer gives heading `p`'s bookmark (github-slugger's, a repeat suffixed by `slugger`); `None` for a heading of no slug.
- **Calls (crate):** `github_slug` (markdown/write.rs:5000) L1625; `shown_text` (markdown/from_docx/mod.rs:4460) L1625; `Slugger::name` (markdown/write.rs:5035) L1626
- **Calls (std/external/unresolved):** `.is_empty`, `.then_some`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96); `Writer::paragraph` (markdown/from_docx/mod.rs:2546)

#### `is_own_slug`  (L1633-1635)
`fn is_own_slug(name: &str, own: Option<&str>) -> bool`

- **Does:** Whether bookmark `name` is a heading's `own` slug: that name, or one the writer gives it (`_` before a slug of no first letter, cut to 40, suffixed `-1` when an anchor took that name).
- **Calls (crate):** `is_bookmark_of` (markdown/write.rs:4976) L1634
- **Calls (std/external/unresolved):** `.is_some_and`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96); `Writer::slug_of_heading` (markdown/from_docx/mod.rs:2469)

#### `heading_level`  (L1642-1660)
`fn heading_level(styles: &Styles, p: &Element, agent: bool) -> Option<usize>`

- **Does:** A paragraph's heading level: its own outline level, else its style's; `None` for body text.
- **Calls (crate):** `Element::child` (markdown/from_docx/ooxml.rs:315) L1643; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:315); `Scope::child` (xmllinq/serialize.rs:225)) L1645,1650; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L1646,1651; `Styles::heading_level` (markdown/from_docx/mod.rs:1825) L1647
- **Calls (std/external/unresolved):** `.and_then`, `.min`, `.ok`, `.parse`, `Some`, `from_style`
- **Called by:** `body_headings::{fn walk}` (markdown/from_docx/mod.rs:1607); `Writer::heading_of` (markdown/from_docx/mod.rs:2475)

#### `markdown_table`  (L1662-1693)
`fn markdown_table(rows: &[Vec<String>], aligns: &[Option<&str>], escaped: bool) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `table_cell` (markdown/from_docx/mod.rs:1389) L1672
- **Calls (std/external/unresolved):** `.copied`, `.enumerate`, `.flatten`, `.get`, `.is_empty`, `.iter`, `.map`, `.max`, `.push_str`, `.push`, `.unwrap_or`, `String::new`
- **Called by:** `Writer::table` (markdown/from_docx/mod.rs:3498)

#### `Styles::parse`  (L1726-1781)
`fn parse(root: &Element) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::children_named` (markdown/from_docx/ooxml.rs:308) L1728,1771; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L1729,1739,1744,1749,1753,1762...; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:315); `Scope::child` (xmllinq/serialize.rs:225)) L1732,1733,1752,1755,1758,1761; `Element::path` (markdown/from_docx/ooxml.rs:320) (by name) L1737,1742,1747; `Element::toggle` (markdown/from_docx/ooxml.rs:360) (by name) L1756,1757,1759; `Element::path` (markdown/from_docx/ooxml.rs:320) L1768
- **Calls (std/external/unresolved):** `.and_then`, `.find`, `.insert`, `.map`, `.ok`, `.parse`, `.to_ascii_lowercase`, `.to_string`, `.unwrap_or`, `HashMap::new`, `Some`
- **Called by:** `markdown_source_owner_boundary_tests::style_title_outline_and_long_ancestry_follow_authored_nearest_properties` (markdown/from_docx/mod.rs:6469)

#### `Styles::font`  (L1784-1788)
`fn font(&self, id: &str) -> Option<&str>`

- **Does:** The face a run of style `id` is set in, nearest style first.
- **Calls (crate):** `Styles::chain` (markdown/from_docx/mod.rs:1807) L1785
- **Calls (std/external/unresolved):** `.as_deref`, `.find_map`, `.into_iter`
- **Called by:** `Styles::monospace_paragraph` (markdown/from_docx/mod.rs:1792); `Writer::run` (markdown/from_docx/mod.rs:3119)

#### `Styles::monospace_paragraph`  (L1792-1797)
`fn monospace_paragraph(&self, id: Option<&str>) -> bool`

- **Does:** Whether a paragraph of style `id` (or the default paragraph style) is set in a monospace face: its monospace runs are then not code.
- **Calls (crate):** `Styles::font` (markdown/from_docx/mod.rs:1784) L1794
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.is_some_and`, `.or`
- **Called by:** `Writer::paragraph_inline` (markdown/from_docx/mod.rs:2964) (?)

#### `Styles::jc`  (L1800-1804)
`fn jc(&self, id: &str) -> Option<&str>`

- **Does:** The alignment a paragraph of style `id` has, nearest first.
- **Calls (crate):** `Styles::chain` (markdown/from_docx/mod.rs:1807) L1801
- **Calls (std/external/unresolved):** `.as_deref`, `.find_map`, `.into_iter`
- **Called by:** `Transaction::resolve_one` (edit.rs:2528) (?); `Writer::alignment` (markdown/from_docx/mod.rs:3646)

#### `Styles::chain`  (L1807-1823)
`fn chain(&self, id: &str) -> Vec<&Style>`

- **Does:** The style and its `basedOn` ancestors, nearest first.
- **Calls (std/external/unresolved):** `.clone`, `.get`, `.len`, `.push`, `.to_string`, `Some`, `Vec::new`
- **Called by:** `Styles::font` (markdown/from_docx/mod.rs:1784); `Styles::jc` (markdown/from_docx/mod.rs:1800); `Styles::heading_level` (markdown/from_docx/mod.rs:1825); `Styles::num` (markdown/from_docx/mod.rs:1852); `Styles::emphasis` (markdown/from_docx/mod.rs:1856); `Styles::hidden` (markdown/from_docx/mod.rs:1866); `Styles::underline` (markdown/from_docx/mod.rs:1875)

#### `Styles::heading_level`  (L1825-1850)
`fn heading_level(&self, id: &str) -> Option<usize>`

- **Does:** _no doc comment_
- **Calls (crate):** `Styles::chain` (markdown/from_docx/mod.rs:1807) L1826
- **Calls (std/external/unresolved):** `.and_then`, `.clamp`, `.map`, `.min`, `.ok`, `.parse`, `.strip_prefix`, `.then_some`, `.to_ascii_lowercase`, `.trim`, `Some`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96); `heading_level` (markdown/from_docx/mod.rs:1642); `markdown_source_owner_boundary_tests::style_title_outline_and_long_ancestry_follow_authored_nearest_properties` (markdown/from_docx/mod.rs:6469) (?)

#### `Styles::num`  (L1852-1854)
`fn num(&self, id: &str) -> Option<(String, usize)>`

- **Does:** _no doc comment_
- **Calls (crate):** `Styles::chain` (markdown/from_docx/mod.rs:1807) L1853
- **Calls (std/external/unresolved):** `.clone`, `.find_map`, `.into_iter`
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2546)

#### `Styles::emphasis`  (L1856-1862)
`fn emphasis(&self, id: &str) -> (Option<bool>, Option<bool>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Styles::chain` (markdown/from_docx/mod.rs:1807) L1857
- **Calls (std/external/unresolved):** `.find_map`, `.iter`
- **Called by:** `Writer::paragraph_inline` (markdown/from_docx/mod.rs:2964); `Writer::run` (markdown/from_docx/mod.rs:3119); `markdown_source_owner_boundary_tests::style_title_outline_and_long_ancestry_follow_authored_nearest_properties` (markdown/from_docx/mod.rs:6469) (?)

#### `Styles::hidden`  (L1866-1872)
`fn hidden(&self) -> HashSet<String>`

- **Does:** The character styles whose runs are hidden: the nearest `w:vanish` in the style chain is on.
- **Calls (crate):** `Styles::chain` (markdown/from_docx/mod.rs:1807) L1869
- **Calls (std/external/unresolved):** `.cloned`, `.collect`, `.filter`, `.find_map`, `.iter`, `.keys`, `Some`
- **Called by:** `Layout::push_unpainted` (convert/mod.rs:26522) (?); `residual_byte_and_geometry_contract_tests::notdef_runs_carry_their_text_as_actual_text` (convert/pdf.rs:3023) (?); `tests::hidden_text_paints_nothing_but_reads_back` (convert/raster.rs:748) (?)

#### `Styles::underline`  (L1875-1877)
`fn underline(&self, id: &str) -> Option<bool>`

- **Does:** Whether the style chain underlines, nearest style first.
- **Calls (crate):** `Styles::chain` (markdown/from_docx/mod.rs:1807) L1876
- **Calls (std/external/unresolved):** `.find_map`, `.iter`
- **Called by:** `Writer::run` (markdown/from_docx/mod.rs:3119)

#### `font_of`  (L1881-1887)
`fn font_of(rpr: &Element) -> Option<String>`

- **Does:** The face `w:rFonts` names for Latin text in `rpr`.
- **Calls (crate):** `Element::child` (markdown/from_docx/ooxml.rs:315) L1882; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L1883,1885
- **Calls (std/external/unresolved):** `.map`, `.or_else`
- **Called by:** none found in src

#### `highlight_color`  (L1893-1905)
`fn highlight_color(c: &str) -> Option<&'static str>`

- **Does:** `w:highlight w:val="…"` as one of Word's 16 named highlight colours (`ST_HighlightColor`), the static `&str` the view prints.
- **Calls (std/external/unresolved):** `.copied`, `.find`, `.is_empty`, `.iter`, `.trim`, `.unwrap_or`, `Some`
- **Called by:** none found in src

#### `is_monospace`  (L1909-1934)
`fn is_monospace(font: &str) -> bool`

- **Does:** Whether `font` is a monospace face: the common code and typewriter faces, and any whose name says so.
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.contains`, `.split`, `.starts_with`, `.to_ascii_lowercase`, `.trim`
- **Macros:** `matches!`
- **Called by:** `tests::monospace_faces_are_named_by_whole_words` (markdown/from_docx/mod.rs:4857)

#### `is_underlined`  (L1937-1939)
`fn is_underlined(u: &Element) -> bool`

- **Does:** `w:u` underlines unless its value is `none`.
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:286) L1938
- **Calls (std/external/unresolved):** `.is_none_or`
- **Called by:** none found in src

#### `num_pr`  (L1941-1952)
`fn num_pr(numpr: &Element) -> Option<(String, usize)>`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::child` (markdown/from_docx/ooxml.rs:315) L1942,1946; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L1944,1948
- **Calls (std/external/unresolved):** `.and_then`, `.ok`, `.parse`, `.to_string`, `.unwrap_or`, `Some`
- **Called by:** none found in src

#### `Numbering::parse`  (L1970-2011)
`fn parse(root: &Element) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::children_named` (markdown/from_docx/ooxml.rs:308) L1972,1985; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L1973,1978,1987,1988,1994,1999; `Element::children_named` (markdown/from_docx/ooxml.rs:308) (by name) L1977,1993; `level_of` (markdown/from_docx/mod.rs:2029) L1981,2001; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:315); `Scope::child` (xmllinq/serialize.rs:225)) L1988,1997,2001
- **Calls (std/external/unresolved):** `.and_then`, `.insert`, `.map`, `.ok`, `.or_else`, `.parse`, `.to_string`, `HashMap::new`, `Self::default`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

#### `Numbering::level`  (L2013-2026)
`fn level(&self, num_id: &str, ilvl: usize) -> Option<(&str, Level)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.cloned`, `.get`, `.into`, `.unwrap_or`, `Some`
- **Called by:** `paragraph_block` (convert/mod.rs:11573) (?); `table_block` (convert/mod.rs:13059) (?); `rendering_uncovered_contract_tests::list_level_geometry_respects_style_and_each_direct_indent_attribute_independently` (convert/mod.rs:59839) (?); `Writer::list_marker` (markdown/from_docx/mod.rs:2898) (?)

#### `level_of`  (L2029-2046)
`fn level_of(lvl: &Element) -> Level`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::child` (markdown/from_docx/ooxml.rs:315) L2031,2036,2041; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L2033,2038,2043
- **Calls (std/external/unresolved):** `.and_then`, `.map`, `.ok`, `.parse`, `.to_string`, `.unwrap_or`
- **Called by:** `Numbering::parse` (markdown/from_docx/mod.rs:1970)

#### `closing_fields`  (L2070-2114)
`fn closing_fields(p: &Element) -> HashSet<*const Element>`

- **Does:** The `fldChar begin` of each field in `p` whose `separate` and `end` are in `p` too, inside the same tracked changes as its begin: the writer reads a field within one change, never one a change boundary cuts.
- **Calls (crate):** `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1572) L2112
- **Calls (std/external/unresolved):** `HashSet::new`, `Vec::new`
- **Called by:** `Writer::paragraph_inline` (markdown/from_docx/mod.rs:2964)

#### `closing_fields::{fn walk}`  (L2074-2110)
`fn walk<'e>( e: &'e Element, changes: &mut Vec<*const Element>, open: &mut Vec<Open<'e>>, out: &mut HashSet<*const Element>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L2080; `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L2081,2084; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L2085; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L2102; `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1572) L2104,2107
- **Calls (std/external/unresolved):** `.clone`, `.insert`, `.last_mut`, `.pop`, `.push`, `std::ptr::from_ref`
- **Macros:** `matches!`
- **Called by:** `text_follows_reference` (markdown/from_docx/mod.rs:1566); `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1572); `body_headings` (markdown/from_docx/mod.rs:1606); `body_headings::{fn walk}` (markdown/from_docx/mod.rs:1607); `closing_fields` (markdown/from_docx/mod.rs:2070); `closing_fields::{fn walk}` (markdown/from_docx/mod.rs:2074); `spanning_fields` (markdown/from_docx/mod.rs:2121); `spanning_fields::{fn walk}` (markdown/from_docx/mod.rs:2125); `Writer::open_after` (markdown/from_docx/mod.rs:2848); `Writer::open_after::{fn walk}` (markdown/from_docx/mod.rs:2849); `shown_text` (markdown/from_docx/mod.rs:4460); `shown_text::{fn walk}` (markdown/from_docx/mod.rs:4461); +2 more

#### `spanning_fields`  (L2121-2171)
`fn spanning_fields(container: &Element) -> HashSet<*const Element>`

- **Does:** The `fldChar begin` of each field in `container`'s paragraphs whose `end` lies in a later paragraph, with only paragraphs between (a table of contents), its begin, separate and end outside any tracked change: such a field prints `{CODE}` at the start of its first paragraph and `{/}` in its last.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L2163; `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L2164; `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1572) L2165
- **Calls (std/external/unresolved):** `.clear`, `.enumerate`, `HashSet::new`, `Vec::new`
- **Called by:** `Writer::blocks` (markdown/from_docx/mod.rs:2392)

#### `spanning_fields::{fn walk}`  (L2125-2160)
`fn walk<'e>( e: &'e Element, paragraph: usize, changed: bool, open: &mut Vec<Open<'e>>, out: &mut HashSet<*const Element>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L2132; `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L2133,2136; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L2137; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L2156; `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1572) L2157
- **Calls (std/external/unresolved):** `.insert`, `.last_mut`, `.pop`, `.push`, `std::ptr::from_ref`
- **Macros:** `matches!`
- **Called by:** `text_follows_reference` (markdown/from_docx/mod.rs:1566); `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1572); `body_headings` (markdown/from_docx/mod.rs:1606); `body_headings::{fn walk}` (markdown/from_docx/mod.rs:1607); `closing_fields` (markdown/from_docx/mod.rs:2070); `closing_fields::{fn walk}` (markdown/from_docx/mod.rs:2074); `spanning_fields` (markdown/from_docx/mod.rs:2121); `spanning_fields::{fn walk}` (markdown/from_docx/mod.rs:2125); `Writer::open_after` (markdown/from_docx/mod.rs:2848); `Writer::open_after::{fn walk}` (markdown/from_docx/mod.rs:2849); `shown_text` (markdown/from_docx/mod.rs:4460); `shown_text::{fn walk}` (markdown/from_docx/mod.rs:4461); +2 more

#### `printed_code`  (L2175-2178)
`fn printed_code(instruction: &str) -> Option<String>`

- **Does:** The `{CODE}` the agent view prints for a field with `instruction`, when it is a field code (see [`crate::markdown::critic::is_field_code`]).
- **Calls (crate):** `is_field_code` (markdown/critic.rs:1426) L2176
- **Calls (std/external/unresolved):** `.then`, `.trim`
- **Macros:** `format!`
- **Called by:** `story_text_with::{fn walk}` (markdown/from_docx/header.rs:1638); `story_text_with::{fn run_content}` (markdown/from_docx/header.rs:1736); `Writer::inline` (markdown/from_docx/mod.rs:3021); `Writer::run_child` (markdown/from_docx/mod.rs:3232)

#### `change_of`  (L2183-2190)
`fn change_of(element: &Element, story: Option<&str>) -> Change`

- **Does:** A revision element (`w:ins`, `w:del`, `w:moveTo`, `w:moveFrom`) as a tracked change.
- **Calls (crate):** `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L2184; `attribution` (markdown/from_docx/mod.rs:2197) L2189
- **Macros:** `matches!`
- **Called by:** `Writer::change_of` (markdown/from_docx/mod.rs:3790)

#### `attribution`  (L2197-2214)
`fn attribution(element: &Element, story: Option<&str>) -> Option<String>`

- **Does:** Which change it is, and who made it when: a plain-view tag ([`critic::PLAIN`]) of the change's `w:id` in `story` and its note, `w:author` and `w:date` exactly as stored, in the same form as a comment's.
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:286) L2199,2202,2205; `comment_note` (markdown/from_docx/mod.rs:2244) L2202; `plain_tag_of` (markdown/from_docx/agent.rs:179) L2212
- **Calls (std/external/unresolved):** `.is_empty`, `.is_none`, `.map_or`, `.map`, `.trim_start_matches`, `Some`, `String::new`
- **Macros:** `format!`
- **Called by:** `collect_in` (markdown/from_docx/agent.rs:2182); `change_of` (markdown/from_docx/mod.rs:2183); `revision_marks` (markdown/from_docx/mod.rs:2218)

#### `revision_marks`  (L2218-2227)
`fn revision_marks(properties: &Element, story: Option<&str>) -> Vec<Change>`

- **Does:** Tracked changes recorded in a properties element (`trPr`, `tcPr`, a paragraph mark's `rPr`), in document order.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L2219; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L2221; `attribution` (markdown/from_docx/mod.rs:2197) L2222,2223
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `Some`
- **Called by:** `paragraph_mark` (markdown/from_docx/mod.rs:2232); `Writer::revision_marks` (markdown/from_docx/mod.rs:3800)

#### `paragraph_mark`  (L2232-2239)
`fn paragraph_mark(p: &Element, story: Option<&str>) -> Option<Change>`

- **Does:** The tracked change on the mark that ends a paragraph, that is on the break between it and the next one.
- **Calls (crate):** `Element::path` (markdown/from_docx/ooxml.rs:320) L2233; `revision_marks` (markdown/from_docx/mod.rs:2218) L2235
- **Calls (std/external/unresolved):** `.into_iter`, `.iter`, `.map`, `.nth`, `.position`, `.unwrap_or_default`, `.unwrap_or`
- **Called by:** `Writer::paragraph_mark` (markdown/from_docx/mod.rs:3739); `Writer::note_mark` (markdown/from_docx/mod.rs:3749)

#### `comment_note`  (L2244-2258)
`fn comment_note(author: Option<&str>, date: Option<&str>, text: &str) -> String`

- **Does:** A comment's `{>>…<<}` text: `Author (date): comment`.
- **Calls (crate):** `escape` (markdown/from_docx/critic.rs:1009) L2247,2250
- **Calls (std/external/unresolved):** `.filter`, `.is_empty`, `.join`, `.map`, `.push`, `.to_string`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `attribution` (markdown/from_docx/mod.rs:2197); `Writer::note` (markdown/from_docx/mod.rs:3820); `tests::comment_note_leaves_out_what_is_missing` (markdown/from_docx/mod.rs:5440)

#### `Writer::take_notes`  (L2385-2390)
`fn take_notes(&mut self) -> String`

- **Does:** The notes waiting for the next paragraph, as CriticMarkup comments: their block is prose, whatever their anchor was.
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`, `std::mem::take`
- **Macros:** `format!`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96); `Writer::blocks_in` (markdown/from_docx/mod.rs:2401)

#### `Writer::blocks`  (L2392-2399)
`fn blocks(&mut self, container: &Element, blocks: &mut Blocks, list: &mut ListIndent)`

- **Does:** _no doc comment_
- **Calls (crate):** `spanning_fields` (markdown/from_docx/mod.rs:2121) L2395; `Writer::blocks_in` (markdown/from_docx/mod.rs:2401) L2396
- **Calls (std/external/unresolved):** `.clear`, `std::mem::replace`
- **Called by:** `tests::a_page_break_stays_inside_its_paragraph_and_a_section_mark_ends_it` (legacy_doc.rs:1919) (?); `tests::table_story` (legacy_doc.rs:2144) (?); `legacy_byte_source_boundary_tests::unterminated_story_tail_and_whitespace_spans_keep_all_authored_characters` (legacy_doc.rs:2821) (?); `convert` (markdown/from_docx/mod.rs:96); `Writer::blocks_in` (markdown/from_docx/mod.rs:2401); `Writer::box_comments` (markdown/from_docx/mod.rs:3422); `Writer::drawing_text` (markdown/from_docx/mod.rs:3464)

#### `Writer::blocks_in`  (L2401-2453)
`fn blocks_in(&mut self, container: &Element, blocks: &mut Blocks, list: &mut ListIndent)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L2402; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L2403; `Writer::paragraph` (markdown/from_docx/mod.rs:2546) L2404; `ListIndent::reset` (markdown/from_docx/ooxml.rs:1221) L2406; `Writer::take_notes` (markdown/from_docx/mod.rs:2385) L2409; `Blocks::push_prefixed` (markdown/from_docx/ooxml.rs:1145) L2410; `Writer::threads_of` (markdown/from_docx/mod.rs:2832) L2412; `Writer::open_after` (markdown/from_docx/mod.rs:2848) L2412; `table_line` (markdown/from_docx/agent.rs:1812) L2414; `view_table_style` (markdown/from_docx/mod.rs:4406) L2417; `Writer::column_alignments` (markdown/from_docx/mod.rs:3600) L2419; `Writer::cell_styles` (markdown/from_docx/mod.rs:3563) L2419; `Writer::flush_empty` (markdown/from_docx/mod.rs:2769) L2426; `table_breaks` (markdown/from_docx/agent.rs:1485) L2428; `Writer::opens_page` (markdown/from_docx/mod.rs:2811) L2429; `Writer::page_lines` (markdown/from_docx/mod.rs:2783) L2430; `Blocks::push_line` (markdown/from_docx/ooxml.rs:1151) L2431; `Writer::table` (markdown/from_docx/mod.rs:3498) L2434; `Blocks::push` (markdown/from_docx/ooxml.rs:1137) L2435; `Writer::turn_pages` (markdown/from_docx/mod.rs:2820) L2436; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:315); `Scope::child` (xmllinq/serialize.rs:225)) L2441; `Writer::blocks` (markdown/from_docx/mod.rs:2392) L2442,2449; `Writer::revised` (markdown/from_docx/mod.rs:2456) L2446; `Writer::change_of` (markdown/from_docx/mod.rs:3790) L2446; `.blocks` (ambiguous: `Story::blocks` (legacy_doc.rs:1591); `Writer::blocks` (markdown/from_docx/mod.rs:2392)) L2446; `Writer::comment_range` (markdown/from_docx/mod.rs:2479) L2448
- **Calls (std/external/unresolved):** `.clear`, `.concat`, `.flatten`, `.is_none`, `.then_some`, `.then`
- **Called by:** `Writer::blocks` (markdown/from_docx/mod.rs:2392)

#### `Writer::revised`  (L2456-2460)
`fn revised(&mut self, change: Change, write: impl FnOnce(&mut Self))`

- **Does:** Writes content inside a tracked change.
- **Calls (crate):** `write` (markdown/view_stories.rs:1013) (by name) L2458
- **Calls (std/external/unresolved):** `.pop`, `.push`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2401); `Writer::inline` (markdown/from_docx/mod.rs:3021); `Writer::cell_parts` (markdown/from_docx/mod.rs:3693)

#### `Writer::slug_of_heading`  (L2469-2471)
`fn slug_of_heading(&self, name: &str) -> bool`

- **Does:** A comment range start or end.
- **Calls (crate):** `is_own_slug` (markdown/from_docx/mod.rs:1633) L2470
- **Calls (std/external/unresolved):** `.as_deref`
- **Called by:** `Writer::inline` (markdown/from_docx/mod.rs:3021)

#### `Writer::heading_of`  (L2475-2477)
`fn heading_of(&self, p: &Element) -> Option<usize>`

- **Does:** A paragraph's heading level: its own outline level, else its style's; `None` for body text.
- **Calls (crate):** `heading_level` (markdown/from_docx/mod.rs:1642) L2476
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2546)

#### `Writer::comment_range`  (L2479-2544)
`fn comment_range(&mut self, range: &Element, out: Option<&mut Critic>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:286) L2482,2503; `Element::is` (markdown/from_docx/ooxml.rs:281) L2486,2493,2510; `Writer::note` (markdown/from_docx/mod.rs:3820) L2529,2536
- **Calls (std/external/unresolved):** `.any`, `.close`, `.comment`, `.contains_key`, `.contains`, `.extend`, `.filter`, `.flatten`, `.insert`, `.is_empty`, `.iter`, `.open`, `.position`, `.push`, `.remove`, `.then`, `.to_string`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2401); `Writer::inline` (markdown/from_docx/mod.rs:3021); `Writer::cell_parts` (markdown/from_docx/mod.rs:3693)

#### `Writer::paragraph`  (L2546-2758)
`fn paragraph(&mut self, p: &Element, blocks: &mut Blocks, list: &mut ListIndent)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::child` (markdown/from_docx/ooxml.rs:315) L2547; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:315); `Scope::child` (xmllinq/serialize.rs:225)) L2549,2554,2636; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L2550,2637; `Writer::heading_of` (markdown/from_docx/mod.rs:2475) L2552; `Styles::num` (markdown/from_docx/mod.rs:1852) L2556; `Writer::open_ranges` (markdown/from_docx/mod.rs:2888) L2558; `own_slug` (markdown/from_docx/mod.rs:1624) L2559; `Writer::paragraph_inline` (markdown/from_docx/mod.rs:2964) L2560; `Critic::set_code_block` (markdown/from_docx/critic.rs:310) (by name) L2564; `Critic::is_blank` (markdown/from_docx/critic.rs:420) (by name) L2566; `Critic::into_inline` (markdown/from_docx/critic.rs:443) (by name) L2567; `Writer::threads_of` (markdown/from_docx/mod.rs:2832) L2570; `Writer::open_threads` (markdown/from_docx/mod.rs:2882) L2571; `Writer::list_marker` (markdown/from_docx/mod.rs:2898) L2586,2595; `Element::attr` (markdown/from_docx/ooxml.rs:286) L2602; `has_page_break` (markdown/from_docx/agent.rs:1266) L2607,2754; `has_section_break` (markdown/from_docx/agent.rs:1496) L2617; `unshown_tags` (markdown/from_docx/agent.rs:836) L2624,2626; `.render` (ambiguous, 4 candidates) L2624,2718,2726,2735; `holds_revision_facts` (markdown/from_docx/agent.rs:1173) L2649; `has_rendered_page_break` (markdown/from_docx/agent.rs:1270) L2651,2664; `Writer::flush_empty` (markdown/from_docx/mod.rs:2769) L2652,2663; `Writer::page_lines` (markdown/from_docx/mod.rs:2783) L2653,2665; `Writer::turn_pages` (markdown/from_docx/mod.rs:2820) L2654,2667; `rendered_page_breaks` (markdown/from_docx/agent.rs:1281) L2654,2668; `ListIndent::place` (markdown/from_docx/ooxml.rs:1228) L2661; `Writer::opens_page` (markdown/from_docx/mod.rs:2811) L2664; `page_breaks` (markdown/from_docx/agent.rs:1275) L2670; `Blocks::push_line` (markdown/from_docx/ooxml.rs:1151) L2707; `id_line` (markdown/from_docx/agent.rs:1514) L2710; `Inline::edges` (markdown/from_docx/ooxml.rs:574) (by name) L2715; `ListIndent::reset` (markdown/from_docx/ooxml.rs:1221) L2717,2731,2734,2742; `escape_closing_hashes` (markdown/from_docx/agent.rs:1375) L2722; `Blocks::push_paragraph` (markdown/from_docx/ooxml.rs:1164) L2723,2729,2732,2736; `Writer::block_text` (markdown/from_docx/mod.rs:2763) L2726,2735; `ListIndent::item` (markdown/from_docx/ooxml.rs:1236) L2727; `hard_breaks` (markdown/from_docx/mod.rs:1356) L2727,2736; `fenced` (markdown/from_docx/mod.rs:1342) L2732; `Inline::render_code` (markdown/from_docx/ooxml.rs:555) (by name) L2732; `Blocks::push` (markdown/from_docx/ooxml.rs:1137) L2743,2755; `Writer::paragraph_mark` (markdown/from_docx/mod.rs:3739) L2748; `Blocks::set_separator` (markdown/from_docx/ooxml.rs:1202) L2757
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.as_ref`, `.as_str`, `.clear`, `.clone`, `.contains`, `.filter`, `.flatten`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.len`, `.map_or`, `.map`, `.min`, `.ok`, `.or_else`, `.or`, `.parse`, `.push`, `.repeat`, `.replace`, `.retain`, `.saturating_sub`, `.then`, `.to_vec`, `.trim_start`, `Some`, `Vec::new`, `std::mem::take`
- **Macros:** `format!`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2401)

#### `Writer::block_text`  (L2763-2765)
`fn block_text(&self, rendered: &str) -> String`

- **Does:** A paragraph's rendered text as its block holds it: a line that would open a Markdown block (`# `, `- `, `1.
- **Calls (crate):** `escape_line_starts` (markdown/from_docx/agent.rs:1421) L2764
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2546)

#### `Writer::flush_empty`  (L2769-2779)
`fn flush_empty(&mut self, blocks: &mut Blocks)`

- **Does:** Agent view: writes the pending `<!-- pN empty -->` lines, after the page-1 marker when they open the document.
- **Calls (crate):** `Writer::opens_page` (markdown/from_docx/mod.rs:2811) L2774; `Writer::page_lines` (markdown/from_docx/mod.rs:2783) L2775; `empty_lines` (markdown/from_docx/agent.rs:1638) L2776; `Blocks::push_line` (markdown/from_docx/ooxml.rs:1151) L2777
- **Calls (std/external/unresolved):** `.is_empty`, `.repeat`, `std::mem::take`
- **Macros:** `format!`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96); `Writer::blocks_in` (markdown/from_docx/mod.rs:2401); `Writer::paragraph` (markdown/from_docx/mod.rs:2546)

#### `Writer::page_lines`  (L2783-2807)
`fn page_lines(&mut self, blocks: &mut Blocks, opens: bool)`

- **Does:** Agent view, cached-break fallback: the page markers due before a block, page 1 included.
- **Calls (crate):** `Blocks::push_line` (markdown/from_docx/ooxml.rs:1151) L2790,2801; `page_marker` (markdown/from_docx/agent.rs:1652) L2793,2804
- **Calls (std/external/unresolved):** `.repeat`, `std::mem::take`
- **Macros:** `format!`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2401); `Writer::paragraph` (markdown/from_docx/mod.rs:2546); `Writer::flush_empty` (markdown/from_docx/mod.rs:2769)

#### `Writer::opens_page`  (L2811-2817)
`fn opens_page(&mut self, cached_break: bool) -> bool`

- **Does:** Whether the next block opens a page: its own cached break, or, without cached breaks, the hard break that ended the block before.
- **Calls (std/external/unresolved):** `.is_some`, `std::mem::take`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2401); `Writer::paragraph` (markdown/from_docx/mod.rs:2546); `Writer::flush_empty` (markdown/from_docx/mod.rs:2769)

#### `Writer::turn_pages`  (L2820-2829)
`fn turn_pages(&mut self, turned: usize)`

- **Does:** Pages turned inside a block, named before the next one.
- **Calls (std/external/unresolved):** `.min`, `.saturating_add`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2401); `Writer::paragraph` (markdown/from_docx/mod.rs:2546)

#### `Writer::threads_of`  (L2832-2843)
`fn threads_of(&self, ids: &[String]) -> Vec<String>`

- **Does:** The thread roots among `ids`, for an id line's `in c5` (agent view).
- **Calls (std/external/unresolved):** `.clone`, `.contains_key`, `.contains`, `.push`, `Vec::new`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2401); `Writer::paragraph` (markdown/from_docx/mod.rs:2546); `Writer::open_threads` (markdown/from_docx/mod.rs:2882)

#### `Writer::open_after`  (L2848-2879)
`fn open_after(&self, container: &Element) -> Vec<String>`

- **Does:** The comment ranges open here plus those `container` opens and leaves open: every range a table line's `in c5` names.
- **Calls (crate):** `Writer::open_ranges` (markdown/from_docx/mod.rs:2888) L2869; `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1572) L2871
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.push`, `.to_vec`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2401)

#### `Writer::open_after::{fn walk}`  (L2849-2868)
`fn walk(e: &Element, comments: &HashMap<String, Element>, open: &mut Vec<String>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L2850; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L2851; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L2854; `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L2856; `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1572) L2865
- **Calls (std/external/unresolved):** `.contains_key`, `.filter`, `.iter`, `.position`, `.push`, `.remove`, `.to_string`
- **Called by:** `text_follows_reference` (markdown/from_docx/mod.rs:1566); `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1572); `body_headings` (markdown/from_docx/mod.rs:1606); `body_headings::{fn walk}` (markdown/from_docx/mod.rs:1607); `closing_fields` (markdown/from_docx/mod.rs:2070); `closing_fields::{fn walk}` (markdown/from_docx/mod.rs:2074); `spanning_fields` (markdown/from_docx/mod.rs:2121); `spanning_fields::{fn walk}` (markdown/from_docx/mod.rs:2125); `Writer::open_after` (markdown/from_docx/mod.rs:2848); `Writer::open_after::{fn walk}` (markdown/from_docx/mod.rs:2849); `shown_text` (markdown/from_docx/mod.rs:4460); `shown_text::{fn walk}` (markdown/from_docx/mod.rs:4461); +2 more

#### `Writer::open_threads`  (L2882-2884)
`fn open_threads(&self) -> Vec<String>`

- **Does:** The comment threads whose range is open here.
- **Calls (crate):** `Writer::threads_of` (markdown/from_docx/mod.rs:2832) L2883; `Writer::open_ranges` (markdown/from_docx/mod.rs:2888) L2883
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2546)

#### `Writer::open_ranges`  (L2888-2894)
`fn open_ranges(&self) -> &[String]`

- **Does:** The comment ranges open here: the highlighted ones, or with the agent view's comments hidden, the ones only id lines name.
- **Calls:** none
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2546); `Writer::open_after` (markdown/from_docx/mod.rs:2848); `Writer::open_threads` (markdown/from_docx/mod.rs:2882)

#### `Writer::list_marker`  (L2898-2959)
`fn list_marker(&mut self, num_id: &str, ilvl: usize) -> Option<(String, String)>`

- **Does:** The Markdown marker for a numbered paragraph (`1.`) and the label Word prints for it (`a)`, `1.2`), advancing Word's counters.
- **Calls (crate):** `.level` (ambiguous: `Numbering::level` (convert/mod.rs:5534); `Numbering::level` (markdown/from_docx/mod.rs:2013)) L2902,2949; `names_deeper_level` (convert/mod.rs:5780) L2933; `Parser::peek` (markdown/math.rs:804) (by name) L2940; `list_number` (convert/mod.rs:5848) L2953
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_str`, `.chars`, `.contains`, `.entry`, `.filter`, `.flatten`, `.get`, `.into`, `.iter_mut`, `.iter`, `.map_or`, `.max`, `.next`, `.or_insert`, `.peekable`, `.push_str`, `.push`, `.skip`, `.then`, `.to_digit`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `Some`, `String::new`, `i64::from`, `u32::try_from`
- **Macros:** `format!`
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2546)

#### `Writer::paragraph_inline`  (L2964-3019)
`fn paragraph_inline(&mut self, p: &Element) -> (Critic, Vec<String>)`

- **Does:** Inline content of a paragraph plus any text-box blocks found inside it.
- **Calls (crate):** `Critic::set_inline_end` (markdown/from_docx/critic.rs:315) L2967; `Critic::set_agent` (markdown/from_docx/critic.rs:304) L2970; `Element::path` (markdown/from_docx/ooxml.rs:320) L2974,2982; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L2976,2982; `Styles::emphasis` (markdown/from_docx/mod.rs:1856) L2977; `Styles::monospace_paragraph` (markdown/from_docx/mod.rs:1792) (by name) L2980; `Critic::comment` (markdown/from_docx/critic.rs:388) L2985; `Critic::open` (markdown/from_docx/critic.rs:393) L2994,2999; `Critic::len` (markdown/from_docx/critic.rs:379) L2997; `closing_fields` (markdown/from_docx/mod.rs:2070) L3001; `Writer::inline` (markdown/from_docx/mod.rs:3021) L3002; `Critic::is_blank` (markdown/from_docx/critic.rs:420) L3010; `Critic::insert_raw` (markdown/from_docx/critic.rs:349) L3013; `Critic::unattributed` (markdown/from_docx/critic.rs:404) L3016
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.iter`, `.map`, `.take`, `.unwrap_or`, `Critic::default`, `Vec::new`, `std::mem::replace`, `std::mem::take`
- **Macros:** `format!`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96); `Writer::paragraph` (markdown/from_docx/mod.rs:2546); `Writer::cell_parts` (markdown/from_docx/mod.rs:3693); `Writer::note_text` (markdown/from_docx/mod.rs:3867)

#### `Writer::inline`  (L3021-3117)
`fn inline( &mut self, container: &Element, link: Option<&str>, base: (bool, bool), fields: &mut Vec<Field>, out: &mut Critic, extra: &mut Vec<String>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L3030; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L3031; `.run` (ambiguous: `Writer::run` (markdown/from_docx/mod.rs:3119); `Writer::run` (markdown/math.rs:1698)) L3032; `Element::rel_attr` (markdown/from_docx/ooxml.rs:294) (by name) L3034; `Rels::link` (markdown/from_docx/ooxml.rs:239) L3034; `link_target` (markdown/from_docx/mod.rs:4477) L3035; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L3035,3036,3040,3048; `titled` (markdown/from_docx/ooxml.rs:251) L3036; `Writer::inline` (markdown/from_docx/mod.rs:3021) L3037,3056,3071,3080; `Writer::slug_of_heading` (markdown/from_docx/mod.rs:2469) L3042; `Critic::raw` (markdown/from_docx/critic.rs:339) L3044,3054,3058,3108,3111; `escape` (markdown/xml.rs:426) L3044; `hyperlink_instruction` (markdown/from_docx/mod.rs:4372) L3049; `printed_code` (markdown/from_docx/mod.rs:2175) L3051; `Writer::change_of` (markdown/from_docx/mod.rs:3790) L3062,3106; `Critic::open` (markdown/from_docx/critic.rs:393) L3063,3107; `Writer::revised` (markdown/from_docx/mod.rs:2456) L3064,3108; `Writer::inline` (markdown/from_docx/mod.rs:3021) (by name) L3065; `Critic::close` (markdown/from_docx/critic.rs:397) L3067,3109; `Writer::comment_range` (markdown/from_docx/mod.rs:2479) L3069; `capturable_alternate` (markdown/from_docx/drawing.rs:1290) L3073; `Writer::drawing` (markdown/from_docx/mod.rs:3366) L3075; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:315); `Scope::child` (xmllinq/serialize.rs:225)) L3078; `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L3084; `math` (markdown/from_docx/mod.rs:3928) L3085; `Element::children_named` (markdown/from_docx/ooxml.rs:308) (by name) L3091; `whole_math_change` (markdown/from_docx/mod.rs:4156) L3104; `strip_math_changes` (markdown/from_docx/mod.rs:4181) L3108
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.as_deref`, `.collect`, `.contains`, `.filter`, `.flatten`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.join`, `.map`, `.or_else`, `.or`, `.then`, `.unwrap_or_default`, `Some`
- **Macros:** `format!`
- **Called by:** `Writer::paragraph_inline` (markdown/from_docx/mod.rs:2964); `Writer::inline` (markdown/from_docx/mod.rs:3021)

#### `Writer::run`  (L3119-3230)
`fn run( &mut self, run: &Element, link: Option<&str>, base: (bool, bool), fields: &mut Vec<Field>, out: &mut Critic, extra: &mut Vec<String>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::child` (markdown/from_docx/ooxml.rs:315) L3128; `hidden_run` (markdown/from_docx/agent.rs:133) L3129; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:315); `Scope::child` (xmllinq/serialize.rs:225)) L3133,3152,3156,3169,3180; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L3134,3157,3170,3181,3205; `Styles::emphasis` (markdown/from_docx/mod.rs:1856) L3135; `Element::toggle` (markdown/from_docx/ooxml.rs:360) (by name) L3138,3142,3176; `Styles::underline` (markdown/from_docx/mod.rs:1875) L3158; `Styles::font` (markdown/from_docx/mod.rs:1784) L3184; `Element::elements` (markdown/from_docx/ooxml.rs:301) L3200,3220; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L3202; `Element::path` (markdown/from_docx/ooxml.rs:320) (by name) L3204; `Critic::reopen` (markdown/from_docx/critic.rs:369) L3214; `Critic::len` (markdown/from_docx/critic.rs:379) L3215,3219,3224; `Critic::raw` (markdown/from_docx/critic.rs:339) L3217,3227; `Writer::run_child` (markdown/from_docx/mod.rs:3232) L3221; `Critic::truncate` (markdown/from_docx/critic.rs:384) L3225
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.filter`, `.flatten`, `.is_some_and`, `.is_some`, `.iter`, `.map`, `.or_else`, `.or`, `.then`, `.unwrap_or`, `Some`
- **Macros:** `matches!`
- **Called by:** `boundary_coverage_tests::run_projection_preserves_field_delimiters_breaks_and_old_formatting` (debug/diff.rs:2372) (?); `raw_source_projection_contract_tests::recorded_run_properties_and_saved_page_breaks_keep_exact_raw_source_projection` (debug/diff.rs:2538) (?); `Writer::inline` (markdown/from_docx/mod.rs:3021) (?); `Writer::node` (markdown/math.rs:1734) (?)

#### `Writer::run_child`  (L3232-3358)
`fn run_child( &mut self, child: &Element, link: Option<&str>, style: Marks, fields: &mut Vec<Field>, out: &mut Critic, extra: &mut Vec<String>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::local` (markdown/from_docx/ooxml.rs:277) L3244; `Critic::push_styled` (markdown/from_docx/critic.rs:329) L3245,3246,3252; `Element::text` (markdown/from_docx/ooxml.rs:344) L3245,3257; `Element::attr` (markdown/from_docx/ooxml.rs:286) L3248,3260,3312,3326; `Critic::push` (markdown/from_docx/critic.rs:319) L3249; `Critic::len` (markdown/from_docx/critic.rs:379) L3271; `hyperlink_instruction` (markdown/from_docx/mod.rs:4372) L3289; `printed_code` (markdown/from_docx/mod.rs:2175) L3295; `Critic::raw` (markdown/from_docx/critic.rs:339) L3298,3306,3322; `Element::is` (markdown/from_docx/ooxml.rs:281) L3313; `Writer::note` (markdown/from_docx/mod.rs:3820) L3340; `Critic::comment` (markdown/from_docx/critic.rs:388) L3341; `Writer::drawing` (markdown/from_docx/mod.rs:3366) L3345,3347; `capturable_alternate` (markdown/from_docx/drawing.rs:1290) L3346; `Element::child` (markdown/from_docx/ooxml.rs:315) L3350; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L3351; `Writer::run_child` (markdown/from_docx/mod.rs:3232) L3352
- **Calls (std/external/unresolved):** `.all`, `.any`, `.as_deref`, `.clone`, `.contains_key`, `.contains`, `.find_map`, `.flatten`, `.insert`, `.is_none`, `.is_some_and`, `.iter`, `.last_mut`, `.len`, `.or_else`, `.or`, `.pop`, `.position`, `.push_str`, `.push`, `.rev`, `.split_last`, `.then`, `.to_string`, `Some`, `String::new`, `std::ptr::from_ref`
- **Macros:** `format!`
- **Called by:** `Writer::run` (markdown/from_docx/mod.rs:3119); `Writer::run_child` (markdown/from_docx/mod.rs:3232)

#### `Writer::drawing`  (L3366-3414)
`fn drawing( &mut self, element: &Element, wrap: bool, out: &mut Critic, extra: &mut Vec<String>, )`

- **Does:** Drawings as Markdown: `![alt](name)` for a plain inline image whose raster extracts to a file, a `drawing:` payload (its element, parts and relationships verbatim) for everything else — anchors, shapes, text boxes, VML, AlternateContent and unextracted images.
- **Calls (crate):** `Writer::drawing_text` (markdown/from_docx/mod.rs:3464) L3376,3380; `Writer::box_comments` (markdown/from_docx/mod.rs:3422) L3383; `is_plain_inline` (markdown/from_docx/drawing.rs:1265) L3385; `Element::find` (markdown/from_docx/ooxml.rs:325) L3386,3393; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L3388,3391; `Element::rel_attr` (markdown/from_docx/ooxml.rs:294) (by name) L3395; `Rels::get` (markdown/from_docx/ooxml.rs:206) L3396; `Media::extracted_markdown` (markdown/from_docx/ooxml.rs:1088) L3398; `Critic::raw` (markdown/from_docx/critic.rs:339) L3399,3406; `ScopeMap::scope` (markdown/from_docx/drawing.rs:187) L3404; `Pending::push` (markdown/from_docx/drawing.rs:211) L3406
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.clone`, `.filter`, `.is_empty`, `.map`, `.or_else`, `.trim`, `.unwrap_or`
- **Called by:** `Writer::inline` (markdown/from_docx/mod.rs:3021); `Writer::run_child` (markdown/from_docx/mod.rs:3232)

#### `Writer::box_comments`  (L3422-3458)
`fn box_comments(&mut self, element: &Element, extra: &mut Vec<String>)`

- **Does:** The `txbxContent` Word renders of `element` — under an `mc:AlternateContent`, only the shown branch's, never the `mc:Fallback` copy — each rendered as a block of `<!-- -->` comment lines pushed to `extra`.
- **Calls (crate):** `Element::child` (markdown/from_docx/ooxml.rs:315) L3424,3426; `Element::find_all` (markdown/from_docx/ooxml.rs:333) (by name) L3431; `Blocks::new` (markdown/from_docx/ooxml.rs:1125) L3439; `Writer::blocks` (markdown/from_docx/mod.rs:2392) L3441; `Blocks::finish` (markdown/from_docx/ooxml.rs:1206) L3442
- **Calls (std/external/unresolved):** `.is_empty`, `.lines`, `.or_else`, `.push`, `.replace`, `.to_string`, `.trim_end_matches`, `ListIndent::default`, `String::new`, `Vec::new`, `std::mem::replace`, `std::mem::take`
- **Macros:** `writeln!`
- **Called by:** `Writer::drawing` (markdown/from_docx/mod.rs:3366)

#### `Writer::drawing_text`  (L3464-3496)
`fn drawing_text(&mut self, drawing: &Element, out: &mut Critic, extra: &mut Vec<String>)`

- **Does:** A drawing's printable content inside a comment or a drawn text box: its text boxes as extra blocks (a text box is a story of its own, so comment ranges around its anchor do not cover it) and its image's alt text.
- **Calls (crate):** `Element::find_all` (markdown/from_docx/ooxml.rs:333) L3469; `Blocks::new` (markdown/from_docx/ooxml.rs:1125) L3474; `Writer::blocks` (markdown/from_docx/mod.rs:2392) L3476; `Blocks::finish` (markdown/from_docx/ooxml.rs:1206) L3477; `Element::find` (markdown/from_docx/ooxml.rs:325) L3481,3487; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L3482,3485; `Element::rel_attr` (markdown/from_docx/ooxml.rs:294) (by name) L3489; `Rels::get` (markdown/from_docx/ooxml.rs:206) L3490; `Media::markdown` (markdown/from_docx/ooxml.rs:1095) L3492; `Critic::raw` (markdown/from_docx/critic.rs:339) L3493
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.clone`, `.filter`, `.is_empty`, `.map`, `.or_else`, `.push`, `.trim`, `.unwrap_or`, `ListIndent::default`, `Vec::new`, `std::mem::take`
- **Called by:** `Writer::drawing` (markdown/from_docx/mod.rs:3366)

#### `Writer::table`  (L3498-3554)
`fn table(&mut self, table: &Element) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `table_rows` (markdown/from_docx/mod.rs:3883) L3500; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:315); `Scope::child` (xmllinq/serialize.rs:225)) L3502; `Writer::revision_marks` (markdown/from_docx/mod.rs:3800) L3504; `Element::path` (markdown/from_docx/ooxml.rs:320) (by name) L3506,3522; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L3508,3524; `row_cells` (markdown/from_docx/mod.rs:3894) L3512; `text_marked` (markdown/from_docx/agent.rs:644) L3516; `nested_marked` (markdown/from_docx/agent.rs:654) L3516; `Writer::cell_text` (markdown/from_docx/mod.rs:3670) L3521; `Writer::column_alignments` (markdown/from_docx/mod.rs:3600) L3549; `markdown_table` (markdown/from_docx/mod.rs:1662) L3553
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.clamp`, `.extend`, `.find`, `.get`, `.is_empty`, `.is_some_and`, `.iter`, `.map_or`, `.map`, `.max`, `.min`, `.ok`, `.parse`, `.push`, `.resize`, `.retain`, `.rev`, `.trim`, `.truncate`, `.unwrap_or_default`, `.unwrap_or`, `String::new`, `Vec::new`, `std::iter::repeat_n`
- **Called by:** `cjk_code_pages` (convert/font.rs:3532) (?); `panose_serif` (convert/font.rs:3543) (?); `avg_char_width` (convert/font.rs:3550) (?); `tests::subset_drops_glyph_names_and_unused_metrics` (convert/pdf.rs:2384) (?); `tests::subset_cmap_and_name_keep_only_what_the_pdf_uses` (convert/pdf.rs:2462) (?); `Writer::blocks_in` (markdown/from_docx/mod.rs:2401)

#### `Writer::cell_styles`  (L3563-3598)
`fn cell_styles(&self, table: &Element) -> Vec<String>`

- **Does:** Each grid column's alignment, as most of its cell paragraphs have it ([`majority`]), and the cell paragraphs aligned against their column, as the table line names them (`p5 center`).
- **Calls (crate):** `table_rows` (markdown/from_docx/mod.rs:3883) L3565; `row_cells` (markdown/from_docx/mod.rs:3894) L3566; `Element::find_all` (markdown/from_docx/ooxml.rs:333) (by name) L3569; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L3572,3573; `Element::path` (markdown/from_docx/ooxml.rs:320) (by name) L3573; `style_token` (markdown/view_notes.rs:1185) L3590
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.into_iter`, `.last_mut`, `.map`, `.ok`, `.parse`, `.push`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2401)

#### `Writer::column_alignments`  (L3600-3642)
`fn column_alignments(&self, table: &Element) -> (Vec<Option<&'static str>>, Vec<String>)`

- **Does:** _no doc comment_
- **Calls (crate):** `table_rows` (markdown/from_docx/mod.rs:3883) L3602; `Element::path` (markdown/from_docx/ooxml.rs:320) (by name) L3603,3616; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L3605,3614,3618; `row_cells` (markdown/from_docx/mod.rs:3894) L3609; `Element::children_named` (markdown/from_docx/ooxml.rs:308) (by name) L3613; `Writer::alignment` (markdown/from_docx/mod.rs:3646) L3614; `majority` (markdown/from_docx/mod.rs:1501) L3628
- **Calls (std/external/unresolved):** `.and_then`, `.clamp`, `.is_none_or`, `.iter`, `.len`, `.map`, `.min`, `.ok`, `.parse`, `.push`, `.resize_with`, `.unwrap_or`, `Vec::new`, `leftish`
- **Macros:** `format!`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2401); `Writer::table` (markdown/from_docx/mod.rs:3498)

#### `Writer::alignment`  (L3646-3663)
`fn alignment(&self, p: &Element) -> Option<&'static str>`

- **Does:** A paragraph's alignment: its own `w:jc`, else its style's; `None` when neither sets one.
- **Calls (crate):** `Element::child` (markdown/from_docx/ooxml.rs:315) L3647; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:315); `Scope::child` (xmllinq/serialize.rs:225)) L3649,3652; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L3650,3653; `Styles::jc` (markdown/from_docx/mod.rs:1800) L3654
- **Calls (std/external/unresolved):** `.and_then`, `.or_else`, `Some`
- **Called by:** `Writer::column_alignments` (markdown/from_docx/mod.rs:3600)

#### `Writer::cell_text`  (L3670-3690)
`fn cell_text(&mut self, row_marks: &[Change], cell: &Element) -> String`

- **Does:** A cell's paragraphs joined with `<br>`, inside the tracked changes of its row (`trPr`) and of the cell itself (`tcPr`).
- **Calls (crate):** `text_marked` (markdown/from_docx/agent.rs:644) L3672; `nested_marked` (markdown/from_docx/agent.rs:654) L3672; `Element::child` (markdown/from_docx/ooxml.rs:315) L3676; `Writer::revision_marks` (markdown/from_docx/mod.rs:3800) L3677; `Writer::cell_parts` (markdown/from_docx/mod.rs:3693) L3681; `fold_tracked_breaks` (markdown/from_docx/mod.rs:1536) L3684; `join_paragraphs` (markdown/from_docx/critic.rs:1413) L3689
- **Calls (std/external/unresolved):** `.extend_from_slice`, `.extend`, `.filter`, `.len`, `.replace`, `.truncate`, `Vec::new`
- **Called by:** `Writer::table` (markdown/from_docx/mod.rs:3498); `Writer::cell_parts` (markdown/from_docx/mod.rs:3693)

#### `Writer::cell_parts`  (L3693-3734)
`fn cell_parts(&mut self, container: &Element, parts: &mut Vec<critic::Part>)`

- **Does:** Paragraph texts in a cell, each with the tracked change on its mark.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L3694; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L3695; `Writer::paragraph_inline` (markdown/from_docx/mod.rs:2964) L3697; `paragraph_mark_kind` (markdown/from_docx/mod.rs:1551) L3701; `Writer::paragraph_mark` (markdown/from_docx/mod.rs:3739) L3703; `Critic::into_inline` (markdown/from_docx/critic.rs:443) (by name) L3706; `.render` (ambiguous, 4 candidates) L3707; `Inline::edges` (markdown/from_docx/ooxml.rs:574) (by name) L3707; `table_rows` (markdown/from_docx/mod.rs:3883) L3713; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:315); `Scope::child` (xmllinq/serialize.rs:225)) L3714; `Writer::revision_marks` (markdown/from_docx/mod.rs:3800) L3716; `row_cells` (markdown/from_docx/mod.rs:3894) L3718; `Writer::cell_text` (markdown/from_docx/mod.rs:3670) L3719; `join_changed` (markdown/from_docx/critic.rs:1390) L3722; `Writer::revised` (markdown/from_docx/mod.rs:2456) L3727; `Writer::change_of` (markdown/from_docx/mod.rs:3790) L3727; `Writer::cell_parts` (markdown/from_docx/mod.rs:3693) (by name) L3727; `Writer::comment_range` (markdown/from_docx/mod.rs:2479) L3729; `Writer::cell_parts` (markdown/from_docx/mod.rs:3693) L3730
- **Calls (std/external/unresolved):** `.collect`, `.extend`, `.filter`, `.into_iter`, `.is_empty`, `.map`, `.push`, `.unwrap_or_default`
- **Called by:** `Writer::cell_text` (markdown/from_docx/mod.rs:3670); `Writer::cell_parts` (markdown/from_docx/mod.rs:3693) (?)

#### `Writer::paragraph_mark`  (L3739-3745)
`fn paragraph_mark(&self, p: &Element) -> Option<Change>`

- **Does:** The tracked change on a paragraph's mark, without its author inside a comment's own text.
- **Calls (crate):** `paragraph_mark` (markdown/from_docx/mod.rs:2232) L3743
- **Calls (std/external/unresolved):** `.as_deref`, `.filter`, `.map`
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2546); `Writer::cell_parts` (markdown/from_docx/mod.rs:3693); `Writer::note_text` (markdown/from_docx/mod.rs:3867)

#### `Writer::note_mark`  (L3749-3764)
`fn note_mark(&self, p: &Element) -> Option<Change>`

- **Does:** The tracked change on a note paragraph's mark.
- **Calls (crate):** `paragraph_mark` (markdown/from_docx/mod.rs:2232) L3751; `Element::path` (markdown/from_docx/ooxml.rs:320) L3753; `Writer::revision_marks` (markdown/from_docx/mod.rs:3800) L3755
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.filter`, `.into_iter`, `.iter`, `.map`, `.nth`, `.position`, `.unwrap_or_default`, `.unwrap_or`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

#### `Writer::agent_tag`  (L3768-3786)
`fn agent_tag(&self, element: &Element) -> String`

- **Does:** The agent tag of a revision element, with its timestamp when `--dates` asks for one and the author has several.
- **Calls (crate):** `tag_of` (markdown/from_docx/agent.rs:170) L3776; `Handles::needs_date` (markdown/from_docx/agent.rs:73) L3779; `Element::attr` (markdown/from_docx/ooxml.rs:286) L3779,3780
- **Calls (std/external/unresolved):** `.as_deref`, `.map_or`, `.push_str`, `.push`, `String::new`
- **Macros:** `format!`
- **Called by:** `Writer::change_of` (markdown/from_docx/mod.rs:3790); `Writer::revision_marks` (markdown/from_docx/mod.rs:3800)

#### `Writer::change_of`  (L3790-3797)
`fn change_of(&self, element: &Element) -> Change`

- **Does:** A tracked change's mark with its attribution: the agent tag, or the `Author (date)` note of the plain conversion.
- **Calls (crate):** `change_of` (markdown/from_docx/mod.rs:2183) L3791; `Writer::agent_tag` (markdown/from_docx/mod.rs:3768) L3793
- **Calls (std/external/unresolved):** `.as_deref`, `Some`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2401); `Writer::inline` (markdown/from_docx/mod.rs:3021); `Writer::cell_parts` (markdown/from_docx/mod.rs:3693)

#### `Writer::revision_marks`  (L3800-3815)
`fn revision_marks(&self, properties: &Element) -> Vec<Change>`

- **Does:** [`revision_marks`] with agent tags when the agent view is on.
- **Calls (crate):** `revision_marks` (markdown/from_docx/mod.rs:2218) L3802; `Element::elements` (markdown/from_docx/ooxml.rs:301) L3804; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L3807; `Writer::agent_tag` (markdown/from_docx/mod.rs:3768) L3812
- **Calls (std/external/unresolved):** `.as_deref`, `.collect`, `.filter_map`, `Some`
- **Called by:** `Writer::table` (markdown/from_docx/mod.rs:3498); `Writer::cell_text` (markdown/from_docx/mod.rs:3670); `Writer::cell_parts` (markdown/from_docx/mod.rs:3693); `Writer::note_mark` (markdown/from_docx/mod.rs:3749)

#### `Writer::note`  (L3820-3861)
`fn note(&mut self, id: &str) -> Option<critic::Note>`

- **Does:** A comment's note, rendered the first time it is needed.
- **Calls (crate):** `Element::children_named` (markdown/from_docx/ooxml.rs:308) (by name) L3828; `Writer::note_text` (markdown/from_docx/mod.rs:3867) L3829,3830; `Handles::needs_date` (markdown/from_docx/agent.rs:73) L3835; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L3835,3836,3841,3855,3856; `comment_head` (markdown/from_docx/agent.rs:2150) L3839; `comment_note` (markdown/from_docx/mod.rs:2244) L3855,3856
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.flatten`, `.get`, `.insert`, `.push_str`, `.then`, `.to_string`, `Some`, `head`, `std::mem::take`
- **Called by:** `boundary_coverage_tests::empty_property_blocks_and_change_records_are_distinct` (debug/diff.rs:2385) (?); `Writer::comment_range` (markdown/from_docx/mod.rs:2479); `Writer::run_child` (markdown/from_docx/mod.rs:3232)

#### `Writer::note_text`  (L3867-3880)
`fn note_text(&mut self, paragraphs: &[&Element], code: bool) -> String`

- **Does:** The note's paragraphs as one line of text.
- **Calls (crate):** `Writer::paragraph_inline` (markdown/from_docx/mod.rs:2964) L3870; `Critic::set_code_block` (markdown/from_docx/critic.rs:310) (by name) L3872; `Writer::paragraph_mark` (markdown/from_docx/mod.rs:3739) L3874; `Critic::into_inline` (markdown/from_docx/critic.rs:443) (by name) L3875; `.render` (ambiguous, 4 candidates) L3876; `Inline::edges` (markdown/from_docx/ooxml.rs:574) (by name) L3876; `join_marked` (markdown/from_docx/critic.rs:1407) L3879
- **Calls (std/external/unresolved):** `.extend`, `.filter`, `.into_iter`, `.is_empty`, `.map`, `.push`, `.replace`, `Vec::new`
- **Called by:** `Writer::note` (markdown/from_docx/mod.rs:3820)

#### `table_rows`  (L3883-3892)
`fn table_rows(table: &Element) -> Vec<&Element>`

- **Does:** _no doc comment_
- **Calls (crate):** `collect_named` (markdown/from_docx/mod.rs:3905) L3885
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `table_line` (markdown/from_docx/agent.rs:1812); `Writer::table` (markdown/from_docx/mod.rs:3498); `Writer::cell_styles` (markdown/from_docx/mod.rs:3563); `Writer::column_alignments` (markdown/from_docx/mod.rs:3600); `Writer::cell_parts` (markdown/from_docx/mod.rs:3693)

#### `row_cells`  (L3894-3903)
`fn row_cells(row: &Element) -> impl Iterator<Item = &Element>`

- **Does:** _no doc comment_
- **Calls (crate):** `collect_named` (markdown/from_docx/mod.rs:3905) L3896
- **Calls (std/external/unresolved):** `.into_iter`, `Vec::new`
- **Called by:** `row_text_marked` (markdown/from_docx/agent.rs:637); `border_clauses` (markdown/from_docx/agent.rs:1731); `table_line` (markdown/from_docx/agent.rs:1812); `Writer::table` (markdown/from_docx/mod.rs:3498); `Writer::cell_styles` (markdown/from_docx/mod.rs:3563); `Writer::column_alignments` (markdown/from_docx/mod.rs:3600); `Writer::cell_parts` (markdown/from_docx/mod.rs:3693)

#### `collect_named`  (L3905-3918)
`fn collect_named<'a>( container: &'a Element, name: &str, wrappers: &[&str], out: &mut Vec<&'a Element>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L3911; `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L3912; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L3914; `collect_named` (markdown/from_docx/mod.rs:3905) L3915
- **Calls (std/external/unresolved):** `.contains`, `.push`
- **Called by:** `table_rows` (markdown/from_docx/mod.rs:3883); `row_cells` (markdown/from_docx/mod.rs:3894); `collect_named` (markdown/from_docx/mod.rs:3905)

#### `math`  (L3928-3930)
`fn math(element: &Element) -> String`

- **Does:** Office Math (OMML) to LaTeX, covering the structures Word's equation editor emits.
- **Calls (crate):** `math_space` (markdown/from_docx/mod.rs:3936) L3929; `math_inner` (markdown/from_docx/mod.rs:3946) L3929
- **Called by:** `coverage_real_source_route_matrix_tests::table_routes_distinguish_authored_math_content_from_missing_lexical_titles` (comparer/lcs.rs:20621) (?); `Writer::inline` (markdown/from_docx/mod.rs:3021); `tests::math_keeps_a_run_of_spaces_and_reads_a_line_break_as_one` (markdown/from_docx/mod.rs:4627)

#### `math_space`  (L3936-3942)
`fn math_space(latex: &str) -> String`

- **Does:** The equation's LaTeX with its edges trimmed.
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.map`, `.trim`
- **Macros:** `matches!`
- **Called by:** `math` (markdown/from_docx/mod.rs:3928)

#### `math_inner`  (L3946-4152)
`fn math_inner(element: &Element, depth: usize, name: bool) -> String`

- **Does:** The LaTeX of `element`; `name` while inside a function's name, whose words print as their command (`\sin`, `\operatorname{argmax}`).
- **Calls (crate):** `math_inner` (markdown/from_docx/mod.rs:3946) L3950,3956,4039; `Element::child` (markdown/from_docx/ooxml.rs:315) L3954,4037; `Element::path` (markdown/from_docx/ooxml.rs:320) L3960; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L3962,4013; `Element::local` (markdown/from_docx/ooxml.rs:277) L3965,4091; `math_run` (markdown/from_docx/mod.rs:4199) L3966; `horizontal_partition_tests::prop` (comparer/lcs_table.rs:863) (by name) L3969,4004,4005,4006,4044,4061...; `prime_count` (markdown/math.rs:1476) L3977; `base` (markdown/from_docx/mod.rs:4361) L3978,3979,3982,3985,4101; `group` (markdown/from_docx/mod.rs:4351) L3979,3982,3986,3987,4053,4056...; `Element::children_named` (markdown/from_docx/ooxml.rs:308) L4007,4109,4127; `structure` (markdown/from_docx/mod.rs:4190) L4011; `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L4012,4125; `Element::path` (markdown/from_docx/ooxml.rs:320) (by name) L4013; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:315); `Scope::child` (xmllinq/serialize.rs:225)) L4015,4041,4058; `fence` (markdown/from_docx/mod.rs:4312) L4030,4031,4032; `operand` (markdown/from_docx/mod.rs:4324) L4041,4058; `nary_command` (markdown/math.rs:527) L4047; `Element::children_named` (markdown/from_docx/ooxml.rs:308) (by name) L4112; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L4125,4130; `Element::elements` (markdown/from_docx/ooxml.rs:301) L4150
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.as_deref`, `.as_slice`, `.as_str`, `.chars`, `.clone`, `.collect`, `.concat`, `.ends_with`, `.into_iter`, `.into`, `.is_empty`, `.is_some_and`, `.join`, `.len`, `.map_or_else`, `.map`, `.next`, `.push_str`, `.repeat`, `.replace`, `.starts_with`, `.to_string`, `.trim_start`, `.trim`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `Some`, `String::new`, `half`, `inner`, `own_runs`, `part`
- **Macros:** `format!`
- **Called by:** `math` (markdown/from_docx/mod.rs:3928); `math_inner` (markdown/from_docx/mod.rs:3946); `operand` (markdown/from_docx/mod.rs:4324)

#### `whole_math_change`  (L4156-4178)
`fn whole_math_change(math: &Element) -> Option<&Element>`

- **Does:** The `w:ins` or `w:del` of math every run of which is inserted, or every run deleted.
- **Calls (crate):** `whole_math_change::{fn runs}` (markdown/from_docx/mod.rs:4157) L4167; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L4170; `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L4170; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L4172
- **Calls (std/external/unresolved):** `.find`, `Some`, `Vec::new`
- **Called by:** `Writer::inline` (markdown/from_docx/mod.rs:3021)

#### `whole_math_change::{fn runs}`  (L4157-4165)
`fn runs<'e>(element: &'e Element, out: &mut Vec<&'e Element>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L4158; `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L4159; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L4161; `whole_math_change::{fn runs}` (markdown/from_docx/mod.rs:4157) L4162
- **Calls (std/external/unresolved):** `.ends_with`, `.push`
- **Called by:** `whole_math_change` (markdown/from_docx/mod.rs:4156); `whole_math_change::{fn runs}` (markdown/from_docx/mod.rs:4157)

#### `strip_math_changes`  (L4181-4187)
`fn strip_math_changes(latex: &str) -> String`

- **Does:** `latex` without the marks of the changes inside it.
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.contains`, `.filter`
- **Called by:** `Writer::inline` (markdown/from_docx/mod.rs:3021)

#### `structure`  (L4190-4194)
`fn structure(e: &Element) -> Vec<&Element>`

- **Does:** The structures inside `e`, without its properties.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L4191; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L4192
- **Calls (std/external/unresolved):** `.collect`, `.ends_with`, `.filter`
- **Called by:** `horizontal_partition_tests::horizontal_row_and_cell_lifetimes_survive_with_format_tracking_disabled` (comparer/lcs_table.rs:1062) (?); `Transaction::resolve_comment_text` (edit.rs:2242) (?); `math_inner` (markdown/from_docx/mod.rs:3946); `operand` (markdown/from_docx/mod.rs:4324)

#### `math_run`  (L4199-4225)
`fn math_run(run: &Element, name: bool) -> String`

- **Does:** A math run's text as LaTeX: in a function's name, the name's command; else escaped and set in its style, inside CriticMarkup when the run is inserted or deleted.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L4202; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L4203; `.text` (ambiguous, 10 candidates) L4204,4215; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L4212; `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L4214; `math_text` (markdown/from_docx/mod.rs:4228) L4220
- **Calls (std/external/unresolved):** `.filter`, `.for_each`, `.is_empty`, `.push_str`, `.replace`, `Some`, `String::new`
- **Macros:** `format!`
- **Called by:** `math_inner` (markdown/from_docx/mod.rs:3946)

#### `math_text`  (L4228-4270)
`fn math_text(run: &Element, text: &str, name: bool) -> String`

- **Does:** A math run's text, `text`, as LaTeX in the run's style.
- **Calls (crate):** `Element::path` (markdown/from_docx/ooxml.rs:320) L4233,4234,4235; `properties` (ambiguous, 5 candidates) L4237,4247,4251,4252; `escape_math` (markdown/from_docx/mod.rs:4274) L4244,4265; `escape_math_text` (markdown/from_docx/mod.rs:4296) L4248; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L4251,4252
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.chars`, `.contains`, `.is_empty`, `.is_none`, `.is_some`, `.or_else`, `.trim`, `Some`, `String::new`
- **Macros:** `format!`
- **Called by:** `math_run` (markdown/from_docx/mod.rs:4199)

#### `escape_math`  (L4274-4293)
`fn escape_math(text: &str) -> String`

- **Does:** Math text with the characters LaTeX acts on escaped; brackets, which would build a delimiter, as their commands.
- **Calls (std/external/unresolved):** `.chars`, `.len`, `.push_str`, `.push`, `String::with_capacity`
- **Called by:** `math_text` (markdown/from_docx/mod.rs:4228)

#### `escape_math_text`  (L4296-4309)
`fn escape_math_text(text: &str) -> String`

- **Does:** `\text{…}`'s text: braces, backslashes and `$` escaped.
- **Calls (std/external/unresolved):** `.chars`, `.len`, `.push_str`, `.push`, `String::with_capacity`
- **Called by:** `math_text` (markdown/from_docx/mod.rs:4228)

#### `fence`  (L4312-4320)
`fn fence(chr: &str) -> String`

- **Does:** A delimiter's character after `\left`, `\middle` or `\right`: none is `.`.
- **Calls (std/external/unresolved):** `.into`
- **Called by:** `tests::the_writer_unwinds_the_code_escape_inside_a_fence` (markdown/from_docx/critic.rs:2530); `tests::the_fence_escape_pair_is_exact` (markdown/from_docx/critic.rs:2562); `math_inner` (markdown/from_docx/mod.rs:3946)

#### `operand`  (L4324-4340)
`fn operand(e: Option<&Element>, depth: usize) -> String`

- **Does:** What a function or an n-ary operator applies to: one bracketed delimiter, one character after a space, anything else in braces.
- **Calls (crate):** `math_inner` (markdown/from_docx/mod.rs:3946) L4328; `math_chars` (markdown/from_docx/mod.rs:4343) L4330; `structure` (markdown/from_docx/mod.rs:4190) L4331; `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L4331
- **Calls (std/external/unresolved):** `.as_slice`, `.into`, `.starts_with`, `.to_string`, `.trim`
- **Macros:** `format!`, `matches!`
- **Called by:** `math_inner` (markdown/from_docx/mod.rs:3946)

#### `math_chars`  (L4343-4349)
`fn math_chars(latex: &str) -> usize`

- **Does:** The characters of math LaTeX, without the marks of changes inside it.
- **Calls (std/external/unresolved):** `.chars`, `.contains`, `.count`, `.filter`
- **Called by:** `operand` (markdown/from_docx/mod.rs:4324); `group` (markdown/from_docx/mod.rs:4351)

#### `group`  (L4351-4358)
`fn group(s: &str) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `math_chars` (markdown/from_docx/mod.rs:4343) L4353
- **Calls (std/external/unresolved):** `.to_string`, `.trim`
- **Macros:** `format!`
- **Called by:** `math_inner` (markdown/from_docx/mod.rs:3946); `base` (markdown/from_docx/mod.rs:4361)

#### `base`  (L4361-4368)
`fn base(s: &str) -> String`

- **Does:** The base of a script or limit: a function's command bare, else a group.
- **Calls (crate):** `ChangeFilter::matches` (changes.rs:177) (by name) L4365; `group` (markdown/from_docx/mod.rs:4351) L4367
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.count`, `.is_ascii_alphabetic`, `.is_some_and`, `.starts_with`, `.strip_prefix`, `.to_string`, `.trim`
- **Called by:** `tests::recording_refuses_what_the_plan_names_differently` (edit/flags.rs:835) (?); `math_inner` (markdown/from_docx/mod.rs:3946); `tests::decoded_part_name_aliases_are_refused_not_overwritten` (opc/mod.rs:1479) (?)

#### `hyperlink_instruction`  (L4372-4399)
`fn hyperlink_instruction(instruction: &str) -> Option<String>`

- **Does:** The link of a `HYPERLINK` field instruction: its URL, `#bookmark` for `\l "bookmark"`, both (`url#bookmark`), titled with `\o`'s tooltip.
- **Calls (crate):** `field_words` (markdown/from_docx/mod.rs:4491) L4373; `link_target` (markdown/from_docx/mod.rs:4477) L4398; `titled` (markdown/from_docx/ooxml.rs:251) L4398
- **Calls (std/external/unresolved):** `.as_str`, `.eq_ignore_ascii_case`, `.flatten`, `.get_or_insert`, `.iter`, `.map`, `.next`, `.split_first`, `.starts_with`, `.then`, `.to_ascii_lowercase`
- **Macros:** `matches!`
- **Called by:** `Writer::inline` (markdown/from_docx/mod.rs:3021); `Writer::run_child` (markdown/from_docx/mod.rs:3232); `bookmark_targets` (markdown/from_docx/mod.rs:4527); `tests::a_link_to_a_bookmark_is_a_markdown_link_and_its_bookmark_an_anchor` (markdown/from_docx/mod.rs:5120); `markdown_source_owner_boundary_tests::numbering_zero_and_out_of_range_overrides_do_not_change_source_order` (markdown/from_docx/mod.rs:6534)

#### `view_table_style`  (L4406-4412)
`fn view_table_style(t: &Element, custom_grid: bool) -> Option<&str>`

- **Does:** The table style the view names for `t`.
- **Calls (crate):** `Element::path` (markdown/from_docx/ooxml.rs:320) L4407; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L4407
- **Calls (std/external/unresolved):** `.and_then`, `Some`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2401); `table_styles` (markdown/from_docx/mod.rs:4442)

#### `custom_table_grid`  (L4418-4438)
`fn custom_table_grid(styles: Option<&Element>) -> bool`

- **Does:** Whether `styles` defines Table Grid with one single border all round that is not Word's 0.5pt: a grid the legend can name (`TableGrid, all borders 1.5pt`) and the writer would otherwise replace with the thin one.
- **Calls (crate):** `Element::children_named` (markdown/from_docx/ooxml.rs:308) (by name) L4421; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L4422,4433,4434; `Element::path` (markdown/from_docx/ooxml.rs:320) (by name) L4424; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:315); `Scope::child` (xmllinq/serialize.rs:225)) L4431
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.collect`, `.filter`, `.find`, `.is_some_and`, `.iter`, `.map`, `Some`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

#### `table_styles`  (L4442-4456)
`fn table_styles(body: &Element, custom_grid: bool) -> (Vec<String>, bool)`

- **Does:** The table styles of `body`'s tables the view names, in the order they first come, and whether a table has none (the writer's Table Grid).
- **Calls (crate):** `tables` (markdown/from_docx/agent.rs:1429) L4445; `view_table_style` (markdown/from_docx/mod.rs:4406) L4446
- **Calls (std/external/unresolved):** `.any`, `.iter`, `.push`, `.to_string`, `Vec::new`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

#### `shown_text`  (L4460-4473)
`fn shown_text(p: &Element) -> String`

- **Does:** The text of `p` the accepted document shows: its `w:t`, not deleted or moved-away text.
- **Calls (crate):** `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1572) L4471
- **Calls (std/external/unresolved):** `String::new`
- **Called by:** `own_slug` (markdown/from_docx/mod.rs:1624)

#### `shown_text::{fn walk}`  (L4461-4469)
`fn walk(e: &Element, out: &mut String)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L4462; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L4463; `.text` (ambiguous, 10 candidates) L4465; `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1572) L4466
- **Calls (std/external/unresolved):** `.push_str`
- **Called by:** `text_follows_reference` (markdown/from_docx/mod.rs:1566); `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1572); `body_headings` (markdown/from_docx/mod.rs:1606); `body_headings::{fn walk}` (markdown/from_docx/mod.rs:1607); `closing_fields` (markdown/from_docx/mod.rs:2070); `closing_fields::{fn walk}` (markdown/from_docx/mod.rs:2074); `spanning_fields` (markdown/from_docx/mod.rs:2121); `spanning_fields::{fn walk}` (markdown/from_docx/mod.rs:2125); `Writer::open_after` (markdown/from_docx/mod.rs:2848); `Writer::open_after::{fn walk}` (markdown/from_docx/mod.rs:2849); `shown_text` (markdown/from_docx/mod.rs:4460); `shown_text::{fn walk}` (markdown/from_docx/mod.rs:4461); +2 more

#### `link_target`  (L4477-4487)
`fn link_target(url: Option<&str>, anchor: Option<&str>) -> Option<String>`

- **Does:** A link's destination from its URL and its bookmark: `url`, `#bookmark`, or `url#bookmark`.
- **Calls (std/external/unresolved):** `.filter`, `.is_empty`, `.to_string`, `Some`
- **Macros:** `format!`
- **Called by:** `Writer::inline` (markdown/from_docx/mod.rs:3021); `hyperlink_instruction` (markdown/from_docx/mod.rs:4372)

#### `field_words`  (L4491-4521)
`fn field_words(instruction: &str) -> Vec<(String, bool)>`

- **Does:** A field instruction's words, each with whether it was quoted: `"a b"` is one word, and `\"` inside quotes is a quote.
- **Calls (crate):** `Parser::peek` (markdown/math.rs:804) (by name) L4494,4504,4513
- **Calls (std/external/unresolved):** `.chars`, `.extend`, `.filter`, `.is_whitespace`, `.next`, `.peekable`, `.push`, `Some`, `String::new`, `Vec::new`
- **Macros:** `matches!`
- **Called by:** `hyperlink_instruction` (markdown/from_docx/mod.rs:4372); `bookmark_targets` (markdown/from_docx/mod.rs:4527)

#### `bookmark_targets`  (L4527-4595)
`fn bookmark_targets<'e>( roots: impl Iterator<Item = &'e Element>, fields_print: bool, ) -> HashSet<String>`

- **Does:** The bookmarks under `roots` a link (`w:hyperlink w:anchor`, `HYPERLINK \l`) or a reference field (`REF`, `PAGEREF`, `NOTEREF`) names.
- **Calls (crate):** `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1572) L4566; `field_words` (markdown/from_docx/mod.rs:4491) L4569; `hyperlink_instruction` (markdown/from_docx/mod.rs:4372) L4576
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.first`, `.get`, `.insert`, `.map`, `.next`, `.split_once`, `.split`, `.to_ascii_uppercase`, `.to_string`, `.unwrap_or`, `HashSet::new`, `Vec::new`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

#### `bookmark_targets::{fn walk}`  (L4531-4562)
`fn walk(e: &Element, refs: bool, codes: &mut Vec<String>, out: &mut HashSet<String>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L4532; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:277)) L4533; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L4535,4541,4544,4555; `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1572) L4538,4542,4559; `.text` (ambiguous, 10 candidates) L4547
- **Calls (std/external/unresolved):** `.filter`, `.insert`, `.is_empty`, `.last_mut`, `.push_str`, `.push`, `.starts_with`, `.to_string`, `.unwrap_or_default`, `Some`, `String::new`
- **Called by:** `text_follows_reference` (markdown/from_docx/mod.rs:1566); `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1572); `body_headings` (markdown/from_docx/mod.rs:1606); `body_headings::{fn walk}` (markdown/from_docx/mod.rs:1607); `closing_fields` (markdown/from_docx/mod.rs:2070); `closing_fields::{fn walk}` (markdown/from_docx/mod.rs:2074); `spanning_fields` (markdown/from_docx/mod.rs:2121); `spanning_fields::{fn walk}` (markdown/from_docx/mod.rs:2125); `Writer::open_after` (markdown/from_docx/mod.rs:2848); `Writer::open_after::{fn walk}` (markdown/from_docx/mod.rs:2849); `shown_text` (markdown/from_docx/mod.rs:4460); `shown_text::{fn walk}` (markdown/from_docx/mod.rs:4461); +2 more

#### `story_color`  (L4599-4608)
`fn story_color(p: &Element) -> Option<String>`

- **Does:** The colour every text run of a header or footer paragraph `p` sets itself (`w:rPr/w:color`), when they all set the same one.
- **Calls (crate):** `Element::find_all` (markdown/from_docx/ooxml.rs:333) L4601; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:315); `Scope::child` (xmllinq/serialize.rs:225)) L4604; `Element::path` (markdown/from_docx/ooxml.rs:320) (by name) L4605; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L4605
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.filter`, `.is_some`, `.iter`, `.map`, `.next`, `.then`, `.to_string`, `Some`, `Vec::new`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

## Test functions (85)

- `tests::a_break_inside_an_equation_stays_a_newline` (L4619): _no doc comment_
- `tests::math_keeps_a_run_of_spaces_and_reads_a_line_break_as_one` (L4627): _no doc comment_
- `tests::a_name_loses_the_xml_indentation_and_keeps_its_spaces` (L4651): _no doc comment_
- `tests::zip` (L4659): _no doc comment_
- `tests::docx` (L4670): _no doc comment_
- `tests::md` (L4685): _no doc comment_
- `tests::p` (L4689): _no doc comment_
- `tests::r` (L4698): _no doc comment_
- `tests::headings_emphasis_links_and_title_style` (L4703): _no doc comment_
- `tests::numbered_and_bulleted_lists` (L4739): _no doc comment_
- `tests::tables_with_spans_and_nested_tables` (L4772): _no doc comment_
- `tests::subscript_and_superscript_runs_keep_their_tags_through_a_round_trip` (L4797): _no doc comment_
- `tests::a_table_cell_keeps_its_equations_latex` (L4832): _no doc comment_
- `tests::an_agent_view_cell_keeps_an_equations_norm_bars` (L4843): _no doc comment_
- `tests::monospace_faces_are_named_by_whole_words` (L4857): _no doc comment_
- `tests::office_math_structures_become_latex` (L4880): _no doc comment_
- `tests::footnotes_images_and_fields` (L5083): _no doc comment_
- `tests::a_link_to_a_bookmark_is_a_markdown_link_and_its_bookmark_an_anchor` (L5120): _no doc comment_
- `tests::a_cross_reference_target_prints_its_anchor_with_nothing_naming_it` (L5162): _no doc comment_
- `tests::image_names_keep_delimiter_characters_out_of_the_markup` (L5175): _no doc comment_
- `tests::tracked_changes_become_critic_markup_without_losing_formatting` (L5190): _no doc comment_
- `tests::tracked_whole_paragraphs_keep_markdown_block_structure` (L5212): _no doc comment_
- `tests::tracked_table_rows_and_cells_keep_the_table_valid` (L5226): _no doc comment_
- `tests::ins` (L5244): Tracked-change building blocks, written the way Word writes them. _(from a code comment)_
- `tests::del` (L5250): _no doc comment_
- `tests::dr` (L5256): _no doc comment_
- `tests::pm` (L5262): A paragraph whose mark (the break after it) carries `mark` (`ins`, `del`, `moveFrom`, `moveTo`, or several), with an optional style.
- `tests::text_box` (L5275): _no doc comment_
- `tests::review_deleted_row_with_deleted_runs_is_marked_once` (L5282): _no doc comment_
- `tests::review_text_box_inside_a_tracked_change_keeps_the_change` (L5296): _no doc comment_
- `tests::review_tracked_paragraph_marks_mark_the_break` (L5313): _no doc comment_
- `tests::review_footnote_revisions_are_marked` (L5338): _no doc comment_
- `tests::comments` (L5360): `word/comments.xml` with one `w:comment` per entry: (id, author, date, paragraphs).
- `tests::start` (L5380): _no doc comment_
- `tests::end` (L5384): _no doc comment_
- `tests::reference` (L5389): The reference run Word writes right after a comment's range end.
- `tests::with_comments` (L5395): _no doc comment_
- `tests::comment_highlights_its_range_and_names_author_and_date_as_word_stores_them` (L5402): _no doc comment_
- `tests::comment_date_is_copied_verbatim_whatever_its_form` (L5423): _no doc comment_
- `tests::comment_note_leaves_out_what_is_missing` (L5440): _no doc comment_
- `tests::comment_without_a_range_is_a_plain_comment_at_its_reference` (L5460): _no doc comment_
- `tests::replies_and_overlapping_ranges_share_one_highlight` (L5478): _no doc comment_
- `tests::comment_range_across_paragraphs_highlights_each_paragraph` (L5510): _no doc comment_
- `tests::comment_ranges_at_block_level_and_in_tables` (L5528): _no doc comment_
- `tests::comment_without_a_reference_gets_its_note_at_the_range_end` (L5545): _no doc comment_
- `tests::unknown_or_malformed_comments_are_ignored` (L5555): _no doc comment_
- `tests::comment_text_keeps_formatting_joins_paragraphs_and_escapes_delimiters` (L5601): _no doc comment_
- `tests::comments_meet_tracked_changes_without_breaking_either` (L5618): _no doc comment_
- `tests::comment_range_around_a_text_box_does_not_cover_the_box` (L5665): _no doc comment_
- `tests::comment_part_is_found_through_the_document_relationships` (L5685): _no doc comment_
- `tests::comments_in_footnotes_and_ranges_left_open_before_them` (L5700): _no doc comment_
- `tests::substitutions_merges_and_empty_changes` (L5740): _no doc comment_
- `tests::delimiters_in_document_text_do_not_open_spans` (L5763): _no doc comment_
- `tests::tracked_changes_with_links_fields_and_hidden_runs` (L5777): _no doc comment_
- `tests::tracked_breaks_keep_headings_lists_and_blocks_valid` (L5795): _no doc comment_
- `tests::tracked_cells_rows_and_nested_tables` (L5843): _no doc comment_
- `tests::tracked_blocks_inside_content_controls` (L5862): _no doc comment_
- `tests::tracked_changes_inside_inline_wrappers_and_styled_breaks` (L5875): _no doc comment_
- `tests::comment_without_an_author_shows_only_its_date` (L5903): _no doc comment_
- `tests::text_boxes_and_content_controls_inside_tracked_cells` (L5913): _no doc comment_
- `tests::deleted_paragraph_mark_preserves_text_and_marks_only_the_separator` (L5931): _no doc comment_
- `tests::deleted_text_boxes_keep_block_structure_without_an_empty_paragraph` (L5942): _no doc comment_
- `tests::deleted_row_with_inline_deletion_has_no_nested_delimiters` (L5984): _no doc comment_
- `tests::by` (L6000): A revision element with an explicit author and date (either may be empty to leave the attribute out).
- `tests::every_tracked_change_names_its_author_and_date_as_word_stores_them` (L6015): _no doc comment_
- `tests::paragraph_breaks_rows_and_cells_carry_their_own_attribution` (L6072): _no doc comment_
- `tests::footnotes_in_comment_text_are_numbered_where_the_comment_is` (L6103): _no doc comment_
- `tests::a_range_ending_between_paragraphs_keeps_its_note` (L6139): _no doc comment_
- `tests::a_note_between_paragraph_and_table_stays_out_of_the_table` (L6159): _no doc comment_
- `tests::comment_text_keeps_text_boxes_without_nesting_comments` (L6186): _no doc comment_
- `tests::a_comment_used_twice_is_written_once_and_its_footnote_counted_once` (L6214): _no doc comment_
- `tests::resolved` (L6245): _no doc comment_
- `tests::accept_and_reject_resolve_paragraphs_breaks_and_moves` (L6254): _no doc comment_
- `tests::a_break_that_goes_away_joins_across_empty_markers` (L6277): _no doc comment_
- `tests::a_joined_paragraph_takes_the_later_paragraphs_properties` (L6304): _no doc comment_
- `tests::accept_and_reject_resolve_rows_cells_and_nested_tables` (L6316): _no doc comment_
- `tests::accept_and_reject_resolve_text_boxes_and_footnotes` (L6342): _no doc comment_
- `tests::reject_restores_formatting_that_a_tracked_change_replaced` (L6377): _no doc comment_
- `tests::accept_and_reject_leave_comments_out` (L6393): _no doc comment_
- `tests::a_note_whose_reference_is_changed_is_changed_too` (L6406): _no doc comment_
- `tests::malformed_input_is_invalid_not_panic` (L6438): _no doc comment_
- `markdown_source_owner_boundary_tests::docx` (L6459): _no doc comment_
- `markdown_source_owner_boundary_tests::style_title_outline_and_long_ancestry_follow_authored_nearest_properties` (L6469): _no doc comment_
- `markdown_source_owner_boundary_tests::hidden_runs_cannot_leak_text_symbols_note_links_or_comment_markers` (L6506): _no doc comment_
- `markdown_source_owner_boundary_tests::numbering_zero_and_out_of_range_overrides_do_not_change_source_order` (L6534): _no doc comment_
