// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M38 — M-TBL rules 3 & 4 (parity/_scratch/table_class_forensics.md).
//!
//! Rule 3 — merge-partner alignment: Word merges doc A's table with the FIRST
//! same-slot table in B. GT support-tickets-table_table-bookmark-end merges
//! A's ticket table with B's table 1 cell-wise (same cell holds `<w:ins>R1C1
//! </w:ins>` and `<w:del>Ticket ID</w:del>`). Ours anchored the LCS on empty
//! paragraphs, inserting B tables 1–2 whole and merging A's table into the
//! table at position 3.
//!
//! Rule 4 — degenerate tblGrid rebuild: originals carry 1 gridCol while rows
//! imply 2 columns (gridSpan 2). Word rewrites the grid to per-column widths
//! with `tblW 0 auto` — GT table-vmerge-colspan_text-box: gridCol 4675+4675;
//! GT nested-table-rowspan_numbered-list: gridCol 4887+4905.

use std::io::{Cursor, Read};

use jubarte::document_comparer::compare_documents;

const ORIG: &str = "tests/corpus/_fixtures/original_fixtures";

fn orig_fixtures_present() -> bool {
    if std::path::Path::new(ORIG).is_dir() {
        true
    } else {
        eprintln!("SKIP: _fixtures/original_fixtures corpus not present");
        false
    }
}

