// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M396 — file_34 comprehensive × file_35 strikethrough short Demo.
//!
//! Word multi-MIX nests short residual into the head of the long residual
//! (stamp + titles), then pure-D remaining long. M131 modest-cap ≤40 missed
//! comprehensive residual (~70 groups) → pure-I short + pure-D long.

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
fn file_34_x_35_multi_mix_titles_not_pure_i_short() {
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
    let n_i = cls.iter().filter(|&&c| c == 'I').count();
    let n_d = cls.iter().filter(|&&c| c == 'D').count();
    // Word ≈ M3 D59 M2 D6 E4 (MIX≥5). Pre-M396 pure-I short residual (~I2)
    // then pure-D long with MIX≈2.
    assert!(
        n_m >= 3,
        "Word multi-MIX nests strikethrough into comprehensive head; got MIX={n_m} I={n_i} D={n_d} {cls:?}"
    );
    // Must not be wholesale pure-I short then pure-D all long (I-block ≤6 then D≥50).
    let pure_i_then_d = n_i <= 6 && n_d >= 50 && n_m <= 2;
    assert!(
        !pure_i_then_d,
        "must not pure-I short Demo then pure-D comprehensive; MIX={n_m} I={n_i} D={n_d}"
    );
}
