# Functions and methods in `src/cli.rs`

93 non-test, 10 test. Part of the [symbol manifest](../README.md).

#### `EditOptions::target`  (L358-376, pub)
`pub fn target(&self, file: &Path) -> EditTarget`

- **Does:** Where `edit FILE` (or `add FILE`) writes: `--out-dir [DIR]`'s bundle, else `-o`, else `<FILE's dir>/<stem>_modified.docx`.
- **Calls (std/external/unresolved):** `.as_os_str`, `.clone`, `.ends_with`, `.file_stem`, `.filter`, `.into_owned`, `.is_empty`, `.join`, `.map_or_else`, `.parent`, `.to_string_lossy`, `.to_string`, `EditTarget::Bundle`, `EditTarget::Single`, `PathBuf::from`, `beside`
- **Macros:** `format!`
- **Called by:** `run_flags` (bin/jubarte.rs:2511); `parse_subcommand_json` (cli.rs:3779) (?); `with_operation_targets` (edit/v2_apply.rs:97) (?); `compose_threads` (edit/v2_apply.rs:1821) (?); `Transaction::resolve_thread` (edit.rs:4379) (?); `Transaction::check_thread_ops` (edit.rs:5715) (?); `run` (update.rs:225) (?)

#### `ExistingArg::plan_value`  (L398-407, pub)
`pub fn plan_value(self) -> Option<crate::edit::ExistingRevisions>`

- **Does:** The plan's `existing_revisions`; `None` for `auto`.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `edit_plan` (bin/jubarte.rs:1304); `record_plan` (bin/jubarte.rs:2625) (?)

#### `flag_operations`  (L412-470, pub)
`pub fn flag_operations(matches: &clap::ArgMatches) -> Result<Vec<FlagOp>, String>`

- **Does:** Groups the operation flags of an `edit` or `add` invocation by their `-p`, in command-line order.
- **Calls (std/external/unresolved):** `.clone`, `.extend`, `.flatten`, `.get_many`, `.indices_of`, `.is_none`, `.is_some`, `.last_mut`, `.map`, `.ok`, `.push`, `.sort_by_key`, `.try_get_many`, `.unwrap_or_default`, `.zip`, `Err`, `FlagOp::default`, `Ok`, `Some`, `Vec::new`, `twice`
- **Macros:** `format!`
- **Called by:** `fill_flag_operations` (cli.rs:738)

#### `plan_path`  (L477-488)
`fn plan_path() -> impl clap::builder::TypedValueParser<Value = PathBuf>`

- **Does:** `--plan`'s parser: a path that may be empty, the value `--plan` takes when given bare (clap's `PathBuf` parser refuses an empty one).
- **Calls (std/external/unresolved):** `.ends_with`, `.to_string_lossy`, `.try_map`, `Err`, `Ok`, `PathBuf::from`, `clap::builder::OsStringValueParser::new`
- **Macros:** `cfg!`, `format!`
- **Called by:** none found in src

#### `hoist_quiet`  (L494-522)
`fn hoist_quiet<T: AsRef<std::ffi::OsStr> + Clone>( command: &clap::Command, arguments: &[T], ) -> Vec<T>`

- **Does:** `-q`/`--quiet` before the task moves after it, so `jubarte -q validate F` reads as `jubarte validate -q F` (D6 follow-up).
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.as_ref`, `.bytes`, `.clone`, `.count`, `.extend_from_slice`, `.find_subcommand`, `.get`, `.is_none_or`, `.is_none`, `.iter`, `.len`, `.push`, `.skip`, `.starts_with`, `.take_while`, `.to_str`, `.to_vec`, `Some`, `Vec::with_capacity`, `is_quiet`
- **Macros:** `vec!`
- **Called by:** `edit_arguments` (cli.rs:3359); `parse_json` (cli.rs:3954)

#### `fill_date`  (L526-530)
`fn fill_date(date: &mut Option<String>)`

- **Does:** `compare`'s and `diff`'s missing `-d` is now, in UTC, to the second (D17): filled once here, so the binary and every host share it.
- **Calls (crate):** `utc_now_iso8601` (convert/mod.rs:12307) L528
- **Calls (std/external/unresolved):** `.is_none`, `Some`
- **Called by:** `fill_task_date` (cli.rs:533); `Cli::try_parse_from` (cli.rs:3491); `parse_implicit_compare_json` (cli.rs:3890)

#### `fill_task_date`  (L533-539)
`fn fill_task_date(task: &mut Command)`

- **Does:** [`fill_date`] for the task that dates its revisions.
- **Calls (crate):** `fill_date` (cli.rs:526) L535,536
- **Called by:** `Cli::try_parse_from` (cli.rs:3491); `parse_subcommand_json` (cli.rs:3779)

#### `slash_path`  (L553-556, pub)
`pub fn slash_path(path: &Path) -> String`

- **Does:** A path as the CLI's agent-facing outputs spell it: `/` between the parts on every OS (#568).
- **Calls (crate):** `unverbatimed` (cli.rs:565) L554
- **Calls (std/external/unresolved):** `.replace`, `.to_path_buf`, `.to_string_lossy`, `.unwrap_or_else`
- **Called by:** `absolute` (bin/jubarte.rs:168); `Output::of_task` (bin/jubarte.rs:299); `check_side_file` (bin/jubarte.rs:423); `run_convert` (bin/jubarte.rs:646); `print_success` (bin/jubarte.rs:1181); `run_edit` (bin/jubarte.rs:1341); `publish_edit` (bin/jubarte.rs:1402); `taken_output` (bin/jubarte.rs:1674); `refuse_output_alias` (bin/jubarte.rs:1701); `recheck_edit_source` (bin/jubarte.rs:1730); `refuse_bundle_input_containment` (bin/jubarte.rs:1758); `comma_paths` (bin/jubarte.rs:1787); +9 more

#### `unverbatimed`  (L565-578)
`fn unverbatimed(path: &Path) -> Option<PathBuf>`

- **Does:** `path` without the verbatim prefix `canonicalize` adds on Windows, or `None` when there is none to drop.
- **Calls (crate):** `verbatim_head` (cli.rs:585) L567
- **Calls (std/external/unresolved):** `.as_os_str`, `.components`, `.next`, `.push`, `Some`
- **Macros:** `matches!`
- **Called by:** `slash_path` (cli.rs:553); `slash_path_tests::unverbatimed_keeps_every_nonverbatim_path` (cli.rs:4097); `slash_path_tests::a_verbatim_literal_is_an_ordinary_name_off_windows` (cli.rs:4107); `slash_path_tests::unverbatimed_returns_the_stripped_components` (cli.rs:4130)

#### `verbatim_head`  (L585-604)
`fn verbatim_head(component: Option<std::path::Component>) -> Option<(PathBuf, bool)>`

- **Does:** The path head a verbatim prefix becomes, and whether the root after it stays, or `None` for any other first component (`Prefix` is a Windows notion; off Windows this is always `None` and [`unverbatimed`] changes nothing).
- **Calls (std/external/unresolved):** `.kind`, `.push`, `.to_ascii_uppercase`, `PathBuf::from`, `Some`
- **Macros:** `format!`
- **Called by:** `unverbatimed` (cli.rs:565)

#### `verbatim_head`  (L608-610)
`fn verbatim_head(_component: Option<std::path::Component>) -> Option<(PathBuf, bool)>`

