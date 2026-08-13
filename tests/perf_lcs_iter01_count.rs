// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! LCS-ITER-01 / 1f0ab33 fix — non-allocating atom count must equal
//! `descendant_atoms().len()`.
//!
//! Regression: counting a Word as `1` (instead of its atom cardinality) under-
//! counts multi-atom words and flips LCS correlation thresholds on dense input
//! (pdense 15k hung for minutes vs ~25 s).

use jubarte::comparer::atoms::{
    ComparisonUnit, ComparisonUnitAtom, ComparisonUnitGroup, ComparisonUnitWord,
};
use jubarte::comparer::{ComparisonUnitGroupType, CorrelationStatus};
use jubarte::util::sha1::sha1_fingerprint;
use jubarte::xmllinq::NodeId;

fn atom(tag: &str) -> ComparisonUnitAtom {
    ComparisonUnitAtom::new(NodeId(1), vec![], format!("hash-{tag}"))
}

fn group(gt: ComparisonUnitGroupType, contents: Vec<ComparisonUnit>) -> ComparisonUnit {
    ComparisonUnit::Group(ComparisonUnitGroup {
        correlation_status: CorrelationStatus::Nil,
        group_type: gt,
        contents,
        level: 0,
        sha1_key: sha1_fingerprint(""),
        sha1_key128: jubarte::util::sha1::sha1_fingerprint128(""),
        sha1_hash: String::new(),
        correlated_sha1_hash: None,
        structure_sha1_hash: None,
        atom_count_memo: std::cell::Cell::new(usize::MAX),
    })
}

#[test]
fn word_count_equals_atom_cardinality() {
    for n in [0usize, 1, 2, 5, 17, 40] {
        let contents: Vec<_> = (0..n).map(|i| atom(&i.to_string())).collect();
        let word = ComparisonUnit::Word(ComparisonUnitWord::new(contents));
        assert_eq!(
            word.descendant_content_atoms_count(),
            word.descendant_atoms().len(),
            "Word with {n} atoms"
        );
        assert_eq!(word.descendant_content_atoms_count(), n);
    }
}

#[test]
fn group_count_sums_nested_words() {
    let w1 = ComparisonUnit::Word(ComparisonUnitWord::new(vec![atom("a"), atom("b")]));
    let w2 = ComparisonUnit::Word(ComparisonUnitWord::new(vec![atom("c")]));
    let w3 = ComparisonUnit::Word(ComparisonUnitWord::new(vec![
        atom("d"),
        atom("e"),
        atom("f"),
    ]));
    let g_inner = group(ComparisonUnitGroupType::Paragraph, vec![w1, w2]);
    let g = group(ComparisonUnitGroupType::Table, vec![g_inner, w3]);
    assert_eq!(
        g.descendant_content_atoms_count(),
        g.descendant_atoms().len()
    );
    assert_eq!(g.descendant_content_atoms_count(), 2 + 1 + 3);
}
