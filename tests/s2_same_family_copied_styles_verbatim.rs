// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! S2 gate repair (the 5360dd4 regression class, five pairs incl. file_52 ×
//! file_53 and file_198 × file_199 — both bisect-proven): the copied-style
//! docDefaults bake must fire ONLY when the two documents' docDefaults
//! change the ASCII FONT FAMILY itself (the motivating oracle
//! image_inline_and_block × rtl_page_numpages: theme font vs concrete Times
//! New Roman). When both sides resolve the same ascii family
//! (tab_test × table_autofit_colspan: both minorHAnsi) and differ only in
//! metrics (sz 22 vs 24, kern, ligatures, szCs, eastAsia theme, spacing),
//! Word copies B-only styles VERBATIM — TableBody's live rPr is exactly its
//! declared `sz 18`, TableListBullet inherits size through basedOn and gains
//! NO live sz, TableNote stays just italic. Our bake stamped kern 2 /
//! sz 24 / szCs 24 / ligatures / theme rFonts / spacing line=240 onto them
//! (12pt table bullets vs Word's 9pt).

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

/// The style's LIVE rPr with any rPrChange record stripped.
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
fn same_ascii_family_copies_b_only_styles_verbatim() {
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

    // TableBody: live rPr must be its B declaration — sz 18 and NOTHING baked.
    let tb = style_elem(&xml, "TableBody").expect("TableBody copied");
    let tb_live = live_rpr(&tb);
    assert!(
        tb_live.contains("w:sz w:val=\"18\""),
        "TableBody keeps its declared sz 18: {tb_live}"
    );
    for banned in ["w:kern", "w14:ligatures", "asciiTheme", "w:szCs"] {
        assert!(
            !tb_live.contains(banned),
            "TableBody live rPr must not bake {banned}: {tb_live}"
        );
    }

    // TableListBullet: inherits size via basedOn TableBody — NO live sz.
    let tlb = style_elem(&xml, "TableListBullet").expect("TableListBullet copied");
    let tlb_live = live_rpr(&tlb);
    assert!(
        !tlb_live.contains("<w:sz ") && !tlb_live.contains("w:kern"),
        "TableListBullet must not gain baked sz/kern (renders 12pt vs Word's 9pt): {tlb_live}"
    );

    // TableNote: declared italic only.
    let tn = style_elem(&xml, "TableNote").expect("TableNote copied");
    let tn_live = live_rpr(&tn);
    assert!(
        tn_live.contains("<w:i") && !tn_live.contains("<w:sz ") && !tn_live.contains("w:kern"),
        "TableNote stays just italic: {tn_live}"
    );
}
