# Functions and methods in `src/xmllinq/serialize.rs`

31 non-test, 7 test. Part of the [symbol manifest](../README.md).

#### `well_known_prefix`  (L181-186, pub(crate))
`pub(crate) fn well_known_prefix(ns: &str) -> Option<&'static str>`

- **Does:** The conventional prefix for `ns`.
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.map`
- **Called by:** `element_path` (validate.rs:1418); `Scope::assign` (xmllinq/serialize.rs:290); `Scope::ensure_prefix` (xmllinq/serialize.rs:326)

#### `well_known_namespace`  (L190-195, pub(crate))
`pub(crate) fn well_known_namespace(prefix: &str) -> Option<&'static str>`

- **Does:** The namespace a conventional prefix stands for: what an `mc:Choice Requires="wps"` means when nothing in scope binds `wps`.
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.map`
- **Called by:** `bind_compatibility_prefixes` (comparer/finalize.rs:1026); `Part::resolve` (validate.rs:644); `check_namespace_qname_context` (validate.rs:2562)

#### `Scope::root`  (L211-221)
`fn root() -> Scope<'static>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.insert`, `.to_string`, `HashMap::new`
- **Called by:** `CommentFamily::adopt` (comments.rs:743) (?); `coverage_atomizer_owned_boundaries::note_definition_owns_every_character_and_mark_but_the_part_is_a_path_stop` (comparer/atomize.rs:827) (?); `deep_production_path_tests::deletion_peel_keeps_nontext_only_original_anchors_in_their_source_paragraph` (comparer/finalize.rs:22795) (?); `deep_production_path_tests::deletion_peel_keeps_punctuation_and_whitespace_only_original_anchors` (comparer/finalize.rs:22854) (?); `deep_production_path_tests::deletion_peel_keeps_original_closing_boundary_when_all_original_words_are_deleted` (comparer/finalize.rs:22901) (?); `deep_production_path_tests::wholesale_deleted_body_peel_transfers_its_retained_closing_boundary` (comparer/finalize.rs:22946) (?); `deep_production_path_tests::deletion_peel_does_not_move_an_original_suffix_past_equal_nonperiod_text` (comparer/finalize.rs:22999) (?); `coverage_real_revision_merge_matrix_tests::exercise` (comparer/finalize.rs:24981) (?); `demo_title_pilcrow_history_tests::title_fold_preserves_existing_mark_history_and_section_break_ownership` (comparer/finalize.rs:28787) (?); `finalizer_distinct_boundary_contract_tests::faithful_merged_closing_carrier_records_both_complete_property_sources` (comparer/finalize.rs:30208) (?); `finalizer_remaining_source_boundary_tests::two_authored_layout_blanks_are_restored_only_at_the_shared_table_boundary` (comparer/finalize.rs:31283) (?); `complete_shape_revision_contract_tests::matching_opaque_shapes_keep_every_neighbor_when_revision_wrappers_split` (comparer/textbox.rs:260) (?); +17 more

#### `Scope::child`  (L225-251)
`fn child(parent: &'a Scope<'a>, dom: &Dom, e: NodeId) -> Scope<'a>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::attr_count` (xmllinq/mod.rs:909) L229; `Dom::attr_at` (xmllinq/mod.rs:920) L230; `Dom::is_namespace_declaration` (xmllinq/mod.rs:1012) L231; `XName::local_name` (xmllinq/mod.rs:208) (by name) L234,237; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L234
- **Calls (std/external/unresolved):** `.insert`, `.is_empty`, `.to_string`, `HashMap::new`, `Some`
- **Called by:** `shown` (markdown/data_stores.rs:266) (?); `hidden_run` (markdown/from_docx/agent.rs:157) (?); `revision_tags::{fn flatten}` (markdown/from_docx/agent.rs:569) (?); `own_size::{fn walk}` (markdown/from_docx/agent.rs:1132) (?); `id_line` (markdown/from_docx/agent.rs:1994) (?); `geometry_clauses` (markdown/from_docx/agent.rs:2286) (?); `table_line` (markdown/from_docx/agent.rs:2378) (?); `tests::stamps_paragraphs_in_document_order_cells_included_text_boxes_excluded` (markdown/from_docx/agent.rs:3457) (?); `tests::a_universal_measure_indent_on_the_id_line_reads_as_twips` (markdown/from_docx/agent.rs:3472) (?); `tests::an_absurd_measure_indent_drops_its_clause` (markdown/from_docx/agent.rs:3501) (?); `tests::review_3_revision_tags_keep_unequal_source_dates_separate` (markdown/from_docx/agent.rs:3550) (?); `tests::revision_tags_keep_distinct_ids_and_pair_a_deletion_with_the_insertion_after_it` (markdown/from_docx/agent.rs:3566) (?); +35 more

