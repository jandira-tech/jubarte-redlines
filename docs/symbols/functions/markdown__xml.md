# Functions and methods in `src/markdown/xml.rs`

57 non-test, 6 test. Part of the [symbol manifest](../README.md).

#### `size_half_points`  (L190-195, pub(super))
`pub(super) fn size_half_points(value: &str) -> Option<u32>`

- **Does:** `9pt`, `10.5pt`: a paragraph's text size (#1474) in the half-points `w:sz` stores, Word's nearest; `None` past `ST_HpsMeasure`'s 1 to 3276 (1638pt).
- **Calls (std/external/unresolved):** `.contains`, `.is_finite`, `.ok`, `.parse`, `.round`, `.strip_suffix`, `.then_some`
- **Called by:** `IdLine::clause` (markdown/view_notes.rs:1415); `spacing_words` (markdown/view_notes.rs:1483)

#### `TableGeometry::clause`  (L247-280, pub(super))
`pub(super) fn clause(&mut self, clause: &str) -> Option<()>`

- **Does:** One `widths 2.14in 1in`, `width auto|6.5in|50%`, `indent 0.25in` or `layout fixed` clause into `self`; `None` when it is none of these or a value is out of Word's range.
- **Calls (crate):** `twips` (ambiguous: `twips` (edit/watermark.rs:132); `twips` (markdown/from_docx/header.rs:618)) L272; `twips_in` (markdown/view_styles.rs:229) L275
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.filter`, `.is_empty`, `.is_finite`, `.map`, `.ok`, `.parse`, `.round`, `.split_once`, `.split`, `.strip_suffix`, `.then_some`, `Some`, `TableWidth::Fiftieths`, `TableWidth::Twips`
- **Called by:** `table_line` (markdown/view_notes.rs:781); `IdLine::clause` (markdown/view_notes.rs:1415) (?)

#### `Spacing::is_empty`  (L284-286, pub(super))
`pub(super) fn is_empty(&self) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Self::default`
- **Called by:** `spacing_words` (markdown/view_notes.rs:1483)

#### `Spacing::clause`  (L293-327, pub(super))
`pub(super) fn clause(&mut self, word: &str, value: &str) -> Option<()>`

- **Does:** One `before 6pt`, `before auto`, `after 0pt`, `line 1.5` or `line 8.65pt exact` clause into `self`; `None` when it is none of these or its value is negative or past what a twip count holds.
- **Calls (crate):** `points` (ambiguous: `points` (markdown/borders.rs:273); `points` (markdown/view_styles.rs:222)) L306,307,310,311
- **Calls (std/external/unresolved):** `.contains`, `.is_finite`, `.ok`, `.parse`, `.round`, `.split_once`, `.strip_suffix`, `.then_some`, `Some`, `f64::from`
- **Called by:** `IdLine::clause` (markdown/view_notes.rs:1415) (?); `spacing_words` (markdown/view_notes.rs:1483)

#### `Document::index_wrapping_parents`  (L547-557, pub(super))
`pub(super) fn index_wrapping_parents(&mut self)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clear`, `.clone`, `.entry`, `.filter`, `.get`, `.insert`, `.is_empty`, `.or_default`
- **Called by:** `read` (markdown/write.rs:137) (?)

#### `Document::section`  (L561-571, pub(super))
`pub(super) fn section(&self, number: usize, fallback: &str) -> String`

- **Does:** The `w:sectPr` of section `number` (from 1) as the view names its page: its own, else the first section's, else `fallback`.
- **Calls (crate):** `PageSetup::sect_pr` (markdown/view_notes.rs:2165) (by name) L566; `with_section_change` (markdown/xml.rs:2879) L568
- **Calls (std/external/unresolved):** `.get`, `.map_or_else`, `.or_else`, `.to_string`
- **Called by:** `assemble` (markdown/package.rs:103); `paragraph_properties` (markdown/xml.rs:2039)

#### `Document::section_breaks`  (L574-579, pub(super))
`pub(super) fn section_breaks(&self) -> usize`

- **Does:** How many paragraphs end a section.
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.iter`, `Item::Paragraph`
- **Macros:** `matches!`
- **Called by:** `assemble` (markdown/package.rs:103)

#### `escape`  (L601-626, pub(super))
`pub(super) fn escape(text: &str) -> Cow<'_, str>`

