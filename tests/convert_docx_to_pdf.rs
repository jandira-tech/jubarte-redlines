// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Drive the shipped `docx_to_pdf` entry (library + `jubarte convert` CLI).

use std::io::{Cursor, Read, Write};
use std::process::Command;

use jubarte::convert::{docx_to_pdf, pdf_page_count};
use zip::ZipArchive;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

const BIN: &str = env!("CARGO_BIN_EXE_jubarte");
const FIXTURE: &str = "tests/fixtures/redline/original.docx";

fn minimal_docx(paragraphs: &[&str], table: Option<&[&[&str]]>) -> Vec<u8> {
    let mut body = String::new();
    for para in paragraphs {
        body.push_str("<w:p><w:r><w:t>");
        body.push_str(para);
        body.push_str("</w:t></w:r></w:p>");
    }
    body.push_str(
        "<w:p><w:pPr><w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"1\"/></w:numPr></w:pPr>\
         <w:r><w:t>Listed item</w:t></w:r></w:p>",
    );
    if let Some(rows) = table {
        body.push_str("<w:tbl>");
        for row in rows {
            body.push_str("<w:tr>");
            for cell in *row {
                body.push_str("<w:tc><w:p><w:r><w:t>");
                body.push_str(cell);
                body.push_str("</w:t></w:r></w:p></w:tc>");
            }
            body.push_str("</w:tr>");
        }
        body.push_str("</w:tbl>");
    }
    let document = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
         <w:body>{body}<w:sectPr/></w:body></w:document>"
    );
    let content_types = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
        <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
        <Default Extension=\"xml\" ContentType=\"application/xml\"/>\
        <Override PartName=\"/word/document.xml\" \
          ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>\
        </Types>";
    let rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
        <Relationship Id=\"rId1\" \
          Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" \
          Target=\"word/document.xml\"/>\
        </Relationships>";
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let opts = SimpleFileOptions::default();
    zip.start_file("[Content_Types].xml", opts).unwrap();
    zip.write_all(content_types.as_bytes()).unwrap();
    zip.start_file("_rels/.rels", opts).unwrap();
    zip.write_all(rels.as_bytes()).unwrap();
    zip.start_file("word/document.xml", opts).unwrap();
    zip.write_all(document.as_bytes()).unwrap();
    zip.finish().unwrap().into_inner()
}

fn minimal_docx_body(body: &str) -> Vec<u8> {
    let document = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
         <w:body>{body}</w:body></w:document>"
    );
    let content_types = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
        <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
        <Default Extension=\"xml\" ContentType=\"application/xml\"/>\
        <Override PartName=\"/word/document.xml\" \
          ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>\
        </Types>";
    let rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
        <Relationship Id=\"rId1\" \
          Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" \
          Target=\"word/document.xml\"/>\
        </Relationships>";
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let opts = SimpleFileOptions::default();
    zip.start_file("[Content_Types].xml", opts).unwrap();
    zip.write_all(content_types.as_bytes()).unwrap();
    zip.start_file("_rels/.rels", opts).unwrap();
    zip.write_all(rels.as_bytes()).unwrap();
    zip.start_file("word/document.xml", opts).unwrap();
    zip.write_all(document.as_bytes()).unwrap();
    zip.finish().unwrap().into_inner()
}

#[test]
fn iso_strict_fixture_converts_to_pdf() {
    let bytes = std::fs::read("tests/fixtures/strict/Strict01.docx").expect("strict fixture");
    let pdf = docx_to_pdf(&bytes).expect("strict convert");
    assert!(pdf.starts_with(b"%PDF"));
    assert!(pdf_page_count(&pdf) >= 1);
}

#[test]
fn shipped_docx_to_pdf_writes_real_pdf_with_a_page() {
    let bytes = std::fs::read(FIXTURE).expect("fixture");
    let pdf = docx_to_pdf(&bytes).expect("convert");
    assert!(
        pdf.starts_with(b"%PDF"),
        "header {:?}",
        &pdf[..pdf.len().min(8)]
    );
    assert!(
        pdf_page_count(&pdf) >= 1,
        "expected at least one /Type /Page"
    );
    let text = String::from_utf8_lossy(&pdf);
    assert!(text.contains("/Producer (jubarte)"), "must not be soffice");
    assert!(
        text.contains("/FontFile2"),
        "must embed a TrueType face (Carlito/Liberation), not Helvetica"
    );
}

#[test]
fn long_docx_emits_multiple_pages() {
    let paras: Vec<String> = (0..80)
        .map(|i| format!("Paragraph {i} lorem ipsum dolor sit amet, consectetur adipiscing elit."))
        .collect();
    let refs: Vec<&str> = paras.iter().map(String::as_str).collect();
    let table = [["Name", "Value"], ["Alpha", "1"], ["Beta", "2"]];
    let table_refs: Vec<&[&str]> = table.iter().map(|row| row.as_slice()).collect();
    let docx = minimal_docx(&refs, Some(&table_refs));
    let pdf = docx_to_pdf(&docx).expect("convert long");
    assert!(pdf.starts_with(b"%PDF"));
    assert!(
        pdf_page_count(&pdf) >= 2,
        "long document must paginate, got {}",
        pdf_page_count(&pdf)
    );
}

#[test]
fn explicit_page_break_starts_a_new_page() {
    let docx = minimal_docx_body(
        "<w:p><w:r><w:t>First page</w:t></w:r></w:p>\
         <w:p><w:r><w:br w:type=\"page\"/></w:r></w:p>\
         <w:p><w:r><w:t>Second page</w:t></w:r></w:p>\
         <w:sectPr/>",
    );
    let pdf = docx_to_pdf(&docx).expect("convert page break");
    assert_eq!(
        pdf_page_count(&pdf),
        2,
        "w:br type=page must start a new page"
    );
}

#[test]
fn next_page_section_break_starts_a_new_page() {
    let docx = minimal_docx_body(
        "<w:p><w:r><w:t>Section one</w:t></w:r></w:p>\
         <w:p><w:pPr><w:sectPr><w:type w:val=\"nextPage\"/></w:sectPr></w:pPr></w:p>\
         <w:p><w:r><w:t>Section two</w:t></w:r></w:p>\
         <w:sectPr/>",
    );
    let pdf = docx_to_pdf(&docx).expect("convert section break");
    assert_eq!(
        pdf_page_count(&pdf),
        2,
        "nextPage sectPr must start a new page"
    );
}

#[test]
fn adjacent_page_and_section_breaks_coalesce() {
    // Word often emits an empty page-break para plus an empty next-page
    // sectPr para for one visual page transition (multi_section fixture).
    let docx = minimal_docx_body(
        "<w:p><w:r><w:t>One</w:t></w:r></w:p>\
         <w:p><w:r><w:br w:type=\"page\"/></w:r></w:p>\
         <w:p><w:pPr><w:sectPr><w:type w:val=\"nextPage\"/></w:sectPr></w:pPr></w:p>\
         <w:p><w:r><w:t>Two</w:t></w:r></w:p>\
         <w:p><w:r><w:br w:type=\"page\"/></w:r></w:p>\
         <w:p><w:pPr><w:sectPr><w:type w:val=\"nextPage\"/></w:sectPr></w:pPr></w:p>\
         <w:p><w:r><w:t>Three</w:t></w:r></w:p>\
         <w:sectPr/>",
    );
    let pdf = docx_to_pdf(&docx).expect("convert coalesced breaks");
    assert_eq!(
        pdf_page_count(&pdf),
        3,
        "adjacent page+section breaks are one transition, not a blank page"
    );
}

#[test]
fn continuous_section_does_not_add_a_page() {
    let docx = minimal_docx_body(
        "<w:p><w:r><w:t>Still one page</w:t></w:r></w:p>\
         <w:p><w:pPr><w:sectPr><w:type w:val=\"continuous\"/></w:sectPr></w:pPr>\
         <w:r><w:t>Also first page</w:t></w:r></w:p>\
         <w:sectPr/>",
    );
    let pdf = docx_to_pdf(&docx).expect("convert continuous");
    assert_eq!(
        pdf_page_count(&pdf),
        1,
        "continuous sectPr must not force a page"
    );
}

#[test]
fn trailing_body_sectpr_does_not_add_a_page() {
    let docx = minimal_docx_body("<w:p><w:r><w:t>Only page</w:t></w:r></w:p><w:sectPr/>");
    let pdf = docx_to_pdf(&docx).expect("convert trailing sectPr");
    assert_eq!(pdf_page_count(&pdf), 1);
}

#[test]
fn last_paragraph_sectpr_does_not_add_a_page() {
    let docx = minimal_docx_body(
        "<w:p><w:r><w:t>Only page</w:t></w:r></w:p>\
         <w:p><w:pPr><w:sectPr><w:type w:val=\"nextPage\"/></w:sectPr></w:pPr>\
         <w:r><w:t>Still only page</w:t></w:r></w:p>",
    );
    let pdf = docx_to_pdf(&docx).expect("convert last-para sectPr");
    assert_eq!(
        pdf_page_count(&pdf),
        1,
        "final paragraph sectPr is page setup, not a break"
    );
}

/// 1×1 RGB PNG.
const TINY_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
    0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00,
    0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D, 0xB0, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
    0x44, 0xAE, 0x42, 0x60, 0x82,
];

fn drawing_docx(body: &str) -> Vec<u8> {
    drawing_docx_media(body, "dot.png", TINY_PNG)
}

fn drawing_docx_media(body: &str, media_name: &str, media: &[u8]) -> Vec<u8> {
    let document = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" \
           xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" \
           xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\" \
           xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" \
           xmlns:pic=\"http://schemas.openxmlformats.org/drawingml/2006/picture\">\
         <w:body>{body}</w:body></w:document>"
    );
    let content_types = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
        <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
        <Default Extension=\"xml\" ContentType=\"application/xml\"/>\
        <Default Extension=\"png\" ContentType=\"image/png\"/>\
        <Override PartName=\"/word/document.xml\" \
          ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>\
        </Types>";
    let rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
        <Relationship Id=\"rId1\" \
          Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" \
          Target=\"word/document.xml\"/>\
        </Relationships>";
    let doc_rels = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
        <Relationship Id=\"rIdImg\" \
          Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/image\" \
          Target=\"media/{media_name}\"/>\
        </Relationships>"
    );
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let opts = SimpleFileOptions::default();
    zip.start_file("[Content_Types].xml", opts).unwrap();
    zip.write_all(content_types.as_bytes()).unwrap();
    zip.start_file("_rels/.rels", opts).unwrap();
    zip.write_all(rels.as_bytes()).unwrap();
    zip.start_file("word/document.xml", opts).unwrap();
    zip.write_all(document.as_bytes()).unwrap();
    zip.start_file("word/_rels/document.xml.rels", opts)
        .unwrap();
    zip.write_all(doc_rels.as_bytes()).unwrap();
    zip.start_file(format!("word/media/{media_name}"), opts)
        .unwrap();
    zip.write_all(media).unwrap();
    zip.finish().unwrap().into_inner()
}

fn chart_docx(body: &str, chart_xml: &str) -> Vec<u8> {
    let document = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" \
           xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" \
           xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\" \
           xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">\
         <w:body>{body}</w:body></w:document>"
    );
    let content_types = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
        <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
        <Default Extension=\"xml\" ContentType=\"application/xml\"/>\
        <Override PartName=\"/word/document.xml\" \
          ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>\
        <Override PartName=\"/word/charts/chart1.xml\" \
          ContentType=\"application/vnd.openxmlformats-officedocument.drawingml.chart+xml\"/>\
        </Types>";
    let rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
        <Relationship Id=\"rId1\" \
          Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" \
          Target=\"word/document.xml\"/>\
        </Relationships>";
    let doc_rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
        <Relationship Id=\"rIdChart\" \
          Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart\" \
          Target=\"charts/chart1.xml\"/>\
        </Relationships>";
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let opts = SimpleFileOptions::default();
    zip.start_file("[Content_Types].xml", opts).unwrap();
    zip.write_all(content_types.as_bytes()).unwrap();
    zip.start_file("_rels/.rels", opts).unwrap();
    zip.write_all(rels.as_bytes()).unwrap();
    zip.start_file("word/document.xml", opts).unwrap();
    zip.write_all(document.as_bytes()).unwrap();
    zip.start_file("word/_rels/document.xml.rels", opts)
        .unwrap();
    zip.write_all(doc_rels.as_bytes()).unwrap();
    zip.start_file("word/charts/chart1.xml", opts).unwrap();
    zip.write_all(chart_xml.as_bytes()).unwrap();
    zip.finish().unwrap().into_inner()
}

fn fixture_zip_part(docx_path: &str, part: &str) -> Vec<u8> {
    let bytes = std::fs::read(docx_path).expect(docx_path);
    let mut zip = ZipArchive::new(Cursor::new(bytes)).expect("docx zip");
    let mut file = zip
        .by_name(part)
        .unwrap_or_else(|_| panic!("missing {part}"));
    let mut out = Vec::new();
    file.read_to_end(&mut out).expect("read part");
    out
}

fn blip(cx: &str, cy: &str, inner_open: &str, inner_close: &str) -> String {
    format!(
        "<w:drawing>{inner_open}\
           <wp:extent cx=\"{cx}\" cy=\"{cy}\"/>\
           <wp:docPr id=\"1\" name=\"Picture 0\" descr=\"dot.png\"/>\
           <a:graphic><a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/picture\">\
             <pic:pic><pic:blipFill><a:blip r:embed=\"rIdImg\"/></pic:blipFill></pic:pic>\
           </a:graphicData></a:graphic>\
         {inner_close}</w:drawing>"
    )
}

#[test]
fn inline_extent_is_written_to_pdf_cm() {
    // 137160 EMU = 10.8 pt. The previous default (missing unnamespaced cx/cy)
    // emitted q 200.00 0 0 120.00.
    let drawing = blip(
        "137160",
        "137160",
        "<wp:inline distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\">",
        "</wp:inline>",
    );
    let docx = drawing_docx(&format!(
        "<w:p><w:r><w:t>Hello</w:t></w:r><w:r>{drawing}</w:r></w:p><w:sectPr/>"
    ));
    let pdf = docx_to_pdf(&docx).expect("convert inline image");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("10.80 0 0 10.80"),
        "expected 10.80pt image cm, got snippet around Im: {}",
        text.split("/Im")
            .nth(1)
            .unwrap_or(&text[text.len().saturating_sub(200)..])
    );
    assert!(
        !text.contains("200.00 0 0 120.00"),
        "must not fall back to the 200x120 default"
    );
}

#[test]
fn placeable_wmf_blip_paints_rgb_ink() {
    // Strict01 image1.bin is a placeable WMF (d7 cd c6 9a) of filled
    // polygons — the P2 hammer/computer clipart. Reserve-only leaves a
    // white hole; the shipped convert path must rasterize it to RGB.
    let wmf = fixture_zip_part(
        "tests/fixtures/strict/Strict01.docx",
        "word/media/image1.bin",
    );
    assert_eq!(
        &wmf[..4],
        b"\xd7\xcd\xc6\x9a",
        "fixture must stay placeable WMF"
    );
    let drawing = blip(
        "2000000",
        "7800000",
        "<wp:inline distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\">",
        "</wp:inline>",
    );
    let docx = drawing_docx_media(
        &format!(
            "<w:p><w:r><w:t>Before</w:t></w:r><w:r>{drawing}</w:r></w:p>\
             <w:p><w:r><w:t>AfterPic</w:t></w:r></w:p><w:sectPr/>"
        ),
        "clip.wmf",
        &wmf,
    );
    let pdf = docx_to_pdf(&docx).expect("convert placeable WMF");
    assert_eq!(
        pdf_page_count(&pdf),
        2,
        "WMF extent must still reserve flow (AfterPic on page 2)"
    );
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/Subtype /Image"),
        "WMF must be painted as a PDF image, not an empty reserve; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
    assert!(
        rgb_image_has_dark_samples(&pdf),
        "rasterized WMF must contain ink, not an all-white bitmap"
    );
}

#[test]
fn emf_blip_paints_rgb_ink() {
    // Strict01 image2.emf is a 448×200 EMF of pen strokes + PATCOPY BITBLT.
    let emf = fixture_zip_part(
        "tests/fixtures/strict/Strict01.docx",
        "word/media/image2.emf",
    );
    assert_eq!(&emf[40..44], b" EMF", "fixture must stay EMF");
    let drawing = blip(
        "1903730",
        "1515822",
        "<wp:inline distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\">",
        "</wp:inline>",
    );
    let docx = drawing_docx_media(
        &format!("<w:p><w:r>{drawing}</w:r><w:r><w:t>AfterEmf</w:t></w:r></w:p><w:sectPr/>"),
        "clip.emf",
        &emf,
    );
    let pdf = docx_to_pdf(&docx).expect("convert EMF");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/Subtype /Image"),
        "EMF must be painted as a PDF image; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
    assert!(
        rgb_image_has_dark_samples(&pdf),
        "rasterized EMF must contain ink"
    );
}

fn rgb_image_has_dark_samples(pdf: &[u8]) -> bool {
    let hay = String::from_utf8_lossy(pdf);
    let Some(idx) = hay.find("/Subtype /Image") else {
        return false;
    };
    let rest = &pdf[idx..];
    let Some(stream_at) = rest.windows(7).position(|w| w == b"stream\n") else {
        return false;
    };
    let data = &rest[stream_at + 7..];
    let end = data
        .windows(9)
        .position(|w| w == b"endstream")
        .unwrap_or(data.len().min(200_000));
    data[..end].iter().any(|&b| b < 200)
}

