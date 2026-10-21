# Functions and methods in `src/edit/native_keys.rs`

2 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `para_id`  (L18-24, pub(crate))
`pub(crate) fn para_id(value: &str) -> Option<u32>`

- **Does:** A `w14:paraId` attribute value as a native long-ID key: exactly eight ASCII hex digits that parse into Word's range `1..=0x7FFFFFFF` ([MS-DOCX] requires paraIds below 0x80000000).
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.contains`, `.is_ascii_hexdigit`, `.len`, `.ok`, `.then_some`, `u32::from_str_radix`
- **Called by:** `native_para_id` (edit/for_edit.rs:897); `story_keys` (expect.rs:945)

#### `revision_id`  (L30-39, pub(super))
`pub(super) fn revision_id(value: &str) -> Option<i32>`

- **Does:** A revision `w:id` attribute value as a native long-ID key: non-empty, digits with an optional sign, parsing as the `i32` width Word writes.
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.is_ascii_digit`, `.is_empty`, `.ok`, `.parse`
- **Macros:** `matches!`
- **Called by:** `revision_pair` (edit/for_edit.rs:904); `revision_long_id` (edit/v2_apply.rs:780)
