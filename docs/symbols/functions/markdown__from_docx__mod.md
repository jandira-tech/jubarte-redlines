# Functions and methods in `src/markdown/from_docx/mod.rs`

131 non-test, 86 test. Part of the [symbol manifest](../README.md).

#### `<ConvertError as std::fmt::Display>::fmt`  (L71-75, impl std::fmt::Display)
`fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.write_str`
- **Called by:** none found in src

#### `part_stem`  (L92-94)
`fn part_stem(path: &str) -> String`

- **Does:** A part's file stem: `word/footnotes.xml` → `footnotes`, the story `jubarte changes` names its changes by.
- **Calls (crate):** `part_story` (ids.rs:392) L93
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `convert`  (L96-105, pub(crate))
`pub(crate) fn convert(bytes: &[u8], options: &Options) -> Result<Converted, ConvertError>`

- **Does:** _no doc comment_
- **Calls (crate):** `list_logical_changes_tolerant` (changes.rs:613) L100; `with_wholes` (markdown/from_docx/agent.rs:493) L102; `with_logical` (markdown/from_docx/agent.rs:393) L103; `convert_marks` (markdown/from_docx/mod.rs:108) L103
- **Calls (std/external/unresolved):** `.as_deref`, `.map`, `.ok`, `.unwrap_or_default`
- **Called by:** `tests::md` (markdown/from_docx/mod.rs:5000); `tests::resolved` (markdown/from_docx/mod.rs:6606); `tests::malformed_input_is_invalid_not_panic` (markdown/from_docx/mod.rs:6800); `markdown_source_owner_boundary_tests::style_title_outline_and_long_ancestry_follow_authored_nearest_properties` (markdown/from_docx/mod.rs:6831); `markdown_source_owner_boundary_tests::hidden_runs_cannot_leak_text_symbols_note_links_or_comment_markers` (markdown/from_docx/mod.rs:6868); `markdown_source_owner_boundary_tests::numbering_zero_and_out_of_range_overrides_do_not_change_source_order` (markdown/from_docx/mod.rs:6896); `docx_to_markdown` (markdown/mod.rs:651)

#### `convert_marks`  (L108-1366)
`fn convert_marks(bytes: &[u8], options: &Options) -> Result<Converted, ConvertError>`

- **Does:** [`convert`], its tags naming the changes [`agent::with_wholes`] set.
- **Calls (crate):** `Package::open` (ambiguous: `Package::open` (debug.rs:180); `Package::open` (markdown/from_docx/ooxml.rs:43)) L109; `Package::check_tag_policy` (markdown/from_docx/ooxml.rs:61) L111; `Package::main_part` (markdown/from_docx/ooxml.rs:179) L112; `Package::xml` (markdown/from_docx/ooxml.rs:132) L113,127,150,168,182,188...; `invalid` (markdown/from_docx/ooxml.rs:31) L115; `Package::rels` (markdown/from_docx/ooxml.rs:142) L121; `Rels::first_of_type` (markdown/from_docx/ooxml.rs:249) (by name) L123; `limit_test_support::part` (admission.rs:2726) (by name) L128,151,167,189,293; `PartFs::open` (opc/mod.rs:585) L134; `CommentFamily::load` (comments.rs:457) L135; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L139,253,326,329; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L140,219,237,287,289,301...; `CommentFamily::reported_date` (comments.rs:637) (by name) L141; `threads` (markdown/from_docx/agent.rs:2693) L157; `part_stem` (markdown/from_docx/mod.rs:92) L169,184,977; `ScopeMap::of` (markdown/from_docx/drawing.rs:174) L170,446,982; `Rels::targets_of_type` (markdown/from_docx/ooxml.rs:258) (by name) L177,180; `handles` (markdown/from_docx/agent.rs:968) L202; `Element::children_named` (markdown/from_docx/ooxml.rs:379) (by name) L217,284,300,345,450,747...; `bookmark_targets` (markdown/from_docx/mod.rs:4805) L223,406; `body_headings` (markdown/from_docx/mod.rs:1690) L232; `own_slug` (markdown/from_docx/mod.rs:1708) L233; `Element::find_all` (markdown/from_docx/ooxml.rs:404) (by name) L235,456,458,784,933; `is_own_slug` (markdown/from_docx/mod.rs:1717) L238; `stamp` (markdown/from_docx/agent.rs:753) L256; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L267,398,404,468,728,772...; `page_counts` (markdown/from_docx/agent.rs:2132) L268,736; `page_sections` (markdown/from_docx/agent.rs:1943) L271; `resolve` (markdown/from_docx/revise.rs:20) L279,303,318; `is_paragraph_style` (markdown/from_docx/header.rs:913) L286,885,917; `Numbering::parse` (markdown/from_docx/mod.rs:2053) L296; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L325,954; `paragraphs` (markdown/from_docx/agent.rs:1908) L364,524,846; `Media::load` (markdown/from_docx/ooxml.rs:1185) L376; `table_styles` (markdown/from_docx/mod.rs:4720) L397,925; `custom_table_grid` (markdown/from_docx/mod.rs:4689) L399,403,925; `text_width` (markdown/from_docx/header.rs:629) L404; `paragraph_style_sizes` (markdown/from_docx/header.rs:921) L405; `held_marks` (markdown/from_docx/mod.rs:2270) L466; `Blocks::new` (markdown/from_docx/ooxml.rs:1258) L469; `Writer::blocks` (markdown/from_docx/mod.rs:2544) L471; `Writer::flush_empty` (markdown/from_docx/mod.rs:2966) L472; `Writer::take_notes` (markdown/from_docx/mod.rs:2537) L474; `Blocks::push` (markdown/from_docx/ooxml.rs:1270) L475; `text_follows_reference` (markdown/from_docx/mod.rs:1650) L507; `Element::path` (markdown/from_docx/ooxml.rs:391) (by name) L537,787,850,875,938,987...; `Critic::into_inline` (markdown/from_docx/critic.rs:512) (by name) L584; `Writer::paragraph_inline` (markdown/from_docx/mod.rs:3161) L584; `.render` (ambiguous, 4 candidates) L588; `Writer::note_mark` (markdown/from_docx/mod.rs:3998) L601; `paragraph_mark_kind` (markdown/from_docx/mod.rs:1635) L605; `Inline::edges` (markdown/from_docx/ooxml.rs:675) (by name) L608; `join_paragraphs` (markdown/from_docx/critic.rs:1441) L622; `join_marked` (markdown/from_docx/critic.rs:1435) L624; `story_unshown` (markdown/from_docx/agent.rs:1450) L639,1142,1231; `Blocks::finish` (markdown/from_docx/ooxml.rs:1351) L676; `resolve_author` (markdown/from_docx/mod.rs:1371) L688; `revised_by` (markdown/from_docx/agent.rs:851) L693; `coverage_tests::revised` (comparer/finalize.rs:15542) (by name) L698,703; `.of` (ambiguous, 6 candidates) L713; `select_blocks` (markdown/from_docx/agent.rs:3092) L715; `shown_changes` (markdown/from_docx/agent.rs:417) L732; `without_layout` (markdown/from_docx/mod.rs:1408) L750; `named` (ambiguous: `coverage_round_three_tests::coverage_round_five_tests::named` (convert/mod.rs:50395); `named` (markdown/write.rs:6250)) L752; `comment_anchors` (markdown/from_docx/agent.rs:869) L761,763; `page_texts` (markdown/mod.rs:1067) L825; `Styles::heading_level` (markdown/from_docx/mod.rs:1908) L853; `paragraphs_and_boxes` (markdown/from_docx/agent.rs:1926) L874; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L955; `story_paragraphs` (markdown/from_docx/header.rs:1717) L981; `story_color` (markdown/from_docx/mod.rs:4914) L1010; `own_borders` (markdown/from_docx/agent.rs:2174) L1011; `describe` (markdown/borders.rs:209) L1017; `story_frame` (markdown/from_docx/mod.rs:4880) L1022; `story_text_with` (markdown/from_docx/header.rs:1754) L1034; `Pending::push` (markdown/from_docx/drawing.rs:226) L1035; `ScopeMap::scope` (markdown/from_docx/drawing.rs:200) L1039; `story_style` (markdown/from_docx/header.rs:1196) L1101,1128; `story_paragraph_marks` (markdown/from_docx/agent.rs:1513) L1140; `short_paragraph` (ids.rs:435) L1156,1171,1172; `source_sha256` (inspect.rs:299) L1268; `list_changes_tolerant` (changes.rs:607) L1275; `.text` (ambiguous, 10 candidates) L1283; `render` (markdown/from_docx/header.rs:363) L1301,1319; `printed_change_ids` (markdown/from_docx/agent.rs:1270) L1307; `unplaced_entry` (markdown/from_docx/agent.rs:1411) L1315; `note_label` (markdown/from_docx/agent.rs:3281) L1330,1335; `style_token` (markdown/view_notes.rs:1246) L1340; `content_types` (markdown/from_docx/drawing.rs:891) L1357; `fill` (markdown/from_docx/drawing.rs:929) L1358; `Media::into_files` (markdown/from_docx/ooxml.rs:1239) L1364
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.as_deref`, `.as_ref`, `.as_str`, `.chain`, `.checked_sub`, `.clear`, `.clone`, `.cloned`, `.collect`, `.contains`, `.dedup`, `.entry`, `.enumerate`, `.eq_ignore_ascii_case`, `.extend`, `.filter_map`, `.filter`, `.find_map`, `.find`, `.first`, `.flatten`, `.get`, `.hidden`, `.insert`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some_and`, `.is_some`, `.iter_mut`, `.iter`, `.join`, `.keys`, `.last`, `.len`, `.map_err`, `.map_or_else`, `.map_or`, `.map`, `.max_by_key`, `.max`, `.min`, `.ok_or_else`, `.ok`, `.or_default`, `.or_else`, `.or_insert_with`, `.or`, `.parse`, `.pop`, `.position`, `.push_str`, `.push`, `.remove`, `.replace`, `.rposition`, `.saturating_sub`, `.sort`, `.starts_with`, `.take`, `.then`, `.to_string`, `.to_vec`, `.trim_end`, `.trim_start_matches`, `.trim`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `.values`, `.zip`, `HashMap::new`, `HashSet::new`, `ListIndent::default`, `Ok`, `Some`, `String::new`, `Vec::new`, `agent::Handles::default`, `crate::convert::RevisionStyle::default`, `crate::markdown::paginate`, `header::Owner::Creator`, `header::Owner::LastModifiedBy`, `header::StoryView::Accept`, `header::StoryView::Reject`, `header::StoryView::Tagged`, `std::collections::BTreeMap::new`, `std::iter::once`, `std::mem::replace`, `std::ptr::from_ref`, `style_name`, `super::write::Slugger::default`
- **Macros:** `format!`, `matches!`, `vec!`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

#### `resolve_author`  (L1371-1381)
`fn resolve_author(by: &str, handles: &agent::Handles) -> String`

- **Does:** `--by`'s author as `@HH` when it names a known author: a full name as stored, or a handle with or without its `@`.
- **Calls (std/external/unresolved):** `.find`, `.get`, `.map_or_else`, `.or_else`, `.strip_prefix`, `.to_string`, `.unwrap_or`, `.values`
- **Macros:** `format!`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `creator_created`  (L1386-1403, pub(crate))
`pub(crate) fn creator_created(docx: &[u8]) -> (Option<String>, Option<String>)`

- **Does:** A `.docx`'s first non-empty `dc:creator` (its spaces kept, not the XML's indentation) and its `dcterms:created` when that is a timestamp; `None` for each the package does not tell.
- **Calls (crate):** `Package::open` (ambiguous: `Package::open` (debug.rs:180); `Package::open` (markdown/from_docx/ooxml.rs:43)) L1387; `.xml` (ambiguous: `Border::xml` (markdown/borders.rs:158); `Package::xml` (markdown/from_docx/ooxml.rs:132); `XNamespace::xml` (xmllinq/mod.rs:128)) L1389; `Element::children_named` (markdown/from_docx/ooxml.rs:379) (by name) L1393,1398; `without_layout` (markdown/from_docx/mod.rs:1408) L1397; `parse_datetime` (util/datetime.rs:11) L1401
- **Calls (std/external/unresolved):** `.and_then`, `.find_map`, `.find`, `.flatten`, `.is_empty`, `.map`, `.ok`, `.to_string`, `.trim`
- **Called by:** `docx_stamp` (markdown/mod.rs:580)

#### `without_layout`  (L1408-1422)
`fn without_layout(text: &str) -> &str`

- **Does:** `text` without an edge of whitespace that holds a line break: the indentation of pretty-printed XML (`Un-named\n `).
- **Calls (crate):** `breaks` (edit/rewrite.rs:65) (by name) L1411,1417
- **Calls (std/external/unresolved):** `.contains`, `.len`, `.trim_end`, `.trim_start`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108); `creator_created` (markdown/from_docx/mod.rs:1386); `tests::a_name_loses_the_xml_indentation_and_keeps_its_spaces` (markdown/from_docx/mod.rs:4966)

#### `fenced`  (L1426-1434)
`fn fenced(code: &str) -> String`

