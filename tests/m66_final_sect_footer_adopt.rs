// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M66 — B's final-section footer must land on the output final sectPr even
//! when mid-body sections already carry footer/default (file_21: footer20).

use std::io::{Cursor, Read};
use std::path::Path;

use jubarte::document_comparer::compare_documents;

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

fn zip_names(docx: &[u8]) -> Vec<String> {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    (0..zip.len())
        .filter_map(|i| zip.by_index(i).ok().map(|f| f.name().to_string()))
        .collect()
}

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn m66_file_21_adopts_b_final_footer20() {
    let Some((a, b)) = corpus_pair("file_21.docx", "file_22.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let names = zip_names(&out);
    assert!(
        names.iter().any(|n| n.contains("footer20")),
        "file_21 must package B's footer20; got {:?}",
        names
            .iter()
            .filter(|n| n.contains("footer"))
            .collect::<Vec<_>>()
    );
    // Package must also wire footer20 into document.xml.rels (not orphan part).
    let rels = {
        let mut zip = zip::ZipArchive::new(Cursor::new(out.to_vec())).unwrap();
        let mut f = zip.by_name("word/_rels/document.xml.rels").unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        s
    };
    assert!(
        rels.contains("footer20.xml"),
        "document.xml.rels must target footer20.xml"
    );
    let doc = document_xml(&out);
    assert!(
        doc.contains("footerReference") || doc.contains("w:footerReference"),
        "document must retain footerReference elements"
    );
}
