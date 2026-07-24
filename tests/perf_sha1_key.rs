//! PR1 — fixed-width fingerprint keys for the LCS hot path.
//!
//! These lock the *contract* of the `u64` fingerprint used as a pre-filter in
//! `longest_common_run`. Correctness of the redline itself is guarded by the
//! full corpus suite (byte-identity) + the parity ladder; here we only pin the
//! two properties the pre-filter relies on:
//!
//!   1. Soundness — equal hash string ⟹ equal key (so `key_eq && str_eq` never
//!      wrongly rejects a true match). Includes the empty hash, which real
//!      groups carry when `pt:SHA1Hash` is absent (`units.rs` `unwrap_or_default`).
//!   2. Discrimination — distinct hashes map to distinct keys, else the filter
//!      is useless. A constant/stub fingerprint FAILS this (the RED assertion).

use std::collections::HashSet;

use jubarte::util::sha1::{sha1_fingerprint, sha1_hex};

#[test]
fn fingerprint_is_deterministic() {
    assert_eq!(sha1_fingerprint("deadbeef"), sha1_fingerprint("deadbeef"));
    let h = sha1_hex("hello world");
    assert_eq!(sha1_fingerprint(&h), sha1_fingerprint(&h));
}

/// Soundness: equal strings ⇒ equal keys, for a representative spread including
/// the empty hash and a non-hex string (groups without a stamped hash).
#[test]
fn fingerprint_equal_hash_equal_key() {
    for s in ["", "a", "not-a-40-char-hash", &sha1_hex("x")] {
        let owned = s.to_string();
        assert_eq!(
            sha1_fingerprint(s),
            sha1_fingerprint(&owned),
            "equal strings must fingerprint equal (s = {s:?})"
        );
    }
}

/// Discrimination: 5000 distinct SHA-1 hex strings must map to 5000 distinct
/// keys (zero collisions on this batch). The stub returns a constant → this
/// collapses to 1 unique key and the assertion fails for the right reason.
#[test]
fn fingerprint_discriminates_distinct_hashes() {
    let hashes: Vec<String> = (0..5000).map(|i| sha1_hex(&format!("unit-{i}"))).collect();
    let keys: HashSet<u64> = hashes.iter().map(|h| sha1_fingerprint(h)).collect();
    assert_eq!(
        keys.len(),
        hashes.len(),
        "fingerprint collided on distinct hashes: {} keys for {} inputs",
        keys.len(),
        hashes.len()
    );
}
