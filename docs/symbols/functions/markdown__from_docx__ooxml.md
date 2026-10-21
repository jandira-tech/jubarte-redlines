# Functions and methods in `src/markdown/from_docx/ooxml.rs`

78 non-test, 19 test. Part of the [symbol manifest](../README.md).

#### `invalid`  (L31-33, pub(crate))
`pub(crate) fn invalid(message: impl std::fmt::Display) -> ConvertError`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.to_string`, `ConvertError::Invalid`
- **Called by:** `declared_entry_count` (admission.rs:571) (?); `check_xml_depth` (admission.rs:858) (?); `picture` (edit/images.rs:86) (?); `load_stories` (edit.rs:1680) (?); `check_operations` (edit.rs:2072) (?); `convert_marks` (markdown/from_docx/mod.rs:108); `Package::open` (markdown/from_docx/ooxml.rs:43); `Package::check_tag_policy` (markdown/from_docx/ooxml.rs:61); `Package::read` (markdown/from_docx/ooxml.rs:105); `Package::xml` (markdown/from_docx/ooxml.rs:132); `check_limits` (markdown/from_docx/ooxml.rs:191); `apply_markdown` (markdown/patch.rs:121) (?)

#### `Package::open`  (L43-57, pub(crate))
`pub(crate) fn open(bytes: &'a [u8], kind: &str) -> Result<Self, ConvertError>`

- **Does:** _no doc comment_
- **Calls (crate):** `invalid` (markdown/from_docx/ooxml.rs:31) L45; `check_limits` (markdown/from_docx/ooxml.rs:191) L46; `<InputLimits as Default>::default` (admission.rs:64) L55
- **Calls (std/external/unresolved):** `.collect`, `.file_names`, `.into_owned`, `.map_err`, `.map`, `Cursor::new`, `Ok`, `crate::opc::open_archive`
- **Macros:** `format!`
- **Called by:** `diff` (debug/diff.rs:60) (?); `story_diagnostic_contract_tests::package` (debug/render.rs:1264) (?); `document_text_view` (debug.rs:2181) (?); `list` (debug.rs:2509) (?); `findings` (debug.rs:2575) (?); `report` (debug.rs:2655) (?); `tests::hybrid_packages_translate_their_strict_subparts` (debug.rs:3986) (?); `residual_byte_diagnostic_contract_tests::malformed_xml_byte_parts_report_exact_findings_and_selected_filter_is_respected` (debug.rs:4389) (?); `convert_marks` (markdown/from_docx/mod.rs:108) (?); `creator_created` (markdown/from_docx/mod.rs:1386) (?); `ooxml_package_boundary_tests::media_relationships_extract_owned_bytes_once_and_leave_non_images_as_alt_text` (markdown/from_docx/ooxml.rs:1602) (?); `ooxml_package_boundary_tests::media_part_limit_retains_the_first_512_authored_targets_in_relationship_order` (markdown/from_docx/ooxml.rs:1666) (?)

#### `Package::check_tag_policy`  (L61-87, pub(crate))
`pub(crate) fn check_tag_policy(&mut self) -> Result<(), ConvertError>`

- **Does:** Policy refusals are fatal even for auxiliary parts the renderer would silently skip.
- **Calls (crate):** `.read` (ambiguous: `Package::read` (audit.rs:354); `Package::read` (markdown/from_docx/ooxml.rs:105)) L66; `decode` (markdown/from_docx/ooxml.rs:527) L69; `TagPolicy::tolerant` (xmllinq/parse.rs:261) L71; `TagPolicy::open` (xmllinq/parse.rs:273) L74; `TagPolicy::empty` (xmllinq/parse.rs:295) L75; `TagPolicy::close` (xmllinq/parse.rs:306) L77; `invalid` (markdown/from_docx/ooxml.rs:31) L83
- **Calls (std/external/unresolved):** `.clone`, `.ends_with`, `.map_err`, `.read_event`, `Ok`, `Reader::from_str`
- **Macros:** `format!`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `Package::resolve_name`  (L89-98)
`fn resolve_name(&self, name: &str) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.any`, `.cloned`, `.eq_ignore_ascii_case`, `.find`, `.iter`, `.to_string`, `.trim_start_matches`, `Some`
- **Called by:** `Package::has` (markdown/from_docx/ooxml.rs:100); `Package::read` (markdown/from_docx/ooxml.rs:105)

#### `Package::has`  (L100-102, pub(crate))
`pub(crate) fn has(&self, name: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Package::resolve_name` (markdown/from_docx/ooxml.rs:89) L101
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `Package::main_part` (markdown/from_docx/ooxml.rs:179)

#### `Package::read`  (L105-129, pub(crate))
`pub(crate) fn read(&mut self, name: &str) -> Result<Option<Vec<u8>>, ConvertError>`

- **Does:** Read one entry, or `None` when it does not exist.
- **Calls (crate):** `Package::resolve_name` (markdown/from_docx/ooxml.rs:89) L106; `invalid` (markdown/from_docx/ooxml.rs:31) L112,114,122,124
- **Calls (std/external/unresolved):** `.by_name`, `.len`, `.map_err`, `.min`, `.read_to_end`, `.size`, `.take`, `Err`, `Ok`, `Some`, `Vec::with_capacity`
- **Macros:** `format!`
- **Called by:** `audit_report_with` (audit.rs:177) (?); `Deps::dep_for` (markdown/from_docx/drawing.rs:592) (?); `Package::check_tag_policy` (markdown/from_docx/ooxml.rs:61) (?); `Package::xml` (markdown/from_docx/ooxml.rs:132) (?); `Media::load` (markdown/from_docx/ooxml.rs:1185) (?)

#### `Package::xml`  (L132-139, pub(crate))
`pub(crate) fn xml(&mut self, name: &str) -> Result<Option<Element>, ConvertError>`

- **Does:** Read and parse an XML entry, or `None` when it does not exist.
- **Calls (crate):** `.read` (ambiguous: `Package::read` (audit.rs:354); `Package::read` (markdown/from_docx/ooxml.rs:105)) L133; `parse_xml_with_limits` (markdown/from_docx/ooxml.rs:444) L134; `invalid` (markdown/from_docx/ooxml.rs:31) L136
- **Calls (std/external/unresolved):** `.map_err`, `.map`, `Ok`
- **Macros:** `format!`
- **Called by:** `xml` (markdown/borders.rs:254) (?); `content_types` (markdown/from_docx/drawing.rs:891); `convert_marks` (markdown/from_docx/mod.rs:108); `creator_created` (markdown/from_docx/mod.rs:1386) (?); `Package::rels` (markdown/from_docx/ooxml.rs:142)

#### `Package::rels`  (L142-176, pub(crate))
`pub(crate) fn rels(&mut self, part: &str) -> Result<Rels, ConvertError>`

- **Does:** Relationships of a part (`dir/_rels/file.rels`), keyed by id.
- **Calls (crate):** `split_part` (markdown/from_docx/ooxml.rs:203) L143; `Package::xml` (markdown/from_docx/ooxml.rs:132) L150; `Element::children_named` (markdown/from_docx/ooxml.rs:379) (by name) L153; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L154,157,168; `resolve_target` (markdown/from_docx/ooxml.rs:212) L163
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`, `.insert`, `.is_empty`, `.is_some_and`, `.push`, `.to_string`, `.unwrap_or_default`, `Ok`, `Rels::default`
- **Macros:** `format!`
- **Called by:** `Deps::dep_for` (markdown/from_docx/drawing.rs:592); `resolve` (markdown/from_docx/drawing.rs:704); `convert_marks` (markdown/from_docx/mod.rs:108); `Package::main_part` (markdown/from_docx/ooxml.rs:179); `ooxml_package_boundary_tests::media_relationships_extract_owned_bytes_once_and_leave_non_images_as_alt_text` (markdown/from_docx/ooxml.rs:1602) (?); `ooxml_package_boundary_tests::media_part_limit_retains_the_first_512_authored_targets_in_relationship_order` (markdown/from_docx/ooxml.rs:1666) (?)

#### `Package::main_part`  (L179-187, pub(crate))
`pub(crate) fn main_part(&mut self, fallback: &str) -> Result<String, ConvertError>`

- **Does:** The main part named by the package-level officeDocument relationship.
- **Calls (crate):** `Package::rels` (markdown/from_docx/ooxml.rs:142) L180; `Rels::first_of_type` (markdown/from_docx/ooxml.rs:249) (by name) L181; `Package::has` (markdown/from_docx/ooxml.rs:100) L185
- **Calls (std/external/unresolved):** `.clone`, `.filter`, `.map`, `.to_string`, `.unwrap_or_else`, `Ok`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108)

#### `check_limits`  (L191-201, pub(crate))
`pub(crate) fn check_limits<R: Read + std::io::Seek>( archive: &zip::ZipArchive<R>, ) -> Result<(), ConvertError>`

- **Does:** Reject containers that declare too many entries or oversized members.
- **Calls (crate):** `invalid` (markdown/from_docx/ooxml.rs:31) L195
- **Calls (std/external/unresolved):** `.len`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `Package::open` (markdown/from_docx/ooxml.rs:43)

#### `split_part`  (L203-209)
`fn split_part(part: &str) -> (&str, &str)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.rfind`, `.trim_start_matches`
- **Called by:** `Package::rels` (markdown/from_docx/ooxml.rs:142)

#### `resolve_target`  (L212-229)
`fn resolve_target(dir: &str, target: &str) -> String`

- **Does:** Resolve a relationship target relative to the source part's directory.
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.is_empty`, `.join`, `.next`, `.pop`, `.push`, `.split`, `.starts_with`, `.unwrap_or_default`, `Vec::new`
- **Called by:** `Package::rels` (markdown/from_docx/ooxml.rs:142); `tests::resolves_relative_targets` (markdown/from_docx/ooxml.rs:1414)

#### `Rels::get`  (L245-247, pub(crate))
`pub(crate) fn get(&self, id: &str) -> Option<&Rel>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.get`
- **Called by:** `symbolize` (markdown/from_docx/drawing.rs:660); `Writer::drawing` (markdown/from_docx/mod.rs:3572); `Writer::drawing_text` (markdown/from_docx/mod.rs:3670); `Rels::link` (markdown/from_docx/ooxml.rs:278)

#### `Rels::first_of_type`  (L249-254, pub(crate))
`pub(crate) fn first_of_type(&self, suffix: &str) -> Option<&Rel>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.ends_with`, `.filter_map`, `.find`, `.get`, `.iter`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108) (?); `Package::main_part` (markdown/from_docx/ooxml.rs:179) (?)

