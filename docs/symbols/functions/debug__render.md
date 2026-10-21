# Functions and methods in `src/debug/render.rs`

37 non-test, 22 test. Part of the [symbol manifest](../README.md).

#### `render_parts`  (L28-64, pub(super))
`pub(super) fn render_parts( pkg: &Package, part: Option<&str>, ) -> BTreeMap<String, (Vec<String>, usize)>`

- **Does:** Part name → (lines, paragraph or page count), the `(layout)` entry last.
- **Calls (crate):** `main_rels` (debug/render.rs:76) L32; `parse` (debug/render.rs:104) L36,45; `StyleBook::new` (debug/render.rs:156) L37; `story_roles_all` (debug.rs:2037) L39; `local` (debug.rs:381) L48; `part_lines` (debug/render.rs:515) L57; `layout_lines` (debug/render.rs:1156) L61
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.clone`, `.collect`, `.contains`, `.ends_with`, `.find`, `.flatten`, `.get`, `.insert`, `.into_iter`, `.is_none_or`, `.is_some_and`, `.iter`, `.map`, `.to_string`, `.unwrap_or_default`, `BTreeMap::new`
- **Macros:** `format!`
- **Called by:** `story_diagnostic_contract_tests::section_geometry_prints_one_spelling_of_the_same_measure` (debug/render.rs:1554); `story_diagnostic_contract_tests::section_prints_a_plain_value_as_the_file_spells_it` (debug/render.rs:1602); `story_diagnostic_contract_tests::memory_package_filter_resolves_story_roles_and_excludes_nonstories_without_mutation` (debug/render.rs:1691); `story_diagnostic_contract_tests::malformed_package_relationship_diagnostics_do_not_invent_main_or_story_parts` (debug/render.rs:1745); `listing_lines` (debug.rs:2110)

#### `main_rels`  (L76-102)
`fn main_rels(pkg: &Package) -> Vec<Rel>`

- **Does:** The main document part's relationships, found through the package's own `officeDocument` relationship (the part need not be `word/document.xml`).
- **Calls (crate):** `parse` (debug/render.rs:104) L78; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L81; `attr` (debug.rs:641) L83,85,90,91; `resolve_target` (debug.rs:822) L91; `percent_decode` (debug.rs:800) L91; `rels_of` (debug.rs:845) L101
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.find`, `.into_iter`, `.map`, `.next`, `.rsplit`, `.to_string`, `.unwrap_or_default`, `Vec::new`, `rels_in`
- **Called by:** `render_parts` (debug/render.rs:28); `story_diagnostic_contract_tests::malformed_package_relationship_diagnostics_do_not_invent_main_or_story_parts` (debug/render.rs:1745)

#### `parse`  (L104-115)
`fn parse(pkg: &Package, name: &str) -> Option<(Dom, NodeId)>`

- **Does:** _no doc comment_
- **Calls (crate):** `decode_xml` (debug.rs:330) L106; `Dom::new` (xmllinq/mod.rs:387) L107; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L108; `Dom::root` (xmllinq/mod.rs:822) L109; `normalize_unit_measures` (comparer/finalize.rs:11498) L113
- **Calls (std/external/unresolved):** `.find`, `.iter`, `Some`
- **Called by:** `render_parts` (debug/render.rs:28); `main_rels` (debug/render.rs:76); `layout_lines` (debug/render.rs:1156); `story_diagnostic_contract_tests::malformed_package_relationship_diagnostics_do_not_invent_main_or_story_parts` (debug/render.rs:1745)

