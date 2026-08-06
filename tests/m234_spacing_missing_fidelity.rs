// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word keeps line=276 on pure-del/mixed residuals for
//! paragraph_spacing_missing × exported_list_font (median30 ~78).

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
fn spacing_missing_x_list_font_keeps_line276_on_del_residuals() {
    let Some(a) = load("super_editor__paragraph_spacing_missing_de418c38.docx") else {
        eprintln!("skip");
        return;
    };
    let Some(b) = load("super_editor__exported_list_font_8e6db734.docx") else {
        eprintln!("skip");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    assert!(
        doc.contains(r#"w:line="276""#) || doc.contains("w:line=\"276\""),
        "Word keeps line=276 on del residuals; doc missing it"
    );
}
