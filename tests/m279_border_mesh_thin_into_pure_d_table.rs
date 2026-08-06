// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M279 — pirates×table_border_widths after M278: Word meshes thin pure-I
//! table `A1|B1` into pure-D pirates table first row (`A1`|`B1`+del Flats).
//! Tip left separate TD then TI. Mesh pure-I cells into pure-D table.

use jubarte::document_comparer::compare_documents;
use std::io::{Cursor, Read};
use std::path::Path;

fn load(name: &str) -> Option<Vec<u8>> {
    let p = Path::new(
        "/Users/arthrod/temp/T/neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source",
    )
    .join(name);
    std::fs::read(p).ok()
}

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn border_widths_x_pirates_first_table_has_ins_a1_and_del_flats() {
    let Some(a) = load("super_editor__sd_2766_pirates_tracked_changes_3285d875.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load("behavior__sd_2343_table_border_widths_b5148e83.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    // First table after pure-D residual should be MIX: ins A1/B1 and del Flats.
    let Some(tbl_start) = doc.find("<w:tbl") else {
        panic!("no table");
    };
    // Find table that contains Flats delText (pirates residual table).
    let mut search = &doc[tbl_start..];
    let mut found_mix = false;
    while let Some(rel) = search.find("<w:tbl") {
        let chunk = &search[rel..];
        let end = chunk.find("</w:tbl>").unwrap_or(chunk.len());
        let tbl = &chunk[..end];
        if tbl.contains("Flats") || tbl.contains("delText") && tbl.contains("Square") {
            let has_a1 = tbl.contains(">A1<") || tbl.contains(">A1</");
            let has_b1 = tbl.contains(">B1<") || tbl.contains(">B1</");
            let has_ins = tbl.contains("<w:ins");
            let has_del = tbl.contains("<w:del") || tbl.contains("delText");
            assert!(
                has_ins && has_del,
                "pirates residual table must be MIX after mesh"
            );
            assert!(
                has_a1 || has_b1,
                "first row should carry thin-table ins A1/B1; snippet={}",
                &tbl[..tbl.len().min(400)]
            );
            found_mix = true;
            break;
        }
        search = &chunk[end.saturating_add(1)..];
    }
    assert!(
        found_mix,
        "expected pure-D pirates table with Flats residual"
    );
}
