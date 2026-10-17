# Functions and methods in `src/opc/mod.rs`

45 non-test, 26 test. Part of the [symbol manifest](../README.md).

#### `Relationships::new`  (L41-43, pub)
`pub fn new() -> Self`

- **Does:** An empty collection whose generated IDs start at `rId1`.
- **Calls (std/external/unresolved):** `Self`, `oxml_opc::Relationships::new`
- **Called by:** `read_relationships` (opc/mod.rs:214); `read_package` (opc/mod.rs:266); `PartFs::part_rels_mut` (opc/mod.rs:794)

#### `Relationships::from_xml`  (L50-52, pub)
`pub fn from_xml(xml: &[u8]) -> Result<Self, OpcError>`

- **Does:** Read a relationships part with the package reader's refusal contract.
- **Calls (crate):** `read_relationships` (opc/mod.rs:214) L51
- **Called by:** `tests::public_and_package_relationship_readers_agree` (opc/mod.rs:882)

#### `<Relationships as Deref>::deref`  (L58-60, impl Deref)
`fn deref(&self) -> &Self::Target`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `<Relationships as DerefMut>::deref_mut`  (L64-66, impl DerefMut)
`fn deref_mut(&mut self) -> &mut Self::Target`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `refused`  (L76-81, pub(crate))
`pub(crate) fn refused<E>(err: E) -> OpcError where E: std::error::Error + Send + Sync + 'static,`

- **Does:** Input the engine refuses before or while reading it, as an `Err` the caller can handle rather than a panic, which would abort a WASM instance or the Python interpreter.
- **Calls (std/external/unresolved):** `OpcError::Io`, `std::io::Error::new`
- **Called by:** `admit_input` (document_comparer.rs:4907); `tests::recording_refuses_what_the_plan_names_differently` (edit/flags.rs:835) (?); `Select::parse` (markdown/mod.rs:453) (?); `check_opc_attributes` (opc/mod.rs:369); `check_package_metadata` (opc/mod.rs:390)

#### `drop_repeated_relationships`  (L91-118)
`fn drop_repeated_relationships(rels: &mut Relationships) -> Result<Vec<String>, OpcError>`

