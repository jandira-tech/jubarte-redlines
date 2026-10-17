# Functions and methods in `src/result.rs`

53 non-test, 13 test. Part of the [symbol manifest](../README.md).

#### `Artifact::new`  (L186-200, pub)
`pub fn new( path: impl Into<String>, role: MediaRole, media_type: impl Into<String>, payload: &[u8], ) -> Self`

- **Does:** Build an artifact from `payload`; the digest is computed and `committed` defaults to `false`.
- **Calls (crate):** `source_sha256` (inspect.rs:271) L196
- **Calls (std/external/unresolved):** `.into`, `.len`
- **Called by:** `Artifact::committed` (result.rs:203); `tests::artifact_sha256_matches_source_sha256_helper` (result.rs:1308)

#### `Artifact::committed`  (L203-212, pub)
`pub fn committed( path: impl Into<String>, role: MediaRole, media_type: impl Into<String>, payload: &[u8], ) -> Self`

- **Does:** Build an artifact and mark it committed.
- **Calls (crate):** `Artifact::new` (result.rs:186) L209
- **Called by:** none found in src

#### `Artifact::with_committed`  (L216-219, pub)
`pub fn with_committed(mut self, committed: bool) -> Self`

- **Does:** Override `committed`.
- **Calls:** none
- **Called by:** none found in src

#### `Context::new`  (L240-251, pub)
`pub fn new( before: impl Into<String>, matched: impl Into<String>, after: impl Into<String>, ) -> Self`

- **Does:** Build a context with `truncated: false`; the caller sets the flag if the window was bounded.
- **Calls (std/external/unresolved):** `.into`
- **Called by:** none found in src

#### `Context::with_truncated`  (L255-258, pub)
`pub fn with_truncated(mut self, truncated: bool) -> Self`

- **Does:** Override the truncation flag.
- **Calls:** none
- **Called by:** none found in src

#### `NextAction::new`  (L278-284, pub)
`pub fn new(argv: Vec<String>) -> Self`

- **Does:** Build a next action from a structured argv array.
- **Calls:** none
- **Called by:** none found in src

#### `NextAction::with_purpose`  (L288-291, pub)
`pub fn with_purpose(mut self, purpose: impl Into<String>) -> Self`

- **Does:** Set the purpose string.
- **Calls (std/external/unresolved):** `.into`, `Some`
- **Called by:** none found in src

#### `NextAction::with_prerequisites`  (L295-298, pub)
`pub fn with_prerequisites(mut self, prerequisites: Vec<String>) -> Self`

- **Does:** Set the prerequisites list.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** none found in src

#### `Error::with_commit_state`  (L338-342, pub)
`pub fn with_commit_state(mut self, state: CommitState) -> Self`

- **Does:** Set the known publication state.
- **Calls (crate):** `Error::normalize_retryable` (result.rs:344) L340
- **Called by:** `Envelope::refused` (result.rs:849)

