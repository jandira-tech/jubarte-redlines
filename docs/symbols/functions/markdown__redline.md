# Functions and methods in `src/markdown/redline.rs`

2 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `<RedlineOptions as std::fmt::Debug>::fmt`  (L67-74, impl std::fmt::Debug)
`fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.debug_struct`, `.field`, `.finish`, `.is_some`, `.map`
- **Called by:** none found in src

#### `redline`  (L109-155, pub)
`pub fn redline( original: Source<'_>, modified: Source<'_>, options: &RedlineOptions<'_>, ) -> Result<Vec<u8>, MarkdownError>`

- **Does:** Compares `original` against `modified` and returns a Word tracked-changes document: the original with every difference as a revision, as [`crate::document_comparer::compare_documents_with_settings`] writes it.
- **Calls (crate):** `markdown_to_docx` (markdown/write.rs:59) L119; `apply_markdown` (markdown/patch.rs:121) L144,148; `word` (ambiguous, 5 candidates) L151; `compare_documents_with_settings` (document_comparer.rs:6423) L153
- **Calls (std/external/unresolved):** `.clone`, `.map_err`, `.to_vec`, `MarkdownError::Package`, `Ok`
- **Macros:** `format!`
- **Called by:** `patch_documents` (markdown/unified.rs:337)
