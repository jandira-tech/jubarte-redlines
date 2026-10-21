# Functions and methods in `src/opc/mod.rs`

48 non-test, 46 test. Part of the [symbol manifest](../README.md).

#### `Relationships::new`  (L41-43, pub)
`pub fn new() -> Self`

- **Does:** An empty collection whose generated IDs start at `rId1`.
- **Calls (std/external/unresolved):** `Self`, `oxml_opc::Relationships::new`
- **Called by:** `read_relationships` (opc/mod.rs:265); `read_package` (opc/mod.rs:325); `PartFs::remove_part` (opc/mod.rs:710); `PartFs::part_rels_mut` (opc/mod.rs:1025)

#### `Relationships::from_xml`  (L50-52, pub)
`pub fn from_xml(xml: &[u8]) -> Result<Self, OpcError>`

- **Does:** Read a relationships part with the package reader's refusal contract.
- **Calls (crate):** `read_relationships` (opc/mod.rs:265) L51
- **Called by:** `tests::public_and_package_relationship_readers_agree` (opc/mod.rs:1135)

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
- **Called by:** `admit_input` (document_comparer.rs:4907); `read_package` (opc/mod.rs:325); `check_opc_attributes` (opc/mod.rs:508); `PartFs::to_zip` (opc/mod.rs:749)

#### `refusal_message`  (L88-99, pub)
`pub fn refusal_message(docx: &[u8], err: &OpcError) -> String`

- **Does:** The message a text surface prints for an [`OpcError`].
- **Calls (crate):** `open_archive_unchecked` (opc/archive.rs:20) (by name) L90; `duplicate_parts` (opc/archive.rs:743) (by name) L92; `ZipDefect::refusal` (opc/archive.rs:366) (by name) L95
- **Calls (std/external/unresolved):** `.central_directory_start`, `.into_iter`, `.map`, `.next`, `.to_string`, `.unwrap_or_else`, `Cursor::new`
- **Called by:** `tests::a_duplicate_refusal_reads_as_the_admission_detail` (admission.rs:2617); `docx_to_markdown` (markdown/mod.rs:651); `assemble` (markdown/package.rs:103); `scrub` (scrub.rs:109)

#### `drop_repeated_relationships`  (L118-145)
`fn drop_repeated_relationships(rels: &mut Relationships) -> Result<Vec<String>, OpcError>`