- **Does:** Off Windows no component carries a prefix, so there is nothing to drop.
- **Calls:** none
- **Called by:** `unverbatimed` (cli.rs:565)

#### `resolve_plan`  (L613-642)
`fn resolve_plan(task: &mut Command)`

- **Does:** `--plan` without a value (D16) is `<FILE's dir>/<stem>_plan.json`.
- **Calls (crate):** `.parent` (ambiguous: `Marks::parent` (markdown/view_stories.rs:450); `Dom::parent` (xmllinq/mod.rs:597)) L637
- **Calls (std/external/unresolved):** `.as_os_str`, `.file_stem`, `.filter`, `.into_owned`, `.is_empty`, `.join`, `.map_or_else`, `.to_string_lossy`, `.to_string`, `PathBuf::from`
- **Macros:** `format!`
- **Called by:** `Cli::try_parse_from` (cli.rs:3491); `parse_subcommand_json` (cli.rs:3779)

#### `names_word_document`  (L646-654)
`fn names_word_document(path: &Path) -> bool`

- **Does:** A Word document's name, by extension: what an optional value swallowed when it should have been FILE.
- **Calls (std/external/unresolved):** `.any`, `.ends_with`, `.file_name`, `.is_some_and`, `.iter`, `.to_ascii_lowercase`, `.to_string_lossy`
- **Called by:** `usage_error` (cli.rs:670); `validate_matches` (cli.rs:3007)

#### `swallowed_file`  (L658-664)
`fn swallowed_file(flag: &str, value: &Path) -> String`

- **Does:** The message for an optional value (`--plan`, `--out-dir`) that names a Word document: FILE was read as the value.
- **Calls (std/external/unresolved):** `.display`
- **Macros:** `format!`
- **Called by:** `usage_error` (cli.rs:670); `validate_matches` (cli.rs:3007)

#### `usage_error`  (L670-734)
`fn usage_error(command: &clap::Command, tokens: &[String], mut error: clap::Error) -> clap::Error`

- **Does:** Rewrite known usage traps through clap's own error formatter.
- **Calls (crate):** `deeper_boundary_tests::names` (edit.rs:8789) L684,687; `read_usage_error` (cli.rs:2967) L685; `read_view_conflict` (cli.rs:2957) L685; `names_word_document` (cli.rs:646) L721; `swallowed_file` (cli.rs:658) L723; `ComparisonLog::error` (comparison_log.rs:64) (by name) L733
- **Calls (std/external/unresolved):** `.any`, `.as_str`, `.clone`, `.collect`, `.filter_map`, `.filter`, `.find_subcommand`, `.get_name`, `.get`, `.insert`, `.into_iter`, `.into`, `.iter`, `.join`, `.kind`, `.map`, `.position`, `.set_bin_name`, `.skip`, `.split_once`, `.split_whitespace`, `.starts_with`, `.take_while`, `Path::new`, `Some`, `clap::error::ContextValue::StyledStr`
- **Macros:** `format!`, `matches!`
- **Called by:** `Cli::try_parse_from` (cli.rs:3491); `parse_json` (cli.rs:3954)

#### `fill_flag_operations`  (L738-754)
`fn fill_flag_operations( task: &mut Command, matches: &clap::ArgMatches, model: &mut clap::Command, ) -> Result<(), clap::Error>`

- **Does:** Groups the flag operations of `edit` and `add` into their `operations` field; a grouping error is a usage error.
- **Calls (crate):** `flag_operations` (cli.rs:412) L751
- **Calls (std/external/unresolved):** `.error`, `.map_err`, `.subcommand_matches`, `Ok`
- **Called by:** `Cli::try_parse_from` (cli.rs:3491); `parse_subcommand_json` (cli.rs:3779)

#### `top_level_command`  (L856-869, pub)
`pub fn top_level_command() -> clap::Command`

- **Does:** The derive model with the shorthands' options hidden: `jubarte --help` lists the tasks; `jubarte compare --help` and `jubarte read --help` show every option.
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.contains`, `.get_arguments`, `.get_id`, `.hide`, `.map`, `.mut_args`, `::augment_args`, `Cli::command`, `clap::Command::new`
- **Called by:** `usage` (bin/jubarte/edit_template.rs:292); `typed_edit_protocol` (cli.rs:3370); `next_command` (cli.rs:3413); `Cli::try_parse_from` (cli.rs:3491); `wants_json` (cli.rs:3611); `parse_json` (cli.rs:3954)

#### `<Cli as std::ops::Deref>::deref`  (L873-875, impl std::ops::Deref)
`fn deref(&self) -> &Self::Target`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `<crate::debug::Check as From>::from`  (L1753-1774, impl From)
`fn from(c: DebugCheck) -> Self`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `PatchFormat::is_text_view`  (L1801-1803, pub)
`pub fn is_text_view(self) -> bool`

- **Does:** Whether this format compares document snapshots without constructing a redline.
- **Macros:** `matches!`
- **Called by:** `cli_main` (bin/jubarte.rs:3935) (?); `validate_matches` (cli.rs:3007) (?)

#### `extension`  (L1840-1848)
`fn extension(path: &Path) -> Option<String>`

- **Does:** A file name's extension: `Path::extension`, or the whole name after its dot when the name is only an extension (`.json`, `out/.md`), which Rust reads as a hidden file with none.
- **Calls (std/external/unresolved):** `.contains`, `.extension`, `.file_name`, `.filter`, `.is_empty`, `.map`, `.strip_prefix`, `.to_str`
- **Called by:** `Format::of_path` (cli.rs:1853); `Task::known` (cli.rs:1960)

#### `Format::of_path`  (L1853-1855, pub)
`pub fn of_path(path: &Path) -> Option<Self>`

- **Does:** The format a file name says, by extension (`.json`, a name that is only an extension, says json too).
- **Calls (crate):** `Format::of_name` (cli.rs:1858) L1854; `extension` (cli.rs:1840) L1854
- **Called by:** `Format::of_input` (cli.rs:1882); `Task::known` (cli.rs:1960); `validate_matches` (cli.rs:3007); `compare_json` (cli.rs:3542); `parse_subcommand_json` (cli.rs:3779)

#### `Format::of_name`  (L1858-1867, pub)
`pub fn of_name(extension: &str) -> Option<Self>`

- **Does:** The format an extension names, in any case.
- **Calls (std/external/unresolved):** `.as_str`, `.to_ascii_lowercase`, `Some`
- **Called by:** `Format::of_path` (cli.rs:1853); `Task::allowed` (cli.rs:1945)

#### `Format::name`  (L1870-1878, pub)
`pub fn name(self) -> &'static str`

- **Does:** The name `--to` takes and messages print.
- **Calls:** none
- **Called by:** `write_output` (bin/jubarte.rs:353); `Task::cannot_write` (cli.rs:1971)

#### `Format::of_input`  (L1882-1890, pub)
`pub fn of_input(asked: Option<Self>, path: &Path, bytes: &[u8]) -> Self`

- **Does:** An input's format: the one asked for, else its extension, else its bytes (a zip is Word, anything else Markdown).
- **Calls (crate):** `Format::of_path` (cli.rs:1853) L1884
- **Calls (std/external/unresolved):** `.filter`, `.or_else`, `.starts_with`, `.unwrap_or`
- **Macros:** `matches!`
- **Called by:** `run` (bin/jubarte.rs:2862); `diff_inputs` (bin/jubarte.rs:3347); `run_convert_any` (bin/jubarte.rs:3458)

