# Functions and methods in `src/comparer/parts.rs`

18 non-test, 12 test. Part of the [symbol manifest](../README.md).

#### `decode_xml_attribute`  (L44-50, pub)
`pub fn decode_xml_attribute(value: &str) -> String`

- **Does:** Undo the four entity escapes in an OPC attribute value (the C# `DecodeXmlAttribute` at WmlComparer.ts :6967).
- **Calls (std/external/unresolved):** `.replace`
- **Called by:** `parse_relationship_rows` (comparer/parts.rs:80)

#### `attr_of`  (L52-73)
`fn attr_of(tag: &str, name: &str) -> Option<String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_bytes`, `.find`, `.is_ascii_whitespace`, `.len`, `.starts_with`, `.strip_prefix`, `.to_string`, `.trim_start`, `Some`
- **Called by:** `parse_relationship_rows` (comparer/parts.rs:80)

#### `parse_relationship_rows`  (L80-120, pub)
`pub fn parse_relationship_rows(rels_xml: &str) -> Vec<RelationshipRow>`

- **Does:** Parse every `<Relationship …>` row out of a `.rels` XML document, running [`decode_xml_attribute`] on each `Target` and applying [`is_external_relationship`] to set the [`RelationshipRow::external`] flag (because hyperlink rels and absolute-URI targets have no part behind them and would otherwise be misclassified inte...
- **Calls (crate):** `attr_of` (comparer/parts.rs:52) L97,98,99,100; `decode_xml_attribute` (comparer/parts.rs:44) L102; `is_external_relationship` (comparer/parts.rs:142) L110
- **Calls (std/external/unresolved):** `.as_deref`, `.chars`, `.find`, `.is_ascii_whitespace`, `.len`, `.next`, `.push`, `Some`, `Vec::new`
- **Macros:** `matches!`
- **Called by:** `tests::relationship_uri_schemes_and_malformed_rows_have_explicit_classification` (comparer/parts.rs:817)

#### `required_rel_type_suffix`  (L127-137, pub)
`pub fn required_rel_type_suffix(local_name: &str) -> Option<&'static str>`

- **Does:** The relationship-type suffix an element's rId must resolve to (by its element local name), or `None` when the element accepts any rel type.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `reconcile_dangling_relationships` (comparer/parts.rs:273); `carry_subtree_relationships` (comparer/parts.rs:590); `reconcile_one_part` (comparer/parts.rs:670); `carry_revised_part_relationships` (document_comparer.rs:25)

#### `is_external_relationship`  (L142-161, pub)
`pub fn is_external_relationship(rel_type: &str, target: &str) -> bool`

- **Does:** True when a relationship has no internal target part: hyperlink rel types, or any absolute-URI target whose scheme is alphanumeric plus `+.-` (RFC 3986 grammar).
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.contains`, `.ends_with`, `.find`, `.is_ascii_alphabetic`, `.is_ascii_alphanumeric`, `.is_empty`, `.next`, `.starts_with`, `.unwrap`
- **Called by:** `parse_relationship_rows` (comparer/parts.rs:80); `reconcile_dangling_relationships` (comparer/parts.rs:273); `carry_relationship` (comparer/parts.rs:517); `tests::relationship_uri_schemes_and_malformed_rows_have_explicit_classification` (comparer/parts.rs:817)

#### `dest_uri_for_reconciled_part`  (L176-226)
`fn dest_uri_for_reconciled_part(dest: &PartFs, target_part: &str, bytes: &[u8]) -> String`

- **Does:** Package URI for a part being copied during dangling-rel reconcile.
- **Calls (crate):** `source_sha256` (inspect.rs:299) L188; `PartFs::part_bytes` (opc/mod.rs:628) L196,204,218
- **Calls (std/external/unresolved):** `.as_str`, `.contains`, `.filter`, `.next`, `.rsplit_once`, `.rsplit`, `.starts_with`, `.to_ascii_lowercase`, `.to_string`, `.unwrap_or`
- **Macros:** `format!`, `matches!`
- **Called by:** `reconcile_dangling_relationships` (comparer/parts.rs:273); `carry_relationship` (comparer/parts.rs:517); `tests::media_name_is_the_sha256_of_the_bytes_and_ignores_earlier_calls` (comparer/parts.rs:737); `tests::extension_case_does_not_split_identical_media` (comparer/parts.rs:754); `tests::a_name_held_by_different_bytes_is_never_reused` (comparer/parts.rs:766); `tests::markup_collision_names_reuse_bytes_without_overwriting_original_parts` (comparer/parts.rs:1036)

#### `ref_from_inserted_content`  (L234-245)
`fn ref_from_inserted_content(dom: &Dom, mut el: NodeId) -> bool`

- **Does:** True when `el` sits inside inserted / moved-in content (a `w:ins` or `w:moveTo` ancestor).
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) L235; `Dom::name` (xmllinq/mod.rs:526) L236; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L237; `XName::local_name` (xmllinq/mod.rs:208) (by name) L238
- **Macros:** `matches!`
- **Called by:** `reconcile_dangling_relationships` (comparer/parts.rs:273)

#### `dest_rid_part_bytes`  (L249-258)
`fn dest_rid_part_bytes(dest: &PartFs, doc_part: &str, rid: &str) -> Option<Vec<u8>>`

- **Does:** Bytes of the part the destination's `rid` currently resolves to (via the main-document rels), or None if unresolved.
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:991) L250; `PartFs::resolve_rel_target` (opc/mod.rs:829) L256; `PartFs::part_bytes` (opc/mod.rs:628) L257
- **Calls (std/external/unresolved):** `.clone`, `.find`, `.iter`, `.map`, `.to_vec`
- **Called by:** `reconcile_dangling_relationships` (comparer/parts.rs:273)

#### `reconcile_dangling_relationships`  (L273-440, pub)
`pub fn reconcile_dangling_relationships( dom: &mut Dom, root: NodeId, dest: &mut PartFs, sources: &[&PartFs], )`

- **Does:** Ensure every rId the redline's main document references resolves in the destination package.
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:1070) L279; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L285; `Dom::attributes` (xmllinq/mod.rs:899) L286; `ref_from_inserted_content` (comparer/parts.rs:234) L288; `PartFs::read_rels_for` (opc/mod.rs:991) L305; `PartFs::main_document_part` (opc/mod.rs:1070) (by name) L319; `PartFs::read_rels_for` (opc/mod.rs:991) (by name) L322; `is_external_relationship` (comparer/parts.rs:142) L333; `Dom::name` (xmllinq/mod.rs:526) L350; `required_rel_type_suffix` (comparer/parts.rs:127) L352; `XName::local_name` (xmllinq/mod.rs:208) (by name) L352; `PartFs::resolve_rel_target` (opc/mod.rs:829) (by name) L373,409; `PartFs::part_bytes` (opc/mod.rs:628) (by name) L374,410; `dest_rid_part_bytes` (comparer/parts.rs:249) L376; `dest_uri_for_reconciled_part` (comparer/parts.rs:176) L377,418; `PartFs::content_type_for` (opc/mod.rs:837) (by name) L378,419; `PartFs::add_content_type_override` (opc/mod.rs:845) L379,420; `PartFs::set_part` (opc/mod.rs:661) L381,422; `relative_rel_target` (opc/mod.rs:180) L382,424; `PartFs::add_document_relationship` (opc/mod.rs:1003) L384,425; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L385,394,406,430,433; `PartFs::add_document_relationship_external` (opc/mod.rs:1047) L401
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.clone`, `.collect`, `.contains`, `.ends_with`, `.enumerate`, `.find`, `.get`, `.is_empty`, `.is_none_or`, `.is_some_and`, `.iter`, `.map`, `.push`, `.rev`, `.to_string`, `.to_vec`, `.unwrap_or_default`, `.unwrap_or_else`, `Some`, `Vec::new`, `find_source`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520)

#### `part_sources`  (L450-455)
`fn part_sources<'a>(part: &str, a: &'a PartFs, b: &'a PartFs) -> Vec<PartSource<'a>>`

- **Does:** Where the content of the output part `part` can have come from, in lookup order.
- **Calls (crate):** `revised_part_name` (comparer/parts.rs:459) L451
- **Calls (std/external/unresolved):** `.to_string`
- **Macros:** `vec!`
- **Called by:** `reconcile_one_part` (comparer/parts.rs:670)

#### `revised_part_name`  (L459-471)
`fn revised_part_name(part: &str) -> Option<String>`

- **Does:** B's own name for a `redlineB_*` output part (`redlineB_{base}`, or `redlineB_{n}_{base}` for the n-th collision); None for any other part.
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.filter`, `.is_ascii_digit`, `.is_empty`, `.map_or`, `.rsplit_once`, `.split_once`, `.strip_prefix`, `.to_string`, `.unwrap_or`, `Some`
- **Macros:** `format!`
- **Called by:** `part_sources` (comparer/parts.rs:450); `tests::revised_part_name_undoes_both_collision_forms` (comparer/parts.rs:783)

#### `reconcile_part_relationships`  (L479-489, pub)
`pub fn reconcile_part_relationships(dest: &mut PartFs, main_part: &str, a: &PartFs, b: &PartFs)`

- **Does:** Reconcile the `r:*` references every non-main `word/*.xml` part of the destination package makes in its own `.rels`.
- **Calls (crate):** `PartFs::parts` (opc/mod.rs:733) L480; `reconcile_one_part` (comparer/parts.rs:670) L487
- **Calls (std/external/unresolved):** `.collect`, `.ends_with`, `.filter`, `.into_iter`, `.sort`, `.starts_with`
- **Called by:** `compare_documents_impl` (document_comparer.rs:6520)

#### `has_relationship_attribute`  (L493-508)
`fn has_relationship_attribute(xml: &str) -> bool`

- **Does:** True when `xml` carries an attribute in the relationships namespace — the cheap gate that keeps parts without references from being parsed.
- **Calls (std/external/unresolved):** `.all`, `.any`, `.as_bytes`, `.bytes`, `.is_ascii_alphanumeric`, `.is_ascii_whitespace`, `.is_empty`, `.is_some_and`, `.len`, `.match_indices`, `.rfind`
- **Macros:** `format!`
- **Called by:** `reconcile_one_part` (comparer/parts.rs:670); `tests::relationship_attribute_gate_sees_any_whitespace_before_the_prefix` (comparer/parts.rs:801); `coverage_recursive_part_ownership::relationship_prefix_gate_accepts_valid_unusual_prefixes_without_text_false_positives` (comparer/parts.rs:1251)

#### `carry_relationship`  (L517-567, pub)
`pub fn carry_relationship( dest: &mut PartFs, part: &str, src: &PartFs, src_part: &str, rid: &str, fits: impl Fn(&str) -> bool, ) -> Option<String>`

- **Does:** Carry `src_part`'s relationship `rid` (when its type passes `fits`) onto `part` of `dest` under a fresh id: external targets verbatim, internal targets copied under a collision-proof name.
- **Calls (crate):** `PartFs::read_rels_for` (opc/mod.rs:991) L525,541; `is_external_relationship` (comparer/parts.rs:142) L532,545; `PartFs::add_document_relationship_external` (opc/mod.rs:1047) L534; `PartFs::resolve_rel_target` (opc/mod.rs:829) L536,549; `PartFs::part_bytes` (opc/mod.rs:628) L537,550; `PartFs::content_type_for` (opc/mod.rs:837) L551,558; `dest_uri_for_reconciled_part` (comparer/parts.rs:176) L557; `PartFs::add_content_type_override` (opc/mod.rs:845) L559; `PartFs::set_part` (opc/mod.rs:661) L561; `PartFs::add_document_relationship` (opc/mod.rs:1003) L562; `relative_rel_target` (opc/mod.rs:180) L565
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.as_slice`, `.clone`, `.ends_with`, `.find`, `.iter`, `.to_vec`, `Some`, `fits`
- **Called by:** `carry_with_own_relationships` (comparer/parts.rs:627); `reconcile_one_part` (comparer/parts.rs:670); `tests::carried_images_reuse_only_matching_bytes_content_type_and_internal_type` (comparer/parts.rs:870); `tests::carried_external_links_and_missing_internal_targets_do_not_create_media` (comparer/parts.rs:949); `carry_revised_part_relationships` (document_comparer.rs:25); `compare_documents_impl` (document_comparer.rs:6520)

#### `carry_part_relationships`  (L576-585, pub)
`pub fn carry_part_relationships( dest: &mut PartFs, dest_part: &str, src: &PartFs, src_part: &str, dom: &mut Dom, root: NodeId, )`

- **Does:** Walk the subtree `root` and carry every relationship it names (`r:id`, `r:embed`, `r:link`, …) from `src_part` of `src` onto `dest_part` of `dest` via [`carry_relationship`], rewriting each attribute to the carried id.
- **Calls (crate):** `carry_subtree_relationships` (comparer/parts.rs:590) L584
- **Called by:** `append_documents` (append.rs:152); `coverage_recursive_part_ownership::recursive_xml_parts_keep_each_relationship_local_and_each_source_unchanged` (comparer/parts.rs:1067); `coverage_recursive_part_ownership::repeated_references_share_one_carried_local_relationship_without_merging_owners` (comparer/parts.rs:1190)

#### `carry_subtree_relationships`  (L590-623)
`fn carry_subtree_relationships( dest: &mut PartFs, dest_part: &str, src: &PartFs, src_part: &str, dom: &mut Dom, root: NodeId, depth: usize, )`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::descendants_and_self` (xmllinq/mod.rs:746) L601; `Dom::attributes` (xmllinq/mod.rs:899) L602; `Dom::name` (xmllinq/mod.rs:526) L606; `required_rel_type_suffix` (comparer/parts.rs:127) L608; `XName::local_name` (xmllinq/mod.rs:208) (by name) L608; `carry_with_own_relationships` (comparer/parts.rs:627) L613; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L620
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.clone`, `.contains`, `.get`, `.insert`, `.unwrap_or`, `std::collections::HashMap::new`
- **Macros:** `format!`
- **Called by:** `carry_part_relationships` (comparer/parts.rs:576); `carry_with_own_relationships` (comparer/parts.rs:627)

#### `carry_with_own_relationships`  (L627-668)
`fn carry_with_own_relationships( dest: &mut PartFs, dest_part: &str, src: &PartFs, src_part: &str, rid: &str, suffix: Option<&str>, depth: usize, ) -> Option<String>`

- **Does:** [`carry_relationship`], then the same for the relationships of an XML part it newly copied.
- **Calls (crate):** `PartFs::parts` (opc/mod.rs:733) L637; `carry_relationship` (comparer/parts.rs:517) L638; `PartFs::read_rels_for` (opc/mod.rs:991) (by name) L640; `PartFs::resolve_rel_target` (opc/mod.rs:829) (by name) L642; `target` (changes.rs:1480) (by name) L645,646; `PartFs::read_rels_for` (opc/mod.rs:991) L649; `PartFs::part_string` (opc/mod.rs:636) L650; `Dom::new` (xmllinq/mod.rs:387) L652; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L653; `Dom::root` (xmllinq/mod.rs:822) L654; `carry_subtree_relationships` (comparer/parts.rs:590) L655; `PartFs::set_part` (opc/mod.rs:661) L664; `Dom::serialize_document` (xmllinq/mod.rs:1338) L664
- **Calls (std/external/unresolved):** `.as_deref`, `.collect`, `.contains`, `.ends_with`, `.find`, `.into_bytes`, `.into_iter`, `.is_none_or`, `.is_some`, `.iter`, `.then`, `Some`
- **Called by:** `carry_subtree_relationships` (comparer/parts.rs:590)

#### `reconcile_one_part`  (L670-727)
`fn reconcile_one_part(dest: &mut PartFs, part: &str, a: &PartFs, b: &PartFs)`

- **Does:** _no doc comment_
- **Calls (crate):** `PartFs::part_string` (opc/mod.rs:636) L671; `has_relationship_attribute` (comparer/parts.rs:493) L674; `Dom::new` (xmllinq/mod.rs:387) L677; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L678; `Dom::root` (xmllinq/mod.rs:822) L679; `PartFs::read_rels_for` (opc/mod.rs:991) L682; `part_sources` (comparer/parts.rs:450) L691; `Dom::descendants_and_self` (xmllinq/mod.rs:746) L695; `Dom::attributes` (xmllinq/mod.rs:899) L696; `Dom::name` (xmllinq/mod.rs:526) L700; `required_rel_type_suffix` (comparer/parts.rs:127) L702; `XName::local_name` (xmllinq/mod.rs:208) (by name) L702; `carry_relationship` (comparer/parts.rs:517) L712; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L720; `PartFs::set_part` (opc/mod.rs:661) L725; `Dom::serialize_element` (xmllinq/mod.rs:1315) L725
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.clone`, `.collect`, `.contains`, `.ends_with`, `.find_map`, `.get`, `.insert`, `.into_bytes`, `.is_none_or`, `.is_some_and`, `.iter`, `.map`, `.unwrap_or_default`, `.unwrap_or`, `Some`, `fits`, `std::collections::HashMap::new`
- **Macros:** `format!`
- **Called by:** `reconcile_part_relationships` (comparer/parts.rs:479)

## Test functions (12)

- `tests::media_name_is_the_sha256_of_the_bytes_and_ignores_earlier_calls` (L737): _no doc comment_
- `tests::extension_case_does_not_split_identical_media` (L754): _no doc comment_
- `tests::a_name_held_by_different_bytes_is_never_reused` (L766): _no doc comment_
- `tests::revised_part_name_undoes_both_collision_forms` (L783): _no doc comment_
- `tests::relationship_attribute_gate_sees_any_whitespace_before_the_prefix` (L801): _no doc comment_
- `tests::relationship_uri_schemes_and_malformed_rows_have_explicit_classification` (L817): _no doc comment_
- `tests::carried_images_reuse_only_matching_bytes_content_type_and_internal_type` (L870): _no doc comment_
- `tests::carried_external_links_and_missing_internal_targets_do_not_create_media` (L949): _no doc comment_
- `tests::markup_collision_names_reuse_bytes_without_overwriting_original_parts` (L1036): _no doc comment_
- `coverage_recursive_part_ownership::recursive_xml_parts_keep_each_relationship_local_and_each_source_unchanged` (L1067): _no doc comment_
- `coverage_recursive_part_ownership::repeated_references_share_one_carried_local_relationship_without_merging_owners` (L1190): _no doc comment_
- `coverage_recursive_part_ownership::relationship_prefix_gate_accepts_valid_unusual_prefixes_without_text_false_positives` (L1251): _no doc comment_