- **Does:** Exactly one leading slash: callers build `/{part}` from names that may already be absolute, and a `//word/…` part name is invalid OPC.
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.contains`, `.get`, `.insert`, `.into_iter`, `.len`, `.next`, `.push`, `.retain`, `.unwrap_or`, `Err`, `HashMap::new`, `Ok`, `Vec::new`, `Vec::with_capacity`
- **Called by:** `PartFs::open_within` (opc/mod.rs:593)

#### `norm`  (L147-149)
`fn norm(name: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.trim_start_matches`
- **Macros:** `format!`
- **Called by:** `scan_family_faces` (convert/font.rs:2281) (?); `family_faces_in_with` (convert/font.rs:2473) (?); `font_name_index` (convert/font.rs:2593) (?); `catalogue_paints_family` (convert/font.rs:3481) (?); `read_package` (opc/mod.rs:325); `resolve_rel_target` (opc/mod.rs:549); `PartFs::set_part` (opc/mod.rs:661); `PartFs::content_type_for` (opc/mod.rs:837); `PartFs::add_content_type_override` (opc/mod.rs:845); `PartFs::adopt_missing_content_types` (opc/mod.rs:860); `PartFs::remove_content_type_override` (opc/mod.rs:897); `PartFs::part_rels_mut` (opc/mod.rs:1025)

#### `identity`  (L157-173)
`fn identity(name: &str) -> String`

- **Does:** The OPC identity of a part or relationship-owner name: resolved like a relationship target (leading slashes, `.`/`..` segments) and compared under ASCII case (ECMA-376 Part 2 §6.2.2.3 M1.12 — `word/Document.xml` IS `word/document.xml`, which is how Word opens probes p3–p5 of ZIP-15/#600).
- **Calls (crate):** `resolve_rel_target` (opc/mod.rs:549) L167
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.make_ascii_lowercase`, `.split`, `.starts_with`, `.to_ascii_lowercase`
- **Called by:** `duplicate_parts` (opc/archive.rs:743); `read_package` (opc/mod.rs:325); `PartFs::part_bytes` (opc/mod.rs:628); `PartFs::set_part` (opc/mod.rs:661); `PartFs::remove_part` (opc/mod.rs:710); `PartFs::remove_untargeted_part` (opc/mod.rs:947); `PartFs::part_rels_key` (opc/mod.rs:997); `PartFs::part_rels_mut` (opc/mod.rs:1025); `tests::identity_resolves_like_a_relationship_target` (opc/mod.rs:2009)

#### `relative_rel_target`  (L180-188, pub)
`pub fn relative_rel_target(source_part: &str, target_part: &str) -> String`

- **Does:** The relationship `Target` value that reaches `target_part` from `source_part`: relative to the source's folder when the target sits under it, absolute (`/media/image.bin`) otherwise.
- **Calls (std/external/unresolved):** `.and_then`, `.is_empty`, `.map_or`, `.rsplit_once`, `.strip_prefix`, `.to_string`, `.trim_start_matches`
- **Macros:** `format!`
- **Called by:** `add_related_part` (append.rs:796); `CommentFamily::store` (comments.rs:854); `install_parts_from` (comparer/comments.rs:937); `union_comments_xml` (comparer/comments.rs:1190); `reconcile_dangling_relationships` (comparer/parts.rs:273); `carry_relationship` (comparer/parts.rs:517); `ensure_factory_package_chrome` (document_comparer.rs:3402); `adopt_revised_header_footer` (document_comparer.rs:5174); `adopt_b_notes_when_a_lacks_separators` (document_comparer.rs:5871); `repair_missing_core_relationships` (document_comparer.rs:6258); `adopted_story_relationship_owner_tests::adopted_root_and_nested_stories_keep_colliding_media_relationships_resolvable` (document_comparer.rs:12648); `Transaction::image_paragraph` (edit/images.rs:143); +10 more

#### `metadata_records`  (L198-263)
`fn metadata_records( xml: &[u8], root: &str, namespace: &str, tags: &[&str], ) -> Result<Vec<Vec<u8>>, OpcError>`

- **Does:** Select the same unprefixed metadata records as the legacy adapter, folding explicit closes, then let upstream decode and validate each record.
- **Calls (crate):** `Dom::attributes` (xmllinq/mod.rs:899) (by name) L211
- **Calls (std/external/unresolved):** `.as_bytes`, `.as_ref`, `.copied`, `.find`, `.into_inner`, `.iter`, `.map_err`, `.name`, `.push_attribute`, `.push`, `.read_event_into`, `.take`, `.to_owned`, `.with_checks`, `.write_event`, `BytesEnd::new`, `BytesStart::new`, `Event::Empty`, `Event::End`, `Event::Start`, `Ok`, `Some`, `Vec::new`, `fold`, `matched`, `quick_xml_041::Reader::from_reader`, `quick_xml_041::Writer::new`
- **Macros:** `matches!`
- **Called by:** `read_relationships` (opc/mod.rs:265); `read_content_types` (opc/mod.rs:297)

#### `read_relationships`  (L265-295)
`fn read_relationships(xml: &[u8]) -> Result<Relationships, OpcError>`

- **Does:** _no doc comment_
- **Calls (crate):** `Relationships::new` (opc/mod.rs:41) L266; `metadata_records` (opc/mod.rs:198) L268
- **Calls (std/external/unresolved):** `.add_with_id`, `.and_then`, `.filter_map`, `.iter`, `.map`, `.max_by_key`, `.ok`, `.parse`, `.push`, `.strip_prefix`, `Ok`, `Vec::new`, `oxml_opc::Relationships::from_xml`
- **Called by:** `Relationships::from_xml` (opc/mod.rs:50); `read_package` (opc/mod.rs:325); `PartFs::set_part` (opc/mod.rs:661)

#### `read_content_types`  (L297-313)
`fn read_content_types(xml: &[u8]) -> Result<ContentTypes, OpcError>`

- **Does:** _no doc comment_
- **Calls (crate):** `metadata_records` (opc/mod.rs:198) L302
- **Calls (std/external/unresolved):** `.extend`, `ContentTypes::from_xml`, `HashMap::new`, `Ok`
- **Called by:** `read_package` (opc/mod.rs:325); `PartFs::set_part` (opc/mod.rs:661)

#### `read_package`  (L325-445)
`fn read_package(bytes: &[u8], limits: crate::admission::InputLimits) -> Result<PartFs, OpcError>`

- **Does:** Populate parts and local relationships without invoking upstream's whole-package normalization and duplicate-refusal policies ahead of the adapter's repairs.
- **Calls (crate):** `preindex_budget` (admission.rs:399) L326; `open_archive_within` (opc/archive.rs:50) (by name) L334; `duplicate_parts` (opc/archive.rs:743) (by name) L336; `zip_entry_name` (admission.rs:775) L349; `unicode_path_fields` (admission.rs:664) L352; `refused` (opc/mod.rs:76) L359,379; `part_declared_over` (admission.rs:369) L359; `tally_inflated` (admission.rs:411) L392; `check_opc_attributes` (opc/mod.rs:508) L398; `Relationships::new` (opc/mod.rs:41) L404; `read_content_types` (opc/mod.rs:297) L420; `read_relationships` (opc/mod.rs:265) L426,437; `rels_path_to_part_name` (opc/mod.rs:468) L430; `norm` (opc/mod.rs:147) L435,439; `identity` (opc/mod.rs:157) L436,440
- **Calls (std/external/unresolved):** `.and_then`, `.as_str`, `.as_u16`, `.by_index`, `.central_directory_start`, `.clone`, `.cloned`, `.ends_with`, `.eq_ignore_ascii_case`, `.extra_data_fields`, `.find`, `.flags`, `.get`, `.insert`, `.into_iter`, `.is_dir`, `.keys`, `.len`, `.map_err`, `.min`, `.name_raw`, `.next`, `.ok_or_else`, `.read_to_end`, `.remove`, `.saturating_add`, `.saturating_sub`, `.size`, `.take`, `.to_ascii_lowercase`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `Cursor::new`, `Err`, `HashMap::new`, `Ok`, `OpcError::DuplicatePartName`, `OpcError::PartNotFound`, `OpcPackage::new`, `Vec::new`, `std::io::copy`, `std::io::sink`, `u64::try_from`
- **Macros:** `format!`
- **Called by:** `PartFs::open_within` (opc/mod.rs:593)

#### `denorm`  (L447-449)
`fn denorm(name: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.to_string`, `.trim_start_matches`
- **Called by:** `PartFs::parts` (opc/mod.rs:733); `PartFs::main_document_part` (opc/mod.rs:1070)

#### `part_name_to_rels_path`  (L453-460)
`fn part_name_to_rels_path(part_name: &str) -> String`

- **Does:** `.rels` file path (e.g.
- **Calls (std/external/unresolved):** `.rsplit_once`, `.strip_prefix`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `PartFs::to_zip` (opc/mod.rs:749)

#### `rels_path_to_part_name`  (L468-485)
`fn rels_path_to_part_name(name: &str) -> Option<String>`

- **Does:** The part a `.rels` path belongs to (`word/_rels/header1.xml.rels` → `word/header1.xml`; `_rels/.rels` → the package, as `""`), or None when `name` is not a relationships path.
- **Calls (std/external/unresolved):** `.checked_sub`, `.get`, `.is_empty`, `.len`, `.rsplit_once`, `.strip_prefix`, `.strip_suffix`, `.to_ascii_lowercase`, `.to_string`, `.unwrap_or`, `Some`
- **Macros:** `format!`
- **Called by:** `read_package` (opc/mod.rs:325); `PartFs::set_part` (opc/mod.rs:661); `PartFs::remove_part` (opc/mod.rs:710); `tests::rels_paths_name_their_owning_part` (opc/mod.rs:1526)

#### `ZipEntry::len`  (L495-501)
`fn len(&self) -> u64`

- **Does:** The uncompressed byte length of the entry's content.
- **Calls (std/external/unresolved):** `.len`, `.map_or`, `.to_xml`
- **Called by:** none found in src

#### `check_opc_attributes`  (L508-527)
`fn check_opc_attributes(xml: &[u8]) -> Result<(), OpcError>`

- **Does:** Retain the OPC metadata budget independently of the parser upgrade that fixed RUSTSEC-2026-0194/0195. _(from a code comment)_
- **Calls (crate):** `Dom::attributes` (xmllinq/mod.rs:899) (by name) L513; `refused` (opc/mod.rs:76) L515
- **Calls (std/external/unresolved):** `.enumerate`, `.map_err`, `.read_event`, `.with_checks`, `Err`, `Ok`, `quick_xml::Reader::from_reader`, `std::io::Error::new`
- **Called by:** `read_package` (opc/mod.rs:325); `PartFs::set_part` (opc/mod.rs:661); `tests::opc_attribute_budget_bounds_upstream_duplicate_checks` (opc/mod.rs:1085)

#### `resolve_rel_target`  (L549-556, pub(crate))
`pub(crate) fn resolve_rel_target(source_part: &str, rel_target: &str) -> String`

- **Does:** Canonical target resolution shared by package APIs and text story labels.
- **Calls (crate):** `norm` (opc/mod.rs:147) L553
- **Calls (std/external/unresolved):** `.to_string`, `.trim_start_matches`, `OpcPackage::resolve_rel_target`
- **Called by:** `story_roles_all` (debug.rs:2037); `identity` (opc/mod.rs:157); `PartFs::resolve_rel_target` (opc/mod.rs:829)

#### `PartFs::open`  (L585-587, pub)
`pub fn open(bytes: &[u8]) -> Result<Self, OpcError>`

- **Does:** Open a `.docx`/OPC package from raw bytes: read its parts, relationships and `[Content_Types].xml`, normalize their part names to the no-leading-slash form, fold any explicit `<…></…>` closes of empty elements (`<Relationship …></Relationship>` and its `[Content_Types]` siblings) into self-closing ones, decode XML ent...
- **Calls (crate):** `PartFs::open_within` (opc/mod.rs:593) L586; `InputLimits::compare` (admission.rs:87) L586
- **Called by:** `tests::admission_and_partfs_agree_on_decoded_names_and_duplicates` (admission.rs:1452); `tests::duplicate_zip_identities_refuse_with_both_spellings` (admission.rs:2550); `tests::a_duplicate_refusal_reads_as_the_admission_detail` (admission.rs:2617); `open` (append.rs:316); `tests::free_sibling_numbers_a_taken_name` (append.rs:1372); `public_memory_append_owner_boundary_tests::package` (append.rs:1487); `public_memory_append_owner_boundary_tests::public_append_keeps_source_property_history_and_fresh_annotation_owners_at_every_join_boundary` (append.rs:1516); `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1599); `public_memory_spacer_ownership_tests::package` (audit.rs:964); `public_memory_spacer_ownership_tests::spacer_audit_respects_authored_sections_page_breaks_and_table_cell_ownership` (audit.rs:983); `warn_nested_text_boxes` (bin/jubarte.rs:1121); `tolerant` (changes.rs:620); +158 more

#### `PartFs::open_within`  (L593-615, pub(crate))
`pub(crate) fn open_within( bytes: &[u8], limits: crate::admission::InputLimits, ) -> Result<Self, OpcError>`

- **Does:** [`PartFs::open`] under an explicit budget: the entry points that admit an input under their own limits (the comparer's settings) open it under the same budget here, so a looser caller budget still governs and a tighter one has already refused the package.
- **Calls (crate):** `read_package` (opc/mod.rs:325) L597; `drop_repeated_relationships` (opc/mod.rs:118) L599,608
- **Calls (std/external/unresolved):** `.clone`, `.cmp`, `.collect`, `.extend`, `.into_iter`, `.iter_mut`, `.map`, `.sort_by`, `.to_string`, `.trim_start_matches`, `Ok`, `String::new`
- **Called by:** `fix_up_drawing_ids_in_package_within` (comparer/fixups.rs:126); `accept_source_revisions` (document_comparer.rs:4972); `get_revisions` (document_comparer.rs:6293); `main_part_xml` (document_comparer.rs:6486); `docx_has_tracked_changes` (document_comparer.rs:6495); `compare_documents_impl` (document_comparer.rs:6520); `Opened::open_within` (inspect.rs:739); `docx_to_markdown` (markdown/mod.rs:651); `PartFs::open` (opc/mod.rs:585); `tests::opc_attribute_budget_bounds_upstream_duplicate_checks` (opc/mod.rs:1085); `tests::decoded_part_name_aliases_are_refused_not_overwritten` (opc/mod.rs:1746); `bounded_input_tests::refuses` (opc/mod.rs:2183); +1 more

#### `PartFs::repaired_duplicate_ids`  (L621-623, pub)
`pub fn repaired_duplicate_ids(&self) -> &[(String, String)]`

- **Does:** `(part, Id)` of each relationship the source repeated identically, kept once on open (`""`: the package's own relationships).
- **Calls:** none
- **Called by:** `tests::a_repeated_relationship_id_is_repaired_when_identical_and_refused_otherwise` (opc/mod.rs:1229) (?); `tests::a_relationship_repeated_more_than_twice_is_repaired_once` (opc/mod.rs:2156) (?); `check_relationship_integrity` (validate.rs:1548)

#### `PartFs::part_bytes`  (L628-633, pub)
`pub fn part_bytes(&self, name: &str) -> Option<&[u8]>`

- **Does:** `PartFS.partBytes(name)` — raw bytes of a part.
- **Calls (crate):** `identity` (opc/mod.rs:157) L631
- **Calls (std/external/unresolved):** `.get`, `.map`
- **Called by:** `tests::admission_and_partfs_agree_on_decoded_names_and_duplicates` (admission.rs:1452) (?); `main_part` (append.rs:329); `parse` (append.rs:336); `related` (append.rs:766); `free_sibling` (append.rs:777); `carry_styles_and_numbering` (append.rs:867); `public_memory_append_owner_boundary_tests::snapshot` (append.rs:1498); `public_memory_append_owner_boundary_tests::public_append_keeps_source_property_history_and_fresh_annotation_owners_at_every_join_boundary` (append.rs:1516) (?); `public_memory_spacer_ownership_tests::snapshot` (audit.rs:970); `people_and_text_boundary_tests::removing_the_last_person_removes_its_part_type_and_relationship` (comments.rs:1309) (?); `people_and_text_boundary_tests::unchanged_people_are_byte_identical_and_new_authors_do_not_create_a_people_part` (comments.rs:1332) (?); `public_memory_anchor_boundary_tests::snapshot` (comments.rs:1409); +90 more

#### `PartFs::part_string`  (L636-639, pub)
`pub fn part_string(&self, name: &str) -> Option<String>`

- **Does:** Read a part as a UTF-8 string.
- **Calls (crate):** `PartFs::part_bytes` (opc/mod.rs:628) L637
- **Calls (std/external/unresolved):** `.into_owned`, `.map`, `String::from_utf8_lossy`
- **Called by:** `drop_comments_in_new_parts` (append.rs:456); `carry_styles_and_numbering` (append.rs:867); `insert_lists` (append.rs:1165); `public_memory_append_owner_boundary_tests::public_append_keeps_source_property_history_and_fresh_annotation_owners_at_every_join_boundary` (append.rs:1516) (?); `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1599) (?); `audit_stories` (changes.rs:168) (?); `tolerant` (changes.rs:620) (?); `for_each_story` (changes.rs:657); `comment_changes_in` (changes.rs:1003); `CommentFamily::load` (comments.rs:457); `CommentFamily::store_people` (comments.rs:1033); `related_part` (comments.rs:1083); +180 more

#### `PartFs::set_part`  (L661-703, pub)
`pub fn set_part(&mut self, name: &str, data: Vec<u8>)`

- **Does:** `PartFS.setPart(name, data)` — replace or add a part.
- **Calls (crate):** `rels_path_to_part_name` (opc/mod.rs:468) L662; `check_opc_attributes` (opc/mod.rs:508) L663,687; `read_relationships` (opc/mod.rs:265) L664; `identity` (opc/mod.rs:157) L671,674,679,691,698,701; `norm` (opc/mod.rs:147) L673,700; `read_content_types` (opc/mod.rs:297) L688
- **Calls (std/external/unresolved):** `.clone`, `.cloned`, `.eq_ignore_ascii_case`, `.get`, `.insert`, `.is_empty`, `.is_ok`, `.remove`, `.trim_start_matches`, `.unwrap_or_else`
- **Called by:** `append_documents` (append.rs:152) (?); `drop_comments_in_new_parts` (append.rs:456); `add_related_part` (append.rs:796); `carry_styles_and_numbering` (append.rs:867); `merge_part_namespaces` (append.rs:1087); `insert_lists` (append.rs:1165); `tests::free_sibling_numbers_a_taken_name` (append.rs:1372) (?); `public_memory_append_owner_boundary_tests::package` (append.rs:1487) (?); `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1599) (?); `public_memory_spacer_ownership_tests::package` (audit.rs:964) (?); `CommentFamily::store` (comments.rs:854); `CommentFamily::store_people` (comments.rs:1033); +179 more

#### `PartFs::remove_part`  (L710-730, pub)
`pub fn remove_part(&mut self, name: &str)`

- **Does:** Remove a part (no-op when absent), resolved under OPC identity: an alias removes the stored part, a `.rels` name removes the owner's relationships, and the `[Content_Types].xml` identity clears the content types.
- **Calls (crate):** `rels_path_to_part_name` (opc/mod.rs:468) L711; `Relationships::new` (opc/mod.rs:41) L713; `identity` (opc/mod.rs:157) L714,727
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`, `.is_empty`, `.remove`, `.trim_start_matches`, `HashMap::new`
- **Called by:** `CommentFamily::store` (comments.rs:854); `CommentFamily::store_people` (comments.rs:1033); `remove_family_part` (comparer/comments.rs:956); `deeper_boundary_tests::package` (comparer/comments.rs:1838) (?); `coverage_boundary_tests::package` (document_comparer.rs:8851) (?); `coverage_boundary_tests::custom_property_adoption_refuses_unrelated_collision_and_malformed_xml` (document_comparer.rs:10034) (?); `coverage_boundary_tests::empty_missing_and_non_xml_package_parts_are_safe_noops` (document_comparer.rs:10069) (?); `coverage_boundary_tests::header_adoption_refuses_unresolvable_references_and_missing_story_structure` (document_comparer.rs:10706) (?); `coverage_boundary_tests::tracked_change_admission_scans_each_xml_story_and_refuses_binary_or_non_xml_carriers` (document_comparer.rs:11373) (?); `coverage_package_source_matrix_tests::package` (document_comparer.rs:11610) (?); `adopted_story_relationship_owner_tests::adopted_root_and_nested_stories_keep_colliding_media_relationships_resolvable` (document_comparer.rs:12648) (?); `deeper_boundary_tests::paragraph_styles_missing_and_rootless_parts` (edit.rs:9022) (?); +6 more

#### `PartFs::parts`  (L733-737, pub)
`pub fn parts(&self) -> Vec<String>`

- **Does:** Enumerate all part names (docxodus style, no leading slash), sorted.
- **Calls (crate):** `denorm` (opc/mod.rs:447) L734
- **Calls (std/external/unresolved):** `.collect`, `.keys`, `.map`, `.sort`
- **Called by:** `append_documents` (append.rs:152) (?); `drop_comments_in_new_parts` (append.rs:456); `public_memory_append_owner_boundary_tests::snapshot` (append.rs:1498); `public_memory_spacer_ownership_tests::snapshot` (audit.rs:970); `package_para_ids` (comments.rs:1175); `public_memory_anchor_boundary_tests::snapshot` (comments.rs:1409); `public_option_source_ownership_tests::strict_and_transitional_packages_preserve_complete_authored_body_ownership` (comparer/cross_para.rs:3031) (?); `reconcile_part_relationships` (comparer/parts.rs:479); `carry_with_own_relationships` (comparer/parts.rs:627); `tests::carried_external_links_and_missing_internal_targets_do_not_create_media` (comparer/parts.rs:949) (?); `coverage_recursive_part_ownership::recursive_xml_parts_keep_each_relationship_local_and_each_source_unchanged` (comparer/parts.rs:1067) (?); `adopt_revised_header_footer` (document_comparer.rs:5174); +41 more

#### `PartFs::to_zip`  (L749-820, pub)
`pub fn to_zip(&self) -> Result<Vec<u8>, OpcError>`

- **Does:** Serialize the package back to zip bytes.
- **Calls (crate):** `part_name_to_rels_path` (opc/mod.rs:453) L764; `rank` (ambiguous, 4 candidates) L775; `refused` (opc/mod.rs:76) L799; `AdmissionError::new` (admission.rs:298) L799; `zip32_plan` (opc/zip32.rs:71) (by name) L804
- **Calls (std/external/unresolved):** `.as_ref`, `.chain`, `.cmp`, `.collect`, `.compression_level`, `.compression_method`, `.copied`, `.find`, `.finish`, `.get`, `.insert`, `.into_inner`, `.into_iter`, `.iter`, `.last_modified_time`, `.len`, `.map_err`, `.map`, `.sort_by`, `.start_file`, `.strip_prefix`, `.to_ascii_lowercase`, `.to_xml`, `.unwrap_or`, `.write_all`, `Cow::Borrowed`, `Cow::Owned`, `Cursor::new`, `Err`, `HashSet::new`, `Ok`, `SimpleFileOptions::default`, `Some`, `Vec::new`, `ZipEntry::Part`, `ZipEntry::Rels`, `ZipWriter::new`, `zip::DateTime::default`
- **Macros:** `format!`
- **Called by:** `append_documents` (append.rs:152) (?); `public_memory_append_owner_boundary_tests::public_append_keeps_source_property_history_and_fresh_annotation_owners_at_every_join_boundary` (append.rs:1516) (?); `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1599) (?); `public_memory_spacer_ownership_tests::spacer_audit_respects_authored_sections_page_breaks_and_table_cell_ownership` (audit.rs:983) (?); `resolve_for_edit_or_cli` (changes.rs:922) (?); `public_memory_anchor_boundary_tests::block_and_point_comments_preserve_exact_unicode_anchor_context_and_package` (comments.rs:1427) (?); `public_memory_anchor_boundary_tests::reversed_ranges_record_both_endpoints` (comments.rs:1516) (?); `authored_thread_reference_contract_tests::cyclic_and_missing_parent_metadata_terminate_without_discarding_comment_owners` (comments.rs:1629) (?); `authored_thread_reference_contract_tests::paragraph_id_scan_reserves_only_complete_hex_ids_from_other_xml_parts` (comments.rs:1661) (?); `public_option_source_ownership_tests::public_boundary_shifts_preserve_all_source_owners_with_lexical_options` (comparer/cross_para.rs:2702) (?); `public_option_source_ownership_tests::public_comparison_options_preserve_reordered_paragraphs_formats_and_note_owners` (comparer/cross_para.rs:2772) (?); `public_option_source_ownership_tests::public_reordered_bulk_paragraphs_preserve_sources_on_move_thrash_boundaries` (comparer/cross_para.rs:2935) (?); +71 more

#### `PartFs::resolve_rel_target`  (L829-834, pub)
`pub fn resolve_rel_target(&self, source_part: &str, rel_target: &str) -> String`

- **Does:** `resolveRelTarget(sourcePart, relTarget)` — the part a rel target names, relative to its source part, as a canonical part name: no leading slash (the form `parts()` returns) and `.`/`..` segments resolved.
- **Calls (crate):** `unescape_xml_text` (xmllinq/parse.rs:897) L832; `resolve_rel_target` (opc/mod.rs:549) L833
- **Called by:** `related` (append.rs:766); `related_part` (comments.rs:1083); `dest_rid_part_bytes` (comparer/parts.rs:249); `reconcile_dangling_relationships` (comparer/parts.rs:273) (?); `carry_relationship` (comparer/parts.rs:517); `carry_with_own_relationships` (comparer/parts.rs:627) (?); `tests::carried_images_reuse_only_matching_bytes_content_type_and_internal_type` (comparer/parts.rs:870) (?); `coverage_recursive_part_ownership::recursive_xml_parts_keep_each_relationship_local_and_each_source_unchanged` (comparer/parts.rs:1067) (?); `coverage_recursive_part_ownership::repeated_references_share_one_carried_local_relationship_without_merging_owners` (comparer/parts.rs:1190) (?); `load_embedded_fonts` (convert/font_table.rs:205); `part_xml_by_rel_kind` (convert/mod.rs:8473); `story_parts_xml` (convert/mod.rs:21285); +36 more

#### `PartFs::content_type_for`  (L837-842, pub)
`pub fn content_type_for(&self, name: &str) -> Option<String>`

- **Does:** `contentTypeFor(part)`.
- **Calls (crate):** `PartFs::content_type_for` (opc/mod.rs:837) (by name) L838; `norm` (opc/mod.rs:147) L840
- **Calls (std/external/unresolved):** `.map`, `.to_string`
- **Called by:** `deeper_boundary_tests::auxiliary_rootless_and_seeded_relationship_boundaries` (comparer/comments.rs:2400) (?); `reconcile_dangling_relationships` (comparer/parts.rs:273) (?); `carry_relationship` (comparer/parts.rs:517); `tests::carried_images_reuse_only_matching_bytes_content_type_and_internal_type` (comparer/parts.rs:870) (?); `coverage_recursive_part_ownership::recursive_xml_parts_keep_each_relationship_local_and_each_source_unchanged` (comparer/parts.rs:1067) (?); `coverage_recursive_part_ownership::repeated_references_share_one_carried_local_relationship_without_merging_owners` (comparer/parts.rs:1190) (?); `coverage_boundary_tests::final_header_footer_adoption_respects_existing_body_owners_and_cached_donor_parts` (document_comparer.rs:10165) (?); `coverage_boundary_tests::header_adoption_preserves_colliding_media_and_uses_correct_content_types` (document_comparer.rs:10516) (?); `coverage_boundary_tests::separator_media_collisions_keep_a_resolvable_relative_relationship` (document_comparer.rs:10945) (?); `Transaction::image_paragraph` (edit/images.rs:143) (?); `Opened::summary` (inspect.rs:765) (?); `write_store` (markdown/data_stores.rs:461); +11 more

#### `PartFs::add_content_type_override`  (L845-849, pub)
`pub fn add_content_type_override(&mut self, part_name: &str, content_type: &str)`

- **Does:** Add an Override entry to `[Content_Types].xml`.
- **Calls (crate):** `norm` (opc/mod.rs:147) L848
- **Calls (std/external/unresolved):** `.add_override`
- **Called by:** `add_related_part` (append.rs:796); `public_memory_append_owner_boundary_tests::package` (append.rs:1487) (?); `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1599) (?); `CommentFamily::store` (comments.rs:854); `people_and_text_boundary_tests::people` (comments.rs:1263); `public_memory_anchor_boundary_tests::block_and_point_comments_preserve_exact_unicode_anchor_context_and_package` (comments.rs:1427) (?); `public_memory_anchor_boundary_tests::reversed_ranges_record_both_endpoints` (comments.rs:1516) (?); `authored_thread_reference_contract_tests::family` (comments.rs:1585) (?); `install_parts_from` (comparer/comments.rs:937); `union_comments_xml` (comparer/comments.rs:1190); `public_option_source_ownership_tests::public_comparison_options_preserve_reordered_paragraphs_formats_and_note_owners` (comparer/cross_para.rs:2772) (?); `coverage_real_revision_merge_matrix_tests::public_compare_multi_original_blank_tail_recovers_complete_clean_sources::{fn package}` (comparer/finalize.rs:26118) (?); +40 more

