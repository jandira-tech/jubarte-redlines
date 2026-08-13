// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M4.D — table/row/cell LCS tests.

use jubarte::comparer::atoms::{
    ComparisonUnit, ComparisonUnitAtom, ComparisonUnitGroup, ComparisonUnitWord, CorrelatedSequence,
};
use jubarte::comparer::lcs_table::{
    apply_lcs_to_table_rows, do_lcs_algorithm_for_table, mark_rows_as_deleted_or_inserted,
};
use jubarte::comparer::{ComparisonUnitGroupType, CorrelationStatus, WmlComparerSettings};
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

fn grp(
    gt: ComparisonUnitGroupType,
    contents: Vec<ComparisonUnit>,
    sha: &str,
    corr: Option<&str>,
) -> ComparisonUnit {
    ComparisonUnit::Group(ComparisonUnitGroup {
        correlation_status: CorrelationStatus::Nil,
        group_type: gt,
        contents,
        level: 0,
        sha1_key: jubarte::util::sha1::sha1_fingerprint(sha),
        sha1_key128: jubarte::util::sha1::sha1_fingerprint128(sha),
        sha1_hash: sha.to_string(),
        correlated_sha1_hash: corr.map(|s| s.to_string()),
        structure_sha1_hash: None,
        atom_count_memo: std::cell::Cell::new(usize::MAX),
    })
}
fn row(sha: &str, corr: Option<&str>) -> ComparisonUnit {
    grp(ComparisonUnitGroupType::Row, vec![], sha, corr)
}
fn statuses(seqs: &[CorrelatedSequence]) -> Vec<CorrelationStatus> {
    seqs.iter().map(|cs| cs.correlation_status).collect()
}

/// M4.D.1 — ApplyLcsToTableRows DP.
#[test]
fn m4_d1_apply_lcs_rows() {
    let r1 = [
        row("A", None),
        row("B", None),
        row("C", None),
        row("D", None),
    ];
    let r2 = [
        row("A", None),
        row("X", None),
        row("C", None),
        row("D", None),
    ];
    let res = apply_lcs_to_table_rows(&r1, &r2);
    assert_eq!(
        statuses(&res),
        vec![
            CorrelationStatus::Unknown,
            CorrelationStatus::Deleted,
            CorrelationStatus::Inserted,
            CorrelationStatus::Unknown,
            CorrelationStatus::Unknown
        ]
    );
    // identical rows → all matched, no del/ins
    let e1 = [row("A", None), row("B", None)];
    let e2 = [row("A", None), row("B", None)];
    let res2 = apply_lcs_to_table_rows(&e1, &e2);
    assert_eq!(
        statuses(&res2),
        vec![CorrelationStatus::Unknown, CorrelationStatus::Unknown]
    );
}

/// M4.D.2 — DoLcsAlgorithmForTable same-row-count, all correlated-equal → positional.
#[test]
fn m4_d2_table_same_count_collapse() {
    let s = WmlComparerSettings::default();
    let dom = Dom::new();
    let t1 = grp(
        ComparisonUnitGroupType::Table,
        vec![
            row("a", Some("M")),
            row("b", Some("N")),
            row("c", Some("O")),
        ],
        "t1",
        None,
    );
    let t2 = grp(
        ComparisonUnitGroupType::Table,
        vec![
            row("a2", Some("M")),
            row("b2", Some("N")),
            row("c2", Some("O")),
        ],
        "t2",
        None,
    );
    let res = do_lcs_algorithm_for_table(&dom, &[t1], &[t2], &s).expect("collapses");
    assert_eq!(res.len(), 3);
    assert!(
        res.iter()
            .all(|c| c.correlation_status == CorrelationStatus::Unknown)
    );

    // rows that do NOT correlate (different correlated hashes) and no merged cells → None
    let t3 = grp(
        ComparisonUnitGroupType::Table,
        vec![row("a", Some("M"))],
        "t3",
        None,
    );
    let t4 = grp(
        ComparisonUnitGroupType::Table,
        vec![row("z", Some("Q"))],
        "t4",
        None,
    );
    assert!(do_lcs_algorithm_for_table(&dom, &[t3], &[t4], &s).is_none());

    // both rows have absent correlated hashes → matches C# null == null, collapses positionally.
    let t5 = grp(
        ComparisonUnitGroupType::Table,
        vec![row("a", None), row("b", None)],
        "t5",
        None,
    );
    let t6 = grp(
        ComparisonUnitGroupType::Table,
        vec![row("a2", None), row("b2", None)],
        "t6",
        None,
    );
    let res5 = do_lcs_algorithm_for_table(&dom, &[t5], &[t6], &s).expect("absent-absent collapses");
    assert_eq!(res5.len(), 2);
    assert!(
        res5.iter()
            .all(|c| c.correlation_status == CorrelationStatus::Unknown)
    );
}