#### `Rels::targets_of_type`  (L258-265, pub(crate))
`pub(crate) fn targets_of_type(&self, suffix: &str) -> Vec<String>`

- **Does:** The package-internal targets of every relationship of a type, in document order.
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.ends_with`, `.filter_map`, `.filter`, `.get`, `.iter`, `.map`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108) (?)

#### `Rels::len`  (L268-270, pub(crate))
`pub(crate) fn len(&self) -> usize`

- **Does:** How many relationships the part holds.
- **Calls (std/external/unresolved):** `.len`
- **Called by:** none found in src

#### `Rels::is_empty`  (L273-275, pub(crate))
`pub(crate) fn is_empty(&self) -> bool`

- **Does:** Whether the part holds no relationships.
- **Calls (std/external/unresolved):** `.is_empty`
- **Called by:** none found in src

#### `Rels::link`  (L278-282, pub(crate))
`pub(crate) fn link(&self, id: &str) -> Option<String>`

- **Does:** A hyperlink target for a relationship id: external URLs only.
- **Calls (crate):** `Rels::get` (markdown/from_docx/ooxml.rs:245) L279
- **Calls (std/external/unresolved):** `.clone`, `.filter`, `.is_empty`, `.map`
- **Called by:** `Writer::inline` (markdown/from_docx/mod.rs:3218); `ooxml_package_boundary_tests::media_relationships_extract_owned_bytes_once_and_leave_non_images_as_alt_text` (markdown/from_docx/ooxml.rs:1602) (?)

#### `titled`  (L290-295, pub(crate))
`pub(crate) fn titled(url: &str, title: Option<&str>) -> String`

- **Does:** A link destination carrying `title` (a tooltip), if it has one.
- **Calls (std/external/unresolved):** `.filter`, `.is_empty`, `.to_string`
- **Macros:** `format!`
- **Called by:** `Writer::inline` (markdown/from_docx/mod.rs:3218); `hyperlink_instruction` (markdown/from_docx/mod.rs:4642)

#### `local`  (L313-315, pub(crate))
`pub(crate) fn local(name: &str) -> &str`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.next`, `.rsplit`, `.unwrap_or`
- **Called by:** `normalize_ids` (markdown/from_docx/drawing.rs:565); `canonical::{fn names}` (markdown/from_docx/drawing.rs:1098); `regenerable` (markdown/from_docx/drawing.rs:1143); `Element::local` (markdown/from_docx/ooxml.rs:318); `Element::attr` (markdown/from_docx/ooxml.rs:327); `Element::rel_attr` (markdown/from_docx/ooxml.rs:365)

#### `Element::local`  (L318-320, pub(crate))
`pub(crate) fn local(&self) -> &str`