#[test]
fn undecodable_inline_blip_still_reserves_flow() {
    // Strict01 cliparts are WMF/EMF (`image1.bin` starts d7 cd c6 9a). If
    // decode fails we currently drop the drawing, so the next text stays on
    // page 1. The extent must still consume flow (8000000 EMU ≈ 630pt).
    let drawing = blip(
        "2000000",
        "7800000",
        "<wp:inline distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\">",
        "</wp:inline>",
    );
    const WMF: &[u8] = b"\xd7\xcd\xc6\x9a\x00\x00not-an-image";
    let docx = drawing_docx_media(
        &format!(
            "<w:p><w:r><w:t>Before</w:t></w:r><w:r>{drawing}</w:r></w:p>\
             <w:p><w:r><w:t>AfterPic</w:t></w:r></w:p><w:sectPr/>"
        ),
        "clip.wmf",
        WMF,
    );
    let pdf = docx_to_pdf(&docx).expect("convert undecodable blip");
    assert_eq!(
        pdf_page_count(&pdf),
        2,
        "WMF/EMF inline extent must still push following text to page 2"
    );
}

#[test]
fn floating_anchors_do_not_force_a_second_page() {
    // Two ~393pt wrapSquare anchors would overflow a letter page if stacked in
    // flow (anchor_images failure mode). Overlays stay on one page.
    let left = blip(
        "5000000",
        "5000000",
        "<wp:anchor distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\" simplePos=\"0\" \
           relativeHeight=\"1\" behindDoc=\"0\" locked=\"0\" layoutInCell=\"1\" allowOverlap=\"1\">\
           <wp:positionH relativeFrom=\"margin\"><wp:align>left</wp:align></wp:positionH>\
           <wp:positionV relativeFrom=\"margin\"><wp:align>top</wp:align></wp:positionV>\
           <wp:wrapSquare wrapText=\"bothSides\"/>",
        "</wp:anchor>",
    );
    let right = blip(
        "5000000",
        "5000000",
        "<wp:anchor distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\" simplePos=\"0\" \
           relativeHeight=\"1\" behindDoc=\"0\" locked=\"0\" layoutInCell=\"1\" allowOverlap=\"1\">\
           <wp:positionH relativeFrom=\"margin\"><wp:align>right</wp:align></wp:positionH>\
           <wp:positionV relativeFrom=\"margin\"><wp:align>top</wp:align></wp:positionV>\
           <wp:wrapSquare wrapText=\"bothSides\"/>",
        "</wp:anchor>",
    );
    let docx = drawing_docx(&format!(
        "<w:p><w:r><w:t>Title</w:t></w:r>\
           <w:r>{left}</w:r>\
           <w:r>{right}</w:r></w:p>\
         <w:p><w:r><w:t>Body text beside floats.</w:t></w:r></w:p><w:sectPr/>"
    ));
    let pdf = docx_to_pdf(&docx).expect("convert floats");
    assert_eq!(
        pdf_page_count(&pdf),
        1,
        "wrapSquare anchors overlay; they must not stack into a second page"
    );
}

#[test]
fn text_box_txbx_content_emits_a_bordered_box() {
    let body = "<w:p><w:r>\
        <w:drawing><wp:anchor simplePos=\"0\" relativeHeight=\"1\" behindDoc=\"0\" \
          locked=\"0\" layoutInCell=\"1\" allowOverlap=\"1\">\
          <wp:positionH relativeFrom=\"column\"><wp:align>center</wp:align></wp:positionH>\
          <wp:positionV relativeFrom=\"paragraph\"><wp:posOffset>0</wp:posOffset></wp:positionV>\
          <wp:extent cx=\"2374265\" cy=\"1403985\"/>\
          <wp:wrapNone/>\
          <wp:docPr id=\"1\" name=\"Text Box 2\"/>\
          <w:txbxContent><w:p><w:r><w:t>Datum plane</w:t></w:r></w:p></w:txbxContent>\
        </wp:anchor></w:drawing></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&drawing_docx(body)).expect("convert text box");
    assert_eq!(pdf_page_count(&pdf), 1);
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("0.60 w"),
        "textbox must stroke a border, stream tail {}",
        &text[text.len().saturating_sub(240)..]
    );
}

#[test]
fn page_origin_float_is_not_stuck_at_the_margin() {
    // image_out_of_folder / anchor_images: wp:positionH/V relativeFrom=page
    // posOffset=0. chrome used to park every float at margin_l/margin_t
    // (~72pt), so the A4 DeepL banner sat in the body instead of the
    // page edge.
    let body = "<w:p><w:r>\
        <w:drawing><wp:anchor simplePos=\"0\" relativeHeight=\"1\" behindDoc=\"0\" \
          locked=\"0\" layoutInCell=\"1\" allowOverlap=\"1\">\
          <wp:positionH relativeFrom=\"page\"><wp:posOffset>0</wp:posOffset></wp:positionH>\
          <wp:positionV relativeFrom=\"page\"><wp:posOffset>0</wp:posOffset></wp:positionV>\
          <wp:extent cx=\"2000000\" cy=\"400000\"/>\
          <wp:wrapNone/>\
          <wp:docPr id=\"1\" name=\"Banner\"/>\
          <w:txbxContent><w:p><w:r><w:t>Edge</w:t></w:r></w:p></w:txbxContent>\
        </wp:anchor></w:drawing></w:r></w:p>\
        <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
          <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&drawing_docx(body)).expect("convert page-origin float");
    let xs = pdf_tf_xs(&pdf, "11.00 Tf");
    assert!(!xs.is_empty(), "banner text must paint; xs={xs:?}");
    let min_x = xs.iter().copied().fold(f32::INFINITY, f32::min);
    assert!(
        min_x < 20.0,
        "page posOffset=0 must sit on the left edge, not margin 72; xs={xs:?}"
    );
    let ys = pdf_horiz_rule_ys(&pdf);
    assert!(ys.len() >= 2, "banner must stroke a box; ys={ys:?}");
    let top = ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        top > 780.0,
        "page posOffset=0 must sit on the top edge (~792), not below margin_t (~720); ys={ys:?}"
    );
}

#[test]
fn empty_diagram_drawing_is_not_stroked() {
    // Strict01 Diagram 1 is an inline graphicData/diagram with no series.
    // Emitting it as an empty stroked box produced a hollow last page.
    let body = "<w:p><w:r><w:drawing><wp:inline>\
          <wp:extent cx=\"5486400\" cy=\"3200400\"/>\
          <wp:docPr id=\"1\" name=\"Diagram 1\"/>\
          <a:graphic><a:graphicData \
            uri=\"http://schemas.microsoft.com/office/drawing/2008/diagram\"/>\
          </a:graphic></wp:inline></w:drawing></w:r>\
          <w:r><w:t>AfterDiagram</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&drawing_docx(body)).expect("convert empty diagram");
    assert_eq!(pdf_page_count(&pdf), 1);
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        !text.contains("0.60 w"),
        "empty diagram must not stroke a hollow box; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
}

#[test]
fn empty_inline_rectangle_is_not_stroked() {
    // Strict01 Rectangle 3 is wp:inline 402×167, no txbx, no blip. Treating
    // it as a flow textbox strokes a hollow box above the chart and drops
    // the four-stem Strict01 cluster to ~35. wrapNone overlays are already
    // skipped; inline empty frames must be too.
    let body = "<w:p><w:r><w:drawing><wp:inline distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\">\
          <wp:extent cx=\"5104263\" cy=\"2122227\"/>\
          <wp:docPr id=\"1\" name=\"Rectangle 3\"/>\
          <a:graphic><a:graphicData \
            uri=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">\
            <wps:wsp xmlns:wps=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\"/>\
          </a:graphicData></a:graphic>\
        </wp:inline></w:drawing></w:r>\
        <w:r><w:t>AfterRect</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&drawing_docx(body)).expect("convert empty inline rect");
    assert_eq!(
        pdf_page_count(&pdf),
        1,
        "empty inline wsp must not consume a page of flow"
    );
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        !text.contains("0.60 w"),
        "empty inline Rectangle 3 must not stroke; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
    assert!(
        !text.contains("401.91") && !text.contains("167.10"),
        "must not emit the 402×167 reserve box; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
}

#[test]
fn empty_wps_shape_is_not_stroked_and_stays_on_one_page() {
    // Empty wrapNone wsp frames (Strict01 cover rectangles) have no txbxContent.
    // Gated empty boxes were a net visual-score loss vs soffice; skip them.
    let body = "<w:p><w:r><w:drawing><wp:anchor simplePos=\"0\" relativeHeight=\"1\" \
          behindDoc=\"0\" locked=\"0\" layoutInCell=\"1\" allowOverlap=\"1\">\
          <wp:positionH relativeFrom=\"margin\"><wp:align>left</wp:align></wp:positionH>\
          <wp:positionV relativeFrom=\"margin\"><wp:align>top</wp:align></wp:positionV>\
          <wp:extent cx=\"5104263\" cy=\"2122227\"/>\
          <wp:wrapNone/>\
          <wp:docPr id=\"1\" name=\"Rectangle 3\"/>\
          <a:graphic><a:graphicData \
            uri=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">\
            <wps:wsp xmlns:wps=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\"/>\
          </a:graphicData></a:graphic>\
        </wp:anchor></w:drawing></w:r>\
        <w:r><w:t>After shape</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&drawing_docx(body)).expect("convert empty shape");
    assert_eq!(
        pdf_page_count(&pdf),
        1,
        "empty wsp overlay must not consume flow"
    );
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        !text.contains("0.60 w"),
        "empty wsp must not stroke a border; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
}

#[test]
fn skinny_connector_shape_is_not_stroked() {
    let body = "<w:p><w:r><w:drawing><wp:anchor simplePos=\"0\" relativeHeight=\"1\" \
          behindDoc=\"0\" locked=\"0\" layoutInCell=\"1\" allowOverlap=\"1\">\
          <wp:extent cx=\"2149522\" cy=\"320723\"/>\
          <wp:wrapNone/>\
          <wp:docPr id=\"1\" name=\"Elbow Connector 6\"/>\
          <a:graphic><a:graphicData \
            uri=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">\
            <wps:wsp xmlns:wps=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\"/>\
          </a:graphicData></a:graphic>\
        </wp:anchor></w:drawing></w:r>\
        <w:r><w:t>After</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&drawing_docx(body)).expect("convert connector");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        !text.contains("0.60 w"),
        "skinny connectors must not stroke a box; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
}

fn numbering_docx(body: &str, numbering: Option<&str>) -> Vec<u8> {
    numbering_docx_with_styles(body, numbering, None)
}

fn numbering_docx_with_styles(
    body: &str,
    numbering: Option<&str>,
    styles: Option<&str>,
) -> Vec<u8> {
    let document = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
         <w:body>{body}</w:body></w:document>"
    );
    let mut types = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
         <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
         <Default Extension=\"xml\" ContentType=\"application/xml\"/>\
         <Override PartName=\"/word/document.xml\" \
           ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>",
    );
    if numbering.is_some() {
        types.push_str(
            "<Override PartName=\"/word/numbering.xml\" \
               ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml\"/>",
        );
    }
    if styles.is_some() {
        types.push_str(
            "<Override PartName=\"/word/styles.xml\" \
               ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml\"/>",
        );
    }
    types.push_str("</Types>");
    let rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
        <Relationship Id=\"rId1\" \
          Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" \
          Target=\"word/document.xml\"/>\
        </Relationships>";
    let doc_rels = {
        let mut rels_xml = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
             <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">",
        );
        if numbering.is_some() {
            rels_xml.push_str(
                "<Relationship Id=\"rIdN\" \
                   Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering\" \
                   Target=\"numbering.xml\"/>",
            );
        }
        if styles.is_some() {
            rels_xml.push_str(
                "<Relationship Id=\"rIdS\" \
                   Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles\" \
                   Target=\"styles.xml\"/>",
            );
        }
        rels_xml.push_str("</Relationships>");
        rels_xml
    };
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let opts = SimpleFileOptions::default();
    zip.start_file("[Content_Types].xml", opts).unwrap();
    zip.write_all(types.as_bytes()).unwrap();
    zip.start_file("_rels/.rels", opts).unwrap();
    zip.write_all(rels.as_bytes()).unwrap();
    zip.start_file("word/document.xml", opts).unwrap();
    zip.write_all(document.as_bytes()).unwrap();
    zip.start_file("word/_rels/document.xml.rels", opts)
        .unwrap();
    zip.write_all(doc_rels.as_bytes()).unwrap();
    if let Some(num) = numbering {
        zip.start_file("word/numbering.xml", opts).unwrap();
        zip.write_all(num.as_bytes()).unwrap();
    }
    if let Some(st) = styles {
        zip.start_file("word/styles.xml", opts).unwrap();
        zip.write_all(st.as_bytes()).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

fn list_items_body() -> &'static str {
    "<w:p><w:r><w:t>Title</w:t></w:r></w:p>\
     <w:p><w:pPr><w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"1\"/></w:numPr></w:pPr>\
       <w:r><w:t>First item</w:t></w:r></w:p>\
     <w:p><w:pPr><w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"1\"/></w:numPr></w:pPr>\
       <w:r><w:t>Second item</w:t></w:r></w:p>\
     <w:sectPr/>"
}

#[test]
fn numpr_without_numbering_xml_still_converts() {
    // Soffice draws no marker when numbering.xml is absent (numbered_list fixture).
    let pdf = docx_to_pdf(&numbering_docx(list_items_body(), None)).expect("convert");
    assert_eq!(pdf_page_count(&pdf), 1);
}

#[test]
fn numbering_hanging_indent_shifts_bullet_off_the_margin() {
    // comments ListBullet: lvl pPr ind left=720 hanging=360 (36pt / 18pt).
    // First-line bullet must sit at margin+left-hanging = 72+36-18 = 90, not 72.
    let numbering = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:numbering xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
          <w:abstractNum w:abstractNumId=\"0\">\
            <w:lvl w:ilvl=\"0\"><w:start w:val=\"1\"/><w:numFmt w:val=\"bullet\"/>\
              <w:lvlText w:val=\"\u{2022}\"/>\
              <w:pPr><w:ind w:left=\"720\" w:hanging=\"360\"/></w:pPr></w:lvl>\
          </w:abstractNum>\
          <w:num w:numId=\"1\"><w:abstractNumId w:val=\"0\"/></w:num>\
        </w:numbering>";
    let body = "<w:p><w:pPr><w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"1\"/></w:numPr></w:pPr>\
         <w:r><w:t>HangItem</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&numbering_docx(body, Some(numbering))).expect("convert hanging list");
    let xs = pdf_tf_xs(&pdf, "11.00 Tf");
    assert!(!xs.is_empty(), "list text must paint; xs={xs:?}");
    let min_x = xs.iter().copied().fold(f32::INFINITY, f32::min);
    assert!(
        (min_x - 90.0).abs() < 4.0,
        "bullet at hanging start 90pt, not margin 72; min_x={min_x} xs={xs:?}"
    );
}

#[test]
fn comments_fixture_fits_oracle_page_count() {
    // docx_lots_of_comments / I_am_sharing / addition* : soffice is 9pp, we
    // emit 10. Shared leftover is Appendix A+B pushed by extra inter-para
    // space (Word uses max(after, before); we were summing).
    let bytes = std::fs::read(
        "../neurotic_docx_bench/corpus/word_based/docx_source/docx_lots_of_comments.docx",
    )
    .expect("comments fixture");
    let pdf = docx_to_pdf(&bytes).expect("convert comments");
    assert_eq!(
        pdf_page_count(&pdf),
        9,
        "comments cluster must match soffice 9 pages, got {}",
        pdf_page_count(&pdf)
    );
}

#[test]
fn comments_addition_matches_oracle_page_count() {
    // addition / addition_redline* : soffice is 11pp (redline 12). We emit
    // 10 because TableGrid wrapped rows drop the +8pt cell chrome, so the
    // inserted capability matrix finishes on page 10 instead of spilling
    // its last three rows. comments itself stays 9 — page 9 is almost empty.
    let bytes = std::fs::read(
        "../neurotic_docx_bench/corpus/word_based/docx_source/docx_lots_of_comments_addition.docx",
    )
    .expect("comments addition fixture");
    let pdf = docx_to_pdf(&bytes).expect("convert comments addition");
    assert_eq!(
        pdf_page_count(&pdf),
        11,
        "addition must match soffice 11 pages, got {}",
        pdf_page_count(&pdf)
    );
}

#[test]
fn adjacent_para_spacing_is_max_not_sum() {
    // after=720 (36pt) then before=720 must be one 36pt gap, not 72pt.
    let docx = minimal_docx_body(
        "<w:p><w:pPr><w:spacing w:after=\"720\" w:line=\"240\" w:lineRule=\"auto\"/></w:pPr>\
           <w:r><w:rPr><w:sz w:val=\"22\"/></w:rPr><w:t>Alpha</w:t></w:r></w:p>\
         <w:p><w:pPr><w:spacing w:before=\"720\" w:line=\"240\" w:lineRule=\"auto\"/></w:pPr>\
           <w:r><w:rPr><w:sz w:val=\"22\"/></w:rPr><w:t>Bravo</w:t></w:r></w:p>\
         <w:sectPr/>",
    );
    let pdf = docx_to_pdf(&docx).expect("convert spacing");
    let ys = pdf_tf_ys(&pdf, "11.00 Tf");
    assert!(ys.len() >= 2, "both lines must paint; ys={ys:?}");
    let gap = (ys[0] - ys[1]).abs();
    assert!(
        gap < 60.0,
        "Word uses max(after,before)=36pt plus the line, not 72pt sum; gap={gap} ys={ys:?}"
    );
}

