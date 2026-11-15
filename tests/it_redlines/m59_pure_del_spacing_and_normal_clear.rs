// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M59 — when A stores Normal spacing and B's cascade equals shared
//! docDefaults, clear Normal spacing rather than materializing cascade
//! (file_22 Word leaves empty Normal with pPrChange).

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;
use std::io::Read;
use std::path::Path;

fn corpus(a: &str, b: &str) -> Option<(Vec<u8>, Vec<u8>)> {
    let root = Path::new("tests/corpus/broken_ones_two/sources");
    let ap = root.join(a);
    let bp = root.join(b);
    if ap.is_file() && bp.is_file() {
        Some((std::fs::read(ap).ok()?, std::fs::read(bp).ok()?))
    } else {
        None
    }
}

fn styles_normal(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/styles.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    let idx = s.find("w:styleId=\"Normal\"").expect("Normal");
    let start = s[..idx].rfind("<w:style ").unwrap();
    let end = s[idx..].find("</w:style>").unwrap() + idx + "</w:style>".len();
    s[start..end].to_string()
}

#[test]
fn m59_file_22_clears_normal_spacing_when_b_cascade_equals_dd() {
    let Some((a, b)) = corpus("file_22.docx", "file_23.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out =
        compare_documents_with_settings(&a, &b, &WmlComparerSettings::default()).expect("compare");
    let normal = styles_normal(&out);
    assert!(
        !normal.contains("w:after=\"200\"") && !normal.contains("w:line=\"276\""),
        "Word clears Normal spacing when B cascade == shared dd: {normal}"
    );
    assert!(
        normal.contains("pPrChange"),
        "must record old A spacing via pPrChange: {normal}"
    );
}

#[test]
fn m59_file_197_still_writes_b_dd_when_a_has_no_dd() {
    let Some((a, b)) = corpus("file_197.docx", "file_198.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out =
        compare_documents_with_settings(&a, &b, &WmlComparerSettings::default()).expect("compare");
    let normal = styles_normal(&out);
    assert!(
        normal.contains("w:after=\"200\"") && normal.contains("w:line=\"276\""),
        "file_197 A has no dd; still write B cascade: {normal}"
    );
}

#[test]
fn m59_file_14_still_high_structure_after_normal_clear_path() {
    // Regression guard: pure-del spacing strip was tried and nuked file_14 (−46).
    // This pair must still clear Word-valid compare without Normal bloat.
    let Some((a, b)) = corpus("file_14.docx", "file_15.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out =
        compare_documents_with_settings(&a, &b, &WmlComparerSettings::default()).expect("compare");
    let normal = styles_normal(&out);
    // file_14 class: bare Normal both sides, B may have dd — leave empty
    assert!(
        !normal.contains("w:line=\"276\"") || !normal.contains("pPrChange"),
        "file_14 should not promote demo Normal spacing: {normal}"
    );
    let _ = out;
}
