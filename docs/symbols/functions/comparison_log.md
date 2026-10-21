# Functions and methods in `src/comparison_log.rs`

5 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `ComparisonLog::new`  (L43-45, pub)
`pub fn new() -> Self`

- **Does:** Returns an empty log; equivalent to [`Default::default`].
- **Calls (std/external/unresolved):** `ComparisonLog::default`
- **Called by:** `CompareContext::new` (comparer/mod.rs:1651)

#### `ComparisonLog::info`  (L48-53, pub)
`pub fn info(&mut self, message: impl Into<String>)`

- **Does:** Appends a [`ComparisonLogCode::Info`] entry with `message`.
- **Calls (std/external/unresolved):** `.into`, `.push`
- **Called by:** none found in src

#### `ComparisonLog::warning`  (L56-61, pub)
`pub fn warning(&mut self, message: impl Into<String>)`

- **Does:** Appends a [`ComparisonLogCode::Warning`] entry with `message`.
- **Calls (std/external/unresolved):** `.into`, `.push`
- **Called by:** none found in src

#### `ComparisonLog::error`  (L64-69, pub)
`pub fn error(&mut self, message: impl Into<String>)`

- **Does:** Appends a [`ComparisonLogCode::Error`] entry with `message`.
- **Calls (std/external/unresolved):** `.into`, `.push`
- **Called by:** `validate` (cli/edit_actions.rs:289) (?); `validate` (cli/edit_template.rs:101) (?); `usage_error` (cli.rs:670) (?); `read_usage_error` (cli.rs:2967) (?); `validate_matches` (cli.rs:3007) (?); `parse_json` (cli.rs:3791) (?)

#### `ComparisonLog::error_count`  (L72-77, pub)
`pub fn error_count(&self) -> usize`

- **Does:** Number of [`ComparisonLogCode::Error`] entries currently in the log.
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.iter`
- **Called by:** none found in src
