# Functions and methods in `src/markdown/from_docx/ooxml.rs`

72 non-test, 15 test. Part of the [symbol manifest](../README.md).

#### `invalid`  (L27-29, pub(crate))
`pub(crate) fn invalid(message: impl std::fmt::Display) -> ConvertError`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.to_string`, `ConvertError::Invalid`
- **Called by:** `declared_entry_count` (admission.rs:470) (?); `check_xml_depth` (admission.rs:678) (?); `picture` (edit/images.rs:86) (?); `load_stories` (edit.rs:1311) (?); `convert` (markdown/from_docx/mod.rs:96); `Package::open` (markdown/from_docx/ooxml.rs:38); `Package::read` (markdown/from_docx/ooxml.rs:66); `Package::xml` (markdown/from_docx/ooxml.rs:93); `check_limits` (markdown/from_docx/ooxml.rs:152); `apply_markdown` (markdown/patch.rs:121) (?)

#### `Package::open`  (L38-48, pub(crate))
`pub(crate) fn open(bytes: &'a [u8], kind: &str) -> Result<Self, ConvertError>`

- **Does:** _no doc comment_
- **Calls (crate):** `invalid` (markdown/from_docx/ooxml.rs:27) L40; `check_limits` (markdown/from_docx/ooxml.rs:152) L41
- **Calls (std/external/unresolved):** `.collect`, `.file_names`, `.into_owned`, `.map_err`, `.map`, `Cursor::new`, `Ok`, `zip::ZipArchive::new`
- **Macros:** `format!`
- **Called by:** `diff` (debug/diff.rs:60) (?); `story_diagnostic_contract_tests::package` (debug/render.rs:1267) (?); `document_text_view` (debug.rs:2011) (?); `list` (debug.rs:2331) (?); `findings` (debug.rs:2397) (?); `report` (debug.rs:2457) (?); `tests::hybrid_packages_translate_their_strict_subparts` (debug.rs:3485) (?); `residual_byte_diagnostic_contract_tests::malformed_xml_byte_parts_report_exact_findings_and_selected_filter_is_respected` (debug.rs:3888) (?); `convert` (markdown/from_docx/mod.rs:96) (?); `creator_created` (markdown/from_docx/mod.rs:1302) (?); `ooxml_package_boundary_tests::media_relationships_extract_owned_bytes_once_and_leave_non_images_as_alt_text` (markdown/from_docx/ooxml.rs:1388) (?); `ooxml_package_boundary_tests::media_part_limit_retains_the_first_512_authored_targets_in_relationship_order` (markdown/from_docx/ooxml.rs:1452) (?)

#### `Package::resolve_name`  (L50-59)
`fn resolve_name(&self, name: &str) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.any`, `.cloned`, `.eq_ignore_ascii_case`, `.find`, `.iter`, `.to_string`, `.trim_start_matches`, `Some`
- **Called by:** `Package::has` (markdown/from_docx/ooxml.rs:61); `Package::read` (markdown/from_docx/ooxml.rs:66)

#### `Package::has`  (L61-63, pub(crate))
`pub(crate) fn has(&self, name: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Package::resolve_name` (markdown/from_docx/ooxml.rs:50) L62
- **Calls (std/external/unresolved):** `.is_some`
- **Called by:** `Package::main_part` (markdown/from_docx/ooxml.rs:140)

#### `Package::read`  (L66-90, pub(crate))
`pub(crate) fn read(&mut self, name: &str) -> Result<Option<Vec<u8>>, ConvertError>`

- **Does:** Read one entry, or `None` when it does not exist.
- **Calls (crate):** `Package::resolve_name` (markdown/from_docx/ooxml.rs:50) L67; `invalid` (markdown/from_docx/ooxml.rs:27) L73,75,83,85
- **Calls (std/external/unresolved):** `.by_name`, `.len`, `.map_err`, `.min`, `.read_to_end`, `.size`, `.take`, `Err`, `Ok`, `Some`, `Vec::with_capacity`
- **Macros:** `format!`
- **Called by:** `audit_report_with` (audit.rs:177) (?); `Deps::dep_for` (markdown/from_docx/drawing.rs:438) (?); `Package::xml` (markdown/from_docx/ooxml.rs:93) (?); `Media::load` (markdown/from_docx/ooxml.rs:1052) (?)

#### `Package::xml`  (L93-100, pub(crate))
`pub(crate) fn xml(&mut self, name: &str) -> Result<Option<Element>, ConvertError>`

- **Does:** Read and parse an XML entry, or `None` when it does not exist.
- **Calls (crate):** `.read` (ambiguous: `Package::read` (audit.rs:354); `Package::read` (markdown/from_docx/ooxml.rs:66)) L94; `parse_xml` (markdown/from_docx/ooxml.rs:367) L95; `invalid` (markdown/from_docx/ooxml.rs:27) L97
- **Calls (std/external/unresolved):** `.map_err`, `.map`, `Ok`
- **Macros:** `format!`
- **Called by:** `xml` (markdown/borders.rs:254) (?); `content_types` (markdown/from_docx/drawing.rs:665); `convert` (markdown/from_docx/mod.rs:96); `creator_created` (markdown/from_docx/mod.rs:1302) (?); `Package::rels` (markdown/from_docx/ooxml.rs:103)

#### `Package::rels`  (L103-137, pub(crate))
`pub(crate) fn rels(&mut self, part: &str) -> Result<Rels, ConvertError>`

- **Does:** Relationships of a part (`dir/_rels/file.rels`), keyed by id.
- **Calls (crate):** `split_part` (markdown/from_docx/ooxml.rs:164) L104; `Package::xml` (markdown/from_docx/ooxml.rs:93) L111; `Element::children_named` (markdown/from_docx/ooxml.rs:308) (by name) L114; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L115,118,129; `resolve_target` (markdown/from_docx/ooxml.rs:173) L124
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`, `.insert`, `.is_empty`, `.is_some_and`, `.push`, `.to_string`, `.unwrap_or_default`, `Ok`, `Rels::default`
- **Macros:** `format!`
- **Called by:** `Deps::dep_for` (markdown/from_docx/drawing.rs:438); `resolve` (markdown/from_docx/drawing.rs:548); `convert` (markdown/from_docx/mod.rs:96); `Package::main_part` (markdown/from_docx/ooxml.rs:140); `ooxml_package_boundary_tests::media_relationships_extract_owned_bytes_once_and_leave_non_images_as_alt_text` (markdown/from_docx/ooxml.rs:1388) (?); `ooxml_package_boundary_tests::media_part_limit_retains_the_first_512_authored_targets_in_relationship_order` (markdown/from_docx/ooxml.rs:1452) (?)

#### `Package::main_part`  (L140-148, pub(crate))
`pub(crate) fn main_part(&mut self, fallback: &str) -> Result<String, ConvertError>`

- **Does:** The main part named by the package-level officeDocument relationship.
- **Calls (crate):** `Package::rels` (markdown/from_docx/ooxml.rs:103) L141; `Rels::first_of_type` (markdown/from_docx/ooxml.rs:210) (by name) L142; `Package::has` (markdown/from_docx/ooxml.rs:61) L146
- **Calls (std/external/unresolved):** `.clone`, `.filter`, `.map`, `.to_string`, `.unwrap_or_else`, `Ok`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96)

#### `check_limits`  (L152-162, pub(crate))
`pub(crate) fn check_limits<R: Read + std::io::Seek>( archive: &zip::ZipArchive<R>, ) -> Result<(), ConvertError>`

- **Does:** Reject containers that declare too many entries or oversized members.
- **Calls (crate):** `invalid` (markdown/from_docx/ooxml.rs:27) L156
- **Calls (std/external/unresolved):** `.len`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `Package::open` (markdown/from_docx/ooxml.rs:38)

#### `split_part`  (L164-170)
`fn split_part(part: &str) -> (&str, &str)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.rfind`, `.trim_start_matches`
- **Called by:** `Package::rels` (markdown/from_docx/ooxml.rs:103)

#### `resolve_target`  (L173-190)
`fn resolve_target(dir: &str, target: &str) -> String`

- **Does:** Resolve a relationship target relative to the source part's directory.
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.is_empty`, `.join`, `.next`, `.pop`, `.push`, `.split`, `.starts_with`, `.unwrap_or_default`, `Vec::new`
- **Called by:** `Package::rels` (markdown/from_docx/ooxml.rs:103); `tests::resolves_relative_targets` (markdown/from_docx/ooxml.rs:1269)

#### `Rels::get`  (L206-208, pub(crate))
`pub(crate) fn get(&self, id: &str) -> Option<&Rel>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.get`
- **Called by:** `symbolize` (markdown/from_docx/drawing.rs:504); `Writer::drawing` (markdown/from_docx/mod.rs:3366); `Writer::drawing_text` (markdown/from_docx/mod.rs:3464); `Rels::link` (markdown/from_docx/ooxml.rs:239)

#### `Rels::first_of_type`  (L210-215, pub(crate))
`pub(crate) fn first_of_type(&self, suffix: &str) -> Option<&Rel>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.ends_with`, `.filter_map`, `.find`, `.get`, `.iter`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96) (?); `Package::main_part` (markdown/from_docx/ooxml.rs:140) (?)

