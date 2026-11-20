// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! # jubarte-opc
//!
//! The package floor of the jubarte redline engine: the OPC adapter
//! (`opc::PartFs`), input admission with its budgets and refusals
//! (`admission`), markup simplification and strict-to-transitional
//! normalization (`markup_simplifier`, `strict_translation`), settings
//! application (`settings`), and the shared Word tables every layer above
//! reads (default theme, built-in styles, borders, data stores, numbering,
//! table DOM questions, Word-validity predicates, and the pure render
//! option types).
//!
//! Everything here speaks packages and parts; document comparison,
//! rendering, editing and the CLI live in the higher jubarte crates.
//!
//! # Example
//!
//! ```rust,no_run
//! use jubarte_opc::admission::{InputLimits, admit};
//! use jubarte_opc::opc::PartFs;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let bytes = std::fs::read("contract.docx")?;
//!     // Bound the ZIP before anything inflates it.
//!     admit(&bytes, InputLimits::default())?;
//!     let package = PartFs::open(&bytes)?;
//!     let main = package.main_document_part().ok_or("no main document part")?;
//!     println!("main document: {main}");
//!     for part in package.parts() {
//!         println!("{part}");
//!     }
//!     Ok(())
//! }
//! ```

#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

// Shims (plan A5): the dom layer's modules under the names the moved code
// already uses, so `crate::<module>::…` paths compile unchanged.
pub(crate) use jubarte_dom::limits;
pub use jubarte_dom::{ids, namespaces, perf, unid, util, xmllinq};

/// Input admission: budgets, refusal reasons, and the limits types the
/// bindings and CLI expose.
pub mod admission;
/// Word table border aliases and the borders part of `w:tblPr`
/// normalization. Private in the facade.
#[doc(hidden)]
pub mod borders;
/// Word's built-in style ids and names. Private in the facade.
#[doc(hidden)]
pub mod builtin_styles;
/// Custom XML data-store lookup and the writer's missing-store builder.
/// Private in the facade.
#[doc(hidden)]
pub mod data_stores;
/// Markup simplification (PowerTools `MarkupSimplifier` port).
pub mod markup_simplifier;
/// List numbering shared by every layer above the package floor.
/// Private in the facade.
#[doc(hidden)]
pub mod numbering;
/// Open Packaging Conventions adapter (`PartFs`).
pub mod opc;
/// The pure option types of a DOCX -> PDF conversion. Private in the
/// facade; the renderer re-exports its types publicly.
#[doc(hidden)]
pub mod render_options;
/// Settings application (`w:settings`).
pub mod settings;
/// ISO Strict → Transitional package normalization.
pub mod strict_translation;
/// Word table structure questions a DOM walk answers. Private in the
/// facade.
#[doc(hidden)]
pub mod table_dom;
/// Pure DOM structural predicates shared by validate and the comparers.
/// Private in the facade.
#[doc(hidden)]
pub mod validate_dom;
/// Word's default theme part, for originals without one. Private in the
/// facade.
#[doc(hidden)]
pub mod word_default_theme;
