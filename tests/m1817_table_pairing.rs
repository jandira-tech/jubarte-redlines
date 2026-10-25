// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! #1817 — a table replaced in place pairs with the revision's table the way
//! Word 16's Compare pairs it, in Word mode. Before, jubarte wrote the
//! revision's rows inserted beside the original's rows deleted, and Word
//! 16.115 would not open those redlines of w04 and w23 (the whole deleted
//! row plus a layout partner, jubarte-loop/fix23/NOTES.md rounds 1-15).
//!
//! - w04: the original's one-row table and the revision's two-row table are
//!   one table, as in Word's redline: the first row kept, with "Név" inserted
//!   and the old text deleted in it, the second row inserted.
//! - w23: the original's table pairs with the revision's first table only;
//!   the revision's second table, adjacent to it, stays a table of its own.

mod common;

use std::path::PathBuf;

use common::validity::assert_word_valid_package;
use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::{
    accept_revisions, compare_documents_with_settings, reject_revisions,
};
use jubarte::namespaces::W;
use jubarte::opc::PartFs;
use jubarte::xmllinq::{Dom, NodeId};

fn fixture(name: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/corpus/neurotic_docx_bench/corpus/word/clean/docx")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// One row: its `trPr` mark (`ins`, `del` or empty) and per cell
/// (live text, deleted text, has a `tcPrChange`).
#[derive(Debug)]
struct Row {
    mark: String,
    cells: Vec<(String, String, bool)>,
}

/// The body's tables, each with its rows and whether it holds a `tblPrChange`.
fn tables(docx: &[u8]) -> Vec<(Vec<Row>, bool)> {
    let pkg = PartFs::open(docx).unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&pkg.part_string("word/document.xml").unwrap());
    let root = dom.root(doc).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    let text = |dom: &Dom, n: NodeId, name| -> String {
        dom.descendants(n, Some(&name))
            .into_iter()
            .map(|t| dom.value(t))
            .collect()
    };
    dom.elements(body, Some(&W::tbl()))
        .into_iter()
        .map(|tbl| {
            let changed = dom
                .element(tbl, &W::tbl_pr())
                .is_some_and(|pr| dom.element(pr, &W::name("tblPrChange")).is_some());
            let rows = dom
                .elements(tbl, Some(&W::tr()))
                .into_iter()
                .map(|tr| {
                    let mark = dom
                        .element(tr, &W::tr_pr())
                        .map(|pr| {
                            dom.elements(pr, None)
                                .into_iter()
                                .filter_map(|c| dom.name(c))
                                .map(|n| n.local_name().to_string())
                                .filter(|n| n == "ins" || n == "del")
                                .collect()
                        })
                        .unwrap_or_default();
                    let cells = dom
                        .elements(tr, Some(&W::tc()))
                        .into_iter()
                        .map(|tc| {
                            let pr = dom.element(tc, &W::tc_pr());
                            let marked = |name: &str| {
                                pr.is_some_and(|pr| dom.element(pr, &W::name(name)).is_some())
                            };
                            assert!(
                                !marked("cellIns") && !marked("cellDel"),
                                "Word writes no cell insertion or deletion"
                            );
                            (
                                text(&dom, tc, W::t()),
                                text(&dom, tc, W::del_text()),
                                marked("tcPrChange"),
                            )
                        })
                        .collect();
                    Row { mark, cells }
                })
                .collect();
            (rows, changed)
        })
        .collect()
}

fn holds(rows: &[Row], needle: &str) -> bool {
    rows.iter().any(|row| {
        row.cells
            .iter()
            .any(|(live, deleted, _)| live.contains(needle) || deleted.contains(needle))
    })
}

fn compare(a: &[u8], b: &[u8]) -> Vec<u8> {
    let redline =
        compare_documents_with_settings(a, b, &WmlComparerSettings::default()).expect("compare");
    assert_word_valid_package(&redline);
    redline
}

/// Accepting keeps the revision's table count and rejecting the original's.
fn assert_table_counts(redline: &[u8], a: &[u8], b: &[u8]) {
    let accepted = accept_revisions(redline).unwrap();
    let rejected = reject_revisions(redline).unwrap();
    assert_eq!(tables(&accepted).len(), tables(b).len(), "accepted tables");
    assert_eq!(tables(&rejected).len(), tables(a).len(), "rejected tables");
}

#[test]
fn w04_pairs_the_replaced_table_and_inserts_its_new_row() {
    let a = fixture("2b4475cba6_0021f639e5dbf6c217c8d81a7ac3374624ff9901a5d4b200.docx");
    let b = fixture("342f364d46_001342333d45627868f4dc5282abbba4598a91b4a7584b08.docx");
    let redline = compare(&a, &b);
    let all = tables(&redline);
    let (rows, changed) = all
        .iter()
        .find(|(rows, _)| holds(rows, "Név"))
        .expect("a table holds Név");
    assert!(
        holds(&rows[..1], "Al Responsabile"),
        "the old text is deleted in the paired first row: {rows:#?}"
    );
    assert!(rows[0].mark.is_empty(), "the first row is kept: {rows:#?}");
    assert!(
        rows[0].cells.iter().all(|&(_, _, recorded)| recorded),
        "every cell of the paired row records its properties: {rows:#?}"
    );
    assert_eq!(
        rows[1].mark, "ins",
        "the new second row is inserted: {rows:#?}"
    );
    assert!(holds(&rows[1..2], "Beosztás"), "{rows:#?}");
    assert!(*changed, "the table records the old table's properties");
    assert!(
        !all.iter().any(|(rows, _)| rows
            .iter()
            .any(|row| row.mark == "del" && holds(std::slice::from_ref(row), "Al Responsabile"))),
        "no whole deleted row holds the old text"
    );
    assert_table_counts(&redline, &a, &b);
}

#[test]
fn w23_pairs_the_original_table_with_the_first_adjacent_table_only() {
    let a = fixture("f1ea4a51fd_hr_onboarding_checklist_table.docx");
    let b = fixture("d0e7f278b8_i_am_sharing_microsoft_word_vs_google_docs_compr.docx");
    let redline = compare(&a, &b);
    let all = tables(&redline);
    let (rows, _) = all
        .iter()
        .find(|(rows, _)| holds(rows, "Sign NDA"))
        .expect("a table holds the original's rows");
    assert!(
        holds(&rows[..1], "Positioning thesis") && holds(&rows[..1], "Step"),
        "the original's first row pairs with the first table's: {rows:#?}"
    );
    assert!(
        !holds(rows, "Prepared for"),
        "the second table's rows stay in their own table: {rows:#?}"
    );
    assert!(rows[0].mark.is_empty(), "{rows:#?}");
    assert!(
        rows[1..].iter().all(|row| row.mark == "del"),
        "the original's other rows are deleted: {rows:#?}"
    );
    assert_table_counts(&redline, &a, &b);
}
