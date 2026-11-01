// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Python bindings for the canonical **jubarte-redlines** engine.
//!
//! Built with **PyO3** + **maturin** as the `jubarte_redlines._native`
//! extension module; the public Python surface (typed `Document`, edit plans,
//! the `python -m jubarte_redlines` CLI) lives in `python/jubarte_redlines/`.
//!
//! Every entry point detaches from the interpreter for the whole pure-Rust
//! compute, so long operations don't block other Python threads. Structured
//! results cross the boundary as JSON strings produced by the engine's own
//! serializers; the Python layer turns them into frozen dataclasses and never
//! interprets OOXML itself.

use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyBytes, PyInt};

create_exception!(
    jubarte_redlines,
    JubarteError,
    PyException,
    "Raised when the jubarte-redlines engine cannot process a document."
);

/// An engine error as `JubarteError`; an admission refusal reads code
/// first (`LEGACY_DOC: …`, `INPUT_LIMIT: …`), whatever wrapped it.
fn err(e: impl std::fmt::Display) -> PyErr {
    let message = e.to_string();
    JubarteError::new_err(jubarte::admission::code_first(&message).unwrap_or(message))
}

/// A selection refusal as `JubarteError` with `USAGE: ` before its
/// message, the category the CLI exits 2 for; the hosts map that prefix
/// to the binary's usage exit and code.
fn usage_err(message: String) -> PyErr {
    JubarteError::new_err(format!("USAGE: {message}"))
}

/// Where a PDF's comments go and which pages it keeps: the binary's
/// `--move-comments` and `--changed-only`.
#[derive(Clone, Copy, Default)]
struct PageChoice {
    move_comments: bool,
    changed_only: bool,
}

fn pdf_options(
    compress: bool,
    revisions: &str,
    revision_palette: Option<&str>,
    page: PageChoice,
) -> PyResult<jubarte::convert::PdfOptions> {
    let revisions = jubarte::convert::RevisionStyle::from_choice(revisions, revision_palette)
        .map_err(JubarteError::new_err)?;
    Ok(jubarte::convert::PdfOptions {
        compress,
        revisions,
        comments: if page.move_comments {
            jubarte::convert::CommentPlacement::End
        } else {
            jubarte::convert::CommentPlacement::Margin
        },
        comment_notes: false,
        changed_only: page.changed_only,
    })
}

/// `WmlComparerSettings::default()` with `input_limits` (a dict such as
/// `{"max_part_bytes": 67108864}`) laid over its compare budget. Unknown
/// keys are refused.
fn settings_with_limits(
    input_limits: Option<std::collections::HashMap<String, u64>>,
) -> PyResult<jubarte::comparer::WmlComparerSettings> {
    let settings = jubarte::comparer::WmlComparerSettings::default();
    let Some(limits) = input_limits else {
        return Ok(settings);
    };
    let json = serde_json::to_string(&limits).map_err(err)?;
    let overrides =
        jubarte::admission::InputLimitOverrides::from_json(&json).map_err(JubarteError::new_err)?;
    let base = settings.input_limits;
    Ok(settings.with_input_limits(overrides.apply(base)))
}

/// Compare two DOCX packages (bytes) → redline DOCX bytes (`w:ins`/`w:del`).
///
/// Mirrors `jubarte::document_comparer::compare_documents`; `date` (ISO-8601
/// `w:date` stamp) defaults to the engine's fixed epoch for deterministic
/// output. `input_limits` overrides the admission budget key by key
/// (`max_compressed_bytes`, `max_entries`, `max_part_bytes`,
/// `max_uncompressed_bytes`, `max_xml_depth`, `max_xml_elements`); a package
/// past it raises
/// `JubarteError` with `INPUT_LIMIT`.
#[pyfunction]
#[pyo3(signature = (original, modified, author = "Redline", date = None, *, input_limits = None))]
fn compare_documents(
    py: Python<'_>,
    original: &[u8],
    modified: &[u8],
    author: &str,
    date: Option<&str>,
    input_limits: Option<std::collections::HashMap<String, u64>>,
) -> PyResult<Py<PyBytes>> {
    let settings = settings_with_limits(input_limits)?
        .with_author(author)
        .with_date(date.unwrap_or(jubarte::document_comparer::DEFAULT_DATE));
    let out = py
        .detach(|| {
            jubarte::document_comparer::compare_documents_with_settings(
                original, modified, &settings,
            )
        })
        .map_err(err)?;
    Ok(PyBytes::new(py, &out).unbind())
}

/// Accept every tracked revision (package-wide) → clean DOCX bytes.
/// The author and timestamp a Markdown document gives the changes and
/// comments that name none: its front matter, else the reference `.docx`'s
/// creator and creation time; `None` for each neither tells (the CLI then
/// takes `Redline` and now).
#[pyfunction]
#[pyo3(signature = (text, reference = None))]
fn markdown_stamp(text: &str, reference: Option<&[u8]>) -> (Option<String>, Option<String>) {
    jubarte::markdown::markdown_stamp(text, reference)
}

#[pyfunction]
fn accept_revisions(py: Python<'_>, docx: &[u8]) -> PyResult<Py<PyBytes>> {
    let out = py
        .detach(|| jubarte::document_comparer::accept_revisions(docx))
        .map_err(err)?;
    Ok(PyBytes::new(py, &out).unbind())
}

/// Reject every tracked revision (package-wide) → base DOCX bytes.
#[pyfunction]
fn reject_revisions(py: Python<'_>, docx: &[u8]) -> PyResult<Py<PyBytes>> {
    let out = py
        .detach(|| jubarte::document_comparer::reject_revisions(docx))
        .map_err(err)?;
    Ok(PyBytes::new(py, &out).unbind())
}

/// List the tracked changes as a JSON array string, one object per
/// logical change, each with the id `accept_changes` / `reject_changes`
/// select by (the same objects as `jubarte changes --json`): a change Word wrote as several marks is one
/// object with `members`. `authors` keeps those authors' changes (exact
/// names), as `jubarte changes --author`.
#[pyfunction]
#[pyo3(signature = (docx, authors = None))]
fn list_changes_json(
    py: Python<'_>,
    docx: &[u8],
    authors: Option<Vec<String>>,
) -> PyResult<String> {
    py.detach(|| {
        let changes = jubarte::changes::list_logical_changes(docx).map_err(|e| e.to_string())?;
        let changes = jubarte::changes::by_authors(changes, &authors.unwrap_or_default());
        serde_json::to_string(&changes).map_err(|e| e.to_string())
    })
    .map_err(|e: String| err(e))
}

/// Every comment as a JSON array (the objects `jubarte comments --json`
/// prints): thread (`parent`, `done`) and anchored text with its
/// surroundings. `author` keeps one author's comments; `latest` keeps the
/// newest comment of each thread.
#[pyfunction]
#[pyo3(signature = (docx, author = None, latest = false))]
fn list_comments_json(
    py: Python<'_>,
    docx: &[u8],
    author: Option<String>,
    latest: bool,
) -> PyResult<String> {
    py.detach(|| {
        let comments = jubarte::comments::list_comments(docx).map_err(|e| e.to_string())?;
        let comments = jubarte::comments::select_comments(comments, author.as_deref(), latest);
        serde_json::to_string(&comments).map_err(|e| e.to_string())
    })
    .map_err(|e: String| err(e))
}

/// The listing `jubarte changes` prints: a line per tracked change with its
/// short id (`r12`), kind, target, author, paragraph (`p3`) and text, then
/// `N change(s)`. `authors` filters as in [`list_changes_json`].
#[pyfunction]
#[pyo3(signature = (docx, authors = None))]
fn changes_text(py: Python<'_>, docx: &[u8], authors: Option<Vec<String>>) -> PyResult<String> {
    py.detach(|| {
        jubarte::changes::list_logical_changes(docx)
            .map(|changes| jubarte::changes::by_authors(changes, &authors.unwrap_or_default()))
            .map(|changes| jubarte::changes::changes_text(&changes))
            .map_err(|e| e.to_string())
    })
    .map_err(|e: String| err(e))
}

/// The listing `jubarte comments` prints: a line per comment with its short
/// id (`c5`), paragraph, author, text and anchor, then `N comment(s)`.
/// `author` and `latest` filter as in [`list_comments_json`].
#[pyfunction]
#[pyo3(signature = (docx, author = None, latest = false))]
fn comments_text(
    py: Python<'_>,
    docx: &[u8],
    author: Option<String>,
    latest: bool,
) -> PyResult<String> {
    py.detach(|| {
        let comments = jubarte::comments::list_comments(docx).map_err(|e| e.to_string())?;
        let comments = jubarte::comments::select_comments(comments, author.as_deref(), latest);
        Ok(jubarte::comments::comments_text(&comments))
    })
    .map_err(|e: String| err(e))
}

