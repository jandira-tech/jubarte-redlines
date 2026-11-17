// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Tracked-change editing. The engine lives in the jubarte-edit crate;
//! the inline CLI actions stay here until T4.8 moves them with the CLI.

pub use jubarte_edit::edit::*;

#[cfg(feature = "cli")]
pub mod inline;
