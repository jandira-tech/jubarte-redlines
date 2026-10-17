# Functions and methods in `src/markdown/xml.rs`

47 non-test, 6 test. Part of the [symbol manifest](../README.md).

#### `Document::section`  (L386-396, pub(super))
`pub(super) fn section(&self, number: usize, fallback: &str) -> String`

- **Does:** The `w:sectPr` of section `number` (from 1) as the view names its page: its own, else the first section's, else `fallback`.
- **Calls (crate):** `PageSetup::sect_pr` (markdown/view_notes.rs:1991) (by name) L391; `with_section_change` (markdown/xml.rs:2435) L393
- **Calls (std/external/unresolved):** `.get`, `.map_or_else`, `.or_else`, `.to_string`
- **Called by:** `assemble` (markdown/package.rs:97); `paragraph_properties` (markdown/xml.rs:1699)

#### `Document::section_breaks`  (L399-404, pub(super))
`pub(super) fn section_breaks(&self) -> usize`

- **Does:** How many paragraphs end a section.
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.iter`, `Item::Paragraph`
- **Macros:** `matches!`
- **Called by:** `assemble` (markdown/package.rs:97)

#### `escape`  (L426-451, pub(super))
`pub(super) fn escape(text: &str) -> Cow<'_, str>`

- **Does:** Text for XML content or an attribute: markup escaped, characters XML 1.0 forbids dropped.
- **Calls (crate):** `nested_paragraph_property_source_history_tests::clean` (comparer/formatchg.rs:788) (by name) L433,438
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.filter`, `.len`, `.push_str`, `.push`, `Cow::Borrowed`, `Cow::Owned`, `String::with_capacity`
- **Macros:** `matches!`
- **Called by:** `serialize::{fn put}` (markdown/from_docx/drawing.rs:386); `Writer::inline` (markdown/from_docx/mod.rs:3021); `Writer::run` (markdown/math.rs:1698); `Writer::node` (markdown/math.rs:1734); `assemble` (markdown/package.rs:97); `renamed` (markdown/package.rs:507); `with_table_styles` (markdown/package.rs:537); `built_in_table_style` (markdown/package.rs:593); `with_named_styles` (markdown/package.rs:881); `Marks::mark_id` (markdown/view_stories.rs:152); `runs` (markdown/view_stories.rs:612); `flush` (markdown/view_stories.rs:755); +17 more

#### `Relate::hyperlink`  (L456-456, trait decl, no body)
`fn hyperlink(&mut self, url: &str) -> String`

- **Does:** The relationship id of an external hyperlink from this story's part.
- **Calls:** none
- **Called by:** `story` (markdown/xml.rs:587)

#### `Relate::picture`  (L458-458, trait decl, no body)
`fn picture(&mut self, index: usize) -> String`

- **Does:** The relationship id of a picture from this story's part.
- **Calls:** none
- **Called by:** `write_run` (markdown/xml.rs:1854)

#### `Relate::drawing`  (L462-462, trait decl, no body)
`fn drawing(&mut self, index: usize, next_doc_pr: &mut u32) -> Option<String>`

- **Does:** The XML of drawing `index` written into the package beside this story's part: its relationships and dependent parts too.
- **Calls:** none
- **Called by:** `write_run` (markdown/xml.rs:1854)

#### `Context::revision_id`  (L512-534)
`fn revision_id(&mut self, by: Option<u32>) -> u32`

- **Does:** A `w:ins` or `w:del` start tag's name and attributes, with the author and date change `by` names, or the document's.
- **Calls (std/external/unresolved):** `.entry`, `.get`, `.insert`, `.or_insert`
- **Called by:** `Context::format_mark` (markdown/xml.rs:538); `Context::revision` (markdown/xml.rs:557); `story` (markdown/xml.rs:587)

#### `Context::format_mark`  (L538-548)
`fn format_mark(&mut self, name: &str, mark: &FormatMark) -> String`

- **Does:** A formatting change's start tag: `name` with its id (the view's, unless a mark took it) and its author and date.
- **Calls (crate):** `Context::revision_id` (markdown/xml.rs:512) L542; `escape` (markdown/xml.rs:426) L545; `date_attribute` (markdown/xml.rs:2464) L546
- **Calls (std/external/unresolved):** `.filter`, `.insert`, `.unwrap_or_else`
- **Macros:** `format!`
- **Called by:** `story` (markdown/xml.rs:587); `paragraph_properties` (markdown/xml.rs:1699)

#### `Context::ids_left`  (L552-555)
`fn ids_left(&self, by: u32) -> usize`

- **Does:** How many of the ids the view names for change `by` no mark has taken yet.
- **Calls (std/external/unresolved):** `.copied`, `.get`, `.map_or`, `.saturating_sub`, `.unwrap_or`
- **Called by:** `Context::revision` (markdown/xml.rs:557); `story` (markdown/xml.rs:587); `close_mark` (markdown/xml.rs:1490); `split_for_ids` (markdown/xml.rs:1608); `paragraph_properties` (markdown/xml.rs:1699)

#### `Context::revision`  (L557-577)
`fn revision(&mut self, kind: Kind, by: Option<u32>) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Context::revision_id` (markdown/xml.rs:512) L558; `Context::ids_left` (markdown/xml.rs:552) L564; `escape` (markdown/xml.rs:426) L569; `date_attribute` (markdown/xml.rs:2464) L570
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.get`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `story` (markdown/xml.rs:587); `close_mark` (markdown/xml.rs:1490); `paragraph_properties` (markdown/xml.rs:1699); `box_xml` (markdown/xml.rs:1949)

