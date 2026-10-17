# Functions and methods in `src/cli.rs`

74 non-test, 0 test. Part of the [symbol manifest](../README.md).

#### `EditOptions::target`  (L212-230, pub)
`pub fn target(&self, file: &Path) -> EditTarget`

- **Does:** Where `edit FILE` (or `add FILE`) writes: `--out-dir [DIR]`'s bundle, else `-o`, else `<FILE's dir>/<stem>_modified.docx`.
- **Calls (std/external/unresolved):** `.as_os_str`, `.clone`, `.ends_with`, `.file_stem`, `.filter`, `.into_owned`, `.is_empty`, `.join`, `.map_or_else`, `.parent`, `.to_string_lossy`, `.to_string`, `EditTarget::Bundle`, `EditTarget::Single`, `PathBuf::from`, `beside`
- **Macros:** `format!`
- **Called by:** `run_flags` (bin/jubarte.rs:1804); `parse_json` (cli.rs:3053) (?); `Transaction::resolve_thread` (edit.rs:3342) (?); `Transaction::check_thread_ops` (edit.rs:4326) (?); `run` (update.rs:186) (?)

#### `ExistingArg::plan_value`  (L252-261, pub)
`pub fn plan_value(self) -> Option<crate::edit::ExistingRevisions>`

- **Does:** The plan's `existing_revisions`; `None` for `auto`.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `edit_plan` (bin/jubarte.rs:927); `record_plan` (bin/jubarte.rs:1887) (?)

#### `flag_operations`  (L266-318, pub)
`pub fn flag_operations(matches: &clap::ArgMatches) -> Result<Vec<FlagOp>, String>`

- **Does:** Groups the operation flags of an `edit` or `add` invocation by their `-p`, in command-line order.
- **Calls (std/external/unresolved):** `.clone`, `.extend`, `.flatten`, `.get_many`, `.indices_of`, `.is_none`, `.is_some`, `.last_mut`, `.map`, `.ok`, `.push`, `.sort_by_key`, `.try_get_many`, `.unwrap_or_default`, `.zip`, `Err`, `FlagOp::default`, `Ok`, `Some`, `Vec::new`, `twice`
- **Macros:** `format!`
- **Called by:** `fill_flag_operations` (cli.rs:504)

#### `plan_path`  (L325-336)
`fn plan_path() -> impl clap::builder::TypedValueParser<Value = PathBuf>`

- **Does:** `--plan`'s parser: a path that may be empty, the value `--plan` takes when given bare (clap's `PathBuf` parser refuses an empty one).
- **Calls (std/external/unresolved):** `.ends_with`, `.to_string_lossy`, `.try_map`, `Err`, `Ok`, `PathBuf::from`, `clap::builder::OsStringValueParser::new`
- **Macros:** `cfg!`, `format!`
- **Called by:** none found in src

#### `hoist_quiet`  (L342-370)
`fn hoist_quiet<T: AsRef<std::ffi::OsStr> + Clone>( command: &clap::Command, arguments: &[T], ) -> Vec<T>`

- **Does:** `-q`/`--quiet` before the task moves after it, so `jubarte -q validate F` reads as `jubarte validate -q F` (D6 follow-up).
- **Calls (std/external/unresolved):** `.all`, `.and_then`, `.as_ref`, `.bytes`, `.clone`, `.count`, `.extend_from_slice`, `.find_subcommand`, `.get`, `.is_none_or`, `.is_none`, `.iter`, `.len`, `.push`, `.skip`, `.starts_with`, `.take_while`, `.to_str`, `.to_vec`, `Some`, `Vec::with_capacity`, `is_quiet`
- **Macros:** `vec!`
- **Called by:** `Cli::try_parse_from` (cli.rs:2913); `parse_json` (cli.rs:3053)

#### `fill_date`  (L374-378)
`fn fill_date(date: &mut Option<String>)`

- **Does:** `compare`'s and `diff`'s missing `-d` is now, in UTC, to the second (D17): filled once here, so the binary and every host share it.
- **Calls (crate):** `utc_now_iso8601` (convert/mod.rs:12306) L376
- **Calls (std/external/unresolved):** `.is_none`, `Some`
- **Called by:** `fill_task_date` (cli.rs:381); `Cli::try_parse_from` (cli.rs:2913); `parse_json` (cli.rs:3053)

#### `fill_task_date`  (L381-387)
`fn fill_task_date(task: &mut Command)`

- **Does:** [`fill_date`] for the task that dates its revisions.
- **Calls (crate):** `fill_date` (cli.rs:374) L383,384
- **Called by:** `Cli::try_parse_from` (cli.rs:2913); `parse_json` (cli.rs:3053)

#### `slash_path`  (L396-398, pub)
`pub fn slash_path(path: &Path) -> String`

- **Does:** A path as the CLI's agent-facing outputs spell it: `/` between the parts on every OS (#568).
- **Calls (std/external/unresolved):** `.replace`, `.to_string_lossy`
- **Called by:** `Output::of_task` (bin/jubarte.rs:225); `run_edit` (bin/jubarte.rs:953); `comma_paths` (bin/jubarte.rs:1289); `write_edited` (bin/jubarte.rs:1657); `run` (bin/jubarte.rs:2095); `parse_json` (cli.rs:3053)

#### `resolve_plan`  (L401-430)
`fn resolve_plan(task: &mut Command)`

- **Does:** `--plan` without a value (D16) is `<FILE's dir>/<stem>_plan.json`.
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) (by name) L425
- **Calls (std/external/unresolved):** `.as_os_str`, `.file_stem`, `.filter`, `.into_owned`, `.is_empty`, `.join`, `.map_or_else`, `.to_string_lossy`, `.to_string`, `PathBuf::from`
- **Macros:** `format!`
- **Called by:** `Cli::try_parse_from` (cli.rs:2913); `parse_json` (cli.rs:3053)

