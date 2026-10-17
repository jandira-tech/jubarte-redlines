# Functions and methods in `src/xmllinq/serialize.rs`

31 non-test, 6 test. Part of the [symbol manifest](../README.md).

#### `well_known_prefix`  (L181-186, pub(crate))
`pub(crate) fn well_known_prefix(ns: &str) -> Option<&'static str>`

- **Does:** The conventional prefix for `ns`.
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.map`
- **Called by:** `element_path` (validate.rs:546); `Scope::assign` (xmllinq/serialize.rs:290); `Scope::ensure_prefix` (xmllinq/serialize.rs:326)

#### `well_known_namespace`  (L190-195, pub(crate))
`pub(crate) fn well_known_namespace(prefix: &str) -> Option<&'static str>`

- **Does:** The namespace a conventional prefix stands for: what an `mc:Choice Requires="wps"` means when nothing in scope binds `wps`.
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.map`
- **Called by:** `bind_compatibility_prefixes` (comparer/finalize.rs:983); `check_namespace_qname_context` (validate.rs:1588)

#### `Scope::root`  (L211-221)
`fn root() -> Scope<'static>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.insert`, `.to_string`, `HashMap::new`
- **Called by:** `CommentFamily::adopt` (comments.rs:687) (?); `coverage_atomizer_owned_boundaries::note_definition_owns_every_character_and_mark_but_the_part_is_a_path_stop` (comparer/atomize.rs:827) (?); `deep_production_path_tests::deletion_peel_keeps_nontext_only_original_anchors_in_their_source_paragraph` (comparer/finalize.rs:22773) (?); `deep_production_path_tests::deletion_peel_keeps_punctuation_and_whitespace_only_original_anchors` (comparer/finalize.rs:22832) (?); `deep_production_path_tests::deletion_peel_keeps_original_closing_boundary_when_all_original_words_are_deleted` (comparer/finalize.rs:22879) (?); `deep_production_path_tests::wholesale_deleted_body_peel_transfers_its_retained_closing_boundary` (comparer/finalize.rs:22924) (?); `deep_production_path_tests::deletion_peel_does_not_move_an_original_suffix_past_equal_nonperiod_text` (comparer/finalize.rs:22977) (?); `coverage_real_revision_merge_matrix_tests::exercise` (comparer/finalize.rs:24959) (?); `demo_title_pilcrow_history_tests::title_fold_preserves_existing_mark_history_and_section_break_ownership` (comparer/finalize.rs:28768) (?); `finalizer_distinct_boundary_contract_tests::faithful_merged_closing_carrier_records_both_complete_property_sources` (comparer/finalize.rs:30192) (?); `finalizer_remaining_source_boundary_tests::two_authored_layout_blanks_are_restored_only_at_the_shared_table_boundary` (comparer/finalize.rs:31268) (?); `complete_shape_revision_contract_tests::matching_opaque_shapes_keep_every_neighbor_when_revision_wrappers_split` (comparer/textbox.rs:260) (?); +16 more