#### `Context::highlighted`  (L579-582)
`fn highlighted(&self, run: &Run) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`, `.is_some_and`
- **Called by:** `run_properties` (markdown/xml.rs:1806)

#### `story`  (L587-1275, pub(super))
`pub(super) fn story( context: &mut Context<'_>, items: &[Item], relate: &mut dyn Relate, note: Option<Option<Change>>, ) -> String`

- **Does:** Writes a story's items.
- **Calls (crate):** `split_for_ids` (markdown/xml.rs:1608) L593; `natural_marks` (markdown/xml.rs:1536) L615; `starts_inserted_side` (markdown/xml.rs:1455) L634; `close_mark` (markdown/xml.rs:1490) L637,684,697,700,873,883...; `spend` (markdown/xml.rs:1693) L639,705,748,783,908,924...; `Context::revision` (markdown/xml.rs:557) L640,706,753,816,833,909...; `field_link` (markdown/xml.rs:1283) L651; `note_change` (markdown/xml.rs:1322) L655; `runs_ahead` (markdown/xml.rs:1428) L663; `ends_inserted_side` (markdown/xml.rs:1479) L668; `continues` (markdown/xml.rs:1337) L676; `close_marks` (markdown/xml.rs:1520) L682,704,827,929,1273; `continues_change` (markdown/xml.rs:1377) L691,878,917; `escape` (markdown/xml.rs:426) L718,741,1030,1041,1052,1071; `paragraph_properties` (markdown/xml.rs:1699) L723; `Context::format_mark` (markdown/xml.rs:538) L729,802,812,1090,1186,1190...; `text_box_xml` (markdown/xml.rs:1930) L730; `omml` (markdown/math.rs:1923) L836; `named` (ambiguous: `coverage_round_three_tests::coverage_round_five_tests::named` (convert/mod.rs:50359); `named` (markdown/from_docx/header.rs:941); `named` (markdown/write.rs:5387)) L854; `Context::ids_left` (markdown/xml.rs:552) L855,958; `write_run` (markdown/xml.rs:1854) L889,977,979; `resumes_later` (markdown/xml.rs:1435) L895; `resumes_after` (markdown/xml.rs:1392) L905; `field_argument` (markdown/xml.rs:1474) L1013,1015,1019; `Relate::hyperlink` (markdown/xml.rs:456) L1045; `xml` (markdown/borders.rs:254) L1081,1130,1224; `Context::revision_id` (markdown/xml.rs:512) L1108; `CellsAt::holds` (markdown/view_notes.rs:561) (by name) L1223
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.as_deref`, `.as_ref`, `.as_slice`, `.clone_from`, `.clone`, `.cloned`, `.contains`, `.copied`, `.enumerate`, `.filter`, `.find_map`, `.find`, `.flatten`, `.get`, `.insert_str`, `.insert`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.last_mut`, `.last`, `.len`, `.map_or`, `.map`, `.max`, `.pop`, `.push_str`, `.push`, `.rev`, `.rfind`, `.rposition`, `.saturating_sub`, `.take`, `.to_string`, `.to_vec`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `Item::Bookmark`, `Item::CommentReference`, `Item::RangeEnd`, `Item::RangeStart`, `Some`, `String::new`, `Vec::new`, `std::mem::take`, `u32::try_from`, `width`
- **Macros:** `format!`, `matches!`, `write!`
- **Called by:** `tests::a_page_break_stays_inside_its_paragraph_and_a_section_mark_ends_it` (legacy_doc.rs:1919); `assemble` (markdown/package.rs:97); `footnotes` (markdown/package.rs:1147)

