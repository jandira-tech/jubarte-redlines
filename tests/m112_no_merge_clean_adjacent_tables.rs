// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M112 — clean (no-revision) adjacent tables stay separate on accept/merge
//! (file_130/131 metadata tables: Word keeps 1-col + 2-col, not one 2-col).

use std::io::{Cursor, Read};
use std::path::Path;

use jubarte::document_comparer::compare_documents;
use jubarte::namespaces::W;
use jubarte::revision_processor::merge_adjacent_tables_transform;
use jubarte::xmllinq::{Dom, NodeId};

fn body_from(dom: &mut Dom, inner: &str) -> NodeId {
    let xml = format!(
        "<w:document xmlns:w=\"{}\"><w:body>{}</w:body></w:document>",
        W::URI,
        inner
    );
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    dom.element(root, &W::body()).unwrap()
}

#[test]
fn m112_clean_adjacent_tables_not_merged() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:tbl>\
         <w:tblPr/>\
         <w:tblGrid><w:gridCol w:w=\"10080\"/></w:tblGrid>\
         <w:tr>\
         <w:tc><w:tcPr><w:tcW w:w=\"10080\" w:type=\"dxa\"/></w:tcPr><w:p><w:r><w:t>thesis</w:t></w:r></w:p></w:tc>\
         </w:tr>\
         </w:tbl>\
         <w:tbl>\
         <w:tblPr/>\
         <w:tblGrid><w:gridCol w:w=\"5040\"/><w:gridCol w:w=\"5040\"/></w:tblGrid>\
         <w:tr>\
         <w:tc><w:tcPr><w:tcW w:w=\"5040\" w:type=\"dxa\"/></w:tcPr><w:p><w:r><w:t>left</w:t></w:r></w:p></w:tc>\
         <w:tc><w:tcPr><w:tcW w:w=\"5040\" w:type=\"dxa\"/></w:tcPr><w:p><w:r><w:t>right</w:t></w:r></w:p></w:tc>\
         </w:tr>\
         </w:tbl>",
    );
    let out = merge_adjacent_tables_transform(&mut d, body);
    let tbls = d.elements(out, Some(&W::name("tbl")));
    assert_eq!(
        tbls.len(),
        2,
        "clean adjacent tables must stay separate (Word Compare / file_131)"
    );
}

fn corpus_pair(a: &str, b: &str) -> Option<(Vec<u8>, Vec<u8>)> {
    let root = Path::new("tests/corpus/broken_ones_two/sources");
    let ap = root.join(a);
    let bp = root.join(b);
    if ap.is_file() && bp.is_file() {
        Some((std::fs::read(ap).ok()?, std::fs::read(bp).ok()?))
    } else {
        None
    }
}

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn m112_file_130_keeps_two_metadata_tables() {
    let Some((a, b)) = corpus_pair("file_130.docx", "file_131.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let tbls: Vec<_> = doc
        .split("<w:tbl")
        .skip(1)
        .map(|s| s.split("</w:tbl>").next().unwrap_or(""))
        .collect();
    let n_meta = tbls
        .iter()
        .filter(|t| t.contains("Positioning") || t.contains("Prepared for"))
        .count();
    assert!(
        n_meta >= 2,
        "metadata must stay as ≥2 tables (not merged), found {n_meta}"
    );
}