#### `Scope::child`  (L225-251)
`fn child(parent: &'a Scope<'a>, dom: &Dom, e: NodeId) -> Scope<'a>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::attr_count` (xmllinq/mod.rs:895) L229; `Dom::attr_at` (xmllinq/mod.rs:906) L230; `Dom::is_namespace_declaration` (xmllinq/mod.rs:996) L231; `XName::local_name` (xmllinq/mod.rs:208) (by name) L234,237; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L234
- **Calls (std/external/unresolved):** `.insert`, `.is_empty`, `.to_string`, `HashMap::new`, `Some`
- **Called by:** `hidden_run` (markdown/from_docx/agent.rs:133) (?); `revision_tags::{fn flatten}` (markdown/from_docx/agent.rs:340) (?); `id_line` (markdown/from_docx/agent.rs:1514) (?); `table_line` (markdown/from_docx/agent.rs:1812) (?); `tests::stamps_paragraphs_in_document_order_cells_included_text_boxes_excluded` (markdown/from_docx/agent.rs:2786) (?); `tests::a_universal_measure_indent_on_the_id_line_reads_as_twips` (markdown/from_docx/agent.rs:2801) (?); `tests::an_absurd_measure_indent_drops_its_clause` (markdown/from_docx/agent.rs:2829) (?); `tests::revision_tags_join_neighbours_then_pair_an_insertion_with_the_deletion_after_it` (markdown/from_docx/agent.rs:2877) (?); `tests::revision_tags_pass_over_empty_marks_wherever_they_sit` (markdown/from_docx/agent.rs:2902) (?); `tests::a_shown_range_or_bookmark_between_marks_keeps_them_apart` (markdown/from_docx/agent.rs:2932) (?); `tests::tags_always_carry_the_handle_and_format_for_printing` (markdown/from_docx/agent.rs:2957) (?); `tests::range_markers_do_not_break_a_move_or_a_substitution` (markdown/from_docx/agent.rs:3026) (?); +28 more

#### `Scope::active_prefix_uri`  (L254-264)
`fn active_prefix_uri(&self, prefix: &str) -> Option<&str>`

- **Does:** Active prefix→URI binding for `prefix` in this scope (local first, then ancestors).
- **Calls (std/external/unresolved):** `.get`, `Some`
- **Called by:** `Scope::uri_for_prefix` (xmllinq/serialize.rs:267); `Scope::prefix_for_uri` (xmllinq/serialize.rs:272); `Scope::assign` (xmllinq/serialize.rs:290); `Scope::ensure_prefix` (xmllinq/serialize.rs:326); `Scope::next_generated` (xmllinq/serialize.rs:350)

#### `Scope::uri_for_prefix`  (L267-269)
`fn uri_for_prefix(&self, prefix: &str) -> Option<&str>`

- **Does:** Resolve a prefix token to the URI it is bound to in this scope.
- **Calls (crate):** `Scope::active_prefix_uri` (xmllinq/serialize.rs:254) L268
- **Called by:** `write_attributes` (xmllinq/serialize.rs:579); `emit` (xmllinq/serialize.rs:656); `emit_structure` (xmllinq/serialize.rs:727)

#### `Scope::prefix_for_uri`  (L272-287)
`fn prefix_for_uri(&self, uri: &str) -> Option<&str>`

- **Does:** Find the active prefix bound to `uri` in this scope.
- **Calls (crate):** `Scope::active_prefix_uri` (xmllinq/serialize.rs:254) L279
- **Calls (std/external/unresolved):** `.get`, `.is_empty`, `Some`
- **Called by:** `Scope::assign` (xmllinq/serialize.rs:290); `Scope::ensure_prefix` (xmllinq/serialize.rs:326); `resolve_prefix` (xmllinq/serialize.rs:571); `write_attributes` (xmllinq/serialize.rs:579)

#### `Scope::assign`  (L290-315)
`fn assign(&mut self, state: &mut State, uri: &str) -> String`

- **Does:** Assign (or reuse) a prefix for `uri` in this scope.
- **Calls (crate):** `Scope::prefix_for_uri` (xmllinq/serialize.rs:272) L297; `well_known_prefix` (xmllinq/serialize.rs:181) L300; `Scope::active_prefix_uri` (xmllinq/serialize.rs:254) L301,309; `Scope::next_generated` (xmllinq/serialize.rs:350) L304,307,310
- **Calls (std/external/unresolved):** `.clone`, `.insert`, `.is_empty`, `.is_none`, `.is_some`, `.to_string`, `String::new`
- **Called by:** `resolve_prefix` (xmllinq/serialize.rs:571)

#### `Scope::ensure_prefix`  (L326-348)
`fn ensure_prefix(&mut self, state: &mut State, uri: &str)`

