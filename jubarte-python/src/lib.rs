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
use pyo3::types::PyBytes;

create_exception!(
    jubarte_redlines,
    JubarteError,
    PyException,
    "Raised when the jubarte-redlines engine cannot process a document."
);

fn err(e: impl std::fmt::Display) -> PyErr {
    JubarteError::new_err(e.to_string())
}

fn pdf_options(
    compress: bool,
    revisions: &str,
    revision_palette: Option<&str>,
) -> PyResult<jubarte::convert::PdfOptions> {
    let revisions = jubarte::convert::RevisionStyle::from_choice(revisions, revision_palette)
        .map_err(JubarteError::new_err)?;
    Ok(jubarte::convert::PdfOptions {
        compress,
        revisions,
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
/// `max_uncompressed_bytes`, `max_xml_depth`); a package past it raises
/// `JubarteError` with `INPUT_LIMIT`.
#[pyfunction]
#[pyo3(signature = (original, modified, author = "jubarte", date = None, *, input_limits = None))]
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

/// List the tracked changes one by one as a JSON array string, each with the
/// id `accept_changes` / `reject_changes` select by (the same objects as
/// `jubarte changes --json`).
#[pyfunction]
fn list_changes_json(py: Python<'_>, docx: &[u8]) -> PyResult<String> {
    py.detach(|| {
        let changes = jubarte::changes::list_changes(docx).map_err(|e| e.to_string())?;
        serde_json::to_string(&changes).map_err(|e| e.to_string())
    })
    .map_err(|e: String| JubarteError::new_err(e))
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
    .map_err(|e: String| JubarteError::new_err(e))
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
        .map_err(err)?;
    Ok(PyBytes::new(py, &out).unbind())
}

/// Reject the changes `filter_json` selects (as in `accept_changes`) and
/// keep the rest tracked → DOCX bytes.
#[pyfunction]
fn reject_changes(py: Python<'_>, docx: &[u8], filter_json: &str) -> PyResult<Py<PyBytes>> {
    let filter = change_filter(filter_json)?;
    let out = py
        .detach(|| jubarte::changes::reject_changes(docx, &filter))
        .map_err(err)?;
    Ok(PyBytes::new(py, &out).unbind())
}

/// List the tracked revisions in a DOCX as a JSON array string — the same
/// object shape as the CLI `jubarte revisions --json` lines
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
    .map_err(|e: String| JubarteError::new_err(e))
}

/// Render a DOCX package (bytes) → PDF bytes (Word-style layout).
///
/// `compress=True` deflates the PDF's streams (`/FlateDecode`), which is much
/// smaller but no longer plain text. `revisions` paints tracked changes:
/// `"conventional"` (red struck deletions, blue double-underlined insertions,
/// green moves), `"word"` (Microsoft Word's markup) or `"custom"` with
/// `revision_palette="deleted=#AA0000:strike,..."`.
#[pyfunction]
#[pyo3(signature = (docx, compress = false, revisions = "conventional", revision_palette = None))]
fn docx_to_pdf(
    py: Python<'_>,
    docx: &[u8],
    compress: bool,
    revisions: &str,
    revision_palette: Option<&str>,
) -> PyResult<Py<PyBytes>> {
    let options = pdf_options(compress, revisions, revision_palette)?;
    let out = py
        .detach(|| jubarte::convert::docx_to_pdf_with(docx, options))
        .map_err(err)?;
    Ok(PyBytes::new(py, &out).unbind())
}

/// Rasterize every page to PNG at `dpi` → list of PNG bytes, page order.
#[pyfunction]
#[pyo3(signature = (docx, dpi = 96.0, revisions = "conventional", revision_palette = None))]
fn docx_to_png(
    py: Python<'_>,
    docx: &[u8],
    dpi: f32,
    revisions: &str,
    revision_palette: Option<&str>,
) -> PyResult<Vec<Py<PyBytes>>> {
    let options = pdf_options(false, revisions, revision_palette)?;
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
/// `report_json` is `{"page_count", "pages": [{"index", "text"}], "fonts": [...]}`.
/// `pages` (zero-based) rasterizes only those pages, ascending and without
/// repeats; the report still covers every page.
#[pyfunction]
#[pyo3(signature = (docx, pdf = true, png_dpi = None, compress = false, revisions = "conventional", revision_palette = None, pages = None))]
fn render(
    docx: &Bound<'_, PyBytes>,
    pdf: bool,
    png_dpi: Option<f32>,
    compress: bool,
    revisions: &str,
    revision_palette: Option<&str>,
    pages: Option<Vec<usize>>,
) -> PyResult<Rendered> {
    let options = pdf_options(compress, revisions, revision_palette)?;
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
        pdf: pdf_options(false, revisions, revision_palette)?,
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
#[allow(clippy::too_many_arguments)]
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
                    &jubarte::markdown::MarkdownOptions::default(),
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

/// Body paragraphs as Markdown with `[body:p:N]` ids.
#[pyfunction]
fn markdown(py: Python<'_>, docx: &[u8]) -> PyResult<String> {
    py.detach(|| jubarte::inspect::markdown(docx)).map_err(err)
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
        let plan = jubarte::edit::EditPlan::from_json(plan_json)?;
        jubarte::edit::preview_plan(docx, &plan)
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

/// Word-validity findings beyond the schema as a JSON array (`code`,
/// `part`, `path`, `message`, `word_fatal`, `repairable`); `[]` is a pass.
/// Mirrors `jubarte::validate::validate`; a package that cannot be read at
/// all raises.
#[pyfunction]
fn validate_json(py: Python<'_>, docx: &[u8]) -> PyResult<String> {
    py.detach(|| {
        let findings = jubarte::validate::validate(docx).map_err(|e| e.to_string())?;
        serde_json::to_string(&findings).map_err(|e| e.to_string())
    })
    .map_err(|e: String| JubarteError::new_err(e))
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
/// `UNTRACKED_EDIT` finding per paragraph, and another author's change a
/// `FOREIGN_AUTHOR` one, as a JSON array. Mirrors
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
    .map_err(|e: String| JubarteError::new_err(e))
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

/// What this build can do (`runtime: "python"`).
#[pyfunction]
fn capabilities_json() -> String {
    jubarte::capabilities::capabilities_json("python")
}

/// Markdown (with CriticMarkup) → DOCX bytes, as `jubarte convert draft.md`.
///
/// `page` is `letter` or `a4` and applies when there is no `reference`
/// (a `.docx` whose styles and page setup are taken). `track_changes` is
/// `all`, `accept` or `reject`. `date` defaults to the engine's fixed epoch
/// so the same Markdown writes the same bytes. Images are written as their
/// alt text: this entry point reads no files. Each engine warning (such as
/// a `page` overridden by the reference) is raised as a `UserWarning`.
#[pyfunction]
#[pyo3(signature = (
    text,
    *,
    reference = None,
    page = "letter",
    author = "Redline",
    date = None,
    critic = true,
    track_changes = "all",
))]
fn markdown_to_docx(
    text: &str,
    reference: Option<&[u8]>,
    page: &str,
    author: &str,
    date: Option<&str>,
    critic: bool,
    track_changes: &str,
) -> PyResult<Py<PyBytes>> {
    let page = jubarte::markdown::PageSize::parse(page)
        .ok_or_else(|| JubarteError::new_err(format!("page must be letter or a4, not {page:?}")))?;
    let track_changes = jubarte::markdown::TrackChanges::parse(track_changes).ok_or_else(|| {
        JubarteError::new_err(format!(
            "track_changes must be all, accept or reject, not {track_changes:?}"
        ))
    })?;
    // Seven parameters keep clippy's argument limit; the interpreter is
    // reached through `attach`, which only borrows the caller's.
    Python::attach(|py| {
        let written = py
            .detach(|| {
                // Built here: `DocxOptions` can hold an image loader, which
                // is not `Send`, so it cannot cross into the detached call.
                let mut options = jubarte::markdown::DocxOptions {
                    reference,
                    critic,
                    track_changes,
                    author: author.to_string(),
                    page,
                    ..jubarte::markdown::DocxOptions::default()
                };
                if let Some(date) = date {
                    options.date = date.to_string();
                }
                jubarte::markdown::markdown_to_docx(text, &options)
            })
            .map_err(err)?;
        let category = py.get_type::<pyo3::exceptions::PyUserWarning>();
        for warning in &written.warnings {
            let message = std::ffi::CString::new(warning.as_str()).map_err(err)?;
            PyErr::warn(py, &category, &message, 1)?;
        }
        Ok(PyBytes::new(py, &written.docx).unbind())
    })
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

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("JubarteError", m.py().get_type::<JubarteError>())?;
    m.add_function(wrap_pyfunction!(compare_documents, m)?)?;
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
    m.add_function(wrap_pyfunction!(diff_json, m)?)?;
    m.add_function(wrap_pyfunction!(redline_diff_json, m)?)?;
    m.add_function(wrap_pyfunction!(list_comments_json, m)?)?;
    m.add_function(wrap_pyfunction!(append_json, m)?)?;
    m.add_function(wrap_pyfunction!(validate_json, m)?)?;
    m.add_function(wrap_pyfunction!(repair_json, m)?)?;
    m.add_function(wrap_pyfunction!(audit_tracked_json, m)?)?;
    m.add_function(wrap_pyfunction!(markdown_to_docx, m)?)?;
    m.add_function(wrap_pyfunction!(update_fields, m)?)?;
    m.add_function(wrap_pyfunction!(scrub_json, m)?)?;
    m.add_function(wrap_pyfunction!(audit_json, m)?)?;
    Ok(())
}
