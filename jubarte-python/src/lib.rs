// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Python bindings for the canonical **jubarte-redlines** Word-mode compare.
//!
//! Built with **PyO3** + **maturin** as the `jubarte_redlines._native`
//! extension module; the public Python surface (including `get_revisions`
//! returning parsed objects) lives in `python/jubarte_redlines/__init__.py`.
//!
//! Every entry point copies nothing extra and detaches from the interpreter for the whole
//! pure-Rust compute, so long compares don't block other Python threads.

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
/// smaller but no longer plain text.
#[pyfunction]
#[pyo3(signature = (docx, compress = false))]
fn docx_to_pdf(py: Python<'_>, docx: &[u8], compress: bool) -> PyResult<Py<PyBytes>> {
    let options = jubarte::convert::PdfOptions { compress };
    let out = py
        .detach(|| jubarte::convert::docx_to_pdf_with(docx, options))
        .map_err(err)?;
    Ok(PyBytes::new(py, &out).unbind())
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
    Ok(())
}
