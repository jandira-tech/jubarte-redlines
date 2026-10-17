# Functions and methods in `src/builtin_styles.rs`

1 non-test, 5 test. Part of the [symbol manifest](../README.md).

#### `is_built_in`  (L394-402, pub(crate))
`pub(crate) fn is_built_in(name: &str) -> bool`

- **Does:** Whether `name` names one of Word's built-in styles, in any case.
- **Calls (std/external/unresolved):** `.binary_search_by`, `.bytes`, `.cmp`, `.is_ok`, `.map`, `.to_ascii_lowercase`
- **Called by:** `style_key` (append.rs:825); `tests::built_in_names_match_in_any_case` (builtin_styles.rs:410); `tests::is_built_in_matches_the_lowercasing_definition` (builtin_styles.rs:463); `style_match_key` (document_comparer.rs:1843)

## Test functions (5)

- `tests::built_in_names_match_in_any_case` (L410): _no doc comment_
- `tests::is_built_in_by_lowercasing` (L428): The allocation-free lookup answers exactly as the lowercase-then-search definition it replaced did.
- `tests::title_case` (L434): _no doc comment_
- `tests::alternating_case` (L449): _no doc comment_
- `tests::is_built_in_matches_the_lowercasing_definition` (L463): _no doc comment_