- **Does:** A code block's text in a fence of backticks, longer than any run of backticks that opens one of its lines.
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.lines`, `.map`, `.max`, `.repeat`, `.take_while`, `.trim_start`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2706)

#### `hard_breaks`  (L1440-1464)
`fn hard_breaks(text: &str) -> String`

- **Does:** A paragraph's line breaks (`w:br`) as Markdown hard breaks, which a bare newline is not: it reads as a space.
- **Calls (crate):** `unescaped_dollar` (markdown/from_docx/mod.rs:1499) L1444; `math_body` (markdown/from_docx/mod.rs:1515) L1455
- **Calls (std/external/unresolved):** `.is_empty`, `.len`, `.push_str`, `.replace`, `.starts_with`, `String::with_capacity`
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2706); `tests::a_break_inside_an_equation_stays_a_newline` (markdown/from_docx/mod.rs:4934)

#### `table_cell`  (L1473-1496)
`fn table_cell(value: &str, escaped: bool) -> String`

- **Does:** Escape a table cell for a Markdown pipe table.
- **Calls (crate):** `cell_math_kept` (markdown/from_docx/mod.rs:1550) L1475; `unescaped_dollar` (markdown/from_docx/mod.rs:1499) L1479; `escaped_cell_text` (markdown/from_docx/mod.rs:1530) L1489,1494; `one_line` (markdown/from_docx/mod.rs:1573) L1495
- **Calls (std/external/unresolved):** `.find`, `.len`, `.push_str`, `.replace`, `.starts_with`, `String::with_capacity`
- **Called by:** `markdown_table` (markdown/from_docx/mod.rs:1746); `tests::a_table_cell_keeps_its_equations_latex` (markdown/from_docx/mod.rs:5171); `tests::an_agent_view_cell_keeps_an_equations_norm_bars` (markdown/from_docx/mod.rs:5182)

#### `unescaped_dollar`  (L1499-1511)
`fn unescaped_dollar(value: &str) -> Option<usize>`

- **Does:** The byte offset of the first `$` no backslash escapes.
- **Calls (std/external/unresolved):** `.char_indices`, `.next`, `Some`
- **Called by:** `defuse` (markdown/from_docx/critic.rs:1240); `hard_breaks` (markdown/from_docx/mod.rs:1440); `table_cell` (markdown/from_docx/mod.rs:1473); `math_body` (markdown/from_docx/mod.rs:1515); `equation_split` (markdown/from_docx/ooxml.rs:1106)

#### `math_body`  (L1515-1527)
`fn math_body(after_open: &str, fence: &str) -> Option<usize>`

- **Does:** The byte length of an equation's body after its opening fence, when `fence` (`$` or `$$`) closes it.
- **Calls (crate):** `unescaped_dollar` (markdown/from_docx/mod.rs:1499) L1518
- **Calls (std/external/unresolved):** `.starts_with`, `Some`
- **Called by:** `defuse` (markdown/from_docx/critic.rs:1240); `hard_breaks` (markdown/from_docx/mod.rs:1440); `equation_split` (markdown/from_docx/ooxml.rs:1106)

#### `escaped_cell_text`  (L1530-1546)
`fn escaped_cell_text(value: &str) -> String`

- **Does:** Escaped cell text outside equations: escapes copied, bare `|` escaped.
- **Calls (std/external/unresolved):** `.chars`, `.len`, `.next`, `.push_str`, `.push`, `String::with_capacity`
- **Called by:** `table_cell` (markdown/from_docx/mod.rs:1473)

#### `cell_math_kept`  (L1550-1570)
`fn cell_math_kept(value: &str) -> String`

- **Does:** `table_cell` for unescaped text: backslashes doubled outside equations, kept inside them; `|` escaped everywhere, as a pipe table needs.
- **Calls (crate):** `one_line` (markdown/from_docx/mod.rs:1573) L1569
- **Calls (std/external/unresolved):** `.find`, `.len`, `.push_str`, `.replace`, `.starts_with`, `String::with_capacity`
- **Called by:** `table_cell` (markdown/from_docx/mod.rs:1473)

#### `one_line`  (L1573-1579)
`fn one_line(value: &str) -> String`

- **Does:** A cell's text on one line: Markdown table rows cannot break.
- **Calls (std/external/unresolved):** `.replace`, `.to_string`, `.trim`
- **Called by:** `table_cell` (markdown/from_docx/mod.rs:1473); `cell_math_kept` (markdown/from_docx/mod.rs:1550)

#### `majority`  (L1585-1611)
`fn majority(aligns: impl Iterator<Item = Option<&'static str>>) -> Option<&'static str>`

- **Does:** The alignment most of a column's cell paragraphs have, as its delimiter row writes it: none on a tie, for justified text, or when most have none.
- **Calls (std/external/unresolved):** `.filter`, `.find`, `.is_some`, `.iter_mut`, `.iter`, `.map`, `.max`, `.next`, `.push`, `.then_some`, `Vec::new`
- **Called by:** `Writer::column_alignments` (markdown/from_docx/mod.rs:3849)

#### `fold_tracked_breaks`  (L1620-1632)
`fn fold_tracked_breaks(parts: &mut [critic::Part])`

- **Does:** Rows as a Markdown pipe table; the first row is the header, and the delimiter row aligns each column as `aligns` says.
- **Calls (crate):** `whole_change` (markdown/from_docx/critic.rs:1397) L1626,1628
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.get`, `.len`, `.or_else`, `.take`
- **Called by:** `Writer::cell_text` (markdown/from_docx/mod.rs:3919)

#### `paragraph_mark_kind`  (L1635-1643)
`fn paragraph_mark_kind(p: &Element) -> Option<Mark>`

- **Does:** The tracked change on `p`'s own mark: `w:pPr/w:rPr/w:ins` or `w:del`.
- **Calls (crate):** `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L1636; `Element::path` (markdown/from_docx/ooxml.rs:391) L1636; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L1638
- **Calls (std/external/unresolved):** `.find_map`, `Some`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108); `Writer::cell_parts` (markdown/from_docx/mod.rs:3942)

#### `text_follows_reference`  (L1650-1686)
`fn text_follows_reference(note: &Element, changes: &[Change]) -> bool`

- **Does:** Whether `note` goes with its reference's change (`changes`, innermost last): it holds no tracked change of its own (resolving the reference resolves the note), or every run of it that holds text sits in a mark of that kind, Word's shape for a note inserted or deleted with its reference.
- **Calls (crate):** `text_follows_reference::{fn tracked}` (markdown/from_docx/mod.rs:1651) L1680; `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1656) L1684
- **Calls (std/external/unresolved):** `.find_map`, `.iter`, `.rev`, `Some`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `text_follows_reference::{fn tracked}`  (L1651-1655)
`fn tracked(e: &Element) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L1652; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L1653; `text_follows_reference::{fn tracked}` (markdown/from_docx/mod.rs:1651) L1653
- **Calls (std/external/unresolved):** `.any`
- **Macros:** `matches!`
- **Called by:** `text_follows_reference` (markdown/from_docx/mod.rs:1650); `text_follows_reference::{fn tracked}` (markdown/from_docx/mod.rs:1651)

#### `text_follows_reference::{fn walk}`  (L1656-1670)
`fn walk(e: &Element, kind: &[&str], inside: bool, all: &mut bool)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L1657; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L1658,1662; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L1660; `.text` (ambiguous, 10 candidates) L1662; `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1656) L1667
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.is_empty`
- **Macros:** `matches!`
- **Called by:** `text_follows_reference` (markdown/from_docx/mod.rs:1650); `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1656); `body_headings` (markdown/from_docx/mod.rs:1690); `body_headings::{fn walk}` (markdown/from_docx/mod.rs:1691); `closing_fields` (markdown/from_docx/mod.rs:2153); `closing_fields::{fn walk}` (markdown/from_docx/mod.rs:2157); `spanning_fields` (markdown/from_docx/mod.rs:2204); `spanning_fields::{fn walk}` (markdown/from_docx/mod.rs:2208); `held_marks` (markdown/from_docx/mod.rs:2270); `held_marks::{fn walk}` (markdown/from_docx/mod.rs:2271); `Writer::open_after` (markdown/from_docx/mod.rs:3045); `Writer::open_after::{fn walk}` (markdown/from_docx/mod.rs:3046); +4 more

#### `body_headings`  (L1690-1704)
`fn body_headings<'e>(body: &'e Element, styles: &Styles) -> Vec<&'e Element>`

- **Does:** The paragraphs of `body` the view writes as blocks, in order, that are headings: not those in tables, which a cell writes as text.
- **Calls (crate):** `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1656) L1702
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `body_headings::{fn walk}`  (L1691-1700)
`fn walk<'e>(e: &'e Element, styles: &Styles, out: &mut Vec<&'e Element>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L1692; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L1693; `heading_level` (markdown/from_docx/mod.rs:1726) L1694; `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1656) L1696
- **Calls (std/external/unresolved):** `.is_some`, `.push`
- **Called by:** `text_follows_reference` (markdown/from_docx/mod.rs:1650); `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1656); `body_headings` (markdown/from_docx/mod.rs:1690); `body_headings::{fn walk}` (markdown/from_docx/mod.rs:1691); `closing_fields` (markdown/from_docx/mod.rs:2153); `closing_fields::{fn walk}` (markdown/from_docx/mod.rs:2157); `spanning_fields` (markdown/from_docx/mod.rs:2204); `spanning_fields::{fn walk}` (markdown/from_docx/mod.rs:2208); `held_marks` (markdown/from_docx/mod.rs:2270); `held_marks::{fn walk}` (markdown/from_docx/mod.rs:2271); `Writer::open_after` (markdown/from_docx/mod.rs:3045); `Writer::open_after::{fn walk}` (markdown/from_docx/mod.rs:3046); +4 more

#### `own_slug`  (L1708-1712)
`fn own_slug(slugger: &mut super::write::Slugger, p: &Element) -> Option<String>`

- **Does:** The name the writer gives heading `p`'s bookmark (github-slugger's, a repeat suffixed by `slugger`); `None` for a heading of no slug.
- **Calls (crate):** `github_slug` (markdown/write.rs:5647) L1709; `shown_text` (markdown/from_docx/mod.rs:4738) L1709; `Slugger::name` (markdown/write.rs:5682) L1710
- **Calls (std/external/unresolved):** `.is_empty`, `.then_some`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108); `Writer::paragraph` (markdown/from_docx/mod.rs:2706)

#### `is_own_slug`  (L1717-1719)
`fn is_own_slug(name: &str, own: Option<&str>) -> bool`

- **Does:** Whether bookmark `name` is a heading's `own` slug: that name, or one the writer gives it (`_` before a slug of no first letter, cut to 40, suffixed `-1` when an anchor took that name).
- **Calls (crate):** `is_bookmark_of` (markdown/write.rs:5623) L1718
- **Calls (std/external/unresolved):** `.is_some_and`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108); `Writer::slug_of_heading` (markdown/from_docx/mod.rs:2627)

#### `heading_level`  (L1726-1744)
`fn heading_level(styles: &Styles, p: &Element, agent: bool) -> Option<usize>`

- **Does:** A paragraph's heading level: its own outline level, else its style's; `None` for body text.
- **Calls (crate):** `Element::child` (markdown/from_docx/ooxml.rs:386) L1727; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L1729,1734; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L1730,1735; `Styles::heading_level` (markdown/from_docx/mod.rs:1908) L1731
- **Calls (std/external/unresolved):** `.and_then`, `.min`, `.ok`, `.parse`, `Some`, `from_style`
- **Called by:** `body_headings::{fn walk}` (markdown/from_docx/mod.rs:1691); `Writer::heading_of` (markdown/from_docx/mod.rs:2633)

#### `markdown_table`  (L1746-1777)
`fn markdown_table(rows: &[Vec<String>], aligns: &[Option<&str>], escaped: bool) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `table_cell` (markdown/from_docx/mod.rs:1473) L1756
- **Calls (std/external/unresolved):** `.copied`, `.enumerate`, `.flatten`, `.get`, `.is_empty`, `.iter`, `.map`, `.max`, `.push_str`, `.push`, `.unwrap_or`, `String::new`
- **Called by:** `Writer::table` (markdown/from_docx/mod.rs:3689)

#### `Styles::parse`  (L1809-1864)
`fn parse(root: &Element) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::children_named` (markdown/from_docx/ooxml.rs:379) L1811,1854; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L1812,1822,1827,1832,1836,1845...; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L1815,1816,1835,1838,1841,1844; `Element::path` (markdown/from_docx/ooxml.rs:391) (by name) L1820,1825,1830; `Element::toggle` (markdown/from_docx/ooxml.rs:431) (by name) L1839,1840,1842; `Element::path` (markdown/from_docx/ooxml.rs:391) L1851; `is_paragraph_style` (markdown/from_docx/header.rs:913) L1856
- **Calls (std/external/unresolved):** `.and_then`, `.find`, `.insert`, `.map`, `.ok`, `.parse`, `.to_ascii_lowercase`, `.to_string`, `.unwrap_or`, `HashMap::new`, `Some`
- **Called by:** `markdown_source_owner_boundary_tests::style_title_outline_and_long_ancestry_follow_authored_nearest_properties` (markdown/from_docx/mod.rs:6831)

#### `Styles::font`  (L1867-1871)
`fn font(&self, id: &str) -> Option<&str>`

- **Does:** The face a run of style `id` is set in, nearest style first.
- **Calls (crate):** `Styles::chain` (markdown/from_docx/mod.rs:1890) L1868
- **Calls (std/external/unresolved):** `.as_deref`, `.find_map`, `.into_iter`
- **Called by:** `Styles::monospace_paragraph` (markdown/from_docx/mod.rs:1875); `Writer::run` (markdown/from_docx/mod.rs:3324)

#### `Styles::monospace_paragraph`  (L1875-1880)
`fn monospace_paragraph(&self, id: Option<&str>) -> bool`