#[test]
fn unstyled_paras_use_default_normal_after_not_docdefaults() {
    // sd_2517: docDefaults after=480 twips (24pt) but Normal (w:default=1)
    // sets after=0. Unstyled paras must use Normal — 24 short lines then
    // stay on one letter page. Using docDefaults after packs ~2 pages.
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
          <w:docDefaults><w:pPrDefault><w:pPr>\
            <w:spacing w:after=\"480\" w:line=\"276\" w:lineRule=\"auto\"/>\
          </w:pPr></w:pPrDefault></w:docDefaults>\
          <w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
            <w:name w:val=\"Normal\"/>\
            <w:pPr><w:spacing w:after=\"0\" w:line=\"240\" w:lineRule=\"auto\"/></w:pPr>\
          </w:style>\
        </w:styles>";
    let mut body = String::new();
    for i in 0..24 {
        body.push_str(&format!("<w:p><w:r><w:t>Line {i}</w:t></w:r></w:p>"));
    }
    body.push_str("<w:sectPr/>");
    let pdf = docx_to_pdf(&numbering_docx_with_styles(&body, None, Some(styles)))
        .expect("convert default Normal");
    assert_eq!(
        pdf_page_count(&pdf),
        1,
        "implicit Normal after=0 must keep 24 short lines on one page"
    );
}

#[test]
fn empty_normal_does_not_invent_word2007_after() {
    // sample_document / eigenpal: styles.xml has empty pPrDefault and empty
    // Normal. Direct `<w:spacing w:before="60"/>` must keep after=0 from that
    // Normal — not a synthetic Word-2007 after=200 twips (10pt). That extra
    // 10pt on every spacer para is why the fixture went 4pp vs soffice 3.
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
          <w:docDefaults><w:pPrDefault/></w:docDefaults>\
          <w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
            <w:name w:val=\"Normal\"/><w:qFormat/>\
          </w:style>\
        </w:styles>";
    let body = "<w:p><w:pPr><w:spacing w:before=\"60\" w:line=\"240\" w:lineRule=\"auto\"/></w:pPr>\
           <w:r><w:rPr><w:sz w:val=\"22\"/></w:rPr><w:t>AAAA</w:t></w:r></w:p>\
         <w:p><w:pPr><w:spacing w:before=\"60\" w:line=\"240\" w:lineRule=\"auto\"/></w:pPr>\
           <w:r><w:rPr><w:sz w:val=\"22\"/></w:rPr><w:t>BBBB</w:t></w:r></w:p>\
         <w:sectPr/>";
    let pdf = docx_to_pdf(&numbering_docx_with_styles(body, None, Some(styles)))
        .expect("convert empty Normal");
    let ys = pdf_tf_ys(&pdf, "11.00 Tf");
    let mut unique: Vec<i32> = ys
        .iter()
        .map(|y| (*y * 10.0).round() as i32)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    unique.sort_by(|a, b| b.cmp(a));
    assert!(
        unique.len() >= 2,
        "both lines must paint; ys={ys:?} unique={unique:?}"
    );
    let gap = (unique[0] - unique[1]) as f32 / 10.0;
    assert!(
        gap < 22.0,
        "empty Normal after=0 + before=3pt + one 11pt line, not +10pt invented after; gap={gap} ys={ys:?}"
    );
}

#[test]
fn docdefaults_after_pt_unit_is_eight_points() {
    // Strict01 ISO spacing uses `w:after="8pt"` / `w:line="12.95pt"`, not
    // twips. Parsing those as f32 fails and used to drop after to 0 once
    // empty-Normal stopped inventing 200 twips — 13pp oracle became 10pp.
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
          <w:docDefaults><w:pPrDefault><w:pPr>\
            <w:spacing w:after=\"8pt\" w:line=\"12.95pt\" w:lineRule=\"auto\"/>\
          </w:pPr></w:pPrDefault></w:docDefaults>\
          <w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
            <w:name w:val=\"Normal\"/><w:qFormat/>\
          </w:style>\
        </w:styles>";
    let body = "<w:p><w:r><w:rPr><w:sz w:val=\"22\"/></w:rPr><w:t>AAAA</w:t></w:r></w:p>\
         <w:p><w:r><w:rPr><w:sz w:val=\"22\"/></w:rPr><w:t>BBBB</w:t></w:r></w:p>\
         <w:sectPr/>";
    let pdf = docx_to_pdf(&numbering_docx_with_styles(body, None, Some(styles)))
        .expect("convert pt spacing");
    let ys = pdf_tf_ys(&pdf, "11.00 Tf");
    let mut unique: Vec<i32> = ys
        .iter()
        .map(|y| (*y * 10.0).round() as i32)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    unique.sort_by(|a, b| b.cmp(a));
    assert!(
        unique.len() >= 2,
        "both lines must paint; ys={ys:?} unique={unique:?}"
    );
    let gap = (unique[0] - unique[1]) as f32 / 10.0;
    assert!(
        (20.0..32.0).contains(&gap),
        "after=8pt must sit in the gap, not be dropped as a failed twip parse; gap={gap} ys={ys:?}"
    );
}

#[test]
fn list_style_numpr_without_para_numpr_converts() {
    // comments cluster: ListBullet/ListNumber put numPr on the style.
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
          <w:style w:type=\"paragraph\" w:styleId=\"ListBullet\">\
            <w:pPr><w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"1\"/></w:numPr></w:pPr>\
          </w:style>\
        </w:styles>";
    let numbering = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:numbering xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
          <w:abstractNum w:abstractNumId=\"0\">\
            <w:lvl w:ilvl=\"0\"><w:start w:val=\"1\"/><w:numFmt w:val=\"bullet\"/>\
              <w:lvlText w:val=\"\u{f0b7}\"/></w:lvl>\
          </w:abstractNum>\
          <w:num w:numId=\"1\"><w:abstractNumId w:val=\"0\"/></w:num>\
        </w:numbering>";
    let body = "<w:p><w:pPr><w:pStyle w:val=\"ListBullet\"/></w:pPr>\
         <w:r><w:t>First item</w:t></w:r></w:p>\
       <w:p><w:pPr><w:pStyle w:val=\"ListBullet\"/></w:pPr>\
         <w:r><w:t>Second item</w:t></w:r></w:p>\
       <w:sectPr/>";
    let pdf = docx_to_pdf(&numbering_docx_with_styles(
        body,
        Some(numbering),
        Some(styles),
    ))
    .expect("convert style-numPr list");
    assert!(pdf.starts_with(b"%PDF"));
    assert_eq!(pdf_page_count(&pdf), 1);
}

#[test]
fn numbering_xml_decimal_list_converts() {
    let numbering = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:numbering xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
          <w:abstractNum w:abstractNumId=\"0\">\
            <w:lvl w:ilvl=\"0\"><w:start w:val=\"1\"/><w:numFmt w:val=\"decimal\"/>\
              <w:lvlText w:val=\"%1.\"/></w:lvl>\
          </w:abstractNum>\
          <w:num w:numId=\"1\"><w:abstractNumId w:val=\"0\"/></w:num>\
        </w:numbering>";
    let pdf = docx_to_pdf(&numbering_docx(list_items_body(), Some(numbering))).expect("convert");
    assert_eq!(pdf_page_count(&pdf), 1);
}

#[test]
fn fixed_width_table_is_not_stretched_to_the_page() {
    // 2000+2000+2000 twips = 300pt. Page content is 468pt; stretching would
    // draw the right edge at 72+468=540 (table_bookmark_end failure mode).
    let body = "<w:p><w:r><w:t>Title</w:t></w:r></w:p>\
         <w:tbl><w:tblPr><w:tblBorders>\
           <w:top w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
           <w:left w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
           <w:bottom w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
           <w:right w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
         </w:tblBorders></w:tblPr><w:tblGrid>\
           <w:gridCol w:w=\"2000\"/><w:gridCol w:w=\"2000\"/><w:gridCol w:w=\"2000\"/>\
         </w:tblGrid>\
         <w:tr>\
           <w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc>\
           <w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc>\
           <w:tc><w:p><w:r><w:t>C</w:t></w:r></w:p></w:tc>\
         </w:tr></w:tbl><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert table");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("372.00"),
        "300pt table ends at 72+300=372; stream tail {}",
        &text[text.len().saturating_sub(240)..]
    );
    assert!(
        !text.contains("540.00"),
        "must not stretch a 300pt table to the 468pt content box"
    );
}

#[test]
fn four_row_table_stays_on_one_page() {
    let mut body = String::from("<w:tbl><w:tblGrid><w:gridCol w:w=\"4000\"/></w:tblGrid>");
    for i in 0..4 {
        body.push_str(&format!(
            "<w:tr><w:tc><w:p><w:r><w:t>Row {i}</w:t></w:r></w:p></w:tc></w:tr>"
        ));
    }
    body.push_str("</w:tbl><w:sectPr/>");
    let pdf = docx_to_pdf(&minimal_docx_body(&body)).expect("convert short table");
    assert_eq!(pdf_page_count(&pdf), 1);
}

#[test]
fn trailing_empty_cell_para_does_not_double_row() {
    // Word cells end with an empty <w:p>. Joining that with \\n made every
    // row two lines (table-median cluster dropped ~2–7 points; Strict01
    // tables spilled). Skip empty cell paras; keep stacking real ones.
    let mut body = String::from("<w:tbl><w:tblGrid><w:gridCol w:w=\"4000\"/></w:tblGrid>");
    for i in 0..30 {
        body.push_str(&format!(
            "<w:tr><w:tc>\
               <w:p><w:r><w:t>Row{i}</w:t></w:r></w:p>\
               <w:p></w:p>\
             </w:tc></w:tr>"
        ));
    }
    body.push_str("</w:tbl><w:sectPr/>");
    let pdf = docx_to_pdf(&minimal_docx_body(&body)).expect("convert trailing empty cell p");
    assert_eq!(
        pdf_page_count(&pdf),
        1,
        "trailing empty cell para must not turn 30 single-line rows into two pages, got {}",
        pdf_page_count(&pdf)
    );
}

#[test]
fn table_cell_hyperlink_keeps_run_color() {
    // sample_document / eigenpal npm|github cell: first run is black "npm",
    // the hyperlink run is 2563EB + underline. emit_table used runs.first()
    // for the whole cell, so the blue never painted (8 stems ~39).
    let body = "<w:tbl><w:tblGrid><w:gridCol w:w=\"8000\"/></w:tblGrid>\
         <w:tr><w:tc><w:p>\
           <w:r><w:t>npm</w:t></w:r>\
           <w:hyperlink>\
             <w:r><w:rPr><w:color w:val=\"2563EB\"/><w:u w:val=\"single\"/></w:rPr>\
               <w:t>pkg</w:t></w:r>\
           </w:hyperlink>\
         </w:p></w:tc></w:tr></w:tbl><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert cell hyperlink");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("0.145 0.388 0.922 rg"),
        "hyperlink run must keep 2563EB; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
    assert!(
        text.contains("0.145 0.388 0.922 RG"),
        "hyperlink underline must stroke 2563EB"
    );
}

#[test]
fn preserved_trailing_spaces_push_the_next_run() {
    // sample_document / eigenpal npm|github cells: generator padding
    // (`npm               `) is xml:space=preserve. soffice still paints
    // one word-gap, not 15 spaces (that ran the github underline off-page).
    let body = "<w:p>\
           <w:r><w:t xml:space=\"preserve\">npm               </w:t></w:r>\
           <w:r><w:rPr><w:sz w:val=\"28\"/><w:color w:val=\"FF0000\"/></w:rPr>\
             <w:t>X</w:t></w:r>\
         </w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert preserve spaces");
    let npm_x = pdf_tf_xs(&pdf, "11.00 Tf");
    let x_x = pdf_tf_xs(&pdf, "14.00 Tf");
    assert!(!npm_x.is_empty() && !x_x.is_empty(), "npm+X must paint");
    let gap = x_x[0] - npm_x[0];
    assert!(
        (12.0..36.0).contains(&gap),
        "generator padding after npm is one word-gap, gap={gap} npm={npm_x:?} x={x_x:?}"
    );
}

#[test]
fn generator_preserve_padding_is_one_word_gap() {
    // sample/eigenpal: every run is xml:space=preserve with ~9–15 trailing
    // spaces of generator padding. soffice paints one word-gap; keeping all
    // of them blew wrap (8 stems ~38.8) and ran the github underline off-page.
    let body = "<w:p>\
           <w:r><w:t xml:space=\"preserve\">Hello         </w:t></w:r>\
           <w:r><w:rPr><w:sz w:val=\"28\"/></w:rPr>\
             <w:t>World</w:t></w:r>\
         </w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert squeezed pad");
    let hello = pdf_tf_xs(&pdf, "11.00 Tf");
    let world = pdf_tf_xs(&pdf, "14.00 Tf");
    assert!(
        !hello.is_empty() && !world.is_empty(),
        "both runs must paint"
    );
    let gap = world[0] - hello[0];
    assert!(
        (20.0..42.0).contains(&gap),
        "generator padding must collapse to one space, gap={gap} hello={hello:?} world={world:?}"
    );
}

#[test]
fn courier_new_embeds_liberation_mono_not_carlito() {
    // sample_document / eigenpal / Strict01: inline code is Courier New.
    // resolve() mapped every unknown family to Carlito, so 31 fixtures
    // painted proportional Calibri metrics for mono runs. Soffice embeds
    // Liberation Mono (Courier-metric).
    let body = "<w:p><w:r>\
           <w:rPr><w:rFonts w:ascii=\"Courier New\" w:hAnsi=\"Courier New\"/>\
             <w:sz w:val=\"22\"/></w:rPr>\
           <w:t>Monoiiii</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert courier");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/LiberationMono"),
        "Courier New must embed Liberation Mono; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
    assert!(
        !text.contains("/Carlito 11.00 Tf"),
        "Courier run must not paint with Carlito"
    );
}

#[test]
fn inter_maps_to_liberation_serif_not_carlito() {
    // sample_document / eigenpal rPrDefault is Inter. Soffice has no Inter
    // and embeds Liberation Serif in the oracle PDF; we painted Carlito
    // (unknown → Calibri-metric), so the 8-stem cluster sat at ~40.
    let body = "<w:p><w:r>\
           <w:rPr><w:rFonts w:ascii=\"Inter\" w:hAnsi=\"Inter\"/>\
             <w:sz w:val=\"22\"/></w:rPr>\
           <w:t>InterBody</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert Inter");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/LiberationSerif"),
        "Inter must embed Liberation Serif like soffice; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
    assert!(
        !text.contains("/Carlito 11.00 Tf"),
        "Inter run must not paint with Carlito"
    );
}

#[test]
fn aptos_maps_to_liberation_sans_not_carlito() {
    // comments / I_am_sharing: body is Aptos. Soffice substitutes
    // Liberation Sans; we painted Carlito so the 6-stem cluster sat at ~40.
    let body = "<w:p><w:r>\
           <w:rPr><w:rFonts w:ascii=\"Aptos\" w:hAnsi=\"Aptos\"/>\
             <w:sz w:val=\"22\"/></w:rPr>\
           <w:t>AptosBody</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert Aptos");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/LiberationSans"),
        "Aptos must embed Liberation Sans like soffice; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
    assert!(
        !text.contains("/Carlito 11.00 Tf"),
        "Aptos run must not paint with Carlito"
    );
}

fn docx_with_styles_and_theme(body: &str, styles: &str, theme: &str) -> Vec<u8> {
    let document = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
         <w:body>{body}</w:body></w:document>"
    );
    let content_types = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
        <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
        <Default Extension=\"xml\" ContentType=\"application/xml\"/>\
        <Override PartName=\"/word/document.xml\" \
          ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>\
        <Override PartName=\"/word/styles.xml\" \
          ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml\"/>\
        <Override PartName=\"/word/theme/theme1.xml\" \
          ContentType=\"application/vnd.openxmlformats-officedocument.theme+xml\"/>\
        </Types>";
    let rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
        <Relationship Id=\"rId1\" \
          Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" \
          Target=\"word/document.xml\"/>\
        </Relationships>";
    let doc_rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
        <Relationship Id=\"rIdS\" \
          Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles\" \
          Target=\"styles.xml\"/>\
        <Relationship Id=\"rIdT\" \
          Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme\" \
          Target=\"theme/theme1.xml\"/>\
        </Relationships>";
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let opts = SimpleFileOptions::default();
    zip.start_file("[Content_Types].xml", opts).unwrap();
    zip.write_all(content_types.as_bytes()).unwrap();
    zip.start_file("_rels/.rels", opts).unwrap();
    zip.write_all(rels.as_bytes()).unwrap();
    zip.start_file("word/document.xml", opts).unwrap();
    zip.write_all(document.as_bytes()).unwrap();
    zip.start_file("word/_rels/document.xml.rels", opts)
        .unwrap();
    zip.write_all(doc_rels.as_bytes()).unwrap();
    zip.start_file("word/styles.xml", opts).unwrap();
    zip.write_all(styles.as_bytes()).unwrap();
    zip.start_file("word/theme/theme1.xml", opts).unwrap();
    zip.write_all(theme.as_bytes()).unwrap();
    zip.finish().unwrap().into_inner()
}

