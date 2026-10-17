# Functions and methods in `src/comparer/preprocess.rs`

43 non-test, 4 test. Part of the [symbol manifest](../README.md).

#### `null_rel_resolver`  (L44-46, pub)
`pub fn null_rel_resolver(_r_id: &str) -> Option<String>`

- **Does:** Default resolver when no OPC package is available: every rId is treated as unresolvable and replaced with the C# fallback string `"NULL Relationship"`.
- **Calls (std/external/unresolved):** `.to_string`, `Some`
- **Called by:** none found in src

#### `is_rsid_attr`  (L48-61)
`fn is_rsid_attr(name: &XName) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L49; `XName::local_name` (xmllinq/mod.rs:208) (by name) L51
- **Macros:** `matches!`
- **Called by:** `clone_internal_unsalted` (comparer/preprocess.rs:245); `filtered_p_attr_xml` (comparer/preprocess.rs:742)

#### `is_pt`  (L63-65)
`fn is_pt(name: &XName) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L64
- **Called by:** `clone_internal_unsalted` (comparer/preprocess.rs:245); `filtered_p_attr_xml` (comparer/preprocess.rs:742); `filtered_leaf_attr_xml` (comparer/preprocess.rs:846)

#### `is_volatile_para_attr`  (L71-74)
`fn is_volatile_para_attr(name: &XName) -> bool`

- **Does:** `w14:paraId` / `w14:textId` are volatile per-paragraph ids Word regenerates; they must not affect content correlation (the golden correlates paragraphs with differing paraIds → del=0 on pure-additive docs), so strip them from the block hash.
- **Calls (crate):** `XName::namespace_name` (xmllinq/mod.rs:219) L72; `XName::local_name` (xmllinq/mod.rs:208) (by name) L73
- **Macros:** `matches!`
- **Called by:** `clone_internal_unsalted` (comparer/preprocess.rs:245); `filtered_p_attr_xml` (comparer/preprocess.rs:742); `filtered_leaf_attr_xml` (comparer/preprocess.rs:846)

#### `whitespace_invariant_for_hash`  (L80-82)
`fn whitespace_invariant_for_hash(text: &str) -> String`

- **Does:** Whitespace-invariant form for **correlated** block hashes only (Word-visual).
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.filter`, `.is_whitespace`
- **Called by:** `strip_whitespace_in_clone_text` (comparer/preprocess.rs:96); `collect_simple_p_fragments` (comparer/preprocess.rs:775)

#### `apply_text_transform`  (L84-93)
`fn apply_text_transform(text: &str, settings: &WmlComparerSettings) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.replace`, `.to_string`, `.to_uppercase`
- **Called by:** `clone_internal_unsalted` (comparer/preprocess.rs:245); `collect_simple_p_fragments` (comparer/preprocess.rs:775)

#### `strip_whitespace_in_clone_text`  (L96-108)
`fn strip_whitespace_in_clone_text(dom: &mut Dom, root: NodeId)`

- **Does:** Replace every text node under `root` with whitespace-stripped form.
- **Calls (crate):** `Dom::descendant_nodes` (xmllinq/mod.rs:730) L97; `Dom::is_text` (xmllinq/mod.rs:506) L99; `Dom::text_value` (xmllinq/mod.rs:547) L102; `whitespace_invariant_for_hash` (comparer/preprocess.rs:80) L103; `Dom::set_text_value` (xmllinq/mod.rs:555) L105
- **Calls (std/external/unresolved):** `.to_string`, `.unwrap_or`
- **Called by:** `block_sha1_from_source` (comparer/preprocess.rs:657)

#### `clone_block_level_content_for_hashing`  (L120-156, pub)
`pub fn clone_block_level_content_for_hashing( dom: &mut Dom, node: NodeId, include_related_parts: bool, settings: &WmlComparerSettings, rel_hash: &RelHashResolver, ) -> NodeId`

- **Does:** M4.B.4 — `CloneBlockLevelContentForHashing` (WmlComparer.ts :5142): clone a subtree for hashing, then strip every `xmlns:*` declaration from the clone (the wrapper would otherwise wrap every text in a different prefix).
- **Calls (crate):** `clone_internal` (comparer/preprocess.rs:225) L127; `Dom::new_element` (xmllinq/mod.rs:478) L131; `Dom::name` (xmllinq/mod.rs:526) L131; `clone_block_level_content_for_hashing::{fn strip_nsdecls}` (comparer/preprocess.rs:134) L154
- **Calls (std/external/unresolved):** `.into_iter`, `.next`, `.unwrap_or_else`, `W::name`
- **Called by:** `block_sha1_from_source` (comparer/preprocess.rs:657); `add_sha1_hash_to_block_level_content` (comparer/preprocess.rs:1259); `coverage_hash_owner_boundaries::ole_object_hash_projection_distinguishes_payload_identity_from_word_instance_ids` (comparer/preprocess.rs:1396)

#### `clone_block_level_content_for_hashing::{fn strip_nsdecls}`  (L134-153)
`fn strip_nsdecls(dom: &mut Dom, id: NodeId)`

- **Does:** remove all namespace-declaration attributes across the result (index walk — no descendants_and_self / attributes() Vec per node) _(from a code comment)_
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L135; `Dom::attr_count` (xmllinq/mod.rs:895) L136; `Dom::attr_at` (xmllinq/mod.rs:906) L139; `Dom::is_namespace_declaration` (xmllinq/mod.rs:996) L140; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L145; `Dom::child_count` (xmllinq/mod.rs:622) L147; `Dom::child_at` (xmllinq/mod.rs:632) L149; `clone_block_level_content_for_hashing::{fn strip_nsdecls}` (comparer/preprocess.rs:134) L150
- **Calls (std/external/unresolved):** `.clone`, `.push`, `Vec::new`
- **Called by:** `clone_block_level_content_for_hashing` (comparer/preprocess.rs:120); `clone_block_level_content_for_hashing::{fn strip_nsdecls}` (comparer/preprocess.rs:134)

#### `new_with_filtered_attrs`  (L162-188)
`fn new_with_filtered_attrs( dom: &mut Dom, name: XName, src: NodeId, drop: impl Fn(&XName) -> bool, children: Vec<NodeId>, ) -> NodeId`

- **Does:** Build a new element named `name`, copying `src`'s attributes except those for which `drop(name)` is true, then appending `children`.
- **Calls (crate):** `Dom::new_element` (xmllinq/mod.rs:478) L169; `Dom::attr_count` (xmllinq/mod.rs:895) L171; `Dom::attr_at` (xmllinq/mod.rs:906) L173; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L182; `Dom::add` (xmllinq/mod.rs:1058) L185
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.filter_map`, `.to_string`, `Some`, `drop`
- **Called by:** `clone_internal_unsalted` (comparer/preprocess.rs:245)

