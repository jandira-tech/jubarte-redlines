# Functions and methods in `src/inspect.rs`

52 non-test, 6 test. Part of the [symbol manifest](../README.md).

#### `Snapshot::retain`  (L182-184, pub)
`pub fn retain(&mut self, body_paragraphs: &[usize])`

- **Does:** Keep the body paragraphs numbered in `body_paragraphs` (`N` of `pN`), and the changes and comments sitting in one of them; a comment whose range spans a kept paragraph counts.
- **Calls (crate):** `Snapshot::retain_with` (inspect.rs:189) L183
- **Called by:** none found in src

#### `Snapshot::retain_with`  (L189-210, pub(crate))
`pub(crate) fn retain_with(&mut self, body_paragraphs: &[usize], stories: &[String])`

- **Does:** [`Snapshot::retain`], keeping too the changes and comments in `stories` (`header1`, `footnotes`): those a `--changed` selection changed.
- **Calls (crate):** `body_index` (inspect.rs:214) L192,199,203,204; `paragraph_place` (ids.rs:417) L195
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.collect`, `.contains`, `.copied`, `.is_some_and`, `.is_some`, `.iter`, `.max`, `.next`, `.or`, `.range`, `.retain`, `Some`, `in_story`
- **Called by:** `Snapshot::retain` (inspect.rs:182); `read_snapshot` (markdown/mod.rs:821) (?)

#### `body_index`  (L214-218)
`fn body_index(id: Option<&String>) -> Option<usize>`

- **Does:** `N` of a body paragraph's id `pN`.
- **Calls (crate):** `paragraph_place` (ids.rs:417) L215
- **Calls (std/external/unresolved):** `.filter`, `.map`
- **Called by:** `Snapshot::retain_with` (inspect.rs:189)

#### `<InspectError as fmt::Display>::fmt`  (L285-292, impl fmt::Display)
`fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.write_str`
- **Macros:** `write!`
- **Called by:** none found in src

#### `source_sha256`  (L299-303, pub)
`pub fn source_sha256(bytes: &[u8]) -> String`

- **Does:** SHA-256 of the exact input bytes, lowercase hex: the snapshot guard shared by inspection and edit plans.
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`, `Sha256::digest`
- **Macros:** `format!`
- **Called by:** `generate` (bin/jubarte/edit_template.rs:32); `check_hash` (bin/jubarte/edit_typed.rs:266); `prepare_and_publish` (bin/jubarte/edit_typed.rs:398); `publish_edit` (bin/jubarte.rs:1402); `recheck_edit_source` (bin/jubarte.rs:1730); `refused` (bin/jubarte.rs:2697); `dest_uri_for_reconciled_part` (comparer/parts.rs:176); `tests::media_name_is_the_sha256_of_the_bytes_and_ignores_earlier_calls` (comparer/parts.rs:737); `open` (edit/anchor_error.rs:229) (?); `basis_digest` (edit/for_edit.rs:456) (?); `make_row` (edit/for_edit.rs:1293) (?); `for_edit` (edit/for_edit.rs:1581) (?); +21 more

#### `paragraphs`  (L312-315, pub)
`pub fn paragraphs(docx: &[u8]) -> Result<Vec<Paragraph>, InspectError>`

- **Does:** Body paragraphs in body order.
- **Calls (crate):** `Opened::open` (inspect.rs:735) L313; `body_paragraphs` (inspect.rs:1193) L314
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `tests::refresh_fields_writes_the_report_and_says_when_it_cannot` (bin/jubarte.rs:5748); `tests::final_text` (edit/inline.rs:403); `check_serialized_stories` (edit/prepared.rs:763); `check_redline_stories` (edit/prepared.rs:844); `produce` (edit/prepared.rs:928); `result_v2_with_paragraphs` (edit/tracked.rs:231); `result_with_order` (edit/tracked.rs:252); `deeper_boundary_tests::revision_selection_empty_and_reject_only_preserve_unselected_change` (edit.rs:8914); `projected_paragraphs` (expect.rs:554); `apply_markdown` (markdown/patch.rs:121); `all_paragraphs` (validate.rs:2986)

#### `paragraphs_and_stories_within`  (L319-328, pub(crate))
`pub(crate) fn paragraphs_and_stories_within( docx: &[u8], limits: crate::admission::InputLimits, ) -> Result<(Vec<Paragraph>, Vec<Story>), InspectError>`

- **Does:** Paragraph locations for a comparison rendered under its active budget.
- **Calls (crate):** `Opened::open_within` (inspect.rs:739) L323; `body_paragraphs` (inspect.rs:1193) L325; `Opened::stories_within` (inspect.rs:879) L326
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `word_paragraphs` (markdown/unified.rs:449)

#### `summary`  (L339-342, pub)
`pub fn summary(docx: &[u8]) -> Result<Summary, InspectError>`