- **Does:** Exactly one leading slash: callers build `/{part}` from names that may already be absolute, and a `//word/…` part name is invalid OPC.
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.contains`, `.get`, `.insert`, `.into_iter`, `.len`, `.next`, `.push`, `.retain`, `.unwrap_or`, `Err`, `HashMap::new`, `Ok`, `Vec::new`, `Vec::with_capacity`
- **Called by:** `PartFs::open` (opc/mod.rs:456)

#### `norm`  (L120-122)
`fn norm(name: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.trim_start_matches`
- **Macros:** `format!`
- **Called by:** `scan_family_faces` (convert/font.rs:2281) (?); `family_faces_in_with` (convert/font.rs:2473) (?); `font_name_index` (convert/font.rs:2593) (?); `catalogue_paints_family` (convert/font.rs:3481) (?); `read_package` (opc/mod.rs:266); `resolve_rel_target` (opc/mod.rs:426); `PartFs::part_bytes` (opc/mod.rs:493); `PartFs::set_part` (opc/mod.rs:510); `PartFs::remove_part` (opc/mod.rs:527); `PartFs::content_type_for` (opc/mod.rs:608); `PartFs::add_content_type_override` (opc/mod.rs:616); `PartFs::adopt_missing_content_types` (opc/mod.rs:631); +3 more

#### `relative_rel_target`  (L129-137, pub)
`pub fn relative_rel_target(source_part: &str, target_part: &str) -> String`

- **Does:** The relationship `Target` value that reaches `target_part` from `source_part`: relative to the source's folder when the target sits under it, absolute (`/media/image.bin`) otherwise.
- **Calls (std/external/unresolved):** `.and_then`, `.is_empty`, `.map_or`, `.rsplit_once`, `.strip_prefix`, `.to_string`, `.trim_start_matches`
- **Macros:** `format!`
- **Called by:** `add_related_part` (append.rs:790); `CommentFamily::store` (comments.rs:798); `install_parts_from` (comparer/comments.rs:937); `union_comments_xml` (comparer/comments.rs:1190); `reconcile_dangling_relationships` (comparer/parts.rs:273); `carry_relationship` (comparer/parts.rs:517); `ensure_factory_package_chrome` (document_comparer.rs:3402); `adopt_revised_header_footer` (document_comparer.rs:5166); `adopt_b_notes_when_a_lacks_separators` (document_comparer.rs:5863); `repair_missing_core_relationships` (document_comparer.rs:6250); `adopted_story_relationship_owner_tests::adopted_root_and_nested_stories_keep_colliding_media_relationships_resolvable` (document_comparer.rs:12527); `Transaction::image_paragraph` (edit/images.rs:143); +9 more

#### `metadata_records`  (L147-212)
`fn metadata_records( xml: &[u8], root: &str, namespace: &str, tags: &[&str], ) -> Result<Vec<Vec<u8>>, OpcError>`

- **Does:** Select the same unprefixed metadata records as the legacy adapter, folding explicit closes, then let upstream decode and validate each record.
- **Calls (crate):** `Dom::attributes` (xmllinq/mod.rs:885) (by name) L160
- **Calls (std/external/unresolved):** `.as_bytes`, `.as_ref`, `.copied`, `.find`, `.into_inner`, `.iter`, `.map_err`, `.name`, `.push_attribute`, `.push`, `.read_event_into`, `.take`, `.to_owned`, `.with_checks`, `.write_event`, `BytesEnd::new`, `BytesStart::new`, `Event::Empty`, `Event::End`, `Event::Start`, `Ok`, `Some`, `Vec::new`, `fold`, `matched`, `quick_xml_041::Reader::from_reader`, `quick_xml_041::Writer::new`
- **Macros:** `matches!`
- **Called by:** `read_relationships` (opc/mod.rs:214); `read_content_types` (opc/mod.rs:246)

#### `read_relationships`  (L214-244)
`fn read_relationships(xml: &[u8]) -> Result<Relationships, OpcError>`

- **Does:** _no doc comment_
- **Calls (crate):** `Relationships::new` (opc/mod.rs:41) L215; `metadata_records` (opc/mod.rs:147) L217
- **Calls (std/external/unresolved):** `.add_with_id`, `.and_then`, `.filter_map`, `.iter`, `.map`, `.max_by_key`, `.ok`, `.parse`, `.push`, `.strip_prefix`, `Ok`, `Vec::new`, `oxml_opc::Relationships::from_xml`
- **Called by:** `Relationships::from_xml` (opc/mod.rs:50); `read_package` (opc/mod.rs:266); `PartFs::set_part` (opc/mod.rs:510)

#### `read_content_types`  (L246-262)
`fn read_content_types(xml: &[u8]) -> Result<ContentTypes, OpcError>`

- **Does:** _no doc comment_
- **Calls (crate):** `metadata_records` (opc/mod.rs:147) L251
- **Calls (std/external/unresolved):** `.extend`, `ContentTypes::from_xml`, `HashMap::new`, `Ok`
- **Called by:** `read_package` (opc/mod.rs:266)

#### `read_package`  (L266-323)
`fn read_package(bytes: &[u8]) -> Result<PartFs, OpcError>`

- **Does:** Populate parts and local relationships without invoking upstream's whole-package normalization and duplicate-refusal policies ahead of the adapter's repairs.
- **Calls (crate):** `zip_entry_name` (admission.rs:595) L278; `unicode_path_fields` (admission.rs:563) L281; `Relationships::new` (opc/mod.rs:41) L298; `read_content_types` (opc/mod.rs:246) L303; `read_relationships` (opc/mod.rs:214) L309,317; `rels_path_to_part_name` (opc/mod.rs:343) L313; `norm` (opc/mod.rs:120) L317,319
- **Calls (std/external/unresolved):** `.as_u16`, `.by_index`, `.clone`, `.cloned`, `.ends_with`, `.extra_data_fields`, `.flags`, `.insert`, `.is_dir`, `.len`, `.name_raw`, `.ok_or_else`, `.read_to_end`, `.remove`, `.strip_suffix`, `.to_ascii_lowercase`, `.to_string`, `.unwrap_or_else`, `.unwrap_or`, `Cursor::new`, `Err`, `HashMap::new`, `HashSet::new`, `Ok`, `OpcError::DuplicatePartName`, `OpcError::PartNotFound`, `OpcPackage::new`, `Vec::new`, `ZipArchive::new`
- **Called by:** `PartFs::open` (opc/mod.rs:456)

#### `denorm`  (L325-327)
`fn denorm(name: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.to_string`, `.trim_start_matches`
- **Called by:** `PartFs::parts` (opc/mod.rs:532); `PartFs::main_document_part` (opc/mod.rs:829)

#### `part_name_to_rels_path`  (L331-338)
`fn part_name_to_rels_path(part_name: &str) -> String`

- **Does:** `.rels` file path (e.g.
- **Calls (std/external/unresolved):** `.rsplit_once`, `.strip_prefix`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `PartFs::to_zip` (opc/mod.rs:548)

#### `rels_path_to_part_name`  (L343-357)
`fn rels_path_to_part_name(name: &str) -> Option<String>`

- **Does:** The part a `.rels` path belongs to (`word/_rels/header1.xml.rels` → `word/header1.xml`; `_rels/.rels` → the package, as `""`), or None when `name` is not a relationships path.
- **Calls (std/external/unresolved):** `.is_empty`, `.rsplit_once`, `.strip_prefix`, `.strip_suffix`, `.to_string`, `.unwrap_or`, `Some`
- **Macros:** `format!`
- **Called by:** `read_package` (opc/mod.rs:266); `PartFs::set_part` (opc/mod.rs:510); `tests::rels_paths_name_their_owning_part` (opc/mod.rs:1259)

#### `check_opc_attributes`  (L369-388)
`fn check_opc_attributes(xml: &[u8]) -> Result<(), OpcError>`

- **Does:** Retain the OPC metadata budget independently of the parser upgrade that fixed RUSTSEC-2026-0194/0195. _(from a code comment)_
- **Calls (crate):** `Dom::attributes` (xmllinq/mod.rs:885) (by name) L374; `refused` (opc/mod.rs:76) L376
- **Calls (std/external/unresolved):** `.enumerate`, `.map_err`, `.read_event`, `.with_checks`, `Err`, `Ok`, `quick_xml::Reader::from_reader`, `std::io::Error::new`
- **Called by:** `check_package_metadata` (opc/mod.rs:390); `PartFs::set_part` (opc/mod.rs:510); `tests::opc_attribute_budget_bounds_upstream_duplicate_checks` (opc/mod.rs:843)

#### `check_package_metadata`  (L390-409)
`fn check_package_metadata(bytes: &[u8], limit: u64) -> Result<(), OpcError>`

- **Does:** _no doc comment_
- **Calls (crate):** `refused` (opc/mod.rs:76) L401; `check_opc_attributes` (opc/mod.rs:369) L406
- **Calls (std/external/unresolved):** `.by_index`, `.ends_with`, `.len`, `.name`, `.read_to_end`, `.saturating_add`, `.take`, `.unwrap_or`, `Cursor::new`, `Err`, `Ok`, `Vec::new`, `ZipArchive::new`, `std::io::Error::new`, `u64::try_from`
- **Called by:** `PartFs::open` (opc/mod.rs:456); `tests::opc_attribute_budget_bounds_upstream_duplicate_checks` (opc/mod.rs:843)

#### `resolve_rel_target`  (L426-439, pub(crate))
`pub(crate) fn resolve_rel_target(source_part: &str, rel_target: &str) -> String`

- **Does:** Canonical target resolution shared by package APIs and text story labels.
- **Calls (crate):** `norm` (opc/mod.rs:120) L427
- **Calls (std/external/unresolved):** `.join`, `.pop`, `.push`, `.split`, `OpcPackage::resolve_rel_target`, `Vec::new`
- **Called by:** `story_roles_all` (debug.rs:1867); `PartFs::resolve_rel_target` (opc/mod.rs:600); `PartFs::part_rels_key` (opc/mod.rs:762)

#### `PartFs::open`  (L456-482, pub)
`pub fn open(bytes: &[u8]) -> Result<Self, OpcError>`

- **Does:** Open a `.docx`/OPC package from raw bytes: read its parts, relationships and `[Content_Types].xml`, normalize their part names to the no-leading-slash form, fold any explicit `<…></…>` closes of empty elements (`<Relationship …></Relationship>` and its `[Content_Types]` siblings) into self-closing ones, decode XML ent...
- **Calls (crate):** `check_package_metadata` (opc/mod.rs:390) L457; `InputLimits::compare` (admission.rs:75) L459; `read_package` (opc/mod.rs:266) L461; `drop_repeated_relationships` (opc/mod.rs:91) L463,474
- **Calls (std/external/unresolved):** `.clone`, `.cloned`, `.collect`, `.extend`, `.get_mut`, `.into_iter`, `.keys`, `.map`, `.sort`, `.to_string`, `.trim_start_matches`, `Ok`, `String::new`
- **Called by:** `tests::admission_and_partfs_agree_on_decoded_names_and_duplicates` (admission.rs:1052); `open` (append.rs:316); `tests::free_sibling_numbers_a_taken_name` (append.rs:1366); `public_memory_append_owner_boundary_tests::package` (append.rs:1482); `public_memory_append_owner_boundary_tests::public_append_keeps_source_property_history_and_fresh_annotation_owners_at_every_join_boundary` (append.rs:1511); `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1594); `public_memory_spacer_ownership_tests::package` (audit.rs:964); `public_memory_spacer_ownership_tests::spacer_audit_respects_authored_sections_page_breaks_and_table_cell_ownership` (audit.rs:983); `list_changes_tolerant` (changes.rs:294); `open` (changes.rs:391); `people_and_text_boundary_tests::pkg` (comments.rs:1203); `public_memory_anchor_boundary_tests::package` (comments.rs:1347); +148 more

#### `PartFs::repaired_duplicate_ids`  (L488-490, pub)
`pub fn repaired_duplicate_ids(&self) -> &[(String, String)]`

- **Does:** `(part, Id)` of each relationship the source repeated identically, kept once on open (`""`: the package's own relationships).
- **Calls:** none
- **Called by:** `tests::a_repeated_relationship_id_is_repaired_when_identical_and_refused_otherwise` (opc/mod.rs:976) (?); `check_relationship_integrity` (validate.rs:646)

#### `PartFs::part_bytes`  (L493-495, pub)
`pub fn part_bytes(&self, name: &str) -> Option<&[u8]>`

- **Does:** `PartFS.partBytes(name)` — raw bytes of a part.
- **Calls (crate):** `norm` (opc/mod.rs:120) L494
- **Calls (std/external/unresolved):** `.get_part`
- **Called by:** `tests::admission_and_partfs_agree_on_decoded_names_and_duplicates` (admission.rs:1052) (?); `main_part` (append.rs:323); `parse` (append.rs:330); `related` (append.rs:760); `free_sibling` (append.rs:771); `carry_styles_and_numbering` (append.rs:861); `public_memory_append_owner_boundary_tests::snapshot` (append.rs:1493); `public_memory_append_owner_boundary_tests::public_append_keeps_source_property_history_and_fresh_annotation_owners_at_every_join_boundary` (append.rs:1511) (?); `public_memory_spacer_ownership_tests::snapshot` (audit.rs:970); `people_and_text_boundary_tests::removing_the_last_person_removes_its_part_type_and_relationship` (comments.rs:1253) (?); `people_and_text_boundary_tests::unchanged_people_are_byte_identical_and_new_authors_do_not_create_a_people_part` (comments.rs:1276) (?); `public_memory_anchor_boundary_tests::snapshot` (comments.rs:1353); +80 more

#### `PartFs::part_string`  (L498-501, pub)
`pub fn part_string(&self, name: &str) -> Option<String>`

- **Does:** Read a part as a UTF-8 string.
- **Calls (crate):** `PartFs::part_bytes` (opc/mod.rs:493) L499
- **Calls (std/external/unresolved):** `.into_owned`, `.map`, `String::from_utf8_lossy`
- **Called by:** `drop_comments_in_new_parts` (append.rs:450); `carry_styles_and_numbering` (append.rs:861); `insert_lists` (append.rs:1159); `public_memory_append_owner_boundary_tests::public_append_keeps_source_property_history_and_fresh_annotation_owners_at_every_join_boundary` (append.rs:1511) (?); `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1594) (?); `changes_in` (changes.rs:302); `CommentFamily::load` (comments.rs:429); `CommentFamily::store_people` (comments.rs:977); `related_part` (comments.rs:1027); `package_para_ids` (comments.rs:1119); `people_and_text_boundary_tests::people_updates_preserve_existing_presence_and_add_each_remaining_author_once` (comments.rs:1217) (?); `people_and_text_boundary_tests::unchanged_people_are_byte_identical_and_new_authors_do_not_create_a_people_part` (comments.rs:1276) (?); +165 more

#### `PartFs::set_part`  (L510-523, pub)
`pub fn set_part(&mut self, name: &str, data: Vec<u8>)`

- **Does:** `PartFS.setPart(name, data)` — replace or add a part.
- **Calls (crate):** `rels_path_to_part_name` (opc/mod.rs:343) L511; `check_opc_attributes` (opc/mod.rs:369) L512; `read_relationships` (opc/mod.rs:214) L513; `norm` (opc/mod.rs:120) L518,522
- **Calls (std/external/unresolved):** `.insert`, `.is_empty`, `.is_ok`, `.set_part`
- **Called by:** `append_documents` (append.rs:152) (?); `drop_comments_in_new_parts` (append.rs:450); `add_related_part` (append.rs:790); `carry_styles_and_numbering` (append.rs:861); `merge_part_namespaces` (append.rs:1081); `insert_lists` (append.rs:1159); `tests::free_sibling_numbers_a_taken_name` (append.rs:1366) (?); `public_memory_append_owner_boundary_tests::package` (append.rs:1482) (?); `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1594) (?); `public_memory_spacer_ownership_tests::package` (audit.rs:964) (?); `CommentFamily::store` (comments.rs:798); `CommentFamily::store_people` (comments.rs:977); +166 more

#### `PartFs::remove_part`  (L527-529, pub)
`pub fn remove_part(&mut self, name: &str)`

- **Does:** Remove a part (no-op when absent).
- **Calls (crate):** `norm` (opc/mod.rs:120) L528
- **Calls (std/external/unresolved):** `.remove`
- **Called by:** `CommentFamily::store` (comments.rs:798); `CommentFamily::store_people` (comments.rs:977); `remove_family_part` (comparer/comments.rs:956); `deeper_boundary_tests::package` (comparer/comments.rs:1838) (?); `coverage_boundary_tests::package` (document_comparer.rs:8799) (?); `coverage_boundary_tests::custom_property_adoption_refuses_unrelated_collision_and_malformed_xml` (document_comparer.rs:9927) (?); `coverage_boundary_tests::empty_missing_and_non_xml_package_parts_are_safe_noops` (document_comparer.rs:9962) (?); `coverage_boundary_tests::header_adoption_refuses_unresolvable_references_and_missing_story_structure` (document_comparer.rs:10593) (?); `coverage_boundary_tests::tracked_change_admission_scans_each_xml_story_and_refuses_binary_or_non_xml_carriers` (document_comparer.rs:11260) (?); `coverage_package_source_matrix_tests::package` (document_comparer.rs:11492) (?); `adopted_story_relationship_owner_tests::adopted_root_and_nested_stories_keep_colliding_media_relationships_resolvable` (document_comparer.rs:12527) (?); `deeper_boundary_tests::paragraph_styles_missing_and_rootless_parts` (edit.rs:6974) (?); +4 more

#### `PartFs::parts`  (L532-536, pub)
`pub fn parts(&self) -> Vec<String>`

- **Does:** Enumerate all part names (docxodus style, no leading slash), sorted.
- **Calls (crate):** `denorm` (opc/mod.rs:325) L533
- **Calls (std/external/unresolved):** `.collect`, `.keys`, `.map`, `.sort`
- **Called by:** `append_documents` (append.rs:152) (?); `drop_comments_in_new_parts` (append.rs:450); `public_memory_append_owner_boundary_tests::snapshot` (append.rs:1493); `public_memory_spacer_ownership_tests::snapshot` (audit.rs:970); `package_para_ids` (comments.rs:1119); `public_memory_anchor_boundary_tests::snapshot` (comments.rs:1353); `public_option_source_ownership_tests::strict_and_transitional_packages_preserve_complete_authored_body_ownership` (comparer/cross_para.rs:3034) (?); `reconcile_part_relationships` (comparer/parts.rs:479); `carry_with_own_relationships` (comparer/parts.rs:627); `tests::carried_external_links_and_missing_internal_targets_do_not_create_media` (comparer/parts.rs:949) (?); `coverage_recursive_part_ownership::recursive_xml_parts_keep_each_relationship_local_and_each_source_unchanged` (comparer/parts.rs:1067) (?); `adopt_revised_header_footer` (document_comparer.rs:5166); +33 more

#### `PartFs::to_zip`  (L548-591, pub)
`pub fn to_zip(&self) -> Result<Vec<u8>, OpcError>`

- **Does:** Serialize the package back to zip bytes.
- **Calls (crate):** `part_name_to_rels_path` (opc/mod.rs:331) L569; `rank` (ambiguous, 4 candidates) L580
- **Calls (std/external/unresolved):** `.as_ref`, `.chain`, `.cmp`, `.collect`, `.compression_level`, `.compression_method`, `.copied`, `.finish`, `.get`, `.into_inner`, `.iter`, `.last_modified_time`, `.map`, `.sort_by`, `.start_file`, `.strip_prefix`, `.to_xml`, `.unwrap_or`, `.write_all`, `Cow::Borrowed`, `Cow::Owned`, `Cursor::new`, `Ok`, `SimpleFileOptions::default`, `Some`, `Vec::new`, `ZipEntry::Part`, `ZipEntry::Rels`, `ZipWriter::new`, `zip::DateTime::default`
- **Called by:** `append_documents` (append.rs:152) (?); `public_memory_append_owner_boundary_tests::public_append_keeps_source_property_history_and_fresh_annotation_owners_at_every_join_boundary` (append.rs:1511) (?); `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1594) (?); `public_memory_spacer_ownership_tests::spacer_audit_respects_authored_sections_page_breaks_and_table_cell_ownership` (audit.rs:983) (?); `resolve` (changes.rs:349) (?); `public_memory_anchor_boundary_tests::block_and_point_comments_preserve_exact_unicode_anchor_context_and_package` (comments.rs:1371) (?); `authored_thread_reference_contract_tests::cyclic_and_missing_parent_metadata_terminate_without_discarding_comment_owners` (comments.rs:1548) (?); `authored_thread_reference_contract_tests::paragraph_id_scan_reserves_only_complete_hex_ids_from_other_xml_parts` (comments.rs:1580) (?); `public_option_source_ownership_tests::public_boundary_shifts_preserve_all_source_owners_with_lexical_options` (comparer/cross_para.rs:2705) (?); `public_option_source_ownership_tests::public_comparison_options_preserve_reordered_paragraphs_formats_and_note_owners` (comparer/cross_para.rs:2775) (?); `public_option_source_ownership_tests::public_reordered_bulk_paragraphs_preserve_sources_on_move_thrash_boundaries` (comparer/cross_para.rs:2938) (?); `public_option_source_ownership_tests::strict_and_transitional_packages_preserve_complete_authored_body_ownership` (comparer/cross_para.rs:3034) (?); +58 more

#### `PartFs::resolve_rel_target`  (L600-605, pub)
`pub fn resolve_rel_target(&self, source_part: &str, rel_target: &str) -> String`

- **Does:** `resolveRelTarget(sourcePart, relTarget)` — the part a rel target names, relative to its source part, as a canonical part name: no leading slash (the form `parts()` returns) and `.`/`..` segments resolved.
- **Calls (crate):** `unescape_xml_text` (xmllinq/parse.rs:704) L603; `resolve_rel_target` (opc/mod.rs:426) L604
- **Called by:** `related` (append.rs:760); `related_part` (comments.rs:1027); `dest_rid_part_bytes` (comparer/parts.rs:249); `reconcile_dangling_relationships` (comparer/parts.rs:273) (?); `carry_relationship` (comparer/parts.rs:517); `carry_with_own_relationships` (comparer/parts.rs:627) (?); `tests::carried_images_reuse_only_matching_bytes_content_type_and_internal_type` (comparer/parts.rs:870) (?); `coverage_recursive_part_ownership::recursive_xml_parts_keep_each_relationship_local_and_each_source_unchanged` (comparer/parts.rs:1067) (?); `coverage_recursive_part_ownership::repeated_references_share_one_carried_local_relationship_without_merging_owners` (comparer/parts.rs:1190) (?); `load_embedded_fonts` (convert/font_table.rs:205); `part_xml_by_rel_kind` (convert/mod.rs:8472); `story_parts_xml` (convert/mod.rs:21249); +33 more

#### `PartFs::content_type_for`  (L608-613, pub)
`pub fn content_type_for(&self, name: &str) -> Option<String>`

- **Does:** `contentTypeFor(part)`.
- **Calls (crate):** `PartFs::content_type_for` (opc/mod.rs:608) (by name) L609; `norm` (opc/mod.rs:120) L611
- **Calls (std/external/unresolved):** `.map`, `.to_string`
- **Called by:** `deeper_boundary_tests::auxiliary_rootless_and_seeded_relationship_boundaries` (comparer/comments.rs:2400) (?); `reconcile_dangling_relationships` (comparer/parts.rs:273) (?); `carry_relationship` (comparer/parts.rs:517); `tests::carried_images_reuse_only_matching_bytes_content_type_and_internal_type` (comparer/parts.rs:870) (?); `coverage_recursive_part_ownership::recursive_xml_parts_keep_each_relationship_local_and_each_source_unchanged` (comparer/parts.rs:1067) (?); `coverage_recursive_part_ownership::repeated_references_share_one_carried_local_relationship_without_merging_owners` (comparer/parts.rs:1190) (?); `coverage_boundary_tests::final_header_footer_adoption_respects_existing_body_owners_and_cached_donor_parts` (document_comparer.rs:10052) (?); `coverage_boundary_tests::header_adoption_preserves_colliding_media_and_uses_correct_content_types` (document_comparer.rs:10403) (?); `coverage_boundary_tests::separator_media_collisions_keep_a_resolvable_relative_relationship` (document_comparer.rs:10832) (?); `Transaction::image_paragraph` (edit/images.rs:143) (?); `Opened::summary` (inspect.rs:545) (?); `assemble` (markdown/package.rs:97) (?); +8 more

#### `PartFs::add_content_type_override`  (L616-620, pub)
`pub fn add_content_type_override(&mut self, part_name: &str, content_type: &str)`

- **Does:** Add an Override entry to `[Content_Types].xml`.
- **Calls (crate):** `norm` (opc/mod.rs:120) L619
- **Calls (std/external/unresolved):** `.add_override`
- **Called by:** `add_related_part` (append.rs:790); `public_memory_append_owner_boundary_tests::package` (append.rs:1482) (?); `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1594) (?); `CommentFamily::store` (comments.rs:798); `people_and_text_boundary_tests::people` (comments.rs:1207); `public_memory_anchor_boundary_tests::block_and_point_comments_preserve_exact_unicode_anchor_context_and_package` (comments.rs:1371) (?); `authored_thread_reference_contract_tests::family` (comments.rs:1504) (?); `install_parts_from` (comparer/comments.rs:937); `union_comments_xml` (comparer/comments.rs:1190); `public_option_source_ownership_tests::public_comparison_options_preserve_reordered_paragraphs_formats_and_note_owners` (comparer/cross_para.rs:2775) (?); `coverage_real_revision_merge_matrix_tests::public_compare_multi_original_blank_tail_recovers_complete_clean_sources::{fn package}` (comparer/finalize.rs:26097) (?); `coverage_real_revision_merge_matrix_tests::organic_package` (comparer/finalize.rs:27913) (?); +37 more

