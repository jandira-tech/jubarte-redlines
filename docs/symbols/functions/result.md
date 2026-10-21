# Functions and methods in `src/result.rs`

53 non-test, 13 test. Part of the [symbol manifest](../README.md).

#### `Artifact::new`  (L186-200, pub)
`pub fn new( path: impl Into<String>, role: MediaRole, media_type: impl Into<String>, payload: &[u8], ) -> Self`

- **Does:** Build an artifact from `payload`; the digest is computed and `committed` defaults to `false`.
- **Calls (crate):** `source_sha256` (inspect.rs:299) L196
- **Calls (std/external/unresolved):** `.into`, `.len`
- **Called by:** `Artifact::committed` (result.rs:203); `tests::artifact_sha256_matches_source_sha256_helper` (result.rs:1354)

#### `Artifact::committed`  (L203-212, pub)
`pub fn committed( path: impl Into<String>, role: MediaRole, media_type: impl Into<String>, payload: &[u8], ) -> Self`

- **Does:** Build an artifact and mark it committed.
- **Calls (crate):** `Artifact::new` (result.rs:186) L209
- **Called by:** `generate` (bin/jubarte/edit_template.rs:32); `artifact` (bin/jubarte/edit_typed.rs:378)

#### `Artifact::with_committed`  (L216-219, pub)
`pub fn with_committed(mut self, committed: bool) -> Self`

- **Does:** Override `committed`.
- **Calls:** none
- **Called by:** none found in src

#### `Context::new`  (L240-251, pub)
`pub fn new( before: impl Into<String>, matched: impl Into<String>, after: impl Into<String>, ) -> Self`

- **Does:** Build a context with `truncated: false`; the caller sets the flag if the window was bounded.
- **Calls (std/external/unresolved):** `.into`
- **Called by:** `refused` (edit/anchor_error.rs:462); `around` (edit/anchor_error.rs:607)

#### `Context::with_truncated`  (L255-258, pub)
`pub fn with_truncated(mut self, truncated: bool) -> Self`

- **Does:** Override the truncation flag.
- **Calls:** none
- **Called by:** `refused` (edit/anchor_error.rs:462) (?); `around` (edit/anchor_error.rs:607) (?)

#### `NextAction::new`  (L278-284, pub)
`pub fn new(argv: Vec<String>) -> Self`

- **Does:** Build a next action from a structured argv array.
- **Calls:** none
- **Called by:** `regeneration_action` (bin/jubarte/edit_typed.rs:111); `read_action` (edit/anchor_error.rs:630); `EditError::revision_next_actions` (edit/refusal.rs:159)

#### `NextAction::with_purpose`  (L288-291, pub)
`pub fn with_purpose(mut self, purpose: impl Into<String>) -> Self`

- **Does:** Set the purpose string.
- **Calls (std/external/unresolved):** `.into`, `Some`
- **Called by:** `regeneration_action` (bin/jubarte/edit_typed.rs:111) (?); `read_target` (edit/anchor_error.rs:653) (?); `read_head` (edit/anchor_error.rs:659) (?); `suggest_read` (edit/anchor_error.rs:669) (?); `EditError::revision_next_actions` (edit/refusal.rs:159) (?)

#### `NextAction::with_prerequisites`  (L295-298, pub)
`pub fn with_prerequisites(mut self, prerequisites: Vec<String>) -> Self`

- **Does:** Set the prerequisites list.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `regeneration_action` (bin/jubarte/edit_typed.rs:111) (?); `read_action` (edit/anchor_error.rs:630) (?); `suggest_read` (edit/anchor_error.rs:669) (?)

#### `Error::with_commit_state`  (L347-351, pub)
`pub fn with_commit_state(mut self, state: CommitState) -> Self`

- **Does:** Set the known publication state.
- **Calls (crate):** `Error::normalize_retryable` (result.rs:353) L349
- **Called by:** `failure` (bin/jubarte/edit_template.rs:19) (?); `failure` (bin/jubarte/edit_typed.rs:17) (?); `publish_failure` (bin/jubarte/edit_typed.rs:39) (?); `base` (edit/anchor_error.rs:446) (?); `Envelope::refused` (result.rs:895)

