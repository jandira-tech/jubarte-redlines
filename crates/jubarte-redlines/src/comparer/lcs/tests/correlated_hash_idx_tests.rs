// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::comparer::atoms::{ComparisonUnitGroup, ComparisonUnitWord, Sha1Keyed};
use crate::xmllinq::NodeId;

fn group_atoms(
    hash: &str,
    correlated: Option<&str>,
    group_type: ComparisonUnitGroupType,
    atom_count: usize,
) -> ComparisonUnit {
    let atoms: Vec<ComparisonUnitAtom> = (0..atom_count.max(1))
        .map(|i| {
            ComparisonUnitAtom::new(
                NodeId(i as u32),
                Vec::<NodeId>::new(),
                format!("atom-{hash}-{i}"),
            )
        })
        .collect();
    // One word holding all atoms so descendant_content_atoms_count == atom_count.
    let word = ComparisonUnit::Word(ComparisonUnitWord::new(atoms));
    ComparisonUnit::Group(ComparisonUnitGroup {
        correlation_status: CorrelationStatus::Nil,
        group_type,
        contents: vec![word],
        level: 0,
        sha1: Sha1Keyed::new(hash.to_string()),
        correlated_sha1_hash: correlated.map(|s| s.to_string()),
        structure_sha1_hash: None,
        atom_count_memo: std::cell::Cell::new(usize::MAX),
    })
}

fn run_eq(a: Option<CorrelatedHashRun>, b: Option<CorrelatedHashRun>) {
    assert_eq!(
        a.map(|r| (r.left_start, r.right_start, r.len)),
        b.map(|r| (r.left_start, r.right_start, r.len)),
    );
}

#[test]
fn indexed_matches_scan_on_owned_fixture() {
    // Reuse the LCS-OWN multi-match shape (len>3 ⇒ threshold always on).
    let left = vec![
        group_atoms("lp", Some("lo"), ComparisonUnitGroupType::Paragraph, 1),
        group_atoms("l0", Some("m0"), ComparisonUnitGroupType::Paragraph, 1),
        group_atoms("l1", Some("m1"), ComparisonUnitGroupType::Paragraph, 1),
        group_atoms("l2", Some("m2"), ComparisonUnitGroupType::Paragraph, 1),
        group_atoms("l3", Some("m3"), ComparisonUnitGroupType::Paragraph, 1),
        group_atoms("ls", Some("lt"), ComparisonUnitGroupType::Paragraph, 1),
    ];
    let right = vec![
        group_atoms("rp", Some("ro"), ComparisonUnitGroupType::Paragraph, 1),
        group_atoms("r0", Some("m0"), ComparisonUnitGroupType::Paragraph, 1),
        group_atoms("r1", Some("m1"), ComparisonUnitGroupType::Paragraph, 1),
        group_atoms("r2", Some("m2"), ComparisonUnitGroupType::Paragraph, 1),
        group_atoms("r3", Some("m3"), ComparisonUnitGroupType::Paragraph, 1),
        group_atoms("rs", Some("rt"), ComparisonUnitGroupType::Paragraph, 1),
    ];
    let unknown = CorrelatedSequence::paired(CorrelationStatus::Unknown, left, right);
    run_eq(
        correlated_hash_run_scan(&unknown),
        correlated_hash_run_indexed(&unknown),
    );
    // Production path must accept.
    assert!(correlated_hash_run(&unknown).is_some());
}

#[test]
fn indexed_matches_scan_first_found_tiebreak() {
    // Two equal-length runs with identical atom totals — earliest (i1,i2) wins.
    // Left:  X A A A Y A A A
    // Right: Z A A A W A A A   (both runs length 3, 1 atom each → need >3 for auto)
    // Use 2 atoms × 4 groups so len>3 triggers without size gate.
    let mk =
        |tag: &str, corr: &str| group_atoms(tag, Some(corr), ComparisonUnitGroupType::Paragraph, 2);
    let left = vec![
        mk("lx", "x"),
        mk("la0", "a0"),
        mk("la1", "a1"),
        mk("la2", "a2"),
        mk("la3", "a3"),
        mk("ly", "y"),
        mk("lb0", "a0"),
        mk("lb1", "a1"),
        mk("lb2", "a2"),
        mk("lb3", "a3"),
    ];
    let right = vec![
        mk("rz", "z"),
        mk("ra0", "a0"),
        mk("ra1", "a1"),
        mk("ra2", "a2"),
        mk("ra3", "a3"),
        mk("rw", "w"),
        mk("rb0", "a0"),
        mk("rb1", "a1"),
        mk("rb2", "a2"),
        mk("rb3", "a3"),
    ];
    let unknown = CorrelatedSequence::paired(CorrelationStatus::Unknown, left, right);
    let scan = correlated_hash_run_scan(&unknown).expect("scan");
    let idx = correlated_hash_run_indexed(&unknown).expect("idx");
    assert_eq!(
        (scan.left_start, scan.right_start, scan.len),
        (idx.left_start, idx.right_start, idx.len)
    );
    // First run starts at left index 1 / right index 1.
    assert_eq!(scan.left_start, 1);
    assert_eq!(scan.right_start, 1);
    assert_eq!(scan.len, 4);
}

