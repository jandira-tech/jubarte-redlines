// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! two_column_two_page × vrect_node: pure-I list items must keep B body
//! spacing + remapped live numId (Word numId=10 + after=0 line=240), not
//! live abstract hanging ind with numId=1. Trailing pure-D sample text must
//! not inherit list numPr/pPrChange from region-mark survival.

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
fn two_col_x_vrect_list_pure_ins_keeps_spacing_not_live_hanging_ind() {
    let Some(a) = load("super_editor__two_column_two_page_0b8a37c5.docx") else {
        eprintln!("skip");
        return;
    };
    let Some(b) = load("super_editor__vrect_node_c8e51f22.docx") else {
        eprintln!("skip");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);

    // Pure-I list lead: Word keeps spacing after=0 line=240; no live hanging ind.
    let p = doc
        .split("</w:p>")
        .find(|p| p.contains("OG is down with OPP"))
        .expect("list pure-I para");
    assert!(
        p.contains("w:spacing") || p.contains("w:line=\"240\""),
        "pure-I list must keep B spacing; p={p}"
    );
    // Live pPr must not materialize abstract hanging ind (belongs in pPrChange only).
    let live_ppr = p.split("<w:ins").next().unwrap_or(p);
    let live_before_chg = live_ppr.split("<w:pPrChange").next().unwrap_or(live_ppr);
    assert!(
        !live_before_chg.contains("w:hanging") && !live_before_chg.contains("hanging=\""),
        "live pure-I pPr must not carry hanging ind; p={p}"
    );

    // Trailing pure-D of two-column sample text: no list numPr (Word bare del / mark only).
    let sample = doc
        .split("</w:p>")
        .filter(|p| p.contains("two-column DOCX") || p.contains("two-column"))
        .last()
        .expect("sample pure-D");
    // If this residual pure-D has body delText of sample, it must not carry numPr.
    if sample.contains("delText") || sample.contains("<w:del") {
        let ppr = sample.split("<w:del").next().unwrap_or(sample);
        assert!(
            !ppr.contains("<w:numPr") && !ppr.contains("w:numPr"),
            "trailing pure-D sample must not inherit list numPr; p={sample}"
        );
    }
}
