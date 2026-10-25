// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M4.G — moves + format-change detection tests.

use jubarte::comparer::atoms::ComparisonUnitAtom;
use jubarte::comparer::formatchg::{
    are_run_properties_equal, detect_format_changes_in_atom_list, friendly_property_name,
    get_changed_property_names,
};
use jubarte::comparer::moves::{
    count_words, detect_moves_in_atom_list, group_consecutive_atoms_by_status, jaccard,
    split_by_chars, tokenize,
};
use jubarte::comparer::{CorrelationStatus, WmlComparerSettings};
use jubarte::namespaces::W;
use jubarte::xmllinq::{Dom, NodeId};

/// M4.G.1 — tokenizer / count / jaccard.
#[test]
fn m4_g1_tokenize_jaccard() {
    let s = WmlComparerSettings {
        word_separators: vec![' ', '-', ')', '(', ';', ','],
        ..Default::default()
    };
    assert_eq!(
        split_by_chars("a, b)c", &s.word_separators),
        vec!["a", "", "b", "c"]
    );
    assert_eq!(tokenize("the the cat", &s).len(), 2);
    assert_eq!(count_words("the the cat", &s), 3);
    assert_eq!(jaccard("a b c", "a b d", &s), 0.5);
    assert_eq!(jaccard("a b", "a b", &s), 1.0);
    assert_eq!(jaccard("", "a", &s), 0.0);
}

/// M4.G.3 — consecutive grouping + move detection (opt-in).
#[test]
fn m4_g3_moves() {
    let mut d = Dom::new();
    // helper: atom with text + status
    let mk = |d: &mut Dom, text: &str, st: CorrelationStatus| {
        let t = d.new_element(W::t());
        d.add_text(t, text);
        let mut a = ComparisonUnitAtom::new(t, vec![], "h");
        a.correlation_status = st;
        a
    };
    // ≥ move_minimum_word_count (default 6) so the block qualifies.
    let moved = "the quick brown fox jumps over lazy";
    let mut atoms = vec![
        mk(&mut d, moved, CorrelationStatus::Deleted),
        mk(&mut d, "unchanged", CorrelationStatus::Equal),
        mk(&mut d, moved, CorrelationStatus::Inserted),
    ];
    // grouping
    let del = group_consecutive_atoms_by_status(&atoms, CorrelationStatus::Deleted);
    assert_eq!(del.len(), 1);
    assert_eq!(del[0].start_index, 0);

    // detect_moves OFF (PowerTools-faithful) → no retag
    let off = WmlComparerSettings {
        detect_moves: false,
        ..Default::default()
    };
    let mut a2 = atoms.clone();
    detect_moves_in_atom_list(&d, &mut a2, &off);
    assert_eq!(
        a2[0].correlation_status,
        CorrelationStatus::Deleted,
        "moves off → unchanged"
    );

    // Word-visual default (detect_moves ON) → matched block retagged
    let on = WmlComparerSettings::default();
    assert!(on.detect_moves, "default must enable moves");
    assert!(on.move_minimum_word_count <= 6);
    detect_moves_in_atom_list(&d, &mut atoms, &on);
    assert_eq!(atoms[0].correlation_status, CorrelationStatus::MovedSource);
    assert_eq!(
        atoms[2].correlation_status,
        CorrelationStatus::MovedDestination
    );
    assert_eq!(atoms[0].move_name.as_deref(), Some("move1"));
    assert_eq!(atoms[0].move_group_id, atoms[2].move_group_id);
}