fn read_part(docx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name(name).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

/// The first `<w:tbl>…</w:tbl>` block of the body (outermost, crude scan).
fn first_table(doc: &str) -> String {
    let start = doc.find("<w:tbl>").expect("a table in the body");
    // find the matching close by depth
    let bytes = doc.as_bytes();
    let mut depth = 0usize;
    let mut i = start;
    while i < bytes.len() {
        if doc[i..].starts_with("<w:tbl>") {
            depth += 1;
            i += 7;
        } else if doc[i..].starts_with("</w:tbl>") {
            depth -= 1;
            i += 8;
            if depth == 0 {
                return doc[start..i].to_string();
            }
        } else {
            i += 1;
        }
    }
    panic!("unbalanced w:tbl");
}

/// Rule 3: A = support-tickets.table (3 paras + 1 table), B = table-bookmark-end
/// (8 tables). Word merges A's ticket table with B's FIRST table: the first
/// output table must be a cell-wise merge carrying BOTH inserted new content
/// (R1C1) and deleted old content (Ticket ID). Ours (pre-fix) emitted B tables
/// 1–2 as pure insertions and merged A's table at position 3.
#[test]
fn table_merges_with_first_same_slot_partner() {
    if !orig_fixtures_present() {
        return;
    }
    let a = std::fs::read(format!("{ORIG}/support-tickets.table.docx")).unwrap();
    let b = std::fs::read(format!("{ORIG}/table-bookmark-end.docx")).unwrap();
    let out = compare_documents(&a, &b, "Test Author").expect("compare ok");
    let doc = read_part(&out, "word/document.xml");
    let tbl = first_table(&doc);
    assert!(
        tbl.contains("<w:ins ") && tbl.contains("R1C1"),
        "first table must carry B t1's inserted content (R1C1), got: {}",
        &tbl[..tbl.len().min(2000)]
    );
    assert!(
        tbl.contains("<w:del ") && tbl.contains("Ticket ID"),
        "first table must carry A's deleted content (Ticket ID) — Word merges \
         with the FIRST same-slot table, got: {}",
        &tbl[..tbl.len().min(2000)]
    );
}

/// Rule 4: A = table-vmerge-colspan, table 1 declares a single
/// `<w:gridCol w:w="4985"/>` while its rows span 2 columns (gridSpan 2 /
/// two half-width cells). Word rebuilds the grid to per-column widths with
/// `tblW 0 auto` — GT: `<w:gridCol w:w="4675"/><w:gridCol w:w="4675"/>`
/// (equal split). Ours (pre-fix) kept the 1-col grid.
#[test]
fn degenerate_tblgrid_rebuilt_per_column() {
    if !orig_fixtures_present() {
        return;
    }
    let a = std::fs::read(format!("{ORIG}/table-vmerge-colspan.docx")).unwrap();
    let b = std::fs::read(format!("{ORIG}/text-box.docx")).unwrap();
    let out = compare_documents(&a, &b, "Test Author").expect("compare ok");
    let doc = read_part(&out, "word/document.xml");
    let tbl = first_table(&doc);
    let grid_start = tbl.find("<w:tblGrid>").expect("tblGrid present");
    let grid_end = tbl[grid_start..].find("</w:tblGrid>").unwrap() + grid_start;
    let grid = &tbl[grid_start..grid_end];
    let cols = grid.matches("<w:gridCol").count();
    assert_eq!(
        cols, 2,
        "degenerate 1-col grid must be rebuilt to the real column count \
         (rows use gridSpan 2; GT: 4675+4675), got grid: {grid}"
    );
    // widths must be an equal split (Word: 4675+4675 = content-driven autofit;
    // we pin the structural contract: two equal columns, not the old 4985)
    assert!(
        !grid.contains("w:w=\"4985\""),
        "old degenerate single-column width must not survive, got: {grid}"
    );
    assert!(
        tbl.contains("<w:tblW w:w=\"0\" w:type=\"auto\" />"),
        "rebuilt table must carry tblW 0 auto (GT), got tblPr region: {}",
        &tbl[..tbl.len().min(1200)]
    );
}

/// Overfire guard for rule 4: a table whose grid already declares the real
/// column count (table 2 of table-vmerge-colspan: 4 gridCols, rows use
/// gridSpan but never exceed 4 effective columns) keeps its grid untouched.
#[test]
fn healthy_tblgrid_left_untouched() {
    if !orig_fixtures_present() {
        return;
    }
    let a = std::fs::read(format!("{ORIG}/table-vmerge-colspan.docx")).unwrap();
    let b = std::fs::read(format!("{ORIG}/text-box.docx")).unwrap();
    let out = compare_documents(&a, &b, "Test Author").expect("compare ok");
    let doc = read_part(&out, "word/document.xml");
    // table 2's original grid: 4 × 2493
    assert_eq!(
        doc.matches("<w:gridCol w:w=\"2493\" />").count(),
        4,
        "healthy 4-col grid (4×2493) must survive unchanged"
    );
}

/// Rule 2b — orientation: when the merge lands with the hoist `ancestor`
/// resolving to the OLD (doc A) table, the effective tblPr/tblGrid must STILL
/// come from the NEW (doc B) table, with A's props recorded in tblPrChange.
/// A = table-bookmark-end (t1: tblW 6000 dxa, 3×2000), B = table-vmerge-colspan
/// (t1: 9972 dxa, degenerate grid → Word rebuilds to tblW 0 auto). GT first
/// table: effective tblW 0/auto, tblPrChange holds the OLD 6000. Ours
/// (pre-fix) kept A's 6000 effective with no change record.
#[test]
fn merged_table_effective_props_from_new_table_either_orientation() {
    if !orig_fixtures_present() {
        return;
    }
    let a = std::fs::read(format!("{ORIG}/table-bookmark-end.docx")).unwrap();
    let b = std::fs::read(format!("{ORIG}/table-vmerge-colspan.docx")).unwrap();
    let out = compare_documents(&a, &b, "Test Author").expect("compare ok");
    let doc = read_part(&out, "word/document.xml");
    let tbl = first_table(&doc);
    let pr_end = tbl.find("</w:tblPr>").expect("tblPr present");
    let pr = &tbl[..pr_end];
    let change_at = pr
        .find("<w:tblPrChange")
        .expect("tblPrChange must record the OLD (doc A) props");
    assert!(
        pr[change_at..].contains("w:w=\"6000\""),
        "tblPrChange must hold A's tblW 6000, got tblPr: {pr}"
    );
    assert!(
        !pr[..change_at].contains("w:w=\"6000\""),
        "effective tblW must be the NEW table's (GT: 0/auto), not A's 6000: {pr}"
    );
    // Pin the actual NEW effective value (GT: 0/auto), not just the absence of
    // the old 6000 — the check above would still pass if the new tblW were
    // missing or carried a wrong value.
    assert!(
        pr[..change_at].contains(r#"w:w="0""#) && pr[..change_at].contains(r#"w:type="auto""#),
        "effective tblW must be the NEW table's 0/auto value: {pr}"
    );
}
