// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M37 — merged Normal style (M-PAG mechanism 2). In word mode, when A's
//! Normal pPr spacing differs from B's EFFECTIVE Normal spacing, the output
//! styles.xml Normal must carry the revised effective spacing with a
//! `w:pPrChange` recording A's old pPr. GT evidence
//! (sd-2517_sectpr-headerref): B's Normal has an EMPTY pPr, which Word
//! resolves to FACTORY defaults after=160 line=278 lineRule=auto; the GT
//! stylesheet's Normal = that resolved spacing + pPrChange holding the old
//! value. Identity comparisons must leave Normal untouched.

use std::io::{Cursor, Read};

use jubarte::document_comparer::compare_documents;

const ORIG: &str = "tests/corpus/_fixtures/original_fixtures";

fn orig_fixtures_present() -> bool {
    if std::path::Path::new(ORIG).is_dir() {
        true
    } else {
        eprintln!("SKIP: _fixtures/original_fixtures corpus not present");
        false
    }
}

fn read_part(docx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name(name).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

fn normal_style(styles_xml: &str) -> String {
    // crude but sufficient: first <w:style ...>Normal...</w:style> block
    let idx = styles_xml
        .find("w:styleId=\"Normal\"")
        .expect("Normal style present");
    let start = styles_xml[..idx].rfind("<w:style ").unwrap();
    let end = styles_xml[idx..].find("</w:style>").unwrap() + idx + "</w:style>".len();
    styles_xml[start..end].to_string()
}

/// A = sd-2517-localized-heading-styles (Normal spacing after=0 line=240
/// lineRule=auto), B = sectpr-headerref (Normal pPr absent → factory
/// 160/278/auto). Output Normal must be the revised effective spacing with a
/// pPrChange holding A's old value.
#[test]
fn normal_spacing_merged_with_pprchange() {
    if !orig_fixtures_present() {
        return;
    }
    let a = std::fs::read(format!("{ORIG}/sd-2517-localized-heading-styles.docx")).unwrap();
    let b = std::fs::read(format!("{ORIG}/sectpr-headerref.docx")).unwrap();
    let out = compare_documents(&a, &b, "Test Author").expect("compare ok");
    let normal = normal_style(&read_part(&out, "word/styles.xml"));

    assert!(
        normal.contains("w:after=\"160\"")
            && normal.contains("w:line=\"278\"")
            && normal.contains("w:lineRule=\"auto\""),
        "Normal must carry B's effective (factory-resolved) spacing 160/278/auto, got: {normal}"
    );
    assert!(
        normal.contains("<w:pPrChange"),
        "Normal must carry a pPrChange recording the old pPr, got: {normal}"
    );
    // old value = A's spacing after=0 line=240
    let chg_start = normal.find("<w:pPrChange").unwrap();
    let chg = &normal[chg_start..];
    assert!(
        chg.contains("w:after=\"0\"") && chg.contains("w:line=\"240\""),
        "pPrChange must hold A's old spacing (after=0 line=240), got: {chg}"
    );
    assert!(
        chg.contains("w:author=") && chg.contains("w:date="),
        "pPrChange must carry author/date, got: {chg}"
    );
}

/// Overfire guard: a styles-neutral pair where BOTH sides' Normal stores no
/// pPr (both resolve to factory defaults) must leave Normal untouched — no
/// injected spacing, no pPrChange.
#[test]
fn both_empty_normals_left_untouched() {
    if !orig_fixtures_present() {
        return;
    }
    let a = std::fs::read(format!("{ORIG}/1-5-line-spacing.id-paraid-overflow.docx")).unwrap();
    let b = std::fs::read(format!("{ORIG}/24.id-paraid-overflow.docx")).unwrap();
    let out = compare_documents(&a, &b, "Test Author").expect("compare ok");
    let normal = normal_style(&read_part(&out, "word/styles.xml"));
    assert!(
        !normal.contains("pPrChange") && !normal.contains("<w:spacing"),
        "styles-neutral pair: Normal must stay untouched, got: {normal}"
    );
}

/// Identity comparison (a document against itself) must leave Normal
/// untouched — same spacing, no pPrChange.
#[test]
fn identity_pair_leaves_normal_untouched() {
    if !orig_fixtures_present() {
        return;
    }
    let a = std::fs::read(format!("{ORIG}/sd-2517-localized-heading-styles.docx")).unwrap();
    let out = compare_documents(&a, &a, "Test Author").expect("compare ok");
    let normal = normal_style(&read_part(&out, "word/styles.xml"));
    assert!(
        normal.contains("w:after=\"0\"") && normal.contains("w:line=\"240\""),
        "identity: Normal spacing must stay A's (0/240), got: {normal}"
    );
    assert!(
        !normal.contains("pPrChange"),
        "identity: no pPrChange on Normal, got: {normal}"
    );
}
