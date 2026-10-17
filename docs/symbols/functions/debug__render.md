# Functions and methods in `src/debug/render.rs`

37 non-test, 22 test. Part of the [symbol manifest](../README.md).

#### `render_parts`  (L28-64, pub(super))
`pub(super) fn render_parts( pkg: &Package, part: Option<&str>, ) -> BTreeMap<String, (Vec<String>, usize)>`

- **Does:** Part name → (lines, paragraph or page count), the `(layout)` entry last.
- **Calls (crate):** `main_rels` (debug/render.rs:76) L32; `parse` (debug/render.rs:104) L36,45; `StyleBook::new` (debug/render.rs:156) L37; `story_roles_all` (debug.rs:1867) L39; `local` (debug.rs:277) L48; `part_lines` (debug/render.rs:516) L57; `layout_lines` (debug/render.rs:1159) L61
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.clone`, `.collect`, `.contains`, `.ends_with`, `.find`, `.flatten`, `.get`, `.insert`, `.into_iter`, `.is_none_or`, `.is_some_and`, `.iter`, `.map`, `.to_string`, `.unwrap_or_default`, `BTreeMap::new`
- **Macros:** `format!`
- **Called by:** `story_diagnostic_contract_tests::section_geometry_prints_one_spelling_of_the_same_measure` (debug/render.rs:1557); `story_diagnostic_contract_tests::section_prints_a_plain_value_as_the_file_spells_it` (debug/render.rs:1605); `story_diagnostic_contract_tests::memory_package_filter_resolves_story_roles_and_excludes_nonstories_without_mutation` (debug/render.rs:1694); `story_diagnostic_contract_tests::malformed_package_relationship_diagnostics_do_not_invent_main_or_story_parts` (debug/render.rs:1748); `listing_lines` (debug.rs:1940)

#### `main_rels`  (L76-102)
`fn main_rels(pkg: &Package) -> Vec<Rel>`

- **Does:** The main document part's relationships, found through the package's own `officeDocument` relationship (the part need not be `word/document.xml`).
- **Calls (crate):** `parse` (debug/render.rs:104) L78; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L81; `attr` (debug.rs:523) L83,85,90,91; `resolve_target` (debug.rs:700) L91; `percent_decode` (debug.rs:678) L91; `rels_of` (debug.rs:723) L101
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.find`, `.into_iter`, `.map`, `.next`, `.rsplit`, `.to_string`, `.unwrap_or_default`, `Vec::new`, `rels_in`
- **Called by:** `render_parts` (debug/render.rs:28); `story_diagnostic_contract_tests::malformed_package_relationship_diagnostics_do_not_invent_main_or_story_parts` (debug/render.rs:1748)

#### `parse`  (L104-115)
`fn parse(pkg: &Package, name: &str) -> Option<(Dom, NodeId)>`

- **Does:** _no doc comment_
- **Calls (crate):** `decode_xml` (debug.rs:239) L106; `Dom::new` (xmllinq/mod.rs:387) L107; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L108; `Dom::root` (xmllinq/mod.rs:822) L109; `normalize_unit_measures` (comparer/finalize.rs:11493) L113
- **Calls (std/external/unresolved):** `.find`, `.iter`, `Some`
- **Called by:** `render_parts` (debug/render.rs:28); `main_rels` (debug/render.rs:76); `layout_lines` (debug/render.rs:1159); `story_diagnostic_contract_tests::malformed_package_relationship_diagnostics_do_not_invent_main_or_story_parts` (debug/render.rs:1748)

