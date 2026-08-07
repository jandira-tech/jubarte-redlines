// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M397b — short Demo × OOXML bold: Word pure-I short residual first.
//!
//! Free-meshing OOXML long residual on share=0 thrash-rewrote file_2
//! CenterBoldDemo×OOXML (95→44). Word pure-Is short Demo then pure-Ds OOXML
//! (I2M1D32). file_41 same family after M397b gate.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

fn body_classes(xml: &str) -> Vec<char> {
    let mut rest = xml;
    if let Some(i) = rest.find("<w:body") {
        rest = &rest[i..];
    }
    if let Some(i) = rest.find("</w:body>") {
        rest = &rest[..i];
    }
    let mut out = Vec::new();
    while let Some(start) = rest.find("<w:p") {
        let after = &rest[start..];
        let end_rel = after
            .find("</w:p>")
            .map(|j| j + "</w:p>".len())
            .or_else(|| after.find("/>").map(|j| j + 2));
        let Some(end_rel) = end_rel else { break };
        let p = &after[..end_rel];
        rest = &after[end_rel..];
        let has_ins = p.contains("<w:ins");
        let has_del = p.contains("<w:del") || p.contains("<w:delText");
        out.push(match (has_ins, has_del) {
            (true, true) => 'M',
            (true, false) => 'I',
            (false, true) => 'D',
            (false, false) => 'E',
        });
    }
    out
}

#[test]
fn file_2_center_bold_demo_pure_i_short_first_not_ooxml_head_mesh() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_based/docx_source_randomized");
    let a = src.join("file_2.docx");
    let b = src.join("file_3.docx");
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
    let cls = body_classes(&xml);
    // After stamp, first non-stamp content should be pure-I short Demo title
    // (not pure-D OOXML title). Word: M stamp, I Center Aligned..., I body.
    let first_content = cls
        .iter()
        .enumerate()
        .find(|&(_, &c)| c != 'M' && c != 'E');
    let Some((i, &c)) = first_content else {
        panic!("expected content paras");
    };
    assert_eq!(
        c, 'I',
        "Word pure-Is short Demo first; got {c} at {i} classes={cls:?}"
    );
    // Must have pure-I short residual before bulk pure-D OOXML.
    let n_i = cls.iter().filter(|&&c| c == 'I').count();
    let n_d = cls.iter().filter(|&&c| c == 'D').count();
    assert!(
        n_i >= 2 && n_d >= 20,
        "expected pure-I short + pure-D OOXML bulk; I={n_i} D={n_d} {cls:?}"
    );
}

#[test]
fn file_34_still_multi_mix_after_m397b() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_based/docx_source_randomized");
    let a = src.join("file_34.docx");
    let b = src.join("file_35.docx");
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
    let cls = body_classes(&xml);
    let n_m = cls.iter().filter(|&&c| c == 'M').count();
    assert!(
        n_m >= 3,
        "M396 file_34 multi-MIX must survive M397b; MIX={n_m} {cls:?}"
    );
}