#### `Error::normalize_retryable`  (L353-357)
`fn normalize_retryable(&mut self)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `Error::with_commit_state` (result.rs:347); `Error::with_retryable` (result.rs:439); `Envelope::with_failure_data` (result.rs:820) (?); `Envelope::new_error` (result.rs:916)

#### `Error::new`  (L360-375, pub)
`pub fn new(code: impl Into<String>) -> Self`

- **Does:** Build an error with unknown publication state and no retry hint.
- **Calls (std/external/unresolved):** `.into`, `String::new`, `Vec::new`
- **Called by:** `usage` (bin/jubarte/check.rs:95); `io_error` (bin/jubarte/check.rs:107); `run` (bin/jubarte/check.rs:125); `failure` (bin/jubarte/edit_template.rs:19); `failure` (bin/jubarte/edit_typed.rs:17); `publish_failure` (bin/jubarte/edit_typed.rs:39); `base` (edit/anchor_error.rs:446); `refusal` (edit/inline.rs:16); `edit_error` (edit/inline.rs:19); `transaction` (publication.rs:83); `validate_relative` (publication.rs:228); `StagedFile::cleanup` (publication.rs:355); +8 more

#### `Error::with_message`  (L379-382, pub)
`pub fn with_message(mut self, message: impl Into<String>) -> Self`

- **Does:** Set the message.
- **Calls (std/external/unresolved):** `.into`
- **Called by:** `usage` (bin/jubarte/check.rs:95) (?); `io_error` (bin/jubarte/check.rs:107) (?); `run` (bin/jubarte/check.rs:125) (?); `failure` (bin/jubarte/edit_template.rs:19) (?); `failure` (bin/jubarte/edit_typed.rs:17) (?); `publish_failure` (bin/jubarte/edit_typed.rs:39) (?); `open` (edit/anchor_error.rs:229) (?); `missing_target` (edit/anchor_error.rs:364) (?); `several_changes` (edit/anchor_error.rs:422) (?); `refused` (edit/anchor_error.rs:462) (?); `refusal` (edit/inline.rs:16) (?); `edit_error` (edit/inline.rs:19) (?)

#### `Error::with_operation`  (L386-389, pub)
`pub fn with_operation(mut self, operation: impl Into<String>) -> Self`

- **Does:** Set the operation pointer/id.
- **Calls (std/external/unresolved):** `.into`, `Some`
- **Called by:** `require_story` (edit/anchor_error.rs:210) (?); `base` (edit/anchor_error.rs:446) (?); `compile` (edit/inline.rs:38) (?)

#### `Error::with_source`  (L393-396, pub)
`pub fn with_source(mut self, source: Source) -> Self`

- **Does:** Bind the source digest.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** none found in src

#### `Error::with_source_digest`  (L400-403, pub)
`pub fn with_source_digest(mut self, digest: impl Into<String>) -> Self`

- **Does:** Override the source digest directly.
- **Calls (std/external/unresolved):** `.into`, `Some`
- **Called by:** `base` (edit/anchor_error.rs:446) (?)

#### `Error::with_target`  (L407-410, pub)
`pub fn with_target(mut self, target: impl Into<String>) -> Self`

- **Does:** Set the resolved target pointer.
- **Calls (std/external/unresolved):** `.into`, `Some`
- **Called by:** `base` (edit/anchor_error.rs:446) (?)

#### `Error::with_matches`  (L414-417, pub)
`pub fn with_matches(mut self, matches: usize) -> Self`

- **Does:** Set the matches count.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `missing_target` (edit/anchor_error.rs:364) (?); `refused` (edit/anchor_error.rs:462) (?)

#### `Error::with_context`  (L421-424, pub)
`pub fn with_context(mut self, context: Context) -> Self`

- **Does:** Set the bounded context.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `refused` (edit/anchor_error.rs:462) (?)

#### `Error::set_context`  (L428-430, pub)
`pub fn set_context(&mut self, context: Context)`

- **Does:** Set the bounded context (in-place variant for chaining after mutating other fields).
- **Calls (std/external/unresolved):** `Some`
- **Called by:** none found in src

#### `Error::push_next_action`  (L433-435, pub)
`pub fn push_next_action(&mut self, action: NextAction)`

- **Does:** Push a structured argv next action.
- **Calls (std/external/unresolved):** `.push`
- **Called by:** `prepare_and_publish` (bin/jubarte/edit_typed.rs:398) (?); `missing_target` (edit/anchor_error.rs:364) (?); `several_changes` (edit/anchor_error.rs:422) (?); `refused` (edit/anchor_error.rs:462) (?)

#### `Error::with_retryable`  (L439-443, pub)
`pub fn with_retryable(mut self, retryable: bool) -> Self`

- **Does:** Set the retryable hint.
- **Calls (crate):** `Error::normalize_retryable` (result.rs:353) L441
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `base` (edit/anchor_error.rs:446) (?)

#### `Source::new`  (L463-469, pub)
`pub fn new(bytes: &[u8]) -> Self`

- **Does:** Build a `Source` from raw bytes: the digest is computed and the byte size recorded; no path is set.
- **Calls (crate):** `source_sha256` (inspect.rs:299) L465
- **Calls (std/external/unresolved):** `.len`, `Some`
- **Called by:** `run` (bin/jubarte/check.rs:125); `generate` (bin/jubarte/edit_template.rs:32); `run` (bin/jubarte/edit_template.rs:199); `success` (bin/jubarte/edit_typed.rs:388); `run` (bin/jubarte/edit_typed.rs:787); `tests::status_serializes_as_lowercase_string` (result.rs:1343); `tests::jsonl_round_trip_preserves_terminal_record` (result.rs:1361)

#### `Source::from_digest`  (L472-478, pub)
`pub fn from_digest(digest: impl Into<String>) -> Self`

- **Does:** Build a `Source` from a precomputed digest.
- **Calls (std/external/unresolved):** `.into`
- **Called by:** `path_only` (bin/jubarte/check.rs:113); `path_only` (bin/jubarte/edit_template.rs:287); `usage` (bin/jubarte/edit_template.rs:292); `run` (bin/jubarte/edit_typed.rs:787); `usage` (bin/jubarte/edit_typed.rs:962)

#### `Source::with_path`  (L482-485, pub)
`pub fn with_path(mut self, path: impl Into<String>) -> Self`

- **Does:** Set the path.
- **Calls (std/external/unresolved):** `.into`, `Some`
- **Called by:** `path_only` (bin/jubarte/check.rs:113) (?); `run` (bin/jubarte/check.rs:125) (?); `run` (bin/jubarte/edit_template.rs:199) (?); `path_only` (bin/jubarte/edit_template.rs:287) (?); `usage` (bin/jubarte/edit_template.rs:292) (?); `run` (bin/jubarte/edit_typed.rs:787) (?); `usage` (bin/jubarte/edit_typed.rs:962) (?)

#### `Runtime::native`  (L500-505, pub)
`pub fn native() -> Self`

- **Does:** The native CLI runtime.
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `run` (bin/jubarte/check.rs:125); `generate` (bin/jubarte/edit_template.rs:32); `success` (bin/jubarte/edit_typed.rs:388); `Envelope::refused` (result.rs:895); `Envelope::new_error` (result.rs:916); `tests::jsonl_round_trip_preserves_terminal_record` (result.rs:1361)

#### `<Verification as Default>::default`  (L532-542, impl Default)
`fn default() -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `ResultData::failure` (result.rs:697); `ResultData::new` (result.rs:716)

