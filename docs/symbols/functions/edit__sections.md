# Functions and methods in `src/edit/sections.rs`

12 non-test, 8 test. Part of the [symbol manifest](../README.md).

#### `Geometry::of`  (L151-180)
`fn of(dom: &Dom, sect: Option<NodeId>) -> Self`

- **Does:** The geometry `sect` sets, Word's default for what it leaves out.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L155,162; `Dom::element` (xmllinq/mod.rs:655) L158,164; `side` (markdown/borders.rs:189) (by name) L160,161; `read` (ambiguous: `read` (legacy_doc.rs:180); `read` (markdown/mod.rs:749); `read` (markdown/write.rs:125)) L160,161,174
- **Calls (std/external/unresolved):** `.and_then`, `.ok`, `.parse`, `.trim`, `.unwrap_or`, `Some`, `W::name`, `u32::try_from`
- **Called by:** `Transaction::resolve_page_setup` (edit/sections.rs:363)

#### `Geometry::changed`  (L183-264)
`fn changed( mut self, page: Option<PageSize>, orientation: Option<Orientation>, margins: Margins, ) -> Result<Self, String>`

- **Does:** The geometry after the plan's changes, or why it is refused.
- **Calls (crate):** `set` (ambiguous: `deeper_boundary_tests::set` (comparer/comments.rs:1863); `set` (comparer/tables.rs:29)) L230,231,232,233,234,235
- **Calls (std/external/unresolved):** `.abs`, `.contains`, `.is_some`, `.map`, `.max`, `.min`, `Err`, `Ok`, `PageSize::Custom`, `Some`, `i64::from`
- **Macros:** `format!`, `matches!`
- **Called by:** `Transaction::resolve_page_setup` (edit/sections.rs:363) (?); `tests::named_pages_follow_the_orientation` (edit/sections.rs:539) (?); `tests::custom_pages_keep_their_sides_unless_an_orientation_is_given` (edit/sections.rs:570) (?); `tests::margins_must_leave_text_room_and_stay_in_range` (edit/sections.rs:586) (?)

#### `sect_rank`  (L293-298)
`fn sect_rank(local: &str) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.iter`, `.len`, `.position`, `.unwrap_or`
- **Called by:** `sect_child` (edit/sections.rs:301)

#### `sect_child`  (L301-316)
`fn sect_child(dom: &mut Dom, sect: NodeId, local: &str) -> NodeId`

- **Does:** `sect`'s child `local`, created at its schema position when absent.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L302; `Dom::new_element` (xmllinq/mod.rs:478) L305; `sect_rank` (edit/sections.rs:293) L306,309; `Dom::elements` (xmllinq/mod.rs:640) L307; `Dom::name` (xmllinq/mod.rs:526) L308; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L309; `XName::local_name` (xmllinq/mod.rs:208) (by name) L309; `Dom::add_after_self` (xmllinq/mod.rs:1131) L312; `Dom::add_first` (xmllinq/mod.rs:1085) L313
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.is_some_and`, `.rev`, `W::name`
- **Called by:** `write_geometry` (edit/sections.rs:319)

#### `write_geometry`  (L319-343, pub(super))
`pub(super) fn write_geometry(dom: &mut Dom, sect: NodeId, geometry: &Geometry)`

- **Does:** Write `geometry` into `sect`'s `w:pgSz` and `w:pgMar`.
- **Calls (crate):** `sect_child` (edit/sections.rs:301) L320,331; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L321,322,323,329,341
- **Calls (std/external/unresolved):** `.then_some`, `.to_string`, `Some`, `W::name`
- **Called by:** `Transaction::apply_page_setup` (edit/sections.rs:394); `tests::section_children_go_in_schema_order` (edit/sections.rs:621)

#### `Transaction::sections`  (L353-360)
`fn sections(&self) -> Vec<Option<NodeId>>`

- **Does:** The body's section properties in document order, the final one last (`None` when the body has no final `w:sectPr`).
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) (by name) L356; `mid_sections` (edit/sections.rs:414) L357
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`, `.push`, `W::sect_pr`
- **Called by:** `Transaction::resolve_page_setup` (edit/sections.rs:363)

#### `Transaction::resolve_page_setup`  (L363-390, pub(super))
`pub(super) fn resolve_page_setup( &self, scope: SectionScope, page: Option<PageSize>, orientation: Option<Orientation>, margins: Margins, outcome: &mut EditOutcome, ) -> Result<Targets, (String, String)>`

- **Does:** Resolve a `page_setup`: each target section with its new geometry.
- **Calls (crate):** `Transaction::sections` (edit/sections.rs:353) L377; `Geometry::changed` (edit/sections.rs:183) (by name) L384; `Geometry::of` (edit/sections.rs:151) L384
- **Calls (std/external/unresolved):** `.into`, `.is_none`, `.len`, `.map_err`, `.push`, `.split_off`, `.to_string`, `Err`, `Margins::default`, `Ok`, `Vec::new`
- **Called by:** `Transaction::resolve` (edit.rs:2375)

#### `Transaction::apply_page_setup`  (L394-409, pub(super))
`pub(super) fn apply_page_setup(&mut self, targets: &Targets)`

- **Does:** Apply resolved page setups; a missing final `w:sectPr` is appended to the body.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L398; `Dom::new_element` (xmllinq/mod.rs:478) L401; `Dom::add` (xmllinq/mod.rs:1058) L402; `write_geometry` (edit/sections.rs:319) L407
- **Calls (std/external/unresolved):** `W::sect_pr`
- **Called by:** `Transaction::apply` (edit.rs:4489)

#### `mid_sections`  (L414-426)
`fn mid_sections(dom: &Dom, body: NodeId) -> Vec<NodeId>`

