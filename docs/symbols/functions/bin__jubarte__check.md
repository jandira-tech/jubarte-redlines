# Functions and methods in `src/bin/jubarte/check.rs`

8 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `finish`  (L30-70)
`fn finish( envelope: Result<Envelope, ResultDataError>, line: Option<String>, json: bool, quiet: bool, ) -> ExitCode`

- **Does:** Print the envelope (`--json`) or the final scope line, and map the proposal §12 exit code.
- **Calls (crate):** `exit_status_for_envelope` (result.rs:1234) L43; `write_stdout` (bin/jubarte.rs:1240) (by name) L49,57
- **Calls (std/external/unresolved):** `.expect`, `.into`, `ExitCode::from`, `serde_json::to_string`
- **Macros:** `eprintln!`, `format!`
- **Called by:** `run` (bin/jubarte/check.rs:125)

#### `scope_line`  (L74-93)
`fn scope_line(state: &str, report: &ExpectationReport, failed: usize) -> String`

- **Does:** The final human line that states the check's scope, e.g.
- **Calls (std/external/unresolved):** `.push_str`, `.push`
- **Macros:** `format!`
- **Called by:** `run` (bin/jubarte/check.rs:125)

#### `usage`  (L95-101)
`fn usage(source: Source, message: String) -> Result<Envelope, ResultDataError>`

- **Does:** _no doc comment_
- **Calls (crate):** `Envelope::new_error` (result.rs:916) L96; `Error::with_message` (result.rs:379) (by name) L99; `Error::new` (result.rs:360) L99
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `run` (bin/jubarte/check.rs:125); `run` (bin/jubarte.rs:2862) (?); `record_json` (edit/record.rs:270) (?)

#### `refused`  (L103-105)
`fn refused(source: Source, error: Error) -> Result<Envelope, ResultDataError>`

- **Does:** _no doc comment_
- **Calls (crate):** `Envelope::refused` (result.rs:895) L104
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `run` (bin/jubarte/check.rs:125)

#### `io_error`  (L107-109)
`fn io_error(message: String) -> Error`

- **Does:** _no doc comment_
- **Calls (crate):** `Error::with_message` (result.rs:379) (by name) L108; `Error::new` (result.rs:360) L108
- **Called by:** `run` (bin/jubarte/check.rs:125)

#### `path_only`  (L113-115)
`fn path_only(file: &Path) -> Source`

- **Does:** The inspected file's own `Source` for failures that never read its bytes: the user-named path, no invented digest (#1084 review F5).
- **Calls (crate):** `.with_path` (ambiguous: `AnchorRefusal::with_path` (edit/anchor_error.rs:100); `Source::with_path` (result.rs:482)) L114; `Source::from_digest` (result.rs:472) L114; `absolute` (bin/jubarte.rs:168) (by name) L114
- **Called by:** `run` (bin/jubarte/check.rs:125)

#### `first_of`  (L118-120)
`fn first_of(report: &ExpectationReport, outcome: Outcome) -> Option<&CheckOutcome>`

- **Does:** One named failing or refused check, for the error message.
- **Calls (std/external/unresolved):** `.find`, `.iter`
- **Called by:** `run` (bin/jubarte/check.rs:125)

#### `run`  (L125-298, pub(super))
`pub(super) fn run(file: &Path, expect: Option<&Path>, json: bool, quiet: bool) -> ExitCode`

- **Does:** Run the check.
- **Calls (crate):** `finish` (bin/jubarte/check.rs:30) L127,143,158,170,184,196...; `usage` (bin/jubarte/check.rs:95) L128,166; `path_only` (bin/jubarte/check.rs:113) L129,144; `io_error` (bin/jubarte/check.rs:107) L142,157; `Envelope::new_error` (result.rs:916) L144,159,215; `.with_path` (ambiguous: `AnchorRefusal::with_path` (edit/anchor_error.rs:100); `Source::with_path` (result.rs:482)) L151; `Source::new` (result.rs:463) L151; `absolute` (bin/jubarte.rs:168) (by name) L151; `parse_expectations` (expect.rs:410) (by name) L181; `refused` (bin/jubarte/check.rs:103) L197; `Error::with_message` (result.rs:379) (by name) L199,212,283; `Error::new` (result.rs:360) L199,212,283; `identity_correspondence` (expect.rs:536) (by name) L209; `evaluate_with_receipt` (expect.rs:590) (by name) L222; `content_sha256` (expect.rs:520) L229; `ResultData::new` (result.rs:716) L249; `scope_line` (bin/jubarte/check.rs:74) L254,294; `first_of` (bin/jubarte/check.rs:118) L256,265; `Envelope::with_failure_data` (result.rs:820) (by name) L286; `Envelope::refused` (result.rs:895) L286; `Envelope::ok` (result.rs:866) L293; `Runtime::native` (result.rs:500) L293
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.count`, `.display`, `.expect`, `.filter`, `.iter`, `.map`, `.to_string`, `.unwrap_or_default`, `Default::default`, `Ok`, `Some`, `parse_or_usage`, `std::fs::read`, `std::str::from_utf8`
- **Macros:** `format!`
- **Called by:** `cli_main` (bin/jubarte.rs:3935)