- **Does:** _no doc comment_
- **Calls (crate):** `local` (markdown/from_docx/ooxml.rs:313) L319
- **Called by:** `Build::row` (debug/diff.rs:1107) (?); `Build::sdt` (debug/diff.rs:1136) (?); `Build::note` (debug/diff.rs:1165) (?); `plain` (markdown/data_stores.rs:309) (?); `hollow` (markdown/from_docx/agent.rs:130); `silent_run` (markdown/from_docx/agent.rs:139); `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168) (?); `revision_tags` (markdown/from_docx/agent.rs:563) (?); `revision_tags::{fn flatten}` (markdown/from_docx/agent.rs:569) (?); `mark_tags_by` (markdown/from_docx/agent.rs:695) (?); `stamp_in` (markdown/from_docx/agent.rs:759); `revised_by` (markdown/from_docx/agent.rs:851) (?); +60 more

#### `Element::is`  (L322-324, pub(crate))
`pub(crate) fn is(&self, name: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::local` (markdown/from_docx/ooxml.rs:318) L323
- **Called by:** `stores` (markdown/data_stores.rs:155) (?); `bindings` (markdown/data_stores.rs:191) (?); `unbound` (markdown/data_stores.rs:510) (?); `format_change_tags_by::{fn walk}` (markdown/from_docx/agent.rs:726) (?); `stamp_in` (markdown/from_docx/agent.rs:759); `shown_children` (markdown/from_docx/agent.rs:883); `comment_ids::{fn walk}` (markdown/from_docx/agent.rs:1559) (?); `holds` (markdown/from_docx/agent.rs:1682) (?); `count` (markdown/from_docx/agent.rs:1688) (?); `tracked_page_break::{fn walk}` (markdown/from_docx/agent.rs:1698) (?); `is_page_break` (markdown/from_docx/agent.rs:1723); `has_rendered_page_break` (markdown/from_docx/agent.rs:1731) (?); +43 more

#### `Element::attr`  (L327-332, pub(crate))
`pub(crate) fn attr(&self, name: &str) -> Option<&str>`

- **Does:** Attribute by local name (any prefix).
- **Calls (crate):** `local` (markdown/from_docx/ooxml.rs:313) L330
- **Calls (std/external/unresolved):** `.as_str`, `.find`, `.iter`, `.map`
- **Called by:** `stores` (markdown/data_stores.rs:155) (?); `bindings` (markdown/data_stores.rs:191) (?); `shown` (markdown/data_stores.rs:266) (?); `unbound` (markdown/data_stores.rs:510) (?); `silent_run` (markdown/from_docx/agent.rs:139) (?); `hidden_run` (markdown/from_docx/agent.rs:157) (?); `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:168) (?); `tag_of` (markdown/from_docx/agent.rs:194); `required_date_tag_of` (markdown/from_docx/agent.rs:202); `plain_tag_of` (markdown/from_docx/agent.rs:216); `revision_tags` (markdown/from_docx/agent.rs:563) (?); `stamped_revs` (markdown/from_docx/agent.rs:835); +94 more

#### `Element::attr_ns`  (L335-345)
`fn attr_ns(&self, namespace: &str, name: &str) -> Option<&str>`

- **Does:** Attribute by namespace URI and local name.
- **Calls (std/external/unresolved):** `.as_str`, `.find_map`, `.get`, `.is_some_and`, `.iter`, `.split_once`, `.then_some`
- **Called by:** `Element::revision_date` (markdown/from_docx/ooxml.rs:354)

#### `Element::comment_date`  (L350-352, pub(crate))
`pub(crate) fn comment_date(&self) -> Option<&str>`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::attr` (markdown/from_docx/ooxml.rs:327) L351
- **Calls (std/external/unresolved):** `.or_else`
- **Called by:** `handles` (markdown/from_docx/agent.rs:968) (?); `Writer::note` (markdown/from_docx/mod.rs:4068) (?); `Writer::agent_head` (markdown/from_docx/mod.rs:4099)

#### `Element::revision_date`  (L354-362, pub(crate))
`pub(crate) fn revision_date(&self) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (crate):** `revision_date` (changes.rs:1082) L355; `Element::attr` (markdown/from_docx/ooxml.rs:327) L357; `Element::attr_ns` (markdown/from_docx/ooxml.rs:335) L359
- **Called by:** `required_date_tag_of` (markdown/from_docx/agent.rs:202); `revision_tags` (markdown/from_docx/agent.rs:563) (?); `walk_revision_authors` (markdown/from_docx/agent.rs:936); `attribution` (markdown/from_docx/mod.rs:2337); `Writer::inline` (markdown/from_docx/mod.rs:3218) (?); `Writer::agent_tag` (markdown/from_docx/mod.rs:4017)

#### `Element::rel_attr`  (L365-370, pub(crate))
`pub(crate) fn rel_attr(&self, name: &str) -> Option<&str>`

- **Does:** A relationship reference (`r:id`, `r:embed`): a prefixed attribute with this local name.
- **Calls (crate):** `local` (markdown/from_docx/ooxml.rs:313) L368
- **Calls (std/external/unresolved):** `.as_str`, `.contains`, `.find`, `.iter`, `.map`
- **Called by:** `regenerable` (markdown/from_docx/drawing.rs:1143) (?); `Writer::inline` (markdown/from_docx/mod.rs:3218) (?); `Writer::drawing` (markdown/from_docx/mod.rs:3572) (?); `Writer::drawing_text` (markdown/from_docx/mod.rs:3670) (?)

#### `Element::elements`  (L372-377, pub(crate))
`pub(crate) fn elements(&self) -> impl Iterator<Item = &Element>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.filter_map`, `.iter`, `Some`
- **Called by:** `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1599) (?); `Package::table_no_header_row` (audit.rs:586) (?); `CommentFamily::read_meta` (comments.rs:485) (?); `CommentFamily::adopt` (comments.rs:743) (?); `final_section_table_ownership_tests::a_table_only_story_keeps_its_final_section_outside_all_cell_paragraphs` (comparer/atomize.rs:747) (?); `final_section_table_ownership_tests::block_control_paragraph_still_owns_the_closing_section_instead_of_later_table_cells` (comparer/atomize.rs:771) (?); `final_section_table_ownership_tests::existing_cell_section_properties_and_history_are_never_replaced_by_body_geometry` (comparer/atomize.rs:794) (?); `coverage_atomizer_owned_boundaries::note_definition_owns_every_character_and_mark_but_the_part_is_a_path_stop` (comparer/atomize.rs:827) (?); `coverage_atomizer_field_chain_boundaries::header_and_footer_roots_stop_paths_and_preserve_each_whitespace_boundary` (comparer/atomize.rs:1123) (?); `deeper_boundary_tests::namespace_conflicts_reuse_aliases_or_skip_occupied_generated_prefixes` (comparer/comments.rs:2317) (?); `public_option_source_ownership_tests::source_events::{fn walk}` (comparer/cross_para.rs:2512) (?); `coverage_tests::content_controls_unwrap_deepest_first_and_keep_text_order` (comparer/finalize.rs:15897) (?); +217 more

#### `Element::children_named`  (L379-384, pub(crate))
`pub(crate) fn children_named<'s>( &'s self, name: &'s str, ) -> impl Iterator<Item = &'s Element> + 's`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L383; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L383
- **Calls (std/external/unresolved):** `.filter`
- **Called by:** `shown` (markdown/data_stores.rs:266) (?); `geometry_clauses` (markdown/from_docx/agent.rs:2286) (?); `table_line` (markdown/from_docx/agent.rs:2378) (?); `threads` (markdown/from_docx/agent.rs:2693) (?); `style_by_id` (markdown/from_docx/header.rs:785); `paragraph_style_sizes` (markdown/from_docx/header.rs:921) (?); `section_changes` (markdown/from_docx/header.rs:1424) (?); `convert_marks` (markdown/from_docx/mod.rs:108) (?); `creator_created` (markdown/from_docx/mod.rs:1386) (?); `Styles::parse` (markdown/from_docx/mod.rs:1809); `Numbering::parse` (markdown/from_docx/mod.rs:2053); `Writer::inline` (markdown/from_docx/mod.rs:3218) (?); +5 more

