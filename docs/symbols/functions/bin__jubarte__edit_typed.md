# Functions and methods in `src/bin/jubarte/edit_typed.rs`

18 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `failure`  (L17-23)
`fn failure(code: &str, message: impl Into<String>) -> Failure`

- **Does:** _no doc comment_
- **Calls (crate):** `Error::with_commit_state` (result.rs:347) (by name) L19; `Error::with_message` (result.rs:379) (by name) L19; `Error::new` (result.rs:360) L19
- **Calls (std/external/unresolved):** `Box::new`
- **Called by:** `edit_error` (bin/jubarte/edit_typed.rs:24); `io_error` (bin/jubarte/edit_typed.rs:30); `alias` (bin/jubarte/edit_typed.rs:59); `writable` (bin/jubarte/edit_typed.rs:62); `evaluate_expectations` (bin/jubarte/edit_typed.rs:200); `check_hash` (bin/jubarte/edit_typed.rs:266); `request` (bin/jubarte/edit_typed.rs:294); `prepare_and_publish` (bin/jubarte/edit_typed.rs:398); `usage` (bin/jubarte/edit_typed.rs:962); `compare_history` (edit/v2_apply.rs:2422) (?); `transaction` (publication.rs:83) (?)

#### `edit_error`  (L24-29)
`fn edit_error(e: jubarte::edit::EditError) -> Failure`

- **Does:** _no doc comment_
- **Calls (crate):** `failure` (bin/jubarte/edit_typed.rs:17) L25
- **Calls (std/external/unresolved):** `.map`
- **Called by:** `prepare_and_publish` (bin/jubarte/edit_typed.rs:398)

#### `io_error`  (L30-32)
`fn io_error(message: String) -> Failure`

- **Does:** _no doc comment_
- **Calls (crate):** `failure` (bin/jubarte/edit_typed.rs:17) L31
- **Called by:** `read` (bin/jubarte/edit_typed.rs:56); `prepare_and_publish` (bin/jubarte/edit_typed.rs:398)

#### `publish_failure`  (L39-55, pub(super))
`pub(super) fn publish_failure((_, message): (u8, String)) -> Failure`

- **Does:** A publication refusal or failure in the vocabulary of the typed and template routes.
- **Calls (crate):** `Error::with_commit_state` (result.rs:347) (by name) L51; `Error::with_message` (result.rs:379) (by name) L51; `Error::new` (result.rs:360) L51
- **Calls (std/external/unresolved):** `.contains`, `.map`, `.split_once`, `Box::new`
- **Called by:** none found in src

#### `read`  (L56-58)
`fn read(path: &Path) -> Result<Vec<u8>, Failure>`

- **Does:** _no doc comment_
- **Calls (crate):** `io_error` (bin/jubarte/edit_typed.rs:30) L57
- **Calls (std/external/unresolved):** `.display`, `.map_err`, `std::fs::read`
- **Macros:** `format!`
- **Called by:** `evaluate_expectations` (bin/jubarte/edit_typed.rs:200); `prepare_and_publish` (bin/jubarte/edit_typed.rs:398); `run` (bin/jubarte/edit_typed.rs:787)

#### `alias`  (L59-61)
`fn alias(path: &Path, inputs: &[(&Path, &str)]) -> Result<(), Failure>`

- **Does:** _no doc comment_
- **Calls (crate):** `refuse_output_alias` (bin/jubarte.rs:1701) (by name) L60; `failure` (bin/jubarte/edit_typed.rs:17) L60
- **Calls (std/external/unresolved):** `.map_err`
- **Called by:** `writable` (bin/jubarte/edit_typed.rs:62); `prepare_and_publish` (bin/jubarte/edit_typed.rs:398)

#### `writable`  (L62-70)
`fn writable(path: &Path, inputs: &[(&Path, &str)], force: bool) -> Result<(), Failure>`

- **Does:** _no doc comment_
- **Calls (crate):** `alias` (bin/jubarte/edit_typed.rs:59) L63; `ensure_writable` (bin/jubarte.rs:237) (by name) L64; `failure` (bin/jubarte/edit_typed.rs:17) L65
- **Calls (std/external/unresolved):** `.map_err`, `.trim_start_matches`
- **Called by:** `prepare_and_publish` (bin/jubarte/edit_typed.rs:398)

