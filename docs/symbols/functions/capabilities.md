# Functions and methods in `src/capabilities.rs`

6 non-test, 1 test. Part of the [symbol manifest](../README.md).

#### `Operations::entries`  (L200-255, pub)
`pub fn entries(&self) -> Vec<(&'static str, bool)>`

- **Does:** Each operation and whether this build has it, in declaration order (the same order as the field list).
- **Macros:** `vec!`
- **Called by:** `capabilities_text` (capabilities.rs:441)

#### `<InputBudget as From>::from`  (L311-319, impl From)
`fn from(l: crate::admission::InputLimits) -> Self`

- **Does:** Converts an [`crate::admission::InputLimits`] budget into the manifest-facing [`InputBudget`] (same fields, different struct).
- **Calls:** none
- **Called by:** none found in src

#### `capabilities`  (L355-429, pub)
`pub fn capabilities(runtime: &str) -> Capabilities`

- **Does:** Builds a [`Capabilities`] manifest for `runtime` (typically `cli`, `python`, `rust`, `wasm`).
- **Calls (crate):** `dispatched_operation_kinds` (edit.rs:1477) L388; `<InputLimits as Default>::default` (admission.rs:56) L397; `strings` (capabilities.rs:431) L420,421,422,423,424,425
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.copied`, `.into`, `.iter`, `.map`, `.to_string`, `std::iter::once`
- **Macros:** `env!`, `format!`, `vec!`
- **Called by:** `cli_main` (bin/jubarte.rs:2900); `tests::assert_runtime_dispatch` (capabilities/discovery.rs:324); `capabilities_json` (capabilities.rs:506); `tests::manifest_reports_the_crate_version_and_every_edit_kind` (capabilities.rs:516)

#### `strings`  (L431-433)
`fn strings(values: &[&str]) -> Vec<String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`, `.to_string`
- **Called by:** `capabilities` (capabilities.rs:355); `residual_byte_diagnostic_contract_tests::line_diff_handles_identical_large_replacements_and_separated_hunks_exactly` (debug.rs:4042) (?)

#### `capabilities_text`  (L441-501, pub)
`pub fn capabilities_text(c: &Capabilities) -> String`

- **Does:** Renders a [`Capabilities`] as text, one line per field: an `engine:` header, `NAME: yes|no` per operation, plan versions and operations, stories, limits, validate rules, audit codes, `-o` formats and id spellings, ending with `run with --json for the manifest`.
- **Calls (crate):** `Operations::entries` (capabilities.rs:200) L444
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.filter_map`, `.get`, `.iter`, `.join`, `.map`, `.push_str`, `Some`, `yes`
- **Macros:** `format!`
- **Called by:** `cli_main` (bin/jubarte.rs:2900)

#### `capabilities_json`  (L506-508, pub)
`pub fn capabilities_json(runtime: &str) -> String`

- **Does:** The manifest serialized as pretty-printed JSON; the body `jubarte capabilities --json` prints and the form a host stores.
- **Calls (crate):** `capabilities` (capabilities.rs:355) L507
- **Calls (std/external/unresolved):** `.expect`, `serde_json::to_string_pretty`
- **Called by:** `tests::manifest_reports_the_crate_version_and_every_edit_kind` (capabilities.rs:516)

## Test functions (1)

- `tests::manifest_reports_the_crate_version_and_every_edit_kind` (L516): _no doc comment_
