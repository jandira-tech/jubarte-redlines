// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M462 — when A has no styles part, Word does NOT adopt B's docDefaults
//! wholesale. It synthesizes its FACTORY docDefaults (kern=2 sz=24 szCs=24
//! ligatures standardContextual; pPrDefault after=160 line=278 lineRule=auto)
//! and bakes each B style's effective metrics into the style so it still
//! renders as it did in B: BodyText gains line=276 (B's pPrDefault) and
//! kern=0 sz=22 szCs=22 (B's rPrDefault vs factory); ListParagraph gains
//! after=200 line=276; Normal stays bare (A-context wins for Normal).
//!
//! Oracle: behavior__tiff_image_2d531f83 (no styles.xml) ×
//! behavior__two_column_simple_e77be963. Adopting B's docDefaults renders
//! A's bare paragraphs at sz=22/after=200 instead of the oracle's
//! sz=24/after=160 — 17 pages vs the oracle's 20 and a 39.6 score.

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
fn factory_docdefaults_and_style_bake_when_a_has_no_styles() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("behavior__tiff_image_2d531f83.docx");
    let b = src.join("behavior__two_column_simple_e77be963.docx");
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
    drop(f);

    // Factory docDefaults, not B's.
    let dd_start = xml.find("<w:docDefaults>").expect("docDefaults");
    let dd_end = xml.find("</w:docDefaults>").expect("docDefaults close");
    let dd = &xml[dd_start..dd_end];
    assert!(dd.contains("w:kern w:val=\"2\""), "factory kern=2, got: {dd}");
    assert!(dd.contains("w:sz w:val=\"24\""), "factory sz=24, got: {dd}");
    assert!(
        dd.contains("w:after=\"160\"") && dd.contains("w:line=\"278\""),
        "factory pPrDefault after=160 line=278, got: {dd}"
    );

    // Normal stays bare (no baked pPr/rPr).
    let normal = live(&style_elem(&xml, "Normal").expect("Normal"));
    assert!(
        !normal.contains("<w:spacing") && !normal.contains("<w:sz"),
        "Normal must stay bare, got: {normal}"
    );

    // BodyText: declared after=120 + baked line=276 + kern=0 sz=22 szCs=22.
    let bt = live(&style_elem(&xml, "BodyText").expect("BodyText"));
    assert!(
        bt.contains("w:after=\"120\"") && bt.contains("w:line=\"276\""),
        "BodyText spacing after=120 line=276, got: {bt}"
    );
    assert!(
        bt.contains("<w:kern w:val=\"0\"") && bt.contains("<w:sz w:val=\"22\""),
        "BodyText rPr kern=0 sz=22, got: {bt}"
    );

    // ListParagraph: baked after=200 line=276 (all from B's pPrDefault).
    let lp = live(&style_elem(&xml, "ListParagraph").expect("ListParagraph"));
    assert!(
        lp.contains("w:after=\"200\"") && lp.contains("w:line=\"276\""),
        "ListParagraph spacing after=200 line=276, got: {lp}"
    );
}
