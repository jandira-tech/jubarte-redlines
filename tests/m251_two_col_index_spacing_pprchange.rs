// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M251 — two_col_index × two_col_tab_positions: Word keeps live `w:spacing
//! line=276` on equal page-label paras and records `w:pPrChange` with old
//! tabs-only pPr (A had tabs, B added spacing). Pre-M251: tip stripped line=276
//! (demo-default strip) and emitted zero pPrChange (M130 requires empty old).

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
fn two_col_index_x_tab_has_spacing_pprchange() {
    let Some(a) = load("super_editor__sd_1480_two_col_index_0138dccc.docx") else {
        eprintln!("skip: two_col_index source missing");
        return;
    };
    let Some(b) = load("super_editor__sd_1480_two_col_tab_positions_00953280.docx") else {
        eprintln!("skip: two_col_tab source missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);

    let n_change = doc.matches("<w:pPrChange").count();
    // Word: pPrChange on p1, p2, p3 (and pure-del residual p6). ≥3 is the
    // equal page-label body set that must surface spacing addition.
    assert!(
        n_change >= 3,
        "expected ≥3 pPrChange for tabs→tabs+spacing page labels, got {n_change}"
    );
    // Live spacing must survive demo-default strip when pPrChange records it.
    assert!(
        doc.contains("w:line=\"276\""),
        "live line=276 must remain on page-label paras with pPrChange history"
    );
    assert!(
        doc.contains("w:spacing"),
        "live w:spacing required (Word two_col_index×tab)"
    );
}

#[test]
fn two_col_index_page4_no_trailing_ws_del() {
    let Some(a) = load("super_editor__sd_1480_two_col_index_0138dccc.docx") else {
        eprintln!("skip: two_col_index source missing");
        return;
    };
    let Some(b) = load("super_editor__sd_1480_two_col_tab_positions_00953280.docx") else {
        eprintln!("skip: two_col_tab source missing");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    // Word: Page4 is clean equal (no trailing whitespace-only del on that para).
    // Locate the paragraph whose only digit body is equal "4" after "Page".
    let mut page4_ok = false;
    for p in doc.split("</w:p>") {
        if !p.contains(">Page<") || !p.contains(">4<") {
            continue;
        }
        // Strip pPr so mark-level ins does not look like body ins.
        let body = if let Some(rest) = p.split("</w:pPr>").nth(1) {
            rest
        } else {
            p
        };
        // Skip Page lines that also carry other digits (1/2/3/5/6/7/8).
        let other_digit = [">1<", ">2<", ">3<", ">5<", ">6<", ">7<", ">8<"]
            .iter()
            .any(|d| body.contains(d));
        if other_digit {
            continue;
        }
        // Trailing whitespace-only del after equal 4 is the glue bug.
        let has_ws_only_del = body.split("<w:delText").skip(1).any(|s| {
            let Some(start) = s.find('>') else {
                return false;
            };
            let Some(end) = s.find("</w:delText>") else {
                return false;
            };
            let t = &s[start + 1..end];
            !t.is_empty() && t.chars().all(char::is_whitespace)
        });
        assert!(
            !has_ws_only_del,
            "Page4 equal para must not end with whitespace-only del"
        );
        page4_ok = true;
    }
    assert!(page4_ok, "expected Page4 equal para in redline");
}
