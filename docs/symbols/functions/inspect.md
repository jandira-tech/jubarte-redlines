# Functions and methods in `src/inspect.rs`

44 non-test, 4 test. Part of the [symbol manifest](../README.md).

#### `Snapshot::retain`  (L154-156, pub)
`pub fn retain(&mut self, body_paragraphs: &[usize])`

- **Does:** Keep the body paragraphs numbered in `body_paragraphs` (`N` of `pN`), and the changes and comments sitting in one of them; a comment whose range spans a kept paragraph counts.
- **Calls (crate):** `Snapshot::retain_with` (inspect.rs:161) L155
- **Called by:** none found in src

#### `Snapshot::retain_with`  (L161-182, pub)
`pub fn retain_with(&mut self, body_paragraphs: &[usize], stories: &[String])`

- **Does:** [`Snapshot::retain`], keeping too the changes and comments in `stories` (`header1`, `footnotes`): those a `--changed` selection changed.
- **Calls (crate):** `body_index` (inspect.rs:186) L164,171,175,176; `paragraph_place` (ids.rs:417) L167
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.collect`, `.contains`, `.copied`, `.is_some_and`, `.is_some`, `.iter`, `.max`, `.next`, `.or`, `.range`, `.retain`, `Some`, `in_story`
- **Called by:** `Snapshot::retain` (inspect.rs:154); `read_snapshot` (markdown/mod.rs:798) (?)

#### `body_index`  (L186-190)
`fn body_index(id: Option<&String>) -> Option<usize>`

- **Does:** `N` of a body paragraph's id `pN`.
- **Calls (crate):** `paragraph_place` (ids.rs:417) L187
- **Calls (std/external/unresolved):** `.filter`, `.map`
- **Called by:** `Snapshot::retain_with` (inspect.rs:161)

#### `<InspectError as fmt::Display>::fmt`  (L257-264, impl fmt::Display)
`fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.write_str`
- **Macros:** `write!`
- **Called by:** none found in src

#### `source_sha256`  (L271-275, pub)
`pub fn source_sha256(bytes: &[u8]) -> String`

- **Does:** SHA-256 of the exact input bytes, lowercase hex: the snapshot guard shared by inspection and edit plans.
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`, `Sha256::digest`
- **Macros:** `format!`
- **Called by:** `run_edit` (bin/jubarte.rs:953); `recheck_edit_source` (bin/jubarte.rs:1232); `dest_uri_for_reconciled_part` (comparer/parts.rs:176); `tests::media_name_is_the_sha256_of_the_bytes_and_ignores_earlier_calls` (comparer/parts.rs:737); `record` (edit/record.rs:181); `tests::a_recording_pins_the_source_and_counts_on_from_the_held_plan` (edit/record.rs:334); `before_sha256` (edit/v2.rs:797); `Transaction::start` (edit.rs:2004); `snapshot` (inspect.rs:362); `tests::sha256_is_lowercase_hex_of_the_bytes` (inspect.rs:1301); `convert` (markdown/from_docx/mod.rs:96); `Artifact::new` (result.rs:186); +2 more

#### `paragraphs`  (L284-287, pub)
`pub fn paragraphs(docx: &[u8]) -> Result<Vec<Paragraph>, InspectError>`

- **Does:** Body paragraphs in body order.
- **Calls (crate):** `Opened::open` (inspect.rs:525) L285; `body_paragraphs` (inspect.rs:952) L286
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `tests::refresh_fields_writes_the_report_and_says_when_it_cannot` (bin/jubarte.rs:4503); `result` (edit/tracked.rs:167); `apply_plan` (edit.rs:1621); `deeper_boundary_tests::revision_selection_empty_and_reject_only_preserve_unselected_change` (edit.rs:6789); `apply_markdown` (markdown/patch.rs:121); `word_paragraphs` (markdown/unified.rs:441); `all_paragraphs` (validate.rs:2005)

#### `summary`  (L298-301, pub)
`pub fn summary(docx: &[u8]) -> Result<Summary, InspectError>`

- **Does:** Package facts, following OPC relationships for related parts.
- **Calls (crate):** `Opened::open` (inspect.rs:525) L299; `Opened::summary` (inspect.rs:545) L300
- **Called by:** `apply_markdown` (markdown/patch.rs:121)

#### `stories`  (L312-314, pub)
`pub fn stories(docx: &[u8]) -> Result<Vec<Story>, InspectError>`

