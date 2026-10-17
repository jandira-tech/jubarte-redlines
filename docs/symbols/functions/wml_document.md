# Functions and methods in `src/wml_document.rs`

9 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `WmlDocument::from_bytes`  (L50-59, pub)
`pub fn from_bytes(bytes: &[u8]) -> Result<Self, OpcError>`

- **Does:** Opens `bytes` as a `.docx`, admits it under [`InputLimits::compare`](crate::admission::InputLimits::compare), and lazily parses the main document part on the first [`Self::main_document`] call.
- **Calls (crate):** `admit` (admission.rs:310) L51; `InputLimits::compare` (admission.rs:75) L51; `PartFs::open` (opc/mod.rs:456) L55; `Dom::new` (xmllinq/mod.rs:387) L56
- **Calls (std/external/unresolved):** `.map_err`, `.to_vec`, `Ok`, `String::new`
- **Called by:** none found in src

#### `WmlDocument::bytes`  (L64-66, pub)
`pub fn bytes(&self) -> &[u8]`

- **Does:** The bytes the document was opened from.
- **Calls:** none
- **Called by:** none found in src

#### `WmlDocument::main_document_part_name`  (L71-75, pub)
`pub fn main_document_part_name(&self) -> String`

- **Does:** The main document part name (`word/document.xml` unless OPC relationships name otherwise).
- **Calls (crate):** `PartFs::main_document_part` (opc/mod.rs:829) L72
- **Calls (std/external/unresolved):** `.to_string`, `.unwrap_or_else`
- **Called by:** `WmlDocument::main_document` (wml_document.rs:85); `WmlDocument::main_document_root` (wml_document.rs:105)

#### `WmlDocument::main_document`  (L85-97, pub)
`pub fn main_document(&mut self) -> Result<NodeId, OpcError>`

- **Does:** Parses the main document part (once) and returns its document node.
- **Calls (crate):** `WmlDocument::main_document_part_name` (wml_document.rs:71) L89; `PartFs::part_string` (opc/mod.rs:498) (by name) L90; `Dom::parse_xdocument` (xmllinq/mod.rs:1283) L94
- **Calls (std/external/unresolved):** `.clone`, `.ok_or_else`, `Ok`, `OpcError::PartNotFound`, `Some`
- **Called by:** `WmlDocument::main_document_root` (wml_document.rs:105)

#### `WmlDocument::main_document_root`  (L105-111, pub)
`pub fn main_document_root(&mut self) -> Result<NodeId, OpcError>`

- **Does:** The root element (`<w:document>`) of the main document part.
- **Calls (crate):** `WmlDocument::main_document` (wml_document.rs:85) L106; `WmlDocument::main_document_part_name` (wml_document.rs:71) L107; `Dom::root` (xmllinq/mod.rs:822) L108
- **Calls (std/external/unresolved):** `.ok_or_else`, `OpcError::PartNotFound`
- **Macros:** `format!`
- **Called by:** none found in src

#### `WmlDocument::dom`  (L115-117, pub)
`pub fn dom(&self) -> &Dom`

- **Does:** The arena DOM (read-only).
- **Calls:** none
- **Called by:** none found in src

#### `WmlDocument::dom_mut`  (L120-122, pub)
`pub fn dom_mut(&mut self) -> &mut Dom`

- **Does:** The arena DOM (mutable).
- **Calls:** none
- **Called by:** none found in src

#### `WmlDocument::part_fs`  (L125-127, pub)
`pub fn part_fs(&self) -> &PartFs`

- **Does:** The opened OPC package (read-only), by which every other part and relationship is read.
- **Calls:** none
- **Called by:** none found in src

#### `WmlDocument::part_fs_mut`  (L129-131, pub)
`pub fn part_fs_mut(&mut self) -> &mut PartFs`

- **Does:** The opened OPC package (mutable).
- **Calls:** none
- **Called by:** none found in src