#### `Task::name`  (L1916-1927, pub)
`pub fn name(self) -> &'static str`

- **Does:** The command's name.
- **Calls:** none
- **Called by:** `Task::allowed` (cli.rs:1945)

#### `Task::of_name`  (L1930-1942, pub)
`pub fn of_name(name: &str) -> Option<Self>`

- **Does:** The task a command name (or its alias) runs.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `validate_matches` (cli.rs:3007)

#### `Task::allowed`  (L1945-1956)
`fn allowed(self) -> Vec<Format>`

- **Does:** The formats its `-o` writes (`capabilities::OUTPUT_FORMATS`).
- **Calls (crate):** `Task::name` (cli.rs:1916) L1948; `Format::of_name` (cli.rs:1858) L1952
- **Calls (std/external/unresolved):** `.collect`, `.expect`, `.find`, `.iter`, `.map`, `.unwrap_or_default`
- **Called by:** `output_format` (cli.rs:1996)

#### `Task::known`  (L1960-1967)
`fn known(self, path: &Path) -> Option<Format>`

- **Does:** What `-o PATH`'s extension says for this task: `diff` reads `.patch` and `.diff` as text too.
- **Calls (crate):** `extension` (cli.rs:1840) L1961; `Format::of_path` (cli.rs:1853) L1966
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`, `.is_some_and`, `Some`
- **Called by:** `output_format` (cli.rs:1996); `json_not_yet` (cli.rs:2912)

#### `Task::cannot_write`  (L1971-1988)
`fn cannot_write(self, format: Format, typed: Option<&str>) -> String`

- **Does:** The refusal for a format this task cannot write; the advice names the file as typed (`c.EXT`/`v.EXT` when `--to` said the format).
- **Calls (crate):** `Format::name` (cli.rs:1870) L1972; `named` (ambiguous: `coverage_round_three_tests::coverage_round_five_tests::named` (convert/mod.rs:50395); `named` (validate.rs:748)) L1978,1983
- **Calls (std/external/unresolved):** `.map_or_else`, `.name`, `.to_string`
- **Macros:** `format!`
- **Called by:** `output_format` (cli.rs:1996)

#### `output_format`  (L1996-2035, pub)
`pub fn output_format( task: Task, output: Option<&Path>, to: Option<Format>, ) -> Result<Option<Format>, String>`

- **Does:** The format a task writes.
- **Calls (crate):** `Task::known` (cli.rs:1960) L2001; `Task::allowed` (cli.rs:1945) L2026; `Task::cannot_write` (cli.rs:1971) L2033
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.contains`, `.display`, `.file_name`, `.filter`, `.into_owned`, `.map_or_else`, `.map`, `.name`, `.or`, `.to_string_lossy`, `.to_string`, `Err`, `Ok`, `Some`
- **Macros:** `format!`
- **Called by:** `task_output` (bin/jubarte.rs:324); `run_text` (bin/jubarte.rs:1036); `run` (bin/jubarte.rs:2862); `compare_output` (cli.rs:2927); `read_output` (cli.rs:2980); `validate_matches` (cli.rs:3007); `compare_json` (cli.rs:3542); `parse_subcommand_json` (cli.rs:3779); `parse_implicit_compare_json` (cli.rs:3890)

#### `refused_input`  (L2042-2056, pub)
`pub fn refused_input(path: &Path, bytes: &[u8], word: bool) -> Option<String>`

- **Does:** The error line for an input no task reads, `CODE: FILE is …`: an OLE (`.doc`, encrypted) or RTF file, or, when it must be `word`, a file that is no ZIP package.
- **Calls (crate):** `sniff_package` (admission.rs:283) L2044; `sniff` (admission.rs:259) L2046; `.code` (ambiguous, 4 candidates) L2051
- **Calls (std/external/unresolved):** `.display`, `.err`, `.map`
- **Macros:** `format!`
- **Called by:** `read_document` (bin/jubarte.rs:3038); `admit_word` (bin/jubarte.rs:3106)

#### `wrote_line`  (L2060-2067, pub)
`pub fn wrote_line(path: &Path, format: Format, bytes: usize, pages: Option<usize>) -> String`

- **Does:** The line a writing task prints on stderr for the file it wrote: `wrote r.pdf (pdf, 48213 bytes, 3 pages)`; `pages` is a PDF's.
- **Calls (crate):** `count` (cli.rs:2149) L2061
- **Calls (std/external/unresolved):** `.display`, `.map_or_else`, `.name`
- **Macros:** `format!`
- **Called by:** `write_output` (bin/jubarte.rs:353); `run_convert` (bin/jubarte.rs:646); `run_for_edit` (bin/jubarte.rs:961); `write_edited` (bin/jubarte.rs:2216); `run_convert_any` (bin/jubarte.rs:3458); `run_validate` (bin/jubarte.rs:3812)

#### `success_block`  (L2070-2090, pub)
`pub fn success_block(before_name: &str, before: &str, after_name: &str, after: &str) -> String`

- **Does:** Confirmation of the published document, comparing the two agent views.
- **Calls (std/external/unresolved):** `.bytes`, `.context_radius`, `.count`, `.header`, `.lines`, `.map`, `.max`, `.repeat`, `.take_while`, `.to_string`, `.unified_diff`, `.unwrap_or`, `similar::TextDiff::from_lines`
- **Macros:** `format!`
- **Called by:** `print_success` (bin/jubarte.rs:1181); `success_tests::equal_views` (cli.rs:2097); `success_tests::labels_and_known_hunk` (cli.rs:2105); `success_tests::fence_exceeds_leading_tildes_in_diff_body` (cli.rs:2118); `success_tests::three_context_lines_and_missing_newline` (cli.rs:2127)

#### `wrote_png_line`  (L2140-2146, pub)
`pub fn wrote_png_line(pattern: &Path, pages: usize, dpi: f32) -> String`

- **Does:** The line for PNG pages: `wrote out/r-page-NN.png (png, 3 pages, 96 dpi)`, `pattern` naming them all.
- **Calls (crate):** `count` (cli.rs:2149) L2144
- **Calls (std/external/unresolved):** `.display`
- **Macros:** `format!`
- **Called by:** `run_convert` (bin/jubarte.rs:646); `write_edited` (bin/jubarte.rs:2216)

#### `count`  (L2149-2151)
`fn count(n: usize, word: &str) -> String`

- **Does:** `1 page`, `3 pages`.
- **Macros:** `format!`
- **Called by:** `wrote_line` (cli.rs:2060); `wrote_png_line` (cli.rs:2140); `Walker::hunk` (debug/diff.rs:1293) (?)

#### `removed_tasks`  (L2240-2244, pub(crate))
`pub(crate) fn removed_tasks<'a>( argv: impl IntoIterator<Item = &'a str>, ) -> Vec<(&'static str, &'static str)>`

- **Does:** Each [`REMOVED_TASKS`] name `argv` puts where the task goes, as [`retired_spellings`] finds them.
- **Calls (crate):** `spellings_in` (cli.rs:2402) L2243
- **Called by:** `notes` (cli.rs:2462)

