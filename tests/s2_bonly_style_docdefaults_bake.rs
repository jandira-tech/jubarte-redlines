// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Workstream S phase 2 — styles copied from B (absent in A) must bake B's
//! docDefaults-level run metrics and carry style-level change records.
//!
//! Oracle: image_inline_and_block × rtl_page_numpages. A's stylesheet never
//! declares Footer/FootnoteText/Strong1; B's does, and B's docDefaults are
//! Times New Roman with no sz/kern/ligatures (implicit 20/0/none) while A's
//! are theme fonts + sz 24 + kern 2 + ligatures standardContextual. Word's
//! output styles.xml declares each copied style with the neutralizing delta
//! live (rFonts TNR, sz 20, szCs 20, kern 0, w14:ligatures none) and marks
//! the style with `w:rPrChange` (old = declared + lang) and `w:pPrChange`
//! (old = declared pPr). Without the bake the copied styles render with A's
//! docDefaults — the R2 cluster's cumulative vertical drift.

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

#[test]
fn s2_bonly_styles_bake_docdefaults_delta_with_change_records() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__image_inline_and_block_ad6109b3.docx");
    let b = src.join("behavior__rtl_page_numpages_54739e26.docx");
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

    for sid in ["Footer", "FootnoteText", "Strong1"] {
        let s = style_elem(&xml, sid).unwrap_or_else(|| panic!("{sid} copied into output"));
        assert!(
            s.contains("rPrChange"),
            "{sid} carries style-level rPrChange: {s}"
        );
        assert!(
            s.contains("pPrChange"),
            "{sid} carries style-level pPrChange: {s}"
        );
        assert!(
            s.contains("Times New Roman"),
            "{sid} bakes B rFonts live: {s}"
        );
        assert!(
            s.contains("w:sz w:val=\"20\"") || s.contains("w:sz w:val='20'"),
            "{sid} bakes implicit sz 20: {s}"
        );
        assert!(s.contains("w:kern w:val=\"0\""), "{sid} bakes kern 0: {s}");
        assert!(
            s.contains("w14:ligatures w14:val=\"none\""),
            "{sid} bakes ligatures none: {s}"
        );
    }
    // Strong1's declared bold must stay declared (live) and echoed in old.
    let strong = style_elem(&xml, "Strong1").unwrap();
    assert!(strong.contains("<w:b"), "Strong1 keeps declared bold");
}