#### `Error::normalize_retryable`  (L344-348)
`fn normalize_retryable(&mut self)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `Error::with_commit_state` (result.rs:338); `Error::with_retryable` (result.rs:428); `Envelope::with_failure_data` (result.rs:774) (?); `Envelope::new_error` (result.rs:870)

#### `Error::new`  (L351-364, pub)
`pub fn new(code: impl Into<String>) -> Self`

- **Does:** Build an error with unknown publication state and no retry hint.
- **Calls (std/external/unresolved):** `.into`, `String::new`, `Vec::new`
- **Called by:** `tests::exit_status_for_refused_with_no_code_is_refused` (result.rs:1263); `tests::error_status_for_unclassified_code_is_input_failure` (result.rs:1273); `tests::dedicated_exit_codes_win_over_the_status_fallback` (result.rs:1282); `tests::status_serializes_as_lowercase_string` (result.rs:1297)

#### `Error::with_message`  (L368-371, pub)
`pub fn with_message(mut self, message: impl Into<String>) -> Self`

- **Does:** Set the message.
- **Calls (std/external/unresolved):** `.into`
- **Called by:** none found in src

#### `Error::with_operation`  (L375-378, pub)
`pub fn with_operation(mut self, operation: impl Into<String>) -> Self`

- **Does:** Set the operation pointer/id.
- **Calls (std/external/unresolved):** `.into`, `Some`
- **Called by:** none found in src

#### `Error::with_source`  (L382-385, pub)
`pub fn with_source(mut self, source: Source) -> Self`

- **Does:** Bind the source digest.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** none found in src

#### `Error::with_source_digest`  (L389-392, pub)
`pub fn with_source_digest(mut self, digest: impl Into<String>) -> Self`

- **Does:** Override the source digest directly.
- **Calls (std/external/unresolved):** `.into`, `Some`
- **Called by:** none found in src

#### `Error::with_target`  (L396-399, pub)
`pub fn with_target(mut self, target: impl Into<String>) -> Self`

- **Does:** Set the resolved target pointer.
- **Calls (std/external/unresolved):** `.into`, `Some`
- **Called by:** none found in src

#### `Error::with_matches`  (L403-406, pub)
`pub fn with_matches(mut self, matches: usize) -> Self`

- **Does:** Set the matches count.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** none found in src

#### `Error::with_context`  (L410-413, pub)
`pub fn with_context(mut self, context: Context) -> Self`

- **Does:** Set the bounded context.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** none found in src

#### `Error::set_context`  (L417-419, pub)
`pub fn set_context(&mut self, context: Context)`

- **Does:** Set the bounded context (in-place variant for chaining after mutating other fields).
- **Calls (std/external/unresolved):** `Some`
- **Called by:** none found in src

#### `Error::push_next_action`  (L422-424, pub)
`pub fn push_next_action(&mut self, action: NextAction)`

- **Does:** Push a structured argv next action.
- **Calls (std/external/unresolved):** `.push`
- **Called by:** none found in src

#### `Error::with_retryable`  (L428-432, pub)
`pub fn with_retryable(mut self, retryable: bool) -> Self`

- **Does:** Set the retryable hint.
- **Calls (crate):** `Error::normalize_retryable` (result.rs:344) L430
- **Calls (std/external/unresolved):** `Some`
- **Called by:** none found in src

#### `Source::new`  (L452-458, pub)
`pub fn new(bytes: &[u8]) -> Self`

- **Does:** Build a `Source` from raw bytes: the digest is computed and the byte size recorded; no path is set.
- **Calls (crate):** `source_sha256` (inspect.rs:271) L454
- **Calls (std/external/unresolved):** `.len`, `Some`
- **Called by:** `tests::status_serializes_as_lowercase_string` (result.rs:1297); `tests::jsonl_round_trip_preserves_terminal_record` (result.rs:1315)

#### `Source::from_digest`  (L461-467, pub)
`pub fn from_digest(digest: impl Into<String>) -> Self`

- **Does:** Build a `Source` from a precomputed digest.
- **Calls (std/external/unresolved):** `.into`
- **Called by:** none found in src

#### `Source::with_path`  (L471-474, pub)
`pub fn with_path(mut self, path: impl Into<String>) -> Self`

- **Does:** Set the path.
- **Calls (std/external/unresolved):** `.into`, `Some`
- **Called by:** none found in src

#### `Runtime::native`  (L489-494, pub)
`pub fn native() -> Self`

- **Does:** The native CLI runtime.
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `Envelope::refused` (result.rs:849); `Envelope::new_error` (result.rs:870); `tests::jsonl_round_trip_preserves_terminal_record` (result.rs:1315)

#### `<Verification as Default>::default`  (L512-519, impl Default)
`fn default() -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.to_string`
- **Called by:** `ResultData::failure` (result.rs:659); `ResultData::new` (result.rs:674)

#### `Counts::total_operations`  (L578-580, pub)
`pub fn total_operations(&self) -> usize`

- **Does:** Sum of `body_operations_applied` + `review_operations_applied`.
- **Calls:** none
- **Called by:** none found in src

#### `is_zero`  (L583-585)
`fn is_zero(value: &usize) -> bool`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `<ResultDataError as std::fmt::Display>::fmt`  (L635-652, impl std::fmt::Display)
`fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`

- **Does:** _no doc comment_
- **Macros:** `write!`
- **Called by:** none found in src

#### `ResultData::failure`  (L659-667, pub)
`pub fn failure(phase: Option<Phase>) -> Self`

- **Does:** Build pre-commit failure data with the last completed phase.
- **Calls (crate):** `<Verification as Default>::default` (result.rs:512) L665
- **Calls (std/external/unresolved):** `Counts::default`
- **Called by:** `Envelope::refused` (result.rs:849); `Envelope::new_error` (result.rs:870)

#### `ResultData::new`  (L674-682, pub)
`pub fn new(phase: Phase, effect: Effect) -> Self`

- **Does:** Build a result data block.
- **Calls (crate):** `<Verification as Default>::default` (result.rs:512) L680
- **Calls (std/external/unresolved):** `Counts::default`, `Some`
- **Called by:** `tests::not_applied_required_for_generated_phase` (result.rs:1204); `tests::not_applied_required_for_validated_phase` (result.rs:1215); `tests::committed_phase_may_carry_changed_or_unchanged` (result.rs:1223); `tests::committed_phase_with_not_applied_is_rejected` (result.rs:1231); `tests::predicted_effect_requires_validated_phase` (result.rs:1245); `tests::jsonl_round_trip_preserves_terminal_record` (result.rs:1315)

#### `ResultData::set_predicted`  (L686-688, pub)
`pub fn set_predicted(&mut self, predicted: PredictedEffect)`

- **Does:** Set the predicted effect (dry runs only).
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `tests::predicted_effect_requires_validated_phase` (result.rs:1245); `tests::jsonl_round_trip_preserves_terminal_record` (result.rs:1315)

#### `ResultData::validate`  (L692-720, pub)
`pub fn validate(&self) -> Result<(), ResultDataError>`

- **Does:** Validate the (phase, effect, predicted_effect) combination.
- **Calls (std/external/unresolved):** `.is_some`, `Err`, `Ok`
- **Called by:** `Envelope::with_failure_data` (result.rs:774); `Envelope::ok` (result.rs:820)

#### `Envelope::with_failure_data`  (L774-791, pub)
`pub fn with_failure_data(mut self, data: ResultData) -> Result<Self, ResultDataError>`

- **Does:** Attach validated phase/effect/counts to a failure envelope.
- **Calls (crate):** `ResultData::validate` (result.rs:692) L778; `Error::normalize_retryable` (result.rs:344) (by name) L787
- **Calls (std/external/unresolved):** `Err`, `Ok`, `Some`
- **Called by:** none found in src

#### `Envelope::new`  (L797-812, pub)
`pub fn new(command: impl Into<String>, source: Source, runtime: Runtime) -> Self`

- **Does:** Build a placeholder envelope with `status: ok` and no data; used by tests and scaffold code.
- **Calls (std/external/unresolved):** `.into`, `.to_string`, `BTreeMap::new`, `Vec::new`
- **Macros:** `env!`
- **Called by:** none found in src

#### `Envelope::ok`  (L820-844, pub)
`pub fn ok( command: impl Into<String>, source: Source, data: ResultData, runtime: Runtime, ) -> Result<Self, ResultDataError>`

- **Does:** Build a terminal envelope with `status: ok` and the supplied `data`.
- **Calls (crate):** `ResultData::validate` (result.rs:692) L826
- **Calls (std/external/unresolved):** `.into`, `.is_none`, `.to_string`, `BTreeMap::new`, `Err`, `Ok`, `Some`, `Vec::new`
- **Macros:** `env!`
- **Called by:** `tests::jsonl_round_trip_preserves_terminal_record` (result.rs:1315)

#### `Envelope::refused`  (L849-865, pub)
`pub fn refused(command: impl Into<String>, source: Source, error: Error) -> Self`

- **Does:** Build a terminal envelope with `status: refused` and a typed refusal.
- **Calls (crate):** `Error::with_commit_state` (result.rs:338) L850; `Runtime::native` (result.rs:489) L855; `ResultData::failure` (result.rs:659) L858
- **Calls (std/external/unresolved):** `.into`, `.to_string`, `BTreeMap::new`, `Some`, `Vec::new`
- **Macros:** `env!`
- **Called by:** `tests::status_serializes_as_lowercase_string` (result.rs:1297)

#### `Envelope::new_error`  (L870-886, pub)
`pub fn new_error(command: impl Into<String>, source: Source, mut error: Error) -> Self`

- **Does:** Build a terminal envelope with `status: error` and a typed failure.
- **Calls (crate):** `Error::normalize_retryable` (result.rs:344) L871; `Runtime::native` (result.rs:489) L876; `ResultData::failure` (result.rs:659) L879
- **Calls (std/external/unresolved):** `.into`, `.to_string`, `BTreeMap::new`, `Some`, `Vec::new`
- **Macros:** `env!`
- **Called by:** none found in src

#### `Envelope::push_artifact`  (L889-891, pub)
`pub fn push_artifact(&mut self, artifact: Artifact)`

- **Does:** Append an artifact to the envelope.
- **Calls (std/external/unresolved):** `.push`
- **Called by:** none found in src

#### `Envelope::push_warning`  (L894-896, pub)
`pub fn push_warning(&mut self, warning: impl Into<String>)`

- **Does:** Append a warning to the envelope.
- **Calls (std/external/unresolved):** `.into`, `.push`
- **Called by:** none found in src

#### `Envelope::set_effective_option`  (L899-901, pub)
`pub fn set_effective_option(&mut self, key: impl Into<String>, value: serde_json::Value)`

- **Does:** Set an effective option.
- **Calls (std/external/unresolved):** `.insert`, `.into`
- **Called by:** none found in src

#### `<StreamHeader as From>::from`  (L920-927, impl From)
`fn from(envelope: &Envelope) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.clone`
- **Called by:** `write_jsonl_stream` (result.rs:973)

#### `ProgressEvent::new`  (L944-946, pub)
`pub fn new(value: serde_json::Value) -> Self`

- **Does:** Build a progress event from a JSON value.
- **Calls:** none
- **Called by:** `<ProgressEvent as From>::from` (result.rs:955); `read_jsonl_stream` (result.rs:1057)

#### `ProgressEvent::value`  (L949-951, pub)
`pub fn value(&self) -> &serde_json::Value`

- **Does:** Borrow the underlying JSON value.
- **Calls:** none
- **Called by:** none found in src

#### `<ProgressEvent as From>::from`  (L955-957, impl From)
`fn from(value: serde_json::Value) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `ProgressEvent::new` (result.rs:944) L956
- **Called by:** none found in src