#### `PartFs::add_content_type_default`  (L852-854, pub)
`pub fn add_content_type_default(&mut self, ext: &str, content_type: &str)`

- **Does:** Add a Default extension mapping to `[Content_Types].xml`.
- **Calls (std/external/unresolved):** `.add_default`
- **Called by:** `adopt_revised_header_footer` (document_comparer.rs:5174); `coverage_boundary_tests::separator_media_collisions_keep_a_resolvable_relative_relationship` (document_comparer.rs:10945) (?); `adopted_story_relationship_owner_tests::adopted_root_and_nested_stories_keep_colliding_media_relationships_resolvable` (document_comparer.rs:12648) (?); `Transaction::image_paragraph` (edit/images.rs:143) (?); `<PartRelate as Relate>::picture` (markdown/package.rs:1703); `PartFs::adopt_missing_content_types` (opc/mod.rs:860); `tests::parts_without_a_content_type_adopt_the_sources_default` (opc/mod.rs:1341) (?); `tests::an_upper_case_extension_is_already_typed_by_the_lower_case_default` (opc/mod.rs:1381) (?); `tests::content_types_survive_round_trips_unescaped` (opc/mod.rs:1458) (?)

#### `PartFs::adopt_missing_content_types`  (L860-892, pub)
`pub fn adopt_missing_content_types(&mut self, sources: &[&PartFs])`

