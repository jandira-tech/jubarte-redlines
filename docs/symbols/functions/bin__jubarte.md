# Functions and methods in `src/bin/jubarte.rs`

92 non-test, 48 test. Part of the [symbol manifest](../README.md).

#### `stdout_line`  (L41-50)
`fn stdout_line(line: std::fmt::Arguments<'_>)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.kind`, `.lock`, `std::io::stdout`, `std::process::exit`
- **Macros:** `eprintln!`, `writeln!`
- **Called by:** none found in src

#### `ScrubOptions::options`  (L63-63, trait decl, no body)
`fn options(&self) -> jubarte::scrub::ScrubOptions`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `cli_main` (bin/jubarte.rs:2900) (?); `tests::scrub_flags_select_what_goes_and_none_selects_everything` (bin/jubarte.rs:3760) (?)

#### `<ScrubSelection as ScrubOptions>::options`  (L67-77, impl ScrubOptions)
`fn options(&self) -> jubarte::scrub::ScrubOptions`

- **Does:** _no doc comment_
- **Calls (crate):** `<ScrubOptions as Default>::default` (scrub.rs:69) L69
- **Calls (std/external/unresolved):** `.clone`, `Self::default`
- **Called by:** `cli_main` (bin/jubarte.rs:2900) (?); `tests::scrub_flags_select_what_goes_and_none_selects_everything` (bin/jubarte.rs:3760) (?)