#### `<serde_json::Value as From>::from`  (L961-963, impl From)
`fn from(event: ProgressEvent) -> Self`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `jsonl_terminal_line`  (L967-969, pub)
`pub fn jsonl_terminal_line(terminal: &Envelope) -> String`

- **Does:** Serialize `terminal` as a single JSONL line (no trailing newline).
- **Calls (std/external/unresolved):** `.expect`, `serde_json::to_string`
- **Called by:** `write_jsonl_stream` (result.rs:973)

#### `write_jsonl_stream`  (L973-984, pub)
`pub fn write_jsonl_stream(events: &[ProgressEvent], terminal: &Envelope) -> String`

- **Does:** Serialize a typed header, `events`, then exactly one terminal result.
- **Calls (crate):** `<StreamHeader as From>::from` (result.rs:920) L975; `jsonl_terminal_line` (result.rs:967) L981
- **Calls (std/external/unresolved):** `.expect`, `.push_str`, `.push`, `.value`, `serde_json::to_string`
- **Called by:** `tests::jsonl_round_trip_preserves_terminal_record` (result.rs:1315)

#### `<StreamError as std::fmt::Display>::fmt`  (L1030-1045, impl std::fmt::Display)
`fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`

- **Does:** _no doc comment_
- **Macros:** `write!`
- **Called by:** none found in src

