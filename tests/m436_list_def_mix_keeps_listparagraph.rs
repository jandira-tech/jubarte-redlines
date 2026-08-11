// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M436 — list free-mesh MIX keeps Deleted ListParagraph + numPr + del mark.
//!
//! list_def_mix × list_numbering_reimport: Word MIX of last pure-I "test" into
//! first pure-D "Num 1" keeps live `pStyle=ListParagraph`, base `numPr`, and a
//! para-mark `w:del`. Engine kept Inserted numId only (no ListParagraph, no
//! del mark) — LO ~52 vs docxodus ~90.

use std::io::Read;
use std::path::PathBuf;

use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::compare_documents_with_settings;

#[test]
fn list_def_mix_x_numbering_mix_keeps_listparagraph() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__list_def_mix_d7cec092.docx");
    let b = src.join("super_editor__list_numbering_reimport_d788d573.docx");
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

    let mut found = false;
    let mut rest = xml.as_str();
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
        if !(has_ins && has_del) {
            continue;
        }
        let lower = p.to_ascii_lowercase();
        if !(lower.contains("test") && lower.contains("num")) {
            continue;
        }
        found = true;
        let ppr = p
            .find("<w:pPr")
            .map(|i| &p[i..])
            .and_then(|s| s.find("</w:pPr>").map(|j| &s[..j + 7]));
        let Some(ppr) = ppr else {
            panic!("MIX test×Num missing pPr: {}", &p[..p.len().min(300)]);
        };
        let live = if let Some(i) = ppr.find("<w:pPrChange") {
            &ppr[..i]
        } else {
            ppr
        };
        assert!(
            live.contains("ListParagraph"),
            "Word keeps live ListParagraph on list free-mesh MIX; live pPr was {live}"
        );
        assert!(
            live.contains("numPr") || live.contains("w:numPr"),
            "Word keeps live numPr on list free-mesh MIX"
        );
        assert!(
            live.contains("<w:del") || live.contains("w:del"),
            "Word keeps del mark on list free-mesh MIX pPr/rPr"
        );
        break;
    }
    assert!(found, "expected MIX free-mesh of test × Num 1");
}