#### `StyleBook::new`  (L156-240)
`fn new(dom: &Dom, root: NodeId) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L158; `local` (debug.rs:277) L159; `descendant` (debug/render.rs:368) L161,164,175; `child` (debug/render.rs:362) L161,165,166,172,186,187...; `shd` (debug/render.rs:383) L167,193,213; `color` (debug/render.rs:416) L168,194; `highlight` (debug/render.rs:435) L169,195; `run_shd` (debug/render.rs:441) L170,196; `on` (debug/render.rs:446) L172,198; `Dom::attributes` (xmllinq/mod.rs:885) L176; `XName::local_name` (xmllinq/mod.rs:208) (by name) L179; `attr` (debug.rs:523) L186,226,228,229,230; `attr_pairs` (debug/render.rs:853) L201; `font_slots` (debug/render.rs:831) L205; `table` (debug/render.rs:871) L224,226; `kids` (debug/render.rs:355) L225
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.clone_from`, `.clone`, `.collect`, `.insert`, `.into_iter`, `.is_empty`, `.join`, `.map`, `.push`, `.sort`, `.to_string`, `.unwrap_or_default`, `String::new`, `StyleBook::default`, `StyleInfo::default`, `Vec::new`, `val`
- **Macros:** `format!`, `matches!`
- **Called by:** `render_parts` (debug/render.rs:28); `layout_lines` (debug/render.rs:1159); `story_diagnostic_contract_tests::inherited_frame_overrides_and_repeated_run_tallies_keep_first_source_sample` (debug/render.rs:1364); `story_diagnostic_contract_tests::table_style_resets_and_look_flags_report_only_live_conditional_fills` (debug/render.rs:1772)

#### `StyleBook::name`  (L242-248)
`fn name(&self, id: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone`, `.filter`, `.get`, `.is_empty`, `.map`, `.to_string`, `.unwrap_or_else`
- **Called by:** `StyleBook::font_slots` (debug/render.rs:283); `StyleBook::find` (debug/render.rs:339)

#### `StyleBook::table_shd`  (L253-278)
`fn table_shd(&self, id: &str) -> Vec<(String, String)>`

- **Does:** `id`'s table shading down its `basedOn` chain: a base's conditions stay unless the derived style sets the same one; a nil `shd` resets it.
- **Calls (std/external/unresolved):** `.clone_from`, `.clone`, `.collect`, `.filter_map`, `.find`, `.get`, `.into_iter`, `.is_empty`, `.iter_mut`, `.push`, `.rev`, `.to_string`, `Some`, `Vec::new`
- **Called by:** `table` (debug/render.rs:871)

#### `StyleBook::font_slots`  (L283-304)
`fn font_slots(&self, id: &str) -> Vec<(&'static str, String, String)>`

- **Does:** The faces `id`'s `basedOn` chain asks for, slot by slot, each with the name of the style that sets it (a derived style's slot over its base's, as the converter overlays them).
- **Calls (crate):** `StyleBook::name` (debug/render.rs:242) L290
- **Calls (std/external/unresolved):** `.clone`, `.get`, `.into_iter`, `.is_empty`, `.push`, `.retain`, `.rev`, `.to_string`, `Vec::new`
- **Called by:** `run` (debug/render.rs:718)

#### `StyleBook::frame`  (L308-326)
`fn frame(&self, id: &str) -> Vec<(String, String)>`

- **Does:** `id`'s frame attributes down its `basedOn` chain, a derived style's value over its base's.
- **Calls (crate):** `merge_pairs` (debug/render.rs:861) L323
- **Calls (std/external/unresolved):** `.clone`, `.get`, `.into_iter`, `.is_empty`, `.push`, `.rev`, `.to_string`, `Vec::new`
- **Called by:** `paragraph` (debug/render.rs:591)

#### `StyleBook::para_style`  (L329-335)
`fn para_style(&self, id: String) -> String`

- **Does:** The paragraph style `id` names, or the default one.
- **Calls (std/external/unresolved):** `.clone`, `.is_empty`
- **Called by:** `paragraph` (debug/render.rs:591); `run` (debug/render.rs:718)

#### `StyleBook::find`  (L339-352)
`fn find<T>(&self, id: &str, f: impl Fn(&StyleInfo) -> Option<T>) -> Option<(T, String)>`