#### `read_jsonl_stream`  (L1057-1128, pub)
`pub fn read_jsonl_stream(input: &str) -> std::result::Result<ParsedStream, StreamError>`

- **Does:** Require a typed header first and exactly one terminal result last.
- **Calls (crate):** `ProgressEvent::new` (result.rs:944) L1117
- **Calls (std/external/unresolved):** `.and_then`, `.enumerate`, `.get`, `.is_empty`, `.is_none`, `.is_some`, `.lines`, `.map_err`, `.map`, `.ok_or`, `.push`, `.to_string`, `.trim`, `Err`, `Some`, `Vec::new`, `serde_json::from_str`, `serde_json::from_value`
- **Called by:** `tests::jsonl_round_trip_preserves_terminal_record` (result.rs:1315)

#### `<u8 as From>::from`  (L1155-1157, impl From)
`fn from(value: ExitStatus) -> Self`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `exit_status_for`  (L1169-1185, pub)
`pub fn exit_status_for(status: Status, error: Option<&Error>) -> ExitStatus`

- **Does:** Map a [`Status`] and the optional [`Error`] code to the proposal §12 exit code.
- **Calls (std/external/unresolved):** `.as_str`, `.map`
- **Called by:** `exit_status_for_envelope` (result.rs:1188); `tests::exit_status_for_ok_is_success` (result.rs:1258); `tests::exit_status_for_refused_with_no_code_is_refused` (result.rs:1263); `tests::error_status_for_unclassified_code_is_input_failure` (result.rs:1273); `tests::dedicated_exit_codes_win_over_the_status_fallback` (result.rs:1282)