- **Does:** Text for XML content or an attribute: markup escaped, characters XML 1.0 forbids dropped.
- **Calls (crate):** `nested_paragraph_property_source_history_tests::clean` (comparer/formatchg.rs:788) (by name) L608,613
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.filter`, `.len`, `.push_str`, `.push`, `Cow::Borrowed`, `Cow::Owned`, `String::with_capacity`
- **Macros:** `matches!`
- **Called by:** `xml` (markdown/frames.rs:116); `serialize::{fn put}` (markdown/from_docx/drawing.rs:540); `Writer::inline` (markdown/from_docx/mod.rs:3218); `Writer::run` (markdown/math.rs:1698); `Writer::node` (markdown/math.rs:1734); `assemble` (markdown/package.rs:103); `renamed` (markdown/package.rs:676); `with_table_styles` (markdown/package.rs:706); `table_border_sides` (markdown/package.rs:753); `built_in_table_style` (markdown/package.rs:794); `with_named_styles` (markdown/package.rs:1082); `Marks::mark_id` (markdown/view_stories.rs:471); +19 more

#### `Relate::hyperlink`  (L631-631, trait decl, no body)
`fn hyperlink(&mut self, url: &str) -> String`

- **Does:** The relationship id of an external hyperlink from this story's part.
- **Calls:** none
- **Called by:** `story` (markdown/xml.rs:786)

#### `Relate::picture`  (L633-633, trait decl, no body)
`fn picture(&mut self, index: usize) -> String`

- **Does:** The relationship id of a picture from this story's part.
- **Calls:** none
- **Called by:** `write_run` (markdown/xml.rs:2227)

#### `Relate::drawing`  (L638-643, trait decl, no body)
`fn drawing( &mut self, index: usize, next_doc_pr: &mut u32, story: Option<&[(String, String)]>, ) -> Option<String>`

- **Does:** The XML of drawing `index` written into the package beside this story's part: its relationships and dependent parts too.
- **Calls:** none
- **Called by:** `write_run` (markdown/xml.rs:2227)

#### `Context::revision_id`  (L698-733)
`fn revision_id(&mut self, by: Option<u32>) -> u32`

- **Does:** A `w:ins` or `w:del` start tag's name and attributes, with the author and date change `by` names, or the document's.
- **Calls (crate):** `Context::revision_id` (markdown/xml.rs:698) L708
- **Calls (std/external/unresolved):** `.copied`, `.entry`, `.get`, `.insert`, `.or_insert`, `.push`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `Context::revision_id` (markdown/xml.rs:698); `Context::format_mark` (markdown/xml.rs:737); `Context::revision` (markdown/xml.rs:756); `story` (markdown/xml.rs:786)

#### `Context::format_mark`  (L737-747)
`fn format_mark(&mut self, name: &str, mark: &FormatMark) -> String`

- **Does:** A formatting change's start tag: `name` with its id (the view's, unless a mark took it) and its author and date.
- **Calls (crate):** `Context::revision_id` (markdown/xml.rs:698) L741; `escape` (markdown/xml.rs:601) L744; `date_attribute` (markdown/xml.rs:2921) L745
- **Calls (std/external/unresolved):** `.filter`, `.insert`, `.unwrap_or_else`
- **Macros:** `format!`
- **Called by:** `story` (markdown/xml.rs:786); `paragraph_properties` (markdown/xml.rs:2039)

#### `Context::ids_left`  (L751-754)
`fn ids_left(&self, by: u32) -> usize`

- **Does:** How many of the ids the view names for change `by` no mark has taken yet.
- **Calls (std/external/unresolved):** `.copied`, `.get`, `.map_or`, `.saturating_sub`, `.unwrap_or`
- **Called by:** `Context::revision` (markdown/xml.rs:756); `story` (markdown/xml.rs:786); `close_mark` (markdown/xml.rs:1830); `split_for_ids` (markdown/xml.rs:1948); `paragraph_properties` (markdown/xml.rs:2039)

#### `Context::revision`  (L756-776)
`fn revision(&mut self, kind: Kind, by: Option<u32>) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Context::revision_id` (markdown/xml.rs:698) L757; `Context::ids_left` (markdown/xml.rs:751) L763; `escape` (markdown/xml.rs:601) L768; `date_attribute` (markdown/xml.rs:2921) L769
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.get`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `story` (markdown/xml.rs:786); `close_mark` (markdown/xml.rs:1830); `paragraph_properties` (markdown/xml.rs:2039); `box_xml` (markdown/xml.rs:2392)

#### `Context::highlighted`  (L778-781)
`fn highlighted(&self, run: &Run) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`, `.is_some_and`
- **Called by:** `run_properties` (markdown/xml.rs:2175)

#### `story`  (L786-1567, pub(super))
`pub(super) fn story( context: &mut Context<'_>, items: &[Item], relate: &mut dyn Relate, note: Option<Option<Change>>, ) -> String`