- **Does:** Package facts, following OPC relationships for related parts.
- **Calls (crate):** `Opened::open` (inspect.rs:735) L340; `Opened::summary` (inspect.rs:765) L341
- **Called by:** `apply_markdown` (markdown/patch.rs:121)

#### `stories`  (L353-355, pub)
`pub fn stories(docx: &[u8]) -> Result<Vec<Story>, InspectError>`

- **Does:** Header, footer and note stories in a stable order (headers, footers, footnotes, endnotes).
- **Calls (crate):** `.stories` (ambiguous: `Opened::stories` (inspect.rs:873); `Writer::stories` (markdown/write.rs:4442)) L354; `Opened::open` (inspect.rs:735) L354
- **Called by:** `check_serialized_stories` (edit/prepared.rs:763); `check_redline_stories` (edit/prepared.rs:844); `tests::redline_only_defects_name_the_story_and_paragraph` (edit/prepared.rs:1190); `projected_paragraphs` (expect.rs:554); `all_paragraphs` (validate.rs:2986)

#### `controls`  (L364-367, pub)
`pub fn controls(docx: &[u8]) -> Result<Vec<ContentControl>, InspectError>`

- **Does:** Content controls (`w:sdt`) in the body, in document order.
- **Calls (crate):** `Opened::open` (inspect.rs:735) L365; `collect_controls` (inspect.rs:1005) L366
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** none found in src

#### `markdown`  (L372-379, pub)
`pub fn markdown(docx: &[u8]) -> Result<String, InspectError>`

- **Does:** Paragraphs prefixed with their ids and direct formatting as Markdown marks: `[p12 Heading1] (a) **Confidentiality.** You will ...`.
- **Calls (crate):** `Opened::open` (inspect.rs:735) L373; `body_paragraphs` (inspect.rs:1193) L374; `Opened::stories` (inspect.rs:873) L375; `render_markdown` (inspect.rs:637) L378
- **Calls (std/external/unresolved):** `.extend`, `Ok`
- **Called by:** `view_is_output` (cli.rs:3582) (?)

#### `tables`  (L389-392, pub)
`pub fn tables(docx: &[u8]) -> Result<Vec<Table>, InspectError>`

- **Does:** Body tables as grids: each cell's text and paragraph ids, the header rows and the grid widths.
- **Calls (crate):** `Opened::open` (inspect.rs:735) L390; `body_tables` (inspect/tables.rs:79) L391
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `coverage_real_source_route_matrix_tests::exercise` (comparer/lcs.rs:18961) (?); `mark_word_table_mesh_context` (comparer/lcs_table.rs:381) (?)

#### `snapshot`  (L403-429, pub)
`pub fn snapshot(docx: &[u8]) -> Result<Snapshot, InspectError>`

- **Does:** The full snapshot: `schema_version`, `source_sha256`, `summary`, `paragraphs`, `stories`, `tables`, `controls`, `changes` and `comments`.
- **Calls (crate):** `Opened::open` (inspect.rs:735) L406; `Opened::summary` (inspect.rs:765) L407; `body_paragraphs` (inspect.rs:1193) L408; `Opened::stories` (inspect.rs:873) L409; `body_tables` (inspect/tables.rs:79) L410; `collect_controls` (inspect.rs:1005) L411; `logical_changes_in` (changes.rs:584) L412; `comments_in` (comments.rs:138) L413; `source_sha256` (inspect.rs:299) L419
- **Calls (std/external/unresolved):** `.map_err`, `InspectError::Admission`, `InspectError::Package`, `Ok`
- **Called by:** `settled_snapshot` (inspect.rs:442); `inspect_json` (inspect.rs:542); `read_snapshot` (markdown/mod.rs:821)

#### `settled_snapshot`  (L442-456, pub)
`pub fn settled_snapshot(docx: &[u8], settle: Settle) -> Result<Snapshot, InspectError>`

- **Does:** [`snapshot`] with every tracked change settled by `settle`, and `track_changes` saying how.
- **Calls (crate):** `snapshot` (inspect.rs:403) L443; `Opened::open` (inspect.rs:735) L448; `reject_text` (inspect.rs:485) L449; `body_paragraphs` (inspect.rs:1193) L450; `body_tables` (inspect/tables.rs:79) L451; `collect_controls` (inspect.rs:1005) L452; `Opened::stories_within` (inspect.rs:879) L453; `<InputLimits as Default>::default` (admission.rs:64) L453
- **Calls (std/external/unresolved):** `Ok`, `Some`
- **Called by:** `read_snapshot` (markdown/mod.rs:821)

#### `view_tables`  (L465-478, pub)
`pub fn view_tables( docx: &[u8], settle: Option<Settle>, ) -> Result<Vec<(usize, Table)>, InspectError>`