#### `names_word_document`  (L434-442)
`fn names_word_document(path: &Path) -> bool`

- **Does:** A Word document's name, by extension: what an optional value swallowed when it should have been FILE.
- **Calls (std/external/unresolved):** `.any`, `.ends_with`, `.file_name`, `.is_some_and`, `.iter`, `.to_ascii_lowercase`, `.to_string_lossy`
- **Called by:** `swallowed_file_error` (cli.rs:457); `validate_matches` (cli.rs:2566)

#### `swallowed_file`  (L446-452)
`fn swallowed_file(flag: &str, value: &Path) -> String`

- **Does:** The message for an optional value (`--plan`, `--out-dir`) that names a Word document: FILE was read as the value.
- **Calls (std/external/unresolved):** `.display`
- **Macros:** `format!`
- **Called by:** `swallowed_file_error` (cli.rs:457); `validate_matches` (cli.rs:2566)

#### `swallowed_file_error`  (L457-500)
`fn swallowed_file_error( command: &clap::Command, tokens: &[String], error: clap::Error, ) -> clap::Error`

- **Does:** `edit --plan a.docx -p p1 …`: clap reads the document as the plan and then misses FILE.
- **Calls (crate):** `names_word_document` (cli.rs:434) L487; `swallowed_file` (cli.rs:446) L489; `ComparisonLog::error` (comparison_log.rs:64) (by name) L499
- **Calls (std/external/unresolved):** `.as_str`, `.clone`, `.filter`, `.find_subcommand`, `.get_name`, `.iter`, `.kind`, `.map`, `.position`, `.set_bin_name`, `.skip`, `.split_once`, `.starts_with`, `.take_while`, `Path::new`, `Some`
- **Macros:** `format!`, `matches!`
- **Called by:** `Cli::try_parse_from` (cli.rs:2913); `parse_json` (cli.rs:3053)

#### `fill_flag_operations`  (L504-520)
`fn fill_flag_operations( task: &mut Command, matches: &clap::ArgMatches, model: &mut clap::Command, ) -> Result<(), clap::Error>`

- **Does:** Groups the flag operations of `edit` and `add` into their `operations` field; a grouping error is a usage error.
- **Calls (crate):** `flag_operations` (cli.rs:266) L517
- **Calls (std/external/unresolved):** `.error`, `.map_err`, `.subcommand_matches`, `Ok`
- **Called by:** `Cli::try_parse_from` (cli.rs:2913); `parse_json` (cli.rs:3053)

#### `top_level_command`  (L619-632, pub)
`pub fn top_level_command() -> clap::Command`

- **Does:** The derive model with the shorthands' options hidden: `jubarte --help` lists the tasks; `jubarte compare --help` and `jubarte read --help` show every option.
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.contains`, `.get_arguments`, `.get_id`, `.hide`, `.map`, `.mut_args`, `::augment_args`, `Cli::command`, `clap::Command::new`
- **Called by:** `Cli::try_parse_from` (cli.rs:2913); `parse_json` (cli.rs:3053)

#### `<Cli as std::ops::Deref>::deref`  (L636-638, impl std::ops::Deref)
`fn deref(&self) -> &Self::Target`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `<crate::debug::Check as From>::from`  (L1414-1435, impl From)
`fn from(c: DebugCheck) -> Self`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `PatchFormat::is_text_view`  (L1462-1464, pub)
`pub fn is_text_view(self) -> bool`

- **Does:** Whether this format compares document snapshots without constructing a redline.
- **Macros:** `matches!`
- **Called by:** `cli_main` (bin/jubarte.rs:2900) (?); `validate_matches` (cli.rs:2566) (?)

#### `extension`  (L1501-1509)
`fn extension(path: &Path) -> Option<String>`

- **Does:** A file name's extension: `Path::extension`, or the whole name after its dot when the name is only an extension (`.json`, `out/.md`), which Rust reads as a hidden file with none.
- **Calls (std/external/unresolved):** `.contains`, `.extension`, `.file_name`, `.filter`, `.is_empty`, `.map`, `.strip_prefix`, `.to_str`
- **Called by:** `Format::of_path` (cli.rs:1514); `Task::known` (cli.rs:1621)

#### `Format::of_path`  (L1514-1516, pub)
`pub fn of_path(path: &Path) -> Option<Self>`

- **Does:** The format a file name says, by extension (`.json`, a name that is only an extension, says json too).
- **Calls (crate):** `Format::of_name` (cli.rs:1519) L1515; `extension` (cli.rs:1501) L1515
- **Called by:** `Format::of_input` (cli.rs:1543); `Task::known` (cli.rs:1621); `validate_matches` (cli.rs:2566); `compare_json` (cli.rs:2964); `parse_json` (cli.rs:3053)

#### `Format::of_name`  (L1519-1528, pub)
`pub fn of_name(extension: &str) -> Option<Self>`

- **Does:** The format an extension names, in any case.
- **Calls (std/external/unresolved):** `.as_str`, `.to_ascii_lowercase`, `Some`
- **Called by:** `Format::of_path` (cli.rs:1514); `Task::allowed` (cli.rs:1606)

#### `Format::name`  (L1531-1539, pub)
`pub fn name(self) -> &'static str`