- **Does:** Writes a story's items.
- **Calls (crate):** `split_for_ids` (markdown/xml.rs:1948) L792; `natural_marks` (markdown/xml.rs:1876) L818; `starts_inserted_side` (markdown/xml.rs:1788) L836; `close_mark` (markdown/xml.rs:1830) L839,887,900,903,1077,1102...; `spend` (markdown/xml.rs:2033) L841,908,951,986,1143,1148...; `Context::revision` (markdown/xml.rs:756) L842,909,956,1019,1037,1144...; `field_link` (markdown/xml.rs:1575) L852; `note_change` (markdown/xml.rs:1614) L856; `runs_ahead` (markdown/xml.rs:1761) L864; `ends_inserted_side` (markdown/xml.rs:1812) L869; `continues` (markdown/xml.rs:1629) L879,1121; `close_marks` (markdown/xml.rs:1860) L885,907,1031,1169,1565; `continues_change` (markdown/xml.rs:1701) L894,1087,1097,1131,1138,1157...; `escape` (markdown/xml.rs:601) L921,944,1288,1299,1310,1337; `paragraph_properties` (markdown/xml.rs:2039) L926; `Context::format_mark` (markdown/xml.rs:737) L932,1005,1015,1371,1469,1473...; `text_box_xml` (markdown/xml.rs:2373) L933; `omml` (markdown/math.rs:1923) L1040; `named` (ambiguous, 5 candidates) L1058; `Context::ids_left` (markdown/xml.rs:751) L1059,1193; `wrapping_chain` (markdown/xml.rs:1672) L1092,1177; `write_run` (markdown/xml.rs:2227) L1108,1212,1220; `resumes_later` (markdown/xml.rs:1768) L1120; `resumes_after` (markdown/xml.rs:1725) L1133; `field_argument` (markdown/xml.rs:1807) L1269,1271,1275; `Relate::hyperlink` (markdown/xml.rs:631) L1303; `xml` (markdown/borders.rs:254) L1359,1416,1515; `Context::revision_id` (markdown/xml.rs:698) L1390; `CellsAt::holds` (markdown/view_notes.rs:582) (by name) L1514
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.any`, `.as_deref`, `.as_ref`, `.as_slice`, `.clone_from`, `.clone`, `.cloned`, `.contains`, `.copied`, `.enumerate`, `.filter`, `.find_map`, `.find`, `.flatten`, `.get`, `.insert_str`, `.insert`, `.into_iter`, `.is_empty`, `.is_none_or`, `.is_none`, `.is_some_and`, `.is_some`, `.iter`, `.last_mut`, `.last`, `.len`, `.map_or`, `.map`, `.max`, `.pop`, `.position`, `.push_str`, `.push`, `.rev`, `.rfind`, `.rposition`, `.saturating_sub`, `.sum`, `.take`, `.to_string`, `.to_vec`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `.zip`, `Item::Bookmark`, `Item::CommentReference`, `Item::RangeEnd`, `Item::RangeStart`, `Some`, `String::new`, `Vec::new`, `std::mem::take`, `u32::try_from`
- **Macros:** `format!`, `matches!`, `vec!`, `write!`
- **Called by:** `tests::a_page_break_stays_inside_its_paragraph_and_a_section_mark_ends_it` (legacy_doc.rs:1998); `assemble` (markdown/package.rs:103); `footnotes` (markdown/package.rs:1348); `write_run` (markdown/xml.rs:2227)

#### `field_link`  (L1575-1610)
`fn field_link(rest: &[Item], document: &Document) -> Option<Change>`

- **Does:** Whether the next run after a comment's marks continues the open change.
- **Calls (std/external/unresolved):** `.filter`, `.get`, `.is_none_or`, `.len`, `HashSet::::from_iter`, `Some`
- **Called by:** `story` (markdown/xml.rs:786); `continues` (markdown/xml.rs:1629); `natural_marks` (markdown/xml.rs:1876)

#### `note_change`  (L1614-1623)
`fn note_change(notes: &HashMap<u32, (Change, bool)>, item: &Item) -> Option<Change>`

- **Does:** The change a comment's reference, or its range end, sits in, when the view put the comment's note inside one (see `Document::note_changes`).
- **Calls (std/external/unresolved):** `.filter`, `.get`, `.map`
- **Called by:** `story` (markdown/xml.rs:786); `natural_marks` (markdown/xml.rs:1876)

#### `continues`  (L1629-1667)
`fn continues(document: &Document, rest: &[Item], open: Option<Change>, marks: &[Change]) -> bool`

- **Does:** Whether the next real item in `rest` is a run still inside a mark: `open`'s own, one a nested change suspended (`marks`), whose other pieces name the same ids, or a change `open` resumes after, which the open mark then holds.
- **Calls (crate):** `field_link` (markdown/xml.rs:1575) L1642; `continues_change` (markdown/xml.rs:1701) L1652,1659; `wrapping_chain` (markdown/xml.rs:1672) L1658; `resumes_after` (markdown/xml.rs:1725) L1665
- **Calls (std/external/unresolved):** `.any`, `.chain`, `.copied`, `.get`, `.into_iter`, `.is_some_and`, `.iter`, `.position`, `Item::Bookmark`, `Item::CommentReference`, `Item::RangeEnd`, `Item::RangeStart`, `Some`
- **Macros:** `matches!`
- **Called by:** `distribute` (markdown/write.rs:425) (?); `story` (markdown/xml.rs:786); `natural_marks` (markdown/xml.rs:1876)

#### `wrapping_chain`  (L1672-1695)
`fn wrapping_chain(document: &Document, change: Change) -> Vec<Change>`

- **Does:** The named parents, including those another piece of the same revision names.
- **Calls (crate):** `ShortId::range` (ids/short.rs:344) (by name) L1679
- **Calls (std/external/unresolved):** `.contains`, `.first_key_value`, `.get`, `.map`, `.next_back`, `.or_else`, `.push`, `Vec::new`
- **Called by:** `story` (markdown/xml.rs:786); `continues` (markdown/xml.rs:1629)

#### `continues_change`  (L1701-1720)
`fn continues_change(document: &Document, inner: Change, outer: Change) -> bool`

- **Does:** Whether `inner` is another piece of `outer`'s change: the view names its `rN` ids (`{++a++}{r1 @AL}…{++b++}{r1 @AL}`), which `outer` shares.
- **Calls (crate):** `attribution` (ambiguous: `attribution` (markdown/from_docx/critic.rs:1109); `attribution` (markdown/from_docx/mod.rs:2337); `attribution` (markdown/write.rs:6237)) L1712
- **Calls (std/external/unresolved):** `.all`, `.contains`, `.get`, `.is_empty`, `.is_some_and`, `.iter`, `.map`, `.unwrap_or`
- **Called by:** `story` (markdown/xml.rs:786); `continues` (markdown/xml.rs:1629); `resumes_after` (markdown/xml.rs:1725); `resumes_later` (markdown/xml.rs:1768)

#### `resumes_after`  (L1725-1758)
`fn resumes_after(document: &Document, rest: &[Item], inner: Change, outer: Change) -> bool`

- **Does:** Whether `outer`'s change has a piece past `inner`'s group: then `inner` sits inside `outer`'s mark, which stays open around it.
- **Calls (crate):** `continues_change` (markdown/xml.rs:1701) L1742,1746
- **Calls (std/external/unresolved):** `.iter`, `.push`, `.rposition`, `.truncate`
- **Macros:** `vec!`
- **Called by:** `story` (markdown/xml.rs:786); `continues` (markdown/xml.rs:1629)

#### `runs_ahead`  (L1761-1763)
`fn runs_ahead(rest: &[Item]) -> bool`

- **Does:** Whether any run is left in `rest`.
- **Calls (std/external/unresolved):** `.any`, `.iter`, `Item::Run`
- **Macros:** `matches!`
- **Called by:** `story` (markdown/xml.rs:786)

#### `resumes_later`  (L1768-1781)
`fn resumes_later(document: &Document, rest: &[Item], outer: Change) -> bool`

- **Does:** Whether `outer`'s change holds another change ahead in `rest`: a run of a piece of it is still to come, past the nested ones between, so its mark stays open around them as around the piece it just resumed.
- **Calls (crate):** `continues_change` (markdown/xml.rs:1701) L1778
- **Calls (std/external/unresolved):** `.any`, `.is_some_and`, `.iter`
- **Called by:** `story` (markdown/xml.rs:786)

#### `starts_inserted_side`  (L1788-1803)
`fn starts_inserted_side(document: &Document, rest: &[Item]) -> Option<Change>`

- **Does:** Ends the open mark.
- **Calls (crate):** `ends_inserted_side` (markdown/xml.rs:1812) L1801
- **Calls (std/external/unresolved):** `.count`, `.first`, `.get`, `.iter`, `.take_while`, `.then_some`, `Item::RangeEnd`, `Item::Run`, `Some`
- **Macros:** `matches!`
- **Called by:** `story` (markdown/xml.rs:786)

#### `field_argument`  (L1807-1809)
`fn field_argument(text: &str) -> String`

- **Does:** `text` as a quoted field-code argument: Word reads `\"` as a quote and `\\` as a backslash there.
- **Calls (std/external/unresolved):** `.replace`
- **Called by:** `story` (markdown/xml.rs:786)

