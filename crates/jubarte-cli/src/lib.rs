// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! # jubarte-cli
//!
//! The jubarte engine's command line: every operation of the library behind
//! one clap surface - compare, convert, changes, accept, reject, edit, read,
//! validate, audit, scrub, append, comments, fields, debug and `self-update`
//! (behind its feature). `build.rs` stamps `--version` with the source
//! commit when the crate is built from this repository, and nothing when it
//! is built from an unpacked package inside another repository.

#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

// A5 shims: every layer below under the names the moved code uses.
pub use jubarte_doc::comment_settlement;
pub use jubarte_doc::{append, audit, changes, comments, debug, fields, inspect, scrub, validate};
pub use jubarte_dom::{ids, namespaces, perf, unid, util, xmllinq};
pub use jubarte_edit::{capabilities, edit, markdown, publication, result, text_diff};
pub use jubarte_markdown::legacy_doc;
pub use jubarte_opc::{admission, opc, settings, strict_translation};
pub use jubarte_redlines::{document_comparer, revision_processor, wml_document};
#[cfg(feature = "render")]
pub use jubarte_render;

/// The `jubarte` command's parser and dispatch.
pub mod cli;
/// The typed inline CLI actions, compiled into plan contracts.
pub mod inline;
/// `jubarte self-update`: install a GitHub release, only when asked.
#[cfg(feature = "self-update")]
pub mod update;