#### `Rels::targets_of_type`  (L219-226, pub(crate))
`pub(crate) fn targets_of_type(&self, suffix: &str) -> Vec<String>`

- **Does:** The package-internal targets of every relationship of a type, in document order.
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.ends_with`, `.filter_map`, `.filter`, `.get`, `.iter`, `.map`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96) (?)

#### `Rels::len`  (L229-231, pub(crate))
`pub(crate) fn len(&self) -> usize`

- **Does:** How many relationships the part holds.
- **Calls (std/external/unresolved):** `.len`
- **Called by:** none found in src

#### `Rels::is_empty`  (L234-236, pub(crate))
`pub(crate) fn is_empty(&self) -> bool`

- **Does:** Whether the part holds no relationships.
- **Calls (std/external/unresolved):** `.is_empty`
- **Called by:** none found in src

#### `Rels::link`  (L239-243, pub(crate))
`pub(crate) fn link(&self, id: &str) -> Option<String>`

- **Does:** A hyperlink target for a relationship id: external URLs only.
- **Calls (crate):** `Rels::get` (markdown/from_docx/ooxml.rs:206) L240
- **Calls (std/external/unresolved):** `.clone`, `.filter`, `.is_empty`, `.map`
- **Called by:** `Writer::inline` (markdown/from_docx/mod.rs:3021); `ooxml_package_boundary_tests::media_relationships_extract_owned_bytes_once_and_leave_non_images_as_alt_text` (markdown/from_docx/ooxml.rs:1388) (?)

#### `titled`  (L251-256, pub(crate))
`pub(crate) fn titled(url: &str, title: Option<&str>) -> String`

- **Does:** A link destination carrying `title` (a tooltip), if it has one.
- **Calls (std/external/unresolved):** `.filter`, `.is_empty`, `.to_string`
- **Macros:** `format!`
- **Called by:** `Writer::inline` (markdown/from_docx/mod.rs:3021); `hyperlink_instruction` (markdown/from_docx/mod.rs:4372)

#### `local`  (L272-274, pub(crate))
`pub(crate) fn local(name: &str) -> &str`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.next`, `.rsplit`, `.unwrap_or`
- **Called by:** `normalize_ids` (markdown/from_docx/drawing.rs:411); `canonical::{fn names}` (markdown/from_docx/drawing.rs:872); `regenerable` (markdown/from_docx/drawing.rs:917); `Element::local` (markdown/from_docx/ooxml.rs:277); `Element::attr` (markdown/from_docx/ooxml.rs:286); `Element::rel_attr` (markdown/from_docx/ooxml.rs:294)

#### `Element::local`  (L277-279, pub(crate))
`pub(crate) fn local(&self) -> &str`