/// The grids `jubarte read --tables` prints: a `table N: RxC …` line per
/// body table, then a line per row of `ids=text` cells; `no tables` when
/// there are none. `track_changes` `reject` reads the rejected text.
#[pyfunction]
#[pyo3(signature = (docx, *, track_changes = None))]
fn tables_text(py: Python<'_>, docx: &[u8], track_changes: Option<&str>) -> PyResult<String> {
    let options = jubarte::markdown::ReadOptions {
        track_changes: track_changes_of(track_changes)?,
        ..jubarte::markdown::ReadOptions::default()
    };
    py.detach(|| jubarte::markdown::read_tables(docx, &options).map(|view| view.markdown))
        .map_err(err)
}

/// What `jubarte read --tables` prints, and the view's warnings: the
/// grids ([`tables_text`]), or with `paragraphs` (`t0`, `p11`, `p3-p20`)
/// the grids of the tables the view's selection reaches (a `pN` inside a
/// table, that table), each with the tables nested in it, then an empty
/// line and the view [`read_view`] prints of that selection with the same
/// options. A pick that reaches no table raises `JubarteError` with
/// `USAGE: ` before its message, as the CLI exits 2.
#[pyfunction]
#[pyo3(signature = (
    docx,
    *,
    track_changes = None,
    comments = true,
    dates = false,
    page_markers = true,
    paragraphs = None,
    source = None,
))]
#[expect(
    clippy::too_many_arguments,
    reason = "a Python signature of keyword arguments with defaults, as the binary's flags"
)]
fn tables_view(
    py: Python<'_>,
    docx: &[u8],
    track_changes: Option<&str>,
    comments: bool,
    dates: bool,
    page_markers: bool,
    paragraphs: Option<&str>,
    source: Option<String>,
) -> PyResult<(String, Vec<String>)> {
    let options = jubarte::markdown::ReadOptions {
        track_changes: track_changes_of(track_changes)?,
        comments,
        dates,
        page_markers,
        select: jubarte::markdown::Select::from_flags(paragraphs, None, None, false, None)
            .map_err(usage_err)?,
        source,
    };
    let view = py
        .detach(|| jubarte::markdown::read_tables(docx, &options))
        .map_err(|e| match e {
            jubarte::markdown::TablesError::Usage(message) => {
                JubarteError::new_err(format!("USAGE: {message}"))
            }
            jubarte::markdown::TablesError::Read(e) => err(e),
        })?;
    Ok((view.markdown, view.warnings))
}

/// `track_changes` as the view reads it: `all` (the default), `accept` or
/// `reject`, with their `accepted` and `rejected` spellings.
fn track_changes_of(track_changes: Option<&str>) -> PyResult<jubarte::markdown::TrackChanges> {
    match track_changes {
        None => Ok(jubarte::markdown::TrackChanges::All),
        Some(choice) => jubarte::markdown::TrackChanges::parse(choice)
            .ok_or_else(|| err("track_changes must be all, accept or reject")),
    }
}

/// The snapshot `jubarte read --json` prints, with its `changes` and
/// `comments`. `paragraphs`, `head`, `tail`, `changed` and `by` select as in
/// [`read_view`]; the snapshot keeps the selected paragraphs and their
/// changes and comments. `track_changes` `accept` or `reject` settles the
/// text and names the choice in `track_changes`. `validity_file`, the path
/// as the command line spells it, adds `summary.input_validity`: the
/// input's validity findings counted, as the binary reports them.
#[pyfunction]
#[pyo3(signature = (
    docx,
    *,
    track_changes = None,
    paragraphs = None,
    head = None,
    tail = None,
    changed = false,
    by = None,
    source = None,
    validity_file = None,
))]
#[expect(
    clippy::too_many_arguments,
    reason = "a Python signature of keyword arguments with defaults, as the binary's flags"
)]
fn read_json(
    py: Python<'_>,
    docx: &[u8],
    track_changes: Option<&str>,
    paragraphs: Option<&str>,
    head: Option<usize>,
    tail: Option<usize>,
    changed: bool,
    by: Option<&str>,
    source: Option<String>,
    validity_file: Option<&str>,
) -> PyResult<String> {
    let track_changes = track_changes_of(track_changes)?;
    let select = jubarte::markdown::Select::from_flags(paragraphs, head, tail, changed, by)
        .map_err(usage_err)?;
    let options = jubarte::markdown::ReadOptions {
        track_changes,
        select,
        source,
        ..jubarte::markdown::ReadOptions::default()
    };
    py.detach(|| {
        let mut snapshot =
            jubarte::markdown::read_snapshot(docx, &options).map_err(|e| e.to_string())?;
        snapshot.summary.input_validity =
            validity_file.map(|file| jubarte::validate::input_validity_of(docx, file));
        serde_json::to_string(&snapshot).map_err(|e| e.to_string())
    })
    .map_err(|e: String| err(e))
}

/// The working-document JSON `jubarte read --for-edit` prints: the
/// selection (`paragraphs`, `head`, `tail`, `find`/`regex`, `changed`/`by`)
/// frozen on the original source, exact working text and per-operation
/// readiness of the basis `accept_change`/`reject_change`,
/// `existing_revisions` and `track` settle it to. `limit`, `max_chars`,
/// `all` and `cursor` page the rows as the flags do; `context` adds
/// read-only original neighbours. `validity_file`, the path as the command
/// line spells it, adds `input_validity`. A `USAGE` error raises
/// `JubarteError` with `USAGE: ` before its message, as the CLI exits 2;
/// the other refusals (an unknown `--accept-change` id, an unsettleable
/// basis) read `CODE: `, as the CLI exits 3.
#[pyfunction]
#[pyo3(signature = (
    docx,
    *,
    paragraphs = None,
    head = None,
    tail = None,
    find = None,
    regex = None,
    ignore_case = false,
    changed = false,
    by = None,
    context = None,
    limit = None,
    max_chars = None,
    all = false,
    cursor = None,
    accept_change = None,
    reject_change = None,
    existing_revisions = None,
    track = None,
    validity_file = None,
))]
#[expect(
    clippy::too_many_arguments,
    reason = "a Python signature of keyword arguments with defaults, as the binary's flags"
)]
fn for_edit_json(
    py: Python<'_>,
    docx: &[u8],
    paragraphs: Option<String>,
    head: Option<usize>,
    tail: Option<usize>,
    find: Option<String>,
    regex: Option<String>,
    ignore_case: bool,
    changed: bool,
    by: Option<String>,
    context: Option<usize>,
    limit: Option<usize>,
    max_chars: Option<usize>,
    all: bool,
    cursor: Option<String>,
    accept_change: Option<Vec<String>>,
    reject_change: Option<Vec<String>>,
    existing_revisions: Option<&str>,
    track: Option<&str>,
    validity_file: Option<String>,
) -> PyResult<String> {
    let existing_revisions = existing_revisions
        .map(|mode| {
            jubarte::cli::ExistingArg::from_choice(mode).ok_or_else(|| {
                JubarteError::new_err(format!(
                    "USAGE: invalid value '{mode}' for '--existing-revisions <MODE>'"
                ))
            })
        })
        .transpose()?;
    let track = track
        .map(|choice| {
            jubarte::cli::TrackArg::from_choice(choice).ok_or_else(|| {
                JubarteError::new_err(format!(
                    "USAGE: invalid value '{choice}' for '--track <CHOICE>'"
                ))
            })
        })
        .transpose()?;
    let args = jubarte::cli::ReadArgs {
        paragraphs: paragraphs.into_iter().collect(),
        head,
        tail,
        find,
        regex,
        ignore_case,
        changed,
        by,
        context,
        limit,
        max_chars,
        all,
        cursor,
        accept_change: accept_change.unwrap_or_default(),
        reject_change: reject_change.unwrap_or_default(),
        existing_revisions,
        track,
        ..Default::default()
    };
    py.detach(|| {
        jubarte::edit::for_edit::for_edit_json(
            docx,
            &args.for_edit_request(),
            validity_file
                .as_deref()
                .map(|file| jubarte::validate::input_validity_of(docx, file)),
        )
    })
    .map_err(err)
}

