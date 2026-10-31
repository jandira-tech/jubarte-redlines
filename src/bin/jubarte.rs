// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte` — generate a tracked-changes (redline) `.docx` from two documents.
//!
//! ```text
//! jubarte original.docx modified.docx
//!   → writes original_v_modified.docx
//! jubarte -b a.docx -m b.docx -o out.docx --author "Jane" --datetime 2024-01-02T00:00:00Z
//! jubarte compare original.docx modified.docx
//! jubarte diff old.md new.md                 (inline paragraph patch on stdout)
//! jubarte diff old.docx new.docx --format github (unified text patch)
//! jubarte convert draft.md -o draft.docx     (CriticMarkup as tracked changes)
//! ```
//!
//! Either document may be Markdown (`.md`, `.markdown`); see `jubarte diff`
//! and `jubarte convert --help`.
//!
//! Positional args are `<ORIGINAL> <MODIFIED>`; the `--original`/`--modified`
//! flags override them. Argument parsing, `--help`, `--version`, short/long
//! flags, and validation are handled by clap (gated behind the default `cli`
//! feature).

#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use jubarte::cli::*;

#[path = "jubarte/check.rs"]
mod check;
#[path = "jubarte/edit_template.rs"]
mod edit_template;
#[path = "jubarte/edit_typed.rs"]
mod edit_typed;

/// `println!` for the listings: a reader that stops early (`jubarte read
/// FILE | head`) ends the process quietly, as `write_stdout` ends a view.
macro_rules! outln {
    ($($arg:tt)*) => {
        stdout_line(format_args!($($arg)*))
    };
}

fn stdout_line(line: std::fmt::Arguments<'_>) {
    use std::io::Write;
    if let Err(e) = writeln!(std::io::stdout().lock(), "{line}") {
        if e.kind() != std::io::ErrorKind::BrokenPipe {
            eprintln!("error: writing to stdout: {e}");
            std::process::exit(1);
        }
        std::process::exit(0);
    }
}

/// CLI-only global allocator. The redline pipeline spends ~41% of CPU self-time
/// in allocation/copy/free/drop of xmllinq nodes (measured with samply on the
/// RFP17 fixtures — `produce::coalesce_recurse`/`reconstruct_element` churn).
/// mimalloc lowers that per-allocation cost; it changes performance only, never
/// program semantics. Library consumers are unaffected (this lives in the
/// binary). Toggle off with `--no-default-features --features cli` for A/B.
#[cfg(feature = "fast-alloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

trait ScrubOptions {
    fn options(&self) -> jubarte::scrub::ScrubOptions;
}

impl ScrubOptions for ScrubSelection {
    fn options(&self) -> jubarte::scrub::ScrubOptions {
        if *self == Self::default() {
            return jubarte::scrub::ScrubOptions::default();
        }
        jubarte::scrub::ScrubOptions {
            author_alias: self.author_alias.clone(),
            rsids: self.rsids,
            docprops: self.docprops,
            comments: self.comments,
        }
    }
}

trait SelectionFilter {
    fn filter(&self) -> jubarte::changes::ChangeFilter;
}

impl SelectionFilter for Selection {
    fn filter(&self) -> jubarte::changes::ChangeFilter {
        use jubarte::changes::ChangeKind;
        let given = |v: &[String]| (!v.is_empty()).then(|| v.to_vec());
        jubarte::changes::ChangeFilter {
            ids: given(&self.ids),
            authors: given(&self.authors),
            kinds: (!self.kinds.is_empty()).then(|| {
                self.kinds
                    .iter()
                    .map(|k| match k {
                        KindArg::Insertion => ChangeKind::Insertion,
                        KindArg::Deletion => ChangeKind::Deletion,
                        KindArg::Move => ChangeKind::Move,
                        KindArg::Formatting => ChangeKind::Formatting,
                        KindArg::Replacement => ChangeKind::Replacement,
                    })
                    .collect()
            }),
        }
    }
}

fn run_scrub(
    file: &Path,
    out: &Output<'_>,
    options: &jubarte::scrub::ScrubOptions,
) -> Result<(), ConvertFailure> {
    check_output(out)?;
    let bytes = read_word(file)?;
    let scrubbed =
        jubarte::scrub::scrub(&bytes, options).map_err(|e| format!("scrub failed: {e}"))?;
    write_output(out, Produced::Docx(&scrubbed))
}

/// The library's revision style for `--revisions` / `--revision-palette`.
fn revision_style(
    revisions: Revisions,
    palette: Option<&str>,
) -> Result<jubarte::convert::RevisionStyle, String> {
    use jubarte::convert::{RevisionPalette, RevisionStyle};
    match (revisions, palette) {
        (Revisions::Custom, Some(spec)) => Ok(RevisionStyle::Custom(
            RevisionPalette::parse(spec).map_err(|e| format!("--revision-palette: {e}"))?,
        )),
        (Revisions::Custom, None) => Err("--revisions custom needs --revision-palette".to_string()),
        (_, Some(_)) => Err("--revision-palette needs --revisions custom".to_string()),
        (Revisions::Conventional, None) => Ok(RevisionStyle::Conventional),
        (Revisions::Word, None) => Ok(RevisionStyle::Word),
    }
}

/// The renderer options a [`RenderArgs`] group asks for.
fn pdf_options(
    render: &RenderArgs,
    compress: bool,
) -> Result<jubarte::convert::PdfOptions, String> {
    Ok(render.apply(jubarte::convert::PdfOptions {
        compress,
        revisions: revision_style(render.revisions, render.revision_palette.as_deref())?,
        ..jubarte::convert::PdfOptions::default()
    }))
}

/// The absolute form of a CLI path: the real path when it exists, else the
/// deepest existing ancestor's real path joined with the rest. Error
/// envelopes use it to name the file the caller supplied even when its
/// bytes were never read (#1084 review F5; proposal §11 keeps
/// `source.path` absolute).
///
/// An artifact is recorded before it is published, so `canonicalize(path)`
/// fails for every `-o`. The working-directory join alone kept Windows' 8.3
/// short names (`RUNNER~1` on CI, #1244): `std::fs::canonicalize` expands
/// them (`GetFinalPathNameByHandleW`, `VOLUME_NAME_DOS`), so the nearest
/// ancestor that exists is canonicalized and the missing tail rejoined. A
/// path with no existing ancestor keeps the working-directory join, still
/// absolute.
fn absolute(path: &Path) -> String {
    let joined = std::env::current_dir().unwrap_or_default().join(path);
    let resolved = std::fs::canonicalize(&joined).unwrap_or_else(|_| {
        let mut missing = Vec::new();
        let mut ancestor = joined.as_path();
        loop {
            if let Ok(real) = std::fs::canonicalize(ancestor) {
                break missing.iter().rev().fold(real, |acc, name| acc.join(name));
            }
            match (ancestor.parent(), ancestor.file_name()) {
                (Some(parent), Some(name)) => {
                    missing.push(name.to_os_string());
                    ancestor = parent;
                }
                _ => break joined.clone(),
            }
        }
    });
    slash_path(&resolved)
}

/// Whether two CLI paths name the same file. Two existing paths are compared
/// by identity, which sees through hard links, symlinks and a volume that
/// ignores case; a path not written yet is keyed by its canonicalized parent
/// and its name (`out.pdf` and `./out.pdf` match), after following a symlink
/// whose target does not exist yet, since a write goes through it. Names
/// are matched ignoring case: on a volume that ignores it they are one file
/// once written, and on one that does not the cost is a refused pair the
/// user can rename.
fn same_path(a: &Path, b: &Path) -> bool {
    if let Ok(same) = same_file::is_same_file(a, b) {
        return same;
    }
    fn key(p: &Path) -> Option<PathBuf> {
        let mut p = p.to_path_buf();
        // The kernel's own bound on a link chain (Linux MAXSYMLINKS).
        for _ in 0..40 {
            let Ok(target) = std::fs::read_link(&p) else {
                break;
            };
            p = match p.parent() {
                Some(dir) if target.is_relative() => dir.join(target),
                _ => target,
            };
        }
        let name = p.file_name()?;
        let parent = match p.parent() {
            Some(dir) if !dir.as_os_str().is_empty() => dir.to_path_buf(),
            _ => PathBuf::from("."),
        };
        Some(
            std::fs::canonicalize(parent)
                .ok()?
                .join(name.to_string_lossy().to_lowercase()),
        )
    }
    a == b || matches!((key(a), key(b)), (Some(x), Some(y)) if x == y)
}

/// Two directory paths name the same existing directory. `same_path` keys on
/// the file name, which `.` and `./` do not have.
fn same_dir(a: &Path, b: &Path) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(x), Ok(y)) => x == y,
        _ => same_path(a, b),
    }
}

/// No-clobber contract shared by every writing subcommand.
fn ensure_writable(output: &Path, force: bool) -> Result<(), ConvertFailure> {
    if output.exists() && !force {
        return Err(taken_output(output).into());
    }
    Ok(())
}

fn report_publication(
    result: Result<jubarte::publication::PublishReport, jubarte::publication::PublishError>,
    stderr: &mut impl std::io::Write,
) -> Result<(), jubarte::publication::PublishError> {
    let warnings = match &result {
        Ok(report) => &report.warnings,
        Err(error) => &error.warnings,
    };
    write_publication_warnings(warnings, stderr);
    result.map(|_| ())
}

fn write_publication_warnings(warnings: &[String], stderr: &mut impl std::io::Write) {
    for warning in warnings {
        let _ = writeln!(stderr, "warning: {warning}");
    }
}

fn publish_paths(
    files: &[(PathBuf, &[u8])],
    force: bool,
) -> Result<(), jubarte::publication::PublishError> {
    report_publication(
        jubarte::publication::publish_paths(files, force),
        &mut std::io::stderr().lock(),
    )
}

/// Stage complete bytes and atomically replace a single authorized output.
fn publish_file(path: &Path, bytes: &[u8], force: bool) -> Result<(), String> {
    publish_paths(&[(path.to_path_buf(), bytes)], force).map_err(|e| e.to_string())
}

/// One task's `-o`: where, as what, and how a Word result becomes it.
struct Output<'a> {
    kind: Format,
    path: &'a Path,
    force: bool,
    quiet: bool,
    /// The document the task read (named in a refusal).
    input: &'a Path,
    /// The task actually read Markdown (an explicit --from can override its suffix).
    markdown_input: bool,
    /// The view's options for a `.md` output and the selection for a `.json`
    /// one: `read`'s own, the defaults elsewhere.
    view: &'a ReadArgs,
    /// The name a `.md` view's header gives the document.
    source: String,
    /// How a `.pdf` or `.png` output is drawn.
    render: &'a RenderArgs,
}

impl<'a> Output<'a> {
    /// The `-o` of accept, reject, append, scrub and convert: the
    /// view's header names the Word file the task did not write.
    fn of_task(
        kind: Format,
        path: &'a Path,
        input: &'a Path,
        render: &'a RenderArgs,
        force: bool,
        quiet: bool,
    ) -> Self {
        static VIEW: std::sync::LazyLock<ReadArgs> = std::sync::LazyLock::new(ReadArgs::default);
        Self {
            kind,
            path,
            force,
            quiet,
            input,
            markdown_input: false,
            view: &VIEW,
            // An identifier agents read back: `/` on every OS (#568).
            source: format!("{} (not written)", slash_path(&path.with_extension("docx"))),
            render,
        }
    }
}

/// The [`Output`] of a task whose `-o` the parser checked.
fn task_output<'a>(
    task: Task,
    path: &'a Path,
    input: &'a Path,
    render: &'a RenderArgs,
    force: bool,
    quiet: bool,
) -> Result<Output<'a>, String> {
    let kind = output_format(task, Some(path), None)?.unwrap_or(Format::Docx);
    Ok(Output::of_task(kind, path, input, render, force, quiet))
}

/// A Word package, or text already in the output's format (CriticMarkup,
/// convert's Markdown).
enum Produced<'a> {
    Docx(&'a [u8]),
    Text(&'a str),
}

/// Refuse an existing output before any work: PNG pages are checked once
/// their names are known.
fn check_output(out: &Output<'_>) -> Result<(), ConvertFailure> {
    if out.kind == Format::Png {
        return Ok(());
    }
    ensure_writable(out.path, out.force)
}

/// Writes `produced` as `out.kind` and says `wrote …` on stderr unless `-q`.
fn write_output(out: &Output<'_>, produced: Produced<'_>) -> Result<(), ConvertFailure> {
    let text = match (out.kind, produced) {
        (Format::Pdf | Format::Png, Produced::Docx(bytes)) => {
            let options = pdf_options(out.render, false)?;
            let pages = out.render.pages.as_deref().map(parse_pages).transpose()?;
            return run_convert(&ConvertJob {
                file: out.input,
                bytes: Some(bytes),
                output: Some(out.path),
                force: out.force,
                font_report: None,
                options,
                to: Some(out.kind),
                dpi: out.render.dpi,
                report: None,
                pages: pages.as_deref(),
                fail_on_substitution: false,
                quiet: out.quiet,
            });
        }
        (Format::Docx, Produced::Docx(bytes)) => {
            ensure_writable(out.path, out.force)?;
            let before =
                (!out.quiet).then(|| input_confirmation_view(out.input, out.markdown_input));
            publish_file(out.path, bytes, out.force)?;
            if !out.quiet {
                eprintln!("{}", wrote_line(out.path, Format::Docx, bytes.len(), None));
            }
            if let Some(before) = before {
                print_success(out.input, before, out.path, bytes)?;
            }
            return Ok(());
        }
        (Format::Md | Format::Json, Produced::Text(text)) => text.to_string(),
        (Format::Md, Produced::Docx(bytes)) => {
            agent_view(bytes, Some(out.source.clone()), out.view)?.markdown
        }
        (Format::Json, Produced::Docx(bytes)) => {
            let snapshot =
                jubarte::markdown::read_snapshot(bytes, &snapshot_options(out.view, None)?)
                    .map_err(|e| e.to_string())?;
            let json = serde_json::to_string(&snapshot).map_err(|e| e.to_string())?;
            format!("{json}\n")
        }
        (kind, _) => {
            return Err(
                format!("internal error: no {} writer for this result", kind.name()).into(),
            );
        }
    };
    write_text(out, &text, out.kind.name())
}

/// Publish text through the common writer, naming the representation written.
fn write_text(out: &Output<'_>, text: &str, label: &str) -> Result<(), ConvertFailure> {
    ensure_writable(out.path, out.force)?;
    publish_file(out.path, text.as_bytes(), out.force)?;
    if !out.quiet {
        eprintln!(
            "wrote {} ({label}, {} bytes)",
            out.path.display(),
            text.len()
        );
    }
    Ok(())
}

/// A side file (`--report`, `--font-report`) is written apart from the
/// output, after the input is read, so sharing a path with either would
/// silently replace one with the other; `--force` never allows that.
fn check_side_file(
    side: &Path,
    flag: &str,
    others: [(&Path, &str); 2],
    force: bool,
) -> Result<(), ConvertFailure> {
    for (other, name) in others {
        if same_path(side, other) {
            return Err(format!(
                "{flag} '{}' is the same file as the {name}",
                slash_path(side)
            )
            .into());
        }
    }
    ensure_writable(side, force)
}

/// The note a selection prints when it leaves tracked changes inside
/// comment bodies, which no `--id` can name (#1223).
fn comment_changes_note(left: usize) -> String {
    let (s, were, them) = if left == 1 {
        ("", "was", "it")
    } else {
        ("s", "were", "them")
    };
    format!(
        "note: {left} tracked change{s} inside comments {were} left; accept or reject all to settle {them}"
    )
}

/// Shared body for `accept` / `reject`: read the redline, apply the package-wide
/// resolution, and write the result under the compare path's no-clobber
/// contract. Generic over the resolver's error so neither `OpcError`'s path nor
/// the two arms' bodies are duplicated.
fn run_resolution(
    file: &Path,
    out: &Output<'_>,
    filter: &jubarte::changes::ChangeFilter,
    apply: fn(
        &[u8],
        &jubarte::changes::ChangeFilter,
    ) -> Result<jubarte::changes::ResolvedChanges, jubarte::changes::ChangeError>,
    what: &str,
) -> ExitCode {
    use jubarte::changes::ChangeError;
    let run = || -> Result<(), (ExitCode, String)> {
        let failed = |e: String| (ExitCode::FAILURE, e);
        // An id that is not a change id, or names two changes, is a usage
        // error: exit 2, as clap's own.
        let filter = filter
            .normalized()
            .map_err(|e| (ExitCode::from(2), e.to_string()))?;
        check_output(out).map_err(|f| (ExitCode::from(f.code), f.message))?;
        let bytes = read_word(file).map_err(failed)?;
        let resolved = apply(&bytes, &filter).map_err(|e| match e {
            ChangeError::Ambiguous { .. }
            | ChangeError::InvalidId(_)
            | ChangeError::PartOfChange { .. } => (ExitCode::from(2), e.to_string()),
            e => failed(format!("{what} failed: {e}")),
        })?;
        write_output(out, Produced::Docx(&resolved.document))
            .map_err(|f| (ExitCode::from(f.code), f.message))?;
        for (comment, why) in resolved.removed_comments.iter().zip(&resolved.removed_why) {
            let preview: String = comment.text.chars().take(60).collect();
            eprintln!(
                "warning: {what} removed comment c{} ({}: {preview:?}): {why}",
                comment.id, comment.author,
            );
        }
        for note in settlement_notes(
            &[],
            &resolved.kept_comments,
            &resolved.document,
            &bytes,
            what,
        ) {
            eprintln!("note: {note}");
        }
        // #1223: --id cannot name a comment-body change, so a selection can
        // leave some; say how many (an empty selection resolves them all).
        if resolved.comment_changes_left > 0 {
            eprintln!("{}", comment_changes_note(resolved.comment_changes_left));
        }
        Ok(())
    };
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err((code, e)) => {
            eprintln!("error: {e}");
            code
        }
    }
}

fn run_changes(file: &Path, json: bool, authors: &[String], quiet: bool) -> Result<(), String> {
    let listed = |bytes: &[u8]| {
        jubarte::changes::list_logical_changes(bytes)
            .map(|changes| jubarte::changes::by_authors(changes, authors))
            .map_err(|e| e.to_string())
    };
    if json {
        let (bytes, validity) = read_input_json(file)?;
        let changes = listed(&bytes)?;
        for c in &changes {
            outln!("{}", serde_json::to_string(c).map_err(|e| e.to_string())?);
        }
        return validity_line(&validity, quiet);
    }
    let bytes = read_input(file, quiet)?;
    let changes = listed(&bytes)?;
    outln!(
        "{}",
        jubarte::changes::changes_text(&changes).trim_end_matches('\n')
    );
    Ok(())
}

fn run_comments(
    file: &Path,
    json: bool,
    author: Option<&str>,
    latest: bool,
    quiet: bool,
) -> Result<(), String> {
    let (bytes, validity) = if json {
        let (bytes, validity) = read_input_json(file)?;
        (bytes, Some(validity))
    } else {
        (read_input(file, quiet)?, None)
    };
    let comments = jubarte::comments::list_comments(&bytes).map_err(|e| e.to_string())?;
    let comments = jubarte::comments::select_comments(comments, author, latest);
    let Some(validity) = validity else {
        outln!(
            "{}",
            jubarte::comments::comments_text(&comments).trim_end_matches('\n')
        );
        return Ok(());
    };
    for c in &comments {
        outln!("{}", serde_json::to_string(c).map_err(|e| e.to_string())?);
    }
    validity_line(&validity, quiet)
}

/// The quietable stderr status for `changes --json` and `comments --json` (#1925).
fn validity_line(validity: &jubarte::validate::InputValidity, quiet: bool) -> Result<(), String> {
    if !quiet {
        eprintln!(
            "{}",
            jubarte::validate::input_validity_line(validity).map_err(|e| e.to_string())?
        );
    }
    Ok(())
}

/// Which artifacts `convert` writes and where.
struct ConvertJob<'a> {
    file: &'a Path,
    /// The .docx to render instead of reading `file` (written from Markdown).
    bytes: Option<&'a [u8]>,
    output: Option<&'a Path>,
    force: bool,
    font_report: Option<&'a Path>,
    /// Compression, revision marks, comment placement and kept pages.
    options: jubarte::convert::PdfOptions,
    /// The format asked for (`--to`, or the retired `--pdf`/`--png`);
    /// `None` takes it from `output`, else from the input.
    to: Option<Format>,
    dpi: f32,
    report: Option<&'a Path>,
    /// Zero-based document pages to write; `None` for all.
    pages: Option<&'a [usize]>,
    /// Exit [`EXIT_FONT_SUBSTITUTED`] when a requested font was substituted.
    fail_on_substitution: bool,
    /// `-q`: say nothing of what was written.
    quiet: bool,
}

/// `convert --timeout`: the deadline passed.
const EXIT_TIMEOUT: i32 = 124;

/// Exit [`EXIT_TIMEOUT`] once `limit` has passed, whatever the main thread
/// is doing (layout of a pathological document cannot be interrupted).
fn arm_timeout(limit: std::time::Duration) {
    std::thread::spawn(move || {
        std::thread::sleep(limit);
        eprintln!(
            "error: timed out after {}s (--timeout)",
            limit.as_secs_f64()
        );
        std::process::exit(EXIT_TIMEOUT);
    });
}

/// `convert --fail-on-substitution`: the outputs were written, but a
/// requested font was drawn with a substitute.
const EXIT_FONT_SUBSTITUTED: u8 = 4;

/// Why `convert` failed, and the exit status that says so.
#[derive(Debug, PartialEq, Eq)]
struct ConvertFailure {
    code: u8,
    message: String,
}

impl From<String> for ConvertFailure {
    fn from(message: String) -> Self {
        Self { code: 1, message }
    }
}

impl From<(u8, String)> for ConvertFailure {
    fn from((code, message): (u8, String)) -> Self {
        Self { code, message }
    }
}

impl From<&str> for ConvertFailure {
    fn from(message: &str) -> Self {
        message.to_string().into()
    }
}

/// The exit status of a `convert` run: 1 for an error, or its own code.
fn convert_exit_code(r: Result<(), ConvertFailure>) -> ExitCode {
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(ConvertFailure { code: 1, message }) => exit_code(Err(message)),
        Err(ConvertFailure { code, message }) => {
            say_error(&message, code);
            ExitCode::from(code)
        }
    }
}

/// Render `job` as a PDF, or as PNG pages when it asks for PNG.
fn run_convert(job: &ConvertJob<'_>) -> Result<(), ConvertFailure> {
    let output = job
        .output
        .map(Path::to_path_buf)
        .unwrap_or_else(|| job.file.with_extension("pdf"));
    let png = job.to == Some(Format::Png);
    let want_pdf = !png;
    for (side, flag) in [(job.font_report, "--font-report"), (job.report, "--report")] {
        if let Some(side) = side {
            let named = if png { "PNG output" } else { "PDF output" };
            let others = [(output.as_path(), named), (job.file, "input")];
            check_side_file(side, flag, others, job.force)?;
        }
    }
    if want_pdf {
        ensure_writable(&output, job.force)?;
    }
    let stem = output
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "page".to_string());
    let dir = output.parent().map(Path::to_path_buf).unwrap_or_default();
    let bytes = match job.bytes {
        Some(bytes) => bytes.to_vec(),
        None => read_word(job.file)?,
    };
    let rendered = jubarte::convert::render(
        &bytes,
        job.options,
        jubarte::convert::RenderRequest {
            pdf: want_pdf,
            png_dpi: png.then_some(job.dpi),
            pages: job.pages.map(<[usize]>::to_vec),
        },
    )
    .map_err(|e| format!("convert failed: {e}"))?;
    let pages = rendered.report.page_count;
    // Each PNG is named by its document page, padded to the document's page
    // count. The pages are known only now; check every PNG path before the
    // first write so a refused page leaves no partial bundle.
    let png_paths: Vec<PathBuf> = rendered
        .report
        .pages
        .iter()
        .take(rendered.pngs.len())
        .map(|p| {
            dir.join(png_name(
                &stem,
                p.number - 1,
                rendered.report.document_pages,
            ))
        })
        .collect();
    for path in &png_paths {
        ensure_writable(path, job.force)?;
        // `-o r.png --report r-page-01.png` would write the page, then
        // the report over it.
        for (side, flag) in [(job.font_report, "--font-report"), (job.report, "--report")] {
            if let Some(side) = side.filter(|side| same_path(side, path)) {
                return Err(format!(
                    "{flag} '{}' is the same file as the PNG output",
                    slash_path(side)
                )
                .into());
            }
        }
    }
    let font_json = job
        .font_report
        .map(|_| jubarte::convert::font_report_json(&rendered.report.fonts));
    let report_json = job.report.map(|_| rendered.report.to_json());
    let mut outputs: Vec<(PathBuf, &[u8])> = Vec::new();
    if let Some(pdf) = &rendered.pdf {
        outputs.push((output.clone(), pdf));
    }
    if png {
        outputs.extend(
            png_paths
                .iter()
                .cloned()
                .zip(rendered.pngs.iter().map(Vec::as_slice)),
        );
    }
    if let (Some(path), Some(json)) = (job.font_report, &font_json) {
        outputs.push((path.to_path_buf(), json.as_bytes()));
    }
    if let (Some(path), Some(json)) = (job.report, &report_json) {
        outputs.push((path.to_path_buf(), json.as_bytes()));
    }
    publish_paths(&outputs, job.force).map_err(|e| e.to_string())?;
    if !job.quiet {
        if let Some(pdf) = &rendered.pdf {
            eprintln!(
                "{}",
                wrote_line(&output, Format::Pdf, pdf.len(), Some(pages))
            );
        }
        if png {
            let pattern = dir.join(format!("{stem}-page-NN.png"));
            eprintln!("{}", wrote_png_line(&pattern, rendered.pngs.len(), job.dpi));
        }
    }
    if job.fail_on_substitution {
        let substituted: Vec<_> = rendered
            .report
            .fonts
            .iter()
            .filter(|f| f.substituted())
            .collect();
        if !substituted.is_empty() {
            for f in &substituted {
                eprintln!(
                    "substituted: {} -> {} ({})",
                    f.requested, f.physical, f.step
                );
            }
            return Err(ConvertFailure {
                code: EXIT_FONT_SUBSTITUTED,
                message: format!(
                    "{} requested font{} substituted (--fail-on-substitution)",
                    substituted.len(),
                    if substituted.len() == 1 {
                        " was"
                    } else {
                        "s were"
                    }
                ),
            });
        }
    }
    Ok(())
}

