//! M4.C — core LCS engine tests (on ComparisonUnit / CorrelatedSequence).

use jubarte::comparer::atoms::{
    ComparisonUnit, ComparisonUnitAtom, ComparisonUnitWord, CorrelatedSequence,
};
use jubarte::comparer::lcs::{
    do_lcs_algorithm, find_index_of_next_para_mark, lcs, longest_common_run,
    split_at_paragraph_mark,
};
use jubarte::comparer::{CorrelationStatus, WmlComparerSettings};
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

/// A word backed by a real `w:t` node carrying `text`, with content-hash from `tag`.
fn w_text(dom: &mut Dom, tag: &str, text: &str) -> ComparisonUnit {
    let t = dom.new_element(W::t());
    if !text.is_empty() {
        dom.add_text(t, text);
    }
    ComparisonUnit::Word(ComparisonUnitWord::new(vec![ComparisonUnitAtom::new(
        t,
        vec![],
        tag.to_string(),
    )]))
}
fn w(dom: &mut Dom, tag: &str) -> ComparisonUnit {
    w_text(dom, tag, "")
}
/// A bare paragraph-mark word (single `w:pPr` atom).
fn pmark(dom: &mut Dom, tag: &str) -> ComparisonUnit {
    let p = dom.new_element(W::p_pr());
    ComparisonUnit::Word(ComparisonUnitWord::new(vec![ComparisonUnitAtom::new(
        p,
        vec![],
        tag.to_string(),
    )]))
}
fn statuses(seqs: &[CorrelatedSequence]) -> Vec<CorrelationStatus> {
    seqs.iter().map(|cs| cs.correlation_status).collect()
}

/// M4.C.2 — longest common contiguous run by sha1.
#[test]
fn m4_c2_longest_common_run() {
    let mut d = Dom::new();
    let a = [
        w(&mut d, "a"),
        w(&mut d, "b"),
        w(&mut d, "c"),
        w(&mut d, "d"),
    ];
    let b = [
        w(&mut d, "x"),
        w(&mut d, "b"),
        w(&mut d, "c"),
        w(&mut d, "y"),
    ];
    assert_eq!(longest_common_run(&a, &b), (1, 1, 2));
    let dd = [w(&mut d, "p"), w(&mut d, "q")];
    assert_eq!(longest_common_run(&a, &dd).2, 0);
    let e = [w(&mut d, "m"), w(&mut d, "m")];
    let f = [w(&mut d, "m")];
    assert_eq!(longest_common_run(&e, &f), (0, 0, 1));
}

/// M4.C.1 — worklist driver fully resolves to Equal/Deleted/Inserted.
#[test]
fn m4_c1_driver() {
    let s = WmlComparerSettings::default();
    let mut d = Dom::new();
    assert!(lcs(&mut d, vec![], vec![], &s).is_empty());

    let a = w(&mut d, "a");
    let del = lcs(&mut d, vec![a], vec![], &s);
    assert_eq!(statuses(&del), vec![CorrelationStatus::Deleted]);

    let a = w(&mut d, "a");
    let ins = lcs(&mut d, vec![], vec![a], &s);
    assert_eq!(statuses(&ins), vec![CorrelationStatus::Inserted]);

    let (a1, a2) = (w(&mut d, "a"), w(&mut d, "a"));
    let eq = lcs(&mut d, vec![a1], vec![a2], &s);
    assert_eq!(statuses(&eq), vec![CorrelationStatus::Equal]);

    // [a,b,c] vs [x,b,c] → Deleted([a]), Inserted([x]), Equal([b,c])
    let l = vec![w(&mut d, "a"), w(&mut d, "b"), w(&mut d, "c")];
    let r = vec![w(&mut d, "x"), w(&mut d, "b"), w(&mut d, "c")];
    let res = lcs(&mut d, l, r, &s);
    assert_eq!(
        statuses(&res),
        vec![
            CorrelationStatus::Deleted,
            CorrelationStatus::Inserted,
            CorrelationStatus::Equal
        ]
    );
}

