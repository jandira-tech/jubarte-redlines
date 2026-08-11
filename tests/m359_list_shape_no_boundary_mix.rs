// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M359 — list_with_indents × shape_group: Word pure-I/D, no shape+list MIX.
//!
//! Word/27c: pure-I all shapes then pure-D list residual (IIII…DDDD).
//! M345 empty pure-I × long list pure-D folded first list item into an empty
//! shell; multi-del then MIX-ed last "My test with some shapes." with list
//! text (pagefair −9.4). Skip whitespace fold when pure-D is long prose or
//! pure-I has a drawing.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn list_with_indents_x_shape_group_no_shape_list_mix() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__list_with_indents_efc7d4f5.docx");
    let b = src.join("super_editor__shape_group_ce60e1e6.docx");
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

    // No single para may carry both shape title and list residual text.
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
        let has_shape = t.to_ascii_lowercase().contains("shapes");
        let has_list = t.contains("um has been");
        assert!(
            !(has_shape && has_list),
            "Word pure-I/D: must not MIX shape title with list residual; got {t:?}"
        );
    }

    // Shape titles should appear as pure-I (ins without del) somewhere.
    assert!(
        xml.contains("My test with some shapes."),
        "expected pure-I shape title text"
    );
}