- **Does:** Give every part that has no content type the one its source declares: the source's entry for the same part name, else the source's Default for the extension (case-insensitive, as OPC matches extensions).
- **Calls (crate):** `PartFs::parts` (opc/mod.rs:733) L861; `PartFs::content_type_for` (opc/mod.rs:837) L866; `norm` (opc/mod.rs:147) L880; `PartFs::add_content_type_override` (opc/mod.rs:845) L883; `PartFs::add_content_type_default` (opc/mod.rs:852) L887
- **Calls (std/external/unresolved):** `.clone`, `.contains`, `.eq_ignore_ascii_case`, `.filter`, `.find`, `.is_some`, `.iter`, `.map`, `.override_for`, `.rsplit_once`, `by_ext`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520) (?); `tests::parts_without_a_content_type_adopt_the_sources_default` (opc/mod.rs:1341) (?); `tests::an_upper_case_extension_is_already_typed_by_the_lower_case_default` (opc/mod.rs:1381) (?)

#### `PartFs::remove_content_type_override`  (L897-899, pub)
`pub fn remove_content_type_override(&mut self, part_name: &str)`

- **Does:** Remove an Override entry from `[Content_Types].xml` (no-op when absent).
- **Calls (crate):** `norm` (opc/mod.rs:147) L898
- **Calls (std/external/unresolved):** `.remove_override`
- **Called by:** `CommentFamily::store` (comments.rs:854); `CommentFamily::store_people` (comments.rs:1033); `remove_family_part` (comparer/comments.rs:956); `core_properties` (markdown/package.rs:1454); `PartFs::remove_untargeted_part` (opc/mod.rs:947); `remove_part` (revision_processor/comments.rs:495); `scrub_properties` (scrub.rs:358)