#### `StyleBook::new`  (L156-239)
`fn new(dom: &Dom, root: NodeId) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L158; `local` (debug.rs:381) L159; `descendant` (debug/render.rs:367) L161,164,175; `child` (debug/render.rs:361) L161,165,166,172,185,186...; `shd` (debug/render.rs:382) L167,192,212; `color` (debug/render.rs:415) L168,193; `highlight` (debug/render.rs:434) L169,194; `run_shd` (debug/render.rs:440) L170,195; `on` (debug/render.rs:445) L172,197; `Dom::attributes_iter` (xmllinq/mod.rs:886) L176; `XName::local_name` (xmllinq/mod.rs:208) (by name) L178; `attr` (debug.rs:641) L185,225,227,228,229; `attr_pairs` (debug/render.rs:852) L200; `font_slots` (debug/render.rs:830) L204; `table` (debug/render.rs:869) L223,225; `kids` (debug/render.rs:354) L224
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.clone_from`, `.clone`, `.collect`, `.insert`, `.is_empty`, `.join`, `.map`, `.push`, `.sort`, `.to_string`, `.unwrap_or_default`, `String::new`, `StyleBook::default`, `StyleInfo::default`, `Vec::new`, `val`
- **Macros:** `format!`, `matches!`
- **Called by:** `render_parts` (debug/render.rs:28); `layout_lines` (debug/render.rs:1156); `story_diagnostic_contract_tests::inherited_frame_overrides_and_repeated_run_tallies_keep_first_source_sample` (debug/render.rs:1361); `story_diagnostic_contract_tests::table_style_resets_and_look_flags_report_only_live_conditional_fills` (debug/render.rs:1769)

#### `StyleBook::name`  (L241-247)
`fn name(&self, id: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone`, `.filter`, `.get`, `.is_empty`, `.map`, `.to_string`, `.unwrap_or_else`
- **Called by:** `StyleBook::font_slots` (debug/render.rs:282); `StyleBook::find` (debug/render.rs:338)

#### `StyleBook::table_shd`  (L252-277)
`fn table_shd(&self, id: &str) -> Vec<(String, String)>`

- **Does:** `id`'s table shading down its `basedOn` chain: a base's conditions stay unless the derived style sets the same one; a nil `shd` resets it.
- **Calls (std/external/unresolved):** `.clone_from`, `.clone`, `.collect`, `.filter_map`, `.find`, `.get`, `.into_iter`, `.is_empty`, `.iter_mut`, `.push`, `.rev`, `.to_string`, `Some`, `Vec::new`
- **Called by:** `table` (debug/render.rs:869)

#### `StyleBook::font_slots`  (L282-303)
`fn font_slots(&self, id: &str) -> Vec<(&'static str, String, String)>`

- **Does:** The faces `id`'s `basedOn` chain asks for, slot by slot, each with the name of the style that sets it (a derived style's slot over its base's, as the converter overlays them).
- **Calls (crate):** `StyleBook::name` (debug/render.rs:241) L289
- **Calls (std/external/unresolved):** `.clone`, `.get`, `.into_iter`, `.is_empty`, `.push`, `.retain`, `.rev`, `.to_string`, `Vec::new`
- **Called by:** `run` (debug/render.rs:717)

#### `StyleBook::frame`  (L307-325)
`fn frame(&self, id: &str) -> Vec<(String, String)>`

- **Does:** `id`'s frame attributes down its `basedOn` chain, a derived style's value over its base's.
- **Calls (crate):** `merge_pairs` (debug/render.rs:859) L322
- **Calls (std/external/unresolved):** `.clone`, `.get`, `.into_iter`, `.is_empty`, `.push`, `.rev`, `.to_string`, `Vec::new`
- **Called by:** `paragraph` (debug/render.rs:590)

#### `StyleBook::para_style`  (L328-334)
`fn para_style(&self, id: String) -> String`

- **Does:** The paragraph style `id` names, or the default one.
- **Calls (std/external/unresolved):** `.clone`, `.is_empty`
- **Called by:** `paragraph` (debug/render.rs:590); `run` (debug/render.rs:717)

#### `StyleBook::find`  (L338-351)
`fn find<T>(&self, id: &str, f: impl Fn(&StyleInfo) -> Option<T>) -> Option<(T, String)>`