#### `PartFs::add_content_type_default`  (L623-625, pub)
`pub fn add_content_type_default(&mut self, ext: &str, content_type: &str)`

- **Does:** Add a Default extension mapping to `[Content_Types].xml`.
- **Calls (std/external/unresolved):** `.add_default`
- **Called by:** `adopt_revised_header_footer` (document_comparer.rs:5166); `coverage_boundary_tests::separator_media_collisions_keep_a_resolvable_relative_relationship` (document_comparer.rs:10832) (?); `adopted_story_relationship_owner_tests::adopted_root_and_nested_stories_keep_colliding_media_relationships_resolvable` (document_comparer.rs:12527) (?); `Transaction::image_paragraph` (edit/images.rs:143) (?); `<PartRelate as Relate>::picture` (markdown/package.rs:1497); `PartFs::adopt_missing_content_types` (opc/mod.rs:631); `tests::parts_without_a_content_type_adopt_the_sources_default` (opc/mod.rs:1088) (?); `tests::an_upper_case_extension_is_already_typed_by_the_lower_case_default` (opc/mod.rs:1114) (?); `tests::content_types_survive_round_trips_unescaped` (opc/mod.rs:1191) (?)

#### `PartFs::adopt_missing_content_types`  (L631-665, pub)
`pub fn adopt_missing_content_types(&mut self, sources: &[&PartFs])`

