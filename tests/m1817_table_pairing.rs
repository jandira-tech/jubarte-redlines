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
//! - w12 and the Erasmus form: in a region the revision replaces, Word pairs
//!   the original's k-th table with the revision's k-th table, whatever their
//!   text, and writes the revision's blocks before the original's around
//!   them. jubarte inserted the revision's tables whole and deleted the
//!   original's whole, and Word 16.115 crashed on w12's deleted rows.
//! - 3x3: where the paired tables' styles (by name) or layouts differ, the
//!   original's deleted rows leave the revision's table for one of their own.

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

fn superdoc(name: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/corpus/neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The body's children as letters (`T` table, `P` paragraph) and each
/// table's grid widths.
fn body_shape(docx: &[u8]) -> (String, Vec<Vec<String>>) {
    let pkg = PartFs::open(docx).unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&pkg.part_string("word/document.xml").unwrap());
    let root = dom.root(doc).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    let mut kinds = String::new();
    let mut grids = Vec::new();
    for child in dom.elements(body, None) {
        if dom.name_is(child, &W::tbl()) {
            kinds.push('T');
            let grid = dom.element(child, &W::name("tblGrid")).unwrap();
            grids.push(
                dom.elements(grid, Some(&W::name("gridCol")))
                    .into_iter()
                    .map(|c| dom.attribute(c, &W::name("w")).unwrap_or("").to_string())
                    .collect(),
            );
        } else if dom.name_is(child, &W::p()) {
            kinds.push('P');
        }
    }
    (kinds, grids)
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

/// Each table's row marks, in body order: `=` kept, `I` inserted, `D` deleted.
fn row_marks(all: &[(Vec<Row>, bool)]) -> Vec<String> {
    all.iter()
        .map(|(rows, _)| {
            rows.iter()
                .map(|row| match row.mark.as_str() {
                    "ins" => 'I',
                    "del" => 'D',
                    _ => '=',
                })
                .collect()
        })
        .collect()
}

#[test]
fn w12_pairs_the_replaced_regions_tables_in_order() {
    // The original is a proposal form whose labels each head an empty
    // one-row box (19 tables); the revision is an application form with five
    // tables. Word's redline: each of the revision's tables keeps its first
    // row (the original's box) and inserts the rest; the original's other
    // 14 tables are deleted.
    let a = fixture("7b8a36fc9d_6b022d77b530d0e93df9ee17f79f06423a54fa1788a95c5d.docx");
    let b = fixture("f620bc5c6d_41bae32dd2e76af1f00d37af7b3a353ca826cd6e8c20052c.docx");
    let redline = compare(&a, &b);
    let all = tables(&redline);
    let marks = row_marks(&all);
    let revised = [
        "Last name",
        "Proposed date of departure",
        "Brief title",
        "Proposed Budget",
        "Applicant Name",
    ];
    assert_eq!(all.len(), 19, "{marks:?}");
    for (k, needle) in revised.iter().enumerate() {
        let (rows, _) = &all[k];
        assert!(
            holds(rows, needle),
            "table {k} is the revision's table {k}: {marks:?}"
        );
        assert!(
            marks[k].starts_with('=') && marks[k][1..].chars().all(|m| m == 'I'),
            "table {k} keeps the original's box and inserts the rest: {marks:?}"
        );
    }
    assert!(
        marks[5..].iter().all(|m| m.chars().all(|c| c == 'D')),
        "the original's other tables are deleted: {marks:?}"
    );
    assert_table_counts(&redline, &a, &b);
}

#[test]
fn erasmus_form_pairs_the_original_tables_with_the_first_two() {
    // The original (two five-row tables) and the revision (five tables)
    // share no text. Word's redline pairs the original's two tables with the
    // revision's first two and inserts the other three, revision blocks
    // first in the region before each pair.
    let a = fixture("f9b9dbd790_0017bce9abd08007241b106f0fcf1d64e45c05cd82a05a1d.docx");
    let b = fixture("342f364d46_001342333d45627868f4dc5282abbba4598a91b4a7584b08.docx");
    let redline = compare(&a, &b);
    let all = tables(&redline);
    assert_eq!(
        row_marks(&all),
        ["==DDD", "=====II", "II", "I", "I"],
        "Word's row marks"
    );
    assert!(holds(&all[0].0, "Név") && holds(&all[0].0, "Droits de scolarité"));
    assert!(holds(&all[1].0, "Mobilitás típusa") && holds(&all[1].0, "Le ou la doctorant"));
    assert_table_counts(&redline, &a, &b);
}

#[test]
fn plain_3x3_moves_the_originals_surplus_rows_to_a_table_of_their_own() {
    // The original's one-column table (table style "a", which has no name)
    // pairs with the revision's 3x3 table (the default table style). Word
    // keeps rows of one table style and layout in one table: the three
    // paired rows take the revision's table, and the original's other two
    // rows, deleted, follow as a table of their own with the original's
    // properties (Word's redline of the pair, a24b3782e4).
    let a = superdoc("super_editor__list_with_table_break_ff0c4c1f.docx");
    let b = superdoc("behavior__sd_2672_plain_3x3_87943d5d.docx");
    let redline = compare(&a, &b);
    let all = tables(&redline);
    assert_eq!(row_marks(&all), ["===", "DD"], "Word's row marks");
    assert!(
        all[0].1,
        "the paired table records the original's properties"
    );
    assert!(
        !all[1].1,
        "the deleted rows keep the original's properties as their own"
    );
    assert!(holds(&all[1].0, "Row data 5"), "{:#?}", all[1].0);
    let (kinds, grids) = body_shape(&redline);
    assert!(kinds.contains("TT"), "the two tables are adjacent: {kinds}");
    assert_eq!(grids[1], ["2790"], "the original's grid");
    assert_table_counts(&redline, &a, &b);
}

/// Constructed #1913 fixtures isolate the split grid from corpus row pairing.
fn grid_document(grid: &[u32], rows: &str, fixed: bool) -> Vec<u8> {
    let cols: String = grid
        .iter()
        .map(|w| format!(r#"<w:gridCol w:w="{w}"/>"#))
        .collect();
    let layout = if fixed {
        r#"<w:tblLayout w:type="fixed"/>"#
    } else {
        ""
    };
    common::docx::docx(&format!(
        r#"<w:p><w:r><w:t>Before</w:t></w:r></w:p><w:tbl><w:tblPr>{layout}</w:tblPr><w:tblGrid>{cols}</w:tblGrid>{rows}</w:tbl><w:p><w:r><w:t>After</w:t></w:r></w:p>"#
    ))
}

fn grid_row(spans: &[usize], properties: &str, text: &str) -> String {
    let cells: String = spans.iter().enumerate().map(|(i, span)| format!(r#"<w:tc><w:tcPr><w:gridSpan w:val="{span}"/></w:tcPr><w:p><w:r><w:t>{text} {i}</w:t></w:r></w:p></w:tc>"#)).collect();
    format!("<w:tr><w:trPr>{properties}</w:trPr>{cells}</w:tr>")
}

fn assert_split_grid(
    grid: &[u32],
    deleted: &str,
    expected: &[u32],
    spans: &[Vec<usize>],
    skips: &[(usize, usize)],
) {
    let kept = grid_row(&vec![1; grid.len()], "", "Keep this exact cell");
    let a = grid_document(grid, &format!("{kept}{deleted}"), false);
    let b = grid_document(grid, &kept, true);
    let output = compare(&a, &b); // Existing Ring-1 Word-validity helper.
    let pkg = PartFs::open(&output).unwrap();
    assert!(
        jubarte::validate::ring1(&pkg).is_empty(),
        "jubarte validate Ring-1"
    );
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&pkg.part_string("word/document.xml").unwrap());
    let body = dom.element(dom.root(doc).unwrap(), &W::body()).unwrap();
    let tables = dom.elements(body, Some(&W::tbl()));
    assert_eq!(tables.len(), 2, "one live and one deleted run");
    let split = tables[1];
    let g = dom.element(split, &W::tbl_grid()).unwrap();
    let widths: Vec<u32> = dom
        .elements(g, Some(&W::grid_col()))
        .iter()
        .map(|&c| dom.attribute(c, &W::name("w")).unwrap().parse().unwrap())
        .collect();
    assert_eq!(widths, expected, "#1913 split-off grid");
    assert!(
        dom.element(g, &W::name("tblGridChange")).is_none(),
        "Word saves a split deleted table without a grid change"
    );
    let count = |pr: Option<NodeId>, name: &str, default: usize| {
        pr.and_then(|pr| dom.element(pr, &W::name(name)))
            .and_then(|n| dom.attribute(n, &W::val()))
            .map(|v| v.parse().unwrap())
            .unwrap_or(default)
    };
    let rows = dom.elements(split, Some(&W::tr()));
    let actual: Vec<Vec<usize>> = rows
        .iter()
        .map(|&r| {
            dom.elements(r, Some(&W::tc()))
                .iter()
                .map(|&c| count(dom.element(c, &W::tc_pr()), "gridSpan", 1))
                .collect()
        })
        .collect();
    assert_eq!(actual, spans, "cell spans in the rebuilt grid");
    let actual: Vec<_> = rows
        .iter()
        .map(|&r| {
            let pr = dom.element(r, &W::tr_pr());
            (count(pr, "gridBefore", 0), count(pr, "gridAfter", 0))
        })
        .collect();
    assert_eq!(actual, skips, "row omissions in the rebuilt grid");
    for (&row, &(before, after)) in rows.iter().zip(skips) {
        let pr = dom.element(row, &W::tr_pr()).unwrap();
        for (name, count, width) in [
            ("wBefore", before, expected[..before].iter().sum::<u32>()),
            (
                "wAfter",
                after,
                expected[expected.len() - after..].iter().sum::<u32>(),
            ),
        ] {
            if let Some(el) = dom.element(pr, &W::name(name)) {
                assert_ne!(count, 0, "no stale omission width");
                assert_eq!(dom.attribute(el, &W::name("w")).unwrap(), width.to_string());
            }
        }
    }
    // Integration gate: exercise the same output through the public CLI.
    let dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let path = dir.path().join("split.docx");
    std::fs::write(&path, &output).unwrap();
    let validation = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .arg("validate")
        .arg(&path)
        .args(["--rules", "word"])
        .output()
        .unwrap();
    assert!(
        validation.status.success(),
        "validate: {}{}",
        String::from_utf8_lossy(&validation.stdout),
        String::from_utf8_lossy(&validation.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&validation.stdout).trim(),
        "no findings"
    );
}

#[test]
fn split_grid_1913_be486e3b25_drops_unused_trailing_columns() {
    assert_split_grid(
        &[4606, 4606],
        &grid_row(
            &[1],
            r#"<w:gridAfter w:val="1"/><w:wAfter w:w="4606" w:type="dxa"/>"#,
            "Deleted",
        ),
        &[4606],
        &[vec![1]],
        &[(0, 0)],
    );
}

#[test]
fn split_grid_1913_81ecf9d6d2_full_width_cell() {
    assert_split_grid(
        &[4159, 3329, 3420],
        &grid_row(&[3], "", "Deleted"),
        &[10908],
        &[vec![1]],
        &[(0, 0)],
    );
}

#[test]
fn split_grid_1913_70c30e926c_full_width_cell() {
    assert_split_grid(
        &[6570, 2880],
        &grid_row(&[2], "", "Deleted"),
        &[9450],
        &[vec![1]],
        &[(0, 0)],
    );
}

#[test]
fn split_grid_1913_b13a27e36d_three_columns_from_eight() {
    let rows = format!(
        "{}{}",
        grid_row(&[3, 3, 2], "", "Deleted"),
        grid_row(&[8], "", "Removed")
    );
    assert_split_grid(
        &[1663, 2193, 1331, 952, 2740, 1494, 3805, 1381],
        &rows,
        &[5187, 5186, 5186],
        &[vec![1, 1, 1], vec![3]],
        &[(0, 0), (0, 0)],
    );
}

#[test]
fn split_grid_1913_fea87b305c_removes_seven_twip_sliver() {
    assert_split_grid(
        &[3091, 7, 5928],
        &grid_row(&[2, 1], "", "Deleted"),
        &[3098, 5928],
        &[vec![1, 1]],
        &[(0, 0)],
    );
}

#[test]
fn split_grid_1913_reindexes_omissions_and_preserves_their_widths() {
    let rows = format!(
        "{}{}",
        grid_row(&[6], "", "Deleted"),
        grid_row(
            &[2],
            r#"<w:gridBefore w:val="2"/><w:gridAfter w:val="2"/><w:wBefore w:w="300" w:type="dxa"/><w:wAfter w:w="1100" w:type="dxa"/>"#,
            "Removed"
        )
    );
    assert_split_grid(
        &[100, 200, 300, 400, 500, 600],
        &rows,
        &[300, 700, 1100],
        &[vec![3], vec![1]],
        &[(0, 0), (1, 1)],
    );
}

#[test]
fn split_grid_1913_trims_only_the_common_trailing_omission() {
    let rows = format!(
        "{}{}",
        grid_row(
            &[2],
            r#"<w:gridAfter w:val="2"/><w:wAfter w:w="700" w:type="dxa"/>"#,
            "Deleted"
        ),
        grid_row(
            &[3],
            r#"<w:gridAfter w:val="1"/><w:wAfter w:w="400" w:type="dxa"/>"#,
            "Removed"
        )
    );
    assert_split_grid(
        &[100, 200, 300, 400],
        &rows,
        &[300, 300],
        &[vec![1], vec![2]],
        &[(0, 1), (0, 0)],
    );
}

#[test]
fn split_grid_1913_keeps_leading_omission_widths() {
    let rows = format!(
        "{}{}",
        grid_row(
            &[2],
            r#"<w:gridBefore w:val="2"/><w:gridAfter w:val="2"/><w:wBefore w:w="300" w:type="dxa"/><w:wAfter w:w="1100" w:type="dxa"/>"#,
            "Deleted"
        ),
        grid_row(
            &[3],
            r#"<w:gridBefore w:val="1"/><w:gridAfter w:val="2"/><w:wBefore w:w="100" w:type="dxa"/><w:wAfter w:w="1100" w:type="dxa"/>"#,
            "Removed"
        )
    );
    assert_split_grid(
        &[100, 200, 300, 400, 500, 600],
        &rows,
        &[100, 200, 700],
        &[vec![1], vec![2]],
        &[(2, 0), (1, 0)],
    );
}
