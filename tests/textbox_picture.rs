// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A picture inside a text box survives docx to markdown to docx.

mod common;

use jubarte::markdown::{DocxOptions, ReadOptions, markdown_to_docx, read};
use std::io::Read as _;

const PNG_1X1: &[u8] = &[
    0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0x0D, b'I', b'H', b'D', b'R', 0, 0, 0,
    1, 0, 0, 0, 1, 8, 6, 0, 0, 0, 0x1F, 0x15, 0xC4, 0x89, 0, 0, 0, 0x0A, b'I', b'D', b'A', b'T',
    0x78, 0x9C, 0x63, 0, 1, 0, 0, 5, 0, 1, 0x0D, 0x0A, 0x2D, 0xB4, 0, 0, 0, 0, b'I', b'E', b'N',
    b'D', 0xAE, 0x42, 0x60, 0x82,
];

/// A text box whose story is one inline PNG.
fn package() -> Vec<u8> {
    let picture = r#"<w:p><w:r><w:drawing><wp:inline xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"><wp:extent cx="914400" cy="914400"/><wp:docPr id="2" name="Pic" descr="dot"/><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:nvPicPr><pic:cNvPr id="0" name="dot.png"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed="rIdX0"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="914400" cy="914400"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#;
    let body = format!(
        r#"<w:p><w:r><w:t>Before.</w:t></w:r></w:p><w:p><w:r><w:drawing><wp:anchor xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" distT="0" distB="0" distL="114300" distR="114300" simplePos="0" relativeHeight="251658240" behindDoc="0" locked="0" layoutInCell="1" allowOverlap="1"><wp:simplePos x="0" y="0"/><wp:positionH relativeFrom="column"><wp:posOffset>0</wp:posOffset></wp:positionH><wp:positionV relativeFrom="paragraph"><wp:posOffset>0</wp:posOffset></wp:positionV><wp:extent cx="2374265" cy="457200"/><wp:effectExtent l="0" t="0" r="0" b="0"/><wp:wrapSquare wrapText="bothSides"/><wp:docPr id="7" name="Text Box 7"/><wp:cNvGraphicFramePr/><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><wps:wsp xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><wps:cNvSpPr txBox="1"/><wps:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="2374265" cy="457200"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></wps:spPr><wps:txbx><w:txbxContent>{picture}</w:txbxContent></wps:txbx><wps:bodyPr rot="0" vert="horz" wrap="square" lIns="91440" tIns="45720" rIns="91440" bIns="45720" anchor="t"/></wps:wsp></a:graphicData></a:graphic></wp:anchor></w:drawing></w:r></w:p><w:p><w:r><w:t>After.</w:t></w:r></w:p>"#
    );
    let image_rel = format!("{}/image", common::docx::R_NS);
    let docx = common::docx::docx_with(
        &body,
        &[common::docx::Part {
            name: "word/media/dot.png",
            content_type: "image/png",
            rel_type: &image_rel,
            xml: "",
        }],
    );
    common::docx::replace_entry(&docx, "word/media/dot.png", PNG_1X1)
}

fn png_parts(docx: &[u8]) -> Vec<Vec<u8>> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
    let mut found = Vec::new();
    for index in 0..zip.len() {
        let mut file = zip.by_index(index).unwrap();
        let name = file.name().expect("decode ZIP fixture name").to_string();
        if name.contains("media/") && name.ends_with(".png") {
            let mut data = Vec::new();
            file.read_to_end(&mut data).unwrap();
            found.push(data);
        }
    }
    found
}

fn document_xml(docx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
    let mut xml = String::new();
    zip.by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    xml
}

#[test]
fn a_picture_inside_a_text_box_round_trips() {
    let source = package();
    let options = ReadOptions {
        page_markers: false,
        ..ReadOptions::default()
    };
    let view = read(&source, &options).expect("the docx reads").markdown;
    let written = markdown_to_docx(&view, &DocxOptions::default())
        .unwrap_or_else(|error| panic!("picture in a text box refused the round trip: {error}"));
    let xml = document_xml(&written.docx);
    let boxed = xml
        .split("<w:txbxContent")
        .nth(1)
        .unwrap_or_else(|| panic!("the text box lost its story: {xml}"));
    assert!(
        boxed.contains("blip") || boxed.contains("a:blip"),
        "the text box lost its picture: {boxed}"
    );
    assert_eq!(png_parts(&written.docx), vec![PNG_1X1.to_vec()]);
    common::validity::assert_word_valid_package(&written.docx);
}