#### `removed_task_note`  (L2248-2250, pub(crate))
`pub(crate) fn removed_task_note(old: &str, now: &str) -> String`

- **Does:** The note a removed task name earns: the run goes on, reading it as a file.
- **Macros:** `format!`
- **Called by:** `notes` (cli.rs:2462)

#### `value_taken`  (L2295-2302)
`fn value_taken(arg: Option<&clap::Arg>) -> Option<bool>`

- **Does:** How many values `arg` takes after it: `None` for a flag (an option `cmd` does not know, or one with `num_args = 0` such as `--delete`), else `Some(optional)`, where an optional value (`num_args = 0..=1`) is taken only when the next token is not an option, as clap does.
- **Calls (std/external/unresolved):** `.filter`, `.get_action`, `.get_num_args`, `.max_values`, `.min_values`, `.takes_values`, `Some`
- **Called by:** `spelled_options` (cli.rs:2325)

#### `long_arg`  (L2306-2310)
`fn long_arg<'c>(cmd: &'c clap::Command, name: &str) -> Option<&'c clap::Arg>`

- **Does:** The argument of `cmd` spelled `--name`, by its long name or any of its aliases, hidden ones included.
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.get_all_aliases`, `.get_arguments`, `.get_long`, `.is_some_and`, `Some`
- **Called by:** `spelled_options` (cli.rs:2325)

#### `short_arg`  (L2313-2319)
`fn short_arg(cmd: &clap::Command, short: char) -> Option<&clap::Arg>`

- **Does:** The argument of `cmd` spelled `-c`, by its short name or a short alias.
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.get_all_short_aliases`, `.get_arguments`, `.get_short`, `.is_some_and`, `Some`
- **Called by:** `spelled_options` (cli.rs:2325)

#### `spelled_options`  (L2325-2373)
`fn spelled_options(cmd: &clap::Command, tokens: &[&str]) -> (Vec<Spelled>, usize)`