- **Does:** Header, footer and note stories in a stable order (headers, footers, footnotes, endnotes).
- **Calls (crate):** `.stories` (ambiguous: `Opened::stories` (inspect.rs:653); `Writer::stories` (markdown/write.rs:4064)) L313; `Opened::open` (inspect.rs:525) L313
- **Called by:** `word_paragraphs` (markdown/unified.rs:441); `all_paragraphs` (validate.rs:2005)

#### `controls`  (L323-326, pub)
`pub fn controls(docx: &[u8]) -> Result<Vec<ContentControl>, InspectError>`

- **Does:** Content controls (`w:sdt`) in the body, in document order.
- **Calls (crate):** `Opened::open` (inspect.rs:525) L324; `collect_controls` (inspect.rs:764) L325
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** none found in src

#### `markdown`  (L331-338, pub)
`pub fn markdown(docx: &[u8]) -> Result<String, InspectError>`

- **Does:** Paragraphs prefixed with their ids and direct formatting as Markdown marks: `[p12 Heading1] (a) **Confidentiality.** You will ...`.
- **Calls (crate):** `Opened::open` (inspect.rs:525) L332; `body_paragraphs` (inspect.rs:952) L333; `Opened::stories` (inspect.rs:653) L334; `render_markdown` (inspect.rs:451) L337
- **Calls (std/external/unresolved):** `.extend`, `Ok`
- **Called by:** `view_is_output` (cli.rs:3004) (?)

#### `tables`  (L348-351, pub)
`pub fn tables(docx: &[u8]) -> Result<Vec<Table>, InspectError>`

- **Does:** Body tables as grids: each cell's text and paragraph ids, the header rows and the grid widths.
- **Calls (crate):** `Opened::open` (inspect.rs:525) L349; `body_tables` (inspect/tables.rs:48) L350
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `run_read_tables` (bin/jubarte.rs:776); `coverage_real_source_route_matrix_tests::exercise` (comparer/lcs.rs:18961) (?); `mark_word_table_mesh_context` (comparer/lcs_table.rs:381) (?)

#### `snapshot`  (L362-387, pub)
`pub fn snapshot(docx: &[u8]) -> Result<Snapshot, InspectError>`

- **Does:** The full snapshot: `schema_version`, `source_sha256`, `summary`, `paragraphs`, `stories`, `tables`, `controls`, `changes` and `comments`.
- **Calls (crate):** `Opened::open` (inspect.rs:525) L365; `Opened::summary` (inspect.rs:545) L366; `body_paragraphs` (inspect.rs:952) L367; `Opened::stories` (inspect.rs:653) L368; `body_tables` (inspect/tables.rs:48) L369; `collect_controls` (inspect.rs:764) L370; `changes_in` (changes.rs:302) L371; `comments_in` (comments.rs:136) L372; `source_sha256` (inspect.rs:271) L378
- **Calls (std/external/unresolved):** `.map_err`, `InspectError::Admission`, `InspectError::Package`, `Ok`
- **Called by:** `inspect_json` (inspect.rs:395); `read_snapshot` (markdown/mod.rs:798)

#### `inspect_json`  (L395-397, pub)
`pub fn inspect_json(docx: &[u8]) -> Result<String, InspectError>`