- **Does:** The first value `f` finds walking `id`'s `basedOn` chain, with the name of the style that holds it.
- **Calls (crate):** `StyleBook::name` (debug/render.rs:242) L344
- **Calls (std/external/unresolved):** `.clone`, `.get`, `.is_empty`, `.to_string`, `Some`, `f`
- **Called by:** `paragraph` (debug/render.rs:591); `run` (debug/render.rs:718)

#### `kids`  (L355-360)
`fn kids(dom: &Dom, n: NodeId, name: &str) -> Vec<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L356; `local` (debug.rs:277) L358
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`
- **Called by:** `StyleBook::new` (debug/render.rs:156); `table` (debug/render.rs:871); `section` (debug/render.rs:1031); `layout_lines` (debug/render.rs:1159); `numbering_lines` (debug.rs:1432) (?)

#### `child`  (L362-366)
`fn child(dom: &Dom, n: NodeId, name: &str) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L363; `local` (debug.rs:277) L365
- **Calls (std/external/unresolved):** `.find`, `.into_iter`
- **Called by:** `StyleBook::new` (debug/render.rs:156); `color` (debug/render.rs:416); `highlight` (debug/render.rs:435); `run_shd` (debug/render.rs:441); `on` (debug/render.rs:446); `paragraph` (debug/render.rs:591); `run` (debug/render.rs:718); `table` (debug/render.rs:871); `section` (debug/render.rs:1031); `anchor` (debug/render.rs:1109); `layout_lines` (debug/render.rs:1159)

#### `descendant`  (L368-372)
`fn descendant(dom: &Dom, n: NodeId, name: &str) -> Option<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L369; `local` (debug.rs:277) L371
- **Calls (std/external/unresolved):** `.find`, `.into_iter`
- **Called by:** `StyleBook::new` (debug/render.rs:156); `layout_lines` (debug/render.rs:1159)

#### `nearest`  (L375-379)
`fn nearest(dom: &Dom, n: NodeId, name: &str) -> Option<NodeId>`

- **Does:** The nearest ancestor of `n` named `name`.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L376; `local` (debug.rs:277) L378
- **Calls (std/external/unresolved):** `.find`, `.into_iter`
- **Called by:** `own_text` (debug/render.rs:453); `part_lines` (debug/render.rs:516); `run` (debug/render.rs:718); `table` (debug/render.rs:871); `Ids::table_paragraph` (edit/origin.rs:95) (?); `operations` (markdown/patch.rs:496) (?)

#### `shd`  (L383-412)
`fn shd(dom: &Dom, x: NodeId) -> Option<String>`

- **Does:** A `w:shd`'s visible fill: `fill` (or `theme:…`), the pattern's colour after `/` when the pattern is not clear.
- **Calls (crate):** `attr` (debug.rs:523) L385,386,387,388
- **Calls (std/external/unresolved):** `.as_str`, `.is_empty`, `.then`, `.to_string`, `.unwrap_or_default`, `Some`, `String::new`
- **Macros:** `format!`, `matches!`
- **Called by:** `StyleBook::new` (debug/render.rs:156); `run_shd` (debug/render.rs:441); `paragraph` (debug/render.rs:591); `table` (debug/render.rs:871); `story_diagnostic_contract_tests::shading_reports_literal_solid_theme_and_pattern_ink_with_explicit_resets` (debug/render.rs:1320)

#### `color`  (L416-432)
`fn color(dom: &Dom, rpr: NodeId) -> Option<Option<String>>`

- **Does:** `rPr`'s text colour: the value, or `theme:…` for a theme colour; `Some(None)` for `auto`, which resets an inherited one.
- **Calls (crate):** `child` (debug/render.rs:362) L417; `attr` (debug.rs:523) L418,423,430
- **Calls (std/external/unresolved):** `.is_empty`, `.push_str`, `.then_some`, `Some`
- **Macros:** `format!`
- **Called by:** `StyleBook::new` (debug/render.rs:156); `run` (debug/render.rs:718); `story_diagnostic_contract_tests::shading_reports_literal_solid_theme_and_pattern_ink_with_explicit_resets` (debug/render.rs:1320)