- **Does:** _no doc comment_
- **Calls (crate):** `local` (markdown/from_docx/ooxml.rs:272) L278
- **Called by:** `Build::row` (debug/diff.rs:1107) (?); `Build::sdt` (debug/diff.rs:1136) (?); `Build::note` (debug/diff.rs:1165) (?); `hollow` (markdown/from_docx/agent.rs:106); `silent_run` (markdown/from_docx/agent.rs:115); `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:144) (?); `revision_tags` (markdown/from_docx/agent.rs:334) (?); `revision_tags::{fn flatten}` (markdown/from_docx/agent.rs:340) (?); `mark_tags` (markdown/from_docx/agent.rs:468) (?); `stamp_in` (markdown/from_docx/agent.rs:515); `revised_by` (markdown/from_docx/agent.rs:587) (?); `comment_anchors` (markdown/from_docx/agent.rs:605) (?); +51 more

#### `Element::is`  (L281-283, pub(crate))
`pub(crate) fn is(&self, name: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::local` (markdown/from_docx/ooxml.rs:277) L282
- **Called by:** `format_change_tags::{fn walk}` (markdown/from_docx/agent.rs:487) (?); `stamp_in` (markdown/from_docx/agent.rs:515); `shown_children` (markdown/from_docx/agent.rs:619); `comment_ids::{fn walk}` (markdown/from_docx/agent.rs:1184) (?); `holds` (markdown/from_docx/agent.rs:1221) (?); `count` (markdown/from_docx/agent.rs:1227) (?); `tracked_page_break::{fn walk}` (markdown/from_docx/agent.rs:1237) (?); `is_page_break` (markdown/from_docx/agent.rs:1262); `has_rendered_page_break` (markdown/from_docx/agent.rs:1270) (?); `rendered_page_breaks` (markdown/from_docx/agent.rs:1281) (?); `tables::{fn walk}` (markdown/from_docx/agent.rs:1430) (?); `paragraphs::{fn walk}` (markdown/from_docx/agent.rs:1445) (?); +30 more

#### `Element::attr`  (L286-291, pub(crate))
`pub(crate) fn attr(&self, name: &str) -> Option<&str>`

- **Does:** Attribute by local name (any prefix).
- **Calls (crate):** `local` (markdown/from_docx/ooxml.rs:272) L289
- **Calls (std/external/unresolved):** `.as_str`, `.find`, `.iter`, `.map`
- **Called by:** `silent_run` (markdown/from_docx/agent.rs:115) (?); `hidden_run` (markdown/from_docx/agent.rs:133) (?); `half_ranges::{fn walk}` (markdown/from_docx/agent.rs:144) (?); `tag_of` (markdown/from_docx/agent.rs:170); `plain_tag_of` (markdown/from_docx/agent.rs:179); `revision_tags` (markdown/from_docx/agent.rs:334) (?); `stamped_revs` (markdown/from_docx/agent.rs:571); `revised_by` (markdown/from_docx/agent.rs:587) (?); `comment_anchors` (markdown/from_docx/agent.rs:605) (?); `walk_revision_authors` (markdown/from_docx/agent.rs:672); `handles` (markdown/from_docx/agent.rs:704) (?); `unshown_tags::{fn walk}` (markdown/from_docx/agent.rs:837) (?); +71 more

#### `Element::rel_attr`  (L294-299, pub(crate))
`pub(crate) fn rel_attr(&self, name: &str) -> Option<&str>`

- **Does:** A relationship reference (`r:id`, `r:embed`): a prefixed attribute with this local name.
- **Calls (crate):** `local` (markdown/from_docx/ooxml.rs:272) L297
- **Calls (std/external/unresolved):** `.as_str`, `.contains`, `.find`, `.iter`, `.map`
- **Called by:** `regenerable` (markdown/from_docx/drawing.rs:917) (?); `Writer::inline` (markdown/from_docx/mod.rs:3021) (?); `Writer::drawing` (markdown/from_docx/mod.rs:3366) (?); `Writer::drawing_text` (markdown/from_docx/mod.rs:3464) (?)

#### `Element::elements`  (L301-306, pub(crate))
`pub(crate) fn elements(&self) -> impl Iterator<Item = &Element>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.filter_map`, `.iter`, `Some`
- **Called by:** `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1594) (?); `Package::table_no_header_row` (audit.rs:586) (?); `CommentFamily::read_meta` (comments.rs:457) (?); `CommentFamily::adopt` (comments.rs:687) (?); `final_section_table_ownership_tests::a_table_only_story_keeps_its_final_section_outside_all_cell_paragraphs` (comparer/atomize.rs:747) (?); `final_section_table_ownership_tests::block_control_paragraph_still_owns_the_closing_section_instead_of_later_table_cells` (comparer/atomize.rs:771) (?); `final_section_table_ownership_tests::existing_cell_section_properties_and_history_are_never_replaced_by_body_geometry` (comparer/atomize.rs:794) (?); `coverage_atomizer_owned_boundaries::note_definition_owns_every_character_and_mark_but_the_part_is_a_path_stop` (comparer/atomize.rs:827) (?); `coverage_atomizer_field_chain_boundaries::header_and_footer_roots_stop_paths_and_preserve_each_whitespace_boundary` (comparer/atomize.rs:1123) (?); `deeper_boundary_tests::namespace_conflicts_reuse_aliases_or_skip_occupied_generated_prefixes` (comparer/comments.rs:2317) (?); `public_option_source_ownership_tests::source_events::{fn walk}` (comparer/cross_para.rs:2513) (?); `coverage_tests::content_controls_unwrap_deepest_first_and_keep_text_order` (comparer/finalize.rs:15875) (?); +204 more

#### `Element::children_named`  (L308-313, pub(crate))
`pub(crate) fn children_named<'s>( &'s self, name: &'s str, ) -> impl Iterator<Item = &'s Element> + 's`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L312; `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L312
- **Calls (std/external/unresolved):** `.filter`
- **Called by:** `table_line` (markdown/from_docx/agent.rs:1812) (?); `threads` (markdown/from_docx/agent.rs:2121) (?); `style_by_id` (markdown/from_docx/header.rs:713); `section_changes` (markdown/from_docx/header.rs:1233) (?); `convert` (markdown/from_docx/mod.rs:96) (?); `creator_created` (markdown/from_docx/mod.rs:1302) (?); `Styles::parse` (markdown/from_docx/mod.rs:1726); `Numbering::parse` (markdown/from_docx/mod.rs:1970); `Writer::inline` (markdown/from_docx/mod.rs:3021) (?); `Writer::column_alignments` (markdown/from_docx/mod.rs:3600) (?); `Writer::note` (markdown/from_docx/mod.rs:3820) (?); `math_inner` (markdown/from_docx/mod.rs:3946); +2 more

#### `Element::child`  (L315-317, pub(crate))
`pub(crate) fn child(&self, name: &str) -> Option<&Element>`

- **Does:** _no doc comment_
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L316; `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L316
- **Calls (std/external/unresolved):** `.find`
- **Called by:** `silent_run` (markdown/from_docx/agent.rs:115); `hidden_run` (markdown/from_docx/agent.rs:133) (?); `revision_tags::{fn flatten}` (markdown/from_docx/agent.rs:340) (?); `stamp_in` (markdown/from_docx/agent.rs:515); `shown_children` (markdown/from_docx/agent.rs:619); `page_sections` (markdown/from_docx/agent.rs:1463); `id_line` (markdown/from_docx/agent.rs:1514); `table_line` (markdown/from_docx/agent.rs:1812); `collect_in` (markdown/from_docx/agent.rs:2182); `tests::stamps_paragraphs_in_document_order_cells_included_text_boxes_excluded` (markdown/from_docx/agent.rs:2786) (?); `tests::a_universal_measure_indent_on_the_id_line_reads_as_twips` (markdown/from_docx/agent.rs:2801) (?); `tests::an_absurd_measure_indent_drops_its_clause` (markdown/from_docx/agent.rs:2829) (?); +41 more