#### `Element::child`  (L386-388, pub(crate))
`pub(crate) fn child(&self, name: &str) -> Option<&Element>`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L387; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L387
- **Calls (std/external/unresolved):** `.find`
- **Called by:** `shown` (markdown/data_stores.rs:266); `silent_run` (markdown/from_docx/agent.rs:139); `hidden_run` (markdown/from_docx/agent.rs:157) (?); `revision_tags::{fn flatten}` (markdown/from_docx/agent.rs:569) (?); `stamp_in` (markdown/from_docx/agent.rs:759); `shown_children` (markdown/from_docx/agent.rs:883); `own_size::{fn walk}` (markdown/from_docx/agent.rs:1132) (?); `page_sections` (markdown/from_docx/agent.rs:1943); `id_line` (markdown/from_docx/agent.rs:1994); `geometry_clauses` (markdown/from_docx/agent.rs:2286); `table_line` (markdown/from_docx/agent.rs:2378); `tests::stamps_paragraphs_in_document_order_cells_included_text_boxes_excluded` (markdown/from_docx/agent.rs:3457) (?); +49 more

#### `Element::path`  (L391-393, pub(crate))
`pub(crate) fn path(&self, names: &[&str]) -> Option<&Element>`

- **Does:** Follow a path of local names through first-matching children.
- **Calls (crate):** `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:386); `Scope::child` (xmllinq/serialize.rs:225)) L392
- **Calls (std/external/unresolved):** `.iter`, `.try_fold`
- **Called by:** `tests::an_unreadable_bundle_is_foreign` (bin/jubarte.rs:4494) (?); `tests::bundle_restore_failure` (bin/jubarte.rs:4550) (?); `tests::debug_opens_a_dual_header_package_instead_of_refusing` (bin/jubarte.rs:4638) (?); `tests::convert_rejects_a_font_report_path_equal_to_the_pdf` (bin/jubarte.rs:5194) (?); `tests::a_legacy_doc_is_refused_with_a_save_as_hint` (bin/jubarte.rs:5275) (?); `tests::fail_on_substitution_is_exit_4_after_the_outputs` (bin/jubarte.rs:5315) (?); `tests::convert_font_report_writes_json` (bin/jubarte.rs:5355) (?); `tests::convert_pages_writes_the_selected_pages_named_by_page_number` (bin/jubarte.rs:5585) (?); `tests::convert_pages_needs_png_and_an_existing_page` (bin/jubarte.rs:5609) (?); `tests::diff_render_writes_only_the_changed_pages_and_diff_json` (bin/jubarte.rs:5637) (?); `tests::diff_render_of_equal_documents_writes_only_diff_json` (bin/jubarte.rs:5693) (?); `tests::diff_render_without_an_overlay_writes_no_diff_png` (bin/jubarte.rs:5704) (?); +49 more

#### `Element::find`  (L396-401, pub(crate))
`pub(crate) fn find(&self, name: &str) -> Option<&Element>`

- **Does:** First descendant (depth-first, including self) with this local name.
- **Calls (crate):** `Element::is` (markdown/from_docx/ooxml.rs:322) L397; `Element::elements` (markdown/from_docx/ooxml.rs:372) L400
- **Calls (std/external/unresolved):** `.find_map`, `.find`, `Some`
- **Called by:** `label` (markdown/from_docx/drawing.rs:399); `alt` (markdown/from_docx/drawing.rs:414); `media_slot` (markdown/from_docx/drawing.rs:790); `is_plain_inline` (markdown/from_docx/drawing.rs:1491); `title` (markdown/from_docx/mermaid.rs:68); `Writer::drawing` (markdown/from_docx/mod.rs:3572); `Writer::drawing_text` (markdown/from_docx/mod.rs:3670)

#### `Element::find_all`  (L404-412, pub(crate))
`pub(crate) fn find_all<'s>(&'s self, name: &str, out: &mut Vec<&'s Element>)`

- **Does:** All descendants with this local name, not descending into matches.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:372) L405; `Element::is` (markdown/from_docx/ooxml.rs:322) (by name) L406; `Element::find_all` (markdown/from_docx/ooxml.rs:404) (by name) L409
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `table_line` (markdown/from_docx/agent.rs:2378) (?); `nested_pairs` (markdown/from_docx/drawing.rs:347); `Drawing::nested_tokens` (markdown/from_docx/drawing.rs:1557) (?); `fill_boxes::{fn walk}` (markdown/from_docx/drawing.rs:1728) (?); `series` (markdown/from_docx/mermaid.rs:18); `title` (markdown/from_docx/mermaid.rs:68) (?); `convert_marks` (markdown/from_docx/mod.rs:108) (?); `Writer::box_blocks` (markdown/from_docx/mod.rs:3636) (?); `Writer::cell_styles` (markdown/from_docx/mod.rs:3754) (?); `Writer::cell_spacings` (markdown/from_docx/mod.rs:3807) (?); `story_color` (markdown/from_docx/mod.rs:4914); `Element::find_all` (markdown/from_docx/ooxml.rs:404) (?)

#### `Element::text`  (L415-419, pub(crate))
`pub(crate) fn text(&self) -> String`

- **Does:** Concatenated character data of this subtree.
- **Calls (crate):** `Element::collect_text` (markdown/from_docx/ooxml.rs:421) L417
- **Calls (std/external/unresolved):** `String::new`
- **Called by:** `strip_box_words::{fn strip}` (markdown/from_docx/drawing.rs:834); `strip_box_words::{fn walk}` (markdown/from_docx/drawing.rs:856); `fill_boxes::{fn words}` (markdown/from_docx/drawing.rs:1688); `story_text_with::{fn run_content}` (markdown/from_docx/header.rs:1923); `Writer::run_child` (markdown/from_docx/mod.rs:3437)

#### `Element::collect_text`  (L421-428)
`fn collect_text(&self, out: &mut String)`

- **Does:** _no doc comment_
- **Calls (crate):** `.collect_text` (ambiguous: `Element::collect_text` (markdown/from_docx/ooxml.rs:421); `Dom::collect_text` (xmllinq/mod.rs:1207)) L425
- **Calls (std/external/unresolved):** `.push_str`
- **Called by:** `Element::text` (markdown/from_docx/ooxml.rs:415); `Element::collect_text` (markdown/from_docx/ooxml.rs:421) (?)

#### `Element::toggle`  (L431-434, pub(crate))
`pub(crate) fn toggle(&self, name: &str) -> Option<bool>`

