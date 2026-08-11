// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M464 — the S2 copied-style bake must cover pPr spacing, not only run
//! metrics. When a B-only paragraph style resolves a spacing attr through
//! B's docDefaults and the output keeps A's docDefaults with a different
//! value, Word materializes B's effective value on the style.
//!
//! Oracle: file_13 × file_14 (randomized pool). A's pPrDefault is
//! after=200 line=276; B's is empty (effective after=0 line=240). Word's
//! output bakes `spacing after=0 line=240 lineRule=auto` into every copied
//! style (Heading1-6, Title, ListParagraph, Strong1, FootnoteText). The S2
//! half-bake (rPr only, empty pPr) landed in 5360dd4 and REGRESSED this
//! pair 75.4 → 53.3 — fonts moved to B metrics while line spacing stayed
//! A's, shifting every page.

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

fn live(style: &str) -> String {
    let mut s = style.to_string();
    for tag in ["pPrChange", "rPrChange"] {
        while let Some(i) = s.find(&format!("<w:{tag}")) {
            let close = format!("</w:{tag}>");
            let Some(j) = s[i..].find(&close) else { break };
            s.replace_range(i..i + j + close.len(), "");
        }
    }
    s
}

#[test]
fn s2_bakes_b_effective_ppr_spacing() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_based/docx_source_randomized");
    let a = src.join("file_13.docx");
    let b = src.join("file_14.docx");
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

    for sid in ["Heading1", "Heading2", "Title", "ListParagraph", "Strong1"] {
        let Some(style) = style_elem(&xml, sid) else {
            continue;
        };
        let lv = live(&style);
        assert!(
            lv.contains("w:after=\"0\"") && lv.contains("w:line=\"240\""),
            "{sid} live pPr must bake B-effective spacing after=0 line=240, got: {lv}"
        );
    }
}