#### `Counts::total_operations`  (L601-603, pub)
`pub fn total_operations(&self) -> usize`

- **Does:** Sum of `body_operations_applied` + `review_operations_applied`.
- **Calls:** none
- **Called by:** none found in src

#### `is_zero`  (L606-608)
`fn is_zero(value: &usize) -> bool`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `<ResultDataError as std::fmt::Display>::fmt`  (L673-690, impl std::fmt::Display)
`fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`

- **Does:** _no doc comment_
- **Macros:** `write!`
- **Called by:** none found in src

#### `ResultData::failure`  (L697-709, pub)
`pub fn failure(phase: Option<Phase>) -> Self`

- **Does:** Build pre-commit failure data with the last completed phase.
- **Calls (crate):** `<Verification as Default>::default` (result.rs:532) L705
- **Calls (std/external/unresolved):** `Counts::default`, `Vec::new`
- **Called by:** `Envelope::refused` (result.rs:895); `Envelope::new_error` (result.rs:916)

#### `ResultData::new`  (L716-728, pub)
`pub fn new(phase: Phase, effect: Effect) -> Self`

- **Does:** Build a result data block.
- **Calls (crate):** `<Verification as Default>::default` (result.rs:532) L724
- **Calls (std/external/unresolved):** `Counts::default`, `Some`, `Vec::new`
- **Called by:** `run` (bin/jubarte/check.rs:125); `generate` (bin/jubarte/edit_template.rs:32); `success` (bin/jubarte/edit_typed.rs:388); `tests::not_applied_required_for_generated_phase` (result.rs:1250); `tests::not_applied_required_for_validated_phase` (result.rs:1261); `tests::committed_phase_may_carry_changed_or_unchanged` (result.rs:1269); `tests::committed_phase_with_not_applied_is_rejected` (result.rs:1277); `tests::predicted_effect_requires_validated_phase` (result.rs:1291); `tests::jsonl_round_trip_preserves_terminal_record` (result.rs:1361)

