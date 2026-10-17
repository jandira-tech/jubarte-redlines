# Functions and methods in `src/edit/watermark.rs`

14 non-test, 3 test. Part of the [symbol manifest](../README.md).

#### `spec`  (L43-73, pub(super))
`pub(super) fn spec( text: &str, color: Option<&str>, diagonal: Option<bool>, font: Option<&str>, ) -> Result<Spec, String>`

- **Does:** Check the operation's parameters; `None` takes Word's defaults (`C0C0C0`, diagonal, Calibri).
- **Calls (crate):** `check_text` (edit.rs:5368) L49,63
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.contains`, `.count`, `.is_ascii_hexdigit`, `.is_empty`, `.len`, `.to_ascii_uppercase`, `.to_string`, `.trim`, `.unwrap_or`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `Transaction::resolve_watermark` (edit/watermark.rs:174); `tests::defaults_follow_word` (edit/watermark.rs:338); `tests::horizontal_markup_has_no_rotation` (edit/watermark.rs:354)

#### `points`  (L76-79)
`fn points(v: f64) -> String`

- **Does:** `12.30` -> `12.3`, `406.17` stays.
- **Calls (std/external/unresolved):** `.to_string`, `.trim_end_matches`
- **Macros:** `format!`
- **Called by:** `markup` (edit/watermark.rs:91); `tests::points_drop_trailing_zeros` (edit/watermark.rs:347)

#### `escape`  (L81-86)
`fn escape(s: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.replace`
- **Called by:** `markup` (edit/watermark.rs:91)

#### `markup`  (L91-121)
`fn markup(spec: &Spec, n: usize, size: (f64, f64)) -> String`

- **Does:** Word's watermark paragraph for header number `n` (1-based) with a shape of `size` points: the `Watermarks` content control around the header paragraph holding the VML shape type and shape.
- **Calls (crate):** `points` (edit/watermark.rs:76) L93; `escape` (edit/watermark.rs:81) L95
- **Macros:** `concat!`, `format!`
- **Called by:** `Transaction::apply_watermark` (edit/watermark.rs:254); `tests::horizontal_markup_has_no_rotation` (edit/watermark.rs:354)

#### `header_open`  (L124-130)
`fn header_open() -> String`

- **Does:** The `w:hdr` start tag with every namespace the markup uses.
- **Macros:** `format!`
- **Called by:** `Transaction::mark_header` (edit/watermark.rs:284); `Transaction::new_header` (edit/watermark.rs:306)

#### `twips`  (L132-135)
`fn twips(dom: &Dom, node: Option<NodeId>, attr: &str) -> Option<f64>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L133
- **Calls (std/external/unresolved):** `.and_then`, `.ok`, `.parse`, `W::name`
- **Called by:** `paragraph_block` (convert/mod.rs:11573) (?); `shape_size` (edit/watermark.rs:139); `text_width` (fields.rs:1135) (?); `IdLine::clause` (markdown/view_notes.rs:1349) (?)

#### `shape_size`  (L139-153)
`fn shape_size(dom: &Dom, sect_pr: NodeId) -> (f64, f64)`

- **Does:** The shape size for a section: Word's own numbers on a portrait Letter page; elsewhere 0.9 of the text width at Word's 4:1 ratio.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L140,141; `twips` (edit/watermark.rs:132) L142,148,149
- **Calls (std/external/unresolved):** `.max`, `.round`, `.unwrap_or`, `Some`, `W::name`
- **Called by:** `Transaction::apply_watermark` (edit/watermark.rs:254)

#### `holds_watermark`  (L156-169)
`fn holds_watermark(dom: &Dom, root: NodeId) -> bool`

- **Does:** Whether a header already holds a text or picture watermark.
- **Calls (crate):** `XNamespace::get` (xmllinq/mod.rs:108) L157; `Dom::descendants` (xmllinq/mod.rs:668) L161,164; `named` (ambiguous: `coverage_round_three_tests::coverage_round_five_tests::named` (convert/mod.rs:50359); `named` (markdown/from_docx/header.rs:941); `named` (markdown/write.rs:5387)) L163,168; `Dom::attribute` (xmllinq/mod.rs:874) L163,168; `XName::get` (xmllinq/mod.rs:180) L163,168; `Dom::name` (xmllinq/mod.rs:526) L167; `XName::local_name` (xmllinq/mod.rs:208) (by name) L167
- **Calls (std/external/unresolved):** `.any`, `.filter`, `.into_iter`, `.is_some_and`, `.iter`, `.name`, `.starts_with`, `Some`
- **Called by:** `Transaction::resolve_watermark` (edit/watermark.rs:174)

#### `Transaction::resolve_watermark`  (L174-216, pub(super))
`pub(super) fn resolve_watermark( &self, id: &str, params: (&str, Option<&str>, Option<bool>, Option<&str>), ) -> Result<(Spec, EditOutcome), Box<(EditError, EditOutcome)>>`

- **Does:** Resolve a `watermark` operation: check its parameters and that the document holds no watermark yet.
- **Calls (crate):** `err` (edit.rs:1401) L193; `spec` (edit/watermark.rs:43) L196; `Dom::name_is` (xmllinq/mod.rs:535) L203; `holds_watermark` (edit/watermark.rs:156) L204
- **Calls (std/external/unresolved):** `.any`, `.filter`, `.into`, `.is_some`, `.iter`, `.to_string`, `Box::new`, `Err`, `Ok`, `Some`, `String::new`, `W::name`, `fail`
- **Macros:** `format!`
- **Called by:** `Transaction::resolve` (edit.rs:2375)

#### `Transaction::section_properties`  (L220-230)
`fn section_properties(&self) -> Vec<NodeId>`

- **Does:** Every section's `w:sectPr` in document order (paragraph section breaks, then the body's own), leaving out tracked old properties.
- **Calls (crate):** `Dom::descendants` (xmllinq/mod.rs:668) (by name) L223; `Dom::parent` (xmllinq/mod.rs:597) (by name) L226; `Dom::name_is` (xmllinq/mod.rs:535) (by name) L227
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`, `.is_some_and`, `Some`, `W::p_pr`, `W::sect_pr`
- **Called by:** `Transaction::apply_watermark` (edit/watermark.rs:254)

#### `Transaction::default_header`  (L233-250)
`fn default_header(&self, sect_pr: NodeId) -> Option<Option<usize>>`

- **Does:** The story index of the default header a section references, if any.
- **Calls (crate):** `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L235; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L238,239; `PartFs::read_rels_for` (opc/mod.rs:756) (by name) L241; `PartFs::resolve_rel_target` (opc/mod.rs:600) L248
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.iter`, `.map`, `.position`, `R::name`, `Some`, `W::name`
- **Called by:** `Transaction::apply_watermark` (edit/watermark.rs:254)

#### `Transaction::apply_watermark`  (L254-281, pub(super))
`pub(super) fn apply_watermark(&mut self) -> Result<BTreeSet<usize>, EditError>`

- **Does:** Write the resolved watermark into every default header; returns the header stories it changed (a new header part is written here).
- **Calls (crate):** `Transaction::section_properties` (edit/watermark.rs:220) L260; `shape_size` (edit/watermark.rs:139) L261; `Transaction::default_header` (edit/watermark.rs:233) L262; `Transaction::mark_header` (edit/watermark.rs:284) L266; `markup` (edit/watermark.rs:91) L266,276; `Transaction::new_header` (edit/watermark.rs:306) L276
- **Calls (std/external/unresolved):** `.clone`, `.enumerate`, `.insert`, `.into_iter`, `BTreeSet::new`, `Ok`
- **Called by:** `Transaction::finish` (edit.rs:5077)

#### `Transaction::mark_header`  (L284-301)
`fn mark_header(&mut self, story: usize, markup: &str) -> Result<(), EditError>`

- **Does:** Put the watermark first in an existing header story.
- **Calls (crate):** `Dom::parse_xdocument` (xmllinq/mod.rs:1283) (by name) L287; `header_open` (edit/watermark.rs:124) L287; `.root` (ambiguous: `Dom::root` (xmllinq/mod.rs:822); `Scope::root` (xmllinq/serialize.rs:211)) L288; `Dom::element` (xmllinq/mod.rs:655) (by name) L290; `err` (edit.rs:1401) L291; `Dom::add_first` (xmllinq/mod.rs:1085) (by name) L293; `XNamespace::xmlns` (xmllinq/mod.rs:122) L294; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L296; `XNamespace::name` (xmllinq/mod.rs:134) L296,297; `Dom::set_attribute_value` (xmllinq/mod.rs:961) (by name) L297
- **Calls (std/external/unresolved):** `.and_then`, `.is_none`, `.ok_or_else`, `.remove`, `Ok`, `Some`, `W::sdt`
- **Macros:** `format!`
- **Called by:** `Transaction::apply_watermark` (edit/watermark.rs:254)

#### `Transaction::new_header`  (L306-329)
`fn new_header(&mut self, sect_pr: NodeId, markup: &str)`

- **Does:** Create `headerN.xml` beside the main part (`word/` in every package Word writes) holding only the watermark, relate it to the main part and reference it first in the section's properties.
- **Calls (crate):** `PartFs::parts` (opc/mod.rs:532) L311; `header_open` (edit/watermark.rs:124) L318; `PartFs::set_part` (opc/mod.rs:510) (by name) L321; `PartFs::add_content_type_override` (opc/mod.rs:616) (by name) L322; `PartFs::add_document_relationship` (opc/mod.rs:772) (by name) L323; `Dom::new_element` (xmllinq/mod.rs:478) (by name) L325; `Dom::set_attribute_value` (xmllinq/mod.rs:961) (by name) L326,327; `Dom::add_first` (xmllinq/mod.rs:1085) (by name) L328
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.find`, `.into_bytes`, `.into_iter`, `.rsplit_once`, `.unwrap_or`, `R::name`, `Some`, `String::new`, `W::name`
- **Macros:** `format!`
- **Called by:** `Transaction::apply_watermark` (edit/watermark.rs:254)

## Test functions (3)

- `tests::defaults_follow_word` (L338): _no doc comment_
- `tests::points_drop_trailing_zeros` (L347): _no doc comment_
- `tests::horizontal_markup_has_no_rotation` (L354): _no doc comment_
