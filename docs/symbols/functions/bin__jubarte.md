# Functions and methods in `src/bin/jubarte.rs`

128 non-test, 57 test. Part of the [symbol manifest](../README.md).

#### `stdout_line`  (L48-57)
`fn stdout_line(line: std::fmt::Arguments<'_>)`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.kind`, `.lock`, `std::io::stdout`, `std::process::exit`
- **Macros:** `eprintln!`, `writeln!`
- **Called by:** none found in src

#### `ScrubOptions::options`  (L70-70, trait decl, no body)
`fn options(&self) -> jubarte::scrub::ScrubOptions`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `cli_main` (bin/jubarte.rs:3935) (?); `tests::scrub_flags_select_what_goes_and_none_selects_everything` (bin/jubarte.rs:5005) (?)

#### `<ScrubSelection as ScrubOptions>::options`  (L74-84, impl ScrubOptions)
`fn options(&self) -> jubarte::scrub::ScrubOptions`

- **Does:** _no doc comment_
- **Calls (crate):** `<ScrubOptions as Default>::default` (scrub.rs:69) L76
- **Calls (std/external/unresolved):** `.clone`, `Self::default`
- **Called by:** `cli_main` (bin/jubarte.rs:3935) (?); `tests::scrub_flags_select_what_goes_and_none_selects_everything` (bin/jubarte.rs:5005) (?)

#### `SelectionFilter::filter`  (L88-88, trait decl, no body)
`fn filter(&self) -> jubarte::changes::ChangeFilter`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** none found in src

#### `<Selection as SelectionFilter>::filter`  (L92-111, impl SelectionFilter)
`fn filter(&self) -> jubarte::changes::ChangeFilter`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.collect`, `.is_empty`, `.iter`, `.map`, `.then`, `.to_vec`, `given`
- **Called by:** none found in src

#### `run_scrub`  (L114-124)
`fn run_scrub( file: &Path, out: &Output<'_>, options: &jubarte::scrub::ScrubOptions, ) -> Result<(), ConvertFailure>`

- **Does:** _no doc comment_
- **Calls (crate):** `check_output` (bin/jubarte.rs:345) L119; `read_word` (bin/jubarte.rs:3059) L120; `scrub` (scrub.rs:109) L122; `write_output` (bin/jubarte.rs:353) L123
- **Calls (std/external/unresolved):** `.map_err`, `Produced::Docx`
- **Macros:** `format!`
- **Called by:** `cli_main` (bin/jubarte.rs:3935)

#### `revision_style`  (L127-141)
`fn revision_style( revisions: Revisions, palette: Option<&str>, ) -> Result<jubarte::convert::RevisionStyle, String>`