- **Does:** Give every part that has no content type the one its source declares: the source's entry for the same part name, else the source's Default for the extension (case-insensitive, as OPC matches extensions).
- **Calls (crate):** `PartFs::parts` (opc/mod.rs:532) L632; `PartFs::content_type_for` (opc/mod.rs:608) L633; `norm` (opc/mod.rs:120) L653; `PartFs::add_content_type_override` (opc/mod.rs:616) L656; `PartFs::add_content_type_default` (opc/mod.rs:623) L660
- **Calls (std/external/unresolved):** `.clone`, `.cloned`, `.contains`, `.eq_ignore_ascii_case`, `.filter`, `.find`, `.get`, `.is_some`, `.iter`, `.map`, `.rsplit_once`, `by_ext`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6501) (?); `tests::parts_without_a_content_type_adopt_the_sources_default` (opc/mod.rs:1088) (?); `tests::an_upper_case_extension_is_already_typed_by_the_lower_case_default` (opc/mod.rs:1114) (?)

#### `PartFs::remove_content_type_override`  (L668-670, pub)
`pub fn remove_content_type_override(&mut self, part_name: &str)`

- **Does:** Remove an Override entry from `[Content_Types].xml` (no-op when absent).
- **Calls (crate):** `norm` (opc/mod.rs:120) L669
- **Calls (std/external/unresolved):** `.remove`
- **Called by:** `CommentFamily::store` (comments.rs:798); `CommentFamily::store_people` (comments.rs:977); `remove_family_part` (comparer/comments.rs:956); `core_properties` (markdown/package.rs:1248); `PartFs::remove_untargeted_part` (opc/mod.rs:717); `remove_part` (revision_processor/comments.rs:279); `scrub_properties` (scrub.rs:357)