#### `Element::path`  (L320-322, pub(crate))
`pub(crate) fn path(&self, names: &[&str]) -> Option<&Element>`

- **Does:** Follow a path of local names through first-matching children.
- **Calls (crate):** `.child` (ambiguous: `Element::child` (markdown/from_docx/ooxml.rs:315); `Scope::child` (xmllinq/serialize.rs:225)) L321
- **Calls (std/external/unresolved):** `.iter`, `.try_fold`
- **Called by:** `tests::convert_rejects_a_font_report_path_equal_to_the_pdf` (bin/jubarte.rs:3949) (?); `tests::a_legacy_doc_is_refused_with_a_save_as_hint` (bin/jubarte.rs:4030) (?); `tests::fail_on_substitution_is_exit_4_after_the_outputs` (bin/jubarte.rs:4070) (?); `tests::convert_font_report_writes_json` (bin/jubarte.rs:4110) (?); `tests::convert_pages_writes_the_selected_pages_named_by_page_number` (bin/jubarte.rs:4340) (?); `tests::convert_pages_needs_png_and_an_existing_page` (bin/jubarte.rs:4364) (?); `tests::diff_render_writes_only_the_changed_pages_and_diff_json` (bin/jubarte.rs:4392) (?); `tests::diff_render_of_equal_documents_writes_only_diff_json` (bin/jubarte.rs:4448) (?); `tests::diff_render_without_an_overlay_writes_no_diff_png` (bin/jubarte.rs:4459) (?); `tests::refresh_fields_writes_the_report_and_says_when_it_cannot` (bin/jubarte.rs:4503) (?); `sorted_dir_listing` (convert/font.rs:2241) (?); `scan_cjk_family_faces` (convert/font.rs:2866) (?); +32 more

#### `Element::find`  (L325-330, pub(crate))
`pub(crate) fn find(&self, name: &str) -> Option<&Element>`

- **Does:** First descendant (depth-first, including self) with this local name.
- **Calls (crate):** `Element::is` (markdown/from_docx/ooxml.rs:281) L326; `Element::elements` (markdown/from_docx/ooxml.rs:301) L329
- **Calls (std/external/unresolved):** `.find_map`, `.find`, `Some`
- **Called by:** `label` (markdown/from_docx/drawing.rs:245); `alt` (markdown/from_docx/drawing.rs:260); `media_slot` (markdown/from_docx/drawing.rs:622); `is_plain_inline` (markdown/from_docx/drawing.rs:1265); `title` (markdown/from_docx/mermaid.rs:68); `Writer::drawing` (markdown/from_docx/mod.rs:3366); `Writer::drawing_text` (markdown/from_docx/mod.rs:3464)

#### `Element::find_all`  (L333-341, pub(crate))
`pub(crate) fn find_all<'s>(&'s self, name: &str, out: &mut Vec<&'s Element>)`

- **Does:** All descendants with this local name, not descending into matches.
- **Calls (crate):** `Element::elements` (markdown/from_docx/ooxml.rs:301) L334; `Element::is` (markdown/from_docx/ooxml.rs:281) (by name) L335; `Element::find_all` (markdown/from_docx/ooxml.rs:333) (by name) L338
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `table_line` (markdown/from_docx/agent.rs:1812) (?); `series` (markdown/from_docx/mermaid.rs:18); `title` (markdown/from_docx/mermaid.rs:68) (?); `convert` (markdown/from_docx/mod.rs:96) (?); `Writer::box_comments` (markdown/from_docx/mod.rs:3422) (?); `Writer::drawing_text` (markdown/from_docx/mod.rs:3464); `Writer::cell_styles` (markdown/from_docx/mod.rs:3563) (?); `story_color` (markdown/from_docx/mod.rs:4599); `Element::find_all` (markdown/from_docx/ooxml.rs:333) (?)

#### `Element::text`  (L344-348, pub(crate))
`pub(crate) fn text(&self) -> String`

- **Does:** Concatenated character data of this subtree.
- **Calls (crate):** `Element::collect_text` (markdown/from_docx/ooxml.rs:350) L346
- **Calls (std/external/unresolved):** `String::new`
- **Called by:** `story_text_with::{fn run_content}` (markdown/from_docx/header.rs:1736); `Writer::run_child` (markdown/from_docx/mod.rs:3232)

#### `Element::collect_text`  (L350-357)
`fn collect_text(&self, out: &mut String)`

- **Does:** _no doc comment_
- **Calls (crate):** `.collect_text` (ambiguous: `Element::collect_text` (markdown/from_docx/ooxml.rs:350); `Dom::collect_text` (xmllinq/mod.rs:1191)) L354
- **Calls (std/external/unresolved):** `.push_str`
- **Called by:** `Element::text` (markdown/from_docx/ooxml.rs:344); `Element::collect_text` (markdown/from_docx/ooxml.rs:350) (?)

#### `Element::toggle`  (L360-363, pub(crate))
`pub(crate) fn toggle(&self, name: &str) -> Option<bool>`

- **Does:** Whether a toggle property element (`<w:b/>`, `<w:b w:val="0"/>`) is on.
- **Calls (crate):** `Element::child` (markdown/from_docx/ooxml.rs:315) L361; `Element::attr` (markdown/from_docx/ooxml.rs:286) (by name) L362
- **Calls (std/external/unresolved):** `.map`, `Some`
- **Macros:** `matches!`
- **Called by:** `hidden_run` (markdown/from_docx/agent.rs:133) (?); `Styles::parse` (markdown/from_docx/mod.rs:1726) (?); `Writer::run` (markdown/from_docx/mod.rs:3119) (?); `document_defaults` (markdown/view_styles.rs:630) (?)

