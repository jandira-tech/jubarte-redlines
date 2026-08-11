// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M469 — a head title MIX pairing B's SHORT inserted title (≤4 tokens) with
//! A's LONG deleted opening (≥10 tokens, no shared significant token) splits:
//! Word keeps B's title paragraph pure-INS and moves A's deleted text into
//! its own style-less paragraph with a deleted paragraph mark (bare pPr, no
//! pStyle → renders at body size).
//!
//! Oracle: super_editor__ooxml_rfonts_rstyle_linked_combos ×
//! behavior__sd_2672_rtl_table (47.5 vs docxodus 93.5): our fold kept the
//! 30-token deleted tester-description inside the Heading1 title paragraph,
//! rendering six title-size strike lines and shifting the whole document
//! ~50pt; the oracle shows three compact body-size lines. Family:
//! ooxml_*_rstyle_linked_combos (~15-20 pairs at 47-60).

use std::io::Read;
use std::path::PathBuf;

use jubarte::document_comparer::compare_documents;

#[test]
fn head_title_mix_splits_long_del_into_bare_paragraph() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__ooxml_rfonts_rstyle_linked_combos_dem_213298de.docx");
    let b = src.join("behavior__sd_2672_rtl_table_63bd9d10.docx");
    if !a.exists() || !b.exists() {
        eprintln!("skip: fixtures missing");
        return;
    }
    let out = compare_documents(
        &std::fs::read(&a).unwrap(),
        &std::fs::read(&b).unwrap(),
        "Redline",
    )
    .expect("compare");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut xml = String::new();
    f.read_to_string(&mut xml).unwrap();

    let mut rest = xml.as_str();
    let mut title_para = None;
    let mut del_para = None;
    loop {
        let i = match (rest.find("<w:p "), rest.find("<w:p>")) {
            (Some(a), Some(b)) => a.min(b),
            (Some(a), None) => a,
            (None, Some(b)) => b,
            (None, None) => break,
        };
        let after = &rest[i..];
        let Some(j) = after.find("</w:p>") else { break };
        let p = &after[..j];
        rest = &after[j + 6..];
        if p.contains("SD-2672 RTL") {
            title_para = Some(p.to_string());
        }
        if p.contains("OOXML w:rFonts tester") {
            del_para = Some(p.to_string());
        }
    }
    let title = title_para.expect("title paragraph present");
    let del = del_para.expect("deleted opening present");
    assert!(
        !title.contains("OOXML w:rFonts tester"),
        "B's short title must not swallow A's long deleted opening"
    );
    assert!(
        !del.contains("pStyle"),
        "deleted opening lives in a style-less paragraph, got: {}",
        &del[..del.len().min(300)]
    );
    assert!(
        del.contains("<w:rPr><w:del ") || del.contains("<w:rPr>\n"),
        "deleted opening paragraph carries a deleted paragraph mark, got: {}",
        &del[..del.len().min(300)]
    );
}
