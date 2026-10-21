# Functions and methods in `src/cli/edit_template.rs`

2 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `validate`  (L101-128, pub(super))
`pub(super) fn validate( args: &clap::ArgMatches, model: &mut clap::Command, ) -> Result<(), clap::Error>`

- **Does:** _no doc comment_
- **Calls (crate):** `ComparisonLog::error` (comparison_log.rs:64) (by name) L107
- **Calls (std/external/unresolved):** `.any`, `.clone`, `.expect`, `.get_flag`, `.get_one`, `.is_none`, `.is_some_and`, `.iter`, `.subcommand`, `Err`, `Ok`, `error`, `std::path::Path::new`
- **Called by:** `validate_matches` (cli.rs:3007)

#### `author_filter`  (L130-132)
`fn author_filter(value: &str) -> Result<String, String>`

- **Does:** _no doc comment_
- **Calls (crate):** `author` (markdown/mod.rs:404) L131
- **Calls (std/external/unresolved):** `.map`
- **Called by:** none found in src