- **Does:** Whether a paragraph of style `id` (or the default paragraph style) is set in a monospace face: its monospace runs are then not code.
- **Calls (crate):** `Styles::font` (markdown/from_docx/mod.rs:1867) L1877
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.is_some_and`, `.or`
- **Called by:** `Writer::paragraph_inline` (markdown/from_docx/mod.rs:3161) (?)

#### `Styles::jc`  (L1883-1887)
`fn jc(&self, id: &str) -> Option<&str>`

- **Does:** The alignment a paragraph of style `id` has, nearest first.
- **Calls (crate):** `Styles::chain` (markdown/from_docx/mod.rs:1890) L1884
- **Calls (std/external/unresolved):** `.as_deref`, `.find_map`, `.into_iter`
- **Called by:** `Transaction::resolve_one_unreported` (edit.rs:3545) (?); `Writer::alignment` (markdown/from_docx/mod.rs:3895)

#### `Styles::chain`  (L1890-1906)
`fn chain(&self, id: &str) -> Vec<&Style>`

- **Does:** The style and its `basedOn` ancestors, nearest first.
- **Calls (std/external/unresolved):** `.clone`, `.get`, `.len`, `.push`, `.to_string`, `Some`, `Vec::new`
- **Called by:** `Styles::font` (markdown/from_docx/mod.rs:1867); `Styles::jc` (markdown/from_docx/mod.rs:1883); `Styles::heading_level` (markdown/from_docx/mod.rs:1908); `Styles::num` (markdown/from_docx/mod.rs:1935); `Styles::emphasis` (markdown/from_docx/mod.rs:1939); `Styles::hidden` (markdown/from_docx/mod.rs:1949); `Styles::underline` (markdown/from_docx/mod.rs:1958)

#### `Styles::heading_level`  (L1908-1933)
`fn heading_level(&self, id: &str) -> Option<usize>`

- **Does:** _no doc comment_
- **Calls (crate):** `Styles::chain` (markdown/from_docx/mod.rs:1890) L1909
- **Calls (std/external/unresolved):** `.and_then`, `.clamp`, `.map`, `.min`, `.ok`, `.parse`, `.strip_prefix`, `.then_some`, `.to_ascii_lowercase`, `.trim`, `Some`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108); `heading_level` (markdown/from_docx/mod.rs:1726); `markdown_source_owner_boundary_tests::style_title_outline_and_long_ancestry_follow_authored_nearest_properties` (markdown/from_docx/mod.rs:6831) (?)

#### `Styles::num`  (L1935-1937)
`fn num(&self, id: &str) -> Option<(String, usize)>`

- **Does:** _no doc comment_
- **Calls (crate):** `Styles::chain` (markdown/from_docx/mod.rs:1890) L1936
- **Calls (std/external/unresolved):** `.clone`, `.find_map`, `.into_iter`
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2706)

#### `Styles::emphasis`  (L1939-1945)
`fn emphasis(&self, id: &str) -> (Option<bool>, Option<bool>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Styles::chain` (markdown/from_docx/mod.rs:1890) L1940
- **Calls (std/external/unresolved):** `.find_map`, `.iter`
- **Called by:** `Writer::paragraph_inline` (markdown/from_docx/mod.rs:3161); `Writer::run` (markdown/from_docx/mod.rs:3324); `markdown_source_owner_boundary_tests::style_title_outline_and_long_ancestry_follow_authored_nearest_properties` (markdown/from_docx/mod.rs:6831) (?)

#### `Styles::hidden`  (L1949-1955)
`fn hidden(&self) -> HashSet<String>`

- **Does:** The character styles whose runs are hidden: the nearest `w:vanish` in the style chain is on.
- **Calls (crate):** `Styles::chain` (markdown/from_docx/mod.rs:1890) L1952
- **Calls (std/external/unresolved):** `.cloned`, `.collect`, `.filter`, `.find_map`, `.iter`, `.keys`, `Some`
- **Called by:** `Layout::push_unpainted` (convert/mod.rs:26558) (?); `residual_byte_and_geometry_contract_tests::notdef_runs_carry_their_text_as_actual_text` (convert/pdf.rs:3023) (?); `tests::hidden_text_paints_nothing_but_reads_back` (convert/raster.rs:748) (?)

#### `Styles::underline`  (L1958-1960)
`fn underline(&self, id: &str) -> Option<bool>`

- **Does:** Whether the style chain underlines, nearest style first.
- **Calls (crate):** `Styles::chain` (markdown/from_docx/mod.rs:1890) L1959
- **Calls (std/external/unresolved):** `.find_map`, `.iter`
- **Called by:** `Writer::run` (markdown/from_docx/mod.rs:3324)

#### `font_of`  (L1964-1970)
`fn font_of(rpr: &Element) -> Option<String>`

- **Does:** The face `w:rFonts` names for Latin text in `rpr`.
- **Calls (crate):** `Element::child` (markdown/from_docx/ooxml.rs:386) L1965; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L1966,1968
- **Calls (std/external/unresolved):** `.map`, `.or_else`
- **Called by:** none found in src

#### `highlight_color`  (L1976-1988)
`fn highlight_color(c: &str) -> Option<&'static str>`

- **Does:** `w:highlight w:val="…"` as one of Word's 16 named highlight colours (`ST_HighlightColor`), the static `&str` the view prints.
- **Calls (std/external/unresolved):** `.copied`, `.find`, `.is_empty`, `.iter`, `.trim`, `.unwrap_or`, `Some`
- **Called by:** none found in src

#### `is_monospace`  (L1992-2017)
`fn is_monospace(font: &str) -> bool`

- **Does:** Whether `font` is a monospace face: the common code and typewriter faces, and any whose name says so.
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.contains`, `.split`, `.starts_with`, `.to_ascii_lowercase`, `.trim`
- **Macros:** `matches!`
- **Called by:** `tests::monospace_faces_are_named_by_whole_words` (markdown/from_docx/mod.rs:5196)

#### `is_underlined`  (L2020-2022)
`fn is_underlined(u: &Element) -> bool`

- **Does:** `w:u` underlines unless its value is `none`.
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:327) L2021
- **Calls (std/external/unresolved):** `.is_none_or`
- **Called by:** none found in src

#### `num_pr`  (L2024-2035)
`fn num_pr(numpr: &Element) -> Option<(String, usize)>`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::child` (markdown/from_docx/ooxml.rs:386) L2025,2029; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L2027,2031
- **Calls (std/external/unresolved):** `.and_then`, `.ok`, `.parse`, `.to_string`, `.unwrap_or`, `Some`
- **Called by:** none found in src

#### `Numbering::parse`  (L2053-2094)
`fn parse(root: &Element) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::children_named` (markdown/from_docx/ooxml.rs:379) L2055,2068; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L2056,2061,2070,2071,2077,2082; `Element::children_named` (markdown/from_docx/ooxml.rs:379) (by name) L2060,2076; `level_of` (markdown/from_docx/mod.rs:2112) L2064,2084; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L2071,2080,2084
- **Calls (std/external/unresolved):** `.and_then`, `.insert`, `.map`, `.ok`, `.or_else`, `.parse`, `.to_string`, `HashMap::new`, `Self::default`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `Numbering::level`  (L2096-2109)
`fn level(&self, num_id: &str, ilvl: usize) -> Option<(&str, Level)>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.cloned`, `.get`, `.into`, `.unwrap_or`, `Some`
- **Called by:** `paragraph_block` (convert/mod.rs:11574) (?); `table_block` (convert/mod.rs:13095) (?); `rendering_uncovered_contract_tests::list_level_geometry_respects_style_and_each_direct_indent_attribute_independently` (convert/mod.rs:59875) (?); `Writer::list_marker` (markdown/from_docx/mod.rs:3095) (?)

#### `level_of`  (L2112-2129)
`fn level_of(lvl: &Element) -> Level`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::child` (markdown/from_docx/ooxml.rs:386) L2114,2119,2124; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L2116,2121,2126
- **Calls (std/external/unresolved):** `.and_then`, `.map`, `.ok`, `.parse`, `.to_string`, `.unwrap_or`
- **Called by:** `Numbering::parse` (markdown/from_docx/mod.rs:2053)

#### `closing_fields`  (L2153-2197)
`fn closing_fields(p: &Element) -> HashSet<*const Element>`

- **Does:** The `fldChar begin` of each field in `p` whose `separate` and `end` are in `p` too, inside the same tracked changes as its begin: the writer reads a field within one change, never one a change boundary cuts.
- **Calls (crate):** `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1656) L2195
- **Calls (std/external/unresolved):** `HashSet::new`, `Vec::new`
- **Called by:** `Writer::paragraph_inline` (markdown/from_docx/mod.rs:3161)

#### `closing_fields::{fn walk}`  (L2157-2193)
`fn walk<'e>( e: &'e Element, changes: &mut Vec<*const Element>, open: &mut Vec<Open<'e>>, out: &mut HashSet<*const Element>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L2163; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L2164,2167; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L2168; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L2185; `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1656) L2187,2190
- **Calls (std/external/unresolved):** `.clone`, `.insert`, `.last_mut`, `.pop`, `.push`, `std::ptr::from_ref`
- **Macros:** `matches!`
- **Called by:** `text_follows_reference` (markdown/from_docx/mod.rs:1650); `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1656); `body_headings` (markdown/from_docx/mod.rs:1690); `body_headings::{fn walk}` (markdown/from_docx/mod.rs:1691); `closing_fields` (markdown/from_docx/mod.rs:2153); `closing_fields::{fn walk}` (markdown/from_docx/mod.rs:2157); `spanning_fields` (markdown/from_docx/mod.rs:2204); `spanning_fields::{fn walk}` (markdown/from_docx/mod.rs:2208); `held_marks` (markdown/from_docx/mod.rs:2270); `held_marks::{fn walk}` (markdown/from_docx/mod.rs:2271); `Writer::open_after` (markdown/from_docx/mod.rs:3045); `Writer::open_after::{fn walk}` (markdown/from_docx/mod.rs:3046); +4 more

#### `spanning_fields`  (L2204-2254)
`fn spanning_fields(container: &Element) -> HashSet<*const Element>`

- **Does:** The `fldChar begin` of each field in `container`'s paragraphs whose `end` lies in a later paragraph, with only paragraphs between (a table of contents), its begin, separate and end outside any tracked change: such a field prints `{CODE}` at the start of its first paragraph and `{/}` in its last.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L2246; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L2247; `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1656) L2248
- **Calls (std/external/unresolved):** `.clear`, `.enumerate`, `HashSet::new`, `Vec::new`
- **Called by:** `Writer::blocks` (markdown/from_docx/mod.rs:2544)

#### `spanning_fields::{fn walk}`  (L2208-2243)
`fn walk<'e>( e: &'e Element, paragraph: usize, changed: bool, open: &mut Vec<Open<'e>>, out: &mut HashSet<*const Element>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L2215; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L2216,2219; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L2220; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L2239; `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1656) L2240
- **Calls (std/external/unresolved):** `.insert`, `.last_mut`, `.pop`, `.push`, `std::ptr::from_ref`
- **Macros:** `matches!`
- **Called by:** `text_follows_reference` (markdown/from_docx/mod.rs:1650); `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1656); `body_headings` (markdown/from_docx/mod.rs:1690); `body_headings::{fn walk}` (markdown/from_docx/mod.rs:1691); `closing_fields` (markdown/from_docx/mod.rs:2153); `closing_fields::{fn walk}` (markdown/from_docx/mod.rs:2157); `spanning_fields` (markdown/from_docx/mod.rs:2204); `spanning_fields::{fn walk}` (markdown/from_docx/mod.rs:2208); `held_marks` (markdown/from_docx/mod.rs:2270); `held_marks::{fn walk}` (markdown/from_docx/mod.rs:2271); `Writer::open_after` (markdown/from_docx/mod.rs:3045); `Writer::open_after::{fn walk}` (markdown/from_docx/mod.rs:3046); +4 more

#### `printed_code`  (L2258-2261)
`fn printed_code(instruction: &str) -> Option<String>`

- **Does:** The `{CODE}` the agent view prints for a field with `instruction`, when it is a field code (see [`crate::markdown::critic::is_field_code`]).
- **Calls (crate):** `is_field_code` (markdown/critic.rs:1625) L2259
- **Calls (std/external/unresolved):** `.then`, `.trim`
- **Macros:** `format!`
- **Called by:** `story_text_with::{fn walk}` (markdown/from_docx/header.rs:1819); `story_text_with::{fn run_content}` (markdown/from_docx/header.rs:1923); `Writer::inline` (markdown/from_docx/mod.rs:3218); `Writer::run_child` (markdown/from_docx/mod.rs:3437)

#### `held_marks`  (L2270-2321)
`fn held_marks(document: &Element) -> HashMap<String, u32>`

- **Does:** A revision element (`w:ins`, `w:del`, `w:moveTo`, `w:moveFrom`) as a tracked change.
- **Calls (crate):** `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1656) L2312; `Element::child` (markdown/from_docx/ooxml.rs:386) L2313
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.into_iter`, `.map`, `.unwrap_or`, `HashMap::new`, `Vec::new`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `held_marks::{fn walk}`  (L2271-2310)
`fn walk( element: &Element, open: &mut Vec<Option<u32>>, found: &mut HashMap<String, Option<u32>>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L2276; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L2277,2279; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L2280,2289; `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1656) L2285,2307
- **Calls (std/external/unresolved):** `.and_modify`, `.and_then`, `.as_slice`, `.entry`, `.filter`, `.flatten`, `.ok`, `.or_insert`, `.parse`, `.pop`, `.push`, `.then`, `.to_string`
- **Macros:** `matches!`
- **Called by:** `text_follows_reference` (markdown/from_docx/mod.rs:1650); `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1656); `body_headings` (markdown/from_docx/mod.rs:1690); `body_headings::{fn walk}` (markdown/from_docx/mod.rs:1691); `closing_fields` (markdown/from_docx/mod.rs:2153); `closing_fields::{fn walk}` (markdown/from_docx/mod.rs:2157); `spanning_fields` (markdown/from_docx/mod.rs:2204); `spanning_fields::{fn walk}` (markdown/from_docx/mod.rs:2208); `held_marks` (markdown/from_docx/mod.rs:2270); `held_marks::{fn walk}` (markdown/from_docx/mod.rs:2271); `Writer::open_after` (markdown/from_docx/mod.rs:3045); `Writer::open_after::{fn walk}` (markdown/from_docx/mod.rs:3046); +4 more

#### `change_of`  (L2323-2330)
`fn change_of(element: &Element, story: Option<&str>) -> Change`

- **Does:** _no doc comment_
- **Calls (crate):** `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L2324; `attribution` (markdown/from_docx/mod.rs:2337) L2329
- **Macros:** `matches!`
- **Called by:** `Writer::change_of` (markdown/from_docx/mod.rs:4038)

#### `attribution`  (L2337-2354)
`fn attribution(element: &Element, story: Option<&str>) -> Option<String>`

- **Does:** Which change it is, and who made it when: a plain-view tag ([`critic::PLAIN`]) of the change's `w:id` in `story` and its note, `w:author` and the preferred revision date, in the same form as a comment's.
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:327) L2339,2345; `comment_note` (markdown/from_docx/mod.rs:2384) L2342; `Element::revision_date` (markdown/from_docx/ooxml.rs:354) L2342; `plain_tag_of` (markdown/from_docx/agent.rs:216) L2352
- **Calls (std/external/unresolved):** `.as_deref`, `.is_empty`, `.is_none`, `.map_or`, `.map`, `.trim_start_matches`, `Some`, `String::new`
- **Macros:** `format!`
- **Called by:** `change_of` (markdown/from_docx/mod.rs:2323); `revision_marks` (markdown/from_docx/mod.rs:2358); `continues_change` (markdown/xml.rs:1701) (?)

#### `revision_marks`  (L2358-2367)
`fn revision_marks(properties: &Element, story: Option<&str>) -> Vec<Change>`

- **Does:** Tracked changes recorded in a properties element (`trPr`, `tcPr`, a paragraph mark's `rPr`), in document order.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L2359; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L2361; `attribution` (markdown/from_docx/mod.rs:2337) L2362,2363
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `Some`
- **Called by:** `paragraph_mark` (markdown/from_docx/mod.rs:2372); `Writer::revision_marks` (markdown/from_docx/mod.rs:4048)

#### `paragraph_mark`  (L2372-2379)
`fn paragraph_mark(p: &Element, story: Option<&str>) -> Option<Change>`

- **Does:** The tracked change on the mark that ends a paragraph, that is on the break between it and the next one.
- **Calls (crate):** `Element::path` (markdown/from_docx/ooxml.rs:391) L2373; `revision_marks` (markdown/from_docx/mod.rs:2358) L2375
- **Calls (std/external/unresolved):** `.into_iter`, `.iter`, `.map`, `.nth`, `.position`, `.unwrap_or_default`, `.unwrap_or`
- **Called by:** `Writer::paragraph_mark` (markdown/from_docx/mod.rs:3988); `Writer::note_mark` (markdown/from_docx/mod.rs:3998)

#### `comment_note`  (L2384-2398)
`fn comment_note(author: Option<&str>, date: Option<&str>, text: &str) -> String`

- **Does:** A comment's `{>>…<<}` text: `Author (date): comment`.
- **Calls (crate):** `escape` (markdown/from_docx/critic.rs:1135) L2387,2390
- **Calls (std/external/unresolved):** `.filter`, `.is_empty`, `.join`, `.map`, `.push`, `.to_string`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `attribution` (markdown/from_docx/mod.rs:2337); `Writer::note` (markdown/from_docx/mod.rs:4068); `tests::comment_note_leaves_out_what_is_missing` (markdown/from_docx/mod.rs:5800)

#### `Writer::take_notes`  (L2537-2542)
`fn take_notes(&mut self) -> String`

- **Does:** The notes waiting for the next paragraph, as CriticMarkup comments: their block is prose, whatever their anchor was.
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`, `std::mem::take`
- **Macros:** `format!`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108); `Writer::blocks_in` (markdown/from_docx/mod.rs:2553)

#### `Writer::blocks`  (L2544-2551)
`fn blocks(&mut self, container: &Element, blocks: &mut Blocks, list: &mut ListIndent)`

