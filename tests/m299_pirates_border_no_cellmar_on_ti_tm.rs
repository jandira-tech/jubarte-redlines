// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M299 — pirates×table_border_widths: Word omits synthesized `tblCellMar` /
//! `tblInd` on pure-I (TI) and mixed (TM) bordered tables, not only pure-D
//! (M296). Tip still added mar10/ind10 on every bordered revised table.
//! Skip synthesize whenever the table carries revision marks (ins or del).

use jubarte::document_comparer::compare_documents;
use std::io::{Cursor, Read};
use std::path::Path;

fn load_sd(name: &str) -> Option<Vec<u8>> {
    let p = Path::new(
        "/Users/arthrod/temp/T/neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source",
    )
    .join(name);
    std::fs::read(p).ok()
}

fn load_be(name: &str) -> Option<Vec<u8>> {
    // behavior fixtures live under same superdoc or word_based
    for root in [
        "/Users/arthrod/temp/T/neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source",
        "/Users/arthrod/temp/T/neurotic_docx_bench/corpus/word_based/docx_source",
    ] {
        let p = Path::new(root).join(name);
        if let Ok(b) = std::fs::read(&p) {
            return Some(b);
        }
    }
    None
}

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn pirates_x_border_ti_tm_tables_have_no_synthesized_cellmar() {
    let Some(a) = load_sd("super_editor__sd_2766_pirates_tracked_changes_3285d875.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let Some(b) = load_be("behavior__sd_2343_table_border_widths_b5148e83.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);

    let mut rest = doc.as_str();
    let mut revised_bordered = 0;
    let mut revised_with_cellmar = 0;
    let mut revised_with_ind10 = 0;
    while let Some(start) = rest.find("<w:tbl") {
        rest = &rest[start..];
        let end = rest.find("</w:tbl>").map(|i| i + 8).unwrap_or(rest.len());
        let tbl = &rest[..end];
        rest = &rest[end..];
        let has_borders = tbl.contains("<w:tblBorders");
        let has_del = tbl.contains("<w:delText") || tbl.contains("<w:del ");
        let has_ins = tbl.contains("<w:ins ") || tbl.contains("<w:ins>");
        if has_borders && (has_del || has_ins) {
            revised_bordered += 1;
            if tbl.contains("<w:tblCellMar") {
                revised_with_cellmar += 1;
            }
            // Synthesized ind is w="10"; source may carry other tblInd.
            if tbl.contains(r#"w:tblInd"#) && tbl.contains(r#"w:w="10""#) {
                revised_with_ind10 += 1;
            }
        }
    }
    assert!(
        revised_bordered >= 3,
        "expected several TI/TM bordered tables; n={revised_bordered}"
    );
    assert_eq!(
        revised_with_cellmar, 0,
        "revised bordered tables must not get synthesized tblCellMar (Word omits on pirates×border)"
    );
    assert_eq!(
        revised_with_ind10, 0,
        "revised bordered tables must not get synthesized tblInd w=10 (Word omits)"
    );
}
