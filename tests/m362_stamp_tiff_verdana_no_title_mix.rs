// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M362 — stamp file_173×174: keep pure-I TIFF title separate from Verdana.
//!
//! Word/27c: X stamp, pure-I "TIFF test document", Xd Verdana+drawing, pure-D
//! body (XIXdDD). Content×content fold MIX-ed TIFF+Verdana titles (XXXdD)
//! thrash pagefair 100→67. Fold trailing empty/drawing pure-I with first
//! pure-D title instead.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn file_173_x_174_tiff_title_pure_i() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_based/docx_source_randomized");
    let a = src.join("file_173.docx");
    let b = src.join("file_174.docx");
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

    // No single para with both TIFF title and Verdana title.
    let mut rest = xml.as_str();
    if let Some(i) = rest.find("<w:body") {
        rest = &rest[i..];
    }
    if let Some(i) = rest.find("</w:body>") {
        rest = &rest[..i];
    }
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        let end_rel = after
            .find("</w:p>")
            .map(|j| j + "</w:p>".len())
            .or_else(|| after.find("/>").map(|j| j + 2));
        let Some(end_rel) = end_rel else { break };
        let p = &after[..end_rel];
        rest = &after[end_rel..];
        let mut t = String::new();
        for tag in ["<w:t", "<w:delText"] {
            let mut s = p;
            while let Some(i) = s.find(tag) {
                let s2 = &s[i..];
                if let Some(j) = s2.find('>') {
                    let s3 = &s2[j + 1..];
                    let end = if tag == "<w:t" {
                        s3.find("</w:t>")
                    } else {
                        s3.find("</w:delText>")
                    };
                    if let Some(k) = end {
                        t.push_str(&s3[..k]);
                        s = &s3[k..];
                        continue;
                    }
                }
                break;
            }
        }
        let has_tiff = t.contains("TIFF test document");
        let has_verdana = t.contains("Verdana Italic Centered Demo");
        assert!(
            !(has_tiff && has_verdana),
            "Word pure-I TIFF then Xd Verdana; must not MIX titles; got {t:?}"
        );
    }
    assert!(
        xml.contains("TIFF test document"),
        "expected pure-I TIFF title"
    );
}
