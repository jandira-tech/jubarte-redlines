# Functions and methods in `src/comparer/fixups.rs`

7 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `id_attr`  (L23-25)
`fn id_attr() -> XName`

- **Does:** _no doc comment_
- **Calls (crate):** `XName::get` (xmllinq/mod.rs:180) L24
- **Called by:** `fix_up_doc_pr_ids` (comparer/fixups.rs:32); `fix_up_shape_ids` (comparer/fixups.rs:54); `fix_up_shape_type_ids` (comparer/fixups.rs:75); `fix_up_group_ids` (comparer/fixups.rs:102)

#### `fix_up_doc_pr_ids`  (L32-48, pub)
`pub fn fix_up_doc_pr_ids(dom: &mut Dom, root: NodeId)`

- **Does:** Renumber every drawing's `wp:docPr/@id` starting from 1, in document order.
- **Calls (crate):** `id_attr` (comparer/fixups.rs:23) L33; `Dom::descendants` (xmllinq/mod.rs:668) L34; `Dom::name` (xmllinq/mod.rs:526) L38,39; `XName::local_name` (xmllinq/mod.rs:208) (by name) L38; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L46
- **Calls (std/external/unresolved):** `.as_ref`, `.collect`, `.filter`, `.insert`, `.into_iter`, `.is_some_and`, `.to_string`, `.zip`, `Some`, `WP::name`, `std::collections::HashSet::new`
- **Called by:** `fix_up_drawing_ids_in_package_within` (comparer/fixups.rs:126); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215); `compare_documents_impl` (document_comparer.rs:6521)

#### `fix_up_shape_ids`  (L54-70, pub)
`pub fn fix_up_shape_ids(dom: &mut Dom, root: NodeId)`

- **Does:** Renumber every `v:shape/@id` from 1, in document order, and rewrite any sibling `o:OLEObject/@ShapeID` that referenced the old id so the OLE ↔ shape linkage survives the renumber.
- **Calls (crate):** `id_attr` (comparer/fixups.rs:23) L55; `XName::get` (xmllinq/mod.rs:180) L56; `Dom::descendants` (xmllinq/mod.rs:668) L57; `Dom::attribute` (xmllinq/mod.rs:874) L58,64; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L60,65; `Dom::parent` (xmllinq/mod.rs:597) L62; `Dom::elements` (xmllinq/mod.rs:640) L63
- **Calls (std/external/unresolved):** `.clone`, `.map`, `.to_string`, `.zip`, `O::name`, `Some`, `VML::name`
- **Called by:** `fix_up_drawing_ids_in_package_within` (comparer/fixups.rs:126); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215); `compare_documents_impl` (document_comparer.rs:6521)

#### `fix_up_shape_type_ids`  (L75-95, pub)
`pub fn fix_up_shape_type_ids(dom: &mut Dom, root: NodeId)`

- **Does:** Renumber every `v:shapetype/@id` from 1, in document order, and rewrite the sibling `v:shape/@type` reference to the new id (both the bare value and the `#…` prefixed form).
- **Calls (crate):** `id_attr` (comparer/fixups.rs:23) L76; `XName::get` (xmllinq/mod.rs:180) L77; `Dom::descendants` (xmllinq/mod.rs:668) L78; `Dom::attribute` (xmllinq/mod.rs:874) L79,85; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L81,87,89; `Dom::parent` (xmllinq/mod.rs:597) L82; `Dom::elements` (xmllinq/mod.rs:640) L84
- **Calls (std/external/unresolved):** `.map`, `.to_string`, `.zip`, `Some`, `VML::name`
- **Macros:** `format!`
- **Called by:** `fix_up_drawing_ids_in_package_within` (comparer/fixups.rs:126); `try_compare_bodies_faithful_with_notes` (comparer/mod.rs:215); `compare_documents_impl` (document_comparer.rs:6521)

#### `fix_up_group_ids`  (L102-107, pub)
`pub fn fix_up_group_ids(dom: &mut Dom, root: NodeId)`

- **Does:** Renumber every `v:group/@id` from 1, in document order.
- **Calls (crate):** `id_attr` (comparer/fixups.rs:23) L103; `Dom::descendants` (xmllinq/mod.rs:668) L104; `Dom::set_attribute_value` (xmllinq/mod.rs:977) L105
- **Calls (std/external/unresolved):** `.to_string`, `.zip`, `Some`, `VML::name`
- **Called by:** none found in src

#### `fix_up_drawing_ids_in_package`  (L121-123, pub)
`pub fn fix_up_drawing_ids_in_package(docx: &[u8]) -> Result<Vec<u8>, crate::opc::OpcError>`

- **Does:** Apply the three drawing-id fixups (doc / shape / shape-type) to a package's main document and write the redlined package back to bytes.
- **Calls (crate):** `fix_up_drawing_ids_in_package_within` (comparer/fixups.rs:126) L122; `InputLimits::compare` (admission.rs:87) L122
- **Called by:** `append_documents` (append.rs:152); `compare_documents_impl` (document_comparer.rs:6521); `repair` (validate.rs:2623)

#### `fix_up_drawing_ids_in_package_within`  (L126-153, pub(crate))
`pub(crate) fn fix_up_drawing_ids_in_package_within( docx: &[u8], limits: crate::admission::InputLimits, ) -> Result<Vec<u8>, crate::opc::OpcError>`

- **Does:** Keep the identical-input comparer's already-admitted budget through the fixup.
- **Calls (crate):** `PartFs::open_within` (opc/mod.rs:593) L132; `PartFs::main_document_part` (opc/mod.rs:1070) L133; `PartFs::part_string` (opc/mod.rs:636) L136; `Dom::new` (xmllinq/mod.rs:387) L141; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L142; `Dom::root` (xmllinq/mod.rs:822) L143; `fix_up_doc_pr_ids` (comparer/fixups.rs:32) L148; `fix_up_shape_ids` (comparer/fixups.rs:54) L149; `fix_up_shape_type_ids` (comparer/fixups.rs:75) L150; `PartFs::set_part` (opc/mod.rs:661) L151; `Dom::serialize_element` (xmllinq/mod.rs:1315) L151; `PartFs::to_zip` (opc/mod.rs:749) L152
- **Calls (std/external/unresolved):** `.into_bytes`, `.to_string`, `.unwrap_or_else`, `Err`, `crate::opc::OpcError::PartNotFound`
- **Macros:** `format!`
- **Called by:** `fix_up_drawing_ids_in_package` (comparer/fixups.rs:121); `compare_documents_impl` (document_comparer.rs:6521)
