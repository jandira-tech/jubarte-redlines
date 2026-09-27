// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M92 — trailing empty body para: live spacing → pPrChange (file_30).

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

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

/// Last body paragraph fragment (before sectPr), roughly.
fn last_body_p(doc: &str) -> &str {
    let body = doc
        .split("<w:body")
        .nth(1)
        .and_then(|s| s.split("</w:body>").next())
        .unwrap_or("");
    // Take last </w:p> chunk that is not inside sectPr-only noise.
    body.rsplit("</w:p>").nth(1).unwrap_or("")
}

#[test]
fn m92_file_30_trailing_empty_spacing_in_pprchange() {
    let Some((a, b)) = corpus_pair("file_30.docx", "file_31.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let last = last_body_p(&doc);
    // Trailing empty: no t/delText in last para; check last p chunk.
    let chunks: Vec<&str> = doc.split("</w:p>").collect();
    // Walk from end for a chunk with no delText and no w:t content text.
    let mut found_empty = false;
    for chunk in chunks.iter().rev().take(4) {
        let has_text = chunk.contains("<w:t") || chunk.contains("delText");
        if has_text {
            continue;
        }
        if !chunk.contains("<w:p") && !chunk.contains("pPr") {
            continue;
        }
        found_empty = true;
        let live = if let Some(i) = chunk.find("pPrChange") {
            &chunk[..i]
        } else {
            *chunk
        };
        assert!(
            !live.contains("<w:spacing") && !live.contains("w:line="),
            "trailing empty must not keep live spacing: {chunk}"
        );
        assert!(
            chunk.contains("pPrChange") && chunk.contains("spacing"),
            "trailing empty spacing must sit under pPrChange: {chunk}"
        );
        break;
    }
    assert!(found_empty, "expected trailing empty paragraph: {last}");
}

#[test]
fn m92_file_23_last_del_still_spacing_pprchange() {
    // Guard M83b under M92.
    let Some((a, b)) = corpus_pair("file_23.docx", "file_24.docx") else {
        eprintln!("skip: corpus missing");
        return;
    };
    let out = compare_documents(&a, &b, "Arthur Souza Rodrigues").expect("compare");
    let doc = document_xml(&out);
    let mut found = false;
    for chunk in doc.split("</w:p>") {
        if !chunk.contains("Document Title") {
            continue;
        }
        found = true;
        let live = if let Some(i) = chunk.find("pPrChange") {
            &chunk[..i]
        } else {
            chunk
        };
        assert!(!live.contains("w:line=\"240\""));
        assert!(chunk.contains("pPrChange") && chunk.contains("w:line=\"240\""));
    }
    assert!(found);
}

fn docx(body: &str) -> Vec<u8> {
    use std::io::Write;
    let doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body}<w:sectPr/></w:body></w:document>"#
    );
    let ct = br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;
    let rels = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    let mut buf = Cursor::new(Vec::new());
    {
        let mut z = zip::ZipWriter::new(&mut buf);
        let opt = zip::write::SimpleFileOptions::default();
        for (name, body) in [
            ("[Content_Types].xml", &ct[..]),
            ("_rels/.rels", &rels[..]),
            ("word/document.xml", doc.as_bytes()),
        ] {
            z.start_file(name, opt).unwrap();
            z.write_all(body).unwrap();
        }
        z.finish().unwrap();
    }
    buf.into_inner()
}

/// Both documents end in the same spaced empty paragraph, so that spacing is
/// the revised document's own and Word keeps it live, unrevised
/// (super_editor complex2×complexexport1). M92 moved it into a pPrChange,
/// which also painted a change bar Word does not show.
#[test]
fn equal_trailing_empty_keeps_live_spacing() {
    let tail = r#"<w:p><w:pPr><w:spacing w:before="0" w:after="0" w:line="240" w:lineRule="auto"/></w:pPr></w:p>"#;
    let a = docx(&format!(
        "<w:p><w:r><w:t>Old words here.</w:t></w:r></w:p>{tail}"
    ));
    let b = docx(&format!(
        "<w:p><w:r><w:t>New words here.</w:t></w:r></w:p>{tail}"
    ));
    let settings = jubarte::comparer::WmlComparerSettings {
        merge_replaced_paragraphs: true,
        ..Default::default()
    };
    let out =
        jubarte::document_comparer::compare_documents_with_settings(&a, &b, &settings).unwrap();
    let doc = document_xml(&out);
    let last = doc.rsplit("<w:p>").next().unwrap();
    assert!(
        !last.contains("pPrChange") && last.contains("w:line=\"240\""),
        "trailing empty must keep its own spacing live: {last}"
    );
}
