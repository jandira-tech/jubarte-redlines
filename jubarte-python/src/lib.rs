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

/// Compare two DOCX packages (bytes) → redline DOCX bytes (`w:ins`/`w:del`).
///
/// Mirrors `jubarte::document_comparer::compare_documents`; `date` (ISO-8601
/// `w:date` stamp) defaults to the engine's fixed epoch for deterministic
/// output.
#[pyfunction]
#[pyo3(signature = (original, modified, author = "jubarte", date = None))]
fn compare_documents(
    py: Python<'_>,
    original: &[u8],
    modified: &[u8],
    author: &str,
    date: Option<&str>,
) -> PyResult<Py<PyBytes>> {
    let out = py
        .detach(|| match date {
            Some(d) => jubarte::document_comparer::compare_documents_with_options(
                original, modified, author, d,
            ),
            None => jubarte::document_comparer::compare_documents(original, modified, author),
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

/// List the tracked revisions in a DOCX as a JSON array string — the same
/// object shape as the CLI `jubarte revisions --json` lines
/// (`type`/`author`/`date`/`part`/`moveGroupId`/`isMoveSource`/`formatChange`/`text`).
#[pyfunction]
fn get_revisions_json(py: Python<'_>, docx: &[u8]) -> PyResult<String> {
    py.detach(|| {
        let settings = jubarte::comparer::WmlComparerSettings::default();
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

/// One layout pass → `(pdf_bytes | None, [png_bytes, ...], report_json)`.
///
/// `report_json` is `{"page_count", "pages": [{"index", "text"}], "fonts": [...]}`.
#[pyfunction]
#[pyo3(signature = (docx, pdf = true, png_dpi = None, compress = false, revisions = "conventional", revision_palette = None))]
fn render(
    py: Python<'_>,
    docx: &[u8],
    pdf: bool,
    png_dpi: Option<f32>,
    compress: bool,
    revisions: &str,
    revision_palette: Option<&str>,
) -> PyResult<(Option<Py<PyBytes>>, Vec<Py<PyBytes>>, String)> {
    let options = pdf_options(compress, revisions, revision_palette)?;
    let request = jubarte::convert::RenderRequest { pdf, png_dpi };
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
fn edit_json(
    py: Python<'_>,
    docx: &[u8],
    plan_json: &str,
) -> PyResult<(bool, Option<Py<PyBytes>>, Option<Py<PyBytes>>, String)> {
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

/// What this build can do (`runtime: "python"`).
#[pyfunction]
fn capabilities_json() -> String {
    jubarte::capabilities::capabilities_json("python")
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("JubarteError", m.py().get_type::<JubarteError>())?;
    m.add_function(wrap_pyfunction!(compare_documents, m)?)?;
    m.add_function(wrap_pyfunction!(accept_revisions, m)?)?;
    m.add_function(wrap_pyfunction!(reject_revisions, m)?)?;
    m.add_function(wrap_pyfunction!(get_revisions_json, m)?)?;
    m.add_function(wrap_pyfunction!(docx_to_pdf, m)?)?;
    m.add_function(wrap_pyfunction!(docx_to_png, m)?)?;
    m.add_function(wrap_pyfunction!(render, m)?)?;
    m.add_function(wrap_pyfunction!(source_sha256, m)?)?;
    m.add_function(wrap_pyfunction!(inspect_json, m)?)?;
    m.add_function(wrap_pyfunction!(markdown, m)?)?;
    m.add_function(wrap_pyfunction!(edit_json, m)?)?;
    m.add_function(wrap_pyfunction!(preview_json, m)?)?;
    m.add_function(wrap_pyfunction!(report_jsonl, m)?)?;
    m.add_function(wrap_pyfunction!(capabilities_json, m)?)?;
    Ok(())
}