#### `highlight`  (L435-438)
`fn highlight(dom: &Dom, rpr: NodeId) -> Option<Option<String>>`

- **Does:** `rPr`'s highlight; `Some(None)` for `none`, a reset.
- **Calls (crate):** `attr` (debug.rs:523) L436; `child` (debug/render.rs:362) L436
- **Calls (std/external/unresolved):** `.is_empty`, `.then_some`, `Some`
- **Called by:** `StyleBook::new` (debug/render.rs:156); `run` (debug/render.rs:718); `story_diagnostic_contract_tests::shading_reports_literal_solid_theme_and_pattern_ink_with_explicit_resets` (debug/render.rs:1320)

#### `run_shd`  (L441-443)
`fn run_shd(dom: &Dom, rpr: NodeId) -> Option<Option<String>>`

- **Does:** `rPr`'s shading; `Some(None)` for one that paints nothing (nil), a reset.
- **Calls (crate):** `child` (debug/render.rs:362) L442; `shd` (debug/render.rs:383) L442
- **Calls (std/external/unresolved):** `.map`
- **Called by:** `StyleBook::new` (debug/render.rs:156); `run` (debug/render.rs:718)

#### `on`  (L446-449)
`fn on(dom: &Dom, pr: NodeId, name: &str) -> bool`

- **Does:** A toggle property that is on (`w:val` absent, `1`, `true` or `on`).
- **Calls (crate):** `child` (debug/render.rs:362) L447; `attr` (debug.rs:523) L448
- **Calls (std/external/unresolved):** `.as_str`, `.is_some_and`
- **Macros:** `matches!`
- **Called by:** `parse_tbl_style` (convert/mod.rs:3853) (?); `table_block` (convert/mod.rs:13059) (?); `form_checkbox` (convert/mod.rs:16132) (?); `StyleBook::new` (debug/render.rs:156); `run` (debug/render.rs:718); `section` (debug/render.rs:1031); `analyze` (debug.rs:1057) (?); `run_format` (inspect.rs:1245) (?); `chpx_props` (legacy_doc.rs:1450) (?)

#### `own_text`  (L453-463)
`fn own_text(dom: &Dom, n: NodeId, para: NodeId) -> String`