#### `field_link`  (L1283-1318)
`fn field_link(rest: &[Item], document: &Document) -> Option<Change>`

- **Does:** Whether the next run after a comment's marks continues the open change.
- **Calls (std/external/unresolved):** `.filter`, `.get`, `.is_none_or`, `.len`, `HashSet::::from_iter`, `Some`
- **Called by:** `story` (markdown/xml.rs:587); `natural_marks` (markdown/xml.rs:1536)

#### `note_change`  (L1322-1331)
`fn note_change(notes: &HashMap<u32, (Change, bool)>, item: &Item) -> Option<Change>`

- **Does:** The change a comment's reference, or its range end, sits in, when the view put the comment's note inside one (see `Document::note_changes`).
- **Calls (std/external/unresolved):** `.filter`, `.get`, `.map`
- **Called by:** `story` (markdown/xml.rs:587); `natural_marks` (markdown/xml.rs:1536)

#### `continues`  (L1337-1371)
`fn continues(document: &Document, rest: &[Item], open: Option<Change>, marks: &[Change]) -> bool`

- **Does:** Whether the next real item in `rest` is a run still inside a mark: `open`'s own, one a nested change suspended (`marks`), whose other pieces name the same ids, or a change `open` resumes after, which the open mark then holds.
- **Calls (crate):** `continues_change` (markdown/xml.rs:1377) L1356,1363; `resumes_after` (markdown/xml.rs:1392) L1369
- **Calls (std/external/unresolved):** `.any`, `.chain`, `.copied`, `.get`, `.into_iter`, `.is_some_and`, `.iter`, `.position`, `Item::Bookmark`, `Item::CommentReference`, `Item::RangeEnd`, `Item::RangeStart`, `Some`
- **Macros:** `matches!`
- **Called by:** `distribute` (markdown/write.rs:242) (?); `story` (markdown/xml.rs:587); `natural_marks` (markdown/xml.rs:1536)

#### `continues_change`  (L1377-1387)
`fn continues_change(document: &Document, inner: Change, outer: Change) -> bool`

- **Does:** Whether `inner` is another piece of `outer`'s change: the view names its `rN` ids (`{++a++}{>>r1 @AL<<}…{++b++}{>>r1 @AL<<}`), which `outer` shares.
- **Calls (std/external/unresolved):** `.all`, `.contains`, `.get`, `.is_empty`, `.is_some_and`, `.iter`
- **Called by:** `story` (markdown/xml.rs:587); `continues` (markdown/xml.rs:1337); `resumes_after` (markdown/xml.rs:1392); `resumes_later` (markdown/xml.rs:1435)