- **Does:** [`tables`], with every change rejected when `settle` says so, each beside the number of its table in `read`'s view: the `tN` of the table outside every cell that is it or holds it, as `read -p tN` picks it.
- **Calls (crate):** `Opened::open` (inspect.rs:735) L469; `reject_text` (inspect.rs:485) L471; `body_tables` (inspect/tables.rs:79) L474
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`, `Ok`, `Some`
- **Called by:** `read_tables` (markdown/mod.rs:912)

#### `reject_text`  (L485-534)
`fn reject_text(dom: &mut Dom, root: NodeId)`

- **Does:** Word's Reject All on the text under `root`, in place: inserted and moved-to runs go, deleted and moved-from runs come back, their `w:delText` read as `w:t`, and a formatting change gives back the properties it replaced.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L493,518,523,530; `Dom::parent` (xmllinq/mod.rs:597) L494; `Dom::elements` (xmllinq/mod.rs:640) L497,506; `Dom::nodes` (xmllinq/mod.rs:604) L501,524; `Dom::remove_nodes` (xmllinq/mod.rs:1110) L502,511,525; `Dom::name_is` (xmllinq/mod.rs:535) L509; `Dom::add` (xmllinq/mod.rs:1074) L513; `Dom::remove` (xmllinq/mod.rs:1118) L519; `Dom::replace_with` (xmllinq/mod.rs:1165) L526; `Dom::set_name` (xmllinq/mod.rs:539) L531
- **Calls (std/external/unresolved):** `.any`, `.chain`, `.collect`, `.filter`, `.first`, `.into_iter`, `.iter`, `.map`, `.unwrap_or_default`, `Some`, `W::name`
- **Called by:** `settled_snapshot` (inspect.rs:442); `view_tables` (inspect.rs:465); `Opened::stories_within` (inspect.rs:879)

#### `inspect_json`  (L542-544, pub)
`pub fn inspect_json(docx: &[u8]) -> Result<String, InspectError>`

- **Does:** [`snapshot`] as JSON.
- **Calls (crate):** `snapshot` (inspect.rs:403) L543
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`, `InspectError::Invalid`, `serde_json::to_string`
- **Called by:** none found in src

#### `tables_text`  (L554-571, pub)
`pub fn tables_text(tables: &[Table]) -> String`