#### `ends_inserted_side`  (L1812-1828)
`fn ends_inserted_side(document: &Document, rest: &[Item], open: Option<Change>) -> bool`

- **Does:** Whether `open` is an insertion and the next item a deleted run.
- **Calls (std/external/unresolved):** `.contains_key`, `.find`, `.get`, `.is_none_or`, `.iter`, `Item::RangeStart`, `Item::Run`, `Some`
- **Macros:** `matches!`
- **Called by:** `story` (markdown/xml.rs:786); `starts_inserted_side` (markdown/xml.rs:1788)

#### `close_mark`  (L1830-1856)
`fn close_mark( context: &mut Context<'_>, natural: &HashMap<u32, usize>, out: &mut String, open: &mut Option<Change>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Context::ids_left` (markdown/xml.rs:751) L1851; `Context::revision` (markdown/xml.rs:756) L1852
- **Calls (std/external/unresolved):** `.copied`, `.count`, `.filter`, `.flatten`, `.get`, `.iter`, `.push_str`, `.saturating_sub`, `.take`, `.unwrap_or`, `Some`
- **Macros:** `write!`
- **Called by:** `story` (markdown/xml.rs:786); `close_marks` (markdown/xml.rs:1860)

#### `close_marks`  (L1860-1872)
`fn close_marks( context: &mut Context<'_>, natural: &HashMap<u32, usize>, out: &mut String, open: &mut Option<Change>, marks: &mut Vec<Change>, )`

