// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! # jubarte
//!
//! Word-faithful `.docx` toolkit. It compares two Word documents into a
//! tracked-changes redline that opens cleanly in Microsoft Word, and it also
//! edits, inspects, checks, cleans, merges and renders them: every function
//! takes the complete package as `&[u8]` and returns a new one, with no temp
//! files and no Word or LibreOffice process.
//!
//! ## What you can do
//!
//! | Task | Start here |
//! |---|---|
//! | Compare two documents into a redline | [`document_comparer::compare_documents`]; [`comparer::WmlComparerSettings`] with [`document_comparer::compare_documents_with_settings`] for author, date, moves and detail |
//! | List, accept or reject tracked changes, all or some | [`document_comparer::get_revisions`], [`document_comparer::accept_revisions`], [`document_comparer::reject_revisions`]; one at a time with [`changes::list_changes`], [`changes::accept_changes`], [`changes::reject_changes`] |
//! | Edit a document as tracked changes | [`edit::apply_plan`] / [`edit::apply_plan_json`] with an [`edit::EditPlan`]: rewrite, insert, tables, lists, run formatting, footnotes, images, page setup, content controls, watermark, redact, settings |
//! | Read a document the way an edit addresses it | [`inspect::paragraphs`], [`inspect::summary`], [`inspect::stories`], [`inspect::controls`], [`inspect::inspect_json`], [`inspect::markdown`] |
//! | Render to PDF or PNG | [`convert::docx_to_pdf`], [`convert::docx_to_pdf_with`] and [`convert::PdfOptions`] (revision marks: [`convert::RevisionStyle`]), [`convert::docx_to_png`]; page-by-page differences with [`convert::diff_render`] |
//! | Convert a Word 97-2003 `.doc` | [`legacy_doc::doc_to_docx`], [`legacy_doc::doc_to_markdown`]: text, headings and tables |
//! | Markdown in and out | [`markdown::markdown_to_docx`] (CriticMarkup becomes tracked changes and comments), [`markdown::docx_to_markdown`], [`markdown::diff_markdown`], [`markdown::redline`] |
//! | Check that Word will open it, and repair it | [`validate::validate`], [`validate::repair`], [`validate::audit_tracked`]; triage with [`debug::report`] |
//! | Accessibility, style and structure findings | [`audit::audit_report`] |
//! | Remove authors and metadata before sending | [`scrub::scrub`] with [`scrub::ScrubOptions`]; find leftover text with [`scrub::leaks`] |
//! | Append one document after another | [`append::append_documents`] with [`append::AppendOptions`] |
//! | List comment threads | [`comments::list_comments`] |
//! | Refresh a table of contents and other fields | [`fields::update_fields`] |
//! | Bound untrusted input before it is opened | [`admission::admit`] with [`admission::InputLimits`] (every entry point above admits its input too) |
//! | Ask what this build supports | [`capabilities::capabilities_json`] |
//!
//! The [Rust guide](https://github.com/jandira-tech/jubarte-redlines/blob/main/docs/rust.md)
//! walks through each one; the same operations are in Python (`jubarte-redlines`
//! on PyPI), JavaScript (`jubarte-wasm` on npm) and the `jubarte` CLI.
//!
//! ## Example
//!
//! ```no_run
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let original = std::fs::read("original.docx")?;
//!     let modified = std::fs::read("modified.docx")?;
//!     let redline =
//!         jubarte::document_comparer::compare_documents(&original, &modified, "Reviewer")?;
//!     std::fs::write("original_v_modified.docx", &redline)?;
//!     std::fs::write("original_v_modified.pdf", jubarte::convert::docx_to_pdf(&redline)?)?;
//!     Ok(())
//! }
//! ```
//!
//! An edit plan is JSON, so an agent can write one. Each operation names its
//! paragraph (an id from [`inspect::paragraphs`]) and text that must occur
//! there exactly once; the result holds the edited document and its redline:
//!
//! ```no_run
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let contract = std::fs::read("contract.docx")?;
//!     let plan = r#"{"schema_version": 1, "author": "Reviewer", "operations": [
//!         {"kind": "replace", "paragraph": "body:p:4",
//!          "find": "thirty (30) days", "replacement": "sixty (60) days"}
//!     ]}"#;
//!     let edited = jubarte::edit::apply_plan_json(&contract, plan)?;
//!     std::fs::write("contract_clean.docx", &edited.clean)?;
//!     std::fs::write("contract_redline.docx", &edited.redline)?;
//!     Ok(())
//! }
//! ```
//!
//! ## Comparison fidelity
//!
//! The redline is Word-valid, and differences in text and formatting are
//! revisions, but the engine also normalizes some markup so the output opens
//! cleanly and matches what Word's own Compare writes. This list is the main
//! cases, not every one. Both inputs lose non-standard `w:`-namespace children
//! of `w:sdtPr` and have `mc:AlternateContent` resolved to a single branch.
//! In the default (Word-visual) mode, the original's own tracked changes are
//! flattened and compared like text, internal anchor `w:hyperlink` wrappers
//! (no `r:id`) become runs styled as hyperlinks, content controls in any
//! paragraph that carries a revision are unwrapped, some redundant default
//! spacing is stripped, and a breaking and a non-breaking space compare equal
//! ([`comparer::WmlComparerSettings::conflate_breaking_and_nonbreaking_spaces`]).
//! [`comparer::WmlComparerSettings::powertools_faithful`] skips the
//! mode-specific passes.
//!
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
//! ## Provenance
//!
//! The comparer is a Rust port of the `WmlComparer`/`DocumentComparer` engine
//! from [Docxodus](https://github.com/JSv4/Docxodus) (MIT), itself a fork of
//! Microsoft's [Open-Xml-PowerTools](https://github.com/OfficeDev/Open-Xml-PowerTools)
//! (MIT). The repository itself is AGPL-3.0-only; `LICENSES/` preserves those
//! upstream attribution texts without changing the repository license.

pub mod admission;
pub mod append;
pub mod audit;
/// Word's built-in style names.
mod builtin_styles;
/// Machine-readable manifest of what this build can do.
pub mod capabilities;
/// Tracked changes one at a time: list, accept or reject a selection.
pub mod changes;
#[cfg(feature = "cli")]
pub mod cli;
pub mod comments;
/// Core WmlComparer engine (atomize → LCS → produce → finalize).
pub mod comparer;
/// Structured comparison log (info / warning / error entries).
pub mod comparison_log;
/// Independent DOCX → PDF conversion (not LibreOffice).
pub mod convert;
/// `jubarte debug`: short Word-validity triage of a package, or two compared.
pub mod debug;
/// Byte-level package API: compare, list, accept, and reject revisions.
pub mod document_comparer;
pub mod edit;
pub mod fields;
/// Read-only paragraph/package views and the Markdown projection for agents.
pub mod inspect;
pub mod legacy_doc;
pub mod markdown;
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
pub mod scrub;
pub mod settings;
/// ISO Strict → Transitional package normalization.
pub mod strict_translation;
pub mod text_diff;
pub mod unid;
/// `jubarte self-update`: install a GitHub release, only when asked.
#[cfg(feature = "self-update")]
pub mod update;
/// Shared small utilities.
pub mod util;
pub mod validate;
/// `WmlDocument` — document bytes + lazily parsed main part.
pub mod wml_document;
/// Word's default theme part, for originals without one.
mod word_default_theme;
/// Arena DOM (`xmllinq`) used by the comparer.
pub mod xmllinq;

/// [`WmlDocument`] re-export for the common library entry point.
pub use wml_document::WmlDocument;