- **Does:** [`snapshot`] as JSON.
- **Calls (crate):** `snapshot` (inspect.rs:362) L396
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`, `InspectError::Invalid`, `serde_json::to_string`
- **Called by:** none found in src

#### `tables_text`  (L406-448, pub)
`pub fn tables_text(tables: &[Table]) -> String`

- **Does:** Tables as `read --tables` prints them: a `table N: RxC header_rows=H widths=W` line per table, then a line per row of tab-separated `ids=text` cells (`-` for a cell with no paragraph; `\`, newlines and tabs escaped), or `no tables`.
- **Calls (crate):** `short_paragraph` (ids.rs:435) L432
- **Calls (std/external/unresolved):** `.collect`, `.is_empty`, `.iter`, `.join`, `.len`, `.map`, `.max`, `.push_str`, `.replace`, `.to_string`, `.unwrap_or`, `String::new`
- **Macros:** `format!`, `writeln!`
- **Called by:** `run_read_tables` (bin/jubarte.rs:776)

#### `render_markdown`  (L451-472, pub)
`pub fn render_markdown(paragraphs: &[Paragraph]) -> String`

- **Does:** Markdown rendering of already inspected paragraphs.
- **Calls (crate):** `markdown_body` (inspect.rs:474) L461
- **Calls (std/external/unresolved):** `.ends_with`, `.is_empty`, `.pop`, `.push_str`, `.push`, `String::new`
- **Called by:** `markdown` (inspect.rs:331); `tests::markdown_of_empty_and_styled_paragraphs` (inspect.rs:1330)

#### `markdown_body`  (L474-492)
`fn markdown_body(p: &Paragraph) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `emphasize` (inspect.rs:494) L485
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.extend`, `.iter`, `.len`, `.min`, `.push_str`, `.to_string`, `.trim_end`, `String::new`
- **Called by:** `render_markdown` (inspect.rs:451)

#### `emphasize`  (L494-513)
`fn emphasize(piece: &str, span: &Span) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.is_empty`, `.is_none`, `.is_some`, `.len`, `.push_str`, `.push`, `.rev`, `.to_string`, `.trim_end`, `.trim_start`, `.trim`, `String::new`
- **Macros:** `format!`
- **Called by:** `markdown_body` (inspect.rs:474); `tests::markdown_moves_edge_whitespace_outside_emphasis` (inspect.rs:1309)

#### `Opened::open`  (L525-543, pub(crate))
`pub(crate) fn open(bytes: &[u8]) -> Result<Self, InspectError>`

- **Does:** _no doc comment_
- **Calls (crate):** `admit` (admission.rs:310) L526; `<InputLimits as Default>::default` (admission.rs:56) L526; `strict_to_transitional_docx` (strict_translation.rs:743) L528; `PartFs::open` (opc/mod.rs:456) L530; `main_part` (inspect.rs:683) L531; `parse_part` (inspect.rs:693) L532; `Dom::element` (xmllinq/mod.rs:655) (by name) L533
- **Calls (std/external/unresolved):** `.map_err`, `.ok_or`, `.to_string`, `InspectError::Package`, `Ok`, `W::body`
- **Called by:** `audit_report_with` (audit.rs:177); `list_comments` (comments.rs:130); `Origin::build` (edit/origin.rs:256); `record_mid_changes` (edit/sections.rs:453); `rewrite` (edit/whole.rs:130); `settle` (edit.rs:1260); `paragraph_ids` (edit.rs:1365); `Transaction::start` (edit.rs:2004); `comment_holds_kept_text` (edit.rs:5236); `update_fields` (fields.rs:115); `paragraphs` (inspect.rs:284); `summary` (inspect.rs:298); +5 more

#### `Opened::summary`  (L545-636)
`fn summary(&self) -> Result<Summary, InspectError>`

- **Does:** _no doc comment_
- **Calls (crate):** `body_paragraphs` (inspect.rs:952) L548; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L549,550,551,556,564,599...; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L554,612,621; `has_ancestor` (inspect.rs:709) L560,561; `PartFs::parts` (opc/mod.rs:532) (by name) L567; `PartFs::content_type_for` (opc/mod.rs:608) (by name) L568; `parse_part` (inspect.rs:693) L580,594; `changes_in` (changes.rs:302) L583; `Opened::related` (inspect.rs:669) L585,586,587,588,589,590...; `note_count` (inspect.rs:734) L604,606; `Dom::element` (xmllinq/mod.rs:655) (by name) L619
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.count`, `.filter`, `.into_iter`, `.is_empty`, `.is_some_and`, `.iter`, `.len`, `.starts_with`, `Err`, `InspectError::Invalid`, `Ok`, `Some`, `Summary::default`, `W::fld_char`, `W::fld_simple`, `W::name`, `W::num_pr`, `W::p_pr_change`, `W::sect_pr`, `W::tbl`, `W::val`
- **Macros:** `format!`
- **Called by:** `record_plan` (bin/jubarte.rs:1887) (?); `record_json` (edit/record.rs:257) (?); `tests::a_recording_pins_the_source_and_counts_on_from_the_held_plan` (edit/record.rs:334) (?); `tests::a_recording_of_revisions_only_says_so` (edit/record.rs:361) (?); `summary` (inspect.rs:298); `snapshot` (inspect.rs:362)

#### `Opened::story_parts`  (L641-651, pub(crate))
`pub(crate) fn story_parts(&self) -> Vec<(String, &'static str, String)>`