- **Does:** _no doc comment_
- **Calls (crate):** `spanning_fields` (markdown/from_docx/mod.rs:2204) L2547; `Writer::blocks_in` (markdown/from_docx/mod.rs:2553) L2548
- **Calls (std/external/unresolved):** `.clear`, `std::mem::replace`
- **Called by:** `tests::a_page_break_stays_inside_its_paragraph_and_a_section_mark_ends_it` (legacy_doc.rs:1998) (?); `tests::table_story` (legacy_doc.rs:2223) (?); `legacy_byte_source_boundary_tests::unterminated_story_tail_and_whitespace_spans_keep_all_authored_characters` (legacy_doc.rs:2900) (?); `convert_marks` (markdown/from_docx/mod.rs:108); `Writer::blocks_in` (markdown/from_docx/mod.rs:2553); `Writer::box_blocks` (markdown/from_docx/mod.rs:3636)

#### `Writer::blocks_in`  (L2553-2611)
`fn blocks_in(&mut self, container: &Element, blocks: &mut Blocks, list: &mut ListIndent)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L2554; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L2555; `Writer::paragraph` (markdown/from_docx/mod.rs:2706) L2556; `ListIndent::reset` (markdown/from_docx/ooxml.rs:1366) L2558; `Writer::take_notes` (markdown/from_docx/mod.rs:2537) L2561; `Blocks::push_prefixed` (markdown/from_docx/ooxml.rs:1278) L2562; `Writer::threads_of` (markdown/from_docx/mod.rs:3029) L2564; `Writer::open_after` (markdown/from_docx/mod.rs:3045) L2564; `table_line` (markdown/from_docx/agent.rs:2378) L2566; `view_table_style` (markdown/from_docx/mod.rs:4676) L2569; `Writer::column_alignments` (markdown/from_docx/mod.rs:3849) L2572; `Writer::cell_styles` (markdown/from_docx/mod.rs:3754) L2573; `Writer::cell_spacings` (markdown/from_docx/mod.rs:3807) L2574; `geometry_clauses` (markdown/from_docx/agent.rs:2286) L2575; `Writer::flush_empty` (markdown/from_docx/mod.rs:2966) L2584; `table_breaks` (markdown/from_docx/agent.rs:1965) L2586; `Writer::opens_page` (markdown/from_docx/mod.rs:3008) L2587; `Writer::page_lines` (markdown/from_docx/mod.rs:2980) L2588; `Blocks::push_line` (markdown/from_docx/ooxml.rs:1284) L2589; `Writer::table` (markdown/from_docx/mod.rs:3689) L2592; `Blocks::push` (markdown/from_docx/ooxml.rs:1270) L2593; `Writer::turn_pages` (markdown/from_docx/mod.rs:3017) L2594; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L2599; `Writer::blocks` (markdown/from_docx/mod.rs:2544) L2600,2607; `Writer::revised` (markdown/from_docx/mod.rs:2614) L2604; `Writer::change_of` (markdown/from_docx/mod.rs:4038) L2604; `.blocks` (ambiguous: `Story::blocks` (legacy_doc.rs:1629); `Writer::blocks` (markdown/from_docx/mod.rs:2544)) L2604; `Writer::comment_range` (markdown/from_docx/mod.rs:2637) L2606
- **Calls (std/external/unresolved):** `.clear`, `.concat`, `.flatten`, `.is_none`, `.then_some`, `.then`
- **Called by:** `Writer::blocks` (markdown/from_docx/mod.rs:2544)

#### `Writer::revised`  (L2614-2618)
`fn revised(&mut self, change: Change, write: impl FnOnce(&mut Self))`

- **Does:** Writes content inside a tracked change.
- **Calls (crate):** `write` (markdown/view_stories.rs:1599) (by name) L2616
- **Calls (std/external/unresolved):** `.pop`, `.push`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2553); `Writer::inline` (markdown/from_docx/mod.rs:3218); `Writer::cell_parts` (markdown/from_docx/mod.rs:3942)

#### `Writer::slug_of_heading`  (L2627-2629)
`fn slug_of_heading(&self, name: &str) -> bool`

- **Does:** A comment range start or end.
- **Calls (crate):** `is_own_slug` (markdown/from_docx/mod.rs:1717) L2628
- **Calls (std/external/unresolved):** `.as_deref`
- **Called by:** `Writer::inline` (markdown/from_docx/mod.rs:3218)

#### `Writer::heading_of`  (L2633-2635)
`fn heading_of(&self, p: &Element) -> Option<usize>`

- **Does:** A paragraph's heading level: its own outline level, else its style's; `None` for body text.
- **Calls (crate):** `heading_level` (markdown/from_docx/mod.rs:1726) L2634
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2706)

#### `Writer::comment_range`  (L2637-2704)
`fn comment_range(&mut self, range: &Element, out: Option<&mut Critic>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:327) L2640,2661; `Element::is` (markdown/from_docx/ooxml.rs:322) L2644,2651,2668; `Writer::note` (markdown/from_docx/mod.rs:4068) L2687,2696; `Writer::held_note` (markdown/from_docx/mod.rs:4117) L2690
- **Calls (std/external/unresolved):** `.any`, `.close`, `.comment`, `.contains_key`, `.contains`, `.extend`, `.filter`, `.flatten`, `.insert`, `.is_empty`, `.iter`, `.open`, `.position`, `.push`, `.remove`, `.then`, `.to_string`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2553); `Writer::inline` (markdown/from_docx/mod.rs:3218); `Writer::cell_parts` (markdown/from_docx/mod.rs:3942)

#### `Writer::paragraph`  (L2706-2955)
`fn paragraph(&mut self, p: &Element, blocks: &mut Blocks, list: &mut ListIndent)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::child` (markdown/from_docx/ooxml.rs:386) L2707; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L2709,2714,2818,2828; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L2710,2819; `Writer::heading_of` (markdown/from_docx/mod.rs:2633) L2712; `Styles::num` (markdown/from_docx/mod.rs:1935) L2716; `Writer::open_ranges` (markdown/from_docx/mod.rs:3085) L2718; `own_slug` (markdown/from_docx/mod.rs:1708) L2719; `Writer::paragraph_inline` (markdown/from_docx/mod.rs:3161) L2720; `Critic::set_code_block` (markdown/from_docx/critic.rs:353) (by name) L2724; `Element::path` (markdown/from_docx/ooxml.rs:391) L2730; `Writer::revision_marks` (markdown/from_docx/mod.rs:4048) L2732; `attribution_note` (markdown/from_docx/critic.rs:1115) L2740; `display` (markdown/from_docx/critic.rs:148) L2740; `.raw` (ambiguous: `RawEntry::raw` (admission.rs:1021); `Critic::raw` (markdown/from_docx/critic.rs:382)) L2746; `Critic::is_blank` (markdown/from_docx/critic.rs:471) (by name) L2748; `Critic::into_inline` (markdown/from_docx/critic.rs:512) (by name) L2749; `Writer::threads_of` (markdown/from_docx/mod.rs:3029) L2752; `Writer::open_threads` (markdown/from_docx/mod.rs:3079) L2753; `Writer::list_marker` (markdown/from_docx/mod.rs:3095) L2768,2777; `Element::attr` (markdown/from_docx/ooxml.rs:327) L2784; `has_page_break` (markdown/from_docx/agent.rs:1727) L2789,2951; `has_section_break` (markdown/from_docx/agent.rs:1976) L2799; `unshown_tags` (markdown/from_docx/agent.rs:1171) L2806,2808; `.render` (ambiguous, 4 candidates) L2806,2910,2918,2929; `spacing_clauses` (markdown/from_docx/agent.rs:1606) L2829; `Writer::own_size` (markdown/from_docx/mod.rs:3794) L2831; `holds_revision_facts` (markdown/from_docx/agent.rs:1548) L2840; `has_rendered_page_break` (markdown/from_docx/agent.rs:1731) L2842,2855; `Writer::flush_empty` (markdown/from_docx/mod.rs:2966) L2843,2854; `Writer::page_lines` (markdown/from_docx/mod.rs:2980) L2844,2856; `Writer::turn_pages` (markdown/from_docx/mod.rs:3017) L2845,2858; `rendered_page_breaks` (markdown/from_docx/agent.rs:1742) L2845,2859; `ListIndent::place` (markdown/from_docx/ooxml.rs:1373) L2852; `Writer::opens_page` (markdown/from_docx/mod.rs:3008) L2855; `page_breaks` (markdown/from_docx/agent.rs:1736) L2861; `Blocks::push_line` (markdown/from_docx/ooxml.rs:1284) L2899; `id_line` (markdown/from_docx/agent.rs:1994) L2902; `Inline::edges` (markdown/from_docx/ooxml.rs:675) (by name) L2907; `ListIndent::reset` (markdown/from_docx/ooxml.rs:1366) L2909,2923,2928,2938; `escape_closing_hashes` (markdown/from_docx/agent.rs:1836) L2914; `Blocks::push_paragraph` (markdown/from_docx/ooxml.rs:1297) L2915,2921,2926,2930; `Writer::block_text` (markdown/from_docx/mod.rs:2960) L2918,2929; `ListIndent::item` (markdown/from_docx/ooxml.rs:1381) L2919; `hard_breaks` (markdown/from_docx/mod.rs:1440) L2919,2930; `Inline::render_code` (markdown/from_docx/ooxml.rs:656) (by name) L2924; `fenced` (markdown/from_docx/mod.rs:1426) L2926; `Blocks::empty_paragraph` (markdown/from_docx/ooxml.rs:1335) L2933; `Blocks::push` (markdown/from_docx/ooxml.rs:1270) L2939,2952; `Writer::paragraph_mark` (markdown/from_docx/mod.rs:3988) L2944; `Blocks::set_separator` (markdown/from_docx/ooxml.rs:1347) L2954
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.as_ref`, `.as_str`, `.clear`, `.clone`, `.contains`, `.filter`, `.flatten`, `.is_empty`, `.is_none`, `.is_some_and`, `.is_some`, `.len`, `.map_or`, `.map`, `.min`, `.ok`, `.or_else`, `.or`, `.parse`, `.push_str`, `.push`, `.repeat`, `.replace`, `.retain`, `.saturating_sub`, `.then`, `.to_vec`, `.trim_start`, `.unwrap_or_default`, `Some`, `String::new`, `Vec::new`, `std::mem::take`
- **Macros:** `format!`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2553)

#### `Writer::block_text`  (L2960-2962)
`fn block_text(&self, rendered: &str) -> String`

- **Does:** A paragraph's rendered text as its block holds it: a line that would open a Markdown block (`# `, `- `, `1.
- **Calls (crate):** `escape_line_starts` (markdown/from_docx/agent.rs:1882) L2961
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2706)

#### `Writer::flush_empty`  (L2966-2976)
`fn flush_empty(&mut self, blocks: &mut Blocks)`

- **Does:** Agent view: writes the pending `<!-- pN empty -->` lines, after the page-1 marker when they open the document.
- **Calls (crate):** `Writer::opens_page` (markdown/from_docx/mod.rs:3008) L2971; `Writer::page_lines` (markdown/from_docx/mod.rs:2980) L2972; `empty_lines` (markdown/from_docx/agent.rs:2111) L2973; `Blocks::push_line` (markdown/from_docx/ooxml.rs:1284) L2974
- **Calls (std/external/unresolved):** `.is_empty`, `.repeat`, `std::mem::take`
- **Macros:** `format!`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108); `Writer::blocks_in` (markdown/from_docx/mod.rs:2553); `Writer::paragraph` (markdown/from_docx/mod.rs:2706)

#### `Writer::page_lines`  (L2980-3004)
`fn page_lines(&mut self, blocks: &mut Blocks, opens: bool)`

- **Does:** Agent view, cached-break fallback: the page markers due before a block, page 1 included.
- **Calls (crate):** `Blocks::push_line` (markdown/from_docx/ooxml.rs:1284) L2987,2998; `page_marker` (markdown/from_docx/agent.rs:2125) L2990,3001
- **Calls (std/external/unresolved):** `.repeat`, `std::mem::take`
- **Macros:** `format!`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2553); `Writer::paragraph` (markdown/from_docx/mod.rs:2706); `Writer::flush_empty` (markdown/from_docx/mod.rs:2966)

#### `Writer::opens_page`  (L3008-3014)
`fn opens_page(&mut self, cached_break: bool) -> bool`

- **Does:** Whether the next block opens a page: its own cached break, or, without cached breaks, the hard break that ended the block before.
- **Calls (std/external/unresolved):** `.is_some`, `std::mem::take`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2553); `Writer::paragraph` (markdown/from_docx/mod.rs:2706); `Writer::flush_empty` (markdown/from_docx/mod.rs:2966)

#### `Writer::turn_pages`  (L3017-3026)
`fn turn_pages(&mut self, turned: usize)`

- **Does:** Pages turned inside a block, named before the next one.
- **Calls (std/external/unresolved):** `.min`, `.saturating_add`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2553); `Writer::paragraph` (markdown/from_docx/mod.rs:2706)

#### `Writer::threads_of`  (L3029-3040)
`fn threads_of(&self, ids: &[String]) -> Vec<String>`

- **Does:** The thread roots among `ids`, for an id line's `in c5` (agent view).
- **Calls (std/external/unresolved):** `.clone`, `.contains_key`, `.contains`, `.push`, `Vec::new`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2553); `Writer::paragraph` (markdown/from_docx/mod.rs:2706); `Writer::open_threads` (markdown/from_docx/mod.rs:3079)

#### `Writer::open_after`  (L3045-3076)
`fn open_after(&self, container: &Element) -> Vec<String>`

- **Does:** The comment ranges open here plus those `container` opens and leaves open: every range a table line's `in c5` names.
- **Calls (crate):** `Writer::open_ranges` (markdown/from_docx/mod.rs:3085) L3066; `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1656) L3068
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.push`, `.to_vec`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2553)