/// Multi-paragraph deleted run: each paragraph must match independently so a
/// mega-block does not drown Jaccard (file_8_file_9 class).
#[test]
fn m4_g3_paragraph_split_matches_relocated_paras() {
    use jubarte::comparer::atoms::AtomBlock;
    use jubarte::comparer::moves::split_block_on_paragraphs;

    let mut d = Dom::new();
    let mk_t = |d: &mut Dom, text: &str, st: CorrelationStatus| {
        let t = d.new_element(W::t());
        d.add_text(t, text);
        let mut a = ComparisonUnitAtom::new(t, vec![], "h");
        a.correlation_status = st;
        a
    };
    let mk_ppr = |d: &mut Dom, st: CorrelationStatus| {
        let p = d.new_element(W::p_pr());
        let mut a = ComparisonUnitAtom::new(p, vec![], "h");
        a.correlation_status = st;
        a
    };
    // Deleted run: paraA + pPr + paraB + pPr  (two paragraphs, one consecutive del run)
    // Inserted: paraB elsewhere, then other, then paraA
    let mut atoms = vec![
        mk_t(
            &mut d,
            "alpha unique block one words here",
            CorrelationStatus::Deleted,
        ),
        mk_ppr(&mut d, CorrelationStatus::Deleted),
        mk_t(
            &mut d,
            "bravo unique block two words here",
            CorrelationStatus::Deleted,
        ),
        mk_ppr(&mut d, CorrelationStatus::Deleted),
        mk_t(&mut d, "unchanged middle", CorrelationStatus::Equal),
        mk_t(
            &mut d,
            "bravo unique block two words here",
            CorrelationStatus::Inserted,
        ),
        mk_ppr(&mut d, CorrelationStatus::Inserted),
        mk_t(
            &mut d,
            "other new material here now",
            CorrelationStatus::Inserted,
        ),
        mk_ppr(&mut d, CorrelationStatus::Inserted),
        mk_t(
            &mut d,
            "alpha unique block one words here",
            CorrelationStatus::Inserted,
        ),
        mk_ppr(&mut d, CorrelationStatus::Inserted),
    ];
    let run = AtomBlock {
        atoms: vec![0, 1, 2, 3],
        start_index: 0,
    };
    let parts = split_block_on_paragraphs(&d, &atoms, &run);
    assert_eq!(parts.len(), 2, "two paragraphs → two move candidates");

    let on = WmlComparerSettings::default();
    detect_moves_in_atom_list(&d, &mut atoms, &on);
    assert_eq!(atoms[0].correlation_status, CorrelationStatus::MovedSource);
    assert_eq!(atoms[2].correlation_status, CorrelationStatus::MovedSource);
    assert_eq!(
        atoms[5].correlation_status,
        CorrelationStatus::MovedDestination
    );
    assert_eq!(
        atoms[9].correlation_status,
        CorrelationStatus::MovedDestination
    );
    assert_ne!(
        atoms[0].move_group_id, atoms[2].move_group_id,
        "each relocated paragraph is its own move group"
    );
}

/// M4.G.4 — rPr normalization + changed-property names.
#[test]
fn m4_g4_format_compare() {
    let mut d = Dom::new();
    // rPr with <w:b/> vs empty rPr → not equal; changed → ["bold"]
    let rpr_bold = d.new_element(W::r_pr());
    let b = d.new_element(W::name("b"));
    d.add(rpr_bold, b);
    let rpr_empty = d.new_element(W::r_pr());
    assert!(!are_run_properties_equal(
        &mut d,
        Some(rpr_bold),
        Some(rpr_empty)
    ));
    let empty2 = d.new_element(W::r_pr());
    assert!(
        are_run_properties_equal(&mut d, None, Some(empty2)),
        "null ≡ empty"
    );
    let changed = get_changed_property_names(&mut d, Some(rpr_empty), Some(rpr_bold));
    assert_eq!(changed, vec!["bold"]);
    assert_eq!(friendly_property_name("sz"), "fontSize");

    // rsid-only difference → equal
    let r1 = d.new_element(W::r_pr());
    let b1 = d.new_element(W::name("b"));
    d.set_attribute_value(b1, &W::name("rsidR"), Some("X"));
    d.add(r1, b1);
    let r2 = d.new_element(W::r_pr());
    let b2 = d.new_element(W::name("b"));
    d.add(r2, b2);
    assert!(
        are_run_properties_equal(&mut d, Some(r1), Some(r2)),
        "rsid ignored"
    );
}

/// M4.G.5 — DetectFormatChangesInAtomList retags Equal atoms with differing rPr.
#[test]
fn m4_g5_detect_format() {
    let s = WmlComparerSettings::default(); // detect_format_changes = true
    let mut d = Dom::new();
    // before run: plain; after run: bold. Equal atom linking them.
    let before_r = d.new_element(W::r());
    let before_t = d.new_element(W::t());
    d.add_text(before_t, "x");
    d.add(before_r, before_t);
    let after_r = d.new_element(W::r());
    let after_rpr = d.new_element(W::r_pr());
    let after_b = d.new_element(W::name("b"));
    d.add(after_rpr, after_b);
    d.add(after_r, after_rpr);
    let after_t = d.new_element(W::t());
    d.add_text(after_t, "x");
    d.add(after_r, after_t);

    let mut before_atom = ComparisonUnitAtom::new(before_t, vec![before_r, before_t], "h");
    before_atom.correlation_status = CorrelationStatus::Equal;
    let mut atom = ComparisonUnitAtom::new(after_t, vec![after_r, after_t], "h");
    atom.correlation_status = CorrelationStatus::Equal;
    atom.comparison_unit_atom_before = Some(std::sync::Arc::new(before_atom));

    let mut atoms = vec![atom];
    detect_format_changes_in_atom_list(&mut d, &mut atoms, &s);
    assert_eq!(
        atoms[0].correlation_status,
        CorrelationStatus::FormatChanged
    );
    let fc = atoms[0].format_change.as_ref().unwrap();
    assert!(fc.changed_properties.contains(&"bold".to_string()));

    let _ = NodeId(0);
}
