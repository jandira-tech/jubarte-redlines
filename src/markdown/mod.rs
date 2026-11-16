// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Markdown in and out of Word documents. The bulk lives in the
//! jubarte-markdown crate; the patch, redline and unified modules are
//! edit-layer files composed back here (T4.7 moves them on).

pub use jubarte_markdown::markdown::*;

mod patch;
mod redline;
mod unified;

pub use patch::{Patched, apply_markdown, apply_markdown_strict};
pub use redline::{RedlineOptions, Source, redline};
pub use unified::{
    Attribution, Change, Comment, DEFAULT_COLUMNS, Hunk, Locator, Patch, PatchOptions,
    patch_critic, patch_documents, patch_markdown, patch_own_changes, patch_redline,
};