- **Does:** Internal targets of the main part's relationships of one kind.
- **Calls (crate):** `Opened::related` (inspect.rs:669) L644; `part_story` (ids.rs:392) L647
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.copied`, `.into_iter`, `.iter`, `.len`, `.push`, `.sort_by_key`, `Vec::new`
- **Called by:** `Package::read` (audit.rs:354); `comments_in` (comments.rs:136); `load_stories` (edit.rs:1311); `comment_holds_kept_text` (edit.rs:5236) (?); `update_fields` (fields.rs:115); `Opened::stories` (inspect.rs:653)

#### `Opened::stories`  (L653-667)
`fn stories(&self) -> Result<Vec<Story>, InspectError>`

- **Does:** _no doc comment_
- **Calls (crate):** `Opened::story_parts` (inspect.rs:641) L654; `parse_part` (inspect.rs:693) L657; `paragraphs_of` (inspect.rs:956) L658; `story_paragraph_nodes` (inspect.rs:937) L658
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`, `.to_string`, `Ok`
- **Called by:** `stories` (inspect.rs:312) (?); `markdown` (inspect.rs:331); `snapshot` (inspect.rs:362)

#### `Opened::related`  (L669-680, pub(crate))
`pub(crate) fn related(&self, kind: &str) -> BTreeSet<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:756) L670; `PartFs::resolve_rel_target` (opc/mod.rs:600) L678
- **Calls (std/external/unresolved):** `.as_deref`, `.collect`, `.filter`, `.flat_map`, `.into_iter`, `.map`, `.next`, `.rsplit`, `Some`
- **Called by:** `Package::read` (audit.rs:354); `Transaction::footnotes_story` (edit/notes.rs:51); `Transaction::style_defined` (edit/notes.rs:83); `table_styles` (edit/structural.rs:89); `next_numbering_ids` (edit/structural.rs:350); `Transaction::paragraph_styles` (edit.rs:3735); `existing_comment_ids` (edit.rs:5264); `update_fields` (fields.rs:115); `Opened::summary` (inspect.rs:545); `Opened::story_parts` (inspect.rs:641)

#### `main_part`  (L683-690)
`fn main_part(pkg: &PartFs) -> Result<String, InspectError>`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:829) L684; `PartFs::part_bytes` (opc/mod.rs:493) L686
- **Calls (std/external/unresolved):** `.map`, `.ok_or`, `.or_else`, `.to_string`
- **Called by:** `Opened::open` (inspect.rs:525)

#### `parse_part`  (L693-707, pub(crate))
`pub(crate) fn parse_part(pkg: &PartFs, name: &str) -> Result<(Dom, NodeId, NodeId), InspectError>`

- **Does:** Parse one XML part through the checked reader: `(dom, document, root)`.
- **Calls (crate):** `PartFs::part_bytes` (opc/mod.rs:493) L694; `validate_xml` (xmllinq/parse.rs:792) L699; `Dom::new` (xmllinq/mod.rs:387) L701; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L702; `Dom::root` (xmllinq/mod.rs:822) L703
- **Calls (std/external/unresolved):** `.map_err`, `.ok_or_else`, `InspectError::Invalid`, `Ok`, `std::str::from_utf8`
- **Macros:** `format!`
- **Called by:** `Package::read` (audit.rs:354) (?); `comments_in` (comments.rs:136); `comment_holds_kept_text` (edit.rs:5236); `update_fields` (fields.rs:115); `Opened::open` (inspect.rs:525); `Opened::summary` (inspect.rs:545); `Opened::stories` (inspect.rs:653)

#### `has_ancestor`  (L709-711)
`fn has_ancestor(dom: &Dom, node: NodeId, name: &XName) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L710
- **Calls (std/external/unresolved):** `.is_empty`, `Some`
- **Called by:** `Opened::summary` (inspect.rs:545); `in_text_box` (inspect.rs:721); `paragraphs_of` (inspect.rs:956)

#### `text_box_names`  (L716-718, pub(crate))
`pub(crate) fn text_box_names() -> [XName; 2]`

- **Does:** The two names a text box's content goes by: `w:txbxContent`, and the `wne:txbxContent` a Strict package's shapes hold (the agent view skips both by local name, so every id walk must too).
- **Calls (std/external/unresolved):** `W::txbx_content`, `WNE::name`
- **Called by:** `in_text_box` (inspect.rs:721); `holds_text_box` (inspect.rs:728)

#### `in_text_box`  (L721-725, pub(crate))
`pub(crate) fn in_text_box(dom: &Dom, node: NodeId) -> bool`