- **Does:** Register a prefix for `uri` in this scope if not already bound.
- **Calls (crate):** `Scope::prefix_for_uri` (xmllinq/serialize.rs:272) L330; `well_known_prefix` (xmllinq/serialize.rs:181) L334; `Scope::active_prefix_uri` (xmllinq/serialize.rs:254) L335,343; `Scope::next_generated` (xmllinq/serialize.rs:350) L338,341,344
- **Calls (std/external/unresolved):** `.clone`, `.insert`, `.is_empty`, `.is_none`, `.is_some`, `.to_string`
- **Called by:** `write_attributes` (xmllinq/serialize.rs:579); `emit` (xmllinq/serialize.rs:656); `emit_structure` (xmllinq/serialize.rs:727)

#### `Scope::next_generated`  (L350-358)
`fn next_generated(state: &mut State, scope: &Scope<'_>) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `Scope::active_prefix_uri` (xmllinq/serialize.rs:254) L354
- **Calls (std/external/unresolved):** `.is_none`
- **Macros:** `format!`
- **Called by:** `Scope::assign` (xmllinq/serialize.rs:290); `Scope::ensure_prefix` (xmllinq/serialize.rs:326)

#### `is_namespace_prefix_list`  (L365-382, pub(crate))
`pub(crate) fn is_namespace_prefix_list(element: &XName, name: &XName) -> bool`

- **Does:** True for attributes whose value is a list of namespace prefixes that must be rewritten when prefixes are rebound in this scope.
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L366,370; `XName::local_name` (xmllinq/mod.rs:208) L367,371
- **Calls (std/external/unresolved):** `.is_empty`
- **Macros:** `matches!`
- **Called by:** `is_namespace_qname_list` (comparer/comments.rs:1027); `bind_compatibility_prefixes` (comparer/finalize.rs:983); `emit` (xmllinq/serialize.rs:656); `emit_structure` (xmllinq/serialize.rs:727)

#### `serialize_element`  (L401-407, pub)
`pub fn serialize_element(dom: &Dom, el: NodeId) -> String`

- **Does:** Serialize an element subtree (`el` and its descendants) to an XML string.
- **Calls (crate):** `Scope::root` (xmllinq/serialize.rs:211) L403; `emit` (xmllinq/serialize.rs:656) L405
- **Calls (std/external/unresolved):** `String::new`
- **Called by:** `final_section` (markdown/package.rs:394); `separator_formats` (markdown/package.rs:1107); `Dom::serialize_element` (xmllinq/mod.rs:1299); `tests::word_2016_plus_namespaces_serialize_with_word_prefixes` (xmllinq/serialize.rs:847); `tests::rebound_prefixes_generate_unoccupied_names_and_preserve_expanded_names` (xmllinq/serialize.rs:929); `tests::streamed_structure_hash_drops_only_non_element_payload` (xmllinq/serialize.rs:969); `tests::reserved_namespace_rebindings_cannot_corrupt_xml_attributes` (xmllinq/serialize.rs:1005); `xml_structure_namespace_boundary_tests::structure_hash_keeps_mc_required_namespace_ownership_while_omitting_payload` (xmllinq/serialize.rs:1035)

#### `serialize_element_sha1_hex`  (L418-424, pub)
`pub fn serialize_element_sha1_hex(dom: &Dom, el: NodeId) -> String`

- **Does:** SHA-1 hex of [`serialize_element`] output with the first `xmlns="http://schemas.openxmlformats.org/wordprocessingml/2006/main"` declaration stripped from the root open tag (HASH-STREAM).
- **Calls (crate):** `Scope::root` (xmllinq/serialize.rs:211) L420; `HashXmlBuf::new` (xmllinq/serialize.rs:472) L421; `emit` (xmllinq/serialize.rs:656) L422; `HashXmlBuf::finish_hex` (xmllinq/serialize.rs:481) L423
- **Called by:** `Dom::serialize_element_sha1_hex` (xmllinq/mod.rs:1309); `tests::streamed_structure_hash_drops_only_non_element_payload` (xmllinq/serialize.rs:969)

#### `serialize_element_structure_sha1_hex`  (L432-438, pub)
`pub fn serialize_element_structure_sha1_hex(dom: &Dom, el: NodeId) -> String`

- **Does:** SHA-1 hex of the structural shape only: element names, attributes, and nesting; no text, comment, or PI content.
- **Calls (crate):** `Scope::root` (xmllinq/serialize.rs:211) L434; `HashXmlBuf::new` (xmllinq/serialize.rs:472) L435; `emit_structure` (xmllinq/serialize.rs:727) L436; `HashXmlBuf::finish_hex` (xmllinq/serialize.rs:481) L437
- **Called by:** `Dom::serialize_element_structure_sha1_hex` (xmllinq/mod.rs:1317); `tests::streamed_structure_hash_drops_only_non_element_payload` (xmllinq/serialize.rs:969); `xml_structure_namespace_boundary_tests::structure_hash_keeps_mc_required_namespace_ownership_while_omitting_payload` (xmllinq/serialize.rs:1035)

#### `XmlBuf::push_str`  (L442-442, trait decl, no body)
`fn push_str(&mut self, s: &str)`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `write_qname` (xmllinq/serialize.rs:527); `write_escape_attr` (xmllinq/serialize.rs:537); `write_escape_text` (xmllinq/serialize.rs:555); `write_attributes` (xmllinq/serialize.rs:579); `emit` (xmllinq/serialize.rs:656); `emit_structure` (xmllinq/serialize.rs:727)

#### `XmlBuf::push`  (L443-443, trait decl, no body)
`fn push(&mut self, c: char)`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `write_qname` (xmllinq/serialize.rs:527); `write_escape_attr` (xmllinq/serialize.rs:537); `write_escape_text` (xmllinq/serialize.rs:555); `write_attributes` (xmllinq/serialize.rs:579); `emit` (xmllinq/serialize.rs:656); `emit_structure` (xmllinq/serialize.rs:727)

#### `<String as XmlBuf>::push_str`  (L448-450, impl XmlBuf)
`fn push_str(&mut self, s: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `<String as XmlBuf>::push_str` (xmllinq/serialize.rs:448) L449
- **Called by:** `<String as XmlBuf>::push_str` (xmllinq/serialize.rs:448)

#### `<String as XmlBuf>::push`  (L452-454, impl XmlBuf)
`fn push(&mut self, c: char)`

- **Does:** _no doc comment_
- **Calls (crate):** `<String as XmlBuf>::push` (xmllinq/serialize.rs:452) L453
- **Called by:** `<String as XmlBuf>::push` (xmllinq/serialize.rs:452)

#### `HashXmlBuf::new`  (L472-479)
`fn new() -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `String::with_capacity`, `sha1::Sha1::new`
- **Called by:** `serialize_element_sha1_hex` (xmllinq/serialize.rs:418); `serialize_element_structure_sha1_hex` (xmllinq/serialize.rs:432)

#### `HashXmlBuf::finish_hex`  (L481-488)
`fn finish_hex(mut self) -> String`

- **Does:** _no doc comment_
- **Calls (crate):** `HashXmlBuf::flush_open_tag` (xmllinq/serialize.rs:490) L485; `hex_string_from_bytes` (util/sha1.rs:22) L487
- **Calls (std/external/unresolved):** `.finalize`
- **Called by:** `serialize_element_sha1_hex` (xmllinq/serialize.rs:418); `serialize_element_structure_sha1_hex` (xmllinq/serialize.rs:432)

#### `HashXmlBuf::flush_open_tag`  (L490-498)
`fn flush_open_tag(&mut self)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_bytes`, `.clear`, `.drain`, `.find`, `.len`, `.update`
- **Called by:** `HashXmlBuf::finish_hex` (xmllinq/serialize.rs:481); `<HashXmlBuf as XmlBuf>::push_str` (xmllinq/serialize.rs:502)

#### `<HashXmlBuf as XmlBuf>::push_str`  (L502-517, impl XmlBuf)
`fn push_str(&mut self, s: &str)`

- **Does:** _no doc comment_
- **Calls (crate):** `HashXmlBuf::flush_open_tag` (xmllinq/serialize.rs:490) L512
- **Calls (std/external/unresolved):** `.as_bytes`, `.find`, `.is_empty`, `.push_str`, `.to_string`, `.truncate`, `.update`
- **Called by:** `<HashXmlBuf as XmlBuf>::push` (xmllinq/serialize.rs:519)

#### `<HashXmlBuf as XmlBuf>::push`  (L519-522, impl XmlBuf)
`fn push(&mut self, c: char)`

- **Does:** _no doc comment_
- **Calls (crate):** `<HashXmlBuf as XmlBuf>::push_str` (xmllinq/serialize.rs:502) L521
- **Calls (std/external/unresolved):** `.encode_utf8`
- **Called by:** none found in src

#### `write_qname`  (L527-533)
`fn write_qname(out: &mut impl XmlBuf, prefix: &str, local: &str)`

- **Does:** SER-01: write `prefix:local` (or bare local) into `out` with no temporary String.
- **Calls (crate):** `XmlBuf::push_str` (xmllinq/serialize.rs:442) L529,532; `XmlBuf::push` (xmllinq/serialize.rs:443) L530
- **Calls (std/external/unresolved):** `.is_empty`
- **Called by:** `write_attributes` (xmllinq/serialize.rs:579); `emit` (xmllinq/serialize.rs:656); `emit_structure` (xmllinq/serialize.rs:727)

#### `write_escape_attr`  (L537-551)
`fn write_escape_attr(out: &mut impl XmlBuf, s: &str)`

- **Does:** SER-01: escape attribute values into `out`.
- **Calls (crate):** `XmlBuf::push_str` (xmllinq/serialize.rs:442) L539,544,545,546,547; `XmlBuf::push` (xmllinq/serialize.rs:443) L548
- **Calls (std/external/unresolved):** `.any`, `.bytes`, `.chars`
- **Macros:** `matches!`
- **Called by:** `write_attributes` (xmllinq/serialize.rs:579)

#### `write_escape_text`  (L555-568)
`fn write_escape_text(out: &mut impl XmlBuf, s: &str)`

- **Does:** SER-01: escape text content into `out`.
- **Calls (crate):** `XmlBuf::push_str` (xmllinq/serialize.rs:442) L557,562,563,564; `XmlBuf::push` (xmllinq/serialize.rs:443) L565
- **Calls (std/external/unresolved):** `.any`, `.bytes`, `.chars`
- **Macros:** `matches!`
- **Called by:** `emit` (xmllinq/serialize.rs:656); `serialize_document` (xmllinq/serialize.rs:793)

#### `resolve_prefix`  (L571-576)
`fn resolve_prefix(scope: &mut Scope<'_>, state: &mut State, uri: &str) -> String`

- **Does:** Resolve prefix for a URI, assigning if needed.
- **Calls (crate):** `Scope::prefix_for_uri` (xmllinq/serialize.rs:272) L572; `Scope::assign` (xmllinq/serialize.rs:290) L575
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `emit` (xmllinq/serialize.rs:656); `emit_structure` (xmllinq/serialize.rs:727)

#### `write_attributes`  (L579-654)
`fn write_attributes( out: &mut impl XmlBuf, scope: &mut Scope<'_>, state: &mut State, real_attrs: &[(&XName, &str)], prefix_list_attrs: &[(&XName, &str)], )`

- **Does:** Write namespace decls + real attrs + rewritten QName-list attrs into `out`.
- **Calls (crate):** `XmlBuf::push_str` (xmllinq/serialize.rs:442) L601,605,606,607,623,633; `write_escape_attr` (xmllinq/serialize.rs:537) L602,608,624,651; `XmlBuf::push` (xmllinq/serialize.rs:443) L603,609,621,625,631,652; `Scope::ensure_prefix` (xmllinq/serialize.rs:326) L619,629; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L619,620,629,630; `Scope::prefix_for_uri` (xmllinq/serialize.rs:272) L620,630,643; `write_qname` (xmllinq/serialize.rs:527) L622,632; `XName::local_name` (xmllinq/mod.rs:208) (by name) L622,632; `Scope::uri_for_prefix` (xmllinq/serialize.rs:267) L642
- **Calls (std/external/unresolved):** `.cmp`, `.collect`, `.is_empty`, `.iter`, `.push_str`, `.push`, `.sort_by`, `.split_whitespace`, `.unwrap_or`, `String::new`
- **Called by:** `emit` (xmllinq/serialize.rs:656); `emit_structure` (xmllinq/serialize.rs:727)

#### `emit`  (L656-723)
`fn emit(dom: &Dom, e: NodeId, parent: &Scope, state: &mut State, out: &mut impl XmlBuf)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L657; `Scope::child` (xmllinq/serialize.rs:225) L658; `Scope::ensure_prefix` (xmllinq/serialize.rs:326) L663,671,675; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L663,671,684; `Dom::attr_count` (xmllinq/mod.rs:895) L666; `Dom::attr_at` (xmllinq/mod.rs:906) L667; `Dom::is_namespace_declaration` (xmllinq/mod.rs:996) L668; `is_namespace_prefix_list` (xmllinq/serialize.rs:365) L672; `Scope::uri_for_prefix` (xmllinq/serialize.rs:267) L674; `resolve_prefix` (xmllinq/serialize.rs:571) L684; `XName::local_name` (xmllinq/mod.rs:208) (by name) L685; `Dom::child_count` (xmllinq/mod.rs:622) L686; `XmlBuf::push` (xmllinq/serialize.rs:443) L689,698,722; `write_qname` (xmllinq/serialize.rs:527) L690,721; `write_attributes` (xmllinq/serialize.rs:579) L691; `XmlBuf::push_str` (xmllinq/serialize.rs:442) L694,706,707,708,710,711...; `Dom::child_at` (xmllinq/mod.rs:632) L700; `Dom::is_element` (xmllinq/mod.rs:502) L701; `emit` (xmllinq/serialize.rs:656) L702; `Dom::is_text` (xmllinq/mod.rs:506) L703; `write_escape_text` (xmllinq/serialize.rs:555) L704; `Dom::text_value` (xmllinq/mod.rs:547) L704,707; `Dom::is_comment` (xmllinq/mod.rs:515) L705; `Dom::is_pi` (xmllinq/mod.rs:519) L709; `Dom::pi_target` (xmllinq/mod.rs:564) L711; `Dom::pi_data` (xmllinq/mod.rs:572) L712
- **Calls (std/external/unresolved):** `.expect`, `.is_empty`, `.map`, `.push`, `.split_whitespace`, `.to_string`, `.unwrap_or`, `Vec::new`
- **Called by:** `segment_region` (comparer/cross_para.rs:1503) (?); `tests::a_comment_on_a_vertical_page_turns_with_its_text` (convert/pdf.rs:2304) (?); `Pieces::feed` (markdown/critic.rs:1711) (?); `serialize_element` (xmllinq/serialize.rs:401); `serialize_element_sha1_hex` (xmllinq/serialize.rs:418); `emit` (xmllinq/serialize.rs:656); `serialize_document` (xmllinq/serialize.rs:793)

#### `emit_structure`  (L727-783)
`fn emit_structure(dom: &Dom, e: NodeId, parent: &Scope, state: &mut State, out: &mut impl XmlBuf)`

- **Does:** HASH-STREAM-02: like [`emit`] but drops all non-element children (text / comment / PI), matching `CloneForStructureHash` then serialize.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L728; `Scope::child` (xmllinq/serialize.rs:225) L729; `Scope::ensure_prefix` (xmllinq/serialize.rs:326) L731,739,743; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L731,739,752; `Dom::attr_count` (xmllinq/mod.rs:895) L734; `Dom::attr_at` (xmllinq/mod.rs:906) L735; `Dom::is_namespace_declaration` (xmllinq/mod.rs:996) L736; `is_namespace_prefix_list` (xmllinq/serialize.rs:365) L740; `Scope::uri_for_prefix` (xmllinq/serialize.rs:267) L742; `resolve_prefix` (xmllinq/serialize.rs:571) L752; `XName::local_name` (xmllinq/mod.rs:208) (by name) L753; `Dom::child_count` (xmllinq/mod.rs:622) L754; `Dom::is_element` (xmllinq/mod.rs:502) L759,776; `Dom::child_at` (xmllinq/mod.rs:632) L759,775; `XmlBuf::push` (xmllinq/serialize.rs:443) L764,773,782; `write_qname` (xmllinq/serialize.rs:527) L765,781; `write_attributes` (xmllinq/serialize.rs:579) L766; `XmlBuf::push_str` (xmllinq/serialize.rs:442) L769,780; `emit_structure` (xmllinq/serialize.rs:727) L777
- **Calls (std/external/unresolved):** `.expect`, `.map`, `.push`, `.split_whitespace`, `.to_string`, `Vec::new`
- **Called by:** `serialize_element_structure_sha1_hex` (xmllinq/serialize.rs:432); `emit_structure` (xmllinq/serialize.rs:727)

#### `serialize_document`  (L793-836, pub)
`pub fn serialize_document(dom: &Dom, doc: NodeId) -> String`

- **Does:** Serialize a whole document (declaration + root + siblings) to a string.
- **Calls (crate):** `Dom::declaration` (xmllinq/mod.rs:581) L795; `Dom::child_count` (xmllinq/mod.rs:622) L811; `Dom::child_at` (xmllinq/mod.rs:632) L812; `Dom::is_element` (xmllinq/mod.rs:502) L813; `Scope::root` (xmllinq/serialize.rs:211) L816; `emit` (xmllinq/serialize.rs:656) L817; `Dom::is_text` (xmllinq/mod.rs:506) L818; `write_escape_text` (xmllinq/serialize.rs:555) L819; `Dom::text_value` (xmllinq/mod.rs:547) L819,822; `Dom::is_comment` (xmllinq/mod.rs:515) L820; `Dom::is_pi` (xmllinq/mod.rs:519) L824; `Dom::pi_target` (xmllinq/mod.rs:564) L826; `Dom::pi_data` (xmllinq/mod.rs:572) L827
- **Calls (std/external/unresolved):** `.as_deref`, `.is_empty`, `.push_str`, `.push`, `.unwrap_or`, `String::new`
- **Called by:** `Dom::serialize_document` (xmllinq/mod.rs:1322); `tests::document_serialization_preserves_declarations_comments_and_processing_instructions` (xmllinq/serialize.rs:873)

## Test functions (6)

- `tests::word_2016_plus_namespaces_serialize_with_word_prefixes` (L847): Word binds `w16` to the 2018 wordml namespace (and writes `w16sdtfl`, `w16du` for its 2024/2023 ones); an unbound use gets Word's prefix.
- `tests::document_serialization_preserves_declarations_comments_and_processing_instructions` (L873): _no doc comment_
- `tests::rebound_prefixes_generate_unoccupied_names_and_preserve_expanded_names` (L929): _no doc comment_
- `tests::streamed_structure_hash_drops_only_non_element_payload` (L969): _no doc comment_
- `tests::reserved_namespace_rebindings_cannot_corrupt_xml_attributes` (L1005): _no doc comment_
- `xml_structure_namespace_boundary_tests::structure_hash_keeps_mc_required_namespace_ownership_while_omitting_payload` (L1035): _no doc comment_
