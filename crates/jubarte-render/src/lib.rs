// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! # jubarte-render
//!
//! The render product of the jubarte engine: `convert::docx_to_pdf` and
//! friends lay a Word document out page by page and write a PDF (or PNG
//! pages) with Word-faithful layout - the fidelity benchmarks measure the
//! `word` revision style against Microsoft Word's own Save as PDF. The
//! pure option types (`PdfOptions`, `RevisionStyle`, …) live in
//! jubarte-opc's `render_options` and are re-exported here at their
//! historical `convert::` paths.
//!
//! # Example
//!
//! ```rust,no_run
//! use jubarte_render::convert::{PdfOptions, RevisionStyle, docx_to_pdf, docx_to_pdf_with, docx_to_png};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let docx = std::fs::read("redline.docx")?;
//!
//!     // Default options: tracked changes in the conventional colours.
//!     std::fs::write("redline.pdf", docx_to_pdf(&docx)?)?;
//!
//!     // Tracked changes drawn the way Microsoft Word prints them.
//!     let word = PdfOptions { revisions: RevisionStyle::Word, ..PdfOptions::default() };
//!     std::fs::write("redline-word.pdf", docx_to_pdf_with(&docx, word)?)?;
//!
//!     // One PNG per page, at 96 dpi.
//!     for (n, png) in docx_to_png(&docx, PdfOptions::default(), 96.0)?.iter().enumerate() {
//!         std::fs::write(format!("page-{}.png", n + 1), png)?;
//!     }
//!     Ok(())
//! }
//! ```

#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

// A5 shims: the floors' modules under the names the moved code uses, so
// `crate::<module>::…` paths compile unchanged.
pub use jubarte_dom::{namespaces, util, xmllinq};
pub use jubarte_opc::{admission, opc};
pub(crate) use jubarte_opc::{
    markup_simplifier, numbering, render_options, strict_translation, table_dom,
};

/// DOCX to PDF and PNG, with Word-faithful layout.
pub mod convert;