#[test]
fn heading1_ascii_theme_major_embeds_theme_calibri_not_body_aptos() {
    // comments: Heading1 is asciiTheme=majorHAnsi with no ascii name.
    // Theme major latin is Calibri → Carlito-Bold. We inherited
    // Normal=Aptos → Liberation Sans. Do not put an ascii cache on
    // this style — explicit ascii wins, and I_am_sharing's Aptos Display
    // cache must stay Aptos.
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
             <w:name w:val=\"Normal\"/>\
             <w:rPr><w:rFonts w:ascii=\"Aptos\" w:hAnsi=\"Aptos\"/>\
               <w:sz w:val=\"22\"/></w:rPr>\
           </w:style>\
           <w:style w:type=\"paragraph\" w:styleId=\"Heading1\">\
             <w:name w:val=\"heading 1\"/>\
             <w:basedOn w:val=\"Normal\"/>\
             <w:rPr>\
               <w:rFonts w:asciiTheme=\"majorHAnsi\" w:hAnsiTheme=\"majorHAnsi\"/>\
               <w:b/><w:color w:val=\"1F4E79\"/><w:sz w:val=\"28\"/>\
             </w:rPr>\
           </w:style>\
         </w:styles>";
    let theme = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <a:theme xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">\
           <a:themeElements><a:fontScheme name=\"Office\">\
             <a:majorFont><a:latin typeface=\"Calibri\"/></a:majorFont>\
             <a:minorFont><a:latin typeface=\"Cambria\"/></a:minorFont>\
           </a:fontScheme></a:themeElements>\
         </a:theme>";
    let body = "<w:p><w:pPr><w:pStyle w:val=\"Heading1\"/></w:pPr>\
           <w:r><w:t>ThemeHead</w:t></w:r></w:p>\
         <w:p><w:r><w:t>AptosBody</w:t></w:r></w:p>\
         <w:sectPr/>";
    let pdf = docx_to_pdf(&docx_with_styles_and_theme(body, styles, theme))
        .expect("convert themed Heading1");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/Carlito"),
        "Heading1 majorHAnsi=Calibri must embed Carlito; tail {}",
        &text[text.len().saturating_sub(320)..]
    );
    assert!(
        text.contains("/LiberationSans"),
        "Normal Aptos body must stay Liberation Sans; tail {}",
        &text[text.len().saturating_sub(320)..]
    );
    assert!(
        text.contains("14.00 Tf"),
        "Heading1 sz=28 is 14pt; tail {}",
        &text[text.len().saturating_sub(200)..]
    );
}

fn sd_2517_heading_styles_xml() -> &'static str {
    // sd_2517: empty-ish Normal after=0 line=240 Times 12, plus the two
    // custom heading styles that carry before=120 after=120 (6pt+6pt).
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
     <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
       <w:docDefaults><w:pPrDefault><w:pPr>\
         <w:spacing w:after=\"480\" w:line=\"276\" w:lineRule=\"auto\"/>\
       </w:pPr></w:pPrDefault></w:docDefaults>\
       <w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
         <w:name w:val=\"Normal\"/>\
         <w:pPr><w:spacing w:after=\"0\" w:line=\"240\" w:lineRule=\"auto\"/></w:pPr>\
         <w:rPr><w:rFonts w:ascii=\"Times New Roman\" w:hAnsi=\"Times New Roman\"/>\
           <w:sz w:val=\"24\"/></w:rPr>\
       </w:style>\
       <w:style w:type=\"paragraph\" w:customStyle=\"1\" w:styleId=\"TextHeading2\">\
         <w:name w:val=\"Text Heading 2\"/>\
         <w:pPr><w:spacing w:before=\"120\" w:after=\"120\" w:line=\"240\" w:lineRule=\"auto\"/>\
           <w:jc w:val=\"both\"/></w:pPr>\
         <w:rPr><w:rFonts w:ascii=\"Times New Roman\" w:hAnsi=\"Times New Roman\"/>\
           <w:sz w:val=\"24\"/></w:rPr>\
       </w:style>\
       <w:style w:type=\"paragraph\" w:customStyle=\"1\" w:styleId=\"TextHeading3\">\
         <w:name w:val=\"Text Heading 3\"/>\
         <w:pPr><w:spacing w:before=\"120\" w:after=\"120\" w:line=\"240\" w:lineRule=\"auto\"/>\
           <w:ind w:left=\"720\" w:right=\"720\"/></w:pPr>\
         <w:rPr><w:rFonts w:ascii=\"Times New Roman\" w:hAnsi=\"Times New Roman\"/>\
           <w:sz w:val=\"24\"/></w:rPr>\
       </w:style>\
     </w:styles>"
}

#[test]
fn text_heading2_before_after_overflows_one_letter_page() {
    // 40 × 12pt Times with after=0 fits on one Letter page (~14pt × 40).
    // TextHeading2 before=120 after=120 adds 6pt+6pt per para → 2pp.
    // sd_2517 has 280+268 of these; missing the pad is 100 vs soffice 107.
    let mut body = String::new();
    for i in 0..40 {
        body.push_str(&format!(
            "<w:p><w:pPr><w:pStyle w:val=\"TextHeading2\"/></w:pPr>\
               <w:r><w:t>Heading {i}</w:t></w:r></w:p>"
        ));
    }
    body.push_str(
        "<w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>",
    );
    let pdf = docx_to_pdf(&numbering_docx_with_styles(
        &body,
        None,
        Some(sd_2517_heading_styles_xml()),
    ))
    .expect("convert TextHeading2 stack");
    assert!(
        pdf_page_count(&pdf) >= 2,
        "40 TextHeading2 paras with before=after=120 must overflow one Letter page, got {}",
        pdf_page_count(&pdf)
    );
}

#[test]
fn text_heading3_right_indent_wraps_a_long_line() {
    // sd_2517 TextHeading3 / Título3 set w:ind left=720 right=720. We
    // honored only left, so 268 long headings wrapped to the full
    // measure (100 vs 107). A ~430pt line fits 468pt but not 396pt.
    let body = "<w:p><w:pPr><w:pStyle w:val=\"TextHeading3\"/></w:pPr>\
         <w:r><w:t>MMMMMMMMMM MMMMMMMMMM MMMMMMMMMM MMMMMMMMMM</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&numbering_docx_with_styles(
        body,
        None,
        Some(sd_2517_heading_styles_xml()),
    ))
    .expect("convert TextHeading3 indent");
    let ys = pdf_tf_ys(&pdf, "12.00 Tf");
    let unique: std::collections::BTreeSet<i32> =
        ys.iter().map(|y| (*y * 2.0).round() as i32).collect();
    assert!(
        unique.len() >= 2,
        "TextHeading3 left+right 0.5in must wrap this line, ys={ys:?}"
    );
}

#[test]
fn sd_2517_matches_oracle_page_count() {
    let bytes = std::fs::read(
        "../neurotic_docx_bench/corpus/word_based/docx_source/sd_2517_localized_heading_styles.docx",
    )
    .expect("sd_2517 fixture");
    let pdf = docx_to_pdf(&bytes).expect("convert sd_2517");
    let n = pdf_page_count(&pdf);
    assert!(
        (106..=107).contains(&n),
        "sd_2517 must match soffice 107 pages (was 100; TextHeading3 right indent), got {n}"
    );
}

#[test]
fn table_cell_paragraphs_stack_as_lines() {
    // sample_document / eigenpal code listing: one cell, many <w:p>. Joining
    // them with a space mashes `import foo import bar` onto one line.
    let body = "<w:tbl><w:tblGrid><w:gridCol w:w=\"8000\"/></w:tblGrid>\
         <w:tr><w:tc>\
           <w:p><w:r><w:t>importfoo</w:t></w:r></w:p>\
           <w:p><w:r><w:t>importbar</w:t></w:r></w:p>\
         </w:tc></w:tr></w:tbl><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert stacked cell");
    let ys = pdf_tf_ys(&pdf, "11.00 Tf");
    let unique: std::collections::BTreeSet<i32> =
        ys.iter().map(|y| (*y * 10.0).round() as i32).collect();
    assert!(
        unique.len() >= 2,
        "two cell paragraphs must paint on two lines, ys={ys:?}"
    );
}

#[test]
fn table_jc_center_is_not_left_stuck() {
    // comments / I_am_sharing / addition* : tblPr w:jc=center on
    // LightShading tables. emit_table pinned every table at margin_l, so
    // 12 centered tables sat on the left while soffice centered them (~40).
    let body = "<w:tbl><w:tblPr><w:tblW w:w=\"3000\" w:type=\"dxa\"/>\
           <w:jc w:val=\"center\"/></w:tblPr>\
           <w:tblGrid><w:gridCol w:w=\"3000\"/></w:tblGrid>\
           <w:tr><w:tc><w:p><w:r><w:t>MidTbl</w:t></w:r></w:p></w:tc></w:tr>\
         </w:tbl>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert centered table");
    let xs = pdf_tf_xs(&pdf, "11.00 Tf");
    assert!(!xs.is_empty(), "MidTbl must paint");
    let min_x = xs.iter().copied().fold(f32::INFINITY, f32::min);
    // content 468pt, table 150pt → left = 72 + 159 = 231.
    assert!(
        min_x > 180.0,
        "tbl jc=center must sit mid-page, not at left=76; xs={xs:?}"
    );
}

#[test]
fn sample_document_stays_three_pages() {
    // Joining cell paras with \\n plus full xml:space padding used to make
    // this 4 vs soffice 3. Collapse_ws stays; only the cell join changes.
    let bytes = std::fs::read(
        "../neurotic_docx_bench/corpus/word_based/docx_source/sample_document_really_repaired_word_repaired.docx",
    )
    .expect("sample_document fixture");
    let pdf = docx_to_pdf(&bytes).expect("convert sample_document");
    assert_eq!(
        pdf_page_count(&pdf),
        3,
        "sample_document must stay 3 pages like soffice, got {}",
        pdf_page_count(&pdf)
    );
}

#[test]
fn tall_table_after_title_stays_on_two_pages() {
    // comments cluster: a title plus a table taller than the leftover
    // page must continue on page 1, not bump the whole table to page 2/3.
    let mut body = String::from(
        "<w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr>\
           <w:r><w:rPr><w:sz w:val=\"60\"/></w:rPr><w:t>TableTitle</w:t></w:r></w:p>\
         <w:tbl><w:tblGrid><w:gridCol w:w=\"9000\"/></w:tblGrid>",
    );
    for i in 0..40 {
        body.push_str(&format!(
            "<w:tr><w:tc><w:p><w:r><w:t>Row {i} lorem ipsum</w:t></w:r></w:p></w:tc></w:tr>"
        ));
    }
    body.push_str("</w:tbl><w:sectPr/>");
    let pdf = docx_to_pdf(&minimal_docx_body(&body)).expect("convert tall table");
    assert_eq!(
        pdf_page_count(&pdf),
        2,
        "split table rows across pages; got {}",
        pdf_page_count(&pdf)
    );
}

#[test]
fn vmerge_gridspan_table_stays_on_one_page() {
    let body = "<w:tbl><w:tblGrid>\
         <w:gridCol w:w=\"2493\"/><w:gridCol w:w=\"2493\"/>\
         <w:gridCol w:w=\"2493\"/><w:gridCol w:w=\"2493\"/></w:tblGrid>\
         <w:tr>\
           <w:tc><w:p><w:r><w:t>1</w:t></w:r></w:p></w:tc>\
           <w:tc><w:p><w:r><w:t>2</w:t></w:r></w:p></w:tc>\
           <w:tc><w:p><w:r><w:t>3</w:t></w:r></w:p></w:tc>\
           <w:tc><w:p><w:r><w:t>4</w:t></w:r></w:p></w:tc></w:tr>\
         <w:tr>\
           <w:tc><w:p><w:r><w:t>5</w:t></w:r></w:p></w:tc>\
           <w:tc><w:tcPr><w:gridSpan w:val=\"2\"/><w:vMerge w:val=\"restart\"/></w:tcPr>\
             <w:p><w:r><w:t>6</w:t></w:r></w:p></w:tc>\
           <w:tc><w:p><w:r><w:t>7</w:t></w:r></w:p></w:tc></w:tr>\
         <w:tr>\
           <w:tc><w:p><w:r><w:t>8</w:t></w:r></w:p></w:tc>\
           <w:tc><w:tcPr><w:gridSpan w:val=\"2\"/><w:vMerge/></w:tcPr>\
             <w:p><w:r><w:t></w:t></w:r></w:p></w:tc>\
           <w:tc><w:p><w:r><w:t>9</w:t></w:r></w:p></w:tc></w:tr>\
         </w:tbl><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert vmerge");
    assert_eq!(pdf_page_count(&pdf), 1);
}

#[test]
fn inline_chart_extent_is_drawn_as_a_box() {
    // 5486400 EMU = 432pt. With 72pt left margin the right edge is 504.
    let body = "<w:p><w:r><w:drawing><wp:inline>\
         <wp:extent cx=\"5486400\" cy=\"3200400\"/>\
         <a:graphic><a:graphicData \
           uri=\"http://schemas.openxmlformats.org/drawingml/2006/chart\"/>\
         </a:graphic></wp:inline></w:drawing></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&drawing_docx(body)).expect("convert chart");
    assert_eq!(pdf_page_count(&pdf), 1);
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("504.00"),
        "432pt chart + 72pt margin ends at 504; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
}

#[test]
fn bar_chart_series_are_filled_rects() {
    let body = "<w:p><w:r><w:drawing><wp:inline>\
         <wp:extent cx=\"5486400\" cy=\"3200400\"/>\
         <a:graphic><a:graphicData \
           uri=\"http://schemas.openxmlformats.org/drawingml/2006/chart\">\
           <c:chart xmlns:c=\"http://schemas.openxmlformats.org/drawingml/2006/chart\" \
             r:id=\"rIdChart\"/>\
         </a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p><w:sectPr/>";
    let chart = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <c:chartSpace xmlns:c=\"http://schemas.openxmlformats.org/drawingml/2006/chart\">\
        <c:chart><c:title><c:tx><c:rich><a:p xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">\
          <a:r><a:t>Chart Title</a:t></a:r></a:p></c:rich></c:tx></c:title>\
        <c:autoTitleDeleted val=\"0\"/>\
        <c:plotArea><c:barChart>\
          <c:ser><c:cat><c:strLit>\
            <c:pt idx=\"0\"><c:v>Category 1</c:v></c:pt>\
            <c:pt idx=\"1\"><c:v>Category 2</c:v></c:pt>\
          </c:strLit></c:cat>\
          <c:val><c:numLit>\
            <c:pt idx=\"0\"><c:v>4</c:v></c:pt>\
            <c:pt idx=\"1\"><c:v>2</c:v></c:pt>\
          </c:numLit></c:val></c:ser>\
        </c:barChart></c:plotArea></c:chart></c:chartSpace>";
    let pdf = docx_to_pdf(&chart_docx(body, chart)).expect("convert bar chart");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains(" re f"),
        "bars must be fill rects; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
    assert!(
        text.contains("14.00 Tf"),
        "chart title is 14pt (Strict01 page-1); tail {}",
        &text[text.len().saturating_sub(280)..]
    );
    assert!(
        text.contains("9.00 Tf"),
        "category / axis labels are 9pt; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
}

#[test]
fn page_field_instruction_is_not_in_the_pdf_stream_as_literals() {
    let body = "<w:p>\
         <w:r><w:t xml:space=\"preserve\">Page </w:t></w:r>\
         <w:r><w:fldChar w:fldCharType=\"begin\"/></w:r>\
         <w:r><w:instrText>PAGE</w:instrText></w:r>\
         <w:r><w:fldChar w:fldCharType=\"separate\"/></w:r>\
         <w:r><w:t>1</w:t></w:r>\
         <w:r><w:fldChar w:fldCharType=\"end\"/></w:r>\
         </w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert field");
    assert_eq!(pdf_page_count(&pdf), 1);
}

#[test]
fn shaded_cell_emits_a_fill_rect() {
    let body = "<w:tbl><w:tblGrid><w:gridCol w:w=\"4000\"/></w:tblGrid>\
         <w:tr><w:tc><w:tcPr><w:shd w:val=\"clear\" w:fill=\"D9EAF7\"/></w:tcPr>\
           <w:p><w:r><w:t>Shaded</w:t></w:r></w:p></w:tc></w:tr>\
         </w:tbl><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert shaded cell");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("0.851 0.918 0.969 rg"),
        "D9EAF7 fill must be painted; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
    assert!(text.contains(" re f"));
}

fn docx_with_styles(body: &str, styles: &str) -> Vec<u8> {
    let document = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
         <w:body>{body}</w:body></w:document>"
    );
    let content_types = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
        <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
        <Default Extension=\"xml\" ContentType=\"application/xml\"/>\
        <Override PartName=\"/word/document.xml\" \
          ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>\
        <Override PartName=\"/word/styles.xml\" \
          ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml\"/>\
        </Types>";
    let rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
        <Relationship Id=\"rId1\" \
          Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" \
          Target=\"word/document.xml\"/>\
        </Relationships>";
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let opts = SimpleFileOptions::default();
    zip.start_file("[Content_Types].xml", opts).unwrap();
    zip.write_all(content_types.as_bytes()).unwrap();
    zip.start_file("_rels/.rels", opts).unwrap();
    zip.write_all(rels.as_bytes()).unwrap();
    zip.start_file("word/document.xml", opts).unwrap();
    zip.write_all(document.as_bytes()).unwrap();
    zip.start_file("word/styles.xml", opts).unwrap();
    zip.write_all(styles.as_bytes()).unwrap();
    zip.finish().unwrap().into_inner()
}

