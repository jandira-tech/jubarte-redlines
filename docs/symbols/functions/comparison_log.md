# Functions and methods in `src/comparison_log.rs`

5 non-test, 5 test. Part of the [symbol manifest](../README.md).

#### `ComparisonLog::new`  (L43-45, pub)
`pub fn new() -> Self`

- **Does:** Returns an empty log; equivalent to [`Default::default`].
- **Calls (std/external/unresolved):** `ComparisonLog::default`
- **Called by:** `CompareContext::new` (comparer/mod.rs:1651); `tests::test_new_is_empty` (comparison_log.rs:85); `tests::test_info` (comparison_log.rs:91); `tests::test_warning` (comparison_log.rs:100); `tests::test_error` (comparison_log.rs:109); `tests::test_error_count` (comparison_log.rs:118)

#### `ComparisonLog::info`  (L48-53, pub)
`pub fn info(&mut self, message: impl Into<String>)`

- **Does:** Appends a [`ComparisonLogCode::Info`] entry with `message`.
- **Calls (std/external/unresolved):** `.into`, `.push`
- **Called by:** `tests::test_info` (comparison_log.rs:91); `tests::test_error_count` (comparison_log.rs:118)

#### `ComparisonLog::warning`  (L56-61, pub)
`pub fn warning(&mut self, message: impl Into<String>)`

- **Does:** Appends a [`ComparisonLogCode::Warning`] entry with `message`.
- **Calls (std/external/unresolved):** `.into`, `.push`
- **Called by:** `tests::test_warning` (comparison_log.rs:100); `tests::test_error_count` (comparison_log.rs:118)

#### `ComparisonLog::error`  (L64-69, pub)
`pub fn error(&mut self, message: impl Into<String>)`

- **Does:** Appends a [`ComparisonLogCode::Error`] entry with `message`.
- **Calls (std/external/unresolved):** `.into`, `.push`
- **Called by:** `validate` (cli/edit_actions.rs:289) (?); `validate` (cli/edit_template.rs:101) (?); `usage_error` (cli.rs:670) (?); `read_usage_error` (cli.rs:2967) (?); `validate_matches` (cli.rs:3007) (?); `tests::test_error` (comparison_log.rs:109); `tests::test_error_count` (comparison_log.rs:118)

#### `ComparisonLog::error_count`  (L72-77, pub)
`pub fn error_count(&self) -> usize`

- **Does:** Number of [`ComparisonLogCode::Error`] entries currently in the log.
- **Calls (std/external/unresolved):** `.count`, `.filter`, `.iter`
- **Called by:** `tests::test_error_count` (comparison_log.rs:118) (?)

## Test functions (5)

- `tests::test_new_is_empty` (L85): _no doc comment_
- `tests::test_info` (L91): _no doc comment_
- `tests::test_warning` (L100): _no doc comment_
- `tests::test_error` (L109): _no doc comment_
- `tests::test_error_count` (L118): _no doc comment_
