// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! comment × complex_list_def_issue: Word pure-I all list items including
//! "FOUR", then MIX/del of A's "My comment". Free residual mesh puts
//! "FOUR"+"My comment" in one MIX para.

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
fn comment_x_complex_list_four_is_pure_ins_not_meshed_with_comment() {
    let Some(a) = load("super_editor__comment_23ee5ec1.docx") else {
        eprintln!("skip");
        return;
    };
    let Some(b) = load("super_editor__complex_list_def_issue_326369f9.docx") else {
        eprintln!("skip");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);

    // Find the para that contains FOUR as insert text.
    let four_p = doc
        .split("</w:p>")
        .find(|p| p.contains(">FOUR<") || p.contains(">FOUR</"))
        .expect("FOUR para");
    assert!(
        !four_p.contains("My comment") && !four_p.contains("delText"),
        "FOUR must not mesh with del 'My comment'; para={four_p}"
    );
    assert!(
        four_p.contains("<w:ins") || four_p.contains("w:ins "),
        "FOUR must be pure insert"
    );
    // A comment text still appears as a deletion somewhere.
    assert!(
        doc.contains("My comment") && doc.contains("<w:del"),
        "My comment must remain as pure-del / MIX residual"
    );
}