/// M4.D.4 — MarkRowsAsDeletedOrInserted stamps w:trPr/w:del on a deleted row.
#[test]
fn m4_d4_mark_rows() {
    let s = WmlComparerSettings::default();
    let mut dom = Dom::new();
    // real w:tr ancestor for the row's first atom
    let tr = dom.new_element(W::name("tr"));
    let t = dom.new_element(W::t());
    let atom = ComparisonUnitAtom::new(t, vec![tr], "x".to_string());
    let word = ComparisonUnit::Word(ComparisonUnitWord::new(vec![atom]));
    let rowg = grp(ComparisonUnitGroupType::Row, vec![word], "row", None);

    let seqs = vec![CorrelatedSequence::deleted(vec![rowg])];
    let mut id = 1u32;
    mark_rows_as_deleted_or_inserted(&mut dom, &s, &seqs, &mut id);

    let trpr = dom.element(tr, &W::name("trPr")).expect("trPr created");
    let del = dom.element(trpr, &W::del()).expect("w:del in trPr");
    assert_eq!(
        dom.attribute(del, &W::author()).unwrap(),
        s.author_for_revisions
    );
    assert!(dom.attribute(del, &W::id()).is_some());

    // a Deleted Paragraph (not a Row) → untouched
    let mut dom2 = Dom::new();
    let p = dom2.new_element(W::p());
    let t2 = dom2.new_element(W::t());
    let atom2 = ComparisonUnitAtom::new(t2, vec![p], "y".to_string());
    let word2 = ComparisonUnit::Word(ComparisonUnitWord::new(vec![atom2]));
    let pg = grp(ComparisonUnitGroupType::Paragraph, vec![word2], "p", None);
    let mut id2 = 1u32;
    mark_rows_as_deleted_or_inserted(
        &mut dom2,
        &s,
        &[CorrelatedSequence::deleted(vec![pg])],
        &mut id2,
    );
    assert!(
        dom2.element(p, &W::name("trPr")).is_none(),
        "paragraph untouched"
    );
    assert_eq!(id2, 1, "no id consumed for non-row");

    // Inserted branch: row CorrelatedSequence::inserted → trPr gets w:ins with author/id/date,
    // next_id advances past the stamp.
    let mut dom3 = Dom::new();
    let tr3 = dom3.new_element(W::name("tr"));
    let t3a = dom3.new_element(W::t());
    let atom3 = ComparisonUnitAtom::new(t3a, vec![tr3], "z".to_string());
    let word3 = ComparisonUnit::Word(ComparisonUnitWord::new(vec![atom3]));
    let rowg3 = grp(ComparisonUnitGroupType::Row, vec![word3], "row-ins", None);
    let mut id3 = 10u32;
    mark_rows_as_deleted_or_inserted(
        &mut dom3,
        &s,
        &[CorrelatedSequence::inserted(vec![rowg3])],
        &mut id3,
    );
    let trpr3 = dom3
        .element(tr3, &W::name("trPr"))
        .expect("trPr created (inserted)");
    let ins3 = dom3.element(trpr3, &W::ins()).expect("w:ins in trPr");
    assert_eq!(
        dom3.attribute(ins3, &W::author()).unwrap(),
        s.author_for_revisions
    );
    assert_eq!(
        dom3.attribute(ins3, &W::id()).unwrap(),
        "10",
        "id assigned at stamp"
    );
    assert_eq!(
        dom3.attribute(ins3, &W::date()).unwrap(),
        s.date_time_for_revisions
    );
    assert_eq!(id3, 11, "next_id advanced past inserted stamp");
}