/// The line a writing task prints on stderr for a file it wrote:
/// `wrote r.pdf (pdf, 48213 bytes, 3 pages)`. `format` is `docx`, `md`,
/// `pdf`, `png` or `json`; `pages` is a PDF's page count.
#[pyfunction]
#[pyo3(signature = (path, format, size, pages = None))]
fn wrote_line(
    path: std::path::PathBuf,
    format: &str,
    size: usize,
    pages: Option<usize>,
) -> PyResult<String> {
    use jubarte::cli::Format;
    let format = match format {
        "docx" => Format::Docx,
        "md" => Format::Md,
        "pdf" => Format::Pdf,
        "png" => Format::Png,
        "json" => Format::Json,
        other => return Err(err(format!("unknown format {other:?}"))),
    };
    Ok(jubarte::cli::wrote_line(&path, format, size, pages))
}

/// The error line for an input no task reads, as the binary prints it:
/// `LEGACY_DOC: a.docx is a Word 97-2003 …`, or (`word`) `INVALID_PACKAGE:
/// a.docx is not a ZIP file: …`; `None` when the bytes may be read.
#[pyfunction]
fn refused_input(path: std::path::PathBuf, data: &[u8], word: bool) -> Option<String> {
    jubarte::cli::refused_input(&path, data, word)
}

/// [`refused_input`] for a package read whole (`validate --original`): it
/// must also pass admission, as the binary's does; `None` when admitted.
#[pyfunction]
fn refused_package(path: std::path::PathBuf, data: &[u8]) -> Option<String> {
    jubarte::cli::refused_package(&path, data)
}

/// The line for PNG pages: `wrote out/r-page-NN.png (png, 3 pages, 96 dpi)`,
/// `pattern` naming them all.
#[pyfunction]
fn wrote_png_line(pattern: std::path::PathBuf, pages: usize, dpi: f32) -> String {
    jubarte::cli::wrote_png_line(&pattern, pages, dpi)
}

/// A resolution's error: an id that is no change id, names two changes, or
/// names part of one, is a bad argument (`ValueError`, the binary's exit
/// 2); the rest are `JubarteError`.
fn change_err(e: jubarte::changes::ChangeError) -> PyErr {
    use jubarte::changes::ChangeError;
    match e {
        ChangeError::InvalidId(_)
        | ChangeError::Ambiguous { .. }
        | ChangeError::PartOfChange { .. } => {
            pyo3::exceptions::PyValueError::new_err(e.to_string())
        }
        e => err(e),
    }
}

fn change_filter(filter_json: &str) -> PyResult<jubarte::changes::ChangeFilter> {
    serde_json::from_str(filter_json)
        .map_err(|e| JubarteError::new_err(format!("invalid change filter: {e}")))
}

/// Accept the changes `filter_json` selects (`{"ids": [...], "authors":
/// [...], "kinds": [...]}`, every list given must match; `{}` selects every
/// change, an empty list none) and keep the rest tracked → DOCX bytes.
#[pyfunction]
fn accept_changes(py: Python<'_>, docx: &[u8], filter_json: &str) -> PyResult<Py<PyBytes>> {
    let filter = change_filter(filter_json)?;
    let out = py
        .detach(|| jubarte::changes::accept_changes(docx, &filter))
        .map_err(change_err)?;
    Ok(PyBytes::new(py, &out).unbind())
}

/// Reject the changes `filter_json` selects (as in `accept_changes`) and
/// keep the rest tracked → DOCX bytes.
#[pyfunction]
fn reject_changes(py: Python<'_>, docx: &[u8], filter_json: &str) -> PyResult<Py<PyBytes>> {
    let filter = change_filter(filter_json)?;
    let out = py
        .detach(|| jubarte::changes::reject_changes(docx, &filter))
        .map_err(change_err)?;
    Ok(PyBytes::new(py, &out).unbind())
}

/// List the tracked revisions in a DOCX as a JSON array string, grouped as
/// Docxodus's `GetRevisions` groups them; each object has
/// (`type`/`author`/`date`/`part`/`moveGroupId`/`isMoveSource`/`formatChange`/`text`).
/// `input_limits` as in `compare_documents`.
#[pyfunction]
#[pyo3(signature = (docx, *, input_limits = None))]
fn get_revisions_json(
    py: Python<'_>,
    docx: &[u8],
    input_limits: Option<std::collections::HashMap<String, u64>>,
) -> PyResult<String> {
    let settings = settings_with_limits(input_limits)?;
    py.detach(|| {
        let revs = jubarte::document_comparer::get_revisions(docx, &settings)
            .map_err(|e| e.to_string())?;
        Ok(jubarte::document_comparer::revisions_to_json(&revs))
    })
    .map_err(|e: String| err(e))
}

/// Render a DOCX package (bytes) → PDF bytes (Word-style layout).
///
/// `compress=True` deflates the PDF's streams (`/FlateDecode`), which is much
/// smaller but no longer plain text. `revisions` paints tracked changes:
/// `"conventional"` (red struck deletions, blue underlined insertions, green
/// moves double-struck and double-underlined), `"word"` (Microsoft Word's
/// markup) or `"custom"` with
/// `revision_palette="deleted=#AA0000:strike,..."`. `move_comments=True`
/// lists the comments after the last page instead of in balloons beside the
/// text; `changed_only=True` keeps only the pages a tracked change touches,
/// the pages a comment is anchored on and each section's first page on which
/// a revised header or footer paints (else the first page).
#[pyfunction]
#[pyo3(signature = (docx, compress = false, revisions = "conventional", revision_palette = None, move_comments = false, changed_only = false))]
fn docx_to_pdf(
    py: Python<'_>,
    docx: &[u8],
    compress: bool,
    revisions: &str,
    revision_palette: Option<&str>,
    move_comments: bool,
    changed_only: bool,
) -> PyResult<Py<PyBytes>> {
    let page = PageChoice {
        move_comments,
        changed_only,
    };
    let options = pdf_options(compress, revisions, revision_palette, page)?;
    let out = py
        .detach(|| jubarte::convert::docx_to_pdf_with(docx, options))
        .map_err(err)?;
    Ok(PyBytes::new(py, &out).unbind())
}

/// Rasterize every page to PNG at `dpi` → list of PNG bytes, page order.
/// `move_comments` and `changed_only` as in `docx_to_pdf`.
#[pyfunction]
#[pyo3(signature = (docx, dpi = 96.0, revisions = "conventional", revision_palette = None, move_comments = false, changed_only = false))]
fn docx_to_png(
    py: Python<'_>,
    docx: &[u8],
    dpi: f32,
    revisions: &str,
    revision_palette: Option<&str>,
    move_comments: bool,
    changed_only: bool,
) -> PyResult<Vec<Py<PyBytes>>> {
    let page = PageChoice {
        move_comments,
        changed_only,
    };
    let options = pdf_options(false, revisions, revision_palette, page)?;
    let pages = py
        .detach(|| jubarte::convert::docx_to_png(docx, options, dpi))
        .map_err(err)?;
    Ok(pages
        .iter()
        .map(|png| PyBytes::new(py, png).unbind())
        .collect())
}

/// `render`'s result: the PDF (when asked for), one PNG per page, and the
/// layout report as JSON.
type Rendered = (Option<Py<PyBytes>>, Vec<Py<PyBytes>>, String);

/// `edit_json`'s result: ok, the clean copy and the tracked redline (both
/// `None` on refusal), and the report or the structured refusal as JSON.
type EditOutcome = (bool, Option<Py<PyBytes>>, Option<Py<PyBytes>>, String);

/// One layout pass → `(pdf_bytes | None, [png_bytes, ...], report_json)`.
///
/// `report_json` is `{"page_count", "document_pages", "pages": [{"index",
/// "number", "text"}], "fonts": [...]}`. `pages` (zero-based document pages) writes only those
/// pages, ascending and without repeats: the PDF, the PNGs and the report.
/// They count the document's pages whatever `changed_only` keeps; a dropped
/// page or one past the last raises. `move_comments` and `changed_only` as
/// in `docx_to_pdf`.
#[pyfunction]
#[pyo3(signature = (docx, pdf = true, png_dpi = None, compress = false, revisions = "conventional", revision_palette = None, pages = None, move_comments = false, changed_only = false))]
#[expect(
    clippy::too_many_arguments,
    reason = "a Python signature of keyword arguments with defaults, as the binary's flags"
)]
fn render(
    docx: &Bound<'_, PyBytes>,
    pdf: bool,
    png_dpi: Option<f32>,
    compress: bool,
    revisions: &str,
    revision_palette: Option<&str>,
    pages: Option<Vec<usize>>,
    move_comments: bool,
    changed_only: bool,
) -> PyResult<Rendered> {
    let page = PageChoice {
        move_comments,
        changed_only,
    };
    let options = pdf_options(compress, revisions, revision_palette, page)?;
    let request = jubarte::convert::RenderRequest {
        pdf,
        png_dpi,
        pages,
    };
    let py = docx.py();
    let docx = docx.as_bytes();
    let rendered = py
        .detach(|| jubarte::convert::render(docx, options, request))
        .map_err(err)?;
    Ok((
        rendered.pdf.map(|b| PyBytes::new(py, &b).unbind()),
        rendered
            .pngs
            .iter()
            .map(|png| PyBytes::new(py, png).unbind())
            .collect(),
        rendered.report.to_json(),
    ))
}

