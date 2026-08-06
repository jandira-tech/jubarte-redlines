// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M266 — pirates×table_left_indent: Word emits pure-D A table as two tables
//! (2+3 rows: Flats/Square then Circle/Cone/triangle). First slice keeps B
//! tblPr (ind=1440)+tblPrChange; remainder uses A props (ind=108). Tip kept
//! one 5-row pure-D under B props.

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

fn table_shapes(doc: &str) -> Vec<(usize, Vec<String>, bool, bool, bool)> {
    // (nrows, row_texts, has_del, has_ins, has_tblPrChange)
    let Some(body) = doc
        .split("<w:body>")
        .nth(1)
        .and_then(|b| b.split("</w:body>").next())
    else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut rest = body;
    while let Some(s) = rest.find("<w:tbl") {
        rest = &rest[s..];
        let Some(e) = rest.find("</w:tbl>") else {
            break;
        };
        let chunk = &rest[..e + "</w:tbl>".len()];
        rest = &rest[e + "</w:tbl>".len()..];
        let has_del = chunk.contains("<w:del ") || chunk.contains("<w:del>");
        let has_ins = chunk.contains("<w:ins ") || chunk.contains("<w:ins>");
        let has_chg = chunk.contains("tblPrChange");
        let mut rows = Vec::new();
        let mut row_rest = chunk;
        while let Some(rs) = row_rest.find("<w:tr") {
            row_rest = &row_rest[rs..];
            let Some(re) = row_rest.find("</w:tr>") else {
                break;
            };
            let row = &row_rest[..re + "</w:tr>".len()];
            row_rest = &row_rest[re + "</w:tr>".len()..];
            let mut text = String::new();
            for part in row.split("<w:delText").skip(1) {
                if let (Some(a), Some(b)) = (part.find('>'), part.find("</w:delText>")) {
                    text.push_str(&part[a + 1..b]);
                }
            }
            for part in row.split("<w:t").skip(1) {
                if let (Some(a), Some(b)) = (part.find('>'), part.find("</w:t>")) {
                    text.push_str(&part[a + 1..b]);
                }
            }
            rows.push(text);
        }
        out.push((rows.len(), rows, has_del, has_ins, has_chg));
    }
    out
}

#[test]
fn pirates_x_table_indent_pure_d_table_splits_2_plus_3() {
    let Some(a) = load("super_editor__sd_2766_pirates_tracked_changes_3285d875.docx") else {
        eprintln!("skip");
        return;
    };
    let Some(b) = load("super_editor__sd_1494_table_left_indent_11bb24c7.docx") else {
        eprintln!("skip");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    let shapes = table_shapes(&doc);

    let flats = shapes
        .iter()
        .find(|(_, rows, has_del, _, _)| *has_del && rows.iter().any(|r| r.contains("Flats")));
    let circle = shapes
        .iter()
        .find(|(_, rows, has_del, _, _)| *has_del && rows.iter().any(|r| r.contains("Circle")));
    let (fn_rows, f_rows, _, _, f_chg) = flats.expect("Flats pure-D table");
    let (cn_rows, c_rows, _, _, c_chg) = circle.expect("Circle pure-D table");

    let flats_idx = shapes
        .iter()
        .position(|(_, rows, has_del, _, _)| *has_del && rows.iter().any(|r| r.contains("Flats")))
        .unwrap();
    let circle_idx = shapes
        .iter()
        .position(|(_, rows, has_del, _, _)| *has_del && rows.iter().any(|r| r.contains("Circle")))
        .unwrap();
    assert_ne!(
        flats_idx, circle_idx,
        "Flats and Circle must be distinct tables; shapes={shapes:?}"
    );
    assert_eq!(*fn_rows, 2, "first pure-D slice 2 rows; rows={f_rows:?}");
    assert_eq!(
        *cn_rows, 3,
        "remainder pure-D slice 3 rows; rows={c_rows:?}"
    );
    assert!(
        f_rows.iter().any(|r| r.contains("Square")),
        "2-row slice includes Square: {f_rows:?}"
    );
    assert!(
        c_rows.iter().any(|r| r.contains("Cone")) && c_rows.iter().any(|r| r.contains("triangle")),
        "3-row slice includes Cone+triangle: {c_rows:?}"
    );
    // First slice keeps B-merged props (tblPrChange); remainder uses A props.
    assert!(
        *f_chg,
        "first pure-D slice should retain tblPrChange (B props live)"
    );
    assert!(
        !*c_chg,
        "remainder pure-D slice should use A props without tblPrChange"
    );
}
