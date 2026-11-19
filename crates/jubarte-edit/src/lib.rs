// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! # jubarte-edit
//!
//! The edit product of the jubarte engine: `edit::apply_plan` /
//! `apply_plan_json` write an edit plan as tracked changes (rewrite,
//! insert, tables, lists, run formatting, footnotes, images, page setup,
//! content controls, watermark, redact, settings, field updates),
//! `text_diff` renders text diffs as the agent view prints them,
//! `expect` checks plans against their schema, `capabilities` names what
//! the bindings expose, and `publication` stages host filesystem output.
//! The composed `markdown` module adds the patch, redline and unified
//! views to the markdown crate's reader and writer.
//!
//! # Example
//!
//! ```rust,no_run
//! use jubarte_edit::edit::apply_plan_json;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let source = std::fs::read("contract.docx")?;
//!     // Paragraph ids (`p12`) are the ones `jubarte read contract.docx` prints.
//!     let plan = r#"{
//!         "schema_version": 1,
//!         "author": "Reviewer",
//!         "operations": [{
//!             "id": "term",
//!             "kind": "replace",
//!             "paragraph": "p12",
//!             "find": "two years",
//!             "replacement": "three years"
//!         }]
//!     }"#;
//!     let result = apply_plan_json(&source, plan)?;
//!     std::fs::write("contract_redline.docx", &result.redline)?;
//!     std::fs::write("contract_clean.docx", &result.clean)?;
//!     Ok(())
//! }
//! ```

#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

// A5 shims: every layer below under the names the moved code uses.
#[cfg(feature = "render")]
pub(crate) use jubarte_doc::fields;
pub(crate) use jubarte_doc::{audit, scrub};
pub use jubarte_doc::{changes, comment_settlement, comments, debug, inspect, validate};
pub(crate) use jubarte_dom::ooxml;
pub use jubarte_dom::{ids, namespaces, util, xmllinq};
pub use jubarte_opc::{admission, opc, settings, strict_translation};
pub mod markdown;
pub use jubarte_redlines::{comparer, document_comparer, revision_processor};

/// What the bindings expose, feature by feature.
pub mod capabilities;
/// Tracked-change editing: plans in, redlines and clean copies out.
pub mod edit;
/// The `--expect` plan checker.
pub mod expect;
/// Host filesystem publication, capability-confined and staged.
/// Host filesystem publication is unavailable to browser/WASM byte APIs.
#[cfg(not(target_arch = "wasm32"))]
pub mod publication;
/// Stable result codes the bindings print.
pub mod result;
/// Text diff views (agent view and CriticMarkup).
pub mod text_diff;