- **Does:** Whether a toggle property element (`<w:b/>`, `<w:b w:val="0"/>`) is on.
- **Calls (crate):** `Element::child` (markdown/from_docx/ooxml.rs:386) L432; `Element::attr` (markdown/from_docx/ooxml.rs:327) (by name) L433
- **Calls (std/external/unresolved):** `.map`, `Some`
- **Macros:** `matches!`
- **Called by:** `hidden_run` (markdown/from_docx/agent.rs:157) (?); `Styles::parse` (markdown/from_docx/mod.rs:1809) (?); `Writer::run` (markdown/from_docx/mod.rs:3324) (?); `document_defaults` (markdown/view_styles.rs:832) (?)

#### `parse_xml`  (L440-442, pub(crate))
`pub(crate) fn parse_xml(bytes: &[u8]) -> Result<Element, String>`

- **Does:** Parse XML bytes into an element tree (the document element).
- **Calls (crate):** `parse_xml_with_limits` (markdown/from_docx/ooxml.rs:444) L441; `<InputLimits as Default>::default` (admission.rs:64) L441
- **Called by:** `write_missing` (markdown/data_stores.rs:63) (?); `stores` (markdown/data_stores.rs:155) (?); `tests::a_control_s_store_value_is_its_text_only_where_word_keeps_it_so` (markdown/data_stores.rs:663) (?); `tests::body` (markdown/from_docx/agent.rs:3438); `tests::doc` (markdown/from_docx/agent.rs:3442); `tests::handles_prefer_comment_initials_disambiguate_collisions_and_collect_timestamps` (markdown/from_docx/agent.rs:3710); `tests::a_story_substitution_keeps_each_inline_date_when_written_back` (markdown/from_docx/critic.rs:2219); `Pending::attach::{fn transfer}` (markdown/from_docx/drawing.rs:281); `canonical` (markdown/from_docx/drawing.rs:1093); `regenerable` (markdown/from_docx/drawing.rs:1143); `Drawing::box_count` (markdown/from_docx/drawing.rs:1550); `Drawing::nested_tokens` (markdown/from_docx/drawing.rs:1557); +22 more

#### `parse_xml_with_limits`  (L444-525, pub(crate))
`pub(crate) fn parse_xml_with_limits(bytes: &[u8], limits: InputLimits) -> Result<Element, String>`

- **Does:** _no doc comment_
- **Calls (crate):** `TagPolicy::new` (xmllinq/parse.rs:247) L445; `decode` (markdown/from_docx/ooxml.rs:527) L446; `TagPolicy::open` (xmllinq/parse.rs:273) L456; `element_from` (markdown/from_docx/ooxml.rs:552) L462,472; `TagPolicy::empty` (xmllinq/parse.rs:295) L468; `push_child` (markdown/from_docx/ooxml.rs:581) L473,481,515; `TagPolicy::close` (xmllinq/parse.rs:306) L476; `push_text` (markdown/from_docx/ooxml.rs:587) L485,489,506
- **Calls (std/external/unresolved):** `.buffer_position`, `.config_mut`, `.expect`, `.find_map`, `.into_iter`, `.into`, `.last`, `.len`, `.map_err`, `.ok_or_else`, `.pop`, `.push`, `.read_event`, `.resolve_char_ref`, `.to_string`, `.trim_text`, `.unwrap_or_default`, `.xml_content`, `Err`, `Node::Element`, `Reader::from_str`, `Some`
- **Macros:** `format!`, `vec!`
- **Called by:** `Deps::dep_for` (markdown/from_docx/drawing.rs:592); `Package::xml` (markdown/from_docx/ooxml.rs:132); `parse_xml` (markdown/from_docx/ooxml.rs:440); `tests::issue650_converter_tree_preserves_attribute_width_with_raised_limits` (markdown/from_docx/ooxml.rs:1497); `tests::issue650_converter_tree_preserves_long_names_with_raised_limits` (markdown/from_docx/ooxml.rs:1522)

#### `decode`  (L527-548)
`fn decode(bytes: &[u8]) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_chunks`, `.collect`, `.into_owned`, `.iter`, `.map`, `.strip_prefix`, `.unwrap_or`, `String::from_utf16_lossy`, `String::from_utf8_lossy`, `u16::from_be_bytes`, `u16::from_le_bytes`
- **Called by:** `Package::check_tag_policy` (markdown/from_docx/ooxml.rs:61); `parse_xml_with_limits` (markdown/from_docx/ooxml.rs:444)

#### `element_from`  (L552-579, pub(crate))
`pub(crate) fn element_from( start: &quick_xml::events::BytesStart<'_>, inherited: &Arc<HashMap<String, String>>, ) -> Result<Element, String>`

- **Does:** An element, childless, from its start tag: its name, its attributes as the parser normalizes them, and the namespaces in scope.
- **Calls (std/external/unresolved):** `.as_ref`, `.attributes`, `.clone`, `.insert`, `.into_owned`, `.map_err`, `.name`, `.normalized_value`, `.push`, `.strip_prefix`, `.to_string`, `.with_checks`, `Arc::clone`, `Arc::make_mut`, `Ok`, `Vec::new`
- **Called by:** `unbound` (markdown/data_stores.rs:510) (?); `parse_xml_with_limits` (markdown/from_docx/ooxml.rs:444)

#### `push_child`  (L581-585)
`fn push_child(stack: &mut [Element], node: Node)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.last_mut`, `.push`
- **Called by:** `parse_xml_with_limits` (markdown/from_docx/ooxml.rs:444)

#### `push_text`  (L587-598)
`fn push_text(stack: &mut [Element], text: &str)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_empty`, `.last_mut`, `.push_str`, `.push`, `.to_string`, `Node::Text`
- **Called by:** `parse_xml_with_limits` (markdown/from_docx/ooxml.rs:444)

#### `collapse_ws`  (L600-602, pub(crate))
`pub(crate) fn collapse_ws(s: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.join`, `.split_whitespace`
- **Called by:** `image_markdown` (markdown/from_docx/ooxml.rs:1148)

#### `Inline::keep_spaces`  (L643-645, pub(crate))
`pub(crate) fn keep_spaces(&mut self)`

- **Does:** Agent view: keeps runs of spaces as the file holds them.
- **Calls:** none
- **Called by:** `Critic::into_inline` (markdown/from_docx/critic.rs:512); `ooxml_package_boundary_tests::an_agent_line_keeps_its_leading_spaces_as_character_references` (markdown/from_docx/ooxml.rs:1743)

#### `Inline::verbatim`  (L648-651, pub(crate))
`pub(crate) fn verbatim(&mut self)`

- **Does:** A code block: keeps every space, a line's indent included.
- **Calls:** none
- **Called by:** `Critic::into_inline` (markdown/from_docx/critic.rs:512); `ooxml_package_boundary_tests::an_agent_line_keeps_its_leading_spaces_as_character_references` (markdown/from_docx/ooxml.rs:1743)