#### `PartFs::remove_relationships_by_type`  (L675-681, pub)
`pub fn remove_relationships_by_type(&mut self, source_part: &str, rel_type: &str)`

- **Does:** Remove every relationship of `source_part` with the given type.
- **Calls (crate):** `PartFs::part_rels_key` (opc/mod.rs:762) L676
- **Calls (std/external/unresolved):** `.cloned`, `.get_mut`, `.retain`
- **Called by:** `CommentFamily::store` (comments.rs:798); `CommentFamily::store_people` (comments.rs:977); `remove_family_part` (comparer/comments.rs:956); `ensure_factory_package_chrome` (document_comparer.rs:3402); `prune` (markdown/package.rs:426); `remove_part` (revision_processor/comments.rs:279)

#### `PartFs::remove_package_relationships_by_type`  (L685-687, pub)
`pub fn remove_package_relationships_by_type(&mut self, rel_type: &str)`

- **Does:** Remove every package-level relationship (`_rels/.rels`) of the given type.
- **Calls (std/external/unresolved):** `.retain`
- **Called by:** `core_properties` (markdown/package.rs:1248); `scrub_properties` (scrub.rs:357)

#### `PartFs::remove_related_part`  (L694-711, pub)
`pub fn remove_related_part(&mut self, source_part: &str, rel_id: &str)`

