// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! # jubarte-doc
//!
//! The document operations of the jubarte engine above compare: listing
//! and settling changes (`changes`), comment threads (`comments`,
//! `comment_settlement`), Word-validity checks and repair (`validate`),
//! the accessibility and structure audit (`audit`), metadata scrubbing
//! (`scrub`), appending documents (`append`), field updates (`fields`)
//! and the maintainer views (`debug`, `inspect`). The CriticMarkup
//! reader (`critic`) lives here too: it is pure text, and the markdown
//! layer above re-exports it.
//!
//! Layout needs the renderer: with the `render` feature off, the public
//! items that need layout (audit's `word_layout`, the fields update
//! API, the debug render check) do not exist.

#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

// A5 shims: the floors' and the compare product's modules under the
// names the moved code uses, so `crate::<module>::…` paths compile
// unchanged.
pub(crate) use jubarte_dom::ooxml;
pub use jubarte_dom::{ids, namespaces, util, xmllinq};
pub use jubarte_opc::{admission, opc, strict_translation};
pub(crate) use jubarte_opc::{
    borders, builtin_styles, data_stores, markup_simplifier, numbering, table_dom, validate_dom,
};
pub use jubarte_redlines::{comparer, document_comparer, revision_processor};

/// Append one document after another.
pub mod append;
/// Accessibility, style and structure findings.
pub mod audit;
/// Tracked changes one at a time: list, accept or reject a selection.
pub mod changes;
/// Thread-level receipts for revision settlement.
#[doc(hidden)]
pub mod comment_settlement;
/// Comment threads across the parts that hold them.
pub mod comments;
/// Maintainer triage views.
pub mod debug;
/// Refresh a table of contents and other fields (needs `render`).
#[cfg(feature = "render")]
pub mod fields;
/// Read-only paragraph/package views and the Markdown projection.
pub mod inspect;
/// Remove authors and metadata before sending.
pub mod scrub;
/// Check that Word will open a package, and repair it.
pub mod validate;
