// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! HASH-02 — stream atom-hash bytes into SHA-1 instead of concatenating a String.
//!
//! `ComparisonUnitWord::new` historically built `concat = atom_hashes.join("")`
//! then `sha1_hex(&concat)`. Streaming each 40-byte hex digest into the hasher
//! must produce the *exact* same lowercase hex digest (byte-identical input to
//! SHA-1). A wrong stream order, missing piece, or double-hex would fail these.

use jubarte::util::sha1::{sha1_hex, sha1_hex_parts};

/// Empty parts: same as hashing the empty string.
#[test]
fn stream_empty_equals_concat_empty() {
    assert_eq!(sha1_hex_parts(std::iter::empty::<&str>()), sha1_hex(""));
    assert_eq!(sha1_hex_parts([""]), sha1_hex(""));
}

/// Single part: streaming is a pure no-op relative to the existing path.
#[test]
fn stream_single_part_equals_sha1_hex() {
    for s in ["", "a", "abc", "deadbeef", &"x".repeat(40), &"0".repeat(80)] {
        assert_eq!(
            sha1_hex_parts([s]),
            sha1_hex(s),
            "single-part stream must match sha1_hex for {s:?}"
        );
    }
}

/// Multi-part: stream of pieces must equal SHA-1 of their concatenation.
#[test]
fn stream_multipart_equals_concat_then_hash() {
    let cases: &[&[&str]] = &[
        &["aa", "bb"],
        &["", "aa", ""],
        &["0123456789abcdef0123456789abcdef01234567", "89abcdef"],
        &["a", "b", "c", "d", "e"],
        // realistic word: several 40-char atom hex digests
        &[
            "a9993e364706816aba3e25717850c26c9cd0d89d",
            "da39a3ee5e6b4b0d3255bfef95601890afd80709",
            "2fd4e1c67a2d28fced849ee1bb76e7391b93eb12",
        ],
    ];
    for parts in cases {
        let concat: String = parts.iter().copied().collect();
        assert_eq!(
            sha1_hex_parts(parts.iter().copied()),
            sha1_hex(&concat),
            "stream != concat for parts={parts:?}"
        );
    }
}

/// Exhaustive small generator: every 3-way split of a short alphabet string.
#[test]
fn stream_generated_splits_match_concat() {
    let alphabet = b"0123456789abcdef";
    for n in 0..=6 {
        let mut raw = Vec::with_capacity(n);
        for i in 0..n {
            raw.push(alphabet[i % alphabet.len()]);
        }
        let full = String::from_utf8(raw).unwrap();
        // all cut positions (i, j) with 0 <= i <= j <= n
        for i in 0..=n {
            for j in i..=n {
                let a = &full[..i];
                let b = &full[i..j];
                let c = &full[j..];
                assert_eq!(
                    sha1_hex_parts([a, b, c]),
                    sha1_hex(&full),
                    "n={n} cuts=({i},{j}) full={full:?}"
                );
            }
        }
    }
}