- **Does:** The first value `f` finds walking `id`'s `basedOn` chain, with the name of the style that holds it.
- **Calls (crate):** `StyleBook::name` (debug/render.rs:241) L343
- **Calls (std/external/unresolved):** `.clone`, `.get`, `.is_empty`, `.to_string`, `Some`, `f`
- **Called by:** `paragraph` (debug/render.rs:590); `run` (debug/render.rs:717)

#### `kids`  (L354-359)
`fn kids(dom: &Dom, n: NodeId, name: &str) -> Vec<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L355; `local` (debug.rs:381) L357
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`
- **Called by:** `StyleBook::new` (debug/render.rs:156); `table` (debug/render.rs:869); `section` (debug/render.rs:1028); `layout_lines` (debug/render.rs:1156); `numbering_lines` (debug.rs:1602) (?)

#### `child`  (L361-365)
`fn child(dom: &Dom, n: NodeId, name: &str) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L362; `local` (debug.rs:381) L364
- **Calls (std/external/unresolved):** `.find`, `.into_iter`
- **Called by:** `StyleBook::new` (debug/render.rs:156); `color` (debug/render.rs:415); `highlight` (debug/render.rs:434); `run_shd` (debug/render.rs:440); `on` (debug/render.rs:445); `paragraph` (debug/render.rs:590); `run` (debug/render.rs:717); `table` (debug/render.rs:869); `section` (debug/render.rs:1028); `anchor` (debug/render.rs:1106); `layout_lines` (debug/render.rs:1156)

#### `descendant`  (L367-371)
`fn descendant(dom: &Dom, n: NodeId, name: &str) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L368; `local` (debug.rs:381) L370
- **Calls (std/external/unresolved):** `.find`, `.into_iter`
- **Called by:** `StyleBook::new` (debug/render.rs:156); `layout_lines` (debug/render.rs:1156)

#### `nearest`  (L374-378)
`fn nearest(dom: &Dom, n: NodeId, name: &str) -> Option<NodeId>`

- **Does:** The nearest ancestor of `n` named `name`.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L375; `local` (debug.rs:381) L377
- **Calls (std/external/unresolved):** `.find`, `.into_iter`
- **Called by:** `own_text` (debug/render.rs:452); `part_lines` (debug/render.rs:515); `run` (debug/render.rs:717); `table` (debug/render.rs:869)

#### `shd`  (L382-411)
`fn shd(dom: &Dom, x: NodeId) -> Option<String>`

- **Does:** A `w:shd`'s visible fill: `fill` (or `theme:…`), the pattern's colour after `/` when the pattern is not clear.
- **Calls (crate):** `attr` (debug.rs:641) L384,385,386,387
- **Calls (std/external/unresolved):** `.as_str`, `.is_empty`, `.then`, `.to_string`, `.unwrap_or_default`, `Some`, `String::new`
- **Macros:** `format!`, `matches!`
- **Called by:** `StyleBook::new` (debug/render.rs:156); `run_shd` (debug/render.rs:440); `paragraph` (debug/render.rs:590); `table` (debug/render.rs:869); `story_diagnostic_contract_tests::shading_reports_literal_solid_theme_and_pattern_ink_with_explicit_resets` (debug/render.rs:1317)

#### `color`  (L415-431)
`fn color(dom: &Dom, rpr: NodeId) -> Option<Option<String>>`

- **Does:** `rPr`'s text colour: the value, or `theme:…` for a theme colour; `Some(None)` for `auto`, which resets an inherited one.
- **Calls (crate):** `child` (debug/render.rs:361) L416; `attr` (debug.rs:641) L417,422,429
- **Calls (std/external/unresolved):** `.is_empty`, `.push_str`, `.then_some`, `Some`
- **Macros:** `format!`
- **Called by:** `StyleBook::new` (debug/render.rs:156); `run` (debug/render.rs:717); `story_diagnostic_contract_tests::shading_reports_literal_solid_theme_and_pattern_ink_with_explicit_resets` (debug/render.rs:1317)

#### `highlight`  (L434-437)
`fn highlight(dom: &Dom, rpr: NodeId) -> Option<Option<String>>`