#[test]
fn tbl_style_band_emits_fill_rect() {
    // LightShading-Accent1 band1Horz D3DFEE (docx_lots_of_comments page-1 table).
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
          <w:style w:type=\"table\" w:styleId=\"LightShading-Accent1\">\
            <w:pPr><w:spacing w:after=\"0\" w:line=\"240\" w:lineRule=\"auto\"/></w:pPr>\
            <w:tblPr><w:tblBorders>\
              <w:top w:val=\"single\" w:color=\"4F81BD\"/>\
              <w:bottom w:val=\"single\" w:color=\"4F81BD\"/>\
            </w:tblBorders></w:tblPr>\
            <w:tblStylePr w:type=\"band1Horz\">\
              <w:tcPr><w:shd w:val=\"clear\" w:fill=\"D3DFEE\"/></w:tcPr>\
            </w:tblStylePr>\
          </w:style>\
        </w:styles>";
    let body = "<w:tbl>\
         <w:tblPr><w:tblStyle w:val=\"LightShading-Accent1\"/>\
           <w:tblLook w:firstRow=\"1\" w:noHBand=\"0\"/></w:tblPr>\
         <w:tblGrid><w:gridCol w:w=\"4000\"/></w:tblGrid>\
         <w:tr><w:tc><w:p><w:r><w:t>Header</w:t></w:r></w:p></w:tc></w:tr>\
         <w:tr><w:tc><w:p><w:r><w:t>Banded</w:t></w:r></w:p></w:tc></w:tr>\
         </w:tbl><w:sectPr/>";
    let pdf = docx_to_pdf(&docx_with_styles(body, styles)).expect("convert banded table");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("0.827 0.875 0.933 rg"),
        "D3DFEE band fill must be painted; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
}

#[test]
fn light_shading_bands_first_row_when_firstrow_has_no_fill() {
    // comments / I_am_sharing LightShading-Accent1: firstRow has bold but
    // no shd. soffice still applies band1Horz to row 0 (Prepared for is
    // D3DFEE). Skipping the header for banding inverted every other row.
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
          <w:style w:type=\"table\" w:styleId=\"LightShading-Accent1\">\
            <w:pPr><w:spacing w:after=\"0\" w:line=\"240\" w:lineRule=\"auto\"/></w:pPr>\
            <w:tblStylePr w:type=\"firstRow\"><w:rPr><w:b/></w:rPr></w:tblStylePr>\
            <w:tblStylePr w:type=\"band1Horz\">\
              <w:tcPr><w:shd w:val=\"clear\" w:fill=\"D3DFEE\"/></w:tcPr>\
            </w:tblStylePr>\
          </w:style>\
        </w:styles>";
    let body = "<w:tbl>\
         <w:tblPr><w:tblStyle w:val=\"LightShading-Accent1\"/>\
           <w:tblLook w:firstRow=\"1\" w:noHBand=\"0\"/></w:tblPr>\
         <w:tblGrid><w:gridCol w:w=\"4000\"/></w:tblGrid>\
         <w:tr><w:tc><w:p><w:r><w:t>R0</w:t></w:r></w:p></w:tc></w:tr>\
         <w:tr><w:tc><w:p><w:r><w:t>R1</w:t></w:r></w:p></w:tc></w:tr>\
         <w:tr><w:tc><w:p><w:r><w:t>R2</w:t></w:r></w:p></w:tc></w:tr>\
         </w:tbl><w:sectPr/>";
    let pdf = docx_to_pdf(&docx_with_styles(body, styles)).expect("convert light shading");
    let text = String::from_utf8_lossy(&pdf);
    let n = text.matches("0.827 0.875 0.933 rg").count();
    assert_eq!(
        n,
        2,
        "rows 0 and 2 must paint band1 D3DFEE (soffice), not only row 1; n={n} tail {}",
        &text[text.len().saturating_sub(280)..]
    );
}

#[test]
fn cli_convert_twice_writes_openable_pdfs() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("in.docx");
    std::fs::copy(FIXTURE, &input).unwrap();
    for name in ["out1.pdf", "out2.pdf"] {
        let output = dir.path().join(name);
        let ran = Command::new(BIN)
            .args(["convert"])
            .arg(&input)
            .args(["-o"])
            .arg(&output)
            .arg("--force")
            .output()
            .unwrap();
        assert!(
            ran.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&ran.stderr)
        );
        let pdf = std::fs::read(&output).unwrap();
        assert!(pdf.starts_with(b"%PDF"), "{name} missing %PDF header");
        assert!(pdf_page_count(&pdf) >= 1, "{name} has no pages");
        assert!(
            String::from_utf8_lossy(&ran.stdout).contains(name),
            "stdout: {}",
            String::from_utf8_lossy(&ran.stdout)
        );
    }
}

#[test]
fn cli_convert_refuses_overwrite_without_force() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("in.docx");
    let output = dir.path().join("out.pdf");
    std::fs::copy(FIXTURE, &input).unwrap();
    let first = Command::new(BIN)
        .args(["convert"])
        .arg(&input)
        .args(["-o"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(first.status.success());
    let second = Command::new(BIN)
        .args(["convert"])
        .arg(&input)
        .args(["-o"])
        .arg(&output)
        .output()
        .unwrap();
    assert_eq!(second.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&second.stderr).contains("already exists"));
}

#[test]
fn convert_help_mentions_pdf() {
    let out = Command::new(BIN)
        .args(["convert", "--help"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.to_ascii_lowercase().contains("pdf"));
}

#[test]
fn default_output_is_sibling_pdf() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("memo.docx");
    std::fs::copy(FIXTURE, &input).unwrap();
    let ran = Command::new(BIN)
        .args(["convert"])
        .arg(&input)
        .output()
        .unwrap();
    assert!(
        ran.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&ran.stderr)
    );
    assert!(dir.path().join("memo.pdf").is_file());
}

#[test]
fn header_pbdr_bottom_paints_a_rule() {
    // sample_document: header pBdr bottom E2E8F0. Use red so the PDF stream
    // is an exact RGB we can search for.
    let header = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:hdr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:p><w:pPr><w:pBdr>\
             <w:bottom w:val=\"single\" w:sz=\"12\" w:space=\"1\" w:color=\"FF0000\"/>\
           </w:pBdr></w:pPr>\
           <w:r><w:t>HeadRule</w:t></w:r></w:p></w:hdr>";
    let body = "<w:p><w:r><w:t>Body</w:t></w:r></w:p>\
         <w:sectPr><w:headerReference w:type=\"default\" r:id=\"rIdH1\"/>\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\" \
             w:header=\"720\"/></w:sectPr>";
    let pdf = docx_to_pdf(&hf_docx(
        body,
        &[("rIdH1", "header", "header1.xml")],
        &[("word/header1.xml", header.to_string())],
    ))
    .expect("convert header rule");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("1.000 0.000 0.000 RG"),
        "header pBdr bottom must stroke red; tail {}",
        &text[text.len().saturating_sub(400)..]
    );
}

#[test]
fn body_pbdr_bottom_paints_a_rule() {
    // sample_document / eigenpal: 14 heading paras carry
    // `<w:pBdr><w:bottom … color="E2E8F0"/>`. Header chrome already
    // strokes pBdr; body emit_runs ignored it, so the 8-stem cluster
    // sits at ~39 with matching 3pp. Red so the stream is searchable.
    let body = "<w:p><w:pPr><w:pBdr>\
           <w:bottom w:val=\"single\" w:sz=\"12\" w:space=\"4\" w:color=\"FF0000\"/>\
         </w:pBdr></w:pPr>\
         <w:r><w:t>HeadingRule</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert body pBdr");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("1.000 0.000 0.000 RG"),
        "body pBdr bottom must stroke red; tail {}",
        &text[text.len().saturating_sub(400)..]
    );
}

#[test]
fn numpages_field_uses_real_page_count_not_cached_result() {
    // sample_document footer caches NUMPAGES as "9"; soffice paints the
    // real count. Body has no digits so a "2" glyph can only come from the field.
    let footer = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:ftr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:p>\
             <w:r><w:fldChar w:fldCharType=\"begin\"/></w:r>\
             <w:r><w:instrText>NUMPAGES</w:instrText></w:r>\
             <w:r><w:fldChar w:fldCharType=\"separate\"/></w:r>\
             <w:r><w:t>9</w:t></w:r>\
             <w:r><w:fldChar w:fldCharType=\"end\"/></w:r>\
           </w:p></w:ftr>";
    let body = "<w:p><w:r><w:t>Alpha</w:t></w:r></w:p>\
         <w:p><w:r><w:br w:type=\"page\"/></w:r></w:p>\
         <w:p><w:r><w:t>Beta</w:t></w:r></w:p>\
         <w:sectPr><w:footerReference w:type=\"default\" r:id=\"rIdF1\"/>\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\" \
             w:footer=\"720\"/></w:sectPr>";
    let pdf = docx_to_pdf(&hf_docx(
        body,
        &[("rIdF1", "footer", "footer1.xml")],
        &[("word/footer1.xml", footer.to_string())],
    ))
    .expect("convert numpages");
    assert_eq!(pdf_page_count(&pdf), 2);
    let two = docx_to_pdf(&minimal_docx_body(
        "<w:p><w:r><w:t>2</w:t></w:r></w:p><w:sectPr/>",
    ))
    .expect("glyph oracle");
    let hex2 = tj_hex(&two).expect("2 glyph");
    let nine = docx_to_pdf(&minimal_docx_body(
        "<w:p><w:r><w:t>9</w:t></w:r></w:p><w:sectPr/>",
    ))
    .expect("glyph 9");
    let hex9 = tj_hex(&nine).expect("9 glyph");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains(&hex2),
        "NUMPAGES must paint real count 2 (hex {hex2}); cached 9 still? {}",
        text.contains(&hex9)
    );
}

#[test]
fn numpages_on_serif_face_is_still_patched() {
    // sample/eigenpal footer is Inter → Liberation Serif. chrome() writes
    // @@N@@ in that face, but patch_numpages only compared Carlito
    // Regular glyph ids, so the mark leaked on the 8-stem cluster.
    let rpr = "<w:rPr><w:rFonts w:ascii=\"Inter\" w:hAnsi=\"Inter\"/>\
         <w:sz w:val=\"22\"/></w:rPr>";
    let footer = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:ftr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:p>\
             <w:r><w:fldChar w:fldCharType=\"begin\"/></w:r>\
             <w:r><w:instrText>NUMPAGES</w:instrText></w:r>\
             <w:r><w:fldChar w:fldCharType=\"separate\"/></w:r>\
             <w:r>{rpr}<w:t>9</w:t></w:r>\
             <w:r><w:fldChar w:fldCharType=\"end\"/></w:r>\
           </w:p></w:ftr>"
    );
    let body = "<w:p><w:r><w:t>Alpha</w:t></w:r></w:p>\
         <w:p><w:r><w:br w:type=\"page\"/></w:r></w:p>\
         <w:p><w:r><w:t>Beta</w:t></w:r></w:p>\
         <w:sectPr><w:footerReference w:type=\"default\" r:id=\"rIdF1\"/>\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\" \
             w:footer=\"720\"/></w:sectPr>";
    let pdf = docx_to_pdf(&hf_docx(
        body,
        &[("rIdF1", "footer", "footer1.xml")],
        &[("word/footer1.xml", footer)],
    ))
    .expect("convert serif numpages");
    assert_eq!(pdf_page_count(&pdf), 2);
    let run = |t: &str| {
        format!(
            "<w:p><w:r><w:rPr><w:rFonts w:ascii=\"Inter\" w:hAnsi=\"Inter\"/>\
               <w:sz w:val=\"22\"/></w:rPr><w:t>{t}</w:t></w:r></w:p><w:sectPr/>"
        )
    };
    let hex2 = tj_hex(&docx_to_pdf(&minimal_docx_body(&run("2"))).expect("serif 2"))
        .expect("serif 2 glyph");
    let hex_mark = tj_hex(&docx_to_pdf(&minimal_docx_body(&run("@@N@@"))).expect("serif mark"))
        .expect("serif mark glyph");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains(&hex2),
        "Inter NUMPAGES must paint 2 in the serif face (hex {hex2})"
    );
    assert!(
        !text.contains(&hex_mark),
        "@@N@@ must not leak when the footer face is not Carlito"
    );
}

fn tj_hex(pdf: &[u8]) -> Option<String> {
    let hay = String::from_utf8_lossy(pdf);
    let mut from = 0;
    while let Some(rel) = hay[from..].find(" Tj") {
        let slice = &hay[..from + rel];
        if let Some(lt) = slice.rfind('<')
            && let Some(gt) = slice[lt + 1..].find('>')
        {
            let hex = &slice[lt + 1..lt + 1 + gt];
            if hex.len() >= 4 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
                return Some(hex.to_string());
            }
        }
        from += rel + 3;
    }
    None
}

fn hf_part(tag: &str, half_points: u32, text: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:{tag} xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:p><w:r><w:rPr><w:sz w:val=\"{half_points}\"/></w:rPr>\
             <w:t>{text}</w:t></w:r></w:p></w:{tag}>"
    )
}