- **Does:** [`close_mark`] for the open mark and every mark a nested change's mark suspended, innermost first.
- **Calls (crate):** `close_mark` (markdown/xml.rs:1830) L1867,1870
- **Calls (std/external/unresolved):** `.pop`, `Some`
- **Called by:** `story` (markdown/xml.rs:786)

#### `natural_marks`  (L1876-1942)
`fn natural_marks(document: &Document, items: &[Item]) -> HashMap<u32, usize>`

- **Does:** The marks each numbered change opens in `items` when no change splits: a run that does not continue the open mark, a paragraph mark, a row.
- **Calls (crate):** `note_change` (markdown/xml.rs:1614) L1882; `field_link` (markdown/xml.rs:1575) L1892; `continues` (markdown/xml.rs:1629) L1910
- **Calls (std/external/unresolved):** `.entry`, `.enumerate`, `.flatten`, `.is_none`, `.is_some`, `.iter`, `.or_default`, `.pop`, `.push`, `HashMap::new`, `Some`, `Vec::new`
- **Called by:** `story` (markdown/xml.rs:786)

#### `split_for_ids`  (L1948-2015)
`fn split_for_ids<'i>(context: &Context<'_>, items: &'i [Item]) -> Cow<'i, [Item]>`

- **Does:** `items` with text runs split until each change has a piece for every id the view names it: `{++one two++}{r70+71 @AL}` is two marks, `one ` and `two`.
- **Calls (crate):** `Context::ids_left` (markdown/xml.rs:751) L1973; `split_point` (markdown/xml.rs:2019) L1993
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.entry`, `.enumerate`, `.filter_map`, `.find_map`, `.insert`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.map`, `.or_default`, `.rev`, `.sort_unstable`, `.split_at`, `.then`, `.to_string`, `.to_vec`, `Content::Text`, `Cow::Borrowed`, `Cow::Owned`, `HashMap::new`, `Item::Run`
- **Called by:** `story` (markdown/xml.rs:786)

#### `split_point`  (L2019-2030, pub(super))
`pub(super) fn split_point(text: &str) -> Option<usize>`

- **Does:** Where a run's text splits in two: after its last space with text after it, else before its last character; `None` for one character.
- **Calls (std/external/unresolved):** `.char_indices`, `.filter`, `.find`, `.last`, `.len`, `.map`, `.or_else`, `.rev`
- **Called by:** `tracked` (markdown/view_stories.rs:1393); `split_for_ids` (markdown/xml.rs:1948); `tests::a_run_splits_after_its_last_inner_space_else_before_its_last_character` (markdown/xml.rs:2952)

#### `spend`  (L2033-2037)
`fn spend(natural: &mut HashMap<u32, usize>, by: u32)`

- **Does:** One natural mark of change `by` is written.
- **Calls (std/external/unresolved):** `.get_mut`, `.saturating_sub`
- **Called by:** `story` (markdown/xml.rs:786)

#### `paragraph_properties`  (L2039-2164)
`fn paragraph_properties(context: &mut Context<'_>, paragraph: &Paragraph, out: &mut String)`

- **Does:** _no doc comment_
- **Calls (crate):** `escape` (markdown/xml.rs:601) L2042; `Context::ids_left` (markdown/xml.rs:751) L2121; `Context::revision` (markdown/xml.rs:756) L2124; `Context::format_mark` (markdown/xml.rs:737) L2137,2153; `Document::section` (markdown/xml.rs:561) L2149; `SectionStories::apply` (markdown/view_stories.rs:1539) L2150
- **Calls (std/external/unresolved):** `.as_slice`, `.clone`, `.copied`, `.get`, `.is_empty`, `.is_some_and`, `.is_some`, `.len`, `.map`, `.or`, `.push_str`, `.push`, `.unwrap_or_default`, `Some`, `String::new`
- **Macros:** `write!`
- **Called by:** `story` (markdown/xml.rs:786); `box_xml` (markdown/xml.rs:2392)

#### `is_endnote`  (L2168-2173)
`fn is_endnote(context: &Context<'_>, position: u32) -> bool`

- **Does:** Whether the note a reference at `position` (1-based) points to is an endnote.
- **Calls (std/external/unresolved):** `.and_then`, `.get`, `.is_some_and`, `.ok`, `.saturating_sub`, `usize::try_from`
- **Called by:** `run_properties` (markdown/xml.rs:2175); `write_run` (markdown/xml.rs:2227)