#### `Inline::render_code`  (L656-671, pub(crate))
`pub(crate) fn render_code(&self) -> String`

- **Does:** A code block's paragraph: every mark dropped, for a fence's content is literal and formatting printed inside it would come back as text.
- **Calls (crate):** `Inline::render` (markdown/from_docx/ooxml.rs:715) L670
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.iter`, `.map`, `Marks::default`
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2706) (?)

#### `Inline::edges`  (L675-690, pub(crate))
`pub(crate) fn edges(&self) -> (bool, bool)`

- **Does:** Whether the text starts and ends with a space.
- **Calls (std/external/unresolved):** `.as_str`, `.ends_with`, `.filter`, `.is_empty`, `.is_some_and`, `.iter`, `.map`, `.next`, `.rfind`, `.starts_with`
- **Macros:** `matches!`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108) (?); `Writer::paragraph` (markdown/from_docx/mod.rs:2706) (?); `Writer::cell_parts` (markdown/from_docx/mod.rs:3942) (?); `Writer::note_text` (markdown/from_docx/mod.rs:4137) (?)

#### `Inline::push_styled`  (L694-710, pub(crate))
`pub(crate) fn push_styled(&mut self, text: &str, marks: Marks, link: Option<&str>)`

- **Does:** Appends text with its marks, merging into the last span when they and the link match.
- **Calls (std/external/unresolved):** `.as_deref`, `.is_empty`, `.last_mut`, `.map`, `.push_str`, `.push`, `.to_string`
- **Called by:** `Critic::into_inline` (markdown/from_docx/critic.rs:512); `Inline::push` (markdown/from_docx/ooxml.rs:1403); `ooxml_package_boundary_tests::a_plain_space_between_code_spans_stays_between_them` (markdown/from_docx/ooxml.rs:1766)

#### `Inline::render`  (L715-754, pub(crate))
`pub(crate) fn render(&self, emphasis: bool) -> String`

- **Does:** Render to Markdown.
- **Calls (crate):** `render_emphasis` (markdown/from_docx/ooxml.rs:806) L727,749; `split_ws` (markdown/from_docx/ooxml.rs:1030) L728; `link_destination` (markdown/from_docx/ooxml.rs:761) L731; `email_autolink` (markdown/from_docx/ooxml.rs:777) L740; `tidy_inline` (markdown/from_docx/ooxml.rs:1042) L753
- **Calls (std/external/unresolved):** `.all`, `.as_str`, `.clone`, `.is_empty`, `.iter`, `.len`, `.push_str`, `.replace`, `.split_once`, `.strip_prefix`, `.trim`, `.unwrap_or`, `Some`, `String::new`
- **Macros:** `format!`
- **Called by:** `Inline::render_code` (markdown/from_docx/ooxml.rs:656)

#### `link_destination`  (L761-772)
`fn link_destination(url: &str) -> String`

- **Does:** Spaces become `%20`.
- **Calls (std/external/unresolved):** `.contains`, `.replace`
- **Macros:** `format!`
- **Called by:** `Inline::render` (markdown/from_docx/ooxml.rs:715)

#### `email_autolink`  (L777-783)
`fn email_autolink(address: &str) -> bool`

- **Does:** Whether `address` is the inside of a CommonMark email autolink, the addresses pulldown-cmark reads (`<local@domain>`).
- **Calls (crate):** `local_part` (markdown/from_docx/ooxml.rs:785) L782; `domain` (markdown/from_docx/ooxml.rs:794) L782
- **Calls (std/external/unresolved):** `.as_bytes`, `.iter`, `.position`
- **Called by:** `Inline::render` (markdown/from_docx/ooxml.rs:715)

#### `local_part`  (L785-792)
`fn local_part(bytes: &[u8]) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.all`, `.contains`, `.is_ascii_alphanumeric`, `.is_empty`, `.iter`
- **Called by:** `email_autolink` (markdown/from_docx/ooxml.rs:777)

#### `domain`  (L794-804)
`fn domain(bytes: &[u8]) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.all`, `.contains`, `.ends_with`, `.is_ascii_alphanumeric`, `.is_empty`, `.iter`, `.len`, `.split`, `.starts_with`
- **Called by:** `email_autolink` (markdown/from_docx/ooxml.rs:777)

#### `render_emphasis`  (L806-954)
`fn render_emphasis(spans: &[Span], emphasis: bool) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `edges` (markdown/from_docx/ooxml.rs:960) L849,858; `pair` (ambiguous, 5 candidates) L877; `delimiters_read` (markdown/from_docx/ooxml.rs:995) L893,907,909; `opens_empty_emphasis` (markdown/from_docx/ooxml.rs:978) L919; `code_fence` (markdown/from_docx/ooxml.rs:1016) L939; `split_ws` (markdown/from_docx/ooxml.rs:1030) L942
- **Calls (std/external/unresolved):** `.as_ref`, `.chars`, `.clone`, `.collect`, `.copied`, `.enumerate`, `.flatten`, `.get`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.join`, `.last_mut`, `.last`, `.map_or`, `.map`, `.next`, `.or`, `.push_str`, `.push`, `.split`, `.to_string`, `.trim`, `Marks::default`, `Some`, `String::new`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `Inline::render` (markdown/from_docx/ooxml.rs:715)

#### `edges`  (L960-974)
`fn edges(marks: Marks, text: &str) -> (&str, std::borrow::Cow<'_, str>, &str)`

- **Does:** `text` as a stretch with `marks` prints it: the spaces before, the text inside the marks, the spaces after.
- **Calls (crate):** `split_ws` (markdown/from_docx/ooxml.rs:1030) L964
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.is_some`, `.len`, `.repeat`, `space`, `std::borrow::Cow::Borrowed`, `std::borrow::Cow::Owned`
- **Macros:** `format!`
- **Called by:** `render_emphasis` (markdown/from_docx/ooxml.rs:806)

#### `opens_empty_emphasis`  (L978-988)
`fn opens_empty_emphasis(out: &str, open: &str) -> bool`

- **Does:** `open` (`**`, `_`, `**_`) against the marks `out` already ends with.
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.next`, `.rev`, `.take_while`
- **Called by:** `render_emphasis` (markdown/from_docx/ooxml.rs:806)

#### `delimiters_read`  (L995-1011)
`fn delimiters_read( before: Option<char>, inner: &str, after: Option<char>, underscore: bool, ) -> bool`

- **Does:** Whether emphasis delimiters around `inner`, between `before` and `after` (None: the line's edge), open and close as CommonMark reads them: an opener before punctuation needs space or punctuation before it, a closer after punctuation the same after it; `_` (`underscore`) also opens or closes nowhere inside a word.
- **Calls (crate):** `word` (ambiguous, 5 candidates) L1010
- **Calls (std/external/unresolved):** `.chars`, `.is_alphanumeric`, `.is_ascii_punctuation`, `.is_none_or`, `.is_some_and`, `.is_whitespace`, `.last`, `.next`, `open_side`, `punctuation`
- **Called by:** `render_emphasis` (markdown/from_docx/ooxml.rs:806)

#### `code_fence`  (L1016-1028)
`fn code_fence(text: &str) -> (String, String)`

- **Does:** The backticks around a code span's text: one more than its longest run of backticks, padded when an edge backtick or space needs protecting.
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.ends_with`, `.map`, `.max`, `.repeat`, `.split`, `.starts_with`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `render_emphasis` (markdown/from_docx/ooxml.rs:806)

#### `split_ws`  (L1030-1034)
`fn split_ws(s: &str) -> (&str, &str, &str)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.len`, `.max`, `.trim_end`, `.trim_start`
- **Called by:** `Inline::render` (markdown/from_docx/ooxml.rs:715); `render_emphasis` (markdown/from_docx/ooxml.rs:806); `edges` (markdown/from_docx/ooxml.rs:960)

#### `tidy_inline`  (L1042-1102)
`fn tidy_inline(s: &str, keep_spaces: bool, verbatim: bool) -> String`

- **Does:** Collapse runs of spaces and trim each line.
- **Calls (crate):** `parser_options` (markdown/write.rs:1626) L1053; `equation_split` (markdown/from_docx/ooxml.rs:1106) L1069; `finish_line` (markdown/from_docx/ooxml.rs:1126) L1093
- **Calls (std/external/unresolved):** `.chain`, `.chars`, `.clone`, `.collect`, `.contains`, `.filter_map`, `.first`, `.into_iter`, `.into_offset_iter`, `.is_empty`, `.is_some_and`, `.join`, `.last`, `.len`, `.pop`, `.push_str`, `.push`, `.remove`, `.split`, `.then_some`, `String::with_capacity`, `Vec::new`, `pulldown_cmark::Event::Code`, `pulldown_cmark::Parser::new_ext`, `std::iter::once`
- **Macros:** `matches!`
- **Called by:** `Inline::render` (markdown/from_docx/ooxml.rs:715); `tests::inline_code_keeps_spaces_while_surrounding_text_collapses` (markdown/from_docx/ooxml.rs:1425); `tests::an_equation_keeps_its_spaces_where_text_collapses` (markdown/from_docx/ooxml.rs:1440)

#### `equation_split`  (L1106-1123)
`fn equation_split(line: &str) -> (&str, &str, &str)`

- **Does:** `line` as the text before its first closed equation, the equation with its fences, and the rest; all text when it holds none.
- **Calls (crate):** `unescaped_dollar` (markdown/from_docx/mod.rs:1499) L1107; `math_body` (markdown/from_docx/mod.rs:1515) L1116
- **Calls (std/external/unresolved):** `.len`, `.starts_with`
- **Called by:** `tidy_inline` (markdown/from_docx/ooxml.rs:1042)

#### `finish_line`  (L1126-1145)
`fn finish_line(line: &str, keep_spaces: bool, verbatim: bool) -> String`

- **Does:** One line of [`tidy_inline`].
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.is_empty`, `.len`, `.push_str`, `.take_while`, `.to_string`, `.trim_end`, `.trim_start`, `.trim`, `String::with_capacity`
- **Called by:** `tidy_inline` (markdown/from_docx/ooxml.rs:1042)

#### `image_markdown`  (L1148-1168, pub(crate))
`pub(crate) fn image_markdown(alt: &str, target: Option<&str>) -> Option<String>`

- **Does:** Markdown for an image with alt text; images without alt text are dropped.
- **Calls (crate):** `collapse_ws` (markdown/from_docx/ooxml.rs:600) L1149
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.is_empty`, `.next`, `.replace`, `.rsplit`, `.unwrap_or`, `Some`
- **Macros:** `format!`
- **Called by:** `Media::markdown` (markdown/from_docx/ooxml.rs:1228)

#### `Media::load`  (L1185-1217, pub(crate))
`pub(crate) fn load( package: &mut Package<'_>, rels: &Rels, extracted: Option<Extracted>, ) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `.read` (ambiguous: `Package::read` (audit.rs:354); `Package::read` (markdown/from_docx/ooxml.rs:105)) L1205
- **Calls (std/external/unresolved):** `.clone`, `.contains_key`, `.ends_with`, `.get`, `.insert`, `.len`, `HashMap::new`, `Self::default`, `Some`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108); `ooxml_package_boundary_tests::media_relationships_extract_owned_bytes_once_and_leave_non_images_as_alt_text` (markdown/from_docx/ooxml.rs:1602); `ooxml_package_boundary_tests::media_part_limit_retains_the_first_512_authored_targets_in_relationship_order` (markdown/from_docx/ooxml.rs:1666)

#### `Media::extracted_markdown`  (L1221-1224, pub(crate))
`pub(crate) fn extracted_markdown(&mut self, alt: &str, target: Option<&str>) -> Option<String>`

- **Does:** Markdown only when the picture's raster is extracted to a file; `None` when pictures are not extracted or this one is no raster.
- **Calls (crate):** `.add` (ambiguous, 5 candidates) L1223
- **Calls (std/external/unresolved):** `.and_then`, `.as_mut`, `.get`
- **Called by:** `Writer::drawing` (markdown/from_docx/mod.rs:3572)

#### `Media::markdown`  (L1228-1236, pub(crate))
`pub(crate) fn markdown(&mut self, alt: &str, target: Option<&str>) -> Option<String>`

- **Does:** Markdown for a picture: the extracted file when pictures are extracted and this one is a raster, else the alt-text form.
- **Calls (crate):** `.add` (ambiguous, 5 candidates) L1231; `image_markdown` (markdown/from_docx/ooxml.rs:1148) L1235
- **Calls (std/external/unresolved):** `.and_then`, `.as_mut`, `.get`, `Some`
- **Called by:** `Writer::drawing_text` (markdown/from_docx/mod.rs:3670); `ooxml_package_boundary_tests::media_relationships_extract_owned_bytes_once_and_leave_non_images_as_alt_text` (markdown/from_docx/ooxml.rs:1602) (?)

#### `Media::into_files`  (L1239-1241, pub(crate))
`pub(crate) fn into_files(self) -> std::collections::BTreeMap<String, Vec<u8>>`

- **Does:** The pictures written, by file name.
- **Calls (std/external/unresolved):** `.map`, `.unwrap_or_default`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108); `ooxml_package_boundary_tests::media_relationships_extract_owned_bytes_once_and_leave_non_images_as_alt_text` (markdown/from_docx/ooxml.rs:1602); `ooxml_package_boundary_tests::media_part_limit_retains_the_first_512_authored_targets_in_relationship_order` (markdown/from_docx/ooxml.rs:1666) (?)

#### `Blocks::new`  (L1258-1266, pub(crate))
`pub(crate) fn new() -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `String::new`
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108); `Writer::box_blocks` (markdown/from_docx/mod.rs:3636)

#### `Blocks::push`  (L1270-1273, pub(crate))
`pub(crate) fn push(&mut self, block: &str, is_list: bool)`

- **Does:** A block whose start cannot carry a CriticMarkup delimiter (a table, a text box); a tracked break before it is left unmarked.
- **Calls (crate):** `Blocks::push_prefixed` (markdown/from_docx/ooxml.rs:1278) L1272
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108); `Writer::blocks_in` (markdown/from_docx/mod.rs:2553); `Writer::paragraph` (markdown/from_docx/mod.rs:2706)

#### `Blocks::push_prefixed`  (L1278-1280, pub(crate))
`pub(crate) fn push_prefixed(&mut self, prefix: &str, body: &str, is_list: bool)`

- **Does:** A block made of Markdown syntax (`## `, ` 1.
- **Calls (crate):** `Blocks::push_paragraph` (markdown/from_docx/ooxml.rs:1297) L1279
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2553); `Blocks::push` (markdown/from_docx/ooxml.rs:1270)

