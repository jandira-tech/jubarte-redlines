// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M74 — inserted VML `w:pict` must survive atomize/coalesce as an opaque
//! leaf (like `w:drawing` / `mc:AlternateContent`). Recursing into
//! shapetype/shape/`v:imagedata` produced zero atoms for attribute-only
//! leaves, so the redline dropped the image and never carried media
//! (file_11×file_12 Word oracle keeps pict under `w:ins` + image rel).

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

fn zip_has_media(docx: &[u8]) -> bool {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    for i in 0..zip.len() {
        let name = zip.by_index(i).unwrap().name().to_string();
        if name.contains("/media/") {
            return true;
        }
    }
    false
}

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn m74_file_11_file_12_keeps_inserted_vml_pict_and_media() {
    let Some((a, b)) = corpus_pair("file_11.docx", "file_12.docx") else {
        eprintln!("SKIP: broken_ones_two sources missing");
        return;
    };
    // B alone carries the VML pict + png.
    assert!(
        zip_has_media(&b),
        "fixture file_12 must include media for this gate"
    );

    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare ok");
    let doc = document_xml(&out);
    assert!(
        doc.contains("w:pict") || doc.contains("<w:pict"),
        "inserted VML pict must remain in redline body: {}",
        &doc[..doc.len().min(400)]
    );
    assert!(
        doc.contains("imagedata") || doc.contains("v:imagedata"),
        "v:imagedata must survive inside pict"
    );
    assert!(
        zip_has_media(&out),
        "image part must be carried into the redline package"
    );
    // Word wraps the pict run under w:ins for this pure-insert image para.
    assert!(
        doc.contains("<w:ins") || doc.contains("w:ins "),
        "pict insert should be revision-marked"
    );
}