#### `Writer::open_after::{fn walk}`  (L3046-3065)
`fn walk(e: &Element, comments: &HashMap<String, Element>, open: &mut Vec<String>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L3047; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L3048; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L3051; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L3053; `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1656) L3062
- **Calls (std/external/unresolved):** `.contains_key`, `.filter`, `.iter`, `.position`, `.push`, `.remove`, `.to_string`
- **Called by:** `text_follows_reference` (markdown/from_docx/mod.rs:1650); `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1656); `body_headings` (markdown/from_docx/mod.rs:1690); `body_headings::{fn walk}` (markdown/from_docx/mod.rs:1691); `closing_fields` (markdown/from_docx/mod.rs:2153); `closing_fields::{fn walk}` (markdown/from_docx/mod.rs:2157); `spanning_fields` (markdown/from_docx/mod.rs:2204); `spanning_fields::{fn walk}` (markdown/from_docx/mod.rs:2208); `held_marks` (markdown/from_docx/mod.rs:2270); `held_marks::{fn walk}` (markdown/from_docx/mod.rs:2271); `Writer::open_after` (markdown/from_docx/mod.rs:3045); `Writer::open_after::{fn walk}` (markdown/from_docx/mod.rs:3046); +4 more

#### `Writer::open_threads`  (L3079-3081)
`fn open_threads(&self) -> Vec<String>`

- **Does:** The comment threads whose range is open here.
- **Calls (crate):** `Writer::threads_of` (markdown/from_docx/mod.rs:3029) L3080; `Writer::open_ranges` (markdown/from_docx/mod.rs:3085) L3080
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2706)

#### `Writer::open_ranges`  (L3085-3091)
`fn open_ranges(&self) -> &[String]`

- **Does:** The comment ranges open here: the highlighted ones, or with the agent view's comments hidden, the ones only id lines name.
- **Calls:** none
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2706); `Writer::open_after` (markdown/from_docx/mod.rs:3045); `Writer::open_threads` (markdown/from_docx/mod.rs:3079)

#### `Writer::list_marker`  (L3095-3156)
`fn list_marker(&mut self, num_id: &str, ilvl: usize) -> Option<(String, String)>`

- **Does:** The Markdown marker for a numbered paragraph (`1.`) and the label Word prints for it (`a)`, `1.2`), advancing Word's counters.
- **Calls (crate):** `.level` (ambiguous: `Numbering::level` (convert/mod.rs:5535); `Numbering::level` (markdown/from_docx/mod.rs:2096)) L3099,3146; `names_deeper_level` (convert/mod.rs:5781) L3130; `Parser::peek` (markdown/math.rs:804) (by name) L3137; `list_number` (convert/mod.rs:5849) L3150
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_str`, `.chars`, `.contains`, `.entry`, `.filter`, `.flatten`, `.get`, `.into`, `.iter_mut`, `.iter`, `.map_or`, `.max`, `.next`, `.or_insert`, `.peekable`, `.push_str`, `.push`, `.skip`, `.then`, `.to_digit`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `Some`, `String::new`, `i64::from`, `u32::try_from`
- **Macros:** `format!`
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2706)

#### `Writer::paragraph_inline`  (L3161-3216)
`fn paragraph_inline(&mut self, p: &Element) -> (Critic, Vec<String>)`

- **Does:** Inline content of a paragraph plus any text-box blocks found inside it.
- **Calls (crate):** `Critic::set_inline_end` (markdown/from_docx/critic.rs:358) L3164; `Critic::set_agent` (markdown/from_docx/critic.rs:347) L3167; `Element::path` (markdown/from_docx/ooxml.rs:391) L3171,3179; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L3173,3179; `Styles::emphasis` (markdown/from_docx/mod.rs:1939) L3174; `Styles::monospace_paragraph` (markdown/from_docx/mod.rs:1875) (by name) L3177; `Critic::comment` (markdown/from_docx/critic.rs:431) L3182; `Critic::open` (markdown/from_docx/critic.rs:436) L3191,3196; `Critic::len` (markdown/from_docx/critic.rs:422) L3194; `closing_fields` (markdown/from_docx/mod.rs:2153) L3198; `Writer::inline` (markdown/from_docx/mod.rs:3218) L3199; `Critic::is_blank` (markdown/from_docx/critic.rs:471) L3207; `Critic::insert_raw` (markdown/from_docx/critic.rs:392) L3210; `Critic::unattributed` (markdown/from_docx/critic.rs:455) L3213
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.iter`, `.map`, `.take`, `.unwrap_or`, `Critic::default`, `Vec::new`, `std::mem::replace`, `std::mem::take`
- **Macros:** `format!`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108); `Writer::paragraph` (markdown/from_docx/mod.rs:2706); `Writer::cell_parts` (markdown/from_docx/mod.rs:3942); `Writer::note_text` (markdown/from_docx/mod.rs:4137)

#### `Writer::inline`  (L3218-3322)
`fn inline( &mut self, container: &Element, link: Option<&str>, base: (bool, bool), fields: &mut Vec<Field>, out: &mut Critic, extra: &mut Vec<String>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L3227; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L3228; `.run` (ambiguous: `Writer::run` (markdown/from_docx/mod.rs:3324); `Writer::run` (markdown/math.rs:1698)) L3229; `Element::rel_attr` (markdown/from_docx/ooxml.rs:365) (by name) L3231; `Rels::link` (markdown/from_docx/ooxml.rs:278) L3231; `link_target` (markdown/from_docx/mod.rs:4755) L3232; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L3232,3233,3237,3245; `titled` (markdown/from_docx/ooxml.rs:290) L3233; `Writer::inline` (markdown/from_docx/mod.rs:3218) L3234,3253,3272,3281; `Writer::slug_of_heading` (markdown/from_docx/mod.rs:2627) L3239; `Critic::raw` (markdown/from_docx/critic.rs:382) L3241,3251,3255,3313,3316; `escape` (markdown/xml.rs:601) L3241; `hyperlink_instruction` (markdown/from_docx/mod.rs:4642) L3246; `printed_code` (markdown/from_docx/mod.rs:2258) L3248; `Writer::change_of` (markdown/from_docx/mod.rs:4038) L3259,3307; `Critic::open_dated` (markdown/from_docx/critic.rs:441) L3261,3309; `Element::revision_date` (markdown/from_docx/ooxml.rs:354) (by name) L3261,3309; `Critic::open` (markdown/from_docx/critic.rs:436) L3263,3311; `Writer::revised` (markdown/from_docx/mod.rs:2614) L3265,3313; `.inline` (ambiguous: `EditAction::inline` (cli/edit_actions.rs:341); `Writer::inline` (markdown/from_docx/mod.rs:3218)) L3266; `Critic::close` (markdown/from_docx/critic.rs:448) L3268,3314; `Writer::comment_range` (markdown/from_docx/mod.rs:2637) L3270; `capturable_alternate` (markdown/from_docx/drawing.rs:1516) L3274; `Writer::drawing` (markdown/from_docx/mod.rs:3572) L3276; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L3279; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L3285; `math` (markdown/from_docx/mod.rs:4198) L3286; `Element::children_named` (markdown/from_docx/ooxml.rs:379) (by name) L3292; `whole_math_change` (markdown/from_docx/mod.rs:4426) L3305; `strip_math_changes` (markdown/from_docx/mod.rs:4451) L3313
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.as_deref`, `.collect`, `.contains`, `.filter`, `.flatten`, `.is_empty`, `.is_none`, `.is_some`, `.iter`, `.join`, `.map`, `.or_else`, `.or`, `.then`, `.unwrap_or_default`, `Some`
- **Macros:** `format!`
- **Called by:** `Writer::paragraph_inline` (markdown/from_docx/mod.rs:3161); `Writer::inline` (markdown/from_docx/mod.rs:3218)

#### `Writer::run`  (L3324-3435)
`fn run( &mut self, run: &Element, link: Option<&str>, base: (bool, bool), fields: &mut Vec<Field>, out: &mut Critic, extra: &mut Vec<String>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::child` (markdown/from_docx/ooxml.rs:386) L3333; `hidden_run` (markdown/from_docx/agent.rs:157) L3334; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L3338,3357,3361,3374,3385; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L3339,3362,3375,3386,3410; `Styles::emphasis` (markdown/from_docx/mod.rs:1939) L3340; `Element::toggle` (markdown/from_docx/ooxml.rs:431) (by name) L3343,3347,3381; `Styles::underline` (markdown/from_docx/mod.rs:1958) L3363; `Styles::font` (markdown/from_docx/mod.rs:1867) L3389; `Element::elements` (markdown/from_docx/ooxml.rs:372) L3405,3425; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L3407; `Element::path` (markdown/from_docx/ooxml.rs:391) (by name) L3409; `Critic::reopen` (markdown/from_docx/critic.rs:412) L3419; `Critic::len` (markdown/from_docx/critic.rs:422) L3420,3424,3429; `Critic::raw` (markdown/from_docx/critic.rs:382) L3422,3432; `Writer::run_child` (markdown/from_docx/mod.rs:3437) L3426; `Critic::truncate` (markdown/from_docx/critic.rs:427) L3430
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.filter`, `.flatten`, `.is_some_and`, `.is_some`, `.iter`, `.map`, `.or_else`, `.or`, `.then`, `.unwrap_or`, `Some`
- **Macros:** `matches!`
- **Called by:** `boundary_coverage_tests::run_projection_preserves_field_delimiters_breaks_and_old_formatting` (debug/diff.rs:2372) (?); `raw_source_projection_contract_tests::recorded_run_properties_and_saved_page_breaks_keep_exact_raw_source_projection` (debug/diff.rs:2538) (?); `Writer::inline` (markdown/from_docx/mod.rs:3218) (?); `Writer::node` (markdown/math.rs:1734) (?)

#### `Writer::run_child`  (L3437-3563)
`fn run_child( &mut self, child: &Element, link: Option<&str>, style: Marks, fields: &mut Vec<Field>, out: &mut Critic, extra: &mut Vec<String>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::local` (markdown/from_docx/ooxml.rs:318) L3449; `Critic::push_styled` (markdown/from_docx/critic.rs:372) L3450,3451,3457; `Element::text` (markdown/from_docx/ooxml.rs:415) L3450,3462; `Element::attr` (markdown/from_docx/ooxml.rs:327) L3453,3465,3517,3531; `Critic::push` (markdown/from_docx/critic.rs:362) L3454; `Critic::len` (markdown/from_docx/critic.rs:422) L3476; `hyperlink_instruction` (markdown/from_docx/mod.rs:4642) L3494; `printed_code` (markdown/from_docx/mod.rs:2258) L3500; `Critic::raw` (markdown/from_docx/critic.rs:382) L3503,3511,3527; `Element::is` (markdown/from_docx/ooxml.rs:322) L3518; `Writer::note` (markdown/from_docx/mod.rs:4068) L3545; `Critic::comment` (markdown/from_docx/critic.rs:431) L3546; `Writer::drawing` (markdown/from_docx/mod.rs:3572) L3550,3552; `capturable_alternate` (markdown/from_docx/drawing.rs:1516) L3551; `Element::child` (markdown/from_docx/ooxml.rs:386) L3555; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L3556; `Writer::run_child` (markdown/from_docx/mod.rs:3437) L3557
- **Calls (std/external/unresolved):** `.all`, `.any`, `.as_deref`, `.clone`, `.contains_key`, `.contains`, `.find_map`, `.flatten`, `.insert`, `.is_none`, `.is_some_and`, `.iter`, `.last_mut`, `.len`, `.or_else`, `.or`, `.pop`, `.position`, `.push_str`, `.push`, `.rev`, `.split_last`, `.then`, `.to_string`, `Some`, `String::new`, `std::ptr::from_ref`
- **Macros:** `format!`
- **Called by:** `Writer::run` (markdown/from_docx/mod.rs:3324); `Writer::run_child` (markdown/from_docx/mod.rs:3437)

#### `Writer::drawing`  (L3572-3632)
`fn drawing( &mut self, element: &Element, wrap: bool, out: &mut Critic, extra: &mut Vec<String>, )`

- **Does:** Drawings as Markdown: `![alt](name)` for a plain inline image whose raster extracts to a file, a `drawing:` payload (its element, parts and relationships verbatim) for everything else — anchors, shapes, text boxes, VML, AlternateContent and unextracted images.
- **Calls (crate):** `Writer::drawing_text` (markdown/from_docx/mod.rs:3670) L3580,3609; `is_plain_inline` (markdown/from_docx/drawing.rs:1491) L3584; `Element::find` (markdown/from_docx/ooxml.rs:396) L3585,3592; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L3587,3590; `Element::rel_attr` (markdown/from_docx/ooxml.rs:365) (by name) L3594; `Rels::get` (markdown/from_docx/ooxml.rs:245) L3595; `Media::extracted_markdown` (markdown/from_docx/ooxml.rs:1221) L3597; `Critic::raw` (markdown/from_docx/critic.rs:382) L3598,3626; `Pending::reserve` (markdown/from_docx/drawing.rs:242) L3602; `ScopeMap::scope` (markdown/from_docx/drawing.rs:200) L3606; `Writer::box_blocks` (markdown/from_docx/mod.rs:3636) L3614; `Pending::attach` (markdown/from_docx/drawing.rs:265) L3625
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.clone`, `.collect`, `.enumerate`, `.extend`, `.filter`, `.is_empty`, `.iter`, `.map`, `.or_else`, `.trim`, `.unwrap_or`, `Vec::new`, `std::mem::replace`, `std::mem::take`
- **Macros:** `format!`
- **Called by:** `Writer::inline` (markdown/from_docx/mod.rs:3218); `Writer::run_child` (markdown/from_docx/mod.rs:3437)

#### `Writer::box_blocks`  (L3636-3666)
`fn box_blocks(&mut self, element: &Element, extra: &mut Vec<String>)`

- **Does:** Only the nearest-owned stories in the shown branch.
- **Calls (crate):** `Element::child` (markdown/from_docx/ooxml.rs:386) L3638,3640; `Element::find_all` (markdown/from_docx/ooxml.rs:404) (by name) L3642; `nested_editable` (markdown/from_docx/drawing.rs:383) L3643; `Blocks::new` (markdown/from_docx/ooxml.rs:1258) L3653; `Writer::blocks` (markdown/from_docx/mod.rs:2544) L3655; `Blocks::finish` (markdown/from_docx/ooxml.rs:1351) L3656
- **Calls (std/external/unresolved):** `.or_else`, `.push`, `.take`, `.unwrap_or`, `ListIndent::default`, `Vec::new`, `std::mem::replace`, `std::mem::take`
- **Called by:** `Writer::drawing` (markdown/from_docx/mod.rs:3572); `Writer::drawing_text` (markdown/from_docx/mod.rs:3670)

#### `Writer::drawing_text`  (L3670-3687)
`fn drawing_text(&mut self, drawing: &Element, out: &mut Critic, extra: &mut Vec<String>)`

- **Does:** Rejected nested drawings remain readable, with no independent editable token.
- **Calls (crate):** `Writer::box_blocks` (markdown/from_docx/mod.rs:3636) L3671; `Element::find` (markdown/from_docx/ooxml.rs:396) L3672,3678; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L3673,3676; `Element::rel_attr` (markdown/from_docx/ooxml.rs:365) (by name) L3680; `Rels::get` (markdown/from_docx/ooxml.rs:245) L3681; `Media::markdown` (markdown/from_docx/ooxml.rs:1228) L3683; `Critic::raw` (markdown/from_docx/critic.rs:382) L3684
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.clone`, `.filter`, `.is_empty`, `.map`, `.or_else`, `.trim`, `.unwrap_or`
- **Called by:** `Writer::drawing` (markdown/from_docx/mod.rs:3572)

#### `Writer::table`  (L3689-3745)
`fn table(&mut self, table: &Element) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `table_rows` (markdown/from_docx/mod.rs:4153) L3691; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L3693; `Writer::revision_marks` (markdown/from_docx/mod.rs:4048) L3695; `Element::path` (markdown/from_docx/ooxml.rs:391) (by name) L3697,3713; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L3699,3715; `row_cells` (markdown/from_docx/mod.rs:4164) L3703; `text_marked` (markdown/from_docx/agent.rs:908) L3707; `nested_marked` (markdown/from_docx/agent.rs:918) L3707; `Writer::cell_text` (markdown/from_docx/mod.rs:3919) L3712; `Writer::column_alignments` (markdown/from_docx/mod.rs:3849) L3740; `markdown_table` (markdown/from_docx/mod.rs:1746) L3744
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.clamp`, `.extend`, `.find`, `.get`, `.is_empty`, `.is_some_and`, `.iter`, `.map_or`, `.map`, `.max`, `.min`, `.ok`, `.parse`, `.push`, `.resize`, `.retain`, `.rev`, `.trim`, `.truncate`, `.unwrap_or_default`, `.unwrap_or`, `String::new`, `Vec::new`, `std::iter::repeat_n`
- **Called by:** `cjk_code_pages` (convert/font.rs:3532) (?); `panose_serif` (convert/font.rs:3543) (?); `avg_char_width` (convert/font.rs:3550) (?); `tests::subset_drops_glyph_names_and_unused_metrics` (convert/pdf.rs:2384) (?); `tests::subset_cmap_and_name_keep_only_what_the_pdf_uses` (convert/pdf.rs:2462) (?); `Writer::blocks_in` (markdown/from_docx/mod.rs:2553)

#### `Writer::cell_styles`  (L3754-3789)
`fn cell_styles(&self, table: &Element) -> Vec<String>`

- **Does:** Each grid column's alignment, as most of its cell paragraphs have it ([`majority`]), and the cell paragraphs aligned against their column, as the table line names them (`p5 center`).
- **Calls (crate):** `table_rows` (markdown/from_docx/mod.rs:4153) L3756; `row_cells` (markdown/from_docx/mod.rs:4164) L3757; `Element::find_all` (markdown/from_docx/ooxml.rs:404) (by name) L3760; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L3763,3764; `Element::path` (markdown/from_docx/ooxml.rs:391) (by name) L3764; `style_token` (markdown/view_notes.rs:1246) L3781
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.into_iter`, `.last_mut`, `.map`, `.ok`, `.parse`, `.push`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2553)

#### `Writer::own_size`  (L3794-3802)
`fn own_size(&self, p: &Element) -> Option<u32>`

- **Does:** The text size paragraph `p` states and its style does not give (`size 9pt`), in half-points (#1474).
- **Calls (crate):** `Element::path` (markdown/from_docx/ooxml.rs:391) L3796; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L3798; `own_size` (markdown/from_docx/agent.rs:1106) L3801
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.get`, `.unwrap_or`
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2706); `Writer::cell_spacings` (markdown/from_docx/mod.rs:3807)

#### `Writer::cell_spacings`  (L3807-3847)
`fn cell_spacings(&self, table: &Element) -> Vec<String>`

- **Does:** `p8-p38 line 8.65pt exact size 9pt`: the cell paragraphs' own spacing (#1473) and text size (#1474), consecutive ones that say the same as one range.
- **Calls (crate):** `table_rows` (markdown/from_docx/mod.rs:4153) L3809; `row_cells` (markdown/from_docx/mod.rs:4164) L3810; `Element::find_all` (markdown/from_docx/ooxml.rs:404) (by name) L3813; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L3815; `Element::path` (markdown/from_docx/ooxml.rs:391) (by name) L3819; `Writer::own_size` (markdown/from_docx/mod.rs:3794) L3824
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.extend`, `.into_iter`, `.is_empty`, `.join`, `.last_mut`, `.map`, `.ok`, `.parse`, `.push`, `.unwrap_or_default`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2553)

#### `Writer::column_alignments`  (L3849-3891)
`fn column_alignments(&self, table: &Element) -> (Vec<Option<&'static str>>, Vec<String>)`

- **Does:** _no doc comment_
- **Calls (crate):** `table_rows` (markdown/from_docx/mod.rs:4153) L3851; `Element::path` (markdown/from_docx/ooxml.rs:391) (by name) L3852,3865; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L3854,3863,3867; `row_cells` (markdown/from_docx/mod.rs:4164) L3858; `Element::children_named` (markdown/from_docx/ooxml.rs:379) (by name) L3862; `Writer::alignment` (markdown/from_docx/mod.rs:3895) L3863; `majority` (markdown/from_docx/mod.rs:1585) L3877
- **Calls (std/external/unresolved):** `.and_then`, `.clamp`, `.is_none_or`, `.iter`, `.len`, `.map`, `.min`, `.ok`, `.parse`, `.push`, `.resize_with`, `.unwrap_or`, `Vec::new`, `leftish`
- **Macros:** `format!`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2553); `Writer::table` (markdown/from_docx/mod.rs:3689)

#### `Writer::alignment`  (L3895-3912)
`fn alignment(&self, p: &Element) -> Option<&'static str>`

- **Does:** A paragraph's alignment: its own `w:jc`, else its style's; `None` when neither sets one.
- **Calls (crate):** `Element::child` (markdown/from_docx/ooxml.rs:386) L3896; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L3898,3901; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L3899,3902; `Styles::jc` (markdown/from_docx/mod.rs:1883) L3903
- **Calls (std/external/unresolved):** `.and_then`, `.or_else`, `Some`
- **Called by:** `Writer::column_alignments` (markdown/from_docx/mod.rs:3849)

#### `Writer::cell_text`  (L3919-3939)
`fn cell_text(&mut self, row_marks: &[Change], cell: &Element) -> String`

- **Does:** A cell's paragraphs joined with `<br>`, inside the tracked changes of its row (`trPr`) and of the cell itself (`tcPr`).
- **Calls (crate):** `text_marked` (markdown/from_docx/agent.rs:908) L3921; `nested_marked` (markdown/from_docx/agent.rs:918) L3921; `Element::child` (markdown/from_docx/ooxml.rs:386) L3925; `Writer::revision_marks` (markdown/from_docx/mod.rs:4048) L3926; `Writer::cell_parts` (markdown/from_docx/mod.rs:3942) L3930; `fold_tracked_breaks` (markdown/from_docx/mod.rs:1620) L3933; `join_paragraphs` (markdown/from_docx/critic.rs:1441) L3938
- **Calls (std/external/unresolved):** `.extend_from_slice`, `.extend`, `.filter`, `.len`, `.replace`, `.truncate`, `Vec::new`
- **Called by:** `Writer::table` (markdown/from_docx/mod.rs:3689); `Writer::cell_parts` (markdown/from_docx/mod.rs:3942)

#### `Writer::cell_parts`  (L3942-3983)
`fn cell_parts(&mut self, container: &Element, parts: &mut Vec<critic::Part>)`

- **Does:** Paragraph texts in a cell, each with the tracked change on its mark.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L3943; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L3944; `Writer::paragraph_inline` (markdown/from_docx/mod.rs:3161) L3946; `paragraph_mark_kind` (markdown/from_docx/mod.rs:1635) L3950; `Writer::paragraph_mark` (markdown/from_docx/mod.rs:3988) L3952; `Critic::into_inline` (markdown/from_docx/critic.rs:512) (by name) L3955; `.render` (ambiguous, 4 candidates) L3956; `Inline::edges` (markdown/from_docx/ooxml.rs:675) (by name) L3956; `table_rows` (markdown/from_docx/mod.rs:4153) L3962; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L3963; `Writer::revision_marks` (markdown/from_docx/mod.rs:4048) L3965; `row_cells` (markdown/from_docx/mod.rs:4164) L3967; `Writer::cell_text` (markdown/from_docx/mod.rs:3919) L3968; `join_changed` (markdown/from_docx/critic.rs:1418) L3971; `Writer::revised` (markdown/from_docx/mod.rs:2614) L3976; `Writer::change_of` (markdown/from_docx/mod.rs:4038) L3976; `Writer::cell_parts` (markdown/from_docx/mod.rs:3942) (by name) L3976; `Writer::comment_range` (markdown/from_docx/mod.rs:2637) L3978; `Writer::cell_parts` (markdown/from_docx/mod.rs:3942) L3979
- **Calls (std/external/unresolved):** `.collect`, `.extend`, `.filter`, `.into_iter`, `.is_empty`, `.map`, `.push`, `.unwrap_or_default`
- **Called by:** `Writer::cell_text` (markdown/from_docx/mod.rs:3919); `Writer::cell_parts` (markdown/from_docx/mod.rs:3942) (?)

#### `Writer::paragraph_mark`  (L3988-3994)
`fn paragraph_mark(&self, p: &Element) -> Option<Change>`

- **Does:** The tracked change on a paragraph's mark, without its author inside a comment's own text.
- **Calls (crate):** `paragraph_mark` (markdown/from_docx/mod.rs:2372) L3992
- **Calls (std/external/unresolved):** `.as_deref`, `.filter`, `.map`
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2706); `Writer::cell_parts` (markdown/from_docx/mod.rs:3942); `Writer::note_text` (markdown/from_docx/mod.rs:4137)

