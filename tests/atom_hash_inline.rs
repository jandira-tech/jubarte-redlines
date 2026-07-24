// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! ATOM-HASH-INLINE-01 — the `AtomHash` contract.
//!
//! The atom content hash moved from a heap `String` (40-char lowercase hex) to an
//! inline, `Copy` `AtomHash([u8; 20])` (the raw SHA-1 digest) so that cloning a
//! freshly-atomized atom allocates nothing (see `perf_atom_clone_allocs.rs`).
//!
//! The representation change MUST be invisible to behavior. These tests pin the
//! bijection that makes it so:
//!   - equal content ⇒ equal hash; distinct content ⇒ distinct hash;
//!   - `from_hex(sha1_hex(x)) == of_bytes(x)` — the precomputed-`pt:SHA1Hash`
//!     path (decode a stamped hex) must land on the SAME digest as a freshly
//!     hashed identical atom (`of_bytes`), or precomputed and fresh atoms of
//!     identical content would stop correlating Equal;
//!   - `to_hex_string()` reproduces `sha1_hex(x)` exactly;
//!   - `ComparisonUnitWord::new` produces a byte-identical word hash to the old
//!     "SHA-1 of the concatenation of the atoms' hex digests" path.

use jubarte::comparer::atoms::{AtomHash, ComparisonUnitAtom, ComparisonUnitWord};
use jubarte::util::sha1::{sha1_hex, sha1_hex_parts};
use jubarte::xmllinq::NodeId;

#[test]
fn atom_hash_is_copy_and_inline() {
    let h = AtomHash::of_bytes(b"abc");
    let h2 = h; // Copy — this must not move `h`
    assert_eq!(h, h2, "copied hash equals the original");
    assert_eq!(h, h, "the original is still usable after the copy");
    // Inline: exactly the 20-byte SHA-1 digest, no pointer/len/cap.
    assert_eq!(std::mem::size_of::<AtomHash>(), 20);
}

#[test]
fn equal_content_equal_hash_distinct_content_distinct() {
    assert_eq!(AtomHash::of_bytes(b"foo"), AtomHash::of_bytes(b"foo"));
    assert_ne!(AtomHash::of_bytes(b"foo"), AtomHash::of_bytes(b"bar"));
}

#[test]
fn from_hex_decodes_and_cross_correlates_with_of_bytes() {
    // The pt:SHA1Hash attribute stamped by PreProcess is `sha1_hex(content)`.
    // Reading it via from_hex must land on the same digest as of_bytes(content),
    // otherwise a precomputed atom and a fresh identical atom stop matching.
    let content = "wt-hello";
    let fresh = AtomHash::of_bytes(content.as_bytes());
    let stamped_hex = sha1_hex(content);
    let from_attr = AtomHash::from_hex(&stamped_hex);
    assert_eq!(
        fresh, from_attr,
        "from_hex(sha1_hex(x)) must equal of_bytes(x)"
    );
}

#[test]
fn to_hex_string_roundtrips_sha1_hex() {
    for content in ["", "roundtrip", "café ☕", "PREDEL|deadbeef"] {
        let h = AtomHash::of_bytes(content.as_bytes());
        assert_eq!(
            h.to_hex_string(),
            sha1_hex(content),
            "to_hex_string must reproduce sha1_hex for {content:?}"
        );
    }
}

#[test]
fn from_hex_to_hex_is_identity() {
    let hex = sha1_hex("some-atom-content");
    assert_eq!(AtomHash::from_hex(&hex).to_hex_string(), hex);
}

#[test]
fn word_hash_byte_identical_to_hex_concat_path() {
    // ComparisonUnitWord::new must yield the SAME sha1_hash string as the legacy
    // path: SHA-1 over the concatenation of the atoms' 40-char hex digests.
    let a1 = AtomHash::of_bytes(b"alpha");
    let a2 = AtomHash::of_bytes(b"beta");
    let atoms = vec![
        ComparisonUnitAtom::new(NodeId(0), Vec::<NodeId>::new(), a1),
        ComparisonUnitAtom::new(NodeId(0), Vec::<NodeId>::new(), a2),
    ];
    let word = ComparisonUnitWord::new(atoms);
    let expected = sha1_hex_parts([a1.to_hex_string().as_str(), a2.to_hex_string().as_str()]);
    assert_eq!(
        word.sha1_hash, expected,
        "word hash must be byte-identical to the hex-concatenation path"
    );
}

#[test]
fn atom_new_accepts_opaque_str_sentinels() {
    // Unit tests across the suite build atoms with opaque string sentinels
    // ("h", "x", "deadbeef", …) whose only contract is equality. Same sentinel
    // ⇒ equal hash; different sentinel ⇒ different hash.
    let a = ComparisonUnitAtom::new(NodeId(0), Vec::<NodeId>::new(), "h");
    let b = ComparisonUnitAtom::new(NodeId(1), Vec::<NodeId>::new(), "h");
    let c = ComparisonUnitAtom::new(NodeId(2), Vec::<NodeId>::new(), "x");
    assert_eq!(a.sha1_hash, b.sha1_hash);
    assert_ne!(a.sha1_hash, c.sha1_hash);
}

/// Struct-size regression guard. `ComparisonUnitAtom` is held in high
/// multiplicity during correlation (millions of atoms × several simultaneous
/// copies), so its *size* — not the hash representation — dominates the peak
/// footprint (MEM-PROFILE-01). The inline `AtomHash` kept the hash field from
/// bloating it (20 bytes vs the former 24-byte `String`); this guard pins the
/// current size so an accidental field addition can't silently grow the peak.
#[test]
fn atom_struct_size_is_pinned() {
    let size = std::mem::size_of::<ComparisonUnitAtom>();
    assert!(
        size <= 200,
        "ComparisonUnitAtom grew to {size} bytes (was 200); it is held in high \
         multiplicity during correlation, so a larger struct raises the peak \
         footprint — box the new cold field instead of inlining it"
    );
    // The hash contributes only its inline 20 bytes now (no heap `String`).
    assert_eq!(std::mem::size_of::<AtomHash>(), 20);
}