#### `resumes_after`  (L1392-1425)
`fn resumes_after(document: &Document, rest: &[Item], inner: Change, outer: Change) -> bool`

- **Does:** Whether `outer`'s change has a piece past `inner`'s group: then `inner` sits inside `outer`'s mark, which stays open around it.
- **Calls (crate):** `continues_change` (markdown/xml.rs:1377) L1409,1413
- **Calls (std/external/unresolved):** `.iter`, `.push`, `.rposition`, `.truncate`
- **Macros:** `vec!`
- **Called by:** `story` (markdown/xml.rs:587); `continues` (markdown/xml.rs:1337)

#### `runs_ahead`  (L1428-1430)
`fn runs_ahead(rest: &[Item]) -> bool`

- **Does:** Whether any run is left in `rest`.
- **Calls (std/external/unresolved):** `.any`, `.iter`, `Item::Run`
- **Macros:** `matches!`
- **Called by:** `story` (markdown/xml.rs:587)

#### `resumes_later`  (L1435-1448)
`fn resumes_later(document: &Document, rest: &[Item], outer: Change) -> bool`

- **Does:** Whether `outer`'s change holds another change ahead in `rest`: a run of a piece of it is still to come, past the nested ones between, so its mark stays open around them as around the piece it just resumed.
- **Calls (crate):** `continues_change` (markdown/xml.rs:1377) L1445
- **Calls (std/external/unresolved):** `.any`, `.is_some_and`, `.iter`
- **Called by:** `story` (markdown/xml.rs:587)

#### `starts_inserted_side`  (L1455-1470)
`fn starts_inserted_side(rest: &[Item]) -> Option<Change>`

- **Does:** Ends the open mark.
- **Calls (crate):** `ends_inserted_side` (markdown/xml.rs:1479) L1468
- **Calls (std/external/unresolved):** `.count`, `.first`, `.get`, `.iter`, `.take_while`, `.then_some`, `Item::RangeEnd`, `Item::Run`, `Some`
- **Macros:** `matches!`
- **Called by:** `story` (markdown/xml.rs:587)

#### `field_argument`  (L1474-1476)
`fn field_argument(text: &str) -> String`

- **Does:** `text` as a quoted field-code argument: Word reads `\"` as a quote and `\\` as a backslash there.
- **Calls (std/external/unresolved):** `.replace`
- **Called by:** `story` (markdown/xml.rs:587)

#### `ends_inserted_side`  (L1479-1488)
`fn ends_inserted_side(rest: &[Item], open: Option<Change>) -> bool`

- **Does:** Whether `open` is an insertion and the next item a deleted run.
- **Calls (std/external/unresolved):** `.first`, `Item::Run`, `Some`
- **Macros:** `matches!`
- **Called by:** `story` (markdown/xml.rs:587); `starts_inserted_side` (markdown/xml.rs:1455)

#### `close_mark`  (L1490-1516)
`fn close_mark( context: &mut Context<'_>, natural: &HashMap<u32, usize>, out: &mut String, open: &mut Option<Change>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Context::ids_left` (markdown/xml.rs:552) L1511; `Context::revision` (markdown/xml.rs:557) L1512
- **Calls (std/external/unresolved):** `.copied`, `.count`, `.filter`, `.flatten`, `.get`, `.iter`, `.push_str`, `.saturating_sub`, `.take`, `.unwrap_or`, `Some`
- **Macros:** `write!`
- **Called by:** `story` (markdown/xml.rs:587); `close_marks` (markdown/xml.rs:1520)

#### `close_marks`  (L1520-1532)
`fn close_marks( context: &mut Context<'_>, natural: &HashMap<u32, usize>, out: &mut String, open: &mut Option<Change>, marks: &mut Vec<Change>, )`

- **Does:** [`close_mark`] for the open mark and every mark a nested change's mark suspended, innermost first.
- **Calls (crate):** `close_mark` (markdown/xml.rs:1490) L1527,1530
- **Calls (std/external/unresolved):** `.pop`, `Some`
- **Called by:** `story` (markdown/xml.rs:587)