#### `Writer::note_mark`  (L3998-4013)
`fn note_mark(&self, p: &Element) -> Option<Change>`

- **Does:** The tracked change on a note paragraph's mark.
- **Calls (crate):** `paragraph_mark` (markdown/from_docx/mod.rs:2372) L4000; `Element::path` (markdown/from_docx/ooxml.rs:391) L4002; `Writer::revision_marks` (markdown/from_docx/mod.rs:4048) L4004
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.filter`, `.into_iter`, `.iter`, `.map`, `.nth`, `.position`, `.unwrap_or_default`, `.unwrap_or`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `Writer::agent_tag`  (L4017-4034)
`fn agent_tag(&self, element: &Element) -> String`

- **Does:** The agent tag of a revision element, with its timestamp when the author has undated marks, or `--dates` asks for differing dates.
- **Calls (crate):** `tag_of` (markdown/from_docx/agent.rs:194) L4025; `Handles::needs_date` (markdown/from_docx/agent.rs:92) L4027; `Element::attr` (markdown/from_docx/ooxml.rs:327) L4027; `Element::revision_date` (markdown/from_docx/ooxml.rs:354) L4028
- **Calls (std/external/unresolved):** `.as_deref`, `.map_or`, `.push_str`, `.push`, `String::new`
- **Macros:** `format!`
- **Called by:** `Writer::change_of` (markdown/from_docx/mod.rs:4038); `Writer::revision_marks` (markdown/from_docx/mod.rs:4048)

#### `Writer::change_of`  (L4038-4045)
`fn change_of(&self, element: &Element) -> Change`

- **Does:** A tracked change's mark with its attribution: the agent tag, or the `Author (date)` note of the plain conversion.
- **Calls (crate):** `change_of` (markdown/from_docx/mod.rs:2323) L4039; `Writer::agent_tag` (markdown/from_docx/mod.rs:4017) L4041
- **Calls (std/external/unresolved):** `.as_deref`, `Some`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2553); `Writer::inline` (markdown/from_docx/mod.rs:3218); `Writer::cell_parts` (markdown/from_docx/mod.rs:3942)

#### `Writer::revision_marks`  (L4048-4063)
`fn revision_marks(&self, properties: &Element) -> Vec<Change>`

- **Does:** [`revision_marks`] with agent tags when the agent view is on.
- **Calls (crate):** `revision_marks` (markdown/from_docx/mod.rs:2358) L4050; `Element::elements` (markdown/from_docx/ooxml.rs:372) L4052; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L4055; `Writer::agent_tag` (markdown/from_docx/mod.rs:4017) L4060
- **Calls (std/external/unresolved):** `.as_deref`, `.collect`, `.filter_map`, `Some`
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2706); `Writer::table` (markdown/from_docx/mod.rs:3689); `Writer::cell_text` (markdown/from_docx/mod.rs:3919); `Writer::cell_parts` (markdown/from_docx/mod.rs:3942); `Writer::note_mark` (markdown/from_docx/mod.rs:3998)

#### `Writer::note`  (L4068-4096)
`fn note(&mut self, id: &str) -> Option<critic::Note>`

- **Does:** A comment's note, rendered the first time it is needed.
- **Calls (crate):** `Element::children_named` (markdown/from_docx/ooxml.rs:379) (by name) L4076; `Writer::note_text` (markdown/from_docx/mod.rs:4137) L4077,4078; `Writer::agent_head` (markdown/from_docx/mod.rs:4099) L4083; `comment_note` (markdown/from_docx/mod.rs:2384) L4090,4091; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L4090,4091; `Element::comment_date` (markdown/from_docx/ooxml.rs:350) (by name) L4090,4091
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.get`, `.insert`, `.to_string`, `Some`, `head`, `std::mem::take`
- **Called by:** `boundary_coverage_tests::empty_property_blocks_and_change_records_are_distinct` (debug/diff.rs:2385) (?); `Writer::comment_range` (markdown/from_docx/mod.rs:2637); `Writer::run_child` (markdown/from_docx/mod.rs:3437)

#### `Writer::agent_head`  (L4099-4113)
`fn agent_head(&self, id: &str, comment: &Element, held: Option<u32>) -> String`

- **Does:** The agent view's head of comment `id`'s note ([`agent::comment_head`]).
- **Calls (crate):** `Handles::needs_date` (markdown/from_docx/agent.rs:92) (by name) L4100; `Element::attr` (markdown/from_docx/ooxml.rs:327) L4102,4107; `Element::comment_date` (markdown/from_docx/ooxml.rs:350) L4103; `comment_head` (markdown/from_docx/agent.rs:2724) L4105
- **Calls (std/external/unresolved):** `.flatten`, `.then`
- **Called by:** `Writer::note` (markdown/from_docx/mod.rs:4068); `Writer::held_note` (markdown/from_docx/mod.rs:4117)

#### `Writer::held_note`  (L4117-4131)
`fn held_note(&self, id: &str, note: critic::Note) -> critic::Note`

- **Does:** `note`, its head naming the change that holds comment `id`'s reference (`c0 @B in r13: `), when one does.
- **Calls (crate):** `Writer::agent_head` (markdown/from_docx/mod.rs:4099) L4121,4122
- **Calls (std/external/unresolved):** `.get`, `.strip_prefix`, `Some`, `swap`
- **Macros:** `format!`
- **Called by:** `Writer::comment_range` (markdown/from_docx/mod.rs:2637)

#### `Writer::note_text`  (L4137-4150)
`fn note_text(&mut self, paragraphs: &[&Element], code: bool) -> String`

- **Does:** The note's paragraphs as one line of text.
- **Calls (crate):** `Writer::paragraph_inline` (markdown/from_docx/mod.rs:3161) L4140; `Critic::set_code_block` (markdown/from_docx/critic.rs:353) (by name) L4142; `Writer::paragraph_mark` (markdown/from_docx/mod.rs:3988) L4144; `Critic::into_inline` (markdown/from_docx/critic.rs:512) (by name) L4145; `.render` (ambiguous, 4 candidates) L4146; `Inline::edges` (markdown/from_docx/ooxml.rs:675) (by name) L4146; `join_marked` (markdown/from_docx/critic.rs:1435) L4149
- **Calls (std/external/unresolved):** `.extend`, `.filter`, `.into_iter`, `.is_empty`, `.map`, `.push`, `.replace`, `Vec::new`
- **Called by:** `Writer::note` (markdown/from_docx/mod.rs:4068)

#### `table_rows`  (L4153-4162)
`fn table_rows(table: &Element) -> Vec<&Element>`

