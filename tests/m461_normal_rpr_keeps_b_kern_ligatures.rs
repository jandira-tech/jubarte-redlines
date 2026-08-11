// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M461 — the Normal rPr merge must carry B's stored `w:kern` and
//! `w14:ligatures` into the live output rPr, not only fonts/sz/szCs.
//!
//! Oracle: super_editor__basic_comment_d3ba5f1e × cli_legacy__sample_3a8f1f93.
//! B's Normal stores `kern=0` + `w14:ligatures none`; A has no stored Normal
//! rPr and its docDefaults say `kern=2` + `standardContextual`. Word's output
//! Normal keeps B's kern/ligatures. Dropping them leaves kerning ON from A's
//! docDefaults, narrowing every long paragraph by a glyph or two — the "Jane
//! Wilde" paragraph renders 7 lines vs the oracle's 8, shifting the rest of
//! the page by one line height (13.8pt) and compounding across pages.

use std::io::Read;
use std::path::PathBuf;

use jubarte::document_comparer::compare_documents;

fn style_elem(xml: &str, sid: &str) -> Option<String> {
    let i = xml.find(&format!("w:styleId=\"{sid}\""))?;
    let start = xml[..i].rfind("<w:style ")?;
    let seg = &xml[start..];
    let end = seg.find("</w:style>")?;
    Some(seg[..end].to_string())
}

/// The live (non-rPrChange) portion of a style block.
fn live(style: &str) -> String {
    match style.find("<w:rPrChange") {
        Some(i) => style[..i].to_string(),
        None => style.to_string(),
    }
}

#[test]
fn normal_rpr_merge_carries_b_kern_and_ligatures() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__basic_comment_d3ba5f1e.docx");
    let b = src.join("cli_legacy__sample_3a8f1f93.docx");
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
    let mut f = zip.by_name("word/styles.xml").unwrap();
    let mut xml = String::new();
    f.read_to_string(&mut xml).unwrap();

    let normal = live(&style_elem(&xml, "Normal").expect("Normal style present"));
    assert!(
        normal.contains("<w:kern w:val=\"0\""),
        "live Normal rPr must keep B's kern=0, got: {normal}"
    );
    assert!(
        normal.contains("w14:ligatures w14:val=\"none\""),
        "live Normal rPr must keep B's ligatures=none, got: {normal}"
    );
}
