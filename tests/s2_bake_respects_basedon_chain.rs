// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! S2 chain-awareness (file_198 × file_199, 97.08 → 91.51 bisected to
//! 5360dd4): B is a LibreOffice document whose styles inherit their look
//! from a promoted Normal (Liberation Serif, sz 24). After the copied
//! styles' props are deduped into Normal, the docDefaults bake re-filled
//! attrs "missing" from each style's OWN rPr with B-docDefaults values —
//! stamping THEME fonts and metrics onto Textbody/List/Caption and
//! overriding the Liberation Serif the basedOn chain provides. Word's
//! oracle keeps the chain's fonts. The bake must consult the output
//! basedOn chain (as the M464 spacing bake already does) before adding an
//! attr.

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

fn live_rpr(style: &str) -> String {
    let Some(i) = style.find("<w:rPr>") else {
        return String::new();
    };
    let seg = &style[i..style[i..].find("</w:rPr>").map_or(style.len(), |e| i + e)];
    match seg.find("<w:rPrChange") {
        Some(c) => seg[..c].to_string(),
        None => seg.to_string(),
    }
}

#[test]
fn bake_never_overrides_attrs_the_chain_provides() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("../neurotic_docx_bench/corpus/word_based/docx_source_randomized");
    let a = src.join("file_198.docx");
    let b = src.join("file_199.docx");
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

    // Normal (promoted from B) declares the concrete fonts.
    let normal = style_elem(&xml, "Normal").expect("Normal present");
    assert!(
        normal.contains("Liberation Serif"),
        "promoted Normal keeps B's concrete fonts"
    );

    // Styles inheriting from Normal must NOT get theme fonts baked over the
    // chain (Word renders them in Liberation Serif via basedOn). Metric
    // bakes (sz/kern from B's docDefaults) are correct — the oracle's
    // Textbody carries kern 2 / sz 24 — only the FONT override is the bug.
    for sid in ["Textbody", "List", "Caption", "Index"] {
        let Some(st) = style_elem(&xml, sid) else {
            continue;
        };
        let live = live_rpr(&st);
        assert!(
            !live.contains("asciiTheme"),
            "{sid}: baked theme fonts override the chain's Liberation Serif: {live}"
        );
    }

    // Promotion completeness: Word's promoted Normal carries B's FULL
    // effective block — widowControl and suppressAutoHyphens change line
    // breaking and pagination (oracle Normal: widowControl 0, tabs 709,
    // suppressAutoHyphens, color 00000A, kern 2, lang zh-CN/hi-IN,
    // ligatures).
    assert!(
        normal.contains("widowControl"),
        "promoted Normal must carry B's widowControl (pagination): {normal}"
    );
    assert!(
        normal.contains("suppressAutoHyphens"),
        "promoted Normal must carry B's suppressAutoHyphens: {normal}"
    );
    assert!(
        normal.contains("<w:kern "),
        "promoted Normal must carry B-dd kern: {normal}"
    );
}