#### `ResultData::set_predicted`  (L732-734, pub)
`pub fn set_predicted(&mut self, predicted: PredictedEffect)`

- **Does:** Set the predicted effect (dry runs only).
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `prepare_and_publish` (bin/jubarte/edit_typed.rs:398) (?); `tests::predicted_effect_requires_validated_phase` (result.rs:1291); `tests::jsonl_round_trip_preserves_terminal_record` (result.rs:1361)

#### `ResultData::validate`  (L738-766, pub)
`pub fn validate(&self) -> Result<(), ResultDataError>`

- **Does:** Validate the (phase, effect, predicted_effect) combination.
- **Calls (std/external/unresolved):** `.is_some`, `Err`, `Ok`
- **Called by:** `Envelope::with_failure_data` (result.rs:820); `Envelope::ok` (result.rs:866)

#### `Envelope::with_failure_data`  (L820-837, pub)
`pub fn with_failure_data(mut self, data: ResultData) -> Result<Self, ResultDataError>`

- **Does:** Attach validated phase/effect/counts to a failure envelope.
- **Calls (crate):** `ResultData::validate` (result.rs:738) L824; `Error::normalize_retryable` (result.rs:353) (by name) L833
- **Calls (std/external/unresolved):** `Err`, `Ok`, `Some`
- **Called by:** `run` (bin/jubarte/check.rs:125) (?)

#### `Envelope::new`  (L843-858, pub)
`pub fn new(command: impl Into<String>, source: Source, runtime: Runtime) -> Self`

- **Does:** Build a placeholder envelope with `status: ok` and no data; used by tests and scaffold code.
- **Calls (std/external/unresolved):** `.into`, `.to_string`, `BTreeMap::new`, `Vec::new`
- **Macros:** `env!`
- **Called by:** none found in src

#### `Envelope::ok`  (L866-890, pub)
`pub fn ok( command: impl Into<String>, source: Source, data: ResultData, runtime: Runtime, ) -> Result<Self, ResultDataError>`

- **Does:** Build a terminal envelope with `status: ok` and the supplied `data`.
- **Calls (crate):** `ResultData::validate` (result.rs:738) L872
- **Calls (std/external/unresolved):** `.into`, `.is_none`, `.to_string`, `BTreeMap::new`, `Err`, `Ok`, `Some`, `Vec::new`
- **Macros:** `env!`
- **Called by:** `run` (bin/jubarte/check.rs:125); `generate` (bin/jubarte/edit_template.rs:32); `success` (bin/jubarte/edit_typed.rs:388); `tests::jsonl_round_trip_preserves_terminal_record` (result.rs:1361)

#### `Envelope::refused`  (L895-911, pub)
`pub fn refused(command: impl Into<String>, source: Source, error: Error) -> Self`

