// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! # jubarte
//!
//! Lossless DOCX redline engine: compare two Word documents and produce a
//! tracked-changes (redline) `.docx` — the original document with every
//! difference against the modified one expressed as Word revisions
//! (insertions, deletions, moves, and format changes) — that opens cleanly
//! in Microsoft Word. Also lists, accepts, and rejects tracked revisions.
//!
//! ## Example
//!
//! ```no_run
//! let original = std::fs::read("original.docx").unwrap();
//! let modified = std::fs::read("modified.docx").unwrap();
//! let redline =
//!     jubarte::document_comparer::compare_documents(&original, &modified, "Reviewer")
//!         .expect("compare");
//! std::fs::write("original_v_modified.docx", &redline).unwrap();
//! ```
//!
//! For author/date/detail-threshold control, build a
//! [`comparer::WmlComparerSettings`] and call
//! [`document_comparer::compare_documents_with_settings`]. To inspect a
//! redline, use [`document_comparer::get_revisions`]; to flatten one, use
//! [`document_comparer::accept_revisions`] / [`document_comparer::reject_revisions`].
//!
#![warn(missing_docs)]
//! ## Provenance
//!
//! The comparer is a Rust port of the `WmlComparer`/`DocumentComparer` engine
//! from [Docxodus](https://github.com/JSv4/Docxodus) (MIT), itself a fork of
//! Microsoft's [Open-Xml-PowerTools](https://github.com/OfficeDev/Open-Xml-PowerTools)
//! (MIT). The repository itself is AGPL-3.0-only; `LICENSES/` preserves those
//! upstream attribution texts without changing the repository license.

/// Core WmlComparer engine (atomize → LCS → produce → finalize).
pub mod comparer;
/// Structured comparison log (info / warning / error entries).
pub mod comparison_log;
/// Byte-level package API: compare, list, accept, and reject revisions.
pub mod document_comparer;
/// Markup simplification (PowerTools `MarkupSimplifier` port).
pub mod markup_simplifier;
/// WordprocessingML and related namespace / `XName` constants.
pub mod namespaces;
/// Open Packaging Conventions adapter (`PartFs`).
pub mod opc;
/// P0-LAB-01 stage counters/timers — no-ops unless `perf-profile` is enabled.
pub mod perf;
/// Accept / reject tracked revisions across a package.
pub mod revision_processor;
/// ISO Strict → Transitional package normalization.
pub mod strict_translation;
/// Unique id helpers for revision markup.
pub mod unid;
/// Shared small utilities.
pub mod util;
/// `WmlDocument` — document bytes + lazily parsed main part.
pub mod wml_document;
/// Arena DOM (`xmllinq`) used by the comparer.
pub mod xmllinq;

/// [`WmlDocument`] re-export for the common library entry point.
pub use wml_document::WmlDocument;

#[cfg(test)]
mod smoke {
    #[test]
    fn crate_builds() {
        assert_eq!(2 + 2, 4);
    }
}