#### `PartFs::remove_relationships_by_type`  (L904-910, pub)
`pub fn remove_relationships_by_type(&mut self, source_part: &str, rel_type: &str)`

- **Does:** Remove every relationship of `source_part` with the given type.
- **Calls (crate):** `PartFs::part_rels_key` (opc/mod.rs:997) L905
- **Calls (std/external/unresolved):** `.cloned`, `.get_mut`, `.retain`
- **Called by:** `CommentFamily::store` (comments.rs:854); `CommentFamily::store_people` (comments.rs:1033); `remove_family_part` (comparer/comments.rs:956); `ensure_factory_package_chrome` (document_comparer.rs:3402); `prune` (markdown/package.rs:595); `tests::helper_mutations_resolve_under_the_same_index` (opc/mod.rs:2083) (?); `remove_part` (revision_processor/comments.rs:495)

#### `PartFs::remove_package_relationships_by_type`  (L914-916, pub)
`pub fn remove_package_relationships_by_type(&mut self, rel_type: &str)`

- **Does:** Remove every package-level relationship (`_rels/.rels`) of the given type.
- **Calls (std/external/unresolved):** `.retain`
- **Called by:** `core_properties` (markdown/package.rs:1454); `tests::helper_mutations_resolve_under_the_same_index` (opc/mod.rs:2083) (?); `scrub_properties` (scrub.rs:358)

