// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A unit's cached LCS keys always equal the fingerprints of the hash they are
//! cached for. The LCS consults them ahead of the string (`sha1_key128` alone
//! decides common-run equality), so a key left pointing at an old hash would
//! make units that should correlate silently miss. `Sha1Keyed` derives both
//! keys in `new`/`set_hash`; these tests pin that through the constructors and
//! accessors the engine actually uses.

use jubarte::comparer::atoms::{ComparisonUnit, ComparisonUnitAtom, ComparisonUnitWord, Sha1Keyed};
use jubarte::util::sha1::{sha1_fingerprint, sha1_fingerprint128};
use jubarte::xmllinq::NodeId;

fn atom(tag: &str) -> ComparisonUnitAtom {
    ComparisonUnitAtom::new(NodeId(1), vec![], format!("hash-{tag}"))
}

/// Empty, all-zero and all-f digests, the SHA-1 of "", and the non-hex
/// sentinels tests use as hashes.
fn hashes() -> Vec<String> {
    vec![
        String::new(),
        "0".repeat(40),
        "f".repeat(40),
        "PARAHASH".to_string(),
        "h".to_string(),
        "da39a3ee5e6b4b0d3255bfef95601890afd80709".to_string(),
        "ünïcödé-sentinel".to_string(),
    ]
}

fn assert_keys_follow(k: &Sha1Keyed, h: &str) {
    assert_eq!(k.hash(), h, "hash must round-trip verbatim");
    assert_eq!(k.key(), sha1_fingerprint(h), "u64 key of {h:?}");
    assert_eq!(k.key128(), sha1_fingerprint128(h), "128-bit key of {h:?}");
}

#[test]
fn new_and_set_hash_derive_both_keys() {
    let mut moving = Sha1Keyed::new("before".to_string());
    for h in hashes() {
        assert_keys_follow(&Sha1Keyed::new(h.clone()), &h);
        moving.set_hash(h.clone());
        assert_keys_follow(&moving, &h);
        assert_keys_follow(&moving.clone(), &h);
    }
}

#[test]
fn words_hand_the_lcs_consistent_keys() {
    for n in [0usize, 1, 2, 5, 17] {
        let word = ComparisonUnitWord::new((0..n).map(|i| atom(&i.to_string())).collect());
        let hash = word.sha1.hash().to_string();
        assert_keys_follow(&word.sha1, &hash);
        let unit = ComparisonUnit::Word(word);
        assert_eq!(unit.sha1(), hash);
        assert_eq!(unit.sha1_key(), sha1_fingerprint(&hash), "{n} atoms");
        assert_eq!(unit.sha1_key128(), sha1_fingerprint128(&hash), "{n} atoms");
    }
}
