# Functions and methods in `src/edit/record.rs`

8 non-test, 9 test. Part of the [symbol manifest](../README.md).

#### `plan_flag_conflict`  (L39-57, pub)
`pub fn plan_flag_conflict(recording: bool, given: &[&str], plan: &str) -> Option<String>`

- **Does:** Why `given` (the flags the command line spells out) cannot go with `--plan plan`: `recording` is `--plan` with operation flags, otherwise the plan is applied.
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.is_empty`, `.iter`, `.to_string`, `Some`
- **Macros:** `format!`
- **Called by:** `validate_matches` (cli.rs:3007); `tests::flags_are_refused_on_the_side_of_plan_they_do_not_belong_to` (edit/record.rs:439)

#### `Recorded::json`  (L84-86, pub)
`pub fn json(&self) -> String`

- **Does:** The plan as written to PLAN.json: pretty JSON with a final newline.
- **Calls (std/external/unresolved):** `.expect`, `serde_json::to_string_pretty`
- **Called by:** `record_plan` (bin/jubarte.rs:2625) (?); `record_json` (edit/record.rs:270) (?); `tests::a_recording_pins_the_source_and_counts_on_from_the_held_plan` (edit/record.rs:348) (?); `tests::a_recording_of_revisions_only_says_so` (edit/record.rs:375) (?); `tests::recorded_revisions_count_each_long_id_once` (edit/record.rs:407) (?)

#### `Recorded::summary`  (L91-126, pub)
`pub fn summary(&self, file: &str, plan: &str, default_plan: bool, dry_run: bool) -> String`

- **Does:** The line a recording prints: what was added, what the plan now holds, and the command that applies it.
- **Calls (crate):** `side` (markdown/borders.rs:189) (by name) L96; `reordered_retained_control_identity_regression::ids` (comparer/produce.rs:2433) (by name) L100
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.len`, `.map_or`, `.to_string`, `s`
- **Macros:** `format!`
- **Called by:** `record_plan` (bin/jubarte.rs:2625) (?); `record_json` (edit/record.rs:270) (?); `tests::a_recording_pins_the_source_and_counts_on_from_the_held_plan` (edit/record.rs:348) (?); `tests::a_recording_of_revisions_only_says_so` (edit/record.rs:375) (?)

#### `plan_target`  (L131-140, pub(crate))
`pub(crate) fn plan_target(plan: &Path) -> PathBuf`

- **Does:** Where a recording is written: the file a symbolic link names (the link stays a link), else `plan` itself.
- **Calls (std/external/unresolved):** `.join`, `.parent`, `.to_path_buf`, `.unwrap_or`, `Path::new`, `std::fs::canonicalize`, `std::fs::read_link`
- **Called by:** `write_plan` (edit/record.rs:161)

#### `partial_path`  (L145-150, pub)
`pub fn partial_path(target: &Path) -> PathBuf`

- **Does:** Legacy temporary-name helper, retained for API compatibility.
- **Calls (std/external/unresolved):** `.file_name`, `.into_owned`, `.map_or_else`, `.to_string_lossy`, `.with_file_name`, `std::process::id`
- **Macros:** `format!`
- **Called by:** `tests::a_plan_is_written_through_its_link_and_leaves_no_partial` (edit/record.rs:459)

#### `write_plan`  (L161-178, pub)
`pub fn write_plan(plan: &Path, json: &str) -> std::io::Result<Vec<String>>`

- **Does:** Write `json` (a [`Recorded::json`]) to PLAN.json at `plan`, atomically: a precommit error leaves the plan as it was.
- **Calls (crate):** `plan_target` (edit/record.rs:131) L164; `publish_paths` (publication.rs:211) L165
- **Calls (std/external/unresolved):** `.as_bytes`, `.display`, `.kind`, `.len`, `.map_err`, `.map`, `Err`, `std::io::Error::new`
- **Macros:** `format!`
- **Called by:** `record_plan` (bin/jubarte.rs:2625); `tests::a_plan_is_written_through_its_link_and_leaves_no_partial` (edit/record.rs:459)