#### `natural_marks`  (L1536-1602)
`fn natural_marks(document: &Document, items: &[Item]) -> HashMap<u32, usize>`

- **Does:** The marks each numbered change opens in `items` when no change splits: a run that does not continue the open mark, a paragraph mark, a row.
- **Calls (crate):** `note_change` (markdown/xml.rs:1322) L1542; `field_link` (markdown/xml.rs:1283) L1552; `continues` (markdown/xml.rs:1337) L1570
- **Calls (std/external/unresolved):** `.entry`, `.enumerate`, `.flatten`, `.is_none`, `.is_some`, `.iter`, `.or_default`, `.pop`, `.push`, `HashMap::new`, `Some`, `Vec::new`
- **Called by:** `story` (markdown/xml.rs:587)

#### `split_for_ids`  (L1608-1675)
`fn split_for_ids<'i>(context: &Context<'_>, items: &'i [Item]) -> Cow<'i, [Item]>`

- **Does:** `items` with text runs split until each change has a piece for every id the view names it: `{++one two++}{>>r70+71 @AL<<}` is two marks, `one ` and `two`.
- **Calls (crate):** `Context::ids_left` (markdown/xml.rs:552) L1633; `split_point` (markdown/xml.rs:1679) L1653
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.entry`, `.enumerate`, `.filter_map`, `.find_map`, `.insert`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.map`, `.or_default`, `.rev`, `.sort_unstable`, `.split_at`, `.then`, `.to_string`, `.to_vec`, `Content::Text`, `Cow::Borrowed`, `Cow::Owned`, `HashMap::new`, `Item::Run`
- **Called by:** `story` (markdown/xml.rs:587)

#### `split_point`  (L1679-1690, pub(super))
`pub(super) fn split_point(text: &str) -> Option<usize>`

- **Does:** Where a run's text splits in two: after its last space with text after it, else before its last character; `None` for one character.
- **Calls (std/external/unresolved):** `.char_indices`, `.filter`, `.find`, `.last`, `.len`, `.map`, `.or_else`, `.rev`
- **Called by:** `tracked` (markdown/view_stories.rs:814); `split_for_ids` (markdown/xml.rs:1608); `tests::a_run_splits_after_its_last_inner_space_else_before_its_last_character` (markdown/xml.rs:2478)

#### `spend`  (L1693-1697)
`fn spend(natural: &mut HashMap<u32, usize>, by: u32)`

- **Does:** One natural mark of change `by` is written.
- **Calls (std/external/unresolved):** `.get_mut`, `.saturating_sub`
- **Called by:** `story` (markdown/xml.rs:587)

#### `paragraph_properties`  (L1699-1795)
`fn paragraph_properties(context: &mut Context<'_>, paragraph: &Paragraph, out: &mut String)`

- **Does:** _no doc comment_
- **Calls (crate):** `escape` (markdown/xml.rs:426) L1702; `Context::ids_left` (markdown/xml.rs:552) L1761; `Context::revision` (markdown/xml.rs:557) L1764; `Context::format_mark` (markdown/xml.rs:538) L1769,1785; `Document::section` (markdown/xml.rs:386) L1781; `SectionStories::apply` (markdown/view_stories.rs:953) L1782
- **Calls (std/external/unresolved):** `.as_slice`, `.clone`, `.copied`, `.get`, `.is_empty`, `.is_some_and`, `.is_some`, `.len`, `.map`, `.or`, `.push_str`, `.push`, `.unwrap_or_default`, `Some`, `String::new`
- **Macros:** `write!`
- **Called by:** `story` (markdown/xml.rs:587); `box_xml` (markdown/xml.rs:1949)

#### `is_endnote`  (L1799-1804)
`fn is_endnote(context: &Context<'_>, position: u32) -> bool`