fn hf_docx(body: &str, rels: &[(&str, &str, &str)], parts: &[(&str, String)]) -> Vec<u8> {
    let document = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" \
           xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">\
         <w:body>{body}</w:body></w:document>"
    );
    let mut types = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
         <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
         <Default Extension=\"xml\" ContentType=\"application/xml\"/>\
         <Override PartName=\"/word/document.xml\" \
           ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>",
    );
    for (name, _) in parts {
        let kind = if name.contains("header") {
            "header"
        } else {
            "footer"
        };
        types.push_str(&format!(
            "<Override PartName=\"/{name}\" \
               ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.{kind}+xml\"/>"
        ));
    }
    types.push_str("</Types>");
    let pkg_rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
        <Relationship Id=\"rId1\" \
          Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" \
          Target=\"word/document.xml\"/>\
        </Relationships>";
    let mut doc_rels = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">",
    );
    for (id, kind, target) in rels {
        doc_rels.push_str(&format!(
            "<Relationship Id=\"{id}\" \
               Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/{kind}\" \
               Target=\"{target}\"/>"
        ));
    }
    doc_rels.push_str("</Relationships>");
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let opts = SimpleFileOptions::default();
    zip.start_file("[Content_Types].xml", opts).unwrap();
    zip.write_all(types.as_bytes()).unwrap();
    zip.start_file("_rels/.rels", opts).unwrap();
    zip.write_all(pkg_rels.as_bytes()).unwrap();
    zip.start_file("word/document.xml", opts).unwrap();
    zip.write_all(document.as_bytes()).unwrap();
    zip.start_file("word/_rels/document.xml.rels", opts)
        .unwrap();
    zip.write_all(doc_rels.as_bytes()).unwrap();
    for (name, xml) in parts {
        zip.start_file(*name, opts).unwrap();
        zip.write_all(xml.as_bytes()).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

#[test]
fn first_section_without_hf_refs_ignores_orphan_footer_parts() {
    // sd_2517: first sectPr has no footerReference; leftover footerN.xml
    // parts must not be concatenated into chrome.
    let body = "<w:p><w:r><w:t>Title page</w:t></w:r></w:p>\
         <w:p><w:pPr><w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr></w:pPr></w:p>\
         <w:p><w:r><w:t>Later section</w:t></w:r></w:p>\
         <w:sectPr><w:footerReference w:type=\"default\" r:id=\"rIdF1\"/>\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr>";
    let docx = hf_docx(
        body,
        &[
            ("rIdF1", "footer", "footer1.xml"),
            ("rIdF2", "footer", "footer2.xml"),
        ],
        &[
            ("word/footer1.xml", hf_part("ftr", 28, "LaterOnlyFooter")),
            ("word/footer2.xml", hf_part("ftr", 42, "OrphanFooter")),
        ],
    );
    let pdf = docx_to_pdf(&docx).expect("convert");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        !text.contains("21.00 Tf"),
        "unreferenced 21pt footers must not paint; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
}

#[test]
fn first_section_footer_ref_does_not_concatenate_other_footer_parts() {
    let body = "<w:p><w:r><w:t>Body</w:t></w:r></w:p>\
         <w:p><w:pPr><w:sectPr>\
           <w:footerReference w:type=\"default\" r:id=\"rIdF1\"/>\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr></w:pPr></w:p>\
         <w:p><w:r><w:t>Section two</w:t></w:r></w:p>\
         <w:sectPr><w:footerReference w:type=\"default\" r:id=\"rIdF2\"/>\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr>";
    let docx = hf_docx(
        body,
        &[
            ("rIdF1", "footer", "footer1.xml"),
            ("rIdF2", "footer", "footer2.xml"),
            ("rIdH1", "header", "header1.xml"),
            ("rIdH2", "header", "header2.xml"),
        ],
        &[
            ("word/footer1.xml", hf_part("ftr", 28, "LiveFooter")),
            ("word/footer2.xml", hf_part("ftr", 28, "LaterFooter")),
            ("word/header1.xml", hf_part("hdr", 28, "LiveHeader")),
            ("word/header2.xml", hf_part("hdr", 42, "JunkHeader")),
        ],
    );
    let pdf = docx_to_pdf(&docx).expect("convert");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("14.00 Tf"),
        "first-section 14pt footer must paint; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
    assert!(
        !text.contains("21.00 Tf"),
        "later-section / orphan 21pt chrome must not concatenate; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
}

fn pdf_rgb_rule_widths(pdf: &[u8], r: f32, g: f32, b: f32) -> Vec<f32> {
    let needle = format!("{r:.3} {g:.3} {b:.3} RG");
    let hay = String::from_utf8_lossy(pdf);
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(rel) = hay[from..].find(&needle) {
        let rest = &hay[from + rel + needle.len()..];
        let end = rest.find(" l S").unwrap_or(rest.len());
        let line = &rest[..end];
        let parts: Vec<&str> = line.split_whitespace().collect();
        if let Some(mi) = parts.iter().position(|t| *t == "m")
            && mi >= 2
            && mi + 2 < parts.len()
            && let (Ok(x1), Ok(x2)) = (parts[mi - 2].parse::<f32>(), parts[mi + 1].parse::<f32>())
        {
            out.push((x2 - x1).abs());
        }
        from += rel + needle.len();
    }
    out
}

fn pdf_vertical_rule_xs(pdf: &[u8]) -> Vec<f32> {
    let hay = String::from_utf8_lossy(pdf);
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(rel) = hay[from..].find(" l S\n") {
        let start = hay[..from + rel].rfind('\n').map_or(0, |i| i + 1);
        let line = &hay[start..from + rel];
        let parts: Vec<&str> = line.split_whitespace().collect();
        if let Some(mi) = parts.iter().position(|t| *t == "m")
            && mi >= 2
            && mi + 2 < parts.len()
            && let (Ok(x1), Ok(y1), Ok(x2), Ok(y2)) = (
                parts[mi - 2].parse::<f32>(),
                parts[mi - 1].parse::<f32>(),
                parts[mi + 1].parse::<f32>(),
                parts[mi + 2].parse::<f32>(),
            )
            && (x1 - x2).abs() < 0.4
            && (y1 - y2).abs() > 6.0
        {
            out.push(x1);
        }
        from += rel + 4;
    }
    out
}

fn pdf_horiz_rule_ys(pdf: &[u8]) -> Vec<f32> {
    // `0.50 w 0.000 0.000 0.000 RG x1 y1 m x2 y2 l S`
    let hay = String::from_utf8_lossy(pdf);
    let mut raw = Vec::new();
    let mut from = 0;
    while let Some(rel) = hay[from..].find(" l S\n") {
        let start = hay[..from + rel].rfind('\n').map_or(0, |i| i + 1);
        let line = &hay[start..from + rel];
        let parts: Vec<&str> = line.split_whitespace().collect();
        if let Some(mi) = parts.iter().position(|t| *t == "m")
            && mi >= 2
            && mi + 2 < parts.len()
            && let (Ok(x1), Ok(y1), Ok(x2), Ok(y2)) = (
                parts[mi - 2].parse::<f32>(),
                parts[mi - 1].parse::<f32>(),
                parts[mi + 1].parse::<f32>(),
                parts[mi + 2].parse::<f32>(),
            )
            && (y1 - y2).abs() < 0.2
            && (x2 - x1).abs() > 40.0
        {
            raw.push(y1);
        }
        from += rel + 4;
    }
    raw.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    let mut ys = Vec::new();
    for y in raw {
        if ys.last().is_none_or(|prev: &f32| (*prev - y).abs() > 0.4) {
            ys.push(y);
        }
    }
    ys
}

fn pdf_tf_ys(pdf: &[u8], tf: &str) -> Vec<f32> {
    let hay = String::from_utf8_lossy(pdf);
    let mut ys = Vec::new();
    let mut from = 0;
    while let Some(rel) = hay[from..].find(tf) {
        let slice = &hay[from + rel..from + rel + tf.len() + 80.min(hay.len() - from - rel)];
        if let Some(td) = slice.find(" Td") {
            let before = &slice[..td];
            let mut parts = before.rsplit([' ', '\n']);
            let y_s = parts.next();
            let _x_s = parts.next();
            if let Some(y) = y_s.and_then(|s| s.parse::<f32>().ok()) {
                ys.push(y);
            }
        }
        from += rel + tf.len();
    }
    ys
}

#[test]
fn title_30pt_center_sits_below_header_band() {
    // comments / Word-vs-Docs cluster: 30pt centered title must not share the
    // header baseline (pgMar top == header). Word keeps body under the header.
    let body = "<w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr>\
           <w:r><w:rPr><w:sz w:val=\"60\"/></w:rPr><w:t>TitleThirty</w:t></w:r></w:p>\
         <w:sectPr>\
           <w:headerReference w:type=\"default\" r:id=\"rIdH1\"/>\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"720\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\" \
             w:header=\"720\" w:footer=\"720\"/></w:sectPr>";
    let docx = hf_docx(
        body,
        &[("rIdH1", "header", "header1.xml")],
        &[("word/header1.xml", hf_part("hdr", 22, "HdrBand"))],
    );
    let pdf = docx_to_pdf(&docx).expect("convert");
    let header_ys = pdf_tf_ys(&pdf, "11.00 Tf");
    let title_ys = pdf_tf_ys(&pdf, "30.00 Tf");
    assert!(
        !header_ys.is_empty(),
        "11pt header must paint; {}",
        String::from_utf8_lossy(&pdf)
            .lines()
            .rev()
            .take(8)
            .collect::<Vec<_>>()
            .join(" | ")
    );
    assert!(
        !title_ys.is_empty(),
        "30pt title must paint; header_ys={header_ys:?}"
    );
    let header_y = header_ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let title_y = title_ys.iter().copied().fold(f32::INFINITY, f32::min);
    assert!(
        title_y + 24.0 < header_y,
        "30pt title baseline {title_y} must sit a full title ascent below header {header_y}"
    );
    let title_xs = {
        let hay = String::from_utf8_lossy(&pdf);
        let mut xs = Vec::new();
        let mut from = 0;
        while let Some(rel) = hay[from..].find("30.00 Tf") {
            let slice = &hay[from + rel..from + rel + 100.min(hay.len() - from - rel)];
            if let Some(td) = slice.find(" Td") {
                let before = &slice[..td];
                let mut parts = before.rsplit([' ', '\n']);
                let _y = parts.next();
                if let Some(x) = parts.next().and_then(|s| s.parse::<f32>().ok()) {
                    xs.push(x);
                }
            }
            from += rel + 8;
        }
        xs
    };
    let min_x = title_xs.iter().copied().fold(f32::INFINITY, f32::min);
    assert!(
        min_x > 90.0,
        "30pt title stays centered, not left-stuck; min_x={min_x} xs={title_xs:?}"
    );
}

#[test]
fn comments_pgmar_starts_body_at_top_margin() {
    // comments / I_am_sharing: top=936 twips, header=720. The header line
    // sits in that 10.8pt gap. Word starts the body at top margin — adding
    // header_band on top of w:header shrinks every page and spills +1.
    let body = "<w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr>\
           <w:r><w:rPr><w:sz w:val=\"60\"/></w:rPr><w:t>TitleThirty</w:t></w:r></w:p>\
         <w:sectPr>\
           <w:headerReference w:type=\"default\" r:id=\"rIdH1\"/>\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"936\" w:right=\"1080\" w:bottom=\"936\" w:left=\"1080\" \
             w:header=\"720\" w:footer=\"720\"/></w:sectPr>";
    let docx = hf_docx(
        body,
        &[("rIdH1", "header", "header1.xml")],
        &[("word/header1.xml", hf_part("hdr", 22, "HdrBand"))],
    );
    let pdf = docx_to_pdf(&docx).expect("convert");
    let title_y = pdf_tf_ys(&pdf, "30.00 Tf")
        .into_iter()
        .fold(f32::INFINITY, f32::min);
    // Letter 792, top 46.8pt, 30pt ascent ≈ 28pt → baseline ≈ 717.
    // header+line (~49pt) would land the title near 714.
    assert!(
        title_y > 716.0,
        "body must start at top=936 twips, not header+band; title_y={title_y}"
    );
}

fn pdf_tf_xs(pdf: &[u8], tf: &str) -> Vec<f32> {
    let hay = String::from_utf8_lossy(pdf);
    let mut xs = Vec::new();
    let mut from = 0;
    while let Some(rel) = hay[from..].find(tf) {
        let slice = &hay[from + rel..from + rel + tf.len() + 80.min(hay.len() - from - rel)];
        if let Some(td) = slice.find(" Td") {
            let before = &slice[..td];
            let mut parts = before.rsplit([' ', '\n']);
            let _y = parts.next();
            if let Some(x) = parts.next().and_then(|s| s.parse::<f32>().ok()) {
                xs.push(x);
            }
        }
        from += rel + tf.len();
    }
    xs
}

#[test]
fn first_section_left_margin_beats_last_section() {
    // sd_2517: first sectPr is 2160 twips L/R; last is 1800. Geometry comes
    // from the first section, not the trailing body sectPr.
    let body = "<w:p><w:r><w:t>Edge</w:t></w:r></w:p>\
         <w:p><w:pPr><w:sectPr>\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"2880\" \
             w:header=\"720\" w:footer=\"720\"/></w:sectPr></w:pPr></w:p>\
         <w:p><w:r><w:t>Later</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\" \
             w:header=\"720\" w:footer=\"720\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert");
    let xs = pdf_tf_xs(&pdf, "11.00 Tf");
    assert!(
        xs.iter().any(|&x| x > 130.0),
        "first-section left=2880 twips (144pt) must appear; xs={xs:?}"
    );
}

#[test]
fn later_section_uses_its_margin_and_footer() {
    // After a nextPage sectPr, later pages take that section's pgMar + footer
    // (sd_2517 body sections are 1800-twip with their own footer).
    let body = "<w:p><w:r><w:t>One</w:t></w:r></w:p>\
         <w:p><w:pPr><w:sectPr><w:type w:val=\"nextPage\"/>\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"2880\" \
             w:header=\"720\" w:footer=\"720\"/></w:sectPr></w:pPr></w:p>\
         <w:p><w:r><w:t>Two</w:t></w:r></w:p>\
         <w:sectPr><w:footerReference w:type=\"default\" r:id=\"rIdF1\"/>\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\" \
             w:header=\"720\" w:footer=\"720\"/></w:sectPr>";
    let docx = hf_docx(
        body,
        &[("rIdF1", "footer", "footer1.xml")],
        &[("word/footer1.xml", hf_part("ftr", 28, "LaterFooter"))],
    );
    let pdf = docx_to_pdf(&docx).expect("convert");
    assert_eq!(pdf_page_count(&pdf), 2);
    let xs = pdf_tf_xs(&pdf, "11.00 Tf");
    assert!(
        xs.iter().any(|&x| x > 130.0),
        "section 1 at 144pt; xs={xs:?}"
    );
    assert!(xs.iter().any(|&x| x < 90.0), "section 2 at 72pt; xs={xs:?}");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("14.00 Tf"),
        "later-section 14pt footer must paint"
    );
}

fn pdf_mediaboxes(pdf: &[u8]) -> Vec<(f32, f32)> {
    let hay = String::from_utf8_lossy(pdf);
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(rel) = hay[from..].find("/MediaBox [") {
        let rest = &hay[from + rel + "/MediaBox [".len()..];
        let Some(end) = rest.find(']') else {
            break;
        };
        let nums: Vec<f32> = rest[..end]
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();
        if nums.len() == 4 {
            out.push((nums[2], nums[3]));
        }
        from += rel + 11;
    }
    out
}

#[test]
fn sectpr_last_in_sdt_still_switches_to_landscape() {
    // Cover-page SDTs often end with the section's sectPr. Looking only at
    // siblings inside the SDT used to treat that as the document-final
    // sectPr and never apply the following landscape page.
    let docx = minimal_docx_body(
        "<w:p><w:r><w:t>Portrait</w:t></w:r></w:p>\
         <w:sdt><w:sdtContent>\
           <w:p><w:pPr><w:sectPr>\
             <w:pgSz w:w=\"612pt\" w:h=\"792pt\"/></w:sectPr></w:pPr></w:p>\
         </w:sdtContent></w:sdt>\
         <w:p><w:r><w:t>Landscape</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"792pt\" w:h=\"612pt\" w:orient=\"landscape\"/></w:sectPr>",
    );
    let pdf = docx_to_pdf(&docx).expect("convert sdt sectPr");
    let boxes = pdf_mediaboxes(&pdf);
    assert_eq!(pdf_page_count(&pdf), 2, "SDT-final sectPr is still a break");
    assert!(
        boxes
            .iter()
            .any(|&(w, h)| (w - 792.0).abs() < 1.0 && (h - 612.0).abs() < 1.0),
        "following section must be landscape; {boxes:?}"
    );
}

#[test]
fn overflow_before_landscape_keeps_a_portrait_page() {
    // Strict01: the last portrait lines spill onto a new page, then a
    // landscape sectPr arrives. new_page() used to clear page_has_body so
    // the leftover page was retagged 792×612 (pairing starts at page 3).
    let mut body = String::new();
    for i in 0..55 {
        body.push_str(&format!(
            "<w:p><w:pPr><w:spacing w:after=\"200\" w:line=\"276\" w:lineRule=\"auto\"/></w:pPr>\
               <w:r><w:t>Fill {i} xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx</w:t></w:r></w:p>"
        ));
    }
    body.push_str(
        "<w:p><w:pPr><w:sectPr><w:pgSz w:w=\"612pt\" w:h=\"792pt\"/></w:sectPr></w:pPr>\
           <w:r><w:t>LastPortrait</w:t></w:r></w:p>\
         <w:p><w:r><w:t>LandscapeStart</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"792pt\" w:h=\"612pt\" w:orient=\"landscape\"/></w:sectPr>",
    );
    let pdf = docx_to_pdf(&minimal_docx_body(&body)).expect("convert overflow section");
    let boxes = pdf_mediaboxes(&pdf);
    assert!(
        boxes.len() >= 3,
        "spill + section break must keep the leftover portrait page; boxes={boxes:?}"
    );
    assert!(
        (boxes[0].0 - 612.0).abs() < 2.0 && (boxes[0].1 - 792.0).abs() < 2.0,
        "page 1 portrait; {boxes:?}"
    );
    assert!(
        (boxes[1].0 - 612.0).abs() < 2.0 && (boxes[1].1 - 792.0).abs() < 2.0,
        "leftover page stays portrait; {boxes:?}"
    );
}

#[test]
fn empty_nextpage_section_keeps_a_blank_page() {
    // Strict01: landscape sectPr, then an empty nextPage portrait sectPr,
    // then more body. Word keeps the empty section as its own page. We
    // applied the following section onto that empty page (10 vs 13).
    let docx = minimal_docx_body(
        "<w:p><w:r><w:t>PortraitOne</w:t></w:r></w:p>\
         <w:p><w:pPr><w:sectPr>\
           <w:pgSz w:w=\"612pt\" w:h=\"792pt\"/></w:sectPr></w:pPr></w:p>\
         <w:p><w:r><w:t>LandscapeBody</w:t></w:r></w:p>\
         <w:p><w:pPr><w:sectPr>\
           <w:pgSz w:w=\"792pt\" w:h=\"612pt\" w:orient=\"landscape\"/></w:sectPr></w:pPr></w:p>\
         <w:p><w:pPr><w:sectPr>\
           <w:pgSz w:w=\"612pt\" w:h=\"792pt\"/></w:sectPr></w:pPr></w:p>\
         <w:p><w:r><w:t>PortraitTwo</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"612pt\" w:h=\"792pt\"/></w:sectPr>",
    );
    let pdf = docx_to_pdf(&docx).expect("convert empty section");
    let boxes = pdf_mediaboxes(&pdf);
    assert_eq!(
        pdf_page_count(&pdf),
        4,
        "empty nextPage section is a blank page, not collapsed; boxes={boxes:?}"
    );
    assert!(
        boxes
            .iter()
            .any(|&(w, h)| (w - 792.0).abs() < 1.0 && (h - 612.0).abs() < 1.0),
        "landscape section must still emit 792×612; {boxes:?}"
    );
}

#[test]
fn strict01_matches_oracle_page_pairing() {
    // soffice Strict01 is 13 pages: 3 portrait, 6 landscape, 4 portrait.
    // Collapsing the empty post-landscape section plus missing breaks
    // produced 10 pages and unpaired MediaBoxes (score ~31).
    let docx = std::fs::read("tests/fixtures/strict/Strict01.docx")
        .expect("tests/fixtures/strict/Strict01.docx");
    let pdf = docx_to_pdf(&docx).expect("convert Strict01");
    let boxes = pdf_mediaboxes(&pdf);
    let n = pdf_page_count(&pdf);
    assert!(
        n >= 12,
        "Strict01 was 10pp with landscape on page 3; soffice is 13; boxes={boxes:?}"
    );
    assert!(
        boxes.len() >= 3
            && boxes[..3]
                .iter()
                .all(|&(w, h)| (w - 612.0).abs() < 2.0 && (h - 792.0).abs() < 2.0),
        "first three pages stay portrait; {boxes:?}"
    );
    let land = boxes.iter().filter(|&&(w, h)| w > h + 10.0).count();
    assert!(
        land >= 6,
        "mid-doc landscape section is six 792×612 pages; land={land} boxes={boxes:?}"
    );
}

