# Functions and methods in `src/edit/record.rs`

8 non-test, 9 test. Part of the [symbol manifest](../README.md).

#### `plan_flag_conflict`  (L39-57, pub)
`pub fn plan_flag_conflict(recording: bool, given: &[&str], plan: &str) -> Option<String>`

- **Does:** Why `given` (the flags the command line spells out) cannot go with `--plan plan`: `recording` is `--plan` with operation flags, otherwise the plan is applied.
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.is_empty`, `.iter`, `.to_string`, `Some`
- **Macros:** `format!`
- **Called by:** `validate_matches` (cli.rs:2566); `tests::flags_are_refused_on_the_side_of_plan_they_do_not_belong_to` (edit/record.rs:425)

#### `Recorded::json`  (L84-86, pub)
`pub fn json(&self) -> String`

- **Does:** The plan as written to PLAN.json: pretty JSON with a final newline.
- **Calls (std/external/unresolved):** `.expect`, `serde_json::to_string_pretty`
- **Called by:** `record_plan` (bin/jubarte.rs:1887) (?); `record_json` (edit/record.rs:257) (?); `tests::a_recording_pins_the_source_and_counts_on_from_the_held_plan` (edit/record.rs:334) (?); `tests::a_recording_of_revisions_only_says_so` (edit/record.rs:361) (?); `tests::recorded_revisions_count_each_long_id_once` (edit/record.rs:393) (?)

#### `Recorded::summary`  (L91-126, pub)
`pub fn summary(&self, file: &str, plan: &str, default_plan: bool, dry_run: bool) -> String`

- **Does:** The line a recording prints: what was added, what the plan now holds, and the command that applies it.
- **Calls (crate):** `side` (markdown/borders.rs:189) (by name) L96; `reordered_retained_control_identity_regression::ids` (comparer/produce.rs:2434) (by name) L100
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.len`, `.map_or`, `.to_string`, `s`
- **Macros:** `format!`
- **Called by:** `record_plan` (bin/jubarte.rs:1887) (?); `record_json` (edit/record.rs:257) (?); `tests::a_recording_pins_the_source_and_counts_on_from_the_held_plan` (edit/record.rs:334) (?); `tests::a_recording_of_revisions_only_says_so` (edit/record.rs:361) (?)

#### `plan_target`  (L131-140, pub)
`pub fn plan_target(plan: &Path) -> PathBuf`

- **Does:** Where a recording is written: the file a symbolic link names (the link stays a link), else `plan` itself.
- **Calls (std/external/unresolved):** `.join`, `.parent`, `.to_path_buf`, `.unwrap_or`, `Path::new`, `std::fs::canonicalize`, `std::fs::read_link`
- **Called by:** `write_plan` (edit/record.rs:160)

#### `partial_path`  (L146-151, pub)
`pub fn partial_path(target: &Path) -> PathBuf`

- **Does:** The temporary file a recording is written to before it replaces `target`: hidden, beside it, and named for this process, so it names no other plan or document.
- **Calls (std/external/unresolved):** `.file_name`, `.into_owned`, `.map_or_else`, `.to_string_lossy`, `.with_file_name`, `std::process::id`
- **Macros:** `format!`
- **Called by:** `write_plan` (edit/record.rs:160); `tests::a_plan_is_written_through_its_link_and_leaves_no_partial` (edit/record.rs:445)

#### `write_plan`  (L160-168, pub)
`pub fn write_plan(plan: &Path, json: &str) -> std::io::Result<()>`

- **Does:** Write `json` (a [`Recorded::json`]) to PLAN.json at `plan`, atomically: a failed write leaves the plan as it was and no partial file behind.
- **Calls (crate):** `plan_target` (edit/record.rs:131) L161; `partial_path` (edit/record.rs:146) L162
- **Calls (std/external/unresolved):** `.and_then`, `.inspect_err`, `std::fs::remove_file`, `std::fs::rename`, `std::fs::write`
- **Called by:** `record_plan` (bin/jubarte.rs:1887); `tests::a_plan_is_written_through_its_link_and_leaves_no_partial` (edit/record.rs:445)

#### `record`  (L181-221, pub)
`pub fn record( verb: Verb, ops: &[FlagOp], author: &str, date: Option<&str>, existing: Option<ExistingRevisions>, source: &[u8], held: Option<&str>, ) -> Result<Recorded, RecordError>`

- **Does:** The operations `ops` recorded into `held` (PLAN.json's text, `None` before the first recording), pinned to `source`'s SHA-256.
- **Calls (crate):** `ChangeFilter::normalized` (changes.rs:130) (by name) L204; `plan_from_flags` (edit/flags.rs:76) L212; `source_sha256` (inspect.rs:271) L213; `record_into` (edit/flags.rs:574) L214; `renumber_notes` (edit/flags.rs:696) L218
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.clone`, `.collect`, `.count`, `.difference`, `.filter_map`, `.flatten`, `.into_iter`, `.len`, `.map_err`, `.map_or`, `.map`, `.to_string`, `.transpose`, `.unwrap_or_default`, `.unwrap_or_else`, `Ok`, `RecordError::Plan`, `Some`, `revisions`
- **Called by:** `record_plan` (bin/jubarte.rs:1887); `public_memory_anchor_boundary_tests::latest_thread_selection_preserves_complete_winning_source_records_and_thread_order` (comments.rs:1460) (?); `complete_root_style_change_records` (document_comparer.rs:1873) (?); `record_json` (edit/record.rs:257); `tests::once` (edit/record.rs:321)

#### `record_json`  (L257-307, pub)
`pub fn record_json( verb: &str, operations: &str, source: &[u8], held: Option<&str>, options: &str, ) -> Result<String, RecordError>`

- **Does:** [`record`] over JSON, for the Python and npm CLIs: `verb` is `edit` or `add`, `operations` the parsed command's `operations` array, `held` PLAN.json's text, `options` a [`RecordOptions`] object.
- **Calls (crate):** `record` (edit/record.rs:181) L294; `Recorded::json` (edit/record.rs:84) (by name) L297; `.summary` (ambiguous: `Recorded::summary` (edit/record.rs:91); `Opened::summary` (inspect.rs:545)) L299
- **Calls (std/external/unresolved):** `.as_deref`, `.expect`, `.map_err`, `.map`, `.to_string`, `.transpose`, `.unwrap_or`, `Err`, `Ok`, `RecordError::Usage`, `Some`, `serde_json::from_str`, `serde_json::to_string`, `usage`
- **Macros:** `format!`, `serde_json::json!`
- **Called by:** `tests::record_json_reads_the_options_the_bindings_pass` (edit/record.rs:490)

## Test functions (9)

- `tests::rewrite` (L313): _no doc comment_
- `tests::once` (L321): _no doc comment_
- `tests::a_recording_pins_the_source_and_counts_on_from_the_held_plan` (L334): _no doc comment_
- `tests::a_recording_of_revisions_only_says_so` (L361): _no doc comment_
- `tests::recorded_revisions_count_each_long_id_once` (L393): Codex on #433: a held plan's ids count as `record_into` joins them, so `r1+2` held is two ids and a duplicate spelling one.
- `tests::a_held_file_that_is_not_a_plan_is_a_plan_error` (L413): _no doc comment_
- `tests::flags_are_refused_on_the_side_of_plan_they_do_not_belong_to` (L425): _no doc comment_
- `tests::a_plan_is_written_through_its_link_and_leaves_no_partial` (L445): _no doc comment_
- `tests::record_json_reads_the_options_the_bindings_pass` (L490): _no doc comment_
