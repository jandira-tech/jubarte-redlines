// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! # jubarte-markdown
//!
//! Markdown in and out of Word documents: `markdown::markdown_to_docx`
//! writes CriticMarkup as tracked changes and comments, the readers
//! (`view_notes`, `view_stories`, `view_styles`, `from_docx`) project a
//! package to the agent view's Markdown, and `markdown::diff_markdown`
//! diffs two documents as pandiff-style CriticMarkup. `legacy_doc` reads
//! Word 97-2003 `.doc` files.
//!
//! The patch/redline/unified modules stay with the edit layer (T4.7);
//! the facade composes them back into `jubarte::markdown`. With the
//! `render` feature off, the layout-dependent items (`page_texts`,
//! `with_page_markers`, page markers in reads) do not exist.

#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

// A5 shims: the floors', the compare product's and the doc layer's
// modules under the names the moved code uses.
pub use jubarte_doc::{changes, comments, debug, inspect, validate};
pub(crate) use jubarte_dom::ooxml;
pub use jubarte_dom::{ids, namespaces, util, xmllinq};
pub use jubarte_opc::{admission, opc, settings, strict_translation};
pub(crate) use jubarte_opc::{borders, markup_simplifier, numbering, word_default_theme};
pub use jubarte_redlines::{comparer, document_comparer};

/// Word 97-2003 `.doc` reading: text, headings and tables.
pub mod legacy_doc;
/// Markdown in and out of Word documents.
pub mod markdown;