#### `PartFs::remove_related_part`  (L923-941, pub)
`pub fn remove_related_part(&mut self, source_part: &str, rel_id: &str)`

- **Does:** Remove the relationship `rel_id` of `source_part` and, unless another internal relationship still targets it, the part it names with that part's own relationships and content-type override, then the parts (a header's images) nothing else targets any more.
- **Calls (crate):** `PartFs::part_rels_key` (opc/mod.rs:997) L924; `PartFs::resolve_rel_target` (opc/mod.rs:829) L939; `PartFs::remove_untargeted_part` (opc/mod.rs:947) L940
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.cloned`, `.find`, `.get_mut`, `.iter`, `.map`, `.retain`, `Some`
- **Called by:** `tests::redline_only_defects_name_the_story_and_paragraph` (edit/prepared.rs:1190) (?); `prune` (markdown/package.rs:595); `write` (markdown/view_stories.rs:1599); `tests::removing_a_related_part_drops_the_parts_only_it_targeted` (opc/mod.rs:1590) (?); `tests::helper_mutations_resolve_under_the_same_index` (opc/mod.rs:2083) (?); `drop_unreferenced_headers_footers` (revision_processor/word_save.rs:59)

#### `PartFs::remove_untargeted_part`  (L947-988)
`fn remove_untargeted_part(&mut self, part: &str)`

- **Does:** Remove `part`, its relationships and content-type override when no internal relationship of another part targets it, then each part it targeted that is left untargeted.
- **Calls (crate):** `identity` (opc/mod.rs:157) L948,955; `PartFs::resolve_rel_target` (opc/mod.rs:829) L961,979; `PartFs::remove_content_type_override` (opc/mod.rs:897) L984; `PartFs::remove_untargeted_part` (opc/mod.rs:947) L986
- **Calls (std/external/unresolved):** `.any`, `.as_deref`, `.as_str`, `.chain`, `.cloned`, `.collect`, `.eq_ignore_ascii_case`, `.filter`, `.get`, `.iter`, `.map`, `.remove`, `.unwrap_or_default`, `.unwrap_or`, `Some`, `std::iter::once`
- **Macros:** `format!`
- **Called by:** `PartFs::remove_related_part` (opc/mod.rs:923); `PartFs::remove_untargeted_part` (opc/mod.rs:947)

#### `PartFs::read_rels_for`  (L991-993, pub)
`pub fn read_rels_for(&self, part_name: &str) -> Option<&Relationships>`

- **Does:** `readRelsFor(part)` — the relationships of a part, if any.
- **Calls (crate):** `PartFs::part_rels_key` (opc/mod.rs:997) L992
- **Calls (std/external/unresolved):** `.get`
- **Called by:** `related` (append.rs:766); `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1599) (?); `related_part` (comments.rs:1083); `people_and_text_boundary_tests::removing_the_last_person_removes_its_part_type_and_relationship` (comments.rs:1309) (?); `install_parts_from` (comparer/comments.rs:937); `union_comments_xml` (comparer/comments.rs:1190); `deeper_boundary_tests::auxiliary_rootless_and_seeded_relationship_boundaries` (comparer/comments.rs:2400) (?); `public_option_source_ownership_tests::strict_and_transitional_packages_preserve_complete_authored_body_ownership` (comparer/cross_para.rs:3031) (?); `dest_rid_part_bytes` (comparer/parts.rs:249); `reconcile_dangling_relationships` (comparer/parts.rs:273); `carry_relationship` (comparer/parts.rs:517); `carry_with_own_relationships` (comparer/parts.rs:627) (?); +68 more

#### `PartFs::part_rels_key`  (L997-999)
`fn part_rels_key(&self, part_name: &str) -> Option<&String>`

- **Does:** The stored owner key for `part_name` under OPC identity — upstream's case-insensitive, normalized lookup over the rels index. _(from a code comment)_
- **Calls (crate):** `identity` (opc/mod.rs:157) L998
- **Calls (std/external/unresolved):** `.get`
- **Called by:** `PartFs::remove_relationships_by_type` (opc/mod.rs:904); `PartFs::remove_related_part` (opc/mod.rs:923); `PartFs::read_rels_for` (opc/mod.rs:991)

#### `PartFs::add_document_relationship`  (L1003-1010, pub)
`pub fn add_document_relationship( &mut self, source_part: &str, rel_type: &str, target: &str, ) -> String`

- **Does:** `addDocumentRelationship(...)` — add a relationship to a part, returning the new relationship id.
- **Calls (crate):** `.add` (ambiguous, 5 candidates) L1009; `PartFs::part_rels_mut` (opc/mod.rs:1025) L1009
- **Called by:** `add_related_part` (append.rs:796); `public_memory_append_owner_boundary_tests::append_resolves_inherited_lists_and_style_collisions_through_both_numbering_serializations` (append.rs:1599) (?); `CommentFamily::store` (comments.rs:854); `people_and_text_boundary_tests::people` (comments.rs:1263); `public_memory_anchor_boundary_tests::block_and_point_comments_preserve_exact_unicode_anchor_context_and_package` (comments.rs:1427) (?); `public_memory_anchor_boundary_tests::reversed_ranges_record_both_endpoints` (comments.rs:1516) (?); `authored_thread_reference_contract_tests::family` (comments.rs:1585) (?); `install_parts_from` (comparer/comments.rs:937); `union_comments_xml` (comparer/comments.rs:1190); `deeper_boundary_tests::auxiliary_rootless_and_seeded_relationship_boundaries` (comparer/comments.rs:2400) (?); `public_option_source_ownership_tests::public_comparison_options_preserve_reordered_paragraphs_formats_and_note_owners` (comparer/cross_para.rs:2772) (?); `coverage_real_revision_merge_matrix_tests::public_compare_multi_original_blank_tail_recovers_complete_clean_sources::{fn package}` (comparer/finalize.rs:26118) (?); +51 more