/// `diff_render_json`'s result: the page diffs as JSON, both sides' PNG
/// pages, one overlay (or `None`) per page diff, and both page reports as
/// JSON.
type RenderDiffOut = (
    String,
    Vec<Py<PyBytes>>,
    Vec<Py<PyBytes>>,
    Vec<Option<Py<PyBytes>>>,
    String,
    String,
);

/// Which pages of `a` and `b` differ, pixel for pixel, from one layout pass
/// each at `dpi` → `(pages_json, a_pngs, b_pngs, overlays, a_report_json,
/// b_report_json)`. `pages_json` is `[{"index", "changed_ratio", "bbox",
/// "only_in"?}]`.
#[pyfunction]
#[pyo3(signature = (a, b, dpi = 100.0, overlay = true, revisions = "conventional", revision_palette = None))]
fn diff_render_json(
    py: Python<'_>,
    a: &[u8],
    b: &[u8],
    dpi: f32,
    overlay: bool,
    revisions: &str,
    revision_palette: Option<&str>,
) -> PyResult<RenderDiffOut> {
    let options = jubarte::convert::DiffOptions {
        dpi,
        pdf: pdf_options(false, revisions, revision_palette, PageChoice::default())?,
        overlay,
    };
    let diff = py
        .detach(|| jubarte::convert::diff_render(a, b, &options))
        .map_err(err)?;
    let pngs = |pages: &[Vec<u8>]| -> Vec<Py<PyBytes>> {
        pages
            .iter()
            .map(|png| PyBytes::new(py, png).unbind())
            .collect()
    };
    Ok((
        serde_json::to_string(&diff.pages).map_err(err)?,
        pngs(&diff.a),
        pngs(&diff.b),
        diff.overlays
            .iter()
            .map(|o| o.as_ref().map(|png| PyBytes::new(py, png).unbind()))
            .collect(),
        diff.a_report.to_json(),
        diff.b_report.to_json(),
    ))
}

/// One side of a diff: a Word document's bytes, or Markdown text.
#[derive(FromPyObject)]
enum Side<'py> {
    Word(Bound<'py, PyBytes>),
    Markdown(String),
}

impl Side<'_> {
    fn source(&self) -> jubarte::markdown::Source<'_> {
        match self {
            Side::Word(bytes) => jubarte::markdown::Source::Docx(bytes.as_bytes()),
            Side::Markdown(text) => jubarte::markdown::Source::Markdown(text),
        }
    }
}

/// `diff_json`'s and `redline_diff_json`'s result: the text, and the hunks
/// as JSON (`[{"at", "removed", "text"}]`, empty for CriticMarkup).
type Diffed = (String, String);

/// Python integers only: PyO3's u32 extraction otherwise accepts bool.
struct Context(u32);

impl FromPyObject<'_, '_> for Context {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, '_, PyAny>) -> PyResult<Self> {
        let invalid = || {
            pyo3::exceptions::PyValueError::new_err(
                "context must be an integer in the u32 range (0..4294967295)",
            )
        };
        if obj.is_instance_of::<PyBool>() || !obj.is_instance_of::<PyInt>() {
            return Err(invalid());
        }
        obj.extract::<u32>().map(Self).map_err(|_| invalid())
    }
}

/// Complete document text as a Git unified patch; no redline is constructed.
#[pyfunction]
#[pyo3(signature = (old, new, *, old_name = "old.docx", new_name = "new.docx", context = Context(3)))]
fn diff_unified(
    py: Python<'_>,
    old: Side<'_>,
    new: Side<'_>,
    old_name: &str,
    new_name: &str,
    context: Context,
) -> PyResult<String> {
    let options = jubarte::text_diff::UnifiedOptions {
        old_name: old_name.to_string(),
        new_name: new_name.to_string(),
        context: context.0 as usize,
    };
    let (old, new) = (old.source(), new.source());
    py.detach(|| jubarte::text_diff::diff_documents(old, new, &options))
        .map_err(err)
}

/// A document review view, with the core display window by default.
#[pyfunction]
#[pyo3(signature = (old, new, *, format = "github", context = Context(3), **view))]
fn diff_view(
    py: Python<'_>,
    old: Side<'_>,
    new: Side<'_>,
    format: &str,
    context: Context,
    view: Option<&Bound<'_, pyo3::types::PyDict>>,
) -> PyResult<String> {
    use jubarte::text_diff::{TextFormat, TextOptions, UnifiedOptions};
    let mut old_name = "old.docx".to_string();
    let mut new_name = "new.docx".to_string();
    let mut accept_changes = false;
    let mut full_lines = false;
    if let Some(view) = view {
        for (key, value) in view.iter() {
            let key: String = key.extract()?;
            match key.as_str() {
                "old_name" => old_name = value.extract()?,
                "new_name" => new_name = value.extract()?,
                "accept_changes" => accept_changes = value.extract()?,
                "full_lines" => full_lines = value.extract()?,
                _ => {
                    return Err(pyo3::exceptions::PyTypeError::new_err(format!(
                        "unexpected keyword argument '{key}'"
                    )));
                }
            }
        }
    }
    let format = match format {
        "github" | "unified" | "text" => TextFormat::Github,
        "word" => TextFormat::Word,
        "normal" => TextFormat::Normal,
        "context" => TextFormat::Context,
        "side-by-side" => TextFormat::SideBySide,
        _ => {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "format must be github, word, normal, context or side-by-side",
            ));
        }
    };
    let defaults = TextOptions::default();
    let options = TextOptions {
        unified: UnifiedOptions {
            old_name,
            new_name,
            context: context.0 as usize,
        },
        format,
        accept_changes,
        window: if full_lines { None } else { defaults.window },
    };
    let (old, new) = (old.source(), new.source());
    py.detach(|| jubarte::text_diff::diff_documents_view(old, new, &options))
        .map_err(err)
}

/// Pure shared clap parser. Arguments exclude argv0; no host I/O is done.
#[pyfunction]
#[pyo3(signature = (arguments, program = "jubarte-redlines", supported = None))]
fn parse_cli_json(arguments: Vec<String>, program: &str, supported: Option<Vec<String>>) -> String {
    jubarte::cli::parse_json(&arguments, program, &supported.unwrap_or_default())
}

/// Whether a command line (arguments exclude argv0) asks for JSON: a real
/// `--json` flag before any `--`, read as the binary reads it (the `--json`
/// in `--text --json` is a value). An error then prints its
/// [`runtime_json_error`] line on stdout too.
#[pyfunction]
fn wants_json(arguments: Vec<String>) -> bool {
    jubarte::cli::wants_json(&arguments)
}

/// The line an error prints on stdout under `--json`:
/// `{"error":{"code":…,"message":…}}`, its code the message's leading
/// `CODE:`, else `USAGE` for an exit 2 and `ERROR` for any other.
#[pyfunction]
fn runtime_json_error(message: &str, exit: u8) -> String {
    jubarte::cli::runtime_json_error(message, exit)
}

/// Word or Markdown sides as a Word tracked-changes document.
#[pyfunction]
#[pyo3(signature = (old, new, *, author, date))]
fn redline_documents(
    py: Python<'_>,
    old: Side<'_>,
    new: Side<'_>,
    author: &str,
    date: &str,
) -> PyResult<Py<PyBytes>> {
    let (old, new) = (old.source(), new.source());
    let settings = jubarte::comparer::WmlComparerSettings {
        author_for_revisions: author.to_string(),
        date_time_for_revisions: date.to_string(),
        ..Default::default()
    };
    let bytes = py
        .detach(|| {
            jubarte::markdown::redline(
                old,
                new,
                &jubarte::markdown::RedlineOptions {
                    settings,
                    ..Default::default()
                },
            )
        })
        .map_err(err)?;
    Ok(PyBytes::new(py, &bytes).unbind())
}

fn diffed(patch: &jubarte::markdown::Patch, columns: usize) -> PyResult<Diffed> {
    let hunks: Vec<serde_json::Value> = patch
        .hunks
        .iter()
        .map(|h| serde_json::json!({"at": h.at.to_string(), "removed": h.removed, "text": h.text}))
        .collect();
    Ok((
        patch.render(columns),
        serde_json::to_string(&hunks).map_err(err)?,
    ))
}