/// `--pages` as zero-based indices, ascending and without repeats: `1-3,7`
/// is `[0, 1, 2, 6]`. Pages are counted from 1 on the command line.
fn parse_pages(spec: &str) -> Result<Vec<usize>, String> {
    let page = |text: &str| -> Result<usize, String> {
        match text.trim().parse::<usize>() {
            Ok(0) => Err(format!("--pages '{spec}': pages are counted from 1")),
            Ok(n) => Ok(n - 1),
            Err(_) => Err(format!(
                "--pages '{spec}': '{}' is not a page number",
                text.trim()
            )),
        }
    };
    let mut pages = Vec::new();
    for item in spec.split(',') {
        if item.trim().is_empty() {
            return Err(format!("--pages '{spec}': empty item"));
        }
        match item.split_once('-') {
            Some((first, last)) => {
                let (first, last) = (page(first)?, page(last)?);
                if first > last {
                    return Err(format!(
                        "--pages '{spec}': '{}' runs backwards",
                        item.trim()
                    ));
                }
                pages.extend(first..=last);
            }
            None => pages.push(page(item)?),
        }
    }
    pages.sort_unstable();
    pages.dedup();
    Ok(pages)
}

/// What `visual-diff` compares and where it writes.
struct DiffRenderJob<'a> {
    a: &'a Path,
    b: &'a Path,
    out_dir: Option<&'a Path>,
    dpi: f32,
    json: bool,
    overlay: bool,
    force: bool,
}

/// Compare two documents page by page; `Ok(true)` when any page differs.
fn run_diff_render(job: &DiffRenderJob<'_>) -> Result<bool, ConvertFailure> {
    let a = read_word(job.a)?;
    let b = read_word(job.b)?;
    let options = jubarte::convert::DiffOptions {
        dpi: job.dpi,
        overlay: job.overlay && job.out_dir.is_some(),
        ..jubarte::convert::DiffOptions::default()
    };
    let diff = jubarte::convert::diff_render(&a, &b, &options)
        .map_err(|e| format!("visual-diff failed: {e}"))?;
    let changed: Vec<&jubarte::convert::PageDiff> =
        diff.pages.iter().filter(|p| p.differs()).collect();
    /// `diff.json`. Serialized directly (not through `serde_json::Value`)
    /// so the `f32` ratios print at their own precision.
    #[derive(serde::Serialize)]
    struct Summary<'a> {
        a: String,
        b: String,
        dpi: f32,
        a_pages: usize,
        b_pages: usize,
        changed: usize,
        pages: &'a [jubarte::convert::PageDiff],
    }
    let summary = serde_json::to_string(&Summary {
        a: job.a.display().to_string(),
        b: job.b.display().to_string(),
        dpi: job.dpi,
        a_pages: diff.a_report.page_count,
        b_pages: diff.b_report.page_count,
        changed: changed.len(),
        pages: &diff.pages,
    })
    .map_err(|e| format!("diff.json: {e}"))?;
    if let Some(dir) = job.out_dir {
        let count = diff.pages.len();
        let mut files: Vec<(PathBuf, &[u8])> = Vec::new();
        for page in &changed {
            let i = page.index;
            let sides = [
                ("a", diff.a.get(i)),
                ("b", diff.b.get(i)),
                ("diff", diff.overlays[i].as_ref()),
            ];
            for (prefix, png) in sides {
                if let Some(png) = png {
                    files.push((dir.join(png_name(prefix, i, count)), png));
                }
            }
        }
        files.push((dir.join("diff.json"), summary.as_bytes()));
        // Check every path before the first write so a refused file leaves
        // no partial bundle.
        for (path, _) in &files {
            ensure_writable(path, job.force)?;
        }
        std::fs::create_dir_all(dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
        publish_paths(&files, job.force).map_err(|e| e.to_string())?;
    }
    if job.json {
        outln!("{summary}");
    } else {
        for page in &changed {
            match page.only_in {
                Some(side) => outln!("page {}: only in {side}", page.index + 1),
                None => outln!(
                    "page {}: {:.2}% of pixels changed, box {:?}",
                    page.index + 1,
                    page.changed_ratio * 100.0,
                    page.bbox.unwrap_or_default()
                ),
            }
        }
        outln!(
            "{} of {} page{} differ{}",
            changed.len(),
            diff.pages.len(),
            if diff.pages.len() == 1 { "" } else { "s" },
            job.out_dir
                .map(|dir| format!(" (wrote {})", dir.display()))
                .unwrap_or_default()
        );
    }
    Ok(!changed.is_empty())
}

/// `<stem>-page-NN.png` for the zero-based page `index`, zero-padded to the
/// page count's width (at least 2).
fn png_name(stem: &str, index: usize, count: usize) -> String {
    let width = count.to_string().len().max(2);
    format!("{stem}-page-{:0width$}.png", index + 1)
}

/// `read --tables`: each body table as a grid; with `-p` the grids of the
/// tables the view's selection reaches, then that view (#1161), and
/// `--track-changes reject` reads the rejected text. A pick that reaches
/// no table exits 2; what the view refuses, as `read -p` does. `-o FILE`
/// writes the same text to FILE instead of stdout (#1690).
fn run_read_tables(
    file: &Path,
    args: &ReadArgs,
    output: Option<&Path>,
    force: bool,
    quiet: bool,
) -> Result<(), ConvertFailure> {
    let render = RenderArgs::default();
    let out = output.map(|path| Output {
        kind: Format::Md,
        path,
        force,
        quiet,
        input: file,
        markdown_input: false,
        view: args,
        source: String::new(),
        render: &render,
    });
    if let Some(out) = &out {
        refuse_output_alias(out.path, &[(file, "the input")])
            .map_err(|(code, message)| ConvertFailure { code, message })?;
        check_output(out)?;
    }
    let bytes = read_input(file, quiet)?;
    warn_nested_text_boxes(&bytes);
    let view = jubarte::markdown::read_tables(
        &bytes,
        &jubarte::markdown::ReadOptions {
            track_changes: args.track_changes.unwrap_or(TrackChanges::All).into(),
            comments: args.comments == CommentsArg::Inline,
            dates: args.dates,
            page_markers: !args.no_page_markers,
            select: selection(args).map_err(usage_failure)?,
            source: file.file_name().map(|n| n.to_string_lossy().into_owned()),
        },
    )
    .map_err(|e| match e {
        jubarte::markdown::TablesError::Usage(message) => usage_failure(message),
        jubarte::markdown::TablesError::Read(e) => e.to_string().into(),
    })?;
    for warning in &view.warnings {
        eprintln!("warning: {warning}");
    }
    match &out {
        Some(out) => write_text(out, &view.markdown, "tables"),
        None => Ok(write_stdout(view.markdown)?),
    }
}

/// A usage error: exit 2.
fn usage_failure(message: String) -> ConvertFailure {
    ConvertFailure {
        code: EXIT_USAGE,
        message,
    }
}

/// `read --for-edit` (V2-10): the working document template/apply
/// derive — exact working text, `before_sha256`, paired targets and
/// per-operation readiness — as JSON on stdout or in a `.json` file.
/// Usage failures exit 2, a refused basis 3, I/O 1.
fn run_for_edit(
    file: &Path,
    args: &ReadArgs,
    output: Option<&Path>,
    force: bool,
    quiet: bool,
) -> Result<(), ConvertFailure> {
    use jubarte::edit::for_edit::for_edit_json;
    let request = args.for_edit_request();
    let refused = |e: jubarte::edit::EditError| {
        let code = if e.code == "USAGE" {
            EXIT_USAGE
        } else {
            EXIT_PLAN_REFUSED
        };
        ConvertFailure {
            code,
            message: format!("{e}"),
        }
    };
    // A JSON read: the result carries the input's findings (#1340).
    let (bytes, validity) = read_input_json(file).map_err(ConvertFailure::from)?;
    // A file leaves stdout free, so the notice stays, as at `read -o FILE.json`.
    if output.is_some()
        && !quiet
        && let Some(line) = jubarte::validate::input_notice_of(&validity)
    {
        eprintln!("{line}");
    }
    warn_nested_text_boxes(&bytes);
    let json = for_edit_json(&bytes, &request, Some(validity)).map_err(refused)?;
    if let Some(path) = output {
        ensure_writable(path, force)?;
        publish_file(path, json.as_bytes(), force).map_err(ConvertFailure::from)?;
        if !quiet {
            eprintln!("{}", wrote_line(path, Format::Json, json.len(), None));
        }
        return Ok(());
    }
    write_stdout(json).map_err(ConvertFailure::from)
}

/// `read`: the agent view, with page markers from the layout pass unless
/// `--no-page-markers` skips it; or the snapshot (`--json`), or the grids
/// (`--tables`). `-o FILE.md` writes the view, `-o FILE.json` the snapshot.
fn run_text(
    file: &Path,
    args: &ReadArgs,
    output: Option<&Path>,
    force: bool,
    quiet: bool,
) -> Result<(), ConvertFailure> {
    // A selection that does not parse is a usage error, before I/O.
    selection(args).map_err(usage_failure)?;
    if args.for_edit {
        return run_for_edit(file, args, output, force, quiet);
    }
    if args.tables {
        return run_read_tables(file, args, output, force, quiet);
    }
    let source = file.file_name().map(|n| n.to_string_lossy().into_owned());
    if let Some(path) = output {
        // The parser allowed only .md (any unknown extension) and .json.
        let kind = output_format(Task::Read, Some(path), None)?.unwrap_or(Format::Md);
        let render = RenderArgs::default();
        let out = Output {
            kind,
            path,
            force,
            quiet,
            input: file,
            markdown_input: false,
            view: args,
            source: source.unwrap_or_default(),
            render: &render,
        };
        check_output(&out)?;
        if kind == Format::Json {
            // The file holds the bytes --json prints, its input_validity
            // too; stdout is free, so the stderr notice stays.
            let (bytes, validity) = read_input_json(file)?;
            if !quiet && let Some(line) = jubarte::validate::input_notice_of(&validity) {
                eprintln!("{line}");
            }
            warn_nested_text_boxes(&bytes);
            let json = read_json(&bytes, args, None, validity)?;
            return write_output(&out, Produced::Text(&json));
        }
        let bytes = read_input(file, quiet)?;
        warn_nested_text_boxes(&bytes);
        return write_output(&out, Produced::Docx(&bytes));
    }
    if args.json {
        let (bytes, validity) = read_input_json(file)?;
        warn_nested_text_boxes(&bytes);
        return Ok(write_stdout(read_json(&bytes, args, source, validity)?)?);
    }
    let bytes = read_input(file, quiet)?;
    warn_nested_text_boxes(&bytes);
    Ok(print_agent_view(&bytes, source, args)?)
}

/// The options `read --json` and `read -o FILE.json` read the snapshot
/// with: the selection, and the text `--track-changes` settles. The view's
/// other options change nothing in a snapshot.
fn snapshot_options(
    args: &ReadArgs,
    source: Option<String>,
) -> Result<jubarte::markdown::ReadOptions, String> {
    Ok(jubarte::markdown::ReadOptions {
        track_changes: args.track_changes.unwrap_or(TrackChanges::All).into(),
        select: selection(args)?,
        source,
        ..jubarte::markdown::ReadOptions::default()
    })
}

/// The paragraphs `-p`, `--head`, `--tail`, `--changed` and `--by` select.
fn selection(args: &ReadArgs) -> Result<Option<jubarte::markdown::Select>, String> {
    jubarte::markdown::Select::from_flags(
        args.paragraph_spec().as_deref(),
        args.head,
        args.tail,
        args.changed,
        args.by.as_deref(),
    )
}

/// Report the first Word-fatal nest once per invocation, before editing or
/// selecting content. Ring-1 owns branch selection, names and the diagnosis.
fn warn_nested_text_boxes(bytes: &[u8]) {
    // The command reports admission errors through its ordinary fallible
    // reader. Do not run the preliminary DOM-based warning on refused XML.
    if jubarte::admission::admit(bytes, jubarte::admission::InputLimits::default()).is_err() {
        return;
    }
    let Ok(package) = jubarte::opc::PartFs::open(bytes) else {
        return;
    };
    if let Some(finding) = jubarte::validate::ring1(&package)
        .into_iter()
        .find(|finding| finding.code == "NESTED_TEXT_BOX")
    {
        eprintln!("warning: {}", finding.message);
    }
}

/// The agent view of `bytes`, its warnings said on stderr; `source` is the
/// name the header prints.
fn agent_view(
    bytes: &[u8],
    source: Option<String>,
    args: &ReadArgs,
) -> Result<jubarte::markdown::ReadView, String> {
    let select = selection(args)?;
    let view = jubarte::markdown::read(
        bytes,
        &jubarte::markdown::ReadOptions {
            track_changes: args.track_changes.unwrap_or(TrackChanges::All).into(),
            comments: args.comments == CommentsArg::Inline,
            dates: args.dates,
            page_markers: !args.no_page_markers,
            select,
            source,
        },
    )
    .map_err(|e| e.to_string())?;
    for warning in &view.warnings {
        eprintln!("warning: {warning}");
    }
    Ok(view)
}

/// Use the same default view and source basename as `read PATH`.
fn confirmation_view(path: &Path, bytes: &[u8]) -> Result<String, String> {
    let source = path.file_name().map(|n| n.to_string_lossy().into_owned());
    agent_view(bytes, source, &ReadArgs::default()).map(|view| view.markdown)
}

/// Markdown inputs are compared exactly as supplied, without a view header.
fn input_confirmation_view(path: &Path, markdown_input: bool) -> Result<String, String> {
    let bytes = read_document(path)?;
    if markdown_input {
        return String::from_utf8(bytes).map_err(|e| e.to_string());
    }
    confirmation_view(path, &bytes)
}

/// Render the exact buffer that was published, never an edit plan. Failure
/// to read a view cannot turn a completed write into a failed operation.
fn print_success(
    before_path: &Path,
    before: Result<String, String>,
    after_path: &Path,
    published: &[u8],
) -> Result<(), String> {
    let views = before
        .and_then(|before| confirmation_view(after_path, published).map(|after| (before, after)));
    let (before, after) = match views {
        Ok(views) => views,
        Err(error) => {
            edit_status(format_args!(
                "warning: the before/after views cannot be shown: {error}"
            ));
            return Ok(());
        }
    };
    let block = jubarte::cli::success_block(
        &slash_path(before_path),
        &before,
        &slash_path(after_path),
        &after,
    );
    write_stdout(block)
}

/// A redaction's original view would repeat the text we just removed.
fn print_edit_success(
    before_path: &Path,
    source: &[u8],
    after_path: &Path,
    published: &[u8],
    redacted: bool,
) -> Result<(), String> {
    if redacted {
        return match confirmation_view(after_path, published) {
            Ok(after) => write_stdout(format!("SUCCESS:\n{after}")),
            Err(_) => {
                edit_status(format_args!("warning: the published view cannot be shown"));
                Ok(())
            }
        };
    }
    print_success(
        before_path,
        confirmation_view(before_path, source),
        after_path,
        published,
    )
}

/// The agent view of `bytes` on stdout.
fn print_agent_view(bytes: &[u8], source: Option<String>, args: &ReadArgs) -> Result<(), String> {
    write_stdout(&agent_view(bytes, source, args)?.markdown)
}

/// Writes a view, a conversion or a diff to stdout. A reader that stops early
/// (`jubarte FILE | head`) closes the pipe; the output then ends quietly, as
/// `cat`'s does.
fn write_stdout(text: impl AsRef<[u8]>) -> Result<(), String> {
    use std::io::Write;
    let mut out = std::io::stdout().lock();
    match out.write_all(text.as_ref()).and_then(|()| out.flush()) {
        Err(e) if e.kind() != std::io::ErrorKind::BrokenPipe => {
            Err(format!("writing to stdout: {e}"))
        }
        _ => Ok(()),
    }
}

/// Whether `edit` and `add` track their changes.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// Tracked changes: redline, clean copy, patch and report.
    Suggesting,
    /// The edits land directly: clean copy and report only.
    Editing,
}

/// Options for `edit` and `add`.
struct EditJob<'a> {
    file: &'a Path,
    /// The plan's path when `--plan` named one: an input of the command,
    /// protected from output aliases like `file` (V2-05).
    plan: Option<&'a Path>,
    /// `-o` named the document. When it did not, the default name of a
    /// `X_modified.docx` input is that input itself: the documented
    /// in-place iteration, not an output alias (V2-05).
    explicit_output: bool,
    /// One document, or the bundle directory.
    target: &'a EditTarget,
    mode: Mode,
    dry_run: bool,
    /// `--skip-failed`: where the plan of the failed operations goes.
    skip_failed: Option<PathBuf>,
    /// `--require-change`: refuse when the primary has no document effect.
    require_change: bool,
    force: bool,
    pdf: bool,
    png: bool,
    dpi: f32,
    revisions: jubarte::convert::RevisionStyle,
    quiet: bool,
}

/// Exit 3: the plan was refused (stale source, ambiguous anchor, ...); the
/// per-operation report is on stdout and nothing was written.
const EXIT_PLAN_REFUSED: u8 = 3;

/// Exit 2: the operation flags do not describe an operation.
const EXIT_USAGE: u8 = 2;

/// The plan of an `edit` or `add` invocation, with its notes.
struct PlanInput {
    plan: jubarte::edit::EditPlan,
    /// The notes the operation flags left.
    notes: Vec<String>,
    /// The plan's own notes: a field `--plan`'s file names another way.
    plan_notes: Vec<String>,
}

/// The plan of an `edit` or `add` invocation: `--plan`'s file, or the plan
/// the operation flags describe.
fn edit_plan(
    verb: jubarte::edit::flags::Verb,
    plan: Option<&Path>,
    operations: &[FlagOp],
    options: &EditOptions,
    source: &[u8],
) -> Result<PlanInput, (u8, String)> {
    if let Some(path) = plan {
        let json = std::fs::read_to_string(path)
            .map_err(|e| (1, format!("reading {}: {e}", path.display())))?;
        let (plan, plan_notes) = jubarte::edit::EditPlan::from_json_with_notes(&json)
            .map_err(|e| (EXIT_PLAN_REFUSED, e.to_string()))?;
        return Ok(PlanInput {
            plan,
            notes: Vec::new(),
            plan_notes,
        });
    }
    let built = jubarte::edit::flags::plan_from_flags(
        verb,
        operations,
        &options.author,
        options.datetime.as_deref(),
        options.existing_revisions.plan_value(),
        source,
    )
    .map_err(|m| (EXIT_USAGE, m))?;
    Ok(PlanInput {
        plan: built.plan,
        notes: built.notes,
        plan_notes: Vec::new(),
    })
}

/// Prepare `plan` on `source` and publish it as `job` says. The number of
/// operations `--skip-failed` left out of what was written (none on a dry
/// run).
fn run_edit(
    job: &EditJob<'_>,
    plan: &jubarte::edit::EditPlan,
    source: &[u8],
    notes: &[String],
    plan_notes: &[String],
) -> Result<usize, (u8, String)> {
    let fail = |m: String| (1u8, m);
    // Dry run and apply share one preparation path (V2-08): the dry run
    // prepares and checks the same serialized candidate and stops before
    // publication.
    let options = jubarte::edit::PrepareOptions {
        require_change: job.require_change,
        // Refusal hints quote the flags a flags-built plan came from.
        syntax: if job.plan.is_some() {
            jubarte::edit::PlanSyntax::V1
        } else {
            jubarte::edit::PlanSyntax::Flags
        },
        skip_failed: job.skip_failed.is_some(),
        ..jubarte::edit::PrepareOptions::default()
    };
    let refuse = |e: jubarte::edit::EditError| {
        let syntax = if job.plan.is_some() {
            jubarte::edit::refusal::RevisionSyntax::V1Plan
        } else {
            jubarte::edit::refusal::RevisionSyntax::LegacyFlags
        };
        refused(&e.with_revision_syntax(syntax), job.file, source)
    };
    if job.dry_run {
        let mut report =
            jubarte::edit::preview_plan_with(source, plan, &options).map_err(refuse)?;
        report.notes.splice(0..0, plan_notes.iter().cloned());
        write_stdout(report.to_jsonl()).map_err(fail)?;
        // A clean dry run says how to apply it: the same command, with the
        // --force an output already there needs. An output --force cannot
        // replace gets no command, only what to do. The default output is
        // spelled as `-o`, so the command run as printed writes the file
        // the note names (#1847); an X_modified.docx input is updated in
        // place, which `-o` naming the input would refuse (V2-05).
        if report.ok && !job.quiet {
            let in_place = matches!(job.target, EditTarget::Single(path) if !job.explicit_output && path == job.file);
            let default_output = match job.target {
                EditTarget::Single(path) if !job.explicit_output && !in_place => {
                    Some(slash_path(path))
                }
                _ => None,
            };
            let output = default_output.as_deref();
            match taken_edit_output(job) {
                Some(Taken::Blocked(why)) => edit_status(format_args!("note: {why}")),
                Some(Taken::Replaceable(_)) if in_place && !job.force => {
                    edit_status(format_args!(
                        "note: the next command updates {} in place (--force)",
                        slash_path(job.file)
                    ));
                    edit_status(format_args!("next: {}", next_command(true, output)));
                }
                Some(Taken::Replaceable(taken)) if !job.force => {
                    edit_status(format_args!(
                        "note: {} exists; the next command replaces it (--force)",
                        slash_path(&taken)
                    ));
                    edit_status(format_args!("next: {}", next_command(true, output)));
                }
                _ => {
                    if let Some(output) = output {
                        edit_status(format_args!(
                            "note: the next command writes {output}; {} stays as it is",
                            slash_path(job.file)
                        ));
                    }
                    edit_status(format_args!("next: {}", next_command(false, output)));
                }
            }
        }
        return Ok(0);
    }
    // The plan first: a refused plan prints its whole report, whatever the
    // outputs (D15).
    let prepared = jubarte::edit::prepare(source, plan, &options).map_err(refuse)?;
    let skipped = prepared.skipped.len();
    publish_edit(job, plan, source, notes, plan_notes, prepared).map(|()| skipped)
}