- **Does:** Remove the relationship `rel_id` of `source_part` and, unless another internal relationship still targets it, the part it names with that part's own relationships and content-type override, then the parts (a header's images) nothing else targets any more.
- **Calls (crate):** `PartFs::part_rels_key` (opc/mod.rs:762) L695; `PartFs::resolve_rel_target` (opc/mod.rs:600) L706; `PartFs::remove_untargeted_part` (opc/mod.rs:717) L710
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.clone`, `.cloned`, `.find`, `.get_mut`, `.get`, `.iter`, `.map`, `.retain`, `Some`
- **Called by:** `prune` (markdown/package.rs:426); `write` (markdown/view_stories.rs:1013); `tests::removing_a_related_part_drops_the_parts_only_it_targeted` (opc/mod.rs:1323) (?); `drop_unreferenced_headers_footers` (revision_processor/word_save.rs:59)

#### `PartFs::remove_untargeted_part`  (L717-753)
`fn remove_untargeted_part(&mut self, part: &str)`

- **Does:** Remove `part`, its relationships and content-type override when no internal relationship of another part targets it, then each part it targeted that is left untargeted.
- **Calls (crate):** `norm` (opc/mod.rs:120) L718; `PartFs::resolve_rel_target` (opc/mod.rs:600) L731,745; `PartFs::remove_content_type_override` (opc/mod.rs:668) L749; `PartFs::remove_untargeted_part` (opc/mod.rs:717) L751
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.as_str`, `.chain`, `.collect`, `.contains_key`, `.filter`, `.iter`, `.map`, `.remove`, `.unwrap_or_default`, `Some`, `std::iter::once`
- **Macros:** `format!`
- **Called by:** `PartFs::remove_related_part` (opc/mod.rs:694); `PartFs::remove_untargeted_part` (opc/mod.rs:717)

#### `PartFs::read_rels_for`  (L756-758, pub)
`pub fn read_rels_for(&self, part_name: &str) -> Option<&Relationships>`