- **Does:** Build a terminal envelope with `status: refused` and a typed refusal.
- **Calls (crate):** `Error::with_commit_state` (result.rs:347) L896; `Runtime::native` (result.rs:500) L901; `ResultData::failure` (result.rs:697) L904
- **Calls (std/external/unresolved):** `.into`, `.to_string`, `BTreeMap::new`, `Some`, `Vec::new`
- **Macros:** `env!`
- **Called by:** `refused` (bin/jubarte/check.rs:103); `run` (bin/jubarte/check.rs:125); `run` (bin/jubarte/edit_template.rs:199); `run` (bin/jubarte/edit_typed.rs:787); `tests::status_serializes_as_lowercase_string` (result.rs:1343)

#### `Envelope::new_error`  (L916-932, pub)
`pub fn new_error(command: impl Into<String>, source: Source, mut error: Error) -> Self`

- **Does:** Build a terminal envelope with `status: error` and a typed failure.
- **Calls (crate):** `Error::normalize_retryable` (result.rs:353) L917; `Runtime::native` (result.rs:500) L922; `ResultData::failure` (result.rs:697) L925
- **Calls (std/external/unresolved):** `.into`, `.to_string`, `BTreeMap::new`, `Some`, `Vec::new`
- **Macros:** `env!`
- **Called by:** `usage` (bin/jubarte/check.rs:95); `run` (bin/jubarte/check.rs:125); `run` (bin/jubarte/edit_template.rs:199); `usage` (bin/jubarte/edit_template.rs:292); `run` (bin/jubarte/edit_typed.rs:787); `usage` (bin/jubarte/edit_typed.rs:962)

#### `Envelope::push_artifact`  (L935-937, pub)
`pub fn push_artifact(&mut self, artifact: Artifact)`

- **Does:** Append an artifact to the envelope.
- **Calls (std/external/unresolved):** `.push`
- **Called by:** none found in src

#### `Envelope::push_warning`  (L940-942, pub)
`pub fn push_warning(&mut self, warning: impl Into<String>)`

- **Does:** Append a warning to the envelope.
- **Calls (std/external/unresolved):** `.into`, `.push`
- **Called by:** none found in src

#### `Envelope::set_effective_option`  (L945-947, pub)
`pub fn set_effective_option(&mut self, key: impl Into<String>, value: serde_json::Value)`

- **Does:** Set an effective option.
- **Calls (std/external/unresolved):** `.insert`, `.into`
- **Called by:** none found in src

#### `<StreamHeader as From>::from`  (L966-973, impl From)
`fn from(envelope: &Envelope) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone`
- **Called by:** `write_jsonl_stream` (result.rs:1019)

#### `ProgressEvent::new`  (L990-992, pub)
`pub fn new(value: serde_json::Value) -> Self`

- **Does:** Build a progress event from a JSON value.
- **Calls:** none
- **Called by:** `<ProgressEvent as From>::from` (result.rs:1001); `read_jsonl_stream` (result.rs:1103)

#### `ProgressEvent::value`  (L995-997, pub)
`pub fn value(&self) -> &serde_json::Value`

- **Does:** Borrow the underlying JSON value.
- **Calls:** none
- **Called by:** none found in src