/// Write what `prepared` holds where `job` says, with its status lines.
fn publish_edit(
    job: &EditJob<'_>,
    plan: &jubarte::edit::EditPlan,
    source: &[u8],
    notes: &[String],
    plan_notes: &[String],
    prepared: jubarte::edit::PreparedEdit,
) -> Result<(), (u8, String)> {
    let fail = |m: String| (1u8, m);
    let mut result = prepared.result;
    result.report.notes.splice(0..0, plan_notes.iter().cloned());
    let primary = match job.mode {
        Mode::Suggesting => &result.redline,
        Mode::Editing => &result.clean,
    };
    let digest = jubarte::inspect::source_sha256(primary);
    result.report.output_sha256 = Some(digest.clone());
    // The failed operations, as a plan bound to the document written:
    // under suggesting mode that is a redline, whose tracked changes are
    // the plan's own, so the plan keeps them.
    let failed = job
        .skip_failed
        .as_ref()
        .filter(|_| !prepared.skipped.is_empty())
        .map(|path| {
            let revisions = fixes_revisions(job, plan);
            let json = jubarte::edit::failed_plan_json(
                source,
                plan,
                &prepared.skipped,
                &result.report.date,
                &digest,
                revisions,
                job.mode == Mode::Suggesting,
            );
            // What the fixes cannot do on the output, on the skip line.
            let mut notes = Vec::new();
            if plan.update_fields && revisions == jubarte::edit::ExistingRevisions::Keep {
                notes.push(
                    "update_fields left out: it cannot go with existing_revisions keep".to_string(),
                );
            }
            notes.extend(jubarte::edit::keep_notes(&prepared.skipped, revisions));
            FailedPlan {
                path: path.clone(),
                json: json.into_bytes(),
                notes,
            }
        });
    let result = result;
    let dir = match job.target {
        EditTarget::Single(path) => {
            return write_edited(job, path, &result, failed.as_ref(), notes, source);
        }
        EditTarget::Bundle(dir) => dir,
    };
    let input_dir = job
        .file
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    if same_dir(dir, input_dir) {
        return Err(fail(
            "--out-dir must not be the input's own directory".into(),
        ));
    }
    let out_dir = dir.as_path();
    let mut jsonl = result.report.to_jsonl();
    // Render (one layout pass per document) before creating the directory,
    // so a failed render leaves no partial bundle.
    let options = jubarte::convert::PdfOptions {
        compress: true,
        revisions: job.revisions,
        ..jubarte::convert::PdfOptions::default()
    };
    let request = jubarte::convert::RenderRequest {
        pdf: job.pdf,
        png_dpi: job.png.then_some(job.dpi),
        pages: None,
    };
    let mut renders = Vec::new();
    if job.pdf || job.png {
        let docs = match job.mode {
            Mode::Suggesting => vec![("redline", &result.redline), ("clean", &result.clean)],
            Mode::Editing => vec![("clean", &result.clean)],
        };
        for (name, bytes) in docs {
            let rendered = jubarte::convert::render(bytes, options, request.clone())
                .map_err(|e| fail(format!("rendering {name}: {e}")))?;
            renders.push((name, rendered));
        }
        let mut pages = serde_json::Map::new();
        let mut starts = serde_json::Map::new();
        for (name, rendered) in &renders {
            pages.insert((*name).into(), rendered.report.page_count.into());
            starts.insert((*name).into(), rendered.report.page_starts().into());
        }
        let render_line = serde_json::json!({
            "ev": "render",
            "engine": format!("jubarte {}", env!("CARGO_PKG_VERSION")),
            "pages": pages,
            "page_starts": starts,
        });
        insert_before_summary(&mut jsonl, &render_line.to_string());
    }
    let name = job.file.file_name().map_or_else(
        || job.file.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    );
    // Under keep the redline also holds the other party's changes; the
    // patch shows the plan's own.
    let patch_of = if result.report.existing_revisions == jubarte::edit::ExistingRevisions::Keep {
        jubarte::markdown::patch_own_changes
    } else {
        jubarte::markdown::patch_redline
    };
    let patch = patch_of(
        &result.redline,
        &jubarte::markdown::PatchOptions {
            old_name: name.clone(),
            new_name: name,
            owner: jubarte::markdown::Attribution {
                author: result.report.author.clone(),
                date: result.report.date.clone(),
            },
        },
    )
    .map_err(|e| fail(format!("writing the patch: {e}")))?
    .render(jubarte::markdown::DEFAULT_COLUMNS);
    let mut outputs: Vec<(String, Vec<u8>)> = match job.mode {
        Mode::Suggesting => vec![
            ("clean.docx".into(), result.clean.clone()),
            ("redline.docx".into(), result.redline.clone()),
            ("patch.diff".into(), patch.into_bytes()),
        ],
        Mode::Editing => vec![("clean.docx".into(), result.clean.clone())],
    };
    // The failed plan goes with the document it binds to.
    let failed = failed.map(|failed| {
        let name = failed.path.file_name().map_or_else(
            || "plan.failed.json".into(),
            |n| n.to_string_lossy().into_owned(),
        );
        outputs.push((name.clone(), failed.json));
        (out_dir.join(name), failed.notes)
    });
    for (name, rendered) in renders {
        if let Some(pdf) = rendered.pdf {
            outputs.push((format!("{name}.pdf"), pdf));
        }
        let count = rendered.pngs.len();
        for (i, png) in rendered.pngs.into_iter().enumerate() {
            outputs.push((png_name(name, i, count), png));
        }
    }
    // The report names every member and closes with the summary; both are
    // settled before anything is staged.
    let saved: Vec<_> = outputs
        .iter()
        .map(|(name, bytes)| {
            serde_json::json!({"f": name, "bytes": bytes.len(), "sha256": jubarte::inspect::source_sha256(bytes)})
        })
        .collect();
    let save_line = serde_json::json!({"ev": "save", "dir": slash_path(out_dir), "outputs": saved});
    insert_before_summary(&mut jsonl, &save_line.to_string());
    let summary = jsonl.lines().last().unwrap_or("").to_string();
    outputs.push(("report.jsonl".into(), jsonl.into_bytes()));
    // V2-05: a bundle member that is an input refuses the publication;
    // otherwise the bundle is staged in a sibling temp directory and
    // published as a new directory — never merged into the existing one.
    let inputs = job_inputs(job);
    for (name, _) in &outputs {
        refuse_output_alias(&out_dir.join(name), &inputs)?;
    }
    refuse_bundle_input_containment(out_dir, &inputs)?;
    let paths: Vec<PathBuf> = outputs.iter().map(|(name, _)| out_dir.join(name)).collect();
    refuse_output_collisions(&paths)?;
    if out_dir.symlink_metadata().is_ok() && !job.force {
        return Err(taken_output(out_dir));
    }
    if job.force {
        refuse_foreign_bundle_contents(out_dir)?;
    }
    let stage = stage_bundle_dir(out_dir).map_err(fail)?;
    let files: Vec<_> = outputs
        .iter()
        .map(|(name, bytes)| (stage.join(name), bytes.as_slice()))
        .collect();
    if let Err(error) = publish_paths(&files, false) {
        let _ = std::fs::remove_dir_all(&stage);
        return Err(fail(error.to_string()));
    }
    publish_dir(&stage, out_dir, job.force, || {
        for path in &paths {
            refuse_output_alias(path, &inputs)?;
        }
        refuse_bundle_input_containment(out_dir, &inputs)?;
        refuse_output_collisions(&paths)?;
        if job.force {
            refuse_foreign_bundle_contents(out_dir)?;
        }
        recheck_edit_source(job.file, source)
    })?;
    let committed = |e: String| {
        fail(format!(
            "committed {} ({} files), but reporting the result failed: {e}",
            slash_path(out_dir),
            outputs.len()
        ))
    };
    let failed = failed
        .as_ref()
        .map(|(path, notes)| (path.as_path(), notes.as_slice()));
    if job.quiet {
        skipped_status(&result, failed);
        return Ok(());
    }
    // Preserve the existing summary contract for ordinary edits.
    if !is_pure_resolution(plan, &result) {
        write_stdout(format!("{summary}\n")).map_err(committed)?;
    }
    let names: Vec<&str> = outputs.iter().map(|(name, _)| name.as_str()).collect();
    // The status lines on stderr (D6); the summary and the view are data.
    edit_status(format_args!(
        "wrote {} ({} files: {})",
        out_dir.display(),
        names.len(),
        names.join(", ")
    ));
    let primary = match job.mode {
        Mode::Suggesting => &result.redline,
        Mode::Editing => &result.clean,
    };
    written_status(&result, failed);
    for note in edit_notes(notes, &result, primary, source) {
        edit_status(format_args!("{note}"));
    }
    for (name, bytes) in &outputs {
        if Path::new(name).extension().is_some_and(|ext| ext == "docx") {
            print_edit_success(
                job.file,
                source,
                &out_dir.join(name),
                bytes,
                result
                    .report
                    .operations
                    .iter()
                    .any(|op| op.kind == "redact"),
            )
            .map_err(committed)?;
        }
    }
    Ok(())
}

/// Keep typed edit/template output refusals on the CLI's usage exit code.
fn edit_envelope_exit(receipt: &jubarte::result::Envelope) -> u8 {
    if receipt
        .error
        .as_ref()
        .is_some_and(|error| error.code == "OUTPUT_EXISTS")
    {
        EXIT_USAGE
    } else {
        jubarte::result::exit_status_for_envelope(receipt).into()
    }
}

/// An existing output without --force is a usage refusal: exit 2.
/// Usage refusals name paths as identifiers an agent reads back, so they
/// use `slash_path`'s `/` spelling on every OS (#568), not the native one.
fn taken_output(path: &Path) -> (u8, String) {
    (
        EXIT_USAGE,
        format!(
            "OUTPUT_EXISTS: output '{}' already exists (use --force to overwrite)",
            slash_path(path)
        ),
    )
}

/// The inputs an output of `edit` or `add` must never overwrite (V2-05).
fn job_inputs<'a>(job: &EditJob<'a>) -> Vec<(&'a Path, &'static str)> {
    let mut inputs: Vec<(&'a Path, &'static str)> = vec![(job.file, "the input")];
    if let Some(plan) = job.plan {
        inputs.push((plan, "the plan"));
    }
    inputs
}

/// Exit 2, stable code `OUTPUT_ALIAS`: an output that names an input
/// through another spelling, a symlink or a hard link, refused before
/// anything is written (V2-05). `same_path` compares device and inode for
/// existing paths (volume serial and file index on Windows) and keys a
/// path not written yet by its canonicalized parent and lowercase name;
/// there a hard link under a third name is not visible, but a hard link is
/// an existing directory entry, so every alias of an existing input is
/// seen.
fn refuse_output_alias(output: &Path, inputs: &[(&Path, &str)]) -> Result<(), (u8, String)> {
    for (input, what) in inputs {
        if same_path(output, input) {
            return Err((
                EXIT_USAGE,
                format!(
                    "OUTPUT_ALIAS: the output '{}' is the same file as {what} '{}'",
                    slash_path(output),
                    slash_path(input)
                ),
            ));
        }
    }
    Ok(())
}

/// --force authorizes an output replacement, never an input alias. The
/// historical implicit X_modified.docx iteration remains permitted.
fn refuse_edit_output_alias(job: &EditJob<'_>, output: &Path) -> Result<(), (u8, String)> {
    let inplace_default = !job.explicit_output && output == job.file;
    let inputs: Vec<(&Path, &str)> = job_inputs(job)
        .into_iter()
        .filter(|(input, _)| !inplace_default || *input != job.file)
        .collect();
    refuse_output_alias(output, &inputs)
}

/// Compare the exact preparation bytes with a fresh read of the named
/// source. Never replace the preparation bytes with this second read.
fn recheck_edit_source(file: &Path, source: &[u8]) -> Result<(), (u8, String)> {
    let current =
        std::fs::read(file).map_err(|e| (1, format!("rechecking {}: {e}", file.display())))?;
    if jubarte::inspect::source_sha256(source) != jubarte::inspect::source_sha256(&current) {
        return Err((
            EXIT_PLAN_REFUSED,
            format!(
                "SOURCE_CHANGED: '{}' changed since the edit was prepared; nothing was published",
                slash_path(file)
            ),
        ));
    }
    Ok(())
}

/// No two roles of a publication may resolve to the same output file.
fn refuse_output_collisions(paths: &[PathBuf]) -> Result<(), (u8, String)> {
    for (index, output) in paths.iter().enumerate() {
        let previous: Vec<(&Path, &str)> = paths[..index]
            .iter()
            .map(|path| (path.as_path(), "another output"))
            .collect();
        refuse_output_alias(output, &previous)?;
    }
    Ok(())
}

/// A wholesale directory replacement must never remove an input's name.
fn refuse_bundle_input_containment(
    dir: &Path,
    inputs: &[(&Path, &str)],
) -> Result<(), (u8, String)> {
    if let Ok(canonical_dir) = std::fs::canonicalize(dir) {
        for (input, what) in inputs {
            if std::fs::canonicalize(input).is_ok_and(|path| path.starts_with(&canonical_dir)) {
                return Err((
                    EXIT_USAGE,
                    format!(
                        "OUTPUT_ALIAS: the output directory '{}' contains {what} '{}'",
                        slash_path(dir),
                        slash_path(input)
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// Post-commit status is best effort: a closed stderr cannot undo a commit
/// or prevent its result from reaching stdout.
fn edit_status(line: std::fmt::Arguments<'_>) {
    use std::io::Write;
    let _ = writeln!(std::io::stderr().lock(), "{line}");
}

/// `paths` comma-separated, with the `/` spelling the views use.
fn comma_paths(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|p| slash_path(p))
        .collect::<Vec<_>>()
        .join(", ")
}

/// A staging path beside `dest`: the same filesystem, so the commit is a
/// rename; hidden; unique to this process (`n` breaks a tie with the
/// survivor of an interrupted run).
fn staging_path(dest: &Path, n: usize) -> PathBuf {
    let dir = dest
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut name = std::ffi::OsString::from(".");
    name.push(dest.file_name().unwrap_or(std::ffi::OsStr::new("out")));
    name.push(format!(".jubarte-tmp-{}-{n}", std::process::id()));
    dir.join(name)
}

/// Test-only synchronization for filesystem races in debug Unix builds.
/// JUBARTE_TEST_PUBLICATION_HOOK names a local Unix socket owned by the
/// integration test. Send the phase/PID, then wait for one release byte.
/// No hook is available in release builds; no environment variable means
/// no socket access. This never changes the prepared document bytes.
#[cfg(all(unix, debug_assertions))]
fn publication_test_hook(phase: &str) -> Result<(), String> {
    use std::io::{Read, Write};
    let Some(socket) = std::env::var_os("JUBARTE_TEST_PUBLICATION_HOOK") else {
        return Ok(());
    };
    let hook = || -> std::io::Result<()> {
        let mut stream = std::os::unix::net::UnixStream::connect(socket)?;
        stream.set_read_timeout(Some(std::time::Duration::from_secs(30)))?;
        writeln!(stream, "{phase} {}", std::process::id())?;
        stream.read_exact(&mut [0])
    };
    hook().map_err(|e| format!("test-only publication hook: {e}"))
}

#[cfg(not(all(unix, debug_assertions)))]
fn publication_test_hook(_phase: &str) -> Result<(), String> {
    Ok(())
}

/// What a publication rechecks after staging and before each commit
/// (V2-05): an output that became unsafe, or a source that changed since it
/// was read, publishes nothing new. `edit` builds it from its job, the typed
/// edit routes from their own inputs.
struct Recheck<'a> {
    /// `--force`: replace an existing destination instead of refusing it.
    force: bool,
    /// Refuse an output that now resolves to one of the command's inputs.
    alias: &'a dyn Fn(&Path) -> Refused,
    /// Refuse when the source changed since it was read.
    source: &'a dyn Fn() -> Refused,
}

/// A publication check: `Err` carries the exit status and the message.
type Refused = Result<(), (u8, String)>;

/// Stage every output before committing any of them. Without --force,
/// hard-link the staged inode into its destination atomically: even an
/// entry appearing after the recheck cannot be clobbered. A mid-publication
/// failure names the artifacts already committed, without rolling them back.
fn publish_files(
    outputs: &[(PathBuf, &[u8])],
    recheck: &Recheck<'_>,
) -> Result<Vec<PathBuf>, (u8, String)> {
    let mut staged = Vec::new();
    let mut committed: Vec<PathBuf> = Vec::new();
    let publish = |staged: &mut Vec<(jubarte::publication::StagedFile, PathBuf)>,
                   committed: &mut Vec<PathBuf>| {
        for (path, bytes) in outputs {
            let held = jubarte::publication::StagedFile::prepare(path, bytes, recheck.force)
                .map_err(|e| (1, format!("writing {}: {e}", path.display())))?;
            staged.push((held, path.clone()));
        }
        publication_test_hook("staged").map_err(|e| (1, e))?;
        // Recheck the entire set before committing the primary.
        for (path, _) in outputs {
            (recheck.alias)(path)?;
        }
        refuse_output_collisions(
            &outputs
                .iter()
                .map(|(path, _)| path.clone())
                .collect::<Vec<_>>(),
        )?;
        if !recheck.force
            && let Some((path, _)) = outputs
                .iter()
                .find(|(path, _)| path.symlink_metadata().is_ok())
        {
            return Err(taken_output(path));
        }
        for (held, path) in staged.drain(..) {
            (recheck.alias)(&path)?;
            if committed.is_empty() {
                (recheck.source)()?;
            }
            publication_test_hook("checked").map_err(|e| (1, e))?;
            if let Err(error) = report_publication(held.commit(), &mut std::io::stderr().lock()) {
                committed.extend(error.committed.iter().cloned());
                if error.source.kind() == std::io::ErrorKind::AlreadyExists {
                    return Err(taken_output(&path));
                }
                return Err((1, format!("committing {}: {error}", path.display())));
            }
            committed.push(path);
        }
        Ok(())
    };
    let outcome = publish(&mut staged, &mut committed);
    drop(staged);
    if let Err((code, mut message)) = outcome {
        if !committed.is_empty() {
            message += &format!(" (already committed: {})", comma_paths(&committed));
        }
        return Err((code, message));
    }
    Ok(committed)
}

/// Exit 2, stable code `OUTPUT_DIR_NOT_EMPTY`: `--force` replaces an
/// existing `--out-dir` wholesale only when it is an earlier jubarte bundle,
/// every entry named by its `report.jsonl` save line (V2-05), or when it
/// is empty (no earlier bundle/report is required). Anything else
/// in it (a user's file, an input under a subdirectory) would be deleted by
/// the replacement, so the publication refuses before staging.
fn refuse_foreign_bundle_contents(dir: &Path) -> Result<(), (u8, String)> {
    let foreign = foreign_bundle_entries(dir);
    if foreign.is_empty() {
        return Ok(());
    }
    Err((
        EXIT_USAGE,
        format!(
            "OUTPUT_DIR_NOT_EMPTY: '{}' holds what no earlier bundle wrote ({}); {FORCE_BUNDLE_ONLY}",
            dir.display(),
            foreign.join(", ")
        ),
    ))
}

/// What to do about a bundle directory --force will not replace.
const FORCE_BUNDLE_ONLY: &str =
    "--force replaces only a jubarte bundle, so move those files or name another --out-dir";

/// The entries of `dir` no earlier bundle's `report.jsonl` names, sorted,
/// a directory with a trailing `/`; none when `dir` does not exist. A
/// directory that cannot be read is no bundle `--force` may replace: it
/// reads as one foreign `(unreadable: …)` entry (#1431 review).
fn foreign_bundle_entries(dir: &Path) -> Vec<String> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(e) => return vec![format!("(unreadable: {e})")],
    };
    let mut ours: std::collections::BTreeSet<String> = ["report.jsonl".to_string()].into();
    if let Ok(report) = std::fs::read_to_string(dir.join("report.jsonl")) {
        let saved = report
            .lines()
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
            .filter(|line| line["ev"] == "save")
            .flat_map(|line| line["outputs"].as_array().cloned().unwrap_or_default());
        ours.extend(saved.filter_map(|o| o["f"].as_str().map(str::to_string)));
    }
    let mut foreign: Vec<String> = entries
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_ok_and(|t| !t.is_dir()))
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|name| !ours.contains(name))
        .collect();
    foreign.extend(
        std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
            .map(|e| format!("{}/", e.file_name().to_string_lossy())),
    );
    foreign.sort();
    foreign
}

/// The sibling temp directory a bundle is staged in (V2-05). Its parent
/// chain is created first, as `--out-dir` has always allowed nested names.
fn stage_bundle_dir(dir: &Path) -> Result<PathBuf, String> {
    if let Some(parent) = dir.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("creating {}: {e}", parent.display()))?;
    }
    for n in 0..100 {
        let stage = staging_path(dir, n);
        let mut builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        match builder.create(&stage) {
            Ok(()) => return Ok(stage),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(format!("creating {}: {e}", dir.display())),
        }
    }
    Err(format!("creating {}: no free staging name", dir.display()))
}

fn sync_bundle_parent(dir: &Path) -> Result<(), (u8, String)> {
    let parent = dir
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    jubarte::publication::sync_directory(parent).map_err(|e| {
        (
            1,
            format!(
                "durability uncertain: syncing bundle parent: {e} (already committed: {})",
                slash_path(dir)
            ),
        )
    })
}

/// Publish the staged bundle as `dir` (V2-05): a rename when `dir` is
/// free; under --force over an existing directory, a swap that replaces it
/// wholesale — never a merge into its files. Precommit failures preserve the
/// prior bundle; postcommit durability errors name the committed directory.
fn publish_dir(
    stage: &Path,
    dir: &Path,
    force: bool,
    recheck: impl Fn() -> Result<(), (u8, String)>,
) -> Result<(), (u8, String)> {
    let failed = |error: (u8, String)| {
        let _ = std::fs::remove_dir_all(stage);
        error
    };
    publication_test_hook("staged").map_err(|e| failed((1, e)))?;
    if dir.symlink_metadata().is_err() {
        recheck().map_err(failed)?;
        std::fs::rename(stage, dir)
            .map_err(|e| failed((1, format!("publishing {}: {e}", dir.display()))))?;
        return sync_bundle_parent(dir);
    }
    // Alias checks precede a no-clobber refusal, here as in preflight.
    recheck().map_err(failed)?;
    if !force {
        return Err(failed(taken_output(dir)));
    }
    if !dir.is_dir() {
        return Err(failed((
            1,
            format!(
                "--out-dir '{}' is an existing file, not a directory",
                dir.display()
            ),
        )));
    }
    let mut aside = None;
    for n in 100..200 {
        let candidate = staging_path(dir, n);
        if candidate.symlink_metadata().is_ok() {
            continue;
        }
        // The source guard runs immediately before the first commit rename.
        recheck().map_err(failed)?;
        match std::fs::rename(dir, &candidate) {
            Ok(()) => {
                aside = Some(candidate);
                break;
            }
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::AlreadyExists | std::io::ErrorKind::DirectoryNotEmpty
                ) =>
            {
                continue;
            }
            Err(e) => return Err(failed((1, format!("moving aside {}: {e}", dir.display())))),
        }
    }
    let aside = aside.ok_or_else(|| {
        failed((
            1,
            format!("moving aside {}: no free staging name", dir.display()),
        ))
    })?;
    let restore = |error: (u8, String)| {
        let (code, message) = failed(error);
        match std::fs::rename(&aside, dir) {
            Ok(()) => (code, message),
            Err(e) => (
                code,
                format!(
                    "{message}; restoring original bundle from {} to {} failed: {e}",
                    aside.display(),
                    dir.display()
                ),
            ),
        }
    };
    if let Err(error) = recheck() {
        return Err(restore(error));
    }
    if let Err(e) = std::fs::rename(stage, dir) {
        return Err(restore((1, format!("publishing {}: {e}", dir.display()))));
    }
    sync_bundle_parent(dir)?;
    // The old directory is replaced, not merged: drop what it held.
    let dropped = if aside
        .symlink_metadata()
        .is_ok_and(|m| m.file_type().is_symlink())
    {
        std::fs::remove_file(&aside)
    } else {
        std::fs::remove_dir_all(&aside)
    };
    if let Err(e) = dropped {
        edit_status(format_args!(
            "warning: the replaced directory could not be removed; its contents remain at {}: {e}",
            aside.display()
        ));
    }
    Ok(())
}

/// One receipt line per thread, shared by legacy and typed edit publication.
/// `source_for_paragraph` carries the original doc's comments: paragraph
/// lookups read source ids when settlement renumbered the kept threads.
fn settlement_notes(
    removed: &[jubarte::edit::RemovedComment],
    kept: &[jubarte::edit::KeptComment],
    document: &[u8],
    source_for_paragraph: &[u8],
    decision: &str,
) -> Vec<String> {
    let reply_text = |replies: &[String]| match replies {
        [] => String::new(),
        [one] => format!(" and its reply {one}"),
        many => format!(" and its replies {}", many.join(", ")),
    };
    let mut lines = Vec::new();
    for c in removed {
        // A receipt read from an older report has no `why`.
        let why = if c.why.is_empty() {
            let decision = c.decision.as_deref().unwrap_or(decision);
            format!("their anchor was text the {decision} removed")
        } else {
            c.why.clone()
        };
        lines.push(format!(
            "removed {} ({}){}: {why}",
            c.id,
            c.author,
            reply_text(&c.replies)
        ));
    }
    let source_comments =
        jubarte::comments::list_comments(source_for_paragraph).unwrap_or_default();
    let document_comments = jubarte::comments::list_comments(document).unwrap_or_default();
    for c in kept {
        let source_int = c.id.strip_prefix('c').and_then(|s| s.parse::<u32>().ok());
        // Prefer the published doc: an edit plan can move the kept anchor
        // to a different paragraph (a deleted container, a plan delete),
        // and `list_comments` on the published bytes tracks that. CLI
        // `--keep-nested-comment` resolves through a path that renumbers
        // the doc, so the published lookup misses; the source has the
        // anchor's original paragraph.
        let paragraph = source_int
            .and_then(|id| {
                document_comments
                    .iter()
                    .find(|r| r.id == id)
                    .and_then(|r| r.paragraph.as_deref())
            })
            .or_else(|| {
                let id = source_int?;
                source_comments
                    .iter()
                    .find(|r| r.id == id)
                    .and_then(|r| r.paragraph.as_deref())
            })
            .unwrap_or("the document");
        lines.push(format!(
            "kept {}{} on an empty anchor in {paragraph}",
            c.id,
            reply_text(&c.replies)
        ));
    }
    lines
}