#### `run_properties`  (L2175-2225)
`fn run_properties(context: &Context<'_>, run: &Run) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `is_endnote` (markdown/xml.rs:2168) L2179; `Context::highlighted` (markdown/xml.rs:778) L2208
- **Calls (std/external/unresolved):** `.copied`, `.flatten`, `.is_empty`, `.last`, `.push_str`, `.unwrap_or`, `Some`, `String::new`
- **Macros:** `format!`, `write!`
- **Called by:** `peel_trailing_del_from_mix_into_following_pure_ins` (comparer/finalize.rs:12266) (?); `write_run` (markdown/xml.rs:2227)

#### `write_run`  (L2227-2324)
`fn write_run( context: &mut Context<'_>, run: &Run, relate: &mut dyn Relate, out: &mut String, parent_deleted: bool, )`

- **Does:** _no doc comment_
- **Calls (crate):** `run_properties` (markdown/xml.rs:2175) L2234; `escape` (markdown/xml.rs:601) L2243,2260; `is_endnote` (markdown/xml.rs:2168) L2266; `Relate::picture` (markdown/xml.rs:633) L2277; `drawing_xml` (markdown/xml.rs:2441) L2281; `Drawing::wraps_run` (markdown/from_docx/drawing.rs:1818) (by name) L2289; `outside_anchor` (markdown/xml.rs:2331) L2296; `story` (markdown/xml.rs:786) L2297; `Relate::drawing` (markdown/xml.rs:638) L2310
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.collect`, `.copied`, `.get`, `.is_empty`, `.is_some_and`, `.iter`, `.map`, `.push_str`, `.saturating_sub`, `.to_string`, `.unwrap_or`, `Some`, `usize::try_from`
- **Macros:** `matches!`, `write!`
- **Called by:** `story` (markdown/xml.rs:786); `box_xml` (markdown/xml.rs:2392)

#### `outside_anchor`  (L2331-2367)
`fn outside_anchor<'i>( document: &Document, items: &'i [Item], anchor: Option<Change>, ) -> Cow<'i, [Item]>`

