// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::comparer::atoms::{ComparisonUnitWord, Sha1Keyed};

/// A bare word unit carrying a chosen content hash (key = fingerprint(hash),
/// the production invariant).
fn mk_word(hash: &str) -> ComparisonUnit {
    ComparisonUnit::Word(ComparisonUnitWord {
        correlation_status: CorrelationStatus::Nil,
        contents: Vec::new(),
        sha1: Sha1Keyed::new(hash.to_string()),
    })
}

/// A word with an EXPLICIT (hash, key) pair — used to simulate a u64
/// fingerprint collision (distinct hash strings sharing a key). Real FNV-1a
/// keys make this astronomically rare, but the 128-bit compare must still
/// reject it identically in both paths.
fn mk_word_key(hash: &str, key: u64) -> ComparisonUnit {
    ComparisonUnit::Word(ComparisonUnitWord {
        correlation_status: CorrelationStatus::Nil,
        contents: Vec::new(),
        sha1: Sha1Keyed::with_colliding_key(hash.to_string(), key),
    })
}

fn mk_seq(hashes: &[&str]) -> Vec<ComparisonUnit> {
    hashes.iter().map(|h| mk_word(h)).collect()
}

/// Tiny deterministic LCG (Numerical Recipes constants) — no external rng, no
/// time/random (both unavailable). Same seed ⇒ same sequence every run.
struct Lcg(u64);
impl Lcg {
    fn below(&mut self, n: u32) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 33) as u32) % n
    }
}

/// The `dom=None` path (content == len) exercises the full candidate ordering
/// and first-found tie-break — precisely where the indexed rewrite could
/// diverge. A 4-symbol alphabet with lengths 0..=12 ⇒ frequent equal runs and
/// ties across thousands of trials.
#[test]
fn indexed_matches_scan_random() {
    const ALPHABET: &[&str] = &["A", "B", "C", "D"];
    let mut rng = Lcg(0x9E37_79B9_7F4A_7C15);
    for trial in 0..5000 {
        let n = rng.below(13) as usize;
        let m = rng.below(13) as usize;
        let a: Vec<ComparisonUnit> = (0..n)
            .map(|_| mk_word(ALPHABET[rng.below(ALPHABET.len() as u32) as usize]))
            .collect();
        let b: Vec<ComparisonUnit> = (0..m)
            .map(|_| mk_word(ALPHABET[rng.below(ALPHABET.len() as u32) as usize]))
            .collect();
        let expect = longest_common_run_scan(None, &a, &b, None);
        let got = longest_common_run_indexed(None, &a, &b, None);
        assert_eq!(
            got,
            expect,
            "trial {trial}: indexed != scan\n a={:?}\n b={:?}",
            a.iter().map(|u| u.sha1()).collect::<Vec<_>>(),
            b.iter().map(|u| u.sha1()).collect::<Vec<_>>(),
        );
    }
}

#[test]
fn indexed_matches_scan_edge_cases() {
    let cases: &[(Vec<ComparisonUnit>, Vec<ComparisonUnit>)] = &[
        (mk_seq(&[]), mk_seq(&[])),
        (mk_seq(&["A"]), mk_seq(&[])),
        (mk_seq(&[]), mk_seq(&["A"])),
        (mk_seq(&["A"]), mk_seq(&["A"])),
        (mk_seq(&["A"]), mk_seq(&["B"])),
        (mk_seq(&["A", "A", "A"]), mk_seq(&["A", "A"])),
        (mk_seq(&["A", "B", "C"]), mk_seq(&["C", "B", "A"])),
        (mk_seq(&["A", "B", "A", "B"]), mk_seq(&["A", "B", "A", "B"])),
    ];
    for (a, b) in cases {
        assert_eq!(
            longest_common_run_indexed(None, a, b, None),
            longest_common_run_scan(None, a, b, None),
        );
    }
}

/// A forced u64-key collision (distinct hashes, shared key) must NOT be read
/// as a match by the bucket probe: the differing 128-bit fingerprints keep the
/// indexed output identical to the scan, which relies on the same compare.
#[test]
fn indexed_handles_key_collision() {
    let k = 0xDEAD_BEEF_u64;
    // "X" and "Y" pretend to collide on key k; "Z" is a genuine matching pair.
    let a = vec![mk_word_key("X", k), mk_word_key("Z", k)];
    let b = vec![mk_word_key("Y", k), mk_word_key("Z", k)];
    let got = longest_common_run_indexed(None, &a, &b, None);
    assert_eq!(got, longest_common_run_scan(None, &a, &b, None));
    assert_eq!(got, (1, 1, 1), "collision must not fabricate an X==Y match");
}