/// The `note:` lines of an applied edit: the flags' notes, the plan's (a
/// field read as another), then each anchor that matched as the paragraph
/// reads rather than as typed, and what a strike or a restore left (a
/// mark already struck, a replacement's other half). A kept comment names
/// its paragraph in `published`, the copy the mode writes.
fn edit_notes(
    notes: &[String],
    result: &jubarte::edit::EditResult,
    published: &[u8],
    source: &[u8],
) -> Vec<String> {
    let mut lines: Vec<String> = notes
        .iter()
        .chain(&result.report.notes)
        .map(|note| format!("note: {note}"))
        .collect();
    let decision = if result.report.resolved_revisions.rejected.is_empty() {
        "acceptance"
    } else {
        "rejection"
    };
    lines.extend(settlement_notes(
        &result.report.removed_comments,
        &result.report.kept_comments,
        published,
        source,
        decision,
    ));
    for outcome in &result.report.operations {
        if let (Some(given), Some(read_as)) = (&outcome.anchor_given, &outcome.anchor_read_as) {
            let why = if jubarte::markdown::plain_anchor(given) == *read_as {
                "Markdown marks are not document text"
            } else {
                "matched as it reads: spaces, quotes and hyphens as the paragraph has them"
            };
            lines.push(format!(
                "note: {}: anchor {given:?} read as {read_as:?} ({why})",
                outcome.id
            ));
        }
        if let ("strike" | "restore", Some(message)) = (outcome.kind.as_str(), &outcome.message) {
            lines.push(format!("note: {}: {message}", outcome.id));
        }
    }
    lines
}

/// A pure revision resolution keeps its existing absence of a bundle summary.
fn is_pure_resolution(plan: &jubarte::edit::EditPlan, result: &jubarte::edit::EditResult) -> bool {
    plan.operations.is_empty()
        && plan.resolve_revisions.is_some()
        && !result.report.resolved_revisions.is_empty()
}

/// Publish edit/add outputs, then confirm the document with full agent views.
fn write_edited(
    job: &EditJob<'_>,
    path: &Path,
    result: &jubarte::edit::EditResult,
    failed: Option<&FailedPlan>,
    notes: &[String],
    source: &[u8],
) -> Result<(), (u8, String)> {
    let fail = |m: String| (1u8, m);
    let document = match job.mode {
        Mode::Suggesting => &result.redline,
        Mode::Editing => &result.clean,
    };
    let mut outputs: Vec<(PathBuf, &[u8])> = vec![(path.to_path_buf(), document)];
    // `--skip-failed`: the plan of what was left out, published with the
    // document it binds to.
    if let Some(failed) = failed {
        outputs.push((failed.path.clone(), &failed.json));
    }
    // Render before writing, so a failed render writes nothing.
    let rendered = if job.pdf || job.png {
        let options = jubarte::convert::PdfOptions {
            compress: true,
            revisions: job.revisions,
            ..jubarte::convert::PdfOptions::default()
        };
        let request = jubarte::convert::RenderRequest {
            pdf: job.pdf,
            png_dpi: job.png.then_some(job.dpi),
            pages: None,
        };
        Some(
            jubarte::convert::render(document, options, request)
                .map_err(|e| fail(format!("rendering {}: {e}", path.display())))?,
        )
    } else {
        None
    };
    let dir = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map_or_else(PathBuf::new, Path::to_path_buf);
    let stem = path.file_stem().map_or_else(
        || "document".to_string(),
        |s| s.to_string_lossy().into_owned(),
    );
    let (mut pdf_pages, mut png_count) = (None, 0);
    if let Some(rendered) = &rendered {
        if let Some(pdf) = &rendered.pdf {
            outputs.push((path.with_extension("pdf"), pdf));
            pdf_pages = Some(rendered.report.page_count);
        }
        png_count = rendered.pngs.len();
        for (i, png) in rendered.pngs.iter().enumerate() {
            outputs.push((dir.join(png_name(&stem, i, png_count)), png));
        }
    }
    // V2-05: never write over an input, whatever spelling or link named
    // it; --force authorizes replacing an output, never an input. The
    // in-place default (`X_modified.docx` edited again, without -o) is the
    // command's own name for its input, not an alias of it.
    for (output, _) in &outputs {
        refuse_edit_output_alias(job, output)?;
    }
    refuse_output_collisions(
        &outputs
            .iter()
            .map(|(path, _)| path.clone())
            .collect::<Vec<_>>(),
    )?;
    // The PNG names are known only now: nothing is written over a file.
    if !job.force
        && let Some((taken, _)) = outputs.iter().find(|(p, _)| p.symlink_metadata().is_ok())
    {
        return Err(taken_output(taken));
    }
    // Stage beside each destination, then commit without clobbering unless
    // --force was requested; a precommit failure leaves nothing behind.
    let committed = publish_files(
        &outputs,
        &Recheck {
            force: job.force,
            alias: &|path| refuse_edit_output_alias(job, path),
            source: &|| recheck_edit_source(job.file, source),
        },
    )?;
    let failed = failed.map(|f| (f.path.as_path(), f.notes.as_slice()));
    if job.quiet {
        skipped_status(result, failed);
        return Ok(());
    }
    edit_status(format_args!(
        "{}",
        wrote_line(path, Format::Docx, document.len(), None)
    ));
    if let Some(pages) = pdf_pages {
        let pdf = path.with_extension("pdf");
        let bytes = outputs
            .iter()
            .find(|(p, _)| *p == pdf)
            .map_or(0, |(_, b)| b.len());
        edit_status(format_args!(
            "{}",
            wrote_line(&pdf, Format::Pdf, bytes, Some(pages))
        ));
    }
    if png_count > 0 {
        let pattern = dir.join(format!("{stem}-page-NN.png"));
        edit_status(format_args!(
            "{}",
            wrote_png_line(&pattern, png_count, job.dpi)
        ));
    }
    written_status(result, failed);
    for note in edit_notes(notes, result, document, source) {
        edit_status(format_args!("{note}"));
    }
    print_edit_success(
        job.file,
        source,
        path,
        document,
        result
            .report
            .operations
            .iter()
            .any(|op| op.kind == "redact"),
    )
    .map_err(|e| {
        fail(format!(
            "committed {}, but reporting the result failed: {e}",
            comma_paths(&committed)
        ))
    })
}

/// The plan of what `--skip-failed` left out: where it goes, its JSON,
/// and what it cannot do on the document written (the skip line's notes).
struct FailedPlan {
    path: PathBuf,
    json: Vec<u8>,
    notes: Vec<String>,
}

/// The status lines after a write: the written document's SHA-256, then
/// what `--skip-failed` left out and where its plan went.
fn written_status(result: &jubarte::edit::EditResult, failed: Option<(&Path, &[String])>) {
    if let Some(digest) = &result.report.output_sha256 {
        edit_status(format_args!("sha256: {digest}"));
    }
    skipped_status(result, failed);
}

/// The skip line of `--skip-failed`, a warning even under `-q`: operations
/// asked for are not in the document. Its notes say what the plan of the
/// fixes, which keeps the output's revisions, cannot do.
fn skipped_status(result: &jubarte::edit::EditResult, failed: Option<(&Path, &[String])>) {
    if let Some((path, notes)) = failed {
        let notes = if notes.is_empty() {
            String::new()
        } else {
            format!(" ({})", notes.join("; "))
        };
        edit_status(format_args!(
            "skipped {} of {} operations; the fixes are in {}{notes}",
            result.report.skipped,
            result.report.operations.len(),
            slash_path(path),
        ));
    }
}

/// The `existing_revisions` of the fixes plan: under suggesting mode the
/// document written is a redline, whose tracked changes are the plan's
/// own, so the fixes keep them.
fn fixes_revisions(
    job: &EditJob<'_>,
    plan: &jubarte::edit::EditPlan,
) -> jubarte::edit::ExistingRevisions {
    match job.mode {
        Mode::Suggesting => jubarte::edit::ExistingRevisions::Keep,
        Mode::Editing => plan.existing_revisions,
    }
}

/// An output of `edit` already on disk.
enum Taken {
    /// One --force replaces.
    Replaceable(PathBuf),
    /// One --force is refused over too: why, and what to do instead.
    Blocked(String),
}

/// The output of `job` already on disk, if any: the document or its PDF,
/// or the bundle directory, empty or not (an existing --out-dir is refused
/// without --force). A directory where the document goes, a file where the
/// bundle goes, or a bundle directory holding what no bundle wrote is
/// refused under --force as well. The PNG names are known only once
/// rendered, so a dry run does not look for them.
fn taken_edit_output(job: &EditJob<'_>) -> Option<Taken> {
    match job.target {
        EditTarget::Single(path) => {
            let taken: Vec<PathBuf> = [
                Some(path.clone()),
                job.pdf.then(|| path.with_extension("pdf")),
            ]
            .into_iter()
            .flatten()
            .filter(|p| p.symlink_metadata().is_ok())
            .collect();
            match taken
                .iter()
                .find(|p| p.symlink_metadata().is_ok_and(|m| m.is_dir()))
            {
                Some(dir) => Some(Taken::Blocked(format!(
                    "{} is a directory; name another output",
                    slash_path(dir)
                ))),
                None => taken.into_iter().next().map(Taken::Replaceable),
            }
        }
        EditTarget::Bundle(dir) => {
            dir.symlink_metadata().ok()?;
            let foreign = foreign_bundle_entries(dir);
            Some(if !dir.is_dir() {
                Taken::Blocked(format!(
                    "{} is an existing file, not a directory; name another --out-dir",
                    slash_path(dir)
                ))
            } else if foreign.is_empty() {
                Taken::Replaceable(dir.clone())
            } else {
                Taken::Blocked(format!(
                    "{} holds what no earlier bundle wrote ({}); {FORCE_BUNDLE_ONLY}",
                    slash_path(dir),
                    foreign.join(", ")
                ))
            })
        }
    }
}

/// Where `--skip-failed` writes the failed operations: beside the plan as
/// `PLAN.failed.json`, or beside FILE as `FILE_plan.failed.json` (the name
/// `--plan` records under, D16) when operation flags made the plan.
fn failed_plan_path(plan: Option<&Path>, file: &Path) -> PathBuf {
    let stem = |path: &Path| {
        path.file_stem()
            .map_or_else(String::new, |s| s.to_string_lossy().into_owned())
    };
    match plan {
        Some(plan) => plan.with_file_name(format!("{}.failed.json", stem(plan))),
        None => file.with_file_name(format!("{}_plan.failed.json", stem(file))),
    }
}

/// This invocation without its `--dry-run` flag (`-o OUTPUT` added when
/// `output` names the default it writes, `--force` when `force`), quoted
/// for a POSIX shell: what a clean dry run prints as its `next:` command.
fn next_command(force: bool, output: Option<&str>) -> String {
    let mut args = std::env::args_os().map(|arg| arg.to_string_lossy().into_owned());
    let program = args.next().unwrap_or_default();
    let rest: Vec<String> = args.collect();
    std::iter::once(program)
        .chain(jubarte::cli::next_command_to(&rest, force, output))
        .map(|arg| jubarte::cli::shell_word(&arg))
        .collect::<Vec<_>>()
        .join(" ")
}

/// One `edit` or `add` invocation, parsed.
struct EditFlags<'a> {
    verb: jubarte::edit::flags::Verb,
    file: &'a Path,
    plan: Option<&'a Path>,
    /// `--plan` without a value: FILE_plan.json (D16).
    plan_default: bool,
    consume: bool,
    operations: &'a [FlagOp],
    options: &'a EditOptions,
    dry_run: bool,
    /// `--skip-failed` (`edit` only; `add` passes false).
    skip_failed: bool,
    /// `--require-change` (`edit` only; `add` passes false).
    require_change: bool,
    pdf: bool,
    png: bool,
    dpi: f32,
    revisions: jubarte::convert::RevisionStyle,
    /// `--rewrite-others-comments` (`edit` only; `add` passes false).
    /// Accepted and ignored: it does not turn the guard off.
    rewrite_others_comments: bool,
    /// `--keep-others` (`edit` and `add`).
    keep_others: bool,
    /// The global `-q`.
    quiet: bool,
}

/// `JUBARTE_KEEP_OTHERS=1` in the CLI only. Any other value, including an
/// empty one, leaves the guard off. The library does not read it.
fn env_keep_others() -> bool {
    std::env::var("JUBARTE_KEEP_OTHERS").ok().as_deref() == Some("1")
}

fn run_flags(flags: &EditFlags<'_>) -> Result<(), (u8, String)> {
    let source = read_word(flags.file).map_err(|mut message| {
        if !flags.file.exists() && flags.plan.is_some() && matches!(flags.file.to_str(), Some("apply" | "replace" | "rewrite" | "insert" | "delete")) {
            message.push_str("; this is legacy edit FILE --plan PLAN; for typed apply use: jubarte edit apply FILE --plan PLAN");
        }
        (1u8, message)
    })?;
    warn_nested_text_boxes(&source);
    if let Some(path) = flags.plan
        && !flags.operations.is_empty()
    {
        if flags.skip_failed {
            return Err((
                EXIT_USAGE,
                "--skip-failed applies a plan, and with --plan the operation flags record into it: drop --plan to apply them with --skip-failed, or drop --skip-failed to record".into(),
            ));
        }
        return record_plan(flags, path, &source);
    }
    // What --consume deletes once the edit is written: the plan file as it
    // was read, never a link to it or an output written over it.
    let consumed = match flags.plan.filter(|_| flags.consume && !flags.dry_run) {
        Some(path) => {
            if path.symlink_metadata().is_ok_and(|m| m.is_symlink()) {
                return Err((
                    EXIT_USAGE,
                    format!(
                        "--consume deletes {}, a symbolic link; pass the file it names",
                        path.display()
                    ),
                ));
            }
            let held =
                std::fs::read(path).map_err(|e| (1, format!("reading {}: {e}", path.display())))?;
            Some((path, held))
        }
        None => None,
    };
    let PlanInput {
        mut plan,
        notes,
        plan_notes,
    } = edit_plan(
        flags.verb,
        flags.plan,
        flags.operations,
        flags.options,
        &source,
    )?;
    plan.rewrite_others_comments |= flags.rewrite_others_comments;
    if flags.keep_others || env_keep_others() {
        plan.others = jubarte::edit::Others::Keep;
    }
    if flags.options.editing_mode
        && let Some(message) = jubarte::edit::flags::editing_mode_conflict(&plan, &source)
    {
        return Err((EXIT_USAGE, message));
    }
    let target = flags.options.target(flags.file);
    let job = EditJob {
        file: flags.file,
        plan: flags.plan,
        explicit_output: flags.options.output.is_some(),
        target: &target,
        mode: if flags.options.editing_mode {
            Mode::Editing
        } else {
            Mode::Suggesting
        },
        dry_run: flags.dry_run,
        skip_failed: flags
            .skip_failed
            .then(|| failed_plan_path(flags.plan, flags.file)),
        require_change: flags.require_change,
        force: flags.options.force,
        pdf: flags.pdf,
        png: flags.png,
        dpi: flags.dpi,
        revisions: flags.revisions,
        quiet: flags.quiet,
    };
    let skipped = run_edit(&job, &plan, &source, &notes, &plan_notes)?;
    if let Some((path, _)) = consumed.as_ref().filter(|_| skipped > 0) {
        // Even under -q: the caller asked for the plan gone. The skip line
        // can send them back to it.
        edit_status(format_args!(
            "note: --consume kept {}: {skipped} of its {} operations did not land",
            slash_path(path),
            plan.operations.len()
        ));
        return Ok(());
    }
    if let Some((path, held)) = consumed {
        let kept = |why: String| {
            (
                1,
                format!(
                    "the plan was applied, but {} was not deleted: {why}",
                    path.display()
                ),
            )
        };
        if std::fs::read(path).ok().as_ref() != Some(&held) {
            return Err(kept(
                "it no longer holds the plan (an output was written over it?)".into(),
            ));
        }
        std::fs::remove_file(path).map_err(|e| kept(e.to_string()))?;
    }
    Ok(())
}

/// `--plan PLAN.json` with operation flags: the operations go into the plan
/// (created on first use) instead of into FILE. The merged plan is resolved
/// against FILE first, so it never holds an operation FILE cannot place (it
/// can still fail later, while the redline is compared); a refusal leaves
/// PLAN.json as it was.
fn record_plan(flags: &EditFlags<'_>, path: &Path, source: &[u8]) -> Result<(), (u8, String)> {
    use jubarte::edit::record::{RecordError, record, write_plan};
    let options = flags.options;
    let held = match std::fs::read_to_string(path) {
        Ok(json) => Some(json),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err((1, format!("reading {}: {e}", path.display()))),
    };
    let mut recorded = record(
        flags.verb,
        flags.operations,
        &options.author,
        options.datetime.as_deref(),
        options.existing_revisions.plan_value(),
        source,
        held.as_deref(),
    )
    .map_err(|e| match e {
        RecordError::Usage(m) => (EXIT_USAGE, m),
        RecordError::Plan(m) => (EXIT_PLAN_REFUSED, format!("{}: {m}", path.display())),
    })?;
    recorded.plan.rewrite_others_comments |= flags.rewrite_others_comments;
    if flags.keep_others || env_keep_others() {
        recorded.plan.others = jubarte::edit::Others::Keep;
    }
    jubarte::edit::check_plan_quoting(source, &recorded.plan, jubarte::edit::PlanSyntax::Flags)
        .map_err(|e| {
            refused(
                &e.with_revision_syntax(jubarte::edit::refusal::RevisionSyntax::LegacyFlags),
                flags.file,
                source,
            )
        })?;
    if !flags.quiet {
        for note in &recorded.notes {
            outln!("note: {note}");
        }
    }
    if !flags.dry_run {
        let warnings = write_plan(path, &recorded.json()).map_err(|error| {
            if let Some(publish) = error
                .get_ref()
                .and_then(|source| source.downcast_ref::<jubarte::publication::PublishError>())
            {
                write_publication_warnings(&publish.warnings, &mut std::io::stderr().lock());
            }
            (1, format!("writing {}: {error}", path.display()))
        })?;
        write_publication_warnings(&warnings, &mut std::io::stderr().lock());
    }
    if !flags.quiet {
        outln!(
            "{}",
            recorded.summary(
                &flags.file.display().to_string(),
                &path.display().to_string(),
                flags.plan_default,
                flags.dry_run
            )
        );
    }
    Ok(())
}

fn edit_exit(result: Result<(), (u8, String)>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err((code, message)) => {
            eprintln!("error: {message}");
            ExitCode::from(code)
        }
    }
}

/// A refused plan: its report goes to stdout, the error to stderr, exit 3.
fn refused(e: &jubarte::edit::EditError, file: &Path, source: &[u8]) -> (u8, String) {
    let report_lines: String = e
        .outcomes
        .iter()
        .enumerate()
        .map(|(i, o)| {
            let mut v = serde_json::json!({"ev": "op", "i": i + 1, "id": o.id, "op": o.kind, "status": o.status, "matches": o.matches});
            let m = v.as_object_mut().expect("object");
            if let Some(p) = &o.paragraph {
                m.insert("at".into(), p.clone().into());
            }
            if let Some(c) = &o.context {
                m.insert("ctx".into(), c.clone().into());
            }
            if let Some(near) = &o.near {
                m.insert("near".into(), serde_json::to_value(near).expect("near serializes"));
            }
            if let Some(c) = &o.code {
                m.insert("code".into(), c.clone().into());
            }
            if let Some(msg) = &o.message {
                m.insert("message".into(), msg.clone().into());
            }
            v.to_string() + "\n"
        })
        .collect();
    let mut summary = serde_json::json!({
        "ev": "summary",
        "status": "failed",
        "code": e.code,
        "operation": e.operation,
        "message": e.message,
    });
    let actions =
        e.revision_next_actions(&absolute(file), &jubarte::inspect::source_sha256(source));
    if !actions.is_empty() {
        summary["next_actions"] = serde_json::to_value(actions).expect("actions serialize");
    }
    outln!("{report_lines}{summary}");
    let mut message = format!("plan refused: {e}");
    for near in e.outcomes.iter().filter_map(|o| o.near.as_ref()) {
        message.push('\n');
        message.push_str(&near.copy_hint());
    }
    (EXIT_PLAN_REFUSED, message)
}

/// Insert `line` before the trailing `summary` line of a JSONL report.
fn insert_before_summary(jsonl: &mut String, line: &str) {
    let trimmed = jsonl.trim_end_matches('\n');
    let (head, summary) = match trimmed.rfind('\n') {
        Some(i) => (&trimmed[..i], &trimmed[i + 1..]),
        None => ("", trimmed),
    };
    *jsonl = if head.is_empty() {
        format!("{line}\n{summary}\n")
    } else {
        format!("{head}\n{line}\n{summary}\n")
    };
}

/// A fully-resolved comparison job (positional/named merged).
#[derive(Debug, PartialEq)]
struct Job {
    original: PathBuf,
    modified: PathBuf,
    /// `None`: print the redline's agent view.
    output: Option<PathBuf>,
    /// The options of that view (printed, or written to a `.md`).
    read: ReadArgs,
    /// Whether the command line spelled out any of them.
    view_given: bool,
    /// How a `.pdf` or `.png` output is drawn.
    render: RenderArgs,
    /// `--reference-doc`, `--critic` and `--resource-path`: how Markdown
    /// inputs become Word.
    reference: Option<PathBuf>,
    critic: bool,
    resource_path: Option<PathBuf>,
    author: String,
    date: String,
    force: bool,
    quiet: bool,
    detail_threshold: Option<f64>,
    powertools_faithful: bool,
    no_paragraph_merge: bool,
}

/// Native runtime defaults; required inputs have already been checked by clap.
/// `compare A B` and the shorthand `A B` are one task: without -o the
/// redline's view is printed.
fn resolve_compare(compare: CompareArgs, quiet: bool) -> Job {
    let original = compare
        .original
        .or(compare.original_pos)
        .expect("clap requires ORIGINAL");
    let modified = compare
        .modified
        .or(compare.modified_pos)
        .expect("clap requires MODIFIED");
    Job {
        original,
        modified,
        output: compare.output,
        read: compare.read,
        view_given: compare.view_given,
        render: compare.render,
        reference: compare.reference_doc,
        critic: compare.critic,
        resource_path: compare.resource_path,
        // A change is made now, by Jubarte unless a flag names someone:
        // never the files' own metadata (Arthur, 2026-10-14).
        author: compare.author,
        date: compare
            .datetime
            .unwrap_or_else(jubarte::convert::utc_now_iso8601),
        force: compare.force,
        quiet,
        detail_threshold: compare.detail_threshold,
        powertools_faithful: compare.powertools_faithful || compare.mode == CompareMode::Powertools,
        no_paragraph_merge: compare.no_paragraph_merge,
    }
}

/// Build the default output path: `<original-dir>/<orig-stem>_v_<mod-stem>.docx`.
fn default_output(original: &Path, modified: &Path) -> PathBuf {
    let stem = |p: &Path| {
        p.file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "doc".to_string())
    };
    let name = format!("{}_v_{}.docx", stem(original), stem(modified));
    match original.parent().filter(|p| !p.as_os_str().is_empty()) {
        Some(dir) => dir.join(name),
        None => PathBuf::from(name),
    }
}

/// The comparer's settings for the `--mode`, `--detail-threshold`, author
/// and date flags.
fn comparer_settings(
    author: &str,
    date: &str,
    detail_threshold: Option<f64>,
    powertools_faithful: bool,
    no_paragraph_merge: bool,
) -> jubarte::comparer::WmlComparerSettings {
    let base = if powertools_faithful {
        jubarte::comparer::WmlComparerSettings::powertools_faithful()
    } else {
        jubarte::comparer::WmlComparerSettings::default()
    };
    jubarte::comparer::WmlComparerSettings {
        author_for_revisions: author.to_string(),
        date_time_for_revisions: date.to_string(),
        detail_threshold: detail_threshold.unwrap_or(base.detail_threshold),
        merge_replaced_paragraphs: if no_paragraph_merge {
            false
        } else {
            base.merge_replaced_paragraphs
        },
        ..base
    }
}

