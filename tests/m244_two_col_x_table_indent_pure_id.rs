// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! two_col_tab_positions × table_left_indent: Word pure-I all of B (table doc)
//! then pure-D all of A (short page labels). Free LCS meshes "Page1" into the
//! first table title (MIX + drops a pure-D Page1).

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
fn two_col_x_table_indent_lead_title_is_pure_ins() {
    let Some(a) = load("super_editor__sd_1480_two_col_tab_positions_00953280.docx") else {
        eprintln!("skip");
        return;
    };
    let Some(b) = load("super_editor__sd_1494_table_left_indent_03277d35.docx") else {
        eprintln!("skip");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);

    // Lead content para must be pure-I "Table with left indent" — no delText /
    // del body mesh of "Page"/"1" into the title (Word I[I]III[I]DDDDD).
    assert!(
        doc.contains("Table with left indent") || doc.contains("left indent"),
        "expected B table title in redline"
    );
    // First paragraph block: pure ins of table title, not MIX with Page1.
    let first_p = doc
        .split("</w:p>")
        .find(|p| p.contains("Table with left indent") || p.contains("left indent"))
        .expect("title para");
    assert!(
        !first_p.contains("<w:del ") && !first_p.contains("<w:del>"),
        "title must not carry del mesh of Page labels; para={first_p}"
    );
    assert!(
        first_p.contains("<w:ins") || first_p.contains("w:ins "),
        "title must be pure insert; para={first_p}"
    );
    // Page labels must appear as pure deletions (not only fused into title).
    assert!(
        doc.contains("Page") && doc.contains("<w:del"),
        "A page labels must remain as pure-del content"
    );
}
