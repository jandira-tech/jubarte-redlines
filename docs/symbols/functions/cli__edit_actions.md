# Functions and methods in `src/cli/edit_actions.rs`

9 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `positive`  (L240-245)
`fn positive(s: &str) -> Result<usize, String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.filter`, `.into`, `.ok_or_else`, `.ok`, `.parse`
- **Called by:** none found in src

#### `digest`  (L246-255, pub(super))
`pub(super) fn digest(s: &str) -> Result<String, String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.contains`, `.into`, `.is_ascii_digit`, `.len`, `Err`, `Ok`
- **Called by:** none found in src

#### `route`  (L260-287, pub)
`pub fn route(arguments: &[OsString], model: &clap::Command) -> Vec<OsString>`

- **Does:** Route typed actions only when the tokens are not a valid legacy invocation.
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.clone`, `.find`, `.get`, `.into`, `.is_none_or`, `.is_ok`, `.is_some_and`, `.iter`, `.len`, `.skip`, `.starts_with`, `.to_str`, `.to_vec`, `.try_get_matches_from`, `Some`
- **Macros:** `matches!`
- **Called by:** `usage` (bin/jubarte/edit_template.rs:292); `edit_arguments` (cli.rs:3359); `parse_json` (cli.rs:3954)

#### `validate`  (L289-327, pub(super))
`pub(super) fn validate( args: &clap::ArgMatches, model: &mut clap::Command, ) -> Result<(), clap::Error>`

- **Does:** _no doc comment_
- **Calls (crate):** `ComparisonLog::error` (comparison_log.rs:64) (by name) L299; `present` (edit/v2.rs:420) (by name) L315
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.expect`, `.get_one`, `.into_iter`, `.is_some_and`, `.subcommand`, `.try_contains_id`, `.unwrap_or`, `.value_source`, `Err`, `Ok`, `Some`, `error`
- **Called by:** `validate_matches` (cli.rs:3007)

#### `EditAction::name`  (L331-339, pub)
`pub fn name(&self) -> &'static str`

- **Does:** Canonical action name for receipts.
- **Calls:** none
- **Called by:** none found in src

#### `EditAction::inline`  (L341-349, pub)
`pub fn inline(&self) -> Option<&InlineOptions>`

- **Does:** Inline policy, when this action compiles a new plan.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `prepare_and_publish` (bin/jubarte/edit_typed.rs:398); `EditAction::file` (cli/edit_actions.rs:360); `EditAction::delivery` (cli/edit_actions.rs:367); `compile` (edit/inline.rs:38); `Writer::inline` (markdown/from_docx/mod.rs:3218) (?)

#### `EditAction::payload`  (L351-358, pub)
`pub fn payload(&self) -> Option<&TextPayload>`

- **Does:** Exact payload or resource input for actions with text.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `prepare_and_publish` (bin/jubarte/edit_typed.rs:398); `compile` (edit/inline.rs:38)

#### `EditAction::file`  (L360-365, pub)
`pub fn file(&self) -> &std::path::Path`

- **Does:** Source path.
- **Calls (crate):** `EditAction::inline` (cli/edit_actions.rs:341) L363
- **Calls (std/external/unresolved):** `.expect`
- **Called by:** `prepare_and_publish` (bin/jubarte/edit_typed.rs:398) (?); `run` (bin/jubarte/edit_typed.rs:787) (?)

#### `EditAction::delivery`  (L367-372, pub)
`pub fn delivery(&self) -> &DeliveryOptions`

- **Does:** Delivery options, independent of plan-owned policy.
- **Calls (crate):** `EditAction::inline` (cli/edit_actions.rs:341) L370
- **Calls (std/external/unresolved):** `.expect`
- **Called by:** `request` (bin/jubarte/edit_typed.rs:294); `prepare_and_publish` (bin/jubarte/edit_typed.rs:398); `run` (bin/jubarte/edit_typed.rs:787) (?)