fn patch_options(
    old_name: &str,
    new_name: &str,
    author: &str,
    date: &str,
) -> jubarte::markdown::PatchOptions {
    jubarte::markdown::PatchOptions {
        old_name: old_name.to_string(),
        new_name: new_name.to_string(),
        owner: jubarte::markdown::Attribution {
            author: author.to_string(),
            date: date.to_string(),
        },
    }
}

/// The changes from `old` to `new` (each Word bytes or Markdown text) →
/// `(text, hunks_json)`: the patch of the changed paragraphs at `columns`
/// (0: not wrapped), or with `critic=True` the whole document as
/// CriticMarkup and no hunks. `author` and `date` own the changes; a Word
/// side is compared as `compare_documents` does, with them.
#[pyfunction]
#[pyo3(signature = (old, new, *, old_name, new_name, author, date, columns = 72, critic = false))]
#[expect(
    clippy::too_many_arguments,
    reason = "the parameters are the Python keyword API of diff_json"
)]
fn diff_json(
    py: Python<'_>,
    old: Side<'_>,
    new: Side<'_>,
    old_name: &str,
    new_name: &str,
    author: &str,
    date: &str,
    columns: usize,
    critic: bool,
) -> PyResult<Diffed> {
    let options = patch_options(old_name, new_name, author, date);
    let (old, new) = (old.source(), new.source());
    let settings = jubarte::comparer::WmlComparerSettings {
        author_for_revisions: author.to_string(),
        date_time_for_revisions: date.to_string(),
        ..jubarte::comparer::WmlComparerSettings::default()
    };
    // Built on the detached thread: its image loader is not `Sync`.
    let redline = || jubarte::markdown::RedlineOptions {
        settings: settings.clone(),
        ..jubarte::markdown::RedlineOptions::default()
    };
    if !critic {
        let patch = py
            .detach(|| jubarte::markdown::patch_documents(old, new, &redline(), &options))
            .map_err(err)?;
        return diffed(&patch, columns);
    }
    let text = py
        .detach(|| match (old, new) {
            (
                jubarte::markdown::Source::Markdown(old),
                jubarte::markdown::Source::Markdown(new),
            ) => Ok(jubarte::markdown::diff_markdown(old, new)),
            _ => jubarte::markdown::redline(old, new, &redline()).and_then(|docx| {
                jubarte::markdown::docx_to_markdown(
                    &docx,
                    &jubarte::markdown::MarkdownOptions {
                        input_limits: settings.input_limits,
                        ..Default::default()
                    },
                )
                .map(|read| read.markdown)
            }),
        })
        .map_err(err)?;
    Ok((text, "[]".to_string()))
}

/// The changes a Word redline tracks, as `diff_json`'s patch of the
/// document named `name` → `(text, hunks_json)`. `own_only` keeps only the
/// changes by `author` on `date` (an edit plan under `keep`).
#[pyfunction]
#[pyo3(signature = (docx, *, name, author, date, columns = 72, own_only = false))]
fn redline_diff_json(
    py: Python<'_>,
    docx: &[u8],
    name: &str,
    author: &str,
    date: &str,
    columns: usize,
    own_only: bool,
) -> PyResult<Diffed> {
    let options = patch_options(name, name, author, date);
    let patch = py
        .detach(|| {
            if own_only {
                jubarte::markdown::patch_own_changes(docx, &options)
            } else {
                jubarte::markdown::patch_redline(docx, &options)
            }
        })
        .map_err(err)?;
    diffed(&patch, columns)
}

/// SHA-256 (lowercase hex) of the bytes: the snapshot guard of edit plans.
#[pyfunction]
fn source_sha256(docx: &[u8]) -> String {
    jubarte::inspect::source_sha256(docx)
}

/// The inspection snapshot as JSON (`schema_version`, `source_sha256`,
/// `summary`, `paragraphs`).
#[pyfunction]
fn inspect_json(py: Python<'_>, docx: &[u8]) -> PyResult<String> {
    py.detach(|| jubarte::inspect::inspect_json(docx))
        .map_err(err)
}

/// Body paragraphs with ids, or Markdown with a tracked-change selection.
#[pyfunction]
#[pyo3(signature = (docx, track_changes = None))]
fn markdown(py: Python<'_>, docx: &[u8], track_changes: Option<&str>) -> PyResult<String> {
    let Some(choice) = track_changes else {
        return py.detach(|| jubarte::inspect::markdown(docx)).map_err(err);
    };
    let choice = jubarte::markdown::TrackChanges::parse(choice)
        .ok_or_else(|| err("track_changes must be all, accept or reject"))?;
    py.detach(|| {
        jubarte::markdown::docx_to_markdown(
            docx,
            &jubarte::markdown::MarkdownOptions {
                track_changes: choice,
                extract_media: None,
                ..jubarte::markdown::MarkdownOptions::default()
            },
        )
        .map(|read| read.markdown)
    })
    .map_err(err)
}

/// The agent view (`read`) → `(markdown, warnings)`: a YAML header, then
/// Markdown with `<!-- pN -->` id lines and every change and comment with
/// its id. `paragraphs` (`"p3,p10-p20,t0"`), `head` and `tail` select blocks;
/// `changed` keeps the blocks with marks, `by` only one author's (a handle
/// or a full name).
#[pyfunction]
#[pyo3(signature = (
    docx,
    *,
    track_changes = None,
    comments = true,
    dates = false,
    page_markers = true,
    paragraphs = None,
    head = None,
    tail = None,
    changed = false,
    by = None,
    source = None,
))]
#[expect(
    clippy::too_many_arguments,
    reason = "a Python signature of keyword arguments with defaults, as the binary's flags"
)]
fn read_view(
    py: Python<'_>,
    docx: &[u8],
    track_changes: Option<&str>,
    comments: bool,
    dates: bool,
    page_markers: bool,
    paragraphs: Option<&str>,
    head: Option<usize>,
    tail: Option<usize>,
    changed: bool,
    by: Option<&str>,
    source: Option<String>,
) -> PyResult<(String, Vec<String>)> {
    let track_changes = track_changes_of(track_changes)?;
    let select = jubarte::markdown::Select::from_flags(paragraphs, head, tail, changed, by)
        .map_err(usage_err)?;
    let options = jubarte::markdown::ReadOptions {
        track_changes,
        comments,
        dates,
        page_markers,
        select,
        source,
    };
    let view = py
        .detach(|| jubarte::markdown::read(docx, &options))
        .map_err(err)?;
    Ok((view.markdown, view.warnings))
}

/// Why `--editing-mode` cannot apply `plan_json` to `docx`: the tracked
/// changes it keeps and does not itself accept or reject, or a reply to a
/// comment on a paragraph it deletes; `None` when neither.
#[pyfunction]
fn editing_mode_conflict(py: Python<'_>, plan_json: &str, docx: &[u8]) -> Option<String> {
    py.detach(|| jubarte::edit::flags::editing_mode_conflict_json(plan_json, docx))
}

/// The plan the `edit`/`add` operation flags describe → `(plan_json,
/// notes)`; `operations` is the parsed command's `operations` array as JSON.
/// A `ValueError` is a usage error.
#[pyfunction]
#[pyo3(signature = (verb, operations, docx, *, author, date = None, existing = "auto"))]
fn flag_plan(
    py: Python<'_>,
    verb: &str,
    operations: &str,
    docx: &[u8],
    author: &str,
    date: Option<&str>,
    existing: &str,
) -> PyResult<(String, Vec<String>)> {
    py.detach(|| {
        jubarte::edit::flags::plan_from_flags_json(verb, operations, author, date, existing, docx)
    })
    .map_err(pyo3::exceptions::PyValueError::new_err)
}

/// `edit`/`add --plan PLAN.json` with operation flags: the operations
/// recorded into `held` (PLAN.json's text, `None` before the first
/// recording). `options` is JSON: `author`, `date`, `existingRevisions`,
/// `file`, `plan`, `defaultPlan` (a bare `--plan`), `dryRun`,
/// `rewriteOthersComments`. Returns `{plan, json, notes, summary}` as
/// JSON. A `ValueError` is a usage error; a `JubarteError` means `held` is
/// not a plan.
#[pyfunction]
#[pyo3(signature = (verb, operations, docx, options, *, held = None))]
fn record_plan(
    py: Python<'_>,
    verb: &str,
    operations: &str,
    docx: &[u8],
    options: &str,
    held: Option<&str>,
) -> PyResult<String> {
    use jubarte::edit::record::{RecordError, record_json};
    py.detach(|| record_json(verb, operations, docx, held, options))
        .map_err(|e| match e {
            RecordError::Usage(m) => pyo3::exceptions::PyValueError::new_err(m),
            RecordError::Plan(m) => JubarteError::new_err(m),
        })
}

