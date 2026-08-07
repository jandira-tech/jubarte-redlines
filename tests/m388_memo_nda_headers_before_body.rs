// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M388 — memo×nda: short pure-D memo headers before pure-I NDA body.
//!
//! Word after title MIX: pure-D TO/FROM/DATE/RE then NDA body. Eng dumps
//! pure-I NDA first and parks headers midstream (~46 pagefair).

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn memo_x_nda_to_header_before_amazing_corp_body() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("evals__memorandum_258c774a.docx");
    let b = src.join("evals__nda_7f304918.docx");
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
    let mut pos_to: Option<usize> = None;
    let mut pos_amazing: Option<usize> = None;
    let mut idx = 0usize;
    let mut scan = rest;
    while let Some(start) = scan.find("<w:p") {
        let after = &scan[start..];
        let end_rel = after
            .find("</w:p>")
            .map(|j| j + "</w:p>".len())
            .or_else(|| after.find("/>").map(|j| j + 2));
        let Some(end_rel) = end_rel else { break };
        let p = &after[..end_rel];
        scan = &after[end_rel..];
        if pos_to.is_none()
            && p.contains("TO:")
            && p.contains("General Counsel")
            && p.contains("<w:del")
        {
            pos_to = Some(idx);
        }
        if pos_amazing.is_none()
            && p.contains("Amazing Corp")
            && p.contains("<w:ins")
            && !p.contains("<w:del")
        {
            pos_amazing = Some(idx);
        }
        idx += 1;
    }
    let (Some(to), Some(am)) = (pos_to, pos_amazing) else {
        panic!("expected pure-D TO: and pure-I Amazing Corp; to={pos_to:?} am={pos_amazing:?}");
    };
    assert!(
        to < am,
        "Word parks pure-D memo TO: before pure-I NDA body; to={to} amazing={am}"
    );
}