#### `Blocks::push_line`  (L1284-1293, pub(crate))
`pub(crate) fn push_line(&mut self, line: &str)`

- **Does:** An id, table or page line of the agent view.
- **Calls (std/external/unresolved):** `.is_empty`, `.push_str`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2553); `Writer::paragraph` (markdown/from_docx/mod.rs:2706); `Writer::flush_empty` (markdown/from_docx/mod.rs:2966); `Writer::page_lines` (markdown/from_docx/mod.rs:2980)

#### `Blocks::push_paragraph`  (L1297-1331, pub(crate))
`pub(crate) fn push_paragraph( &mut self, prefix: &str, body: &str, is_list: bool, (leading, trailing): (bool, bool), )`

- **Does:** A paragraph, with whether its text started and ended with a space before rendering trimmed it (see [`critic::splice`]).
- **Calls (crate):** `splice` (markdown/from_docx/critic.rs:1344) L1319
- **Calls (std/external/unresolved):** `.as_ref`, `.is_empty`, `.push_str`, `.take`, `.trim_end`, `.trim`
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2706); `Blocks::push_prefixed` (markdown/from_docx/ooxml.rs:1278)

#### `Blocks::empty_paragraph`  (L1335-1343, pub(crate))
`pub(crate) fn empty_paragraph(&mut self)`