- **Does:** The name `--to` takes and messages print.
- **Calls:** none
- **Called by:** `Task::cannot_write` (cli.rs:1632)

#### `Format::of_input`  (L1543-1551, pub)
`pub fn of_input(asked: Option<Self>, path: &Path, bytes: &[u8]) -> Self`

- **Does:** An input's format: the one asked for, else its extension, else its bytes (a zip is Word, anything else Markdown).
- **Calls (crate):** `Format::of_path` (cli.rs:1514) L1545
- **Calls (std/external/unresolved):** `.filter`, `.or_else`, `.starts_with`, `.unwrap_or`
- **Macros:** `matches!`
- **Called by:** `run` (bin/jubarte.rs:2095); `diff_inputs` (bin/jubarte.rs:2390); `run_convert_any` (bin/jubarte.rs:2477)

#### `Task::name`  (L1577-1588, pub)
`pub fn name(self) -> &'static str`

- **Does:** The command's name.
- **Calls:** none
- **Called by:** `Task::allowed` (cli.rs:1606)

#### `Task::of_name`  (L1591-1603, pub)
`pub fn of_name(name: &str) -> Option<Self>`

- **Does:** The task a command name (or its alias) runs.
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `validate_matches` (cli.rs:2566)

#### `Task::allowed`  (L1606-1617)
`fn allowed(self) -> Vec<Format>`

- **Does:** The formats its `-o` writes (`capabilities::OUTPUT_FORMATS`).
- **Calls (crate):** `Task::name` (cli.rs:1577) L1609; `Format::of_name` (cli.rs:1519) L1613
- **Calls (std/external/unresolved):** `.collect`, `.expect`, `.find`, `.iter`, `.map`, `.unwrap_or_default`
- **Called by:** `output_format` (cli.rs:1657)

#### `Task::known`  (L1621-1628)
`fn known(self, path: &Path) -> Option<Format>`

- **Does:** What `-o PATH`'s extension says for this task: `diff` reads `.patch` and `.diff` as text too.
- **Calls (crate):** `extension` (cli.rs:1501) L1622; `Format::of_path` (cli.rs:1514) L1627
- **Calls (std/external/unresolved):** `.eq_ignore_ascii_case`, `.is_some_and`, `Some`
- **Called by:** `output_format` (cli.rs:1657); `json_not_yet` (cli.rs:2487)

#### `Task::cannot_write`  (L1632-1649)
`fn cannot_write(self, format: Format, typed: Option<&str>) -> String`

- **Does:** The refusal for a format this task cannot write; the advice names the file as typed (`c.EXT`/`v.EXT` when `--to` said the format).
- **Calls (crate):** `Format::name` (cli.rs:1531) L1633; `coverage_round_three_tests::coverage_round_five_tests::named` (convert/mod.rs:50359) L1639,1644
- **Calls (std/external/unresolved):** `.map_or_else`, `.name`, `.to_string`
- **Macros:** `format!`
- **Called by:** `output_format` (cli.rs:1657)

#### `output_format`  (L1657-1696, pub)
`pub fn output_format( task: Task, output: Option<&Path>, to: Option<Format>, ) -> Result<Option<Format>, String>`