fn run(job: &Job) -> Result<(), ConvertFailure> {
    let kind = output_format(Task::Compare, job.output.as_deref(), None)?;
    let source = format!(
        "{} (not written; -o keeps it)",
        // An identifier agents read back: `/` on every OS (#568).
        slash_path(&default_output(&job.original, &job.modified))
    );
    let mut out = job.output.as_deref().map(|path| Output {
        kind: kind.unwrap_or(Format::Docx),
        path,
        force: job.force,
        quiet: job.quiet,
        input: &job.original,
        markdown_input: false,
        view: &job.read,
        source: source.clone(),
        render: &job.render,
    });
    let original = read_document(&job.original)?;
    let modified = read_document(&job.modified)?;
    let settings = comparer_settings(
        &job.author,
        &job.date,
        job.detail_threshold,
        job.powertools_faithful,
        job.no_paragraph_merge,
    );
    let old = Input::new(
        &job.original,
        Format::of_input(None, &job.original, &original),
        original,
    )?;
    let new = Input::new(
        &job.modified,
        Format::of_input(None, &job.modified, &modified),
        modified,
    )?;
    // A choice the documents read make void is a usage error, before any
    // output is checked.
    let usage = |message: String| ConvertFailure { code: 2, message };
    let markdown_pair = old.markdown.is_some() && new.markdown.is_some();
    if markdown_pair && job.view_given && out.as_ref().is_some_and(|o| o.kind == Format::Md) {
        return Err(usage(
            "read options shape the view; two Markdown documents write CriticMarkup to a .md"
                .to_string(),
        ));
    }
    if let Some(out) = &mut out {
        out.markdown_input = old.markdown.is_some();
    }
    let word_pair = old.markdown.is_none() && new.markdown.is_none();
    if word_pair
        && let Some(flag) = [
            (job.reference.is_some(), "--reference-doc"),
            (job.critic, "--critic"),
            (job.resource_path.is_some(), "--resource-path"),
        ]
        .into_iter()
        .find_map(|(given, flag)| given.then_some(flag))
    {
        return Err(usage(format!(
            "{flag} applies to Markdown inputs; both documents are Word"
        )));
    }
    if let Some(out) = &out {
        check_output(out)?;
    }
    // Two Markdown documents written to a .md: CriticMarkup, not the view.
    if let (Some(out), Some(old_text), Some(new_text)) = (&out, &old.markdown, &new.markdown)
        && out.kind == Format::Md
    {
        let critic = jubarte::markdown::diff_markdown(old_text, new_text);
        return write_output(out, Produced::Text(&critic));
    }
    let redline = if word_pair {
        jubarte::document_comparer::compare_documents_with_settings(
            &old.bytes, &new.bytes, &settings,
        )
        .map_err(|e| refusal("compare", &e).unwrap_or_else(|| format!("compare failed: {e:?}")))?
    } else {
        let reference = job.reference.as_deref().map(read_document).transpose()?;
        // Images of a Markdown side are read next to it (both sides share
        // --resource-path when given).
        let markdown_side = if old.markdown.is_some() {
            &job.original
        } else {
            &job.modified
        };
        let refusals = ImageRefusals::default();
        let loader = image_loader(
            &resource_dir(markdown_side, job.resource_path.as_deref()),
            std::rc::Rc::clone(&refusals),
        );
        let options = jubarte::markdown::RedlineOptions {
            settings,
            reference: reference.as_deref(),
            critic: job.critic,
            images: Some(&loader),
        };
        let docx = jubarte::markdown::redline(old.source(), new.source(), &options)
            .map_err(|e| format!("compare failed: {e}"))?;
        // `redline` drops the writer's warnings; the refusals its loader
        // recorded become stderr's (#617).
        for warning in image_refusal_warnings(&refusals) {
            eprintln!("warning: {warning}");
        }
        docx
    };
    match &out {
        None => Ok(print_agent_view(&redline, Some(source), &job.read)?),
        Some(out) => write_output(out, Produced::Docx(&redline)),
    }
}

/// Shared `Result → ExitCode` mapping for every command arm.
fn exit_code(r: Result<(), String>) -> ExitCode {
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            say_error(&e, 1);
            ExitCode::FAILURE
        }
    }
}

/// Set when the command line asks for JSON (`--json` before any `--`): an
/// error then prints on stdout too, as one
/// [`jubarte::cli::json_error`] line, for a reader that pipes stdout to a
/// JSON parser (#1158).
static JSON_ERRORS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Says `message` as the error a run ends with `exit` for: `error: …` on
/// stderr, and under `--json` its JSON object on stdout too.
fn say_error(message: &str, exit: u8) {
    if JSON_ERRORS.load(std::sync::atomic::Ordering::Relaxed) {
        // stdout may be closed already; stderr still says it.
        let _ = write_stdout(jubarte::cli::runtime_json_error(message, exit));
    }
    eprintln!("error: {message}");
}

/// A document read for comparison: Word bytes or Markdown text.
struct Input {
    bytes: Vec<u8>,
    markdown: Option<String>,
}

impl Input {
    fn new(path: &Path, format: Format, bytes: Vec<u8>) -> Result<Self, String> {
        let markdown = match format {
            Format::Md => Some(markdown_text(path, bytes.clone())?),
            Format::Docx => {
                admit_word(path, &bytes)?;
                None
            }
            Format::Pdf | Format::Png | Format::Json => {
                return Err(format!(
                    "{}: PDF, PNG and JSON are not inputs",
                    path.display()
                ));
            }
        };
        Ok(Self { bytes, markdown })
    }

    fn source(&self) -> jubarte::markdown::Source<'_> {
        match &self.markdown {
            Some(text) => jubarte::markdown::Source::Markdown(text),
            None => jubarte::markdown::Source::Docx(&self.bytes),
        }
    }
}

/// Reads an input document, refusing the OLE files Word alone can open and
/// RTF by their signature (`admission::sniff`, the check the library and
/// the bindings run too).
fn read_document(path: &Path) -> Result<Vec<u8>, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    if let Some(refused) = jubarte::cli::refused_input(path, &bytes, false) {
        // Only a .doc that converts gets the hint; an encrypted document
        // or another OLE file would be sent to a command that fails.
        let hint = if jubarte::legacy_doc::read(&bytes).is_ok() {
            format!(
                "; a .doc converts with: jubarte convert {} -o {}",
                path.display(),
                path.with_extension("docx").display()
            )
        } else {
            String::new()
        };
        return Err(format!("{refused}{hint}"));
    }
    Ok(bytes)
}

/// [`read_document`] for an input that must be Word: a file that is not a
/// ZIP package is refused by name (`admission::sniff_package`).
fn read_word(path: &Path) -> Result<Vec<u8>, String> {
    let bytes = read_document(path)?;
    admit_word(path, &bytes)?;
    Ok(bytes)
}

/// [`read_word`] for an input a task must open before it writes anything
/// (`validate --original`): the package is admitted whole
/// ([`jubarte::admission::admit`]), so a ZIP that holds no Word main part
/// is refused by name, `CODE: FILE is …`, before any output exists.
fn read_package(path: &Path) -> Result<Vec<u8>, String> {
    let bytes = read_word(path)?;
    jubarte::cli::refused_package(path, &bytes).map_or(Ok(bytes), Err)
}

/// [`read_word`] for a task that reads the document (`read`, `changes`,
/// `comments`): unless `quiet`, one stderr line says how many validity
/// findings the input has ([`jubarte::validate::input_notice`]), so they
/// are not taken for the reader's own, and where `validate` lists them.
fn read_input(path: &Path, quiet: bool) -> Result<Vec<u8>, String> {
    let bytes = read_word(path)?;
    if !quiet
        && let Ok(findings) = jubarte::validate::validate(&bytes)
        && let Some(line) = jubarte::validate::input_notice(&findings, &path.to_string_lossy())
    {
        eprintln!("{line}");
    }
    Ok(bytes)
}

/// `read --json`'s snapshot of `bytes`, one line, its `summary` carrying
/// the input's `validity` (#1309).
fn read_json(
    bytes: &[u8],
    args: &ReadArgs,
    source: Option<String>,
    validity: jubarte::validate::InputValidity,
) -> Result<String, ConvertFailure> {
    let mut snapshot = jubarte::markdown::read_snapshot(bytes, &snapshot_options(args, source)?)
        .map_err(|e| e.to_string())?;
    snapshot.summary.input_validity = Some(validity);
    let json = serde_json::to_string(&snapshot).map_err(|e| e.to_string())?;
    Ok(format!("{json}\n"))
}

/// [`read_word`] for a JSON read (`read --json`, `changes --json`,
/// `comments --json`): no stderr line, which would break `--json 2>&1 |
/// jq`; the input's findings come back for the output to carry (#1309),
/// under `-q` too. A package that cannot be validated has none.
fn read_input_json(path: &Path) -> Result<(Vec<u8>, jubarte::validate::InputValidity), String> {
    let bytes = read_word(path)?;
    let validity = jubarte::validate::input_validity_of(&bytes, &path.to_string_lossy());
    Ok((bytes, validity))
}

/// `CODE: FILE is …` when `bytes`, read from `path`, are no Word package.
fn admit_word(path: &Path, bytes: &[u8]) -> Result<(), String> {
    jubarte::cli::refused_input(path, bytes, true).map_or(Ok(()), Err)
}

/// `{what} failed: CODE: …` for an admission refusal, whatever wrapped it
/// (`admission::code_first`); `None` for any other error.
fn refusal(what: &str, error: &impl std::fmt::Display) -> Option<String> {
    jubarte::admission::code_first(&error.to_string()).map(|m| format!("{what} failed: {m}"))
}

/// Markdown bytes as text, without a byte order mark.
fn markdown_text(path: &Path, bytes: Vec<u8>) -> Result<String, String> {
    let text = String::from_utf8(bytes)
        .map_err(|_| format!("{}: Markdown must be UTF-8", path.display()))?;
    Ok(text
        .strip_prefix('\u{FEFF}')
        .map(str::to_string)
        .unwrap_or(text))
}

/// `(name, reason)` for every image load [`image_loader`] refuses, so the
/// CLI can say why an image became its alt text: at `convert` the reason
/// annotates the writer's own warning, and `redline` / `patch_documents`
/// drop the writer's warnings, so `compare` and `diff` print this
/// collection themselves (#617).
type ImageRefusals = std::rc::Rc<std::cell::RefCell<Vec<(String, &'static str)>>>;

/// `path` names a URL (`scheme:…`): a scheme is a letter, then letters,
/// digits and `+-.`, up to its `:`. A decoded absolute path (`C:\…`) is
/// not a scheme; the containment check judges it.
fn url_scheme(path: &str) -> bool {
    let Some((scheme, _)) = path.split_once(':') else {
        return false;
    };
    scheme
        .bytes()
        .next()
        .is_some_and(|b| b.is_ascii_alphabetic())
        && scheme
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'-' | b'.'))
}

/// `name`'s image path, its `%XX` escapes decoded and resolved against
/// `root`, as the path inside `root` it lands on, or the reason it is
/// refused. A URL is never fetched; an absolute path, a `..` walk or a
/// symlink whose target leaves `root` is refused (#617).
fn image_policy(root: &Path, name: &str) -> Result<PathBuf, &'static str> {
    let mut decoded = Vec::with_capacity(name.len());
    let bytes = name.as_bytes();
    let mut at = 0;
    while at < bytes.len() {
        let hex = bytes
            .get(at + 1..at + 3)
            .and_then(|h| std::str::from_utf8(h).ok())
            .and_then(|h| u8::from_str_radix(h, 16).ok());
        match (bytes[at], hex) {
            (b'%', Some(byte)) => {
                decoded.push(byte);
                at += 3;
            }
            (byte, _) => {
                decoded.push(byte);
                at += 1;
            }
        }
    }
    let named = std::str::from_utf8(&decoded).map_err(|_| "not found")?;
    let path = PathBuf::from(named);
    // Decoded first, so an encoded scheme (`file%3A%2F%2F…`, `DATA%3A`)
    // is judged as what it names, not as an opaque relative path.
    if !path.is_absolute() && url_scheme(named) {
        return Err("a URL scheme");
    }
    // Canonicalized, `..` and links resolve to what the name really
    // names; the prefix check keeps the read inside `root`.
    let target = root.join(path).canonicalize().map_err(|_| "not found")?;
    target
        .strip_prefix(root)
        .map(Path::to_path_buf)
        .map_err(|_| "outside the resource root")
}

/// Reads images a Markdown file names: local paths, relative to `dir`,
/// with `%XX` escapes decoded first. A decoded target that names a URL is
/// never fetched, and the path it names must stay inside `dir` once
/// symlinks are resolved, so a hostile document cannot read files its
/// folder does not hold (#617). `--resource-path` names the trusted root
/// the containment is checked against; it may sit anywhere. `None` —
/// the image written as its alt text — records its reason in `refusals`.
fn image_loader(dir: &Path, refusals: ImageRefusals) -> impl Fn(&str) -> Option<Vec<u8>> + use<> {
    // Both sides of the containment check must be canonical: a resource
    // root reached through a symlink is its target, or no image inside
    // it would count as inside. The open root is the read's capability:
    // resolving inside it, a link swapped in after the canonicalization
    // cannot point the read out of it.
    let root = dir
        .canonicalize()
        .and_then(|root| jubarte::publication::OutputRoot::open(&root).map(|cap| (root, cap)))
        .ok();
    move |name: &str| {
        let loaded = root.as_ref().ok_or("not found").and_then(|(root, cap)| {
            image_policy(root, name).and_then(|inside| cap.read(&inside).map_err(|_| "not found"))
        });
        match loaded {
            Ok(bytes) => Some(bytes),
            Err(reason) => {
                refusals.borrow_mut().push((name.to_string(), reason));
                None
            }
        }
    }
}

/// The image loads `markdown`'s images would be refused under the root
/// `dir` names, recorded like [`image_loader`]'s. `diff`'s text patch of
/// two Markdown documents runs no loader — its images are never read —
/// but it still says which loads its policy would refuse (#617). The
/// `drawing:` and `data:` payloads the writer resolves inline are left
/// out, exactly as the loader never sees them.
fn report_image_refusals(dir: &Path, markdown: &[&str], refusals: &ImageRefusals) {
    let Some(root) = dir.canonicalize().ok() else {
        return;
    };
    for target in markdown
        .iter()
        .flat_map(|text| jubarte::markdown::image_targets(text))
        .filter(|target| !(target.starts_with("drawing:") || target.starts_with("data:")))
    {
        if let Err(reason) = image_policy(&root, &target) {
            refusals.borrow_mut().push((target, reason));
        }
    }
}

/// The refusals an [`image_loader`] recorded, as the writer phrases its
/// fallback, for the compare and diff paths whose library calls drop
/// `written.warnings` (#617).
fn image_refusal_warnings(refusals: &ImageRefusals) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    refusals
        .borrow()
        .iter()
        .filter(|refusal| seen.insert((*refusal).clone()))
        .map(|(name, reason)| format!("image '{name}' was written as its alt text ({reason})"))
        .collect()
}

/// `warning` with its reason when it is an image written as its alt text
/// [`image_loader`] refused; `convert` annotates the writer's warnings
/// rather than printing its collection a second time (#617).
fn image_warning_reason(warning: &str, refusals: &ImageRefusals) -> String {
    let reason = warning
        .strip_prefix("image '")
        .and_then(|tail| tail.strip_suffix("' was written as its alt text"))
        .and_then(|name| {
            refusals
                .borrow()
                .iter()
                .find(|(refused, _)| refused == name)
                .map(|(_, reason)| *reason)
        });
    match reason {
        Some(reason) => format!("{warning} ({reason})"),
        None => warning.to_string(),
    }
}

/// The directory a Markdown file's images are read from.
fn resource_dir(markdown: &Path, resource_path: Option<&Path>) -> PathBuf {
    resource_path.map(Path::to_path_buf).unwrap_or_else(|| {
        markdown
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
    })
}

/// Options for `diff`: text only (D2).
struct DiffJob<'a> {
    old: &'a Path,
    new: &'a Path,
    output: Option<&'a Path>,
    from: Option<Format>,
    force: bool,
    /// The patch's owner (`--author`, `--datetime`).
    settings: jubarte::comparer::WmlComparerSettings,
    /// Print the patch, wrapped at these columns; `None` for `--format
    /// critic`.
    patch: Option<usize>,
    /// `-q`: say nothing of what was written.
    quiet: bool,
}

/// `diff -o FILE`: text, through the shared writer; its existing file is
/// refused before any work.
fn diff_output<'a>(
    output: Option<&'a Path>,
    input: &'a Path,
    render: &'a RenderArgs,
    force: bool,
    quiet: bool,
) -> Result<Option<Output<'a>>, ConvertFailure> {
    let Some(path) = output else {
        return Ok(None);
    };
    let out = Output::of_task(Format::Md, path, input, render, force, quiet);
    check_output(&out)?;
    Ok(Some(out))
}

/// `text` on stdout, or written to `out`.
fn print_or_write(out: Option<&Output<'_>>, text: &str, label: &str) -> Result<(), ConvertFailure> {
    match out {
        Some(out) => write_text(out, text, label),
        None => write_stdout(text).map_err(ConvertFailure::from),
    }
}

/// The text views (github, word, normal, context, side-by-side) use the
/// shared core directly, with no redline pass.
fn run_text_diff(
    job: &DiffJob<'_>,
    options: &jubarte::text_diff::TextOptions,
) -> Result<(), ConvertFailure> {
    let render = RenderArgs::default();
    let out = diff_output(job.output, job.old, &render, job.force, job.quiet)?;
    let (old, new) = diff_inputs(job)?;
    let view = jubarte::text_diff::diff_documents_view(old.source(), new.source(), options)?;
    use jubarte::text_diff::TextFormat;
    let label = match options.format {
        TextFormat::Github => "github diff",
        TextFormat::Word => "word diff",
        TextFormat::Normal => "normal diff",
        TextFormat::Context => "context diff",
        TextFormat::SideBySide => "side-by-side diff",
    };
    print_or_write(out.as_ref(), &view, label)
}

/// The two documents of a `diff`, each read as its name or bytes say.
fn diff_inputs(job: &DiffJob<'_>) -> Result<(Input, Input), String> {
    let (old_bytes, new_bytes) = (read_document(job.old)?, read_document(job.new)?);
    let old = Input::new(
        job.old,
        Format::of_input(job.from, job.old, &old_bytes),
        old_bytes,
    )?;
    let new = Input::new(
        job.new,
        Format::of_input(job.from, job.new, &new_bytes),
        new_bytes,
    )?;
    Ok((old, new))
}

/// `diff`'s paragraph patch, or `--format critic`'s CriticMarkup (two
/// Markdown documents only), on stdout or in `-o`'s text file.
fn run_diff(job: &DiffJob<'_>) -> Result<(), ConvertFailure> {
    let render = RenderArgs::default();
    let out = diff_output(job.output, job.old, &render, job.force, job.quiet)?;
    let (old, new) = diff_inputs(job)?;
    let refusals = ImageRefusals::default();
    // The patch of two Markdown documents runs no loader — its images
    // are never read — but it still reports the loads the policy would
    // refuse; a mixed pair's loader records its own (#617).
    if let (Some(old_text), Some(new_text)) = (&old.markdown, &new.markdown) {
        report_image_refusals(
            &resource_dir(job.old, None),
            &[old_text, new_text],
            &refusals,
        );
    }
    let text = match job.patch {
        Some(columns) => {
            // Images of a Markdown side are read next to it.
            let markdown_side = if old.markdown.is_some() {
                job.old
            } else {
                job.new
            };
            let loader = image_loader(
                &resource_dir(markdown_side, None),
                std::rc::Rc::clone(&refusals),
            );
            let options = jubarte::markdown::RedlineOptions {
                settings: job.settings.clone(),
                reference: None,
                critic: false,
                images: Some(&loader),
            };
            let name = |path: &Path| path.display().to_string();
            jubarte::markdown::patch_documents(
                old.source(),
                new.source(),
                &options,
                &jubarte::markdown::PatchOptions {
                    old_name: name(job.old),
                    new_name: name(job.new),
                    owner: jubarte::markdown::Attribution {
                        author: job.settings.author_for_revisions.clone(),
                        date: job.settings.date_time_for_revisions.clone(),
                    },
                },
            )
            .map_err(|e| format!("compare failed: {e}"))?
            .render(columns)
        }
        None => match (&old.markdown, &new.markdown) {
            (Some(old), Some(new)) => jubarte::markdown::diff_markdown(old, new),
            _ => {
                return Err(
                    "--format critic needs both documents in Markdown; `jubarte compare OLD NEW` prints their redline as the agent view"
                        .to_string()
                        .into(),
                );
            }
        },
    };
    let label = if job.patch.is_some() {
        "patch diff"
    } else {
        "critic"
    };
    // `patch_documents` drops the writer's warnings; the refusals its
    // loader recorded become stderr's (#617).
    for warning in image_refusal_warnings(&refusals) {
        eprintln!("warning: {warning}");
    }
    print_or_write(out.as_ref(), &text, label)
}

/// `convert`, for every pair of formats it takes.
/// `markdown` (read from `docx`) with `<!-- page N of M -->` lines: the
/// document is laid out as its PDF would be, with its changes kept,
/// accepted or rejected as the Markdown has them, and each block found on
/// its page. When layout fails the Markdown is written without markers and
/// stderr says why.
fn paginated(
    docx: &[u8],
    markdown: &str,
    track_changes: TrackChanges,
    revisions: jubarte::convert::RevisionStyle,
) -> String {
    let (text, warning) =
        jubarte::markdown::with_page_markers(docx, markdown, track_changes.into(), revisions);
    if let Some(warning) = warning {
        eprintln!("warning: {warning}");
    }
    text
}