#### `exit_status_for_envelope`  (L1188-1190, pub)
`pub fn exit_status_for_envelope(envelope: &Envelope) -> ExitStatus`

- **Does:** Map an [`Envelope`] to the proposal §12 exit code.
- **Calls (crate):** `exit_status_for` (result.rs:1169) L1189
- **Calls (std/external/unresolved):** `.as_ref`
- **Called by:** none found in src

## Test functions (13)

- `tests::fixture` (L1199): _no doc comment_
- `tests::not_applied_required_for_generated_phase` (L1204): _no doc comment_
- `tests::not_applied_required_for_validated_phase` (L1215): _no doc comment_
- `tests::committed_phase_may_carry_changed_or_unchanged` (L1223): _no doc comment_
- `tests::committed_phase_with_not_applied_is_rejected` (L1231): _no doc comment_
- `tests::predicted_effect_requires_validated_phase` (L1245): _no doc comment_
- `tests::exit_status_for_ok_is_success` (L1258): _no doc comment_
- `tests::exit_status_for_refused_with_no_code_is_refused` (L1263): _no doc comment_
- `tests::error_status_for_unclassified_code_is_input_failure` (L1273): _no doc comment_
- `tests::dedicated_exit_codes_win_over_the_status_fallback` (L1282): _no doc comment_
- `tests::status_serializes_as_lowercase_string` (L1297): _no doc comment_
- `tests::artifact_sha256_matches_source_sha256_helper` (L1308): _no doc comment_
- `tests::jsonl_round_trip_preserves_terminal_record` (L1315): _no doc comment_