- **Does:** The format a task writes.
- **Calls (crate):** `Task::known` (cli.rs:1621) L1662; `Task::allowed` (cli.rs:1606) L1687; `Task::cannot_write` (cli.rs:1632) L1694
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.contains`, `.display`, `.file_name`, `.filter`, `.into_owned`, `.map_or_else`, `.map`, `.name`, `.or`, `.to_string_lossy`, `.to_string`, `Err`, `Ok`, `Some`
- **Macros:** `format!`
- **Called by:** `task_output` (bin/jubarte.rs:249); `run_text` (bin/jubarte.rs:785); `run` (bin/jubarte.rs:2095); `compare_output` (cli.rs:2502); `read_output` (cli.rs:2534); `validate_matches` (cli.rs:2566); `compare_json` (cli.rs:2964); `parse_json` (cli.rs:3053)

#### `refused_input`  (L1703-1717, pub)
`pub fn refused_input(path: &Path, bytes: &[u8], word: bool) -> Option<String>`

- **Does:** The error line for an input no task reads, `CODE: FILE is …`: an OLE (`.doc`, encrypted) or RTF file, or, when it must be `word`, a file that is no ZIP package.
- **Calls (crate):** `sniff_package` (admission.rs:240) L1705; `sniff` (admission.rs:216) L1707; `.code` (ambiguous, 4 candidates) L1712
- **Calls (std/external/unresolved):** `.display`, `.err`, `.map`
- **Macros:** `format!`
- **Called by:** `read_document` (bin/jubarte.rs:2241); `admit_word` (bin/jubarte.rs:2269)

#### `wrote_line`  (L1721-1728, pub)
`pub fn wrote_line(path: &Path, format: Format, bytes: usize, pages: Option<usize>) -> String`

- **Does:** The line a writing task prints on stderr for the file it wrote: `wrote r.pdf (pdf, 48213 bytes, 3 pages)`; `pages` is a PDF's.
- **Calls (crate):** `count` (cli.rs:1741) L1722
- **Calls (std/external/unresolved):** `.display`, `.map_or_else`, `.name`
- **Macros:** `format!`
- **Called by:** `write_output` (bin/jubarte.rs:278); `run_convert` (bin/jubarte.rs:507); `write_edited` (bin/jubarte.rs:1657); `run_validate` (bin/jubarte.rs:2793)

#### `wrote_png_line`  (L1732-1738, pub)
`pub fn wrote_png_line(pattern: &Path, pages: usize, dpi: f32) -> String`

- **Does:** The line for PNG pages: `wrote out/r-page-NN.png (png, 3 pages, 96 dpi)`, `pattern` naming them all.
- **Calls (crate):** `count` (cli.rs:1741) L1736
- **Calls (std/external/unresolved):** `.display`
- **Macros:** `format!`
- **Called by:** `run_convert` (bin/jubarte.rs:507); `write_edited` (bin/jubarte.rs:1657)

#### `count`  (L1741-1743)
`fn count(n: usize, word: &str) -> String`

- **Does:** `1 page`, `3 pages`.
- **Macros:** `format!`
- **Called by:** `wrote_line` (cli.rs:1721); `wrote_png_line` (cli.rs:1732); `Walker::hunk` (debug/diff.rs:1293) (?); `findings_text` (validate.rs:359) (?)

#### `removed_tasks`  (L1832-1836, pub)
`pub fn removed_tasks<'a>( argv: impl IntoIterator<Item = &'a str>, ) -> Vec<(&'static str, &'static str)>`

- **Does:** Each [`REMOVED_TASKS`] name `argv` puts where the task goes, as [`retired_spellings`] finds them.
- **Calls (crate):** `spellings_in` (cli.rs:1990) L1835
- **Called by:** `notes` (cli.rs:2048)

#### `removed_task_note`  (L1840-1842, pub)
`pub fn removed_task_note(old: &str, now: &str) -> String`

- **Does:** The note a removed task name earns: the run goes on, reading it as a file.
- **Macros:** `format!`
- **Called by:** `notes` (cli.rs:2048)

#### `value_taken`  (L1885-1892)
`fn value_taken(arg: Option<&clap::Arg>) -> Option<bool>`

- **Does:** How many values `arg` takes after it: `None` for a flag (an option `cmd` does not know, or one with `num_args = 0` such as `--delete`), else `Some(optional)`, where an optional value (`num_args = 0..=1`) is taken only when the next token is not an option, as clap does.
- **Calls (std/external/unresolved):** `.filter`, `.get_action`, `.get_num_args`, `.max_values`, `.min_values`, `.takes_values`, `Some`
- **Called by:** `spelled_options` (cli.rs:1915)

#### `long_arg`  (L1896-1900)
`fn long_arg<'c>(cmd: &'c clap::Command, name: &str) -> Option<&'c clap::Arg>`

- **Does:** The argument of `cmd` spelled `--name`, by its long name or any of its aliases, hidden ones included.
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.get_all_aliases`, `.get_arguments`, `.get_long`, `.is_some_and`, `Some`
- **Called by:** `spelled_options` (cli.rs:1915)

#### `short_arg`  (L1903-1909)
`fn short_arg(cmd: &clap::Command, short: char) -> Option<&clap::Arg>`

- **Does:** The argument of `cmd` spelled `-c`, by its short name or a short alias.
- **Calls (std/external/unresolved):** `.contains`, `.find`, `.get_all_short_aliases`, `.get_arguments`, `.get_short`, `.is_some_and`, `Some`
- **Called by:** `spelled_options` (cli.rs:1915)

#### `spelled_options`  (L1915-1961)
`fn spelled_options(cmd: &clap::Command, tokens: &[&str]) -> (Vec<Spelled>, usize)`

- **Does:** The options of `tokens` (read with `cmd`'s arguments: which take a value) up to the first token that is not an option, and the index of that one.
- **Calls (crate):** `value_taken` (cli.rs:1885) L1934,1944; `long_arg` (cli.rs:1896) L1934; `short_arg` (cli.rs:1903) L1944
- **Calls (std/external/unresolved):** `.char_indices`, `.copied`, `.filter`, `.get`, `.is_empty`, `.is_none_or`, `.is_none`, `.len_utf8`, `.push`, `.split_once`, `.starts_with`, `.strip_prefix`, `Some`, `Vec::new`, `skip_value`
- **Macros:** `format!`
- **Called by:** `spellings_in` (cli.rs:1990)

#### `retired_spellings`  (L1970-1974, pub)
`pub fn retired_spellings<'a>( argv: impl IntoIterator<Item = &'a str>, ) -> Vec<(&'static str, &'static str)>`

- **Does:** The retired spellings in `argv` (the arguments after the program name), each once, in the order they first appear.
- **Calls (crate):** `spellings_in` (cli.rs:1990) L1973
- **Called by:** `notes` (cli.rs:2048); `Cli::try_parse_from` (cli.rs:2913); `parse_json` (cli.rs:3053)

#### `removed_spellings`  (L1978-1982, pub)
`pub fn removed_spellings<'a>( argv: impl IntoIterator<Item = &'a str>, ) -> Vec<(&'static str, &'static str)>`

- **Does:** Each [`REMOVED`] spelling `argv` uses, as [`retired_spellings`] finds them.
- **Calls (crate):** `spellings_in` (cli.rs:1990) L1981
- **Called by:** `cli_main` (bin/jubarte.rs:2900); `parse_json` (cli.rs:3053)

#### `removed_error`  (L1986-1988, pub)
`pub fn removed_error(old: &str, now: &str) -> String`

- **Does:** The error a removed spelling ends the run with, naming what to say instead.
- **Macros:** `format!`
- **Called by:** `cli_main` (bin/jubarte.rs:2900); `parse_json` (cli.rs:3053)

#### `spellings_in`  (L1990-2032)
`fn spellings_in<'a>( table: &[Retired], argv: impl IntoIterator<Item = &'a str>, ) -> Vec<(&'static str, &'static str)>`

- **Does:** _no doc comment_
- **Calls (crate):** `spelled_options` (cli.rs:1915) L1997,2025
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.find_subcommand`, `.find`, `.get_name`, `.get`, `.insert`, `.into_iter`, `.is_empty`, `.is_none`, `.iter`, `.push`, `.retain`, `.take_while`, `.unwrap_or_default`, `Cli::command`, `Some`, `Vec::new`, `spot`, `std::collections::HashSet::new`
- **Called by:** `removed_tasks` (cli.rs:1832); `retired_spellings` (cli.rs:1970); `removed_spellings` (cli.rs:1978)