#### `clone_children`  (L193-213)
`fn clone_children( dom: &mut Dom, node: NodeId, include_related_parts: bool, settings: &WmlComparerSettings, rel_hash: &RelHashResolver, ) -> Vec<NodeId>`

- **Does:** Recurse-clone all child nodes, flattening fragment results.
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L201; `Dom::child_at` (xmllinq/mod.rs:632) L203; `clone_internal` (comparer/preprocess.rs:225) L204
- **Calls (std/external/unresolved):** `.extend`, `Vec::new`
- **Called by:** `clone_internal_unsalted` (comparer/preprocess.rs:245)

#### `clone_internal`  (L225-243)
`fn clone_internal( dom: &mut Dom, node: NodeId, include_related_parts: bool, settings: &WmlComparerSettings, rel_hash: &RelHashResolver, ) -> Vec<NodeId>`

- **Does:** Port of `CloneBlockLevelContentForHashingInternal` (:5160).
- **Calls (crate):** `clone_internal_unsalted` (comparer/preprocess.rs:245) L232; `Dom::is_element` (xmllinq/mod.rs:502) L233,237; `Dom::attribute` (xmllinq/mod.rs:874) L234; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L238
- **Calls (std/external/unresolved):** `PT::name`, `Some`
- **Called by:** `clone_block_level_content_for_hashing` (comparer/preprocess.rs:120); `clone_children` (comparer/preprocess.rs:193); `clone_internal_unsalted` (comparer/preprocess.rs:245); `clone_children_elements` (comparer/preprocess.rs:531)

