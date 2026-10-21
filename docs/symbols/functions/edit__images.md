# Functions and methods in `src/edit/images.rs`

3 non-test, 2 test. Part of the [symbol manifest](../README.md).

#### `decode_base64`  (L38-83, pub(super))
`pub(super) fn decode_base64(text: &str) -> Result<Vec<u8>, String>`

- **Does:** Decode standard base64 (RFC 4648, `+/`, optional `=` padding); ASCII whitespace is skipped so wrapped encodings are accepted.
- **Calls (std/external/unresolved):** `.bytes`, `.into`, `.is_ascii_whitespace`, `.is_empty`, `.len`, `.push`, `Err`, `Ok`, `Vec::with_capacity`, `from`, `u32::from`
- **Macros:** `format!`
- **Called by:** `picture` (edit/images.rs:86); `tests::decodes_padded_unpadded_and_wrapped_base64` (edit/images.rs:201); `tests::refuses_malformed_base64` (edit/images.rs:211)

#### `picture`  (L86-137, pub(super))
`pub(super) fn picture( base64: &str, content_type: Option<&str>, width_emu: Option<u64>, alt: &str, ) -> Result<Picture, (String, String)>`

- **Does:** Decode, sniff and size the picture; `(code, message)` on refusal.
- **Calls (crate):** `check_text` (edit.rs:7094) L94; `invalid` (ambiguous: `invalid` (append.rs:309); `invalid` (edit/v2.rs:458); `invalid` (markdown/from_docx/ooxml.rs:31)) L94,110,127; `decode_base64` (edit/images.rs:38) L114; `read_picture` (markdown/write.rs:6479) L115
- **Calls (std/external/unresolved):** `.contains`, `.into`, `.is_empty`, `.join`, `.map_err`, `.max`, `.ok_or_else`, `.to_string`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `Transaction::resolve_one_unreported` (edit.rs:3545)

#### `Transaction::image_paragraph`  (L143-192, pub(super))
`pub(super) fn image_paragraph( &mut self, picture: &Picture, used: &mut HashSet<String>, drawing_id: u32, ) -> NodeId`

- **Does:** Add the picture's media part, content type and relationship to the copy and build its paragraph.
- **Calls (crate):** `PartFs::parts` (opc/mod.rs:733) L150; `PartFs::set_part` (opc/mod.rs:661) L164; `PartFs::content_type_for` (opc/mod.rs:837) (by name) L165; `PartFs::add_content_type_default` (opc/mod.rs:852) (by name) L171; `PartFs::add_document_relationship` (opc/mod.rs:1003) L175; `relative_rel_target` (opc/mod.rs:180) L178; `drawing_xml` (markdown/xml.rs:2441) L181; `Dom::parse_xdocument` (xmllinq/mod.rs:1299) L188; `Dom::root` (xmllinq/mod.rs:822) L189; `Dom::remove` (xmllinq/mod.rs:1118) L190
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.contains`, `.expect`, `.find`, `.insert`, `.into_iter`, `.is_empty`, `.is_none`, `.map_or`, `.map`, `.rsplit_once`, `String::new`
- **Macros:** `format!`
- **Called by:** `Transaction::apply_insertions` (edit.rs:6186)

## Test functions (2)

- `tests::decodes_padded_unpadded_and_wrapped_base64` (L201): _no doc comment_
- `tests::refuses_malformed_base64` (L211): _no doc comment_