fn run_convert_any(
    job: &ConvertJob<'_>,
    markdown: &MarkdownArgs,
    update_fields: bool,
) -> Result<(), ConvertFailure> {
    let raw =
        std::fs::read(job.file).map_err(|e| format!("reading {}: {e}", job.file.display()))?;
    // A Word 97-2003 `.doc` is read into a `.docx` first; everything after
    // sees that package. An encrypted document is still refused.
    let legacy = jubarte::legacy_doc::is_compound_file(&raw);
    let bytes = if legacy {
        jubarte::legacy_doc::doc_to_docx(&raw).map_err(|e| format!("convert failed: {e}"))?
    } else {
        jubarte::admission::sniff(&raw)
            .map_err(|refused| format!("{} is {}", job.file.display(), refused.message))?;
        raw
    };
    let from = if legacy {
        Format::Docx
    } else {
        Format::of_input(markdown.from, job.file, &bytes)
    };
    if from == Format::Docx && !legacy {
        admit_word(job.file, &bytes)?;
    }
    let to = job
        .to
        .or_else(|| job.output.and_then(Format::of_path))
        .unwrap_or(match from {
            Format::Md => Format::Docx,
            _ if legacy => Format::Docx,
            _ => Format::Pdf,
        });
    // clap refused any other output; a Markdown input is the one left.
    if update_fields && from != Format::Docx {
        return Err("--update-fields needs a Word document in".into());
    }
    // Word and Markdown output lay nothing out, so these would be ignored.
    if matches!(to, Format::Docx | Format::Md) {
        for (given, flag) in [
            (job.report.is_some() && !update_fields, "--report"),
            (job.font_report.is_some(), "--font-report"),
            (job.fail_on_substitution, "--fail-on-substitution"),
            (
                job.options.comments == jubarte::convert::CommentPlacement::End,
                "--move-comments",
            ),
            (job.options.comment_notes, "--comment-notes"),
            (job.options.changed_only, "--changed-pages"),
            (job.pages.is_some(), "--pages"),
        ] {
            if given {
                return Err(format!("{flag} applies to PDF or PNG output only").into());
            }
        }
    }
    let converted = legacy.then_some(bytes.as_slice());
    if from == Format::Docx && to == Format::Md {
        warn_nested_text_boxes(&bytes);
    }

    // Pre-settle the .docx when keep_nested_comment is on (a wholly removed
    // thread becomes an empty anchor the convert pipeline then renders).
    // The receipt is printed after a successful output publication; the
    // helper only resolves. Every input that yields a writable docx goes
    // through the keeper: the .doc and Markdown paths build one first.
    let needs_keep = markdown.keep_nested_comment
        && matches!(from, Format::Docx | Format::Md)
        && matches!(
            markdown.track_changes,
            TrackChanges::Accept | TrackChanges::Reject
        );
    type KeepResolver = fn(&[u8]) -> Result<jubarte::changes::ResolvedChanges, String>;
    let keep_resolver: Option<KeepResolver> = if needs_keep {
        Some(match markdown.track_changes {
            TrackChanges::Accept => |b: &[u8]| {
                jubarte::document_comparer::accept_revisions_keeping_comments(b)
                    .map_err(|e| format!("convert failed: {e}"))
            },
            TrackChanges::Reject => |b: &[u8]| {
                jubarte::document_comparer::reject_revisions_keeping_comments(b)
                    .map_err(|e| format!("convert failed: {e}"))
            },
            _ => unreachable!("needs_keep gates this"),
        })
    } else {
        None
    };
    // Held across every match arm; printed only after a successful run.
    let mut keep_receipts: Vec<String> = Vec::new();

    let render = RenderArgs::default();
    // Where a Word, Markdown or JSON output goes: -o, else FILE.<ext>.
    let output_at = |kind: Format| {
        job.output
            .map_or_else(|| job.file.with_extension(kind.name()), Path::to_path_buf)
    };
    let at = |kind: Format, path: &Path, produced: Option<Produced<'_>>| {
        let mut out = Output::of_task(kind, path, job.file, &render, job.force, job.quiet);
        out.markdown_input = from == Format::Md;
        match produced {
            Some(produced) => write_output(&out, produced),
            None => Ok(check_output(&out)?),
        }
    };
    let pdf_job = |bytes: Option<&[u8]>, to: Format| {
        run_convert(&ConvertJob {
            bytes,
            to: Some(to),
            ..*job
        })
    };
    let result: Result<(), ConvertFailure> = match (from, to) {
        (Format::Docx, Format::Json) => {
            let path = output_at(Format::Json);
            at(Format::Json, &path, None)?;
            let resolved = if let Some(resolve) = keep_resolver {
                let r = resolve(&bytes)?;
                keep_receipts.extend(settlement_notes(
                    &[],
                    &r.kept_comments,
                    &r.document,
                    &bytes,
                    "settlement",
                ));
                r.document
            } else {
                match markdown.track_changes {
                    TrackChanges::All => bytes,
                    TrackChanges::Accept => jubarte::document_comparer::accept_revisions(&bytes)
                        .map_err(|e| {
                            refusal("convert", &e)
                                .unwrap_or_else(|| format!("convert failed: {e:?}"))
                        })?,
                    TrackChanges::Reject => jubarte::document_comparer::reject_revisions(&bytes)
                        .map_err(|e| {
                            refusal("convert", &e)
                                .unwrap_or_else(|| format!("convert failed: {e:?}"))
                        })?,
                }
            };
            at(Format::Json, &path, Some(Produced::Docx(&resolved)))
        }
        (Format::Docx, Format::Pdf | Format::Png) => {
            if let Some(resolve) = keep_resolver {
                let r = resolve(&bytes)?;
                keep_receipts.extend(settlement_notes(
                    &[],
                    &r.kept_comments,
                    &r.document,
                    &bytes,
                    "settlement",
                ));
                pdf_job(Some(&r.document), to)
            } else {
                match markdown.track_changes {
                    TrackChanges::All => pdf_job(converted, to),
                    // The pages of the document with every change accepted or rejected.
                    choice => {
                        let resolve = if choice == TrackChanges::Accept {
                            jubarte::document_comparer::accept_revisions
                        } else {
                            jubarte::document_comparer::reject_revisions
                        };
                        let resolved = resolve(&bytes).map_err(|e| {
                            refusal("convert", &e)
                                .unwrap_or_else(|| format!("convert failed: {e:?}"))
                        })?;
                        pdf_job(Some(&resolved), to)
                    }
                }
            }
        }
        (Format::Docx, Format::Md) if !markdown.commonmark => {
            let source = if let Some(resolve) = keep_resolver {
                let r = resolve(&bytes)?;
                keep_receipts.extend(settlement_notes(
                    &[],
                    &r.kept_comments,
                    &r.document,
                    &bytes,
                    "settlement",
                ));
                r.document
            } else {
                bytes
            };
            // The agent view, as `jubarte read` prints it: the dialect that
            // comes back whole through `convert` to Word.
            let view = jubarte::markdown::read(
                &source,
                &jubarte::markdown::ReadOptions {
                    track_changes: markdown.track_changes.into(),
                    comments: true,
                    dates: false,
                    page_markers: !markdown.no_page_markers,
                    select: None,
                    source: job
                        .file
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned()),
                },
            )
            .map_err(|e| format!("convert failed: {e}"))?;
            if !job.quiet {
                for warning in &view.warnings {
                    eprintln!("warning: {warning}");
                }
            }
            match job.output {
                Some(path) => at(Format::Md, path, Some(Produced::Text(&view.markdown))),
                None => write_stdout(&view.markdown).map_err(ConvertFailure::from),
            }
        }
        (Format::Docx, Format::Md) => {
            let source = if let Some(resolve) = keep_resolver {
                let r = resolve(&bytes)?;
                keep_receipts.extend(settlement_notes(
                    &[],
                    &r.kept_comments,
                    &r.document,
                    &bytes,
                    "settlement",
                ));
                r.document
            } else {
                bytes
            };
            let read = jubarte::markdown::docx_to_markdown(
                &source,
                &jubarte::markdown::MarkdownOptions {
                    track_changes: markdown.track_changes.into(),
                    extract_media: None,
                    ..jubarte::markdown::MarkdownOptions::default()
                },
            )
            .map_err(|e| format!("convert failed: {e}"))?;
            let text = if markdown.no_page_markers {
                read.markdown
            } else {
                paginated(
                    &source,
                    &read.markdown,
                    markdown.track_changes,
                    job.options.revisions,
                )
            };
            match job.output {
                Some(path) => at(Format::Md, path, Some(Produced::Text(&text))),
                None => write_stdout(&text).map_err(ConvertFailure::from),
            }
        }
        (Format::Docx, Format::Docx) => {
            let resolve: Option<Resolve> = if keep_resolver.is_some() {
                None
            } else {
                match markdown.track_changes {
                    TrackChanges::Accept => {
                        Some(jubarte::document_comparer::accept_revisions as Resolve)
                    }
                    TrackChanges::Reject => {
                        Some(jubarte::document_comparer::reject_revisions as Resolve)
                    }
                    // The `.doc` read as it is, or the fields refreshed.
                    TrackChanges::All if legacy || update_fields => None,
                    TrackChanges::All => {
                        return Err(format!(
                            "{} is already Word: give --track-changes accept or reject, --update-fields, or another --to",
                            job.file.display()
                        )
                        .into());
                    }
                }
            };
            let path = match job.output {
                Some(path) => path.to_path_buf(),
                None if legacy => job.file.with_extension("docx"),
                None => return Err("--output is required to write Word from Word".into()),
            };
            if let Some(report) = job.report {
                let others = [(path.as_path(), "Word output"), (job.file, "input")];
                check_side_file(report, "--report", others, job.force)?;
            }
            ensure_writable(&path, job.force)?;
            let before = (!job.quiet).then(|| confirmation_view(job.file, &bytes));
            let out = if let Some(resolve) = keep_resolver {
                let r = resolve(&bytes)?;
                keep_receipts.extend(settlement_notes(
                    &[],
                    &r.kept_comments,
                    &r.document,
                    &bytes,
                    "settlement",
                ));
                r.document
            } else {
                match resolve {
                    Some(resolve) => resolve(&bytes).map_err(|e| {
                        refusal("convert", &e).unwrap_or_else(|| format!("convert failed: {e:?}"))
                    })?,
                    None => bytes,
                }
            };
            if update_fields {
                let (updated, report_json) = refresh_fields(&out)?;
                let docx = &updated.docx;
                let mut outputs = vec![(path.clone(), docx.as_slice())];
                if let Some(report) = job.report {
                    outputs.push((report.to_path_buf(), report_json.as_bytes()));
                }
                publish_paths(&outputs, job.force).map_err(|e| e.to_string())?;
                if !job.quiet {
                    for field in &updated.fields {
                        outln!(
                            "{}\t{}\t{:?} -> {:?}",
                            field.paragraph,
                            field.kind,
                            field.old,
                            field.new
                        );
                    }
                    eprintln!(
                        "{} field(s) written; {} page(s)",
                        updated.fields.len(),
                        updated.page_count
                    );
                    eprintln!("{}", wrote_line(&path, Format::Docx, docx.len(), None));
                }
                if let Some(before) = before {
                    print_success(job.file, before, &path, docx)?;
                }
                Ok(())
            } else {
                at(Format::Docx, &path, Some(Produced::Docx(&out)))
            }
        }
        (Format::Md, Format::Md) if keep_resolver.is_none() => {
            let text = markdown_text(job.file, bytes)?;
            let out = if markdown.no_critic {
                text
            } else {
                jubarte::markdown::resolve_critic(&text, markdown.track_changes.into())
            };
            match job.output {
                Some(path) => at(Format::Md, path, Some(Produced::Text(&out))),
                None => write_stdout(&out).map_err(ConvertFailure::from),
            }
        }
        (Format::Md, _) => {
            let text = markdown_text(job.file, bytes)?;
            let reference = markdown
                .reference_doc
                .as_deref()
                .map(read_document)
                .transpose()?;
            let refusals = ImageRefusals::default();
            let loader = image_loader(
                &resource_dir(job.file, markdown.resource_path.as_deref()),
                std::rc::Rc::clone(&refusals),
            );
            // The changes and comments that name no author or date take
            // the flag's, else the front matter's, else the reference
            // document's creator and creation time, else Jubarte and now.
            let (creator, created) = reference
                .as_deref()
                .map(jubarte::markdown::docx_stamp)
                .unwrap_or_default();
            let options = jubarte::markdown::DocxOptions {
                reference: reference.as_deref(),
                critic: !markdown.no_critic,
                // --keep-nested-comment: write tracked changes first, then
                // resolve through the keeper so the rescue sees the
                // rejected ranges; without the flag, resolve at write
                // time the way the markdown engine usually does.
                track_changes: if keep_resolver.is_some() {
                    jubarte::markdown::TrackChanges::All
                } else {
                    markdown.track_changes.into()
                },
                author: markdown
                    .author
                    .clone()
                    .or(creator)
                    .unwrap_or_else(|| "Jubarte".to_string()),
                date: markdown
                    .datetime
                    .clone()
                    .or(created)
                    .unwrap_or_else(jubarte::convert::utc_now_iso8601),
                author_fixed: markdown.author.is_some(),
                date_fixed: markdown.datetime.is_some(),
                images: Some(&loader),
                page: markdown.page.into(),
                tracking: markdown
                    .tracking
                    .map(|tracking| tracking == jubarte::cli::Tracking::On),
                scholar: markdown.scholar,
            };
            let written = jubarte::markdown::markdown_to_docx(&text, &options)
                .map_err(|e| format!("convert failed: {e}"))?;
            for warning in &written.warnings {
                eprintln!("warning: {}", image_warning_reason(warning, &refusals));
            }
            // --keep-nested-comment: resolve the freshly built DOCX
            // through the keeper; rerender to text only when the output is
            // .md (a path), so the .md file does not contain DOCX bytes.
            let docx = if let Some(resolve) = keep_resolver {
                let r = resolve(&written.docx)?;
                keep_receipts.extend(settlement_notes(
                    &[],
                    &r.kept_comments,
                    &r.document,
                    &written.docx,
                    "settlement",
                ));
                r.document
            } else {
                written.docx
            };
            // --keep-nested-comment on Markdown output: the .md file holds
            // the agent view of the settled document, not the DOCX package.
            if to == Format::Md {
                let view = jubarte::markdown::read(
                    &docx,
                    &jubarte::markdown::ReadOptions {
                        page_markers: !markdown.no_page_markers,
                        ..Default::default()
                    },
                )
                .map_err(|e| format!("convert failed: {e}"))?;
                match job.output {
                    Some(path) => at(Format::Md, path, Some(Produced::Text(&view.markdown))),
                    None => write_stdout(&view.markdown).map_err(ConvertFailure::from),
                }
            } else if matches!(to, Format::Pdf | Format::Png) {
                pdf_job(Some(&docx), to)
            } else {
                let path = output_at(to);
                at(to, &path, Some(Produced::Docx(&docx)))
            }
        }
        (Format::Pdf | Format::Png | Format::Json, _) => {
            Err(format!("{}: PDF, PNG and JSON are not inputs", job.file.display()).into())
        }
    };
    if result.is_ok() {
        for note in &keep_receipts {
            eprintln!("note: {note}");
        }
    }
    result
}

/// `jubarte debug`'s report options: the checks asked for, else the
/// triage.
fn debug_options(
    checks: Vec<DebugCheck>,
    opts: jubarte::debug::Options,
) -> jubarte::debug::Options {
    if checks.is_empty() {
        opts
    } else {
        jubarte::debug::Options {
            checks: checks.into_iter().map(Into::into).collect(),
            ..opts
        }
    }
}

/// `jubarte debug`: print the report for one package, or two compared.
fn run_debug(files: &[PathBuf], list: bool, opts: &jubarte::debug::Options) -> Result<(), String> {
    let read = |p: &PathBuf| std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()));
    let a = read(&files[0])?;
    let b = files.get(1).map(read).transpose()?;
    let out = if list {
        jubarte::debug::list(&a, b.as_deref(), opts)?
    } else {
        jubarte::debug::report(&a, b.as_deref(), opts)?
    };
    write_stdout(&out)
}

/// One `jubarte validate` run.
struct ValidateJob<'a> {
    file: &'a Path,
    json: bool,
    rules: &'a [String],
    strict: bool,
    repair: Option<&'a Path>,
    /// The document the edits started from (`--baseline`).
    baseline: Option<&'a Path>,
    original: Option<&'a Path>,
    author: Option<&'a str>,
    force: bool,
    quiet: bool,
}

/// `jubarte validate`: `Ok(true)` when the findings fail the run
/// ([`jubarte::validate::fails`]), `Err` when a file could not be read or
/// written or the rules cannot run. With `--repair` the findings are the
/// repaired package's; with `--baseline` only the findings the baseline
/// lacks count ([`jubarte::validate::baseline`]), and the baseline is the
/// edits rule's original when `--original` is not given.
fn run_validate(job: &ValidateJob<'_>) -> Result<bool, ConvertFailure> {
    use jubarte::validate::{
        baselined_json, baselined_text, fails, findings_text, repair, validate_baselined,
        validate_with,
    };
    let read = |p: &Path| std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()));
    let mut docx = read(job.file)?;
    // An original that cannot be read, or is no Word package, stops the run
    // before any repair.
    let before = job.original.map(read_package).transpose()?;
    if let Some(out) = job.repair {
        ensure_writable(out, job.force)?;
        let fixed = repair(&docx).map_err(|e| e.to_string())?;
        publish_file(out, &fixed.docx, job.force)?;
        if !job.json {
            // A structural repair: the edits' history is a separate check.
            let history = match job.original {
                Some(original) => format!("history check against {} follows", original.display()),
                None => "history not checked (--original, --author)".to_string(),
            };
            outln!(
                "repaired {} finding(s), structure only; {history}",
                fixed.repaired.len()
            );
        }
        if !job.quiet {
            eprintln!("{}", wrote_line(out, Format::Docx, fixed.docx.len(), None));
        }
        docx = fixed.docx;
    }
    let rules: Vec<&str> = job.rules.iter().map(String::as_str).collect();
    if let Some(path) = job.baseline {
        // A baseline that is no Word package is refused by its name.
        let base = read_word(path)?;
        let split = validate_baselined(
            &docx,
            &base,
            &rules,
            before.as_deref(),
            job.author,
            Some(&mut jubarte::audit::word_layout),
        )
        .map_err(|e| e.to_string())?;
        if job.json {
            write_stdout(baselined_json(&split).map_err(|e| e.to_string())?)?;
        } else {
            write_stdout(baselined_text(&split))?;
        }
        return Ok(fails(&split.new, job.strict));
    }
    let findings = validate_with(
        &docx,
        &rules,
        before.as_deref().zip(job.author),
        Some(&mut jubarte::audit::word_layout),
    )
    .map_err(|e| e.to_string())?;
    if job.json {
        for f in &findings {
            outln!("{}", serde_json::to_string(f).map_err(|e| e.to_string())?);
        }
    } else {
        write_stdout(findings_text(&findings))?;
    }
    Ok(fails(&findings, job.strict))
}

/// `jubarte debug diff`: each file labelled by its stem (by its folder too
/// when stems repeat, by position when both do).
fn run_debug_diff(
    files: &[PathBuf],
    opts: &jubarte::debug::diff::DiffOptions,
) -> Result<(), String> {
    let stem = |p: &PathBuf| {
        p.file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default()
    };
    let with_dir = |p: &PathBuf| {
        let dir = p
            .parent()
            .and_then(|d| d.file_name())
            .map(|d| d.to_string_lossy().into_owned())
            .unwrap_or_default();
        format!("{dir}/{}", stem(p))
    };
    let unique = |labels: &[String]| {
        labels
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len()
            == labels.len()
    };
    let mut labels: Vec<String> = files.iter().map(stem).collect();
    if !unique(&labels) {
        labels = files.iter().map(with_dir).collect();
    }
    if !unique(&labels) {
        labels = (0..files.len())
            .map(|i| char::from(b'A' + (i % 26) as u8).to_string())
            .collect();
    }
    let bytes = files
        .iter()
        .map(|p| std::fs::read(p).map_err(|e| format!("{}: {e}", p.display())))
        .collect::<Result<Vec<_>, _>>()?;
    let pairs: Vec<(&str, &[u8])> = labels
        .iter()
        .map(String::as_str)
        .zip(bytes.iter().map(Vec::as_slice))
        .collect();
    write_stdout(&jubarte::debug::diff::diff(&pairs, opts)?)
}

/// The stack the CLI runs on: what Linux and macOS give a main thread.
/// Windows gives 1 MiB, which the debug build's command dispatch overflows
/// before it reads an argument.
const STACK_BYTES: usize = 8 * 1024 * 1024;

fn main() -> ExitCode {
    let cli = std::thread::Builder::new()
        .name("main".into())
        .stack_size(STACK_BYTES)
        .spawn(cli_main);
    match cli.map(std::thread::JoinHandle::join) {
        Ok(Ok(code)) => code,
        // The panic was reported on its own thread; leave as a panic does.
        Ok(Err(panic)) => std::panic::resume_unwind(panic),
        // No thread to be had: the stack the system gave is the only one.
        Err(_) => cli_main(),
    }
}

fn cli_main() -> ExitCode {
    let argv: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    let cli = Cli::try_parse_from(
        // The help and usage lines name `jubarte` on every OS, not the
        // binary's own file name (`jubarte.exe` on Windows, #568).
        std::iter::once(std::ffi::OsString::from("jubarte")).chain(std::env::args_os().skip(1)),
    )
    .unwrap_or_else(|error| {
        // The shared parser rejects non-DOCX edit outputs before execution.
        // Preserve that refusal, but identify colliding publication roles
        // with the same stable code as the executor (C9).
        if error.kind() == clap::error::ErrorKind::ArgumentConflict
            && let Ok(raw) = <Cli as clap::Parser>::try_parse_from(std::env::args_os())
            && let Some(Command::Edit {
                options, pdf: true, ..
            }) = raw.command
            && let Some(output) = options.output
            && let Err((code, message)) =
                refuse_output_collisions(&[output.clone(), output.with_extension("pdf")])
        {
            eprintln!("error: {message}");
            std::process::exit(i32::from(code));
        }
        if edit_template::usage(&argv, &error) || edit_typed::usage(&argv, &error) {
            std::process::exit(64);
        }
        // A removed flag names what replaced it; clap's tip may name a
        // lookalike.
        if let Some((old, now)) =
            jubarte::cli::removed_spellings(argv.iter().map(String::as_str)).first()
        {
            let removed = jubarte::cli::removed_error(old, now);
            if jubarte::cli::wants_json(&argv) {
                let message = removed.strip_prefix("error: ").unwrap_or(&removed);
                let _ = write_stdout(jubarte::cli::removed_json_error(
                    &error, &argv, "jubarte", message,
                ));
            }
            eprintln!("{removed}");
            std::process::exit(2);
        }
        // Help, version and usage errors say a name is retired too: the run
        // that fails is the one that needs the new name.
        for note in jubarte::cli::notes(&argv) {
            eprintln!("{note}");
        }
        if jubarte::cli::wants_json(&argv)
            && let Some(line) = jubarte::cli::usage_json_error(&error, &argv, "jubarte")
        {
            let _ = write_stdout(line);
        }
        error.exit()
    });
    for note in jubarte::cli::notes(&argv) {
        eprintln!("{note}");
    }
    JSON_ERRORS.store(
        jubarte::cli::wants_json(&argv),
        std::sync::atomic::Ordering::Relaxed,
    );
    let quiet = cli.quiet;
    match cli.command {
        Some(Command::EditTemplate(jubarte::cli::edit_template::TemplateDispatch {
            action: jubarte::cli::edit_template::TemplateAction::Template(options),
        })) => return edit_template::run(&options, quiet),
        Some(Command::TypedEdit(typed)) => return edit_typed::run(&typed, quiet),
        Some(Command::Check { file, expect, json }) => {
            return check::run(&file, expect.as_deref(), json, quiet);
        }
        Some(Command::Compare(compare)) => {
            return convert_exit_code(run(&resolve_compare(compare, quiet)));
        }
        Some(Command::Changes { file, json, author }) => {
            return exit_code(run_changes(&file, json, &author, quiet));
        }
        Some(Command::Comments {
            file,
            json,
            author,
            latest,
        }) => {
            return exit_code(run_comments(&file, json, author.as_deref(), latest, quiet));
        }
        Some(Command::Accept {
            file,
            output,
            force,
            keep_nested_comment,
            selection,
            render,
        }) => {
            let out = match task_output(Task::Accept, &output, &file, &render, force, quiet) {
                Ok(out) => out,
                Err(e) => return exit_code(Err(e)),
            };
            let apply = if keep_nested_comment {
                jubarte::changes::accept_changes_keeping_comments
            } else {
                jubarte::changes::accept_changes_with_report
            };
            return run_resolution(&file, &out, &selection.filter(), apply, "accept");
        }
        Some(Command::Reject {
            file,
            output,
            force,
            keep_nested_comment,
            selection,
            render,
        }) => {
            let out = match task_output(Task::Reject, &output, &file, &render, force, quiet) {
                Ok(out) => out,
                Err(e) => return exit_code(Err(e)),
            };
            let apply = if keep_nested_comment {
                jubarte::changes::reject_changes_keeping_comments
            } else {
                jubarte::changes::reject_changes_with_report
            };
            return run_resolution(&file, &out, &selection.filter(), apply, "reject");
        }
        Some(Command::Convert {
            file,
            output,
            force,
            pdf,
            png,
            report,
            compress,
            font_report,
            markdown,
            render,
            fail_on_substitution,
            update_fields,
            timeout,
        }) => {
            if let Some(limit) = timeout {
                arm_timeout(limit);
            }
            let options = match pdf_options(&render, compress) {
                Ok(options) => options,
                Err(e) => return exit_code(Err(e)),
            };
            let selected = match render.pages.as_deref().map(parse_pages).transpose() {
                Ok(selected) => selected,
                Err(e) => return exit_code(Err(e)),
            };
            let job = ConvertJob {
                file: &file,
                bytes: None,
                output: output.as_deref(),
                force,
                font_report: font_report.as_deref(),
                options,
                // The parser refused --to with a retired flag naming another.
                to: markdown.to.or_else(|| {
                    pdf.then_some(Format::Pdf)
                        .or_else(|| png.then_some(Format::Png))
                }),
                dpi: render.dpi,
                report: report.as_deref(),
                pages: selected.as_deref(),
                fail_on_substitution,
                quiet,
            };
            return convert_exit_code(run_convert_any(&job, &markdown, update_fields));
        }
        Some(Command::Diff {
            old,
            new,
            output,
            from,
            format,
            columns,
            context,
            accept_changes,
            full_lines,
            force,
            author,
            datetime: date,
            ..
        }) => {
            let job = DiffJob {
                old: &old,
                new: &new,
                output: output.as_deref(),
                from,
                force,
                settings: comparer_settings(
                    &author,
                    &date.unwrap_or_else(jubarte::convert::utc_now_iso8601),
                    None,
                    false,
                    false,
                ),
                patch: (format == PatchFormat::Patch).then_some(columns),
                quiet,
            };
            if format.is_text_view() {
                use jubarte::text_diff::{TextFormat, TextOptions, UnifiedOptions};
                let style = match format {
                    PatchFormat::Github => TextFormat::Github,
                    PatchFormat::Word => TextFormat::Word,
                    PatchFormat::Normal => TextFormat::Normal,
                    PatchFormat::Context => TextFormat::Context,
                    PatchFormat::SideBySide => TextFormat::SideBySide,
                    PatchFormat::Patch | PatchFormat::Critic => unreachable!("text view"),
                };
                let options = TextOptions {
                    unified: UnifiedOptions {
                        old_name: old.display().to_string(),
                        new_name: new.display().to_string(),
                        context,
                    },
                    format: style,
                    accept_changes,
                    window: (!full_lines).then_some(70),
                };
                return convert_exit_code(run_text_diff(&job, &options));
            }
            return convert_exit_code(run_diff(&job));
        }
        Some(Command::Read {
            file,
            args,
            output,
            force,
        }) => {
            return convert_exit_code(run_text(&file, &args, output.as_deref(), force, quiet));
        }
        Some(Command::Edit {
            file,
            plan,
            plan_default,
            consume,
            options,
            dry_run,
            skip_failed,
            require_change,
            pdf,
            png,
            dpi,
            revisions,
            revision_palette,
            rewrite_others_comments,
            keep_others,
            operations,
            ..
        }) => {
            let style = match revision_style(revisions, revision_palette.as_deref()) {
                Ok(style) => style,
                Err(e) => return exit_code(Err(e)),
            };
            let flags = EditFlags {
                verb: jubarte::edit::flags::Verb::Edit,
                file: &file,
                plan: plan.as_deref(),
                plan_default,
                consume,
                operations: &operations,
                options: &options,
                dry_run,
                skip_failed,
                require_change,
                pdf,
                png,
                dpi,
                revisions: style,
                rewrite_others_comments,
                keep_others,
                quiet,
            };
            return edit_exit(run_flags(&flags));
        }
        Some(Command::Add {
            file,
            plan,
            plan_default,
            consume,
            options,
            keep_others,
            operations,
            ..
        }) => {
            let flags = EditFlags {
                verb: jubarte::edit::flags::Verb::Add,
                file: &file,
                plan: plan.as_deref(),
                plan_default,
                consume,
                operations: &operations,
                options: &options,
                dry_run: false,
                skip_failed: false,
                require_change: false,
                pdf: false,
                png: false,
                dpi: 96.0,
                revisions: jubarte::convert::RevisionStyle::Conventional,
                rewrite_others_comments: false,
                keep_others,
                quiet,
            };
            return edit_exit(run_flags(&flags));
        }
        Some(Command::Capabilities {
            json,
            operation,
            field,
            matrix,
        }) => {
            if matrix || operation.is_some() || field.is_some() {
                let answer =
                    jubarte::capabilities::discovery("cli", operation.as_deref(), field.as_deref());
                let printed = if json {
                    write_stdout(format!(
                        "{}\n",
                        serde_json::to_string_pretty(&answer).expect("discovery serializes")
                    ))
                } else {
                    write_stdout(jubarte::capabilities::discovery_text(&answer))
                };
                return exit_code(printed);
            }
            let manifest = jubarte::capabilities::capabilities("cli");
            let printed = if json {
                outln!(
                    "{}",
                    serde_json::to_string_pretty(&manifest).expect("manifest serializes")
                );
                Ok(())
            } else {
                write_stdout(jubarte::capabilities::capabilities_text(&manifest))
            };
            return exit_code(printed);
        }
        Some(Command::SelfUpdate {
            check,
            yes,
            version,
        }) => return exit_code(run_self_update(check, yes, version)),
        Some(Command::Debug {
            sub:
                Some(DebugCommand::Diff {
                    files,
                    part,
                    style,
                    para_text,
                    raw,
                    full,
                    limit,
                }),
            ..
        }) => {
            let opts = jubarte::debug::diff::DiffOptions {
                part,
                style,
                para_text,
                raw,
                full,
                limit,
            };
            return exit_code(run_debug_diff(&files, &opts));
        }
        Some(Command::VisualDiff {
            a,
            b,
            dpi,
            out_dir,
            json,
            no_overlay,
            force,
        }) => {
            return match run_diff_render(&DiffRenderJob {
                a: &a,
                b: &b,
                out_dir: out_dir.as_deref(),
                dpi,
                json,
                overlay: !no_overlay,
                force,
            }) {
                Ok(false) => ExitCode::SUCCESS,
                // Like `edit`'s 3: a CI step can gate on a visual change.
                Ok(true) => ExitCode::from(5),
                Err(e) => convert_exit_code(Err(e)),
            };
        }
        Some(Command::Debug {
            sub: None,
            files,
            list,
            checks,
            part,
            grep,
            limit,
            context,
            full_lines,
        }) => {
            let opts = debug_options(
                checks,
                jubarte::debug::Options {
                    part,
                    grep,
                    limit,
                    context,
                    full_lines,
                    ..Default::default()
                },
            );
            return exit_code(run_debug(&files, list, &opts));
        }
        Some(Command::Append {
            files,
            output,
            section_break,
            keep_sections,
            carry_comments,
            force,
            render,
        }) => {
            let options = jubarte::append::AppendOptions {
                section_break: section_break.into(),
                keep_sections,
                comments: if carry_comments {
                    jubarte::append::AppendComments::Carry
                } else {
                    jubarte::append::AppendComments::Drop
                },
            };
            let first = files.first().map_or(Path::new(""), PathBuf::as_path);
            let out = match task_output(Task::Append, &output, first, &render, force, quiet) {
                Ok(out) => out,
                Err(e) => return exit_code(Err(e)),
            };
            return convert_exit_code(run_append(&files, &out, &options));
        }
        Some(Command::Validate {
            file,
            json,
            rules,
            strict,
            repair,
            baseline,
            original,
            author,
            force,
        }) => {
            let job = ValidateJob {
                file: &file,
                json,
                rules: &rules,
                strict,
                repair: repair.as_deref(),
                baseline: baseline.as_deref(),
                original: original.as_deref(),
                author: author.as_deref(),
                force,
                quiet,
            };
            return match run_validate(&job) {
                Ok(true) => ExitCode::from(2),
                result => convert_exit_code(result.map(|_| ())),
            };
        }
        Some(Command::Scrub {
            file,
            output,
            force,
            scrub,
            render,
        }) => {
            let out = match task_output(Task::Scrub, &output, &file, &render, force, quiet) {
                Ok(out) => out,
                Err(e) => return exit_code(Err(e)),
            };
            return convert_exit_code(run_scrub(&file, &out, &scrub.options()));
        }
        None => {}
    }
    if cli.compare.modified.is_none() && cli.compare.modified_pos.is_none() {
        let file = cli
            .compare
            .original
            .or(cli.compare.original_pos)
            .expect("clap requires ORIGINAL");
        let output = cli.compare.output.as_deref();
        return convert_exit_code(run_text(
            &file,
            &cli.compare.read,
            output,
            cli.compare.force,
            quiet,
        ));
    }
    convert_exit_code(run(&resolve_compare(cli.compare, quiet)))
}