- **Does:** `readRelsFor(part)` — the relationships of a part, if any.
- **Calls (crate):** `PartFs::part_rels_key` (opc/mod.rs:762) L757
- **Calls (std/external/unresolved):** `.get`
- **Called by:** `related` (append.rs:760); `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1594) (?); `related_part` (comments.rs:1027); `people_and_text_boundary_tests::removing_the_last_person_removes_its_part_type_and_relationship` (comments.rs:1253) (?); `install_parts_from` (comparer/comments.rs:937); `union_comments_xml` (comparer/comments.rs:1190); `deeper_boundary_tests::auxiliary_rootless_and_seeded_relationship_boundaries` (comparer/comments.rs:2400) (?); `public_option_source_ownership_tests::strict_and_transitional_packages_preserve_complete_authored_body_ownership` (comparer/cross_para.rs:3034) (?); `dest_rid_part_bytes` (comparer/parts.rs:249); `reconcile_dangling_relationships` (comparer/parts.rs:273); `carry_relationship` (comparer/parts.rs:517); `carry_with_own_relationships` (comparer/parts.rs:627) (?); +60 more

#### `PartFs::part_rels_key`  (L762-768)
`fn part_rels_key(&self, part_name: &str) -> Option<&String>`

- **Does:** Preserve upstream's case-insensitive, normalized lookup and deterministic spelling choice when more than one stored owner has the same identity. _(from a code comment)_
- **Calls (crate):** `resolve_rel_target` (opc/mod.rs:426) L763,766
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`, `.filter`, `.keys`, `.min`
- **Called by:** `PartFs::remove_relationships_by_type` (opc/mod.rs:675); `PartFs::remove_related_part` (opc/mod.rs:694); `PartFs::read_rels_for` (opc/mod.rs:756)

#### `PartFs::add_document_relationship`  (L772-779, pub)
`pub fn add_document_relationship( &mut self, source_part: &str, rel_type: &str, target: &str, ) -> String`

