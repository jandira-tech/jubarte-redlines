// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Utilities ported from `PtUtil.ts` — M1.6.

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
