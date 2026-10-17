# Functions and methods in `src/markdown/from_docx/media.rs`

4 non-test, 4 test. Part of the [symbol manifest](../README.md).

#### `Extracted::new`  (L25-30, pub(crate))
`pub(crate) fn new(dir: &str) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.to_string`, `.trim_end_matches`, `BTreeMap::new`
- **Called by:** `tests::names_by_content_and_collects_once` (markdown/from_docx/media.rs:111); `tests::refuses_non_images_and_huge_images` (markdown/from_docx/media.rs:127); `tests::quotes_odd_paths` (markdown/from_docx/media.rs:138); `ooxml_package_boundary_tests::media_relationships_extract_owned_bytes_once_and_leave_non_images_as_alt_text` (markdown/from_docx/ooxml.rs:1388); `ooxml_package_boundary_tests::media_part_limit_retains_the_first_512_authored_targets_in_relationship_order` (markdown/from_docx/ooxml.rs:1452)

#### `Extracted::add`  (L34-58, pub(crate))
`pub(crate) fn add(&mut self, bytes: &[u8], alt: &str) -> Option<String>`

- **Does:** `![alt](dir/name.ext)` for a raster picture, collecting its bytes; `None` for anything else.
- **Calls (crate):** `raster_extension` (markdown/from_docx/media.rs:77) L36; `destination` (markdown/from_docx/media.rs:63) L57
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.entry`, `.is_empty`, `.iter`, `.join`, `.map`, `.or_insert_with`, `.replace`, `.split_whitespace`, `.take`, `.to_vec`, `Sha256::digest`, `Some`
- **Macros:** `format!`
- **Called by:** `tests::names_by_content_and_collects_once` (markdown/from_docx/media.rs:111)

#### `destination`  (L63-73)
`fn destination(path: &str) -> String`

- **Does:** A link destination: `<...>` around paths with spaces or parentheses, and `{`, `}` encoded so they never form CriticMarkup delimiters.
- **Calls (std/external/unresolved):** `.any`, `.chars`, `.is_whitespace`, `.replace`
- **Macros:** `format!`, `matches!`
- **Called by:** `Extracted::add` (markdown/from_docx/media.rs:34); `tests::quotes_odd_paths` (markdown/from_docx/media.rs:138)

#### `raster_extension`  (L77-93)
`fn raster_extension(bytes: &[u8]) -> Option<&'static str>`

- **Does:** The extension of a PNG, JPEG, GIF, BMP, TIFF or WebP picture of at most [`MAX_PIXELS`], read from its header.
- **Calls (crate):** `.format` (ambiguous: `ListKind::format` (edit.rs:642); `RunSpec::format` (edit.rs:891); `Writer::format` (markdown/write.rs:2950)) L82
- **Calls (std/external/unresolved):** `.into_dimensions`, `.ok`, `.then_some`, `.with_guessed_format`, `image::ImageReader::new`, `std::io::Cursor::new`, `u64::from`
- **Called by:** `Extracted::add` (markdown/from_docx/media.rs:34)

## Test functions (4)

- `tests::png` (L100): _no doc comment_
- `tests::names_by_content_and_collects_once` (L111): _no doc comment_
- `tests::refuses_non_images_and_huge_images` (L127): _no doc comment_
- `tests::quotes_odd_paths` (L138): _no doc comment_