#[test]
fn later_section_landscape_writes_its_mediabox() {
    // Strict01 cluster: first sectPr is letter portrait; the next is
    // 792pt×612pt landscape (type omitted = nextPage). Each PDF page must
    // keep that section's MediaBox — a single last-section box scores ~32.
    let docx = minimal_docx_body(
        "<w:p><w:r><w:t>Portrait</w:t></w:r></w:p>\
         <w:p><w:pPr><w:sectPr>\
           <w:pgSz w:w=\"612pt\" w:h=\"792pt\"/></w:sectPr></w:pPr></w:p>\
         <w:p><w:r><w:t>Landscape</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"792pt\" w:h=\"612pt\" w:orient=\"landscape\"/></w:sectPr>",
    );
    let pdf = docx_to_pdf(&docx).expect("convert landscape section");
    let boxes = pdf_mediaboxes(&pdf);
    assert_eq!(pdf_page_count(&pdf), 2, "omitted sect type is nextPage");
    assert_eq!(boxes.len(), 2, "one MediaBox per page; {boxes:?}");
    assert!(
        (boxes[0].0 - 612.0).abs() < 1.0 && (boxes[0].1 - 792.0).abs() < 1.0,
        "page 1 stays portrait 612×792; {boxes:?}"
    );
    assert!(
        (boxes[1].0 - 792.0).abs() < 1.0 && (boxes[1].1 - 612.0).abs() < 1.0,
        "page 2 must be landscape 792×612; {boxes:?}"
    );
}

#[test]
fn strict01_fixture_emits_a_landscape_page() {
    let docx = std::fs::read("tests/fixtures/strict/Strict01.docx")
        .expect("tests/fixtures/strict/Strict01.docx");
    let pdf = docx_to_pdf(&docx).expect("convert Strict01");
    let boxes = pdf_mediaboxes(&pdf);
    assert!(
        boxes.iter().any(|&(w, h)| w > h + 10.0),
        "Strict01 mid-doc landscape sectPr must emit a 792×612 page; boxes={boxes:?}"
    );
}

#[test]
fn first_section_valign_center_moves_short_title() {
    let body = "<w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr>\
           <w:r><w:rPr><w:sz w:val=\"60\"/></w:rPr><w:t>MidTitle</w:t></w:r></w:p>\
         <w:sectPr><w:vAlign w:val=\"center\"/>\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert");
    let ys = pdf_tf_ys(&pdf, "30.00 Tf");
    assert!(!ys.is_empty(), "30pt title must paint");
    let title_y = ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        title_y < 550.0,
        "vAlign=center must drop a short title off the top band; y={title_y}"
    );
}

#[test]
fn space_before_is_dropped_at_top_of_a_new_page() {
    // Word/soffice suppress w:spacing before at the top of a page.
    // Heading1 in the comments cluster is before=480 twips; applying it
    // after a page break stacks ~24pt × N headings and spills a page.
    let body = "<w:p><w:r><w:t>FirstPage</w:t></w:r></w:p>\
         <w:p><w:r><w:br w:type=\"page\"/></w:r></w:p>\
         <w:p><w:pPr><w:spacing w:before=\"480\"/></w:pPr>\
           <w:r><w:rPr><w:sz w:val=\"22\"/></w:rPr><w:t>TopHead</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert");
    let ys = pdf_tf_ys(&pdf, "11.00 Tf");
    assert!(ys.len() >= 2, "both pages must paint; ys={ys:?}");
    let min_y = ys.iter().copied().fold(f32::INFINITY, f32::min);
    // Letter 792, top margin 72: first baseline ~ 792-72-ascent ≈ 705.
    // A 24pt before would push the new-page heading below 690.
    assert!(
        min_y > 698.0,
        "space-before must not push a page-top heading down; min_y={min_y} ys={ys:?}"
    );
}

#[test]
fn contextual_spacing_keeps_same_style_list_on_one_page() {
    // comments / I_am_sharing cluster: ListBullet+ListNumber set
    // w:contextualSpacing. Soffice drops after=200 between adjacent
    // same-style items; we were stacking 10pt × ~48 items → +1 page.
    let mut body = String::new();
    for i in 0..40 {
        body.push_str(&format!(
            "<w:p><w:pPr><w:spacing w:after=\"200\" w:line=\"276\"/>\
             <w:contextualSpacing/></w:pPr>\
             <w:r><w:t>Item {i}</w:t></w:r></w:p>"
        ));
    }
    body.push_str(
        "<w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
         <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>",
    );
    let pdf = docx_to_pdf(&minimal_docx_body(&body)).expect("convert contextual list");
    assert_eq!(
        pdf_page_count(&pdf),
        1,
        "40 same-style contextualSpacing items must stay on one Letter page, got {}",
        pdf_page_count(&pdf)
    );
}

#[test]
fn table_tr_height_exact_does_not_add_cell_pad() {
    // Letter + 1" margins → 648pt body. 30 × exact 360-twip (18pt) = 540pt.
    // Content+8pt pad would spill this onto page 2.
    let mut rows = String::new();
    for i in 0..30 {
        rows.push_str(&format!(
            "<w:tr><w:trPr><w:trHeight w:val=\"360\" w:hRule=\"exact\"/></w:trPr>\
             <w:tc><w:p><w:r><w:t>R{i}</w:t></w:r></w:p></w:tc></w:tr>"
        ));
    }
    let docx = minimal_docx_body(&format!(
        "<w:tbl><w:tblGrid><w:gridCol w:w=\"4000\"/></w:tblGrid>{rows}</w:tbl>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>"
    ));
    let pdf = docx_to_pdf(&docx).expect("convert table rows");
    assert_eq!(
        pdf_page_count(&pdf),
        1,
        "30 exact-18pt rows must stay on one Letter page, got {}",
        pdf_page_count(&pdf)
    );
}

#[test]
fn table_tr_height_at_least_single_line_matches_soffice_row() {
    // Median lock: meeting_agenda / q1_sales / employee_directory / …
    // tblW=9360, 3×gridCol=3120, trHeight atLeast 360, empty Normal,
    // docDefaults after=200 line=276. Soffice row rules are 25.2–26.1pt;
    // we emit 11*1.15+8=20.65. Raising every single-line pad would spill
    // sample_document (no trHeight) and comments (44 rows, no trHeight).
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:docDefaults><w:pPrDefault><w:pPr>\
             <w:spacing w:after=\"200\" w:line=\"276\" w:lineRule=\"auto\"/>\
           </w:pPr></w:pPrDefault></w:docDefaults>\
           <w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
             <w:name w:val=\"Normal\"/>\
           </w:style>\
         </w:styles>";
    let mut rows = String::new();
    for label in ["Time", "09:00", "10:00", "11:00"] {
        rows.push_str(&format!(
            "<w:tr><w:trPr><w:trHeight w:val=\"360\" w:hRule=\"atLeast\"/></w:trPr>\
               <w:tc><w:tcPr><w:tcW w:w=\"3120\" w:type=\"dxa\"/></w:tcPr>\
                 <w:p><w:r><w:t>{label}</w:t></w:r></w:p></w:tc>\
               <w:tc><w:tcPr><w:tcW w:w=\"3120\" w:type=\"dxa\"/></w:tcPr>\
                 <w:p><w:r><w:t>Topic</w:t></w:r></w:p></w:tc>\
               <w:tc><w:tcPr><w:tcW w:w=\"3120\" w:type=\"dxa\"/></w:tcPr>\
                 <w:p><w:r><w:t>Who</w:t></w:r></w:p></w:tc></w:tr>"
        ));
    }
    let body = format!(
        "<w:tbl><w:tblPr><w:tblW w:w=\"9360\" w:type=\"dxa\"/>\
           <w:tblBorders>\
             <w:top w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:left w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:bottom w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:right w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:insideH w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:insideV w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
           </w:tblBorders></w:tblPr>\
           <w:tblGrid><w:gridCol w:w=\"3120\"/><w:gridCol w:w=\"3120\"/>\
             <w:gridCol w:w=\"3120\"/></w:tblGrid>{rows}</w:tbl>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>"
    );
    let pdf = docx_to_pdf(&numbering_docx_with_styles(&body, None, Some(styles)))
        .expect("convert atLeast table");
    let ys = pdf_horiz_rule_ys(&pdf);
    assert!(
        ys.len() >= 5,
        "4 rows must stroke 5 unique horizontal rules, ys={ys:?}"
    );
    let gaps: Vec<f32> = ys.windows(2).map(|w| w[0] - w[1]).collect();
    for gap in &gaps {
        assert!(
            (24.5..=27.0).contains(gap),
            "atLeast-360 single-line rows must be ~25.6pt like soffice, not 20.65; gaps={gaps:?}"
        );
    }
}

#[test]
fn table_default_cell_left_is_word_108_twips() {
    // Median lock: meeting_agenda / q1_sales / employee_directory / …
    // no tblStyle, no tblCellMar. Word default tcMar left is 108 twips
    // (5.4pt). Soffice paints "Time" at x≈77.2; we inset only 4pt (x=76).
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:docDefaults><w:pPrDefault><w:pPr>\
             <w:spacing w:after=\"200\" w:line=\"276\" w:lineRule=\"auto\"/>\
           </w:pPr></w:pPrDefault></w:docDefaults>\
           <w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
             <w:name w:val=\"Normal\"/>\
           </w:style>\
         </w:styles>";
    let body = "<w:p><w:pPr><w:spacing w:line=\"276\"/></w:pPr>\
           <w:r><w:t>Meeting Agenda</w:t></w:r></w:p>\
         <w:p/>\
         <w:tbl><w:tblPr><w:tblW w:w=\"9360\" w:type=\"dxa\"/>\
           <w:tblBorders>\
             <w:top w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:left w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:bottom w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:right w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:insideH w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:insideV w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
           </w:tblBorders></w:tblPr>\
           <w:tblGrid><w:gridCol w:w=\"3120\"/><w:gridCol w:w=\"3120\"/>\
             <w:gridCol w:w=\"3120\"/></w:tblGrid>\
           <w:tr><w:trPr><w:trHeight w:val=\"360\" w:hRule=\"atLeast\"/></w:trPr>\
             <w:tc><w:tcPr><w:tcW w:w=\"3120\" w:type=\"dxa\"/></w:tcPr>\
               <w:p><w:r><w:t>Time</w:t></w:r></w:p></w:tc>\
             <w:tc><w:tcPr><w:tcW w:w=\"3120\" w:type=\"dxa\"/></w:tcPr>\
               <w:p><w:r><w:t>Topic</w:t></w:r></w:p></w:tc>\
             <w:tc><w:tcPr><w:tcW w:w=\"3120\" w:type=\"dxa\"/></w:tcPr>\
               <w:p><w:r><w:t>Who</w:t></w:r></w:p></w:tc></w:tr>\
         </w:tbl>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&numbering_docx_with_styles(body, None, Some(styles)))
        .expect("convert default cell mar table");
    let xs = pdf_tf_xs(&pdf, "11.00 Tf");
    assert!(xs.len() >= 2, "title plus cell text must paint; xs={xs:?}");
    let title_x = xs[0];
    assert!(
        (title_x - 72.0).abs() < 1.0,
        "title stays at the left margin; xs={xs:?}"
    );
    assert!(
        xs.iter().any(|x| (76.8..=78.0).contains(x)),
        "default left cell mar is 108 twips (x≈77.4), not 4pt (x=76); xs={xs:?}"
    );
    assert!(
        xs.iter().all(|x| (*x - 76.0).abs() > 0.15),
        "must not still paint the old 4pt inset; xs={xs:?}"
    );
}

#[test]
fn table_title_empty_para_keeps_grid_on_soffice_baseline() {
    // Median / hr_onboarding cluster: title + empty <w:p/> + atLeast-360
    // table. emit_runs used em-box*1.15 (15.4pt) per body line; Word/soffice
    // auto line is size*1.15 (12.65pt). Two paras ≈ +5.6pt, so the grid
    // sits ~5pt too low (ink_f1 0.39). Soffice title→cell baseline ≈ 51.4pt.
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:docDefaults><w:pPrDefault><w:pPr>\
             <w:spacing w:after=\"200\" w:line=\"276\" w:lineRule=\"auto\"/>\
           </w:pPr></w:pPrDefault></w:docDefaults>\
           <w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
             <w:name w:val=\"Normal\"/>\
           </w:style>\
         </w:styles>";
    let body = "<w:p><w:pPr><w:spacing w:line=\"276\"/></w:pPr>\
           <w:r><w:t>Meeting Agenda</w:t></w:r></w:p>\
         <w:p/>\
         <w:tbl><w:tblPr><w:tblW w:w=\"9360\" w:type=\"dxa\"/>\
           <w:tblBorders>\
             <w:top w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:left w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:bottom w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:right w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:insideH w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:insideV w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
           </w:tblBorders></w:tblPr>\
           <w:tblGrid><w:gridCol w:w=\"3120\"/><w:gridCol w:w=\"3120\"/>\
             <w:gridCol w:w=\"3120\"/></w:tblGrid>\
           <w:tr><w:trPr><w:trHeight w:val=\"360\" w:hRule=\"atLeast\"/></w:trPr>\
             <w:tc><w:tcPr><w:tcW w:w=\"3120\" w:type=\"dxa\"/></w:tcPr>\
               <w:p><w:r><w:t>Time</w:t></w:r></w:p></w:tc>\
             <w:tc><w:tcPr><w:tcW w:w=\"3120\" w:type=\"dxa\"/></w:tcPr>\
               <w:p><w:r><w:t>Topic</w:t></w:r></w:p></w:tc>\
             <w:tc><w:tcPr><w:tcW w:w=\"3120\" w:type=\"dxa\"/></w:tcPr>\
               <w:p><w:r><w:t>Who</w:t></w:r></w:p></w:tc></w:tr>\
         </w:tbl>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&numbering_docx_with_styles(body, None, Some(styles)))
        .expect("convert title+empty+table");
    let ys = pdf_tf_ys(&pdf, "11.00 Tf");
    assert!(ys.len() >= 2, "title and cell must paint; ys={ys:?}");
    let title_y = ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let cell_y = ys
        .iter()
        .copied()
        .filter(|y| *y < title_y - 20.0)
        .fold(f32::NEG_INFINITY, f32::max);
    assert!(
        cell_y.is_finite(),
        "table cell baseline below the title; ys={ys:?}"
    );
    let gap = title_y - cell_y;
    assert!(
        (48.0..=52.5).contains(&gap),
        "title→cell baseline must match soffice ~51.4pt, not em-box stack ~53.9; gap={gap} title={title_y} cell={cell_y}"
    );
}

#[test]
fn table_cell_wrap_uses_painted_face_not_carlito_count() {
    // comments / hr wrap fallout: emit_table counted lines with wrap_plain
    // on Carlito at cw-pad, then painted with wrap_runs on the real face.
    // Courier "i" is ~3× Carlito "i". Ten "ii" tokens fit Carlito in a
    // 3120-twip cell (nlines=1, row≈20.7) but wrap in Mono (2 lines).
    // Height must follow the painted face, not the Carlito census.
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:docDefaults><w:pPrDefault><w:pPr>\
             <w:spacing w:after=\"200\" w:line=\"276\" w:lineRule=\"auto\"/>\
           </w:pPr></w:pPrDefault></w:docDefaults>\
           <w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
             <w:name w:val=\"Normal\"/>\
           </w:style>\
         </w:styles>";
    let body = "<w:tbl><w:tblPr><w:tblW w:w=\"3120\" w:type=\"dxa\"/>\
           <w:tblBorders>\
             <w:top w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:left w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:bottom w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:right w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
           </w:tblBorders></w:tblPr>\
           <w:tblGrid><w:gridCol w:w=\"3120\"/></w:tblGrid>\
           <w:tr><w:tc><w:p><w:r>\
             <w:rPr><w:rFonts w:ascii=\"Courier New\" w:hAnsi=\"Courier New\"/>\
               <w:sz w:val=\"22\"/></w:rPr>\
             <w:t>ii ii ii ii ii ii ii ii ii ii</w:t>\
           </w:r></w:p></w:tc></w:tr>\
         </w:tbl>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&numbering_docx_with_styles(body, None, Some(styles)))
        .expect("convert mono wrap cell");
    let ys = pdf_horiz_rule_ys(&pdf);
    assert!(
        ys.len() >= 2,
        "cell must stroke top and bottom rules, ys={ys:?}"
    );
    let gap = ys[0] - ys[1];
    assert!(
        (24.0..=28.0).contains(&gap),
        "Courier 30×i must wrap to 2 painted lines (~25pt), not a Carlito 1-line row (~20.7); gap={gap} ys={ys:?}"
    );
}

#[test]
fn header_jc_center_is_not_left_stuck() {
    // sample_document / eigenpal / Strict01: header1.xml is w:jc=center.
    // chrome() used to force Align::Left, so the header ink sat at the left
    // margin while soffice painted it mid-page (12 worst stems).
    let header = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:hdr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr>\
             <w:r><w:rPr><w:sz w:val=\"28\"/></w:rPr><w:t>MidHead</w:t></w:r></w:p>\
         </w:hdr>";
    let body = "<w:p><w:r><w:t>Body</w:t></w:r></w:p>\
         <w:sectPr><w:headerReference w:type=\"default\" r:id=\"rIdH1\"/>\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\" \
             w:header=\"720\"/></w:sectPr>";
    let pdf = docx_to_pdf(&hf_docx(
        body,
        &[("rIdH1", "header", "header1.xml")],
        &[("word/header1.xml", header.to_string())],
    ))
    .expect("convert centered header");
    let xs = pdf_tf_xs(&pdf, "14.00 Tf");
    assert!(!xs.is_empty(), "14pt header must paint");
    let min_x = xs.iter().copied().fold(f32::INFINITY, f32::min);
    assert!(
        min_x > 180.0,
        "w:jc=center header must sit mid-page, not at left=72; xs={xs:?}"
    );
}