- **Does:** _no doc comment_
- **Calls (crate):** `collect_named` (markdown/from_docx/mod.rs:4175) L4155
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `table_line` (markdown/from_docx/agent.rs:2378); `Writer::table` (markdown/from_docx/mod.rs:3689); `Writer::cell_styles` (markdown/from_docx/mod.rs:3754); `Writer::cell_spacings` (markdown/from_docx/mod.rs:3807); `Writer::column_alignments` (markdown/from_docx/mod.rs:3849); `Writer::cell_parts` (markdown/from_docx/mod.rs:3942)

#### `row_cells`  (L4164-4173)
`fn row_cells(row: &Element) -> impl Iterator<Item = &Element>`

- **Does:** _no doc comment_
- **Calls (crate):** `collect_named` (markdown/from_docx/mod.rs:4175) L4166
- **Calls (std/external/unresolved):** `.into_iter`, `Vec::new`
- **Called by:** `row_text_marked` (markdown/from_docx/agent.rs:901); `border_clauses` (markdown/from_docx/agent.rs:2204); `table_line` (markdown/from_docx/agent.rs:2378); `Writer::table` (markdown/from_docx/mod.rs:3689); `Writer::cell_styles` (markdown/from_docx/mod.rs:3754); `Writer::cell_spacings` (markdown/from_docx/mod.rs:3807); `Writer::column_alignments` (markdown/from_docx/mod.rs:3849); `Writer::cell_parts` (markdown/from_docx/mod.rs:3942)

#### `collect_named`  (L4175-4188)
`fn collect_named<'a>( container: &'a Element, name: &str, wrappers: &[&str], out: &mut Vec<&'a Element>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L4181; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L4182; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L4184; `collect_named` (markdown/from_docx/mod.rs:4175) L4185
- **Calls (std/external/unresolved):** `.contains`, `.push`
- **Called by:** `table_rows` (markdown/from_docx/mod.rs:4153); `row_cells` (markdown/from_docx/mod.rs:4164); `collect_named` (markdown/from_docx/mod.rs:4175)

#### `math`  (L4198-4200)
`fn math(element: &Element) -> String`

- **Does:** Office Math (OMML) to LaTeX, covering the structures Word's equation editor emits.
- **Calls (crate):** `math_space` (markdown/from_docx/mod.rs:4206) L4199; `math_inner` (markdown/from_docx/mod.rs:4216) L4199
- **Called by:** `coverage_real_source_route_matrix_tests::table_routes_distinguish_authored_math_content_from_missing_lexical_titles` (comparer/lcs.rs:20620) (?); `Writer::inline` (markdown/from_docx/mod.rs:3218); `tests::math_keeps_a_run_of_spaces_and_reads_a_line_break_as_one` (markdown/from_docx/mod.rs:4942)

#### `math_space`  (L4206-4212)
`fn math_space(latex: &str) -> String`

- **Does:** The equation's LaTeX with its edges trimmed.
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.map`, `.trim`
- **Macros:** `matches!`
- **Called by:** `math` (markdown/from_docx/mod.rs:4198)

#### `math_inner`  (L4216-4422)
`fn math_inner(element: &Element, depth: usize, name: bool) -> String`

- **Does:** The LaTeX of `element`; `name` while inside a function's name, whose words print as their command (`\sin`, `\operatorname{argmax}`).
- **Calls (crate):** `math_inner` (markdown/from_docx/mod.rs:4216) L4220,4226,4309; `Element::child` (markdown/from_docx/ooxml.rs:386) L4224,4307; `Element::path` (markdown/from_docx/ooxml.rs:391) L4230; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L4232,4283; `Element::local` (markdown/from_docx/ooxml.rs:318) L4235,4361; `math_run` (markdown/from_docx/mod.rs:4469) L4236; `limit_test_support::part` (admission.rs:2726) (by name) L4238,4246,4248,4249,4252,4255...; `horizontal_partition_tests::prop` (comparer/lcs_table.rs:863) (by name) L4239,4274,4275,4276,4314,4331...; `prime_count` (markdown/math.rs:1476) L4247; `base` (markdown/from_docx/mod.rs:4631) L4248,4249,4252,4255,4371; `group` (markdown/from_docx/mod.rs:4621) L4249,4252,4256,4257,4323,4326...; `Element::children_named` (markdown/from_docx/ooxml.rs:379) L4277,4379,4397; `structure` (markdown/from_docx/mod.rs:4460) L4281; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L4282,4395; `Element::path` (markdown/from_docx/ooxml.rs:391) (by name) L4283; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L4285,4311,4328; `fence` (markdown/from_docx/mod.rs:4582) L4300,4301,4302; `operand` (markdown/from_docx/mod.rs:4594) L4311,4328; `nary_command` (markdown/math.rs:527) L4317; `Element::children_named` (markdown/from_docx/ooxml.rs:379) (by name) L4382; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L4395,4400; `Element::elements` (markdown/from_docx/ooxml.rs:372) L4420
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.as_deref`, `.as_slice`, `.as_str`, `.chars`, `.clone`, `.collect`, `.concat`, `.ends_with`, `.into_iter`, `.into`, `.is_empty`, `.is_some_and`, `.join`, `.len`, `.map_or_else`, `.map`, `.next`, `.push_str`, `.repeat`, `.replace`, `.starts_with`, `.to_string`, `.trim_start`, `.trim`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `Some`, `String::new`, `half`, `inner`, `own_runs`
- **Macros:** `format!`
- **Called by:** `math` (markdown/from_docx/mod.rs:4198); `math_inner` (markdown/from_docx/mod.rs:4216); `operand` (markdown/from_docx/mod.rs:4594)

#### `whole_math_change`  (L4426-4448)
`fn whole_math_change(math: &Element) -> Option<&Element>`

- **Does:** The `w:ins` or `w:del` of math every run of which is inserted, or every run deleted.
- **Calls (crate):** `whole_math_change::{fn runs}` (markdown/from_docx/mod.rs:4427) L4437; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L4440; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L4440; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L4442
- **Calls (std/external/unresolved):** `.find`, `Some`, `Vec::new`
- **Called by:** `Writer::inline` (markdown/from_docx/mod.rs:3218)

#### `whole_math_change::{fn runs}`  (L4427-4435)
`fn runs<'e>(element: &'e Element, out: &mut Vec<&'e Element>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L4428; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L4429; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L4431; `whole_math_change::{fn runs}` (markdown/from_docx/mod.rs:4427) L4432
- **Calls (std/external/unresolved):** `.ends_with`, `.push`
- **Called by:** `whole_math_change` (markdown/from_docx/mod.rs:4426); `whole_math_change::{fn runs}` (markdown/from_docx/mod.rs:4427)

#### `strip_math_changes`  (L4451-4457)
`fn strip_math_changes(latex: &str) -> String`

- **Does:** `latex` without the marks of the changes inside it.
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.contains`, `.filter`
- **Called by:** `Writer::inline` (markdown/from_docx/mod.rs:3218)

#### `structure`  (L4460-4464)
`fn structure(e: &Element) -> Vec<&Element>`

- **Does:** The structures inside `e`, without its properties.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L4461; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L4462
- **Calls (std/external/unresolved):** `.collect`, `.ends_with`, `.filter`
- **Called by:** `horizontal_partition_tests::horizontal_row_and_cell_lifetimes_survive_with_format_tracking_disabled` (comparer/lcs_table.rs:1061) (?); `Transaction::resolve_comment_text` (edit.rs:3057) (?); `math_inner` (markdown/from_docx/mod.rs:4216); `operand` (markdown/from_docx/mod.rs:4594)

#### `math_run`  (L4469-4495)
`fn math_run(run: &Element, name: bool) -> String`

- **Does:** A math run's text as LaTeX: in a function's name, the name's command; else escaped and set in its style, inside CriticMarkup when the run is inserted or deleted.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L4472; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L4473; `.text` (ambiguous, 10 candidates) L4474,4485; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L4482; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L4484; `math_text` (markdown/from_docx/mod.rs:4498) L4490
- **Calls (std/external/unresolved):** `.filter`, `.for_each`, `.is_empty`, `.push_str`, `.replace`, `Some`, `String::new`
- **Macros:** `format!`
- **Called by:** `math_inner` (markdown/from_docx/mod.rs:4216)

#### `math_text`  (L4498-4540)
`fn math_text(run: &Element, text: &str, name: bool) -> String`

- **Does:** A math run's text, `text`, as LaTeX in the run's style.
- **Calls (crate):** `Element::path` (markdown/from_docx/ooxml.rs:391) L4503,4504,4505; `properties` (ambiguous, 5 candidates) L4507,4517,4521,4522; `escape_math` (markdown/from_docx/mod.rs:4544) L4514,4535; `escape_math_text` (markdown/from_docx/mod.rs:4566) L4518; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L4521,4522
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.chars`, `.contains`, `.is_empty`, `.is_none`, `.is_some`, `.or_else`, `.trim`, `Some`, `String::new`
- **Macros:** `format!`
- **Called by:** `math_run` (markdown/from_docx/mod.rs:4469)

#### `escape_math`  (L4544-4563)
`fn escape_math(text: &str) -> String`

- **Does:** Math text with the characters LaTeX acts on escaped; brackets, which would build a delimiter, as their commands.
- **Calls (std/external/unresolved):** `.chars`, `.len`, `.push_str`, `.push`, `String::with_capacity`
- **Called by:** `math_text` (markdown/from_docx/mod.rs:4498)

#### `escape_math_text`  (L4566-4579)
`fn escape_math_text(text: &str) -> String`

- **Does:** `\text{…}`'s text: braces, backslashes and `$` escaped.
- **Calls (std/external/unresolved):** `.chars`, `.len`, `.push_str`, `.push`, `String::with_capacity`
- **Called by:** `math_text` (markdown/from_docx/mod.rs:4498)

#### `fence`  (L4582-4590)
`fn fence(chr: &str) -> String`

- **Does:** A delimiter's character after `\left`, `\middle` or `\right`: none is `.`.
- **Calls (std/external/unresolved):** `.into`
- **Called by:** `tests::the_writer_unwinds_the_code_escape_inside_a_fence` (markdown/from_docx/critic.rs:2608); `tests::the_fence_escape_pair_is_exact` (markdown/from_docx/critic.rs:2640); `math_inner` (markdown/from_docx/mod.rs:4216)

#### `operand`  (L4594-4610)
`fn operand(e: Option<&Element>, depth: usize) -> String`

- **Does:** What a function or an n-ary operator applies to: one bracketed delimiter, one character after a space, anything else in braces.
- **Calls (crate):** `math_inner` (markdown/from_docx/mod.rs:4216) L4598; `math_chars` (markdown/from_docx/mod.rs:4613) L4600; `structure` (markdown/from_docx/mod.rs:4460) L4601; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L4601
- **Calls (std/external/unresolved):** `.as_slice`, `.into`, `.starts_with`, `.to_string`, `.trim`
- **Macros:** `format!`, `matches!`
- **Called by:** `math_inner` (markdown/from_docx/mod.rs:4216)

#### `math_chars`  (L4613-4619)
`fn math_chars(latex: &str) -> usize`

- **Does:** The characters of math LaTeX, without the marks of changes inside it.
- **Calls (std/external/unresolved):** `.chars`, `.contains`, `.count`, `.filter`
- **Called by:** `operand` (markdown/from_docx/mod.rs:4594); `group` (markdown/from_docx/mod.rs:4621)

#### `group`  (L4621-4628)
`fn group(s: &str) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `math_chars` (markdown/from_docx/mod.rs:4613) L4623
- **Calls (std/external/unresolved):** `.to_string`, `.trim`
- **Macros:** `format!`
- **Called by:** `math_inner` (markdown/from_docx/mod.rs:4216); `base` (markdown/from_docx/mod.rs:4631)

#### `base`  (L4631-4638)
`fn base(s: &str) -> String`

- **Does:** The base of a script or limit: a function's command bare, else a group.
- **Calls (crate):** `ChangeFilter::matches` (changes.rs:405) (by name) L4635; `group` (markdown/from_docx/mod.rs:4621) L4637
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.count`, `.is_ascii_alphabetic`, `.is_some_and`, `.starts_with`, `.strip_prefix`, `.to_string`, `.trim`
- **Called by:** `tests::recording_refuses_what_the_plan_names_differently` (edit/flags.rs:964) (?); `math_inner` (markdown/from_docx/mod.rs:4216); `tests::decoded_part_name_aliases_are_refused_not_overwritten` (opc/mod.rs:1783) (?)

#### `hyperlink_instruction`  (L4642-4669)
`fn hyperlink_instruction(instruction: &str) -> Option<String>`

- **Does:** The link of a `HYPERLINK` field instruction: its URL, `#bookmark` for `\l "bookmark"`, both (`url#bookmark`), titled with `\o`'s tooltip.
- **Calls (crate):** `field_words` (markdown/from_docx/mod.rs:4769) L4643; `link_target` (markdown/from_docx/mod.rs:4755) L4668; `titled` (markdown/from_docx/ooxml.rs:290) L4668
- **Calls (std/external/unresolved):** `.as_str`, `.eq_ignore_ascii_case`, `.flatten`, `.get_or_insert`, `.iter`, `.map`, `.next`, `.split_first`, `.starts_with`, `.then`, `.to_ascii_lowercase`
- **Macros:** `matches!`
- **Called by:** `Writer::inline` (markdown/from_docx/mod.rs:3218); `Writer::run_child` (markdown/from_docx/mod.rs:3437); `bookmark_targets` (markdown/from_docx/mod.rs:4805); `tests::a_link_to_a_bookmark_is_a_markdown_link_and_its_bookmark_an_anchor` (markdown/from_docx/mod.rs:5465); `markdown_source_owner_boundary_tests::numbering_zero_and_out_of_range_overrides_do_not_change_source_order` (markdown/from_docx/mod.rs:6896)

#### `view_table_style`  (L4676-4682)
`fn view_table_style(t: &Element, custom_grid: bool) -> Option<&str>`

- **Does:** The table style the view names for `t`.
- **Calls (crate):** `Element::path` (markdown/from_docx/ooxml.rs:391) L4677; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L4677
- **Calls (std/external/unresolved):** `.and_then`, `Some`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2553); `table_styles` (markdown/from_docx/mod.rs:4720)

#### `custom_table_grid`  (L4689-4716)
`fn custom_table_grid(styles: Option<&Element>) -> bool`

- **Does:** Whether `styles` defines Table Grid with borders other than Word's thin grid (one single 0.5pt line on every side): a grid the legend names (`TableGrid, all borders 1.5pt`, `TableGrid, borders top double 3pt`) and the writer would otherwise replace with the thin one.
- **Calls (crate):** `Element::children_named` (markdown/from_docx/ooxml.rs:379) (by name) L4692; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L4693,4713; `Element::path` (markdown/from_docx/ooxml.rs:391) (by name) L4695; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L4708; `side` (markdown/borders.rs:189) L4713,4715
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.find_map`, `.find`, `.is_some_and`, `.is_some`, `.iter`, `Some`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `table_styles`  (L4720-4734)
`fn table_styles(body: &Element, custom_grid: bool) -> (Vec<String>, bool)`

- **Does:** The table styles of `body`'s tables the view names, in the order they first come, and whether a table has none (the writer's Table Grid).
- **Calls (crate):** `tables` (markdown/from_docx/agent.rs:1890) L4723; `view_table_style` (markdown/from_docx/mod.rs:4676) L4724
- **Calls (std/external/unresolved):** `.any`, `.iter`, `.push`, `.to_string`, `Vec::new`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `shown_text`  (L4738-4751)
`fn shown_text(p: &Element) -> String`

- **Does:** The text of `p` the accepted document shows: its `w:t`, not deleted or moved-away text.
- **Calls (crate):** `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1656) L4749
- **Calls (std/external/unresolved):** `String::new`
- **Called by:** `own_slug` (markdown/from_docx/mod.rs:1708)

