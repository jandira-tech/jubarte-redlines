// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M459 — wholesale body MIX free-mesh is **disabled** after full-ITT thrash.
//!
//! file_163×164 −29 and ooxml_style_link×open_sans −12. These tests lock the
//! disabled state: wholesale residuals must not be free-meshed.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

fn compare_pair(a_name: &str, b_name: &str) -> Option<String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_based/docx_source");
    let src_r = root.join("../neurotic_docx_bench/corpus/word_based/docx_source_randomized");
    let a = if src.join(a_name).exists() {
        src.join(a_name)
    } else {
        src_r.join(a_name)
    };
    let b = if src.join(b_name).exists() {
        src.join(b_name)
    } else {
        src_r.join(b_name)
    };
    if !a.exists() || !b.exists() {
        return None;
    }
    let out = compare_documents_with_settings(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        &WmlComparerSettings {
            author_for_revisions: "Redline".into(),
            merge_replaced_paragraphs: true,
            ..WmlComparerSettings::default()
        },
    )
    .expect("compare");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut xml = String::new();
    f.read_to_string(&mut xml).unwrap();
    Some(xml)
}

#[test]
fn file_163_164_wholesale_del_not_free_meshed() {
    let Some(xml) = compare_pair("file_163.docx", "file_164.docx") else {
        eprintln!("skip: fixtures missing");
        return;
    };
    assert!(
        xml.contains("traditional for formal academic papers"),
        "wholesale del residual free-meshed despite M459 disable"
    );
}

#[test]
fn ooxml_style_link_not_free_meshed() {
    let Some(xml) = compare_pair(
        "ooxml_style_link.docx",
        "open_sans_bold_underline_id_paraid_overflow.docx",
    ) else {
        eprintln!("skip: fixtures missing");
        return;
    };
    assert!(
        !xml.contains(">OOXML w b<") && !xml.contains("OOXML w b"),
        "ooxml technical del free-meshed despite M459 disable; fragment present"
    );
    // Wholesale OOXML del should remain as a long delText run.
    assert!(
        xml.contains("OOXML") && xml.contains("tester"),
        "expected OOXML tester del content"
    );
}