/// The blocks of `docx` (a redline) carrying `author`'s marks, as the agent
/// view `edit` and `add` print; `accepted` reads its accepted text.
#[pyfunction]
#[pyo3(signature = (docx, author, *, accepted = false, source = None))]
fn changed_view(
    py: Python<'_>,
    docx: &[u8],
    author: &str,
    accepted: bool,
    source: Option<&str>,
) -> PyResult<String> {
    py.detach(|| jubarte::markdown::changed_view(docx, author, accepted, source))
        .map_err(err)
}

/// The lines `edit -p rN --accept/--reject` prints for a pure resolution:
/// one per resolved change from `source` (the bytes before resolution) and
/// `resolved_json` (`{"accepted": [...], "rejected": [...]}` short ids).
/// Mirrors `jubarte::edit::resolution::resolution_text`.
#[pyfunction]
fn resolution_text(py: Python<'_>, source: &[u8], resolved_json: &str) -> PyResult<String> {
    py.detach(|| {
        let resolved: jubarte::edit::ResolvedRevisions =
            serde_json::from_str(resolved_json).map_err(|e| format!("invalid resolved: {e}"))?;
        jubarte::edit::resolution::resolution_text(source, &resolved).map_err(|e| e.to_string())
    })
    .map_err(err)
}

/// Apply an edit plan (JSON) → `(ok, clean | None, redline | None, json)`.
///
/// On success `json` is the report; on refusal it is the structured error
/// (`code`, `operation`, `message`, `outcomes`) and both documents are `None`.
/// Refusals are data, not exceptions: the Python layer raises
/// `EditPlanError` from them so the outcomes stay attached.
#[pyfunction]
fn edit_json(py: Python<'_>, docx: &[u8], plan_json: &str) -> PyResult<EditOutcome> {
    let result = py.detach(|| jubarte::edit::apply_plan_json(docx, plan_json));
    Ok(match result {
        Ok(r) => (
            true,
            Some(PyBytes::new(py, &r.clean).unbind()),
            Some(PyBytes::new(py, &r.redline).unbind()),
            serde_json::to_string(&r.report).map_err(err)?,
        ),
        Err(e) => (false, None, None, serde_json::to_string(&e).map_err(err)?),
    })
}

/// Resolve an edit plan without producing documents → `(ok, json)`; `json`
/// is the report or the structured error.
#[pyfunction]
fn preview_json(py: Python<'_>, docx: &[u8], plan_json: &str) -> PyResult<(bool, String)> {
    let result = py.detach(|| {
        let (plan, notes) = jubarte::edit::EditPlan::from_json_with_notes(plan_json)?;
        jubarte::edit::preview_plan(docx, &plan).map(|mut report| {
            report.notes = notes;
            report
        })
    });
    Ok(match result {
        Ok(report) => (true, serde_json::to_string(&report).map_err(err)?),
        Err(e) => (false, serde_json::to_string(&e).map_err(err)?),
    })
}

/// The JSON-lines form of a report JSON (`load`, `op`..., `summary`).
#[pyfunction]
fn report_jsonl(report_json: &str) -> PyResult<String> {
    let report: jubarte::edit::EditReport = serde_json::from_str(report_json).map_err(err)?;
    Ok(report.to_jsonl())
}

/// Append B after A (`options_json` as `jubarte::append::AppendOptions`)
/// → `(docx_bytes, warnings_json)`.
#[pyfunction]
fn append_json(
    py: Python<'_>,
    a: &[u8],
    b: &[u8],
    options_json: &str,
) -> PyResult<(Py<PyBytes>, String)> {
    let options: jubarte::append::AppendOptions = serde_json::from_str(options_json)
        .map_err(|e| JubarteError::new_err(format!("invalid append options: {e}")))?;
    let out = py
        .detach(|| jubarte::append::append_documents(a, b, &options))
        .map_err(err)?;
    let warnings = serde_json::to_string(&out.warnings).map_err(err)?;
    Ok((PyBytes::new(py, &out.docx).unbind(), warnings))
}

/// Word-validity findings beyond the schema as a JSON array (`code`, `set`,
/// `severity`, `part`, `location`, `message`, `word_fatal`, `repairable`);
/// `[]` is a pass.
/// Mirrors `jubarte::validate::validate`; a package that cannot be read at
/// all raises.
#[pyfunction]
fn validate_json(py: Python<'_>, docx: &[u8]) -> PyResult<String> {
    py.detach(|| {
        let findings = jubarte::validate::validate(docx).map_err(|e| e.to_string())?;
        serde_json::to_string(&findings).map_err(|e| e.to_string())
    })
    .map_err(|e: String| err(e))
}

/// `validate_run`'s result: the findings as JSON Lines, as the CLI's text,
/// and whether they fail the run.
type ValidateRun = (String, String, bool);

/// `jubarte validate` on bytes: `rules` are rule sets or audit codes
/// (`None`: `word`, plus `edits` with `original`), `original` and `author`
/// check tracked edits, `strict` fails on warnings too. With `baseline`
/// (the document the edits started from) only the findings it lacks fail
/// the run, the JSON Lines and text are `validate --baseline`'s, and it is
/// the edits rule's original when `original` is not given. Mirrors
/// `jubarte::validate::validate_with`, `findings_text`, `baseline` and
/// `fails`.
#[pyfunction]
#[pyo3(signature = (docx, *, rules=None, original=None, author=None, strict=false, baseline=None))]
fn validate_run(
    py: Python<'_>,
    docx: &[u8],
    rules: Option<Vec<String>>,
    original: Option<&[u8]>,
    author: Option<&str>,
    strict: bool,
    baseline: Option<&[u8]>,
) -> PyResult<ValidateRun> {
    let rules = rules.unwrap_or_default();
    py.detach(|| {
        use jubarte::validate::{baselined_json, baselined_text, fails, validate_baselined};
        let rules: Vec<&str> = rules.iter().map(String::as_str).collect();
        if let Some(base) = baseline {
            let split = validate_baselined(
                docx,
                base,
                &rules,
                original,
                author,
                Some(&mut jubarte::audit::word_layout),
            )
            .map_err(|e| e.to_string())?;
            return Ok((
                baselined_json(&split, strict).map_err(|e| e.to_string())?,
                baselined_text(&split, strict),
                fails(&split.new, strict),
            ));
        }
        let findings = jubarte::validate::validate_with(
            docx,
            &rules,
            original.zip(author),
            Some(&mut jubarte::audit::word_layout),
        )
        .map_err(|e| e.to_string())?;
        let mut jsonl = String::new();
        for finding in &findings {
            jsonl.push_str(&serde_json::to_string(finding).map_err(|e| e.to_string())?);
            jsonl.push('\n');
        }
        Ok((
            jsonl,
            jubarte::validate::findings_text(&findings),
            jubarte::validate::fails(&findings, strict),
        ))
    })
    .map_err(|e: String| err(e))
}

/// The one stderr line `read`, `changes` and `comments` give an input with
/// validity findings (`input has 2 validity warnings; jubarte validate FILE
/// lists them`), or `None`; `file` is the path as the command line spells
/// it. Mirrors `jubarte::validate::input_notice`; a package that cannot be
/// validated has nothing to say.
#[pyfunction]
fn input_notice(py: Python<'_>, docx: &[u8], file: &str) -> Option<String> {
    py.detach(|| {
        jubarte::validate::validate(docx)
            .ok()
            .and_then(|findings| jubarte::validate::input_notice(&findings, file))
    })
}

/// The JSON status `changes --json` and `comments --json` print on stderr
/// unless quiet, with the input's validity findings counted (`{"input_validity":{"errors":0,"warnings":2,
/// "next":"jubarte validate FILE"}}`); `file` is the path as the command
/// line spells it. Mirrors `jubarte::validate::input_validity_line`.
#[pyfunction]
fn input_validity_line(py: Python<'_>, docx: &[u8], file: &str) -> PyResult<String> {
    py.detach(|| {
        jubarte::validate::input_validity_line(&jubarte::validate::input_validity_of(docx, file))
    })
    .map_err(|e| err(e.to_string()))
}

/// `repair_json`'s result: the repaired package and `{"repaired": [...],
/// "remaining": [...]}` as JSON.
type RepairOutcome = (Py<PyBytes>, String);

/// The package with every repairable finding fixed, and the findings it
/// fixed and could not fix. Mirrors `jubarte::validate::repair`.
#[pyfunction]
fn repair_json(py: Python<'_>, docx: &[u8]) -> PyResult<RepairOutcome> {
    let repaired = py.detach(|| jubarte::validate::repair(docx)).map_err(err)?;
    let json = serde_json::json!({
        "repaired": repaired.repaired,
        "remaining": repaired.remaining,
    });
    Ok((
        PyBytes::new(py, &repaired.docx).unbind(),
        serde_json::to_string(&json).map_err(err)?,
    ))
}

