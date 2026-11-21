// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! The composed markdown module (A6): the markdown crate's module plus
//! this crate's patch, redline and unified modules, reproducing the
//! facade's historical `markdown` from one place.

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