/// `jubarte append`: fold the documents left, `append(append(A, B), C)`.
fn run_append(
    files: &[PathBuf],
    out: &Output<'_>,
    options: &jubarte::append::AppendOptions,
) -> Result<(), ConvertFailure> {
    check_output(out)?;
    let mut paths = files.iter();
    let first = paths.next().ok_or("append needs two documents")?;
    let mut joined = read_word(first)?;
    for path in paths {
        let next = read_word(path)?;
        let appended = jubarte::append::append_documents(&joined, &next, options)
            .map_err(|e| format!("appending {}: {e}", path.display()))?;
        for warning in &appended.warnings {
            eprintln!("warning: {}: {warning}", path.display());
        }
        joined = appended.docx;
    }
    write_output(out, Produced::Docx(&joined))
}

#[cfg(feature = "self-update")]
fn run_self_update(check: bool, yes: bool, version: Option<String>) -> Result<(), String> {
    jubarte::update::run(&jubarte::update::Options {
        check,
        yes,
        version,
    })
}

#[cfg(not(feature = "self-update"))]
fn run_self_update(_check: bool, _yes: bool, _version: Option<String>) -> Result<(), String> {
    Err("this jubarte was built without the self-update feature; update it the way it was installed".into())
}

/// `jubarte fields update`: refresh, write, then list what was written.
/// `accept_revisions` / `reject_revisions`: a package in, the resolved one out.
type Resolve = fn(&[u8]) -> Result<Vec<u8>, jubarte::opc::OpcError>;