/// Every text change from `original` to `edited` must be a revision by
/// `author`: the residue after rejecting that author's changes is an
/// `UNTRACKED_EDIT` finding per paragraph, another author's change a
/// `FOREIGN_AUTHOR` one, another author's comment gone, reworded, resolved
/// or reopened (or a new comment not signed `author`) a `FOREIGN_COMMENT`
/// one, and a permission endpoint of `original` that `edited` lost a
/// `PERMISSION_RANGE_DROPPED` one, as a JSON array. Mirrors
/// `jubarte::validate::audit_tracked`.
#[pyfunction]
fn audit_tracked_json(
    py: Python<'_>,
    original: &[u8],
    edited: &[u8],
    author: &str,
) -> PyResult<String> {
    py.detach(|| {
        let findings = jubarte::validate::audit_tracked(original, edited, author)
            .map_err(|e| e.to_string())?;
        serde_json::to_string(&findings).map_err(|e| e.to_string())
    })
    .map_err(|e: String| err(e))
}

/// Refresh field results from jubarte's layout → `(docx, json)`; `json` is
/// `{"page_count", "fields": [...]}`.
#[pyfunction]
fn update_fields(py: Python<'_>, docx: &[u8]) -> PyResult<(Py<PyBytes>, String)> {
    let updated = py
        .detach(|| jubarte::fields::update_fields(docx))
        .map_err(err)?;
    let report = serde_json::json!({
        "page_count": updated.page_count,
        "fields": updated.fields,
    });
    Ok((PyBytes::new(py, &updated.docx).unbind(), report.to_string()))
}

/// Remove the identifying data `options_json` names (`{"author_alias":
/// "Author", "rsids": true, "docprops": true, "comments": true}`; a field
/// left out is off) → DOCX bytes.
#[pyfunction]
fn scrub_json(py: Python<'_>, docx: &[u8], options_json: &str) -> PyResult<Py<PyBytes>> {
    let options: jubarte::scrub::ScrubOptions = serde_json::from_str(options_json)
        .map_err(|e| JubarteError::new_err(format!("invalid scrub options: {e}")))?;
    let out = py
        .detach(|| jubarte::scrub::scrub(docx, &options))
        .map_err(err)?;
    Ok(PyBytes::new(py, &out).unbind())
}

/// This build's manifest (`runtime: "python"`): an edit plan here takes
/// no `--skip-failed`, as the shared manifest says.
fn manifest() -> jubarte::capabilities::Capabilities {
    jubarte::capabilities::capabilities("python")
}

/// What this build can do (`runtime: "python"`).
#[pyfunction]
fn capabilities_json() -> String {
    serde_json::to_string_pretty(&manifest()).expect("manifest serializes")
}

/// Focused edit discovery, or the edit support matrix when both filters are absent.
#[pyfunction]
#[pyo3(signature = (operation=None, field=None))]
fn capability_query(operation: Option<&str>, field: Option<&str>) -> String {
    jubarte::capabilities::discovery("python", operation, field).to_string()
}

/// [`capabilities_json`] as the text summary `jubarte capabilities` prints.
#[pyfunction]
#[pyo3(signature = (operation=None, field=None, matrix=false))]
fn capabilities_text(operation: Option<&str>, field: Option<&str>, matrix: bool) -> String {
    if matrix || operation.is_some() || field.is_some() {
        jubarte::capabilities::discovery_text(&jubarte::capabilities::discovery(
            "python", operation, field,
        ))
    } else {
        jubarte::capabilities::capabilities_text(&manifest())
    }
}

/// Markdown (with CriticMarkup) → DOCX bytes, as `jubarte convert draft.md`.
///
/// `options_json` holds [`jubarte::markdown::docx_options_from_json`]'s
/// fields, each optional: `page` (`letter` or `a4`, when there is no
/// `reference`, a `.docx` whose styles and page setup are taken),
/// `track_changes` (`all`, `accept` or `reject`), `author` and `date` (for
/// a change or comment that names none; else the front matter's, else
/// `Redline` and the engine's fixed epoch, so the same Markdown writes the
/// same bytes), `critic`, `tracking` (Word's Track Changes switch) and
/// `scholar`. Images are written as their alt text: this entry point reads
/// no files. Each engine warning (such as a `page` overridden by the
/// reference) is raised as a `UserWarning`.
#[pyfunction]
#[pyo3(signature = (text, options_json = None, reference = None))]
fn markdown_to_docx(
    py: Python<'_>,
    text: &str,
    options_json: Option<&str>,
    reference: Option<&[u8]>,
) -> PyResult<Py<PyBytes>> {
    let written = py
        .detach(|| {
            // Built here: `DocxOptions` can hold an image loader, which is
            // not `Send`, so it cannot cross into the detached call.
            let options =
                jubarte::markdown::docx_options_from_json(options_json.unwrap_or("{}"), reference)?;
            jubarte::markdown::markdown_to_docx(text, &options).map_err(|e| e.to_string())
        })
        .map_err(JubarteError::new_err)?;
    let category = py.get_type::<pyo3::exceptions::PyUserWarning>();
    for warning in &written.warnings {
        let message = std::ffi::CString::new(warning.as_str()).map_err(err)?;
        PyErr::warn(py, &category, &message, 1)?;
    }
    Ok(PyBytes::new(py, &written.docx).unbind())
}

/// `{findings, rules, layout}` from `jubarte::audit` as JSON; `rules`
/// names rule sets or codes (`None`: every rule).
#[pyfunction]
#[pyo3(signature = (docx, rules=None))]
fn audit_json(py: Python<'_>, docx: &[u8], rules: Option<Vec<String>>) -> PyResult<String> {
    let rules = rules.unwrap_or_default();
    let report = py
        .detach(|| {
            let rules: Vec<&str> = rules.iter().map(String::as_str).collect();
            jubarte::audit::audit_report(docx, &rules)
        })
        .map_err(err)?;
    serde_json::to_string(&report).map_err(err)
}

/// Persistent filesystem authority for the hosted MCP adapter. The root is
/// opened once, not re-resolved by pathname for every write.
#[pyclass(name = "_OutputRoot")]
struct PyOutputRoot {
    root: jubarte::publication::OutputRoot,
}

#[pymethods]
impl PyOutputRoot {
    #[new]
    fn new(root: &str) -> PyResult<Self> {
        Ok(Self {
            root: jubarte::publication::OutputRoot::open(std::path::Path::new(root))?,
        })
    }

    fn read<'py>(&self, py: Python<'py>, path: &str) -> PyResult<Bound<'py, PyBytes>> {
        let bytes = py.detach(|| self.root.read(std::path::Path::new(path)))?;
        Ok(PyBytes::new(py, &bytes))
    }

    fn publish(
        &self,
        py: Python<'_>,
        files: Vec<(String, Vec<u8>)>,
        overwrite: bool,
    ) -> PyResult<()> {
        let result = py.detach(|| {
            let files: Vec<_> = files
                .iter()
                .map(|(path, bytes)| (std::path::PathBuf::from(path), bytes.as_slice()))
                .collect();
            self.root.publish(&files, overwrite)
        });
        publish_result(py, result)
    }
}

