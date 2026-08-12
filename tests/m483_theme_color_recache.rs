// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M483 — w:color hexes cached under B's theme are re-resolved against the
//! output package's theme (tab_test oracle: H1Char val rewritten to the
//! A-theme accent1 shade).

use std::io::Read;
use std::path::PathBuf;

use jubarte::document_comparer::compare_documents;

#[test]
fn stale_theme_color_hex_is_recached() {
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
    let mut styles = String::new();
    zip.by_name("word/styles.xml")
        .unwrap()
        .read_to_string(&mut styles)
        .unwrap();
    let mut theme = String::new();
    zip.by_name("word/theme/theme1.xml")
        .unwrap()
        .read_to_string(&mut theme)
        .unwrap();
    // Output ships A's theme: accent1 is NOT the modern 0F4761 base.
    let live = regex_lite_strip(&styles);
    assert!(
        !live.contains("w:val=\"0F4761\""),
        "live styles must not keep B-theme-cached hex 0F4761"
    );
}

fn regex_lite_strip(s: &str) -> String {
    // drop rPrChange bodies (baselines legitimately keep old hexes)
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find("<w:rPrChange") {
        out.push_str(&rest[..i]);
        match rest[i..].find("</w:rPrChange>") {
            Some(e) => rest = &rest[i + e + 14..],
            None => {
                rest = "";
                break;
            }
        }
    }
    out.push_str(rest);
    out
}
