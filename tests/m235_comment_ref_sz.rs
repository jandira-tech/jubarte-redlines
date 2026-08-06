// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word materializes sz/szCs on comment-reference runs. Missing sz costs LO on
//! comment-heavy pairs (nested_comments, verdana×suggesting).

use jubarte::document_comparer::compare_documents;
use std::io::{Cursor, Read};
use std::path::Path;

fn load(name: &str) -> Option<Vec<u8>> {
    let roots = [
        Path::new(
            "/Users/arthrod/temp/T/neurotic_docx_bench/corpus/word_redlines_superdoc/docx_source",
        ),
        Path::new("/Users/arthrod/temp/T/neurotic_docx_bench/corpus/word_based/docx_source"),
    ];
    for root in roots {
        let p = root.join(name);
        if p.is_file() {
            return std::fs::read(p).ok();
        }
    }
    None
}

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn nested_comments_reference_runs_have_sz() {
    let Some(a) = load("super_editor__nested_comments_gdocs_0c8668e1.docx") else {
        eprintln!("skip");
        return;
    };
    let Some(b) = load("super_editor__nested_comments_84f214bb.docx") else {
        eprintln!("skip");
        return;
    };
    let out = compare_documents(&a, &b, "Redline").expect("compare");
    let doc = document_xml(&out);
    assert!(
        doc.contains("commentReference"),
        "expected comment anchors in redline"
    );
    // Word materializes sz=22 on comment-reference rPr (docDefaults half-points).
    let sz_count = doc.matches("<w:sz").count();
    assert!(
        sz_count >= 2,
        "comment-heavy redline must materialize sz on refs; sz_count={sz_count}"
    );
    assert!(
        doc.contains(r#"w:val="22""#) || doc.contains("w:val=\"22\""),
        "expected sz val 22"
    );
}