- **Does:** Whether the note a reference at `position` (1-based) points to is an endnote.
- **Calls (std/external/unresolved):** `.and_then`, `.get`, `.is_some_and`, `.ok`, `.saturating_sub`, `usize::try_from`
- **Called by:** `run_properties` (markdown/xml.rs:1806); `write_run` (markdown/xml.rs:1854)

#### `run_properties`  (L1806-1852)
`fn run_properties(context: &Context<'_>, run: &Run) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `is_endnote` (markdown/xml.rs:1799) L1810; `Context::highlighted` (markdown/xml.rs:579) L1835
- **Calls (std/external/unresolved):** `.is_empty`, `.push_str`, `.unwrap_or`, `Some`, `String::new`
- **Macros:** `format!`, `write!`
- **Called by:** `peel_trailing_del_from_mix_into_following_pure_ins` (comparer/finalize.rs:12261) (?); `write_run` (markdown/xml.rs:1854)

#### `write_run`  (L1854-1924)
`fn write_run(context: &mut Context<'_>, run: &Run, relate: &mut dyn Relate, out: &mut String)`

- **Does:** _no doc comment_
- **Calls (crate):** `run_properties` (markdown/xml.rs:1806) L1855; `escape` (markdown/xml.rs:426) L1864,1881; `is_endnote` (markdown/xml.rs:1799) L1887; `Relate::picture` (markdown/xml.rs:458) L1898; `drawing_xml` (markdown/xml.rs:1997) L1902; `Drawing::wraps_run` (markdown/from_docx/drawing.rs:1308) (by name) L1910; `Relate::drawing` (markdown/xml.rs:462) L1911
- **Calls (std/external/unresolved):** `.copied`, `.get`, `.is_some_and`, `.push_str`, `.saturating_sub`, `.unwrap_or`, `Some`, `usize::try_from`
- **Macros:** `matches!`, `write!`
- **Called by:** `story` (markdown/xml.rs:587); `box_xml` (markdown/xml.rs:1949)

#### `text_box_xml`  (L1930-1944)
`fn text_box_xml(context: &mut Context<'_>, boxed: &TextBox, relate: &mut dyn Relate) -> String`

- **Does:** The `w:drawing` of a minimal anchored text box `boxed`: a rectangle the box's content fills.
- **Calls (crate):** `box_xml` (markdown/xml.rs:1949) L1934
- **Calls (std/external/unresolved):** `.push_str`, `String::new`
- **Macros:** `write!`
- **Called by:** `story` (markdown/xml.rs:587)

#### `box_xml`  (L1949-1985)
`fn box_xml(context: &mut Context<'_>, items: &[Item], relate: &mut dyn Relate, out: &mut String)`

- **Does:** A text box's content: paragraphs and runs, each change its own mark.
- **Calls (crate):** `paragraph_properties` (markdown/xml.rs:1699) L1955; `close_tag` (markdown/xml.rs:1988) L1960,1978; `Context::revision` (markdown/xml.rs:557) L1963,1974; `write_run` (markdown/xml.rs:1854) L1968
- **Calls (std/external/unresolved):** `.flatten`, `.insert_str`, `.pop`, `.push_str`, `.take`, `Some`
- **Macros:** `format!`, `write!`
- **Called by:** `text_box_xml` (markdown/xml.rs:1930)

#### `close_tag`  (L1988-1993)
`fn close_tag(kind: Kind) -> &'static str`

- **Does:** The closing tag of a mark of `kind`.
- **Calls:** none
- **Called by:** `box_xml` (markdown/xml.rs:1949)

#### `drawing_xml`  (L1997-2017, pub(crate))
`pub(crate) fn drawing_xml(picture: &Picture, rel: &str, id: u32, out: &mut String)`

- **Does:** A `w:drawing` holding `picture` inline, embedded through `rel`, with drawing id `id`.
- **Calls (crate):** `escape` (markdown/xml.rs:426) L1998
- **Macros:** `write!`
- **Called by:** `Transaction::image_paragraph` (edit/images.rs:143); `write_run` (markdown/xml.rs:1854)

#### `comments_part`  (L2020-2094, pub(super))
`pub(super) fn comments_part(context: &Context<'_>, comments: &[Comment]) -> String`

- **Does:** A comments part holding `comments`.
- **Calls (crate):** `escape` (markdown/xml.rs:426) L2049,2051,2072,2081,2087; `date_attribute` (markdown/xml.rs:2464) L2050
- **Calls (std/external/unresolved):** `.as_ref`, `.as_str`, `.chars`, `.clone`, `.collect`, `.filter_map`, `.filter`, `.find`, `.is_empty`, `.iter`, `.map_or`, `.map`, `.next`, `.or`, `.push_str`, `.split_whitespace`, `.trim`, `.unwrap_or_default`, `.unwrap_or_else`, `String::from`, `String::new`
- **Macros:** `format!`, `write!`
- **Called by:** `assemble` (markdown/package.rs:97)

#### `abstract_num`  (L2109-2118, pub(crate))
`pub(crate) fn abstract_num(id: u32, format: ListFormat) -> String`

- **Does:** A `w:abstractNum` with nine levels of `format`, each indented a further half inch.
- **Calls (crate):** `write_list_level` (markdown/xml.rs:2123) L2114
- **Calls (std/external/unresolved):** `.push_str`
- **Macros:** `format!`
- **Called by:** `emit` (edit/tracked.rs:275); `Transaction::apply` (edit.rs:4489); `numbering` (markdown/xml.rs:2168); `tests::label_levels_match_abstract_num_for_the_same_kind` (markdown/xml.rs:2547)

#### `write_list_level`  (L2123-2137)
`fn write_list_level(out: &mut String, level: u32, format: ListFormat)`

- **Does:** One level, as [`abstract_num`] writes it: `left` half an inch further each level, hanging 360, and the glyph or `%n.` of `format`.
- **Calls (std/external/unresolved):** `.len`, `.to_string`
- **Macros:** `format!`, `write!`
- **Called by:** `abstract_num` (markdown/xml.rs:2109); `label_numbering` (markdown/xml.rs:2199)

#### `plain_decimal`  (L2141-2150)
`fn plain_decimal(text: &str) -> Option<u32>`

- **Does:** `1.`, `12.`: the number a decimal level prints.
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.is_ascii_digit`, `.is_empty`, `.len`, `.ok`, `.parse`, `.starts_with`, `.strip_suffix`
- **Called by:** `label_numbering` (markdown/xml.rs:2199)

#### `num`  (L2154-2164, pub(crate))
`pub(crate) fn num(num_id: u32, abstract_id: u32, start: Option<(u32, u32)>) -> String`

- **Does:** A `w:num` instance of `abstract_id`, optionally starting `level` at `start`.
- **Calls (std/external/unresolved):** `.push_str`
- **Macros:** `format!`, `write!`
- **Called by:** `drawing_group` (convert/mod.rs:16690) (?); `xfrm_box` (convert/mod.rs:16778) (?); `collect_group` (convert/mod.rs:16804) (?); `cust_geom` (convert/mod.rs:16963) (?); `vml_group_box` (convert/mod.rs:19263) (?); `emit` (edit/tracked.rs:275); `Transaction::apply` (edit.rs:4489); `numbering` (markdown/xml.rs:2168); `label_numbering` (markdown/xml.rs:2199)

#### `numbering`  (L2168-2193, pub(super))
`pub(super) fn numbering( lists: &[List], bullets: u32, decimal: u32, first_num: u32, ) -> (String, String)`

- **Does:** The `w:abstractNum` elements for bullets (`bullets`) and for decimal numbering (`decimal`), and one `w:num` per instance.
- **Calls (crate):** `abstract_num` (markdown/xml.rs:2109) L2182; `num` (markdown/xml.rs:2154) L2190
- **Calls (std/external/unresolved):** `.any`, `.enumerate`, `.iter`, `.push_str`, `.then_some`, `.unwrap_or`, `String::new`, `u32::try_from`
- **Called by:** `numbering` (markdown/package.rs:948); `tests::numbering_restarts_each_ordered_list_at_its_start` (markdown/xml.rs:2620)

#### `label_numbering`  (L2199-2250, pub(super))
`pub(super) fn label_numbering( labels: &[Label], uses: &[LabelUse], first_abstract: u32, first_num: u32, ) -> (String, String)`

- **Does:** One list per label.
- **Calls (crate):** `write_list_level` (markdown/xml.rs:2123) L2215,2220; `plain_decimal` (markdown/xml.rs:2141) L2218,2242; `escape` (markdown/xml.rs:426) L2230; `num` (markdown/xml.rs:2154) L2247
- **Calls (std/external/unresolved):** `.get`, `.is_some`, `.map`, `.or_else`, `.push_str`, `.unwrap_or`, `.zip`, `String::new`, `u32::try_from`
- **Macros:** `write!`
- **Called by:** `label_numbering` (markdown/package.rs:964); `tests::label_levels_match_abstract_num_for_the_same_kind` (markdown/xml.rs:2547)

#### `named_style_definition`  (L2255-2265, pub(super))
`pub(super) fn named_style_definition(id: &str) -> String`

- **Does:** The definition of a style this writer uses, for a document that lacks it.
- **Calls (crate):** `escape` (markdown/xml.rs:426) L2259,2260
- **Calls (std/external/unresolved):** `.replace`
- **Macros:** `format!`
- **Called by:** `with_named_styles` (markdown/package.rs:881)

#### `style_definition`  (L2267-2428, pub(crate))
`pub(crate) fn style_definition(id: &str) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `paragraph` (ambiguous, 17 candidates) L2327,2335,2342,2348,2353,2367...
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`, `.ok`, `.parse`, `.to_string`, `Some`, `border`, `character`
- **Macros:** `format!`
- **Called by:** `definition` (edit/structural.rs:470); `define_styles` (legacy_doc.rs:404); `with_styles` (markdown/package.rs:864); `with_named_styles` (markdown/package.rs:881); `gallery_flags` (markdown/view_styles.rs:707); `tests::every_style_the_writer_names_has_a_definition` (markdown/xml.rs:2498)

#### `with_section_change`  (L2435-2462)
`fn with_section_change(section: &str, mark: &FormatMark) -> String`

- **Does:** ` w:date="…"`, or nothing for an empty date: the attribute is optional, and a note the view gave no exact date (an author with several, the `authors:` table only spanning days) is written undated, never with an invented one.
- **Calls (crate):** `escape` (markdown/xml.rs:426) L2458; `date_attribute` (markdown/xml.rs:2464) L2459
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.contains`, `.ends_with`, `.filter`, `.find`, `.map`, `.rfind`, `.split_inclusive`, `.to_string`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `Document::section` (markdown/xml.rs:386)

#### `date_attribute`  (L2464-2470)
`fn date_attribute(date: &str) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `escape` (markdown/xml.rs:426) L2468
- **Calls (std/external/unresolved):** `.is_empty`, `String::new`
- **Macros:** `format!`
- **Called by:** `Context::format_mark` (markdown/xml.rs:538); `Context::revision` (markdown/xml.rs:557); `comments_part` (markdown/xml.rs:2020); `with_section_change` (markdown/xml.rs:2435)

## Test functions (6)

- `tests::a_run_splits_after_its_last_inner_space_else_before_its_last_character` (L2478): _no doc comment_
- `tests::escape_drops_characters_xml_forbids` (L2489): _no doc comment_
- `tests::every_style_the_writer_names_has_a_definition` (L2498): _no doc comment_
- `tests::level_xml` (L2529): The `<w:lvl>` of abstract `index` at `level`, tags included.
- `tests::label_levels_match_abstract_num_for_the_same_kind` (L2547): _no doc comment_
- `tests::numbering_restarts_each_ordered_list_at_its_start` (L2620): _no doc comment_
