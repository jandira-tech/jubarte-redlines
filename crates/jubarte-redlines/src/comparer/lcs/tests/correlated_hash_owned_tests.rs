// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;
use crate::comparer::atoms::{ComparisonUnitGroup, ComparisonUnitWord, Sha1Keyed};
use crate::xmllinq::NodeId;

fn group(hash: &str, correlated: &str) -> ComparisonUnit {
    let atom = ComparisonUnitAtom::new(NodeId(0), Vec::<NodeId>::new(), format!("atom-{hash}"));
    ComparisonUnit::Group(ComparisonUnitGroup {
        correlation_status: CorrelationStatus::Nil,
        group_type: ComparisonUnitGroupType::Paragraph,
        contents: vec![ComparisonUnit::Word(ComparisonUnitWord::new(vec![atom]))],
        level: 0,
        sha1: Sha1Keyed::new(hash.to_string()),
        correlated_sha1_hash: Some(correlated.to_string()),
        structure_sha1_hash: None,
        atom_count_memo: std::cell::Cell::new(usize::MAX),
    })
}

fn correlated_unknown() -> CorrelatedSequence {
    let left = vec![
        group("left-prefix", "left-only"),
        group("left-0", "match-0"),
        group("left-1", "match-1"),
        group("left-2", "match-2"),
        group("left-3", "match-3"),
        group("left-suffix", "left-tail"),
    ];
    let right = vec![
        group("right-prefix", "right-only"),
        group("right-0", "match-0"),
        group("right-1", "match-1"),
        group("right-2", "match-2"),
        group("right-3", "match-3"),
        group("right-suffix", "right-tail"),
    ];
    CorrelatedSequence::paired(CorrelationStatus::Unknown, left, right)
}

fn signature(
    sequences: &[CorrelatedSequence],
) -> Vec<(CorrelationStatus, Vec<String>, Vec<String>)> {
    sequences
        .iter()
        .map(|sequence| {
            let left = sequence
                .com_units_1
                .as_deref()
                .unwrap_or_default()
                .iter()
                .map(|unit| unit.sha1().to_string())
                .collect();
            let right = sequence
                .com_units_2
                .as_deref()
                .unwrap_or_default()
                .iter()
                .map(|unit| unit.sha1().to_string())
                .collect();
            (sequence.correlation_status, left, right)
        })
        .collect()
}

fn unit_string_buffers(sequences: &[CorrelatedSequence]) -> Vec<usize> {
    let mut pointers: Vec<usize> = sequences
        .iter()
        .flat_map(|sequence| {
            sequence
                .com_units_1
                .iter()
                .chain(sequence.com_units_2.iter())
                .flat_map(|units| units.iter())
        })
        .map(|unit| unit.sha1().as_ptr() as usize)
        .collect();
    pointers.sort_unstable();
    pointers
}

#[test]
fn owned_correlated_hash_resolution_matches_reference_output() {
    let unknown = correlated_unknown();
    let expected = process_correlated_hashes(&unknown).expect("reference resolves");

    let actual = process_correlated_hashes_owned(unknown).expect("owned path resolves");

    assert_eq!(signature(&actual), signature(&expected));
}

#[test]
fn owned_correlated_hash_resolution_moves_unit_buffers() {
    let unknown = correlated_unknown();
    let original_buffers = unit_string_buffers(std::slice::from_ref(&unknown));

    let actual = process_correlated_hashes_owned(unknown).expect("owned path resolves");

    assert_eq!(unit_string_buffers(&actual), original_buffers);
}
