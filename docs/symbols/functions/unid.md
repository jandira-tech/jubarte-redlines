# Functions and methods in `src/unid.rs`

3 non-test, 2 test. Part of the [symbol manifest](../README.md).

#### `generate_unid`  (L47-51, pub)
`pub fn generate_unid() -> String`

- **Does:** Mint a fresh 32-char hex id.
- **Calls (std/external/unresolved):** `.fetch_add`
- **Macros:** `format!`
- **Called by:** `resync_paragraph_unids` (comparer/cross_para.rs:220); `assemble_ancestor_unids` (comparer/produce.rs:470); `assign_to_all_elements` (unid.rs:60); `assign_to_self_and_descendants` (unid.rs:83); `tests::test_generate_unid_format` (unid.rs:102); `tests::test_generate_unid_uniqueness` (unid.rs:113)

#### `assign_to_all_elements`  (L60-75, pub)
`pub fn assign_to_all_elements(dom: &mut Dom, content_parent: NodeId)`

- **Does:** Stamp a `pt:Unid` attribute on every element in `content_parent`'s subtree that does not already have one, and on the root itself when it is a `w:footnote` / `w:endnote`.
- **Calls (crate):** `Dom::name` (xmllinq/mod.rs:526) L62; `Dom::attribute` (xmllinq/mod.rs:874) L64,70; `generate_unid` (unid.rs:47) L66,71; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L67,72; `Dom::descendants` (xmllinq/mod.rs:668) L69
- **Calls (std/external/unresolved):** `.is_none`, `PT::unid`, `Some`, `W::endnote`, `W::footnote`
- **Called by:** `create_comparison_unit_atom_list` (comparer/atomize.rs:307); `coverage_atomizer_owned_boundaries::tracked_payload_status_and_paragraph_mark_status_have_independent_owners` (comparer/atomize.rs:973); `horizontal_partition_tests::actual_atomized_split_with_deleted_table_conserves_source_through_production` (comparer/lcs_table.rs:1186); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215); `pre_process_markup` (document_comparer.rs:4714)

#### `assign_to_self_and_descendants`  (L83-95, pub)
`pub fn assign_to_self_and_descendants(dom: &mut Dom, root: NodeId)`

- **Does:** Stamp a `pt:Unid` attribute on `root` and on every descendant element that does not already have one.
- **Calls (crate):** `Dom::attribute` (xmllinq/mod.rs:874) L85,90; `generate_unid` (unid.rs:47) L86,91; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L87,92; `Dom::descendants` (xmllinq/mod.rs:668) L89
- **Calls (std/external/unresolved):** `.is_none`, `PT::unid`, `Some`
- **Called by:** none found in src

## Test functions (2)

- `tests::test_generate_unid_format` (L102): _no doc comment_
- `tests::test_generate_unid_uniqueness` (L113): _no doc comment_
