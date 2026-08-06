// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! auto_page_break_2 × auto_page_break: Word pure-I "This icon appears…" then
//! pure-D "TITLE IN HERE" (MIIIID…). Folding the long icon body into the short
//! title produces MIMDD… and mixes unrelated residual.

use jubarte::document_comparer::compare_documents;
use std::io::{Cursor, Read};
use std::path::Path;

fn load(name: &str) -> Option<Vec<u8>> {
    let p = Path::new(
        "/Users/arthrod/temp/T/neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source",
    )
    .join(name);
    std::fs::read(p).ok()
}

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn auto_page_break_icon_body_not_mixed_into_title_del() {
    let Some(a) = load("super_editor__sd_1495_auto_page_break_2_f0f3bf0e.docx") else {
        eprintln!("skip");
        return;
    };
    let Some(b) = load("super_editor__sd_1495_auto_page_break_854a2dd9.docx") else {
        eprintln!("skip");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);

    let icon = doc
        .split("</w:p>")
        .find(|p| p.contains("This icon appears") || p.contains("icon appears throughout"))
        .expect("icon pure-I para");
    assert!(
        !icon.contains("TITLE IN HERE") && !icon.contains("<w:delText"),
        "icon body must not mesh with del TITLE; para={icon}"
    );
    assert!(
        icon.contains("<w:ins") || icon.contains("w:ins "),
        "icon body must be pure insert"
    );
    // Title remains as pure-del residual somewhere.
    assert!(
        doc.contains("TITLE IN HERE") && doc.contains("<w:del"),
        "TITLE IN HERE must remain as pure-del"
    );
}
