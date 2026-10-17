# Functions and methods in `src/edit/runs.rs`

2 non-test, 1 test. Part of the [symbol manifest](../README.md).

#### `Transaction::resolve_format_run`  (L17-33, pub(super))
`pub(super) fn resolve_format_run( &self, projection: &Projection, find: &str, occurrence: Option<usize>, format: &RunFormat, outcome: &mut EditOutcome, ) -> Result<(usize, usize), (String, String)>`

- **Does:** Resolve a `format_run`: the range to format, or the error code and message.
- **Calls (crate):** `check_format` (edit.rs:6434) L31; `Transaction::find_range` (edit.rs:3901) L32
- **Calls (std/external/unresolved):** `.into`, `.map_err`, `.to_string`, `Err`, `RunFormat::default`
- **Called by:** `Transaction::resolve_one` (edit.rs:2528)

#### `format_overlaps_edit`  (L39-48, pub(super))
`pub(super) fn format_overlaps_edit( (format_start, format_end): (usize, usize), (start, end): (usize, usize), ) -> bool`

- **Does:** A `format_run` range and a text edit's range in one paragraph conflict when the edit changes text inside the formatted range: a replacement or deletion overlapping it, or an insertion strictly inside it.
- **Calls:** none
- **Called by:** `tests::insertions_at_the_edges_and_edits_beside_the_range_do_not_overlap` (edit/runs.rs:56); `Transaction::check_conflicts` (edit.rs:4044)

## Test functions (1)

- `tests::insertions_at_the_edges_and_edits_beside_the_range_do_not_overlap` (L56): _no doc comment_