- **Does:** The library's revision style for `--revisions` / `--revision-palette`.
- **Calls (crate):** `RevisionPalette::parse` (convert/mod.rs:209) L134
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`, `Err`, `Ok`, `RevisionStyle::Custom`
- **Macros:** `format!`
- **Called by:** `pdf_options` (bin/jubarte.rs:144); `cli_main` (bin/jubarte.rs:3935); `tests::convert_revisions_default_to_conventional_and_validate_the_palette` (bin/jubarte.rs:4763); `tests::regression_tests::invalid_revision_flags_are_rejected_with_option_context` (bin/jubarte.rs:5434)

#### `pdf_options`  (L144-153)
`fn pdf_options( render: &RenderArgs, compress: bool, ) -> Result<jubarte::convert::PdfOptions, String>`

- **Does:** The renderer options a [`RenderArgs`] group asks for.
- **Calls (crate):** `RenderArgs::apply` (cli.rs:2571) L148; `revision_style` (bin/jubarte.rs:127) L150
- **Calls (std/external/unresolved):** `.as_deref`, `Ok`, `jubarte::convert::PdfOptions::default`
- **Called by:** `write_output` (bin/jubarte.rs:353); `cli_main` (bin/jubarte.rs:3935); `tests::regression_tests::convert_without_revision_flags_uses_conventional_marks` (bin/jubarte.rs:5396); `tests::regression_tests::custom_revision_flags_reach_the_palette_parser` (bin/jubarte.rs:5410)

#### `absolute`  (L168-187)
`fn absolute(path: &Path) -> String`

- **Does:** The absolute form of a CLI path: the real path when it exists, else the deepest existing ancestor's real path joined with the rest.
- **Calls (crate):** `.parent` (ambiguous: `Marks::parent` (markdown/view_stories.rs:450); `Dom::parent` (xmllinq/mod.rs:597)) L177; `slash_path` (cli.rs:553) L186
- **Calls (std/external/unresolved):** `.as_path`, `.clone`, `.file_name`, `.fold`, `.iter`, `.join`, `.push`, `.rev`, `.to_os_string`, `.unwrap_or_default`, `.unwrap_or_else`, `Vec::new`, `std::env::current_dir`, `std::fs::canonicalize`
- **Called by:** `path_only` (bin/jubarte/check.rs:113) (?); `run` (bin/jubarte/check.rs:125) (?); `generate` (bin/jubarte/edit_template.rs:32) (?); `run` (bin/jubarte/edit_template.rs:199) (?); `path_only` (bin/jubarte/edit_template.rs:287) (?); `usage` (bin/jubarte/edit_template.rs:292) (?); `regeneration_action` (bin/jubarte/edit_typed.rs:111) (?); `artifact` (bin/jubarte/edit_typed.rs:378) (?); `prepare_and_publish` (bin/jubarte/edit_typed.rs:398) (?); `run` (bin/jubarte/edit_typed.rs:787) (?); `usage` (bin/jubarte/edit_typed.rs:962) (?); `refused` (bin/jubarte.rs:2697)

#### `same_path`  (L197-225)
`fn same_path(a: &Path, b: &Path) -> bool`

- **Does:** Whether two CLI paths name the same file.
- **Calls (crate):** `same_path::{fn key}` (bin/jubarte.rs:201) L224
- **Calls (std/external/unresolved):** `Some`, `same_file::is_same_file`
- **Macros:** `matches!`
- **Called by:** `generate` (bin/jubarte/edit_template.rs:32) (?); `prepare_and_publish` (bin/jubarte/edit_typed.rs:398) (?); `same_dir` (bin/jubarte.rs:229); `check_side_file` (bin/jubarte.rs:423); `run_convert` (bin/jubarte.rs:646); `refuse_output_alias` (bin/jubarte.rs:1701)

#### `same_path::{fn key}`  (L201-223)
`fn key(p: &Path) -> Option<PathBuf>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.as_os_str`, `.file_name`, `.is_empty`, `.is_relative`, `.join`, `.ok`, `.parent`, `.to_lowercase`, `.to_path_buf`, `.to_string_lossy`, `PathBuf::from`, `Some`, `std::fs::canonicalize`, `std::fs::read_link`
- **Called by:** `same_path` (bin/jubarte.rs:197); `earliest` (changes/logical.rs:383) (?); `fold_twins` (changes.rs:1280) (?); `step_h` (comparer/lcs.rs:3621) (?); `has_common_run_ge` (comparer/lcs.rs:5736) (?); `group_by_key_stable` (comparer/produce.rs:692) (?)

#### `same_dir`  (L229-234)
`fn same_dir(a: &Path, b: &Path) -> bool`

- **Does:** Two directory paths name the same existing directory.
- **Calls (crate):** `same_path` (bin/jubarte.rs:197) L232
- **Calls (std/external/unresolved):** `std::fs::canonicalize`
- **Called by:** `publish_edit` (bin/jubarte.rs:1402)

#### `ensure_writable`  (L237-242)
`fn ensure_writable(output: &Path, force: bool) -> Result<(), ConvertFailure>`

- **Does:** No-clobber contract shared by every writing subcommand.
- **Calls (crate):** `taken_output` (bin/jubarte.rs:1674) L239
- **Calls (std/external/unresolved):** `.exists`, `.into`, `Err`, `Ok`
- **Called by:** `generate` (bin/jubarte/edit_template.rs:32) (?); `writable` (bin/jubarte/edit_typed.rs:62) (?); `check_output` (bin/jubarte.rs:345); `write_output` (bin/jubarte.rs:353); `write_text` (bin/jubarte.rs:407); `check_side_file` (bin/jubarte.rs:423); `run_convert` (bin/jubarte.rs:646); `run_diff_render` (bin/jubarte.rs:828); `run_for_edit` (bin/jubarte.rs:961); `run_convert_any` (bin/jubarte.rs:3458); `run_validate` (bin/jubarte.rs:3812)

#### `report_publication`  (L244-254)
`fn report_publication( result: Result<jubarte::publication::PublishReport, jubarte::publication::PublishError>, stderr: &mut impl std::io::Write, ) -> Result<(), jubarte::publication::PublishError>`

- **Does:** _no doc comment_
- **Calls (crate):** `write_publication_warnings` (bin/jubarte.rs:256) L252
- **Calls (std/external/unresolved):** `.map`
- **Called by:** `publish_paths` (bin/jubarte.rs:262); `publish_files` (bin/jubarte.rs:1854); `tests::publication_cleanup_warnings_reach_stderr_for_both_outcomes` (bin/jubarte.rs:4513)

#### `write_publication_warnings`  (L256-260)
`fn write_publication_warnings(warnings: &[String], stderr: &mut impl std::io::Write)`

- **Does:** _no doc comment_
- **Macros:** `writeln!`
- **Called by:** `report_publication` (bin/jubarte.rs:244); `record_plan` (bin/jubarte.rs:2625)

#### `publish_paths`  (L262-270)
`fn publish_paths( files: &[(PathBuf, &[u8])], force: bool, ) -> Result<(), jubarte::publication::PublishError>`

- **Does:** _no doc comment_
- **Calls (crate):** `report_publication` (bin/jubarte.rs:244) L266; `publish_paths` (publication.rs:211) L267
- **Calls (std/external/unresolved):** `.lock`, `std::io::stderr`
- **Called by:** `prepare_and_publish` (bin/jubarte/edit_typed.rs:398) (?); `publish_file` (bin/jubarte.rs:273); `run_convert` (bin/jubarte.rs:646); `run_diff_render` (bin/jubarte.rs:828); `publish_edit` (bin/jubarte.rs:1402); `run_convert_any` (bin/jubarte.rs:3458)

#### `publish_file`  (L273-275)
`fn publish_file(path: &Path, bytes: &[u8], force: bool) -> Result<(), String>`

- **Does:** Stage complete bytes and atomically replace a single authorized output.
- **Calls (crate):** `publish_paths` (bin/jubarte.rs:262) L274
- **Calls (std/external/unresolved):** `.map_err`, `.to_path_buf`, `.to_string`
- **Called by:** `write_output` (bin/jubarte.rs:353); `write_text` (bin/jubarte.rs:407); `run_for_edit` (bin/jubarte.rs:961); `run_validate` (bin/jubarte.rs:3812); `tests::refresh_fields_writes_the_report_and_says_when_it_cannot` (bin/jubarte.rs:5748)

#### `Output::of_task`  (L299-320)
`fn of_task( kind: Format, path: &'a Path, input: &'a Path, render: &'a RenderArgs, force: bool, quiet: bool, ) -> Self`

- **Does:** The `-o` of accept, reject, append, scrub and convert: the view's header names the Word file the task did not write.
- **Calls (crate):** `slash_path` (cli.rs:553) L317
- **Calls (std/external/unresolved):** `.with_extension`, `std::sync::LazyLock::new`
- **Macros:** `format!`
- **Called by:** `task_output` (bin/jubarte.rs:324); `diff_output` (bin/jubarte.rs:3302); `run_convert_any` (bin/jubarte.rs:3458)

#### `task_output`  (L324-334)
`fn task_output<'a>( task: Task, path: &'a Path, input: &'a Path, render: &'a RenderArgs, force: bool, quiet: bool, ) -> Result<Output<'a>, String>`

- **Does:** The [`Output`] of a task whose `-o` the parser checked.
- **Calls (crate):** `output_format` (cli.rs:1996) L332; `Output::of_task` (bin/jubarte.rs:299) L333
- **Calls (std/external/unresolved):** `.unwrap_or`, `Ok`, `Some`
- **Called by:** `cli_main` (bin/jubarte.rs:3935)

#### `check_output`  (L345-350)
`fn check_output(out: &Output<'_>) -> Result<(), ConvertFailure>`

- **Does:** Refuse an existing output before any work: PNG pages are checked once their names are known.
- **Calls (crate):** `ensure_writable` (bin/jubarte.rs:237) L349
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `run_scrub` (bin/jubarte.rs:114); `run_resolution` (bin/jubarte.rs:458); `run_text` (bin/jubarte.rs:1036); `run` (bin/jubarte.rs:2862); `diff_output` (bin/jubarte.rs:3302); `run_convert_any` (bin/jubarte.rs:3458); `run_append` (bin/jubarte.rs:4433)

#### `write_output`  (L353-404)
`fn write_output(out: &Output<'_>, produced: Produced<'_>) -> Result<(), ConvertFailure>`

- **Does:** Writes `produced` as `out.kind` and says `wrote …` on stderr unless `-q`.
- **Calls (crate):** `pdf_options` (bin/jubarte.rs:144) L356; `run_convert` (bin/jubarte.rs:646) L358; `ensure_writable` (bin/jubarte.rs:237) L374; `input_confirmation_view` (bin/jubarte.rs:1171) L376; `publish_file` (bin/jubarte.rs:273) L377; `wrote_line` (cli.rs:2060) L379; `print_success` (bin/jubarte.rs:1181) L382; `agent_view` (bin/jubarte.rs:1140) L388; `read_snapshot` (markdown/mod.rs:821) L392; `snapshot_options` (bin/jubarte.rs:1096) L392; `write_text` (bin/jubarte.rs:407) L403; `Format::name` (cli.rs:1870) L403
- **Calls (std/external/unresolved):** `.as_deref`, `.clone`, `.into`, `.len`, `.map_err`, `.map`, `.name`, `.then`, `.to_string`, `.transpose`, `Err`, `Ok`, `Some`, `serde_json::to_string`
- **Macros:** `eprintln!`, `format!`
- **Called by:** `run_scrub` (bin/jubarte.rs:114); `run_resolution` (bin/jubarte.rs:458); `run_text` (bin/jubarte.rs:1036); `run` (bin/jubarte.rs:2862); `run_convert_any` (bin/jubarte.rs:3458); `run_append` (bin/jubarte.rs:4433)

#### `write_text`  (L407-418)
`fn write_text(out: &Output<'_>, text: &str, label: &str) -> Result<(), ConvertFailure>`

- **Does:** Publish text through the common writer, naming the representation written.
- **Calls (crate):** `ensure_writable` (bin/jubarte.rs:237) L408; `publish_file` (bin/jubarte.rs:273) L409
- **Calls (std/external/unresolved):** `.as_bytes`, `.display`, `.len`, `Ok`
- **Macros:** `eprintln!`
- **Called by:** `write_output` (bin/jubarte.rs:353); `print_or_write` (bin/jubarte.rs:3318)

#### `check_side_file`  (L423-439)
`fn check_side_file( side: &Path, flag: &str, others: [(&Path, &str); 2], force: bool, ) -> Result<(), ConvertFailure>`

- **Does:** A side file (`--report`, `--font-report`) is written apart from the output, after the input is read, so sharing a path with either would silently replace one with the other; `--force` never allows that.
- **Calls (crate):** `same_path` (bin/jubarte.rs:197) L430; `slash_path` (cli.rs:553) L433; `ensure_writable` (bin/jubarte.rs:237) L438
- **Calls (std/external/unresolved):** `.into`, `Err`
- **Macros:** `format!`
- **Called by:** `run_convert` (bin/jubarte.rs:646); `run_convert_any` (bin/jubarte.rs:3458)

#### `comment_changes_note`  (L443-452)
`fn comment_changes_note(left: usize) -> String`

- **Does:** The note a selection prints when it leaves tracked changes inside comment bodies. They are `comments.rN`; this selection did not name them.
- **Macros:** `format!`
- **Called by:** `run_resolution` (bin/jubarte.rs:458); `tests::comment_changes_note_agrees_with_its_count` (bin/jubarte.rs:4539)

#### `run_resolution`  (L458-507)
`fn run_resolution( file: &Path, out: &Output<'_>, filter: &jubarte::changes::ChangeFilter, apply: fn( &[u8], &jubarte::changes::ChangeFilter, ) -> Result<jubarte::changes::ResolvedChanges, jubarte::changes::ChangeError>, what: &str, ) -> ExitCode`

- **Does:** Shared body for `accept` / `reject`: read the redline, apply the package-wide resolution, and write the result under the compare path's no-clobber contract.
- **Calls (crate):** `ChangeFilter::normalized` (changes.rs:349) L473; `check_output` (bin/jubarte.rs:345) L476; `read_word` (bin/jubarte.rs:3059) L477; `apply` (ambiguous: `tests::apply` (edit/rewrite.rs:225); `apply` (markdown/from_docx/header.rs:816); `apply` (markdown/view_styles.rs:586)) L478; `write_output` (bin/jubarte.rs:353) L484; `comment_changes_note` (bin/jubarte.rs:443) L496; `run` (bin/jubarte.rs:2862) L500
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.iter`, `.map_err`, `.take`, `.to_string`, `.zip`, `ExitCode::from`, `Ok`, `Produced::Docx`, `failed`
- **Macros:** `eprintln!`, `format!`
- **Called by:** `cli_main` (bin/jubarte.rs:3935)

#### `run_changes`  (L509-525)
`fn run_changes(file: &Path, json: bool, quiet: bool) -> Result<(), String>`

- **Does:** _no doc comment_
- **Calls (crate):** `read_input_json` (bin/jubarte.rs:3099) L511; `list_logical_changes` (changes.rs:579) L512,519; `validity_line` (bin/jubarte.rs:557) L516; `read_input` (bin/jubarte.rs:3069) L518; `changes_text` (changes.rs:526) L522
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`, `.trim_end_matches`, `Ok`, `serde_json::to_string`
- **Macros:** `outln!`
- **Called by:** `cli_main` (bin/jubarte.rs:3935)

#### `run_comments`  (L527-553)
`fn run_comments( file: &Path, json: bool, author: Option<&str>, latest: bool, quiet: bool, ) -> Result<(), String>`

- **Does:** _no doc comment_
- **Calls (crate):** `read_input_json` (bin/jubarte.rs:3099) L535; `read_input` (bin/jubarte.rs:3069) L538; `list_comments` (comments.rs:132) L540; `select_comments` (comments.rs:190) L541; `comments_text` (comments.rs:164) L545; `validity_line` (bin/jubarte.rs:557) L552
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`, `.trim_end_matches`, `Ok`, `Some`, `serde_json::to_string`
- **Macros:** `outln!`
- **Called by:** `cli_main` (bin/jubarte.rs:3935)

#### `validity_line`  (L557-563)
`fn validity_line(validity: &jubarte::validate::InputValidity) -> Result<(), String>`

- **Does:** The input's validity, the last line of `changes --json` and `comments --json` (#1309).
- **Calls (crate):** `input_validity_line` (validate.rs:1101) L560
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`, `Ok`
- **Macros:** `outln!`
- **Called by:** `run_changes` (bin/jubarte.rs:509); `run_comments` (bin/jubarte.rs:527)

#### `arm_timeout`  (L593-602)
`fn arm_timeout(limit: std::time::Duration)`

- **Does:** Exit [`EXIT_TIMEOUT`] once `limit` has passed, whatever the main thread is doing (layout of a pathological document cannot be interrupted).
- **Calls (std/external/unresolved):** `.as_secs_f64`, `std::process::exit`, `std::thread::sleep`, `std::thread::spawn`
- **Macros:** `eprintln!`
- **Called by:** `cli_main` (bin/jubarte.rs:3935)

#### `<ConvertFailure as From>::from`  (L616-618, impl From)
`fn from(message: String) -> Self`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `run_for_edit` (bin/jubarte.rs:961) (?); `tests::a_plain_convert_error_keeps_exit_1` (bin/jubarte.rs:5349) (?)

#### `<ConvertFailure as From>::from`  (L622-624, impl From)
`fn from((code, message): (u8, String)) -> Self`

- **Does:** _no doc comment_
- **Calls:** none
- **Called by:** `run_for_edit` (bin/jubarte.rs:961) (?); `tests::a_plain_convert_error_keeps_exit_1` (bin/jubarte.rs:5349) (?)

#### `<ConvertFailure as From>::from`  (L628-630, impl From)
`fn from(message: &str) -> Self`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`, `.to_string`
- **Called by:** `run_for_edit` (bin/jubarte.rs:961) (?); `tests::a_plain_convert_error_keeps_exit_1` (bin/jubarte.rs:5349) (?)

#### `convert_exit_code`  (L634-643)
`fn convert_exit_code(r: Result<(), ConvertFailure>) -> ExitCode`

- **Does:** The exit status of a `convert` run: 1 for an error, or its own code.
- **Calls (crate):** `exit_code` (bin/jubarte.rs:2977) L637; `say_error` (bin/jubarte.rs:2995) L639
- **Calls (std/external/unresolved):** `Err`, `ExitCode::from`
- **Called by:** `cli_main` (bin/jubarte.rs:3935)

#### `run_convert`  (L646-777)
`fn run_convert(job: &ConvertJob<'_>) -> Result<(), ConvertFailure>`

- **Does:** Render `job` as a PDF, or as PNG pages when it asks for PNG.
- **Calls (crate):** `check_side_file` (bin/jubarte.rs:423) L657; `ensure_writable` (bin/jubarte.rs:237) L661,700; `.parent` (ambiguous: `Marks::parent` (markdown/view_stories.rs:450); `Dom::parent` (xmllinq/mod.rs:597)) L667; `read_word` (bin/jubarte.rs:3059) L670; `render` (convert/mod.rs:592) L672; `png_name` (bin/jubarte.rs:916) L692; `same_path` (bin/jubarte.rs:197) L704; `slash_path` (cli.rs:553) L707; `.to_json` (ambiguous, 5 candidates) L716; `publish_paths` (bin/jubarte.rs:262) L735; `wrote_line` (cli.rs:2060) L740; `wrote_png_line` (cli.rs:2140) L745; `FontReportEntry::substituted` (convert/font.rs:350) (by name) L753
- **Calls (std/external/unresolved):** `.as_bytes`, `.as_path`, `.clone`, `.cloned`, `.collect`, `.extend`, `.file_stem`, `.filter`, `.into_owned`, `.into`, `.is_empty`, `.iter`, `.join`, `.len`, `.map_err`, `.map`, `.push`, `.take`, `.then_some`, `.to_path_buf`, `.to_string_lossy`, `.to_string`, `.to_vec`, `.unwrap_or_default`, `.unwrap_or_else`, `.with_extension`, `.zip`, `Err`, `Ok`, `Some`, `Vec::new`, `jubarte::convert::font_report_json`
- **Macros:** `eprintln!`, `format!`
- **Called by:** `write_output` (bin/jubarte.rs:353); `run_convert_any` (bin/jubarte.rs:3458); `tests::convert_rejects_a_font_report_path_equal_to_the_pdf` (bin/jubarte.rs:5194); `tests::fail_on_substitution_is_exit_4_after_the_outputs` (bin/jubarte.rs:5315); `tests::convert_font_report_writes_json` (bin/jubarte.rs:5355); `tests::convert_pages_writes_the_selected_pages_named_by_page_number` (bin/jubarte.rs:5585); `tests::convert_pages_needs_png_and_an_existing_page` (bin/jubarte.rs:5609)

#### `parse_pages`  (L781-814)
`fn parse_pages(spec: &str) -> Result<Vec<usize>, String>`

- **Does:** `--pages` as zero-based indices, ascending and without repeats: `1-3,7` is `[0, 1, 2, 6]`.
- **Calls (crate):** `page` (ambiguous: `coverage_round_three_tests::page` (convert/mod.rs:49554); `style_hit_and_following_line_contract_tests::page` (convert/mod.rs:63843)) L799,808
- **Calls (std/external/unresolved):** `.dedup`, `.extend`, `.is_empty`, `.parse`, `.push`, `.sort_unstable`, `.split_once`, `.split`, `.trim`, `Err`, `Ok`, `Vec::new`
- **Macros:** `format!`
- **Called by:** `tests::parse_pages_counts_from_one_and_sorts_without_repeats` (bin/jubarte.rs:5477); `tests::parse_pages_refuses_zero_empty_backwards_and_words` (bin/jubarte.rs:5485)

#### `run_diff_render`  (L828-912)
`fn run_diff_render(job: &DiffRenderJob<'_>) -> Result<bool, ConvertFailure>`

- **Does:** Compare two documents page by page; `Ok(true)` when any page differs.
- **Calls (crate):** `read_word` (bin/jubarte.rs:3059) L829,830; `jubarte::convert::DiffOptions::default` (ambiguous: `<DiffOptions as Default>::default` (convert/diff_render.rs:44); `<DiffOptions as Default>::default` (debug/diff.rs:32)) L834; `.differs` (ambiguous: `PageDiff::differs` (convert/diff_render.rs:73); `RenderDiff::differs` (convert/diff_render.rs:101)) L839; `png_name` (bin/jubarte.rs:916) L874; `ensure_writable` (bin/jubarte.rs:237) L882; `publish_paths` (bin/jubarte.rs:262) L885
- **Calls (std/external/unresolved):** `.as_bytes`, `.as_ref`, `.collect`, `.display`, `.filter`, `.get`, `.is_empty`, `.is_some`, `.iter`, `.join`, `.len`, `.map_err`, `.map`, `.push`, `.to_string`, `.unwrap_or_default`, `Ok`, `Vec::new`, `jubarte::convert::diff_render`, `serde_json::to_string`, `std::fs::create_dir_all`
- **Macros:** `format!`, `outln!`
- **Called by:** `cli_main` (bin/jubarte.rs:3935); `tests::diff_render_writes_only_the_changed_pages_and_diff_json` (bin/jubarte.rs:5637); `tests::diff_render_of_equal_documents_writes_only_diff_json` (bin/jubarte.rs:5693); `tests::diff_render_without_an_overlay_writes_no_diff_png` (bin/jubarte.rs:5704)

#### `png_name`  (L916-919)
`fn png_name(stem: &str, index: usize, count: usize) -> String`

- **Does:** `<stem>-page-NN.png` for the zero-based page `index`, zero-padded to the page count's width (at least 2).
- **Calls (std/external/unresolved):** `.len`, `.max`, `.to_string`
- **Macros:** `format!`
- **Called by:** `run_convert` (bin/jubarte.rs:646); `run_diff_render` (bin/jubarte.rs:828); `publish_edit` (bin/jubarte.rs:1402); `write_edited` (bin/jubarte.rs:2216)

#### `run_read_tables`  (L925-947)
`fn run_read_tables(file: &Path, args: &ReadArgs, quiet: bool) -> Result<(), ConvertFailure>`

- **Does:** `read --tables`: each body table as a grid; with `-p` the grids of the tables the view's selection reaches, then that view (#1161), and `--track-changes reject` reads the rejected text.
- **Calls (crate):** `read_input` (bin/jubarte.rs:3069) L926; `warn_nested_text_boxes` (bin/jubarte.rs:1121) L927; `read_tables` (markdown/mod.rs:912) L928; `selection` (bin/jubarte.rs:1109) L935; `usage_failure` (bin/jubarte.rs:950) L940; `write_stdout` (bin/jubarte.rs:1240) L946
- **Calls (std/external/unresolved):** `.file_name`, `.into_owned`, `.into`, `.map_err`, `.map`, `.to_string_lossy`, `.to_string`, `.unwrap_or`, `Ok`
- **Macros:** `eprintln!`
- **Called by:** `run_text` (bin/jubarte.rs:1036)

#### `usage_failure`  (L950-955)
`fn usage_failure(message: String) -> ConvertFailure`

- **Does:** A usage error: exit 2.
- **Calls:** none
- **Called by:** `run_read_tables` (bin/jubarte.rs:925)

#### `run_for_edit`  (L961-1031)
`fn run_for_edit( file: &Path, args: &ReadArgs, output: Option<&Path>, force: bool, quiet: bool, ) -> Result<(), ConvertFailure>`

- **Does:** `read --for-edit` (V2-10): the working document template/apply derive — exact working text, `before_sha256`, paired targets and per-operation readiness — as JSON on stdout or in a `.json` file.
- **Calls (crate):** `read_input_json` (bin/jubarte.rs:3099) L1009; `input_notice_of` (validate.rs:1038) L1013; `warn_nested_text_boxes` (bin/jubarte.rs:1121) L1017; `for_edit` (edit/for_edit.rs:1581) (by name) L1018; `ConvertFailure::from` (ambiguous: `<ConvertFailure as From>::from` (bin/jubarte.rs:616); `<ConvertFailure as From>::from` (bin/jubarte.rs:622); `<ConvertFailure as From>::from` (bin/jubarte.rs:628)) L1021; `ensure_writable` (bin/jubarte.rs:237) L1023; `publish_file` (bin/jubarte.rs:273) L1024; `wrote_line` (cli.rs:2060) L1026; `write_stdout` (bin/jubarte.rs:1240) L1030
- **Calls (std/external/unresolved):** `.and_then`, `.as_bytes`, `.clone`, `.is_some`, `.len`, `.map_err`, `.to_string`, `.unwrap_or`, `Ok`, `Some`, `Vec::new`, `serde_json::to_string`
- **Macros:** `eprintln!`, `format!`
- **Called by:** `run_text` (bin/jubarte.rs:1036)

#### `run_text`  (L1036-1091)
`fn run_text( file: &Path, args: &ReadArgs, output: Option<&Path>, force: bool, quiet: bool, ) -> Result<(), ConvertFailure>`

- **Does:** `read`: the agent view, with page markers from the layout pass unless `--no-page-markers` skips it; or the snapshot (`--json`), or the grids (`--tables`).
- **Calls (crate):** `selection` (bin/jubarte.rs:1109) L1044; `run_for_edit` (bin/jubarte.rs:961) L1046; `run_read_tables` (bin/jubarte.rs:925) L1049; `output_format` (cli.rs:1996) L1054; `<RenderArgs as Default>::default` (cli.rs:2556) L1055; `check_output` (bin/jubarte.rs:345) L1067; `read_input_json` (bin/jubarte.rs:3099) L1071,1084; `input_notice_of` (validate.rs:1038) L1072; `warn_nested_text_boxes` (bin/jubarte.rs:1121) L1075,1080,1085,1089; `read_json` (bin/jubarte.rs:3082) L1076,1086; `write_output` (bin/jubarte.rs:353) L1077,1081; `read_input` (bin/jubarte.rs:3069) L1079,1088; `write_stdout` (bin/jubarte.rs:1240) L1086; `print_agent_view` (bin/jubarte.rs:1233) L1090
- **Calls (std/external/unresolved):** `.file_name`, `.into_owned`, `.map_err`, `.map`, `.to_string_lossy`, `.unwrap_or_default`, `.unwrap_or`, `Ok`, `Produced::Docx`, `Produced::Text`, `Some`
- **Macros:** `eprintln!`
- **Called by:** `cli_main` (bin/jubarte.rs:3935)

#### `snapshot_options`  (L1096-1106)
`fn snapshot_options( args: &ReadArgs, source: Option<String>, ) -> Result<jubarte::markdown::ReadOptions, String>`

- **Does:** The options `read --json` and `read -o FILE.json` read the snapshot with: the selection, and the text `--track-changes` settles.
- **Calls (crate):** `selection` (bin/jubarte.rs:1109) L1102; `<ReadOptions as Default>::default` (markdown/mod.rs:713) L1104
- **Calls (std/external/unresolved):** `.into`, `.unwrap_or`, `Ok`
- **Called by:** `write_output` (bin/jubarte.rs:353); `read_json` (bin/jubarte.rs:3082)

#### `selection`  (L1109-1117)
`fn selection(args: &ReadArgs) -> Result<Option<jubarte::markdown::Select>, String>`

- **Does:** The paragraphs `-p`, `--head`, `--tail`, `--changed` and `--by` select.
- **Calls (crate):** `Select::from_flags` (markdown/mod.rs:429) L1110
- **Calls (std/external/unresolved):** `.as_deref`
- **Called by:** `run_read_tables` (bin/jubarte.rs:925); `run_text` (bin/jubarte.rs:1036); `snapshot_options` (bin/jubarte.rs:1096); `agent_view` (bin/jubarte.rs:1140)

#### `warn_nested_text_boxes`  (L1121-1136)
`fn warn_nested_text_boxes(bytes: &[u8])`

- **Does:** Report the first Word-fatal nest once per invocation, before editing or selecting content.
- **Calls (crate):** `admit` (admission.rs:439) L1124; `<InputLimits as Default>::default` (admission.rs:64) L1124; `PartFs::open` (opc/mod.rs:585) L1127; `ring1` (validate.rs:1165) L1130
- **Calls (std/external/unresolved):** `.find`, `.into_iter`, `.is_err`
- **Macros:** `eprintln!`
- **Called by:** `run` (bin/jubarte/edit_template.rs:199); `run` (bin/jubarte/edit_typed.rs:787); `run_read_tables` (bin/jubarte.rs:925); `run_for_edit` (bin/jubarte.rs:961); `run_text` (bin/jubarte.rs:1036); `run_flags` (bin/jubarte.rs:2511); `run_convert_any` (bin/jubarte.rs:3458)

#### `agent_view`  (L1140-1162)
`fn agent_view( bytes: &[u8], source: Option<String>, args: &ReadArgs, ) -> Result<jubarte::markdown::ReadView, String>`

- **Does:** The agent view of `bytes`, its warnings said on stderr; `source` is the name the header prints.
- **Calls (crate):** `selection` (bin/jubarte.rs:1109) L1145; `read` (markdown/mod.rs:767) L1146
- **Calls (std/external/unresolved):** `.into`, `.map_err`, `.to_string`, `.unwrap_or`, `Ok`
- **Macros:** `eprintln!`
- **Called by:** `write_output` (bin/jubarte.rs:353); `confirmation_view` (bin/jubarte.rs:1165); `print_agent_view` (bin/jubarte.rs:1233)

#### `confirmation_view`  (L1165-1168)
`fn confirmation_view(path: &Path, bytes: &[u8]) -> Result<String, String>`

- **Does:** Use the same default view and source basename as `read PATH`.
- **Calls (crate):** `agent_view` (bin/jubarte.rs:1140) L1167
- **Calls (std/external/unresolved):** `.file_name`, `.into_owned`, `.map`, `.to_string_lossy`, `ReadArgs::default`
- **Called by:** `input_confirmation_view` (bin/jubarte.rs:1171); `print_success` (bin/jubarte.rs:1181); `print_edit_success` (bin/jubarte.rs:1208); `run_convert_any` (bin/jubarte.rs:3458)

#### `input_confirmation_view`  (L1171-1177)
`fn input_confirmation_view(path: &Path, markdown_input: bool) -> Result<String, String>`

- **Does:** Markdown inputs are compared exactly as supplied, without a view header.
- **Calls (crate):** `read_document` (bin/jubarte.rs:3038) L1172; `confirmation_view` (bin/jubarte.rs:1165) L1176
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`, `String::from_utf8`
- **Called by:** `write_output` (bin/jubarte.rs:353)

#### `print_success`  (L1181-1205)
`fn print_success( before_path: &Path, before: Result<String, String>, after_path: &Path, published: &[u8], ) -> Result<(), String>`

- **Does:** Render the exact buffer that was published, never an edit plan.
- **Calls (crate):** `confirmation_view` (bin/jubarte.rs:1165) L1188; `edit_status` (bin/jubarte.rs:1781) L1192; `success_block` (cli.rs:2070) L1198; `slash_path` (cli.rs:553) L1199,1201; `write_stdout` (bin/jubarte.rs:1240) L1204
- **Calls (std/external/unresolved):** `.and_then`, `.map`, `Ok`
- **Macros:** `format_args!`
- **Called by:** `write_output` (bin/jubarte.rs:353); `print_edit_success` (bin/jubarte.rs:1208); `run_convert_any` (bin/jubarte.rs:3458)

#### `print_edit_success`  (L1208-1230)
`fn print_edit_success( before_path: &Path, source: &[u8], after_path: &Path, published: &[u8], redacted: bool, ) -> Result<(), String>`

- **Does:** A redaction's original view would repeat the text we just removed.
- **Calls (crate):** `confirmation_view` (bin/jubarte.rs:1165) L1216,1226; `write_stdout` (bin/jubarte.rs:1240) L1217; `edit_status` (bin/jubarte.rs:1781) L1219; `print_success` (bin/jubarte.rs:1181) L1224
- **Calls (std/external/unresolved):** `Ok`
- **Macros:** `format!`, `format_args!`
- **Called by:** `run` (bin/jubarte/edit_typed.rs:787) (?); `publish_edit` (bin/jubarte.rs:1402); `write_edited` (bin/jubarte.rs:2216)

#### `print_agent_view`  (L1233-1235)
`fn print_agent_view(bytes: &[u8], source: Option<String>, args: &ReadArgs) -> Result<(), String>`

- **Does:** The agent view of `bytes` on stdout.
- **Calls (crate):** `write_stdout` (bin/jubarte.rs:1240) L1234; `agent_view` (bin/jubarte.rs:1140) L1234
- **Called by:** `run_text` (bin/jubarte.rs:1036); `run` (bin/jubarte.rs:2862)

#### `write_stdout`  (L1240-1249)
`fn write_stdout(text: impl AsRef<[u8]>) -> Result<(), String>`

- **Does:** Writes a view, a conversion or a diff to stdout.
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.flush`, `.kind`, `.lock`, `.write_all`, `Err`, `Ok`, `std::io::stdout`
- **Macros:** `format!`
- **Called by:** `finish` (bin/jubarte/check.rs:30) (?); `run` (bin/jubarte/edit_template.rs:199) (?); `usage` (bin/jubarte/edit_template.rs:292) (?); `run` (bin/jubarte/edit_typed.rs:787) (?); `usage` (bin/jubarte/edit_typed.rs:962) (?); `run_read_tables` (bin/jubarte.rs:925); `run_for_edit` (bin/jubarte.rs:961); `run_text` (bin/jubarte.rs:1036); `print_success` (bin/jubarte.rs:1181); `print_edit_success` (bin/jubarte.rs:1208); `print_agent_view` (bin/jubarte.rs:1233); `run_edit` (bin/jubarte.rs:1341); +8 more

#### `edit_plan`  (L1304-1336)
`fn edit_plan( verb: jubarte::edit::flags::Verb, plan: Option<&Path>, operations: &[FlagOp], options: &EditOptions, source: &[u8], ) -> Result<PlanInput, (u8, String)>`

- **Does:** The plan of an `edit` or `add` invocation: `--plan`'s file, or the plan the operation flags describe.
- **Calls (crate):** `EditPlan::from_json_with_notes` (edit.rs:1805) L1314; `plan_from_flags` (edit/flags.rs:85) L1322; `ExistingArg::plan_value` (cli.rs:398) L1327
- **Calls (std/external/unresolved):** `.as_deref`, `.display`, `.map_err`, `.to_string`, `Ok`, `Vec::new`, `std::fs::read_to_string`
- **Macros:** `format!`
- **Called by:** `run_flags` (bin/jubarte.rs:2511)

#### `run_edit`  (L1341-1399)
`fn run_edit( job: &EditJob<'_>, plan: &jubarte::edit::EditPlan, source: &[u8], notes: &[String], plan_notes: &[String], ) -> Result<usize, (u8, String)>`

- **Does:** Prepare `plan` on `source` and publish it as `job` says.
- **Calls (crate):** `<PrepareOptions as Default>::default` (edit/prepared.rs:158) L1361; `refused` (bin/jubarte.rs:2697) L1369; `EditError::with_revision_syntax` (edit/refusal.rs:147) (by name) L1369; `preview_plan_with` (edit.rs:2380) L1373; `write_stdout` (bin/jubarte.rs:1240) L1375; `EditReport::to_jsonl` (edit.rs:1246) (by name) L1375; `taken_edit_output` (bin/jubarte.rs:2415) L1380; `edit_status` (bin/jubarte.rs:1781) L1381,1383,1387,1389; `slash_path` (cli.rs:553) L1385; `next_command` (bin/jubarte.rs:2475) L1387,1389; `publish_edit` (bin/jubarte.rs:1402) L1398
- **Calls (std/external/unresolved):** `.cloned`, `.is_some`, `.iter`, `.len`, `.map_err`, `.map`, `.splice`, `Ok`, `jubarte::edit::prepare`
- **Macros:** `format_args!`
- **Called by:** `run_flags` (bin/jubarte.rs:2511)

#### `publish_edit`  (L1402-1656)
`fn publish_edit( job: &EditJob<'_>, plan: &jubarte::edit::EditPlan, source: &[u8], notes: &[String], plan_notes: &[String], prepared: jubarte::edit::PreparedEdit, ) -> Result<(), (u8, String)>`

- **Does:** Write what `prepared` holds where `job` says, with its status lines.
- **Calls (crate):** `source_sha256` (inspect.rs:299) L1417,1562; `fixes_revisions` (bin/jubarte.rs:2391) L1427; `write_edited` (bin/jubarte.rs:2216) L1454; `.parent` (ambiguous: `Marks::parent` (markdown/view_stories.rs:450); `Dom::parent` (xmllinq/mod.rs:597)) L1458; `same_dir` (bin/jubarte.rs:229) L1463; `EditReport::to_jsonl` (edit.rs:1246) (by name) L1469; `render` (convert/mod.rs:592) L1489; `RenderReport::page_starts` (convert/mod.rs:499) (by name) L1497; `insert_before_summary` (bin/jubarte.rs:2745) L1505,1566; `.render` (ambiguous, 4 candidates) L1518; `png_name` (bin/jubarte.rs:916) L1554; `slash_path` (cli.rs:553) L1565,1608; `job_inputs` (bin/jubarte.rs:1685) L1572; `refuse_output_alias` (bin/jubarte.rs:1701) L1574,1596; `refuse_bundle_input_containment` (bin/jubarte.rs:1758) L1576,1598; `refuse_output_collisions` (bin/jubarte.rs:1746) L1578,1599; `taken_output` (bin/jubarte.rs:1674) L1580; `refuse_foreign_bundle_contents` (bin/jubarte.rs:1919) L1583,1601; `stage_bundle_dir` (bin/jubarte.rs:1977) L1585; `publish_paths` (bin/jubarte.rs:262) L1590; `publish_dir` (bin/jubarte.rs:2019) L1594; `recheck_edit_source` (bin/jubarte.rs:1730) L1603; `skipped_status` (bin/jubarte.rs:2372) L1616; `is_pure_resolution` (bin/jubarte.rs:2209) L1620; `write_stdout` (bin/jubarte.rs:1240) L1621; `edit_status` (bin/jubarte.rs:1781) L1625,1637; `written_status` (bin/jubarte.rs:2362) L1635; `edit_notes` (bin/jubarte.rs:2171) L1636; `print_edit_success` (bin/jubarte.rs:1208) L1641
- **Calls (std/external/unresolved):** `.any`, `.as_os_str`, `.as_path`, `.as_ref`, `.as_slice`, `.as_str`, `.clone`, `.cloned`, `.collect`, `.display`, `.enumerate`, `.extend`, `.extension`, `.file_name`, `.filter`, `.insert`, `.into_bytes`, `.into_iter`, `.into_owned`, `.into`, `.is_empty`, `.is_ok`, `.is_some_and`, `.iter`, `.join`, `.last`, `.len`, `.lines`, `.map_err`, `.map_or_else`, `.map`, `.push`, `.splice`, `.symlink_metadata`, `.then_some`, `.to_string_lossy`, `.to_string`, `.unwrap_or`, `Err`, `Ok`, `Path::new`, `Some`, `Vec::new`, `fail`, `jubarte::convert::PdfOptions::default`, `jubarte::edit::failed_plan_json`, `jubarte::edit::keep_notes`, `patch_of`, `serde_json::Map::new`, `std::fs::remove_dir_all`
- **Macros:** `env!`, `format!`, `format_args!`, `serde_json::json!`, `vec!`
- **Called by:** `run_edit` (bin/jubarte.rs:1341)

#### `edit_envelope_exit`  (L1659-1669)
`fn edit_envelope_exit(receipt: &jubarte::result::Envelope) -> u8`

- **Does:** Keep typed edit/template output refusals on the CLI's usage exit code.
- **Calls (crate):** `exit_status_for_envelope` (result.rs:1234) L1667
- **Calls (std/external/unresolved):** `.as_ref`, `.into`, `.is_some_and`
- **Called by:** `run` (bin/jubarte/edit_template.rs:199) (?); `run` (bin/jubarte/edit_typed.rs:787) (?)

#### `taken_output`  (L1674-1682)
`fn taken_output(path: &Path) -> (u8, String)`

- **Does:** An existing output without --force is a usage refusal: exit 2.
- **Calls (crate):** `slash_path` (cli.rs:553) L1679
- **Macros:** `format!`
- **Called by:** `ensure_writable` (bin/jubarte.rs:237); `publish_edit` (bin/jubarte.rs:1402); `publish_files` (bin/jubarte.rs:1854); `publish_dir` (bin/jubarte.rs:2019); `write_edited` (bin/jubarte.rs:2216)

#### `job_inputs`  (L1685-1691)
`fn job_inputs<'a>(job: &EditJob<'a>) -> Vec<(&'a Path, &'static str)>`

- **Does:** The inputs an output of `edit` or `add` must never overwrite (V2-05).
- **Calls (std/external/unresolved):** `.push`
- **Macros:** `vec!`
- **Called by:** `publish_edit` (bin/jubarte.rs:1402); `refuse_edit_output_alias` (bin/jubarte.rs:1719)

#### `refuse_output_alias`  (L1701-1715)
`fn refuse_output_alias(output: &Path, inputs: &[(&Path, &str)]) -> Result<(), (u8, String)>`

- **Does:** Exit 2, stable code `OUTPUT_ALIAS`: an output that names an input through another spelling, a symlink or a hard link, refused before anything is written (V2-05).
- **Calls (crate):** `same_path` (bin/jubarte.rs:197) L1703; `slash_path` (cli.rs:553) L1708,1709
- **Calls (std/external/unresolved):** `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `alias` (bin/jubarte/edit_template.rs:29) (?); `generate` (bin/jubarte/edit_template.rs:32) (?); `alias` (bin/jubarte/edit_typed.rs:59) (?); `prepare_and_publish` (bin/jubarte/edit_typed.rs:398) (?); `publish_edit` (bin/jubarte.rs:1402); `refuse_edit_output_alias` (bin/jubarte.rs:1719); `refuse_output_collisions` (bin/jubarte.rs:1746)

#### `refuse_edit_output_alias`  (L1719-1726)
`fn refuse_edit_output_alias(job: &EditJob<'_>, output: &Path) -> Result<(), (u8, String)>`

- **Does:** --force authorizes an output replacement, never an input alias.
- **Calls (crate):** `job_inputs` (bin/jubarte.rs:1685) L1721; `refuse_output_alias` (bin/jubarte.rs:1701) L1725
- **Calls (std/external/unresolved):** `.collect`, `.filter`, `.into_iter`
- **Called by:** `write_edited` (bin/jubarte.rs:2216)

#### `recheck_edit_source`  (L1730-1743)
`fn recheck_edit_source(file: &Path, source: &[u8]) -> Result<(), (u8, String)>`

- **Does:** Compare the exact preparation bytes with a fresh read of the named source.
- **Calls (crate):** `source_sha256` (inspect.rs:299) L1733; `slash_path` (cli.rs:553) L1738
- **Calls (std/external/unresolved):** `.display`, `.map_err`, `Err`, `Ok`, `std::fs::read`
- **Macros:** `format!`
- **Called by:** `generate` (bin/jubarte/edit_template.rs:32) (?); `prepare_and_publish` (bin/jubarte/edit_typed.rs:398) (?); `publish_edit` (bin/jubarte.rs:1402); `write_edited` (bin/jubarte.rs:2216)

#### `refuse_output_collisions`  (L1746-1755)
`fn refuse_output_collisions(paths: &[PathBuf]) -> Result<(), (u8, String)>`

- **Does:** No two roles of a publication may resolve to the same output file.
- **Calls (crate):** `refuse_output_alias` (bin/jubarte.rs:1701) L1752
- **Calls (std/external/unresolved):** `.as_path`, `.collect`, `.enumerate`, `.iter`, `.map`, `Ok`
- **Called by:** `publish_edit` (bin/jubarte.rs:1402); `publish_files` (bin/jubarte.rs:1854); `write_edited` (bin/jubarte.rs:2216); `cli_main` (bin/jubarte.rs:3935)

#### `refuse_bundle_input_containment`  (L1758-1777)
`fn refuse_bundle_input_containment( dir: &Path, inputs: &[(&Path, &str)], ) -> Result<(), (u8, String)>`

- **Does:** A wholesale directory replacement must never remove an input's name.
- **Calls (crate):** `slash_path` (cli.rs:553) L1769,1770
- **Calls (std/external/unresolved):** `.is_ok_and`, `.starts_with`, `Err`, `Ok`, `std::fs::canonicalize`
- **Macros:** `format!`
- **Called by:** `prepare_and_publish` (bin/jubarte/edit_typed.rs:398) (?); `publish_edit` (bin/jubarte.rs:1402)

#### `edit_status`  (L1781-1784)
`fn edit_status(line: std::fmt::Arguments<'_>)`

- **Does:** Post-commit status is best effort: a closed stderr cannot undo a commit or prevent its result from reaching stdout.
- **Calls (std/external/unresolved):** `.lock`, `std::io::stderr`
- **Macros:** `writeln!`
- **Called by:** `run` (bin/jubarte/edit_typed.rs:787) (?); `print_success` (bin/jubarte.rs:1181); `print_edit_success` (bin/jubarte.rs:1208); `run_edit` (bin/jubarte.rs:1341); `publish_edit` (bin/jubarte.rs:1402); `publish_dir` (bin/jubarte.rs:2019); `write_edited` (bin/jubarte.rs:2216); `written_status` (bin/jubarte.rs:2362); `skipped_status` (bin/jubarte.rs:2372); `run_flags` (bin/jubarte.rs:2511)

#### `comma_paths`  (L1787-1793)
`fn comma_paths(paths: &[PathBuf]) -> String`

- **Does:** `paths` comma-separated, with the `/` spelling the views use.
- **Calls (crate):** `slash_path` (cli.rs:553) L1790
- **Calls (std/external/unresolved):** `.collect`, `.iter`, `.join`, `.map`
- **Called by:** `publish_files` (bin/jubarte.rs:1854); `write_edited` (bin/jubarte.rs:2216)

#### `staging_path`  (L1798-1807)
`fn staging_path(dest: &Path, n: usize) -> PathBuf`

- **Does:** A staging path beside `dest`: the same filesystem, so the commit is a rename; hidden; unique to this process (`n` breaks a tie with the survivor of an interrupted run).
- **Calls (std/external/unresolved):** `.as_os_str`, `.file_name`, `.filter`, `.is_empty`, `.join`, `.parent`, `.push`, `.unwrap_or`, `Path::new`, `std::ffi::OsStr::new`, `std::ffi::OsString::from`, `std::process::id`
- **Macros:** `format!`
- **Called by:** `stage_bundle_dir` (bin/jubarte.rs:1977); `publish_dir` (bin/jubarte.rs:2019); `tests::bundle_restore_failure` (bin/jubarte.rs:4550)

#### `publication_test_hook`  (L1815-1827)
`fn publication_test_hook(phase: &str) -> Result<(), String>`

- **Does:** Test-only synchronization for filesystem races in debug Unix builds.
- **Calls (std/external/unresolved):** `.map_err`, `.read_exact`, `.set_read_timeout`, `Ok`, `Some`, `hook`, `std::env::var_os`, `std::os::unix::net::UnixStream::connect`, `std::process::id`, `std::time::Duration::from_secs`
- **Macros:** `format!`, `writeln!`
- **Called by:** `publish_files` (bin/jubarte.rs:1854); `publish_dir` (bin/jubarte.rs:2019)

#### `publication_test_hook`  (L1830-1832)
`fn publication_test_hook(_phase: &str) -> Result<(), String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `publish_files` (bin/jubarte.rs:1854); `publish_dir` (bin/jubarte.rs:2019)

#### `publish_files`  (L1854-1911)
`fn publish_files( outputs: &[(PathBuf, &[u8])], recheck: &Recheck<'_>, ) -> Result<Vec<PathBuf>, (u8, String)>`

- **Does:** Stage every output before committing any of them.
- **Calls (crate):** `StagedFile::prepare` (publication.rs:299) L1863; `publication_test_hook` (bin/jubarte.rs:1815) L1867,1890; `refuse_output_collisions` (bin/jubarte.rs:1746) L1872; `taken_output` (bin/jubarte.rs:1674) L1883,1894; `report_publication` (bin/jubarte.rs:244) L1891; `StagedFile::commit` (publication.rs:317) (by name) L1891; `comma_paths` (bin/jubarte.rs:1787) L1906
- **Calls (std/external/unresolved):** `.clone`, `.cloned`, `.collect`, `.display`, `.drain`, `.extend`, `.find`, `.is_empty`, `.is_ok`, `.iter`, `.kind`, `.lock`, `.map_err`, `.map`, `.push`, `.symlink_metadata`, `Err`, `Ok`, `Vec::new`, `drop`, `publish`, `std::io::stderr`
- **Macros:** `format!`
- **Called by:** `generate` (bin/jubarte/edit_template.rs:32) (?); `prepare_and_publish` (bin/jubarte/edit_typed.rs:398) (?); `write_edited` (bin/jubarte.rs:2216)

#### `refuse_foreign_bundle_contents`  (L1919-1932)
`fn refuse_foreign_bundle_contents(dir: &Path) -> Result<(), (u8, String)>`

- **Does:** Exit 2, stable code `OUTPUT_DIR_NOT_EMPTY`: `--force` replaces an existing `--out-dir` wholesale only when it is an earlier jubarte bundle, every entry named by its `report.jsonl` save line (V2-05), or when it is empty (no earlier bundle/report is required).
- **Calls (crate):** `foreign_bundle_entries` (bin/jubarte.rs:1942) L1920
- **Calls (std/external/unresolved):** `.display`, `.is_empty`, `.join`, `Err`, `Ok`
- **Macros:** `format!`
- **Called by:** `publish_edit` (bin/jubarte.rs:1402)

#### `foreign_bundle_entries`  (L1942-1973)
`fn foreign_bundle_entries(dir: &Path) -> Vec<String>`

- **Does:** The entries of `dir` no earlier bundle's `report.jsonl` names, sorted, a directory with a trailing `/`; none when `dir` does not exist.
- **Calls (std/external/unresolved):** `.as_array`, `.as_str`, `.cloned`, `.collect`, `.contains`, `.extend`, `.file_name`, `.file_type`, `.filter_map`, `.filter`, `.flat_map`, `.flatten`, `.into_iter`, `.into_owned`, `.into`, `.is_dir`, `.is_ok_and`, `.join`, `.kind`, `.lines`, `.map`, `.ok`, `.sort`, `.to_string_lossy`, `.to_string`, `.unwrap_or_default`, `Vec::new`, `serde_json::from_str`, `std::fs::read_dir`, `std::fs::read_to_string`
- **Macros:** `format!`, `vec!`
- **Called by:** `refuse_foreign_bundle_contents` (bin/jubarte.rs:1919); `taken_edit_output` (bin/jubarte.rs:2415); `tests::an_unreadable_bundle_is_foreign` (bin/jubarte.rs:4494)

#### `stage_bundle_dir`  (L1977-1997)
`fn stage_bundle_dir(dir: &Path) -> Result<PathBuf, String>`

- **Does:** The sibling temp directory a bundle is staged in (V2-05).
- **Calls (crate):** `staging_path` (bin/jubarte.rs:1798) L1983
- **Calls (std/external/unresolved):** `.as_os_str`, `.create`, `.display`, `.filter`, `.is_empty`, `.kind`, `.map_err`, `.mode`, `.parent`, `Err`, `Ok`, `std::fs::DirBuilder::new`, `std::fs::create_dir_all`
- **Macros:** `format!`
- **Called by:** `prepare_and_publish` (bin/jubarte/edit_typed.rs:398) (?); `publish_edit` (bin/jubarte.rs:1402); `tests::bundle_restore_failure` (bin/jubarte.rs:4550)

#### `sync_bundle_parent`  (L1999-2013)
`fn sync_bundle_parent(dir: &Path) -> Result<(), (u8, String)>`

- **Does:** _no doc comment_
- **Calls (crate):** `sync_directory` (publication.rs:409) L2004; `slash_path` (cli.rs:553) L2009
- **Calls (std/external/unresolved):** `.as_os_str`, `.filter`, `.is_empty`, `.map_err`, `.parent`, `.unwrap_or`, `Path::new`
- **Macros:** `format!`
- **Called by:** `publish_dir` (bin/jubarte.rs:2019)

#### `publish_dir`  (L2019-2117)
`fn publish_dir( stage: &Path, dir: &Path, force: bool, recheck: impl Fn() -> Result<(), (u8, String)>, ) -> Result<(), (u8, String)>`

- **Does:** Publish the staged bundle as `dir` (V2-05): a rename when `dir` is free; under --force over an existing directory, a swap that replaces it wholesale — never a merge into its files.
- **Calls (crate):** `publication_test_hook` (bin/jubarte.rs:1815) L2029; `sync_bundle_parent` (bin/jubarte.rs:1999) L2034,2100; `taken_output` (bin/jubarte.rs:1674) L2039; `staging_path` (bin/jubarte.rs:1798) L2052; `restore` (ambiguous: `restore` (edit/hint.rs:285); `restore` (edit/ranges.rs:432); `restore` (markdown/critic.rs:1861)) L2095,2098; `edit_status` (bin/jubarte.rs:1781) L2111
- **Calls (std/external/unresolved):** `.display`, `.file_type`, `.is_dir`, `.is_err`, `.is_ok_and`, `.is_ok`, `.is_symlink`, `.kind`, `.map_err`, `.ok_or_else`, `.symlink_metadata`, `Err`, `Ok`, `Some`, `failed`, `recheck`, `std::fs::remove_dir_all`, `std::fs::remove_file`, `std::fs::rename`
- **Macros:** `format!`, `format_args!`, `matches!`
- **Called by:** `prepare_and_publish` (bin/jubarte/edit_typed.rs:398) (?); `publish_edit` (bin/jubarte.rs:1402); `tests::bundle_restore_failure` (bin/jubarte.rs:4550)

#### `settlement_notes`  (L2120-2165)
`fn settlement_notes( removed: &[jubarte::edit::RemovedComment], kept: &[jubarte::edit::KeptComment], document: &[u8], decision: &str, ) -> Vec<String>`

- **Does:** One receipt line per thread, shared by legacy and typed edit publication.
- **Calls (crate):** `list_comments` (comments.rs:132) L2150
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.clone`, `.find`, `.is_empty`, `.iter`, `.join`, `.push`, `.unwrap_or_default`, `.unwrap_or`, `String::new`, `Vec::new`, `reply_text`
- **Macros:** `format!`
- **Called by:** `run` (bin/jubarte/edit_typed.rs:787); `edit_notes` (bin/jubarte.rs:2171)

#### `edit_notes`  (L2171-2206)
`fn edit_notes( notes: &[String], result: &jubarte::edit::EditResult, published: &[u8], ) -> Vec<String>`

- **Does:** The `note:` lines of an applied edit: the flags' notes, the plan's (a field read as another), then each anchor that matched as the paragraph reads rather than as typed.
- **Calls (crate):** `settlement_notes` (bin/jubarte.rs:2120) L2186
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.extend`, `.is_empty`, `.iter`, `.map`, `.push`, `jubarte::markdown::plain_anchor`
- **Macros:** `format!`
- **Called by:** `publish_edit` (bin/jubarte.rs:1402); `write_edited` (bin/jubarte.rs:2216)

#### `is_pure_resolution`  (L2209-2213)
`fn is_pure_resolution(plan: &jubarte::edit::EditPlan, result: &jubarte::edit::EditResult) -> bool`

- **Does:** A pure revision resolution keeps its existing absence of a bundle summary.
- **Calls (crate):** `ResolvedRevisions::is_empty` (edit.rs:271) L2212
- **Calls (std/external/unresolved):** `.is_empty`, `.is_some`
- **Called by:** `publish_edit` (bin/jubarte.rs:1402)

#### `write_edited`  (L2216-2350)
`fn write_edited( job: &EditJob<'_>, path: &Path, result: &jubarte::edit::EditResult, failed: Option<&FailedPlan>, notes: &[String], source: &[u8], ) -> Result<(), (u8, String)>`

- **Does:** Publish edit/add outputs, then confirm the document with full agent views.
- **Calls (crate):** `render` (convert/mod.rs:592) L2248; `png_name` (bin/jubarte.rs:916) L2270; `refuse_edit_output_alias` (bin/jubarte.rs:1719) L2278,2298; `refuse_output_collisions` (bin/jubarte.rs:1746) L2280; `taken_output` (bin/jubarte.rs:1674) L2290; `publish_files` (bin/jubarte.rs:1854) L2294; `recheck_edit_source` (bin/jubarte.rs:1730) L2299; `skipped_status` (bin/jubarte.rs:2372) L2304; `edit_status` (bin/jubarte.rs:1781) L2307,2317,2324,2331; `wrote_line` (cli.rs:2060) L2309,2319; `wrote_png_line` (cli.rs:2140) L2326; `written_status` (bin/jubarte.rs:2362) L2329; `edit_notes` (bin/jubarte.rs:2171) L2330; `print_edit_success` (bin/jubarte.rs:1208) L2333; `comma_paths` (bin/jubarte.rs:1787) L2347
- **Calls (std/external/unresolved):** `.any`, `.as_os_str`, `.as_path`, `.as_slice`, `.clone`, `.collect`, `.display`, `.enumerate`, `.file_stem`, `.filter`, `.find`, `.into_owned`, `.is_empty`, `.is_ok`, `.iter`, `.join`, `.len`, `.map_err`, `.map_or_else`, `.map_or`, `.map`, `.parent`, `.push`, `.symlink_metadata`, `.then_some`, `.to_path_buf`, `.to_string_lossy`, `.to_string`, `.with_extension`, `Err`, `Ok`, `Some`, `fail`, `jubarte::convert::PdfOptions::default`
- **Macros:** `format!`, `format_args!`, `vec!`
- **Called by:** `publish_edit` (bin/jubarte.rs:1402)

#### `written_status`  (L2362-2367)
`fn written_status(result: &jubarte::edit::EditResult, failed: Option<(&Path, &[String])>)`

- **Does:** The status lines after a write: the written document's SHA-256, then what `--skip-failed` left out and where its plan went.
- **Calls (crate):** `edit_status` (bin/jubarte.rs:1781) L2364; `skipped_status` (bin/jubarte.rs:2372) L2366
- **Macros:** `format_args!`
- **Called by:** `publish_edit` (bin/jubarte.rs:1402); `write_edited` (bin/jubarte.rs:2216)

#### `skipped_status`  (L2372-2386)
`fn skipped_status(result: &jubarte::edit::EditResult, failed: Option<(&Path, &[String])>)`

- **Does:** The skip line of `--skip-failed`, a warning even under `-q`: operations asked for are not in the document.
- **Calls (crate):** `edit_status` (bin/jubarte.rs:1781) L2379; `slash_path` (cli.rs:553) L2383
- **Calls (std/external/unresolved):** `.is_empty`, `.join`, `.len`, `String::new`
- **Macros:** `format!`, `format_args!`
- **Called by:** `publish_edit` (bin/jubarte.rs:1402); `write_edited` (bin/jubarte.rs:2216); `written_status` (bin/jubarte.rs:2362)

#### `fixes_revisions`  (L2391-2399)
`fn fixes_revisions( job: &EditJob<'_>, plan: &jubarte::edit::EditPlan, ) -> jubarte::edit::ExistingRevisions`

- **Does:** The `existing_revisions` of the fixes plan: under suggesting mode the document written is a redline, whose tracked changes are the plan's own, so the fixes keep them.
- **Calls:** none
- **Called by:** `publish_edit` (bin/jubarte.rs:1402)

#### `taken_edit_output`  (L2415-2456)
`fn taken_edit_output(job: &EditJob<'_>) -> Option<Taken>`

- **Does:** The output of `job` already on disk, if any: the document or its PDF, or the bundle directory, empty or not (an existing --out-dir is refused without --force).
- **Calls (crate):** `slash_path` (cli.rs:553) L2432,2443,2450; `foreign_bundle_entries` (bin/jubarte.rs:1942) L2439
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.filter`, `.find`, `.flatten`, `.into_iter`, `.is_dir`, `.is_empty`, `.is_ok_and`, `.is_ok`, `.iter`, `.join`, `.map`, `.next`, `.ok`, `.symlink_metadata`, `.then`, `.with_extension`, `Some`, `Taken::Blocked`, `Taken::Replaceable`
- **Macros:** `format!`
- **Called by:** `run_edit` (bin/jubarte.rs:1341)

#### `failed_plan_path`  (L2461-2470)
`fn failed_plan_path(plan: Option<&Path>, file: &Path) -> PathBuf`

- **Does:** Where `--skip-failed` writes the failed operations: beside the plan as `PLAN.failed.json`, or beside FILE as `FILE_plan.failed.json` (the name `--plan` records under, D16) when operation flags made the plan.
- **Calls (std/external/unresolved):** `.file_stem`, `.into_owned`, `.map_or_else`, `.to_string_lossy`, `.with_file_name`, `stem`
- **Macros:** `format!`
- **Called by:** `run_flags` (bin/jubarte.rs:2511)

#### `next_command`  (L2475-2484)
`fn next_command(force: bool) -> String`

- **Does:** This invocation without its `--dry-run` flag (`--force` added when `force`), quoted for a POSIX shell: what a clean dry run prints as its `next:` command.
- **Calls (crate):** `next_command` (cli.rs:3413) L2480
- **Calls (std/external/unresolved):** `.chain`, `.collect`, `.into_owned`, `.join`, `.map`, `.next`, `.to_string_lossy`, `.unwrap_or_default`, `jubarte::cli::shell_word`, `std::env::args_os`, `std::iter::once`
- **Called by:** `run_edit` (bin/jubarte.rs:1341)

#### `run_flags`  (L2511-2618)
`fn run_flags(flags: &EditFlags<'_>) -> Result<(), (u8, String)>`

- **Does:** _no doc comment_
- **Calls (crate):** `read_word` (bin/jubarte.rs:3059) L2512; `warn_nested_text_boxes` (bin/jubarte.rs:1121) L2518; `record_plan` (bin/jubarte.rs:2625) L2528; `edit_plan` (bin/jubarte.rs:1304) L2553; `editing_mode_conflict` (edit/flags.rs:650) L2562; `EditOptions::target` (cli.rs:358) L2566; `failed_plan_path` (bin/jubarte.rs:2461) L2580; `run_edit` (bin/jubarte.rs:1341) L2589; `edit_status` (bin/jubarte.rs:1781) L2593; `slash_path` (cli.rs:553) L2595; `kept` (ambiguous: `kept` (edit/rewrite.rs:18); `kept` (markdown/from_docx/revise.rs:49)) L2611,2615
- **Calls (std/external/unresolved):** `.as_ref`, `.display`, `.exists`, `.filter`, `.into`, `.is_empty`, `.is_ok_and`, `.is_some`, `.is_symlink`, `.len`, `.map_err`, `.ok`, `.push_str`, `.symlink_metadata`, `.then`, `.to_str`, `.to_string`, `Err`, `Ok`, `Some`, `std::fs::read`, `std::fs::remove_file`
- **Macros:** `format!`, `format_args!`, `matches!`
- **Called by:** `cli_main` (bin/jubarte.rs:3935)

#### `record_plan`  (L2625-2684)
`fn record_plan(flags: &EditFlags<'_>, path: &Path, source: &[u8]) -> Result<(), (u8, String)>`

- **Does:** `--plan PLAN.json` with operation flags: the operations go into the plan (created on first use) instead of into FILE.
- **Calls (crate):** `record` (edit/record.rs:191) L2633; `ExistingArg::plan_value` (cli.rs:398) (by name) L2638; `check_plan_quoting` (edit.rs:2349) L2647; `refused` (bin/jubarte.rs:2697) L2649; `EditError::with_revision_syntax` (edit/refusal.rs:147) (by name) L2650; `write_plan` (edit/record.rs:161) L2661; `Recorded::json` (edit/record.rs:84) (by name) L2661; `write_publication_warnings` (bin/jubarte.rs:256) L2666,2670; `.summary` (ambiguous: `Recorded::summary` (edit/record.rs:91); `Opened::summary` (inspect.rs:765)) L2675
- **Calls (std/external/unresolved):** `.and_then`, `.as_deref`, `.display`, `.downcast_ref`, `.get_ref`, `.kind`, `.lock`, `.map_err`, `.to_string`, `Err`, `Ok`, `Some`, `std::fs::read_to_string`, `std::io::stderr`
- **Macros:** `format!`, `outln!`
- **Called by:** `run_flags` (bin/jubarte.rs:2511)

#### `edit_exit`  (L2686-2694)
`fn edit_exit(result: Result<(), (u8, String)>) -> ExitCode`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `ExitCode::from`
- **Macros:** `eprintln!`
- **Called by:** `cli_main` (bin/jubarte.rs:3935)

#### `refused`  (L2697-2742)
`fn refused(e: &jubarte::edit::EditError, file: &Path, source: &[u8]) -> (u8, String)`

- **Does:** A refused plan: its report goes to stdout, the error to stderr, exit 3.
- **Calls (crate):** `EditError::revision_next_actions` (edit/refusal.rs:159) L2731; `absolute` (bin/jubarte.rs:168) L2731; `source_sha256` (inspect.rs:299) L2731; `NearMatches::copy_hint` (edit/anchor.rs:139) (by name) L2739
- **Calls (std/external/unresolved):** `.as_object_mut`, `.as_ref`, `.clone`, `.collect`, `.enumerate`, `.expect`, `.filter_map`, `.insert`, `.into`, `.is_empty`, `.iter`, `.map`, `.push_str`, `.push`, `.to_string`, `serde_json::to_value`
- **Macros:** `format!`, `outln!`, `serde_json::json!`
- **Called by:** `run_edit` (bin/jubarte.rs:1341); `record_plan` (bin/jubarte.rs:2625)

#### `insert_before_summary`  (L2745-2756)
`fn insert_before_summary(jsonl: &mut String, line: &str)`

- **Does:** Insert `line` before the trailing `summary` line of a JSONL report.
- **Calls (std/external/unresolved):** `.is_empty`, `.rfind`, `.trim_end_matches`
- **Macros:** `format!`
- **Called by:** `publish_edit` (bin/jubarte.rs:1402)

#### `resolve_compare`  (L2788-2819)
`fn resolve_compare(compare: CompareArgs, quiet: bool) -> Job`

- **Does:** Native runtime defaults; required inputs have already been checked by clap.
- **Calls (std/external/unresolved):** `.expect`, `.or`, `.unwrap_or_else`
- **Called by:** `cli_main` (bin/jubarte.rs:3935); `tests::job_of` (bin/jubarte.rs:4789); `tests::revisions_is_no_subcommand` (bin/jubarte.rs:4954); `tests::plain_compare_positionals_leave_command_none` (bin/jubarte.rs:4966)

#### `default_output`  (L2822-2833)
`fn default_output(original: &Path, modified: &Path) -> PathBuf`

- **Does:** Build the default output path: `<original-dir>/<orig-stem>_v_<mod-stem>.docx`.
- **Calls (std/external/unresolved):** `.as_os_str`, `.file_stem`, `.filter`, `.into_owned`, `.is_empty`, `.join`, `.map`, `.parent`, `.to_string_lossy`, `.to_string`, `.unwrap_or_else`, `PathBuf::from`, `stem`
- **Macros:** `format!`
- **Called by:** `run` (bin/jubarte.rs:2862); `tests::default_output_uses_original_directory_and_stems` (bin/jubarte.rs:4826)

#### `comparer_settings`  (L2837-2860)
`fn comparer_settings( author: &str, date: &str, detail_threshold: Option<f64>, powertools_faithful: bool, no_paragraph_merge: bool, ) -> jubarte::comparer::WmlComparerSettings`

- **Does:** The comparer's settings for the `--mode`, `--detail-threshold`, author and date flags.
- **Calls (crate):** `WmlComparerSettings::powertools_faithful` (comparer/mod.rs:1482) L2845; `<WmlComparerSettings as Default>::default` (comparer/mod.rs:1493) L2847
- **Calls (std/external/unresolved):** `.to_string`, `.unwrap_or`
- **Called by:** `run` (bin/jubarte.rs:2862); `cli_main` (bin/jubarte.rs:3935)

#### `run`  (L2862-2974)
`fn run(job: &Job) -> Result<(), ConvertFailure>`

- **Does:** _no doc comment_
- **Calls (crate):** `output_format` (cli.rs:1996) L2863; `slash_path` (cli.rs:553) L2867; `default_output` (bin/jubarte.rs:2822) L2867; `read_document` (bin/jubarte.rs:3038) L2880,2881; `comparer_settings` (bin/jubarte.rs:2837) L2882; `Input::new` (bin/jubarte.rs:3010) L2889,2894; `Format::of_input` (cli.rs:1882) L2891,2896; `usage` (ambiguous: `usage` (bin/jubarte/check.rs:95); `usage` (bin/jubarte/edit_template.rs:292); `usage` (bin/jubarte/edit_typed.rs:962)) L2904,2922; `check_output` (bin/jubarte.rs:345) L2927; `write_output` (bin/jubarte.rs:353) L2934,2972; `compare_documents_with_settings` (document_comparer.rs:6423) L2937; `refusal` (bin/jubarte.rs:3112) L2940; `image_loader` (bin/jubarte.rs:3196) L2951; `resource_dir` (bin/jubarte.rs:3275) L2952; `Input::source` (bin/jubarte.rs:3027) L2961; `image_refusal_warnings` (bin/jubarte.rs:3244) L2965; `print_agent_view` (bin/jubarte.rs:1233) L2971
- **Calls (std/external/unresolved):** `.as_deref`, `.as_ref`, `.clone`, `.find_map`, `.into_iter`, `.is_none`, `.is_some_and`, `.is_some`, `.map_err`, `.map`, `.then_some`, `.to_string`, `.transpose`, `.unwrap_or_else`, `.unwrap_or`, `Err`, `ImageRefusals::default`, `Ok`, `Produced::Docx`, `Produced::Text`, `Some`, `jubarte::markdown::diff_markdown`, `jubarte::markdown::redline`, `std::rc::Rc::clone`
- **Macros:** `eprintln!`, `format!`
- **Called by:** `run_resolution` (bin/jubarte.rs:458); `cli_main` (bin/jubarte.rs:3935); `tests::a_legacy_doc_is_refused_with_a_save_as_hint` (bin/jubarte.rs:5275)

#### `exit_code`  (L2977-2985)
`fn exit_code(r: Result<(), String>) -> ExitCode`

- **Does:** Shared `Result → ExitCode` mapping for every command arm.
- **Calls (crate):** `say_error` (bin/jubarte.rs:2995) L2981
- **Called by:** `convert_exit_code` (bin/jubarte.rs:634); `cli_main` (bin/jubarte.rs:3935)

#### `say_error`  (L2995-3001)
`fn say_error(message: &str, exit: u8)`

- **Does:** Says `message` as the error a run ends with `exit` for: `error: …` on stderr, and under `--json` its JSON object on stdout too.
- **Calls (crate):** `.load` (ambiguous, 8 candidates) L2996; `write_stdout` (bin/jubarte.rs:1240) L2998; `runtime_json_error` (cli.rs:3656) L2998
- **Macros:** `eprintln!`
- **Called by:** `convert_exit_code` (bin/jubarte.rs:634); `exit_code` (bin/jubarte.rs:2977)

#### `Input::new`  (L3010-3025)
`fn new(path: &Path, format: Format, bytes: Vec<u8>) -> Result<Self, String>`

- **Does:** _no doc comment_
- **Calls (crate):** `markdown_text` (bin/jubarte.rs:3117) L3012; `admit_word` (bin/jubarte.rs:3106) L3014
- **Calls (std/external/unresolved):** `.clone`, `.display`, `Err`, `Ok`, `Some`
- **Macros:** `format!`
- **Called by:** `run` (bin/jubarte.rs:2862); `diff_inputs` (bin/jubarte.rs:3347)

#### `Input::source`  (L3027-3032)
`fn source(&self) -> jubarte::markdown::Source<'_>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `jubarte::markdown::Source::Docx`, `jubarte::markdown::Source::Markdown`
- **Called by:** `run` (bin/jubarte.rs:2862); `run_text_diff` (bin/jubarte.rs:3327) (?); `run_diff` (bin/jubarte.rs:3364) (?)

#### `read_document`  (L3038-3055)
`fn read_document(path: &Path) -> Result<Vec<u8>, String>`

- **Does:** Reads an input document, refusing the OLE files Word alone can open and RTF by their signature (`admission::sniff`, the check the library and the bindings run too).
- **Calls (crate):** `refused_input` (cli.rs:2042) L3040; `read` (legacy_doc.rs:180) L3043
- **Calls (std/external/unresolved):** `.display`, `.is_ok`, `.map_err`, `.with_extension`, `Err`, `Ok`, `String::new`, `std::fs::read`
- **Macros:** `format!`
- **Called by:** `input_confirmation_view` (bin/jubarte.rs:1171); `run` (bin/jubarte.rs:2862); `read_word` (bin/jubarte.rs:3059); `diff_inputs` (bin/jubarte.rs:3347); `tests::a_legacy_doc_is_refused_with_a_save_as_hint` (bin/jubarte.rs:5275)

#### `read_word`  (L3059-3063)
`fn read_word(path: &Path) -> Result<Vec<u8>, String>`

- **Does:** [`read_document`] for an input that must be Word: a file that is not a ZIP package is refused by name (`admission::sniff_package`).
- **Calls (crate):** `read_document` (bin/jubarte.rs:3038) L3060; `admit_word` (bin/jubarte.rs:3106) L3061
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `run_scrub` (bin/jubarte.rs:114); `run_resolution` (bin/jubarte.rs:458); `run_convert` (bin/jubarte.rs:646); `run_diff_render` (bin/jubarte.rs:828); `run_flags` (bin/jubarte.rs:2511); `read_input` (bin/jubarte.rs:3069); `read_input_json` (bin/jubarte.rs:3099); `run_validate` (bin/jubarte.rs:3812); `run_append` (bin/jubarte.rs:4433)

#### `read_input`  (L3069-3078)
`fn read_input(path: &Path, quiet: bool) -> Result<Vec<u8>, String>`

- **Does:** [`read_word`] for a task that reads the document (`read`, `changes`, `comments`): unless `quiet`, one stderr line says how many validity findings the input has ([`jubarte::validate::input_notice`]), so they are not taken for the reader's own, and where `validate` lists them.
- **Calls (crate):** `read_word` (bin/jubarte.rs:3059) L3070; `validate` (validate.rs:1140) L3072; `input_notice` (validate.rs:1032) L3073
- **Calls (std/external/unresolved):** `.to_string_lossy`, `Ok`
- **Macros:** `eprintln!`
- **Called by:** `run_changes` (bin/jubarte.rs:509); `run_comments` (bin/jubarte.rs:527); `run_read_tables` (bin/jubarte.rs:925); `run_text` (bin/jubarte.rs:1036)

#### `read_json`  (L3082-3093)
`fn read_json( bytes: &[u8], args: &ReadArgs, source: Option<String>, validity: jubarte::validate::InputValidity, ) -> Result<String, ConvertFailure>`

- **Does:** `read --json`'s snapshot of `bytes`, one line, its `summary` carrying the input's `validity` (#1309).
- **Calls (crate):** `read_snapshot` (markdown/mod.rs:821) L3088; `snapshot_options` (bin/jubarte.rs:1096) L3088
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`, `Ok`, `Some`, `serde_json::to_string`
- **Macros:** `format!`
- **Called by:** `run_text` (bin/jubarte.rs:1036)

#### `read_input_json`  (L3099-3103)
`fn read_input_json(path: &Path) -> Result<(Vec<u8>, jubarte::validate::InputValidity), String>`

- **Does:** [`read_word`] for a JSON read (`read --json`, `changes --json`, `comments --json`): no stderr line, which would break `--json 2>&1 | jq`; the input's findings come back for the output to carry (#1309), under `-q` too.
- **Calls (crate):** `read_word` (bin/jubarte.rs:3059) L3100; `input_validity_of` (validate.rs:1091) L3101
- **Calls (std/external/unresolved):** `.to_string_lossy`, `Ok`
- **Called by:** `run_changes` (bin/jubarte.rs:509); `run_comments` (bin/jubarte.rs:527); `run_for_edit` (bin/jubarte.rs:961); `run_text` (bin/jubarte.rs:1036)

#### `admit_word`  (L3106-3108)
`fn admit_word(path: &Path, bytes: &[u8]) -> Result<(), String>`

- **Does:** `CODE: FILE is …` when `bytes`, read from `path`, are no Word package.
- **Calls (crate):** `refused_input` (cli.rs:2042) L3107
- **Calls (std/external/unresolved):** `.map_or`, `Ok`
- **Called by:** `Input::new` (bin/jubarte.rs:3010); `read_word` (bin/jubarte.rs:3059); `run_convert_any` (bin/jubarte.rs:3458)

#### `refusal`  (L3112-3114)
`fn refusal(what: &str, error: &impl std::fmt::Display) -> Option<String>`

- **Does:** `{what} failed: CODE: …` for an admission refusal, whatever wrapped it (`admission::code_first`); `None` for any other error.
- **Calls (crate):** `code_first` (admission.rs:228) L3113
- **Calls (std/external/unresolved):** `.map`, `.to_string`
- **Macros:** `format!`
- **Called by:** `run` (bin/jubarte.rs:2862); `run_convert_any` (bin/jubarte.rs:3458); `tests::revision_and_reply_flags_pair_only_with_their_kind` (edit/flags.rs:1667) (?)

#### `markdown_text`  (L3117-3124)
`fn markdown_text(path: &Path, bytes: Vec<u8>) -> Result<String, String>`

- **Does:** Markdown bytes as text, without a byte order mark.
- **Calls (std/external/unresolved):** `.display`, `.map_err`, `.map`, `.strip_prefix`, `.unwrap_or`, `Ok`, `String::from_utf8`
- **Macros:** `format!`
- **Called by:** `Input::new` (bin/jubarte.rs:3010); `run_convert_any` (bin/jubarte.rs:3458)

#### `url_scheme`  (L3136-3147)
`fn url_scheme(path: &str) -> bool`

- **Does:** `path` names a URL (`scheme:…`): a scheme is a letter, then letters, digits and `+-.`, up to its `:`.
- **Calls (std/external/unresolved):** `.all`, `.bytes`, `.is_ascii_alphabetic`, `.is_ascii_alphanumeric`, `.is_some_and`, `.next`, `.split_once`
- **Macros:** `matches!`
- **Called by:** `image_policy` (bin/jubarte.rs:3153)

#### `image_policy`  (L3153-3187)
`fn image_policy(root: &Path, name: &str) -> Result<PathBuf, &'static str>`

- **Does:** `name`'s image path, its `%XX` escapes decoded and resolved against `root`, as the path inside `root` it lands on, or the reason it is refused.
- **Calls (crate):** `url_scheme` (bin/jubarte.rs:3136) L3177
- **Calls (std/external/unresolved):** `.and_then`, `.as_bytes`, `.canonicalize`, `.get`, `.is_absolute`, `.join`, `.len`, `.map_err`, `.map`, `.ok`, `.push`, `.strip_prefix`, `Err`, `PathBuf::from`, `Vec::with_capacity`, `std::str::from_utf8`, `u8::from_str_radix`
- **Called by:** `image_loader` (bin/jubarte.rs:3196); `report_image_refusals` (bin/jubarte.rs:3226)

#### `image_loader`  (L3196-3218)
`fn image_loader(dir: &Path, refusals: ImageRefusals) -> impl Fn(&str) -> Option<Vec<u8>> + use<>`

- **Does:** Reads images a Markdown file names: local paths, relative to `dir`, with `%XX` escapes decoded first.
- **Calls (crate):** `OutputRoot::open` (publication.rs:155) L3204; `image_policy` (bin/jubarte.rs:3153) L3208
- **Calls (std/external/unresolved):** `.and_then`, `.as_ref`, `.borrow_mut`, `.canonicalize`, `.map_err`, `.map`, `.ok_or`, `.ok`, `.push`, `.read`, `.to_string`, `Some`
- **Called by:** `run` (bin/jubarte.rs:2862); `run_diff` (bin/jubarte.rs:3364); `run_convert_any` (bin/jubarte.rs:3458)

#### `report_image_refusals`  (L3226-3239)
`fn report_image_refusals(dir: &Path, markdown: &[&str], refusals: &ImageRefusals)`

- **Does:** The image loads `markdown`'s images would be refused under the root `dir` names, recorded like [`image_loader`]'s.
- **Calls (crate):** `image_policy` (bin/jubarte.rs:3153) L3235
- **Calls (std/external/unresolved):** `.borrow_mut`, `.canonicalize`, `.filter`, `.flat_map`, `.iter`, `.ok`, `.push`, `.starts_with`, `jubarte::markdown::image_targets`
- **Called by:** `run_diff` (bin/jubarte.rs:3364)

#### `image_refusal_warnings`  (L3244-3252)
`fn image_refusal_warnings(refusals: &ImageRefusals) -> Vec<String>`

- **Does:** The refusals an [`image_loader`] recorded, as the writer phrases its fallback, for the compare and diff paths whose library calls drop `written.warnings` (#617).
- **Calls (std/external/unresolved):** `.borrow`, `.clone`, `.collect`, `.filter`, `.insert`, `.iter`, `.map`, `std::collections::HashSet::new`
- **Macros:** `format!`
- **Called by:** `run` (bin/jubarte.rs:2862); `run_diff` (bin/jubarte.rs:3364)

#### `image_warning_reason`  (L3257-3272)
`fn image_warning_reason(warning: &str, refusals: &ImageRefusals) -> String`

- **Does:** `warning` with its reason when it is an image written as its alt text [`image_loader`] refused; `convert` annotates the writer's warnings rather than printing its collection a second time (#617).
- **Calls (std/external/unresolved):** `.and_then`, `.borrow`, `.find`, `.iter`, `.map`, `.strip_prefix`, `.strip_suffix`, `.to_string`
- **Macros:** `format!`
- **Called by:** `run_convert_any` (bin/jubarte.rs:3458)

#### `resource_dir`  (L3275-3282)
`fn resource_dir(markdown: &Path, resource_path: Option<&Path>) -> PathBuf`

- **Does:** The directory a Markdown file's images are read from.
- **Calls (std/external/unresolved):** `.as_os_str`, `.filter`, `.is_empty`, `.map_or_else`, `.map`, `.parent`, `.unwrap_or_else`, `PathBuf::from`
- **Called by:** `run` (bin/jubarte.rs:2862); `run_diff` (bin/jubarte.rs:3364); `run_convert_any` (bin/jubarte.rs:3458)

#### `diff_output`  (L3302-3315)
`fn diff_output<'a>( output: Option<&'a Path>, input: &'a Path, render: &'a RenderArgs, force: bool, quiet: bool, ) -> Result<Option<Output<'a>>, ConvertFailure>`

- **Does:** `diff -o FILE`: text, through the shared writer; its existing file is refused before any work.
- **Calls (crate):** `Output::of_task` (bin/jubarte.rs:299) L3312; `check_output` (bin/jubarte.rs:345) L3313
- **Calls (std/external/unresolved):** `Ok`, `Some`
- **Called by:** `run_text_diff` (bin/jubarte.rs:3327); `run_diff` (bin/jubarte.rs:3364)

#### `print_or_write`  (L3318-3323)
`fn print_or_write(out: Option<&Output<'_>>, text: &str, label: &str) -> Result<(), ConvertFailure>`

- **Does:** `text` on stdout, or written to `out`.
- **Calls (crate):** `write_text` (bin/jubarte.rs:407) L3320; `write_stdout` (bin/jubarte.rs:1240) L3321
- **Calls (std/external/unresolved):** `.map_err`
- **Called by:** `run_text_diff` (bin/jubarte.rs:3327); `run_diff` (bin/jubarte.rs:3364)

#### `run_text_diff`  (L3327-3344)
`fn run_text_diff( job: &DiffJob<'_>, options: &jubarte::text_diff::TextOptions, ) -> Result<(), ConvertFailure>`

- **Does:** The text views (github, word, normal, context, side-by-side) use the shared core directly, with no redline pass.
- **Calls (crate):** `<RenderArgs as Default>::default` (cli.rs:2556) L3331; `diff_output` (bin/jubarte.rs:3302) L3332; `diff_inputs` (bin/jubarte.rs:3347) L3333; `diff_documents_view` (text_diff.rs:125) L3334; `.source` (ambiguous: `Input::source` (bin/jubarte.rs:3027); `<PublishError as std::error::Error>::source` (publication.rs:65)) L3334; `print_or_write` (bin/jubarte.rs:3318) L3343
- **Calls (std/external/unresolved):** `.as_ref`
- **Called by:** `cli_main` (bin/jubarte.rs:3935)

#### `diff_inputs`  (L3347-3360)
`fn diff_inputs(job: &DiffJob<'_>) -> Result<(Input, Input), String>`

- **Does:** The two documents of a `diff`, each read as its name or bytes say.
- **Calls (crate):** `read_document` (bin/jubarte.rs:3038) L3348; `Input::new` (bin/jubarte.rs:3010) L3349,3354; `Format::of_input` (cli.rs:1882) L3351,3356
- **Calls (std/external/unresolved):** `Ok`
- **Called by:** `run_text_diff` (bin/jubarte.rs:3327); `run_diff` (bin/jubarte.rs:3364)

#### `run_diff`  (L3364-3436)
`fn run_diff(job: &DiffJob<'_>) -> Result<(), ConvertFailure>`

- **Does:** `diff`'s paragraph patch, or `--format critic`'s CriticMarkup (two Markdown documents only), on stdout or in `-o`'s text file.
- **Calls (crate):** `<RenderArgs as Default>::default` (cli.rs:2556) L3365; `diff_output` (bin/jubarte.rs:3302) L3366; `diff_inputs` (bin/jubarte.rs:3347) L3367; `report_image_refusals` (bin/jubarte.rs:3226) L3373; `resource_dir` (bin/jubarte.rs:3275) L3374,3388; `image_loader` (bin/jubarte.rs:3196) L3387; `.render` (ambiguous, 4 candidates) L3398; `.source` (ambiguous: `Input::source` (bin/jubarte.rs:3027); `<PublishError as std::error::Error>::source` (publication.rs:65)) L3399,3400; `image_refusal_warnings` (bin/jubarte.rs:3244) L3432; `print_or_write` (bin/jubarte.rs:3318) L3435
- **Calls (std/external/unresolved):** `.as_ref`, `.clone`, `.display`, `.into`, `.is_some`, `.map_err`, `.to_string`, `Err`, `ImageRefusals::default`, `Some`, `jubarte::markdown::diff_markdown`, `jubarte::markdown::patch_documents`, `name`, `std::rc::Rc::clone`
- **Macros:** `eprintln!`, `format!`
- **Called by:** `cli_main` (bin/jubarte.rs:3935)

#### `paginated`  (L3444-3456)
`fn paginated( docx: &[u8], markdown: &str, track_changes: TrackChanges, revisions: jubarte::convert::RevisionStyle, ) -> String`

- **Does:** `convert`, for every pair of formats it takes.
- **Calls (crate):** `with_page_markers` (markdown/mod.rs:1101) L3451
- **Calls (std/external/unresolved):** `.into`
- **Macros:** `eprintln!`
- **Called by:** `run_convert_any` (bin/jubarte.rs:3458)

#### `run_convert_any`  (L3458-3760)
`fn run_convert_any( job: &ConvertJob<'_>, markdown: &MarkdownArgs, update_fields: bool, ) -> Result<(), ConvertFailure>`

- **Does:** _no doc comment_
- **Calls (crate):** `is_compound_file` (legacy_doc.rs:166) L3467; `doc_to_docx` (legacy_doc.rs:235) L3469; `sniff` (admission.rs:259) L3471; `Format::of_input` (cli.rs:1882) L3478; `admit_word` (bin/jubarte.rs:3106) L3481; `warn_nested_text_boxes` (bin/jubarte.rs:1121) L3516; `<RenderArgs as Default>::default` (cli.rs:2556) L3518; `Output::of_task` (bin/jubarte.rs:299) L3525; `write_output` (bin/jubarte.rs:353) L3528; `check_output` (bin/jubarte.rs:345) L3529; `run_convert` (bin/jubarte.rs:646) L3533; `at` (markdown/from_docx/drawing.rs:1360) (by name) L3542,3554,3595,3620,3689,3699...; `accept_revisions` (document_comparer.rs:4962) L3545; `refusal` (bin/jubarte.rs:3112) L3547,3551,3566,3655; `reject_revisions` (document_comparer.rs:4985) L3549; `resolve` (changes.rs:872) L3565,3654; `read` (markdown/mod.rs:767) L3574; `write_stdout` (bin/jubarte.rs:1240) L3596,3621,3700; `docx_to_markdown` (markdown/mod.rs:651) L3600; `<MarkdownOptions as Default>::default` (markdown/mod.rs:347) L3605; `paginated` (bin/jubarte.rs:3444) L3612; `check_side_file` (bin/jubarte.rs:423) L3649; `ensure_writable` (bin/jubarte.rs:237) L3651; `confirmation_view` (bin/jubarte.rs:1165) L3652; `refresh_fields` (bin/jubarte.rs:4474) L3660; `publish_paths` (bin/jubarte.rs:262) L3666; `wrote_line` (cli.rs:2060) L3682; `print_success` (bin/jubarte.rs:1181) L3685; `markdown_text` (bin/jubarte.rs:3117) L3692,3704; `resolve_critic` (markdown/mod.rs:150) L3696; `image_loader` (bin/jubarte.rs:3196) L3711; `resource_dir` (bin/jubarte.rs:3275) L3712; `image_warning_reason` (bin/jubarte.rs:3257) L3748
- **Calls (std/external/unresolved):** `.and_then`, `.as_bytes`, `.as_deref`, `.as_path`, `.as_slice`, `.clone`, `.display`, `.file_name`, `.into_owned`, `.into`, `.is_some`, `.len`, `.map_err`, `.map_or_else`, `.map`, `.name`, `.or_else`, `.or`, `.push`, `.then_some`, `.then`, `.to_path_buf`, `.to_string_lossy`, `.to_string`, `.transpose`, `.unwrap_or_default`, `.unwrap_or_else`, `.unwrap_or`, `.with_extension`, `Err`, `ImageRefusals::default`, `Ok`, `Produced::Docx`, `Produced::Text`, `Some`, `jubarte::markdown::markdown_to_docx`, `output_at`, `pdf_job`, `std::fs::read`, `std::rc::Rc::clone`
- **Macros:** `eprintln!`, `format!`, `matches!`, `outln!`, `vec!`
- **Called by:** `cli_main` (bin/jubarte.rs:3935)

#### `debug_options`  (L3764-3776)
`fn debug_options( checks: Vec<DebugCheck>, opts: jubarte::debug::Options, ) -> jubarte::debug::Options`

- **Does:** `jubarte debug`'s report options: the checks asked for, else the triage.
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.is_empty`, `.map`
- **Called by:** `cli_main` (bin/jubarte.rs:3935)

#### `run_debug`  (L3779-3789)
`fn run_debug(files: &[PathBuf], list: bool, opts: &jubarte::debug::Options) -> Result<(), String>`

- **Does:** `jubarte debug`: print the report for one package, or two compared.
- **Calls (crate):** `read` (ambiguous: `read` (legacy_doc.rs:180); `read` (markdown/mod.rs:767); `read` (markdown/write.rs:137)) L3781; `list` (debug.rs:2509) L3784; `report` (debug.rs:2655) L3786; `write_stdout` (bin/jubarte.rs:1240) L3788
- **Calls (std/external/unresolved):** `.as_deref`, `.display`, `.get`, `.map_err`, `.map`, `.transpose`, `std::fs::read`
- **Macros:** `format!`
- **Called by:** `cli_main` (bin/jubarte.rs:3935); `tests::debug_opens_a_dual_header_package_instead_of_refusing` (bin/jubarte.rs:4638)

#### `run_validate`  (L3812-3867)
`fn run_validate(job: &ValidateJob<'_>) -> Result<bool, ConvertFailure>`

- **Does:** `jubarte validate`: `Ok(true)` when the findings fail the run ([`jubarte::validate::fails`]), `Err` when a file could not be read or written or the rules cannot run.
- **Calls (crate):** `read` (ambiguous: `read` (legacy_doc.rs:180); `read` (markdown/mod.rs:767); `read` (markdown/write.rs:137)) L3818; `ensure_writable` (bin/jubarte.rs:237) L3820; `repair` (validate.rs:2623) L3821; `publish_file` (bin/jubarte.rs:273) L3822; `wrote_line` (cli.rs:2060) L3827; `read_word` (bin/jubarte.rs:3059) L3835; `validate_baselined` (validate.rs:370) L3836; `write_stdout` (bin/jubarte.rs:1240) L3846,3848,3864; `baselined_json` (validate.rs:1006) L3846; `baselined_text` (validate.rs:965) L3848; `fails` (validate.rs:405) L3850,3866; `validate_with` (validate.rs:316) L3852; `findings_text` (validate.rs:419) L3864
- **Calls (std/external/unresolved):** `.as_deref`, `.collect`, `.display`, `.iter`, `.len`, `.map_err`, `.map`, `.to_string`, `.transpose`, `.zip`, `Ok`, `Some`, `serde_json::to_string`, `std::fs::read`
- **Macros:** `eprintln!`, `format!`, `outln!`
- **Called by:** `cli_main` (bin/jubarte.rs:3935)

#### `run_debug_diff`  (L3871-3914)
`fn run_debug_diff( files: &[PathBuf], opts: &jubarte::debug::diff::DiffOptions, ) -> Result<(), String>`

- **Does:** `jubarte debug diff`: each file labelled by its stem (by its folder too when stems repeat, by position when both do).
- **Calls (crate):** `.parent` (ambiguous: `Marks::parent` (markdown/view_stories.rs:450); `Dom::parent` (xmllinq/mod.rs:597)) L3881; `unique` (edit/v2.rs:474) (by name) L3896,3899; `write_stdout` (bin/jubarte.rs:1240) L3913; `diff` (debug/diff.rs:60) L3913
- **Calls (std/external/unresolved):** `.and_then`, `.collect`, `.display`, `.file_name`, `.file_stem`, `.into_owned`, `.iter`, `.len`, `.map_err`, `.map`, `.to_string_lossy`, `.to_string`, `.unwrap_or_default`, `.zip`, `char::from`, `std::fs::read`, `stem`
- **Macros:** `format!`
- **Called by:** `cli_main` (bin/jubarte.rs:3935)

#### `main`  (L3921-3933)
`fn main() -> ExitCode`

- **Does:** _no doc comment_
- **Calls (crate):** `cli_main` (bin/jubarte.rs:3935) L3931
- **Calls (std/external/unresolved):** `.into`, `.map`, `.name`, `.spawn`, `.stack_size`, `std::panic::resume_unwind`, `std::thread::Builder::new`
- **Called by:** none found in src

#### `cli_main`  (L3935-4430)
`fn cli_main() -> ExitCode`

- **Does:** _no doc comment_
- **Calls (crate):** `Cli::try_parse_from` (cli.rs:3491) L3940; `refuse_output_collisions` (bin/jubarte.rs:1746) L3956; `usage` (bin/jubarte/edit_template.rs:292) L3961; `usage` (bin/jubarte/edit_typed.rs:962) L3961; `removed_spellings` (cli.rs:2390) L3967; `removed_error` (cli.rs:2398) L3969; `wants_json` (cli.rs:3611) L3970,3984,3995; `write_stdout` (bin/jubarte.rs:1240) L3972,3987,4249,4254,4266; `removed_json_error` (cli.rs:3687) L3972; `notes` (cli.rs:2462) L3981,3991; `usage_json_error` (cli.rs:3679) L3985; `.store` (ambiguous: `CommentFamily::store` (comments.rs:854); `Parsed::store` (revision_processor/comments.rs:54); `Part::store` (scrub.rs:203)) L3994; `run` (bin/jubarte/edit_template.rs:199) L4002; `run` (bin/jubarte/edit_typed.rs:787) L4003; `run` (bin/jubarte/check.rs:125) L4005; `convert_exit_code` (bin/jubarte.rs:634) L4008,4103,4156,4158,4166,4319...; `run` (bin/jubarte.rs:2862) L4008,4429; `resolve_compare` (bin/jubarte.rs:2788) L4008,4429; `exit_code` (bin/jubarte.rs:2977) L4011,4019,4030,4049,4079,4083...; `run_changes` (bin/jubarte.rs:509) L4011; `run_comments` (bin/jubarte.rs:527) L4019; `task_output` (bin/jubarte.rs:324) L4028,4047,4365,4406; `run_resolution` (bin/jubarte.rs:458) L4032,4051; `arm_timeout` (bin/jubarte.rs:593) L4075; `pdf_options` (bin/jubarte.rs:144) L4077; `run_convert_any` (bin/jubarte.rs:3458) L4103; `comparer_settings` (bin/jubarte.rs:2837) L4126; `PatchFormat::is_text_view` (cli.rs:1801) (by name) L4136; `run_text_diff` (bin/jubarte.rs:3327) L4156; `run_diff` (bin/jubarte.rs:3364) L4158; `run_text` (bin/jubarte.rs:1036) L4166,4421; `revision_style` (bin/jubarte.rs:127) L4186; `edit_exit` (bin/jubarte.rs:2686) L4208,4237; `run_flags` (bin/jubarte.rs:2511) L4208,4237; `capabilities` (capabilities.rs:399) L4258; `capabilities_text` (capabilities.rs:486) L4266; `run_self_update` (bin/jubarte.rs:4455) L4274; `run_debug_diff` (bin/jubarte.rs:3871) L4296; `run_diff_render` (bin/jubarte.rs:828) L4307; `debug_options` (bin/jubarte.rs:3764) L4333; `run_debug` (bin/jubarte.rs:3779) L4344; `run_append` (bin/jubarte.rs:4433) L4369; `run_validate` (bin/jubarte.rs:3812) L4394; `run_scrub` (bin/jubarte.rs:114) L4410; `.options` (ambiguous: `ScrubOptions::options` (bin/jubarte.rs:70); `<ScrubSelection as ScrubOptions>::options` (bin/jubarte.rs:74)) L4410
- **Calls (std/external/unresolved):** `.as_deref`, `.chain`, `.clone`, `.collect`, `.display`, `.exit`, `.expect`, `.filter`, `.first`, `.into_owned`, `.into`, `.is_none`, `.is_some`, `.iter`, `.kind`, `.map_or`, `.map`, `.or_else`, `.or`, `.skip`, `.strip_prefix`, `.then_some`, `.to_string_lossy`, `.to_string`, `.transpose`, `.unwrap_or_else`, `.unwrap_or`, `.with_extension`, `::try_parse_from`, `Default::default`, `Err`, `ExitCode::from`, `Ok`, `Path::new`, `i32::from`, `jubarte::capabilities::discovery_text`, `jubarte::capabilities::discovery`, `serde_json::to_string_pretty`, `std::env::args_os`, `std::ffi::OsString::from`, `std::iter::once`, `std::process::exit`
- **Macros:** `eprintln!`, `format!`, `outln!`, `unreachable!`
- **Called by:** `main` (bin/jubarte.rs:3921)

#### `run_append`  (L4433-4452)
`fn run_append( files: &[PathBuf], out: &Output<'_>, options: &jubarte::append::AppendOptions, ) -> Result<(), ConvertFailure>`

- **Does:** `jubarte append`: fold the documents left, `append(append(A, B), C)`.
- **Calls (crate):** `check_output` (bin/jubarte.rs:345) L4438; `read_word` (bin/jubarte.rs:3059) L4441,4443; `append_documents` (append.rs:152) L4444; `write_output` (bin/jubarte.rs:353) L4451
- **Calls (std/external/unresolved):** `.display`, `.iter`, `.map_err`, `.next`, `.ok_or`, `Produced::Docx`
- **Macros:** `eprintln!`, `format!`
- **Called by:** `cli_main` (bin/jubarte.rs:3935)

#### `run_self_update`  (L4455-4461)
`fn run_self_update(check: bool, yes: bool, version: Option<String>) -> Result<(), String>`

- **Does:** _no doc comment_
- **Calls (crate):** `run` (update.rs:225) L4456
- **Called by:** `cli_main` (bin/jubarte.rs:3935)

#### `run_self_update`  (L4464-4466)
`fn run_self_update(_check: bool, _yes: bool, _version: Option<String>) -> Result<(), String>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`, `Err`
- **Called by:** `cli_main` (bin/jubarte.rs:3935)

#### `refresh_fields`  (L4474-4481)
`fn refresh_fields(bytes: &[u8]) -> Result<(jubarte::fields::Updated, String), String>`

- **Does:** Prepare the refreshed package and its JSON report together, without file I/O.
- **Calls (crate):** `update_fields` (fields.rs:115) L4475
- **Calls (std/external/unresolved):** `.map_err`, `.to_string`, `Ok`
- **Macros:** `serde_json::json!`
- **Called by:** `run_convert_any` (bin/jubarte.rs:3458); `tests::refresh_fields_writes_the_report_and_says_when_it_cannot` (bin/jubarte.rs:5748)

## Test functions (57)

- `tests::an_unreadable_bundle_is_foreign` (L4494): A bundle directory `--force` cannot read is no earlier bundle it may replace: an unreadable one blocks (#1431 review), a missing one is nothing to replace.
- `tests::publication_cleanup_warnings_reach_stderr_for_both_outcomes` (L4513): _no doc comment_
- `tests::comment_changes_note_agrees_with_its_count` (L4539): _no doc comment_
- `tests::bundle_restore_failure` (L4550): _no doc comment_
- `tests::audit_d13_bundle_recheck_restore_failure_names_original` (L4615): _no doc comment_
- `tests::audit_d13_bundle_install_restore_failure_names_original` (L4620): _no doc comment_
- `tests::audit_d13_bundle_recheck_restore_success_keeps_message` (L4625): _no doc comment_
- `tests::audit_d13_bundle_install_restore_success_keeps_message` (L4630): _no doc comment_
- `tests::debug_opens_a_dual_header_package_instead_of_refusing` (L4638): GHSA-m665-v7pm-7grj: `jubarte debug` opens a package every other route refuses — a local header naming an entry differently than the central directory — because triaging refused input is its purpose.
- `tests::append_parses_files_output_and_break` (L4678): _no doc comment_
- `tests::self_update_parses_check_yes_and_a_pinned_version` (L4738): _no doc comment_
- `tests::convert_revisions_default_to_conventional_and_validate_the_palette` (L4763): _no doc comment_
- `tests::job_of` (L4789): _no doc comment_
- `tests::cli_definition_is_valid` (L4800): clap's own invariants (catches derive-config mistakes like duplicate shorts).
- `tests::positional_args_and_default_output` (L4805): _no doc comment_
- `tests::default_output_uses_original_directory_and_stems` (L4826): _no doc comment_
- `tests::named_flags_override_positionals` (L4838): _no doc comment_
- `tests::all_options_long_and_short` (L4853): _no doc comment_
- `tests::compare_mode_defaults_to_word_and_powertools_has_two_spellings` (L4876): _no doc comment_
- `tests::flags_can_supply_both_inputs_without_positionals` (L4891): _no doc comment_
- `tests::double_dash_treats_rest_as_positionals` (L4908): _no doc comment_
- `tests::help_and_version_are_handled_by_clap` (L4915): _no doc comment_
- `tests::missing_inputs_are_clap_usage_errors` (L4924): _no doc comment_
- `tests::extra_positional_and_unknown_flag_rejected_by_clap` (L4937): _no doc comment_
- `tests::revisions_is_no_subcommand` (L4954): `revisions` (Docxodus's GetRevisions listing) left the CLI: `changes` is the one listing.
- `tests::plain_compare_positionals_leave_command_none` (L4966): Prior behavior path: a two-positional invocation whose filenames do NOT collide with the subcommand name is unaffected by adding `command` to `Cli` — `cli.command` stays `None` and the native resolver merges the positionals exactly as before this PR.
- `tests::accept_subcommand_parses_file_output_and_force` (L4977): `accept <file> -o <out> --force` parses into `Command::Accept` with the output and force flag captured.
- `tests::scrub_flags_select_what_goes_and_none_selects_everything` (L5005): `scrub` without a selection removes all four kinds, under the alias `Author`; any flag narrows it to the ones given.
- `tests::reject_subcommand_parses_file_output_and_defaults_force_false` (L5042): `reject <file> -o <out>` parses into `Command::Reject`; `force` defaults to false (the same no-clobber contract as `accept` / compare).
- `tests::accept_selection_flags_build_the_change_filter` (L5068): `--id`, `--author` and `--kind` repeat and select the changes to resolve; a flag left out constrains nothing.
- `tests::reject_subcommand_requires_output` (L5100): `reject` requires `-o/--output` (clap usage error when omitted), matching `accept`.
- `tests::convert_takes_fail_on_substitution_and_defaults_it_off` (L5107): _no doc comment_
- `tests::convert_subcommand_parses_font_report` (L5125): _no doc comment_
- `tests::tiny_docx_bytes` (L5160): _no doc comment_
- `tests::body_docx_bytes` (L5167): A Letter-size package whose body is `body` (WordprocessingML).
- `tests::convert_rejects_a_font_report_path_equal_to_the_pdf` (L5194): _no doc comment_
- `tests::a_legacy_doc_is_refused_with_a_save_as_hint` (L5275): _no doc comment_
- `tests::fail_on_substitution_is_exit_4_after_the_outputs` (L5315): _no doc comment_
- `tests::a_plain_convert_error_keeps_exit_1` (L5349): _no doc comment_
- `tests::convert_font_report_writes_json` (L5355): _no doc comment_
- `tests::regression_tests::convert_without_revision_flags_uses_conventional_marks` (L5396): _no doc comment_
- `tests::regression_tests::custom_revision_flags_reach_the_palette_parser` (L5410): _no doc comment_
- `tests::regression_tests::invalid_revision_flags_are_rejected_with_option_context` (L5434): _no doc comment_
- `tests::pages_docx` (L5459): _no doc comment_
- `tests::file_names` (L5467): _no doc comment_
- `tests::parse_pages_counts_from_one_and_sorts_without_repeats` (L5477): _no doc comment_
- `tests::parse_pages_refuses_zero_empty_backwards_and_words` (L5485): _no doc comment_
- `tests::convert_and_diff_render_parse_their_page_flags` (L5502): _no doc comment_
- `tests::convert_job` (L5560): _no doc comment_
- `tests::convert_pages_writes_the_selected_pages_named_by_page_number` (L5585): _no doc comment_
- `tests::convert_pages_needs_png_and_an_existing_page` (L5609): _no doc comment_
- `tests::diff_job` (L5624): _no doc comment_
- `tests::diff_render_writes_only_the_changed_pages_and_diff_json` (L5637): _no doc comment_
- `tests::diff_render_of_equal_documents_writes_only_diff_json` (L5693): _no doc comment_
- `tests::diff_render_without_an_overlay_writes_no_diff_png` (L5704): _no doc comment_
- `tests::convert_takes_update_fields` (L5726): _no doc comment_
- `tests::refresh_fields_writes_the_report_and_says_when_it_cannot` (L5748): _no doc comment_
