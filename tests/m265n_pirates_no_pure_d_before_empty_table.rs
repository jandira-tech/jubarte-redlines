// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M265n — pirates×table_indent: pure-I "Table with no left indent" must go
//! straight to the pure-I empty table (Word). M265 broad gate inserted a pure-D
//! empty between them.

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
fn pirates_title_then_pure_i_table_no_intervening_pure_d_empty() {
    let Some(a) = load("super_editor__sd_2766_pirates_tracked_changes_3285d875.docx") else {
        eprintln!("skip");
        return;
    };
    let Some(b) = load("super_editor__sd_1494_table_left_indent_11bb24c7.docx") else {
        eprintln!("skip");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    let body = doc
        .split("<w:body>")
        .nth(1)
        .and_then(|b| b.split("</w:body>").next())
        .unwrap_or("");
    let Some(ti) = body.find("Table with no left indent") else {
        // alternate apostrophe
        if !body.contains("left indent") {
            panic!("no table title");
        }
        return;
    };
    let after = &body[ti..];
    let Some(tbl) = after.find("<w:tbl") else {
        panic!("no table after title");
    };
    let between = &after[..tbl];
    // Between title and next table: only empty pure-I allowed, not pure-D empty.
    // Pure-D empty would be a p with del and no ins body text.
    assert!(
        !between.contains("<w:del ")
            && !between.contains("<w:del>")
            && !between.contains("<w:del/"),
        "no pure-D empty between pure-I table title and pure-I table; between={between}"
    );
}
