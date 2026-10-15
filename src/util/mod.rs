// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Small shared utilities, ported from `PtUtil.ts` (M1.6).
//!
//! Three submodules plus re-exports at this path:
//!
//! - [`sha1`]: SHA-1 hex helpers, FNV-1a 64 / 128 fingerprints, and
//!   hex encoding helpers used by the LCS and atom-hash stream.
//! - `group_adjacent`: the LINQ `GroupAdjacent` port — runs of items
//!   sharing a key.
//! - `words`: a word tokenizer that understands combining marks,
//!   zero-width joiners, and skin-tone modifiers (so a decomposed
//!   `cafe\u{301}` is one word like its composed form, and emoji
//!   sequences stay together).
//!
//! The functions here back the comparer's LCS hot path and the
//! Markdown projection. Nothing in this module is CLI-specific, but the
//! `jubarte` CLI calls into `sha1_hex` (block hash) and
//! `word_tokens` (diff mark rendering).

pub(crate) mod datetime;
pub mod group_adjacent;
pub mod sha1;
pub mod words;

pub use group_adjacent::group_adjacent;
pub use sha1::{fnv1a_64, fnv1a_128, sha1_hex, sha1_hex_bytes, sha1_hex_parts};
pub use words::word_tokens;

/// Former name of [`sha1::fnv1a_64`] at this path. It never computed SHA-1.
#[deprecated(since = "0.10.2", note = "renamed to `fnv1a_64`; this was never SHA-1")]
pub fn sha1_fingerprint(s: &str) -> u64 {
    sha1::fnv1a_64(s)
}