- **Does:** A text box's `items` without the copies of `anchor`, the change of the run that holds the box.
- **Calls (crate):** `single` (changes/logical.rs:287) (by name) L2345,2350
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.collect`, `.filter`, `.get`, `.iter`, `.len`, `.map`, `Cow::Borrowed`, `Cow::Owned`, `Item::Run`, `Some`, `copy`
- **Macros:** `matches!`
- **Called by:** `write_run` (markdown/xml.rs:2227)

#### `text_box_xml`  (L2373-2387)
`fn text_box_xml(context: &mut Context<'_>, boxed: &TextBox, relate: &mut dyn Relate) -> String`

- **Does:** The `w:drawing` of a minimal anchored text box `boxed`: a rectangle the box's content fills.
- **Calls (crate):** `box_xml` (markdown/xml.rs:2392) L2377
- **Calls (std/external/unresolved):** `.push_str`, `String::new`
- **Macros:** `write!`
- **Called by:** `story` (markdown/xml.rs:786)

#### `box_xml`  (L2392-2429)
`fn box_xml(context: &mut Context<'_>, items: &[Item], relate: &mut dyn Relate, out: &mut String)`

- **Does:** A text box's content: paragraphs and runs, each change its own mark.
- **Calls (crate):** `paragraph_properties` (markdown/xml.rs:2039) L2398; `close_tag` (markdown/xml.rs:2432) L2403,2422; `Context::revision` (markdown/xml.rs:756) L2406,2417; `write_run` (markdown/xml.rs:2227) L2411
- **Calls (std/external/unresolved):** `.flatten`, `.insert_str`, `.pop`, `.push_str`, `.take`, `Some`
- **Macros:** `format!`, `write!`
- **Called by:** `text_box_xml` (markdown/xml.rs:2373)

#### `close_tag`  (L2432-2437)
`fn close_tag(kind: Kind) -> &'static str`

- **Does:** The closing tag of a mark of `kind`.
- **Calls:** none
- **Called by:** `box_xml` (markdown/xml.rs:2392)

#### `drawing_xml`  (L2441-2461, pub(crate))
`pub(crate) fn drawing_xml(picture: &Picture, rel: &str, id: u32, out: &mut String)`

- **Does:** A `w:drawing` holding `picture` inline, embedded through `rel`, with drawing id `id`.
- **Calls (crate):** `escape` (markdown/xml.rs:601) L2442
- **Macros:** `write!`
- **Called by:** `Transaction::image_paragraph` (edit/images.rs:143); `write_run` (markdown/xml.rs:2227)

#### `comments_part`  (L2464-2538, pub(super))
`pub(super) fn comments_part(context: &Context<'_>, comments: &[Comment]) -> String`

- **Does:** A comments part holding `comments`.
- **Calls (crate):** `escape` (markdown/xml.rs:601) L2493,2495,2516,2525,2531; `legacy_date_attribute` (markdown/xml.rs:2938) L2494
- **Calls (std/external/unresolved):** `.as_ref`, `.as_str`, `.chars`, `.clone`, `.collect`, `.filter_map`, `.filter`, `.find`, `.is_empty`, `.iter`, `.map_or`, `.map`, `.next`, `.or`, `.push_str`, `.split_whitespace`, `.trim`, `.unwrap_or_default`, `.unwrap_or_else`, `String::from`, `String::new`
- **Macros:** `format!`, `write!`
- **Called by:** `assemble` (markdown/package.rs:103)

#### `abstract_num`  (L2553-2562, pub(crate))
`pub(crate) fn abstract_num(id: u32, format: ListFormat) -> String`

- **Does:** A `w:abstractNum` with nine levels of `format`, each indented a further half inch.
- **Calls (crate):** `write_list_level` (markdown/xml.rs:2567) L2558
- **Calls (std/external/unresolved):** `.push_str`
- **Macros:** `format!`
- **Called by:** `emit` (edit/tracked.rs:452); `Transaction::apply_lists` (edit.rs:6303); `numbering` (markdown/xml.rs:2612); `tests::label_levels_match_abstract_num_for_the_same_kind` (markdown/xml.rs:3021)

#### `write_list_level`  (L2567-2581)
`fn write_list_level(out: &mut String, level: u32, format: ListFormat)`

- **Does:** One level, as [`abstract_num`] writes it: `left` half an inch further each level, hanging 360, and the glyph or `%n.` of `format`.
- **Calls (std/external/unresolved):** `.len`, `.to_string`
- **Macros:** `format!`, `write!`
- **Called by:** `abstract_num` (markdown/xml.rs:2553); `label_numbering` (markdown/xml.rs:2643)

#### `plain_decimal`  (L2585-2594)
`fn plain_decimal(text: &str) -> Option<u32>`

- **Does:** `1.`, `12.`: the number a decimal level prints.
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.is_ascii_digit`, `.is_empty`, `.len`, `.ok`, `.parse`, `.starts_with`, `.strip_suffix`
- **Called by:** `label_numbering` (markdown/xml.rs:2643)

#### `num`  (L2598-2608, pub(crate))
`pub(crate) fn num(num_id: u32, abstract_id: u32, start: Option<(u32, u32)>) -> String`

- **Does:** A `w:num` instance of `abstract_id`, optionally starting `level` at `start`.
- **Calls (std/external/unresolved):** `.push_str`
- **Macros:** `format!`, `write!`
- **Called by:** `drawing_group` (convert/mod.rs:16726) (?); `xfrm_box` (convert/mod.rs:16814) (?); `collect_group` (convert/mod.rs:16840) (?); `cust_geom` (convert/mod.rs:16999) (?); `vml_group_box` (convert/mod.rs:19299) (?); `emit` (edit/tracked.rs:452); `Transaction::apply_lists` (edit.rs:6303); `numbering` (markdown/xml.rs:2612); `label_numbering` (markdown/xml.rs:2643)

#### `numbering`  (L2612-2637, pub(super))
`pub(super) fn numbering( lists: &[List], bullets: u32, decimal: u32, first_num: u32, ) -> (String, String)`

- **Does:** The `w:abstractNum` elements for bullets (`bullets`) and for decimal numbering (`decimal`), and one `w:num` per instance.
- **Calls (crate):** `abstract_num` (markdown/xml.rs:2553) L2626; `num` (markdown/xml.rs:2598) L2634
- **Calls (std/external/unresolved):** `.any`, `.enumerate`, `.iter`, `.push_str`, `.then_some`, `.unwrap_or`, `String::new`, `u32::try_from`
- **Called by:** `numbering` (markdown/package.rs:1149); `tests::numbering_restarts_each_ordered_list_at_its_start` (markdown/xml.rs:3094)

#### `label_numbering`  (L2643-2694, pub(super))
`pub(super) fn label_numbering( labels: &[Label], uses: &[LabelUse], first_abstract: u32, first_num: u32, ) -> (String, String)`

- **Does:** One list per label.
- **Calls (crate):** `write_list_level` (markdown/xml.rs:2567) L2659,2664; `plain_decimal` (markdown/xml.rs:2585) L2662,2686; `escape` (markdown/xml.rs:601) L2674; `num` (markdown/xml.rs:2598) L2691
- **Calls (std/external/unresolved):** `.get`, `.is_some`, `.map`, `.or_else`, `.push_str`, `.unwrap_or`, `.zip`, `String::new`, `u32::try_from`
- **Macros:** `write!`
- **Called by:** `label_numbering` (markdown/package.rs:1165); `tests::label_levels_match_abstract_num_for_the_same_kind` (markdown/xml.rs:3021)

#### `named_style_definition`  (L2699-2709, pub(super))
`pub(super) fn named_style_definition(id: &str) -> String`

- **Does:** The definition of a style this writer uses, for a document that lacks it.
- **Calls (crate):** `escape` (markdown/xml.rs:601) L2703,2704
- **Calls (std/external/unresolved):** `.replace`
- **Macros:** `format!`
- **Called by:** `with_named_styles` (markdown/package.rs:1082)

#### `style_definition`  (L2711-2872, pub(crate))
`pub(crate) fn style_definition(id: &str) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `paragraph` (ambiguous, 17 candidates) L2771,2779,2786,2792,2797,2811...
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`, `.ok`, `.parse`, `.to_string`, `Some`, `border`, `character`
- **Macros:** `format!`
- **Called by:** `definition` (edit/structural.rs:470); `define_styles` (legacy_doc.rs:404); `with_styles` (markdown/package.rs:1065); `with_named_styles` (markdown/package.rs:1082); `gallery_flags` (markdown/view_styles.rs:912); `tests::every_style_the_writer_names_has_a_definition` (markdown/xml.rs:2972)

#### `with_section_change`  (L2879-2906)
`fn with_section_change(section: &str, mark: &FormatMark) -> String`

- **Does:** ` w:date="…"`, or nothing for an empty date: the attribute is optional, and a note the view gave no exact date (an author with several, the `authors:` table only spanning days) is written undated, never with an invented one.
- **Calls (crate):** `escape` (markdown/xml.rs:601) L2902; `date_attribute` (markdown/xml.rs:2921) L2903
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.contains`, `.ends_with`, `.filter`, `.find`, `.map`, `.rfind`, `.split_inclusive`, `.to_string`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `Document::section` (markdown/xml.rs:561)

#### `word_date`  (L2909-2918, pub(super))
`pub(super) fn word_date(date: &str) -> std::borrow::Cow<'_, str>`

- **Does:** A view's offset records an instant; Word stores its wall clock with Z.
- **Calls (crate):** `parse_datetime` (util/datetime.rs:11) L2912
- **Calls (std/external/unresolved):** `.as_bytes`, `.get`, `.is_ok`, `.len`, `.saturating_sub`, `Some`, `std::borrow::Cow::Borrowed`, `std::borrow::Cow::Owned`
- **Macros:** `format!`, `matches!`
- **Called by:** `offset_date_utc` (markdown/xml.rs:2930); `legacy_date_attribute` (markdown/xml.rs:2938)

#### `date_attribute`  (L2921-2927, pub(super))
`pub(super) fn date_attribute(date: &str) -> String`

- **Does:** A revision keeps Word's wall clock and, for an explicit offset, its instant.
- **Calls (crate):** `legacy_date_attribute` (markdown/xml.rs:2938) L2922; `offset_date_utc` (markdown/xml.rs:2930) L2923
- **Macros:** `write!`
- **Called by:** `Marks::mark_id` (markdown/view_stories.rs:471); `with_style_changes` (markdown/view_styles.rs:45); `Context::format_mark` (markdown/xml.rs:737); `Context::revision` (markdown/xml.rs:756); `with_section_change` (markdown/xml.rs:2879)

#### `offset_date_utc`  (L2930-2936, pub(super))
`pub(super) fn offset_date_utc(date: &str) -> Option<String>`

- **Does:** The instant of an explicit offset view date; `Z` keeps the legacy path.
- **Calls (crate):** `word_date` (markdown/xml.rs:2909) L2931; `timestamp_order_key` (convert/mod.rs:12427) L2934; `timestamp_with_offset` (convert/mod.rs:12443) L2935
- **Calls (std/external/unresolved):** `Cow::Owned`, `Some`
- **Macros:** `matches!`
- **Called by:** `threads` (markdown/package.rs:74); `date_attribute` (markdown/xml.rs:2921)

#### `legacy_date_attribute`  (L2938-2944)
`fn legacy_date_attribute(date: &str) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `escape` (markdown/xml.rs:601) L2942; `word_date` (markdown/xml.rs:2909) L2942
- **Calls (std/external/unresolved):** `.is_empty`, `String::new`
- **Macros:** `format!`
- **Called by:** `comments_part` (markdown/xml.rs:2464); `date_attribute` (markdown/xml.rs:2921)

## Test functions (6)

- `tests::a_run_splits_after_its_last_inner_space_else_before_its_last_character` (L2952): _no doc comment_
- `tests::escape_drops_characters_xml_forbids` (L2963): _no doc comment_
- `tests::every_style_the_writer_names_has_a_definition` (L2972): _no doc comment_
- `tests::level_xml` (L3003): The `<w:lvl>` of abstract `index` at `level`, tags included.
- `tests::label_levels_match_abstract_num_for_the_same_kind` (L3021): _no doc comment_
- `tests::numbering_restarts_each_ordered_list_at_its_start` (L3094): _no doc comment_