/// Prepare the refreshed package and its JSON report together, without file I/O.
/// `convert --update-fields` publishes both only after staging succeeds.
fn refresh_fields(bytes: &[u8]) -> Result<(jubarte::fields::Updated, String), String> {
    let updated = jubarte::fields::update_fields(bytes).map_err(|e| e.to_string())?;
    let json = serde_json::json!({
        "page_count": updated.page_count,
        "fields": updated.fields,
    });
    Ok((updated, json.to_string()))
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;
    use jubarte::convert::{MarkLines, RevisionStyle};

    /// A bundle directory `--force` cannot read is no earlier bundle it may
    /// replace: an unreadable one blocks (#1431 review), a missing one is
    /// nothing to replace.
    #[cfg(unix)]
    #[test]
    fn an_unreadable_bundle_is_foreign() {
        use std::os::unix::fs::PermissionsExt;
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("bundle");
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("notes.txt"), "mine").unwrap();
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o000)).unwrap();
        let readable = std::fs::read_dir(&dir).is_ok();
        let foreign = foreign_bundle_entries(&dir);
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();
        if readable {
            return; // root reads any directory
        }
        assert_eq!(foreign.len(), 1, "{foreign:?}");
        assert!(foreign[0].starts_with("(unreadable: "), "{foreign:?}");
        assert!(foreign_bundle_entries(&root.path().join("missing")).is_empty());
    }

    #[test]
    fn publication_cleanup_warnings_reach_stderr_for_both_outcomes() {
        use jubarte::publication::{PublishError, PublishReport};
        let warnings = vec!["cleaning staging: injected cleanup".to_string()];
        for result in [
            Ok(PublishReport {
                warnings: warnings.clone(),
            }),
            Err(PublishError {
                committed: vec!["output".into()],
                durability_uncertain: true,
                source: std::io::Error::other("injected SyncParent"),
                warnings: warnings.clone(),
            }),
        ] {
            let failed = result.is_err();
            let mut stderr = Vec::new();
            let reported = report_publication(result, &mut stderr);
            assert_eq!(reported.is_err(), failed);
            assert_eq!(
                String::from_utf8(stderr).unwrap(),
                "warning: cleaning staging: injected cleanup\n"
            );
        }
    }

    #[test]
    fn comment_changes_note_agrees_with_its_count() {
        assert_eq!(
            comment_changes_note(1),
            "note: 1 tracked change inside comments was left; accept or reject all to settle it"
        );
        assert_eq!(
            comment_changes_note(2),
            "note: 2 tracked changes inside comments were left; accept or reject all to settle them"
        );
    }

    fn bundle_restore_failure(recheck_fails: bool, restore_fails: bool) {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().join("bundle");
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("original"), b"old bundle").unwrap();
        let stage = stage_bundle_dir(&dir).unwrap();
        std::fs::write(stage.join("replacement"), b"new bundle").unwrap();
        let aside = staging_path(&dir, 100);
        let restore_error = std::cell::RefCell::new(String::new());
        let original_error = std::cell::RefCell::new(String::new());
        let error = publish_dir(&stage, &dir, true, || {
            if !aside.exists() {
                return Ok(());
            }
            if restore_fails {
                // Obstruct with a non-empty directory: every platform refuses
                // to rename a directory onto one. Unix answers ENOTEMPTY;
                // Windows answers ERROR_DIR_NOT_EMPTY (std's rename falls
                // back to POSIX-semantics FileRenameInfoEx and maps that
                // refusal). A plain file cannot obstruct: Windows
                // MOVEFILE_REPLACE_EXISTING replaces it, so the restore
                // would succeed and never reach the path under test.
                std::fs::create_dir(&dir).unwrap();
                std::fs::write(dir.join("obstruction"), b"blocked").unwrap();
                *restore_error.borrow_mut() =
                    std::fs::rename(&aside, &dir).unwrap_err().to_string();
            }
            if recheck_fails {
                let message = "SOURCE_CHANGED: deterministic recheck failure".to_string();
                *original_error.borrow_mut() = message.clone();
                return Err((EXIT_USAGE, message));
            }
            std::fs::remove_dir_all(&stage).unwrap();
            *original_error.borrow_mut() = format!(
                "publishing {}: {}",
                dir.display(),
                std::fs::rename(&stage, &dir).unwrap_err()
            );
            Ok(())
        })
        .unwrap_err();
        assert_eq!(error.0, if recheck_fails { EXIT_USAGE } else { 1 });
        assert!(!stage.exists(), "staging was not cleaned up");
        if restore_fails {
            assert_eq!(
                std::fs::read(aside.join("original")).unwrap(),
                b"old bundle"
            );
            assert_eq!(std::fs::read(dir.join("obstruction")).unwrap(), b"blocked");
            assert!(error.1.contains(&*original_error.borrow()), "{}", error.1);
            assert!(
                error.1.contains(&aside.display().to_string()),
                "{}",
                error.1
            );
            assert!(error.1.contains("restoring"), "{}", error.1);
            assert!(error.1.contains(&*restore_error.borrow()), "{}", error.1);
        } else {
            assert_eq!(std::fs::read(dir.join("original")).unwrap(), b"old bundle");
            assert!(!aside.exists());
            assert_eq!(error.1, *original_error.borrow());
        }
    }

    #[test]
    fn audit_d13_bundle_recheck_restore_failure_names_original() {
        bundle_restore_failure(true, true);
    }

    #[test]
    fn audit_d13_bundle_install_restore_failure_names_original() {
        bundle_restore_failure(false, true);
    }

    #[test]
    fn audit_d13_bundle_recheck_restore_success_keeps_message() {
        bundle_restore_failure(true, false);
    }

    #[test]
    fn audit_d13_bundle_install_restore_success_keeps_message() {
        bundle_restore_failure(false, false);
    }

    /// GHSA-m665-v7pm-7grj: `jubarte debug` opens a package every other
    /// route refuses — a local header naming an entry differently than the
    /// central directory — because triaging refused input is its purpose.
    #[test]
    fn debug_opens_a_dual_header_package_instead_of_refusing() {
        use std::io::Write as _;
        let dir = tempfile::tempdir().unwrap();
        let mut package = Vec::new();
        {
            let mut writer = zip::ZipWriter::new(std::io::Cursor::new(&mut package));
            let options = zip::write::SimpleFileOptions::default();
            for (name, body) in [
                (
                    "[Content_Types].xml",
                    "<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"/>",
                ),
                (
                    "_rels/.rels",
                    "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"/>",
                ),
                (
                    "word/document.xml",
                    "<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:body/></w:document>",
                ),
            ] {
                writer.start_file(name, options).unwrap();
                writer.write_all(body.as_bytes()).unwrap();
            }
            writer.finish().unwrap();
        }
        let at = {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&package)).unwrap();
            archive.by_name("word/document.xml").unwrap().header_start() as usize
        };
        assert_eq!(&package[at..at + 4], b"PK\x03\x04");
        let range = at + 30..at + 30 + b"word/document.xml".len();
        package[range].copy_from_slice(b"w0rd/document.xml");
        let file = dir.path().join("cloaked.docx");
        std::fs::write(&file, &package).unwrap();
        run_debug(&[file], false, &jubarte::debug::Options::default())
            .expect("debug triages a dual-header package");
    }

    #[test]
    fn append_parses_files_output_and_break() {
        let cli = Cli::try_parse_from(["jubarte", "append", "a.docx", "b.docx", "-o", "out.docx"])
            .unwrap();
        let Some(Command::Append {
            files,
            output,
            section_break,
            keep_sections,
            carry_comments,
            force,
            render,
        }) = cli.command
        else {
            panic!("not append");
        };
        assert!(render.pages.is_none() && !render.changed_pages);
        assert_eq!(files.len(), 2);
        assert_eq!(output, PathBuf::from("out.docx"));
        assert_eq!(section_break, SectionBreakArg::NextPage);
        assert!(!keep_sections && !carry_comments && !force && !cli.quiet);
        let quiet = Cli::try_parse_from([
            "jubarte", "append", "a.docx", "b.docx", "-o", "o.docx", "-q",
        ])
        .unwrap();
        assert!(quiet.quiet, "-q is the global flag after the task");
        let cli = Cli::try_parse_from([
            "jubarte",
            "append",
            "a.docx",
            "b.docx",
            "c.docx",
            "-o",
            "o.docx",
            "--section-break",
            "none",
            "--keep-sections",
            "--carry-comments",
        ])
        .unwrap();
        let Some(Command::Append {
            files,
            section_break,
            keep_sections,
            carry_comments,
            ..
        }) = cli.command
        else {
            panic!("not append");
        };
        assert_eq!(files.len(), 3);
        assert_eq!(
            jubarte::append::SectionBreak::from(section_break),
            jubarte::append::SectionBreak::None
        );
        assert!(keep_sections && carry_comments);
        assert!(Cli::try_parse_from(["jubarte", "append", "a.docx", "-o", "o.docx"]).is_err());
        assert!(Cli::try_parse_from(["jubarte", "append", "a.docx", "b.docx"]).is_err());
    }

    #[test]
    fn self_update_parses_check_yes_and_a_pinned_version() {
        let cli = Cli::try_parse_from(["jubarte", "self-update", "--check"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::SelfUpdate {
                check: true,
                yes: false,
                version: None
            })
        ));
        let cli =
            Cli::try_parse_from(["jubarte", "self-update", "-y", "--version", "0.9.3"]).unwrap();
        let Some(Command::SelfUpdate {
            check,
            yes,
            version,
        }) = cli.command
        else {
            panic!("expected self-update");
        };
        assert!(!check && yes);
        assert_eq!(version.as_deref(), Some("0.9.3"));
    }

    #[test]
    fn convert_revisions_default_to_conventional_and_validate_the_palette() {
        let cli =
            Cli::try_parse_from(["jubarte", "convert", "in.docx", "--revisions", "word"]).unwrap();
        let Some(Command::Convert { render, .. }) = cli.command else {
            panic!("expected convert");
        };
        assert_eq!(render.revisions, Revisions::Word);
        assert_eq!(
            revision_style(Revisions::Conventional, None),
            Ok(RevisionStyle::Conventional)
        );
        assert!(revision_style(Revisions::Custom, None).is_err());
        assert!(revision_style(Revisions::Word, Some("deleted=#000000")).is_err());
        let Ok(RevisionStyle::Custom(p)) = revision_style(
            Revisions::Custom,
            Some("deleted=#112233:double-strike,moved-to=#00FF00:underline"),
        ) else {
            panic!("custom palette parses");
        };
        assert_eq!(p.deleted.color, [0x11, 0x22, 0x33]);
        assert_eq!(p.deleted.strike, MarkLines::Double);
        assert_eq!(p.moved_to.underline, MarkLines::Single);
        assert!(revision_style(Revisions::Custom, Some("deleted=red")).is_err());
    }
    use clap::CommandFactory;

    fn job_of(args: &[&str]) -> Job {
        let cli = Cli::try_parse_from(args).expect("parse");
        match cli.command {
            Some(Command::Compare(compare)) => resolve_compare(compare, cli.quiet),
            None => resolve_compare(cli.compare, cli.quiet),
            other => panic!("expected compare, got {other:?}"),
        }
    }

    /// clap's own invariants (catches derive-config mistakes like duplicate shorts).
    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn positional_args_and_default_output() {
        let j = job_of(&["jubarte", "a.docx", "b.docx"]);
        assert_eq!(j.original, PathBuf::from("a.docx"));
        assert_eq!(j.modified, PathBuf::from("b.docx"));
        assert_eq!(j.output, None, "the shorthand prints the view without -o");
        let before = jubarte::convert::utc_now_iso8601();
        let j = job_of(&["jubarte", "compare", "a.docx", "b.docx"]);
        let after = jubarte::convert::utc_now_iso8601();
        assert_eq!(j.output, None, "so does compare");
        assert_eq!(j.author, "Jubarte");
        // Without -d the revisions are dated now (D17); the fixed-width
        // UTC form orders as text.
        assert!(
            before <= j.date && j.date <= after,
            "{before} {} {after}",
            j.date
        );
        assert!(!j.force && !j.quiet);
    }

    #[test]
    fn default_output_uses_original_directory_and_stems() {
        let o = default_output(
            Path::new("docs/contract.docx"),
            Path::new("rev/contract-2.docx"),
        );
        assert_eq!(o, PathBuf::from("docs/contract_v_contract-2.docx"));
        // no directory → bare name
        let o2 = default_output(Path::new("contract.docx"), Path::new("contract-2.docx"));
        assert_eq!(o2, PathBuf::from("contract_v_contract-2.docx"));
    }

    #[test]
    fn named_flags_override_positionals() {
        let j = job_of(&[
            "jubarte",
            "a.docx",
            "b.docx",
            "-b",
            "real-orig.docx",
            "--modified",
            "real-mod.docx",
        ]);
        assert_eq!(j.original, PathBuf::from("real-orig.docx"));
        assert_eq!(j.modified, PathBuf::from("real-mod.docx"));
    }

    #[test]
    fn all_options_long_and_short() {
        let j = job_of(&[
            "jubarte",
            "-b",
            "o.docx",
            "-m",
            "n.docx",
            "-o",
            "out.docx",
            "-a",
            "Jane Doe",
            "-d",
            "2024-01-02T00:00:00Z",
            "--force",
            "--quiet",
        ]);
        assert_eq!(j.output, Some(PathBuf::from("out.docx")));
        assert_eq!(j.author, "Jane Doe");
        assert_eq!(j.date, "2024-01-02T00:00:00Z");
        assert!(j.force && j.quiet);
    }

    #[test]
    fn compare_mode_defaults_to_word_and_powertools_has_two_spellings() {
        assert!(!job_of(&["jubarte", "a.docx", "b.docx"]).powertools_faithful);
        assert!(!job_of(&["jubarte", "a.docx", "b.docx", "--mode", "word"]).powertools_faithful);
        assert!(
            job_of(&["jubarte", "a.docx", "b.docx", "--mode", "powertools"]).powertools_faithful
        );
        assert!(
            job_of(&["jubarte", "a.docx", "b.docx", "--powertools-faithful"]).powertools_faithful
        );
        assert!(
            Cli::try_parse_from(["jubarte", "a.docx", "b.docx", "--mode", "libreoffice"]).is_err()
        );
    }

    #[test]
    fn flags_can_supply_both_inputs_without_positionals() {
        let j = job_of(&["jubarte", "--original", "x.docx", "--modified", "y.docx"]);
        assert_eq!(j.original, PathBuf::from("x.docx"));
        assert_eq!(j.modified, PathBuf::from("y.docx"));
        assert_eq!(j.output, None);
        let j = job_of(&[
            "jubarte",
            "compare",
            "--original",
            "x.docx",
            "--modified",
            "y.docx",
        ]);
        assert_eq!(j.output, None);
    }

    #[test]
    fn double_dash_treats_rest_as_positionals() {
        let j = job_of(&["jubarte", "--", "-weird-name.docx", "b.docx"]);
        assert_eq!(j.original, PathBuf::from("-weird-name.docx"));
        assert_eq!(j.modified, PathBuf::from("b.docx"));
    }

    #[test]
    fn help_and_version_are_handled_by_clap() {
        use clap::error::ErrorKind;
        let help = Cli::try_parse_from(["jubarte", "--help"]).unwrap_err();
        assert_eq!(help.kind(), ErrorKind::DisplayHelp);
        let ver = Cli::try_parse_from(["jubarte", "-V"]).unwrap_err();
        assert_eq!(ver.kind(), ErrorKind::DisplayVersion);
    }

    #[test]
    fn missing_inputs_are_clap_usage_errors() {
        // `jubarte one.docx` is the agent view; `compare` still needs two.
        let read = Cli::try_parse_from(["jubarte", "one.docx"]).unwrap();
        assert!(read.command.is_none() && read.compare.modified_pos.is_none());
        let only_one = Cli::try_parse_from(["jubarte", "compare", "one.docx"]).unwrap_err();
        assert_eq!(
            only_one.kind(),
            clap::error::ErrorKind::MissingRequiredArgument
        );
        assert!(Cli::try_parse_from(["jubarte"]).is_err());
    }

    #[test]
    fn extra_positional_and_unknown_flag_rejected_by_clap() {
        use clap::error::ErrorKind;
        let extra = Cli::try_parse_from(["jubarte", "a.docx", "b.docx", "c.docx"]).unwrap_err();
        assert!(matches!(
            extra.kind(),
            ErrorKind::UnknownArgument | ErrorKind::ArgumentConflict
        ));
        let bogus = Cli::try_parse_from(["jubarte", "--bogus"]).unwrap_err();
        assert_eq!(bogus.kind(), ErrorKind::UnknownArgument);
        let missing_val = Cli::try_parse_from(["jubarte", "--author"]).unwrap_err();
        assert_eq!(missing_val.kind(), ErrorKind::InvalidValue);
    }

    /// `revisions` (Docxodus's GetRevisions listing) left the CLI: `changes`
    /// is the one listing. The word is a file name again, the legacy
    /// compare's ORIGINAL.
    #[test]
    fn revisions_is_no_subcommand() {
        let cli = Cli::try_parse_from(["jubarte", "revisions", "b.docx"]).unwrap();
        assert!(cli.command.is_none());
        let job = resolve_compare(cli.compare, cli.quiet);
        assert_eq!(job.original, PathBuf::from("revisions"));
    }

    /// Prior behavior path: a two-positional invocation whose filenames do
    /// NOT collide with the subcommand name is unaffected by adding
    /// `command` to `Cli` — `cli.command` stays `None` and the native resolver
    /// merges the positionals exactly as before this PR.
    #[test]
    fn plain_compare_positionals_leave_command_none() {
        let cli = Cli::try_parse_from(["jubarte", "a.docx", "b.docx"]).unwrap();
        assert!(cli.command.is_none());
        let job = resolve_compare(cli.compare, cli.quiet);
        assert_eq!(job.original, PathBuf::from("a.docx"));
        assert_eq!(job.modified, PathBuf::from("b.docx"));
    }

    /// `accept <file> -o <out> --force` parses into `Command::Accept` with the
    /// output and force flag captured.
    #[test]
    fn accept_subcommand_parses_file_output_and_force() {
        let cli =
            Cli::try_parse_from(["jubarte", "accept", "rl.docx", "-o", "out.docx", "--force"])
                .unwrap();
        match cli.command {
            Some(Command::Accept {
                file,
                output,
                force,
                selection,
                render,
                ..
            }) => {
                assert!(render.pages.is_none() && !render.move_comments);
                assert_eq!(
                    selection.filter(),
                    jubarte::changes::ChangeFilter::default()
                );
                assert_eq!(file, PathBuf::from("rl.docx"));
                assert_eq!(output, PathBuf::from("out.docx"));
                assert!(force);
            }
            other => panic!("expected accept subcommand, got {other:?}"),
        }
    }

    /// `scrub` without a selection removes all four kinds, under the alias
    /// `Author`; any flag narrows it to the ones given.
    #[test]
    fn scrub_flags_select_what_goes_and_none_selects_everything() {
        let options = |args: &[&str]| {
            let cli = Cli::try_parse_from(
                ["jubarte", "scrub", "in.docx", "-o", "out.docx"]
                    .iter()
                    .chain(args),
            )
            .unwrap();
            match cli.command {
                Some(Command::Scrub { scrub, .. }) => scrub.options(),
                other => panic!("expected scrub subcommand, got {other:?}"),
            }
        };
        assert_eq!(options(&[]), jubarte::scrub::ScrubOptions::default());
        assert_eq!(
            options(&["--rsids", "--author-alias", "Counsel"]),
            jubarte::scrub::ScrubOptions {
                author_alias: Some("Counsel".into()),
                rsids: true,
                docprops: false,
                comments: false,
            }
        );
        assert_eq!(
            options(&["--comments"]),
            jubarte::scrub::ScrubOptions {
                author_alias: None,
                rsids: false,
                docprops: false,
                comments: true,
            }
        );
    }

    /// `reject <file> -o <out>` parses into `Command::Reject`; `force` defaults
    /// to false (the same no-clobber contract as `accept` / compare).
    #[test]
    fn reject_subcommand_parses_file_output_and_defaults_force_false() {
        let cli = Cli::try_parse_from(["jubarte", "reject", "rl.docx", "-o", "out.docx"]).unwrap();
        match cli.command {
            Some(Command::Reject {
                file,
                output,
                force,
                selection,
                render,
                ..
            }) => {
                assert!(render.pages.is_none() && !render.move_comments);
                assert_eq!(
                    selection.filter(),
                    jubarte::changes::ChangeFilter::default()
                );
                assert_eq!(file, PathBuf::from("rl.docx"));
                assert_eq!(output, PathBuf::from("out.docx"));
                assert!(!force);
            }
            other => panic!("expected reject subcommand, got {other:?}"),
        }
    }

    /// `--id`, `--author` and `--kind` repeat and select the changes to
    /// resolve; a flag left out constrains nothing.
    #[test]
    fn accept_selection_flags_build_the_change_filter() {
        use jubarte::changes::{ChangeFilter, ChangeKind};
        let cli = Cli::try_parse_from([
            "jubarte",
            "accept",
            "rl.docx",
            "-o",
            "out.docx",
            "--id",
            "r1",
            "--id",
            "header1.r2",
            "--kind",
            "move",
        ])
        .unwrap();
        let Some(Command::Accept { selection, .. }) = cli.command else {
            panic!("expected accept subcommand");
        };
        assert_eq!(
            selection.filter(),
            ChangeFilter {
                ids: Some(vec!["r1".into(), "header1.r2".into()]),
                authors: None,
                kinds: Some(vec![ChangeKind::Move]),
            }
        );
    }

    /// `reject` requires `-o/--output` (clap usage error when omitted), matching
    /// `accept`.
    #[test]
    fn reject_subcommand_requires_output() {
        use clap::error::ErrorKind;
        let err = Cli::try_parse_from(["jubarte", "reject", "rl.docx"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn convert_takes_fail_on_substitution_and_defaults_it_off() {
        let flag = |args: &[&str]| match Cli::try_parse_from(args).unwrap().command {
            Some(Command::Convert {
                fail_on_substitution,
                ..
            }) => fail_on_substitution,
            other => panic!("expected convert, got {other:?}"),
        };
        assert!(flag(&[
            "jubarte",
            "convert",
            "in.docx",
            "--fail-on-substitution"
        ]));
        assert!(!flag(&["jubarte", "convert", "in.docx"]));
    }

    #[test]
    fn convert_subcommand_parses_font_report() {
        let cli = Cli::try_parse_from([
            "jubarte",
            "convert",
            "in.docx",
            "--font-report",
            "out.json",
            "-o",
            "out.pdf",
        ])
        .unwrap();
        match cli.command {
            Some(Command::Convert {
                file,
                output,
                font_report,
                compress,
                force,
                render,
                pdf,
                png,
                report,
                ..
            }) => {
                assert_eq!(render, RenderArgs::default());
                assert_eq!(file, PathBuf::from("in.docx"));
                assert_eq!(output.as_deref(), Some(Path::new("out.pdf")));
                assert_eq!(font_report.as_deref(), Some(Path::new("out.json")));
                assert!(!compress && !force);
                assert!(!pdf && !png && report.is_none());
            }
            other => panic!("expected convert subcommand, got {other:?}"),
        }
    }

    fn tiny_docx_bytes(family: &str) -> Vec<u8> {
        body_docx_bytes(&format!(
            r#"<w:p><w:r><w:rPr><w:rFonts w:ascii="{family}" w:hAnsi="{family}"/></w:rPr><w:t>HELLO</w:t></w:r></w:p>"#
        ))
    }

    /// A Letter-size package whose body is `body` (WordprocessingML).
    fn body_docx_bytes(body: &str) -> Vec<u8> {
        use std::io::{Cursor, Write};
        let mut buf = Vec::new();
        {
            let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
            let opt = zip::write::SimpleFileOptions::default();
            z.start_file("[Content_Types].xml", opt).unwrap();
            z.write_all(
                br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#,
            )
            .unwrap();
            z.start_file("_rels/.rels", opt).unwrap();
            z.write_all(
                br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rIdM" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#,
            )
            .unwrap();
            z.start_file("word/document.xml", opt).unwrap();
            let doc = format!(
                r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body}<w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/></w:sectPr></w:body></w:document>"#
            );
            z.write_all(doc.as_bytes()).unwrap();
            z.finish().unwrap();
        }
        buf
    }

    #[test]
    fn convert_rejects_a_font_report_path_equal_to_the_pdf() {
        let dir = tempfile::tempdir().expect("tempdir");
        let docx = dir.path().join("in.docx");
        let pdf = dir.path().join("out.pdf");
        std::fs::write(&docx, tiny_docx_bytes("Calibri")).expect("docx");
        let same = dir.path().join(".").join("out.pdf");
        let err = run_convert(&ConvertJob {
            file: &docx,
            bytes: None,
            output: Some(&pdf),
            force: false,
            font_report: Some(&same),
            options: jubarte::convert::PdfOptions {
                revisions: RevisionStyle::Word,
                ..jubarte::convert::PdfOptions::default()
            },
            to: None,
            dpi: 96.0,
            report: None,
            pages: None,
            fail_on_substitution: false,
            quiet: false,
        })
        .expect_err("report over the PDF must be refused");
        assert!(
            err.message.contains("same file as the PDF output"),
            "{}",
            err.message
        );
        // A PNG run names its own output.
        let err = run_convert(&ConvertJob {
            file: &docx,
            bytes: None,
            output: Some(&pdf),
            force: false,
            font_report: Some(&same),
            options: jubarte::convert::PdfOptions {
                revisions: RevisionStyle::Word,
                ..jubarte::convert::PdfOptions::default()
            },
            to: Some(Format::Png),
            dpi: 96.0,
            report: None,
            pages: None,
            fail_on_substitution: false,
            quiet: false,
        })
        .expect_err("report over the PNG output must be refused");
        assert!(
            err.message.contains("same file as the PNG output"),
            "{}",
            err.message
        );
        assert!(!pdf.exists(), "nothing is written when the paths collide");
        let err = run_convert(&ConvertJob {
            file: &docx,
            bytes: None,
            output: Some(&pdf),
            force: false,
            font_report: Some(&docx),
            options: jubarte::convert::PdfOptions {
                revisions: RevisionStyle::Word,
                ..jubarte::convert::PdfOptions::default()
            },
            to: None,
            dpi: 96.0,
            report: None,
            pages: None,
            fail_on_substitution: false,
            quiet: false,
        })
        .expect_err("report over the input must be refused");
        assert!(
            err.message.contains("same file as the input"),
            "{}",
            err.message
        );
        assert!(std::fs::read(&docx).expect("docx").starts_with(b"PK"));
    }

    #[test]
    fn a_legacy_doc_is_refused_with_a_save_as_hint() {
        // `.doc` (Word 97-2003) and password-encrypted documents are OLE
        // compound files, not zips; without the hint compare called the file
        // Markdown that is not UTF-8.
        let dir = tempfile::tempdir().expect("tempdir");
        let (docx, doc, out) = (
            dir.path().join("a.docx"),
            dir.path().join("b.doc"),
            dir.path().join("r.docx"),
        );
        std::fs::write(&docx, tiny_docx_bytes("Calibri")).expect("docx");
        let mut ole = b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1".to_vec();
        ole.resize(4096, 0);
        std::fs::write(&doc, ole).expect("doc");
        let args = [
            "jubarte",
            docx.to_str().unwrap(),
            doc.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ];
        let err = run(&job_of(&args))
            .expect_err("a .doc must be refused")
            .message;
        assert!(
            err.contains("b.doc is a Word 97-2003 (.doc) or encrypted document"),
            "{err}"
        );
        assert!(err.contains("save it as .docx"), "{err}");
        assert!(!out.exists());
        let err = read_document(&doc).expect_err("every command reads through read_document");
        assert!(err.contains("save it as .docx"), "{err}");
        // The same check names RTF for what it is.
        let rtf = dir.path().join("c.rtf");
        std::fs::write(&rtf, b"{\\rtf1\\ansi hello}").expect("rtf");
        let err = read_document(&rtf).expect_err("RTF must be refused");
        assert!(err.contains("c.rtf is an RTF file"), "{err}");
    }

    #[test]
    fn fail_on_substitution_is_exit_4_after_the_outputs() {
        let dir = tempfile::tempdir().expect("tempdir");
        let docx = dir.path().join("in.docx");
        let pdf = dir.path().join("out.pdf");
        let report = dir.path().join("fonts.json");
        std::fs::write(&docx, tiny_docx_bytes("DefinitelyNotAFont")).expect("docx");
        let err = run_convert(&ConvertJob {
            file: &docx,
            bytes: None,
            output: Some(&pdf),
            force: false,
            font_report: Some(&report),
            options: jubarte::convert::PdfOptions {
                revisions: RevisionStyle::Word,
                ..jubarte::convert::PdfOptions::default()
            },
            to: None,
            dpi: 96.0,
            report: None,
            pages: None,
            fail_on_substitution: true,
            quiet: false,
        })
        .expect_err("a substituted font fails the run");
        assert_eq!(err.code, EXIT_FONT_SUBSTITUTED);
        assert!(
            err.message.contains("--fail-on-substitution"),
            "{}",
            err.message
        );
        assert!(pdf.exists() && report.exists(), "outputs are written first");
    }

    #[test]
    fn a_plain_convert_error_keeps_exit_1() {
        let failure = ConvertFailure::from("boom");
        assert_eq!((failure.code, failure.message.as_str()), (1, "boom"));
    }

    #[test]
    fn convert_font_report_writes_json() {
        let dir = tempfile::tempdir().expect("tempdir");
        let docx = dir.path().join("in.docx");
        let pdf = dir.path().join("out.pdf");
        let report = dir.path().join("fonts.json");
        std::fs::write(&docx, tiny_docx_bytes("DefinitelyNotAFont")).expect("docx");
        run_convert(&ConvertJob {
            file: &docx,
            bytes: None,
            output: Some(&pdf),
            force: false,
            font_report: Some(&report),
            options: jubarte::convert::PdfOptions {
                revisions: RevisionStyle::Word,
                ..jubarte::convert::PdfOptions::default()
            },
            to: None,
            dpi: 96.0,
            report: None,
            pages: None,
            fail_on_substitution: false,
            quiet: false,
        })
        .expect("convert");
        assert!(pdf.exists());
        let json = std::fs::read_to_string(&report).expect("report");
        let v: serde_json::Value = serde_json::from_str(&json).expect("json");
        let rows = v.as_array().expect("array");
        assert!(
            rows.iter().any(|row| {
                row.get("requested").and_then(|x| x.as_str()) == Some("DefinitelyNotAFont")
                    && row.get("step").and_then(|x| x.as_str()) == Some("unknown")
            }),
            "report missing unknown family: {json}"
        );
    }

    mod regression_tests {
        use super::*;

        #[test]
        fn convert_without_revision_flags_uses_conventional_marks() {
            let cli = Cli::try_parse_from(["jubarte", "convert", "in.docx"]).unwrap();
            let Some(Command::Convert { render, .. }) = cli.command else {
                panic!("convert")
            };
            assert_eq!(render.revisions, Revisions::Conventional);
            assert_eq!(render.revision_palette, None);
            assert_eq!(
                pdf_options(&render, false).map(|o| o.revisions),
                Ok(RevisionStyle::Conventional)
            );
        }

        #[test]
        fn custom_revision_flags_reach_the_palette_parser() {
            let spec = "inserted=#aabbcc:plain";
            let cli = Cli::try_parse_from([
                "jubarte",
                "convert",
                "in.docx",
                "--revisions",
                "custom",
                "--revision-palette",
                spec,
            ])
            .unwrap();
            let Some(Command::Convert { render, .. }) = cli.command else {
                panic!("convert")
            };
            assert_eq!(render.revisions, Revisions::Custom);
            assert_eq!(render.revision_palette.as_deref(), Some(spec));
            assert_eq!(
                pdf_options(&render, false).map(|o| o.revisions),
                RevisionStyle::from_choice("custom", Some(spec))
            );
        }

        #[test]
        fn invalid_revision_flags_are_rejected_with_option_context() {
            assert!(
                Cli::try_parse_from(["jubarte", "convert", "in.docx", "--revisions", "unknown"])
                    .is_err()
            );
            assert_eq!(
                revision_style(Revisions::Custom, None).unwrap_err(),
                "--revisions custom needs --revision-palette"
            );
            for mode in [Revisions::Conventional, Revisions::Word] {
                assert_eq!(
                    revision_style(mode, Some("deleted=#000000")).unwrap_err(),
                    "--revision-palette needs --revisions custom"
                );
            }
            assert!(
                revision_style(Revisions::Custom, Some("deleted=red"))
                    .unwrap_err()
                    .starts_with("--revision-palette:")
            );
        }
    }

    const PAGE_BREAK: &str = r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#;

    fn pages_docx(texts: &[&str]) -> Vec<u8> {
        let body: Vec<String> = texts
            .iter()
            .map(|t| format!("<w:p><w:r><w:t>{t}</w:t></w:r></w:p>"))
            .collect();
        body_docx_bytes(&body.join(PAGE_BREAK))
    }

    fn file_names(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .expect("read_dir")
            .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn parse_pages_counts_from_one_and_sorts_without_repeats() {
        assert_eq!(parse_pages("1-3,7"), Ok(vec![0, 1, 2, 6]));
        assert_eq!(parse_pages("7, 2-3 ,2"), Ok(vec![1, 2, 6]));
        assert_eq!(parse_pages("4"), Ok(vec![3]));
        assert_eq!(parse_pages("5-5"), Ok(vec![4]));
    }

    #[test]
    fn parse_pages_refuses_zero_empty_backwards_and_words() {
        for (spec, why) in [
            ("0", "counted from 1"),
            ("2-0", "counted from 1"),
            ("", "empty item"),
            ("1,,2", "empty item"),
            ("3-1", "runs backwards"),
            ("two", "not a page number"),
            ("1-x", "not a page number"),
            ("-2", "not a page number"),
        ] {
            let err = parse_pages(spec).expect_err(spec);
            assert!(err.contains(why), "{spec}: {err}");
        }
    }

    #[test]
    fn convert_and_diff_render_parse_their_page_flags() {
        let cli =
            Cli::try_parse_from(["jubarte", "convert", "a.docx", "--pages", "2-3"]).expect("parse");
        let Some(Command::Convert { render, png, .. }) = cli.command else {
            panic!("expected convert");
        };
        assert_eq!(render.pages.as_deref(), Some("2-3"));
        assert!(!png);
        let cli = Cli::try_parse_from([
            "jubarte",
            "diff-render",
            "a.docx",
            "b.docx",
            "--dpi",
            "72",
            "--out-dir",
            "d",
            "--json",
            "--no-overlay",
            "--force",
        ])
        .expect("parse");
        let Some(Command::VisualDiff {
            a,
            b,
            dpi,
            out_dir,
            json,
            no_overlay,
            force,
        }) = cli.command
        else {
            panic!("expected diff-render");
        };
        assert_eq!(
            (a.as_path(), b.as_path()),
            (Path::new("a.docx"), Path::new("b.docx"))
        );
        assert_eq!(dpi, 72.0);
        assert_eq!(out_dir.as_deref(), Some(Path::new("d")));
        assert!(json && no_overlay && force);
        let cli =
            Cli::try_parse_from(["jubarte", "diff-render", "a.docx", "b.docx"]).expect("parse");
        let Some(Command::VisualDiff {
            dpi,
            out_dir,
            json,
            no_overlay,
            force,
            ..
        }) = cli.command
        else {
            panic!("expected diff-render");
        };
        assert_eq!(dpi, 100.0);
        assert!(out_dir.is_none() && !json && !no_overlay && !force);
    }

    fn convert_job<'a>(
        file: &'a Path,
        output: &'a Path,
        pages: Option<&'a [usize]>,
    ) -> ConvertJob<'a> {
        ConvertJob {
            file,
            bytes: None,
            output: Some(output),
            force: false,
            font_report: None,
            options: jubarte::convert::PdfOptions {
                revisions: RevisionStyle::Conventional,
                ..jubarte::convert::PdfOptions::default()
            },
            to: Some(Format::Png),
            dpi: 20.0,
            report: None,
            pages,
            fail_on_substitution: false,
            quiet: false,
        }
    }

    #[test]
    fn convert_pages_writes_the_selected_pages_named_by_page_number() {
        let dir = tempfile::tempdir().expect("tempdir");
        let docx = dir.path().join("in.docx");
        std::fs::write(&docx, pages_docx(&["A", "B", "C"])).expect("docx");
        let out = dir.path().join("out.pdf");
        run_convert(&convert_job(&docx, &out, Some(&[2, 0]))).expect("convert");
        assert_eq!(
            file_names(dir.path()),
            ["in.docx", "out-page-01.png", "out-page-03.png"]
        );
        let page_three = std::fs::read(dir.path().join("out-page-03.png")).expect("png");
        let all = jubarte::convert::docx_to_png(
            &pages_docx(&["A", "B", "C"]),
            jubarte::convert::PdfOptions::default(),
            20.0,
        )
        .expect("png");
        assert_eq!(
            page_three, all[2],
            "a selected page is the same bytes as in a full render"
        );
    }

    #[test]
    fn convert_pages_needs_png_and_an_existing_page() {
        let dir = tempfile::tempdir().expect("tempdir");
        let docx = dir.path().join("in.docx");
        std::fs::write(&docx, pages_docx(&["A", "B", "C"])).expect("docx");
        let out = dir.path().join("out.pdf");
        let err = run_convert(&convert_job(&docx, &out, Some(&[5]))).expect_err("page 6 of 3");
        assert!(
            err.message
                .contains("page 6 is out of range: the document has 3 pages"),
            "{}",
            err.message
        );
        assert_eq!(file_names(dir.path()), ["in.docx"], "nothing written");
    }

    fn diff_job<'a>(a: &'a Path, b: &'a Path, out_dir: Option<&'a Path>) -> DiffRenderJob<'a> {
        DiffRenderJob {
            a,
            b,
            out_dir,
            dpi: 20.0,
            json: false,
            overlay: true,
            force: false,
        }
    }

    #[test]
    fn diff_render_writes_only_the_changed_pages_and_diff_json() {
        let dir = tempfile::tempdir().expect("tempdir");
        let a = dir.path().join("a.docx");
        let b = dir.path().join("b.docx");
        std::fs::write(&a, pages_docx(&["Page one.", "The fee is ten."])).expect("a");
        std::fs::write(
            &b,
            pages_docx(&["Page one.", "The fee is twenty.", "Extra."]),
        )
        .expect("b");
        let out = dir.path().join("diff");
        assert_eq!(run_diff_render(&diff_job(&a, &b, Some(&out))), Ok(true));
        assert_eq!(
            file_names(&out),
            [
                "a-page-02.png",
                "b-page-02.png",
                "b-page-03.png",
                "diff-page-02.png",
                "diff.json"
            ]
        );
        let json: serde_json::Value =
            serde_json::from_slice(&std::fs::read(out.join("diff.json")).expect("json"))
                .expect("parse");
        assert_eq!(json["dpi"], 20.0);
        assert_eq!(json["a_pages"], 2);
        assert_eq!(json["b_pages"], 3);
        assert_eq!(json["changed"], 2);
        assert_eq!(json["pages"][0]["changed_ratio"], 0.0);
        assert!(json["pages"][1]["changed_ratio"].as_f64().expect("ratio") > 0.0);
        let raw = std::fs::read_to_string(out.join("diff.json")).expect("json");
        let ratio = raw
            .split("\"changed_ratio\":")
            .nth(2)
            .expect("page 2's ratio");
        let ratio = &ratio[..ratio.find(',').expect("comma")];
        let shortest = ratio.parse::<f32>().expect("f32").to_string();
        assert_eq!(
            ratio, shortest,
            "an f32 printed at f32 precision, not widened"
        );
        assert_eq!(json["pages"][2]["only_in"], "b");

        let err = run_diff_render(&diff_job(&a, &b, Some(&out))).expect_err("files exist");
        assert!(err.message.contains("already exists"), "{err:?}");
        let again = DiffRenderJob {
            force: true,
            overlay: false,
            json: true,
            ..diff_job(&a, &b, Some(&out))
        };
        assert_eq!(run_diff_render(&again), Ok(true));
    }

    #[test]
    fn diff_render_of_equal_documents_writes_only_diff_json() {
        let dir = tempfile::tempdir().expect("tempdir");
        let a = dir.path().join("a.docx");
        std::fs::write(&a, pages_docx(&["Same."])).expect("a");
        let out = dir.path().join("diff");
        assert_eq!(run_diff_render(&diff_job(&a, &a, Some(&out))), Ok(false));
        assert_eq!(file_names(&out), ["diff.json"]);
        assert_eq!(run_diff_render(&diff_job(&a, &a, None)), Ok(false));
    }

    #[test]
    fn diff_render_without_an_overlay_writes_no_diff_png() {
        let dir = tempfile::tempdir().expect("tempdir");
        let a = dir.path().join("a.docx");
        let b = dir.path().join("b.docx");
        std::fs::write(&a, pages_docx(&["The fee is ten."])).expect("a");
        std::fs::write(&b, pages_docx(&["The fee is twenty."])).expect("b");
        let out = dir.path().join("diff");
        let job = DiffRenderJob {
            overlay: false,
            ..diff_job(&a, &b, Some(&out))
        };
        assert_eq!(run_diff_render(&job), Ok(true));
        assert_eq!(
            file_names(&out),
            ["a-page-01.png", "b-page-01.png", "diff.json"]
        );
        let missing = dir.path().join("missing.docx");
        let err = run_diff_render(&diff_job(&missing, &b, None)).expect_err("no file");
        assert!(err.message.contains("missing.docx"), "{err:?}");
    }

    #[test]
    fn convert_takes_update_fields() {
        let cli = Cli::try_parse_from([
            "jubarte",
            "convert",
            "in.docx",
            "-o",
            "out.docx",
            "--update-fields",
        ])
        .unwrap();
        let Some(Command::Convert { update_fields, .. }) = cli.command else {
            panic!("expected convert");
        };
        assert!(update_fields);
        // `fields update` left with the subcommand: three words are no compare.
        assert!(
            Cli::try_parse_from(["jubarte", "fields", "update", "in.docx", "-o", "o.docx"])
                .is_err()
        );
    }

    #[test]
    fn refresh_fields_writes_the_report_and_says_when_it_cannot() {
        let dir = tempfile::tempdir().expect("tempdir");
        let report = dir.path().join("fields.json");
        let (updated, json) = refresh_fields(&tiny_docx_bytes("Calibri")).expect("update");
        publish_file(&report, json.as_bytes(), false).expect("report");
        assert_eq!(
            jubarte::inspect::paragraphs(&updated.docx).unwrap()[0].text,
            "HELLO"
        );
        let json: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&report).unwrap()).unwrap();
        assert_eq!(json["page_count"], 1);
        assert!(json["fields"].is_array());
        let err = publish_file(&dir.path().join("no/such/dir.json"), b"{}", false).unwrap_err();
        assert!(err.contains("writing"), "{err}");
        assert!(refresh_fields(b"not a zip").is_err());
    }
}
