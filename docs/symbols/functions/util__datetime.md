# Functions and methods in `src/util/datetime.rs`

1 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `parse_datetime`  (L11-76, pub(crate))
`pub(crate) fn parse_datetime(value: &str) -> Result<String, String>`

- **Does:** A revision timestamp: an xsd:dateTime as Word writes `w:date` (`2026-10-01T09:00:00Z`, optional fraction and offset of at most 14:00).
- **Calls (std/external/unresolved):** `.all`, `.as_bytes`, `.bytes`, `.contains`, `.count`, `.flatten`, `.get`, `.is_ascii_digit`, `.iter`, `.len`, `.min`, `.ok`, `.parse`, `.strip_prefix`, `.take_while`, `.then`, `.to_string`, `.wrapping_sub`, `Err`, `Ok`, `bad`, `digits`, `u32::from`
- **Called by:** `creator_created` (markdown/from_docx/mod.rs:1302); `is_date` (markdown/write.rs:5457)