#### `PartFs::package_relationships`  (L1013-1015, pub)
`pub fn package_relationships(&self) -> &Relationships`

- **Does:** The package's own relationships (`_rels/.rels`).
- **Calls:** none
- **Called by:** `public_option_source_ownership_tests::strict_and_transitional_packages_preserve_complete_authored_body_ownership` (comparer/cross_para.rs:3031) (?); `custom_props_part` (document_comparer.rs:6181); `tests::public_and_package_relationship_readers_agree` (opc/mod.rs:1135) (?); `tests::migration_skips_zip_directories_and_keeps_a_missing_package_relationship_scope_empty` (opc/mod.rs:1645) (?); `tests::a_case_variant_name_is_the_part_on_every_lookup` (opc/mod.rs:1853) (?); `tests::mutations_resolve_under_the_one_identity` (opc/mod.rs:1939) (?); `tests::helper_mutations_resolve_under_the_same_index` (opc/mod.rs:2083) (?); `tests::a_relationship_repeated_more_than_twice_is_repaired_once` (opc/mod.rs:2156) (?)

#### `PartFs::add_package_relationship`  (L1018-1020, pub)
`pub fn add_package_relationship(&mut self, rel_type: &str, target: &str) -> String`

- **Does:** Add a package-level relationship, returning its id.
- **Calls (std/external/unresolved):** `.add`
- **Called by:** `merge_custom_properties` (document_comparer.rs:6196); `coverage_boundary_tests::package` (document_comparer.rs:8851) (?); `coverage_boundary_tests::custom_property_merge_is_case_insensitive_original_wins_and_new_ids_are_unique` (document_comparer.rs:10005) (?); `coverage_boundary_tests::custom_property_adoption_refuses_unrelated_collision_and_malformed_xml` (document_comparer.rs:10034) (?); `coverage_package_source_matrix_tests::package` (document_comparer.rs:11610) (?); `core_properties` (markdown/package.rs:1454); `tests::helper_mutations_resolve_under_the_same_index` (opc/mod.rs:2083) (?); `public_block_control_ownership_tests::package` (revision_processor.rs:4774) (?)

#### `PartFs::part_rels_mut`  (L1025-1042)
`fn part_rels_mut(&mut self, source_part: &str) -> &mut Relationships`

- **Does:** A part's relationships, created empty when missing.
- **Calls (crate):** `identity` (opc/mod.rs:157) L1032,1038; `norm` (opc/mod.rs:147) L1034; `Relationships::new` (opc/mod.rs:41) L1039
- **Calls (std/external/unresolved):** `.clone`, `.cloned`, `.entry`, `.get`, `.insert`, `.into_mut`, `.unwrap_or_else`
- **Called by:** `PartFs::add_document_relationship` (opc/mod.rs:1003); `PartFs::add_document_relationship_external` (opc/mod.rs:1047); `PartFs::set_rel_target_mode_external` (opc/mod.rs:1062)

#### `PartFs::add_document_relationship_external`  (L1047-1059, pub)
`pub fn add_document_relationship_external( &mut self, source_part: &str, rel_type: &str, target: &str, ) -> String`