#### `<ProgressEvent as From>::from`  (L1001-1003, impl From)
`fn from(value: serde_json::Value) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `ProgressEvent::new` (result.rs:990) L1002
- **Called by:** none found in src

#### `<serde_json::Value as From>::from`  (L1007-1009, impl From)
`fn from(event: ProgressEvent) -> Self`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `jsonl_terminal_line`  (L1013-1015, pub)
`pub fn jsonl_terminal_line(terminal: &Envelope) -> String`

- **Does:** Serialize `terminal` as a single JSONL line (no trailing newline).
- **Calls (std/external/unresolved):** `.expect`, `serde_json::to_string`
- **Called by:** `write_jsonl_stream` (result.rs:1019)

#### `write_jsonl_stream`  (L1019-1030, pub)
`pub fn write_jsonl_stream(events: &[ProgressEvent], terminal: &Envelope) -> String`

- **Does:** Serialize a typed header, `events`, then exactly one terminal result.
- **Calls (crate):** `<StreamHeader as From>::from` (result.rs:966) L1021; `jsonl_terminal_line` (result.rs:1013) L1027
- **Calls (std/external/unresolved):** `.expect`, `.push_str`, `.push`, `.value`, `serde_json::to_string`
- **Called by:** `run` (bin/jubarte/edit_template.rs:199); `usage` (bin/jubarte/edit_template.rs:292); `run` (bin/jubarte/edit_typed.rs:787); `usage` (bin/jubarte/edit_typed.rs:962); `tests::jsonl_round_trip_preserves_terminal_record` (result.rs:1361)

#### `<StreamError as std::fmt::Display>::fmt`  (L1076-1091, impl std::fmt::Display)
`fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`

- **Does:** _no doc comment_
- **Macros:** `write!`
- **Called by:** none found in src

#### `read_jsonl_stream`  (L1103-1174, pub)
`pub fn read_jsonl_stream(input: &str) -> std::result::Result<ParsedStream, StreamError>`

- **Does:** Require a typed header first and exactly one terminal result last.
- **Calls (crate):** `ProgressEvent::new` (result.rs:990) L1163
- **Calls (std/external/unresolved):** `.and_then`, `.enumerate`, `.get`, `.is_empty`, `.is_none`, `.is_some`, `.lines`, `.map_err`, `.map`, `.ok_or`, `.push`, `.to_string`, `.trim`, `Err`, `Some`, `Vec::new`, `serde_json::from_str`, `serde_json::from_value`
- **Called by:** `tests::jsonl_round_trip_preserves_terminal_record` (result.rs:1361)

#### `<u8 as From>::from`  (L1201-1203, impl From)
`fn from(value: ExitStatus) -> Self`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `exit_status_for`  (L1215-1231, pub)
`pub fn exit_status_for(status: Status, error: Option<&Error>) -> ExitStatus`

- **Does:** Map a [`Status`] and the optional [`Error`] code to the proposal §12 exit code.
- **Calls (std/external/unresolved):** `.as_str`, `.map`
- **Called by:** `exit_status_for_envelope` (result.rs:1234); `tests::exit_status_for_ok_is_success` (result.rs:1304); `tests::exit_status_for_refused_with_no_code_is_refused` (result.rs:1309); `tests::error_status_for_unclassified_code_is_input_failure` (result.rs:1319); `tests::dedicated_exit_codes_win_over_the_status_fallback` (result.rs:1328)

#### `exit_status_for_envelope`  (L1234-1236, pub)
`pub fn exit_status_for_envelope(envelope: &Envelope) -> ExitStatus`

- **Does:** Map an [`Envelope`] to the proposal §12 exit code.
- **Calls (crate):** `exit_status_for` (result.rs:1215) L1235
- **Calls (std/external/unresolved):** `.as_ref`
- **Called by:** `finish` (bin/jubarte/check.rs:30); `edit_envelope_exit` (bin/jubarte.rs:1659)

## Test functions (13)

- `tests::fixture` (L1245): _no doc comment_
- `tests::not_applied_required_for_generated_phase` (L1250): _no doc comment_
- `tests::not_applied_required_for_validated_phase` (L1261): _no doc comment_
- `tests::committed_phase_may_carry_changed_or_unchanged` (L1269): _no doc comment_
- `tests::committed_phase_with_not_applied_is_rejected` (L1277): _no doc comment_
- `tests::predicted_effect_requires_validated_phase` (L1291): _no doc comment_
- `tests::exit_status_for_ok_is_success` (L1304): _no doc comment_
- `tests::exit_status_for_refused_with_no_code_is_refused` (L1309): _no doc comment_
- `tests::error_status_for_unclassified_code_is_input_failure` (L1319): _no doc comment_
- `tests::dedicated_exit_codes_win_over_the_status_fallback` (L1328): _no doc comment_
- `tests::status_serializes_as_lowercase_string` (L1343): _no doc comment_
- `tests::artifact_sha256_matches_source_sha256_helper` (L1354): _no doc comment_
- `tests::jsonl_round_trip_preserves_terminal_record` (L1361): _no doc comment_