- **Does:** Mid-document `w:sectPr`s under `body`, in document order: the final one and recorded old properties inside revision marks are left out.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L415; `Dom::descendants` (xmllinq/mod.rs:668) L416; `Dom::ancestors` (xmllinq/mod.rs:770) L420; `Dom::name` (xmllinq/mod.rs:526) L421; `XName::local_name` (xmllinq/mod.rs:208) (by name) L422
- **Calls (std/external/unresolved):** `.all`, `.collect`, `.filter`, `.into_iter`, `.is_some_and`, `Some`, `W::sect_pr`
- **Macros:** `matches!`
- **Called by:** `Transaction::sections` (edit/sections.rs:353); `record_mid_changes` (edit/sections.rs:453); `mid_section_authored_history_boundary_tests::changed_mid_section_records_only_authored_base_properties_and_preserves_relationship_owners` (edit/sections.rs:733)

#### `geometry_signature`  (L429-446)
`fn geometry_signature(dom: &Dom, sect: NodeId) -> Vec<(String, Vec<(String, String)>)>`

- **Does:** `w:pgSz` and `w:pgMar` of a section, attributes sorted, for comparison.
- **Calls (crate):** `Dom::element` (xmllinq/mod.rs:655) L433; `Dom::attributes` (xmllinq/mod.rs:885) L436; `XName::clark` (xmllinq/mod.rs:226) (by name) L438
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.iter`, `.map`, `.sort`, `.to_string`, `.unwrap_or_default`, `W::name`
- **Called by:** `record_mid_changes` (edit/sections.rs:453); `mid_section_authored_history_boundary_tests::changed_mid_section_records_only_authored_base_properties_and_preserves_relationship_owners` (edit/sections.rs:733)

#### `record_mid_changes`  (L453-522, pub(super))
`pub(super) fn record_mid_changes( redline: &[u8], base: &[u8], author: &str, date: &str, ) -> Result<Vec<u8>, EditError>`

- **Does:** The comparer records old section properties for the final section only.
- **Calls (crate):** `Opened::open` (inspect.rs:525) L459; `err` (edit.rs:1401) L459,521; `open` (ambiguous: `open` (append.rs:316); `open` (changes.rs:391)) L460,461; `mid_sections` (edit/sections.rs:414) L462,463; `Dom::descendants` (xmllinq/mod.rs:668) (by name) L467; `Dom::attribute` (xmllinq/mod.rs:874) (by name) L471; `Dom::element` (xmllinq/mod.rs:655) (by name) L476; `geometry_signature` (edit/sections.rs:429) L477; `Dom::new_element` (xmllinq/mod.rs:478) (by name) L481,488; `Dom::set_attribute_value` (xmllinq/mod.rs:961) (by name) L482,485,487; `.elements` (ambiguous: `Build::elements` (debug/diff.rs:495); `Element::elements` (markdown/from_docx/ooxml.rs:301); `Dom::elements` (xmllinq/mod.rs:640)) L489,503; `XName::local_name` (xmllinq/mod.rs:208) (by name) L492; `Dom::serialize_element` (xmllinq/mod.rs:1299) (by name) L497; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) (by name) L498; `.root` (ambiguous: `Dom::root` (xmllinq/mod.rs:822); `Scope::root` (xmllinq/serialize.rs:211)) L502; `.add` (ambiguous, 5 candidates) L504,509,510; `Dom::serialize_document` (xmllinq/mod.rs:1322) (by name) L516; `PartFs::set_part` (opc/mod.rs:510) (by name) L518; `PartFs::to_zip` (opc/mod.rs:548) (by name) L519
- **Calls (std/external/unresolved):** `.clone`, `.filter_map`, `.into_bytes`, `.into_iter`, `.is_some_and`, `.is_some`, `.iter`, `.len`, `.map_err`, `.map_or`, `.max`, `.name`, `.ok`, `.parse`, `.to_string`, `.to_vec`, `.zip`, `Ok`, `Some`, `W::author`, `W::date`, `W::id`, `W::name`, `W::sect_pr`
- **Macros:** `format!`, `matches!`
- **Called by:** `mid_section_authored_history_boundary_tests::unchanged_and_already_recorded_mid_sections_preserve_exact_package_and_attribution` (edit/sections.rs:704); `mid_section_authored_history_boundary_tests::changed_mid_section_records_only_authored_base_properties_and_preserves_relationship_owners` (edit/sections.rs:733); `apply_plan` (edit.rs:1621)

#### `targets_overlap`  (L525-527, pub(super))
`pub(super) fn targets_overlap(a: &Targets, b: &Targets) -> bool`

- **Does:** Whether two page setups change a section in common.
- **Calls (std/external/unresolved):** `.any`, `.iter`
- **Called by:** `Transaction::check_conflicts` (edit.rs:4044)

## Test functions (8)

- `tests::letter` (L534): _no doc comment_
- `tests::named_pages_follow_the_orientation` (L539): _no doc comment_
- `tests::custom_pages_keep_their_sides_unless_an_orientation_is_given` (L570): _no doc comment_
- `tests::margins_must_leave_text_room_and_stay_in_range` (L586): _no doc comment_
- `tests::section_children_go_in_schema_order` (L621): _no doc comment_
- `mid_section_authored_history_boundary_tests::package` (L647): _no doc comment_
- `mid_section_authored_history_boundary_tests::unchanged_and_already_recorded_mid_sections_preserve_exact_package_and_attribution` (L704): _no doc comment_
- `mid_section_authored_history_boundary_tests::changed_mid_section_records_only_authored_base_properties_and_preserves_relationship_owners` (L733): _no doc comment_