#### `Request::prepare`  (L78-102)
`fn prepare( &self, source: &[u8], base: &PrepareOptions, ) -> Result<PreparedEdit, jubarte::edit::EditError>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone_from`, `Ok`, `jubarte::edit::prepare_v2`, `jubarte::edit::prepare`
- **Called by:** `prepare_and_publish` (bin/jubarte/edit_typed.rs:398) (?)

#### `Request::count`  (L103-108)
`fn count(&self) -> usize`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.len`
- **Called by:** none found in src

#### `regeneration_action`  (L111-188)
`fn regeneration_action( error: &jubarte::edit::EditError, plan: &PlanV2, source: &Path, plan_path: &Path, ) -> Option<NextAction>`

- **Does:** _no doc comment_
- **Calls (crate):** `absolute` (bin/jubarte.rs:168) (by name) L143,176; `NextAction::with_purpose` (result.rs:288) (by name) L179; `NextAction::with_prerequisites` (result.rs:295) (by name) L179; `NextAction::new` (result.rs:278) L179
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.collect`, `.extend`, `.filter_map`, `.filter`, `.get`, `.insert`, `.into`, `.is_empty`, `.is_ok`, `.iter`, `.join`, `.map`, `.ok`, `.parse`, `.strip_prefix`, `.symlink_metadata`, `.with_extension`, `Some`, `std::collections::BTreeSet::new`
- **Macros:** `format!`, `vec!`
- **Called by:** `prepare_and_publish` (bin/jubarte/edit_typed.rs:398)

#### `evaluate_expectations`  (L200-264)
`fn evaluate_expectations( path: &Path, prepared: &PreparedEdit, source: &[u8], ) -> Result<ExpectationReport, Failure>`

- **Does:** Read, bind and evaluate the optional expectation contract on the prepared transaction's chosen primary bytes (V2-13, proposal §10).
- **Calls (crate):** `read` (bin/jubarte/edit_typed.rs:56) L205; `failure` (bin/jubarte/edit_typed.rs:17) L207,213,215,221,238; `parse_expectations` (expect.rs:410) (by name) L212; `evaluate_with_receipt` (expect.rs:590) (by name) L226; `content_sha256` (expect.rs:520) L233; `ExpectationReport::blocking_failure` (expect.rs:277) (by name) L237; `rebind_to_output` (expect.rs:1008) L262
- **Calls (std/external/unresolved):** `.as_ref`, `.display`, `.map_err`, `.map`, `.to_string`, `.unwrap_or_default`, `Err`, `Ok`, `Some`, `std::str::from_utf8`
- **Macros:** `format!`
- **Called by:** `prepare_and_publish` (bin/jubarte/edit_typed.rs:398)

#### `check_hash`  (L266-292)
`fn check_hash( recorded: Option<&str>, expected: Option<&str>, source: &[u8], waiver: bool, ) -> Result<&'static str, Failure>`

- **Does:** _no doc comment_
- **Calls (crate):** `source_sha256` (inspect.rs:299) L272; `failure` (bin/jubarte/edit_typed.rs:17) L277,287
- **Calls (std/external/unresolved):** `.is_some_and`, `.is_some`, `.zip`, `Err`, `Ok`
- **Called by:** `request` (bin/jubarte/edit_typed.rs:294)

#### `request`  (L294-376)
`fn request( action: &EditAction, source: &[u8], payload: Option<&str>, plan_bytes: Option<&[u8]>, ) -> Result<(Request, &'static str), Failure>`