#### `Scope::active_prefix_uri`  (L254-264)
`fn active_prefix_uri(&self, prefix: &str) -> Option<&str>`

- **Does:** Active prefix→URI binding for `prefix` in this scope (local first, then ancestors).
- **Calls (std/external/unresolved):** `.get`, `Some`
- **Called by:** `Scope::uri_for_prefix` (xmllinq/serialize.rs:267); `Scope::prefix_for_uri` (xmllinq/serialize.rs:272); `Scope::assign` (xmllinq/serialize.rs:290); `Scope::ensure_prefix` (xmllinq/serialize.rs:326); `Scope::next_generated` (xmllinq/serialize.rs:350)

#### `Scope::uri_for_prefix`  (L267-269)
`fn uri_for_prefix(&self, prefix: &str) -> Option<&str>`

- **Does:** Resolve a prefix token to the URI it is bound to in this scope.
- **Calls (crate):** `Scope::active_prefix_uri` (xmllinq/serialize.rs:254) L268
- **Called by:** `write_attributes` (xmllinq/serialize.rs:591); `emit` (xmllinq/serialize.rs:668); `emit_structure` (xmllinq/serialize.rs:739)

#### `Scope::prefix_for_uri`  (L272-287)
`fn prefix_for_uri(&self, uri: &str) -> Option<&str>`

- **Does:** Find the active prefix bound to `uri` in this scope.
- **Calls (crate):** `Scope::active_prefix_uri` (xmllinq/serialize.rs:254) L279
- **Calls (std/external/unresolved):** `.get`, `.is_empty`, `Some`
- **Called by:** `Scope::assign` (xmllinq/serialize.rs:290); `Scope::ensure_prefix` (xmllinq/serialize.rs:326); `resolve_prefix` (xmllinq/serialize.rs:583); `write_attributes` (xmllinq/serialize.rs:591)

#### `Scope::assign`  (L290-315)
`fn assign(&mut self, state: &mut State, uri: &str) -> String`

- **Does:** Assign (or reuse) a prefix for `uri` in this scope.
- **Calls (crate):** `Scope::prefix_for_uri` (xmllinq/serialize.rs:272) L297; `well_known_prefix` (xmllinq/serialize.rs:181) L300; `Scope::active_prefix_uri` (xmllinq/serialize.rs:254) L301,309; `Scope::next_generated` (xmllinq/serialize.rs:350) L304,307,310
- **Calls (std/external/unresolved):** `.clone`, `.insert`, `.is_empty`, `.is_none`, `.is_some`, `.to_string`, `String::new`
- **Called by:** `resolve_prefix` (xmllinq/serialize.rs:583)

#### `Scope::ensure_prefix`  (L326-348)
`fn ensure_prefix(&mut self, state: &mut State, uri: &str)`