- **Does:** Whether `node` sits inside a text box.
- **Calls (crate):** `text_box_names` (inspect.rs:716) L722; `has_ancestor` (inspect.rs:709) L724
- **Calls (std/external/unresolved):** `.any`, `.iter`
- **Called by:** `Ids::table_paragraph` (edit/origin.rs:95); `body_paragraph_nodes` (inspect.rs:747); `body_control_nodes` (inspect.rs:756)

#### `holds_text_box`  (L728-732)
`fn holds_text_box(dom: &Dom, node: NodeId) -> bool`

- **Does:** Whether `node` holds a text box.
- **Calls (crate):** `text_box_names` (inspect.rs:716) L729; `Dom::descendants` (xmllinq/mod.rs:668) L731
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.iter`, `Some`
- **Called by:** `walk_run` (inspect.rs:1124)

#### `note_count`  (L734-744)
`fn note_count(dom: &Dom, root: NodeId, local: &str) -> usize`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L735; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L739
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.iter`, `Some`, `W::name`
- **Macros:** `matches!`
- **Called by:** `Opened::summary` (inspect.rs:545)

#### `body_paragraph_nodes`  (L747-752, pub(crate))
`pub(crate) fn body_paragraph_nodes(dom: &Dom, body: NodeId) -> Vec<NodeId>`

- **Does:** The body paragraphs (`w:p` outside text boxes) in document order.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L748; `in_text_box` (inspect.rs:721) L750
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `Some`, `W::p`
- **Called by:** `Package::read` (audit.rs:354); `Anchors::scan` (comments.rs:241); `load_stories` (edit.rs:1311); `paragraph_ids` (fields.rs:921); `headings` (fields.rs:1023); `body_tables` (inspect/tables.rs:48); `collect_controls` (inspect.rs:764); `story_paragraph_nodes` (inspect.rs:937); `body_paragraphs` (inspect.rs:952)

#### `body_control_nodes`  (L756-761, pub(crate))
`pub(crate) fn body_control_nodes(dom: &Dom, body: NodeId) -> Vec<NodeId>`

- **Does:** The body's content controls (`w:sdt` outside text boxes) in document order, the order [`collect_controls`] numbers them in.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L757; `in_text_box` (inspect.rs:721) L759
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `Some`, `W::sdt`
- **Called by:** `Origin::build` (edit/origin.rs:256); `Transaction::start` (edit.rs:2004); `collect_controls` (inspect.rs:764)

#### `collect_controls`  (L764-796, pub(crate))
`pub(crate) fn collect_controls(dom: &Dom, body: NodeId) -> Vec<ContentControl>`

- **Does:** One record per [`body_control_nodes`] entry.
- **Calls (crate):** `body_paragraph_nodes` (inspect.rs:747) L765; `body_control_nodes` (inspect.rs:756) L775; `Dom::ancestors` (xmllinq/mod.rs:770) L778; `paragraph_id` (ids.rs:410) L786; `control_record` (inspect.rs:798) L793
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.copied`, `.entry`, `.enumerate`, `.first`, `.get`, `.into_iter`, `.iter`, `.map`, `.or_default`, `Some`, `W::p`, `std::collections::HashMap::new`
- **Macros:** `format!`
- **Called by:** `Origin::build` (edit/origin.rs:256); `Transaction::start` (edit.rs:2004); `controls` (inspect.rs:323); `snapshot` (inspect.rs:362)

#### `control_record`  (L798-873)
`fn control_record( dom: &Dom, sdt: NodeId, id: String, index: &std::collections::HashMap<NodeId, usize>, ) -> ContentControl`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L804,805,826,836; `child` (ambiguous, 4 candidates) L807,812,825,830; `Dom::attribute` (xmllinq/mod.rs:874) L808,818,819,827,831; `control_kind` (inspect.rs:896) L811; `Dom::elements` (xmllinq/mod.rs:640) L815; `Dom::ancestors` (xmllinq/mod.rs:770) L837; `project_container` (inspect.rs:1042) L840; `Dom::descendants` (xmllinq/mod.rs:668) L844; `project_paragraph` (inspect.rs:1024) L851; `paragraph_id` (ids.rs:410) L866
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.collect`, `.copied`, `.filter_map`, `.first`, `.get`, `.into_iter`, `.is_none_or`, `.is_some_and`, `.iter`, `.join`, `.map_or_else`, `.map_or`, `.map`, `.or_else`, `.to_string`, `.unwrap_or_default`, `Some`, `String::new`, `Vec::new`, `W14::name`, `W::name`, `W::p`, `W::sdt_content`, `W::sdt_pr`, `W::val`, `val`
- **Macros:** `matches!`, `vec!`
- **Called by:** `collect_controls` (inspect.rs:764)

