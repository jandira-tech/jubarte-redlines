// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M480b — the both-sides declared-blocks merge writes B's block plus the
//! docDefaults-delta DISABLING neutralizers: with A-dd kern=2 + ligatures
//! and B-dd kern-none (implicit 0), a merged Heading1 renders with A's
//! kerning unless the style carries kern 0 + w14:ligatures none live.
//! Word's oracle (evals__memorandum × evals__nda, the "51 styles differ"
//! census cluster) writes exactly those stamps on every merged style whose
//! basedOn chain doesn't already provide the attribute; the oracle
//! neutralized 81/81 disabling-direction pairs. (The B-dd spacing line=276
//! value-write is a separate, still-unmined axis — not asserted here.)

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
    // B-dd spacing line=276 riding into declared spacing is a VALUE write on
    // a different axis (skip class unmined) — deliberately not asserted.
}