#### `apply_retired`  (L2037-2044, pub)
`pub fn apply_retired(task: &mut Command, retired: &[(&str, &str)])`

- **Does:** What a retired spelling still means once parsed: `audit` with no `--rules` checks `a11y,style,structure`, as the retired task did, not `validate`'s default `word`.
- **Calls (std/external/unresolved):** `.any`, `.is_empty`, `.iter`, `.map`, `.to_vec`
- **Called by:** `Cli::try_parse_from` (cli.rs:2913); `parse_json` (cli.rs:3053)

#### `notes`  (L2048-2066, pub)
`pub fn notes(argv: &[String]) -> Vec<String>`

- **Does:** The stderr lines `argv` earns before its run or its usage error: each retired spelling's note, then each removed task's.
- **Calls (crate):** `retired_spellings` (cli.rs:1970) L2054; `retired_note` (cli.rs:2069) L2057,2058; `removed_tasks` (cli.rs:1832) L2061; `removed_task_note` (cli.rs:1840) L2063
- **Calls (std/external/unresolved):** `.any`, `.chain`, `.collect`, `.into_iter`, `.iter`, `.map`, `.starts_with`, `.take_while`, `argv`
- **Called by:** `cli_main` (bin/jubarte.rs:2900); `parse_json` (cli.rs:3053); `tests::a_plain_span_across_paragraphs_names_every_piece_s_id` (markdown/from_docx/critic.rs:2043) (?)

#### `retired_note`  (L2069-2071, pub)
`pub fn retired_note(old: &str, now: &str) -> String`

- **Does:** The stderr line a retired spelling prints once.
- **Macros:** `format!`
- **Called by:** `notes` (cli.rs:2048)

#### `<crate::markdown::TrackChanges as From>::from`  (L2086-2092, impl From)
`fn from(choice: TrackChanges) -> Self`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `<RenderArgs as Default>::default`  (L2140-2150, impl Default)
`fn default() -> Self`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `run_text` (bin/jubarte.rs:785); `run_text_diff` (bin/jubarte.rs:2378); `run_diff` (bin/jubarte.rs:2407); `run_convert_any` (bin/jubarte.rs:2477)

#### `RenderArgs::apply`  (L2155-2166, pub)
`pub fn apply(&self, options: crate::convert::PdfOptions) -> crate::convert::PdfOptions`

- **Does:** Apply comment placement and page filtering to the renderer options.
- **Calls:** none
- **Called by:** `pdf_options` (bin/jubarte.rs:136)

#### `<crate::markdown::PageSize as From>::from`  (L2274-2279, impl From)
`fn from(choice: Page) -> Self`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `parse_timeout`  (L2323-2333)
`fn parse_timeout(value: &str) -> Result<std::time::Duration, String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.is_finite`, `.map_err`, `.parse`, `Err`, `std::time::Duration::try_from_secs_f64`
- **Macros:** `format!`
- **Called by:** none found in src

#### `<crate::append::SectionBreak as From>::from`  (L2349-2355, impl From)
`fn from(arg: SectionBreakArg) -> Self`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `tests::append_parses_files_output_and_break` (bin/jubarte.rs:3433)

