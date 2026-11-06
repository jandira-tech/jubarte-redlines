// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A picture inside a nested text box's fallback survives docx to markdown to docx.

mod common;

use jubarte::markdown::{DocxOptions, ReadOptions, markdown_to_docx, read};
use std::io::Read as _;

const PNG_CHOICE: &[u8] = &[
    0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0x0D, b'I', b'H', b'D', b'R', 0, 0, 0,
    1, 0, 0, 0, 1, 8, 6, 0, 0, 0, 0x1F, 0x15, 0xC4, 0x89, 0, 0, 0, 0x0A, b'I', b'D', b'A', b'T',
    0x78, 0x9C, 0x63, 0, 1, 0, 0, 5, 0, 1, 0x0D, 0x0A, 0x2D, 0xB4, 0, 0, 0, 0, b'I', b'E', b'N',
    b'D', 0xAE, 0x42, 0x60, 0x82,
];

const PNG_FALLBACK: &[u8] = &[
    0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0x0D, b'I', b'H', b'D', b'R', 0, 0, 0,
    1, 0, 0, 0, 1, 8, 6, 0, 0, 0, 0x1F, 0x15, 0xC4, 0x89, 0, 0, 0, 0x0A, b'I', b'D', b'A', b'T',
    0x78, 0x9C, 0x63, 0, 1, 0, 0, 5, 0, 1, 0x0D, 0x0A, 0x2D, 0xB4, 0, 0, 0, 0, b'I', b'E', b'N',
    b'D', 0xAE, 0x42, 0x60, 0x83,
];

/// A wps text box. Callers pass the same frame for a choice and its fallback.
fn frame(story: &str) -> String {
    format!(
        r#"<w:drawing><wp:anchor xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"><wp:extent cx="914400" cy="914400"/><wp:docPr id="7" name="Text Box 7"/><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><wps:wsp xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><wps:cNvSpPr txBox="1"/><wps:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="914400" cy="914400"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></wps:spPr><wps:txbx><w:txbxContent>{story}</w:txbxContent></wps:txbx><wps:bodyPr/></wps:wsp></a:graphicData></a:graphic></wp:anchor></w:drawing>"#
    )
}

/// A plain inline picture. `embed` is the only difference between the two copies.
fn picture(embed: &str) -> String {
    format!(
        r#"<w:p><w:r><w:drawing><wp:inline xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"><wp:extent cx="914400" cy="914400"/><wp:docPr id="2" name="Pic"/><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:pic xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture"><pic:nvPicPr><pic:cNvPr id="0" name="dot.png"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed="{embed}"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="914400" cy="914400"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>"#
    )
}

/// Choice and fallback are the same wps frame. Each holds one nested text box,
/// and those nested boxes differ only by the picture they embed.
fn package() -> Vec<u8> {
    let branch = |embed: &str| {
        let nested = frame(&picture(embed));
        frame(&format!(
            r#"<w:p><w:r><w:t>Keep.</w:t></w:r><w:r>{nested}</w:r></w:p>"#
        ))
    };
    let body = format!(
        r#"<w:p><w:r><mc:AlternateContent xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006"><mc:Choice Requires="wps">{}</mc:Choice><mc:Fallback>{}</mc:Fallback></mc:AlternateContent></w:r></w:p>"#,
        branch("rIdX0"),
        branch("rIdX1")
    );
    let image_rel = format!("{}/image", common::docx::R_NS);
    let docx = common::docx::docx_with(
        &body,
        &[
            common::docx::Part {
                name: "word/media/a.png",
                content_type: "image/png",
                rel_type: &image_rel,
                xml: "",
            },
            common::docx::Part {
                name: "word/media/b.png",
                content_type: "image/png",
                rel_type: &image_rel,
                xml: "",
            },
        ],
    );
    let docx = common::docx::replace_entry(&docx, "word/media/a.png", PNG_CHOICE);
    common::docx::replace_entry(&docx, "word/media/b.png", PNG_FALLBACK)
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
fn a_fallback_picture_inside_a_nested_text_box_round_trips() {
    let source = package();
    assert!(
        png_parts(&source)
            .iter()
            .any(|part| part.as_slice() == PNG_FALLBACK),
        "the fixture lost its fallback picture"
    );
    let options = ReadOptions {
        page_markers: false,
        ..ReadOptions::default()
    };
    let view = read(&source, &options).expect("the docx reads").markdown;
    let written = markdown_to_docx(&view, &DocxOptions::default())
        .unwrap_or_else(|error| panic!("round trip refused: {error}"));
    let xml = document_xml(&written.docx);
    let pngs = png_parts(&written.docx);
    assert!(
        pngs.iter().any(|part| part.as_slice() == PNG_FALLBACK),
        "the fallback picture was dropped: {xml}"
    );
    assert!(
        pngs.iter().any(|part| part.as_slice() == PNG_CHOICE),
        "the choice picture was dropped: {xml}"
    );
    assert!(
        xml.matches("r:embed").count() >= 2,
        "the written story kept one picture: {xml}"
    );
}