/// M4.C.3 — DoLcsAlgorithm one step: equal split leaves Unknown remainders.
#[test]
fn m4_c3_do_lcs_step() {
    let s = WmlComparerSettings::default();
    let mut d = Dom::new();
    let mk = |a, b| CorrelatedSequence::paired(CorrelationStatus::Unknown, a, b);

    let l = vec![w(&mut d, "a"), w(&mut d, "b"), w(&mut d, "c")];
    let r = vec![w(&mut d, "x"), w(&mut d, "b"), w(&mut d, "c")];
    let res = do_lcs_algorithm(&d, &mk(l, r), &s);
    assert_eq!(res[0].correlation_status, CorrelationStatus::Unknown);
    assert_eq!(res[1].correlation_status, CorrelationStatus::Equal);

    assert!(do_lcs_algorithm(&d, &mk(vec![], vec![]), &s).is_empty());
}

/// M4.C.4 — para-mark helpers + Step C (never start common run on a pPr).
#[test]
fn m4_c4_para_mark_guards() {
    let mut d = Dom::new();
    // find_index_of_next_para_mark
    let units = vec![
        w(&mut d, "a"),
        w(&mut d, "b"),
        pmark(&mut d, "p"),
        w(&mut d, "c"),
    ];
    assert_eq!(find_index_of_next_para_mark(&d, &units), 2);
    let no_pm = vec![w(&mut d, "a"), w(&mut d, "b")];
    assert_eq!(find_index_of_next_para_mark(&d, &no_pm), 2);

    // split_at_paragraph_mark keeps the pmark at head of chunk 2
    let u2 = vec![w(&mut d, "a"), pmark(&mut d, "p"), w(&mut d, "b")];
    let split = split_at_paragraph_mark(&d, &u2);
    assert_eq!(split.len(), 2);
    assert_eq!(split[0].len(), 1);
    assert_eq!(split[1].len(), 2);

    // Step C lives in do_lcs_algorithm: a common run [pmark, a] is trimmed so the
    // leading pmark splits off (the run does not START on a paragraph mark).
    // (Via the full lcs() pipeline FindCommon's front path would match the whole
    // run as one Equal, so Step C is exercised on do_lcs_algorithm directly.)
    let s = WmlComparerSettings::default();
    let l = vec![pmark(&mut d, "p"), w(&mut d, "a")];
    let r = vec![pmark(&mut d, "p"), w(&mut d, "a")];
    let cs = CorrelatedSequence::paired(CorrelationStatus::Unknown, l, r);
    let res = do_lcs_algorithm(&d, &cs, &s);
    // leading pmark → its own Unknown, then Equal([a]) (I.6 may append empty
    // Unknowns that the driver drops — assert the meaningful prefix).
    assert_eq!(
        res[0].correlation_status,
        CorrelationStatus::Unknown,
        "Step C splits off the pmark"
    );
    assert_eq!(res[0].com_units_1.as_ref().unwrap().len(), 1);
    assert_eq!(res[1].correlation_status, CorrelationStatus::Equal);
    // the Equal carries just [a]
    assert_eq!(res[1].com_units_2.as_ref().unwrap().len(), 1);
}

