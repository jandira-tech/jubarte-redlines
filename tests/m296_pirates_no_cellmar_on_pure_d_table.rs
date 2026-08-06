// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M296 — pirates×table_indent: Word pure-D bordered tables do **not** carry
//! synthesized `tblCellMar left/right=10`. Tip `synthesize_table_cell_margins`
//! added mar10 to every bordered table including pure-D (pirates T1).
//! Skip synthesize on pure-D tables (del present, no ins).

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
fn pirates_pure_d_bordered_table_has_no_tbl_cell_mar() {
    let Some(a) = load("super_editor__sd_2766_pirates_tracked_changes_3285d875.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load("super_editor__sd_1494_table_left_indent_11bb24c7.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);

    // Find pure-D tables: contain delText, no w:ins in table body.
    // Split tables roughly.
    let mut rest = doc.as_str();
    let mut pure_d_with_borders = 0;
    let mut pure_d_with_cellmar = 0;
    while let Some(start) = rest.find("<w:tbl") {
        rest = &rest[start..];
        let end = rest.find("</w:tbl>").map(|i| i + 8).unwrap_or(rest.len());
        let tbl = &rest[..end];
        rest = &rest[end..];
        let has_borders = tbl.contains("<w:tblBorders");
        let has_del = tbl.contains("<w:delText") || tbl.contains("<w:del ");
        let has_ins = tbl.contains("<w:ins ") || tbl.contains("<w:ins>");
        if has_borders && has_del && !has_ins {
            pure_d_with_borders += 1;
            if tbl.contains("<w:tblCellMar") {
                pure_d_with_cellmar += 1;
            }
        }
    }
    assert!(
        pure_d_with_borders >= 1,
        "expected pure-D bordered table (pirates mid tables)"
    );
    assert_eq!(
        pure_d_with_cellmar, 0,
        "pure-D bordered tables must not get synthesized tblCellMar (Word omits)"
    );
}
