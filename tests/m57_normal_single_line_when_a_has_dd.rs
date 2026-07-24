// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M57 — when both Normals store no spacing but A has a **structured** Normal
//! (pPr/rPr) and docDefaults, Word rewrites Normal to after=0 line=240.
//! file_77_file_78: without this, LO renders 5pp vs Word 3pp.
//! M60: bare+bare with only A dd must NOT promote (file_145).

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

#[test]
fn file_77_normal_gets_single_line_when_a_has_docdefaults() {
    let Some((a, b)) = corpus_pair("file_77.docx", "file_78.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let normal = normal_block(&styles_xml(&out));
    assert!(
        normal.contains("w:after=\"0\"") && normal.contains("w:line=\"240\""),
        "Word writes after=0 line=240 for file_77 class, got: {normal}"
    );
    assert!(
        normal.contains("pPrChange"),
        "must record A's old Normal pPr: {normal}"
    );
}

#[test]
fn file_69_still_leaves_bare_normal_empty() {
    let Some((a, b)) = corpus_pair("file_69.docx", "file_70.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let normal = normal_block(&styles_xml(&out));
    assert!(
        !normal.contains("<w:spacing") && !normal.contains("pPrChange"),
        "file_69 bare Normal must stay empty: {normal}"
    );
}

/// Both sides carry the same docDefaults (after=200 line=276) with empty
/// Normal — Word leaves Normal empty. Promoting dd into Normal costs ~18
/// score points on file_8 (page bloat).
#[test]
fn file_8_both_sides_docdefaults_leave_normal_empty() {
    let Some((a, b)) = corpus_pair("file_8.docx", "file_9.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let normal = normal_block(&styles_xml(&out));
    assert!(
        !normal.contains("w:after=\"200\"") && !normal.contains("w:line=\"276\""),
        "must not materialize shared docDefaults into Normal: {normal}"
    );
}

/// A bare of dd but Normal has rPr; B has dd after=200/line=276. Word promotes
/// B's dd onto Normal (file_34_file_35).
#[test]
fn file_34_promotes_b_docdefaults_when_a_normal_has_rpr() {
    let Some((a, b)) = corpus_pair("file_34.docx", "file_35.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let normal = normal_block(&styles_xml(&out));
    assert!(
        normal.contains("w:after=\"200\"") && normal.contains("w:line=\"276\""),
        "Word promotes B docDefaults into Normal: {normal}"
    );
}

/// Differing docDefaults (A 200/276, B 160/278) and B Normal has non-spacing
/// pPr → Word writes B's values (M60: bare+bare differing dd leaves empty).
#[test]
fn file_46_writes_b_docdefaults_when_dds_differ() {
    let Some((a, b)) = corpus_pair("file_46.docx", "file_47.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let normal = normal_block(&styles_xml(&out));
    assert!(
        normal.contains("w:after=\"160\"") && normal.contains("w:line=\"278\""),
        "Word writes B's differing docDefaults: {normal}"
    );
}