/// M4.C.7 — voiding guards (Step F word-break, Step G threshold).
#[test]
fn m4_c7_voiding_guards() {
    // Step G needs the PowerTools threshold (0.15); the word-mode default is 0.0.
    let s = WmlComparerSettings::powertools_faithful(); // detail_threshold = 0.15, space is a separator

    // Step F (in do_lcs_algorithm): a single common separator-only word is voided.
    // (Via lcs() FindCommon's front path would match it first; Step F is tested on
    // do_lcs_algorithm directly.)
    let mut d = Dom::new();
    let l = vec![w_text(&mut d, "sp", " ")];
    let r = vec![w_text(&mut d, "sp", " ")];
    let cs = CorrelatedSequence::paired(CorrelationStatus::Unknown, l, r);
    let res = do_lcs_algorithm(&d, &cs, &s);
    assert!(
        !statuses(&res).contains(&CorrelationStatus::Equal),
        "single separator word must not match (Step F)"
    );

    // Step G: a 1-of-7 common run (1/7 ≈ 0.14 < 0.15) on pure-word sides is voided.
    let mut d2 = Dom::new();
    let left: Vec<_> = ["a", "b", "c", "d", "e", "f", "g"]
        .iter()
        .map(|t| w(&mut d2, t))
        .collect();
    let right: Vec<_> = ["a", "z1", "z2", "z3", "z4", "z5", "z6"]
        .iter()
        .map(|t| w(&mut d2, t))
        .collect();
    let res2 = lcs(&mut d2, left, right, &s);
    assert!(
        !statuses(&res2).contains(&CorrelationStatus::Equal),
        "below-threshold common run must be voided (Step G)"
    );
}

use jubarte::comparer::ComparisonUnitGroupType;
use jubarte::comparer::atoms::ComparisonUnitGroup;

fn group(gt: ComparisonUnitGroupType, contents: Vec<ComparisonUnit>, sha: &str) -> ComparisonUnit {
    ComparisonUnit::Group(ComparisonUnitGroup {
        correlation_status: CorrelationStatus::Nil,
        group_type: gt,
        contents,
        level: 0,
        sha1_hash: sha.to_string(),
        correlated_sha1_hash: None,
        structure_sha1_hash: None,
    })
}
fn unk(a: Vec<ComparisonUnit>, b: Vec<ComparisonUnit>) -> CorrelatedSequence {
    CorrelatedSequence::paired(CorrelationStatus::Unknown, a, b)
}

/// M4.C.8 — Step H1 (words + rows mix).
#[test]
fn m4_c8_step_h1_word_row() {
    let s = WmlComparerSettings::default();
    let mut d = Dom::new();
    let wa = w(&mut d, "a");
    let row_l = group(ComparisonUnitGroupType::Row, vec![], "rL");
    let row_r = group(ComparisonUnitGroupType::Row, vec![], "rR");
    let res = do_lcs_algorithm(&d, &unk(vec![wa, row_l], vec![row_r]), &s);
    // Inserted([rR]) then Deleted([wa]) + Deleted([rowL]) (faithful H1 outcome)
    assert_eq!(
        statuses(&res),
        vec![
            CorrelationStatus::Inserted,
            CorrelationStatus::Deleted,
            CorrelationStatus::Deleted
        ]
    );
}

/// M4.C.9 — Step H4 (both sides only paragraphs → flatten one level).
#[test]
fn m4_c9_step_h4_flatten() {
    let s = WmlComparerSettings::default();
    let mut d = Dom::new();
    let pg1 = group(
        ComparisonUnitGroupType::Paragraph,
        vec![w(&mut d, "a"), pmark(&mut d, "p1")],
        "ph1",
    );
    let pg2 = group(
        ComparisonUnitGroupType::Paragraph,
        vec![w(&mut d, "b"), pmark(&mut d, "p2")],
        "ph2",
    );
    let res = do_lcs_algorithm(&d, &unk(vec![pg1], vec![pg2]), &s);
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].correlation_status, CorrelationStatus::Unknown);
    // flattened to the paragraph children (word + pmark) on each side
    assert_eq!(res[0].com_units_1.as_ref().unwrap().len(), 2);
    assert_eq!(res[0].com_units_2.as_ref().unwrap().len(), 2);
}