- **Does:** The options of `tokens` (read with `cmd`'s arguments: which take a value) up to the first token that is not an option, and the index of that one.
- **Calls (crate):** `value_taken` (cli.rs:2295) L2344,2356; `long_arg` (cli.rs:2306) L2344; `short_arg` (cli.rs:2313) L2356
- **Calls (std/external/unresolved):** `.char_indices`, `.copied`, `.filter`, `.get`, `.is_empty`, `.is_none_or`, `.is_none`, `.len_utf8`, `.push`, `.split_once`, `.starts_with`, `.strip_prefix`, `Some`, `Vec::new`, `skip_value`
- **Macros:** `format!`
- **Called by:** `spellings_in` (cli.rs:2402); `typed_edit_protocol` (cli.rs:3370); `next_command` (cli.rs:3413); `first_positional` (cli.rs:3484); `wants_json` (cli.rs:3611)

#### `retired_spellings`  (L2382-2386, pub)
`pub fn retired_spellings<'a>( argv: impl IntoIterator<Item = &'a str>, ) -> Vec<(&'static str, &'static str)>`

- **Does:** The retired spellings in `argv` (the arguments after the program name), each once, in the order they first appear.
- **Calls (crate):** `spellings_in` (cli.rs:2402) L2385
- **Called by:** `notes` (cli.rs:2462); `Cli::try_parse_from` (cli.rs:3491); `parse_subcommand_json` (cli.rs:3779)

#### `removed_spellings`  (L2390-2394, pub)
`pub fn removed_spellings<'a>( argv: impl IntoIterator<Item = &'a str>, ) -> Vec<(&'static str, &'static str)>`

- **Does:** Each [`REMOVED`] spelling `argv` uses, as [`retired_spellings`] finds them.
- **Calls (crate):** `spellings_in` (cli.rs:2402) L2393
- **Called by:** `cli_main` (bin/jubarte.rs:3935); `parse_json` (cli.rs:3954)

#### `removed_error`  (L2398-2400, pub)
`pub fn removed_error(old: &str, now: &str) -> String`

- **Does:** The error a removed spelling ends the run with, naming what to say instead.
- **Macros:** `format!`
- **Called by:** `cli_main` (bin/jubarte.rs:3935); `parse_json` (cli.rs:3954)

#### `spellings_in`  (L2402-2444)
`fn spellings_in<'a>( table: &[Retired], argv: impl IntoIterator<Item = &'a str>, ) -> Vec<(&'static str, &'static str)>`

- **Does:** _no doc comment_
- **Calls (crate):** `spelled_options` (cli.rs:2325) L2409,2437
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.find_subcommand`, `.find`, `.get_name`, `.get`, `.insert`, `.into_iter`, `.is_empty`, `.is_none`, `.iter`, `.push`, `.retain`, `.take_while`, `.unwrap_or_default`, `Cli::command`, `Some`, `Vec::new`, `spot`, `std::collections::HashSet::new`
- **Called by:** `removed_tasks` (cli.rs:2240); `retired_spellings` (cli.rs:2382); `removed_spellings` (cli.rs:2390)

#### `apply_retired`  (L2451-2458, pub(crate))
`pub(crate) fn apply_retired(task: &mut Command, retired: &[(&str, &str)])`

- **Does:** What a retired spelling still means once parsed: `audit` with no `--rules` checks `a11y,style,structure`, as the retired task did, not `validate`'s default `word`.
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.iter`, `.map`, `.to_vec`
- **Called by:** `Cli::try_parse_from` (cli.rs:3491); `parse_subcommand_json` (cli.rs:3779)

#### `notes`  (L2462-2480, pub)
`pub fn notes(argv: &[String]) -> Vec<String>`

- **Does:** The stderr lines `argv` earns before its run or its usage error: each retired spelling's note, then each removed task's.
- **Calls (crate):** `retired_spellings` (cli.rs:2382) L2468; `retired_note` (cli.rs:2483) L2471,2472; `removed_tasks` (cli.rs:2240) L2475; `removed_task_note` (cli.rs:2248) L2477
- **Calls (std/external/unresolved):** `.any`, `.chain`, `.collect`, `.into_iter`, `.iter`, `.map`, `.starts_with`, `.take_while`, `argv`
- **Called by:** `cli_main` (bin/jubarte.rs:3935); `parse_json` (cli.rs:3954); `tests::a_plain_span_across_paragraphs_names_every_piece_s_id` (markdown/from_docx/critic.rs:2070) (?)

#### `retired_note`  (L2483-2485, pub)
`pub fn retired_note(old: &str, now: &str) -> String`

- **Does:** The stderr line a retired spelling prints once.
- **Macros:** `format!`
- **Called by:** `notes` (cli.rs:2462)

#### `<crate::markdown::TrackChanges as From>::from`  (L2502-2508, impl From)
`fn from(choice: TrackChanges) -> Self`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `<RenderArgs as Default>::default`  (L2556-2566, impl Default)
`fn default() -> Self`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `run_text` (bin/jubarte.rs:1036); `run_text_diff` (bin/jubarte.rs:3327); `run_diff` (bin/jubarte.rs:3364); `run_convert_any` (bin/jubarte.rs:3458)

#### `RenderArgs::apply`  (L2571-2582, pub)
`pub fn apply(&self, options: crate::convert::PdfOptions) -> crate::convert::PdfOptions`

- **Does:** Apply comment placement and page filtering to the renderer options.
- **Calls:** none
- **Called by:** `pdf_options` (bin/jubarte.rs:144)

#### `<crate::markdown::PageSize as From>::from`  (L2692-2697, impl From)
`fn from(choice: Page) -> Self`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `parse_timeout`  (L2748-2758)
`fn parse_timeout(value: &str) -> Result<std::time::Duration, String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_finite`, `.map_err`, `.parse`, `Err`, `std::time::Duration::try_from_secs_f64`
- **Macros:** `format!`
- **Called by:** none found in src

#### `<crate::append::SectionBreak as From>::from`  (L2774-2780, impl From)
`fn from(arg: SectionBreakArg) -> Self`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `tests::append_parses_files_output_and_break` (bin/jubarte.rs:4678)

#### `styles`  (L2783-2790)
`fn styles() -> clap::builder::Styles`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.header`, `.literal`, `.on_default`, `.placeholder`, `.usage`, `clap::builder::Styles::styled`
- **Called by:** `parse_json` (cli.rs:3954); `word_style_line_owned_boundary_tests::word_single_line_style_propagation_keeps_authored_history_and_unrelated_properties` (document_comparer.rs:12809) (?)

#### `parse_author`  (L2793-2795)
`fn parse_author(value: &str) -> Result<String, String>`

- **Does:** `--by`: a handle (`AC`, `@AC`) or a full name, never blank.
- **Calls (crate):** `author` (markdown/mod.rs:404) L2794
- **Calls (std/external/unresolved):** `.map`
- **Called by:** none found in src

#### `parse_dpi`  (L2797-2804)
`fn parse_dpi(value: &str) -> Result<f32, String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`, `.is_finite`, `.map_err`, `.parse`, `Err`, `Ok`
- **Called by:** none found in src

#### `input_format_parser`  (L2806-2815)
`fn input_format_parser() -> impl clap::builder::TypedValueParser<Value = Format>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.map`, `clap::builder::PossibleValuesParser::new`
- **Called by:** none found in src

#### `parse_context`  (L2817-2822)
`fn parse_context(value: &str) -> Result<usize, String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.map_err`, `.map`, `.parse`, `.to_string`
- **Called by:** none found in src

#### `parse_threshold`  (L2824-2831)
`fn parse_threshold(value: &str) -> Result<f64, String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`, `.into`, `.is_finite`, `.map_err`, `.parse`, `Err`, `Ok`
- **Called by:** none found in src

#### `parse_palette`  (L2833-2836)
`fn parse_palette(value: &str) -> Result<String, String>`

- **Does:** _no doc comment_
- **Calls (crate):** `RevisionPalette::parse` (convert/mod.rs:209) L2834
- **Calls (std/external/unresolved):** `.to_string`, `Ok`
- **Called by:** none found in src

#### `parse_rule`  (L2839-2850)
`fn parse_rule(value: &str) -> Result<String, String>`

- **Does:** A `--rules` item: a [`crate::validate::RULE_SETS`] name or an audit code.
- **Calls (crate):** `is_rule` (audit.rs:156) L2840
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.iter`, `.join`, `.map`, `.to_string`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** none found in src

#### `validate_pages`  (L2854-2883)
`fn validate_pages(value: &str) -> Result<String, String>`

- **Does:** Validate without expanding ranges: parsing help or arguments never allocates a potentially enormous list of page indices. _(from a code comment)_
- **Calls (crate):** `page` (ambiguous: `coverage_round_three_tests::page` (convert/mod.rs:49554); `style_hit_and_following_line_contract_tests::page` (convert/mod.rs:63843)) L2875,2879
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.into`, `.is_ascii_digit`, `.is_empty`, `.map_err`, `.parse`, `.split_once`, `.split`, `.to_string`, `.trim`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** none found in src

#### `serialize_timeout`  (L2885-2890)
`fn serialize_timeout<S: serde::Serializer>( timeout: &Option<std::time::Duration>, serializer: S, ) -> Result<S::Ok, S::Error>`

- **Does:** _no doc comment_
- **Calls (crate):** `.serialize` (ambiguous: `<LineSpacing as Serialize>::serialize` (edit.rs:846); `<Points as Serialize>::serialize` (edit.rs:869); `<HalfPoints as Serialize>::serialize` (edit.rs:960)) L2889
- **Calls (std/external/unresolved):** `.as_secs_f64`, `.map`
- **Called by:** none found in src

#### `json_not_yet`  (L2912-2921)
`fn json_not_yet(matches: &clap::ArgMatches, task: Task) -> Option<String>`

- **Does:** `diff` writes no JSON snapshot: its `-o x.json` and `--to json` are refused here, so the binary and both bindings exit 2 alike instead of writing a patch into a `.json`.
- **Calls (crate):** `Task::known` (cli.rs:1960) L2918
- **Calls (std/external/unresolved):** `.and_then`, `.copied`, `.flatten`, `.name`, `.ok`, `.then`, `.try_get_one`, `Some`
- **Macros:** `format!`
- **Called by:** `validate_matches` (cli.rs:3007)

#### `compare_output`  (L2927-2954)
`fn compare_output( output: Option<&Path>, given: &dyn Fn(&str) -> bool, ) -> Result<Option<Format>, String>`

- **Does:** What `compare`'s `-o` writes (`None`: the view is printed), refusing the options that do not go with it: read's `--json` and `--tables` (one document), the view's options with a file that is not the view, and the rendering options without a PDF or PNG.
- **Calls (crate):** `output_format` (cli.rs:1996) L2939
- **Calls (std/external/unresolved):** `.any`, `.find`, `.into_iter`, `.is_some_and`, `.replace`, `.to_string`, `Err`, `Ok`, `Some`, `given`
- **Macros:** `format!`, `matches!`
- **Called by:** `validate_matches` (cli.rs:3007)

#### `read_view_conflict`  (L2957-2962)
`fn read_view_conflict(flag: &str) -> String`

- **Does:** The diagnosis for a view-only flag beside --for-edit.
- **Macros:** `format!`
- **Called by:** `usage_error` (cli.rs:670)

#### `read_usage_error`  (L2967-2975)
`fn read_usage_error(command: &clap::Command, message: String) -> clap::Error`

- **Does:** As in the swallowed-FILE diagnosis, rebuild a task error through clap.
- **Calls (crate):** `ComparisonLog::error` (comparison_log.rs:64) (by name) L2974
- **Calls (std/external/unresolved):** `.clone`, `.find_subcommand`, `.get_name`, `.override_usage`, `.set_bin_name`, `.unwrap_or`
- **Macros:** `format!`
- **Called by:** `usage_error` (cli.rs:670); `validate_matches` (cli.rs:3007)

#### `read_output`  (L2980-2994)
`fn read_output(output: &Path, given: &dyn Fn(&str) -> bool) -> Result<Format, String>`

- **Does:** What `read -o PATH` (and the one-document shorthand's -o) writes: `.md` the view, `.json` the snapshot.
- **Calls (crate):** `output_format` (cli.rs:1996) L2981
- **Calls (std/external/unresolved):** `.to_string`, `.unwrap_or`, `Err`, `Ok`, `Some`, `given`
- **Called by:** `validate_matches` (cli.rs:3007)

#### `palette_without_custom`  (L2999-3005)
`fn palette_without_custom(args: &clap::ArgMatches) -> bool`

- **Does:** `--revision-palette` without `--revisions custom`, in any task that draws revisions (convert, edit, compare, accept, reject, append, scrub): a usage error before any document is read (Codex on #440 and #441).
- **Calls (std/external/unresolved):** `.flatten`, `.is_some`, `.ok`, `.try_get_one`, `Some`
- **Called by:** `validate_matches` (cli.rs:3007)

#### `validate_matches`  (L3007-3331)
`fn validate_matches( matches: &clap::ArgMatches, command: &mut clap::Command, ) -> Result<(), clap::Error>`

- **Does:** _no doc comment_
- **Calls (crate):** `compare_output` (cli.rs:2927) L3020,3150; `palette_without_custom` (cli.rs:2999) L3022,3230; `read_output` (cli.rs:2980) L3031,3147; `read_usage_error` (cli.rs:2967) L3032,3147; `Task::of_name` (cli.rs:1930) L3097,3156; `json_not_yet` (cli.rs:2912) L3099; `ComparisonLog::error` (comparison_log.rs:64) (by name) L3103,3113,3125,3139; `validate` (cli/edit_template.rs:101) L3131; `validate` (cli/edit_actions.rs:289) L3134; `output_format` (cli.rs:1996) L3164,3238,3302; `.ids` (ambiguous, 6 candidates) L3182; `plan_flag_conflict` (edit/record.rs:39) L3190; `names_word_document` (cli.rs:646) L3199,3206; `swallowed_file` (cli.rs:658) L3201,3208; `Format::of_path` (cli.rs:1853) L3214; `PatchFormat::is_text_view` (cli.rs:1801) (by name) L3247
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_str`, `.chain`, `.clone`, `.collect`, `.contains_id`, `.contains`, `.copied`, `.display`, `.error`, `.expect`, `.extension`, `.filter`, `.find_subcommand`, `.find`, `.flatten`, `.get_flag`, `.get_many`, `.get_name`, `.get_one`, `.into_iter`, `.is_none`, `.is_some_and`, `.iter`, `.map_err`, `.map_or_else`, `.map`, `.name`, `.ok`, `.or`, `.replace`, `.set_bin_name`, `.subcommand`, `.to_lowercase`, `.to_str`, `.to_string_lossy`, `.to_string`, `.try_contains_id`, `.try_get_one`, `.unwrap_or`, `.value_source`, `Err`, `Ok`, `Some`, `error`, `given`, `lone`
- **Macros:** `format!`, `matches!`
- **Called by:** `Cli::try_parse_from` (cli.rs:3491); `parse_json` (cli.rs:3954)

#### `edit_arguments`  (L3359-3364, pub)
`pub fn edit_arguments( command: &clap::Command, arguments: &[std::ffi::OsString], ) -> Vec<std::ffi::OsString>`

- **Does:** Normalize edit routing identically for parsing and usage-error reporting.
- **Calls (crate):** `route` (cli/edit_actions.rs:260) L3363; `hoist_quiet` (cli.rs:494) L3363
- **Called by:** `typed_edit_protocol` (cli.rs:3370); `next_command` (cli.rs:3413); `Cli::try_parse_from` (cli.rs:3491); `wants_json` (cli.rs:3611)

#### `typed_edit_protocol`  (L3370-3407, pub)
`pub fn typed_edit_protocol( arguments: &[std::ffi::OsString], ) -> Option<(String, bool, Option<String>)>`

- **Does:** Requested protocol on a recognized typed edit route, respecting values and `--`: the action name, whether the protocol is JSONL (else JSON), and the FILE operand the tokens named when they carried one before the grammar failure (used to keep the usage envelope's `source.path` honest).
- **Calls (crate):** `top_level_command` (cli.rs:856) L3373; `edit_arguments` (cli.rs:3359) L3374; `spelled_options` (cli.rs:2325) L3385,3393
- **Calls (std/external/unresolved):** `.collect`, `.filter_map`, `.find_subcommand`, `.get`, `.is_empty`, `.is_none_or`, `.is_none`, `.iter`, `.skip`, `.take_while`, `.then`, `.to_str`, `.to_string`, `.unwrap_or_default`, `Some`
- **Called by:** `usage` (bin/jubarte/edit_typed.rs:962)

#### `next_command`  (L3413-3477, pub)
`pub fn next_command(argv: &[String], force: bool) -> Vec<String>`

- **Does:** `argv` (the arguments after the program name) as a dry run's `next:` command: without its `--dry-run` flag, and with `--force` before any `--` when `force`.
- **Calls (crate):** `top_level_command` (cli.rs:856) L3414; `edit_arguments` (cli.rs:3359) L3420; `spelled_options` (cli.rs:2325) L3431
- **Calls (std/external/unresolved):** `.chain`, `.clone`, `.collect`, `.contains`, `.enumerate`, `.extend`, `.filter_map`, `.filter`, `.find_subcommand`, `.get`, `.into`, `.iter`, `.len`, `.map`, `.push`, `.skip`, `.take_while`, `.to_str`, `.to_string`, `Vec::with_capacity`, `std::collections::BTreeSet::new`, `std::iter::once`
- **Called by:** `next_command` (bin/jubarte.rs:2475)

#### `first_positional`  (L3484-3487, pub)
`pub fn first_positional(cmd: &clap::Command, tokens: &[&str]) -> Option<String>`

- **Does:** The first positional operand the tokens name for `cmd`, when they carry one before any grammar failure: options and their values are skipped through the command model's own spelling, so an option's value is never mistaken for the operand.
- **Calls (crate):** `spelled_options` (cli.rs:2325) L3485
- **Calls (std/external/unresolved):** `.get`, `.map`
- **Called by:** `usage` (bin/jubarte/edit_template.rs:292)

#### `Cli::try_parse_from`  (L3491-3531, pub)
`pub fn try_parse_from<I, T>(arguments: I) -> Result<Self, clap::Error> where I: IntoIterator<Item = T>, T: Into<std::ffi::OsString> + Clone,`

- **Does:** Parse native arguments with clap and validate cross-option constraints.
- **Calls (crate):** `top_level_command` (cli.rs:856) L3496; `edit_arguments` (cli.rs:3359) L3498; `usage_error` (cli.rs:670) L3506; `validate_matches` (cli.rs:3007) L3508; `fill_flag_operations` (cli.rs:738) L3511; `resolve_plan` (cli.rs:613) L3512; `apply_retired` (cli.rs:2451) L3518; `retired_spellings` (cli.rs:2382) L3518; `fill_task_date` (cli.rs:533) L3519; `fill_date` (cli.rs:526) L3521; `view_given` (cli.rs:3535) L3525,3527
- **Calls (std/external/unresolved):** `.as_mut`, `.clone`, `.collect`, `.into_iter`, `.into_owned`, `.iter`, `.map_err`, `.map`, `.skip`, `.subcommand`, `.to_string_lossy`, `.try_get_matches_from_mut`, `Ok`, `Self::from_arg_matches`
- **Called by:** `cli_main` (bin/jubarte.rs:3935); `tests::append_parses_files_output_and_break` (bin/jubarte.rs:4678); `tests::self_update_parses_check_yes_and_a_pinned_version` (bin/jubarte.rs:4738); `tests::convert_revisions_default_to_conventional_and_validate_the_palette` (bin/jubarte.rs:4763); `tests::job_of` (bin/jubarte.rs:4789); `tests::compare_mode_defaults_to_word_and_powertools_has_two_spellings` (bin/jubarte.rs:4876); `tests::help_and_version_are_handled_by_clap` (bin/jubarte.rs:4915); `tests::missing_inputs_are_clap_usage_errors` (bin/jubarte.rs:4924); `tests::extra_positional_and_unknown_flag_rejected_by_clap` (bin/jubarte.rs:4937); `tests::revisions_is_no_subcommand` (bin/jubarte.rs:4954); `tests::plain_compare_positionals_leave_command_none` (bin/jubarte.rs:4966); `tests::accept_subcommand_parses_file_output_and_force` (bin/jubarte.rs:4977); +12 more

#### `view_given`  (L3535-3540)
`fn view_given(matches: &clap::ArgMatches) -> bool`

- **Does:** Whether `matches` spells out any of `read`'s options.
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.try_contains_id`, `.unwrap_or`, `.value_source`, `Some`
- **Called by:** `Cli::try_parse_from` (cli.rs:3491); `parse_subcommand_json` (cli.rs:3779); `parse_implicit_compare_json` (cli.rs:3890)

#### `compare_json`  (L3542-3577)
`fn compare_json(compare: &CompareArgs) -> serde_json::Value`

- **Does:** _no doc comment_
- **Calls (crate):** `Format::of_path` (cli.rs:1853) L3553; `output_format` (cli.rs:1996) L3565
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.as_ref`, `.expect`, `.filter`, `.flatten`, `.ok`, `.or`, `Some`, `input_format`, `serde_json::to_value`
- **Macros:** `matches!`
- **Called by:** `parse_subcommand_json` (cli.rs:3779); `parse_implicit_compare_json` (cli.rs:3890)

#### `view_is_output`  (L3582-3587)
`fn view_is_output(args: &serde_json::Value) -> bool`

- **Does:** Whether compare's result is surely the agent view: printed, or a .md of documents whose names do not both say Markdown.
- **Calls (crate):** `markdown` (inspect.rs:372) (by name) L3586
- **Calls (std/external/unresolved):** `.is_null`
- **Called by:** `parse_subcommand_json` (cli.rs:3779); `parse_implicit_compare_json` (cli.rs:3890)

#### `view_not_supported`  (L3591-3596)
`fn view_not_supported(command: &mut clap::Command) -> clap::Error`

- **Does:** A comparison whose result is the agent view, in an adapter without `read`: only `-o` with a file that is not the view works there.
- **Calls (std/external/unresolved):** `.error`
- **Called by:** `parse_subcommand_json` (cli.rs:3779); `parse_implicit_compare_json` (cli.rs:3890)

#### `error_json`  (L3598-3605)
`fn error_json(error: &clap::Error) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.exit_code`, `.to_string`, `.use_stderr`
- **Macros:** `serde_json::json!`
- **Called by:** `parse_json` (cli.rs:3954)

#### `wants_json`  (L3611-3640, pub)
`pub fn wants_json(argv: &[String]) -> bool`

- **Does:** Whether a command line (without the program name) asks for JSON: a `--json` flag before any `--`.
- **Calls (crate):** `top_level_command` (cli.rs:856) L3614; `edit_arguments` (cli.rs:3359) L3618; `spelled_options` (cli.rs:2325) L3628
- **Calls (std/external/unresolved):** `.any`, `.chain`, `.collect`, `.filter_map`, `.find_subcommand`, `.get`, `.into`, `.iter`, `.map`, `.skip`, `.take_while`, `.to_str`, `std::iter::once`
- **Called by:** `cli_main` (bin/jubarte.rs:3935); `parse_json` (cli.rs:3954)

#### `json_error`  (L3644-3650, pub)
`pub fn json_error(code: &str, message: &str, usage: Option<&str>) -> String`

- **Does:** `{"error":{"code":…,"message":…,"usage":…}}` and a newline: the line an error prints on stdout under `--json`.
- **Calls (std/external/unresolved):** `.into`
- **Macros:** `format!`, `json!`, `serde_json::json!`
- **Called by:** `runtime_json_error` (cli.rs:3656); `usage_json_error` (cli.rs:3679); `removed_json_error` (cli.rs:3687)

#### `runtime_json_error`  (L3656-3671, pub)
`pub fn runtime_json_error(message: &str, exit: u8) -> String`

- **Does:** The [`json_error`] line of a runtime error that ends a run with `exit`, `message` being what follows `error: ` on stderr.
- **Calls (crate):** `json_error` (cli.rs:3644) L3670
- **Calls (std/external/unresolved):** `.all`, `.chars`, `.is_ascii_digit`, `.is_ascii_uppercase`, `.len`, `.map_or`, `.split_once`, `.starts_with`
- **Called by:** `say_error` (bin/jubarte.rs:2995)

#### `usage_json_error`  (L3679-3682, pub)
`pub fn usage_json_error(error: &clap::Error, argv: &[String], program: &str) -> Option<String>`

- **Does:** The [`json_error`] line of a usage error clap reports for `argv` (no program name) run as `program`: its message without `error: `, on one line (a list clap names under it joined on, each tip after it), and the task's usage (`jubarte read [OPTIONS] <FILE>`; clap's own usage, which names the flags given, when no task i...
- **Calls (crate):** `usage_parts` (cli.rs:3698) L3680; `json_error` (cli.rs:3644) L3681
- **Calls (std/external/unresolved):** `.as_deref`, `Some`
- **Called by:** `cli_main` (bin/jubarte.rs:3935); `parse_json` (cli.rs:3954)

#### `removed_json_error`  (L3687-3695, pub)
`pub fn removed_json_error( error: &clap::Error, argv: &[String], program: &str, message: &str, ) -> String`

- **Does:** The [`json_error`] line of a removed spelling's error, `message` being what follows `error: ` ([`removed_error`]), with the usage [`usage_json_error`] gives clap's `error` for the same `argv`.
- **Calls (crate):** `usage_parts` (cli.rs:3698) L3693; `json_error` (cli.rs:3644) L3694
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`
- **Called by:** `cli_main` (bin/jubarte.rs:3935); `parse_json` (cli.rs:3954)

#### `usage_parts`  (L3698-3751)
`fn usage_parts( error: &clap::Error, argv: &[String], program: &str, ) -> Option<(String, Option<String>)>`

- **Does:** [`usage_json_error`]'s message and usage.
- **Calls (crate):** `task_usage` (cli.rs:3755) L3749
- **Calls (std/external/unresolved):** `.is_empty`, `.join`, `.last_mut`, `.lines`, `.or_else`, `.push_str`, `.push`, `.render`, `.strip_prefix`, `.then`, `.to_string`, `.trim`, `.use_stderr`, `Some`, `String::new`, `Vec::::new`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `usage_json_error` (cli.rs:3679); `removed_json_error` (cli.rs:3687)

#### `task_usage`  (L3755-3765)
`fn task_usage(argv: &[String], program: &str) -> Option<String>`

- **Does:** The usage of the task `argv` names (`jubarte read [OPTIONS] <FILE>`), a line per form, or `None` when its first word is no task.
- **Calls (crate):** `.build` (ambiguous: `ListCopy::build` (append.rs:1117); `Selection::build` (edit/for_edit.rs:547); `Origin::build` (edit/origin.rs:260)) L3757
- **Calls (std/external/unresolved):** `.bin_name`, `.collect`, `.find_subcommand_mut`, `.find`, `.iter`, `.join`, `.lines`, `.map`, `.render_usage`, `.starts_with`, `.strip_prefix`, `.take_while`, `.to_string`, `Cli::command`, `Some`
- **Called by:** `usage_parts` (cli.rs:3698)

#### `missing_task`  (L3768-3773)
`fn missing_task(command: &mut clap::Command, program: &str, task: &str) -> clap::Error`

- **Does:** The usage error for a task the host (`program`) does not have.
- **Calls (std/external/unresolved):** `.error`
- **Macros:** `format!`
- **Called by:** `parse_implicit_compare_json` (cli.rs:3890); `parse_json` (cli.rs:3954)

#### `parse_subcommand_json`  (L3779-3888)
`fn parse_subcommand_json( command: &mut clap::Command, matches: &clap::ArgMatches, arguments: &[String], accepts: &dyn Fn(&str) -> bool, ) -> Result<serde_json::Value, clap::Error>`

- **Does:** The `{exit_code, command, args}` value for a subcommand clap matched: the task with its flag operations, plan, retired spellings and date resolved, and compare's view or read routing.
- **Calls (crate):** `fill_flag_operations` (cli.rs:738) L3786; `resolve_plan` (cli.rs:613) L3787; `apply_retired` (cli.rs:2451) L3788; `retired_spellings` (cli.rs:2382) L3790; `fill_task_date` (cli.rs:533) L3792; `view_given` (cli.rs:3535) L3796; `compare_json` (cli.rs:3542) L3798; `view_is_output` (cli.rs:3582) L3799; `view_not_supported` (cli.rs:3591) L3800; `Format::of_path` (cli.rs:1853) L3814; `output_format` (cli.rs:1996) L3820,3838,3883; `.target` (ambiguous: `EditOptions::target` (cli.rs:358); `Operation::target` (edit/v2.rs:585); `ThreadOp::target` (edit.rs:2567)) L3860; `slash_path` (cli.rs:553) L3866
- **Calls (std/external/unresolved):** `.as_deref`, `.as_path`, `.as_ref`, `.expect`, `.filter`, `.flatten`, `.into`, `.is_empty`, `.is_some`, `.iter`, `.map`, `.ok`, `.or_else`, `.or`, `.subcommand`, `.then_some`, `.unwrap_or`, `Command::from_arg_matches`, `Err`, `Ok`, `Some`, `accepts`, `input_format`, `serde_json::Value::String`, `serde_json::to_value`, `slashed`
- **Macros:** `matches!`
- **Called by:** `parse_json` (cli.rs:3954)

#### `parse_implicit_compare_json`  (L3890-3936)
`fn parse_implicit_compare_json( command: &mut clap::Command, matches: &clap::ArgMatches, model: &clap::Command, program: &str, accepts: &dyn Fn(&str) -> bool, ) -> Result<serde_json::Value, clap::Error>`

- **Does:** _no doc comment_
- **Calls (crate):** `fill_date` (cli.rs:526) L3898; `missing_task` (cli.rs:3768) L3907; `output_format` (cli.rs:1996) L3923; `view_given` (cli.rs:3535) L3930; `compare_json` (cli.rs:3542) L3931; `view_is_output` (cli.rs:3582) L3932; `view_not_supported` (cli.rs:3591) L3933
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.as_ref`, `.error`, `.expect`, `.filter`, `.find_subcommand`, `.flatten`, `.get_name`, `.into`, `.is_none`, `.map`, `.ok`, `.or`, `.to_str`, `.to_string`, `CompareArgs::from_arg_matches`, `Err`, `Ok`, `accepts`, `serde_json::to_value`
- **Macros:** `serde_json::json!`
- **Called by:** `parse_json` (cli.rs:3954)

#### `parse_json`  (L3954-4074, pub)
`pub fn parse_json(arguments: &[String], program: &str, supported: &[String]) -> String`

- **Does:** Parse arguments (excluding argv0) for native-compatible adapters.
- **Calls (crate):** `top_level_command` (cli.rs:856) L3955; `AuthorColors::color` (convert/mod.rs:15095) (by name) L3959; `styles` (cli.rs:2783) L3963; `missing_task` (cli.rs:3768) L4013; `hoist_quiet` (cli.rs:494) L4015; `route` (cli/edit_actions.rs:260) L4016; `usage_error` (cli.rs:670) L4028; `validate_matches` (cli.rs:3007) L4029; `parse_subcommand_json` (cli.rs:3779) L4032; `parse_implicit_compare_json` (cli.rs:3890) L4034; `notes` (cli.rs:2462) L4040,4064; `removed_spellings` (cli.rs:2390) L4043; `removed_error` (cli.rs:2398) L4045; `wants_json` (cli.rs:3611) L4052,4065; `removed_json_error` (cli.rs:3687) L4054; `error_json` (cli.rs:3598) L4060; `usage_json_error` (cli.rs:3679) L4066
- **Calls (std/external/unresolved):** `.about`, `.after_help`, `.and_then`, `.any`, `.arg_required_else_help`, `.args_conflicts_with_subcommands`, `.args`, `.chain`, `.clone`, `.cloned`, `.collect`, `.filter`, `.find_subcommand`, `.find`, `.first`, `.get_arguments`, `.get_flag`, `.get_groups`, `.get_name`, `.get_subcommands`, `.groups`, `.into_iter`, `.into_owned`, `.into`, `.is_empty`, `.is_global_set`, `.is_some`, `.iter`, `.join`, `.map_err`, `.map`, `.starts_with`, `.strip_prefix`, `.styles`, `.subcommand_help_heading`, `.subcommand_negates_reqs`, `.subcommand_required`, `.subcommand`, `.subcommands`, `.take_while`, `.term_width`, `.to_string_lossy`, `.to_string`, `.try_get_matches_from_mut`, `.unwrap_or_default`, `.unwrap_or`, `.version`, `Err`, `accepts`, `clap::Command::new`, `serde_json::from_str`, `std::iter::once`
- **Macros:** `env!`, `format!`, `serde_json::json!`
- **Called by:** none found in src

## Test functions (10)

- `success_tests::equal_views` (L2097): _no doc comment_
- `success_tests::labels_and_known_hunk` (L2105): _no doc comment_
- `success_tests::fence_exceeds_leading_tildes_in_diff_body` (L2118): _no doc comment_
- `success_tests::three_context_lines_and_missing_newline` (L2127): _no doc comment_
- `slash_path_tests::slash_path_uses_forward_slashes` (L4082): _no doc comment_
- `slash_path_tests::slash_path_canonical_drive_letter_spells_agent_facing` (L4090): The agent-facing contract: a canonical drive-letter path spelled `C:/…` regardless of the OS the test runs on (#1244).
- `slash_path_tests::unverbatimed_keeps_every_nonverbatim_path` (L4097): _no doc comment_
- `slash_path_tests::a_verbatim_literal_is_an_ordinary_name_off_windows` (L4107): _no doc comment_
- `slash_path_tests::unverbatimed_strips_every_verbatim_prefix_kind` (L4115): _no doc comment_
- `slash_path_tests::unverbatimed_returns_the_stripped_components` (L4130): _no doc comment_
