// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

/// A story whose every sequence is deleted has no revised final mark to
/// pair: the sequences stay as they are.
#[test]
fn a_wholly_deleted_story_keeps_its_sequences() {
    let dom = Dom::new();
    let mut seqs = vec![
        CorrelatedSequence::deleted(Vec::new()),
        CorrelatedSequence::deleted(Vec::new()),
    ];
    pair_story_final_marks(&dom, &mut seqs);
    assert_eq!(seqs.len(), 2);
    assert!(
        seqs.iter()
            .all(|s| s.correlation_status == CorrelationStatus::Deleted && s.com_units_2.is_none())
    );
}
