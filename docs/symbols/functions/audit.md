# Functions and methods in `src/audit.rs`

39 non-test, 3 test. Part of the [symbol manifest](../README.md).

#### `<AuditError as fmt::Display>::fmt`  (L95-109, impl fmt::Display)
`fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.join`, `.map`
- **Macros:** `write!`
- **Called by:** none found in src

#### `<AuditError as From>::from`  (L115-117, impl From)
`fn from(error: InspectError) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Self::Inspect`
- **Called by:** none found in src

#### `audit`  (L127-129, pub)
`pub fn audit(docx: &[u8], rules: &[&str]) -> Result<Vec<AuditFinding>, AuditError>`

- **Does:** Audit `docx` with `rules` (rule sets or codes; empty selects every rule).
- **Calls (crate):** `audit_report` (audit.rs:136) L128
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** none found in src

#### `audit_report`  (L136-138, pub)
`pub fn audit_report(docx: &[u8], rules: &[&str]) -> Result<AuditReport, AuditError>`

- **Does:** [`audit`] plus the codes that ran and whether a layout pass ran.
- **Calls (crate):** `audit_report_with` (audit.rs:177) L137
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `audit` (audit.rs:127)

#### `word_layout`  (L147-153, pub)
`pub fn word_layout(docx: &[u8]) -> Result<RenderReport, AuditError>`