- **Does:** Register a prefix for `uri` in this scope if not already bound.
- **Calls (crate):** `Scope::prefix_for_uri` (xmllinq/serialize.rs:272) L330; `well_known_prefix` (xmllinq/serialize.rs:181) L334; `Scope::active_prefix_uri` (xmllinq/serialize.rs:254) L335,343; `Scope::next_generated` (xmllinq/serialize.rs:350) L338,341,344
- **Calls (std/external/unresolved):** `.clone`, `.insert`, `.is_empty`, `.is_none`, `.is_some`, `.to_string`
- **Called by:** `write_attributes` (xmllinq/serialize.rs:591); `emit` (xmllinq/serialize.rs:668); `emit_structure` (xmllinq/serialize.rs:739)

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
- **Called by:** `is_namespace_qname_list` (comparer/comments.rs:1027); `bind_compatibility_prefixes` (comparer/finalize.rs:1026); `emit` (xmllinq/serialize.rs:668); `emit_structure` (xmllinq/serialize.rs:739)

#### `serialize_element`  (L401-407, pub)
`pub fn serialize_element(dom: &Dom, el: NodeId) -> String`

- **Does:** Serialize an element subtree (`el` and its descendants) to an XML string.
- **Calls (crate):** `Scope::root` (xmllinq/serialize.rs:211) L403; `emit` (xmllinq/serialize.rs:668) L405
- **Calls (std/external/unresolved):** `String::new`
- **Called by:** `assemble` (markdown/package.rs:103); `final_section` (markdown/package.rs:553); `separator_formats` (markdown/package.rs:1308); `Dom::serialize_element` (xmllinq/mod.rs:1315); `tests::word_2016_plus_namespaces_serialize_with_word_prefixes` (xmllinq/serialize.rs:859); `tests::rebound_prefixes_generate_unoccupied_names_and_preserve_expanded_names` (xmllinq/serialize.rs:941); `tests::streamed_structure_hash_drops_only_non_element_payload` (xmllinq/serialize.rs:981); `tests::attribute_and_text_whitespace_survive_serialization_round_trips` (xmllinq/serialize.rs:1022); `tests::reserved_namespace_rebindings_cannot_corrupt_xml_attributes` (xmllinq/serialize.rs:1040); `xml_structure_namespace_boundary_tests::structure_hash_keeps_mc_required_namespace_ownership_while_omitting_payload` (xmllinq/serialize.rs:1070)

#### `serialize_element_sha1_hex`  (L418-424, pub)
`pub fn serialize_element_sha1_hex(dom: &Dom, el: NodeId) -> String`

- **Does:** SHA-1 hex of [`serialize_element`] output with the first `xmlns="http://schemas.openxmlformats.org/wordprocessingml/2006/main"` declaration stripped from the root open tag (HASH-STREAM).
- **Calls (crate):** `Scope::root` (xmllinq/serialize.rs:211) L420; `HashXmlBuf::new` (xmllinq/serialize.rs:472) L421; `emit` (xmllinq/serialize.rs:668) L422; `HashXmlBuf::finish_hex` (xmllinq/serialize.rs:481) L423
- **Called by:** `Dom::serialize_element_sha1_hex` (xmllinq/mod.rs:1325); `tests::streamed_structure_hash_drops_only_non_element_payload` (xmllinq/serialize.rs:981)

#### `serialize_element_structure_sha1_hex`  (L432-438, pub)
`pub fn serialize_element_structure_sha1_hex(dom: &Dom, el: NodeId) -> String`

- **Does:** SHA-1 hex of the structural shape only: element names, attributes, and nesting; no text, comment, or PI content.
- **Calls (crate):** `Scope::root` (xmllinq/serialize.rs:211) L434; `HashXmlBuf::new` (xmllinq/serialize.rs:472) L435; `emit_structure` (xmllinq/serialize.rs:739) L436; `HashXmlBuf::finish_hex` (xmllinq/serialize.rs:481) L437
- **Called by:** `Dom::serialize_element_structure_sha1_hex` (xmllinq/mod.rs:1333); `tests::streamed_structure_hash_drops_only_non_element_payload` (xmllinq/serialize.rs:981); `xml_structure_namespace_boundary_tests::structure_hash_keeps_mc_required_namespace_ownership_while_omitting_payload` (xmllinq/serialize.rs:1070)