#### `record`  (L191-231, pub)
`pub fn record( verb: Verb, ops: &[FlagOp], author: &str, date: Option<&str>, existing: Option<ExistingRevisions>, source: &[u8], held: Option<&str>, ) -> Result<Recorded, RecordError>`

- **Does:** The operations `ops` recorded into `held` (PLAN.json's text, `None` before the first recording), pinned to `source`'s SHA-256.
- **Calls (crate):** `ChangeFilter::normalized` (changes.rs:349) (by name) L214; `plan_from_flags` (edit/flags.rs:85) L222; `source_sha256` (inspect.rs:299) L223; `record_into` (edit/flags.rs:690) L224; `renumber_notes` (edit/flags.rs:825) L228
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.clone`, `.collect`, `.count`, `.difference`, `.filter_map`, `.flatten`, `.into_iter`, `.len`, `.map_err`, `.map_or`, `.map`, `.to_string`, `.transpose`, `.unwrap_or_default`, `.unwrap_or_else`, `Ok`, `RecordError::Plan`, `Some`, `revisions`
- **Called by:** `record_plan` (bin/jubarte.rs:2625); `public_memory_anchor_boundary_tests::latest_thread_selection_preserves_complete_winning_source_records_and_thread_order` (comments.rs:1541) (?); `public_body_heuristic_story_scope_tests::pass_report` (comparer/finalize.rs:32644) (?); `complete_root_style_change_records` (document_comparer.rs:1873) (?); `record_json` (edit/record.rs:270); `tests::once` (edit/record.rs:335)

#### `record_json`  (L270-321, pub)
`pub fn record_json( verb: &str, operations: &str, source: &[u8], held: Option<&str>, options: &str, ) -> Result<String, RecordError>`

- **Does:** [`record`] over JSON, for the Python and npm CLIs: `verb` is `edit` or `add`, `operations` the parsed command's `operations` array, `held` PLAN.json's text, `options` a `RecordOptions` object.
- **Calls (crate):** `usage` (ambiguous: `usage` (bin/jubarte/check.rs:95); `usage` (bin/jubarte/edit_template.rs:292); `usage` (bin/jubarte/edit_typed.rs:962)) L281,284,286,293,301; `record` (edit/record.rs:191) L307; `Recorded::json` (edit/record.rs:84) (by name) L311; `.summary` (ambiguous: `Recorded::summary` (edit/record.rs:91); `Opened::summary` (inspect.rs:765)) L313
- **Calls (std/external/unresolved):** `.as_deref`, `.expect`, `.map_err`, `.map`, `.to_string`, `.transpose`, `.unwrap_or`, `Err`, `Ok`, `RecordError::Usage`, `Some`, `serde_json::from_str`, `serde_json::to_string`
- **Macros:** `format!`, `serde_json::json!`
- **Called by:** `tests::record_json_reads_the_options_the_bindings_pass` (edit/record.rs:504)

## Test functions (9)

- `tests::rewrite` (L327): _no doc comment_
- `tests::once` (L335): _no doc comment_
- `tests::a_recording_pins_the_source_and_counts_on_from_the_held_plan` (L348): _no doc comment_
- `tests::a_recording_of_revisions_only_says_so` (L375): _no doc comment_
- `tests::recorded_revisions_count_each_long_id_once` (L407): Codex on #433: a held plan's ids count as `record_into` joins them, so `r1+2` held is two ids and a duplicate spelling one.
- `tests::a_held_file_that_is_not_a_plan_is_a_plan_error` (L427): _no doc comment_
- `tests::flags_are_refused_on_the_side_of_plan_they_do_not_belong_to` (L439): _no doc comment_
- `tests::a_plan_is_written_through_its_link_and_leaves_no_partial` (L459): _no doc comment_
- `tests::record_json_reads_the_options_the_bindings_pass` (L504): _no doc comment_