- **Does:** `rPr`'s highlight; `Some(None)` for `none`, a reset.
- **Calls (crate):** `attr` (debug.rs:641) L435; `child` (debug/render.rs:361) L435
- **Calls (std/external/unresolved):** `.is_empty`, `.then_some`, `Some`
- **Called by:** `StyleBook::new` (debug/render.rs:156); `run` (debug/render.rs:717); `story_diagnostic_contract_tests::shading_reports_literal_solid_theme_and_pattern_ink_with_explicit_resets` (debug/render.rs:1317)

#### `run_shd`  (L440-442)
`fn run_shd(dom: &Dom, rpr: NodeId) -> Option<Option<String>>`

- **Does:** `rPr`'s shading; `Some(None)` for one that paints nothing (nil), a reset.
- **Calls (crate):** `child` (debug/render.rs:361) L441; `shd` (debug/render.rs:382) L441
- **Calls (std/external/unresolved):** `.map`
- **Called by:** `StyleBook::new` (debug/render.rs:156); `run` (debug/render.rs:717)

#### `on`  (L445-448)
`fn on(dom: &Dom, pr: NodeId, name: &str) -> bool`

- **Does:** A toggle property that is on (`w:val` absent, `1`, `true` or `on`).
- **Calls (crate):** `child` (debug/render.rs:361) L446; `attr` (debug.rs:641) L447
- **Calls (std/external/unresolved):** `.as_str`, `.is_some_and`
- **Macros:** `matches!`
- **Called by:** `parse_tbl_style` (convert/mod.rs:3854) (?); `table_block` (convert/mod.rs:13095) (?); `form_checkbox` (convert/mod.rs:16168) (?); `StyleBook::new` (debug/render.rs:156); `run` (debug/render.rs:717); `section` (debug/render.rs:1028); `analyze` (debug.rs:1220) (?); `run_format` (inspect.rs:1486) (?); `chpx_props` (legacy_doc.rs:1488) (?)

#### `own_text`  (L452-462)
`fn own_text(dom: &Dom, n: NodeId, para: NodeId) -> String`

