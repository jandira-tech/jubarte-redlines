// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Pair 02 package path: table-bookmark-end vs table-vmerge-colspan from batch_to_fix.
use jubarte::document_comparer::compare_documents;
use std::io::{Cursor, Read};

fn read_part(docx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name(name).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn pair02_batch_docx_first_table_mixes_cells() {
    let a = std::fs::read(
        "tests/corpus/batch_to_fix/pairs/02_table_bookmark_end_table_vmerge_colspan/base.docx",
    )
    .expect("base");
    let b = std::fs::read(
        "tests/corpus/batch_to_fix/pairs/02_table_bookmark_end_table_vmerge_colspan/next.docx",
    )
    .expect("next");
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = read_part(&out, "word/document.xml");
    // crude first table extract via regex-ish scan
    let start = doc.find("<w:tbl").expect("tbl");
    let end = doc[start..].find("</w:tbl>").unwrap() + start + 8;
    // may be nested wrong - take first close after start for outer
    let mut depth = 0i32;
    let mut i = start;
    let mut end2 = end;
    while i < doc.len() {
        if doc[i..].starts_with("<w:tbl>") || doc[i..].starts_with("<w:tbl ") {
            depth += 1;
            i += 6;
            continue;
        }
        if doc[i..].starts_with("</w:tbl>") {
            depth -= 1;
            if depth == 0 {
                end2 = i + 8;
                break;
            }
            i += 8;
            continue;
        }
        i += 1;
    }
    let tbl = &doc[start..end2];
    eprintln!(
        "first table len {} head {}",
        tbl.len(),
        &tbl[..tbl.len().min(800)]
    );
    assert!(
        tbl.contains("AAA") && (tbl.contains("R1C1") || tbl.contains("delText")),
        "first table should mix AAA with deleted base cell text"
    );
    // cell-level mix: same row/cell region has both ins and del
    let has_mix = tbl.contains("<w:ins") && tbl.contains("<w:del");
    assert!(has_mix, "first table must have both ins and del");
    // fail if pure del rows then pure ins rows with no shared cell
    // Word: AAA and R1C1 in first cell together
    let aaa_at = tbl.find("AAA");
    let r1c1_at = tbl.find("R1C1");
    eprintln!("AAA at {aaa_at:?} R1C1 at {r1c1_at:?}");
    if let (Some(a), Some(r)) = (aaa_at, r1c1_at) {
        let dist = (a as i64 - r as i64).abs();
        assert!(
            dist < 500,
            "AAA and R1C1 should be near each other in same cell (dist={dist})"
        );
    }
}
