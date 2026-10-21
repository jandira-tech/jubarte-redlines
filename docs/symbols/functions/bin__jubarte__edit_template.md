# Functions and methods in `src/bin/jubarte/edit_template.rs`

7 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `failure`  (L19-25)
`fn failure(code: &str, message: impl Into<String>) -> Failure`

- **Does:** _no doc comment_
- **Calls (crate):** `Error::with_commit_state` (result.rs:347) (by name) L21; `Error::with_message` (result.rs:379) (by name) L21; `Error::new` (result.rs:360) L21
- **Calls (std/external/unresolved):** `Box::new`
- **Called by:** `read` (bin/jubarte/edit_template.rs:26); `alias` (bin/jubarte/edit_template.rs:29); `generate` (bin/jubarte/edit_template.rs:32); `usage` (bin/jubarte/edit_template.rs:292); `compare_history` (edit/v2_apply.rs:2422) (?); `transaction` (publication.rs:83) (?)

#### `read`  (L26-28)
`fn read(path: &Path) -> Result<Vec<u8>, Failure>`

- **Does:** _no doc comment_
- **Calls (crate):** `failure` (bin/jubarte/edit_template.rs:19) L27
- **Calls (std/external/unresolved):** `.display`, `.map_err`, `std::fs::read`
- **Macros:** `format!`
- **Called by:** `generate` (bin/jubarte/edit_template.rs:32); `run` (bin/jubarte/edit_template.rs:199)

#### `alias`  (L29-31)
`fn alias(path: &Path, input: &Path) -> Result<(), Failure>`

- **Does:** _no doc comment_
- **Calls (crate):** `refuse_output_alias` (bin/jubarte.rs:1701) (by name) L30; `failure` (bin/jubarte/edit_template.rs:19) L30
- **Calls (std/external/unresolved):** `.map_err`
- **Called by:** `generate` (bin/jubarte/edit_template.rs:32)

#### `generate`  (L32-197)
`fn generate( options: &TemplateOptions, source: &[u8], ) -> Result<(Envelope, Option<Vec<u8>>), Failure>`

- **Does:** _no doc comment_
- **Calls (crate):** `source_sha256` (inspect.rs:299) L39,108; `failure` (bin/jubarte/edit_template.rs:19) L41,54,92,140,145,157; `alias` (bin/jubarte/edit_template.rs:29) L52,143; `same_path` (bin/jubarte.rs:197) (by name) L53,155; `generate` (edit/template.rs:84) L60; `ResultData::new` (result.rs:716) L93; `Envelope::ok` (result.rs:866) L96; `Source::new` (result.rs:463) L98; `Runtime::native` (result.rs:500) L100; `absolute` (bin/jubarte.rs:168) (by name) L103,163,173; `read` (bin/jubarte/edit_template.rs:26) L139; `ensure_writable` (bin/jubarte.rs:237) (by name) L144; `Artifact::committed` (result.rs:203) L162,172; `publish_files` (bin/jubarte.rs:1854) (by name) L187; `refuse_output_alias` (bin/jubarte.rs:1701) (by name) L191; `recheck_edit_source` (bin/jubarte.rs:1730) (by name) L192
- **Calls (std/external/unresolved):** `.any`, `.as_bytes`, `.as_deref`, `.as_path`, `.as_ref`, `.as_slice`, `.as_str`, `.clone`, `.collect`, `.enumerate`, `.expect`, `.filter`, `.flatten`, `.insert`, `.into_iter`, `.into`, `.is_none`, `.is_some_and`, `.iter`, `.len`, `.map_err`, `.map`, `.push`, `.then_some`, `.to_path_buf`, `.trim_start_matches`, `.unwrap_or_else`, `.zip`, `Default::default`, `Err`, `Ok`, `Path::new`, `Some`, `Vec::new`, `serde_json::Map::new`, `serde_json::Value::Object`, `serde_json::Value::from`, `serde_json::to_string`, `serde_json::to_value`, `serde_json::to_vec_pretty`
- **Called by:** `run` (bin/jubarte/edit_template.rs:199)

#### `run`  (L199-283, pub(super))
`pub(super) fn run(options: &TemplateOptions, quiet: bool) -> ExitCode`

- **Does:** _no doc comment_
- **Calls (crate):** `read` (bin/jubarte/edit_template.rs:26) L200; `warn_nested_text_boxes` (bin/jubarte.rs:1121) L202; `generate` (bin/jubarte/edit_template.rs:32) L207; `path_only` (bin/jubarte/edit_template.rs:287) L214; `.with_path` (ambiguous: `AnchorRefusal::with_path` (edit/anchor_error.rs:100); `Source::with_path` (result.rs:482)) L215; `Source::new` (result.rs:463) L215; `absolute` (bin/jubarte.rs:168) (by name) L215; `Envelope::new_error` (result.rs:916) L221; `Envelope::refused` (result.rs:895) L223; `edit_envelope_exit` (bin/jubarte.rs:1659) (by name) L228; `write_jsonl_stream` (result.rs:1019) L232; `write_stdout` (bin/jubarte.rs:1240) (by name) L269
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.as_str`, `.collect`, `.expect`, `.filter`, `.is_empty`, `.iter`, `.join`, `.map_err`, `.map_or_else`, `.map`, `ExitCode::from`, `String::from_utf8`, `String::new`, `serde_json::to_string`
- **Macros:** `eprintln!`, `format!`, `matches!`
- **Called by:** `cli_main` (bin/jubarte.rs:3935)

#### `path_only`  (L287-289)
`fn path_only(file: &Path) -> Source`

- **Does:** The source file's `Source` for failures that never read its bytes: the user-named path, no invented digest (#1084 review F5).
- **Calls (crate):** `.with_path` (ambiguous: `AnchorRefusal::with_path` (edit/anchor_error.rs:100); `Source::with_path` (result.rs:482)) L288; `Source::from_digest` (result.rs:472) L288; `absolute` (bin/jubarte.rs:168) (by name) L288
- **Called by:** `run` (bin/jubarte/edit_template.rs:199)

#### `usage`  (L292-355, pub(super))
`pub(super) fn usage(argv: &[String], error: &clap::Error) -> bool`

- **Does:** Recognized machine mode applies even to invalid generator syntax.
- **Calls (crate):** `top_level_command` (cli.rs:856) L303; `route` (cli/edit_actions.rs:260) L304; `first_positional` (cli.rs:3484) L327; `.with_path` (ambiguous: `AnchorRefusal::with_path` (edit/anchor_error.rs:100); `Source::with_path` (result.rs:482)) L330; `Source::from_digest` (result.rs:472) L330,331; `absolute` (bin/jubarte.rs:168) (by name) L330; `Envelope::new_error` (result.rs:916) L333; `failure` (bin/jubarte/edit_template.rs:19) L336; `write_jsonl_stream` (result.rs:1019) L344; `write_stdout` (bin/jubarte.rs:1240) (by name) L351
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.chain`, `.collect`, `.copied`, `.expect`, `.filter_map`, `.find_subcommand`, `.first`, `.get`, `.iter`, `.kind`, `.map`, `.position`, `.replace`, `.skip`, `.take_while`, `.to_str`, `.to_string`, `.unwrap_or_default`, `.zip`, `Path::new`, `serde_json::to_string`, `std::iter::once`
- **Macros:** `eprintln!`, `format!`, `matches!`
- **Called by:** `run` (bin/jubarte.rs:2862) (?); `cli_main` (bin/jubarte.rs:3935); `record_json` (edit/record.rs:270) (?)