#### `styles`  (L2358-2365)
`fn styles() -> clap::builder::Styles`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.header`, `.literal`, `.on_default`, `.placeholder`, `.usage`, `clap::builder::Styles::styled`
- **Called by:** `parse_json` (cli.rs:3053); `word_style_line_owned_boundary_tests::word_single_line_style_propagation_keeps_authored_history_and_unrelated_properties` (document_comparer.rs:12688) (?)

#### `parse_author`  (L2368-2370)
`fn parse_author(value: &str) -> Result<String, String>`

- **Does:** `--by`: a handle (`AC`, `@AC`) or a full name, never blank.
- **Calls (crate):** `author` (markdown/mod.rs:396) L2369
- **Calls (std/external/unresolved):** `.map`
- **Called by:** none found in src

#### `parse_dpi`  (L2372-2379)
`fn parse_dpi(value: &str) -> Result<f32, String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`, `.is_finite`, `.map_err`, `.parse`, `Err`, `Ok`
- **Called by:** none found in src

#### `input_format_parser`  (L2381-2390)
`fn input_format_parser() -> impl clap::builder::TypedValueParser<Value = Format>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.map`, `clap::builder::PossibleValuesParser::new`
- **Called by:** none found in src

#### `parse_context`  (L2392-2397)
`fn parse_context(value: &str) -> Result<usize, String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.map_err`, `.map`, `.parse`, `.to_string`
- **Called by:** none found in src

#### `parse_threshold`  (L2399-2406)
`fn parse_threshold(value: &str) -> Result<f64, String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.contains`, `.into`, `.is_finite`, `.map_err`, `.parse`, `Err`, `Ok`
- **Called by:** none found in src

#### `parse_palette`  (L2408-2411)
`fn parse_palette(value: &str) -> Result<String, String>`

- **Does:** _no doc comment_
- **Calls (crate):** `RevisionPalette::parse` (convert/mod.rs:209) L2409
- **Calls (std/external/unresolved):** `.to_string`, `Ok`
- **Called by:** none found in src

#### `parse_rule`  (L2414-2425)
`fn parse_rule(value: &str) -> Result<String, String>`

- **Does:** A `--rules` item: a [`crate::validate::RULE_SETS`] name or an audit code.
- **Calls (crate):** `is_rule` (audit.rs:156) L2415
- **Calls (std/external/unresolved):** `.collect`, `.contains`, `.iter`, `.join`, `.map`, `.to_string`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** none found in src

#### `validate_pages`  (L2429-2458)
`fn validate_pages(value: &str) -> Result<String, String>`

- **Does:** Validate without expanding ranges: parsing help or arguments never allocates a potentially enormous list of page indices. _(from a code comment)_
- **Calls (crate):** `page` (ambiguous: `coverage_round_three_tests::page` (convert/mod.rs:49518); `style_hit_and_following_line_contract_tests::page` (convert/mod.rs:63807)) L2450,2454
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.into`, `.is_ascii_digit`, `.is_empty`, `.map_err`, `.parse`, `.split_once`, `.split`, `.to_string`, `.trim`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** none found in src

#### `serialize_timeout`  (L2460-2465)
`fn serialize_timeout<S: serde::Serializer>( timeout: &Option<std::time::Duration>, serializer: S, ) -> Result<S::Ok, S::Error>`

- **Does:** _no doc comment_
- **Calls (crate):** `.serialize` (ambiguous: `<LineSpacing as Serialize>::serialize` (edit.rs:666); `<Points as Serialize>::serialize` (edit.rs:689); `<HalfPoints as Serialize>::serialize` (edit.rs:780)) L2464
- **Calls (std/external/unresolved):** `.as_secs_f64`, `.map`
- **Called by:** none found in src

#### `json_not_yet`  (L2487-2496)
`fn json_not_yet(matches: &clap::ArgMatches, task: Task) -> Option<String>`

- **Does:** `diff` writes no JSON snapshot: its `-o x.json` and `--to json` are refused here, so the binary and both bindings exit 2 alike instead of writing a patch into a `.json`.
- **Calls (crate):** `Task::known` (cli.rs:1621) L2493
- **Calls (std/external/unresolved):** `.and_then`, `.copied`, `.flatten`, `.name`, `.ok`, `.then`, `.try_get_one`, `Some`
- **Macros:** `format!`
- **Called by:** `validate_matches` (cli.rs:2566)

#### `compare_output`  (L2502-2529)
`fn compare_output( output: Option<&Path>, given: &dyn Fn(&str) -> bool, ) -> Result<Option<Format>, String>`

- **Does:** What `compare`'s `-o` writes (`None`: the view is printed), refusing the options that do not go with it: read's `--json` and `--tables` (one document), the view's options with a file that is not the view, and the rendering options without a PDF or PNG.
- **Calls (crate):** `output_format` (cli.rs:1657) L2514
- **Calls (std/external/unresolved):** `.any`, `.find`, `.into_iter`, `.is_some_and`, `.replace`, `.to_string`, `Err`, `Ok`, `Some`, `given`
- **Macros:** `format!`, `matches!`
- **Called by:** `validate_matches` (cli.rs:2566)

#### `read_output`  (L2534-2553)
`fn read_output(output: &Path, given: &dyn Fn(&str) -> bool) -> Result<Format, String>`