#### `parse_xml`  (L367-436, pub(crate))
`pub(crate) fn parse_xml(bytes: &[u8]) -> Result<Element, String>`

- **Does:** Parse XML bytes into an element tree (the document element).
- **Calls (crate):** `decode` (markdown/from_docx/ooxml.rs:438) L368; `element_from` (markdown/from_docx/ooxml.rs:461) L381,384; `push_child` (markdown/from_docx/ooxml.rs:480) L385,392,426; `push_text` (markdown/from_docx/ooxml.rs:486) L396,400,417
- **Calls (std/external/unresolved):** `.buffer_position`, `.config_mut`, `.find_map`, `.into_iter`, `.into`, `.len`, `.map_err`, `.ok_or_else`, `.pop`, `.push`, `.read_event`, `.resolve_char_ref`, `.to_string`, `.trim_text`, `.unwrap_or_default`, `.xml_content`, `Err`, `Node::Element`, `Reader::from_str`, `Some`
- **Macros:** `format!`, `vec!`
- **Called by:** `tests::body` (markdown/from_docx/agent.rs:2767); `tests::doc` (markdown/from_docx/agent.rs:2771); `tests::handles_prefer_comment_initials_disambiguate_collisions_and_collect_timestamps` (markdown/from_docx/agent.rs:3000); `Deps::dep_for` (markdown/from_docx/drawing.rs:438); `canonical` (markdown/from_docx/drawing.rs:867); `regenerable` (markdown/from_docx/drawing.rs:917); `tests::element` (markdown/from_docx/drawing.rs:1525); `tests::ftr` (markdown/from_docx/header.rs:1948); `tests::story_paragraphs_agree_with_the_paragraphs_edit_resolves` (markdown/from_docx/header.rs:1957); `tests::a_section_s_universal_measures_read_as_twips` (markdown/from_docx/header.rs:1971); `tests::an_mm_spelled_page_is_the_page_its_twips_twin_names` (markdown/from_docx/header.rs:2000); `tests::word_s_paper_list_names_its_sizes` (markdown/from_docx/header.rs:2018); +10 more

#### `decode`  (L438-459)
`fn decode(bytes: &[u8]) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_chunks`, `.collect`, `.into_owned`, `.iter`, `.map`, `.strip_prefix`, `.unwrap_or`, `String::from_utf16_lossy`, `String::from_utf8_lossy`, `u16::from_be_bytes`, `u16::from_le_bytes`
- **Called by:** `parse_xml` (markdown/from_docx/ooxml.rs:367)

#### `element_from`  (L461-478)
`fn element_from(start: &quick_xml::events::BytesStart<'_>) -> Result<Element, String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_ref`, `.attributes`, `.into_owned`, `.map_err`, `.name`, `.normalized_value`, `.push`, `.to_string`, `.with_checks`, `Ok`, `Vec::new`
- **Called by:** `parse_xml` (markdown/from_docx/ooxml.rs:367)

#### `push_child`  (L480-484)
`fn push_child(stack: &mut [Element], node: Node)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.last_mut`, `.push`
- **Called by:** `parse_xml` (markdown/from_docx/ooxml.rs:367)

#### `push_text`  (L486-497)
`fn push_text(stack: &mut [Element], text: &str)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_empty`, `.last_mut`, `.push_str`, `.push`, `.to_string`, `Node::Text`
- **Called by:** `parse_xml` (markdown/from_docx/ooxml.rs:367)

#### `collapse_ws`  (L499-501, pub(crate))
`pub(crate) fn collapse_ws(s: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.join`, `.split_whitespace`
- **Called by:** `image_markdown` (markdown/from_docx/ooxml.rs:1015)

#### `Inline::keep_spaces`  (L542-544, pub(crate))
`pub(crate) fn keep_spaces(&mut self)`

- **Does:** Agent view: keeps runs of spaces as the file holds them.
- **Calls:** none
- **Called by:** `Critic::into_inline` (markdown/from_docx/critic.rs:443); `ooxml_package_boundary_tests::an_agent_line_keeps_its_leading_spaces_as_character_references` (markdown/from_docx/ooxml.rs:1529)

#### `Inline::verbatim`  (L547-550, pub(crate))
`pub(crate) fn verbatim(&mut self)`

- **Does:** A code block: keeps every space, a line's indent included.
- **Calls:** none
- **Called by:** `Critic::into_inline` (markdown/from_docx/critic.rs:443); `ooxml_package_boundary_tests::an_agent_line_keeps_its_leading_spaces_as_character_references` (markdown/from_docx/ooxml.rs:1529)

#### `Inline::render_code`  (L555-570, pub(crate))
`pub(crate) fn render_code(&self) -> String`