/// M4.C.10 — Step H5 (first unit a Row on both sides → cell alignment with pad).
#[test]
fn m4_c10_step_h5_row_cells() {
    let s = WmlComparerSettings::default();
    let mut d = Dom::new();
    let cell_a = group(ComparisonUnitGroupType::Cell, vec![w(&mut d, "ca")], "cA");
    let cell_b = group(ComparisonUnitGroupType::Cell, vec![w(&mut d, "cb")], "cB");
    let cell_a2 = group(ComparisonUnitGroupType::Cell, vec![w(&mut d, "ca2")], "cA2");
    let row_l = group(ComparisonUnitGroupType::Row, vec![cell_a, cell_b], "rowL");
    let row_r = group(ComparisonUnitGroupType::Row, vec![cell_a2], "rowR");
    let res = do_lcs_algorithm(&d, &unk(vec![row_l], vec![row_r]), &s);
    // 2 cells vs 1: [Unknown(cellA,cellA2), Deleted(cellB contents)]
    assert_eq!(
        statuses(&res),
        vec![CorrelationStatus::Unknown, CorrelationStatus::Deleted]
    );
}

use jubarte::comparer::lcs::{detect_unrelated_sources, process_correlated_hashes};

fn group_corr(
    gt: ComparisonUnitGroupType,
    sha: &str,
    corr: &str,
    atoms: usize,
    d: &mut Dom,
) -> ComparisonUnit {
    // build `atoms` word children so descendant_content_atoms_count == atoms
    let contents: Vec<ComparisonUnit> = (0..atoms).map(|i| w(d, &format!("{sha}-{i}"))).collect();
    ComparisonUnit::Group(ComparisonUnitGroup {
        correlation_status: CorrelationStatus::Nil,
        group_type: gt,
        contents,
        level: 0,
        sha1_hash: sha.to_string(),
        correlated_sha1_hash: Some(corr.to_string()),
        structure_sha1_hash: None,
    })
}

/// M4.C.12 — DetectUnrelatedSources.
#[test]
fn m4_c12_detect_unrelated() {
    let mut d = Dom::new();
    // 4 groups each side, disjoint sha → unrelated
    let l: Vec<_> = ["a", "b", "c", "e"]
        .iter()
        .map(|t| group(ComparisonUnitGroupType::Paragraph, vec![], t))
        .collect();
    let r: Vec<_> = ["w", "x", "y", "z"]
        .iter()
        .map(|t| group(ComparisonUnitGroupType::Paragraph, vec![], t))
        .collect();
    let res = detect_unrelated_sources(&l, &r).expect("unrelated");
    assert_eq!(
        statuses(&res),
        vec![CorrelationStatus::Deleted, CorrelationStatus::Inserted]
    );
    let _ = &mut d;

    // overlap (shared "a") → None
    let l2: Vec<_> = ["a", "b", "c", "e"]
        .iter()
        .map(|t| group(ComparisonUnitGroupType::Paragraph, vec![], t))
        .collect();
    let r2: Vec<_> = ["a", "x", "y", "z"]
        .iter()
        .map(|t| group(ComparisonUnitGroupType::Paragraph, vec![], t))
        .collect();
    assert!(detect_unrelated_sources(&l2, &r2).is_none());

    // <4 groups → None
    let l3: Vec<_> = ["a", "b"]
        .iter()
        .map(|t| group(ComparisonUnitGroupType::Paragraph, vec![], t))
        .collect();
    let r3: Vec<_> = ["x", "y"]
        .iter()
        .map(|t| group(ComparisonUnitGroupType::Paragraph, vec![], t))
        .collect();
    assert!(detect_unrelated_sources(&l3, &r3).is_none());
}

