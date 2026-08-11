// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M467 — the merged Normal rPr carries only per-attribute DELTAS vs the
//! output (A) docDefaults. Word drops attrs whose effective value the
//! context already supplies:
//!
//! tab_test × table_autofit: B's Normal stores Arial + kern=0 + sz=20 +
//! szCs=22 + ligatures none + eastAsiaTheme=minorEastAsia; A's dd has no
//! kern (kerning off), szCs=22, eastAsiaTheme=minorEastAsia. Word's output
//! Normal is `rFonts ascii/hAnsi Arial` + `sz=20` ONLY — kern/szCs/
//! eastAsiaTheme/ligatures dropped as redundant. Carrying kern=0 flipped
//! kerning relative to the oracle... wait, both are off — but carrying
//! szCs/kern noise perturbed LO metrics enough to cost 64.2 → 51.4 with
//! the S2-era styles. (basic_comment keeps kern=0 because A-dd kern=2 —
//! covered by m461.)

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
fn merged_normal_rpr_is_delta_vs_a_docdefaults() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source");
    let a = src.join("super_editor__tab_test_576c8317.docx");
    let b = src.join("super_editor__table_autofit_colspan_1fd7723c.docx");
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

    let normal = live(&style_elem(&xml, "Normal").expect("Normal"));
    assert!(
        normal.contains("w:ascii=\"Arial\"") && normal.contains("<w:sz w:val=\"20\""),
        "Normal keeps the real deltas (Arial, sz=20), got: {normal}"
    );
    for noise in ["<w:kern", "<w:szCs", "w:eastAsiaTheme=", "w14:ligatures"] {
        assert!(
            !normal.contains(noise),
            "Normal must drop {noise} (equal to A docDefaults), got: {normal}"
        );
    }
}