- **Does:** Keeps an untracked empty paragraph after an explicitly inserted paragraph mark.
- **Calls (crate):** `attribution_tag` (markdown/critic.rs:1330) L1338
- **Calls (std/external/unresolved):** `.is_some_and`, `.len`, `.push_str`, `.rsplit_once`, `.trim_end`
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2706)

#### `Blocks::set_separator`  (L1347-1349, pub(crate))
`pub(crate) fn set_separator(&mut self, change: Option<Change>)`

- **Does:** Records that the paragraph mark ending the last block was inserted or deleted (`None` clears it).
- **Calls:** none
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2706)

#### `Blocks::finish`  (L1351-1353, pub(crate))
`pub(crate) fn finish(self) -> String`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `convert_marks` (markdown/from_docx/mod.rs:108); `Writer::box_blocks` (markdown/from_docx/mod.rs:3636)

#### `ListIndent::reset`  (L1366-1368, pub(crate))
`pub(crate) fn reset(&mut self)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clear`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2553); `Writer::paragraph` (markdown/from_docx/mod.rs:2706); `tests::list_indent_nests_under_content` (markdown/from_docx/ooxml.rs:1561); `ooxml_package_boundary_tests::literal_hyperlinks_and_mail_addresses_use_exact_autolink_forms` (markdown/from_docx/ooxml.rs:1701)

#### `ListIndent::place`  (L1373-1378, pub(crate))
`pub(crate) fn place(&self, level: usize) -> (usize, usize)`

- **Does:** Where an item at `level` goes: its indentation, and the level that indentation says (the items it nests under; less than `level` when a level between is skipped).
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.iter`, `.len`, `.map`, `.min`, `.sum`
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2706)

#### `ListIndent::item`  (L1381-1394, pub(crate))
`pub(crate) fn item(&mut self, level: usize, marker: &str, text: &str) -> String`

- **Does:** Format an item at `level` with `marker` (e.g.
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.iter`, `.len`, `.map`, `.min`, `.push`, `.repeat`, `.replace`, `.sum`, `.truncate`
- **Macros:** `format!`
- **Called by:** `Build::segments` (debug/diff.rs:931) (?); `Build::run` (debug/diff.rs:1000) (?); `boundary_coverage_tests::toggle_normalization_distinguishes_boolean_values_from_numeric_properties` (debug/diff.rs:2269) (?); `boundary_coverage_tests::relationship_attributes_resolve_targets_without_inventing_missing_targets` (debug/diff.rs:2300) (?); `tests::list_tables_are_read_within_their_fib_ranges` (legacy_doc.rs:2454) (?); `Writer::paragraph` (markdown/from_docx/mod.rs:2706); `tests::list_indent_nests_under_content` (markdown/from_docx/ooxml.rs:1561) (?); `ooxml_package_boundary_tests::literal_hyperlinks_and_mail_addresses_use_exact_autolink_forms` (markdown/from_docx/ooxml.rs:1701) (?)

## Test functions (19)

- `Inline::push` (L1403): _no doc comment_
- `tests::resolves_relative_targets` (L1414): _no doc comment_
- `tests::inline_code_keeps_spaces_while_surrounding_text_collapses` (L1425): _no doc comment_
- `tests::an_equation_keeps_its_spaces_where_text_collapses` (L1440): _no doc comment_
- `tests::bold_closed_against_bold_is_not_empty_emphasis` (L1456): _no doc comment_
- `tests::inline_merges_and_places_markers` (L1482): _no doc comment_
- `tests::issue650_converter_tree_preserves_attribute_width_with_raised_limits` (L1497): _no doc comment_
- `tests::issue650_converter_tree_preserves_long_names_with_raised_limits` (L1522): _no doc comment_
- `tests::issue650_converter_tree_refuses_aliased_duplicate` (L1540): _no doc comment_
- `tests::xml_entities_and_depth` (L1551): _no doc comment_
- `tests::list_indent_nests_under_content` (L1561): _no doc comment_
- `ooxml_package_boundary_tests::package_bytes` (L1582): _no doc comment_
- `ooxml_package_boundary_tests::png` (L1592): _no doc comment_
- `ooxml_package_boundary_tests::media_relationships_extract_owned_bytes_once_and_leave_non_images_as_alt_text` (L1602): _no doc comment_
- `ooxml_package_boundary_tests::media_part_limit_retains_the_first_512_authored_targets_in_relationship_order` (L1666): _no doc comment_
- `ooxml_package_boundary_tests::literal_hyperlinks_and_mail_addresses_use_exact_autolink_forms` (L1701): _no doc comment_
- `ooxml_package_boundary_tests::an_agent_line_keeps_its_leading_spaces_as_character_references` (L1743): _no doc comment_
- `ooxml_package_boundary_tests::a_plain_space_between_code_spans_stays_between_them` (L1766): _no doc comment_
- `ooxml_package_boundary_tests::utf16_xml_keeps_namespaced_attributes_and_authored_text` (L1785): _no doc comment_