- **Does:** A code block's paragraph: every mark dropped, for a fence's content is literal and formatting printed inside it would come back as text.
- **Calls (crate):** `Inline::render` (markdown/from_docx/ooxml.rs:614) L569
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.iter`, `.map`, `Marks::default`
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2546) (?)

#### `Inline::edges`  (L574-589, pub(crate))
`pub(crate) fn edges(&self) -> (bool, bool)`

- **Does:** Whether the text starts and ends with a space.
- **Calls (std/external/unresolved):** `.as_str`, `.ends_with`, `.filter`, `.is_empty`, `.is_some_and`, `.iter`, `.map`, `.next`, `.rfind`, `.starts_with`
- **Macros:** `matches!`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96) (?); `Writer::paragraph` (markdown/from_docx/mod.rs:2546) (?); `Writer::cell_parts` (markdown/from_docx/mod.rs:3693) (?); `Writer::note_text` (markdown/from_docx/mod.rs:3867) (?)

#### `Inline::push_styled`  (L593-609, pub(crate))
`pub(crate) fn push_styled(&mut self, text: &str, marks: Marks, link: Option<&str>)`

- **Does:** Appends text with its marks, merging into the last span when they and the link match.
- **Calls (std/external/unresolved):** `.as_deref`, `.is_empty`, `.last_mut`, `.map`, `.push_str`, `.push`, `.to_string`
- **Called by:** `Critic::into_inline` (markdown/from_docx/critic.rs:443); `Inline::push` (markdown/from_docx/ooxml.rs:1258); `ooxml_package_boundary_tests::a_plain_space_between_code_spans_stays_between_them` (markdown/from_docx/ooxml.rs:1552)

#### `Inline::render`  (L614-653, pub(crate))
`pub(crate) fn render(&self, emphasis: bool) -> String`

- **Does:** Render to Markdown.
- **Calls (crate):** `render_emphasis` (markdown/from_docx/ooxml.rs:705) L626,648; `split_ws` (markdown/from_docx/ooxml.rs:921) L627; `link_destination` (markdown/from_docx/ooxml.rs:660) L630; `email_autolink` (markdown/from_docx/ooxml.rs:676) L639; `tidy_inline` (markdown/from_docx/ooxml.rs:933) L652
- **Calls (std/external/unresolved):** `.all`, `.as_str`, `.clone`, `.is_empty`, `.iter`, `.len`, `.push_str`, `.replace`, `.split_once`, `.strip_prefix`, `.trim`, `.unwrap_or`, `Some`, `String::new`
- **Macros:** `format!`
- **Called by:** `Inline::render_code` (markdown/from_docx/ooxml.rs:555)

#### `link_destination`  (L660-671)
`fn link_destination(url: &str) -> String`

- **Does:** Spaces become `%20`.
- **Calls (std/external/unresolved):** `.contains`, `.replace`
- **Macros:** `format!`
- **Called by:** `Inline::render` (markdown/from_docx/ooxml.rs:614)

#### `email_autolink`  (L676-682)
`fn email_autolink(address: &str) -> bool`

- **Does:** Whether `address` is the inside of a CommonMark email autolink, the addresses pulldown-cmark reads (`<local@domain>`).
- **Calls (crate):** `local_part` (markdown/from_docx/ooxml.rs:684) L681; `domain` (markdown/from_docx/ooxml.rs:693) L681
- **Calls (std/external/unresolved):** `.as_bytes`, `.iter`, `.position`
- **Called by:** `Inline::render` (markdown/from_docx/ooxml.rs:614)

#### `local_part`  (L684-691)
`fn local_part(bytes: &[u8]) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.all`, `.contains`, `.is_ascii_alphanumeric`, `.is_empty`, `.iter`
- **Called by:** `email_autolink` (markdown/from_docx/ooxml.rs:676)

#### `domain`  (L693-703)
`fn domain(bytes: &[u8]) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.all`, `.contains`, `.ends_with`, `.is_ascii_alphanumeric`, `.is_empty`, `.iter`, `.len`, `.split`, `.starts_with`
- **Called by:** `email_autolink` (markdown/from_docx/ooxml.rs:676)

#### `render_emphasis`  (L705-852)
`fn render_emphasis(spans: &[Span], emphasis: bool) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `edges` (markdown/from_docx/ooxml.rs:858) L748,757; `pair` (ambiguous: `pair` (markdown/diff.rs:621); `pair` (markdown/patch.rs:445)) L776; `delimiters_read` (markdown/from_docx/ooxml.rs:890) L792,806,808; `opens_empty_emphasis` (markdown/from_docx/ooxml.rs:873) L818; `split_ws` (markdown/from_docx/ooxml.rs:921) L837; `code_fence` (markdown/from_docx/ooxml.rs:910) L841
- **Calls (std/external/unresolved):** `.as_ref`, `.chars`, `.clone`, `.collect`, `.copied`, `.enumerate`, `.flatten`, `.get`, `.into_iter`, `.is_empty`, `.is_some`, `.iter`, `.join`, `.last_mut`, `.last`, `.map_or`, `.map`, `.next`, `.or`, `.push_str`, `.push`, `.split`, `.to_string`, `.trim`, `Marks::default`, `Some`, `String::new`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `Inline::render` (markdown/from_docx/ooxml.rs:614)

#### `edges`  (L858-869)
`fn edges(marks: Marks, text: &str) -> (&str, std::borrow::Cow<'_, str>, &str)`

- **Does:** `text` as a stretch with `marks` prints it: the spaces before, the text inside the marks, the spaces after.
- **Calls (crate):** `split_ws` (markdown/from_docx/ooxml.rs:921) L859
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.is_some`, `.len`, `.repeat`, `space`, `std::borrow::Cow::Borrowed`, `std::borrow::Cow::Owned`
- **Macros:** `format!`
- **Called by:** `render_emphasis` (markdown/from_docx/ooxml.rs:705)

#### `opens_empty_emphasis`  (L873-883)
`fn opens_empty_emphasis(out: &str, open: &str) -> bool`

- **Does:** `open` (`**`, `_`, `**_`) against the marks `out` already ends with.
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.next`, `.rev`, `.take_while`
- **Called by:** `render_emphasis` (markdown/from_docx/ooxml.rs:705)

#### `delimiters_read`  (L890-906)
`fn delimiters_read( before: Option<char>, inner: &str, after: Option<char>, underscore: bool, ) -> bool`