#### `XmlBuf::push_str`  (L442-442, trait decl, no body)
`fn push_str(&mut self, s: &str)`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `write_qname` (xmllinq/serialize.rs:527); `write_escape_attr` (xmllinq/serialize.rs:540); `write_escape_text` (xmllinq/serialize.rs:566); `write_attributes` (xmllinq/serialize.rs:591); `emit` (xmllinq/serialize.rs:668); `emit_structure` (xmllinq/serialize.rs:739)

#### `XmlBuf::push`  (L443-443, trait decl, no body)
`fn push(&mut self, c: char)`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `write_qname` (xmllinq/serialize.rs:527); `write_escape_attr` (xmllinq/serialize.rs:540); `write_escape_text` (xmllinq/serialize.rs:566); `write_attributes` (xmllinq/serialize.rs:591); `emit` (xmllinq/serialize.rs:668); `emit_structure` (xmllinq/serialize.rs:739)

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
- **Called by:** `write_attributes` (xmllinq/serialize.rs:591); `emit` (xmllinq/serialize.rs:668); `emit_structure` (xmllinq/serialize.rs:739)

#### `write_escape_attr`  (L540-560)
`fn write_escape_attr(out: &mut impl XmlBuf, s: &str)`

- **Does:** SER-01: escape attribute values into `out`.
- **Calls (crate):** `XmlBuf::push_str` (xmllinq/serialize.rs:442) L545,550,551,552,553,554...; `XmlBuf::push` (xmllinq/serialize.rs:443) L557
- **Calls (std/external/unresolved):** `.any`, `.bytes`, `.chars`
- **Macros:** `matches!`
- **Called by:** `write_attributes` (xmllinq/serialize.rs:591)

#### `write_escape_text`  (L566-580)
`fn write_escape_text(out: &mut impl XmlBuf, s: &str)`

- **Does:** SER-01: escape text content into `out`.
- **Calls (crate):** `XmlBuf::push_str` (xmllinq/serialize.rs:442) L568,573,574,575,576; `XmlBuf::push` (xmllinq/serialize.rs:443) L577
- **Calls (std/external/unresolved):** `.any`, `.bytes`, `.chars`
- **Macros:** `matches!`
- **Called by:** `emit` (xmllinq/serialize.rs:668); `serialize_document` (xmllinq/serialize.rs:805)

#### `resolve_prefix`  (L583-588)
`fn resolve_prefix(scope: &mut Scope<'_>, state: &mut State, uri: &str) -> String`

- **Does:** Resolve prefix for a URI, assigning if needed.
- **Calls (crate):** `Scope::prefix_for_uri` (xmllinq/serialize.rs:272) L584; `Scope::assign` (xmllinq/serialize.rs:290) L587
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `emit` (xmllinq/serialize.rs:668); `emit_structure` (xmllinq/serialize.rs:739)

#### `write_attributes`  (L591-666)
`fn write_attributes( out: &mut impl XmlBuf, scope: &mut Scope<'_>, state: &mut State, real_attrs: &[(&XName, &str)], prefix_list_attrs: &[(&XName, &str)], )`

- **Does:** Write namespace decls + real attrs + rewritten QName-list attrs into `out`.
- **Calls (crate):** `XmlBuf::push_str` (xmllinq/serialize.rs:442) L613,617,618,619,635,645; `write_escape_attr` (xmllinq/serialize.rs:540) L614,620,636,663; `XmlBuf::push` (xmllinq/serialize.rs:443) L615,621,633,637,643,664; `Scope::ensure_prefix` (xmllinq/serialize.rs:326) L631,641; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L631,632,641,642; `Scope::prefix_for_uri` (xmllinq/serialize.rs:272) L632,642,655; `write_qname` (xmllinq/serialize.rs:527) L634,644; `XName::local_name` (xmllinq/mod.rs:208) (by name) L634,644; `Scope::uri_for_prefix` (xmllinq/serialize.rs:267) L654
- **Calls (std/external/unresolved):** `.cmp`, `.collect`, `.is_empty`, `.iter`, `.push_str`, `.push`, `.sort_by`, `.split_whitespace`, `.unwrap_or`, `String::new`
- **Called by:** `emit` (xmllinq/serialize.rs:668); `emit_structure` (xmllinq/serialize.rs:739)