#### `clone_internal_unsalted`  (L245-503)
`fn clone_internal_unsalted( dom: &mut Dom, node: NodeId, include_related_parts: bool, settings: &WmlComparerSettings, rel_hash: &RelHashResolver, ) -> Vec<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L252,291,345; `Dom::is_text` (xmllinq/mod.rs:506) L254; `Dom::text_value` (xmllinq/mod.rs:547) L255; `apply_text_transform` (comparer/preprocess.rs:84) L258,320; `Dom::new_text` (xmllinq/mod.rs:483) (by name) L262; `Dom::clone_subtree` (xmllinq/mod.rs:1215) (by name) L265; `Dom::name` (xmllinq/mod.rs:526) L267,348; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L277; `Dom::new_element` (xmllinq/mod.rs:478) (by name) L283; `clone_children` (comparer/preprocess.rs:193) L288,447,455,470,480,484...; `group_adjacent` (util/group_adjacent.rs:20) L297; `is_single_t_run` (comparer/preprocess.rs:544) L298; `Dom::attribute` (xmllinq/mod.rs:874) L300,409; `new_with_filtered_attrs` (comparer/preprocess.rs:162) L306,456,471,481,485,496; `is_rsid_attr` (comparer/preprocess.rs:48) L310; `is_pt` (comparer/preprocess.rs:63) L310,435,461,475,489,500; `is_volatile_para_attr` (comparer/preprocess.rs:71) L310,500; `Dom::value_str` (xmllinq/mod.rs:1179) L318; `Dom::new_element` (xmllinq/mod.rs:478) L321,325,352,369,381,390...; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L323,411,440,444; `Dom::add_text` (xmllinq/mod.rs:1071) L326; `Dom::add` (xmllinq/mod.rs:1058) L327,328,331,354,371,383...; `Dom::child_count` (xmllinq/mod.rs:622) L342; `Dom::child_at` (xmllinq/mod.rs:632) L344; `clone_internal` (comparer/preprocess.rs:225) L351,367,379,400; `element_children_named` (comparer/preprocess.rs:519) L365,377,398; `clone_children_elements` (comparer/preprocess.rs:531) L389,416; `XName::get` (xmllinq/mod.rs:180) L411,462,463,464,475,489; `Dom::attr_count` (xmllinq/mod.rs:895) L428; `Dom::attr_at` (xmllinq/mod.rs:906) L430
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.contains`, `.copied`, `.filter`, `.flat_map`, `.into_iter`, `.into_owned`, `.is_some`, `.iter`, `.map`, `.push`, `.to_string`, `.unwrap_or`, `.unwrap`, `O::name`, `PT::name`, `R::name`, `Some`, `VML::name`, `Vec::new`, `W::bookmark_end`, `W::bookmark_start`, `W::grid_span`, `W::name`, `W::object`, `W::p_pr`, `W::p`, `W::r_pr`, `W::r`, `W::t`, `W::tbl`, `W::tc_pr`, `W::tc`, `W::tr`, `W::txbx_content`, `W::val`, `WP::name`, `rel_hash`
- **Macros:** `vec!`
- **Called by:** `clone_internal` (comparer/preprocess.rs:225)

#### `element_children`  (L506-516)
`fn element_children(dom: &Dom, node: NodeId) -> Vec<NodeId>`

- **Does:** DOM-ITER-02: direct-child element ids (no `elements()` Vec).
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L508; `Dom::child_at` (xmllinq/mod.rs:632) L510; `Dom::is_element` (xmllinq/mod.rs:502) L511
- **Calls (std/external/unresolved):** `.push`, `Vec::new`
- **Called by:** `clone_children_elements` (comparer/preprocess.rs:531)

#### `element_children_named`  (L519-529)
`fn element_children_named(dom: &Dom, node: NodeId, filter: &XName) -> Vec<NodeId>`

- **Does:** DOM-ITER-02: direct-child elements matching `filter` (no `elements()` Vec).
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L521; `Dom::child_at` (xmllinq/mod.rs:632) L523; `Dom::is_element` (xmllinq/mod.rs:502) L524; `Dom::name` (xmllinq/mod.rs:526) L524
- **Calls (std/external/unresolved):** `.as_ref`, `.push`, `Some`, `Vec::new`
- **Called by:** `clone_internal_unsalted` (comparer/preprocess.rs:245)

#### `clone_children_elements`  (L531-542)
`fn clone_children_elements( dom: &mut Dom, node: NodeId, include_related_parts: bool, settings: &WmlComparerSettings, rel_hash: &RelHashResolver, ) -> Vec<NodeId>`

- **Does:** _no doc comment_
- **Calls (crate):** `element_children` (comparer/preprocess.rs:506) L538; `clone_internal` (comparer/preprocess.rs:225) L540
- **Calls (std/external/unresolved):** `.collect`, `.flat_map`, `.into_iter`
- **Called by:** `clone_internal_unsalted` (comparer/preprocess.rs:245)