- **Does:** Whether emphasis delimiters around `inner`, between `before` and `after` (None: the line's edge), open and close as CommonMark reads them: an opener before punctuation needs space or punctuation before it, a closer after punctuation the same after it; `_` (`underscore`) also opens or closes nowhere inside a word.
- **Calls (crate):** `word` (ambiguous, 5 candidates) L905
- **Calls (std/external/unresolved):** `.chars`, `.is_alphanumeric`, `.is_ascii_punctuation`, `.is_none_or`, `.is_some_and`, `.is_whitespace`, `.last`, `.next`, `open_side`, `punctuation`
- **Called by:** `render_emphasis` (markdown/from_docx/ooxml.rs:705)

#### `code_fence`  (L910-919)
`fn code_fence(text: &str) -> (String, String)`

- **Does:** The backticks around a code span's text: one more than its longest run of backticks, and a space inside when the text starts or ends with one.
- **Calls (std/external/unresolved):** `.ends_with`, `.map`, `.max`, `.repeat`, `.split`, `.starts_with`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `render_emphasis` (markdown/from_docx/ooxml.rs:705)

#### `split_ws`  (L921-925)
`fn split_ws(s: &str) -> (&str, &str, &str)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.len`, `.max`, `.trim_end`, `.trim_start`
- **Called by:** `Inline::render` (markdown/from_docx/ooxml.rs:614); `render_emphasis` (markdown/from_docx/ooxml.rs:705); `edges` (markdown/from_docx/ooxml.rs:858)

#### `tidy_inline`  (L933-969)
`fn tidy_inline(s: &str, keep_spaces: bool, verbatim: bool) -> String`

- **Does:** Collapse runs of spaces and trim each line.
- **Calls (crate):** `equation_split` (markdown/from_docx/ooxml.rs:973) L942; `finish_line` (markdown/from_docx/ooxml.rs:993) L960
- **Calls (std/external/unresolved):** `.chars`, `.first`, `.is_empty`, `.is_some_and`, `.join`, `.last`, `.len`, `.pop`, `.push_str`, `.push`, `.remove`, `.split`, `String::with_capacity`, `Vec::new`
- **Called by:** `Inline::render` (markdown/from_docx/ooxml.rs:614); `tests::an_equation_keeps_its_spaces_where_text_collapses` (markdown/from_docx/ooxml.rs:1280)

#### `equation_split`  (L973-990)
`fn equation_split(line: &str) -> (&str, &str, &str)`

- **Does:** `line` as the text before its first closed equation, the equation with its fences, and the rest; all text when it holds none.
- **Calls (crate):** `unescaped_dollar` (markdown/from_docx/mod.rs:1415) L974; `math_body` (markdown/from_docx/mod.rs:1431) L983
- **Calls (std/external/unresolved):** `.len`, `.starts_with`
- **Called by:** `tidy_inline` (markdown/from_docx/ooxml.rs:933)

#### `finish_line`  (L993-1012)
`fn finish_line(line: &str, keep_spaces: bool, verbatim: bool) -> String`

- **Does:** One line of [`tidy_inline`].
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.is_empty`, `.len`, `.push_str`, `.take_while`, `.to_string`, `.trim_end`, `.trim_start`, `.trim`, `String::with_capacity`
- **Called by:** `tidy_inline` (markdown/from_docx/ooxml.rs:933)

#### `image_markdown`  (L1015-1035, pub(crate))
`pub(crate) fn image_markdown(alt: &str, target: Option<&str>) -> Option<String>`

- **Does:** Markdown for an image with alt text; images without alt text are dropped.
- **Calls (crate):** `collapse_ws` (markdown/from_docx/ooxml.rs:499) L1016
- **Calls (std/external/unresolved):** `.and_then`, `.filter`, `.is_empty`, `.next`, `.replace`, `.rsplit`, `.unwrap_or`, `Some`
- **Macros:** `format!`
- **Called by:** `Media::markdown` (markdown/from_docx/ooxml.rs:1095)

#### `Media::load`  (L1052-1084, pub(crate))
`pub(crate) fn load( package: &mut Package<'_>, rels: &Rels, extracted: Option<Extracted>, ) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `.read` (ambiguous: `Package::read` (audit.rs:354); `Package::read` (markdown/from_docx/ooxml.rs:66)) L1072
- **Calls (std/external/unresolved):** `.clone`, `.contains_key`, `.ends_with`, `.get`, `.insert`, `.len`, `HashMap::new`, `Self::default`, `Some`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96); `ooxml_package_boundary_tests::media_relationships_extract_owned_bytes_once_and_leave_non_images_as_alt_text` (markdown/from_docx/ooxml.rs:1388); `ooxml_package_boundary_tests::media_part_limit_retains_the_first_512_authored_targets_in_relationship_order` (markdown/from_docx/ooxml.rs:1452)

#### `Media::extracted_markdown`  (L1088-1091, pub(crate))
`pub(crate) fn extracted_markdown(&mut self, alt: &str, target: Option<&str>) -> Option<String>`

- **Does:** Markdown only when the picture's raster is extracted to a file; `None` when pictures are not extracted or this one is no raster.
- **Calls (crate):** `.add` (ambiguous, 5 candidates) L1090
- **Calls (std/external/unresolved):** `.and_then`, `.as_mut`, `.get`
- **Called by:** `Writer::drawing` (markdown/from_docx/mod.rs:3366)

#### `Media::markdown`  (L1095-1103, pub(crate))
`pub(crate) fn markdown(&mut self, alt: &str, target: Option<&str>) -> Option<String>`

- **Does:** Markdown for a picture: the extracted file when pictures are extracted and this one is a raster, else the alt-text form.
- **Calls (crate):** `.add` (ambiguous, 5 candidates) L1098; `image_markdown` (markdown/from_docx/ooxml.rs:1015) L1102
- **Calls (std/external/unresolved):** `.and_then`, `.as_mut`, `.get`, `Some`
- **Called by:** `Writer::drawing_text` (markdown/from_docx/mod.rs:3464); `ooxml_package_boundary_tests::media_relationships_extract_owned_bytes_once_and_leave_non_images_as_alt_text` (markdown/from_docx/ooxml.rs:1388) (?)

#### `Media::into_files`  (L1106-1108, pub(crate))
`pub(crate) fn into_files(self) -> std::collections::BTreeMap<String, Vec<u8>>`

- **Does:** The pictures written, by file name.
- **Calls (std/external/unresolved):** `.map`, `.unwrap_or_default`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96); `ooxml_package_boundary_tests::media_relationships_extract_owned_bytes_once_and_leave_non_images_as_alt_text` (markdown/from_docx/ooxml.rs:1388); `ooxml_package_boundary_tests::media_part_limit_retains_the_first_512_authored_targets_in_relationship_order` (markdown/from_docx/ooxml.rs:1452) (?)

#### `Blocks::new`  (L1125-1133, pub(crate))
`pub(crate) fn new() -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `String::new`
- **Called by:** `convert` (markdown/from_docx/mod.rs:96); `Writer::box_comments` (markdown/from_docx/mod.rs:3422); `Writer::drawing_text` (markdown/from_docx/mod.rs:3464)

#### `Blocks::push`  (L1137-1140, pub(crate))
`pub(crate) fn push(&mut self, block: &str, is_list: bool)`

- **Does:** A block whose start cannot carry a CriticMarkup delimiter (a table, a text box); a tracked break before it is left unmarked.
- **Calls (crate):** `Blocks::push_prefixed` (markdown/from_docx/ooxml.rs:1145) L1139
- **Called by:** `convert` (markdown/from_docx/mod.rs:96); `Writer::blocks_in` (markdown/from_docx/mod.rs:2401); `Writer::paragraph` (markdown/from_docx/mod.rs:2546)

#### `Blocks::push_prefixed`  (L1145-1147, pub(crate))
`pub(crate) fn push_prefixed(&mut self, prefix: &str, body: &str, is_list: bool)`

- **Does:** A block made of Markdown syntax (`## `, ` 1.
- **Calls (crate):** `Blocks::push_paragraph` (markdown/from_docx/ooxml.rs:1164) L1146
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2401); `Blocks::push` (markdown/from_docx/ooxml.rs:1137)

