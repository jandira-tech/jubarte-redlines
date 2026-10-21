// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M4.G — moves + format-change detection tests.

mod common;

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

// #1214: Word's Reject All duplicates text with unequal per-name range counts.
const MOVE_HEAD: &str = "Closing section remains";
const MOVE_TAIL: &str = " last in the original agreement.";

fn move_source(marker: &str) -> Vec<u8> {
    use common::docx::{Part, docx_with, para, run};
    let start = match marker {
        "bookmarkEnd" => r#"<w:bookmarkStart w:id="40" w:name="Clause"/>"#,
        "permEnd" => r#"<w:permStart w:id="40" w:edGrp="everyone"/>"#,
        "commentRangeEnd" => r#"<w:commentRangeStart w:id="40"/>"#,
        _ => "",
    };
    let end = if marker.is_empty() {
        String::new()
    } else {
        format!(r#"<w:{marker} w:id="40"/>"#)
    };
    let reference = if marker == "commentRangeEnd" {
        r#"<w:r><w:commentReference w:id="40"/></w:r>"#
    } else {
        ""
    };
    let body = format!(
        "{}<w:p>{start}{}{reference}</w:p><w:p>{}{end}{}</w:p>",
        para("Introduction stays put here. Lead live text."),
        run(
            "Carol Diaz reviews the complete agreement with Alice and Bob before final approval.",
            false,
            false,
            None
        ),
        run(MOVE_HEAD, marker.is_empty(), false, None),
        run(MOVE_TAIL, false, false, None),
    );
    let comments = Part {
        name: "word/comments.xml",
        content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
        rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
        xml: r#"<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:comment w:id="40" w:author="Alice"><w:p><w:r><w:t>Review the clause.</w:t></w:r></w:p></w:comment></w:comments>"#,
    };
    docx_with(
        &body,
        if marker == "commentRangeEnd" {
            std::slice::from_ref(&comments)
        } else {
            &[]
        },
    )
}

fn move_plan() -> jubarte::edit::EditPlan {
    jubarte::edit::EditPlan::from_json(r#"{"schema_version":1,"author":"Alice","date":"2026-10-18T12:00:00Z","operations":[{"kind":"delete_paragraph","paragraph":"p2"},{"kind":"insert_paragraph","paragraph":"p0","position":"after","runs":[{"text":"Closing section remains last in the original agreement."}]}]}"#).unwrap()
}

fn move_texts(bytes: &[u8]) -> Vec<String> {
    jubarte::inspect::paragraphs(bytes)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect()
}

fn assert_one_move_range(bytes: &[u8], marker: &str) {
    use std::collections::BTreeMap;
    let xml = common::docx::part_string(bytes, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let nodes = dom.descendants(root, None);
    let mut sides = Vec::new();
    let from_count = dom
        .descendants(root, Some(&W::name("moveFromRangeStart")))
        .len();
    let to_count = dom
        .descendants(root, Some(&W::name("moveToRangeStart")))
        .len();
    for side in ["moveFrom", "moveTo"] {
        let starts = dom.descendants(root, Some(&W::name(&format!("{side}RangeStart"))));
        let ends = dom.descendants(root, Some(&W::name(&format!("{side}RangeEnd"))));
        let mut names = BTreeMap::new();
        for &start in &starts {
            *names
                .entry(dom.attribute(start, &W::name("name")).unwrap().to_string())
                .or_insert(0) += 1;
            let id = dom.attribute(start, &W::id()).unwrap();
            let matched: Vec<_> = ends
                .iter()
                .filter(|&&end| dom.attribute(end, &W::id()) == Some(id))
                .collect();
            assert_eq!(matched.len(), 1, "unbalanced {side} range: {xml}");
            assert!(
                nodes.iter().position(|&n| n == start)
                    < nodes.iter().position(|&n| n == **matched.first().unwrap())
            );
        }
        assert!(!names.is_empty(), "expected a detected move: {xml}");
        assert_eq!(starts.len(), ends.len(), "unbalanced {side}: {xml}");
        for (name, count) in &names {
            assert_eq!(
                *count, 1,
                "{side} {name} must have one range (source={from_count}, destination={to_count})"
            );
        }
        if side == "moveFrom" {
            assert_eq!(
                dom.descendants(root, Some(&W::name(side))).len(),
                2,
                "each source run keeps its wrapper"
            );
            let first = nodes.iter().position(|&n| n == starts[0]).unwrap();
            let last = nodes.iter().position(|&n| n == ends[0]).unwrap();
            for wrapper in dom.descendants(root, Some(&W::name(side))) {
                let position = nodes.iter().position(|&n| n == wrapper).unwrap();
                assert!(first < position && position < last);
            }
            if !marker.is_empty() {
                let marks = dom.descendants(root, Some(&W::name(marker)));
                assert_eq!(marks.len(), 1);
                let position = nodes.iter().position(|&n| n == marks[0]).unwrap();
                assert!(
                    first < position && position < last,
                    "marker outside source range: {xml}"
                );
                let before: String = nodes[first..position]
                    .iter()
                    .filter(|&&n| dom.name_is(n, &W::t()))
                    .map(|&n| dom.value(n))
                    .collect();
                let after: String = nodes[position..last]
                    .iter()
                    .filter(|&&n| dom.name_is(n, &W::t()))
                    .map(|&n| dom.value(n))
                    .collect();
                assert_eq!((before.as_str(), after.as_str()), (MOVE_HEAD, MOVE_TAIL));
            }
        }
        sides.push(names);
    }
    assert_eq!(sides[0], sides[1]);
}

/// The fixed redline must not trip the unequal-count move check (#1214).
fn assert_no_move_range_mismatch(bytes: &[u8]) {
    assert!(
        !jubarte::validate::validate(bytes)
            .unwrap()
            .iter()
            .any(|f| f.code == "MOVE_RANGE_COUNT_MISMATCH")
    );
}

fn move_marker_location(bytes: &[u8], marker: &str) -> (String, String) {
    let xml = common::docx::part_string(bytes, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let marks = dom.descendants(root, Some(&W::name(marker)));
    assert_eq!(marks.len(), 1, "endpoint must survive settlement");
    let paragraph = dom.ancestors(marks[0], Some(&W::p()))[0];
    let mut before = String::new();
    let mut after = String::new();
    let mut seen = false;
    for node in dom.descendants(paragraph, None) {
        if node == marks[0] {
            seen = true;
        } else if dom.name_is(node, &W::t()) {
            if seen {
                after.push_str(&dom.value(node));
            } else {
                before.push_str(&dom.value(node));
            }
        }
    }
    (before, after)
}

fn check_split_source_edit(marker: &str) {
    use jubarte::document_comparer::{accept_revisions, reject_revisions};
    let original = move_source(marker);
    let result = jubarte::edit::apply_plan(&original, &move_plan()).unwrap();
    assert_one_move_range(&result.redline, marker);
    assert_no_move_range_mismatch(&result.redline);
    let rejected = reject_revisions(&result.redline).unwrap();
    assert_eq!(move_texts(&rejected), move_texts(&original));
    if !marker.is_empty() {
        assert_eq!(
            move_marker_location(&rejected, marker),
            move_marker_location(&original, marker)
        );
    }
    assert_eq!(
        move_texts(&accept_revisions(&result.redline).unwrap()),
        move_texts(&result.clean)
    );
    if let Some(dir) = std::env::var_os("JUBARTE_MOVE_PROBES") {
        let dir = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        let name = if marker.is_empty() { "two-run" } else { marker };
        std::fs::write(dir.join(format!("{name}-original.docx")), &original).unwrap();
        std::fs::write(dir.join(format!("{name}-redline.docx")), &result.redline).unwrap();
        std::fs::write(dir.join(format!("{name}-original.txt")), "Original must show Introduction / Carol Diaz / Closing, Closing text once, endpoint between remains and last when present.\n").unwrap();
        std::fs::write(dir.join(format!("{name}-redline.txt")), "Reject All must show Introduction / Carol Diaz / Closing, Closing text once and endpoint between remains and last when present; Accept All must show Introduction / Closing / Carol Diaz, Closing text once.\n").unwrap();
    }
}

#[test]
fn split_source_edit_has_one_move_range() {
    check_split_source_edit("");
}
#[test]
fn bookmark_end_split_source_edit_has_one_move_range() {
    check_split_source_edit("bookmarkEnd");
}
#[test]
fn permission_end_split_source_edit_has_one_move_range() {
    check_split_source_edit("permEnd");
}
#[test]
fn comment_end_split_source_edit_has_one_move_range() {
    check_split_source_edit("commentRangeEnd");
}

#[test]
fn compare_different_run_splits_has_one_move_range() {
    use jubarte::document_comparer::{accept_revisions, compare_documents, reject_revisions};
    let original = move_source("");
    let modified = jubarte::edit::apply_plan(&original, &move_plan())
        .unwrap()
        .clean;
    let redline = compare_documents(&original, &modified, "Alice").unwrap();
    assert_one_move_range(&redline, "");
    assert_eq!(
        move_texts(&reject_revisions(&redline).unwrap()),
        move_texts(&original)
    );
    assert_eq!(
        move_texts(&accept_revisions(&redline).unwrap()),
        move_texts(&modified)
    );
}

#[test]
fn compare_two_run_move_both_sides_has_one_move_range() {
    use common::docx::{docx, para, run};
    use jubarte::document_comparer::{accept_revisions, compare_documents, reject_revisions};
    let original = move_source("");
    let moved = format!(
        "<w:p>{}{}</w:p>",
        run(MOVE_HEAD, true, false, None),
        run(MOVE_TAIL, false, false, None)
    );
    let modified = docx(
        &(para("Introduction stays put here. Lead live text.")
            + &moved
            + &para(
                "Carol Diaz reviews the complete agreement with Alice and Bob before final approval.",
            )),
    );
    let redline = compare_documents(&original, &modified, "Alice").unwrap();
    assert_one_move_range(&redline, "");
    assert_eq!(
        move_texts(&reject_revisions(&redline).unwrap()),
        move_texts(&original)
    );
    assert_eq!(
        move_texts(&accept_revisions(&redline).unwrap()),
        move_texts(&modified)
    );
}