#### `shown_text::{fn walk}`  (L4739-4747)
`fn walk(e: &Element, out: &mut String)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L4740; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L4741; `.text` (ambiguous, 10 candidates) L4743; `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1656) L4744
- **Calls (std/external/unresolved):** `.push_str`
- **Called by:** `text_follows_reference` (markdown/from_docx/mod.rs:1650); `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1656); `body_headings` (markdown/from_docx/mod.rs:1690); `body_headings::{fn walk}` (markdown/from_docx/mod.rs:1691); `closing_fields` (markdown/from_docx/mod.rs:2153); `closing_fields::{fn walk}` (markdown/from_docx/mod.rs:2157); `spanning_fields` (markdown/from_docx/mod.rs:2204); `spanning_fields::{fn walk}` (markdown/from_docx/mod.rs:2208); `held_marks` (markdown/from_docx/mod.rs:2270); `held_marks::{fn walk}` (markdown/from_docx/mod.rs:2271); `Writer::open_after` (markdown/from_docx/mod.rs:3045); `Writer::open_after::{fn walk}` (markdown/from_docx/mod.rs:3046); +4 more

#### `link_target`  (L4755-4765)
`fn link_target(url: Option<&str>, anchor: Option<&str>) -> Option<String>`

- **Does:** A link's destination from its URL and its bookmark: `url`, `#bookmark`, or `url#bookmark`.
- **Calls (std/external/unresolved):** `.filter`, `.is_empty`, `.to_string`, `Some`
- **Macros:** `format!`
- **Called by:** `Writer::inline` (markdown/from_docx/mod.rs:3218); `hyperlink_instruction` (markdown/from_docx/mod.rs:4642)

#### `field_words`  (L4769-4799)
`fn field_words(instruction: &str) -> Vec<(String, bool)>`

- **Does:** A field instruction's words, each with whether it was quoted: `"a b"` is one word, and `\"` inside quotes is a quote.
- **Calls (crate):** `Parser::peek` (markdown/math.rs:804) (by name) L4772,4782,4791
- **Calls (std/external/unresolved):** `.chars`, `.extend`, `.filter`, `.is_whitespace`, `.next`, `.peekable`, `.push`, `Some`, `String::new`, `Vec::new`
- **Macros:** `matches!`
- **Called by:** `hyperlink_instruction` (markdown/from_docx/mod.rs:4642); `bookmark_targets` (markdown/from_docx/mod.rs:4805)

#### `bookmark_targets`  (L4805-4873)
`fn bookmark_targets<'e>( roots: impl Iterator<Item = &'e Element>, fields_print: bool, ) -> HashSet<String>`

- **Does:** The bookmarks under `roots` a link (`w:hyperlink w:anchor`, `HYPERLINK \l`) or a reference field (`REF`, `PAGEREF`, `NOTEREF`) names.
- **Calls (crate):** `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1656) L4844; `field_words` (markdown/from_docx/mod.rs:4769) L4847; `hyperlink_instruction` (markdown/from_docx/mod.rs:4642) L4854
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.first`, `.get`, `.insert`, `.map`, `.next`, `.split_once`, `.split`, `.to_ascii_uppercase`, `.to_string`, `.unwrap_or`, `HashSet::new`, `Vec::new`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `bookmark_targets::{fn walk}`  (L4809-4840)
`fn walk(e: &Element, refs: bool, codes: &mut Vec<String>, out: &mut HashSet<String>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L4810; `.local` (ambiguous: `Build::local` (debug/diff.rs:491); `Element::local` (markdown/from_docx/ooxml.rs:318)) L4811; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L4813,4819,4822,4833; `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1656) L4816,4820,4837; `.text` (ambiguous, 10 candidates) L4825
- **Calls (std/external/unresolved):** `.filter`, `.insert`, `.is_empty`, `.last_mut`, `.push_str`, `.push`, `.starts_with`, `.to_string`, `.unwrap_or_default`, `Some`, `String::new`
- **Called by:** `text_follows_reference` (markdown/from_docx/mod.rs:1650); `text_follows_reference::{fn walk}` (markdown/from_docx/mod.rs:1656); `body_headings` (markdown/from_docx/mod.rs:1690); `body_headings::{fn walk}` (markdown/from_docx/mod.rs:1691); `closing_fields` (markdown/from_docx/mod.rs:2153); `closing_fields::{fn walk}` (markdown/from_docx/mod.rs:2157); `spanning_fields` (markdown/from_docx/mod.rs:2204); `spanning_fields::{fn walk}` (markdown/from_docx/mod.rs:2208); `held_marks` (markdown/from_docx/mod.rs:2270); `held_marks::{fn walk}` (markdown/from_docx/mod.rs:2271); `Writer::open_after` (markdown/from_docx/mod.rs:3045); `Writer::open_after::{fn walk}` (markdown/from_docx/mod.rs:3046); +4 more

#### `story_frame`  (L4880-4912)
`fn story_frame(p: &Element, styles: Option<&Element>, default: Option<&str>) -> Option<String>`

- **Does:** The colour every text run of a header or footer paragraph `p` sets itself (`w:rPr/w:color`), when they all set the same one.
- **Calls (crate):** `Element::path` (markdown/from_docx/ooxml.rs:391) L4883,4909; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L4885,4895,4904; `is_paragraph_style` (markdown/from_docx/header.rs:913) L4895; `Element::path` (markdown/from_docx/ooxml.rs:391) (by name) L4899; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L4902; `merged` (markdown/frames.rs:82) L4910; `describe` (markdown/frames.rs:107) L4911
- **Calls (std/external/unresolved):** `.and_then`, `.as_slice`, `.as_str`, `.children_named`, `.clone`, `.find`, `.insert`, `.map`, `.or`, `.push`, `.reverse`, `.take`, `HashSet::new`, `Some`, `Vec::new`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `story_color`  (L4914-4923)
`fn story_color(p: &Element) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::find_all` (markdown/from_docx/ooxml.rs:404) L4916; `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L4919; `Element::path` (markdown/from_docx/ooxml.rs:391) (by name) L4920; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L4920
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.filter`, `.is_some`, `.iter`, `.map`, `.next`, `.then`, `.to_string`, `Some`, `Vec::new`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

## Test functions (86)

- `tests::a_break_inside_an_equation_stays_a_newline` (L4934): _no doc comment_
- `tests::math_keeps_a_run_of_spaces_and_reads_a_line_break_as_one` (L4942): _no doc comment_
- `tests::a_name_loses_the_xml_indentation_and_keeps_its_spaces` (L4966): _no doc comment_
- `tests::zip` (L4974): _no doc comment_
- `tests::docx` (L4985): _no doc comment_
- `tests::md` (L5000): _no doc comment_
- `tests::payloads_elided` (L5008): `markdown` with each drawing definition's payload elided: the expectation names the reference (`[d1]: drawing:…`) without quoting the base64 the payload is.
- `tests::p` (L5028): _no doc comment_
- `tests::r` (L5037): _no doc comment_
- `tests::headings_emphasis_links_and_title_style` (L5042): _no doc comment_
- `tests::numbered_and_bulleted_lists` (L5078): _no doc comment_
- `tests::tables_with_spans_and_nested_tables` (L5111): _no doc comment_
- `tests::subscript_and_superscript_runs_keep_their_tags_through_a_round_trip` (L5136): _no doc comment_
- `tests::a_table_cell_keeps_its_equations_latex` (L5171): _no doc comment_
- `tests::an_agent_view_cell_keeps_an_equations_norm_bars` (L5182): _no doc comment_
- `tests::monospace_faces_are_named_by_whole_words` (L5196): _no doc comment_
- `tests::office_math_structures_become_latex` (L5219): _no doc comment_
- `tests::footnotes_images_and_fields` (L5422): _no doc comment_
- `tests::a_link_to_a_bookmark_is_a_markdown_link_and_its_bookmark_an_anchor` (L5465): _no doc comment_
- `tests::a_cross_reference_target_prints_its_anchor_with_nothing_naming_it` (L5507): _no doc comment_
- `tests::image_names_keep_delimiter_characters_out_of_the_markup` (L5520): _no doc comment_
- `tests::tracked_changes_become_critic_markup_without_losing_formatting` (L5548): _no doc comment_
- `tests::tracked_whole_paragraphs_keep_markdown_block_structure` (L5570): _no doc comment_
- `tests::tracked_table_rows_and_cells_keep_the_table_valid` (L5584): _no doc comment_
- `tests::ins` (L5602): Tracked-change building blocks, written the way Word writes them. _(from a code comment)_
- `tests::del` (L5608): _no doc comment_
- `tests::dr` (L5614): _no doc comment_
- `tests::pm` (L5620): A paragraph whose mark (the break after it) carries `mark` (`ins`, `del`, `moveFrom`, `moveTo`, or several), with an optional style.
- `tests::text_box` (L5633): _no doc comment_
- `tests::review_deleted_row_with_deleted_runs_is_marked_once` (L5640): _no doc comment_
- `tests::review_text_box_inside_a_tracked_change_keeps_the_change` (L5654): _no doc comment_
- `tests::review_tracked_paragraph_marks_mark_the_break` (L5673): _no doc comment_
- `tests::review_footnote_revisions_are_marked` (L5698): _no doc comment_
- `tests::comments` (L5720): `word/comments.xml` with one `w:comment` per entry: (id, author, date, paragraphs).
- `tests::start` (L5740): _no doc comment_
- `tests::end` (L5744): _no doc comment_
- `tests::reference` (L5749): The reference run Word writes right after a comment's range end.
- `tests::with_comments` (L5755): _no doc comment_
- `tests::comment_highlights_its_range_and_names_author_and_date_as_word_stores_them` (L5762): _no doc comment_
- `tests::comment_date_is_copied_verbatim_whatever_its_form` (L5783): _no doc comment_
- `tests::comment_note_leaves_out_what_is_missing` (L5800): _no doc comment_
- `tests::comment_without_a_range_is_a_plain_comment_at_its_reference` (L5820): _no doc comment_
- `tests::replies_and_overlapping_ranges_share_one_highlight` (L5838): _no doc comment_
- `tests::comment_range_across_paragraphs_highlights_each_paragraph` (L5870): _no doc comment_
- `tests::comment_ranges_at_block_level_and_in_tables` (L5888): _no doc comment_
- `tests::comment_without_a_reference_gets_its_note_at_the_range_end` (L5905): _no doc comment_
- `tests::unknown_or_malformed_comments_are_ignored` (L5915): _no doc comment_
- `tests::comment_text_keeps_formatting_joins_paragraphs_and_escapes_delimiters` (L5961): _no doc comment_
- `tests::comments_meet_tracked_changes_without_breaking_either` (L5978): _no doc comment_
- `tests::comment_range_around_a_text_box_does_not_cover_the_box` (L6025): _no doc comment_
- `tests::comment_part_is_found_through_the_document_relationships` (L6048): _no doc comment_
- `tests::comments_in_footnotes_and_ranges_left_open_before_them` (L6063): _no doc comment_
- `tests::substitutions_merges_and_empty_changes` (L6103): _no doc comment_
- `tests::delimiters_in_document_text_do_not_open_spans` (L6125): _no doc comment_
- `tests::tracked_changes_with_links_fields_and_hidden_runs` (L6139): _no doc comment_
- `tests::tracked_breaks_keep_headings_lists_and_blocks_valid` (L6157): _no doc comment_
- `tests::tracked_cells_rows_and_nested_tables` (L6204): _no doc comment_
- `tests::tracked_blocks_inside_content_controls` (L6223): _no doc comment_
- `tests::tracked_changes_inside_inline_wrappers_and_styled_breaks` (L6236): _no doc comment_
- `tests::comment_without_an_author_shows_only_its_date` (L6264): _no doc comment_
- `tests::text_boxes_and_content_controls_inside_tracked_cells` (L6274): _no doc comment_
- `tests::deleted_paragraph_mark_preserves_text_and_marks_only_the_separator` (L6292): _no doc comment_
- `tests::deleted_text_boxes_keep_block_structure_without_an_empty_paragraph` (L6300): _no doc comment_
- `tests::deleted_row_with_inline_deletion_has_no_nested_delimiters` (L6345): _no doc comment_
- `tests::by` (L6361): A revision element with an explicit author and date (either may be empty to leave the attribute out).
- `tests::every_tracked_change_names_its_author_and_date_as_word_stores_them` (L6376): _no doc comment_
- `tests::paragraph_breaks_rows_and_cells_carry_their_own_attribution` (L6433): _no doc comment_
- `tests::footnotes_in_comment_text_are_numbered_where_the_comment_is` (L6464): _no doc comment_
- `tests::a_range_ending_between_paragraphs_keeps_its_note` (L6500): _no doc comment_
- `tests::a_note_between_paragraph_and_table_stays_out_of_the_table` (L6520): _no doc comment_
- `tests::comment_text_keeps_text_boxes_without_nesting_comments` (L6547): _no doc comment_
- `tests::a_comment_used_twice_is_written_once_and_its_footnote_counted_once` (L6575): _no doc comment_
- `tests::resolved` (L6606): _no doc comment_
- `tests::accept_and_reject_resolve_paragraphs_breaks_and_moves` (L6615): _no doc comment_
- `tests::a_break_that_goes_away_joins_across_empty_markers` (L6638): _no doc comment_
- `tests::a_joined_paragraph_takes_the_later_paragraphs_properties` (L6665): _no doc comment_
- `tests::accept_and_reject_resolve_rows_cells_and_nested_tables` (L6677): _no doc comment_
- `tests::accept_and_reject_resolve_text_boxes_and_footnotes` (L6703): _no doc comment_
- `tests::reject_restores_formatting_that_a_tracked_change_replaced` (L6739): _no doc comment_
- `tests::accept_and_reject_leave_comments_out` (L6755): _no doc comment_
- `tests::a_note_whose_reference_is_changed_is_changed_too` (L6768): _no doc comment_
- `tests::malformed_input_is_invalid_not_panic` (L6800): _no doc comment_
- `markdown_source_owner_boundary_tests::docx` (L6821): _no doc comment_
- `markdown_source_owner_boundary_tests::style_title_outline_and_long_ancestry_follow_authored_nearest_properties` (L6831): _no doc comment_
- `markdown_source_owner_boundary_tests::hidden_runs_cannot_leak_text_symbols_note_links_or_comment_markers` (L6868): _no doc comment_
- `markdown_source_owner_boundary_tests::numbering_zero_and_out_of_range_overrides_do_not_change_source_order` (L6896): _no doc comment_