- **Does:** Add a relationship with `TargetMode="External"` (absolute-URI targets are ILLEGAL for the default Internal mode — strict packaging layers like Word's reject the package without this).
- **Calls (crate):** `PartFs::part_rels_mut` (opc/mod.rs:1025) L1053; `.add` (ambiguous, 5 candidates) L1054
- **Calls (std/external/unresolved):** `.find`, `.iter_mut`, `.to_string`, `Some`
- **Called by:** `reconcile_dangling_relationships` (comparer/parts.rs:273); `carry_relationship` (comparer/parts.rs:517); `tests::carried_images_reuse_only_matching_bytes_content_type_and_internal_type` (comparer/parts.rs:870) (?); `tests::carried_external_links_and_missing_internal_targets_do_not_create_media` (comparer/parts.rs:949) (?); `coverage_boundary_tests::relationship_agreement_refuses_missing_mismatched_and_dangling_targets` (document_comparer.rs:9808) (?); `coverage_boundary_tests::carry_relationships_reuses_minted_id_and_refuses_unresolved_references` (document_comparer.rs:9852) (?); `coverage_boundary_tests::notes_copy_preserves_media_and_escapes_external_links_without_io` (document_comparer.rs:9938) (?); `coverage_boundary_tests::header_adoption_preserves_colliding_media_and_uses_correct_content_types` (document_comparer.rs:10516) (?); `comparer_relationship_contract_tests::referenced_relationship_agreement_requires_matching_type_mode_and_payload_or_external_target` (document_comparer.rs:11995) (?); `comparer_relationship_contract_tests::repeated_hyperlinks_share_one_carried_relationship_and_keep_independent_source_text_and_format` (document_comparer.rs:12086) (?); `relate` (markdown/from_docx/drawing.rs:1910); `<PartRelate as Relate>::hyperlink` (markdown/package.rs:1695); +4 more

#### `PartFs::set_rel_target_mode_external`  (L1062-1067, pub)
`pub fn set_rel_target_mode_external(&mut self, source_part: &str, rel_id: &str)`

- **Does:** Mark an existing relationship of `source_part` as External (test aid).
- **Calls (crate):** `PartFs::part_rels_mut` (opc/mod.rs:1025) L1063
- **Calls (std/external/unresolved):** `.find`, `.iter_mut`, `.to_string`, `Some`
- **Called by:** `coverage_boundary_tests::relationship_agreement_refuses_missing_mismatched_and_dangling_targets` (document_comparer.rs:9808) (?); `tests::helper_mutations_resolve_under_the_same_index` (opc/mod.rs:2083) (?)

#### `PartFs::main_document_part`  (L1070-1074, pub)
`pub fn main_document_part(&self) -> Option<String>`

- **Does:** The main document part name (docxodus style).
- **Calls (crate):** `denorm` (opc/mod.rs:447) L1073
- **Calls (std/external/unresolved):** `.get_by_type`, `.map`
- **Called by:** `tests::admission_and_partfs_agree_on_decoded_names_and_duplicates` (admission.rs:1452) (?); `main_part` (append.rs:329); `main_part` (changes.rs:1039); `public_body_heuristic_story_scope_tests::document_xml` (comparer/finalize.rs:32636) (?); `fix_up_drawing_ids_in_package_within` (comparer/fixups.rs:126); `reconcile_dangling_relationships` (comparer/parts.rs:273); `with_layout` (convert/mod.rs:769) (?); `main_rel_part` (convert/mod.rs:21310); `comments_spacing_tests::comments_chart_drawing_is_not_also_a_flow_box` (convert/mod.rs:45341) (?); `comments_spacing_tests::comments_lists_collapse_between_siblings` (convert/mod.rs:45409) (?); `comments_spacing_tests::potpourri_listnumber_gets_numbering_hanging` (convert/mod.rs:45491) (?); `comments_spacing_tests::comments_listbullet_gets_numbering_hanging` (convert/mod.rs:45550) (?); +34 more

## Test functions (46)

- `tests::opc_attribute_budget_bounds_upstream_duplicate_checks` (L1085): _no doc comment_
- `tests::public_and_package_relationship_readers_agree` (L1135): _no doc comment_
- `tests::package_with_header_rels` (L1181): _no doc comment_
- `tests::package_with_document_rels` (L1203): A package whose `word/_rels/document.xml.rels` holds `rels`.
- `tests::a_repeated_relationship_id_is_repaired_when_identical_and_refused_otherwise` (L1229): Word refuses a package that repeats a relationship Id in one part (docxide case8's fontTable rels, opened in Word 16: unreadable).
- `tests::the_package_is_written_in_the_source_order_then_new_parts_by_name` (L1268): _no doc comment_
- `tests::entries_carry_no_wall_clock_time` (L1297): _no doc comment_
- `tests::part_names_carry_exactly_one_leading_slash` (L1315): _no doc comment_
- `tests::parts_without_a_content_type_adopt_the_sources_default` (L1341): _no doc comment_
- `tests::an_upper_case_extension_is_already_typed_by_the_lower_case_default` (L1381): _no doc comment_
- `tests::migration_keeps_canonical_metadata_and_decodes_literal_entities_once` (L1400): _no doc comment_
- `tests::rel_attributes_survive_round_trips_unescaped` (L1436): _no doc comment_
- `tests::content_types_survive_round_trips_unescaped` (L1458): _no doc comment_
- `tests::rel_targets_resolve_to_canonical_part_names` (L1474): _no doc comment_
- `tests::rel_targets_are_relative_to_the_source_folder` (L1501): _no doc comment_
- `tests::rels_paths_name_their_owning_part` (L1526): _no doc comment_
- `tests::a_new_rels_part_starts_at_rid1` (L1543): A part's first relationship is `rId1`, as Word numbers them: the dependency's `Default` relationships start from `rId0`.
- `tests::writing_a_rels_part_replaces_the_parts_relationships` (L1560): _no doc comment_
- `tests::removing_a_related_part_drops_the_parts_only_it_targeted` (L1590): Dropping a header takes the media only it used with it; media another part still uses stays, and so does a part reached in a loop.
- `tests::migration_skips_zip_directories_and_keeps_a_missing_package_relationship_scope_empty` (L1645): _no doc comment_
- `tests::migration_refuses_missing_required_metadata_attributes_in_package_loading` (L1668): _no doc comment_
- `tests::a_record_local_default_namespace_cannot_bypass_metadata_validation` (L1687): _no doc comment_
- `tests::legacy_media_package` (L1715): A minimal package whose one media entry carries the raw name bytes and flag `media` names, with the document relationship pointing at the CP437 spelling of `\xc3\xa9` (`├⌐`), as a legacy writer left it.
- `tests::legacy_cp437_entry_names_keep_resolving_their_relationships` (L1723): _no doc comment_
- `tests::decoded_part_name_aliases_are_refused_not_overwritten` (L1746): _no doc comment_
- `tests::unicode_path_extra_fields_override_entry_names_as_zip8_did` (L1812): _no doc comment_
- `tests::variant_package` (L1837): A package with caller-chosen entry names — case-variant spellings of the reserved parts and the main part (#600 probes p3–p5).
- `tests::a_case_variant_name_is_the_part_on_every_lookup` (L1853): ZIP-15 (#600): OPC part names ignore ASCII case, so a single case-variant name is the part — Word opens these packages (probes p3, p4, p5) and every lookup must resolve the variant's content.
- `tests::to_zip_writes_one_entry_per_identity` (L1907): `to_zip` of a package opened from case-variant spellings writes exactly one entry per identity: the reserved parts come out with their canonical names, an ordinary part keeps the spelling it was read with (#600).
- `tests::mutations_resolve_under_the_one_identity` (L1939): Mutation keeps one part per identity: `set_part` with a case alias replaces the stored part and keeps its spelling; `remove_part` with an alias removes it; a `.rels` alias routes to the owner's parsed relationships (#600).
- `tests::a_raw_rels_part_is_superseded_by_parsed_relationships` (L1980): `set_part` with a `.rels` name whose XML does not parse keeps the raw part, and a later parseable `.rels` supersedes it — `to_zip` writes the parsed copy and no second entry of the one identity (#600).
- `tests::identity_resolves_like_a_relationship_target` (L2009): The identity a part or owner name resolves to: the part a relationship target names (leading slashes and `.`/`..` segments resolved), compared under ASCII case (#600).
- `tests::to_zip_refuses_an_identity_collision` (L2022): `to_zip` must not write two entries of one identity: a raw part whose name aliases a rels path (or the reserved names) is `DUPLICATE_PART`, not a broken archive (#600).
- `tests::a_root_rels_part_belongs_to_its_stem` (L2046): A `.rels` entry that is not a `…/_rels/….rels` path still parses: its owner is the part the name's stem names, and `to_zip` writes it back beside that root part (#600 identity coverage).
- `tests::a_package_without_content_types_is_part_not_found` (L2065): A package without `[Content_Types].xml` names the reserved part it is missing, under any spelling the entry could have carried.
- `tests::helper_mutations_resolve_under_the_same_index` (L2083): Mutations resolve under the one OPC identity everywhere: a `.rels` name drops its owner's parsed relationships, the `[Content_Types].xml` identity clears the content types, a raw part of a reserved identity is superseded by a parsed one, the relationship helpers no-op on absent owners and unknown ids, and a root part'...
- `tests::a_relationship_repeated_more_than_twice_is_repaired_once` (L2156): A relationship Id repeated three or more times still repairs to one entry and reports the Id once, in the package's own rels as in a part's (#600 coverage of the package-level repair and the recorded-ids dedup).
- `bounded_input_tests::refuses` (L2183): _no doc comment_
- `bounded_input_tests::compressed_limit` (L2193): _no doc comment_
- `bounded_input_tests::count_limit` (L2197): _no doc comment_
- `bounded_input_tests::part_limit` (L2201): _no doc comment_
- `bounded_input_tests::aggregate_limit` (L2205): _no doc comment_
- `bounded_input_tests::exact_budget_control` (L2209): _no doc comment_
- `zip32_tests::header_versions` (L2225): `(name, local version-needed, central version-needed, local extra length, central extra length)` for every entry, read from the raw headers — the local field at +4 (APPNOTE 4.3.7), the central one at +6 (APPNOTE 4.3.12).
- `zip32_tests::to_zip_entries_carry_version_20_and_no_zip64` (L2250): ZIP-10 (#604): `to_zip` writes Deflated entries, and a Deflated entry's version-needed is 20 (zip's `get_version_needed`), in both headers — never a hardcoded value, never the 45 a ZIP64 record carries.
- `zip32_tests::version_needed_distinguishes_stored_deflate_and_zip64` (L2283): The version-needed values differ by what the entry is, which is why no assertion may hardcode 20 (ZIP-10/#604): a Stored file needs 10 (zip's `MIN_VERSION`), a Stored directory needs 20 (`get_version_needed` raises a directory to 20), and a forced `large_file(true)` entry needs 45 and carries the 0x0001 ZIP64 field —...