- **Does:** _no doc comment_
- **Calls (crate):** `EditAction::delivery` (cli/edit_actions.rs:367) L300; `failure` (bin/jubarte/edit_typed.rs:17) L306,312,318,323,335,361; `EditPlan::from_json` (edit.rs:1790) L310,317; `PlanV2::from_json` (edit/v2.rs:514) L333; `check_hash` (bin/jubarte/edit_typed.rs:266) L340,350,368; `EditPlan::from_json_with_notes` (edit.rs:1805) L349; `compile` (edit/inline.rs:38) L367
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.clone_from`, `.err`, `.expect`, `.get`, `.is_none`, `.is_object`, `.map_err`, `.map_or_else`, `.strip_prefix`, `.to_string`, `.unwrap_or`, `Err`, `Ok`, `Request::V1`, `Request::V2`, `Some`, `serde_json::from_str`, `std::str::from_utf8`
- **Called by:** `prepare_and_publish` (bin/jubarte/edit_typed.rs:398)

#### `artifact`  (L378-387)
`fn artifact(path: &Path, role: MediaRole, bytes: &[u8]) -> Artifact`

- **Does:** _no doc comment_
- **Calls (crate):** `Artifact::committed` (result.rs:203) L386; `absolute` (bin/jubarte.rs:168) (by name) L386
- **Called by:** `prepare_and_publish` (bin/jubarte/edit_typed.rs:398)

#### `success`  (L388-396)
`fn success(command: &str, source: &[u8], phase: Phase, effect: Effect) -> Envelope`

- **Does:** _no doc comment_
- **Calls (crate):** `Envelope::ok` (result.rs:866) L389; `Source::new` (result.rs:463) L391; `ResultData::new` (result.rs:716) L392; `Runtime::native` (result.rs:500) L393
- **Calls (std/external/unresolved):** `.expect`
- **Called by:** `prepare_and_publish` (bin/jubarte/edit_typed.rs:398)

#### `prepare_and_publish`  (L398-785)
`fn prepare_and_publish( action: &EditAction, source: &[u8], command: &str, ) -> Result<(Envelope, bool), Failure>`

- **Does:** _no doc comment_
- **Calls (crate):** `EditAction::delivery` (cli/edit_actions.rs:367) L403; `EditAction::payload` (cli/edit_actions.rs:351) L404,414,424,717; `read` (bin/jubarte/edit_typed.rs:56) L407,709,713,718; `failure` (bin/jubarte/edit_typed.rs:17) L413,439,449,460,696,710...; `EditAction::file` (cli/edit_actions.rs:360) (by name) L420; `EditAction::inline` (cli/edit_actions.rs:341) L430,467; `alias` (bin/jubarte/edit_typed.rs:59) L436; `same_path` (bin/jubarte.rs:197) (by name) L438,695,728; `request` (bin/jubarte/edit_typed.rs:294) L455; `.prepare` (ambiguous: `Request::prepare` (bin/jubarte/edit_typed.rs:78); `StagedFile::prepare` (publication.rs:299)) L483; `<PrepareOptions as Default>::default` (edit/prepared.rs:158) L491; `regeneration_action` (bin/jubarte/edit_typed.rs:111) L497; `EditAction::file` (cli/edit_actions.rs:360) L497,503,537,709,766,779; `EditError::with_revision_syntax` (edit/refusal.rs:147) (by name) L501; `EditError::revision_next_actions` (edit/refusal.rs:159) (by name) L502; `absolute` (bin/jubarte.rs:168) (by name) L503,537; `source_sha256` (inspect.rs:299) L504; `edit_error` (bin/jubarte/edit_typed.rs:24) L506; `Error::push_next_action` (result.rs:433) (by name) L508,511; `evaluate_expectations` (bin/jubarte/edit_typed.rs:200) L521; `success` (bin/jubarte/edit_typed.rs:388) L536; `CandidateChecks::verification` (edit/prepared.rs:94) (by name) L606; `ResultData::set_predicted` (result.rs:732) (by name) L641; `diff_documents` (text_diff.rs:745) L659,678; `<UnifiedOptions as Default>::default` (text_diff.rs:32) L662,681; `artifact` (bin/jubarte/edit_typed.rs:378) L692,705; `writable` (bin/jubarte/edit_typed.rs:62) L726; `stage_bundle_dir` (bin/jubarte.rs:1977) (by name) L744; `publish_paths` (ambiguous: `publish_paths` (bin/jubarte.rs:262); `publish_paths` (publication.rs:211)) L757; `io_error` (bin/jubarte/edit_typed.rs:30) L759; `publish_dir` (bin/jubarte.rs:2019) (by name) L761; `refuse_output_alias` (bin/jubarte.rs:1701) (by name) L763,778; `refuse_bundle_input_containment` (bin/jubarte.rs:1758) (by name) L765; `recheck_edit_source` (bin/jubarte.rs:1730) (by name) L766,779; `publish_files` (bin/jubarte.rs:1854) (by name) L774
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_bytes`, `.as_deref`, `.as_mut`, `.as_path`, `.as_ref`, `.as_slice`, `.clone`, `.collect`, `.contains`, `.count`, `.enumerate`, `.expect`, `.file_name`, `.filter`, `.flatten`, `.insert`, `.into_bytes`, `.into_iter`, `.into`, `.is_empty`, `.is_ok`, `.is_some`, `.iter`, `.join`, `.len`, `.map_err`, `.map`, `.or_else`, `.push`, `.saturating_sub`, `.symlink_metadata`, `.to_string`, `.transpose`, `.with_extension`, `Err`, `Ok`, `Request::V1`, `Request::V2`, `Some`, `Vec::<(PathBuf, Vec, MediaRole)>::new`, `jubarte::markdown::Source::Docx`, `serde_json::to_value`, `serde_json::to_vec_pretty`, `std::fs::remove_dir_all`
- **Macros:** `format!`, `matches!`, `serde_json::json!`, `unreachable!`, `vec!`
- **Called by:** `run` (bin/jubarte/edit_typed.rs:787)