/// M4.C.11 — ProcessCorrelatedHashes (≥4 matched groups → correlate).
#[test]
fn m4_c11_process_correlated_hashes() {
    use jubarte::comparer::atoms::CorrelatedSequence;
    let mut d = Dom::new();
    // 5 paragraph groups each; indices 1..5 share correlated hash (4 matched > 3)
    let pgt = ComparisonUnitGroupType::Paragraph;
    let l = vec![
        group_corr(pgt, "g0", "c0", 1, &mut d),
        group_corr(pgt, "g1", "M1", 1, &mut d),
        group_corr(pgt, "g2", "M2", 1, &mut d),
        group_corr(pgt, "g3", "M3", 1, &mut d),
        group_corr(pgt, "g4", "M4", 1, &mut d),
    ];
    let r = vec![
        group_corr(pgt, "h0", "d0", 1, &mut d),
        group_corr(pgt, "h1", "M1", 1, &mut d),
        group_corr(pgt, "h2", "M2", 1, &mut d),
        group_corr(pgt, "h3", "M3", 1, &mut d),
        group_corr(pgt, "h4", "M4", 1, &mut d),
    ];
    let cs = CorrelatedSequence::paired(CorrelationStatus::Unknown, l, r);
    let res = process_correlated_hashes(&cs).expect("correlate (4 matched > 3)");
    // before Unknown (g0/h0), then 4 single-group Unknowns
    assert_eq!(res.len(), 5);
    assert!(
        res.iter()
            .all(|c| c.correlation_status == CorrelationStatus::Unknown)
    );
    // each matched Unknown carries exactly one group per side
    assert_eq!(res[1].com_units_1.as_ref().unwrap().len(), 1);

    // <3 units → None
    let l2 = vec![
        group_corr(pgt, "a", "x", 1, &mut d),
        group_corr(pgt, "b", "y", 1, &mut d),
    ];
    let r2 = vec![
        group_corr(pgt, "a", "x", 1, &mut d),
        group_corr(pgt, "b", "y", 1, &mut d),
    ];
    let cs2 = CorrelatedSequence::paired(CorrelationStatus::Unknown, l2, r2);
    assert!(process_correlated_hashes(&cs2).is_none());
}

use jubarte::comparer::lcs::find_common_at_beginning_and_end;

/// M4.C.5/C.6 — FindCommonAtBeginningAndEnd front + back + only-pmark guard.
#[test]
fn m4_c5_c6_find_common() {
    let s = WmlComparerSettings::default();
    let mut d = Dom::new();

    // Front: [a,b,X]/[a,b,Y] → Equal([a,b]), Unknown([X],[Y])
    let l = vec![w(&mut d, "a"), w(&mut d, "b"), w(&mut d, "X")];
    let r = vec![w(&mut d, "a"), w(&mut d, "b"), w(&mut d, "Y")];
    let cs = CorrelatedSequence::paired(CorrelationStatus::Unknown, l, r);
    let res = find_common_at_beginning_and_end(&d, &cs, &s).expect("front match");
    assert_eq!(
        statuses(&res),
        vec![CorrelationStatus::Equal, CorrelationStatus::Unknown]
    );

    // Back: [X,a,b]/[Y,a,b] → Unknown([X],[Y]), Equal([a,b])
    let l = vec![w(&mut d, "X"), w(&mut d, "a"), w(&mut d, "b")];
    let r = vec![w(&mut d, "Y"), w(&mut d, "a"), w(&mut d, "b")];
    let cs = CorrelatedSequence::paired(CorrelationStatus::Unknown, l, r);
    let res = find_common_at_beginning_and_end(&d, &cs, &s).expect("back match");
    assert_eq!(
        statuses(&res),
        vec![CorrelationStatus::Unknown, CorrelationStatus::Equal]
    );

    // Only-paragraph-mark tail → None (WC010 guard); driver falls to DoLcs.
    let l = vec![w(&mut d, "X"), pmark(&mut d, "p")];
    let r = vec![w(&mut d, "Y"), pmark(&mut d, "p")];
    let cs = CorrelatedSequence::paired(CorrelationStatus::Unknown, l, r);
    assert!(
        find_common_at_beginning_and_end(&d, &cs, &s).is_none(),
        "tail-only-pmark declines"
    );
}
