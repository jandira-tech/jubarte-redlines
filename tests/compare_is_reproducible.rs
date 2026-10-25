// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Comparing the same pair twice in one process must give identical bytes.
//!
//! A server, a WASM instance or a parallel test run compares many pairs in one
//! process. Names that depend on how many compares ran before (for example a
//! process-wide counter used in the ZIP entry name of a copied image) make the
//! output differ between two runs of the same input.

use std::io::{Cursor, Read, Write};

use jubarte::document_comparer::compare_documents_with_options;

const REL_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const FIXED_DATE: &str = "2026-01-01T00:00:00Z";

/// Two different byte strings standing in for two different PNG files. The
/// comparer copies media bytes verbatim and never decodes them.
const IMAGE_ONE: &[u8] = b"\x89PNG\r\n\x1a\nfake image one";
const IMAGE_TWO: &[u8] = b"\x89PNG\r\n\x1a\nfake image two, different bytes";

/// A minimal valid package. `images` are `(part name, bytes)` pairs and each
/// gets a relationship `rIdImgN` in order.
fn build_docx(body: &str, images: &[(&str, &[u8])]) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
        let opt = zip::write::SimpleFileOptions::default();
        z.start_file("[Content_Types].xml", opt).unwrap();
        z.write_all(br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Default Extension="png" ContentType="image/png"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#).unwrap();
        z.start_file("_rels/.rels", opt).unwrap();
        z.write_all(br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rIdM" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#).unwrap();
        z.start_file("word/document.xml", opt).unwrap();
        z.write_all(
            format!(
                "<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" \
                 xmlns:r=\"{REL_NS}\" xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" \
                 xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\" \
                 xmlns:pic=\"http://schemas.openxmlformats.org/drawingml/2006/picture\">\
                 <w:body>{body}<w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr></w:body></w:document>"
            )
            .as_bytes(),
        )
        .unwrap();
        z.start_file("word/_rels/document.xml.rels", opt).unwrap();
        let mut rels = String::from(
            r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
        );
        for (i, (name, _)) in images.iter().enumerate() {
            let target = name.trim_start_matches("word/");
            rels.push_str(&format!(
                r#"<Relationship Id="rIdImg{i}" Type="{REL_NS}/image" Target="{target}"/>"#
            ));
        }
        rels.push_str("</Relationships>");
        z.write_all(rels.as_bytes()).unwrap();
        for (name, bytes) in images {
            z.start_file(*name, opt).unwrap();
            z.write_all(bytes).unwrap();
        }
        z.finish().unwrap();
    }
    buf
}

/// An inline picture paragraph that embeds relationship `rid`.
fn picture_paragraph(rid: &str, docpr_id: u32) -> String {
    format!(
        "<w:p><w:r><w:drawing><wp:inline distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\">\
         <wp:extent cx=\"914400\" cy=\"914400\"/><wp:docPr id=\"{docpr_id}\" name=\"Picture {docpr_id}\"/>\
         <a:graphic><a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/picture\">\
         <pic:pic><pic:nvPicPr><pic:cNvPr id=\"{docpr_id}\" name=\"p{docpr_id}.png\"/><pic:cNvPicPr/></pic:nvPicPr>\
         <pic:blipFill><a:blip r:embed=\"{rid}\"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill>\
         <pic:spPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"914400\" cy=\"914400\"/></a:xfrm>\
         <a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></pic:spPr></pic:pic>\
         </a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"
    )
}

fn text_paragraph(text: &str) -> String {
    format!("<w:p><w:r><w:t>{text}</w:t></w:r></w:p>")
}

fn entry_names(docx: &[u8]) -> Vec<String> {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    (0..zip.len())
        .map(|i| zip.by_index(i).unwrap().name().to_string())
        .collect()
}

fn read_entry(docx: &[u8], name: &str) -> Vec<u8> {
    let mut zip = zip::ZipArchive::new(Cursor::new(docx.to_vec())).unwrap();
    let mut out = Vec::new();
    zip.by_name(name).unwrap().read_to_end(&mut out).unwrap();
    out
}

fn media_entries(docx: &[u8]) -> Vec<String> {
    entry_names(docx)
        .into_iter()
        .filter(|n| n.starts_with("word/media/"))
        .collect()
}

/// Original: text only. Modified: the same text plus an inserted picture, so
/// the picture's media part is copied into the output package.
fn inserted_image_pair() -> (Vec<u8>, Vec<u8>) {
    let a = build_docx(&text_paragraph("Shared paragraph."), &[]);
    let b = build_docx(
        &format!(
            "{}{}",
            text_paragraph("Shared paragraph."),
            picture_paragraph("rIdImg0", 1)
        ),
        &[("word/media/image1.png", IMAGE_ONE)],
    );
    (a, b)
}

#[test]
fn inserted_image_pair_copies_a_media_part() {
    let (a, b) = inserted_image_pair();
    let out = compare_documents_with_options(&a, &b, "Test", FIXED_DATE).expect("compare ok");
    let media = media_entries(&out);
    assert_eq!(
        media.len(),
        1,
        "the inserted picture's media must be copied into the output: {:?}",
        entry_names(&out)
    );
    assert_eq!(read_entry(&out, &media[0]), IMAGE_ONE);
}

#[test]
fn comparing_the_same_pair_twice_gives_identical_bytes() {
    let (a, b) = inserted_image_pair();
    let first = compare_documents_with_options(&a, &b, "Test", FIXED_DATE).expect("compare ok");
    let second = compare_documents_with_options(&a, &b, "Test", FIXED_DATE).expect("compare ok");
    assert_eq!(
        media_entries(&first),
        media_entries(&second),
        "copied media part names must not depend on earlier compares"
    );
    assert!(
        first == second,
        "two compares of the same pair in one process must be byte-identical"
    );
}

#[test]
fn compare_result_does_not_depend_on_earlier_unrelated_compares() {
    let (a, b) = inserted_image_pair();
    let alone = compare_documents_with_options(&a, &b, "Test", FIXED_DATE).expect("compare ok");
    // Unrelated compares advance any process-wide state.
    let x = build_docx(&text_paragraph("Something else."), &[]);
    let y = build_docx(&text_paragraph("Something else, changed."), &[]);
    for _ in 0..3 {
        compare_documents_with_options(&x, &y, "Test", FIXED_DATE).expect("compare ok");
    }
    let after = compare_documents_with_options(&a, &b, "Test", FIXED_DATE).expect("compare ok");
    assert!(
        alone == after,
        "output must not depend on how many compares ran before it"
    );
}

/// Two different images that share an extension must keep distinct part names.
#[test]
fn different_images_never_share_a_part_name() {
    let a = build_docx(&text_paragraph("Shared paragraph."), &[]);
    let b = build_docx(
        &format!(
            "{}{}{}",
            text_paragraph("Shared paragraph."),
            picture_paragraph("rIdImg0", 1),
            picture_paragraph("rIdImg1", 2)
        ),
        &[
            ("word/media/image1.png", IMAGE_ONE),
            ("word/media/image2.png", IMAGE_TWO),
        ],
    );
    let out = compare_documents_with_options(&a, &b, "Test", FIXED_DATE).expect("compare ok");
    let media = media_entries(&out);
    assert_eq!(media.len(), 2, "both images are copied: {media:?}");
    let mut bodies: Vec<Vec<u8>> = media.iter().map(|n| read_entry(&out, n)).collect();
    bodies.sort();
    let mut want = vec![IMAGE_ONE.to_vec(), IMAGE_TWO.to_vec()];
    want.sort();
    assert_eq!(bodies, want);
}