#### `emit`  (L668-735)
`fn emit(dom: &Dom, e: NodeId, parent: &Scope, state: &mut State, out: &mut impl XmlBuf)`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L669; `Scope::child` (xmllinq/serialize.rs:225) L670; `Scope::ensure_prefix` (xmllinq/serialize.rs:326) L675,683,687; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L675,683,696; `Dom::attr_count` (xmllinq/mod.rs:909) L678; `Dom::attr_at` (xmllinq/mod.rs:920) L679; `Dom::is_namespace_declaration` (xmllinq/mod.rs:1012) L680; `is_namespace_prefix_list` (xmllinq/serialize.rs:365) L684; `Scope::uri_for_prefix` (xmllinq/serialize.rs:267) L686; `resolve_prefix` (xmllinq/serialize.rs:583) L696; `XName::local_name` (xmllinq/mod.rs:208) (by name) L697; `Dom::child_count` (xmllinq/mod.rs:622) L698; `XmlBuf::push` (xmllinq/serialize.rs:443) L701,710,734; `write_qname` (xmllinq/serialize.rs:527) L702,733; `write_attributes` (xmllinq/serialize.rs:591) L703; `XmlBuf::push_str` (xmllinq/serialize.rs:442) L706,718,719,720,722,723...; `Dom::child_at` (xmllinq/mod.rs:632) L712; `Dom::is_element` (xmllinq/mod.rs:502) L713; `emit` (xmllinq/serialize.rs:668) L714; `Dom::is_text` (xmllinq/mod.rs:506) L715; `write_escape_text` (xmllinq/serialize.rs:566) L716; `Dom::text_value` (xmllinq/mod.rs:547) L716,719; `Dom::is_comment` (xmllinq/mod.rs:515) L717; `Dom::is_pi` (xmllinq/mod.rs:519) L721; `Dom::pi_target` (xmllinq/mod.rs:564) L723; `Dom::pi_data` (xmllinq/mod.rs:572) L724
- **Calls (std/external/unresolved):** `.expect`, `.is_empty`, `.map`, `.push`, `.split_whitespace`, `.to_string`, `.unwrap_or`, `Vec::new`
- **Called by:** `segment_region` (comparer/cross_para.rs:1503) (?); `tests::a_comment_on_a_vertical_page_turns_with_its_text` (convert/pdf.rs:2304) (?); `Pieces::feed` (markdown/critic.rs:1913) (?); `serialize_element` (xmllinq/serialize.rs:401); `serialize_element_sha1_hex` (xmllinq/serialize.rs:418); `emit` (xmllinq/serialize.rs:668); `serialize_document` (xmllinq/serialize.rs:805)

#### `emit_structure`  (L739-795)
`fn emit_structure(dom: &Dom, e: NodeId, parent: &Scope, state: &mut State, out: &mut impl XmlBuf)`

