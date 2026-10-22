# Functions and methods in `src/capabilities.rs`

9 non-test, 1 test. Part of the [symbol manifest](../README.md).

#### `Operations::entries`  (L210-267, pub)
`pub fn entries(&self) -> Vec<(&'static str, bool)>`

- **Does:** Each operation and whether this build has it, in declaration order (the same order as the field list).
- **Macros:** `vec!`
- **Called by:** `capabilities_text` (capabilities.rs:486); `rebind_to_output` (expect.rs:1008) (?)

#### `<InputBudget as From>::from`  (L323-331, impl From)
`fn from(l: crate::admission::InputLimits) -> Self`

- **Does:** Converts an [`crate::admission::InputLimits`] budget into the manifest-facing [`InputBudget`] (same fields, different struct).
- **Calls:** none
- **Called by:** none found in src

#### `dispatches_v2`  (L372-374)
`fn dispatches_v2(runtime: &str) -> bool`

- **Does:** Whether `runtime` dispatches PlanV2 end to end.
- **Macros:** `matches!`
- **Called by:** `profiles` (capabilities/discovery.rs:38); `support` (capabilities/discovery.rs:117); `dispatched_plan_versions` (capabilities.rs:385)

#### `skips_failed`  (L379-381)
`fn skips_failed(runtime: &str) -> bool`

- **Does:** Whether `runtime` runs an edit plan with `--skip-failed`: the CLI and a Rust embedder ([`crate::edit::PrepareOptions::skip_failed`]).
- **Macros:** `matches!`
- **Called by:** `capabilities` (capabilities.rs:399)

#### `dispatched_plan_versions`  (L385-391)
`fn dispatched_plan_versions(runtime: &str) -> Vec<u32>`

- **Does:** The plan versions `runtime` dispatches, ascending: the manifest's `edit_plan_versions` and the focused matrix agree on it.
- **Calls (crate):** `dispatches_v2` (capabilities.rs:372) L386
- **Macros:** `vec!`
- **Called by:** `discovery` (capabilities/discovery.rs:225); `capabilities` (capabilities.rs:399)

#### `capabilities`  (L399-474, pub)
`pub fn capabilities(runtime: &str) -> Capabilities`

- **Does:** Builds a [`Capabilities`] manifest for `runtime` (typically `cli`, `python`, `rust`, `wasm`).
- **Calls (crate):** `skips_failed` (capabilities.rs:379) L430; `dispatched_plan_versions` (capabilities.rs:385) L432; `dispatched_operation_kinds` (edit.rs:1891) L433; `<InputLimits as Default>::default` (admission.rs:64) L442; `strings` (capabilities.rs:476) L465,466,467,468,469,470
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.copied`, `.into`, `.iter`, `.map`, `.to_string`, `std::iter::once`
- **Macros:** `env!`, `format!`
- **Called by:** `cli_main` (bin/jubarte.rs:3935); `tests::assert_runtime_dispatch` (capabilities/discovery.rs:359); `capabilities_json` (capabilities.rs:551); `tests::manifest_reports_the_crate_version_and_every_edit_kind` (capabilities.rs:561)

#### `strings`  (L476-478)
`fn strings(values: &[&str]) -> Vec<String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`, `.to_string`
- **Called by:** `capabilities` (capabilities.rs:399); `residual_byte_diagnostic_contract_tests::line_diff_handles_identical_large_replacements_and_separated_hunks_exactly` (debug.rs:4543) (?)

#### `capabilities_text`  (L486-546, pub)
`pub fn capabilities_text(c: &Capabilities) -> String`

- **Does:** Renders a [`Capabilities`] as text, one line per field: an `engine:` header, `NAME: yes|no` per operation, plan versions and operations, stories, limits, validate rules, audit codes, `-o` formats and id spellings, ending with `run with --json for the manifest`.
- **Calls (crate):** `Operations::entries` (capabilities.rs:210) L489
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.filter_map`, `.get`, `.iter`, `.join`, `.map`, `.push_str`, `Some`, `yes`
- **Macros:** `format!`
- **Called by:** `cli_main` (bin/jubarte.rs:3935)

#### `capabilities_json`  (L551-553, pub)
`pub fn capabilities_json(runtime: &str) -> String`

- **Does:** The manifest serialized as pretty-printed JSON; the body `jubarte capabilities --json` prints and the form a host stores.
- **Calls (crate):** `capabilities` (capabilities.rs:399) L552
- **Calls (std/external/unresolved):** `.expect`, `serde_json::to_string_pretty`
- **Called by:** `tests::manifest_reports_the_crate_version_and_every_edit_kind` (capabilities.rs:561)

## Test functions (1)

- `tests::manifest_reports_the_crate_version_and_every_edit_kind` (L561): _no doc comment_