#### `on_off`  (L876-878)
`fn on_off(value: &str) -> bool`

- **Does:** `1`, `true` and `on` are set; anything else is clear.
- **Macros:** `matches!`
- **Called by:** none found in src

#### `control_kind`  (L896-934)
`fn control_kind(dom: &Dom, pr: NodeId) -> &'static str`

- **Does:** The control's type from its `w:sdtPr` (see [`ContentControl::kind`]).
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L897; `Dom::name` (xmllinq/mod.rs:526) L898; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L901; `XName::local_name` (xmllinq/mod.rs:208) (by name) L901
- **Calls (std/external/unresolved):** `.contains`
- **Called by:** `control_record` (inspect.rs:798)

#### `story_paragraph_nodes`  (L937-950, pub(crate))
`pub(crate) fn story_paragraph_nodes(dom: &Dom, root: NodeId) -> Vec<NodeId>`

- **Does:** A header, footer or notes part's paragraphs, separator notes left out.
- **Calls (crate):** `body_paragraph_nodes` (inspect.rs:747) L938; `Dom::ancestors` (xmllinq/mod.rs:770) L941; `Dom::name_is` (xmllinq/mod.rs:535) L942; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L944
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.filter`, `.into_iter`, `Some`, `W::name`
- **Macros:** `matches!`
- **Called by:** `Package::read` (audit.rs:354); `paragraph_index` (changes.rs:665); `Anchors::scan` (comments.rs:241); `load_stories` (edit.rs:1311); `paragraph_ids` (fields.rs:921); `Opened::stories` (inspect.rs:653); `tests::story_paragraphs_agree_with_the_paragraphs_edit_resolves` (markdown/from_docx/header.rs:1957)

#### `body_paragraphs`  (L952-954)
`fn body_paragraphs(dom: &Dom, body: NodeId) -> Vec<Paragraph>`

- **Does:** _no doc comment_
- **Calls (crate):** `paragraphs_of` (inspect.rs:956) L953; `body_paragraph_nodes` (inspect.rs:747) L953
- **Called by:** `paragraphs` (inspect.rs:284); `markdown` (inspect.rs:331); `snapshot` (inspect.rs:362); `Opened::summary` (inspect.rs:545)

#### `paragraphs_of`  (L956-981)
`fn paragraphs_of(dom: &Dom, nodes: Vec<NodeId>, story: &str) -> Vec<Paragraph>`

- **Does:** _no doc comment_
- **Calls (crate):** `project_paragraph` (inspect.rs:1024) L961; `Dom::element` (xmllinq/mod.rs:655) L962,964,967; `Dom::attribute` (xmllinq/mod.rs:874) L965; `paragraph_id` (ids.rs:410) L970; `has_ancestor` (inspect.rs:709) L974
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.enumerate`, `.into_iter`, `.is_some_and`, `.is_some`, `.map`, `W::num_pr`, `W::p_pr`, `W::p_style`, `W::tc`, `W::val`
- **Called by:** `Opened::stories` (inspect.rs:653); `body_paragraphs` (inspect.rs:952)

#### `project_paragraph`  (L1024-1038, pub(crate))
`pub(crate) fn project_paragraph(dom: &Dom, paragraph: NodeId) -> Projection`

- **Does:** Project one `w:p` (see module docs for the rules).
- **Calls (crate):** `walk_container` (inspect.rs:1061) L1028; `merge_spans` (inspect.rs:1269) L1036
- **Calls (std/external/unresolved):** `Projection::default`, `Vec::new`
- **Called by:** `Package::literal_bullet` (audit.rs:636); `Package::empty_spacers` (audit.rs:661); `Anchors::scan` (comments.rs:241); `insert_reference` (edit/notes.rs:168); `Origin::build` (edit/origin.rs:256); `formatting_tracked` (edit/tracked.rs:67); `emit` (edit/tracked.rs:275); `emit_text` (edit/tracked.rs:732); `isolate_range` (edit/tracked.rs:826); `logical_text` (edit/v2.rs:783); `Transaction::start` (edit.rs:2004); `Transaction::apply` (edit.rs:4489); +16 more

