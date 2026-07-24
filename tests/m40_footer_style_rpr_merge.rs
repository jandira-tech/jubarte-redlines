// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M40 — footer/header linked-style run metrics (M-PAG mechanism 2b). In word
//! mode, when header/footer parts reference styles whose basedOn chain
//! resolves through Normal, and the output Normal's EFFECTIVE run metrics
//! (rFonts/sz/szCs, docDefaults-resolved) differ from the REVISED document's,
//! the output Normal must carry the revised effective rPr with a
//! `w:rPrChange` recording the old value. GT evidence (sample-document ×
//! sd-2517-localized-heading-styles): GT Normal rPr = Times New Roman
//! sz=24/szCs=24 (B's stored Normal rPr + B docDefaults eastAsia) with an
//! rPrChange holding A's docDefaults-resolved Inter sz=22 — this is what
//! makes GT footers render a 13.3pt line box vs our 12.2pt, flipping ~4
//! knife-edge footer-spill blank pages.

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
    let idx = styles_xml
        .find("w:styleId=\"Normal\"")
        .expect("Normal style present");
    let start = styles_xml[..idx].rfind("<w:style ").unwrap();
    let end = styles_xml[idx..].find("</w:style>").unwrap() + idx + "</w:style>".len();
    styles_xml[start..end].to_string()
}

/// A = sample-document (Normal has no stored rPr → docDefaults Inter sz=22),
/// B = sd-2517-localized-heading-styles (Normal rPr Times New Roman
/// sz=24/szCs=24; docDefaults eastAsia=Times New Roman). B's footers
/// reference Rodap/Nmerodepgina which resolve through Normal. Output Normal
/// must carry B's effective run metrics with an rPrChange holding A's old
/// effective rPr.
#[test]
fn footer_linked_normal_rpr_merged_with_rprchange() {
    if !orig_fixtures_present() {
        return;
    }
    let a = std::fs::read(format!(
        "{ORIG}/sample-document.word-repair-of-our-output-word-repaired.docx"
    ))
    .unwrap();
    let b = std::fs::read(format!("{ORIG}/sd-2517-localized-heading-styles.docx")).unwrap();
    let out = compare_documents(&a, &b, "Test Author").expect("compare ok");
    let normal = normal_style(&read_part(&out, "word/styles.xml"));

    assert!(
        normal.contains("w:ascii=\"Times New Roman\"")
            && normal.contains("w:hAnsi=\"Times New Roman\"")
            && normal.contains("w:eastAsia=\"Times New Roman\"")
            && normal.contains("w:cs=\"Times New Roman\""),
        "Normal must carry B's effective rFonts (Times New Roman), got: {normal}"
    );
    assert!(
        normal.contains("<w:sz w:val=\"24\"") && normal.contains("<w:szCs w:val=\"24\""),
        "Normal must carry B's effective sz/szCs 24, got: {normal}"
    );
    assert!(
        normal.contains("<w:rPrChange"),
        "Normal must carry an rPrChange recording the old rPr, got: {normal}"
    );
    // old value = A's effective (docDefaults-resolved) rPr: Inter sz=22
    let chg_start = normal.find("<w:rPrChange").unwrap();
    let chg = &normal[chg_start..];
    assert!(
        chg.contains("w:ascii=\"Inter\"") && chg.contains("<w:sz w:val=\"22\""),
        "rPrChange must hold A's old effective rPr (Inter sz=22), got: {chg}"
    );
    assert!(
        chg.contains("w:author=") && chg.contains("w:date="),
        "rPrChange must carry author/date, got: {chg}"
    );
}

/// Overfire guard: a styles-neutral pair (both sides' effective Normal rPr
/// resolve identically) must leave Normal's run properties untouched — no
/// injected rFonts/sz, no rPrChange anywhere in the stylesheet.
#[test]
fn styles_neutral_pair_normal_rpr_untouched() {
    if !orig_fixtures_present() {
        return;
    }
    let a = std::fs::read(format!("{ORIG}/1-5-line-spacing.id-paraid-overflow.docx")).unwrap();
    let b = std::fs::read(format!("{ORIG}/24.id-paraid-overflow.docx")).unwrap();
    let out = compare_documents(&a, &b, "Test Author").expect("compare ok");
    let styles = read_part(&out, "word/styles.xml");
    assert!(
        !styles.contains("rPrChange"),
        "styles-neutral pair: no rPrChange may appear in styles.xml"
    );
    let normal = normal_style(&styles);
    assert!(
        !normal.contains("<w:rFonts") && !normal.contains("<w:sz"),
        "styles-neutral pair: Normal must stay untouched, got: {normal}"
    );
}

/// Identity comparison must leave Normal's rPr exactly as stored — no
/// rPrChange.
#[test]
fn identity_pair_leaves_normal_rpr_untouched() {
    if !orig_fixtures_present() {
        return;
    }
    let a = std::fs::read(format!("{ORIG}/sd-2517-localized-heading-styles.docx")).unwrap();
    let out = compare_documents(&a, &a, "Test Author").expect("compare ok");
    let normal = normal_style(&read_part(&out, "word/styles.xml"));
    assert!(
        !normal.contains("rPrChange"),
        "identity: no rPrChange on Normal, got: {normal}"
    );
    assert!(
        normal.contains("w:ascii=\"Times New Roman\"") && normal.contains("<w:sz w:val=\"24\""),
        "identity: Normal keeps its stored rPr, got: {normal}"
    );
}
