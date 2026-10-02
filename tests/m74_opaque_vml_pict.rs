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

mod common;

/// A paragraph with an embedded `w:object` (Word.Picture.8: a VML
/// `v:shape` over `v:imagedata`) between two text paragraphs.
fn object_docx(with_object: bool) -> Vec<u8> {
    use common::docx::{Part, docx_with, para};
    let object = if with_object {
        r#"<w:p><w:r><w:object w:dxaOrig="3000" w:dyaOrig="600" xmlns:v="urn:schemas-microsoft-com:vml" xmlns:o="urn:schemas-microsoft-com:office:office"><v:shape id="s1" style="width:150pt;height:30pt"><v:imagedata r:id="rIdX0" o:title=""/></v:shape></w:object></w:r></w:p>"#
    } else {
        ""
    };
    let body = format!("{}{object}{}", para("Before"), para("After"));
    docx_with(
        &body,
        &[Part {
            name: "word/media/image1.png",
            content_type: "image/png",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image",
            xml: "png",
        }],
    )
}

/// The `w:object` element of `doc`, start tag to end tag.
fn object_xml(doc: &str) -> &str {
    let start = doc.find("<w:object").expect("w:object kept in the redline");
    let end = doc[start..]
        .find("</w:object>")
        .map(|e| start + e + "</w:object>".len())
        .unwrap_or_else(|| {
            panic!(
                "w:object emptied: {}",
                &doc[start..doc.len().min(start + 200)]
            )
        });
    &doc[start..end]
}

/// The B-side 5a6c "End-Point Assessment" picture: an inserted or deleted
/// `w:object` lost its VML children (an empty `<w:object …/>` shell) and its
/// revision mark, because only `w:pict` was re-emitted whole.
#[test]
fn an_inserted_or_deleted_vml_object_keeps_its_picture_and_mark() {
    let (a, b) = (object_docx(false), object_docx(true));
    for (old, new, mark) in [(&a, &b, "w:ins"), (&b, &a, "w:del")] {
        let out = compare_documents(old, new, "Arthur Souza Rodrigues").expect("compare ok");
        let doc = document_xml(&out);
        let object = object_xml(&doc);
        assert!(
            object.contains("imagedata"),
            "{mark}: v:imagedata must survive inside w:object: {object}"
        );
        let open = doc
            .rfind(&format!("<{mark} "))
            .filter(|&i| i < doc.find("<w:object").unwrap());
        let close = doc[doc.find("</w:object>").unwrap()..].find(&format!("</{mark}>"));
        assert!(
            open.is_some() && close.is_some(),
            "the object's run must sit under {mark}: {doc}"
        );
    }
}