- **Does:** Text `n` paints in its own paragraph: `w:t` and `w:delText` whose nearest paragraph is `para` (a text box's paragraphs are their own).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L455; `local` (debug.rs:277) L456; `nearest` (debug/render.rs:375) L456; `Dom::nodes` (xmllinq/mod.rs:604) L457; `Dom::text_value` (xmllinq/mod.rs:547) L458
- **Calls (std/external/unresolved):** `.as_str`, `.push_str`, `.unwrap_or_default`, `Some`, `String::new`
- **Macros:** `matches!`
- **Called by:** `paragraph` (debug/render.rs:591); `run` (debug/render.rs:718); `table` (debug/render.rs:871)

#### `Tally::next_run`  (L481-489)
`fn next_run(&mut self, para: NodeId)`

- **Does:** A run of `para` starts: the facts of the run before it carry on.
- **Calls (std/external/unresolved):** `.clear`, `Some`, `std::mem::take`
- **Called by:** `run` (debug/render.rs:718)

#### `Tally::add`  (L491-501)
`fn add(&mut self, key: String, sample: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `clip` (debug.rs:336) L498
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.entry`, `.insert`, `.or_default`
- **Called by:** `part_lines` (debug/render.rs:516); `paragraph` (debug/render.rs:591); `run` (debug/render.rs:718)

#### `Tally::lines`  (L503-511)
`fn lines(self, out: &mut Vec<String>)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_empty`, `.push`
- **Macros:** `format!`
- **Called by:** `part_lines` (debug/render.rs:516)

#### `part_lines`  (L516-588)
`fn part_lines( dom: &Dom, root: NodeId, styles: &StyleBook, rels: &[Rel], out: &mut Vec<String>, ) -> usize`

- **Does:** One story part's lines: sections, tables, frames and drawings in document order, then the tallies.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L529; `nearest` (debug/render.rs:375) L532,546; `local` (debug.rs:277) L535,552; `paragraph` (debug/render.rs:591) L538; `run` (debug/render.rs:718) L540; `table` (debug/render.rs:871) L543; `section` (debug/render.rs:1031) L548; `anchor` (debug/render.rs:1109) L550; `Tally::add` (debug/render.rs:491) L551,552,553,557,568; `attr` (debug.rs:523) L555,560; `Dom::nodes` (xmllinq/mod.rs:604) L578; `Dom::text_value` (xmllinq/mod.rs:547) L579; `Tally::lines` (debug/render.rs:503) L586
- **Calls (std/external/unresolved):** `.as_str`, `.is_none`, `.is_some`, `.last_mut`, `.next`, `.pop`, `.push_str`, `.push`, `.split_whitespace`, `.to_string`, `.to_uppercase`, `.unwrap_or_default`, `String::new`, `Tally::default`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `render_parts` (debug/render.rs:28); `story_diagnostic_contract_tests::report` (debug/render.rs:1287); `story_diagnostic_contract_tests::strict_rprs_tally_by_local_name` (debug/render.rs:1409)

#### `paragraph`  (L591-641)
`fn paragraph(dom: &Dom, p: NodeId, styles: &StyleBook, tally: &mut Tally, out: &mut Vec<String>)`

- **Does:** A paragraph's style, shading, frame and the order of its revisions.
- **Calls (crate):** `own_text` (debug/render.rs:453) L592; `child` (debug/render.rs:362) L593,595,602,618; `attr` (debug.rs:523) L596; `Tally::add` (debug/render.rs:491) L599,603,606,610,636,637; `StyleBook::para_style` (debug/render.rs:329) L601; `shd` (debug/render.rs:383) L602; `StyleBook::find` (debug/render.rs:339) L605; `StyleBook::frame` (debug/render.rs:308) L617; `merge_pairs` (debug/render.rs:861) L619; `attr_pairs` (debug/render.rs:853) L619; `clip` (debug.rs:336) L627; `revision_order` (debug/render.rs:646) L633
- **Calls (std/external/unresolved):** `.and_then`, `.array_windows`, `.clone`, `.collect`, `.is_empty`, `.iter`, `.join`, `.map`, `.name`, `.push`, `.sort`, `.to_string`, `.unwrap_or_default`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `part_lines` (debug/render.rs:516)

#### `revision_order`  (L646-663)
`fn revision_order(dom: &Dom, n: NodeId, order: &mut Vec<char>)`

- **Does:** The revision containers under `n` in document order, looking through hyperlinks, smart tags, content controls and customXml (their runs are the paragraph's own).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L647; `local` (debug.rs:277) L648; `paints` (debug/render.rs:668) L652; `revision_order` (debug/render.rs:646) L654
- **Calls (std/external/unresolved):** `.as_str`, `.last`, `.push`, `Some`
- **Called by:** `paragraph` (debug/render.rs:591); `revision_order` (debug/render.rs:646)

#### `paints`  (L668-679)
`fn paints(dom: &Dom, r: NodeId) -> bool`

- **Does:** A run that puts something on the page: text, a tab or break, a symbol, a field character or a drawing (anything but its properties, a rendered-page-break hint, an empty `w:t` or a comment reference).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L669; `local` (debug.rs:277) L671; `Dom::nodes` (xmllinq/mod.rs:604) L673; `Dom::text_value` (xmllinq/mod.rs:547) L676
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.into_iter`, `.is_empty`, `.unwrap_or_default`
- **Called by:** `revision_order` (debug/render.rs:646)

#### `paints_glyphs`  (L685-696)
`fn paints_glyphs(dom: &Dom, r: NodeId) -> bool`

- **Does:** A run that paints something a highlight can colour: text, a symbol, a tab (plain or positioned) or a hyphen glyph (a non-breaking or soft one).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L686; `local` (debug.rs:277) L688; `Dom::nodes` (xmllinq/mod.rs:604) L689; `Dom::text_value` (xmllinq/mod.rs:547) L692
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.into_iter`, `.is_empty`, `.unwrap_or_default`
- **Called by:** `run` (debug/render.rs:718)

#### `leading_rprs`  (L700-711)
`fn leading_rprs(dom: &Dom, r: NodeId) -> Vec<NodeId>`

- **Does:** The run's leading `rPr` blocks by local name: a Strict package writes them in the Strict namespace, which a qualified lookup misses.
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L702; `Dom::child_at` (xmllinq/mod.rs:632) L703; `local` (debug.rs:277) L704; `Dom::text_value` (xmllinq/mod.rs:547) L706
- **Calls (std/external/unresolved):** `.is_none`, `.push`, `Vec::new`
- **Called by:** `run` (debug/render.rs:718)

#### `run`  (L718-826)
`fn run(dom: &Dom, r: NodeId, styles: &StyleBook, tally: &mut Tally)`

- **Does:** A run's colour, highlight, shading, hidden state and requested font, direct or from its character or paragraph style, over `docDefaults`.
- **Calls (crate):** `nearest` (debug/render.rs:375) L719; `Tally::next_run` (debug/render.rs:481) L722; `own_text` (debug/render.rs:453) L723; `leading_rprs` (debug/render.rs:700) L724; `child` (debug/render.rs:362) L726,730,775,807; `attr` (debug.rs:523) L727; `StyleBook::para_style` (debug/render.rs:329) L730; `StyleBook::find` (debug/render.rs:339) L737,745; `Tally::add` (debug/render.rs:491) L755,756,779,780,821,822; `color` (debug/render.rs:416) L759; `paints_glyphs` (debug/render.rs:685) L762; `highlight` (debug/render.rs:435) L765; `run_shd` (debug/render.rs:441) L770; `on` (debug/render.rs:446) L775; `StyleBook::font_slots` (debug/render.rs:283) L785,800; `font_slots` (debug/render.rs:831) L808
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.clone`, `.collect`, `.flatten`, `.into_iter`, `.is_empty`, `.iter`, `.map`, `.push`, `.retain`, `.then`, `.to_string`, `.unwrap_or_default`, `Some`, `Vec::new`, `add`, `as_value`, `fold`, `from_style`, `get`, `overlay`, `style_of`
- **Macros:** `format!`
- **Called by:** `part_lines` (debug/render.rs:516)

#### `font_slots`  (L831-850)
`fn font_slots(dom: &Dom, f: NodeId) -> Vec<(&'static str, String)>`

- **Does:** Each slot's face in `w:rFonts` (a theme font where the slot names none): a run can paint Latin, East Asian and complex-script text in different faces.
- **Calls (crate):** `attr` (debug.rs:523) L839
- **Calls (std/external/unresolved):** `.is_empty`, `.push`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `StyleBook::new` (debug/render.rs:156); `run` (debug/render.rs:718)

#### `attr_pairs`  (L853-858)
`fn attr_pairs(dom: &Dom, n: NodeId) -> Vec<(String, String)>`

- **Does:** An element's attributes as `(local name, value)`.
- **Calls (crate):** `Dom::attributes` (xmllinq/mod.rs:885) L854; `XName::local_name` (xmllinq/mod.rs:208) (by name) L856
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`, `.to_string`
- **Called by:** `StyleBook::new` (debug/render.rs:156); `paragraph` (debug/render.rs:591)

#### `merge_pairs`  (L861-868)
`fn merge_pairs(base: &mut Vec<(String, String)>, over: &[(String, String)])`

- **Does:** `over`'s values replace or join `base`'s.
- **Calls (std/external/unresolved):** `.clone_from`, `.clone`, `.find`, `.iter_mut`, `.push`
- **Called by:** `StyleBook::frame` (debug/render.rs:308); `paragraph` (debug/render.rs:591)

#### `table`  (L871-1028)
`fn table(dom: &Dom, t: NodeId, n: usize, styles: &StyleBook) -> String`

- **Does:** `table N RxC style=… float(…) shd=… cells-shd[…] style-shd[…] "first row"`.
- **Calls (crate):** `child` (debug/render.rs:362) L872,879,883,885,893,894...; `wrapped_children` (convert/mod.rs:14544) L875,889,1015,1018; `kids` (debug/render.rs:355) L879; `attr` (debug.rs:523) L886,895,917,947; `cell_is_deleted` (convert/mod.rs:14294) L902; `nearest` (debug/render.rs:375) L909,912,969; `Dom::attributes` (xmllinq/mod.rs:885) L923; `XName::local_name` (xmllinq/mod.rs:208) (by name) L926; `shd` (debug/render.rs:383) L933,974; `Dom::elements` (xmllinq/mod.rs:640) L942; `side` (markdown/borders.rs:189) L946; `local` (debug.rs:277) L946,969; `Border::new` (markdown/borders.rs:51) L948; `describe` (markdown/borders.rs:209) L957; `ordered` (markdown/borders.rs:200) L957; `Dom::descendants` (xmllinq/mod.rs:668) L968; `table_look` (convert/mod.rs:12942) L999; `StyleBook::table_shd` (debug/render.rs:253) L1000; `own_text` (debug/render.rs:453) L1020; `clip` (debug.rs:336) L1025
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.as_str`, `.collect`, `.entry`, `.filter_map`, `.filter`, `.first`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.join`, `.len`, `.map_or`, `.map`, `.max`, `.name`, `.ok`, `.or_default`, `.parse`, `.push_str`, `.sort`, `.sum`, `.unwrap_or_default`, `.unwrap_or`, `BTreeMap::new`, `Some`, `borders`, `get`, `skip`, `usize::from`
- **Macros:** `format!`
- **Called by:** `StyleBook::new` (debug/render.rs:156); `part_lines` (debug/render.rs:516)

#### `section`  (L1031-1106)
`fn section(dom: &Dom, sect: NodeId, n: usize, rels: &[Rel]) -> String`

- **Does:** `section N page=WxH margins=t,r,b,l header=… footer=… cols=… TYPE-KIND=part`.
- **Calls (crate):** `child` (debug/render.rs:362) L1033,1047,1059,1080; `attr` (debug.rs:523) L1036,1037,1042,1043,1048,1064...; `number` (ambiguous: `deep_lcs_source_conservation_tests::number` (comparer/lcs.rs:15436); `number` (ids.rs:211)) L1043; `kids` (debug/render.rs:355) L1061; `on` (debug/render.rs:446) L1083; `Dom::elements` (xmllinq/mod.rs:640) L1086; `local` (debug.rs:277) L1087
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.collect`, `.find`, `.into_iter`, `.is_empty`, `.iter`, `.join`, `.len`, `.map_or_else`, `.map`, `.parse`, `.push_str`, `.to_string`, `.unwrap_or`, `a`
- **Macros:** `format!`, `matches!`
- **Called by:** `part_lines` (debug/render.rs:516)

#### `anchor`  (L1109-1154)
`fn anchor(dom: &Dom, a: NodeId) -> String`

- **Does:** `anchor wrap H=from:offset V=from:offset CXxCY [behind] "name"`.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1110,1118; `local` (debug.rs:277) L1113,1127; `child` (debug/render.rs:362) L1117,1138,1146; `Dom::nodes` (xmllinq/mod.rs:604) L1122; `Dom::text_value` (xmllinq/mod.rs:547) L1125; `attr` (debug.rs:523) L1135,1139,1141,1147
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.filter_map`, `.find`, `.into_iter`, `.map_or_else`, `.map`, `.next`, `.starts_with`, `.to_string`, `.trim`, `.unwrap_or_default`, `pos`
- **Macros:** `format!`, `matches!`
- **Called by:** `part_lines` (debug/render.rs:516); `tests::render_marks_a_true_behind_doc_anchor` (debug.rs:3511) (?)

#### `layout_lines`  (L1159-1241)
`fn layout_lines(pkg: &Package, rels: &[Rel]) -> (Vec<String>, usize)`

- **Does:** `(layout)`: jubarte's page count and font resolution (Word revision style, as the bench renders it), then the fonts the package asks for: `docDefaults`, the theme and `fontTable.xml`.
- **Calls (crate):** `docx_render_report` (convert/mod.rs:735) L1165; `parse` (debug/render.rs:104) L1199,1207,1221; `StyleBook::new` (debug/render.rs:156) L1200; `descendant` (debug/render.rs:368) L1210; `child` (debug/render.rs:362) L1211,1225; `attr` (debug.rs:523) L1212,1224,1226; `kids` (debug/render.rs:355) L1223; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L1228; `local` (debug.rs:277) L1231
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.extend`, `.filter_map`, `.find`, `.into_iter`, `.is_empty`, `.iter`, `.join`, `.map`, `.push_str`, `.push`, `.sort`, `.strip_prefix`, `Default::default`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `render_parts` (debug/render.rs:28)

## Test functions (22)

- `story_diagnostic_contract_tests::dom` (L1253): _no doc comment_
- `story_diagnostic_contract_tests::document` (L1260): _no doc comment_
- `story_diagnostic_contract_tests::package` (L1267): The dependency's own in-memory ZIP writer, also used by debug.rs tests. _(from a code comment)_
- `story_diagnostic_contract_tests::report` (L1287): _no doc comment_
- `story_diagnostic_contract_tests::nested_complex_fields_record_each_code_once_and_ignore_unowned_instructions` (L1297): _no doc comment_
- `story_diagnostic_contract_tests::shading_reports_literal_solid_theme_and_pattern_ink_with_explicit_resets` (L1320): _no doc comment_
- `story_diagnostic_contract_tests::inherited_frame_overrides_and_repeated_run_tallies_keep_first_source_sample` (L1364): _no doc comment_
- `story_diagnostic_contract_tests::a_stretch_split_into_runs_counts_once_and_a_gap_counts_twice` (L1388): _no doc comment_
- `story_diagnostic_contract_tests::strict_rprs_tally_by_local_name` (L1409): _no doc comment_
- `story_diagnostic_contract_tests::a_highlight_on_a_run_that_paints_no_glyph_is_not_tallied` (L1425): _no doc comment_
- `story_diagnostic_contract_tests::a_highlight_on_a_hyphen_or_position_tab_is_tallied` (L1442): _no doc comment_
- `story_diagnostic_contract_tests::a_table_reports_its_own_borders_and_its_cells` (L1473): _no doc comment_
- `story_diagnostic_contract_tests::nested_table_shading_and_textbox_frames_report_only_their_own_text` (L1486): _no doc comment_
- `story_diagnostic_contract_tests::section_columns_geometry_and_story_reference_diagnostics_are_literal` (L1510): _no doc comment_
- `story_diagnostic_contract_tests::section_geometry_prints_one_spelling_of_the_same_measure` (L1557): _no doc comment_
- `story_diagnostic_contract_tests::section_prints_a_plain_value_as_the_file_spells_it` (L1605): _no doc comment_
- `story_diagnostic_contract_tests::section_orientation_is_the_pages_not_the_attributes` (L1634): _no doc comment_
- `story_diagnostic_contract_tests::anchor_alignment_and_offsets_preserve_authored_extent_name_and_layer` (L1671): _no doc comment_
- `story_diagnostic_contract_tests::memory_package_filter_resolves_story_roles_and_excludes_nonstories_without_mutation` (L1694): _no doc comment_
- `story_diagnostic_contract_tests::malformed_package_relationship_diagnostics_do_not_invent_main_or_story_parts` (L1748): _no doc comment_
- `story_diagnostic_contract_tests::table_style_resets_and_look_flags_report_only_live_conditional_fills` (L1772): _no doc comment_
- `story_diagnostic_contract_tests::revisions_cross_transparent_owners_but_live_field_payload_breaks_adjacency` (L1804): _no doc comment_
