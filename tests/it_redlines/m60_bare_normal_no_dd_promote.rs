// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M60 — when both Normals store no spacing and both are structurally bare
//! (no pPr/rPr), Word leaves Normal empty even if docDefaults differ or only
//! A has dd. Promoting into Normal page-bloats LO (file_19 6pp vs Word 5pp;
//! file_145 class residual ~41).

use std::io::{Cursor, Read};
use std::path::Path;

use jubarte::document_comparer::compare_documents;

fn corpus_pair(a: &str, b: &str) -> Option<(Vec<u8>, Vec<u8>)> {
    let root = Path::new("tests/corpus/broken_ones_two/sources");
    let ap = root.join(a);
    let bp = root.join(b);
    if ap.is_file() && bp.is_file() {
        Some((std::fs::read(ap).ok()?, std::fs::read(bp).ok()?))
    } else {
        None
    }
}

fn styles_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/styles.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

fn normal_block(styles: &str) -> String {
    let idx = styles
        .find("w:styleId=\"Normal\"")
        .expect("Normal style present");
    let start = styles[..idx].rfind("<w:style ").unwrap();
    let end = styles[idx..].find("</w:style>").unwrap() + idx + "</w:style>".len();
    styles[start..end].to_string()
}

/// file_19: A dd 120/240, B dd 200/276, both bare Normal → Word empty Normal.
#[test]
fn m60_file_19_differing_dd_both_bare_leaves_normal_empty() {
    let Some((a, b)) = corpus_pair("file_19.docx", "file_20.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let normal = normal_block(&styles_xml(&out));
    assert!(
        !normal.contains("<w:spacing") && !normal.contains("pPrChange"),
        "file_19 bare+differing-dd must not promote B into Normal: {normal}"
    );
}

/// file_145: A dd 200/276, B no dd, both bare → Word empty (not after=0/line=240).
#[test]
fn m60_file_145_a_dd_b_none_both_bare_leaves_normal_empty() {
    let Some((a, b)) = corpus_pair("file_145.docx", "file_146.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let normal = normal_block(&styles_xml(&out));
    assert!(
        !normal.contains("w:after=\"0\"")
            && !normal.contains("w:line=\"240\"")
            && !normal.contains("pPrChange"),
        "file_145 bare must not get single-line Normal: {normal}"
    );
}

/// file_46 still promotes when B Normal has non-spacing pPr + differing dd.
#[test]
fn m60_file_46_still_writes_b_dd_when_b_normal_structured() {
    let Some((a, b)) = corpus_pair("file_46.docx", "file_47.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let normal = normal_block(&styles_xml(&out));
    assert!(
        normal.contains("w:after=\"160\"") && normal.contains("w:line=\"278\""),
        "file_46 B-structured still writes B dd: {normal}"
    );
}

/// file_33: A has dd, B has rPr on Normal → still single-line 0/240.
#[test]
fn m60_file_33_still_single_line_when_b_has_rpr() {
    let Some((a, b)) = corpus_pair("file_33.docx", "file_34.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let normal = normal_block(&styles_xml(&out));
    assert!(
        normal.contains("w:after=\"0\"") && normal.contains("w:line=\"240\""),
        "file_33 B-rPr still gets single-line Normal: {normal}"
    );
}

/// file_77: A has Normal pPr, B none → still single-line 0/240.
#[test]
fn m60_file_77_still_single_line_when_a_has_ppr() {
    let Some((a, b)) = corpus_pair("file_77.docx", "file_78.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let normal = normal_block(&styles_xml(&out));
    assert!(
        normal.contains("w:after=\"0\"") && normal.contains("w:line=\"240\""),
        "file_77 A-pPr still gets single-line Normal: {normal}"
    );
}

/// file_196: B Normal stores line=276 without after → Word writes after=0 line=276.
#[test]
fn m60_file_196_fills_missing_after_when_line_present() {
    let Some((a, b)) = corpus_pair("file_196.docx", "file_197.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let normal = normal_block(&styles_xml(&out));
    assert!(
        normal.contains("w:after=\"0\"") && normal.contains("w:line=\"276\""),
        "Word fills after=0 when B only stores line: {normal}"
    );
}
