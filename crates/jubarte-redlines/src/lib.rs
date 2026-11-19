// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! # jubarte-redlines
//!
//! The compare product of the jubarte engine: two Word documents in, a
//! tracked-changes redline that opens cleanly in Microsoft Word out.
//! `document_comparer::compare_documents` is the one-call entry point;
//! `comparer::WmlComparerSettings` tunes author, date, moves and detail;
//! `WmlDocument` carries document bytes with their lazily parsed main
//! part; `revision_processor` accepts and rejects; `comparison_log`
//! records what a comparison did.
//!
//! The lower package and DOM floors are re-exported under their historical
//! names (`admission`, `opc`, `strict_translation`, `xmllinq`,
//! `namespaces`), so a standalone consumer needs this crate alone.
//!
//! # Example
//!
//! ```rust,no_run
//! use jubarte_redlines::document_comparer::compare_documents;
//! use jubarte_redlines::opc::PartFs;
//! use jubarte_redlines::revision_processor::accept_revisions_package;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let original = std::fs::read("original.docx")?;
//!     let modified = std::fs::read("modified.docx")?;
//!
//!     // A tracked-changes redline that opens cleanly in Microsoft Word.
//!     let redline = compare_documents(&original, &modified, "Reviewer")?;
//!     std::fs::write("redline.docx", &redline)?;
//!
//!     // Accepting every revision gives the modified text back.
//!     let mut package = PartFs::open(&redline)?;
//!     accept_revisions_package(&mut package);
//!     std::fs::write("accepted.docx", package.to_zip()?)?;
//!     Ok(())
//! }
//! ```

#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

// A2 re-exports: one dependency is all a standalone consumer needs.
pub use jubarte_dom::{namespaces, xmllinq};
pub use jubarte_opc::{admission, opc, strict_translation};

// A5 shims: the floors' modules under the names the moved code uses, so
// `crate::<module>::…` paths compile unchanged.
pub(crate) use jubarte_dom::{limits, ooxml, perf, unid, util};
pub(crate) use jubarte_opc::{builtin_styles, markup_simplifier, validate_dom, word_default_theme};

/// The comparison core: atomizing, matching, and the redline writer.
pub mod comparer;
/// What a comparison logged, as the CLI prints it.
pub mod comparison_log;
/// One-call document comparison and revision accept/reject.
pub mod document_comparer;
/// Accept and reject tracked revisions across a package.
pub mod revision_processor;
/// Document builders shared by the sweep tests.
#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod test_docs;
/// `WmlDocument`: document bytes plus the lazily parsed main part.
pub mod wml_document;

pub use wml_document::WmlDocument;