#[test]
fn footer_jc_right_is_not_centered() {
    // comments / I_am_sharing cluster (8 stems): footer1.xml is w:jc=right.
    // chrome() forced Align::Center, so "Page N of M" sat mid-page while
    // soffice painted it on the right margin.
    let footer = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:ftr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:p><w:pPr><w:jc w:val=\"right\"/></w:pPr>\
             <w:r><w:rPr><w:sz w:val=\"28\"/></w:rPr><w:t>PageMark</w:t></w:r></w:p>\
         </w:ftr>";
    let body = "<w:p><w:r><w:t>Body</w:t></w:r></w:p>\
         <w:sectPr><w:footerReference w:type=\"default\" r:id=\"rIdF1\"/>\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\" \
             w:footer=\"720\"/></w:sectPr>";
    let pdf = docx_to_pdf(&hf_docx(
        body,
        &[("rIdF1", "footer", "footer1.xml")],
        &[("word/footer1.xml", footer.to_string())],
    ))
    .expect("convert right footer");
    let xs = pdf_tf_xs(&pdf, "14.00 Tf");
    assert!(!xs.is_empty(), "14pt footer must paint");
    let min_x = xs.iter().copied().fold(f32::INFINITY, f32::min);
    assert!(
        min_x > 400.0,
        "w:jc=right footer must sit near the right margin, not centered; xs={xs:?}"
    );
}

#[test]
fn superscript_is_smaller_and_raised() {
    // bold_superscript_demo (57.65) + sample/Strict01/endnotes: w:vertAlign
    // superscript. Soffice paints ~7.8pt-tall raised glyphs; we painted
    // full 11pt on the baseline.
    let body = "<w:p>\
           <w:r><w:t>x</w:t></w:r>\
           <w:r><w:rPr><w:vertAlign w:val=\"superscript\"/></w:rPr>\
             <w:t>2</w:t></w:r>\
         </w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert superscript");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("11.00 Tf"),
        "base run stays 11pt; tail {}",
        &text[text.len().saturating_sub(200)..]
    );
    let super_ys = pdf_tf_ys(&pdf, "7.15 Tf");
    assert!(
        !super_ys.is_empty(),
        "superscript must paint at 65% (7.15pt); tail {}",
        &text[text.len().saturating_sub(240)..]
    );
    let base_ys = pdf_tf_ys(&pdf, "11.00 Tf");
    assert!(!base_ys.is_empty(), "base x must paint");
    let super_y = super_ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let base_y = base_ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        super_y > base_y + 2.0,
        "superscript must sit above the baseline; super={super_y} base={base_y}"
    );
}

#[test]
fn del_and_ins_paint_revision_colors() {
    let docx = minimal_docx_body(
        "<w:p>\
           <w:del w:id=\"0\" w:author=\"a\"><w:r><w:delText>gone</w:delText></w:r></w:del>\
           <w:ins w:id=\"1\" w:author=\"a\"><w:r><w:t>fresh</w:t></w:r></w:ins>\
         </w:p><w:sectPr/>",
    );
    let pdf = docx_to_pdf(&docx).expect("convert revisions");
    let text = String::from_utf8_lossy(&pdf);
    // soffice colors by author, not by type. Same author → gold #C09000.
    assert!(
        text.contains("0.753 0.565 0.000"),
        "same-author del/ins must paint soffice gold; pdf snippet {}",
        &text[text.find("rg").unwrap_or(0)..]
            .chars()
            .take(200)
            .collect::<String>()
    );
}

#[test]
fn ins_xml_space_padding_explodes_underline() {
    // sample/eigenpal: w:ins keeps generator `xml:space` pad
    // (`This library skips it           `). Soffice explodes the
    // underline across that pad; collapsing it to one word-gap
    // shortens the mark and shifts wrap on the 8-stem cluster.
    let padded = format!(
        "<w:p><w:ins w:id=\"1\" w:author=\"a\">\
           <w:r><w:t xml:space=\"preserve\">fresh{}</w:t></w:r>\
         </w:ins></w:p><w:sectPr/>",
        " ".repeat(12)
    );
    let tight = "<w:p><w:ins w:id=\"1\" w:author=\"a\">\
           <w:r><w:t>fresh</w:t></w:r></w:ins></w:p><w:sectPr/>";
    let exploded = docx_to_pdf(&minimal_docx_body(&padded)).expect("convert padded ins");
    let compact = docx_to_pdf(&minimal_docx_body(tight)).expect("convert tight ins");
    let wide = pdf_rgb_rule_widths(&exploded, 192.0 / 255.0, 144.0 / 255.0, 0.0);
    let slim = pdf_rgb_rule_widths(&compact, 192.0 / 255.0, 144.0 / 255.0, 0.0);
    let wmax = wide.iter().copied().fold(0.0_f32, f32::max);
    let smax = slim.iter().copied().fold(0.0_f32, f32::max);
    assert!(smax > 10.0, "tight ins must still underline; slim={slim:?}");
    assert!(
        wmax > smax + 20.0,
        "ins xml:space pad must explode the underline ({wmax} vs tight {smax}); wide={wide:?}"
    );
}

#[test]
fn revision_authors_use_soffice_palette() {
    // sample/eigenpal: soffice colors tracked changes by author, not
    // by type. First author is gold #C09000, second is blue #0040A0.
    // Type-green/type-red misses every revision pixel on the cluster.
    let body = "<w:p>\
           <w:del w:id=\"0\" w:author=\"sara.k\">\
             <w:r><w:delText>gone</w:delText></w:r></w:del>\
           <w:ins w:id=\"1\" w:author=\"sara.k\">\
             <w:r><w:t>one</w:t></w:r></w:ins>\
           <w:ins w:id=\"2\" w:author=\"thomas.v\">\
             <w:r><w:t>two</w:t></w:r></w:ins>\
         </w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert authors");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("0.753 0.565 0.000"),
        "first author must paint soffice gold #C09000; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
    assert!(
        text.contains("0.000 0.251 0.627"),
        "second author must paint soffice blue #0040A0; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
    assert!(
        !text.contains("0.000 0.502 0.000"),
        "must not still use type-green insertions"
    );
    assert!(
        !text.contains("1.000 0.000 0.000 rg") && !text.contains("1.000 0.000 0.000 RG"),
        "must not still use type-red deletions"
    );
}

#[test]
fn ins_para_paints_left_revision_bar() {
    // sample/eigenpal + project_tasks: soffice strokes a changed-line
    // bar in the left margin next to every ins/del paragraph/row.
    let body = "<w:p>\
           <w:ins w:id=\"1\" w:author=\"a\">\
             <w:r><w:t>tracked</w:t></w:r></w:ins>\
         </w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert rev bar");
    let xs = pdf_vertical_rule_xs(&pdf);
    assert!(
        xs.iter().any(|x| (50.0..72.0).contains(x)),
        "ins para must stroke a bar in the left margin (x<72); xs={xs:?}"
    );
}

#[test]
fn shaded_callout_table_without_borders_is_not_stroked() {
    // comments / I_am_sharing cluster: Positioning thesis, Bottom line,
    // and Demo suggestion are 1-cell tables with w:shd and no tblBorders
    // / tblStyle. Word and soffice paint the fill only. We used to emit a
    // default black grid (`borders: None` → all four edges).
    let body = "<w:tbl>\
           <w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/><w:jc w:val=\"center\"/></w:tblPr>\
           <w:tblGrid><w:gridCol w:w=\"9000\"/></w:tblGrid>\
           <w:tr><w:tc>\
             <w:tcPr><w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"D9EAF7\"/></w:tcPr>\
             <w:p><w:r><w:t>Positioning thesis</w:t></w:r></w:p>\
           </w:tc></w:tr>\
         </w:tbl>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert shaded callout");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("0.851 0.918 0.969 rg"),
        "D9EAF7 fill must still paint; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
    assert!(
        pdf_horiz_rule_ys(&pdf).is_empty(),
        "no tblBorders must not stroke a black grid, ys={:?}",
        pdf_horiz_rule_ys(&pdf)
    );
    assert!(
        pdf_vertical_rule_xs(&pdf).is_empty(),
        "no tblBorders must not stroke verticals, xs={:?}",
        pdf_vertical_rule_xs(&pdf)
    );
}

#[test]
fn tblw_pct_sixty_stretches_narrow_grid() {
    // table_bookmark_end Tests 3–5: tblW type=pct is 50ths of a percent
    // (3000 = 60%). We used only tblGrid (here 2000+2000 twips = 200pt)
    // and never stretched, so a 60% table sat at 72+200=272 instead of
    // 72+0.6*468=352.8.
    let body = "<w:tbl><w:tblPr>\
           <w:tblW w:w=\"3000\" w:type=\"pct\"/>\
           <w:tblBorders>\
             <w:top w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:left w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:bottom w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:right w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
           </w:tblBorders></w:tblPr>\
           <w:tblGrid><w:gridCol w:w=\"2000\"/><w:gridCol w:w=\"2000\"/></w:tblGrid>\
           <w:tr>\
             <w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc>\
             <w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc>\
           </w:tr></w:tbl>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert pct table");
    let xs = pdf_vertical_rule_xs(&pdf);
    assert!(
        xs.iter().any(|x| (350.0..=356.0).contains(x)),
        "60% of 468pt content is 280.8pt so right edge is 352.8, not grid 272; xs={xs:?}"
    );
    assert!(
        xs.iter().all(|x| (*x - 272.0).abs() > 1.0),
        "must not still paint the 200pt grid edge at 272; xs={xs:?}"
    );
}

fn table_grid_line240_styles() -> String {
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
             <w:name w:val=\"Normal\"/></w:style>\
           <w:style w:type=\"paragraph\" w:styleId=\"Heading2\">\
             <w:name w:val=\"heading 2\"/>\
             <w:pPr><w:spacing w:before=\"200\" w:after=\"0\"/></w:pPr>\
             <w:rPr><w:b/><w:sz w:val=\"26\"/></w:rPr></w:style>\
           <w:style w:type=\"table\" w:styleId=\"TableGrid\">\
             <w:pPr><w:spacing w:after=\"0\" w:line=\"240\" w:lineRule=\"auto\"/></w:pPr>\
             <w:tblPr><w:tblBorders>\
               <w:top w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
               <w:left w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
               <w:bottom w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
               <w:right w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
               <w:insideH w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
               <w:insideV w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             </w:tblBorders></w:tblPr></w:style>\
         </w:styles>"
        .into()
}

fn three_row_grid_table() -> String {
    let mut rows = String::new();
    for label in ["R1", "R2", "R3"] {
        rows.push_str(&format!(
            "<w:tr><w:tc><w:p><w:r><w:t>{label}</w:t></w:r></w:p></w:tc></w:tr>"
        ));
    }
    format!(
        "<w:tbl><w:tblPr><w:tblStyle w:val=\"TableGrid\"/></w:tblPr>\
           <w:tblGrid><w:gridCol w:w=\"3000\"/></w:tblGrid>{rows}</w:tbl>"
    )
}

#[test]
fn table_grid_line240_single_row_is_tighter_than_line276_chrome() {
    // table_bookmark_end TableGrid is line=240 (1.0×font). The +8pt
    // single-line chrome was measured on line=276 meeting_agenda rows
    // (11*1.15+8=20.65). Applying it here makes 19pt rows; soffice is
    // ~16pt, which spills Tests 6–7 onto page 2.
    let body = format!(
        "{}<w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>",
        three_row_grid_table()
    );
    let pdf = docx_to_pdf(&docx_with_styles(&body, &table_grid_line240_styles()))
        .expect("convert line240 grid");
    let ys = pdf_horiz_rule_ys(&pdf);
    assert!(
        ys.len() >= 4,
        "3 rows must stroke 4 unique horizontal rules, ys={ys:?}"
    );
    let gaps: Vec<f32> = ys.windows(2).map(|w| w[0] - w[1]).collect();
    for gap in &gaps {
        assert!(
            (14.5..=17.0).contains(gap),
            "TableGrid line=240 single-line rows must be ~16pt, not 11+8=19; gaps={gaps:?}"
        );
    }
}

#[test]
fn multi_para_footer_stacks_title_and_page() {
    // sd_2517 / complex_style_attr: footer is an empty spacer <w:p>, then
    // "Smith Family Trust", then PAGE. Flattening those runs paints
    // "Trust106" on one baseline; soffice stacks title above the page no.
    let footer = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:ftr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr></w:p>\
           <w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr>\
             <w:r><w:rPr><w:sz w:val=\"26\"/></w:rPr><w:t>FamilyTrust</w:t></w:r></w:p>\
           <w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr>\
             <w:r><w:rPr><w:sz w:val=\"26\"/></w:rPr><w:t>PageSeven</w:t></w:r></w:p>\
         </w:ftr>";
    let body = "<w:p><w:r><w:t>Body</w:t></w:r></w:p>\
         <w:sectPr><w:footerReference w:type=\"default\" r:id=\"rIdF1\"/>\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\" \
             w:footer=\"720\"/></w:sectPr>";
    let pdf = docx_to_pdf(&hf_docx(
        body,
        &[("rIdF1", "footer", "footer1.xml")],
        &[("word/footer1.xml", footer.to_string())],
    ))
    .expect("convert stacked footer");
    let ys = pdf_tf_ys(&pdf, "13.00 Tf");
    let unique: std::collections::BTreeSet<i32> =
        ys.iter().map(|y| (*y * 2.0).round() as i32).collect();
    assert!(
        unique.len() >= 2,
        "title and page-no paras must sit on two baselines, not Trust106; ys={ys:?}"
    );
}

#[test]
fn table_grid_heading_pairs_fit_seven_on_one_letter_page() {
    // table_bookmark_end: soffice keeps Tests 1–7 on page 1 (1800-twip)
    // margins, Heading2 before=200, 3-row TableGrid). 19pt rows spill
    // Tests 6–7.
    let mut body = String::from(
        "<w:p><w:r><w:rPr><w:sz w:val=\"52\"/></w:rPr><w:t>Table Widths</w:t></w:r></w:p>\
         <w:p><w:r><w:t>Intro line about explicit table widths.</w:t></w:r></w:p>",
    );
    for i in 1..=7 {
        body.push_str(&format!(
            "<w:p><w:pPr><w:pStyle w:val=\"Heading2\"/></w:pPr>\
               <w:r><w:t>Test {i}</w:t></w:r></w:p>"
        ));
        body.push_str(&three_row_grid_table());
    }
    body.push_str(
        "<w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1800\" w:right=\"1800\" w:bottom=\"1800\" w:left=\"1800\"/></w:sectPr>",
    );
    let pdf = docx_to_pdf(&docx_with_styles(&body, &table_grid_line240_styles()))
        .expect("convert seven grid pairs");
    assert_eq!(
        pdf_page_count(&pdf),
        1,
        "7 Heading2+3-row TableGrid pairs must stay on one Letter page like soffice, got {}",
        pdf_page_count(&pdf)
    );
}

#[test]
fn table_grid_wrapped_header_keeps_cell_chrome() {
    // comments-addition capability matrix: TableGrid line=240, 2505-twip
    // cols, Normal=Aptos 10.5. "Google Docs expectation" wraps to 2 lines.
    // nlines>1 used to drop the +8pt chrome (22pt row). soffice is ~30pt
    // (2×11+8), which is why the last three matrix rows spill to page 11.
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
             <w:name w:val=\"Normal\"/>\
             <w:rPr><w:rFonts w:ascii=\"Aptos\" w:hAnsi=\"Aptos\"/>\
               <w:sz w:val=\"21\"/></w:rPr></w:style>\
           <w:style w:type=\"table\" w:styleId=\"TableGrid\">\
             <w:pPr><w:spacing w:after=\"0\" w:line=\"240\" w:lineRule=\"auto\"/></w:pPr>\
             <w:tblPr><w:tblBorders>\
               <w:top w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
               <w:left w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
               <w:bottom w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
               <w:right w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
               <w:insideH w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
               <w:insideV w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             </w:tblBorders></w:tblPr></w:style>\
         </w:styles>";
    let header = [
        "Capability",
        "Google Docs expectation",
        "Microsoft Word answer",
        "Word advantage",
    ];
    let mut cells = String::new();
    for text in header {
        cells.push_str(&format!(
            "<w:tc><w:p><w:r><w:rPr><w:b/></w:rPr><w:t>{text}</w:t></w:r></w:p></w:tc>"
        ));
    }
    let body = format!(
        "<w:tbl><w:tblPr><w:tblStyle w:val=\"TableGrid\"/></w:tblPr>\
           <w:tblGrid>\
             <w:gridCol w:w=\"2505\"/><w:gridCol w:w=\"2505\"/>\
             <w:gridCol w:w=\"2508\"/><w:gridCol w:w=\"2552\"/>\
           </w:tblGrid>\
           <w:tr>{cells}</w:tr></w:tbl>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"936\" w:right=\"1080\" w:bottom=\"936\" w:left=\"1080\"\
             w:header=\"720\" w:footer=\"720\"/></w:sectPr>"
    );
    let pdf = docx_to_pdf(&docx_with_styles(&body, styles)).expect("convert wrapped header");
    let ys = pdf_horiz_rule_ys(&pdf);
    assert!(
        ys.len() >= 2,
        "header row must stroke top and bottom rules, ys={ys:?}"
    );
    let gap = ys[0] - ys[1];
    assert!(
        (26.0..=34.0).contains(&gap),
        "2-line TableGrid header must keep +8pt chrome (~30pt), not 2×11=22; gap={gap} ys={ys:?}"
    );
}
