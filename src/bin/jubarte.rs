// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte` — generate a tracked-changes (redline) `.docx` from two documents.
//!
//! ```text
//! jubarte original.docx modified.docx
//!   → writes original_v_modified.docx
//! jubarte -b a.docx -m b.docx -o out.docx --author "Jane" --date 2024-01-02T00:00:00Z
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

/// `println!` for the listings: a reader that stops early (`jubarte inspect
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
                    })
                    .collect()
            }),
        }
    }
}

fn run_scrub(
    file: &Path,
    output: &Path,
    force: bool,
    options: &jubarte::scrub::ScrubOptions,
) -> Result<(), String> {
    ensure_writable(output, force)?;
    let bytes = read_document(file)?;
    let out = jubarte::scrub::scrub(&bytes, options).map_err(|e| format!("scrub failed: {e}"))?;
    std::fs::write(output, &out).map_err(|e| format!("writing {}: {e}", output.display()))
}

/// Who a patch's changes are by when `--author` does not say: git's
/// `user.name`, else Redline.
fn default_author() -> String {
    std::process::Command::new("git")
        .args(["config", "user.name"])
        .stderr(std::process::Stdio::null())
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "Redline".to_string())
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

/// Whether two CLI paths name the same file. Two existing paths are compared
/// by identity, which sees through hard links, symlinks and a volume that
/// ignores case; a path not written yet is keyed by its canonicalized parent
/// and its name (`out.pdf` and `./out.pdf` match).
fn same_path(a: &Path, b: &Path) -> bool {
    if let Ok(same) = same_file::is_same_file(a, b) {
        return same;
    }
    fn key(p: &Path) -> Option<PathBuf> {
        let name = p.file_name()?;
        let parent = match p.parent() {
            Some(dir) if !dir.as_os_str().is_empty() => dir.to_path_buf(),
            _ => PathBuf::from("."),
        };
        Some(std::fs::canonicalize(parent).ok()?.join(name))
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
fn ensure_writable(output: &Path, force: bool) -> Result<(), String> {
    if output.exists() && !force {
        return Err(format!(
            "output '{}' already exists (use --force to overwrite)",
            output.display()
        ));
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
) -> Result<(), String> {
    for (other, name) in others {
        if same_path(side, other) {
            return Err(format!(
                "{flag} '{}' is the same file as the {name}",
                side.display()
            ));
        }
    }
    ensure_writable(side, force)
}

/// Shared body for `accept` / `reject`: read the redline, apply the package-wide
/// resolution, and write the result under the compare path's no-clobber
/// contract. Generic over the resolver's error so neither `OpcError`'s path nor
/// the two arms' bodies are duplicated.
fn run_resolution(
    file: &Path,
    output: &Path,
    force: bool,
    filter: &jubarte::changes::ChangeFilter,
    apply: fn(
        &[u8],
        &jubarte::changes::ChangeFilter,
    ) -> Result<Vec<u8>, jubarte::changes::ChangeError>,
    what: &str,
) -> Result<(), String> {
    ensure_writable(output, force)?;
    let bytes = read_document(file)?;
    let out = apply(&bytes, filter).map_err(|e| format!("{what} failed: {e}"))?;
    std::fs::write(output, &out).map_err(|e| format!("writing {}: {e}", output.display()))
}

fn run_changes(file: &Path, json: bool) -> Result<(), String> {
    let bytes = read_document(file)?;
    let changes = jubarte::changes::list_changes(&bytes).map_err(|e| e.to_string())?;
    for c in &changes {
        if json {
            outln!("{}", serde_json::to_string(c).map_err(|e| e.to_string())?);
            continue;
        }
        let kind = serde_json::to_value(c.kind).map_err(|e| e.to_string())?;
        let preview: String = c.text.chars().take(60).collect();
        let inside = c
            .inside
            .as_deref()
            .map(|id| format!("\tinside {id}"))
            .unwrap_or_default();
        outln!(
            "{}\t{}\t{}\t{}\t{preview:?}{inside}",
            c.id,
            kind.as_str().unwrap_or("?"),
            c.target,
            c.author.as_deref().unwrap_or("-"),
        );
    }
    if !json {
        outln!("{} change(s)", changes.len());
    }
    Ok(())
}

fn run_comments(file: &Path, json: bool, author: Option<&str>, latest: bool) -> Result<(), String> {
    let bytes = read_document(file)?;
    let comments = jubarte::comments::list_comments(&bytes).map_err(|e| e.to_string())?;
    let comments = jubarte::comments::select_comments(comments, author, latest);
    for c in &comments {
        if json {
            outln!("{}", serde_json::to_string(c).map_err(|e| e.to_string())?);
            continue;
        }
        let text: String = c.text.chars().take(60).collect();
        let anchor: String = c.anchor_text.chars().take(40).collect();
        let thread = c
            .parent
            .map(|p| format!("\treply to {p}"))
            .unwrap_or_default();
        let done = if c.done { "\tresolved" } else { "" };
        outln!(
            "{}\t{}\t{}\t{text:?}\ton {anchor:?}{thread}{done}",
            c.id,
            c.paragraph.as_deref().unwrap_or("-"),
            c.author,
        );
    }
    if !json {
        outln!("{} comment(s)", comments.len());
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
    compress: bool,
    font_report: Option<&'a Path>,
    revisions: jubarte::convert::RevisionStyle,
    pdf: bool,
    png: bool,
    dpi: f32,
    report: Option<&'a Path>,
    /// Zero-based pages to rasterize; `None` for all.
    pages: Option<&'a [usize]>,
    /// Exit [`EXIT_FONT_SUBSTITUTED`] when a requested font was substituted.
    fail_on_substitution: bool,
    /// Comment placement and page selection.
    page: PageOptions,
    /// Say what was written on stderr: stdout carries a patch.
    status_to_stderr: bool,
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
#[derive(Debug)]
struct ConvertFailure {
    code: u8,
    message: String,
}

impl From<String> for ConvertFailure {
    fn from(message: String) -> Self {
        Self { code: 1, message }
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
            eprintln!("error: {message}");
            ExitCode::from(code)
        }
    }
}

fn run_convert(job: &ConvertJob<'_>) -> Result<(), ConvertFailure> {
    let output = job
        .output
        .map(Path::to_path_buf)
        .unwrap_or_else(|| job.file.with_extension("pdf"));
    let want_pdf = job.pdf || !job.png;
    if job.pages.is_some() && !job.png {
        return Err("--pages selects PNG pages; add --png".into());
    }
    for (side, flag) in [(job.font_report, "--font-report"), (job.report, "--report")] {
        if let Some(side) = side {
            let others = [(output.as_path(), "PDF output"), (job.file, "input")];
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
        None => read_document(job.file)?,
    };
    let options = job.page.apply(jubarte::convert::PdfOptions {
        compress: job.compress,
        revisions: job.revisions,
        ..jubarte::convert::PdfOptions::default()
    });
    let rendered = jubarte::convert::render(
        &bytes,
        options,
        jubarte::convert::RenderRequest {
            pdf: want_pdf,
            png_dpi: job.png.then_some(job.dpi),
            pages: job.pages.map(<[usize]>::to_vec),
        },
    )
    .map_err(|e| format!("convert failed: {e}"))?;
    let pages = rendered.report.page_count;
    // `render` returns the selected pages ascending and without repeats.
    let indices: Vec<usize> = match job.pages {
        Some(selected) => {
            let mut selected = selected.to_vec();
            selected.sort_unstable();
            selected.dedup();
            selected
        }
        None => (0..rendered.pngs.len()).collect(),
    };
    // The page count is known only now; check every PNG path before the
    // first write so a refused page leaves no partial bundle.
    let png_paths: Vec<PathBuf> = indices
        .iter()
        .map(|&i| dir.join(png_name(&stem, i, pages)))
        .collect();
    for path in &png_paths {
        ensure_writable(path, job.force)?;
    }
    if let Some(pdf) = &rendered.pdf {
        std::fs::write(&output, pdf).map_err(|e| format!("writing {}: {e}", output.display()))?;
        say(
            job.status_to_stderr,
            format_args!(
                "wrote {} ({} bytes, {pages} page{})",
                output.display(),
                pdf.len(),
                if pages == 1 { "" } else { "s" }
            ),
        );
    }
    if job.png {
        for (path, png) in png_paths.iter().zip(&rendered.pngs) {
            std::fs::write(path, png).map_err(|e| format!("writing {}: {e}", path.display()))?;
        }
        say(
            job.status_to_stderr,
            format_args!(
                "wrote {} PNG page{} ({}-page-NN.png, {} dpi)",
                rendered.pngs.len(),
                if rendered.pngs.len() == 1 { "" } else { "s" },
                dir.join(&stem).display(),
                job.dpi
            ),
        );
    }
    if let Some(report) = job.font_report {
        let json = jubarte::convert::font_report_json(&rendered.report.fonts);
        std::fs::write(report, json).map_err(|e| format!("writing {}: {e}", report.display()))?;
    }
    if let Some(report) = job.report {
        std::fs::write(report, rendered.report.to_json())
            .map_err(|e| format!("writing {}: {e}", report.display()))?;
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

/// A status line: on stdout, or on stderr when stdout carries a patch.
fn say(to_stderr: bool, line: std::fmt::Arguments<'_>) {
    if to_stderr {
        eprintln!("{line}");
    } else {
        outln!("{line}");
    }
}

/// `<stem>-page-NN.png`, zero-padded to the page count's width (at least 2).
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

/// What `diff-render` compares and where it writes.
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
fn run_diff_render(job: &DiffRenderJob<'_>) -> Result<bool, String> {
    let a = read_document(job.a)?;
    let b = read_document(job.b)?;
    let options = jubarte::convert::DiffOptions {
        dpi: job.dpi,
        overlay: job.overlay && job.out_dir.is_some(),
        ..jubarte::convert::DiffOptions::default()
    };
    let diff = jubarte::convert::diff_render(&a, &b, &options)
        .map_err(|e| format!("diff-render failed: {e}"))?;
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
        for (path, bytes) in &files {
            std::fs::write(path, bytes).map_err(|e| format!("writing {}: {e}", path.display()))?;
        }
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

fn png_name(stem: &str, index: usize, count: usize) -> String {
    let width = count.to_string().len().max(2);
    format!("{stem}-page-{:0width$}.png", index + 1)
}

fn run_inspect(file: &Path, json: bool) -> Result<(), String> {
    let bytes = read_document(file)?;
    if json {
        outln!(
            "{}",
            jubarte::inspect::inspect_json(&bytes).map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    let summary = jubarte::inspect::summary(&bytes).map_err(|e| e.to_string())?;
    let paragraphs = jubarte::inspect::paragraphs(&bytes).map_err(|e| e.to_string())?;
    outln!(
        "sha256: {}\nparagraphs: {}  tables: {}  fields: {}  sections: {}  comments: {}  revisions: {}  footnotes: {}  endnotes: {}  headers: {}  footers: {}  images: {}  numbering: {}  track_changes: {}",
        jubarte::inspect::source_sha256(&bytes),
        summary.paragraphs,
        summary.tables,
        summary.fields,
        summary.sections,
        summary.comments,
        summary.revisions,
        summary.footnotes,
        summary.endnotes,
        summary.headers,
        summary.footers,
        summary.images,
        summary.list_numbering,
        summary.track_changes
    );
    for p in &paragraphs {
        let mut flags = Vec::new();
        if let Some(style) = &p.style {
            flags.push(style.clone());
        }
        if p.numbered {
            flags.push("numbered".into());
        }
        if p.in_table {
            flags.push("table".into());
        }
        if p.page_break {
            flags.push("page-break".into());
        }
        flags.extend(p.limitations.iter().cloned());
        let preview: String = p.text.chars().take(80).collect();
        let more = if p.text.chars().count() > 80 {
            "…"
        } else {
            ""
        };
        outln!("{}\t[{}]\t{preview}{more}", p.id, flags.join(","));
    }
    Ok(())
}

fn run_inspect_tables(file: &Path) -> Result<(), String> {
    let bytes = read_document(file)?;
    let tables = jubarte::inspect::tables(&bytes).map_err(|e| e.to_string())?;
    if tables.is_empty() {
        outln!("no tables");
    }
    for table in &tables {
        let columns = table.rows.iter().map(Vec::len).max().unwrap_or(0);
        let widths: Vec<String> = table.widths_dxa.iter().map(u32::to_string).collect();
        outln!(
            "table {}: {}x{columns} header_rows={} widths={}",
            table.index,
            table.rows.len(),
            table.header_rows,
            widths.join(",")
        );
        for row in &table.rows {
            let cells: Vec<String> = row
                .iter()
                .map(|cell| {
                    let ids = if cell.paragraph_ids.is_empty() {
                        "-".to_string()
                    } else {
                        cell.paragraph_ids.join(",")
                    };
                    let text = cell
                        .text
                        .replace('\\', "\\\\")
                        .replace('\n', "\\n")
                        .replace('\t', "\\t");
                    format!("{ids}={text}")
                })
                .collect();
            outln!("{}", cells.join("\t"));
        }
    }
    Ok(())
}

/// `jubarte audit`: print the findings; `Ok(true)` when one fails the run
/// (an `error`, or a `warning` under `strict`).
fn run_audit(file: &Path, json: bool, rules: &[String], strict: bool) -> Result<bool, String> {
    let bytes = read_document(file)?;
    let rules: Vec<&str> = rules.iter().map(String::as_str).collect();
    let report = jubarte::audit::audit_report(&bytes, &rules).map_err(|e| e.to_string())?;
    if json {
        outln!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
        );
    } else {
        for finding in &report.findings {
            outln!(
                "{}\t{}\t{}\t{}",
                finding.severity,
                finding.code,
                finding.location,
                finding.message
            );
        }
        let count = |severity: &str| {
            report
                .findings
                .iter()
                .filter(|finding| finding.severity == severity)
                .count()
        };
        outln!(
            "{} findings: {} error, {} warning, {} info ({} rules{})",
            report.findings.len(),
            count("error"),
            count("warning"),
            count("info"),
            report.rules.len(),
            if report.layout { ", with layout" } else { "" }
        );
    }
    Ok(report
        .findings
        .iter()
        .any(|finding| finding.severity == "error" || (strict && finding.severity == "warning")))
}

/// `read`: the agent view, with page markers from the layout pass unless
/// `--no-page-markers` skips it.
fn run_text(file: &Path, args: &ReadArgs) -> Result<(), String> {
    let bytes = read_document(file)?;
    let source = file.file_name().map(|n| n.to_string_lossy().into_owned());
    print_agent_view(&bytes, source, args)
}

/// The agent view of `bytes` on stdout, its warnings on stderr; `source` is
/// the name the header prints.
fn print_agent_view(bytes: &[u8], source: Option<String>, args: &ReadArgs) -> Result<(), String> {
    let select = jubarte::markdown::Select::from_flags(
        args.paragraphs.as_deref(),
        args.head,
        args.tail,
        args.changed,
        args.by.as_deref(),
    )?;
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
    write_stdout(&view.markdown)
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

/// `<dir>/<stem>.edit` next to the source.
fn default_out_dir(file: &Path) -> PathBuf {
    let stem = file.file_stem().map_or_else(
        || "document".to_string(),
        |s| s.to_string_lossy().into_owned(),
    );
    match file.parent().filter(|p| !p.as_os_str().is_empty()) {
        Some(dir) => dir.join(format!("{stem}.edit")),
        None => PathBuf::from(format!("{stem}.edit")),
    }
}

/// Options for `edit` and `add`.
struct EditJob<'a> {
    file: &'a Path,
    out_dir: &'a Path,
    mode: Mode,
    dry_run: bool,
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

/// The plan of an `edit` or `add` invocation and the notes its flags left:
/// `--plan`'s file, or the plan the operation flags describe.
fn edit_plan(
    verb: jubarte::edit::flags::Verb,
    plan: Option<&Path>,
    operations: &[FlagOp],
    options: &EditOptions,
    source: &[u8],
) -> Result<(jubarte::edit::EditPlan, Vec<String>), (u8, String)> {
    if let Some(path) = plan {
        let json = std::fs::read_to_string(path)
            .map_err(|e| (1, format!("reading {}: {e}", path.display())))?;
        let plan = jubarte::edit::EditPlan::from_json(&json)
            .map_err(|e| (EXIT_PLAN_REFUSED, e.to_string()))?;
        return Ok((plan, Vec::new()));
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
    Ok((built.plan, built.notes))
}

fn run_edit(
    job: &EditJob<'_>,
    plan: &jubarte::edit::EditPlan,
    source: &[u8],
    notes: &[String],
) -> Result<(), (u8, String)> {
    let fail = |m: String| (1u8, m);
    if !job.dry_run {
        if job.out_dir.exists() && !job.force {
            return Err(fail(format!(
                "output directory '{}' already exists (use --force to replace its files)",
                job.out_dir.display()
            )));
        }
        let input_dir = job
            .file
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        if same_dir(job.out_dir, input_dir) {
            return Err(fail(
                "--out-dir must not be the input's own directory".into(),
            ));
        }
    }
    if job.dry_run {
        let report = jubarte::edit::preview_plan(source, plan).map_err(|e| refused(&e))?;
        return write_stdout(report.to_jsonl()).map_err(fail);
    }
    let result = jubarte::edit::apply_plan(source, plan).map_err(|e| refused(&e))?;
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
            let firsts: Vec<serde_json::Value> = rendered
                .report
                .pages
                .iter()
                .map(|p| {
                    p.text
                        .lines()
                        .next()
                        .unwrap_or("")
                        .chars()
                        .take(60)
                        .collect::<String>()
                        .into()
                })
                .collect();
            starts.insert((*name).into(), firsts.into());
        }
        let render_line = serde_json::json!({
            "ev": "render",
            "engine": format!("jubarte {}", env!("CARGO_PKG_VERSION")),
            "pages": pages,
            "page_starts": starts,
        });
        insert_before_summary(&mut jsonl, &render_line.to_string());
    }
    std::fs::create_dir_all(job.out_dir)
        .map_err(|e| fail(format!("creating {}: {e}", job.out_dir.display())))?;
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
    for (name, rendered) in renders {
        if let Some(pdf) = rendered.pdf {
            outputs.push((format!("{name}.pdf"), pdf));
        }
        let count = rendered.pngs.len();
        for (i, png) in rendered.pngs.into_iter().enumerate() {
            outputs.push((png_name(name, i, count), png));
        }
    }
    let mut saved = Vec::new();
    for (name, bytes) in &outputs {
        let path = job.out_dir.join(name);
        std::fs::write(&path, bytes)
            .map_err(|e| fail(format!("writing {}: {e}", path.display())))?;
        saved.push(serde_json::json!({"f": name, "bytes": bytes.len(), "sha256": jubarte::inspect::source_sha256(bytes)}));
    }
    let save_line = serde_json::json!({"ev": "save", "dir": job.out_dir.display().to_string(), "outputs": saved});
    insert_before_summary(&mut jsonl, &save_line.to_string());
    std::fs::write(job.out_dir.join("report.jsonl"), &jsonl)
        .map_err(|e| fail(format!("writing report.jsonl: {e}")))?;
    if job.quiet {
        return Ok(());
    }
    let summary = jsonl.lines().last().unwrap_or("").to_string();
    outln!("{summary}");
    let names: Vec<&str> = outputs
        .iter()
        .map(|(name, _)| name.as_str())
        .chain(["report.jsonl"])
        .collect();
    outln!(
        "wrote {} ({} files: {})",
        job.out_dir.display(),
        names.len(),
        names.join(", ")
    );
    for note in notes {
        outln!("note: {note}");
    }
    for outcome in &result.report.operations {
        if let (Some(given), Some(read_as)) = (&outcome.anchor_given, &outcome.anchor_read_as) {
            outln!(
                "note: {}: anchor {given:?} read as {read_as:?} (Markdown marks are not document text)",
                outcome.id
            );
        }
    }
    // The changed blocks, as the agent view reads them back: the tracked
    // redline, or in editing mode its accepted text.
    let shown = match job.mode {
        Mode::Suggesting => "redline.docx",
        Mode::Editing => "clean.docx",
    };
    // The files are written and report.jsonl says so: a view that cannot be
    // read back is a warning, not a failed edit.
    match jubarte::markdown::changed_view(
        &result.redline,
        &result.report.author,
        job.mode == Mode::Editing,
        Some(&job.out_dir.join(shown).display().to_string()),
    ) {
        Ok(view) => write_stdout(&view).map_err(fail),
        Err(e) => {
            eprintln!("warning: the changed blocks cannot be shown: {e}");
            Ok(())
        }
    }
}

/// One `edit` or `add` invocation, parsed.
struct EditFlags<'a> {
    verb: jubarte::edit::flags::Verb,
    file: &'a Path,
    plan: Option<&'a Path>,
    operations: &'a [FlagOp],
    options: &'a EditOptions,
    dry_run: bool,
    pdf: bool,
    png: bool,
    dpi: f32,
    revisions: jubarte::convert::RevisionStyle,
}

fn run_flags(flags: &EditFlags<'_>) -> Result<(), (u8, String)> {
    let source = read_document(flags.file).map_err(|m| (1u8, m))?;
    let (plan, notes) = edit_plan(
        flags.verb,
        flags.plan,
        flags.operations,
        flags.options,
        &source,
    )?;
    if flags.options.editing_mode
        && let Some(message) = jubarte::edit::flags::editing_mode_conflict(&plan)
    {
        return Err((EXIT_USAGE, message.to_string()));
    }
    let out_dir = flags
        .options
        .out_dir
        .clone()
        .unwrap_or_else(|| default_out_dir(flags.file));
    let job = EditJob {
        file: flags.file,
        out_dir: &out_dir,
        mode: if flags.options.editing_mode {
            Mode::Editing
        } else {
            Mode::Suggesting
        },
        dry_run: flags.dry_run,
        force: flags.options.force,
        pdf: flags.pdf,
        png: flags.png,
        dpi: flags.dpi,
        revisions: flags.revisions,
        quiet: flags.options.quiet,
    };
    run_edit(&job, &plan, &source, &notes)
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
fn refused(e: &jubarte::edit::EditError) -> (u8, String) {
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
            if let Some(c) = &o.code {
                m.insert("code".into(), c.clone().into());
            }
            if let Some(msg) = &o.message {
                m.insert("message".into(), msg.clone().into());
            }
            v.to_string() + "\n"
        })
        .collect();
    let summary = serde_json::json!({
        "ev": "summary",
        "status": "failed",
        "code": e.code,
        "operation": e.operation,
        "message": e.message,
    });
    outln!("{report_lines}{summary}");
    (EXIT_PLAN_REFUSED, format!("plan refused: {e}"))
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

/// A fully-resolved comparison job (positional/named merged, output computed).
#[derive(Debug, PartialEq)]
struct Job {
    original: PathBuf,
    modified: PathBuf,
    /// `None` (the shorthand without -o): print the redline's agent view.
    output: Option<PathBuf>,
    /// The options of that view.
    read: ReadArgs,
    author: String,
    date: String,
    force: bool,
    quiet: bool,
    detail_threshold: Option<f64>,
    powertools_faithful: bool,
    no_paragraph_merge: bool,
}

/// Native runtime defaults; required inputs have already been checked by clap.
/// `explicit` is `compare A B`, which writes `<A>_v_<B>.docx` by default;
/// the shorthand `A B` prints the redline's view unless -o names a file.
fn resolve_compare(compare: CompareArgs, read: ReadArgs, explicit: bool) -> Job {
    let original = compare
        .original
        .or(compare.original_pos)
        .expect("clap requires ORIGINAL");
    let modified = compare
        .modified
        .or(compare.modified_pos)
        .expect("clap requires MODIFIED");
    let output = compare
        .output
        .or_else(|| explicit.then(|| default_output(&original, &modified)));
    Job {
        original,
        modified,
        output,
        read,
        author: compare.author,
        date: compare.date,
        force: compare.force,
        quiet: compare.quiet,
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

fn run(job: &Job) -> Result<(), String> {
    if let Some(output) = &job.output {
        ensure_writable(output, job.force)?;
    }
    let to_markdown = job.output.as_deref().and_then(Format::of_path) == Some(Format::Md);
    let original = read_document(&job.original)?;
    let modified = read_document(&job.modified)?;
    let settings = comparer_settings(
        &job.author,
        &job.date,
        job.detail_threshold,
        job.powertools_faithful,
        job.no_paragraph_merge,
    );
    let formats = (
        Format::of_input(None, &job.original, &original),
        Format::of_input(None, &job.modified, &modified),
    );
    let out = if formats == (Format::Docx, Format::Docx) && !to_markdown {
        jubarte::document_comparer::compare_documents_with_settings(&original, &modified, &settings)
            .map_err(|e| {
                refusal("compare", &e).unwrap_or_else(|| format!("compare failed: {e:?}"))
            })?
    } else {
        let old = Input::new(&job.original, formats.0, original)?;
        let new = Input::new(&job.modified, formats.1, modified)?;
        let to = job
            .output
            .as_deref()
            .and_then(Format::of_path)
            .unwrap_or(Format::Docx);
        let options = jubarte::markdown::RedlineOptions {
            settings,
            ..Default::default()
        };
        compared(&old, &new, to, &options)?
    };

    let Some(output) = &job.output else {
        let name = default_output(&job.original, &job.modified);
        let source = format!("{} (not written; -o keeps it)", name.display());
        return print_agent_view(&out, Some(source), &job.read);
    };
    std::fs::write(output, &out).map_err(|e| format!("writing {}: {e}", output.display()))?;

    if !job.quiet {
        outln!("wrote {} ({} bytes)", output.display(), out.len());
    }
    Ok(())
}

/// Shared `Result → ExitCode` mapping for every command arm.
fn exit_code(r: Result<(), String>) -> ExitCode {
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

/// A document read for comparison: Word bytes or Markdown text.
struct Input<'p> {
    path: &'p Path,
    bytes: Vec<u8>,
    markdown: Option<String>,
}

impl<'p> Input<'p> {
    fn new(path: &'p Path, format: Format, bytes: Vec<u8>) -> Result<Self, String> {
        let markdown = match format {
            Format::Md => Some(markdown_text(path, bytes.clone())?),
            Format::Docx => None,
            Format::Pdf | Format::Png => {
                return Err(format!("{}: PDF and PNG are not inputs", path.display()));
            }
        };
        Ok(Self {
            path,
            bytes,
            markdown,
        })
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
    jubarte::admission::sniff(&bytes).map_err(|refused| {
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
        format!(
            "{}: {} is {}{hint}",
            refused.code(),
            path.display(),
            refused.message
        )
    })?;
    Ok(bytes)
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

/// Reads images a Markdown file names: local paths, relative to `dir`, with
/// `%XX` escapes decoded. URLs are not fetched.
fn image_loader(dir: PathBuf) -> impl Fn(&str) -> Option<Vec<u8>> {
    move |name: &str| {
        if name.contains("://") || name.starts_with("data:") {
            return None;
        }
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
        let path = PathBuf::from(String::from_utf8(decoded).ok()?);
        std::fs::read(if path.is_absolute() {
            path
        } else {
            dir.join(path)
        })
        .ok()
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

/// `old` against `new` in format `to`: CriticMarkup (both Markdown) or a
/// Word redline.
fn compared(
    old: &Input<'_>,
    new: &Input<'_>,
    to: Format,
    options: &jubarte::markdown::RedlineOptions<'_>,
) -> Result<Vec<u8>, String> {
    match to {
        Format::Md => match (&old.markdown, &new.markdown) {
            (Some(old), Some(new)) => Ok(jubarte::markdown::diff_markdown(old, new).into_bytes()),
            _ => Err(format!(
                "Markdown output needs both documents in Markdown ({} is Word): \
                 write a Word redline (-o FILE.docx) or a PDF (-o FILE.pdf) instead",
                if old.markdown.is_none() {
                    old.path
                } else {
                    new.path
                }
                .display()
            )),
        },
        _ => jubarte::markdown::redline(old.source(), new.source(), options)
            .map_err(|e| format!("compare failed: {e}")),
    }
}

/// Options for `diff`.
struct DiffJob<'a> {
    old: &'a Path,
    new: &'a Path,
    output: Option<&'a Path>,
    to: Option<Format>,
    from: Option<Format>,
    force: bool,
    settings: jubarte::comparer::WmlComparerSettings,
    reference: Option<&'a Path>,
    critic: bool,
    resource_path: Option<&'a Path>,
    revisions: jubarte::convert::RevisionStyle,
    /// Comment placement and page selection in PDF or PNG output.
    page: PageOptions,
    /// Print the patch, wrapped at these columns; `None` for `--format
    /// critic`.
    patch: Option<usize>,
}

/// Unified text diff uses the shared core directly, with no redline/render pass.
fn run_text_diff(
    old_path: &Path,
    new_path: &Path,
    output: Option<&Path>,
    from: Option<Format>,
    force: bool,
    options: &jubarte::text_diff::TextOptions,
) -> Result<(), String> {
    if let Some(path) = output {
        ensure_writable(path, force)?;
    }
    let old_bytes = read_document(old_path)?;
    let new_bytes = read_document(new_path)?;
    let old = Input::new(
        old_path,
        Format::of_input(from, old_path, &old_bytes),
        old_bytes,
    )?;
    let new = Input::new(
        new_path,
        Format::of_input(from, new_path, &new_bytes),
        new_bytes,
    )?;
    let patch = jubarte::text_diff::diff_documents_view(old.source(), new.source(), options)?;
    if let Some(path) = output {
        std::fs::write(path, &patch).map_err(|e| format!("writing {}: {e}", path.display()))?;
        eprintln!("wrote {} ({} bytes)", path.display(), patch.len());
        Ok(())
    } else {
        write_stdout(&patch)
    }
}

fn run_diff(job: &DiffJob<'_>) -> Result<(), String> {
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
    let both_markdown = old.markdown.is_some() && new.markdown.is_some();
    let to = job
        .to
        .or_else(|| job.output.and_then(Format::of_path))
        .unwrap_or(if both_markdown {
            Format::Md
        } else {
            Format::Docx
        });
    if matches!(to, Format::Docx | Format::Md) {
        for (given, flag) in [
            (job.page.move_comments, "--move-comments"),
            (job.page.changed_only, "--changed-only"),
        ] {
            if given {
                return Err(format!("{flag} applies to PDF or PNG output only"));
            }
        }
    }
    let output = match (job.output, to) {
        (Some(path), _) => Some(path.to_path_buf()),
        (None, Format::Md) => None,
        (None, Format::Docx) => Some(default_output(job.old, job.new)),
        (None, Format::Pdf | Format::Png) => {
            Some(default_output(job.old, job.new).with_extension("pdf"))
        }
    };
    let reference = job.reference.map(read_document).transpose()?;
    // Images of a Markdown side are read next to it (both sides share
    // --resource-path when given).
    let markdown_side = if old.markdown.is_some() {
        job.old
    } else {
        job.new
    };
    let loader = image_loader(resource_dir(markdown_side, job.resource_path));
    let options = jubarte::markdown::RedlineOptions {
        settings: job.settings.clone(),
        reference: reference.as_deref(),
        critic: job.critic,
        images: Some(&loader),
    };
    // Two Markdown documents with no --output: the patch is all there is.
    let out = if job.patch.is_some() && output.is_none() && both_markdown {
        Vec::new()
    } else {
        compared(
            &old,
            &new,
            if to == Format::Md { to } else { Format::Docx },
            &options,
        )?
    };
    // The patch is printed only once the output is written, so a refused
    // output prints no patch.
    let patch = match job.patch {
        Some(columns) => {
            let name = |path: &Path| path.display().to_string();
            let patch = jubarte::markdown::patch_documents(
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
            .map_err(|e| format!("compare failed: {e}"))?;
            Some(patch.render(columns))
        }
        None => None,
    };
    if let (Some(patch), None) = (&patch, &output) {
        return write_stdout(patch);
    }
    write_diff_output(job, to, output, &out, patch.is_some())?;
    match patch {
        Some(patch) => write_stdout(&patch),
        None => Ok(()),
    }
}

/// Write `diff`'s Markdown, Word, PDF or PNG output. With a patch on
/// stdout (`patch_on_stdout`), what was written is said on stderr.
fn write_diff_output(
    job: &DiffJob<'_>,
    to: Format,
    output: Option<PathBuf>,
    out: &[u8],
    patch_on_stdout: bool,
) -> Result<(), String> {
    match (to, output) {
        (Format::Md, None) => write_stdout(out),
        (Format::Md | Format::Docx, Some(path)) => {
            ensure_writable(&path, job.force)?;
            std::fs::write(&path, out).map_err(|e| format!("writing {}: {e}", path.display()))?;
            say(
                patch_on_stdout,
                format_args!("wrote {} ({} bytes)", path.display(), out.len()),
            );
            Ok(())
        }
        (Format::Pdf | Format::Png, output) => run_convert(&ConvertJob {
            file: job.old,
            bytes: Some(out),
            output: output.as_deref(),
            force: job.force,
            compress: false,
            font_report: None,
            revisions: job.revisions,
            pdf: to == Format::Pdf,
            png: to == Format::Png,
            dpi: 96.0,
            report: None,
            pages: None,
            fail_on_substitution: false,
            page: job.page,
            status_to_stderr: patch_on_stdout,
        })
        .map_err(|f| f.message),
        (Format::Docx, None) => unreachable!("a Word output always has a path"),
    }
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
    match jubarte::markdown::page_texts(docx, track_changes.into(), revisions) {
        Ok(pages) => {
            let pages: Vec<&str> = pages.iter().map(String::as_str).collect();
            jubarte::markdown::paginate(markdown, &pages)
        }
        Err(e) => {
            eprintln!("warning: no page markers: {e}");
            markdown.to_string()
        }
    }
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
    let to = markdown
        .to
        .or_else(|| (job.pdf || job.png).then_some(Format::Pdf))
        .or_else(|| job.output.and_then(Format::of_path))
        .unwrap_or(match from {
            Format::Md => Format::Docx,
            _ if legacy => Format::Docx,
            _ => Format::Pdf,
        });
    if job.pages.is_some() && to != Format::Png && !job.png {
        return Err("--pages selects PNG pages; add --png".into());
    }
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
            (job.page.move_comments, "--move-comments"),
            (job.page.changed_only, "--changed-only"),
        ] {
            if given {
                return Err(format!("{flag} applies to PDF or PNG output only").into());
            }
        }
    }
    let converted = legacy.then_some(bytes.as_slice());
    let pdf_job = |bytes: Option<&[u8]>, to: Format| {
        let mut rendered = ConvertJob { bytes, ..*job };
        if to == Format::Png && !job.png {
            rendered.png = true;
        }
        run_convert(&rendered)
    };
    match (from, to) {
        (Format::Docx, Format::Pdf | Format::Png) => match markdown.track_changes {
            TrackChanges::All => pdf_job(converted, to),
            // The pages of the document with every change accepted or rejected.
            choice => {
                let resolve = if choice == TrackChanges::Accept {
                    jubarte::document_comparer::accept_revisions
                } else {
                    jubarte::document_comparer::reject_revisions
                };
                let resolved = resolve(&bytes).map_err(|e| {
                    refusal("convert", &e).unwrap_or_else(|| format!("convert failed: {e:?}"))
                })?;
                pdf_job(Some(&resolved), to)
            }
        },
        (Format::Docx, Format::Md) => {
            let read = jubarte::markdown::docx_to_markdown(
                &bytes,
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
                    &bytes,
                    &read.markdown,
                    markdown.track_changes,
                    job.revisions,
                )
            };
            match job.output {
                Some(output) => {
                    ensure_writable(output, job.force)?;
                    std::fs::write(output, &text)
                        .map_err(|e| format!("writing {}: {e}", output.display()))?;
                    outln!("wrote {} ({} bytes)", output.display(), text.len());
                    Ok(())
                }
                None => write_stdout(&text).map_err(ConvertFailure::from),
            }
        }
        (Format::Docx, Format::Docx) => {
            let resolve = match markdown.track_changes {
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
            };
            let output = match job.output {
                Some(output) => output.to_path_buf(),
                None if legacy => job.file.with_extension("docx"),
                None => return Err("--output is required to write Word from Word".into()),
            };
            if let Some(report) = job.report {
                let others = [(output.as_path(), "Word output"), (job.file, "input")];
                check_side_file(report, "--report", others, job.force)?;
            }
            ensure_writable(&output, job.force)?;
            let mut out = match resolve {
                Some(resolve) => resolve(&bytes).map_err(|e| {
                    refusal("convert", &e).unwrap_or_else(|| format!("convert failed: {e:?}"))
                })?,
                None => bytes,
            };
            if update_fields {
                out = refresh_fields(&out, job.report)?;
            }
            std::fs::write(&output, &out)
                .map_err(|e| format!("writing {}: {e}", output.display()))?;
            outln!("wrote {} ({} bytes)", output.display(), out.len());
            Ok(())
        }
        (Format::Md, Format::Md) => {
            let text = markdown_text(job.file, bytes)?;
            let out = if markdown.no_critic {
                text
            } else {
                jubarte::markdown::resolve_critic(&text, markdown.track_changes.into())
            };
            match job.output {
                Some(output) => {
                    ensure_writable(output, job.force)?;
                    std::fs::write(output, &out)
                        .map_err(|e| format!("writing {}: {e}", output.display()).into())
                }
                None => write_stdout(&out).map_err(ConvertFailure::from),
            }
        }
        (Format::Md, Format::Docx | Format::Pdf | Format::Png) => {
            let text = markdown_text(job.file, bytes)?;
            let reference = markdown
                .reference_doc
                .as_deref()
                .map(read_document)
                .transpose()?;
            let loader = image_loader(resource_dir(job.file, markdown.resource_path.as_deref()));
            let options = jubarte::markdown::DocxOptions {
                reference: reference.as_deref(),
                critic: !markdown.no_critic,
                track_changes: markdown.track_changes.into(),
                author: markdown.author.clone(),
                date: markdown.date.clone(),
                images: Some(&loader),
                page: markdown.page.into(),
            };
            let written = jubarte::markdown::markdown_to_docx(&text, &options)
                .map_err(|e| format!("convert failed: {e}"))?;
            for warning in &written.warnings {
                eprintln!("warning: {warning}");
            }
            if to != Format::Docx {
                return pdf_job(Some(&written.docx), to);
            }
            let output = job
                .output
                .map_or_else(|| job.file.with_extension("docx"), Path::to_path_buf);
            ensure_writable(&output, job.force)?;
            std::fs::write(&output, &written.docx)
                .map_err(|e| format!("writing {}: {e}", output.display()))?;
            outln!("wrote {} ({} bytes)", output.display(), written.docx.len());
            Ok(())
        }
        (Format::Pdf | Format::Png, _) => {
            Err(format!("{}: PDF and PNG are not inputs", job.file.display()).into())
        }
    }
}

/// `jubarte debug`: print the report for one package, or two compared.
fn run_debug(
    files: &[PathBuf],
    list: bool,
    checks: Vec<DebugCheck>,
    part: Option<String>,
    grep: Option<String>,
    limit: usize,
    context: usize,
) -> Result<(), String> {
    let read = |p: &PathBuf| std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()));
    let a = read(&files[0])?;
    let b = files.get(1).map(read).transpose()?;
    let mut opts = jubarte::debug::Options {
        part,
        grep,
        limit,
        context,
        ..Default::default()
    };
    if !checks.is_empty() {
        opts.checks = checks.into_iter().map(Into::into).collect();
    }
    let out = if list {
        jubarte::debug::list(&a, b.as_deref(), &opts)?
    } else {
        jubarte::debug::report(&a, b.as_deref(), &opts)?
    };
    write_stdout(&out)
}

/// One `jubarte validate` run.
struct ValidateJob<'a> {
    file: &'a Path,
    json: bool,
    repair: Option<&'a Path>,
    original: Option<&'a Path>,
    author: Option<&'a str>,
    force: bool,
}

/// `jubarte validate`: `Ok(true)` when nothing was found, `Ok(false)` when
/// findings were printed, `Err` when a file could not be read or written.
fn run_validate(job: &ValidateJob<'_>) -> Result<bool, String> {
    use jubarte::validate::{Finding, audit_tracked, repair, validate};
    let read = |p: &Path| std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()));
    let docx = read(job.file)?;
    let mut findings: Vec<Finding> = Vec::new();
    match job.repair {
        Some(out) => {
            if out.exists() && !job.force {
                return Err(format!(
                    "output '{}' already exists (use --force to overwrite)",
                    out.display()
                ));
            }
            let fixed = repair(&docx).map_err(|e| e.to_string())?;
            std::fs::write(out, &fixed.docx).map_err(|e| format!("{}: {e}", out.display()))?;
            if !job.json {
                outln!(
                    "repaired {} finding(s) into {}",
                    fixed.repaired.len(),
                    out.display()
                );
            }
            findings.extend(fixed.remaining);
        }
        None => findings.extend(validate(&docx).map_err(|e| e.to_string())?),
    }
    if let (Some(original), Some(author)) = (job.original, job.author) {
        let before = read(original)?;
        findings.extend(audit_tracked(&before, &docx, author).map_err(|e| e.to_string())?);
    }
    for f in &findings {
        if job.json {
            outln!("{}", serde_json::to_string(f).map_err(|e| e.to_string())?);
        } else {
            let star = if f.word_fatal { '*' } else { ' ' };
            outln!("{star} {}\t{}#{}\t{}", f.code, f.part, f.path, f.message);
        }
    }
    if !job.json {
        if findings.is_empty() {
            outln!("no findings");
        } else {
            let fatal = findings.iter().filter(|f| f.word_fatal).count();
            outln!("{} finding(s), {fatal} Word-fatal", findings.len());
        }
    }
    Ok(findings.is_empty())
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
    let cli = Cli::try_parse_from(std::env::args_os()).unwrap_or_else(|error| error.exit());
    match cli.command {
        Some(Command::Compare(compare)) => {
            return exit_code(run(&resolve_compare(compare, ReadArgs::default(), true)));
        }
        Some(Command::Changes { file, json }) => {
            return exit_code(run_changes(&file, json));
        }
        Some(Command::Comments {
            file,
            json,
            author,
            latest,
        }) => {
            return exit_code(run_comments(&file, json, author.as_deref(), latest));
        }
        Some(Command::Accept {
            file,
            output,
            force,
            selection,
        }) => {
            return exit_code(run_resolution(
                &file,
                &output,
                force,
                &selection.filter(),
                jubarte::changes::accept_changes,
                "accept",
            ));
        }
        Some(Command::Reject {
            file,
            output,
            force,
            selection,
        }) => {
            return exit_code(run_resolution(
                &file,
                &output,
                force,
                &selection.filter(),
                jubarte::changes::reject_changes,
                "reject",
            ));
        }
        Some(Command::Convert {
            file,
            output,
            force,
            pdf,
            png,
            dpi,
            report,
            compress,
            font_report,
            revisions,
            revision_palette,
            markdown,
            pages: page_spec,
            fail_on_substitution,
            update_fields,
            timeout,
            page,
        }) => {
            if let Some(limit) = timeout {
                arm_timeout(limit);
            }
            let style = match revision_style(revisions, revision_palette.as_deref()) {
                Ok(style) => style,
                Err(e) => return exit_code(Err(e)),
            };
            let selected = match page_spec.as_deref().map(parse_pages).transpose() {
                Ok(selected) => selected,
                Err(e) => return exit_code(Err(e)),
            };
            let job = ConvertJob {
                file: &file,
                bytes: None,
                output: output.as_deref(),
                force,
                compress,
                font_report: font_report.as_deref(),
                revisions: style,
                pdf,
                png,
                dpi,
                report: report.as_deref(),
                pages: selected.as_deref(),
                fail_on_substitution,
                page,
                status_to_stderr: false,
            };
            return convert_exit_code(run_convert_any(&job, &markdown, update_fields));
        }
        Some(Command::Diff {
            old,
            new,
            output,
            to,
            from,
            format,
            columns,
            context,
            accept_changes,
            full_lines,
            force,
            author,
            date,
            mode,
            detail_threshold,
            reference_doc,
            critic,
            resource_path,
            revisions,
            revision_palette,
            page,
        }) => {
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
                return exit_code(run_text_diff(
                    &old,
                    &new,
                    output.as_deref(),
                    from,
                    force,
                    &options,
                ));
            }
            let style = match revision_style(revisions, revision_palette.as_deref()) {
                Ok(style) => style,
                Err(e) => return exit_code(Err(e)),
            };
            return exit_code(run_diff(&DiffJob {
                old: &old,
                new: &new,
                output: output.as_deref(),
                to,
                from,
                force,
                settings: comparer_settings(
                    &author.unwrap_or_else(default_author),
                    &date.unwrap_or_else(jubarte::convert::utc_now_iso8601),
                    detail_threshold,
                    mode == CompareMode::Powertools,
                    false,
                ),
                reference: reference_doc.as_deref(),
                critic,
                resource_path: resource_path.as_deref(),
                revisions: style,
                page,
                patch: (format == PatchFormat::Patch).then_some(columns),
            }));
        }
        Some(Command::Inspect { file, json, tables }) => {
            return exit_code(if tables {
                run_inspect_tables(&file)
            } else {
                run_inspect(&file, json)
            });
        }
        Some(Command::Read { file, args }) => return exit_code(run_text(&file, &args)),
        Some(Command::Edit {
            file,
            plan,
            options,
            dry_run,
            pdf,
            png,
            dpi,
            revisions,
            revision_palette,
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
                operations: &operations,
                options: &options,
                dry_run,
                pdf,
                png,
                dpi,
                revisions: style,
            };
            return edit_exit(run_flags(&flags));
        }
        Some(Command::Add {
            file,
            options,
            operations,
            ..
        }) => {
            let flags = EditFlags {
                verb: jubarte::edit::flags::Verb::Add,
                file: &file,
                plan: None,
                operations: &operations,
                options: &options,
                dry_run: false,
                pdf: false,
                png: false,
                dpi: 96.0,
                revisions: jubarte::convert::RevisionStyle::Conventional,
            };
            return edit_exit(run_flags(&flags));
        }
        Some(Command::Capabilities { .. }) => {
            outln!("{}", jubarte::capabilities::capabilities_json("cli"));
            return ExitCode::SUCCESS;
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
        Some(Command::DiffRender {
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
                Err(e) => {
                    eprintln!("error: {e}");
                    ExitCode::FAILURE
                }
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
        }) => {
            return exit_code(run_debug(&files, list, checks, part, grep, limit, context));
        }
        Some(Command::Append {
            files,
            output,
            section_break,
            keep_sections,
            carry_comments,
            force,
            quiet,
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
            return exit_code(run_append(&files, &output, &options, force, quiet));
        }
        Some(Command::Validate {
            file,
            json,
            repair,
            original,
            author,
            force,
        }) => {
            let job = ValidateJob {
                file: &file,
                json,
                repair: repair.as_deref(),
                original: original.as_deref(),
                author: author.as_deref(),
                force,
            };
            return match run_validate(&job) {
                Ok(true) => ExitCode::SUCCESS,
                Ok(false) => ExitCode::from(2),
                Err(e) => {
                    eprintln!("error: {e}");
                    ExitCode::FAILURE
                }
            };
        }
        Some(Command::Scrub {
            file,
            output,
            force,
            scrub,
        }) => {
            return exit_code(run_scrub(&file, &output, force, &scrub.options()));
        }
        Some(Command::Audit {
            file,
            json,
            rules,
            strict,
        }) => {
            return match run_audit(&file, json, &rules, strict) {
                Ok(true) => ExitCode::from(2),
                result => exit_code(result.map(|_| ())),
            };
        }
        None => {}
    }
    if cli.compare.modified.is_none() && cli.compare.modified_pos.is_none() {
        let file = cli
            .compare
            .original
            .or(cli.compare.original_pos)
            .expect("clap requires ORIGINAL");
        return exit_code(run_text(&file, &cli.read));
    }
    exit_code(run(&resolve_compare(cli.compare, cli.read, false)))
}

/// `jubarte append`: fold the documents left, `append(append(A, B), C)`.
fn run_append(
    files: &[PathBuf],
    output: &Path,
    options: &jubarte::append::AppendOptions,
    force: bool,
    quiet: bool,
) -> Result<(), String> {
    ensure_writable(output, force)?;
    let mut paths = files.iter();
    let first = paths.next().ok_or("append needs two documents")?;
    let mut out = read_document(first)?;
    for path in paths {
        let next = read_document(path)?;
        let appended = jubarte::append::append_documents(&out, &next, options)
            .map_err(|e| format!("appending {}: {e}", path.display()))?;
        for warning in &appended.warnings {
            eprintln!("warning: {}: {warning}", path.display());
        }
        out = appended.docx;
    }
    std::fs::write(output, &out).map_err(|e| format!("writing {}: {e}", output.display()))?;
    if !quiet {
        outln!("wrote {} ({} bytes)", output.display(), out.len());
    }
    Ok(())
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

/// `convert --update-fields`: the package with its field results refreshed
/// from jubarte's layout, one line per field written on stdout and
/// `{page_count, fields}` in `report` when given.
fn refresh_fields(bytes: &[u8], report: Option<&Path>) -> Result<Vec<u8>, String> {
    let updated = jubarte::fields::update_fields(bytes).map_err(|e| e.to_string())?;
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
    if let Some(report) = report {
        let json = serde_json::json!({
            "page_count": updated.page_count,
            "fields": updated.fields,
        });
        std::fs::write(report, json.to_string())
            .map_err(|e| format!("writing {}: {e}", report.display()))?;
    }
    Ok(updated.docx)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use jubarte::convert::{MarkLines, RevisionStyle};

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
            quiet,
        }) = cli.command
        else {
            panic!("not append");
        };
        assert_eq!(files.len(), 2);
        assert_eq!(output, PathBuf::from("out.docx"));
        assert_eq!(section_break, SectionBreakArg::NextPage);
        assert!(!keep_sections && !carry_comments && !force && !quiet);
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
        let Some(Command::Convert { revisions, .. }) = cli.command else {
            panic!("expected convert");
        };
        assert_eq!(revisions, Revisions::Word);
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
            Some(Command::Compare(compare)) => resolve_compare(compare, ReadArgs::default(), true),
            None => resolve_compare(cli.compare, cli.read, false),
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
        let j = job_of(&["jubarte", "compare", "a.docx", "b.docx"]);
        assert_eq!(j.output, Some(PathBuf::from("a_v_b.docx")));
        assert_eq!(j.author, "Redline");
        assert_eq!(j.date, "1970-01-01T00:00:00Z");
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
        assert_eq!(j.output, Some(PathBuf::from("x_v_y.docx")));
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
        let job = resolve_compare(cli.compare, cli.read, false);
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
        let job = resolve_compare(cli.compare, cli.read, false);
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
            }) => {
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
            }) => {
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
            "body:rev:1",
            "--id",
            "header1:rev:2",
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
                ids: Some(vec!["body:rev:1".into(), "header1:rev:2".into()]),
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
                revisions,
                revision_palette,
                pdf,
                png,
                dpi,
                report,
                ..
            }) => {
                assert_eq!(revisions, Revisions::Conventional);
                assert!(revision_palette.is_none());
                assert_eq!(file, PathBuf::from("in.docx"));
                assert_eq!(output.as_deref(), Some(Path::new("out.pdf")));
                assert_eq!(font_report.as_deref(), Some(Path::new("out.json")));
                assert!(!compress && !force);
                assert!(!pdf && !png && report.is_none());
                assert_eq!(dpi, 96.0);
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
            compress: false,
            font_report: Some(&same),
            revisions: RevisionStyle::Word,
            pdf: false,
            png: false,
            dpi: 96.0,
            report: None,
            pages: None,
            fail_on_substitution: false,
            page: PageOptions::default(),
            status_to_stderr: false,
        })
        .expect_err("report over the PDF must be refused");
        assert!(
            err.message.contains("same file as the PDF output"),
            "{}",
            err.message
        );
        assert!(!pdf.exists(), "nothing is written when the paths collide");
        let err = run_convert(&ConvertJob {
            file: &docx,
            bytes: None,
            output: Some(&pdf),
            force: false,
            compress: false,
            font_report: Some(&docx),
            revisions: RevisionStyle::Word,
            pdf: false,
            png: false,
            dpi: 96.0,
            report: None,
            pages: None,
            fail_on_substitution: false,
            page: PageOptions::default(),
            status_to_stderr: false,
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
        let err = run(&job_of(&args)).expect_err("a .doc must be refused");
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
            compress: false,
            font_report: Some(&report),
            revisions: RevisionStyle::Word,
            pdf: false,
            png: false,
            dpi: 96.0,
            report: None,
            pages: None,
            fail_on_substitution: true,
            page: PageOptions::default(),
            status_to_stderr: false,
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
            compress: false,
            font_report: Some(&report),
            revisions: RevisionStyle::Word,
            pdf: false,
            png: false,
            dpi: 96.0,
            report: None,
            pages: None,
            fail_on_substitution: false,
            page: PageOptions::default(),
            status_to_stderr: false,
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
            let Some(Command::Convert {
                revisions,
                revision_palette,
                ..
            }) = cli.command
            else {
                panic!("convert")
            };
            assert_eq!(revisions, Revisions::Conventional);
            assert_eq!(revision_palette, None);
            assert_eq!(
                revision_style(revisions, None),
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
            let Some(Command::Convert {
                revisions,
                revision_palette,
                ..
            }) = cli.command
            else {
                panic!("convert")
            };
            assert_eq!(revisions, Revisions::Custom);
            assert_eq!(revision_palette.as_deref(), Some(spec));
            assert_eq!(
                revision_style(revisions, revision_palette.as_deref()),
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
        let cli = Cli::try_parse_from(["jubarte", "convert", "a.docx", "--png", "--pages", "2-3"])
            .expect("parse");
        let Some(Command::Convert { pages, png, .. }) = cli.command else {
            panic!("expected convert");
        };
        assert_eq!(pages.as_deref(), Some("2-3"));
        assert!(png);
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
        let Some(Command::DiffRender {
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
        let Some(Command::DiffRender {
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
            compress: false,
            font_report: None,
            revisions: RevisionStyle::Conventional,
            pdf: false,
            png: true,
            dpi: 20.0,
            report: None,
            pages,
            fail_on_substitution: false,
            page: PageOptions::default(),
            status_to_stderr: false,
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
        let err = run_convert(&ConvertJob {
            png: false,
            ..convert_job(&docx, &out, Some(&[0]))
        })
        .expect_err("--pages without PNG output");
        assert!(err.message.contains("--pages"), "{}", err.message);
        let err = run_convert(&convert_job(&docx, &out, Some(&[5]))).expect_err("page 6 of 3");
        assert!(
            err.message
                .contains("page 6 is out of range: the output has 3 pages"),
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
        assert!(err.contains("already exists"), "{err}");
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
        assert!(err.contains("missing.docx"), "{err}");
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
        let written = refresh_fields(&tiny_docx_bytes("Calibri"), Some(&report)).expect("update");
        assert_eq!(
            jubarte::inspect::paragraphs(&written).unwrap()[0].text,
            "HELLO"
        );
        let json: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&report).unwrap()).unwrap();
        assert_eq!(json["page_count"], 1);
        let err = refresh_fields(
            &tiny_docx_bytes("Calibri"),
            Some(&dir.path().join("no/such/dir.json")),
        )
        .unwrap_err();
        assert!(err.contains("writing"), "{err}");
        assert!(refresh_fields(b"not a zip", None).is_err());
    }
}