- **Does:** Text `n` paints in its own paragraph: `w:t` and `w:delText` whose nearest paragraph is `para` (a text box's paragraphs are their own).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L454; `local` (debug.rs:381) L455; `nearest` (debug/render.rs:374) L455; `Dom::nodes` (xmllinq/mod.rs:604) L456; `Dom::text_value` (xmllinq/mod.rs:547) L457
- **Calls (std/external/unresolved):** `.as_str`, `.push_str`, `.unwrap_or_default`, `Some`, `String::new`
- **Macros:** `matches!`
- **Called by:** `paragraph` (debug/render.rs:590); `run` (debug/render.rs:717); `table` (debug/render.rs:869)

#### `Tally::next_run`  (L480-488)
`fn next_run(&mut self, para: NodeId)`

- **Does:** A run of `para` starts: the facts of the run before it carry on.
- **Calls (std/external/unresolved):** `.clear`, `Some`, `std::mem::take`
- **Called by:** `run` (debug/render.rs:717)

#### `Tally::add`  (L490-500)
`fn add(&mut self, key: String, sample: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `clip` (debug.rs:440) L497
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.entry`, `.insert`, `.or_default`
- **Called by:** `part_lines` (debug/render.rs:515); `paragraph` (debug/render.rs:590); `run` (debug/render.rs:717)

#### `Tally::lines`  (L502-510)
`fn lines(self, out: &mut Vec<String>)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_empty`, `.push`
- **Macros:** `format!`
- **Called by:** `part_lines` (debug/render.rs:515)

#### `part_lines`  (L515-587)
`fn part_lines( dom: &Dom, root: NodeId, styles: &StyleBook, rels: &[Rel], out: &mut Vec<String>, ) -> usize`

- **Does:** One story part's lines: sections, tables, frames and drawings in document order, then the tallies.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L528; `nearest` (debug/render.rs:374) L531,545; `local` (debug.rs:381) L534,551; `paragraph` (debug/render.rs:590) L537; `run` (debug/render.rs:717) L539; `table` (debug/render.rs:869) L542; `section` (debug/render.rs:1028) L547; `anchor` (debug/render.rs:1106) L549; `Tally::add` (debug/render.rs:490) L550,551,552,556,567; `attr` (debug.rs:641) L554,559; `Dom::nodes` (xmllinq/mod.rs:604) L577; `Dom::text_value` (xmllinq/mod.rs:547) L578; `Tally::lines` (debug/render.rs:502) L585
- **Calls (std/external/unresolved):** `.as_str`, `.is_none`, `.is_some`, `.last_mut`, `.next`, `.pop`, `.push_str`, `.push`, `.split_whitespace`, `.to_string`, `.to_uppercase`, `.unwrap_or_default`, `String::new`, `Tally::default`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `render_parts` (debug/render.rs:28); `story_diagnostic_contract_tests::report` (debug/render.rs:1284); `story_diagnostic_contract_tests::strict_rprs_tally_by_local_name` (debug/render.rs:1406)

#### `paragraph`  (L590-640)
`fn paragraph(dom: &Dom, p: NodeId, styles: &StyleBook, tally: &mut Tally, out: &mut Vec<String>)`

- **Does:** A paragraph's style, shading, frame and the order of its revisions.
- **Calls (crate):** `own_text` (debug/render.rs:452) L591; `child` (debug/render.rs:361) L592,594,601,617; `attr` (debug.rs:641) L595; `Tally::add` (debug/render.rs:490) L598,602,605,609,635,636; `StyleBook::para_style` (debug/render.rs:328) L600; `shd` (debug/render.rs:382) L601; `StyleBook::find` (debug/render.rs:338) L604; `StyleBook::frame` (debug/render.rs:307) L616; `merge_pairs` (debug/render.rs:859) L618; `attr_pairs` (debug/render.rs:852) L618; `clip` (debug.rs:440) L626; `revision_order` (debug/render.rs:645) L632
- **Calls (std/external/unresolved):** `.and_then`, `.array_windows`, `.clone`, `.collect`, `.is_empty`, `.iter`, `.join`, `.map`, `.name`, `.push`, `.sort`, `.to_string`, `.unwrap_or_default`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `part_lines` (debug/render.rs:515)

#### `revision_order`  (L645-662)
`fn revision_order(dom: &Dom, n: NodeId, order: &mut Vec<char>)`

- **Does:** The revision containers under `n` in document order, looking through hyperlinks, smart tags, content controls and customXml (their runs are the paragraph's own).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L646; `local` (debug.rs:381) L647; `paints` (debug/render.rs:667) L651; `revision_order` (debug/render.rs:645) L653
- **Calls (std/external/unresolved):** `.as_str`, `.last`, `.push`, `Some`
- **Called by:** `paragraph` (debug/render.rs:590); `revision_order` (debug/render.rs:645)

#### `paints`  (L667-678)
`fn paints(dom: &Dom, r: NodeId) -> bool`

- **Does:** A run that puts something on the page: text, a tab or break, a symbol, a field character or a drawing (anything but its properties, a rendered-page-break hint, an empty `w:t` or a comment reference).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L668; `local` (debug.rs:381) L670; `Dom::nodes` (xmllinq/mod.rs:604) L672; `Dom::text_value` (xmllinq/mod.rs:547) L675
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.into_iter`, `.is_empty`, `.unwrap_or_default`
- **Called by:** `revision_order` (debug/render.rs:645)

#### `paints_glyphs`  (L684-695)
`fn paints_glyphs(dom: &Dom, r: NodeId) -> bool`

- **Does:** A run that paints something a highlight can colour: text, a symbol, a tab (plain or positioned) or a hyphen glyph (a non-breaking or soft one).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L685; `local` (debug.rs:381) L687; `Dom::nodes` (xmllinq/mod.rs:604) L688; `Dom::text_value` (xmllinq/mod.rs:547) L691
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.into_iter`, `.is_empty`, `.unwrap_or_default`
- **Called by:** `run` (debug/render.rs:717)

#### `leading_rprs`  (L699-710)
`fn leading_rprs(dom: &Dom, r: NodeId) -> Vec<NodeId>`

- **Does:** The run's leading `rPr` blocks by local name: a Strict package writes them in the Strict namespace, which a qualified lookup misses.
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L701; `Dom::child_at` (xmllinq/mod.rs:632) L702; `local` (debug.rs:381) L703; `Dom::text_value` (xmllinq/mod.rs:547) L705
- **Calls (std/external/unresolved):** `.is_none`, `.push`, `Vec::new`
- **Called by:** `run` (debug/render.rs:717)

#### `run`  (L717-825)
`fn run(dom: &Dom, r: NodeId, styles: &StyleBook, tally: &mut Tally)`

- **Does:** A run's colour, highlight, shading, hidden state and requested font, direct or from its character or paragraph style, over `docDefaults`.
- **Calls (crate):** `nearest` (debug/render.rs:374) L718; `Tally::next_run` (debug/render.rs:480) L721; `own_text` (debug/render.rs:452) L722; `leading_rprs` (debug/render.rs:699) L723; `child` (debug/render.rs:361) L725,729,774,806; `attr` (debug.rs:641) L726; `StyleBook::para_style` (debug/render.rs:328) L729; `StyleBook::find` (debug/render.rs:338) L736,744; `Tally::add` (debug/render.rs:490) L754,755,778,779,820,821; `color` (debug/render.rs:415) L758; `paints_glyphs` (debug/render.rs:684) L761; `highlight` (debug/render.rs:434) L764; `run_shd` (debug/render.rs:440) L769; `on` (debug/render.rs:445) L774; `StyleBook::font_slots` (debug/render.rs:282) L784,799; `font_slots` (debug/render.rs:830) L807
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.clone`, `.collect`, `.flatten`, `.into_iter`, `.is_empty`, `.iter`, `.map`, `.push`, `.retain`, `.then`, `.to_string`, `.unwrap_or_default`, `Some`, `Vec::new`, `add`, `as_value`, `fold`, `from_style`, `get`, `overlay`, `style_of`
- **Macros:** `format!`
- **Called by:** `part_lines` (debug/render.rs:515)

#### `font_slots`  (L830-849)
`fn font_slots(dom: &Dom, f: NodeId) -> Vec<(&'static str, String)>`

- **Does:** Each slot's face in `w:rFonts` (a theme font where the slot names none): a run can paint Latin, East Asian and complex-script text in different faces.
- **Calls (crate):** `attr` (debug.rs:641) L838
- **Calls (std/external/unresolved):** `.is_empty`, `.push`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `StyleBook::new` (debug/render.rs:156); `run` (debug/render.rs:717)

#### `attr_pairs`  (L852-856)
`fn attr_pairs(dom: &Dom, n: NodeId) -> Vec<(String, String)>`

- **Does:** An element's attributes as `(local name, value)`.
- **Calls (crate):** `Dom::attributes_iter` (xmllinq/mod.rs:886) L853; `XName::local_name` (xmllinq/mod.rs:208) (by name) L854
- **Calls (std/external/unresolved):** `.collect`, `.map`, `.to_string`
- **Called by:** `StyleBook::new` (debug/render.rs:156); `paragraph` (debug/render.rs:590)

#### `merge_pairs`  (L859-866)
`fn merge_pairs(base: &mut Vec<(String, String)>, over: &[(String, String)])`

- **Does:** `over`'s values replace or join `base`'s.
- **Calls (std/external/unresolved):** `.clone_from`, `.clone`, `.find`, `.iter_mut`, `.push`
- **Called by:** `StyleBook::frame` (debug/render.rs:307); `paragraph` (debug/render.rs:590)

#### `table`  (L869-1025)
`fn table(dom: &Dom, t: NodeId, n: usize, styles: &StyleBook) -> String`

- **Does:** `table N RxC style=… float(…) shd=… cells-shd[…] style-shd[…] "first row"`.
- **Calls (crate):** `child` (debug/render.rs:361) L870,877,881,883,891,892...; `wrapped_children` (convert/mod.rs:14580) L873,887,1012,1015; `kids` (debug/render.rs:354) L877; `attr` (debug.rs:641) L884,893,915,944; `cell_is_deleted` (convert/mod.rs:14330) L900; `nearest` (debug/render.rs:374) L907,910,966; `Dom::attributes_iter` (xmllinq/mod.rs:886) L921; `XName::local_name` (xmllinq/mod.rs:208) (by name) L923; `shd` (debug/render.rs:382) L930,971; `Dom::elements` (xmllinq/mod.rs:640) L939; `side` (markdown/borders.rs:189) L943; `local` (debug.rs:381) L943,966; `Border::new` (markdown/borders.rs:51) L945; `describe` (markdown/borders.rs:209) L954; `ordered` (markdown/borders.rs:200) L954; `Dom::descendants` (xmllinq/mod.rs:668) L965; `table_look` (convert/mod.rs:12978) L996; `StyleBook::table_shd` (debug/render.rs:252) L997; `own_text` (debug/render.rs:452) L1017; `clip` (debug.rs:440) L1022
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.as_str`, `.collect`, `.entry`, `.filter_map`, `.filter`, `.first`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.join`, `.len`, `.map_or`, `.map`, `.max`, `.name`, `.ok`, `.or_default`, `.parse`, `.push_str`, `.sort`, `.sum`, `.unwrap_or_default`, `.unwrap_or`, `BTreeMap::new`, `Some`, `borders`, `get`, `skip`, `usize::from`
- **Macros:** `format!`
- **Called by:** `StyleBook::new` (debug/render.rs:156); `part_lines` (debug/render.rs:515)

#### `section`  (L1028-1103)
`fn section(dom: &Dom, sect: NodeId, n: usize, rels: &[Rel]) -> String`

- **Does:** `section N page=WxH margins=t,r,b,l header=… footer=… cols=… TYPE-KIND=part`.
- **Calls (crate):** `child` (debug/render.rs:361) L1030,1044,1056,1077; `attr` (debug.rs:641) L1033,1034,1039,1040,1045,1061...; `number` (ambiguous: `deep_lcs_source_conservation_tests::number` (comparer/lcs.rs:15436); `number` (ids.rs:211)) L1040; `kids` (debug/render.rs:354) L1058; `on` (debug/render.rs:445) L1080; `Dom::elements` (xmllinq/mod.rs:640) L1083; `local` (debug.rs:381) L1084
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.collect`, `.find`, `.into_iter`, `.is_empty`, `.iter`, `.join`, `.len`, `.map_or_else`, `.map`, `.parse`, `.push_str`, `.to_string`, `.unwrap_or`, `a`
- **Macros:** `format!`, `matches!`
- **Called by:** `part_lines` (debug/render.rs:515)

#### `anchor`  (L1106-1151)
`fn anchor(dom: &Dom, a: NodeId) -> String`

- **Does:** `anchor wrap H=from:offset V=from:offset CXxCY [behind] "name"`.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1107,1115; `local` (debug.rs:381) L1110,1124; `child` (debug/render.rs:361) L1114,1135,1143; `Dom::nodes` (xmllinq/mod.rs:604) L1119; `Dom::text_value` (xmllinq/mod.rs:547) L1122; `attr` (debug.rs:641) L1132,1136,1138,1144
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.filter_map`, `.find`, `.into_iter`, `.map_or_else`, `.map`, `.next`, `.starts_with`, `.to_string`, `.trim`, `.unwrap_or_default`, `pos`
- **Macros:** `format!`, `matches!`
- **Called by:** `part_lines` (debug/render.rs:515); `tests::render_marks_a_true_behind_doc_anchor` (debug.rs:4012) (?)

#### `layout_lines`  (L1156-1238)
`fn layout_lines(pkg: &Package, rels: &[Rel]) -> (Vec<String>, usize)`

- **Does:** `(layout)`: jubarte's page count and font resolution (Word revision style, as the bench renders it), then the fonts the package asks for: `docDefaults`, the theme and `fontTable.xml`.
- **Calls (crate):** `docx_render_report` (convert/mod.rs:735) L1162; `parse` (debug/render.rs:104) L1196,1204,1218; `StyleBook::new` (debug/render.rs:156) L1197; `descendant` (debug/render.rs:367) L1207; `child` (debug/render.rs:361) L1208,1222; `attr` (debug.rs:641) L1209,1221,1223; `kids` (debug/render.rs:354) L1220; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:372); `Dom::elements` (xmllinq/mod.rs:640)) L1225; `local` (debug.rs:381) L1228
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.extend`, `.filter_map`, `.find`, `.into_iter`, `.is_empty`, `.iter`, `.join`, `.map`, `.push_str`, `.push`, `.sort`, `.strip_prefix`, `Default::default`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `render_parts` (debug/render.rs:28)

## Test functions (22)

- `story_diagnostic_contract_tests::dom` (L1250): _no doc comment_
- `story_diagnostic_contract_tests::document` (L1257): _no doc comment_
- `story_diagnostic_contract_tests::package` (L1264): The dependency's own in-memory ZIP writer, also used by debug.rs tests. _(from a code comment)_
- `story_diagnostic_contract_tests::report` (L1284): _no doc comment_
- `story_diagnostic_contract_tests::nested_complex_fields_record_each_code_once_and_ignore_unowned_instructions` (L1294): _no doc comment_
- `story_diagnostic_contract_tests::shading_reports_literal_solid_theme_and_pattern_ink_with_explicit_resets` (L1317): _no doc comment_
- `story_diagnostic_contract_tests::inherited_frame_overrides_and_repeated_run_tallies_keep_first_source_sample` (L1361): _no doc comment_
- `story_diagnostic_contract_tests::a_stretch_split_into_runs_counts_once_and_a_gap_counts_twice` (L1385): _no doc comment_
- `story_diagnostic_contract_tests::strict_rprs_tally_by_local_name` (L1406): _no doc comment_
- `story_diagnostic_contract_tests::a_highlight_on_a_run_that_paints_no_glyph_is_not_tallied` (L1422): _no doc comment_
- `story_diagnostic_contract_tests::a_highlight_on_a_hyphen_or_position_tab_is_tallied` (L1439): _no doc comment_
- `story_diagnostic_contract_tests::a_table_reports_its_own_borders_and_its_cells` (L1470): _no doc comment_
- `story_diagnostic_contract_tests::nested_table_shading_and_textbox_frames_report_only_their_own_text` (L1483): _no doc comment_
- `story_diagnostic_contract_tests::section_columns_geometry_and_story_reference_diagnostics_are_literal` (L1507): _no doc comment_
- `story_diagnostic_contract_tests::section_geometry_prints_one_spelling_of_the_same_measure` (L1554): _no doc comment_
- `story_diagnostic_contract_tests::section_prints_a_plain_value_as_the_file_spells_it` (L1602): _no doc comment_
- `story_diagnostic_contract_tests::section_orientation_is_the_pages_not_the_attributes` (L1631): _no doc comment_
- `story_diagnostic_contract_tests::anchor_alignment_and_offsets_preserve_authored_extent_name_and_layer` (L1668): _no doc comment_
- `story_diagnostic_contract_tests::memory_package_filter_resolves_story_roles_and_excludes_nonstories_without_mutation` (L1691): _no doc comment_
- `story_diagnostic_contract_tests::malformed_package_relationship_diagnostics_do_not_invent_main_or_story_parts` (L1745): _no doc comment_
- `story_diagnostic_contract_tests::table_style_resets_and_look_flags_report_only_live_conditional_fills` (L1769): _no doc comment_
- `story_diagnostic_contract_tests::revisions_cross_transparent_owners_but_live_field_payload_breaks_adjacency` (L1801): _no doc comment_
