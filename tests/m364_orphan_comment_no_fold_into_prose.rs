// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M364 — orphan_comment × yellow_highlight: Word keeps pure-D "Ouch" separate.
//!
//! Base is a short commented residual ("Ouch.") and next is a multi-para
//! yellow-highlight demo. Word leaves pure-I "Highlighted text…" then pure-D
//! "Ouch" with comment anchors (pagefair 93.8). Folding invents a MIX that
//! attaches comments to the highlight paragraph (−6.4).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn orphan_comment_x_yellow_highlight_ouch_stays_pure_d() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_based/docx_source");
    let a = src.join("word_tolerated_orphan_comment.docx");
    let b = src.join("yellow_highlight_demo_id_paraid_overflow.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip: fixtures missing");
        return;
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

    let mut rest = xml.as_str();
    if let Some(i) = rest.find("<w:body") {
        rest = &rest[i..];
    }
    if let Some(i) = rest.find("</w:body>") {
        rest = &rest[..i];
    }
    let mut found_highlight = false;
    let mut found_ouch_pure_d = false;
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        let end_rel = after
            .find("</w:p>")
            .map(|j| j + "</w:p>".len())
            .or_else(|| after.find("/>").map(|j| j + 2));
        let Some(end_rel) = end_rel else { break };
        let p = &after[..end_rel];
        rest = &after[end_rel..];
        if p.contains("Highlighted text stands out") {
            found_highlight = true;
            assert!(p.contains("<w:ins"), "highlight body should be inserted");
            assert!(
                !p.contains("Ouch") && !p.contains("<w:delText"),
                "Word keeps pure-D Ouch separate — no fold into highlight MIX: {p}"
            );
            assert!(
                !p.contains("commentReference") && !p.contains("commentRange"),
                "comments stay on pure-D Ouch, not on highlight pure-I"
            );
        }
        if p.contains("Ouch") {
            found_ouch_pure_d = true;
            assert!(
                p.contains("<w:del") && !p.contains("<w:ins"),
                "Ouch must stay pure-D with comments"
            );
            assert!(
                p.contains("commentReference") || p.contains("commentRange"),
                "pure-D Ouch must keep comment anchors"
            );
        }
    }
    assert!(found_highlight, "expected Highlighted text pure-I");
    assert!(found_ouch_pure_d, "expected pure-D Ouch with comments");
}
