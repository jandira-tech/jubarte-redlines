// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M480a — the both-sides declared-blocks merge writes B's block but never
//! neutralized the docDefaults delta: with A-dd kern=2 + ligatures and
//! B-dd kern-none (implicit 0), a merged Heading1 renders with A's kerning
//! and line metrics. Word's oracle (evals__memorandum × evals__nda,
//! bench9 45.87 — the "51 styles differ" census cluster) writes the
//! neutralizers live: kern 0, w14:ligatures none, spacing line=276
//! (B-dd's line riding into the declared spacing).

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

fn live_of(style: &str, block: &str, change: &str) -> String {
    let Some(i) = style.find(&format!("<w:{block}>")) else {
        return String::new();
    };
    let seg = &style[i..style[i..]
        .find(&format!("</w:{block}>"))
        .map_or(style.len(), |e| i + e)];
    match seg.find(&format!("<w:{change}")) {
        Some(c) => seg[..c].to_string(),
        None => seg.to_string(),
    }
}

#[test]
#[ignore = "M480a reverted: the single-cluster neutralizer rule broke tab_test (12->28 effective diffs) and paragraph_spacing (12->19) — same trap as M476. Word neutralizes both-sides merged styles in the memorandum x nda cluster but NOT in tab_test/paragraph_spacing. Derive the discriminator from BOTH classes (multi-oracle evidence matrix) before re-attempting; see bench-90 campaign memory."]
fn merged_heading_gets_dd_neutralizers() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("evals__memorandum_258c774a.docx");
    let b = src.join("evals__nda_7f304918.docx");
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

    let h1 = style_elem(&xml, "Heading1").expect("Heading1 present");
    let rpr = live_of(&h1, "rPr", "rPrChange");
    assert!(
        rpr.contains("w:kern w:val=\"0\""),
        "Heading1 live rPr must neutralize A-dd kern=2 with kern 0: {rpr}"
    );
    assert!(
        rpr.contains("w14:ligatures w14:val=\"none\""),
        "Heading1 live rPr must neutralize A-dd ligatures: {rpr}"
    );
    let ppr = live_of(&h1, "pPr", "pPrChange");
    assert!(
        ppr.contains("w:line=\"276\""),
        "Heading1 live spacing must carry B-dd line=276: {ppr}"
    );
}
