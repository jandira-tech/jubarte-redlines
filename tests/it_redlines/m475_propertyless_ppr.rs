// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A paragraph property that carries nothing is not in Word's redline.
//!
//! The short title mix (blue centered title × blue italic) grows an empty
//! `w:pPrChange` around an empty `w:pPr`. Word's redline has no paragraph
//! properties on that title. The center-bold × clear-formatting redline has
//! no `w:pPr` anywhere; three empty `w:pPr` elements were the whole difference.

use std::io::Read;
use std::path::PathBuf;

use jubarte::document_comparer::compare_documents;

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx.to_vec())).expect("zip");
    let mut f = zip.by_name("word/document.xml").expect("document.xml");
    let mut xml = String::new();
    f.read_to_string(&mut xml).expect("utf8");
    xml
}

fn paragraphs(xml: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = xml;
    loop {
        let i = match (rest.find("<w:p "), rest.find("<w:p>")) {
            (Some(a), Some(b)) => a.min(b),
            (Some(a), None) => a,
            (None, Some(b)) => b,
            (None, None) => break,
        };
        let after = &rest[i..];
        let Some(j) = after.find("</w:p>") else { break };
        out.push(after[..j + 6].to_string());
        rest = &after[j + 6..];
    }
    out
}

fn compare_pair(a_name: &str, b_name: &str) -> Option<String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("tests/corpus/neurotic_docx_bench/corpus/word_based/docx_source");
    let a = src.join(a_name);
    let b = src.join(b_name);
    if !a.exists() || !b.exists() {
        eprintln!("skip: fixtures missing");
        return None;
    }
    let out = compare_documents(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        "Redline",
    )
    .expect("compare");
    Some(document_xml(&out))
}

#[test]
fn mixed_title_has_no_empty_ppr_change() {
    let Some(xml) = compare_pair(
        "blue_centered_title_demo_style_default_missing.docx",
        "blue_italic_text_demo_id_paraid_overflow.docx",
    ) else {
        return;
    };
    let title = paragraphs(&xml)
        .into_iter()
        .find(|p| p.contains("Centered Title") && p.contains("Italic Text"))
        .expect("mixed title paragraph");
    assert!(
        !title.contains("pPrChange"),
        "Word's title mix has no pPrChange: {title}"
    );
    assert!(
        !title.contains("<w:pPr"),
        "Word's title mix has no pPr: {title}"
    );
}

#[test]
fn center_bold_redline_has_no_empty_ppr() {
    let Some(xml) = compare_pair(
        "center_bold_demo_id_paraid_overflow.docx",
        "clear_formatting_demo_id_paraid_overflow.docx",
    ) else {
        return;
    };
    assert!(
        !xml.contains("<w:pPr"),
        "Word's redline of this pair has no w:pPr"
    );
}