#### `is_single_t_run`  (L544-563)
`fn is_single_t_run(dom: &Dom, e: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L545,558; `Dom::child_count` (xmllinq/mod.rs:622) L551; `Dom::child_at` (xmllinq/mod.rs:632) L553; `Dom::is_element` (xmllinq/mod.rs:502) L554
- **Calls (std/external/unresolved):** `Some`, `W::r`, `W::t`
- **Called by:** `clone_internal_unsalted` (comparer/preprocess.rs:245)

#### `remove_existing_powertools_markup`  (L572-591, pub)
`pub fn remove_existing_powertools_markup(dom: &mut Dom, root: NodeId)`

- **Does:** M4.B.1 — `RemoveExistingPowerToolsMarkup` (WmlComparer.ts :5049): drop every `pt:*` attribute EXCEPT `pt:Unid`, across `root` and all descendants.
- **Calls (crate):** `Dom::for_each_descendant_and_self` (xmllinq/mod.rs:753) L576; `Dom::attr_count` (xmllinq/mod.rs:895) L580; `Dom::attr_at` (xmllinq/mod.rs:906) L582; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L583; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L588
- **Calls (std/external/unresolved):** `.clone`, `.push`, `PT::unid`, `Vec::new`
- **Called by:** `get_revisions` (document_comparer.rs:6285)

#### `test_for_invalid_content`  (L602-624, pub)
`pub fn test_for_invalid_content(dom: &Dom, root: NodeId) -> Result<(), String>`

- **Does:** M4.B.1 — `TestForInvalidContent` (WmlComparer.ts :5037): the *preprocess* guard, which rejects only `w:altChunk`, `w:subDoc`, `w:contentPart` (a narrower set than the atomizer's [`super::atomize::verify_no_invalid_content`]).
- **Calls (crate):** `Dom::for_each_descendant_element` (xmllinq/mod.rs:677) L610; `Dom::name` (xmllinq/mod.rs:526) L614; `XName::local_name` (xmllinq/mod.rs:208) (by name) L617
- **Calls (std/external/unresolved):** `.contains`, `.is_some`, `Err`, `Ok`, `Some`, `W::name`
- **Macros:** `format!`
- **Called by:** `get_revisions` (document_comparer.rs:6285)

#### `block_hash_string`  (L630-637, pub)
`pub fn block_hash_string(dom: &Dom, clone: NodeId) -> String`

- **Does:** M4.B.2 — the block hash *string*: serialize the clone, then strip the single re-emitted wordprocessingml default-xmlns declaration (the wrapper has already stripped every `xmlns` decl, so exactly one is re-emitted on the root).
- **Calls (crate):** `Dom::serialize_element` (xmllinq/mod.rs:1299) L631
- **Calls (std/external/unresolved):** `.drain`, `.find`, `.len`
- **Called by:** none found in src

#### `block_sha1`  (L641-645, pub)
`pub fn block_sha1(dom: &Dom, clone: NodeId) -> String`

- **Does:** M4.B.2 — SHA-1 of the block hash string.
- **Calls (crate):** `Dom::serialize_element_sha1_hex` (xmllinq/mod.rs:1309) L644
- **Called by:** `block_sha1_from_source` (comparer/preprocess.rs:657); `add_sha1_hash_to_block_level_content` (comparer/preprocess.rs:1259)

#### `block_sha1_from_source`  (L657-680, pub)
`pub fn block_sha1_from_source( dom: &mut Dom, node: NodeId, include_related_parts: bool, settings: &WmlComparerSettings, rel_hash: &RelHashResolver, correlated_ws: bool, ) -> String`

- **Does:** HASH-STREAM-03 / HASH-STREAM-04: content SHA-1 of a **source** block node, preferring a no-clone stream for simple `w:p` and simple `w:tbl` / `w:tr`.
- **Calls (crate):** `try_stream_hash_simple_paragraph` (comparer/preprocess.rs:690) L665; `try_stream_hash_simple_table_or_tr` (comparer/preprocess.rs:928) L670; `clone_block_level_content_for_hashing` (comparer/preprocess.rs:120) L675; `strip_whitespace_in_clone_text` (comparer/preprocess.rs:96) L677; `block_sha1` (comparer/preprocess.rs:641) L679
- **Called by:** `add_sha1_hash_to_block_level_content` (comparer/preprocess.rs:1259); `hash_block_level_content` (comparer/preprocess.rs:1317)

#### `try_stream_hash_simple_paragraph`  (L690-722, pub)
`pub fn try_stream_hash_simple_paragraph( dom: &Dom, node: NodeId, settings: &WmlComparerSettings, correlated_ws: bool, ) -> Option<String>`

- **Does:** HASH-STREAM-03 / HASH-STREAM-05: stream-hash a simple paragraph without a hash-clone subtree.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L696; `element_or_desc_has_predelete_orig` (comparer/preprocess.rs:1004) L703; `filtered_p_attr_xml` (comparer/preprocess.rs:742) L706; `collect_simple_p_fragments` (comparer/preprocess.rs:775) L707; `emit_merged_run_fragments` (comparer/preprocess.rs:879) L715; `sha1_hex` (util/sha1.rs:35) L721
- **Calls (std/external/unresolved):** `.drain`, `.find`, `.is_empty`, `.len`, `.push_str`, `.push`, `Some`, `String::with_capacity`, `W::p`
- **Called by:** `block_sha1_from_source` (comparer/preprocess.rs:657)

#### `is_streamable_empty_run_leaf`  (L734-739)
`fn is_streamable_empty_run_leaf(local: &str) -> bool`

- **Does:** _no doc comment_
- **Macros:** `matches!`
- **Called by:** `collect_simple_p_fragments` (comparer/preprocess.rs:775)

#### `filtered_p_attr_xml`  (L742-772)
`fn filtered_p_attr_xml(dom: &Dom, node: NodeId) -> Option<String>`

- **Does:** Paragraph attrs surviving rsid/pt/volatile filter (w: or empty ns only).
- **Calls (crate):** `Dom::attr_count` (xmllinq/mod.rs:895) L744; `Dom::attr_at` (xmllinq/mod.rs:906) L745; `Dom::is_namespace_declaration` (xmllinq/mod.rs:996) L746; `is_rsid_attr` (comparer/preprocess.rs:48) L749; `is_pt` (comparer/preprocess.rs:63) L749; `is_volatile_para_attr` (comparer/preprocess.rs:71) L749; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L752; `XName::local_name` (xmllinq/mod.rs:208) (by name) L753; `escape_xml_attr` (comparer/preprocess.rs:1191) L759,765
- **Calls (std/external/unresolved):** `.is_empty`, `.push_str`, `.push`, `Some`, `String::new`
- **Called by:** `try_stream_hash_simple_paragraph` (comparer/preprocess.rs:690); `stream_simple_p_fragment` (comparer/preprocess.rs:1156)

#### `collect_simple_p_fragments`  (L775-833)
`fn collect_simple_p_fragments( dom: &Dom, node: NodeId, settings: &WmlComparerSettings, correlated_ws: bool, ) -> Option<Vec<RunFrag>>`

- **Does:** Expand runs under `w:p` to fragments; empty runs skipped; complex → None.
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L782,797; `Dom::child_at` (xmllinq/mod.rs:632) L784,798; `Dom::is_element` (xmllinq/mod.rs:502) L785,799; `Dom::name` (xmllinq/mod.rs:526) L788,802; `apply_text_transform` (comparer/preprocess.rs:84) L808; `Dom::value_str` (xmllinq/mod.rs:1179) L808; `whitespace_invariant_for_hash` (comparer/preprocess.rs:80) L810; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L816; `is_streamable_empty_run_leaf` (comparer/preprocess.rs:734) L817; `XName::local_name` (xmllinq/mod.rs:208) (by name) L817,823; `has_element_child` (comparer/preprocess.rs:835) L818; `filtered_leaf_attr_xml` (comparer/preprocess.rs:846) L820
- **Calls (std/external/unresolved):** `.push`, `.to_string`, `RunFrag::Text`, `Some`, `Vec::new`, `W::bookmark_end`, `W::bookmark_start`, `W::p_pr`, `W::r_pr`, `W::r`, `W::t`
- **Called by:** `try_stream_hash_simple_paragraph` (comparer/preprocess.rs:690); `stream_simple_p_fragment` (comparer/preprocess.rs:1156)

#### `has_element_child`  (L835-843)
`fn has_element_child(dom: &Dom, node: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L836; `Dom::is_element` (xmllinq/mod.rs:502) L838; `Dom::child_at` (xmllinq/mod.rs:632) L838
- **Called by:** `collect_simple_p_fragments` (comparer/preprocess.rs:775)

#### `filtered_leaf_attr_xml`  (L846-876)
`fn filtered_leaf_attr_xml(dom: &Dom, node: NodeId) -> Option<String>`

- **Does:** Attrs on empty leaf after clone's default filter (pt / volatile / trim set).
- **Calls (crate):** `Dom::attr_count` (xmllinq/mod.rs:895) L848; `Dom::attr_at` (xmllinq/mod.rs:906) L849; `Dom::is_namespace_declaration` (xmllinq/mod.rs:996) L850; `is_pt` (comparer/preprocess.rs:63) L853; `is_volatile_para_attr` (comparer/preprocess.rs:71) L853; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L856; `XName::local_name` (xmllinq/mod.rs:208) (by name) L857; `escape_xml_attr` (comparer/preprocess.rs:1191) L863,869
- **Calls (std/external/unresolved):** `.contains`, `.is_empty`, `.push_str`, `.push`, `Some`, `String::new`
- **Called by:** `collect_simple_p_fragments` (comparer/preprocess.rs:775)

#### `emit_merged_run_fragments`  (L879-912)
`fn emit_merged_run_fragments(frags: &[RunFrag], out: &mut String, structure_only: bool)`

- **Does:** Merge adjacent text fragments, emit `<w:r>…</w:r>` sequence into `out`.
- **Calls (crate):** `escape_xml_text` (comparer/preprocess.rs:1202) L899
- **Calls (std/external/unresolved):** `.clone`, `.is_empty`, `.len`, `.push_str`
- **Called by:** `try_stream_hash_simple_paragraph` (comparer/preprocess.rs:690); `stream_simple_p_fragment` (comparer/preprocess.rs:1156)

#### `try_stream_hash_simple_table_or_tr`  (L928-996, pub)
`pub fn try_stream_hash_simple_table_or_tr( dom: &Dom, node: NodeId, settings: &WmlComparerSettings, correlated_ws: bool, ) -> Option<(String, String)>`

- **Does:** HASH-STREAM-04/06: stream content + structure digests for a simple `w:tbl`, `w:tr`, or `w:tc` without materializing a hash-clone subtree.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L934; `element_or_desc_has_predelete_orig` (comparer/preprocess.rs:1004) L939; `emit_open_root` (comparer/preprocess.rs:998) L945,946,958,959,972,973; `stream_tbl_body` (comparer/preprocess.rs:1023) L947; `stream_tr_body` (comparer/preprocess.rs:1048) L960; `stream_tc_body` (comparer/preprocess.rs:1074) L974; `sha1_hex` (util/sha1.rs:35) L993,994
- **Calls (std/external/unresolved):** `.drain`, `.find`, `.len`, `.push_str`, `Some`, `String::with_capacity`, `W::tbl`, `W::tc`, `W::tr`
- **Called by:** `block_sha1_from_source` (comparer/preprocess.rs:657); `add_sha1_hash_to_block_level_content` (comparer/preprocess.rs:1259); `coverage_hash_owner_boundaries::direct_cell_bookmarks_are_ignored_only_in_hash_projection` (comparer/preprocess.rs:1448)

#### `emit_open_root`  (L998-1002)
`fn emit_open_root(out: &mut String, local: &str)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.push_str`
- **Called by:** `try_stream_hash_simple_table_or_tr` (comparer/preprocess.rs:928)

#### `element_or_desc_has_predelete_orig`  (L1004-1020)
`fn element_or_desc_has_predelete_orig(dom: &Dom, node: NodeId) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L1008; `Dom::attribute` (xmllinq/mod.rs:874) L1011; `Dom::child_count` (xmllinq/mod.rs:622) L1014; `Dom::child_at` (xmllinq/mod.rs:632) L1016
- **Calls (std/external/unresolved):** `.pop`, `.push`, `PT::name`, `Some`
- **Macros:** `vec!`
- **Called by:** `try_stream_hash_simple_paragraph` (comparer/preprocess.rs:690); `try_stream_hash_simple_table_or_tr` (comparer/preprocess.rs:928)

#### `stream_tbl_body`  (L1023-1045)
`fn stream_tbl_body( dom: &Dom, tbl: NodeId, settings: &WmlComparerSettings, correlated_ws: bool, content: &mut String, structure: &mut String, ) -> Option<()>`

- **Does:** Stream all `w:tr` children of a table (non-tr children ignored like clone).
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L1032; `Dom::child_at` (xmllinq/mod.rs:632) L1034; `Dom::is_element` (xmllinq/mod.rs:502) L1035; `Dom::name` (xmllinq/mod.rs:526) L1035; `stream_tr_body` (comparer/preprocess.rs:1048) L1040
- **Calls (std/external/unresolved):** `.as_ref`, `.push_str`, `Some`, `W::tr`
- **Called by:** `try_stream_hash_simple_table_or_tr` (comparer/preprocess.rs:928); `stream_tc_body` (comparer/preprocess.rs:1074)

#### `stream_tr_body`  (L1048-1070)
`fn stream_tr_body( dom: &Dom, tr: NodeId, settings: &WmlComparerSettings, correlated_ws: bool, content: &mut String, structure: &mut String, ) -> Option<()>`

- **Does:** Stream all `w:tc` children of a row (non-tc children ignored like clone).
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L1057; `Dom::child_at` (xmllinq/mod.rs:632) L1059; `Dom::is_element` (xmllinq/mod.rs:502) L1060; `Dom::name` (xmllinq/mod.rs:526) L1060; `stream_tc_body` (comparer/preprocess.rs:1074) L1065
- **Calls (std/external/unresolved):** `.as_ref`, `.push_str`, `Some`, `W::tc`
- **Called by:** `try_stream_hash_simple_table_or_tr` (comparer/preprocess.rs:928); `stream_tbl_body` (comparer/preprocess.rs:1023)

#### `stream_tc_body`  (L1074-1121)
`fn stream_tc_body( dom: &Dom, tc: NodeId, settings: &WmlComparerSettings, correlated_ws: bool, content: &mut String, structure: &mut String, ) -> Option<()>`

- **Does:** Stream cell body: simple paragraphs + tcPr (gridSpan only).
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L1082; `Dom::child_at` (xmllinq/mod.rs:632) L1084; `Dom::is_element` (xmllinq/mod.rs:502) L1085; `Dom::name` (xmllinq/mod.rs:526) L1088; `.namespace_name` (ambiguous: `XNamespace::namespace_name` (xmllinq/mod.rs:140); `XName::namespace_name` (xmllinq/mod.rs:219)) L1097; `stream_tc_pr` (comparer/preprocess.rs:1125) L1101; `stream_simple_p_fragment` (comparer/preprocess.rs:1156) L1105; `stream_tbl_body` (comparer/preprocess.rs:1023) L1112
- **Calls (std/external/unresolved):** `.push_str`, `Some`, `W::bookmark_end`, `W::bookmark_start`, `W::p_pr`, `W::p`, `W::r_pr`, `W::tbl`, `W::tc_pr`
- **Called by:** `try_stream_hash_simple_table_or_tr` (comparer/preprocess.rs:928); `stream_tr_body` (comparer/preprocess.rs:1048)

#### `stream_tc_pr`  (L1125-1152)
`fn stream_tc_pr(dom: &Dom, tc_pr: NodeId, content: &mut String, structure: &mut String)`

- **Does:** `tcPr` clone keeps only `w:gridSpan` children; empty → `<w:tcPr />`.
- **Calls (crate):** `Dom::child_count` (xmllinq/mod.rs:622) L1128; `Dom::child_at` (xmllinq/mod.rs:632) L1130; `Dom::is_element` (xmllinq/mod.rs:502) L1131; `Dom::name` (xmllinq/mod.rs:526) L1131; `Dom::attribute` (xmllinq/mod.rs:874) L1134; `escape_xml_attr` (comparer/preprocess.rs:1191) L1146
- **Calls (std/external/unresolved):** `.as_ref`, `.is_empty`, `.push_str`, `.push`, `.to_string`, `.unwrap_or`, `Some`, `Vec::new`, `W::grid_span`, `W::val`
- **Macros:** `format!`
- **Called by:** `stream_tc_body` (comparer/preprocess.rs:1074)

#### `stream_simple_p_fragment`  (L1156-1189)
`fn stream_simple_p_fragment( dom: &Dom, node: NodeId, settings: &WmlComparerSettings, correlated_ws: bool, content: &mut String, structure: &mut String, ) -> Option<()>`

- **Does:** Emit simple-paragraph projection as a fragment (no root xmlns).
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L1164; `filtered_p_attr_xml` (comparer/preprocess.rs:742) L1167; `collect_simple_p_fragments` (comparer/preprocess.rs:775) L1168; `emit_merged_run_fragments` (comparer/preprocess.rs:879) L1181,1186
- **Calls (std/external/unresolved):** `.is_empty`, `.push_str`, `.push`, `Some`, `W::p`
- **Called by:** `stream_tc_body` (comparer/preprocess.rs:1074)

#### `escape_xml_attr`  (L1191-1200)
`fn escape_xml_attr(s: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.any`, `.bytes`, `.replace`, `.to_string`
- **Macros:** `matches!`
- **Called by:** `filtered_p_attr_xml` (comparer/preprocess.rs:742); `filtered_leaf_attr_xml` (comparer/preprocess.rs:846); `stream_tc_pr` (comparer/preprocess.rs:1125); `escape_xml_tests::plain_text_roundtrips_without_entities` (comparer/preprocess.rs:1378); `escape_xml_tests::specials_are_escaped` (comparer/preprocess.rs:1384)

#### `escape_xml_text`  (L1202-1209)
`fn escape_xml_text(s: &str) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.any`, `.bytes`, `.replace`, `.to_string`
- **Macros:** `matches!`
- **Called by:** `emit_merged_run_fragments` (comparer/preprocess.rs:879); `escape_xml_tests::plain_text_roundtrips_without_entities` (comparer/preprocess.rs:1378); `escape_xml_tests::specials_are_escaped` (comparer/preprocess.rs:1384)

#### `structure_sha1`  (L1213-1215, pub)
`pub fn structure_sha1(dom: &Dom, node: NodeId) -> String`

- **Does:** HASH-STREAM-02: SHA-1 of the structure projection of `node` (elements + attrs only).
- **Calls (crate):** `Dom::serialize_element_structure_sha1_hex` (xmllinq/mod.rs:1317) L1214
- **Called by:** `add_sha1_hash_to_block_level_content` (comparer/preprocess.rs:1259)

#### `clone_for_structure_hash`  (L1221-1245, pub)
`pub fn clone_for_structure_hash(dom: &mut Dom, node: NodeId) -> Option<NodeId>`

- **Does:** M4.B.3 — `CloneForStructureHash` (WmlComparer.ts :5121): keep element name + attributes + element nesting, drop ALL text/value nodes.
- **Calls (crate):** `Dom::is_element` (xmllinq/mod.rs:502) L1222; `Dom::name` (xmllinq/mod.rs:526) L1225; `Dom::new_element` (xmllinq/mod.rs:478) L1226; `Dom::attr_count` (xmllinq/mod.rs:895) L1228; `Dom::attr_at` (xmllinq/mod.rs:906) L1230; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1235; `Dom::child_count` (xmllinq/mod.rs:622) L1237; `Dom::child_at` (xmllinq/mod.rs:632) L1239; `clone_for_structure_hash` (comparer/preprocess.rs:1221) L1240; `Dom::add` (xmllinq/mod.rs:1058) L1241
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.map`, `.to_string`, `.unwrap`, `Some`
- **Called by:** `clone_for_structure_hash` (comparer/preprocess.rs:1221)

#### `add_sha1_hash_to_block_level_content`  (L1259-1304, pub)
`pub fn add_sha1_hash_to_block_level_content( dom: &mut Dom, content_parent: NodeId, settings: &WmlComparerSettings, rel_hash: &RelHashResolver, )`

- **Does:** M4.B.5 — `AddSha1HashToBlockLevelContent` (WmlComparer.ts :5080): stamp `pt:SHA1Hash` on every [`super::tables::ELEMENTS_TO_HAVE_SHA1`] descendant of `content_parent`; additionally `pt:StructureSHA1Hash` on `w:tbl` / `w:tr` (the structure-clone of the hashing-clone).
- **Calls (crate):** `Dom::for_each_descendant_element` (xmllinq/mod.rs:677) L1267; `Dom::name` (xmllinq/mod.rs:526) L1268,1276; `block_sha1_from_source` (comparer/preprocess.rs:657) L1280; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1281,1288,1291,1297,1301; `try_stream_hash_simple_table_or_tr` (comparer/preprocess.rs:928) L1286; `clone_block_level_content_for_hashing` (comparer/preprocess.rs:120) L1295; `block_sha1` (comparer/preprocess.rs:641) L1296; `structure_sha1` (comparer/preprocess.rs:1213) L1300
- **Calls (std/external/unresolved):** `.contains`, `.is_some_and`, `.push`, `.unwrap`, `PT::sha1_hash`, `PT::structure_sha1_hash`, `Some`, `Vec::new`, `W::p`, `W::tbl`, `W::tc`, `W::tr`
- **Called by:** `public_option_source_ownership_tests::sourced_cross_paragraph_windows_decline_quadratic_tables_without_mutating_owners::{fn member}` (comparer/cross_para.rs:3260); `process_footnote_endnote` (comparer/footnotes.rs:1115); `coverage_real_source_route_matrix_tests::source` (comparer/lcs.rs:18303); `horizontal_partition_tests::actual_atomized_split_with_deleted_table_conserves_source_through_production` (comparer/lcs_table.rs:1187); `deterministic_table_dispatch_tests::source` (comparer/lcs_table.rs:1621); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209); `repeated_source_anchor_promotion_tests::authored_units` (comparer/moves.rs:1114); `source_paragraph_promotion_boundary_tests::source` (comparer/moves.rs:1234)

#### `hash_block_level_content`  (L1317-1370, pub)
`pub fn hash_block_level_content( dom: &mut Dom, source_root: NodeId, after_proc_root: NodeId, settings: &WmlComparerSettings, rel_hash: &RelHashResolver, ) -> Result<(), String>`

- **Does:** M4.B.6 — `HashBlockLevelContent` (WmlComparer.ts :832): hash every `w:p` / `w:tbl` / `w:tr` block of the `after_proc` projection and store it as `pt:CorrelatedSHA1Hash` back onto the corresponding ORIGINAL `source_root` element (matched by surviving `pt:Unid`).
- **Calls (crate):** `Dom::for_each_descendant_element` (xmllinq/mod.rs:677) L1331,1347; `Dom::name` (xmllinq/mod.rs:526) L1335,1348; `markdown_pairing_boundary_tests::block` (markdown/patch.rs:893) (by name) L1335,1348; `Dom::attribute` (xmllinq/mod.rs:874) L1336,1363; `block_sha1_from_source` (comparer/preprocess.rs:657) L1355; `Dom::set_attribute_value` (xmllinq/mod.rs:961) L1366
- **Calls (std/external/unresolved):** `.get`, `.insert`, `.is_some_and`, `.is_some`, `.map`, `.push`, `.to_string`, `Err`, `HashMap::new`, `Ok`, `PT::correlated_sha1_hash`, `PT::unid`, `Some`, `Vec::new`, `W::p`, `W::tbl`, `W::tr`
- **Macros:** `format!`
- **Called by:** `horizontal_partition_tests::actual_atomized_split_with_deleted_table_conserves_source_through_production` (comparer/lcs_table.rs:1187); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:209)

## Test functions (4)

- `escape_xml_tests::plain_text_roundtrips_without_entities` (L1378): _no doc comment_
- `escape_xml_tests::specials_are_escaped` (L1384): _no doc comment_
- `coverage_hash_owner_boundaries::ole_object_hash_projection_distinguishes_payload_identity_from_word_instance_ids` (L1396): _no doc comment_
- `coverage_hash_owner_boundaries::direct_cell_bookmarks_are_ignored_only_in_hash_projection` (L1448): _no doc comment_