#### `SelectionFilter::filter`  (L81-81, trait decl, no body)
`fn filter(&self) -> jubarte::changes::ChangeFilter`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `<Selection as SelectionFilter>::filter`  (L85-103, impl SelectionFilter)
`fn filter(&self) -> jubarte::changes::ChangeFilter`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.is_empty`, `.iter`, `.map`, `.then`, `.to_vec`, `given`
- **Called by:** none found in src

#### `run_scrub`  (L106-116)
`fn run_scrub( file: &Path, out: &Output<'_>, options: &jubarte::scrub::ScrubOptions, ) -> Result<(), ConvertFailure>`

- **Does:** _no doc comment_
- **Calls (crate):** `check_output` (bin/jubarte.rs:270) L111; `read_word` (bin/jubarte.rs:2262) L112; `scrub` (scrub.rs:109) L114; `write_output` (bin/jubarte.rs:278) L115
- **Calls (std/external/unresolved):** `.map_err`, `Produced::Docx`
- **Macros:** `format!`
- **Called by:** `cli_main` (bin/jubarte.rs:2900)

#### `revision_style`  (L119-133)
`fn revision_style( revisions: Revisions, palette: Option<&str>, ) -> Result<jubarte::convert::RevisionStyle, String>`

- **Does:** The library's revision style for `--revisions` / `--revision-palette`.
- **Calls (crate):** `RevisionPalette::parse` (convert/mod.rs:209) L126
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`, `Err`, `Ok`, `RevisionStyle::Custom`
- **Macros:** `format!`
- **Called by:** `pdf_options` (bin/jubarte.rs:136); `cli_main` (bin/jubarte.rs:2900); `tests::convert_revisions_default_to_conventional_and_validate_the_palette` (bin/jubarte.rs:3518); `tests::regression_tests::invalid_revision_flags_are_rejected_with_option_context` (bin/jubarte.rs:4189)

#### `pdf_options`  (L136-145)
`fn pdf_options( render: &RenderArgs, compress: bool, ) -> Result<jubarte::convert::PdfOptions, String>`

- **Does:** The renderer options a [`RenderArgs`] group asks for.
- **Calls (crate):** `RenderArgs::apply` (cli.rs:2155) L140; `revision_style` (bin/jubarte.rs:119) L142
- **Calls (std/external/unresolved):** `.as_deref`, `Ok`, `jubarte::convert::PdfOptions::default`
- **Called by:** `write_output` (bin/jubarte.rs:278); `cli_main` (bin/jubarte.rs:2900); `tests::regression_tests::convert_without_revision_flags_uses_conventional_marks` (bin/jubarte.rs:4151); `tests::regression_tests::custom_revision_flags_reach_the_palette_parser` (bin/jubarte.rs:4165)

#### `same_path`  (L155-183)
`fn same_path(a: &Path, b: &Path) -> bool`

- **Does:** Whether two CLI paths name the same file.
- **Calls (crate):** `same_path::{fn key}` (bin/jubarte.rs:159) L182
- **Calls (std/external/unresolved):** `Some`, `same_file::is_same_file`
- **Macros:** `matches!`
- **Called by:** `same_dir` (bin/jubarte.rs:187); `check_side_file` (bin/jubarte.rs:340); `run_convert` (bin/jubarte.rs:507); `refuse_output_alias` (bin/jubarte.rs:1203)

#### `same_path::{fn key}`  (L159-181)
`fn key(p: &Path) -> Option<PathBuf>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_os_str`, `.file_name`, `.is_empty`, `.is_relative`, `.join`, `.ok`, `.parent`, `.to_lowercase`, `.to_path_buf`, `.to_string_lossy`, `PathBuf::from`, `Some`, `std::fs::canonicalize`, `std::fs::read_link`
- **Called by:** `same_path` (bin/jubarte.rs:155); `fold_twins` (changes.rs:568) (?); `step_h` (comparer/lcs.rs:3621) (?); `has_common_run_ge` (comparer/lcs.rs:5736) (?); `group_by_key_stable` (comparer/produce.rs:692) (?)

#### `same_dir`  (L187-192)
`fn same_dir(a: &Path, b: &Path) -> bool`

- **Does:** Two directory paths name the same existing directory.
- **Calls (crate):** `same_path` (bin/jubarte.rs:155) L190
- **Calls (std/external/unresolved):** `std::fs::canonicalize`
- **Called by:** `run_edit` (bin/jubarte.rs:953)

#### `ensure_writable`  (L195-203)
`fn ensure_writable(output: &Path, force: bool) -> Result<(), String>`

- **Does:** No-clobber contract shared by every writing subcommand.
- **Calls (std/external/unresolved):** `.display`, `.exists`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `check_output` (bin/jubarte.rs:270); `write_output` (bin/jubarte.rs:278); `check_side_file` (bin/jubarte.rs:340); `run_convert` (bin/jubarte.rs:507); `run_diff_render` (bin/jubarte.rs:680); `run_convert_any` (bin/jubarte.rs:2477)

#### `Output::of_task`  (L225-245)
`fn of_task( kind: Format, path: &'a Path, input: &'a Path, render: &'a RenderArgs, force: bool, quiet: bool, ) -> Self`

- **Does:** The `-o` of accept, reject, append, scrub and convert: the view's header names the Word file the task did not write.
- **Calls (crate):** `slash_path` (cli.rs:396) L242
- **Calls (std/external/unresolved):** `.with_extension`, `std::sync::LazyLock::new`
- **Macros:** `format!`
- **Called by:** `task_output` (bin/jubarte.rs:249); `diff_output` (bin/jubarte.rs:2353); `run_convert_any` (bin/jubarte.rs:2477)

#### `task_output`  (L249-259)
`fn task_output<'a>( task: Task, path: &'a Path, input: &'a Path, render: &'a RenderArgs, force: bool, quiet: bool, ) -> Result<Output<'a>, String>`

- **Does:** The [`Output`] of a task whose `-o` the parser checked.
- **Calls (crate):** `output_format` (cli.rs:1657) L257; `Output::of_task` (bin/jubarte.rs:225) L258
- **Calls (std/external/unresolved):** `.unwrap_or`, `Ok`, `Some`
- **Called by:** `cli_main` (bin/jubarte.rs:2900)

#### `check_output`  (L270-275)
`fn check_output(out: &Output<'_>) -> Result<(), String>`

- **Does:** Refuse an existing output before any work: PNG pages are checked once their names are known.
- **Calls (crate):** `ensure_writable` (bin/jubarte.rs:195) L274
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `run_scrub` (bin/jubarte.rs:106); `run_resolution` (bin/jubarte.rs:361); `run_text` (bin/jubarte.rs:785); `run` (bin/jubarte.rs:2095); `diff_output` (bin/jubarte.rs:2353); `run_convert_any` (bin/jubarte.rs:2477); `run_append` (bin/jubarte.rs:3357)

#### `write_output`  (L278-335)
`fn write_output(out: &Output<'_>, produced: Produced<'_>) -> Result<(), ConvertFailure>`

- **Does:** Writes `produced` as `out.kind` and says `wrote …` on stderr unless `-q`.
- **Calls (crate):** `pdf_options` (bin/jubarte.rs:136) L281; `run_convert` (bin/jubarte.rs:507) L283; `ensure_writable` (bin/jubarte.rs:195) L299,329; `wrote_line` (cli.rs:1721) L303,332; `agent_view` (bin/jubarte.rs:844) L309; `read_snapshot` (markdown/mod.rs:798) L312; `selection` (bin/jubarte.rs:832) L315; `<ReadOptions as Default>::default` (markdown/mod.rs:695) L316
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.display`, `.into`, `.len`, `.map_err`, `.map`, `.name`, `.to_string`, `.transpose`, `Err`, `Ok`, `Some`, `serde_json::to_string`, `std::fs::write`
- **Macros:** `eprintln!`, `format!`
- **Called by:** `run_scrub` (bin/jubarte.rs:106); `run_resolution` (bin/jubarte.rs:361); `run_text` (bin/jubarte.rs:785); `run` (bin/jubarte.rs:2095); `print_or_write` (bin/jubarte.rs:2369); `run_convert_any` (bin/jubarte.rs:2477); `run_append` (bin/jubarte.rs:3357)

#### `check_side_file`  (L340-355)
`fn check_side_file( side: &Path, flag: &str, others: [(&Path, &str); 2], force: bool, ) -> Result<(), String>`

- **Does:** A side file (`--report`, `--font-report`) is written apart from the output, after the input is read, so sharing a path with either would silently replace one with the other; `--force` never allows that.
- **Calls (crate):** `same_path` (bin/jubarte.rs:155) L347; `ensure_writable` (bin/jubarte.rs:195) L354
- **Calls (std/external/unresolved):** `.display`, `Err`
- **Macros:** `format!`
- **Called by:** `run_convert` (bin/jubarte.rs:507); `run_convert_any` (bin/jubarte.rs:2477)

#### `run_resolution`  (L361-397)
`fn run_resolution( file: &Path, out: &Output<'_>, filter: &jubarte::changes::ChangeFilter, apply: fn( &[u8], &jubarte::changes::ChangeFilter, ) -> Result<Vec<u8>, jubarte::changes::ChangeError>, what: &str, ) -> ExitCode`

- **Does:** Shared body for `accept` / `reject`: read the redline, apply the package-wide resolution, and write the result under the compare path's no-clobber contract.
- **Calls (crate):** `ChangeFilter::normalized` (changes.rs:130) L376; `check_output` (bin/jubarte.rs:270) L379; `read_word` (bin/jubarte.rs:2262) L380; `apply` (ambiguous: `tests::apply` (edit/rewrite.rs:164); `apply` (markdown/from_docx/header.rs:744); `apply` (markdown/view_styles.rs:497)) L381; `write_output` (bin/jubarte.rs:278) L387; `run` (bin/jubarte.rs:2095) L390
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`, `ExitCode::from`, `Produced::Docx`, `failed`
- **Macros:** `eprintln!`, `format!`
- **Called by:** `cli_main` (bin/jubarte.rs:2900)

#### `run_changes`  (L399-413)
`fn run_changes(file: &Path, json: bool) -> Result<(), String>`

- **Does:** _no doc comment_
- **Calls (crate):** `read_word` (bin/jubarte.rs:2262) L400; `list_changes` (changes.rs:287) L401; `changes_text` (changes.rs:259) L405
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`, `.trim_end_matches`, `Ok`, `serde_json::to_string`
- **Macros:** `outln!`
- **Called by:** `cli_main` (bin/jubarte.rs:2900)

#### `run_comments`  (L415-430)
`fn run_comments(file: &Path, json: bool, author: Option<&str>, latest: bool) -> Result<(), String>`

- **Does:** _no doc comment_
- **Calls (crate):** `read_word` (bin/jubarte.rs:2262) L416; `list_comments` (comments.rs:130) L417; `select_comments` (comments.rs:188) L418; `comments_text` (comments.rs:162) L422
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`, `.trim_end_matches`, `Ok`, `serde_json::to_string`
- **Macros:** `outln!`
- **Called by:** `cli_main` (bin/jubarte.rs:2900)

#### `arm_timeout`  (L460-469)
`fn arm_timeout(limit: std::time::Duration)`

- **Does:** Exit [`EXIT_TIMEOUT`] once `limit` has passed, whatever the main thread is doing (layout of a pathological document cannot be interrupted).
- **Calls (std/external/unresolved):** `.as_secs_f64`, `std::process::exit`, `std::thread::sleep`, `std::thread::spawn`
- **Macros:** `eprintln!`
- **Called by:** `cli_main` (bin/jubarte.rs:2900)

#### `<ConvertFailure as From>::from`  (L483-485, impl From)
`fn from(message: String) -> Self`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `tests::a_plain_convert_error_keeps_exit_1` (bin/jubarte.rs:4104) (?)

#### `<ConvertFailure as From>::from`  (L489-491, impl From)
`fn from(message: &str) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`, `.to_string`
- **Called by:** `tests::a_plain_convert_error_keeps_exit_1` (bin/jubarte.rs:4104) (?)

#### `convert_exit_code`  (L495-504)
`fn convert_exit_code(r: Result<(), ConvertFailure>) -> ExitCode`

- **Does:** The exit status of a `convert` run: 1 for an error, or its own code.
- **Calls (crate):** `exit_code` (bin/jubarte.rs:2196) L498
- **Calls (std/external/unresolved):** `Err`, `ExitCode::from`
- **Macros:** `eprintln!`
- **Called by:** `cli_main` (bin/jubarte.rs:2900)

#### `run_convert`  (L507-629)
`fn run_convert(job: &ConvertJob<'_>) -> Result<(), ConvertFailure>`

- **Does:** Render `job` as a PDF, or as PNG pages when it asks for PNG.
- **Calls (crate):** `check_side_file` (bin/jubarte.rs:340) L518; `ensure_writable` (bin/jubarte.rs:195) L522,561; `Dom::parent` (xmllinq/mod.rs:597) (by name) L528; `read_word` (bin/jubarte.rs:2262) L531; `render` (convert/mod.rs:592) L533; `png_name` (bin/jubarte.rs:770) L553; `same_path` (bin/jubarte.rs:155) L565; `wrote_line` (cli.rs:1721) L579; `wrote_png_line` (cli.rs:1732) L589; `.to_json` (ambiguous, 5 candidates) L597; `FontReportEntry::substituted` (convert/font.rs:350) (by name) L605
- **Calls (std/external/unresolved):** `.as_path`, `.collect`, `.display`, `.file_stem`, `.filter`, `.into_owned`, `.into`, `.is_empty`, `.iter`, `.join`, `.len`, `.map_err`, `.map`, `.take`, `.then_some`, `.to_string_lossy`, `.to_string`, `.to_vec`, `.unwrap_or_default`, `.unwrap_or_else`, `.with_extension`, `.zip`, `Err`, `Ok`, `Some`, `jubarte::convert::font_report_json`, `std::fs::write`
- **Macros:** `eprintln!`, `format!`
- **Called by:** `write_output` (bin/jubarte.rs:278); `run_convert_any` (bin/jubarte.rs:2477); `tests::convert_rejects_a_font_report_path_equal_to_the_pdf` (bin/jubarte.rs:3949); `tests::fail_on_substitution_is_exit_4_after_the_outputs` (bin/jubarte.rs:4070); `tests::convert_font_report_writes_json` (bin/jubarte.rs:4110); `tests::convert_pages_writes_the_selected_pages_named_by_page_number` (bin/jubarte.rs:4340); `tests::convert_pages_needs_png_and_an_existing_page` (bin/jubarte.rs:4364)

#### `parse_pages`  (L633-666)
`fn parse_pages(spec: &str) -> Result<Vec<usize>, String>`

- **Does:** `--pages` as zero-based indices, ascending and without repeats: `1-3,7` is `[0, 1, 2, 6]`.
- **Calls (crate):** `page` (ambiguous: `coverage_round_three_tests::page` (convert/mod.rs:49518); `style_hit_and_following_line_contract_tests::page` (convert/mod.rs:63807)) L651,660
- **Calls (std/external/unresolved):** `.dedup`, `.extend`, `.is_empty`, `.parse`, `.push`, `.sort_unstable`, `.split_once`, `.split`, `.trim`, `Err`, `Ok`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `tests::parse_pages_counts_from_one_and_sorts_without_repeats` (bin/jubarte.rs:4232); `tests::parse_pages_refuses_zero_empty_backwards_and_words` (bin/jubarte.rs:4240)

#### `run_diff_render`  (L680-766)
`fn run_diff_render(job: &DiffRenderJob<'_>) -> Result<bool, String>`

- **Does:** Compare two documents page by page; `Ok(true)` when any page differs.
- **Calls (crate):** `read_word` (bin/jubarte.rs:2262) L681,682; `jubarte::convert::DiffOptions::default` (ambiguous: `<DiffOptions as Default>::default` (convert/diff_render.rs:44); `<DiffOptions as Default>::default` (debug/diff.rs:32)) L686; `.differs` (ambiguous: `PageDiff::differs` (convert/diff_render.rs:73); `RenderDiff::differs` (convert/diff_render.rs:101)) L691; `png_name` (bin/jubarte.rs:770) L726; `ensure_writable` (bin/jubarte.rs:195) L734
- **Calls (std/external/unresolved):** `.as_bytes`, `.as_ref`, `.collect`, `.display`, `.filter`, `.get`, `.is_empty`, `.is_some`, `.iter`, `.join`, `.len`, `.map_err`, `.map`, `.push`, `.to_string`, `.unwrap_or_default`, `Ok`, `Vec::new`, `jubarte::convert::diff_render`, `serde_json::to_string`, `std::fs::create_dir_all`, `std::fs::write`
- **Macros:** `format!`, `outln!`
- **Called by:** `cli_main` (bin/jubarte.rs:2900); `tests::diff_render_writes_only_the_changed_pages_and_diff_json` (bin/jubarte.rs:4392); `tests::diff_render_of_equal_documents_writes_only_diff_json` (bin/jubarte.rs:4448); `tests::diff_render_without_an_overlay_writes_no_diff_png` (bin/jubarte.rs:4459)

#### `png_name`  (L770-773)
`fn png_name(stem: &str, index: usize, count: usize) -> String`

- **Does:** `<stem>-page-NN.png` for the zero-based page `index`, zero-padded to the page count's width (at least 2).
- **Calls (std/external/unresolved):** `.len`, `.max`, `.to_string`
- **Macros:** `format!`
- **Called by:** `run_convert` (bin/jubarte.rs:507); `run_diff_render` (bin/jubarte.rs:680); `run_edit` (bin/jubarte.rs:953); `write_edited` (bin/jubarte.rs:1657)

#### `run_read_tables`  (L776-780)
`fn run_read_tables(file: &Path) -> Result<(), String>`

- **Does:** `read --tables`: each body table as a grid.
- **Calls (crate):** `read_word` (bin/jubarte.rs:2262) L777; `tables` (inspect.rs:348) L778; `write_stdout` (bin/jubarte.rs:876) L779; `tables_text` (inspect.rs:406) L779
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`
- **Called by:** `run_text` (bin/jubarte.rs:785)

#### `run_text`  (L785-829)
`fn run_text( file: &Path, args: &ReadArgs, output: Option<&Path>, force: bool, quiet: bool, ) -> Result<(), ConvertFailure>`

- **Does:** `read`: the agent view, with page markers from the layout pass unless `--no-page-markers` skips it; or the snapshot (`--json`), or the grids (`--tables`).
- **Calls (crate):** `run_read_tables` (bin/jubarte.rs:776) L793; `output_format` (cli.rs:1657) L798; `<RenderArgs as Default>::default` (cli.rs:2140) L799; `check_output` (bin/jubarte.rs:270) L810; `read_word` (bin/jubarte.rs:2262) L811,814; `write_output` (bin/jubarte.rs:278) L812; `read_snapshot` (markdown/mod.rs:798) L816; `selection` (bin/jubarte.rs:832) L819; `<ReadOptions as Default>::default` (markdown/mod.rs:695) L821; `write_stdout` (bin/jubarte.rs:876) L826; `print_agent_view` (bin/jubarte.rs:869) L828
- **Calls (std/external/unresolved):** `.file_name`, `.into_owned`, `.map_err`, `.map`, `.to_string_lossy`, `.to_string`, `.unwrap_or_default`, `.unwrap_or`, `Ok`, `Produced::Docx`, `Some`, `serde_json::to_string`
- **Macros:** `format!`
- **Called by:** `cli_main` (bin/jubarte.rs:2900)

#### `selection`  (L832-840)
`fn selection(args: &ReadArgs) -> Result<Option<jubarte::markdown::Select>, String>`

- **Does:** The paragraphs `-p`, `--head`, `--tail`, `--changed` and `--by` select.
- **Calls (crate):** `Select::from_flags` (markdown/mod.rs:421) L833
- **Calls (std/external/unresolved):** `.as_deref`
- **Called by:** `write_output` (bin/jubarte.rs:278); `run_text` (bin/jubarte.rs:785); `agent_view` (bin/jubarte.rs:844); `cli_main` (bin/jubarte.rs:2900)

#### `agent_view`  (L844-866)
`fn agent_view( bytes: &[u8], source: Option<String>, args: &ReadArgs, ) -> Result<jubarte::markdown::ReadView, String>`

- **Does:** The agent view of `bytes`, its warnings said on stderr; `source` is the name the header prints.
- **Calls (crate):** `selection` (bin/jubarte.rs:832) L849; `read` (markdown/mod.rs:749) L850
- **Calls (std/external/unresolved):** `.into`, `.map_err`, `.to_string`, `.unwrap_or`, `Ok`
- **Macros:** `eprintln!`
- **Called by:** `write_output` (bin/jubarte.rs:278); `print_agent_view` (bin/jubarte.rs:869)

#### `print_agent_view`  (L869-871)
`fn print_agent_view(bytes: &[u8], source: Option<String>, args: &ReadArgs) -> Result<(), String>`

- **Does:** The agent view of `bytes` on stdout.
- **Calls (crate):** `write_stdout` (bin/jubarte.rs:876) L870; `agent_view` (bin/jubarte.rs:844) L870
- **Called by:** `run_text` (bin/jubarte.rs:785); `run` (bin/jubarte.rs:2095)

#### `write_stdout`  (L876-885)
`fn write_stdout(text: impl AsRef<[u8]>) -> Result<(), String>`

- **Does:** Writes a view, a conversion or a diff to stdout.
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.flush`, `.kind`, `.lock`, `.write_all`, `Err`, `Ok`, `std::io::stdout`
- **Macros:** `format!`
- **Called by:** `run_read_tables` (bin/jubarte.rs:776); `run_text` (bin/jubarte.rs:785); `print_agent_view` (bin/jubarte.rs:869); `run_edit` (bin/jubarte.rs:953); `write_edited` (bin/jubarte.rs:1657); `print_or_write` (bin/jubarte.rs:2369); `run_convert_any` (bin/jubarte.rs:2477); `run_debug` (bin/jubarte.rs:2746); `run_validate` (bin/jubarte.rs:2793); `run_debug_diff` (bin/jubarte.rs:2836); `cli_main` (bin/jubarte.rs:2900)

#### `edit_plan`  (L927-951)
`fn edit_plan( verb: jubarte::edit::flags::Verb, plan: Option<&Path>, operations: &[FlagOp], options: &EditOptions, source: &[u8], ) -> Result<(jubarte::edit::EditPlan, Vec<String>), (u8, String)>`

- **Does:** The plan of an `edit` or `add` invocation and the notes its flags left: `--plan`'s file, or the plan the operation flags describe.
- **Calls (crate):** `EditPlan::from_json` (edit.rs:1418) L937; `plan_from_flags` (edit/flags.rs:76) L941; `ExistingArg::plan_value` (cli.rs:252) L946
- **Calls (std/external/unresolved):** `.as_deref`, `.display`, `.map_err`, `.to_string`, `Ok`, `Vec::new`, `std::fs::read_to_string`
- **Macros:** `format!`
- **Called by:** `run_flags` (bin/jubarte.rs:1804)

#### `run_edit`  (L953-1173)
`fn run_edit( job: &EditJob<'_>, plan: &jubarte::edit::EditPlan, source: &[u8], notes: &[String], ) -> Result<(), (u8, String)>`

- **Does:** _no doc comment_
- **Calls (crate):** `preview_plan` (edit.rs:1742) L961; `refused` (bin/jubarte.rs:1943) L961,966; `write_stdout` (bin/jubarte.rs:876) L962,1136,1138,1165; `EditReport::to_jsonl` (edit.rs:1029) (by name) L962,982; `apply_plan` (edit.rs:1621) L966; `write_edited` (bin/jubarte.rs:1657) L968; `Dom::parent` (xmllinq/mod.rs:597) (by name) L971; `same_dir` (bin/jubarte.rs:187) L976; `render` (convert/mod.rs:592) L1002; `RenderReport::page_starts` (convert/mod.rs:499) (by name) L1010; `insert_before_summary` (bin/jubarte.rs:1978) L1018,1070; `.render` (ambiguous, 4 candidates) L1031; `png_name` (bin/jubarte.rs:770) L1058; `source_sha256` (inspect.rs:271) L1066; `slash_path` (cli.rs:396) L1069,1113,1163; `job_inputs` (bin/jubarte.rs:1187) L1076; `refuse_output_alias` (bin/jubarte.rs:1203) L1078,1101; `refuse_bundle_input_containment` (bin/jubarte.rs:1260) L1080,1103; `refuse_output_collisions` (bin/jubarte.rs:1248) L1082,1104; `taken_output` (bin/jubarte.rs:1176) L1084; `refuse_foreign_bundle_contents` (bin/jubarte.rs:1452) L1087,1106; `stage_bundle_dir` (bin/jubarte.rs:1495) L1089; `publish_dir` (bin/jubarte.rs:1515) L1099; `recheck_edit_source` (bin/jubarte.rs:1232) L1108; `is_pure_resolution` (bin/jubarte.rs:1626) L1122; `resolution_text` (edit/resolution.rs:367) L1124; `edit_status` (bin/jubarte.rs:1283) L1127,1134,1141,1148,1167; `edit_notes` (bin/jubarte.rs:1605) L1133,1147; `changed_view` (markdown/mod.rs:866) L1158
- **Calls (std/external/unresolved):** `.as_os_str`, `.as_path`, `.as_str`, `.clone`, `.collect`, `.display`, `.enumerate`, `.file_name`, `.filter`, `.insert`, `.into_bytes`, `.into_iter`, `.into_owned`, `.into`, `.is_empty`, `.is_ok`, `.iter`, `.join`, `.last`, `.len`, `.lines`, `.map_err`, `.map_or_else`, `.map`, `.push`, `.symlink_metadata`, `.then_some`, `.to_string_lossy`, `.to_string`, `.unwrap_or`, `Err`, `Ok`, `Path::new`, `Some`, `Vec::new`, `fail`, `jubarte::convert::PdfOptions::default`, `patch_of`, `serde_json::Map::new`, `std::fs::remove_dir_all`, `std::fs::write`
- **Macros:** `env!`, `format!`, `format_args!`, `serde_json::json!`, `vec!`
- **Called by:** `run_flags` (bin/jubarte.rs:1804)

#### `taken_output`  (L1176-1184)
`fn taken_output(path: &Path) -> (u8, String)`

- **Does:** An existing output of `edit` or `add` without --force: exit 2.
- **Calls (std/external/unresolved):** `.display`
- **Macros:** `format!`
- **Called by:** `run_edit` (bin/jubarte.rs:953); `publish_files` (bin/jubarte.rs:1359); `publish_dir` (bin/jubarte.rs:1515); `write_edited` (bin/jubarte.rs:1657)

#### `job_inputs`  (L1187-1193)
`fn job_inputs<'a>(job: &EditJob<'a>) -> Vec<(&'a Path, &'static str)>`

- **Does:** The inputs an output of `edit` or `add` must never overwrite (V2-05).
- **Calls (std/external/unresolved):** `.push`
- **Macros:** `vec!`
- **Called by:** `run_edit` (bin/jubarte.rs:953); `refuse_edit_output_alias` (bin/jubarte.rs:1221)

#### `refuse_output_alias`  (L1203-1217)
`fn refuse_output_alias(output: &Path, inputs: &[(&Path, &str)]) -> Result<(), (u8, String)>`

- **Does:** Exit 2, stable code `OUTPUT_ALIAS`: an output that names an input through another spelling, a symlink or a hard link, refused before anything is written (V2-05).
- **Calls (crate):** `same_path` (bin/jubarte.rs:155) L1205
- **Calls (std/external/unresolved):** `.display`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `run_edit` (bin/jubarte.rs:953); `refuse_edit_output_alias` (bin/jubarte.rs:1221); `refuse_output_collisions` (bin/jubarte.rs:1248)

#### `refuse_edit_output_alias`  (L1221-1228)
`fn refuse_edit_output_alias(job: &EditJob<'_>, output: &Path) -> Result<(), (u8, String)>`

- **Does:** --force authorizes an output replacement, never an input alias.
- **Calls (crate):** `job_inputs` (bin/jubarte.rs:1187) L1223; `refuse_output_alias` (bin/jubarte.rs:1203) L1227
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`
- **Called by:** `publish_files` (bin/jubarte.rs:1359); `write_edited` (bin/jubarte.rs:1657)

#### `recheck_edit_source`  (L1232-1245)
`fn recheck_edit_source(file: &Path, source: &[u8]) -> Result<(), (u8, String)>`

- **Does:** Compare the exact preparation bytes with a fresh read of the named source.
- **Calls (crate):** `source_sha256` (inspect.rs:271) L1235
- **Calls (std/external/unresolved):** `.display`, `.map_err`, `Err`, `Ok`, `std::fs::read`
- **Macros:** `format!`
- **Called by:** `run_edit` (bin/jubarte.rs:953); `publish_files` (bin/jubarte.rs:1359)

#### `refuse_output_collisions`  (L1248-1257)
`fn refuse_output_collisions(paths: &[PathBuf]) -> Result<(), (u8, String)>`

- **Does:** No two roles of a publication may resolve to the same output file.
- **Calls (crate):** `refuse_output_alias` (bin/jubarte.rs:1203) L1254
- **Calls (std/external/unresolved):** `.as_path`, `.collect`, `.enumerate`, `.iter`, `.map`, `Ok`
- **Called by:** `run_edit` (bin/jubarte.rs:953); `publish_files` (bin/jubarte.rs:1359); `write_edited` (bin/jubarte.rs:1657); `cli_main` (bin/jubarte.rs:2900)

#### `refuse_bundle_input_containment`  (L1260-1279)
`fn refuse_bundle_input_containment( dir: &Path, inputs: &[(&Path, &str)], ) -> Result<(), (u8, String)>`

- **Does:** A wholesale directory replacement must never remove an input's name.
- **Calls (std/external/unresolved):** `.display`, `.is_ok_and`, `.starts_with`, `Err`, `Ok`, `std::fs::canonicalize`
- **Macros:** `format!`
- **Called by:** `run_edit` (bin/jubarte.rs:953)

#### `edit_status`  (L1283-1286)
`fn edit_status(line: std::fmt::Arguments<'_>)`

- **Does:** Post-commit status is best effort: a closed stderr cannot undo a commit or prevent its result from reaching stdout.
- **Calls (std/external/unresolved):** `.lock`, `std::io::stderr`
- **Macros:** `writeln!`
- **Called by:** `run_edit` (bin/jubarte.rs:953); `publish_dir` (bin/jubarte.rs:1515); `write_edited` (bin/jubarte.rs:1657)

#### `comma_paths`  (L1289-1295)
`fn comma_paths(paths: &[PathBuf]) -> String`

- **Does:** `paths` comma-separated, with the `/` spelling the views use.
- **Calls (crate):** `slash_path` (cli.rs:396) L1292
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.join`, `.map`
- **Called by:** `publish_files` (bin/jubarte.rs:1359); `write_edited` (bin/jubarte.rs:1657)

#### `staging_path`  (L1300-1309)
`fn staging_path(dest: &Path, n: usize) -> PathBuf`

- **Does:** A staging path beside `dest`: the same filesystem, so the commit is a rename; hidden; unique to this process (`n` breaks a tie with the survivor of an interrupted run).
- **Calls (std/external/unresolved):** `.as_os_str`, `.file_name`, `.filter`, `.is_empty`, `.join`, `.parent`, `.push`, `.unwrap_or`, `Path::new`, `std::ffi::OsStr::new`, `std::ffi::OsString::from`, `std::process::id`
- **Macros:** `format!`
- **Called by:** `stage_file` (bin/jubarte.rs:1313); `stage_bundle_dir` (bin/jubarte.rs:1495); `publish_dir` (bin/jubarte.rs:1515)

#### `stage_file`  (L1313-1328)
`fn stage_file(path: &Path, bytes: &[u8]) -> Result<PathBuf, String>`

- **Does:** `bytes` staged in a temp file beside `path`, ready for the rename commit.
- **Calls (crate):** `staging_path` (bin/jubarte.rs:1300) L1315
- **Calls (std/external/unresolved):** `.and_then`, `.display`, `.kind`, `Err`, `Ok`, `std::fs::File::create_new`, `std::fs::remove_file`, `std::io::Write::write_all`
- **Macros:** `format!`
- **Called by:** `publish_files` (bin/jubarte.rs:1359)

#### `publication_test_hook`  (L1336-1348)
`fn publication_test_hook(phase: &str) -> Result<(), String>`

- **Does:** Test-only synchronization for filesystem races in debug Unix builds.
- **Calls (std/external/unresolved):** `.map_err`, `.read_exact`, `.set_read_timeout`, `Ok`, `Some`, `hook`, `std::env::var_os`, `std::os::unix::net::UnixStream::connect`, `std::process::id`, `std::time::Duration::from_secs`
- **Macros:** `format!`, `writeln!`
- **Called by:** `publish_files` (bin/jubarte.rs:1359); `publish_dir` (bin/jubarte.rs:1515)

#### `publication_test_hook`  (L1351-1353)
`fn publication_test_hook(_phase: &str) -> Result<(), String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `publish_files` (bin/jubarte.rs:1359); `publish_dir` (bin/jubarte.rs:1515)

#### `publish_files`  (L1359-1444)
`fn publish_files( outputs: &[(PathBuf, &[u8])], job: &EditJob<'_>, source: &[u8], ) -> Result<Vec<PathBuf>, (u8, String)>`

- **Does:** Stage every output before committing any of them.
- **Calls (crate):** `stage_file` (bin/jubarte.rs:1313) L1368; `publication_test_hook` (bin/jubarte.rs:1336) L1371,1395; `refuse_edit_output_alias` (bin/jubarte.rs:1221) L1375,1391,1409; `refuse_output_collisions` (bin/jubarte.rs:1248) L1377; `taken_output` (bin/jubarte.rs:1176) L1388,1404,1411; `recheck_edit_source` (bin/jubarte.rs:1232) L1393,1414; `comma_paths` (bin/jubarte.rs:1289) L1439
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.display`, `.find`, `.is_empty`, `.is_ok`, `.iter`, `.kind`, `.map_err`, `.map`, `.push`, `.symlink_metadata`, `.to_path_buf`, `Err`, `Ok`, `Vec::new`, `publish`, `std::fs::hard_link`, `std::fs::remove_file`, `std::fs::rename`
- **Macros:** `format!`
- **Called by:** `write_edited` (bin/jubarte.rs:1657)

#### `refuse_foreign_bundle_contents`  (L1452-1491)
`fn refuse_foreign_bundle_contents(dir: &Path) -> Result<(), (u8, String)>`

- **Does:** Exit 2, stable code `OUTPUT_DIR_NOT_EMPTY`: `--force` replaces an existing `--out-dir` wholesale only when it is an earlier jubarte bundle, every entry named by its `report.jsonl` save line (V2-05), or when it is empty (no earlier bundle/report is required).
- **Calls (std/external/unresolved):** `.as_array`, `.as_str`, `.cloned`, `.collect`, `.contains`, `.display`, `.extend`, `.file_name`, `.file_type`, `.filter_map`, `.filter`, `.flat_map`, `.flatten`, `.into_iter`, `.into_owned`, `.into`, `.is_dir`, `.is_empty`, `.is_ok_and`, `.join`, `.lines`, `.map`, `.ok`, `.sort`, `.to_string_lossy`, `.to_string`, `.unwrap_or_default`, `Err`, `Ok`, `serde_json::from_str`, `std::fs::read_dir`, `std::fs::read_to_string`
- **Macros:** `format!`
- **Called by:** `run_edit` (bin/jubarte.rs:953)

#### `stage_bundle_dir`  (L1495-1509)
`fn stage_bundle_dir(dir: &Path) -> Result<PathBuf, String>`

- **Does:** The sibling temp directory a bundle is staged in (V2-05).
- **Calls (crate):** `staging_path` (bin/jubarte.rs:1300) L1501
- **Calls (std/external/unresolved):** `.as_os_str`, `.display`, `.filter`, `.is_empty`, `.kind`, `.map_err`, `.parent`, `Err`, `Ok`, `std::fs::create_dir_all`, `std::fs::create_dir`
- **Macros:** `format!`
- **Called by:** `run_edit` (bin/jubarte.rs:953)

#### `publish_dir`  (L1515-1601)
`fn publish_dir( stage: &Path, dir: &Path, force: bool, recheck: impl Fn() -> Result<(), (u8, String)>, ) -> Result<(), (u8, String)>`

- **Does:** Publish the staged bundle as `dir` (V2-05): a rename when `dir` is free; under --force over an existing directory, a swap that replaces it wholesale — never a merge into its files.
- **Calls (crate):** `publication_test_hook` (bin/jubarte.rs:1336) L1525; `taken_output` (bin/jubarte.rs:1176) L1534; `staging_path` (bin/jubarte.rs:1300) L1547; `edit_status` (bin/jubarte.rs:1283) L1595
- **Calls (std/external/unresolved):** `.display`, `.file_type`, `.is_dir`, `.is_err`, `.is_ok_and`, `.is_ok`, `.is_symlink`, `.kind`, `.map_err`, `.ok_or_else`, `.symlink_metadata`, `Err`, `Ok`, `Some`, `failed`, `recheck`, `std::fs::remove_dir_all`, `std::fs::remove_file`, `std::fs::rename`
- **Macros:** `format!`, `format_args!`, `matches!`
- **Called by:** `run_edit` (bin/jubarte.rs:953)

#### `edit_notes`  (L1605-1621)
`fn edit_notes(notes: &[String], result: &jubarte::edit::EditResult) -> Vec<String>`

- **Does:** The `note:` lines of an applied edit: the flags' notes, then each anchor that matched as the paragraph reads rather than as typed.
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.map`, `.push`, `jubarte::markdown::plain_anchor`
- **Macros:** `format!`
- **Called by:** `run_edit` (bin/jubarte.rs:953); `write_edited` (bin/jubarte.rs:1657)

#### `is_pure_resolution`  (L1626-1630)
`fn is_pure_resolution(plan: &jubarte::edit::EditPlan, result: &jubarte::edit::EditResult) -> bool`

- **Does:** Whether `plan` resolved only revisions and `result` resolved some: an `edit -p rN --accept/--reject` with no content operation prints the resolved lines, not the view.
- **Calls (crate):** `ResolvedRevisions::is_empty` (edit.rs:142) L1629
- **Calls (std/external/unresolved):** `.is_empty`, `.is_some`
- **Called by:** `run_edit` (bin/jubarte.rs:953); `resolved_or_view` (bin/jubarte.rs:1635)

#### `resolved_or_view`  (L1635-1651)
`fn resolved_or_view( source: &[u8], plan: &jubarte::edit::EditPlan, result: &jubarte::edit::EditResult, author: &str, accepted: bool, shown: Option<&str>, ) -> Result<String, String>`

- **Does:** The resolved lines for a pure accept/reject, or the changed view otherwise.
- **Calls (crate):** `is_pure_resolution` (bin/jubarte.rs:1626) L1643; `resolution_text` (edit/resolution.rs:367) L1645; `changed_view` (markdown/mod.rs:866) L1649
- **Calls (std/external/unresolved):** `.map_err`, `Ok`
- **Macros:** `format!`
- **Called by:** `write_edited` (bin/jubarte.rs:1657)

#### `write_edited`  (L1657-1783)
`fn write_edited( job: &EditJob<'_>, path: &Path, result: &jubarte::edit::EditResult, notes: &[String], source: &[u8], plan: &jubarte::edit::EditPlan, ) -> Result<(), (u8, String)>`

- **Does:** `edit` and `add` without --out-dir: the redline (in editing mode, the result) as one .docx at `path`, its PDF and PNG pages beside it, the `wrote` lines and notes on stderr, and the changed blocks' view of that file on stdout (a pure accept/reject prints the resolved lines).
- **Calls (crate):** `render` (convert/mod.rs:592) L1684; `png_name` (bin/jubarte.rs:770) L1706; `refuse_edit_output_alias` (bin/jubarte.rs:1221) L1714; `refuse_output_collisions` (bin/jubarte.rs:1248) L1716; `taken_output` (bin/jubarte.rs:1176) L1726; `publish_files` (bin/jubarte.rs:1359) L1730; `edit_status` (bin/jubarte.rs:1283) L1734,1744,1751,1757,1779; `wrote_line` (cli.rs:1721) L1736,1746; `wrote_png_line` (cli.rs:1732) L1753; `edit_notes` (bin/jubarte.rs:1605) L1756; `resolved_or_view` (bin/jubarte.rs:1635) L1761; `slash_path` (cli.rs:396) L1768; `write_stdout` (bin/jubarte.rs:876) L1772; `comma_paths` (bin/jubarte.rs:1289) L1775
- **Calls (std/external/unresolved):** `.as_os_str`, `.clone`, `.collect`, `.display`, `.enumerate`, `.file_stem`, `.filter`, `.find`, `.into_owned`, `.is_empty`, `.is_ok`, `.iter`, `.join`, `.len`, `.map_err`, `.map_or_else`, `.map_or`, `.map`, `.parent`, `.push`, `.symlink_metadata`, `.then_some`, `.to_path_buf`, `.to_string_lossy`, `.to_string`, `.with_extension`, `Err`, `Ok`, `Some`, `fail`, `jubarte::convert::PdfOptions::default`
- **Macros:** `format!`, `format_args!`, `vec!`
- **Called by:** `run_edit` (bin/jubarte.rs:953)

#### `run_flags`  (L1804-1880)
`fn run_flags(flags: &EditFlags<'_>) -> Result<(), (u8, String)>`

- **Does:** _no doc comment_
- **Calls (crate):** `read_word` (bin/jubarte.rs:2262) L1805; `record_plan` (bin/jubarte.rs:1887) L1809; `edit_plan` (bin/jubarte.rs:927) L1830; `editing_mode_conflict` (edit/flags.rs:544) L1838; `EditOptions::target` (cli.rs:212) L1842; `run_edit` (bin/jubarte.rs:953) L1861; `kept` (ambiguous: `kept` (edit/rewrite.rs:14); `kept` (markdown/from_docx/revise.rs:36)) L1873,1877
- **Calls (std/external/unresolved):** `.as_ref`, `.display`, `.filter`, `.into`, `.is_empty`, `.is_ok_and`, `.is_some`, `.is_symlink`, `.map_err`, `.ok`, `.symlink_metadata`, `.to_string`, `Err`, `Ok`, `Some`, `std::fs::read`, `std::fs::remove_file`
- **Macros:** `format!`
- **Called by:** `cli_main` (bin/jubarte.rs:2900)

#### `record_plan`  (L1887-1930)
`fn record_plan(flags: &EditFlags<'_>, path: &Path, source: &[u8]) -> Result<(), (u8, String)>`

- **Does:** `--plan PLAN.json` with operation flags: the operations go into the plan (created on first use) instead of into FILE.
- **Calls (crate):** `record` (edit/record.rs:181) L1895; `ExistingArg::plan_value` (cli.rs:252) (by name) L1900; `check_plan` (edit.rs:1728) L1908; `refused` (bin/jubarte.rs:1943) L1908; `write_plan` (edit/record.rs:160) L1915; `Recorded::json` (edit/record.rs:84) (by name) L1915; `.summary` (ambiguous: `Recorded::summary` (edit/record.rs:91); `Opened::summary` (inspect.rs:545)) L1921
- **Calls (std/external/unresolved):** `.as_deref`, `.display`, `.kind`, `.map_err`, `.to_string`, `Err`, `Ok`, `Some`, `std::fs::read_to_string`
- **Macros:** `format!`, `outln!`
- **Called by:** `run_flags` (bin/jubarte.rs:1804)

#### `edit_exit`  (L1932-1940)
`fn edit_exit(result: Result<(), (u8, String)>) -> ExitCode`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `ExitCode::from`
- **Macros:** `eprintln!`
- **Called by:** `cli_main` (bin/jubarte.rs:2900)

#### `refused`  (L1943-1975)
`fn refused(e: &jubarte::edit::EditError) -> (u8, String)`

- **Does:** A refused plan: its report goes to stdout, the error to stderr, exit 3.
- **Calls (std/external/unresolved):** `.as_object_mut`, `.clone`, `.collect`, `.enumerate`, `.expect`, `.insert`, `.into`, `.iter`, `.map`, `.to_string`
- **Macros:** `format!`, `outln!`, `serde_json::json!`
- **Called by:** `run_edit` (bin/jubarte.rs:953); `record_plan` (bin/jubarte.rs:1887); `tests::recording_refuses_what_the_plan_names_differently` (edit/flags.rs:835) (?); `Select::parse` (markdown/mod.rs:453) (?)

#### `insert_before_summary`  (L1978-1989)
`fn insert_before_summary(jsonl: &mut String, line: &str)`

- **Does:** Insert `line` before the trailing `summary` line of a JSONL report.
- **Calls (std/external/unresolved):** `.is_empty`, `.rfind`, `.trim_end_matches`
- **Macros:** `format!`
- **Called by:** `run_edit` (bin/jubarte.rs:953)

#### `resolve_compare`  (L2021-2052)
`fn resolve_compare(compare: CompareArgs, quiet: bool) -> Job`

- **Does:** Native runtime defaults; required inputs have already been checked by clap.
- **Calls (std/external/unresolved):** `.expect`, `.or`, `.unwrap_or_else`
- **Called by:** `cli_main` (bin/jubarte.rs:2900); `tests::job_of` (bin/jubarte.rs:3544); `tests::revisions_is_no_subcommand` (bin/jubarte.rs:3709); `tests::plain_compare_positionals_leave_command_none` (bin/jubarte.rs:3721)

#### `default_output`  (L2055-2066)
`fn default_output(original: &Path, modified: &Path) -> PathBuf`

- **Does:** Build the default output path: `<original-dir>/<orig-stem>_v_<mod-stem>.docx`.
- **Calls (std/external/unresolved):** `.as_os_str`, `.file_stem`, `.filter`, `.into_owned`, `.is_empty`, `.join`, `.map`, `.parent`, `.to_string_lossy`, `.to_string`, `.unwrap_or_else`, `PathBuf::from`, `stem`
- **Macros:** `format!`
- **Called by:** `run` (bin/jubarte.rs:2095); `tests::default_output_uses_original_directory_and_stems` (bin/jubarte.rs:3581)

#### `comparer_settings`  (L2070-2093)
`fn comparer_settings( author: &str, date: &str, detail_threshold: Option<f64>, powertools_faithful: bool, no_paragraph_merge: bool, ) -> jubarte::comparer::WmlComparerSettings`

- **Does:** The comparer's settings for the `--mode`, `--detail-threshold`, author and date flags.
- **Calls (crate):** `WmlComparerSettings::powertools_faithful` (comparer/mod.rs:1464) L2078; `<WmlComparerSettings as Default>::default` (comparer/mod.rs:1475) L2080
- **Calls (std/external/unresolved):** `.to_string`, `.unwrap_or`
- **Called by:** `run` (bin/jubarte.rs:2095); `cli_main` (bin/jubarte.rs:2900)

#### `run`  (L2095-2193)
`fn run(job: &Job) -> Result<(), ConvertFailure>`

- **Does:** _no doc comment_
- **Calls (crate):** `output_format` (cli.rs:1657) L2096; `slash_path` (cli.rs:396) L2100; `default_output` (bin/jubarte.rs:2055) L2100; `read_document` (bin/jubarte.rs:2241) L2112,2113; `comparer_settings` (bin/jubarte.rs:2070) L2114; `Input::new` (bin/jubarte.rs:2213) L2121,2126; `Format::of_input` (cli.rs:1543) L2123,2128; `check_output` (bin/jubarte.rs:270) L2156; `write_output` (bin/jubarte.rs:278) L2163,2191; `compare_documents_with_settings` (document_comparer.rs:6414) L2166; `refusal` (bin/jubarte.rs:2275) L2169; `image_loader` (bin/jubarte.rs:2291) L2179; `resource_dir` (bin/jubarte.rs:2326) L2179; `Input::source` (bin/jubarte.rs:2230) L2186; `print_agent_view` (bin/jubarte.rs:869) L2190
- **Calls (std/external/unresolved):** `.as_deref`, `.as_ref`, `.clone`, `.find_map`, `.into_iter`, `.is_none`, `.is_some_and`, `.is_some`, `.map_err`, `.map`, `.then_some`, `.to_string`, `.transpose`, `.unwrap_or_else`, `.unwrap_or`, `Err`, `Ok`, `Produced::Docx`, `Produced::Text`, `Some`, `jubarte::markdown::diff_markdown`, `jubarte::markdown::redline`, `usage`
- **Macros:** `format!`
- **Called by:** `run_resolution` (bin/jubarte.rs:361); `cli_main` (bin/jubarte.rs:2900); `tests::a_legacy_doc_is_refused_with_a_save_as_hint` (bin/jubarte.rs:4030)

#### `exit_code`  (L2196-2204)
`fn exit_code(r: Result<(), String>) -> ExitCode`

- **Does:** Shared `Result → ExitCode` mapping for every command arm.
- **Macros:** `eprintln!`
- **Called by:** `convert_exit_code` (bin/jubarte.rs:495); `cli_main` (bin/jubarte.rs:2900)

#### `Input::new`  (L2213-2228)
`fn new(path: &Path, format: Format, bytes: Vec<u8>) -> Result<Self, String>`

- **Does:** _no doc comment_
- **Calls (crate):** `markdown_text` (bin/jubarte.rs:2280) L2215; `admit_word` (bin/jubarte.rs:2269) L2217
- **Calls (std/external/unresolved):** `.clone`, `.display`, `Err`, `Ok`, `Some`
- **Macros:** `format!`
- **Called by:** `run` (bin/jubarte.rs:2095); `diff_inputs` (bin/jubarte.rs:2390)

#### `Input::source`  (L2230-2235)
`fn source(&self) -> jubarte::markdown::Source<'_>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `jubarte::markdown::Source::Docx`, `jubarte::markdown::Source::Markdown`
- **Called by:** `run` (bin/jubarte.rs:2095); `run_text_diff` (bin/jubarte.rs:2378) (?); `run_diff` (bin/jubarte.rs:2407) (?)

#### `read_document`  (L2241-2258)
`fn read_document(path: &Path) -> Result<Vec<u8>, String>`

- **Does:** Reads an input document, refusing the OLE files Word alone can open and RTF by their signature (`admission::sniff`, the check the library and the bindings run too).
- **Calls (crate):** `refused_input` (cli.rs:1703) L2243; `read` (legacy_doc.rs:180) L2246
- **Calls (std/external/unresolved):** `.display`, `.is_ok`, `.map_err`, `.with_extension`, `Err`, `Ok`, `String::new`, `std::fs::read`
- **Macros:** `format!`
- **Called by:** `run` (bin/jubarte.rs:2095); `read_word` (bin/jubarte.rs:2262); `diff_inputs` (bin/jubarte.rs:2390); `tests::a_legacy_doc_is_refused_with_a_save_as_hint` (bin/jubarte.rs:4030)

#### `read_word`  (L2262-2266)
`fn read_word(path: &Path) -> Result<Vec<u8>, String>`

- **Does:** [`read_document`] for an input that must be Word: a file that is not a ZIP package is refused by name (`admission::sniff_package`).
- **Calls (crate):** `read_document` (bin/jubarte.rs:2241) L2263; `admit_word` (bin/jubarte.rs:2269) L2264
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `run_scrub` (bin/jubarte.rs:106); `run_resolution` (bin/jubarte.rs:361); `run_changes` (bin/jubarte.rs:399); `run_comments` (bin/jubarte.rs:415); `run_convert` (bin/jubarte.rs:507); `run_diff_render` (bin/jubarte.rs:680); `run_read_tables` (bin/jubarte.rs:776); `run_text` (bin/jubarte.rs:785); `run_flags` (bin/jubarte.rs:1804); `run_append` (bin/jubarte.rs:3357)

#### `admit_word`  (L2269-2271)
`fn admit_word(path: &Path, bytes: &[u8]) -> Result<(), String>`

- **Does:** `CODE: FILE is …` when `bytes`, read from `path`, are no Word package.
- **Calls (crate):** `refused_input` (cli.rs:1703) L2270
- **Calls (std/external/unresolved):** `.map_or`, `Ok`
- **Called by:** `Input::new` (bin/jubarte.rs:2213); `read_word` (bin/jubarte.rs:2262); `run_convert_any` (bin/jubarte.rs:2477)

#### `refusal`  (L2275-2277)
`fn refusal(what: &str, error: &impl std::fmt::Display) -> Option<String>`

- **Does:** `{what} failed: CODE: …` for an admission refusal, whatever wrapped it (`admission::code_first`); `None` for any other error.
- **Calls (crate):** `code_first` (admission.rs:185) L2276
- **Calls (std/external/unresolved):** `.map`, `.to_string`
- **Macros:** `format!`
- **Called by:** `run` (bin/jubarte.rs:2095); `run_convert_any` (bin/jubarte.rs:2477); `tests::revision_and_reply_flags_pair_only_with_their_kind` (edit/flags.rs:1410) (?)

#### `markdown_text`  (L2280-2287)
`fn markdown_text(path: &Path, bytes: Vec<u8>) -> Result<String, String>`

- **Does:** Markdown bytes as text, without a byte order mark.
- **Calls (std/external/unresolved):** `.display`, `.map_err`, `.map`, `.strip_prefix`, `.unwrap_or`, `Ok`, `String::from_utf8`
- **Macros:** `format!`
- **Called by:** `Input::new` (bin/jubarte.rs:2213); `run_convert_any` (bin/jubarte.rs:2477)

#### `image_loader`  (L2291-2323)
`fn image_loader(dir: PathBuf) -> impl Fn(&str) -> Option<Vec<u8>>`

- **Does:** Reads images a Markdown file names: local paths, relative to `dir`, with `%XX` escapes decoded.
- **Calls (std/external/unresolved):** `.and_then`, `.as_bytes`, `.contains`, `.get`, `.is_absolute`, `.join`, `.len`, `.ok`, `.push`, `.starts_with`, `PathBuf::from`, `String::from_utf8`, `Vec::with_capacity`, `std::fs::read`, `std::str::from_utf8`, `u8::from_str_radix`
- **Called by:** `run` (bin/jubarte.rs:2095); `run_diff` (bin/jubarte.rs:2407); `run_convert_any` (bin/jubarte.rs:2477)

#### `resource_dir`  (L2326-2333)
`fn resource_dir(markdown: &Path, resource_path: Option<&Path>) -> PathBuf`

- **Does:** The directory a Markdown file's images are read from.
- **Calls (std/external/unresolved):** `.as_os_str`, `.filter`, `.is_empty`, `.map_or_else`, `.map`, `.parent`, `.unwrap_or_else`, `PathBuf::from`
- **Called by:** `run` (bin/jubarte.rs:2095); `run_diff` (bin/jubarte.rs:2407); `run_convert_any` (bin/jubarte.rs:2477)

#### `diff_output`  (L2353-2366)
`fn diff_output<'a>( output: Option<&'a Path>, input: &'a Path, render: &'a RenderArgs, force: bool, quiet: bool, ) -> Result<Option<Output<'a>>, ConvertFailure>`

- **Does:** `diff -o FILE`: text, through the shared writer; its existing file is refused before any work.
- **Calls (crate):** `Output::of_task` (bin/jubarte.rs:225) L2363; `check_output` (bin/jubarte.rs:270) L2364
- **Calls (std/external/unresolved):** `Ok`, `Some`
- **Called by:** `run_text_diff` (bin/jubarte.rs:2378); `run_diff` (bin/jubarte.rs:2407)

#### `print_or_write`  (L2369-2374)
`fn print_or_write(out: Option<&Output<'_>>, text: &str) -> Result<(), ConvertFailure>`

- **Does:** `text` on stdout, or written to `out`.
- **Calls (crate):** `write_output` (bin/jubarte.rs:278) L2371; `write_stdout` (bin/jubarte.rs:876) L2372
- **Calls (std/external/unresolved):** `.map_err`, `Produced::Text`
- **Called by:** `run_text_diff` (bin/jubarte.rs:2378); `run_diff` (bin/jubarte.rs:2407)

#### `run_text_diff`  (L2378-2387)
`fn run_text_diff( job: &DiffJob<'_>, options: &jubarte::text_diff::TextOptions, ) -> Result<(), ConvertFailure>`

- **Does:** The text views (github, word, normal, context, side-by-side) use the shared core directly, with no redline pass.
- **Calls (crate):** `<RenderArgs as Default>::default` (cli.rs:2140) L2382; `diff_output` (bin/jubarte.rs:2353) L2383; `diff_inputs` (bin/jubarte.rs:2390) L2384; `diff_documents_view` (text_diff.rs:125) L2385; `Input::source` (bin/jubarte.rs:2230) (by name) L2385; `print_or_write` (bin/jubarte.rs:2369) L2386
- **Calls (std/external/unresolved):** `.as_ref`
- **Called by:** `cli_main` (bin/jubarte.rs:2900)

#### `diff_inputs`  (L2390-2403)
`fn diff_inputs(job: &DiffJob<'_>) -> Result<(Input, Input), String>`

- **Does:** The two documents of a `diff`, each read as its name or bytes say.
- **Calls (crate):** `read_document` (bin/jubarte.rs:2241) L2391; `Input::new` (bin/jubarte.rs:2213) L2392,2397; `Format::of_input` (cli.rs:1543) L2394,2399
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `run_text_diff` (bin/jubarte.rs:2378); `run_diff` (bin/jubarte.rs:2407)

#### `run_diff`  (L2407-2455)
`fn run_diff(job: &DiffJob<'_>) -> Result<(), ConvertFailure>`

- **Does:** `diff`'s paragraph patch, or `--format critic`'s CriticMarkup (two Markdown documents only), on stdout or in `-o`'s text file.
- **Calls (crate):** `<RenderArgs as Default>::default` (cli.rs:2140) L2408; `diff_output` (bin/jubarte.rs:2353) L2409; `diff_inputs` (bin/jubarte.rs:2390) L2410; `image_loader` (bin/jubarte.rs:2291) L2419; `resource_dir` (bin/jubarte.rs:2326) L2419; `.render` (ambiguous, 4 candidates) L2427; `Input::source` (bin/jubarte.rs:2230) (by name) L2428,2429; `print_or_write` (bin/jubarte.rs:2369) L2454
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.display`, `.into`, `.is_some`, `.map_err`, `.to_string`, `Err`, `Some`, `jubarte::markdown::diff_markdown`, `jubarte::markdown::patch_documents`, `name`
- **Macros:** `format!`
- **Called by:** `cli_main` (bin/jubarte.rs:2900)

#### `paginated`  (L2463-2475)
`fn paginated( docx: &[u8], markdown: &str, track_changes: TrackChanges, revisions: jubarte::convert::RevisionStyle, ) -> String`

- **Does:** `convert`, for every pair of formats it takes.
- **Calls (crate):** `with_page_markers` (markdown/mod.rs:951) L2470
- **Calls (std/external/unresolved):** `.into`
- **Macros:** `eprintln!`
- **Called by:** `run_convert_any` (bin/jubarte.rs:2477)

#### `run_convert_any`  (L2477-2743)
`fn run_convert_any( job: &ConvertJob<'_>, markdown: &MarkdownArgs, update_fields: bool, ) -> Result<(), ConvertFailure>`

- **Does:** _no doc comment_
- **Calls (crate):** `is_compound_file` (legacy_doc.rs:166) L2486; `doc_to_docx` (legacy_doc.rs:235) L2488; `sniff` (admission.rs:216) L2490; `Format::of_input` (cli.rs:1543) L2497; `admit_word` (bin/jubarte.rs:2269) L2500; `<RenderArgs as Default>::default` (cli.rs:2140) L2534; `Output::of_task` (bin/jubarte.rs:225) L2541; `write_output` (bin/jubarte.rs:278) L2543; `check_output` (bin/jubarte.rs:270) L2544; `run_convert` (bin/jubarte.rs:507) L2548; `at` (markdown/from_docx/drawing.rs:1134) (by name) L2557,2569,2610,2635,2676,2686...; `accept_revisions` (document_comparer.rs:4959) L2560; `refusal` (bin/jubarte.rs:2275) L2562,2566,2581,2669; `reject_revisions` (document_comparer.rs:4976) L2564; `resolve` (changes.rs:349) L2580,2668; `read` (markdown/mod.rs:749) L2589; `write_stdout` (bin/jubarte.rs:876) L2611,2636,2687; `docx_to_markdown` (markdown/mod.rs:641) L2615; `<MarkdownOptions as Default>::default` (markdown/mod.rs:340) L2620; `paginated` (bin/jubarte.rs:2463) L2627; `check_side_file` (bin/jubarte.rs:340) L2664; `ensure_writable` (bin/jubarte.rs:195) L2666; `refresh_fields` (bin/jubarte.rs:3399) L2674; `markdown_text` (bin/jubarte.rs:2280) L2679,2691; `resolve_critic` (markdown/mod.rs:146) L2683; `image_loader` (bin/jubarte.rs:2291) L2697; `resource_dir` (bin/jubarte.rs:2326) L2697
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.as_path`, `.as_slice`, `.clone`, `.display`, `.file_name`, `.into_owned`, `.into`, `.is_some`, `.map_err`, `.map_or_else`, `.map`, `.name`, `.or_else`, `.or`, `.then_some`, `.to_path_buf`, `.to_string_lossy`, `.to_string`, `.transpose`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `.with_extension`, `Err`, `Ok`, `Produced::Docx`, `Produced::Text`, `Some`, `jubarte::markdown::markdown_to_docx`, `output_at`, `pdf_job`, `std::fs::read`
- **Macros:** `eprintln!`, `format!`, `matches!`
- **Called by:** `cli_main` (bin/jubarte.rs:2900)

#### `run_debug`  (L2746-2774)
`fn run_debug( files: &[PathBuf], list: bool, checks: Vec<DebugCheck>, part: Option<String>, grep: Option<String>, limit: usize, context: usize, ) -> Result<(), String>`

- **Does:** `jubarte debug`: print the report for one package, or two compared.
- **Calls (crate):** `read` (ambiguous: `read` (legacy_doc.rs:180); `read` (markdown/mod.rs:749); `read` (markdown/write.rs:125)) L2756; `list` (debug.rs:2331) L2769; `report` (debug.rs:2457) L2771; `write_stdout` (bin/jubarte.rs:876) L2773
- **Calls (std/external/unresolved):** `.as_deref`, `.collect`, `.display`, `.get`, `.into_iter`, `.is_empty`, `.map_err`, `.map`, `.transpose`, `Default::default`, `std::fs::read`
- **Macros:** `format!`
- **Called by:** `cli_main` (bin/jubarte.rs:2900)

#### `run_validate`  (L2793-2832)
`fn run_validate(job: &ValidateJob<'_>) -> Result<bool, String>`

- **Does:** `jubarte validate`: `Ok(true)` when the findings fail the run ([`jubarte::validate::fails`]), `Err` when a file could not be read or written or the rules cannot run.
- **Calls (crate):** `read` (ambiguous: `read` (legacy_doc.rs:180); `read` (markdown/mod.rs:749); `read` (markdown/write.rs:125)) L2796; `repair` (validate.rs:1649) L2804; `wrote_line` (cli.rs:1721) L2810; `validate_with` (validate.rs:290) L2817; `write_stdout` (bin/jubarte.rs:876) L2829; `findings_text` (validate.rs:359) L2829; `fails` (validate.rs:345) L2831
- **Calls (std/external/unresolved):** `.as_deref`, `.collect`, `.display`, `.exists`, `.iter`, `.len`, `.map_err`, `.map`, `.to_string`, `.transpose`, `.zip`, `Err`, `Ok`, `Some`, `serde_json::to_string`, `std::fs::read`, `std::fs::write`
- **Macros:** `eprintln!`, `format!`, `outln!`
- **Called by:** `cli_main` (bin/jubarte.rs:2900)

#### `run_debug_diff`  (L2836-2879)
`fn run_debug_diff( files: &[PathBuf], opts: &jubarte::debug::diff::DiffOptions, ) -> Result<(), String>`

- **Does:** `jubarte debug diff`: each file labelled by its stem (by its folder too when stems repeat, by position when both do).
- **Calls (crate):** `Dom::parent` (xmllinq/mod.rs:597) (by name) L2846; `unique` (edit/v2.rs:463) (by name) L2861,2864; `write_stdout` (bin/jubarte.rs:876) L2878; `diff` (debug/diff.rs:60) L2878
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.display`, `.file_name`, `.file_stem`, `.into_owned`, `.iter`, `.len`, `.map_err`, `.map`, `.to_string_lossy`, `.to_string`, `.unwrap_or_default`, `.zip`, `char::from`, `std::fs::read`, `stem`
- **Macros:** `format!`
- **Called by:** `cli_main` (bin/jubarte.rs:2900)

#### `main`  (L2886-2898)
`fn main() -> ExitCode`

- **Does:** _no doc comment_
- **Calls (crate):** `cli_main` (bin/jubarte.rs:2900) L2896
- **Calls (std/external/unresolved):** `.into`, `.map`, `.name`, `.spawn`, `.stack_size`, `std::panic::resume_unwind`, `std::thread::Builder::new`
- **Called by:** none found in src

#### `cli_main`  (L2900-3354)
`fn cli_main() -> ExitCode`

- **Does:** _no doc comment_
- **Calls (crate):** `Cli::try_parse_from` (cli.rs:2913) L2905; `refuse_output_collisions` (bin/jubarte.rs:1248) L2921; `removed_spellings` (cli.rs:1978) L2929; `removed_error` (cli.rs:1986) L2931; `notes` (cli.rs:2048) L2936,2941; `convert_exit_code` (bin/jubarte.rs:495) L2947,3042,3095,3097,3110,3295...; `run` (bin/jubarte.rs:2095) L2947,3353; `resolve_compare` (bin/jubarte.rs:2021) L2947,3353; `exit_code` (bin/jubarte.rs:2196) L2950,2958,2969,2988,3018,3022...; `run_changes` (bin/jubarte.rs:399) L2950; `run_comments` (bin/jubarte.rs:415) L2958; `task_output` (bin/jubarte.rs:249) L2967,2986,3291,3330; `run_resolution` (bin/jubarte.rs:361) L2971,2990; `arm_timeout` (bin/jubarte.rs:460) L3014; `pdf_options` (bin/jubarte.rs:136) L3016; `run_convert_any` (bin/jubarte.rs:2477) L3042; `comparer_settings` (bin/jubarte.rs:2070) L3065; `PatchFormat::is_text_view` (cli.rs:1462) (by name) L3075; `run_text_diff` (bin/jubarte.rs:2378) L3095; `run_diff` (bin/jubarte.rs:2407) L3097; `selection` (bin/jubarte.rs:832) L3106; `run_text` (bin/jubarte.rs:785) L3110,3345; `revision_style` (bin/jubarte.rs:119) L3127; `edit_exit` (bin/jubarte.rs:1932) L3146,3172; `run_flags` (bin/jubarte.rs:1804) L3146,3172; `write_stdout` (bin/jubarte.rs:876) L3184,3189,3201; `capabilities` (capabilities.rs:355) L3193; `capabilities_text` (capabilities.rs:441) L3201; `run_self_update` (bin/jubarte.rs:3379) L3209; `run_debug_diff` (bin/jubarte.rs:2836) L3231; `run_diff_render` (bin/jubarte.rs:680) L3242; `run_debug` (bin/jubarte.rs:2746) L3270; `run_append` (bin/jubarte.rs:3357) L3295; `run_validate` (bin/jubarte.rs:2793) L3318; `run_scrub` (bin/jubarte.rs:106) L3334; `.options` (ambiguous: `ScrubOptions::options` (bin/jubarte.rs:63); `<ScrubSelection as ScrubOptions>::options` (bin/jubarte.rs:67)) L3334
- **Calls (std/external/unresolved):** `.as_deref`, `.chain`, `.clone`, `.collect`, `.display`, `.exit`, `.expect`, `.filter`, `.first`, `.into_owned`, `.into`, `.is_none`, `.is_some`, `.iter`, `.kind`, `.map_or`, `.map`, `.or_else`, `.or`, `.skip`, `.then_some`, `.to_string_lossy`, `.to_string`, `.transpose`, `.unwrap_or_else`, `.with_extension`, `::try_parse_from`, `Err`, `ExitCode::from`, `Ok`, `Path::new`, `i32::from`, `jubarte::capabilities::discovery_text`, `jubarte::capabilities::discovery`, `serde_json::to_string_pretty`, `std::env::args_os`, `std::ffi::OsString::from`, `std::iter::once`, `std::process::exit`
- **Macros:** `eprintln!`, `format!`, `outln!`, `unreachable!`
- **Called by:** `main` (bin/jubarte.rs:2886)

#### `run_append`  (L3357-3376)
`fn run_append( files: &[PathBuf], out: &Output<'_>, options: &jubarte::append::AppendOptions, ) -> Result<(), ConvertFailure>`

- **Does:** `jubarte append`: fold the documents left, `append(append(A, B), C)`.
- **Calls (crate):** `check_output` (bin/jubarte.rs:270) L3362; `read_word` (bin/jubarte.rs:2262) L3365,3367; `append_documents` (append.rs:152) L3368; `write_output` (bin/jubarte.rs:278) L3375
- **Calls (std/external/unresolved):** `.display`, `.iter`, `.map_err`, `.next`, `.ok_or`, `Produced::Docx`
- **Macros:** `eprintln!`, `format!`
- **Called by:** `cli_main` (bin/jubarte.rs:2900)

#### `run_self_update`  (L3379-3385)
`fn run_self_update(check: bool, yes: bool, version: Option<String>) -> Result<(), String>`

- **Does:** _no doc comment_
- **Calls (crate):** `run` (update.rs:186) L3380
- **Called by:** `cli_main` (bin/jubarte.rs:2900)

#### `run_self_update`  (L3388-3390)
`fn run_self_update(_check: bool, _yes: bool, _version: Option<String>) -> Result<(), String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`, `Err`
- **Called by:** `cli_main` (bin/jubarte.rs:2900)

#### `refresh_fields`  (L3399-3424)
`fn refresh_fields(bytes: &[u8], report: Option<&Path>) -> Result<Vec<u8>, String>`

- **Does:** `convert --update-fields`: the package with its field results refreshed from jubarte's layout, one line per field written on stdout and `{page_count, fields}` in `report` when given.
- **Calls (crate):** `update_fields` (fields.rs:115) L3400
- **Calls (std/external/unresolved):** `.display`, `.len`, `.map_err`, `.to_string`, `Ok`, `std::fs::write`
- **Macros:** `eprintln!`, `format!`, `outln!`, `serde_json::json!`
- **Called by:** `run_convert_any` (bin/jubarte.rs:2477); `tests::refresh_fields_writes_the_report_and_says_when_it_cannot` (bin/jubarte.rs:4503)

## Test functions (48)

- `tests::append_parses_files_output_and_break` (L3433): _no doc comment_
- `tests::self_update_parses_check_yes_and_a_pinned_version` (L3493): _no doc comment_
- `tests::convert_revisions_default_to_conventional_and_validate_the_palette` (L3518): _no doc comment_
- `tests::job_of` (L3544): _no doc comment_
- `tests::cli_definition_is_valid` (L3555): clap's own invariants (catches derive-config mistakes like duplicate shorts).
- `tests::positional_args_and_default_output` (L3560): _no doc comment_
- `tests::default_output_uses_original_directory_and_stems` (L3581): _no doc comment_
- `tests::named_flags_override_positionals` (L3593): _no doc comment_
- `tests::all_options_long_and_short` (L3608): _no doc comment_
- `tests::compare_mode_defaults_to_word_and_powertools_has_two_spellings` (L3631): _no doc comment_
- `tests::flags_can_supply_both_inputs_without_positionals` (L3646): _no doc comment_
- `tests::double_dash_treats_rest_as_positionals` (L3663): _no doc comment_
- `tests::help_and_version_are_handled_by_clap` (L3670): _no doc comment_
- `tests::missing_inputs_are_clap_usage_errors` (L3679): _no doc comment_
- `tests::extra_positional_and_unknown_flag_rejected_by_clap` (L3692): _no doc comment_
- `tests::revisions_is_no_subcommand` (L3709): `revisions` (Docxodus's GetRevisions listing) left the CLI: `changes` is the one listing.
- `tests::plain_compare_positionals_leave_command_none` (L3721): Prior behavior path: a two-positional invocation whose filenames do NOT collide with the subcommand name is unaffected by adding `command` to `Cli` — `cli.command` stays `None` and the native resolver merges the positionals exactly as before this PR.
- `tests::accept_subcommand_parses_file_output_and_force` (L3732): `accept <file> -o <out> --force` parses into `Command::Accept` with the output and force flag captured.
- `tests::scrub_flags_select_what_goes_and_none_selects_everything` (L3760): `scrub` without a selection removes all four kinds, under the alias `Author`; any flag narrows it to the ones given.
- `tests::reject_subcommand_parses_file_output_and_defaults_force_false` (L3797): `reject <file> -o <out>` parses into `Command::Reject`; `force` defaults to false (the same no-clobber contract as `accept` / compare).
- `tests::accept_selection_flags_build_the_change_filter` (L3823): `--id`, `--author` and `--kind` repeat and select the changes to resolve; a flag left out constrains nothing.
- `tests::reject_subcommand_requires_output` (L3855): `reject` requires `-o/--output` (clap usage error when omitted), matching `accept`.
- `tests::convert_takes_fail_on_substitution_and_defaults_it_off` (L3862): _no doc comment_
- `tests::convert_subcommand_parses_font_report` (L3880): _no doc comment_
- `tests::tiny_docx_bytes` (L3915): _no doc comment_
- `tests::body_docx_bytes` (L3922): A Letter-size package whose body is `body` (WordprocessingML).
- `tests::convert_rejects_a_font_report_path_equal_to_the_pdf` (L3949): _no doc comment_
- `tests::a_legacy_doc_is_refused_with_a_save_as_hint` (L4030): _no doc comment_
- `tests::fail_on_substitution_is_exit_4_after_the_outputs` (L4070): _no doc comment_
- `tests::a_plain_convert_error_keeps_exit_1` (L4104): _no doc comment_
- `tests::convert_font_report_writes_json` (L4110): _no doc comment_
- `tests::regression_tests::convert_without_revision_flags_uses_conventional_marks` (L4151): _no doc comment_
- `tests::regression_tests::custom_revision_flags_reach_the_palette_parser` (L4165): _no doc comment_
- `tests::regression_tests::invalid_revision_flags_are_rejected_with_option_context` (L4189): _no doc comment_
- `tests::pages_docx` (L4214): _no doc comment_
- `tests::file_names` (L4222): _no doc comment_
- `tests::parse_pages_counts_from_one_and_sorts_without_repeats` (L4232): _no doc comment_
- `tests::parse_pages_refuses_zero_empty_backwards_and_words` (L4240): _no doc comment_
- `tests::convert_and_diff_render_parse_their_page_flags` (L4257): _no doc comment_
- `tests::convert_job` (L4315): _no doc comment_
- `tests::convert_pages_writes_the_selected_pages_named_by_page_number` (L4340): _no doc comment_
- `tests::convert_pages_needs_png_and_an_existing_page` (L4364): _no doc comment_
- `tests::diff_job` (L4379): _no doc comment_
- `tests::diff_render_writes_only_the_changed_pages_and_diff_json` (L4392): _no doc comment_
- `tests::diff_render_of_equal_documents_writes_only_diff_json` (L4448): _no doc comment_
- `tests::diff_render_without_an_overlay_writes_no_diff_png` (L4459): _no doc comment_
- `tests::convert_takes_update_fields` (L4481): _no doc comment_
- `tests::refresh_fields_writes_the_report_and_says_when_it_cannot` (L4503): _no doc comment_