- **Does:** Tables as `read --tables` prints them: a `table N: RxC header_rows=H widths=W` line per table, then a line per row of tab-separated `ids=text` cells (`-` for a cell with no paragraph; `\`, newlines and tabs escaped), or `no tables`.
- **Calls (crate):** `render_grid` (inspect.rs:573) L560
- **Calls (std/external/unresolved):** `.is_empty`, `.push_str`, `String::new`
- **Macros:** `format!`
- **Called by:** `read_tables` (markdown/mod.rs:912)

#### `render_grid`  (L573-634)
`fn render_grid( out: &mut String, label: &str, rows: &[Vec<TableCell>], header_rows: usize, widths_dxa: &[u32], nested: &[NestedTable], depth: usize, )`

- **Does:** _no doc comment_
- **Calls (crate):** `short_paragraph` (ids.rs:435) L601; `render_grid` (inspect.rs:573) L623
- **Calls (std/external/unresolved):** `.collect`, `.enumerate`, `.filter`, `.is_empty`, `.is_some_and`, `.iter`, `.join`, `.len`, `.map`, `.max`, `.repeat`, `.replace`, `.rsplit_once`, `.starts_with`, `.to_string`, `.unwrap_or`
- **Macros:** `format!`, `writeln!`
- **Called by:** `tables_text` (inspect.rs:554); `render_grid` (inspect.rs:573)

#### `render_markdown`  (L637-658, pub)
`pub fn render_markdown(paragraphs: &[Paragraph]) -> String`

- **Does:** Markdown rendering of already inspected paragraphs.
- **Calls (crate):** `markdown_body` (inspect.rs:660) L647
- **Calls (std/external/unresolved):** `.ends_with`, `.is_empty`, `.pop`, `.push_str`, `.push`, `String::new`
- **Called by:** `markdown` (inspect.rs:372); `tests::markdown_of_empty_and_styled_paragraphs` (inspect.rs:1571)

#### `markdown_body`  (L660-678)
`fn markdown_body(p: &Paragraph) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `emphasize` (inspect.rs:680) L671
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.extend`, `.iter`, `.len`, `.min`, `.push_str`, `.to_string`, `.trim_end`, `String::new`
- **Called by:** `render_markdown` (inspect.rs:637)

#### `emphasize`  (L680-699)
`fn emphasize(piece: &str, span: &Span) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.is_empty`, `.is_none`, `.is_some`, `.len`, `.push_str`, `.push`, `.rev`, `.to_string`, `.trim_end`, `.trim_start`, `.trim`, `String::new`
- **Macros:** `format!`
- **Called by:** `markdown_body` (inspect.rs:660); `tests::markdown_moves_edge_whitespace_outside_emphasis` (inspect.rs:1550)

#### `Opened::open`  (L735-737, pub(crate))
`pub(crate) fn open(bytes: &[u8]) -> Result<Self, InspectError>`

- **Does:** _no doc comment_
- **Calls (crate):** `Opened::open_within` (inspect.rs:739) L736; `<InputLimits as Default>::default` (admission.rs:64) L736
- **Called by:** `audit_report_with` (audit.rs:177); `list_comments` (comments.rs:132); `for_edit` (edit/for_edit.rs:1581); `Origin::build` (edit/origin.rs:260); `Paragraphs::open` (edit/origin.rs:585); `written_places` (edit/origin.rs:649); `tests::split_correspondence_is_refused_by_origin` (edit/origin.rs:911); `range_markers` (edit/prepared.rs:368); `typing_order` (edit/replacement_order.rs:82); `record_mid_changes` (edit/sections.rs:453); `decisions` (edit/source_refs.rs:25); `reparsed_final_views` (edit/v2_apply.rs:563); +20 more

#### `Opened::open_within`  (L739-763)
`fn open_within( bytes: &[u8], limits: crate::admission::InputLimits, ) -> Result<Self, InspectError>`

- **Does:** _no doc comment_
- **Calls (crate):** `bump_reopened_packages` (inspect.rs:714) L744; `admit` (admission.rs:439) L745; `strict_to_transitional_docx_within` (strict_translation.rs:769) L747; `PartFs::open_within` (opc/mod.rs:593) L749; `main_part` (inspect.rs:916) L751; `parse_part_within` (inspect.rs:930) L752; `Dom::element` (xmllinq/mod.rs:655) (by name) L753
- **Calls (std/external/unresolved):** `.map_err`, `.ok_or`, `.to_string`, `InspectError::Package`, `Ok`, `W::body`
- **Called by:** `paragraphs_and_stories_within` (inspect.rs:319); `Opened::open` (inspect.rs:735)

#### `Opened::summary`  (L765-856)
`fn summary(&self) -> Result<Summary, InspectError>`

- **Does:** _no doc comment_
- **Calls (crate):** `body_paragraphs` (inspect.rs:1193) L768; `top_level_table_nodes` (inspect/tables.rs:71) L769; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L770,771,776,784,819,828...; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L774,832,841; `has_ancestor` (inspect.rs:950) L780,781; `PartFs::parts` (opc/mod.rs:733) (by name) L787; `PartFs::content_type_for` (opc/mod.rs:837) (by name) L788; `parse_part` (inspect.rs:926) L800,814; `logical_changes_in` (changes.rs:584) L803; `Opened::related` (inspect.rs:902) L805,806,807,808,809,810...; `note_count` (inspect.rs:975) L824,826; `Dom::element` (xmllinq/mod.rs:655) (by name) L839
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.count`, `.filter`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.starts_with`, `Err`, `InspectError::Invalid`, `Ok`, `Some`, `Summary::default`, `W::fld_char`, `W::fld_simple`, `W::name`, `W::num_pr`, `W::p_pr_change`, `W::sect_pr`, `W::val`
- **Macros:** `format!`
- **Called by:** `record_plan` (bin/jubarte.rs:2625) (?); `record_json` (edit/record.rs:270) (?); `tests::a_recording_pins_the_source_and_counts_on_from_the_held_plan` (edit/record.rs:348) (?); `tests::a_recording_of_revisions_only_says_so` (edit/record.rs:375) (?); `summary` (inspect.rs:339); `snapshot` (inspect.rs:403)

#### `Opened::story_parts`  (L861-871, pub(crate))
`pub(crate) fn story_parts(&self) -> Vec<(String, &'static str, String)>`

- **Does:** Internal targets of the main part's relationships of one kind.
- **Calls (crate):** `Opened::related` (inspect.rs:902) L864; `part_story` (ids.rs:392) L867
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.copied`, `.into_iter`, `.iter`, `.len`, `.push`, `.sort_by_key`, `Vec::new`
- **Called by:** `Package::read` (audit.rs:354); `comments_in` (comments.rs:138); `range_markers` (edit/prepared.rs:368) (?); `typing_order` (edit/replacement_order.rs:82) (?); `reparsed_final_views` (edit/v2_apply.rs:563) (?); `load_stories` (edit.rs:1680); `next_annotation_id` (edit.rs:6893); `comment_holds_kept_text` (edit.rs:6932) (?); `native_key_inventory` (expect.rs:919) (?); `update_fields` (fields.rs:115); `Opened::stories_within` (inspect.rs:879)

#### `Opened::stories`  (L873-875)
`fn stories(&self) -> Result<Vec<Story>, InspectError>`

- **Does:** _no doc comment_
- **Calls (crate):** `Opened::stories_within` (inspect.rs:879) L874; `<InputLimits as Default>::default` (admission.rs:64) L874
- **Called by:** `stories` (inspect.rs:353) (?); `markdown` (inspect.rs:372); `snapshot` (inspect.rs:403)

#### `Opened::stories_within`  (L879-900)
`fn stories_within( &self, limits: crate::admission::InputLimits, reject: bool, ) -> Result<Vec<Story>, InspectError>`

- **Does:** [`Self::stories`] under `limits`, every change rejected when `reject` is set.
- **Calls (crate):** `Opened::story_parts` (inspect.rs:861) L884; `parse_part_within` (inspect.rs:930) L887; `reject_text` (inspect.rs:485) L889; `paragraphs_of` (inspect.rs:1197) L891; `story_paragraph_nodes` (inspect.rs:1178) L891
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`, `.to_string`, `Ok`
- **Called by:** `paragraphs_and_stories_within` (inspect.rs:319); `settled_snapshot` (inspect.rs:442); `Opened::stories` (inspect.rs:873)

#### `Opened::related`  (L902-913, pub(crate))
`pub(crate) fn related(&self, kind: &str) -> BTreeSet<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:991) L903; `PartFs::resolve_rel_target` (opc/mod.rs:829) L911
- **Calls (std/external/unresolved):** `.as_deref`, `.collect`, `.filter`, `.flat_map`, `.into_iter`, `.map`, `.next`, `.rsplit`, `Some`
- **Called by:** `Package::read` (audit.rs:354); `Transaction::footnotes_story` (edit/notes.rs:51); `Transaction::style_defined` (edit/notes.rs:83); `table_styles` (edit/structural.rs:89); `next_numbering_ids` (edit/structural.rs:350); `Transaction::paragraph_styles` (edit.rs:4903); `existing_comment_ids` (edit.rs:6960); `update_fields` (fields.rs:115); `Opened::summary` (inspect.rs:765); `Opened::story_parts` (inspect.rs:861)

#### `main_part`  (L916-923)
`fn main_part(pkg: &PartFs) -> Result<String, InspectError>`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:1070) L917; `PartFs::part_bytes` (opc/mod.rs:628) L919
- **Calls (std/external/unresolved):** `.map`, `.ok_or`, `.or_else`, `.to_string`
- **Called by:** `Opened::open_within` (inspect.rs:739)

#### `parse_part`  (L926-928, pub(crate))
`pub(crate) fn parse_part(pkg: &PartFs, name: &str) -> Result<(Dom, NodeId, NodeId), InspectError>`

- **Does:** Parse one XML part through the checked reader: `(dom, document, root)`.
- **Calls (crate):** `parse_part_within` (inspect.rs:930) L927; `<InputLimits as Default>::default` (admission.rs:64) L927
- **Called by:** `Package::read` (audit.rs:354) (?); `comments_in` (comments.rs:138); `range_markers` (edit/prepared.rs:368); `reparsed_final_views` (edit/v2_apply.rs:563); `next_annotation_id` (edit.rs:6893); `comment_holds_kept_text` (edit.rs:6932); `native_key_inventory` (expect.rs:919); `update_fields` (fields.rs:115); `Opened::summary` (inspect.rs:765)

#### `parse_part_within`  (L930-948)
`fn parse_part_within( pkg: &PartFs, name: &str, limits: crate::admission::InputLimits, ) -> Result<(Dom, NodeId, NodeId), InspectError>`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::part_bytes` (opc/mod.rs:628) L935; `validate_xml_with_limits` (xmllinq/parse.rs:1061) L940; `Dom::new` (xmllinq/mod.rs:387) L942; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L943; `Dom::root` (xmllinq/mod.rs:822) L944
- **Calls (std/external/unresolved):** `.map_err`, `.ok_or_else`, `InspectError::Invalid`, `Ok`, `std::str::from_utf8`
- **Macros:** `format!`
- **Called by:** `Opened::open_within` (inspect.rs:739); `Opened::stories_within` (inspect.rs:879); `parse_part` (inspect.rs:926)

#### `has_ancestor`  (L950-952)
`fn has_ancestor(dom: &Dom, node: NodeId, name: &XName) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L951
- **Calls (std/external/unresolved):** `.is_empty`, `Some`
- **Called by:** `Opened::summary` (inspect.rs:765); `in_text_box` (inspect.rs:962); `paragraphs_of` (inspect.rs:1197)

#### `text_box_names`  (L957-959, pub(crate))
`pub(crate) fn text_box_names() -> [XName; 2]`

- **Does:** The two names a text box's content goes by: `w:txbxContent`, and the `wne:txbxContent` a Strict package's shapes hold (the agent view skips both by local name, so every id walk must too).
- **Calls (std/external/unresolved):** `W::txbx_content`, `WNE::name`
- **Called by:** `in_text_box` (inspect.rs:962); `holds_text_box` (inspect.rs:969)

#### `in_text_box`  (L962-966, pub(crate))
`pub(crate) fn in_text_box(dom: &Dom, node: NodeId) -> bool`

- **Does:** Whether `node` sits inside a text box.
- **Calls (crate):** `text_box_names` (inspect.rs:957) L963; `has_ancestor` (inspect.rs:950) L965
- **Calls (std/external/unresolved):** `.any`, `.iter`
- **Called by:** `Ids::table_paragraph` (edit/origin.rs:98); `Paragraphs::control_count` (edit/origin.rs:612); `Transaction::emits_directly` (edit.rs:2828); `opaque_table` (inspect/tables.rs:66); `body_paragraph_nodes` (inspect.rs:988); `body_control_nodes` (inspect.rs:997)

#### `holds_text_box`  (L969-973)
`fn holds_text_box(dom: &Dom, node: NodeId) -> bool`

- **Does:** Whether `node` holds a text box.
- **Calls (crate):** `text_box_names` (inspect.rs:957) L970; `Dom::descendants` (xmllinq/mod.rs:668) L972
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.iter`, `Some`
- **Called by:** `walk_run` (inspect.rs:1365)

#### `note_count`  (L975-985)
`fn note_count(dom: &Dom, root: NodeId, local: &str) -> usize`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L976; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L980
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.iter`, `Some`, `W::name`
- **Macros:** `matches!`
- **Called by:** `Opened::summary` (inspect.rs:765)

#### `body_paragraph_nodes`  (L988-993, pub(crate))
`pub(crate) fn body_paragraph_nodes(dom: &Dom, body: NodeId) -> Vec<NodeId>`

- **Does:** The body paragraphs (`w:p` outside text boxes) in document order.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L989; `in_text_box` (inspect.rs:962) L991
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `Some`, `W::p`
- **Called by:** `Package::read` (audit.rs:354); `Anchors::scan` (comments.rs:251); `check_serialized_stories` (edit/prepared.rs:763) (?); `reparsed_final_views` (edit/v2_apply.rs:563); `load_stories` (edit.rs:1680); `Transaction::paragraph_walk` (edit.rs:5097); `native_key_inventory` (expect.rs:919); `paragraph_ids` (fields.rs:921); `headings` (fields.rs:1023); `body_tables` (inspect/tables.rs:79); `collect_controls` (inspect.rs:1005); `story_paragraph_nodes` (inspect.rs:1178); +2 more

#### `body_control_nodes`  (L997-1002, pub(crate))
`pub(crate) fn body_control_nodes(dom: &Dom, body: NodeId) -> Vec<NodeId>`

- **Does:** The body's content controls (`w:sdt` outside text boxes) in document order, the order [`collect_controls`] numbers them in.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L998; `in_text_box` (inspect.rs:962) L1000
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `Some`, `W::sdt`
- **Called by:** `Origin::build` (edit/origin.rs:260); `SourceModel::container_node` (edit/v2_apply.rs:938); `Transaction::start` (edit.rs:2680); `collect_controls` (inspect.rs:1005)

#### `collect_controls`  (L1005-1037, pub(crate))
`pub(crate) fn collect_controls(dom: &Dom, body: NodeId) -> Vec<ContentControl>`

- **Does:** One record per [`body_control_nodes`] entry.
- **Calls (crate):** `body_paragraph_nodes` (inspect.rs:988) L1006; `body_control_nodes` (inspect.rs:997) L1016; `Dom::ancestors` (xmllinq/mod.rs:770) L1019; `paragraph_id` (ids.rs:410) L1027; `control_record` (inspect.rs:1039) L1034
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.copied`, `.entry`, `.enumerate`, `.first`, `.get`, `.into_iter`, `.iter`, `.map`, `.or_default`, `Some`, `W::p`, `std::collections::HashMap::new`
- **Macros:** `format!`
- **Called by:** `Origin::build` (edit/origin.rs:260); `SourceModel::container_node` (edit/v2_apply.rs:938); `Transaction::start` (edit.rs:2680); `controls` (inspect.rs:364); `snapshot` (inspect.rs:403); `settled_snapshot` (inspect.rs:442)

#### `control_record`  (L1039-1114)
`fn control_record( dom: &Dom, sdt: NodeId, id: String, index: &std::collections::HashMap<NodeId, usize>, ) -> ContentControl`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1045,1046,1067,1077; `child` (ambiguous, 4 candidates) L1048,1053,1066,1071; `Dom::attribute` (xmllinq/mod.rs:874) L1049,1059,1060,1068,1072; `control_kind` (inspect.rs:1137) L1052; `Dom::elements` (xmllinq/mod.rs:640) L1056; `Dom::ancestors` (xmllinq/mod.rs:770) L1078; `project_container` (inspect.rs:1283) L1081; `Dom::descendants` (xmllinq/mod.rs:668) L1085; `project_paragraph` (inspect.rs:1265) L1092; `paragraph_id` (ids.rs:410) L1107
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.collect`, `.copied`, `.filter_map`, `.first`, `.get`, `.into_iter`, `.is_none_or`, `.is_some_and`, `.iter`, `.join`, `.map_or_else`, `.map_or`, `.map`, `.or_else`, `.to_string`, `.unwrap_or_default`, `Some`, `String::new`, `Vec::new`, `W14::name`, `W::name`, `W::p`, `W::sdt_content`, `W::sdt_pr`, `W::val`, `val`
- **Macros:** `matches!`, `vec!`
- **Called by:** `collect_controls` (inspect.rs:1005)

#### `on_off`  (L1117-1119)
`fn on_off(value: &str) -> bool`

- **Does:** `1`, `true` and `on` are set; anything else is clear.
- **Macros:** `matches!`
- **Called by:** none found in src

#### `control_kind`  (L1137-1175)
`fn control_kind(dom: &Dom, pr: NodeId) -> &'static str`

- **Does:** The control's type from its `w:sdtPr` (see [`ContentControl::kind`]).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1138; `Dom::name` (xmllinq/mod.rs:526) L1139; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1142; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1142
- **Calls (std/external/unresolved):** `.contains`
- **Called by:** `control_record` (inspect.rs:1039)

#### `story_paragraph_nodes`  (L1178-1191, pub(crate))
`pub(crate) fn story_paragraph_nodes(dom: &Dom, root: NodeId) -> Vec<NodeId>`

- **Does:** A header, footer or notes part's paragraphs, separator notes left out.
- **Calls (crate):** `body_paragraph_nodes` (inspect.rs:988) L1179; `Dom::ancestors` (xmllinq/mod.rs:770) L1182; `Dom::name_is` (xmllinq/mod.rs:535) L1183; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L1185
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.filter`, `.into_iter`, `Some`, `W::name`
- **Macros:** `matches!`
- **Called by:** `Package::read` (audit.rs:354); `audit_stories` (changes.rs:168); `paragraph_index` (changes.rs:1377); `Anchors::scan` (comments.rs:251); `check_serialized_stories` (edit/prepared.rs:763); `reparsed_final_views` (edit/v2_apply.rs:563); `load_stories` (edit.rs:1680); `Transaction::paragraph_walk` (edit.rs:5097); `native_key_inventory` (expect.rs:919); `paragraph_ids` (fields.rs:921); `Opened::stories_within` (inspect.rs:879); `tests::story_paragraphs_agree_with_the_paragraphs_edit_resolves` (markdown/from_docx/header.rs:2097); +1 more

#### `body_paragraphs`  (L1193-1195)
`fn body_paragraphs(dom: &Dom, body: NodeId) -> Vec<Paragraph>`

- **Does:** _no doc comment_
- **Calls (crate):** `paragraphs_of` (inspect.rs:1197) L1194; `body_paragraph_nodes` (inspect.rs:988) L1194
- **Called by:** `paragraphs` (inspect.rs:312); `paragraphs_and_stories_within` (inspect.rs:319); `markdown` (inspect.rs:372); `snapshot` (inspect.rs:403); `settled_snapshot` (inspect.rs:442); `Opened::summary` (inspect.rs:765)

#### `paragraphs_of`  (L1197-1222)
`fn paragraphs_of(dom: &Dom, nodes: Vec<NodeId>, story: &str) -> Vec<Paragraph>`

- **Does:** _no doc comment_
- **Calls (crate):** `project_paragraph` (inspect.rs:1265) L1202; `Dom::element` (xmllinq/mod.rs:655) L1203,1205,1208; `Dom::attribute` (xmllinq/mod.rs:874) L1206; `paragraph_id` (ids.rs:410) L1211; `has_ancestor` (inspect.rs:950) L1215
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.enumerate`, `.into_iter`, `.is_some_and`, `.is_some`, `.map`, `W::num_pr`, `W::p_pr`, `W::p_style`, `W::tc`, `W::val`
- **Called by:** `Opened::stories_within` (inspect.rs:879); `body_paragraphs` (inspect.rs:1193)

#### `project_paragraph`  (L1265-1279, pub(crate))
`pub(crate) fn project_paragraph(dom: &Dom, paragraph: NodeId) -> Projection`

- **Does:** Project one `w:p` (see module docs for the rules).
- **Calls (crate):** `walk_container` (inspect.rs:1302) L1269; `merge_spans` (inspect.rs:1510) L1277
- **Calls (std/external/unresolved):** `Projection::default`, `Vec::new`
- **Called by:** `Package::literal_bullet` (audit.rs:636); `Package::empty_spacers` (audit.rs:661); `Anchors::scan` (comments.rs:251); `text_selection` (edit/for_edit.rs:663) (?); `obstacles` (edit/for_edit.rs:1041) (?); `make_row` (edit/for_edit.rs:1293) (?); `context_of` (edit/for_edit.rs:1505) (?); `insert_reference` (edit/notes.rs:168); `Origin::build` (edit/origin.rs:260); `check_dom_realized` (edit/prepared.rs:712) (?); `control_text` (edit/prepared.rs:753) (?); `check_serialized_stories` (edit/prepared.rs:763) (?); +26 more

#### `project_container`  (L1283-1297, pub(crate))
`pub(crate) fn project_container(dom: &Dom, container: NodeId) -> Projection`

- **Does:** Project the inline content of any container (a run-level `w:sdtContent`) with the paragraph rules.
- **Calls (crate):** `walk_container` (inspect.rs:1302) L1287; `merge_spans` (inspect.rs:1510) L1295
- **Calls (std/external/unresolved):** `Projection::default`, `Vec::new`
- **Called by:** `control_record` (inspect.rs:1039)

#### `walk_container`  (L1302-1363)
`fn walk_container( dom: &Dom, container: NodeId, direct: bool, fields: &mut usize, out: &mut Projection, formats: &mut Vec<(usize, usize, Format)>, )`

- **Does:** `fields` counts the complex fields open at this point of the paragraph: their `begin`/`end` runs are siblings of the result runs, so a result run is direct by position yet belongs to the field.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1310; `Dom::name` (xmllinq/mod.rs:526) L1311; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1314; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1315; `Dom::element` (xmllinq/mod.rs:655) L1319,1351; `walk_container` (inspect.rs:1302) L1321,1338,1343,1347,1352,1356; `walk_run` (inspect.rs:1365) L1340
- **Calls (std/external/unresolved):** `.insert`, `.into`, `MC::name`, `W::sdt_content`
- **Macros:** `format!`
- **Called by:** `project_paragraph` (inspect.rs:1265); `project_container` (inspect.rs:1283); `walk_container` (inspect.rs:1302)

#### `walk_run`  (L1365-1484)
`fn walk_run( dom: &Dom, run: NodeId, direct: bool, fields: &mut usize, out: &mut Projection, formats: &mut Vec<(usize, usize, Format)>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `run_format` (inspect.rs:1486) L1373; `Dom::elements` (xmllinq/mod.rs:640) L1375; `Dom::name` (xmllinq/mod.rs:526) L1376; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1381,1394; `Dom::descendants` (xmllinq/mod.rs:668) L1385; `holds_text_box` (inspect.rs:969) L1389,1451; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1396,1413; `Dom::value_str` (xmllinq/mod.rs:1195) L1416; `tests::glyph` (convert/math_layout.rs:799) (by name) L1425,1426,1432,1434,1435,1438; `Dom::attribute` (xmllinq/mod.rs:874) L1427,1443
- **Calls (std/external/unresolved):** `.any`, `.insert`, `.into`, `.is_empty`, `.iter`, `.len`, `.push_str`, `.push`, `.saturating_sub`, `Some`, `W::name`
- **Macros:** `format!`
- **Called by:** `walk_container` (inspect.rs:1302)

#### `run_format`  (L1486-1508)
`fn run_format(dom: &Dom, run: NodeId) -> Format`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1487,1491,1494,1497; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L1492; `Dom::attribute` (xmllinq/mod.rs:874) L1496,1499; `on` (ambiguous: `on` (audit.rs:299); `on` (debug/render.rs:445)) L1503,1504
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.is_none_or`, `.is_some_and`, `.map`, `Format::default`, `Some`, `W::name`, `W::r_pr`, `W::val`
- **Macros:** `matches!`
- **Called by:** `walk_run` (inspect.rs:1365)

#### `merge_spans`  (L1510-1534)
`fn merge_spans(text: &str, formats: Vec<(usize, usize, Format)>) -> Vec<Span>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.count`, `.into_iter`, `.last_mut`, `.map`, `.push`, `Vec::new`, `char_at`
- **Called by:** `project_paragraph` (inspect.rs:1265); `project_container` (inspect.rs:1283); `tests::spans_merge_only_adjacent_equal_formatting` (inspect.rs:1589)

## Test functions (6)

- `bump_reopened_packages` (L714): Add one to this thread's [`REOPENED_PACKAGES`].
- `reset_reopened_packages` (L721): Zero this thread's [`REOPENED_PACKAGES`].
- `tests::sha256_is_lowercase_hex_of_the_bytes` (L1542): _no doc comment_
- `tests::markdown_moves_edge_whitespace_outside_emphasis` (L1550): _no doc comment_
- `tests::markdown_of_empty_and_styled_paragraphs` (L1571): _no doc comment_
- `tests::spans_merge_only_adjacent_equal_formatting` (L1589): _no doc comment_