#### `Blocks::push_line`  (L1151-1160, pub(crate))
`pub(crate) fn push_line(&mut self, line: &str)`

- **Does:** An id, table or page line of the agent view.
- **Calls (std/external/unresolved):** `.is_empty`, `.push_str`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2401); `Writer::paragraph` (markdown/from_docx/mod.rs:2546); `Writer::flush_empty` (markdown/from_docx/mod.rs:2769); `Writer::page_lines` (markdown/from_docx/mod.rs:2783)

#### `Blocks::push_paragraph`  (L1164-1198, pub(crate))
`pub(crate) fn push_paragraph( &mut self, prefix: &str, body: &str, is_list: bool, (leading, trailing): (bool, bool), )`

- **Does:** A paragraph, with whether its text started and ended with a space before rendering trimmed it (see [`critic::splice`]).
- **Calls (crate):** `splice` (markdown/from_docx/critic.rs:1197) L1186
- **Calls (std/external/unresolved):** `.as_ref`, `.is_empty`, `.push_str`, `.take`, `.trim_end`, `.trim`
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2546); `Blocks::push_prefixed` (markdown/from_docx/ooxml.rs:1145)

#### `Blocks::set_separator`  (L1202-1204, pub(crate))
`pub(crate) fn set_separator(&mut self, change: Option<Change>)`

- **Does:** Records that the paragraph mark ending the last block was inserted or deleted (`None` clears it).
- **Calls:** none
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2546)

#### `Blocks::finish`  (L1206-1208, pub(crate))
`pub(crate) fn finish(self) -> String`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `convert` (markdown/from_docx/mod.rs:96); `Writer::box_comments` (markdown/from_docx/mod.rs:3422); `Writer::drawing_text` (markdown/from_docx/mod.rs:3464)

#### `ListIndent::reset`  (L1221-1223, pub(crate))
`pub(crate) fn reset(&mut self)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clear`
- **Called by:** `Writer::blocks_in` (markdown/from_docx/mod.rs:2401); `Writer::paragraph` (markdown/from_docx/mod.rs:2546); `tests::list_indent_nests_under_content` (markdown/from_docx/ooxml.rs:1347); `ooxml_package_boundary_tests::literal_hyperlinks_and_mail_addresses_use_exact_autolink_forms` (markdown/from_docx/ooxml.rs:1487)

#### `ListIndent::place`  (L1228-1233, pub(crate))
`pub(crate) fn place(&self, level: usize) -> (usize, usize)`

- **Does:** Where an item at `level` goes: its indentation, and the level that indentation says (the items it nests under; less than `level` when a level between is skipped).
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.iter`, `.len`, `.map`, `.min`, `.sum`
- **Called by:** `Writer::paragraph` (markdown/from_docx/mod.rs:2546); `tests::list_indent_nests_under_content` (markdown/from_docx/ooxml.rs:1347) (?)

#### `ListIndent::item`  (L1236-1249, pub(crate))
`pub(crate) fn item(&mut self, level: usize, marker: &str, text: &str) -> String`

- **Does:** Format an item at `level` with `marker` (e.g.
- **Calls (std/external/unresolved):** `.chars`, `.count`, `.iter`, `.len`, `.map`, `.min`, `.push`, `.repeat`, `.replace`, `.sum`, `.truncate`
- **Macros:** `format!`
- **Called by:** `Build::segments` (debug/diff.rs:931) (?); `Build::run` (debug/diff.rs:1000) (?); `boundary_coverage_tests::toggle_normalization_distinguishes_boolean_values_from_numeric_properties` (debug/diff.rs:2269) (?); `boundary_coverage_tests::relationship_attributes_resolve_targets_without_inventing_missing_targets` (debug/diff.rs:2300) (?); `tests::list_tables_are_read_within_their_fib_ranges` (legacy_doc.rs:2454) (?); `Writer::paragraph` (markdown/from_docx/mod.rs:2546); `tests::list_indent_nests_under_content` (markdown/from_docx/ooxml.rs:1347) (?); `ooxml_package_boundary_tests::literal_hyperlinks_and_mail_addresses_use_exact_autolink_forms` (markdown/from_docx/ooxml.rs:1487) (?)

## Test functions (15)

- `Inline::push` (L1258): _no doc comment_
- `tests::resolves_relative_targets` (L1269): _no doc comment_
- `tests::an_equation_keeps_its_spaces_where_text_collapses` (L1280): _no doc comment_
- `tests::bold_closed_against_bold_is_not_empty_emphasis` (L1296): _no doc comment_
- `tests::inline_merges_and_places_markers` (L1322): _no doc comment_
- `tests::xml_entities_and_depth` (L1337): _no doc comment_
- `tests::list_indent_nests_under_content` (L1347): _no doc comment_
- `ooxml_package_boundary_tests::package_bytes` (L1368): _no doc comment_
- `ooxml_package_boundary_tests::png` (L1378): _no doc comment_
- `ooxml_package_boundary_tests::media_relationships_extract_owned_bytes_once_and_leave_non_images_as_alt_text` (L1388): _no doc comment_
- `ooxml_package_boundary_tests::media_part_limit_retains_the_first_512_authored_targets_in_relationship_order` (L1452): _no doc comment_
- `ooxml_package_boundary_tests::literal_hyperlinks_and_mail_addresses_use_exact_autolink_forms` (L1487): _no doc comment_
- `ooxml_package_boundary_tests::an_agent_line_keeps_its_leading_spaces_as_character_references` (L1529): _no doc comment_
- `ooxml_package_boundary_tests::a_plain_space_between_code_spans_stays_between_them` (L1552): _no doc comment_
- `ooxml_package_boundary_tests::utf16_xml_keeps_namespaced_attributes_and_authored_text` (L1571): _no doc comment_