- **Does:** What `read -o PATH` (and the one-document shorthand's -o) writes: `.md` the view, `.json` the snapshot.
- **Calls (crate):** `output_format` (cli.rs:1657) L2535
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.replace`, `.to_string`, `.unwrap_or`, `Err`, `Ok`, `Some`, `given`
- **Macros:** `format!`
- **Called by:** `validate_matches` (cli.rs:2566)

#### `palette_without_custom`  (L2558-2564)
`fn palette_without_custom(args: &clap::ArgMatches) -> bool`

- **Does:** `--revision-palette` without `--revisions custom`, in any task that draws revisions (convert, edit, compare, accept, reject, append, scrub): a usage error before any document is read (Codex on #440 and #441).
- **Calls (std/external/unresolved):** `.flatten`, `.is_some`, `.ok`, `.try_get_one`, `Some`
- **Called by:** `validate_matches` (cli.rs:2566)

#### `validate_matches`  (L2566-2884)
`fn validate_matches( matches: &clap::ArgMatches, command: &mut clap::Command, ) -> Result<(), clap::Error>`

- **Does:** _no doc comment_
- **Calls (crate):** `compare_output` (cli.rs:2502) L2579,2703; `palette_without_custom` (cli.rs:2558) L2581,2783; `read_output` (cli.rs:2534) L2590,2700; `Task::of_name` (cli.rs:1591) L2656,2709; `json_not_yet` (cli.rs:2487) L2658; `ComparisonLog::error` (comparison_log.rs:64) (by name) L2662,2672,2684,2692; `output_format` (cli.rs:1657) L2717,2791,2855; `.ids` (ambiguous: `ChangeFilter::ids` (changes.rs:117); `Origin::ids` (edit/origin.rs:397); `Transaction::ids` (edit.rs:3763)) L2735; `plan_flag_conflict` (edit/record.rs:39) L2743; `names_word_document` (cli.rs:434) L2752,2759; `swallowed_file` (cli.rs:446) L2754,2761; `Format::of_path` (cli.rs:1514) L2767; `PatchFormat::is_text_view` (cli.rs:1462) (by name) L2800
- **Calls (std/external/unresolved):** `.and_then`, `.any`, `.as_str`, `.chain`, `.clone`, `.collect`, `.contains_id`, `.contains`, `.copied`, `.display`, `.error`, `.expect`, `.extension`, `.filter`, `.find_subcommand`, `.find`, `.flatten`, `.get_flag`, `.get_many`, `.get_name`, `.get_one`, `.into_iter`, `.is_none`, `.is_some_and`, `.iter`, `.map_err`, `.map_or_else`, `.map`, `.name`, `.ok`, `.or`, `.replace`, `.set_bin_name`, `.subcommand`, `.to_lowercase`, `.to_str`, `.to_string_lossy`, `.to_string`, `.try_contains_id`, `.try_get_one`, `.unwrap_or`, `.value_source`, `Err`, `Ok`, `Some`, `error`, `given`, `lone`
- **Macros:** `format!`, `matches!`
- **Called by:** `Cli::try_parse_from` (cli.rs:2913); `parse_json` (cli.rs:3053)

#### `Cli::try_parse_from`  (L2913-2953, pub)
`pub fn try_parse_from<I, T>(arguments: I) -> Result<Self, clap::Error> where I: IntoIterator<Item = T>, T: Into<std::ffi::OsString> + Clone,`

- **Does:** Parse native arguments with clap and validate cross-option constraints.
- **Calls (crate):** `top_level_command` (cli.rs:619) L2918; `hoist_quiet` (cli.rs:342) L2920; `swallowed_file_error` (cli.rs:457) L2928; `validate_matches` (cli.rs:2566) L2930; `fill_flag_operations` (cli.rs:504) L2933; `resolve_plan` (cli.rs:401) L2934; `apply_retired` (cli.rs:2037) L2940; `retired_spellings` (cli.rs:1970) L2940; `fill_task_date` (cli.rs:381) L2941; `fill_date` (cli.rs:374) L2943; `view_given` (cli.rs:2957) L2947,2949
- **Calls (std/external/unresolved):** `.as_mut`, `.clone`, `.collect`, `.into_iter`, `.into_owned`, `.iter`, `.map_err`, `.map`, `.skip`, `.subcommand`, `.to_string_lossy`, `.try_get_matches_from_mut`, `Ok`, `Self::from_arg_matches`
- **Called by:** `cli_main` (bin/jubarte.rs:2900); `tests::append_parses_files_output_and_break` (bin/jubarte.rs:3433); `tests::self_update_parses_check_yes_and_a_pinned_version` (bin/jubarte.rs:3493); `tests::convert_revisions_default_to_conventional_and_validate_the_palette` (bin/jubarte.rs:3518); `tests::job_of` (bin/jubarte.rs:3544); `tests::compare_mode_defaults_to_word_and_powertools_has_two_spellings` (bin/jubarte.rs:3631); `tests::help_and_version_are_handled_by_clap` (bin/jubarte.rs:3670); `tests::missing_inputs_are_clap_usage_errors` (bin/jubarte.rs:3679); `tests::extra_positional_and_unknown_flag_rejected_by_clap` (bin/jubarte.rs:3692); `tests::revisions_is_no_subcommand` (bin/jubarte.rs:3709); `tests::plain_compare_positionals_leave_command_none` (bin/jubarte.rs:3721); `tests::accept_subcommand_parses_file_output_and_force` (bin/jubarte.rs:3732); +11 more

#### `view_given`  (L2957-2962)
`fn view_given(matches: &clap::ArgMatches) -> bool`

- **Does:** Whether `matches` spells out any of `read`'s options.
- **Calls (std/external/unresolved):** `.any`, `.into_iter`, `.try_contains_id`, `.unwrap_or`, `.value_source`, `Some`
- **Called by:** `Cli::try_parse_from` (cli.rs:2913); `parse_json` (cli.rs:3053)

#### `compare_json`  (L2964-2999)
`fn compare_json(compare: &CompareArgs) -> serde_json::Value`

- **Does:** _no doc comment_
- **Calls (crate):** `Format::of_path` (cli.rs:1514) L2975; `output_format` (cli.rs:1657) L2987
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.as_ref`, `.expect`, `.filter`, `.flatten`, `.ok`, `.or`, `Some`, `input_format`, `serde_json::to_value`
- **Macros:** `matches!`
- **Called by:** `parse_json` (cli.rs:3053)

#### `view_is_output`  (L3004-3009)
`fn view_is_output(args: &serde_json::Value) -> bool`

- **Does:** Whether compare's result is surely the agent view: printed, or a .md of documents whose names do not both say Markdown.
- **Calls (crate):** `markdown` (inspect.rs:331) (by name) L3008
- **Calls (std/external/unresolved):** `.is_null`
- **Called by:** `parse_json` (cli.rs:3053)

#### `view_not_supported`  (L3013-3018)
`fn view_not_supported(command: &mut clap::Command) -> clap::Error`

- **Does:** A comparison whose result is the agent view, in an adapter without `read`: only `-o` with a file that is not the view works there.
- **Calls (std/external/unresolved):** `.error`
- **Called by:** `parse_json` (cli.rs:3053)

#### `error_json`  (L3020-3027)
`fn error_json(error: &clap::Error) -> String`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.exit_code`, `.to_string`, `.use_stderr`
- **Macros:** `serde_json::json!`
- **Called by:** `parse_json` (cli.rs:3053)

#### `missing_task`  (L3030-3035)
`fn missing_task(command: &mut clap::Command, program: &str, task: &str) -> clap::Error`

- **Does:** The usage error for a task the host (`program`) does not have.
- **Calls (std/external/unresolved):** `.error`
- **Macros:** `format!`
- **Called by:** `parse_json` (cli.rs:3053)

#### `parse_json`  (L3053-3300, pub)
`pub fn parse_json(arguments: &[String], program: &str, supported: &[String]) -> String`

- **Does:** Parse arguments (excluding argv0) for native-compatible adapters.
- **Calls (crate):** `top_level_command` (cli.rs:619) L3054; `AuthorColors::color` (convert/mod.rs:15059) (by name) L3058; `styles` (cli.rs:2358) L3062; `missing_task` (cli.rs:3030) L3112,3243; `hoist_quiet` (cli.rs:342) L3114; `swallowed_file_error` (cli.rs:457) L3117; `validate_matches` (cli.rs:2566) L3118; `fill_flag_operations` (cli.rs:504) L3122; `resolve_plan` (cli.rs:401) L3123; `apply_retired` (cli.rs:2037) L3124; `retired_spellings` (cli.rs:1970) L3126; `fill_task_date` (cli.rs:381) L3128; `view_given` (cli.rs:2957) L3132,3268; `compare_json` (cli.rs:2964) L3134,3269; `view_is_output` (cli.rs:3004) L3135,3270; `view_not_supported` (cli.rs:3013) L3136,3271; `Format::of_path` (cli.rs:1514) L3150; `output_format` (cli.rs:1657) L3158,3177,3227,3261; `.target` (ambiguous: `EditOptions::target` (cli.rs:212); `Operation::target` (edit/v2.rs:573); `ThreadOp::target` (edit.rs:1906)) L3202; `slash_path` (cli.rs:396) L3209; `fill_date` (cli.rs:374) L3234; `ComparisonLog::error` (comparison_log.rs:64) (by name) L3247; `notes` (cli.rs:2048) L3279,3295; `removed_spellings` (cli.rs:1978) L3282; `removed_error` (cli.rs:1986) L3286; `error_json` (cli.rs:3020) L3291
- **Calls (std/external/unresolved):** `.about`, `.after_help`, `.and_then`, `.any`, `.arg_required_else_help`, `.args_conflicts_with_subcommands`, `.args`, `.as_deref`, `.as_path`, `.as_ref`, `.chain`, `.clone`, `.cloned`, `.collect`, `.expect`, `.filter`, `.find_subcommand`, `.find`, `.first`, `.flatten`, `.get_arguments`, `.get_flag`, `.get_groups`, `.get_name`, `.get_subcommands`, `.groups`, `.into_iter`, `.into`, `.is_empty`, `.is_global_set`, `.is_none`, `.is_some`, `.iter`, `.join`, `.map_err`, `.map`, `.ok`, `.or_else`, `.or`, `.starts_with`, `.styles`, `.subcommand_help_heading`, `.subcommand_negates_reqs`, `.subcommand_required`, `.subcommand`, `.subcommands`, `.take_while`, `.term_width`, `.then_some`, `.to_str`, `.to_string`, `.try_get_matches_from_mut`, `.unwrap_or_default`, `.unwrap_or`, `.version`, `Command::from_arg_matches`, `CompareArgs::from_arg_matches`, `Err`, `Ok`, `Some`, `accepts`, `clap::Command::new`, `input_format`, `serde_json::Value::String`, `serde_json::from_str`, `serde_json::to_value`, `slashed`, `std::iter::once`
- **Macros:** `env!`, `format!`, `matches!`, `serde_json::json!`
- **Called by:** none found in src