- **Does:** HASH-STREAM-02: like [`emit`] but drops all non-element children (text / comment / PI), matching `CloneForStructureHash` then serialize.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L740; `Scope::child` (xmllinq/serialize.rs:225) L741; `Scope::ensure_prefix` (xmllinq/serialize.rs:326) L743,751,755; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L743,751,764; `Dom::attr_count` (xmllinq/mod.rs:909) L746; `Dom::attr_at` (xmllinq/mod.rs:920) L747; `Dom::is_namespace_declaration` (xmllinq/mod.rs:1012) L748; `is_namespace_prefix_list` (xmllinq/serialize.rs:365) L752; `Scope::uri_for_prefix` (xmllinq/serialize.rs:267) L754; `resolve_prefix` (xmllinq/serialize.rs:583) L764; `XName::local_name` (xmllinq/mod.rs:208) (by name) L765; `Dom::child_count` (xmllinq/mod.rs:622) L766; `Dom::is_element` (xmllinq/mod.rs:502) L771,788; `Dom::child_at` (xmllinq/mod.rs:632) L771,787; `XmlBuf::push` (xmllinq/serialize.rs:443) L776,785,794; `write_qname` (xmllinq/serialize.rs:527) L777,793; `write_attributes` (xmllinq/serialize.rs:591) L778; `XmlBuf::push_str` (xmllinq/serialize.rs:442) L781,792; `emit_structure` (xmllinq/serialize.rs:739) L789
- **Calls (std/external/unresolved):** `.expect`, `.map`, `.push`, `.split_whitespace`, `.to_string`, `Vec::new`
- **Called by:** `serialize_element_structure_sha1_hex` (xmllinq/serialize.rs:432); `emit_structure` (xmllinq/serialize.rs:739)

#### `serialize_document`  (L805-848, pub)
`pub fn serialize_document(dom: &Dom, doc: NodeId) -> String`

- **Does:** Serialize a whole document (declaration + root + siblings) to a string.
- **Calls (crate):** `Dom::declaration` (xmllinq/mod.rs:581) L807; `Dom::child_count` (xmllinq/mod.rs:622) L823; `Dom::child_at` (xmllinq/mod.rs:632) L824; `Dom::is_element` (xmllinq/mod.rs:502) L825; `Scope::root` (xmllinq/serialize.rs:211) L828; `emit` (xmllinq/serialize.rs:668) L829; `Dom::is_text` (xmllinq/mod.rs:506) L830; `write_escape_text` (xmllinq/serialize.rs:566) L831; `Dom::text_value` (xmllinq/mod.rs:547) L831,834; `Dom::is_comment` (xmllinq/mod.rs:515) L832; `Dom::is_pi` (xmllinq/mod.rs:519) L836; `Dom::pi_target` (xmllinq/mod.rs:564) L838; `Dom::pi_data` (xmllinq/mod.rs:572) L839
- **Calls (std/external/unresolved):** `.as_deref`, `.is_empty`, `.push_str`, `.push`, `.unwrap_or`, `String::new`
- **Called by:** `unique_revision_ids` (markdown/package.rs:378); `Dom::serialize_document` (xmllinq/mod.rs:1338); `tests::document_serialization_preserves_declarations_comments_and_processing_instructions` (xmllinq/serialize.rs:885)

## Test functions (7)

- `tests::word_2016_plus_namespaces_serialize_with_word_prefixes` (L859): Word binds `w16` to the 2018 wordml namespace (and writes `w16sdtfl`, `w16du` for its 2024/2023 ones); an unbound use gets Word's prefix.
- `tests::document_serialization_preserves_declarations_comments_and_processing_instructions` (L885): _no doc comment_
- `tests::rebound_prefixes_generate_unoccupied_names_and_preserve_expanded_names` (L941): _no doc comment_
- `tests::streamed_structure_hash_drops_only_non_element_payload` (L981): _no doc comment_
- `tests::attribute_and_text_whitespace_survive_serialization_round_trips` (L1022): XML 1.0 §3.3.3: attribute-value normalization turns a literal TAB, CR or LF in an attribute value into a space on every read, so the DOM's whitespace must be written back as character references.
- `tests::reserved_namespace_rebindings_cannot_corrupt_xml_attributes` (L1040): _no doc comment_
- `xml_structure_namespace_boundary_tests::structure_hash_keeps_mc_required_namespace_ownership_while_omitting_payload` (L1070): _no doc comment_