#### `run`  (L787-959, pub(super))
`pub(super) fn run(typed: &TypedEdit, quiet: bool) -> ExitCode`

- **Does:** _no doc comment_
- **Calls (crate):** `read` (bin/jubarte/edit_typed.rs:56) L790; `EditAction::file` (cli/edit_actions.rs:360) (by name) L790,822,943; `warn_nested_text_boxes` (bin/jubarte.rs:1121) L792; `prepare_and_publish` (bin/jubarte/edit_typed.rs:398) L795; `Envelope::new_error` (result.rs:916) L801,820; `Source::new` (result.rs:463) L801,803; `Envelope::refused` (result.rs:895) L803; `.with_path` (ambiguous: `AnchorRefusal::with_path` (edit/anchor_error.rs:100); `Source::with_path` (result.rs:482)) L822; `Source::from_digest` (result.rs:472) L822; `absolute` (bin/jubarte.rs:168) (by name) L822; `edit_envelope_exit` (bin/jubarte.rs:1659) (by name) L828; `EditAction::delivery` (cli/edit_actions.rs:367) (by name) L829; `NearMatches::copy_hint` (edit/anchor.rs:139) (by name) L834; `write_jsonl_stream` (result.rs:1019) L839; `write_stdout` (bin/jubarte.rs:1240) (by name) L898; `settlement_notes` (bin/jubarte.rs:2120) L934; `edit_status` (bin/jubarte.rs:1781) (by name) L940,952; `print_edit_success` (bin/jubarte.rs:1208) (by name) L943
- **Calls (std/external/unresolved):** `.as_mut`, `.as_ref`, `.as_str`, `.clone`, `.collect`, `.display`, `.expect`, `.is_empty`, `.is_some_and`, `.iter`, `.join`, `.map`, `.name`, `.or_else`, `.push_str`, `.to_string`, `.unwrap_or_default`, `.unwrap_or_else`, `ExitCode::from`, `Some`, `String::new`, `serde_json::to_string`, `std::fs::read`
- **Macros:** `eprintln!`, `format!`, `format_args!`, `matches!`
- **Called by:** `cli_main` (bin/jubarte.rs:3935)

#### `usage`  (L962-1000, pub(super))
`pub(super) fn usage(argv: &[String], error: &clap::Error) -> bool`

- **Does:** Emit only recognized typed-route usage failures in the requested protocol.
- **Calls (crate):** `typed_edit_protocol` (cli.rs:3370) L973; `.with_path` (ambiguous: `AnchorRefusal::with_path` (edit/anchor_error.rs:100); `Source::with_path` (result.rs:482)) L980; `Source::from_digest` (result.rs:472) L980,981; `absolute` (bin/jubarte.rs:168) (by name) L980; `Envelope::new_error` (result.rs:916) L983; `failure` (bin/jubarte/edit_typed.rs:17) L986; `write_jsonl_stream` (result.rs:1019) L989; `write_stdout` (bin/jubarte.rs:1240) (by name) L996
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.expect`, `.iter`, `.kind`, `.map`, `.replace`, `.to_string`, `Path::new`, `serde_json::to_string`, `std::iter::once`
- **Macros:** `eprintln!`, `format!`, `matches!`
- **Called by:** `run` (bin/jubarte.rs:2862) (?); `cli_main` (bin/jubarte.rs:3935); `record_json` (edit/record.rs:270) (?)