fn publish_result(
    py: Python<'_>,
    result: Result<jubarte::publication::PublishReport, jubarte::publication::PublishError>,
) -> PyResult<()> {
    let (warnings, result) = match result {
        Ok(report) => (report.warnings, Ok(())),
        Err(error) => {
            // PublishError's Display preserves the primary error and committed
            // state, but does not include its cleanup warnings.
            let mut message = error.to_string();
            for warning in &error.warnings {
                message.push_str(&format!("\nwarning: {warning}"));
            }
            (
                error.warnings,
                Err(pyo3::exceptions::PyOSError::new_err(message)),
            )
        }
    };
    let category = py.get_type::<pyo3::exceptions::PyRuntimeWarning>();
    for warning in warnings {
        let emitted = std::ffi::CString::new(warning)
            .map_err(err)
            .and_then(|message| PyErr::warn(py, &category, &message, 1));
        // Warning filters may raise. Preserve the original publication error
        // when there is one; its message already contains every warning.
        if result.is_ok() {
            emitted?;
        }
    }
    result
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<PyOutputRoot>()?;
    m.add("JubarteError", m.py().get_type::<JubarteError>())?;
    m.add_function(wrap_pyfunction!(compare_documents, m)?)?;
    m.add_function(wrap_pyfunction!(read_view, m)?)?;
    m.add_function(wrap_pyfunction!(flag_plan, m)?)?;
    m.add_function(wrap_pyfunction!(editing_mode_conflict, m)?)?;
    m.add_function(wrap_pyfunction!(record_plan, m)?)?;
    m.add_function(wrap_pyfunction!(changed_view, m)?)?;
    m.add_function(wrap_pyfunction!(resolution_text, m)?)?;
    m.add_function(wrap_pyfunction!(accept_revisions, m)?)?;
    m.add_function(wrap_pyfunction!(reject_revisions, m)?)?;
    m.add_function(wrap_pyfunction!(get_revisions_json, m)?)?;
    m.add_function(wrap_pyfunction!(list_changes_json, m)?)?;
    m.add_function(wrap_pyfunction!(accept_changes, m)?)?;
    m.add_function(wrap_pyfunction!(reject_changes, m)?)?;
    m.add_function(wrap_pyfunction!(docx_to_pdf, m)?)?;
    m.add_function(wrap_pyfunction!(docx_to_png, m)?)?;
    m.add_function(wrap_pyfunction!(render, m)?)?;
    m.add_function(wrap_pyfunction!(diff_render_json, m)?)?;
    m.add_function(wrap_pyfunction!(source_sha256, m)?)?;
    m.add_function(wrap_pyfunction!(inspect_json, m)?)?;
    m.add_function(wrap_pyfunction!(markdown, m)?)?;
    m.add_function(wrap_pyfunction!(edit_json, m)?)?;
    m.add_function(wrap_pyfunction!(preview_json, m)?)?;
    m.add_function(wrap_pyfunction!(report_jsonl, m)?)?;
    m.add_function(wrap_pyfunction!(capabilities_json, m)?)?;
    m.add_function(wrap_pyfunction!(capability_query, m)?)?;
    m.add_function(wrap_pyfunction!(capabilities_text, m)?)?;
    m.add_function(wrap_pyfunction!(diff_json, m)?)?;
    m.add_function(wrap_pyfunction!(diff_unified, m)?)?;
    m.add_function(wrap_pyfunction!(diff_view, m)?)?;
    m.add_function(wrap_pyfunction!(parse_cli_json, m)?)?;
    m.add_function(wrap_pyfunction!(wants_json, m)?)?;
    m.add_function(wrap_pyfunction!(runtime_json_error, m)?)?;
    m.add_function(wrap_pyfunction!(redline_documents, m)?)?;
    m.add_function(wrap_pyfunction!(redline_diff_json, m)?)?;
    m.add_function(wrap_pyfunction!(list_comments_json, m)?)?;
    m.add_function(wrap_pyfunction!(changes_text, m)?)?;
    m.add_function(wrap_pyfunction!(comments_text, m)?)?;
    m.add_function(wrap_pyfunction!(tables_text, m)?)?;
    m.add_function(wrap_pyfunction!(tables_view, m)?)?;
    m.add_function(wrap_pyfunction!(read_json, m)?)?;
    m.add_function(wrap_pyfunction!(for_edit_json, m)?)?;
    m.add_function(wrap_pyfunction!(wrote_line, m)?)?;
    m.add_function(wrap_pyfunction!(wrote_png_line, m)?)?;
    m.add_function(wrap_pyfunction!(refused_input, m)?)?;
    m.add_function(wrap_pyfunction!(refused_package, m)?)?;
    m.add_function(wrap_pyfunction!(append_json, m)?)?;
    m.add_function(wrap_pyfunction!(validate_json, m)?)?;
    m.add_function(wrap_pyfunction!(validate_run, m)?)?;
    m.add_function(wrap_pyfunction!(input_notice, m)?)?;
    m.add_function(wrap_pyfunction!(input_validity_line, m)?)?;
    m.add_function(wrap_pyfunction!(repair_json, m)?)?;
    m.add_function(wrap_pyfunction!(audit_tracked_json, m)?)?;
    m.add_function(wrap_pyfunction!(markdown_to_docx, m)?)?;
    m.add_function(wrap_pyfunction!(markdown_stamp, m)?)?;
    m.add_function(wrap_pyfunction!(update_fields, m)?)?;
    m.add_function(wrap_pyfunction!(scrub_json, m)?)?;
    m.add_function(wrap_pyfunction!(audit_json, m)?)?;
    Ok(())
}

#[cfg(test)]
mod publication_tests {
    use super::*;
    use jubarte::publication::{PublishError, PublishReport};
    use pyo3::exceptions::{PyOSError, PyRuntimeWarning};

    #[test]
    fn publication_report_emits_each_runtime_warning_and_keeps_unit() {
        Python::initialize();
        Python::attach(|py| {
            let warnings = py.import("warnings").unwrap();
            let context = warnings
                .call_method(
                    "catch_warnings",
                    (),
                    Some(&{
                        let kwargs = pyo3::types::PyDict::new(py);
                        kwargs.set_item("record", true).unwrap();
                        kwargs
                    }),
                )
                .unwrap();
            let caught = context.call_method0("__enter__").unwrap();
            warnings.call_method1("simplefilter", ("always",)).unwrap();
            let report = PublishReport {
                warnings: vec![
                    "cleaning staging: first".into(),
                    "cleaning staging: second".into(),
                ],
            };
            let result = publish_result(py, Ok(report));
            assert!(result.is_ok(), "{result:?}");
            assert_eq!(caught.len().unwrap(), 2);
            for (index, message) in ["cleaning staging: first", "cleaning staging: second"]
                .iter()
                .enumerate()
            {
                let warning = caught.get_item(index).unwrap();
                assert!(
                    warning
                        .getattr("category")
                        .unwrap()
                        .is(py.get_type::<PyRuntimeWarning>())
                );
                assert_eq!(
                    warning
                        .getattr("message")
                        .unwrap()
                        .str()
                        .unwrap()
                        .to_str()
                        .unwrap(),
                    *message
                );
            }
            context
                .call_method1("__exit__", (py.None(), py.None(), py.None()))
                .unwrap();
        });
    }

    #[test]
    fn publication_error_preserves_primary_state_and_all_cleanup_warnings() {
        Python::initialize();
        Python::attach(|py| {
            let warnings = py.import("warnings").unwrap();
            let kwargs = pyo3::types::PyDict::new(py);
            kwargs.set_item("record", true).unwrap();
            let context = warnings
                .call_method("catch_warnings", (), Some(&kwargs))
                .unwrap();
            let caught = context.call_method0("__enter__").unwrap();
            warnings.call_method1("simplefilter", ("always",)).unwrap();
            let error = PublishError {
                committed: vec!["a.txt".into()],
                durability_uncertain: true,
                source: std::io::Error::other("primary failure"),
                warnings: vec![
                    "cleaning staging: first".into(),
                    "cleaning staging: second".into(),
                ],
            };
            let raised = publish_result(py, Err(error)).unwrap_err();
            assert_eq!(caught.len().unwrap(), 2);
            for index in 0..2 {
                assert!(
                    caught
                        .get_item(index)
                        .unwrap()
                        .getattr("category")
                        .unwrap()
                        .is(py.get_type::<PyRuntimeWarning>())
                );
            }
            context
                .call_method1("__exit__", (py.None(), py.None(), py.None()))
                .unwrap();
            assert!(raised.is_instance_of::<PyOSError>(py));
            let message = raised.value(py).str().unwrap().to_str().unwrap().to_owned();
            for expected in [
                "primary failure",
                "a.txt",
                "durability uncertain",
                "cleaning staging: first",
                "cleaning staging: second",
            ] {
                assert!(message.contains(expected), "missing {expected}: {message}");
            }
        });
    }
    #[test]
    fn publication_success_obeys_warnings_as_errors() {
        Python::initialize();
        Python::attach(|py| {
            let warnings = py.import("warnings").unwrap();
            let context = warnings.call_method0("catch_warnings").unwrap();
            context.call_method0("__enter__").unwrap();
            warnings.call_method1("simplefilter", ("error",)).unwrap();
            assert!(publish_result(py, Ok(PublishReport::default())).is_ok());
            let raised = publish_result(
                py,
                Ok(PublishReport {
                    warnings: vec!["cleaning staging: first".into()],
                }),
            )
            .unwrap_err();
            assert!(raised.is_instance_of::<PyRuntimeWarning>(py));
            let raised = publish_result(
                py,
                Err(PublishError {
                    committed: vec![],
                    durability_uncertain: false,
                    source: std::io::Error::other("primary failure"),
                    warnings: vec!["cleaning staging: first".into()],
                }),
            )
            .unwrap_err();
            assert!(raised.is_instance_of::<PyOSError>(py));
            assert!(raised.to_string().contains("cleaning staging: first"));
            context
                .call_method1("__exit__", (py.None(), py.None(), py.None()))
                .unwrap();
        });
    }
}