#[test]
fn indexed_matches_scan_threshold_decline_len1() {
    // Single matching group with too few atoms → decline both paths.
    let left = vec![
        group_atoms("a", Some("m"), ComparisonUnitGroupType::Paragraph, 5),
        group_atoms("b", Some("x"), ComparisonUnitGroupType::Paragraph, 5),
        group_atoms("c", Some("y"), ComparisonUnitGroupType::Paragraph, 5),
    ];
    let right = vec![
        group_atoms("d", Some("m"), ComparisonUnitGroupType::Paragraph, 5),
        group_atoms("e", Some("u"), ComparisonUnitGroupType::Paragraph, 5),
        group_atoms("f", Some("v"), ComparisonUnitGroupType::Paragraph, 5),
    ];
    let unknown = CorrelatedSequence::paired(CorrelationStatus::Unknown, left, right);
    run_eq(
        correlated_hash_run_scan(&unknown),
        correlated_hash_run_indexed(&unknown),
    );
    assert!(correlated_hash_run_scan(&unknown).is_none());
}

#[test]
fn indexed_matches_scan_random_trials() {
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
    let corrs = ["c0", "c1", "c2", "c3", "uniq"];
    let mut rng = Lcg(0xC0FF_EE42_DEAD_BEEF);
    for trial in 0..800 {
        let n = 3 + rng.below(8) as usize;
        let m = 3 + rng.below(8) as usize;
        let mut left = Vec::with_capacity(n);
        let mut right = Vec::with_capacity(m);
        for i in 0..n {
            let c = corrs[rng.below(corrs.len() as u32) as usize];
            let atoms = 1 + rng.below(20) as usize;
            left.push(group_atoms(
                &format!("L{trial}-{i}"),
                Some(c),
                ComparisonUnitGroupType::Paragraph,
                atoms,
            ));
        }
        for i in 0..m {
            let c = corrs[rng.below(corrs.len() as u32) as usize];
            let atoms = 1 + rng.below(20) as usize;
            right.push(group_atoms(
                &format!("R{trial}-{i}"),
                Some(c),
                ComparisonUnitGroupType::Paragraph,
                atoms,
            ));
        }
        // Occasionally drop correlated hash to exercise None branches.
        if rng.below(10) == 0
            && let ComparisonUnit::Group(g) = &mut left[0]
        {
            g.correlated_sha1_hash = None;
        }
        let unknown = CorrelatedSequence::paired(CorrelationStatus::Unknown, left, right);
        let scan = correlated_hash_run_scan(&unknown);
        let idx = correlated_hash_run_indexed(&unknown);
        assert_eq!(
            scan.map(|r| (r.left_start, r.right_start, r.len)),
            idx.map(|r| (r.left_start, r.right_start, r.len)),
            "trial {trial}"
        );
    }
}

#[test]
fn production_process_matches_scan_oracle_signature() {
    let left = vec![
        group_atoms("p", Some("pre"), ComparisonUnitGroupType::Paragraph, 2),
        group_atoms("0", Some("k0"), ComparisonUnitGroupType::Paragraph, 2),
        group_atoms("1", Some("k1"), ComparisonUnitGroupType::Paragraph, 2),
        group_atoms("2", Some("k2"), ComparisonUnitGroupType::Paragraph, 2),
        group_atoms("3", Some("k3"), ComparisonUnitGroupType::Paragraph, 2),
        group_atoms("s", Some("suf"), ComparisonUnitGroupType::Paragraph, 2),
    ];
    let right = vec![
        group_atoms("P", Some("PRE"), ComparisonUnitGroupType::Paragraph, 2),
        group_atoms("0", Some("k0"), ComparisonUnitGroupType::Paragraph, 2),
        group_atoms("1", Some("k1"), ComparisonUnitGroupType::Paragraph, 2),
        group_atoms("2", Some("k2"), ComparisonUnitGroupType::Paragraph, 2),
        group_atoms("3", Some("k3"), ComparisonUnitGroupType::Paragraph, 2),
        group_atoms("S", Some("SUF"), ComparisonUnitGroupType::Paragraph, 2),
    ];
    let unknown = CorrelatedSequence::paired(CorrelationStatus::Unknown, left, right);
    // Force production through indexed via process_correlated_hashes.
    let got = process_correlated_hashes(&unknown).expect("resolve");
    // Rebuild expected by temporarily using scan result coordinates.
    let run = correlated_hash_run_scan(&unknown).expect("scan run");
    assert_eq!(
        correlated_hash_run(&unknown).map(|r| (r.left_start, r.right_start, r.len)),
        Some((run.left_start, run.right_start, run.len))
    );
    assert!(!got.is_empty());
}