- **Does:** The native layout pass: the document laid out in [`RevisionStyle::Word`], whose page count the `NUMPAGES` cache is checked against.
- **Calls (crate):** `docx_render_report` (convert/mod.rs:735) L152
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`, `AuditError::Layout`, `PdfOptions::default`
- **Called by:** none found in src

#### `is_rule`  (L156-158, pub)
`pub fn is_rule(rule: &str) -> bool`

- **Does:** True when `rule` is one of the audit's rule sets or codes.
- **Calls (std/external/unresolved):** `.any`, `.contains`, `.iter`
- **Called by:** `parse_rule` (cli.rs:2414); `validate_with` (validate.rs:290)

#### `audit_report_with`  (L177-235, pub)
`pub fn audit_report_with( docx: &[u8], rules: &[&str], mut layout: Option<LayoutPass<'_>>, ) -> Result<AuditReport, AuditError>`

- **Does:** [`audit_report`] with the layout pass supplied by the caller, so a build without the renderer (the slim WASM build) can audit without linking it.
- **Calls (crate):** `select` (audit.rs:239) L182; `Opened::open` (inspect.rs:525) L183; `Package::read` (ambiguous: `Package::read` (audit.rs:354); `Package::read` (markdown/from_docx/ooxml.rs:66)) L184; `Package::heading_skip` (audit.rs:532) L189; `Package::image_no_descr` (audit.rs:558) L190; `Package::table_no_header_row` (audit.rs:586) L191; `Package::missing_lang` (audit.rs:617) L192; `Package::literal_bullet` (audit.rs:636) L193; `Package::empty_spacers` (audit.rs:661) L194; `Package::direct_formatting` (audit.rs:714) L195; `Package::fields` (audit.rs:764) L197; `.number` (ambiguous: `Field::number` (audit.rs:862); `Step::number` (ids/short.rs:96)) L200; `lay_out` (audit.rs:274) L202,211; `stale_fields` (audit.rs:869) L205; `FontReportEntry::substituted` (convert/font.rs:350) (by name) L213; `finding` (audit.rs:285) L215
- **Calls (std/external/unresolved):** `.any`, `.as_deref_mut`, `.as_str`, `.clone`, `.collect`, `.expect`, `.filter`, `.insert`, `.is_some`, `.iter`, `.map`, `.push`, `.to_string`, `BTreeSet::new`, `Ok`, `Some`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `audit_report` (audit.rs:136); `public_memory_spacer_ownership_tests::spacer_audit_respects_authored_sections_page_breaks_and_table_cell_ownership` (audit.rs:983); `validate_with` (validate.rs:290)

#### `select`  (L239-271)
`fn select( requested: &[&str], can_lay_out: bool, ) -> Result<Vec<(&'static str, &'static str, &'static str)>, AuditError>`

- **Does:** The rules `requested` names, in [`RULES`] order.
- **Calls (crate):** `names` (ambiguous: `deeper_boundary_tests::names` (edit.rs:6691); `canonical::{fn names}` (markdown/from_docx/drawing.rs:872); `move_range_bookmark_regressions::names` (revision_processor/bookmarks.rs:172)) L246,252
- **Calls (std/external/unresolved):** `.any`, `.collect`, `.contains`, `.copied`, `.extend`, `.filter`, `.is_empty`, `.iter`, `.map`, `.remove`, `.to_string`, `.trim`, `AuditError::Layout`, `AuditError::UnknownRule`, `BTreeSet::new`, `Err`, `Ok`
- **Called by:** `audit_report_with` (audit.rs:177); `resolve_selected` (edit.rs:1161) (?)

#### `lay_out`  (L274-283)
`fn lay_out<'a>( docx: &[u8], pass: &mut dyn FnMut(&[u8]) -> Result<RenderReport, AuditError>, slot: &'a mut Option<RenderReport>, ) -> Result<&'a RenderReport, AuditError>`

- **Does:** The layout report, computed by `pass` on first use.
- **Calls (std/external/unresolved):** `.as_ref`, `.expect`, `.is_none`, `Ok`, `Some`, `pass`
- **Called by:** `audit_report_with` (audit.rs:177)

#### `finding`  (L285-297)
`fn finding(code: &str, location: String, message: String) -> AuditFinding`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.expect`, `.find`, `.iter`, `.to_string`
- **Called by:** `audit_report_with` (audit.rs:177); `Package::heading_skip` (audit.rs:532); `Package::image_no_descr` (audit.rs:558); `Package::table_no_header_row` (audit.rs:586); `Package::missing_lang` (audit.rs:617); `Package::literal_bullet` (audit.rs:636); `Package::empty_spacers` (audit.rs:661); `Package::direct_formatting` (audit.rs:714); `stale_fields` (audit.rs:869)

#### `on`  (L299-301)
`fn on(value: Option<&str>) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Some`
- **Macros:** `matches!`
- **Called by:** `Package::table_no_header_row` (audit.rs:586); `parse_tbl_style` (convert/mod.rs:3853) (?); `table_block` (convert/mod.rs:13059) (?); `form_checkbox` (convert/mod.rs:16132) (?); `analyze` (debug.rs:1057) (?); `run_format` (inspect.rs:1245) (?); `chpx_props` (legacy_doc.rs:1450) (?)

#### `Story::location`  (L317-322)
`fn location(&self, node: NodeId) -> String`

- **Does:** The id of the indexed paragraph holding `node` (the nearest one, so an image in a text box reports the paragraph that anchors it).
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L319; `Story::paragraph_id` (audit.rs:324) L321
- **Calls (std/external/unresolved):** `.chain`, `.clone`, `.find_map`, `.iter`, `.map_or_else`, `.position`, `Some`, `W::p`, `std::iter::once`
- **Called by:** `Package::image_no_descr` (audit.rs:558) (?); `Package::table_no_header_row` (audit.rs:586) (?); `Field::finish` (audit.rs:848) (?)

#### `Story::paragraph_id`  (L324-326)
`fn paragraph_id(&self, index: usize) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `paragraph_id` (ids.rs:410) L325
- **Called by:** `Story::location` (audit.rs:317); `Package::heading_skip` (audit.rs:532) (?); `Package::literal_bullet` (audit.rs:636) (?); `Package::empty_spacers` (audit.rs:661) (?); `Transaction::cell_control` (edit/controls.rs:298) (?); `Transaction::resolve_comment_text` (edit.rs:2242) (?); `Transaction::resolve_one` (edit.rs:2528) (?); `Transaction::resolve_list` (edit.rs:3563) (?); `Transaction::select` (edit.rs:3778) (?)

#### `Package::read`  (L354-404)
`fn read(opened: Opened) -> Result<Self, AuditError>`

- **Does:** _no doc comment_
- **Calls (crate):** `Opened::story_parts` (inspect.rs:641) L355; `Opened::related` (inspect.rs:669) L356,357; `body_paragraph_nodes` (inspect.rs:747) L368; `parse_part` (ambiguous: `parse_part` (inspect.rs:693); `parse_part` (validate.rs:570)) L373,394,398; `story_paragraph_nodes` (inspect.rs:937) L376; `Dom::element` (xmllinq/mod.rs:655) (by name) L399; `has_lang` (audit.rs:911) L401
- **Calls (std/external/unresolved):** `.clone`, `.into_iter`, `.is_some_and`, `.next`, `.push`, `.read_styles`, `.to_string`, `BTreeMap::new`, `Ok`, `W::name`
- **Macros:** `vec!`
- **Called by:** `audit_report_with` (audit.rs:177) (?); `Deps::dep_for` (markdown/from_docx/drawing.rs:438) (?); `Package::xml` (markdown/from_docx/ooxml.rs:93) (?); `Media::load` (markdown/from_docx/ooxml.rs:1052) (?)

#### `Package::read_styles`  (L406-473)
`fn read_styles(&mut self, dom: &Dom, root: NodeId)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L407,408,410,439,446,448...; `run_font_and_size` (audit.rs:897) L412,453; `Dom::descendants` (xmllinq/mod.rs:668) L414; `has_lang` (audit.rs:911) L417,457; `Dom::elements` (xmllinq/mod.rs:640) L421,454; `Dom::attribute` (xmllinq/mod.rs:874) L422,425,432,440; `numbering_on` (audit.rs:920) L449
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_deref`, `.eq_ignore_ascii_case`, `.get_or_insert_with`, `.insert`, `.into_iter`, `.is_some_and`, `.map`, `.or`, `.to_string`, `.unwrap_or`, `Some`, `StyleFacts::default`, `W::name`, `W::num_pr`, `W::p_pr`, `W::r_pr`, `W::val`, `child_val`
- **Macros:** `matches!`
- **Called by:** none found in src

#### `Package::style_chain`  (L476-491)
`fn style_chain(&self, id: &str) -> Vec<&StyleFacts>`

- **Does:** Paragraph styles from `id` up its `basedOn` chain (cycles cut).
- **Calls (std/external/unresolved):** `.clone`, `.get`, `.insert`, `.push`, `.to_string`, `BTreeSet::new`, `Some`, `Vec::new`
- **Called by:** `Package::numbered` (audit.rs:513); `Package::missing_lang` (audit.rs:617); `Package::direct_formatting` (audit.rs:714)

#### `Package::paragraph_style`  (L493-497)
`fn paragraph_style<'a>(&self, dom: &'a Dom, p: NodeId) -> Option<&'a str>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L494,495; `Dom::attribute` (xmllinq/mod.rs:874) L496
- **Calls (std/external/unresolved):** `.and_then`, `W::p_pr`, `W::p_style`, `W::val`
- **Called by:** `Package::numbered` (audit.rs:513); `Package::heading_skip` (audit.rs:532); `Package::direct_formatting` (audit.rs:714)

#### `Package::heading_level`  (L502-509)
`fn heading_level(&self, style: &str) -> Option<u32>`

- **Does:** The heading level of a paragraph style: its `w:name` is `heading N` (how Word names them in every UI language), else its id is `HeadingN`.
- **Calls (crate):** `level_after` (audit.rs:927) L507,508
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.get`, `.or_else`
- **Called by:** `Package::heading_skip` (audit.rs:532); `markdown_source_owner_boundary_tests::style_title_outline_and_long_ancestry_follow_authored_nearest_properties` (markdown/from_docx/mod.rs:6469) (?)

#### `Package::numbered`  (L513-526)
`fn numbered(&self, dom: &Dom, p: NodeId) -> bool`

- **Does:** Direct `w:numPr` decides (a `numId` of 0 removes numbering); otherwise the style chain's.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L514,516; `numbering_on` (audit.rs:920) L517; `Package::paragraph_style` (audit.rs:493) L519; `Package::style_chain` (audit.rs:476) L520
- **Calls (std/external/unresolved):** `.and_then`, `.find_map`, `.into_iter`, `.is_some_and`, `.unwrap_or_else`, `.unwrap_or`, `W::num_pr`, `W::p_pr`
- **Called by:** `Package::literal_bullet` (audit.rs:636)

#### `Package::body`  (L528-530)
`fn body(&self) -> &Story`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `Package::heading_skip` (audit.rs:532); `Package::direct_formatting` (audit.rs:714)

#### `Package::heading_skip`  (L532-556)
`fn heading_skip(&self, findings: &mut Vec<AuditFinding>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Package::body` (audit.rs:528) L533; `Package::paragraph_style` (audit.rs:493) L536; `Package::heading_level` (audit.rs:502) L538; `finding` (audit.rs:285) L552; `.paragraph_id` (ambiguous: `Story::paragraph_id` (audit.rs:324); `Ids::paragraph_id` (edit/origin.rs:48); `Transaction::paragraph_id` (edit.rs:3876)) L552
- **Calls (std/external/unresolved):** `.and_then`, `.enumerate`, `.iter`, `.push`, `Some`
- **Macros:** `format!`
- **Called by:** `audit_report_with` (audit.rs:177)

#### `Package::image_no_descr`  (L558-584)
`fn image_no_descr(&self, findings: &mut Vec<AuditFinding>)`

- **Does:** _no doc comment_
- **Calls (crate):** `XName::get` (xmllinq/mod.rs:180) L559,573; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L562; `under_fallback` (audit.rs:954) L563; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L566,572; `decorative` (audit.rs:942) L569; `finding` (audit.rs:285) L575; `Story::location` (audit.rs:317) (by name) L577
- **Calls (std/external/unresolved):** `.is_empty`, `.is_some_and`, `.push`, `.trim`, `.unwrap_or`, `Some`, `WP::name`
- **Macros:** `format!`
- **Called by:** `audit_report_with` (audit.rs:177)

#### `Package::table_no_header_row`  (L586-615)
`fn table_no_header_row(&self, findings: &mut Vec<AuditFinding>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) (by name) L589,601; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L590; `Dom::element` (xmllinq/mod.rs:655) (by name) L594,596; `on` (audit.rs:299) L597; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L597; `Story::location` (audit.rs:317) (by name) L604; `finding` (audit.rs:285) L605
- **Calls (std/external/unresolved):** `.and_then`, `.clone`, `.first`, `.is_some_and`, `.len`, `.map_or_else`, `.push`, `Some`, `W::name`, `W::p`, `W::tbl`, `W::tr_pr`, `W::tr`, `W::val`
- **Macros:** `format!`
- **Called by:** `audit_report_with` (audit.rs:177)

#### `Package::missing_lang`  (L617-634)
`fn missing_lang(&self, findings: &mut Vec<AuditFinding>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Package::style_chain` (audit.rs:476) L618; `finding` (audit.rs:285) L625
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.into_iter`, `.push`, `.to_string`, `.unwrap_or_else`
- **Called by:** `audit_report_with` (audit.rs:177)

#### `Package::literal_bullet`  (L636-659)
`fn literal_bullet(&self, findings: &mut Vec<AuditFinding>)`

- **Does:** _no doc comment_
- **Calls (crate):** `project_paragraph` (inspect.rs:1024) L639; `Package::numbered` (audit.rs:513) L647; `finding` (audit.rs:285) L650; `.paragraph_id` (ambiguous: `Story::paragraph_id` (audit.rs:324); `Ids::paragraph_id` (edit/origin.rs:48); `Transaction::paragraph_id` (edit.rs:3876)) L652
- **Calls (std/external/unresolved):** `.chars`, `.contains`, `.enumerate`, `.iter`, `.next`, `.push`, `.trim_start_matches`
- **Macros:** `format!`, `matches!`
- **Called by:** `audit_report_with` (audit.rs:177)

#### `Package::empty_spacers`  (L661-712)
`fn empty_spacers(&self, findings: &mut Vec<AuditFinding>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) (by name) L665,666; `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L669; `project_paragraph` (inspect.rs:1024) L672; `deep_production_path_tests::empty` (comparer/finalize.rs:20895) (by name) L680,687; `Dom::parent` (xmllinq/mod.rs:597) (by name) L686,696; `finding` (audit.rs:285) L699; `.paragraph_id` (ambiguous: `Story::paragraph_id` (audit.rs:324); `Ids::paragraph_id` (edit/origin.rs:48); `Transaction::paragraph_id` (edit.rs:3876)) L701,704,705; `short_paragraph` (ids.rs:435) L704,705
- **Calls (std/external/unresolved):** `.get`, `.is_empty`, `.is_some_and`, `.is_some`, `.len`, `.push`, `.trim`, `Some`, `W::p_pr`, `W::sect_pr`, `W::tc`, `breaks_section`
- **Macros:** `format!`
- **Called by:** `audit_report_with` (audit.rs:177)

#### `Package::direct_formatting`  (L714-761)
`fn direct_formatting(&self, findings: &mut Vec<AuditFinding>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Package::style_chain` (audit.rs:476) L715; `Package::body` (audit.rs:528) L724; `Package::paragraph_style` (audit.rs:493) L728; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L734; `Dom::ancestors` (xmllinq/mod.rs:770) (by name) L735; `Dom::element` (xmllinq/mod.rs:655) (by name) L736,740; `run_font_and_size` (audit.rs:897) L743; `finding` (audit.rs:285) L752
- **Calls (std/external/unresolved):** `.clone`, `.find_map`, `.first`, `.is_none_or`, `.is_none`, `.is_some`, `.iter`, `.or_else`, `.push`, `Some`, `W::p`, `W::r_pr`, `W::r`, `W::t`
- **Macros:** `format!`
- **Called by:** `audit_report_with` (audit.rs:177)

#### `Package::fields`  (L764-820)
`fn fields(&self) -> Vec<Field>`

- **Does:** Every complex and simple field, story by story in document order.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) (by name) L769,805; `under_fallback` (audit.rs:954) L770; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L773,789,793,798; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L774,800; `Field::new` (audit.rs:836) L775,799; `Field::finish` (audit.rs:848) L783,810,815; `Dom::value_str` (xmllinq/mod.rs:1179) (by name) L791,794
- **Calls (std/external/unresolved):** `.collect`, `.enumerate`, `.filter`, `.into_iter`, `.iter_mut`, `.iter`, `.last_mut`, `.map`, `.pop`, `.push_str`, `.push`, `.rev`, `.to_string`, `.unwrap_or_default`, `.value`, `Some`, `Vec::new`, `W::fld_char`, `W::fld_simple`, `W::name`, `W::t`
- **Called by:** `audit_report_with` (audit.rs:177)

#### `Field::new`  (L836-846)
`fn new(story: usize, node: NodeId) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `String::new`
- **Called by:** `Package::fields` (audit.rs:764)

#### `Field::finish`  (L848-856)
`fn finish(&mut self, package: &Package)`

- **Does:** _no doc comment_
- **Calls (crate):** `Story::location` (audit.rs:317) (by name) L855
- **Calls (std/external/unresolved):** `.next`, `.split_whitespace`, `.to_ascii_uppercase`, `.unwrap_or`
- **Called by:** `Package::fields` (audit.rs:764)

#### `Field::cached`  (L858-860)
`fn cached(&self) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_empty`, `.trim`
- **Called by:** `Field::number` (audit.rs:862); `stale_fields` (audit.rs:869) (?)

#### `Field::number`  (L862-866)
`fn number(&self) -> Option<usize>`

- **Does:** _no doc comment_
- **Calls (crate):** `Field::cached` (audit.rs:858) L863
- **Calls (std/external/unresolved):** `.flatten`, `.ok`, `.parse`, `.then`, `.trim`
- **Called by:** `audit_report_with` (audit.rs:177) (?); `stale_fields` (audit.rs:869) (?); `ShortId::range` (ids/short.rs:342) (?)

#### `stale_fields`  (L869-894)
`fn stale_fields(fields: &[Field], pages: Option<usize>, findings: &mut Vec<AuditFinding>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Field::cached` (audit.rs:858) (by name) L872,875; `.number` (ambiguous: `Field::number` (audit.rs:862); `Step::number` (ids/short.rs:96)) L878; `finding` (audit.rs:285) L887
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.push`, `.to_string`, `Some`
- **Macros:** `format!`
- **Called by:** `audit_report_with` (audit.rs:177)

#### `run_font_and_size`  (L897-908)
`fn run_font_and_size(dom: &Dom, rpr: NodeId) -> (Option<String>, Option<String>)`

- **Does:** `rFonts` (`ascii`, else `hAnsi`) and `sz` of an `rPr`.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L898,903; `Dom::attribute` (xmllinq/mod.rs:874) L899,900,905
- **Calls (std/external/unresolved):** `.and_then`, `.map`, `.or_else`, `W::name`, `W::val`
- **Called by:** `Package::read_styles` (audit.rs:406); `Package::direct_formatting` (audit.rs:714)

#### `has_lang`  (L911-916)
`fn has_lang(dom: &Dom, lang: NodeId) -> bool`

- **Does:** A `w:lang`-shaped element naming any language.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L913
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.is_some_and`, `.iter`, `.trim`, `W::name`
- **Called by:** `Package::read` (audit.rs:354); `Package::read_styles` (audit.rs:406)

#### `numbering_on`  (L920-924)
`fn numbering_on(dom: &Dom, num_pr: NodeId) -> Option<bool>`

- **Does:** `Some(false)` when a `numPr` names `numId` 0 (numbering removed), `Some(true)` for any other `numId`, `None` when it names none.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L921; `Dom::attribute` (xmllinq/mod.rs:874) L922
- **Calls (std/external/unresolved):** `.and_then`, `.map`, `.trim`, `W::name`, `W::val`
- **Called by:** `Package::read_styles` (audit.rs:406); `Package::numbered` (audit.rs:513)

#### `level_after`  (L927-937)
`fn level_after(text: &str, prefix: &str) -> Option<u32>`

- **Does:** `N` from `{prefix}N` (case-insensitive), for heading levels 1-9.
- **Calls (std/external/unresolved):** `.contains`, `.eq_ignore_ascii_case`, `.filter`, `.get`, `.len`, `.ok`, `.parse`, `.trim`
- **Called by:** `Package::heading_level` (audit.rs:502)

#### `decorative`  (L942-951)
`fn decorative(dom: &Dom, doc_pr: NodeId) -> bool`

- **Does:** The Office 2019 decorative flag (`adec:decorative val="1"` in the `docPr`'s extension list).
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) L943; `Dom::name` (xmllinq/mod.rs:526) L944; `XName::local_name` (xmllinq/mod.rs:208) (by name) L945; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L947; `XName::get` (xmllinq/mod.rs:180) L947
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.is_some_and`, `Some`
- **Macros:** `matches!`
- **Called by:** `Package::image_no_descr` (audit.rs:558)

#### `under_fallback`  (L954-956)
`fn under_fallback(dom: &Dom, node: NodeId) -> bool`

- **Does:** Inside an `mc:Fallback`, which repeats its `mc:Choice` for old readers.
- **Calls (crate):** `Dom::ancestors` (xmllinq/mod.rs:770) L955
- **Calls (std/external/unresolved):** `.is_empty`, `MC::name`, `Some`
- **Called by:** `Package::image_no_descr` (audit.rs:558); `Package::fields` (audit.rs:764)

## Test functions (3)

- `public_memory_spacer_ownership_tests::package` (L964): _no doc comment_
- `public_memory_spacer_ownership_tests::snapshot` (L970): _no doc comment_
- `public_memory_spacer_ownership_tests::spacer_audit_respects_authored_sections_page_breaks_and_table_cell_ownership` (L983): _no doc comment_