#### `project_container`  (L1042-1056, pub(crate))
`pub(crate) fn project_container(dom: &Dom, container: NodeId) -> Projection`

- **Does:** Project the inline content of any container (a run-level `w:sdtContent`) with the paragraph rules.
- **Calls (crate):** `walk_container` (inspect.rs:1061) L1046; `merge_spans` (inspect.rs:1269) L1054
- **Calls (std/external/unresolved):** `Projection::default`, `Vec::new`
- **Called by:** `control_record` (inspect.rs:798)

#### `walk_container`  (L1061-1122)
`fn walk_container( dom: &Dom, container: NodeId, direct: bool, fields: &mut usize, out: &mut Projection, formats: &mut Vec<(usize, usize, Format)>, )`

- **Does:** `fields` counts the complex fields open at this point of the paragraph: their `begin`/`end` runs are siblings of the result runs, so a result run is direct by position yet belongs to the field.
- **Calls (crate):** `Dom::elements` (xmllinq/mod.rs:640) L1069; `Dom::name` (xmllinq/mod.rs:526) L1070; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1073; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1074; `Dom::element` (xmllinq/mod.rs:655) L1078,1110; `walk_container` (inspect.rs:1061) L1080,1097,1102,1106,1111,1115; `walk_run` (inspect.rs:1124) L1099
- **Calls (std/external/unresolved):** `.insert`, `.into`, `MC::name`, `W::sdt_content`
- **Macros:** `format!`
- **Called by:** `project_paragraph` (inspect.rs:1024); `project_container` (inspect.rs:1042); `walk_container` (inspect.rs:1061)

#### `walk_run`  (L1124-1243)
`fn walk_run( dom: &Dom, run: NodeId, direct: bool, fields: &mut usize, out: &mut Projection, formats: &mut Vec<(usize, usize, Format)>, )`

- **Does:** _no doc comment_
- **Calls (crate):** `run_format` (inspect.rs:1245) L1132; `Dom::elements` (xmllinq/mod.rs:640) L1134; `Dom::name` (xmllinq/mod.rs:526) L1135; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1140,1153; `Dom::descendants` (xmllinq/mod.rs:668) L1144; `holds_text_box` (inspect.rs:728) L1148,1210; `XName::local_name` (xmllinq/mod.rs:208) (by name) L1155,1172; `Dom::value_str` (xmllinq/mod.rs:1179) L1175; `tests::glyph` (convert/math_layout.rs:799) (by name) L1184,1185,1191,1193,1194,1197; `Dom::attribute` (xmllinq/mod.rs:874) L1186,1202
- **Calls (std/external/unresolved):** `.any`, `.insert`, `.into`, `.is_empty`, `.iter`, `.len`, `.push_str`, `.push`, `.saturating_sub`, `Some`, `W::name`
- **Macros:** `format!`
- **Called by:** `walk_container` (inspect.rs:1061)

#### `run_format`  (L1245-1267)
`fn run_format(dom: &Dom, run: NodeId) -> Format`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L1246,1250,1253,1256; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L1251; `Dom::attribute` (xmllinq/mod.rs:874) L1255,1258; `on` (ambiguous: `on` (audit.rs:299); `on` (debug/render.rs:446)) L1262,1263
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.is_none_or`, `.is_some_and`, `.map`, `Format::default`, `Some`, `W::name`, `W::r_pr`, `W::val`
- **Macros:** `matches!`
- **Called by:** `walk_run` (inspect.rs:1124)

#### `merge_spans`  (L1269-1293)
`fn merge_spans(text: &str, formats: Vec<(usize, usize, Format)>) -> Vec<Span>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.count`, `.into_iter`, `.last_mut`, `.map`, `.push`, `Vec::new`, `char_at`
- **Called by:** `project_paragraph` (inspect.rs:1024); `project_container` (inspect.rs:1042); `tests::spans_merge_only_adjacent_equal_formatting` (inspect.rs:1348)

## Test functions (4)

- `tests::sha256_is_lowercase_hex_of_the_bytes` (L1301): _no doc comment_
- `tests::markdown_moves_edge_whitespace_outside_emphasis` (L1309): _no doc comment_
- `tests::markdown_of_empty_and_styled_paragraphs` (L1330): _no doc comment_
- `tests::spans_merge_only_adjacent_equal_formatting` (L1348): _no doc comment_