- **Does:** `addDocumentRelationship(...)` — add a relationship to a part, returning the new relationship id.
- **Calls (crate):** `.add` (ambiguous, 5 candidates) L778; `PartFs::part_rels_mut` (opc/mod.rs:794) L778
- **Called by:** `add_related_part` (append.rs:790); `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1594) (?); `CommentFamily::store` (comments.rs:798); `people_and_text_boundary_tests::people` (comments.rs:1207); `public_memory_anchor_boundary_tests::block_and_point_comments_preserve_exact_unicode_anchor_context_and_package` (comments.rs:1371) (?); `authored_thread_reference_contract_tests::family` (comments.rs:1504) (?); `install_parts_from` (comparer/comments.rs:937); `union_comments_xml` (comparer/comments.rs:1190); `deeper_boundary_tests::auxiliary_rootless_and_seeded_relationship_boundaries` (comparer/comments.rs:2400) (?); `public_option_source_ownership_tests::public_comparison_options_preserve_reordered_paragraphs_formats_and_note_owners` (comparer/cross_para.rs:2775) (?); `coverage_real_revision_merge_matrix_tests::public_compare_multi_original_blank_tail_recovers_complete_clean_sources::{fn package}` (comparer/finalize.rs:26097) (?); `coverage_real_revision_merge_matrix_tests::organic_package` (comparer/finalize.rs:27913) (?); +48 more

#### `PartFs::package_relationships`  (L782-784, pub)
`pub fn package_relationships(&self) -> &Relationships`

- **Does:** The package's own relationships (`_rels/.rels`).
- **Calls:** none
- **Called by:** `public_option_source_ownership_tests::strict_and_transitional_packages_preserve_complete_authored_body_ownership` (comparer/cross_para.rs:3034) (?); `custom_props_part` (document_comparer.rs:6173); `tests::public_and_package_relationship_readers_agree` (opc/mod.rs:882) (?); `tests::migration_skips_zip_directories_and_keeps_a_missing_package_relationship_scope_empty` (opc/mod.rs:1378) (?)

#### `PartFs::add_package_relationship`  (L787-789, pub)
`pub fn add_package_relationship(&mut self, rel_type: &str, target: &str) -> String`

- **Does:** Add a package-level relationship, returning its id.
- **Calls (std/external/unresolved):** `.add`
- **Called by:** `merge_custom_properties` (document_comparer.rs:6188); `coverage_boundary_tests::package` (document_comparer.rs:8799) (?); `coverage_boundary_tests::custom_property_merge_is_case_insensitive_original_wins_and_new_ids_are_unique` (document_comparer.rs:9898) (?); `coverage_boundary_tests::custom_property_adoption_refuses_unrelated_collision_and_malformed_xml` (document_comparer.rs:9927) (?); `coverage_package_source_matrix_tests::package` (document_comparer.rs:11492) (?); `core_properties` (markdown/package.rs:1248); `public_block_control_ownership_tests::package` (revision_processor.rs:4385) (?)

#### `PartFs::part_rels_mut`  (L794-801)
`fn part_rels_mut(&mut self, source_part: &str) -> &mut Relationships`

- **Does:** A part's relationships, created empty when missing.
- **Calls (crate):** `norm` (opc/mod.rs:120) L797; `Relationships::new` (opc/mod.rs:41) L799
- **Calls (std/external/unresolved):** `.entry`, `.insert`, `.into_mut`
- **Called by:** `PartFs::add_document_relationship` (opc/mod.rs:772); `PartFs::add_document_relationship_external` (opc/mod.rs:806); `PartFs::set_rel_target_mode_external` (opc/mod.rs:821)

#### `PartFs::add_document_relationship_external`  (L806-818, pub)
`pub fn add_document_relationship_external( &mut self, source_part: &str, rel_type: &str, target: &str, ) -> String`

- **Does:** Add a relationship with `TargetMode="External"` (absolute-URI targets are ILLEGAL for the default Internal mode — strict packaging layers like Word's reject the package without this).
- **Calls (crate):** `PartFs::part_rels_mut` (opc/mod.rs:794) L812; `.add` (ambiguous, 5 candidates) L813
- **Calls (std/external/unresolved):** `.find`, `.iter_mut`, `.to_string`, `Some`
- **Called by:** `reconcile_dangling_relationships` (comparer/parts.rs:273); `carry_relationship` (comparer/parts.rs:517); `tests::carried_images_reuse_only_matching_bytes_content_type_and_internal_type` (comparer/parts.rs:870) (?); `tests::carried_external_links_and_missing_internal_targets_do_not_create_media` (comparer/parts.rs:949) (?); `coverage_boundary_tests::relationship_agreement_refuses_missing_mismatched_and_dangling_targets` (document_comparer.rs:9701) (?); `coverage_boundary_tests::carry_relationships_reuses_minted_id_and_refuses_unresolved_references` (document_comparer.rs:9745) (?); `coverage_boundary_tests::notes_copy_preserves_media_and_escapes_external_links_without_io` (document_comparer.rs:9831) (?); `coverage_boundary_tests::header_adoption_preserves_colliding_media_and_uses_correct_content_types` (document_comparer.rs:10403) (?); `comparer_relationship_contract_tests::referenced_relationship_agreement_requires_matching_type_mode_and_payload_or_external_target` (document_comparer.rs:11866) (?); `comparer_relationship_contract_tests::repeated_hyperlinks_share_one_carried_relationship_and_keep_independent_source_text_and_format` (document_comparer.rs:11957) (?); `relate` (markdown/from_docx/drawing.rs:1400); `<PartRelate as Relate>::hyperlink` (markdown/package.rs:1489); +3 more

#### `PartFs::set_rel_target_mode_external`  (L821-826, pub)
`pub fn set_rel_target_mode_external(&mut self, source_part: &str, rel_id: &str)`

- **Does:** Mark an existing relationship of `source_part` as External (test aid).
- **Calls (crate):** `PartFs::part_rels_mut` (opc/mod.rs:794) L822
- **Calls (std/external/unresolved):** `.find`, `.iter_mut`, `.to_string`, `Some`
- **Called by:** `coverage_boundary_tests::relationship_agreement_refuses_missing_mismatched_and_dangling_targets` (document_comparer.rs:9701) (?)

#### `PartFs::main_document_part`  (L829-833, pub)
`pub fn main_document_part(&self) -> Option<String>`

- **Does:** The main document part name (docxodus style).
- **Calls (crate):** `denorm` (opc/mod.rs:325) L832
- **Calls (std/external/unresolved):** `.get_by_type`, `.map`
- **Called by:** `tests::admission_and_partfs_agree_on_decoded_names_and_duplicates` (admission.rs:1052) (?); `main_part` (append.rs:323); `main_part` (changes.rs:398); `fix_up_drawing_ids_in_package` (comparer/fixups.rs:121); `reconcile_dangling_relationships` (comparer/parts.rs:273); `with_layout` (convert/mod.rs:769) (?); `main_rel_part` (convert/mod.rs:21274); `comments_spacing_tests::comments_chart_drawing_is_not_also_a_flow_box` (convert/mod.rs:45305) (?); `comments_spacing_tests::comments_lists_collapse_between_siblings` (convert/mod.rs:45373) (?); `comments_spacing_tests::potpourri_listnumber_gets_numbering_hanging` (convert/mod.rs:45455) (?); `comments_spacing_tests::comments_listbullet_gets_numbering_hanging` (convert/mod.rs:45514) (?); `comments_spacing_tests::sd_2517_pagebreak_census` (convert/mod.rs:45699) (?); +30 more

## Test functions (26)

- `tests::opc_attribute_budget_bounds_upstream_duplicate_checks` (L843): _no doc comment_
- `tests::public_and_package_relationship_readers_agree` (L882): _no doc comment_
- `tests::package_with_header_rels` (L928): _no doc comment_
- `tests::package_with_document_rels` (L950): A package whose `word/_rels/document.xml.rels` holds `rels`.
- `tests::a_repeated_relationship_id_is_repaired_when_identical_and_refused_otherwise` (L976): Word refuses a package that repeats a relationship Id in one part (docxide case8's fontTable rels, opened in Word 16: unreadable).
- `tests::the_package_is_written_in_the_source_order_then_new_parts_by_name` (L1015): _no doc comment_
- `tests::entries_carry_no_wall_clock_time` (L1044): _no doc comment_
- `tests::part_names_carry_exactly_one_leading_slash` (L1062): _no doc comment_
- `tests::parts_without_a_content_type_adopt_the_sources_default` (L1088): _no doc comment_
- `tests::an_upper_case_extension_is_already_typed_by_the_lower_case_default` (L1114): _no doc comment_
- `tests::migration_keeps_canonical_metadata_and_decodes_literal_entities_once` (L1133): _no doc comment_
- `tests::rel_attributes_survive_round_trips_unescaped` (L1169): _no doc comment_
- `tests::content_types_survive_round_trips_unescaped` (L1191): _no doc comment_
- `tests::rel_targets_resolve_to_canonical_part_names` (L1207): _no doc comment_
- `tests::rel_targets_are_relative_to_the_source_folder` (L1234): _no doc comment_
- `tests::rels_paths_name_their_owning_part` (L1259): _no doc comment_
- `tests::a_new_rels_part_starts_at_rid1` (L1276): A part's first relationship is `rId1`, as Word numbers them: the dependency's `Default` relationships start from `rId0`.
- `tests::writing_a_rels_part_replaces_the_parts_relationships` (L1293): _no doc comment_
- `tests::removing_a_related_part_drops_the_parts_only_it_targeted` (L1323): Dropping a header takes the media only it used with it; media another part still uses stays, and so does a part reached in a loop.
- `tests::migration_skips_zip_directories_and_keeps_a_missing_package_relationship_scope_empty` (L1378): _no doc comment_
- `tests::migration_refuses_missing_required_metadata_attributes_in_package_loading` (L1401): _no doc comment_
- `tests::a_record_local_default_namespace_cannot_bypass_metadata_validation` (L1420): _no doc comment_
- `tests::legacy_media_package` (L1448): A minimal package whose one media entry carries the raw name bytes and flag `media` names, with the document relationship pointing at the CP437 spelling of `\xc3\xa9` (`├⌐`), as a legacy writer left it.
- `tests::legacy_cp437_entry_names_keep_resolving_their_relationships` (L1456): _no doc comment_
- `tests::decoded_part_name_aliases_are_refused_not_overwritten` (L1479): _no doc comment_
- `tests::unicode_path_extra_fields_override_entry_names_as_zip8_did` (L1523): _no doc comment_
